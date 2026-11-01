# Claims and open questions

**Evidence base.** Every figure comes from `results/run-001`:

- SIMULATED fixtures;
- one machine: AMD Ryzen 5 7600X, Windows, rustc 1.97.1, release build;
- medians of 21 runs (5 for `field_at` over long fixtures).

Absolute times are specific to that machine.

## Supported claims

### Numerical fidelity (B1)

1. **Exact records keep the recorded value and their identity.** All 8 exact
   records:
   - re-hash to their content identity after a write and read;
   - re-serialize byte for byte;
   - have no value that differs from the recorded decimal.
2. **Binary64 records cannot hold most recorded decimals.**
   - Between 56% and 79% of their values differ from the recorded decimal:
     172 / 306 in short-sparse-unchanged, 19,804 / 25,000 in
     long-dense-changing.
   - The largest relative difference is 1.1 × 10⁻¹⁶.
   - Their identities are also stable, because JSON numbers are parsed
     exactly (serde_json `float_roundtrip`).
3. **The exact record has a cost:**
   - parsing is 1.3× to 3.0× slower;
   - compact JSON is 3% to 26% larger;
   - the parsed record holds about 11% more memory: 5.87 MB against 5.27 MB
     for long-dense-changing.
   - Serialization is faster for exact records (they write back stored
     text): 437 µs against 932 µs for long-dense-changing.
4. **Per-value costs** (long-dense-changing, median):

   | Operation | Time |
   |---|---|
   | Uncached exact-to-binary64 conversion | 70 ns |
   | Cached conversion | 1.2 ns |
   | Owned copy | 37 ns |
   | Shared clone | 9 ns |
   | Binary64 copy | 0.4 ns |

### Computational efficiency (B2)

**Separating architecture from representation.** In the table below, rows
show the representation effect and columns show the architecture effect.

5. **The single-pass reconstruction removes the rescan.** Building every
   frame's field with `fields` instead of calling `field_at` per frame is
   faster in every case:
   - 27× to 181× faster at 500 frames;
   - 237× to 1,415× faster at 5,000 frames.

   The ratio grows with sequence length because `field_at` rescans from the
   first frame. This is the algorithm, not the number type: the binary64
   records show the same effect (126× to 997× for dense fixtures).
6. **Representation cost depends on how exact values are carried.**

   | long-dense-changing | `field_at` every frame | `fields` single pass |
   |---|---|---|
   | binary64 | 2.86 s | 2.87 ms |
   | exact, owned (copy text, recompute conversion) | 8.95 s | 6.33 ms |
   | exact, shared (shared text, cached conversion) | 3.48 s | 3.73 ms |

   - **Allocation:** sharing cuts what `field_at` allocates from 1.62 GB to
     119 MB, the same as binary64.
   - **Remaining cost:** with sharing, exact values cost about 1.2× to 1.3×
     binary64 under either architecture for this fixture. On short fixtures the
     single-pass ratio is larger (up to 2.4× for short-dense-unchanged) and
     less stable (see the measurement caveats).
7. **Kernel time is the same in every mode.** Over all frames it is 0.64 ms
   (500 frames) or 7.3 ms (5,000 frames). The kernel calls and inputs are
   identical, so none of the speedup above comes from the kernel.

### Kernel consistency

8. **Identical declared inputs give bit-identical Kernel V9 outputs.** Five
   implementations were compared for every fixture:
   - `field_at` and `fields` on the exact records;
   - `fields` on the binary64 records;
   - both value policies;
   - inputs from retained state.

   They give one output digest per fixture. Every frame's output also equals
   the explicit operator composition.

### Continuous relational state (B3)

9. **When only kernel inputs are needed, the retained field does less work
   and holds far less memory.** On 5,000 frames, processing changes and
   reading kernel inputs (changing fixtures):

   | | Retained | Forward-fill table (build + read) |
   |---|---|---|
   | dense, binary64 | 0.38 ms | 0.70 ms |
   | dense, exact | 0.63 ms | 1.32 ms |
   | sparse, binary64 | 0.10 ms | 0.41 ms |
   | sparse, exact | 0.16 ms | 0.80 ms |

   - **Memory:** the state holds 986 bytes; the table holds 2.56 MB.
   - **Maintenance:** advancing the state with no observations costs about
     2 µs per 5,000 frames.
10. **When every frame's full snapshot is needed, there is no clear
    advantage.** Snapshot materialization dominates, at about 2.1 to 2.4 ms
    per 5,000 frames for both approaches:
    - retained plus snapshots: 2.2 to 3.1 ms;
    - forward-fill build plus snapshots: 2.5 to 3.7 ms;
    - `fields` single pass: 1.5 to 3.6 ms.

    The forward-fill table also keeps every frame for random access, which
    the retained state does not.

## Not claimed

- **That exact arithmetic or the ABR operators cause the speedups.** Exact
  values are slower to parse and, without sharing, slower to reconstruct. The
  B2 speedup comes from the single-pass algorithm. The kernel's cost is the
  same in every mode.
- **That these results hold on other machines, compilers or workloads, or
  against other software.** The `field_at` baseline is a rescanning reference
  definition, and software that already processes in one pass would not see
  the B2 ratio.
- **Empirical validity of any kind.** The fixtures are SIMULATED, and the
  kernel parameters (`rho_base = 0.5`, `chi_0 = 1.0`) are illustrative.
- **That retained state is faster in general.** It is faster for the
  kernel-input workload measured here (claim 9), and not when full snapshots
  are needed (claim 10).

## Measurement caveats

- **Variability between measurements exceeds the reported quartiles.** One
  workload was timed in both B2 and B3 (`fields`, binary64,
  short-dense-unchanged) and gave medians of 218 µs and 415 µs. Differences
  smaller than about 2× between rows measured at different times should not
  be relied on; the claims above use larger differences or same-run ratios.
- **Background load on the machine was not controlled.**
- **The counting allocator** adds a small overhead to every allocation, the
  same for all implementations.

## OPEN

- **Other machines:** reproduction on other hardware, operating systems
  (Linux, macOS) and compilers. Not done. The minimum supported Rust version
  is not established.
- **Design attribution:** the exact-value and single-pass designs follow the
  private IRRS reference (exact-representation branch, commit `2d926f0`).
  This repository's implementation was written independently and contains no
  IRRS source. A local cross-check, not part of this repository, found
  identical exact values (61,920) and identical whole fields (22,000 frames)
  across all 8 fixtures. Origin authorized publication on 2026-10-31.
- **Resolved by Origin (2026-10-31):** the license is Apache-2.0, the
  copyright holder is Metatron Dynamics, Inc., and the hardware details in
  `environment.json` are published as provenance.
- **Kernel licensing metadata:** the kernel's `Cargo.toml` declares no
  `license` field; its LICENSE file is Apache-2.0.
- **Build requirement:** building fetches abr-kernel from GitHub at the
  pinned commit, so it needs network access the first time.
- **F-1 demonstration:** the identity loss that serde_json's default float
  parser causes is not reproduced here, because this build always parses
  exactly.

## Dependency and licensing inventory

This repository is Apache-2.0 (Origin approved), Copyright 2026 Metatron Dynamics, Inc. It has three direct
dependencies:

| Crate | Version | License | Source |
|---|---|---|---|
| abr-kernel | 9.0.0 | Apache-2.0 (LICENSE file; no `license` field) | git, Relational-Relativity-Corporation/abr-kernel @ 1c1ef70a |
| serde | 1.0.229 | MIT OR Apache-2.0 | crates.io |
| serde_json | 1.0.154 | MIT OR Apache-2.0 | crates.io |
| sha2 | 0.10.9 | MIT OR Apache-2.0 | crates.io |

Transitive dependencies (crates.io):

- **MIT OR Apache-2.0:** block-buffer, cfg-if, cpufeatures, crypto-common,
  digest, itoa, libc, proc-macro2, quote, serde_core, serde_derive, syn,
  typenum.
- **MIT/Apache-2.0:** version_check.
- **MIT:** generic-array, zmij.
- **Unlicense OR MIT:** memchr.
- **(MIT OR Apache-2.0) AND Unicode-3.0:** unicode-ident.

All are permissive and compatible with Apache-2.0. Exact versions are pinned
in `Cargo.lock`.
