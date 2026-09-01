# Read-only root-cause investigation — wave 3 (perfect-kernel corpus)

Use your MAXIMUM (xhigh) reasoning effort. This is a deep root-causing task.

## Context
Repo: /home/deyan/git/latexml-oxide (branch `perfect_kernel`) — a Perl→Rust port of
LaTeXML. Perl source (ground truth) is in `LaTeXML/lib/LaTeXML/` (`*.pool.ltxml`,
`Package/*.ltxml`, `Core/*.pm`). Real TeX/LaTeX sources: `kpsewhich <file>` finds
`.sty/.cls/.tex/.ltx` under /usr/local/texlive/2025/texmf-dist; `background/`
holds tex.web / TeXbook. Rust engine crates: latexml_core (mouth/gullet/stomach/
state), latexml_engine (kernel pools: tex_*.rs, latex_*.rs, etex.rs),
latexml_package/src/package/*_sty.rs (package bindings), latexml_contrib/src.

Mission: PERFECT KERNEL EMULATION. The corpus is the TeX Live doc manuals converted
via raw interpretation of the real .sty/.cls files:

    cd /usr/local/texlive/2025/texmf-dist/doc/<bundle>/ && \
    /home/deyan/git/latexml-oxide/target/debug/latexml_oxide --timeout=300 \
      --preload='[rawstyles,rawclasses]latexml.sty' --dest=<YOUR_SCRATCH>/<name>.xml <name>.tex \
      2> <YOUR_SCRATCH>/<name>.stderr
    # errors: sed 's/\x1b\[[0-9;]*m//g' <name>.stderr | grep '^Error:\|^Fatal:'
    # (a trailing `Error: "Permission denied"` after "Wrote ..." is the log write into
    #  the read-only doc dir — ignore it)

A same-host Perl LaTeXML (0.8.8) is on PATH for SHARED-vs-RUST-ONLY classification:
    latexml --preload='[rawstyles,rawclasses]latexml.sty' --dest=<scratch>/<name>.perl.xml <name>.tex 2> <name>.perl.stderr
(Perl caps at 100 errors; a Perl timeout is not an error. Use `timeout 300`.)

## HARD RULES
- READ-ONLY: do NOT edit any repo file, do NOT run `cargo build`/`cargo test`/nextest
  (the tree and binary are owned by the main session and a running corpus sweep).
  Use the prebuilt binary above as-is. Put ALL scratch files under your own directory:
  /tmp/claude-1000/-home-deyan-git-latexml-oxide/d8421fee-1145-4525-aa04-750c83212695/scratchpad/<your-tag>/
- First principles: derive the root cause from latex.ltx / the real .sty / tex.web /
  Perl source and cite file:line. No stopgap guards, no stubs, no "skip this macro".
- Bindings outrank raw: if a Rust binding exists for a package (grep
  latexml_package/src/package and latexml_contrib/src), the fix goes into the binding's
  faithful semantics; if none exists, the fix is in the kernel/engine so raw code runs.
- PARKED families — if the root cause is one of these, say so in one line and STOP on
  that witness: (a) "current frame is mode-switch to <mode> due to …" mode-frame
  family; (b) Japanese/pTeX/upTeX engine primitives (jlreq, pLaTeX kernels, CJK
  vertical); (c) LuaTeX-only primitives that double as engine-detection probes
  (\directlua, \luatexversion, \csstring) — never propose defining them.
- Classification vocabulary: RUST-ONLY (Perl clean), SHARED (Perl fails the same way —
  still a kernel-quality bug to fix, but flag it), PERL-ORIGIN (a Perl binding bug we
  inherited — cite the .ltxml line).
- Don't chase cosmetic differences; the target is Error:/Fatal: lines, and correct
  XML structure.

## Deliverable (your final message, ≤ 2 pages, conclusions not play-by-play)
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
