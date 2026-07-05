use latexml_package::prelude::*;

LoadDefinitions!({
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
  // still caught well before the worker wall-clock lease (≈350k cond/s ⇒
  // 16M in ≈46 s ≪ the 180 s lease) and by the RSS fuse. **iflimit raised
  // 8M → 16M (4× Perl's 3,999,999)** to recover coverage on the ≈17-paper
  // `\tikz@dashphase` Timeout cluster that Perl cannot convert at all
  // (Perl chokes on these papers' expl3 first). Pre-approved 2026-06-30.
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
  pass_options("latexml", "sty", vec![
    s!("ids"),
    s!("rawstyles"),
    s!("bibconfig=bbl,bib"),
    s!("nobreakuntex"),
    s!("magnify=1.2"),
    s!("zoomout=1.2"),
    s!("tokenlimit=999999999"),
    s!("iflimit=16000000"),
    s!("absorblimit=1299999"),
    s!("pushbacklimit=650000"),
  ])?;
  RequirePackage!("latexml");

  // Practical maximum for warnings
  AssignValue!("MAX_WARNINGS" => 10000i64, Scope::Global);

  // No \today in archival conversions. Perl L23-25:
  //   AtBeginDocument(sub {
  //     DefMacroI('\today', undef, '\relax', locked => 1, scope => 'global');
  //   });
  // We bind at load time with `locked => true, Scope::Global` instead of
  // wrapping in `\AtBeginDocument{\def\today{\relax}}` (which loses both
  // flags — `\def` is plain-TeX, with no LaTeXML lock). The lock makes
  // timing irrelevant: any preamble package that tries to (re)define
  // \today after this point is silently rejected, matching the intent of
  // Perl's AtBeginDocument hook (defer until all packages have loaded).
  DefMacro!("\\today", "\\relax", locked => true, scope => Some(Scope::Global));

  // Perl L30-35: drop all non-remote <ltx:resource> nodes (keep only `http*`
  // src so the archival run doesn't embed default local CSS / JS).
  //   DefRewrite(xpath => 'descendant-or-self::ltx:resource', replace => sub {
  //     my ($self, $node) = @_;
  //     my $src = $node->getAttribute('src') || '';
  //     return if $src !~ /^http/;      # non-remote → silently drop
  //     $self->getNode->appendChild($node); });   # remote → re-attach
  DefRewrite!(xpath => "descendant-or-self::ltx:resource",
  replace => sub[document, nodes] {
    let node = nodes.pop().unwrap();
    let src = node.get_attribute("src").unwrap_or_default();
    if src.starts_with("http") {
      document.get_node_mut().add_child(node)?;
    }
  });

  // NOTE: Perl additionally monkey-patches LaTeXML::Post::MathML::outerWrapper
  // to set intent=':literal' on the top-level math element. That is a
  // post-processing hook (not a compile-time binding), tracked separately —
  // we do not emulate it here. See Perl source L45-73 for context.
});
