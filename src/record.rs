//! Observation records: declared variables (unit, resolution) and frames that
//! hold only actual acquisitions. A content identity is assigned when the
//! record is sealed and checked when it is loaded.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::exact::Exact;

/// A recorded value: `Exact` (JSON string) or `Binary64` (JSON number).
#[derive(Debug, Clone, PartialEq)]
pub enum Recorded {
    Exact(Exact),
    Binary64(f64),
}

impl Serialize for Recorded {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Recorded::Exact(e) => e.serialize(s),
            Recorded::Binary64(x) => s.serialize_f64(*x),
        }
    }
}

impl<'de> Deserialize<'de> for Recorded {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Visit;
        impl serde::de::Visitor<'_> for Visit {
            type Value = Recorded;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("an exact value as a string, or a JSON number")
            }
            fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Recorded, E> {
                Exact::parse(v).map(Recorded::Exact).map_err(E::custom)
            }
            fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<Recorded, E> { Ok(Recorded::Binary64(v)) }
            fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Recorded, E> { Ok(Recorded::Binary64(v as f64)) }
            fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Recorded, E> { Ok(Recorded::Binary64(v as f64)) }
        }
        d.deserialize_any(Visit)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub unit: String,
    pub quantity: String,
    /// Declared measurement resolution in `unit` (exact).
    pub resolution: Exact,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    pub frame: u64,
    /// Variables acquired in this frame; absent = not acquired.
    pub observations: BTreeMap<String, Recorded>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservationSet {
    pub content_id: String,
    pub name: String,
    /// Always "SIMULATED" for the fixtures of this repository.
    pub origin: String,
    pub description: String,
    pub variables: Vec<Variable>,
    pub frames: Vec<Frame>,
}

/// SHA-256 (hex) of the record's canonical JSON without its `content_id`:
/// object keys sorted, compact encoding.
pub fn content_id<T: Serialize>(record: &T) -> String {
    let mut v = serde_json::to_value(record).expect("record serializes");
    if let serde_json::Value::Object(m) = &mut v {
        m.remove("content_id");
    }
    hex(&Sha256::digest(serde_json::to_string(&v).expect("canonical JSON").as_bytes()))
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

impl ObservationSet {
    /// Assign the content identity.
    pub fn seal(mut self) -> Self {
        self.content_id = content_id(&self);
        self
    }

    /// Contradictions in the record (empty = none).
    pub fn validate(&self) -> Vec<String> {
        let mut out = Vec::new();
        if content_id(self) != self.content_id {
            out.push(format!("content_id {} does not match the content", self.content_id));
        }
        let declared: BTreeMap<&str, &Variable> = self.variables.iter().map(|v| (v.name.as_str(), v)).collect();
        if declared.len() != self.variables.len() {
            out.push("a variable is declared more than once".into());
        }
        for w in self.frames.windows(2) {
            if w[1].frame <= w[0].frame {
                out.push(format!("frame {} does not follow frame {}", w[1].frame, w[0].frame));
            }
        }
        for f in &self.frames {
            for (name, v) in &f.observations {
                match (declared.get(name.as_str()), v) {
                    (None, _) => out.push(format!("frame {}: undeclared variable {name}", f.frame)),
                    (_, Recorded::Binary64(x)) if !x.is_finite() => out.push(format!("frame {}: {name} not finite", f.frame)),
                    _ => {}
                }
            }
        }
        out
    }

    /// Exact observations that are not integer multiples of their variable's declared resolution.
    pub fn off_resolution(&self) -> Vec<(u64, String)> {
        let res: BTreeMap<&str, &Exact> = self.variables.iter().map(|v| (v.name.as_str(), &v.resolution)).collect();
        let mut out = Vec::new();
        for f in &self.frames {
            for (name, v) in &f.observations {
                if let (Recorded::Exact(e), Some(step)) = (v, res.get(name.as_str())) {
                    if e.on_grid(step) != Some(true) {
                        out.push((f.frame, name.clone()));
                    }
                }
            }
        }
        out
    }

    pub fn n_observations(&self) -> usize {
        self.frames.iter().map(|f| f.observations.len()).sum()
    }
}
