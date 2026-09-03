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
