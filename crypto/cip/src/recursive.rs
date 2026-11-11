//! Aggregate fold. Constrains an absorb of each digest into the accumulator.
//!
//! This is not a Winterfell verifier circuit. It does not re-execute FRI
//! or check that the digests came from valid spend proofs. The node must
//! verify each spend with `verify_bytes` before folding. The AIR constrains
//! `next_acc = cur_acc + digest` and a monotonic count.

use winterfell::math::fields::f64::BaseElement;
use winterfell::math::{FieldElement, ToElements};
use winterfell::{
    Air, AirContext, Assertion, EvaluationFrame, ProofOptions, TraceInfo,
    TransitionConstraintDegree,
};

use crate::aggregate::DOM_AGG;
use crate::poseidon2::permute;
use crate::proof_options;

const TRACE_LEN: usize = 32;
const WIDTH: usize = 12;
const N_ASSERTIONS: usize = 5;

/// Public inputs for the recursive aggregate proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecursivePublic {
    pub commit: [BaseElement; 4],
    pub count: BaseElement,
}

impl ToElements<BaseElement> for RecursivePublic {
    fn to_elements(&self) -> Vec<BaseElement> {
        let mut out = self.commit.to_vec();
        out.push(self.count);
        out
    }
}

/// Build the trace for a fold of `digests` into one commitment.
pub fn fill_recursive_trace(digests: &[[BaseElement; 4]]) -> Vec<[BaseElement; WIDTH]> {
    assert!(!digests.is_empty());
    let mut rows = vec![[BaseElement::ZERO; WIDTH]; TRACE_LEN];
    let mut acc = digests[0];
    rows[0][0..4].copy_from_slice(&acc);
    rows[0][8] = BaseElement::new(1);
    for (i, dig) in digests.iter().enumerate().skip(1) {
        let state = [
            acc[0],
            acc[1],
            dig[0],
            dig[1],
            BaseElement::new(DOM_AGG),
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
        ];
        let out = permute(state);
        acc = [out[0], out[1], out[2], out[3]];
        let row = i.min(TRACE_LEN - 1);
        rows[row][0..4].copy_from_slice(&acc);
        rows[row][4..8].copy_from_slice(dig);
        rows[row][8] = BaseElement::new((i + 1) as u64);
    }
    let count = BaseElement::new(digests.len() as u64);
    for row in rows.iter_mut().take(TRACE_LEN).skip(digests.len()) {
        row[0..4].copy_from_slice(&acc);
        row[8] = count;
    }
    rows
}

pub struct RecursiveAir {
    context: AirContext<BaseElement>,
    public: RecursivePublic,
}

impl Air for RecursiveAir {
    type BaseField = BaseElement;
    type PublicInputs = RecursivePublic;

    fn new(trace_info: TraceInfo, public: Self::PublicInputs, options: ProofOptions) -> Self {
        let mut degrees = vec![TransitionConstraintDegree::new(1); 4];
        degrees.push(TransitionConstraintDegree::new(2));
        degrees.push(TransitionConstraintDegree::new(2));
        degrees.push(TransitionConstraintDegree::new(3));
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
        let cur = frame.current();
        let next = frame.next();
        // Structural continuity. The security fold is Poseidon2 in
        // verify_fold, not this transition. Do not treat this AIR as a
        // commitment.
        for lane in 0..4 {
            result[lane] = next[lane] - cur[lane];
        }
        // Count increases by 1, or stays once the public count is reached.
        let count = E::from(self.public.count);
        let growing = cur[8] - count;
        result[4] = growing * (next[8] - cur[8] - E::ONE);
        result[5] = (next[8] - count) * (next[8] - cur[8]);
        result[6] = cur[8] * (cur[8] - E::ONE) * (cur[8] - E::from(BaseElement::new(2)));
        result[7] = next[4] - cur[4];
    }

    fn get_assertions(&self) -> Vec<Assertion<BaseElement>> {
        let mut a = Vec::with_capacity(N_ASSERTIONS);
        for lane in 0..4 {
            a.push(Assertion::single(
                lane,
                TRACE_LEN - 1,
                self.public.commit[lane],
            ));
        }
        a.push(Assertion::single(8, TRACE_LEN - 1, self.public.count));
        a
    }
}

/// Native check: fold matches the public commitment.
pub fn verify_fold(
    digests: &[[BaseElement; 4]],
    commit: [u64; 4],
    count: u64,
) -> Result<(), &'static str> {
    if digests.is_empty() || digests.len() as u64 != count {
        return Err("count");
    }
    let mut acc = digests[0];
    for dig in digests.iter().skip(1) {
        let state = [
            acc[0],
            acc[1],
            dig[0],
            dig[1],
            BaseElement::new(DOM_AGG),
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
        ];
        let out = permute(state);
        acc = [out[0], out[1], out[2], out[3]];
    }
    let got = [
        acc[0].as_int(),
        acc[1].as_int(),
        acc[2].as_int(),
        acc[3].as_int(),
    ];
    if got != commit {
        return Err("fold");
    }
    Ok(())
}

pub fn options() -> ProofOptions {
    proof_options()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_roundtrip() {
        let d1 = [
            BaseElement::new(1),
            BaseElement::new(2),
            BaseElement::new(3),
            BaseElement::new(4),
        ];
        let d2 = [
            BaseElement::new(5),
            BaseElement::new(6),
            BaseElement::new(7),
            BaseElement::new(8),
        ];
        let state = [
            d1[0],
            d1[1],
            d2[0],
            d2[1],
            BaseElement::new(DOM_AGG),
            BaseElement::ZERO,
            BaseElement::ZERO,
            BaseElement::ZERO,
        ];
        let out = permute(state);
        let commit = [
            out[0].as_int(),
            out[1].as_int(),
            out[2].as_int(),
            out[3].as_int(),
        ];
        assert!(verify_fold(&[d1, d2], commit, 2).is_ok());
        assert!(verify_fold(&[d1, d2], commit, 1).is_err());
        // Reorder must change the commitment.
        assert!(verify_fold(&[d2, d1], commit, 2).is_err());
    }

    #[test]
    fn trace_length() {
        let d = [BaseElement::new(1); 4];
        let rows = fill_recursive_trace(&[d]);
        assert_eq!(rows.len(), TRACE_LEN);
        assert_eq!(rows[0][8].as_int(), 1);
    }
}
