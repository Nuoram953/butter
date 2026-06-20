#!/usr/bin/env bash

set -euo pipefail

REPO="${REPO:-Nuoram953/butter}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="${BINARY_NAME:-butter}"
API_URL="https://api.github.com/repos/${REPO}/releases/latest"

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "This installer only supports Linux tar.gz releases." >&2
  exit 1
fi

for cmd in curl tar python3; do
  if ! command -v "${cmd}" >/dev/null 2>&1; then
    echo "${cmd} is required to install ${BINARY_NAME}." >&2
    exit 1
  fi
done

mkdir -p "${INSTALL_DIR}"

tmp_dir="$(mktemp -d)"
trap 'rm -rf "${tmp_dir}"' EXIT

release_json="${tmp_dir}/release.json"
curl -fsSL \
  -H "Accept: application/vnd.github+json" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  "${API_URL}" \
  -o "${release_json}"

arch="$(uname -m)"
os="$(uname -s)"

mapfile -t release_info < <(
  python3 - "${release_json}" "${arch}" "${os}" <<'PY'
import json
import re
import sys

release_path, arch, os_name = sys.argv[1], sys.argv[2], sys.argv[3]

with open(release_path, "r", encoding="utf-8") as fh:
    release = json.load(fh)

assets = release.get("assets", [])
tarballs = [
    asset
    for asset in assets
    if asset.get("name", "").endswith(".tar.gz")
    and asset.get("browser_download_url")
]

if not tarballs:
    raise SystemExit("No .tar.gz asset found in the latest release.")

os_patterns = {
    "Linux":  (r"linux", r"gnu"),
    "Darwin": (r"darwin", r"apple"),
}

arch_patterns = {
    "x86_64": (r"x86_64", r"amd64"),
    "amd64":  (r"x86_64", r"amd64"),
    "aarch64": (r"aarch64", r"arm64"),
    "arm64":   (r"aarch64", r"arm64"),
}

os_pats   = os_patterns.get(os_name, ())
arch_pats = arch_patterns.get(arch, ())

# Prefer assets matching both OS and arch
selected = next(
    (
        asset
        for asset in tarballs
        if any(re.search(p, asset["name"], re.IGNORECASE) for p in os_pats)
        and any(re.search(p, asset["name"], re.IGNORECASE) for p in arch_pats)
    ),
    None,
)

if selected is None:
    raise SystemExit(f"No matching .tar.gz asset found for {os_name}/{arch}.")

print(release.get("tag_name", "latest"))
print(selected["name"])
print(selected["browser_download_url"])
PY
)

release_tag="${release_info[0]}"
asset_name="${release_info[1]}"
asset_url="${release_info[2]}"

curl -fsSL "${asset_url}" -o "${tmp_dir}/${asset_name}"
tar -xzf "${tmp_dir}/${asset_name}" -C "${tmp_dir}"

binary_path="$(find "${tmp_dir}" -type f -name "${BINARY_NAME}" | head -n 1)"
if [[ -z "${binary_path}" ]]; then
  echo "Could not find binary '${BINARY_NAME}' inside the archive." >&2
  exit 1
fi

install -m 0755 "${binary_path}" "${INSTALL_DIR}/${BINARY_NAME}"

echo "Installed ${release_tag} to ${INSTALL_DIR}/${BINARY_NAME}"

if [[ ":${PATH}:" != *":${INSTALL_DIR}:"* ]]; then
  echo "${INSTALL_DIR} is not on your PATH." >&2
  echo "Add this to your shell profile:" >&2
  echo "  export PATH=\"${INSTALL_DIR}:\$PATH\"" >&2
fi

echo "Run '${BINARY_NAME}' to start the app."
