# Testing Guide

`hu20ctl` separates **hardware-free automated testing** from **physical scanner testing** so that CI and local development never require physical hardware or real biometric data in the repository.

## 1. Hardware-Free Automated Testing

All workspace unit and integration tests run against deterministic synthetic fixtures (`300×400` L8 @ 500 DPI) and the mock USB transport:

```sh
cargo test --workspace --verbose
```

Verify formatting, lints, and workspace cleanliness:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
./scripts/check-clean-workspace.sh
```

## 2. Physical Scanner Testing (SecuGen HU20)

When testing against a real **SecuGen Hamster Pro 20 (HU20, `1162:2200`)**, use the dedicated physical smoke-test script:

```sh
./scripts/test-physical.sh
```

The script writes temporary captures to a directory outside the repository (`${HU20_OUTPUT_DIR:-$HOME/Downloads}/hu20-physical.XXXXXX`) and never stores biometric material in the workspace.

Supported environment variables for `./scripts/test-physical.sh`:

| Variable | Description | Default |
| --- | --- | --- |
| `HU20_BUS` | USB bus number to target (requires `HU20_ADDRESS`) | Auto-detect |
| `HU20_ADDRESS` | USB device address to target (requires `HU20_BUS`) | Auto-detect |
| `HU20_WAIT` | Set to `1` to pass `--wait` during capture | `0` |
| `HU20_OUTPUT_DIR` | Parent directory for temporary physical test artifacts | `$HOME/Downloads` |

To test disabling hardware Smart Capture auto-exposure manually on physical hardware:

```sh
cargo run -p hu20-cli -- capture --no-auto-exposure --output /tmp/hu20-manual-exp.png
```

### ISO/IEC 19794-2:2011 Template Round-Trip Verification

After capturing two images of the same finger to a protected temporary directory:

```sh
cargo run -p hu20-cli -- template extract /tmp/capture1.png --output /tmp/t1.iso2011
cargo run -p hu20-cli -- template extract /tmp/capture2.png --output /tmp/t2.iso2011
cargo run -p hu20-cli -- compare /tmp/t1.iso2011 /tmp/t2.iso2011 --threshold 40
```

Delete all temporary biometric files when finished.

## 3. Authorized Biometric Dataset Validation

For large-scale accuracy, NFIQ 2, or forensic dataset validation under authorization, follow [docs/FORENSIC_BIOMETRIC_VALIDATION.md](docs/FORENSIC_BIOMETRIC_VALIDATION.md) and [docs/NFIQ2_INTEGRATION.md](docs/NFIQ2_INTEGRATION.md).
