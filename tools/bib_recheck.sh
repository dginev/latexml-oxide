#!/usr/bin/env bash
# bib_recheck.sh — reconvert arXiv papers and check they end up with a
# bibliography (`class="ltx_bibitem"` entries in the HTML).
#
# The driver for the bibliography-absence campaign
# (docs/parity/BIB_ABSENCE_AUDIT_2026-07-29.md, SYNC_STATUS row R3): feed it a
# batch of ids, it reconverts each with the CURRENT binary under the same
# profile the arXiv fleet uses (`cortex_worker --standalone`, ar5iv profile),
# then reports how many bibitems came out versus how many the source implies.
#
# Usage:
#   tools/bib_recheck.sh 2605.01115 2605.00125           # ids on the command line
#   tools/bib_recheck.sh --list batch1.txt               # one id per line
#   tools/bib_recheck.sh -j 20 -o /tmp/run1 --list b.txt
#
# Options:
#   -j N        parallel conversions (default 20 — one "batch")
#   -o DIR      keep outputs under DIR (default: a fresh mktemp dir, printed)
#   -t SECS     per-paper timeout (default 120)
#   --list F    read ids from F (blank lines and #-comments ignored)
#   --baseline SUFFIX
#               also read the pre-existing fleet result zip
#               oxidized_tex_to_html<SUFFIX>.zip for a before/after column
#               (e.g. --baseline .sandbox-13 for 2605, .sandbox-14 for 2606)
#
# Env:
#   CORPUS      corpus root (default /data/arxiv)
#   WORKER_BIN  cortex_worker path (default ./target/release/cortex_worker)
#
# Output: TSV on stdout and in $OUTDIR/results.tsv, one row per paper:
#   id  now  want  cited  miss  base  status  nerr  verdict
#     now     ltx_bibitem count after reconversion
#     want    entries the SOURCE implies (\bibitem in .bbl, else distinct \cite
#             keys, else @entries in .bib) — a sanity target, not an exact oracle
#     cited   MakeBibliography's own "N cited" tally (the authoritative count)
#     miss    unresolved citations ("Missing Entry for citation") — what a
#             reader sees as "[?]"; the honest completeness signal
#     base    ltx_bibitem count in the fleet baseline zip (- if not requested)
#     status  cortex Status:conversion:N (3 fatal, 2 error, 1 warn, 0 ok)
#     nerr    count of Error:/Fatal: lines
#     verdict OK (now>0, and now matches "cited" when known) | THIN (short of
#             what was cited) | EMPTY (now==0) | NOHTML | NOSRC | TIMEOUT
# Per-paper artifacts stay in $OUTDIR/<id>/ (out.zip, log, html) for reading.
set -uo pipefail

CORPUS="${CORPUS:-/data/arxiv}"
WORKER_BIN="${WORKER_BIN:-./target/release/cortex_worker}"
JOBS=20
TIMEOUT=120
OUTDIR=
BASELINE=
IDS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    -j) JOBS="$2"; shift 2 ;;
    -o) OUTDIR="$2"; shift 2 ;;
    -t) TIMEOUT="$2"; shift 2 ;;
    --list) mapfile -t -O "${#IDS[@]}" IDS < <(grep -vE '^\s*(#|$)' "$2" | awk '{print $1}'); shift 2 ;;
    --baseline) BASELINE="$2"; shift 2 ;;
    -h|--help) sed -n '2,40p' "$0"; exit 0 ;;
    *) IDS+=("$1"); shift ;;
  esac
done

if [[ ${#IDS[@]} -eq 0 ]]; then
  echo "ERROR: no ids given (use --list FILE or pass ids)" >&2
  exit 2
fi
if [[ ! -x "$WORKER_BIN" ]]; then
  echo "ERROR: $WORKER_BIN not found — build it first:" >&2
  echo "  cargo build --release --bin cortex_worker --no-default-features --features cortex" >&2
  exit 1
fi
WORKER_BIN=$(readlink -f "$WORKER_BIN")
OUTDIR="${OUTDIR:-$(mktemp -d "${TMPDIR:-/tmp}/bibrecheck_XXXXXX")}"
mkdir -p "$OUTDIR"
echo "[bib_recheck] outdir: $OUTDIR  jobs: $JOBS  papers: ${#IDS[@]}" >&2

recheck_one() {
  local id="$1" corpus="$2" worker="$3" outdir="$4" timeout_s="$5" baseline="$6"
  local yymm digits src dir out log html
  if [[ "$id" =~ ^([0-9]{4})\.[0-9]+$ ]]; then yymm="${BASH_REMATCH[1]}"
  else digits="${id##*[!0-9]}"; yymm="${digits:0:4}"; fi
  src="$corpus/$yymm/$id/$id.zip"
  dir="$outdir/$id"; mkdir -p "$dir"
  out="$dir/out.zip"; log="$dir/cortex.log"

  if [[ ! -f "$src" ]]; then
    printf '%s\t-\t-\t-\t-\t-\t-\t-\tNOSRC\n' "$id"; return
  fi

  # --- what the SOURCE implies (want) ---
  local want
  # `\bibitem` FOLLOWED BY its argument — REVTeX/apsrev `.bbl` files open with
  # `\providecommand\bibitemStop`/`\bibitemNoStop` boilerplate that a bare
  # `\bibitem` grep counts as entries (2605.21570: 36 counted, 34 real, and the
  # file's own `\begin{thebibliography}{34}` agrees).
  want=$(unzip -p "$src" '*.bbl' 2>/dev/null | grep -acE '\\bibitem[ \t]*[[{]')
  if [[ "$want" == 0 ]]; then
    want=$(unzip -p "$src" '*.tex' '*.ltx' 2>/dev/null | grep -acE '\\bibitem[ \t]*[[{]')
  fi
  if [[ "$want" == 0 ]]; then
    want=$(unzip -p "$src" '*.tex' '*.ltx' 2>/dev/null |
      grep -aoE '\\(cite|citep|citet|citealp|citeyear|nocite)[a-zA-Z]*\*?(\[[^]]*\])*\{[^}]*\}' |
      sed 's/.*{//; s/}//' | tr ',' '\n' | sed 's/^ *//; s/ *$//' | grep -v '^$' | sort -u | grep -c .)
  fi
  if [[ "$want" == 0 ]]; then
    want=$(unzip -p "$src" '*.bib' 2>/dev/null | grep -acE '^\s*@[a-zA-Z]+\s*[{(]')
  fi

  # --- baseline (previous fleet result) ---
  local base='-'
  if [[ -n "$baseline" ]]; then
    local bz="$corpus/$yymm/$id/oxidized_tex_to_html${baseline}.zip"
    [[ -f "$bz" ]] || bz="$corpus/$yymm/$id/oxidized_tex_to_html.zip"
    if [[ -f "$bz" ]]; then
      base=$(unzip -p "$bz" '*.html' 2>/dev/null | grep -ac 'ltx_bibitem')
    fi
  fi

  # --- reconvert ---
  local rc=0
  timeout $((timeout_s + 60)) "$worker" --standalone --input "$src" --output "$out" \
    --timeout "$timeout_s" >"$dir/stdout.txt" 2>"$dir/stderr.txt" || rc=$?
  if [[ $rc -ne 0 && ! -s "$out" ]]; then
    printf '%s\t-\t%s\t-\t-\t%s\t-\t-\tTIMEOUT\n' "$id" "$want" "$base"; return
  fi

  unzip -p "$out" cortex.log 2>/dev/null | sed 's/\x1b\[[0-9;]*m//g' >"$log"
  html=$(unzip -Z1 "$out" '*.html' 2>/dev/null | head -1)
  [[ -n "$html" ]] && unzip -p "$out" "$html" >"$dir/out.html" 2>/dev/null

  local now status nerr first miss cited
  # grep -c prints a count even when it exits 1 on no-match — do NOT add a
  # `|| echo 0` fallback here, it appends a second line and corrupts the TSV.
  now=$(grep -ac 'ltx_bibitem' "$dir/out.html" 2>/dev/null); now=${now:-0}
  status=$(unzip -p "$out" status 2>/dev/null | grep -oE '[0-9]+$' | head -1)
  [[ -n "$status" ]] || status='-'
  nerr=$(grep -acE '^(Error|Fatal):' "$log" 2>/dev/null); nerr=${nerr:-0}
  # Unresolved citations are what a reader sees as "[?]", and they are the
  # honest "is this bibliography complete" signal — `want` is only a crude
  # source-side estimate (it counts cite keys that may not exist in the .bib
  # at all: 2605.00125 estimates 57 where the log says "54 bibentries, 54
  # cited" and all 54 render).
  miss=$(grep -ac 'Missing Entry for citation' "$log" 2>/dev/null); miss=${miss:-0}
  cited=$(grep -aoE 'MakeBibliography: [0-9]+ bibentries, [0-9]+ cited' "$log" 2>/dev/null |
    tail -1 | grep -oE '[0-9]+ cited' | grep -oE '[0-9]+')
  [[ -n "$cited" ]] || cited='-'
  first=$(grep -m1 -aE '^(Error|Fatal):' "$log" 2>/dev/null | cut -c1-140)
  [[ -n "$first" ]] || first='-'

  local verdict
  if [[ ! -s "$dir/out.html" ]]; then verdict=NOHTML
  elif [[ "$now" -eq 0 ]]; then verdict=EMPTY
  elif [[ "$cited" != '-' && "$cited" -gt 0 && "$now" -lt "$cited" ]]; then verdict=THIN
  elif [[ "$cited" == '-' && "$want" -gt 0 ]] && (( now * 10 < want * 9 )); then verdict=THIN
  else verdict=OK
  fi
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$id" "$now" "$want" "$cited" "$miss" "$base" "$status" "$nerr" "$verdict"
}
export -f recheck_one

printf '%s\n' "${IDS[@]}" |
  xargs -d '\n' -P "$JOBS" -I{} bash -c \
    'recheck_one "$@"' _ {} "$CORPUS" "$WORKER_BIN" "$OUTDIR" "$TIMEOUT" "$BASELINE" |
  tee "$OUTDIR/results.tsv"

echo >&2
echo "[bib_recheck] verdicts:" >&2
cut -f9 "$OUTDIR/results.tsv" | sort | uniq -c | sort -rn >&2
echo "[bib_recheck] results: $OUTDIR/results.tsv" >&2
