#!/usr/bin/env bash
set -euo pipefail

README_FILE="README.md"

if [ ! -f "$README_FILE" ]; then
    echo "Error: $README_FILE not found" >&2
    exit 1
fi

grep -q "test_coverage-100%25-brightgreen" "$README_FILE"
grep -q "doc_coverage-100%25-brightgreen" "$README_FILE"
