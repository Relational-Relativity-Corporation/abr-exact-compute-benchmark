//! abr-exact-compute-benchmark: three separately measured properties.
//!
//! * B1 numerical representation: binary64 versus exact rational observation
//!   records ([`exact`], [`record`]).
//! * B2 field reconstruction: repeated `field_at` versus single-pass `fields`,
//!   each under an owned or a shared value policy ([`field`]).
//! * B3 continuous relational state: a retained current field versus a
//!   forward-filled table ([`state`]).
//!
//! Every implementation hands the canonical Kernel V9 the same declared inputs
//! ([`kernel`]); its outputs must be bit-identical. All fixtures are SIMULATED
//! ([`fixtures`]).

pub mod exact;
pub mod field;
pub mod fixtures;
pub mod kernel;
pub mod measure;
pub mod record;
pub mod state;

use field::Field;
use kernel::{Declaration, Kernel, Step};

/// Run the kernel over a sequence of whole fields. Frames whose field lacks a
/// declared node are not evaluated (returned separately).
pub fn run_kernel(decl: &Declaration, fields: impl IntoIterator<Item = (u64, Field)>) -> (Vec<Step>, Vec<u64>, Vec<u64>) {
    let mut k = Kernel::new(decl);
    let (mut steps, mut skipped) = (Vec::new(), Vec::new());
    let mut x = Vec::with_capacity(decl.nodes.len());
    for (frame, field) in fields {
        x.clear();
        x.extend(decl.nodes.iter().map_while(|n| field.get(n).map(|v| v.value)));
        if x.len() == decl.nodes.len() { steps.push(k.step(frame, &x)); } else { skipped.push(frame); }
    }
    (steps, skipped, k.nonconformant)
}
