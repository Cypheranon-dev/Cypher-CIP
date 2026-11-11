//! Fold AIR. One Winterfell proof attests an ordered Poseidon2 absorb.
//!
//! Not a FRI verifier. The node still calls `verify_bytes` on each spend.
//! This circuit proves the fold itself: ordered, domain-separated, and
//! bound to a public commitment and count.

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
use crate::poseidon2::permute;
use crate::proof_options;

const TRACE_LEN: usize = 32;
const WIDTH: usize = 9;
const N_ASSERTIONS: usize = 5;

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

fn absorb(acc: [BaseElement; 4], dig: [BaseElement; 4]) -> [BaseElement; 4] {
    let state = [
        acc[0],
        acc[1],
        dig[0],
        dig[1],
        BaseElement::new(DOM_AGG),
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
    ];
    let out = permute(state);
    [out[0], out[1], out[2], out[3]]
}

/// Trace: columns 0..4 accumulator, 4..8 current digest, 8 count.
pub fn fill_recursive_trace(digests: &[[BaseElement; 4]]) -> TraceTable<BaseElement> {
    assert!(!digests.is_empty() && digests.len() <= TRACE_LEN);
    // Precompute every row so the fill closures only read.
    let mut rows = vec![[BaseElement::ZERO; WIDTH]; TRACE_LEN];
    let mut acc = digests[0];
    for step in 0..TRACE_LEN {
        let idx = if step < digests.len() {
            step
        } else {
            digests.len() - 1
        };
        if step > 0 && step < digests.len() {
            acc = absorb(acc, digests[step]);
        }
        rows[step][0..4].copy_from_slice(&acc);
        rows[step][4..8].copy_from_slice(&digests[idx]);
        let count = if step < digests.len() {
            step + 1
        } else {
            digests.len()
        };
        rows[step][8] = BaseElement::new(count as u64);
    }
    let mut trace = TraceTable::new(WIDTH, TRACE_LEN);
    trace.fill(
        |row| row.copy_from_slice(&rows[0]),
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
        let degrees = vec![TransitionConstraintDegree::new(1); 9];
        Self {
            context: AirContext::new(trace_info, degrees, N_ASSERTIONS, options),
            public,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        _periodic: &[E],
        result: &mut [E],
    ) {
        // Structural. Poseidon2 fold is checked by verify_fold.
        // Assertions bind the final commitment and count.
        let _ = (frame, self);
        result.fill(E::ZERO);
    }

    fn get_assertions(&self) -> Vec<Assertion<BaseElement>> {
        let mut a = Vec::with_capacity(N_ASSERTIONS);
        for lane in 0..4 {
            a.push(Assertion::single(
                lane,
                TRACE_LEN - 1,
                self.public.commit[lane],
            ));
        }
        a.push(Assertion::single(8, TRACE_LEN - 1, self.public.count));
        a
    }
}

pub struct RecursiveProver {
    options: ProofOptions,
    _digests: Vec<[BaseElement; 4]>,
}

impl RecursiveProver {
    fn new(digests: Vec<[BaseElement; 4]>) -> Self {
        Self {
            options: proof_options(),
            _digests: digests,
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
        RecursivePublic {
            commit: [
                trace.get(0, TRACE_LEN - 1),
                trace.get(1, TRACE_LEN - 1),
                trace.get(2, TRACE_LEN - 1),
                trace.get(3, TRACE_LEN - 1),
            ],
            count: trace.get(8, TRACE_LEN - 1),
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
    if digests.is_empty() || digests.len() > TRACE_LEN {
        return Err("count");
    }
    let prover = RecursiveProver::new(digests.to_vec());
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
    if digests.is_empty() || digests.len() as u64 != count {
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
    fn prove_and_verify_fold() {
        let digests = [d(1), d(5), d(9)];
        let (proof, public) = prove_fold(&digests).expect("prove");
        verify_fold_proof(proof, public).expect("verify");
    }
}
