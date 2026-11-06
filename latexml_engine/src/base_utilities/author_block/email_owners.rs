//! E-mail addresses in an author block: labelled e-mails, owners spelled or by elimination.

use super::*;

/// A `\\`-delimited line in a no-marker author block is really an EMAIL, not an
/// affiliation, when its visible text is a single bare address: it contains `@`
/// and NO whitespace. Institution names always contain spaces, so this is
/// conservative by construction (near-zero false positives). Relabels the
/// trailing `\texttt{user@host}` line of the NeurIPS idiom, which would
/// otherwise become a bogus `role="affiliation"` (OXIDIZED_DESIGN #52; witness
/// arXiv:2606.00315). Visible text = letters/other chars only; control sequences
/// (`\texttt`, `\textit`, `\{`, `\}`) and grouping are skipped.
pub(in crate::base_utilities) fn line_is_email(line: &Tokens) -> bool {
  let mut visible = String::new();
  for t in line.unlist_ref() {
    if t.code == Catcode::LETTER || t.code == Catcode::OTHER {
      t.with_str(|s| visible.push_str(s));
    } else if t.code == Catcode::SPACE || *t == T_CS!("\\quad") {
      // (the `\quad` of a row of addresses, `dispatch`'s join, parts them as a space does)
      visible.push(' ');
    }
  }
  let v = visible.trim();
  !v.is_empty() && v.contains('@') && !v.chars().any(|c| c.is_whitespace())
}

/// An author-block line holding email addresses behind the author's own label, or given by the class's email command,
/// as its label and addresses:
/// `E-mail: \email{x}` (JHEP.cls/PoS.cls author blocks, astro-ph0611258), `E-mail: \texttt{x@y}`, `\email{x}`. Each
/// address becomes `\lx@add@email`, the label (as written, "E-mail:") its `name`, so it stays markup a stylesheet can
/// show or hide (as the default "Email: " is). Before, the label stood as an affiliation of its own ("E-mail: ") and
/// a lone `\email{x}` line nested its own `\lx@add@email` in another, leaving an empty email contact. A link line to a
/// mailbox is one too, under whatever label it prints (`\textbf{Correspondence:} \href{mailto:…}{…}`, [`mailto_line`]).
/// `None` for any other line.
pub(in crate::base_utilities) fn labelled_emails(
  line: &Tokens,
) -> Result<Option<(Option<Tokens>, Vec<EmailAddress>)>> {
  // (a footnote symbol before the label is the line's mark, no part of it: `$^*$Corresponding author:
  // \href{mailto:…}{…}`, the r5 re-review's u23)
  let line = &Tokens::new(after_symbol_mark(line.unlist_ref()).to_vec());
  let (label, rest) = split_email_label(line);
  if let Some(calls) = email_command_calls(&rest)? {
    return Ok(Some((label, calls)));
  }
  let commands = rest
    .unlist_ref()
    .iter()
    .filter(|t| t.code == Catcode::CS)
    .map(is_email_command)
    .collect::<Result<Vec<bool>>>()?;
  if label.is_some() && line_is_email(&rest) && !commands.contains(&true) {
    return Ok(Some((label, vec![EmailAddress {
      address: rest,
      call:    None,
    }])));
  }
  // (several, a comma or a semicolon between them, each an address or a link to one: `Emails:
  // \href{mailto:a@x}{a@x}, \href{mailto:b@y}{b@y}`, the r5 re-review's u22)
  if label.is_some() && !commands.contains(&true) {
    let items = split_top_level_list(rest.unlist_ref());
    if items.len() > 1
      && items
        .iter()
        .all(|item| line_is_email(item) || mailto_line(item).is_some())
    {
      // (a tie after the comma parts them, no text of the address: `e-mail: a@x, ~~ b@y`)
      return Ok(Some((
        label,
        items
          .into_iter()
          .map(|address| EmailAddress {
            address: Tokens::new(without_row_spacing(address.unlist())),
            call:    None,
          })
          .collect(),
      )));
    }
    // (or a list the address reader parts, a brace list among them: `Email: aable@x.edu; \{bbaker, ccole\}@y.edu`,
    // 2604.10316; the r8 re-review's §B, probes t1, t2 — had been one affiliation)
    if let Some(addresses) = email_addresses(&rest).filter(|addresses| addresses.len() > 1) {
      return Ok(Some((
        label,
        address_tokens(&rest, addresses)?
          .into_iter()
          .map(|address| EmailAddress { address, call: None })
          .collect(),
      )));
    }
  }
  // …and a link line to a mailbox, under any label (`\textbf{Correspondence:} \href{mailto:…}{…}`, [`mailto_line`]),
  // an address each of a brace list it prints ([`mailto_addresses`]); and several such links after one label, a comma or
  // a semicolon between them (`\textbf{Correspondence:} \href{mailto:a@x}{a@x}, \href{mailto:c@x}{c@x}`, 2604.25444;
  // the r8 re-review's §B, probe t6 — had been one affiliation)
  if let Some((label, address)) = mailto_line(line) {
    return Ok(Some((label, mailto_addresses(address))));
  }
  let items = split_top_level_list(line.unlist_ref());
  if items.len() < 2 {
    return Ok(None);
  }
  let mut label = None;
  let mut addresses = Vec::new();
  for (at, item) in items.iter().enumerate() {
    match mailto_line(item) {
      Some((first, address)) if at == 0 => {
        label = first;
        addresses.extend(mailto_addresses(address));
      },
      Some((None, address)) => addresses.extend(mailto_addresses(address)),
      _ => return Ok(None),
    }
  }
  Ok(Some((label, addresses)))
}

/// The addresses a mailbox link prints ([`mailto_line`]): itself, or, when it prints a list of them, each address of it
/// ([`email_addresses`]: `\href{mailto:julia.belikova@skol.tech}{\{julia.belikova, a.panchenko\}@skol.tech}`,
/// 2602.12235; the r7 re-review's finding 7) — the one its target reaches kept the link, the others as they print.
fn mailto_addresses(link: EmailAddress) -> Vec<EmailAddress> {
  let v = link.address.unlist_ref();
  let Some(at) = v.iter().position(|t| *t == T_CS!("\\href")) else {
    return vec![link];
  };
  let (open, close) = link_printed_argument(v, at);
  let Some(addresses) =
    email_addresses(&Tokens::new(v[open..close].to_vec())).filter(|a| a.len() > 1)
  else {
    return vec![link];
  };
  let ((target_open, target_close), _) = link_arguments(v, at);
  let (target, _) = link_texts(v, at);
  let reached = target.trim_start_matches("mailto:").trim().to_lowercase();
  addresses
    .into_iter()
    .map(|address| {
      let printed = mouth::tokenize(TeXString::assembled(address.replace('_', "\\_"))).unlist();
      let tokens = if address.to_lowercase() == reached {
        let mut linked = vec![T_CS!("\\href")];
        linked.extend_from_slice(&v[target_open..target_close]);
        linked.push(T_BEGIN!());
        linked.extend(printed);
        linked.push(T_END!());
        linked
      } else {
        printed
      };
      EmailAddress {
        address: Tokens::new(tokens),
        call:    None,
      }
    })
    .collect()
}

/// `tokens` after the footnote symbol mark that leads them, as [`rewrite_symbol_superscripts`] writes it
/// (`\lx@frontmatter@keepsup{*}\lx@frontmatter@symbolmark{*}`), a size or font before it and the spaces after it;
/// `tokens` when none does.
pub(in crate::base_utilities) fn after_symbol_mark(tokens: &[Token]) -> &[Token] {
  let whole = tokens;
  // (through a group around the whole piece: `{\small $^{\dagger}$\,University of Toronto}`, 2606.28166;
  // `{\it \normalsize $^\sharp$Department of Biostatistics, …}`, 2602.16540 — the r7 re-review's finding 1)
  let tokens = unwrapped(tokens);
  // (and a size or font before it, `\normalsize $^{\dagger}$Department of Statistics`, 2505.23260)
  let lead = tokens
    .iter()
    .take_while(|t| {
      **t == T_SPACE!()
        || (t.get_catcode() == Catcode::CS && t.with_str(|s| NAME_TEXT_COMMANDS.contains(&s)))
    })
    .count();
  if tokens.get(lead) != Some(&T_CS!("\\lx@frontmatter@keepsup")) {
    return whole;
  }
  let mut at = skip_group(tokens, lead + 1);
  if tokens.get(at) == Some(&T_CS!("\\lx@frontmatter@symbolmark")) {
    at = skip_group(tokens, at + 1);
  }
  let at = at.min(tokens.len());
  &tokens[at
    + tokens[at..]
      .iter()
      .take_while(|t| **t == T_SPACE!())
      .count()..]
}

/// `tokens` inside the groups around the whole of them, a one-argument font command's too (`\textit{…}`), and the spaces
/// around those (`{\small $^{\dagger}$\,Univ A \qquad $^{\ddagger}$\,EPFL}`, 2606.28166): what a wrapper group holds,
/// its font aside, is the piece; `tokens` when no group holds them whole.
pub(in crate::base_utilities) fn unwrapped(tokens: &[Token]) -> &[Token] {
  let mut v = tokens;
  loop {
    let start = v.iter().take_while(|t| **t == T_SPACE!()).count();
    let end = v
      .iter()
      .rposition(|t| *t != T_SPACE!())
      .map_or(start, |at| at + 1);
    let inner = &v[start..end.max(start)];
    // (and a one-argument font command around them, as a group: `\textit{$^{\dagger}$Univ A \qquad $^{\ddagger}$Univ
    // B}`, the r8 re-review's finding 8, probe s7)
    let font = usize::from(
      inner.len() >= 3
        && inner[0].get_catcode() == Catcode::CS
        && inner[0].with_str(|s| s.starts_with("\\text") || s == "\\emph")
        && inner[0].with_str(|s| NAME_TEXT_COMMANDS.contains(&s)),
    );
    let group = &inner[font..];
    if group
      .first()
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      && skip_group(group, 0) == group.len()
      && group.len() >= 2
    {
      v = &group[1..group.len() - 1];
    } else {
      return v;
    }
  }
}

/// `tokens` before the footnote symbol marks that end them after their text (`Carnegie Mellon University, Pittsburgh, PA,
/// USA$^{\dagger}$`, `\texttt{mazeyuwhu@gmail.com}$^{\dagger}$`, 2109.13296; the r7 re-review's finding 1), inside a
/// group around the whole of them ([`unwrapped`]), without the spaces before the marks, and the marks as written
/// (`\lx@frontmatter@keepsup{…}\lx@frontmatter@symbolmark{…}`, [`keepsup`]): a mark after its text labels that text, as
/// one before it does. `None` when no mark ends them, or nothing stands before it.
pub(in crate::base_utilities) fn before_trailing_symbol_mark(
  tokens: &[Token],
) -> Option<(&[Token], &[Token])> {
  let v = unwrapped(tokens);
  let last = v.iter().rposition(|t| *t != T_SPACE!())? + 1;
  let mut end = last;
  while let Some(open) = group_opening(v, end)
    && open >= 1
    && v[open - 1] == T_CS!("\\lx@frontmatter@symbolmark")
    && let Some(sup) = group_opening(v, open - 1)
    && sup >= 1
    && v[sup - 1] == T_CS!("\\lx@frontmatter@keepsup")
  {
    end = sup - 1;
  }
  if end == last {
    return None;
  }
  let text = v[..end].iter().rposition(|t| *t != T_SPACE!())? + 1;
  Some((&v[..text], &v[end..last]))
}

/// Where the group that the `}` at `end - 1` of `v` closes opens; `None` when `v[end - 1]` closes none.
fn group_opening(v: &[Token], end: usize) -> Option<usize> {
  if end == 0 || v.get(end - 1)?.get_catcode() != Catcode::END {
    return None;
  }
  let mut depth = 0usize;
  for at in (0..end).rev() {
    match v[at].get_catcode() {
      Catcode::END => depth += 1,
      Catcode::BEGIN => {
        depth -= 1;
        if depth == 0 {
          return Some(at);
        }
      },
      _ => {},
    }
  }
  None
}

/// The items of `tokens` at its commas and semicolons outside groups, each without the spaces around it; empty items
/// dropped.
fn split_top_level_list(tokens: &[Token]) -> Vec<Tokens> {
  let mut items: Vec<Tokens> = Vec::new();
  let mut item: Vec<Token> = Vec::new();
  let mut depth = 0usize;
  let mut finish = |item: &mut Vec<Token>| {
    let start = item.iter().take_while(|t| **t == T_SPACE!()).count();
    let end = item
      .iter()
      .rposition(|t| *t != T_SPACE!())
      .map_or(0, |at| at + 1);
    if start < end {
      items.push(Tokens::new(item[start..end].to_vec()));
    }
    item.clear();
  };
  for t in tokens {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ if depth == 0 && (*t == T_OTHER!(",") || *t == T_OTHER!(";")) => {
        finish(&mut item);
        continue;
      },
      _ => {},
    }
    item.push(*t);
  }
  finish(&mut item);
  items
}

/// An address's tokens as TeX prints them where a brace is set in math: `$\{$`, `$\lbrace$` the brace `\{`, `$\}$`,
/// `$\rbrace$` the brace `\}` — the math shifts read as characters (a class's verbatim `\email` argument: llncs's
/// `\email{$\{$fargyriou,mikebekos,symvonis$\}$@math.ntua.gr}`, 1106.2694, 2308.00146, 1602.04934) or as shifts —, so
/// a list of local parts in math braces is the list in text braces (`\lx@add@email`, `\lx@split@addresses`; census138
/// ADDR-b, OXIDIZED_DESIGN_DIVERGENCES #972). Anything else in math stays as it is.
pub(in crate::base_utilities) fn math_braces_as_printed(address: &Tokens) -> Tokens {
  let v = address.unlist_ref();
  let shift = |t: &Token| *t == T_OTHER!("$") || t.get_catcode() == Catcode::MATH;
  let brace = |t: &Token| {
    if *t == T_CS!("\\{") || *t == T_CS!("\\lbrace") {
      Some(T_CS!("\\{"))
    } else if *t == T_CS!("\\}") || *t == T_CS!("\\rbrace") {
      Some(T_CS!("\\}"))
    } else {
      None
    }
  };
  let spaces = |from: usize| {
    from
      + v[from.min(v.len())..]
        .iter()
        .take_while(|t| **t == T_SPACE!())
        .count()
  };
  let mut out: Vec<Token> = Vec::with_capacity(v.len());
  let mut at = 0;
  while at < v.len() {
    if shift(&v[at]) {
      let inner = spaces(at + 1);
      if let Some(printed) = v.get(inner).and_then(brace) {
        let close = spaces(inner + 1);
        if v.get(close).is_some_and(shift) {
          out.push(printed);
          at = close + 1;
          continue;
        }
      }
    }
    out.push(v[at]);
    at += 1;
  }
  Tokens::new(out)
}

/// One address of an email line, and the class's email command that gave it (`\email{x}`), if any.
pub(in crate::base_utilities) struct EmailAddress {
  pub(in crate::base_utilities) address: Tokens,
  pub(in crate::base_utilities) call:    Option<Tokens>,
}

/// The words that can stand for a person in an email address ([`spelled_owners`]), lowercased, of the name as printed,
/// its notes aside (1706.03762's `Ashish Vaswani\thanks{… Noam proposed …}` is not "noam").
#[derive(Clone, Debug, Default)]
pub(in crate::base_utilities) struct NameKeys {
  /// the name's words after the first that have three letters or more, counted in their base spelling ([`SpelledWord`]:
  /// `Ph.~Boucaud` "boucaud", `J.~Rodr\'iguez-Quintero` "rodriguez", "quintero")
  surnames: Vec<SpelledWord>,
  /// the name's words after the first of two letters (`Weili An` "an", `Wei Li` "li"), which spell only a local part of
  /// those letters alone, digits aside (`an107@purdue.edu` Weili An's, not Anusuya Nallathambi's initials "an", 2608.22602)
  short_surnames: Vec<String>,
  /// its first word, when it has three letters or more (`Ahna R. Girshick` "ahna"; not the initial of `J.~Smith`)
  given: Option<SpelledWord>,
  /// the spellings an address opens with that join two of its words: the first word's initial and a later word of two
  /// letters or more (`Prathamjyot Singh` "psingh", 2608.10670's `psingh1_be22@…` among three Singhs; `Tianjian Li`
  /// "tli", 2606.23525's `tli104@…`; `Karamvir Singh Batra` "kbatra"), a later word before the whole first, the
  /// family name first (`Fei Wu` "wufei", `Yu Cheng` "chengyu": 2509.10377), or after it (`Di Wu` "diwu", 2604.01354;
  /// `Mo Yu` "moyu", 2608.18082 — 77p r10, where an address no name spells is the block's), and the initials of the
  /// words before the last with the last (`Kai-Wei Chang` "kwchang", 2604.01354); and the initials of every word, two or more
  /// (`Ann Able` "aa"), or of the first and the last when more stand between (`Joseph G. Wallwork` "jgw" and "jw",
  /// 2407.00382's `jw2423@cam.ac.uk`), which only a local part of those letters alone spells
  pub(in crate::base_utilities) joined: Vec<String>,
  pub(in crate::base_utilities) initials: Vec<String>,
  /// the same keys of the name's transliterated spelling, where it differs from the base letters (`Schäffler`
  /// "schaeffler" beside "schaffler", [`resolver::transliterated`]): a name scores the better of its two spellings
  transliterated: Option<Box<NameKeys>>,
}

/// A word of a name, as [`spelled_owners`] scores it: its letters (as an address spells them), and its length class —
/// the letters of its base spelling, which decide whether it is a surname whose first six letters spell it, a given
/// name a local part may hold anywhere, or a short word only a local part of its letters alone spells: the
/// transliteration never lifts a word over a threshold (`Wei L\"u`'s "lue" spells no `bluesky@`, review rev107a
/// finding 3; OXIDIZED_DESIGN_DIVERGENCES #970).
#[derive(Clone, Debug, Default)]
pub(in crate::base_utilities) struct SpelledWord {
  text:    String,
  letters: usize,
}

impl NameKeys {
  /// The keys of a name as an address spells it: its letters folded to their base letters (`Martínez` "martinez",
  /// [`resolver::base_letters`]) and, beside those, transliterated (`Schäffler` "schaeffler", `Wei{\ss}enborn`
  /// "weissenborn", `Hj\o rth` "hjoerth", [`resolver::transliterated`]; html_feedback #7233, OXIDIZED_DESIGN_DIVERGENCES
  /// #970) — a letter written with an accent or letter command read as the letter it prints ([`spelling_name_text`]).
  pub(in crate::base_utilities) fn of(name: &Tokens) -> NameKeys {
    let text = spelling_name_text(name.unlist_ref());
    let base = resolver::base_letters(&text);
    let transliterated = resolver::base_letters(&resolver::transliterated(&text));
    let base_words = name_words(&base);
    let classes: Vec<usize> = base_words.iter().map(|word| word.chars().count()).collect();
    let mut keys = NameKeys::of_words(&base_words, &classes);
    if transliterated != base {
      // (each transliterated word in its base spelling's length class, the words one for one; else its own)
      let words = name_words(&transliterated);
      let own: Vec<usize> = words.iter().map(|word| word.chars().count()).collect();
      let classes = if words.len() == classes.len() {
        &classes
      } else {
        &own
      };
      let mut spelled = NameKeys::of_words(&words, classes);
      spelled.joined.extend(hyphenated_surname(&transliterated));
      keys.transliterated = Some(Box::new(spelled));
    }
    keys.joined.extend(hyphenated_surname(&base));
    keys
  }

  /// The keys of a name as it prints, its letters unfolded, and of its transliteration (`Müller` "mueller", the
  /// letters [`resolver::transliteration`] spells, the others as printed): whom `insert::respell_emails` and the
  /// frontmatter observer's `Resolver::respell_emails` move an author's e-mail to, an address spelling that author as
  /// printed — a transliterated spelling is one (user ruling 2026-11-05: `joerg.mueller@` is Jörg Müller's, "also
  /// written Joerg Mueller"; `\email{mueller-group@x.de}` on Anna Smith's own call moves to `\author{J\"org M\"uller}`
  /// as its ASCII twin does, FIDELITY_CHARTER §4 #420), a base-letter fold is not (`muller-group@` stays). Each word in
  /// its printed length class ([`SpelledWord`]); a tie between a literal and a transliterated spelling is a tie
  /// (`mueller@` beside Hans Mueller and Jörg Müller moves to neither, user ruling 2026-11-05). It expects the name's
  /// digested tokens, as both respelling sites pass them: an undigested `Sch\"affler` reads through
  /// [`visible_name_text`]'s accent fold as "Schaffler", whose base fold it then spells (review rev107a2 N3).
  pub(in crate::base_utilities) fn as_printed(name: &Tokens) -> NameKeys {
    let printed = name_words(&visible_name_text(name.unlist_ref()));
    let classes: Vec<usize> = printed.iter().map(|word| word.chars().count()).collect();
    let mut keys = NameKeys::of_words(&printed, &classes);
    let transliterated = name_words(&resolver::transliterated(&spelling_name_text(
      name.unlist_ref(),
    )));
    if transliterated != printed {
      let own: Vec<usize> = transliterated
        .iter()
        .map(|word| word.chars().count())
        .collect();
      let classes = if transliterated.len() == classes.len() {
        &classes
      } else {
        &own
      };
      keys.transliterated = Some(Box::new(NameKeys::of_words(&transliterated, classes)));
    }
    keys
  }

  /// The name's spellings: its base letters', and its transliterated one where it has one.
  pub(in crate::base_utilities) fn spellings(&self) -> impl Iterator<Item = &NameKeys> {
    std::iter::once(self).chain(self.transliterated.as_deref())
  }

  /// The keys of a name's `words`, each in its length class (`classes`, its letters in the base spelling: [`SpelledWord`]).
  fn of_words(words: &[String], classes: &[usize]) -> NameKeys {
    let Some((first, later)) = words.split_first() else {
      return NameKeys::default();
    };
    let class = |at: usize| classes.get(at).copied().unwrap_or_default();
    let initial: String = first.chars().take(1).collect();
    let mut joined: Vec<String> = Vec::new();
    for (_, word) in later
      .iter()
      .enumerate()
      .filter(|(at, _)| class(at + 1) >= 2)
    {
      joined.push(format!("{initial}{word}"));
      if class(0) >= 2 {
        joined.push(format!("{word}{first}"));
        joined.push(format!("{first}{word}"));
      }
    }
    // (the initials of every word before the last, then the last: `Kai-Wei Chang` "kwchang", 2604.01354)
    if let Some((last, before)) = words.split_last()
      && before.len() >= 2
      && class(words.len() - 1) >= 2
    {
      let initials: String = before
        .iter()
        .filter_map(|word| word.chars().next())
        .collect();
      joined.push(format!("{initials}{last}"));
    }
    let initials: String = words
      .iter()
      .filter_map(|word| word.chars().next())
      .collect();
    NameKeys {
      surnames: later
        .iter()
        .enumerate()
        .filter(|(at, _)| class(at + 1) >= 3)
        .map(|(at, word)| SpelledWord {
          text:    word.clone(),
          letters: class(at + 1),
        })
        .collect(),
      // (a name particle another word of the name follows is no surname of its own: `Ana de Souza` spells no `de@`,
      // review rev100b S3; the last word is the surname whatever it reads, `Thanh Le`'s `le@`, review rev100b2 S3)
      short_surnames: later
        .iter()
        .enumerate()
        .filter(|(at, word)| {
          class(at + 1) == 2
            && (*at + 1 == later.len()
              || !matches!(word.as_str(), "de" | "la" | "da" | "le" | "di" | "du"))
        })
        .map(|(_, word)| word.clone())
        .collect(),
      given: (class(0) >= 3).then(|| SpelledWord {
        text:    first.clone(),
        letters: class(0),
      }),
      joined,
      initials: [
        Some(initials.clone()),
        (words.len() >= 3).then(|| {
          initials
            .chars()
            .take(1)
            .chain(initials.chars().last())
            .collect()
        }),
      ]
      .into_iter()
      .flatten()
      .filter(|initials: &String| initials.chars().count() >= 2)
      .collect(),
      transliterated: None,
    }
  }
}

/// A name's words, lowercased: its runs of letters, an apostrophe between two letters inside its word (`D'Amico`
/// "damico", whose address `damicos@` holds it whole, 2606.17317; `Gol'dshtein` "goldshtein" — brief 107f, where a
/// surname spells only a run of the local part's words).
/// A hyphen-linked surname with its hyphen dropped, a spelling as printed (`L. Tal-Or` "talor": `levtalor@`, 2112.09029,
/// review rev107g), as a hyphenated given name's already is (`Wang-Chiew` "wangchiew"). The name's last word only, its
/// parts all letters: spaced words stay apart (`de Vries`). A joined key, so where it ends a word it does not open it is
/// a weak spelling ([`LocalPart::joined_standing`]) and owns only an address no other name spells.
fn hyphenated_surname(text: &str) -> Option<String> {
  let last = text.split_whitespace().last()?;
  let parts: Vec<&str> = last.split(['-', '\u{2010}', '\u{2011}']).collect();
  (parts.len() >= 2
    && parts
      .iter()
      .all(|part| !part.is_empty() && part.chars().all(char::is_alphabetic)))
  .then(|| parts.concat().to_lowercase())
}

fn name_words(text: &str) -> Vec<String> {
  let chars: Vec<char> = text.chars().collect();
  let text: String = chars
    .iter()
    .enumerate()
    .filter(|&(at, c)| {
      !(matches!(c, '\'' | '\u{2019}' | '\u{02bc}')
        && at > 0
        && chars[at - 1].is_alphabetic()
        && chars.get(at + 1).is_some_and(|next| next.is_alphabetic()))
    })
    .map(|(_, c)| *c)
    .collect();
  text
    .split(|c: char| !c.is_alphabetic())
    .filter(|word| !word.is_empty())
    .map(str::to_lowercase)
    .collect()
}

/// An address's letters and other characters, as [`spelled_owners`] reads them.
pub(in crate::base_utilities) fn address_text(address: &Tokens) -> String {
  // (a colour, a font, a scale is no letter of it, [`without_non_printing_arguments`]: 2604.19440, 2609.25669)
  let printed = without_non_printing_arguments(address.unlist_ref());
  let v = printed.as_slice();
  // (an escaped underscore prints one, and parts the local part's words: `ricky\_ang@`, 2604.02636 — [`spelled_owners`])
  let text = |v: &[Token]| -> String {
    v.iter()
      .filter_map(|t| match t.code {
        Catcode::LETTER | Catcode::OTHER => Some(t.to_string()),
        _ if *t == T_CS!("\\_") || *t == T_CS!("\\textunderscore") => Some("_".to_string()),
        _ => None,
      })
      .collect()
  };
  // `\href{mailto:t}{p}`: the address it prints, not its target and the printed one run together; its target's, past
  // `mailto:`, when it prints none (a placeholder target, `\href{mailto:email@domain}{yuxi.xia@univie.ac.at}`,
  // 2601.07974)
  if let Some(at) = v.iter().position(|t| *t == T_CS!("\\href")) {
    let open = at + 1 + v[at + 1..].iter().take_while(|t| **t == T_SPACE!()).count();
    let close = skip_group(v, open).min(v.len());
    let target = text(&v[open..close]);
    let printed = text(&v[..at]) + &text(&v[close..]);
    return if printed.contains('@') {
      printed
    } else {
      target.trim_start_matches("mailto:").to_string()
    };
  }
  text(v)
}

/// Every name (index into `name_keys`) whose surname the local part of `address` spells ([`spelled_owners`]).
pub(in crate::base_utilities) fn address_owners(
  name_keys: &[NameKeys],
  address: &Tokens,
) -> Vec<usize> {
  spelled_owners(name_keys, &address_text(address))
}

/// The fewest letters a key spells an address with strictly inside a word of its local part, between letters that are
/// none of the name's initials ([`LocalPart::standing`]) — a weak spelling, which owns only an address no other name
/// spells (brief 107g: 4, chosen on the pop107g A/B, OXIDIZED_DESIGN_DIVERGENCES #970).
const WEAK_KEY_LETTERS: usize = 4;

/// The fewest letters a joined spelling ending a word it does not open spells an address with
/// ([`LocalPart::joined_standing`]) — a weak spelling (brief 107f r2).
const WEAK_JOINED_LETTERS: usize = 5;

/// Whether `text` prints the name `name` whole, a name of two words at least, its words in a row (`insert::
/// link_entry_addresses`: `David Macêdo, Pedro Dreyer, … are with …`, 2102.08716).
pub(in crate::base_utilities) fn text_prints_name(text: &str, name: &str) -> bool {
  let name = name_words(name);
  name.len() >= 2
    && name_words(text)
      .windows(name.len())
      .any(|words| words == name)
}

/// How a key's letters stand among a local part's words ([`LocalPart::standing`]): at full strength, or weakly — a
/// spelling that owns an address only no other name spells, and never decides between names that do ([`spelled_owners`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Standing {
  Weak,
  Full,
}

/// An address's local part as [`spelled_owners`] reads it: its letters, lowercased, and where its words meet (`bounds`,
/// byte offsets into `letters`): its separators — `.` `_` `-`, a digit, any other character —, its start and its end.
struct LocalPart {
  letters: String,
  bounds:  Vec<usize>,
}

impl LocalPart {
  fn of(address: &str) -> Option<LocalPart> {
    let raw = address.split('@').next()?;
    let mut letters = String::new();
    let mut bounds: Vec<usize> = vec![0];
    for c in raw.chars() {
      if c.is_alphabetic() {
        letters.extend(c.to_lowercase());
      } else if bounds.last() != Some(&letters.len()) {
        bounds.push(letters.len());
      }
    }
    if bounds.last() != Some(&letters.len()) {
      bounds.push(letters.len());
    }
    Some(LocalPart { letters, bounds })
  }

  /// Every place `key` stands in the letters, overlapping ones too.
  fn places(&self, key: &str) -> Vec<usize> {
    if key.is_empty() {
      return Vec::new();
    }
    (0..self.letters.len())
      .filter(|&at| self.letters.is_char_boundary(at) && self.letters[at..].starts_with(key))
      .collect()
  }

  /// The word holding the letters at `start..end`, or `None` when they run across a separator.
  fn word_around(&self, start: usize, end: usize) -> Option<(usize, usize)> {
    if self.bounds.iter().any(|&b| start < b && b < end) {
      return None;
    }
    let before = self.bounds.iter().copied().filter(|&b| b <= start).max()?;
    let after = self.bounds.iter().copied().filter(|&b| b >= end).min()?;
    Some((before, after))
  }

  /// THE boundary predicate (brief 107f r2, on review rev107f's Q1): how a name's key at `start..end` stands. At full
  /// strength when it is a whole word, or a word's prefix or suffix (`ricky_ang@` Lay Kee Ang's "ang", `yang91@`,
  /// `kee.ang@`), or strictly inside a word between the name's own initials (`own`), at most two letters on each side
  /// (`rjosem@` José M. Rodríguez-Nogales', 1809.06911; `kmfonsecar@` K. M. Fonseca Romero's, 1202.4210); weakly when a
  /// key of [`WEAK_KEY_LETTERS`] letters or more stands there after two letters at most, whatever follows it (`vemarinc@`
  /// Víctor Marín's, 2311.18024; `tpengqy@` Q. Y. Peng's, 2004.06253; `eetanggz@` Guozhi Tang's, 2106.10681;
  /// `ylwanguestc@` Yinglong Wang's, 2208.05114); else not at all — a shorter key between other letters is a word's
  /// letters (`yvanwy@` no "van", 2202.02002; `byang@` no "yan", 2602.00582; `bangkok@` no "ang"), and no key runs across
  /// a separator (`ricky_ang@` is no Shengyuan A. Yang's "yang", 2604.02636). The sides are counted in characters (review
  /// rev107f finding 5).
  fn standing(&self, start: usize, end: usize, own: &str) -> Option<Standing> {
    let (before, after) = self.word_around(start, end)?;
    if start == before || end == after {
      return Some(Standing::Full);
    }
    let (lead, trail) = (&self.letters[before..start], &self.letters[end..after]);
    if lead.chars().count() > 2 {
      return None;
    }
    let initials = |letters: &str| letters.chars().all(|c| own.contains(c));
    if trail.chars().count() <= 2 && initials(lead) && initials(trail) {
      Some(Standing::Full)
    } else if self.letters[start..end].chars().count() >= WEAK_KEY_LETTERS {
      Some(Standing::Weak)
    } else {
      None
    }
  }

  /// How a joined spelling (`NameKeys::joined`) at `start..end` stands: at full strength opening a word, its separators
  /// read through (`jhuang@`, `j.huang@`, `nlp.wangyi@`); weakly ending a word it does not open, at
  /// [`WEAK_JOINED_LETTERS`] letters or more (`sechenqi@` Qi Chen's "chenqi", 2003.00397; `xjhuang@`'s "jhuang", which is
  /// no tie-break between Jixuan Huang and Xuanjing Huang, 2510.24320 — review rev107f B1; `pyli@` no Yanan Li's "yli",
  /// 2105.07637); else not at all.
  fn joined_standing(&self, start: usize, end: usize) -> Option<Standing> {
    if self.bounds.contains(&start) {
      Some(Standing::Full)
    } else if self.bounds.contains(&end)
      && self.letters[start..end].chars().count() >= WEAK_JOINED_LETTERS
    {
      Some(Standing::Weak)
    } else {
      None
    }
  }
}

/// A name's score for the local part `part`, and its surname part ([`spelled_owners`], which reads the scores): with its
/// weak spellings or without (`weak`, [`Standing`]).
fn spelling_score(keys: &NameKeys, part: &LocalPart, weak: bool) -> (usize, usize) {
  let local = part.letters.as_str();
  let counts = |standing: Option<Standing>| standing.is_some_and(|s| weak || s == Standing::Full);
  // (a surname's first six letters only outside the letters its given name matched (`given`, their place in the local
  // part): `andrea.rossi@` is Andrea Rossi's, not Andrea Andreassen's "andrea" counted twice, `soeren.hansen@` Søren
  // Hansen's, not Søren Sørensen's — review rev107a finding 2). A truncated surname stands as a whole one does: the
  // longest of the surname's first letters standing there, six or more (`mkaufman@` for Kaufmann, `leyaouan@` for Le
  // Yaouanc)
  let surname_score = |key: &SpelledWord, given: Option<(usize, usize)>, own: &str| {
    let len = key.text.len();
    if part
      .places(&key.text)
      .into_iter()
      .any(|at| counts(part.standing(at, at + len, own)))
    {
      return key.text.chars().count();
    }
    if key.letters < 6 {
      return 0;
    }
    let prefix: String = key.text.chars().take(6).collect();
    let outside = |at: usize| match given {
      Some((start, end)) => at + prefix.len() <= start || at >= end,
      None => true,
    };
    let truncated = part.places(&prefix).into_iter().any(|at| {
      let held = local[at..]
        .chars()
        .zip(key.text.chars())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum::<usize>();
      outside(at) && counts(part.standing(at, at + held, own))
    });
    if truncated { 6 } else { 0 }
  };
  // (where the given name stands in the local part: opening it, or anywhere when it has four letters or more)
  let given_place = |given: &SpelledWord, own: &str| {
    let len = given.text.len();
    let fits = |at: usize| counts(part.standing(at, at + len, own));
    if local.starts_with(given.text.as_str()) && fits(0) {
      Some((0, len))
    } else if given.letters >= 4 {
      part
        .places(&given.text)
        .into_iter()
        .find(|&at| fits(at))
        .map(|at| (at, at + len))
    } else {
      None
    }
  };
  // (a joined spelling standing in the local part, its length; the initials, the local part's letters alone)
  // (or its first six letters or more, as a surname's: a truncated family-first spelling, `zhuzhenh22@` for Zhenhao
  // Zhu, `chenyuxi22@` for Yuxiang Chen, 2602.03328, 2609.11716; the r7 re-review's finding 9 — scored the letters it
  // holds less one, below a spelling whole of as many: `wangyu@` is Yu Wang's "wangyu", not Yuxin Wang's "wangyuxin",
  // `liyang@` Yang Li's, not Yangfan Li's; the r8 re-review's finding 3 — where the local part ends there; one going on
  // past a shared start scores its six letters, as before)
  let joined_score = |keys: &NameKeys| {
    keys
      .joined
      .iter()
      .filter_map(|key| {
        if part
          .places(key)
          .into_iter()
          .any(|at| counts(part.joined_standing(at, at + key.len())))
        {
          Some(key.chars().count())
        } else {
          let held = local
            .chars()
            .zip(key.chars())
            .take_while(|(a, b)| a == b)
            .count();
          // (a truncation only when the local part's letters are used up, else its first six letters: `zhangweiwei@`
          // parts from "zhangweiwen" and is Wei Zhang's "zhangwei", the 77y review's finding 5, probe n1)
          if held == local.chars().count() {
            (held >= 6).then(|| held - 1)
          } else {
            (held >= 6).then_some(6)
          }
        }
      })
      .max()
      .unwrap_or(0)
      + keys
        .initials
        .iter()
        .find(|initials| **initials == local)
        .map_or(0, |initials| initials.chars().count())
  };
  // (the score, then its surname part: a tie goes to the surname, `chen@…` Wei Chen's before Chen Li's). A name scores
  // the better of its words spelled apart and a joined spelling, never both added: one spelling of the letters is
  // counted once (`khan.muhammad@ieee.org` scores 12 for Khan Muhammad, "muhammad" and "khan" apart, and 12 for
  // Muhammad Haris Khan, "khanmuhammad" joined, and goes to the surname's spelling, 2602.07835).
  // (a name's score the better of its spellings', its base letters' and its transliterated one's: `mo.schaeffler@`
  // Moritz Schäffler's, html_feedback #7233. A tie between a name spelled literally and one spelled through the
  // transliteration is a tie — an address carries no umlaut, so `mueller@` spells Hans Mueller and Jörg Müller alike
  // and is neither's by its spelling (user ruling 2026-11-05, on review rev107a finding 1); the rest of the placement
  // order decides it)
  keys
    .spellings()
    .map(|keys| {
      // (the initials of the name's words, which a part of it inside a word may stand between: `standing`)
      let own = keys.initials.first().map_or("", String::as_str);
      let given = keys
        .given
        .as_ref()
        .and_then(|given| Some((given, given_place(given, own)?)));
      let place = given.map(|(_, place)| place);
      let surname: usize = keys
        .surnames
        .iter()
        .map(|key| surname_score(key, place, own))
        .sum::<usize>()
        + keys
          .short_surnames
          .iter()
          .find(|short| **short == local)
          .map_or(0, |short| short.chars().count());
      let given = given.map_or(0, |(given, _)| given.text.chars().count());
      ((surname + given).max(joined_score(keys)), surname)
    })
    .max()
    .unwrap_or((0, 0))
}

/// The names (indices into `name_keys`) the local part of `address` spells best: each name scores the letters of its
/// keys the local part holds — a surname whole, or its first six letters (`leyaouan@…` for "A.~Le~Yaouanc"); the given
/// name it opens with, or holds when the given name has four letters or more (2603.24730's
/// `\{yuqihu,~ahna,~vasha\}@berkeley.edu` for Yuqi Hu, Ahna R. Girshick, Vasha DuTell; 2603.20851's
/// `nlp2ct.shanshan@…` for Shanshan Wang, `lidiasc@…` for Lidia S. Chao), or a joined spelling it opens with when
/// that scores better (the better of the two, never their sum: 2602.07835) — and the highest scores own it: a longer
/// spelling outranks a key it holds (`fucheng.fuc@…` is Cheng Fu's, not Jiawei Chen's, 2403.00801; `wangchiew@…`
/// Wang-Chiew Tan's, not Xiaolan Wang's, 2104.01371; `yipengzhang97@…` Yipeng Zhang's, not Xiaoying Zhang's,
/// 2406.06326), a tie going to the surname's spelling. No name when none scores.
///
/// Each key counts only where the one boundary predicate places it ([`LocalPart::standing`], [`LocalPart::joined_standing`];
/// census138 EMAILSPELL, OXIDIZED_DESIGN_DIVERGENCES #970 "107f"). A weak spelling owns an address only no other name
/// spells; where another does, the names are ranked by their full-strength spellings, and a weak one never decides
/// against a full one: the two rankings agreeing, their owner; else every name either ranks first — a tie, which the rest
/// of the placement order (the list's order, the elimination, the block) decides. All of them weak, the one ranking
/// decides. Nothing is guessed (FIDELITY_CHARTER §4): `xjhuang@`
/// beside Jixuan Huang and Xuanjing Huang is a tie (each "huang" ending the word; Jixuan's "jhuang" weak, review rev107f
/// B1, 2510.24320), and so is `xywang@` beside Yi Wang and Xiaoyu Wang, which the elimination gives to Xiaoyu Wang once
/// `yiwang@` is Yi Wang's.
pub(in crate::base_utilities) fn spelled_owners(
  name_keys: &[NameKeys],
  address: &str,
) -> Vec<usize> {
  let Some(part) = LocalPart::of(address) else {
    return Vec::new();
  };
  let scores = |weak: bool| -> Vec<(usize, usize)> {
    name_keys
      .iter()
      .map(|keys| spelling_score(keys, &part, weak))
      .collect()
  };
  let all = scores(true);
  let spelling: Vec<usize> = (0..all.len()).filter(|&at| all[at].0 > 0).collect();
  if spelling.len() < 2 {
    return spelling;
  }
  // (several names spell it: ranked with their weak spellings and without; where a weak one would decide against a full
  // one, a tie. All of them weak, the longer spelling ranks first, with no tie-break by the surname: `sechenqi@` Qi Chen's
  // "chenqi" beside Wei Chen's "chen"; `kkwongar@` no one's beside Hiu Tung Wong and Wong Kam-Kwai, 2608.26715)
  let first = |scores: &[(usize, usize)]| -> Vec<usize> {
    let best = scores.iter().copied().max().unwrap_or((0, 0));
    (0..scores.len()).filter(|&at| scores[at] == best).collect()
  };
  let full = scores(false);
  if full.iter().all(|score| score.0 == 0) {
    let best = all.iter().map(|score| score.0).max().unwrap_or(0);
    return (0..all.len()).filter(|&at| all[at].0 == best).collect();
  }
  let mut owners = first(&all);
  for at in first(&full) {
    if !owners.contains(&at) {
      owners.push(at);
    }
  }
  owners.sort_unstable();
  owners
}

/// A name's full-strength score for the local part `part` ([`spelling_score`], no weak spelling counted).
fn full_score(part: &LocalPart, keys: &NameKeys) -> usize { spelling_score(keys, part, false).0 }

/// Whether an author's own call keeps `address` from `owner`, the name spelling it best (FIDELITY_CHARTER §4 #420): it
/// moves only if the owner's full-strength score ([`spelling_score`], no weak spelling counted) is strictly higher than
/// the holder's (`holder`), so a tie never moves an own call — amsart's `\author{Yang Zhang}…\email{yang91@…}` beside
/// Hengyun Yang, 1707.09730; `minhnnhat@` under Nhat Ho beside Anh-Minh Phan, 2511.18493; `\author{Lay Kee
/// Ang}\email{ricky\_ang@…}`, 2604.02636 — and a longer spelling does, whichever part of it the holder's key is:
/// `sanderson@` on Mark Anderson's call is Lee Sanderson's, `anderson@` on Peter Andersen's is Maria Anderson's (review
/// rev107f finding 3), `angela.rossi@` on Lay Kee Ang's is Angela Rossi's (rev107f2 finding 1), `dongsu.kim@` on Jang
/// Soo Kim's is Dongsu Kim's (0902.4311), `giordano.pola@` on Gianni Pola's Giordano's (0907.3301), `zx.liang55@` on Yi
/// Liang's Zhu-Xing's (1008.3764), `filippo.neri.email@` on the misread "Italy. email"'s Filippo Neri's (2107.06981).
/// So a weak spelling moves no own call (`yhsheng@` on Husheng Yu's stays, no Xiangmin Shen's "shen", 2411.01273;
/// `zhonggong.hc@` on Chao Huang's, no Wang Hong's "hong", 2609.11148) — but for an address set after the last author
/// (`last`), which no name's position claims: it is the name's that spells it, the holder's not standing there even weakly
/// (`tpengqy@` after Y. J. Shang, Q. Y. Peng's, 2004.06253; #420's count, 22 of 24 addresses set after the last author his;
/// `kkwongar@` stays on Wong Kam-Kwai beside Hiu Tung Wong, 2608.26715). Read by
/// `insert::respell_emails` and the observer's `Resolver::respell_emails`, which else move it to the name spelling it
/// better.
pub(in crate::base_utilities) fn own_call_keeps(
  holder: &NameKeys,
  owner: &NameKeys,
  address: &str,
  last: bool,
) -> bool {
  let Some(part) = LocalPart::of(address) else {
    return false;
  };
  full_score(&part, owner) <= full_score(&part, holder)
    && !(last && spelling_score(holder, &part, true).0 == 0)
}

/// The owner (index into `name_keys`) of each of a line's `addresses`: the one name it spells ([`spelled_owner`]); and,
/// by elimination, an address no name spells goes to a name no address spells — on this line, or on another e-mail line
/// of the block (`taken`, by index; 2605.06223's `kyle.min@…` on the next line leaves `treejhk@…` Junhyuk Kwon's) — only
/// where the order is confirmed (user decision 2026-10-20, batch 77p r4):
///  - exactly one address and one name are left (`\{lzhang,zhangw,smith\}@…` under Wei Zhang, Li Zhang, Bob Smith:
///    "zhangw" Wei Zhang's once "lzhang" is Li Zhang's); or
///  - the addresses the names spell stand in name order (one at least), and the addresses left are as many as the names
///    left (2605.26032's `\{jzxchen,chenzhuo,archerdw,gore,congyued,billf,soljacic\}@mit.edu`), or those between two
///    of them as many as the names between their owners (never a run before the first or after the last, which no
///    spelled address bounds: user ruling R10 2026-10-20, 2509.10377), every one of those
///    names counted, spelled on another line or not, and none of them spelled or given another address (`\{aszlam,jase,
///    robfergus\}@fb.com`: "jase" to Jason Weston, 1503.08895; 2603.24730's `j8675309@mit.edu` to Jennifer E. Corbett;
///    2303.11225's `xuta@…`, `musclewu@…` between `tianyuhe@…`, `tabaltru@…`, `runnan.li@…`).
///
/// A name left over is no one's to give while another e-mail line of the block holds an address no name spells
/// (`elsewhere_open`): that address may be the name's, so neither the one-left nor the as-many-left rule places one
/// (the r5 re-review's q6, `\{aable,q77,ddorn\}@x.edu \\ \{bbaker,z99\}@y.edu`: `q77@` is not Cat Cole's;
/// 2602.23777's `dongsli@` on the next line). The run between two spelled owners is read by the order alone, every
/// author between them counted, spelled on another line or not: 2602.23777's `nnwang@` between `xu_zhipeng@` and
/// `dcheng@` has Zilong Wang, Xinyang Jiang and Dongsheng Li between Zhipeng Xu and De Cheng, so it goes to none of
/// them; 2303.11225's `xuta@` has Xu Tan alone between Tianyu He and Tadas Baltrušaitis, as its PDF prints the list
/// in the authors' order (and its `\thanks` names `xuta@microsoft.com` Xu Tan's), though `ztk21@` on the line above
/// spells no one (batch 77p r6).
///
/// Not in order — a list by institution (2509.10377), or none spelled (`\{bb,aa\}@x.edu` reads by the initials, not
/// the order) — the rest stay `None`, as does a role mailbox (RFC 2142's `info@`, `support@`, …, [`is_role_mailbox`]),
/// which is no person's: the block's, on its shared creator (user ruling R10 2026-10-20).
pub(in crate::base_utilities) fn owners_by_elimination(
  name_keys: &[NameKeys],
  addresses: &[String],
  taken: &[bool],
  elsewhere_open: bool,
) -> Vec<Option<usize>> {
  let mut owners: Vec<Option<usize>> = addresses
    .iter()
    .map(|address| spelled_owner(name_keys, address))
    .collect();
  let free = |name: usize| !taken.get(name).copied().unwrap_or(false);
  let left: Vec<usize> = (0..name_keys.len())
    .filter(|name| !owners.contains(&Some(*name)) && free(*name))
    .collect();
  let open: Vec<usize> = (0..owners.len())
    .filter(|at| owners[*at].is_none() && !is_role_mailbox(&addresses[*at]))
    .collect();
  if let ([at], [name]) = (open.as_slice(), left.as_slice())
    && !elsewhere_open
  {
    owners[*at] = Some(*name);
    return owners;
  }
  let spelled: Vec<usize> = owners.iter().flatten().copied().collect();
  if open.is_empty() || spelled.is_empty() || !spelled.windows(2).all(|pair| pair[0] < pair[1]) {
    return owners;
  }
  // (as many left as there are names left: in order, 2605.26032's `jzxchen@`, `billf@`, `soljacic@`)
  if open.len() == left.len() && !elsewhere_open {
    for (at, name) in open.into_iter().zip(left) {
      owners[at] = Some(name);
    }
    return owners;
  }
  // (each run of open addresses, with the owners of the spelled addresses around it, and every name between them)
  let mut run: Vec<usize> = Vec::new();
  let mut below: Option<usize> = None;
  for at in 0..=owners.len() {
    let bound = if at == owners.len() {
      Some(name_keys.len())
    } else {
      owners[at]
    };
    match bound {
      Some(above) => {
        let between = below.map_or(0, |below| below + 1)..above;
        // (a run before the first spelled owner or after the last is bounded by the list's end alone, which no spelled
        // address confirms: no position placement at a list's end (user ruling R10 2026-10-20; 2509.10377's `12421181@`
        // first in its list, the r5 re-review's q7 `z99@`); a run between two spelled owners is read by the order,
        // 2303.11225)
        let at_an_end = below.is_none() || at == owners.len();
        if !run.is_empty() && between.len() == run.len() && between.clone().all(free) && !at_an_end
        {
          for (address, name) in run.iter().zip(between) {
            owners[*address] = Some(name);
          }
        }
        run.clear();
        below = Some(above);
      },
      None if open.contains(&at) => run.push(at),
      None => {},
    }
  }
  owners
}

/// How many of a line's `addresses` no name spells ([`spelled_owner`]), a role mailbox aside: the addresses that keep
/// the names of the other e-mail lines of the block from going by elimination ([`owners_by_elimination`]).
pub(in crate::base_utilities) fn unspelled_addresses(
  name_keys: &[NameKeys],
  addresses: &[String],
) -> usize {
  addresses
    .iter()
    .filter(|address| spelled_owner(name_keys, address).is_none() && !is_role_mailbox(address))
    .count()
}

/// Whether `line` prints nothing: spacing (`leading_spacing_end`, `\vspace{2pt}`), a paragraph break, and the groups, font and size
/// commands around them (`\large{~~}`, 2609.05430); a strut, an alignment declaration, an environment's `\begin{…}` or
/// `\end{…}` itself (`\end{@IEEEauthorhalign}\hfill\strut`, 2309.15562's IEEE block, whose environment name had read as an
/// address, then as an empty affiliation).
pub fn prints_nothing(line: &Tokens) -> bool {
  let v = line.unlist_ref();
  let mut i = 0;
  while i < v.len() {
    let t = &v[i];
    if matches!(
      t.get_catcode(),
      Catcode::BEGIN | Catcode::END | Catcode::SPACE
    ) || *t == T_CS!("\\par")
      || is_spacing_command(t)
      || (t.get_catcode() == Catcode::CS
        && t.with_str(|s| {
          NAME_TEXT_COMMANDS.contains(&s)
            || matches!(
              s,
              "\\strut" | "\\centering" | "\\raggedright" | "\\raggedleft"
            )
        }))
    {
      i += 1;
    } else if *t == T_CS!("\\begin") || *t == T_CS!("\\end") {
      i = skip_group(v, i + 1).max(i + 1);
    } else if *t == T_CS!("\\vspace") || *t == T_CS!("\\hspace") {
      // (its length, after a star)
      let mut at = i + 1;
      while v
        .get(at)
        .is_some_and(|u| *u == T_SPACE!() || *u == T_OTHER!("*"))
      {
        at += 1;
      }
      i = skip_group(v, at);
    } else {
      match leading_spacing_end(&v[i..]) {
        0 => return false,
        skipped => i += skipped,
      }
    }
  }
  true
}

/// Whether an e-mail address is a pattern for the authors' addresses, not one of them: its local part is only the words
/// of a name template (`FirstName.LastName@SIT.Fraunhofer.de`, `{firstname.lastname}@…`, `name.surname@…`; 2006.12418;
/// `forename.surname@tu-dortmund.de`, 2410.09681, review rev106a S2).
pub(in crate::base_utilities) fn is_pattern_address(address: &str) -> bool {
  let Some((local, _)) = address.split_once('@') else {
    return false;
  };
  let words: Vec<String> = local
    .split(|c: char| !c.is_alphanumeric())
    .filter(|w| !w.is_empty())
    .map(str::to_lowercase)
    .collect();
  !words.is_empty()
    && words.iter().all(|w| {
      matches!(
        w.as_str(),
        "first"
          | "last"
          | "firstname"
          | "lastname"
          | "name"
          | "surname"
          | "givenname"
          | "familyname"
          | "given"
          | "family"
          | "initial"
          | "initials"
          | "fname"
          | "lname"
          | "forename"
          | "forenames"
          | "firstnames"
          | "givennames"
          | "middle"
          | "middlename"
          | "lastnames"
          | "surnames"
          | "familynames"
          | "vorname"
          | "nachname"
          | "prenom"
      )
    })
}

/// Whether `address` is a mailbox its entry names, no one's surname: its local part two letters alone (digits aside,
/// as a short surname spells one, [`NameKeys`]) that are a word the entry prints beside its addresses (`ai@x.org` under
/// "AI Lab, Univ X" and Wei Ai, review rev100b S3). A state code after a comma is no such word: "Cambridge, MA 02138"
/// leaves Jun Ma's `ma@` his (review rev100b2 S3). The Link step reads it as spelling no one.
pub(in crate::base_utilities) fn is_entry_word_mailbox(address: &str, entry: &str) -> bool {
  let local: String = address
    .split('@')
    .next()
    .unwrap_or_default()
    .chars()
    .filter(|c| c.is_alphabetic())
    .flat_map(char::to_lowercase)
    .collect();
  if local.chars().count() != 2 {
    return false;
  }
  // (each word of letters, with whether a comma is the last mark before it)
  let mut words: Vec<(String, bool)> = Vec::new();
  let mut after_comma = false;
  for raw in entry.split_whitespace().filter(|word| !word.contains('@')) {
    let mut word = String::new();
    for c in raw.chars() {
      if c.is_alphabetic() {
        word.push(c);
        continue;
      }
      if !word.is_empty() {
        words.push((std::mem::take(&mut word), after_comma));
      }
      after_comma = c == ',';
    }
    if !word.is_empty() {
      words.push((word, after_comma));
      after_comma = false;
    }
  }
  words.iter().any(|(word, after_comma)| {
    let state_code = *after_comma && word.chars().all(|c| c.is_uppercase());
    word.to_lowercase() == local && !state_code
  })
}

/// The owners FIDELITY_CHARTER §4's chain gives the e-mail addresses printed inside an affiliation-kind entry (an
/// affiliation, alternate affiliation or address: its lines of addresses, its e-mail calls), the one Link step both
/// engines run — the author block's at the flush (`insert::link_entry_addresses`) and the frontmatter observer's
/// reading (`Resolver::take_out_entry_addresses`, `Resolver::answer_inner_emails`). `spelled` is each address of the
/// entry's the author it spells (any live author: [`spelled_owner`]), `run` the addresses read here (one run of them,
/// in print order), `carriers` the authors holding the entry, in order, and `symbol_line` whether a footnote symbol
/// leads the entry, `addressed` the carriers holding an address of their own beside it, and `sole_author` whether the
/// document has one author, whose every address is (the charter's "the group's only author"). Each address of `run`, in
/// order: the author it spells (`owner:spelled`); else the entry's only carrier, when it is the entry's one address no
/// one spells, no other address of the entry spells that carrier and the carrier holds none of its own
/// (`owner:only-carrier`: `{steen, sund}@…` under Steen alone is two people's); else, on a symbol-led line, every carrier
/// no other address of the entry spells (`owner:carriers`, the charter's "every symbol
/// carrier", 2026-10-30 — before the order, so `$^\dagger$alpha@x.org, beta@y.org` under two carriers of `\dagger` is
/// both carriers'); else, the run's addresses one per carrier that every spelled one agrees with, the carrier at its
/// place (`owner:order`, OD #805); else `None`, the block's, once (`owner:block`: ruling E, main session 2026-11-02 —
/// the charter's step 4 wins over the "stays in the entry" fail-safe of OD #938; astro-ph0512555's `Email:dzg@,
/// xywang@, hyf@` under four carriers of entry 2). Surname-first initials (`dzg` for Z. G. Dai) are not read: a false
/// match is worse than the block.
pub fn entry_address_owners(
  spelled: &[Option<usize>],
  run: std::ops::Range<usize>,
  carriers: &[usize],
  symbol_line: bool,
  addressed: &[usize],
  sole_author: bool,
) -> Vec<(Option<Vec<usize>>, &'static str)> {
  // (the only carrier's only where the entry prints that one address no one spells and the carrier holds none of its own:
  // several unspelled under one name are no one name's — 2403.17706's `1022041122@, 1223045440@, hhp@` under Haiping
  // Huang's second institute, 2506.06908's welded entry under Chenzhi Xu —, nor is one under a name already holding its
  // own — 2411.17073's `bull@` under Awais Naeem, `awais.naeem@` his)
  let unspelled = spelled.iter().filter(|s| s.is_none()).count();
  let in_order = run.len() == carriers.len()
    && carriers.len() >= 2
    && spelled[run.clone()]
      .iter()
      .zip(carriers)
      .all(|(s, c)| s.is_none_or(|s| s == *c));
  // (the symbol carriers no other address of the entry spells, as the only-carrier step excludes them: 1907.01388's
  // `* chinarentao@163.com, zhutou@ustc.edu` under Tao Ren* and Tao Zhou* gives `zhutou@` to Zhou alone, review rev100b
  // S1; none left, the order and the block decide)
  let free: Vec<usize> = carriers
    .iter()
    .copied()
    .filter(|c| !spelled.contains(&Some(*c)))
    .collect();
  run
    .clone()
    .map(|at| match (spelled[at], carriers) {
      (Some(owner), _) => (Some(vec![owner]), "owner:spelled"),
      (None, [only])
        if sole_author
          || (!spelled.contains(&Some(*only)) && unspelled == 1 && !addressed.contains(only)) =>
      {
        (Some(vec![*only]), "owner:only-carrier")
      },
      (None, [] | [_]) => (None, "owner:block"),
      (None, _) if symbol_line && !free.is_empty() => (Some(free.clone()), "owner:carriers"),
      (None, _) if in_order => (Some(vec![carriers[at - run.start]]), "owner:order"),
      (None, _) => (None, "owner:block"),
    })
    .collect()
}

/// The owners of the addresses of a `{…}@domain` list set over several lines, which the Kind step for an address line
/// took out of an affiliation-kind entry (`insert::link_entry_addresses`, the list's `LINE_LIST`), placed as the author
/// block's reader places the list on one line (FIDELITY_CHARTER §4's order, review rev107a finding 4; html_feedback
/// #7233): `name_keys` every author's, in order; `addresses` the list's, in print order; `spelled` each one's owner as the
/// Link step reads it ([`spelled_owner`], an entry's word aside: [`is_entry_word_mailbox`]); `taken` the authors an
/// address outside the list spells or who hold one; `spelled_elsewhere` whether an address outside the list spells an
/// author; `elsewhere_open` whether one outside it spells no one and is placed on no author. Each address: the author it
/// spells (`owner:spelled`); else, the list one address per author that every spelled one agrees with and no address
/// outside it spells an author, the author at its place (`owner:order`, OXIDIZED_DESIGN_DIVERGENCES #805 — the
/// synthetic `email_list_split_order_only`); else the only author's, where there is one (`owner:only-author`, the
/// charter's "the group's only author", as [`entry_address_owners`]' `sole_author` — 2311.03226); else by elimination
/// (`owner:elimination`, batch 77p r4, [`owners_by_elimination`] — 1808.00588's `kdm@`; 2305.13191's `wshui@`, `drot@`,
/// `ballemig@`, 9 entries for 10 names); else `None`, the block's
/// (`owner:block`, OD #159's block creator).
pub(in crate::base_utilities) fn list_owners(
  name_keys: &[NameKeys],
  addresses: &[String],
  spelled: &[Option<usize>],
  taken: &[bool],
  spelled_elsewhere: bool,
  elsewhere_open: bool,
) -> Vec<(Option<usize>, &'static str)> {
  let in_order = addresses.len() == name_keys.len()
    && name_keys.len() >= 2
    && !spelled_elsewhere
    && spelled
      .iter()
      .enumerate()
      .all(|(at, owner)| owner.is_none_or(|owner| owner == at));
  if in_order {
    return spelled
      .iter()
      .enumerate()
      .map(|(at, owner)| match owner {
        Some(owner) => (Some(*owner), "owner:spelled"),
        None => (Some(at), "owner:order"),
      })
      .collect();
  }
  if let [_] = name_keys {
    return spelled
      .iter()
      .map(|owner| match owner {
        Some(owner) => (Some(*owner), "owner:spelled"),
        None => (Some(0), "owner:only-author"),
      })
      .collect();
  }
  owners_by_elimination(name_keys, addresses, taken, elsewhere_open)
    .into_iter()
    .zip(spelled)
    .zip(addresses)
    .map(|((owner, spelled), address)| match (spelled, owner) {
      (Some(owner), _) => (Some(*owner), "owner:spelled"),
      // (an address the elimination reads as spelled that the Link step does not — an entry's word — is the block's)
      (None, Some(owner)) if spelled_owner(name_keys, address).is_none() => {
        (Some(owner), "owner:elimination")
      },
      _ => (None, "owner:block"),
    })
    .collect()
}
