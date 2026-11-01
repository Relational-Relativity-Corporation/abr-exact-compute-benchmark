//! Benchmarks B1 (numerical representation), B2 (field reconstruction) and
//! B3 (continuous relational state) on the SIMULATED fixtures.
//!
//! Before anything is timed, each run checks:
//!
//! * fixture identities against `fixtures/MANIFEST.json`;
//! * equivalence of every reconstruction at every frame;
//! * bit-identical kernel outputs.
//!
//! It writes `results.json`, `results.md`, `environment.json` and `SHA256SUMS`
//! to a new directory, and refuses to overwrite one.
//!
//! Usage: `cargo run --release --bin bench -- <out_dir> [--quick]`
//!
//! `--quick` uses fewer repetitions and labels the run as quick.

use std::collections::BTreeMap;
use std::process::Command;

use exactbench::exact::Exact;
use exactbench::field::{field_at, fields, identical, Field, Policy};
use exactbench::fixtures::{declaration, generate, specs, Spec, LONG};
use exactbench::kernel::{digest, Kernel};
use exactbench::measure::{allocation, time, Counting, Timing};
use exactbench::record::{content_id, hex, ObservationSet, Recorded};
use exactbench::run_kernel;
use exactbench::state::{ForwardFill, Layout, Retained};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[global_allocator]
static ALLOC: Counting = Counting;

struct Reps { warmup: usize, reps: usize, slow_warmup: usize, slow_reps: usize }

fn sh(cmd: &str, args: &[&str]) -> Option<String> {
    let o = Command::new(cmd).args(args).output().ok()?;
    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

fn environment(reps: &Reps, quick: bool) -> Value {
    let (cpu, mem) = if cfg!(windows) {
        (sh("powershell", &["-NoProfile", "-Command", "(Get-CimInstance Win32_Processor).Name"]),
         sh("powershell", &["-NoProfile", "-Command", "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory"]))
    } else if cfg!(target_os = "macos") {
        (sh("sysctl", &["-n", "machdep.cpu.brand_string"]), sh("sysctl", &["-n", "hw.memsize"]))
    } else {
        let info = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
        let meminfo = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
        (info.lines().find(|l| l.starts_with("model name")).map(|l| l.split(':').nth(1).unwrap_or("").trim().to_string()),
         meminfo.lines().find(|l| l.starts_with("MemTotal")).map(|l| l.to_string()))
    };
    let lock = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.lock"));
    let kernel_source = lock.split("[[package]]").find(|p| p.contains("name = \"abr-kernel\""))
        .and_then(|p| p.lines().find(|l| l.starts_with("source")).map(|l| l.to_string()));
    json!({
        "rustc": env!("BUILD_RUSTC_VV"), "profile": env!("BUILD_PROFILE"), "opt_level": env!("BUILD_OPT_LEVEL"),
        "debug": env!("BUILD_DEBUG"), "target": env!("BUILD_TARGET"),
        "cargo_profile_release": "opt-level 3, codegen-units 1, lto false, debug false (Cargo.toml)",
        "debug_assertions": cfg!(debug_assertions),
        "cpu": cpu, "logical_cpus": std::thread::available_parallelism().map(|n| n.get()).ok(), "physical_memory": mem,
        "os": std::env::consts::OS, "arch": std::env::consts::ARCH,
        "kernel": kernel_source, "kernel_statement": "canonical Kernel V9, abr-kernel at the pinned commit, unmodified",
        "threads": "single-threaded measurement", "clock": "std::time::Instant (monotonic wall clock)",
        "warmup_procedure": format!("{} discarded runs, then {} measured runs (field_at over a long fixture: {} + {}); medians and \
            quartiles reported", reps.warmup, reps.reps, reps.slow_warmup, reps.slow_reps),
        "quick": quick })
}

fn manifest() -> BTreeMap<String, (String, String)> {
    let m: Value = serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/MANIFEST.json"))).unwrap();
    m["fixtures"].as_array().unwrap().iter().map(|e| (e["name"].as_str().unwrap().to_string(),
        (e["exact_content_id"].as_str().unwrap().to_string(), e["binary64_content_id"].as_str().unwrap().to_string()))).collect()
}

fn exact_values(set: &ObservationSet) -> Vec<Exact> {
    set.frames.iter().flat_map(|f| f.observations.values().filter_map(|v| match v { Recorded::Exact(e) => Some(e.clone()), _ => None })).collect()
}

fn t(x: &Timing) -> Value { serde_json::to_value(x).unwrap() }

// ── B1 ──────────────────────────────────────────────────────────────────────

fn b1(spec: &Spec, ex: &ObservationSet, bi: &ObservationSet, r: &Reps) -> Value {
    let mut out = json!({});
    for (rep, set) in [("binary64", bi), ("exact", ex)] {
        let compact = serde_json::to_string(set).unwrap();
        let pretty = serde_json::to_string_pretty(set).unwrap();
        let back: ObservationSet = serde_json::from_str(&pretty).unwrap();
        let identity_stable = content_id(&back) == set.content_id;
        let byte_stable = serde_json::to_string(&back).unwrap() == compact;
        let (loaded, parse_alloc) = allocation(|| serde_json::from_str::<ObservationSet>(&compact).unwrap());
        drop(loaded);
        out[rep] = json!({
            "content_id": set.content_id, "identity_stable_after_write_read": identity_stable, "byte_stable_reserialization": byte_stable,
            "compact_bytes": compact.len(), "pretty_bytes": pretty.len(),
            "parse": t(&time(r.warmup, r.reps, || { std::hint::black_box(serde_json::from_str::<ObservationSet>(&compact).unwrap()); })),
            "serialize": t(&time(r.warmup, r.reps, || { std::hint::black_box(serde_json::to_string(set).unwrap()); })),
            "content_id_time": t(&time(r.warmup, r.reps, || { std::hint::black_box(content_id(set)); })),
            "parse_allocation": parse_alloc });
    }
    // Precision: recorded decimal (exact record) against the binary64 record, observation by observation.
    let (mut n, mut lossy, mut max_rel) = (0u64, 0u64, 0f64);
    for (fe, fb) in ex.frames.iter().zip(&bi.frames) {
        for (ve, vb) in fe.observations.values().zip(fb.observations.values()) {
            let (Recorded::Exact(e), Recorded::Binary64(x)) = (ve, vb) else { unreachable!() };
            let loss = e.checked_sub(&Exact::from_f64(*x).unwrap()).unwrap();
            n += 1;
            if !loss.is_zero() { lossy += 1; max_rel = max_rel.max((loss.to_f64() / e.to_f64()).abs()); }
            assert_eq!(e.to_f64().to_bits(), x.to_bits(), "binary64 record holds the nearest value");
        }
    }
    out["precision"] = json!({ "observations": n,
        "binary64_record_values_differing_from_recorded_decimal": lossy, "max_relative_difference": max_rel,
        "exact_record_values_differing_from_recorded_decimal": 0,
        "kernel_boundary_conversions_with_loss_exact_record": lossy,
        "statement": "the binary64 record carries the rounding loss in the record itself; the exact record carries the recorded value \
                      and incurs the same loss only at the kernel boundary, where it is computed exactly" });
    // Conversion and copy costs per exact value.
    let values = exact_values(ex);
    let per = |x: &Timing| x.median_us * 1e3 / values.len() as f64;
    let fresh: Vec<Exact> = values.iter().map(|v| v.deep_copy()).collect();
    let uncached = time(r.warmup, r.reps, || for v in &fresh { std::hint::black_box(v.to_f64_uncached()); });
    for v in &values { v.to_f64(); }
    let cached = time(r.warmup, r.reps, || for v in &values { std::hint::black_box(v.to_f64()); });
    let deep = time(r.warmup, r.reps, || for v in &values { std::hint::black_box(v.deep_copy()); });
    let shared = time(r.warmup, r.reps, || for v in &values { std::hint::black_box(v.clone()); });
    let bits: Vec<f64> = values.iter().map(|v| v.to_f64()).collect();
    let copy_f64 = time(r.warmup, r.reps, || for v in &bits { std::hint::black_box(*v); });
    out["per_value_ns"] = json!({ "values": values.len(), "exact_to_f64_uncached": per(&uncached), "exact_to_f64_cached": per(&cached),
        "exact_deep_copy": per(&deep), "exact_shared_clone": per(&shared), "binary64_copy": per(&copy_f64) });
    out["fixture"] = json!(spec.name);
    out
}

// ── B2 ──────────────────────────────────────────────────────────────────────

fn b2(spec: &Spec, ex: &ObservationSet, bi: &ObservationSet, r: &Reps, kernel_digests: &mut Vec<(String, String)>) -> Value {
    let decl = declaration();
    let long = spec.frames == LONG;
    let mut out = json!({ "fixture": spec.name });
    let mut kernel_time = None;
    for (label, set, policy) in [("binary64", bi, Policy::Shared), ("exact_owned", ex, Policy::Owned), ("exact_shared", ex, Policy::Shared)] {
        // Equivalence (untimed): field_at and fields agree at every frame; kernel outputs from both.
        let seq: Vec<(u64, Field)> = fields(set, policy).unwrap().collect();
        let mut agree = 0u64;
        for (f, field) in &seq {
            assert!(identical(&field_at(set, *f, policy).unwrap(), field), "{} {label}: frame {f}", spec.name);
            agree += 1;
        }
        let (steps_fields, skipped, nonconformant) = run_kernel(&decl, seq.iter().cloned());
        let (steps_at, _, _) = run_kernel(&decl, set.frames.iter().map(|f| (f.frame, field_at(set, f.frame, policy).unwrap())));
        assert!(skipped.is_empty() && nonconformant.is_empty() && steps_fields == steps_at, "{} {label}: kernel", spec.name);
        let d = digest(&steps_fields);
        kernel_digests.push((format!("{}/{label}", spec.name), d.clone()));
        let (sw, sr) = if long { (r.slow_warmup, r.slow_reps) } else { (r.warmup, r.reps) };
        let at_time = time(sw, sr, || for f in &set.frames { std::hint::black_box(field_at(set, f.frame, policy)); });
        let (_, at_alloc) = allocation(|| for f in &set.frames { std::hint::black_box(field_at(set, f.frame, policy)); });
        let one_time = time(r.warmup, r.reps, || { std::hint::black_box(fields(set, policy).unwrap().collect::<Vec<_>>()); });
        let (_, one_alloc) = allocation(|| { std::hint::black_box(fields(set, policy).unwrap().collect::<Vec<_>>()); });
        if kernel_time.is_none() {
            kernel_time = Some(time(r.warmup, r.reps, || { std::hint::black_box(run_kernel(&decl, seq.iter().cloned())); }));
        }
        out[label] = json!({ "frames_agreeing": agree, "kernel_digest": d,
            "field_at_every_frame": t(&at_time), "field_at_allocation": at_alloc,
            "fields_single_pass": t(&one_time), "fields_allocation": one_alloc,
            "speedup_median": at_time.median_us / one_time.median_us });
    }
    out["kernel_over_all_frames"] = t(kernel_time.as_ref().unwrap());
    out["kernel_statement"] = json!("the same canonical kernel calls on the same inputs in every mode; timed once (fields input, binary64)");
    out
}

// ── B3 ──────────────────────────────────────────────────────────────────────

fn b3(spec: &Spec, ex: &ObservationSet, bi: &ObservationSet, r: &Reps, kernel_digests: &mut Vec<(String, String)>) -> Value {
    let decl = declaration();
    let mut out = json!({ "fixture": spec.name });
    for (label, set) in [("binary64", bi), ("exact_shared", ex)] {
        let policy = Policy::Shared;
        let layout = Layout::of(set);
        let nodes: Vec<usize> = decl.nodes.iter().map(|n| layout.index(n)).collect();
        // Equivalence (untimed): retained snapshot = forward-fill row = fields() at every frame; kernel outputs identical.
        let seq: Vec<(u64, Field)> = fields(set, policy).unwrap().collect();
        let table = ForwardFill::build(set, policy);
        let mut state = Retained::new(set, policy);
        let mut kernel = Kernel::new(&decl);
        let mut steps = Vec::new();
        let mut x = Vec::new();
        for (i, f) in set.frames.iter().enumerate() {
            state.advance(f);
            assert!(identical(&state.snapshot(), &seq[i].1) && identical(&table.snapshot(i), &seq[i].1), "{} {label}: frame {}", spec.name, f.frame);
            assert!(state.inputs(&nodes, &mut x));
            steps.push(kernel.step(f.frame, &x));
        }
        let d = digest(&steps);
        assert_eq!(d, digest(&run_kernel(&decl, seq.iter().cloned()).0));
        kernel_digests.push((format!("{}/{label}/retained", spec.name), d.clone()));

        let w = (r.warmup, r.reps);
        let changes = time(w.0, w.1, || { let mut s = Retained::new(set, policy); for f in &set.frames { s.advance(f); } std::hint::black_box(&s); });
        let maintain = time(w.0, w.1, || { let mut s = Retained::new(set, policy); for f in &set.frames { s.tick(f.frame); } std::hint::black_box(&s); });
        let with_inputs = time(w.0, w.1, || { let mut s = Retained::new(set, policy); let mut x = Vec::new();
            for f in &set.frames { s.advance(f); std::hint::black_box(s.inputs(&nodes, &mut x)); } });
        let with_snapshots = time(w.0, w.1, || { let mut s = Retained::new(set, policy);
            for f in &set.frames { s.advance(f); std::hint::black_box(s.snapshot()); } });
        let (state_after, state_alloc) = allocation(|| { let mut s = Retained::new(set, policy); for f in &set.frames { s.advance(f); } s });
        drop(state_after);
        let build = time(w.0, w.1, || { std::hint::black_box(ForwardFill::build(set, policy)); });
        let (table_kept, table_alloc) = allocation(|| ForwardFill::build(set, policy));
        let table_inputs = time(w.0, w.1, || { let mut x = Vec::new(); for i in 0..table_kept.rows.len() { std::hint::black_box(table_kept.inputs(i, &nodes, &mut x)); } });
        let table_snapshots = time(w.0, w.1, || for i in 0..table_kept.rows.len() { std::hint::black_box(table_kept.snapshot(i)); });
        drop(table_kept);
        let fields_pass = time(w.0, w.1, || { std::hint::black_box(fields(set, policy).unwrap().collect::<Vec<_>>()); });
        out[label] = json!({
            "kernel_digest": d,
            "retained": { "maintain_state_frame_advance_only": t(&maintain), "process_changes": t(&changes),
                          "process_changes_and_read_kernel_inputs": t(&with_inputs), "process_changes_and_materialize_snapshots": t(&with_snapshots),
                          "state_allocation": state_alloc,
                          "derived_snapshot_materialization_median_us": with_snapshots.median_us - changes.median_us },
            "forward_fill": { "build_table": t(&build), "table_allocation": table_alloc,
                              "read_kernel_inputs_all_rows": t(&table_inputs), "materialize_snapshots_all_rows": t(&table_snapshots) },
            "fields_single_pass_snapshots": t(&fields_pass) });
    }
    out
}

fn md_us(v: &Value) -> String {
    let m = v["median_us"].as_f64().unwrap();
    let iqr = v["p75_us"].as_f64().unwrap() - v["p25_us"].as_f64().unwrap();
    if m >= 1000.0 { format!("{:.2} ms (IQR {:.2})", m / 1e3, iqr / 1e3) } else { format!("{:.1} µs (IQR {:.1})", m, iqr) }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out_dir = args.get(1).expect("usage: bench <out_dir> [--quick]");
    let quick = args.iter().any(|a| a == "--quick");
    let out = std::path::Path::new(out_dir);
    assert!(!out.exists(), "refusing to overwrite {}", out.display());
    let reps = if quick { Reps { warmup: 1, reps: 3, slow_warmup: 0, slow_reps: 1 } } else { Reps { warmup: 3, reps: 21, slow_warmup: 1, slow_reps: 5 } };
    let env = environment(&reps, quick);
    let ids = manifest();
    let (mut r1, mut r2, mut r3, mut digests, mut fixtures_used) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for spec in specs() {
        let (ex, bi) = generate(&spec);
        assert_eq!(ids[&spec.name], (ex.content_id.clone(), bi.content_id.clone()), "{}: fixture identity differs from the manifest", spec.name);
        assert!(ex.validate().is_empty() && bi.validate().is_empty() && ex.off_resolution().is_empty());
        eprintln!("{} ({} frames, {} observations)", spec.name, spec.frames, ex.n_observations());
        fixtures_used.push(json!({ "name": spec.name, "frames": spec.frames, "observations": ex.n_observations(),
                                   "exact_content_id": ex.content_id, "binary64_content_id": bi.content_id }));
        r1.push(b1(&spec, &ex, &bi, &reps));
        r2.push(b2(&spec, &ex, &bi, &reps, &mut digests));
        r3.push(b3(&spec, &ex, &bi, &reps, &mut digests));
    }
    // Kernel consistency: one digest per fixture across every representation, policy and reconstruction.
    let mut by_fixture: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (k, d) in &digests { by_fixture.entry(k.split('/').next().unwrap().to_string()).or_default().push(d.clone()); }
    let consistency: Value = by_fixture.iter().map(|(f, ds)| (f.clone(), json!({ "digest": ds[0], "implementations": ds.len(),
        "all_identical": ds.iter().all(|d| d == &ds[0]) }))).collect::<serde_json::Map<_, _>>().into();
    assert!(by_fixture.values().all(|ds| ds.iter().all(|d| d == &ds[0])), "kernel outputs differ between implementations");

    let results = json!({ "label": "SIMULATED fixtures; benchmark of this implementation on one machine",
        "fixtures": fixtures_used, "kernel_consistency": consistency, "b1_representation": r1, "b2_field_reconstruction": r2, "b3_relational_state": r3 });

    let mut md = String::from("# Benchmark results\n\nSIMULATED fixtures. One machine; see environment.json. Medians with interquartile range.\n\n");
    md += "## B1: numerical representation\n\n| fixture | record | stable identity | compact bytes | parse | serialize | values differing from recorded decimal |\n|---|---|---|---|---|---|---|\n";
    for b in &r1 {
        for rep in ["binary64", "exact"] {
            let v = &b[rep];
            let diff = if rep == "exact" { 0 } else { b["precision"]["binary64_record_values_differing_from_recorded_decimal"].as_u64().unwrap() };
            md += &format!("| {} | {rep} | {} | {} | {} | {} | {diff} / {} |\n", b["fixture"].as_str().unwrap(), v["identity_stable_after_write_read"],
                v["compact_bytes"], md_us(&v["parse"]), md_us(&v["serialize"]), b["precision"]["observations"]);
        }
    }
    md += "\nPer exact value (ns, median): see results.json `per_value_ns`.\n\n## B2: field reconstruction (all frames)\n\n| fixture | values | field_at every frame | fields single pass | ratio |\n|---|---|---|---|---|\n";
    for b in &r2 {
        for label in ["binary64", "exact_owned", "exact_shared"] {
            let v = &b[label];
            md += &format!("| {} | {label} | {} | {} | {:.1}x |\n", b["fixture"].as_str().unwrap(), md_us(&v["field_at_every_frame"]),
                md_us(&v["fields_single_pass"]), v["speedup_median"].as_f64().unwrap());
        }
    }
    md += "\n## B3: continuous relational state (all frames)\n\n| fixture | values | retained: advance only | retained: process changes | retained: + snapshots | forward fill: build | forward fill: snapshots | fields single pass |\n|---|---|---|---|---|---|---|---|\n";
    for b in &r3 {
        for label in ["binary64", "exact_shared"] {
            let v = &b[label];
            md += &format!("| {} | {label} | {} | {} | {} | {} | {} | {} |\n", b["fixture"].as_str().unwrap(),
                md_us(&v["retained"]["maintain_state_frame_advance_only"]), md_us(&v["retained"]["process_changes"]),
                md_us(&v["retained"]["process_changes_and_materialize_snapshots"]), md_us(&v["forward_fill"]["build_table"]),
                md_us(&v["forward_fill"]["materialize_snapshots_all_rows"]), md_us(&v["fields_single_pass_snapshots"]));
        }
    }
    md += "\n## Kernel consistency\n\n| fixture | implementations compared | identical | digest |\n|---|---|---|---|\n";
    for (f, v) in consistency.as_object().unwrap() {
        md += &format!("| {f} | {} | {} | `{}` |\n", v["implementations"], v["all_identical"], &v["digest"].as_str().unwrap()[..16]);
    }

    std::fs::create_dir_all(out).unwrap();
    let files = [("environment.json", serde_json::to_string_pretty(&env).unwrap()), ("results.json", serde_json::to_string_pretty(&results).unwrap()),
                 ("results.md", md)];
    let mut sums = String::new();
    for (name, text) in &files {
        std::fs::write(out.join(name), text).unwrap();
        sums += &format!("{}  {name}\n", hex(&Sha256::digest(text.as_bytes())));
    }
    std::fs::write(out.join("SHA256SUMS"), &sums).unwrap();
    print!("{sums}");
}
