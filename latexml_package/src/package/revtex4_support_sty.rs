use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: revtex4_support.sty.ltxml — 433 lines
  // Support macros for RevTeX4 class (APS journals)

  RequirePackage!("hyperref");
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
  DefMacro!("\\title[]{}",
    "\\gdef\\@shorttitle{#1}\\gdef\\@title{#2}\\ifx.#1.\\else\\lx@add@toctitle{#1}\\fi\\lx@add@title{#2}");

  // \author[labels]{name}   One \author per author
  // If labels given, the corresponding affiliation from \affil is attached
  // otherwise, \author should be followed by \affiliation
  // `\author{Brent Preston and Eric Poisson}` names two (gr-qc0606093): the article `\author`'s parse, each author
  // with the labels (Perl revtex4_support makes it one author; 63e).
  // `\lx@author@after` (sect05.rs): what a package appends to `\author` (titling's `\theauthor`) runs here too.
  DefMacro!("\\author[]{}",    "\\lx@add@authors@append[annotations={#1}]{#2}\\lx@author@after{#2}");
  // An `\affiliation` that starts with a letter or number mark (all `\author{Name$^{a}$}`s first, then `$^{a}$`-marked
  // lines: 2011.01984, 2301.08449) is labelled by its marks under marked authors, and goes to the authors showing them
  // (relocate_annotations); any other is revtex's group rule, every author before it without one (`annotate=new`): a
  // superscript inside the line (`Laboratory for $^{3}$He`) or a symbol (`$^{\dagger}$`) is no mark, and an
  // interleaved `\author{A$^{1}$}\affiliation{$^{1}$…}` group keeps the group rule, exact there (63i review).
  DefMacro!("\\affiliation{}", sub[(content)] {
    let attr = Some(mouth::tokenize_internal("annotate=new"));
    let cs = if leads_with_mark(&content)
      && !queued_group_shares_the_mark(&content)
    {
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
  DefMacro!("\\@affiliation{}", "\\endgroup\\lx@add@affiliation[annotate=new]{#1}");
  DefMacro!("\\noaffiliation", "\\lx@add@affiliation[annotate=new]{}");
  DefMacro!("\\affil OptionalSemiverbatim {}",
    "\\lx@add@affiliation[annotate={\\ifx.#1.new\\else 1\\fi},label={#1}]{#2}");
  // \address provides an address to the previous \author
  DefMacro!("\\address{}", "\\lx@add@address[annotate=new]{#1}");
  // These add contacts to most recent author
  // The optional arguments here are a sort of prefix to the footnote. (NOT label)
  // REVTeX 4.1/4.2 set the address with `\href`, without hyperref `\@href`: in the current font
  // (revtex4-1.cls:1959-1961, 2383-2391), its `_` as typed (revtex4_cls.rs sets REVTeX 4.0's in typewriter type;
  // 2301.03570, 2301.03067)
  DefMacro!("\\email[] Semiverbatim",    "\\lx@add@email[name={#1}]{\\lx@address@textfont{#2}}");
  DefMacro!("\\homepage[] Semiverbatim", "\\lx@add@url[name={#1}]{\\lx@address@textfont{#2}}");
  // revtex4-1.cls:2383-2392 (revtex4.cls:4481-4491, revtex4-2.cls:2404-2413): `\email` is `\begingroup\sanitize@url
  // \@email{#1}`, and `\@email{label}{address}` closes that group. A class that re-dresses `\email` over them
  // (cej.cls:88 `\renewcommand*\email[1][E-mail: ]{\begingroup\sanitize@url\@email{#1}}`; 0710.1022, 1201.4230 and 5
  // more) left the group open without them, `%` other to the end of the document.
  DefMacro!("\\@email{} Semiverbatim",    "\\endgroup\\lx@add@email[name={#1}]{\\lx@address@textfont{#2}}");
  DefMacro!("\\@homepage{} Semiverbatim", "\\endgroup\\lx@add@url[name={#1}]{\\lx@address@textfont{#2}}");
  DefMacro!("\\thanks[]{}",              "\\lx@add@contact[role=thanks,name={#1}]{#2}");
  // revtex4-2.cls:2472-2493 (revtex4.cls, revtex4-1.cls alike): the front-matter footnote under `\thanks` and the
  // `\author`'s optional notes, a mark shared by identical texts; documents call it for a note on several authors
  // (`\newcommand{\equalcontrib}{\frontmatter@footnote{…contributed equally…}}`, 2605.04155). Each author keeps it.
  DefMacro!("\\frontmatter@footnote{}",  "\\lx@add@contact[role=thanks]{#1}");
  // revtex4-2.cls:2120/2160/2236 `\@AF@join{note}`: the join every `\email`, `\homepage`, `\thanks` and
  // `\altaffiliation` ends in (:2404-2421), dispatched to the title, author or affiliation last given. A class that
  // re-dresses `\@email` calls it directly (JASA.cls:1232-1238 `\def\@email#1#2{\endgroup\@AF@join{#1\href{mailto:#2}
  // {#2}}}`; 2605.18495, 2605.23416: `\@AF@join` undefined). The note goes where our `\thanks` puts its notes, on the
  // last author, whatever revtex's dispatch would join it to, and as a thanks note even when it holds an email: the
  // content and its link are kept.
  DefMacro!("\\@AF@join{}",              "\\lx@add@contact[role=thanks]{#1}");
  DefMacro!("\\orcid[]{}",               "\\lx@add@orcid[name={#1}]{#2}"); // ?

  DefMacro!("\\collaboration{}",    "\\author{#1}");
  DefMacro!("\\altaffiliation[]{}", "\\lx@add@contact[annotate=new,role=affiliation,name={#1}]{#2}");

  def_macro_noop("\\firstname")?;
  DefConstructor!("\\surname{}", "#1", enter_horizontal => true);

  // 4.4 Abstract
  DefMacro!("\\abstractname", "Abstract");

  DefMacro!("\\pacs{}",     "\\lx@add@classification[scheme=pacs,name={PACS:~}]{#1}");
  DefMacro!("\\keywords{}", "\\lx@add@keywords{#1}");
  DefMacro!("\\preprint{}", "\\lx@add@pubnote[role=preprint]{#1}");

  // Extra
  def_macro_noop("\\blankaffiliation")?;
  DefMacro!("\\checkindate", "\\today");

  DefMacro!("\\received[]{}", "\\lx@add@date[role=received]{#2}");
  DefMacro!("\\revised[]{}", "\\lx@add@date[role=revised]{#2}");
  DefMacro!("\\accepted[]{}", "\\lx@add@date[role=accepted]{#2}");
  DefMacro!("\\published[]{}", "\\lx@add@date[role=published]{#2}");

  // 5.3 Widetext
  def_macro_noop("\\widetext")?;
  def_macro_noop("\\endwidetext")?;
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

  // Grid / column macros
  DefMacro!("\\thepagegrid", "one");
  def_macro_noop("\\onecolumngrid")?;
  def_macro_noop("\\twocolumngrid")?;
  def_macro_noop("\\restorecolumngrid")?;
  // revtex4-1.cls L4388: \do@columngrid{layout}{N}. Layout-only.
  // Witness 2406.02666 (revtex4-1 with explicit \onecolumngrid call
  // before our stub binding loads).
  def_macro_noop("\\do@columngrid{}{}")?;
  // revtex4-2.cls:7424 `\close@column@grid` ends the current column grid before
  // a page break; layout-only. A paper's own `\balancecolsandclearpage`
  // (`\close@column@grid\clearpage\twocolumngrid`) calls it: arXiv 2605.07942.
  def_macro_noop("\\close@column@grid")?;
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
  DefConstructor!("\\text{}", "<ltx:text _noautoclose='true'>#1</ltx:text>",
    mode => "restricted_horizontal", locked => true);

  // RevTeX3 bold math (obsolete in RevTeX4) — Perl L165-171. RevTeX 4.x's own `\bm` (revtex4-2.cls:7573-7586,
  // revtex4-1.cls:7552-7565, revtex4.cls:6018-6032) asks for bm.sty ("To use \bm, please load the bm package!"),
  // drops its first argument and leaves later ones in a `\relax{…}` group; Perl keeps a bold `\bm`, its argument in a
  // group: a `\left` without its `\right` there is set at the outer level with a Warning (bm_sty.rs
  // `define_grouped_bm`, OXIDIZED_DESIGN #544).
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
