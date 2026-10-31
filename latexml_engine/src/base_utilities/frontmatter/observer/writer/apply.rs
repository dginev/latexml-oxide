//! The writer's operations on today's frontmatter (FM S2b): copies of today's items for the authors the reading gives
//! them, the items made from queued tokens (a cut entry's pieces, a legend's note) digested as today's digests an
//! entry (`digest_frontmatter_item`) and inserted as today's inserts one (`insert_frontmatter_entry`), roles set, and
//! today's items taken away — in that order, so a copy is made before its source goes, and a copy that stays takes back
//! the ids of the one taken.

use std::collections::BTreeMap;

use super::{
  dom::{Front, read_creator},
  plan::{Key, Made, Op, Owner, Place, Plan},
  *,
};

/// The element children of `node`.
pub(super) fn elements(node: &Node) -> Vec<Node> {
  node
    .get_child_nodes()
    .into_iter()
    .filter(|n| n.get_type() == Some(NodeType::ElementNode))
    .collect()
}

/// The author block's own creator (OD #159): today's first nameless creator, or a new one after the last creator.
fn block_creator(document: &mut Document, front: &Front) -> Result<Option<Node>> {
  if let Some(creator) = front.nameless.first() {
    return Ok(Some(creator.node.clone()));
  }
  let Some(mut last) = front.creators.last().map(|c| c.node.clone()) else {
    return Ok(None);
  };
  let Some(parent) = last.get_parent() else {
    return Ok(None);
  };
  let saved = document.get_node().clone();
  document.set_node(&parent);
  let mut attributes = HashMap::default();
  attributes.insert("role".to_string(), "author".to_string());
  let mut creator = document.open_element("ltx:creator", Some(attributes), None)?;
  document.close_element("ltx:creator")?;
  document.set_node(&saved);
  // (after the last creator, where today's puts the block's own)
  if last.add_next_sibling(&mut creator).is_err() {
    return Ok(None);
  }
  Ok(Some(creator))
}

/// The attributes an item made from `template` (today's contact it replaces) takes: its own, the internal ones aside.
fn template_attributes(template: &Node) -> TagAttrs {
  let mut attrs = TagAttrs::default();
  for (key, value) in template.get_attributes() {
    if !key.starts_with('_') && key != "id" && key != "xml:id" {
      attrs.insert(key, value);
    }
  }
  attrs
}

/// Digest `tokens` as `made` and insert it into `owner`, before `before` when given. An affiliation takes the
/// attributes of `template` (today's contact it replaces), or with none (the re-part's entry today's printed inside a
/// name) those of a contact of `role`: its role and its name label, as `\lx@add@affiliation` gives them. A legend
/// shows its `label` as its note's mark ([`show_legend_mark`]).
#[allow(clippy::too_many_arguments)]
pub(super) fn make(
  document: &mut Document,
  owner: &Node,
  made: Made,
  tokens: &Tokens,
  template: Option<&Node>,
  role: &str,
  before: Option<&Node>,
  label: Option<&str>,
) -> Result<()> {
  let entry = match made {
    Made::Affiliation => {
      // (the wrapper today's affiliation calls digest their content in: `\inst` sets a label, the author markup)
      let wrapped = Invocation!(T_CS!("\\lx@affiliation@withinst"), vec![Some(
        tokens.clone()
      )]);
      let digested = digest_frontmatter_item("ltx:contact", wrapped)?;
      let attr = match template {
        Some(template) => template_attributes(template),
        None => {
          let mut attr = TagAttrs::default();
          attr.insert("role".to_string(), role.to_string());
          if let Some(name) = get_frontmatter_name(None, "ltx:contact", role)? {
            attr.insert("name".to_string(), name);
          }
          attr
        },
      };
      TagData {
        tag: "ltx:contact".to_string(),
        attr,
        content: vec![TagContent::Box(digested)],
      }
    },
    Made::Legend => {
      // (the kernel's author note, `\lx@add@thanks`: a marked note with its content kind; its mark's ink kept in its
      // text, the label-text ruling)
      let kind = classify_thanks(tokens.clone().untex_string().as_ref());
      let digested = digest_frontmatter_item("ltx:note", tokens.clone())?;
      let mut attr = TagAttrs::default();
      attr.insert("role".to_string(), "thanks".to_string());
      attr.insert(
        "class".to_string(),
        s!("ltx_note_frontmatter ltx_thanks_{kind}"),
      );
      TagData {
        tag: "ltx:note".to_string(),
        attr,
        content: vec![TagContent::Box(digested)],
      }
    },
  };
  let saved = document.get_node().clone();
  document.set_node(owner);
  document.set_box_to_absorb(
    Tbox::new(
      pin!(""),
      lookup_font(),
      None,
      Tokens!(T_SPACE!()),
      SymHashMap::default(),
    )
    .into(),
  );
  let inserted = insert_frontmatter_entry(document, &entry);
  document.expire_box_to_absorb();
  document.set_node(&saved);
  inserted?;
  if made == Made::Legend
    && let (Some(note), Some(label)) = (elements(owner).pop(), label)
  {
    show_legend_mark(document, owner, &note, label)?;
  }
  if let Some(before) = before
    && let Some(mut new) = elements(owner).pop()
  {
    let mut before = before.clone();
    let _ = before.add_prev_sibling(&mut new);
  }
  Ok(())
}

/// A legend made a note shows the mark its line prints (charter: a note keeps the label the class prints): its `mark`
/// is the legend's label as printed — the glyph of each label ([`unanswered_mark_text`]), or the superscript the note's
/// text opens with where it prints that glyph (`\textsuperscript{*}`'s `*`, `$^*$`'s `∗`) —, and the name of `owner`
/// showing that mark shows it no more: the note's mark is printed beside the name, as a `\thanks` note's is, and a
/// superscript of the name's own (`$^*$`, `\lx@frontmatter@keepsup`) or one shown for no entry beside it printed the
/// mark twice — with no `mark` the page set the note's default `†` beside the name's `*` (review rev94a r5: f08's
/// "Observatory*†", k01's `Ann Able$^*$, Bob Baker\\ $^*$Equal contribution`).
fn show_legend_mark(document: &mut Document, owner: &Node, note: &Node, label: &str) -> Result<()> {
  let unify = |m: &str| m.replace('\u{2217}', "*");
  let leading = dom::segments(note, false)
    .into_iter()
    .find_map(|seg| match seg {
      dom::Seg::Hidden(mark) if !dom::is(&mark, "ltx:tags") => Some(mark.get_content()),
      dom::Seg::Text(text) if text.get_content().trim().is_empty() => None,
      _ => Some(String::new()),
    })
    .map(|m| m.trim().to_string());
  let mut glyphs: Vec<String> = mark_labels(label)
    .iter()
    .filter_map(|l| unanswered_mark_text(l))
    .collect();
  // (a label of several symbols — `$^{**}$`, `$^{\dagger\dagger}$`, `\textsuperscript{**}` — is no one glyph: its mark
  // is the note's leading superscript where that prints the label's symbols, each as its glyph; l04's `Bob Baker$^{**}$
  // … $^{**}$Corresponding author` showed "Bob Baker** †", the default dagger — review rev94a r6 S1)
  if glyphs.is_empty() {
    let symbols = |m: &str| -> String {
      m.chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| unify(&unanswered_mark_text(&c.to_string()).unwrap_or_else(|| c.to_string())))
        .collect()
    };
    match leading
      .as_deref()
      .filter(|m| !m.is_empty() && symbols(m) == symbols(label))
    {
      Some(m) => glyphs.push(m.to_string()),
      None => return Ok(()),
    }
  }
  let printed = leading.filter(|m| unify(m) == unify(&glyphs.join(",")));
  let mark = printed.unwrap_or_else(|| glyphs.join(","));
  let mut note = note.clone();
  document.set_attribute(&mut note, "mark", &mark)?;
  let shown: Vec<String> = glyphs.iter().map(|g| unify(g)).collect();
  for sup in document.findnodes("ltx:personname//ltx:sup", Some(owner)) {
    let text = sup.get_content();
    let kept: Vec<&str> = text
      .split(',')
      .filter(|m| !shown.contains(&unify(m.trim())))
      .collect();
    if kept.is_empty()
      || (kept.len() < text.split(',').count()
        && !dom::take_sup_marks(&sup, &|m| shown.contains(&unify(m))))
    {
      document.remove_node(sup);
    }
  }
  // (a mark formula the name shows, not yet rewritten into its superscript — revtex's `Bob Baker$^{1,\star}$`: the
  // glyph's tokens taken, with the comma beside them, and a formula left with no mark taken whole. A mark of several
  // symbols is a run of adjacent tokens between commas, one per symbol — `$^{1,\dagger\dagger}$`'s `†`, `†` —, taken
  // when its joined text is the mark: no single token was `††`, and the name showed "Ann Able1,††" beside the note's
  // mark, review rev94a r7 SF1, v/rv15, v/rv2 `$^{2,**}$`)
  for mut math in document.findnodes("ltx:personname//ltx:Math", Some(owner)) {
    if !matches!(marks::read_formula(&math), marks::FormulaMarks::Marks(_)) {
      continue;
    }
    // (a run that is the mark is taken whole — `$^{1,\dagger\dagger}$` under a `$^{\dagger\dagger}$` legend, OD #927's
    // compound reading; `$^{\star 1}$`'s glued `⋆` is a run of its own, `token_runs`. A run that is not the mark gives
    // its marks of one distinct symbol each that are: `$^{1,*\spadesuit}$`'s `∗` under a `$^*$` legend, each symbol
    // requested (2606.15609). A symbol repeated is one mark of its own, not the label's symbol: `$^{1,**}$` under a
    // `$^{*}$` legend and `$^{1,\dagger\dagger}$` under a `$^\dagger$` one keep their `∗∗` and `††` shown, as pdflatex
    // prints them and as a superscript's are (`$^{\dagger\dagger}$`, w/w17) — round 7 took the run's tokens one by one
    // and the name showed "Ann Able1" beside the note's ∗, review rev94a r8 P1, w/w11, w/w16)
    let runs: Vec<Vec<Node>> =
      token_runs(&document.findnodes("descendant::ltx:XMTok", Some(&math)))
        .into_iter()
        .flat_map(|run| {
          let joined = |run: &[Node]| -> String {
            run
              .iter()
              .map(|token| token.get_content().trim().to_string())
              .collect()
          };
          if shown.contains(&unify(&joined(&run))) {
            vec![run]
          } else {
            repeated_symbols(run)
              .into_iter()
              .filter(|symbol| shown.contains(&unify(&joined(symbol))))
              .collect()
          }
        })
        .collect();
    // (its `tex=` spelled without them too, `{}^{1,\star}` → `{}^{1}`: a run whose spelling the source does not show
    // leaves the formula whole, both marks printed, rather than a `tex=` the formula no longer holds — review rev94a r6
    // N2)
    let mut tex = math.get_attribute("tex");
    if let Some(spelled) = tex.as_mut()
      && !runs.iter().all(|run| drop_spelling(spelled, run))
    {
      continue;
    }
    let comma = |node: Option<Node>| {
      node.filter(|n| dom::is(n, "ltx:XMTok") && n.get_content().trim() == ",")
    };
    let mut taken = false;
    for run in &runs {
      let (Some(first), Some(last)) = (run.first(), run.last()) else {
        continue;
      };
      if let Some(comma) =
        comma(first.get_prev_sibling()).or_else(|| comma(last.get_next_sibling()))
      {
        document.remove_node(comma);
      }
      for token in run {
        let parent = token.get_parent();
        document.remove_node(token.clone());
        // (a script the run was all of goes with it: `${}^{2}{}^{\star}$`'s second `FLOATSUPERSCRIPT`)
        let mut at = parent;
        while let Some(node) = at.filter(|n| !dom::is(n, "ltx:XMath") && elements(n).is_empty()) {
          at = node.get_parent();
          document.remove_node(node);
        }
      }
      taken = true;
    }
    let empty = |scripts: &[String]| {
      scripts
        .iter()
        .all(|s| s.chars().all(|c| c == ',' || c.is_whitespace()))
    };
    let left = document.findnodes("descendant::ltx:XMTok", Some(&math));
    if taken
      && (left.is_empty()
        || matches!(marks::read_formula(&math), marks::FormulaMarks::Marks(scripts) if empty(&scripts)))
    {
      document.remove_node(math);
    } else if taken && let Some(tex) = tex {
      document.set_attribute(&mut math, "tex", &tex)?;
    }
  }
  Ok(())
}

/// The runs of a formula's `tokens` (in document order) a comma parts: adjacent element siblings, none a comma, a
/// letter or digit glued to a symbol parted from it — one mark each, `$^{1,\dagger\dagger}$`'s and `$^{1\dagger
/// \dagger}$`'s `1` and `††`.
fn token_runs(tokens: &[Node]) -> Vec<Vec<Node>> {
  let mut runs: Vec<Vec<Node>> = Vec::new();
  for token in tokens {
    if token.get_content().trim() == "," {
      runs.push(Vec::new());
      continue;
    }
    let follows = runs.last().and_then(|run| run.last()).is_some_and(|last| {
      let mut next = last.get_next_sibling();
      while let Some(n) = next
        .as_ref()
        .filter(|n| n.get_type() != Some(NodeType::ElementNode))
      {
        next = n.get_next_sibling();
      }
      next.as_ref() == Some(token)
        && marks::has_word(&last.get_content()) == marks::has_word(&token.get_content())
    });
    match runs.last_mut() {
      Some(run) if follows => run.push(token.clone()),
      _ => runs.push(vec![token.clone()]),
    }
  }
  runs.retain(|run| !run.is_empty());
  runs
}

/// A run of a formula's mark tokens parted into its marks of one symbol each: adjacent tokens printing the same glyph
/// are one mark (`**`, `††`), a different glyph starts the next (`*†` is `*` and `†`).
fn repeated_symbols(run: Vec<Node>) -> Vec<Vec<Node>> {
  let glyph = |token: &Node| token.get_content().trim().replace('\u{2217}', "*");
  let mut symbols: Vec<Vec<Node>> = Vec::new();
  for token in run {
    match symbols.last_mut() {
      Some(symbol)
        if symbol
          .last()
          .is_some_and(|last| glyph(last) == glyph(&token)) =>
      {
        symbol.push(token)
      },
      _ => symbols.push(vec![token]),
    }
  }
  symbols
}

/// The spellings a formula's mark `token` may have in its `tex`: the control word its `name` spells (`\star`), its
/// text, the spellings of its glyph (`*`, `\ast` for `∗`).
fn token_spellings(token: &Node) -> Vec<String> {
  let text = token.get_content().trim().to_string();
  let mut spellings: Vec<String> = token
    .get_attribute("name")
    .map(|name| s!("\\{name}"))
    .into_iter()
    .collect();
  spellings.push(text.clone());
  let glyphs: &[&str] = match text.as_str() {
    "\u{2217}" | "*" => &["*", "\\ast"],
    "\u{2020}" => &["\\dagger", "\\dag"],
    "\u{2021}" => &["\\ddagger", "\\ddag"],
    "\u{22C6}" => &["\\star"],
    _ => &[],
  };
  spellings.extend(glyphs.iter().map(|g| g.to_string()));
  spellings.retain(|s| !s.is_empty());
  spellings
}

/// The commands that print no mark of their own around one: a font or a style (`\mathbf{*}`, `{\rm a}`), a space.
const MARK_WRAPPERS: &[&str] = &[
  "\\mathbf",
  "\\mathrm",
  "\\mathit",
  "\\mathsf",
  "\\mathtt",
  "\\mathnormal",
  "\\boldsymbol",
  "\\bm",
  "\\pmb",
  "\\textbf",
  "\\textit",
  "\\textrm",
  "\\textnormal",
  "\\text",
  "\\mbox",
  "\\bf",
  "\\rm",
  "\\it",
  "\\sf",
  "\\tt",
  "\\scriptstyle",
  "\\scriptscriptstyle",
  "\\textstyle",
  "\\displaystyle",
  "\\,",
  "\\;",
  "\\:",
  "\\!",
  "\\ ",
  "~",
];

/// The TeX tokens of `spelled` (a control sequence, or one character) with their spans, its braces, spaces and
/// [`MARK_WRAPPERS`] out.
fn mark_tokens(spelled: &str) -> Vec<(String, usize, usize)> {
  let mut out = Vec::new();
  let mut chars = spelled.char_indices().peekable();
  while let Some((at, c)) = chars.next() {
    let mut end = at + c.len_utf8();
    if c == '\\' {
      match chars.next() {
        Some((i, l)) if l.is_ascii_alphabetic() => {
          end = i + l.len_utf8();
          while let Some(&(i, l)) = chars.peek().filter(|(_, l)| l.is_ascii_alphabetic()) {
            end = i + l.len_utf8();
            chars.next();
          }
        },
        Some((i, symbol)) => end = i + symbol.len_utf8(),
        None => {},
      }
    }
    let token = &spelled[at..end];
    if !(token == "{" || token == "}" || token.trim().is_empty() || MARK_WRAPPERS.contains(&token))
    {
      out.push((token.to_string(), at, end));
    }
  }
  out
}

/// `tex` without the spelling of a run of mark tokens of its formula, and the comma beside it: the item of a script
/// group (`^{…}`, `^x`) its tokens spell one by one ([`token_spellings`]; a font wrapper or a space around them taken
/// too, `^{2,\mathbf{*}}` → `^{2}`), the last group holding one first, as the marks end the formula; a group left
/// empty goes whole, with an empty base before it (`{}^{2}{}^{\star}` → `{}^{2}`, review rev94a r7 N1). A run that is
/// part of an item — a token taken from a glued mark, `^{\star 1}`, `^{1,\dagger\dagger}` under a `\dagger` legend —
/// is taken from it, its last such place first. Whether it was found.
fn drop_spelling(tex: &mut String, run: &[Node]) -> bool {
  drop_spelled(tex, &run.iter().map(token_spellings).collect::<Vec<_>>())
}

/// [`drop_spelling`] of a run whose tokens may each be spelled one of `spellings`.
fn drop_spelled(tex: &mut String, spellings: &[Vec<String>]) -> bool {
  // (where in `item`'s tokens the run is spelled, the last place first)
  let spelled_at = |item: &str| -> Option<(usize, usize, bool)> {
    let tokens = mark_tokens(item);
    let n = spellings.len();
    (n > 0 && n <= tokens.len())
      .then(|| {
        (0..=tokens.len() - n).rev().find(|&at| {
          tokens[at..at + n]
            .iter()
            .zip(spellings)
            .all(|((token, ..), spellings)| spellings.contains(token))
        })
      })
      .flatten()
      .map(|at| (tokens[at].1, tokens[at + n - 1].2, tokens.len() == n))
  };
  let groups = script_groups(tex);
  // (an item spelling the run whole first, in any group; else a part of one)
  for whole in [true, false] {
    for &(caret, (start, end), close) in groups.iter().rev() {
      let items = top_level_items(&tex[start..end]);
      let Some((from, to, part)) = items.iter().rev().find_map(|&(from, to)| {
        spelled_at(&tex[start + from..start + to])
          .filter(|&(.., is_whole)| is_whole == whole)
          .map(|(f, t, _)| {
            (
              start + from,
              start + to,
              (start + from + f, start + from + t),
            )
          })
      }) else {
        continue;
      };
      if !whole {
        tex.replace_range(part.0..part.1, "");
        return true;
      }
      let rest_empty = items
        .iter()
        .all(|&(f, t)| start + f == from || tex[start + f..start + t].trim().is_empty());
      if rest_empty {
        // (the whole group: its caret to its end, and an empty base right before it)
        let mut open = caret;
        let before = tex[..caret].trim_end();
        if before.ends_with("{}") {
          open = before.len() - 2;
        }
        tex.replace_range(open..close, "");
      } else {
        // (the comma before the item, else the one after it)
        let before = tex[start..from].trim_end();
        let after = tex[to..end].trim_start();
        let (from, to) = if before.ends_with(',') {
          (start + before.len() - 1, to)
        } else if after.starts_with(',') {
          (from, end - after.len() + 1)
        } else {
          (from, to)
        };
        tex.replace_range(from..to, "");
      }
      return true;
    }
  }
  false
}

/// The superscript groups of `tex`: each its caret's place, its body's span — a braced group's inside, or the one
/// control word or character after the caret — and the group's end.
fn script_groups(tex: &str) -> Vec<(usize, (usize, usize), usize)> {
  let bytes = tex.as_bytes();
  let mut groups = Vec::new();
  let mut at = 0;
  while let Some(offset) = tex[at..].find('^') {
    let caret = at + offset;
    let mut start = caret + 1;
    while start < bytes.len() && bytes[start] == b' ' {
      start += 1;
    }
    if start >= bytes.len() {
      break;
    }
    if bytes[start] == b'{' {
      let mut depth = 0usize;
      let mut close = None;
      for (i, b) in bytes.iter().enumerate().skip(start) {
        // (an escaped brace, `\{`, is none)
        match b {
          b'{' if i == 0 || bytes[i - 1] != b'\\' => depth += 1,
          b'}' if i == 0 || bytes[i - 1] != b'\\' => {
            depth -= 1;
            if depth == 0 {
              close = Some(i);
              break;
            }
          },
          _ => {},
        }
      }
      let Some(close) = close else {
        break;
      };
      groups.push((caret, (start + 1, close), close + 1));
      at = close + 1;
    } else {
      let token = mark_token_end(tex, start);
      groups.push((caret, (start, token), token));
      at = token;
    }
  }
  groups
}

/// The end of the TeX token starting at byte `start` of `tex`: a control word, a control symbol, or one character.
fn mark_token_end(tex: &str, start: usize) -> usize {
  let rest = &tex[start..];
  let mut chars = rest.char_indices();
  match chars.next() {
    Some((_, '\\')) => match chars.next() {
      Some((i, l)) if l.is_ascii_alphabetic() => {
        let mut end = i + l.len_utf8();
        for (j, c) in chars {
          if !c.is_ascii_alphabetic() {
            break;
          }
          end = j + c.len_utf8();
        }
        start + end
      },
      Some((i, c)) => start + i + c.len_utf8(),
      None => tex.len(),
    },
    Some((_, c)) => start + c.len_utf8(),
    None => start,
  }
}

/// The spans of the items a comma parts in a group's `body`, at its own brace level.
fn top_level_items(body: &str) -> Vec<(usize, usize)> {
  let mut items = Vec::new();
  let (mut depth, mut from) = (0i32, 0usize);
  let mut prev = '\0';
  for (i, c) in body.char_indices() {
    match c {
      '{' if prev != '\\' => depth += 1,
      '}' if prev != '\\' => depth -= 1,
      ',' if depth == 0 => {
        items.push((from, i));
        from = i + 1;
      },
      _ => {},
    }
    prev = c;
  }
  items.push((from, body.len()));
  items
}

/// The element whose text a contact prints: the innermost of the chain of only children `node` opens (a contact's
/// `ltx:text` of the typewriter font), when it holds text alone — `None` when the chain holds anything else (a link,
/// a formula, two pieces), whose text the writer does not set.
pub(super) fn text_leaf(node: &Node) -> Option<Node> {
  let mut at = node.clone();
  loop {
    let children = at.get_child_nodes();
    let mut elements = children
      .iter()
      .filter(|c| c.get_type() != Some(NodeType::TextNode) || !c.get_content().trim().is_empty());
    match (elements.next(), elements.next()) {
      (Some(only), None)
        if only.get_type() == Some(NodeType::ElementNode)
          && document::get_node_qname(only) == pin("ltx:text") =>
      {
        at = only.clone();
      },
      _ => break,
    }
  }
  at.get_child_nodes()
    .iter()
    .all(|c| c.get_type() == Some(NodeType::TextNode))
    .then_some(at)
}

/// `node`'s text set to `text`, in its [`text_leaf`].
fn set_text(document: &Document, node: &Node, text: &str) -> Result<()> {
  let Some(mut at) = text_leaf(node) else {
    return Ok(());
  };
  for mut child in at.get_child_nodes() {
    child.unlink();
  }
  let mut node = Node::new_text(text, &document.document)?;
  at.add_child(&mut node)?;
  Ok(())
}

/// The elements of `node`'s subtree that carry an `xml:id`, with it, in document order.
pub(super) fn subtree_ids(document: &mut Document, node: &Node) -> Vec<(Node, String)> {
  document
    .findnodes("descendant-or-self::*[@xml:id]", Some(node))
    .into_iter()
    .filter_map(|n| n.get_attribute_ns("id", XML_NS).map(|id| (n, id)))
    .collect()
}

/// A taken item's ids given back to a copy of it that stays. Today's numbers each creator's copy of an item it places
/// under several (`footnotex3`, then `modify_id`'s `footnotex3a`), and a copy the writer makes is numbered so too: taking
/// the copy that held the plain id left the other's suffixed one standing alone (`footnotex3a`, no `footnotex3`). The
/// copy is the element named as the taken subtree's first id-holder whose id is that one's with a `radix_alpha` suffix
/// (a contact holds no id, the footnote mark in it does), its subtree's ids paired in order with the taken subtree's,
/// and references to them follow.
pub(super) fn reclaim_ids(document: &mut Document, taken: &[(SymStr, String)]) -> Result<()> {
  let Some(&(qname, ref id)) = taken.first() else {
    return Ok(());
  };
  if document.lookup_id(id).is_some() {
    return Ok(());
  }
  let Some(copy) = (1..=702).find_map(|n| {
    document
      .lookup_id(&s!("{id}{}", radix::radix_alpha(n)))
      .filter(|c| c.get_parent().is_some() && document::get_node_qname(c) == qname)
      .cloned()
  }) else {
    return Ok(());
  };
  let copy_ids = subtree_ids(document, &copy);
  if copy_ids.len() != taken.len() || taken.iter().any(|(_, id)| document.lookup_id(id).is_some()) {
    return Ok(());
  }
  for ((mut node, old), (_, new)) in copy_ids.into_iter().zip(taken) {
    document.unrecord_id(&old);
    document.set_attribute(&mut node, "xml:id", new)?;
    for mut reference in document.findnodes(&s!("//*[@idref='{old}']"), None) {
      document.set_attribute(&mut reference, "idref", new)?;
    }
  }
  Ok(())
}

/// `new`, an item just added to `owner` (the node of `who`), moved to where the reading prints it: before the first of
/// the owner's items the reading prints after it — not at the creator's end, where a copy made in the order of the
/// items' texts reordered what the PDF prints (rev92c S1: Sangyoun Lee's institutes 1 and 2 copied KIST before Yonsei,
/// 2207.06953; S. Mangla's ORIGINS before LMU and O. M. Smirnov's in the order of their names, 2608.05923; Ilyass
/// Haloui's written-out Airbus address after ISAE, 1812.02463). An item the reading places nowhere stays last.
/// Whether the reading places it.
pub(super) fn place(
  owner: &Node,
  who: Owner,
  new: &Node,
  order: &BTreeMap<(Owner, Key), Place>,
) -> bool {
  let items = read_creator(owner).items;
  // (within one print, an entry's own text before the contacts and notes it holds: `Univ X \\ \email{…}`)
  let place_of = |item: &dom::Item| {
    order
      .get(&(who, (item.kind.clone(), item.key.clone())))
      .map(|place| (*place, item.kind.clone()))
  };
  let Some(at) = items
    .iter()
    .find(|item| item.movable && item.node == *new)
    .and_then(place_of)
  else {
    return false;
  };
  if let Some(next) = items
    .iter()
    .filter(|item| item.movable && item.node != *new)
    .find(|item| place_of(item).is_some_and(|p| prints_after(&p, &at)))
  {
    let mut next = next.node.clone();
    let mut new = new.clone();
    let _ = next.add_prev_sibling(&mut new);
  }
  true
}

/// Whether the reading prints an item placed at `a` after one at `b`: a note's item after the author block's (aa's
/// `\offprints` e-mail below the institutes, review rev92c r2 B-1), then by their blocks or calls, then within one by
/// their spans — or, neither spanned, an entry's own text before what it holds (`Univ X \\ \email{…}`). One spanned
/// and one not in the same block are not ordered: a span is a position in the block, its absence none (an
/// `\institute`'s legend line cut at its span and the address line before it, which the row parted out with no span:
/// the legend had gone before the address it prints after, `perfect_kernel_batch61::marked_affiliation_review_shapes`).
fn prints_after(a: &(Place, dom::Kind), b: &(Place, dom::Kind)) -> bool {
  let ((anote, ai, (af, astart)), akind) = a;
  let ((bnote, bi, (bf, bstart)), bkind) = b;
  match ((anote, ai).cmp(&(bnote, bi)), af, bf) {
    (std::cmp::Ordering::Equal, 0, 0) => (astart, akind) > (bstart, bkind),
    (std::cmp::Ordering::Equal, 1, 1) => akind > bkind,
    (std::cmp::Ordering::Equal, ..) => false,
    (order, ..) => order == std::cmp::Ordering::Greater,
  }
}

/// Carry out `plan` on the document.
pub(super) fn apply(document: &mut Document, front: &Front, plan: &Plan) -> Result<()> {
  let mut block: Option<Node> = None;
  // (the owners the operations touch, whose items are then in the reading's order)
  let mut touched: Vec<(Owner, Node)> = Vec::new();
  let mut touch = |who: Owner, node: &Node| {
    if !touched.iter().any(|(o, _)| *o == who) {
      touched.push((who, node.clone()));
    }
  };
  let mut owner_node = |document: &mut Document, owner: Owner| -> Result<Option<Node>> {
    match owner {
      Owner::Creator(i) => Ok(plan.creators.get(&i).cloned()),
      Owner::Block => {
        if block.is_none() {
          block = block_creator(document, front)?;
        }
        Ok(block.clone())
      },
    }
  };
  for op in &plan.ops {
    if let Op::Copy { from, to } = op
      && let Some(mut owner) = owner_node(document, *to)?
    {
      document.append_clone(&mut owner, vec![from.clone()])?;
      touch(*to, &owner);
    }
  }
  for op in &plan.ops {
    if let Op::Make {
      owner: who,
      made,
      tokens,
      template,
      before,
      label,
    } = op
      && let Some(owner) = owner_node(document, *who)?
    {
      make(
        document,
        &owner,
        *made,
        tokens,
        Some(template),
        "affiliation",
        before.as_ref(),
        label.as_deref(),
      )?;
      touch(*who, &owner);
      // (an affiliation replacing none of today's items where the reading prints it, as a copy: 2311.16503's Jing Liu
      // had his institutes after the address the block prints below them; a legend stays last — the row's e-mail lines
      // are queued after the entry they print inside, so the calls' order does not place a legend line among them,
      // `perfect_kernel_batch61::marked_affiliation_review_shapes`)
      if before.is_none()
        && *made == Made::Affiliation
        && let Some(new) = elements(&owner).pop()
      {
        place(&owner, *who, &new, &plan.order);
      }
    }
  }
  for op in &plan.ops {
    if let Op::WriteOut { owner: who, template, address } = op
      && let Some(mut owner) = owner_node(document, *who)?
    {
      document.append_clone(&mut owner, vec![template.clone()])?;
      touch(*who, &owner);
      if let Some(mut copy) = elements(&owner).pop() {
        set_text(document, &copy, address)?;
        // (where the reading prints it; one it places nowhere at the template's place on the owner holding it, taken
        // after — today's place of a group may be no print order: Yuan Gao's `{malan,gaoy97}@` before his China
        // Telecom, 2603.23924)
        if !place(&owner, *who, &copy, &plan.order)
          && template.get_parent().as_ref() == Some(&owner)
        {
          let mut template = template.clone();
          let _ = template.add_prev_sibling(&mut copy);
        }
      }
    }
  }
  for op in &plan.ops {
    if let Op::Role { node, role } = op {
      let mut node = node.clone();
      document.set_attribute(&mut node, "role", role)?;
      if let Some(name) = get_frontmatter_name(None, "ltx:contact", role)? {
        document.set_attribute(&mut node, "name", &name)?;
      }
    }
  }
  // (the items taken, with their ids: a copy of one that stays takes them back)
  let mut taken = Vec::new();
  for op in &plan.ops {
    if let Op::Remove { node } = op {
      if let Some(parent) = node.get_parent() {
        if let Some((&i, _)) = plan.creators.iter().find(|(_, n)| **n == parent) {
          touch(Owner::Creator(i), &parent);
        } else if block.as_ref() == Some(&parent) || front.nameless.iter().any(|c| c.node == parent)
        {
          touch(Owner::Block, &parent);
        }
      }
      // (named while attached: a detached element's name no longer resolves its prefix)
      let ids: Vec<(SymStr, String)> = subtree_ids(document, node)
        .into_iter()
        .map(|(holder, id)| (document::get_node_qname(&holder), id))
        .collect();
      document.remove_node(node.clone());
      taken.push(ids);
    }
  }
  for ids in &taken {
    reclaim_ids(document, ids)?;
  }
  // (each touched owner's items in the reading's order, today's ones too: what an operation left — 2603.13900's Claudio
  // Di Ciccio kept today's address before his institute once the copy after it was taken; rev92c S1. Its notes stay
  // where they are, as a made legend does: the row's e-mail lines are queued after the entry they print inside, so the
  // calls' order does not place a note line among them)
  for (who, owner) in &touched {
    if owner.get_parent().is_some() {
      let items: Vec<Node> = read_creator(owner)
        .items
        .into_iter()
        .filter(|item| item.movable && item.kind != dom::Kind::Note)
        .map(|item| item.node)
        .collect();
      for item in items {
        place(owner, *who, &item, &plan.order);
      }
    }
  }
  // (a nameless creator the operations emptied is gone, as `coalesce_empty_creators` drops one)
  for creator in front.nameless.iter().map(|c| c.node.clone()).chain(block) {
    if creator.get_parent().is_some()
      && elements(&creator).iter().all(|e| {
        document::get_node_qname(e) == pin("ltx:personname") && e.get_content().trim().is_empty()
      })
    {
      document.remove_node(creator);
    }
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::drop_spelled;

  /// A mark run's spelling taken from a formula's `tex` (review rev94a r7 N1): from the last script group holding it,
  /// with its comma, a font wrapper or a space around it, and a group it was all of with the empty base before it.
  #[test]
  fn a_mark_runs_spelling_leaves_the_rest_of_the_tex() {
    let star = || vec!["\\star".to_string(), "\u{22C6}".to_string()];
    let ast = || vec!["\u{2217}".to_string(), "*".to_string(), "\\ast".to_string()];
    let dagger = || {
      vec![
        "\\dagger".to_string(),
        "\u{2020}".to_string(),
        "\\dag".to_string(),
      ]
    };
    for (tex, run, left) in [
      ("{}^{1,\\star}", vec![star()], Some("{}^{1}")),
      ("{}^{2}{}^{\\star}", vec![star()], Some("{}^{2}")),
      ("^{2,\\mathbf{*}}", vec![ast()], Some("^{2}")),
      ("^{2, \\star}", vec![star()], Some("^{2}")),
      ("^{\\star,1}", vec![star()], Some("^{1}")),
      (
        "^{1,\\dagger\\dagger}",
        vec![dagger(), dagger()],
        Some("^{1}"),
      ),
      ("^{2,**}", vec![ast(), ast()], Some("^{2}")),
      ("^{**,*}", vec![ast()], Some("^{**}")),
      ("^{*}{}^{1}", vec![ast()], Some("{}^{1}")),
      ("^{\\star 1}", vec![star()], Some("^{ 1}")),
      (
        "^{1,\\dagger\\dagger}",
        vec![dagger()],
        Some("^{1,\\dagger}"),
      ),
      ("^{1,\\stare}", vec![star()], None),
      ("^{1,\\dagger}", vec![dagger(), dagger()], None),
    ] {
      let mut spelled = tex.to_string();
      let found = drop_spelled(&mut spelled, &run);
      assert_eq!(found.then_some(spelled.as_str()), left, "{tex}");
    }
  }
}
