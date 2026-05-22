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
  // ≈ 8000000. We pick 650000 / 8000000 with a small safety margin over
  // the minimum-pass bracket. The gap to Perl's defaults suggests our
  // pushback / conditional accounting is roughly 1.1× Perl's (pushback)
  // and 2× Perl's (iflimit) on heavily macro-driven IEEEtran + tikz +
  // pgfplots input; root-causing those is tracked as a follow-up but
  // would let us tighten the constants back toward the Perl defaults.
  latexml_core::binding::content::pass_options("latexml", "sty", vec![
    s!("ids"),
    s!("rawstyles"),
    s!("bibconfig=bbl,bib"),
    s!("nobreakuntex"),
    s!("magnify=1.2"),
    s!("zoomout=1.2"),
    s!("tokenlimit=249999999"),
    s!("iflimit=8000000"),
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
