#!/bin/bash
# Test Protection Pre-Commit Hook
# Prevents accidental overwriting of contract test files

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "🔒 Checking protected test files..."

# Protected files with their minimum expected line counts
declare -A PROTECTED_FILES
PROTECTED_FILES=(
	["diagram_tool/src/models/io_tests.rs"]="300"
	["diagram_tool/src/test_infrastructure_tests.rs"]="250"
)

# Check each protected file
VIOLATIONS=0
for FILE in "${!PROTECTED_FILES[@]}"; do
	if [ -f "$FILE" ]; then
		LINES=$(wc -l <"$FILE")
		MIN_LINES=${PROTECTED_FILES[$FILE]}

		if [ "$LINES" -lt "$MIN_LINES" ]; then
			echo -e "${RED}❌ VIOLATION: $FILE has fewer lines than expected ($LINES < $MIN_LINES)${NC}"
			echo -e "${YELLOW}   This may indicate the file was truncated or overwritten!${NC}"
			VIOLATIONS=$((VIOLATIONS + 1))
		else
			echo -e "${GREEN}✓ $FILE (${LINES} lines)${NC}"
		fi
	else
		echo -e "${RED}❌ VIOLATION: $FILE is missing!${NC}"
		echo -e "${YELLOW}   Contract test file was deleted!${NC}"
		VIOLATIONS=$((VIOLATIONS + 1))
	fi
done

# Check for geometry test markers
if [ -d "diagram_tool/src/geometry" ]; then
	GEO_COUNT=$(grep -r "GEO-0" diagram_tool/src/geometry/ | grep -v "TEST_PROTECTION.md" | wc -l || echo "0")
	if [ "$GEO_COUNT" -lt 30 ]; then
		echo -e "${RED}❌ VIOLATION: geometry/ missing GEO test markers ($GEO_COUNT < 30)${NC}"
		VIOLATIONS=$((VIOLATIONS + 1))
	else
		echo -e "${GREEN}✓ geometry/mod.rs (GEO tests present: $GEO_COUNT)${NC}"
	fi
fi

# Check if any protected file is being deleted in this commit
DELETED_TESTS=$(git diff --cached --name-only --diff-filter=D | grep -E "io_tests\.rs|test_infrastructure_tests\.rs" || true)
if [ -n "$DELETED_TESTS" ]; then
	echo -e "${RED}❌ CRITICAL: Attempting to delete protected test files:${NC}"
	echo "$DELETED_TESTS"
	echo -e "${YELLOW}   See .beads/TEST_PROTECTION.md for details${NC}"
	VIOLATIONS=$((VIOLATIONS + 100))
fi

# Final verdict
if [ $VIOLATIONS -gt 0 ]; then
	echo ""
	echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
	echo -e "${RED}🚨 TEST PROTECTION VIOLATIONS DETECTED!${NC}"
	echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
	echo ""
	echo "Protected test files are contract tests for beads."
	echo "See .beads/TEST_PROTECTION.md for details."
	echo ""
	echo "To bypass (only if you know what you're doing):"
	echo "  git commit --no-verify"
	echo ""
	exit 1
fi

echo -e "${GREEN}✅ All protected tests verified!${NC}"
exit 0
