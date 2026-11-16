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
//! - a mark between two names is the nearer one's by the printed gaps, a tie the next name's; one followed by a
//!   separator, `\\` or the block's end is carried back; one alone on the line after `\\` is no name's (ruling 4);
//! - on the page a shared note prints once, in the margin, its mark on every carrying name; a marked note of one name
//!   prints under it with no mark; an unmarked note prints its text under it with no glyph, a mark-only note with no
//!   glyph nothing (DG 2026-11-16: no invented glyph, OD #990); a mark-only note's mark links only to the one
//!   note of the block it marks, and only when it is a symbol (a letter, digit or Greek letter stays plain); the marks
//!   on a name in the order the PDF prints them.
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
fn creator_lines(xml: &str) -> Vec<String> {
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
/// note prints under the name with no mark, an unmarked note its text under the name with no glyph, a note with a
/// pop-up outside the name's wrapper its glyph and its body in the margin), then the block's `margin: body ; body`
/// (sorted); and the page's broken references: an id twice, an `href="#x"` with no `x`.
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
  if blocks.is_empty() {
    return (vec!["(no author block)".to_string()], problems);
  }
  let mut lines = Vec::new();
  for authors in blocks {
    let mut margin = Vec::new();
    for creator in descendants(&page, authors) {
      if !page[creator].has_class("ltx_creator") {
        continue;
      }
      let inside = descendants(&page, creator);
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
        let content = descendants(&page, note)
          .into_iter()
          .find(|&e| page[e].has_class("ltx_note_content"));
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
        let body = text_of(&page, content, &|e| {
          e.has_class("ltx_note_mark")
            || e.has_class("ltx_note_type")
            || e.has_class("ltx_tag_note")
        });
        if wrapped && page[note].has_class("ltx_note_shared") {
          marks.push(format!("{glyph}→note"));
          margin.push(body);
        } else if wrapped {
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
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Cat Cole | under: Corresponding author: Cat Cole"#,
      r#"margin: Equal contribution"#,
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
      r#"Ann Able | under: Corresponding author."#,
      r#"Bob Baker ; †→note"#,
      r#"Cat Cole ; †→note"#,
      r#"margin: These authors contributed equally to this work."#,
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
      r#"Ann Able ; †→note"#,
      r#"Bob Baker ; †→note | under: Corresponding author"#,
      r#"Cat Cole"#,
      r#"margin: These authors contributed equally."#,
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
      r#"Ann Able ; *→note"#,
      r#"Bob Bell ; *→note"#,
      r#"Cat Cole | under: Corresponding author: Cat Cole cat@c.edu"#,
      r#"Dan Dorn"#,
      r#"margin: Equal contribution"#,
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
      r#"Ann Able"#,
      r#"Bob Baker | under: Corresponding author: Univ A, Town."#,
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
      r#"Ann Able ; footnotemark[§]= ; thanks[*]=These authors contributed equally to this work. | affiliation=Univ A, Town"#,
      r#"Bob Baker ; thanks[*]=These authors contributed equally to this work. | affiliation=Univ A, Town"#,
      r#"Cat Cole | affiliation=Univ A, Town ; affiliation=Present address: Lab B, City"#,
    ],
    moves:      r#"MARKSAFE (NEITHER in phase 1: the `§` label no `\affil` answers is dropped with no Warning; a mark-only note)"#,
    page:       &[
      r#"Ann Able ; *→note ; § plain"#,
      r#"Bob Baker ; *→note"#,
      r#"Cat Cole"#,
      r#"margin: These authors contributed equally to this work."#,
    ],
    page_moves: r#"the XML layer's: MARKSAFE (NEITHER in phase 1: the `§` label no `\affil` answers is dropped with no Warning; a mark-only note)"#,
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
    page:       &[r#"Ann Able | under: Equal contribution"#, r#"Bob Bell"#],
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
    page:       &[r#"Ann Able ; * plain"#, r#"Bob Bee ; 1 plain"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_pure_href_marks_red",
    tex:        repro!("fmmarks_pure_href_marks_red"),
    warnings:   0,
    creators:   &[r#"Ann Able ; footnotemark[*]="#, r#"Bob Baker | url=†"#],
    moves:      r#""#,
    page:       &[r#"Ann Able ; * plain"#, r#"Bob Baker"#],
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
    page:       &[r#"Ann Able ; a plain | under: a visitor at the Institute."#],
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
      r#"Ann Able ; * plain | under: Email: ann@x.org"#,
      r#"Bob Baker ; * plain | under: Email: bob@x.org"#,
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
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"Cy Cole"#,
    ],
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
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#],
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
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#],
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
    page:       &[r#"Ann Able ; * plain | under: Equal."#, r#"Bob Baker"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "pure_names/pure_name_hash_at_a_names_end",
    tex:        repro!("pure_names/pure_name_hash_at_a_names_end"),
    warnings:   0,
    creators:   &[r#"Team C#"#, r#"Bob Baker ; footnotemark[*]="#],
    moves:      r#""#,
    page:       &[r#"Team C#"#, r#"Bob Baker ; * plain"#],
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
    page:       &[r#"Ann Able ; 12 plain"#, r#"Bob Baker"#],
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
      r#"Ann Able"#,
      r#"Bob Baker ; §→note | under: Equal contribution."#,
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
    page:       &[r#"Ann Able | under: Note X."#, r#"Bob Baker ; * plain"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "author_footnote_is_the_marked_thanks_note",
    tex:        repro!("author_footnote_is_the_marked_thanks_note"),
    warnings:   0,
    creators:   &[r#"Ann Able ; thanks[*]=Univ X"#, r#"Bob Baker"#],
    moves:      r#""#,
    page:       &[r#"Ann Able | under: Univ X"#, r#"Bob Baker"#],
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
    page:       &[r#"Ann | under: Equal."#, r#"Bob | under: Intern."#],
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
    page:       &[
      r#"Ann Able ; † plain | under: Univ X."#,
      r#"Bob Baker ; ‡ plain"#,
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
      r#"Ann Able ; *→note ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Carl Cole | under: Univ Y."#,
      r#"margin: Univ X."#,
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
    page:       &[
      r#"Ann Able ; † plain | under: Univ X."#,
      r#"Bob Baker | under: Univ Y."#,
      r#"Carl Cole ; § plain"#,
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
    page:       &[r#"Ann Able ; † plain | under: Lead note."#, r#"Bob Baker"#],
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
      r#"Ann Able ; †→note"#,
      r#"Bob Baker ; †→note"#,
      r#"Cy Cole ; * plain"#,
      r#"margin: Equal contribution."#,
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
      r#"Ann Able ; ‡→note"#,
      r#"Bob Baker ; ‡→note"#,
      r#"Cy Cole ; * plain"#,
      r#"margin: Equal contribution."#,
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
      r#"Ann Able | under: First note."#,
      r#"Bob Baker | under: Second note."#,
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
      r#"Ann Able | under: Univ A. Email: ann@a.edu ; Institute C."#,
      r#"Bob Baker | under: Univ B. Email: bob@b.edu"#,
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
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note ; † plain"#,
      r#"Cat Cole"#,
      r#"margin: Ann Able and Bob Baker contributed equally."#,
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
    moves:      r#"row R COMMAMARK (SHGIVEN open: a given mark after `Name,$^{1}$`, 1912.01737, 1811.08622, 2101.04763)"#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Carl Cole ; *→note"#,
      r#"Dan Dory"#,
      r#"margin: Univ X."#,
    ],
    page_moves: r#"the XML layer's: row R COMMAMARK (SHGIVEN open: a given mark after `Name,$^{1}$`, 1912.01737, 1811.08622, 2101.04763)"#,
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
    moves:      r#"row R COMMAMARK (SHGIVEN open: a given mark after `Name,$^{1}$`, 1912.01737, 1811.08622, 2101.04763)"#,
    page:       &[
      r#"Ann Able ; *→note"#,
      r#"Bob Baker ; *→note"#,
      r#"Carl Cole ; † plain"#,
      r#"Dan Dory ; ‡ plain"#,
      r#"margin: Univ X."#,
    ],
    page_moves: r#"the XML layer's: row R COMMAMARK (SHGIVEN open: a given mark after `Name,$^{1}$`, 1912.01737, 1811.08622, 2101.04763)"#,
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
];
/// Links: a `\footnotemark[n]`, `\samethanks[n]` or `\authornotemark[n]` on a name is a copy of the note it marks, its
/// mark as the class numbers it after the title's notes; a reference into a copied note still resolves (2502.19103,
/// 1911.13299, 1706.03762, 2607.15705, 2203.12881, 1803.03669; revSHMARK p1, p2, p9; revREND2b p2).
#[test]
fn a_mark_on_a_name_is_a_copy_of_its_note() { assert_green(LINK); }
/// The known gaps of [`a_mark_on_a_name_is_a_copy_of_its_note`].
#[test]
#[ignore = "RED: the row R lines its rows name"]
fn a_mark_on_a_name_is_a_copy_of_its_note_red() { assert_red_layers_now_green(LINK); }

/// The rows of [`a_mark_past_spacing_is_the_nearer_names`] and its `_red` twin.
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
    page:       &[
      r#"Ann Able ; * plain ; † plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  // the mark as printed, no Warning (the mark ledger writes the prefix Bob's: SHGNEXT, OD #1057)
  Row {
    repro:      "ne2e_prefix_mark_past_spacing_before_and",
    tex:        repro!("ne2e_prefix_mark_past_spacing_before_and"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker ; † plain"#, r#"Carl Cole"#],
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
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"Carl Cole ; * plain"#,
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
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker | under: X"#,
      r#"Carl Cole ; ‡ plain"#,
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
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_mark_inside_the_names_font_group",
    tex:        repro!("fmmarks_mark_inside_the_names_font_group"),
    warnings:   0,
    creators:   &[r#"Ann Able ; footnotemark[*]="#, r#"Bob Baker"#],
    moves:      r#""#,
    page:       &[r#"Ann Able ; * plain"#, r#"Bob Baker"#],
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
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#],
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
    page:       &[r#"Ann Able ; * plain ; † plain"#, r#"Bob Baker"#],
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
    page:       &[r#"Ann Able | under: Univ X."#, r#"Bob Baker ; ‡ plain"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2e_prefix_mark_past_hfill",
    tex:        repro!("ne2e_prefix_mark_past_hfill"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=X"#,
      r#"Bob Baker ; footnotemark[†]="#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able | under: X"#, r#"Bob Baker ; † plain"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2f_bare_mark_nearer_next_past_quads",
    tex:        repro!("ne2f_bare_mark_nearer_next_past_quads"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=X"#,
      r#"Bob Baker ; footnotemark[†]= ; footnotemark[‡]="#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able | under: X"#, r#"Bob Baker ; † plain ; ‡ plain"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2f_mark_and_note_nearer_next",
    tex:        repro!("ne2f_mark_and_note_nearer_next"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=X"#,
      r#"Bob Baker ; footnotemark[†]= ; thanks[‡]=Y"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: X"#,
      r#"Bob Baker ; † plain | under: Y"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2f_mark_nearer_next_past_qquad",
    tex:        repro!("ne2f_mark_nearer_next_past_qquad"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker ; † plain"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2f_mark_nearer_next_past_quads",
    tex:        repro!("ne2f_mark_nearer_next_past_quads"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker ; † plain"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2f_mark_nearer_next_past_ties",
    tex:        repro!("ne2f_mark_nearer_next_past_ties"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker ; † plain"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2g_mark_in_group_with_previous_name",
    tex:        repro!("ne2g_mark_in_group_with_previous_name"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker ; † plain"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2g_mark_in_its_own_group",
    tex:        repro!("ne2g_mark_in_its_own_group"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker ; † plain"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2f_marks_wider_after_and_tied",
    tex:        repro!("ne2f_marks_wider_after_and_tied"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Bob Baker"#,
      r#"Carl Cole ; footnotemark[‡]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain ; † plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole ; ‡ plain"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2g_mark_past_hfill_undecided",
    tex:        repro!("ne2g_mark_past_hfill_undecided"),
    warnings:   0,
    creators:   &[r#"Ann Able ; footnotemark[†]="#, r#"Bob Baker"#],
    moves:      r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block) (the centred line leaves the fill 0pt: † 3.9pt after Ann, 11.8pt before Bob)"#,
    page:       &[r#"Ann Able ; † plain"#, r#"Bob Baker"#],
    page_moves: r#"the XML layer's: row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block) (the centred line leaves the fill 0pt: † 3.9pt after Ann, 11.8pt before Bob)"#,
  },
  Row {
    repro:      "ne2g_mark_then_orcid_nearer_next",
    tex:        repro!("ne2g_mark_then_orcid_nearer_next"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | orcid="#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#,
    page:       &[r#"Ann Able"#, r#"Bob Baker ; † plain"#, r#"Carl Cole"#],
    page_moves: r#"the XML layer's: row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#,
  },
  Row {
    repro:      "ne2g_mark_then_superscript_nearer_next",
    tex:        repro!("ne2g_mark_then_superscript_nearer_next"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[a]= ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block); PURE (fix/pure-personname: the marks out of the personname)"#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; † plain ; a plain"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#"the XML layer's: row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block); PURE (fix/pure-personname: the marks out of the personname)"#,
  },
  Row {
    repro:      "ne2h_marks_run_parted_at_its_widest_gap",
    tex:        repro!("ne2h_marks_run_parted_at_its_widest_gap"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2h_marks_run_wider_before_goes_to_the_next_name",
    tex:        repro!("ne2h_marks_run_wider_before_goes_to_the_next_name"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; * plain ; † plain"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_bare_prefix_mark_after_quad_red",
    tex:        repro!("fmmarks_bare_prefix_mark_after_quad_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[*]="#,
      r#"Carl Cole ; footnotemark[†]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; * plain"#,
      r#"Carl Cole ; † plain"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_gap_mark_then_thanks_red",
    tex:        repro!("fmmarks_gap_mark_then_thanks_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]= ; thanks[*]=Ann note"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; † plain | under: Ann note"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_gap_thanks_then_mark_red",
    tex:        repro!("fmmarks_gap_thanks_then_mark_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]= ; thanks[*]=Ann note"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; † plain | under: Ann note"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_mark_after_wide_hspace_nearer_next_red",
    tex:        repro!("fmmarks_mark_after_wide_hspace_nearer_next_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[*]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker ; * plain"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_mark_and_note_floating_between_names_red",
    tex:        repro!("fmmarks_mark_and_note_floating_between_names_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[*]=Ann note body"#,
      r#"Bob Baker ; footnotemark[†]= ; thanks[‡]=Floating body"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: Ann note body"#,
      r#"Bob Baker ; † plain | under: Floating body"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_mark_between_interword_spaces_red",
    tex:        repro!("fmmarks_mark_between_interword_spaces_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker ; † plain"#, r#"Carl Cole"#],
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
    page:       &[r#"Ann Able ; * plain"#, r#"Bob Baker"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_mark_nearer_next_bold_name_red",
    tex:        repro!("fmmarks_mark_nearer_next_bold_name_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker ; † plain"#, r#"Carl Cole"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_mark_then_cite_nearer_next_red",
    tex:        repro!("fmmarks_mark_then_cite_nearer_next_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#,
    page:       &[r#"Ann Able"#, r#"Bob Baker ; † plain"#, r#"Carl Cole"#],
    page_moves: r#"the XML layer's: row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#,
  },
  Row {
    repro:      "fmmarks_mark_then_footnotetext_nearer_next_red",
    tex:        repro!("fmmarks_mark_then_footnotetext_nearer_next_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; thanks[†]=Bob note"#,
      r#"Carl Cole"#,
    ],
    moves:      r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker | under: Bob note"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#"the XML layer's: row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#,
  },
  Row {
    repro:      "fmmarks_mark_then_hfill_before_next_red",
    tex:        repro!("fmmarks_mark_then_hfill_before_next_red"),
    warnings:   0,
    creators:   &[r#"Ann Able"#, r#"Bob Baker ; footnotemark[†]="#],
    moves:      r#"row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#,
    page:       &[r#"Ann Able"#, r#"Bob Baker ; † plain"#],
    page_moves: r#"the XML layer's: row R SHGIVENNEXT (a mark nearer the next name, or a bare one; today the counted loss or the whole block)"#,
  },
  Row {
    repro:      "fmmarks_mark_then_superscript_label_of_next_red",
    tex:        repro!("fmmarks_mark_then_superscript_label_of_next_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[b]= ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; † plain ; b plain"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#"row R MARKORDER (the notes' document order is not the order the PDF prints their marks; DG 2026-11-16: minor, each mark is on its carrier)"#,
  },
  Row {
    repro:      "fmmarks_mark_then_thanks_before_next_name_red",
    tex:        repro!("fmmarks_mark_then_thanks_before_next_name_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[†]= ; thanks[*]=Bob note"#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; † plain | under: Bob note"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_marks_run_split_by_wide_space_red",
    tex:        repro!("fmmarks_marks_run_split_by_wide_space_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; † plain"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_marks_three_names_one_line_red",
    tex:        repro!("fmmarks_marks_three_names_one_line_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; footnotemark[*]="#,
      r#"Bob Baker"#,
      r#"Carl Cole ; footnotemark[†]="#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole ; † plain"#,
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
    page:       &[r#"Ann Able"#, r#"Bob Baker ; † plain"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_single_name_marks_after_newline_thin_spaces_red",
    tex:        repro!("fmmarks_single_name_marks_after_newline_thin_spaces_red"),
    warnings:   0,
    creators:   &[r#"Ann Able ; footnotemark[1]= ; footnotemark[2]="#],
    moves:      r#""#,
    page:       &[r#"Ann Able ; 1 plain ; 2 plain"#],
    page_moves: r#""#,
  },
  // the marks a writer writes each from its own first token, never the one before it again (review revSHGNEXT B1, P1:
  // a mark's event span starts where the box before it began)
  Row {
    repro:      "ne2j_two_marks_glued_nearer_next",
    tex:        repro!("ne2j_two_marks_glued_nearer_next"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; * plain ; † plain"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2j_three_marks_glued_nearer_next",
    tex:        repro!("ne2j_three_marks_glued_nearer_next"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[*]= ; footnotemark[†]= ; footnotemark[‡]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; * plain ; † plain ; ‡ plain"#,
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
    page:       &[r#"Ann Able ; * plain ; † plain"#, r#"Bob Baker"#],
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
    page:       &[r#"Ann Able ; * plain ; † plain"#, r#"Bob Baker"#],
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
    page:       &[
      r#"Ann Able ; * plain ; † plain"#,
      r#"Bob Baker"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "ne2j_macro_marks_nearer_next",
    tex:        repro!("ne2j_macro_marks_nearer_next"),
    warnings:   0,
    creators:   &[
      r#"Ann Able"#,
      r#"Bob Baker ; footnotemark[*]= ; footnotemark[†]="#,
      r#"Carl Cole"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker ; * plain ; † plain"#,
      r#"Carl Cole"#,
    ],
    page_moves: r#""#,
  },
  // a formula's comma glued between two marks is ink of their run (revSHGNEXT S1; 1703.08733's shape)
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
];
/// Marks after spacing (ruling 4 and the nearer-name rule): a mark spacing parts from the names is the nearer name's by
/// the printed gaps (a tie the next's); followed by a separator, `\\` or the block's end it is carried back (1911.13299,
/// 2412.20504, 1807.02791; every revSHGIVEN probe).
#[test]
fn a_mark_past_spacing_is_the_nearer_names() { assert_green(SPACING); }
/// The known gaps of [`a_mark_past_spacing_is_the_nearer_names`].
#[test]
#[ignore = "RED: the row R lines its rows name"]
fn a_mark_past_spacing_is_the_nearer_names_red() { assert_red_layers_now_green(SPACING); }

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
      r#"Ann Able | under: E-mail: ann@a.org"#,
      r#"Bob Baker | under: E-mail: bob@c.org"#,
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
    moves:      r#"row R REVLETTERS (REVTeX 4.1/4.2 print a name's mark in their minipage letters, "a", unknown here: MARKSAFE writes none)"#,
    page:       &[
      r#"Ann Able ; a plain"#,
      r#"Bob Baker ; a plain"#,
      r#"Cat Cole"#,
    ],
    page_moves: r#"the XML layer's: row R REVLETTERS"#,
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
    moves:      r#"OD #1054 fail-safe (a name `\footnotemark` leaves the counted marks unknown); MARKSAFE (Ann's mark: REVTeX prints "a", we write "*")"#,
    page:       &[
      r#"Ann Able ; a plain"#,
      r#"Bob Baker ; †→note"#,
      r#"Cy Cole ; †→note"#,
      r#"margin: Equal contribution."#,
    ],
    page_moves: r#"the XML layer's: OD #1054 fail-safe (a name `\footnotemark` leaves the counted marks unknown); MARKSAFE (Ann's mark: REVTeX prints "a", we write "*")"#,
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
    creators:   &[
      r#"Ann Able ; thanks[]=3 He detectors were funded by X."#,
      r#"Bob Baker ; thanks[]=* marks the starred items below."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: 3 He detectors were funded by X."#,
      r#"Bob Baker | under: * marks the starred items below."#,
    ],
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
      r#"Ann Able | under: ∗ -algebras are her field."#,
      r#"Bob Baker | under: 3 He detectors were funded by X."#,
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
    page:       &[r#"Ann Able"#, r#"Bob Baker | under: Corresponding author."#],
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
      r#"Ann Able | under: 3 He detectors were funded by X."#,
      r#"Bob Baker | under: * marks the starred items."#,
      r#"Cy Cole | under: ∗ -algebras are studied."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_ieee_journal_thanks_unmarked",
    tex:        repro!("fmmarks_ieee_journal_thanks_unmarked"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | membership=Student Member, IEEE"#,
      r#"Bob Baker ; thanks[]=A. Able and B. Baker are with Univ A (e-mail: ann@a.edu). ; thanks[]=Part of this research was presented at Conf X. This research is supported by Y. | membership=Senior Member, IEEE"#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able"#,
      r#"Bob Baker | under: Part of this research was presented at Conf X. This research is supported by Y. ; A. Able and B. Baker are with Univ A (e-mail: ann@a.edu)."#,
    ],
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
      r#"Ann Able | under: First note."#,
      r#"Bob Baker | under: Second note."#,
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
    creators:   &[
      r#"Ann Able ; thanks[]=ORCID: profile"#,
      r#"Bob Baker ; thanks[]=ORCID: profile"#,
      r#"Cy Cole ; thanks[]=Supported by grant x 2 ."#,
      r#"Dee Dunn ; thanks[]=Supported by grant x 2 ."#,
    ],
    moves:      r#""#,
    page:       &[
      r#"Ann Able | under: ORCID: profile"#,
      r#"Bob Baker | under: ORCID: profile"#,
      r#"Cy Cole | under: Supported by grant x 2 ."#,
      r#"Dee Dunn | under: Supported by grant x 2 ."#,
    ],
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
    moves:      r#"row R SHAUTO (a document's own `\maketitle`: its marks unknown here, written none)"#,
    page:       &[
      r#"Ann Able | under: First note."#,
      r#"Bob Baker | under: Second note."#,
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
    moves:      r#"row R SHAUTO/T1 (a `\NewCommandCopy` wrapper of the class's `\maketitle` is not seen: written none)"#,
    page:       &[
      r#"Ann Able | under: First note."#,
      r#"Bob Baker | under: Second note."#,
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
    page:       &[
      r#"Ann Able ; * plain"#,
      r#"Bob Baker ; 2 plain"#,
      r#"Cy Cole ; ‡ plain"#,
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
      r#"Ann Able | under: Note A."#,
      r#"Bob Baker | under: Note B."#,
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
      r#"Ann Able | under: Note A."#,
      r#"Bob Baker | under: Note B."#,
      r#"Cat Cole | under: Note C."#,
    ],
    page_moves: r#""#,
  },
  Row {
    repro:      "marksafe_maketitle_wrapper_ieeetran_is_neutral",
    tex:        repro!("marksafe_maketitle_wrapper_ieeetran_is_neutral"),
    warnings:   0,
    creators:   &[
      r#"Ann Able | note=Ann note."#,
      r#"Bob Baker ; footnotemark[1]="#,
    ],
    moves:      r#""#,
    page:       &[r#"Ann Able"#, r#"Bob Baker ; 1 plain"#],
    page_moves: r#""#,
  },
  Row {
    repro:      "fmmarks_own_maketitle_relax_red",
    tex:        repro!("fmmarks_own_maketitle_relax_red"),
    warnings:   0,
    creators:   &[
      r#"Ann Able ; thanks[]=First note."#,
      r#"Bob Baker ; thanks[]=Second note."#,
    ],
    moves:      r#"row R SHAUTO/T1 (a document's own `\maketitle` naming `\maketitle` counts as the class's: ∗ † written)"#,
    page:       &[
      r#"Ann Able | under: First note."#,
      r#"Bob Baker | under: Second note."#,
    ],
    page_moves: r#""#,
  },
];
/// Class-printed marks against none: a note carries a mark only where its class prints one (IEEEtran and ieeeconf none,
/// 2606.00397, 1510.02728), a symbol opening its text that is the text's own stays text (review revSHMARK S2), and
/// unmarked notes alike in text but not in their links or math all print (review revREND2b, the reverted dedupe); a mark
/// unknown here (an unbound class's, a document's own `\maketitle`'s) is no glyph on the page, the text under its name
/// (revMARKSAFE B1, ADVGEN I-3), a document's wrapper of the class's `\maketitle` changes nothing (revMARKSAFEb B1), and
/// an author block's `\thefootnote` holds in its cell alone (revMARKSAFE S1).
#[test]
fn notes_are_marked_only_where_the_class_prints_marks() { assert_green(CLASS); }
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
    &[
      "Ann Able | under: Ann's own numbered footnote.",
      "Bob Baker ; 1 plain",
      "Cy Cole | under: Cy's own lettered thanks.",
      "Dee Dunn ; a plain",
      "Eve Ezra | under: Eve's own Greek-lettered thanks.",
      "Fay Fox ; α plain",
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
