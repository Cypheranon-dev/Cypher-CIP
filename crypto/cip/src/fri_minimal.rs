//! Minimal FRI fold circuit (research prototype).
//!
//! Constrains a single degree-respecting projection step on a short trace.
//! folded = e0 + α·e1 + α²·e2 + α³·e3.
//!
//! Not a FRI verifier. Does not check Merkle openings, multiple layers,
//! remainder, transcript, or an inner AIR. Native verification remains required.

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

use crate::proof_options;

const TRACE_LEN: usize = 16;
const WIDTH: usize = 13;
const N_CONSTRAINTS: usize = 4;
const N_ASSERTIONS: usize = 5;

const EVALS: usize = 0;
const ALPHA: usize = 4;
const FOLDED: usize = 8;
const COMMIT: usize = 12; // single column for simplicity in this skeleton

/// Public inputs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FriMinimalPublic {
    pub commit: [BaseElement; 4],
    pub folded: [BaseElement; 4],
}

impl ToElements<BaseElement> for FriMinimalPublic {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut v = self.commit.to_vec();
        v.extend_from_slice(&self.folded);
        v
    }
}

/// Build a constant trace that already satisfies the fold.
pub fn fill_trace(
    commit: [BaseElement; 4],
    evals: [BaseElement; 4],
    alpha0: BaseElement,
) -> TraceTable<BaseElement> {
    let mut folded = [BaseElement::ZERO; 4];
    let mut apow = BaseElement::ONE;
    for i in 0..4 {
        for lane in 0..4 {
            folded[lane] += apow * evals[i];
        }
        apow *= alpha0;
    }

    let mut rows = vec![[BaseElement::ZERO; WIDTH]; TRACE_LEN];
    for i in 0..4 {
        rows[0][EVALS + i] = evals[i];
        rows[0][FOLDED + i] = folded[i];
    }
    rows[0][ALPHA] = alpha0;
    // Store commit in a simple way (first lane only for the skeleton)
    rows[0][COMMIT] = commit[0];

    let mut trace = TraceTable::new(WIDTH, TRACE_LEN);
    let first = rows[0];
    trace.fill(
        |row| row.copy_from_slice(&first),
        |_step, row| row.copy_from_slice(&first),
    );
    trace
}

pub struct FriMinimalAir {
    context: AirContext<BaseElement>,
    public: FriMinimalPublic,
}

impl Air for FriMinimalAir {
    type BaseField = BaseElement;
    type PublicInputs = FriMinimalPublic;

    fn new(trace_info: TraceInfo, public: Self::PublicInputs, options: ProofOptions) -> Self {
        // Degree 1: next = current (constant columns).
        let degrees = vec![TransitionConstraintDegree::new(1); N_CONSTRAINTS];
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
        let cur = frame.current();
        let next = frame.next();
        // Enforce that the folded columns stay constant.
        for lane in 0..4 {
            result[lane] = next[FOLDED + lane] - cur[FOLDED + lane];
        }
    }

    fn get_assertions(&self) -> Vec<Assertion<BaseElement>> {
        let mut a = Vec::with_capacity(N_ASSERTIONS);
        for i in 0..4 {
            a.push(Assertion::single(FOLDED + i, 0, self.public.folded[i]));
        }
        a.push(Assertion::single(COMMIT, 0, self.public.commit[0]));
        a
    }
}

pub struct FriMinimalProver {
    options: ProofOptions,
}

impl FriMinimalProver {
    fn new() -> Self {
        Self { options: proof_options() }
    }
}

impl Prover for FriMinimalProver {
    type BaseField = BaseElement;
    type Air = FriMinimalAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> FriMinimalPublic {
        FriMinimalPublic {
            commit: [
                trace.get(COMMIT, 0),
                BaseElement::ZERO,
                BaseElement::ZERO,
                BaseElement::ZERO,
            ],
            folded: [0, 1, 2, 3].map(|i| trace.get(FOLDED + i, 0)),
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

pub fn prove_minimal(
    commit: [BaseElement; 4],
    evals: [BaseElement; 4],
    alpha0: BaseElement,
) -> Result<(Proof, FriMinimalPublic), &'static str> {
    let prover = FriMinimalProver::new();
    let trace = fill_trace(commit, evals, alpha0);
    let public = prover.get_pub_inputs(&trace);
    let proof = prover.prove(trace).map_err(|_| "prove")?;
    Ok((proof, public))
}

pub fn verify_minimal(
    proof: Proof,
    public: FriMinimalPublic,
) -> Result<(), winterfell::VerifierError> {
    let opts = proof_options();
    if proof.options() != &opts {
        return Err(winterfell::VerifierError::UnacceptableProofOptions);
    }
    let acceptable = winterfell::AcceptableOptions::OptionSet(vec![opts]);
    winterfell::verify::<FriMinimalAir, Rp64_256, DefaultRandomCoin<Rp64_256>, MerkleTree<Rp64_256>>(
        proof,
        public,
        &acceptable,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fe(n: u64) -> BaseElement {
        BaseElement::new(n)
    }

    #[test]
    fn satisfiable_fold_verifies() {
        let commit = [fe(1), fe(2), fe(3), fe(4)];
        let evals = [fe(10), fe(20), fe(30), fe(40)];
        let alpha0 = fe(7);
        let (proof, public) = prove_minimal(commit, evals, alpha0).expect("prove");
        verify_minimal(proof, public).expect("verify");
    }

    #[test]
    fn wrong_folded_fails() {
        let commit = [fe(1), fe(2), fe(3), fe(4)];
        let evals = [fe(10), fe(20), fe(30), fe(40)];
        let alpha0 = fe(7);
        let (proof, mut public) = prove_minimal(commit, evals, alpha0).expect("prove");
        public.folded[0] = fe(999);
        assert!(verify_minimal(proof, public).is_err());
    }
}
