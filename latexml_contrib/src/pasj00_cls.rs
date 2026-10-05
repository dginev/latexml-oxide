//! The PASJ classes (Publications of the Astronomical Society of Japan) — `pasj00`, `pasj01`, `pasj02`: shipped with
//! every paper, not in TeX Live. The shipped class is interpreted raw (in every profile), for its own macros, and what
//! LaTeXML builds semantically is put back over it: the kernel `\caption`, and the frontmatter its title page only
//! typesets.
use latexml_package::prelude::*;

/// Interpret the shipped class `name` raw, then restore the semantic definitions over it:
/// - the kernel `\caption` (`\lx@caption`): PASJ's own (pasj00.cls:814-835, pasj01.cls:898-919, pasj02.cls:887-908)
///   steps the counter and calls the class's `\@makecaption` directly, bypassing `\@caption`, the one path that builds
///   `<caption>`, so its captions were paragraphs without their number and their labels dangled (1310.7069,
///   1505.02769, 2504.06663);
/// - `\@maketitle` empty: everything its title page lays out (pasj01.cls:1912-1950) is in the frontmatter, and the
///   deposit of the rest left an empty "[ Received ; Accepted ]" frame in the body (1505.02769, 2005.13750).
///
/// A paper whose source lacks the class falls to OmniBus, as before the binding. Either way the frontmatter macros the
/// classes store for `\maketitle` to typeset (pasj01.cls:84-159) are the frontmatter's: they were lost under OmniBus
/// and raw alike.
pub(crate) fn pasj_class(name: &str) -> Result<()> {
  let shipped = find_file(
    name,
    Some(FindFileOptions {
      ext_type: Some(Cow::Borrowed("cls")),
      forbid_ltxml: true,
      ..Default::default()
    }),
  )
  .is_some();
  if shipped {
    InputDefinitions!(name, noltxml => true, extension => Some(Cow::Borrowed("cls")));
    Let!("\\caption", "\\lx@caption");
    Let!("\\@maketitle", "\\@empty");
  } else {
    LoadClass!("OmniBus");
  }
  pasj_frontmatter()
}

/// PASJ's frontmatter macros as the frontmatter. One `\author` lists every author, separated by commas and `\&`, its
/// rows broken by `\\` (pasj00.cls:73-104), and the template puts each author's marks after the comma that ends the
/// name (`Name,\altaffilmark{1}\orcid{…}`, pasj02's usage). The kernel would read each line after a `\\` as an
/// affiliation (most of 0707.3867's 26 authors) and hand a mark after a comma to the next author (2504.06663,
/// 0704.3654), so the binding splits the list itself ([`pasj_author_list`]). Documents write `\altaffiltext{n}{text}`
/// and `\affil{text}` under all three versions, whatever pasj00's argument shapes; pasj00's `\email` takes an optional
/// name; pasj02's `\altemailmark` only marks the author `\email` belongs to.
fn pasj_frontmatter() -> Result<()> {
  Let!("\\lx@pasj@author", "\\author");
  DefMacro!("\\author[]{}", sub[(label, author)] {
    let mut call = vec![T_CS!("\\lx@pasj@author")];
    if let Some(label) = label {
      call.push(T_OTHER!("["));
      call.extend(label.unlist());
      call.push(T_OTHER!("]"));
    }
    call.push(T_BEGIN!());
    call.extend(pasj_author_list(&author.unlist()));
    call.push(T_END!());
    Ok(Tokens::new(call))
  });
  DefMacro!("\\KeyWords{}", "\\lx@add@keywords{#1}");
  Let!("\\kword", "\\KeyWords");
  DefMacro!("\\affil{}", "\\lx@add@affiliation{#1}");
  DefMacro!(
    "\\altaffilmark Semiverbatim",
    "\\lx@request@frontmatter@annotation[altaffil]{#1}"
  );
  // An empty text (a template's placeholder) makes no contact, which would stand as an empty creator (0704.3654).
  DefMacro!("\\altaffiltext Semiverbatim {}", sub[(label, text)] {
    if text.unlist_ref().iter().all(|t| t.get_catcode() == Catcode::SPACE) {
      Ok(Tokens!())
    } else {
      Ok(Invocation!("\\lx@add@contact[role=altaffiliation,label={altaffil:#1}]{#2}", vec![Some(label), Some(text)]))
    }
  });
  DefMacro!("\\email[]{}", "\\lx@add@email{#2}");
  def_macro_noop("\\altemailmark")?;
  DefMacro!("\\orcid Semiverbatim", "\\lx@add@orcid{#1}");
  DefMacro!("\\Received{}", "\\lx@add@date[role=received]{#1}");
  DefMacro!("\\Accepted{}", "\\lx@add@date[role=accepted]{#1}");
  Ok(())
}

/// The author marks a PASJ name carries, with their arguments: (control sequence, takes an optional argument, braced
/// arguments) — `\altaffilmark{…}`, `\orcid{…}`, `\thanks[…]{…}` (pasj01.cls:2019), `\email[…]{…}`, `\altemailmark`.
const PASJ_AUTHOR_MARKS: [(&str, bool, usize); 5] = [
  ("\\altaffilmark", false, 1),
  ("\\orcid", false, 1),
  ("\\thanks", true, 1),
  ("\\email", true, 1),
  ("\\altemailmark", false, 0),
];

/// A PASJ author list as `\and`-separated names: it splits at depth-0 `,`, `\&`, `\and` and `\\` (with the `*` and a
/// closed `[<skip>]` that follow it), gives a piece's leading marks (and math superscripts) back to the name before it,
/// drops a piece's leading "and", and keeps no empty piece.
fn pasj_author_list(toks: &[Token]) -> Vec<Token> {
  let mut pieces: Vec<Vec<Token>> = vec![Vec::new()];
  let mut depth = 0usize;
  let mut i = 0;
  while i < toks.len() {
    let t = toks[i];
    i += 1;
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    let separator = depth == 0
      && (t == T_OTHER!(",") || t == T_CS!("\\&") || t == T_CS!("\\and") || t == T_CS!("\\\\"));
    if !separator {
      pieces.last_mut().unwrap().push(t);
      continue;
    }
    if t == T_CS!("\\\\") {
      let mut k = i;
      while k < toks.len() && toks[k].get_catcode() == Catcode::SPACE {
        k += 1;
      }
      if k < toks.len() && toks[k] == T_OTHER!("*") {
        k += 1;
      }
      if k < toks.len()
        && toks[k] == T_OTHER!("[")
        && let Some(close) = closing_bracket(&toks[k..])
      {
        k += close + 1;
      }
      i = k;
    }
    pieces.push(Vec::new());
  }
  let mut names: Vec<Vec<Token>> = Vec::new();
  for mut piece in pieces {
    let marks = leading_marks(&piece);
    if marks > 0
      && let Some(prev) = names.last_mut()
    {
      prev.extend(piece.drain(..marks));
    }
    let piece = strip_leading_and(piece);
    if piece.iter().any(|t| t.get_catcode() != Catcode::SPACE) {
      names.push(piece);
    }
  }
  let mut list = Vec::new();
  for (n, name) in names.into_iter().enumerate() {
    if n > 0 {
      list.push(T_CS!("\\and"));
    }
    list.extend(name);
  }
  list
}

/// The length of a piece's leading run of author marks (with their arguments), math superscripts (`$^{1}$`) and
/// spaces — 0 when it starts with none: the marks that follow the comma ending the previous name.
fn leading_marks(piece: &[Token]) -> usize {
  let mut end = 0;
  let mut i = 0;
  while i < piece.len() {
    let t = piece[i];
    match t.get_catcode() {
      Catcode::SPACE => i += 1,
      Catcode::MATH => {
        let Some(len) = piece[i + 1..]
          .iter()
          .position(|t| t.get_catcode() == Catcode::MATH)
        else {
          break;
        };
        if piece.get(i + 1).map(|t| t.get_catcode()) != Some(Catcode::SUPER) {
          break;
        }
        i += len + 2;
        end = i;
      },
      Catcode::CS => {
        let Some((_, optional, braced)) = PASJ_AUTHOR_MARKS
          .iter()
          .find(|(cs, ..)| t.with_str(|s| s == *cs))
        else {
          break;
        };
        i += 1;
        // Exactly its own arguments: a following brace group belongs to the next name (`{\'A}lvaro`).
        let skip_spaces = |mut k: usize| {
          while k < piece.len() && piece[k].get_catcode() == Catcode::SPACE {
            k += 1;
          }
          k
        };
        if *optional {
          let k = skip_spaces(i);
          if piece.get(k) == Some(&T_OTHER!("[")) {
            match closing_bracket(&piece[k..]) {
              Some(close) => i = k + close + 1,
              None => return end,
            }
          }
        }
        for _ in 0..*braced {
          let k = skip_spaces(i);
          match piece.get(k) {
            Some(b) if b.get_catcode() == Catcode::BEGIN => {
              let mut level = 0usize;
              i = k;
              while i < piece.len() {
                match piece[i].get_catcode() {
                  Catcode::BEGIN => level += 1,
                  Catcode::END => {
                    level -= 1;
                    if level == 0 {
                      i += 1;
                      break;
                    }
                  },
                  _ => {},
                }
                i += 1;
              }
            },
            // An undelimited argument is one token (`\altaffilmark1`).
            Some(_) => i = k + 1,
            None => return end,
          }
        }
        end = i;
      },
      _ => break,
    }
  }
  end
}

/// The offset of the `]` closing the `[` that `toks` starts with, at brace depth 0 (`\thanks[{a]b}]{…}`).
fn closing_bracket(toks: &[Token]) -> Option<usize> {
  let mut depth = 0usize;
  for (n, t) in toks.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && *t == T_OTHER!("]") => return Some(n),
      _ => {},
    }
  }
  None
}

/// A piece without a leading "and" — the last name of a comma list ("…, and D. Four").
fn strip_leading_and(piece: Vec<Token>) -> Vec<Token> {
  let start = piece
    .iter()
    .position(|t| t.get_catcode() != Catcode::SPACE)
    .unwrap_or(piece.len());
  let word: String = piece[start..]
    .iter()
    .take(3)
    .map(|t| t.to_string())
    .collect();
  let after = piece.get(start + 3);
  if word == "and" && after.is_some_and(|t| t.get_catcode() == Catcode::SPACE) {
    piece[start + 3..].to_vec()
  } else {
    piece
  }
}

LoadDefinitions!({
  pasj_class("pasj00")?;
});
