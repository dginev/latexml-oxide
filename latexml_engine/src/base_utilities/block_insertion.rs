//! `insert_block_as` (Perl `insertBlock`, TeX_Box.pool.ltxml), one function.

use super::*;

/// This attempts to be a generalize vbox construction;
///
/// The idea is to receeive block-like material, possibly wrapped in appropriate
/// container which gets attributes.
///
/// The contents are constructed in an ltx:_CaptureBlock_ element,
/// designed to accept all reasonable block material from several levels,
/// and then determine which container element is most apprpriate for both the conent & context
/// from block, logical-block or sectional-block, or the inline- variants.
///
/// `boxed`: the block is a TeX box (a `\vbox`, `\vtop`, `\parbox`, a minipage, a `p` column, a framed box), its
/// capture marked for a frontmatter field's print (core `Document::mark_field_box`, OXIDIZED_DESIGN #715); not an
/// aligning list environment's (`{center}`, `{flushright}`: [`aligning_environment`]).
pub(super) fn insert_block_as(
  document: &mut Document,
  contents: &Digested,
  block_attr: HashMap<String, String>,
  in_paragraph: bool,
  boxed: bool,
) -> Result<Vec<Node>> {
  // Perl #2829 (TeX_Box.pool insertBlock L449-456): callers may (sloppily)
  // pass ALL Whatsit properties, but only SOME are intended as attributes.
  // Filter out non-attributes, based on what ltx:figure accepts — a maximal
  // set of the attributes for candidate containers.
  let block_attr: HashMap<String, String> = block_attr
    .into_iter()
    .filter(|(k, _)| document::can_have_attribute("ltx:figure", k))
    .collect();
  // Create something like:
  // "<ltx:inline-block vattach='$vattach' height='#height'>#2</ltx:inline-block>"
  let context_opt = document.get_element(); // Where we originally start inserting.
  if context_opt.is_none() {
    // edge case: if we start the doc with a block, the context is empty
    document.absorb(contents, None)?;
    return Ok(Vec::new());
  }
  let mut context = context_opt.unwrap();
  let mut context_tag = document::get_node_qname(&context);
  // svg is slightly tricky
  // An `svg:foreignObject` context is LaTeXML content again (it holds ltx
  // elements), not SVG proper: the block keeps its attributes and its own
  // wrapper there. With the foreignObject model widened to Flow
  // (OXIDIZED_DESIGN #186) the SVG rule "attributes are ignorable" would
  // unwrap a minipage straight into the foreignObject and hoist its
  // `width="31.37em"` over the foreignObject's px width (fixture
  // tikz/various_colors).
  let (is_svg, is_xmath, is_xmtext) = with(context_tag, |tag| {
    (
      tag.starts_with("svg:") && tag != "svg:foreignObject",
      tag.starts_with("ltx:XM"),
      tag == "ltx:XMText",
    )
  });
  // The width stays the Dimension (Perl's insertBlock), in SVG context too: a block a measured foreignObject holds
  // is rewritten to ems of that foreignObject's anchor by its sizer (tex_box.rs), the one basis the browser reads.
  let ignorable_attr = is_svg || block_attr.is_empty(); // if we do not REQUIRE the attributes
  if is_xmath && !is_xmtext {
    // but math always needs this
    context = document.open_element("ltx:XMText", None, None)?;
    context_tag = document::get_node_qname(&context);
  }
  // An inline-only container that holds `inline-block` (Misc.class) but neither
  // `#PCDATA` nor `block` (Block.class): the picture family `ltx:g`/`ltx:picture`/
  // `ltx:clippath`, plus `ltx:creator`/`ltx:equation`/`ltx:inline-item` (the full set
  // this clause flips — verified against LaTeXML.model). `<block>` is schema-invalid in
  // every one, and `<inline-block>` is valid there AND holds full Block.class content,
  // so a `\put`-positioned `\parbox`/`\makebox` digested there — an LR-box, never a
  // block — is renamed to a schema-valid `<inline-block>` IN PLACE: no move, so the
  // `\put` position in the enclosing `<g transform="translate(x,y)">` is preserved
  // (fidelity-safe). Emitting `<block>` here is invalid (e.g. `g_model`,
  // LaTeXML-picture.rng) — a shared Perl bug (`TeX_Box.pool.ltxml:493`), surpassed on
  // the schema-validity axis like #240. A `<para>`/`<inline-block>`/`<float>`/`<note>`
  // holds BOTH block and inline-block, so this clause is false there and the #240
  // block-climb (`context_tag == "ltx:para"`) stays untouched.
  let mut is_inline = is_svg
    || document::can_contain(&context, "#PCDATA")
    || (document::can_contain_qsym(context_tag, pin_static("ltx:inline-block"))
      && !document::can_contain_qsym(context_tag, pin_static("ltx:block")));
  let container_attr = block_attr.clone();
  let mut container = document.open_element("ltx:_CaptureBlock_", Some(container_attr), None)?;
  // the box's own element: the mark rides the capture's rename, an attribute-taking lone child, a paragraph-move; an
  // unwrapped capture leaves its material in the context, unmarked
  if boxed {
    document.mark_field_box(&mut container);
  }
  document.absorb(contents, None)?;

  // The row separators the panel grid rule records in a float's `{center}`-like capture (`note_panel_separator`) are
  // marks for the float's `arrange_panels`, not the box's content: they choose no container, are hoisted nowhere and
  // stay where they are — reaching the float where the capture unwraps into it, and leaving with a capture that stays
  // a container (`drop_panel_separators`), in which they separate none of the float's panels. As content, a leading
  // `\hspace*{5pt}`'s (or `\hskip`'s) `ltx:break` chose the inline candidate that holds it: the `\parbox` panels
  // after it were hoisted out as `logical-block`s and the caption kept in the capture's `ltx:block` (1710.06503 Fig 1,
  // the ws-procs961x669 binding net; repro captions-floats/an_hspace_star_opens_the_row_paragraph).
  let mut nodes = content_nodes(&container);
  nodes.retain(|node| !crate::latex_constructs::is_panel_separator(node));
  let mut node_tags = nodes
    .iter()
    .map(document::get_node_qname)
    .collect::<Vec<_>>();
  document.close_to_node(&container, true)?;
  document.close_node(&container)?;
  document.close_to_node(&context, true)?;

  // A sectioning unit inside a block-mode box (`\\section` in a `minipage[t]`, a
  // `framed`/`tcolorbox`) is a REAL section in LaTeX: `\\@startsection` ran, the
  // counter advanced, and the text after the box belongs to the new unit. Perl's
  // `insertBlock` (TeX_Box.pool.ltxml:512) instead wraps the box as
  // `<sectional-block>` and leaves it inside the enclosing section, where the
  // schema admits no `sectional-block` (only `document.body.class` does,
  // LaTeXML-structure.rnc:99) — SHARED, and invalid. User ruling (2026-09-20): the
  // box is presentation, the sectioning is semantics — never bend the schema to a
  // layout container. So when the captured content holds a sectioning unit and the
  // insertion context has an auto-closeable path to an ancestor that admits it, every
  // built unit is MOVED to where a live `\\section` would open (closing the enclosing
  // unit for a same-or-higher level, nesting a deeper one) and becomes the insertion
  // node, so trailing box content and post-box document text nest inside it; the
  // box's own annotation (`class="ltx_minipage"` and the frame attributes
  // `Sectional.attributes` admit — `width`/`vattach` are not, as `sectional-block`
  // already dropped them) rides the FIRST floated unit. A box inside a `<quote>`, a
  // list `<item>` or a `<figure>` has no such path (none auto-closes to a section
  // holder) and keeps today's Perl-parity shape. Witnesses: aguplus, fancyvrb-doc,
  // latex-via-exemplos, phonenumbers-{de,en}, recorder-fingering, tasks-manual,
  // unamth-template/tesis (8 s106 docs). OXIDIZED_DESIGN #250.
  if !is_inline
    && let Some(first_unit) = nodes.iter().find(|n| is_sectioning_unit(n))
    && sectioning_floatable(&context, document::get_node_qname(first_unit))
  {
    document.set_node(&context);
    // The floated units are what the caller gets back: `aligning_environment`
    // ({center}, {flushleft}, {flushright}) stamps its alignment on them.
    let mut floated: Vec<Node> = Vec::new();
    for child in nodes.iter().cloned() {
      if child.get_type() != Some(NodeType::ElementNode) {
        continue;
      }
      let q = document::get_node_qname(&child);
      let mut point = document.find_insertion_point_qsym(q, None)?;
      let mut c = child;
      c.unlink();
      point.add_child(&mut c)?;
      if is_sectioning_unit(&c) {
        floated.push(c.clone());
        document.set_node(&c);
      }
    }
    if let Some(mut unit) = floated.first().cloned() {
      let unit_q = document::get_node_qname(&unit);
      for (k, v) in block_attr.iter() {
        if k == "class" {
          document.add_class(&mut unit, v)?;
        } else if document::sym_can_have_attribute(unit_q, pin(k)) {
          document.set_attribute(&mut unit, k, v)?;
        }
      }
    }
    // Everything of substance has been moved out; the capture is spent.
    document.remove_node(container);
    return Ok(floated);
  }

  // A box that is clearly a figure becomes its float, holding its image (`absorb_figure_material`,
  // 57g); whether it then stands as that float or sits in a paragraph is settled when the
  // paragraph closes (`settle_float_paragraph`, latex_constructs).
  if let Some(float) = absorb_figure_material(document, &nodes)? {
    node_tags = vec![document::get_node_qname(&float)];
    nodes = vec![float];
  }
  // Paragraph material (`insert_block_in_paragraph`): where text would open an `ltx:p`, the box
  // goes into that paragraph, which stays open for the text after it. Decided after the capture,
  // so the sectioning float-out above (#250) still sees the original context. Not in a float,
  // whose content is panels (`in_panel_float`: arXiv 2605.27134's `\parbox` at a figure
  // minipage's top; captioned `\parbox` panels in a figure's `{center}` stay sub-figures); not
  // inside a restricted box (`\rotatebox{90}{\parbox…}`, arXiv 2605.20645), where the flag is
  // unset; not for an empty box, which leaves no material; not for content no inline box holds
  // (a bibliography, an index, a caption), which the hoist below places after the box as before.
  // A box whose whole content is one float goes into the paragraph too; a paragraph of nothing but
  // such boxes becomes those floats when it closes (user ruling 2026-09-27: `<figure
  // class="ltx_minipage">`, Perl's fold, for a lone one; a row of panels for several).
  if in_paragraph
    && !is_inline
    && !nodes.is_empty()
    && text_opens_paragraph(context_tag)
    && !in_panel_float(&context)
    && [
      "ltx:inline-block",
      "ltx:inline-logical-block",
      "ltx:inline-sectional-block",
    ]
    .iter()
    .any(|inline| {
      let inline = pin_static(inline);
      node_tags
        .iter()
        .all(|tag| document::sym_can_contain_somehow(inline, *tag).is_some())
    })
  {
    let mut paragraph = document.open_element("ltx:p", None, None)?;
    container.unlink();
    paragraph.add_child(&mut container)?;
    context = paragraph;
    context_tag = document::get_node_qname(&context);
    is_inline = true;
  }

  // Perl `insertBlock` (TeX_Box.pool.ltxml:406-472): the first candidate, in
  // preference order, that can hold ALL of the box's content becomes the
  // container (:457-458 give the candidate sets; :471 the hard `ltx:block`
  // fallback that then reports every child the model rejects). One general
  // rule extends it (Rust-only): when no candidate holds everything, the
  // first candidate that holds the box's LEADING content is the box, and the
  // content it cannot hold is placed after the box, in the nearest flow
  // ancestor that can hold it — the outcome an autoclosing frame produces
  // for the same content written live (`\printbibliography` in mdframed:
  // `perfect_kernel_gemini::mdframed_block_bibliography_juradiss`; a
  // `\caption` in a rotated `\parbox`: heria-proposal, rubik —
  // `perfect_kernel_batch56::caption_in_inline_parbox_floats_to_figure`).
  // The climb stays inside text containers (an ancestor holding paragraphs
  // or text): a drawing or a math box is a different medium, and content
  // hoisted past it would strand the rest of the box (Gemini J3,
  // biblatex-ext) — there the model's own verdict stands, as in Perl.
  // `LXML_TRACE_INSERT_BLOCK=1` prints each decision.
  let all_candidates = [
    "ltx:inline-block",
    "ltx:inline-logical-block",
    "ltx:inline-sectional-block",
    "ltx:block",
    "ltx:logical-block",
    "ltx:sectional-block",
    "ltx:figure",
  ]
  .map(pin_static);
  let candidate_set: &[SymStr] = if is_inline {
    &all_candidates[0..3]
  } else {
    &all_candidates
  };
  let holds = |c: SymStr, t: SymStr| document::sym_can_contain_somehow(c, t).is_some();
  // A box no candidate holds whole only because its paragraphs are wrapped in `ltx:para` — a caption beside a
  // `\noindent` paragraph (adjustbox's collectbox; `\caption{First} \noindent x` in a minipage, 2312.04535, 2209.02973,
  // 2404.07198, 2404.18532, 2406.00259, 2502.04073) — holds them unwrapped, the `ltx:p`s a candidate takes beside the
  // caption (the #244 demotion). Perl warns and falls back to an `ltx:block` the caption and the para are invalid in.
  let para = pin_static("ltx:para");
  if !candidate_set
    .iter()
    .any(|c| node_tags.iter().all(|t| holds(*c, *t)))
    && node_tags.contains(&para)
  {
    let unwrapped: Vec<SymStr> = nodes
      .iter()
      .flat_map(|n| {
        if document::get_node_qname(n) == para {
          content_nodes(n)
            .iter()
            .map(document::get_node_qname)
            .collect::<Vec<_>>()
        } else {
          vec![document::get_node_qname(n)]
        }
      })
      .collect();
    if candidate_set
      .iter()
      .any(|c| unwrapped.iter().all(|t| holds(*c, *t)))
    {
      for wrapper in nodes.iter().filter(|n| document::get_node_qname(n) == para) {
        // (what the paragraph carried goes to the elements it held, its id going with it: a text paragraph (`ltx:p`)
        // takes it as the para had it; a box (an adjustbox's `ltx:inline-block`, a picture) is centred as a box
        // (`ltx_centering`: an `align` on it centres only its own text), and the paragraph's indentation is not its)
        let attributes = wrapper.get_attributes();
        for mut child in wrapper.get_child_elements() {
          let tag = document::get_node_qname(&child);
          let text_block = tag == pin_static("ltx:p");
          for (key, value) in &attributes {
            if key == "id" || key == "xml:id" || key.starts_with('_') {
              continue;
            }
            if key == "class" {
              for class in value.split_whitespace() {
                if text_block || (class != "ltx_indent" && class != "ltx_noindent") {
                  document.add_class(&mut child, class)?;
                }
              }
            } else if key == "align" && !text_block {
              // (a box's other alignments are not carried: no class places a box right; a residual)
              if value == "center" {
                document.add_class(&mut child, "ltx_centering")?;
              }
            } else if !child.has_attribute(key) && document::sym_can_have_attribute(tag, pin(key)) {
              document.set_attribute(&mut child, key, value)?;
            }
          }
        }
        document.unwrap_nodes(wrapper.clone())?;
      }
      nodes = content_nodes(&container);
      node_tags = nodes.iter().map(document::get_node_qname).collect();
    }
  }
  let holds_all = |c: SymStr| node_tags.iter().all(|t| holds(c, *t));
  // Perl's order (TeX_Box.pool.ltxml:503-509): the first candidate that holds all the content and that the context
  // holds, else the first that holds all the content.
  let allowed = |c: SymStr| document::can_contain_qsym(context_tag, c);
  let fits_all = candidate_set
    .iter()
    .copied()
    .find(|c| holds_all(*c) && allowed(*c))
    .or_else(|| candidate_set.iter().copied().find(|c| holds_all(*c)));
  let container_tag = match fits_all {
    // One the context does not hold (a figure for a `\parbox` with a closing `\caption` in a `\rotatebox`'s
    // `ltx:inline-block`, LaTeXML-misc.rnc:28: heria.cls:737-745 `SidewaysFigure`, schema-invalid) gives way to one it
    // holds for the box's leading content, when what that one cannot hold is the caption closing the box and it has
    // a home after it ([`tail_home`]: the caption goes to the figure the box is in, read in the same order; 1810.08269).
    // A box that opens with its caption (`\rotatebox{90}{\begin{minipage}…\caption…\begin{tabular}…}`) keeps Perl's
    // choice: hoisting the caption would put it after the content it precedes, and an empty box would be left in the
    // transform; so does one whose closing part holds more than a caption, one beside other content in the
    // transform or a frame around it (`\scalebox{…}{minipage \hfill minipage}`, 2605.06364: the hoist moves the
    // insertion point, so the next panel would leave the transform), and one whose float holds another caption
    // ([`caption_hoist_is_safe`]: two captions would become one float).
    Some(c) if !allowed(c) => candidate_set
      .iter()
      .copied()
      .filter(|alt| allowed(*alt))
      .find(|alt| {
        let kept = node_tags.iter().take_while(|t| holds(*alt, **t)).count();
        let tail = &node_tags[kept..];
        kept > 0
          && !tail.is_empty()
          && tail
            .iter()
            .all(|t| !holds(*alt, *t) && with(*t, |q| q == "ltx:caption" || q == "ltx:toccaption"))
          && tail_home(
            &context,
            &container,
            tail,
            nodes[..kept]
              .iter()
              .all(|n| n.get_content().trim().is_empty()),
          )
          .is_some_and(|(_, home)| caption_hoist_is_safe(document, &context, &home))
      })
      .or(Some(c)),
    Some(c) => Some(c),
    None => node_tags
      .first()
      .and_then(|first| candidate_set.iter().copied().find(|c| holds(*c, *first))),
  };
  let hoisted: Vec<bool> = node_tags
    .iter()
    .map(|t| container_tag.is_none_or(|c| !holds(c, *t)))
    .collect();
  if std::env::var_os("LXML_TRACE_INSERT_BLOCK").is_some() {
    eprintln!(
      "insert_block: context={} inline={} tags={:?} container={:?} hoisted={:?}",
      with(context_tag, |s| s.to_string()),
      is_inline,
      node_tags
        .iter()
        .map(|t| with(*t, |s| s.to_string()))
        .collect::<Vec<_>>(),
      container_tag.map(|c| with(c, |s| s.to_string())),
      hoisted
    );
  }
  if hoisted.iter().any(|h| *h) {
    // The box's content in source order, for a figure panel that takes all of it (below).
    let in_order = nodes.clone();
    let order_tags = node_tags.clone();
    let mut tail = Vec::new();
    let mut kept = Vec::new();
    let mut kept_tags = Vec::new();
    for (n, (node, tag)) in nodes.drain(..).zip(node_tags.iter().copied()).enumerate() {
      if hoisted[n] {
        tail.push(node);
      } else {
        kept.push(node);
        kept_tags.push(tag);
      }
    }
    let tail_tags: Vec<SymStr> = tail.iter().map(document::get_node_qname).collect();
    // A text container: holds paragraphs, or text itself (`ltx:p`, an
    // inline block). A drawing (`ltx:picture`, `svg:g`) holds neither —
    // backmatter trapped in a drawing (xebaposter's References `\headerbox`
    // is a pgf node → `svg:foreignObject`) still floats to the enclosing flow
    // exactly as from a plain minipage, but never at the cost of stranding
    // TEXT: the climb crosses a drawing only when no kept node carries
    // non-whitespace text (a graphic in the box is drawing content and stays
    // where it is; Perl's `adjustBackmatterElement` cannot cross at all —
    // `ltx:bibliography` isn't allowed in `ltx:block`, SHARED; batch 56bq).
    let kept_empty = kept.iter().all(|n| n.get_content().trim().is_empty());
    let home = tail_home(&context, &container, &tail_tags, kept_empty);
    if let Some((anchor, ancestor)) = home {
      // `anchor` is an element and a box's content nodes are already
      // coalesced, so no two adjacent text nodes reach `add_next_sibling`
      // (libxml2 merges those and frees the second — see `replace_node`).
      let mut prev = anchor;
      for mut n in tail {
        n.unlink();
        prev.add_next_sibling(&mut n)?;
        prev = n;
      }
      document.set_node(&ancestor);
      nodes = kept;
      node_tags = kept_tags;
      if nodes.is_empty() {
        document.remove_node(container);
        return Ok(Vec::new());
      }
    } else if is_inline
      && order_tags
        .iter()
        .any(|t| with(*t, |s| s == "ltx:caption" || s == "ltx:toccaption"))
      && order_tags
        .iter()
        .all(|t| holds(pin_static("ltx:figure"), *t))
    {
      // A box in running text holding a caption, all of whose content a figure may hold — a minipage in a table
      // cell captioned as a figure, caption first or last: it becomes a figure panel in the box, its content in
      // source order, as the same minipage between paragraphs
      // becomes one (`figure class="ltx_figure_panel ltx_minipage"`), the box an inline logical block, whose model
      // holds a figure where a cell holds none (captions_in_minipages_in_a_tabular; Perl reports the captions in
      // an `ltx:block`).
      drop((kept, tail));
      if let Some(mut panel) = document.wrap_nodes("ltx:figure", in_order.clone())? {
        document.add_class(&mut panel, "ltx_figure_panel")?;
        nodes = vec![panel];
        node_tags = vec![pin_static("ltx:figure")];
      } else {
        // The model's verdict, as below (a box's own nodes always wrap).
        nodes = in_order;
        node_tags = order_tags;
      }
    } else {
      // No flow ancestor holds it: the content stays in the box and the
      // model reports it (Perl's outcome).
      nodes = kept;
      nodes.extend(tail);
      node_tags = kept_tags;
      node_tags.extend(tail_tags);
    }
  }
  let nnodes = nodes.len();

  // Perl: Hack: apparently TeX doesn't shift (vattach) a single node in a vbox/vtop/...
  #[allow(clippy::redundant_locals)]
  let mut block_attr = block_attr;
  let mut ignorable_attr = ignorable_attr;
  if nnodes == 1 && block_attr.contains_key("vattach") && is_v_attached(&nodes[0]) {
    container.remove_attribute("vattach")?;
    block_attr.remove("vattach");
    ignorable_attr = is_svg || block_attr.is_empty();
  }

  if nnodes < 1 {
    // Insertion came up empty?
    document.remove_node(container); // then remove the new block entirely
    return Ok(nodes);
  } else if ignorable_attr
    && node_tags
      .iter()
      .all(|tag| document::can_contain_qsym(context_tag, *tag))
  {
    // No attributes, contents allowed in context?
    document.unwrap_nodes(container)?; // No container needed, at all.
    return Ok(nodes);
  } else if nnodes == 1 {
    if document::can_contain_qsym(context_tag, node_tags[0])
      && (ignorable_attr
        || block_attr
          .keys()
          .all(|key| document::sym_can_have_attribute(node_tags[0], pin(key))))
    {
      // IF: Single node, allowed in context & accepts attributes
      // THEN: Add attributes and unwrap the single node
      //
      // SURPASS-PERL (OXIDIZED_DESIGN #125): `class` MERGES, it does not
      // overwrite. Perl's insertBlock (`TeX_Box.pool.ltxml` L492) uses
      // `setAttribute(class => …)`, so a wrapper's class (e.g. minipage's
      // `ltx_minipage`) clobbered the single child's own class — a `lstlisting`
      // that is the sole content of a `minipage`-in-`figure` became
      // `class="ltx_minipage"`, losing `ltx_lstlisting` and thus the
      // whitespace-preserving CSS, so its indentation collapsed
      // (html_feedback#6632, arXiv:2605.03143). LaTeXML already distinguishes
      // `addClass` from `setAttribute`; merging the child's class with the
      // wrapper's keeps both (`ltx_lstlisting ltx_minipage`).
      for (k, v) in block_attr.iter() {
        if k == "class" {
          document.add_class(&mut nodes[0], v)?;
        } else {
          document.set_attribute(&mut nodes[0], k, v)?;
        }
      }
      // the lone child takes the box's attributes, and so its mark (OXIDIZED_DESIGN #715)
      if container.has_attribute(document::FIELD_BOX_ATTRIBUTE) {
        document.mark_field_box(&mut nodes[0]);
      }
      document.unwrap_nodes(container)?;
      return Ok(nodes);
    } else if let Some(newcontainer) = document::sym_can_contain_somehow(context_tag, node_tags[0])
      && (ignorable_attr
        || block_attr.keys().all(|key| {
          newcontainer
            .map(|nc| document::sym_can_have_attribute(nc, pin(key)))
            .unwrap_or(false)
        }))
      && let Some(nc) = newcontainer
      // never an SVG element: with the foreignObject model widened to Flow
      // (OXIDIZED_DESIGN #186) a minipage in a TikZ node resolved here to
      // `svg:foreignObject` and the capture — class, vattach, `width` —
      // was renamed INTO a foreignObject, clobbering the real one's px
      // width (fixture tikz/various_colors); it belongs in the inline-block
      // the candidates below pick.
      && !with(nc, |s| s.starts_with("svg:"))
    {
      // rename the capture to that container
      drop_panel_separators(document, &container);
      document.rename_node_qsym(container, nc, true)?;
      return Ok(nodes);
    }
  }
  // This jagged conditional is a "code smell", due to the difficulty of refactoring
  // the in-conditional-assignments from Perl.

  // Otherwise, rename the capture
  drop_panel_separators(document, &container);
  // MAY need foreignObject wrapper
  if is_svg
    && node_tags
      .iter()
      .any(|tag| with(*tag, |tag_str| tag_str.starts_with("ltx:")))
  {
    context = document
      .wrap_nodes("svg:foreignObject", vec![container.clone()])?
      .expect("foreign object wrap should always succeed in SVG");
    context_tag = document::get_node_qname(&context);
  }
  let candidates = if is_inline {
    [
      "ltx:inline-block",
      "ltx:inline-logical-block",
      "ltx:inline-sectional-block",
    ]
    .map(pin_static)
    .to_vec()
  } else {
    [
      "ltx:block",
      "ltx:logical-block",
      "ltx:sectional-block",
      "ltx:figure",
    ]
    .map(pin_static)
    .to_vec()
  };
  let filtered_candidates = candidates
    .into_iter()
    .filter(|candidate| {
      node_tags
        .iter()
        .all(|tag| document::sym_can_contain_somehow(*candidate, *tag).is_some())
    })
    .collect::<Vec<_>>();
  // and are allowed in the context
  let allowed_candidates = filtered_candidates
    .iter()
    .filter(|candidate| document::can_contain_qsym(context_tag, **candidate))
    .copied()
    .collect::<Vec<_>>();
  // TODO: There is an arena code smell here. The `Model` interface needs to become lock-free
  // where Symbol tickets and &str are equally intuitive to use without runtime panics from
  // arena mutability exceptions.
  if let Some(final_tag) = allowed_candidates.first() {
    // The context can hold this block — rename the capture in place.
    document.rename_node(container, &to_string(*final_tag), true)?;
  } else if let Some(&final_tag) = filtered_candidates.first() {
    // Surpass (OXIDIZED_DESIGN #240): the context (e.g. an already-open `<para>`)
    // cannot hold this Para.class block, but an ancestor can. Climb to the
    // nearest such ancestor and move the capture there — ending the paragraph —
    // instead of renaming in place, which yields a schema-invalid
    // `<para>/<logical-block>` (LaTeXML-para.rnc:16,56). Perl renames in place
    // (TeX_Box.pool.ltxml:512-513, in insertBlock :449-519), a SHARED schema bug;
    // we keep the block
    // rendering (a `<div>`, LaTeXML-para-xhtml.xsl:53) — the inline variant would
    // wrongly nest block content (a float/table) in a `<span>`. Witnesses:
    // tikz-network, numerica (framed `{shaded}` / `{minipage}` after inline text).
    // Only reached on the currently-invalid path (`allowed_candidates` empty in a
    // non-inline context); the inline/`<p>` path always has a valid candidate.
    // Restrict the climb to a `<para>` context — the "block box ends the open
    // paragraph" case (tikz-network/numerica). Other block-holding contexts (an
    // SVG `foreignObject` / box that wraps its block in place) keep the pre-#4
    // in-place rename, so a `\parbox` inside a picture stays in its foreignObject.
    // A `<para>` only ever lives inside a Para.model container (item, section
    // body, logical-block, …), so the immediate parent holds a `logical-block`
    // and the climb stops one level up. A `sectional-block` (Para.class content
    // carrying a `\paragraph`/`\subsection`) instead ends any enclosing section
    // and lands at body level — valid (document.body.class holds it), a net
    // improvement over Perl's invalid in-place nesting.
    if with(context_tag, |t| t == "ltx:para") {
      let mut child = context.clone();
      while let Some(parent) = child.get_parent() {
        if !matches!(parent.get_type(), Some(NodeType::ElementNode)) {
          break;
        }
        if document::can_contain_qsym(document::get_node_qname(&parent), final_tag) {
          container.unlink();
          child.add_next_sibling(&mut container)?;
          document.set_node(&parent);
          break;
        }
        child = parent;
      }
    } else if document::can_contain_qsym(context_tag, pin_static("ltx:item"))
      && document::can_contain_indirect(context_tag, final_tag).is_some()
    {
      // A list container (`itemize`/`enumerate`/`description`, model `item*`) holds
      // no block, so a box opened in a list before its first `\item`, or between
      // items, lands where the model has no room for it. LaTeX typesets such a box
      // as list material (framed.sty's `shaded` around an `\item`, colorframed-doc
      // :176; a `minipage` of `\item`s beside a table minipage, tableaux/exemples
      // :71). The only schema-valid home is an item, which is autoOpen
      // (latex_constructs.pool.ltxml:1277): place the capture through
      // `find_insertion_point`, which opens `item` → `para` from the list exactly
      // as it does for text arriving before the first `\item` (Perl's
      // `computeIndirectModel` route). Perl's `insertBlock` renames in place
      // (TeX_Box.pool.ltxml:512), a SHARED schema error; the next `\item` closes the
      // auto-opened item. Only where such a route exists: a `sectional-block` has
      // none (no item holds it) and keeps Perl's in-place rename. OXIDIZED_DESIGN
      // #261. Guard
      // `perfect_kernel_batch56::block_in_a_list_before_an_item_gets_an_auto_item`.
      container.unlink();
      document.set_node(&context);
      let mut point = document.find_insertion_point_qsym(final_tag, None)?;
      point.add_child(&mut container)?;
      document.set_node(&point);
    }
    // If no ancestor can hold it (rare — Para.class always fits the section/body
    // model), the capture stays where it is. Perl renames in place to the invalid
    // Para.class element (TeX_Box.pool.ltxml:512); instead, if the current context
    // holds `<block>` but not `final_tag`, emit a schema-valid `<block>` and recursively
    // demote the Para.model content (surpass #244) — a box-forming construct
    // (`\begin{center}`/minipage/`\parbox`) whose body auto-opened a `<para>` when a
    // genuine block sat mid-content, placed in a Block.model container (titlepage,
    // quote, figure, abstract, inline-block). Demotion is IN PLACE (no reorder; render:
    // ltx_para/block/logical-block all `display:block`, only a redundant `<para>`
    // grouping + its auto-`xml:id` drop). The #240 `ltx:para` climb above already moved
    // the capture to an ancestor that holds `final_tag`, so this branch is false there
    // (parent holds `final_tag`) and that path is untouched. Witnesses: webquiz,
    // tabularcalc, short-math-guide, heria (+ the `caption_in_inline_parbox` sibling).
    let demote_para = !is_inline
      && container.get_parent().is_some_and(|parent| {
        let pq = document::get_node_qname(&parent);
        !document::can_contain_qsym(pq, final_tag)
          && document::can_contain_qsym(pq, pin_static("ltx:block"))
      })
      // Only demote when the result is provably valid: a container holding a
      // Para.model-only element `<block>` can't take (a float, a TOC, a sectioning
      // unit) stays at Perl-parity rather than acquiring a fresh `malformed` error.
      && subtree_is_block_demotable(&container);
    // A box (`ltx:inline-block`: `\resizebox`, `\scalebox`, `\rotatebox`) holds none of the
    // block containers but does hold the inline ones: a `{minipage}` there whose content cannot be
    // demoted (a float — tabularray's tall table, 57cp review —, a theorem, a `\section*`) is the
    // first inline container its parent holds that holds all of it (`inline-logical-block` holds
    // floats, theorems and paragraphs; `inline-sectional-block` sections). Perl renames it in place to
    // its first block candidate, `ltx:logical-block` (TeX_Box.pool.ltxml:502-513), invalid in the box
    // (divergence #385). Guard `perfect_kernel_batch57::minipage_blocks_in_a_scaled_box_are_inline`.
    let inline_home = (!is_inline && !demote_para)
      .then(|| container.get_parent())
      .flatten()
      .and_then(|parent| {
        let pq = document::get_node_qname(&parent);
        if document::can_contain_qsym(pq, final_tag) {
          return None;
        }
        [
          "ltx:inline-block",
          "ltx:inline-logical-block",
          "ltx:inline-sectional-block",
        ]
        .map(pin_static)
        .into_iter()
        .find(|c| {
          document::can_contain_qsym(pq, *c)
            && node_tags
              .iter()
              .all(|t| document::sym_can_contain_somehow(*c, *t).is_some())
        })
      });
    if demote_para {
      // Bottom-up: demote the Para.class content while the container still allows it,
      // THEN rename the container to <block> — so nothing is ever moved into a <block>
      // while still Para.class (which would emit a spurious `malformed` Error).
      demote_para_class_content(document, &container)?;
      document.rename_node(container, "ltx:block", true)?;
    } else if let Some(inline) = inline_home {
      document.rename_node_qsym(container, inline, true)?;
    } else {
      document.rename_node(container, &to_string(final_tag), true)?;
    }
  } else {
    // we didn't know what to do?
    let message = with(context_tag, |ctxt_str| {
      s!(
        "Did not find a block-like candidate in {} (with attributes ({})",
        ctxt_str,
        block_attr
          .iter()
          .map(|(k, v)| s!("{k}={v}"))
          .collect::<Vec<_>>()
          .join(";")
      )
    });
    Warn!("malformed", "_CaptureBlock_", message);
    document.rename_node(container, "ltx:block", true)?;
  }
  Ok(nodes)
}

/// The row separators a capture that stays a container holds (`insert_block_as`): inside it they separate none of the
/// float's panels, and a container the model chose without them may not hold an `ltx:break`.
fn drop_panel_separators(document: &mut Document, container: &Node) {
  for child in container.get_child_elements() {
    if crate::latex_constructs::is_panel_separator(&child) {
      document.remove_node(child);
    }
  }
}
