//! Cover-title and field ink: the marked ink of `\title`/`\author` lifted into the front matter.

use super::*;

/// A `{titlepage}` built AFTER body content began cannot be the schema's leading
/// `ltx:titlepage` (`document_model` admits it only in the front group): a leaked
/// preamble argument, an undefined-command marker or genuine prose before it has
/// already opened the body, so the element stands stranded — `element "titlepage"
/// not allowed here` — in both engines (Perl's `{titlepage}` is the same
/// `<ltx:titlepage>#body` constructor, latex_constructs.pool.ltxml:1167, with only an
/// Info: "Frontmatter will not be well-structured"; verified on the minimal repro:
/// identical output, one jing error). Its content is hand-typeset layout, so it is
/// demoted in place to an `ltx:para` carrying `class="ltx_titlepage"` — the same
/// text in the same order, no wrapper the schema forbids (OXIDIZED_DESIGN #257;
/// chemexec_de/en, stanli, l2picfaq, pst-calendar-doc, classicthesis in sweep s110).
/// A titlepage in its proper leading position, or one whose children `ltx:para`
/// cannot hold (a `\maketitle` unwound inside it), is left alone.
pub fn demote_stranded_titlepage(document: &mut Document) -> Result<()> {
  let Some(root) = document.get_document().get_root_element() else {
    return Ok(());
  };
  let kids: Vec<Node> = root
    .get_child_nodes()
    .into_iter()
    .filter(|n| n.get_type() == Some(NodeType::ElementNode))
    .collect();
  let Some(pos) = kids
    .iter()
    .rposition(|n| document::get_node_qname(n) == pin_static("ltx:titlepage"))
  else {
    return Ok(());
  };
  let stranded = kids[..pos].iter().any(|k| {
    document::get_node_qname(k) != pin_static("ltx:resource")
      && !is_frontmatter_group_element(k)
      && !node_is_content_free(k)
  });
  if !stranded {
    return Ok(());
  }
  let titlepage = kids[pos].clone();
  // The same child normalization `wrap_as_titlepage` applies: a `center`'s
  // `logical-block` and a `para` become `block`s (#244's demotion), which
  // `ltx:para` — like `ltx:titlepage` — admits. Only when every such child
  // SURVIVES the demotion (`subtree_is_block_demotable`: no float/figure
  // inside — stanli's cover holds `\begin{figure}`s in its `center`); else the
  // titlepage is left as Perl leaves it, nothing touched.
  let children: Vec<Node> = titlepage
    .get_child_nodes()
    .into_iter()
    .filter(|c| c.get_type() == Some(NodeType::ElementNode))
    .collect();
  // Decide BEFORE mutating: every layout child must survive the demotion, and
  // every child — as it will be after it (`logical-block`/`para` → `block`) —
  // must be something `ltx:para` holds. Otherwise nothing is touched.
  let layout = |q: SymStr| q == pin_static("ltx:logical-block") || q == pin_static("ltx:para");
  let demotable = children
    .iter()
    .all(|c| !layout(document::get_node_qname(c)) || subtree_is_block_demotable(c));
  let holdable = children.iter().all(|c| {
    let q = document::get_node_qname(c);
    layout(q) || document::can_contain_qsym(pin_static("ltx:para"), q)
  });
  if !(demotable && holdable) {
    return Ok(());
  }
  // A demoted cover no longer stands in for the document's `<ltx:title>`, which the HTML then displays: the class's
  // own printing `\title` run on it would be a second copy (OXIDIZED_DESIGN #629).
  drop_cover_title_ink(document, &titlepage);
  normalize_titlepage_layout_children(document, &titlepage)?;
  let mut para = document.rename_node(titlepage, "ltx:para", true)?;
  document.add_class(&mut para, "ltx_titlepage")?;
  Ok(())
}

/// The marks around the class's own printing `\title` run on a cover (sect05.rs `\lx@cover@title@begin`/`@end`,
/// `titlepage_class_run`): empty internal elements, admitted anywhere by the model (`_Capture_`-suffixed), never in the
/// output (removed here; a stray one the core finalize drops).
pub const COVER_TITLE_BEGIN: &str = "ltx:_CoverTitleBegin_Capture_";
/// See [`COVER_TITLE_BEGIN`].
pub const COVER_TITLE_END: &str = "ltx:_CoverTitleEnd_Capture_";

/// The class's own printing `\title` run on a cover inside `scope` (sect05.rs `titlepage_class_run`): everything that
/// lies between each pair of cover-title marks in document order — elements and bare text (`\def\title#1{#1}` in the
/// cover's open paragraph) — removed with the marks, and the wrappers it leaves empty (a `center`'s paragraph). Called
/// where a `{titlepage}` stops standing in for the document's `<ltx:title>`, so the HTML displays that title
/// (LaTeXML-structure-xhtml.xsl:374-375 hides it only beside an `ltx:titlepage`): `\unwind@titlepage` (a `\maketitle`
/// inside the cover), [`demote_stranded_titlepage`], and [`settle_cover_title_ink`]. An unmatched mark is removed
/// alone. OXIDIZED_DESIGN #629.
pub fn drop_cover_title_ink(document: &mut Document, scope: &Node) {
  let begin_q = pin(COVER_TITLE_BEGIN);
  let marks = cover_title_marks(document, scope);
  if marks.is_empty() {
    return;
  }
  let mut doomed: Vec<Node> = Vec::new();
  let mut chains: Vec<Vec<Node>> = Vec::new();
  let mut open: Option<Node> = None;
  for mark in marks {
    chains.push(ancestors_below(&mark, scope));
    if document::get_node_qname(&mark) == begin_q {
      if let Some(stray) = open.replace(mark) {
        doomed.push(stray);
      }
    } else if let Some(begin) = open.take() {
      doomed.extend(marked_range(scope, &begin, &mark));
      doomed.push(begin);
      doomed.push(mark);
    } else {
      doomed.push(mark);
    }
  }
  doomed.extend(open);
  remove_marked_ink(document, doomed, chains, |_| true);
}

/// Remove the `doomed` nodes (marks and the ink between them), then the marks' ancestors (`chains`) the removal leaves
/// with no content (a `center`'s paragraph holding only the print), innermost first, until none more empties — each a
/// wrapper `prunable` admits. Everything is decided before anything is removed: `remove_node` frees the subtree it
/// takes out (`discard_subtree`), so no handle into it may be used afterwards.
fn remove_marked_ink(
  document: &mut Document,
  doomed: Vec<Node>,
  chains: Vec<Vec<Node>>,
  prunable: impl Fn(&Node) -> bool,
) {
  // the nodes that go, by identity (`to_hashable`: a `Node` is no hash key, its handle is mutable)
  let mut gone: HashSet<usize> = doomed.iter().map(Node::to_hashable).collect();
  // is `node` or one of its ancestors going?
  let goes_with = |node: &Node, gone: &HashSet<usize>| {
    let mut n = Some(node.clone());
    while let Some(cur) = n {
      if gone.contains(&cur.to_hashable()) {
        return true;
      }
      n = cur.get_parent();
    }
    false
  };
  // an ancestor of a mark inside another range goes with that range
  let mut wrappers: Vec<Node> = Vec::new();
  for node in chains.into_iter().flatten() {
    if !goes_with(&node, &gone) && prunable(&node) && !wrappers.contains(&node) {
      wrappers.push(node);
    }
  }
  loop {
    let emptied: Vec<Node> = wrappers
      .iter()
      .filter(|w| !gone.contains(&w.to_hashable()))
      .filter(|w| {
        w.get_child_nodes().iter().all(|c| {
          gone.contains(&c.to_hashable())
            || (c.get_type() != Some(NodeType::ElementNode) && c.get_content().trim().is_empty())
        })
      })
      .cloned()
      .collect();
    if emptied.is_empty() {
      break;
    }
    gone.extend(emptied.iter().map(Node::to_hashable));
  }
  // each removed whole, once: the outermost of the nodes that go (no ancestor of it goes), in a stable order
  let mut victims: Vec<Node> = Vec::new();
  for n in doomed.iter().chain(wrappers.iter()) {
    if gone.contains(&n.to_hashable())
      && !n.get_parent().is_some_and(|p| goes_with(&p, &gone))
      && !victims.contains(n)
    {
      victims.push(n.clone());
    }
  }
  for victim in victims {
    document.remove_node(victim);
  }
}

/// The cover-title marks inside `scope`, in document order.
fn cover_title_marks(document: &mut Document, scope: &Node) -> Vec<Node> {
  document.findnodes(
    "descendant::ltx:_CoverTitleBegin_Capture_ | descendant::ltx:_CoverTitleEnd_Capture_",
    Some(scope),
  )
}

/// `node`'s ancestors strictly below `scope`, innermost first.
fn ancestors_below(node: &Node, scope: &Node) -> Vec<Node> {
  let mut out = Vec::new();
  let mut p = node.get_parent();
  while let Some(n) = p {
    if &n == scope {
      break;
    }
    p = n.get_parent();
    out.push(n);
  }
  out
}

/// The nodes after `begin` and before `end` in document order, each taken whole: the following siblings of `begin`
/// and of each of its ancestors up to the one on the path to `end`, then the children before that path down to `end`.
fn marked_range(scope: &Node, begin: &Node, end: &Node) -> Vec<Node> {
  let mut end_path = ancestors_below(end, scope);
  end_path.push(end.clone());
  let mut range: Vec<Node> = Vec::new();
  let mut cur = begin.clone();
  let mut fork: Option<Node> = None;
  'up: loop {
    let mut sib = cur.get_next_sibling();
    while let Some(s) = sib {
      if end_path.contains(&s) {
        fork = Some(s);
        break 'up;
      }
      sib = s.get_next_sibling();
      range.push(s);
    }
    match cur.get_parent() {
      Some(p) if &p != scope => cur = p,
      _ => break,
    }
  }
  while let Some(node) = fork.take() {
    if &node == end {
      break;
    }
    for child in node.get_child_nodes() {
      if end_path.contains(&child) {
        fork = Some(child);
        break;
      }
      range.push(child);
    }
  }
  range
}

/// At `\end{document}`, the cover-title marks still in the tree (OXIDIZED_DESIGN #629), by the XSLT's own test
/// (LaTeXML-structure-xhtml.xsl:374-375): while the document has a top-level `ltx:titlepage`, its `<ltx:title>` is not
/// displayed and the class's print is the title the page shows — kept, its marks removed, wherever the cover's
/// typesetting put it (a `\vbox` print the absorber set beside an emptied cover: hep-th0010274); without one (a
/// titlepage nested in a section or a box, every cover unwound or demoted) the title is displayed and the print goes
/// ([`drop_cover_title_ink`]).
pub fn settle_cover_title_ink(document: &mut Document) {
  let Some(root) = document.get_document().get_root_element() else {
    return;
  };
  if document
    .findnode("/ltx:document/ltx:titlepage", None)
    .is_some()
  {
    for mark in cover_title_marks(document, &root) {
      document.remove_node(mark);
    }
  } else {
    drop_cover_title_ink(document, &root);
  }
}

/// The field runs' marks (core `Document::absorb`, [`document::FIELD_MARK_TARGET`]): processing instructions
/// `<?ltx-field-mark begin <field> <id>?>` … `<?ltx-field-mark end <field> <id> <min> <max>?>` around a run of ink
/// digested from a frontmatter field's stored tokens (their out-of-band provenance, `\lx@field@mark`). Not elements: a
/// processing instruction is no node of content (`content_nodes`, text content), so no block is built around it.
/// Never in the output: removed by [`lift_marked_field_ink`] and [`settle_field_marks`], and by the core finalize.
pub const FIELD_MARK_TARGET: &str = document::FIELD_MARK_TARGET;
/// The fields the kernel setters stamp, by their provenance id (`Token::prov`'s field).
const FIELD_MARK_FIELDS: &[(&str, u16)] = &[("title", 1), ("author", 2)];

/// The provenance id of a stamped field (`title`, `author`).
pub(in crate::base_utilities) fn field_id(field: &str) -> Option<u16> {
  FIELD_MARK_FIELDS
    .iter()
    .find(|(name, _)| *name == field)
    .map(|(_, id)| *id)
}

/// One run of a field's ink, from its two marks ([`FIELD_MARK_TARGET`]).
pub(in crate::base_utilities) struct FieldInkRun {
  field:       u16,
  begin:       Node,
  end:         Node,
  /// the field-ledger serials of the boxes it absorbed, inclusive intervals (core `token::field_ledger_cover`), and
  /// of those an alignment took between its parts (a joined run's cells' `\\`)
  serials:     Vec<(u32, u32)>,
  /// the positions of the current call's value (`lx_field_range_<id>`) the run consumed: sorted disjoint
  /// `start..end` intervals
  cover:       Vec<(u16, u16)>,
  /// does the run consume positions of the current call's value only? A print of an earlier call's value (`\title{A}`
  /// printed, then `\title{B}`; an `\edef` copy) is joined with no other run (re-review finding 2, probes r12, r13).
  in_range:    bool,
  /// is the run preceded, in the document's content order, by body text ([`field_runs_after_body_text`])? Then it
  /// is not in the document's title block.
  body_before: bool,
  /// does the run hold a box of one character of the value twice? A print digests each character once, so it read
  /// the value again from an earlier position the way a macro's expansion does (all its boxes at the macro's
  /// position): `\StrLeft{\@title}{1}\@title`'s two `T`s. No whole print.
  reread:      bool,
}

impl FieldInkRun {
  fn min_serial(&self) -> u32 {
    self
      .serials
      .iter()
      .map(|&(first, _)| first)
      .min()
      .unwrap_or(0)
  }
  fn max_serial(&self) -> u32 {
    self
      .serials
      .iter()
      .map(|&(_, last)| last)
      .max()
      .unwrap_or(0)
  }
}

/// The `start:end` intervals of a field's extent (`lx_field_extent_<id>`), a call's range (`lx_field_range_<id>`) or a
/// run's serials (its end mark).
fn parse_intervals<T: FromStr>(text: &str) -> Vec<(T, T)> {
  text
    .split(',')
    .filter_map(|iv| {
      let (start, end) = iv.split_once(':')?;
      Some((start.trim().parse().ok()?, end.trim().parse().ok()?))
    })
    .collect()
}

/// The positions of field `field`'s stored value a whole print covers (`\lx@field@mark`'s extent).
fn field_extent(field: u16) -> Vec<(u16, u16)> {
  parse_intervals(&lookup_string(&s!("lx_field_extent_{field}")))
}

/// The positions of the current call's value of field `field` (`\lx@field@mark`'s range): `start..end`.
fn field_range(field: u16) -> Option<(u16, u16)> {
  parse_intervals(&lookup_string(&s!("lx_field_range_{field}")))
    .first()
    .copied()
}

/// `cover` within `range` (sorted disjoint intervals stay so).
fn clip_cover(cover: &[(u16, u16)], range: Option<(u16, u16)>) -> Vec<(u16, u16)> {
  let Some((start, end)) = range else {
    return Vec::new();
  };
  cover
    .iter()
    .filter_map(|&(s, e)| {
      let (s, e) = (s.max(start), e.min(end));
      (s < e).then_some((s, e))
    })
    .collect()
}

/// Does `cover` cover every position of `extent`, so the run is a whole print of the field (review finding 2: every
/// position, a macro's too, not the first and the last character)? Both are sorted disjoint intervals, the cover's
/// merged where they touch, so each extent interval lies in one cover interval.
fn covers_extent(cover: &[(u16, u16)], extent: &[(u16, u16)]) -> bool {
  !extent.is_empty()
    && extent
      .iter()
      .all(|&(s, e)| cover.iter().any(|&(cs, ce)| cs <= s && e <= ce))
}

/// Is `node` inside invisible ink (`\phantom`, `\vphantom`, `\hphantom` in text: `<ltx:text class="ltx_phantom">`)?
fn in_phantom(node: &Node) -> bool {
  let mut p = node.get_parent();
  while let Some(n) = p {
    if n
      .get_attribute("class")
      .is_some_and(|c| c.split_whitespace().any(|c| c == "ltx_phantom"))
    {
      return true;
    }
    p = n.get_parent();
  }
  false
}

/// Do the boxes with serials `serials` (inclusive intervals) hold a box of one character of field `field`'s value
/// (`lx_field_chars_<id>`) twice?
fn rereads_a_char(field: u16, serials: &[(u32, u32)]) -> bool {
  let chars: Vec<(u16, u16)> = parse_intervals(&lookup_string(&s!("lx_field_chars_{field}")));
  let mut seen: HashSet<u16> = HashSet::default();
  for &(first, last) in serials {
    for serial in first..=last {
      if let Some(entry) = field_ledger_entry(serial)
        && entry.cover.field == field
        && entry.cover.end == entry.cover.start + 1
        && chars
          .iter()
          .any(|&(s, e)| s <= entry.cover.start && entry.cover.start < e)
        && !seen.insert(entry.cover.start)
      {
        return true;
      }
    }
  }
  false
}

/// The serials of the field-ledger entries of field `field` an alignment took with serials `from..to` (exclusive), as
/// inclusive intervals (core `token::field_ledger_take`).
fn taken_serials(field: u16, from: u32, to: u32) -> Vec<(u32, u32)> {
  (from..to)
    .filter(|&serial| field_ledger_entry(serial).is_some_and(|e| e.taken && e.cover.field == field))
    .map(|serial| (serial, serial))
    .collect()
}

/// The field runs inside `scope`, each with its own two marks (paired by the run's id, so never with another print's),
/// in document order of their begin marks; and every mark node. A run's cover is what the boxes it absorbed consumed
/// of the current call's value (and what an alignment took between and right after them): never what a box absorbed
/// elsewhere (in another run, in a formula) or never absorbed (an unused `\setbox`/`\sbox`/`{lrbox}`) consumed
/// (re-review finding 1, probes r19, r20). A run set invisibly (`\phantom`) is no print.
pub(in crate::base_utilities) fn field_ink_runs(
  document: &mut Document,
  scope: &Node,
) -> (Vec<FieldInkRun>, Vec<Node>) {
  let marks = field_marks(document, scope);
  if marks.is_empty() {
    return (Vec::new(), marks);
  }
  let body_before = field_runs_after_body_text(scope);
  let mut begins: Vec<(u32, u16, usize)> = Vec::new();
  let mut runs: Vec<(usize, FieldInkRun)> = Vec::new();
  for (at, mark) in marks.iter().enumerate() {
    let content = mark.get_content();
    let parts: Vec<&str> = content.split_whitespace().collect();
    match parts.as_slice() {
      ["begin", field, id] => {
        if let (Ok(field), Ok(id)) = (field.parse(), id.parse()) {
          begins.push((id, field, at));
        }
      },
      ["end", field, id, serials] => {
        let (Ok(field), Ok(id)) = (field.parse::<u16>(), id.parse::<u32>()) else {
          continue;
        };
        let serials: Vec<(u32, u32)> = parse_intervals(serials);
        if let Some(i) = begins
          .iter()
          .position(|(bid, bfield, _)| *bid == id && *bfield == field)
        {
          let (_, _, begin_at) = begins.swap_remove(i);
          if in_phantom(&marks[begin_at]) {
            continue;
          }
          let range = field_range(field);
          let cover = field_ledger_cover(field, &serials);
          let in_range =
            range.is_some_and(|(start, end)| cover.iter().all(|&(s, e)| start <= s && e <= end));
          let reread = rereads_a_char(field, &serials);
          runs.push((begin_at, FieldInkRun {
            field,
            begin: marks[begin_at].clone(),
            end: mark.clone(),
            reread,
            serials,
            cover: clip_cover(&cover, range),
            in_range,
            body_before: body_before.contains(&id),
          }));
        }
      },
      _ => {},
    }
  }
  runs.sort_by_key(|(at, _)| *at);
  // One print split by the structure it is set in (a cover `tabular`'s cells, `\and`'s `\end{tabular}…
  // \begin{tabular}`): a run of a field that goes on with the field's later positions, with no ink between, is one run
  // with the one before when, together with what the alignment took between them (the cells' `\\`, its row-end
  // marker never absorbed, taken once the alignment is shown, core `document::field_ink_taken_serials`), they cover one contiguous stretch of the field's extent.
  // Both must be prints of the current call's value. A second print starts over, and two slices of the value with a
  // gap between (`\StrLeft…\par\StrRight…`, nothing of the gap consumed) stay two runs, neither whole (review finding
  // 2).
  let mut merged: Vec<FieldInkRun> = Vec::new();
  for (_, run) in runs {
    if let Some(last) = merged.last_mut()
      && last.field == run.field
      && last.in_range
      && run.in_range
      && !last.reread
      && !run.reread
      && run.min_serial() > last.max_serial()
      && let (Some(&(_, last_end)), Some(&(run_start, _))) = (last.cover.last(), run.cover.first())
      && run_start >= last_end
      && marked_range(scope, &last.end, &run.begin)
        .iter()
        .all(is_inkless)
    {
      let mut serials = last.serials.clone();
      serials.extend(taken_serials(
        run.field,
        last.max_serial() + 1,
        run.min_serial(),
      ));
      serials.extend(run.serials.iter().copied());
      let joined = clip_cover(
        &field_ledger_cover(run.field, &serials),
        field_range(run.field),
      );
      let extent = field_extent(run.field);
      // every extent position from where the first stopped to where the second starts is consumed between them
      let contiguous = extent
        .iter()
        .flat_map(|&(s, e)| s.max(last_end)..e.min(run_start))
        .all(|p| joined.iter().any(|&(cs, ce)| cs <= p && p < ce));
      if contiguous {
        last.end = run.end;
        last.serials = serials;
        last.cover = joined;
        continue;
      }
    }
    merged.push(run);
  }
  // What the alignment took right after a run's last box (a value's closing `\\`, its row-end marker absorbed in no
  // run, 1605.00182) is the run's too: the ledger's next entries of the field an alignment took, while they continue
  // its cover (no position of the extent skipped), past entries that cover no position of the extent (the space the
  // cell dropped before it). Nothing else: an entry absorbed elsewhere or never absorbed is no part of this print.
  for run in &mut merged {
    let Some(&(_, mut end)) = run.cover.last() else {
      continue;
    };
    let extent = field_extent(run.field);
    let limit = field_range(run.field).map_or(end, |(_, e)| e);
    let mut serial = run.max_serial() + 1;
    while let Some(next) = field_ledger_entry(serial)
      && next.cover.field == run.field
    {
      let continues =
        next.cover.start <= end || !extent.iter().any(|&(s, e)| s < next.cover.start && e > end);
      if next.taken && continues {
        end = end.max(next.cover.end).min(limit);
      } else if next.taken
        || extent
          .iter()
          .any(|&(s, e)| s < next.cover.end && next.cover.start < e)
      {
        break;
      }
      // (an entry of no extent position the alignment dropped, the space before a cell's end, is stepped over: it
      // adds nothing to the cover)
      serial += 1;
    }
    if let Some(last) = run.cover.last_mut() {
      last.1 = last.1.max(end);
    }
  }
  (merged, marks)
}

/// The ids of the field runs inside `scope` that body text precedes in the document's content order (re-review finding
/// 3: a letter that ends `\noindent\@author` signs it; pdflatex prints the name there and only there). Body text is
/// any text of the document before the run's begin mark, the text before it in the run's own block too, except:
/// - a field's ink (between a run's two marks: a title before its author);
/// - what is set apart from the page's flow: a frontmatter element ([`FRONTMATTER_ELEMENTS`]), a note, a float, an
///   error's markup, invisible ink (`\phantom`), the internal `_Capture_` marks;
/// - the text of a TeX box that does not hold the run: a cover's header box (a preprint-number `\parbox`, 0706.0979,
///   1605.00182; `\rightline{LA-UR-…}`, `\makebox[\textwidth][r]{…}`, `\mbox{preprint}`, `\colorbox`,
///   `\fcolorbox`, `\scalebox`) is no body. A box is decided by the TeX mechanism, not by the element it builds
///   (review r4 finding 3), and marks only the element its own constructor opened, at the point it opens it (review r5
///   finding 1; core `Document::mark_field_box`'s `_fieldbox`): a constructor declared `restricted_horizontal`
///   outside math (`\hbox`, so `\rightline`/`\leftline`/`\centerline` and `\colorbox`; `\mbox`, `\makebox`,
///   `\raisebox`, `\fcolorbox`, `\scalebox`/`\resizebox`/`\rotatebox`/`\reflectbox`), `\framebox`/`\fbox`, the
///   capture of a `\vbox`, `\vtop`, `\parbox`, minipage, `p` column or framed box ([`insert_block`]), an
///   environment whose begin code opens a box group, an alignment's container. Material a `\vbox` absorbs straight
///   into its context marks nothing.
///
/// A list environment is no box: text in `{flushright}`, `{center}`, `quote` before the print is body text, and the
/// print stays (the header cases OXIDIZED_DESIGN #715 lists).
fn field_runs_after_body_text(scope: &Node) -> HashSet<u32> {
  /// Is `node` set apart from the page's flow, its text no body of the document?
  fn set_apart(node: &Node) -> bool {
    node.get_name().ends_with("_Capture_")
      || node
        .get_attribute("class")
        .is_some_and(|c| c.split_whitespace().any(|c| c == "ltx_phantom"))
      || with(document::get_node_qname(node), |q| {
        FRONTMATTER_ELEMENTS.contains(&q)
          || matches!(
            q,
            "ltx:note" | "ltx:figure" | "ltx:table" | "ltx:float" | "ltx:ERROR" | "ltx:indexmark"
          )
      })
  }
  /// Was `node` opened by a TeX box's constructor (core `Document::mark_field_box`): its text is no body for a run
  /// outside it?
  fn is_box(node: &Node) -> bool { node.has_attribute(document::FIELD_BOX_ATTRIBUTE) }
  #[derive(Default)]
  struct Walk {
    /// the ids of the runs open at this point of the walk (their marks seen, the end not yet)
    open:  Vec<u32>,
    /// body text seen outside every box
    body:  bool,
    /// the boxes around this point, innermost last, each with whether body text was seen in it
    boxes: Vec<bool>,
    /// the runs body text precedes
    after: HashSet<u32>,
  }
  fn walk(node: &Node, w: &mut Walk) {
    for child in node.get_child_nodes() {
      match child.get_type() {
        Some(NodeType::ElementNode) => {
          if set_apart(&child) {
            continue;
          }
          let boxed = is_box(&child);
          if boxed {
            w.boxes.push(false);
          }
          walk(&child, w);
          if boxed {
            w.boxes.pop();
          }
        },
        Some(NodeType::TextNode) => {
          if w.open.is_empty() && !child.get_content().trim().is_empty() {
            match w.boxes.last_mut() {
              Some(seen) => *seen = true,
              None => w.body = true,
            }
          }
        },
        Some(NodeType::PiNode) if child.get_name() == FIELD_MARK_TARGET => {
          let content = child.get_content();
          match content.split_whitespace().collect::<Vec<_>>().as_slice() {
            ["begin", _, id, ..] => {
              if let Ok(id) = id.parse::<u32>() {
                if w.body || w.boxes.iter().any(|seen| *seen) {
                  w.after.insert(id);
                }
                w.open.push(id);
              }
            },
            ["end", _, id, ..] => {
              if let Ok(id) = id.parse::<u32>() {
                w.open.retain(|open| *open != id);
              }
            },
            _ => {},
          }
        },
        _ => {},
      }
    }
  }
  let mut w = Walk::default();
  walk(scope, &mut w);
  w.after
}

/// Does `node` hold no ink: whitespace, a mark, a line break, or elements holding only those?
fn is_inkless(node: &Node) -> bool {
  match node.get_type() {
    Some(NodeType::ElementNode) => {
      document::get_node_qname(node) == pin_static("ltx:break")
        || (node.get_first_child().is_some() && node.get_child_nodes().iter().all(is_inkless))
    },
    Some(NodeType::TextNode) => node.get_content().trim().is_empty(),
    _ => true,
  }
}

/// The field marks inside `scope`, in document order.
fn field_marks(document: &mut Document, scope: &Node) -> Vec<Node> {
  document.findnodes(
    &s!("descendant::processing-instruction('{FIELD_MARK_TARGET}')"),
    Some(scope),
  )
}

/// The elements a printed field's ink may sit in to be the document's title block: paragraphs and their wrappers
/// (`\begin{center}`, a cover minipage, a font group, `\emph`), and a cover's `tabular` (`\begin{tabular}[t]{c}\@author
/// \end{tabular}`). Not a float, a note, a list, a theorem, a sectional unit, a `{titlepage}` or a frontmatter element:
/// what is set there is the author's content (or the frontmatter itself) even when it prints the field.
fn is_field_ink_wrapper(q: SymStr) -> bool {
  with(q, |q| {
    matches!(
      q,
      "ltx:para"
        | "ltx:p"
        | "ltx:logical-block"
        | "ltx:inline-logical-block"
        | "ltx:block"
        | "ltx:inline-block"
        | "ltx:inline-para"
        | "ltx:text"
        | "ltx:emph"
        | "ltx:tabular"
        | "ltx:thead"
        | "ltx:tbody"
        | "ltx:tfoot"
        | "ltx:tr"
        | "ltx:td"
    )
  })
}

/// Does `node` hold content besides the nodes in `gone`, empty internal marks (`_Capture_`) and line breaks? A
/// processing instruction (a field mark) is none.
fn holds_content_besides(node: &Node, gone: &[Node]) -> bool {
  node.get_child_nodes().iter().any(|c| {
    if gone.contains(c) {
      return false;
    }
    match c.get_type() {
      Some(NodeType::ElementNode) => {
        if c.get_name().ends_with("_Capture_") && c.get_first_element_child().is_none() {
          return false;
        }
        // a line's break (`\centerline`'s, a `\\` after the print) ends the line, it holds nothing
        if document::get_node_qname(c) == pin_static("ltx:break") {
          return false;
        }
        // an element with no children is content of its own (a `<break/>`, a graphic, a rule)
        c.get_first_child().is_none() || holds_content_besides(c, gone)
      },
      Some(NodeType::TextNode) => !c.get_content().trim().is_empty(),
      _ => false,
    }
  })
}

/// The printed ink of each field the frontmatter displays, removed — the field's print by construction: the ink
/// digested from the field's stored tokens (their out-of-band provenance, `\lx@field@mark`; the title-print study's
/// rule R3, frontmatter model M3 stage S1, OXIDIZED_DESIGN #715), one run of it (`field_ink_runs`) at a time, with the
/// title-block wrappers it leaves empty (not a `\label` target, which is the author's own). Called once, where the
/// frontmatter is placed (the `\maketitle` flush `\lx@frontmatterhere`, the fallback at the first sectional unit or
/// `\end{document}`), so only what was printed before it is the field's ink; every mark goes, and later prints are not
/// marked (`lx_field_ink_settled`, `Document::settle_field_ink`). A run is lifted only:
/// - when it is the whole field: what its own boxes consumed of the current call's value covers every position of
///   the field's extent (`lx_field_extent_<id>`, `covers_extent`), a macro's too, so a fragment the document takes of
///   it (`\StrLeft{\@title}{5}`, also of `About \nm`; `\@car`, `\@gobble`) stays, also beside the rest of the value
///   set in a box never shown, in a formula or in a `\phantom` (`field_ink_runs`);
/// - where the frontmatter displays the field: a document-level `<ltx:title>`, or a document-level author
///   `<ltx:creator>`, with no `<ltx:titlepage>` beside it (beside one the XSLT shows neither, LaTeXML-structure-xhtml.xsl
///   :374-375 = Perl :358-359 and :687-692, and the print is the displayed copy: 1304.0681, 1503.03277);
/// - in the document's title block: no body text before it in the document's content order
///   (`field_runs_after_body_text`: a letter's closing `\noindent\@author` stays), and `is_field_ink_wrapper` for every ancestor of both marks — never inside a
///   `{titlepage}`, a float, a note, a frontmatter element or a frame;
/// - when the fields' prints fill their lines: the paragraph or table cell around each mark holds nothing but lifted
///   ink, so a mention in prose (`Title: \thetitle`) keeps its words and a title and author set on one line go
///   together.
///
/// Every whole print in the title block is lifted, a title the document prints twice too: it shows once, in the
/// frontmatter element (user ruling 2026-10-17, "lift every whole print"; OXIDIZED_DESIGN #715).
///
/// Returns whether a title print was lifted: the text-matching dedup (`maybe_dedup_leading_title_ink`) then has
/// nothing left to explain. P2 witnesses: 1605.00182, 2402.14614, 1603.09631 (lrec `\maketitleabstract`), 1503.06457.
pub fn lift_marked_field_ink(document: &mut Document) -> Result<bool> {
  if lookup_bool("lx_field_ink_settled") {
    return Ok(false);
  }
  assign_value("lx_field_ink_settled", true, Some(Scope::Global));
  document.settle_field_ink()?;
  let Some(root) = document.get_document().get_root_element() else {
    return Ok(false);
  };
  let (runs, marks) = field_ink_runs(document, &root);
  if marks.is_empty() {
    return Ok(false);
  }
  let no_titlepage = document
    .findnode("/ltx:document/ltx:titlepage", None)
    .is_none();
  let title_shown = no_titlepage && document.findnode("/ltx:document/ltx:title", None).is_some();
  let creators_shown = no_titlepage
    && document
      .findnode("/ltx:document/ltx:creator[@role='author']", None)
      .is_some();
  // a framed wrapper (`\fbox{\@title}`, a framed cover minipage) is the author's own ink around the print: emptied, it
  // would leave a frame the PDF does not have (as the dedup, 68s review finding 4)
  let in_title_block = |mark: &Node| {
    ancestors_below(mark, &root)
      .iter()
      .all(|a| is_field_ink_wrapper(document::get_node_qname(a)) && !a.has_attribute("framed"))
  };
  // the line a mark sits in: its nearest paragraph or table cell
  let line_of = |mark: &Node| {
    ancestors_below(mark, &root).into_iter().find(|a| {
      let q = document::get_node_qname(a);
      q == pin_static("ltx:p") || q == pin_static("ltx:td")
    })
  };
  // the whole prints of a displayed field in the title block, with their ink and their lines
  let mut candidates: Vec<(&FieldInkRun, Vec<Node>, Vec<Node>)> = Vec::new();
  for run in &runs {
    let Some((name, _)) = FIELD_MARK_FIELDS.iter().find(|(_, id)| *id == run.field) else {
      continue;
    };
    let shown = if *name == "title" {
      title_shown
    } else {
      creators_shown
    };
    let whole = !run.reread && covers_extent(&run.cover, &field_extent(run.field));
    if !shown
      || !whole
      || run.body_before
      || !in_title_block(&run.begin)
      || !in_title_block(&run.end)
    {
      continue;
    }
    let range = marked_range(&root, &run.begin, &run.end);
    if range.is_empty() {
      continue;
    }
    let lines: Vec<Node> = [&run.begin, &run.end]
      .into_iter()
      .filter_map(line_of)
      .collect();
    candidates.push((run, range, lines));
  }
  // the prints that fill their lines with lifted ink alone, all fields together (finding 5: a title and its author on
  // one line), until no more drop out
  loop {
    let gone: Vec<Node> = candidates
      .iter()
      .flat_map(|(_, range, _)| range.iter())
      .chain(marks.iter())
      .cloned()
      .collect();
    let before = candidates.len();
    candidates.retain(|(_, _, lines)| !lines.iter().any(|line| holds_content_besides(line, &gone)));
    if candidates.len() == before {
      break;
    }
  }
  let mut doomed: Vec<Node> = marks.clone();
  let mut chains: Vec<Vec<Node>> = Vec::new();
  let mut lifted_title = false;
  for (run, mut range, lines) in candidates {
    let name = FIELD_MARK_FIELDS
      .iter()
      .find(|(_, id)| *id == run.field)
      .map_or("", |(name, _)| *name);
    // the lines' breaks go with the print they end
    for line in &lines {
      range.extend(document.findnodes("descendant::ltx:break", Some(line)));
    }
    let text = normalize_frontmatter_text(
      &range
        .iter()
        .filter(|n| n.get_type() != Some(NodeType::PiNode))
        .map(|n| n.get_content())
        .collect::<Vec<_>>()
        .join(" "),
    );
    Info!(
      "frontmatter",
      "field_ink",
      s!("lifted the printed \\@{name} (\"{text}\"): the frontmatter displays it")
    );
    lifted_title |= name == "title";
    chains.push(ancestors_below(&run.begin, &root));
    chains.push(ancestors_below(&run.end, &root));
    doomed.extend(range);
  }
  remove_marked_ink(document, doomed, chains, |w| {
    is_field_ink_wrapper(document::get_node_qname(w)) && !w.has_attribute("labels")
  });
  Ok(lifted_title)
}

/// At `\end{document}`, the field marks still in the tree (made with no placement to settle them, `\lx@field@mark`)
/// removed; their ink stays.
pub fn settle_field_marks(document: &mut Document) {
  let Some(root) = document.get_document().get_root_element() else {
    return;
  };
  for mark in field_marks(document, &root) {
    document.remove_node(mark);
  }
}

/// Would `insert_frontmatter` place anything right now? Mirrors its three early
/// returns (already done; nothing queued; the abstract-only first-call deferral).
pub(in crate::base_utilities) fn frontmatter_pending() -> bool {
  if lookup_bool("frontmatter_done") {
    return false;
  }
  let keys: Vec<String> = with_value("frontmatter", |v| match v {
    Some(Stored::HashTagData(frnt)) => frnt.keys().cloned().collect(),
    _ => Vec::new(),
  });
  !(keys.is_empty()
    || (keys.len() == 1 && keys[0] == "ltx:abstract" && !lookup_bool("frontmatter_deferred")))
}
