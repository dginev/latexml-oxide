use once_cell::sync::Lazy;

use crate::prelude::*;

// Process-once cached env vars (see WISDOM #56 — getenv hot-path race).
static INI_MODE: Lazy<bool> = Lazy::new(|| std::env::var_os("LATEXML_INI_MODE").is_some());
static NODUMP: Lazy<bool> = Lazy::new(|| std::env::var_os("LATEXML_NODUMP").is_some());

/// Perl: DefAutoload — define a macro that auto-loads a package on first use.
/// When the command is first invoked, it loads the specified package (via
/// RequirePackage), then re-emits the original CS so it gets re-executed
/// with the proper definition.
///
/// Mirrors Perl `Package.pm` `DefAutoload` (L1081-1100) + `ClearAutoLoad`
/// (L1106-1111): clear the trigger's meaning GLOBALLY before loading, then
/// run the package, then re-emit the original CS. Without the global clear,
/// if the autoload fires from inside a `\begin{X}` group whose env-CS is
/// itself the trigger, the package's bindings install at LOCAL scope on
/// the env's frame and get popped on `\end{X}` — reverting the trigger CS
/// to the autoload closure and re-firing on every subsequent invocation
/// (1.6M+ infinite locked-redefinition loop, observed in 1711.11576 etc).
/// Clearing globally short-circuits the loop: after first fire, the
/// trigger is `Stored::None` globally, so subsequent invocations with no
/// in-scope local binding hit a clean undefined-CS error rather than
/// re-entering the autoload.
///
/// Sandbox math0004154 + amsppt class-route hang at amsfonts.sty
/// (see project_amsppt_cls_dispatcher.md memory) — the original re-entry
/// guard was a `Tokens(vec![])` no-op installed at LOCAL scope before the
/// package load. The Perl-faithful global clear (Stored::None at Global)
/// covers the same re-entry-during-package-load case AND the
/// autoload-from-inside-a-group case.
fn def_autoload(cs_name: &str, package: &str) -> Result<()> {
  use latexml_core::{common::store::Stored, definition::ExpansionBody};
  let cs_tok = T_CS!(cs_name);
  // Don't overwrite if already defined
  if IsDefined!(&cs_tok) {
    return Ok(());
  }
  let pkg_name = package.to_string();
  let cs_for_closure = cs_tok;
  def_macro(
    cs_tok,
    None,
    ExpansionBody::Closure(Rc::new(move |_args| {
      // If the target package is *already* loaded, this closure is firing
      // because a separate CS was \let to the autoload trigger BEFORE the
      // package later loaded under a different name (e.g. `\let\varmathbb
      // =\mathbb` while \mathbb is still the autoload trigger; then
      // amssymb → amsfonts loads and \mathbb gets a real def, but
      // \varmathbb still holds the closure). In that case, clearing
      // cs_for_closure (\mathbb) would erase its real definition, and
      // re-emitting \mathbb would land on an undefined CS — producing the
      // spurious `Error:undefined:\mathbb` reported during `\varmathbb{D}`
      // expansion. Witness 2310.13684. Skip the clear+load and just
      // re-emit cs_for_closure, which by now has the real meaning the
      // already-loaded package installed.
      let pkg_loaded_key = s!("{}.sty_loaded", pkg_name);
      let pkg_raw_key = s!("{}.sty_raw_loaded", pkg_name);
      if lookup_bool(&pkg_loaded_key) || lookup_bool(&pkg_raw_key) {
        return Ok(Tokens::new(vec![cs_for_closure]));
      }
      // Perl `ClearAutoLoad` — assign_internal('meaning', $trigger => undef,
      // 'global'). Removes the autoload trigger globally.
      assign_meaning(&cs_for_closure, Stored::None, Some(Scope::Global));
      // Snapshot the calling frame's MEANING keys before the package load.
      // Anything new the package installs at the calling frame is then
      // hoisted to GLOBAL so it survives `\end{X}`'s pop_frame.
      //
      // Without this, autoload-triggered package loads from inside a body
      // group (e.g. amsmath autoload via `\subequations` inside an
      // environment group at depth>=1) install ALL of the package's CSes
      // at the calling frame; those CSes vanish when the group exits, so
      // a sibling autoload trigger (e.g. `\align` later in the body)
      // fires, finds no real binding, and clears+re-emits → undefined CS
      // → 10000-error MaxLimit cascade. Observed in 1711.11576 + cluster.
      //
      // The single-trigger `let_i(\trigger, \trigger, Global)` from the
      // earlier round-17 fix is subsumed by the delta-hoist (since the
      // trigger CS is itself a meaning-key newly-installed by the package
      // load). (1711.11576: cleanly converts post-fix.)
      let pre_keys = snapshot_top_frame_meaning_keys();
      require_package(&pkg_name, RequireOptions::default())?;
      hoist_top_frame_meaning_delta(&pre_keys);
      Ok(Tokens::new(vec![cs_for_closure]))
    })),
    None,
  )?;
  // Mark this CS as an autoload trigger so `isDefinableLaTeX` treats
  // `\newcommand{\cs}{…}` as a redefinition of an undefined CS (matching
  // Perl, where the equivalent `DefAutoload` entries live in
  // `OmniBus.cls.ltxml` and only fire when OmniBus is actually loaded —
  // i.e. they don't block user `\newcommand` in normal LaTeX papers).
  // Witness: nucl-th9902037 redefines `\Bbb` via `\newcommand` to a
  // paper-local symbol; without this flag the autoload trigger silently
  // wins, then expands `\Bbb $arg…$` as `\mathbb{$}` and cascades into
  // 62 mode-switch errors. Perl on the same input: 0 errors.
  //
  // Store the PACKAGE NAME (not just a bool) so readers can distinguish an
  // UNFIRED trigger from one whose package has since loaded. Once `<package>`
  // is loaded — explicitly via `\usepackage` (which redefines the trigger CS
  // to the real macro WITHOUT firing this closure) or via the closure firing —
  // the CS is genuinely defined, and `\lx@ifundefined` must report it as such.
  // Witness: `\@ifundefined{align}` after `\usepackage{amsmath}` wrongly
  // returned "undefined" (the stale flag masked the real `\align`), breaking
  // extract.sty's `\begin ` env-existence probe → 90-error cascade on
  // 1611.02736. `.pool` triggers keep the bool form (no `<pkg>.sty_loaded`).
  assign_value(
    &s!("{cs_name}:autoload"),
    Stored::String(pin(package)),
    Some(Scope::Global),
  );
  Ok(())
}

/// Variant of `def_autoload` that loads a `.pool` (engine-level
/// definitions file) instead of a `.sty` package. Mirrors Perl's
/// `DefAutoload($trigger, '<Pool>.pool.ltxml')` form used by TeX.pool
/// to lazy-load AmSTeX.pool on amstex-style triggers.
fn def_autoload_pool(cs_name: &str, pool: &str) -> Result<()> {
  use latexml_core::{common::store::Stored, definition::ExpansionBody};
  let cs_tok = T_CS!(cs_name);
  if IsDefined!(&cs_tok) {
    return Ok(());
  }
  let pool_name = pool.to_string();
  let cs_for_closure = cs_tok;
  def_macro(
    cs_tok,
    None,
    ExpansionBody::Closure(Rc::new(move |_args| {
      assign_meaning(&cs_for_closure, Stored::None, Some(Scope::Global));
      let pre_keys = snapshot_top_frame_meaning_keys();
      input_definitions(&pool_name, InputDefinitionOptions {
        extension: Some(Cow::Borrowed("pool")),
        ..InputDefinitionOptions::default()
      })?;
      hoist_top_frame_meaning_delta(&pre_keys);
      Ok(Tokens::new(vec![cs_for_closure]))
    })),
    None,
  )?;
  assign_value(
    &s!("{cs_name}:autoload"),
    Stored::Bool(true),
    Some(Scope::Global),
  );
  Ok(())
}

/// Perl `FindFile($format._dump, ...)` parity for the plain dump.
/// Delegates to [`crate::plain_dump::plain_dump_available`], which
/// consults env overrides, the exe-relative install layout, the
/// dev-tree path, and the embedded fallback.
fn plain_dump_available() -> bool { crate::plain_dump::plain_dump_available() }

LoadDefinitions!({
  // port of TeX.pool.ltxml
  // commit 4cd73e7584c5f0422293ba38f9b757332584afec
  // Author: Bruce Miller <nebconinc@gmail.com>
  // Date:   Thu May 9 13:19:32 2024 -0400

  // Perl TeX.pool.ltxml L22: LoadPool('Base'); — loads the entire
  // Base subsystem (schemas, parameter types, utilities, XMath,
  // TeX_*, eTeX, pdfTeX, Base_Deprecated). Rust mirrors this with
  // a single call to the `base` LoadDefinitions module so the same
  // pool is reusable from `ini_tex.rs` for dump-build (which
  // needs ONLY Base.pool, not the autoload triggers below).
  InnerPool!(base);

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Pre-LoadFormat content. In Perl TeX.pool.ltxml these are AFTER
  // `LoadFormat('plain')` (lines 33-65), but Rust must define them
  // BEFORE the dump-write snapshot — otherwise they pollute
  // plain.dump.txt with autoload triggers that Perl's
  // plain_dump.pool.ltxml does NOT contain. Functionally equivalent:
  // by engine init's end, all these are in state regardless of order.
  //======================================================================

  Let!("\\protect", "\\relax");

  // Perl TeX.pool.ltxml L33-56: autoload triggers for LaTeX, expl3, AmSTeX.
  for ltxtrigger in [
    "\\documentclass",
    "\\newcommand",
    "\\renewcommand",
    "\\newenvironment",
    "\\renewenvironment",
    "\\NeedsTeXFormat",
    "\\ProvidesPackage",
    "\\RequirePackage",
    "\\ProvidesFile",
    "\\makeatletter",
    "\\makeatother",
    "\\begin",
    "\\listfiles",
    "\\nofiles",
    "\\typeout",
    "\\PassOptionsToPackage",
    // \UseRawInputEncoding is a LaTeX kernel command defined by latex.ltx
    // (L18268-18324). Some papers invoke it on line 1 col 1, BEFORE
    // \documentclass — e.g. as a UTF-8 / fontenc shield. Treat it as a
    // LaTeX-pool trigger so the kernel binds load and its `\relax` stub
    // (latex_constructs_rust_only.rs L59) is in place. Witness 2403.19280.
    "\\UseRawInputEncoding",
    // \DocumentMetadata{...} is the LaTeX 2024 kernel command for PDF
    // accessibility metadata; LaTeX expects it BEFORE \documentclass.
    // Same pre-documentclass autoload pattern as \UseRawInputEncoding.
    // Witness 2305.08034.
    "\\DocumentMetadata",
  ]
  .iter()
  {
    DefMacro!(T_CS!(ltxtrigger), None, {
      Tokens!(T_CS!("\\@load@latex@pool"), T_CS!(ltxtrigger))
    });
  }
  DefPrimitive!("\\@load@latex@pool", {
    input_definitions("LaTeX", InputDefinitionOptions {
      extension: Some(Cow::Borrowed("pool")),
      ..InputDefinitionOptions::default()
    })?;
    // Restore our `\documentstyle` impl after the LaTeX pool load. The
    // latex_dump unconditionally redefines `\documentstyle` to the
    // kernel-style `\input{latex209.def}\documentclass`; in Perl that's
    // overridden by latex_constructs.pool.ltxml's DefConstructor, but
    // our latex_constructs.rs port doesn't redefine it. Re-Let to our
    // backup to win. See tex_job.rs `\lx@documentstyle@impl` for the
    // full diagnostic context (hep-th9912229 witness).
    Let!("\\documentstyle", "\\lx@documentstyle@impl");
  });

  // Perl TeX.pool.ltxml L42-48: expl3 triggers
  for ltx3trigger in [
    "\\ExplSyntaxOn",
    "\\ProvidesExplClass",
    "\\ProvidesExplPackage",
  ] {
    def_autoload(ltx3trigger, "expl3")?;
  }

  // OmniBus autoloads (Perl: OmniBus.cls.ltxml DefAutoload entries)
  def_autoload("\\mathfrak", "amsfonts")?;
  def_autoload("\\mathbb", "amsfonts")?;
  def_autoload("\\Bbb", "amsfonts")?;
  def_autoload("\\theoremstyle", "amsthm")?;
  def_autoload("\\numberwithin", "amsmath")?;
  def_autoload("\\align", "amsmath")?;
  def_autoload("\\subequations", "amsmath")?;
  def_autoload("\\multline", "amsmath")?;
  def_autoload("\\curraddr", "ams_support")?;
  def_autoload("\\subjclass", "ams_support")?;

  // LaTeX2HTML-era papers use the html.sty CSes without an explicit
  // `\usepackage{html}` because in the original LaTeX2HTML toolchain
  // those CSes were part of the implicit "html-on-load" expectation.
  // Auto-load `html.sty` (our binding maps these to hyperref equivalents)
  // when any of these triggers is referenced. Driver:
  // arxiv-examples/2108.04969 (uses `\htmladdnormallink{...}{...}` without
  // loading html.sty).
  def_autoload("\\htmladdnormallink", "html")?;
  def_autoload("\\htmladdnormallinkfoot", "html")?;
  def_autoload("\\htmladdimg", "html")?;
  def_autoload("\\latextohtml", "html")?;
  def_autoload("\\externalref", "html")?;
  def_autoload("\\externalcite", "html")?;
  def_autoload("\\htmlref", "html")?;
  def_autoload("\\htmlurl", "html")?;
  def_autoload("\\latexonly", "html")?;

  // Perl TeX.pool.ltxml L50-56: AmSTeX-pool autoload triggers. When any
  // of these CSes is invoked in plain-TeX-with-AmSTeX style (or before
  // `\documentstyle{amsppt}` arrives — see e.g. math/9610224's
  // `\NoBlackBoxes\documentstyle{amsppt}` ordering), Perl auto-loads
  // `AmSTeX.pool.ltxml` first. Without this, our Rust port emitted
  // `Error:undefined:\NoBlackBoxes` because the AmSTeX pool hadn't
  // been loaded yet.
  for amstrigger in [
    "\\BlackBoxes",
    "\\NoBlackBoxes",
    "\\TagsAsMath",
    "\\TagsAsText",
    "\\TagsOnLeft",
    "\\TagsOnRight",
    "\\CenteredTagsOnSplits",
    "\\TopOrBottomTagsOnSplits",
    "\\LimitsOnInts",
    "\\NoLimitsOnInts",
    "\\LimitsOnNames",
    "\\NoLimitsOnNames",
    "\\LimitsOnSums",
    "\\NoLimitsOnSums",
    "\\loadbold",
    "\\loadeufb",
    "\\loadeufm",
    "\\loadeurb",
    "\\loadeurm",
    "\\loadeusb",
    "\\loadeusm",
    "\\loadmathfont",
    "\\loadmsam",
    "\\loadmsbm",
  ] {
    def_autoload_pool(amstrigger, "AmSTeX")?;
  }

  // File bookkeeping (Perl TeX.pool.ltxml, needed before LaTeX.pool loads)
  DefMacro!(
    "\\@pushfilename",
    r"\xdef\@currnamestack{{\@currname}{\@currext}{\the\catcode`\@}\@currnamestack}"
  );
  DefMacro!(
    "\\@popfilename",
    r"\expandafter\@p@pfilename\@currnamestack\@nil"
  );
  DefMacro!(
    "\\@p@pfilename {}{}{} Until:\\@nil",
    r"\gdef\@currname{#1}%
      \gdef\@currext{#2}%
      \catcode`\@#3\relax
      \gdef\@currnamestack{#4}"
  );
  DefMacro!(T_CS!("\\@currnamestack"), None, Tokens!());
  Let!("\\@currname", "\\lx@empty");
  Let!("\\@currext", "\\lx@empty");

  // Early stubs needed by ProcessOptions/DeclareOption before LaTeX.pool loads
  if !IsDefined!(&T_CS!("\\@unknownoptionerror")) {
    DefPrimitive!("\\@unknownoptionerror", {});
  }
  if !IsDefined!(&T_CS!("\\OptionNotUsed")) {
    DefPrimitive!("\\OptionNotUsed", {});
  }
  if !IsDefined!(&T_CS!("\\AtBeginDocument")) {
    def_macro_noop("\\AtBeginDocument{}")?;
  }
  if !IsDefined!(&T_CS!("\\@addtofilelist")) {
    def_macro_noop("\\@addtofilelist{}")?;
  }

  // Perl: LoadFormat('plain') — Package.pm L2734-2752. Mutually
  // exclusive dump-or-base path (see strict-Perl LoadFormat parity).
  InnerPool!(plain_bootstrap); // Perl: plain_bootstrap.pool.ltxml

  // In `--init=plain.tex` (dump-build) mode, stop after plain_bootstrap.
  // The whole point is to take a snapshot at this point, then digest raw
  // plain.tex against it — pre-loading plain_dump / plain_base /
  // plain_constructs would pollute the snapshot and silence the diff for
  // every register/macro plain.tex defines (e.g. `\countdef\allocationnumber=21`
  // → `Stored::Register{address:"\count21"}` then identical to itself).
  // `LATEXML_INI_MODE=1` is set by `bin/latexml_oxide.rs` BEFORE
  // `prepare_session`, so this branch fires before tex.rs runs in init mode.
  // Mirrors Perl `Core.pm::iniTeX` default `mode='Base'`, which loads only
  // `Base.pool` (no LoadFormat) before `DumpFile`.
  if !*INI_MODE {
    // Perl `LoadFormat('plain')` strict split (mirrors latex.rs):
    //   if dump available: bootstrap → dump → constructs (NO base)
    //   else:              bootstrap → base → constructs (NO dump)
    if !*NODUMP && plain_dump_available() {
      InnerPool!(plain_dump); // runtime loader for resources/dumps/plain.dump.txt
    } else {
      // Degraded raw-load path — surface the missing dump loudly (shared
      // one-shot banner with latex.rs). See dump_paths::warn_degraded_no_dump.
      if !*NODUMP {
        crate::dump_paths::warn_degraded_no_dump();
      }
      InnerPool!(plain_base); // Perl: plain_base.pool.ltxml
    }
    InnerPool!(plain_constructs); // Perl: plain_constructs.pool.ltxml → math_common

    // Symmetric with latex.rs: any PA/MPA let-aliases whose target wasn't
    // defined at dump-load time were queued; flush them now that
    // plain_constructs has run.
    let (applied, skipped) = dump_reader::flush_deferred_aliases();
    if applied + skipped > 0 {
      Info!(
        "plain_dump",
        "deferred",
        s!("deferred aliases: {} applied, {} skipped", applied, skipped)
      );
    }
  }

  //======================================================================
  // After all other rewrites have acted, a little cleanup
  // [This suggests that it should be (one of) the LAST (math) rewrite applied?
  // Do we need to define it last?]
  // DefRewrite(xpath => 'descendant-or-self::ltx:XMWrap[count(child::*)=1]',
  //   replace => sub { my ($document, $wrap) = @_;
  //     if (my $node = $document->getFirstChildElement($wrap)) {
  //       # Copy attributes but NOT internal ones,
  //       # NOR xml:id, else we get clashes
  //       foreach my $attribute ($wrap->attributes) {
  //         if ($attribute->nodeType == XML_ATTRIBUTE_NODE) {
  //           my $attr = $document->getNodeQName($attribute);
  //           $document->setAttribute($node, $attr => $attribute->getValue)
  //             unless ($attr eq 'xml:id') || $attr =~ /^_/;
  //           if    ($attr =~ /^_/) { }
  //           elsif ($attr eq 'xml:id') {
  //             my $id = $attribute->getValue;
  //             if (my $previd = $node->getAttribute('xml:id')) {    # Keep original id
  //                   # but swap any references to the one on the wrapper!
  //               foreach my $ref ($document->findnodes("//*[\@idref='$id']")) {
  //                 $ref->setAttribute(idref => $previd); }
  //               $wrap->removeAttribute('xml"id');
  //               $document->unRecordID($id); }
  //             else {
  //               $wrap->removeAttribute('xml:id');
  //               $document->unRecordID($id);
  //               $document->setAttribute($node, 'xml:id' => $id); } }
  //           else {
  //             $document->setAttribute($node, $attr => $attribute->getValue); } } }
  //       # But keep $node's font from being overwritten.
  //       $document->setNodeFont($wrap, $document->getNodeFont($node));
  //       ## WHY THIS????
  //       $document->getNode->appendChild($node);
  // } });
});
