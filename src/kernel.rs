//! Canonical Kernel V9 (abr-kernel, frozen, unmodified) over a sequence of
//! whole fields.
//!
//! * **First evaluated frame:** `operator_e` (A -> B -> R).
//! * **Every later frame:** `operator_e_v5`, with
//!   `PersistenceState::from_prior` built from the previous frame's output.
//! * **Conformance check:** each output is also rebuilt from explicit
//!   `operator_a` / `operator_b` / `compute_rho` / `operator_r` calls and must
//!   be bit-identical.
//! * **Incomplete fields:** a frame where a declared node has never been
//!   acquired is not evaluated, and the prior carries over.
//!
//! This module only calls the kernel. It adds no relational arithmetic.

use abr_kernel::operators::{
    compute_rho, operator_a, operator_b, operator_e, operator_e_v5, operator_r, DeclaredRelations, EdgeField,
    EdgeProvenance, NodeField, PersistenceState,
};
use sha2::{Digest, Sha256};

use crate::record::hex;

/// Declared kernel inputs (the same for every implementation compared).
#[derive(Debug, Clone)]
pub struct Declaration {
    /// Node variables, in node order.
    pub nodes: Vec<String>,
    /// Spatial edges (src, tgt), all Continuation.
    pub edges: Vec<(usize, usize)>,
    /// ILLUSTRATIVE parameters of the synthetic domain.
    pub rho_base: f64,
    pub chi_0: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub frame: u64,
    pub a: Vec<f64>,
    pub b: Vec<f64>,
    pub rho: Vec<f64>,
    pub r: Vec<f64>,
    pub a_persistence: Option<Vec<f64>>,
}

pub struct Kernel {
    rel: DeclaredRelations,
    rho_base: f64,
    chi_0: f64,
    prior: Option<EdgeField>,
    /// Frames where operator_e differed from the explicit composition.
    pub nonconformant: Vec<u64>,
}

impl Kernel {
    pub fn new(d: &Declaration) -> Self {
        let rel = DeclaredRelations::from_edges_with_provenance(d.nodes.len(), d.edges.clone(),
            vec![EdgeProvenance::Continuation; d.edges.len()]);
        Kernel { rel, rho_base: d.rho_base, chi_0: d.chi_0, prior: None, nonconformant: Vec::new() }
    }

    /// Evaluate one frame from its node inputs (the kernel boundary values).
    pub fn step(&mut self, frame: u64, inputs: &[f64]) -> Step {
        let x = NodeField::new(vec![inputs.to_vec()]);
        let (e, ap) = match self.prior.take() {
            None => (operator_e(&x, &self.rel, &[], &[], self.rho_base, self.chi_0), None),
            Some(prior) => {
                let (e, p) = operator_e_v5(&x, &self.rel, &[], &[], &PersistenceState::from_prior(prior), self.rho_base, self.chi_0);
                (e, Some(p.a_persistence[0].clone()))
            }
        };
        let a = operator_a(&x, &self.rel, &[]);
        let b = operator_b(&a, &self.rel);
        let rho = compute_rho(&a, &self.rel, self.rho_base, self.chi_0);
        let r = operator_r(&b, &self.rel, &rho, &[]);
        if r.spatial != e.spatial {
            self.nonconformant.push(frame);
        }
        let step = Step { frame, a: a.spatial[0].clone(), b: b.spatial[0].clone(), rho, r: e.spatial[0].clone(), a_persistence: ap };
        self.prior = Some(e);
        step
    }
}

/// SHA-256 over the frame numbers and the bit patterns of every output value.
pub fn digest(steps: &[Step]) -> String {
    let mut h = Sha256::new();
    for s in steps {
        h.update(s.frame.to_le_bytes());
        for v in s.a.iter().chain(&s.b).chain(&s.rho).chain(&s.r).chain(s.a_persistence.iter().flatten()) {
            h.update(v.to_bits().to_le_bytes());
        }
    }
    hex(&h.finalize())
}
