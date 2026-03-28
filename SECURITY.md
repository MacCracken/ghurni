# Security Policy

## Scope

ghurni is a pure computation library for mechanical sound synthesis. It performs no I/O, no networking, and contains no `unsafe` code. All synthesis is deterministic from seeded parameters.

## Attack Surface

| Area | Risk | Mitigation |
|------|------|------------|
| Sample rate validation | Division by zero, NaN | Rejects ≤0, NaN, Infinity |
| Duration validation | Allocation panic | Rejects ≤0, NaN, Infinity |
| RPM/load parameters | Out-of-range values | Clamped to valid ranges |
| Firing order length | Mismatched vector | Validated against cylinder count |
| DC blocker coefficient | Oscillation at low SR | R clamped to [0.9, 0.9999] |
| Serde deserialization | Crafted JSON extremes | Enum validation; parameters clamped |
| Buffer lengths | Mismatched buffers | min() fallback in mixer |
| alloc::format! in errors | Error path allocation | Only in constructors/synthesize |

## Reporting Vulnerabilities

Please report security issues via [GitHub Security Advisories](https://github.com/MacCracken/ghurni/security/advisories/new). Do not open public issues for security vulnerabilities.

## Dependencies

| Crate | Purpose | Risk |
|-------|---------|------|
| naad | DSP primitives | Optional; same ecosystem |
| serde | Serialization | Widely audited |
| thiserror | Error derive | Proc-macro only |
| tracing | Structured logging | Optional; zero-cost when disabled |
| libm | Math for no_std | Pure computation, no I/O |
