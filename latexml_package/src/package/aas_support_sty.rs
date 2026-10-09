use crate::{
  engine::latex_constructs::{after_float, before_float_ex},
  package::subcaption_sty::subcaption_width_props,
  prelude::*,
};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: aas_support.sty.ltxml — support macros for AAS styles

  // Package dependencies — Perl L28-39
  RequirePackage!("aas_macros");
  RequirePackage!("url");
  RequirePackage!("longtable");
  // Pre-load with [dvipsnames, table] so AAS-style papers using
  // `\usepackage[table]{xcolor}` get colortbl loaded too.
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string(), "table".to_string()]);
  RequirePackage!("hyperref");
  RequirePackage!("array");
  RequirePackage!("lineno");
  RequirePackage!("amssymb");
  RequirePackage!("epsf");
  RequirePackage!("ulem");

  // 2.1.3 Editorial Information
  DefMacro!("\\received{}", "\\lx@add@date[role=received,name=Received]{#1}");
  DefMacro!("\\revised{}", "\\lx@add@date[role=revised,name=Revised]{#1}");
  DefMacro!("\\accepted{}", "\\lx@add@date[role=accepted,name=Accepted]{#1}");
  // Journal metadata — preserve as ltx:note frontmatter (the values
  // are real article identifiers that downstream tools may want).
  DefMacro!("\\journalid{}{}",
    "\\lx@add@frontmatter{ltx:note}[role=journalid]{#1: #2}");
  DefMacro!("\\articleid{}{}",
    "\\lx@add@frontmatter{ltx:note}[role=articleid]{#1: #2}");
  DefMacro!("\\paperid{}",
    "\\lx@add@frontmatter{ltx:note}[role=paperid]{#1}");
  DefMacro!("\\msid{}",
    "\\lx@add@frontmatter{ltx:note}[role=msid]{#1}");
  // Review markup — pass-through (#1) for now, since no LaTeXML container
  // element accepts ltx:section. ltx:text is inline-only, ltx:inline-block
  // takes Block.model (paragraphs/equations but no sections), ltx:note
  // takes Flow.model (still no sections). When authors use
  // `\added{\section{...}...multi-paragraph...}` to annotate a whole
  // revised section (common in aastex appendices), any wrapper auto-
  // closes when \section opens, then the }-token tries to close an
  // already-closed wrapper → `Error:malformed:ltx:text/inline-block`.
  // Witness 2110.12098. Pass-through preserves content; the review
  // semantic class is lost in HTML output but the body survives.
  DefMacro!("\\added{}",    "#1");
  DefMacro!("\\replaced{}", "#1");
  DefMacro!("\\deleted{}",  "#1");
  DefMacro!("\\explain{}",  "#1");
  DefMacro!("\\edit{}{}",   "#2");
  def_macro_noop("\\ccc{}")?;
  DefMacro!("\\cpright{}{}", "\\lx@add@copyright{#2: #1}");
  DefMacro!("\\journal{}",
    "\\lx@add@pubnote[role=journal]{#1}");
  DefMacro!("\\volume{}",
    "\\lx@add@pubnote[role=volume]{#1}");
  DefMacro!("\\issue{}",
    "\\lx@add@pubnote[role=issue]{#1}");
  DefMacro!("\\SGMLbi{}", "#1");
  DefMacro!("\\SGMLbsc{}", "#1");
  DefMacro!("\\SGMLclc{}", "#1");
  DefMacro!("\\SGMLentity{}", "#1");
  def_macro_noop("\\SGML{}")?;

  // 2.1.4 Short Comment
  DefMacro!("\\slugcomment{}", "\\lx@add@pubnote[role=note]{#1}");

  // 2.1.5 Running Heads
  DefMacro!("\\shorttitle{}", "\\lx@add@toctitle{#1}");
  // Perl `aas_support.sty.ltxml` L83: `DefMacro('\shortauthors{}', '')` — GOBBLE
  // (comment: "not useful?", "redundantly with an \author macro"). We previously
  // preserved it as `ltx:note[role=shortauthors]` (Rust-over-Perl), but (a) it
  // duplicates `\author`, and (b) digesting its content errors when an author
  // writes a literal `&` (an "and" typo for `\&`) in the running head — the
  // catcode-4 `&` hits the stray-`&` constructor (no alignment open). Witness
  // 0709.4236 (`\shortauthors{Riaz, Gizis & Sammaddar}`): RUST 1 error → 0
  // (Perl clean). Match Perl: gobble.
  def_macro_noop("\\shortauthors{}")?;
  // \lefthead{author} / \righthead{title} — running-header text;
  // preserve as ltx:note (was gobbled).
  DefMacro!("\\lefthead{}",
    "\\lx@add@frontmatter{ltx:note}[role=lefthead]{#1}");
  DefMacro!("\\righthead{}",
    "\\lx@add@frontmatter{ltx:note}[role=righthead]{#1}");

  // 2.3 Title and Author Information (Perl PR #2767)

  def_macro_noop("\\allauthors")?; // Presumably should collect all authors?

  // \author[orcid,KV]{name}  Use once per author
  // Note optional arg starts with orcid ID,
  // followed by keyval (gname,sname,suffix for parts of author's name)
  // A name list in one `\author` (older aastex, emulateapj: `Joshua N.\ Winn\altaffilmark{1}, Andrew W.\ Howard
  // \altaffilmark{2},\\ Avi Shporer\altaffilmark{1}`, 1010.1318) is one author each, as the article `\author` reads it
  // (Perl aas_support.sty.ltxml:103 makes it one author; 63e).
  // The orcid goes to the last author the block makes, after it (inside, it rode on the last `\\` line).
  DefMacro!("\\author[]{}", "\\lx@add@authors@append{#2}\\lx@aas@checkorcid{#1}");
  DefMacro!("\\lx@aas@checkorcid{}", "\\lx@aas@checkorcid@#1,\\done");
  // In later aas, $junk may be keywords? (gname,sname,suffix,...?)
  DefMacro!("\\lx@aas@checkorcid@ Until:, Until:\\done", sub[(possibleid, _junk)] {
    // Perl: /^\s*\d\d\d\d-\d\d\d\d-\d\d\d\d-\d\d\d\d\s*$/ — digits only
    let id = possibleid.to_string();
    let trimmed = id.trim();
    let is_orcid = trimmed.len() == 19
      && trimmed.split('-').count() == 4
      && trimmed.split('-').all(|seg| seg.len() == 4
          && seg.chars().all(|c| c.is_ascii_digit()));
    if is_orcid {
      Ok(Invocation!("\\lx@add@orcid{#1}", vec![Some(possibleid)]))
    } else {
      Ok(Tokens!())
    }
  });

  // \email[]{addr} applies to previous author
  // \correspondingauthor{name}
  DefMacro!("\\correspondingauthor{}",
    "\\lx@add@contact[role=correspondent,label={fuzzy:#1}]{#1}");

  // Various contact information attaches to previous \author
  // AASTeX 6 (revtex-derived): an affiliation is every preceding author's that has none yet (`\author{Ann Able and Bob
  // Baker}\affiliation{Univ}` gives it to both), as revtex's is (`annotate=new`; 63e).
  DefMacro!("\\affiliation{}",    "\\lx@add@affiliation[annotate=new]{#1}");
  DefMacro!("\\altaffiliation{}", "\\lx@add@altaffiliation{#1}");
  DefMacro!("\\affil",            "\\affiliation");
  DefMacro!("\\authoraddr{}",     "\\lx@add@address{#1}");
  // AASTeX 7 `\email[show]{…}` (aastex701.cls:13341 `\@ifnextchar[\@email{\@email[]}`; `show` prints it in the
  // title footnote "Email: …", :2455-2456): the option is no part of the address (2608.21320; Perl's `\email{}`,
  // aas_support.sty.ltxml:122, left "[" as the address and "show]…" in the body).
  DefMacro!("\\email[]{}",        "\\lx@add@email{#2}");

  // \collaboration{n}{name} applies to previous authors w/o collab; n is how many to SHOW
  // \nocollaboration{n} ditto, but says they are not in a collaboration group.
  DefMacro!("\\collaboration{}{}", "\\lx@add@contact[role=collaboration,annotate=new]{#2}");
  DefMacro!("\\nocollaboration{}", "\\lx@add@contact[role=collaboration,annotate=new]{}");

  // Older aas versions: put mark within \author, text will be attached
  DefMacro!("\\altaffilmark Semiverbatim",
    "\\lx@request@frontmatter@annotation[altaffil]{#1}");
  DefMacro!("\\altaffiltext Semiverbatim {}",
    "\\lx@add@contact[role=altaffiliation,label={altaffil:#1}]{#2}");
  DefMacro!("\\authoremail", "\\email"); // Obsolete form

  // Redefine to straight email address after document begin.
  // (Perl PR #2767 disables the former \@startsection@hook \let\email\@@email.)

  DefPrimitive!("\\and", None);

  DefMacro!("\\software{}", "\\lx@add@pubnote[role=software]{#1}");
  DefMacro!("\\submitjournal{}", "\\lx@add@pubnote[role=journal]{#1}");

  // Alas \doi is not frontmatter.
  DefConstructor!("\\doi{}", "<ltx:ref href='https://doi.org/#1'>#1</ltx:ref>",
    enter_horizontal => true);

  // 2.4 Abstract
  DefMacro!("\\abstract",    "\\lx@begin@abstract");
  DefMacro!("\\endabstract", "\\lx@end@abstract");

  // 2.5 Keywords
  DefMacro!("\\keywords{}",
    "\\lx@add@keywords[name={\\@ifundefined{keywordsname}{}{\\keywordsname}}]{#1}");
  Let!("\\subjectheadings", "\\keywords");

  // 2.6 Comments to Editors. Faithful to Perl aas_support.sty.ltxml L162-163:
  //   # DefConstructor('\notetoeditor{}',"<ltx:note role='toeditor'>#1</ltx:note>");
  //   DefMacro('\notetoeditor{}', '');
  // Perl COMMENTS OUT the constructor and gobbles the argument to nothing —
  // an editor note is editorial metadata directed at the journal, never
  // reader-facing published content. A prior Rust divergence used the
  // commented-out constructor, which DIGESTS the free-form note prose in text
  // mode and so errors on any special char it contains — e.g. a bare `_` in a
  // filename like `tab2_online.tex` → "Script _ can only appear in math mode"
  // (witness aastex paper 0805.1040: Rust 1 error, Perl clean). Match Perl:
  // gobble it, like the sibling \placetable/\placefigure/\placeplate below.
  def_macro_noop("\\notetoeditor{}")?;

  // 2.8 Figure and Table Placement
  def_macro_noop("\\placetable{}")?;
  def_macro_noop("\\placefigure{}")?;
  def_macro_noop("\\placeplate{}")?;
  // `\floattable` (aastex62.cls L4574: `\def\floattable{\global\deluxestartrue
  // \global\floattrue}`) — a no-arg declaration that makes the FOLLOWING
  // deluxetable a full-width (spanning) float in two-column PDF layout. Pure
  // page-layout, moot in our HTML paradigm (WISDOM #50 / size-layout-errors-
  // moot), so a no-op. Both this binding and Perl's aas_support.sty.ltxml had
  // omitted it (it lives in the aastex62/aastex631 .cls, not aas_support), so
  // a paper bundling aastex62.cls + `\floattable` saw it undefined. Witness
  // 1909.08916 (`\documentclass{aastex62}`, `\floattable` before tables):
  // 1 error → 0. (Perl ALSO errors here — its aas_support binding has the same
  // gap; see docs/parity/KNOWN_PERL_ERRORS.md.)
  def_macro_noop("\\floattable")?;
  NewCounter!("plate");
  // AASTeX 5.x's plates list with the figures (aastex.cls:1685-1704 `\def\ext@plate{lof}`, "Plate N."; 0908.0069); the
  // family binding keeps them for every version (aastex 6+ dropped `{plate}`).
  RawTeX!(r"\def\ext@plate{lof}");
  DefMacro!("\\platename", "Plate");
  def_macro_noop("\\platewidth{Dimension}")?;
  DefMacro!("\\platenum{}", "\\def\\theplate{#1}");

  // Plate environments — Perl aas_support.sty.ltxml L179-201.
  // Each variant calls beforeFloat (sets \@captype, rebinds \\ → \lx@newline,
  // assigns \hsize) and afterFloat (closes the float scope, sets the
  // float number / id). The starred variant additionally passes
  // `double => 1` so \hsize gets \textwidth instead of \columnwidth
  // (two-column-spanning plate). Without these hooks, the Rust port
  // emits an empty <ltx:float> shell that loses caption/number metadata
  // and uses single-column box geometry even in the * variant.
  // Template additionally needs `inlist='#inlist' ?#1(placement='#1')`
  // to match the floats produced by \newfloat-style envs (acmart, rotating).
  DefEnvironment!("{plate}[]",
    "<ltx:float xml:id='#id' inlist='#inlist' ?#1(placement='#1') class='ltx_float_plate'>#tags#body</ltx:float>",
    before_digest => { before_float_ex("plate", None, false)?; engine::latex_constructs::reset_float_box()?; },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );
  DefEnvironment!("{plate*}[]",
    "<ltx:float xml:id='#id' inlist='#inlist' ?#1(placement='#1') class='ltx_float_plate'>#tags#body</ltx:float>",
    before_digest => { before_float_ex("plate", None, true)?; engine::latex_constructs::reset_float_box()?; },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );

  // Fig macros — Perl L205-221. The smart `\fig` peeks the token after
  // the first Semiverbatim arg: if it's `{` (T_BEGIN), it's a 3-arg
  // panel (`\fig{file}{width}{caption}`); otherwise it's
  // a single-arg ref-like usage (`\fig{label}` → `\ref{label}`). This
  // dispatch is needed for papers like astro-ph/0003209 + astro-ph/0503342
  // that redefine `\fig` as a one-arg `\ref` shorthand inside captions
  // /footnotes — without this peek, `\fig{F:image}` always opens an
  // `<ltx:figure>` element and can land inside `<ltx:note>`.
  // AASTeX 6+ figure grids (aastex6.cls:4909 … aastex701.cls:12314-12342, the same in every version): `\gridline{…}`
  // is one row of panels, `\hbox to\hsize{…}` between `\vskip6pt`s, and `\fig{file}{width}{caption}` one panel, an
  // unnumbered `\vbox` holding the graphic at that width over its `\footnotesize` sub-caption (`\leftfig`/`\rightfig`
  // without the font, `\boxedfig` framed, `\rotatefig{angle}` rotated). Perl (aas_support.sty.ltxml:208) read
  // `\gridline` as nothing, every panel lost (2609.21324, 2512.02147, 2505.20669, 2103.00374); and a bare `\fig` in a
  // float became a numbered figure of its own, so each panel stepped the figure counter (2103.00666: "Figure 3" where
  // the PDF prints 1). A panel is a child `ltx:figure` as subcaption's `{subfigure}` makes one, with no counter; a
  // gridline's panels carry their row (`_gridrow`), which the panel arrangement keeps whole however wide it adds up
  // (a `\hbox to\hsize` of panels with negative `\hspace`s, 2609.09897).
  DefEnvironment!("{lx@aas@panel}{Dimension}",
    "^<ltx:figure xml:id='#id' ?#gridrow(_gridrow='#gridrow')>#body</ltx:figure>",
    mode => "internal_vertical",
    properties => sub[args] { aas_panel_props(args) });
  DefEnvironment!("{lx@aas@boxedpanel}{Dimension}",
    "^<ltx:figure xml:id='#id' framed='rectangle' ?#gridrow(_gridrow='#gridrow')>#body</ltx:figure>",
    mode => "internal_vertical",
    properties => sub[args] { aas_panel_props(args) });
  // (an empty or blank caption, `\fig{a.pdf}{0.6\textwidth}{}`, prints none)
  DefMacro!("\\lx@aas@panel@caption{}{}", sub[(font, text)] {
    Ok(if text.unlist_ref().iter().all(|t| *t == T_SPACE!()) {
      Tokens!()
    } else {
      let mut caption = vec![T_CS!("\\@@caption"), T_BEGIN!()];
      caption.extend(font.unlist());
      caption.extend(text.unlist());
      caption.push(T_END!());
      Tokens::new(caption)
    })
  });
  DefMacro!("\\lx@aas@fig@panel Semiverbatim {Dimension}{}",
    "\\begin{lx@aas@panel}{#2}\\includegraphics[width=#2]{#1}\\lx@aas@panel@caption{\\footnotesize}{#3}\\end{lx@aas@panel}");
  DefMacro!("\\lx@aas@sidefig@panel Semiverbatim {Dimension}{}",
    "\\begin{lx@aas@panel}{#2}\\includegraphics[width=#2]{#1}\\lx@aas@panel@caption{}{#3}\\end{lx@aas@panel}");
  DefMacro!("\\lx@aas@boxedfig@panel Semiverbatim {Dimension}{}",
    "\\begin{lx@aas@boxedpanel}{#2}\\includegraphics[width=#2]{#1}\\lx@aas@panel@caption{}{#3}\\end{lx@aas@boxedpanel}");
  // The angle is a decimal (aastex701.cls:12337-12342 `angle=#1` → trig.sty:55):
  // a `{Number}` put the `.5` of `{22.5}` back in the input (OXIDIZED_DESIGN #317).
  DefMacro!("\\lx@aas@rotatefig@panel{Float} Semiverbatim {Dimension}{}",
    "\\begin{lx@aas@panel}{#3}\\includegraphics[width=#3,angle=#1]{#2}\\lx@aas@panel@caption{\\footnotesize}{#4}\\end{lx@aas@panel}");
  // A gridline is a row: its panels in their panel forms, and a break from the row before it in the float (none before
  // the first row, none after a caption).
  DefMacro!("\\gridline{}",
    "\\par\\lx@aas@gridline@row\\begingroup\\lx@aas@gridrow@begin\\let\\fig\\lx@aas@fig@panel\
     \\let\\leftfig\\lx@aas@sidefig@panel\\let\\rightfig\\lx@aas@sidefig@panel\\let\\boxedfig\\lx@aas@boxedfig@panel\
     \\let\\rotatefig\\lx@aas@rotatefig@panel#1\\endgroup\\par");
  // (the row number its panels carry, for this group)
  DefPrimitive!("\\lx@aas@gridrow@begin", sub[_args] {
    let row = lookup_int("lx_aas_gridrows") + 1;
    assign_value("lx_aas_gridrows", Number::new(row), Some(Scope::Global));
    assign_value("lx_aas_gridrow", Number::new(row), None);
  });
  // The break goes where the row's panels are — in the float, or in a `{center}` within it (2103.16579) whose content
  // moves into the float with it — after a panel there, none before the first row or after a caption.
  DefConstructor!("\\lx@aas@gridline@row", sub[document] {
    let Some(context) = document.get_element() else {
      return Ok(());
    };
    let mut ancestor = Some(context.clone());
    while let Some(node) = ancestor.as_ref()
      && !with(document::get_node_qname(node), |q| matches!(q, "ltx:figure" | "ltx:table" | "ltx:float"))
    {
      ancestor = node.get_parent();
    }
    let after_panel = context
      .get_last_element_child()
      .is_some_and(|last| !engine::latex_constructs::is_panel_break_name(document::get_node_qname(&last)));
    if ancestor.is_some() && after_panel {
      document.insert_element("ltx:break", Vec::new(), Some(map!("class" => s!("ltx_break"))))?;
    }
  });
  // `\fig` and its kin outside a gridline are the class's panel too — no AAS class has a numbered `\fig` (Perl's
  // float form was its own guess, aas_support.sty.ltxml:210); not followed by a brace, a `\ref` shorthand some papers
  // define (astro-ph/0003209, astro-ph/0503342), LaTeXML's peek.
  DefMacro!("\\fig Semiverbatim Token", sub[(arg, test)] { aas_fig_dispatch(arg, test, "\\lx@aas@fig@panel") });
  DefMacro!("\\leftfig Semiverbatim Token", sub[(arg, test)] { aas_fig_dispatch(arg, test, "\\lx@aas@sidefig@panel") });
  DefMacro!("\\rightfig Semiverbatim Token", sub[(arg, test)] { aas_fig_dispatch(arg, test, "\\lx@aas@sidefig@panel") });
  DefMacro!("\\boxedfig Semiverbatim Token", sub[(arg, test)] { aas_fig_dispatch(arg, test, "\\lx@aas@boxedfig@panel") });
  Let!("\\rotatefig", "\\lx@aas@rotatefig@panel");

  // 2.9 Acknowledgements
  // ltx:acknowledgements Tag (autoClose + inlist=toc) is global — set in
  // latex_constructs.rs (arXiv-fork 23771504 removed the binding-local copies).
  DefConstructor!("\\acknowledgements", "<ltx:acknowledgements>");
  Let!("\\acknowledgments", "\\acknowledgements");
  // AASTeX 6.3+ shortcut: `\ack{...}` (with mandatory arg, distinct from
  // ptephy's argument-less form).
  DefMacro!("\\ack{}", "\\begin{acknowledgements}#1\\end{acknowledgements}");

  // AASTeX 7 `{contribution}` — author-contribution statement placed in the document
  // body just before `{acknowledgments}` (real aastex7/aastex701.cls L7903 renders it as
  // `\section*{Author contributions}` + body). Map to the established acknowledgement-block
  // idiom (cf. `\acknowledgements` above; aa_support/acmart/JHEP), NOT title-block frontmatter.
  // BEYOND PERL: the aastex-v5 `aastex.cls.ltxml` shim (matched via the version-suffix fallback
  // aastex701->aastex) predates aastex7, so Perl also errors `undefined:{contribution}`.
  // Witnesses 2606.03375, 2606.04105 (aastex701).
  DefConstructor!("\\contribution", "<ltx:acknowledgements name='Author contributions'>");
  DefMacro!("\\endcontribution", "");
  // AASTeX draft `\watermark{DRAFT}` — page-header decoration only; gobble. BEYOND PERL.
  // Witness 2606.04105.
  DefMacro!("\\watermark{}", "");

  // \uat{name}{id} — Unified Astronomy Thesaurus term link. Used in
  // \keywords by AASTeX 6.3+. Round-34 surpass: emit as a clickable
  // link to https://astrothesaurus.org/uat/<id> so the UAT id is
  // preserved. Witness 2502.17661 (aastex7).
  DefMacro!("\\uat{}{}", "\\href{https://astrothesaurus.org/uat/#2}{#1}");

  // 2.10 Facilities
  DefConstructor!("\\facility{}", "<ltx:text class='ltx_ast_facility'>#1</ltx:text>",
    enter_horizontal => true);
  DefMacro!("\\facilities{}", "\\lx@add@pubnote[role=thanks,name={Facilities:~}]{#1}");

  // 2.11 Appendices — Perl aas_support.sty.ltxml L247-249
  DefMacro!("\\appendix", "\\@appendix");
  // `\@appendix` starts section-numbered appendices, then re-scopes the
  // equation counter to reset within each appendix section. Perl uses
  // `scope => 'global'` on the `\theequation` redefinition so the new
  // numbering format outlives the current group — in Rust we pass
  // Some(Scope::Global) to def_macro for the same effect. The appendix
  // numbering is `\thesection\arabic{equation}` (no separator) — matches
  // AAS journal style, distinct from Rust's `\eqsecnum` macro L369 which
  // uses a dash separator.
  DefPrimitive!("\\@appendix", {
    start_appendices("section");
    // Perl L248 passes `idprefix => 'E'` so appendix equations get
    // xml:ids like `S1.E2`. Without the prefix, Rust falls back to the
    // default (empty) and collides with body-equation ids once the
    // document reaches its second appendix.
    new_counter(
      "equation",
      "section",
      Some(NewCounterOptions { idprefix: "E", ..Default::default() }),
    )?;
    def_macro(
      T_CS!("\\theequation"),
      None,
      mouth::tokenize_internal("\\thesection\\arabic{equation}"),
      Some(ExpandableOptions { scope: Some(Scope::Global), ..Default::default() }),
    )?;
  });

  // AASTeX 6.3+/7 `\restartappendixnumbering` (== `\apptablenumbers`, aastex701.cls L13630 /
  // aastex631.cls L7206): restart table/figure/equation numbering within each appendix
  // section, prefixed by the lettered section number. Faithful port of `\apptablenumbers`
  // (drops the presentational `\fnum@`/`\theH` lines). BEYOND PERL: absent from the aastex-v5
  // shim, so Perl also errors `undefined:\restartappendixnumbering`. Witnesses 2606.00569,
  // 2606.03850, 2606.07452.
  DefMacro!("\\apptablenumbers",
    "\\setcounter{table}{0}\\setcounter{figure}{0}\\setcounter{equation}{0}\
     \\def\\thetable{\\thesection\\arabic{table}}\
     \\def\\thefigure{\\thesection\\arabic{figure}}");
  Let!("\\restartappendixnumbering", "\\apptablenumbers");

  // 2.12 Equations
  DefMacro!("\\mathletters", "\\lx@equationgroup@subnumbering@begin");
  DefMacro!("\\endmathletters", "\\lx@equationgroup@subnumbering@end");

  // 2.12 Equations — Perl L261 (proper tag setter, not empty stub)
  DefMacro!("\\eqnum{}",
    "\\lx@equation@settag{\\edef\\theequation{#1}\\lx@make@tags{equation}}");

  // 2.13 Citations — Perl L264-293
  def_macro_noop("\\markcite{}")?;
  RequirePackage!("natbib");

  // Perl aas_support.sty.ltxml:283-291:
  //   DefConstructor('\references',
  //     "<ltx:bibliography xml:id='#id' ... ><ltx:title>#title</ltx:title><ltx:biblist>",
  //     afterDigest => sub { beginBibliography($_[1]); });
  //   DefConstructor('\endreferences', sub { maybeCloseElement biblist/bibliography; });
  //
  // Without `afterDigest => beginBibliography`, Rust's \bibitem fires
  // unguarded: the open `<ltx:biblist>` child-admission rules don't take
  // effect (beginBibliography installs them), so `\bibitem` ends up
  // absorbed by whatever the current element is — `<ltx:section>`,
  // `<ltx:para>`, `<ltx:text>`, `<ltx:XMath>` in the 4 failing 10k-sandbox
  // papers (astro-ph9711070, cond-mat0109365, nucl-ex9706010,
  // nucl-th0010030) → "malformed:ltx:bibitem isn't allowed in <ltx:X>".
  //
  // Matching revtex4_support_sty.rs:146-159's pattern for its own
  // `\references` (which already calls begin_bibliography). The Perl
  // attribute set (bibstyle/citestyle/sort/title) is richer than what
  // the Rust template currently emits — that's a separate enhancement;
  // landing the afterDigest hook alone is what closes the 4-paper
  // malformed:ltx:bibitem cluster.
  DefConstructor!(
    "\\references",
    "<ltx:bibliography xml:id='#id'><ltx:biblist>",
    after_digest => sub[whatsit] {
      engine::latex_constructs::begin_bibliography(whatsit)?;
    }
  );
  DefConstructor!(
    "\\endreferences",
    sub[document, _whatsit, _props] {
      document.maybe_close_element("ltx:biblist")?;
      document.maybe_close_element("ltx:bibliography")?;
    }
  );
  Let!("\\reference", "\\bibitem");

  RequirePackage!("graphicx");

  // 2.14 Electronic Art
  DefMacro!("\\figurenum{}", "\\def\\thefigure{#1}");
  def_macro_noop("\\epsscale{}")?;
  DefMacro!("\\plotone Semiverbatim", "\\includegraphics[width=\\textwidth]{#1}");
  DefMacro!("\\plottwo Semiverbatim Semiverbatim",
    "\\hbox{\\includegraphics[width=\\textwidth]{#1}\\includegraphics[width=\\textwidth]{#2}}");
  DefMacro!("\\plotfiddle Semiverbatim {}{}{}{}{}{}",
    "\\includegraphics[width=#4pt,height=#5pt]{#1}");

  // 2.14.2 Figure Captions
  // Perl: `DefMacro('\figcaption OptionalSemiverbatim', sub { ... })`.
  // The optional arg is `OptionalSemiverbatim` — catcodes are neutralized
  // so a literal `_` in `\figcaption[X_Y.ps]{...}` (paper-local filename
  // hint for List-of-Figures) doesn't trigger the math-mode subscript
  // catcode. Driver: arXiv:astro-ph/9808081 has 5× `\figcaption[X_Y.ps]`.
  // \figcaption checks if inside a figure environment.
  // If yes → \caption; if no → \@figcaption (wraps in figure env).
  DefMacro!("\\@figcaption {}", "\\begin{figure}#1\\end{figure}");
  DefMacro!("\\figcaption OptionalSemiverbatim", sub[(opt_arg)] {
    let env = lookup_string_from_sym(pin!("current_environment"));
    if env.contains("figure") {
      // Inside figure: act as \caption
      if let Some(opt) = opt_arg {
        Ok(Tokens!(T_CS!("\\caption"), T_OTHER!("["), opt, T_OTHER!("]")))
      } else {
        Ok(Tokens!(T_CS!("\\caption")))
      }
    } else {
      // Outside figure: wrap in \@figcaption
      Ok(Tokens!(T_CS!("\\@figcaption")))
    }
  });

  // 2.15 Tables
  RequirePackage!("deluxetable");
  Let!("\\planotable", "\\deluxetable");
  Let!("\\endplanotable", "\\enddeluxetable");

  // Perl: aas_support.sty.ltxml L380-383
  Let!("\\splitdeluxetable", "\\deluxetable");
  Let!("\\endsplitdeluxetable", "\\enddeluxetable");
  let_i(&T_CS!("\\splitdeluxetable*"), &T_CS!("\\deluxetable*"), None);
  let_i(&T_CS!("\\endsplitdeluxetable*"), &T_CS!("\\enddeluxetable*"), None);

  // aastex631.cls L4780-4781:
  //   \newif\ifstartlongtable
  //   \def\startlongtable{\vskip1sp\global\startlongtabletrue}
  // We treat as a no-op marker — our deluxetable / longtable handling
  // doesn't need the conditional flag. Driver: 2209.01632 (aastex631)
  // emitted "\startlongtable not defined" + alignment-tab cascade.
  def_macro_noop("\\startlongtable")?;

  // Perl L373: Let('\savedollar' => T_MATH). The hidden 'h' column type
  // used by aas deluxetable tokenizes literal `$` from the template, so
  // the package stashes an active math-shift token into `\savedollar`
  // for later re-insertion. Port via state::let_i with T_MATH!().
  let_i(&T_CS!("\\savedollar"), &T_MATH!(), None);
  // aastex701.cls (TL 2025 copy, 2025/05/09) :8608-8610 `\let$\savedollar` and :8849 `\def\tabular{…\catcode`\$=\active
  // \relax…\savetabular}`: in every table `$` is active and shifts to math, so the `C`/`L`/`R` and decimal cells below can
  // let it do nothing. The catcode is set where each table's bindings run — tabular's and deluxetable's data
  // (`\startdata`, deluxetable_sty.rs) — inside the group the table opens; tabularx shares tabular's bindings here, where
  // in pdflatex it reads its body before `\tabular` and keeps the body's `$` as written. Witness 2609.05675 (an author's `$z_0$` in a `C` cell).
  let_i(&T_ACTIVE!('$'), &T_CS!("\\savedollar"), Some(Scope::Global));
  DefMacro!("\\aas@table@dollar", "\\catcode`\\$=\\active\\relax");
  Let!("\\aas@@tabular@bindings", "\\@tabular@bindings");
  DefMacro!("\\@tabular@bindings", "\\aas@table@dollar\\aas@@tabular@bindings");
  Let!("\\aas@@deluxetable@bindings", "\\@deluxetable@bindings");
  DefMacro!("\\@deluxetable@bindings", "\\aas@table@dollar\\aas@@deluxetable@bindings");

  // Decimal table conditionals — Perl L338-345
  DefConditional!("\\ifcolnumberson");
  DefConditional!("\\ifdeluxedecimals");
  DefMacro!("\\deluxedecimals", "\\global\\deluxedecimalstrue");
  RawTeX!("\\global\\deluxedecimalsfalse");
  Let!("\\decimals", "\\deluxedecimals");
  // aastex701.cls TL :11330, 11424, 11519, 11715 `\global\deluxedecimalsfalse`: each deluxetable (and its starred and
  // split forms, which all set their template here) starts without `\decimals`.
  Let!("\\aas@@set@deluxetable@template", "\\set@deluxetable@template");
  DefMacro!("\\set@deluxetable@template", "\\global\\deluxedecimalsfalse\\aas@@set@deluxetable@template");
  def_macro_noop("\\colnumbers")?;
  DefMacro!("\\deluxedecimalcolnumbers", "\\deluxedecimalstrue\\colnumbersontrue");
  Let!("\\decimalcolnumbers", "\\deluxedecimalcolnumbers");

  // Hidden column environment — Perl L374
  DefEnvironment!("{eatone}", "");

  // A `D` decimal column is two template columns (aastex701.cls TL :12010 `>\newdoit r<{\endnewdoit} @{}l`; Perl
  // aas_support.sty.ltxml:353-372, which reads `d` the same way — the class's `d`, :12011, is that pair hidden). After
  // `\decimals` (`\zdoit` is `\relax` until then, :11993, 12014-12019) a cell's first word — up to its first space
  // outside braces — splits at its first `.` outside braces (`\lookfordecimal#1#2#3#4.#5 `, `\zdoit#1 `,
  // :11980-11991): the integer part flush right in the first column,
  // the point and the fraction in the second, each part in math with `$` doing nothing
  // (`{\let$\relax\savedollar#4\savedollar}`, so a sign is a minus), the point only before a fraction (:11995-11996
  // `\ifx\xtwo\empty`), and the rest of the cell after it in the second column. Without `\decimals` the cell stays as
  // written in the first column. Perl splits with `\lx@alignment@align`, which no Perl file defines (KNOWN_PERL_ERRORS
  // #493); here the split is an alignment tab. Witness 2609.05675 (`\begin{deluxetable*}{llDDDCLll}`: 96 "Extra
  // alignment tab" errors, then Fatal TooManyErrors).
  fn build_d_columns() {
    with_building_template(|template| {
      template.add_column(Cell {
        before: Some(Tokens!(T_CS!("\\hfill"), T_CS!("\\ifdeluxedecimals"), T_CS!("\\expandafter"),
          T_CS!("\\aas@start@D@column"), T_CS!("\\fi"))),
        after: Some(Tokens!(T_CS!("\\aas@end@D@column"))),
        ..Cell::default()
      });
      template.add_column(Cell {
        after: Some(Tokens!(T_CS!("\\hfill"))),
        ..Cell::default()
      });
    });
  }
  DefColumnType!("D", { build_d_columns(); });
  DefColumnType!("d", { build_d_columns(); });
  DefMacro!("\\aas@start@D@column XUntil:\\aas@end@D@column", sub[args] {
    let cell = args[0].clone().into_tokens_result()?.unlist();
    let start = cell.iter().position(|t| t.get_catcode() != Catcode::SPACE).unwrap_or(cell.len());
    let mut depth = 0usize;
    let mut word_end = cell.len();
    let mut point = None;
    for (i, token) in cell.iter().enumerate().skip(start) {
      match token.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth = depth.saturating_sub(1),
        Catcode::SPACE if depth == 0 => {
          word_end = i;
          break;
        },
        Catcode::OTHER if depth == 0 && point.is_none() && token.with_str(|s| s == ".") => point = Some(i),
        _ => {},
      }
    }
    let (integer, fraction) = match point {
      Some(point) => (&cell[start..point], &cell[point + 1..word_end]),
      None => (&cell[start..word_end], &cell[word_end..word_end]),
    };
    let math = |part: &[Token], out: &mut Vec<Token>| {
      if !part.is_empty() {
        out.extend([T_BEGIN!(), T_CS!("\\let"), T_ACTIVE!('$'), T_CS!("\\relax"), T_CS!("\\savedollar")]);
        out.extend_from_slice(part);
        out.extend([T_CS!("\\savedollar"), T_END!()]);
      }
    };
    let mut out: Vec<Token> = Vec::new();
    math(integer, &mut out);
    out.extend(TokenizeInternal!("\\lx@add@cssclass{ltx_norightpad}").unlist());
    out.push(T_ALIGN!());
    out.extend(TokenizeInternal!("\\lx@add@cssclass{ltx_noleftpad}").unlist());
    if !fraction.is_empty() {
      out.push(T_OTHER!("."));
      math(fraction, &mut out);
    }
    out.extend_from_slice(&cell[word_end..]);
    Ok(Tokens::new(out))
  });
  def_primitive_noop("\\aas@end@D@column")?;

  // Perl aas_support.sty.ltxml L373-389: hidden-column types `h` and `B`.
  // Both wrap contents in \eatone (swallowed), producing a zero-width
  // sentinel cell. Perl L385-389 adds `B` with a TODO to "break table
  // eventually" — we match Perl's current behavior (identical to `h`).
  DefColumnType!("h", {
    with_building_template(|template| {
      template.add_column(Cell {
        before: Some(Tokens!(T_BEGIN!(), T_CS!("\\eatone"))),
        after:  Some(Tokens!(T_CS!("\\endeatone"), T_END!())),
        ..Cell::default()
      })
    });
  });
  DefColumnType!("B", {
    with_building_template(|template| {
      template.add_column(Cell {
        before: Some(Tokens!(T_BEGIN!(), T_CS!("\\eatone"))),
        after:  Some(Tokens!(T_CS!("\\endeatone"), T_END!())),
        ..Cell::default()
      })
    });
  });

  // aastex631.cls:2357-2359 (aastex701.cls TL :8857-8859) `\newcolumntype{C}{>{\bgroup\savedollar\let$\relax}c<{\savedollar
  // \egroup}}` and its `L`/`R`: math cells — the saved math shift opens and closes math around the cell — so `\pm`, `^`,
  // `_` in them are math (Perl defines none of them; it has no aastex class file to read, OXIDIZED_DESIGN_DIVERGENCES
  // #450). `$`, active in a table (`\aas@table@dollar` above), does nothing in these cells. Drivers: 2209.01632 (`{ccC}`,
  // "Extra alignment tab" while `C` was unknown), 2609.05675 (`^{\bf *}` in a `C` cell).
  let math_cell = |hfil_before: bool, hfil_after: bool| {
    let mut before = Vec::new();
    if hfil_before {
      before.push(T_CS!("\\hfil"));
    }
    before.extend([T_CS!("\\bgroup"), T_CS!("\\savedollar"), T_CS!("\\let"), T_ACTIVE!('$'), T_CS!("\\relax")]);
    let mut after = vec![T_CS!("\\savedollar"), T_CS!("\\egroup")];
    if hfil_after {
      after.push(T_CS!("\\hfil"));
    }
    with_building_template(|template| {
      template.add_column(Cell {
        before: Some(Tokens::new(before)),
        after: Some(Tokens::new(after)),
        ..Cell::default()
      })
    });
  };
  DefColumnType!("C", { math_cell(true, true); });
  DefColumnType!("L", { math_cell(false, true); });
  DefColumnType!("R", { math_cell(true, false); });

  DefMacro!("\\phn", "\\phantom{0}");
  DefMacro!("\\phd", "\\phantom{.}");
  DefMacro!("\\phs", "\\phantom{+}");
  DefMacro!("\\phm{}", "\\phantom{string}");

  DefEnvironment!("{interactive}{}{}", "#body");
  DefEnvironment!("{longrotatetable}", "#body");
  // aastex701.cls TL :12212-12231 — `rotatetable(*)` sets its table turned 90° on its own page, page layout as for
  // `longrotatetable` (2609.01052).
  DefEnvironment!("{rotatetable}", "#body");
  DefEnvironment!("{rotatetable*}", "#body");

  // 2.17.1 Celestial Objects and Data Sets
  DefConstructor!("\\objectname OptionalSemiverbatim {}",
    "<ltx:text class='ltx_ast_objectname'>#2 (catalog #1)</ltx:text>",
    enter_horizontal => true);
  Let!("\\object", "\\objectname");
  DefConstructor!("\\dataset OptionalSemiverbatim {}",
    "<ltx:text class='ltx_ast_dataset'>#2 (catalog #1)</ltx:text>",
    enter_horizontal => true);

  // 2.17.2 Ionic Species
  DefMacro!("\\ion{}{}", "{#1~\\expandafter\\uppercase\\expandafter{\\romannumeral #2}}");

  DefPrimitive!("\\sbond", "\u{2212}");
  DefPrimitive!("\\dbond", "=");
  DefPrimitive!("\\tbond", "\u{2261}");

  // 2.17.3 Fractions — Perl L435-442: \case uses a semantic text@frac constructor
  DefMacro!("\\case{}{}", "\\ensuremath{\\text@frac{#1}{#2}}");
  DefConstructor!("\\text@frac ScriptStyle ScriptStyle",
    "<ltx:XMApp><ltx:XMTok meaning='divide' role='FRACOP' mathstyle='text'/><ltx:XMArg>#1</ltx:XMArg><ltx:XMArg>#2</ltx:XMArg></ltx:XMApp>");
  Let!("\\slantfrac", "\\case");

  // 2.17.4 Astronomical Symbols
  DefPrimitive!("\\micron", "\u{00B5}m");
  DefMacro!("\\Sun", "\\sun");
  DefMacro!("\\Sol", "\\sun");
  DefPrimitive!("\\sun", "\u{2609}");
  DefPrimitive!("\\Mercury", "\u{263F}");
  DefPrimitive!("\\Venus", "\u{2640}");
  DefMacro!("\\Earth", "\\earth");
  DefMacro!("\\Terra", "\\earth");
  DefPrimitive!("\\earth", "\u{2295}");
  DefPrimitive!("\\Mars", "\u{2642}");
  DefPrimitive!("\\Jupiter", "\u{2643}");
  DefPrimitive!("\\Saturn", "\u{2644}");
  DefPrimitive!("\\Uranus", "\u{2645}");
  DefPrimitive!("\\Neptune", "\u{2646}");
  DefPrimitive!("\\Pluto", "\u{2647}");
  DefPrimitive!("\\Moon", "\u{263D}");
  DefMacro!("\\Luna", "\\Moon");
  DefPrimitive!("\\Aries", "\u{2648}");
  DefMacro!("\\VEq", "\\Aries");
  DefPrimitive!("\\Taurus", "\u{2649}");
  DefPrimitive!("\\Gemini", "\u{264A}");
  DefPrimitive!("\\Cancer", "\u{264B}");
  DefPrimitive!("\\Leo", "\u{264C}");
  DefPrimitive!("\\Virgo", "\u{264D}");
  DefPrimitive!("\\Libra", "\u{264E}");
  DefMacro!("\\AEq", "\\Libra");
  DefPrimitive!("\\Scorpius", "\u{264F}");
  DefPrimitive!("\\Sagittarius", "\u{2650}");
  DefPrimitive!("\\Capricornus", "\u{2651}");
  DefPrimitive!("\\Aquarius", "\u{2652}");
  DefPrimitive!("\\Pisces", "\u{2653}");

  DefPrimitive!("\\diameter", "\u{2300}");
  DefPrimitive!("\\sq", "\u{25A1}");

  DefPrimitive!("\\arcdeg", "\u{00B0}");
  Let!("\\degr", "\\arcdeg");
  DefPrimitive!("\\arcmin", "\u{2032}");
  DefPrimitive!("\\arcsec", "\u{2033}");
  // aastex701.cls TL :8268-8269: its `$` is the active one, so in a math (`C`/`L`/`R`, decimal) cell it does nothing and
  // elsewhere shifts to math.
  DefMacro!(T_CS!("\\nodata"), None, Tokens!(T_SPACE!(), T_ACTIVE!('~'), T_ACTIVE!('$'), T_CS!("\\cdots"),
    T_ACTIVE!('$'), T_ACTIVE!('~'), T_SPACE!()));

  // Perl L491-498: \aas@@fstack constructor — formats astronomical unit
  // superscripts. Perl computes scriptpos dynamically as
  // "mid" . $stomach->getScriptLevel — the trailing digit signals
  // SUPERSCRIPTOP nesting depth (0 at top level, 1 inside a script,
  // etc.). Rust previously hard-coded 'mid1', breaking nested usage.
  // Also pickled `font => { shape => 'upright' }` (Perl L498) — the
  // raised symbol is upright by convention regardless of the
  // surrounding italic math font.
  // Perl 98f6e5de (2025-08-12) added `sizer => '#2'` so the sizer is the
  // symbol body (e.g. `d` in `\fd`), not the whole reversion — otherwise
  // nested fstack expressions miscompute layout width.
  DefConstructor!("\\aas@@fstack Undigested {}",
    "<ltx:XMApp role='POSTFIX'>\
       <ltx:XMTok role='SUPERSCRIPTOP' scriptpos='#scriptpos'/>\
       <ltx:XMTok>.</ltx:XMTok>\
       <ltx:XMWrap>#2</ltx:XMWrap>\
     </ltx:XMApp>",
    bounded => true,
    font => { shape => "upright" },
    reversion => "#1",
    sizer => "#2",
    properties => sub[_args] {
      Ok(stored_map!("scriptpos" => s!("mid{}", get_script_level())))
    }
  );

  // Perl aas_support.sty.ltxml L499: \aas@fstack{sym} — user-facing wrapper
  // around \aas@@fstack that enforces math mode via \ensuremath. This is the
  // CS other aastex-family bindings invoke when composing astronomical-unit
  // stacks; the Rust port had only the internal \aas@@fstack DefConstructor,
  // so direct consumers of \aas@fstack hit undefined-CS.
  DefMacro!("\\aas@fstack{}", "\\ensuremath{\\aas@@fstack{#1}}");

  DefMacro!("\\fd", "\\ensuremath{\\@fd}");
  DefMacro!("\\fh", "\\ensuremath{\\@fh}");
  DefMacro!("\\fm", "\\ensuremath{\\@fm}");
  DefMacro!("\\fs", "\\ensuremath{\\@fs}");
  DefMacro!("\\fdg", "\\ensuremath{\\@fdg}");
  DefMacro!("\\farcm", "\\ensuremath{\\@farcm}");
  DefMacro!("\\farcs", "\\ensuremath{\\@farcs}");
  DefMacro!("\\fp", "\\ensuremath{\\@fp}");

  // Perl L510-517: DefMath for internal \@f* macros — astronomical unit symbols
  DefMath!("\\@fd", "\\aas@@fstack{\\fd}{d}", role => "ID", meaning => "day", alias => "\\fd");
  DefMath!("\\@fh", "\\aas@@fstack{\\fh}{h}", role => "ID", meaning => "hour", alias => "\\fh");
  DefMath!("\\@fm", "\\aas@@fstack{\\fm}{m}", role => "ID", meaning => "minute", alias => "\\fm");
  DefMath!("\\@fs", "\\aas@@fstack{\\fs}{s}", role => "ID", meaning => "second", alias => "\\fs");
  DefMath!("\\@fdg", "\\aas@@fstack{\\fdg}{\\circ}", role => "ID", meaning => "degree", alias => "\\fdg");
  DefMath!("\\@farcm", "\\aas@@fstack{\\farcm}{\\prime}", role => "ID", meaning => "arcminute", alias => "\\farcm");
  DefMath!("\\@farcs", "\\aas@@fstack{\\farcs}{\\prime\\prime}", role => "ID", meaning => "arcsecond", alias => "\\farcs");
  DefMath!("\\@fp", "\\aas@@fstack{\\fp}{p}");

  DefMacro!("\\onehalf", "\\ifmmode\\case{1}{2}\\else\\text@onehalf\\fi");
  DefPrimitive!("\\text@onehalf", "\u{00BD}");
  DefMacro!("\\onethird", "\\ifmmode\\case{1}{3}\\else\\text@onethird\\fi");
  DefPrimitive!("\\text@onethird", "\u{2153}");
  DefMacro!("\\twothirds", "\\ifmmode\\case{2}{3}\\else\\text@twothirds\\fi");
  DefPrimitive!("\\text@twothirds", "\u{2154}");
  DefMacro!("\\onequarter", "\\ifmmode\\case{1}{4}\\else\\text@onequarter\\fi");
  DefPrimitive!("\\text@onequarter", "\u{00BC}");
  DefMacro!("\\threequarters", "\\ifmmode\\case{3}{4}\\else\\text@threequarters\\fi");
  DefPrimitive!("\\text@threequarters", "\u{00BE}");

  // Photometric bands — Perl aas_support.sty.ltxml L529-533. Each takes
  // `bounded => 1, font => { shape => 'italic' }` so the italicization
  // applies only to the band glyph and not to surrounding text — without
  // bounded, an `\ubvr` mid-paragraph would italicize all subsequent text
  // until the next font reset. Match Perl on both flags.
  DefPrimitive!("\\ubvr", "UBVR", bounded => true, font => { shape => "italic" });
  DefPrimitive!("\\ub", "U\u{2000}B", bounded => true, font => { shape => "italic" });
  DefPrimitive!("\\bv", "B\u{2000}V", bounded => true, font => { shape => "italic" });
  DefPrimitive!("\\vr", "V\u{2000}R", bounded => true, font => { shape => "italic" });
  DefPrimitive!("\\ur", "U\u{2000}R", bounded => true, font => { shape => "italic" });

  // amssymb aliases
  RequirePackage!("latexsym");
  RequirePackage!("amssymb");

  Let!("\\la", "\\lesssim");
  Let!("\\ga", "\\gtrsim");

  // Nominal conversion constants — Perl L545-560
  DefMacro!("\\nomSolarEffTemp", "\\leavevmode\\hbox{\\boldmath$\\mathcal{T}^{\\rm N}_{\\mathrm{eff}\\odot}$}");
  DefMacro!("\\nomTerrEqRadius", "\\leavevmode\\hbox{\\boldmath$\\mathcal{R}^{\\rm N}_{E\\mathrm e}$}");
  DefMacro!("\\nomTerrPolarRadius", "\\leavevmode\\hbox{\\boldmath$\\mathcal{R}^{\\rm N}_{E\\mathrm p}$}");
  DefMacro!("\\nomJovianEqRadius", "\\leavevmode\\hbox{\\boldmath$\\mathcal{R}^{\\rm N}_{J\\mathrm e}$}");
  DefMacro!("\\nomJovianPolarRadius", "\\leavevmode\\hbox{\\boldmath$\\mathcal{R}^{\\rm N}_{J\\mathrm p}$}");
  DefMacro!("\\nomTerrMass", "\\leavevmode\\hbox{\\boldmath$(\\mathcal{GM})^{\\rm N}_{\\mathrm E}$}");
  DefMacro!("\\nomJovianMass", "\\leavevmode\\hbox{\\boldmath$(\\mathcal{GM})^{\\rm N}_{\\mathrm J}$}");
  DefMacro!("\\Qnom", "\\leavevmode\\hbox{\\boldmath$\\mathcal{Q}^{\\rm N}_{\\odot}$}");
  Let!("\\Qn", "\\Qnom");
  DefMacro!("\\nom{}", "\\leavevmode\\hbox{\\boldmath$\\mathcal{#1}^{\\rm N}_{\\odot}$}");
  DefMacro!("\\Eenom{}", "\\leavevmode\\hbox{\\boldmath$\\mathcal{#1}^{\\rm N}_{Ee}$}");
  DefMacro!("\\Epnom{}", "\\leavevmode\\hbox{\\boldmath$\\mathcal{#1}^{\\rm N}_{Ep}$}");
  DefMacro!("\\Jenom{}", "\\leavevmode\\hbox{\\boldmath$\\mathcal{#1}^{\\rm N}_{Je}$}");
  DefMacro!("\\Jpnom{}", "\\leavevmode\\hbox{\\boldmath$\\mathcal{#1}^{\\rm N}_{Jp}$}");

  // 2.17.5 Hypertext — Perl L563-577
  // Perl L565: RequirePackage('url') — re-required here alongside the
  // hypertext definitions so `\url{}` is guaranteed loaded before
  // `\anchor`/`\@@email`/etc. AAS-macros that route URL content. The
  // package loader no-ops a re-require, so this is a faithful
  // transcription, not a repeated load.
  RequirePackage!("url");
  DefConstructor!("\\anchor Semiverbatim Semiverbatim", "<ltx:ref href='#1'>#2</ltx:ref>",
    enter_horizontal => true);
  DefConstructor!("\\@@email Semiverbatim", "<ltx:ref href='mailto:#1'>#1</ltx:ref>",
    enter_horizontal => true);

  // Misc
  DefMacro!("\\eqsecnum",
    "\\@addtoreset{equation}{section}\\def\\theequation{\\arabic{section}-\\arabic{equation}}");

  def_macro_noop("\\singlespace")?;
  def_macro_noop("\\doublespace")?;
  def_macro_noop("\\tighten")?;
  def_macro_noop("\\tightenlines")?;
  def_macro_noop("\\nohyphenation")?;
  def_macro_noop("\\offhyphenation")?;
  def_macro_noop("\\ptlandscape")?;
  def_macro_noop("\\refpar")?;
  def_macro_noop("\\traceoutput")?;
  def_macro_noop("\\tracingplain")?;

  def_macro_noop("\\noprint {}")?;
  DefMacro!("\\figsetstart", "{\\bf Fig. Set}");
  def_macro_noop("\\figsetend")?;
  def_macro_noop("\\figsetgrpstart")?;
  def_macro_noop("\\figsetgrpend")?;
  DefMacro!("\\figsetnum {}", "{\\bf #1.}");
  DefMacro!("\\figsettitle {}", "{\\bf #1}");
  def_macro_noop("\\figsetgrpnum {}")?;
  def_macro_noop("\\figsetgrptitle {}")?;
  def_macro_noop("\\figsetplot {}")?;
  def_macro_noop("\\figsetgrpnote {}")?;
});

/// An AASTeX panel's properties: its width, as a sub-figure's, and the gridline row it stands in, if any.
fn aas_panel_props(args: &[Option<Digested>]) -> Result<SymHashMap<Stored>> {
  let mut props = subcaption_width_props(args)?;
  let row = lookup_int("lx_aas_gridrow");
  if row > 0 {
    props.insert("gridrow", Stored::from(row.to_string()));
  }
  Ok(props)
}

/// AASTeX's `\fig`-family dispatch, after its file argument and a peek at the next token (pushed back for the target
/// to read): a brace opens the panel form `panel`; anything else makes it a `\ref` (astro-ph/0003209).
fn aas_fig_dispatch(arg: Tokens, test: Token, panel: &str) -> Result<Tokens> {
  // Push order is reversed for stack semantics: last unread is first read.
  unread_one(test);
  unread_one(T_END!());
  unread_vec(arg.unlist());
  unread_one(T_BEGIN!());
  Ok(if test.get_catcode() != Catcode::BEGIN {
    Tokens!(T_CS!("\\ref"))
  } else {
    Tokens!(T_CS!(panel))
  })
}
