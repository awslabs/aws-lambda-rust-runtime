#!/usr/bin/env bash
set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
export CARGO_TARGET_DIR="$SCRIPT_DIR/../target"

echo "==> Using shared target directory: $CARGO_TARGET_DIR"

for f in *; do
    if [ -d "$f" ]; then
        echo "==> Checking example: $f"
        cd $f
        cargo check
        cd ..
    fi
done
