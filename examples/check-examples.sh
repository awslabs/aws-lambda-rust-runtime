#!/usr/bin/env bash
set -e

for f in *; do
    if [ -d "$f" ]; then
        echo "==> Checking example: $f"
        cd $f
        cargo check
        cd ..
    fi
done
