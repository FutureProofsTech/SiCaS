#!/bin/sh
# Local signing dry run with an ephemeral test key.
#
# This proves the checksum-signing and verification path. It does NOT create a
# production signing identity and must not be presented as a production signature.
set -eu

manifest="${1:-release}"
output="${2:-evidence/signing}"
mkdir -p "$output"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT INT TERM

export GNUPGHOME="$work/gnupg"
mkdir -m 700 "$GNUPGHOME"

gpg --batch --pinentry-mode loopback --passphrase '' \
  --quick-generate-key "SiCaS Local Test <test@example.invalid>" ed25519 sign 0 \
  >"$output/keygen.log" 2>&1

gpg --batch --armor --detach-sign \
  --output "$output/SHA256SUMS.asc" "$manifest/SHA256SUMS"

if gpg --verify "$output/SHA256SUMS.asc" "$manifest/SHA256SUMS" \
    >"$output/verify.log" 2>&1; then
  status="PASS"
else
  status="FAIL"
fi

fingerprint="$(gpg --with-colons --fingerprint 2>/dev/null | awk -F: '$1 == "fpr" { print $10; exit }')"
digest="$(sha256sum "$output/SHA256SUMS.asc" | awk '{ print $1 }')"

printf '# Local Signing Dry Run\n\n' >"$output/SUMMARY.md"
printf -- '- Manifest: `%s`\n' "$manifest" >>"$output/SUMMARY.md"
printf -- '- Test-key fingerprint: `%s`\n' "$fingerprint" >>"$output/SUMMARY.md"
printf -- '- Signature SHA-256: `%s`\n' "$digest" >>"$output/SUMMARY.md"
printf -- '- Verification: %s\n\n' "$status" >>"$output/SUMMARY.md"
printf 'The key is ephemeral and is deleted with the temporary GNUPGHOME.\n' >>"$output/SUMMARY.md"
printf 'This is a process dry run, not a production signing identity.\n' >>"$output/SUMMARY.md"

cat "$output/SUMMARY.md"
[ "$status" = "PASS" ]
