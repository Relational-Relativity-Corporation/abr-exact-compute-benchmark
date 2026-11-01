# Results

Each run directory contains `environment.json`, `results.json`, `results.md`
and `SHA256SUMS`; check a run's files with `sha256sum -c SHA256SUMS` inside
its directory. Runs are never overwritten or deleted. A superseded run is
marked here.

| Run | Kind | Status | Notes |
|---|---|---|---|
| `run-000-quick` | `--quick` (1 warm-up, 3 measured; slow rows 0 + 1) | exploratory | First end-to-end run, used to check the harness before the full run. Same code, before the license metadata was added to `Cargo.toml`. Its timing variability is higher, so it is not used for claims. |
| `run-001` | full (3 warm-up, 21 measured; slow rows 1 + 5) | current | Source of the figures in README.md and docs/CLAIMS.md. `run-001.log` is its console output. |

All runs: SIMULATED fixtures, one machine (see each `environment.json`).
