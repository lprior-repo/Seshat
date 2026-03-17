#!/usr/bin/env bash

set -euo pipefail

ROOT="diagram_tool/src/models"
CANONICAL="diagram_tool/src/models/projection/types.rs"

echo "Checking canonical projection type definitions..."

if [ ! -f "$CANONICAL" ]; then
	echo "ERROR: Canonical projection types file missing: $CANONICAL"
	exit 1
fi

check_symbol() {
	local symbol="$1"
	local matches
	matches=$(grep -R -n --include="*.rs" "$symbol" "$ROOT" || true)
	local violations
	violations=$(printf '%s\n' "$matches" | grep -v "^$CANONICAL:" | grep -v '^$' || true)

	if [ -n "$violations" ]; then
		echo "ERROR: Duplicate canonical type definition detected for '$symbol'"
		echo "$violations"
		return 1
	fi

	return 0
}

check_symbol "pub struct DiagramProjection"
check_symbol "pub enum CyclePolicy"
check_symbol "pub struct EventRecord"

echo "Canonical projection type definitions verified."
