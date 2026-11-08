//! Author notes and thanks: affiliation-mark splits, leading declarations, name annotations.

use super::*;

/// The body a creator's note (`\lx@add@thanks`, `\lx@add@note`) digests: its leading marks as the author line reads
/// them when an author's mark requests them and the note is one legend — a `\thanks{\textsuperscript{$\dagger$} School
/// ...}` is labelled by its mark and goes to the starred authors (2609.00885, 2609.19600) — and the rest, or all of it
/// when no author requests the mark or more legends follow, with the plain superscripts (`\lx@frontmatter@plainsups`): its `10 m$^{2}$` is text, not a mark
/// (KNOWN_PERL_ERRORS #515), and an unrequested leading mark stays shown, its note with its author.
pub(in crate::base_utilities) fn note_body(content: Tokens) -> Tokens {
  let toks = content.unlist();
  let lead = toks.iter().take_while(|t| **t == T_SPACE!()).count();
  let end = lead + leading_marks_end(&toks[lead..]);
  let requested = lookup_string("lx_frontmatter_requested_marks");
  // One legend: no other mark after a space in its text (`$^{\dag}$ These authors ... $^{*}$ This author ...` is several
  // legends and a funding line in one `\thanks`, 2609.39576, which stays whole with its author), where `10 m$^{2}$`
  // is glued to its base.
  // A numeral's spaced suffix (`5 $^{th}$`, shown by `rewrite_ordinal_superscripts`) is no legend.
  let after_numeral = |i: usize| {
    i >= 2
      && toks[i - 2].get_catcode() == Catcode::OTHER
      && toks[i - 2].with_str(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()))
  };
  let another_legend = (end + 1..toks.len())
    .any(|i| toks[i - 1] == T_SPACE!() && !after_numeral(i) && leading_marks_end(&toks[i..]) > 0);
  let marks_requested = end > lead
    && !another_legend
    && mark_operands(&toks[lead..end]).iter().any(|operand| {
      clean_frontmatter_labels(&Tokens::new(operand.clone()).to_string(), "affiliation")
        .iter()
        .any(|label| requested.split('\u{1}').any(|r| r == label))
    });
  let split = if marks_requested { end } else { 0 };
  let mut body = toks[..split].to_vec();
  body.extend([T_CS!("\\bgroup"), T_CS!("\\lx@frontmatter@plainsups")]);
  body.extend_from_slice(&toks[split..]);
  body.push(T_CS!("\\egroup"));
  Tokens::new(body)
}

/// Split an affiliation blob at each embedded superscript mark (`$^{n}$` /
/// `\textsuperscript{n}`), keeping the mark at the head of the following
/// segment. The abuse idiom packs several affiliations onto ONE line delimited
/// only by their marks (no `\\`), so `affil_splits()` (which splits on `\\`)
/// cannot separate them. Each returned segment then flows through
/// `\lx@affiliation@withsup`, which turns its leading mark into the
/// `affiliation:N` label that the authors' own marks already request
/// (`relocate_annotations` links them).
fn split_before_affiliation_marks(tokens: Tokens) -> Vec<Tokens> {
  split_marked_pieces(tokens, true)
}

/// An institution without the SEPARATOR that clings to its end when the affiliations are comma-joined
/// (`\textsuperscript{1}Univ A, \textsuperscript{2}Univ B`) rather than space-joined: without this the affiliation
/// contact reads "Univ A," with a stray trailing comma (html_feedback#6588, arXiv:2606.01317). Only trailing
/// whitespace + a SINGLE trailing comma (or semicolon, `$^1$Univ A; $^2$Univ B`, 2609.22690) are removed, so a comma
/// INSIDE an institution name ("Dept X, Univ Y, City") is untouched. Applied where a marked piece is followed by
/// another: between two on a line ([`split_marked_pieces`]), and at a line's end when the next line opens a new one
/// (`…Univ C,\\ \textsuperscript{4}Univ D`); before a line continuing it the comma is the institution's own, as
/// pdflatex prints it ("Université Laval,\\ pavillon …", 1507.08653).
pub(in crate::base_utilities) fn trim_trailing_separator(mut toks: Vec<Token>) -> Vec<Token> {
  // (and the spacing that parts two entries on one line, `$^1$Univ A \quad $^2$Univ B`, `~~~`: [`is_spacing`]; and a
  // line break left at a piece's end, `{\bfseries $^3$Lab C\\ $^4$Lab D}`'s first)
  while toks
    .last()
    .is_some_and(|t| is_spacing(t) || *t == T_CS!("\\\\") || *t == T_CS!("\\newline"))
  {
    toks.pop();
  }
  // (and the conjunction that joins the last two entries of a run, `$^2$Univ B and $^3$Univ C`, `\&`: no word of the
  // entry before it — a word of its own, after a space or a comma)
  let n = toks.len();
  let and_word = n >= 4
    && toks[n - 3..]
      .iter()
      .map(ToString::to_string)
      .eq(["a", "n", "d"])
    && (is_spacing(&toks[n - 4]) || toks[n - 4] == T_OTHER!(","));
  let ampersand = n >= 2 && toks[n - 1] == T_CS!("\\&") && is_spacing(&toks[n - 2]);
  if and_word {
    toks.truncate(n - 3);
  } else if ampersand {
    toks.truncate(n - 1);
  }
  while toks.last().is_some_and(is_spacing) {
    toks.pop();
  }
  // (a separator glyph between two entries, `\textit{$^1$A} $|$ \textit{$^2$B}` — 2508.07190 —, `A \textbullet{} B`: no
  // glyph of either entry, [`separator_glyph_len`])
  let glyph = separator_glyph_len(&toks);
  if glyph > 0 {
    toks.truncate(toks.len() - glyph);
    while toks.last().is_some_and(is_spacing) {
      toks.pop();
    }
  }
  if toks.last() == Some(&T_OTHER!(",")) || toks.last() == Some(&T_OTHER!(";")) {
    toks.pop();
    while toks.last().is_some_and(is_spacing) {
      toks.pop();
    }
  }
  toks
}

/// The glyphs a run of entries is parted by between their marks, set in math (`|`, `\vert`, `\mid`, `\cdot`,
/// `\bullet`, `\diamond`) or as text (`|`, `\textbar`, `\textbullet`, `\textperiodcentered`).
const SEPARATOR_GLYPHS: &[&str] = &["|", "\\vert", "\\mid", "\\cdot", "\\bullet", "\\diamond"];

/// The length of the separator glyph closing `toks` after a space, when it does: a formula of one separator glyph alone
/// (`$|$`, `$\cdot$`: [`SEPARATOR_GLYPHS`]), or a text one (`|`, `\textbar{}`, `\textbullet`), else 0. After a space
/// only: a glyph glued to a word is the entry's (`A|B`).
fn separator_glyph_len(toks: &[Token]) -> usize {
  let n = toks.len();
  let spaced = |at: usize| at > 0 && is_spacing(&toks[at - 1]);
  let glyph = |t: &Token| SEPARATOR_GLYPHS.iter().any(|g| t.with_str(|s| s == *g));
  // (`$|$`, `$ \cdot $`)
  if n >= 3 && toks[n - 1].get_catcode() == Catcode::MATH {
    let open = toks[..n - 1]
      .iter()
      .rposition(|t| t.get_catcode() == Catcode::MATH);
    if let Some(open) = open {
      let inner: Vec<&Token> = toks[open + 1..n - 1]
        .iter()
        .filter(|t| **t != T_SPACE!())
        .collect();
      if inner.len() == 1 && glyph(inner[0]) && spaced(open) {
        return n - open;
      }
    }
    return 0;
  }
  // (`\textbar{}`, `\textbullet`)
  let mut end = n;
  if end >= 2
    && toks[end - 1].get_catcode() == Catcode::END
    && toks[end - 2].get_catcode() == Catcode::BEGIN
  {
    end -= 2;
  }
  if end >= 1
    && ["\\textbar", "\\textbullet", "\\textperiodcentered"]
      .iter()
      .any(|cs| toks[end - 1] == T_CS!(*cs))
    && spaced(end - 1)
  {
    return n - (end - 1);
  }
  if n >= 1 && toks[n - 1] == T_OTHER!("|") && spaced(n - 1) {
    return 1;
  }
  0
}

/// [`split_before_affiliation_marks`] of a line (`line_start`) or of what follows a wrapper in one (`{…} \vspace{1mm}
/// $^3$Univ C`), where a lead that prints nothing is a separator, not the start of the first institution.
fn split_marked_pieces(tokens: Tokens, line_start: bool) -> Vec<Tokens> {
  let toks = tokens.unlist();
  // (the marks the queued names request, when they carry any: a number or letter mark none of them requests, after a
  // word and a plain space, is the entry's own text — `$^1$Laboratory for $^3$He Studies`, the isotope — not the next
  // entry's label; after a separator or wider spacing it still parts the entries)
  let names_marked = queued_author_marks().iter().any(|marks| !marks.is_empty());
  // (each mark outside every group: where it starts, the label it sets, whether it is glued to the word before it —
  // a plain space between, no separator: no spacing command, line break, comma or semicolon — and where it ends)
  let marks = top_level_marks(&toks);
  let text_mark = |at: usize, last_head: Option<&str>| {
    if !names_marked {
      return false;
    }
    let Some(k) = marks.iter().position(|m| m.at == at) else {
      return false;
    };
    let mark = &marks[k];
    let requested = mark
      .operand
      .as_ref()
      .is_some_and(|operand| !queued_requesters(operand).is_empty());
    // (a mark no name requests that heads nothing — only spacing or a separator follows it before the next mark or the
    // end: `$^1$Univ A $^\dagger$ \quad $^2$Univ B` — is its entry's text, a symbol too; review rev99a S5)
    let next = marks.get(k + 1).map_or(toks.len(), |m| m.at);
    let heads_nothing = toks[mark.end.min(next)..next]
      .iter()
      .all(|t| is_spacing(t) || *t == T_OTHER!(",") || *t == T_OTHER!(";"));
    if !requested && heads_nothing {
      return true;
    }
    if !mark.glued {
      return false;
    }
    // (glued to a word: a number or letter mark no name requests is the entry's own text — the isotope; one list labels
    // each entry once, so a label that also heads an entry after a separator is text where it is glued, `$^1$Inst.\ for
    // $^{2}$H Research \quad $^2$Univ B`, and so is one that breaks the list's running sequence before the sequence
    // goes on, `$^1$Lab for $^3$He Studies \quad $^2$Univ B`; review rev99a S1)
    let symbol = mark
      .operand
      .as_ref()
      .is_none_or(|operand| is_symbol_mark(operand));
    if !symbol && !requested {
      return true;
    }
    let Some(label) = mark.label.as_deref() else {
      return false;
    };
    let heads_elsewhere = marks
      .iter()
      .any(|m| m.at != at && !m.glued && m.label.as_deref() == Some(label));
    // (in a list whose entries a separator parts — a later mark after one — not a run-in list of plain spaces, where a
    // list written out of order, `$^1$Univ A $^3$Univ C $^2$Univ B`, still parts at each mark)
    let separated = marks.iter().skip(1).any(|m| m.at != at && !m.glued);
    let breaks_sequence = separated
      && last_head.and_then(successor).is_some_and(|expected| {
        expected != label
          && marks[k + 1..]
            .iter()
            .any(|m| m.label.as_deref() == Some(expected.as_str()))
      });
    heads_elsewhere || breaks_sequence
  };
  // (the label of the last mark that opened a piece)
  let mut last_head: Option<String> = marks.first().and_then(|m| m.label.clone());
  let mut segments: Vec<Tokens> = Vec::new();
  let mut current: Vec<Token> = Vec::new();
  // A boundary only outside every group: a cut inside `{…}` would leave the pieces unbalanced (a mark
  // list inside a font group is split by `split_wrapped_affiliation_marks`, which repeats the group).
  let mut depth = 0usize;
  for (i, t) in toks.iter().enumerate() {
    let dollar_super = *t == T_MATH!() && toks.get(i + 1).is_some_and(|n| *n == T_SUPER!());
    // (and a mark heading its entry from a group of its own or from the entry's wrapper, `{$^2$} ETH Zurich`,
    // `\textit{$^2$CSIRO Data61}`: [`wrapped_mark_at`], read at the top level only)
    let is_mark_start =
      dollar_super || is_mark_command(t) || (depth == 0 && marks.iter().any(|m| m.at == i));
    // Only treat a mark as an affiliation boundary when it is preceded by
    // whitespace (institutions are space-separated). A mark glued to the
    // preceding text — e.g. a superscript INSIDE an institution name,
    // "Center for R$^2$ Studies" — is not a boundary, so the name is not
    // wrongly split (reviewer-flagged). The first mark (current empty) always
    // opens segment 0.
    // A lead that prints nothing before the first mark — declarations (`\color{blue} $^1$…`), spacing (`\small \vspace{1mm}
    // $^{1}$Univ A`, 2407.15815) — is no segment: it opens the first one.
    // (a bare space is no lead: `\textit{$^1$A, $^2$B} $^3$C` keeps its boundary before `$^3$`)
    let prints_nothing = || {
      line_start
        && segments.is_empty()
        && current.iter().any(|t| t.get_catcode() == Catcode::CS)
        && leading_unprinted_end(&current) == current.len()
    };
    // (a space, or wider spacing: `$^1$Univ A \quad $^2$Univ B` inside a wrapper, `~~~`; 2506.18881)
    // (or a separator with no space after it, `Lisbon,\IEEEauthorrefmark{2}Univ B` — 2411.03010's IEEE blockA run)
    if is_mark_start
      && depth == 0
      && current
        .last()
        .is_some_and(|t| is_spacing(t) || *t == T_OTHER!(",") || *t == T_OTHER!(";"))
      && !text_mark(i, last_head.as_deref())
    {
      if let Some(m) = marks.iter().find(|m| m.at == i) {
        last_head.clone_from(&m.label);
      }
      if prints_nothing() {
        // …nor is the space between it and the mark the institution's
        while current.last() == Some(&T_SPACE!()) {
          current.pop();
        }
      } else {
        segments.push(Tokens::new(trim_trailing_separator(std::mem::take(
          &mut current,
        ))));
      }
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    current.push(*t);
  }
  // (the last piece keeps a comma closing it until the next line shows whether it separates: [`trim_trailing_separator`])
  while current.last() == Some(&T_SPACE!()) {
    current.pop();
  }
  if !current.is_empty() {
    segments.push(Tokens::new(current));
  }
  segments
}

/// A mark [`split_marked_pieces`] reads outside every group: where it starts and ends, its operand and the label it
/// sets, and whether it is glued to the word before it (only a plain space between).
struct ListMark {
  at:      usize,
  end:     usize,
  operand: Option<Vec<Token>>,
  label:   Option<String>,
  glued:   bool,
}

/// A mark heading an entry from inside a group at `toks[i]`, outside every other group: a group the mark alone fills
/// (`{$^2$} ETH Zurich`, 2203.16586's cvpr line `\small {$^1$} Beijing Institute of Technology {$^2$} ETH Zurich …`), or
/// one entry's own wrapper — a text font command's argument or a group, past the declarations opening it — that the
/// mark opens (`\textit{$^2$CSIRO Data61}`, 2508.07190's `\textit{$^1$…} $|$ \textit{$^2$…}`; `{\it $^2$Univ B}`;
/// 2605.25195's `\textbf{$^1$Fudan University}  \textbf{$^2$Tencent Hunyuan}`). The mark's operand, and where what it
/// labels starts: past the group a mark alone fills, past the mark inside a wrapper. A wrapper is the entry's own only
/// where it holds the whole entry — nothing but a separator after it ([`trim_trailing_separator`]) up to the next mark or
/// line break: a font group whose text goes on after it is the entry's text, its mark an isotope's (`Univ A,
/// \textit{$^{2}$H} Lab, Rome $^{2}$Univ B`, `Lab for \textit{$^{3}$He} Studies`; review rev107e finding 3). A mark
/// command's own group (`\textsuperscript{2}`) is the mark itself, read as one; a note's (`\thanks{$^1$…}`, an argument
/// of a name annotation, [`is_name_annotation`]) none.
fn wrapped_mark_at(toks: &[Token], i: usize) -> Option<(Vec<Token>, usize)> {
  let open = match toks[i].get_catcode() {
    Catcode::BEGIN if i > 0 && is_name_annotation(&toks[i - 1]) => return None,
    Catcode::BEGIN => i,
    Catcode::CS
      if is_text_font_command(&toks[i])
        && toks
          .get(i + 1)
          .is_some_and(|t| t.get_catcode() == Catcode::BEGIN) =>
    {
      i + 1
    },
    _ => return None,
  };
  let close = skip_group(toks, open);
  let inner = toks.get(open + 1..close.checked_sub(1)?)?;
  let mut j = leading_unprinted_end(inner);
  while inner.get(j) == Some(&T_SPACE!()) {
    j += 1;
  }
  let dollar_super =
    inner.get(j)?.get_catcode() == Catcode::MATH && inner.get(j + 1) == Some(&T_SUPER!());
  let (operand, mut next) = if dollar_super {
    let (operand, next) = sup_operand_at(inner, j + 2)?;
    (inner.get(next)?.get_catcode() == Catcode::MATH).then_some((operand, next + 1))?
  } else if is_mark_command(&inner[j]) {
    sup_operand_at(inner, j + 1)?
  } else {
    return None;
  };
  while inner.get(next) == Some(&T_SPACE!()) {
    next += 1;
  }
  if next == inner.len() {
    return Some((operand, close));
  }
  // (what follows the wrapper up to the next mark or line break: a separator alone, else the wrapper is no entry's own)
  let mut k = close;
  while k < toks.len() && !opens_entry_or_line(toks, k) {
    k += 1;
  }
  // (or a lead that prints nothing before it, `\hfill`, `\vspace{1mm}`)
  let after = &toks[close.min(k)..k];
  let separator = |piece: &[Token]| trim_trailing_separator(piece.to_vec()).is_empty();
  (separator(after) || separator(&after[leading_unprinted_end(after)..]))
    .then_some((operand, open + 1 + next))
}

/// Whether `toks[k]` opens a mark ([`wrapped_mark_at`]'s scan past a wrapper): a superscript formula, a mark command, a
/// group or text font command whose content a mark leads — or a line break, a paragraph's end (2603.19086's last entry
/// `\textit{$^5$CSIRO Data61}` before a blank line).
fn opens_entry_or_line(toks: &[Token], k: usize) -> bool {
  let t = &toks[k];
  if *t == T_CS!("\\\\") || *t == T_CS!("\\newline") || *t == T_CS!("\\par") || is_mark_command(t) {
    return true;
  }
  if t.get_catcode() == Catcode::MATH {
    return toks.get(k + 1) == Some(&T_SUPER!());
  }
  let brace = match t.get_catcode() {
    Catcode::BEGIN => k,
    Catcode::CS if is_text_font_command(t) => k + 1,
    _ => return false,
  };
  if toks
    .get(brace)
    .is_none_or(|t| t.get_catcode() != Catcode::BEGIN)
  {
    return false;
  }
  let close = skip_group(toks, brace).min(toks.len());
  let inner = &toks[brace + 1..close.saturating_sub(1).max(brace + 1)];
  let mut j = leading_unprinted_end(inner);
  while inner.get(j) == Some(&T_SPACE!()) {
    j += 1;
  }
  inner.get(j).is_some_and(|t| {
    is_mark_command(t)
      || (t.get_catcode() == Catcode::MATH && inner.get(j + 1) == Some(&T_SUPER!()))
  })
}

/// The marks of `toks` outside every group, in order (`$^{…}$`, a mark command: [`is_mark_command`]; a mark heading an
/// entry from a group of its own or from the entry's wrapper, [`wrapped_mark_at`]).
fn top_level_marks(toks: &[Token]) -> Vec<ListMark> {
  let mut marks = Vec::new();
  let mut depth = 0usize;
  for (i, t) in toks.iter().enumerate() {
    let dollar_super = *t == T_MATH!() && toks.get(i + 1).is_some_and(|n| *n == T_SUPER!());
    let wrapped = (depth == 0 && !dollar_super && !is_mark_command(t))
      .then(|| wrapped_mark_at(toks, i))
      .flatten();
    if depth == 0 && (dollar_super || is_mark_command(t) || wrapped.is_some()) {
      let operand_at = if dollar_super { i + 2 } else { i + 1 };
      let read = match wrapped {
        Some((operand, end)) => Some((operand, end)),
        None => sup_operand_at(toks, operand_at),
      };
      let end = match &read {
        Some((_, next)) if dollar_super && toks.get(*next) == Some(&T_MATH!()) => next + 1,
        Some((_, next)) => *next,
        None => i + 1,
      };
      let operand = read.map(|(operand, _)| operand);
      let label = operand.as_deref().and_then(mark_label);
      let before = toks[..i].iter().rev().find(|t| **t != T_SPACE!());
      let glued = toks[..i].last() == Some(&T_SPACE!())
        && before.is_some_and(|t| {
          matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER)
            && *t != T_OTHER!(",")
            && *t != T_OTHER!(";")
        });
      marks.push(ListMark {
        at: i,
        end,
        operand,
        label,
        glued,
      });
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
  }
  marks
}

/// The label after `label` in a running sequence of marks: the next number (`1` → `2`), the next letter (`a` → `b`).
fn successor(label: &str) -> Option<String> {
  let bare = label.rsplit(':').next().unwrap_or(label);
  let next = if let Ok(n) = bare.parse::<u32>() {
    (n + 1).to_string()
  } else {
    let mut chars = bare.chars();
    match (chars.next(), chars.next()) {
      (Some(c), None) if c.is_ascii_alphabetic() && c != 'z' && c != 'Z' => {
        char::from(c as u8 + 1).to_string()
      },
      _ => return None,
    }
  };
  Some(match label.rsplit_once(':') {
    Some((prefix, _)) => s!("{prefix}:{next}"),
    None => next,
  })
}

/// [`split_before_affiliation_marks`] for a marker-led affiliation line that opens with a font command or a group
/// holding the marks (`\textit{$^1$Univ A, $^2$Univ B}`, `{\small $^1$A; $^2$B}`, `\small{$^1$A, $^2$B}. Email: …`):
/// the inner list is split, the wrapper repeated on each piece — a group with the declarations that open it
/// (`\small`) — and what follows the wrapper kept after the last piece; else the marks inside the group were no
/// boundary, or every piece after the first left the font (62zk review; 2609.00995; repro
/// sectioning-frontmatter/affiliation_marks_inside_a_font_group_keep_the_font). A `\thanks{…}` wrapper is not
/// repeated: its mark-led content is the affiliation list (the `\thanks` idiom of `\lx@add@thanks`; 2609.24896).
pub(in crate::base_utilities) fn split_wrapped_affiliation_marks(line: Tokens) -> Vec<Tokens> {
  if let Some((cmd, inner, trailing)) = leading_wrapper(&line) {
    // A bare group's opening declarations (`\small`, `\it`, `\color{blue}` with its argument, among the spacing that
    // prints nothing: `{\small\vspace{1mm}\noindent\it $^{1}$…}`, 2407.15815), repeated in each later piece.
    let opening = if cmd.is_none() {
      lead_declarations(inner.unlist_ref())
    } else {
      Vec::new()
    };
    let mut pieces = split_wrapped_affiliation_marks(inner);
    if pieces.len() > 1 {
      // (a comma closing the wrapped list separates it from what follows the wrapper: `\textit{$^1$A, $^2$B,} $^3$C`)
      if let Some(last) = pieces.last_mut() {
        *last = Tokens::new(trim_trailing_separator(std::mem::take(last).unlist()));
      }
      let mut out: Vec<Tokens> = pieces
        .drain(..)
        .enumerate()
        .map(|(i, piece)| {
          let mut wrapped = Vec::new();
          if cmd == Some(T_CS!("\\thanks")) {
            wrapped.extend(piece.unlist());
          } else {
            wrapped.extend(cmd);
            wrapped.push(T_BEGIN!());
            if i > 0 {
              wrapped.extend(opening.iter().copied());
            }
            wrapped.extend(piece.unlist());
            wrapped.push(T_END!());
          }
          Tokens::new(wrapped)
        })
        .collect();
      // What follows the wrapper is split at its own marks too (`\textit{$^1$A, $^2$B} $^3$C`): its first piece stays
      // with the last wrapped one, the rest are pieces of their own.
      let mut after = split_marked_pieces(Tokens::new(trailing), false).into_iter();
      if let (Some(first), Some(last_piece)) = (after.next(), out.last_mut()) {
        let mut joined = last_piece.clone().unlist();
        joined.extend(first.unlist());
        *last_piece = Tokens::new(joined);
      }
      out.extend(after);
      return out;
    }
  }
  split_before_affiliation_marks(line)
}

/// Whether an author block's `line` holds authors' notes only: one or more `\thanks{…}`, `\footnote{…}` or
/// `\IEEEcompsocitemizethanks{…}` (each with an optional `[…]`), and else only spacing (spaces, `~`, `\par`). Such a
/// line prints no text where it stands, only its notes' marks: the notes are the line's above, as written without the
/// `\\` before them (`author_block/preprocess.rs` `notes_line_joins_names`; the IEEE reader's grade-and-notes line,
/// `grade_only_line`).
pub fn notes_only_line(line: &[Token]) -> bool {
  line_notes(line).is_some_and(|notes| !notes.is_empty())
}

/// A line of notes and of parts printing nothing ([`prints_nothing`]: `\thanks{…}\vspace{2mm}`) as the document's notes,
/// as an author block digests them in a contact (`document_note_call`; a note no pubnote takes as written), the rest
/// left out: the IEEEtran binding's bare run of notes between the names and their shared block
/// (`\IEEEauthorblockN{…}\thanks{…}\IEEEauthorblockA{…}`, 1508.00691, ieeetran_cls.rs `wrap_bare_author_block_text`).
/// `None` for a line holding other ink.
pub fn document_notes(line: &[Token]) -> Option<Vec<Token>> {
  let mut out = Vec::with_capacity(line.len());
  let mut rest = Vec::new();
  let mut i = 0;
  while i < line.len() {
    match note_call_end(line, i) {
      Some(end) => {
        let note = &line[i..end];
        match document_note_call(note) {
          Some(call) => out.extend(call),
          None => out.extend_from_slice(note),
        }
        i = end;
      },
      None => {
        rest.push(line[i]);
        i += 1;
      },
    }
  }
  prints_nothing(&Tokens::new(rest)).then_some(out)
}

/// A note an author block takes out as the document's, as it digests in a contact: `\thanks{…}` the
/// `\lx@add@pubnote@thanks{…}` and `\footnote{…}` the `\lx@add@pubnote{…}` `digestFrontmatterItem` binds them to there
/// (Perl Base_Utility.pool.ltxml:331-332, queue_digest.rs) — `note` one note's span, its command through its closing
/// brace. `None` for a note of another command, or with a `[…]` before its text, which no pubnote takes: such a note
/// stays where it stands. The one conversion of a line of notes no name requests (`notes_the_names_request`) and of an
/// address line's notes (`address_line_notes`).
pub(in crate::base_utilities) fn document_note_call(note: &[Token]) -> Option<Vec<Token>> {
  let plain = !note
    .iter()
    .take_while(|t| t.get_catcode() != Catcode::BEGIN)
    .any(|t| *t == T_OTHER!("["));
  let call = match note.first()? {
    t if plain && *t == T_CS!("\\thanks") => T_CS!("\\lx@add@pubnote@thanks"),
    t if plain && *t == T_CS!("\\footnote") => T_CS!("\\lx@add@pubnote"),
    _ => return None,
  };
  let mut out = Vec::with_capacity(note.len());
  out.push(call);
  out.extend_from_slice(&note[1..]);
  Some(out)
}

/// The notes of a line of notes only ([`notes_only_line`]), each the span `(start, end)` of `line` from its command
/// through its closing brace; `None` when the line holds other ink.
pub(in crate::base_utilities) fn line_notes(line: &[Token]) -> Option<Vec<(usize, usize)>> {
  let spacing = |t: &Token| *t == T_SPACE!() || *t == T_ACTIVE!('~') || *t == T_CS!("\\par");
  let mut notes = Vec::new();
  let mut i = 0;
  while i < line.len() {
    if spacing(&line[i]) {
      i += 1;
    } else {
      let end = note_call_end(line, i)?;
      notes.push((i, end));
      i = end;
    }
  }
  Some(notes)
}

/// The end of the note opening at `line[i]`: a `\thanks`, `\footnote` or `\IEEEcompsocitemizethanks`, its `[…]` and
/// its braced text; `None` for another token or a note not so written.
pub(in crate::base_utilities) fn note_call_end(line: &[Token], i: usize) -> Option<usize> {
  let past_spaces = |mut at: usize| {
    while line.get(at) == Some(&T_SPACE!()) {
      at += 1;
    }
    at
  };
  let t = line.get(i)?;
  if *t != T_CS!("\\thanks")
    && *t != T_CS!("\\footnote")
    && *t != T_CS!("\\IEEEcompsocitemizethanks")
  {
    return None;
  }
  let mut at = past_spaces(i + 1);
  if line.get(at) == Some(&T_OTHER!("[")) {
    let close = line[at..].iter().position(|t| *t == T_OTHER!("]"))?;
    at = past_spaces(at + close + 1);
  }
  if line
    .get(at)
    .is_none_or(|t| t.get_catcode() != Catcode::BEGIN)
  {
    return None;
  }
  let end = skip_group(line, at);
  line
    .get(end.wrapping_sub(1))
    .is_some_and(|t| t.get_catcode() == Catcode::END)
    .then_some(end)
}
