#!/usr/bin/env python3
"""Axis 2b — semantic-markup coverage: LaTeX constructs in the source vs the
LaTeXML elements they should become in the core XML.

For each manual, count construct FAMILIES in the TeX source (comments,
verbatim-like environments and `\\verb` stripped, so the code examples a manual
quotes are not counted) and the matching elements in the XML. A family whose XML
count falls short of the source count is a semantic gap: the construct was
emitted as a generic/presentational fallback, or dropped. Counts are a proxy —
a macro can expand to several elements, a user macro can hide a construct — so
read the ratio as a ranking signal, not a verdict.

Usage:
  semantic_coverage.py <corpus.tsv> <sweep_dir> [--out report.tsv] [--top N]
    corpus.tsv: bundle<TAB>tex path<TAB>pdf path<TAB>...
    sweep_dir:  <sweep_dir>/<bundle>/<name>/<name>.xml
  semantic_coverage.py --doc <file.tex> <file.xml>

Output: per-doc TSV (bundle, name, then src/xml per family) and a corpus
summary: per family, docs with the construct, docs short, and the summed
coverage. Only completed conversions count (a Fatal/timeout verdict is the
no-XML set, not a markup gap).

Known false deficits (read before acting on a ranking): `\\item` inside a
hand-typeset `theindex` is `\\@idxitem` (xdoc/xdocdemo, 557 "items");
exam.cls's `\\part[5]` is a question part, not sectioning (exam/examdoc);
`$$`-redefining packages (nath) turn displays into inline Math on purpose.
First run (s114, 2,274 completed docs, 2026-09-23): sections/lists/floats
93-98 %, equations 93 %, `\\part` 59.5 % — KOMA-Script's `\\part` became a bold
paragraph (scrartcl/scrbook/cnltx-doc).
"""
import argparse
import csv
import os
import re
import sys
from collections import defaultdict

# Environments whose bodies are code, not markup (their `\section` etc. are text).
VERBATIM_ENVS = (
    r"verbatim\*?|Verbatim\*?|BVerbatim|LVerbatim|lstlisting|minted|alltt|"
    r"comment|filecontents\*?|macrocode\*?|tcblisting|tcboutputlisting|"
    r"example|LTXexample|sourcecode|code|lstinputlisting|codeexample|"
    r"verbatimwrite|scontents|luacode\*?|pycode|sagesilent|sageblock|"
    r"dispExample\*?|docspec|latexcode|texexample|demo|showexpl|shortexample"
)
VERB_RE = re.compile(r"\\(?:verb|lstinline|mintinline\{[^}]*\}|Verb|cs|cmd|meta|marg|oarg|texttt)\*?"
                     r"(?:([^\sa-zA-Z{])(?:.*?)\1|\{(?:[^{}]|\{[^{}]*\})*\})", re.S)

# family: (source regex, XML element regex)
FAMILIES = {
    "part": (r"\\part\*?\s*[\[{]", r"<part[\s>]"),
    "chapter": (r"\\chapter\*?\s*[\[{]", r"<chapter[\s>]"),
    "section": (r"\\section\*?\s*[\[{]", r"<section[\s>]"),
    "subsection": (r"\\subsection\*?\s*[\[{]", r"<subsection[\s>]"),
    "subsubsection": (r"\\subsubsection\*?\s*[\[{]", r"<subsubsection[\s>]"),
    "paragraph": (r"\\(?:sub)?paragraph\*?\s*[\[{]", r"<(?:sub)?paragraph[\s>]"),
    "figure": (r"\\begin\{(?:figure|wrapfigure|SCfigure)\*?\}", r"<figure[\s>]"),
    "table": (r"\\begin\{(?:table|wraptable|SCtable)\*?\}", r"<table[\s>]"),
    "itemize": (r"\\begin\{itemize\}", r"<itemize[\s>]"),
    "enumerate": (r"\\begin\{enumerate\}", r"<enumerate[\s>]"),
    "description": (r"\\begin\{description\}", r"<description[\s>]"),
    "item": (r"\\item\b", r"<item[\s>]"),
    "tabular": (r"\\begin\{(?:tabular[x*]?|tabulary|longtable\*?|tabu|array)\}", r"<tabular[\s>]"),
    # `$$…$$` is counted per PAIR in count_source.
    "equation": (r"\\begin\{(?:equation|displaymath)\*?\}|(?<!\\)\\\[",
                 r"<equation[\s>]|<equationgroup[\s>]"),
    "eqgroup": (r"\\begin\{(?:align|gather|multline|flalign|alignat|eqnarray)\*?\}",
                r"<equationgroup[\s>]|<equation[\s>]"),
    "footnote": (r"\\footnote\b", r"<note[^>]*role=\"footnote\""),
    "cite": (r"\\(?:cite|citep|citet|parencite|textcite|autocite|footcite)\w*\*?\s*[\[{]",
             r"<cite[\s>]"),
    "ref": (r"\\(?:ref|eqref|autoref|cref|Cref|pageref|nameref|vref)\*?\s*\{", r"<ref[\s>]"),
    "caption": (r"\\caption\*?\s*[\[{]", r"<caption[\s>]"),
    "verbatim": (r"\\begin\{(?:verbatim|Verbatim|lstlisting|minted)\*?\}",
                 r"<verbatim[\s>]|<listing[\s>]"),
    "graphics": (r"\\includegraphics\*?\s*[\[{]", r"<graphics[\s>]"),
}


def strip_source(tex: str) -> str:
    """Drop comments, verbatim-like bodies and inline verbatim/code macros."""
    # Comments: an unescaped % to end of line (keep `\%`).
    tex = re.sub(r"(?<!\\)%.*", "", tex)
    tex = re.sub(r"\\begin\{(" + VERBATIM_ENVS + r")\}.*?\\end\{\1\}", " ", tex, flags=re.S)
    tex = VERB_RE.sub(" ", tex)
    # Anything before \begin{document} is the preamble (definitions, not markup).
    m = re.search(r"\\begin\{document\}", tex)
    return tex[m.end():] if m else tex


def count_source(tex: str, theorem_envs) -> dict:
    counts = {fam: len(re.findall(src, tex)) for fam, (src, _) in FAMILIES.items()}
    counts["equation"] += len(re.findall(r"(?<!\\)\$\$", tex)) // 2
    if theorem_envs:
        pat = r"\\begin\{(?:" + "|".join(map(re.escape, theorem_envs)) + r")\}"
        counts["theorem"] = len(re.findall(pat, tex))
    else:
        counts["theorem"] = 0
    return counts


def count_xml(xml: str) -> dict:
    counts = {fam: len(re.findall(x, xml)) for fam, (_, x) in FAMILIES.items()}
    counts["theorem"] = len(re.findall(r"<theorem[\s>]|<proof[\s>]", xml))
    return counts


def theorem_envs(full_tex: str):
    envs = set(re.findall(r"\\newtheorem\*?\{([^}]+)\}", full_tex))
    envs |= set(re.findall(r"\\declaretheorem(?:\[[^]]*\])?\{([^}]+)\}", full_tex))
    envs |= {"proof"} if "\\begin{proof}" in full_tex else set()
    return sorted(envs)


def read(path):
    with open(path, "rb") as f:
        return f.read().decode("utf-8", "replace")


def analyse(tex_path, xml_path):
    full = read(tex_path)
    src = count_source(strip_source(full), theorem_envs(full))
    xml = count_xml(read(xml_path)) if os.path.exists(xml_path) else None
    return src, xml


FAMS = list(FAMILIES) + ["theorem"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("corpus", nargs="?")
    ap.add_argument("sweep", nargs="?")
    ap.add_argument("--doc", nargs=2, metavar=("TEX", "XML"))
    ap.add_argument("--out")
    ap.add_argument("--top", type=int, default=25)
    a = ap.parse_args()
    if a.doc:
        src, xml = analyse(*a.doc)
        for fam in FAMS:
            if src[fam] or (xml and xml[fam]):
                print(f"{fam:14s} src={src[fam]:5d} xml={xml[fam] if xml else '-':>5}")
        return
    if not (a.corpus and a.sweep):
        ap.error("need <corpus.tsv> <sweep_dir> or --doc")
    rows = []
    with open(a.corpus) as f:
        for r in csv.reader(f, delimiter="\t"):
            if len(r) < 2:
                continue
            bundle, tex = r[0], r[1]
            name = os.path.splitext(os.path.basename(tex))[0]
            xml = os.path.join(a.sweep, bundle, name, name + ".xml")
            if not os.path.exists(tex):
                continue
            # Only completed conversions: a Fatal (status 3) or timeout (124)
            # leaves a partial XML — that is the no-XML set, not a markup gap.
            verdict = os.path.join(a.sweep, bundle, name, "verdict.tsv")
            if os.path.exists(verdict):
                with open(verdict) as vf:
                    v = vf.read().split("\t")
                if len(v) > 2 and v[2].strip() not in ("0", "1", "2"):
                    continue
            src, x = analyse(tex, xml)
            if x is None:
                continue
            rows.append((bundle, name, src, x))
    agg = defaultdict(lambda: [0, 0, 0, 0])  # docs with construct, docs short, src sum, covered sum
    gaps = []
    for bundle, name, src, x in rows:
        for fam in FAMS:
            s, n = src[fam], x[fam]
            if s == 0:
                continue
            a_ = agg[fam]
            a_[0] += 1
            a_[2] += s
            a_[3] += min(s, n)
            if n < s:
                a_[1] += 1
                gaps.append((s - n, fam, f"{bundle}/{name}", s, n))
    if a.out:
        with open(a.out, "w") as f:
            f.write("doc\t" + "\t".join(f"{fam}_src\t{fam}_xml" for fam in FAMS) + "\n")
            for bundle, name, src, x in rows:
                f.write(f"{bundle}/{name}\t" + "\t".join(f"{src[fam]}\t{x[fam]}" for fam in FAMS) + "\n")
    print(f"docs={len(rows)}")
    print(f"{'family':14s} {'docs':>5s} {'short':>5s} {'src':>7s} {'coverage':>8s}")
    for fam in FAMS:
        d, sh, s, c = agg[fam]
        if d:
            print(f"{fam:14s} {d:5d} {sh:5d} {s:7d} {100.0 * c / s:7.1f}%")
    print(f"\nlargest deficits (src - xml):")
    for miss, fam, doc, s, n in sorted(gaps, reverse=True)[: a.top]:
        print(f"  {miss:5d} {fam:14s} {doc} (src {s}, xml {n})")


if __name__ == "__main__":
    sys.exit(main())
