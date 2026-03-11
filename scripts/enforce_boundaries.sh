#!/usr/bin/env bash

set -e

DIRS=("diagram_tool/src/models" "diagram_tool/src/core")
IGNORE_FILE="scripts/.boundaries_ignore"

echo "Checking for architectural boundary violations..."

VIOLATIONS=""

for dir in "${DIRS[@]}"; do
	if [ -d "$dir" ]; then
		FOUND=$(grep -rnE "use (dioxus|sqlx)::" "$dir" || true)

		while IFS= read -r line; do
			if [ -n "$line" ]; then
				FILE=$(echo "$line" | cut -d: -f1)
				if ! grep -qxF "$FILE" "$IGNORE_FILE" 2>/dev/null; then
					VIOLATIONS="$VIOLATIONS\n$line"
				fi
			fi
		done <<<"$FOUND"
	fi
done

if [ -n "$(echo -e "$VIOLATIONS" | tr -d '[:space:]')" ]; then
	echo "ERROR: Architectural boundary violations found!"
	echo -e "$VIOLATIONS"
	echo ""
	echo "AGENT REMEDIATION INSTRUCTIONS:"
	echo "1. Core and model directories must NOT depend on UI (dioxus) or Database (sqlx) frameworks."
	echo "2. Move dioxus-specific logic to the ui/ directory."
	echo "3. Move sqlx-specific logic to the db/ or store/ directory."
	echo "4. Keep models pure and framework-agnostic."
	exit 1
else
	echo "No boundary violations found."
	exit 0
fi
