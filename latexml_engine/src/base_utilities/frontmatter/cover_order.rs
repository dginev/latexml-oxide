//! An abstract placed after the title-page text printed before it: the cover between the placed frontmatter and the
//! abstract's source position is wrapped as `<ltx:titlepage>`, and the abstract follows it (OXIDIZED_DESIGN_DIVERGENCES
//! #903, extending #246).

use super::*;

/// A root child the class's own title page printed (the `\@maketitle` deposit, the class `\maketitle` replay:
/// `\lx@deposit@maketitle`, sect05.rs): cover by provenance, whatever its typography. Internal, stripped from the
/// output.
const DEPOSIT_PROVENANCE: &str = "_deposit";

/// Where the class's title page began to be built (`\lx@deposit@begin`), until its end (`\lx@deposit@end`).
const DEPOSIT_BEGIN: &str = "ltx:_TitlePageDeposit_Capture_";

/// The class of a `<titlepage>` that continues the title block rather than standing for it: the cover lines printed
/// after the title, before the abstract. The HTML displays the structured title beside it
/// (LaTeXML-structure-xhtml.xsl, the `ltx:title` template), and a class's title print on a cover is no cover title
/// beside it (`settle_cover_title_ink`).
pub const CONTINUED_TITLEPAGE: &str = "ltx_titlepage_continued";

/// A frontmatter element set after `\maketitle` placed the frontmatter (`frontmatter_push`): queued after
/// `\lx@frontmatterhere`'s flush or digested in place after it. A late abstract's cover is followed by the first late
/// abstract and its run, save what was printed before the cover (`printed_before_the_cover`: smfart's `{abstract}`
/// before `\maketitle` and `{altabstract}` after it). Internal, stripped from the output.
pub const LATE_ENTRY: &str = "_late";

/// A frontmatter entry handed on from a raw class's store, named by it (`\lx@store@from`, `harvest_stores`,
/// `\lx@store@hand`, `hand_untypeset_stores`; `queue_front_matter`, `frontmatter_push`): its title-page provenance.
/// The class's own title page may print it anywhere — above the cover, below it, at the page foot (a `\footnotetext`,
/// an `\insert`) — which the frontmatter cannot tell, so a late run holding one the title page may print is not moved
/// at all (the fail-safe of `place_abstract_after_cover`). Internal, stripped from the output.
pub const TITLE_PAGE_ENTRY: &str = "_titlepage";

/// Does this frontmatter element carry title-page provenance ([`TITLE_PAGE_ENTRY`]) from a store the class's title page
/// may print (frontmatter_stores.rs `title_page_may_print`: not one its speculative run showed it does not)?
fn from_the_title_page(node: &Node) -> bool {
  node
    .get_attribute(TITLE_PAGE_ENTRY)
    .is_some_and(|name| crate::frontmatter_stores::title_page_may_print(&name))
}

/// The source position of an abstract queued after `\maketitle` placed the frontmatter (`\lx@frontmatter@mark`); its own
/// name, so the flush's marker sweep (`remove_frontmatter_marks`) leaves it to the placement that reads it.
pub const LATE_ABSTRACT_MARK: &str = "ltx:_LateAbstract_Capture_";

/// `\lx@deposit@begin`: mark the current point as where the class's title page begins.
pub fn begin_deposit_provenance(document: &mut Document) -> Result<()> {
  if document.get_element().is_some() {
    document.insert_element(DEPOSIT_BEGIN, Vec::new(), None)?;
  }
  Ok(())
}

/// `\lx@deposit@end`: every root child built since `\lx@deposit@begin` is the class's title page (`_deposit`).
/// A page begun inside an open paragraph or a box marks only what follows that root child.
pub fn end_deposit_provenance(document: &mut Document) -> Result<()> {
  mark_root_children_after(document, DEPOSIT_BEGIN, DEPOSIT_PROVENANCE);
  Ok(())
}

/// A root child an `\insert` inside the class's title page set (`\lx@deposit@insert@begin` … `@end`, tex_inserts.rs):
/// page-foot material, out of the title page's flow — AAAI's copyright slug (aaai21.sty:53), printed at the foot of the
/// first column, after the abstract in reading order. Internal, stripped from the output.
const DEPOSIT_INSERT: &str = "_inserted";

/// Where an `\insert`'s material began inside the class's title page.
const DEPOSIT_INSERT_BEGIN: &str = "ltx:_DepositInsert_Capture_";

/// `\lx@deposit@insert@begin`: mark the current point as where an `\insert`'s material begins.
pub fn begin_deposit_insert(document: &mut Document) -> Result<()> {
  if document.get_element().is_some() {
    document.insert_element(DEPOSIT_INSERT_BEGIN, Vec::new(), None)?;
  }
  Ok(())
}

/// `\lx@deposit@insert@end`: every root child built since `\lx@deposit@insert@begin` is page-foot material
/// (`_inserted`).
pub fn end_deposit_insert(document: &mut Document) -> Result<()> {
  mark_root_children_after(document, DEPOSIT_INSERT_BEGIN, DEPOSIT_INSERT);
  Ok(())
}

/// Mark (`attribute`) every root child after the one holding the last `marker` element, frontmatter and markers aside;
/// remove the marker.
fn mark_root_children_after(document: &mut Document, marker: &str, attribute: &str) {
  for marker in document.findnodes(&s!("//{marker}"), None) {
    if let Some(top) = root_child_of(&marker) {
      let mut next = top.get_next_element_sibling();
      while let Some(mut node) = next {
        next = node.get_next_element_sibling();
        if !node.get_name().ends_with("_Capture_") && !is_frontmatter_group_element(&node) {
          let _ = node.set_attribute(attribute, "true");
        }
      }
    }
    document.remove_node(marker);
  }
}

/// The bound of [`place_abstract_after_cover`] for an abstract a flush is about to place, read from its position marker
/// (`\lx@frontmatter@mark`, a root child) before the flush removes it: the root child after the marker, or `None` for
/// the end. `None` when there is no such marker.
pub(in crate::base_utilities) fn abstract_mark_bound(
  document: &mut Document,
) -> Option<Option<Node>> {
  let marker = document.findnode("/ltx:document/ltx:_Frontmatter_Capture_", None)?;
  // (not another marker, which the flush removes too: an `{altabstract}` right after the abstract)
  let mut next = marker.get_next_element_sibling();
  while let Some(node) = next
    .as_ref()
    .filter(|n| n.get_name() == "_Frontmatter_Capture_")
  {
    next = node.get_next_element_sibling();
  }
  Some(next)
}

/// The child of the document root that holds `node` (`node` itself when it is one).
fn root_child_of(node: &Node) -> Option<Node> {
  let mut cur = node.clone();
  loop {
    let parent = cur.get_parent()?;
    if parent
      .get_parent()
      .is_none_or(|p| p.get_type() == Some(NodeType::DocumentNode))
    {
      return Some(cur);
    }
    cur = parent;
  }
}

/// A late abstract's position marker ([`LATE_ABSTRACT_MARK`]), where the abstract is already placed (the eager build,
/// whose `\maketitle` flush digested the whole queue): the abstract goes behind the cover printed before it
/// (`place_abstract_after_cover`), and the marker is removed. Before the abstract is placed (a streamed build, where
/// only an abstract `\maketitle` placed may be there), the marker stays for `insert_late_frontmatter`.
pub fn settle_late_abstract_mark(document: &mut Document, marker: Node) -> Result<()> {
  let placed = document
    .findnode(&s!("/ltx:document/ltx:abstract[@{LATE_ENTRY}]"), None)
    .is_some();
  if placed {
    place_abstract_after_late_mark(document, &marker)?;
    document.remove_node(marker);
  }
  Ok(())
}

/// [`place_abstract_after_cover`] up to `marker`, a late abstract's position: the region ends right before it when it
/// is a root child; one inside an open paragraph (text runs into `\begin{abstract}`) ends the region with that
/// paragraph, when nothing follows the marker there.
pub(in crate::base_utilities) fn place_abstract_after_late_mark(
  document: &mut Document,
  marker: &Node,
) -> Result<bool> {
  let Some(top) = root_child_of(marker) else {
    return Ok(false);
  };
  if top == *marker {
    return place_abstract_after_cover(document, Some(&top), true);
  }
  // (the marker ends its root child: nothing after it at any level)
  let mut cur = marker.clone();
  while cur != top {
    let mut next = cur.get_next_sibling();
    while let Some(n) = next {
      let blank = n.get_type() == Some(NodeType::TextNode) && n.get_content().trim().is_empty();
      if !blank {
        return Ok(false);
      }
      next = n.get_next_sibling();
    }
    let Some(parent) = cur.get_parent() else {
      return Ok(false);
    };
    cur = parent;
  }
  match top.get_next_element_sibling() {
    Some(next) => place_abstract_after_cover(document, Some(&next), true),
    None => place_abstract_after_cover(document, None, true),
  }
}

/// The readability bar's print order for an abstract the frontmatter placed AHEAD of title-page text pdflatex prints
/// before it: a `late` abstract (queued after `\maketitle`'s flush, which placed it with the head;
/// `\lx@frontmatter@mark`, `insert_late_frontmatter`) — the first root `ltx:abstract` that is [`LATE_ENTRY`], `bound`
/// its own position marker — or one a document without `\maketitle` sets after its in-place setters (the fallback,
/// `\lx@frontmatter@fallback`) — the first root `ltx:abstract`, `bound` the first abstract's marker, which precedes
/// every abstract in print. The root children between the placed run of frontmatter that begins with that abstract and
/// `bound` (a root child, exclusive; `None`: to the end) are read as OXIDIZED_DESIGN #246 reads a cover:
/// - title-page layout ([`is_titlepage_layout`]: a paragraph, a display block) that the class's title page printed
///   ([`DEPOSIT_PROVENANCE`]) or that is no running body prose ([`is_body_prose`]) is the cover, wrapped as
///   `<ltx:titlepage>` in its order;
/// - a content-free node (a pagebreak), preamble residue, a root note (a footnote's text, out of the flow) and what an
///   `\insert` in the class's title page set ([`DEPOSIT_INSERT`]: page-foot material) are passed over and follow the
///   abstract;
/// - anything else — a sectional unit, a TOC, a float, body prose — keeps today's placement: the region is not the
///   cover.
///
/// With a cover, the abstract and the frontmatter after it in its run (keywords, classification, a title `\thanks`
/// pubnote) move behind the `<titlepage>`, save an abstract queued before `\maketitle`'s flush
/// ([`printed_before_the_cover`]), which keeps its place ahead of it. The fail-safe: a late run holding an entry handed
/// on from the raw class's store that its title page may print ([`from_the_title_page`]), where the frontmatter cannot
/// tell, is not moved at all — the abstract and everything after it keep today's slot. Once per document
/// (`lx_late_abstract_placed`): a second late abstract has nothing more to place. Returns whether it moved anything.
pub(in crate::base_utilities) fn place_abstract_after_cover(
  document: &mut Document,
  bound: Option<&Node>,
  late: bool,
) -> Result<bool> {
  if lookup_bool("lx_late_abstract_placed") {
    return Ok(false);
  }
  let Some(root) = document.get_document().get_root_element() else {
    return Ok(false);
  };
  let kids: Vec<Node> = root
    .get_child_nodes()
    .into_iter()
    .filter(|n| n.get_type() == Some(NodeType::ElementNode))
    .collect();
  let end = match bound {
    Some(bound) => match kids.iter().position(|k| k == bound) {
      Some(end) => end,
      None => return Ok(false),
    },
    None => kids.len(),
  };
  let Some(first) = kids[..end].iter().position(|k| {
    document::get_node_qname(k) == pin_static("ltx:abstract")
      && (!late || k.has_attribute(LATE_ENTRY))
  }) else {
    return Ok(false);
  };
  // The abstract's run: it and the frontmatter after it, as placed (never a titlepage).
  let mut run_end = first;
  while run_end < end
    && is_frontmatter_group_element(&kids[run_end])
    && document::get_node_qname(&kids[run_end]) != pin_static("ltx:titlepage")
  {
    run_end += 1;
  }
  // (the fail-safe: what the class's title page printed of its stores may stand above the cover, below it or at the
  // page foot — `\footnotetext{Keywords: \@keywords}` — so a late run holding one stays where it is)
  if late && kids[first..run_end].iter().any(from_the_title_page) {
    return Ok(false);
  }
  // (a title page still being built — the abstract printed inside the class's own — is the class's from its start)
  let open_cover = kids[..end]
    .iter()
    .rposition(|k| document::get_node_qname(k) == pin_static(DEPOSIT_BEGIN));
  let mut cover: Vec<Node> = Vec::new();
  let mut passed: Vec<Node> = Vec::new();
  for (at, kid) in kids.iter().enumerate().take(end).skip(run_end) {
    if kid.get_name().ends_with("_Capture_") {
      continue;
    }
    let q = document::get_node_qname(kid);
    if node_is_content_free(kid)
      || is_preamble_residue(kid)
      || q == pin_static("ltx:note")
      || kid.has_attribute(DEPOSIT_INSERT)
    {
      passed.push(kid.clone());
    } else if is_titlepage_layout(kid)
      && (kid.has_attribute(DEPOSIT_PROVENANCE)
        || open_cover.is_some_and(|begin| begin < at)
        || !is_body_prose(kid))
    {
      cover.push(kid.clone());
    } else {
      return Ok(false);
    }
  }
  if cover.is_empty() {
    return Ok(false);
  }
  // The build goes on at the root, not inside a paragraph that becomes a block of the titlepage.
  let inside = |document: &Document| {
    let mut cur = Some(document.get_node().clone());
    while let Some(n) = cur {
      if cover.contains(&n) {
        return true;
      }
      cur = n.get_parent();
    }
    false
  };
  if inside(document) {
    close_auto_closable_to_root(document, &root)?;
    if inside(document) {
      return Ok(false);
    }
  }
  assign_value("lx_late_abstract_placed", true, Some(Scope::Global));
  let mut titlepage = wrap_as_titlepage(document, &root, &cover)?;
  // (it follows the title block, which printed the title: the structured title stays displayed beside it)
  document.add_class(&mut titlepage, CONTINUED_TITLEPAGE)?;
  let mut tail = titlepage;
  // (what the title page printed before the cover keeps its place ahead of it: of a late run, an abstract `\maketitle`
  // placed — set before it)
  for mut node in kids[first..run_end]
    .iter()
    .filter(|k| !printed_before_the_cover(k, late))
    .cloned()
  {
    node.unlink();
    tail.add_next_sibling(&mut node)?;
    tail = node;
  }
  move_content_free_after(&passed, tail)?;
  Ok(true)
}

/// Of the run that follows a late abstract behind its cover, an entry printed before the cover, which keeps its place
/// ahead of it: an abstract queued before `\maketitle`'s flush (not [`LATE_ENTRY`]: smfart's `{abstract}` before
/// `\maketitle`). Everything else in the run — keywords and notes a binding prints after the abstract (dmtcs,
/// 0905.1102; aaspp4, astro-ph9911463), a title `\thanks` pubnote (2102.01904) — follows the abstract. (A run holding
/// an entry from a raw class's store its title page may print is not moved at all: [`from_the_title_page`].)
fn printed_before_the_cover(node: &Node, late: bool) -> bool {
  late
    && !node.has_attribute(LATE_ENTRY)
    && document::get_node_qname(node) == pin_static("ltx:abstract")
}

/// A paragraph of running body prose: an indented paragraph (no `\noindent`, no alignment of its own) holding text
/// outside any box, or a list whose items hold one (a body list). A cover line is set apart — `\noindent`ed, centred or
/// flushed, a box alone, a list of such lines (nature's `{affiliations}`, 2304.00893; ae100prg's `\address`, 1303.3004)
/// — so the text set between `\maketitle` and a late abstract is cover only when none of it is prose (`First body
/// paragraph.` before a `\DocInput`-ed abstract stays body,
/// `perfect_kernel_batch56::late_abstract_after_maketitle_leaves_no_marker_and_wraps_nothing`).
fn is_body_prose(node: &Node) -> bool {
  if document::get_node_qname(node) != pin_static("ltx:para") || is_set_apart(node) {
    return false;
  }
  element_nodes(node)
    .iter()
    .any(|child| match child.get_name().as_str() {
      "p" => !is_set_apart(child) && !running_text(child).trim().is_empty(),
      "itemize" | "enumerate" | "description" => element_nodes(child)
        .iter()
        .filter(|item| item.get_name() == "item")
        .any(|item| element_nodes(item).iter().any(is_body_prose)),
      _ => false,
    })
}

/// A paragraph set apart from the running text: `\noindent`ed, or aligned (`center`, `flushleft`, `\raggedright`).
fn is_set_apart(node: &Node) -> bool {
  node.get_attribute("align").is_some()
    || node.get_attribute("class").is_some_and(|class| {
      class
        .split_whitespace()
        .any(|c| c == "ltx_noindent" || c.starts_with("ltx_align_"))
    })
}

/// The text of a paragraph outside the boxes set in it (a minipage, a `\parbox`, a tabular, a picture) and its notes.
fn running_text(node: &Node) -> String {
  fn walk(node: &Node, out: &mut String) {
    for c in node.get_child_nodes() {
      match c.get_type() {
        Some(NodeType::TextNode) => out.push_str(&c.get_content()),
        Some(NodeType::ElementNode) => {
          let name = c.get_name();
          let boxed = name.starts_with("inline-")
            || matches!(
              name.as_str(),
              "tabular" | "picture" | "svg" | "graphics" | "note" | "block" | "logical-block"
            );
          if !boxed {
            walk(&c, out);
          }
        },
        _ => {},
      }
    }
  }
  let mut out = String::new();
  walk(node, &mut out);
  out
}
