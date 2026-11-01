//! Whole-field reconstruction: at each frame, every variable acquired so far,
//! with its latest actual observation, its acquisition frame and its age.
//! A variable never acquired is absent; nothing is padded or interpolated.
//!
//! * [`field_at`] is the reference definition: it rescans the record from the
//!   first frame on every call.
//! * [`fields`] produces the same field for every frame in a single pass.
//!
//! [`Policy`] selects how exact values are carried: copied with a fresh
//! conversion each time (`Owned`), or shared with one cached conversion
//! (`Shared`). Binary64 records are unaffected by the policy.

use std::collections::BTreeMap;

use crate::exact::Exact;
use crate::record::{ObservationSet, Recorded};

#[derive(Debug, Clone, PartialEq)]
pub struct FieldValue {
    /// Value handed across the kernel boundary.
    pub value: f64,
    /// Recorded exact value, if the record is exact.
    pub exact: Option<Exact>,
    pub acquisition_frame: u64,
    /// Frames since acquisition; 0 = acquired in this frame.
    pub age_frames: u64,
}

pub type Field = BTreeMap<String, FieldValue>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    /// New text copy and a recomputed binary64 conversion per reconstruction.
    Owned,
    /// Shared text and a binary64 conversion computed once per stored value.
    Shared,
}

pub fn carry(v: &Recorded, policy: Policy) -> (f64, Option<Exact>) {
    match (v, policy) {
        (Recorded::Binary64(x), _) => (*x, None),
        (Recorded::Exact(e), Policy::Owned) => (e.to_f64_uncached(), Some(e.deep_copy())),
        (Recorded::Exact(e), Policy::Shared) => (e.to_f64(), Some(e.clone())),
    }
}

/// Reference definition. None if `frame` is not a frame of the record.
pub fn field_at(set: &ObservationSet, frame: u64, policy: Policy) -> Option<Field> {
    if !set.frames.iter().any(|f| f.frame == frame) {
        return None;
    }
    let mut field = Field::new();
    for f in set.frames.iter().take_while(|f| f.frame <= frame) {
        for (name, v) in &f.observations {
            let (value, exact) = carry(v, policy);
            field.insert(name.clone(), FieldValue { value, exact, acquisition_frame: f.frame, age_frames: frame - f.frame });
        }
    }
    Some(field)
}

/// The field at every frame, in one pass. None if frame indices are not strictly increasing.
pub fn fields(set: &ObservationSet, policy: Policy) -> Option<impl Iterator<Item = (u64, Field)> + '_> {
    if set.frames.windows(2).any(|w| w[1].frame <= w[0].frame) {
        return None;
    }
    let mut field = Field::new();
    Some(set.frames.iter().map(move |f| {
        for v in field.values_mut() {
            v.age_frames = f.frame - v.acquisition_frame;
        }
        for (name, v) in &f.observations {
            let (value, exact) = carry(v, policy);
            field.insert(name.clone(), FieldValue { value, exact, acquisition_frame: f.frame, age_frames: 0 });
        }
        (f.frame, field.clone())
    }))
}

/// Bitwise and textual equality of two fields (stricter than `==`, which compares f64 by value).
pub fn identical(a: &Field, b: &Field) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|((ka, va), (kb, vb))| {
        ka == kb && va.value.to_bits() == vb.value.to_bits() && va.acquisition_frame == vb.acquisition_frame
            && va.age_frames == vb.age_frames
            && va.exact.as_ref().map(|e| (e.numer(), e.denom(), e.text().to_string()))
                == vb.exact.as_ref().map(|e| (e.numer(), e.denom(), e.text().to_string()))
    })
}
