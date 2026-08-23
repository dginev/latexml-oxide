//! acmart.cls — ACM article class
//! Perl: acmart.cls.ltxml (259 lines)
use crate::{
  engine::latex_constructs::{after_float, before_float},
  prelude::*,
};

/// Add `ids` to `node`'s `aria:describedby`, keeping whatever is already there.
///
/// `aria-describedby` is an id LIST, and a second `\Description` in the same
/// float (or one alongside another that already wired the same host) would
/// otherwise `set_attribute` straight over the first one's reference, leaving
/// that description hidden in the DOM and announced by nothing — silently
/// losing an annotation the author wrote. Existing ids keep their position,
/// since order is announcement order.
fn add_describedby(document: &mut Document, node: &mut Node, ids: &[String]) -> Result<()> {
  let existing = node.get_attribute("aria:describedby").unwrap_or_default();
  let mut refs: Vec<&str> = existing.split_whitespace().collect();
  for id in ids {
    if !refs.contains(&id.as_str()) {
      refs.push(id);
    }
  }
  document.set_attribute(node, "aria:describedby", &refs.join(" "))
}

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
  // under HTML5, so setting them here needs no XSLT change. WHICH element
  // they land on, and which slot each argument fills, is the
  // `before_construct` table below.
  //
  // `[short]` and `{long}` are two DISTINCT authored fields, so they get two
  // distinct elements — never concatenated into one. Merging them yields a
  // run-on ("Fly 1 and Fly 2 look identical. Fly 1 and fly 2 comparison
  // shows…") and destroys the distinction, so no consumer can tell which text
  // the author wrote as the brief alternative. `aria-describedby` takes a
  // SPACE-SEPARATED ID LIST and is announced in order, so where both are
  // referenced it is short first, each still individually addressable.
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
  // The short description needs its OWN id: it is referenced by
  // `aria-describedby` whenever its text is not the one that became the
  // image's `@alt`. It is derived in `properties` rather than written as
  // `#id-short` in the template, which does NOT work: a `#name` hole runs to
  // the end of the identifier, so `#id-short` names a property called
  // `id-short` — absent, so the element silently emits NO xml:id at all —
  // instead of `#id` followed by a literal `-short`. That produced a dangling
  // `aria-describedby`, a reference resolving to nothing. Ids use a `-short`
  // suffix rather than a dotted one so they need no escaping in a CSS
  // selector.
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
    // `\Description` is a TEXT ALTERNATIVE — and the thing it is an alternative
    // TO is the image, not the float. It therefore lands on the IMAGE:
    //
    //   `\Description[s]{l}`           → img @alt = s, aria-describedby → l
    //   `\Description{l}`, l plain     → img @alt = l (it replaces the image)
    //   `\Description{l}`, l w/ markup → img @alt untouched, describedby → l
    //
    // `@alt` (via `ltx:graphics/@description`, `LaTeXML-misc-xhtml.xsl` L167-171)
    // rather than `aria-label`, because that is the attribute an `<img>` has for
    // exactly this purpose and the one assistive technology expects there.
    //
    // NOT `aria:label` on the `<ltx:figure>`, which is what this binding did
    // before: `aria-label` sets the accessible NAME, and a float's name is its
    // caption, so labelling the figure with the description DISPLACED
    // "Figure 1. caption text" and hid the caption from a screen reader
    // (reviewer report, brucemiller/LaTeXML#430 r3674103638; the fix that
    // review asks for is precisely "label + description should be attached to
    // the first image in the figure … and the `<img>` tag eventually gets
    // `@alt` and not `@aria-label`").
    //
    // The markup case is why the argument is read `Undigested`: `@alt` is a
    // plain string and cannot carry markup, so we must inspect the tokens to
    // choose the slot BEFORE expanding anything. A control sequence (or an
    // active/`$`/`^`/`_` token) means real markup, so the block carries it.
    //
    // The block is always emitted, so the text stays addressable, but it is
    // referenced only when it is not already the `@alt` — otherwise the same
    // sentence would be announced twice. An unreferenced hidden block is inert:
    // `display:none` content is announced only when something references it.
    //
    // TWO CASES HAVE NO IMAGE TO USE. An author's annotation is never dropped,
    // so it goes to the next best host — the enclosing float — as
    // `aria:describedby`, which supplements the name instead of replacing it,
    // so the caption survives either way. Both `Warn!`, because the result is
    // second-best and the author can do something about it:
    //
    //  * No `ltx:graphics` in the float at all — a figure built from tabular,
    //    text or TikZ content (`t/complex/acm_aria` is one), a `table` float,
    //    or an empty one. There is no image to be the alternative to.
    //  * MORE than one. A `\Description` is scoped to the whole float, so on a
    //    multi-panel figure it describes the ensemble; making it panel 1's
    //    `@alt` would assert that one sentence is the alternative for one
    //    panel, which is a claim the author never made. The review says "the
    //    first image"; we narrow that to the case where "first" is also "only",
    //    where it is unambiguous.
    //
    // We can only see the graphics ALREADY BUILT when `\Description` is
    // constructed, so a `\Description` written BEFORE its `\includegraphics`
    // falls into the first case. That is the safe direction to fail — the
    // description is still announced, just not as the image's alternative — the
    // warning names it as a possible cause, and acmart's own documentation puts
    // `\Description` after the graphic.
    //
    // acmart's NEWER mechanism for the same purpose is `\includegraphics[alt=…]`
    // (switched on by `\DocumentMetadata`), handled in `graphicx_sty.rs` and
    // landing on the same `description` attribute. When an author uses BOTH —
    // e.g. arXiv:2607.21760, which repeats the same paragraph in each — the
    // explicit `alt=` WINS: it names one image, while `\Description` names the
    // float, so the more specific statement stands. `\Description` then only
    // adds its `aria-describedby` references.
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
      // `\Description[]{…}` gives an empty optional argument, which the
      // template's `?#1(…)` declines to emit — so there is no short note and
      // no id to reference. Treat it as absent.
      let short = whatsit
        .get_arg(1)
        .map(|d| d.to_attribute())
        .filter(|s| !s.is_empty());
      let short_id = whatsit.get_property("shortid").map(|v| v.to_string());
      let Some(mut figure) = document.get_element() else {
        return Ok(());
      };
      let mut graphics = document.findnodes(".//ltx:graphics", Some(&figure));
      // Exactly one image: it IS the figure, so the description is its
      // alternative. Zero or several: see the note above, keep it on the float.
      let lone_graphic = (graphics.len() == 1).then(|| graphics.remove(0));
      let mut described_by: Vec<String> = Vec::new();
      match lone_graphic {
        Some(mut graphic) => {
          // `\includegraphics[alt=…]` already spoke for this image; it is the
          // more specific statement, so it stands.
          let alt = if graphic.get_attribute("description").is_some() {
            None
          } else if short.is_some() {
            short.clone()
          } else if long_is_plain {
            whatsit.get_arg(2).map(|d| d.to_attribute())
          } else {
            None
          };
          match alt {
            // Something of ours became the alternative. Whatever did NOT is
            // still worth announcing, so reference it.
            Some(text) => {
              document.set_attribute(&mut graphic, "description", &text)?;
              if short.is_some() {
                described_by.push(id);
              }
            },
            // Nothing of ours is the alternative — the author's own `alt=`
            // holds it, or the lone description carries markup. Reference
            // everything we emitted.
            None => {
              if short.is_some() && let Some(sid) = short_id {
                described_by.push(sid);
              }
              described_by.push(id);
            },
          }
          if !described_by.is_empty() {
            add_describedby(document, &mut graphic, &described_by)?;
          }
        },
        // Nowhere better to put it. Attach to the enclosing element rather than
        // drop the author's annotation, and say so — the text is still
        // announced, but not as the alternative for any one image, which is
        // what a `\Description` is for.
        None => {
          if short.is_some() && let Some(sid) = short_id {
            described_by.push(sid);
          }
          described_by.push(id);
          add_describedby(document, &mut figure, &described_by)?;
          let host = figure.get_name();
          if graphics.is_empty() && host == "table" {
            // NOT a fallback: a table float has no image BY CONSTRUCTION, and
            // acmart asks for a `\Description` on every float — so the table
            // itself is where the description belongs, and this is the author
            // doing the right thing. Warning here demoted otherwise-clean
            // papers: 27 of 45 sampled `no_problem -> warning` regressions in
            // the 2026-07-30 sandbox-arxiv-2605 rerun were this one message.
            // Report it (the attachment point is worth knowing), as INFO.
            Info!("aria", "\\Description",
              &s!("attached to the enclosing <table> as aria:describedby — a \
                   table has no image to describe, so this is where the \
                   description belongs"));
          } else {
            let why = if !graphics.is_empty() {
              s!("it holds more than one image, so which one is described is ambiguous")
            } else if host == "figure" {
              s!("the float has no image to describe (a \\Description written \
                  before its \\includegraphics, or a figure with no graphic)")
            } else {
              // Not a float at all — `\Description` outside a figure/table,
              // where acmart's own `\@Description@present` bookkeeping expects it.
              s!("it is outside any figure or table, so there is no image to describe")
            };
            Warn!("unexpected", "\\Description",
              &s!("attached to the enclosing <{host}> as aria:describedby, because \
                   {why}. The text is still announced, but it is not any image's \
                   alt text; put a \\Description after the \\includegraphics it \
                   describes, or use \\includegraphics[alt=...] per image"));
          }
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
  DefMacro!("\\orcid{}",                 "\\lx@add@orcid{#1}");

  //======================================================================
  // Internal structure to affiliation (Perl PR #2767: comma-joined parts;
  // empty parts skipped)
  DefMacro!("\\lx@acm@addresspartsep", "");
  // `\ignorespaces` (after `\fi`) gobbles the source newline the author writes
  // between `\institution{}`, `\city{}`, `\state{}`, `\country{}` lines — else it
  // leaks as a space BEFORE the separator comma, so a wrap puts the comma at the
  // start of the next line. The real acmart.cls does the same with `\unskip`/
  // `\ignorespaces` (acmart.cls L1679/L2879). Separator is `, ` (comma + a REGULAR
  // breakable space, matching real acmart's `, `), not `,~`, so the line breaks
  // AFTER the comma, never before it. SHARED surpass over Perl (its
  // acmart.cls.ltxml:98-101 has neither) — OXIDIZED_DESIGN #158. Witness 2605.03143.
  DefMacro!("\\lx@acm@addresspart{}{}",
    "\\ifx.#2.\\else\\lx@acm@addresspartsep\\def\\lx@acm@addresspartsep{, }\\lx@acm@addresspart@{#1}{#2}\\fi\\ignorespaces");
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
