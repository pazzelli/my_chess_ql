#!/usr/bin/env bash
set -eux

SCRIPT_DIR=$( cd ../.. -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

init() {
  # Default build behaviour - copy the template file directly to Cargo.toml with no changes
  cp "${SCRIPT_DIR}"/Cargo_template.toml "${SCRIPT_DIR}"/Cargo.toml
}

$@
