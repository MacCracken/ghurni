# Contributing to ghurni

## Development Workflow

1. Fork and clone the repository
2. Create a feature branch from `main`
3. Make your changes following the conventions below
4. Ensure all checks pass (`make check`)
5. Open a pull request

## Prerequisites

- Rust stable (MSRV 1.89)
- rustfmt and clippy (`rustup component add rustfmt clippy`)
- Optional: `cargo-audit`, `cargo-deny`

## Cleanliness Check

Run all CI checks locally before submitting:

```bash
cargo fmt --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo test --no-default-features --features std
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
cargo audit
cargo deny check
```

Or simply: `make check`

## Code Conventions

- `#[non_exhaustive]` on all public enums
- `#[must_use]` on all pure functions
- `#[inline]` on hot-path sample processing functions
- `Serialize + Deserialize` on all public types (serde)
- Zero `unwrap`/`panic` in library code
- `no_std` compatible — use `alloc` collections, not `std`
- Feature-gate naad usage with `#[cfg(feature = "naad-backend")]`
- DC blocking filter on all synthesis outputs
- `validate_sample_rate` in constructors, `validate_duration` in synthesize methods
- All types must be `Send + Sync`

## Adding a New Synthesizer

1. Create `src/your_synth.rs` following the dual-impl pattern (naad + fallback)
2. Store `sample_rate`, `sample_position`, `dc_blocker`, and `SmoothedParam` for RPM
3. Implement `Synthesizer` trait
4. Add `#[cfg]`-gated naad fields and fallback RNG
5. Register the module in `src/lib.rs` and add to prelude
6. Add integration tests, benchmarks, and Send/Sync assertion

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0-only.
