# Architecture: observation, state and kernel boundaries

The repository has four layers. Each one hands the next a defined value, and
the only numerical rounding happens at one documented place.

```
 recorded text ──► Exact (i128 ratio, text kept) ─┐
                                                   ├─► whole field per frame ─► kernel inputs (binary64) ─► Kernel V9 ─► outputs (binary64)
 recorded number ─► binary64 ──────────────────────┘      (field_at | fields | Retained | ForwardFill)   ▲
                                                                                              single rounding, exact loss known
```

## 1. Observation layer (`src/exact.rs`, `src/record.rs`)

- **Record contents.** A record declares its variables: name, unit, quantity
  and an exact resolution. Its frames hold only the acquisitions that actually
  happened. Nothing is padded or interpolated.
- **Recorded values** take one of two forms:
  - `Exact`: a JSON string, written as an integer, a decimal or `n/d`. It is
    held as a reduced `i128` ratio, and the text is kept and written back as
    recorded.
  - `Binary64`: a JSON number. JSON is parsed with serde_json
    `float_roundtrip`, so every finite binary64 reads back as written.
- **Identity.** The content identity is SHA-256 over the canonical JSON
  (sorted keys, compact) without the identity field.
  - An exact record's identity hashes the recorded text, so it never passes
    through binary64.
  - Spelling is content: `"0.2000"` and `"1/5"` are equal values but give
    different identities.
- **No silent rounding.** Exact arithmetic is checked: a result outside
  `i128` is `None`.

## 2. Field layer (`src/field.rs`)

The whole field at a frame holds, for every variable acquired so far:

- the value handed to the kernel,
- the exact value, if the record is exact,
- the acquisition frame,
- the age in frames.

A variable not yet acquired is absent.

There are two ways to build it:

- **`field_at(set, frame, policy)`** is the reference definition. It rescans
  from the first frame on every call.
- **`fields(set, policy)`** builds every frame's field in one pass, and is
  tested equal to `field_at` at every frame. It refuses records whose frame
  indices are not strictly increasing.

`Policy` controls how exact values are carried:

- **`Owned`:** each reconstruction allocates a new copy of the text and
  recomputes the binary64 conversion.
- **`Shared`:** text is shared through reference counting, and the conversion
  is computed once per stored value and kept.

Binary64 records are unaffected by the policy. Comparing the policies
separates the cost of representation from the cost of the processing
architecture.

## 3. State layer (`src/state.rs`)

- **`Retained`** is the current field kept as dense slots, one per declared
  variable.
  - Only a frame's own observations write to it.
  - Ages are derived when a snapshot is taken, so a frame with no
    observations costs only the advance.
  - Kernel inputs are read directly from the slots.
- **`ForwardFill`** is the conventional dense table, one row per frame.
  - Each row is the previous row copied, with the frame's observations
    written over it.
  - It keeps every frame's field available for later random access, which
    `Retained` does not.
- **Shared behaviour.** Both use the same `Policy`, and both treat
  re-acquiring an unchanged value as an acquisition: the acquisition frame
  moves forward.

## 4. Kernel boundary (`src/kernel.rs`, `run_kernel` in `src/lib.rs`)

- **Kernel version.** The canonical Kernel V9 is the public `abr-kernel`
  repository at commit `1c1ef70a69b3175a2df0458d0ebf1f71475a3d62`, pinned in
  `Cargo.toml` and `Cargo.lock`. It is used unmodified; this repository
  contains no kernel source.
- **Inputs.**
  - Node values come from the field layer as binary64.
  - For an exact record, that is the nearest binary64 to the recorded value
    (ties to even), and `Exact::f64_loss` gives the loss exactly.
  - For a binary64 record, it is the recorded number.
  - Both records of a fixture pair hold the same acquisitions, so they give
    identical kernel inputs.
- **Calls.**
  - The first evaluated frame uses `operator_e`.
  - Every later frame uses `operator_e_v5` with
    `PersistenceState::from_prior` built from the previous frame's output.
  - Each output is also rebuilt from explicit `operator_a`, `operator_b`,
    `compute_rho` and `operator_r` calls and must be bit-identical.
- **Declared structure.** Nodes `x0..x3`, with Continuation edges
  L0→L1→L2→L3. `rho_base = 0.5` and `chi_0 = 1.0` are illustrative
  parameters of the synthetic domain, not physical declarations.
- **Outputs** are binary64. `kernel::digest` hashes every output bit pattern,
  so implementations can be compared exactly.
- **No relational arithmetic outside the kernel.** The benchmark only selects
  inputs and records outputs.

## Not in this repository

- **Correspondence evaluation.** There are no criteria and no comparison
  against observed targets.
- **The private IRRS code.** The exact-representation design of IRRS commit
  `2d926f0` was used as a behavioural reference only. No IRRS source,
  datasets or evidence are included.
- **Empirical observations.** Every fixture is SIMULATED.
