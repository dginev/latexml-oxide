use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: revtex4_support.sty.ltxml — 433 lines
  // Support macros for RevTeX4 class (APS journals)

  RequirePackage!("hyperref");
  // The real revtex4-1/4-2.cls never load hyperref, they only test for it (revtex4-2.cls:476); Perl preloads it here
  // (revtex4_support.sty.ltxml:23), so a document's own `\usepackage{hyperref}` did nothing — and phfnote's
  // `hyperrefdefs=defer` (phfnote.sty:829-860), which `\let`s `\hypersetup` to a deferred copy expecting that later
  // load to restore it, then re-added its `\AtBeginDocument` code for ever from inside the hook (KNOWN_PERL_ERRORS
  // #997; 2207.13707, 2403.04828, 2508.03993, 2508.03994, 2601.20155). The preload stays, for the documents that use
  // hyperref's commands without loading it, but marked unloaded so the document's own load runs the binding again
  // (the `amsmath` precedent of aa_cls.rs, Perl aa.cls.ltxml:57).
  AssignValue!("hyperref.sty_loaded" => Stored::None, Some(Scope::Global));
  RequirePackage!("natbib");
  RequirePackage!("revsymb");
  RequirePackage!("url");
  RequirePackage!("longtable");
  RequirePackage!("dcolumn");

  // ltxutil boolean-switch infrastructure. The real revtex4.cls is a generated
  // monolith that bundles `ltxutil` (ltxutil.sty L146-205), so these low-level
  // switch macros are available to any class built on revtex4. Our revtex4
  // binding REPLACES revtex4.cls with LaTeXML constructs and so never pulled
  // them in — but derived classes use them DIRECTLY in their own bodies. E.g.
  // AIAA.cls does `\LoadClass{revtex4}` then `\@ifxundefined\twoside@sw{...}{}`
  // (AIAA.cls L97), which left `\@ifxundefined`/`\@booleanfalse`/`\twoside@sw`
  // undefined. Ported verbatim from ltxutil.sty so revtex4-derived local
  // classes load. (Perl skips this by treating such a local .cls as a missing
  // file and falling back to article; we resolve the bundled .cls — as real
  // LaTeX does — so we must provide its expected infrastructure.)
  // Witness: 1904.07479 (AIAA.cls). Format-version probing (\IfFormatAtLeastTF)
  // is omitted — not needed for the switch semantics.
  // revtex4.cls:4476, revtex4-1.cls:2378, revtex4-2.cls:2399 `\sanitize@url`: `%`, `~` and `_` read as other
  // characters, so an address keeps them. Classes and styles built on revtex call it for their own address
  // commands: JASA.cls, ujp.sty, cej.cls (witnesses 2006.04679, 1702.04290, 1510.07897, 1309.5259,
  // 2508.01050, 2103.00255).
  TeX!(r"\def\sanitize@url{\@makeother\%\@makeother\~\@makeother\_}");
  TeX!(r"\long\def\true@sw#1#2{#1}%
\long\def\false@sw#1#2{#2}%
\def\@boolean#1#2{\long\def#1{#2\expandafter\true@sw\else\expandafter\false@sw\fi}}%
\def\@boole@def#1#{\@boolean{#1}}%
\def\@booleantrue#1{\let#1\true@sw}%
\def\@booleanfalse#1{\let#1\false@sw}%
\@boole@def\@ifx#1{\ifx#1}%
\@boole@def\@ifx@empty#1{\ifx\@empty#1}%
\@boole@def\@if@empty#1{\if!#1!}%
\long\def\@ifxundefined#1{\@ifx{\undefined#1}}%
\long\def\@ifnotrelax#1#2#3{\@ifx{\relax#1}{#3}{#2}}%
\def\@ifxundefined@cs#1{\expandafter\@ifx\expandafter{\csname#1\endcsname\relax}}%
\def\@if@sw#1#2{#1\expandafter\true@sw\else\expandafter\false@sw#2}%
\long\def\@argswap#1#2{#2#1}%
\long\def\@argswap@val#1#2{#2{#1}}%
\@boole@def\@ifdim#1{\ifdim#1}%
\@boole@def\@ifhmode{\ifhmode}%
\@boole@def\@ifmmode{\ifmmode}%
\@boole@def\@ifnum#1{\ifnum#1}%
\@boole@def\@ifodd#1{\ifodd#1}%");

  // 4.3 Title/Author (Perl PR #2767)
  // 68t: a call inside the document's `{titlepage}` also runs a refused printing `\title` there (sect05.rs
  // `titlepage_class_call`; gr-qc9404034 under `\documentstyle{revtex}`).
  // The class's refused `\title` runs first for its side effects, as under the kernel's (sect05.rs `\title`,
  // OXIDIZED_DESIGN #482: the cvpr template's `\thetitle`; no witness of this binding's own).
  DefMacro!("\\title[]{}",
    "\\lx@titlepage@class@call{title}{#1}{#2}{\\lx@replay@class@setter{title}{#1}{#2}\\gdef\\@shorttitle{#1}\\gdef\\@title{#2}\\ifx.#1.\\else\\lx@add@toctitle{#1}\\fi\\lx@add@title{#2}}");

  // \author[labels]{name}   One \author per author
  // If labels given, the corresponding affiliation from \affil is attached
  // otherwise, \author should be followed by \affiliation
  // `\author{Brent Preston and Eric Poisson}` names two (gr-qc0606093): the article `\author`'s parse, each author
  // with the labels (Perl revtex4_support makes it one author; 63e).
  // `\lx@author@after` (sect05.rs): what a package appends to `\author` (titling's `\theauthor`) runs here too.
  DefMacro!("\\author[]{}",    "\\lx@add@authors@append[annotations={#1}]{#2}\\lx@author@after{#2}");
  // An `\affiliation` that starts with a letter or number mark (all `\author{Name$^{a}$}`s first, then `$^{a}$`-marked
  // lines: 2011.01984, 2301.08449) is labelled by its marks under marked authors, and goes to the authors showing them
  // (relocate_annotations); any other is revtex's group rule, every author of the group before it (`annotate=run`: the
  // run of authors with none yet, and each further line before a new author the same run's — revtex4-2.cls
  // `\move@AU\move@AF`; Perl's `annotate=new`, Base_Utility.pool.ltxml:521, gave a group's second line to its last
  // author only, KNOWN_PERL_ERRORS #1190, 1303.7097): a superscript inside the line (`Laboratory for $^{3}$He`) or a
  // symbol (`$^{\dagger}$`) is no mark, and an interleaved `\author{A$^{1}$}\affiliation{$^{1}$…}` group keeps the
  // group rule, exact there (63i review).
  // (a one-mark line the group takes is declared so at the call, `_grouped=true`, which the frontmatter observer's
  // reading honours: today's run, not the marks — FM S2b, 63i review r2; a line holding several marked entries is a
  // marked list, cut by its marks however today's places it, and so are the lines of names given after an affiliation
  // that came before the first `\author`: 2209.05548's empty `\affiliation{}`, 2510.16137, 2412.01183)
  DefMacro!("\\affiliation{}", sub[(content)] {
    let marked = leads_with_mark(&content);
    let grouped = marked && queued_group_shares_the_mark(&content);
    let declared_grouped =
      grouped && content_marks(&content).len() == 1 && interleaved_group_shares_the_mark(&content);
    let attr = Some(mouth::tokenize_internal(if declared_grouped {
      "annotate=run,_grouped=true"
    } else {
      "annotate=run"
    }));
    let cs = if marked && !grouped {
      T_CS!("\\lx@add@affiliation@marked")
    } else {
      T_CS!("\\lx@add@affiliation")
    };
    Ok(Invocation!(cs, vec![attr, Some(content)]))
  });
  // revtex4-1.cls:2147-2250, the internals of `\affiliation` (`\move@AU\move@AF\begingroup … \@affiliation`,
  // :2206-2211): `\move@AU`/`\move@AF` file the pending authors and affiliations, which the frontmatter API attaches
  // at once, and `\@affiliation{…}` (:2213) closes the group and records the affiliation. A class that redefines
  // `\affiliation` with them keeps its affiliations: openjournal.cls:433-437, read raw on revtex4-1 (33 2609 papers,
  // 2609.05602, 2609.07025).
  def_macro_noop("\\move@AU")?;
  def_macro_noop("\\move@AF")?;
  DefMacro!("\\@affiliation{}", "\\endgroup\\lx@add@affiliation[annotate=run]{#1}");
  DefMacro!("\\noaffiliation", "\\lx@add@affiliation[annotate=run]{}");
  DefMacro!("\\affil OptionalSemiverbatim {}",
    "\\lx@add@affiliation[annotate={\\ifx.#1.new\\else 1\\fi},label={#1}]{#2}");
  // revtex4-2.cls:7519 (revtex4.cls:5964, revtex4-1.cls:7498): the obsolete `\address` becomes `\affiliation`, its
  // mark-led lines going to the names showing their marks (Perl's `\lx@add@address` had no mark path, 0806.3762),
  // as `\altaddress` becomes `\altaffiliation` below.
  RawTeX!(r"\def\address{\replace@command\address\affiliation}");
  // These add contacts to most recent author
  // The optional arguments here are a sort of prefix to the footnote. (NOT label)
  // REVTeX 4.1/4.2 set the address with `\href`, without hyperref `\@href`: in the current font
  // (revtex4-1.cls:1959-1961, 2383-2391), its `_` as typed (revtex4_cls.rs sets REVTeX 4.0's in typewriter type;
  // 2301.03570, 2301.03067)
  // revtex4-1.cls:2383-2392 (revtex4.cls:4481-4491, revtex4-2.cls:2404-2413): `\email` is `\begingroup\sanitize@url
  // \@email{#1}`, so the address is read with `%` other, as typed (`UrlSemiverbatim`):
  // `\homepage{http://www.nhn.ou.edu/%7Emilton}` (0712.3811) is the address, where a `%` comment swallowed the rest
  // of the document, every `\label` with it (KNOWN_PERL_ERRORS #1149). One call `\email[]{}`, as an author-block line
  // reads it (`perfect_kernel_batch64::email_command_option_on_author_line`).
  DefMacro!("\\email[] UrlSemiverbatim",    "\\lx@add@email[name={#1}]{\\lx@address@textfont{#2}}");
  DefMacro!("\\homepage[] UrlSemiverbatim", "\\lx@add@url[name={#1}]{\\lx@address@textfont{#2}}");
  // `\@email{label}{address}` closes the class's group. A class that re-dresses `\email` over them (cej.cls:88
  // `\renewcommand*\email[1][E-mail: ]{\begingroup\sanitize@url\@email{#1}}`; 0710.1022, 1201.4230 and 5 more) left
  // the group open without them, `%` other to the end of the document.
  DefMacro!("\\@email{} Semiverbatim",    "\\endgroup\\lx@add@email[name={#1}]{\\lx@address@textfont{#2}}");
  DefMacro!("\\@homepage{} Semiverbatim", "\\endgroup\\lx@add@url[name={#1}]{\\lx@address@textfont{#2}}");
  DefMacro!("\\thanks[]{}",              "\\lx@add@contact[role=thanks,name={#1}]{#2}");
  // revtex4-2.cls:2472-2493 (revtex4.cls, revtex4-1.cls alike): the front-matter footnote under `\thanks` and the
  // `\author`'s optional notes, a mark shared by identical texts; documents call it for a note on several authors
  // (`\newcommand{\equalcontrib}{\frontmatter@footnote{…contributed equally…}}`, 2605.04155). Each author keeps it.
  DefMacro!("\\frontmatter@footnote{}",  "\\lx@add@contact[role=thanks]{#1}");
  // revtex4-2.cls:3040-3043 (revtex4-1.cls:3019-3022, revtex4.cls:5020-5022): the front matter's footnote marks — the
  // footnote counter's symbols (`\@fnsymbol`), its letters (`\@alph`) under `altaffilletter` — declared as the title
  // block's (`latex_constructs::TITLE_THEFOOTNOTE`): the marks the notes of the authors, the title and the date show, as
  // the PDF prints them, and the ones the frontmatter observer labels, so an affiliation the authors' footnote symbols
  // mark finds its names (`\author{Ann\footnote{…}}` "Ann∗", `\affiliation{$^{\ast,\ddagger}$Univ A}`: 1608.06509).
  DefMacro!("\\frontmatter@thefootnote", sub[_args] {
    let style = if lookup_bool("revtex@altaffilletter") { "\\@alph" } else { "\\@fnsymbol" };
    Ok(Tokens!(T_CS!(style), T_BEGIN!(), T_CS!("\\csname"), T_LETTER!("c"), T_OTHER!("@"), T_CS!("\\@mpfn"), T_CS!("\\endcsname"), T_END!()))
  });
  Let!("\\lx@title@thefootnote", "\\frontmatter@thefootnote");
  // revtex4-2.cls:2120/2160/2236 `\@AF@join{note}`: the join every `\email`, `\homepage`, `\thanks` and
  // `\altaffiliation` ends in (:2404-2421), dispatched to the title, author or affiliation last given. A class that
  // re-dresses `\@email` calls it directly (JASA.cls:1232-1238 `\def\@email#1#2{\endgroup\@AF@join{#1\href{mailto:#2}
  // {#2}}}`; 2605.18495, 2605.23416: `\@AF@join` undefined). The note goes where our `\thanks` puts its notes, on the
  // last author, whatever revtex's dispatch would join it to, and as a thanks note even when it holds an email: the
  // content and its link are kept.
  DefMacro!("\\@AF@join{}",              "\\lx@add@contact[role=thanks]{#1}");
  DefMacro!("\\orcid[]{}",               "\\lx@add@orcid[name={#1}]{#2}"); // ?

  DefMacro!("\\collaboration{}",    "\\author{#1}");
  // revtex4-2.cls:2420 `\newcommand*\altaffiliation[2][]{\@AF@join{#1#2}}` (revtex4-1.cls, revtex4.cls alike): the
  // author's own note, joined to the author last given as `\email` and `\thanks` are — not one of the group's
  // affiliations `\affiliation` files (`\move@AU\move@AF`), so it neither reaches the authors before it nor ends the
  // group's run: role `altaffiliation`, as aastex's binding has it. Perl's `annotate=new,role=affiliation`
  // (revtex4_support.sty.ltxml:61) cost the author before it the group's affiliation (cond-mat0209262; 78 of 5,934
  // revtex papers write an `\altaffiliation` inside a group).
  DefMacro!("\\altaffiliation[]{}", "\\lx@add@contact[role=altaffiliation,name={#1}]{#2}");

  def_macro_noop("\\firstname")?;
  DefConstructor!("\\surname{}", "#1", enter_horizontal => true);

  // 4.4 Abstract
  DefMacro!("\\abstractname", "Abstract");

  DefMacro!("\\pacs{}",     "\\lx@add@classification[scheme=pacs,name={PACS:~}]{#1}");
  DefMacro!("\\keywords{}", "\\lx@add@keywords{#1}");
  // `\preprint` stores the number for the title page, which prints it or not by the class family's rule, so the branch
  // is chosen before the argument is read (`preprint_printed`). A printed number is Perl's
  // `\lx@add@pubnote[role=preprint]{#1}` (revtex4_support.sty.ltxml:71), its argument read as TeX reads it (`$`, `~`
  // are math and a tie, as on the page: "Fe$_{2}$YZ", "DESY~11--210"; a text `_` errs as in pdflatex). One the class
  // does not print is metadata, an `ltx:rdf` identifier per `\\`-separated number (user rulings 2026-10-06,
  // 2026-10-09: a display the class toggles off is not shown, and what it keeps unprinted is RDFa), read as written
  // and set in the identity encoding (`\lx@address@textfont`), so its `_` stays as typed: Perl's digestion raised
  // "Script _" on `DMRG_Sampled_Variance` (1205.1463, 1207.6705, 1912.04576, 2110.00437, 2408.13616;
  // KNOWN_PERL_ERRORS #990, OXIDIZED_DESIGN_DIVERGENCES #770). REVTeX 3 decides it in TeX (revtex3_support_sty.rs).
  DefMacro!("\\preprint", sub[_args] {
    Ok(Tokens!(if preprint_printed() {
      T_CS!("\\lx@revtex@preprint@printed")
    } else {
      T_CS!("\\lx@revtex@preprint@identifier")
    }))
  });
  DefMacro!("\\lx@revtex@preprint@printed{}", "\\lx@add@pubnote[role=preprint]{#1}");
  // The numbers are the argument's `\\`-separated items at its top level: a `\\` in a group or an environment
  // (`\parbox{2in}{A-1\\B-2}`, a `{flushright}` block in a `{minipage}`, FERMILAB's layout) belongs to it.
  DefMacro!("\\lx@revtex@preprint@identifier Semiverbatim", sub[(number)] {
    let number = number.unlist();
    let mut out: Vec<Token> = Vec::new();
    for item in split_top_level_lines(&number) {
      let start = item.iter().take_while(|t| t.get_catcode() == Catcode::SPACE).count();
      let end = item.len() - item.iter().rev().take_while(|t| t.get_catcode() == Catcode::SPACE).count();
      if start >= end {
        continue;
      }
      out.extend(
        mouth::tokenize_internal("\\lx@add@frontmatter{ltx:rdf}[property=dcterms:identifier]{\\lx@address@textfont").unlist(),
      );
      out.push(T_BEGIN!());
      out.extend(item[start..end].iter().copied());
      out.extend([T_END!(), T_END!()]);
    }
    Ok(Tokens::new(out))
  });

  // Extra
  def_macro_noop("\\blankaffiliation")?;
  DefMacro!("\\checkindate", "\\today");

  DefMacro!("\\received[]{}", "\\lx@add@date[role=received]{#2}");
  DefMacro!("\\revised[]{}", "\\lx@add@date[role=revised]{#2}");
  DefMacro!("\\accepted[]{}", "\\lx@add@date[role=accepted]{#2}");
  DefMacro!("\\published[]{}", "\\lx@add@date[role=published]{#2}");

  // 5.3 Widetext
  // A wide passage is set in one column (ltxgrid's `\onecolumngrid` inside it, revtex4-2.cls:7440-7460): the page a
  // float's panels are set in rows against follows (the panel grid rule, `set_panel_page_twocolumn`; 2504.20901's
  // appendix figures in `{widetext}`).
  DefMacro!("\\widetext", "\\lx@panel@page@push@onecolumn");
  DefMacro!("\\endwidetext", "\\lx@panel@page@pop");
  def_macro_noop("\\narrowtext")?;
  def_macro_noop("\\endnarrowtext")?;
  def_macro_noop("\\mediumtext")?;
  def_macro_noop("\\endmediumtext")?;

  // 5.5 Acknowledgements — Perl revtex4_support.sty.ltxml L100-106.
  // Perl: DefConstructor('\acknowledgments', "<ltx:acknowledgements name='#name'>",
  //   properties => sub { (name => Digest(T_CS('\acknowledgmentsname'))); });
  // ltx:acknowledgements Tag (autoClose + inlist=toc) is global — set in
  // latex_constructs.rs (arXiv-fork 23771504 removed the binding-local copies).
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements name='#name'>",
    properties => { Ok(stored_map!("name" => digest_in_group(T_CS!("\\acknowledgmentsname"))?)) });
  // Tolerant close — see omnibus_cls.rs commentary on the
  // \begin{acknowledgments} ... \bibliography ... \end{acknowledgments}
  // pattern that auto-closes the env via \bibliography opening
  // <ltx:bibliography>. Driver: 2202.04803 R=1 → R=0.
  DefConstructor!("\\endacknowledgments", sub[document, _whatsit, _props] {
    let cur = document.get_node().clone();
    let has_open = document.findnode("ancestor-or-self::ltx:acknowledgements", Some(&cur)).is_some();
    if has_open {
      document.close_element("ltx:acknowledgements")?;
    }
  });
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  Let!("\\acknowledgements", "\\acknowledgments");
  Let!("\\endacknowledgements", "\\endacknowledgments");

  // Section numbering style
  DefMacro!("\\thesection", "\\Roman{section}");
  // revtex4.cls:5367, revtex4-1.cls:6759, revtex4-2.cls:6780 and REVTeX 3's aps.sty:636 number tables in capital
  // Roman, "TABLE I"; the caption names stay Perl's "Figure"/"Table" (user ruling 2026-10-21, 18). Perl's binding
  // (revtex4_support.sty.ltxml:111) sets only `\thesection`: KNOWN_PERL_ERRORS #1077. 0708.2630, 0902.4794,
  // 1911.02024, cond-mat9610218, cond-mat0408054. Guard `perfect_kernel_batch78n::revtex_tables_are_roman`.
  DefMacro!("\\thetable", "\\Roman{table}");

  // Grid / column macros
  DefMacro!("\\thepagegrid", "one");
  // (a one-column region of a declared two-column page, the panel grid rule: `set_panel_page_twocolumn`)
  DefMacro!("\\onecolumngrid", "\\lx@panel@page@onecolumn");
  DefMacro!("\\twocolumngrid", "\\lx@panel@page@twocolumn");
  DefMacro!("\\restorecolumngrid", "\\lx@panel@page@restore");
  // revtex4-1.cls L4388: \do@columngrid{layout}{N}. Layout-only.
  // Witness 2406.02666 (revtex4-1 with explicit \onecolumngrid call
  // before our stub binding loads).
  def_macro_noop("\\do@columngrid{}{}")?;
  // revtex4-2.cls:7424 `\close@column@grid` ends the current column grid before
  // a page break; layout-only. A paper's own `\balancecolsandclearpage`
  // (`\close@column@grid\clearpage\twocolumngrid`) calls it: arXiv 2605.07942.
  def_macro_noop("\\close@column@grid")?;
  // ltxgrid.sty:1403-1418 (revtex4.cls, revtex4-1.cls, revtex4-2.cls:4496-4510): `\onecolumngrid@push` switches to one
  // column for a `figure*`/`table*` and `\onecolumngrid@pop` restores the grid — layout only, as `\onecolumngrid`.
  // Papers bracket their wide floats with them (`\makeatletter\onecolumngrid@push\makeatother\begin{table*}`; run-347
  // inventory 16 + 28 papers; 2311.06373, 2104.14555, 2004.00671, 2206.03658).
  def_macro_noop("\\onecolumngrid@push")?;
  def_macro_noop("\\onecolumngrid@pop")?;
  // ltxgrid.sty:1713 `\set@vsize`, the page's text height — layout only.
  def_macro_noop("\\set@vsize")?;
  def_primitive_noop("\\twocolumn")?;
  // The angle is a decimal (graphics.sty:445-452 → trig.sty:55 `\dimen@=#1\p@`):
  // a `{Number}` put the `.5` of `{22.5}` back in the input (OXIDIZED_DESIGN #317).
  DefConstructor!("\\rotatebox{Float}{}", "#2", enter_horizontal => true);
  def_macro_noop("\\pagesofar")?;

  // Endnotes — Perl revtex4_support.sty.ltxml L120-149.
  // For each constructor: if optional arg #1 is present, mark = arg1;
  // else RefStepCounter('endnote') AND mark = DigestText(\theendnote).
  // The make_note_tags helper in latex_constructs.rs implements exactly
  // this dispatch (mark_opt + tag_opt=None branch).
  NewCounter!("endnote");
  DefConstructor!("\\endnote[]{}", "<ltx:note role='endnote' mark='#mark' xml:id='#id'>#tags#2</ltx:note>",
    mode => "internal_vertical",
    sizer => sub[whatsit] { note_size_in_line(whatsit) },
    before_digest => { neutralize_font(); },
    properties => sub[args] {
      make_note_tags("endnote", args[0].as_ref(), None)
    });
  DefConstructor!("\\endnotemark[]", "<ltx:note role='endnotemark' mark='#mark' xml:id='#id'>#tags</ltx:note>",
    mode => "restricted_horizontal", enter_horizontal => true,
    sizer => sub[whatsit] { note_size_in_line(whatsit) },
    before_digest => { neutralize_font(); },
    properties => sub[args] {
      make_note_tags("endnote", args[0].as_ref(), None)
    });
  DefConstructor!("\\endnotetext[]{}", "<ltx:note role='endnotetext' mark='#mark' xml:id='#id'>#2</ltx:note>",
    mode => "internal_vertical",
    sizer => sub[_whatsit] { Ok(out_of_line_size()) },
    before_digest => { neutralize_font(); },
    properties => sub[args] {
      // Perl L143-149: mark = arg1 OR Digest(\theendnote). (No RefStepCounter.)
      let arg1 = args[0].as_ref();
      let mark = match arg1 {
        Some(m) => m.clone(),
        None => digest(T_CS!("\\theendnote"))?,
      };
      Ok(stored_map!("mark" => mark))
    });

  // 6. Math — Perl L159-176
  Let!("\\case", "\\frac");
  Let!("\\slantfrac", "\\frac");
  // Perl L161-162 passes `locked => 1` so RevTeX's \text isn't silently
  // replaced by amsmath's \text (which would miss the restricted_hmode
  // treatment RevTeX relies on for inline-text-in-math spacing).
  // revtex4-2.cls:7547-7560 `\REV@text` sets `#1` in `\hbox{{…}}` (math) or `\mbox{#1}` (text): its argument is
  // digested inside box braces, so the alignment count is raised for it, as amstext_sty.rs `\text`
  // (KNOWN_PERL_ERRORS #931).
  DefConstructor!("\\text{}", "<ltx:text _noautoclose='true'>#1</ltx:text>",
    mode => "restricted_horizontal", locked => true,
    before_digest => { increment_align_group_count(); },
    after_digest => { decrement_align_group_count(); });
  // revtex4-2.cls:7547-7585 (revtex4.cls:5992, revtex4-1.cls:7526): REVTeX's `\text` is `\REV@text`, which apsrev's
  // `.bbl` notes write out (`$t'=R(t)\protect \REV@text { mod } 1$`; 2202.11665, 2107.02243, 1407.5317).
  Let!("\\REV@text", "\\text");

  // RevTeX3 bold math (obsolete in RevTeX4) — Perl L165-171. RevTeX 4.x's own `\bm` (revtex4-2.cls:7573-7586,
  // revtex4-1.cls:7552-7565, revtex4.cls:6018-6032) asks for bm.sty ("To use \bm, please load the bm package!"),
  // drops its first argument and leaves later ones in a `\relax{…}` group; Perl keeps a bold `\bm`, its argument in a
  // group: a `\left` without its `\right` there is set at the outer level with a Warning (bm_sty.rs
  // `define_grouped_bm`, OXIDIZED_DESIGN #544).
  // REVTeX 4.1/4.2 define it at load (revtex4-1.cls:7565, revtex4-2.cls:7586 `\let\bm\REV@bm`), so a preamble's
  // `\newcommand{\bm}` errs "Command \bm already defined", as here; REVTeX 4.0 provides it at `\begin{document}`
  // (revtex4_cls.rs `defer_bm_to_begin_document`; KNOWN_PERL_ERRORS #965).
  bm_sty::define_grouped_bm()?;
  // Perl L166-168: `locked => 1` keeps \bbox bold-wrapped even when a
  // user or co-loaded package redefines it.
  DefConstructor!("\\bbox{}", "#1", bounded => true, require_math => true,
    font => { forcebold => true }, locked => true);
  // Perl revtex4_support.sty.ltxml L169-171: \pmb wraps content in
  // forcebold + family=blackboard + series=medium + shape=upright.
  DefConstructor!("\\pmb{}", "#1", bounded => true, require_math => true,
    font => { forcebold => true, family => "blackboard",
      series => "medium", shape => "upright" });
  // Perl revtex4_support.sty.ltxml L172:
  //   DefMacro('\eqnum {}',
  //     '\lx@equation@settag{\edef\theequation{#2}\lx@make@tags{equation}}',
  //     locked => 1);
  // The Perl body has a known bug — `#2` is out-of-range for a 1-arg macro
  // (KNOWN_PERL_ERRORS.md #15) — so it always tags the equation with the
  // counter default and silently drops the user-supplied label. Rust's
  // empty body is semantically equivalent to Perl's broken effect (same
  // dropped-label outcome) without re-implementing the buggy `#2` lookup.
  // The `locked=>true` flag is independent of the body and IS load-bearing:
  // it prevents a downstream class (revtex3_support? a sibling APS .cls?)
  // from `\renewcommand`-ing \eqnum into something that re-introduces a
  // tag-conflict. Match Perl on the lock.
  DefMacro!("\\eqnum{}", "", locked => true);
  def_macro_noop("\\mathletters")?;
  def_macro_noop("\\endmathletters")?;

  // Citations
  DefMacro!("\\onlinecite", "\\citealp");
  Let!("\\textcite", "\\citet");
  // revtex4-1/4-2 substyle bbls reference internal cite helpers
  // (`\rev@citealp`, `\rev@citealpnum`, `\rev@citet`, `\rev@citemark`)
  // that are normally let-aliased from natbib. Map them to natbib
  // equivalents so .bbl files referencing them resolve cleanly.
  // Witness 2412.13042 (revtex4-2 + main.bbl using \rev@citealp).
  Let!("\\rev@citealp",     "\\citealp");
  // revtex4-1.cls:6939-6945 (revtex4-2.cls:6961): `\citealp` with `\@cite` let to `\NAT@citenum`, the bare number
  // — natbib's `\citenum`; there is no `\citealpnum`. Witnesses 1002.2610, 1511.04077, 1903.06404 (their .bbl).
  Let!("\\rev@citealpnum",  "\\citenum");
  Let!("\\rev@citet",       "\\citet");
  Let!("\\rev@citenum",     "\\citenum");
  Let!("\\rev@citemark",    "\\citenum");

  // 8. Citations and References — Perl revtex4_support.sty.ltxml L190-204
  // RevTeX3; obsolete for RevTeX4 (but semi-implemented there). Should be a
  // simple environment, but tends to be misused, so define separately.
  DefConstructor!("\\references",
    "<ltx:bibliography xml:id='#id' bibstyle='#bibstyle' citestyle='#citestyle' sort='#sort'>\
       <ltx:title font='#titlefont' _force_font='true'>#title</ltx:title>\
       <ltx:biblist>",
    before_digest => {
      engine::latex_constructs::before_digest_bibliography()
    },
    after_digest => sub[whatsit] {
      engine::latex_constructs::begin_bibliography(whatsit)?;
    },
    locked => true
  );
  DefConstructor!("\\endreferences",
    sub[document, _whatsit, _props] {
      document.maybe_close_element("ltx:biblist")?;
      document.maybe_close_element("ltx:bibliography")?;
    },
    locked => true
  );

  // 10. Tables — Perl revtex4_support.sty.ltxml L215-245.
  // {quasitable} re-Lets tabular → longtable inside its body so that
  // an embedded \begin{tabular}...\end{tabular} actually digests as
  // \longtable (which can break across pages). Without these Lets,
  // a quasitable degrades to a plain tabular and loses page-break
  // capability — the entire reason the env exists.
  DefEnvironment!("{ruledtabular}", "#body");
  DefEnvironment!("{quasitable}", "#body",
    before_digest => {
      Let!(T_CS!("\\begin{tabular}"), T_CS!("\\begin{longtable}"));
      Let!(T_CS!("\\end{tabular}"), T_CS!("\\end{longtable}"));
      Let!("\\tabular", "\\longtable");
      Let!("\\endtabular", "\\endlongtable");
    });
  def_macro_noop("\\squeezetable")?;
  DefMacro!("\\toprule", "\\hline\\hline");
  DefMacro!("\\colrule", "\\hline");
  DefMacro!("\\botrule", "\\hline\\hline");
  def_macro_noop("\\frstrut")?;
  def_macro_noop("\\lrstrut")?;
  Let!("\\tableftsep", "\\tabcolsep");
  Let!("\\tabmidsep", "\\tabcolsep");
  Let!("\\tabrightsep", "\\tabcolsep");
  Let!("\\tablenote", "\\footnote");
  Let!("\\tablenotemark", "\\footnotemark");
  Let!("\\tablenotetext", "\\footnotetext");
  Let!("\\tableline", "\\colrule");
  RawTeX!("\\newcolumntype{d}{D{.}{.}{-1}}");

  // Floats
  def_primitive_noop("\\printfigures")?;
  def_primitive_noop("\\printtables")?;
  def_macro_noop("\\oneapage")?;
  def_macro_noop("\\printendnotes")?;

  // Turnpage
  DefEnvironment!("{turnpage}", "#body");

  // Extra
  DefMacro!("\\MakeTextLowercase", "\\lowercase");
  DefMacro!("\\MakeTextUppercase", "\\uppercase");
  def_macro_noop("\\NoCaseChange")?;

  // Macro & control stubs — Perl L280-295
  def_macro_noop("\\absbox")?;
  def_macro_noop("\\addstuff{}{}")?;
  def_macro_noop("\\appdef{}{}")?;
  def_macro_noop("\\gappdef{}{}")?;
  def_macro_noop("\\prepdef{}{}")?;
  def_macro_noop("\\lineloop{}")?;
  def_macro_noop("\\loopuntil{}")?;
  def_macro_noop("\\loopwhile{}")?;
  def_macro_noop("\\traceoutput")?;
  def_macro_noop("\\tracingplain")?;
  def_macro_noop("\\removephantombox")?;
  def_macro_noop("\\removestuff")?;
  def_macro_noop("\\replacestuff{}{}")?;
  def_macro_noop("\\say[]")?;
  def_macro_noop("\\saythe[]")?;

  // i18n
  DefMacro!("\\copyrightname", "??");
  DefMacro!("\\journalname", "??");
  DefMacro!("\\lofname", "List of Figures");
  DefMacro!("\\lotname", "List of Tables");
  DefMacro!("\\notesname", "Notes");
  DefMacro!("\\numbername", "number");
  DefMacro!("\\ppname", "pp");
  DefMacro!("\\tocname", "Contents");
  DefMacro!("\\volumename", "volume");

  // Document info — Perl L309-316
  def_macro_identity("\\volumenumber{}")?;
  def_macro_identity("\\volumeyear{}")?;
  def_macro_identity("\\issuenumber{}")?;
  DefMacro!("\\bibinfo{}{}", "#2");
  DefMacro!("\\eprint{}", "eprint #1");
  def_macro_identity("\\eid{}")?;
  DefMacro!("\\startpage{}", "\\pageref{FirstPage}{#1}");
  // Beneficial divergence from Perl revtex4_support.sty.ltxml:318, which has a
  // typo — `DefMacro('\endpage', '\pageref{LastPage}{#1}')` declares NO `{}`
  // parameter text yet the body references `#1`, so the unbound `#1` (catcode
  // ARG) leaks to the Stomach: "The token #1 should never reach Stomach!" (Perl
  // errors identically — verified on revtex4 + `\endpage{ }` + `\maketitle`).
  // Real revtex4's `\endpage` takes the page number, exactly like the adjacent
  // `\startpage{}` above; restore the missing `{}`. See KNOWN_PERL_ERRORS #34.
  DefMacro!("\\endpage{}", "\\pageref{LastPage}{#1}");

  // Extra stubs — Perl L319-323
  def_macro_noop("\\flushing")?;
  DefMacro!("\\triggerpar", "\\par");
  def_macro_noop("\\fullinterlineskip")?;
  // Perl L322: \footbox as box register (used by revtex footnote handling)
  RawTeX!("\\newbox\\footbox");
  DefRegister!("\\intertabularlinepenalty", Number(100));

  def_macro_noop("\\FL")?;
  def_macro_noop("\\FR")?;
  def_macro_noop("\\draft")?;
  def_macro_noop("\\tighten")?;

  // Journal abbreviations — Perl L336-365
  DefMacro!("\\ao", "Appl.~Opt.~");
  DefMacro!("\\ap", "Appl.~Phys.~");
  DefMacro!("\\apl", "Appl.~Phys.~Lett.~");
  DefMacro!("\\apj", "Astrophys.~J.~");
  DefMacro!("\\bell", "Bell Syst.~Tech.~J.~");
  DefMacro!("\\jqe", "IEEE J.~Quantum Electron.~");
  DefMacro!("\\assp", "IEEE Trans.~Acoust.~Speech Signal Process.~");
  DefMacro!("\\aprop", "IEEE Trans.~Antennas Propag.~");
  DefMacro!("\\mtt", "IEEE Trans.~Microwave Theory Tech.~");
  DefMacro!("\\iovs", "Invest.~Opthalmol.~Vis.~Sci.~");
  DefMacro!("\\jcp", "J.~Chem.~Phys.~");
  DefMacro!("\\jmo", "J.~Mod.~Opt.~");
  DefMacro!("\\josa", "J.~Opt.~Soc.~Am.~");
  DefMacro!("\\josaa", "J.~Opt.~Soc.~Am.~A ");
  DefMacro!("\\josab", "J.~Opt.~Soc.~Am.~B ");
  DefMacro!("\\jpp", "J.~Phys.~(Paris) ");
  DefMacro!("\\nat", "Nature (London) ");
  DefMacro!("\\oc", "Opt.~Commun.~");
  DefMacro!("\\ol", "Opt.~Lett.~");
  DefMacro!("\\pl", "Phys.~Lett.~");
  DefMacro!("\\pra", "Phys.~Rev.~A ");
  DefMacro!("\\prb", "Phys.~Rev.~B ");
  DefMacro!("\\prc", "Phys.~Rev.~C ");
  DefMacro!("\\prd", "Phys.~Rev.~D ");
  DefMacro!("\\pre", "Phys.~Rev.~E ");
  DefMacro!("\\prl", "Phys.~Rev.~Lett.~");
  DefMacro!("\\rmp", "Rev.~Mod.~Phys.~");
  DefMacro!("\\pspie", "Proc.~Soc.~Photo-Opt.~Instrum.~Eng.~");
  DefMacro!("\\sjqe", "Sov.~J.~Quantum Elecron.~");
  DefMacro!("\\vr", "Vision Res.~");

  // Internal macros — Perl L370-431
  def_macro_noop("\\@revmess{}{}")?;
  DefMacro!("\\@ptsize", "0");
  DefMacro!("\\@journal", "pra");

  // Document style options — Perl L372-393
  DefMacro!("\\ds@preprint", "\\global\\preprintstytrue \\def\\@ptsize{2}");
  def_macro_noop("\\ds@twoside")?;
  def_macro_noop("\\ds@draft")?;
  DefMacro!("\\ds@amsfonts", "\\@amsfontstrue");
  DefMacro!("\\ds@amssymb", "\\@amssymbolstrue");
  DefMacro!("\\ds@titlepage", "\\@titlepagefalse");
  def_macro_noop("\\ds@twocolumn")?;
  DefMacro!("\\ds@tighten", "\\@tightenlinestrue");
  DefMacro!("\\ds@floats", "\\@floatstrue");
  DefMacro!("\\ds@eqsecnum", "\\global\\secnumberstrue");
  DefMacro!("\\ds@pra", "\\def\\@journal{pra}");
  DefMacro!("\\ds@prb", "\\def\\@journal{prb}");
  DefMacro!("\\ds@prc", "\\def\\@journal{prc}");
  DefMacro!("\\ds@prd", "\\def\\@journal{prd}");
  DefMacro!("\\ds@pre", "\\def\\@journal{pre}");
  DefMacro!("\\ds@prl", "\\def\\@journal{prl}");
  DefMacro!("\\ds@josaa", "\\def\\@journal{josaa}");
  DefMacro!("\\ds@josab", "\\def\\@journal{josab}");
  DefMacro!("\\ds@aplop", "\\def\\@journal{aplop}");
  Let!("\\ds@manuscript", "\\ds@preprint");

  // newif stubs — Perl L396-414
  TeX!(r"
  \newif\ifpreprintsty \global\preprintstyfalse
  \newif\if@amsfonts  \@amsfontsfalse
  \newif\if@amssymbols  \@amssymbolsfalse
  \newif\if@titlepage  \@titlepagefalse
  \newif\if@tightenlines \@tightenlinesfalse
  \newif\if@floats \@floatsfalse
  \newif\ifsecnumbers \global\secnumbersfalse
  \@namedef{ds@11pt}{\def\@ptsize{1}}
  \@namedef{ds@12pt}{\def\@ptsize{2}}
  \@namedef{ds@aps}{\def\@society{aps}}
  \@namedef{ds@osa}{\def\@society{osa}}
  ");

  // Environment manipulation — Perl L425-430
  DefMacro!("\\replace@command{}{}", "\\global\\let#1#2 #1");
  // ltxutil.sty:97-102 (revtex4.cls, revtex4-1.cls, revtex4-2.cls:102-107) `\let@environment{<new>}{<old>}`, which a
  // document or a class built on REVTeX calls before it defines it itself (1101.2683, 1204.0239 `\let@environment
  // {abstract}{itp@abstract}`; 1510.05418, 1502.05917, 1510.09145; run-347 inventory 17 + 2 papers). Perl lacks it
  // (KNOWN_PERL_ERRORS #722).
  RawTeX!(r"\def\let@environment#1#2{%
 \expandafter\let
 \csname#1\expandafter\endcsname\csname#2\endcsname
 \expandafter\let
 \csname end#1\expandafter\endcsname\csname end#2\endcsname
}");
  // revtex4-2.cls:7459-7460 (revtex4.cls, revtex4-1.cls alike): the rule boxes `{widetext}` sets above and below a
  // wide passage, which a document's own variant fills (2101.04636 `{nobotrulewidetext}`, 2202.08908, 2506.16974).
  RawTeX!(r"\newbox\widetext@top\newbox\widetext@bot");
  // aps4-2.rtx:212-214 (aps4-1.rtx:182-184, revtex4's aps.rtx:158-160) and revtex4-2.cls:3079: the APS title's spacing
  // and font, which a document's second title block reuses (2006.03287 `\frontmatter@title@format Supplementary
  // Information: \ourtitle`; 2210.03153, 2101.08363, 2605.30425).
  RawTeX!(r"\def\frontmatter@title@above{\addvspace{6\p@}}%
\def\frontmatter@title@format{\large\bfseries\centering\parskip\z@skip}%
\def\frontmatter@title@below{}%
\def\frontmatter@finalspace{\addvspace{18\p@}}");
  // revtex4-2.cls:7520 (revtex4.cls:5965, revtex4-1.cls:7499): the obsolete `\altaddress` becomes `\altaffiliation`
  // (1509.02329, 1503.07039, 2109.03348).
  RawTeX!(r"\def\altaddress{\replace@command\altaddress\altaffiliation}");
  // revtex4-2.cls:6676 (revtex4.cls:5269, revtex4-1.cls:6655): the equation number's prefix, empty unless `eqsecnum`.
  RawTeX!(r"\def\theequation@prefix{}");
  // revtex4-2.cls:6283-6308 (revtex4-1.cls, revtex4.cls:4087-4105): the address-grouping switches and the option
  // macros that set them, which a style built on REVTeX calls (2402.07499 ljm.sty:304-305 `\@ifxundefined
  // \groupauthors@sw{\clo@groupedaddress}{}`; 2012.06334, 2206.05563, 2410.06749). The frontmatter API places the
  // affiliations; the switches set nothing here.
  RawTeX!(r"\def\clo@groupedaddress{%
 \@booleantrue\groupauthors@sw
 \@booleantrue\@affils@sw
 \@booleanfalse\runinaddress@sw
}%
\def\clo@unsortedaddress{%
 \@booleantrue\groupauthors@sw
 \@booleanfalse\@affils@sw
 \@booleanfalse\runinaddress@sw
}%
\def\clo@runinaddress{%
 \@booleantrue\groupauthors@sw
 \@booleantrue\@affils@sw
 \@booleantrue\runinaddress@sw
}%
\def\clo@superscriptaddress{%
 \@booleanfalse\groupauthors@sw
 \@booleantrue\@affils@sw
 \@booleanfalse\runinaddress@sw
}");
  def_macro_noop("\\replace@environment{}{}")?;
  def_macro_noop("\\glet@environment{}{}")?;
});

/// The REVTeX 3 option handlers this binding defines (Perl L372-393), for a `\documentstyle[aps,…]{revtex}` to consume.
/// A revtex4/4-1/4-2 class has processed its options before it loads this binding, so it lets them to `\relax`, as
/// latex.ltx's `\@process@pti@ns` (:18593-18594) leaves the handlers it declared: otherwise a later package's
/// undeclared `[draft]` ran revtex's no-op instead of its `\default@ds` (content.rs `process_options_by`).
pub(crate) fn retire_option_handlers() {
  for option in [
    "preprint",
    "twoside",
    "draft",
    "amsfonts",
    "amssymb",
    "titlepage",
    "twocolumn",
    "tighten",
    "floats",
    "eqsecnum",
    "pra",
    "prb",
    "prc",
    "prd",
    "pre",
    "prl",
    "josaa",
    "josab",
    "aplop",
    "manuscript",
    "11pt",
    "12pt",
    "aps",
    "osa",
  ] {
    let_i(&T_CS!(&s!("\\ds@{option}")), &T_RELAX!(), None);
  }
}

/// REVTeX 4.1/4.2's two copies of float.sty's `\newfloat` (revtex4-1.cls:3980-4021, revtex4-2.cls:4001-4042):
/// `\newfloat@float`, float.sty's own, for the `\document@inithook` test that float.sty is loaded unpatched, and
/// `\newfloat@ltx`, the repair the hook lets `\newfloat` to (revtex4-2.cls:4043-4056). A preamble that needs the
/// repair before `\begin{document}`, where algorithm.sty's `\newfloat{algorithm}` runs, lets it itself
/// (`\usepackage{float}\let\newfloat\newfloat@ltx\usepackage{algorithm}`: all 25 papers of the run-349 `\newfloat`
/// sample; 2210.07980, 2210.08566, 2210.07097, 2402.00653, 2002.07183), and found it undefined here, which left
/// `\newfloat` and `{algorithm}` undefined (KNOWN_PERL_ERRORS #683). Both copies set up a float through float.sty's
/// machinery (`\floatplacement`, `\restylefloat`), which this port has as float_sty.rs's `\newfloat`; without
/// float.sty loaded they fail, as in TeX. REVTeX 4.0 has neither (revtex4.cls); AASTeX 6+ has both (aastex_cls.rs).
pub(crate) fn define_float_repair() -> Result<()> {
  DefMacro!(
    "\\newfloat@float{}{}{}",
    "\\@ifnextchar[{\\lx@float@newfloat{#1}{#2}{#3}}{\\lx@float@newfloat{#1}{#2}{#3}[]}"
  );
  DefMacro!(
    "\\newfloat@ltx{}{}{}",
    "\\@ifnextchar[{\\lx@float@newfloat{#1}{#2}{#3}}{\\lx@float@newfloat{#1}{#2}{#3}[]}"
  );
  Ok(())
}

/// The bibliography style REVTeX writes when the document names none:
/// `\bibliographystyle{\@bibstyle}` at the end of the document
/// (revtex4-1.cls:7113-7120, revtex4-2.cls:7134-7141), `\@bibstyle` set by the
/// society's `.rtx` — `apsrev\substyle@post` (aps4-1.rtx:630, aps4-2.rtx:667;
/// REVTeX 4.0's apsrev.bst), `aipauth`/`aipnum\substyle@post` by citation style
/// (aip4-1.rtx:617-640). Recorded for the `.bib` reader only (pre_bibtex.rs
/// `new_from_gullet` reads the style's `MACRO`s: apsrev's `prl`, `jan`, …), not
/// as `BIBSTYLE`, which also decides the bibliography's numbering. `post` is the
/// version's `\substyle@post` ("4-1", "4-2", or "" for REVTeX 4.0). Witness
/// 2603.22119 (revtex4-1, no `\bibliographystyle`).
pub(crate) fn record_implicit_bibstyle(post: &str) {
  let authoryear = lookup_string("revtex_cite_style") == "authoryear";
  let style = match lookup_string("revtex_society").as_str() {
    "" | "aps" => s!("apsrev{post}"),
    "aip" if authoryear => s!("aipauth{post}"),
    "aip" => s!("aipnum{post}"),
    _ => return,
  };
  assign_value("lx@implicit@bibstyle", pin(style), Some(Scope::Global));
}

/// The REVTeX 4.1/4.2 internals documents call (revtex4-1.cls, revtex4-2.cls; REVTeX 4.0 has none of them), for the two
/// class bindings:
/// - :1965-1967 / :1986-1991 `\doi{<doi>}`, the DOI link the APS `.bbl`s and documents write (2010.08759, 1805.10530,
///   2109.10305, 2302.01688; run-347 inventory 101 revtex4-2 papers), and `\doibase`, its resolver, which documents and
///   `.bbl`s put before a DOI in `\href` (2301.10789 `\href {\doibase 10.1007/…}`; 26 papers). `\@sanitize@url` makes
///   URL characters other before the argument is read: `HyperVerbatim` does so, `%` included (2006.12945
///   `\doi{…%2F…}`, a PushbackLimit Fatal as `Semiverbatim`; 2403.08476, 2112.03925 `_`). The resolver is
///   `https://doi.org/` (revtex4-2.cls:1988; 4.1's `http://dx.doi.org/` redirects there). `\@sanitize@url` (:1989)
///   is the class's; `\@@startlink{<url>}`…`\@@endlink` (:1990-2009) open and close the link, which the class makes
///   live when hyperref is loaded (:1992-2009 `\hypertext@enable@ltx`, from `\document@inithook`), as this binding
///   always loads it: an `<ltx:ref>` around the text between them (1211.1037's own `\Doi`, 87 links, whose URLs had
///   stood as text only by the undefined-command recovery).
/// - ltxgrid.sty:506-513 `\onecolumn@grid@setup`/`\twocolumn@grid@setup`, the column grid's insert setup (2609.36113,
///   2606.16884, 2609.15324, 2609.38318) — layout only.
/// - ltxutil.sty:418-421 `\prep@math@patch` over `\prepdef` (a no-op here), ltxutil.sty:612 `\ltx@footnote@pop`
///   (`\@empty`), revtex4-2.cls:5845 `\setup@hook` (`\@empty`), :6746 `\setfloatlink`.
/// - :6283-6285 the address switches' defaults.
pub(crate) fn define_revtex41_internals() -> Result<()> {
  DefMacro!("\\doi HyperVerbatim", "doi:\\href{https://doi.org/#1}{#1}");
  DefMacro!("\\doibase", "https://doi.org/");
  RawTeX!(
    r"\providecommand \@sanitize@url[0]{\chardef\cat@space\the\catcode`\ \@sanitize\catcode`\ \cat@space}%
\def\prep@math@patch{%
 \prepdef\equation{\prep@math}%
 \prepdef\eqnarray{\prep@math}%
}%
\let\ltx@footnote@pop\@empty
\let\setup@hook\@empty
\def\setfloatlink{\def\float@link}%
\@booleanfalse\runinaddress@sw
\@booleantrue\@affils@sw
\@booleanfalse\groupauthors@sw"
  );
  def_macro_noop("\\onecolumn@grid@setup")?;
  def_macro_noop("\\twocolumn@grid@setup")?;
  DefConstructor!("\\@@startlink HyperVerbatim", "<ltx:ref href='#href'>",
  enter_horizontal => true,
  properties => sub[args] {
    let url = args.first().and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
    Ok(stored_map!("href" => clean_url(&url)))
  });
  DefConstructor!("\\@@endlink", sub[document, _args] {
    document.maybe_close_element("ltx:ref")?;
  });
  Ok(())
}

/// REVTeX 4's section and equation numbers, set by the class after its options (revtex4-2.cls:6528-6576
/// `\secnums@rtx`/`\secnums@arabic` run from `\setup@hook` at :7678, revtex4-1.cls:6507-6555 the same; revtex4.cls:5163-5172
/// at load, :6055-6064 its `secnumarabic`), which Perl's binding reduces to `\thesection` (revtex4_support.sty.ltxml:111;
/// KNOWN_PERL_ERRORS #1027):
/// - APS (and AIP, SOR, the class's default): "I", "A", "1", "a", a `\ref` chaining them with thin spaces through
///   `\p@<level>` ("I A 1"); the `rmp` journal with periods (apsrmp4-2.rtx:124-148, apsrmp4-1.rtx:115-139, revtex4's
///   rmp.rtx:113-122: "I.A.1"); REVTeX 4.2's `aapm` society in the numbers themselves (aapm4-2.rtx:290-314: "I.A.1");
///   `secnumarabic` "I.1", "I.1.1" in 4.1/4.2 (its `\thesection` stays `\Roman`), "1.1" in 4.0 (`\arabic`).
/// - `\setcounter{secnumdepth}{4}` (revtex4-2.cls:6677, revtex4-1.cls:6656, revtex4.cls:5273): paragraphs numbered.
///   The society's journal hook, run after it, unnumbers them: PRL's `\c@secnumdepth=-\maxdimen` (aps4-2.rtx:587
///   `\rtx@apsprl`, run from `\@process@journal` at revtex4-2.cls:7671; aps4-1.rtx:564; revtex4's aps.rtx:517 at
///   `\AtEndOfClass`): its headings carry no number, a `\ref` to one prints nothing, and the appendix steps no section,
///   so its equations go on counting ("(16)", the empty `\Alph{0}` their prefix; 1508.03506's PDF). The journal is the
///   one the class selects ([`selected_journal`]). AIP Conference Proceedings' `\setcounter{secnumdepth}{0}`
///   (aip4-2.rtx:1181) never runs: its `\ifx\@journal\jnl@cp` (:1016) is read when the society file loads (:7670),
///   before the journal option is (:7671), so `cp` papers are numbered (pdflatex). No other society file sets either
///   (aapm4-2.rtx, sor4-2.rtx, aip4-1.rtx: only their `{theindex}`).
/// - The heading prints `\csname the#1\endcsname.` (aps4-2.rtx:288 `\@seccntformat`, aps4-1.rtx:258, aip4-2.rtx:375,
///   sor4-2.rtx:338, revtex4's aps.rtx:231; apsrmp and aapm with `\hskip0.5em`): the heading tags' `\fnum@<level>`
///   (base_utilities.rs `\lx@fnum@@`), the refnum staying the `\ref` form.
/// - `\theequation` is `\theequation@prefix\arabic{equation}` (revtex4-2.cls:6672-6676), the prefix `\arabic{section}.`
///   with equations reset per section under `eqsecnum` (:6057-6064).
/// - `\appendix` (revtex4-2.cls:7372-7387, revtex4-1.cls:7351-7366, revtex4.cls:5814-5829) numbers the appendices'
///   subsections `\arabic`, their subsubsections `\alph`, and their equations within each appendix ("A1"); `\appendix*`
///   leaves the appendix unnumbered ("A." for its equations). The heading "Appendix A" is the appendix type's tag
///   (aps4-2.rtx:479 `\@appendixcntformat`).
///
/// Guards `perfect_kernel_batch78d::revtex_sections_number_as_the_class`; `perfect_kernel_batch64::aastex_sections_arabic`
/// (AASTeX's own numbers stay).
pub(crate) fn define_section_numbers(post: &str) -> Result<()> {
  // A class built on REVTeX that sets its own (aastex_cls.rs, which emulateapj_cls.rs loads).
  if lookup_bool("lx@revtex@sections@by@class") {
    return Ok(());
  }
  let revtex4_0 = post.is_empty();
  let opt = |name: &str| lookup_bool(&s!("revtex@opt@{name}"));
  let (society, journal) = selected_journal(post)?;
  let (rmp, aapm, arabic) = (
    society == "aps" && journal == "rmp",
    society == "aapm",
    opt("secnumarabic"),
  );
  let mut tex =
    String::from(r"\def\thepart{\Roman{part}}\def\thesection{\Roman{section}}\def\p@section{}");
  if aapm && !rmp {
    tex.push_str(
      r"\def\thesubsection{\thesection.\Alph{subsection}}\def\p@subsection{}%
\def\thesubsubsection{\thesubsection.\arabic{subsubsection}}\def\p@subsubsection{}%
\def\theparagraph{\thesubsubsection.\alph{paragraph}}\def\p@paragraph{}%
\def\thesubparagraph{\theparagraph.\arabic{subparagraph}}\def\p@subparagraph{}",
    );
  } else {
    let sep = if rmp { "." } else { r"\," };
    tex.push_str(&s!(
      r"\def\thesubsection{{\Alph{{subsection}}}}\def\p@subsection{{\thesection{sep}}}%
\def\thesubsubsection{{\arabic{{subsubsection}}}}\def\p@subsubsection{{\thesection{sep}\thesubsection{sep}}}%
\def\theparagraph{{\alph{{paragraph}}}}\def\p@paragraph{{\thesection{sep}\thesubsection{sep}\thesubsubsection{sep}}}%
\def\thesubparagraph{{\arabic{{subparagraph}}}}%
\def\p@subparagraph{{\thesection{sep}\thesubsection{sep}\thesubsubsection{sep}\theparagraph{sep}}}"
    ));
  }
  if arabic && revtex4_0 {
    tex.push_str(
      r"\def\p@section{}\def\thesection{\arabic{section}}\def\p@subsection{}%
\def\thesubsection{\thesection.\arabic{subsection}}\def\p@subsubsection{}%
\def\thesubsubsection{\thesubsection.\arabic{subsubsection}}",
    );
  } else if arabic {
    tex.push_str(
      r"\def\thesubsection{\thesection.\arabic{subsection}}\def\p@subsection{}%
\def\thesubsubsection{\thesubsection.\arabic{subsubsection}}\def\p@subsubsection{}%
\def\theparagraph{\thesubsubsection.\arabic{paragraph}}\def\p@paragraph{}%
\def\thesubparagraph{\theparagraph.\arabic{subparagraph}}\def\p@subparagraph{}",
    );
  }
  tex.push_str(
    r"\setcounter{secnumdepth}{4}%
\def\fnum@section{\thesection.}\def\fnum@subsection{\thesubsection.}\def\fnum@subsubsection{\thesubsubsection.}%
\def\fnum@paragraph{\theparagraph.}\def\fnum@subparagraph{\thesubparagraph.}%
\def\theequation{\theequation@prefix\arabic{equation}}%
\def\appendix{\par\@appendix\setcounter{subsection}\z@\setcounter{subsubsection}\z@
 \def\thesubsection{\arabic{subsection}}\def\thesubsubsection{\alph{subsubsection}}%
 \@addtoreset{equation}{section}\def\theequation@prefix{\thesection}%
 \@ifstar{\def\thesection{\unskip}\def\theequation@prefix{A.}}{}}",
  );
  if opt("eqsecnum") {
    tex.push_str(r"\@addtoreset{equation}{section}\def\theequation@prefix{\arabic{section}.}");
  }
  if society == "aps" && journal == "prl" {
    tex.push_str(r"\c@secnumdepth=-\maxdimen");
  }
  RawTeX!(&tex);
  Ok(())
}

/// The society and journal a REVTeX class selects from its options, `post` its `\substyle@post` ("4-2", "4-1", or ""
/// for REVTeX 4.0). REVTeX 4.1/4.2 read the options in the order given: the first naming a society file
/// (`<society><post>.rtx`) is the society (revtex4-2.cls:6220-6230 `\@parse@class@options@society`, `aps` without one,
/// :7670), the first naming one of its journals (`\change@journal`, :6094-6104; aps4-2.rtx, aip4-2.rtx `\DeclareOption`s)
/// the journal. REVTeX 4.0 processes them in declaration order, each `\def\@journal` replacing the last
/// (revtex4.cls:4010-4023, :4160 `\ProcessOptions\relax`), `aps` and `osa` emptying it.
pub(crate) fn selected_journal(post: &str) -> Result<(String, String)> {
  let given = crate::shipped_class::class_options()?;
  let named = |option: &str| given.iter().any(|g| g == option);
  if post.is_empty() {
    // The declaration order, `aps` and `osa` emptying the journal, the `osa` ones changing the society.
    let declared: Vec<&str> = "aps pra prb prc prd pre prl prstab rmp osa osameet opex tops josa"
      .split_whitespace()
      .collect();
    let last = declared
      .iter()
      .enumerate()
      .rfind(|(_, option)| named(option));
    return Ok(match last {
      Some((at, option)) => (
        (if at < 9 { "aps" } else { "osa" }).to_string(),
        (if matches!(*option, "aps" | "osa") {
          ""
        } else {
          option
        })
        .to_string(),
      ),
      None => ("aps".to_string(), String::new()),
    });
  }
  let societies = if post == "4-1" {
    "aps aip apsrmp"
  } else {
    "aps aip aapm sor apsrmp"
  };
  let society = given
    .iter()
    .find(|g| {
      societies
        .split_whitespace()
        .any(|society| society == g.as_str())
    })
    .map_or("aps", String::as_str)
    .to_string();
  let journals = match (post, society.as_str()) {
    ("4-1", "aps") => "pra prb prc prd pre prl prstab prstper rmp",
    ("4-1", "aip") => "jcp pop rsi jap apl cha pof bmf rse jmp",
    (_, "aps") => {
      "pra prb prc prd pre prl prab prper rmp prx prapplied prmaterials prfluids physrev"
    },
    (_, "aip") => "jcp pop rsi jap apl apm cha pof bmf rse jmp adv sd jor cp",
    _ => "",
  };
  let journal = given
    .iter()
    .find(|g| {
      journals
        .split_whitespace()
        .any(|journal| journal == g.as_str())
    })
    .cloned()
    .unwrap_or_default();
  Ok((society, journal))
}

/// REVTeX 4.0 provides `\bm` at `\begin{document}`, only while it is still undefined (revtex4.cls:6030-6032
/// `\appdef\class@inithook{…\@ifxundefined\bm{\let\bm\REV@bm}{}…}`, run from `\class@documenthook` at
/// `\AtBeginDocument`, :213-215; `\@ifxundefined` counts a `\relax` as defined), so a preamble's own
/// `\newcommand{\bm}{\bibitem}` (nucl-th0307033) stands; defined at load, the binding's took it away (Perl
/// revtex4_support.sty.ltxml:167 the same, KNOWN_PERL_ERRORS #965). REVTeX 4.1/4.2 define it at load. Guards:
/// `perfect_kernel_batch73a::{revtex_bm_is_provided_at_begin_document, revtex41_bm_is_defined_at_load}`.
pub fn defer_bm_to_begin_document() -> Result<()> {
  RawTeX!(
    r"\let\lx@revtex@bm\bm\let\bm\@undefined\AtBeginDocument{\@ifxundefined\bm{\let\bm\lx@revtex@bm}{}}"
  );
  Ok(())
}

/// Whether the REVTeX 4 class prints a `\preprint` called now: its title page prints the stored numbers under
/// `\preprint@sw` (aps4-1.rtx:216-221 `\ps@titlepage`, aip, aapm, sor alike; revtex4's aps.rtx:195, :208-209).
/// - REVTeX 4.0 (`revtex@preprint@v4.0`, revtex4_cls.rs): `preprintnumbers`/`nopreprintnumbers` set the switch
///   (revtex4.cls:3891-3892) and the `preprint` style forces it on after them (class init hook, :3811-3812); the store
///   is emptied once at load (:4618-4619), so a preamble call is printed too.
/// - REVTeX 4.1/4.2: an unset switch follows the `preprint` style (revtex4-2.cls:5927-5930, revtex4-1.cls alike), and
///   `\frontmatter@init` empties the store at the document's start (revtex4-1.cls:2094, :2517-2519), so a preamble
///   call is never printed.
fn preprint_printed() -> bool {
  let style = lookup_bool("revtex@preprintsty");
  let numbers = match lookup_value("revtex@preprintnumbers") {
    Some(Stored::Bool(numbers)) => Some(numbers),
    _ => None,
  };
  if lookup_bool("revtex@preprint@v4.0") {
    style || numbers == Some(true)
  } else {
    !lookup_bool("inPreamble") && numbers.unwrap_or(style)
  }
}

/// The items of `tokens` separated by `\\` at its top level, outside every group and `\begin`…`\end` environment.
fn split_top_level_lines(tokens: &[Token]) -> Vec<&[Token]> {
  let (mut items, mut start, mut depth, mut envs) = (Vec::new(), 0, 0usize, 0usize);
  for (i, t) in tokens.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if *t == T_CS!("\\begin") => envs += 1,
      _ if *t == T_CS!("\\end") => envs = envs.saturating_sub(1),
      _ if depth == 0 && envs == 0 && t.defined_as(&T_CS!("\\\\")) => {
        items.push(&tokens[start..i]);
        start = i + 1;
      },
      _ => {},
    }
  }
  items.push(&tokens[start..]);
  items
}
