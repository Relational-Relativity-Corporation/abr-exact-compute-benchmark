# abr-exact-compute-benchmark

Status: published with Origin authorization (2026-10-31). All fixtures are
SIMULATED; nothing here is empirical evidence.

This repository measures three properties of relational processing, each
separately:

| Property | Benchmark | Question |
|---|---|---|
| Numerical fidelity | B1 | Do exact rational observations keep their recorded values and content identities through serialization, and at what cost compared with binary64? |
| Computational efficiency | B2 | What does single-pass field reconstruction save over repeated reconstruction, and how much of the cost is representation rather than architecture? |
| Kernel consistency | all | Do identical declared inputs give bit-identical results from the canonical Kernel V9? |

**B3** is an experimental fourth measurement: it compares a retained
relational state, updated only by new observations, against a forward-filled
table.

Kernel V9 is the public [`abr-kernel`](https://github.com/Relational-Relativity-Corporation/abr-kernel)
at commit `1c1ef70a`, used unmodified as a pinned git dependency.

## Results (run-001; one machine; medians)

The machine is an AMD Ryzen 5 7600X running Windows, with rustc 1.97.1 and a
release build. Full tables, quartiles and memory figures are in
[`results/run-001/results.md`](results/run-001/results.md) and
[`docs/CLAIMS.md`](docs/CLAIMS.md).

| Measurement (long-dense-changing: 5,000 frames, 25,000 observations) | binary64 | exact, owned | exact, shared |
|---|---|---|---|
| Values differing from the recorded decimal | 19,804 | 0 | 0 |
| Identity stable after write and read | yes | yes | yes |
| Parse the record | 2.20 ms | 6.50 ms | 6.50 ms |
| `field_at` for every frame | 2.86 s | 8.95 s | 3.48 s |
| `fields`, single pass | 2.87 ms | 6.33 ms | 3.73 ms |
| Kernel V9 over all frames (identical in every mode) | 7.3 ms | 7.3 ms | 7.3 ms |
| Kernel output digest | `d38dfe02…` | `d38dfe02…` | `d38dfe02…` |

What the table shows:

- **Fidelity.** Exact records keep every recorded value. Binary64 records
  differ from the recorded decimal in 56% to 79% of their values, by up to
  1.1 × 10⁻¹⁶ relative.
- **Efficiency.** Single-pass reconstruction is 27× to 1,415× faster than
  rescanning, growing with sequence length. The binary64 records show the
  same effect, so it comes from the algorithm, not from the number
  representation.
- **Representation cost.** Exact values make parsing slower. During
  reconstruction they cost about 2× to 3× binary64 when copied, and about 1.2× to
  1.3× when shared.
- **Retained state (B3).** When only kernel inputs are needed, the retained
  state does about half the work of a forward-filled table or less, in
  986 bytes against 2.56 MB. When every frame needs a full snapshot, it shows
  no clear advantage.

Exact arithmetic and the ABR operators are not claimed to cause any speedup;
see [`docs/CLAIMS.md`](docs/CLAIMS.md).

## Reproduce

```
cargo test --release                                  # correctness and equivalence (14 tests)
cargo run --release --bin gen_fixtures -- check      # committed fixtures = generator
cargo run --release --bin bench -- results/<new-run> # about 5 minutes; --quick for a fast check
```

Requirements:

- a Rust toolchain;
- network access the first time, to fetch abr-kernel at the pinned commit.

No other repository or path is needed.

## Layout

| Path | Contents |
|---|---|
| `src/exact.rs` | exact rational values (i128), recorded text kept, correctly rounded binary64 conversion with exact loss |
| `src/record.rs` | observation records, content identity, validation |
| `src/field.rs` | `field_at` (reference) and `fields` (single pass); owned and shared value policies |
| `src/state.rs` | B3: retained state and the forward-filled table |
| `src/kernel.rs` | canonical Kernel V9 calls, conformance check, output digest |
| `src/fixtures.rs` | deterministic SIMULATED fixture generator |
| `src/measure.rs` | timing (warm-up, repetitions, quartiles) and the counting allocator |
| `src/bin/bench.rs` | runs B1, B2 and B3; writes `results/<run>/` |
| `src/bin/gen_fixtures.rs` | writes or checks `fixtures/` |
| `tests/` | exact values, records and identities, equivalence of reconstructions and kernel outputs |
| `fixtures/` | short fixtures (exact and binary64) and `MANIFEST.json` with every fixture's identity |
| `results/` | benchmark outputs with provenance and checksums |
| `docs/` | `METHODOLOGY.md`, `ARCHITECTURE.md`, `CLAIMS.md` |

## Limitations

- **One machine, operating system and compiler.** Background load was not
  controlled.
- **Variability between measurements exceeds the quartiles.** Differences
  under about 2× between separately measured rows are not reliable.
- **Synthetic workload:** five variables and four kernel nodes.
- **Rescanning baseline.** The `field_at` baseline is a deliberately
  rescanning reference definition, so the B2 ratio applies to replacing that
  algorithm, not to software that already processes in one pass.
- **Illustrative kernel parameters:** `rho_base = 0.5`, `chi_0 = 1.0`.

## License

Apache License 2.0; see [`LICENSE`](LICENSE). Copyright 2026 Metatron
Dynamics, Inc.
