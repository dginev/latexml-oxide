//! gretsi.cls, the class of the French GRETSI conference (not in TeX Live; papers ship it). Two releases, whose
//! frontmatter commands conflict, so the binding follows the paper's copy (`shipped_class_text`), and v1.3 where there
//! is none:
//! - v1.2 (2008/12/15; 45 of 62 papers, 1001.1912's copy): `\titre`, `\auteur` (= `\author`) of `\coord{first}{last}
//!   {mark}` names, `\adresse` (= `\address`) of `\affil{mark}{lines}`, `\email`, the abstracts `\resumefrancais`/
//!   `\frenchabstract` and `\resumeanglais`/`\englishabstract`, and `\legende` for captions.
//! - v1.3 (2023/01/25; 2304.10248's copy): `\titre`, `\auteurs` (= `\author`) of `\auteur{first}{last}{email}{mark}`
//!   names, `\affils` of `\affil{mark}{lines}`, and the abstracts `\resume` (French) and `\abstract` (English).
//!
//! Both store the abstracts and print them under the title, the French one first, whatever order the paper gives
//! them in.
use latexml_package::prelude::*;

use crate::shipped_class::{class_code_contains, class_option_given, shipped_class_text};

/// The calls of `cs` in `tokens`, each with its `nargs` arguments (a braced group's content, or a single token, as TeX
/// reads an undelimited argument), replaced by what `rewrite` makes of them; every other token is kept.
fn rewrite_calls(
  tokens: &[Token],
  cs: &Token,
  nargs: usize,
  rewrite: &dyn Fn(&[Vec<Token>]) -> Vec<Token>,
) -> Vec<Token> {
  let mut out = Vec::new();
  let mut i = 0;
  while i < tokens.len() {
    if tokens[i] != *cs {
      out.push(tokens[i]);
      i += 1;
      continue;
    }
    let start = i;
    i += 1;
    let mut args = Vec::new();
    while args.len() < nargs && i < tokens.len() {
      while i < tokens.len() && tokens[i].get_catcode() == Catcode::SPACE {
        i += 1;
      }
      if i >= tokens.len() {
        break;
      }
      if tokens[i].get_catcode() == Catcode::BEGIN {
        let mut depth = 0usize;
        let open = i;
        while i < tokens.len() {
          match tokens[i].get_catcode() {
            Catcode::BEGIN => depth += 1,
            Catcode::END => {
              depth -= 1;
              if depth == 0 {
                break;
              }
            },
            _ => {},
          }
          i += 1;
        }
        args.push(tokens[open + 1..i.min(tokens.len())].to_vec());
        i += 1;
      } else {
        args.push(vec![tokens[i]]);
        i += 1;
      }
    }
    if args.len() == nargs {
      out.extend(rewrite(&args));
    } else {
      // (a call short of its arguments: kept as written, for TeX to complain about)
      out.extend_from_slice(&tokens[start..i.min(tokens.len())]);
    }
  }
  out
}

fn is_blank(tokens: &[Token]) -> bool { tokens.iter().all(|t| t.get_catcode() == Catcode::SPACE) }

/// `first \textsc{last}\textsuperscript{mark}`: a name as both releases set it (v1.2 `\coord`, v1.3 `\auteur`), its
/// affiliation mark a superscript the author-line parsing reads as a request for the affiliation so labelled (the
/// class's `\aff{mark}` = `\textsuperscript{\footnotesize mark}`); none when the mark is empty (the template's "same
/// address for all authors" form).
fn marked_name(first: &[Token], last: &[Token], mark: &[Token]) -> Vec<Token> {
  let mut out = first.to_vec();
  out.push(T_SPACE!());
  out.push(T_CS!("\\textsc"));
  out.push(T_BEGIN!());
  out.extend_from_slice(last);
  out.push(T_END!());
  if !is_blank(mark) {
    out.push(T_CS!("\\textsuperscript"));
    out.push(T_BEGIN!());
    out.extend_from_slice(mark);
    out.push(T_END!());
  }
  out
}

/// The affiliations of an `\adresse` (v1.2) or `\affils` (v1.3) block: each `\affil{mark}{lines}` (both releases:
/// `\\[5pt]\aff{mark}lines`, a centred line under the names) an affiliation labelled by its mark, for the authors whose
/// names carry it (`\lx@add@affiliation@marked`, its lines the marked ones); an unmarked one is every author's. Text
/// outside the `\affil` calls is an affiliation of the authors before it.
fn affiliation_calls(block: &[Token]) -> Vec<Token> {
  let affils = std::cell::RefCell::new(Vec::new());
  let rest = rewrite_calls(block, &T_CS!("\\affil"), 2, &|args| {
    affils.borrow_mut().push((args[0].clone(), args[1].clone()));
    Vec::new()
  });
  let mut out = Vec::new();
  let mut marked = Vec::new();
  for (mark, lines) in affils.into_inner() {
    if is_blank(&mark) {
      out.extend(TokenizeInternal!("\\lx@add@affiliation[annotate=all]").unlist());
      out.push(T_BEGIN!());
      out.extend(lines);
      out.push(T_END!());
    } else {
      if !marked.is_empty() {
        marked.push(T_CS!("\\\\"));
      }
      marked.extend([T_CS!("\\textsuperscript"), T_BEGIN!()]);
      marked.extend(mark);
      marked.push(T_END!());
      marked.extend(lines);
    }
  }
  if !marked.is_empty() {
    out.push(T_CS!("\\lx@add@affiliation@marked"));
    out.push(T_BEGIN!());
    out.extend(marked);
    out.push(T_END!());
  }
  // (a bare `\\` or space between the calls is the layout's)
  if rest
    .iter()
    .any(|t| t.get_catcode() != Catcode::SPACE && *t != T_CS!("\\\\"))
  {
    out.push(T_CS!("\\lx@add@affiliation"));
    out.push(T_BEGIN!());
    out.extend(rest);
    out.push(T_END!());
  }
  out
}

/// The authors of a v1.3 `\auteurs` block (:96-97, `\auteurs` = `\author` of `\auteur{first}{last}{email}{mark}` names,
/// `\auteur{Mohamed El Amine}{Seddik}{mohamed.seddik@tii.ae}{1}`, 2304.10248): an author each, the mark requesting the
/// affiliation, the address the author's e-mail. What the class prints between the calls is kept with the name before
/// it (before the first, with the first), past the spaces, punctuation and spacing that separate the names
/// (2508.03934's `\auteur{…}{1,2},`), which are the list's.
fn auteurs_calls(names: &[Token]) -> Vec<Token> {
  let marker = T_CS!("\\lx@gretsi@auteur");
  let rewritten = rewrite_calls(names, &T_CS!("\\auteur"), 4, &|args| {
    let mut call = vec![marker];
    for arg in args {
      call.push(T_BEGIN!());
      call.extend_from_slice(arg);
      call.push(T_END!());
    }
    call
  });
  // each author's arguments, and the text after its call (`lead`: before the first)
  let mut authors: Vec<(Vec<Vec<Token>>, Vec<Token>)> = Vec::new();
  let mut lead: Vec<Token> = Vec::new();
  let mut i = 0;
  while i < rewritten.len() {
    if rewritten[i] == marker {
      i += 1;
      let mut args = Vec::new();
      for _ in 0..4 {
        let end = group_end(&rewritten, i);
        args.push(rewritten[i + 1..end].to_vec());
        i = end + 1;
      }
      authors.push((args, Vec::new()));
      continue;
    }
    match authors.last_mut() {
      Some((_, after)) => after.push(rewritten[i]),
      None => lead.push(rewritten[i]),
    }
    i += 1;
  }
  let lead = without_separators(&lead);
  let mut calls = vec![T_CS!("\\lx@clear@creators")];
  for (n, (args, after)) in authors.iter().enumerate() {
    calls.extend([T_CS!("\\lx@add@authors@append"), T_BEGIN!()]);
    if n == 0 && !lead.is_empty() {
      calls.extend_from_slice(&lead);
      calls.push(T_SPACE!());
    }
    calls.extend(marked_name(&args[0], &args[1], &args[3]));
    let after = without_separators(after);
    if !after.is_empty() {
      calls.push(T_SPACE!());
      calls.extend(after);
    }
    calls.push(T_END!());
    if !is_blank(&args[2]) {
      calls.extend([T_CS!("\\lx@add@email"), T_BEGIN!()]);
      calls.extend_from_slice(&args[2]);
      calls.push(T_END!());
    }
  }
  calls
}

/// The index of the `}` closing the group `tokens[open]` opens (the end of `tokens` if unclosed).
fn group_end(tokens: &[Token], open: usize) -> usize {
  let mut depth = 0usize;
  for (i, t) in tokens.iter().enumerate().skip(open) {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth = depth.saturating_sub(1);
        if depth == 0 {
          return i;
        }
      },
      _ => {},
    }
  }
  tokens.len()
}

/// `tokens` past the separators of a name list at either end (spaces and comments, `,`, `;`, `~`, `\and`, `\\`, `\quad`,
/// `\qquad`, `\hfill`, `\hspace{…}`): none when it is only those.
fn without_separators(tokens: &[Token]) -> Vec<Token> {
  // each unit of `tokens` (a token, or `\hspace` with its argument), and whether it separates
  let mut units: Vec<(usize, usize, bool)> = Vec::new();
  let mut i = 0;
  while i < tokens.len() {
    let t = tokens[i];
    let end = if t == T_CS!("\\hspace") {
      let mut j = i + 1;
      if tokens.get(j) == Some(&T_OTHER!("*")) {
        j += 1;
      }
      match tokens.get(j) {
        Some(next) if next.get_catcode() == Catcode::BEGIN => group_end(tokens, j) + 1,
        _ => j,
      }
    } else {
      i + 1
    };
    let separates = matches!(t.get_catcode(), Catcode::SPACE | Catcode::COMMENT)
      || [
        T_OTHER!(","),
        T_OTHER!(";"),
        T_ACTIVE!('~'),
        T_CS!("\\and"),
        T_CS!("\\\\"),
        T_CS!("\\quad"),
        T_CS!("\\qquad"),
        T_CS!("\\hfill"),
        T_CS!("\\hspace"),
      ]
      .contains(&t);
    units.push((i, end.min(tokens.len()), separates));
    i = end;
  }
  let first = units.iter().position(|unit| !unit.2);
  let last = units.iter().rposition(|unit| !unit.2);
  match (first, last) {
    (Some(first), Some(last)) => tokens[units[first].0..units[last].1].to_vec(),
    _ => Vec::new(),
  }
}

LoadDefinitions!({
  let copy = shipped_class_text("gretsi")?;
  let v12 = copy
    .as_deref()
    .is_some_and(|text| class_code_contains(text, "\\newcommand{\\auteur}{\\author}"));
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  RequirePackage!("amsthm");
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");

  // Both releases: `\newcommand{\titre}{\title}` (v1.2 :156, v1.3 :94), 42 papers (1001.1912, 2304.10248).
  Let!("\\titre", "\\title");
  // Both releases store the abstracts (the last call of each wins) and print them under the title, the French one
  // first (v1.2 :219-224 `\@maketitlegretsi`, v1.3 `\maketitle`), each as the class heads it ("Résumé –", "Abstract
  // –"), the English one in English, the French one only when given (`\if@resfra`): the two abstracts in that order
  // when the title is made, whatever order the paper gives them in (2205.13867, 2206.05928, 2304.07234, 2411.07577).
  RawTeX!(
    r"\newif\if@resfra\@resfrafalse
\long\def\lx@gretsi@resume@text{}\long\def\lx@gretsi@abstract@text{}\long\def\lx@gretsi@empty{}
\long\def\lx@gretsi@setresume#1{\@resfratrue\long\gdef\lx@gretsi@resume@text{#1}}
\long\def\lx@gretsi@setabstract#1{\long\gdef\lx@gretsi@abstract@text{#1}}
\def\lx@gretsi@abstracts{%
\if@resfra\expandafter\lx@gretsi@resume\expandafter{\lx@gretsi@resume@text}\fi
\ifx\lx@gretsi@abstract@text\lx@gretsi@empty\else
\expandafter\lx@gretsi@abstract\expandafter{\lx@gretsi@abstract@text}\fi}"
  );
  AddToMacro!("\\lx@maketitle@begin", "\\lx@gretsi@abstracts");
  if v12 {
    // v1.2 :154 `\newcommand{\auteur}{\author}`, and :157 `\newcommand{\coord}[3]{#1 \textsc{#2}\aff{#3}}` the names
    // in it (`\auteur{\coord{Ziad}{Naja}{1}, \coord{Florence}{Alberge}{1}, …}`, 1001.1912; 39 papers call `\coord`,
    // in `\auteur` or the `\author` it stands for: 1512.06999, 1709.05937): the kernel's author list, split at its
    // commas, each name's mark requesting its affiliation.
    Let!("\\lx@gretsi@author", "\\author");
    DefMacro!("\\author[]{}", sub[(short, names)] {
      let names: Tokens = names;
      let rewritten = rewrite_calls(names.unlist_ref(), &T_CS!("\\coord"), 3, &|args| {
        marked_name(&args[0], &args[1], &args[2])
      });
      Ok(Invocation!(T_CS!("\\lx@gretsi@author"), vec![short, Some(Tokens::new(rewritten))]))
    });
    Let!("\\auteur", "\\author");
    // v1.2 :155 `\aff`, the mark itself (used bare in some address blocks).
    DefMacro!("\\aff{}", "\\textsuperscript{#1}");
    DefMacro!("\\coord{}{}{}", "#1 \\textsc{#2}\\aff{#3}");
    // v1.2 :159-160 `\long\def\adresse#1{\long\gdef\@adresse{#1}}`, :166 `\newcommand{\address}{\adresse}`: the
    // affiliation block under the names (14 papers: 1001.1912, 1311.3768, 2204.13557).
    DefMacro!("\\adresse{}", sub[(block)] {
      let block: Tokens = block;
      Ok(Tokens::new(affiliation_calls(block.unlist_ref())))
    });
    Let!("\\address", "\\adresse");
    // v1.2 :106-122 `\resumefrancais`, `\resumeanglais` (the latter in `{texteenanglais}`), and their aliases
    // `\frenchabstract`, `\englishabstract` (20/19 papers: 0706.1914, 1311.3768; 1512.06999, 2206.05928).
    DefMacro!(
      "\\lx@gretsi@resume{}",
      "\\lx@add@frontmatter{ltx:abstract}[name={R\\'esum\\'e}]{#1}"
    );
    DefMacro!(
      "\\lx@gretsi@abstract{}",
      "\\lx@add@frontmatter{ltx:abstract}[name={Abstract}]{\\begin{texteenanglais}#1\\end{texteenanglais}}"
    );
    Let!("\\resumefrancais", "\\lx@gretsi@setresume");
    Let!("\\resumeanglais", "\\lx@gretsi@setabstract");
    Let!("\\frenchabstract", "\\resumefrancais");
    Let!("\\englishabstract", "\\resumeanglais");
    // v1.2 :22-58: at `\begin{document}`, with french.sty `{texteenanglais}` is `\english` and `\legende` an emphasized
    // caption; with babel the text is French, `{texteenanglais}` English and `\legende` the caption (8 papers:
    // 1311.3768, 1503.06716); without either the caption alone.
    RawTeX!(
      r"\AtBeginDocument{%
\@ifpackageloaded{french}{%
\newenvironment{texteenanglais}{\english}{\endenglish}%
\newcommand{\legende}[1]{\caption{\emph{#1}}}%
}{%
\@ifpackageloaded{babel}{%
\selectlanguage{french}%
\newenvironment{texteenanglais}{\selectlanguage{english}}{\selectlanguage{french}}%
\newcommand{\legende}[1]{\caption{#1}}%
}{%
\newenvironment{texteenanglais}{}{}%
\newcommand{\legende}[1]{\caption{#1}}%
}}}"
    );
  } else {
    // v1.3 :35-75: the packages the class loads, babel with French the text's language unless the `english` option
    // asks otherwise.
    RequirePackage!("times");
    RequirePackage!("graphicx");
    RequirePackage!("booktabs");
    RequirePackage!("babel", options => vec!["french".to_string(), "english".to_string()]);
    let french = !class_option_given("english")?;
    if french {
      RequirePackage!("algorithm2e", options => vec!["ruled".to_string(), "french".to_string(), "frenchkw".to_string()]);
      RawTeX!(r"\selectlanguage{french}");
    } else {
      RequirePackage!("algorithm2e", options => vec!["ruled".to_string()]);
      RawTeX!(r"\selectlanguage{english}");
    }
    // v1.3 :96 `\newcommand{\auteurs}{\author}` of :97 `\auteur[4]{\href{mailto:#3}{#1 \textsc{#2}}\textsuperscript{#4}
    // \hspace{1em}}` names (`\auteur{Mohamed El Amine}{Seddik}{mohamed.seddik@tii.ae}{1}`, 2304.10248): an author
    // each, the mark requesting the affiliation, the address the author's e-mail.
    DefMacro!("\\auteurs{}", sub[(names)] {
      let names: Tokens = names;
      Ok(Tokens::new(auteurs_calls(names.unlist_ref())))
    });
    DefMacro!(
      "\\auteur{}{}{}{}",
      "\\href{mailto:#3}{#1 \\textsc{#2}}\\textsuperscript{#4}\\hspace{1em}"
    );
    // v1.3 :98-99 `\long\def\affils#1{\long\gdef\@affils{#1}}` of `\affil{mark}{lines}`.
    DefMacro!("\\affils{}", sub[(block)] {
      let block: Tokens = block;
      Ok(Tokens::new(affiliation_calls(block.unlist_ref())))
    });
    // v1.3 :78-87 `\resume{…}` (French) and `\abstract{…}` (English), each set in its language at `\maketitle`.
    DefMacro!(
      "\\lx@gretsi@resume{}",
      "\\lx@add@frontmatter{ltx:abstract}[name={R\\'esum\\'e}]{\\selectlanguage{french}#1}"
    );
    DefMacro!(
      "\\lx@gretsi@abstract{}",
      "\\lx@add@frontmatter{ltx:abstract}[name={Abstract}]{\\selectlanguage{english}#1}"
    );
    Let!("\\resume", "\\lx@gretsi@setresume");
    Let!("\\abstract", "\\lx@gretsi@setabstract");
  }
});
