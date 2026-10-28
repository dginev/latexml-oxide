//! The visible text of a name: name words, particles, qualifiers, name groups and merged creators.

use super::*;

/// The visible text of an author line as a name-shape test reads it: its letters, other characters and spaces at
/// any brace depth, without control sequences, math, superscripts, or the argument of a note or mark command
/// ([`NAME_ANNOTATIONS`]). `{\it D.A. Johnston}` reads "D.A. Johnston", `Avi Shporer\altaffilmark{1}` "Avi Shporer",
/// `Plech\'a\v{c}` "Plechac", `Zheng \& Jordi` "Zheng & Jordi".
pub(in crate::base_utilities) fn visible_name_text(tokens: &[Token]) -> String {
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
          // command's name aside
          text.push(letter);
          while tokens.get(i + 1) == Some(&T_SPACE!()) {
            i += 1;
          }
        }
        let annotation = is_name_annotation(&t);
        i += 1;
        // an accent's spaces before its letter are not text (`Plech\' a\v{c}`, cond-mat9705101; tex.web §1123
        // `\accent` skips them)
        if t.with_str(|s| ACCENT_COMMANDS.contains(&s)) {
          while tokens.get(i) == Some(&T_SPACE!()) {
            i += 1;
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

/// The index after the brace group (or single token) at `i`.
pub(in crate::base_utilities) fn skip_group(tokens: &[Token], mut i: usize) -> usize {
  if tokens
    .get(i)
    .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
  {
    let mut depth = 0i32;
    while i < tokens.len() {
      match tokens[i].get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => {
          depth -= 1;
          if depth == 0 {
            return i + 1;
          }
        },
        _ => {},
      }
      i += 1;
    }
    i
  } else {
    i + 1
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
  "consortium",
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
  "group",
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
fn strip_glued_marks(text: &str) -> &str {
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
  words.iter().all(|word| {
    if non_name_word(word) {
      return false;
    }
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
fn unglued_initials(word: &str) -> Vec<&str> {
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

/// The societies whose membership grade follows an author's name (`Senior Member, IEEE`).
const NAME_GRADE_SOCIETIES: &[&str] = &[
  "IEEE", "ACM", "OSA", "SPIE", "IET", "IEICE", "AAAI", "SIAM", "APS", "IAPR", "IFAC", "ASCE",
  "ASME",
];
/// The abbreviated grades glued to a society's name (`M.ASCE`, `F.ASCE`, `A.M.ASCE`: ascelike's own
/// `\author[1]{Matthew R. Kuhn, M.ASCE}`, ascexmpl.tex:48; ASCE's Student, Associate, full and Distinguished Members
/// and Fellows).
const NAME_GRADE_ABBREVIATIONS: &[&str] = &["S.M.", "A.M.", "M.", "F.", "Dist.M."];
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
  let words: Vec<&str> = text.split([' ', ',']).filter(|w| !w.is_empty()).collect();
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

/// Whether a name carries a mark after it, outside its notes: a superscript or mark command
/// ([`is_affiliation_mark`]: IEEEtran's `\IEEEauthorrefmark`) or aastex's `\altaffilmark` after its first letter
/// (`Bob Baker$^{2}$`; not `$^{2}$Univ B`).
fn marked_after_name(name: &Tokens) -> bool {
  let outside = outside_notes(name.unlist_ref());
  let Some(first_letter) = outside
    .iter()
    .position(|t| t.get_catcode() == Catcode::LETTER)
  else {
    return false;
  };
  outside[first_letter..]
    .iter()
    .any(|t| is_affiliation_mark(t) || *t == T_CS!("\\altaffilmark"))
}

/// The markup an author line opens with, before its first letter: its control sequences and the groups they open
/// (`{\small\em Bartol …}` → `{`, `\small`, `\em`; `T. Stelzer` → none). Lines styled alike share it.
fn leading_markup(line: &Tokens) -> Vec<Token> {
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
fn has_person_cue(text: &str) -> bool {
  let words: Vec<&str> = text.split_whitespace().collect();
  words.iter().any(|word| is_initial(word))
    || (words.len() > 2
      && words[1..words.len() - 1]
        .iter()
        .any(|word| NAME_PARTICLES.contains(word)))
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
  let mut out: Vec<Tokens> = Vec::with_capacity(pieces.len());
  for (delimiter, piece) in pieces {
    let unseen = only_marks_and_notes(piece.unlist_ref());
    match out.last_mut() {
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
      _ => out.push(piece),
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

/// Whether a personname's child is a mark: an `<ltx:sup>`, or a formula that is only a superscript (`$^{1}$`, its `tex`
/// `{}^{1}` until the math is rewritten into an `<ltx:sup>`; [`shown_marks`]).
fn is_name_mark(node: &Node, qname: SymStr) -> bool {
  with(qname, |q| q == "ltx:sup")
    || (with(qname, |q| q == "ltx:Math")
      && node.get_attribute("tex").is_some_and(|tex| {
        tex
          .trim()
          .trim_start_matches("{}")
          .trim_start()
          .starts_with('^')
      }))
}

/// The text a `<ltx:personname>` shows, without its marks and notes (`ltx:sup`, `ltx:note`, math); an element
/// (a `<ltx:break/>`, a cell) parts the words around it. With `marks_part`, each mark ([`is_name_mark`]) reads as a
/// comma, as an author's mark after a name ends it.
fn personname_text(node: &Node, marks_part: bool) -> String {
  let mut text = String::new();
  for child in node.get_child_nodes() {
    match child.get_type() {
      Some(NodeType::TextNode) => text.push_str(&child.get_content()),
      Some(NodeType::ElementNode) => {
        text.push(' ');
        let qname = document::get_node_qname(&child);
        if marks_part && is_name_mark(&child, qname) {
          text.push(',');
        } else if !with(qname, |q| {
          q == "ltx:sup" || q == "ltx:note" || q == "ltx:Math" || q == "ltx:contact"
        }) {
          text.push_str(&personname_text(&child, marks_part));
        }
        text.push(' ');
      },
      _ => {},
    }
  }
  text
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
    // An author block laid out as a tabular is kept whole on purpose (`add_authors_calls`; Perl
    // Base_Utility.pool.ltxml:693): its names are there, in their layout.
    let Some(person) = document.findnode("ltx:personname[not(.//ltx:tabular)]", Some(&creator))
    else {
      continue;
    };
    let text = personname_text(&person, false);
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    // (a mark after a name ends it, as an author's mark stands: names parted only by marks and spacing,
    // `FLORIAN REETZ$^1$ \ns PRIYA SUBRAMANIAN$^{2,3}$`, 1911.02873, read as one name without them)
    let names = match name_count(&text) {
      n if n >= 2 => n,
      n => n.max(name_count(&personname_text(&person, true))),
    };
    if names >= 2 {
      Error!(
        "frontmatter",
        "merged_creators",
        s!(
          "One author holds {names} names: \"{text}\"; the separators between them were not recognized"
        )
      );
    }
  }
  Ok(())
}
