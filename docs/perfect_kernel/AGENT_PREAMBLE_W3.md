# Read-only root-cause investigation — wave 3 (perfect-kernel corpus)

(Dispatch with the `root-causer` agent type. Fill `<BINARY>`, `<TL>`, `<SCRATCH>` per wave.)

## Context
Repo: the working directory (branch `perfect_kernel`) — a Perl→Rust port of
LaTeXML. Perl source (ground truth) is in `LaTeXML/lib/LaTeXML/` (`*.pool.ltxml`,
`Package/*.ltxml`, `Core/*.pm`). Real TeX/LaTeX sources: `kpsewhich <file>` finds
`.sty/.cls/.tex/.ltx` under `<TL>/texmf-dist` (`kpsewhich -var-value=SELFAUTOPARENT`); `background/`
holds tex.web / TeXbook. Rust engine crates: latexml_core (mouth/gullet/stomach/
state), latexml_engine (kernel pools: tex_*.rs, latex_*.rs, etex.rs),
latexml_package/src/package/*_sty.rs (package bindings), latexml_contrib/src.

Mission: PERFECT KERNEL EMULATION. The corpus is the TeX Live doc manuals converted
via raw interpretation of the real .sty/.cls files:

    cd <TL>/texmf-dist/doc/<bundle>/ && \
    <BINARY> --timeout=300 \
      --preload='[rawstyles,rawclasses]latexml.sty' --dest=<YOUR_SCRATCH>/<name>.xml <name>.tex \
      2> <YOUR_SCRATCH>/<name>.stderr
    # errors: sed 's/\x1b\[[0-9;]*m//g' <name>.stderr | grep '^Error:\|^Fatal:'
    # (a trailing `Error: "Permission denied"` after "Wrote ..." is the log write into
    #  the read-only doc dir — ignore it)

A same-host Perl LaTeXML (0.8.8) is on PATH for SHARED-vs-RUST-ONLY classification:
    latexml --preload='[rawstyles,rawclasses]latexml.sty' --dest=<scratch>/<name>.perl.xml <name>.tex 2> <name>.perl.stderr
(Perl caps at 100 errors; a Perl timeout is not an error. Use `timeout 300`.)

## Rules
Your agent definition's rules apply unchanged (read-only, first principles with
file:line, bindings outrank raw, PARKED families, RUST-ONLY / SHARED / PERL-ORIGIN
classification). Wave-specific additions:
- Scratch directory for this wave: `<SCRATCH>/<your-tag>/`.
- A running corpus sweep owns the tree and binary — that is why builds are off-limits.
- Don't chase cosmetic differences; the target is Error:/Fatal: lines and correct XML
  structure.

## Deliverable (your final message — conclusions, not play-by-play)
For each root cause found:
1. **Root cause** — mechanism, with file:line into the ORIGINAL sources (latex.ltx,
   the .sty, tex.web §, Perl .ltxml) and the Rust site (file:line) that diverges.
2. **Classification** — RUST-ONLY / SHARED / PERL-ORIGIN, with the Perl error count.
3. **Minimal repro** — a ≤15-line standalone .tex (article class unless the class is
   the subject) that reproduces the error TODAY with the binary above, plus the exact
   Error line(s) it emits. Verify it actually reproduces. Name the original witness.
4. **Fix plan** — exact Rust file + function, the mechanism to implement (faithful to
   the source), and what the guard test should assert (0 errors + a structural XML
   assertion). Rate risk (LOW/MED/HIGH) and expected corpus gain (docs / error lines).
5. **Dead ends** — one line each, so they are not re-attempted.
Rank by (docs × error lines)/risk. If a witness turns out to be several unrelated
roots, list them separately. If something is genuinely a pdflatex-too failure or
out-of-scope, say so with evidence.
