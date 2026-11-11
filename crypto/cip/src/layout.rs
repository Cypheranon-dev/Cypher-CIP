//! Depth-parameterized trace layout.
//!
//! For depth 3 these formulas reproduce the current hardcoded rows and
//! columns exactly. Higher depths extend the same pattern.

/// Rows per Poseidon2 permutation slot.
pub const SLOT_ROWS: usize = 32;

/// End row of a slot (where the boundary load is applied).
pub const fn slot_end(slot: usize) -> usize {
    slot * SLOT_ROWS + (SLOT_ROWS - 1)
}

/// Merkle slots are 0..depth. Root, nullifier, output, event follow.
/// Pad is the slot after the event.
pub const fn pad_slot(depth: usize) -> usize {
    depth + 4
}

/// Trace length: next power of two covering pad_slot+1 slots.
pub const fn trace_len(depth: usize) -> usize {
    let used = (pad_slot(depth) + 1) * SLOT_ROWS;
    let mut n = 1usize;
    while n < used {
        n *= 2;
    }
    n
}

/// Root assertion: end of the slot after the last merkle level.
pub const fn root_row(depth: usize) -> usize {
    slot_end(depth)
}

/// Nullifier assertion row.
pub const fn null_row(depth: usize) -> usize {
    slot_end(depth + 1)
}

/// Output assertion row.
pub const fn out_row(depth: usize) -> usize {
    slot_end(depth + 2)
}

/// Event assertion row.
pub const fn event_row(depth: usize) -> usize {
    slot_end(depth + 3)
}

/// Bit column start. Fixed; depth bits follow.
pub const BIT: usize = 20;

/// Sibling column start.
pub const fn sib_start(depth: usize) -> usize {
    BIT + depth
}

/// Range column start.
pub const fn range_start(depth: usize) -> usize {
    sib_start(depth) + depth * 4
}

/// Flag column.
pub const fn flag_col(depth: usize) -> usize {
    range_start(depth) + 96
}

/// Trace width.
pub const fn width(depth: usize) -> usize {
    flag_col(depth) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth3_matches_current() {
        assert_eq!(pad_slot(3), 7);
        assert_eq!(trace_len(3), 256);
        assert_eq!(root_row(3), 3 * 32 + 31);
        assert_eq!(null_row(3), 4 * 32 + 31);
        assert_eq!(out_row(3), 5 * 32 + 31);
        assert_eq!(event_row(3), 6 * 32 + 31);
        assert_eq!(sib_start(3), 23);
        assert_eq!(range_start(3), 35);
        assert_eq!(flag_col(3), 131);
        assert_eq!(width(3), 132);
    }

    #[test]
    fn higher_depths() {
        assert_eq!(trace_len(4), 512);
        assert_eq!(pad_slot(4), 8);
        assert_eq!(trace_len(8), 512);
        assert_eq!(pad_slot(8), 12);
    }
}
