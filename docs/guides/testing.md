# Testing Guide

## Running Tests

```bash
# All features (naad-backend)
cargo test --all-features

# Fallback path (no naad)
cargo test --no-default-features --features std

# Single test
cargo test test_gasoline_engine
```

## Test Categories

### Synthesis Tests
- Every synthesizer type tested with valid parameters
- Output checked for: non-empty, all finite, has energy (non-zero samples)

### Parameter Sweep Tests
- RPM and load swept across full ranges
- Asserts all output samples are finite (no NaN/Inf)

### Continuity Tests
- Verifies process_block produces consistent output across split block boundaries
- Energy comparison between one-block and multi-block synthesis

### Serde Roundtrip Tests
- All public enums serialized to JSON and deserialized back
- Verifies Display output matches

### Event Tests
- Backfire, misfire, knock triggered on engine
- BOV triggered on forced induction
- Output checked for validity after events

### Trait Dispatch Tests
- Multiple synthesizer types accessed through `dyn Synthesizer`
- Verifies trait object dispatch works correctly

## Adding Tests

New synthesizers should have at minimum:
1. Constructor test (valid params)
2. Synthesis test (produces finite output with energy)
3. Serde roundtrip for any new enums
4. Send/Sync assertion in `lib.rs`

## Benchmarks

```bash
cargo bench
```

Criterion benchmarks cover all synthesizers at various block sizes (64-4096 samples).
