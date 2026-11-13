//! The text rules on printed ink and the token primitives of `author_block/name_text.rs`, moved unchanged (the plan's row
//! "Knowledge in, driver out, one pass", user ruling 2026-11-08).

use super::*;

/// The visible text of an author line as a name-shape test reads it: its letters, other characters and spaces at
/// any brace depth, without control sequences, math, superscripts, or the argument of a note or mark command
/// ([`NAME_ANNOTATIONS`]). `{\it D.A. Johnston}` reads "D.A. Johnston", `Avi Shporer\altaffilmark{1}` "Avi Shporer",
/// `Plech\'a\v{c}` "Plechac", `Zheng \& Jordi` "Zheng & Jordi".
pub(in crate::base_utilities) fn visible_name_text(tokens: &[Token]) -> String {
  name_text(tokens, false)
}

/// The visible text of a name as an address spells it ([`NameKeys::of`]): [`visible_name_text`], but a letter an accent
/// or letter command writes that has a conventional transliteration ([`resolver::transliteration`]: `\"a` ä, `{\ss}`
/// ß, `\o` ø, `\aa` and `\r{a}` å, `\ae` æ, `\oe` œ) is that letter, not its base letter alone — the base letter is
/// one fold of it ([`resolver::base_letters`]) and the transliteration another, which the accent dropped here would
/// lose (`Sch\"affler` "schaeffler" as `Schäffler` reads, html_feedback #7233; OXIDIZED_DESIGN_DIVERGENCES #970).
pub(in crate::base_utilities) fn spelling_name_text(tokens: &[Token]) -> String {
  name_text(tokens, true)
}

/// [`visible_name_text`]; `spelling`: the letters with a transliteration kept whole ([`spelling_name_text`]).
fn name_text(tokens: &[Token], spelling: bool) -> String {
  let mut text = String::new();
  let mut i = 0;
  while i < tokens.len() {
    let t = tokens[i];
    match t.get_catcode() {
      Catcode::CS => {
        // a control space or thin space between initials (`Joshua N.\ Winn`, `J.\,R. Smith`) is a space
        if t == T_CS!("\\ ") || t == T_CS!("\\,") {
          text.push(' ');
        } else if t == T_CS!("\\&") {
          text.push('&');
        } else if t == T_CS!("\\and") || t == T_CS!("\\And") || t == T_CS!("\\AND") {
          // (an author separator inside a wrapper reads as the word it prints: `{\bf {\large A \and B}}`, 2401.03555)
          text.push_str(" and ");
        } else if let Some(letter) = t.with_str(letter_command) {
          // a letter written as a command (`{\L}ukasz`, `\O stergaard`) is that letter, the spaces ending the
          // command's name aside (the letter it prints, where an address may spell it by its transliteration)
          match t.with_str(printed_letter_command) {
            Some(printed) if spelling && resolver::transliteration(printed).is_some() => {
              text.push(printed)
            },
            _ => text.push(letter),
          }
          while tokens.get(i + 1) == Some(&T_SPACE!()) {
            i += 1;
          }
        }
        // (and the kernel's own contact call a creator's content holds, a heading's collaboration after the name or a
        // link line's URL, OXIDIZED_DESIGN_DIVERGENCES #946: its keyvals and text are no name's — read, they ran into
        // the name's last word, "WestXYZ", and the address spelling A~D~West went to no one, 1612.09318)
        let annotation = is_name_annotation(&t) || t == T_CS!("\\lx@add@contact");
        i += 1;
        // an environment's `\begin{…}`/`\end{…}` prints no word, its name no more than an item's label does
        // (`\begin{itemize}\item[\Letter] Bob Baker`, svjour3's `\institute` before `\at`, 2305.05902: "itemize" read
        // as a word of the line, the rev98b review's pe1)
        if t == T_CS!("\\begin") || t == T_CS!("\\end") {
          while tokens.get(i) == Some(&T_SPACE!()) {
            i += 1;
          }
          i = skip_group(tokens, i);
          continue;
        }
        // an accent's spaces before its letter are not text (`Plech\' a\v{c}`, cond-mat9705101; tex.web §1123
        // `\accent` skips them)
        if t.with_str(|s| ACCENT_COMMANDS.contains(&s)) {
          while tokens.get(i) == Some(&T_SPACE!()) {
            i += 1;
          }
          // (the accented letter, where it has a transliteration and an address may spell it so: `\"a`, `\"{u}`, `\r a`)
          if spelling && let Some((letter, end)) = accented_letter(&t, tokens, i) {
            text.push(letter);
            i = end;
            continue;
          }
        }
        // a control word's optional argument is no text (`\parbox[c]{\textwidth}{…}`, 2504.03146), nor is glue
        // (`and Yael Shadmi$^{d}$ \vskip 0.3in`, hep-ph0107182)
        if !annotation
          && parts_words(&t)
          && tokens.get(i) == Some(&T_OTHER!("["))
          && let Some(close) = tokens[i..].iter().position(|u| *u == T_OTHER!("]"))
        {
          i += close + 1;
        }
        if ["\\vskip", "\\hskip", "\\kern"]
          .iter()
          .any(|cs| t == T_CS!(*cs))
        {
          i = past_glue(tokens, i);
          if t == T_CS!("\\hskip") {
            text.push(' ');
          }
          continue;
        }
        // an optional `[...]` (`\footnotemark[2]` has nothing more), then a note's or mark's braced argument
        if annotation || is_footnote_mark(&t) {
          if tokens.get(i) == Some(&T_OTHER!("[")) {
            while i < tokens.len() && tokens[i] != T_OTHER!("]") {
              i += 1;
            }
            i += 1;
          }
          if annotation {
            i = skip_group(tokens, i);
          }
        }
        // a space with its length is no text (`Yu.S.Velikzhanin \vspace{1mm}\\`, hep-ex0105093), a horizontal one a
        // space
        if t == T_CS!("\\vspace") || t == T_CS!("\\hspace") {
          if tokens.get(i) == Some(&T_OTHER!("*")) {
            i += 1;
          }
          i = skip_group(tokens, i);
          if t == T_CS!("\\hspace") {
            text.push(' ');
          }
          continue;
        }
        // another command's argument with no letter in it is an identifier or a mark, no name text
        // (`\orcidicon{0000-0001-…}`, `\authorrefmark{1, 3}`: 2104.02493, 2407.03625); a letter is one of any script
        // (`\textbf{Μαρία}`, its letters OTHER under pdfTeX's catcodes)
        if !annotation
          && tokens
            .get(i)
            .is_some_and(|n| n.get_catcode() == Catcode::BEGIN)
        {
          let end = skip_group(tokens, i).min(tokens.len());
          let lettered = tokens[i..end].iter().any(|u| match u.get_catcode() {
            Catcode::LETTER | Catcode::OTHER => u.with_str(|s| s.chars().any(char::is_alphabetic)),
            Catcode::CS => u.with_str(letter_command).is_some(),
            _ => false,
          });
          // (nor is a graphic's file name or a link's address: `\href{https://…}{\includegraphics[scale=0.06]{orcid.pdf}
          // \hspace{1mm}Hossein Bahak, Farzaneh Taheri}`, 2312.07592)
          if !lettered || t == T_CS!("\\includegraphics") || t == T_CS!("\\href") {
            i = end;
          }
        }
        continue;
      },
      Catcode::MATH => {
        // a word set in math is name text — a surname given in math to carry its mark (`M. ${\rm Schreiber}^{*}$`,
        // cond-mat9402093): a run of two or more letters, broken by anything but a command or a brace; a script and its
        // operand (`$^{a}$`, `$^\mathrm{ab}$`) and a lone letter (`$e^+e^-$`) are not
        i += 1;
        let mut word = String::new();
        let end_word = |word: &mut String, text: &mut String| {
          if word.chars().count() >= 2 {
            text.push_str(word);
          }
          word.clear();
        };
        while i < tokens.len() && tokens[i].get_catcode() != Catcode::MATH {
          match tokens[i].get_catcode() {
            Catcode::SUPER | Catcode::SUB => {
              end_word(&mut word, &mut text);
              i = skip_script_operand(tokens, i + 1);
              continue;
            },
            Catcode::LETTER => tokens[i].with_str(|s| word.push_str(s)),
            Catcode::CS | Catcode::BEGIN | Catcode::END => {},
            _ => end_word(&mut word, &mut text),
          }
          i += 1;
        }
        end_word(&mut word, &mut text);
      },
      Catcode::SUPER | Catcode::SUB => {
        i = skip_script_operand(tokens, i + 1);
        continue;
      },
      // a run of bare asterisks after a name is its mark (`S. F. Pereira,* Z. Y. Ou,** and H. J. Kimble`,
      // quant-ph0003094)
      Catcode::OTHER if t == T_OTHER!("*") => {},
      Catcode::LETTER | Catcode::OTHER => t.with_str(|s| text.push_str(s)),
      Catcode::SPACE => text.push(' '),
      // `~` is a space; another active character (babel's `"`) is no text
      Catcode::ACTIVE if t.with_str(|s| s == "~") => text.push(' '),
      _ => {},
    }
    i += 1;
  }
  text
}

/// The letter the accent command `accent` makes of its operand at `i` (a letter, or a group of one letter) when that
/// letter has a transliteration ([`resolver::transliteration`]: `\"a` ä, `\"{U}` Ü, `\r a` å), with the index after
/// the operand; `None` for any other.
fn accented_letter(accent: &Token, tokens: &[Token], i: usize) -> Option<(char, usize)> {
  use unicode_normalization::UnicodeNormalization;
  let mark = accent.with_str(|s| match s {
    "\\\"" => Some('\u{308}'),
    "\\r" => Some('\u{30A}'),
    _ => None,
  })?;
  let (letter, end) = match tokens.get(i)?.get_catcode() {
    Catcode::BEGIN
      if tokens
        .get(i + 2)
        .is_some_and(|t| t.get_catcode() == Catcode::END) =>
    {
      (tokens.get(i + 1)?, i + 3)
    },
    _ => (tokens.get(i)?, i + 1),
  };
  if !matches!(letter.get_catcode(), Catcode::LETTER | Catcode::OTHER) {
    return None;
  }
  let base = letter.with_str(|s| {
    let mut chars = s.chars();
    chars.next().filter(|_| chars.next().is_none())
  })?;
  let composed: String = [base, mark].iter().collect::<String>().nfc().collect();
  let mut chars = composed.chars();
  let one = chars.next().filter(|_| chars.next().is_none())?;
  resolver::transliteration(one).map(|_| (one, end))
}

/// The index after a script's operand at `i`: a brace group, a command with its brace group (`^\mathrm{ab}`), or one
/// token.
fn skip_script_operand(tokens: &[Token], i: usize) -> usize {
  if tokens
    .get(i)
    .is_some_and(|t| t.get_catcode() == Catcode::CS)
    && tokens
      .get(i + 1)
      .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
  {
    skip_group(tokens, i + 1)
  } else {
    skip_group(tokens, i)
  }
}

/// The index after the brace group (or single token) at `i`; the end of `tokens` for a group that never closes.
pub(in crate::base_utilities) fn skip_group(tokens: &[Token], i: usize) -> usize {
  match tokens_group_end(tokens, i) {
    Some(end) => end,
    None
      if tokens
        .get(i)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN) =>
    {
      tokens.len()
    },
    None => i + 1,
  }
}

/// Particles that stand inside a personal name (`Camillo De Lellis`, `Ludwig van Beethoven`) but never open or close it.
pub(in crate::base_utilities) const NAME_PARTICLES: &[&str] = &[
  "van", "von", "de", "der", "den", "del", "della", "da", "di", "du", "la", "le", "ten", "ter",
  "bin", "ibn", "al", "y", "dos", "das", "do",
];

/// Words that name an institution, a place or a role, never a person — whole words, lowercased, the punctuation
/// around them (`.,;:()[]`) dropped, in the languages arXiv affiliations are written in (`Dipartimento di Fisica`, `Sezione di Napoli`,
/// `Osservatorio Astronomico`, `École`). Whole words, not word openings: "Strasser", "Schooler", "Campusano" are
/// surnames.
pub(in crate::base_utilities) const NON_NAME_WORDS: &[&str] = &[
  "univ",
  "university",
  "universities",
  "universität",
  "universitat",
  "universitaet",
  "universidad",
  "universidade",
  "université",
  "universite",
  "universiteit",
  "università",
  "universita",
  "universitet",
  "uniwersytet",
  "institute",
  "institutes",
  "institut",
  "instituto",
  "istituto",
  "institution",
  "inst",
  "laboratory",
  "laboratories",
  "laboratoire",
  "laboratorio",
  "laboratorium",
  "lab",
  "labs",
  "observatory",
  "observatoire",
  "observatorio",
  "osservatorio",
  "observatorium",
  "sterrewacht",
  "academy",
  "academia",
  "académie",
  "academie",
  "akademie",
  "accademia",
  "acad",
  "department",
  "departments",
  "departement",
  "département",
  "departamento",
  "dipartimento",
  "departament",
  "dept",
  "dep",
  "faculty",
  "faculté",
  "faculte",
  "facultad",
  "faculdade",
  "fakultät",
  "fakultat",
  "fachbereich",
  "division",
  "sezione",
  "section",
  "sektion",
  "collaboration",
  "collaborations",
  "consortium",
  "consortia",
  "experiment",
  "experiments",
  "hospital",
  "hôpital",
  "hopital",
  "ospedale",
  "klinik",
  "klinikum",
  "clinic",
  "clinique",
  "foundation",
  "fundación",
  "fundacion",
  "fondazione",
  "stiftung",
  "society",
  "société",
  "societe",
  "società",
  "professor",
  "ministry",
  "museum",
  "company",
  "corporation",
  "corp",
  "inc",
  "llc",
  "ltd",
  "gmbh",
  "physics",
  "physik",
  "research",
  "school",
  "scuola",
  "escuela",
  "école",
  "ecole",
  "hochschule",
  "politecnico",
  "polytechnic",
  "polytechnique",
  "politécnica",
  "politecnica",
  "college",
  "collège",
  "colegio",
  "center",
  "centers",
  "centre",
  "centres",
  "centro",
  "zentrum",
  "centrum",
  "ctr",
  "program",
  "programme",
  "agency",
  "consiglio",
  "conseil",
  "avenue",
  "strasse",
  "straße",
  // a two-word place reads as a name (`\author{John Smith, New York}`): its words that no name carries
  "new",
  "tel",
  "team",
  "teams",
  "group",
  "groups",
  "member",
  "fellow",
  "student",
  "email",
  "e-mail",
  "obs",
  "natl",
  "sch",
  "sci",
];

/// `\footnotemark` or a mark manyfoot declares beside it (`\DeclareNewFootnote{AAffil}` makes `\footnotemarkAAffil`,
/// manyfoot.sty:253-257; 2309.13063): a mark with an optional `[n]` and no argument.
pub(in crate::base_utilities) fn is_footnote_mark(t: &Token) -> bool {
  t.with_str(|s| s.starts_with("\\footnotemark"))
}

/// Acronyms of institutions and societies, in capitals only: "Meta AI", "ESA" are none of them a name, though Qingyao
/// Ai and Esa Räsänen are.
const NON_NAME_ACRONYMS: &[&str] = &[
  "AI", "IEEE", "ACM", "INFN", "CNRS", "INAF", "CSIC", "NASA", "ESA", "ESO", "CERN", "DESY",
  "RIKEN", "KEK", "IPMU", "MIT", "UCLA", "SISSA", "TU",
];

/// The lowercased whole words of `word`: itself, the punctuation around it (`.,;:()[]`) dropped, and the parts of a
/// hyphenated compound (`Max-Planck-Institut`).
pub(in crate::base_utilities) fn word_forms(word: &str) -> impl Iterator<Item = String> + '_ {
  let bare = word.trim_matches(|c: char| ".,;:()[]".contains(c));
  std::iter::once(bare)
    .chain(bare.split('-'))
    .map(str::to_lowercase)
}

/// Whether a word names an institution, a place or a role ([`NON_NAME_WORDS`], [`NON_NAME_ACRONYMS`]), never a person.
pub(in crate::base_utilities) fn non_name_word(word: &str) -> bool {
  let bare = word.trim_matches(|c: char| ".,;:()[]".contains(c));
  word_forms(word).any(|form| NON_NAME_WORDS.contains(&form.as_str()))
    || bare
      .split('-')
      .any(|part| NON_NAME_ACRONYMS.contains(&part))
}

/// `text` without the marks glued to its end (`Ann Able*`, `Bob Baker1,`): digits and symbols right after a letter.
/// Digits after a space are text — a postal code (`CA 93106`), not a mark.
pub(in crate::base_utilities) fn strip_glued_marks(text: &str) -> &str {
  let text = text.trim();
  // (a list's closing `;` too: sn-jnl's `\author[1]{A.~Able, B.~Baker, C.~Cole;}`, 2504.09158)
  let stripped = text.trim_end_matches(|c: char| c.is_ascii_digit() || "*∗†‡§¶♯#,;".contains(c));
  if stripped.len() < text.len() && stripped.ends_with(char::is_whitespace) {
    text
  } else {
    stripped.trim_end()
  }
}

/// Whether `text` — one part of an author line, as [`visible_name_text`] reads it — is shaped like a personal name:
/// two to five words, each capitalised or an initial (`J.`, `D.A.`), name particles inside but not at either end, no
/// digit, `@`, `:`, `/` or parenthesis and no institution word, once the marks glued to its end are dropped.
fn name_shaped(text: &str) -> bool {
  let text = strip_glued_marks(text);
  if text
    .chars()
    .any(|c| c.is_ascii_digit() || "@:/()".contains(c))
  {
    return false;
  }
  let words: Vec<&str> = text.split_whitespace().flat_map(unglued_initials).collect();
  if !(2..=5).contains(&words.len()) {
    return false;
  }
  let particle = |w: &str| NAME_PARTICLES.contains(&w);
  if particle(words[0]) || particle(words[words.len() - 1]) {
    return false;
  }
  words.iter().enumerate().all(|(at, word)| {
    if non_name_word(word) {
      return false;
    }
    // (a particle elided onto the surname is the surname's right after an initial: `D.~d'Enterria`, hep-ph0311131 —
    // and only there, this helper deciding today's reader too: a place or an institution elides as well, `C. Vignat,
    // Université d'Orsay`, 1009.2396; `Villeneuve d'Ascq`, `Saint Martin d'Heres`)
    let after_initial = at > 0 && is_initial(words[at - 1]);
    let word = ["d'", "d’", "l'", "l’", "dell'", "dell’"]
      .iter()
      .filter(|_| after_initial)
      .find_map(|elided| word.strip_prefix(elided))
      .unwrap_or(word);
    particle(word)
      || (word.chars().next().is_some_and(char::is_uppercase)
        && word
          .chars()
          .all(|c| c.is_alphabetic() || "'’.-".contains(c)))
  })
}

/// A word of initials glued to the surname they precede, as its words: `A.G.Bogdanchikov` → `A.`, `G.`,
/// `Bogdanchikov`, `Yu.M.Shatunov` → `Yu.`, `M.`, `Shatunov` (hep-ex0105093's collaboration list; an initial is a
/// capital and at most one small letter, not a place's abbreviation: `St.Petersburg`, `Mt.Stromlo`); a word without a
/// surname after its initials (`U.S.A.`, `D.A.`, `Ph.D.`) or with none glued stays whole.
pub(in crate::base_utilities) fn unglued_initials(word: &str) -> Vec<&str> {
  let mut parts = Vec::new();
  let mut rest = word;
  loop {
    let mut chars = rest.char_indices().peekable();
    let Some((_, capital)) = chars.next() else {
      break;
    };
    chars.next_if(|(_, c)| c.is_lowercase());
    match (chars.next(), chars.next()) {
      (Some((dot, '.')), Some((next, after)))
        if capital.is_uppercase()
          && after.is_uppercase()
          && !["St", "Mt", "Ft"].contains(&&rest[..dot]) =>
      {
        parts.push(&rest[..next]);
        rest = &rest[next..];
      },
      _ => break,
    }
  }
  // (a surname in capitals too, of three letters or more: `M.CACCIA`, hep-ex9910019; not an abbreviation's last letter,
  // `I.N.F.N`)
  let capitals_surname =
    rest.chars().count() >= 3 && rest.chars().all(|c| c.is_uppercase() || "'’-".contains(c));
  if parts.is_empty() || !(rest.chars().skip(1).any(char::is_lowercase) || capitals_surname) {
    return vec![word];
  }
  parts.push(rest);
  parts
}

/// The societies whose membership grade follows an author's name (`Senior Member, IEEE`), or stands on a line of its own
/// (`frontmatter::insert::reads_as_membership_grade`).
pub(in crate::base_utilities) const NAME_GRADE_SOCIETIES: &[&str] = &[
  "IEEE", "ACM", "OSA", "SPIE", "IET", "IEICE", "AAAI", "SIAM", "APS", "IAPR", "IFAC", "ASCE",
  "ASME",
];

/// The abbreviated grades glued to a society's name (`M.ASCE`, `F.ASCE`, `A.M.ASCE`: ascelike's own
/// `\author[1]{Matthew R. Kuhn, M.ASCE}`, ascexmpl.tex:48; ASCE's Student, Associate, full and Distinguished Members
/// and Fellows), and IEEE's written without dots (`SMIEEE`, `FIEEE`, `MIEEE`, `LSMIEEE`: its Senior, full, Life, Life
/// Senior, Graduate Student and Student Members and its Fellows and Life Fellows; `Krishan K. Tiwari$^1$,
/// \textit{SMIEEE}, Eckhard Grass$^{1,2}$, …`, 1906.01719).
pub(in crate::base_utilities) const NAME_GRADE_ABBREVIATIONS: &[&str] = &[
  "S.M.", "A.M.", "M.", "F.", "Dist.M.", "SM", "M", "F", "LM", "LSM", "LF", "GSM", "S",
];

/// Generational suffixes and academic degrees written after a name, comma-separated (`John Smith, Jr.`, `Jane Doe,
/// MD, PhD`).
pub(in crate::base_utilities) const NAME_SUFFIXES: &[&str] = &[
  "Jr", "Jr.", "Sr", "Sr.", "Jun.", "Sen.", "II", "III", "IV", "PhD", "Ph.D.", "Ph.D", "MD",
  "M.D.", "MSc", "M.Sc.", "BSc", "B.Sc.", "MA", "M.A.", "MS", "M.S.", "MBA", "MPH", "MBBS",
  "DPhil", "D.Phil.", "DSc", "D.Sc.", "FRS", "RN", "Esq.", "P.E.",
];

/// Whether an author-list part qualifies the name before it instead of naming a person: a generational suffix or a
/// degree ([`NAME_SUFFIXES`]), or a professional membership grade (`Member, IEEE`, `Senior Member`, or the society
/// alone once the list is split at the grade's comma), which LaTeX prints after the name, comma-separated.
pub(in crate::base_utilities) fn is_name_qualifier(text: &str) -> bool {
  let text = strip_glued_marks(text);
  if NAME_SUFFIXES.contains(&text) {
    return true;
  }
  // (the rest of a list left unnamed is no one: `Ann Able and others`, `\emph{and others}`, `Ann Able, et al.`)
  let words: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
  if ["others", "et al.", "et al", "et. al.", "et.al.", "et.al"]
    .contains(&words.to_lowercase().as_str())
  {
    return true;
  }
  // an abbreviated grade before its society, glued or spaced (`M.ASCE`, `M. ASCE`, `A. M. ASCE`)
  let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
  if NAME_GRADE_ABBREVIATIONS.iter().any(|grade| {
    compact
      .strip_prefix(grade)
      .is_some_and(|society| NAME_GRADE_SOCIETIES.contains(&society))
  }) {
    return true;
  }
  grade_words(text, false)
}

/// Whether `text`'s words are all a membership grade's or its society's (`Senior Member, IEEE`); with `ranked`, one of
/// them the grade's rank (`Member`, `Fellow`), so a society alone is none.
pub fn grade_words(text: &str, ranked: bool) -> bool {
  let words: Vec<&str> = text
    .split(|c: char| c.is_whitespace() || c == ',' || c == '~')
    .filter(|w| !w.is_empty())
    .collect();
  let grade = |w: &&str| {
    [
      "Life",
      "Senior",
      "Student",
      "Graduate",
      "Associate",
      "Member",
      "Fellow",
    ]
    .contains(w)
  };
  !words.is_empty()
    && words
      .iter()
      .all(|w| grade(w) || NAME_GRADE_SOCIETIES.contains(w))
    && (!ranked || words.iter().any(|w| ["Member", "Fellow"].contains(w)))
}

/// Whether `text` is a membership grade or a part of one the names line's commas cut it into (`Senior Member`, `IEEE`
/// of `\textit{Senior Member}, \textit{IEEE}`, 2009.02373): [`is_name_qualifier`]'s grades, no suffix, degree or "et
/// al.". The frontmatter observer's reading leaves it out of the name before it, as `\lx@add@author` takes the grade out
/// of the name ([`written_membership_out`]).
pub(in crate::base_utilities) fn is_membership_part(text: &str) -> bool {
  let text = strip_glued_marks(text);
  let compact: String = text.chars().filter(|c| !c.is_whitespace()).collect();
  grade_words(text, false)
    || NAME_GRADE_ABBREVIATIONS.iter().any(|grade| {
      compact
        .strip_prefix(grade)
        .is_some_and(|society| NAME_GRADE_SOCIETIES.contains(&society))
    })
}

/// The parts of an author-line text that may each be one name: split at commas, " and " and " & ", a leading "and"
/// dropped.
fn name_parts(text: &str) -> Vec<String> {
  text
    .split(',')
    .flat_map(|part| {
      part
        .split(" and ")
        .flat_map(|p| p.split(" & "))
        .map(str::to_string)
        .collect::<Vec<_>>()
    })
    .map(|part| {
      let part = part.trim();
      part
        .strip_prefix("and ")
        .or_else(|| part.strip_prefix("& "))
        .unwrap_or(part)
        .trim()
        .to_string()
    })
    // (a separator alone, of a doubled one: `Barrientos, \& and Omar López-Cruz`, astro-ph9612009)
    .filter(|part| !part.is_empty() && part != "&" && part != "and")
    .collect()
}

/// How many parts of `text` are names when every part is a name or qualifies one ([`is_name_qualifier`]); 0 when a
/// part is neither (an institution, a place).
pub(in crate::base_utilities) fn name_count(text: &str) -> usize {
  let parts = name_parts(text);
  if parts.iter().all(|p| name_shaped(p) || is_name_qualifier(p)) {
    // (a qualifier is no name, whatever its shape: `M.ASCE` reads as a glued initial and surname)
    parts
      .iter()
      .filter(|p| name_shaped(p) && !is_name_qualifier(p))
      .count()
  } else {
    0
  }
}

/// Whether a word is an initial: capitals, each with at most one small letter and its period, hyphens between (`B.`,
/// `V.I.`, `J.-P.`, `Yu.`), not a place's abbreviation (`St.`, `Mt.`, `Ft.`).
fn is_initial(word: &str) -> bool {
  let word = word.trim_end_matches([',', ';']);
  !word.is_empty()
    && !["St.", "Mt.", "Ft."].contains(&word)
    && word.split('-').all(|part| {
      let mut chars = part.chars().peekable();
      let mut any = false;
      while let Some(capital) = chars.next() {
        if !capital.is_uppercase() {
          return false;
        }
        chars.next_if(|c| c.is_lowercase());
        if chars.next() != Some('.') {
          return false;
        }
        any = true;
      }
      any
    })
}

/// Whether a name's text carries what only a person's name does: an initial ([`is_initial`]: `Palash B. Pal`, `James
/// M. Robins`) or a particle inside it ([`NAME_PARTICLES`]: `Ludwig van Beethoven`). Two capitalized words alone are as
/// often a place, an institution's head or a role (`Johns Hopkins`, `Max Planck`, `Santa Barbara`, `Associate Dean`)
/// as a name.
pub(in crate::base_utilities) fn has_person_cue(text: &str) -> bool {
  let words: Vec<&str> = text.split_whitespace().collect();
  words.iter().any(|word| is_initial(word))
    || (words.len() > 2
      && words[1..words.len() - 1]
        .iter()
        .any(|word| NAME_PARTICLES.contains(word)))
}

/// Whether a personname's child is a mark: an `<ltx:sup>`, or a formula that is one (`$^{1}$`, `$\rm ^{a}$`, read from
/// its structure as digested, before the math is rewritten into an `<ltx:sup>`: [`fm_read_formula`]; [`shown_marks`]).
fn is_name_mark(node: &Node, qname: SymStr) -> bool {
  with(qname, |q| q == "ltx:sup")
    || (with(qname, |q| q == "ltx:Math")
      && matches!(fm_read_formula(node), FmFormulaMarks::Marks(_)))
}

/// Where a `<ltx:personname>`'s printed lines part in [`personname_text`]: at a `<ltx:break/>` and at the edges of a
/// paragraph or a box's block (`\parbox{…}{A, B \begin{center}(X Collaboration)\end{center}}`'s `<ltx:p>`s). A
/// whitespace character, so the whole text reads with it as with a space.
const PERSONNAME_LINE_BREAK: char = '\u{2028}';

/// The elements whose edges part a personname's printed lines ([`PERSONNAME_LINE_BREAK`]).
const PERSONNAME_LINE_ELEMENTS: [&str; 6] = [
  "ltx:break",
  "ltx:p",
  "ltx:para",
  "ltx:inline-para",
  "ltx:inline-block",
  "ltx:block",
];

/// The text a `<ltx:personname>` shows, without its marks and notes (`ltx:sup`, `ltx:note`, math); an element
/// (a `<ltx:break/>`, a cell) parts the words around it, a line's edge ([`PERSONNAME_LINE_ELEMENTS`]) with a
/// [`PERSONNAME_LINE_BREAK`]. With `marks_part`, each mark ([`is_name_mark`]) reads as a comma, as an author's mark
/// after a name ends it.
fn personname_text(node: &Node, marks_part: bool) -> String {
  let mut text = String::new();
  for child in node.get_child_nodes() {
    match child.get_type() {
      Some(NodeType::TextNode) => text.push_str(&child.get_content()),
      Some(NodeType::ElementNode) => {
        let qname = document::get_node_qname(&child);
        let edge = if with(qname, |q| PERSONNAME_LINE_ELEMENTS.contains(&q)) {
          PERSONNAME_LINE_BREAK
        } else {
          ' '
        };
        text.push(edge);
        if marks_part && is_name_mark(&child, qname) {
          text.push(',');
        } else if !with(qname, |q| {
          q == "ltx:sup" || q == "ltx:note" || q == "ltx:Math" || q == "ltx:contact"
        }) {
          text.push_str(&personname_text(&child, marks_part));
        }
        text.push(edge);
      },
      _ => {},
    }
  }
  text
}

/// Whether `line` holds a word of two or more capitals and no small letter or dot (`IST`, `NVIDIA`; not the initials
/// `J.-F.`) beside a word that has small letters: an acronym in a mixed-case line, an institution's (`IST Austria`);
/// not a line set all in capitals (`KHIEM HONG PHAN`).
pub(in crate::base_utilities) fn holds_mixed_case_acronym(line: &str) -> bool {
  let words: Vec<&str> = line
    .split_whitespace()
    .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()))
    .filter(|w| w.chars().any(char::is_alphabetic))
    .collect();
  let acronym = |w: &&str| {
    w.chars().filter(|c| c.is_uppercase()).count() >= 2
      && !w.chars().any(|c| c.is_lowercase() || c == '.')
  };
  words.iter().any(acronym) && words.iter().any(|w| w.chars().any(char::is_lowercase))
}

/// How many names a personname's text reads as ([`name_count`]), its marks as written or each read as a comma
/// ([`personname_text`]'s two readings, `plain` and `marked`): the plain reading when it finds two or more, else the
/// larger.
fn personname_name_count(plain: &str, marked: &str) -> usize {
  match name_count(plain) {
    n if n >= 2 => n,
    n => n.max(name_count(marked)),
  }
}

/// A frontmatter author whose name reads as two or more people ("Ann Able and Bob Baker") lost the boundaries between
/// them: LaTeX typesets several authors where the document has one, and nothing else tells. Each such author is an
/// error, once (the creator is marked, for the frontmatter a streaming conversion places late). Every part must be a
/// name or qualify one (`Jane Doe, PhD`): a name beside its institution ("John Smith, Bell Labs") is not two people.
pub(in crate::base_utilities) fn flag_merged_creators(document: &mut Document) -> Result<()> {
  for mut creator in
    document.findnodes("//ltx:creator[@role='author'][not(@_merged_checked)]", None)
  {
    document.set_attribute(&mut creator, "_merged_checked", "1")?;
    // An author block laid out as a tabular or a minipage is kept whole on purpose (the old driver's `add_authors_calls`, deleted in DEL2; Perl
    // Base_Utility.pool.ltxml:693): its names are there, in their layout.
    let Some(person) = document.findnode(
      "ltx:personname[not(.//ltx:tabular) and not(.//ltx:inline-block[contains(@class, 'ltx_minipage')])]",
      Some(&creator),
    ) else {
      continue;
    };
    let plain = personname_text(&person, false);
    let marked = personname_text(&person, true);
    let text = plain.split_whitespace().collect::<Vec<_>>().join(" ");
    // (a mark after a name ends it, as an author's mark stands: names parted only by marks and spacing,
    // `FLORIAN REETZ$^1$ \ns PRIYA SUBRAMANIAN$^{2,3}$`, 1911.02873, read as one name without them)
    let names = personname_name_count(&text, &marked.replace(PERSONNAME_LINE_BREAK, " "));
    // (and each printed line read alone, so that a line that names no one beside the names does not hide them: a
    // names paragraph above its collaboration's, `\parbox{\textwidth}{\Large A,$^{1,2}$ B,$^{3}$ … \begin{center} (DES
    // Collaboration) \end{center}}`, 2311.03613 (55 names); names lines above their affiliations' line,
    // `\textbf{A$^{1}$\thanks{…}, B$^{1}$, …} \\ \textbf{…} \\ $^{1}$MIT \qquad $^{2}$NUS`, 2609.38746: one line of two
    // or more names is a merge; so are two or more lines each of names and nothing else, `Botian Shi <break/>Chiyu
    // Wang`, 2306.04988 — names a `\\` parted. OXIDIZED_DESIGN_DIVERGENCES #897)
    let lines: Vec<(usize, bool)> = plain
      .split(PERSONNAME_LINE_BREAK)
      .zip(marked.split(PERSONNAME_LINE_BREAK))
      .filter(|(line, _)| line.chars().any(char::is_alphabetic))
      .map(|(line, marked_line)| {
        (
          personname_name_count(line, marked_line),
          holds_mixed_case_acronym(line),
        )
      })
      .collect();
    let names = names.max(lines.iter().map(|(n, _)| *n).max().unwrap_or(0));
    // (a line set in mixed case holding an acronym names an institution, not a person: `Vage Egiazarian <break/>IST
    // Austria`, 2504.20039, is one name over its affiliation)
    let by_lines = lines.len() >= 2 && lines.iter().all(|&(n, acronym)| n > 0 && !acronym);
    let (names, line_names) = if by_lines {
      let sum = lines.iter().map(|(n, _)| *n).sum();
      (names.max(sum), names < sum)
    } else {
      (names, false)
    };
    // (and a names line kept whole two or more of whose pieces are marked names, `A.~Able$^{1}$, B.~in der Baker,
    // C.~Cole$^{1}$`: the merge its marks contradict, whatever its names' shape — the count of marked names the cautious
    // split left on it, `\lx@author@merged@names`, 1903.10014, 1212.4143; a creator the writer made again from the
    // reading carries none. OXIDIZED_DESIGN_DIVERGENCES #943)
    let names = names.max(
      creator
        .get_attribute("_mergednames")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0),
    );
    // (counted by its lines, each one name, the line under a name may be the next author's or the name's own
    // affiliation welded to it — `Bob Baker <break/>Carnegie Mellon`, a line no institution word marks as one: the
    // wording says which reading, the rev107b review's N2)
    if names >= 2 && line_names && lines.iter().all(|&(n, _)| n == 1) {
      let shown = plain
        .split(PERSONNAME_LINE_BREAK)
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| line.chars().any(char::is_alphabetic))
        .collect::<Vec<_>>()
        .join(" / ");
      Error!(
        "frontmatter",
        "merged_creators",
        s!(
          "One author holds {names} lines that each read as a name: \"{shown}\"; the names a line break parts, or a name and the line under it (its affiliation), were not told apart"
        )
      );
    } else if names >= 2 {
      Error!(
        "frontmatter",
        "merged_creators",
        s!(
          "One author holds {names} names: \"{text}\"; the separators between them were not recognized"
        )
      );
    } else if names == 0 && reads_as_a_line_under_names(&text) {
      // (a creator whose name is a line under the names — a legend's or an affiliation's text, `also at the Moscow
      // Institute of Physics and Technology, Moscow, Russia` of 1202.2048's `$^{a}$` legend: the rev107b review's B2 —
      // is never silent)
      Error!(
        "frontmatter",
        "creator_not_a_name",
        s!(
          "An author's name reads as no name: \"{text}\"; a line under the names (an affiliation or a legend) was read as an author"
        )
      );
    }
  }
  Ok(())
}

/// Whether a personname's text reads as a line under the names, no name: it opens with a word in lower case that is no
/// name particle ([`NAME_PARTICLES`]: `van der Berg` opens a name) and no article, holds a word naming an institution
/// ([`non_name_word`]), no address and no name ([`name_count`]) — a legend's text (`also at the Moscow Institute of
/// Physics and Technology, Moscow, Russia`, `on leave from …`, `now at …`; 1309.4007's `e-mail:` line) or an
/// affiliation's (`at the University of X`). A collaboration or a team named as an author opens with a capital or an
/// article (`The ATLAS Collaboration`, `the ATLAS Collaboration`, `the PE11 team of Kepler WG#1`) and is not read; nor
/// is a group named as an author in lower case (`sPHENIX Collaboration`, 2606.17184; `openJiuwen Team`, 2605.10052: a
/// creator, [`reads_as_collaboration`]) or whom the authors speak for, a group's name read so after [`SPEAKS_FOR`]
/// (`for the PAROS Clinical Research Network`, 2605.24212; not `for correspondence: Physics Department`).
fn reads_as_a_line_under_names(text: &str) -> bool {
  let Some(first) = text.split_whitespace().next() else {
    return false;
  };
  let speaks_for_a_group = || {
    SPEAKS_FOR.iter().any(|p| {
      text
        .get(..p.len())
        .is_some_and(|q| q.eq_ignore_ascii_case(p))
        && {
          let rest = text[p.len()..].trim_start();
          !rest.is_empty() && !reads_as_a_line_under_names(rest)
        }
    })
  };
  first.chars().next().is_some_and(char::is_lowercase)
    && !NAME_PARTICLES.contains(&first)
    && !["the", "a", "an", "and"].contains(&first)
    && !text.contains('@')
    && name_count(text) == 0
    && text.split_whitespace().any(non_name_word)
    && !reads_as_collaboration(text, true)
    && !speaks_for_a_group()
}
