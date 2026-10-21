//! Math cleanup: `cleanup_math`, Meta content floated out of math, `cleanup_xmtext`.

use super::*;

/// Would `cleanup_math`'s unwrap of a trivial (`XMText`/`XMHint`-only) `<Math>`
/// produce a schema-valid child of a `<MathFork>` main branch?
///
/// The `MathFork` content model is `(Math|text), MathBranch*`
/// (`LaTeXML/lib/LaTeXML/resources/RelaxNG/LaTeXML-block.rnc:109`): its main
/// branch admits exactly one leading `<Math>` or `<text>`. The unwrap is safe
/// there ONLY when it yields that single `<text>` — one `<XMText>` holding one
/// text run, with no `<XMHint>` spacing and no `<rule>`/`inline-block`. Anything
/// else (a `<rule>` cell, a second text run, an `inline-block`) would splice a
/// forbidden node straight under `<MathFork>`, so the caller keeps the `<Math>`
/// wrapper instead.
fn cleanup_math_unwrap_valid_under_mathfork(mathnode: &Node) -> bool {
  // <Math> holds exactly one <XMath>.
  let xmaths = mathnode.get_child_nodes();
  let [xmath] = xmaths.as_slice() else {
    return false;
  };
  // An EMPTY main branch (an entirely-empty aligned column, e.g. a leading `&&`
  // in alignat) unwraps to NOTHING — that splices no forbidden node, and
  // removing the empty <Math> lets the empty <MathFork> be pruned downstream
  // (Perl / pre-56eg behavior). So an empty main branch is SAFE to unwrap.
  // (Without this, batch 56eg kept the wrapper and a spurious
  // <MathFork><Math text="absent">…<td/><td/> survived — mathtools_test S12.E27.)
  let inner = xmath.get_child_nodes();
  if inner
    .iter()
    .all(|n| n.get_type() == Some(NodeType::CommentNode))
  {
    return true;
  }
  // Otherwise: <XMath> holds exactly one child, and it is an <XMText> (no XMHint
  // spacing, no multiple runs).
  let [xmtext] = inner.as_slice() else {
    return false;
  };
  if !document::with_node_qname(xmtext, |qname| qname == "ltx:XMText") {
    return false;
  }
  // The <XMText> holds exactly one non-comment child, text-like enough to
  // become a `<text>` (a text node, or an already-`<text>`/`<Math>` element) —
  // never a `<rule>`/`inline-block`.
  let kids: Vec<Node> = xmtext
    .get_child_nodes()
    .into_iter()
    .filter(|kid| kid.get_type() != Some(NodeType::CommentNode))
    .collect();
  let [kid] = kids.as_slice() else {
    return false;
  };
  match kid.get_type() {
    Some(NodeType::TextNode) => true,
    Some(NodeType::ElementNode) => {
      document::with_node_qname(kid, |qname| qname == "ltx:text" || qname == "ltx:Math")
    },
    _ => false,
  }
}

/// The schema's `Meta.class` (LaTeXML-meta.rnc:17): side content that math text
/// never admits (`XMText_model = (text | Inline.class | Misc.class)*`).
pub(crate) const META_CLASS: [&str; 7] = [
  "ltx:note",
  "ltx:indexmark",
  "ltx:glossarydefinition",
  "ltx:declare",
  "ltx:rdf",
  "ltx:resource",
  "ltx:navigation",
];

/// Float the `Meta.class` content of a Math's text out of the math (user ruling
/// 2026-09-23, OXIDIZED_DESIGN #272). `\footnote`/`\index`/`\gls` inside `\text{}`
/// construct in the `ltx:text` that `\text` opens, which admits a note, so their `^`
/// float stops there. `cleanup_xmtext` then unwraps that `ltx:text`, and the marker
/// is left as a direct `XMText` child: schema-invalid, with the footnote's text read
/// into the formula's `text=`. Perl gives the same tree (ribbonproofs, sidenotesplus,
/// ryethesis). Each outermost Meta element moves, in document order, to just after
/// the Math at the nearest ancestor level that admits it. That is where a bare
/// `\footnote` in math floats: the `p` for inline math, the `equation` for a display.
/// Nothing moves when no level admits it.
fn float_meta_out_of_math(document: &mut Document, mathnode: &Node) -> Result<()> {
  let is_meta = |n: &Node| with(document::get_node_qname(n), |t| META_CLASS.contains(&t));
  let movers: Vec<Node> = document
    .findnodes(
      "descendant::ltx:XMText//*[self::ltx:note or self::ltx:indexmark or \
       self::ltx:glossarydefinition or self::ltx:declare or self::ltx:rdf or \
       self::ltx:resource or self::ltx:navigation]",
      Some(mathnode),
    )
    .into_iter()
    .filter(|m| {
      // Outermost only: a Meta element inside another moves with it.
      let mut up = m.get_parent();
      while let Some(p) = up {
        if p == *mathnode {
          return true;
        }
        if is_meta(&p) {
          return false;
        }
        up = p.get_parent();
      }
      true
    })
    .collect();
  let mut last: Option<(Node, Node)> = None; // (anchor, last node placed after it)
  for mut m in movers {
    let qname = document::get_node_qname(&m);
    let mut anchor = mathnode.clone();
    let placed = loop {
      match anchor.get_parent() {
        Some(p) if p.get_type() == Some(NodeType::ElementNode) => {
          if with(qname, |t| document::can_contain(&p, t)) {
            break true;
          }
          anchor = p;
        },
        _ => break false,
      }
    };
    if !placed {
      continue;
    }
    let mut prev = match &last {
      Some((a, p)) if *a == anchor => p.clone(),
      _ => anchor.clone(),
    };
    let mut emptied = m.get_parent();
    // Elements only, so `add_next_sibling` never meets libxml2's text merge.
    m.unlink();
    prev.add_next_sibling(&mut m)?;
    last = Some((anchor, m));
    // A math-text wrapper the move leaves empty goes too: an empty `XMText` would
    // parse as a term of the formula (`a * [] * b`).
    while let Some(w) = emptied {
      let (is_text, is_xmtext) = with(document::get_node_qname(&w), |t| {
        (t == "ltx:text", t == "ltx:XMText")
      });
      let blank = w.get_child_nodes().iter().all(|c| match c.get_type() {
        Some(NodeType::TextNode) => c.get_content().trim().is_empty(),
        Some(NodeType::CommentNode) => true,
        _ => false,
      });
      // Never the insertion point: the Math's after_close restores it afterwards.
      if !(is_text || is_xmtext)
        || !blank
        || w.get_attribute_ns("id", XML_NS).is_some()
        || w == *document.get_node()
      {
        break;
      }
      emptied = if is_xmtext { None } else { w.get_parent() };
      document.remove_node(w);
    }
  }
  Ok(())
}

/// Whether an `ltx:XMath` of `mathnode` holds real math: Perl TeX_Math.pool.ltxml:197-201
/// `findnodes('ltx:XMath/ltx:*[local-name() != "XMText" and local-name() != "XMHint" and
/// not(local-name() = "XMTok" and (@role="PUNCT" or @role="PERIOD") and not(preceding-sibling::*)
/// and not(following-sibling::*))]')` is non-empty. Evaluated as one walk over the children: the
/// sibling axes, evaluated per child, made the query quadratic in the XMath's width — a math left
/// open to `\end{document}` holds the rest of the document (1907.09124: 265 s of building).
fn has_real_math(mathnode: &Node) -> bool {
  mathnode
    .get_child_elements()
    .iter()
    .filter(|xmath| document::with_node_qname(xmath, |qname| qname == "ltx:XMath"))
    .any(|xmath| {
      let children = xmath.get_child_elements();
      let alone = children.len() == 1;
      children.iter().any(|child| {
        document::with_node_qname(child, |qname| match qname {
          "ltx:XMText" | "ltx:XMHint" => false,
          "ltx:XMTok" => {
            !(alone
              && matches!(
                child.get_attribute("role").as_deref(),
                Some("PUNCT" | "PERIOD")
              ))
          },
          other => other.starts_with("ltx:"),
        })
      })
    })
}

pub fn cleanup_math(document: &mut Document, mathnode: Node) -> Result<()> {
  float_meta_out_of_math(document, &mathnode)?;
  // Cleanup ltx:Math elements; particularly if they aren't "really" math.
  // But record the oddity with class=ltx_markedasmath

  // If the Math ONLY contains XMath/XMText and XMHint, it apparently isn't math at all!?!
  // Single token PUNCTs can also be taken out of math.
  if !has_real_math(&mathnode) {
    // Surpass-Perl schema-validity guard (frege.tex witness; batch 56eg). Perl's
    // cleanup_Math unwraps unconditionally (TeX_Math.pool.ltxml:219) even when
    // the parent is a <MathFork> main branch, splicing a bare <rule>/<inline-block>
    // or a second child straight under <MathFork> — which the MathFork model
    // (Math|text),MathBranch* forbids (LaTeXML-block.rnc:109), yielding
    // schema-invalid XML in ~14 logic/proof manuals (frege, principia, natded, …).
    // When the unwrap here would NOT yield a single valid <text>, keep the <Math>
    // wrapper (the parser's own valid main-branch form) — the same handling the
    // "real math" else-branch below uses.
    if mathnode
      .get_parent()
      .is_some_and(|parent| document::with_node_qname(&parent, |qname| qname == "ltx:MathFork"))
      && !cleanup_math_unwrap_valid_under_mathfork(&mathnode)
    {
      cleanup_xmtext_outer(document, &mathnode)?;
      return Ok(());
    }
    // So unwrap down to the contents of the XMText's.
    let xmath_children: Vec<_> = mathnode
      .get_child_nodes()
      .into_iter()
      .flat_map(|child| child.get_child_nodes())
      .collect();
    let mut texts: Vec<Node> = vec![];
    for xmnode in xmath_children {
      let is_hint = document::with_node_qname(&xmnode, |qname| qname == "ltx:XMHint");
      if is_hint {
        // Convert XMHint width to spacing characters
        if let Some(width_str) = xmnode.get_attribute("width") {
          // Width may be a full glue spec like "2.22217pt plus 1.11108pt minus 2.22217pt"
          // Extract just the base dimension (before "plus" or "minus")
          let base_dim_str = width_str
            .split_once(" plus")
            .or_else(|| width_str.split_once(" minus"))
            .map_or(width_str.as_str(), |(base, _)| base);
          // Try parsing as Dimension (pt). If that fails, handle mu units
          // by converting mu→pt (1mu = font_size/18).
          let dim_opt = Dimension::from_str(base_dim_str).ok().or_else(|| {
            if base_dim_str.ends_with("mu") {
              let mu_str = base_dim_str.trim_end_matches("mu").trim();
              mu_str.parse::<f64>().ok().map(|mu_val| {
                let fs = lookup_font().and_then(|f| f.get_size()).unwrap_or(10.0);
                Dimension::from_str(&format!("{}pt", mu_val * fs / 18.0)).unwrap_or_default()
              })
            } else {
              None
            }
          });
          if let Some(dim) = dim_opt {
            // No font to thread: this runs over an already-built
            // `<ltx:XMHint>`, which records only `width` — the digest font is
            // not on the element. Keeps the ambient read (and with it the
            // WHEN-dependence described in tex_glue::dimension_to_spaces);
            // reachable only for math hints, which the streaming sweep has
            // never caught diverging.
            let spaces = crate::tex_glue::dimension_to_spaces(dim, None);
            if !spaces.is_empty()
              && let Ok(text_node) = Node::new_text(&spaces, &document.document)
            {
              texts.push(text_node);
            }
          }
        }
      } else {
        // is XMText — process its children
        for mut child in xmnode.get_child_nodes() {
          let t = child.get_type();
          if t == Some(NodeType::CommentNode) {
            continue;
          }
          if t != Some(NodeType::ElementNode) {
            // Make sure we've got an element
            child = document.wrap_nodes("ltx:text", vec![child])?.unwrap();
            // Perl makes this `ltx:text` in the Math's place, from an array
            // (`appendTree`), so it records the box of the Math's parent, not
            // of the XMText it is wrapped in here (`replace_node_as_tree`).
            document.purge_node_boxes_rec(&child);
          }
          // Now record that it originally was marked as math
          document.add_class(&mut child, "ltx_markedasmath")?;
          texts.push(child);
        }
      }
    }
    // and replace the whole Math with the pieces — Perl `replaceTree`, whose
    // boxes follow the pieces into an auto-opened parent.
    document.replace_node_as_tree(mathnode.clone(), texts)?;
  } else {
    // Cleanup any remaining XMTexts
    cleanup_xmtext_outer(document, &mathnode)?;
  }
  Ok(())
}

// Here's for an inverse case: when an XMText isn't "really" just text
// if it only contains an Math  ORR, a tabular with only Math in the cells?
// First case: pull it back into the math, but in an XMWrap to isolate it for parsing.
// Should we just pull any mixed text math up or only a single Math?
// For the tabular case, convert it to an XMArray.

// Note that normally, we'd do afterClose on ltx:XMText,
// but since the ltx:XMText closes before the outer ltx:Math,
// we would keep cleanup_Math from recognizing the trivial case of
// a single ltx:tabular in an equation (perverse, but people do that).
// So, we put this one on ltx:Math also, and scan for any contained XMText to fixup.

fn cleanup_xmtext_outer(document: &mut Document, math_node: &Node) -> Result<()> {
  for text_node in document.findnodes("descendant::ltx:XMText", Some(math_node)) {
    cleanup_xmtext(document, text_node)?;
  }
  Ok(())
}

fn cleanup_xmtext(document: &mut Document, mut text_node: Node) -> Result<()> {
  // We're really only interested in reducing nested math, right?
  // But actually also collapsing ltx:XMText/ltx:text
  // Apply "outer" simplifications: remove ltx:text or ltx:p wrappings.

  // A single "simple" element, with a single child
  let mut children;
  loop {
    children = text_node.get_child_nodes();
    if (children.len() != 1)
      || document
        .findnodes(
          "ltx:text | ltx:inline-block[count(*)=1] | ltx:p",
          Some(&text_node),
        )
        .is_empty()
    {
      break;
    }
    let child = children.pop().unwrap();
    document.copy_node_font(&child, &mut text_node)?;
    for (key, value) in child.get_attributes() {
      // Copy the child's attributes (should Merge!!) — through the schema-gated
      // setter: a collapsed `\rotatebox`/`\raisebox` inline-block hands over
      // `angle`/`innerdepth`/`innerwidth`/`xtranslate`…, none of which
      // `XMText_attributes` (LaTeXML-math.rnc:223) admits. Perl's own copy here
      // (TeX_Math.pool.ltxml:260) is raw too, yet its OUTPUT keeps only the
      // model's attributes (`depth height width yoffset rpadding …`; verified on
      // the forced-unparsed witness) — this setter reproduces that output. The
      // raw copy showed only when the math failed to parse (`ltx_math_unparsed`
      // keeps this XMText; a parse rebuilds it): principia's `\pmcexists`, six
      // schema errors on one XMText, Perl clean (batch 56fy). `xml:id` arrives
      // from rust-libxml under its LOCAL name `id`; Perl skips it (`:260`).
      if key != "id" && key != "xml:id" {
        document.set_attribute(&mut text_node, &key, &value)?;
      }
    }
    document.unwrap_nodes(child)?;
  }

  // Now apply a simplifying rule for nested Math
  // If the XMText contains a single Math, pull it's content up in
  if children.len() == 1 && !document.findnodes("ltx:Math", Some(&text_node)).is_empty() {
    // Replace XMText by XMWrap/*  (this should preserve the parse?)
    document.rename_node(text_node, "ltx:XMWrap", false)?; // text_node =
    let first_child = children.pop().unwrap();
    let first_granchildren = first_child.get_child_nodes();
    document.replace_node(
      first_child,
      first_granchildren
        .into_iter()
        .flat_map(|grandchild| grandchild.get_child_nodes())
        .collect(),
    )?;
  // # # RISKY!!!! If SOME nodes are math...
  // # # pull the whole sequence up, unwrap the math and putting the rest back in XMText.
  // # # Even with the XMWrap, this seems to wreak havoc on parsing and structure?
  // # if(document.findnodes('ltx:Math',$text_node)){
  // #   # Replace XMText by XMWrap/*  (this should preserve the parse?)
  // #   $text_node=document.renameNode($text_node,'ltx:XMWrap');
  // #   foreach my $child (@children){
  // #     if($model->getNodeQName($child) eq 'ltx:Math'){
  // #       document.replaceNode($child,map($_->childNodes,$child->childNodes)); }
  // #     else {
  // #       document.wrapNodes('ltx:XMText',$child); }}}
  // If a single tabular that ONLY(?) contains Math, turn into an XMArray
  // Well, a tabular REALLY shouldn't be in math;
  // How much math should determine the switch?
  // [will alignment attributes be lost?]
  } else if children.len() == 1
    && model::with_node_qname(children.first().as_ref().unwrap(), |qname| {
      qname == "ltx:tabular"
    })
  //// Should we ALWAYS do this, or just for some minimal amount of math???
  ////        && !document.findnodes('ltx:tabular/ltx:tr/ltx:td/text()'
  ////                                 .' | ltx:tabular/ltx:tbody/ltx:tr/ltx:td/text()'
  ////                                 .' | ltx:tabular/ltx:tr/ltx:td[not(ltx:Math)]'
  ////                                 .' | ltx:tabular/ltx:tbody/ltx:tr/ltx:td[not(ltx:Math)]',
  ////                                 $text_node)
  {
    // Perl TeX_Math.pool.ltxml L281-310: unwrap tbody, rename
    // tabular→XMArray / tr→XMRow / td→XMCell, within each cell unwrap any
    // Math (pull XMath contents up) or wrap plain content in XMText,
    // propagate XMText attributes up to the table, then unwrap the XMText.
    // First: remove any ltx:tbody wrapping.
    for tb in document.findnodes("ltx:tabular/ltx:tbody", Some(&text_node)) {
      document.unwrap_nodes(tb)?;
    }
    // Rename tabular → XMArray
    let first_child = children.first().cloned().unwrap();
    let mut table = document.rename_node(first_child, "ltx:XMArray", false)?;
    let rows: Vec<Node> = table
      .get_child_nodes()
      .into_iter()
      .filter(|n| n.get_type() == Some(NodeType::ElementNode))
      .collect();
    for row in rows {
      let row = document.rename_node(row, "ltx:XMRow", false)?;
      let cells: Vec<Node> = row
        .get_child_nodes()
        .into_iter()
        .filter(|n| n.get_type() == Some(NodeType::ElementNode))
        .collect();
      for cell in cells {
        let cell = document.rename_node(cell, "ltx:XMCell", false)?;
        // Perl iterates `$cell->childNodes` — TEXT nodes included — and wraps every
        // non-Math child in XMText. A cell that escaped the `${}##{}$` template
        // (`\multispan{n}\upbracefill`, oubraces.sty:40-41; halloweenmath's arrows,
        // nath's braces) holds its glyph as a bare text node, which an
        // elements-only filter left as raw #PCDATA in the XMCell — schema-invalid
        // (`text not allowed here`) where Perl emits `<XMText>⏟</XMText>`. Text nodes
        // are taken as Perl takes them (a whitespace-only cell becomes
        // `<XMText> </XMText>` there too); only comments are skipped (batch 56ez).
        let cell_kids: Vec<Node> = cell
          .get_child_nodes()
          .into_iter()
          .filter(|n| {
            matches!(
              n.get_type(),
              Some(NodeType::ElementNode | NodeType::TextNode)
            )
          })
          .collect();
        for m in cell_kids {
          if model::with_node_qname(&m, |qn| qn == "ltx:Math") {
            // Perl: replaceNode($m, map { $_->childNodes } $m->childNodes)
            //  — Math wraps an XMath, XMath wraps the actual tokens. Pull
            //  those up, discarding the Math/XMath layers.
            let grandkids: Vec<Node> = m
              .get_child_nodes()
              .into_iter()
              .flat_map(|x| x.get_child_nodes())
              .collect();
            document.replace_node(m, grandkids)?;
          } else {
            document.wrap_nodes("ltx:XMText", vec![m])?;
          }
        }
      }
    }
    // Copy all of XMText's attributes (incl. xml:id) onto the table.
    let id_opt = text_node.get_attribute_ns("id", "http://www.w3.org/XML/1998/namespace");
    for (key, value) in text_node.get_attributes() {
      table.set_attribute(&key, &value)?;
    }
    // Unwrap the XMText (its only child is now `table`).
    document.unwrap_nodes(text_node)?;
    if let Some(id) = id_opt {
      document.unrecord_id(&id);
      // Re-record the id on the renamed table (and any nested ids).
      document.record_node_ids(&table)?;
    }
  }
  Ok(())
}
