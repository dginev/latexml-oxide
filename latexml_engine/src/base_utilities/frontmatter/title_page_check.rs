//! The title page's printed words against the output (FIDELITY_CHARTER §4, every residual loss counted by a Warning;
//! OXIDIZED_DESIGN_DIVERGENCES #953 "106s"): a check only — it writes no content and changes no XML.
//!
//! At each `\maketitle` the kernel's deposit (sect05.rs `\lx@deposit@maketitle`) runs the whole title page once more,
//! speculatively, as pdflatex runs it — the class's own refused `\maketitle` or its `\@maketitle`, or LaTeX's printing
//! `\@title`, `\@author` and `\@date` — with the document's values (`note_title_page_print`), and notes the text it
//! prints ([`note_title_page_printed`]; the latest before the authors are sealed). When the document closes
//! ([`check_title_page_words`]) every content word of that text ([`net_content_words`], the scorer's) is looked for in
//! the output, cheapest first while one is unheld: the frontmatter's text, the whole document's, the subtrees a
//! `--streaming` run spilled ([`note_spilled_words`]), its attribute values (a contact's `name`, a link's `href`, what
//! a constructor set from an argument). A word the output does not hold is lost; it is counted when the document's own
//! source writes it ([`source_words`]: the files read as content, `binding::content::CONTENT_FILES`), else it is the
//! class's text — a label, a placeholder at its default, a running head's — tallied apart in an Info. Every lost word
//! is named in one `Warning:frontmatter:unclaimed`, in printed order; a word the unclaimed-text net already counted
//! (insert.rs `keep_unclaimed_net_notes`, [`note_net_counted`]) is not counted again. Witnesses: 2007.07526 (amsart's
//! author lines), the cpc-hepnp `\email` (1504.05444), raa 1304.5354, memsait astro-ph0102098, JUSTC 2409.17603.

use super::*;

/// The text the title page printed ([`note_title_page_printed`]).
const PRINTED: &str = "lx_title_page_printed";
/// The content words of the subtrees a `--streaming` run spilled ([`note_spilled_words`]).
const SPILLED: &str = "lx_spilled_words";
/// The words the unclaimed-text net counted in a Warning of its own ([`note_net_counted`]).
const NET_COUNTED: &str = "lx_net_counted_words";

/// Note the text a title page printed (sect05.rs `note_title_page_print`, read by `ink_word_text`): the frontmatter's,
/// in place of an earlier `\maketitle`'s, which it supersedes (DIVERGENCES #406: a later author list replaces the one
/// flushed); none once the authors are sealed, a supplement's title page after the body began, whose entries the seal
/// drops (`seal_digested_frontmatter`).
pub fn note_title_page_printed(text: &str) {
  if frontmatter_entry_sealed("ltx:creator", "author") {
    return;
  }
  assign_value(PRINTED, Stored::String(pin(text)), Some(Scope::Global));
}

/// Note the words the unclaimed-text net named in its Warning (insert.rs `keep_unclaimed_net_notes`): counted once.
pub(super) fn note_net_counted(words: &[String]) {
  let mut all = lookup_string(NET_COUNTED);
  for word in words {
    all.push(' ');
    all.push_str(word);
  }
  assign_value(NET_COUNTED, Stored::String(pin(all)), Some(Scope::Global));
}

/// The document's frontmatter elements, whose text [`check_title_page_words`] reads first: the title page's words are
/// mostly theirs.
const FRONT_ELEMENTS: &[&str] = &[
  "title",
  "subtitle",
  "toctitle",
  "creator",
  "date",
  "abstract",
  "keywords",
  "classification",
  "note",
  "titlepage",
];

/// The mark [`output_text`] sets at an element's edges: a word of the output may run across it (`Bob
/// <text>Bak</text>and`, "Bakand") or end at it (`…Labs</creator><creator>Bob…`).
const EDGE: char = '\u{2}';

/// The text of `node`'s descendants, an [`EDGE`] at each element's edges, and apart (`attributes`, when given) their
/// attribute values ([`attribute_text`]): an attribute between two pieces of a word does not part them
/// (`<XMTok>p</XMTok><XMTok role=…>p</XMTok>`, "pp"). No comment's text.
fn output_text(node: &Node, out: &mut String, attributes: &mut Option<String>) {
  for child in node.get_child_nodes() {
    match child.get_type() {
      Some(NodeType::ElementNode) => {
        if let Some(attributes) = attributes.as_mut() {
          attribute_text(&child, attributes);
        }
        out.push(EDGE);
        output_text(&child, out, attributes);
        out.push(EDGE);
      },
      Some(NodeType::TextNode | NodeType::CDataSectionNode) => out.push_str(&child.get_content()),
      _ => {},
    }
  }
}

/// The attributes whose value is no word the output holds: a formula's source and its text rendering (`tex`, `text`:
/// long, and the formula's letters are its tokens'), an element's id.
const WORDLESS_ATTRIBUTES: &[&str] = &["tex", "text", "xml:id"];

/// The values of `node`'s attributes, but our internal ones (`_…`) and [`WORDLESS_ATTRIBUTES`], each set apart by
/// spaces: a contact's or a heading's `name` ("Email:"), a link's target, and what a constructor set from an argument
/// that never prints (`\@@cite{cite}`'s `class="ltx_citemacro_cite"`, a rule's `width="0pt"`).
fn attribute_text(node: &Node, out: &mut String) {
  for (name, value) in node.get_attributes() {
    if !name.starts_with('_') && !WORDLESS_ATTRIBUTES.contains(&name.as_str()) {
      out.push(' ');
      out.push_str(&value);
      out.push(' ');
    }
  }
}

/// The most pieces of a run [`output_words`] joins into a word: a word set in pieces by markup (a font, a
/// superscript, an accent) holds a few.
const JOINED_PIECES: usize = 6;

/// The words of the title page's print the check looks for in the output ([`output_words`]): its content words, and
/// those its first and last runs open and end with against no space (`opens`, `ends`), which an output word may
/// continue (`unclaimed_words`).
#[derive(Default)]
struct Wanted {
  words: HashSet<String>,
  opens: Vec<String>,
  ends:  Vec<String>,
}

/// The shortest word the title page's first or last run holds that an output word may continue ([`continues`]): a
/// shorter one is too common a piece of other words to read so (an address's `com`, `cn`).
const RUN_ON_LEAST: usize = 4;

/// Does the output word `word` continue the print's edge word `edge` — begin with it, the print's last (`ends`), or end
/// with it, its first — being longer, `edge` of [`RUN_ON_LEAST`] characters or more ("shownbody" continues "shown")?
fn continues(word: &str, edge: &str, ends: bool) -> bool {
  edge.chars().count() >= RUN_ON_LEAST
    && word.len() > edge.len()
    && if ends {
      word.starts_with(edge)
    } else {
      word.ends_with(edge)
    }
}

/// The content words of `node` and what it holds ([`output_text`], folded): of each run of letters and digits, the
/// words of every stretch of up to [`JOINED_PIECES`] consecutive pieces between element edges — so both "Bak" and
/// "Bakand" of `Bob <text>Bak</text>and`, and "Labs" and "Bob" across two creators; with `attributes`, their attribute
/// values' words too. With `wanted`, only its words are kept, and those it opens or ends with that an output word ends
/// or begins with.
fn output_words(node: &Node, wanted: Option<&Wanted>, attributes: bool) -> HashSet<String> {
  let mut text = String::new();
  let mut values = attributes.then(String::new);
  if let Some(values) = values.as_mut() {
    attribute_text(node, values);
  }
  output_text(node, &mut text, &mut values);
  if let Some(values) = values {
    text.push(' ');
    text.push_str(&values);
  }
  let folded = if text.is_ascii() {
    text.to_ascii_lowercase()
  } else {
    resolver::base_letters(&text).to_lowercase()
  };
  let mut words = HashSet::default();
  let mut keep = |word: &str| match wanted {
    None => {
      words.insert(word.to_string());
    },
    Some(wanted) => {
      if wanted.words.contains(word) {
        words.insert(word.to_string());
      }
      for open in wanted
        .opens
        .iter()
        .filter(|open| continues(word, open, false))
      {
        words.insert(open.clone());
      }
      for end in wanted.ends.iter().filter(|end| continues(word, end, true)) {
        words.insert(end.clone());
      }
    },
  };
  let mut joined = String::new();
  for run in folded.split(|c: char| !(c.is_alphanumeric() || c == EDGE)) {
    let pieces: Vec<&str> = run.split(EDGE).filter(|piece| !piece.is_empty()).collect();
    for from in 0..pieces.len() {
      joined.clear();
      for piece in pieces.iter().skip(from).take(JOINED_PIECES) {
        joined.push_str(piece);
        // (a stretch of ASCII letters is its own one word, or none: no allocation for it)
        if joined.bytes().all(|b| b.is_ascii_lowercase()) {
          if joined.len() >= 2 {
            keep(&joined);
          }
        } else {
          for word in net_content_words(&joined) {
            keep(&word);
          }
        }
      }
    }
  }
  words
}

/// Note the content words of a subtree a `--streaming` run spills (latexml_oxide's `Document::spill_observer`): the
/// output holds them, though the document's tree no longer does when it closes. What it spills before a title page
/// printed is noted too: the title page may print it (an abstract set before `\maketitle`).
pub fn note_spilled_words(node: &Node) {
  let mut all = lookup_string(SPILLED);
  for word in output_words(node, None, true) {
    all.push(' ');
    all.push_str(&word);
  }
  assign_value(SPILLED, Stored::String(pin(all)), Some(Scope::Global));
}

/// The runs of letters and digits of the folded print `folded` ([`resolver::base_letters`]; an [`INK_SEAM`] inside
/// a run), and the indices of the ones its own text opens and ends with, against no space — the title page's text ends
/// where the page foot's begins (`INK_INSERT`).
fn print_runs(folded: &str) -> (Vec<&str>, usize) {
  let in_run = |c: char| c.is_alphanumeric() || c == INK_SEAM;
  let runs: Vec<&str> = folded.split(|c: char| !in_run(c)).collect();
  let last = folded
    .split(INK_INSERT)
    .next()
    .map_or(0, |main| main.split(|c: char| !in_run(c)).count())
    .saturating_sub(1);
  (runs, last)
}

/// The words [`output_words`] looks for: every content word of the print's runs and of their pieces, and those of its
/// first and last runs.
fn wanted_words(folded: &str) -> Wanted {
  let (runs, last) = print_runs(folded);
  let mut wanted = Wanted::default();
  for (at, run) in runs.iter().enumerate() {
    let whole: String = run.chars().filter(|c| *c != INK_SEAM).collect();
    let words = net_content_words(&whole);
    if at == 0 {
      wanted.opens.extend(words.iter().cloned());
    }
    if at == last {
      wanted.ends.extend(words.iter().cloned());
    }
    wanted.words.extend(words);
    if run.contains(INK_SEAM) {
      wanted
        .words
        .extend(run.split(INK_SEAM).flat_map(net_content_words));
    }
  }
  wanted
}

/// The title page's printed words the output does not hold, distinct, in printed order: of each run of letters and
/// digits in `printed` (folded, [`resolver::base_letters`]), its content words ([`net_content_words`]) `claimed`
/// lacks. Where the run crosses the edge of a whatsit that prints (`INK_SEAM`), its pieces are the words when the
/// output holds one of them (`\centerline{Ann Able}\centerline{MIT}`: "ablemit" is "able" and "mit"), or when the
/// whole is no word the document's source writes (`source`, [`source_words`]: an address before a note's mark,
/// "com" and "footnote"); else the whole is (an accent's whatsit inside `Jos\'e`). `folded`: the print, folded
/// ([`resolver::base_letters`]); `claimed`: its words the output holds ([`output_words`]), among them a word its own
/// text opens or ends with against no space that an output word ends or begins with — it runs on in the document's
/// text around the title page (`\maketitle` swallows the line end after it: `SHOWN` and `Body.` are "SHOWNBody.").
fn unclaimed_words(
  folded: &str,
  claimed: &HashSet<String>,
  source: &mut dyn FnMut() -> HashSet<String>,
) -> Vec<String> {
  let (runs, _) = print_runs(folded);
  let mut lost: Vec<String> = Vec::new();
  let mut written: Option<HashSet<String>> = None;
  for run in runs {
    if run.is_empty() {
      continue;
    }
    let whole: String = run.chars().filter(|c| *c != INK_SEAM).collect();
    let mut missing: Vec<String> = net_content_words(&whole)
      .into_iter()
      .filter(|word| !claimed.contains(word))
      .collect();
    if missing.is_empty() {
      continue;
    }
    if run.contains(INK_SEAM) {
      let pieces: Vec<String> = run.split(INK_SEAM).flat_map(net_content_words).collect();
      let written = written.get_or_insert_with(&mut *source);
      if pieces.iter().any(|word| claimed.contains(word))
        || !missing.iter().all(|word| written.contains(word))
      {
        missing = pieces
          .into_iter()
          .filter(|word| !claimed.contains(word))
          .collect();
      }
    }
    for word in missing {
      if !lost.contains(&word) {
        lost.push(word);
      }
    }
  }
  lost
}

/// The text symbols a control word prints as letters (OT1's `\o`, `\ss`, `\ae`, …), for [`source_words`]: `Bj{\o}rn`
/// is "Bjorn".
const LETTER_SYMBOLS: &[(&str, &str)] = &[
  ("o", "o"),
  ("O", "O"),
  ("l", "l"),
  ("L", "L"),
  ("i", "i"),
  ("j", "j"),
  ("ss", "ss"),
  ("ae", "ae"),
  ("AE", "AE"),
  ("oe", "oe"),
  ("OE", "OE"),
  ("aa", "aa"),
  ("AA", "AA"),
  ("dj", "d"),
  ("DJ", "D"),
];

/// The accents written as a control word of one letter (`\c{c}`, `\v{s}`, `\H{o}`), which print no letter of their own.
const ACCENT_WORDS: &[&str] = &["c", "v", "u", "H", "d", "b", "t", "k", "r"];

/// The words the document's own source writes: the content words of `text` (a content file's) read as letters —
/// its comments, a `filecontents` environment's body (a file it writes, a class's or a package's text) and the names
/// of `\begin`/`\end`'s environments out, a control word a space but a text symbol its letters ([`LETTER_SYMBOLS`])
/// and an accent nothing (`M\"{u}ller` "Muller", `Fran\c{c}ois` "Francois"), the braces read both as nothing and as a
/// space (`\author{Ann}{MIT}`, `{\bf A}ble`).
fn source_words(text: &str, words: &mut HashSet<String>) {
  let mut plain = String::new();
  let mut rest = text;
  while let Some(at) = rest.find("\\begin{filecontents") {
    plain.push_str(&rest[..at]);
    rest = &rest[at..];
    match rest.find("\\end{filecontents") {
      Some(end) => {
        rest = &rest[end + 1..];
        rest = rest.find('}').map_or("", |close| &rest[close + 1..]);
      },
      None => rest = "",
    }
  }
  plain.push_str(rest);
  let (mut glued, mut spaced) = (String::new(), String::new());
  for line in plain.lines() {
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
      match c {
        '%' => break,
        '\\' => {
          let mut name = String::new();
          while let Some(&next) = chars.peek() {
            if next.is_ascii_alphabetic() || next == '@' {
              name.push(next);
              chars.next();
            } else {
              break;
            }
          }
          if name.is_empty() {
            // (a control symbol: an accent's prints no letter of its own, `\%` none, any other a space)
            match chars.next() {
              Some('\'' | '`' | '^' | '"' | '~' | '=' | '.' | '%') | None => {},
              Some(_) => {
                glued.push(' ');
                spaced.push(' ');
              },
            }
          } else if name == "begin" || name == "end" {
            // (the environment's name: `\begin{abstract}`)
            if chars.peek() == Some(&'{') {
              for next in chars.by_ref() {
                if next == '}' {
                  break;
                }
              }
            }
            glued.push(' ');
            spaced.push(' ');
          } else if let Some((_, letters)) = LETTER_SYMBOLS.iter().find(|(n, _)| *n == name) {
            glued.push_str(letters);
            spaced.push_str(letters);
          } else if !ACCENT_WORDS.contains(&name.as_str()) {
            glued.push(' ');
            spaced.push(' ');
          }
        },
        '{' | '}' => spaced.push(' '),
        c => {
          glued.push(c);
          spaced.push(c);
        },
      }
    }
    glued.push(' ');
    spaced.push(' ');
  }
  words.extend(net_content_words(&glued));
  words.extend(net_content_words(&spaced));
}

/// The words the document's own content files write ([`source_words`]; `binding::content::CONTENT_FILES`): a virtual
/// file's text (`filecontents`), else the file's on disk; a `literal:` source its own text.
fn document_source_words() -> HashSet<String> {
  let mut words = HashSet::default();
  let files = lookup_string(CONTENT_FILES);
  for file in files.split('\0').filter(|file| !file.is_empty()) {
    if let Some(text) = file.strip_prefix("literal:") {
      source_words(text, &mut words);
    } else if let Some(text) = vfs_read(file) {
      source_words(&text, &mut words);
    } else if let Ok(bytes) = std::fs::read(file) {
      source_words(&String::from_utf8_lossy(&bytes), &mut words);
    }
  }
  words
}

/// The check, when the document closes: the words its title pages printed ([`note_title_page_printed`]) that its
/// output does not hold (`unclaimed_words`), but those the unclaimed-text net counted. Those the document's source
/// writes (`document_source_words`) are lost, named in one `Warning:frontmatter:unclaimed` in printed order; the
/// others are the class's own text (a label, a placeholder at its class default, a running head's words), tallied in
/// an Info (`frontmatter:class_text`, the placeholder ruling pending). Writes nothing.
pub fn check_title_page_words(document: &Document) -> Result<()> {
  let printed = lookup_string(PRINTED);
  if printed.trim().is_empty() {
    return Ok(());
  }
  let Some(root) = document.get_document().get_root_element() else {
    return Ok(());
  };
  let folded = resolver::base_letters(&printed).to_lowercase();
  let wanted = wanted_words(&folded);
  // (in three steps, each only while a printed word is unheld — cheapest first: the frontmatter's text, the whole
  // document's text, its attribute values)
  let front: Vec<Node> = element_nodes(&root)
    .into_iter()
    .filter(|child| FRONT_ELEMENTS.contains(&child.get_name().as_str()))
    .collect();
  let mut claimed: HashSet<String> = HashSet::default();
  for node in &front {
    claimed.extend(output_words(node, Some(&wanted), false));
  }
  let mut source: Option<HashSet<String>> = None;
  let mut held_all = |claimed: &HashSet<String>| {
    unclaimed_words(&folded, claimed, &mut || {
      source.get_or_insert_with(document_source_words).clone()
    })
    .is_empty()
  };
  if held_all(&claimed) {
    return Ok(());
  }
  claimed.extend(output_words(&root, Some(&wanted), false));
  for word in lookup_string(SPILLED)
    .split(' ')
    .chain(lookup_string(NET_COUNTED).split(' '))
  {
    if wanted.words.contains(word) {
      claimed.insert(word.to_string());
    }
    for open in wanted
      .opens
      .iter()
      .filter(|open| continues(word, open, false))
    {
      claimed.insert(open.clone());
    }
    for end in wanted.ends.iter().filter(|end| continues(word, end, true)) {
      claimed.insert(end.clone());
    }
  }
  if held_all(&claimed) {
    return Ok(());
  }
  claimed.extend(output_words(&root, Some(&wanted), true));
  let unclaimed = unclaimed_words(&folded, &claimed, &mut || {
    source.get_or_insert_with(document_source_words).clone()
  });
  if unclaimed.is_empty() {
    return Ok(());
  }
  let source = source.unwrap_or_else(document_source_words);
  let (lost, class): (Vec<String>, Vec<String>) = unclaimed
    .into_iter()
    .partition(|word| source.contains(word));
  if !lost.is_empty() {
    Warn!(
      "frontmatter",
      "unclaimed",
      s!(
        "The title page prints {} word(s) of the document's that its output does not hold: {}",
        lost.len(),
        lost.join(" ")
      )
    );
  }
  if !class.is_empty() {
    Info!(
      "frontmatter",
      "class_text",
      s!(
        "The title page prints {} word(s) of the class's own text (a label, a placeholder at its default) that the output does not hold: {}",
        class.len(),
        class.join(" ")
      )
    );
  }
  Ok(())
}
