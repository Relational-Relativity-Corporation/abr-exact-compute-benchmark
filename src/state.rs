//! B3: continuous relational state.
//!
//! * [`Retained`]: one current field, updated only by the observations a frame
//!   brings. Ages are derived when a snapshot is materialized, so a frame with
//!   no observations costs only the frame advance.
//! * [`ForwardFill`]: the conventional efficient alternative, a dense table
//!   with one row per frame. Each row is copied from the previous row and the
//!   frame's observations are written over it. Every frame's full field is
//!   available afterwards.
//!
//! Both read the same records under the same [`Policy`] and give the same
//! field at every frame (tests). An acquisition of an unchanged value is still
//! an acquisition: it updates the acquisition frame.

use std::collections::BTreeMap;

use crate::exact::Exact;
use crate::field::{carry, Field, FieldValue, Policy};
use crate::record::{Frame, ObservationSet};

#[derive(Debug, Clone)]
pub struct Slot {
    pub value: f64,
    pub exact: Option<Exact>,
    pub acquisition_frame: u64,
}

/// Declared variables in a fixed order, so state can be dense.
#[derive(Debug, Clone)]
pub struct Layout {
    pub names: Vec<String>,
    index: BTreeMap<String, usize>,
}

impl Layout {
    pub fn of(set: &ObservationSet) -> Self {
        let names: Vec<String> = set.variables.iter().map(|v| v.name.clone()).collect();
        let index = names.iter().enumerate().map(|(i, n)| (n.clone(), i)).collect();
        Layout { names, index }
    }
    pub fn index(&self, name: &str) -> usize { self.index[name] }
}

fn snapshot(layout: &Layout, slots: &[Option<Slot>], frame: u64) -> Field {
    slots.iter().zip(&layout.names).filter_map(|(s, n)| s.as_ref().map(|s| (n.clone(), FieldValue {
        value: s.value, exact: s.exact.clone(), acquisition_frame: s.acquisition_frame, age_frames: frame - s.acquisition_frame }))).collect()
}

fn write(layout: &Layout, slots: &mut [Option<Slot>], f: &Frame, policy: Policy) -> usize {
    for (name, v) in &f.observations {
        let (value, exact) = carry(v, policy);
        slots[layout.index(name)] = Some(Slot { value, exact, acquisition_frame: f.frame });
    }
    f.observations.len()
}

fn read(slots: &[Option<Slot>], nodes: &[usize], out: &mut Vec<f64>) -> bool {
    out.clear();
    for &i in nodes {
        match &slots[i] { Some(s) => out.push(s.value), None => return false }
    }
    true
}

pub struct Retained {
    pub layout: Layout,
    slots: Vec<Option<Slot>>,
    frame: u64,
    policy: Policy,
}

impl Retained {
    pub fn new(set: &ObservationSet, policy: Policy) -> Self {
        let layout = Layout::of(set);
        Retained { slots: vec![None; layout.names.len()], layout, frame: 0, policy }
    }
    /// Apply one frame's observations (the only state change). Returns how many.
    pub fn advance(&mut self, f: &Frame) -> usize {
        self.frame = f.frame;
        write(&self.layout, &mut self.slots, f, self.policy)
    }
    /// Advance to a frame that brings no observation.
    pub fn tick(&mut self, frame: u64) { self.frame = frame; }
    /// Kernel inputs for `nodes` (layout indices); false if one was never acquired.
    pub fn inputs(&self, nodes: &[usize], out: &mut Vec<f64>) -> bool { read(&self.slots, nodes, out) }
    /// The full field at the current frame.
    pub fn snapshot(&self) -> Field { snapshot(&self.layout, &self.slots, self.frame) }
}

pub struct ForwardFill {
    pub layout: Layout,
    pub frames: Vec<u64>,
    pub rows: Vec<Vec<Option<Slot>>>,
}

impl ForwardFill {
    pub fn build(set: &ObservationSet, policy: Policy) -> Self {
        let layout = Layout::of(set);
        let mut rows: Vec<Vec<Option<Slot>>> = Vec::with_capacity(set.frames.len());
        for f in &set.frames {
            let mut row = rows.last().cloned().unwrap_or_else(|| vec![None; layout.names.len()]);
            write(&layout, &mut row, f, policy);
            rows.push(row);
        }
        ForwardFill { layout, frames: set.frames.iter().map(|f| f.frame).collect(), rows }
    }
    pub fn inputs(&self, row: usize, nodes: &[usize], out: &mut Vec<f64>) -> bool { read(&self.rows[row], nodes, out) }
    pub fn snapshot(&self, row: usize) -> Field { snapshot(&self.layout, &self.rows[row], self.frames[row]) }
}
