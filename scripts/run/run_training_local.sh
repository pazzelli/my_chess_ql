#!/usr/bin/env bash
set -eux

SCRIPT_DIR=$( cd ../.. -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

source "${SCRIPT_DIR}"/scripts/run/build_training.sh

PGN_PATH="/Users/John/Documents/chessdata"
LOG_DIR="${SCRIPT_DIR}/logs/fit"

start_tensorboard () {
  tensorboard --logdir="${LOG_DIR}/" --port 6006 --host localhost &
  /usr/bin/open -a "/Applications/Google Chrome.app" 'http://localhost:6006/'
}

start_training() {
  python "${SCRIPT_DIR}"/scripts/training/training.py "${PGN_PATH}" "${MODEL_DIR}" "${LOG_DIR}"
}

run() {
  build_x86
  start_tensorboard
  start_training
}

$@
