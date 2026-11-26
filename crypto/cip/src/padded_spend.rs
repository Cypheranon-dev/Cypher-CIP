//! Depth-4 padded membership gadget wired as a standalone AIR.
//!
//! Uses the padded path from `padded_path` and the layout constants.
//! This is **not** yet the production spend path. Production remains
//! fixed-depth 3 in `lib.rs`.
//!
//! Trace is short and only checks membership of a leaf under a padded
//! Merkle path of real depth 4 (MAX_DEPTH levels present, only first 4 real).

use winterfell::math::fields::f64::BaseElement;
use winterfell::math::FieldElement;

use crate::layout::MAX_DEPTH;
use crate::padded_path::{pad, walk, Level};

/// Verify a depth-4 membership using the padded walk.
/// Returns the computed root or an error.
pub fn check_depth4(
    leaf: [BaseElement; 4],
    siblings: &[[BaseElement; 4]; 4],
    bits: &[BaseElement; 4],
) -> Result<[BaseElement; 4], &'static str> {
    let levels = pad(siblings, bits);
    // Ensure only first 4 are real
    for (i, level) in levels.iter().enumerate() {
        if i < 4 {
            if level.real != BaseElement::ONE {
                return Err("expected real");
            }
        } else if level.real != BaseElement::ZERO {
            return Err("expected pad");
        }
    }
    walk(leaf, &levels)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fe(n: u64) -> BaseElement {
        BaseElement::new(n)
    }

    #[test]
    fn valid_depth4_membership() {
        let leaf = [fe(1), fe(2), fe(3), fe(4)];
        let sibs = [[fe(10), fe(0), fe(0), fe(0)]; 4];
        let bits = [fe(0); 4];
        let root = check_depth4(leaf, &sibs, &bits).expect("walk");
        // Recompute manually for the first level
        assert_eq!(root, leaf); // all left-child (bit=0) keeps the leaf
    }

    #[test]
    fn invalid_depth_rejects() {
        let leaf = [fe(1), fe(2), fe(3), fe(4)];
        let mut sibs = [[fe(10), fe(0), fe(0), fe(0)]; 4];
        // Corrupt by making a later level real (pad will not do this, so
        // we test the walk's ordered-selector rule via a crafted levels).
        let mut levels = pad(&sibs, &[fe(0); 4]);
        levels[5].real = BaseElement::ONE;
        assert!(walk(leaf, &levels).is_err());
    }

    #[test]
    fn depth3_still_works_via_pad() {
        let leaf = [fe(1), fe(2), fe(3), fe(4)];
        let sibs = [[fe(10), fe(0), fe(0), fe(0)]; 3];
        let bits = [fe(0); 3];
        let levels = pad(&sibs, &bits);
        let root = walk(leaf, &levels).unwrap();
        assert!(root != [BaseElement::ZERO; 4]);
    }
}

/// Optional depth-4 membership that production prepare can call.
/// Returns Ok only for a correctly ordered depth-4 path.
/// Production CipAir remains fixed at depth 3.
pub fn production_optional_depth4(
    leaf: [BaseElement; 4],
    siblings: &[[BaseElement; 4]; 4],
    bits: &[BaseElement; 4],
) -> Result<[BaseElement; 4], &'static str> {
    check_depth4(leaf, siblings, bits)
}

#[cfg(test)]
mod optional_tests {
    use super::*;

    #[test]
    fn optional_depth4_accepts_valid() {
        let leaf = [BaseElement::new(1); 4];
        let sibs = [[BaseElement::new(9); 4]; 4];
        let bits = [BaseElement::ZERO; 4];
        assert!(production_optional_depth4(leaf, &sibs, &bits).is_ok());
    }
}
