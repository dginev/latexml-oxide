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
      require_package(&pkg_name, RequireOptions::default())?;
      // Promote whatever the package just installed for this CS from its
      // current local scope (which is the calling group's frame, when the
      // autoload fires from inside `\begin{X}`) to GLOBAL — so the binding
      // survives `\end{X}`'s pop_frame. Without this, the second use of
      // the CS would find the cleared `Stored::None` AND a `:locked=true`
      // (which the package's `DefMacro!(..., locked=>true)` set globally),
      // triggering a redef-blocked-by-lock loop on each generate_error_stub
      // attempt. The trigger CS is the only one we promote — the package's
      // internal CSes (referenced by the trigger's body) are typically
      // installed by base-pool layers at depth=0 already, so they survive
      // independently. (1711.11576: 18M-line timeout → clean output.)
      latexml_core::state::let_i(&cs_for_closure, &cs_for_closure, Some(Scope::Global));
      Ok(Tokens::new(vec![cs_for_closure]))
    })),
    None,
  )?;
  Ok(())
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

  // Perl: LoadFormat('plain') — loads bootstrap → base → dump → constructs
  InnerPool!(plain_bootstrap); // Perl: plain_bootstrap.pool.ltxml
  InnerPool!(plain_base); // Perl: plain_base.pool.ltxml
  // plain_dump: not loaded separately — plain.ltx state is subsumed by latex dump
  InnerPool!(plain_constructs); // Perl: plain_constructs.pool.ltxml → math_common

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Perl: TeX.pool.ltxml — remaining definitions
  //======================================================================

  Let!("\\protect", "\\relax");
  // \everyhelp removed — not in Perl (register from plain.tex kernel, comes from dump)
  // \hiderel removed — not in Perl LaTeXML at all

  // Perl TeX.pool.ltxml L33-56: autoload triggers for LaTeX, expl3, AmSTeX
  // (moved from latex_hook.rs which has no Perl equivalent)
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
