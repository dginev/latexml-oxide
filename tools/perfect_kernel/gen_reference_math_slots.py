#!/usr/bin/env python3
"""Generate latexml_engine/src/latex_constructs/reference_math_slots.rs: every symbol fontmath.ltx and
amsfonts.sty/amssymb.sty declare, with the symbol font's encoding and the slot (`\\DeclareMathSymbol`, and the small
variant of a control-sequence `\\DeclareMathDelimiter`, which `\\DeclareMathSymbol` defines; sect08.rs), and the token
LaTeXML's own kernel/amssymb binding makes of the name: its glyph, role, meaning, name and stretchiness. Also every math
accent fontmath.ltx declares (`\\DeclareMathAccent`), with the token the binding makes of it (`reference_math_accent`):
an accent's name is its meaning, so a package's redeclaration at another slot of a font with no map takes it whatever
the slot (sect08.rs).

The binding's token is captured, not transcribed: step 1 writes a document typesetting every name, with amssymb loaded
and `\\meaning` beside it; step 2 reads its conversion (`latexml_oxide --xml --nomathparse`, so each token keeps the
role its definition gives).

The slots a mapped `U` family's fonts have no character at (`--u-font-gaps`): tftopl over every size of the family
(umsa.fd, umsb.fd, ueur.fd: msam5/7/10, msbm5/7/10, eurm5/7/10 and eurb5/7/10), each family's gaps the slots 0-127 one
of its fonts lacks (the union, should the sizes disagree, which the table's comment then says), written into the
generated block of latexml_package/src/package/u_fontmap.rs in place. There pdflatex says "Missing character" and
prints nothing, so no reference name decodes the slot (sect08.rs `symbol_font_lacks_slot`).

Usage:
  python3 -I gen_reference_math_slots.py --capture <fontmath.ltx> <amsfonts.sty> <amssymb.sty> > capture.tex
  latexml_oxide --xml --nomathparse --dest=capture.xml capture.tex
  python3 -I gen_reference_math_slots.py <fontmath.ltx> <amsfonts.sty> <amssymb.sty> capture.xml '<binary label>' > OUT
  python3 -I gen_reference_math_slots.py --u-font-gaps <u_fontmap.rs>
"""
import re
import subprocess
import sys
import xml.etree.ElementTree as ET

ENCODINGS = {"operators": "OT1", "letters": "OML", "symbols": "OMS", "largesymbols": "OMX",
             "AMSa": "AMSa", "AMSb": "AMSb"}
# A name is a control word, a control symbol (fontmath.ltx:467 `\DeclareMathDelimiter{\|}`) or a character.
DECL = re.compile(r"\\(?:ams@)?DeclareMath(Symbol|Delimiter)\s*\{?(\\(?:[A-Za-z@]+|[^A-Za-z@\s])|[^\s{}\\])\}?\s*"
                  r"\{(\\[A-Za-z]+|\d)\}\s*\{(\w+)\}\s*\{([^}]*)\}")
ACCENT = re.compile(r"\\DeclareMathAccent\s*\{?(\\[A-Za-z]+)\}?\s*\{(\\[A-Za-z]+|\d)\}\s*\{(\w+)\}\s*\{([^}]*)\}")
LTX = "{http://dlmf.nist.gov/LaTeXML}"


def slot(text):
    text = text.strip()
    if text.startswith('"'):
        return int(text[1:], 16)
    if text.startswith("'"):
        return int(text[1:], 8)
    if text.startswith("`"):
        return ord(text[-1])
    return int(text)


def version(text):
    match = re.search(r"\\Provides(?:File|Package)\{([^}]*)\}\s*\[([^\]]*)\]", text)
    return f"{match.group(1)} [{' '.join(match.group(2).split()[:2])}]"


def declarations(paths):
    table, accents, versions = {}, {}, []
    for path in paths:
        raw = open(path, encoding="latin-1").read()
        versions.append(version(raw))
        text = "\n".join(line.split("%")[0] for line in raw.splitlines())
        for kind, name, _class, font, code in DECL.findall(text):
            # A character delimiter (`<`, `|`) sets only a delcode; a character's math symbol is its mathcode.
            if font not in ENCODINGS or (kind == "Delimiter" and not name.startswith("\\")):
                continue
            entry = (ENCODINGS[font], slot(code))
            if table.get(name, entry) != entry:
                sys.exit(f"{name} declared twice: {table[name]} and {entry}")
            table[name] = entry
        for name, _class, font, code in ACCENT.findall(text):
            if font in ENCODINGS:
                accents[name] = (ENCODINGS[font], slot(code))
    return table, accents, versions


def capture_document(names, accents):
    """The symbols, then the accents (each over `x`), numbered on, so the symbols' capture is as before."""
    out = ["\\documentclass{article}", "\\usepackage{amssymb}", "\\makeatletter", "\\begin{document}"]
    for i, name in enumerate(names):
        meaning = f" M\\texttt{{\\meaning{name}}}M" if name.startswith("\\") else ""
        out.append(f"\\par Q{i}Q ${name}${meaning}")
    for i, name in enumerate(accents, len(names)):
        out.append(f"\\par Q{i}Q ${name}{{x}}$")
    out.append("\\end{document}")
    return "\n".join(out) + "\n"


def captured_tokens(path, names, accents=()):
    """Each name's binding token, read from the capture's conversion. A name the conversion makes no single token
    of (a punctuation character set as text), or that LaTeXML defines by no binding (the kernel's raw
    `\\mathchardef`, whose `\\meaning` is a character code), has none. An accent's token is the operator of the
    application it makes over `x`."""
    names = list(names) + list(accents)
    tokens = {}
    for par in ET.parse(path).getroot().iter(LTX + "p"):
        match = re.match(r"\s*Q(\d+)Q", par.text or "")
        if not match:
            continue
        name = names[int(match.group(1))]
        meaning = "".join(t.text or "" for t in par.iter(LTX + "text"))
        if re.match(r"\\(math)?char", meaning):
            continue
        xmath = [m for m in par.iter(LTX + "XMath")]
        kids = list(xmath[0]) if len(xmath) == 1 else []
        if name in accents and len(kids) == 1 and kids[0].tag == LTX + "XMApp":
            kids = list(kids[0])[:1]
        if len(kids) != 1 or kids[0].tag != LTX + "XMTok":
            continue
        tok = kids[0]
        role = tok.get("role")
        stretchy = tok.get("stretchy")
        tokens[name] = (tok.text or "", None if role in (None, "UNKNOWN") else role, tok.get("meaning"),
                        tok.get("name") or "", None if stretchy is None else stretchy == "true")
    return tokens


def rust_str(text):
    out = []
    for ch in text:
        if ch in '\\"':
            out.append("\\" + ch)
        elif " " <= ch <= "~":
            out.append(ch)
        else:
            out.append(f"\\u{{{ord(ch):04X}}}")
    return '"' + "".join(out) + '"'


def rust_opt(value):
    if value is None:
        return "None"
    if isinstance(value, bool):
        return f"Some({str(value).lower()})"
    return f"Some({rust_str(value)})"


# Each mapped `U` family (u_fontmap.rs) and the fonts its .fd file loads, every series and size.
U_FAMILIES = {"msa": ["msam5", "msam7", "msam10"], "msb": ["msbm5", "msbm7", "msbm10"],
              "eur": ["eurm5", "eurm7", "eurm10", "eurb5", "eurb7", "eurb10"]}
GAPS_BEGIN = "// @generated begin: gen_reference_math_slots.py --u-font-gaps"
GAPS_END = "// @generated end"


def tfm_characters(font):
    """The character codes `font`'s TFM has (tftopl's `(CHARACTER C a` / `(CHARACTER O 101`), and its tftopl banner."""
    path = subprocess.run(["kpsewhich", font + ".tfm"], capture_output=True, text=True, check=True).stdout.strip()
    pl = subprocess.run(["tftopl", path], capture_output=True, text=True, check=True).stdout
    codes = set()
    for kind, value in re.findall(r"^\(CHARACTER ([CO]) (\S+)", pl, re.M):
        codes.add(ord(value) if kind == "C" else int(value, 8))
    return codes, " ".join(pl.splitlines()[0].strip("()").split()[1:])


def u_font_gaps(path):
    rows, notes, banners = [], [], []
    for family, fonts in U_FAMILIES.items():
        lacks, families = {}, []
        for font in fonts:
            codes, banner = tfm_characters(font)
            if max(codes) > 127:
                sys.exit(f"{font}: a character above 127; the gap range is 0-127")
            lacks[font] = sorted(set(range(128)) - codes)
            families += [banner] if banner not in families else []
        banners.append(f"//   {family}: {' '.join(fonts)} ({', '.join(families)})")
        union = sorted(set().union(*lacks.values()))
        if any(slots != union for slots in lacks.values()):
            notes.append(f"//   {family}: the sizes disagree, the union is recorded: "
                         + "; ".join(f"{font} {slots}" for font, slots in lacks.items()))
        rows.append(f'  ("{family}", &[{", ".join(map(str, union))}]),')
        print(f"U/{family}: {len(union)} gaps {union}", file=sys.stderr)
    block = [GAPS_BEGIN + ", by tftopl (TeX Live 2025) over", *banners,
             "// Do not edit by hand: rerun the generator."]
    block += notes or ["// Every size of each family lacks the same slots."]
    block += ["/// Each mapped `U` family's gaps: the slots 0-127 its fonts have no character at.",
              "#[rustfmt::skip]",
              f"const U_FONT_GAPS: [(&str, &[usize]); {len(rows)}] = [", *rows, "];", GAPS_END]
    text = open(path, encoding="utf-8").read()
    start, end = text.index(GAPS_BEGIN), text.index(GAPS_END) + len(GAPS_END)
    with open(path, "w", encoding="utf-8") as out:
        out.write(text[:start] + "\n".join(block) + text[end:])


def main():
    if sys.argv[1] == "--u-font-gaps":
        u_font_gaps(sys.argv[2])
        return
    if sys.argv[1] == "--capture":
        table, accents, _versions = declarations(sys.argv[2:5])
        sys.stdout.write(capture_document(sorted(table), sorted(accents)))
        return
    table, accents, versions = declarations(sys.argv[1:4])
    names = sorted(table)
    accent_names = sorted(accents)
    tokens = captured_tokens(sys.argv[4], names, accent_names)
    out = ["// @generated by tools/perfect_kernel/gen_reference_math_slots.py from "
           + "; ".join(versions) + " (TeX Live 2025);",
           f"// each binding token captured from {sys.argv[5]}, --nomathparse, amssymb loaded.",
           "// Do not edit by hand: rerun the generator.",
           "",
           "/// A math symbol's reference: the encoding of the symbol font fontmath.ltx or amsfonts/amssymb declares it",
           "/// in (`operators` OT1, `letters` OML, `symbols` OMS, `largesymbols` OMX, `AMSa`/`AMSb` msam/msbm), its slot",
           "/// there, and the token LaTeXML's own kernel/amssymb binding makes of the name, when one does.",
           "pub(crate) struct ReferenceSymbol {",
           "  pub(crate) encoding: &'static str,",
           "  pub(crate) slot:     u8,",
           "  pub(crate) binding:  Option<BindingToken>,",
           "}",
           "",
           "/// The token a binding makes of a name: its text, its role (none for `UNKNOWN`, the default), meaning, `name`",
           "/// attribute (empty for none) and stretchiness.",
           "pub(crate) struct BindingToken {",
           "  pub(crate) glyph:    &'static str,",
           "  pub(crate) role:     Option<&'static str>,",
           "  pub(crate) meaning:  Option<&'static str>,",
           "  pub(crate) name:     &'static str,",
           "  pub(crate) stretchy: Option<bool>,",
           "}",
           "",
           "const fn r(encoding: &'static str, slot: u8) -> ReferenceSymbol {",
           "  ReferenceSymbol { encoding, slot, binding: None }",
           "}",
           "",
           "#[allow(clippy::too_many_arguments)]",
           "const fn b(",
           "  encoding: &'static str,",
           "  slot: u8,",
           "  glyph: &'static str,",
           "  role: Option<&'static str>,",
           "  meaning: Option<&'static str>,",
           "  name: &'static str,",
           "  stretchy: Option<bool>,",
           ") -> ReferenceSymbol {",
           "  ReferenceSymbol {",
           "    encoding,",
           "    slot,",
           "    binding: Some(BindingToken {",
           "      glyph,",
           "      role,",
           "      meaning,",
           "      name,",
           "      stretchy,",
           "    }),",
           "  }",
           "}",
           "",
           "const fn t(",
           "  glyph: &'static str,",
           "  role: Option<&'static str>,",
           "  meaning: Option<&'static str>,",
           "  name: &'static str,",
           "  stretchy: Option<bool>,",
           ") -> BindingToken {",
           "  BindingToken {",
           "    glyph,",
           "    role,",
           "    meaning,",
           "    name,",
           "    stretchy,",
           "  }",
           "}",
           "",
           "/// The reference of the math symbol `name`, the symbol's token as a string (`\\alpha`, `+`).",
           "#[rustfmt::skip]",
           "pub(crate) fn reference_math_slot(name: &str) -> Option<ReferenceSymbol> {",
           "  Some(match name {"]
    unbound = []
    for name in names:
        enc, code = table[name]
        key = rust_str(name)
        if name in tokens:
            glyph, role, meaning, tname, stretchy = tokens[name]
            out.append(f'    {key} => b("{enc}", 0x{code:02X}, {rust_str(glyph)}, {rust_opt(role)}, {rust_opt(meaning)}, '
                       f'{rust_str(tname)}, {rust_opt(stretchy)}),')
        else:
            unbound.append(name)
            out.append(f'    {key} => r("{enc}", 0x{code:02X}),')
    out += ["    _ => return None,", "  })", "}", "",
            "/// The token LaTeXML's own kernel binding makes of the math accent `name` (`\\hat`), one fontmath.ltx declares",
            "/// (its symbol font and slot there in the comment), as the accent's operator.",
            "#[rustfmt::skip]",
            "pub(crate) fn reference_math_accent(name: &str) -> Option<BindingToken> {",
            "  Some(match name {"]
    for name in accent_names:
        enc, code = accents[name]
        if name not in tokens:
            sys.exit(f"accent {name}: no binding token captured")
        glyph, role, meaning, tname, stretchy = tokens[name]
        out.append(f'    {rust_str(name)} => t({rust_str(glyph)}, {rust_opt(role)}, {rust_opt(meaning)}, {rust_str(tname)}, '
                   f'{rust_opt(stretchy)}), // {enc} 0x{code:02X}')
    out += ["    _ => return None,", "  })", "}", ""]
    sys.stdout.write("\n".join(out))
    print(f"{len(table)} entries, {len(unbound)} with no binding token: {' '.join(unbound)}; {len(accents)} accents",
          file=sys.stderr)


if __name__ == "__main__":
    main()
