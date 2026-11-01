//! Write the committed SIMULATED fixtures and the identity manifest.
//!
//! * Short fixtures: written to `fixtures/` as JSON.
//! * Long fixtures: generated at run time and checked against their
//!   identities in `fixtures/MANIFEST.json`.
//!
//! Usage: `cargo run --release --bin gen_fixtures -- <write|check>`
//!
//! * `write`: refuses to replace a file whose content differs.
//! * `check`: fails if anything differs from what is on disk.

use exactbench::fixtures::{generate, specs};
use serde_json::json;

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_default();
    assert!(mode == "write" || mode == "check", "usage: gen_fixtures <write|check>");
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    let mut files = Vec::new();
    let mut entries = Vec::new();
    for spec in specs() {
        let (ex, bi) = generate(&spec);
        entries.push(json!({ "name": spec.name, "frames": spec.frames, "sampling": format!("{:?}", spec.sampling),
            "dynamics": format!("{:?}", spec.dynamics), "seed": spec.seed, "observations": ex.n_observations(),
            "committed": spec.committed, "exact_content_id": ex.content_id, "binary64_content_id": bi.content_id }));
        if spec.committed {
            files.push((format!("{}.exact.json", spec.name), serde_json::to_string_pretty(&ex).unwrap()));
            files.push((format!("{}.binary64.json", spec.name), serde_json::to_string_pretty(&bi).unwrap()));
        }
    }
    let manifest = json!({ "label": "SIMULATED synthetic fixtures -- not empirical evidence",
        "generator": "src/fixtures.rs (deterministic, integer arithmetic only)", "fixtures": entries });
    files.push(("MANIFEST.json".into(), serde_json::to_string_pretty(&manifest).unwrap()));
    let mut differ = 0;
    for (name, text) in &files {
        let path = dir.join(name);
        match std::fs::read_to_string(&path) {
            Ok(old) if old == *text => {}
            Ok(_) => { differ += 1; eprintln!("DIFFERS: {name}"); }
            Err(_) if mode == "write" => { std::fs::create_dir_all(&dir).unwrap(); std::fs::write(&path, text).unwrap(); println!("wrote {name}"); }
            Err(_) => { differ += 1; eprintln!("MISSING: {name}"); }
        }
    }
    if differ > 0 {
        eprintln!("{differ} file(s) differ from the generator; nothing was replaced");
        std::process::exit(1);
    }
    println!("{} files {}", files.len(), if mode == "write" { "written or already identical" } else { "identical to the generator" });
}
