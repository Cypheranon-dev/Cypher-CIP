//! Canonical Poseidon2 permutation for Goldilocks, width 8.
//!
//! Parameters match the current Plonky3 Goldilocks configuration:
//! alpha = 7, RF = 8 (4 initial + 4 terminal), RP = 22.
//! Round constants are generated from the Poseidon2 Grain-LFSR procedure.
//! This module is intentionally self-contained so the AIR and native code can
//! share the same permutation without depending on a second field type.

use winterfell::math::fields::f64::BaseElement;
use winterfell::math::FieldElement;

/// Spend-note domain. Input and output notes use the same domain so an
/// output can be spent.
pub const DOM_NOTE: u64 = 1;
/// Nullifier domain. Distinct from [`DOM_NOTE`].
pub const DOM_NULL: u64 = 2;
/// Event domain. Binds payload, height, fee, and the previous block id.
pub const DOM_EVENT: u64 = 4;

/// Rows per Poseidon2 permutation, including the input row.
pub const SLOT_ROWS: usize = 32;
/// First trace slot that is padding, not a permutation.
pub const PAD_SLOT: usize = 7;

const RC_INITIAL: [[u64; 8]; 4] = [
    [
        0xdd5743e7f2a5a5d9,
        0xcb3a864e58ada44b,
        0xffa2449ed32f8cdc,
        0x42025f65d6bd13ee,
        0x7889175e25506323,
        0x34b98bb03d24b737,
        0xbdcc535ecc4faa2a,
        0x5b20ad869fc0d033,
    ],
    [
        0xf1dda5b9259dfcb4,
        0x27515210be112d59,
        0x4227d1718c766c3f,
        0x26d333161a5bd794,
        0x49b938957bf4b026,
        0x4a56b5938b213669,
        0x1120426b48c8353d,
        0x6b323c3f10a56cad,
    ],
    [
        0xce57d6245ddca6b2,
        0xb1fc8d402bba1eb1,
        0xb5c5096ca959bd04,
        0x6db55cd306d31f7f,
        0xc49d293a81cb9641,
        0x1ce55a4fe979719f,
        0xa92e60a9d178a4d1,
        0x002cc64973bcfd8c,
    ],
    [
        0xcea721cce82fb11b,
        0xe5b55eb8098ece81,
        0x4e30525c6f1ddd66,
        0x43c6702827070987,
        0xaca68430a7b5762a,
        0x3674238634df9c93,
        0x88cee1c825e33433,
        0xde99ae8d74b57176,
    ],
];

const RC_FINAL: [[u64; 8]; 4] = [
    [
        0x014ef1197d341346,
        0x9725e20825d07394,
        0xfdb25aef2c5bae3b,
        0xbe5402dc598c971e,
        0x93a5711f04cdca3d,
        0xc45a9a5b2f8fb97b,
        0xfe8946a924933545,
        0x2af997a27369091c,
    ],
    [
        0xaa62c88e0b294011,
        0x058eb9d810ce9f74,
        0xb3cb23eced349ae4,
        0xa3648177a77b4a84,
        0x43153d905992d95d,
        0xf4e2a97cda44aa4b,
        0x5baa2702b908682f,
        0x082923bdf4f750d1,
    ],
    [
        0x98ae09a325893803,
        0xf8a6475077968838,
        0xceb0735bf00b2c5f,
        0x0a1a5d953888e072,
        0x2fcb190489f94475,
        0xb5be06270dec69fc,
        0x739cb934b09acf8b,
        0x537750b75ec7f25b,
    ],
    [
        0xe9dd318bae1f3961,
        0xf7462137299efe1a,
        0xb1f6b8eee9adb940,
        0xbdebcc8a809dfe6b,
        0x40fc1f791b178113,
        0x3ac1c3362d014864,
        0x9a016184bdb8aeba,
        0x95f2394459fbc25e,
    ],
];

const RC_INTERNAL: [u64; 22] = [
    0x488897d85ff51f56,
    0x1140737ccb162218,
    0xa7eeb9215866ed35,
    0x9bd2976fee49fcc9,
    0xc0c8f0de580a3fcc,
    0x4fb2dae6ee8fc793,
    0x343a89f35f37395b,
    0x223b525a77ca72c8,
    0x56ccb62574aaa918,
    0xc4d507d8027af9ed,
    0xa080673cf0b7e95c,
    0xf0184884eb70dcf8,
    0x044f10b0cb3d5c69,
    0xe9e3f7993938f186,
    0x1b761c80e772f459,
    0x606cec607a1b5fac,
    0x14a0c2e1d45f03cd,
    0x4eace8855398574f,
    0xf905ca7103eff3e6,
    0xf8c8f8d20862c059,
    0xb524fe8bdd678e5a,
    0xfbb7865901a1ec41,
];

const DIAG: [u64; 8] = [
    0xfffffffeffffffff,
    1,
    2,
    0x7fffffff80000001,
    3,
    0x7fffffff80000000,
    0xfffffffefffffffe,
    0xfffffffefffffffd,
];

#[inline]
pub fn sbox<E: FieldElement>(x: E) -> E {
    let x2 = x.square();
    let x4 = x2.square();
    x4 * x2 * x
}

#[inline]
fn mat4<E: FieldElement>(x: &mut [E; 4]) {
    let t01 = x[0] + x[1];
    let t23 = x[2] + x[3];
    let t0123 = t01 + t23;
    let t01123 = t0123 + x[1];
    let t01233 = t0123 + x[3];
    let old0 = x[0];
    let old2 = x[2];
    x[3] = t01233 + old0.double();
    x[1] = t01123 + old2.double();
    x[0] = t01123 + t01;
    x[2] = t01233 + t23;
}

#[inline]
pub fn external_linear<E: FieldElement>(state: &mut [E; 8]) {
    let mut left = [state[0], state[1], state[2], state[3]];
    let mut right = [state[4], state[5], state[6], state[7]];
    mat4(&mut left);
    mat4(&mut right);
    state[..4].copy_from_slice(&left);
    state[4..].copy_from_slice(&right);

    let sums = [
        state[0] + state[4],
        state[1] + state[5],
        state[2] + state[6],
        state[3] + state[7],
    ];
    for i in 0..8 {
        state[i] += sums[i % 4];
    }
}

#[inline]
pub fn internal_linear<E: FieldElement<BaseField = BaseElement>>(state: &mut [E; 8]) {
    let sum = state.iter().copied().fold(E::ZERO, |a, b| a + b);
    for i in 0..8 {
        state[i] = sum + state[i] * E::from(BaseElement::new(DIAG[i]));
    }
}

pub fn linear_layer<E: FieldElement>(mut state: [E; 8]) -> [E; 8] {
    external_linear(&mut state);
    state
}

pub fn full_round<E: FieldElement>(mut state: [E; 8], rc: [E; 8]) -> [E; 8] {
    for i in 0..8 {
        state[i] = sbox(state[i] + rc[i]);
    }
    external_linear(&mut state);
    state
}

pub fn partial_round<E: FieldElement<BaseField = BaseElement>>(
    mut state: [E; 8],
    rc0: E,
) -> [E; 8] {
    state[0] = sbox(state[0] + rc0);
    internal_linear(&mut state);
    state
}

/// Which transition leaves this trace row. Round constants are zero unless
/// the row is a full or partial S-box round.
#[derive(Clone, Copy, Debug)]
pub struct StepFlags {
    pub linear: bool,
    pub full: bool,
    pub partial: bool,
    pub pad: bool,
    pub row0: bool,
    pub boundary: [bool; 7],
    pub rc: [u64; 8],
}

/// Flags for one row of the 256-row spend trace. Slots 0..6 are permutations.
/// Slot 7 is padding. Row 0 is also the note preimage.
pub fn step_flags(step: usize) -> StepFlags {
    let mut flags = StepFlags {
        linear: false,
        full: false,
        partial: false,
        pad: false,
        row0: false,
        boundary: [false; 7],
        rc: [0; 8],
    };
    let slot = step / SLOT_ROWS;
    let local = step % SLOT_ROWS;
    if slot >= PAD_SLOT {
        flags.pad = true;
        return flags;
    }
    if local == SLOT_ROWS - 1 {
        flags.boundary[slot] = true;
        return flags;
    }
    match local {
        0 => flags.linear = true,
        1..=4 => {
            flags.full = true;
            flags.rc = RC_INITIAL[local - 1];
        }
        5..=26 => {
            flags.partial = true;
            flags.rc[0] = RC_INTERNAL[local - 5];
        }
        27..=30 => {
            flags.full = true;
            flags.rc = RC_FINAL[local - 27];
        }
        _ => unreachable!("local row {local}"),
    }
    if step == 0 {
        flags.row0 = true;
    }
    flags
}

/// Apply the canonical width-8 Goldilocks Poseidon2 permutation.
pub fn permute(state: [BaseElement; 8]) -> [BaseElement; 8] {
    let mut state = linear_layer(state);
    for rc in RC_INITIAL {
        state = full_round(state, rc.map(BaseElement::new));
    }
    for rc in RC_INTERNAL {
        state = partial_round(state, BaseElement::new(rc));
    }
    for rc in RC_FINAL {
        state = full_round(state, rc.map(BaseElement::new));
    }
    state
}

/// Fixed-width two-field-element compression with a protocol domain separator.
/// The protocol will use the first four lanes as a 256-bit digest.
pub fn compress2to4(domain: u64, a: BaseElement, b: BaseElement) -> [BaseElement; 4] {
    let state = [
        a,
        b,
        BaseElement::new(domain),
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
        BaseElement::ZERO,
    ];
    let out = permute(state);
    [out[0], out[1], out[2], out[3]]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_plonky3_width_8_kat() {
        let input = [
            BaseElement::new(0),
            BaseElement::new(1),
            BaseElement::new(2),
            BaseElement::new(3),
            BaseElement::new(4),
            BaseElement::new(5),
            BaseElement::new(6),
            BaseElement::new(7),
        ];
        let got = permute(input);
        let expected = [
            0x020cf04a1b214d14,
            0x84e14aaaeacaed25,
            0x1ae0f640e81c7457,
            0xa4d204cbaeb0d8a5,
            0x0cf637b627b3a7ff,
            0x788d304d948b486b,
            0x7327133ea1949af4,
            0xf415abb924da395b,
        ];
        assert_eq!(got.map(|x| x.as_int()), expected);
    }

    #[test]
    fn stepwise_rounds_match_permute() {
        let input = [
            BaseElement::new(9),
            BaseElement::new(8),
            BaseElement::new(7),
            BaseElement::new(DOM_NOTE),
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
        ];
        let mut state = input;
        for local in 0..SLOT_ROWS - 1 {
            let flags = step_flags(local);
            let rc = flags.rc.map(BaseElement::new);
            state = if flags.linear {
                linear_layer(state)
            } else if flags.full {
                full_round(state, rc)
            } else if flags.partial {
                partial_round(state, rc[0])
            } else {
                panic!("row {local} is not a round");
            };
        }
        assert_eq!(state, permute(input));
    }

    #[test]
    fn compression_is_four_field_elements() {
        let digest = compress2to4(0x4359504845525f31, BaseElement::new(1), BaseElement::new(2));
        assert_eq!(digest.len(), 4);
        assert_ne!(digest, [BaseElement::ZERO; 4]);
    }
}
