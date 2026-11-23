//! vgtc.cls (IEEE VIS and TVCG): the papers ship the class (it is not in TeX Live), which is read raw; a paper whose
//! source lacks it falls to OmniBus, with the loader's missing-file Warning. Its own `\@maketitle` prints the teaser
//! (`\teaser{…}`, a minipage set with `\def\@captype{figure}`) under the authors, and its `\maketitle` the abstract after
//! it (vgtc.cls:754-770, :801-850, :877-920; the journal style prints both in `\@maketitle`). The abstract is the
//! frontmatter's, placed with the head, so the teaser, a figure in the deposit, followed it: here the teaser is named
//! `ltx_teaserfigure` (as acmart's) and set after the authors in a continued title page, before the abstract, as the
//! PDF (OXIDIZED_DESIGN_DIVERGENCES #1149; witness 2609.25700).
use latexml_package::{
  engine::base_utilities::{deposit_in_place_floats, is_in_place_float, place_teasers_after_head},
  prelude::*,
};

/// The outermost in-place floats of `node`: the teaser's own figure, not the sub-figures it holds.
fn teaser_figures(node: &Node, out: &mut Vec<Node>) {
  if is_in_place_float(node) {
    out.push(node.clone());
  } else {
    for kid in node.get_child_elements() {
      teaser_figures(&kid, out);
    }
  }
}

LoadDefinitions!({
  if !crate::shipped_class::read_raw(&crate::shipped_class::RawRead {
    shipped: true,
    ..crate::shipped_class::RawRead::new("vgtc", "cls")
  })? {
    return Ok(());
  }
  Tag!("ltx:document", after_close => sub[document, root] {
    let teasers = deposit_in_place_floats(root);
    let mut figures = Vec::new();
    for teaser in &teasers {
      teaser_figures(teaser, &mut figures);
    }
    for mut figure in figures {
      if document::get_node_qname(&figure) == pin!("ltx:figure") {
        document.add_class(&mut figure, "ltx_teaserfigure")?;
      }
    }
    place_teasers_after_head(document, root, "ltx:title | ltx:subtitle | ltx:creator", &teasers)?;
  });
});
