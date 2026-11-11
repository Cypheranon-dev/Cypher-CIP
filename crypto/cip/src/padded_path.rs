//! Padded Merkle path. One circuit shape for every tree depth.
//!
//! MAX_DEPTH levels are always present. Levels below the real depth
//! mix a sibling. Levels at or above the real depth are the identity:
//! the accumulator is unchanged. A selector marks the boundary.

use winterfell::math::fields::f64::BaseElement;
use winterfell::math::FieldElement;

use crate::layout::MAX_DEPTH;

/// One level of a padded path.
#[derive(Clone, Copy, Debug)]
pub struct Level {
    pub sibling: [BaseElement; 4],
    pub bit: BaseElement,
    /// 1 if this level is real, 0 if padding.
    pub real: BaseElement,
}

/// Apply one level. Real levels mix; padding levels copy.
pub fn step(acc: [BaseElement; 4], level: Level) -> [BaseElement; 4] {
    let mixed = mix(acc, level.sibling, level.bit);
    let mut out = [BaseElement::ZERO; 4];
    for lane in 0..4 {
        out[lane] = level.real * mixed[lane] + (BaseElement::ONE - level.real) * acc[lane];
    }
    out
}

fn mix(acc: [BaseElement; 4], sib: [BaseElement; 4], bit: BaseElement) -> [BaseElement; 4] {
    let mut out = [BaseElement::ZERO; 4];
    for lane in 0..4 {
        out[lane] = bit * sib[lane] + (BaseElement::ONE - bit) * acc[lane];
    }
    out
}

/// Walk a full MAX_DEPTH path. Selectors must be 1 then 0, in order.
pub fn walk(leaf: [BaseElement; 4], levels: &[Level; MAX_DEPTH]) -> Result<[BaseElement; 4], &'static str> {
    let mut seen_pad = false;
    let mut acc = leaf;
    for level in levels {
        let real = level.real == BaseElement::ONE;
        if seen_pad && real {
            return Err("selector not ordered");
        }
        if !real {
            seen_pad = true;
        }
        acc = step(acc, *level);
    }
    Ok(acc)
}

/// Build a path of `depth` real levels and pad the rest.
pub fn pad(real: &[[BaseElement; 4]], bits: &[BaseElement]) -> [Level; MAX_DEPTH] {
    let mut levels = [Level {
        sibling: [BaseElement::ZERO; 4],
        bit: BaseElement::ZERO,
        real: BaseElement::ZERO,
    }; MAX_DEPTH];
    for (i, (sib, bit)) in real.iter().zip(bits).enumerate() {
        if i >= MAX_DEPTH {
            break;
        }
        levels[i] = Level {
            sibling: *sib,
            bit: *bit,
            real: BaseElement::ONE,
        };
    }
    levels
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fe(n: u64) -> BaseElement {
        BaseElement::new(n)
    }

    #[test]
    fn depth3_pads_and_preserves_root() {
        let leaf = [fe(1), fe(2), fe(3), fe(4)];
        let real = [[fe(10), fe(0), fe(0), fe(0)]; 3];
        let bits = [fe(0); 3];
        let levels = pad(&real, &bits);
        let root = walk(leaf, &levels).unwrap();
        // Three real left-child mixes, then identity. Root equals the
        // result of mixing three times, not twenty.
        let mut expect = leaf;
        for sib in &real {
            expect = mix(expect, *sib, fe(0));
        }
        assert_eq!(root, expect);
        assert_eq!(levels[3].real, BaseElement::ZERO);
    }

    #[test]
    fn depth4_and_depth8_same_shape() {
        for depth in [4usize, 8] {
            let leaf = [fe(depth as u64), fe(0), fe(0), fe(0)];
            let real = vec![[fe(1), fe(0), fe(0), fe(0)]; depth];
            let bits = vec![fe(0); depth];
            let levels = pad(&real, &bits);
            assert!(walk(leaf, &levels).is_ok());
            assert_eq!(levels[depth - 1].real, BaseElement::ONE);
            assert_eq!(levels[depth].real, BaseElement::ZERO);
        }
    }

    #[test]
    fn rejects_unordered_selector() {
        let leaf = [fe(1), fe(0), fe(0), fe(0)];
        let mut levels = pad(&[], &[]);
        levels[2].real = BaseElement::ONE;
        assert!(walk(leaf, &levels).is_err());
    }
}
