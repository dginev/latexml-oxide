//! FMRG: the red/green suite of frontmatter mark association, on both targets (DG 2026-11-16: "red/green TDD on both
//! XML and HTML targets is the only way to keep so many special cases working properly"). Every arXiv witness and
//! reviewer probe of the shared-note campaign (briefs SHARED, SHMARK, SHGIVEN, SHREST, PURE, REND2) is a repro whose
//! header records its pdflatex ORACLE; each row asserts that oracle, never what we output, in two layers with a status
//! each (a row can be XML-green and HTML-red):
//! - XML: per creator its personname (the name alone), every note it carries as `role[mark]=text` (the mark set,
//!   sorted), then its contacts in order;
//! - HTML (`LaTeXML-html5.xsl`, in process): per creator its name, the marks rendered on it in order (`m→note` linked
//!   to a printed note, `m plain` not), the bodies printed under it; then the bodies printed in the margin. Every note
//!   body prints once, no id twice, no `href="#x"` without its `x`.
//!
//! The rules the oracles are read by (FIDELITY_CHARTER §4, the mark rule; DG's rulings 1-4 of 2026-11-16; OD #1052):
//! - a note shared by several names is a full copy on each carrier, every copy with the printed `mark` (Perl's
//!   `relocateAnnotations`, Base_Utility.pool.ltxml:895-901); a one-carrier note may stay its name's contact, the mark
//!   consumed by it (ruling 2: shown under the name, unmarked);
//! - a mark on a name lives outside the personname: the note it answers, the contact it labels, or a mark-only
//!   `note role="footnotemark"` (answered by no note of the block; a title's note is the title's);
//! - a `\footnotemark`/`\samethanks`/`\authornotemark` on a name is a copy of the note it marks (Perl's
//!   `relocateFootnote_aux`, latex_constructs.pool.ltxml:518-527);
//! - a mark between two names is the name before's, the next name's only at a line's or cell's start, under the block's
//!   edge proof, or where the name before carries its key already (DG 2026-11-17, OD #1057); one followed by a
//!   separator, `\\` or the block's end is carried back; one alone on the line after `\\` is no name's (ruling 4);
//! - on the page a shared note prints once, in the margin, its mark on every carrying name; a marked note of one name
//!   prints under it with no mark on LaTeXML.css's card (a multi-line block, or a creator showing author lines alone),
//!   else its mark shows and its body is a pop-up (revMARKSAFEc S3); an unmarked note prints its text under it with no
//!   glyph, a mark-only note with no glyph nothing (DG 2026-11-16: no invented glyph, OD #990), a shared unmarked note
//!   once under the page's generated `[a]` (OD #1059); a mark-only note's mark links only to the one note of the block
//!   it marks, and only when it is a symbol (a letter, digit or Greek letter stays plain), and one linking nothing shows
//!   plain (the XML cannot prove its glyph prints once: DG 2026-11-16, "hide only when certain", OD #1059); the marks on
//!   a name in the order the PDF prints them.
//!
//! RED layers are known gaps, each naming its row R line (or NEITHER: a contradiction with no recorded gap, reported
//! to the main session); the ignored `_red` tests list what is still red and print the layers now matching their
//! oracle, to promote. A mark's U+2217 (`$^*$`) is folded to `*` (the class's `\@fnsymbol`).
//!
//! Re-classifying after an engine or XSLT change is one command: `cargo nextest run -E 'test(batchfmmarks)'
//! --run-ignored all`. A GREEN test failing is a regression; a `_red` test prints `now GREEN, promote: <repro> (<layer>)`
//! for each layer to promote (empty its `moves`/`page_moves`), and its report shows every RED layer's oracle and output.
use latexml::util::test::{convert_with_setup_then, error_count, warning_count};

use super::perfect_kernel_batch84b::text;

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// A repro and its oracle in both layers.
pub(super) struct Row {
  pub repro:      &'static str,
  pub tex:        &'static str,
  /// The Warnings the conversion honestly counts.
  pub warnings:   usize,
  /// Each creator as [`creator_lines`] writes it.
  pub creators:   &'static [&'static str],
  /// The row R line the XML layer waits on (empty: GREEN).
  pub moves:      &'static str,
  /// The author block as [`page_lines`] writes it.
  pub page:       &'static [&'static str],
  /// The row R line the HTML layer waits on (empty: GREEN).
  pub page_moves: &'static str,
}

/// The value of attribute `name` in the start tag `tag`.
fn attribute(tag: &str, name: &str) -> String {
  tag
    .split(&format!(" {name}=\""))
    .nth(1)
    .and_then(|value| value.split('"').next())
    .unwrap_or("")
    .to_string()
}

/// The text of a note or contact body, its `ltx:tags` left out.
fn body_text(body: &str) -> String {
  let mut body = body.to_string();
  while let (Some(from), Some(to)) = (body.find("<tags>"), body.find("</tags>")) {
    body.replace_range(from..to + "</tags>".len(), "");
  }
  text(&body)
}

/// A mark with U+2217 folded to `*`.
fn folded(mark: &str) -> String { mark.replace('\u{2217}', "*") }

/// Each creator of `xml`: `name ; note ; note | contact ; contact`, a note `role[mark]=text` (the notes sorted), a
/// contact `role=text` (in order); an element inside another is its parent's text.
pub(super) fn creator_lines(xml: &str) -> Vec<String> {
  xml
    .match_indices("<creator ")
    .map(|(start, _)| {
      let end = start + xml[start..].find("</creator>").expect("closed creator");
      let creator = &xml[start..end];
      let name = creator.find("<personname>").map_or(String::new(), |at| {
        let close = at
          + creator[at..]
            .find("</personname>")
            .expect("closed personname");
        text(&creator[at + "<personname>".len()..close])
      });
      let (mut notes, mut contacts) = (Vec::new(), Vec::new());
      let mut pos = 0;
      while let Some(at) = ["<note ", "<contact "]
        .iter()
        .filter_map(|open| creator[pos..].find(open).map(|i| pos + i))
        .min()
      {
        let kind = if creator[at..].starts_with("<note ") {
          "note"
        } else {
          "contact"
        };
        let tag_end = at + creator[at..].find('>').expect("start tag");
        let tag = &creator[at..tag_end];
        let (body, next) = if tag.ends_with('/') {
          ("", tag_end + 1)
        } else {
          // the matching end tag, past any element of the same kind inside
          let (mut depth, mut scan) = (1, tag_end + 1);
          loop {
            let open = creator[scan..].find(&format!("<{kind} ")).map(|i| scan + i);
            let close = scan + creator[scan..].find(&format!("</{kind}>")).expect("closed");
            match open {
              Some(o) if o < close => {
                let o_end = o + creator[o..].find('>').expect("start tag");
                if !creator[o..o_end].ends_with('/') {
                  depth += 1;
                }
                scan = o_end + 1;
              },
              _ => {
                depth -= 1;
                if depth == 0 {
                  break (&creator[tag_end + 1..close], close + kind.len() + 3);
                }
                scan = close + kind.len() + 3;
              },
            }
          }
        };
        if kind == "note" {
          notes.push(format!(
            "{}[{}]={}",
            attribute(tag, "role"),
            folded(&attribute(tag, "mark")),
            body_text(body)
          ));
        } else {
          contacts.push(format!("{}={}", attribute(tag, "role"), body_text(body)));
        }
        pos = next;
      }
      notes.sort();
      let mut line = std::iter::once(name)
        .chain(notes)
        .collect::<Vec<_>>()
        .join(" ; ");
      if !contacts.is_empty() {
        line = format!("{line} | {}", contacts.join(" ; "));
      }
      line
    })
    .collect()
}

/// The page `LaTeXML-html5.xsl` makes of `xml`, in process (the CLI's `html5` post-processing, no resources written).
pub(super) fn page_of(xml: &str) -> String {
  let options = latexml::post::PostOptions {
    pmml:                      true,
    cmml:                      false,
    keep_xmath:                false,
    stylesheet:                latexml::post::default_stylesheet(Some("html5")),
    destination:               None,
    source_directory:          None,
    site_directory:            None,
    search_paths:              &[],
    default_resources:         false,
    css_files:                 &[],
    js_files:                  &[],
    noinvisibletimes:          false,
    plane1:                    true,
    hackplane1:                false,
    mathtex:                   false,
    unicodemath:               false,
    url_style:                 latexml_post::crossref::UrlStyle::File,
    navigationtoc:             None,
    schemadocs:                false,
    split:                     false,
    split_xpath:               None,
    split_naming:              None,
    xslt_parameters:           &[],
    graphics_svg_threshold_kb: 0,
    graphicimages:             false,
    timestamp:                 None,
    icon:                      None,
    whatsout:                  latexml_post::extract::Whatsout::Document,
  };
  latexml::post::run_post_processing_logged(xml, &options).html
}

/// An element of a page: its tag, attributes and children (text or elements, by index).
struct Element {
  tag:      String,
  attrs:    Vec<(String, String)>,
  parent:   Option<usize>,
  children: Vec<Child>,
}

enum Child {
  Text(String),
  Element(usize),
}

impl Element {
  fn attr(&self, name: &str) -> Option<&str> {
    self
      .attrs
      .iter()
      .find(|(key, _)| key == name)
      .map(|(_, value)| value.as_str())
  }
  fn has_class(&self, class: &str) -> bool {
    self
      .attr("class")
      .is_some_and(|classes| classes.split_whitespace().any(|c| c == class))
  }
}

/// `text` with the entities a serialized page holds decoded.
fn unescaped(text: &str) -> String {
  let mut out = String::new();
  let mut rest = text;
  while let Some(at) = rest.find('&') {
    out.push_str(&rest[..at]);
    let after = &rest[at + 1..];
    let Some(end) = after.find(';').filter(|end| *end < 10) else {
      out.push('&');
      rest = after;
      continue;
    };
    let entity = &after[..end];
    let decoded = match entity {
      "amp" => Some('&'),
      "lt" => Some('<'),
      "gt" => Some('>'),
      "quot" => Some('"'),
      "apos" => Some('\''),
      "nbsp" => Some('\u{a0}'),
      _ => entity
        .strip_prefix("#x")
        .map(|hex| u32::from_str_radix(hex, 16))
        .or_else(|| entity.strip_prefix('#').map(str::parse))
        .and_then(Result::ok)
        .and_then(char::from_u32),
    };
    match decoded {
      Some(c) => out.push(c),
      None => out.push_str(&rest[at..=at + 1 + end]),
    }
    rest = &after[end + 1..];
  }
  out.push_str(rest);
  out
}

/// The elements of an HTML page (element 0 the root), its void elements closed where they open.
fn parse_page(html: &str) -> Vec<Element> {
  const VOID: [&str; 10] = [
    "br", "img", "hr", "meta", "link", "input", "col", "wbr", "area", "source",
  ];
  let mut elements = vec![Element {
    tag:      String::new(),
    attrs:    Vec::new(),
    parent:   None,
    children: Vec::new(),
  }];
  let mut current = 0;
  let mut rest = html;
  while let Some(at) = rest.find('<') {
    if at > 0 {
      elements[current]
        .children
        .push(Child::Text(unescaped(&rest[..at])));
    }
    rest = &rest[at..];
    if let Some(comment) = rest.strip_prefix("<!--") {
      rest = comment.find("-->").map_or("", |end| &comment[end + 3..]);
      continue;
    }
    let end = rest.find('>').unwrap_or(rest.len() - 1);
    let tag = &rest[1..end];
    rest = &rest[end + 1..];
    if tag.starts_with('!') || tag.starts_with('?') {
      continue;
    }
    if let Some(name) = tag.strip_prefix('/') {
      // close the nearest open element of that name
      let mut open = current;
      while open != 0 && elements[open].tag != name.trim() {
        open = elements[open].parent.unwrap_or(0);
      }
      if open != 0 {
        current = elements[open].parent.unwrap_or(0);
      }
      continue;
    }
    let self_closing = tag.ends_with('/');
    let tag = tag.trim_end_matches('/');
    let name = tag.split_whitespace().next().unwrap_or("").to_string();
    let mut attrs = Vec::new();
    let mut spec = &tag[name.len()..];
    while let Some(eq) = spec.find("=\"") {
      let key = spec[..eq].trim().to_string();
      let value_end = spec[eq + 2..].find('"').map_or(spec.len(), |e| eq + 2 + e);
      attrs.push((key, unescaped(&spec[eq + 2..value_end])));
      spec = &spec[(value_end + 1).min(spec.len())..];
    }
    let index = elements.len();
    let void = self_closing || VOID.contains(&name.as_str());
    elements.push(Element {
      tag: name,
      attrs,
      parent: Some(current),
      children: Vec::new(),
    });
    elements[current].children.push(Child::Element(index));
    if !void {
      current = index;
    }
  }
  elements
}

/// The elements under `at`, in document order.
fn descendants(page: &[Element], at: usize) -> Vec<usize> {
  let mut out = Vec::new();
  for child in &page[at].children {
    if let Child::Element(index) = child {
      out.push(*index);
      out.extend(descendants(page, *index));
    }
  }
  out
}

/// The text under `at`, whitespace collapsed, leaving out the elements `skip` says.
fn text_of(page: &[Element], at: usize, skip: &dyn Fn(&Element) -> bool) -> String {
  fn gather(page: &[Element], at: usize, skip: &dyn Fn(&Element) -> bool, out: &mut String) {
    for child in &page[at].children {
      match child {
        Child::Text(text) => out.push_str(text),
        Child::Element(index) if !skip(&page[*index]) => {
          out.push(' ');
          gather(page, *index, skip, out);
          out.push(' ');
        },
        Child::Element(_) => {},
      }
    }
  }
  let mut out = String::new();
  gather(page, at, skip, &mut out);
  out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The notes a creator renders: the `.ltx_note` elements under `at` not inside another note.
fn notes_under(page: &[Element], at: usize, out: &mut Vec<usize>) {
  for child in &page[at].children {
    if let Child::Element(index) = child {
      if page[*index].has_class("ltx_note") {
        out.push(*index);
      } else {
        notes_under(page, *index, out);
      }
    }
  }
}

/// Each author block of `html`: per creator `name ; mark ; mark | under: body ; body` (a mark `m→note` linked,
/// `m plain` not; a shared note's first copy shows its mark and prints its body in the margin, a marked one-author
/// note prints under the name with no mark on LaTeXML.css's card (a multi-line block, or a creator showing author lines
/// alone), else its glyph and its body as a pop-up, written as a note with a pop-up is, an unmarked note its text under the name with no glyph, a note with a
/// pop-up outside the name's wrapper its glyph and its body in the margin), then the block's `margin: body ; body`
/// (sorted); and the page's broken references: an id twice, an `href="#x"` with no `x`, a linked mark whose hidden copy
/// (`ltx_note_copy`, DG 2026-11-17) is missing, shown to assistive technology, in the tab order or not its note's body.
pub(super) fn page_lines(html: &str) -> (Vec<String>, Vec<String>) {
  let page = parse_page(html);
  let everything = descendants(&page, 0);
  let mut problems = Vec::new();
  let mut ids: Vec<&str> = everything
    .iter()
    .filter_map(|&e| page[e].attr("id"))
    .collect();
  ids.sort_unstable();
  let mut twice: Vec<&str> = ids
    .windows(2)
    .filter(|w| w[0] == w[1])
    .map(|w| w[0])
    .collect();
  twice.dedup();
  if !twice.is_empty() {
    problems.push(format!("ids twice: {}", twice.join(", ")));
  }
  let mut dangling: Vec<&str> = everything
    .iter()
    .filter(|&&e| page[e].tag == "a")
    .filter_map(|&e| page[e].attr("href")?.strip_prefix('#'))
    .filter(|target| ids.binary_search(target).is_err())
    .collect();
  dangling.sort_unstable();
  dangling.dedup();
  if !dangling.is_empty() {
    problems.push(format!("hrefs to no id: {}", dangling.join(", ")));
  }
  let blocks: Vec<usize> = everything
    .iter()
    .copied()
    .filter(|&e| page[e].has_class("ltx_authors"))
    .collect();
  let multiline = everything
    .iter()
    .any(|&e| page[e].has_class("ltx_document") && page[e].has_class("ltx_authors_multiline"));
  if blocks.is_empty() {
    return (vec!["(no author block)".to_string()], problems);
  }
  let body_of = |content: usize| {
    text_of(&page, content, &|e| {
      e.has_class("ltx_note_mark") || e.has_class("ltx_note_type") || e.has_class("ltx_tag_note")
    })
  };
  // Each linked mark: its glyph, the id it links to, its copy of that note.
  let mut copies: Vec<(String, &str, Option<usize>)> = Vec::new();
  let mut lines = Vec::new();
  for authors in blocks {
    let mut margin = Vec::new();
    for creator in descendants(&page, authors) {
      if !page[creator].has_class("ltx_creator") {
        continue;
      }
      let inside = descendants(&page, creator);
      // (LaTeXML.css's card: a one-author note under its name with no mark only where the author's lines show under it —
      // a multi-line block, or a creator whose contacts are all author lines or notes; elsewhere its mark shows and its
      // body is a pop-up, revMARKSAFEc S3)
      let card = multiline
        || (inside.iter().any(|&e| page[e].has_class("ltx_author_line"))
          && !inside.iter().any(|&e| {
            page[e].has_class("ltx_contact")
              && !["ltx_author_line", "ltx_role_note", "ltx_role_thanks"]
                .iter()
                .any(|class| page[e].has_class(class))
          }));
      let name = inside
        .iter()
        .find(|&&e| page[e].has_class("ltx_personname"))
        .map_or(String::new(), |&e| text_of(&page, e, &|_| false));
      let (mut marks, mut under) = (Vec::new(), Vec::new());
      let mut notes = Vec::new();
      notes_under(&page, creator, &mut notes);
      for note in notes {
        let wrapped = {
          let mut up = page[note].parent;
          let mut found = false;
          while let Some(e) = up.filter(|&e| e != creator) {
            found |= page[e].has_class("ltx_annotated_personname");
            up = page[e].parent;
          }
          found
        };
        let sup = page[note].children.iter().find_map(|child| match child {
          Child::Element(e) if page[*e].has_class("ltx_note_mark") => Some(*e),
          _ => None,
        });
        let glyph = sup.map_or(String::new(), |s| folded(&text_of(&page, s, &|_| false)));
        let linked = sup.is_some_and(|s| {
          descendants(&page, s).iter().any(|&a| {
            page[a].tag == "a" && page[a].attr("href").is_some_and(|h| h.starts_with('#'))
          })
        });
        // (a linked mark's copy of its note is hidden, so read as no body: the page prints the note once)
        let copy = descendants(&page, note)
          .into_iter()
          .find(|&e| page[e].has_class("ltx_note_copy"));
        let copied = copy.map_or(Vec::new(), |c| descendants(&page, c));
        let content = descendants(&page, note)
          .into_iter()
          .find(|&e| page[e].has_class("ltx_note_content") && !copied.contains(&e));
        if linked {
          let target = sup
            .and_then(|s| {
              descendants(&page, s)
                .into_iter()
                .find_map(|a| page[a].attr("href")?.strip_prefix('#'))
            })
            .unwrap_or("");
          copies.push((glyph.clone(), target, copy));
        }
        let Some(content) = content else {
          // (no glyph and no hidden body: an unmarked note's text, shown under the name, OD #990)
          if sup.is_none() {
            let body = text_of(&page, note, &|e| e.has_class("ltx_tag_note"));
            if !body.is_empty() {
              under.push(body);
            }
            continue;
          }
          marks.push(format!(
            "{glyph}{}",
            if linked { "→note" } else { " plain" }
          ));
          continue;
        };
        let body = body_of(content);
        if wrapped && page[note].has_class("ltx_note_shared") {
          marks.push(format!("{glyph}→note"));
          margin.push(body);
        } else if wrapped && card {
          under.push(body);
        } else {
          // (a note after the name's wrapper with a pop-up: its glyph too, which no unmarked note may show — Perl's `†`
          // fallback, revMARKSAFE B1)
          if !glyph.is_empty() {
            marks.push(format!("{glyph} plain"));
          }
          margin.push(body);
        }
      }
      let mut line = std::iter::once(name)
        .chain(marks)
        .collect::<Vec<_>>()
        .join(" ; ");
      if !under.is_empty() {
        line = format!("{line} | under: {}", under.join(" ; "));
      }
      lines.push(line);
    }
    if !margin.is_empty() {
      margin.sort();
      lines.push(format!("margin: {}", margin.join(" ; ")));
    }
  }
  for (glyph, target, copy) in copies {
    let Some(copy) = copy else {
      problems.push(format!("{glyph}→{target}: no copy"));
      continue;
    };
    let note = everything
      .iter()
      .copied()
      .find(|&e| page[e].attr("id") == Some(target));
    let body = |at: usize| {
      descendants(&page, at)
        .into_iter()
        .find(|&e| page[e].has_class("ltx_note_content"))
        .map(body_of)
    };
    if note.and_then(body) != body(copy) {
      problems.push(format!("{glyph}→{target}: the copy is not its note"));
    }
    if page[copy].attr("aria-hidden") != Some("true") {
      problems.push(format!("{glyph}→{target}: the copy is not aria-hidden"));
    }
    if descendants(&page, copy).iter().any(|&e| {
      page[e].tag == "a" && page[e].attr("href").is_some() && page[e].attr("tabindex") != Some("-1")
    }) {
      problems.push(format!(
        "{glyph}→{target}: a link of the copy is in the tab order"
      ));
    }
  }
  (lines, problems)
}

/// Which layers a test asserts: the GREEN ones (its moves empty) or the RED ones.
#[derive(Clone, Copy, PartialEq)]
enum Status {
  Green,
  Red,
}

/// The layers of `rows` with `status` that contradict their oracle (each what it wants and finds), and those of them
/// that match it.
fn check(rows: &[Row], status: Status) -> (Vec<String>, Vec<String>) {
  let wanted = |moves: &str| (status == Status::Green) == moves.is_empty();
  let (mut wrong, mut right) = (Vec::new(), Vec::new());
  for row in rows {
    let (xml_layer, html_layer) = (wanted(row.moves), wanted(row.page_moves));
    if !xml_layer && !html_layer {
      continue;
    }
    let (log, xml, ()) = convert_with_setup_then(row.tex, Some("ar5iv.sty"), || {}, |_| {});
    let mut verdict = |layer: &'static str, moves: &str, oracle: String, found: String| {
      if oracle == found {
        right.push(format!("{} ({layer})", row.repro));
      } else {
        let moves = if moves.is_empty() {
          String::new()
        } else {
          format!(" [{moves}]")
        };
        let report = format!(
          "{} {layer}{moves}\n  oracle: {oracle}\n  found:  {found}",
          row.repro
        );
        wrong.push(report);
      }
    };
    if xml_layer {
      let found = (error_count(&log), warning_count(&log), creator_lines(&xml));
      let oracle = (
        0,
        row.warnings,
        row
          .creators
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<_>>(),
      );
      verdict(
        "XML",
        row.moves,
        format!("{oracle:?}"),
        format!("{found:?}"),
      );
    }
    if html_layer {
      let found = page_lines(&page_of(&xml));
      let oracle = (
        row.page.iter().map(ToString::to_string).collect::<Vec<_>>(),
        Vec::<String>::new(),
      );
      verdict(
        "HTML",
        row.page_moves,
        format!("{oracle:?}"),
        format!("{found:?}"),
      );
    }
  }
  (wrong, right)
}

/// Every GREEN layer of `rows` matches its oracle.
#[track_caller]
pub(super) fn assert_green(rows: &[Row]) {
  let (wrong, right) = check(rows, Status::Green);
  assert!(
    wrong.is_empty(),
    "{} GREEN layers contradict their pdflatex oracle ({} match):\n{}",
    wrong.len(),
    right.len(),
    wrong.join("\n")
  );
}

/// The RED layers of `rows`: prints those now matching their oracle (to promote), fails while any is still red.
#[track_caller]
fn assert_red_layers_now_green(rows: &[Row]) {
  let (red, now_green) = check(rows, Status::Red);
  for layer in &now_green {
    println!("now GREEN, promote: {layer}");
  }
  assert!(
    red.is_empty(),
    "{} RED layers still contradict their oracle:\n{}",
    red.len(),
    red.join("\n")
  );
}

/// The rows of [`shared_notes_are_a_marked_copy_per_carrier`] and its `_red` twin.
const SHARED: &[Row] = &[
  // (a symbol legend is the names' that type it, not a `\thanks` of the same glyph on another name; ADVR2 finding 4)
  Row {
    repro:      "author_typed_star_legend_is_the_typing_names",
    tex:        repro!("author_typed_star_legend_is_the_typing_names"),
    warnings:   0,
    creators:   &[
      r#"Zed Zorn ; thanks[*]=Work done at X."#,
      r#"Bob Baker ; thanks[*]=Equal contribution"#,
      r#"Cy Cole ; thanks[*]=Equal contribution"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Zed Zorn ; * plain"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; *→note"#,
      r#"margin: Equal contribution ; Work done at X."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "authblk_symbol_legend_shared_note_copies_carry_the_mark",
    tex:        repro!("authblk_symbol_legend_shared_note_copies_carry_the_mark"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=These authors contributed equally to this work. | affiliation=Univ A, Town"#,
      r#"Bob Baker ; thanks[*]=These authors contributed equally to this work. | affiliation=Univ A, Town"#,
      r#"Cat Cole | affiliation=Univ A, Town"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cat Cole"#,
      r#"margin: These authors contributed equally to this work."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "author_block_symbol_legend_note_mark_not_in_its_text",
    tex:        repro!("author_block_symbol_legend_note_mark_not_in_its_text"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution | affiliation=Univ A, Town"#,
      r#"Bob Baker ; thanks[*]=Equal contribution | affiliation=Univ A, Town"#,
      r#"Cat Cole | affiliation=Univ A, Town"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cat Cole"#,
      r#"margin: Equal contribution"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "author_block_three_symbol_legend_notes_carry_their_marks",
    tex:        repro!("author_block_three_symbol_legend_notes_carry_their_marks"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[*]=Equal contribution | affiliation=Univ B"#,
      r#"Cat Cole ; thanks[+]=Core contributor | affiliation=Univ B"#,
      r#"Dan Dorn ; thanks[+]=Core contributor | affiliation=Univ A"#,
      r#"Eve Ezra ; thanks[†]=Corresponding author | affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cat Cole ; +→note"#,
      r#"Dan Dorn ; +→note"#,
      r#"Eve Ezra | under: Corresponding author"#,
      r#"margin: Core contributor ; Equal contribution"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ieee_thanks_symbol_legend_note_mark_lifted",
    tex:        repro!("ieee_thanks_symbol_legend_note_mark_lifted"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution | affiliation=Univ A, Town"#,
      r#"Bob Baker ; thanks[*]=Equal contribution | affiliation=Univ A, Town"#,
      r#"Cat Cole ; thanks[†]=Corresponding author: Cat Cole | affiliation=Univ A, Town"#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; *→note",
      "Bob Baker ; *→note",
      "Cat Cole ; † plain",
      "margin: Corresponding author: Cat Cole ; Equal contribution",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "emit_authblk_label_legend_is_its_carriers_note",
    tex:        repro!("emit_authblk_label_legend_is_its_carriers_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Corresponding author. | affiliation=Univ A ; email=ann@a.org"#,
      r#"Bob Baker ; thanks[†]=These authors contributed equally to this work. | affiliation=Univ A"#,
      r#"Cat Cole ; thanks[†]=These authors contributed equally to this work. | affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; * plain",
      "Bob Baker ; †→note",
      "Cat Cole ; †→note",
      "margin: Corresponding author. ; These authors contributed equally to this work.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "legend_kind_authblk_labelled_legends",
    tex:        repro!("legend_kind_authblk_labelled_legends"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[†]=These authors contributed equally. | affiliation=Univ A, Town"#,
      r#"Bob Baker ; thanks[*]=Corresponding author ; thanks[†]=These authors contributed equally. | affiliation=Univ A, Town ; email=bob@a.edu"#,
      r#"Cat Cole | affiliation=Univ B, City"#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; †→note",
      "Bob Baker ; †→note ; * plain",
      "Cat Cole",
      "margin: Corresponding author ; These authors contributed equally.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ieee_refmark_mixed_with_digit_marks",
    tex:        repro!("ieee_refmark_mixed_with_digit_marks"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution | affiliation=Univ A, City A"#,
      r#"Bob Bell ; thanks[*]=Equal contribution | affiliation=Univ A, City A ; affiliation=Univ B, City B"#,
      r#"Cat Cole ; thanks[†]=Corresponding author: Cat Cole cat@c.edu | affiliation=Univ C, City C"#,
      r#"Dan Dorn | affiliation=Univ B, City B"#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; *→note",
      "Bob Bell ; *→note",
      "Cat Cole ; † plain",
      "Dan Dorn",
      "margin: Corresponding author: Cat Cole cat@c.edu ; Equal contribution",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "revtex_frontmatter_footnote_on_two_authors",
    tex:        repro!("revtex_frontmatter_footnote_on_two_authors"),
    warnings:   0,
    creators:   &[
      r#"Ann Author ; thanks[*]=These two authors contributed equally to this work. | email=a@x.org ; affiliation=Univ A"#,
      r#"Bob Author | email=b@x.org ; affiliation=Univ B"#,
      r#"Cat Author ; thanks[*]=These two authors contributed equally to this work. | email=c@x.org ; affiliation=Univ C"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Author ; *→note"#,
      r#"Bob Author"#,
      r#"Cat Author ; *→note"#,
      r#"margin: These two authors contributed equally to this work."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsarticle_fntext_shared_by_two_names_is_their_marked_note",
    tex:        repro!("elsarticle_fntext_shared_by_two_names_is_their_marked_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Corresponding authors. ; thanks[1]=These authors contributed equally. | address=Univ A"#,
      r#"Bob Baker ; thanks[*]=Corresponding authors. ; thanks[1]=These authors contributed equally. | address=Univ A"#,
      r#"Cat Cole | note=Visiting Univ B. ; address=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; 1→note ; *→note"#,
      r#"Bob Baker ; 1→note ; *→note"#,
      r#"Cat Cole"#,
      r#"margin: Corresponding authors. ; These authors contributed equally."#,
    ],
    page_moves: r#"row R MARKORDER (the notes' document order is not the order the PDF prints their marks; DG 2026-11-16: minor, each mark is on its carrier)"#,
  },
  Row {
    repro:      "elsarticle_list_correspondence_item_is_its_names_note",
    tex:        repro!("elsarticle_list_correspondence_item_is_its_names_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[*]=Corresponding author: Univ A, Town. | affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able",
      "Bob Baker ; * plain",
      "margin: Corresponding author: Univ A, Town.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "list_note_led_by_the_names_mark_is_their_note",
    tex:        repro!("list_note_led_by_the_names_mark_is_their_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[‡]=These authors contributed equally. | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[‡]=These authors contributed equally. | affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; ‡→note"#,
      r#"Bob Baker ; ‡→note"#,
      r#"margin: These authors contributed equally."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "list_note_plain_text_is_the_blocks_note",
    tex:        repro!("list_note_plain_text_is_the_blocks_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Univ A , C"#,
      r#"Bob Baker | affiliation=Univ B , C"#,
      r#" ; note[]=Both authors are with the Lab."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker"#,
      r#" | under: Both authors are with the Lab."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_legend_dollar_mark_and_typed_star",
    tex:        repro!("fmmarks_legend_dollar_mark_and_typed_star"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[$]=these authors contributed equally to this work | affiliation=Univ A, Town ; affiliation=Univ B, City"#,
      r#"Bob Baker ; thanks[$]=these authors contributed equally to this work | affiliation=Univ A, Town"#,
      r#"Cat Cole | affiliation=Univ B, City"#,
      r#"Dan Dorn ; thanks[*]=corresponding author | affiliation=Univ A, Town"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; $→note"#,
      r#"Bob Baker ; $→note"#,
      r#"Cat Cole"#,
      r#"Dan Dorn | under: corresponding author"#,
      r#"margin: these authors contributed equally to this work"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_elsarticle_shared_fnref_and_unreferenced_fntext",
    tex:        repro!("fmmarks_elsarticle_shared_fnref_and_unreferenced_fntext"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[1]=Equal. | address=Univ A"#,
      r#"Bob Baker ; thanks[1]=Equal. | address=Univ A"#,
      r#" | note=Unreferenced note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; 1→note"#,
      r#"Bob Baker ; 1→note"#,
      r#""#,
      r#"margin: Equal."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_authblk_double_star_label_and_date_note_red",
    tex:        repro!("fmmarks_authblk_double_star_label_and_date_note_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=These authors contributed equally to this work. | affiliation=Univ A, Town ; email=ann@a.edu"#,
      r#"Bob Baker ; thanks[*]=These authors contributed equally to this work. | affiliation=Univ A, Town"#,
      r#"Cat Cole | affiliation=Univ A, Town ; affiliation=Present address: Lab B, City"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cat Cole"#,
      r#"margin: These authors contributed equally to this work."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "list_note_with_two_marks_is_the_blocks_note",
    tex:        repro!("list_note_with_two_marks_is_the_blocks_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= | affiliation=Univ A"#,
      r#"Bob Baker ; footnotemark[*]= | affiliation=Univ A"#,
      r#" ; note[]=† Equal. ∗ Corresponding."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; † plain"#,
      r#"Bob Baker ; * plain"#,
      r#" | under: † Equal. ∗ Corresponding."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ieee_refmark_beside_a_starred_legend",
    tex:        repro!("ieee_refmark_beside_a_starred_legend"),
    warnings:   2,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution | affiliation=Univ A"#,
      r#"Bob Bell | affiliation=Univ B"#,
    ],
    moves:      r#"PURE (fix/pure-personname: the marks out of the personname); the refmark label leading an affiliation or legend text"#,
    page:       &[
      "Ann Able ; * plain",
      "Bob Bell",
      "margin: Equal contribution",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_ieee_blockn_typed_star_thanks_red",
    tex:        repro!("fmmarks_ieee_blockn_typed_star_thanks_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal Contribution | affiliation=Univ A ; email=ann@a.edu"#,
      r#"Bob Baker ; thanks[*]=Equal Contribution | affiliation=Univ A ; email=bob@a.edu"#,
      r#"Cat Cole | affiliation=Univ B ; email=cat@b.edu"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cat Cole"#,
      r#"margin: Equal Contribution"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_thanks_holds_a_second_legend_red",
    tex:        repro!("fmmarks_thanks_holds_a_second_legend_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= ; thanks[*]=Corresponding Author. † Co-first authors | affiliation=Univ A"#,
      r#"Bob Baker ; footnotemark[†]= | affiliation=Univ A"#,
      r#"Cat Cole | affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; † plain | under: Corresponding Author. † Co-first authors"#,
      r#"Bob Baker ; † plain"#,
      r#"Cat Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_long_shared_note_with_second_marks",
    tex:        repro!("fmmarks_long_shared_note_with_second_marks"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution. Listing order is random. Author A designed part 1 of the model and ran the experiments of section 1. Author B designed part 2 of the model and ran the experiments of section 2. Author C designed part 3 of the model and ran the experiments of section 3. Author D designed part 4 of the model and ran the experiments of section 4. Author E designed part 5 of the model and ran the experiments of section 5. Author F designed part 6 of the model and ran the experiments of section 6. Author G designed part 7 of the model and ran the experiments of section 7. Author H designed part 8 of the model and ran the experiments of section 8. | affiliation=Lab A ; email=ann@a.org"#,
      r#"Bob Baker ; thanks[*]=Equal contribution. Listing order is random. Author A designed part 1 of the model and ran the experiments of section 1. Author B designed part 2 of the model and ran the experiments of section 2. Author C designed part 3 of the model and ran the experiments of section 3. Author D designed part 4 of the model and ran the experiments of section 4. Author E designed part 5 of the model and ran the experiments of section 5. Author F designed part 6 of the model and ran the experiments of section 6. Author G designed part 7 of the model and ran the experiments of section 7. Author H designed part 8 of the model and ran the experiments of section 8. | affiliation=Lab A ; email=bob@a.org"#,
      r#"Cat Cole ; thanks[*]=Equal contribution. Listing order is random. Author A designed part 1 of the model and ran the experiments of section 1. Author B designed part 2 of the model and ran the experiments of section 2. Author C designed part 3 of the model and ran the experiments of section 3. Author D designed part 4 of the model and ran the experiments of section 4. Author E designed part 5 of the model and ran the experiments of section 5. Author F designed part 6 of the model and ran the experiments of section 6. Author G designed part 7 of the model and ran the experiments of section 7. Author H designed part 8 of the model and ran the experiments of section 8. ; thanks[†]=Work performed while at Lab A. | affiliation=Univ C ; email=cat@c.edu"#,
      r#"Dan Dorn ; thanks[*]=Equal contribution. Listing order is random. Author A designed part 1 of the model and ran the experiments of section 1. Author B designed part 2 of the model and ran the experiments of section 2. Author C designed part 3 of the model and ran the experiments of section 3. Author D designed part 4 of the model and ran the experiments of section 4. Author E designed part 5 of the model and ran the experiments of section 5. Author F designed part 6 of the model and ran the experiments of section 6. Author G designed part 7 of the model and ran the experiments of section 7. Author H designed part 8 of the model and ran the experiments of section 8. ; thanks[‡]=Work performed while at Lab B. | affiliation=Lab B ; email=dan@b.org"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cat Cole ; *→note | under: Work performed while at Lab A."#,
      r#"Dan Dorn ; *→note | under: Work performed while at Lab B."#,
      r#"margin: Equal contribution. Listing order is random. Author A designed part 1 of the model and ran the experiments of section 1. Author B designed part 2 of the model and ran the experiments of section 2. Author C designed part 3 of the model and ran the experiments of section 3. Author D designed part 4 of the model and ran the experiments of section 4. Author E designed part 5 of the model and ran the experiments of section 5. Author F designed part 6 of the model and ran the experiments of section 6. Author G designed part 7 of the model and ran the experiments of section 7. Author H designed part 8 of the model and ran the experiments of section 8."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_many_names_two_legend_marks",
    tex:        repro!("fmmarks_many_names_two_legend_marks"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution | affiliation=Univ A, Town"#,
      r#"Bob Baker ; thanks[*]=Equal contribution | affiliation=Univ A, Town"#,
      r#"Cat Cole | affiliation=Univ B, City"#,
      r#"Dan Dorn ; thanks[†]=Corresponding author | affiliation=Lab C, Village"#,
      r#"Eve Ezra | affiliation=Univ A, Town"#,
      r#"Fay Fox | affiliation=Univ A, Town"#,
      r#"Gil Gray | affiliation=Univ A, Town"#,
      r#"Hal Hunt | affiliation=Univ A, Town"#,
      r#"Ida Irwin | affiliation=Univ A, Town"#,
      r#"Jay Judd | affiliation=Univ B, City"#,
      r#"Kim Kent | affiliation=Univ A, Town ; affiliation=Lab C, Village"#,
      r#"Lee Lord | affiliation=Lab C, Village"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cat Cole"#,
      r#"Dan Dorn | under: Corresponding author"#,
      r#"Eve Ezra"#,
      r#"Fay Fox"#,
      r#"Gil Gray"#,
      r#"Hal Hunt"#,
      r#"Ida Irwin"#,
      r#"Jay Judd"#,
      r#"Kim Kent"#,
      r#"Lee Lord"#,
      r#"margin: Equal contribution"#,
    ],
    page_moves: r#""#,
  },
];
/// Shared notes (ruling 1): a legend or note several names print is a full copy on each carrier, every copy marked,
/// printed once in the margin — authblk symbol labels (2606.00446), the hand-typed legend lines of the kernel block
/// (2407.07614, 1910.10432, 2602.18802), IEEEtran `\thanks{$^{*}$…}` and refmarks (2312.10807), the long shared note
/// (1706.03762), REVTeX's joined footnote, elsarticle `\fnref`/`\fntext` (2102.07817), a class's title-page list
/// (2606.06621).
#[test]
fn shared_notes_are_a_marked_copy_per_carrier() { assert_green(SHARED); }
/// The known gaps of [`shared_notes_are_a_marked_copy_per_carrier`].
#[test]
#[ignore = "RED: the row R lines its rows name"]
fn shared_notes_are_a_marked_copy_per_carrier_red() { assert_red_layers_now_green(SHARED); }

/// The rows of [`name_marks_live_outside_the_personname`] and its `_red` twin.
const PURE: &[Row] = &[
  Row {
    repro:      "fmmarks_pure_hash_in_name",
    tex:        repro!("fmmarks_pure_hash_in_name"),
    warnings:   0,
    creators:   &[r#"The C# Team"#],
    moves:      r#""#,
    page:       &[r#"The C# Team"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "author_unmatched_numeric_mark",
    tex:        repro!("author_unmatched_numeric_mark"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Bee ; footnotemark[1]="#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &["Ann Able ; * plain", "Bob Bee ; 1 plain"],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_pure_href_marks_red",
    tex:        repro!("fmmarks_pure_href_marks_red"),
    warnings:   0,
    creators:   &[r#"Ann Able ; footnotemark[*]="#, r#"Bob Baker | url=†"#],
    moves:      r#""#,
    // (the inked mark shows: every unlinked mark does, OD #1059's fail-safe, DG "hide only when certain")
    page:       &["Ann Able ; * plain", "Bob Baker"],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_pure_isotope_collaboration_name_red",
    tex:        repro!("fmmarks_pure_isotope_collaboration_name_red"),
    warnings:   0,
    creators:   &[r#"3 He Collaboration"#],
    moves:      r#"row R (PURE residual: `$^{3}$He Collaboration`, the reader splits the name and drops the 3)"#,
    page:       &[r#"3 He Collaboration"#],
    page_moves: r#"the XML layer's: row R (PURE residual: `$^{3}$He Collaboration`, the reader splits the name and drops the 3)"#,
  },
  Row {
    repro:      "fmmarks_pure_leading_digit_marks_red",
    tex:        repro!("fmmarks_pure_leading_digit_marks_red"),
    warnings:   1,
    creators:   &[
      r#"Ann Able ; footnotemark[1]="#,
      r#"Bob Baker ; footnotemark[2]="#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able ; 1 plain"#, r#"Bob Baker ; 2 plain"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_pure_letter_mark_and_note_opening_word_red",
    tex:        repro!("fmmarks_pure_letter_mark_and_note_opening_word_red"),
    warnings:   0,
    creators:   &[r#"Ann Able ; footnotemark[a]= ; thanks[*]=a visitor at the Institute."#],
    moves:      r#""#,
    page:       &[
      "Ann Able ; a plain ; * plain",
      "margin: a visitor at the Institute.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_star_sup_beside_thanks_body_legend_red",
    tex:        repro!("fmmarks_star_sup_beside_thanks_body_legend_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; thanks[*]=Email: ann@x.org"#,
      r#"Bob Baker ; footnotemark[*]= ; thanks[†]=Email: bob@x.org"#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; * plain ; * plain",
      "Bob Baker ; * plain ; † plain",
      "margin: Email: ann@x.org ; Email: bob@x.org",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2c_mark_after_math_comma",
    tex:        repro!("ne2c_mark_after_math_comma"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; endnotemark[1]="#,
      r#"Bob Baker ; endnotemark[1]= ; endnotemark[2]="#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able ; 1 plain"#, r#"Bob Baker ; 1 plain ; 2 plain"#],
    page_moves: r#"the XML layer's: PURE (fix/pure-personname: the marks out of the personname)"#,
  },
  Row {
    repro:      "ne2e_mark_in_formula_past_thin_spaces",
    tex:        repro!("ne2e_mark_in_formula_past_thin_spaces"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; footnotemark[1]= ; thanks[*]=X"#,
      r#"Bob Baker"#,
    ],
    moves:      r#"PURE (fix/pure-personname: the marks out of the personname); row R (a note inside a formula: the in-formula `\footnotemark[1]` counted lost)"#,
    page:       &[
      r#"Ann Able ; 1 plain ; *→note ; *→note"#,
      r#"Bob Baker"#,
      r#"margin: X"#,
    ],
    page_moves: r#"the XML layer's: PURE (fix/pure-personname: the marks out of the personname); row R (a note inside a formula: the in-formula `\footnotemark[1]` counted lost)"#,
  },
  // (PUREc: an author reference mark's number keys the one mark it printed — 2508.13181's typed §, a typed ∗ or †
  // beside it, an unheld second refmark, ieeeconf's `\authorrefmark` run of 2011.00594; acmart's counter-marked
  // `\authornote`; `C\#` at a name's end; glued digits one number beside a two-digit entry; a note's mark answering
  // one printed mark)
  Row {
    repro:      "pure_names/pure_name_refmark_beside_a_typed_section_mark",
    tex:        repro!("pure_names/pure_name_refmark_beside_a_typed_section_mark"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[§]= | affiliation=† Lab B ; affiliation=* Univ A"#,
      r#"Bob Baker ; footnotemark[§]= | affiliation=* Univ A"#,
      r#"Cy Cole | affiliation=* Univ A"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; § plain"#,
      r#"Bob Baker ; § plain"#,
      r#"Cy Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "pure_names/pure_name_refmark_beside_typed_marks",
    tex:        repro!("pure_names/pure_name_refmark_beside_typed_marks"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= | affiliation=* Univ A"#,
      r#"Bob Baker ; footnotemark[†]= | affiliation=* Univ A"#,
      r#"Cy Cole | affiliation=* Univ A"#,
    ],
    moves:      r#""#,
    // (the inked mark shows: every unlinked mark does, OD #1059's fail-safe, DG "hide only when certain")
    page:       &["Ann Able ; * plain", "Bob Baker ; † plain", "Cy Cole"],
    page_moves: r#""#,
  },
  Row {
    repro:      "pure_names/pure_name_two_refmarks_one_held",
    tex:        repro!("pure_names/pure_name_two_refmarks_one_held"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= | affiliation=* Univ A"#,
      r#"Bob Baker | affiliation=* Univ A"#,
    ],
    moves:      r#""#,
    // (the inked mark shows: every unlinked mark does, OD #1059's fail-safe, DG "hide only when certain")
    page:       &["Ann Able ; † plain", "Bob Baker"],
    page_moves: r#""#,
  },
  Row {
    repro:      "pure_names/pure_name_authorrefmark_comma_run_one_held",
    tex:        repro!("pure_names/pure_name_authorrefmark_comma_run_one_held"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= | affiliation=* Univ A"#,
      r#"Bob Baker | affiliation=* Univ A"#,
    ],
    moves:      r#""#,
    // (the inked mark shows: every unlinked mark does, OD #1059's fail-safe, DG "hide only when certain")
    page:       &["Ann Able ; † plain", "Bob Baker"],
    page_moves: r#""#,
  },
  Row {
    repro:      "pure_names/pure_name_acmart_authornote_beside_a_typed_star",
    tex:        repro!("pure_names/pure_name_acmart_authornote_beside_a_typed_star"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; thanks[*]=Equal. | affiliation=Univ A , X"#,
      r#"Bob Baker | affiliation=Univ B , X"#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; * plain ; * plain",
      "Bob Baker",
      "margin: Equal.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "pure_names/pure_name_hash_at_a_names_end",
    tex:        repro!("pure_names/pure_name_hash_at_a_names_end"),
    warnings:   0,
    creators:   &[r#"Team C#"#, r#"Bob Baker ; footnotemark[*]="#],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &["Team C#", "Bob Baker ; * plain"],
    page_moves: r#""#,
  },
  Row {
    repro:      "pure_names/pure_name_glued_digits_one_number",
    tex:        repro!("pure_names/pure_name_glued_digits_one_number"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[12]= | affiliation=Univ A ; affiliation=Univ B"#,
      r#"Bob Baker | affiliation=Univ J"#,
    ],
    moves:      r#""#,
    // (the inked mark shows: every unlinked mark does, OD #1059's fail-safe, DG "hide only when certain")
    page:       &["Ann Able ; 12 plain", "Bob Baker"],
    page_moves: r#""#,
  },
  Row {
    repro:      "pure_names/pure_name_refmark_legend_note_beside_a_typed_mark",
    tex:        repro!("pure_names/pure_name_refmark_legend_note_beside_a_typed_mark"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=† Univ B"#,
      r#"Bob Baker ; footnotemark[§]= ; thanks[§]=Equal contribution. | affiliation=† Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able",
      "Bob Baker ; §→note ; § plain",
      "margin: Equal contribution.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "pure_names/pure_name_footnotemark_beside_a_typed_dagger_red",
    tex:        repro!("pure_names/pure_name_footnotemark_beside_a_typed_dagger_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=First note."#,
      r#"Bob Baker ; thanks[†]=Second note."#,
      r#"Cat Cole ; footnotemark[†]= ; thanks[†]=Second note."#,
      r#"Dan Dorn ; footnotemark[†]= ; thanks[†]=Second note."#,
    ],
    moves:      r#"row R MARKKEY (a typed mark beside a `\footnotemark` of the same glyph: the reader's glyph-as-label request takes it for the note's, resolver/blocks.rs:1279-1287)"#,
    page:       &[
      r#"Ann Able | under: First note."#,
      r#"Bob Baker ; †→note"#,
      r#"Cat Cole ; †→note ; † plain"#,
      r#"Dan Dorn ; † plain ; †→note"#,
      r#"margin: Second note."#,
    ],
    page_moves: r#"the XML layer's: row R MARKKEY (a typed mark beside a `\footnotemark` of the same glyph)"#,
  },
  // (PUREd: a contact answers one printed mark across the name's mark runs — a number and the glyph its reference mark
  // printed are one mark —, whatever ink parts the runs: an ORCID icon, a graphic, a formula; acmart's
  // `\authornotemark[n]` copy is marked by its counter, as `\authornote` is)
  Row {
    repro:      "pure_names/pure_name_orcid_between_typed_star_and_refmark",
    tex:        repro!("pure_names/pure_name_orcid_between_typed_star_and_refmark"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= | affiliation=* Univ A"#,
      r#"Bob Baker | affiliation=* Univ A"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able ; * plain"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "pure_names/pure_name_graphic_between_typed_star_and_refmark",
    tex:        repro!("pure_names/pure_name_graphic_between_typed_star_and_refmark"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= | affiliation=* Univ A"#,
      r#"Bob Baker ♡ ; footnotemark[*]= | affiliation=* Univ A"#,
      r#"Cy Cole | affiliation=* Univ A"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ♡ ; * plain"#,
      r#"Cy Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "pure_names/pure_name_acmart_authornotemark_beside_a_typed_star",
    tex:        repro!("pure_names/pure_name_acmart_authornotemark_beside_a_typed_star"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal. | affiliation=Univ A , X"#,
      r#"Bob Baker ; footnotemark[*]= ; thanks[*]=Equal. | affiliation=Univ B , X"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; * plain ; *→note"#,
      r#"margin: Equal."#,
    ],
    page_moves: r#""#,
  },
];
/// Name marks re-homed (DG 2026-11-16, personname purity): a mark typed on a name leaves the personname — the note it
/// answers, the contact it labels or a mark-only note — while name ink (`The C\# Team`, `Team C\#`, `$^{3}$He`) stays;
/// a reference mark's number keys only the mark it printed; a note or a contact answers one printed mark across the
/// name's runs (revPURE, revPUREc; 2406.11149, 2506.02863, 0707.3527, 0801.1438, 2508.13181, 2011.00594).
#[test]
fn name_marks_live_outside_the_personname() { assert_green(PURE); }
/// The known gaps of [`name_marks_live_outside_the_personname`].
#[test]
#[ignore = "RED: the row R lines its rows name"]
fn name_marks_live_outside_the_personname_red() { assert_red_layers_now_green(PURE); }

/// The rows of [`a_mark_on_a_name_is_a_copy_of_its_note`] and its `_red` twin.
const LINK: &[Row] = &[
  // (a typed `$^*$` links to the `\thanks` printing the same ∗, its mark `*`: the page's keys fold them; 1705.05427)
  Row {
    repro:      "author_typed_star_links_the_thanks_note",
    tex:        repro!("author_typed_star_links_the_thanks_note"),
    warnings:   0,
    creators:   &[
      r#"Kareem Amin ; thanks[*]=Equal contribution."#,
      r#"Nan Jiang ; footnotemark[*]="#,
      r#"Dee Dorn"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Kareem Amin ; *→note"#,
      r#"Nan Jiang ; *→note"#,
      r#"Dee Dorn"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  // (a typed `$^\ast$` whose legend opens the name's own `\thanks` links to no other name's ∗ note; 2607.28158)
  Row {
    repro:      "author_typed_star_legend_in_own_thanks_links_nothing",
    tex:        repro!("author_typed_star_legend_in_own_thanks_links_nothing"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Email: ann@x.org | affiliation=Univ A"#,
      r#"Bob Baker ; footnotemark[*]= ; thanks[†]=∗ Corresponding author. | affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; * plain ; † plain"#,
      r#"margin: Email: ann@x.org ; ∗ Corresponding author."#,
    ],
    page_moves: r#""#,
  },
  // (a legend that is a name's own `\thanks` stays that name's note beside the names typing its ∗: 1904.09324, 2204.06136)
  Row {
    repro:      "author_typed_star_legend_in_a_names_own_thanks_keeps_its_note",
    tex:        repro!("author_typed_star_legend_in_a_names_own_thanks_keeps_its_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=The first two authors contributed equally. All authors are with Univ X, cy@x.org."#,
      r#"Bob Baker ; thanks[*]=The first two authors contributed equally. All authors are with Univ X, cy@x.org."#,
      r#"Cy Cole ; thanks[*]=The first two authors contributed equally. All authors are with Univ X, cy@x.org."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; *→note"#,
      r#"margin: The first two authors contributed equally. All authors are with Univ X, cy@x.org."#,
    ],
    page_moves: r#""#,
  },
  // (and the note's own name though it shows a mark of its own: the fallback's other carriers still show none)
  Row {
    repro:      "author_typed_star_legend_in_a_marked_names_own_thanks_keeps_its_note",
    tex:        repro!("author_typed_star_legend_in_a_marked_names_own_thanks_keeps_its_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution. Cy is cy@x.org. | affiliation=Univ X"#,
      r#"Bob Baker ; thanks[*]=Equal contribution. Cy is cy@x.org. | affiliation=Univ X"#,
      r#"Cy Cole ; thanks[*]=Equal contribution. Cy is cy@x.org. | affiliation=Univ Y"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; *→note"#,
      r#"margin: Equal contribution. Cy is cy@x.org."#,
    ],
    page_moves: r#""#,
  },
  // (authblk's twin: Cy keeps his note; authblk's reading keeps each typed ∗ in the name beside the copy it requests)
  Row {
    repro:      "authblk_typed_star_legend_in_a_names_own_thanks_keeps_its_note",
    tex:        repro!("authblk_typed_star_legend_in_a_names_own_thanks_keeps_its_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=The first two authors contributed equally. All authors are with Univ X, cy@x.org."#,
      r#"Bob Baker ; thanks[*]=The first two authors contributed equally. All authors are with Univ X, cy@x.org."#,
      r#"Cy Cole ; thanks[*]=The first two authors contributed equally. All authors are with Univ X, cy@x.org."#,
    ],
    moves:      r#"row R AUTHBLKSTAR (authblk keeps a typed ∗ in the name, `\lx@frontmatter@keepsup`, beside the copy of the note it requests: two marks)"#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; *→note"#,
      r#"margin: The first two authors contributed equally. All authors are with Univ X, cy@x.org."#,
    ],
    page_moves: r#"row R AUTHBLKSTAR (authblk keeps a typed ∗ in the name, `\lx@frontmatter@keepsup`, beside the copy of the note it requests: two marks)"#,
  },
  // (a typed ∗ whose legend is the `\date` links to no `\thanks`: the fold's legend exclusion is the title page's)
  Row {
    repro:      "author_typed_star_legend_in_a_title_tabular_links_nothing",
    tex:        repro!("author_typed_star_legend_in_a_title_tabular_links_nothing"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker ; footnotemark[*]="#,
      r#"Cy Cole ; thanks[*]=Corresponding author: cy@x.org"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; * plain"#,
      r#"Cy Cole ; * plain"#,
      r#"margin: Corresponding author: cy@x.org"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "author_typed_star_legend_in_a_date_thanks_links_nothing",
    tex:        repro!("author_typed_star_legend_in_a_date_thanks_links_nothing"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker ; footnotemark[*]="#,
      r#"Cy Cole ; thanks[*]=Corresponding author, cy@x.org."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; * plain"#,
      r#"Cy Cole ; * plain"#,
      r#"margin: Corresponding author, cy@x.org."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "author_typed_star_legend_in_the_date_links_nothing",
    tex:        repro!("author_typed_star_legend_in_the_date_links_nothing"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution."#,
      r#"Bob Baker ; thanks[*]=Equal contribution."#,
      r#"Cy Cole ; thanks[*]=Corresponding author: cy@x.org"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; * plain"#,
      r#"margin: Corresponding author: cy@x.org ; Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  // (a body paragraph opening with ∗ is a legend too (the abstract's alike))
  Row {
    repro:      "author_typed_star_legend_in_a_body_paragraph_links_nothing",
    tex:        repro!("author_typed_star_legend_in_a_body_paragraph_links_nothing"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Corresponding author."#,
      r#"Bob Baker ; footnotemark[*]="#,
      r#"Cy Cole ; footnotemark[†]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; * plain"#,
      r#"Cy Cole ; † plain"#,
      r#"margin: Corresponding author."#,
    ],
    page_moves: r#""#,
  },
  // (a body footnote printing ∗ is a second note of the glyph: no link)
  Row {
    repro:      "author_typed_star_beside_a_body_fnsymbol_footnote_links_nothing",
    tex:        repro!("author_typed_star_beside_a_body_fnsymbol_footnote_links_nothing"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker ; thanks[*]=Supported by grant 42."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; * plain"#,
      r#"margin: Supported by grant 42."#,
    ],
    page_moves: r#""#,
  },
  // (a body note's TeX mark `$\ast$` reads as ∗)
  Row {
    repro:      "author_typed_star_beside_a_body_ast_footnotetext_links_nothing",
    tex:        repro!("author_typed_star_beside_a_body_ast_footnotetext_links_nothing"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker ; footnotemark[*]="#,
      r#"Cy Cole ; thanks[*]=Corresponding author."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; * plain"#,
      r#"Cy Cole ; * plain"#,
      r#"margin: Corresponding author."#,
    ],
    page_moves: r#""#,
  },
  // (but a body footnote printing ∗ in a section is no legend: the title page alone is read)
  Row {
    repro:      "author_typed_star_links_the_thanks_beside_a_section_footnote",
    tex:        repro!("author_typed_star_links_the_thanks_beside_a_section_footnote"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution."#,
      r#"Bob Baker ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  // (an authblk label no `\affil` explains, `\author[1,*]`, is a binding's label: the ∗/* fold never links it to a
  // `\thanks` — authblk prints a text `*`, the `\thanks` its ∗; the INVDAG review's variant of 1705.05427)
  Row {
    repro:      "authblk_declared_star_label_links_no_thanks",
    tex:        repro!("authblk_declared_star_label_links_no_thanks"),
    warnings:   0,
    creators:   &[
      r#"Ann Alpha ; thanks[*]=x | affiliation=Univ One"#,
      r#"Cy Cole ; footnotemark[*]= | affiliation=Univ One"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Alpha ; * plain"#,
      r#"Cy Cole ; * plain"#,
      r#"margin: x"#,
    ],
    page_moves: r#""#,
  },
  // (a `\samethanks` beside a typed legend of its glyph is a copy of the footnote its number names; ADVR2 finding 5)
  Row {
    repro:      "author_samethanks_beside_typed_star_legend",
    tex:        repro!("author_samethanks_beside_typed_star_legend"),
    warnings:   0,
    creators:   &[
      r#"Zed Zorn ; thanks[*]=Equal contribution"#,
      r#"Ann Abel ; thanks[*]=Corresponding author."#,
      r#"Cy Cole ; thanks[*]=Corresponding author."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Zed Zorn ; * plain"#,
      r#"Ann Abel ; *→note"#,
      r#"Cy Cole ; *→note"#,
      r#"margin: Corresponding author. ; Equal contribution"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "author_samethanks_is_a_copy_of_the_marked_note",
    tex:        repro!("author_samethanks_is_a_copy_of_the_marked_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal Contribution. | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[*]=Equal Contribution. | affiliation=Univ A"#,
      r#"Cat Cole | affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cat Cole"#,
      r#"margin: Equal Contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "author_samethanks_mark_is_the_same_note",
    tex:        repro!("author_samethanks_mark_is_the_same_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution."#,
      r#"Bob Baker ; thanks[*]=Equal contribution."#,
      r#"Cat Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cat Cole"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "acmart_authornotemark_is_a_copy_of_the_marked_note",
    tex:        repro!("acmart_authornotemark_is_a_copy_of_the_marked_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution. | affiliation=Univ A , Town , X"#,
      r#"Bob Baker ; thanks[*]=Equal contribution. | affiliation=Univ A , Town , X"#,
      r#"Cat Cole | affiliation=Univ A , Town , X"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cat Cole"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "author_thanks_shared_by_footnotemark_marks_kept",
    tex:        repro!("author_thanks_shared_by_footnotemark_marks_kept"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution. ; thanks[†]=Univ A"#,
      r#"Bob Baker ; thanks[*]=Equal contribution. ; thanks[‡]=Univ B"#,
      r#"Cat Cole ; thanks[†]=Univ A ; thanks[‡]=Univ B"#,
      r#"Dan Dorn ; thanks[‡]=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note ; †→note"#,
      r#"Bob Baker ; *→note ; ‡→note"#,
      r#"Cat Cole ; †→note ; ‡→note"#,
      r#"Dan Dorn ; ‡→note"#,
      r#"margin: Equal contribution. ; Univ A ; Univ B"#,
    ],
    page_moves: r#"row R MARKORDER (the notes' document order is not the order the PDF prints their marks; DG 2026-11-16: minor, each mark is on its carrier)"#,
  },
  Row {
    repro:      "author_thanks_mark_after_a_title_thanks",
    tex:        repro!("author_thanks_mark_after_a_title_thanks"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[†]=Note X."#,
      r#"Bob Baker ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; † plain",
      "Bob Baker ; * plain",
      "margin: Note X.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "author_footnote_is_the_marked_thanks_note",
    tex:        repro!("author_footnote_is_the_marked_thanks_note"),
    warnings:   0,
    creators:   &[r#"Ann Able ; thanks[*]=Univ X"#, r#"Bob Baker"#],
    moves:      r#""#,
    page:       &["Ann Able ; * plain", "Bob Baker", "margin: Univ X"],
    page_moves: r#""#,
  },
  Row {
    repro:      "article_title_block_marks_in_fnsymbol",
    tex:        repro!("article_title_block_marks_in_fnsymbol"),
    warnings:   0,
    creators:   &[
      r#"Ann ; footnote[*]=Equal."#,
      r#"Bob ; footnote[†]=Intern."#,
    ],
    moves:      r#""#,
    page:       &["Ann ; * plain", "Bob ; † plain", "margin: Equal. ; Intern."],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2c_mark_after_note",
    tex:        repro!("ne2c_mark_after_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= ; thanks[*]=Univ X."#,
      r#"Bob Baker ; footnotemark[‡]="#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      "Ann Able ; † plain ; * plain",
      "Bob Baker ; ‡ plain",
      "margin: Univ X.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2c_mark_after_note_numbered",
    tex:        repro!("ne2c_mark_after_note_numbered"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; thanks[*]=Univ X."#,
      r#"Bob Baker ; thanks[*]=Univ X."#,
      r#"Carl Cole ; thanks[†]=Univ Y."#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; *→note ; *→note",
      "Bob Baker ; *→note",
      "Carl Cole ; † plain",
      "margin: Univ X. ; Univ Y.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2c_mark_after_note_spaced",
    tex:        repro!("ne2c_mark_after_note_spaced"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= ; thanks[*]=Univ X."#,
      r#"Bob Baker ; thanks[‡]=Univ Y."#,
      r#"Carl Cole ; footnotemark[§]="#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      "Ann Able ; † plain ; * plain",
      "Bob Baker ; ‡ plain",
      "Carl Cole ; § plain",
      "margin: Univ X. ; Univ Y.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2c_mark_after_footnote_glued",
    tex:        repro!("ne2c_mark_after_footnote_glued"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= ; thanks[*]=Univ X."#,
      r#"Bob Baker ; thanks[*]=Univ X. ; thanks[‡]=Univ Y."#,
      r#"Carl Cole ; footnotemark[§]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note ; † plain"#,
      r#"Bob Baker ; *→note | under: Univ Y."#,
      r#"Carl Cole ; § plain"#,
      r#"margin: Univ X."#,
    ],
    page_moves: r#"row R MARKORDER (the notes' document order is not the order the PDF prints their marks; DG 2026-11-16: minor, each mark is on its carrier)"#,
  },
  Row {
    repro:      "ne2b_note_before_every_name",
    tex:        repro!("ne2b_note_before_every_name"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= ; thanks[*]=Lead note."#,
      r#"Bob Baker"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      "Ann Able ; † plain ; * plain",
      "Bob Baker",
      "margin: Lead note.",
    ],
    page_moves: r#""#,
  },
  // the marks as printed; the Warning is the fail-safe split's (counted, Perl's split)
  Row {
    repro:      "ne2b_counter_moved_between_notes",
    tex:        repro!("ne2b_counter_moved_between_notes"),
    warnings:   1,
    creators:   &[
      r#"Ann Able ; thanks[*]=Univ X. ; thanks[†]=Univ Z."#,
      r#"Bob Baker ; thanks[†]=Univ Z. ; thanks[‡]=Univ Y."#,
      r#"Carl Cole ; thanks[*]=Univ X."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note ; †→note"#,
      r#"Bob Baker ; †→note | under: Univ Y."#,
      r#"Carl Cole ; *→note"#,
      r#"margin: Univ X. ; Univ Z."#,
    ],
    page_moves: r#"row R MARKORDER (the notes' document order is not the order the PDF prints their marks; DG 2026-11-16: minor, each mark is on its carrier)"#,
  },
  Row {
    repro:      "fmmarks_title_thanks_then_author_marks_link",
    tex:        repro!("fmmarks_title_thanks_then_author_marks_link"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[†]=Equal contribution."#,
      r#"Bob Baker ; thanks[†]=Equal contribution."#,
      r#"Cy Cole ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; †→note",
      "Bob Baker ; †→note",
      "Cy Cole ; * plain",
      "margin: Equal contribution.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_title_footnote_and_date_thanks_shift_marks",
    tex:        repro!("fmmarks_title_footnote_and_date_thanks_shift_marks"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[‡]=Equal contribution."#,
      r#"Bob Baker ; thanks[‡]=Equal contribution."#,
      r#"Cy Cole ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; ‡→note",
      "Bob Baker ; ‡→note",
      "Cy Cole ; * plain",
      "margin: Equal contribution.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_counter_set_arabic_thanks",
    tex:        repro!("fmmarks_counter_set_arabic_thanks"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[‡]=First note."#,
      r#"Bob Baker ; thanks[§]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; ‡ plain",
      "Bob Baker ; § plain",
      "margin: First note. ; Second note.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_spaced_marks_before_a_comma_fnsymbol_texts",
    tex:        repro!("fmmarks_spaced_marks_before_a_comma_fnsymbol_texts"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnote[*]=Univ A. Email: ann@a.edu ; footnote[‡]=Institute C."#,
      r#"Bob Baker ; footnote[†]=Univ B. Email: bob@b.edu"#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; * plain ; ‡ plain",
      "Bob Baker ; † plain",
      "margin: Institute C. ; Univ A. Email: ann@a.edu ; Univ B. Email: bob@b.edu",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_thanks_typed_star_text_and_footnotemark",
    tex:        repro!("fmmarks_thanks_typed_star_text_and_footnotemark"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=*Equal contribution | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[*]=*Equal contribution | affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"margin: *Equal contribution"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_acmart_authornotemark_without_its_note_red",
    tex:        repro!("fmmarks_acmart_authornotemark_without_its_note_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Ann Able and Bob Baker contributed equally. | email=ann@a.edu ; affiliation=Univ A , Town , Land"#,
      r#"Bob Baker ; footnotemark[†]= ; thanks[*]=Ann Able and Bob Baker contributed equally. | email=bob@a.edu ; affiliation=Univ A , Town , Land"#,
      r#"Cat Cole | email=cat@a.edu ; affiliation=Univ A , Town , Land"#,
    ],
    moves:      r#""#,
    // (the inked mark shows: every unlinked mark does, OD #1059's fail-safe, DG "hide only when certain")
    page:       &[
      "Ann Able ; *→note",
      "Bob Baker ; *→note ; † plain",
      "Cat Cole",
      "margin: Ann Able and Bob Baker contributed equally.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2d_given_mark_dropped_keeps_the_creators",
    tex:        repro!("ne2d_given_mark_dropped_keeps_the_creators"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Univ X. | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[*]=Univ X. | affiliation=Univ A"#,
      r#"Carl Cole ; thanks[*]=Univ X. | affiliation=Univ B"#,
      r#"Dan Dory | affiliation=Univ B"#,
    ],
    moves:      "",
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Carl Cole ; *→note"#,
      r#"Dan Dory"#,
      r#"margin: Univ X."#,
    ],
    page_moves: "",
  },
  Row {
    repro:      "ne2d_given_and_fresh_mark_dropped",
    tex:        repro!("ne2d_given_and_fresh_mark_dropped"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Univ X. | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[*]=Univ X. | affiliation=Univ A"#,
      r#"Carl Cole ; footnotemark[†]= | affiliation=Univ B"#,
      r#"Dan Dory ; footnotemark[‡]= | affiliation=Univ B"#,
    ],
    moves:      "",
    // (the inked mark shows: every unlinked mark does, OD #1059's fail-safe, DG "hide only when certain")
    page:       &[
      "Ann Able ; *→note",
      "Bob Baker ; *→note",
      "Carl Cole ; † plain",
      "Dan Dory ; ‡ plain",
      "margin: Univ X.",
    ],
    page_moves: "",
  },
  Row {
    repro:      "fmmarks_label_in_a_copied_thanks",
    tex:        repro!("fmmarks_label_in_a_copied_thanks"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution."#,
      r#"Bob Baker ; thanks[*]=Equal contribution."#,
      r#"Cat Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cat Cole"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbx_aaai_equalcontrib_repeats_its_mark",
    tex:        repro!("presbx_aaai_equalcontrib_repeats_its_mark"),
    warnings:   1,
    creators:   &[
      r#"Ann Able ; thanks[*]=These authors contributed equally."#,
      r#"Bob Baker ; thanks[*]=These authors contributed equally."#,
      r#"Cy Cole ; thanks[*]=These authors contributed equally."#,
      r#"Dee Dunn ; thanks[†]=Corresponding author."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; *→note"#,
      r#"Dee Dunn ; † plain"#,
      r#"margin: Corresponding author. ; These authors contributed equally."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbx_aaai_equalcontrib_then_footnotemark",
    tex:        repro!("presbx_aaai_equalcontrib_then_footnotemark"),
    warnings:   1,
    creators:   &[
      r#"Ann Able ; thanks[*]=These authors contributed equally."#,
      r#"Bob Baker ; thanks[*]=These authors contributed equally."#,
      r#"Cy Cole ; thanks[†]=Corresponding authors."#,
      r#"Dee Dunn ; thanks[†]=Corresponding authors."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; †→note"#,
      r#"Dee Dunn ; †→note"#,
      r#"margin: Corresponding authors. ; These authors contributed equally."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbx_aaai2027_corresponding_repeats_its_mark",
    tex:        repro!("presbx_aaai2027_corresponding_repeats_its_mark"),
    warnings:   2,
    creators:   &[
      r#"Ann Able ; thanks[†]=These authors contributed equally. ; thanks[‡]=Corresponding author."#,
      r#"Bob Baker ; thanks[†]=These authors contributed equally."#,
      r#"Cy Cole ; thanks[‡]=Corresponding author."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; †→note ; ‡→note"#,
      r#"Bob Baker ; †→note"#,
      r#"Cy Cole ; ‡→note"#,
      r#"margin: Corresponding author. ; These authors contributed equally."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbxb_aaai_equalcontrib_after_a_period (PDF)",
    tex:        repro!("presbxb_aaai_equalcontrib_after_a_period"),
    warnings:   1,
    creators:   &[
      r#"Ann Able ; thanks[*]=These authors contributed equally."#,
      r#"Bob Baker ; thanks[*]=These authors contributed equally."#,
      r#"Cy Cole ; thanks[*]=These authors contributed equally."#,
      r#"Dee Dunn"#,
    ],
    moves:      r#"row R AAAIDOT (the fail-safe after a ".": each use its own note, the base's glyphs †, ‡)"#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; *→note"#,
      r#"Dee Dunn"#,
      r#"margin: These authors contributed equally."#,
    ],
    page_moves: r#"row R AAAIDOT (each use its own note: Bob's † plain)"#,
  },
  // (the fail-safe twins: the base's output, every use after a "." a note of its own)
  Row {
    repro:      "presbxb_aaai_equalcontrib_after_a_period",
    tex:        repro!("presbxb_aaai_equalcontrib_after_a_period"),
    warnings:   3,
    creators:   &[
      r#"Ann Able ; thanks[*]=These authors contributed equally."#,
      r#"Bob Baker ; thanks[†]=These authors contributed equally."#,
      r#"Cy Cole ; thanks[‡]=These authors contributed equally."#,
      r#"Dee Dunn"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"Cy Cole ; ‡ plain"#,
      r#"Dee Dunn"#,
      r#"margin: These authors contributed equally. ; These authors contributed equally. ; These authors contributed equally."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbxc_aaai_period_after_a_carriers_mark (PDF)",
    tex:        repro!("presbxc_aaai_period_after_a_carriers_mark"),
    warnings:   1,
    creators:   &[
      r#"Ann Able ; thanks[*]=These authors contributed equally."#,
      r#"Bob Baker ; thanks[*]=These authors contributed equally."#,
      r#"Cy Cole"#,
      r#"Dee Dunn"#,
    ],
    moves:      r#"row R AAAIDOT (the fail-safe after a ".": each use its own note, the base's glyphs †, ‡)"#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole"#,
      r#"Dee Dunn"#,
      r#"margin: These authors contributed equally."#,
    ],
    page_moves: r#"row R AAAIDOT (each use its own note: Bob's † plain)"#,
  },
  Row {
    repro:      "presbxc_aaai_period_after_a_carriers_mark",
    tex:        repro!("presbxc_aaai_period_after_a_carriers_mark"),
    warnings:   2,
    creators:   &[
      r#"Ann Able ; thanks[*]=These authors contributed equally."#,
      r#"Bob Baker ; thanks[†]=These authors contributed equally."#,
      r#"Cy Cole"#,
      r#"Dee Dunn"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"Cy Cole"#,
      r#"Dee Dunn"#,
      r#"margin: These authors contributed equally. ; These authors contributed equally."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbxc_aaai_period_after_the_first_carrier (PDF)",
    tex:        repro!("presbxc_aaai_period_after_the_first_carrier"),
    warnings:   1,
    creators:   &[
      r#"Ann Able ; thanks[*]=These authors contributed equally."#,
      r#"Bob Baker ; thanks[*]=These authors contributed equally."#,
      r#"Cy Cole"#,
    ],
    moves:      r#"row R AAAIDOT (the fail-safe after a ".": each use its own note, the base's glyphs †, ‡)"#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole"#,
      r#"margin: These authors contributed equally."#,
    ],
    page_moves: r#"row R AAAIDOT (each use its own note: Bob's † plain)"#,
  },
  Row {
    repro:      "presbxc_aaai_period_after_the_first_carrier",
    tex:        repro!("presbxc_aaai_period_after_the_first_carrier"),
    warnings:   2,
    creators:   &[
      r#"Ann Able ; thanks[*]=These authors contributed equally."#,
      r#"Bob Baker ; thanks[†]=These authors contributed equally."#,
      r#"Cy Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"Cy Cole"#,
      r#"margin: These authors contributed equally. ; These authors contributed equally."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbxc_dotcut_footnotemark_after_a_period_red",
    tex:        repro!("presbxc_dotcut_footnotemark_after_a_period_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal."#,
      r#"Bob Baker ; thanks[*]=Equal."#,
      r#"Cy Cole"#,
      r#"Dee Dunn"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole"#,
      r#"Dee Dunn"#,
      r#"margin: Equal."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducec_own_maketitle_samethanks_links_by_number",
    tex:        repro!("mtreducec_own_maketitle_samethanks_links_by_number"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution."#,
      r#"Bob Baker ; thanks[*]=Equal contribution."#,
      r#"Cy Cole ; thanks[†]=Corresponding: cy@x.org"#,
      r#"Dee Dunn ; thanks[*]=Equal contribution."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; † plain"#,
      r#"Dee Dunn ; *→note"#,
      r#"margin: Corresponding: cy@x.org ; Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducec_own_maketitle_counter_expressions_link_by_number",
    tex:        repro!("mtreducec_own_maketitle_counter_expressions_link_by_number"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution."#,
      r#"Bob Baker ; thanks[*]=Equal contribution."#,
      r#"Cy Cole ; thanks[†]=Corresponding: cy@x.org"#,
      r#"Dee Dunn ; thanks[*]=Equal contribution."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; † plain"#,
      r#"Dee Dunn ; *→note"#,
      r#"margin: Corresponding: cy@x.org ; Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducec_package_maketitle_samethanks_links_by_number",
    tex:        repro!("mtreducec_package_maketitle_samethanks_links_by_number"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution."#,
      r#"Bob Baker ; thanks[*]=Equal contribution."#,
      r#"Cy Cole ; thanks[†]=Corresponding: cy@x.org"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; † plain"#,
      r#"margin: Corresponding: cy@x.org ; Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "mtreducec_own_maketitle_typed_mark_links_red",
    tex:        repro!("mtreducec_own_maketitle_typed_mark_links_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[1]=Equal contribution."#,
      r#"Bob Baker ; thanks[1]=Equal contribution."#,
      r#"Cy Cole ; thanks[2]=Corresponding: cy@x.org"#,
    ],
    moves:      r#"row R MTREDUCE-TYPED (a mark the author types under a `\maketitle` whose glyph is unknown here links nothing: the glyph the `\thanks` prints is unknown, so the typed 1 names no note; formerly the static reader's `\arabic`)"#,
    page:       &[
      r#"Ann Able ; 1→note"#,
      r#"Bob Baker ; 1→note"#,
      r#"Cy Cole ; 2 plain"#,
      r#"margin: Corresponding: cy@x.org ; Equal contribution."#,
    ],
    page_moves: r#"row R MTREDUCE-TYPED (Bob's typed 1 links nothing)"#,
  },
];
/// Links: a `\footnotemark[n]`, `\samethanks[n]` or `\authornotemark[n]` on a name is a copy of the note it marks, its
/// mark as the class numbers it after the title's notes; a reference into a copied note still resolves (2502.19103,
/// 1911.13299, 1706.03762, 2607.15705, 2203.12881, 1803.03669; revSHMARK p1, p2, p9; revREND2b p2); AAAI's
/// `\equalcontrib` followed by "." makes each use a note of its own, the fail-safe (revPRESBXb BL1, 2509.03329); a mark
/// whose number the title block reads off the counter (`\samethanks`' `[\value{footnote}]`, `[\arabic{footnote}]`,
/// `[\numexpr…]`) links by it where the count is TeX's, the glyph unknown under the document's or a package's own
/// `\maketitle` (revMTREDUCE B2).
#[test]
fn a_mark_on_a_name_is_a_copy_of_its_note() { assert_green(LINK); }
/// The known gaps of [`a_mark_on_a_name_is_a_copy_of_its_note`].
#[test]
#[ignore = "RED: the row R lines its rows name"]
fn a_mark_on_a_name_is_a_copy_of_its_note_red() { assert_red_layers_now_green(LINK); }

/// The rows of [`a_mark_past_spacing_is_the_name_befores`] and its `_red` twin.
const SPACING: &[Row] = &[
  Row {
    repro:      "ne2e_mark_past_spacing_is_its_names",
    tex:        repro!("ne2e_mark_past_spacing_is_its_names"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= ; thanks[*]=Univ X."#,
      r#"Bob Baker ; thanks[*]=Univ X."#,
      r#"Carl Cole ; footnotemark[†]="#,
      r#"Dan Dory ; footnotemark[‡]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note ; † plain"#,
      r#"Bob Baker ; *→note"#,
      r#"Carl Cole ; † plain"#,
      r#"Dan Dory ; ‡ plain"#,
      r#"margin: Univ X."#,
    ],
    page_moves: r#"row R MARKORDER (the notes' document order is not the order the PDF prints their marks; DG 2026-11-16: minor, each mark is on its carrier)"#,
  },
  Row {
    repro:      "ne2h_marks_run_carried_back_whole",
    tex:        repro!("ne2h_marks_run_carried_back_whole"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before as printed once; every unlinked mark shows, OD #1059's fail-safe)
    page:       &["Ann Able ; * plain ; † plain", "Bob Baker", "Carl Cole"],
    page_moves: r#""#,
  },
  // the mark as printed, no Warning (the mark ledger writes the prefix Bob's: SHGNEXT, OD #1057)
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2e_prefix_mark_past_spacing_before_and",
    tex:        repro!("ne2e_prefix_mark_past_spacing_before_and"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  // the marks as printed; the one Warning the labels' (the prefix's false "lost: †" gone: SHGNEXT, OD #1057)
  Row {
    repro:      "ne2e_prefix_marks_past_quad",
    tex:        repro!("ne2e_prefix_marks_past_quad"),
    warnings:   1,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole ; footnotemark[*]="#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      "Ann Able ; * plain",
      "Bob Baker ; † plain",
      "Carl Cole ; * plain",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_bare_marks_after_space_and_quad",
    tex:        repro!("fmmarks_bare_marks_after_space_and_quad"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker ; thanks[†]=X"#,
      r#"Carl Cole ; footnotemark[‡]="#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      "Ann Able ; * plain",
      "Bob Baker ; † plain",
      "Carl Cole ; ‡ plain",
      "margin: X",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_mark_before_comma_after_quad",
    tex:        repro!("fmmarks_mark_before_comma_after_quad"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &["Ann Able ; † plain", "Bob Baker", "Carl Cole"],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_mark_inside_the_names_font_group",
    tex:        repro!("fmmarks_mark_inside_the_names_font_group"),
    warnings:   0,
    creators:   &[r#"Ann Able ; footnotemark[*]="#, r#"Bob Baker"#],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &["Ann Able ; * plain", "Bob Baker"],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_mark_then_line_break",
    tex:        repro!("fmmarks_mark_then_line_break"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= | affiliation=Univ A"#,
      r#"Bob Baker"#,
    ],
    moves:      r#""#,
    // (the inked mark shows: every unlinked mark does, OD #1059's fail-safe, DG "hide only when certain")
    page:       &["Ann Able ; † plain", "Bob Baker"],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_marks_run_split_then_line_break",
    tex:        repro!("fmmarks_marks_run_split_then_line_break"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; footnotemark[†]= | affiliation=Univ A"#,
      r#"Bob Baker"#,
    ],
    moves:      r#""#,
    // (the inked mark shows: every unlinked mark does, OD #1059's fail-safe, DG "hide only when certain")
    page:       &["Ann Able ; * plain ; † plain", "Bob Baker"],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2c_mark_on_no_name_keeps_its_footnote_step",
    tex:        repro!("ne2c_mark_on_no_name_keeps_its_footnote_step"),
    warnings:   1,
    creators:   &[
      r#"Ann Able ; thanks[*]=Univ X."#,
      r#"Bob Baker ; footnotemark[‡]="#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      "Ann Able ; * plain",
      "Bob Baker ; ‡ plain",
      "margin: Univ X.",
    ],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2e_prefix_mark_past_hfill",
    tex:        repro!("ne2e_prefix_mark_past_hfill"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= ; thanks[*]=X"#,
      r#"Bob Baker"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; † plain ; * plain"#,
      r#"Bob Baker"#,
      r#"margin: X"#,
    ],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2f_bare_mark_past_quads_is_the_name_befores",
    tex:        repro!("ne2f_bare_mark_past_quads_is_the_name_befores"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= ; thanks[*]=X"#,
      r#"Bob Baker ; footnotemark[‡]="#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; † plain ; * plain"#,
      r#"Bob Baker ; ‡ plain"#,
      r#"margin: X"#,
    ],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2f_mark_and_note_past_qquad_are_the_name_befores",
    tex:        repro!("ne2f_mark_and_note_past_qquad_are_the_name_befores"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= ; thanks[*]=X ; thanks[‡]=Y"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; † plain ; * plain ; ‡ plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
      r#"margin: X ; Y"#,
    ],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2f_mark_past_qquad_is_the_name_befores",
    tex:        repro!("ne2f_mark_past_qquad_is_the_name_befores"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2f_mark_past_quads_is_the_name_befores",
    tex:        repro!("ne2f_mark_past_quads_is_the_name_befores"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2f_mark_past_ties_is_the_name_befores",
    tex:        repro!("ne2f_mark_past_ties_is_the_name_befores"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2g_mark_in_group_with_previous_name",
    tex:        repro!("ne2g_mark_in_group_with_previous_name"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2g_mark_in_its_own_group",
    tex:        repro!("ne2g_mark_in_its_own_group"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2f_marks_wider_after_and_tied",
    tex:        repro!("ne2f_marks_wider_after_and_tied"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Bob Baker ; footnotemark[‡]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; * plain ; † plain"#,
      r#"Bob Baker ; ‡ plain"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2g_mark_past_hfill_undecided",
    tex:        repro!("ne2g_mark_past_hfill_undecided"),
    warnings:   0,
    creators:   &[r#"Ann Able ; footnotemark[†]="#, r#"Bob Baker"#],
    moves:      r#""#,
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2g_mark_then_orcid_nearer_next",
    tex:        repro!("ne2g_mark_then_orcid_nearer_next"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= | orcid="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2g_mark_then_superscript_nearer_next",
    tex:        repro!("ne2g_mark_then_superscript_nearer_next"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[a]= ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; a plain ; † plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2h_marks_run_parted_at_its_widest_gap",
    tex:        repro!("ne2h_marks_run_parted_at_its_widest_gap"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; * plain ; † plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2h_marks_run_wider_before_goes_to_the_next_name",
    tex:        repro!("ne2h_marks_run_wider_before_goes_to_the_next_name"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; * plain ; † plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "fmmarks_bare_prefix_mark_after_quad_red",
    tex:        repro!("fmmarks_bare_prefix_mark_after_quad_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker"#,
      r#"Carl Cole ; footnotemark[†]="#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole ; † plain"#,
    ],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "fmmarks_gap_mark_then_thanks_red",
    tex:        repro!("fmmarks_gap_mark_then_thanks_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= ; thanks[*]=Ann note"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; † plain ; * plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
      r#"margin: Ann note"#,
    ],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "fmmarks_gap_thanks_then_mark_red",
    tex:        repro!("fmmarks_gap_thanks_then_mark_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= ; thanks[*]=Ann note"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; † plain ; * plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
      r#"margin: Ann note"#,
    ],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "fmmarks_mark_after_wide_hspace_nearer_next_red",
    tex:        repro!("fmmarks_mark_after_wide_hspace_nearer_next_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[r#"Ann Able ; * plain"#, r#"Bob Baker"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "fmmarks_mark_and_note_floating_between_names_red",
    tex:        repro!("fmmarks_mark_and_note_floating_between_names_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= ; thanks[*]=Ann note body ; thanks[‡]=Floating body"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; † plain ; * plain ; ‡ plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
      r#"margin: Ann note body ; Floating body"#,
    ],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "fmmarks_mark_between_interword_spaces_red",
    tex:        repro!("fmmarks_mark_between_interword_spaces_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_mark_carried_back_before_bold_name_red",
    tex:        repro!("fmmarks_mark_carried_back_before_bold_name_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &["Ann Able ; * plain", "Bob Baker", "Carl Cole"],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "fmmarks_mark_nearer_next_bold_name_red",
    tex:        repro!("fmmarks_mark_nearer_next_bold_name_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "fmmarks_mark_then_cite_nearer_next_red",
    tex:        repro!("fmmarks_mark_then_cite_nearer_next_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "fmmarks_mark_then_footnotetext_nearer_next_red",
    tex:        repro!("fmmarks_mark_then_footnotetext_nearer_next_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[†]=Bob note"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      "",
    page:       &[
      r#"Ann Able ; † plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
      r#"margin: Bob note"#,
    ],
    page_moves: "",
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "fmmarks_mark_then_hfill_before_next_red",
    tex:        repro!("fmmarks_mark_then_hfill_before_next_red"),
    warnings:   0,
    creators:   &[r#"Ann Able ; footnotemark[†]="#, r#"Bob Baker"#],
    moves:      r#""#,
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "fmmarks_mark_then_superscript_label_of_next_red",
    tex:        repro!("fmmarks_mark_then_superscript_label_of_next_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]="#,
      r#"Bob Baker ; footnotemark[b]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; † plain"#,
      r#"Bob Baker ; b plain"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "fmmarks_mark_then_thanks_before_next_name_red",
    tex:        repro!("fmmarks_mark_then_thanks_before_next_name_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= ; thanks[*]=Bob note"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; † plain ; * plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
      r#"margin: Bob note"#,
    ],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "fmmarks_marks_run_split_by_wide_space_red",
    tex:        repro!("fmmarks_marks_run_split_by_wide_space_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; * plain ; † plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "fmmarks_marks_three_names_one_line_red",
    tex:        repro!("fmmarks_marks_three_names_one_line_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_tabular_cell_bare_prefix_mark_red",
    tex:        repro!("fmmarks_tabular_cell_bare_prefix_mark_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=X | affiliation=Univ A"#,
      r#"Bob Baker ; footnotemark[†]= | affiliation=Univ B"#,
    ],
    moves:      r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#,
    page:       &[r#"Ann Able | under: X"#, r#"Bob Baker ; † plain"#],
    page_moves: r#"the XML layer's: row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#,
  },
  Row {
    repro:      "fmmarks_tabular_cell_given_prefix_mark_red",
    tex:        repro!("fmmarks_tabular_cell_given_prefix_mark_red"),
    warnings:   0,
    creators:   &[r#"Ann Able"#, r#"Bob Baker ; footnotemark[†]="#],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &["Ann Able", "Bob Baker ; † plain"],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_single_name_marks_after_newline_thin_spaces_red",
    tex:        repro!("fmmarks_single_name_marks_after_newline_thin_spaces_red"),
    warnings:   0,
    creators:   &[r#"Ann Able ; footnotemark[1]= ; footnotemark[2]="#],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &["Ann Able ; 1 plain ; 2 plain"],
    page_moves: r#""#,
  },
  // the marks a writer writes each from its own first token, never the one before it again (review revSHGNEXT B1, P1:
  // a mark's event span starts where the box before it began)
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2j_two_marks_glued_nearer_next",
    tex:        repro!("ne2j_two_marks_glued_nearer_next"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; * plain ; † plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2j_three_marks_glued_nearer_next",
    tex:        repro!("ne2j_three_marks_glued_nearer_next"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; footnotemark[†]= ; footnotemark[‡]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; * plain ; † plain ; ‡ plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2j_two_marks_glued_to_a_name_before_and",
    tex:        repro!("ne2j_two_marks_glued_to_a_name_before_and"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Bob Baker"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &["Ann Able ; * plain ; † plain", "Bob Baker"],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2j_two_marks_spaced_from_a_name_before_and",
    tex:        repro!("ne2j_two_marks_spaced_from_a_name_before_and"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Bob Baker"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &["Ann Able ; * plain ; † plain", "Bob Baker"],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2j_two_marks_carried_back_past_qquad",
    tex:        repro!("ne2j_two_marks_carried_back_past_qquad"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &["Ann Able ; * plain ; † plain", "Bob Baker", "Carl Cole"],
    page_moves: r#""#,
  },
  // (re-pinned: DG 2026-11-17 default + edge proof — the name before)
  Row {
    repro:      "ne2j_macro_marks_nearer_next",
    tex:        repro!("ne2j_macro_marks_nearer_next"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      r#"Ann Able ; * plain ; † plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  // a formula's comma glued between two marks is ink of their run (revSHGNEXT S1; 1703.08733's shape)
  // (re-pinned, DG 2026-11-17 S3: Ann's `\thanks` carries key 1 already, so the `\footnotemark[1]` past the `\qquad` and
  // its run are Bob's, as pdflatex prints "∗,†" over his name — Ann ∗ ∗ † before, from the default alone)
  Row {
    repro:      "ne2j_math_comma_between_marks_nearer_next",
    tex:        repro!("ne2j_math_comma_between_marks_nearer_next"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=X"#,
      r#"Bob Baker ; footnotemark[†]= ; thanks[*]=X"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note ; † plain"#,
      r#"Carl Cole"#,
      r#"margin: X"#,
    ],
    page_moves: r#""#,
  },
  // a mark whose key the name before carries already is the next name's (DG 2026-11-17 S3); distinct from the key an
  // earlier name carries, which leaves a mark glued to its name that name's (s08)
  Row {
    repro:      "ne2f_mark_of_a_key_the_name_before_carries_is_the_next_names",
    tex:        repro!("ne2f_mark_of_a_key_the_name_before_carries_is_the_next_names"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnote[†]=Two."#,
      r#"Bob Baker ; footnote[†]=Two."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; †→note"#,
      r#"Bob Baker ; †→note"#,
      r#"margin: Two."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2f_mark_of_a_key_an_earlier_name_carries_stays_the_name_befores",
    tex:        repro!("ne2f_mark_of_a_key_an_earlier_name_carries_stays_the_name_befores"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution."#,
      r#"Bob Baker ; thanks[*]=Equal contribution."#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Carl Cole"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
];
/// Marks after spacing (ruling 4 and DG 2026-11-17): a mark spacing parts from the names is the name before's, whatever
/// the glue, the next name's where the name before carries its key already (S3); followed by a separator, `\\` or the
/// block's end it is carried back (1911.13299, 2412.20504, 1807.02791; every revSHGIVEN probe).
#[test]
fn a_mark_past_spacing_is_the_name_befores() { assert_green(SPACING); }
/// The known gaps of [`a_mark_past_spacing_is_the_name_befores`].
#[test]
#[ignore = "RED: the row R lines its rows name"]
fn a_mark_past_spacing_is_the_name_befores_red() { assert_red_layers_now_green(SPACING); }

/// The rows of [`revtex_marks_follow_its_numbering`] and its `_red` twin.
const REVTEX: &[Row] = &[
  Row {
    repro:      "revtex40_name_footnote_unnumbered_is_its_note_contact",
    tex:        repro!("revtex40_name_footnote_unnumbered_is_its_note_contact"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | thanks=Equal contribution. ; affiliation=Univ A"#,
      r#"Bob Baker | note=Visiting Univ C. ; affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "revtex41_name_footnote_is_its_note_contact",
    tex:        repro!("revtex41_name_footnote_is_its_note_contact"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | note=ann@a.edu ; affiliation=Univ A, Town"#,
      r#"Bob Baker | note=Corresponding author: bob@b.edu ; affiliation=Univ A, Town"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "revtex42_name_footnote_is_its_note_contact",
    tex:        repro!("revtex42_name_footnote_is_its_note_contact"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | thanks=Equal contribution. ; affiliation=Univ A"#,
      r#"Bob Baker | note=Visiting Univ C. ; affiliation=Univ B"#,
      r#"Cy Cole | email=cy@b.edu ; affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#, r#"Cy Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "revtex4_name_footnote_takes_its_number",
    tex:        repro!("revtex4_name_footnote_takes_its_number"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | note=Visiting Univ C. ; affiliation=Univ A"#,
      r#"Bob Baker ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
      r#"Cy Cole ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; †→note"#,
      r#"Cy Cole ; †→note"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "revtex_name_footnote_given_its_number_steps_nothing",
    tex:        repro!("revtex_name_footnote_given_its_number_steps_nothing"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | note=Visiting Univ C. ; affiliation=Univ A"#,
      r#"Bob Baker ; thanks[*]=Equal contribution. | affiliation=Univ B"#,
      r#"Cy Cole ; thanks[*]=Equal contribution. | affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; *→note"#,
      r#"Cy Cole ; *→note"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "revtex_name_footnote_takes_its_number",
    tex:        repro!("revtex_name_footnote_takes_its_number"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | note=Visiting Univ C. ; affiliation=Univ A"#,
      r#"Bob Baker ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
      r#"Cy Cole ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; †→note"#,
      r#"Cy Cole ; †→note"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "revtex_repeated_thanks_is_the_authors_marked_note",
    tex:        repro!("revtex_repeated_thanks_is_the_authors_marked_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | email=a@x.org ; thanks=These authors contributed equally. ; affiliation=Univ A"#,
      r#"Bob Baker ; thanks[†]=These authors contributed equally. | affiliation=Univ A"#,
      r#"Cat Cole ; thanks[†]=These authors contributed equally. | affiliation=Univ A"#,
      r#"Dan Dole | altaffiliation=Univ B. ; affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; †→note"#,
      r#"Cat Cole ; †→note"#,
      r#"Dan Dole"#,
      r#"margin: These authors contributed equally."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "revtex_thanks_in_names_stay_unmarked",
    tex:        repro!("revtex_thanks_in_names_stay_unmarked"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Equal. | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[]=Equal. | affiliation=Univ A"#,
      r#"Cat Cole ; thanks[]=Other. | affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Equal."#,
      r#"Bob Baker | under: Equal."#,
      r#"Cat Cole | under: Other."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "revtex_affiliation_footnotemark_link_is_no_note",
    tex:        repro!("revtex_affiliation_footnotemark_link_is_no_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=E-mail: ann@a.org | affiliation=Univ A, Town ; affiliation=Lab B, City"#,
      r#"Bob Baker ; thanks[†]=E-mail: bob@c.org | affiliation=Univ C, Village"#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; * plain",
      "Bob Baker ; † plain",
      "margin: E-mail: ann@a.org ; E-mail: bob@c.org",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "emit_revtex_macro_name_with_thanks_written_once",
    tex:        repro!("emit_revtex_macro_name_with_thanks_written_once"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Corresponding author. | affiliation=Univ A"#,
      r#"Bob Baker | affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able | under: Corresponding author."#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_revtex40_email_then_name_footnote",
    tex:        repro!("fmmarks_revtex40_email_then_name_footnote"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | email=ann@a.edu ; affiliation=Univ A"#,
      r#"Bob Baker | note=Visiting Univ C. ; affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_revtex40_two_name_footnotes",
    tex:        repro!("fmmarks_revtex40_two_name_footnotes"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Univ A"#,
      r#"Bob Baker | note=Visiting Univ C. ; note=Second. ; affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_revtex42_reprint_thanks_name_footnote_email",
    tex:        repro!("fmmarks_revtex42_reprint_thanks_name_footnote_email"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | thanks=Equal contribution. ; affiliation=Univ A"#,
      r#"Bob Baker | note=Visiting Univ C. ; affiliation=Univ B"#,
      r#"Cy Cole | email=cy@b.edu ; affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#, r#"Cy Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_revtex_shared_thanks_then_email",
    tex:        repro!("fmmarks_revtex_shared_thanks_then_email"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | thanks=Equal. ; affiliation=Univ A"#,
      r#"Bob Baker | thanks=Equal. ; email=b@x.org ; affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_revtex_footnotemark_answered_by_altaffiliation",
    tex:        repro!("fmmarks_revtex_footnotemark_answered_by_altaffiliation"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[a]= | altaffiliation=These authors contribute equally to this work. ; affiliation=Univ A, Town"#,
      r#"Bob Baker ; footnotemark[a]= | altaffiliation=These authors contribute equally to this work. ; affiliation=Univ B, City"#,
      r#"Cat Cole | email=cat@a.edu ; affiliation=Univ A, Town"#,
    ],
    moves:      r#"row R REVNAMEMARK (REVTeX 4.1/4.2 print a name's `\footnotemark` in their minipage letters, "a": the engine's fail-safe, `title_marks_unknown` under `lx_title_footnote_own_marks`, writes none, a given number's letter included)"#,
    page:       &[
      r#"Ann Able ; a plain"#,
      r#"Bob Baker ; a plain"#,
      r#"Cat Cole"#,
    ],
    page_moves: r#"the XML layer's: row R REVNAMEMARK"#,
  },
  Row {
    repro:      "revtex_affiliation_thanks_before_a_shared_note_red",
    tex:        repro!("revtex_affiliation_thanks_before_a_shared_note_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[*]=Visiting. | affiliation=Univ A"#,
      r#"Cy Cole ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
      r#"Di Dorn ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
    ],
    moves:      r#"row R (its own: REVTeX's affiliation footnote order; SHRESTc fail-safe)"#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker | under: Visiting."#,
      r#"Cy Cole ; †→note"#,
      r#"Di Dorn ; †→note"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#"the XML layer's: row R (its own: REVTeX's affiliation footnote order; SHRESTc fail-safe)"#,
  },
  Row {
    repro:      "revtex_name_footnotemark_leaves_the_marks_unknown",
    tex:        repro!("revtex_name_footnotemark_leaves_the_marks_unknown"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[a]= | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
      r#"Cy Cole ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
    ],
    moves:      r#"OD #1054 fail-safe (a name `\footnotemark` leaves the counted marks unknown); row R REVNAMEMARK (REVTeX 4.1/4.2 print a name's `\footnotemark` in their minipage letters, "a": the engine's fail-safe, `title_marks_unknown` under `lx_title_footnote_own_marks`, writes none, a given number's letter included)"#,
    page:       &[
      r#"Ann Able ; a plain"#,
      r#"Bob Baker ; †→note"#,
      r#"Cy Cole ; †→note"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#"the XML layer's: row R REVNAMEMARK"#,
  },
  Row {
    repro:      "revtex_footnotes_numbered_as_printed",
    tex:        repro!("revtex_footnotes_numbered_as_printed"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[†]=These authors contributed equally. ; thanks[‡]=Visiting Univ C. | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[†]=These authors contributed equally. ; thanks[‡]=Visiting Univ C. | affiliation=Univ B ; email=b@x.org"#,
    ],
    moves:      r#"row R REVTITLETHANKS (proposed; SHRESTb residual: a REVTeX title `\thanks` lands on the last author)"#,
    page:       &[
      r#"Ann Able ; †→note ; ‡→note"#,
      r#"Bob Baker ; †→note ; ‡→note"#,
      r#"margin: These authors contributed equally. ; Visiting Univ C."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_revtex_title_thanks_then_shared_thanks_red",
    tex:        repro!("fmmarks_revtex_title_thanks_then_shared_thanks_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[†]=Equal contribution. | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
    ],
    moves:      r#"row R REVTITLETHANKS (proposed; SHRESTb residual: a REVTeX title `\thanks` lands on the last author)"#,
    page:       &[
      r#"Ann Able ; †→note"#,
      r#"Bob Baker ; †→note"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_revtex_altaffiliation_then_shared_thanks_red",
    tex:        repro!("fmmarks_revtex_altaffiliation_then_shared_thanks_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | altaffiliation=Univ B ; affiliation=Univ A"#,
      r#"Bob Baker ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
      r#"Cy Cole ; thanks[†]=Equal contribution. | affiliation=Univ B"#,
    ],
    moves:      r#"row R REVALTWARN (a content-check Warning for "Univ B", printed as Ann's altaffiliation and the others' affiliation)"#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; †→note"#,
      r#"Cy Cole ; †→note"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_revtex42_aip_shared_thanks_letters",
    tex:        repro!("fmmarks_revtex42_aip_shared_thanks_letters"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[a]=Equal contribution. | affiliation=Univ B"#,
      r#"Cy Cole ; thanks[a]=Equal contribution. | affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; a→note"#,
      r#"Cy Cole ; a→note"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_revtex42_aip_reprint_grid_prints_no_mark",
    tex:        repro!("fmmarks_revtex42_aip_reprint_grid_prints_no_mark"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnote[]=Present address: Lab C. | affiliation=Univ A"#,
      r#"Bob Baker | affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Present address: Lab C."#,
      r#"Bob Baker"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_revtex42_sor_lengthcheck_grid_prints_no_mark",
    tex:        repro!("fmmarks_revtex42_sor_lengthcheck_grid_prints_no_mark"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnote[]=Present address: Lab C. | affiliation=Univ A"#,
      r#"Bob Baker | affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Present address: Lab C."#,
      r#"Bob Baker"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_revtex42_aps_galley_prints_the_letter",
    tex:        repro!("fmmarks_revtex42_aps_galley_prints_the_letter"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnote[b]=Present address: Lab C. | affiliation=Univ A"#,
      r#"Bob Baker | thanks=Bob note. ; affiliation=Univ B"#,
    ],
    moves:      r#""#,
    // (revMARKSAFEc S3 card rule, MARKKEY1: one-line block, the mark shows and the note is a pop-up)
    page:       &[
      r#"Ann Able ; b plain"#,
      r#"Bob Baker"#,
      r#"margin: Present address: Lab C."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_revtex41_aip_shared_thanks_letters",
    tex:        repro!("fmmarks_revtex41_aip_shared_thanks_letters"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[a]=Equal contribution. | affiliation=Univ B"#,
      r#"Cy Cole ; thanks[a]=Equal contribution. | affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; a→note"#,
      r#"Cy Cole ; a→note"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbx_revtex_document_fnsymbol_red",
    tex:        repro!("presbx_revtex_document_fnsymbol_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[†]=equal contribution | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[†]=equal contribution | affiliation=Univ A"#,
      r#"Cy Cole | email=cy@a.org ; affiliation=Univ A"#,
    ],
    moves:      r#"row R PRESBX/fnsymbol (REVTeX's numbering pass writes no glyph under a document's `\@fnsymbol`: the fail-safe)"#,
    page:       &[
      r#"Ann Able ; †→note"#,
      r#"Bob Baker ; †→note"#,
      r#"Cy Cole"#,
      r#"margin: equal contribution"#,
    ],
    page_moves: r#"row R PRESBX/fnsymbol (no glyph written)"#,
  },
];
/// REVTeX numbering: its `\thanks`, `\email`, `\altaffiliation` and name `\footnote` share one counter (4.0) or the
/// minipage letters (4.1/4.2 name footnotes); a note shared by names is one marked note per carrier, a one-carrier note
/// its contact; the affiliation's linking mark no note (1607.07280, 2301.00220, 2301.00155, 2605.04155, 2407.11130).
#[test]
fn revtex_marks_follow_its_numbering() { assert_green(REVTEX); }
/// The known gaps of [`revtex_marks_follow_its_numbering`].
#[test]
#[ignore = "RED: the row R lines its rows name"]
fn revtex_marks_follow_its_numbering_red() { assert_red_layers_now_green(REVTEX); }

/// The rows of [`notes_are_marked_only_where_the_class_prints_marks`] and its `_red` twin.
const CLASS: &[Row] = &[
  Row {
    repro:      "ieee_thanks_leading_label_is_no_mark",
    tex:        repro!("ieee_thanks_leading_label_is_no_mark"),
    warnings:   0,
    creators:   &["Ann Able", "Bob Baker"],
    moves:      r#""#,
    page:       &["Ann Able", "Bob Baker"],
    page_moves: r#""#,
  },
  Row {
    repro:      "thanks_leading_symbol_text_is_no_mark_copy",
    tex:        repro!("thanks_leading_symbol_text_is_no_mark_copy"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=∗ -algebras are her field."#,
      r#"Bob Baker ; thanks[†]=3 He detectors were funded by X."#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; * plain",
      "Bob Baker ; † plain",
      "margin: 3 He detectors were funded by X. ; ∗ -algebras are her field.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "thanks_bare_symbol_is_the_notes_mark",
    tex:        repro!("thanks_bare_symbol_is_the_notes_mark"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Univ A, Town"#,
      r#"Bob Baker ; thanks[*]=Corresponding author. | affiliation=Univ A, Town"#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able",
      "Bob Baker ; * plain",
      "margin: Corresponding author.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_thanks_text_opening_symbols_kept",
    tex:        repro!("fmmarks_thanks_text_opening_symbols_kept"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=3 He detectors were funded by X."#,
      r#"Bob Baker ; thanks[†]=* marks the starred items."#,
      r#"Cy Cole ; thanks[‡]=∗ -algebras are studied."#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; * plain",
      "Bob Baker ; † plain",
      "Cy Cole ; ‡ plain",
      "margin: * marks the starred items. ; 3 He detectors were funded by X. ; ∗ -algebras are studied.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_ieee_journal_thanks_unmarked",
    tex:        repro!("fmmarks_ieee_journal_thanks_unmarked"),
    warnings:   0,
    creators:   &[
      "Ann Able | membership=Student Member, IEEE",
      "Bob Baker | membership=Senior Member, IEEE",
    ],
    moves:      r#""#,
    page:       &["Ann Able", "Bob Baker"],
    page_moves: r#""#,
  },
  Row {
    repro:      "titling_thanksmarkseries_marks_red",
    tex:        repro!("titling_thanksmarkseries_marks_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[1]=First note."#,
      r#"Bob Baker ; thanks[2]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; 1 plain",
      "Bob Baker ; 2 plain",
      "margin: First note. ; Second note.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "maketitle_redefined_prints_no_marks_red",
    tex:        repro!("maketitle_redefined_prints_no_marks_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=First note."#,
      r#"Bob Baker ; thanks[]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: First note."#,
      r#"Bob Baker | under: Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_ieee_unmarked_thanks_same_text_but_link_or_math",
    tex:        repro!("fmmarks_ieee_unmarked_thanks_same_text_but_link_or_math"),
    warnings:   0,
    creators:   &["Ann Able", "Bob Baker", "Cy Cole", "Dee Dunn"],
    moves:      r#""#,
    page:       &["Ann Able", "Bob Baker", "Cy Cole", "Dee Dunn"],
    page_moves: r#""#,
  },
  Row {
    repro:      "marksafe_unbound_class_thanks_print_no_glyph",
    tex:        repro!("marksafe_unbound_class_thanks_print_no_glyph"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Note A."#,
      r#"Bob Baker ; thanks[†]=Note B."#,
    ],
    moves:      r#"row R SHAUTO (an unbound class's marks, from its own `\maketitle`: written none)"#,
    page:       &[
      r#"Ann Able | under: Note A."#,
      r#"Bob Baker | under: Note B."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "marksafe_unbound_class_footnotemark_given_number_unknown",
    tex:        repro!("marksafe_unbound_class_footnotemark_given_number_unknown"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal contribution."#,
      r#"Bob Baker ; thanks[*]=Equal contribution."#,
    ],
    moves:      r#"row R MARKKEY (a name's `\footnotemark[1]` paired with its note by the number); row R SHAUTO"#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#"the XML layer's: row R MARKKEY; row R SHAUTO"#,
  },
  Row {
    repro:      "marksafe_unbound_class_footnotemark_answered_by_its_text",
    tex:        repro!("marksafe_unbound_class_footnotemark_answered_by_its_text"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnote[*]=Note A."#,
      r#"Bob Baker ; footnote[†]=Note B."#,
    ],
    moves:      r#"row R SHAUTO"#,
    page:       &[
      r#"Ann Able | under: Note A."#,
      r#"Bob Baker | under: Note B."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "marksafe_unbound_class_name_footnote_unknown",
    tex:        repro!("marksafe_unbound_class_name_footnote_unknown"),
    warnings:   0,
    creators:   &[r#"Ann Able | note=Note A."#, r#"Bob Baker"#],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "marksafe_own_maketitle_copy_marks_unknown",
    tex:        repro!("marksafe_own_maketitle_copy_marks_unknown"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=First note."#,
      r#"Bob Baker ; thanks[†]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "maketitle_newcommandcopy_wrapper_red",
    tex:        repro!("maketitle_newcommandcopy_wrapper_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=First note."#,
      r#"Bob Baker ; thanks[†]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "marksafe_thefootnote_set_in_a_cell_holds_there",
    tex:        repro!("marksafe_thefootnote_set_in_a_cell_holds_there"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker ; footnotemark[2]="#,
      r#"Cy Cole ; footnotemark[‡]="#,
    ],
    moves:      r#""#,
    // (MARKKEY1d: shown, hidden before; every unlinked mark shows, OD #1059's fail-safe)
    page:       &[
      "Ann Able ; * plain",
      "Bob Baker ; 2 plain",
      "Cy Cole ; ‡ plain",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "marksafe_thefootnote_set_in_a_cell_thanks",
    tex:        repro!("marksafe_thefootnote_set_in_a_cell_thanks"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[1]=Note A."#,
      r#"Bob Baker ; thanks[†]=Note B."#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; 1 plain",
      "Bob Baker ; † plain",
      "margin: Note A. ; Note B.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "marksafe_thefootnote_set_in_a_cell_ends_at_its_line",
    tex:        repro!("marksafe_thefootnote_set_in_a_cell_ends_at_its_line"),
    warnings:   1,
    creators:   &[
      r#"Ann Able ; thanks[*]=Note A."#,
      r#"Bob Baker ; thanks[2]=Note B. | affiliation=Univ B"#,
      r#"Cat Cole ; thanks[‡]=Note C."#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; * plain",
      "Bob Baker | under: Note B.",
      "Cat Cole ; ‡ plain",
      "margin: Note A. ; Note C.",
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "marksafe_maketitle_wrapper_ieeetran_is_neutral",
    tex:        repro!("marksafe_maketitle_wrapper_ieeetran_is_neutral"),
    warnings:   0,
    creators:   &["Ann Able", "Bob Baker ; footnotemark[1]="],
    moves:      r#""#,
    // (IEEEtran's `\maketitle` empties `\footnotemark`, so TeX prints Bob's `[1]` as text, "Bob Baker[1]")
    page:       &["Ann Able", "Bob Baker[1]"],
    page_moves: r#"row R IEEEFNMARK (we print a superscript 1, TeX a literal [1]; pre-existing)"#,
  },
  // (a mark the counter steps to 0 or below in the title block, its glyph invented: "-1" before, 1910.09525's PoS shape;
  // no glyph and one Warning, DG 2026-11-17)
  Row {
    repro:      "wrongfix_title_footnotemark_counter_below_one_prints_no_glyph",
    tex:        repro!("wrongfix_title_footnotemark_counter_below_one_prints_no_glyph"),
    warnings:   1,
    creators:   &["Ann Able", "Bob Baker"],
    moves:      r#""#,
    page:       &["Ann Able", "Bob Baker"],
    page_moves: r#""#,
  },
  Row {
    repro:      "markkey_samethanks_under_elsarticle_red",
    tex:        repro!("markkey_samethanks_under_elsarticle_red"),
    warnings:   0,
    creators:   &[
      "Ann Able | note=Equal contribution.",
      "Bob Brown ; footnotemark[1]=",
    ],
    moves:      r#"row R MARKNUMKEY (elsarticle's counter not emulated: Bob's `\samethanks` 1 lost, with a Warning)"#,
    page:       &["Ann Able", "Bob Brown ; 1 plain"],
    page_moves: r#"row R MARKNUMKEY (Bob's 1 not printed)"#,
  },
  Row {
    repro:      "elsart_cortext_key_and_corref_routes_both_carry",
    tex:        repro!("elsart_cortext_key_and_corref_routes_both_carry"),
    warnings:   0,
    creators:   &[
      r#"Alice One ; thanks[*]=Corresponding author | affiliation=Some University, X, Y"#,
      r#"Bob Two ; thanks[*]=Corresponding author | affiliation=Some University, X, Y"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Alice One ; *→note"#,
      r#"Bob Two ; *→note"#,
      r#"margin: Corresponding author"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_cortext_corref_then_key_route_both_carry",
    tex:        repro!("elsart_cortext_corref_then_key_route_both_carry"),
    warnings:   0,
    creators:   &[
      r#"Alice One ; thanks[*]=Corresponding author | affiliation=Some University, X, Y"#,
      r#"Bob Two ; thanks[*]=Corresponding author | affiliation=Some University, X, Y"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Alice One ; *→note"#,
      r#"Bob Two ; *→note"#,
      r#"margin: Corresponding author"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_fntext_key_and_fnref_routes_both_carry",
    tex:        repro!("elsart_fntext_key_and_fnref_routes_both_carry"),
    warnings:   0,
    creators:   &[
      r#"Alice One ; thanks[1]=Equal contribution | affiliation=Some University, X, Y"#,
      r#"Bob Two ; thanks[1]=Equal contribution | affiliation=Some University, X, Y"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Alice One ; 1→note"#,
      r#"Bob Two ; 1→note"#,
      r#"margin: Equal contribution"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_note_keys_an_address_takes_are_no_names",
    tex:        repro!("elsart_note_keys_an_address_takes_are_no_names"),
    warnings:   0,
    creators:   &[
      r#"Alice One | address=Some University"#,
      r#"Bob Two | address=Some University ; address=Other Institute"#,
      r#" | correspondent=Corresponding author ; note=Visiting"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_key_of_a_cortext_and_an_fntext_names_the_fntext",
    tex:        repro!("elsart_key_of_a_cortext_and_an_fntext_names_the_fntext"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y ; note=Equal contribution"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | correspondent=Corresponding author"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_outside_a_name_marks_the_first_author",
    tex:        repro!("elsart_corref_outside_a_name_marks_the_first_author"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y ; correspondent=Corresponding author"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#"Carol Three | affiliation=Some University, X, Y"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#"Carol Three"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_before_the_names_marks_the_first_author",
    tex:        repro!("elsart_corref_before_the_names_marks_the_first_author"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y ; correspondent=Corresponding author"#,
      r#"Bob Two | affiliation=Some University, X, Y ; note=Visiting"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_fnref_outside_yields_to_the_first_names_own",
    tex:        repro!("elsart_fnref_outside_yields_to_the_first_names_own"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y ; correspondent=Corresponding author ; note=Note one"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#"Carol Three | affiliation=Some University, X, Y"#,
      r#" | note=Note two"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#"Carol Three"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_in_a_group_marks_no_one",
    tex:        repro!("elsart_corref_in_a_group_marks_no_one"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | correspondent=Corresponding author"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_after_a_later_name_replaces_the_first_authors",
    tex:        repro!("elsart_corref_after_a_later_name_replaces_the_first_authors"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y ; correspondent=Second corresponding"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | correspondent=First corresponding"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_fnref_after_a_later_name_replaces_the_first_authors",
    tex:        repro!("elsart_fnref_after_a_later_name_replaces_the_first_authors"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y ; note=Note two"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | note=Note one"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_before_the_names_yields_to_a_later_one",
    tex:        repro!("elsart_corref_before_the_names_yields_to_a_later_one"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y ; correspondent=Second corresponding"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | correspondent=First corresponding"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_cormark_after_a_name_marks_no_one",
    tex:        repro!("elsart_cormark_after_a_name_marks_no_one"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | correspondent=Corresponding author"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_cormark_before_the_names_marks_no_one",
    tex:        repro!("elsart_cormark_before_the_names_marks_no_one"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | correspondent=Corresponding author"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_cormark_in_the_first_name_prints_beside_the_outside_corref",
    tex:        repro!("elsart_cormark_in_the_first_name_prints_beside_the_outside_corref"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y ; correspondent=First corresponding ; correspondent=Second corresponding"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_after_a_read_note_moves_the_first_authors_mark",
    tex:        repro!("elsart_corref_after_a_read_note_moves_the_first_authors_mark"),
    warnings:   0,
    creators:   &[
      r#"Alice One | correspondent=Second corresponding ; affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | correspondent=First corresponding"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_cortext_before_the_names_keeps_the_first_authors_corref",
    tex:        repro!("elsart_cortext_before_the_names_keeps_the_first_authors_corref"),
    warnings:   0,
    creators:   &[
      r#"Alice One | correspondent=First corresponding ; affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_fnref_after_the_note_it_moves",
    tex:        repro!("elsart_fnref_after_the_note_it_moves"),
    warnings:   0,
    creators:   &[
      r#"Alice One | note=Two ; affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | note=One"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_after_a_blank_line_marks_the_first_author",
    tex:        repro!("elsart_corref_after_a_blank_line_marks_the_first_author"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y ; correspondent=Corresponding author"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_after_an_ead_marks_the_first_author",
    tex:        repro!("elsart_corref_after_an_ead_marks_the_first_author"),
    warnings:   0,
    creators:   &[
      r#"Alice One | email=a@x.org ; affiliation=Some University, X, Y ; correspondent=Corresponding author"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_cormark_list_in_a_name_marks_no_one",
    tex:        repro!("elsart_cormark_list_in_a_name_marks_no_one"),
    warnings:   1,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | correspondent=First ; correspondent=Second"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_list_in_a_name_marks_no_one",
    tex:        repro!("elsart_corref_list_in_a_name_marks_no_one"),
    warnings:   1,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | correspondent=First corresponding ; correspondent=Second corresponding"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_list_outside_marks_no_one",
    tex:        repro!("elsart_corref_list_outside_marks_no_one"),
    warnings:   1,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | correspondent=First corresponding ; correspondent=Second corresponding"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_after_the_frontmatter_keeps_the_printed_mark",
    tex:        repro!("elsart_corref_after_the_frontmatter_keeps_the_printed_mark"),
    warnings:   0,
    creators:   &[
      r#"Alice One | correspondent=Corr one ; affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | correspondent=Corr two"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_after_the_frontmatter_marks_no_one",
    tex:        repro!("elsart_corref_after_the_frontmatter_marks_no_one"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | correspondent=Corr one"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_fnref_in_the_body_marks_no_name",
    tex:        repro!("elsart_fnref_in_the_body_marks_no_name"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | note=Foot one"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_second_frontmatter_keeps_the_first_printed_mark",
    tex:        repro!("elsart_second_frontmatter_keeps_the_first_printed_mark"),
    warnings:   0,
    creators:   &[
      r#"Alice One | correspondent=Corr one ; affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | correspondent=Corr two"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_of_a_comma_key_marks_the_first_author",
    tex:        repro!("elsart_corref_of_a_comma_key_marks_the_first_author"),
    warnings:   0,
    creators:   &[
      r#"Alice One | correspondent=Corr ab ; affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_of_a_comma_key_in_a_name_marks_it",
    tex:        repro!("elsart_corref_of_a_comma_key_in_a_name_marks_it"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y"#,
      r#"Bob Two | correspondent=Corr ab ; affiliation=Some University, X, Y"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_fntext_of_a_comma_key_is_no_names",
    tex:        repro!("elsart_fntext_of_a_comma_key_is_no_names"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#"Carol Three | affiliation=Some University, X, Y"#,
      r#" | note=Foot ab"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#"Carol Three"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_with_a_trailing_comma_marks_no_one",
    tex:        repro!("elsart_corref_with_a_trailing_comma_marks_no_one"),
    warnings:   1,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y"#,
      r#"Bob Two | affiliation=Some University, X, Y"#,
      r#" | correspondent=Corr one"#,
    ],
    moves:      r#""#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#""#,
  },
  Row {
    repro:      "elsart_corref_after_maketitle_sole_author_red",
    tex:        repro!("elsart_corref_after_maketitle_sole_author_red"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y"#,
      r#" | correspondent=Corr one"#,
    ],
    moves:      r#"row R ELSSOLE (a sole author takes the block's orphaned note, relocate.rs' sole-author rule: Alice carries Corr one)"#,
    page:       &[r#"Alice One"#, r#""#],
    page_moves: r#"row R ELSSOLE"#,
  },
  Row {
    repro:      "elsart_corref_in_a_name_names_the_cortext_an_fntext_shadows_red",
    tex:        repro!("elsart_corref_in_a_name_names_the_cortext_an_fntext_shadows_red"),
    warnings:   0,
    creators:   &[
      r#"Alice One | affiliation=Some University, X, Y"#,
      r#"Bob Two | correspondent=Corr k ; affiliation=Some University, X, Y"#,
      r#" | note=Foot k"#,
    ],
    moves:      r#"row R ELSFNFALLBACK (relocate.rs' prefix-stripped fallback reads Bob's `cor:k` as `k`: Bob carries the `\fntext` too)"#,
    page:       &[r#"Alice One"#, r#"Bob Two"#, r#""#],
    page_moves: r#"row R ELSFNFALLBACK"#,
  },
  Row {
    repro:      "fmmarks_own_maketitle_relax_red",
    tex:        repro!("fmmarks_own_maketitle_relax_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=First note."#,
      r#"Bob Baker ; thanks[]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      "Ann Able ; * plain",
      "Bob Baker ; † plain",
      "margin: First note. ; Second note.",
    ],
    page_moves: r#"row R SHAUTO/T1 (the page model reads the notes under their names; in LaTeXML.css's one-line layout the marks ∗ † show, which pdflatex does not print)"#,
  },
  Row {
    repro:      "fmmarks_mnras_title_marks_are_the_class_symbols",
    tex:        repro!("fmmarks_mnras_title_marks_are_the_class_symbols"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[⋆]=E-mail: ann@x.org | affiliation=Univ A, Town"#,
      r#"Bob Baker ; footnote[†]=Bob note. | affiliation=Univ B, City"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: E-mail: ann@x.org"#,
      r#"Bob Baker | under: Bob note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_mnras_title_marks_follow_thefootnote_in_force",
    tex:        repro!("fmmarks_mnras_title_marks_follow_thefootnote_in_force"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[1]=E-mail: ann@x.org | affiliation=Univ A, Town"#,
      r#"Bob Baker ; thanks[2]=Bob note. | affiliation=Univ B, City"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: E-mail: ann@x.org"#,
      r#"Bob Baker | under: Bob note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_mnras_no_maketitle_footnotetext",
    tex:        repro!("fmmarks_mnras_no_maketitle_footnotetext"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Univ A"#,
      r#"Bob Baker ; footnote[2]=Bob text. | affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker | under: Bob text."#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_article_no_maketitle_footnotetext_red",
    tex:        repro!("fmmarks_article_no_maketitle_footnotetext_red"),
    warnings:   0,
    creators:   &[r#"Ann Able"#, r#"Bob Baker ; footnote[2]=Bob text."#],
    moves:      r#"row R NOMTHOOK (proposed: the title hook applied only where a `\maketitle` digests the field; written †)"#,
    // (revMARKSAFEc S3 card rule, MARKKEY1: one-line block, the mark shows and the note is a pop-up)
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; 2 plain"#,
      r#"margin: Bob text."#,
    ],
    page_moves: r#"row R NOMTHOOK (written †)"#,
  },
  Row {
    repro:      "fmmarks_report_titlepage_marks_in_force_red",
    tex:        repro!("fmmarks_report_titlepage_marks_in_force_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[2]=Ann note."#,
      r#"Bob Baker ; thanks[3]=Bob note."#,
    ],
    moves:      r#"row R TITLEPAGEMARKS (a title page declares no hook: written unmarked, the fail-safe)"#,
    page:       &[
      r#"Ann Able | under: Ann note."#,
      r#"Bob Baker | under: Bob note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_report_titling_fnsymbol_titlepage_red",
    tex:        repro!("fmmarks_report_titling_fnsymbol_titlepage_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Ann note."#,
      r#"Bob Baker ; thanks[†]=Bob note."#,
    ],
    moves:      r#"row R TITLEPAGEMARKS (titling's series, set as its `\maketitle` begins: written unmarked, the fail-safe)"#,
    page:       &[
      r#"Ann Able | under: Ann note."#,
      r#"Bob Baker | under: Bob note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_report_titlepage_empty_makefnmark",
    tex:        repro!("fmmarks_report_titlepage_empty_makefnmark"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=Ann note."#,
      r#"Bob Baker ; thanks[]=Bob note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Ann note."#,
      r#"Bob Baker | under: Bob note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_mnras_late_maketitle_red",
    tex:        repro!("fmmarks_mnras_late_maketitle_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[2]=E-mail: ann@x.org | affiliation=Univ A"#,
      r#"Bob Baker ; thanks[3]=Second. | affiliation=Univ A"#,
    ],
    moves:      r#"row R MNLATE (proposed: a `\maketitle` after the fallback flush digests nothing, so its notes neither mark nor step)"#,
    page:       &[
      r#"Ann Able | under: E-mail: ann@x.org"#,
      r#"Bob Baker | under: Second."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_robust_maketitle_wrapper_red",
    tex:        repro!("fmmarks_robust_maketitle_wrapper_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=First note."#,
      r#"Bob Baker ; thanks[†]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_two_level_maketitle_wrapper_red",
    tex:        repro!("fmmarks_two_level_maketitle_wrapper_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=First note."#,
      r#"Bob Baker ; thanks[†]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_private_class_on_revtex_own_maketitle_red",
    tex:        repro!("fmmarks_private_class_on_revtex_own_maketitle_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[1]=First note."#,
      r#"Bob Baker ; thanks[2]=Second note."#,
    ],
    moves:      r#"row R SHAUTO/T1 (a private class's own `\maketitle` over a bound class: the binding's ∗ † written)"#,
    page:       &[
      r#"Ann Able | under: First note."#,
      r#"Bob Baker | under: Second note."#,
    ],
    page_moves: r#"row R SHAUTO/T1 (the page model reads the notes under their names; in LaTeXML.css's one-line layout the marks ∗ † show where pdflatex prints 1, 2)"#,
  },
  Row {
    repro:      "fmmarks_llncs_title_marks_are_the_class_symbols",
    tex:        repro!("fmmarks_llncs_title_marks_are_the_class_symbols"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[⋆]=Ann note. | affiliation=Lab A"#,
      r#"Bob Baker ; thanks[⋆⋆]=Bob note. | affiliation=Lab A"#,
    ],
    moves:      r#""#,
    // (revMARKSAFEc S3 card rule, MARKKEY1: one-line block, the mark shows and the note is a pop-up)
    page:       &[
      r#"Ann Able ; ⋆ plain"#,
      r#"Bob Baker ; ⋆⋆ plain"#,
      r#"margin: Ann note. ; Bob note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_llncs_preamble_thefootnote_is_not_the_titles",
    tex:        repro!("fmmarks_llncs_preamble_thefootnote_is_not_the_titles"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[⋆]=Ann note. | affiliation=Lab A"#,
      r#"Bob Baker ; thanks[⋆⋆]=Bob note. | affiliation=Lab A"#,
    ],
    moves:      r#""#,
    // (revMARKSAFEc S3 card rule, MARKKEY1: one-line block, the mark shows and the note is a pop-up)
    page:       &[
      r#"Ann Able ; ⋆ plain"#,
      r#"Bob Baker ; ⋆⋆ plain"#,
      r#"margin: Ann note. ; Bob note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_llncs_counting_pass_steps_twice_red",
    tex:        repro!("fmmarks_llncs_counting_pass_steps_twice_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[⋆⋆]=Ann note. | affiliation=Lab A"#,
      r#"Bob Baker ; footnotemark[⋆⋆⋆]= | affiliation=Lab A"#,
    ],
    moves:      r#"row R LLNCSTWICE (proposed: llncs's counting pass of `\@author` steps a bare `\footnotemark` twice; the binding withdraws its marks: Ann's thanks unmarked, Bob's mark none, a Warning; LLNCSMARKS wrote the body's "1")"#,
    page:       &[r#"Ann Able | under: Ann note."#, r#"Bob Baker ; ⋆⋆⋆ plain"#],
    page_moves: r#"row R LLNCSTWICE (Bob's mark none)"#,
  },
  Row {
    repro:      "fmmarks_llncs_names_macro_steps_twice_red",
    tex:        repro!("fmmarks_llncs_names_macro_steps_twice_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[⋆⋆]=Ann note. | affiliation=Lab A"#,
      r#"Bob Baker ; footnotemark[⋆⋆⋆]= | affiliation=Lab A"#,
    ],
    moves:      r#"row R LLNCSTWICE (proposed: the binding withdraws its marks for a macro holding `\footnotemark`, read by meaning: Ann's thanks unmarked, Bob's mark none, a Warning)"#,
    page:       &[r#"Ann Able | under: Ann note."#, r#"Bob Baker ; ⋆⋆⋆ plain"#],
    page_moves: r#"row R LLNCSTWICE (Bob's mark none)"#,
  },
  Row {
    repro:      "fmmarks_llncs_subtitle_note_counts_first_red",
    tex:        repro!("fmmarks_llncs_subtitle_note_counts_first_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[⋆⋆]=Ann note. | affiliation=Lab A"#,
      r#"Bob Baker ; thanks[⋆⋆⋆]=Bob note. | affiliation=Lab A"#,
    ],
    moves:      r#"row R LLNCSSUBTITLE (proposed: the title block's rehearsal counts no `\subtitle` note; the binding withdraws its marks, the names' thanks unmarked)"#,
    page:       &[
      r#"Ann Able | under: Ann note."#,
      r#"Bob Baker | under: Bob note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_llncs_institute_mark_steps_twice_red",
    tex:        repro!("fmmarks_llncs_institute_mark_steps_twice_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[†]= ; thanks[⋆]=Ann note. | affiliation=Lab A"#,
      r#"Bob Baker ; thanks[⋆⋆]=Bob note. | affiliation=Lab B"#,
    ],
    moves:      r#"row R LLNCSINSTTWICE (proposed: `\institutename`'s counting pass steps an institute's bare `\footnotemark` twice, llncs.cls:877; ours counts it once, ⋆⋆⋆)"#,
    page:       &[
      r#"Ann Able ; † plain | under: Ann note."#,
      r#"Bob Baker | under: Bob note."#,
    ],
    page_moves: r#"row R LLNCSINSTTWICE (Lab A's mark ⋆⋆⋆)"#,
  },
  Row {
    repro:      "fmmarks_llncs_replaced_author_steps_red",
    tex:        repro!("fmmarks_llncs_replaced_author_steps_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[⋆]=Ann note. | affiliation=Lab A"#,
      r#"Bob Baker ; thanks[⋆⋆]=Bob note. | affiliation=Lab A"#,
    ],
    moves:      r#"row R AUTHRESET (the reading's rehearsal steps the replaced `\author`'s notes; the binding withdraws its marks, the names' thanks unmarked; LLNCSb wrote ⋆⋆ ⋆⋆⋆)"#,
    page:       &[
      r#"Ann Able | under: Ann note."#,
      r#"Bob Baker | under: Bob note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_llncs_replaced_author_mark_red",
    tex:        repro!("fmmarks_llncs_replaced_author_mark_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[⋆]=Ann note. | affiliation=Lab A"#,
      r#"Bob Baker ; thanks[⋆⋆]=Bob note. | affiliation=Lab A"#,
    ],
    moves:      r#"row R AUTHRESET (as fmmarks_llncs_replaced_author_steps_red, two steps carried; LLNCSb wrote ⋆⋆⋆ †)"#,
    page:       &[
      r#"Ann Able | under: Ann note."#,
      r#"Bob Baker | under: Bob note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_article_replaced_author_steps_red",
    tex:        repro!("fmmarks_article_replaced_author_steps_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Ann note."#,
      r#"Bob Baker ; thanks[†]=Bob note."#,
    ],
    moves:      r#"row R AUTHRESET (the reading's rehearsal steps the replaced `\author`'s notes: ‡ § written)"#,
    // (revMARKSAFEc S3 card rule, MARKKEY1: one-line block, the mark shows and the note is a pop-up)
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"margin: Ann note. ; Bob note."#,
    ],
    page_moves: r#"row R AUTHRESET (the reading's rehearsal steps the replaced `\author`'s notes: ‡ § written)"#,
  },
  Row {
    repro:      "fmmarks_llncs_ifdefined_names_keep_marks",
    tex:        repro!("fmmarks_llncs_ifdefined_names_keep_marks"),
    warnings:   1,
    creators:   &[
      r#"Ann Able ; thanks[⋆]=Ann note. | affiliation=Lab A"#,
      r#"Bob Baker ; thanks[⋆⋆]=Bob note. | affiliation=Lab A"#,
    ],
    moves:      r#""#,
    // (revMARKSAFEc S3 card rule, MARKKEY1: one-line block, the mark shows and the note is a pop-up)
    page:       &[
      r#"Ann Able ; ⋆ plain"#,
      r#"Bob Baker ; ⋆⋆ plain"#,
      r#"margin: Ann note. ; Bob note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_llncs_ifx_names_keep_marks",
    tex:        repro!("fmmarks_llncs_ifx_names_keep_marks"),
    warnings:   1,
    creators:   &[
      r#"Ann Able ; thanks[⋆]=Ann note. | affiliation=Lab A"#,
      r#"Bob Baker ; thanks[⋆⋆]=Bob note. | affiliation=Lab A"#,
    ],
    moves:      r#""#,
    // (revMARKSAFEc S3 card rule, MARKKEY1: one-line block, the mark shows and the note is a pop-up)
    page:       &[
      r#"Ann Able ; ⋆ plain"#,
      r#"Bob Baker ; ⋆⋆ plain"#,
      r#"margin: Ann note. ; Bob note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_llncs_names_macro_defined_later",
    tex:        repro!("fmmarks_llncs_names_macro_defined_later"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[⋆]=Ann note. | affiliation=Lab A"#,
      r#"Bob Baker ; footnotemark[c]= ; thanks[⋆⋆]=Bob note. | affiliation=Lab A"#,
    ],
    moves:      r#""#,
    // (revMARKSAFEc S3 card rule, MARKKEY1: one-line block, the mark shows and the note is a pop-up)
    page:       &[
      r#"Ann Able ; ⋆ plain"#,
      r#"Bob Baker ; c plain ; ⋆⋆ plain"#,
      r#"margin: Ann note. ; Bob note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_llncs_title_note_counts_first",
    tex:        repro!("fmmarks_llncs_title_note_counts_first"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[⋆⋆]=Ann note. | affiliation=Lab A"#,
      r#"Bob Baker ; thanks[⋆⋆⋆]=Bob note. | affiliation=Lab A"#,
      r#"Cy Cole ; thanks[†]=Cy note. | affiliation=Lab A"#,
    ],
    moves:      r#""#,
    // (revMARKSAFEc S3 card rule, MARKKEY1: one-line block, the mark shows and the note is a pop-up)
    page:       &[
      r#"Ann Able ; ⋆⋆ plain"#,
      r#"Bob Baker ; ⋆⋆⋆ plain"#,
      r#"Cy Cole ; † plain"#,
      r#"margin: Ann note. ; Bob note. ; Cy note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbx_spconf_empty_thefootnote_prints_no_mark",
    tex:        repro!("presbx_spconf_empty_thefootnote_prints_no_mark"),
    warnings:   0,
    // (the unmarked `\address` under the names is every author's affiliation, SPCONF, OD #1073)
    creators:   &[
      r#"Ann Able | affiliation=University of Somewhere"#,
      r#"Bob Baker ; thanks[]=Funded by X. | affiliation=University of Somewhere"#,
      r#"Cy Cole ; thanks[†]=Corresponding author. Lab B. | affiliation=University of Somewhere"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker | under: Funded by X."#,
      r#"Cy Cole ; † plain"#,
      r#"margin: Corresponding author. Lab B."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbx_package_maketitle_renewed_thefootnote",
    tex:        repro!("presbx_package_maketitle_renewed_thefootnote"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[1]=Note A."#,
      r#"Bob Baker ; thanks[2]=Note B."#,
    ],
    moves:      r#"row R MTREDUCE (DG 2026-11-17 maketitle reduce: a `\maketitle` not the class's and no binding's, so its marks are unknown here and no glyph is written — formerly rule 2, the body's `\renewcommand\thefootnote`; each note stays under its name, the connection unchanged)"#,
    page:       &[
      r#"Ann Able ; 1 plain"#,
      r#"Bob Baker ; 2 plain"#,
      r#"margin: Note A. ; Note B."#,
    ],
    page_moves: r#"row R MTREDUCE (no glyph: each note under its name)"#,
  },
  Row {
    repro:      "presbx_lomcon_style_option_alph_marks",
    tex:        repro!("presbx_lomcon_style_option_alph_marks"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[a]=e-mail: ann@x.org | address=University of Somewhere"#,
      r#"Bob Baker ; thanks[b]=e-mail: bob@y.org | address=University of Somewhere"#,
      r#"Cy Cole | address=University of Somewhere"#,
    ],
    moves:      r#"row R MTREDUCE (DG 2026-11-17 maketitle reduce: a `\maketitle` not the class's and no binding's, so its marks are unknown here and no glyph is written — formerly rule 4, the `\thefootnote` in force under a body touching no mark (11lomcon, hep-ph0312288); each note stays under its name, the connection unchanged)"#,
    page:       &[
      r#"Ann Able ; a plain"#,
      r#"Bob Baker ; b plain"#,
      r#"Cy Cole"#,
      r#"margin: e-mail: ann@x.org ; e-mail: bob@y.org"#,
    ],
    page_moves: r#"row R MTREDUCE (no glyph: each note under its name)"#,
  },
  Row {
    repro:      "presbxb_restored_maketitle_is_the_class_s",
    tex:        repro!("presbxb_restored_maketitle_is_the_class_s"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=First note."#,
      r#"Bob Baker ; thanks[*]=First note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"margin: First note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbxb_restored_maketitle_is_the_saved_one",
    tex:        repro!("presbxb_restored_maketitle_is_the_saved_one"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[a]=First note."#,
      r#"Bob Baker ; thanks[b]=Second note."#,
    ],
    moves:      r#"row R MTREDUCE (DG 2026-11-17 maketitle reduce: a `\maketitle` not the class's and no binding's, so its marks are unknown here and no glyph is written — formerly rule 2, the saved body's `\thefootnote`; each note stays under its name, the connection unchanged)"#,
    page:       &[
      r#"Ann Able ; a plain"#,
      r#"Bob Baker ; b plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#"row R MTREDUCE (no glyph: each note under its name)"#,
  },
  Row {
    repro:      "presbxb_restored_maketitle_at_begin_document",
    tex:        repro!("presbxb_restored_maketitle_at_begin_document"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=First note."#,
      r#"Bob Baker ; thanks[†]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbxb_maketitle_fnsymbol_reads_the_document_s",
    tex:        repro!("presbxb_maketitle_fnsymbol_reads_the_document_s"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[†]=First note."#,
      r#"Bob Baker ; thanks[‡]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; † plain"#,
      r#"Bob Baker ; ‡ plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbxb_maketitle_csname_thefootnote_marks_unknown",
    tex:        repro!("presbxb_maketitle_csname_thefootnote_marks_unknown"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=First note."#,
      r#"Bob Baker ; thanks[]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: First note."#,
      r#"Bob Baker | under: Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbxb_maketitle_deep_thefootnote_marks_unknown",
    tex:        repro!("presbxb_maketitle_deep_thefootnote_marks_unknown"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=First note."#,
      r#"Bob Baker ; thanks[]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: First note."#,
      r#"Bob Baker | under: Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbxb_maketitle_let_makefnmark_reads_its_source",
    tex:        repro!("presbxb_maketitle_let_makefnmark_reads_its_source"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=First note."#,
      r#"Bob Baker ; thanks[†]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbxb_mark_order_on_one_name_red",
    tex:        repro!("presbxb_mark_order_on_one_name_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Equal."#,
      r#"Bob Baker ; thanks[†]=Corresponding. ; thanks[*]=Equal."#,
      r#"Cy Cole ; thanks[†]=Corresponding."#,
    ],
    moves:      r#"row R MARKORDER (the copy of note 1 written before the name's own note; pdflatex prints "†,∗")"#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; †→note ; *→note"#,
      r#"Cy Cole ; †→note"#,
      r#"margin: Equal. ; Corresponding."#,
    ],
    page_moves: r#"row R MARKORDER (Bob's ∗ shown before his †)"#,
  },
  Row {
    repro:      "presbxc_maketitle_footnote_before_the_title (PDF)",
    tex:        repro!("presbxc_maketitle_footnote_before_the_title"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[†]=First note."#,
      r#"Bob Baker ; thanks[‡]=Second note."#,
    ],
    moves:      r#"row R SHAUTO/T1 (unknown here, the fail-safe writes no glyph)"#,
    page:       &[
      r#"Ann Able ; † plain"#,
      r#"Bob Baker ; ‡ plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#"row R SHAUTO/T1 (no glyph shown)"#,
  },
  Row {
    repro:      "presbxc_maketitle_footnote_before_the_title",
    tex:        repro!("presbxc_maketitle_footnote_before_the_title"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=First note."#,
      r#"Bob Baker ; thanks[]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: First note."#,
      r#"Bob Baker | under: Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbxc_package_maketitle_footnote_in_force (PDF)",
    tex:        repro!("presbxc_package_maketitle_footnote_in_force"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[†]=First note."#,
      r#"Bob Baker ; thanks[‡]=Second note."#,
    ],
    moves:      r#"row R SHAUTO/T1 (unknown here, the fail-safe writes no glyph)"#,
    page:       &[
      r#"Ann Able ; † plain"#,
      r#"Bob Baker ; ‡ plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#"row R SHAUTO/T1 (no glyph shown)"#,
  },
  Row {
    repro:      "presbxc_package_maketitle_footnote_in_force",
    tex:        repro!("presbxc_package_maketitle_footnote_in_force"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=First note."#,
      r#"Bob Baker ; thanks[]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: First note."#,
      r#"Bob Baker | under: Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbxc_maketitle_setcounter_footnote_before_the_title (PDF)",
    tex:        repro!("presbxc_maketitle_setcounter_footnote_before_the_title"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[‡]=First note."#,
      r#"Bob Baker ; thanks[§]=Second note."#,
    ],
    moves:      r#"row R SHAUTO/T1 (unknown here, the fail-safe writes no glyph)"#,
    page:       &[
      r#"Ann Able ; ‡ plain"#,
      r#"Bob Baker ; § plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#"row R SHAUTO/T1 (no glyph shown)"#,
  },
  Row {
    repro:      "presbxc_maketitle_setcounter_footnote_before_the_title",
    tex:        repro!("presbxc_maketitle_setcounter_footnote_before_the_title"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=First note."#,
      r#"Bob Baker ; thanks[]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: First note."#,
      r#"Bob Baker | under: Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbxc_package_restores_refused_maketitle (PDF)",
    tex:        repro!("presbxc_package_restores_refused_maketitle"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=First note."#,
      r#"Bob Baker ; thanks[†]=Second note."#,
    ],
    moves:      r#"row R SHAUTO/T1 (unknown here, the fail-safe writes no glyph)"#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#"row R SHAUTO/T1 (no glyph shown)"#,
  },
  Row {
    repro:      "presbxc_package_restores_refused_maketitle",
    tex:        repro!("presbxc_package_restores_refused_maketitle"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=First note."#,
      r#"Bob Baker ; thanks[]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: First note."#,
      r#"Bob Baker | under: Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbxc_maketitle_setcounter_footnote_zero_keeps_its_marks",
    tex:        repro!("presbxc_maketitle_setcounter_footnote_zero_keeps_its_marks"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=First note."#,
      r#"Bob Baker ; thanks[†]=Second note."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "presbxc_package_copies_refused_maketitle_keeps_it",
    tex:        repro!("presbxc_package_copies_refused_maketitle_keeps_it"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[a]=First note."#,
      r#"Bob Baker ; thanks[b]=Second note."#,
    ],
    moves:      r#"row R MTREDUCE (DG 2026-11-17 maketitle reduce: a `\maketitle` not the class's and no binding's, so its marks are unknown here and no glyph is written — formerly rule 2, a package's copy of the refused body; each note stays under its name, the connection unchanged)"#,
    page:       &[
      r#"Ann Able ; a plain"#,
      r#"Bob Baker ; b plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#"row R MTREDUCE (no glyph: each note under its name)"#,
  },
  Row {
    repro:      "presbxc_wrapper_of_a_copy_after_refusal_red",
    tex:        repro!("presbxc_wrapper_of_a_copy_after_refusal_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[a]=First note."#,
      r#"Bob Baker ; thanks[b]=Second note."#,
    ],
    moves:      r#"row R PRESBX/copywrap (the wrapper of a copy taken after the refusal read as the class's: ∗, †)"#,
    page:       &[
      r#"Ann Able ; a plain"#,
      r#"Bob Baker ; b plain"#,
      r#"margin: First note. ; Second note."#,
    ],
    page_moves: r#"row R PRESBX/copywrap (∗, † shown)"#,
  },
  Row {
    repro:      "fmmarks_achemso_affiliation_reaches_the_waiting_authors",
    tex:        repro!("fmmarks_achemso_affiliation_reaches_the_waiting_authors"),
    warnings:   1,
    creators:   &[
      r#"Ann Able | affiliation=Univ A"#,
      r#"Bob Baker | affiliation=Univ A ; email=bob@x.org"#,
      r#"Cy Cole | affiliation=Univ B, Town"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#, r#"Cy Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_achemso_repeated_affiliation_text_is_one_affiliation",
    tex:        repro!("fmmarks_achemso_repeated_affiliation_text_is_one_affiliation"),
    warnings:   1,
    creators:   &[
      r#"Ann Able | affiliation=Univ A"#,
      r#"Bob Baker | affiliation=Univ B"#,
      r#"Cy Cole | affiliation=Univ A ; email=cy@x.org"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#, r#"Cy Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_achemso_alsoaffiliation_is_the_last_authors",
    tex:        repro!("fmmarks_achemso_alsoaffiliation_is_the_last_authors"),
    warnings:   1,
    creators:   &[
      r#"Ann Able | affiliation=Univ A ; affiliation=Univ B"#,
      r#"Bob Baker | affiliation=Univ B ; affiliation=Univ C ; email=bob@x.org"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_achemso_altaffiliation_is_the_last_authors",
    tex:        repro!("fmmarks_achemso_altaffiliation_is_the_last_authors"),
    warnings:   1,
    creators:   &[
      r#"Ann Able | affiliation=Univ A ; altaffiliation=Current address: Lab C"#,
      r#"Bob Baker | affiliation=Univ A ; email=bob@x.org"#,
      r#"Cy Cole | affiliation=Univ B ; altaffiliation=Current address: Lab D"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#, r#"Cy Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_achemso_email_phone_fax_are_the_last_authors",
    tex:        repro!("fmmarks_achemso_email_phone_fax_are_the_last_authors"),
    warnings:   1,
    creators:   &[
      r#"Ann Able | affiliation=Univ A ; email=ann_able@x.org ; phone=+1 555 0100 ; fax=+1 555 0101"#,
      r#"Bob Baker | affiliation=Univ A ; email=bob@x.org ; phone=+1 555 0200"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_achemso_affiliation_after_affiliation_reaches_no_author",
    tex:        repro!("fmmarks_achemso_affiliation_after_affiliation_reaches_no_author"),
    warnings:   1,
    creators:   &[
      r#"Ann Able | affiliation=Univ A"#,
      r#"Bob Baker | affiliation=Univ A"#,
      r#"Cy Cole | affiliation=Univ C ; email=cy@x.org"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#, r#"Cy Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_achemso_new_alsoaffiliation_is_the_waiting_authors",
    tex:        repro!("fmmarks_achemso_new_alsoaffiliation_is_the_waiting_authors"),
    warnings:   1,
    creators:   &[
      r#"Ann Able | affiliation=Univ B"#,
      r#"Bob Baker | affiliation=Univ B"#,
      r#"Cy Cole | affiliation=Univ A"#,
      r#"Dan Dorn | affiliation=Univ C ; email=dan@x.org"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#, r#"Cy Cole"#, r#"Dan Dorn"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_achemso_pieces_before_any_author_warn",
    tex:        repro!("fmmarks_achemso_pieces_before_any_author_warn"),
    warnings:   3,
    creators:   &[r#"Ann Able | affiliation=Univ A ; email=ann@x.org"#],
    moves:      r#""#,
    page:       &[r#"Ann Able"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_achemso_witness_run_of_two_takes_a_repeated_affiliation",
    tex:        repro!("fmmarks_achemso_witness_run_of_two_takes_a_repeated_affiliation"),
    warnings:   1,
    creators:   &[
      r#"Ann Able | affiliation=Univ A, Town ; affiliation=Inst Q, Town ; email=ann@x.org"#,
      r#"Bob Baker | affiliation=Univ A, Town ; affiliation=Inst Q, Town ; affiliation=Inst K, Town"#,
      r#"Cy Cole | affiliation=Inst Q, Town"#,
      r#"Dan Dorn | affiliation=Inst Q, Town"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#, r#"Cy Cole"#, r#"Dan Dorn"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_achemso_pieces_no_author_holds_stay_unlinked",
    tex:        repro!("fmmarks_achemso_pieces_no_author_holds_stay_unlinked"),
    warnings:   1,
    creators:   &[
      r#"Ann Able | affiliation=Univ A ; altaffiliation=Lab Y"#,
      r#"Bob Baker | affiliation=Univ B ; email=bob@x.org"#,
      r#"Cy Cole | affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#, r#"Cy Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_achemso_email_list_addresses_go_to_the_authors_they_spell",
    tex:        repro!("fmmarks_achemso_email_list_addresses_go_to_the_authors_they_spell"),
    warnings:   2,
    creators:   &[
      r#"Ann Able | affiliation=Univ A ; email=ann.able@x.org"#,
      r#"Bob Baker | affiliation=Univ A"#,
      r#"Cy Cole | affiliation=Univ A ; email=cy.cole@x.org"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#, r#"Cy Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_achemso_replaced_altaffiliation_is_one_note",
    tex:        repro!("fmmarks_achemso_replaced_altaffiliation_is_one_note"),
    warnings:   1,
    creators:   &[
      r#"Ann Able | affiliation=Univ X ; altaffiliation=Now at Q ; email=ann@x.edu"#,
      r#"Bob Baker | affiliation=Univ Y ; altaffiliation=Now at R"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_achemso_second_email_phone_fax_replace_the_first",
    tex:        repro!("fmmarks_achemso_second_email_phone_fax_replace_the_first"),
    warnings:   1,
    creators:   &[
      r#"Ann Able | affiliation=Univ X ; email=ann2@y.edu ; phone=456 ; fax=789"#,
      r#"Bob Baker | affiliation=Univ X"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_achemso_alsoaffiliation_after_a_two_name_author_red",
    tex:        repro!("fmmarks_achemso_alsoaffiliation_after_a_two_name_author_red"),
    warnings:   1,
    creators:   &[
      r#"Ann Able | affiliation=Univ A ; affiliation=Univ C"#,
      r#"Bob Baker | affiliation=Univ A ; affiliation=Univ C"#,
      r#"Cy Cole | affiliation=Univ B ; email=cy@x.org"#,
    ],
    moves:      r#"row R ACHEMSO-CALLS (the API's annotate counts creators, not author calls: an \alsoaffiliation after a two-name \author reaches its last name only)"#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#, r#"Cy Cole"#],
    page_moves: r#""#,
  },
  // sn-jnl (SNJNL): `\equalcont`'s † and `\author*`'s * are the marks of one note each, printed once (2412.01641's
  // sn-jnl.cls:487-596, :742-756; sn_jnl_cls.rs)
  Row {
    repro:      "snjnl_equalcont_marks_its_authors_one_note",
    tex:        repro!("snjnl_equalcont_marks_its_authors_one_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; equal-contributors[†]=These authors contributed equally to this work. | email=ann@a.org ; affiliation=Univ A"#,
      r#"Bob Baker ; equal-contributors[†]=These authors contributed equally to this work. | email=bob@b.org ; affiliation=Univ A ; affiliation=Univ B"#,
      r#"Cat Cole | affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; †→note"#,
      r#"Bob Baker ; †→note"#,
      r#"Cat Cole"#,
      r#"margin: These authors contributed equally to this work."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "snjnl_corresponding_star_is_one_note",
    tex:        repro!("snjnl_corresponding_star_is_one_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; correspondent[*]=Corresponding author(s). | email=ann@a.org ; affiliation=Univ A"#,
      r#"Bob Baker | email=bob@b.org ; affiliation=Univ A"#,
      r#"Cat Cole ; correspondent[*]=Corresponding author(s). | email=cat@c.org ; affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker"#,
      r#"Cat Cole ; *→note"#,
      r#"margin: Corresponding author(s)."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "snjnl_affil_labels_several_per_name",
    tex:        repro!("snjnl_affil_labels_several_per_name"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Univ A ; affiliation=Univ C"#,
      r#"Bob Baker | affiliation=Univ B"#,
      r#"Cat Cole | affiliation=Univ A ; affiliation=Univ B ; affiliation=Univ C"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#, r#"Cat Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "snjnl_witness_equalcont_and_star",
    tex:        repro!("snjnl_witness_equalcont_and_star"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; equal-contributors[†]=These authors contributed equally to this work. | email=ann@a.org ; affiliation=Dept A, Univ A, Town A, China ; affiliation=Univ B"#,
      r#"Bob Baker ; equal-contributors[†]=These authors contributed equally to this work. | email=bob@b.org ; affiliation=Dept A, Univ A, Town A, China ; affiliation=Univ C"#,
      r#"Cat Cole ; correspondent[*]=Corresponding author(s). | email=cat@c.org ; affiliation=Univ C"#,
      r#"Dan Dorn | email=dan@d.org ; affiliation=Dept A, Univ A, Town A, China ; affiliation=Univ C ; affiliation=Univ D"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; †→note"#,
      r#"Bob Baker ; †→note"#,
      r#"Cat Cole ; * plain"#,
      r#"Dan Dorn"#,
      r#"margin: Corresponding author(s). ; These authors contributed equally to this work."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "snjnl_witness_star_and_starred_affil",
    tex:        repro!("snjnl_witness_star_and_starred_affil"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; correspondent[*]=Corresponding author(s). | email=ann@a.org ; affiliation=Dept A, Univ A, Town, 02139, MA, USA"#,
      r#"Bob Baker | affiliation=Dept A, Univ A, Town, 02139, MA, USA"#,
      r#"Cat Cole | affiliation=Dept A, Univ A, Town, 02139, MA, USA"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker"#,
      r#"Cat Cole"#,
      r#"margin: Corresponding author(s)."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "snjnl_equalcont_last_text_and_email_in_name",
    tex:        repro!("snjnl_equalcont_last_text_and_email_in_name"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; equal-contributors[†]=Equal contribution. | affiliation=Univ A"#,
      r#"Bob Baker ; correspondent[*]=Corresponding author(s). ; equal-contributors[†]=Equal contribution. | email=bob@b.org ; affiliation=Univ A"#,
      r#"Cat Cole ; correspondent[*]=Corresponding author(s). ; equal-contributors[†]=Equal contribution. | email=cat@c.org ; affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; †→note"#,
      r#"Bob Baker ; *→note ; †→note"#,
      r#"Cat Cole ; *→note ; †→note"#,
      r#"margin: Corresponding author(s). ; Equal contribution."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "snjnl_starred_names_without_email",
    tex:        repro!("snjnl_starred_names_without_email"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= | affiliation=Univ A"#,
      r#"Bob Baker | affiliation=Univ A"#,
      r#"Cat Cole ; footnotemark[*]= | affiliation=Univ B"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker"#,
      r#"Cat Cole ; * plain"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "snjnl_sole_starred_author_without_labels",
    tex:        repro!("snjnl_sole_starred_author_without_labels"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; correspondent[]=Corresponding author(s). | email=ann@a.org ; affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able | under: Corresponding author(s)."#],
    page_moves: r#""#,
  },
  // (the labels and the `*` print as the names print, under the last `\author`'s labels, :498, :510: revSNJNL B1)
  Row {
    repro:      "snjnl_last_author_unlabelled_prints_no_star",
    tex:        repro!("snjnl_last_author_unlabelled_prints_no_star"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; correspondent[]=Corresponding author(s). | email=ann@a.org ; affiliation=Univ A"#,
      r#"Bob Baker | email=bob@b.org ; affiliation=Univ A"#,
      r#"Cat Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Corresponding author(s)."#,
      r#"Bob Baker"#,
      r#"Cat Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "snjnl_unlabelled_star_under_labelled_last_prints_star",
    tex:        repro!("snjnl_unlabelled_star_under_labelled_last_prints_star"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; correspondent[*]=Corresponding author(s). | email=ann@a.org"#,
      r#"Bob Baker | email=bob@b.org ; affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker"#,
      r#"margin: Corresponding author(s)."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "snjnl_unlabelled_star_without_line_shows",
    tex:        repro!("snjnl_unlabelled_star_without_line_shows"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker | email=bob@b.org ; affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able ; * plain"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  // (the printed `*` is the label `*`, which an `\affil[*]` line answers: 2305.11920)
  Row {
    repro:      "snjnl_affil_star_line_explains_the_star",
    tex:        repro!("snjnl_affil_star_line_explains_the_star"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Univ A ; affiliation=Corresponding author: Ann Able ; email=ann@a.org"#,
      r#"Bob Baker | affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  // (the names flushed before `\maketitle` by a section: the star's label is the name's own, 2510.03878)
  Row {
    repro:      "snjnl_section_before_maketitle_keeps_the_star_note",
    tex:        repro!("snjnl_section_before_maketitle_keeps_the_star_note"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; correspondent[*]=Corresponding author(s). | email=ann@a.org ; affiliation=Univ A"#,
      r#"Bob Baker | affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker"#,
      r#"margin: Corresponding author(s)."#,
    ],
    page_moves: r#""#,
  },
  // (an unlabelled last `\author` prints no `*`: a starred name's unanswered `*` label shows nothing, 2606.13079;
  // revSNJNLb B1)
  Row {
    repro:      "snjnl_unprinted_star_on_a_name_written_as_text",
    tex:        repro!("snjnl_unprinted_star_on_a_name_written_as_text"),
    warnings:   2,
    creators:   &[
      r#"Ann Able | affiliation=† Univ A"#,
      r#"Bob Baker | affiliation=† Univ A"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "snjnl_labelled_star_typed_dagger_under_unlabelled_last",
    tex:        repro!("snjnl_labelled_star_typed_dagger_under_unlabelled_last"),
    warnings:   1,
    creators:   &[
      r#"Bob Baker ; footnotemark[†]= | affiliation=Univ A"#,
      r#"Cat Cole"#,
    ],
    moves:      r#""#,
    page:       &[r#"Bob Baker ; † plain"#, r#"Cat Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "snjnl_star_shapes_under_unlabelled_last_print_no_star",
    tex:        repro!("snjnl_star_shapes_under_unlabelled_last_print_no_star"),
    warnings:   2,
    creators:   &[
      r#"Ann Able ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Cat Cole ; footnotemark[†]= | affiliation=Univ A"#,
      r#"Dan Dorn"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; † plain"#,
      r#"Bob Baker"#,
      r#"Cat Cole ; † plain"#,
      r#"Dan Dorn"#,
    ],
    page_moves: r#""#,
  },
  // (a section before `\maketitle` digests the names; the notes are queued at `\maketitle` alone, the document's end
  // without one: revSNJNLb B2)
  Row {
    repro:      "snjnl_note_before_section_and_maketitle_prints_once",
    tex:        repro!("snjnl_note_before_section_and_maketitle_prints_once"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; correspondent[*]=Corresponding author(s). ; equal-contributors[†]=Outer. | email=ann@a.org ; affiliation=Univ A"#,
      r#"Bob Baker | affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain ; † plain"#,
      r#"Bob Baker"#,
      r#"margin: Corresponding author(s). ; Outer."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "snjnl_note_inside_a_name_before_section_and_maketitle_prints_once",
    tex:        repro!("snjnl_note_inside_a_name_before_section_and_maketitle_prints_once"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Univ A"#,
      r#"Bob Baker | affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "snjnl_note_between_section_and_maketitle_red",
    tex:        repro!("snjnl_note_between_section_and_maketitle_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; correspondent[*]=Corresponding author(s). | email=ann@a.org ; affiliation=Univ A"#,
      r#"Bob Baker ; equal-contributors[†]=Between. | affiliation=Univ A"#,
    ],
    moves:      r#"row R SNJNLAFTERSEC (a note after a section's flush marks no name: the document's, unmarked, as the base)"#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"margin: Corresponding author(s). ; Between."#,
    ],
    page_moves: r#"row R SNJNLAFTERSEC (Bob's † not shown)"#,
  },
  Row {
    repro:      "snjnl_typed_star_beside_the_class_star_red",
    tex:        repro!("snjnl_typed_star_beside_the_class_star_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; correspondent[*]=Corresponding author(s). ; footnotemark[*]= | email=ann@a.org ; affiliation=Univ A"#,
      r#"Bob Baker ; equal-contributors[†]=Equal. ; footnotemark[†]= | affiliation=Univ A"#,
    ],
    moves:      r#"row R SNJNLTYPEDFOLD (a typed mark is folded into the class's identical mark on the same name: Ann's typed ∗ lost)"#,
    page:       &[
      r#"Ann Able ; * plain ; * plain"#,
      r#"Bob Baker ; † plain ; †→note"#,
      r#"margin: Corresponding author(s). ; Equal."#,
    ],
    page_moves: r#"row R SNJNLTYPEDFOLD (Ann's typed ∗ lost); row R MARKORDER (Bob's typed † prints before the class's)"#,
  },
  // (a note inside a name runs last as the names print: its text the one printed, revSNJNL B2)
  Row {
    repro:      "snjnl_note_inside_a_name_text_wins",
    tex:        repro!("snjnl_note_inside_a_name_text_wins"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; equal-contributors[†]=Second text. | affiliation=Univ A"#,
      r#"Bob Baker | affiliation=Univ A"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; † plain"#,
      r#"Bob Baker"#,
      r#"margin: Second text."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "date_legend_marked_lines",
    tex:        repro!("date_legend_marked_lines"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Dept A, Univ X"#,
      r#"Bob Baker | affiliation=Dept B, Univ Y ; email=bob@y.org"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "date_legend_marked_lines_then_today",
    tex:        repro!("date_legend_marked_lines_then_today"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Dept A, Univ X"#,
      r#"Bob Baker | affiliation=Dept B, Univ Y ; email=bob@y.org"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "date_legend_textsuperscript_and_dag",
    tex:        repro!("date_legend_textsuperscript_and_dag"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Dept A, Univ X"#,
      r#"Bob Baker | affiliation=Dept B, Univ Y"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "date_legend_unlabelled_affiliation_every_author",
    tex:        repro!("date_legend_unlabelled_affiliation_every_author"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Department of Physics University of X, Town"#,
      r#"Bob Baker | affiliation=Department of Physics University of X, Town"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "date_legend_hand_typed_numbers",
    tex:        repro!("date_legend_hand_typed_numbers"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Dept A, Univ X"#,
      r#"Bob Baker | affiliation=Dept B, Univ Y"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "date_legend_address_line_by_spelling",
    tex:        repro!("date_legend_address_line_by_spelling"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Department of Physics, University of X ; email=aable@x.edu"#,
      r#"Bob Baker | affiliation=Department of Physics, University of X ; email=bbaker@y.org"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "date_legend_authblk_every_name_typed",
    tex:        repro!("date_legend_authblk_every_name_typed"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | affiliation=Univ A, Town"#,
      r#"Bob Baker | affiliation=Univ A, Town"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "date_legend_unrequested_label_stays",
    tex:        repro!("date_legend_unrequested_label_stays"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[1]="#,
      r#"Bob Baker ; footnotemark[2]="#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able ; 1 plain"#, r#"Bob Baker ; 2 plain"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "date_legend_real_date_unchanged",
    tex:        repro!("date_legend_real_date_unchanged"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[1]="#,
      r#"Bob Baker ; footnotemark[2]="#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able ; 1 plain"#, r#"Bob Baker ; 2 plain"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "date_legend_empty_unchanged",
    tex:        repro!("date_legend_empty_unchanged"),
    warnings:   0,
    creators:   &[r#"Ann Able"#, r#"Bob Baker"#],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "date_legend_amsart_unchanged",
    tex:        repro!("date_legend_amsart_unchanged"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[1]="#,
      r#"Bob Baker ; footnotemark[2]="#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able ; 1 plain"#, r#"Bob Baker ; 2 plain"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "date_legend_revtex_unchanged",
    tex:        repro!("date_legend_revtex_unchanged"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[1]="#,
      r#"Bob Baker ; footnotemark[2]="#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able ; 1 plain"#, r#"Bob Baker ; 2 plain"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "note_email_list_in_abstract",
    tex:        repro!("note_email_list_in_abstract"),
    warnings:   0,
    creators:   &[
      r#"Parshin Shojaee | email=p_shojaee@apple.com"#,
      r#"Iman Mirzadeh | email=imirzadeh@apple.com"#,
      r#"Keivan Alizadeh | email=kalizadehvahid@apple.com"#,
      r#"Maxwell Horton | email=mchorton@apple.com"#,
      r#"Samy Bengio | email=bengio@apple.com"#,
      r#"Mehrdad Farajtabar | email=farajtabar@apple.com"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Parshin Shojaee"#,
      r#"Iman Mirzadeh"#,
      r#"Keivan Alizadeh"#,
      r#"Maxwell Horton"#,
      r#"Samy Bengio"#,
      r#"Mehrdad Farajtabar"#,
    ],
    page_moves: r#""#,
  },
];
/// Class-printed marks against none: a note carries a mark only where its class prints one (IEEEtran and ieeeconf none,
/// 2606.00397, 1510.02728), a symbol opening its text that is the text's own stays text (review revSHMARK S2), and
/// unmarked notes alike in text but not in their links or math all print (review revREND2b, the reverted dedupe); a mark
/// unknown here (an unbound class's, a document's own `\maketitle`'s) is no glyph on the page, the text under its name
/// (revMARKSAFE B1, ADVGEN I-3), a document's wrapper of the class's `\maketitle` changes nothing (revMARKSAFEb B1), and
/// an author block's `\thefootnote` holds in its cell alone (revMARKSAFE S1); a `\maketitle` that makes a footnote or
/// sets the footnote counter before its title, or one a package's `\let` restores, has unknown marks (revPRESBXb BL2, BL3);
/// a mark the counter steps to 0 or below in the title block prints none (1910.09525, WRONGFIXc).
/// achemso's affiliations, e-mails, phones, faxes and alternative affiliations are under the authors the class numbers
/// them for, its generated marks no ink of ours (ACHEMSO, OXIDIZED_DESIGN_DIVERGENCES #1065; 2605.10968, 2306.05209).
/// An elsarticle key names one entry, every route to a note reaches it — a key and a `\corref`/`\fnref` both, in either
/// order; the outside `\corref`/`\fnref` read last the first author's, wherever it is read; one in its own group, or a
/// note whose key another entry takes, in either order, no name's (ELSART, OD #1067: 1501.03323, 1909.01114, 1711.08299);
/// a `\cormark` outside a name, or a key list, no name's (ELSARTb, ELSARTc); a mark read after the names print
/// (`\end{frontmatter}`, `\maketitle`) none, and a `\corref` key with a comma one key where an entry has it (ELSARTd).
#[test]
fn notes_are_marked_only_where_the_class_prints_marks() { assert_green(CLASS); }
/// achemso's pieces no author holds at `\begin{document}` (a CLASS row asserts only the creators): each is one
/// document-level note, as the class lists it with a mark no name carries, once however many authors replaced it
/// (revACHEMSO B1); one a later author repeats is no note, and one before any `\author` is the class's Warning and
/// kept; a replaced `\email`, `\phone` or `\fax`, and a phone or fax of an author with no e-mail, which the class does
/// not print, are kept as notes (revACHEMSO S2) (ACHEMSO, OXIDIZED_DESIGN_DIVERGENCES #1065; 2512.17659, 2510.00865,
/// 2107.00078, 1910.14165).
#[test]
fn achemso_pieces_no_author_holds_are_unlinked_notes() {
  for (tex, warnings, notes) in [
    (
      repro!("fmmarks_achemso_pieces_no_author_holds_stay_unlinked"),
      1,
      &["affiliation=Univ C", "altaffiliation=Lab X"][..],
    ),
    (
      repro!("fmmarks_achemso_affiliation_after_affiliation_reaches_no_author"),
      1,
      &["affiliation=Univ B"][..],
    ),
    (
      repro!("fmmarks_achemso_pieces_before_any_author_warn"),
      3,
      &["affiliation=Univ Z", "email=z@x.org"][..],
    ),
    (
      repro!("fmmarks_achemso_replaced_altaffiliation_is_one_note"),
      1,
      &["altaffiliation=Now at P"][..],
    ),
    (
      repro!("fmmarks_achemso_second_email_phone_fax_replace_the_first"),
      1,
      &["email=ann@x.edu", "phone=123", "phone=555"][..],
    ),
    // (a repeated identical contact replaces nothing: no duplicate note, 2203.15022; revACHEMSOb B1)
    (
      repro!("fmmarks_achemso_repeated_contact_is_one_contact"),
      1,
      &[][..],
    ),
  ] {
    let (log, xml, ()) = convert_with_setup_then(tex, Some("ar5iv.sty"), || {}, |_| {});
    // (the notes outside every creator, `role=text` in document order)
    let mut outside = xml.clone();
    while let (Some(from), Some(to)) = (outside.find("<creator "), outside.find("</creator>")) {
      outside.replace_range(from..to + "</creator>".len(), "");
    }
    let found: Vec<String> = outside
      .match_indices("<note ")
      .map(|(at, _)| {
        let tag_end = at + outside[at..].find('>').expect("start tag");
        let close = at + outside[at..].find("</note>").expect("closed note");
        format!(
          "{}={}",
          attribute(&outside[at..tag_end], "role"),
          text(&outside[tag_end + 1..close])
        )
      })
      .collect();
    assert_eq!(
      (error_count(&log), warning_count(&log), found),
      (
        0,
        warnings,
        notes.iter().map(ToString::to_string).collect::<Vec<_>>()
      ),
      "{xml}"
    );
  }
}

/// The known gaps of [`notes_are_marked_only_where_the_class_prints_marks`].
#[test]
#[ignore = "RED: the row R lines its rows name"]
fn notes_are_marked_only_where_the_class_prints_marks_red() { assert_red_layers_now_green(CLASS); }

/// Core documents the reviews of REND2 wrote by hand, through `LaTeXML-html5.xsl` alone: a letter, digit or Greek
/// mark-only note links to no other author's note of the same glyph and promotes none to the margin (revREND2 p1, the
/// Greek `α` added), and unmarked notes alike in text, in two blocks, beside an editor's note and a contact, each print
/// under its author with no glyph, every reference resolving (revREND2b p2; DG 2026-11-16, OD #990).
const POST_ONLY: [(&str, &str, &[&str]); 2] = [
  (
    "glyph collision",
    r#"<?xml version="1.0" encoding="UTF-8"?>
<?latexml RelaxNGSchema="LaTeXML"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="Document">
  <title>Glyph collision</title>
  <creator role="author">
    <personname>Ann Able</personname>
    <note mark="1" role="footnote" xml:id="id1" fragid="id1">Ann's own numbered footnote.</note>
  </creator>
  <creator role="author">
    <personname>Bob Baker</personname>
    <note mark="1" role="footnotemark" xml:id="id2" fragid="id2"/>
  </creator>
  <creator role="author">
    <personname>Cy Cole</personname>
    <note mark="a" role="thanks" xml:id="id3" fragid="id3">Cy's own lettered thanks.</note>
  </creator>
  <creator role="author">
    <personname>Dee Dunn</personname>
    <note mark="a" role="footnotemark"/>
  </creator>
  <creator role="author">
    <personname>Eve Ezra</personname>
    <note mark="α" role="thanks" xml:id="id5" fragid="id5">Eve's own Greek-lettered thanks.</note>
  </creator>
  <creator role="author">
    <personname>Fay Fox</personname>
    <note mark="α" role="footnotemark"/>
  </creator>
</document>
"#,
    // (a one-line block: each author's own marked note keeps its glyph and pop-up, LaTeXML.css's card aside — revMARKSAFEc
    // S3; each mark-only glyph prints again on its neighbour's note, so it shows, OD #1059)
    &[
      "Ann Able ; 1 plain",
      "Bob Baker ; 1 plain",
      "Cy Cole ; a plain",
      "Dee Dunn ; a plain",
      "Eve Ezra ; α plain",
      "Fay Fox ; α plain",
      "margin: Ann's own numbered footnote. ; Cy's own lettered thanks. ; Eve's own Greek-lettered thanks.",
    ],
  ),
  (
    "unmarked notes alike",
    r#"<?xml version="1.0" encoding="UTF-8"?>
<document xmlns="http://dlmf.nist.gov/LaTeXML" xml:id="Document">
  <title>Blocks</title>
  <creator role="author"><personname>Ann</personname><note role="thanks" xml:id="id1" fragid="id1">Same text.</note></creator>
  <creator role="editor"><personname>Ed</personname><note role="thanks" xml:id="id2" fragid="id2">Same text.</note></creator>
  <creator role="author"><personname>Bob</personname><contact role="thanks">Same text.</contact></creator>
  <creator role="author"><personname>Cy</personname><note role="thanks" xml:id="id3" fragid="id3"><emph>Same</emph> text.</note></creator>
  <para xml:id="p1"><p>See <ref idref="id3">note</ref>.</p></para>
  <section xml:id="S1"><title>S</title>
    <creator role="author"><personname>Dee</personname><note role="thanks" xml:id="id4" fragid="id4">Same text.</note></creator>
  </section>
</document>
"#,
    &[
      "Ann | under: Same text.",
      "Bob",
      "Cy | under: Same text.",
      "Dee | under: Same text.",
    ],
  ),
];

/// The hand-written core documents of [`POST_ONLY`] render as their oracle.
#[test]
fn hand_written_documents_render_every_note_once() {
  for (label, xml, oracle) in POST_ONLY {
    let (lines, problems) = page_lines(&page_of(xml));
    let oracle: Vec<String> = oracle.iter().map(ToString::to_string).collect();
    assert_eq!((lines, problems), (oracle, Vec::<String>::new()), "{label}");
  }
}
