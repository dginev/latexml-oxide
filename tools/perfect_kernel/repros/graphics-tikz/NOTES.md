# graphics-tikz topic — Checkpoint 1 notes (2026-09-03, binary b54r)

Judge: ANSI-stripped `^Error:|^Fatal:`. All repros verified RED on b54r with
`--preload=[rawstyles,rawclasses]latexml.sty`, and pdflatex 0 errors (oracle).
Perl (same-host, /home/deyan/perl5/bin/latexml) counts noted per root.

## Candidate list (residue #35, my topic), ranked by docs x error mass

1. iodhbwm/iodhbwm (147) — color 'none' — SHARED — REPRO ✓ (Root A)
2. dsptricks/dspTricksManual (101) — pstricks \psset no-op → \psk@mnodesize — SHARED/structural — REPRO ✓ (Root B; entangled: alignment psmatrix `&` + parked luapstricks \directlua)
3. tikzbricks/tikzbricks-doc (90) — pgfmath sign() decimal → \ifnum relational — SHARED — REPRO ✓ (Root C)
4. beamer-theme-albi/…-doc (57) — \beamer@size + pgfkeys choice — beamer/pgfkeys (borderline; not pure graphics)
5. zx-calculus/zx-calculus (42) — tikz "Giving up on this path" — tikz-cd/matrix path parse (hard to minimize; \arrow[r] chains)
6. xskak/xskak_and_beamer (34) — \color<overlay>{blue} read as color '<' — beamer overlay spec not stripped by \color
7. schooldocs/schooldocs-examples (17) — \definecolor undefined — schooldocs.sty raw-load cascade (xcolor never required), NOT pure color
8. movie15/overlay-example (15) — \@urlbordercolor — hyperref color (borderline)
9. fancyqr/fancyqr-doc (5) — \@declaredcolor + \moveto — color/graphics internals
10. braids/braids (4) — pgf "No shape named strands-3-s" — pgf shapes (deep)
11. beamerswitch/…-example (3) — \pgfpagesuselayout — pgfpages
12. bxcoloremoji-shortnames (3) — "graphicx not loaded" — graphicx load-detection
13. colorspace/colorspace (2) — "Unknown spot color" — colorspace spot colors
14. prtec/PRTEC24-template (2) — \__color_backend_reset: — l3 color backend cs
15. pagelayout/example-template (2), example-text (2) — malformed ltx:picture (close </ltx:picture> not open) — picture/svg output structural (explicit residue item)
16. pgfornament/tikzrput (2), ornaments (1) — \rput — pstricks \rput
17. europasscv (1) — color named '\ecv@textcolor' — a CS passed as color spec
18. callouts (1) — \color undefined (color not loaded)
19. pagelayout/example-grid (1) — \Ginclude@graphics undefined — graphicx internal
NOT mine: nicematrix \TikzEveryCell (alignment agent); colortbl-DE \therownum (counter);
  coloredtheorem ltx:listingline (listings); udepcolor babel option.
PARKED-adjacent: luapstricks/\directlua (dsptricks, newpax), pTeX families.

## Root A — xcolor `\color@<name>` raw-interop fallback over-expands (iodhbwm, 147)
- Mechanism: ydoc-desc.sty:22 `\expandafter\def\csname\string\color@none\endcsname{\xcolor@ {}{}{}{}}`
  makes 'none' an EMPTY xcolor color; `\colorlet{cls}{none}` (:108) looks it up. Rust's
  fallback (latexml_package/src/package/color_sty.rs:185-203) reads `\color@none` via
  `do_expand` (:187) — but `\xcolor@` is defined (xcolor_sty.rs:586 `\def\xcolor@#1#2#3#4{#2}`),
  so full expansion collapses the payload to "", the `parts.len() >= 5` guard (:193) fails, and
  it falls to "Can't find color named 'none'; assuming Black" (:218-222).
- Classification: SHARED — Perl also errors (1: "color 'none' is undefined"), but pdflatex clean →
  in-scope surpass. Rust already tried to surpass here; the fallback is just buggy.
- Fix (Checkpoint N): in color_sty.rs::lookup_color_obj, read the DEFINITION BODY (replacement
  text) of `\color@<name>` (single-step / macro-body inspection), not `do_expand`; then parse the
  trailing `{model}{spec}` textually (empty/empty ⇒ no-op color, return current ink silently).
- Guard: 0 errors + `\textcolor{cls}{hello}` emits <ltx:text> with no color= attribute.
- Risk: LOW. Gain: iodhbwm (1 doc) + likely the "5-bundle none cluster" (macros2e/ydoc family) the
  existing comment references; verify newpax (7, pgf `none`) separately — it may be a different path.

## Root B — pstricks `\psset` stubbed to no-op kills all `\psk@*` key storage (dsptricks, 101)
- Mechanism: pst-node.tex:1248 `\define@key[psset]{pst-node}{mnodesize}[-1pt]{...}` + :1257
  `\psset[pst-node]{mnodesize=-1pt}` should `\let\psk@mnodesize\relax`. Rust stubs `\psset` to a
  no-op (pstricks_support_sty.rs:36 and pstricks_sty.rs:29 `def_macro_noop("\\psset{}")`), so no
  key handler ever runs and `\psk@mnodesize` stays undefined (verified: both plain
  `\psset{mnodesize=..}` and family `\psset[pst-node]{..}` leave it UNDEFINED).
- Classification: minimal repro SHARED (Perl also `Error:undefined:\psk@mnodesize`, 1) because it
  pokes the macro directly; but the REAL-doc divergence is structural — Rust raw-loads pstricks
  while Perl has a full 557-line pst-node.sty.ltxml binding (its own psmatrix, no raw `\psk@*`).
  BINDINGS OUTRANK RAW.
- Fix (Checkpoint N): LARGE. Either (a) port a faithful pst-node/pstricks binding (Perl-shaped), or
  (b) make Rust's `\psset` actually dispatch to xkeyval `\setkeys{psset}{..}` so `\define@key[psset]`
  handlers run. Entangled: real doc ALSO blocked by psmatrix `&` (alignment topic) and luapstricks
  `\directlua` (parked). DEPRIORITIZE — low net doc gain for high effort.
- Risk: HIGH. Gain: likely 0 full-clean docs until alignment + luapstricks also resolved.

## Root C — pgfmath `sign()` returns a decimal, breaking `\ifnum` (tikzbricks, 90)
- Mechanism: tikzbricks.sty:146-151 `\pgfmathparse{sign(sin(\tdplotmainphi))}` →
  `\let\brick@sin\pgfmathresult` → `\ifnum\brick@sin<0`. Real pgf sign()
  (pgfmathfunctions.basic.code.tex:312-325) returns integer literals `\def\pgfmathresult{1|0|-1}`;
  oracle: pdflatex `\pgfmathparse{sign(sin(0))}` ⇒ `0` (integer), while `sin(0)` ⇒ `0.0`. Rust
  pgfmath sign (pgfmath_code_tex.rs:387-395) returns f64 1.0/0.0/-1.0 → formatted WITH '.'.
  `\ifnum` reads the int part then hits '.', and compare() (tex_logic.rs:143-156) errors
  "Expected a relational token ... Got '.'".
- Classification: SHARED — Perl also emits the relational-token error + cascade (7 lines);
  pdflatex clean → in-scope surpass.
- Fix (Checkpoint N): make pgfmath `sign` (and other integer-valued fns / boolean ops per pgf)
  emit an integer-formatted `\pgfmathresult` (`1`/`0`/`-1`), matching pgf's `\def\pgfmathresult{..}`.
  Localise to the sign case first; audit floor/ceil/round/int and comparison operators similarly.
- Guard: 0 errors + a non-empty <ltx:picture>/SVG for `\brick[color=blue]{4}{2}`.
- Risk: LOW-MED (result-formatting; make sure only whole-number-returning fns lose the decimal).
  Gain: tikzbricks (1) + any doc doing `\ifnum`/`\ifodd` on a pgfmath integer function.

## Dead ends
- Raw tikz `\fill[fill=none,draw=none]` is CLEAN — the iodhbwm 'none' is NOT tikz option handling;
  it is xcolor `\color@none` lookup via ydoc-desc.
- Minimal `\psmatrix` hits "Stray alignment `&`" (alignment topic) BEFORE `\psk@mnodesize`; the
  `\ifx\psk@mnodesize\@undefined` probe isolates the real pstricks root cleanly.
