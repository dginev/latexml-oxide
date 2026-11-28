#!/usr/bin/env bash
# artifact_check.sh <latexml_oxide binary> notex|tex [<latex dump sha256> | <dumps dir>]
#
# Which format a shipped binary loads, and what it converts on it (docs/release/RELEASING.md "Every asset is gated";
# docs/parity/DUMP_DESIGN.md "Embedded dumps", row R DUMPTRIM). Every release leg runs it on the binary it ships, and
# release-platform-check.yml on a published one.
#
#   notex  The binary sees no TeX tree: `env -i`, PATH=/nonexistent, TEXMFCNF=/nonexistent (so a linked libkpathsea
#          finds no texmf.cnf either), a fresh HOME and temp dir. It must load the embedded format dump
#          (`Info:latex_dump:embedded using embedded TL<Y>`, then `Loaded N entries from <embedded TL<Y>>` with 0
#          errors); with a third argument, the dump it decompressed into the temp dir must have that sha256 (or the
#          sum of `<dumps dir>/latex.<Y>.dump.txt`). hello.tex, tools/notex_smoke.sh's article and
#          tools/artifact_check/expl3_ltcmd_hooks.tex convert with no Error or Fatal and keep their text.
#   tex    The binary sees the TeX tree on PATH, with a fresh LATEXML_FORMAT_CACHE: hello.tex builds the latex format
#          into it (no `Warning:dump:build_failed`), and tools/artifact_check/raw_package.tex (raw xparse.sty, which
#          loads expl3.sty) and the expl3 document then load that dump, with no "Mismatched LaTeX support files".
#
# Both modes then run tools/release_smoke.sh (resources/ moved aside, the bundled ar5iv.css) on the same view of the
# machine. The repository's resources/ is moved aside throughout: a binary built in this checkout would otherwise find
# its dev-tree dumps (the dump search's `$CARGO_MANIFEST_DIR/../resources/dumps` step) before the embedded ones or
# the cache. Logs are read ANSI-stripped; every check prints one PASS/FAIL/NOTE line; a FAIL exits 1.
#
# Runs under the bash of every release runner: GNU bash on Linux, Git Bash on Windows, and macOS's bash 3.2.
set -uo pipefail

usage="usage: $0 <latexml_oxide binary> notex|tex [<latex dump sha256> | <dumps dir>]"
BIN="${1:?$usage}"
MODE="${2:?$usage}"
EXPECT="${3:-}"
case "$MODE" in notex | tex) ;; *) echo "$usage" >&2; exit 2 ;; esac
[ -x "$BIN" ] && [ -f "$BIN" ] || { echo "::error::not an executable: $BIN"; exit 2; }
BIN="$(cd "$(dirname "$BIN")" && pwd)/$(basename "$BIN")"
TOOLS="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$TOOLS")"
HELLO="$ROOT/latexml_oxide/tests/hello/hello.tex"
FIXTURES="$TOOLS/artifact_check"

# A native Windows binary needs Windows paths; Git Bash's automatic argument conversion skips `--opt=/path`.
native() { if command -v cygpath > /dev/null 2>&1; then cygpath -m "$1"; else printf '%s' "$1"; fi; }
sha256() { if command -v sha256sum > /dev/null 2>&1; then sha256sum | awk '{print $1}'; else shasum -a 256 | awk '{print $1}'; fi; }
esc=$'\033'
strip() { sed "s/${esc}\\[[0-9;]*m//g" "$1" | tr -d '\r'; }
# A format build runs off the conversion's --timeout (format_dumps.rs), so the whole call is bounded here too.
bounded() { if command -v timeout > /dev/null 2>&1; then timeout 1200 "$@"; else "$@"; fi; }

npass=0 nfail=0 nnote=0
pass() { echo "PASS $1: $2"; npass=$((npass + 1)); }
fail() {
  echo "FAIL $1: $2"
  [ -z "${GITHUB_ACTIONS:-}" ] || echo "::error::artifact_check $MODE, $1: $2"
  nfail=$((nfail + 1))
}
note() {
  echo "NOTE $1: $2"
  [ -z "${GITHUB_ACTIONS:-}" ] || echo "::warning::artifact_check $MODE, $1: $2"
  nnote=$((nnote + 1))
}

WORK="$(mktemp -d)"
hidden=""
restore() {
  if [ -n "$hidden" ] && [ -d "$hidden" ]; then mv "$hidden" "$ROOT/resources"; fi
  rm -rf "$WORK"
}
trap restore EXIT

# The expected sum, or the sums of a dumps directory's latex dumps by year, read before resources/ is moved aside.
: > "$WORK/expected"
if [ -n "$EXPECT" ]; then
  if printf '%s' "$EXPECT" | grep -qE '^[0-9a-f]{64}$'; then
    echo "any $EXPECT" > "$WORK/expected"
  elif [ -d "$EXPECT" ]; then
    for f in "$EXPECT"/latex.*.dump.txt; do
      [ -f "$f" ] || continue
      y="$(basename "$f" | sed -n 's/^latex\.\([0-9]\{4\}\)\.dump\.txt$/\1/p')"
      [ -n "$y" ] && echo "$y $(sha256 < "$f")" >> "$WORK/expected"
    done
    [ -s "$WORK/expected" ] || { echo "::error::no latex.<year>.dump.txt in $EXPECT"; exit 2; }
  else
    echo "::error::neither a sha256 nor a directory: $EXPECT"; exit 2
  fi
fi
if [ -e "$ROOT/resources.artifact-check" ]; then
  echo "::error::$ROOT/resources.artifact-check exists (a killed earlier run?): restore it to resources/ first"; exit 2
fi
if [ -d "$ROOT/resources" ]; then
  hidden="$ROOT/resources.artifact-check"
  mv "$ROOT/resources" "$hidden" || { echo "::error::cannot move $ROOT/resources aside"; exit 2; }
fi

# The binary as this mode's machine sees it: a wrapper that tools/notex_smoke.sh and tools/release_smoke.sh run as
# the binary. The format cache's directory name is what the logs are searched for (a Windows log prints `\`).
mkdir -p "$WORK/home" "$WORK/tmp" "$WORK/format-cache-artifact-check" "$WORK/out"
WRAP="$WORK/latexml_oxide"
{
  echo '#!/usr/bin/env bash'
  if [ "$MODE" = notex ]; then
    # Linux and macOS clear the environment (`env -i`). Windows keeps it: Git Bash's env -i cannot be exercised before a
    # tag (platform-check runs the tag's own copy), and PATH=/nonexistent plus TEXMFCNF still hide kpsewhich and
    # texmf.cnf; the runner has no TeX. LATEXML_* settings are unset either way.
    if command -v cygpath > /dev/null 2>&1; then
      printf 'exec env'
      for v in $(compgen -e | grep '^LATEXML_'); do printf ' -u %q' "$v"; done
    else
      printf 'exec env -i'
    fi
    printf ' PATH=/nonexistent TEXMFCNF=/nonexistent NO_COLOR=1 HOME=%q TMPDIR=%q' \
      "$(native "$WORK/home")" "$(native "$WORK/tmp")"
    # Windows: SYSTEMROOT/WINDIR as the runner has them, and fresh temp and profile directories.
    if command -v cygpath > /dev/null 2>&1; then
      printf ' SYSTEMROOT=%q WINDIR=%q TMP=%q TEMP=%q USERPROFILE=%q LOCALAPPDATA=%q APPDATA=%q' \
        "${SYSTEMROOT:-C:\\Windows}" "${WINDIR:-C:\\Windows}" "$(native "$WORK/tmp")" "$(native "$WORK/tmp")" \
        "$(native "$WORK/home")" "$(native "$WORK/home")" "$(native "$WORK/home")"
    fi
  else
    # The tree on PATH, but none of this shell's LATEXML_* settings (a LATEXML_DUMP_DIR would bypass the cache).
    printf 'exec env'
    for v in $(compgen -e | grep '^LATEXML_'); do printf ' -u %q' "$v"; done
    printf ' NO_COLOR=1 LATEXML_FORMAT_CACHE=%q TMPDIR=%q' \
      "$(native "$WORK/format-cache-artifact-check")" "$(native "$WORK/tmp")"
    if command -v cygpath > /dev/null 2>&1; then
      printf ' TMP=%q TEMP=%q' "$(native "$WORK/tmp")" "$(native "$WORK/tmp")"
    fi
  fi
  printf ' %q "$@"\n' "$BIN"
} > "$WRAP"
chmod +x "$WRAP"

# convert <name> <tex>: the conversion's exit code in $rc, its ANSI-stripped log in $WORK/out/<name>.txt, its HTML's
# text (tags dropped, whitespace collapsed) in $WORK/out/<name>.text, and its Error/Fatal count in $ef.
convert() {
  local name="$1" tex="$2" o="$WORK/out"
  (cd "$WORK/out" && bounded "$WRAP" --timeout=180 --destination="$(native "$o/$name.html")" \
    --log="$(native "$o/$name.latexml.log")" "$(native "$tex")") > "$o/$name.raw" 2>&1
  rc=$?
  strip "$o/$name.raw" > "$o/$name.txt"
  tr '\n\r' '  ' < "$o/$name.html" 2> /dev/null | sed 's/<[^>]*>/ /g; s/[[:space:]][[:space:]]*/ /g' > "$o/$name.text"
  # Anywhere in a line, as a diagnostic can follow other output on it (WISDOM 85).
  ef=$(grep -cE '(Error|Fatal):[^:[:space:]]+:' "$o/$name.txt")
}

# clean <check> <name> <phrase>...: exit 0, no Error/Fatal, a verdict, and each phrase in the HTML's text.
clean() {
  local check="$1" name="$2" o="$WORK/out" lost="" phrase verdict
  shift 2
  for phrase in "$@"; do grep -qF -- "$phrase" "$o/$name.text" || lost="$lost \"$phrase\""; done
  verdict="$(grep -o 'Conversion complete:.*' "$o/$name.txt" | tail -n 1)"
  if [ "$rc" -ne 0 ] || [ "$ef" -ne 0 ] || [ -z "$verdict" ] || [ -n "$lost" ]; then
    fail "$check" "exit $rc, $ef Error/Fatal, verdict \"${verdict:-none}\"${lost:+, lost$lost}"
    grep -E '(Error|Fatal):[^:[:space:]]+:' "$o/$name.txt" | cut -c1-200 | sort | uniq -c | head -n 15 | sed 's/^/    /'
    return 1
  fi
  pass "$check" "0 Error/Fatal, the text kept (${verdict#Conversion complete: })"
}

# warnings <check> <name>: the conversion's Warning lines, reported.
warnings() {
  local n
  n=$(grep -cE 'Warning:[^:[:space:]]+:' "$WORK/out/$2.txt")
  if [ "$n" -eq 0 ]; then
    pass "$1" "no Warning"
  else
    # TODO(row R NOTEX-L3BACKEND): a FAIL once the no-TeX format stops warning
    # `missing_file:l3backend-pdftex` (PR-1); reported until then, so no release leg goes red first.
    note "$1" "$n Warning(s):$(grep -oE 'Warning:[^:[:space:]]+:[^[:space:]]*' "$WORK/out/$2.txt" | sort |
      uniq -c | tr -s ' ' | tr '\n' ';' | sed 's/;$//')"
  fi
}

# loaded_latex <name>: the `Loaded …` line of the latex dump: the first after `Info:latex_dump:embedded`, or the one
# naming a `latex.<year>.dump.txt`.
loaded_latex() {
  awk '/^Info:latex_dump:embedded/ { e = 1; next }
    /^Info:dump_reader:loaded Loaded/ && (e || /latex\.[0-9][0-9][0-9][0-9]\.dump\.txt/) { print; exit }' \
    "$WORK/out/$1.txt"
}

echo "artifact_check $MODE: $BIN"
"$WRAP" --version 2>&1 | head -n 1 | sed 's/^/  /'

if [ "$MODE" = notex ]; then
  convert hello "$HELLO"
  emb="$(grep -m1 '^Info:latex_dump:embedded using embedded TL[0-9]\{4\}' "$WORK/out/hello.txt")"
  year="$(printf '%s' "$emb" | sed -n 's/.*using embedded TL\([0-9]\{4\}\).*/\1/p')"
  loaded="$(loaded_latex hello)"
  backend="$(grep -m1 '^Info:kpathsea:backend' "$WORK/out/hello.txt" | cut -d' ' -f2-)"
  if [ -z "$year" ]; then
    fail format "no \"using embedded TL<year>\" line: ${loaded:-no latex dump loaded (the no-dump branch)}; kpathsea: ${backend:-?}"
  elif ! printf '%s' "$loaded" | grep -qE "Loaded [0-9]+ entries from <embedded TL$year> \(.*, 0 errors\)"; then
    fail format "TL$year embedded, but the load reads \"${loaded:-nothing}\""
  else
    entries="$(printf '%s' "$loaded" | sed 's/.*Loaded \([0-9]*\) entries.*/\1/')"
    pass format "embedded TL$year, $entries entries, 0 errors; kpathsea: ${backend:-?}"
  fi
  if [ -n "$year" ]; then
    dump="$(ls "$WORK/tmp"/latexml-oxide-dumps-*/latex."$year".dump.txt 2> /dev/null | head -n 1)"
    want="$(awk -v y="$year" '$1 == y || $1 == "any" { print $2; exit }' "$WORK/expected")"
    if [ -z "$dump" ]; then
      fail dump-sha256 "no decompressed latex.$year.dump.txt under the temp dir"
    else
      got="$(sha256 < "$dump")"
      if [ -z "$want" ]; then
        if [ -s "$WORK/expected" ]; then
          fail dump-sha256 "no expected sum for TL$year (got $got)"
        else
          note dump-sha256 "latex.$year.dump.txt $got (no expected sum given)"
        fi
      elif [ "$got" = "$want" ]; then
        pass dump-sha256 "latex.$year.dump.txt $got"
      else
        fail dump-sha256 "latex.$year.dump.txt is $got, expected $want"
      fi
    fi
  fi
  clean hello hello "Hello World!"
  warnings hello-warnings hello

  smoke="$(cd "$WORK/out" && NOTEX_SMOKE_TEX_HIDDEN=1 bash "$TOOLS/notex_smoke.sh" "$WRAP" 2>&1)"
  if [ $? -eq 0 ]; then
    pass notex-article "$(printf '%s\n' "$smoke" | tail -n 1)"
  else
    fail notex-article "tools/notex_smoke.sh failed"
  fi
  printf '%s\n' "$smoke" | grep -vE '^no-TeX smoke: clean' | sed 's/^/    /'

  convert expl3 "$FIXTURES/expl3_ltcmd_hooks.tex"
  clean expl3 expl3 "Hello, World and Moon." "Hello, Sun and Stars." "starred-one" "plain-two" \
    "expl-three-word; count=42." "hook-ran-at-begindocument"
  warnings expl3-warnings expl3
else
  convert hello "$HELLO"
  loaded="$(loaded_latex hello)"
  if grep -qE '^Warning:dump:build_failed|^\[format_dumps\].*loads from the engine' "$WORK/out/hello.txt"; then
    fail format-build "$(grep -m1 -E '^Warning:dump:build_failed|^\[format_dumps\].*loads from the engine' "$WORK/out/hello.txt" | cut -c1-240)"
  elif ! grep -q '^\[format_dumps\] no latex dump matches this TeX tree; building one into' "$WORK/out/hello.txt"; then
    # Row R DUMPTRIM: the embedded format serves only a machine whose tree cannot supply one.
    fail format-build "no latex format was built for this tree; the load reads \"${loaded:-nothing}\""
  elif ! printf '%s' "$loaded" | grep -qE 'format-cache-artifact-check.*latex\.[0-9]{4}\.dump\.txt \(.*, 0 errors\)'; then
    fail format-build "built, but the load reads \"${loaded:-nothing}\""
  else
    pass format-build "latex built into the fresh cache and loaded, 0 errors"
  fi
  # TODO(row R DUMPTRIM-PLAIN): a FAIL once the embedded plain dump no longer serves a tree whose plain.tex stamp it matches.
  plain="$(grep -m1 '^Info:dump_reader:loaded' "$WORK/out/hello.txt" | sed 's/^Info:dump_reader:loaded //')"
  if printf '%s' "$plain" | grep -q '<embedded TL'; then
    note format-plain "the embedded plain dump serves this tree: $plain"
  elif [ -z "$plain" ]; then
    note format-plain "no plain dump load was logged"
  else
    pass format-plain "$plain"
  fi
  clean hello hello "Hello World!"
  warnings hello-warnings hello

  convert raw "$FIXTURES/raw_package.tex"
  loaded="$(loaded_latex raw)"
  if grep -q '^\[format_dumps\] no latex dump matches' "$WORK/out/raw.txt"; then
    fail format-reuse "the second conversion built the format again"
  elif grep -q 'Mismatched LaTeX support files' "$WORK/out/raw.txt"; then
    fail format-reuse "expl3.sty refuses the format: $(grep -m1 'Mismatched LaTeX support files' "$WORK/out/raw.txt" | cut -c1-200)"
  elif ! printf '%s' "$loaded" | grep -q 'format-cache-artifact-check'; then
    fail format-reuse "the load reads \"${loaded:-nothing}\", not the cache"
  else
    pass format-reuse "the cached latex dump is loaded; raw xparse.sty/expl3.sty accept it"
  fi
  clean raw-package raw "left-a-right" "left-b-c" "raw-package-text"

  convert expl3 "$FIXTURES/expl3_ltcmd_hooks.tex"
  clean expl3 expl3 "Hello, World and Moon." "Hello, Sun and Stars." "starred-one" "plain-two" \
    "expl-three-word; count=42." "hook-ran-at-begindocument"
  warnings expl3-warnings expl3
fi

smoke="$(cd "$ROOT" && bash "$TOOLS/release_smoke.sh" "$WRAP" "$HELLO" 2>&1)"
if [ $? -eq 0 ]; then
  pass release-smoke "tools/release_smoke.sh: $(printf '%s\n' "$smoke" | grep -c '^OK:') OK lines"
else
  fail release-smoke "tools/release_smoke.sh failed"
  printf '%s\n' "$smoke" | sed 's/^/    /'
fi

echo "artifact_check $MODE: $npass passed, $nfail failed, $nnote reported"
[ "$nfail" -eq 0 ]
