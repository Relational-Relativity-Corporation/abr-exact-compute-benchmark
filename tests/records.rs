//! Record identity and fixtures: identities survive serialization; the
//! committed fixtures and the long fixtures match the generator and the
//! manifest; validation catches contradictions.

use exactbench::fixtures::{generate, specs};
use exactbench::record::{content_id, ObservationSet, Recorded};
use serde_json::Value;

fn fixture_dir() -> std::path::PathBuf { std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures") }

#[test]
fn identities_survive_write_and_read_for_both_representations() {
    for spec in specs().into_iter().filter(|s| s.committed) {
        let (ex, bi) = generate(&spec);
        for rec in [&ex, &bi] {
            for text in [serde_json::to_string(rec).unwrap(), serde_json::to_string_pretty(rec).unwrap()] {
                let back: ObservationSet = serde_json::from_str(&text).unwrap();
                assert_eq!(content_id(&back), rec.content_id, "{}", rec.name);
                assert_eq!(serde_json::to_string(&back).unwrap(), serde_json::to_string(rec).unwrap(), "{}: bytes", rec.name);
                assert!(back.validate().is_empty(), "{:?}", back.validate());
            }
        }
    }
}

#[test]
fn exact_values_need_rounding_and_binary64_records_hold_the_nearest_value() {
    let (ex, bi) = generate(&specs()[1]); // short-dense-changing
    let (mut inexact, mut total) = (0, 0);
    for (fe, fb) in ex.frames.iter().zip(&bi.frames) {
        for ((name, ve), (_, vb)) in fe.observations.iter().zip(&fb.observations) {
            let (Recorded::Exact(e), Recorded::Binary64(x)) = (ve, vb) else { panic!("representation") };
            assert_eq!(e.to_f64().to_bits(), x.to_bits(), "{name}");
            total += 1;
            if !e.f64_loss().unwrap().is_zero() { inexact += 1; }
        }
    }
    assert!(inexact > total / 2, "most recorded decimals are not binary64 values ({inexact}/{total})");
    assert!(ex.off_resolution().is_empty(), "every position is a multiple of the declared 0.0005 m");
}

#[test]
fn committed_fixtures_and_long_fixtures_match_the_manifest() {
    let manifest: Value = serde_json::from_str(&std::fs::read_to_string(fixture_dir().join("MANIFEST.json")).unwrap()).unwrap();
    let entries = manifest["fixtures"].as_array().unwrap();
    assert_eq!(entries.len(), specs().len());
    for spec in specs() {
        let m = entries.iter().find(|e| e["name"] == spec.name.as_str()).unwrap();
        let (ex, bi) = generate(&spec);
        assert_eq!((m["exact_content_id"].as_str(), m["binary64_content_id"].as_str()), (Some(ex.content_id.as_str()), Some(bi.content_id.as_str())), "{}", spec.name);
        if spec.committed {
            for (rep, rec) in [("exact", &ex), ("binary64", &bi)] {
                let text = std::fs::read_to_string(fixture_dir().join(format!("{}.{rep}.json", spec.name))).unwrap();
                let on_disk: ObservationSet = serde_json::from_str(&text).unwrap();
                assert!(on_disk.validate().is_empty(), "{}: {:?}", rec.name, on_disk.validate());
                assert_eq!(&on_disk, rec);
            }
        }
    }
}

#[test]
fn spelling_is_content_and_changes_are_detected() {
    let (ex, _) = generate(&specs()[0]);
    let mut v: Value = serde_json::to_value(&ex).unwrap();
    v["frames"][0]["observations"]["x1"] = Value::String("1/5".into()); // same value as "0.2000", other spelling
    let respelled: ObservationSet = serde_json::from_value(v.clone()).unwrap();
    assert!(!respelled.validate().is_empty(), "stored identity no longer matches");
    assert_ne!(content_id(&respelled), ex.content_id);
    v["frames"][1]["frame"] = Value::from(1);
    let disordered: ObservationSet = serde_json::from_value(v).unwrap();
    assert!(disordered.validate().iter().any(|e| e.contains("does not follow")));
    assert!(serde_json::from_str::<Recorded>("\"1e-3\"").is_err() && serde_json::from_str::<Recorded>("true").is_err());
}
