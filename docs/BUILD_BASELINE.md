# Construction baseline

## POLYGLOT_V0_2

This baseline is intentionally local-first and dependency-light.

### Core proof targets

- Rust state/SLA library compiles and unit tests pass.
- Go restart supervisor compiles and unit tests pass.
- Node 24 executes TypeScript worker directly and timeout tests pass.
- Python timed invoice core regression remains green.
- Elixir/OTP restart semantics are covered by ExUnit.
- Ada/SPARK formal source is present; proof status remains NOT VERIFIED until GNATprove is installed and executed.
- Cross-language data exchange is versioned under `contracts/`.

### Why this split

Using every language everywhere would increase failure surface. The baseline uses each language only where it has a clear engineering advantage.

### Runtime hierarchy

1. Rust accepts/rejects mission transitions and late results.
2. Go keeps replaceable workers alive.
3. Web/Python workers do specialized execution.
4. Elixir can become the outer availability supervisor when continuous/distributed operation warrants it.
5. SPARK is reserved for tiny invariants where formal proof pays for itself.

### Current delivery boundary

This is a construction baseline, not the final income automation. Browser site adapters, OCR image models, persistent distributed queues, economic scheduler and recovery after full machine reboot are subsequent phases.
