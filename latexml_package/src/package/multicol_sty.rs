use latexml_core::document::{can_auto_close, can_contain_node_somehow};

use crate::prelude::*;

/// Whether an `ltx:pagination` can be inserted at the document's current point: the node holds one (directly or
/// through an element it opens), or auto-closing nodes up to one that does (`Document::find_insertion_point`'s
/// admissible cases, without its insertion).
pub fn pagination_insertable(document: &Document) -> bool {
  let mut node = document.get_node().clone();
  loop {
    if node.get_type() == Some(NodeType::ElementNode)
      && can_contain_node_somehow(&node, "ltx:pagination").is_some()
    {
      return true;
    }
    if !can_auto_close(&node) {
      return false;
    }
    match node.get_parent() {
      Some(parent) => node = parent,
      None => return false,
    }
  }
}

/// The `{multicols}` constructor, multicol.sty.ltxml:21-30's template
/// `?#2(<ltx:para><ltx:p>#2<ltx:para>)<ltx:pagination role='start_#1_columns'/>#body<ltx:pagination role='end_#1_columns'/>`
/// with each marker made only where a pagination can go (`pagination_insertable`).
pub fn construct_multicols(
  document: &mut Document,
  args: &[Option<Digested>],
  props: &SymHashMap<Stored>,
) -> Result<()> {
  let columns = args
    .first()
    .and_then(Option::as_ref)
    .map(ToString::to_string)
    .unwrap_or_default();
  let marker = |document: &mut Document, edge: &str| -> Result<()> {
    if pagination_insertable(document) {
      document.insert_element(
        "ltx:pagination",
        Vec::new(),
        Some(string_map!("role" => s!("{edge}_{columns}_columns"))),
      )?;
    }
    Ok(())
  };
  if let Some(heading) = args.get(1).and_then(Option::as_ref)
    && !matches!(heading.to_string().as_str(), "" | "false")
  {
    document.open_element("ltx:para", None, None)?;
    document.open_element("ltx:p", None, None)?;
    document.absorb(heading, None)?;
    document.open_element("ltx:para", None, None)?;
  }
  marker(document, "start")?;
  if let Some(Stored::Digested(body)) = props.get("body") {
    document.absorb(body, None)?;
  }
  marker(document, "end")
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: multicol.sty.ltxml
  // Since this is basically styling, we can ignore the effects.

  // Optional arg is sortof a heading, but w/o any particular styling(?)
  // Note: Perl uses <ltx:para> (open, not close) at end of conditional
  // to auto-close the heading para and open a new one that wraps pagination + body
  // Body is digested in `internal_vertical` so `$$ … $$` display-math
  // gate (`tex_math.rs:443` / Perl `TeX_Math.pool.ltxml:65`) recognizes
  // the second `$`. Without this, multicols's body inherits/defaults to
  // restricted_horizontal and `$$ x_1 $$` errors with `Script _ can only
  // appear in math mode`. Faithful Perl-template wrapping is preserved
  // (Perl multicol.sty.ltxml:21-25) — only the digest mode is made
  // explicit. Cluster: cond-mat0001099 + hep-ph0001306 + math0601451's
  // many script-mode errors all trace here.
  // No explicit `</ltx:p>` after the spanning text: a block-level `#2`
  // (`\begin{multicols}{2}[\section*{…}]`, thuaslogos-doc) auto-closes the
  // `ltx:p` itself and the explicit close then errored "Attempt to close
  // </ltx:p>, which isn't open" (Perl multicol.sty.ltxml:22 identical, KPE
  // #150); for inline `#2` the following `<ltx:para>` open closes it.
  // multicol.sty:145 reads the column count as a NUMBER (`\col@number#1
  // \relax`), and :172-186 take a SECOND optional argument, the free space
  // needed to start — a dimension (`\mult@@cols#1[#2]` → `\enough@room{#2}`).
  // doc.sty:572/656 writes both:
  // `\begin{multicols}\c@IndexColumns[\index@prologue][\IndexMin]`. Read as
  // `{}[]`, the register digested as an assignment (`Missing number`,
  // `role="start__columns"`) and `[\IndexMin]` stayed in the body as `[`, a
  // dimen assignment, `]` (Perl multicol.sty.ltxml:21 shares both).
  // The column markers are page layout: they are made only where a pagination can go (`pagination_insertable`, at
  // the construction point, as `Document::find_insertion_point` would place one). A `multicols` inside `algorithmic`
  // (an `ltx:listing`, which holds none and does not auto-close) put its markers in the listing, "<ltx:pagination>
  // isn't allowed in <ltx:listing>" (1708.03274, 1405.5689; 38 papers of run 336, every one a `multicols` in an
  // `algorithmic`; Perl multicol.sty.ltxml:21-30 shares it, KNOWN_PERL_ERRORS #1094). The `\columnbreak` or
  // `\newpage` such a listing holds breaks no column here (`\newpage`'s marker floats, tex_page.rs). Repro
  // listings-algorithms/multicols_in_algorithmic_keeps_no_markers.
  DefEnvironment!("{multicols}{Number}[][Dimension]",
    sub[document, args, props] { construct_multicols(document, args, props) },
    mode => "internal_vertical",
    // The begin code opens a `\begingroup` (a semi_simple_group) of its own: multicol.sty:145-199 `\mult@cols` `\begingroup`.
    tex_group => { TexGroup::SemiSimple });
  DefEnvironment!("{multicols*}{Number}[][Dimension]",
    sub[document, args, props] { construct_multicols(document, args, props) },
    mode => "internal_vertical",
    // The begin code opens a `\begingroup` (a semi_simple_group) of its own: multicol.sty:145-199 `\mult@cols` `\begingroup`.
    tex_group => { TexGroup::SemiSimple });

  def_macro_noop("\\botmark")?;
  def_macro_noop("\\topmark")?;

  // multicol.sty:936-950 `\newcolumn`: a column break (`\vfill\penalty-\@Mv`
  // in vertical mode, `\vadjust` in horizontal mode) — like `\columnbreak`
  // (`sect12.rs`) it has no meaning in a flow layout; it was undefined here
  // (tikz-ext-manual-en-intro.tex:37). Batch 56bg.
  def_macro_noop("\\newcolumn")?;
  def_macro_noop("\\flushcolumns")?;
  def_macro_noop("\\raggedcolumns")?;
  def_macro_noop("\\setemergencystretch")?;

  DefRegister!("\\premulticols"         => Dimension!("50pt"));
  DefRegister!("\\postmulticols"        => Dimension!("20pt"));
  DefRegister!("\\multicolsep"          => Glue!("12pt plus 4pt minus 3pt"));
  DefRegister!("\\multicolbaselineskip" => Glue::new(0));
  DefRegister!("\\multicolovershoot"    => Dimension!("2pt"));
  DefRegister!("\\multicolundershoot"   => Dimension!("2pt"));
  DefRegister!("\\multicolpretolerance" => Number::new(-1));
  DefRegister!("\\multicoltolerance"    => Number::new(9999));
  DefRegister!("\\doublecol@number"     => Number::new(0));
});
