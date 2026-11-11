//! Fixed-depth layout. One circuit, one verifying key.
//!
//! MAX_DEPTH is the anonymity-set ceiling. A spend of a shallower tree
//! pads the unused levels with a selector column. Depth 3, 4, and 8
//! are the same AIR with different amounts of padding.

/// Ceiling on the note tree. Sized once. Not per proof.
pub const MAX_DEPTH: usize = 20;

/// Rows per Poseidon2 slot.
pub const SLOT_ROWS: usize = 32;

/// End row of a slot.
pub const fn slot_end(slot: usize) -> usize {
    slot * SLOT_ROWS + (SLOT_ROWS - 1)
}

/// Pad slot follows the max-depth merkle levels plus the four assertion slots.
pub const fn pad_slot() -> usize {
    MAX_DEPTH + 4
}

/// Trace length covering the pad slot.
pub const fn trace_len() -> usize {
    let used = (pad_slot() + 1) * SLOT_ROWS;
    let mut n = 1usize;
    while n < used {
        n *= 2;
    }
    n
}

/// Root assertion row: end of the last real-capable merkle slot.
pub const fn root_row() -> usize {
    slot_end(MAX_DEPTH)
}

pub const fn null_row() -> usize {
    slot_end(MAX_DEPTH + 1)
}

pub const fn out_row() -> usize {
    slot_end(MAX_DEPTH + 2)
}

pub const fn event_row() -> usize {
    slot_end(MAX_DEPTH + 3)
}

pub const BIT: usize = 20;

pub const fn sib_start() -> usize {
    BIT + MAX_DEPTH
}

pub const fn range_start() -> usize {
    sib_start() + MAX_DEPTH * 4
}

pub const fn selector_col() -> usize {
    range_start() + 96
}

pub const fn flag_col() -> usize {
    selector_col() + 1
}

pub const fn width() -> usize {
    flag_col() + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(MAX_DEPTH, 20);
        assert_eq!(pad_slot(), 24);
        assert_eq!(trace_len(), 1024);
        assert_eq!(sib_start(), 40);
        assert!(width() > 132);
    }
}
