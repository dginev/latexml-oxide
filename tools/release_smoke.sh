#!/usr/bin/env bash
# release_smoke.sh <latexml_oxide binary> <hello.tex> — the conversion smoke every release leg runs on the binary it
# ships (release.yml; DG 2026-11-21, after rc1's macOS and Windows legs ran only `--version`).
#
# Converts <hello.tex> with the repository's resources/ moved aside, so the binary must serve its dumps, schema and
# XSLT/CSS from memory (CLAUDE.md "Self-contained, portable binary"), and fails unless the conversion exits 0, logs
# no Error or Fatal, and writes HTML that kept the text and carries LaTeXML's ltx_ classes. No TeX Live is needed:
# a basic article converts on the no-dump branch with the embedded bindings (OXIDIZED_DESIGN_DIVERGENCES #1095).
#
# Runs under the bash of every release runner: GNU bash on Linux, Git Bash on Windows, and macOS's bash 3.2.
set -uo pipefail

BIN="${1:?usage: $0 <latexml_oxide binary> <hello.tex>}"
TEX="${2:?usage: $0 <latexml_oxide binary> <hello.tex>}"
[ -x "$BIN" ] || { echo "::error::not an executable: $BIN"; exit 2; }
[ -s "$TEX" ] || { echo "::error::no such input: $TEX"; exit 2; }

# A native Windows binary needs Windows paths; Git Bash's automatic argument conversion skips `--opt=/path`.
native() { if command -v cygpath > /dev/null 2>&1; then cygpath -m "$1"; else printf '%s' "$1"; fi; }

out="$(mktemp -d)"
hidden=""
restore() {
  if [ -n "$hidden" ] && [ -d "$hidden" ]; then mv "$hidden" resources; fi
  rm -rf "$out"
}
trap restore EXIT
if [ -d resources ]; then
  hidden="resources.release-smoke"
  mv resources "$hidden"
fi

NO_COLOR=1 "$BIN" --destination="$(native "$out")/hello.html" --log="$(native "$out")/hello.latexml.log" \
  "$(native "$TEX")" > "$out/log" 2>&1
rc=$?
esc=$'\033'
log="$(sed "s/${esc}\\[[0-9;]*m//g" "$out/log")"
status=0
fail() { echo "::error::release smoke: $1"; status=1; }
[ "$rc" -eq 0 ] || fail "latexml_oxide exited $rc"
errs=$(printf '%s\n' "$log" | grep -cE '^(Error|Fatal):')
[ "$errs" -eq 0 ] || fail "$errs Error/Fatal line(s)"
if [ ! -s "$out/hello.html" ]; then
  fail "no HTML written"
else
  grep -q 'ltx_' "$out/hello.html" || fail "the HTML has no ltx_ classes (embedded CSS/XSLT not applied)"
  grep -q 'Hello World' "$out/hello.html" || fail "the HTML lost the text \"Hello World\""
fi
if [ "$status" -ne 0 ]; then
  printf '%s\n' "$log" | tail -40
  exit 1
fi
echo "OK: $(basename "$TEX") converts clean with resources/ moved aside ($(wc -c < "$out/hello.html" | tr -d ' ') bytes, 0 errors)"
