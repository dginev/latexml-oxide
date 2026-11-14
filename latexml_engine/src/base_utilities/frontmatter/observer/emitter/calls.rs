//! The emitter's vocabulary, moved unchanged from `author_block/` (the plan's row "Knowledge in,
//! driver out, one pass", user ruling 2026-11-08): Perl's API calls and the tokens of a mark or a note as the old reader wrote them — a queued name's
//! call, an entry's mark and label, a note's body and its `\lx@add@note` call, an e-mail command's address, the
//! superscript rewrites a block keeps. It reads the names the old reader exported, as it did there
//! (`crate::base_utilities`), not the writer's own. **Still the driver's, for the one-pass step:** `split_wrapped_affiliation_marks`
//! and its helpers (`split_before_affiliation_marks` … `successor`) still cut a note's raw argument at the marks it
//! prints, the driver's segmentation by spelling; the reading is to hand the emitter the entry's segments instead
//! (docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md, row "Knowledge in, driver out, one pass", step 3).

use crate::base_utilities::*;

/// The name a queued author entry holds: the content of its `\lx@personname` (inside `\lx@author@withinst`), not the
/// `\lx@add@frontmatter@now{ltx:creator}[role=author]` around it, whose words read as surnames ("author" spelled in
/// `Corresponding author: …`; guard `perfect_kernel_batch64u::osa_style_address_email_line`).
pub(in crate::base_utilities) fn queued_name(entry: &Tokens) -> Tokens {
  let tokens = entry.unlist_ref();
  for cs in ["\\lx@author@withinst", "\\lx@personname"] {
    if let Some(at) = tokens.iter().position(|t| *t == T_CS!(cs))
      && tokens
        .get(at + 1)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
    {
      let mut depth = 0usize;
      for (j, t) in tokens.iter().enumerate().skip(at + 1) {
        match t.get_catcode() {
          Catcode::BEGIN => depth += 1,
          Catcode::END => {
            depth -= 1;
            if depth == 0 {
              return Tokens::new(tokens[at + 2..j].to_vec());
            }
          },
          _ => {},
        }
      }
    }
  }
  entry.clone()
}

/// A symbol mark's key, made of its glyph (`unanswered_mark_text`: `\dag` and `\dagger` alike, `\ast` and `*` alike)
/// as hexadecimal code points (`sym-2020`): only letters, digits and `-`, so it passes through a keyval's tokenization
/// as it is (a `#`, `\#`'s glyph, read back as a parameter token raised `Error:misdefined:#`).
pub(in crate::base_utilities) fn symbol_mark_key(mark: &str) -> Option<String> {
  let glyph = unanswered_mark_text(mark)?.replace('\u{2217}', "*");
  (!glyph.chars().all(char::is_alphanumeric)).then(|| {
    s!(
      "sym-{}",
      glyph
        .chars()
        .map(|c| format!("{:x}", c as u32))
        .collect::<Vec<_>>()
        .join("-")
    )
  })
}

/// A class's email command: a macro whose expansion begins with a frontmatter email call — `\lx@add@email`
/// (JHEP.cls.ltxml's `\email`), or one that decides how to record the address (AASTeX 7's `\lx@add@aas@email`, the
/// `\lx@add@email@metadata` of an address the PDF does not print).
pub(in crate::base_utilities) fn is_email_command(cs: &Token) -> Result<bool> {
  let Some(def) = lookup_definition(cs)? else {
    return Ok(false);
  };
  Ok(
    matches!(def.get_expansion(), Some(ExpansionBody::Tokens(body))
    if body.unlist_ref().first().is_some_and(|first| first.with_str(|name| {
      name.strip_prefix("\\lx@add@").is_some_and(|rest| rest.contains("email"))
    }))),
  )
}

/// The address of a call of the email command `cmd` (`\email{x}`) as that command sets it: in the font the class's
/// command gives its address, `\lx@address@urlfont` (llncs's `\email`, `{\tt#1}` at llncs.cls:897; amsart's, JHEP's)
/// or `\lx@address@textfont` (svjour3's, REVTeX 4.1's), else as written — so a contact the author block makes of the
/// call (`\institute{… \\ \email{a@x}}`, `E-mail: \email{x}`) is set as the command's own is. Only the command's own
/// expansion body is read, one level: a command reaching the address macro through another macro is not seen.
pub(in crate::base_utilities) fn email_command_address(
  cmd: &Token,
  address: Tokens,
) -> Result<Tokens> {
  let fonts = [
    T_CS!("\\lx@address@urlfont"),
    T_CS!("\\lx@address@textfont"),
  ];
  let font = match lookup_definition(cmd)?
    .as_ref()
    .and_then(|def| def.get_expansion())
  {
    Some(ExpansionBody::Tokens(body)) => body
      .unlist_ref()
      .iter()
      .find(|t| fonts.contains(t))
      .copied(),
    _ => None,
  };
  Ok(match font {
    Some(font) => Invocation!(font, vec![Some(address)]),
    None => address,
  })
}

/// The addresses `addrs` of an email `line` as tokens, the line's whole-line wrapper (`\texttt`, `\small`, `\url`)
/// repeated on each — but not an email command's, the contact itself.
pub(in crate::base_utilities) fn address_tokens(
  line: &Tokens,
  addrs: Vec<String>,
) -> Result<Vec<Tokens>> {
  // (a row of addresses the `\quad`s of its join part, `dispatch`'s: each piece as it is written, its own font and link —
  // 2609.05531's `\texttt{john3678@…} \quad \texttt{johnson.michael@…}`)
  if let Some(pieces) = address_row_pieces(line) {
    let mut tokens = Vec::with_capacity(addrs.len());
    for piece in &pieces {
      // (a piece whose one address is written in printed braces: `\{hwoo\}@skku.edu`, 2609.18328)
      let written_out = email_addresses(piece).filter(|addresses| {
        addresses.len() == 1
          && piece.unlist_ref().iter().any(|t| {
            *t == T_CS!("\\{") || *t == T_CS!("\\lbrace") || *t == T_CS!("\\textbraceleft")
          })
      });
      match written_out {
        Some(addresses) => tokens.extend(address_tokens(piece, addresses)?),
        None => tokens.extend(bare_email_addresses(piece)?.unwrap_or_default()),
      }
    }
    if tokens.len() == addrs.len() {
      return Ok(tokens);
    }
  }
  let line_wrapper = whole_line_cs_wrapper(line).map(|(cmd, _)| cmd);
  let wrapper = line_wrapper.filter(|cmd| !is_email_wrapper(cmd));
  // (an email command's address set as the command sets it, [`email_command_address`])
  let email_command = line_wrapper.filter(is_email_wrapper);
  // (the line's links to a mailbox, by the address they reach: each address keeps its link, `\texttt{\{}\href{mailto:
  // Xinhao.Zhang@…}{…xinhao.zhang,} \href{mailto:maxime.peyrard@…}{…}\texttt{\}@…}`, 2604.19440)
  let v = line.unlist_ref();
  let links: Vec<(String, Vec<Token>)> = (0..v.len())
    .filter(|at| v[*at] == T_CS!("\\href"))
    .filter_map(|at| {
      let open = at + 1 + v[at + 1..].iter().take_while(|t| **t == T_SPACE!()).count();
      let target = v[open..skip_group(v, open).clamp(open, v.len())].to_vec();
      let (text, _) = link_texts(&v[at..], 0);
      let address = text.strip_prefix("mailto:")?.trim().to_lowercase();
      Some((address, target))
    })
    .collect();
  addrs
    .into_iter()
    .map(|addr| {
      let mut toks = mouth::tokenize(TeXString::assembled(addr.replace('_', "\\_"))).unlist();
      if let Some(cmd) = wrapper {
        let mut wrapped = vec![cmd, T_BEGIN!()];
        wrapped.append(&mut toks);
        wrapped.push(T_END!());
        toks = wrapped;
      }
      if email_command.is_none()
        && let Some((_, target)) = links
          .iter()
          .find(|(address, _)| *address == addr.to_lowercase())
      {
        let mut linked = vec![T_CS!("\\href")];
        linked.extend(target.iter().copied());
        linked.push(T_BEGIN!());
        linked.append(&mut toks);
        linked.push(T_END!());
        toks = linked;
      }
      match email_command {
        Some(cmd) => email_command_address(&cmd, Tokens::new(toks)),
        None => Ok(Tokens::new(toks)),
      }
    })
    .collect()
}

/// The pieces of `line` a top-level `\quad` or `\qquad` parts ([`without_row_spacing`] each), `None` unless two or more: a row of
/// addresses, `dispatch`'s join or a class's own (`\email{\quad a@x \quad\qquad b@y}`, 2601.03839, 2606.13152).
fn address_row_pieces(line: &Tokens) -> Option<Vec<Tokens>> {
  let v = line.unlist_ref();
  let mut pieces = Vec::new();
  let (mut depth, mut from) = (0usize, 0usize);
  for (at, t) in v.iter().enumerate() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && (*t == T_CS!("\\quad") || *t == T_CS!("\\qquad")) => {
        pieces.push(Tokens::new(without_row_spacing(v[from..at].to_vec())));
        from = at + 1;
      },
      _ => {},
    }
  }
  pieces.push(Tokens::new(without_row_spacing(v[from..].to_vec())));
  pieces.retain(|piece| !piece.is_empty());
  (pieces.len() > 1).then_some(pieces)
}

/// A piece of a row of addresses without what parts it from the next, no text of an address: the spaces and ties around it
/// (`\texttt{a@x} ~\quad~~ \texttt{b@y}`, 2610.06195, whose ties read `˜`) and the comma or semicolon it ends with, inside a
/// group that ends it too (`a@x,`, `\texttt{a@x,}`; 2505.22665, 2505.05785's line ending `\texttt{simon.zl@…},`).
pub(in crate::base_utilities) fn without_row_spacing(mut tokens: Vec<Token>) -> Vec<Token> {
  let spacing = |t: &Token| *t == T_SPACE!() || *t == T_ACTIVE!('~');
  // (and the skip it opens with, `\vspace{0.1cm} \texttt{liuqing25@…} \quad \texttt{wuou@…}`, 2606.01635)
  let lead = leading_spacing_end(&tokens);
  tokens.drain(..lead);
  let lead = tokens.iter().take_while(|t| spacing(t)).count();
  tokens.drain(..lead);
  while tokens.last().is_some_and(spacing) {
    tokens.pop();
  }
  let mut end = tokens.len();
  while end > 0 && tokens[end - 1].get_catcode() == Catcode::END {
    end -= 1;
  }
  if end > 0 && (tokens[end - 1] == T_OTHER!(",") || tokens[end - 1] == T_OTHER!(";")) {
    tokens.remove(end - 1);
  }
  while tokens.last().is_some_and(spacing) {
    tokens.pop();
  }
  tokens
}

/// The addresses of an author-block line that is only email addresses (`\texttt{a@x, b@y}`, `{a,b}@dom`), each as
/// tokens ([`address_tokens`]); a line of one address is itself (`\href{mailto:a@x}{a@x}` stays a link). `None` for
/// any other line.
pub(in crate::base_utilities) fn bare_email_addresses(
  line: &Tokens,
) -> Result<Option<Vec<Tokens>>> {
  let Some(addrs) = email_addresses(line).filter(|addrs| !addrs.is_empty()) else {
    return Ok(None);
  };
  if addrs.len() == 1 {
    return Ok(line_is_email(line).then(|| vec![line.clone()]));
  }
  address_tokens(line, addrs).map(Some)
}

/// The operand of an affiliation entry's first mark, which `\lx@sup@setlabel@affiliation` labels it by (a later
/// superscript in the entry is its text, 63i).
pub(in crate::base_utilities) fn entry_mark(entry: &[Token]) -> Option<Vec<Token>> {
  mark_operands(entry).into_iter().next()
}

/// The label `\lx@sup@setlabel@affiliation` sets for a mark's operand: the first of its labels.
pub(in crate::base_utilities) fn mark_label(operand: &[Token]) -> Option<String> {
  clean_frontmatter_labels(&Tokens::new(operand.to_vec()).to_string(), "affiliation")
    .into_iter()
    .next()
}

/// The body a creator's note (`\lx@add@thanks`, `\lx@add@note`) digests: its leading marks as the author line reads
/// them when an author's mark requests them and the note is one legend — a `\thanks{\textsuperscript{$\dagger$} School
/// ...}` is labelled by its mark and goes to the starred authors (2609.00885, 2609.19600) — and the rest, or all of it
/// when no author requests the mark or more legends follow, with the plain superscripts (`\lx@frontmatter@plainsups`): its `10 m$^{2}$` is text, not a mark
/// (KNOWN_PERL_ERRORS #515), and an unrequested leading mark stays shown, its note with its author.
pub(in crate::base_utilities) fn note_body(content: Tokens) -> Tokens {
  let toks = content.unlist();
  let lead = toks.iter().take_while(|t| **t == T_SPACE!()).count();
  let end = lead + leading_marks_end(&toks[lead..]);
  let requested = flush_requested_marks();
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
        .any(|label| requested.contains(label))
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
/// only by their marks (no `\\`), so `author_affil_splits()` (which splits on `\\`)
/// cannot separate them. Each returned segment then flows through
/// `\lx@affiliation@withsup`, which turns its leading mark into the
/// `affiliation:N` label that the authors' own marks already request
/// (`relocate_annotations` links them).
fn split_before_affiliation_marks(tokens: Tokens) -> Result<Vec<Tokens>> {
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
fn split_marked_pieces(tokens: Tokens, line_start: bool) -> Result<Vec<Tokens>> {
  let toks = tokens.unlist();
  // (the marks the queued names request, when they carry any: a number or letter mark none of them requests, after a
  // word and a plain space, is the entry's own text — `$^1$Laboratory for $^3$He Studies`, the isotope — not the next
  // entry's label; after a separator or wider spacing it still parts the entries)
  let names_marked = queued_authors_marked()?;
  // (each mark outside every group: where it starts, the label it sets, whether it is glued to the word before it —
  // a plain space between, no separator: no spacing command, line break, comma or semicolon — and where it ends)
  let marks = top_level_marks(&toks);
  let mut requested_marks = Vec::with_capacity(marks.len());
  for mark in &marks {
    requested_marks.push(match &mark.operand {
      Some(operand) if names_marked => !queued_requesters(operand)?.is_empty(),
      _ => false,
    });
  }
  let text_mark = |at: usize, last_head: Option<&str>| {
    if !names_marked {
      return false;
    }
    let Some(k) = marks.iter().position(|m| m.at == at) else {
      return false;
    };
    let mark = &marks[k];
    let requested = requested_marks[k];
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
  Ok(segments)
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
pub(in crate::base_utilities) fn split_wrapped_affiliation_marks(
  line: Tokens,
) -> Result<Vec<Tokens>> {
  if let Some((cmd, inner, trailing)) = leading_wrapper(&line) {
    // A bare group's opening declarations (`\small`, `\it`, `\color{blue}` with its argument, among the spacing that
    // prints nothing: `{\small\vspace{1mm}\noindent\it $^{1}$…}`, 2407.15815), repeated in each later piece.
    let opening = if cmd.is_none() {
      lead_declarations(inner.unlist_ref())
    } else {
      Vec::new()
    };
    let mut pieces = split_wrapped_affiliation_marks(inner)?;
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
      let mut after = split_marked_pieces(Tokens::new(trailing), false)?.into_iter();
      if let (Some(first), Some(last_piece)) = (after.next(), out.last_mut()) {
        let mut joined = last_piece.clone().unlist();
        joined.extend(first.unlist());
        *last_piece = Tokens::new(joined);
      }
      out.extend(after);
      return Ok(out);
    }
  }
  split_before_affiliation_marks(line)
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

/// Rewrite footnote-SYMBOL author superscripts — `$^{*}$`, `${}^{\dagger}$`,
/// `\textsuperscript{\ddagger}`, a bare `^{*}` — onto the visible
/// `\lx@frontmatter@keepsup` sentinel, BEFORE author-block branch selection.
///
/// Two effects, both wanted (OXIDIZED_DESIGN #52; witness arXiv:2506.06941, where
/// Iman Mirzadeh's literal `$^{*}$` was silently dropped):
///   * the mark no longer counts as an affiliation superscript, so a block whose
///     ONLY superscript is such a note-mark takes the clean no-marker author
///     branch instead of the affiliation-linking one;
///   * the symbol renders as a real superscript instead of being consumed into an
///     `affiliation:*` label that matches no affiliation and is discarded.
///
/// A number or letter glued to such a symbol (`$^{1*}$`, `\textsuperscript{a\dagger}`, the symbol first too:
/// `\textsuperscript{*1}`, 2505.22662) is the affiliation mark it is and the symbol beside it (`split_glued_symbol`): the
/// mark requests affiliation 1, the symbol is shown, as for `$^{1,*}$` (else the one request `affiliation:1*` matched no
/// affiliation, and affiliation 1 went to no one; authblk's `\author{Ann$^{1*}$}`, 77n r3; the neurips fixture's Wenqi
/// Shao, 2507.06670). Both are written in the order they print, for an unanswered mark to show there
/// (`show_unanswered_marks`: 2610.07087's `$^{1*}$`). The symbol is shown as written: a math `*` as `∗`, as the unsplit
/// `$^{1,*}$` keeps it (2210.14772, 2508.09827, 2505.11121; [`keepsup_in_math`]), a text one as `*`.
///
/// NUMERIC/lettered affiliation marks (`$^{1}$`, `\textsuperscript{a}`) are left
/// untouched, so affiliation linking is unaffected. Only the specific
/// superscript-mark token shapes are matched; a superscript inside real math
/// (`$x^2$`) is not (its base is not empty), so math content is preserved.
pub(in crate::base_utilities) fn rewrite_symbol_superscripts(tokens: Tokens) -> Tokens {
  let src = tokens.unlist();
  let n = src.len();
  let mut out: Vec<Token> = Vec::with_capacity(n);
  let mut i = 0;
  // Read a superscript operand at `src[k]`: a braced `{...}` group, or a single
  // token. Returns (operand tokens, index past the operand) or None.
  let read_operand = |k: usize| -> Option<(Vec<Token>, usize)> {
    if k >= n {
      return None;
    }
    if src[k] == T_BEGIN!() {
      let mut depth = 1usize;
      let mut m = k + 1;
      let mut inner = Vec::new();
      while m < n && depth > 0 {
        if src[m] == T_BEGIN!() {
          depth += 1;
        } else if src[m] == T_END!() {
          depth -= 1;
          if depth == 0 {
            break;
          }
        }
        inner.push(src[m]);
        m += 1;
      }
      if depth == 0 {
        Some((inner, m + 1))
      } else {
        None
      }
    } else {
      Some((vec![src[k]], k + 1))
    }
  };
  while i < n {
    let t = src[i];
    // `\textsuperscript{sym}` — always braced in practice.
    if t == T_CS!("\\textsuperscript")
      && src.get(i + 1) == Some(&T_BEGIN!())
      && let Some((sym, next)) = read_operand(i + 1)
      && is_footnote_symbol_operand(&sym)
    {
      out.extend(keepsup(sym));
      i = next;
      continue;
    }
    if t == T_CS!("\\textsuperscript")
      && src.get(i + 1) == Some(&T_BEGIN!())
      && let Some((operand, next)) = read_operand(i + 1)
      && let Some(glued) = glued_marks(&operand)
    {
      for part in glued {
        match part {
          GluedPart::Symbol(symbol) => out.extend(keepsup(symbol)),
          GluedPart::Marks(marks) => {
            out.extend([t, T_BEGIN!()]);
            out.extend(marks);
            out.push(T_END!());
          },
        }
      }
      i = next;
      continue;
    }
    // `$ [ {} ] ^ sym $` — a math span whose whole content is a superscript on an
    // empty base. Rewrite the entire span (delimiters included) to a text sup.
    if t == T_MATH!()
      && let Some(close) = (i + 1..n).find(|&m| src[m] == T_MATH!())
    {
      // optional empty base `{}` before the superscript
      let p = if i + 2 < close && src[i + 1] == T_BEGIN!() && src[i + 2] == T_END!() {
        i + 3
      } else {
        i + 1
      };
      if p < close
        && src[p] == T_SUPER!()
        && let Some((sym, next)) = read_operand(p + 1)
        && next == close
        && is_footnote_symbol_operand(&sym)
      {
        out.extend(keepsup(sym));
        i = close + 1;
        continue;
      }
      if p < close
        && src[p] == T_SUPER!()
        && let Some((operand, next)) = read_operand(p + 1)
        && next == close
        && let Some(glued) = glued_marks(&operand)
      {
        for part in glued {
          match part {
            GluedPart::Symbol(symbol) => out.extend(keepsup_in_math(symbol)),
            GluedPart::Marks(marks) => {
              out.extend_from_slice(&src[i..=p]);
              out.push(T_BEGIN!());
              out.extend(marks);
              out.extend([T_END!(), T_MATH!()]);
            },
          }
        }
        i = close + 1;
        continue;
      }
    }
    // Only the math-delimited (`$…$`) and `\textsuperscript{…}` spellings are
    // rewritten — the correct ways to write a text superscript. A bare `^` is NOT
    // matched here: outside math it is invalid LaTeX, and a `^` reached mid-scan is
    // the superscript operator of real inline math on a non-empty base (`$a^{*}$`),
    // which must be left intact.
    out.push(t);
    i += 1;
  }
  Tokens::new(out)
}

/// [`keepsup`] for a symbol glued to a mark in math (`$^{1*}$`, `$^{1,2*}$`, 2505.11121): a math `*` prints `∗`, shown
/// so (FIDELITY_CHARTER §4, "stays as written"; `\textsuperscript{1*}` keeps its text `*`), its key the `*` the entries
/// answer; beside a second symbol too (`$^{2,1*\dagger}$` prints `∗†`: 2404.16821, 2505.17017). A lone `$^{*}$` shows
/// `*` as it always has, in [`rewrite_symbol_superscripts`]' own branch.
fn keepsup_in_math(sym: Vec<Token>) -> Vec<Token> {
  let shown = sym
    .iter()
    .map(|token| {
      if *token == T_OTHER!("*") {
        T_CS!("\\ast")
      } else {
        *token
      }
    })
    .collect();
  let mut v = visible_sup(shown);
  v.extend([T_CS!("\\lx@frontmatter@symbolmark"), T_BEGIN!()]);
  v.extend(sym);
  v.push(T_END!());
  v
}

/// `\lx@frontmatter@keepsup{<sym>}` as a token list: a superscript shown as it is, requesting nothing.
pub(in crate::base_utilities) fn visible_sup(sym: Vec<Token>) -> Vec<Token> {
  let mut v = vec![T_CS!("\\lx@frontmatter@keepsup"), T_BEGIN!()];
  v.extend(sym);
  v.push(T_END!());
  v
}

/// A footnote-symbol author mark: shown (`visible_sup`), and requesting the affiliation of its symbol
/// (`\lx@frontmatter@symbolmark`).
pub(in crate::base_utilities) fn keepsup(sym: Vec<Token>) -> Vec<Token> {
  let mut v = visible_sup(sym.clone());
  v.extend([T_CS!("\\lx@frontmatter@symbolmark"), T_BEGIN!()]);
  v.extend(sym);
  v.push(T_END!());
  v
}
