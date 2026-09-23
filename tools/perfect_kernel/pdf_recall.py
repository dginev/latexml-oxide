#!/usr/bin/env python3
r"""PDF-to-XML content recall: does every word that reaches the PDF reach the XML?

    pdf_recall.py DOC.xml DOC.pdf [--min-len N] [--show N] [--strict]

The PDF comes from the document's intended engine (pdflatex, xelatex or lualatex).
Both sides are normalized exactly as `s3_audit.sh` does for the corpus manuals:
NFKC (ligatures, compatibility forms), the pdftotext line-break hyphen re-joined
("in-\nput" -> "input"), lowercase runs of letters in any script. Digits are not
words, so page numbers and footnote marks drop out. The XML is tag-stripped with a
space per tag, so text in adjacent elements is never glued; MathML/`tex` annotation
bodies are removed so math counts as rendered text only.

Prints `recall=P% (found/total distinct pdf words; M missing)` and a sample of the
missing words. Known non-loss misses, inspect before calling it loss: page furniture
(running heads); words LaTeX GENERATES that the core XML carries as structure and
the post stage renders ("Abstract", "Contents", "References"); the default `\today`
date of `\maketitle` (LaTeXML emits no date unless `\date` is given — Perl too);
hyphenation the re-join cannot see. `--strict` exits 1 on any missing
word (for minimal repros, where there is no furniture). Default `--min-len 2`
(the corpus audit uses 4).
"""
import html
import re
import subprocess
import sys
import unicodedata

LETTERS = re.compile(r"[^\W\d_]+", re.UNICODE)


def normalize(text, min_len):
    text = unicodedata.normalize("NFKC", text)
    text = re.sub(r"(?<=\w)-\n(?=\w)", "", text)
    return {w for w in LETTERS.findall(text.casefold()) if len(w) >= min_len}


def xml_text(path):
    raw = open(path, encoding="utf-8", errors="replace").read()
    raw = re.sub(r"<(m:)?annotation\b.*?</(m:)?annotation>", " ", raw, flags=re.S)
    raw = re.sub(r"<[^>]*>", " ", raw)
    return html.unescape(raw)


def pdf_text(path):
    out = subprocess.run(["pdftotext", "-q", path, "-"], capture_output=True, text=True)
    if out.returncode != 0:
        sys.exit(f"pdftotext failed on {path}")
    return out.stdout


def main():
    argv = sys.argv[1:]
    opt = lambda name, default: int(argv[argv.index(name) + 1]) if name in argv else default
    min_len, show, strict = opt("--min-len", 2), opt("--show", 15), "--strict" in argv
    files = [a for i, a in enumerate(argv)
             if not a.startswith("--") and (i == 0 or argv[i - 1] not in ("--min-len", "--show"))]
    if len(files) != 2:
        sys.exit(__doc__)
    xml_words = normalize(xml_text(files[0]), min_len)
    pdf_words = normalize(pdf_text(files[1]), min_len)
    missing = sorted(pdf_words - xml_words)
    total = len(pdf_words)
    pct = 100.0 * (total - len(missing)) / total if total else 100.0
    print(f"recall={pct:.1f}% ({total - len(missing)}/{total} distinct pdf words; "
          f"{len(missing)} missing)")
    if missing:
        print("  missing:", " ".join(missing[:show]))
    sys.exit(1 if strict and missing else 0)


if __name__ == "__main__":
    main()
