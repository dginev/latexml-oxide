#!/usr/bin/env bash
# ci_tex_fingerprint.sh — what the CI engine-dump cache key must name about the TeX tree, as step outputs.
#
# A dump records the format sources it was built from: `# source` header lines with the name, size and
# CRC-32 of latex.ltx, expl3-code.tex and plain.tex, looked up as latexml_engine/src/dump_paths.rs
# `format_sources` does. A process whose tree holds different files does not use that dump. So the key
# names those files' contents, not only the TeX Live year: two trees that report the same year can differ
# in them. Examples are Homebrew's texlive vs TeX-Live/setup-texlive-action, and an apt TeX Live update
# within a year.
#
# Writes to $GITHUB_OUTPUT (stdout outside Actions):
#   year=<the TeX Live year in pdflatex's banner>   (informative; keeps keys readable)
#   sources=<cksum over the three files' CRC and size>
set -euo pipefail

out="${GITHUB_OUTPUT:-/dev/stdout}"
banner=$(pdflatex --version 2>/dev/null) || true
year=$(sed -n 's:.*TeX Live \([0-9]\{4\}\).*:\1:p' <<<"$banner")
year=${year%%$'\n'*}

sums=""
for query in base/latex.ltx l3kernel/expl3-code.tex plain/base/plain.tex; do
  path=$(kpsewhich "$query") || true
  path=${path%$'\r'} # kpsewhich.exe ends its line with CRLF under Git Bash
  if [[ -z $path || ! -r $path ]]; then
    echo "::error::kpsewhich resolves no readable $query: no TeX tree to key the engine dumps on" >&2
    exit 1
  fi
  sums+="$query $(cksum <"$path")"$'\n'
done
sources=$(printf '%s' "$sums" | cksum | cut -d' ' -f1)

printf 'year=%s\nsources=%s\n' "$year" "$sources" >>"$out"
printf '%s' "$sums" | sed 's/^/  /'
