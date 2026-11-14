#!/usr/bin/env bash
# next_entry_numbers_test.sh — offline self-test of next_entry_numbers.sh: a scratch git repo whose bare
# "origin" holds refs/pull/{1,2}/head, and a fake `gh` on PATH printing a canned PR list. Run by tools/lint.sh.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
tool="$here/next_entry_numbers.sh"
tmp=$(mktemp -d)
trap 'rm -rf -- "$tmp"' EXIT
fails=0
check() { # <name> <haystack> <needle>: the needle must be a line (or line fragment) of the haystack
  if printf '%s\n' "$2" | grep -qF -- "$3"; then echo "ok   $1"; else
    echo "FAIL $1: expected '$3' in:"; printf '%s\n' "$2" | sed 's/^/     /'; fails=$((fails + 1)); fi
}
check_not() {
  if printf '%s\n' "$2" | grep -qF -- "$3"; then
    echo "FAIL $1: unexpected '$3' in:"; printf '%s\n' "$2" | sed 's/^/     /'; fails=$((fails + 1)); else echo "ok   $1"; fi
}

export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t GIT_CONFIG_GLOBAL=/dev/null
git init -q --bare "$tmp/origin.git"
git init -q -b main "$tmp/work"
w="$tmp/work"
mkdir -p "$w/docs/parity" "$w/docs/math" "$w/tools/perfect_kernel"
cp "$tool" "$w/tools/perfect_kernel/"
printf '# OD index\n> Next free number: **#1035** (allocate at integration).\n' > "$w/docs/parity/OXIDIZED_DESIGN.md"
printf '### 1035. Old entry\n\n### 1036. Main entry\n' > "$w/docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md"
printf '### 7. Math entry\n' > "$w/docs/math/OXIDIZED_DESIGN_MATH.md"
printf '## 1244. Main KPE\n' > "$w/docs/parity/KNOWN_PERL_ERRORS.md"
git -C "$w" add -A && git -C "$w" commit -qm main
git -C "$w" remote add origin "$tmp/origin.git"
git -C "$w" push -q origin HEAD:refs/heads/main
pr_branch() { # <pr> <OD headings...> -- <KPE headings...>
  local pr=$1; shift
  git -C "$w" checkout -q -b "pr$pr" main
  while [ "$1" != -- ]; do printf '\n%s\n' "$1" >> "$w/docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md"; shift; done
  shift
  for h in "$@"; do printf '\n%s\n' "$h" >> "$w/docs/parity/KNOWN_PERL_ERRORS.md"; done
  git -C "$w" commit -qam "pr$pr"
  git -C "$w" push -q origin "HEAD:refs/pull/$pr/head"
  git -C "$w" checkout -q main
}
pr_branch 1 '### 1037. PR one entry' -- '## 1245. PR one KPE'
pr_branch 2 '### 1037. PR two entry' '### 1038. PR two second' --

mkdir "$tmp/bin"
cat > "$tmp/bin/gh" <<'EOF'
#!/usr/bin/env bash
echo called >> "$FAKE_GH_LOG"
[ "${FAKE_GH_FAIL:-0}" = 1 ] && { echo "gh: not logged in" >&2; exit 1; }
echo '[{"headRefName":"pr1","isCrossRepository":false,"number":1},{"headRefName":"fork","isCrossRepository":true,"number":2}]'
EOF
chmod +x "$tmp/bin/gh"
export PATH="$tmp/bin:$PATH" FAKE_GH_LOG="$tmp/gh.log"
run() { (cd "$w" && bash tools/perfect_kernel/next_entry_numbers.sh "$@" 2>&1); }

out=$(run)
check "next free OD over main and both heads" "$out" "OD  next free: #1039"
check "next free KPE over main and both heads" "$out" "KPE next free: #1246"
check "PR #1 holds OD #1037" "$out" "OD #1037 held by #1"
check "PR #2 (a fork) holds OD #1037-#1038" "$out" "OD #1037-#1038 held by #2"
check "PR #1 holds KPE #1245" "$out" "KPE #1245 held by #1"
check "two heads define OD #1037: CLASH" "$out" "CLASH: OD #1037 defined differently by #1 #2"
check_not "main's own entries are not clashes" "$out" "CLASH: OD #1036"
check_not "the math file's by-value collisions are not clashes" "$out" "CLASH: OD #7"
check "the OD index is judged against the combined max" "$out" "states #1035: stale (should be #1039)"
check_not "no scratch refs are left" "$(git -C "$w" for-each-ref)" "next-entry-numbers"

# The working tree's own new entry against an open head's.
cp "$w/docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md" "$tmp/od.bak"
printf '\n### 1037. Local writer entry\n' >> "$w/docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md"
out=$(run)
check "working tree and a head define OD #1037: CLASH" "$out" "CLASH: OD #1037 defined differently by #1 #2 working-tree"
cp "$tmp/od.bak" "$w/docs/parity/OXIDIZED_DESIGN_DIVERGENCES.md"

: > "$FAKE_GH_LOG"
out=$(run --local-only)
check "--local-only: today's numbers" "$out" "OD  next free: #1037"
check "--local-only never calls gh" "$(wc -l < "$FAKE_GH_LOG")" "0"

out=$(FAKE_GH_FAIL=1 run)
check "gh failing: loud on stderr" "$out" "WARNING: open PR heads not scanned"
check "gh failing: falls back to the working tree" "$out" "OD  next free: #1037"

printf 'adds OD #NEW1 and KPE #NEW1\n' > "$tmp/report.md"
out=$(run --renumber "$tmp/report.md")
check "--renumber uses the combined max (OD)" "$out" "OD #NEW1 -> OD #1039"
check "--renumber uses the combined max (KPE)" "$out" "KPE #NEW1 -> KPE #1246"

[ "$fails" -eq 0 ] || { echo "$fails check(s) failed"; exit 1; }
echo "all checks passed"
