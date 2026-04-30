#!/usr/bin/env python3
"""Audit Perl Def* vs Rust Def*! macro-kind parity.

For each paired (Perl .ltxml, Rust .rs) file, extracts each CS-name
definition and reports mismatches where the Perl top-level macro
(DefMacro, DefPrimitive, DefConstructor, DefRegister, DefConditional,
DefMath, DefParameterType, ...) does not correspond to the expected
Rust macro (DefMacro!, DefPrimitive!, ...).

Usage:
  ./tools/audit_def_parity.py                    # audit all engine files
  ./tools/audit_def_parity.py --dir package      # audit package/
  ./tools/audit_def_parity.py --dir contrib      # audit contrib/
  ./tools/audit_def_parity.py --file etex        # single pair only

Output: one line per mismatch:
  <rel_path>  <cs_name>  perl=<PerlKind>  rust=<RustKind or MISSING>
"""
import argparse
import os
import re
import sys
from pathlib import Path

# Perl kind name -> expected Rust macro name (sans `!`).
# `I` suffix variants (DefMacroI / DefPrimitiveI) collapse to the non-I form:
# Rust uses the prototype to determine raw-vs-tokenized body handling.
KIND_MAP = {
  "DefMacro":          "DefMacro",
  "DefMacroI":         "DefMacro",
  "DefPrimitive":      "DefPrimitive",
  "DefPrimitiveI":     "DefPrimitive",
  "DefConstructor":    "DefConstructor",
  "DefConstructorI":   "DefConstructor",
  "DefRegister":       "DefRegister",
  "DefRegisterI":      "DefRegister",
  "DefConditional":    "DefConditional",
  "DefConditionalI":   "DefConditional",
  "DefMath":           "DefMath",
  "DefMathI":          "DefMath",
  "DefParameterType":  "DefParameterType",
  "DefEnvironment":    "DefEnvironment",
  "DefEnvironmentI":   "DefEnvironment",
  "DefKeyVal":         "DefKeyVal",
  "DefColor":          "DefColor",
  "DefColorModel":     "DefColorModel",
  "DefPrimitiveIf":    "DefPrimitiveIf",
}

# Match only top-level Perl Def* calls. Perl LaTeXML uses 0-indent for top-level
# definitions; nested Def* calls inside `beforeDigest => sub { … DefEnvironmentI(…) }`
# or similar sub bodies are indented 4+ spaces and should not be counted as parity
# targets (they install conditional/scoped definitions at runtime, not at load time).
PERL_DEF_RE = re.compile(
  r"^(Def[A-Z][A-Za-z]+)\s*\(\s*['\"]([^'\"]+)['\"]",
  re.MULTILINE,
)
RUST_DEF_RE = re.compile(
  r"(Def[A-Z][A-Za-z]+)!\s*\(\s*\"((?:[^\"\\]|\\.)*?)\"",
  re.MULTILINE,
)

# Rust files also use Let!, LetI!, DefAutoload!, LoadPool!, InputDefinitions!
# — those are not Def* so don't collide.

def scan_perl(path: Path):
  """Return list of (cs, kind, lineno)."""
  out = []
  text = path.read_text(encoding="utf-8", errors="replace")
  for m in PERL_DEF_RE.finditer(text):
    kind, cs = m.group(1), m.group(2)
    # Perl `DefKeyVal('KEYSET', 'KEY', 'type', …)` passes a keyset name as
    # its first string arg, NOT a CS. Skip — otherwise the audit conflates
    # the keyset with a Rust `\<keyset>` DefMacro of the same bare name
    # (e.g. Perl `DefKeyVal('tabular', 'width', …)` vs Rust
    # `DefMacro!("\\tabular[]{}", …)`).
    if kind == "DefKeyVal" and not cs.startswith("\\"):
      continue
    # CS may include the leading '\'; strip it for comparison.
    if cs.startswith("\\"):
      cs = cs[1:]
    # Keep only the head CS name (up to first whitespace/[{(<>/).
    cs_head = re.split(r"[\s\[\]{}()<>/]", cs, 1)[0]
    # Skip entries whose cs_head is empty — Perl `\[`, `\]`, `\(`, `\)` and
    # Rust `DefEnvironment!("{envname}...")` both collapse to "" under this
    # splitter, producing spurious matches between unrelated entries.
    if not cs_head:
      continue
    lineno = text[: m.start()].count("\n") + 1
    out.append((cs_head, kind, lineno))
  return out

def strip_rust_line_comments(text: str) -> str:
  """Replace `//` line comments (and `///` doc comments) with blank of same length
  so line numbers stay stable but Def*! occurrences inside comments don't match."""
  out = []
  for line in text.splitlines(keepends=True):
    # Find `//` outside of string literals. Cheap heuristic: only consider
    # `//` that is not preceded by an open `"` on the same line.
    i = 0
    in_str = False
    in_raw = False
    cut = None
    while i < len(line):
      c = line[i]
      if not in_str and c == '/' and i + 1 < len(line) and line[i + 1] == '/':
        cut = i
        break
      if not in_str and c == '"':
        in_str = True
      elif in_str and c == '"' and (i == 0 or line[i - 1] != '\\'):
        in_str = False
      i += 1
    if cut is not None:
      # Keep any trailing newline to preserve line count.
      tail = line[cut:].rstrip('\n').rstrip('\r')
      suffix = line[cut + len(tail):]
      out.append(line[:cut] + ' ' * len(tail) + suffix)
    else:
      out.append(line)
  return ''.join(out)

def scan_rust(path: Path):
  """Return list of (cs, kind, lineno)."""
  out = []
  raw = path.read_text(encoding="utf-8", errors="replace")
  text = strip_rust_line_comments(raw)
  for m in RUST_DEF_RE.finditer(text):
    kind, cs = m.group(1), m.group(2)
    # Rust string is r"\foo" or "\\foo"; unescape the first '\\' → '\'.
    cs_unescaped = cs.replace("\\\\", "\\")
    if cs_unescaped.startswith("\\"):
      cs_unescaped = cs_unescaped[1:]
    cs_head = re.split(r"[\s\[\]{}()<>/]", cs_unescaped, 1)[0]
    lineno = text[: m.start()].count("\n") + 1
    out.append((cs_head, kind, lineno))
  return out

def pair_files(perl_root: Path, rust_roots, rel_filter=None):
  """Yield (rel_name, perl_path, rust_path, skipped_reason|None).

  `rust_roots` is either a Path or a list of Paths — when multiple roots
  are given, the first one that contains the mapped Rust file wins.
  The package-vs-contrib split has many Perl `Package/*.ltxml` entries
  that port into either `latexml_package/src/package` or
  `latexml_contrib/src`, so scanning both roots is needed for completeness.
  """
  if isinstance(rust_roots, Path):
    rust_roots = [rust_roots]
  # Discover by iterating Perl pool files and mapping to Rust.
  for perl_file in sorted(perl_root.iterdir()):
    if not perl_file.is_file():
      continue
    name = perl_file.name
    # Skip non-.ltxml and sub-files
    if not name.endswith(".ltxml"):
      continue
    # Heuristic mapping. Engine .pool.ltxml drops the .pool suffix;
    # package/contrib encode the TeX extension in the Rust filename
    # (`foo.sty.ltxml` → `foo_sty.rs`).
    stem = name
    rust_name = None
    for (suffix, rust_suffix) in (
        (".pool.ltxml", ""),
        (".sty.ltxml",  "_sty"),
        (".cls.ltxml",  "_cls"),
        (".def.ltxml",  "_def"),
        (".tex.ltxml",  "_tex"),
        (".ltx.ltxml",  "_ltx"),
        (".ltxml",      ""),
    ):
      if stem.endswith(suffix):
        base = stem[: -len(suffix)]
        rust_name = re.sub(r"[^A-Za-z0-9]", "_", base).lower() + rust_suffix
        stem = base
        break
    if rust_name is None:
      continue
    # Some specific overrides:
    overrides = {
      "Base": None,  # inlined in tex.rs
      "Base_Utility": "base_utilities",
      "AmSTeX": "amstex_sty",  # ambiguous mapping
      "TeX": None,  # inlined
    }
    if stem in overrides:
      override = overrides[stem]
      if override is None:
        continue
      rust_name = override
    if rel_filter and rel_filter not in rust_name:
      continue
    rust_file = None
    for root in rust_roots:
      candidate = root / f"{rust_name}.rs"
      if candidate.exists():
        rust_file = candidate
        break
    if rust_file is None:
      first = rust_roots[0] / f"{rust_name}.rs"
      yield (stem, perl_file, first, f"Rust file not found: {first.name}")
      continue
    yield (stem, perl_file, rust_file, None)

def audit_pair(name: str, perl_file: Path, rust_file: Path):
  perl_defs = scan_perl(perl_file)
  rust_defs = scan_rust(rust_file)
  # TeX semantics: last definition wins. If a CS appears multiple times
  # on either side with different kinds (e.g. Perl `DefMacro('\fax', ...)`
  # immediately overridden by `DefPrimitive('\fax', ...)` — the
  # `\lx@nounicode` fallback pattern in marvosym), the audit must compare
  # the final kind. Otherwise the tool flags every such override as a
  # false-positive mismatch.
  def last_wins(defs):
    last = {}
    for cs, kind, lineno in defs:
      last[cs] = (kind, lineno)
    return [(cs, k, ln) for cs, (k, ln) in last.items()]
  perl_defs = last_wins(perl_defs)
  rust_defs = last_wins(rust_defs)
  rust_map = {cs: (kind, lineno) for (cs, kind, lineno) in rust_defs}
  mismatches = []
  missing = []
  for cs, perl_kind, p_lineno in perl_defs:
    expected_rust_kind = KIND_MAP.get(perl_kind)
    if expected_rust_kind is None:
      continue  # unknown kind, skip
    rust_entry = rust_map.get(cs)
    if rust_entry is None:
      missing.append((cs, perl_kind, p_lineno))
      continue
    rust_kind, r_lineno = rust_entry
    if rust_kind != expected_rust_kind:
      mismatches.append((cs, perl_kind, p_lineno, rust_kind, r_lineno))
  return perl_defs, rust_defs, mismatches, missing

def main():
  ap = argparse.ArgumentParser()
  ap.add_argument("--dir", default="engine", choices=["engine", "package", "contrib"])
  ap.add_argument("--file", help="audit only this filename-stem")
  ap.add_argument("--verbose", action="store_true")
  args = ap.parse_args()

  root = Path(__file__).resolve().parent.parent
  if args.dir == "engine":
    perl_root = root / "LaTeXML/lib/LaTeXML/Engine"
    rust_roots = [root / "latexml_package/src/engine"]
  elif args.dir == "package":
    # Perl Package/*.ltxml ports to EITHER latexml_package/src/package
    # OR latexml_contrib/src — search both.
    perl_root = root / "LaTeXML/lib/LaTeXML/Package"
    rust_roots = [
      root / "latexml_package/src/package",
      root / "latexml_contrib/src",
    ]
  else:
    # `contrib` — Perl sources live in `ar5iv-bindings/bindings/*.ltxml`.
    # Rust ports land in `latexml_contrib/src`.
    perl_root = root / "ar5iv-bindings/bindings"
    rust_roots = [
      root / "latexml_contrib/src",
      root / "latexml_package/src/package",
    ]

  total_mismatches = 0
  total_missing_files = 0
  total_missing_defs = 0
  total_pairs_scanned = 0

  for (name, perl_file, rust_file, skip) in pair_files(perl_root, rust_roots, args.file):
    if skip:
      if args.verbose:
        print(f"SKIP {name}: {skip}", file=sys.stderr)
      total_missing_files += 1
      continue
    try:
      perl_defs, rust_defs, mismatches, missing = audit_pair(name, perl_file, rust_file)
    except Exception as e:
      print(f"ERROR {name}: {e}", file=sys.stderr)
      continue
    total_pairs_scanned += 1
    if mismatches:
      total_mismatches += len(mismatches)
      for cs, pk, pl, rk, rl in mismatches:
        print(f"{rust_file.name}:{rl}\t{cs}\tperl={pk}(L{pl})\trust={rk}")
    if missing and args.verbose:
      total_missing_defs += len(missing)
      for cs, pk, pl in missing[:5]:
        print(f"{rust_file.name}\t{cs}\tperl={pk}(L{pl})\trust=MISSING", file=sys.stderr)
      if len(missing) > 5:
        print(f"  ... and {len(missing) - 5} more MISSING in {rust_file.name}", file=sys.stderr)

  print(f"\n# Summary ({args.dir}): {total_pairs_scanned} pairs scanned, "
        f"{total_mismatches} kind mismatches, "
        f"{total_missing_files} missing rust files",
        file=sys.stderr)
  if total_missing_defs:
    print(f"# {total_missing_defs} missing Rust definitions", file=sys.stderr)
  sys.exit(0 if total_mismatches == 0 else 1)

if __name__ == "__main__":
  main()
