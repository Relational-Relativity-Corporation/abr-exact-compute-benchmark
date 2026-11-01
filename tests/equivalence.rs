//! Equivalence of the reconstructions and of the canonical kernel results:
//! field_at = fields = retained state = forward-filled table at every frame,
//! under both value policies; identical kernel inputs give bit-identical
//! Kernel V9 outputs across all of them and across representations.

use exactbench::field::{field_at, fields, identical, Field, Policy};
use exactbench::fixtures::{declaration, generate, specs};
use exactbench::kernel::digest;
use exactbench::run_kernel;
use exactbench::state::{ForwardFill, Retained};

#[test]
fn every_reconstruction_gives_the_same_field_at_every_frame() {
    for spec in specs().into_iter().filter(|s| s.committed) {
        let (ex, bi) = generate(&spec);
        for set in [&ex, &bi] {
            let reference: Vec<Field> = set.frames.iter().map(|f| field_at(set, f.frame, Policy::Owned).unwrap()).collect();
            for policy in [Policy::Owned, Policy::Shared] {
                let seq: Vec<(u64, Field)> = fields(set, policy).unwrap().collect();
                let table = ForwardFill::build(set, policy);
                let mut state = Retained::new(set, policy);
                for (i, f) in set.frames.iter().enumerate() {
                    state.advance(f);
                    let at = field_at(set, f.frame, policy).unwrap();
                    assert!(identical(&at, &reference[i]), "{} frame {}: policy", set.name, f.frame);
                    assert!(seq[i].0 == f.frame && identical(&seq[i].1, &reference[i]), "{} frame {}: fields", set.name, f.frame);
                    assert!(identical(&state.snapshot(), &reference[i]), "{} frame {}: retained", set.name, f.frame);
                    assert!(identical(&table.snapshot(i), &reference[i]), "{} frame {}: forward fill", set.name, f.frame);
                }
            }
        }
    }
}

#[test]
fn sparse_fields_carry_age_and_never_pad() {
    let (ex, _) = generate(&specs()[2]); // short-sparse-unchanged
    let seq: Vec<(u64, Field)> = fields(&ex, Policy::Shared).unwrap().collect();
    let aged = seq.iter().flat_map(|(_, f)| f.values()).filter(|v| v.age_frames > 0).count();
    assert!(aged > 0, "sparse acquisition leaves older observations in the field");
    for (frame, f) in &seq {
        for v in f.values() { assert_eq!(v.age_frames, frame - v.acquisition_frame); }
    }
    let mut w = ex.clone();
    w.frames[0].observations.remove("x2");
    let first = field_at(&w, 1, Policy::Shared).unwrap();
    assert!(!first.contains_key("x2"), "a variable not yet acquired is absent");
    w.frames.swap(1, 2);
    assert!(fields(&w, Policy::Shared).is_none(), "out-of-order frames are refused");
}

#[test]
fn identical_kernel_inputs_give_bit_identical_kernel_outputs() {
    let decl = declaration();
    for spec in specs().into_iter().filter(|s| s.committed) {
        let (ex, bi) = generate(&spec);
        let mut digests = Vec::new();
        for set in [&ex, &bi] {
            for policy in [Policy::Owned, Policy::Shared] {
                let by_field_at = set.frames.iter().map(|f| (f.frame, field_at(set, f.frame, policy).unwrap()));
                let (s1, skip1, nc1) = run_kernel(&decl, by_field_at);
                let (s2, _, nc2) = run_kernel(&decl, fields(set, policy).unwrap());
                let mut state = Retained::new(set, policy);
                let s3 = run_kernel(&decl, set.frames.iter().map(|f| { state.advance(f); (f.frame, state.snapshot()) })).0;
                assert!(skip1.is_empty() && nc1.is_empty() && nc2.is_empty(), "{}", set.name);
                assert_eq!(s1, s2);
                assert_eq!(s1, s3);
                digests.push(digest(&s1));
            }
        }
        assert!(digests.windows(2).all(|w| w[0] == w[1]), "{}: {digests:?}", spec.name);
        // Repeated execution is deterministic.
        assert_eq!(digest(&run_kernel(&decl, fields(&ex, Policy::Shared).unwrap()).0), digests[0]);
    }
}

#[test]
fn kernel_outputs_for_the_pinned_kernel_are_unchanged() {
    // Regression guard: digests recorded from abr-kernel 1c1ef70a (Cargo.toml pin) on first build of this
    // repository. A different kernel revision or a change in the kernel inputs changes them. The two
    // unchanged fixtures agree: their kernel inputs are the same at every frame, whatever the sampling.
    let decl = declaration();
    let expected = [
        ("short-dense-unchanged", "90029c5d6fa04322f55605007fa22a6ffae04d040ce408b7a5c814b2389492fc"),
        ("short-dense-changing", "23992eb9777489b777e1dd58421c2a519dea69f3a5521fbd789ed8f410671063"),
        ("short-sparse-unchanged", "90029c5d6fa04322f55605007fa22a6ffae04d040ce408b7a5c814b2389492fc"),
        ("short-sparse-changing", "2880d91501878e56f27b6bb54fb2f7e043678ea4fe07c16c3eb6c7874ca4936f"),
    ];
    let got: Vec<(&str, String)> = expected.iter().map(|(name, _)| {
        let spec = specs().into_iter().find(|s| s.name == *name).unwrap();
        (*name, digest(&run_kernel(&decl, fields(&generate(&spec).0, Policy::Shared).unwrap()).0))
    }).collect();
    let want: Vec<(&str, String)> = expected.iter().map(|(n, d)| (*n, d.to_string())).collect();
    assert_eq!(got, want);
}
