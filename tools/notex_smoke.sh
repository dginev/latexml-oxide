#!/usr/bin/env bash
# notex_smoke.sh — convert a small article with a latexml_oxide binary on a machine without TeX Live, and fail
# unless it converts cleanly (OXIDIZED_DESIGN_DIVERGENCES #1095).
#
# With no TeX tree there is no format dump either (tools/make_formats.sh needs TeX Live), so the conversion runs on
# the no-dump branch of LoadFormat('latex') with the embedded bindings. It must: exit 0, raise no Error or Fatal,
# name the unavailable kpathsea backend in its log (an Info: Perl warns of no missing resolver), and keep the
# article's words, math, list, table and bibliography.
#
# Usage: tools/notex_smoke.sh <latexml_oxide binary>
set -uo pipefail

BIN="${1:?usage: $0 <latexml_oxide binary>}"
[[ -x $BIN ]] || { echo "::error::not an executable: $BIN"; exit 2; }
# `grep -q` would quit on the match and SIGPIPE `ldconfig`, which `pipefail` turns into "no match".
if command -v kpsewhich > /dev/null || ldconfig -p 2> /dev/null | grep libkpathsea > /dev/null; then
  echo "::error::TeX Live is installed here (kpsewhich or libkpathsea found): this smoke needs a machine without it"
  exit 2
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
cat > "$WORK/article.tex" << 'EOF'
\DocumentMetadata{}
\documentclass{article}
\usepackage{amsmath,amssymb,graphicx,array,hyperref}
\title{A Small Paper}\author{A. Author}
\begin{document}
\maketitle
\begin{abstract}An abstract.\end{abstract}
\section{Intro}\label{s:intro}
Text with $a+b^2=\frac{x}{y}$ and \emph{emphasis}, see Section~\ref{s:intro} and~\cite{k}.
\begin{equation}\label{e1} \int_0^1 f(x)\,dx = \sum_{i=1}^n a_i \end{equation}
\begin{align} x &= 1 \\ y &= 2 \end{align}
\begin{itemize}\item first-item \item second-item\end{itemize}
after-the-list
\begin{table}[h]\centering\begin{tabular}{|>{\bfseries}l|c|}\hline cell-a & cell-b \\ \hline\end{tabular}\caption{T}\end{table}
\stepcounter{equation}\ensuremath{y^2}\footnote{A note.}
\begin{thebibliography}{9}\bibitem{k} K. Author, \emph{Title}, 2020.\end{thebibliography}
\end{document}
EOF

status=0
timeout 600 "$BIN" --format=html5 --timeout=600 --dest="$WORK/article.html" "$WORK/article.tex" > "$WORK/out.txt" 2>&1
code=$?
LOG="$(sed 's/\x1b\[[0-9;]*m//g' "$WORK/out.txt")"
grep -E '^(Error|Fatal|Warning):' <<< "$LOG" | cut -c1-160 | sort | uniq -c
fail() { echo "::error::$1"; status=1; }
((code == 0)) || fail "latexml_oxide exited $code"
errors=$(grep -c '^Error:' <<< "$LOG")
fatals=$(grep -c '^Fatal:' <<< "$LOG")
((errors == 0 && fatals == 0)) || fail "$errors Error and $fatals Fatal lines without TeX Live"
# The converter's own verdict, too: an unreadable or missing one fails (never read as clean).
verdict="$(grep -o 'Conversion complete:.*' <<< "$LOG" | tail -1)"
[[ -n $verdict ]] || fail "no \"Conversion complete:\" verdict"
grep -qiE 'error|fatal|undefined' <<< "$verdict" && fail "verdict: $verdict"
grep -q '^Info:kpathsea:backend unavailable' <<< "$LOG" || fail "the log does not name the unavailable kpathsea backend"
HTML="$(cat "$WORK/article.html" 2> /dev/null)"
for word in 'A Small Paper' 'An abstract.' 'emphasis' 'first-item' 'second-item' 'after-the-list' 'cell-a' \
  'A note.' 'K. Author'; do
  grep -qF "$word" <<< "$HTML" || fail "the HTML lost \"$word\""
done
maths=$(grep -o '<math' <<< "$HTML" | wc -l)
((maths >= 6)) || fail "$maths <math> elements, expected at least 6"
((status == 0)) && echo "no-TeX smoke: clean ($maths <math>)"
exit $status
