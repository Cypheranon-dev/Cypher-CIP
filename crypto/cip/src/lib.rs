//! Spend circuit for one shielded note.
//!
//! Hash: Poseidon2, Goldilocks, width 8, alpha 7, 8 full rounds and 22
//! partial rounds. A digest is the first four lanes. That is Poseidon2.
//! It is not the cubic gadget this circuit used before, and it is not
//! Rescue-Prime. Winterfell's `Rp64_256` commitment hasher is the library
//! hasher only.
//!
//! The trace is 256 rows. Each permutation occupies 32 rows. Depth is 3
//! (8 leaves). Amounts and the fee are range-checked to 32 bits, so the
//! conservation constraint cannot wrap around the field.
//!
//! What the proof binds:
//!   the spent note is a leaf of the public note root
//!   nullifier = Poseidon2(sk, rho, domain)
//!   output note = Poseidon2(amount_out, sk, rho_out, domain)
//!   amount_in = amount_out + fee, all under 2^32
//!   event = Poseidon2(payload, height, previous block id, fee, domain)
//!
//! The nullifier set, the tree update, and the coinbase are node checks.
//! A STARK cannot see the chain. The field `sk` is not a Dilithium key.

pub mod poseidon2;

use poseidon2::{
    full_round, linear_layer, partial_round, step_flags, DOM_EVENT, DOM_NOTE, DOM_NULL, PAD_SLOT,
    SLOT_ROWS,
};
use winterfell::crypto::{hashers::Rp64_256, DefaultRandomCoin, ElementHasher, MerkleTree};
use winterfell::math::{fields::f64::BaseElement, FieldElement, ToElements};
use winterfell::{
    Air, AirContext, Assertion, AuxRandElements, BatchingMethod, CompositionPoly,
    CompositionPolyTrace, ConstraintCompositionCoefficients, DefaultConstraintCommitment,
    DefaultConstraintEvaluator, DefaultTraceLde, EvaluationFrame, FieldExtension, PartitionOptions,
    Proof, ProofOptions, Prover, StarkDomain, TraceInfo, TracePolyTable, TraceTable,
    TransitionConstraintDegree,
};

const TRACE_LEN: usize = 256;
const DEPTH: usize = 3;
pub const LEAVES: usize = 8;
const AMOUNT_BITS: usize = 32;

const STATE: usize = 0;
const SK: usize = 8;
const RIN: usize = 9;
const AMT_IN: usize = 10;
const AMT_OUT: usize = 11;
const FEE: usize = 12;
const HEIGHT: usize = 13;
const PAYLOAD: usize = 14;
const RHO: usize = 15;
const CHAIN: usize = 16;
const BIT: usize = 20;
const SIB: usize = 23;
const RANGE: usize = 35;
const WIDTH: usize = 131;

const ROOT_ROW: usize = 3 * SLOT_ROWS + (SLOT_ROWS - 1);
const NULL_ROW: usize = 4 * SLOT_ROWS + (SLOT_ROWS - 1);
const OUT_ROW: usize = 5 * SLOT_ROWS + (SLOT_ROWS - 1);
const EVENT_ROW: usize = 6 * SLOT_ROWS + (SLOT_ROWS - 1);

const P_LINEAR: usize = 0;
const P_FULL: usize = 1;
const P_PARTIAL: usize = 2;
const P_PAD: usize = 3;
const P_ROW0: usize = 4;
const P_BOUNDARY: usize = 5;
const P_RC: usize = 12;

const N_STATE: usize = 8;
const N_ROW0: usize = 8;
const N_CONST: usize = WIDTH - 8;
const N_PATH_BOOL: usize = DEPTH;
const N_RANGE_BOOL: usize = AMOUNT_BITS * 3;
const N_RANGE_SUM: usize = 3;
const N_CONSTRAINTS: usize =
    N_STATE + N_ROW0 + N_CONST + N_PATH_BOOL + N_RANGE_BOOL + N_RANGE_SUM + 1;
const N_ASSERTIONS: usize = 23;

/// Little-endian public inputs. 4-element digests, then scalars.
pub const PUBLIC_LEN: usize = 23;
const GOLDILOCKS_MODULUS: u64 = 0xffff_ffff_0000_0001;

pub type Digest = [BaseElement; 4];

pub fn fe(value: u64) -> BaseElement {
    BaseElement::new(value)
}

fn checked_fe(value: u64) -> Result<BaseElement, &'static str> {
    if value >= GOLDILOCKS_MODULUS {
        return Err("non-canonical field element");
    }
    Ok(fe(value))
}

fn fit_u32(value: u64) -> Result<u64, &'static str> {
    if value >= 1u64 << AMOUNT_BITS {
        return Err("amount exceeds 2^32");
    }
    Ok(value)
}

fn take4(state: [BaseElement; 8]) -> Digest {
    [state[0], state[1], state[2], state[3]]
}

pub fn digest_words(digest: Digest) -> [u64; 4] {
    digest.map(|element| element.as_int())
}

pub fn digest_from_words(words: [u64; 4]) -> Result<Digest, &'static str> {
    Ok([
        checked_fe(words[0])?,
        checked_fe(words[1])?,
        checked_fe(words[2])?,
        checked_fe(words[3])?,
    ])
}

pub fn root_words(leaves: &[[u64; 4]; LEAVES]) -> Result<[u64; 4], &'static str> {
    let mut digests = [[BaseElement::ZERO; 4]; LEAVES];
    for (dst, src) in digests.iter_mut().zip(leaves) {
        *dst = digest_from_words(*src)?;
    }
    Ok(digest_words(merkle_root(&digests)))
}

/// Note commitment. `rho` is the note randomness. The same function opens
/// an output note, so a spend of it uses this digest as the leaf.
pub fn note_commitment(amount: u64, sk: u64, rho: u64) -> Result<Digest, &'static str> {
    let _ = fit_u32(amount)?;
    let state = [
        checked_fe(amount)?,
        checked_fe(sk)?,
        checked_fe(rho)?,
        fe(DOM_NOTE),
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
    ];
    Ok(take4(poseidon2::permute(state)))
}

pub fn nullifier_of(sk: u64, rho: u64) -> Result<Digest, &'static str> {
    let state = [
        checked_fe(sk)?,
        checked_fe(rho)?,
        fe(DOM_NULL),
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
    ];
    Ok(take4(poseidon2::permute(state)))
}

/// Event digest over the public header fields the node recomputes.
pub fn event_digest(
    payload: u64,
    height: u64,
    chain: [u64; 4],
    fee: u64,
) -> Result<Digest, &'static str> {
    let _ = fit_u32(fee)?;
    let state = [
        checked_fe(payload)?,
        checked_fe(height)?,
        checked_fe(chain[0])?,
        checked_fe(chain[1])?,
        checked_fe(chain[2])?,
        checked_fe(chain[3])?,
        checked_fe(fee)?,
        fe(DOM_EVENT),
    ];
    Ok(take4(poseidon2::permute(state)))
}

fn parent(left: Digest, right: Digest) -> Digest {
    let state = [
        left[0], left[1], left[2], left[3], right[0], right[1], right[2], right[3],
    ];
    take4(poseidon2::permute(state))
}

pub fn merkle_root(leaves: &[Digest; LEAVES]) -> Digest {
    let mut level = leaves.to_vec();
    while level.len() > 1 {
        let mut next = Vec::with_capacity(level.len() / 2);
        for pair in level.chunks(2) {
            next.push(parent(pair[0], pair[1]));
        }
        level = next;
    }
    level[0]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicInputs {
    pub note_root: Digest,
    pub nullifier: Digest,
    pub output: Digest,
    pub event_id: Digest,
    pub height: BaseElement,
    pub fee: BaseElement,
    pub payload: BaseElement,
    pub chain_state: Digest,
}

impl ToElements<BaseElement> for PublicInputs {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = Vec::with_capacity(PUBLIC_LEN);
        out.extend_from_slice(&self.note_root);
        out.extend_from_slice(&self.nullifier);
        out.extend_from_slice(&self.output);
        out.extend_from_slice(&self.event_id);
        out.push(self.height);
        out.push(self.fee);
        out.push(self.payload);
        out.extend_from_slice(&self.chain_state);
        out
    }
}

impl PublicInputs {
    pub fn to_u64s(&self) -> [u64; PUBLIC_LEN] {
        let mut out = [0u64; PUBLIC_LEN];
        out[0..4].copy_from_slice(&digest_words(self.note_root));
        out[4..8].copy_from_slice(&digest_words(self.nullifier));
        out[8..12].copy_from_slice(&digest_words(self.output));
        out[12..16].copy_from_slice(&digest_words(self.event_id));
        out[16] = self.height.as_int();
        out[17] = self.fee.as_int();
        out[18] = self.payload.as_int();
        out[19..23].copy_from_slice(&digest_words(self.chain_state));
        out
    }

    pub fn try_from_u64s(values: [u64; PUBLIC_LEN]) -> Result<Self, &'static str> {
        let digest = |start: usize| -> Result<Digest, &'static str> {
            Ok([
                checked_fe(values[start])?,
                checked_fe(values[start + 1])?,
                checked_fe(values[start + 2])?,
                checked_fe(values[start + 3])?,
            ])
        };
        Ok(Self {
            note_root: digest(0)?,
            nullifier: digest(4)?,
            output: digest(8)?,
            event_id: digest(12)?,
            height: checked_fe(values[16])?,
            fee: checked_fe(fit_u32(values[17])?)?,
            payload: checked_fe(values[18])?,
            chain_state: digest(19)?,
        })
    }
}

#[derive(Clone)]
pub struct Witness {
    pub sk: u64,
    pub r_in: u64,
    pub rho_out: u64,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee: u64,
    pub height: u64,
    pub payload: u64,
    pub chain_state: [u64; 4],
    pub index: usize,
    pub leaves: [[u64; 4]; LEAVES],
}

struct Prepared {
    public: PublicInputs,
    sk: BaseElement,
    r_in: BaseElement,
    rho_out: BaseElement,
    amount_in: BaseElement,
    amount_out: BaseElement,
    fee: BaseElement,
    height: BaseElement,
    payload: BaseElement,
    chain_state: Digest,
    siblings: [Digest; DEPTH],
    bits: [BaseElement; DEPTH],
}

pub fn prepare(witness: &Witness) -> Result<PublicInputs, &'static str> {
    Ok(prepare_inner(witness)?.public)
}

fn prepare_inner(witness: &Witness) -> Result<Prepared, &'static str> {
    if witness.index >= LEAVES {
        return Err("note index out of range");
    }
    let amount_in = fit_u32(witness.amount_in)?;
    let amount_out = fit_u32(witness.amount_out)?;
    let fee = fit_u32(witness.fee)?;
    let expected = amount_out.checked_add(fee).ok_or("amount overflow")?;
    if amount_in != expected {
        return Err("amounts do not conserve");
    }
    let note = note_commitment(amount_in, witness.sk, witness.r_in)?;
    let mut leaves = [[BaseElement::ZERO; 4]; LEAVES];
    for (dst, src) in leaves.iter_mut().zip(witness.leaves) {
        for lane in 0..4 {
            dst[lane] = checked_fe(src[lane])?;
        }
    }
    if leaves[witness.index] != note {
        return Err("note leaf is not the spent commitment");
    }
    let mut level = leaves.to_vec();
    let mut siblings = [[BaseElement::ZERO; 4]; DEPTH];
    let mut bits = [BaseElement::ZERO; DEPTH];
    let mut idx = witness.index;
    for depth in 0..DEPTH {
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
            next.push(parent(pair[0], pair[1]));
        }
        level = next;
        idx /= 2;
    }
    let chain_state = [
        checked_fe(witness.chain_state[0])?,
        checked_fe(witness.chain_state[1])?,
        checked_fe(witness.chain_state[2])?,
        checked_fe(witness.chain_state[3])?,
    ];
    let event_id = event_digest(witness.payload, witness.height, witness.chain_state, fee)?;
    Ok(Prepared {
        public: PublicInputs {
            note_root: level[0],
            nullifier: nullifier_of(witness.sk, witness.r_in)?,
            output: note_commitment(amount_out, witness.sk, witness.rho_out)?,
            event_id,
            height: checked_fe(witness.height)?,
            fee: fe(fee),
            payload: checked_fe(witness.payload)?,
            chain_state,
        },
        sk: checked_fe(witness.sk)?,
        r_in: checked_fe(witness.r_in)?,
        rho_out: checked_fe(witness.rho_out)?,
        amount_in: fe(amount_in),
        amount_out: fe(amount_out),
        fee: fe(fee),
        height: checked_fe(witness.height)?,
        payload: checked_fe(witness.payload)?,
        chain_state,
        siblings,
        bits,
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

struct Sel<E: FieldElement> {
    linear: E,
    full: E,
    partial: E,
    pad: E,
    boundary: [E; 7],
    rc: [E; 8],
}

fn merkle_load<E: FieldElement>(acc: [E; 4], sib: [E; 4], bit: E) -> [E; 8] {
    let one = E::ONE;
    let mut out = [E::ZERO; 8];
    for lane in 0..4 {
        out[lane] = bit * sib[lane] + (one - bit) * acc[lane];
        out[4 + lane] = bit * acc[lane] + (one - bit) * sib[lane];
    }
    out
}

fn boundaries<E: FieldElement<BaseField = BaseElement>>(cur: &[E]) -> [[E; 8]; 7] {
    let mut out = [[E::ZERO; 8]; 7];
    for depth in 0..DEPTH {
        let acc = [cur[STATE], cur[STATE + 1], cur[STATE + 2], cur[STATE + 3]];
        let sib = [
            cur[SIB + depth * 4],
            cur[SIB + depth * 4 + 1],
            cur[SIB + depth * 4 + 2],
            cur[SIB + depth * 4 + 3],
        ];
        out[depth] = merkle_load(acc, sib, cur[BIT + depth]);
    }
    out[3] = [
        cur[SK],
        cur[RIN],
        E::from(fe(DOM_NULL)),
        E::ZERO,
        E::ZERO,
        E::ZERO,
        E::ZERO,
        E::ZERO,
    ];
    out[4] = [
        cur[AMT_OUT],
        cur[SK],
        cur[RHO],
        E::from(fe(DOM_NOTE)),
        E::ZERO,
        E::ZERO,
        E::ZERO,
        E::ZERO,
    ];
    out[5] = [
        cur[PAYLOAD],
        cur[HEIGHT],
        cur[CHAIN],
        cur[CHAIN + 1],
        cur[CHAIN + 2],
        cur[CHAIN + 3],
        cur[FEE],
        E::from(fe(DOM_EVENT)),
    ];
    out
}

fn transition<E: FieldElement<BaseField = BaseElement>>(cur: &[E], sel: &Sel<E>) -> [E; 8] {
    let state = [
        cur[0], cur[1], cur[2], cur[3], cur[4], cur[5], cur[6], cur[7],
    ];
    let linear = linear_layer(state);
    let full = full_round(state, sel.rc);
    let partial = partial_round(state, sel.rc[0]);
    let loads = boundaries(cur);
    let mut out = [E::ZERO; 8];
    for lane in 0..8 {
        out[lane] = sel.linear * linear[lane]
            + sel.full * full[lane]
            + sel.partial * partial[lane]
            + sel.pad * state[lane];
        for (boundary, load) in loads.iter().enumerate() {
            out[lane] += sel.boundary[boundary] * load[lane];
        }
    }
    out
}

fn constraint_degrees() -> Vec<TransitionConstraintDegree> {
    let mut degrees = Vec::with_capacity(N_CONSTRAINTS);
    let round = TransitionConstraintDegree::with_cycles(7, vec![TRACE_LEN]);
    let flagged = TransitionConstraintDegree::with_cycles(1, vec![TRACE_LEN]);
    for _ in 0..N_STATE {
        degrees.push(round.clone());
    }
    for _ in 0..N_ROW0 {
        degrees.push(flagged.clone());
    }
    for _ in 0..N_CONST {
        degrees.push(TransitionConstraintDegree::new(1));
    }
    for _ in 0..N_PATH_BOOL + N_RANGE_BOOL {
        degrees.push(TransitionConstraintDegree::new(2));
    }
    for _ in 0..N_RANGE_SUM + 1 {
        degrees.push(TransitionConstraintDegree::new(1));
    }
    degrees
}

fn write_bits(row: &mut [BaseElement], offset: usize, value: u64) {
    for bit in 0..AMOUNT_BITS {
        row[RANGE + offset + bit] = if (value >> bit) & 1 == 1 {
            BaseElement::ONE
        } else {
            BaseElement::ZERO
        };
    }
}

fn fill_row(row: &mut [BaseElement], prep: &Prepared, state: [BaseElement; 8]) {
    row[..8].copy_from_slice(&state);
    row[SK] = prep.sk;
    row[RIN] = prep.r_in;
    row[AMT_IN] = prep.amount_in;
    row[AMT_OUT] = prep.amount_out;
    row[FEE] = prep.fee;
    row[HEIGHT] = prep.height;
    row[PAYLOAD] = prep.payload;
    row[RHO] = prep.rho_out;
    row[CHAIN..CHAIN + 4].copy_from_slice(&prep.chain_state);
    for depth in 0..DEPTH {
        row[BIT + depth] = prep.bits[depth];
        row[SIB + depth * 4..SIB + depth * 4 + 4].copy_from_slice(&prep.siblings[depth]);
    }
    write_bits(row, 0, prep.amount_in.as_int());
    write_bits(row, AMOUNT_BITS, prep.amount_out.as_int());
    write_bits(row, AMOUNT_BITS * 2, prep.fee.as_int());
}

fn fill_trace(prep: &Prepared) -> TraceTable<BaseElement> {
    let mut trace = TraceTable::new(WIDTH, TRACE_LEN);
    let preimage = [
        prep.amount_in,
        prep.sk,
        prep.r_in,
        fe(DOM_NOTE),
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
    ];
    trace.fill(
        |row| fill_row(row, prep, preimage),
        |step, row| {
            let flags = step_flags(step);
            let sel = Sel {
                linear: flag(flags.linear),
                full: flag(flags.full),
                partial: flag(flags.partial),
                pad: flag(flags.pad),
                boundary: flags.boundary.map(flag),
                rc: flags.rc.map(fe),
            };
            let next = transition(row, &sel);
            fill_row(row, prep, next);
        },
    );
    trace
}

fn flag(on: bool) -> BaseElement {
    if on {
        BaseElement::ONE
    } else {
        BaseElement::ZERO
    }
}

fn digest_at(trace: &TraceTable<BaseElement>, row: usize) -> Digest {
    [
        trace.get(0, row),
        trace.get(1, row),
        trace.get(2, row),
        trace.get(3, row),
    ]
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
        assert_eq!(TRACE_LEN, trace_info.length());
        let degrees = constraint_degrees();
        assert_eq!(degrees.len(), N_CONSTRAINTS);
        Self {
            context: AirContext::new(trace_info, degrees, N_ASSERTIONS, options),
            public,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<BaseElement>> {
        let mut columns = vec![vec![BaseElement::ZERO; TRACE_LEN]; P_RC + 8];
        for (step, flags) in (0..TRACE_LEN).map(step_flags).enumerate() {
            columns[P_LINEAR][step] = flag(flags.linear);
            columns[P_FULL][step] = flag(flags.full);
            columns[P_PARTIAL][step] = flag(flags.partial);
            columns[P_PAD][step] = flag(flags.pad);
            columns[P_ROW0][step] = flag(flags.row0);
            for (lane, on) in flags.boundary.iter().enumerate() {
                columns[P_BOUNDARY + lane][step] = flag(*on);
            }
            for (lane, rc) in flags.rc.iter().enumerate() {
                columns[P_RC + lane][step] = fe(*rc);
            }
        }
        let _ = PAD_SLOT;
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
        let sel = Sel {
            linear: periodic[P_LINEAR],
            full: periodic[P_FULL],
            partial: periodic[P_PARTIAL],
            pad: periodic[P_PAD],
            boundary: [
                periodic[P_BOUNDARY],
                periodic[P_BOUNDARY + 1],
                periodic[P_BOUNDARY + 2],
                periodic[P_BOUNDARY + 3],
                periodic[P_BOUNDARY + 4],
                periodic[P_BOUNDARY + 5],
                periodic[P_BOUNDARY + 6],
            ],
            rc: [
                periodic[P_RC],
                periodic[P_RC + 1],
                periodic[P_RC + 2],
                periodic[P_RC + 3],
                periodic[P_RC + 4],
                periodic[P_RC + 5],
                periodic[P_RC + 6],
                periodic[P_RC + 7],
            ],
        };
        let expected = transition(cur, &sel);
        let mut index = 0;
        for lane in 0..8 {
            result[index] = next[lane] - expected[lane];
            index += 1;
        }
        let preimage = [
            cur[AMT_IN],
            cur[SK],
            cur[RIN],
            E::from(fe(DOM_NOTE)),
            E::ZERO,
            E::ZERO,
            E::ZERO,
            E::ZERO,
        ];
        for lane in 0..8 {
            result[index] = periodic[P_ROW0] * (cur[lane] - preimage[lane]);
            index += 1;
        }
        for column in 8..WIDTH {
            result[index] = next[column] - cur[column];
            index += 1;
        }
        let one = E::ONE;
        for depth in 0..DEPTH {
            let bit = cur[BIT + depth];
            result[index] = bit * (bit - one);
            index += 1;
        }
        for bit_index in 0..N_RANGE_BOOL {
            let bit = cur[RANGE + bit_index];
            result[index] = bit * (bit - one);
            index += 1;
        }
        for (offset, column) in [(0, AMT_IN), (AMOUNT_BITS, AMT_OUT), (AMOUNT_BITS * 2, FEE)] {
            let mut acc = E::ZERO;
            let mut place = E::ONE;
            for bit_index in 0..AMOUNT_BITS {
                acc += cur[RANGE + offset + bit_index] * place;
                place = place.double();
            }
            result[index] = cur[column] - acc;
            index += 1;
        }
        result[index] = cur[AMT_IN] - cur[AMT_OUT] - cur[FEE];
        debug_assert_eq!(index + 1, N_CONSTRAINTS);
    }

    fn get_assertions(&self) -> Vec<Assertion<BaseElement>> {
        let mut assertions = Vec::with_capacity(N_ASSERTIONS);
        let bind = |assertions: &mut Vec<Assertion<BaseElement>>, row: usize, digest: Digest| {
            for (lane, element) in digest.iter().enumerate() {
                assertions.push(Assertion::single(lane, row, *element));
            }
        };
        bind(&mut assertions, ROOT_ROW, self.public.note_root);
        bind(&mut assertions, NULL_ROW, self.public.nullifier);
        bind(&mut assertions, OUT_ROW, self.public.output);
        bind(&mut assertions, EVENT_ROW, self.public.event_id);
        assertions.push(Assertion::single(HEIGHT, 0, self.public.height));
        assertions.push(Assertion::single(FEE, 0, self.public.fee));
        assertions.push(Assertion::single(PAYLOAD, 0, self.public.payload));
        for lane in 0..4 {
            assertions.push(Assertion::single(
                CHAIN + lane,
                0,
                self.public.chain_state[lane],
            ));
        }
        assert_eq!(assertions.len(), N_ASSERTIONS);
        assertions
    }
}

pub struct CipProver {
    options: ProofOptions,
    prepared: Prepared,
}

impl CipProver {
    fn new(prepared: Prepared) -> Self {
        Self {
            options: proof_options(),
            prepared,
        }
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
            note_root: digest_at(trace, ROOT_ROW),
            nullifier: digest_at(trace, NULL_ROW),
            output: digest_at(trace, OUT_ROW),
            event_id: digest_at(trace, EVENT_ROW),
            height: trace.get(HEIGHT, 0),
            fee: trace.get(FEE, 0),
            payload: trace.get(PAYLOAD, 0),
            chain_state: [
                trace.get(CHAIN, 0),
                trace.get(CHAIN + 1, 0),
                trace.get(CHAIN + 2, 0),
                trace.get(CHAIN + 3, 0),
            ],
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
    let prepared = prepare_inner(witness)?;
    let public = prepared.public.clone();
    let prover = CipProver::new(prepared);
    let trace = fill_trace(&prover.prepared);
    let traced = prover.get_pub_inputs(&trace);
    if traced != public {
        return Err("trace does not match public inputs");
    }
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
        let sk = 11;
        let rho = 22;
        let note = digest_words(note_commitment(10, sk, rho).unwrap());
        let mut leaves = [[0u64; 4]; LEAVES];
        leaves[3] = note;
        Witness {
            sk,
            r_in: rho,
            rho_out: 77,
            amount_in: 10,
            amount_out: 9,
            fee: 1,
            height: 7,
            payload: 99,
            chain_state: [1, 2, 3, 4],
            index: 3,
            leaves,
        }
    }

    #[test]
    fn flags_cover_every_row_once() {
        for step in 0..TRACE_LEN {
            let flags = step_flags(step);
            let kinds = flags.linear as u8
                + flags.full as u8
                + flags.partial as u8
                + flags.pad as u8
                + flags.boundary.iter().filter(|on| **on).count() as u8;
            assert_eq!(kinds, 1, "step {step}");
        }
    }

    #[test]
    fn trace_satisfies_every_constraint() {
        let prepared = prepare_inner(&sample()).expect("prepare");
        let trace = fill_trace(&prepared);
        let info = TraceInfo::new(WIDTH, TRACE_LEN);
        let air = CipAir::new(info, prepared.public.clone(), proof_options());
        let periodic = air.get_periodic_column_values();
        for step in 0..TRACE_LEN - 1 {
            let mut current = Vec::with_capacity(WIDTH);
            let mut next = Vec::with_capacity(WIDTH);
            for column in 0..WIDTH {
                current.push(trace.get(column, step));
                next.push(trace.get(column, step + 1));
            }
            let flags: Vec<BaseElement> = periodic.iter().map(|column| column[step]).collect();
            let frame = EvaluationFrame::from_rows(current, next);
            let mut result = vec![BaseElement::ZERO; N_CONSTRAINTS];
            air.evaluate_transition(&frame, &flags, &mut result);
            for (index, value) in result.iter().enumerate() {
                assert_eq!(*value, BaseElement::ZERO, "step {step} constraint {index}");
            }
        }
        assert_eq!(digest_at(&trace, ROOT_ROW), prepared.public.note_root);
        assert_eq!(digest_at(&trace, NULL_ROW), prepared.public.nullifier);
        assert_eq!(digest_at(&trace, OUT_ROW), prepared.public.output);
        assert_eq!(digest_at(&trace, EVENT_ROW), prepared.public.event_id);
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
        let mut wrong = public;
        wrong.nullifier[0] += BaseElement::ONE;
        assert!(verify(proof, wrong).is_err());
    }

    #[test]
    fn rejects_unbalanced_amounts_before_proving() {
        let mut witness = sample();
        witness.fee = 0;
        assert_eq!(prepare(&witness).err(), Some("amounts do not conserve"));
    }

    #[test]
    fn rejects_noncanonical_field_inputs() {
        let mut witness = sample();
        witness.sk = GOLDILOCKS_MODULUS;
        assert_eq!(prepare(&witness).err(), Some("non-canonical field element"));
        let mut words = [0u64; PUBLIC_LEN];
        words[0] = GOLDILOCKS_MODULUS;
        assert!(PublicInputs::try_from_u64s(words).is_err());
    }

    #[test]
    fn rejects_amount_addition_overflow() {
        let mut witness = sample();
        witness.amount_in = 0;
        witness.amount_out = u64::MAX;
        witness.fee = 1;
        assert_eq!(prepare(&witness).err(), Some("amount exceeds 2^32"));
    }

    #[test]
    fn rejects_a_leaf_that_is_not_the_note() {
        let mut witness = sample();
        witness.leaves[3][0] ^= 1;
        assert_eq!(
            prepare(&witness).err(),
            Some("note leaf is not the spent commitment")
        );
    }
}
