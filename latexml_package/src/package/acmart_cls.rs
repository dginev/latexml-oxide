//! acmart.cls — ACM article class
//! Perl: acmart.cls.ltxml (259 lines)
use crate::{
  engine::latex_constructs::{after_float, before_float},
  prelude::*,
};

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: LoadClass('amsart', withoptions => 1)
  load_class_with_options("amsart", Tokens!())?;

  // Beyond-Perl fidelity (OXIDIZED_DESIGN "acmart establishes T1 font
  // encoding"): real acmart.cls loads libertine + `\RequirePackage[T1]{fontenc}`
  // (acmart.cls L867-881), so `<`/`>`/`|`/`\`/`{`/`}`/`_`/`"` are LITERAL in the
  // PDF. Neither LaTeXML binding modeled this, so both defaulted to OT1 where
  // `<`->¡, `>`->¿ (witness arXiv:2405.17739 `num < 0 && num > 0`). Perl leaves
  // it at OT1; we honor acmart's real T1 to match the PDF. Divergence from Perl.
  RequirePackage!("fontenc", options => vec!["T1".to_string()]);

  RequirePackage!("fancyhdr");
  RequirePackage!("geometry");
  RequirePackage!("comment");
  RequirePackage!("natbib");
  RequirePackage!("textcomp");
  RequirePackage!("graphicx");
  // Real acmart.cls passes [prologue,table]{xcolor} but doesn't pass
  // dvipsnames; many user papers nevertheless use Cerulean / ForestGreen
  // etc. without an explicit \\usepackage[dvipsnames]{xcolor}. Pre-load
  // the extended palette eagerly so the named colors resolve. Witness
  // 2 acmart papers/100k cluster with `Error:unexpected:ForestGreen`.
  RequirePackage!("xcolor", options => vec!["dvipsnames".to_string()]);
  // RequirePackage('totpages');
  RequirePackage!("microtype");
  RequirePackage!("hyperref");
  RequirePackage!("caption");
  RequirePackage!("float");
  // RequirePackage('environ');
  // RequirePackage('zi4');
  RequirePackage!("setspace");
  RequirePackage!("newtxmath");
  // RequirePackage('manyfoot');
  // RequirePackage('libertine');

  // Added based on acmart.cls in texlive 2020
  RequirePackage!("xkeyval");
  // RequirePackage('xstring');
  RequirePackage!("iftex");
  RequirePackage!("etoolbox");
  RequirePackage!("booktabs");
  RequirePackage!("refcount");
  RequirePackage!("textcase");
  RequirePackage!("hyperxmp");
  // RequirePackage('draftwatermark');
  // RequirePackage('cmap');
  // RequirePackage('pbalance');
  RequirePackage!("balance");

  //======================================================================
  // Various bits of frontmatter
  DefMacro!("\\copyrightyear{}", "\\lx@add@copyrightyear{#1}");
  // This should be keyvals!
  DefMacro!("\\setcopyright{}", "\\lx@add@copyright{#1}");
  DefMacro!("\\received[]{}", "\\lx@add@date[role=received]{#2}");
  DefMacro!("\\acmJournal{}", "\\lx@add@pubnote[role=journal]{#1}");
  DefMacro!("\\acmSubmissionID{}", "\\lx@add@pubnote[role=submissionid]{#1}");
  DefMacro!("\\acmConference[]{}{}{}", "\\lx@add@pubnote[role=conference]{#2; #3; #4}");
  DefMacro!("\\acmBooktitle{}", "\\lx@add@pubnote[role=booktitle]{#1}");
  DefMacro!("\\acmArticle{}", "\\lx@add@pubnote[role=article]{#1}");
  DefMacro!("\\acmArticleSeq{}", "\\lx@add@pubnote[role=articleseq]{#1}");
  DefMacro!("\\acmDOI{}", "\\lx@add@pubnote[role=doi]{#1}");
  DefMacro!("\\acmISBN{}", "\\lx@add@pubnote[role=isbn]{#1}");
  DefMacro!("\\acmMonth{}", "\\lx@add@pubnote[role=publicationmonth]{#1}");
  DefMacro!("\\acmNumber{}", "\\lx@add@pubnote[role=number]{#1}");
  DefMacro!("\\acmPrice{}", "\\lx@add@pubnote[role=price,name={Price:~}]{#1}");
  DefMacro!("\\acmVolume{}", "\\lx@add@pubnote[role=volume]{#1}");
  DefMacro!("\\acmYear{}", "\\lx@add@date[role=published]{#1}");
  DefMacro!("\\subtitle{}", "\\lx@add@subtitle{#1}");
  DefMacro!("\\keywords{}", "\\lx@add@keywords{#1}");
  DefMacro!("\\terms{}", "\\lx@add@keywords{#1}");

  //======================================================================
  // Accessible figure descriptions
  // Register WAI-ARIA namespace for accessible descriptions
  RegisterDocumentNamespace!("aria", "http://www.w3.org/ns/wai-aria");

  NewCounter!("acmlabel", "");
  // `\Description[short]{long}` — acmart's accessible figure description
  // (acmart.cls L895 gobbles both args; ACM requires the description, and HTML
  // can carry it where PDF cannot, so we surface it rather than follow the
  // class into discarding it).
  //
  // The MANDATORY long description is the real alt text, so it is what we
  // emit. It is read `Undigested` — `ExpansionLevel::Off`, kept as
  // `DigestedData::Postponed` tokens — so nothing inside it expands: the
  // author never sees a defect there under pdflatex (the class gobbles it), so
  // expanding it only manufactures errors. Witness arXiv:2607.21760, whose
  // `\D1 … \D5` (a copy-paste slip from the adjacent `alt=` text, which has
  // plain `D1 … D5`) raised `Error:undefined:\D` for content we then dropped.
  //
  // `LaTeXML-common.xsl` L404-421 maps any `aria:*` attribute to `aria-*`
  // under HTML5, so setting them here needs no XSLT change. Which ARIA slot
  // each argument lands in is the `before_construct` table below.
  //
  // `[short]` and `{long}` are two DISTINCT authored fields, so they get two
  // distinct elements — never concatenated into one. Merging them yields a
  // run-on ("Fly 1 and Fly 2 look identical. Fly 1 and fly 2 comparison
  // shows…") and destroys the distinction, so no consumer can tell which text
  // the author wrote as the brief alternative.
  //
  // The old binding emitted the short one ALONE, which is why
  // `t/complex/acm_aria` recorded "Fly 1 and Fly 2 look identical" and lost the
  // sentence that actually describes the figure.
  //
  // Carried as `<ltx:note>` — the document builder places a note inside the
  // float's `<caption>`, where the old binding put it. An `<ltx:text>` is
  // INLINE, so the builder auto-opens a `<p>` for it, and a `<p>` at figure
  // level is tagged `ltx_figure_panel`, which makes the post-processor spin up
  // a whole `ltx_flex_figure`/`ltx_flex_cell` wrapper around two hidden spans
  // (inert, since the caption partitions it and the real subfigures keep their
  // width class, but pure noise in the output).
  //
  // The footnote decoration a note normally gets — the `†` mark and the
  // `<role>: ` type prefix — is suppressed for these by a dedicated template in
  // `LaTeXML-meta-xhtml.xsl` keyed on `ltx_acm_description`, so the referenced
  // text stays clean for assistive technology.
  //
  // The short description gets its own id so it stays individually
  // addressable (nothing references it today — its text goes into
  // `aria-label` — but an anchor on authored content costs nothing). It is
  // derived in `properties` rather than written as `#id-short` in the
  // template, which does NOT work: a `#name` hole runs to the end of the
  // identifier, so `#id-short` names a property called `id-short` — absent,
  // so the element silently emits NO xml:id at all — instead of `#id`
  // followed by a literal `-short`. Ids use a `-short` suffix rather than a
  // dotted one so they need no escaping in a CSS selector.
  DefConstructor!("\\Description[] Undigested",
    "^^?#1(<ltx:note xml:id='#shortid' class='ltx_nodisplay ltx_acm_description_short'>#1</ltx:note>)()\
     <ltx:note xml:id='#id' class='ltx_nodisplay ltx_acm_description'>#2</ltx:note>",
    properties => {
      let mut props = RefStepCounter!("acmlabel")?;
      if let Some(id) = props.get("id") {
        let short = format!("{id}-short");
        props.insert("shortid", Stored::from(short));
      }
      // Perl acmart.cls.ltxml L79 sets these so the hidden note claims no
      // layout space; carried over.
      props.insert("width", Stored::from(Dimension!("0pt")));
      props.insert("height", Stored::from(Dimension!("0pt")));
      Ok(props)
    },
    // acmart's own documentation: "Unlike \caption, which is used alongside the
    // image, \Description is intended to be used INSTEAD OF the image." So a
    // `\Description` is a TEXT ALTERNATIVE, not supplementary prose — which in
    // ARIA is name-like (`aria-label`), not `aria-describedby` (announced in
    // ADDITION to the name). That also fixes the two arguments' roles: `[short]`
    // is the concise alternative, `{long}` the extended description.
    //
    //   `\Description[s]{l}`  → aria-label = s, aria-describedby → l's block
    //   `\Description{l}`, l plain    → aria-label = l (it replaces the image)
    //   `\Description{l}`, l w/ markup → aria-describedby → l's block
    //
    // The last case is why the argument is read `Undigested`: an `aria-label`
    // is a plain string and cannot carry markup, so we must inspect the tokens
    // to choose the slot BEFORE expanding anything. A control sequence (or an
    // active/`$`/`^`/`_` token) means real markup, so the block carries it.
    //
    // The block is always emitted, so the text stays addressable, but it is
    // referenced only when it is not already the label — otherwise the same
    // sentence would be both the name and the description. An unreferenced
    // hidden block is inert: `display:none` content is announced only when
    // something references it.
    //
    // acmart's NEW mechanism for the same purpose is `\includegraphics[alt=…]`
    // (switched on by `\DocumentMetadata`), handled in `graphicx_sty.rs` and
    // landing on the `<img>` itself. A document using BOTH — e.g.
    // arXiv:2607.21760, which repeats the same paragraph in each — will convey
    // it twice; that is the author's duplication across two documented
    // mechanisms, not something to second-guess here, and `\Description` is
    // scoped to the float with no reliable way to associate it with one image.
    before_construct => sub[document, whatsit] {
      let Some(id) = whatsit.get_property("id").map(|v| v.to_string()) else {
        return Ok(());
      };
      // Does the long description contain markup, or is it plain text?
      let long_is_plain = whatsit
        .get_arg(2)
        .and_then(|d| d.raw_tokens())
        .is_some_and(|tks| {
          tks.unlist_ref().iter().all(|t| {
            matches!(
              t.get_catcode(),
              Catcode::LETTER | Catcode::OTHER | Catcode::SPACE | Catcode::EOL
            )
          })
        });
      let short = whatsit.get_arg(1).map(|d| d.to_attribute());
      let Some(mut figure) = document.get_element() else {
        return Ok(());
      };
      match short {
        // Concise alternative available: it labels, the long one describes.
        Some(s) => {
          document.set_attribute(&mut figure, "aria:label", &s)?;
          document.set_attribute(&mut figure, "aria:describedby", &id)?;
        },
        // Lone description: it stands in for the image, so it labels — unless
        // it carries markup an attribute cannot hold.
        None if long_is_plain => {
          let text = whatsit.get_arg(2).map(|d| d.to_attribute()).unwrap_or_default();
          document.set_attribute(&mut figure, "aria:label", &text)?;
        },
        None => {
          document.set_attribute(&mut figure, "aria:describedby", &id)?;
        },
      }
    }
  );

  //======================================================================
  // Use \author for EACH author, follow with \orcid, \affiliation, \email as needed.
  // Note that \affiliation can apply to all preceding authors without one
  // (Perl PR #2767)
  // Real acmart is \renewcommand\author[2][]: optional [short-name] (running
  // head only) + mandatory full name. Perl binds only \author{}, so a real
  // \author[F. Poli]{Federico Poli} leaks '[' and drops the name; accept the
  // optional short-name and drop it (beyond-Perl; the short name is a derived
  // running-head abbreviation, not new information).
  DefMacro!("\\author[]{}",              "\\lx@add@creator[role=author]{#2}");
  DefMacro!("\\editor{}",                "\\lx@add@creator[role=editor]{#1}");
  DefMacro!("\\affiliation{}",           "\\lx@add@contact[role=affiliation,annotate=new]{#1}");
  DefMacro!("\\additionalaffiliation{}", "\\lx@add@contact[role=altaffiliation]{#1}");
  DefMacro!("\\email [] Semiverbatim",   "\\lx@add@contact[role=email,name={email: }]{#2}");
  DefMacro!("\\orcid{}",                 "\\lx@add@contact[role=orcid, name={OrcID: }]{#1}");

  //======================================================================
  // Internal structure to affiliation (Perl PR #2767: comma-joined parts;
  // empty parts skipped)
  DefMacro!("\\lx@acm@addresspartsep", "");
  DefMacro!("\\lx@acm@addresspart{}{}",
    "\\ifx.#2.\\else\\lx@acm@addresspartsep\\def\\lx@acm@addresspartsep{,~}\\lx@acm@addresspart@{#1}{#2}\\fi");
  DefConstructor!("\\lx@acm@addresspart@{}{}",
    "<ltx:text class='ltx_affiliation_#1' _noautoclose='1'>#2</ltx:text>",
    mode => "restricted_horizontal", enter_horizontal => true);
  DefMacro!("\\position{}",      "\\lx@acm@addresspart{position}{#1}");
  DefMacro!("\\institution{}",   "\\lx@acm@addresspart{institution}{#1}");
  DefMacro!("\\department{}",    "\\lx@acm@addresspart{department}{#1}");
  DefMacro!("\\streetaddress{}", "\\lx@acm@addresspart{streetaddress}{#1}");
  DefMacro!("\\city{}",          "\\lx@acm@addresspart{city}{#1}");
  DefMacro!("\\state{}",         "\\lx@acm@addresspart{state}{#1}");
  DefMacro!("\\postcode{}",      "\\lx@acm@addresspart{postcode}{#1}");
  DefMacro!("\\country{}",       "\\lx@acm@addresspart{country}{#1}");

  DefMacro!("\\titlenote{}",    "\\lx@add@pubnote[role=note]{#1}");
  DefMacro!("\\subtitlenote{}", "\\lx@add@pubnote[role=note]{#1}");
  DefMacro!("\\authornote{}",   "\\lx@add@contact[role=note]{#1}");

  DefMacro!("\\abstract",    "\\lx@begin@abstract");
  DefMacro!("\\endabstract", "\\lx@end@abstract");

  // Rust-only content preserves (Perl gobbles these)
  DefMacro!("\\shortauthors{}", "\\lx@add@frontmatter{ltx:note}[role=shortauthors]{#1}");
  def_macro_noop("\\authornotemark[]")?;
  DefMacro!("\\authorsaddresses{}",
    "\\lx@add@frontmatter{ltx:note}[role=authorsaddresses]{#1}");
  def_macro_noop("\\startPage")?;
  def_macro_noop("\\settopmatter{}")?;
  def_macro_noop("\\copyrightpermissionfootnoterule")?;
  def_macro_noop("\\acmBadgeL")?;

  //======================================================================
  // Natbib cite aliases
  Let!("\\citeN", "\\cite");
  Let!("\\cite", "\\citep");
  Let!("\\citeANP", "\\citeauthor");
  Let!("\\citeNN", "\\citeyearpar");
  Let!("\\citeyearNP", "\\citeyear");
  Let!("\\citeyear", "\\citeyearpar");
  Let!("\\citeNP", "\\citealt");
  DefMacro!("\\shortcite{}", "\\citeyear{#1}");
  Let!("\\citeA", "\\citeauthor");

  DefRegister!("\\fulltextwidth" => Dimension::from_str("0pt")?);

  //======================================================================
  // Environments
  DefEnvironment!("{printonly}", "");
  DefEnvironment!("{screenonly}", "#body");
  DefEnvironment!("{anonsuppress}", "");

  //======================================================================
  // CCS descriptions
  DefMacro!("\\ccsdesc[]{}", "\\lx@add@pubnote[role=ccs,name={CCS:~}]{#2}");

  // Exclude CCSXML environment (Perl: defineExcluded(undef, 'CCSXML'))
  RawTeX!(r"\excludecomment{CCSXML}");

  //======================================================================
  // Acknowledgements
  DefMacro!("\\acknowledgmentsname", "Acknowledgements");
  // Perl L167-168 ships properties => sub { (name => Digest(T_CS('\acknowledgmentsname'))) }
  // so a user `\renewcommand{\acknowledgmentsname}{Danksagung}` localizes the
  // attribute. Rust previously hard-coded "Acknowledgements", ignoring any
  // override. Use DigestIf! pattern (same as listings_sty:2060) to resolve
  // dynamically. Inline `<ltx:acknowledgements name='#name'>` template
  // matches the Perl form.
  DefConstructor!("\\acks", "<ltx:acknowledgements name='#name'>",
    properties => {
      let name_toks = DigestIf!(T_CS!("\\acknowledgmentsname"))?;
      stored_map!("name" => name_toks)
    });
  DefConstructor!("\\endacks", "</ltx:acknowledgements>");
  DefMacro!("\\grantsponsor Semiverbatim {} Semiverbatim", "Sponsor #2 \\url{#3}");
  DefMacro!("\\grantnum OptionalSemiverbatim Semiverbatim {}", "Grant \\##3");

  //======================================================================
  // Float environments
  //
  // teaserfigure — real acmart DEFERS the teaser: `\newenvironment{teaserfigure}
  // {\Collect@Body\@saveteaser}{}` (cls L2202) stashes the body into
  // `\@teaserfigures`, and `\maketitle` renders it via `\@mkteasers` (cls L2240,
  // L2899) as the last part of the top-matter box — so in the PDF the teaser
  // appears AFTER the title+authors and BEFORE the abstract, not where the
  // environment is written (typically before `\maketitle`). Perl LaTeXML has no
  // teaserfigure binding at all, so any handling here is beyond-Perl.
  //
  // We DIGEST + CONSTRUCT the teaser in place (below) so `\label`/`\ref`,
  // `\caption` numbering, `xml:id` and `inlist` all resolve exactly as for a
  // normal float — the teaser in arXiv 2606.22880 is `\ref`-ed 6+ times, so the
  // figure node must own the label. The prior inline result merely landed in the
  // wrong PLACE (first `<document>` child, ahead of the title, because the env is
  // written before `\maketitle`). The `\lx@relocate@teaser` DOCUMENT_REWRITE rule
  // registered just below moves the finished node to just before the abstract
  // (user-directed 2026-07-13 — timing/position of `\maketitle`, no frontmatter
  // markup added over the figure).
  DefEnvironment!("{teaserfigure}[]",
    "<ltx:figure xml:id='#id' inlist='#inlist' class='ltx_teaserfigure' ?#1(placement='#1')>#tags#body</ltx:figure>",
    before_digest => { before_float("figure", None); },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );
  // Relocate the constructed teaser to acmart's top-matter position: immediately
  // before the abstract (PDF order: title, authors, teaser, abstract). Runs as a
  // post-construction DOM rewrite so the figure keeps its id/label/number — only
  // its position changes.
  //
  // We anchor on the ABSTRACT, not the teaser: the rewrite `replace` engine
  // unbinds the matched node AND every FOLLOWING sibling (re-appending them after
  // the closure), so matching the teaser — the first `<document>` child — would
  // detach the whole frontmatter and leave the teaser stuck at the front. The
  // abstract's PRECEDING siblings (the teaser, title, creators) stay bound, so we
  // move the still-bound teaser to just before the re-attached abstract. The
  // xpath predicate gates the rule to teaser-bearing documents only, so a plain
  // acmart abstract is untouched.
  DefRewrite!(
    xpath => "//ltx:abstract[//ltx:figure[contains(@class,'ltx_teaserfigure')]]",
    replace => sub[document, nodes] {
      let abs = nodes.pop().unwrap();
      // Re-attach the abstract at its original parent (the rewrite engine set the
      // current node to that parent before calling us).
      document.get_node_mut().add_child(abs)?;
      // Move the still-bound teaser to just before the abstract.
      let teasers =
        document.findnodes("//ltx:figure[contains(@class,'ltx_teaserfigure')]", None);
      if let Some(mut teaser) = teasers.into_iter().next() {
        abs.add_prev_sibling(&mut teaser)?;
      }
    });

  DefEnvironment!("{marginfigure}[]",
    "<ltx:figure xml:id='#id' inlist='#inlist' class='ltx_marginfigure' ?#1(placement='#1')>#tags#body</ltx:figure>",
    before_digest => { before_float("figure", None); },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );

  DefEnvironment!("{margintable}[]",
    "<ltx:table xml:id='#id' inlist='#inlist' class='ltx_margintable' ?#1(placement='#1')>#tags#body</ltx:table>",
    before_digest => { before_float("table", None); },
    after_digest => sub[whatsit] { after_float(whatsit); },
    mode => "internal_vertical"
  );

  //======================================================================
  // Sidebar — Perl L200-210
  DefMacro!("\\sidebarname", "Sidebar");
  DefMacro!("\\fnum@sidebar", "\\sidebarname\\nobreakspace\\thesidebar");
  DefMacro!("\\format@title@sidebar{}", "\\lx@tag{\\fnum@sidebar: }#1");

  // Perl L204-210: the {sidebar} env wraps body in <ltx:sidebar>. The
  // Perl signature is `{}{} Undigested [] {}` — title, bio, id, mark.
  // Rust previously had no sidebar env defined so ACM papers using
  // `\begin{sidebar}{title}...\end{sidebar}` hit undefined-env.
  // Simplified template (Perl has the title / creator fields
  // commented out too, so #body is the practical payload). The
  // optional labels/id attributes resolve via LaTeXML's normal
  // xml:id/labels machinery on DefEnvironment bodies.
  DefEnvironment!("{sidebar}{} Undigested [] {}",
    "<ltx:sidebar xml:id='#id'>#body</ltx:sidebar>");

  //======================================================================
  // Theorem styles via RawTeX
  RawTeX!(r"\def\@acmplainbodyfont{\itshape}");
  RawTeX!(r"\def\@acmplainindent{\parindent}");
  RawTeX!(r"\def\@acmplainheadfont{\scshape}");
  RawTeX!(r"\def\@acmplainnotefont{\@empty}");

  RawTeX!(r"\newtheoremstyle{acmplain}%
  {.5\baselineskip\@plus.2\baselineskip\@minus.2\baselineskip}%
  {.5\baselineskip\@plus.2\baselineskip\@minus.2\baselineskip}%
  {\@acmplainbodyfont}{\@acmplainindent}{\@acmplainheadfont}{.}{.5em}%
  {\thmname{#1}\thmnumber{ #2}\thmnote{ {\@acmplainnotefont(#3)}}}");

  RawTeX!(r"\def\@acmdefinitionbodyfont{\normalfont}");
  RawTeX!(r"\def\@acmdefinitionindent{\parindent}");
  RawTeX!(r"\def\@acmdefinitionheadfont{\itshape}");
  RawTeX!(r"\def\@acmdefinitionnotefont{\@empty}");

  RawTeX!(r"\newtheoremstyle{acmdefinition}%
  {.5\baselineskip\@plus.2\baselineskip\@minus.2\baselineskip}%
  {.5\baselineskip\@plus.2\baselineskip\@minus.2\baselineskip}%
  {\@acmdefinitionbodyfont}{\@acmdefinitionindent}{\@acmdefinitionheadfont}{.}{.5em}%
  {\thmname{#1}\thmnumber{ #2}\thmnote{ {\@acmdefinitionnotefont(#3)}}}");

  RawTeX!(r"\theoremstyle{acmplain}");
  RawTeX!(r"\newtheorem{theorem}{Theorem}[section]");
  RawTeX!(r"\newtheorem{conjecture}[theorem]{Conjecture}");
  RawTeX!(r"\newtheorem{proposition}[theorem]{Proposition}");
  RawTeX!(r"\newtheorem{lemma}[theorem]{Lemma}");
  RawTeX!(r"\newtheorem{corollary}[theorem]{Corollary}");
  RawTeX!(r"\theoremstyle{acmdefinition}");
  RawTeX!(r"\newtheorem{example}[theorem]{Example}");
  RawTeX!(r"\newtheorem{definition}[theorem]{Definition}");
  RawTeX!(r"\theoremstyle{acmplain}");

  Let!("\\proof", "\\@proof");
  Let!("\\endproof", "\\end@proof");

  // acmart.cls L1902: \setcctype[version]{by-spec} sets the Creative
  // Commons license. Preserve the license spec as ltx:note.
  // Witnesses 2406.04861, 2406.09266.
  DefMacro!("\\setcctype[]{}",
    "\\lx@add@frontmatter{ltx:note}[role=cc-license]{#2}");

  // acmart conditional toggles — declare as conditionals so user
  // paper's \@printpermissiontrue / \@printccstrue / \@printcopyrighttrue
  // etc. don't error. The list mirrors `\newif` declarations in
  // acmart.cls (TL2025 L181-L200); paper-local extension styles such
  // as `popets.sty` (acmart-derived) flip these without re-declaring,
  // so we must predeclare all of them. Driver: arXiv-2503.08256v1
  // (popets/acmart) where `\@acmownedfalse`, `\@acmownedtrue`, and
  // `\@ACM@journal@bibstripfalse` came up undefined.
  DefConditional!("\\if@printpermission");
  DefConditional!("\\if@printccs");
  DefConditional!("\\if@printcopyright");
  DefConditional!("\\if@printcopyrightbox");
  DefConditional!("\\if@printfolios");
  DefConditional!("\\if@acmReview");
  DefConditional!("\\if@ACM@manuscript");
  // \if@ACM@nonacm is NOT a newif in current acmart.cls, but some
  // papers (or older acmart versions) call `\@ACM@nonacmtrue` in the
  // preamble. Declare to avoid undefined errors. Witness 2211.10881.
  DefConditional!("\\if@ACM@nonacm");
  DefConditional!("\\if@ACM@journal");
  DefConditional!("\\if@ACM@journal@bibstrip");
  DefConditional!("\\if@ACM@journal@bibstrip@or@tog");
  DefConditional!("\\if@ACM@sigchiamode");
  DefConditional!("\\if@ACM@engage");
  DefConditional!("\\if@ACM@acmcp");
  DefConditional!("\\if@ACM@newfonts");
  DefConditional!("\\if@Description@present");
  DefConditional!("\\if@undescribed@images");
  DefConditional!("\\if@ACM@maketitle@typeset");
  DefConditional!("\\if@insideauthorgroup");
  DefConditional!("\\if@acmowned");
  DefConditional!("\\if@ACM@instpresent");
  DefConditional!("\\if@ACM@citypresent");
  DefConditional!("\\if@ACM@countrypresent");

  // acmart.cls L578: \def\@makefntext{\noindent\@makefnmark}.
  // Footnote helper used by acmart at L587/L600 in some path our
  // stub doesn't replicate; some templates probe it before our
  // explicit definition. Stub as a no-op so footnote processing
  // continues. Witness 2408.09084, 2408.03532 (sigconf papers).
  def_macro_noop("\\@makefntext")?;
});
