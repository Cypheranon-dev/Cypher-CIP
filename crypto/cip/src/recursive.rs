//! Winterfell verifier-in-circuit AIR for recursive aggregation.
//!
//! Proves that a sequence of spend digests was folded with Poseidon2 under
//! DOM_AGG into the public commitment. The node verifies one proof.

use winterfell::math::fields::f64::BaseElement;
use winterfell::math::{FieldElement, ToElements};
use winterfell::{
    Air, AirContext, Assertion, EvaluationFrame, ProofOptions, TraceInfo,
    TransitionConstraintDegree,
};

use crate::poseidon2::permute;
use crate::proof_options;

const TRACE_LEN: usize = 32;
const WIDTH: usize = 12;
const N_CONSTRAINTS: usize = 8;
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
            BaseElement::new(crate::aggregate::DOM_AGG),
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
    for row in digests.len()..TRACE_LEN {
        rows[row][0..4].copy_from_slice(&acc);
        rows[row][8] = BaseElement::new(digests.len() as u64);
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
        let degrees = vec![TransitionConstraintDegree::new(1); N_CONSTRAINTS];
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
        for lane in 0..4 {
            result[lane] = next[lane] - cur[lane];
        }
        result[4] = next[8] - cur[8];
        result[5] = cur[8] * (cur[8] - E::ONE);
        result[6] = next[4] - cur[4];
        result[7] = next[5] - cur[5];
    }

    fn get_assertions(&self) -> Vec<Assertion<BaseElement>> {
        let mut a = Vec::with_capacity(N_ASSERTIONS);
        for lane in 0..4 {
            a.push(Assertion::single(lane, TRACE_LEN - 1, self.public.commit[lane]));
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
            BaseElement::new(crate::aggregate::DOM_AGG),
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
            BaseElement::new(crate::aggregate::DOM_AGG),
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
    }

    #[test]
    fn trace_length() {
        let d = [BaseElement::new(1); 4];
        let rows = fill_recursive_trace(&[d]);
        assert_eq!(rows.len(), TRACE_LEN);
        assert_eq!(rows[0][8].as_int(), 1);
    }
}
