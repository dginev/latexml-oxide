//! Title-page layout wrappers, leading-title promotion and title-ink dedup.

use super::*;

/// A root child that is title-page layout when it stands before the abstract: a
/// pagebreak, a paragraph, or a `logical-block` whose content survives #244's demotion
/// to `<block>`.
pub(in crate::base_utilities) fn is_titlepage_layout(node: &Node) -> bool {
  let q = document::get_node_qname(node);
  q == pin_static("ltx:pagination")
    || q == pin_static("ltx:para")
    || (q == pin_static("ltx:logical-block") && subtree_is_block_demotable(node))
}

/// Move `run` (consecutive root children) into a new `<ltx:titlepage>` in place, in
/// order; `para` → `block` as is (its content is already Block.model), `logical-block`
/// demoted bottom-up then renamed. Returns the titlepage (its own handle stays live —
/// only children are renamed).
pub(in crate::base_utilities) fn wrap_as_titlepage(
  document: &mut Document,
  root: &Node,
  run: &[Node],
) -> Result<Node> {
  // Bare tag + root namespace, as `maybe_promote_leading_title` does for <title>.
  let mut titlepage = document.insert_element_before(&run[0], "titlepage", None)?;
  if let Some(rns) = root.get_namespace() {
    let _ = titlepage.set_namespace(&rns);
  }
  for mut n in run.iter().cloned() {
    n.unlink();
    titlepage.add_child(&mut n)?;
  }
  normalize_titlepage_layout_children(document, &titlepage)?;
  Ok(titlepage)
}

/// The layout children of a title page as the schema's `titlepage_model`
/// (`Block.class*`) — and `ltx:para` — hold them: a `center`'s `logical-block`
/// and a `para` become `block`s (#244's demotion, content first, rename after).
/// Shared by `wrap_as_titlepage` and `demote_stranded_titlepage`.
pub(in crate::base_utilities) fn normalize_titlepage_layout_children(
  document: &mut Document,
  titlepage: &Node,
) -> Result<()> {
  for child in titlepage.get_child_nodes() {
    if child.get_type() != Some(NodeType::ElementNode) {
      continue;
    }
    let q = document::get_node_qname(&child);
    if q == pin_static("ltx:logical-block") {
      demote_para_class_content(document, &child)?;
      document.rename_node(child, "ltx:block", true)?;
    } else if q == pin_static("ltx:para") {
      document.rename_node(child, "ltx:block", true)?;
    }
  }
  Ok(())
}

/// Flush the queued frontmatter right after `after` (a root child), or — `None` — at
/// the top, after `/ltx:document/ltx:resource[last()]` exactly as Perl's fallback
/// (`Base_Utility.pool.ltxml:927-945`; Perl's `findnode` is document-relative, the Rust
/// cached XPath context is root-relative, hence the absolute path). The `_Capture_`
/// wrapper is filled by `insert_frontmatter` and unwrapped; the placed elements are
/// returned (their handles survive the unwrap — `unwrap_nodes` splices children up).
pub(in crate::base_utilities) fn insert_frontmatter_at(
  document: &mut Document,
  after: Option<&Node>,
) -> Result<Vec<Node>> {
  let savenode = document.get_node().clone();
  let point = match after {
    Some(a) => a.get_next_sibling(),
    None => document
      .findnode("/ltx:document/ltx:resource[last()]", None)
      .and_then(|r| r.get_next_sibling()),
  };
  let wrapper = match point {
    Some(point) => document.insert_element_before(&point, "ltx:_Capture_", None)?,
    None => {
      let parent = match after {
        Some(a) => a.get_parent(),
        None => document.get_document().get_root_element(),
      };
      match parent {
        Some(mut parent) => document.open_element_at(&mut parent, "ltx:_Capture_", None, None)?,
        None => return Ok(Vec::new()),
      }
    },
  };
  document.set_node(&wrapper);
  insert_frontmatter(document)?;
  let placed: Vec<Node> = wrapper
    .get_child_nodes()
    .into_iter()
    .filter(|n| n.get_type() == Some(NodeType::ElementNode))
    .collect();
  document.unwrap_nodes(wrapper)?;
  document.set_node(&savenode);
  Ok(placed)
}

/// Re-attach, in order, every node of `movers` that precedes `tail` to right after it.
/// Elements only, so `add_next_sibling` never meets libxml2's text-merge-and-free.
pub(in crate::base_utilities) fn move_content_free_after(
  movers: &[Node],
  mut tail: Node,
) -> Result<()> {
  for mut m in movers.iter().cloned() {
    if node_precedes(&m, &tail) {
      m.unlink();
      tail.add_next_sibling(&mut m)?;
      tail = m;
    }
  }
  Ok(())
}

/// True when `a` comes before `b` among the same parent's children.
pub(in crate::base_utilities) fn node_precedes(a: &Node, b: &Node) -> bool {
  let mut cur = a.get_next_sibling();
  while let Some(n) = cur {
    if n == *b {
      return true;
    }
    cur = n.get_next_sibling();
  }
  false
}

/// A `<ltx:pagination>` (self-closing pagebreak) or a truly empty `<ltx:para>`/
/// `<ltx:p>`/`<ltx:break>` wrapper — a node with no visible content. "Empty" means no
/// text AND no *visible* element descendant: a `<graphics>`/`<rule>`/`<svg>`/
/// `<tabular>`/`<text>`/image inside an otherwise text-empty para (a logo- or
/// rule-only cover top) renders above the title in the PDF, so it is NOT content-free
/// and must block the hoist — only nested empty `<para>`/`<p>`/`<break>` wrappers are
/// transparent. See `place_frontmatter`.
pub(in crate::base_utilities) fn node_is_content_free(node: &Node) -> bool {
  let is_pagination = with(document::get_node_qname(node), |q| q == "ltx:pagination");
  if is_pagination {
    return true;
  }
  let is_wrapper = with(document::get_node_qname(node), |q| {
    matches!(q, "ltx:para" | "ltx:p" | "ltx:break")
  });
  is_wrapper
    && text_outside_error_markers(node).trim().is_empty()
    && subtree_elements_all_invisible(node)
}

/// The text of `node`'s subtree with every `ltx:ERROR` subtree left out. An
/// undefined-command marker (`<ERROR class="undefined">\foo</ERROR>`) carries the
/// control-sequence NAME as its text — diagnostic ink, not document content (pdflatex
/// had a definition and typeset nothing for it). A leading paragraph holding only such
/// markers is therefore content-free for the frontmatter hoist (OXIDIZED_DESIGN #258;
/// pst-calendar-doc's `\DeclareDocumentMetadata`, forest-doc's `\@escapeifif`,
/// pmhanguljamo's `\fontid`, 8 docs): the marker stays in the output, relocated below
/// the frontmatter like any other content-free mover. A marker whose ARGUMENTS were
/// typeset as text (`\foo{Real words}` → `<ERROR>\foo</ERROR>Real words`) is not
/// content-free — that text is visible and stays where it is.
fn text_outside_error_markers(node: &Node) -> String {
  fn walk(node: &Node, out: &mut String) {
    for c in node.get_child_nodes() {
      match c.get_type() {
        Some(NodeType::TextNode) => out.push_str(&c.get_content()),
        Some(NodeType::ElementNode) if !is_undefined_command_marker(&c) => {
          walk(&c, out);
        },
        _ => {},
      }
    }
  }
  let mut out = String::new();
  walk(node, &mut out);
  out
}

/// `<ltx:ERROR class="undefined">\foo</ltx:ERROR>` — the marker `make_error` leaves for
/// an undefined control sequence; its text is the CS NAME. Only this class is
/// diagnostic ink: `\lx@ERROR{cls}{text}` markers and constructor failures carry
/// arbitrary text and stay visible content.
fn is_undefined_command_marker(node: &Node) -> bool {
  document::get_node_qname(node) == pin_static("ltx:ERROR")
    && node.get_attribute("class").as_deref() == Some("undefined")
}

/// Every element descendant of `node` is itself a structural wrapper
/// (`ltx:para`/`ltx:p`/`ltx:break`, or an ink-free `ltx:text`) — i.e. the subtree
/// renders nothing visible. Any other element (`ltx:graphics`, `ltx:rule`, `ltx:svg`,
/// `ltx:tabular`, an image, …) is visible content even with no text. An `ltx:text` is
/// transparent only when it can draw nothing on its own: the empty `<ltx:text>` an
/// `\hbox{}`/`\null`/`\mbox{}` leaves in a paragraph (`\cleardoublepage`'s blank
/// page; memoir `\frontmatter`) is folded away by `auto_collapse_children`/`finalize`
/// later anyway, but a `\fbox{}`/`\colorbox{..}{}` — `framed`/`backgroundcolor`, or a
/// `class`/`cssstyle` a stylesheet may paint — has ink and stays visible. Text nodes
/// are ignored here: the caller has already required the subtree's text to be
/// whitespace-only, and LaTeXML wraps loose document text in `<para><p>`, so a bare
/// non-whitespace text child does not occur at this level.
fn subtree_elements_all_invisible(node: &Node) -> bool {
  node.get_child_nodes().iter().all(|c| {
    c.get_type() != Some(NodeType::ElementNode)
      // An undefined-command marker is diagnostic ink (`text_outside_error_markers`).
      || is_undefined_command_marker(c)
      || ((with(document::get_node_qname(c), |q| {
        matches!(q, "ltx:para" | "ltx:p" | "ltx:break")
      }) || (document::get_node_qname(c) == pin_static("ltx:text") && text_is_ink_free(c)))
        && subtree_elements_all_invisible(c))
  })
}

/// An `ltx:text` with none of the attributes that can paint without content.
fn text_is_ink_free(text: &Node) -> bool {
  ![
    "framed",
    "framecolor",
    "backgroundcolor",
    "class",
    "cssstyle",
  ]
  .iter()
  .any(|attr| text.has_attribute(attr))
}

/// Is `node` one of the document model's FIRST-group elements (`LaTeXML-structure.rnc:34`),
/// read from the schema: admitted by `ltx:document` but not by body-only content
/// (`ltx:sectional-block`, model `document.body.class*`), and — since `ltx:document` also
/// admits BackMatter.class (bibliography/appendix/index/glossary) that the body group
/// takes — either `titlepage` or something `ltx:bibliography` admits (its model LEADS
/// with exactly `FrontMatter.class*, SectionalFrontMatter.class*`). A Meta.class element
/// (allowed in both groups) is NOT one. Used by `place_frontmatter` (#247).
pub(in crate::base_utilities) fn is_frontmatter_group_element(node: &Node) -> bool {
  let q = document::get_node_qname(node);
  document::can_contain_qsym(pin_static("ltx:document"), q)
    && !document::can_contain_qsym(pin_static("ltx:sectional-block"), q)
    && (q == pin_static("ltx:titlepage")
      || document::can_contain_qsym(pin_static("ltx:bibliography"), q))
}

/// A sectional unit's element (`ltx:section` … `ltx:glossary`): what ends the document-level
/// title block for `maybe_promote_leading_title` and `maybe_dedup_leading_title_ink`.
fn is_sectional_qname(q: SymStr) -> bool {
  with(q, |q| {
    matches!(
      q,
      "ltx:section"
        | "ltx:subsection"
        | "ltx:subsubsection"
        | "ltx:paragraph"
        | "ltx:subparagraph"
        | "ltx:chapter"
        | "ltx:part"
        | "ltx:appendix"
        | "ltx:bibliography"
        | "ltx:index"
        | "ltx:glossary"
    )
  })
}

/// Beyond-Perl heuristic: recover a document title from a hand-formatted leading
/// block when the author never used `\title`/`\maketitle`.
///
/// Many papers set their title as a plain `\begin{center}{\Large ...}\end{center}`
/// block instead of the frontmatter machinery, then declare only an abstract (e.g.
/// arXiv 1609.07638). With no `\title` there is no `ltx:title` frontmatter to
/// flush, so the document renders titleless. This promotes that first block — gated
/// conservatively so it does not fire on epigraphs, dedications, centered figures or
/// "draft" notices — into a real `<ltx:title>` at the document top.
///
/// Called only from the ABSTRACT-ONLY `\lx@frontmatter@fallback` branch: reaching it
/// already means no `\title`/`\author`/`\maketitle` were used yet the paper carries
/// frontmatter (an abstract) — the strongest signal a leading display block is the
/// title. It fires only when no `<ltx:title>` exists yet, the first non-resource body
/// element holds a leading centered `<ltx:p>` set in a larger-than-body font, and
/// that paragraph has non-whitespace text. On a match the paragraph's inline children
/// are MOVED into a fresh `<ltx:title>` after the resources, pruning empty wrappers.
pub(in crate::base_utilities) fn maybe_promote_leading_title(
  document: &mut Document,
) -> Result<Option<Node>> {
  // Never override an existing (real) title.
  if document.findnode("/ltx:document/ltx:title", None).is_some() {
    return Ok(None);
  }
  // The first non-resource body element (the candidate title block). Fetch it
  // with an ABSOLUTE query: nodes returned from a RELATIVE-context findnode are
  // detached for child traversal (a rust-libxml shared-node artifact —
  // get_content works but get_child_nodes is empty), so every step below walks
  // the live DOM by hand from this anchor.
  let Some(first_body) = document.findnode("/ltx:document/*[not(self::ltx:resource)][1]", None)
  else {
    return Ok(None);
  };
  // Never DESCEND INTO SECTIONAL content: on the second fallback pass (fired
  // at the next \section) the first body element can be the COMPLETED first
  // section, and an unconstrained DFS would steal a centered display-font
  // paragraph from inside it (e.g. a "Dedicated to ..." block) as the paper
  // title. A hand-formatted title block is a plain para/quote at document
  // level — reject sectional anchors outright (PR_READINESS must-fix 3).
  let anchor_q = document::get_node_qname(&first_body);
  if is_sectional_qname(anchor_q) {
    return Ok(None);
  }
  // First centered <ltx:p> in document order within the block; it must be set in
  // a larger-than-body font (the display-title signal) and hold real text.
  // Conservative rule (user ruling 2026-09-20, OXIDIZED_DESIGN #246): promote ONLY the
  // unambiguous hand-made-title shape — a plain `center`/paragraph block at document
  // level whose FIRST paragraph is centered and set in a display font that NO other
  // paragraph of the block shares. Corpus counter-examples that a looser rule mis-read
  // as titles: tikz-mirror-lens (author-first cover, `\Large{#1}` leaking onto the
  // title paragraph → two display paragraphs → ambiguous), isosigns/bootstrapicons
  // (a `VERSION 2.1` badge inside a tcolorbox → nested box), tipfr-doc (its real
  // `tipfr.sty` title also sits in a tcolorbox — declined rather than coin-flipped).
  // Whatever is not promoted stays hand-typeset layout and is wrapped as
  // `<ltx:titlepage>` by `place_frontmatter (the titlepage wrap)`.
  let is_plain_block = with(anchor_q, |q| matches!(q, "ltx:logical-block" | "ltx:para"));
  if !is_plain_block {
    return Ok(None);
  }
  let Some(title_p) = first_plain_paragraph(&first_body) else {
    return Ok(None);
  };
  if title_p.get_attribute("align").as_deref() != Some("center") {
    return Ok(None);
  }
  let nominal = {
    // #542: NOMINAL_FONT_SIZE is a float (11pt = 10.95), not an int — mirror
    // `common::font::defsize`, which reads it via lookup_float, not lookup_int.
    let v = lookup_float("NOMINAL_FONT_SIZE").map_or(0.0, |f| f.0);
    if v > 0.0 { v } else { 10.0 }
  };
  if !title_p.get_content().chars().any(char::is_alphabetic)
    || !descendant_has_display_font(document, &title_p, nominal)
  {
    return Ok(None);
  }
  // Uniqueness: any OTHER plain paragraph of the block in a display font makes the
  // title ambiguous (author-first covers, multi-line banners).
  let mut paragraphs = Vec::new();
  collect_plain_paragraphs(&first_body, &mut paragraphs);
  if paragraphs
    .iter()
    .any(|p| *p != title_p && descendant_has_display_font(document, p, nominal))
  {
    return Ok(None);
  }
  // Create <ltx:title> at the document top (after the last resource), mirroring
  // the frontmatter fallback's placement.
  let mut point = document.findnode("/ltx:document/ltx:resource[last()]", None);
  if let Some(p) = point.take() {
    point = p.get_next_sibling();
  }
  // No resources (or resource is the last child): insert before the first
  // body element instead of silently declining (review corner-case).
  let point = point.unwrap_or_else(|| first_body.clone());
  // Create with a BARE tag + bind the root (default LaTeXML) namespace, so it
  // serializes as <title> like a real frontmatter title — a prefixed "ltx:title"
  // qname would emit a stray <ltx:title>. Mirrors open_element_internal's
  // default-namespace path.
  let mut title = document.insert_element_before(&point, "title", None)?;
  if let Some(rns) = document
    .get_document()
    .get_root_element()
    .and_then(|r| r.get_namespace())
  {
    let _ = title.set_namespace(&rns);
  }
  // MOVE the paragraph's inline children into the title (a true move preserves
  // xml:ids, so any footnotes/marks carry over rather than being cloned+orphaned).
  for mut child in title_p.get_child_nodes() {
    child.unbind();
    title.add_child(&mut child)?;
  }
  // Drop the now-empty paragraph, and any wrapper ancestor it leaves empty
  // (para → logical-block), so we don't strand empty blocks above the abstract.
  let mut victim = title_p;
  loop {
    let parent = victim.get_parent();
    document.remove_node(victim);
    match parent {
      Some(p) if !has_element_child(&p) => {
        let is_wrapper = with(document::get_node_qname(&p), |name| {
          name == "ltx:para" || name == "ltx:logical-block"
        });
        if is_wrapper {
          victim = p;
          continue;
        }
      },
      _ => {},
    }
    break;
  }
  // The promoted <title>: the node a deferred frontmatter flush lands AFTER when no
  // titlepage layout follows it. A surviving rest of the hand-formatted block (an
  // author paragraph) is wrapped as <ltx:titlepage> by the caller. `first_body` may
  // have been freed above — never return it.
  Ok(Some(title))
}

/// Whitespace-collapsed, lowercased text — so a `<break>` (`\\`) and any spacing
/// differences between the structured title and its hand-typeset copy don't
/// defeat the comparison in [`maybe_dedup_leading_title_ink`].
pub(in crate::base_utilities) fn normalize_frontmatter_text(s: &str) -> String {
  s.split_whitespace()
    .collect::<Vec<_>>()
    .join(" ")
    .to_lowercase()
}

/// Companion to [`maybe_promote_leading_title`] for the mirror case
/// (arXiv/html_feedback#6924, witness arXiv 2608.10928): a structured
/// `<ltx:title>` DOES exist (from `\title`), but the author ALSO hand-typeset the
/// title as a leading centered display-font block and never called `\maketitle`,
/// so that block reproduces the structured title and the title renders twice.
///
/// Prioritize the structured metadata (LaTeXML's unified Frontmatter API stays
/// authoritative): remove the redundant leading title *ink*, keeping the semantic
/// `<ltx:title>` and any author/abstract ink (which has no structured counterpart,
/// so is the only copy). Fires only on a FULL normalized-text match against the
/// structured title, and only for a leading, non-sectional, display-font centered
/// paragraph of the document-level title block (not inside a float, a titlepage or any
/// other container) — so an unrelated centered display block is never removed — and at
/// most once per document, at the first flush that sees the structured title (68s;
/// guard `perfect_kernel_batch68s`). Reuses the same detection helpers as the promote path.
///
/// It is the LAST RESORT of the title placement rules (OXIDIZED_DESIGN #146): a text match
/// where no print event explains the copy — a print of the marked `\@title` is lifted by its marks first
/// ([`lift_marked_field_ink`], OXIDIZED_DESIGN #715), and then this does not run (`lx_title_ink_dedup_done`).
/// Never in a document with an `<ltx:titlepage>`: there the XSLT does not display the structured
/// title (LaTeXML-structure-xhtml.xsl, the `ltx:title` template's
/// `not(parent::*/child::ltx:titlepage)`; Perl's copy :358-359) — the cover IS the displayed title,
/// and deleting its copy left none (1304.0681, 1503.03277: no `<h1>`, Perl shows the title once).
pub(in crate::base_utilities) fn maybe_dedup_leading_title_ink(
  document: &mut Document,
) -> Result<()> {
  // Needs a structured document title to prioritize over.
  let Some(title) = document.findnode("/ltx:document/ltx:title", None) else {
    return Ok(());
  };
  let title_text = normalize_frontmatter_text(&title.get_content());
  if title_text.is_empty() {
    return Ok(());
  }
  // Once per document: the hand-typeset title block is judged at the first flush that sees the
  // structured title — the fallback runs at EVERY sectioning command (`\@startsection@hook`) and
  // at `\end{document}`, and a second look removed a second copy each time (one per section), down
  // into the body. A copy still there after that first look is the author's ink, not the title.
  if lookup_bool("lx_title_ink_dedup_done") {
    return Ok(());
  }
  assign_value("lx_title_ink_dedup_done", true, Some(Scope::Global));
  // A titlepage beside the title: the structured title is metadata and the cover the display.
  if document
    .findnode("/ltx:document/ltx:titlepage", None)
    .is_some()
  {
    return Ok(());
  }
  let nominal = {
    let v = lookup_float("NOMINAL_FONT_SIZE").map_or(0.0, |f| f.0);
    if v > 0.0 { v } else { 10.0 }
  };
  // Leading centered paragraphs of the document-level title block (before the first sectional
  // unit, reached only through title-block wrappers — a para, a titlepage, a cover minipage;
  // never inside a figure or any other container, whose content is the author's).
  // The nodes come from a walk of the live DOM from the absolutely queried root, so
  // they support child traversal (shared-node caveat in `maybe_promote_leading_title`).
  let Some(root) = document.findnode("/ltx:document", None) else {
    return Ok(());
  };
  let candidates = leading_centered_paragraphs(&root);
  // Remove the FIRST leading centered display-font block that EXACTLY reproduces
  // the structured title. Only one — a paper may repeat the title text later
  // (e.g. a running head); we drop just the redundant hand-typeset title.
  let Some(victim) = candidates.into_iter().find(|p| {
    descendant_has_display_font(document, p, nominal)
      && normalize_frontmatter_text(&p.get_content()) == title_text
  }) else {
    return Ok(());
  };
  // Log what we drop (never remove content silently; #6924).
  Info!(
    "frontmatter",
    "title_ink_dedup",
    s!(
      "dropped a hand-typeset title block duplicating the structured <ltx:title> (\"{title_text}\")"
    )
  );
  // Remove the redundant title ink, pruning any title-block wrapper it leaves empty (para →
  // logical-block, a minipage that held only the title and the paragraph around it) —
  // mirroring the promote path's cleanup. Sibling `<p>`s (e.g. the author block) keep their
  // wrapper non-empty and survive. A wrapper that carries `labels` (a `\label` target) or a
  // frame (`\fbox{\begin{minipage}…}`, `framed`) is kept, empty: removing it would leave a
  // dangling reference or drop ink of its own (68s review probes p7_fbox_cover, p8).
  let mut victim = victim;
  loop {
    let parent = victim.get_parent();
    document.remove_node(victim);
    match parent {
      Some(p)
        if is_title_block_wrapper(document::get_node_qname(&p))
          && !has_element_child(&p)
          && p.get_content().trim().is_empty()
          && !p.has_attribute("labels")
          && !p.has_attribute("framed") =>
      {
        victim = p;
      },
      _ => break,
    }
  }
  Ok(())
}

/// The containers a hand-typeset title block is set in at document level: paragraph wrappers
/// (`ltx:para`, `ltx:logical-block`), a display block (`ltx:block`, a `{center}` holding a logo),
/// a cover minipage (`ltx:inline-block`, inside the `ltx:p` of its paragraph — entered only
/// through a cover paragraph, [`is_cover_box_paragraph`]). Not a float, a list, a theorem, a
/// table or a sectional unit: what is set there is the author's content even when it repeats
/// the title. Not a `{titlepage}` (`ltx:titlepage`) either: the cover is the displayed title
/// there, and the structured title is metadata ([`maybe_dedup_leading_title_ink`]).
fn is_title_block_wrapper(q: SymStr) -> bool {
  with(q, |q| {
    matches!(
      q,
      "ltx:para" | "ltx:logical-block" | "ltx:block" | "ltx:inline-block" | "ltx:p"
    )
  })
}

/// A non-centered `ltx:p` holding only boxes: its element children are all `ltx:inline-block`s
/// and its text is whitespace — the `\noindent\begin{minipage}…\end{minipage}` cover paragraph
/// (2010.07703). A paragraph of running prose with a `\parbox` in it is not: the box's words
/// are the sentence's (68s review probe p2_parbox_only, pdflatex prints them mid-sentence).
fn is_cover_box_paragraph(p: &Node) -> bool {
  let mut has_box = false;
  for c in p.get_child_nodes() {
    match c.get_type() {
      Some(NodeType::ElementNode) => {
        if document::get_node_qname(&c) != pin_static("ltx:inline-block") {
          return false;
        }
        has_box = true;
      },
      Some(NodeType::TextNode) if !c.get_content().trim().is_empty() => return false,
      _ => {},
    }
  }
  has_box
}

/// The `<ltx:p align="center">` paragraphs of `node` reached only through title-block wrappers
/// ([`is_title_block_wrapper`]), in document order; a centered paragraph is taken whole (a
/// minipage inside it is part of its text), any other `ltx:p` is entered only when it is a
/// cover paragraph of boxes ([`is_cover_box_paragraph`]).
fn collect_title_block_paragraphs(node: &Node, out: &mut Vec<Node>) {
  // A framed wrapper (`\fbox{\begin{minipage}…}`, `framed`) is the author's own ink around the title: removing the copy
  // inside left an empty frame the PDF does not have, so it is never searched (main-session ruling 2026-10-16, until
  // the last-resort step promotes the author's block in place; OXIDIZED_DESIGN #146).
  if node.has_attribute("framed") {
    return;
  }
  let q = document::get_node_qname(node);
  if q == pin_static("ltx:p") {
    if node.get_attribute("align").as_deref() == Some("center") {
      out.push(node.clone());
      return;
    }
    if !is_cover_box_paragraph(node) {
      return;
    }
  }
  if !is_title_block_wrapper(q) {
    return;
  }
  for c in node.get_child_nodes() {
    if c.get_type() == Some(NodeType::ElementNode) {
      collect_title_block_paragraphs(&c, out);
    }
  }
}

/// The `<ltx:p align="center">` paragraphs of the document-level title block: those reached from
/// `root`'s children only through title-block wrappers ([`collect_title_block_paragraphs`]) and
/// before its first sectional child (a back-matter list is passed over), in document order. A
/// paragraph inside a float, a list, a theorem, a titlepage or any other container, or inside a
/// chapter or part, is not the leading title block: the walk once took every centered paragraph
/// before the first `<ltx:section>`, so a figure's centered line and a chapter's opening block were
/// removed as the title (68p review probe `float.tex`; 2608.10928 the witness of the rule; a cover
/// minipage 2010.07703; the titlepage covers 1304.0681, 1503.03277, 1601.07172, 2606.13892 are kept
/// whole since 68s round 2). Only the document's children are visited and the walk stops at the
/// first sectional unit, so it stays linear in the leading block (it runs once per document; the
/// old XPath scanned `preceding::` for every centered paragraph at every sectioning command —
/// 2208.14314: 121 of 133 s; 2108.10297: 113 of 146 s; PERFORMANCE.md Principle 6).
fn leading_centered_paragraphs(root: &Node) -> Vec<Node> {
  let mut out = Vec::new();
  let mut cur = root.get_first_child();
  while let Some(node) = cur {
    if node.get_type() == Some(NodeType::ElementNode) {
      let q = document::get_node_qname(&node);
      // WORKAROUND, not a rule of the title block: a bibliography, index or glossary list placed
      // ahead of the cover does not end it. 2010.07703's bibliography (`\bibliography` at
      // thesis.tex:480, under `\part{Listings and Bibliography}`) is built at the head of the XML,
      // before its cover minipage, by bibentry's `\nobibliography*` in the preamble
      // (texpreamble.tex:820) — a separate defect (RED index-bib/nobibliography_star_prints_nothing,
      // SYNC_STATUS); once that is fixed this skip has nothing to pass over.
      let back_matter = with(q, |q| {
        matches!(q, "ltx:bibliography" | "ltx:index" | "ltx:glossary")
      });
      if is_sectional_qname(q) && !back_matter {
        break;
      }
      collect_title_block_paragraphs(&node, &mut out);
    }
    cur = node.get_next_sibling();
  }
  out
}

/// Insert FrontMatter into document, if not already added
/// Perl: insertFrontMatter($document).
/// Drop every `ltx:_Frontmatter_Capture_` position marker (`\lx@frontmatter@mark`);
/// called once the frontmatter is placed and again at `\end{document}` as a safety net.
pub fn remove_frontmatter_marks(document: &mut Document) {
  for mark in document.findnodes("//ltx:_Frontmatter_Capture_", None) {
    document.remove_node(mark);
  }
}
