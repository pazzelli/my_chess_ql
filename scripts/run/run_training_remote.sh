#!/usr/bin/env bash
set -eux

SCRIPT_DIR=$( cd ../.. -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

REMOTE_MACHINE="johnpazzelli@Johns-MacBook-Pro.local"
REMOTE_PATH="~/Personal/Github/training"

source "${SCRIPT_DIR}"/scripts/run/build_training.sh

install() {
  # ref: https://caffeinedev.medium.com/how-to-install-tensorflow-on-m1-mac-8e9b91d93706
  ssh "${REMOTE_MACHINE}" 'bash -l -s' < "${SCRIPT_DIR}/scripts/run/run_remote_training_install.sh"
}

copy_files() {
  scp "${SCRIPT_DIR}"/scripts/training/*.py "${REMOTE_MACHINE}:${REMOTE_PATH}/"
  scp "${SCRIPT_DIR}"/target/debug/my_chess_ql "${REMOTE_MACHINE}:${REMOTE_PATH}/"
  scp "${SCRIPT_DIR}"/target/wheels/my_chess_ql*arm64.whl "${REMOTE_MACHINE}:${REMOTE_PATH}/"
  scp -r "${MODEL_DIR}" "${REMOTE_MACHINE}:${REMOTE_PATH}/"

  ssh "${REMOTE_MACHINE}" "mkdir -p ${REMOTE_PATH}/files"
  ssh "${REMOTE_MACHINE}" "mkdir -p ${REMOTE_PATH}/logs/fit"
  scp /Users/John/Documents/chessdata/*.pgn "${REMOTE_MACHINE}:${REMOTE_PATH}/files/"

  ssh "${REMOTE_MACHINE}" "source ~/.bash_profile; conda activate mlp; pip install --force-reinstall ${REMOTE_PATH}/*.whl"
}

start_training() {
  ssh "${REMOTE_MACHINE}" "source ~/.bash_profile; conda activate mlp; python3 ${REMOTE_PATH}/training.py ${REMOTE_PATH}/files ${REMOTE_PATH}/models ${REMOTE_PATH}/models/logs/fit"
}

run() {
  build_m1
  copy_files
#  start_training
}


$@
