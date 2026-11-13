# Third-Party Notices

`hu20ctl` is licensed under the [MIT License](LICENSE). It builds upon the following open-source Rust crates and native dependencies.

## Rust Workspace Dependencies

| Crate | Version | License |
| --- | --- | --- |
| `secugen-hu20-rs` | `0.1.2` | `MIT` |
| `clap` | `4.6.0` | `MIT OR Apache-2.0` |
| `image` | `0.25.10` | `MIT OR Apache-2.0` |
| `rusb` | `0.9.4` | `MIT` |
| `serde` | `1.0.228` | `MIT OR Apache-2.0` |
| `serde_json` | `1.0.149` | `MIT OR Apache-2.0` |
| `sha2` | `0.10.9` | `MIT OR Apache-2.0` |
| `thiserror` | `2.0.18` | `MIT OR Apache-2.0` |
| `tokio` | `1.50.0` | `MIT` |
| `tracing` | `0.1.44` | `MIT` |
| `tracing-subscriber` | `0.3.23` | `MIT` |
| `uuid` | `1.22.0` | `MIT OR Apache-2.0` |
| `chrono` | `0.4.44` | `MIT OR Apache-2.0` |
| `tempfile` (dev) | `3.27.0` | `MIT OR Apache-2.0` |
| `assert_cmd` (dev) | `2.1.2` | `MIT OR Apache-2.0` |
| `predicates` (dev) | `3.1.4` | `MIT OR Apache-2.0` |

## Vendored `libusb` Notice (`rusb`)

`hu20-driver` enables `rusb` with the `vendored` feature (`libusb1-sys`), which statically compiles and links `libusb` (`LGPL-2.1-or-later`) into binaries when built with that feature.

If you redistribute compiled `hu20ctl` binaries that statically link the vendored `libusb`, ensure compliance with the **GNU Lesser General Public License v2.1 or later (LGPL-2.1-or-later)** (for example, by providing object files or build instructions allowing recipients to relink against a modified `libusb`, or by dynamically linking system `libusb`).
