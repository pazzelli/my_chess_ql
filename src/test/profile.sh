#!/usr/bin/env bash
set -eux
#DIR="$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd && cd ../.. )"
DIR="/Users/John/IdeaProjects/MyChessQL"


#"${DIR}/cargo" build --dev
#cd "${DIR}"
#cargo build

#/Applications/Xcode.app/Contents/Applications/Instruments.app/Contents/MacOS/Instruments -t "Allocations" -D "${DIR}/test/Trace.trace" "${DIR}/target/debug/my_chess_ql"
/Applications/Xcode.app/Contents/Applications/Instruments.app/Contents/MacOS/Instruments -t "Allocations" -D "${DIR}/src/test/Trace.trace" "${DIR}/target/debug/my_chess_ql"