#!/usr/bin/env bash
set -eux

SCRIPT_DIR=$( cd ../.. -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

MODEL_DIR="${SCRIPT_DIR}/models/"

build_init() {
  # Activate the build environment for Cargo
  source "${SCRIPT_DIR}"/venv/bin/activate

  # Remove the rust-tensorflow dependency when building for training since it isn't needed and only currently works with
  # TF v1.15 which is deprecated (and doesn't install well on other machines)
  grep -E -v "^tensorflow" "${SCRIPT_DIR}"/Cargo_template.toml > "${SCRIPT_DIR}"/Cargo.toml
}

build_m1() {
  build_init
  pushd "${SCRIPT_DIR}"
  # Build for the M1's target architecture, passing in a config flag to the compiler to use to strip out the
  # need for the rust-tensorflow dependency
  maturin build --target aarch64-apple-darwin --rustc-extra-args="--cfg 'compile_training'"
  popd
}

build_x86() {
  build_init
  pushd "${SCRIPT_DIR}"
  maturin develop --rustc-extra-args="--cfg 'compile_training'"
  popd
}

#$@
