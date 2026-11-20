//! Lite membership AIR. Degree 2. Eight levels, padded to 32 rows.
//!
//! Proves selector routing only:
//!   mixed = bit * sibling + (1 - bit) * acc
//!   next  = real * mixed + (1 - real) * acc
//! Bits and selectors are boolean. Once a selector is 0, later selectors
//! stay 0. The leaf, the final accumulator, and the number of real
//! levels are public.
//!
//! Not Poseidon2. Not the production spend AIR. Not a depth-24 anonymity
//! set. A linear mix is not a collision-resistant hash.

use winterfell::crypto::{hashers::Rp64_256, DefaultRandomCoin, MerkleTree};
use winterfell::math::fields::f64::BaseElement;
use winterfell::math::{FieldElement, ToElements};
use winterfell::matrix::ColMatrix;
use winterfell::{
    Air, AirContext, Assertion, AuxRandElements, BatchingMethod, CompositionPoly,
    CompositionPolyTrace, ConstraintCompositionCoefficients, DefaultConstraintCommitment,
    DefaultConstraintEvaluator, DefaultTraceLde, EvaluationFrame, FieldExtension, PartitionOptions,
    Proof, ProofOptions, Prover, StarkDomain, TraceInfo, TracePolyTable, TraceTable,
    TransitionConstraintDegree,
};

const TRACE_LEN: usize = 32;
pub const LITE_LEVELS: usize = 8;
const WIDTH: usize = 15;
const N_CONSTRAINTS: usize = 12;
const N_ASSERTIONS: usize = 10;

const ACC: usize = 0;
const SIB: usize = 4;
const BIT: usize = 8;
const REAL: usize = 9;
const MIX: usize = 10;
const COUNT: usize = 14;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LitePublic {
    pub leaf: [BaseElement; 4],
    pub root: [BaseElement; 4],
    pub depth: BaseElement,
}

impl ToElements<BaseElement> for LitePublic {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.leaf.to_vec();
        out.extend_from_slice(&self.root);
        out.push(self.depth);
        out
    }
}

fn lite_options() -> ProofOptions {
    ProofOptions::new(
        16,
        8,
        4,
        FieldExtension::None,
        4,
        7,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    )
}

fn mix_lane(acc: BaseElement, sib: BaseElement, bit: BaseElement) -> BaseElement {
    bit * sib + (BaseElement::ONE - bit) * acc
}

fn route_lane(acc: BaseElement, mixed: BaseElement, real: BaseElement) -> BaseElement {
    real * mixed + (BaseElement::ONE - real) * acc
}

/// Walk `depth` real levels. Selectors are 1 then 0. Linear mix, not a hash.
pub fn walk(
    leaf: [BaseElement; 4],
    siblings: &[[BaseElement; 4]],
    bits: &[BaseElement],
) -> Result<[BaseElement; 4], &'static str> {
    if siblings.len() != bits.len() || siblings.len() > LITE_LEVELS {
        return Err("depth");
    }
    let mut acc = leaf;
    for (sib, bit) in siblings.iter().zip(bits) {
        let mut next = [BaseElement::ZERO; 4];
        for lane in 0..4 {
            let mixed = mix_lane(acc[lane], sib[lane], *bit);
            next[lane] = route_lane(acc[lane], mixed, BaseElement::ONE);
        }
        acc = next;
    }
    Ok(acc)
}

pub fn fill_lite_trace(
    leaf: [BaseElement; 4],
    siblings: &[[BaseElement; 4]],
    bits: &[BaseElement],
) -> Result<TraceTable<BaseElement>, &'static str> {
    if siblings.len() != bits.len() || siblings.len() > LITE_LEVELS {
        return Err("depth");
    }
    let mut rows = vec![[BaseElement::ZERO; WIDTH]; TRACE_LEN];
    let mut acc = leaf;
    let mut count = BaseElement::ZERO;
    for step in 0..TRACE_LEN {
        let real = if step < siblings.len() {
            BaseElement::ONE
        } else {
            BaseElement::ZERO
        };
        let (sib, bit) = if step < siblings.len() {
            (siblings[step], bits[step])
        } else {
            ([BaseElement::ZERO; 4], BaseElement::ZERO)
        };
        rows[step][ACC..ACC + 4].copy_from_slice(&acc);
        rows[step][SIB..SIB + 4].copy_from_slice(&sib);
        rows[step][BIT] = bit;
        rows[step][REAL] = real;
        let mut mixed = [BaseElement::ZERO; 4];
        let mut next = [BaseElement::ZERO; 4];
        for lane in 0..4 {
            mixed[lane] = mix_lane(acc[lane], sib[lane], bit);
            next[lane] = route_lane(acc[lane], mixed[lane], real);
        }
        rows[step][MIX..MIX + 4].copy_from_slice(&mixed);
        rows[step][COUNT] = count;
        acc = next;
        count += real;
    }
    let mut trace = TraceTable::new(WIDTH, TRACE_LEN);
    let first = rows[0];
    trace.fill(
        |row| row.copy_from_slice(&first),
        |step, row| row.copy_from_slice(&rows[step + 1]),
    );
    Ok(trace)
}

pub struct LiteAir {
    context: AirContext<BaseElement>,
    public: LitePublic,
}

impl Air for LiteAir {
    type BaseField = BaseElement;
    type PublicInputs = LitePublic;

    fn new(trace_info: TraceInfo, public: Self::PublicInputs, options: ProofOptions) -> Self {
        // Count step is degree 1. The rest are degree 2 (products).
        let mut degrees = vec![TransitionConstraintDegree::new(2); N_CONSTRAINTS - 1];
        degrees.push(TransitionConstraintDegree::new(1));
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
        let _ = &self.public;
        let cur = frame.current();
        let next = frame.next();
        let bit = cur[BIT];
        let real = cur[REAL];
        result[0] = bit * (bit - E::ONE);
        result[1] = real * (real - E::ONE);
        result[2] = (E::ONE - real) * next[REAL];
        for lane in 0..4 {
            let acc = cur[ACC + lane];
            let sib = cur[SIB + lane];
            let mixed = bit * sib + (E::ONE - bit) * acc;
            result[3 + lane] = cur[MIX + lane] - mixed;
            let out = real * cur[MIX + lane] + (E::ONE - real) * acc;
            result[7 + lane] = next[ACC + lane] - out;
        }
        result[11] = next[COUNT] - cur[COUNT] - real;
    }

    fn get_assertions(&self) -> Vec<Assertion<BaseElement>> {
        let mut a = Vec::with_capacity(N_ASSERTIONS);
        for lane in 0..4 {
            a.push(Assertion::single(ACC + lane, 0, self.public.leaf[lane]));
            a.push(Assertion::single(
                ACC + lane,
                TRACE_LEN - 1,
                self.public.root[lane],
            ));
        }
        a.push(Assertion::single(COUNT, 0, BaseElement::ZERO));
        a.push(Assertion::single(COUNT, TRACE_LEN - 1, self.public.depth));
        a
    }
}

struct LiteProver {
    options: ProofOptions,
}

impl Prover for LiteProver {
    type BaseField = BaseElement;
    type Air = LiteAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> LitePublic {
        LitePublic {
            leaf: [0, 1, 2, 3].map(|i| trace.get(ACC + i, 0)),
            root: [0, 1, 2, 3].map(|i| trace.get(ACC + i, TRACE_LEN - 1)),
            depth: trace.get(COUNT, TRACE_LEN - 1),
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

pub fn prove_lite(
    leaf: [BaseElement; 4],
    siblings: &[[BaseElement; 4]],
    bits: &[BaseElement],
) -> Result<(Proof, LitePublic), &'static str> {
    let root = walk(leaf, siblings, bits)?;
    let trace = fill_lite_trace(leaf, siblings, bits)?;
    let prover = LiteProver {
        options: lite_options(),
    };
    let public = prover.get_pub_inputs(&trace);
    if public.root != root || public.depth != BaseElement::new(siblings.len() as u64) {
        return Err("public");
    }
    let proof = prover.prove(trace).map_err(|_| "prove")?;
    Ok((proof, public))
}

pub fn verify_lite(proof: Proof, public: LitePublic) -> Result<(), winterfell::VerifierError> {
    let opts = lite_options();
    if proof.options() != &opts {
        return Err(winterfell::VerifierError::UnacceptableProofOptions);
    }
    let acceptable = winterfell::AcceptableOptions::OptionSet(vec![opts]);
    winterfell::verify::<LiteAir, Rp64_256, DefaultRandomCoin<Rp64_256>, MerkleTree<Rp64_256>>(
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

    fn path(depth: usize) -> (Vec<[BaseElement; 4]>, Vec<BaseElement>) {
        let siblings = (0..depth)
            .map(|i| [fe(10 + i as u64), fe(0), fe(0), fe(0)])
            .collect();
        let bits = (0..depth).map(|i| fe((i % 2) as u64)).collect();
        (siblings, bits)
    }

    #[test]
    fn walk_depth3_and_depth8() {
        let leaf = [fe(1), fe(2), fe(3), fe(4)];
        for depth in [3usize, 4, 8] {
            let (sib, bits) = path(depth);
            let root = walk(leaf, &sib, &bits).unwrap();
            let mut expect = leaf;
            for (s, b) in sib.iter().zip(&bits) {
                let mut next = [BaseElement::ZERO; 4];
                for lane in 0..4 {
                    next[lane] =
                        route_lane(expect[lane], mix_lane(expect[lane], s[lane], *b), fe(1));
                }
                expect = next;
            }
            assert_eq!(root, expect);
        }
        assert!(walk(leaf, &[[fe(1); 4]; 9], &[fe(0); 9]).is_err());
    }

    #[test]
    fn prove_and_verify_lite_depth3() {
        let leaf = [fe(1), fe(2), fe(3), fe(4)];
        let (sib, bits) = path(3);
        let (proof, public) = prove_lite(leaf, &sib, &bits).expect("prove");
        assert_eq!(public.depth, fe(3));
        verify_lite(proof, public.clone()).expect("verify");
        let mut bad = public;
        bad.root[0] += fe(1);
        let (proof, _) = prove_lite(leaf, &sib, &bits).expect("prove");
        assert!(verify_lite(proof, bad).is_err());
    }
}
