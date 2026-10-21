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
    } else if t.code == Catcode::SPACE {
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
      return Ok(Some((
        label,
        items
          .into_iter()
          .map(|address| EmailAddress { address, call: None })
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

/// One address of an email line, and the class's email command that gave it (`\email{x}`), if any.
pub(in crate::base_utilities) struct EmailAddress {
  pub(in crate::base_utilities) address: Tokens,
  pub(in crate::base_utilities) call:    Option<Tokens>,
}

/// The words that can stand for a person in an email address ([`spelled_owners`]), lowercased, of the name as printed,
/// its notes aside (1706.03762's `Ashish Vaswani\thanks{… Noam proposed …}` is not "noam").
#[derive(Clone, Debug, Default)]
pub(in crate::base_utilities) struct NameKeys {
  /// the name's words after the first that have three letters or more (`Ph.~Boucaud` "boucaud",
  /// `J.~Rodr\'iguez-Quintero` "rodriguez", "quintero")
  surnames: Vec<String>,
  /// its first word, when it has three letters or more (`Ahna R. Girshick` "ahna"; not the initial of `J.~Smith`)
  given: Option<String>,
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
}

impl NameKeys {
  pub(in crate::base_utilities) fn of(name: &Tokens) -> NameKeys {
    let text = visible_name_text(name.unlist_ref());
    let words: Vec<String> = text
      .split(|c: char| !c.is_alphabetic())
      .filter(|word| !word.is_empty())
      .map(str::to_lowercase)
      .collect();
    let Some((first, later)) = words.split_first() else {
      return NameKeys::default();
    };
    let initial: String = first.chars().take(1).collect();
    let mut joined: Vec<String> = Vec::new();
    for word in later.iter().filter(|word| word.chars().count() >= 2) {
      joined.push(format!("{initial}{word}"));
      if first.chars().count() >= 2 {
        joined.push(format!("{word}{first}"));
        joined.push(format!("{first}{word}"));
      }
    }
    // (the initials of every word before the last, then the last: `Kai-Wei Chang` "kwchang", 2604.01354)
    if let Some((last, before)) = words.split_last()
      && before.len() >= 2
      && last.chars().count() >= 2
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
        .filter(|word| word.chars().count() >= 3)
        .cloned()
        .collect(),
      given: Some(first.clone()).filter(|word| word.chars().count() >= 3),
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
    }
  }
}

/// An address's letters and other characters, as [`spelled_owners`] reads them.
pub(in crate::base_utilities) fn address_text(address: &Tokens) -> String {
  // (a colour, a font, a scale is no letter of it, [`without_non_printing_arguments`]: 2604.19440, 2609.25669)
  let printed = without_non_printing_arguments(address.unlist_ref());
  let v = printed.as_slice();
  let text = |v: &[Token]| -> String {
    v.iter()
      .filter(|t| matches!(t.code, Catcode::LETTER | Catcode::OTHER))
      .map(|t| t.to_string())
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

/// The names (indices into `name_keys`) the local part of `address` spells best: each name scores the letters of its
/// keys the local part holds — a surname whole, or its first six letters (`leyaouan@…` for "A.~Le~Yaouanc"); the given
/// name it opens with, or holds when the given name has four letters or more (2603.24730's
/// `\{yuqihu,~ahna,~vasha\}@berkeley.edu` for Yuqi Hu, Ahna R. Girshick, Vasha DuTell; 2603.20851's
/// `nlp2ct.shanshan@…` for Shanshan Wang, `lidiasc@…` for Lidia S. Chao), or a joined spelling it opens with when
/// that scores better (the better of the two, never their sum: 2602.07835) — and the highest scores own it: a longer
/// spelling outranks a key it holds (`fucheng.fuc@…` is Cheng Fu's, not Jiawei Chen's, 2403.00801; `wangchiew@…`
/// Wang-Chiew Tan's, not Xiaolan Wang's, 2104.01371; `yipengzhang97@…` Yipeng Zhang's, not Xiaoying Zhang's,
/// 2406.06326), a tie going to the surname's spelling. No name when none scores.
pub(in crate::base_utilities) fn spelled_owners(
  name_keys: &[NameKeys],
  address: &str,
) -> Vec<usize> {
  let Some(local) = address.split('@').next() else {
    return Vec::new();
  };
  let local: String = local
    .chars()
    .filter(|c| c.is_alphabetic())
    .collect::<String>()
    .to_lowercase();
  let surname_score = |key: &String| {
    if local.contains(key.as_str()) {
      key.chars().count()
    } else if key.chars().count() >= 6 && local.contains(&key.chars().take(6).collect::<String>()) {
      6
    } else {
      0
    }
  };
  let given_score = |given: &String| {
    let length = given.chars().count();
    if local.starts_with(given.as_str()) || (length >= 4 && local.contains(given.as_str())) {
      length
    } else {
      0
    }
  };
  // (a joined spelling the local part opens with, its length; the initials, the local part's letters alone)
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
        if local.starts_with(key.as_str()) {
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
  let scores: Vec<(usize, usize)> = name_keys
    .iter()
    .map(|keys| {
      let surname: usize = keys.surnames.iter().map(surname_score).sum();
      (
        (surname + keys.given.as_ref().map_or(0, given_score)).max(joined_score(keys)),
        surname,
      )
    })
    .collect();
  let best = scores.iter().copied().max().unwrap_or((0, 0));
  if best.0 == 0 {
    return Vec::new();
  }
  (0..scores.len()).filter(|at| scores[*at] == best).collect()
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

/// Whether `line` prints nothing: spacing ([`leading_spacing_end`], `\vspace{2pt}`), a paragraph break, and the groups, font and size
/// commands around them (`\large{~~}`, 2609.05430); a strut, an alignment declaration, an environment's `\begin{…}` or
/// `\end{…}` itself (`\end{@IEEEauthorhalign}\hfill\strut`, 2309.15562's IEEE block, whose environment name had read as an
/// address, then as an empty affiliation).
pub(in crate::base_utilities) fn prints_nothing(line: &Tokens) -> bool {
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
