use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: aipproc.cls.ltxml
  // American Institute of Physics Conference Proceedings.

  // Ignorable options
  for option in [
    "10pt", "11pt", "12pt", "twoside", "onecolumn", "twocolumn",
    "draft", "final", "referee",
    "letter",
  ].iter() {
    DeclareOption!(*option, None);
  }

  // Anything else gets passed to article.
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });

  // Font options — store choice in state
  DeclareOption!("mathptmx", { AssignValue!("aipproc_font" => "mathptmx"); });
  DeclareOption!("mathptm",  { AssignValue!("aipproc_font" => "mathptm"); });
  DeclareOption!("mathtime", { AssignValue!("aipproc_font" => "mathptmx"); });
  DeclareOption!("mtpro",    { AssignValue!("aipproc_font" => "mathptmx"); });

  DeclareOption!("varioref",    None);
  DeclareOption!("nonvarioref", None);

  DeclareOption!("tnotealph",   None);
  DeclareOption!("tnotesymbol", None);

  DeclareOption!("numberedheadings",   None);
  DeclareOption!("unnumberedheadings", None);

  // Default font
  AssignValue!("aipproc_font" => "mathptmx");

  ProcessOptions!();
  load_class("article", Vec::new(), Tokens!())?;

  // Load the selected font package
  let font_pkg = lookup_value("aipproc_font")
    .map(|v| v.to_string())
    .unwrap_or_else(|| "mathptmx".to_string());
  if !font_pkg.is_empty() {
    require_package(&font_pkg, RequireOptions::default())?;
  }

  RequirePackage!("fixltx2e");
  RequirePackage!("fontenc");
  RequirePackage!("calc");
  RequirePackage!("varioref");
  RequirePackage!("times");
  RequirePackage!("graphicx");
  RequirePackage!("textcomp");
  RequirePackage!("url");
  RequirePackage!("textcase");
  RequirePackage!("natbib");

  //======================================================================
  // Frontmatter
  def_macro_noop("\\layoutstyle{}")?;

  // \author{name}[keys]  Use one \author per author;
  // various contact information is supplied via the optional keyvals
  // (Perl PR #2767: each present key becomes a contact shorthand inside
  // the \lx@add@author content).
  DefMacro!("\\author{} RequiredKeyVals", sub[(author, kv)] {
    let mut content: Vec<Token> = author.unlist();
    for (field, code) in [
      ("address",    "\\lx@add@contact[role=address]{#1}"),
      ("altaddress", "\\lx@add@contact[role=altaddress]{#1}"),
      ("email",      "\\lx@add@contact[role=email]{#1}"),
      ("homepage",   "\\lx@add@contact[role=url]{#1}"),
      ("thanks",     "\\lx@add@contact[role=thanks]{#1}"),
    ] {
      if let Some(value) = kv.get_value(field) {
        content.extend(build_invocation_str(code, vec![Some(value.revert()?)])?.unlist());
      }
    }
    Ok(Invocation!("\\lx@add@author{#1}", vec![Some(Tokens::new(content))]))
  });

  DefMacro!("\\keywordsname", "Keywords");
  DefMacro!("\\keywords{}", "\\lx@add@keywords[name={\\keywordsname:~}]{#1}");
  DefMacro!("\\classification{}", "\\lx@add@classification{#1}");

  DefMacro!("\\received{}", "\\lx@add@date[role=received]{#1}");
  DefMacro!("\\revised{}", "\\lx@add@date[role=revised]{#1}");
  DefMacro!("\\accepted{}", "\\lx@add@date[role=accepted]{#1}");
  DefMacro!("\\draftdate{}", "\\lx@add@date[role=draft]{#1}");
  DefMacro!("\\copyrightholder{}", "\\lx@add@copyrightholder{#1}");
  DefMacro!("\\copyrightyear{}", "\\lx@add@copyrightyear{#1}");

  DefEnvironment!("{theacknowledgments}",
    "<ltx:acknowledgements>#body</ltx:acknowledgements>");

  //======================================================================
  DefConstructor!("\\eqref Semiverbatim",
    "(<ltx:ref labelref='#label' _force_font='true'/>)",
    properties => sub[args] {
      unpack_opt_ref!(args => label_opt);
      let label = label_opt.as_ref().unwrap().to_string();
      Ok(stored_map!("label" => Stored::String(pin(clean_label(&label, None)))))
    },
    enter_horizontal => true
  );

  //======================================================================
  DefMacro!("\\source{}", "\\lx@note{source}{#1}");
  def_macro_noop("\\spaceforfigure{}{}")?;

  DefMacro!("\\tablehead{}{}{}{}", "\\multicolumn{#1}{#2}{\\parbox{#3}{#4}}");
  // Perl aipproc.cls.ltxml L101 body references `#1` (the OptionalMatch:*
  // star flag), silently dropping the note content. See
  // docs/KNOWN_PERL_ERRORS.md #16. Rust deliberately indexes `#2` (the
  // content) to match the documented sibling convention from physics.sty.
  DefMacro!("\\tablenote OptionalMatch:* {}", "\\footnote{#2}");

  // Perl aipproc.cls.ltxml does NOT define \references — Perl behaves
  // lossy-silent for `\begin{references}…\bibitem` under aipproc
  // (drops the whole bibliography, reports "No obvious problems").
  // Rust's stricter validator surfaces Error:malformed:ltx:bibitem
  // "…isn't allowed in <ltx:section>". Rust-over-Perl improvement:
  // alias to the thebibliography machinery so the content is
  // preserved. Fixes 4 papers in SANDBOX_TRIAGE_2026-05-21 Class D bibitem-aipproc
  // cluster (astro-ph9711070, cond-mat0109365, nucl-ex9706010,
  // nucl-th0010030). See also the mirror alias in aipproc_sty.rs.
  //
  // CRITICAL: `\reference` is `\let` to `\bibitem` ONLY WITHIN the `references`
  // environment (inside `\references`'s body), NOT globally. Perl leaves
  // `\reference` undefined, so a paper that defines its own
  // `\newcommand{\reference}{...}` (a common math shorthand, e.g.
  // `\newcommand{\reference}{\mathrm{ref}}`) succeeds. A GLOBAL alias here made
  // `\reference` already-defined, so the user `\newcommand` silently failed and
  // `\reference` stayed `\bibitem` — firing a `\bibitem` inside `$…$` math
  // (`$\temp_{\reference}$`) → a `<ltx:bibitem>` in `<ltx:XMArg>` → math-mode
  // leak that swallowed the real bibliography & caption tags. Witness
  // 1701.08966: RUST 102 errors / FATAL_3 → 0 (Perl: 1; surpasses). Scoping to
  // the env keeps the aipproc-bibitem cluster working (those use
  // `\begin{references}\reference{…}`). The `\let` is local to the env group.
  DefMacro!("\\references", "\\let\\reference\\bibitem\\thebibliography{}");
  Let!("\\endreferences", "\\endthebibliography");
});
