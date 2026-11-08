//! The visible text of a name: name words, particles, qualifiers, name groups and merged creators.

use super::*;

/// Whether a name's last word is one its period belongs to: an initial (a single capital, `K`) or a suffix written with
/// a period (`Jr`, `Esq`, [`NAME_SUFFIXES`]). A two-letter word is a surname as often as an initial (`Wei Li.`, `Andrew
/// Ng.`: sentence punctuation), so its period goes, as Perl's.
pub(in crate::base_utilities) fn ends_with_abbreviation(name: &str) -> bool {
  let Some(word) = name.split_whitespace().next_back() else {
    return false;
  };
  let mut chars = word.chars();
  let initial = chars.next().is_some_and(char::is_uppercase) && chars.next().is_none();
  // (and "et al." ending a list, `is_name_qualifier`)
  initial || NAME_SUFFIXES.contains(&format!("{word}.").as_str()) || word == "al" || word == "et.al"
}

/// Whether `text` is a professional membership grade as an author writes it after the name and its comma: the words of
/// one ([`grade_words`]: `Fellow, IEEE`, `Senior Member, IEEE`, `Member, OSA`), or an abbreviated grade before its
/// society (`M.ASCE`, `A. M. ASCE`; [`NAME_GRADE_ABBREVIATIONS`]).
pub(in crate::base_utilities) fn is_membership_grade(text: &str) -> bool {
  let text = strip_glued_marks(text);
  let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
  grade_words(text, true)
    || NAME_GRADE_ABBREVIATIONS.iter().any(|grade| {
      compact
        .strip_prefix(grade)
        .is_some_and(|society| NAME_GRADE_SOCIETIES.contains(&society))
    })
}

/// A name with the membership grade written after its comma taken out as the author's `membership` contact: `Cat
/// Cole\textsuperscript{1}, \textit{Fellow, IEEE}` is "Cat Cole" and the contact "Fellow, IEEE" in its italics, the
/// contact IEEEtran's `\IEEEmembership` makes of the grade it prints (ieeetran_cls.rs `\lx@IEEE@membership`;
/// 2107.05093, 2302.11403, 2602.16174), and so are `Bob Baker, Member, OSA` and ascelike's `Matthew R. Kuhn, M.ASCE`
/// (ascexmpl.tex:48). The grade is the text after the first comma of the name's own (outside its groups and math) after
/// which only a grade is printed ([`is_membership_grade`]); the marks and notes written after it stay the name's, after
/// the contact, as they print after the grade. `None` when the name ends in no such grade (`John Smith, Jr.`, `Jane
/// Doe, MD, PhD` qualify the name).
pub(in crate::base_utilities) fn written_membership_out(name: &[Token]) -> Option<Vec<Token>> {
  let (mut depth, mut in_math, mut found) = (0, false, None);
  let commas: Vec<usize> = (name.iter().enumerate())
    .filter_map(|(c, t)| {
      depth += t.group_step();
      in_math ^= t.get_catcode() == Catcode::MATH;
      (depth == 0 && !in_math && *t == T_OTHER!(",")).then_some(c)
    })
    .collect();
  // (a grade is a suffix of grade words, so its comma is read from the last leftward, the first tail holding a word no grade
  // has ending the reading: every tail before it holds that word too. The first comma after an alphabetic head opening a
  // grade is the one read from the first comma on, each tail read once — not the rest of the name per comma, quadratic on
  // Perl's whole-block creator of a block of 418 names, 1507.02068)
  for c in commas.into_iter().rev() {
    let (grade, annotations) = grade_and_annotations(&name[c + 1..]);
    let text = visible_name_text(&grade);
    if is_membership_grade(&text)
      && visible_name_text(&name[..c])
        .chars()
        .any(char::is_alphabetic)
    {
      found = Some((c, grade, annotations));
    } else if text.chars().any(char::is_alphanumeric)
      && !grade_words(strip_glued_marks(&text), false)
    {
      break;
    }
  }
  // (one grade, the line's last: a names line printing a grade before it holds a name after a grade, `Ann Able,
  // \textit{Member, IEEE}; Bob Baker, \textit{Fellow, IEEE}`, whose grades all stay — taking out the last alone would
  // leave the names before it one creator: 1510.03162, 2507.19818)
  let (c, grade, annotations) = found.filter(|(c, ..)| !holds_grade(&name[..*c]))?;
  let head = &name[..c];
  let end = head
    .iter()
    .rposition(|t| !is_spacing(t))
    .map_or(0, |k| k + 1);
  let mut out = head[..end].to_vec();
  out.extend(TokenizeInternal!(r"\lx@add@contact[role=membership]").unlist());
  out.push(T_BEGIN!());
  out.extend(grade);
  out.push(T_END!());
  out.extend(annotations);
  Some(out)
}

/// Whether a names line prints a membership grade or a part of one ([`is_membership_part`]) between its names'
/// separators — a comma, a semicolon, an "and".
fn holds_grade(line: &[Token]) -> bool {
  visible_name_text(line)
    .replace('~', " ")
    .split([',', ';'])
    .flat_map(|piece| piece.split(" and "))
    .map(str::trim)
    .any(|piece| !piece.is_empty() && is_membership_part(piece))
}

/// Where the name line a creator's content holds begins and ends: inside the block reader's `\lx@author@withsup{…}` and
/// a command the name line is set in as its one argument (a raw IEEE class's `\IEEEauthorblockN{…}`, which the kernel's
/// split puts round each name: icrc2009, 0907.0548, 1111.2718), each holding the whole content; the whole content
/// otherwise.
pub(in crate::base_utilities) fn name_line_span(v: &[Token]) -> (usize, usize) {
  let (mut start, mut end) = (0, v.len());
  for _ in 0..2 {
    while end > start && v[end - 1] == T_SPACE!() {
      end -= 1;
    }
    if end - start < 3 {
      break;
    }
    let cs = v[start];
    if cs.get_catcode() != Catcode::CS
      || v[start + 1].get_catcode() != Catcode::BEGIN
      || skip_group(&v[..end], start + 1) != end
      || (cs != T_CS!("\\lx@author@withsup")
        && (is_name_annotation(&cs)
          || is_footnote_mark(&cs)
          || is_author_mark(&cs)
          || cs.with_str(|n| n.starts_with("\\lx@"))))
    {
      break;
    }
    start += 2;
    end -= 1;
  }
  (start, end)
}

/// A creator's content as `\lx@add@author` reads it (frontmatter_fields.rs): the membership grade its name line writes
/// after the name's comma the author's contact ([`written_membership_out`]), inside what the line is set in
/// ([`name_line_span`]) as in a bare name.
pub(in crate::base_utilities) fn membership_read_out(content: Tokens) -> Tokens {
  let v = content.unlist_ref();
  let (start, end) = name_line_span(v);
  match written_membership_out(&v[start..end]) {
    Some(name) => {
      let mut out = v[..start].to_vec();
      out.extend(name);
      out.extend_from_slice(&v[end..]);
      Tokens::new(out)
    },
    None => content,
  }
}

/// What follows a name's comma parted into the text it prints, its spacing trimmed, and the marks and notes written in
/// it (a note or mark command with its arguments, [`is_name_annotation`], [`is_footnote_mark`], [`is_mark_command`]; a
/// formula, `$^{1}$`; a block reader's contact call after the name, `\lx@add@affiliation[annotate=3]{…}`: 1611.04834's
/// `and Warren J. Gross, \emph{Senior Member, IEEE}\\ Department …`).
fn grade_and_annotations(tail: &[Token]) -> (Vec<Token>, Vec<Token>) {
  let kernel_call = |t: &Token| t.with_str(|s| s.starts_with("\\lx@add@"));
  let (mut text, mut annotations, mut i) = (Vec::new(), Vec::new(), 0);
  while i < tail.len() {
    let t = tail[i];
    let start = i;
    i += 1;
    if t.get_catcode() == Catcode::MATH {
      while i < tail.len() && tail[i].get_catcode() != Catcode::MATH {
        i += 1;
      }
      i = (i + 1).min(tail.len());
      annotations.extend_from_slice(&tail[start..i]);
    } else if t.get_catcode() == Catcode::CS
      && (is_name_annotation(&t) || is_footnote_mark(&t) || is_mark_command(&t) || kernel_call(&t))
    {
      if tail.get(i) == Some(&T_OTHER!("["))
        && let Some(close) = tail[i..].iter().position(|u| *u == T_OTHER!("]"))
      {
        i += close + 1;
      }
      if kernel_call(&t) {
        while tail
          .get(i)
          .is_some_and(|u| u.get_catcode() == Catcode::BEGIN)
        {
          i = skip_group(tail, i).min(tail.len());
        }
      } else if !is_footnote_mark(&t) {
        i = skip_group(tail, i).min(tail.len());
      }
      annotations.extend_from_slice(&tail[start..i]);
    } else if t.get_catcode() == Catcode::BEGIN {
      i = skip_group(tail, start).min(tail.len());
      text.extend_from_slice(&tail[start..i]);
    } else {
      text.push(t);
    }
  }
  let from = text
    .iter()
    .position(|t| !is_spacing(t))
    .unwrap_or(text.len());
  let to = text
    .iter()
    .rposition(|t| !is_spacing(t))
    .map_or(from, |k| k + 1);
  (text[from..to].to_vec(), annotations)
}

/// Whether `line` continues the author names `names` instead of starting their affiliations: styled as the names are,
/// all of it names (or their qualifiers), and either the names read as an unfinished list — ending in a comma, "and"
/// or "&" (`Joshua N.\ Winn\altaffilmark{1}, Andrew W.\ Howard\altaffilmark{2},\\ Avi Shporer\altaffilmark{1}`,
/// 1010.1318) — or the line opens with "and" (`…, \emph{Member, IEEE},\\ and Warren J. Gross, \emph{Senior Member,
/// IEEE}`, 1611.04834) — or the line is a comma list of two or more names and nothing else (`…, Wenhan Liu\\ Haonan
/// Chen, Zheng Liu, Zhicheng Dou, and Ji-Rong Wen`, 2308.07107; 2312.11514, 2406.10513). "and Max--Planck Institut für
/// Mathematik, Bonn" (math0208081), `John Smith,\\ {\it Bell Labs, Murray Hill}` and `Pierre Fernandez\\ Meta FAIR \&
/// Inria Rennes` (2402.14904: joined by "&" alone, as institutions are) stay affiliations.
pub fn names_continue(names: &Tokens, line: &Tokens) -> bool {
  leading_markup(line) == leading_markup(names) && names_list_continues(names, line)
}

/// [`names_continue`]'s text test alone, whatever markup either line opens with: in a block whose authors carry marks,
/// a row of names after names continues them however it is styled (`Ann Able$^{1}$, Bob Baker$^{2}$,\\
/// \textbf{Cat Cole}$^{\dagger}$, \textbf{Dan Dee}$^{\dagger}$`, 2307.00040; `\textbf{Cat Cole, Dan Dee,}`, 2510.11639) —
/// its words tell names from places there, as the unmarked block's styled-affiliation alternation needs the markup too
/// (hep-ph9306253, 0811.1526).
pub(in crate::base_utilities) fn names_list_continues(names: &Tokens, line: &Tokens) -> bool {
  // (a separator in a wrapper of its own is the line's: `\textbf{Seth Ebner}$^{\diamondsuit}$\textrm{,}\\ \textbf{Brandon
  // Smock}`, 2512.18601)
  let names_text = visible_name_text(hoist_wrapped_separators(names.clone()).unlist_ref());
  let names_text = names_text.trim_end();
  let text = visible_name_text(line.unlist_ref());
  let text = text.trim();
  let unfinished =
    names_text.ends_with(',') || names_text.ends_with(" and") || names_text.ends_with('&');
  let count = name_count(text);
  ((unfinished || text.starts_with("and ") || text.starts_with("& ")) && count > 0)
    || (count >= 2 && text.contains(',') && name_count(names_text) >= 2)
}

/// Whether `line` is more names under the names `names`, not their affiliation: styled as they are, all of it names
/// (or their qualifiers), and every name of both carrying its mark after it, as authors' marks stand (`Ann
/// Able$^{1}$\\ Bob Baker$^{2}$`; `A\altaffilmark{1}, B\altaffilmark{2}\\ C\altaffilmark{3}`; [`marked_after_name`]).
fn names_marked_alike(names: &Tokens, line: &Tokens) -> bool {
  leading_markup(line) == leading_markup(names)
    && name_count(visible_name_text(line.unlist_ref()).trim()) > 0
    && [names, line].into_iter().all(|tokens| {
      split_author_line(tokens.clone())
        .iter()
        .all(marked_after_name)
    })
}

/// The markup an author line opens with, before its first letter: its control sequences and the groups they open
/// (`{\small\em Bartol …}` → `{`, `\small`, `\em`; `T. Stelzer` → none). Lines styled alike share it.
pub(in crate::base_utilities) fn leading_markup(line: &Tokens) -> Vec<Token> {
  line
    .unlist_ref()
    .iter()
    .take_while(|t| t.get_catcode() != Catcode::LETTER)
    .filter(|t| matches!(t.get_catcode(), Catcode::CS | Catcode::BEGIN))
    .copied()
    .collect()
}

/// The `\\`-lines of an author group without affiliation marks, as (names, affiliations) pairs. The first line is
/// the names and the lines after it their affiliations (Perl Base_Utility.pool.ltxml:720-725), except that
///  - a line that is only "and" (or `\&`) separates two pairs (cond-mat9705101's `\\ \\ and\\ \\`);
///  - a line continues the names while they read as an unfinished list ([`names_continue`]), and a line of names
///    marked as the names above are opens the next pair ([`names_marked_alike`]);
///  - after an affiliation styled unlike the names, a line of names styled as the names were opens the next pair when
///    an affiliation styled as that one follows it — the alternation (hep-ph9306253's `T. Stelzer` between `{\small
///    \em …}` affiliations; 0811.1526's `Ingo Rehberg and Reinhard Richter` between `{\sl …}` ones). A last line
///    (a city: `Los Angeles, CA 90095, USA`) or an affiliation styled as the names ("University of X" under a plain
///    name) stays an affiliation;
///  - after affiliations, a line of names carrying an author's note (`\thanks`, `\footnote`) opens the next pair
///    when the block's first names carry the same note command ([`author_note_of`]; hep-lat9404004's `C.
///    Michael\thanks{…},`);
///  - after affiliations, a line that reads as a name and nothing else opens the next pair when the block's shape
///    says it is the next author's ([`opens_next_author`]: a spacer line before it, its names' styling, the
///    institution under it; 1702.01413, 1908.10448, 1709.07791).
///
/// `gaps` tells, by line, whether a line that prints nothing stood before it (dropped before the lines were handed
/// here: `\\ \\`, `\vspace{0.1cm}\\`, a positive `\\[len]`). A gap before every line under the names line, the
/// first one's included, or before every line from the second under it on, is the block's spacing, parting no authors
/// (`Ann Able \\[2pt] University of X \\[2pt] Hong Kong`, the 85a r2 review's finding 3; `Ann Able \\ University of X
/// \\[2pt] Santa Barbara \\[2pt] California`, the r3 review's finding 2, probe g11).
pub(in crate::base_utilities) fn name_groups(
  lines: Vec<Tokens>,
  gaps: &[bool],
) -> Vec<(Tokens, Vec<Tokens>)> {
  let gaps = if (gaps.len() > 1 && gaps[1..].iter().all(|gap| *gap))
    || (gaps.len() > 3 && gaps[2..].iter().all(|gap| *gap))
  {
    &[]
  } else {
    gaps
  };
  let mut groups: Vec<(Tokens, Vec<Tokens>)> = Vec::new();
  let mut names_markup: Vec<Token> = Vec::new();
  let mut opens_pair = true;
  let mut pending_and: Option<Tokens> = None;
  for (index, line) in lines.iter().enumerate() {
    let text = visible_name_text(line.unlist_ref());
    let text = text.trim();
    if text == "and" || text == "&" {
      // a second "and" in a row: the first is affiliation text (one before any names separates nothing)
      if let (Some(earlier), Some((_, affiliations))) = (pending_and.take(), groups.last_mut()) {
        affiliations.push(earlier);
      }
      pending_and = Some(line.clone());
      continue;
    }
    // A line of only "and" parts two authors when names follow it (cond-mat9705101's `\\ \\ and\\ \\ {\it P.
    // Plech\' a\v{c}}`); before anything else it is affiliation text, kept with the line after it (`Dept.\ of
    // Physics\\ and\\ Center for Theoretical Physics`; `and\\ SISSA, Trieste`), as Perl keeps both.
    if let Some(and_line) = pending_and.take() {
      if groups.is_empty() || name_count(text) > 0 {
        opens_pair = true;
      } else if let Some((_, affiliations)) = groups.last_mut() {
        affiliations.push(and_line);
        affiliations.push(line.clone());
        continue;
      }
    }
    if opens_pair || groups.is_empty() {
      names_markup = leading_markup(line);
      groups.push((line.clone(), Vec::new()));
      opens_pair = false;
      continue;
    }
    // (the author's note the block's first names carry, below)
    let first_names_note = groups.first().and_then(|(first, _)| author_note_of(first));
    // (whether the line starts the next author after the open pair's lines, below)
    let opens_next = opens_next_author(&groups, &lines, index, gaps, &names_markup);
    let (names, affiliations) = groups.last_mut().expect("a pair is open");
    let alternates = affiliations.last().is_some_and(|last| {
      let affiliation_markup = leading_markup(last);
      affiliation_markup != names_markup
        && lines
          .get(index + 1)
          .is_some_and(|next| leading_markup(next) == affiliation_markup)
    });
    let and_led = text
      .strip_prefix("and ")
      .or_else(|| text.strip_prefix("& "));
    // (a line that opens the next pair after affiliations is a names line: a row of names on it, parted by a run of
    // spacing, is parted as the block's first names line's is, [`normalize_hspace_separators`], then by the split,
    // [`space_parted_names`] — `Ann Able \\ Univ A \\ \\ Bob Baker~~~~Cat Cole \\ Univ B`, two creators, not one named
    // "Bob Baker Cat Cole", the 85a r1 review's finding 5; `{\bf Gil Gray}~~~{\bf Hal Hunt}` between `{\it …}` lines)
    if affiliations.is_empty() && names_continue(names, line) {
      let mut joined = names.clone().unlist();
      // a line of names after a finished list goes on after a comma, where the line break parted them
      let names_text = visible_name_text(names.unlist_ref());
      let names_text = names_text.trim_end();
      if !(names_text.ends_with(',')
        || names_text.ends_with(" and")
        || names_text.ends_with('&')
        || and_led.is_some())
      {
        joined.push(T_OTHER!(","));
      }
      joined.push(T_SPACE!());
      // (an "and" opening the line — inside its group too, `{A, B} \\ {and C}`, 2306.02486, 2411.12606 — is the
      // separator, written where the split reads it)
      match and_led.and_then(|_| without_leading_and(line)) {
        Some(rest) => {
          joined.extend([T_LETTER!("a"), T_LETTER!("n"), T_LETTER!("d"), T_SPACE!()]);
          joined.extend(rest.unlist());
        },
        None => joined.extend(line.clone().unlist()),
      }
      *names = Tokens::new(joined);
    } else if affiliations.is_empty() && names_marked_alike(names, line) {
      // the next names, the lines after them their affiliations as after any names
      groups.push((line.clone(), Vec::new()));
    } else if let Some(names_line) = and_led
      .filter(|rest| {
        leading_markup(line) == names_markup && name_count(rest) > 0 && !names_no_person(rest)
      })
      .and_then(|_| without_leading_and(line))
    {
      // after affiliations, "and" before names opens the next author (`James Brink \\Dept of Math., UC Berkeley,
      // …\\and Zhenghan Wang \\Dept of Math., Indiana University`, math-ph0303018), its "and" dropped — not "and" before
      // a field of study, the line above going on (`Department of Electrical \\and Computer Engineering`, 2510.15190)
      groups.push((normalize_hspace_separators(names_line), Vec::new()));
    } else if alternates && leading_markup(line) == names_markup && name_count(text) > 0 {
      groups.push((normalize_hspace_separators(line.clone()), Vec::new()));
    } else if name_count(text) > 0
      && first_names_note.is_some()
      && author_note_of(line) == first_names_note
    {
      // after affiliations, names carrying an author's note open the next pair, however they are styled, when the
      // block's first names carry the same note command — the block's shape (`A.M. Green\thanks{…},\\ Research
      // Institute …\\ C. Michael\thanks{…},\\ DAMTP …`, hep-lat9404004, 1706.02584; `\footnote` throughout, 1410.7277,
      // 1403.1404); an affiliation with a note under plain names stays one (`Ann Able and Bob Baker\\ Department of
      // Physics\\ Ohio State\footnote{Visiting.}`), and so does one with another note command than the names'
      // (`Ann Able\thanks{x}\\ Dept\\ Ohio State\footnote{Visiting}\\ Bob Baker\thanks{y}`)
      groups.push((normalize_hspace_separators(line.clone()), Vec::new()));
    } else if opens_next {
      groups.push((normalize_hspace_separators(line.clone()), Vec::new()));
    } else {
      affiliations.push(line.clone());
    }
  }
  if groups.is_empty() {
    // a block of `\\` alone still gives its (empty) author
    groups.push((Tokens::default(), Vec::new()));
  }
  // a closing "and" line is affiliation text
  if let (Some(and_line), Some((_, affiliations))) = (pending_and, groups.last_mut()) {
    affiliations.push(and_line);
  }
  groups
}

/// Words a line of capitalized words naming no person holds, though [`name_count`] reads it as a name: the title-case
/// common-noun phrases an author block sets on a line of its own — a role or a title (`Chief Scientist`, `Visiting
/// Scholar`, `Associate Dean`, `Vice Provost`), a field of study (`Mathematical Sciences`, `Computational Science`,
/// `Computer Engineering`: 1110.3939, 1404.5443, 2510.15190; `Cognitive Neuroscience`), a facility (`James Clerk
/// Maxwell Building`, math-ph9903017), a document (`Technical Report`, `Lecture Notes`). Such a line is the line of
/// the author above it, not the next author. Local to [`opens_next_author`] and the "and"-led opening of
/// [`name_groups`]: it only keeps a line a line, where [`NON_NAME_WORDS`] decides what [`name_count`] reads as a name
/// everywhere — surnames such as "Dean", "Head" or "Chair" stay names there. Two capitalized words with no person's
/// cue over an institution need no word here ([`person_over_institution`]); a gap or the names' styling before such
/// a phrase does.
const NON_PERSON_WORDS: &[&str] = &[
  "chief",
  "scientist",
  "lecturer",
  "director",
  "engineer",
  "manager",
  "scholar",
  "researcher",
  "investigator",
  "emeritus",
  "editor",
  "president",
  "officer",
  "adjunct",
  "visiting",
  "postdoctoral",
  "postdoc",
  "candidate",
  "chair",
  "chairman",
  "dean",
  "provost",
  "head",
  "lead",
  "architect",
  "consultant",
  "science",
  "sciences",
  "engineering",
  "mathematics",
  "mathematical",
  "computational",
  "computing",
  "computer",
  "statistics",
  "statistical",
  "technology",
  "technologies",
  "technological",
  "technical",
  "robotics",
  "informatics",
  "economics",
  "chemistry",
  "biology",
  "medicine",
  "medical",
  "neuroscience",
  "psychology",
  "astronomy",
  "astrophysics",
  "studies",
  "systems",
  "building",
  "report",
  "lecture",
  "notes",
  "paper",
  "preprint",
  "manuscript",
  "thesis",
  "dissertation",
];

/// Whether `line` prints nothing ([`prints_nothing`]) once its math shifts are left out: spacing, or a formula holding
/// nothing but spacing (`$ $\\`, 1509.03383's spacer between author blocks).
fn spacer_line(line: &Tokens) -> bool {
  prints_nothing(&Tokens::new(
    line
      .unlist_ref()
      .iter()
      .filter(|t| t.get_catcode() != Catcode::MATH)
      .copied()
      .collect(),
  ))
}

/// Whether a text that reads as a name holds a word that names no person ([`NON_PERSON_WORDS`]).
fn names_no_person(text: &str) -> bool {
  text
    .split_whitespace()
    .any(|word| word_forms(word).any(|form| NON_PERSON_WORDS.contains(&form.as_str())))
}

/// The visible text of `line` as a name, without the address it ends with (the JMLR family's `\name Ann Able \email
/// ann@x.edu`, [`name_and_trailing_email`]; an error reading the address is raised where the name is made).
fn name_text_without_address(line: &Tokens) -> String {
  let name = name_and_trailing_email(line).map_or_else(|_| line.clone(), |(name, _)| name);
  visible_name_text(name.unlist_ref()).trim().to_string()
}

/// Whether `line` reads as a name and nothing else: every part a name ([`name_count`]) once the address it ends with
/// is aside ([`name_text_without_address`]: `\name Peter I. Frazier \email pf98@cornell.edu`, 1702.07694), no word
/// naming no person ([`NON_PERSON_WORDS`]), no note ([`classify_thanks`]), no mark leading it, and no e-mail or link
/// line.
fn reads_as_name_alone(line: &Tokens) -> bool {
  let text = name_text_without_address(line);
  name_count(&text) > 0
    && !names_no_person(&text)
    && classify_thanks(&text) == "note"
    && !line_is_email(line)
    && !is_link_line(line)
    && after_symbol_mark(line.unlist_ref()).len() == line.unlist_ref().len()
}

/// Whether `under`, the line under the name line `line`, is an institution of its own under it, telling that `line` is
/// a person's: an institution ([`reads_as_institution`]) that does not go on with `line`'s words
/// ([`continues_the_line`]: `Johns Hopkins \\ University, Baltimore`), under a name carrying a person's cue
/// ([`has_person_cue`]: `Palash B. Pal \\ Saha Institute of Nuclear Physics`, gr-qc9906006) — not two capitalized words
/// alone, a place or a role over an institution (`University of California \\ Santa Barbara \\ Kavli Institute …`,
/// `Associate Dean \\ School of Medicine`, the 85a r1 review's findings 2-4).
pub(in crate::base_utilities) fn person_over_institution(line: &Tokens, under: &Tokens) -> bool {
  reads_as_institution(under)
    && !continues_the_line(line, under)
    && has_person_cue(&name_text_without_address(line))
}

/// Whether `lines[index]`, after the lines under the open pair's names (the last of `groups`), is the next author's
/// name line: it reads as a name and nothing else ([`reads_as_name_alone`]), does not go on with the line above it
/// (that line ending in a connective, `School of Physical and \\ Mathematical Sciences`, 1110.3939, or the line opening
/// with one, `\\ and Computer Engineering`, 2510.15190), and the block's shape says so — with no gap before it, only
/// when the line above is not itself a name styled as it is, a stacked run of names whose lines below are the run's
/// (`\name Stephen N. Pallone \email … \\ \name Peter I. Frazier \email … \\ \name Shane G. Henderson \email … \\\\
/// \addr …`, 1702.07694: Henderson took the run's shared address alone, the 85a r1 review's finding 1), and after a gap
/// only when not every line under the names line down to it reads as names, a name or a row of them, however styled
/// (`Ann A. Able \\ Bob B. Baker \\[6pt] Cat C. Cole`, the r2 review's finding 5a; `\textbf{6 names} \\ \textbf{5
/// names} \\ \\ CUHK MMLab`, 2604.18394, the r3 review's finding 1); else a gap parts the authors whatever the line
/// above reads as (`Comunidad de Palo Blanco, Salamanca Gto. México.\\ \\ V.I. Tkach`, 0801.1297):
///  - a line that prints nothing stands before it (`gaps`, or a line such as `$ $` kept among the lines: `…UK\\
///    \vspace{0.1cm}\\ A. Plastino\\ IFLP-CCT-Conicet`, 1702.01413; `\\ \\ V.I. Tkach`, 0801.1297, 1608.08479,
///    2201.06821; `$ $\\ {\bf Anne Schilling}`, 1509.03383; a positive `\\[12pt]`, gr-qc9906006,
///    [`strip_linebreak_options_keeping_gaps`]; not one that stood before every line under the names line down to it,
///    the block's spacing) — on the block's last line only with a person's cue ([`has_person_cue`]:
///    `\\ \\ Hong Kong` stays a line, the r2 review's finding 3);
///  - it is styled as the names over the open pair are, in markup of their own the line above it is not in
///    (`{\bf Anne Schilling}` under `{\em …}`; `\name Mohammad Rostami` under `\addr …`, 2408.11742), or plain as
///    they are under a line set in a style group of its own, with a line under it (`Eddie Monroe` under `{\it Institute
///    of Noetic Sciences …}`, 1709.07791, the line under him mistyped `I{\it ONS …}`; not a closing `\textit{University
///    of X} \\ Costa Rica`, the r2 review's finding 2);
///  - the line under it is an e-mail address its names spell ([`addresses_spelling`]), or a line styled as the block's
///    styled affiliations are and unlike the name (`David Hanson \\ {\it Hanson Robotics}` under `{\it …}` lines,
///    1709.07791), or an institution of its own under a person's name ([`person_over_institution`]: `James M. Robins \\
///    Department of …`, 1908.10448). With no gap and no styling, the institution under a cue-less name tells nothing:
///    1908.10448's `Zach Shahn \\ IBM Research,` stays a line of Lin Liu (a SYNC_STATUS residual).
///
/// A line that reads as a name with none of these stays the line it is (`Hanson Robotics` over a name, `Hong Kong` over
/// `The Hong Kong University …`, a closing `Costa Rica`), as the names under names with no lines between them do.
fn opens_next_author(
  groups: &[(Tokens, Vec<Tokens>)],
  lines: &[Tokens],
  index: usize,
  gaps: &[bool],
  names_markup: &[Token],
) -> bool {
  let Some((_, affiliations)) = groups.last() else {
    return false;
  };
  let Some(above) = affiliations.iter().rev().find(|a| !spacer_line(a)) else {
    return false;
  };
  let line = &lines[index];
  let text = name_text_without_address(line);
  let above_text = visible_name_text(above.unlist_ref());
  let connective = |word: Option<&str>| {
    word.is_some_and(|word| {
      [
        "and", "&", "of", "for", "the", "at", "in", "de", "di", "du", "des", "und", "et", "e", "y",
      ]
      .contains(&word.to_lowercase().as_str())
    })
  };
  if !reads_as_name_alone(line)
    || connective(text.split_whitespace().next())
    || connective(above_text.split_whitespace().next_back())
  {
    return false;
  }
  // (a gap at every line break of the block down to it, the first one included, is the block's
  // spacing, as one before every line of the block is: `Ann Able \\[3pt] University of X \\[3pt] Santa Barbara \\
  // California`, the 85a r3 review's probe g12)
  let block_spacing = (1..=index).all(|before| gaps.get(before) == Some(&true));
  let gap = (gaps.get(index).copied().unwrap_or(false) && !block_spacing)
    || index
      .checked_sub(1)
      .is_some_and(|before| spacer_line(&lines[before]));
  let markup = leading_markup(line);
  let above_markup = leading_markup(above);
  // (with no gap before it, a name styled as the line above it, which reads as a name too: a stacked run of names,
  // their lines below; after a gap, any line when every line under the names line down to it reads as names, a name
  // or a row of them, however styled — the gap parts rows of names from the lines they share: `Ann A. Able \\ Bob B.
  // Baker \\[6pt] Cat C. Cole \\ Institute of Y, Z`, the 85a r2 review's finding 5a, probe t16; `\textbf{6 names} \\
  // \textbf{5 names\thanks{…}} \\ \\ CUHK MMLab \\ \texttt{…}`, 2604.18394, the r3 review's finding 1, probe k1)
  let reads_as_names = |a: &Tokens| {
    reads_as_name_alone(a) || {
      let pieces = split_author_line(a.clone());
      pieces.len() > 1 && pieces.iter().all(reads_as_name_alone)
    }
  };
  let stacked = if gap {
    affiliations
      .iter()
      .filter(|a| !spacer_line(a))
      .all(reads_as_names)
  } else {
    above_markup == markup && reads_as_name_alone(above)
  };
  if stacked {
    return false;
  }
  // (a line under it: the block's last line, opened by a gap alone, needs a person's cue — `Ann Able \\ University of
  // X \\[2pt] Hong Kong`, `\\ \\ Hong Kong`, the 85a r2 review's finding 3, probes t1, t22, t23)
  let line_under = index + 1 < lines.len();
  let gap = gap && (line_under || has_person_cue(&text));
  // (styled as the names, under a line styled otherwise — in markup of the names' own, or, with a line under it, in a
  // group of the line's own under plain names: `Eddie Monroe` under `{\it Institute of Noetic Sciences …}`,
  // 1709.07791; not the block's closing place, `\textit{University of X} \\ Costa Rica`, the 85a r2 review's finding 2,
  // probes t5, t6; a declaration's style, `\small Dept of Physics`, goes on over the lines after it)
  let styled_as_names = markup == names_markup
    && above_markup != names_markup
    && (!names_markup.is_empty() || (line_under && above_markup.contains(&T_BEGIN!())));
  // (the markup of the block's affiliation lines that open with some)
  let affiliation_markups: Vec<Vec<Token>> = groups
    .iter()
    .flat_map(|(_, affiliations)| affiliations.iter().map(leading_markup))
    .filter(|m| !m.is_empty())
    .collect();
  let under_it = lines.get(index + 1).is_some_and(|under| {
    let under_markup = leading_markup(under);
    addresses_spelling(line, under)
      || (!under_markup.is_empty()
        && under_markup != markup
        && affiliation_markups.contains(&under_markup))
      || person_over_institution(line, under)
  });
  gap || styled_as_names || under_it
}

/// The author's note command `line` carries at its top level, `\thanks` or `\footnote` (the first one), if any.
fn author_note_of(line: &Tokens) -> Option<Token> {
  let mut depth = 0i32;
  line.unlist_ref().iter().copied().find(|t| {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    depth == 0 && (*t == T_CS!("\\thanks") || *t == T_CS!("\\footnote"))
  })
}

/// `line` without the byline word "by" (in any case) that opens it, when names follow it (see [`without_leading_and`]), and the
/// spaces after it (ties and control spaces too: `By~~{\scshape Vitaly~Bergelson}~~and~~…`, 2603.25922).
pub(in crate::base_utilities) fn without_leading_by(line: &Tokens) -> Option<Tokens> {
  let v = line.unlist_ref();
  let first = v.iter().position(|t| t.get_catcode() == Catcode::LETTER)?;
  let spacing = |t: Option<&Token>| t.is_some_and(|t| *t == T_SPACE!() || is_spacing_command(t));
  // (in any case: "by", "By", "BY")
  if !(v[first] == T_LETTER!("b") || v[first] == T_LETTER!("B"))
    || !(v.get(first + 1) == Some(&T_LETTER!("y")) || v.get(first + 1) == Some(&T_LETTER!("Y")))
    || !spacing(v.get(first + 2))
  {
    return None;
  }
  let mut rest = first + 2;
  while spacing(v.get(rest)) {
    rest += 1;
  }
  let mut out = v[..first].to_vec();
  out.extend_from_slice(&v[rest..]);
  let out = Tokens::new(out);
  (name_count(&visible_name_text(out.unlist_ref())) >= 1).then_some(out)
}

/// `line` without the word "and" its text opens with — after its leading markup, which is kept (`{\it and Zhenghan
/// Wang}` → `{\it Zhenghan Wang}`) — and the spaces after it; `None` when its first word is not "and".
pub(in crate::base_utilities) fn without_leading_and(line: &Tokens) -> Option<Tokens> {
  let v = line.unlist_ref();
  let first = v.iter().position(|t| t.get_catcode() == Catcode::LETTER)?;
  if v.get(first..first + 3) != Some(&[T_LETTER!("a"), T_LETTER!("n"), T_LETTER!("d")][..])
    || v
      .get(first + 3)
      .is_some_and(|t| t.get_catcode() == Catcode::LETTER)
  {
    return None;
  }
  let mut rest = first + 3;
  while v.get(rest) == Some(&T_SPACE!()) {
    rest += 1;
  }
  let mut out = v[..first].to_vec();
  out.extend_from_slice(&v[rest..]);
  Some(Tokens::new(out))
}

/// A name-list piece that qualifies the name before it ([`is_name_qualifier`]: `Jr.`, `PhD`, `\emph{Member, IEEE}`)
/// rejoins that name with the separator written between them, instead of becoming an author of its own; so does a
/// piece of marks and notes only ([`only_marks_and_notes`]: `~\IEEEmembership{Senior Member,~IEEE,}\thanks{…}`,
/// 1510.02728), without its separator, which separated nothing visible.
pub(in crate::base_utilities) fn rejoin_name_qualifiers(
  pieces: Vec<(Vec<Token>, Tokens)>,
) -> Vec<Tokens> {
  rejoin_name_qualifiers_keeping_separators(pieces)
    .into_iter()
    .map(|(_, name)| name)
    .collect()
}

/// [`rejoin_name_qualifiers`], each name with the separator written before it (none for the first).
pub(in crate::base_utilities) fn rejoin_name_qualifiers_keeping_separators(
  pieces: Vec<(Vec<Token>, Tokens)>,
) -> Vec<(Vec<Token>, Tokens)> {
  let mut out: Vec<(Vec<Token>, Tokens)> = Vec::with_capacity(pieces.len());
  for (delimiter, piece) in pieces {
    let unseen = only_marks_and_notes(piece.unlist_ref());
    match out.last_mut().map(|(_, name)| name) {
      Some(name) if unseen || is_name_qualifier(&visible_name_text(piece.unlist_ref())) => {
        let mut joined = name.clone().unlist();
        if !unseen {
          // the comma's space went with the piece's trimming
          let comma = delimiter.last() == Some(&T_OTHER!(","));
          joined.extend(delimiter);
          if comma {
            joined.push(T_SPACE!());
          }
        }
        joined.extend(piece.unlist());
        *name = Tokens::new(joined);
      },
      _ => out.push((delimiter, piece)),
    }
  }
  out
}

/// Whether `tokens` hold nothing to read but marks and notes: spaces (`~`, `\ `, `\,`), a note or mark command with
/// its arguments ([`NAME_ANNOTATIONS`], `\footnotemark[…]`, the mark commands of [`mark_commands`]:
/// `\textsuperscript`, `\IEEEauthorrefmark`, `\authorrefmark`), a superscript or subscript, or math holding only those
/// (`$^{*}$`). An author written as a macro (`\A`) is something to read.
pub(in crate::base_utilities) fn only_marks_and_notes(tokens: &[Token]) -> bool {
  let mut i = 0;
  let mut any = false;
  while i < tokens.len() {
    let t = tokens[i];
    match t.get_catcode() {
      Catcode::SPACE => i += 1,
      Catcode::ACTIVE if t.with_str(|s| s == "~") => i += 1,
      Catcode::CS if t == T_CS!("\\ ") || t == T_CS!("\\,") => i += 1,
      // `${}^{1}$`'s empty group
      Catcode::BEGIN
        if tokens
          .get(i + 1)
          .is_some_and(|n| n.get_catcode() == Catcode::END) =>
      {
        i += 2
      },
      // (a mark command too: ieeeaccess's `\authorrefmark{2}` after a list's closing period, 2201.07394)
      Catcode::CS if is_footnote_mark(&t) || is_name_annotation(&t) || is_mark_command(&t) => {
        any = true;
        i += 1;
        if tokens.get(i) == Some(&T_OTHER!("[")) {
          while i < tokens.len() && tokens[i] != T_OTHER!("]") {
            i += 1;
          }
          i += 1;
        }
        if !is_footnote_mark(&t) {
          i = skip_group(tokens, i);
        }
      },
      Catcode::SUPER | Catcode::SUB => {
        any = true;
        i = skip_group(tokens, i + 1);
      },
      Catcode::MATH => {
        let Some(close) = tokens[i + 1..]
          .iter()
          .position(|t| t.get_catcode() == Catcode::MATH)
        else {
          return false;
        };
        if !only_marks_and_notes(&tokens[i + 1..i + 1 + close])
          || tokens[i + 1..i + 1 + close].is_empty()
        {
          return false;
        }
        any = true;
        i += close + 2;
      },
      _ => return false,
    }
  }
  any
}
