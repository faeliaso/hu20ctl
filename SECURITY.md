# Security and Biometric Data Policy

## Operational Scope and Boundaries

`hu20ctl` is an operational CLI, daemon, and Rust workspace for acquiring, assessing, converting, and comparing fingerprints using the **SecuGen Hamster Pro 20 (HU20)** USB scanner (`1162:2200`).

It is strictly intended for legitimate, authorized biometric capture, enrollment, verification, and forensic/engineering research. Operators must only capture fingerprints with explicit authorization and informed consent where applicable.

## Protection and Lifecycle of Biometric Data

- **No Repository Storage:** Never commit real fingerprint images, raw sensor frames, ISO templates, or `.meta.json` custody manifests to this repository or attach them to public issues or pull requests.
- **Explicit Capture Only:** The CLI and daemon must only initiate hardware capture when explicitly requested by an authorized operator or client.
- **Controlled Storage:** Store captured images and manifests only in restricted local directories with appropriate filesystem permissions.
- **Chain of Custody:** Preserve SHA-256 hashes and `.meta.json` manifests alongside biometric artifacts when forensic traceability is required.
- **Secure Disposal:** Delete temporary captures and test directories (such as those produced by `./scripts/test-physical.sh`) immediately after validation is complete.

## Responsible Vulnerability Reporting

If you discover a security vulnerability, privilege escalation issue (such as `udev` or `systemd` socket permission flaws), or accidental biometric data exposure:

1. **Do not** disclose the issue in a public GitHub issue or pull request.
2. Contact the maintainers privately with reproduction details, affected versions, and impact assessment.
3. Allow reasonable time for diagnosis and a coordinated fix before public disclosure.
