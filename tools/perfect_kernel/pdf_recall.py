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


COMBINING = "̀-ͯ᪰-᫿᷀-᷿⃐-⃿︠-︯"
DOTLESS = re.compile(f"[ıȷ](?=[{COMBINING}])")
COMPOUND = re.compile(r"([^\W\d_]+)[-‐‑](?=([^\W\d_]+))")


def normalize(text, min_len, rejoin=False, compounds=False):
    text = unicodedata.normalize("NFKC", text)
    # A dotless ı/ȷ before a combining accent is the accented i/j (OT1 `\^{\i}` reaches pdftotext as
    # ı + U+0302, which NFKC cannot compose: "reconnaı tre").
    text = unicodedata.normalize("NFC", DOTLESS.sub(lambda m: "i" if m.group() == "ı" else "j", text))
    # pdftotext's line-break hyphen ("in-\nput") is rejoined on the PDF side only: in the XML text a
    # hyphen before a newline is the source's own, which TeX reads as a space (dinbrief `ober-⏎und`).
    if rejoin:
        text = re.sub(r"(?<=\w)-\n(?=\w)", "", text)
    words = {w for w in LETTERS.findall(text.casefold()) if len(w) >= min_len}
    # On the XML side a hyphenated compound also counts joined: pdftotext drops a real hyphen at a line
    # end ("profit-⏎making" → "profitmaking", geradwp).
    if compounds:
        words |= {(a + b).casefold() for a, b in COMPOUND.findall(text) if len(a + b) >= min_len}
    return words


TAG = re.compile(r'^<(/?)([A-Za-z][\w:.-]*)((?:[^>"]|"[^"]*")*?)(/?)>$', re.S)
VOID = re.compile(r"^(?:\w+:)?(?:area|base|br|col|embed|hr|img|input|link|meta|param|source|track|wbr)$", re.I)
# Not rendered, as s3_audit.sh drops it: an inline `display:none`/`visibility:hidden`, and class `ltx_nodisplay`
# (LaTeXML.css `display:none` — acmart's `\Description`, the bibliography only a `\fullcite` reads).
HIDDEN = re.compile(r'visibility\s*:\s*hidden|display\s*:\s*none|class="[^"]*\bltx_nodisplay\b')


def xml_text(path):
    raw = open(path, encoding="utf-8", errors="replace").read()
    raw = re.sub(r"<(m:)?annotation\b.*?</(m:)?annotation>", " ", raw, flags=re.S)
    raw = re.sub(r"<!--.*?-->", " ", raw, flags=re.S)
    out, stack, hidden = [], [], 0
    for piece in re.split(r'(<(?:[^>"]|"[^"]*")*>)', raw):
        tag = TAG.match(piece)
        if tag:
            close, name, attrs, empty = tag.groups()
            if close:
                hidden -= stack.pop() if stack else 0
            elif not empty and not VOID.match(name):
                stack.append(1 if HIDDEN.search(attrs) else 0)
                hidden += stack[-1]
            out.append(" ")
        elif piece.startswith("<"):
            out.append(" ")
        elif not hidden:
            out.append(piece)
    return html.unescape("".join(out))


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
    xml_words = normalize(xml_text(files[0]), min_len, compounds=True)
    pdf_words = normalize(pdf_text(files[1]), min_len, rejoin=True)
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
