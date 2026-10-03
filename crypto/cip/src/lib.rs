//! Four-statement CIP air, proved with Winterfell (FRI, no trusted setup).
//!
//! Field: Goldilocks. Blowup 8, 84 queries, grinding 20, quadratic extension.
//! Those four numbers are the §4.7 proposal. The hash in §4.7 is not.
//!
//! In-circuit hash: H(a, b) = a^3 + 3 b^3 + 7. The paper's name for that
//! role is Poseidon2 (§4.4, §4.7). This gadget is not Poseidon2, and it is
//! not collision-resistant. It is also not Rescue-Prime. Rescue-Prime is
//! the name §4.7.1 uses for a toy circuit standing in for Poseidon2.
//!
//! Winterfell's commitment hasher is Rp64_256. That is the library hasher.
//! It is not the in-circuit hash, and it is not Poseidon2.
//!
//! Statements inside the proof:
//!   miner-set membership of pk = H(sk, 1) in the public miner root (depth 4)
//!   nullifier = H(sk, r_in)
//!   amount_in = amount_out + fee
//!   event_id = H(H(payload, height), chain_state)
//!
//! "Nullifier not in the set" is a node check. A STARK cannot see the chain.
//! The field `sk` is not a Dilithium secret. Dilithium signs the envelope
//! outside this circuit.

use winterfell::crypto::{hashers::Rp64_256, DefaultRandomCoin, ElementHasher, MerkleTree};
use winterfell::math::{fields::f64::BaseElement, FieldElement, ToElements};
use winterfell::{
    Air, AirContext, Assertion, AuxRandElements, BatchingMethod, CompositionPoly,
    CompositionPolyTrace, ConstraintCompositionCoefficients, DefaultConstraintCommitment,
    DefaultConstraintEvaluator, DefaultTraceLde, EvaluationFrame, FieldExtension, PartitionOptions,
    Proof, ProofOptions, Prover, StarkDomain, TraceInfo, TracePolyTable, TraceTable,
    TransitionConstraintDegree,
};

const TRACE_LEN: usize = 16;
const WIDTH: usize = 13;
const ACC: usize = 0;
const SIB: usize = 1;
const BIT: usize = 2;
const SK: usize = 3;
const RIN: usize = 4;
const NULL: usize = 5;
const AMT_IN: usize = 6;
const AMT_OUT: usize = 7;
const FEE: usize = 8;
const HEIGHT: usize = 9;
const PAYLOAD: usize = 10;
const CHAIN: usize = 11;
const EVENT: usize = 12;
/// Domain separator so pk = H(sk, 1) is not the nullifier.
const PK_DOMAIN: u64 = 1;
/// Goldilocks prime modulus: 2^64 - 2^32 + 1.
const GOLDILOCKS_MODULUS: u64 = 0xffff_ffff_0000_0001;

pub fn fe(value: u64) -> BaseElement {
    BaseElement::new(value)
}

fn checked_fe(value: u64) -> Result<BaseElement, &'static str> {
    if value >= GOLDILOCKS_MODULUS {
        return Err("non-canonical field element");
    }
    Ok(fe(value))
}

fn h2(a: BaseElement, b: BaseElement) -> BaseElement {
    a.cube() + b.cube() * fe(3) + fe(7)
}

fn h2e<E: FieldElement<BaseField = BaseElement>>(a: E, b: E) -> E {
    let three = E::from(3u32);
    let seven = E::from(7u32);
    a.cube() + b.cube() * three + seven
}

pub fn public_key(sk: u64) -> BaseElement {
    h2(fe(sk), fe(PK_DOMAIN))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicInputs {
    pub miner_root: BaseElement,
    pub nullifier: BaseElement,
    pub event_id: BaseElement,
    pub height: BaseElement,
}

impl ToElements<BaseElement> for PublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        vec![self.miner_root, self.nullifier, self.event_id, self.height]
    }
}

impl PublicInputs {
    pub fn to_u64s(&self) -> [u64; 4] {
        [
            self.miner_root.as_int(),
            self.nullifier.as_int(),
            self.event_id.as_int(),
            self.height.as_int(),
        ]
    }

    pub fn try_from_u64s(values: [u64; 4]) -> Result<Self, &'static str> {
        Ok(Self {
            miner_root: checked_fe(values[0])?,
            nullifier: checked_fe(values[1])?,
            event_id: checked_fe(values[2])?,
            height: checked_fe(values[3])?,
        })
    }
}

#[derive(Clone)]
pub struct Witness {
    pub sk: u64,
    pub r_in: u64,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee: u64,
    pub height: u64,
    pub payload: u64,
    pub chain_state: u64,
    pub index: usize,
    pub leaves: [u64; 16],
}

pub struct Prepared {
    pub public: PublicInputs,
    siblings: [BaseElement; 4],
    bits: [BaseElement; 4],
    leaf: BaseElement,
    sk: BaseElement,
    r_in: BaseElement,
    amount_in: BaseElement,
    amount_out: BaseElement,
    fee: BaseElement,
    height: BaseElement,
    payload: BaseElement,
    chain_state: BaseElement,
}

pub fn prepare(witness: &Witness) -> Result<Prepared, &'static str> {
    if witness.index >= 16 {
        return Err("miner index out of range");
    }
    let expected_amount_in = witness
        .amount_out
        .checked_add(witness.fee)
        .ok_or("amount overflow")?;
    if witness.amount_in != expected_amount_in {
        return Err("amounts do not conserve");
    }
    let sk = checked_fe(witness.sk)?;
    let pk = h2(sk, checked_fe(PK_DOMAIN)?);
    let mut checked_leaves = [BaseElement::ZERO; 16];
    for (dst, value) in checked_leaves.iter_mut().zip(witness.leaves) {
        *dst = checked_fe(value)?;
    }
    let leaves = checked_leaves;
    if leaves[witness.index] != pk {
        return Err("miner leaf is not H(sk, 1)");
    }
    let leaf = leaves[witness.index];
    let mut level = leaves.to_vec();
    let mut siblings = [fe(0); 4];
    let mut bits = [fe(0); 4];
    let mut idx = witness.index;
    for depth in 0..4 {
        let sibling = if idx.is_multiple_of(2) {
            level[idx + 1]
        } else {
            level[idx - 1]
        };
        siblings[depth] = sibling;
        bits[depth] = if idx.is_multiple_of(2) {
            BaseElement::ZERO
        } else {
            BaseElement::ONE
        };
        let mut next = Vec::with_capacity(level.len() / 2);
        for pair in level.chunks(2) {
            next.push(h2(pair[0], pair[1]));
        }
        level = next;
        idx /= 2;
    }
    let r_in = checked_fe(witness.r_in)?;
    let payload = checked_fe(witness.payload)?;
    let height = checked_fe(witness.height)?;
    let chain_state = checked_fe(witness.chain_state)?;
    Ok(Prepared {
        public: PublicInputs {
            miner_root: level[0],
            nullifier: h2(sk, r_in),
            event_id: h2(h2(payload, height), chain_state),
            height,
        },
        siblings,
        bits,
        leaf,
        sk,
        r_in,
        amount_in: checked_fe(witness.amount_in)?,
        amount_out: checked_fe(witness.amount_out)?,
        fee: checked_fe(witness.fee)?,
        height,
        payload,
        chain_state,
    })
}

pub fn proof_options() -> ProofOptions {
    ProofOptions::new(
        84,
        8,
        20,
        FieldExtension::Quadratic,
        4,
        7,
        BatchingMethod::Linear,
        BatchingMethod::Linear,
    )
}

pub struct CipAir {
    context: AirContext<BaseElement>,
    public: PublicInputs,
}

impl Air for CipAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;

    fn new(trace_info: TraceInfo, public: Self::PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(WIDTH, trace_info.width());
        // Degrees are the ones this AIR actually produces. sk, the amounts, and the
        // event preimage are constant columns, so those cubes do not add degree.
        // Winterfell checks the declared degree exactly.
        let degrees = vec![
            TransitionConstraintDegree::with_cycles(4, vec![TRACE_LEN]),
            TransitionConstraintDegree::with_cycles(1, vec![TRACE_LEN]),
            TransitionConstraintDegree::with_cycles(1, vec![TRACE_LEN]),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::new(2),
            TransitionConstraintDegree::new(1),
            TransitionConstraintDegree::with_cycles(1, vec![TRACE_LEN]),
        ];
        Self {
            context: AirContext::new(trace_info, degrees, 4, options),
            public,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<BaseElement>> {
        let mut merkle = vec![BaseElement::ZERO; TRACE_LEN];
        let mut null_step = vec![BaseElement::ZERO; TRACE_LEN];
        let mut event_step = vec![BaseElement::ZERO; TRACE_LEN];
        let mut pk_step = vec![BaseElement::ZERO; TRACE_LEN];
        for step in merkle.iter_mut().take(4) {
            *step = BaseElement::ONE;
        }
        null_step[4] = BaseElement::ONE;
        event_step[5] = BaseElement::ONE;
        pk_step[0] = BaseElement::ONE;
        vec![merkle, null_step, event_step, pk_step]
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic: &[E],
        result: &mut [E],
    ) {
        let cur = frame.current();
        let next = frame.next();
        let one = E::ONE;
        let merkle = periodic[0];
        let null_step = periodic[1];
        let event_step = periodic[2];
        let pk_step = periodic[3];
        let ordered = {
            let straight = h2e(cur[ACC], cur[SIB]);
            let swapped = h2e(cur[SIB], cur[ACC]);
            cur[BIT] * swapped + (one - cur[BIT]) * straight
        };
        result[0] = merkle * (next[ACC] - ordered) + (one - merkle) * (next[ACC] - cur[ACC]);
        let nullifier = h2e(cur[SK], cur[RIN]);
        result[1] =
            null_step * (next[NULL] - nullifier) + (one - null_step) * (next[NULL] - cur[NULL]);
        let event = h2e(h2e(cur[PAYLOAD], cur[HEIGHT]), cur[CHAIN]);
        result[2] =
            event_step * (next[EVENT] - event) + (one - event_step) * (next[EVENT] - cur[EVENT]);
        result[3] = next[SK] - cur[SK];
        result[4] = next[RIN] - cur[RIN];
        result[5] = next[AMT_IN] - cur[AMT_IN];
        result[6] = next[AMT_OUT] - cur[AMT_OUT];
        result[7] = next[FEE] - cur[FEE];
        result[8] = next[HEIGHT] - cur[HEIGHT];
        result[9] = next[PAYLOAD] - cur[PAYLOAD];
        result[10] = next[CHAIN] - cur[CHAIN];
        result[11] = cur[BIT] * (cur[BIT] - one);
        result[12] = cur[AMT_IN] - cur[AMT_OUT] - cur[FEE];
        let pk = h2e(cur[SK], E::from(fe(PK_DOMAIN)));
        result[13] = pk_step * (cur[ACC] - pk);
    }

    fn get_assertions(&self) -> Vec<Assertion<BaseElement>> {
        vec![
            Assertion::single(ACC, 4, self.public.miner_root),
            Assertion::single(NULL, 5, self.public.nullifier),
            Assertion::single(EVENT, 6, self.public.event_id),
            Assertion::single(HEIGHT, 0, self.public.height),
        ]
    }
}

pub struct CipProver {
    options: ProofOptions,
    prepared: Prepared,
}

impl CipProver {
    pub fn new(prepared: Prepared) -> Self {
        Self {
            options: proof_options(),
            prepared,
        }
    }

    fn build_trace(&self) -> TraceTable<BaseElement> {
        let prep = &self.prepared;
        let mut trace = TraceTable::new(WIDTH, TRACE_LEN);
        trace.fill(
            |state| {
                state[ACC] = prep.leaf;
                state[SIB] = prep.siblings[0];
                state[BIT] = prep.bits[0];
                state[SK] = prep.sk;
                state[RIN] = prep.r_in;
                state[NULL] = BaseElement::ZERO;
                state[AMT_IN] = prep.amount_in;
                state[AMT_OUT] = prep.amount_out;
                state[FEE] = prep.fee;
                state[HEIGHT] = prep.height;
                state[PAYLOAD] = prep.payload;
                state[CHAIN] = prep.chain_state;
                state[EVENT] = BaseElement::ZERO;
            },
            |step, state| {
                if step < 4 {
                    let bit = state[BIT];
                    let acc = state[ACC];
                    let sib = state[SIB];
                    state[ACC] = if bit == BaseElement::ONE {
                        h2(sib, acc)
                    } else {
                        h2(acc, sib)
                    };
                    if step + 1 < 4 {
                        state[SIB] = prep.siblings[step + 1];
                        state[BIT] = prep.bits[step + 1];
                    }
                }
                if step == 4 {
                    state[NULL] = h2(state[SK], state[RIN]);
                }
                if step == 5 {
                    state[EVENT] = h2(h2(state[PAYLOAD], state[HEIGHT]), state[CHAIN]);
                }
            },
        );
        trace
    }
}

impl Prover for CipProver {
    type BaseField = BaseElement;
    type Air = CipAir;
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

    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
        PublicInputs {
            miner_root: trace.get(ACC, 4),
            nullifier: trace.get(NULL, 5),
            event_id: trace.get(EVENT, 6),
            height: trace.get(HEIGHT, 0),
        }
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }

    fn new_trace_lde<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        trace_info: &TraceInfo,
        main_trace: &winterfell::matrix::ColMatrix<Self::BaseField>,
        domain: &StarkDomain<Self::BaseField>,
        partition_option: PartitionOptions,
    ) -> (Self::TraceLde<E>, TracePolyTable<E>) {
        DefaultTraceLde::new(trace_info, main_trace, domain, partition_option)
    }

    fn new_evaluator<'a, E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        air: &'a Self::Air,
        aux_rand_elements: Option<AuxRandElements<E>>,
        composition_coefficients: ConstraintCompositionCoefficients<E>,
    ) -> Self::ConstraintEvaluator<'a, E> {
        DefaultConstraintEvaluator::new(air, aux_rand_elements, composition_coefficients)
    }

    fn build_constraint_commitment<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        composition_poly_trace: CompositionPolyTrace<E>,
        num_constraint_composition_columns: usize,
        domain: &StarkDomain<Self::BaseField>,
        partition_options: PartitionOptions,
    ) -> (Self::ConstraintCommitment<E>, CompositionPoly<E>) {
        DefaultConstraintCommitment::new(
            composition_poly_trace,
            num_constraint_composition_columns,
            domain,
            partition_options,
        )
    }
}

pub fn prove(witness: &Witness) -> Result<(Proof, PublicInputs), &'static str> {
    let prepared = prepare(witness)?;
    let public = prepared.public.clone();
    let prover = CipProver::new(prepared);
    let trace = prover.build_trace();
    let proof = prover.prove(trace).map_err(|_| "proof generation failed")?;
    Ok((proof, public))
}

pub fn verify(proof: Proof, public: PublicInputs) -> Result<(), winterfell::VerifierError> {
    let acceptable = winterfell::AcceptableOptions::OptionSet(vec![proof.options().clone()]);
    winterfell::verify::<CipAir, Rp64_256, DefaultRandomCoin<Rp64_256>, MerkleTree<Rp64_256>>(
        proof,
        public,
        &acceptable,
    )
}

pub fn verify_bytes(bytes: &[u8], public: PublicInputs) -> Result<(), &'static str> {
    let proof = Proof::from_bytes(bytes).map_err(|_| "proof bytes are not a STARK proof")?;
    verify(proof, public).map_err(|_| "proof rejected")
}

fn _hasher_bound() {
    fn assert_hasher<H: ElementHasher<BaseField = BaseElement>>() {}
    assert_hasher::<Rp64_256>();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Witness {
        let mut leaves = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        leaves[3] = public_key(11).as_int();
        Witness {
            sk: 11,
            r_in: 22,
            amount_in: 10,
            amount_out: 9,
            fee: 1,
            height: 7,
            payload: 99,
            chain_state: 5,
            index: 3,
            leaves,
        }
    }

    #[test]
    fn uses_the_paper_fri_parameters() {
        let opts = proof_options();
        assert_eq!(opts.num_queries(), 84);
        assert_eq!(opts.blowup_factor(), 8);
        assert_eq!(opts.grinding_factor(), 20);
    }

    #[test]
    fn accepts_valid_witness_and_rejects_wrong_nullifier() {
        let witness = sample();
        let (proof, public) = prove(&witness).expect("prove");
        verify(proof.clone(), public.clone()).expect("verify");
        let mut wrong = public.clone();
        wrong.nullifier += BaseElement::ONE;
        assert!(verify(proof, wrong).is_err());
    }

    #[test]
    fn rejects_unbalanced_amounts_before_proving() {
        let mut witness = sample();
        witness.fee = 0;
        assert!(prepare(&witness).is_err());
    }

    #[test]
    fn rejects_noncanonical_field_inputs() {
        let mut witness = sample();
        witness.sk = GOLDILOCKS_MODULUS;
        assert_eq!(prepare(&witness).err(), Some("non-canonical field element"));
        assert!(PublicInputs::try_from_u64s([GOLDILOCKS_MODULUS, 0, 0, 0]).is_err());
    }

    #[test]
    fn rejects_amount_addition_overflow() {
        let mut witness = sample();
        witness.amount_in = 0;
        witness.amount_out = u64::MAX;
        witness.fee = 1;
        assert_eq!(prepare(&witness).err(), Some("amount overflow"));
    }

    #[test]
    fn rejects_a_leaf_that_is_not_the_public_key() {
        let mut witness = sample();
        witness.leaves[3] ^= 1;
        assert_eq!(prepare(&witness).err(), Some("miner leaf is not H(sk, 1)"));
    }
}
