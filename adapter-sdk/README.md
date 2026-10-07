# Adapter SDK

Provider-neutral adapter contract surfaces for future workers.

The canonical semantic authority is:
- `contracts/*.schema.json`
- `foundation-rust/`

Language SDK surfaces exist so future Go, Python and TypeScript adapters can implement the same lifecycle without importing provider logic into the Rust mission core.

Required adapter operations:
1. identify capability;
2. declare contract version;
3. declare simulation support;
4. health/readiness probe;
5. invoke through a versioned envelope;
6. return a stable result/error envelope;
7. never serialize secret material, only opaque secret references.
