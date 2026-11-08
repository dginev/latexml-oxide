//! The text rules on printed ink and the token primitives of `author_block/notes.rs`, moved unchanged (the plan's row
//! "Knowledge in, driver out, one pass", user ruling 2026-11-08).

use super::*;

/// Best-effort content-kind classifier for a creator-scope `\thanks`, used ONLY to
/// attach a semantic `ltx_thanks_<kind>` CSS hook (OXIDIZED_DESIGN #156) — it never
/// affects core semantics. Keyword-matched over the flattened, lowercased note text.
/// Order matters: correspondence and equal-contribution are checked before funding
/// (a "supported by …" note is funding; "contributed equally" is contribution).
/// The funding words are read as words (`funding_word`), not inside one: "erc" in "Perception" or "Commerce", "nsf"
/// in "transfer", "fund" in "Fundamental" read an institution as a funder, so its line was no affiliation (2509.05368
/// "Machine Perception", OXIDIZED_DESIGN_DIVERGENCES #880).
/// "Deceased", as a word, is an annotation on the author of the same kind as "Corresponding author" (FIDELITY_CHARTER §4,
/// user ruling 2026-11-02): a legend line `$^\dagger$Deceased` is its carriers' note, `ltx_thanks_deceased` (0708.2630's
/// `$^*$Deceased`, 1003.2628's `\address{{\addrdagger}Deceased on …}`, 1510.03948's `$^\dagger$deceased`).
/// Witnesses: arXiv 2512.24601 (correspondence), 1510.02728 (funding),
/// 2506.06941 "Equal contribution", 2609.32870 "Equal advising." (contribution). Explicitly best-effort.
/// An author's part in the work is a contribution too (batch 103c, census129 cluster 6): a project role, read as a
/// phrase of whole words ([`project_role`]: "†Project lead", 2609.18514, 2609.19719; "Joint first authors.",
/// 2608.22772; "Core Contribution", 2408.15240), and the work's own circumstances where they open the text ([`opens_with_work_done`]:
/// "Work done during an internship", 2106.09776, 2502.02407) — not "Facebook (work done while at JHU/APL)", an
/// institution the author is at (1702.02684).
///
/// The role readings are the ones an institution's name can hold ("Project Lead Institute, Univ D", "Core Contributor
/// Network, Paris", "Senior Author Program, Acme Corp"), so a text reads by them only where every piece of it is a
/// legend, as the Kind step reads an entry with no carrier's name in it ([`entry_legend_kind`]); else it reads as it
/// would without them — one gate for every reader of the kind (the kernel's line route `queue_digest::legend_note_kind`,
/// the frontmatter observer `resolver::is_legend_text`, the line readers' `== "note"` tests, a `\thanks`'s CSS kind),
/// review rev103c S1. "Project lead: Yi Yang" (2602.19190) is left to the Kind step, which knows its carriers.
pub(in crate::base_utilities) fn classify_thanks(text: &str) -> &'static str {
  match thanks_kind(text, true) {
    "contribution"
      if role_contribution(text) && entry_legend_kind(&printed_words(text), &[]).is_none() =>
    {
      thanks_kind(text, false)
    },
    kind => kind,
  }
}

/// A note's text as it prints, for [`classify_thanks`]'s gate: a caller's TeX source (`$^{\dagger}$Project leader.`, a
/// `\thanks`'s untexed tokens) without its control words and the math, group and tie characters around them — the
/// pieces then read are words, not `^{\dagger`.
fn printed_words(text: &str) -> String {
  let mut out = String::with_capacity(text.len());
  let mut chars = text.chars().peekable();
  while let Some(c) = chars.next() {
    match c {
      '\\' => {
        // (a control word and the letters of its name; a control symbol, `\,`, its one character)
        if chars.peek().is_some_and(|n| n.is_ascii_alphabetic()) {
          while chars.peek().is_some_and(|n| n.is_ascii_alphabetic()) {
            chars.next();
          }
        } else {
          chars.next();
        }
        out.push(' ');
      },
      '$' | '^' | '_' | '{' | '}' | '~' => out.push(' '),
      _ => out.push(c),
    }
  }
  out
}

/// [`classify_thanks`]'s reading, with an author's part in the work read as a contribution when `roles`
/// ([`project_role`], [`opens_with_work_done`]) whatever else the text names — the Kind step's reading of its pieces
/// ([`entry_legend_kind`]).
fn thanks_kind(text: &str, roles: bool) -> &'static str {
  let t = text.to_lowercase();
  if t.contains("correspond") {
    "correspondence"
  // (an equal share of the work, contributed or advised: "Equal advising.", "equal supervision", "advised equally" —
  // 2609.32870's legend line, a note as "Equal contribution" is, review rev94a S4; the share a phrase, the two words
  // side by side: not "I thank my advisor for equally generous support", nor the "Equal Opportunity Advisory Board",
  // review rev94a r2 S1)
  } else if (t.contains("equal") && t.contains("contribut"))
    || equal_share(&t)
    || (roles && {
      // (the role read in the words the note prints, its caller's control words and math out: the frontmatter
      // observer's writer reads a legend's untexed tokens, `\textsuperscript{$\star$}Work performed during …`,
      // 2503.04130: 107c)
      let printed = printed_words(&t);
      project_role(&printed) || opens_with_work_done(&printed)
    })
  {
    "contribution"
  } else if t
    .split(|c: char| !c.is_alphanumeric())
    .any(|w| w == "deceased")
  {
    "deceased"
  } else if t.contains("now at") || t.contains("present address") || t.contains("current address") {
    "address"
  } else if t.split(|c: char| !c.is_alphanumeric()).any(funding_word) {
    "funding"
  } else {
    "note"
  }
}

/// Whether a note's visible text opens with the word "deceased" (`†Deceased 3 March 2021; formerly at Univ C`): an
/// annotation on the author even where an institution follows it, while a text naming an institution with the word
/// (`Department of Deceased Estates, Univ C`) is an affiliation — the reading of the kernel's line route
/// (`queue_digest::legend_note_kind`) and of the frontmatter observer (`is_legend_text`) alike (review rev100a S2, probes
/// e1, e2, d9).
pub(in crate::base_utilities) fn opens_with_deceased(text: &str) -> bool {
  text.split_whitespace().next().is_some_and(|word| {
    word
      .trim_matches(|c: char| !c.is_alphanumeric())
      .eq_ignore_ascii_case("deceased")
  })
}

/// Whether a line's text opens a dedication, the marks before it aside (`insert::kind_dedication_lines`, 107c): "Dedicated
/// to …", "This work (paper, article, thesis, volume) is dedicated to …", "To the memory of …", "In memory of …". Read
/// by the frontmatter observer too: a dedication behind a name's symbol is its carriers' legend
/// (`resolver::is_legend_text`, review rev107c S1).
pub(in crate::base_utilities) fn opens_a_dedication(text: &str) -> bool {
  let words: Vec<String> = text
    .trim_start_matches(|c: char| !c.is_alphabetic())
    .split(|c: char| !c.is_alphanumeric())
    .filter(|w| !w.is_empty())
    .map(str::to_lowercase)
    .collect();
  let words: Vec<&str> = words.iter().map(String::as_str).collect();
  matches!(
    words.as_slice(),
    ["dedicated", "to", _, ..]
      | [
        "this",
        "work" | "paper" | "article" | "thesis" | "volume" | "note",
        "is",
        "dedicated",
        "to",
        _,
        ..
      ]
      | ["to", "the", "memory", "of", _, ..]
      | ["in", "memory", "of", _, ..]
  )
}

/// Whether a lowercased note says an equal share of advising: "equal" or "equally" next to a word of advising,
/// supervising or mentoring, either side ("equal advising", "supervised equally") — [`classify_thanks`].
fn equal_share(text: &str) -> bool {
  let words: Vec<&str> = text
    .split(|c: char| !c.is_alphanumeric())
    .filter(|w| !w.is_empty())
    .collect();
  let equal = |w: &str| w == "equal" || w == "equally";
  let shared = |w: &str| {
    ["advis", "supervis", "mentor"]
      .iter()
      .any(|p| w.starts_with(p))
  };
  words
    .windows(2)
    .any(|pair| (equal(pair[0]) && shared(pair[1])) || (shared(pair[0]) && equal(pair[1])))
}

/// The lowercased words of `text`, letters and digits (`co-first` is two).
fn note_words(text: &str) -> Vec<&str> {
  text
    .split(|c: char| !c.is_alphanumeric())
    .filter(|w| !w.is_empty())
    .collect()
}

/// Whether a lowercased note names an author's role in the project, as a phrase of whole words ([`classify_thanks`]):
/// "project lead" (or "leader", "leads"), "joint first author(s)", "co-first author(s)", "joint co-author(s)" and "joint
/// senior co-author(s)" (2403.05680: 107c), "core contribution" or "core contributor(s)", "senior author(s)" or "senior
/// authorship" ("these authors share senior authorship", 2503.10710) — not "Project Leadership Institute", nor "Core
/// Facility" — and a note that is "first author(s)" and nothing else (`\textsuperscript{*}First author`, 2609.03429:
/// 107c; "The first author is supported by …" is funding).
fn project_role(text: &str) -> bool {
  let words = note_words(text);
  let author = |w: &str| w == "author" || w == "authors";
  words.windows(2).any(|pair| {
    (pair[0] == "project" && matches!(pair[1], "lead" | "leader" | "leads" | "leaders"))
      || (pair[0] == "core" && matches!(pair[1], "contribution" | "contributor" | "contributors"))
      || (pair[0] == "senior" && matches!(pair[1], "author" | "authors" | "authorship"))
  }) || words
    .windows(3)
    .any(|three| matches!(three[0], "joint" | "co") && three[1] == "first" && author(three[2]))
    || words
      .windows(3)
      .any(|three| three[0] == "joint" && three[1] == "co" && author(three[2]))
    || words
      .windows(4)
      .any(|four| four[0] == "joint" && four[1] == "senior" && four[2] == "co" && author(four[3]))
    || matches!(words.as_slice(), [first, who] if *first == "first" && author(who))
}

/// Whether `text` reads as a contribution by an author's part in the work alone ([`project_role`],
/// [`opens_with_work_done`]), not by an equal share ([`classify_thanks`]'s gate).
pub(in crate::base_utilities) fn role_contribution(text: &str) -> bool {
  thanks_kind(text, true) == "contribution" && thanks_kind(text, false) != "contribution"
}

/// Whether a lowercased note opens with the circumstances of the author's work, behind any mark: "work done during …",
/// "work done while …", "work was done during …", "work done as an intern", "work performed during …" (2503.04130: 107c)
/// ([`classify_thanks`]) — the opening words only, as [`opens_with_deceased`] reads its word: "Facebook (work done while
/// at JHU/APL)" names the institution first (1702.02684).
fn opens_with_work_done(text: &str) -> bool {
  let words = note_words(text);
  let rest = match words.as_slice() {
    ["work", "was", "done" | "performed", rest @ ..]
    | ["work", "done" | "performed", rest @ ..] => rest,
    _ => return false,
  };
  matches!(rest.first(), Some(&("during" | "while" | "as")))
}

/// The kind of an affiliation-kind entry whose whole text is a legend — annotations on its authors, with no institution
/// (FIDELITY_CHARTER §4): correspondence, a contribution (an equal share, a project role, the work's circumstances) or an
/// author deceased ([`classify_thanks`]). The text is read without its addresses and their e-mail labels (`ann@x.org`,
/// "Email:", [`is_email_label`]: its e-mails stay the entry's), in pieces parted at its marks, brackets, commas, colons,
/// semicolons and lines ([`LEGEND_SEPARATORS`]; `|`, a line break as the caller writes it): "Equal contribution.|*
/// Correspondence:" (2609.03342) is two legends, "Corresponding author: Ann Able" a legend and the name it gives. Every
/// piece is a legend that names no institution by its words ([`reads_as_affiliation`]: "Department of Equal Contribution
/// Studies, Univ A" is one; "Deceased" opening it, [`opens_with_deceased`], is a legend whatever follows, as the line
/// route and the frontmatter observer read it), a name of one of its `carriers` (the personnames holding the entry, as
/// printed: [`spells_a_carrier`]) or initials ("(Y.W.)"), or prints one letter at most (the "(s)" of "corresponding
/// author(s): Xiaofan Xu, Hua Zhu", 2609.08556), and one piece at least is a legend. `None` for any other text: a mixed
/// line is the affiliation it names ("DeepAuto.ai († : Equal advising)", 2609.34327; "ByteDance *equal contribution",
/// 2404.02132; "Facebook (work done while at JHU/APL)", 1702.02684; "Equal contribution, Google DeepMind", a company's
/// name shaped as a person's, review rev103c S2), and an address change and funding stay as the call declares them (an
/// institution is named for its funder as often). The kind is the whole text's ([`classify_thanks`]: correspondence
/// first). The frontmatter API's Kind step for a legend (`insert::kind_legend_contacts`, OXIDIZED_DESIGN_DIVERGENCES
/// #961).
pub(in crate::base_utilities) fn entry_legend_kind(
  text: &str,
  carriers: &[String],
) -> Option<&'static str> {
  if opens_with_deceased(text) {
    return Some("deceased");
  }
  let text = text
    .split_whitespace()
    .filter(|word| {
      !word.contains('@')
        && !is_email_label(word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-'))
    })
    .collect::<Vec<_>>()
    .join(" ");
  let legend = |piece: &str| {
    let affiliation = || reads_as_affiliation(&Tokens::new(ExplodeText!(piece)));
    match thanks_kind(piece, true) {
      "correspondence" | "contribution" => !affiliation(),
      "deceased" => opens_with_deceased(piece) || !affiliation(),
      _ => false,
    }
  };
  let mut legends = 0;
  for piece in text.split(|c: char| LEGEND_SEPARATORS.contains(c)) {
    let piece = piece.trim();
    // (a piece of one letter at most is none — the plural's "(s)" of "corresponding author(s):", 2609.08556)
    if piece.chars().filter(|c| c.is_alphabetic()).count() < 2 {
      continue;
    }
    // (a name's initials, "Y.W.", "A.-M. K.", are a name: each capital followed by its period — not "GEELY.")
    let initials = {
      let letters: Vec<char> = piece.chars().filter(|c| !matches!(c, '-' | ' ')).collect();
      letters.len() >= 2
        && letters
          .chunks(2)
          .all(|pair| pair.len() == 2 && pair[0].is_uppercase() && pair[1] == '.')
    };
    if legend(piece) {
      legends += 1;
    } else if !initials
      && (!spells_a_carrier(piece, carriers)
        || reads_as_affiliation(&Tokens::new(ExplodeText!(piece))))
    {
      return None;
    }
  }
  match thanks_kind(&text, true) {
    kind @ ("correspondence" | "contribution" | "deceased") if legends > 0 => Some(kind),
    _ => None,
  }
}

/// Whether a piece of a legend's text spells one of its `carriers` by name ([`entry_legend_kind`]): each of its words of
/// two letters or more is a word of one carrier's name, as printed (case aside; its marks, digits and initials not
/// read), and it holds one at least — "Ann Able", "A. King" for Alice King; not "Google DeepMind", "Mountain View".
fn spells_a_carrier(piece: &str, carriers: &[String]) -> bool {
  let words = |text: &str| -> Vec<String> {
    text
      .split(|c: char| !c.is_alphabetic())
      .filter(|w| w.chars().count() >= 2)
      .map(str::to_lowercase)
      .collect()
  };
  let spelled = words(piece);
  !spelled.is_empty()
    && carriers.iter().any(|carrier| {
      let name = words(carrier);
      spelled.iter().all(|word| name.contains(word))
    })
}

/// Where [`entry_legend_kind`] parts a text into its pieces: the footnote symbols marks print, brackets, a comma, colon
/// or semicolon, and `|` (a line, as `insert::kind_legend_contacts` writes a line break).
pub(in crate::base_utilities) const LEGEND_SEPARATORS: &str = "*∗⁎✱†‡§¶⋆★☆♠♣♥♦♮♭♯()[]{},:;|";

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

/// Does `content` start with a letter or number superscript mark (`$^{1}$Univ A`, `$^a$…`, `\textsuperscript{b}…`):
/// a marked affiliation list, its lines labelled by their marks? A symbol (`$^{*}$Corresponding author`) is a note's
/// mark, and a superscript later in the line (`Laboratory for $^{3}$He`) is the affiliation's own text.
pub fn leads_with_mark(content: &Tokens) -> bool {
  // (past a leading font switch or group: `\affiliation{\it $^1$ Key Laboratory …}`, 0808.2763)
  let switch = |t: &Token| MARK_SWITCHES.iter().any(|cs| *t == T_CS!(*cs));
  // (a mark spelled otherwise — `${\,\!}^1$`, `\( ^{1} \)`, `\mbox{${\ }^{1}$}` — as the shared reader reads it,
  // `relocate::spaced_mark_at`: 2112.13115, 0807.4407)
  let all = content.unlist_ref();
  if let Some(lead) = all
    .iter()
    .position(|t| *t != T_SPACE!() && *t != T_BEGIN!() && *t != T_CS!("\\noindent") && !switch(t))
    && let Some((operand, _)) = spaced_mark_at(all, lead)
  {
    return operand
      .iter()
      .find(|t| **t != T_BEGIN!() && **t != T_SPACE!() && !switch(t))
      .is_some_and(|t| {
        t.to_string()
          .chars()
          .next()
          .is_some_and(char::is_alphanumeric)
      });
  }
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

/// Does a mark lead `line` — no text before its first mark, looking inside a leading font command or group past the
/// declarations that open it (`{\color{blue} $^1$Univ A, $^2$Univ B}`: the letters of `blue` are no name)?
pub(in crate::base_utilities) fn marker_leads(line: &Tokens) -> bool {
  // What the line prints from its first glyph on: the spacing and declarations before it print nothing, nor do the unit
  // letters of their lengths, which read as a name before the mark (`\\ \vspace{0.1cm} $^{1}$ Univ A`: Univ A … read as
  // authors, 2407.15815; `\kern2pt$^{1}$Univ A`).
  let printed = |tokens: &[Token]| Tokens::new(tokens[leading_unprinted_end(tokens)..].to_vec());
  let line = &printed(line.unlist_ref());
  // (through each wrapper that opens the line and holds a mark, the wrappers a line keeps of the block's —
  // `{\small{\it \vspace{0.2cm} $^{a}$ also at …}}`, a legend line of 1202.2048's `\begin{small} … {\it …}` once
  // [`wrapper_over_lines_as_lines`] gives each line its wrappers: read past one wrapper only, the spacing's letters
  // before the mark made the legend an author)
  let mut probe = line.clone();
  while let Some((_, inner, _)) = leading_wrapper(&probe) {
    let inner = printed(inner.unlist_ref());
    if author_mark_position(&inner).is_none() {
      break;
    }
    probe = inner;
  }
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
pub(in crate::base_utilities) fn lead_declarations(v: &[Token]) -> Vec<Token> {
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
pub(in crate::base_utilities) fn leading_wrapper(
  line: &Tokens,
) -> Option<(Option<Token>, Tokens, Vec<Token>)> {
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

/// The letter a letter command prints where that is no base letter of its own ([`letter_command`] gives the base one):
/// `\ss` ß, `\o` ø, `\aa` å, `\ae` æ, `\oe` œ, upper case likewise (latex.ltx's `\ss`, `\o`, `\aa`, `\ae`, `\oe`, the
/// letters a name's spelling transliterates, `name_text::spelling_name_text`).
pub(in crate::base_utilities) fn printed_letter_command(cs: &str) -> Option<char> {
  Some(match cs {
    "\\ss" => 'ß',
    "\\o" => 'ø',
    "\\O" => 'Ø',
    "\\aa" => 'å',
    "\\AA" => 'Å',
    "\\ae" => 'æ',
    "\\AE" => 'Æ',
    "\\oe" => 'œ',
    "\\OE" => 'Œ',
    _ => return None,
  })
}

/// The accent commands of latex.ltx / LaTeX's text encodings, whose argument is one letter of the word.
pub(in crate::base_utilities) const ACCENT_COMMANDS: &[&str] = &[
  "\\'", "\\`", "\\^", "\\\"", "\\~", "\\=", "\\.", "\\u", "\\v", "\\H", "\\c", "\\d", "\\b",
  "\\t", "\\r", "\\k",
];
