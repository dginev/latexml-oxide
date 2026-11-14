//! The accessible description of a float's image (acmart's `\Description[short]{long}`, oup-authoring-template's
//! `\figalttext[short]{long}`): a hidden note in the float's caption, which the lone image takes as its `@alt` or
//! `aria-describedby`. Shared by the class bindings whose classes carry such a command (`acmart_cls`,
//! `oup_authoring_template_cls`).
use crate::{engine::latex_constructs::out_of_line_size, prelude::*};

/// A description's tokens with the characters text rejects outside math (`_ ^ & #`) as the text commands for them;
/// math is `$…$` or `\(…\)`.
fn description_text(tokens: &Tokens) -> Vec<Token> {
  let mut in_math = false;
  tokens
    .unlist_ref()
    .iter()
    .map(|t| match t.get_catcode() {
      Catcode::MATH => {
        in_math = !in_math;
        *t
      },
      Catcode::CS if *t == T_CS!("\\(") || *t == T_CS!("\\)") => {
        in_math = *t == T_CS!("\\(");
        *t
      },
      Catcode::SUB if !in_math => T_CS!("\\textunderscore"),
      Catcode::SUPER if !in_math => T_CS!("\\textasciicircum"),
      Catcode::ALIGN if !in_math => T_CS!("\\&"),
      Catcode::PARAM if !in_math => T_CS!("\\#"),
      _ => *t,
    })
    .collect()
}

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

/// The tokens of a description command's call, `\lx@acm@description[short]{long}`: the text of each argument with the
/// characters text rejects outside math (`_ ^ & #`) as the text commands for them.
pub fn float_description_tokens(short: Option<&Tokens>, long: &Tokens) -> Tokens {
  let mut out = vec![T_CS!("\\lx@acm@description")];
  if let Some(short) = short {
    out.push(T_OTHER!("["));
    out.push(T_BEGIN!());
    out.extend(description_text(short));
    out.push(T_END!());
    out.push(T_OTHER!("]"));
  }
  out.push(T_BEGIN!());
  out.extend(description_text(long));
  out.push(T_END!());
  Tokens::new(out)
}

/// Define the description's machinery for the class's `command` (which its diagnostics name): the WAI-ARIA namespace,
/// the `acmlabel` counter of its ids and the `\lx@acm@description` constructor that places it.
#[rustfmt::skip]
pub fn define_float_description(command: &'static str) -> Result<()> {
  // The command the diagnostics name: the class's own (`\Description`, `\figalttext`).
  assign_value("float_description_command", Stored::from(command), Some(Scope::Global));
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
  // acmart.cls:895 `\Description` typesets nothing, so authors leave unescaped what LaTeX would reject in text
  // (`\Description{Complaint_Loss}`, 2502.07049): outside math, `_ ^ & #` are the characters they show, and the
  // description is digested as written otherwise.
  DefConstructor!("\\lx@acm@description[]{}",
    "^^?#1(<ltx:note xml:id='#shortid' class='ltx_nodisplay ltx_acm_description_short'>#1</ltx:note>)()\
     <ltx:note xml:id='#id' class='ltx_nodisplay ltx_acm_description'>#2</ltx:note>",
    sizer => sub[_whatsit] { Ok(out_of_line_size()) },
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
    // `@alt` is a plain string and cannot carry markup, so the slot is chosen by
    // the description's tokens as written (its reversion): a control sequence
    // (or an active/`$`/`^`/`_` token) means real markup, so the block carries
    // it. The argument itself is digested, as Perl's `{}` (acmart.cls.ltxml:78)
    // and LaTeX do: read `Undigested`, the block printed its markup as raw TeX
    // (`\sysname{}`, 2403.09168; 2401.04997, 2304.01062, 2312.11013).
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
        .map(|d| d.revert())
        .transpose()?
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
          // The command the author wrote (`define_float_description`'s).
          let command = lookup_string("float_description_command");
          if graphics.is_empty() && host == "table" {
            // NOT a fallback: a table float has no image BY CONSTRUCTION, and
            // acmart asks for a `\Description` on every float — so the table
            // itself is where the description belongs, and this is the author
            // doing the right thing. Warning here demoted otherwise-clean
            // papers: 27 of 45 sampled `no_problem -> warning` regressions in
            // the 2026-07-30 sandbox-arxiv-2605 rerun were this one message.
            // Report it (the attachment point is worth knowing), as INFO.
            Info!("aria", &command,
              &s!("attached to the enclosing <table> as aria:describedby — a \
                   table has no image to describe, so this is where the \
                   description belongs"));
          } else {
            let why = if !graphics.is_empty() {
              s!("it holds more than one image, so which one is described is ambiguous")
            } else if host == "figure" {
              s!("the float has no image to describe (a {command} written \
                  before its \\includegraphics, or a figure with no graphic)")
            } else {
              // Not a float at all — `\Description` outside a figure/table,
              // where acmart's own `\@Description@present` bookkeeping expects it.
              s!("it is outside any figure or table, so there is no image to describe")
            };
            Warn!("unexpected", &command,
              &s!("attached to the enclosing <{host}> as aria:describedby, because \
                   {why}. The text is still announced, but it is not any image's \
                   alt text; put a {command} after the \\includegraphics it \
                   describes, or use \\includegraphics[alt=...] per image"));
          }
        },
      }
    },
    // A `\label` after the `\Description` names the float, as in LaTeX, where
    // `\Description` typesets nothing (acmart.cls:895). `\label` takes the
    // nearest element with an id from the current node's last child
    // (`float_to_label`, Perl Document.pm floatToLabel), and that was the note,
    // which moves into the caption only when the float closes (`arrange_panels`):
    // 2403.09168's ten labels sat on hidden notes, so each `\ref` pointed at one
    // (Perl 0.8.8 the same, KNOWN_PERL_ERRORS #535). So the caption already in the
    // float before it — the last, the description's own — takes the notes now,
    // where the float's close would put them.
    after_construct => sub[document, whatsit] {
      let ids: Vec<String> = ["id", "shortid"]
        .iter()
        .filter_map(|key| whatsit.get_property(key).map(|v| v.to_string()))
        .collect();
      let Some(float) = document.get_element() else {
        return Ok(());
      };
      let children = float.get_child_elements();
      let Some(mut caption) = children
        .iter()
        .rev()
        .find(|c| c.get_name() == "caption").cloned() else {
        return Ok(());
      };
      for mut note in children.into_iter().filter(|c| {
        c.get_name() == "note" && c.get_attribute_ns("id", XML_NS).is_some_and(|id| ids.contains(&id))
      }) {
        note.unlink_node();
        caption.add_child(&mut note)?;
      }
    }
  );
  Ok(())
}
