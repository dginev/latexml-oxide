//! Author notes and thanks: affiliation-mark splits, leading declarations, name annotations.

use super::*;

/// Best-effort content-kind classifier for a creator-scope `\thanks`, used ONLY to
/// attach a semantic `ltx_thanks_<kind>` CSS hook (OXIDIZED_DESIGN #156) — it never
/// affects core semantics. Keyword-matched over the flattened, lowercased note text.
/// Order matters: correspondence and equal-contribution are checked before funding
/// (a "supported by …" note is funding; "contributed equally" is contribution).
/// The funding words are read as words (`funding_word`), not inside one: "erc" in "Perception" or "Commerce", "nsf"
/// in "transfer", "fund" in "Fundamental" read an institution as a funder, so its line was no affiliation (2509.05368
/// "Machine Perception", OXIDIZED_DESIGN_DIVERGENCES #880).
/// Witnesses: arXiv 2512.24601 (correspondence), 1510.02728 (funding),
/// 2506.06941 "Equal contribution", 2609.32870 "Equal advising." (contribution). Explicitly best-effort.
pub(in crate::base_utilities) fn classify_thanks(text: &str) -> &'static str {
  let t = text.to_lowercase();
  if t.contains("correspond") {
    "correspondence"
  // (an equal share of the work, contributed or advised: "Equal advising.", "equal supervision" — 2609.32870's legend
  // line, a note as "Equal contribution" is, review rev94a S4)
  } else if t.contains("equal")
    && ["contribut", "advis", "supervis", "mentor"]
      .iter()
      .any(|w| t.contains(w))
  {
    "contribution"
  } else if t.contains("now at") || t.contains("present address") || t.contains("current address") {
    "address"
  } else if t.split(|c: char| !c.is_alphanumeric()).any(funding_word) {
    "funding"
  } else {
    "note"
  }
}

/// Whether a lowercased word of a note names funding ([`classify_thanks`]): "support" and its forms ("supported",
/// "supporting"), "grant" and "grants", "fund" and its forms ("funded", "funding", "funds"; not "fundamental"), a
/// funder's acronym on its own (NSF and China's NSFC, NIH, ONR, DARPA, ERC, and the funders the in-word reading also
/// caught: Switzerland's SNSF, Belgium's FNRS, Canada's NSERC, the UK's SERC — "SNSF Postdoctoral Fellow",
/// 2507.18869; "Chercheur qualifié FNRS", cond-mat0504758) or its grant number glued to it ("nsf1234").
fn funding_word(word: &str) -> bool {
  const ACRONYMS: [&str; 10] = [
    "nsf", "nsfc", "nih", "onr", "darpa", "erc", "snsf", "fnrs", "nserc", "serc",
  ];
  word.starts_with("support")
    || matches!(word, "grant" | "grants" | "granted")
    || (word.starts_with("fund") && !word.starts_with("fundament"))
    || ACRONYMS.iter().any(|acronym| {
      word
        .strip_prefix(acronym)
        .is_some_and(|rest| rest.chars().all(|c| c.is_ascii_digit()))
    })
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

/// Does `content` start with a letter or number superscript mark (`$^{1}$Univ A`, `$^a$…`, `\textsuperscript{b}…`):
/// a marked affiliation list, its lines labelled by their marks? A symbol (`$^{*}$Corresponding author`) is a note's
/// mark, and a superscript later in the line (`Laboratory for $^{3}$He`) is the affiliation's own text.
pub fn leads_with_mark(content: &Tokens) -> bool {
  // (past a leading font switch or group: `\affiliation{\it $^1$ Key Laboratory …}`, 0808.2763)
  let switch = |t: &Token| MARK_SWITCHES.iter().any(|cs| *t == T_CS!(*cs));
  let tokens: Vec<&Token> = content
    .unlist_ref()
    .iter()
    .filter(|t| **t != T_SPACE!())
    .skip_while(|t| **t == T_BEGIN!() || **t == T_CS!("\\noindent") || switch(t))
    .collect();
  let script = match tokens.as_slice() {
    [t, rest @ ..] if is_mark_command(t) || **t == T_SUPER!() => rest,
    [t, s, rest @ ..] if t.get_catcode() == Catcode::MATH && **s == T_SUPER!() => rest,
    _ => return false,
  };
  script
    .iter()
    .find(|t| ***t != T_BEGIN!() && !switch(t))
    .is_some_and(|t| {
      t.to_string()
        .chars()
        .next()
        .is_some_and(char::is_alphanumeric)
    })
}

/// Does this token list *begin* with a NUMERIC affiliation superscript mark
/// (`$^{1}…` / `$^1…` / `\textsuperscript{1}…`)? This is the signature of the
/// arXiv "`\thanks` abuse" idiom (affiliations smuggled into an author
/// `\thanks{...}` and linked to authors by a leading mark).
///
/// The mark must be a **digit**: affiliation linking is by number, so a numeric
/// mark is the reliable abuse signal. Crucially, footnote-SYMBOL marks
/// (`$^*$`, `$^\dagger$`, `$^\ddagger$`, `$^\S$`) and lettered marks head
/// *legitimate* acknowledgements — corresponding-author / equal-contribution /
/// present-address notes — which must stay `role=thanks`. Requiring a digit
/// excludes them, so those notes are never re-routed into an affiliation that
/// fails to link and is then discarded (i.e. no silent note loss). Operates on
/// the string form so it is agnostic to the Semiverbatim catcode-freeze on
/// `$`/`^`. (user-directed, 2026-07-07; reviewer-hardened.)
pub(in crate::base_utilities) fn starts_with_affiliation_mark(content: &Tokens) -> bool {
  let s = content.to_string();
  let st = s.trim_start();
  let after_opener = match st
    .strip_prefix("$^")
    .or_else(|| st.strip_prefix("\\textsuperscript"))
  {
    Some(rest) => rest,
    None => return false,
  };
  // The superscript content may be braced (`{1}`) or bare (`1`); either way the
  // first content character must be an ASCII digit.
  let content_start = after_opener.strip_prefix('{').unwrap_or(after_opener);
  content_start
    .chars()
    .next()
    .is_some_and(|c| c.is_ascii_digit())
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
  while toks.last() == Some(&T_SPACE!()) {
    toks.pop();
  }
  if toks.last() == Some(&T_OTHER!(",")) || toks.last() == Some(&T_OTHER!(";")) {
    toks.pop();
    while toks.last() == Some(&T_SPACE!()) {
      toks.pop();
    }
  }
  toks
}

/// [`split_before_affiliation_marks`] of a line (`line_start`) or of what follows a wrapper in one (`{…} \vspace{1mm}
/// $^3$Univ C`), where a lead that prints nothing is a separator, not the start of the first institution.
fn split_marked_pieces(tokens: Tokens, line_start: bool) -> Vec<Tokens> {
  let toks = tokens.unlist();
  let mut segments: Vec<Tokens> = Vec::new();
  let mut current: Vec<Token> = Vec::new();
  // A boundary only outside every group: a cut inside `{…}` would leave the pieces unbalanced (a mark
  // list inside a font group is split by `split_wrapped_affiliation_marks`, which repeats the group).
  let mut depth = 0usize;
  for (i, t) in toks.iter().enumerate() {
    let dollar_super = *t == T_MATH!() && toks.get(i + 1).is_some_and(|n| *n == T_SUPER!());
    let is_mark_start = dollar_super || is_mark_command(t);
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
    if is_mark_start && depth == 0 && current.last() == Some(&T_SPACE!()) {
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

/// Does a mark lead `line` — no text before its first mark, looking inside a leading font command or group past the
/// declarations that open it (`{\color{blue} $^1$Univ A, $^2$Univ B}`: the letters of `blue` are no name)?
pub(in crate::base_utilities) fn marker_leads(line: &Tokens) -> bool {
  // What the line prints from its first glyph on: the spacing and declarations before it print nothing, nor do the unit
  // letters of their lengths, which read as a name before the mark (`\\ \vspace{0.1cm} $^{1}$ Univ A`: Univ A … read as
  // authors, 2407.15815; `\kern2pt$^{1}$Univ A`).
  let printed = |tokens: &[Token]| Tokens::new(tokens[leading_unprinted_end(tokens)..].to_vec());
  let line = &printed(line.unlist_ref());
  let probe = match leading_wrapper(line) {
    Some((_, inner, _)) => printed(inner.unlist_ref()),
    None => line.clone(),
  };
  match author_mark_position(&probe) {
    Some(p) => !name_precedes_marker(&probe, p),
    // No mark inside the wrapper (`{}$^1$Univ A`, `\noindent{}$^1$…`): the whole line decides, as before.
    None => author_mark_position(line).is_some_and(|p| !name_precedes_marker(line, p)),
  }
}

/// The spacing commands a line's unprinted lead may hold that take no argument (latex.ltx:9391-9393 `\smallskip` …
/// `\vspace\…amount`; `\hfill`, `\newline`, `\protect`).
pub(in crate::base_utilities) const PARAMETERLESS_SPACING: [&str; 7] = [
  "\\smallskip",
  "\\medskip",
  "\\bigskip",
  "\\hfill",
  "\\hfil",
  "\\newline",
  "\\protect",
];

/// The declarations among a line's unprinted lead, its spacing left out (`\small\vspace{1mm}\noindent\it` → `\small`,
/// `\noindent`, `\it`; [`leading_unprinted_end`]; 2407.15815).
fn lead_declarations(v: &[Token]) -> Vec<Token> {
  let mut declarations = Vec::new();
  let mut i = 0;
  loop {
    let start = i;
    i += leading_spacing_end(&v[i..]);
    let found = opening_declarations(&Tokens::new(v[i..].to_vec()));
    i += found.len();
    declarations.extend(found);
    if i == start {
      // (the space after them is no institution's: `{\color{blue} $^1$A, $^2$B}`)
      while declarations.last() == Some(&T_SPACE!()) {
        declarations.pop();
      }
      return declarations;
    }
  }
}

/// Where a line's unprinted lead ends: its spacing (`leading_spacing_end`) and its declarations
/// (`opening_declarations`), as often as they interleave (`\vspace*{2mm}\noindent`, `\small\vspace{1mm}`). Public
/// for authblk's `\affil` (latexml_package), which reads whether a line opens with a mark past it.
pub fn leading_unprinted_end(v: &[Token]) -> usize {
  let mut i = 0;
  loop {
    let start = i;
    i += leading_spacing_end(&v[i..]);
    i += opening_declarations(&Tokens::new(v[i..].to_vec())).len();
    if i == start {
      return i;
    }
  }
}

/// The declarations that open a group's content (`\small`, `\bfseries`, `\color{blue}` with its argument), up to its
/// first mark, text, or a command over a braced argument (`\textbf{Ann Able}` is content, not a declaration: read
/// as one, `\large\textbf{Ann Able}$^1$` became an affiliation line and lost its author).
pub(in crate::base_utilities) fn opening_declarations(inner: &Tokens) -> Vec<Token> {
  // The declarations that take brace arguments (latex.ltx `\color`, the NFSS font selectors).
  const WITH_ARGUMENTS: &[&str] = &[
    "\\color",
    "\\fontsize",
    "\\fontfamily",
    "\\fontseries",
    "\\fontshape",
    "\\usefont",
  ];
  let v = inner.unlist_ref();
  let mut out = Vec::new();
  let mut i = 0;
  while i < v.len() {
    let t = v[i];
    let braced = v
      .get(i + 1)
      .is_some_and(|n| n.get_catcode() == Catcode::BEGIN);
    let with_arguments = WITH_ARGUMENTS.iter().any(|cs| t == T_CS!(*cs));
    // (spacing that takes a length or a rule's dimensions is no declaration: [`leading_spacing_end`] reads it whole)
    if [
      "\\vspace",
      "\\hspace",
      "\\addvspace",
      "\\vskip",
      "\\hskip",
      "\\kern",
      "\\penalty",
      "\\rule",
    ]
    .iter()
    .any(|cs| t == T_CS!(*cs))
    {
      break;
    }
    if t == T_SPACE!() {
      out.push(t);
      i += 1;
    } else if t.get_catcode() == Catcode::CS
      && !is_mark_command(&t)
      && (with_arguments || !braced)
      // (a command over an optional argument is content too: `{\parbox[c]{\textwidth}{…}}`, 2504.03146)
      && v.get(i + 1) != Some(&T_OTHER!("["))
    {
      out.push(t);
      i += 1;
      // its brace-group arguments
      while i < v.len() && v[i].get_catcode() == Catcode::BEGIN {
        let mut depth = 0usize;
        while i < v.len() {
          let a = v[i];
          out.push(a);
          i += 1;
          match a.get_catcode() {
            Catcode::BEGIN => depth += 1,
            Catcode::END => {
              depth -= 1;
              if depth == 0 {
                break;
              }
            },
            _ => {},
          }
        }
      }
    } else {
      break;
    }
  }
  out
}

/// A line that opens (spaces aside) with `\cmd{…}` or a bare `{…}`: the command (none for a bare group), the group's
/// inside, and the tokens after it. A leading `\textsuperscript{n}` is a mark, not a wrapper.
fn leading_wrapper(line: &Tokens) -> Option<(Option<Token>, Tokens, Vec<Token>)> {
  let v = line.unlist_ref();
  let start = v.iter().position(|t| *t != T_SPACE!())?;
  let (cmd, open) = match v[start].get_catcode() {
    Catcode::CS
      if v
        .get(start + 1)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN) =>
    {
      if is_mark_command(&v[start]) {
        return None;
      }
      (Some(v[start]), start + 1)
    },
    Catcode::BEGIN => (None, start),
    _ => return None,
  };
  let mut depth = 0usize;
  for (i, t) in v.iter().enumerate().skip(open) {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth == 0 {
          return Some((
            cmd,
            Tokens::new(v[open + 1..i].to_vec()),
            v[i + 1..].to_vec(),
          ));
        }
      },
      _ => {},
    }
  }
  None
}

/// Commands whose braced argument is a note or a mark, not part of the name beside it.
pub(in crate::base_utilities) const NAME_ANNOTATIONS: &[&str] = &[
  "\\thanks",
  "\\thanksref",
  "\\footnote",
  "\\altaffilmark",
  "\\inst",
  "\\IEEEauthorrefmark",
  "\\textsuperscript",
  "\\email",
  "\\orcidlink",
  "\\orcidID",
  "\\lx@aas@checkorcid",
  "\\IEEEmembership",
  // IEEEtran's compsoc notes after the names (`\IEEEcompsocitemizethanks{\IEEEcompsocthanksitem …}`, 2408.01902)
  "\\IEEEcompsocitemizethanks",
  // IEEEtran's grade of a name line, made a contact (ieeetran_cls.rs `name_memberships`)
  "\\lx@IEEE@membership",
  // a footnote-symbol mark, rewritten before the parse (`rewrite_symbol_superscripts`: `$^{*}$`)
  "\\lx@frontmatter@keepsup",
  "\\lx@frontmatter@symbolmark",
  // REVTeX 3's note mark on a name (`Itay Furman \cite{itay} and Ofer Biham \cite{ofer}`, cond-mat9611093;
  // `A. V. Germanenko, \cite{cont}`, cond-mat9912430): the cited bibitem is the author's note
  "\\cite",
  // a note's text written among the names (`Bob Baker \footnotetext{On leave from …} $^{1a}$`, revtex4-1 2201.11098;
  // amsppt's `D. Logachev\footnotemark \footnotetext{E-mail: …}`, 2209.04044), elsarticle's note references
  // (`Susarla Raghuram and Anil Bhardwaj \corref{cor1}`, 1201.6291), and a class's per-name note (ecai.cls:450
  // `\newcommand{\institute}[1]{\thanks{#1}}`: `Kenneth Skiba\institute{…} \and Matthias Thimm$^1$`, 2008.02735)
  "\\footnotetext",
  "\\corref",
  "\\fnref",
  "\\tnoteref",
  "\\institute",
];

/// The frontmatter's author annotation commands, whose content is no part of a name ([`is_note_macro`]).
const NAME_ANNOTATORS: &[&str] = &[
  "\\lx@add@contact",
  "\\lx@add@note",
  "\\lx@add@thanks",
  "\\lx@add@email",
  "\\lx@add@affiliation",
];

/// Whether `t` is a note or mark command an author name carries: one of [`NAME_ANNOTATIONS`], or a document's own note
/// command ([`is_note_macro`]).
pub(in crate::base_utilities) fn is_name_annotation(t: &Token) -> bool {
  t.with_str(|s| NAME_ANNOTATIONS.contains(&s)) || is_note_macro(t)
}

/// A macro that is a note command: it takes arguments and every one of them goes into a `\thanks` or `\footnote` in its
/// body, so what it prints beside a name is that note's mark (`\newcommand{\blackthanks}[1]{\begingroup
/// \renewcommand{\thefootnote}{…}\unskip \thanks{#1}\unskip\endgroup}`: `Yuke Peng\textsuperscript{1,}\blackthanks{Co-first
/// authors.}\ ,`, 2506.03876) — or into one of the frontmatter's author annotations ([`NAME_ANNOTATORS`]: a binding's
/// contact command, jheppub's and jinstpub's `\note{…}` = `\lx@add@contact[role=note]{…}`, whose "Corresponding
/// author." read as the name's last word left the corresponding author's e-mail spelling no one, 2406.08712,
/// 2603.01867). A macro that prints an argument itself (`\newcommand{\auth}[2]{#1\thanks{#2}}`) is none.
pub(in crate::base_utilities) fn is_note_macro(cs: &Token) -> bool {
  if cs.get_catcode() != Catcode::CS || lookup_conditional(cs).is_some() {
    return false;
  }
  let Ok(Some(defn)) = lookup_definition(cs) else {
    return false;
  };
  if !defn.is_expandable() || defn.get_num_args() == 0 {
    return false;
  }
  let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
    return false;
  };
  let body = body.unlist_ref();
  let mut notes = 0;
  let mut i = 0;
  while i < body.len() {
    let t = body[i];
    let annotator = t.with_str(|s| NAME_ANNOTATORS.contains(&s));
    // (a class's address list, `\lx@add@contact@list{\op}{#1}` — jheppub's `\emailAdd`, iopart's `\ead` — makes a contact
    // of each address, as an annotator does: its operation, then its list, the note's argument; `\emailAdd` inside
    // `\author[a]{Ann Able \emailAdd{…}}` is Ann Able's e-mail, not an e-mail line of the name, rev78f probe j13)
    if t == T_CS!("\\lx@add@contact@list")
      && body
        .get(i + 1)
        .is_some_and(|n| n.get_catcode() == Catcode::BEGIN)
    {
      let list = skip_group(body, i + 1);
      if body
        .get(list)
        .is_some_and(|n| n.get_catcode() == Catcode::BEGIN)
      {
        notes += 1;
        i = skip_group(body, list);
        continue;
      }
    }
    if t == T_CS!("\\thanks") || t == T_CS!("\\footnote") || annotator {
      let mut j = i + 1;
      while body.get(j) == Some(&T_SPACE!()) {
        j += 1;
      }
      // (an annotator's keyvals, `[role=note]`, are no argument of the name)
      if annotator
        && body.get(j) == Some(&T_OTHER!("["))
        && let Some(close) = body[j..].iter().position(|u| *u == T_OTHER!("]"))
      {
        j += close + 1;
      }
      if body
        .get(j)
        .is_some_and(|n| n.get_catcode() == Catcode::BEGIN)
      {
        notes += 1;
        i = skip_group(body, j);
        continue;
      }
    }
    if t.get_catcode() == Catcode::ARG {
      return false;
    }
    i += 1;
  }
  notes > 0
}

/// The letter a letter command stands for (latex.ltx / the OT1 and T1 encodings: `\L` Ł, `\o` ø, `\ss` ß), as a
/// name-shape test reads it: its base letter, keeping its case.
pub(in crate::base_utilities) fn letter_command(cs: &str) -> Option<char> {
  match cs {
    "\\L" | "\\l" | "\\O" | "\\o" | "\\i" | "\\j" => cs.chars().nth(1),
    "\\AA" | "\\AE" => Some('A'),
    "\\aa" | "\\ae" => Some('a'),
    "\\OE" => Some('O'),
    "\\oe" => Some('o'),
    "\\ss" => Some('s'),
    // (T1 / LY1 `\dj` đ, `\DJ` Đ, `\dh` ð, `\DH` Ð, `\th` þ, `\TH` Þ, `\ng` ŋ, `\NG` Ŋ: t1enc.def)
    "\\dj" | "\\dh" => Some('d'),
    "\\DJ" | "\\DH" => Some('D'),
    "\\th" => Some('t'),
    "\\TH" => Some('T'),
    "\\ng" => Some('n'),
    "\\NG" => Some('N'),
    _ => None,
  }
}

/// The accent commands of latex.ltx / LaTeX's text encodings, whose argument is one letter of the word.
pub(in crate::base_utilities) const ACCENT_COMMANDS: &[&str] = &[
  "\\'", "\\`", "\\^", "\\\"", "\\~", "\\=", "\\.", "\\u", "\\v", "\\H", "\\c", "\\d", "\\b",
  "\\t", "\\r", "\\k",
];
