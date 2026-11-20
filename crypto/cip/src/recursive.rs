//! Fold AIR. Each extra digest is one Poseidon2 permutation (32 rows).
//!
//! Not a FRI verifier. The node still calls `verify_bytes` on each spend.
//! The absorb matches `aggregate::fold`: lanes 0 and 1 of the accumulator
//! and of the next digest, then `DOM_AGG`. The proof binds the commitment
//! and the count. The digest list itself is still checked by `verify_fold`.
//! At most four absorbs (five digests).

use winterfell::crypto::{hashers::Rp64_256, DefaultRandomCoin, MerkleTree};
use winterfell::math::fields::f64::BaseElement;
use winterfell::math::{FieldElement, ToElements};
use winterfell::matrix::ColMatrix;
use winterfell::{
    Air, AirContext, Assertion, AuxRandElements, ConstraintCompositionCoefficients,
    DefaultConstraintCommitment, DefaultConstraintEvaluator, DefaultTraceLde, EvaluationFrame,
    PartitionOptions, Proof, ProofOptions, Prover, StarkDomain, TraceInfo, TracePolyTable,
    TraceTable, TransitionConstraintDegree,
};
use winterfell::{CompositionPoly, CompositionPolyTrace};

use crate::aggregate::DOM_AGG;
use crate::poseidon2::{full_round, linear_layer, partial_round, permute, slot_step, SLOT_ROWS};
use crate::proof_options;

const TRACE_LEN: usize = 128;
const SLOTS: usize = TRACE_LEN / SLOT_ROWS;
pub const MAX_DIGESTS: usize = SLOTS + 1;
const WIDTH: usize = 10;
const COUNT: usize = 8;
const ANCHOR: usize = 9;
const N_CONSTRAINTS: usize = 14;
const N_ASSERTIONS: usize = 6;

const P_LINEAR: usize = 0;
const P_FULL: usize = 1;
const P_PARTIAL: usize = 2;
const P_BOUNDARY: usize = 3;
const P_RC: usize = 4;
const N_PERIODIC: usize = 12;

/// Public inputs: the fold commitment and the number of digests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecursivePublic {
    pub commit: [BaseElement; 4],
    pub count: BaseElement,
}

impl ToElements<BaseElement> for RecursivePublic {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.commit.to_vec();
        out.push(self.count);
        out
    }
}

fn fold_input(acc: [BaseElement; 4], dig: [BaseElement; 4]) -> [BaseElement; 8] {
    [
        acc[0],
        acc[1],
        dig[0],
        dig[1],
        BaseElement::new(DOM_AGG),
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
    ]
}

fn absorb(acc: [BaseElement; 4], dig: [BaseElement; 4]) -> [BaseElement; 4] {
    let out = permute(fold_input(acc, dig));
    [out[0], out[1], out[2], out[3]]
}

fn commit_row(count: usize) -> usize {
    if count <= 1 {
        0
    } else {
        (count - 1) * SLOT_ROWS - 1
    }
}

fn apply_round(local: usize, state: [BaseElement; 8]) -> [BaseElement; 8] {
    let flags = slot_step(local);
    if flags.linear {
        return linear_layer(state);
    }
    if flags.full {
        return full_round(state, flags.rc.map(BaseElement::new));
    }
    if flags.partial {
        return partial_round(state, BaseElement::new(flags.rc[0]));
    }
    state
}

fn boundary_next(state: [BaseElement; 8], dig: [BaseElement; 4]) -> [BaseElement; 8] {
    fold_input([state[0], state[1], state[2], state[3]], dig)
}

/// Trace: columns 0..8 Poseidon state, 8 count, 9 anchor (1 only on row 0).
pub fn fill_recursive_trace(digests: &[[BaseElement; 4]]) -> TraceTable<BaseElement> {
    assert!(!digests.is_empty() && digests.len() <= MAX_DIGESTS);
    let mut rows = vec![[BaseElement::ZERO; WIDTH]; TRACE_LEN];
    let count = BaseElement::new(digests.len() as u64);
    if digests.len() == 1 {
        rows[0][0..4].copy_from_slice(&digests[0]);
    } else {
        rows[0][..8].copy_from_slice(&fold_input(digests[0], digests[1]));
    }
    rows[0][COUNT] = count;
    rows[0][ANCHOR] = BaseElement::ONE;
    let mut state = [BaseElement::ZERO; 8];
    state.copy_from_slice(&rows[0][..8]);
    for step in 0..TRACE_LEN - 1 {
        let local = step % SLOT_ROWS;
        let next = if local + 1 == SLOT_ROWS {
            let slot = step / SLOT_ROWS;
            let dig = if slot + 2 < digests.len() {
                digests[slot + 2]
            } else {
                [BaseElement::ZERO; 4]
            };
            boundary_next(state, dig)
        } else {
            apply_round(local, state)
        };
        rows[step + 1][..8].copy_from_slice(&next);
        rows[step + 1][COUNT] = count;
        state = next;
    }
    let mut trace = TraceTable::new(WIDTH, TRACE_LEN);
    let first = rows[0];
    trace.fill(
        |row| row.copy_from_slice(&first),
        |step, row| row.copy_from_slice(&rows[step + 1]),
    );
    trace
}

pub struct RecursiveAir {
    context: AirContext<BaseElement>,
    public: RecursivePublic,
}

impl Air for RecursiveAir {
    type BaseField = BaseElement;
    type PublicInputs = RecursivePublic;

    fn new(trace_info: TraceInfo, public: Self::PublicInputs, options: ProofOptions) -> Self {
        // Round is degree 7. A periodic round flag and (1 - boundary), also
        // periodic, both multiply it. Two cycles of 32. Blowup stays 8.
        // A trace selector here would raise the base degree past blowup 8.
        let round = TransitionConstraintDegree::with_cycles(7, vec![SLOT_ROWS, SLOT_ROWS]);
        let mut degrees = vec![round; 8];
        degrees.push(TransitionConstraintDegree::new(1));
        degrees.push(TransitionConstraintDegree::new(1));
        // count is constant, so anchor * (count - 1) * lane is degree 2.
        degrees.extend(vec![TransitionConstraintDegree::new(2); 4]);
        debug_assert_eq!(degrees.len(), N_CONSTRAINTS);
        Self {
            context: AirContext::new(trace_info, degrees, N_ASSERTIONS, options),
            public,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<BaseElement>> {
        let mut columns = vec![vec![BaseElement::ZERO; SLOT_ROWS]; N_PERIODIC];
        let flags: Vec<_> = (0..SLOT_ROWS).map(slot_step).collect();
        let on = |flag: bool| {
            if flag {
                BaseElement::ONE
            } else {
                BaseElement::ZERO
            }
        };
        columns[P_LINEAR] = flags.iter().map(|f| on(f.linear)).collect();
        columns[P_FULL] = flags.iter().map(|f| on(f.full)).collect();
        columns[P_PARTIAL] = flags.iter().map(|f| on(f.partial)).collect();
        columns[P_BOUNDARY] = flags.iter().map(|f| on(f.boundary[0])).collect();
        for lane in 0..8 {
            columns[P_RC + lane] = flags.iter().map(|f| BaseElement::new(f.rc[lane])).collect();
        }
        columns
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic: &[E],
        result: &mut [E],
    ) {
        let cur = frame.current();
        let next = frame.next();
        let state = [0, 1, 2, 3, 4, 5, 6, 7].map(|lane| cur[lane]);
        let rc = [0, 1, 2, 3, 4, 5, 6, 7].map(|lane| periodic[P_RC + lane]);
        let linear = linear_layer(state);
        let full = full_round(state, rc);
        let partial = partial_round(state, rc[0]);
        let boundary = periodic[P_BOUNDARY];
        let one = E::ONE;
        let keep = one - boundary;
        for lane in 0..8 {
            let round = periodic[P_LINEAR] * linear[lane]
                + periodic[P_FULL] * full[lane]
                + periodic[P_PARTIAL] * partial[lane];
            let load = match lane {
                0 => cur[0],
                1 => cur[1],
                2 => next[2],
                3 => next[3],
                4 => E::from(BaseElement::new(DOM_AGG)),
                _ => E::ZERO,
            };
            result[lane] = next[lane] - (keep * round + boundary * load);
        }
        result[8] = next[COUNT] - cur[COUNT];
        result[9] = next[ANCHOR];
        let opened = cur[ANCHOR] * (cur[COUNT] - one);
        result[10] = opened * (cur[4] - E::from(BaseElement::new(DOM_AGG)));
        result[11] = opened * cur[5];
        result[12] = opened * cur[6];
        result[13] = opened * cur[7];
    }

    fn get_assertions(&self) -> Vec<Assertion<BaseElement>> {
        let n = self.public.count.as_int() as usize;
        let row = if (1..=MAX_DIGESTS).contains(&n) {
            commit_row(n)
        } else {
            0
        };
        let mut a = Vec::with_capacity(N_ASSERTIONS);
        for (lane, element) in self.public.commit.iter().enumerate() {
            a.push(Assertion::single(lane, row, *element));
        }
        a.push(Assertion::single(COUNT, 0, self.public.count));
        // Anchor must be 1. Otherwise the row-0 domain separator is vacuous.
        a.push(Assertion::single(ANCHOR, 0, BaseElement::ONE));
        a
    }
}

pub struct RecursiveProver {
    options: ProofOptions,
}

impl RecursiveProver {
    fn new() -> Self {
        Self {
            options: proof_options(),
        }
    }
}

impl Prover for RecursiveProver {
    type BaseField = BaseElement;
    type Air = RecursiveAir;
    type Trace = TraceTable<BaseElement>;
    type HashFn = Rp64_256;
    type VC = MerkleTree<Self::HashFn>;
    type RandomCoin = DefaultRandomCoin<Self::HashFn>;
    type TraceLde<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultTraceLde<E, Self::HashFn, Self::VC>;
    type ConstraintCommitment<E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintCommitment<E, Self::HashFn, Self::VC>;
    type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
        DefaultConstraintEvaluator<'a, Self::Air, E>;

    fn get_pub_inputs(&self, trace: &Self::Trace) -> RecursivePublic {
        let count = trace.get(COUNT, 0);
        let n = count.as_int() as usize;
        let row = if (1..=MAX_DIGESTS).contains(&n) {
            commit_row(n)
        } else {
            0
        };
        RecursivePublic {
            commit: [0, 1, 2, 3].map(|lane| trace.get(lane, row)),
            count,
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }

    fn new_trace_lde<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        info: &TraceInfo,
        main: &ColMatrix<Self::BaseField>,
        domain: &StarkDomain<Self::BaseField>,
        part: PartitionOptions,
    ) -> (Self::TraceLde<E>, TracePolyTable<E>) {
        DefaultTraceLde::new(info, main, domain, part)
    }

    fn new_evaluator<'a, E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        air: &'a Self::Air,
        aux: Option<AuxRandElements<E>>,
        coeffs: ConstraintCompositionCoefficients<E>,
    ) -> Self::ConstraintEvaluator<'a, E> {
        DefaultConstraintEvaluator::new(air, aux, coeffs)
    }

    fn build_constraint_commitment<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        poly: CompositionPolyTrace<E>,
        n: usize,
        domain: &StarkDomain<Self::BaseField>,
        part: PartitionOptions,
    ) -> (Self::ConstraintCommitment<E>, CompositionPoly<E>) {
        DefaultConstraintCommitment::new(poly, n, domain, part)
    }
}

/// Prove a fold. Returns the proof and the public commitment.
pub fn prove_fold(digests: &[[BaseElement; 4]]) -> Result<(Proof, RecursivePublic), &'static str> {
    if digests.is_empty() || digests.len() > MAX_DIGESTS {
        return Err("count");
    }
    let prover = RecursiveProver::new();
    let trace = fill_recursive_trace(digests);
    let public = prover.get_pub_inputs(&trace);
    let proof = prover.prove(trace).map_err(|_| "prove")?;
    Ok((proof, public))
}

/// Verify a fold proof against its public inputs.
pub fn verify_fold_proof(
    proof: Proof,
    public: RecursivePublic,
) -> Result<(), winterfell::VerifierError> {
    let n = public.count.as_int() as usize;
    if n == 0 || n > MAX_DIGESTS {
        return Err(winterfell::VerifierError::ProofDeserializationError(
            "count".into(),
        ));
    }
    let opts = proof_options();
    if proof.options() != &opts {
        return Err(winterfell::VerifierError::UnacceptableProofOptions);
    }
    let acceptable = winterfell::AcceptableOptions::OptionSet(vec![opts]);
    winterfell::verify::<RecursiveAir, Rp64_256, DefaultRandomCoin<Rp64_256>, MerkleTree<Rp64_256>>(
        proof,
        public,
        &acceptable,
    )
}

/// Native Poseidon2 fold check. Used by the node before accepting a proof.
pub fn verify_fold(
    digests: &[[BaseElement; 4]],
    commit: [u64; 4],
    count: u64,
) -> Result<(), &'static str> {
    if digests.is_empty() || digests.len() as u64 != count || digests.len() > MAX_DIGESTS {
        return Err("count");
    }
    let mut acc = digests[0];
    for dig in digests.iter().skip(1) {
        acc = absorb(acc, *dig);
    }
    let got = [
        acc[0].as_int(),
        acc[1].as_int(),
        acc[2].as_int(),
        acc[3].as_int(),
    ];
    if got != commit {
        return Err("fold");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(n: u64) -> [BaseElement; 4] {
        [
            BaseElement::new(n),
            BaseElement::new(n + 1),
            BaseElement::new(n + 2),
            BaseElement::new(n + 3),
        ]
    }

    #[test]
    fn fold_roundtrip_and_reorder() {
        let digests = [d(1), d(5)];
        let mut acc = digests[0];
        acc = absorb(acc, digests[1]);
        let commit = [
            acc[0].as_int(),
            acc[1].as_int(),
            acc[2].as_int(),
            acc[3].as_int(),
        ];
        assert!(verify_fold(&digests, commit, 2).is_ok());
        assert!(verify_fold(&[digests[1], digests[0]], commit, 2).is_err());
    }

    #[test]
    fn trace_matches_absorb() {
        let digests = [d(1), d(5), d(9)];
        let trace = fill_recursive_trace(&digests);
        let mut acc = digests[0];
        acc = absorb(acc, digests[1]);
        acc = absorb(acc, digests[2]);
        let row = commit_row(3);
        for (lane, element) in acc.iter().enumerate() {
            assert_eq!(trace.get(lane, row), *element);
        }
    }

    #[test]
    fn prove_and_verify_fold() {
        let digests = [d(1), d(5), d(9)];
        let (proof, public) = prove_fold(&digests).expect("prove");
        verify_fold_proof(proof, public).expect("verify");
    }
}
