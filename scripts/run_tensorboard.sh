#!/usr/bin/env bash
set -eux

SCRIPT_DIR=$( cd .. -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

tensorboard --logdir="${SCRIPT_DIR}/logs/"
