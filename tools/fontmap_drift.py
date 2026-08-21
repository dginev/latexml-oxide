#!/usr/bin/env python3
"""Report where our embedded font-encoding maps drift from pdftex's own golden.

WHY THIS EXISTS
---------------
Each `*_fontmap.rs` maps a TeX font-encoding slot (0..255) to the Unicode
character LaTeXML emits for it. The authoritative "what does pdflatex put in the
PDF's ToUnicode for this slot" is a COMPOSITION of two files pdftex itself uses:

    slot --(<enc>.enc)--> glyph name --(glyphtounicode.tex)--> Unicode

Both live in the TeX tree and are found with kpsewhich; the composition is
authoritative *by construction* -- it is the exact data pdftex embeds, which is
what pdftotext reads back. This tool composes that golden for each text encoding
we ship and prints every slot where our fontmap differs.

This is a TRIPWIRE, not an oracle. LaTeXML fontmaps sometimes diverge from the
glyph's ToUnicode DELIBERATELY (a slot's semantic character vs its glyph shape,
visible-space markers, ligature slots). Such intentional divergences are listed
in ALLOWLIST below with a reason; everything else is reported for human review.
Only TEXT encodings with standard AGL glyph names can be checked -- symbol fonts
(amsa, ding, pzd, ifsym*, lcircle, line) have no such .enc and are skipped.

Usage:  python3 tools/fontmap_drift.py [--all]
        (--all also prints allowlisted, intentional divergences)
Exit:   0 = no un-allowlisted drift; 1 = an unreviewed drift appeared.
"""

import re
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
PKG = REPO_ROOT / "latexml_package" / "src" / "package"

# fontmap name (the DeclareFontMap! key, lowercased) -> the .enc kpsewhich resolves.
# Only text encodings with AGL glyph names; symbol fonts are intentionally absent.
FONTMAP_ENC = {
    "t1":  "ec.enc",
    "t2a": "t2a.enc",
    "t2b": "t2b.enc",
    "t2c": "t2c.enc",
    # Deferred — validated to MIS-align against the named .enc, so every slot
    # would report as false drift. Re-enable once the correct .enc (or the
    # fontmap's true low-slot layout) is confirmed:
    #   "ly1": "texnansi.enc"  — ly1_fontmap low slots (accents/ligatures) do
    #     not match texnansi.enc's (Euro/fraction/…): systematic +11 offset.
    #   "ts1": "tc.enc", "lgr": "lgr.enc", "ot4": "ot4.enc"  — .enc not resolved
    #     by kpsewhich in the ambient tree.
}

# Intentional divergences: (fontmap, slot) -> reason. A slot listed here is NOT
# reported unless --all. Keep in sync with a human decision; cite it.
ALLOWLIST = {
    # slots 94 (^) and 126 (~): Bruce Miller deliberately maps these to the
    # ACCENT characters U+02C6/U+02DC, not ASCII U+005E/U+007E — LaTeXML commit
    # 9ec6a4122 "Encodings (#2435)" (2024-11-20): "^ and ~ which should be
    # accents". We keep that fontmap value; ASCII in verbatim/URL contexts is
    # instead handled at the parameter level (Verbatim/HyperVerbatim install an
    # identity ASCII fontmap through digestion — see base_parameter_types.rs,
    # OXIDIZED_DESIGN #144). glyphtounicode maps asciicircum/asciitilde to ASCII.
    ("t1", 94): "Bruce #2435: ^ mapped as accent U+02C6, not ASCII",
    ("t1", 126): "Bruce #2435: ~ mapped as accent U+02DC, not ASCII",
    ("t2a", 94): "Bruce #2435: ^ mapped as accent U+02C6, not ASCII",
    ("t2a", 126): "Bruce #2435: ~ mapped as accent U+02DC, not ASCII",
    ("t2b", 94): "Bruce #2435: ^ mapped as accent U+02C6, not ASCII",
    ("t2b", 126): "Bruce #2435: ~ mapped as accent U+02DC, not ASCII",
    ("t2c", 94): "Bruce #2435: ^ mapped as accent U+02C6, not ASCII",
    ("t2c", 126): "Bruce #2435: ~ mapped as accent U+02DC, not ASCII",
    # Deliberate LaTeXML choices (all shared with Perl LaTeXML), NOT bugs:
    # slot 127 is the line-break hyphen char (\-); U+2010 is the typographic
    # HYPHEN, kept distinct from slot 45's ASCII U+002D. glyphtounicode maps
    # both glyphs to U+002D.
    ("t1", 127): "line-break hyphen → U+2010 (typographic), distinct from slot 45",
    ("t2a", 127): "line-break hyphen → U+2010 (typographic), distinct from slot 45",
    ("t2b", 127): "line-break hyphen → U+2010 (typographic), distinct from slot 45",
    ("t2c", 127): "line-break hyphen → U+2010 (typographic), distinct from slot 45",
    # T2 Cyrillic slots 14/15 hold single angle quotes; ‹›/U+2039-203A are the
    # correct quotation glyphs, vs glyphtounicode's deprecated CJK 〈〉 U+2329-232A.
    ("t2a", 14): "Cyrillic single angle quote ‹ (U+2039), not CJK 〈",
    ("t2a", 15): "Cyrillic single angle quote › (U+203A), not CJK 〉",
    ("t2b", 14): "Cyrillic single angle quote ‹ (U+2039), not CJK 〈",
    ("t2b", 15): "Cyrillic single angle quote › (U+203A), not CJK 〉",
    ("t2c", 14): "Cyrillic single angle quote ‹ (U+2039), not CJK 〈",
    ("t2c", 15): "Cyrillic single angle quote › (U+203A), not CJK 〉",
}


def sh(*args):
    return subprocess.run(args, capture_output=True, text=True).stdout.strip()


def load_glyphtounicode():
    """glyph name -> codepoint, from pdftex's own table."""
    path = sh("kpsewhich", "glyphtounicode.tex")
    if not path:
        sys.exit("FATAL: glyphtounicode.tex not found (is pdftex installed?)")
    table = {}
    for m in re.finditer(r"\\pdfglyphtounicode\{([^}]+)\}\{([0-9A-Fa-f]+)\}", Path(path).read_text()):
        table[m.group(1)] = int(m.group(2), 16)
    return table


def glyph_to_cp(name, g2u):
    """Resolve a glyph name to a codepoint: pdftex table, then uniXXXX, else None."""
    if name in g2u:
        return g2u[name]
    m = re.fullmatch(r"uni([0-9A-Fa-f]{4})", name)
    if m:
        return int(m.group(1), 16)
    m = re.fullmatch(r"u([0-9A-Fa-f]{4,6})", name)
    if m:
        return int(m.group(1), 16)
    return None  # .notdef / unknown -> no golden


def parse_enc(enc_path):
    """.enc (dvips PostScript) -> list of 256 glyph names in slot order."""
    text = Path(enc_path).read_text()
    # Strip PostScript % comments, then take the tokens inside the [...] array.
    text = re.sub(r"%[^\n]*", "", text)
    m = re.search(r"\[(.*)\]", text, re.DOTALL)
    if not m:
        return None
    names = re.findall(r"/([A-Za-z0-9._]+)", m.group(1))
    return names if len(names) >= 256 else names + [".notdef"] * (256 - len(names))


_CHAR_RE = re.compile(r"'(\\u\{([0-9A-Fa-f]+)\}|\\(.)|([^'\\]))'")


def parse_fontmap(rs_path):
    """*_fontmap.rs -> (name, [256 codepoints]). None if not a DeclareFontMap."""
    text = rs_path.read_text()
    m = re.search(r'DeclareFontMap!\("([^"]+)",\s*mixrc!\[(.*?)\]\s*\)', text, re.DOTALL)
    if not m:
        return None
    name = m.group(1)
    cps = []
    for lit in _CHAR_RE.finditer(m.group(2)):
        if lit.group(2):        # \u{XXXX}
            cps.append(int(lit.group(2), 16))
        elif lit.group(3):      # \\ , \' , \" , etc.
            cps.append(ord({"n": "\n", "t": "\t", "r": "\r", "0": "\0"}.get(lit.group(3), lit.group(3))))
        else:                   # bare char
            cps.append(ord(lit.group(4)))
    return (name, cps)


def main():
    show_all = "--all" in sys.argv
    g2u = load_glyphtounicode()
    total_drift = 0
    checked = 0
    for rs in sorted(PKG.glob("*_fontmap.rs")):
        parsed = parse_fontmap(rs)
        if not parsed:
            continue
        name, cps = parsed
        enc_name = FONTMAP_ENC.get(name.lower())
        if not enc_name:
            continue  # symbol font / unmapped -> out of scope for this pipeline
        enc_path = sh("kpsewhich", enc_name)
        if not enc_path:
            print(f"SKIP {name}: {enc_name} not found in TeX tree")
            continue
        glyphs = parse_enc(enc_path)
        if not glyphs:
            print(f"SKIP {name}: could not parse {enc_name}")
            continue
        checked += 1
        drifts = []
        for slot in range(min(256, len(cps))):
            golden = glyph_to_cp(glyphs[slot], g2u)
            if golden is None:
                continue  # no authoritative golden for this glyph
            if cps[slot] != golden:
                allow = (name.lower(), slot) in ALLOWLIST
                if allow and not show_all:
                    continue
                drifts.append((slot, glyphs[slot], cps[slot], golden, allow))
        if drifts:
            print(f"\n== {name} ({rs.name} vs {enc_name}) ==")
            for slot, glyph, ours, golden, allow in drifts:
                tag = "  [allowlisted]" if allow else ""
                print(f"  slot {slot:3d} 0x{slot:02X} /{glyph:<16} ours U+{ours:04X} '{chr(ours)}'  "
                      f"golden U+{golden:04X} '{chr(golden)}'{tag}")
                if not allow:
                    total_drift += 1
    print(f"\n-- checked {checked} text encoding(s); {total_drift} un-allowlisted drift slot(s) --")
    return 1 if total_drift else 0


if __name__ == "__main__":
    sys.exit(main())
