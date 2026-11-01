//! Deterministic SIMULATED fixtures: a four-locus sensor chain, generated from
//! integer arithmetic only. These are not empirical observations.
//!
//! **Variables**
//!
//! * `x0`..`x3`: position of locus L0..L3 along the chain axis.
//!   * Unit m, declared resolution 0.0005 m.
//!   * Recorded at 0.1 mm decimal precision (4 decimal places), e.g.
//!     "0.2000", "-0.0035".
//!   * Most of these decimals need rounding to become binary64.
//! * `n`: an event count. Unit count, resolution 1.
//!
//! **Variants**
//!
//! | Variant | Short | Long | Sampling | Values |
//! |---|---|---|---|---|
//! | `dense` | 500 frames | 5,000 frames | every variable every frame | |
//! | `sparse` | | | frame 1 complete, then each variable with probability 1/8 per frame | |
//! | `unchanged` | | | | each re-acquisition repeats the value |
//! | `changing` | | | | each acquisition steps by -7..+7 resolution units (n: +0..3) |
//!
//! **Record pairs**
//!
//! Every fixture comes as a pair of records with the same acquisitions:
//!
//! * the **exact** record holds the recorded decimal text;
//! * the **binary64** record holds the nearest binary64 of that decimal,
//!   from Rust's correctly rounded parser.

use std::collections::BTreeMap;

use crate::exact::Exact;
use crate::kernel::Declaration;
use crate::record::{Frame, ObservationSet, Recorded, Variable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sampling { Dense, Sparse }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dynamics { Unchanged, Changing }

#[derive(Debug, Clone)]
pub struct Spec {
    pub name: String,
    pub frames: u64,
    pub sampling: Sampling,
    pub dynamics: Dynamics,
    pub seed: u64,
    /// Short fixtures are committed under fixtures/; long ones are regenerated and checked by identity.
    pub committed: bool,
}

pub const SHORT: u64 = 500;
pub const LONG: u64 = 5_000;

pub fn specs() -> Vec<Spec> {
    let mut out = Vec::new();
    for (len, frames) in [("short", SHORT), ("long", LONG)] {
        for sampling in [Sampling::Dense, Sampling::Sparse] {
            for dynamics in [Dynamics::Unchanged, Dynamics::Changing] {
                let s = match sampling { Sampling::Dense => "dense", Sampling::Sparse => "sparse" };
                let d = match dynamics { Dynamics::Unchanged => "unchanged", Dynamics::Changing => "changing" };
                out.push(Spec { name: format!("{len}-{s}-{d}"), frames, sampling, dynamics, seed: 0x5eed_0001, committed: len == "short" });
            }
        }
    }
    out
}

/// Declared kernel inputs for the chain: nodes x0..x3, edges L0->L1->L2->L3.
pub fn declaration() -> Declaration {
    Declaration { nodes: vec!["x0".into(), "x1".into(), "x2".into(), "x3".into()], edges: vec![(0, 1), (1, 2), (2, 3)],
                  rho_base: 0.5, chi_0: 1.0 }
}

struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }
}

/// k resolution units of 0.0005 m, as the decimal an instrument with 0.1 mm display precision records.
fn position_text(k: i64) -> String {
    let t = k * 5; // units of 0.0001 m
    format!("{}{}.{:04}", if t < 0 { "-" } else { "" }, t.abs() / 10_000, t.abs() % 10_000)
}

fn variables() -> Vec<Variable> {
    let mut v: Vec<Variable> = (0..4).map(|i| Variable { name: format!("x{i}"), unit: "m".into(),
        quantity: format!("position of locus L{i} along the chain axis"), resolution: Exact::parse("0.0005").unwrap() }).collect();
    v.push(Variable { name: "n".into(), unit: "count".into(), quantity: "event count".into(), resolution: Exact::parse("1").unwrap() });
    v
}

/// (exact record, binary64 record), both sealed.
pub fn generate(spec: &Spec) -> (ObservationSet, ObservationSet) {
    let mut rng = Lcg(spec.seed);
    let mut k: [i64; 4] = [0, 400, 800, 1200];
    let mut n: i64 = 0;
    let (mut exact, mut binary) = (Vec::new(), Vec::new());
    for frame in 1..=spec.frames {
        let (mut ex, mut bi) = (BTreeMap::new(), BTreeMap::new());
        for i in 0..5 {
            let r = rng.next();
            let acquired = frame == 1 || spec.sampling == Sampling::Dense || (r >> 7) % 8 == 0;
            if !acquired { continue; }
            let text = if i < 4 {
                if spec.dynamics == Dynamics::Changing && frame > 1 { k[i] += (r % 15) as i64 - 7; }
                position_text(k[i])
            } else {
                if spec.dynamics == Dynamics::Changing && frame > 1 { n += (r % 4) as i64; }
                n.to_string()
            };
            let name = if i < 4 { format!("x{i}") } else { "n".to_string() };
            bi.insert(name.clone(), Recorded::Binary64(text.parse::<f64>().expect("decimal")));
            ex.insert(name, Recorded::Exact(Exact::parse(&text).expect("exact text")));
        }
        exact.push(Frame { frame, observations: ex });
        binary.push(Frame { frame, observations: bi });
    }
    let what = format!("SIMULATED synthetic fixture '{}' ({} frames, {:?} sampling, {:?} values, seed {:#x}); deterministic \
        generator src/fixtures.rs; not empirical evidence.", spec.name, spec.frames, spec.sampling, spec.dynamics, spec.seed);
    let set = |rep: &str, frames: Vec<Frame>, note: &str| ObservationSet { content_id: String::new(),
        name: format!("{}-{rep}", spec.name), origin: "SIMULATED".into(), description: format!("{what} {note}"),
        variables: variables(), frames }.seal();
    (set("exact", exact, "Values are the recorded decimal text (exact)."),
     set("binary64", binary, "Values are the nearest binary64 of the recorded decimal (correctly rounded)."))
}
