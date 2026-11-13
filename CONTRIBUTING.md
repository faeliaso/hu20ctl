# Contributing to `hu20ctl`

Contributions are welcome, especially around hardware protocol fidelity, quality metrics, minutiae extraction, format compatibility, documentation, and test coverage.

## Mandatory Biometric Prohibition

Do not commit, attach to issues or pull requests, or publish real human fingerprints, real biometric templates, custody manifests generated from real fingers, or captured scanner images.

All automated tests and repository fixtures must remain 100% synthetic and deterministic.

## Required Checks Before Opening a Pull Request

Run the full validation suite locally before submitting changes:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./scripts/check-clean-workspace.sh
```

## Architecture Guidelines

- `crates/hu20-core`: Pure biometric logic, codecs, quality assessment, enhancement, minutiae extraction, matching, and custody manifests. Must never depend on USB/hardware crates (`rusb`, `secugen-hu20-rs`).
- `crates/hu20-driver`: USB discovery (`1162:2200` for SecuGen Hamster Pro 20 HU20), transport abstraction, `secugen-hu20-rs` integration, and synthetic mock transport.
- `crates/hu20-cli`: CLI argument parsing, JSON/human output, doctor diagnostics, and Unix-domain socket daemon.

## Code Standards

- Keep functions focused, deterministic, and well-documented.
- Add unit or integration tests for all new behavior using synthetic fixtures.
- Preserve existing custody, privacy, and hardware-isolation guarantees.
