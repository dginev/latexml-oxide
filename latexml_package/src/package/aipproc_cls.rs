use crate::{
  prelude::*,
  shipped_class::{class_code_contains, shipped_class_text},
};

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
  LoadClass!("article");

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

  // aipproc.cls:405 `\RequirePackage{aipxfm}`: the class's front matter (`\title`, `\author{name}{keys}`, `\keywords`,
  // `\classification`, `{abstract}`, `\maketitle`), which this binding defines. The binding records it loaded, as the
  // class does, so a paper's own `\usepackage{aipxfm}` is the no-op it is in LaTeX; it raw-loaded the shipped
  // aipxfm.sty, whose layout code (aipxfm.sty:372-377 `\DeclareParagraphLayout`, `\SetInternalRegister\hyphenpenalty`,
  // and the `.clo` it reads) ran into the undefined internals of the class (117 papers of the inventory: 1010.4521,
  // 0705.1061, 0709.0115; Perl the same, KNOWN_PERL_ERRORS #730).
  AssignValue!("aipxfm.sty_loaded" => true, Some(Scope::Global));
  // aipproc.cls:212-216, 336-364, 383-386: the designer interface the class's layout files use — register setting,
  // paragraph layouts, the space catcodes, the body-text dimensions (a paper's own layout code calls them:
  // `\MakeSpaceNormal`, `\bodytextparindent`, 1010.4521).
  RawTeX!(
    r"\newcommand\MakeSpaceIgnore{\catcode`\~=10\relax\catcode`\ = 9\relax\catcode`\^^M = 9\relax}
\newcommand\MakeSpaceNormal{\catcode`\~= 13\relax\catcode`\ = 10\relax\catcode`\^^M = 5\relax}
\newcommand*\SetInternalRegister[2]{#1=#2\relax}
\newcommand*\DeclareParagraphLayout[9]{\@namedef{#1Para}{\fontsize{#2}{#3}\selectfont#9\setlength\parindent{#4}\setlength\leftskip{#5}\setlength\rightskip{#6}\@rightskip\rightskip\setlength\parfillskip{#7}\setlength\parskip{#8}}}
\newcommand*\UseParagraphLayout[1]{\@ifundefined{#1Para}{}{\@nameuse{#1Para}}}
\newcommand*\DeclareParagraphLayoutAlias[2]{\@ifundefined{#2Para}{}{\expandafter\let\csname#1Para\expandafter\endcsname\csname#2Para\endcsname}}
\newdimen\bodytextsize\newdimen\bodytextbaselineskip\newdimen\bodytextenspace\newdimen\bodytextparindent"
  );
  // The journal abbreviations some copies add (0809.4021's aipproc.cls:1042-1124, 1103.3992's, 5 of the sampled
  // copies: `\apj`, `\apjl`, `\mnras`, `\nat` 5 papers, `\aap`, `\araa`, `\physrep`): defined where the paper's copy
  // has them, so a paper whose copy does not can define its own.
  if shipped_class_text("aipproc")?
    .as_deref()
    .is_some_and(|text| class_code_contains(text, "\\def\\ref@jnl#1{{\\jnl@style#1}}"))
  {
    RawTeX!(
      r"\let\jnl@style=\rmfamily\def\ref@jnl#1{{\jnl@style#1}}
\newcommand\aj{\ref@jnl{AJ}}\newcommand\araa{\ref@jnl{ARA\&A}}\newcommand\apj{\ref@jnl{ApJ}}\newcommand\apjl{\ref@jnl{ApJ}}
\newcommand\apjs{\ref@jnl{ApJS}}\newcommand\ao{\ref@jnl{Appl.~Opt.}}\newcommand\apss{\ref@jnl{Ap\&SS}}
\newcommand\aap{\ref@jnl{A\&A}}\newcommand\aapr{\ref@jnl{A\&A~Rev.}}\newcommand\aaps{\ref@jnl{A\&AS}}
\newcommand\azh{\ref@jnl{AZh}}\newcommand\baas{\ref@jnl{BAAS}}\newcommand\jrasc{\ref@jnl{JRASC}}
\newcommand\memras{\ref@jnl{MmRAS}}\newcommand\mnras{\ref@jnl{MNRAS}}\newcommand\pra{\ref@jnl{Phys.~Rev.~A}}
\newcommand\prb{\ref@jnl{Phys.~Rev.~B}}\newcommand\prc{\ref@jnl{Phys.~Rev.~C}}\newcommand\prd{\ref@jnl{Phys.~Rev.~D}}
\newcommand\pre{\ref@jnl{Phys.~Rev.~E}}\newcommand\prl{\ref@jnl{Phys.~Rev.~Lett.}}\newcommand\pasp{\ref@jnl{PASP}}
\newcommand\pasj{\ref@jnl{PASJ}}\newcommand\qjras{\ref@jnl{QJRAS}}\newcommand\skytel{\ref@jnl{S\&T}}
\newcommand\solphys{\ref@jnl{Sol.~Phys.}}\newcommand\sovast{\ref@jnl{Soviet~Ast.}}\newcommand\ssr{\ref@jnl{Space~Sci.~Rev.}}
\newcommand\zap{\ref@jnl{ZAp}}\newcommand\nat{\ref@jnl{Nature}}\newcommand\iaucirc{\ref@jnl{IAU~Circ.}}
\newcommand\aplett{\ref@jnl{Astrophys.~Lett.}}\newcommand\apspr{\ref@jnl{Astrophys.~Space~Phys.~Res.}}
\newcommand\bain{\ref@jnl{Bull.~Astron.~Inst.~Netherlands}}\newcommand\fcp{\ref@jnl{Fund.~Cosmic~Phys.}}
\newcommand\gca{\ref@jnl{Geochim.~Cosmochim.~Acta}}\newcommand\grl{\ref@jnl{Geophys.~Res.~Lett.}}
\newcommand\jcp{\ref@jnl{J.~Chem.~Phys.}}\newcommand\jgr{\ref@jnl{J.~Geophys.~Res.}}
\newcommand\jqsrt{\ref@jnl{J.~Quant.~Spec.~Radiat.~Transf.}}\newcommand\memsai{\ref@jnl{Mem.~Soc.~Astron.~Italiana}}
\newcommand\nphysa{\ref@jnl{Nucl.~Phys.~A}}\newcommand\physrep{\ref@jnl{Phys.~Rep.}}\newcommand\physscr{\ref@jnl{Phys.~Scr}}
\newcommand\planss{\ref@jnl{Planet.~Space~Sci.}}\newcommand\procspie{\ref@jnl{Proc.~SPIE}}
\let\astap=\aap\let\apjlett=\apjl\let\apjsupp=\apjs\let\applopt=\ao"
    );
  }

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
  // aipxfm.sty:266-269 (loaded by aipproc.cls): `\let\pacs\classification`, the REVTeX name the AIP templates keep
  // (cond-mat0701310, 0911.1625).
  Let!("\\pacs", "\\classification");

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

  // aipproc.cls:668-672 `\multicolumn{#1}{#2}{\AIPtableheadfont\begin{tabular}[#3]{@{}#2@{}}\vrule…#4\unskip\vrule…
  // \end{tabular}}`: `#3` is the head's vertical position (t/c/b), not a width. Perl's `\parbox{#3}{#4}`
  // (aipproc.cls.ltxml:112) set a width of `b`, which calc rejects (calc.sty:144-160, since 56jr: "`b' invalid at this
  // point", ~428 of run 336's papers; KNOWN_PERL_ERRORS #560); the struts and head font are print layout.
  DefMacro!("\\tablehead{}{}{}{}", "\\multicolumn{#1}{#2}{\\begin{tabular}[#3]{@{}#2@{}}\\ignorespaces#4\\unskip\\end{tabular}}");
  // Perl aipproc.cls.ltxml L101 body references `#1` (the OptionalMatch:*
  // star flag), silently dropping the note content. See
  // docs/parity/KNOWN_PERL_ERRORS.md #16. Rust deliberately indexes `#2` (the
  // content) to match the documented sibling convention from physics.sty.
  DefMacro!("\\tablenote OptionalMatch:* {}", "\\footnote{#2}");

  // Perl aipproc.cls.ltxml does NOT define \references — Perl behaves
  // lossy-silent for `\begin{references}…\bibitem` under aipproc
  // (drops the whole bibliography, reports "No obvious problems").
  // Rust's stricter validator surfaces Error:malformed:ltx:bibitem
  // "…isn't allowed in <ltx:section>". Rust-over-Perl improvement:
  // alias to the thebibliography machinery so the content is
  // preserved. Fixes 4 papers in docs/archive/SANDBOX_TRIAGE_2026-05-21.md Class D bibitem-aipproc
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
