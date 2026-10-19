//! aero.cls (latexstudio.net's template for the journal *Astrodynamics*, v1.0 2022/01/04; shipped with the paper, not
//! in TeX Live): the class is loaded raw, and its frontmatter goes to the frontmatter API.
//!
//! The class keeps its title page in expl3 variables, set by `\ADsetup{<key>=<value>,…}` through its own l3keys
//! (aero.cls:97-118: `title`, `author`, `address`, `email`, `abstract`, `keywords`, and the running head's `name` and
//! `url`), and prints them only in its `\RenewDocumentCommand\maketitle` (:128-148), which the kernel's locked
//! `\maketitle` refuses: the title, authors, abstract and keywords were lost, the replayed class `\maketitle` printing
//! them as body paragraphs (2610.12827, arXiv html_feedback; Perl has no binding either and leaves `\ADsetup`
//! undefined). Here the kernel `\maketitle` hands each variable the paper set — one that differs from the class's
//! `initial:n` default, which is the template's placeholder ("Paper title: Preparation of papers for
//! *Astrodynamics*", "First A. Author") — to the frontmatter, in the order the class prints them: the title, the
//! authors (the article `\author` split; the class's corresponding mark `\cor`, :92 `{\rm (\Letter)}`, is a request
//! for the e-mail, as elsarticle's `\corref` is for its `\cortext`), the address under every author (its `\sep` a line
//! break, as the class's hanging paragraphs, :137), the e-mail — the `\cor` author's, else placed by whose name it
//! spells (the class prints it apart, after `\Letter`, :139) —, the abstract and the keywords (its `\sep` the class's
//! ` · `, :145). A variable the paper never set is left alone: a copy of the class whose `\maketitle` prints the
//! kernel's `\@title` and `\@author` instead (2509.08607, 2511.17720: `\title`/`\author`, no `\ADsetup`) keeps them.
//! The class's own `\maketitle` is not replayed. `name` and `url` print only in the running head (:120-126), page
//! furniture. OXIDIZED_DESIGN_DIVERGENCES #775.
use latexml_package::prelude::*;

/// The class's title-page variables (`\l_ADkeys_<key>_tl`) in the order its `\maketitle` prints them, each with the
/// frontmatter call that takes its value.
const FIELDS: &[(&str, &str)] = &[
  ("title", r"\title"),
  ("author", r"\author"),
  ("address", r"\lx@aero@address"),
  ("email", r"\lx@aero@email"),
  ("abstract", r"\lx@add@abstract"),
  ("keywords", r"\lx@aero@keywords"),
];

/// The class variable of `key`, and the copy of its `initial:n` value the binding keeps.
fn variable(key: &str) -> Token { T_CS!(s!("\\l_ADkeys_{key}_tl")) }
fn initial_value(key: &str) -> Token { T_CS!(s!("\\lx@aero@initial@{key}")) }

/// The one-level expansion of a macro without parameters, `None` when it is undefined or no such macro.
fn macro_body(cs: &Token) -> Result<Option<Tokens>> {
  Ok(match lookup_definition(cs)? {
    Some(defn) => match defn.get_expansion() {
      Some(ExpansionBody::Tokens(body)) => Some(body.clone()),
      _ => None,
    },
    None => None,
  })
}

/// The value the paper gave a class variable (its one-level expansion), `None` when it is undefined, blank, or still
/// the class's `initial:n` placeholder.
fn variable_value(key: &str) -> Result<Option<Tokens>> {
  let Some(body) = macro_body(&variable(key))? else {
    return Ok(None);
  };
  if body
    .unlist_ref()
    .iter()
    .all(|t| t.get_catcode() == Catcode::SPACE)
    || macro_body(&initial_value(key))?.as_ref() == Some(&body)
  {
    return Ok(None);
  }
  Ok(Some(body))
}

/// Whether the authors carry the class's corresponding mark `\cor`.
fn marks_corresponding(authors: &Tokens) -> bool { authors.unlist_ref().contains(&T_CS!("\\cor")) }

LoadDefinitions!({
  InputDefinitions!("aero", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  // (the class's placeholders, as its keys' `initial:n` set them, :97-118)
  for (key, _) in FIELDS {
    Let!(initial_value(key), variable(key));
  }
  // The class's `\maketitle` (:128) prints the variables below; the lock refused it, and its replay would print them
  // a second time, as body paragraphs.
  Let!("\\lx@dropped@maketitle", "\\relax");
  DefMacro!("\\lx@aero@frontmatter", sub[()] {
    let mut out: Vec<Token> = Vec::new();
    let corresponding = variable_value("author")?.is_some_and(|authors| marks_corresponding(&authors));
    for (key, setter) in FIELDS {
      if let Some(value) = variable_value(key)? {
        // (the e-mail of the author the class's `\cor` marks)
        let setter = if *key == "email" && corresponding { r"\lx@aero@cor@email" } else { setter };
        out.push(T_CS!(setter));
        out.push(T_BEGIN!());
        if *key == "author" {
          for t in value.unlist() {
            if t == T_CS!("\\cor") {
              out.extend(TokenizeInternal!("\\lx@request@frontmatter@annotation[cor]{aero}").unlist());
            } else {
              out.push(t);
            }
          }
        } else {
          out.extend(value.unlist());
        }
        out.push(T_END!());
      }
    }
    Ok(Tokens::new(out))
  });
  AddToMacro!("\\lx@maketitle@begin", "\\lx@aero@frontmatter");
  // (:137 `\def\sep{\par …}`: each address its own paragraph, here its own line of the one affiliation every author has;
  // marked lines, `$^{1}$Univ A \sep $^{2}$Univ B`, go to the authors their marks name)
  DefMacro!("\\lx@aero@address{}", sub[(address)] {
    let address: Vec<Token> = address
      .unlist()
      .into_iter()
      .map(|t| if t == T_CS!("\\sep") { T_CS!("\\\\") } else { t })
      .collect();
    Ok(Invocation!(T_CS!("\\lx@add@affiliation@marked"), vec![
      Some(TokenizeInternal!("annotate=all")),
      Some(Tokens::new(address))
    ]))
  });
  DefMacro!(
    "\\lx@aero@email{}",
    "\\lx@add@contact@placed[role=email]{}{#1}"
  );
  // (:92 `\newcommand{\cor}{{\rm (\Letter)}}` after the corresponding author's name: a mark requesting the e-mail, as
  // elsarticle's `\corref` requests its `\cortext`, not name text; 2609.23496's `A. K. de Almeida Jr$^{1,2}$\cor{}`)
  DefMacro!(
    "\\lx@aero@cor@email{}",
    "\\lx@add@email[label={cor:aero}]{#1}"
  );
  // (:145 `{\def\sep{~·\ \ } …}`; a `\\` ending the list, 2610.12827's, ends the class's last line and prints nothing)
  DefMacro!("\\lx@aero@keywords{}", sub[(keywords)] {
    let mut keywords = keywords.unlist();
    while keywords
      .last()
      .is_some_and(|t| t.get_catcode() == Catcode::SPACE || *t == T_CS!("\\\\"))
    {
      keywords.pop();
    }
    let mut out = TokenizeInternal!("\\lx@add@keywords{{\\def\\sep{~\\textperiodcentered\\ \\ }").unlist();
    out.extend(keywords);
    out.extend([T_END!(), T_END!()]);
    Ok(Tokens::new(out))
  });
});
