use crate::prelude::*;

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
  use latexml_core::common::store::Stored;
  use latexml_core::definition::ExpansionBody;
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
      // Perl `ClearAutoLoad` — assign_internal('meaning', $trigger => undef,
      // 'global'). Removes the autoload trigger globally.
      state::assign_meaning(&cs_for_closure, Stored::None, Some(Scope::Global));
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
      let pre_keys = latexml_core::state::snapshot_top_frame_meaning_keys();
      require_package(&pkg_name, RequireOptions::default())?;
      latexml_core::state::hoist_top_frame_meaning_delta(&pre_keys);
      Ok(Tokens::new(vec![cs_for_closure]))
    })),
    None,
  )?;
  Ok(())
}

/// Perl `FindFile($format._dump, ...)` parity for the plain dump.
/// Returns `true` if `plain.dump.txt` is reachable through any of the
/// runtime resolution paths (env overrides, exe-relative install layout,
/// dev-tree). Mirrors `plain_dump::resolve_dump_path` exactly so the
/// branch decision in `LoadFormat('plain')` doesn't drift.
fn plain_dump_available() -> bool {
  if let Ok(p) = std::env::var("LATEXML_PLAIN_DUMP_PATH") {
    if std::path::Path::new(&p).is_file() {
      return true;
    }
  }
  if let Ok(dir) = std::env::var("LATEXML_DUMP_DIR") {
    if std::path::Path::new(&dir).join("plain.dump.txt").is_file() {
      return true;
    }
  }
  if let Ok(exe) = std::env::current_exe() {
    if let Some(exe_dir) = exe.parent() {
      if exe_dir.join("../resources/dumps/plain.dump.txt").is_file() {
        return true;
      }
      if exe_dir.join("plain.dump.txt").is_file() {
        return true;
      }
    }
  }
  let dev = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../resources/dumps/plain.dump.txt"
  );
  std::path::Path::new(dev).is_file()
}

LoadDefinitions!({
  // port of TeX.pool.ltxml
  // commit 4cd73e7584c5f0422293ba38f9b757332584afec
  // Author: Bruce Miller <nebconinc@gmail.com>
  // Date:   Thu May 9 13:19:32 2024 -0400
  InnerPool!(base_schema);
  InnerPool!(base_parameter_types);
  InnerPool!(base_utilities);
  InnerPool!(base_xmath);
  InnerPool!(tex_box);
  InnerPool!(tex_character);
  InnerPool!(tex_debugging);
  InnerPool!(tex_file_io);
  InnerPool!(tex_fonts);
  InnerPool!(tex_glue);
  InnerPool!(tex_hyphenation);
  InnerPool!(tex_inserts);
  InnerPool!(tex_job);
  InnerPool!(tex_kern);
  InnerPool!(tex_logic);
  InnerPool!(tex_macro);
  InnerPool!(tex_marks);
  InnerPool!(tex_math);
  // tex_scripts content now inlined in tex_math.rs (Perl: TeX_Math.pool.ltxml)
  InnerPool!(tex_page);
  InnerPool!(tex_paragraph);
  InnerPool!(tex_penalties);
  InnerPool!(tex_registers);
  InnerPool!(tex_tables);
  InnerPool!(etex); // unless... ?
  InnerPool!(pdftex); // unless... ?

  InnerPool!(base_deprecated);

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
    DefMacro!("\\AtBeginDocument{}", "");
  }
  if !IsDefined!(&T_CS!("\\@addtofilelist")) {
    DefMacro!("\\@addtofilelist{}", "");
  }

  // Perl: LoadFormat('plain') — Package.pm L2734-2752. Mutually
  // exclusive dump-or-base path (see strict-Perl LoadFormat parity).
  InnerPool!(plain_bootstrap); // Perl: plain_bootstrap.pool.ltxml

  // Diff baseline for `--init=plain.tex`. Perl's plain_dump.pool.ltxml
  // is generated by iniTeX from this exact point: post-bootstrap,
  // BEFORE the plain_base / plain.tex equivalence runs. The diff then
  // captures plain.tex's full contribution (registers, macros,
  // chardefs) as the dump content — which is what
  // `LoadFormat('plain')` later replays. All autoload triggers,
  // file-bookkeeping CSes, and early stubs above are intentionally
  // pre-LoadFormat in Rust ordering so they're in this baseline and
  // do NOT pollute plain.dump.txt.
  latexml_core::state::stage_snapshot("plain_bootstrap");

  if std::env::var_os("LATEXML_NODUMP").is_none() && plain_dump_available() {
    InnerPool!(plain_dump); // Perl: plain_dump.pool.ltxml
  } else {
    InnerPool!(plain_base); // Perl: plain_base.pool.ltxml
  }
  InnerPool!(plain_constructs); // Perl: plain_constructs.pool.ltxml → math_common

  // Perl: LoadFormat('plain') — precompiled plain.tex state.
  // TODO: Enable once dump has full parity (Let, CharDef, Register entries).
  // Requires _loaded flags to prevent re-loading raw TeX files.
  // if let Err(e) = crate::engine::plain_dump::load_definitions() {
  //   log::warn!("plain_dump: {}", e);
  // }

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
