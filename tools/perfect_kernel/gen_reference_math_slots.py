#!/usr/bin/env python3
"""Generate latexml_engine/src/latex_constructs/reference_math_slots.rs: every symbol fontmath.ltx and
amsfonts.sty/amssymb.sty declare, with the symbol font's encoding and the slot (`\\DeclareMathSymbol`, and the small
variant of a control-sequence `\\DeclareMathDelimiter`, which `\\DeclareMathSymbol` defines; sect08.rs), and the token
LaTeXML's own kernel/amssymb binding makes of the name: its glyph, role, meaning, name and stretchiness. Also every math
accent fontmath.ltx declares (`\\DeclareMathAccent`), with the token the binding makes of it (`reference_math_accent`):
an accent's name is its meaning, so a package's redeclaration at another slot of a font with no map takes it whatever
the slot (sect08.rs). And every accent name LaTeXML's own package bindings define as an over accent (`DefMath!` with
`operator_role => "OVERACCENT"`; mathabx's `\\widecheck`, `\\widebar`), captured from each binding that
defines it (`--capture-packages`, one document per binding, each loading only that binding); a name two bindings give
different tokens is left out, and so is a name some definition in latexml_package, latexml_engine or latexml_core
makes other than as such an accent (a `DefMacro!`, a `DefConstructor!`, a `DefMath!` without the role or with
`UNDERACCENT`, or one outside a `*_sty.rs` binding, which no capture loads), each named in the table's comment.

The binding's token is captured, not transcribed: step 1 writes a document typesetting every name, with amssymb loaded
and `\\meaning` beside it; step 2 reads its conversion (`latexml_oxide --xml --nomathparse`, so each token keeps the
role its definition gives).

The slots a mapped `U` family's fonts have no character at (`--u-font-gaps`): tftopl over every size of the family
(the fonts each umsa.fd, umsb.fd, ueur.fd `\\DeclareFontShape` loads, read through kpsewhich and checked against
`U_FAMILIES`: msam5/7/10, msbm5/7/10, eurm5/7/10 and eurb5/7/10), each family's gaps the slots 0-127 one
of its fonts lacks (the union, should the sizes disagree, which the table's comment then says), written into the
generated block of latexml_package/src/package/u_fontmap.rs in place. There pdflatex says "Missing character" and
prints nothing, so no reference name decodes the slot (sect08.rs `symbol_font_lacks_slot`).

Usage:
  python3 -I gen_reference_math_slots.py --capture <fontmath.ltx> <amsfonts.sty> <amssymb.sty> > capture.tex
  latexml_oxide --xml --nomathparse --dest=capture.xml capture.tex
  python3 -I gen_reference_math_slots.py --capture-packages <fontmath.ltx> <latexml_package/src/package> <capture dir>
  for f in <capture dir>/*.tex; do latexml_oxide --xml --nomathparse --dest=${f%.tex}.xml $f; done
  python3 -I gen_reference_math_slots.py <fontmath.ltx> <amsfonts.sty> <amssymb.sty> capture.xml '<binary label>' \
      <capture dir> > OUT
  python3 -I gen_reference_math_slots.py --u-font-gaps <u_fontmap.rs>
"""
import glob
import os
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
# A package binding's math definition (`DefMath!("\\widecheck Digested", "\u{02C7}", operator_role => "OVERACCENT");`):
# its control word, then the rest of the call, where an accent says its operator role.
BINDING_DEFMATH = re.compile(r'DefMath!\(\s*"\\\\([A-Za-z@]+)(?=[\s{\["])[^"]*"(.*?)\);', re.S)
# Only an over accent: TeX sets a `\mathaccent` above its nucleus (tex.web §738 `make_math_accent`), so an under
# accent (`\utilde`, `\underleftarrow`) is no meaning a `\DeclareMathAccent` can have.
BINDING_ACCENT_ROLE = re.compile(r'operator_role\s*=>\s*"OVERACCENT"')
# Any binding definition of a control word, in either string form (`DefMacro!("\\widebar{}", …)`,
# `DefConstructor!(r"\widebar …", …)`, `DefMath!`, `DefPrimitive!` …), in the crates holding LaTeXML's bindings.
BINDING_DEF = re.compile(r'\b(Def[A-Za-z]*)!\(\s*(?:"\\\\|r#*"\\)([A-Za-z@]+)(?![A-Za-z@])')
BINDING_CRATES = ("latexml_package", "latexml_engine", "latexml_core")
BINDINGS_MANIFEST = "bindings.txt"


def tl_year():
    """The TeX Live release the tools come from, as tftopl names it (`TFtoPL 3.3 (TeX Live 2025)`, or a distribution's
    build, `TFtoPL 3.3 (TeX Live 2025/Debian)`)."""
    banner = subprocess.run(["tftopl", "--version"], capture_output=True, text=True).stdout
    match = re.search(r"\(TeX Live (\d{4})[^)]*\)", banner)
    if not match:
        sys.exit(f"tftopl --version names no TeX Live release: {banner.splitlines()[:1]}")
    return match.group(1)


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


def binding_accents(package_dir, kernel):
    """Each package binding (`<binding>_sty.rs`) and the accent names it defines (`DefMath!` with an `OVERACCENT`
    operator role), but the kernel's (fontmath.ltx's, the kernel binding's) and internal names (`@`); and the names
    left out because some definition of them is not such a `DefMath!` (`binding_definitions_otherwise`)."""
    found = {}
    for path in sorted(glob.glob(os.path.join(package_dir, "*_sty.rs"))):
        text = open(path, encoding="utf-8").read()
        names = sorted({"\\" + name for name, rest in BINDING_DEFMATH.findall(text)
                        if BINDING_ACCENT_ROLE.search(rest) and "@" not in name and "\\" + name not in kernel})
        if names:
            found[os.path.basename(path)[:-len("_sty.rs")]] = names
    otherwise = binding_definitions_otherwise(package_dir, {name for names in found.values() for name in names})
    found = {binding: kept for binding, names in found.items() if (kept := [n for n in names if n not in otherwise])}
    return found, otherwise


def binding_definitions_otherwise(package_dir, names):
    """Each of `names` some definition in LaTeXML's binding crates (`BINDING_CRATES`, beside `package_dir`'s) makes
    other than as an over accent the capture reads, a package binding's `DefMath!("\\<name>…", …, operator_role =>
    "OVERACCENT")`: a `DefMacro!` (`\\widebar` as `\\overline{#1}`), a `DefConstructor!`, a `DefMath!` without the
    role or with `UNDERACCENT`, or any definition outside a `*_sty.rs` binding, which no capture document loads. Such
    a name's token would not be the only meaning a binding gives it, so the table's agreement could not be read."""
    root = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(package_dir))))
    if not all(os.path.isdir(os.path.join(root, crate, "src")) for crate in BINDING_CRATES):
        sys.exit(f"{package_dir}: not latexml_package/src/package beside {', '.join(BINDING_CRATES[1:])}")
    otherwise = {}
    for crate in BINDING_CRATES:
        for path in sorted(glob.glob(os.path.join(root, crate, "src", "**", "*.rs"), recursive=True)):
            text = open(path, encoding="utf-8").read()
            for match in BINDING_DEF.finditer(text):
                name = "\\" + match.group(2)
                if name not in names:
                    continue
                call = BINDING_DEFMATH.match(text, match.start())
                if (path.endswith("_sty.rs") and os.path.dirname(os.path.abspath(path)) == os.path.abspath(package_dir)
                        and call and BINDING_ACCENT_ROLE.search(call.group(2))):
                    continue
                line = text.count("\n", 0, match.start()) + 1
                otherwise.setdefault(name, []).append(f"{os.path.relpath(path, root)}:{line}:{match.group(1)}")
    return otherwise


def capture_packages(package_dir, out_dir, kernel):
    """One capture document per package binding defining an accent, loading only that binding, each accent over `x`
    (as `capture_document`'s), and the manifest naming each document's binding and accents."""
    os.makedirs(out_dir, exist_ok=True)
    manifest = []
    found, otherwise = binding_accents(package_dir, kernel)
    for binding, names in found.items():
        out = ["\\documentclass{article}", f"\\usepackage{{{binding}}}", "\\begin{document}"]
        out += [f"\\par Q{i}Q ${name}{{x}}$" for i, name in enumerate(names)]
        out.append("\\end{document}")
        with open(os.path.join(out_dir, binding + ".tex"), "w", encoding="utf-8") as doc:
            doc.write("\n".join(out) + "\n")
        manifest.append(" ".join([binding, *names]))
    # A name left out, as `# <name> <file>:<line>:<Def*>…`: the table's comment names it.
    manifest += [" ".join(["#", name, *sites]) for name, sites in sorted(otherwise.items())]
    with open(os.path.join(out_dir, BINDINGS_MANIFEST), "w", encoding="utf-8") as out:
        out.write("\n".join(manifest) + "\n")
    print(f"{len(found)} bindings define accents: {' '.join(found)}; left out, defined otherwise: "
          f"{' '.join(otherwise) or 'none'}", file=sys.stderr)


def package_accent_tokens(capture_dir):
    """Each package accent name's token, where every binding defining it gives the same one: `(agreed, disagreed)`,
    `agreed` the name's token and its bindings, `disagreed` each other name's bindings and tokens; and `otherwise`,
    each name left out as some definition of it is not a captured over accent, and those definitions. A binding
    whose capture makes no token of a name stops the generator: the name's meaning would be unread."""
    by_name, otherwise = {}, {}
    for line in open(os.path.join(capture_dir, BINDINGS_MANIFEST), encoding="utf-8").read().split("\n"):
        if not line.strip():
            continue
        if line.startswith("#"):
            _hash, name, *sites = line.split()
            otherwise[name] = sites
            continue
        binding, *names = line.split()
        tokens = captured_tokens(os.path.join(capture_dir, binding + ".xml"), [], names)
        for name in names:
            if name not in tokens:
                sys.exit(f"{binding}: no token captured for the accent {name}")
            by_name.setdefault(name, []).append((binding, tokens[name]))
    agreed, disagreed = {}, {}
    for name, captured in sorted(by_name.items()):
        if len({token for _, token in captured}) == 1:
            agreed[name] = (captured[0][1], [binding for binding, _ in captured])
        else:
            disagreed[name] = captured
    return agreed, disagreed, otherwise


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


# Each mapped `U` family (u_fontmap.rs) and the fonts its .fd file loads, every series and size: checked against the
# .fd (`fd_fonts`), so a font the .fd adds or drops stops the generator.
U_FAMILIES = {"msa": ["msam5", "msam7", "msam10"], "msb": ["msbm5", "msbm7", "msbm10"],
              "eur": ["eurm5", "eurm7", "eurm10", "eurb5", "eurb7", "eurb10"]}
GAPS_BEGIN = "// @generated begin: gen_reference_math_slots.py --u-font-gaps"
GAPS_END = "// @generated end"


def fd_fonts(family):
    """The fonts `u<family>.fd` loads, in order: each `\\DeclareFontShape{U}{<family>}`'s size entries' font names
    (`<-6>msam5`, `<8->msam10`). A size function (`gen *`, `s *`) or another family's shape stops the generator."""
    path = subprocess.run(["kpsewhich", f"u{family}.fd"], capture_output=True, text=True).stdout.strip()
    if not path:
        sys.exit(f"kpsewhich finds no u{family}.fd")
    text = "\n".join(line.split("%")[0] for line in open(path, encoding="latin-1").read().splitlines())
    fonts = []
    for fd_family, sizes in re.findall(r"\\DeclareFontShape\s*\{U\}\s*\{(\w+)\}\s*\{\w*\}\s*\{\w*\}\s*\{([^{}]*)\}", text):
        if fd_family != family:
            sys.exit(f"{path}: a shape of the family {fd_family}, not {family}")
        for entry in re.split(r"<[^>]*>", sizes)[1:]:
            if entry.strip() and not re.fullmatch(r"\s*[A-Za-z][A-Za-z0-9]*\s*", entry):
                sys.exit(f"{path}: the size entry {entry.strip()!r} is no font name")
            fonts += [entry.strip()] if entry.strip() and entry.strip() not in fonts else []
    if not fonts:
        sys.exit(f"{path}: no \\DeclareFontShape for U/{family}")
    return path, fonts


def tfm_characters(font):
    """The character codes `font`'s TFM has (tftopl's `(CHARACTER C a` / `(CHARACTER O 101`), and its tftopl banner."""
    path = subprocess.run(["kpsewhich", font + ".tfm"], capture_output=True, text=True, check=True).stdout.strip()
    pl = subprocess.run(["tftopl", path], capture_output=True, text=True, check=True).stdout
    codes = set()
    for kind, value in re.findall(r"^\(CHARACTER ([CO]) (\S+)", pl, re.M):
        codes.add(ord(value) if kind == "C" else int(value, 8))
    if not codes:
        sys.exit(f"{font}: tftopl lists no character in {path}")
    return codes, " ".join(pl.splitlines()[0].strip("()").split()[1:])


def u_font_gaps(path):
    rows, notes, banners = [], [], []
    for family, fonts in U_FAMILIES.items():
        fd_path, loaded = fd_fonts(family)
        if loaded != fonts:
            sys.exit(f"{fd_path} loads {' '.join(loaded)}, U_FAMILIES says {' '.join(fonts)}: update U_FAMILIES")
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
    block = [GAPS_BEGIN + f", by tftopl (TeX Live {tl_year()}) over", *banners,
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
    if sys.argv[1] == "--capture-packages":
        _table, accents, _versions = declarations(sys.argv[2:3])
        capture_packages(sys.argv[3], sys.argv[4], accents)
        return
    if sys.argv[1] == "--capture":
        table, accents, _versions = declarations(sys.argv[2:5])
        sys.stdout.write(capture_document(sorted(table), sorted(accents)))
        return
    table, accents, versions = declarations(sys.argv[1:4])
    names = sorted(table)
    accent_names = sorted(accents)
    tokens = captured_tokens(sys.argv[4], names, accent_names)
    package_accents, disagreed, otherwise = package_accent_tokens(sys.argv[6])
    out = ["// @generated by tools/perfect_kernel/gen_reference_math_slots.py from "
           + "; ".join(versions) + f" (TeX Live {tl_year()});",
           f"// each binding token captured from {sys.argv[5]}, --nomathparse, amssymb loaded; each package accent's",
           "// from a document loading only the binding that defines it.",
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
            "/// (its symbol font and slot there in the comment), as the accent's operator; or of an accent name LaTeXML's",
            "/// own package bindings define, where every binding defining it makes the same token (the bindings in the",
            "/// comment).",
            *(["/// Left out, the bindings giving different tokens: "
               + "; ".join(f"`{name}` (" + ", ".join(f"{binding} {rust_str(token[0])}{' ' + token[1] if token[1] else ''}"
                                                       f"{' stretchy' if token[4] else ''}"
                                                       for binding, token in captured) + ")"
                           for name, captured in disagreed.items()) + "."] if disagreed else []),
            *(["/// Left out, a binding defining the name other than as an over accent: "
               + "; ".join(f"`{name}` (" + ", ".join(f"{site.rsplit(':', 1)[0]} `{site.rsplit(':', 1)[1]}!`"
                                                       for site in sites) + ")"
                           for name, sites in otherwise.items()) + "."] if otherwise else []),
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
    for name, ((glyph, role, meaning, tname, stretchy), bindings) in package_accents.items():
        out.append(f'    {rust_str(name)} => t({rust_str(glyph)}, {rust_opt(role)}, {rust_opt(meaning)}, {rust_str(tname)}, '
                   f'{rust_opt(stretchy)}), // {" ".join(bindings)}')
    out += ["    _ => return None,", "  })", "}", ""]
    sys.stdout.write("\n".join(out))
    print(f"{len(table)} entries, {len(unbound)} with no binding token: {' '.join(unbound)}; {len(accents)} accents; "
          f"{len(package_accents)} package accents; left out, the bindings disagreeing: {' '.join(disagreed)}, "
          f"defined otherwise: {' '.join(otherwise) or 'none'}",
          file=sys.stderr)


if __name__ == "__main__":
    main()
