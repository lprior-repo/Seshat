#!/usr/bin/env bash

set -euo pipefail

DIR="diagram_tool/src"
MAX_LINES=300
IGNORE_FILE="scripts/.file_lengths_ignore"

echo "Checking file lengths in $DIR (Max $MAX_LINES lines)..."

if [ ! -d "$DIR" ]; then
	echo "Directory $DIR does not exist. Skipping check."
	exit 0
fi

is_production_rust_file() {
	local file="$1"
	case "$file" in
	*/tests/*) return 1 ;;
	*_tests.rs) return 1 ;;
	*/test_*.rs) return 1 ;;
	*/tests.rs) return 1 ;;
	*) return 0 ;;
	esac
}

RUST_FILES=$(find "$DIR" -name "*.rs" -type f)

VIOLATIONS=""
for file in $RUST_FILES; do
	if is_production_rust_file "$file"; then
		LINES=$(wc -l <"$file")
		if [ "$LINES" -gt "$MAX_LINES" ] && ! grep -qxF "$file" "$IGNORE_FILE" 2>/dev/null; then
			VIOLATIONS="$VIOLATIONS\n    $LINES $file"
		fi
	fi
done

if [ -n "$(echo -e "$VIOLATIONS" | tr -d '[:space:]')" ]; then
	echo "ERROR: The following files exceed the $MAX_LINES line limit:"
	echo -e "$VIOLATIONS"
	echo ""
	echo "AGENT REMEDIATION INSTRUCTIONS:"
	echo "1. Do not increase the line limit."
	echo "2. Refactor the file into smaller, focused modules."
	echo "3. Move independent components, models, or logic to separate files."
	echo "4. Follow the Data -> Calc -> Actions pattern to decompose logic."
	exit 1
else
	echo "All file lengths are within the limit (excluding known exceptions)."
	exit 0
fi
