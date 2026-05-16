#!/usr/bin/env sh
set -eu
BIN="${1:-./rexxlint-portable}"
"${BIN}" ./tests/sample.rexx > ./tests/out.txt || true
grep -q "R001" ./tests/out.txt
grep -q "R003" ./tests/out.txt
grep -q "R009" ./tests/out.txt
echo "portable-c smoke test: ok"
