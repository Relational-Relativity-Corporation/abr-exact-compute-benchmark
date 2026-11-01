# Methodology

## Fixtures

All fixtures are deterministic and SIMULATED (`src/fixtures.rs`). They are
generated with integer arithmetic only, and they are not empirical
observations.

- **System:** a four-locus chain, `x0..x3`.
  - Unit m, declared resolution 0.0005 m.
  - Recorded at 0.1 mm decimal precision.
  - Most of these decimals are not binary64 values, so a binary64 record must
    round them.
- **Event count:** `n`, unit count, resolution 1.
- **Lengths:** short, 500 frames; long, 5,000 frames.
- **Sampling:**
  - dense: every variable every frame;
  - sparse: frame 1 complete, then each variable with probability 1/8 per
    frame, from a fixed LCG.
- **Dynamics:**
  - unchanged: each re-acquisition repeats the value;
  - changing: each acquisition steps by −7..+7 resolution units, and `n`
    grows by 0..3.
- **Record pairs:** each of the 8 fixtures is a pair of records with the same
  acquisitions.
  - The exact record holds the decimal text.
  - The binary64 record holds the nearest binary64 of each decimal, from
    Rust's correctly rounded parser.
- **Storage:** the short fixtures are committed in `fixtures/`. Every
  fixture's identity is in `fixtures/MANIFEST.json`; long fixtures are
  regenerated and checked against it.
- **Commands:**
  - `cargo run --release --bin gen_fixtures -- check` verifies the committed
    files;
  - `-- write` never replaces a file whose content differs.

## Checks before timing (every run)

The benchmark binary checks the following before it times anything, and stops
if any check fails:

1. Each fixture's identity equals the manifest. Records validate, and every
   exact value lies on its declared resolution grid.
2. **B1:** each record re-hashes to its identity after a pretty-printed write
   and read, and re-serializes byte for byte. Every binary64 record value
   equals the nearest binary64 of the recorded decimal.
3. **B2:** `fields` equals `field_at` at every frame, for each representation
   and policy. Equality covers the value bits, the exact value and text, the
   acquisition frame and the age.
4. **B3:** `Retained` snapshots, `ForwardFill` rows and `fields` are identical
   at every frame.
5. **Kernel:**
   - `operator_e` equals the explicit composition at every frame;
   - outputs from `field_at`, `fields` and `Retained` inputs, from exact and
     binary64 records, under both policies, are bit-identical: one digest per
     fixture across 5 implementations.

## Workloads

**Equal work across implementations.** Implementations compared in one row
read the same record. They produce the same field for every frame of the
sequence, or the same kernel inputs.

**B1, representation.** For each record:

- serialized size;
- time to parse from compact JSON;
- time to serialize;
- time to compute the content identity;
- bytes allocated while parsing.

Per exact value, it also measures:

- the uncached and cached conversion to binary64;
- an owned copy and a shared clone;
- copying a binary64, as a floor.

**B2, field reconstruction.** For binary64, exact-owned and exact-shared:

- `field_at` called for every frame;
- `fields` collected for the whole sequence;
- bytes allocated by each;
- the kernel over all frames, timed separately.

The kernel calls and inputs are identical in every mode, so the kernel time
is not part of the reconstruction comparison.

**B3, continuous state.** Under the shared policy, for binary64 and exact
records:

- Retained:
  - advance only: the frame advance with no observations, i.e. the cost of
    maintaining state;
  - process changes: apply each frame's observations;
  - process changes and read kernel inputs;
  - process changes and materialize a full snapshot per frame;
  - state bytes.
- ForwardFill:
  - build the table;
  - read kernel inputs from every row;
  - materialize a snapshot from every row;
  - table bytes.
- `fields` single pass, for reference.

The derived snapshot cost (with snapshots minus process changes) is labelled
as derived.

## Timing

- **Clock and threads:** wall clock (`std::time::Instant`) on a single
  thread, release build (opt-level 3, codegen-units 1, no LTO, no debug
  assertions).
- **Repetitions:** each measurement discards warm-up runs, then times
  repeated runs.
  - Full runs: 3 warm-up and 21 measured.
  - `field_at` over a long fixture: 1 warm-up and 5 measured, because each
    run takes seconds; its quadratic cost is the point of that row.
  - `--quick` runs use 1 warm-up and 3 measured (1 measured for slow rows),
    and are labelled quick.
- **Reported:** median, quartiles, minimum, maximum and median absolute
  deviation.
- **Black box:** results are passed through `std::hint::black_box` so the
  work is not optimized away.

## Memory

The benchmark binary installs a counting global allocator
(`src/measure.rs`). For a measured closure it reports:

- bytes allocated;
- peak live bytes above the start;
- live bytes still held when it returns.

The counter is a small constant overhead on every allocation, the same for
all implementations. Tests do not install it.

## Provenance and preservation

Each run writes the following to a new directory, and refuses to overwrite an
existing one:

- `environment.json`:
  - compiler (`rustc -vV` at build time), profile and target;
  - CPU, logical CPUs and physical memory;
  - OS;
  - kernel source line from `Cargo.lock`;
  - warm-up procedure.
- `results.json` and `results.md`;
- `SHA256SUMS`.

Runs are kept; a superseded run is marked in `results/README.md`, never
deleted.

## Reproduction

```
cargo test --release
cargo run --release --bin gen_fixtures -- check
cargo run --release --bin bench -- results/<new-run>
```

The kernel is fetched from its public repository at the pinned commit; no
other repository is needed.

## Limits of these measurements

- **One machine, one OS, one compiler.** Absolute times will differ
  elsewhere; the ratios are what the method is meant to show.
- **Uncontrolled background load.** It is visible in the quartiles and
  maxima.
- **Synthetic workload:** five variables and four kernel nodes. Larger
  fields or other access patterns may change the balance between
  reconstruction, snapshot and kernel costs.
- **Scope of each benchmark:**
  - B2 measures a specific reference algorithm (`field_at` rescans).
    Software that already processes sequences in one pass would not see the
    B2 ratio.
  - B3's `ForwardFill` does more than `Retained`: it keeps every frame for
    random access. Where that is needed, the comparison is not like for like.
