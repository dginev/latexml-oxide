use latexml_core::definition::origin::{DefinitionOrigin, OriginGuard};

use crate::{
  engine::latex_constructs::{after_float, before_float_ex},
  package::{deluxetable_sty, subcaption_sty::subcaption_width_props},
  prelude::*,
};

/// Whether `digested` holds a table float of its own — the `{table}` a deluxetable opens (`{longrotatetable}`).
fn holds_table_float(digested: &Digested) -> bool {
  match digested.data() {
    DigestedData::List(list) => list.borrow().boxes.iter().any(holds_table_float),
    DigestedData::Whatsit(whatsit) => {
      let whatsit = whatsit.borrow();
      let cs = whatsit.get_definition().get_cs().to_string();
      cs == "\\begin{table}"
        || cs == "\\begin{table*}"
        || whatsit
          .get_body()
          .ok()
          .flatten()
          .is_some_and(|body| holds_table_float(&body))
        || whatsit.get_args().iter().flatten().any(holds_table_float)
    },
    _ => false,
  }
}

/// The older AASTeX's affiliation marks (Perl aas_support.sty.ltxml): `\altaffilmark{1,2}` in an `\author` marks the
/// author with the labels it lists, and `\altaffiltext{n}{text}` is the affiliation those marks request, of every
/// author carrying `n` — the AAS idiom the classes that copied it share (OmniBus, aguplus, rmaa: `rmaa_cls`).
pub fn altaffil_markup() -> Result<()> {
  DefMacro!(
    "\\altaffilmark Semiverbatim",
    "\\lx@request@frontmatter@annotation[altaffil]{#1}"
  );
  DefMacro!(
    "\\altaffiltext Semiverbatim {}",
    "\\lx@add@contact[role=altaffiliation,label={altaffil:#1}]{#2}"
  );
  Ok(())
}

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
  // `\author` ends a corresponding author's `\email` run (aastex701.cls:12678 `\global\correspauthorfalse`; aastex_cls.rs).
  DefMacro!("\\author[]{}",
    "\\global\\lx@aas@correspauthorfalse\\lx@add@authors@append{#2}\\lx@aas@checkorcid{#1}\\lx@aas@pending@emails\
     \\gdef\\lx@aas@pending@emails{}\\gdef\\lx@aas@authored{}");
  def_macro_noop("\\lx@aas@pending@emails")?;
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
  // The flag and name are AASTeX 7's (aastex701.cls:13318-13328), for its `\email` (aastex_cls.rs): the address after a
  // `\correspondingauthor{X}` is X's, wherever it stands.
  RawTeX!(r"\newif\iflx@aas@correspauthor\let\lx@aas@correspondent\empty");
  DefMacro!("\\correspondingauthor{}",
    "\\global\\lx@aas@correspauthortrue\\gdef\\lx@aas@correspondent{#1}\\lx@add@contact[role=correspondent,label={fuzzy:#1}]{#1}");


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
  // (in the current font, aastex631.cls:6924-6930 `\href{mailto: #1}{#1}`; 2301.02349, 2101.08512)
  DefMacro!("\\email[]{}",        "\\lx@add@email{\\lx@address@textfont{#2}}");

  // \collaboration{n}{name} applies to previous authors w/o collab; n is how many to SHOW
  // \nocollaboration{n} ditto, but says they are not in a collaboration group.
  // That is AASTeX 6.3+'s (aastex701.cls:13355 `\def\nocollaboration#1`, :13377 `\def\collaboration#1#2`); AASTeX 6.1's
  // are REVTeX's, `\collaboration{name}` and an argument-less `\nocollaboration` (aastex61.cls:6298, revtex4-1's
  // `\@author@def`; aastex701.cls:1942 before its 6.3 redefinition). One binding serves both, by the call's shape: a
  // second group after `\collaboration`'s, a group after `\nocollaboration`, is the 6.3 form — Perl's two-argument
  // `\collaboration` (aas_support.sty.ltxml:126) took the `\author` after a 6.1 `\collaboration{(Spitzer Team)}` as
  // the name, and "P. Mróz" became a paragraph (1802.09023; KNOWN_PERL_ERRORS). Guard
  // `perfect_kernel_batch81::an_aastex61_collaboration_takes_one_argument`.
  DefMacro!("\\collaboration{}",
    "\\@ifnextchar\\bgroup{\\lx@aas@collaboration@count{#1}}{\\lx@aas@collaboration@name{#1}}");
  DefMacro!("\\lx@aas@collaboration@count{}{}", "\\lx@add@contact[role=collaboration,annotate=new]{#2}");
  DefMacro!("\\lx@aas@collaboration@name{}", "\\lx@add@contact[role=collaboration,annotate=new]{#1}");
  DefMacro!("\\nocollaboration",
    "\\@ifnextchar\\bgroup{\\lx@aas@nocollaboration}{\\lx@add@contact[role=collaboration,annotate=new]{}}");
  DefMacro!("\\lx@aas@nocollaboration{}", "\\lx@add@contact[role=collaboration,annotate=new]{}");

  // Older aas versions: put mark within \author, text will be attached
  altaffil_markup()?;
  // The obsolete `\authoremail{…}` (Perl aas_support.sty.ltxml:134 `\email`; aaspp4.sty:129 and aas2pp4.sty:136
  // gobble it, the address unprinted): after authors, `\email`'s, the last author's (astro-ph9612041 gives one after
  // each `\author`); given before any `\author` (astro-ph9803018's `\authoremail{ecford@astro.uva.nl}\author{Eric C.
  // Ford…}`), it waits for the first `\author` and goes to the author its address spells, else to that `\author`'s
  // first (`\lx@add@contact@placed[owner=first]`), never a creator of its own.
  DefMacro!("\\authoremail{}", sub[(address)] {
    // (after an `\author`: `\lx@aas@authored`, which the creators' queue does not tell once they are digested)
    if lookup_meaning(&T_CS!("\\lx@aas@authored")).is_some() {
      let mut call = vec![T_CS!("\\email"), T_BEGIN!()];
      call.extend(address.unlist());
      call.push(T_END!());
      Ok(Tokens::new(call))
    } else {
      let mut call = vec![T_CS!("\\lx@add@contact@placed"), T_OTHER!("[")];
      call.extend(mouth::tokenize_internal("role=email,owner=first").unlist());
      call.extend([T_OTHER!("]"), T_BEGIN!(), T_END!(), T_BEGIN!(), T_CS!("\\lx@address@textfont"), T_BEGIN!()]);
      call.extend(address.unlist());
      call.extend([T_END!(), T_END!()]);
      let mut wait = vec![T_CS!("\\g@addto@macro"), T_CS!("\\lx@aas@pending@emails"), T_BEGIN!()];
      wait.extend(call);
      wait.push(T_END!());
      Ok(Tokens::new(wait))
    }
  });

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
  // family binding defines them for every version, and the AASTeX 6+ and emulateapj class bindings, whose classes have no
  // `{plate}`, undefine the environment (aastex_cls.rs `undefine_plate`).
  RawTeX!(r"\def\ext@plate{lof}");
  DefMacro!("\\platename", "Plate");
  def_macro_noop("\\platewidth{Dimension}")?;
  // aastex.cls 5.x:1691-1695: the plate takes the given number, and the counter steps back so the next plate keeps its
  // own (Perl's only redefined `\theplate`, aas_support.sty.ltxml:206).
  DefMacro!("\\platenum{}", "\\def\\theplate{#1}\\let\\@currentlabel\\theplate\\addtocounter{plate}{\\m@ne}");
  // AASTeX 5.x's other commands the later classes dropped (aastex.cls 5.2): `\subsubsubsection` a level-4 heading
  // (:1088-1091, `\@startsection{subsubsection}{4}…`, unnumbered at secnumdepth 3), `\supportfrom[…]{…}` its text
  // (:1988-1989); kept for every AASTeX version, the family's union (user 2026-10-10), where a 6+ paper's own
  // `\newcommand` of them is skipped. `\subsubsubsection`: 0804.1946 ("Synthetic Secular Theory"; `Error:undefined`
  // before), 0803.0586.
  DefMacro!("\\subsubsubsection", "\\paragraph");
  DefMacro!("\\supportfrom[]{}", "#2");

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
    // The begin code opens a TeX group of its own: a float: `\@float` (latex.ltx:17479 `\color@vbox`).
    tex_group => { TexGroup::Box },
    before_digest => { before_float_ex("plate", None, false)?; engine::latex_constructs::reset_float_box()?; },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );
  DefEnvironment!("{plate*}[]",
    "<ltx:float xml:id='#id' inlist='#inlist' ?#1(placement='#1') class='ltx_float_plate'>#tags#body</ltx:float>",
    // The begin code opens a TeX group of its own: a float: `\@float` (latex.ltx:17479 `\color@vbox`).
    tex_group => { TexGroup::Box },
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
      let mut caption = vec![T_CS!("\\lx@@caption"), T_BEGIN!()];
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
  // float form was its own guess, aas_support.sty.ltxml:210); not followed by a brace, LaTeXML's peek reads it as a
  // `\ref` (aas_support.sty.ltxml:211-221). The papers that define their own `\fig` shorthand are AASTeX 5.x or
  // emulateapj ones, where aastex_cls.rs leaves `\fig` undefined for theirs (astro-ph/0003209, astro-ph/0503342).
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

  // AASTeX 6.2-6.3.1 commands the class files define (aastex62.cls, aastex63.cls, aastex631.cls; aastex701.cls), which
  // the papers that ship those classes call and our binding shadows (run 340, ~1,700 papers by first error:
  // `\listofchanges` 409, `\AuthorCallLimit` 369, `\turnoffedit` 154, `\reportnum` 128, `\journalinfo` 222,
  // `\suppressAffiliations` 104, `\NewPageAfterKeywords` 93, `\setwatermarkfontsize` 65; witnesses 2604.07983,
  // 2508.14467, 2604.22347, 2103.02219, 2103.05148, 1904.02673). Perl's aas_support.sty.ltxml defines none of them
  // (KNOWN_PERL_ERRORS #633).
  // The edit-tracking switches and the list of changes (aastex631.cls:5789-5798, :6108): the `\added`/`\deleted`
  // marks above print as plain text here, so the switches change nothing, and the list, a print summary of those
  // marks, is not typeset.
  // The switches keep their flags (aastex631.cls:5790-5809, the copy 2107.07519 ships), which papers set directly
  // (`\turnoffonetrue`: 2609.30834) and the class's `\authorcomment` (below) reads; `\edit` here prints its text
  // whatever they say (the class drops a turned-off `\edit<n>`'s text, :5823-5833: a residual).
  RawTeX!(r"\newif\ifturnoffedit\def\turnoffedit{\global\turnoffedittrue}\def\turnoffedits{\global\turnoffedittrue}
\let\turnoffediting\turnoffedits\newif\ifturnoffone\newif\ifturnofftwo\newif\ifturnoffthree
\def\turnoffeditone{\turnoffonetrue}\def\turnoffedittwo{\turnofftwotrue}\def\turnoffeditthree{\turnoffthreetrue}");
  def_macro_noop("\\listofchanges")?;
  // aastex631.cls:651 `\reportnum{…}`: a report number set at the top right of the first page.
  DefMacro!("\\reportnum{}", "\\lx@add@pubnote[role=preprint]{#1}");
  // aastex631.cls:1494 `\journalinfo{…}`: the journal line of the first page's head ("Accepted to ApJ"); the same
  // note emulateapj's makes (emulateapj_sty.rs).
  DefMacro!("\\journalinfo{}", "\\lx@add@pubnote[role=journal]{#1}");
  // aastex6.cls:972/984 `\submitted{…}` feeds the same journal line (`\journalinfo{\@submitted}`).
  DefMacro!("\\submitted{}", "\\lx@add@pubnote[role=journal]{#1}");
  // aastex6.cls:5212 `\fullcollaborationName{…}` stores the collaboration's full name for the author block's note.
  DefMacro!("\\fullcollaborationName{}", "\\def\\thefullcollaborationname{#1}");
  // Layout switches (aastex631.cls:7323, :6816, :5856): affiliations hidden from the author block, a page break
  // after the keywords, the watermark's font size.
  // `\suppressAffiliations` sets the class's flag (aastex631.cls:7324-7325), which papers reset
  // (`\suppressAffiliationsfalse`: 39 papers, 2108.09109, 2210.08658, 2001.00952): the author block here keeps
  // every affiliation either way.
  RawTeX!(r"\newif\ifsuppressAffiliations\def\suppressAffiliations{\global\suppressAffiliationstrue}");
  def_macro_noop("\\NewPageAfterKeywords")?;
  def_macro_noop("\\setwatermarkfontsize{}")?;
  // aastex631.cls:6427-6430 the number of authors the author block shows before "et al." (`\AuthorCallLimit` its
  // alias): every author is kept here, so the count is only set.
  RawTeX!(r"\newcount\AuthorCollaborationLimit \AuthorCollaborationLimit=10000 \let\AuthorCallLimit\AuthorCollaborationLimit");
  // aastex631.cls:1930 the old name of `\tabletypesize`; :2096 the skip above a deluxetable.
  DefMacro!("\\tablefontsize", "\\tabletypesize");
  RawTeX!(r"\newlength{\abovedeluxetableskip}");
  // More of AASTeX 6.x (aastex631.cls in 2107.07519's copy; aastex63.cls alike, 1907.08642's), which papers call:
  // - the table size `\currtabletypesize` that `\tabletypesize` keeps (:1928-1932; 2202.09386, 2405.09667 print
  //   their notes in it), `\savetablenum` (:1926), `\tablereferences` (:5391), `\deluxetablecaption` (:4780,
  //   :4798), `\tabcaption[…]{…}`, a table caption outside a float (:1609-1611; 2408.17407, 2502.00504);
  // - `\resetapptablenumbers` (:7210-7216), the appendix numbering of tables, figures and equations (7 papers:
  //   2308.05494, 2409.14445); `\authorcomment<n>{…}`, "(Author note n: …)" unless turned off (:5843-5850; 5 papers:
  //   2004.09658, 2205.09859); `\onecolfootnote` (:7316-7317), a footnote; `\BeforeBegin{env}{code}` (:581), the hook the
  //   class's own `\begin` runs (papers patch nicematrix through it: 2111.03992, 2203.13406; this `\begin` runs no
  //   such hook); `\mdash` (:7141); `\apjsecfont` (:219), the section heading's font (2306.12805); the
  //   acknowledgments box `\ackbox` (:1205; 2605.04423 fills its own); `\ifnumlines` (:253, the `linenumbers`
  //   option's flag; 2608.12156).
  // These are AASTeX 6.2 and later's only (aastex62.cls … aastex701.cls; AASTeX 5.2's aastex.cls, 1001.0026's, has none
  // of them, nor `\authorcomment` aastex701.cls), and this binding serves AASTeX 5 and emulateapj too: as a fallback's
  // guess, so a paper's own `\newcommand\mdash{--}`, `\tabcaption`, `\BeforeBegin` … wins (the review of 68a; repro
  // class-bindings/aastex5_newer_commands_yield).
  {
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    RawTeX!(r"\def\resetapptablenumbers{\global\c@table=0 \global\c@figure=0 \global\c@equation=0
\def\thetable{\thesection\the\c@table}\def\fnum@table{{\bf\tablename~\thetable}}\def\thefigure{\thesection\the\c@figure}}
\expandafter\def\csname editcolor1\endcsname{black}\expandafter\def\csname editcolor2\endcsname{black}%
\expandafter\def\csname editcolor3\endcsname{black}\newcount\colorcount
\let\onecolfootnote\footnote
\long\def\BeforeBegin#1#2{\expandafter\gdef\csname BeforeBeg#1\endcsname{#2}}
\def\mdash{---}\def\apjsecfont{\normalsize}\newbox\ackbox\newif\ifnumlines");
    define_authorcomment(true)?;
  }
  // aastex701.cls:6737 `\lambdabar`, the barred lambda (as revsymb_sty.rs has it).
  DefMath!("\\lambdabar", "\u{03BB}\u{0304}");

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
  // aastex631.cls:1336-1342: in a two-column document the appendix is set in one column (`\onecolumngrid`) unless
  // the class option `twocolappendix` asks for two: the page a float's panels are set in rows against follows (the
  // panel grid rule, `set_panel_page_twocolumn`; 2605.22754 A1.F6, two `0.48\textwidth` panels on a one-column page).
  DefPrimitive!("\\lx@aas@appendix@columns", {
    if !crate::shipped_class::class_option_given("twocolappendix")? {
      set_panel_page_twocolumn(false);
    }
  });
  DefMacro!("\\appendix", "\\lx@aas@appendix@columns\\@appendix");
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
  // `\references` (which already calls begin_bibliography);
  // landing the afterDigest hook alone is what closes the 4-paper
  // malformed:ltx:bibitem cluster. The attributes and the title are Perl's (aas_support.sty.ltxml:289-293): the
  // AAS `{references}` heading (aaspp4.sty:207 `\subsection*{REFERENCES}`, astro-ph9701025), which a bare `<ltx:bibliography>` dropped once
  // OmniBus no longer replaced this `\references` (astro-ph9708112, astro-ph9510040, `[11pt,aaspp4,tighten]{article}`).
  // …and before its body, the bibliography's own setup as revtex4_support's and OmniBus's `\references` have it
  // (`before_digest_bibliography`, Perl latex_constructs.pool.ltxml:3837-3844): with `\if@lx@inbibliography` unset, a
  // `\bibitem` that opens a bibliography when none is open (the kernel's, natbib's) opened a second one, leaving
  // this one's list empty (paspconf's `\reference`, astro-ph9909358).
  DefConstructor!(
    "\\references",
    "<ltx:bibliography xml:id='#id' bibstyle='#bibstyle' citestyle='#citestyle' sort='#sort'>\
       <ltx:title>#title</ltx:title><ltx:biblist>",
    before_digest => {
      // …but, opening no group, not its `\bibliography`/`\bibliographystyle` no-ops, which would last to the end of the
      // document and drop a real bibliography read later from the `.bbl` (Perl's `\references` runs no such setup,
      // aas_support.sty.ltxml:289-293). Repro index-bib/aas_references_then_real_bibliography.
      Let!("\\lx@aas@saved@bibliography", "\\bibliography");
      Let!("\\lx@aas@saved@bibliographystyle", "\\bibliographystyle");
      engine::latex_constructs::before_digest_bibliography()?;
      Let!("\\bibliography", "\\lx@aas@saved@bibliography");
      Let!("\\bibliographystyle", "\\lx@aas@saved@bibliographystyle");
    },
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
  // AASTeX's `\reference{key}` needs its key (aastex701.cls:8013-8016, aaspp4.sty:221-224 `\@ifnextchar\bgroup
  // {\@reference}{\@latexerr …Missing key…}`); RevTeX 3 v1.6's takes none, it starts a hanging paragraph
  // (paspconf.sty:188 `\def\reference{\relax\refpar}`). Perl's `Let('\reference','\bibitem')` (aas_support.sty.ltxml:299) read the next
  // token of a keyless one as its key: every reference lost its first letter ("Kent" → key K, "ent, S. M.") and a
  // `$\O$stensen` entry left math open over the rest of the list (astro-ph9909358, astro-ph9905150).
  DefMacro!("\\reference", "\\@ifnextchar\\bgroup{\\bibitem}{\\bibitem{}}");

  RequirePackage!("graphicx");

  // 2.14 Electronic Art
  DefMacro!("\\figurenum{}", "\\def\\thefigure{#1}");
  def_macro_noop("\\epsscale{}")?;
  // The plots are set at the line (aastex701.cls:8178-8189, emulateapj.cls:1040-1051: `0.85\linewidth`, `0.425\linewidth`
  // each, times `\epsscale`), here a whole line each (Perl's aas_support.sty.ltxml:317-319 `\textwidth`, the same width
  // where the page is not declared): against a two-column class's declared page (`declare_panel_page`) a one-column
  // figure's plot is the column, not 2.09 of it (emulateapj: 1408.6903 Fig 3, astro-ph0609442 Fig 1). The class's
  // 0.85 and `\epsscale` stay unmodelled, as in Perl.
  DefMacro!("\\plotone Semiverbatim", "\\includegraphics[width=\\linewidth]{#1}");
  DefMacro!("\\plottwo Semiverbatim Semiverbatim",
    "\\hbox{\\includegraphics[width=\\linewidth]{#1}\\includegraphics[width=\\linewidth]{#2}}");
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
  // Outside a figure the text is the figure's caption, numbered: AASTeX sets it with `\caption` under
  // `\def\@captype{figure}` (aastex701.cls:8111-8114; AASTeX 4's end-of-paper list prints "Fig. N.—"). Perl wraps the
  // bare text in the figure (aas_support.sty.ltxml:332), a `<p>` with no caption and no number, lost markup with 0
  // errors (astro-ph9806172; silent-loss inventory #24). Guard `perfect_kernel_batch74b::aastex_figcaption_outside_a_figure_is_its_caption`.
  DefMacro!("\\@figcaption {}", "\\begin{figure}\\caption{#1}\\end{figure}");
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
  // aastex701.cls TL :11298-11299 (aastex62.cls, aastex631.cls alike) `\def\deluxetable{\global\deluxetrue
  // \catcode`\&=\active \catcode`\$=\active…`: the table makes `$` active as it begins, before its template and its
  // `\tablehead` are read, so a `$m_{G}$` in the head of a `C` column is the cell's math, which the column's
  // `\let$\relax` leaves alone (2411.04196). `\aas@table@dollar` is that catcode change in AASTeX 6+ only (below).
  Let!("\\aas@@deluxetable", "\\deluxetable");
  DefMacro!("\\deluxetable", "\\aas@table@dollar\\aas@@deluxetable");
  let_i(&T_CS!("\\aas@@deluxetable*"), &T_CS!("\\deluxetable*"), None);
  def_macro(
    T_CS!("\\deluxetable*"),
    None,
    ExpansionBody::Tokens(Tokens!(T_CS!("\\aas@table@dollar"), T_CS!("\\aas@@deluxetable*"))),
    None,
  )?;
  // (after deluxetable, whose `\tablenum`, `\tablerefs`, `\tablecaption` and `\tabletypesize` these follow; the
  // AASTeX 6.x table commands listed above)
  RawTeX!(r"\def\tabletypesize#1{\gdef\currtabletypesize{#1}}\tabletypesize{\small}");
  {
    // (6.2 and later's only, as above)
    let _guess = OriginGuard::new(DefinitionOrigin::Fallback);
    RawTeX!(r"\let\savetablenum\tablenum\let\tablereferences\tablerefs\let\deluxetablecaption\tablecaption
\def\tabcaption{\@ifnextchar[{\@xtabcaption}{\@tabcaption}}\def\@tabcaption#1{{\def\@captype{table}\caption{#1}}}
\def\@xtabcaption[#1]#2{{\def\@captype{table}\caption{#2}}}");
  }
  Let!("\\planotable", "\\deluxetable");
  // AASTeX 3's `{planotable}` ends its data itself, without `\enddata` (aaspptwo.sty:252-256 `\endplanotable`:
  // `\crcr\noalign{\vskip .7ex}\hline\endtabular…`; astro-ph9502038); AASTeX 4's (aas2pp4.sty:322) is
  // `\enddeluxetable`, after an `\enddata`
  DefMacro!("\\endplanotable", "\\iflx@deluxetable@data\\enddata\\fi\\enddeluxetable");

  // Perl: aas_support.sty.ltxml L380-383
  Let!("\\splitdeluxetable", "\\deluxetable");
  Let!("\\endsplitdeluxetable", "\\enddeluxetable");
  let_i(&T_CS!("\\splitdeluxetable*"), &T_CS!("\\deluxetable*"), None);
  let_i(&T_CS!("\\endsplitdeluxetable*"), &T_CS!("\\enddeluxetable*"), None);

  // aastex631.cls L4780-4781:
  //   \newif\ifstartlongtable
  //   \def\startlongtable{\vskip1sp\global\startlongtabletrue}
  // Driver: 2209.01632 (aastex631) emitted "\startlongtable not defined" + alignment-tab cascade. The flag is the next
  // deluxetable's: a long table, whose caption the class always prints (deluxetable_sty.rs `\lx@deluxe@print@caption`,
  // aastex701.cls:11568/:11575), until it ends (:11609).
  DefMacro!("\\startlongtable", r"\global\lx@deluxe@longtrue");
  // aastex701.cls:11516 `\def\longdeluxetable#1` / :11614 `\def\endlongdeluxetable` are the long table `\deluxetable`
  // goes to after `\startlongtable` (:11316 `\let\go\longdeluxetable`); called as an environment
  // (`\begin{longdeluxetable}{ll}`) it is that table without the flag. Undefined, the environment was lost with its
  // caption and `\label` (2608.15711; pdflatex 0 errors, its `\ref` 2 as the table's number here).
  DefMacro!("\\longdeluxetable{}", r"\global\lx@deluxe@longtrue\deluxetable{#1}");
  DefMacro!("\\endlongdeluxetable", r"\enddeluxetable");

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
  // …only AASTeX 6+ (aastex_cls.rs sets it): AASTeX 5.x's aastex.cls (5.2; copies in 1403.0937, astro-ph0609108) and
  // emulateapj.cls have no active `$`, and an author's `$`-delimited macro there (`\def\mcn#1$-$#2 {…}` in a table,
  // 0903.2259) must find its catcode-3 delimiter (repro captions-floats/aastex5_table_dollar_delimited_macro).
  DefMacro!("\\aas@table@dollar", "");
  Let!("\\aas@@tabular@bindings", "\\@tabular@bindings");
  DefMacro!("\\@tabular@bindings", "\\aas@table@dollar\\aas@@tabular@bindings");
  Let!("\\aas@@deluxetable@bindings", "\\@deluxetable@bindings");
  DefMacro!("\\@deluxetable@bindings", "\\aas@table@dollar\\aas@@deluxetable@bindings");

  // Every AASTeX sets the `\tablenotetext` mark in math: aastex701.cls TL :11967-11969 (aastex62.cls :5097-5099,
  // aastex631.cls :5382-5384, 2503.18379's copy) `$^{\hbox to 5pt{$#1$}}$`, the older form (aastex631.cls :2001-2002)
  // `$^{#1}$`, while `\tablenotemark` sets its mark in text (:8494 `\textsuperscript{…\it #1}`). The mark is set in
  // the class's box (`\hbox{$#1$}`, so `\tablenotetext{$a$}` is the text "a", 1807.05231) and keyed by what it shows,
  // as the `\tablenotemark` is (deluxetable_sty.rs `shown_mark`), so `a`, `\rm{b}`, `$\dagger$` pair across the two
  // (1710.01715). Perl digests it as text (deluxetable.sty.ltxml:106), so `\tablenotetext{^\dagger}` set its `^` in
  // text (2503.18379, 0803.1504; KNOWN_PERL_ERRORS #873). emulateapj, which loads this binding, sets it in text
  // (emulateapj_cls.rs). Guards: `perfect_kernel_batch68y::aastex_tablenotetext_mark_is_math`,
  // `aastex_tablenote_marks_pair_by_what_they_show`.
  DefMacro!("\\tablenotetext{}{}", "\\lx@deluxe@tablenotetext{\\lx@deluxe@tablenote@mark{#1}}{#2}");

  // Decimal table conditionals — Perl L338-345
  DefConditional!("\\ifcolnumberson");
  DefConditional!("\\ifdeluxedecimals");
  DefMacro!("\\deluxedecimals", "\\global\\deluxedecimalstrue");
  RawTeX!("\\global\\deluxedecimalsfalse");
  // aastex701.cls TL :12014-12021 (aastex62.cls :5144, 1805.06850's copy) `\gdef\decimals{\noalign{…}}`: in a
  // `tabular` the switch is a `\noalign` at a row start, so a `\tableline` after it is still a rule; each
  // deluxetable lets it be `\deluxedecimals` (:11331, 11425, 11520, 11714; aastex62.cls :4612, 4661), an assignment
  // before `\startdata`. Perl's assignment everywhere (aas_support.sty.ltxml:345-347) opened a cell at the row start
  // and misplaced the `\noalign` after it (1805.06850; KNOWN_PERL_ERRORS #875).
  // Guard: `perfect_kernel_batch68y::aastex_decimals_is_a_noalign_in_a_tabular`.
  DefMacro!("\\decimals", "\\noalign{\\global\\deluxedecimalstrue}");
  // aastex701.cls TL :11330, 11424, 11519, 11715 `\global\deluxedecimalsfalse`: each deluxetable (and its starred and
  // split forms, which all set their template here) starts without `\decimals`. The `\let` stays in the group of
  // the environment's `\begin`.
  Let!("\\aas@@set@deluxetable@template", "\\set@deluxetable@template");
  DefMacro!("\\set@deluxetable@template",
    "\\global\\deluxedecimalsfalse\\let\\decimals\\deluxedecimals\\aas@@set@deluxetable@template");
  def_macro_noop("\\colnumbers")?;
  DefMacro!("\\deluxedecimalcolnumbers", "\\deluxedecimalstrue\\colnumbersontrue");
  Let!("\\decimalcolnumbers", "\\deluxedecimalcolnumbers");

  // The hidden column's cell (`h`, `B` below): aastex701.cls TL :8851-8852 (aastex62.cls :2143-2144) `\eatone` =
  // `\setbox0=\hbox\bgroup\savedollar\let$\relax`, a box that is never used, holding the cell in math, where the
  // table's active `$` does nothing; `\endeatone` closes the math and the box. Perl's `{eatone}` environment (L374)
  // digested the cell as text, so a `{grp1.8_diffuse}` there was a "Script _" error (1907.13126, 76; KNOWN_PERL_ERRORS #991). The class's
  // `\hskip-2\tabcolsep`, which takes the hidden column's padding back, is presentation and is left out.
  RawTeX!(r"\begingroup\catcode`\$=\active
\long\gdef\eatone{\setbox0=\hbox\bgroup\savedollar\let$\relax}\gdef\endeatone{\savedollar\egroup}\endgroup");

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
  // The class's `D` is a column type of its own (aastex701.cls TL :12010, aastex62.cls :5140), which a document's
  // `\usepackage{dcolumn}` after the class replaces with dcolumn's three-argument `D{.}{.}{n}` (the later
  // `\newcolumntype` wins; aastex701.cls :6286-6291 then only adds a `d`). The binding has dcolumn already (REVTeX's,
  // revtex4_support_sty.rs, as Perl's), so the document's load changes nothing: a `D` followed by its arguments is
  // dcolumn's, the class's never takes one. Perl's `D` replaced dcolumn's (aas_support.sty.ltxml:362-363), its
  // arguments then unknown columns (1607.06811 `\newcolumntype{d}[1]{D{,}{\pm}{#1}}`; KNOWN_PERL_ERRORS #876).
  // Guard: `perfect_kernel_batch68y::aastex_d_column_with_arguments_is_dcolumns`.
  if lookup_meaning(&T_CS!("\\NC@rewrite@D")).is_some() {
    let_i(&T_CS!("\\lx@aas@dcolumn@D"), &T_CS!("\\NC@rewrite@D"), None);
  }
  DefColumnType!("D", {
    skip_spaces()?;
    if if_next(T_BEGIN!())? && lookup_meaning(&T_CS!("\\lx@aas@dcolumn@D")).is_some() {
      Tokens!(T_CS!("\\lx@aas@dcolumn@D"))
    } else {
      build_d_columns();
      Tokens!()
    }
  });
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

  // aastex701.cls:13650-13674 `\interactive#1#2` only tests its arguments (the type `lc`/`js`/`animation`, the file),
  // never typesets them: a file `rolling_ns3.mp4` raised "Script _ can only appear in math mode" (2303.02355;
  // OXIDIZED_DESIGN #551). Guard `perfect_kernel_batch65m::file_arguments_are_not_typeset`.
  DefEnvironment!("{interactive} Semiverbatim Semiverbatim", "#body");
  // aastex701.cls:12245-12286 (aastex631.cls alike) `{longrotatetable}` turns the page for the long table it holds, a
  // deluxetable it sets non-floating (`\global\floatfalse`, `\startlongtabletrue`). Around a plain `tabular` it is that
  // table's float here (main-session call 2026-10-18, under user ruling 2 "keep the author's caption"; OXIDIZED_DESIGN
  // _DIVERGENCES #739), its `\label` the table's. Its `\tablecaption` is stored, as the class stores it
  // (deluxetable_sty.rs `\lx@deluxe@tablecaption`, `in_longrotatetable_body`): a deluxetable in it takes the store, and
  // the first `\begin{tabular}` directly in it sets a caption still stored, unnumbered — the PDF never prints it, so
  // the later numbers stay the PDF's (2204.03327). Its `\global\startlongtabletrue` makes the next deluxetable a long
  // one (`\lx@deluxe@longtrue`), whose caption the PDF always prints, numbered (repro
  // class-bindings/longrotatetable_tablecaption_goes_to_its_table). Around a deluxetable —
  // whose own `{table}` the body then holds — it stays the page's setting, the body alone.
  // A caption no `\begin{tabular}` took (a `tabular*`, `tabularx` or `longtable`, or a `\tablecaption` after the
  // tabular) is set at the end, in its group, unnumbered (review 73a r3 finding 1).
  // Guards: `perfect_kernel_batch73a::{longrotatetable_around_a_tabular_is_its_table,
  // longrotatetable_tablecaption_goes_to_its_table, longrotatetable_caption_without_a_tabular}`.
  DefEnvironment!("{longrotatetable}", sub[document, _args, props] {
    let body = match props.get("body") {
      Some(Stored::Digested(body)) => Some(body.clone()),
      _ => None,
    };
    if matches!(props.get("holds_table"), Some(Stored::Bool(true))) {
      if let Some(body) = body {
        document.absorb(&body, None)?;
      }
      return Ok(());
    }
    let save = document.float_to_element("ltx:table", true)?;
    let mut attrs = string_map!();
    for key in ["id", "inlist"] {
      if let Some(value) = props.get(key).map(|v| v.to_string()).filter(|v| !v.trim().is_empty()) {
        attrs.insert(if key == "id" { "xml:id".to_string() } else { key.to_string() }, value);
      }
    }
    document.open_element("ltx:table", Some(attrs), None)?;
    if let Some(Stored::Digested(tags)) = props.get("tags") {
      document.absorb(tags, None)?;
    }
    if let Some(body) = body {
      document.absorb(&body, None)?;
    }
    document.close_element("ltx:table")?;
    if let Some(save) = save {
      document.set_node(&save);
    }
    Ok(())
  },
    properties => { stored_map!("layout" => "vertical") },
    mode => "internal_vertical",
    before_digest => {
      before_float_ex("table", None, true)?;
      engine::latex_constructs::reset_float_box()?;
      assign_value("lx@lrt@frame", lookup_value("lx@float@frame"), Some(Scope::Local));
      // Before `\begin{tabular}`'s group (its `beforebegin` hook, kept after ours): `\tabular` is locked.
      let hook = "@environment@tabular@beforebegin";
      if matches!(lookup_value(hook), None | Some(Stored::None) | Some(Stored::Tokens(_))) {
        let mut tokens = vec![T_CS!("\\lx@lrt@tabular@caption")];
        tokens.extend(lookup_tokens(hook).map(Tokens::unlist).unwrap_or_default());
        assign_value(hook, Stored::Tokens(Tokens::new(tokens)), Some(Scope::Local));
      }
      // A table float in the body (a deluxetable's `{table}`) makes the body the page's setting (`holds_table_float`),
      // and a `{longtable}` is a numbered table of its own, with its id: their `beforebegin` hooks note them here, in
      // the longrotatetable's group, for the end's fallback.
      for (env, note) in [("table", "\\lx@lrt@note@table"), ("table*", "\\lx@lrt@note@table"),
        ("longtable", "\\lx@lrt@note@longtable")]
      {
        let hook = s!("@environment@{env}@beforebegin");
        let hook = hook.as_str();
        if matches!(lookup_value(hook), None | Some(Stored::None) | Some(Stored::Tokens(_))) {
          let mut tokens = vec![T_CS!(note)];
          tokens.extend(lookup_tokens(hook).map(Tokens::unlist).unwrap_or_default());
          assign_value(hook, Stored::Tokens(Tokens::new(tokens)), Some(Scope::Local));
        }
      }
      Digest!(r"\global\lx@deluxe@longtrue")?;
    },
    // The end, inside its group: a caption still stored — no `tabular` took it (a `tabular*`, `tabularx` or
    // `longtable`, or a `\tablecaption` after the tabular) — is set here, unnumbered, as at a `tabular`.
    before_digest_end => { digest(Tokens!(T_CS!("\\lx@lrt@end@caption")))? },
    after_digest => sub[whatsit] { after_float(whatsit); },
    after_digest_body => sub[whatsit] {
      if let Ok(Some(body)) = whatsit.get_body() {
        whatsit.set_property("holds_table", Stored::Bool(holds_table_float(&body)));
      }
    });
  DefMacro!("\\lx@lrt@tabular@caption", sub[_args] {
    Ok(if deluxetable_sty::in_longrotatetable_body() { Tokens!(T_CS!("\\lx@lrt@stored@caption")) } else { Tokens!() })
  });
  DefPrimitive!("\\lx@lrt@note@table", {
    if deluxetable_sty::in_longrotatetable_body() {
      assign_value("lx@lrt@holds@table", Stored::Bool(true), Some(Scope::Local));
    }
  });
  DefPrimitive!("\\lx@lrt@note@longtable", {
    if deluxetable_sty::in_longrotatetable_body() {
      assign_value("lx@lrt@holds@longtable", Stored::Bool(true), Some(Scope::Local));
    }
  });
  // A held `{longtable}` has its number and id (longtable_sty.rs steps at its start), so its caption takes no id step.
  DefMacro!("\\lx@lrt@end@caption", sub[_args] {
    Ok(if !deluxetable_sty::in_longrotatetable_body() || lookup_bool("lx@lrt@holds@table") {
      Tokens!()
    } else if lookup_bool("lx@lrt@holds@longtable") {
      Tokens!(T_CS!("\\lx@lrt@stored@caption@noid"))
    } else {
      Tokens!(T_CS!("\\lx@lrt@stored@caption"))
    })
  });
  DefMacro!("\\lx@lrt@stored@caption@noid",
    r"\ifx\lx@deluxe@caption\@empty\else\expandafter\lx@deluxe@caption@noid\expandafter{\lx@deluxe@caption}\global\let\lx@deluxe@caption\@empty\fi");
  DefMacro!("\\lx@deluxe@caption@noid{}", r"\lx@lrt@longtable@id\lx@setfloatcapt{\lx@@caption{#1}}");
  // The table then is the longtable's: its id (the longtable steps at its start, longtable_sty.rs), which the collapse
  // of the two tables keeps (`collapse_float`).
  DefPrimitive!("\\lx@lrt@longtable@id", {
    if let Some(Stored::HashStored(props)) = lookup_value("LONGTABLE_PROPERTIES")
      && lookup_value("table_id").is_none_or(|v| matches!(v, Stored::None))
    {
      assign_value("table_id", props.get("id"), Some(Scope::Global));
    }
  });
  DefMacro!("\\lx@lrt@stored@caption",
    r"\ifx\lx@deluxe@caption\@empty\else\expandafter\lx@deluxe@unnumbered\expandafter{\lx@deluxe@caption}\global\let\lx@deluxe@caption\@empty\fi");
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
/// `\authorcomment<n>{text}` (aastex631.cls:5843-5850 in 2107.07519's copy), "(Author~note n: text)" unless the edit
/// switches turn it off; `numbered` false for the versions that print "(Author~note: text)" — aastex63.cls (98 of the
/// 102 copies sampled: 2205.09859 :5727-5734, 2004.09658) and aastex62.cls (`aastex_cls`: the paper's copy decides,
/// else the class name). The definer runs at the caller's origin.
pub(crate) fn define_authorcomment(numbered: bool) -> Result<()> {
  let n = |k: u8| if numbered { s!(" {k}") } else { String::new() };
  RawTeX!(&s!(
    r"\def\authorcomment#1#2{{{{\colorcount=#1 \ifturnoffedit\let\go\ignorespaces\else\let\go\relax
\color{{\csname editcolor\the\colorcount\endcsname}}%
\ifnum\colorcount=1{{\ifturnoffone\else\bf (Author~note{}: #2)\ \fi}}\else
\ifnum\colorcount=2{{\ifturnofftwo\else\it (Author~note{}: #2)\ \fi}}\else
\ifnum\colorcount=3{{\ifturnoffthree\else (Author~note{}: #2)\ \fi}}\fi\fi\fi\fi\go}}}}",
    n(1),
    n(2),
    n(3)
  ));
  Ok(())
}

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
