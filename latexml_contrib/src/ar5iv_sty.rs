use latexml_package::prelude::*;

LoadDefinitions!({
  // An archival conversion never stamps the conversion's day. Perl L23-25 makes `\today` print nothing:
  //   AtBeginDocument(sub {
  //     DefMacroI('\today', undef, '\relax', locked => 1, scope => 'global');
  //   });
  // Beyond Perl (user ruling 2026-10-10): the job's clock is the paper's own date instead — the newest modification
  // time among the TeX source files of its bundle (`latexml::source_date`) — so LaTeX's and babel's `\today` print
  // the date arXiv's PDF shows, and a class parsing `\today` gets a date: ptapap.cls:322-325 splits `\edef`'d `\today`
  // with `\def\next#1#2#3#4\relax`, which on `\relax` ran away to "Paragraph ended before \next was complete" (run
  // 336, 1801.05985); 1811.05851's `\item[] \today` prints "November 14, 2018". Only the bundle's date counts, never
  // `SOURCE_DATE_EPOCH` (exported by every Nix shell and Debian build, often as 1980-01-01): an undated bundle keeps
  // Perl's `\relax` whatever that variable says, which then only seeds tex_job's registers. `\pdfcreationdate` reads
  // the same registers. The clock is set before the format loads (`RequirePackage!("latexml")` below): the l3kernel
  // copies it into `\c_sys_year_int` &c. at the job's start (`\g__sys_everyjob_tl`), which must agree with `\today`.
  // (Perl's begin-document `\relax` leaves the preamble reading the conversion's day: its ptapap papers print the
  // conversion's month.)
  let archival_date =
    source_date_epoch().is_some_and(latexml_engine::tex_job::assign_date_registers);

  // Perl: PassOptions('latexml', 'sty', ...) + RequirePackage('latexml')
  // Mirror Perl ar5iv.sty.ltxml: pass `rawstyles` (INCLUDE_STYLES => true,
  // kpsewhich enabled, system-wide texmf reachable). Earlier the Rust
  // port passed `localrawstyles` (kpsewhich suppressed, only paper-local
  // SEARCHPATHS) per past user direction (commit 9869267eb), but that
  // diverged from the Perl ar5iv profile and caused real parity gaps:
  // papers using system-installed-but-unbound .sty packages
  // (colonequals, comment, gnuplot, …) loaded fine in Perl but errored
  // in Rust with `\<missing-cs> undefined`. Switch back to `rawstyles`
  // for Perl-baseline parity (cf. feedback_sandbox_perl_baseline.md).
  // Perl ar5iv.sty.ltxml ships `pushbacklimit=599999, iflimit=3999999`
  // but the Rust port hits both ceilings on real ar5iv-profile papers
  // (witness arXiv:2605.16752v1). Empirical bisect (2026-05-22) on that
  // witness pinned the actual minima at: pushback ≈ 630000, iflimit
  // ≈ 8000000.
  //
  // ROOT-CAUSED 2026-06-30: the iflimit gap to Perl is NOT a bug to
  // "tighten away" — it is *deliberate, more-comprehensive runaway
  // counting*. Rust defines `\ifx`/`\ifcsname` (and the other low-level
  // TeX conditionals) as real `DefConditional`s that increment the global
  // `if_count` runaway guard; Perl does NOT count `\ifx`/`\ifcsname` toward
  // its `if_count` at all. On pgfkeys-driven tikz/pgfplots input (both
  // engines raw-load the real `pgfkeys.code.tex` — Perl's native pgfkeys
  // override is `__END__`-disabled), the gap is enormous: a controlled
  // 2-plot pgfplots figure that BOTH engines render identically (≈86 vs 87
  // graphic nodes) counts **148,078** conditionals in Rust vs **<200** in
  // Perl (≈740×), dominated by `\ifx` (63%) + `\ifcsname` (15%) inside the
  // pgfkeys key-dispatch. Counting these is *correct* — it is exactly what
  // the guard is for (a `\ifx`/`\ifcsname` runaway is invisible to Perl's
  // counter). So the right response is to raise the limit, not to count
  // less. Real *finite* heavy docs (multi-figure pgfplots papers) measure
  // ≈10–15M conditionals and complete in ≈24–43 s; a genuine runaway is
  // was then caught well before the worker's per-document timeout (at 16M:
  // ≈350k cond/s ⇒ ≈46 s ≪ the 180 s `--timeout`; the dispatcher lease is
  // 240 s) and by the RSS fuse. **iflimit raised
  // 8M → 16M (4× Perl's 3,999,999)** to recover coverage on the ≈17-paper
  // `\tikz@dashphase` Timeout cluster that Perl cannot convert at all
  // (Perl chokes on these papers' expl3 first). Pre-approved 2026-06-30.
  // **Raised 16M → 48M (62w)**: pgfplots/mhchem papers measured (release
  // build) 19M (2609.10563, 48 s), 21M (2609.16075, 45 s) and 39M (2609.07725,
  // 78 s) and complete without the limit, as do 2605.27177 (56 s) and 2605.04377
  // (98 s). Any runaway still stops at the 180 s timeout; the limit only ends a
  // faster one sooner, with the `IfLimit` label: one between ≈89k and ≈267k
  // cond/s (16M and 48M over 180 s) now ends at the timeout instead — ≈140
  // IfLimit papers per full pass, so the fleet time is negligible. 2609.30783 (a
  // memory runaway in lirseg) now ends at the box-list memory budget
  // (`Fatal:Stomach:MemoryBudget`, which rides the memory cap; was `IfLimit`).
  //
  // pushbacklimit RAISED 650000 → 5000000 (the binary's own default,
  // latexml_oxide.rs, and TeX's `main_memory`, one word per token,
  // texmf.cnf:820): a macro expansion copies its whole body into the flat
  // pushback (gullet.rs `unread_expansion`), where TeX pushes a pointer to the
  // token list (tex.web §323 `begin_token_list`), so a finite long list trips a
  // limit TeX has no counterpart for. pgf expands a whole soft path at once
  // (pgfsyssoftpath.code.tex:66-75, 94-98, 122-131): a 9000-sample smooth plot
  // is ≈650K tokens (tkz-grapheur's plots have 11,000). Perl with ar5iv stops on
  // that plot too (`Fatal:timeout:pushback_limit 599999`, Gullet.pm:300-306),
  // but not on `\edef\a{\a\a}` ×17, since its `readXToken`/`readBalanced`
  // never check the limit. The limits are pragmatic heuristics that let a large
  // manuscript convert (user ruling 2026-09-26); the cost is that a pushback
  // runaway stops later (a pgfmath-per-iteration loop: 2.3 s → 14.9 s), still
  // bounded by this limit and the timeout.
  //
  // tokenlimit RECALIBRATED 2026-06-10 (PR #249 review P1-2): the gullet
  // read checkpoints now count in all three reader loops (was: read_token
  // only), so the old 249999999 — calibrated under the old accounting —
  // silently shrank by the multi-counting factor. Measured heaviest
  // known-good ar5iv-profile paper under the new accounting: math0402448 at
  // 80.2M (`Info:gullet:progress`). 999999999 keeps the canvas profile's
  // generous backstop posture (runaways are cut much earlier by the cycle
  // guards / pushbacklimit / byte budget; the tokenlimit only bounds
  // aperiodic grind).
  // `localrawclasses` (latexml.sty's option; passing it here is Rust-only, OXIDIZED_DESIGN_DIVERGENCES #444): a class
  // shipped with the paper and covered by no binding is interpreted raw, as TeX would, instead of falling to OmniBus's
  // guesses (webofc 1301.7514, pasj00 0704.3654, raa 0802.3215, cms-tdr 1010.5994). Run 329: ~35% of the erroring
  // papers ran on OmniBus over a class they ship (webofc, epl2, jpconf, iaus, pasj, raa, cms-tdr, …), not in TeX
  // Live; their journal macros (`\pagerange`, `\Name`, `\ack`, `\KeyWords`, …) were undefined. A class in TeX Live
  // without a binding stays on OmniBus.
  pass_options("latexml", "sty", vec![
    s!("ids"),
    s!("rawstyles"),
    s!("localrawclasses"),
    s!("bibconfig=bbl,bib"),
    s!("nobreakuntex"),
    s!("magnify=1.2"),
    s!("zoomout=1.2"),
    s!("tokenlimit=999999999"),
    s!("iflimit=48000000"),
    s!("absorblimit=1299999"),
    s!("pushbacklimit=5000000"),
  ])?;
  RequirePackage!("latexml");

  // Practical maximum for warnings
  AssignValue!("MAX_WARNINGS" => 10000i64, Scope::Global);

  // No date in the source (stdin, a single-file submission's undated entry): Perl's `\relax`.
  // We bind at load time with `locked => true, Scope::Global` instead of
  // wrapping in `\AtBeginDocument{\def\today{\relax}}` (which loses both
  // flags — `\def` is plain-TeX, with no LaTeXML lock). The lock rejects a
  // preamble package's later (re)definition of \today, matching the intent of
  // Perl's AtBeginDocument hook (defer until all packages have loaded). It
  // does NOT make timing irrelevant: binding loads run unlocked
  // (content.rs, state.rs), so the LaTeX format's own `\today`
  // (latex_constructs sect03.rs) would replace this one if it loaded later.
  // It loads earlier today, inside the latexml.sty preload (its body's
  // `\AddToHook` autoloads LaTeX.pool); a change that defers the format load
  // to `\documentclass` must move this to begin-document, as Perl does
  // (repro loader/latexml_preload_keeps_plain.tex).
  if !archival_date {
    DefMacro!("\\today", "\\relax", locked => true, scope => Some(Scope::Global));
  }

  // Perl L30-35: drop all non-remote <ltx:resource> nodes (keep only `http*`
  // src so the archival run doesn't embed default local CSS / JS).
  //   DefRewrite(xpath => 'descendant-or-self::ltx:resource', replace => sub {
  //     my ($self, $node) = @_;
  //     my $src = $node->getAttribute('src') || '';
  //     return if $src !~ /^http/;      # non-remote → silently drop
  //     $self->getNode->appendChild($node); });   # remote → re-attach
  // Beyond Perl: a style sheet given inline, with no file (`\lxRequireResource[type=text/css,content={…}]{}`), is
  // kept too: it embeds nothing local, and is how a paper styles its own HTML, the arXiv source having no room for a
  // CSS file (2606.12996's rules for its nested tables and appendix TOC entries); so is a class option's own rules
  // (article's `openbib`, `.ltx_bibblock{display:block;}`). Inline scripts stay dropped.
  DefRewrite!(xpath => "descendant-or-self::ltx:resource",
  replace => sub[document, nodes] {
    let node = nodes.pop().unwrap();
    let src = node.get_attribute("src").unwrap_or_default();
    let inline_css = src.is_empty()
      && node.get_attribute("type").as_deref() == Some("text/css")
      && !node.get_content().trim().is_empty();
    if src.starts_with("http") || inline_css {
      document.get_node_mut().add_child(node)?;
    }
  });

  // NOTE: Perl additionally monkey-patches LaTeXML::Post::MathML::outerWrapper
  // to set intent=':literal' on the top-level math element. That is a
  // post-processing hook (not a compile-time binding), tracked separately —
  // we do not emulate it here. See Perl source L45-73 for context.
});
