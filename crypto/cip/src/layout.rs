//! Depth-parameterized trace layout.
//!
//! Boundary assertions are derived from depth, not hardcoded row numbers.
//! Trace length is the next power of two covering (depth + 4) slots.
//!
//! Depth 3 (8 leaves) is the current production layout. Depth 4 and 8
//! are supported by the same formulas. Proof generation at those depths
//! requires the column and AIR updates that consume this layout.

/// Rows per Poseidon2 permutation slot.
pub const SLOT_ROWS: usize = 32;

/// Compute the number of permutation slots needed for a given depth.
/// Merkle levels occupy slots 0..depth. Nullifier, output, and event
/// occupy the next three slots.
pub const fn slots_for(depth: usize) -> usize {
    depth + 3
}

/// Trace length: next power of two covering the permutation slots plus one pad slot.
pub const fn trace_len(depth: usize) -> usize {
    let used = (slots_for(depth) + 1) * SLOT_ROWS;
    let mut n = 1usize;
    while n < used {
        n *= 2;
    }
    n
}

/// First padding slot.
pub const fn pad_slot(depth: usize) -> usize {
    slots_for(depth)
}

/// Row at the end of a slot (where the boundary load is applied).
pub const fn slot_end(slot: usize) -> usize {
    slot * SLOT_ROWS + (SLOT_ROWS - 1)
}

/// Root assertion row: end of the last merkle slot.
pub const fn root_row(depth: usize) -> usize {
    slot_end(depth - 1)
}

/// Nullifier assertion row.
pub const fn null_row(depth: usize) -> usize {
    slot_end(depth)
}

/// Output assertion row.
pub const fn out_row(depth: usize) -> usize {
    slot_end(depth + 1)
}

/// Event assertion row.
pub const fn event_row(depth: usize) -> usize {
    slot_end(depth + 2)
}

/// Sibling column start. Bit columns occupy `depth` columns starting at 20.
pub const fn sib_start(depth: usize) -> usize {
    20 + depth
}

/// Range column start. Sibling columns occupy `depth * 4` columns.
pub const fn range_start(depth: usize) -> usize {
    sib_start(depth) + depth * 4
}

/// Trace width: range columns (96) plus flag.
pub const fn width(depth: usize) -> usize {
    range_start(depth) + 96 + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depth3_matches_current_layout() {
        // Current production: depth 3, trace 256, root at slot 2 end?
        // Current code asserts root at slot 3 end (row 127). The new
        // formulas place root at the end of the last merkle slot (slot 2).
        // This is the intended correction. Depth-3 proof gen must be
        // revalidated against the new rows before the gate is claimed.
        assert_eq!(slots_for(3), 6);
        assert_eq!(trace_len(3), 256);
        assert_eq!(pad_slot(3), 6);
        assert_eq!(root_row(3), 2 * 32 + 31);
        assert_eq!(null_row(3), 3 * 32 + 31);
        assert_eq!(out_row(3), 4 * 32 + 31);
        assert_eq!(event_row(3), 5 * 32 + 31);
    }

    #[test]
    fn depth4_and_depth8_sizes() {
        assert_eq!(trace_len(4), 256);
        assert_eq!(slots_for(4), 7);
        assert_eq!(trace_len(8), 512);
        assert_eq!(slots_for(8), 11);
        assert_eq!(width(3), 20 + 3 + 12 + 96 + 1);
        assert_eq!(width(4), 20 + 4 + 16 + 96 + 1);
        assert_eq!(width(8), 20 + 8 + 32 + 96 + 1);
    }
}
