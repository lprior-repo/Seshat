#!/usr/bin/env bash
set -euo pipefail

REPO_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
THIRD_PARTY="${REPO_DIR}/third_party"
RESOURCES_SRC="${THIRD_PARTY}/diagrams/resources"
RESOURCES_DEST="${REPO_DIR}/diagram_tool/resources"

echo "=== Vendoring mingrammer/diagrams icons ==="

mkdir -p "${THIRD_PARTY}"

if [[ ! -d "${THIRD_PARTY}/diagrams" ]]; then
	echo "Cloning mingrammer/diagrams..."
	git clone --depth 1 https://github.com/mingrammer/diagrams "${THIRD_PARTY}/diagrams"
else
	echo "Updating existing diagrams clone..."
	cd "${THIRD_PARTY}/diagrams"
	git fetch origin
	git reset --hard origin/master
fi

echo "Syncing resources to diagram_tool/resources/..."
rm -rf "${RESOURCES_DEST:?}"/*
cd "${RESOURCES_SRC}"
find . -type f \( -name "*.png" -o -name "*.svg" \) | while read -r file; do
	dir=$(dirname "${file}")
	mkdir -p "${RESOURCES_DEST}/${dir}"
	cp "${file}" "${RESOURCES_DEST}/${file}"
done

ICON_COUNT=$(find "${RESOURCES_DEST}" -type f \( -name "*.png" -o -name "*.svg" \) | wc -l)
echo "Vendored ${ICON_COUNT} icon files to ${RESOURCES_DEST}"

echo "=== Done ==="
