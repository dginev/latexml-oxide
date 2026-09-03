# w14 topics: string-mouth + index — Checkpoint 1 (2026-09-03, binary b54l)

Judge = ANSI-stripped `^Error:|^Fatal:`. All rust counts verified with
/home/deyan/data/pk_bin/latexml_oxide.b54l, preload [rawstyles,rawclasses]latexml.sty.

## index candidates (ranked docs x error lines)
1. bibarts/bibarts (16 err), bibarts/ba-short (5 err) — RED (seed
   index/edef_write_ifmmode_bibarts.tex, re-verified 5 err). `\edef`+`\write`
   of `\ifmmode…$…$\fi` (bibarts.sty:798 \ba@textmode) + `\underline`
   (\protect\ifmmode…\else…\fi) torn at the string-mouth boundary of
   \protected@write's \edef -> orphan \fi/\else. SHARED (Perl 7). ROOT for Checkpoint N.
- manyind/mindsample (was 7) — GREEN, fixed by batch 54l (seed
  protect_deferred_manyind.tex, guard index_entry_defers_protected_macros).
- robustindex/robustmanual, multisample, robustsample — NOW GREEN under b54l
  (re-verified robustmanual = 0 err). 54l's \protect deferral fixed the
  \protected@write \newlabel re-read. DROP from worklist.
- esindex — first error \spanishdatedel undefined; plain undefined-macro, NOT
  this theme. Exclude.

## string-mouth candidates (ranked docs x error lines)
1. amsldoc-it/itamsldoc (3 err), amsldoc-vn/amsldoc-vi (4 err) — RED. P73.
   repro string-mouth/sanitizedverb_reforms_ctrlsym_amsldoc.tex (2 err:
   \KV@tempa + readBalanced). ROOT candidate for Checkpoint N.
   (amsldoc-vn also has a separate \textdotbelow undefined, Vietnamese, not this root.)
2. greek-fontenc/char-list-alphabeta (1), greek-fontenc/char-list (1) — RED. P74.
   repro string-mouth/abstract_cmd_pretokenized_greek.tex (1 err: \patch). ROOT candidate.
3. ribbonproofs/ribbonproofsmanual — RED under b54l (3 err: `\fi \iffalse` +
   readBalanced + a downstream RibbonProofs pkg error). Torn `\iffalse` at
   Anonymous String — same "conditional cut at mouth boundary" family as bibarts,
   needs its own root-cause at Checkpoint N (verify shares bibarts fix or distinct).
- greek-fontenc/test-lgrenc, textalpha-doc — first error `\@tabbing@"` (LGR active
  char tabbing), different theme. Exclude.
- screenplay-pkg — readBalanced buried under 14 frontmatter/mode errors; per
  preamble warning, likely downstream of a frontmatter root. Verify at N; low prio.

## Verified classifications (this checkpoint)
- P73 amsldoc `\cn{\\*}`: rust=2, perl=2 (Perl errs differently: 2x "Expected a
  relational token"), pdflatex=0 -> SHARED, oracle-clean, in surpass scope.
- P74 `\abstract{…\makeatletter\patch@level…}`: rust=1, perl=1 (identical \patch
  undefined), pdflatex=0 -> SHARED, oracle-clean, in surpass scope.
- bibarts: rust=5, perl=7 (from seed) -> SHARED.

## Fix sites (to develop at Checkpoint N)
- P73: base_parameter_types.rs:567-591 (SanitizedVerbatim) / :622 recatcode
  reformed control symbols to OTHER; process_index_phrases do_expand_partially in
  latex_constructs.rs ~2875-2900.
- P74: latex_constructs.rs:5728-5740 — route braced `\abstract{` to the environment
  (incremental-read) path so inner \makeatletter takes effect.
- bibarts: the \protect\ifmmode…\fi family (\underline/\overline tex_math.rs:1283;
  \ba@textmode raw) torn by \edef across the write-mouth (P16(vi) / P29 \protect path).

---
# Checkpoint N #1 — P73 sanitizedverb_reforms_ctrlsym_amsldoc (ROOT)

repro: string-mouth/sanitizedverb_reforms_ctrlsym_amsldoc.tex (RED, 2 err:
\KV@tempa + readBalanced). Discriminator: \cn{\|} -> sort key `|` (no
backslash) CLEAN; \cn{\\*} -> sort key `\*` (literal catcode-12 backslash) RED.

## mechanism (ground truth: pdflatex + makeindex)
- amsldoc \cn (amsldoc.cls:114 \cs -> :84 \indexcs -> :87 \@indexcs) writes
  .idx: `\indexentry{\*@\verb"*+\\*+}{1}`. SORT KEY (before @) = literal `\*`
  string; DISPLAY (after @) = `\verb"*+\\*+`.
- makeindex .ind: `\item \verb*+\\*+, 1`. It DROPS the sort key entirely; the
  display is \verb-wrapped so `\*` is typeset verbatim, never executed.
  pdflatex clean (0 bangs, verified). => `\*` NEVER runs in real TeX: sort key
  dropped, display verbatim-protected.
- Rust divergence: base_parameter_types.rs:622 SanitizedVerbatim does
  `mouth::tokenize_internal(TeXString::assembled(writable_tokens(&arg)))`.
  begin_semiverbatim (:605) correctly makes `\` catcode-12 during the read
  (== \@sanitize, latex.ltx:1778 `\@makeother\\`), so the sort-key backslash
  is OTHER in `arg`. But writable_tokens flattens to a STRING and
  tokenize_internal re-reads with the DEFAULT catcode table (`\`=escape),
  UNDOING \@sanitize: catcode-12 `\`+`*` re-forms into the live CS `\*`.
  Then process_index_phrases:2907 `do_expand_partially` EXECUTES the whole
  entry incl. the sort key -> `\*` (amsldoc.cls:213) reads its arg off the end
  (under guit it reaches \setkeys -> \KV@tempa) -> readBalanced ran out.
- Corroboration: amsldoc_cls.rs:7-13 already documents this exact root
  ("SanitizedVerbatim untex->retokenize roundtrip welds its catcode-12 `\`
  into fake CSes") and patches \@nobslash at expansion time to work around it.

## classification
SHARED, oracle-clean -> in surpass scope. rust=2, perl=2 (Perl fails
identically: 2x "Expected a relational token"), pdflatex=0. (verified same-host)

## fix (primary — line 622, honor \@sanitize)
base_parameter_types.rs SanitizedVerbatim: the re-tokenization must preserve
the OTHER catcode of backslashes that were OTHER in `arg`. Segment the round
trip at catcode-12 backslashes: round-trip only runs with no catcode-12 `\`
(keeps §262 `\TIKZ@`->`\TIKZ`+`@` behavior for genuine CS tokens, KNOWN_PERL
_ERRORS #140), and emit each catcode-12 `\`(OTHER) directly as an OTHER token
so a \string'd control symbol never re-forms into an executable CS. Result:
sort key `\*` stays `\`(OTHER)`*`(OTHER) = makeindex sort string; not executed.
Genuine live-CS entries (macro arg #1=\TIKZ) unaffected (already CS, never
catcode-12). ALTERNATIVE (lower blast radius, less faithful): in process_index
_phrases before :2907, recatcode reformed control SYMBOLS (1-char CS whose name
is catcode-other) back to `\`+char OTHER pair.

## guard
0 Error/Fatal on the repro AND exactly one <ltx:indexmark> produced (RED today:
0 indexmarks — entry consumed by \*; clean \cn{\|} baseline: 1 indexmark).
No <ltx:ERROR> node. Reuse string-mouth/sanitizedverb_reforms_ctrlsym_amsldoc.tex.
Regressions to re-run: index_entry_expands_and_keys_sanitized_specials,
tcolorbox_doccommand_index_key_expands (P29), pgfornament \TIKZ §262 (#140),
amsldoc \@nobslash binding.

## expected corpus gain
2 docs. itamsldoc 3->0 (its 3 errors ARE this KV@tempa+2xreadBalanced).
amsldoc-vi 4->1 (removes KV@tempa+readBalanced; keeps \textdotbelow, separate root).

## dead ends
- \cn{\|}, \cn{\bslash}, \cn{notag}: sort key has no backslash -> clean (not this root).
- guit is not itself the trigger; it only routes the reformed \* into \setkeys/\KV@tempa.

---
# Checkpoint N #2 — bibarts \underline torn-conditional (ROOT)

repros: index/edef_write_ifmmode_bibarts.tex (bibarts, 5 err) + NEW
index/underline_robust_edef_kernel.tex (bibarts-free kernel, 3 err, pdflatex 0).

## mechanism
- \underline (tex_math.rs:1283) / \overline (tex_math.rs:1278) are PLAIN macros:
  `\protect\ifmmode\lx@math@underline{#1}\else\lx@text@underline{#1}\fi`.
  In an \edef where \protect=\@unexpandable@protect (=\noexpand\protect\noexpand,
  latex_base.rs:283), \protect freezes ONLY the next token \ifmmode; the
  \else..\fi tail is expanded by the \edef with no open conditional -> orphaned
  \else/\fi ("Didn't expect \else/\fi"; frozen \ifmmode later "fell off end" +
  readBalanced). Real latex.ltx:16369 defines \underline via \DeclareRobustCommand,
  so \protect\underline<space> freezes the ENTIRE \ifmmode..\fi body as one token.
- bibarts trigger: \@initaddtovli (bibarts.sty:2231) does
  \edef\@tempa{\write\@auxout{...\literentry{...#4...}}} where #4 (citation title)
  carries \underline{Publ.}. ISOLATED: \vli{...}{An \underline{Publ.}} = 1 err;
  \ktit-only (robust, frozen whole) = 0 err -> \underline is the SOLE root.
  bibarts.sty:798 \ba@textmode (`\ifmmode $..\fi`, bare) does NOT tear: \ktit is
  frozen, so \ba@textmode is never reached in the \edef.

## classification
SHARED, oracle-clean -> surpass in scope. Perl TeX_Math.pool.ltxml:989/991 has the
IDENTICAL non-robust body (fails the same); pdflatex 0. rust=5 (seed)/3 (kernel).

## fix
tex_math.rs:1277-1283 — add `protected => true` to the `\overline{}` and
`\underline{}` DefMacro!s (eTeX-robust; expandable.rs:201-203/349: protected
expandables aren't expanded under partial expansion/\edef, matching
\DeclareRobustCommand's frozen-in-moving-args effect). Optionally replace the body's
leading \protect with \relax to match latex.ltx:16371 (external flag now provides
robustness; \relax guards \ifmmode at alignment-cell starts). Normal math/text
digestion still expands them (protected macros expand in the main digest loop —
proven by the `~`->\lx@NBSP protected macro, plain_constructs.rs:441). Mirror in
Perl pool 989/991 optional (Rust-only surpass is fine).

## guard
index/underline_robust_edef_kernel.tex: 0 Error/Fatal AND output contains
<ltx:text class="ltx_underline"> with text "Publ." (proves \underline frozen through
the \edef then ran text-mode intact). Keep index/edef_write_ifmmode_bibarts.tex as
integration regression (0 err after fix). Re-run math underline/overline goldens +
alignment tests (\ifmmode-at-cell-start).

## risk / gain
MED. protected changes \underline/\overline to not expand in \edef/moving args/\show.
Gain: bibarts ba-short 5->0, bibarts 16->~0 (P16(vi): 14 lines/2 docs), + any doc
using \underline/\overline in a toc/mark/index moving arg.

## not this root
ribbonproofs: tears on \iffalse (pgf alignment-safe idiom in \edef\stuffToSave,
ribbonproofs.sty:1269/1277), no \underline/\overline. SEPARATE root.
