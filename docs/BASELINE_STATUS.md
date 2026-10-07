# Baseline status

Baseline: POLYGLOT_V0_2
Canonical root: D:\TIMED-MISSION-AGENT

## Required proof before sealing

- Rust core compiles and tests pass.
- Rust self-test succeeds.
- Go supervisor tests pass.
- Go supervisor self-test succeeds.
- Node executes the TypeScript worker and its tests pass.
- Python regression suite passes.
- Elixir retry/supervision tests pass when toolchain is present.
- Ada/SPARK source exists; formal proof is only marked proven after GNATprove is installed and run.
- Git worktree is clean after the baseline commit and tag.
