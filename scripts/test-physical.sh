#!/usr/bin/env bash
set -euo pipefail

# Runs the hardware smoke test without storing biometric material in the repo.
# Optionally select a scanner with HU20_BUS and HU20_ADDRESS.
if [[ -n "${HU20_BUS:-}" || -n "${HU20_ADDRESS:-}" ]]; then
  if [[ -z "${HU20_BUS:-}" || -z "${HU20_ADDRESS:-}" ]]; then
    echo "Set both HU20_BUS and HU20_ADDRESS, or neither." >&2
    exit 2
  fi
  selected=1
else
  selected=0
fi

download_root="${HU20_OUTPUT_DIR:-${HOME}/Downloads}"
mkdir -p "$download_root"
output_dir="$(mktemp -d "${download_root}/hu20-physical.XXXXXX")"
output_png="$output_dir/capture.png"

echo "1/4 Listing compatible scanners"
cargo run -p hu20-cli -- devices

echo "2/4 Opening scanner"
if [[ "$selected" -eq 1 ]]; then
  cargo run -p hu20-cli -- device inspect --bus "$HU20_BUS" --address "$HU20_ADDRESS"
else
  cargo run -p hu20-cli -- device inspect
fi

wait_flag=""
if [[ "${HU20_WAIT:-0}" == "1" ]]; then
  wait_flag="--wait"
fi

read -r -p "3/4 Place a finger on the scanner, then press Enter to capture... "
if [[ "$selected" -eq 1 ]]; then
  if [[ "${HU20_VARIABLE_DOSE:-0}" == "1" ]]; then
    cargo run -p hu20-cli -- capture --bus "$HU20_BUS" --address "$HU20_ADDRESS" --output "$output_png" --variable-dose $wait_flag
  else
    cargo run -p hu20-cli -- capture --bus "$HU20_BUS" --address "$HU20_ADDRESS" --output "$output_png" $wait_flag
  fi
else
  if [[ "${HU20_VARIABLE_DOSE:-0}" == "1" ]]; then
    cargo run -p hu20-cli -- capture --output "$output_png" --variable-dose $wait_flag
  else
    cargo run -p hu20-cli -- capture --output "$output_png" $wait_flag
  fi
fi

echo "4/4 Validating the captured PNG through the local verification pipeline"
cargo run -p hu20-cli -- verify "$output_png" "$output_png" --threshold 40

echo
echo "Physical test passed. Temporary biometric artifacts: $output_dir"
echo "Review them if needed, then remove the directory manually."
