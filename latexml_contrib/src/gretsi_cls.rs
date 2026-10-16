//! gretsi.cls, the class of the French GRETSI conference (not in TeX Live; papers ship it). Two releases, whose
//! frontmatter commands conflict, so the binding follows the paper's copy (`shipped_class_text`), and v1.3 where there
//! is none:
//! - v1.2 (2008/12/15; 45 of 62 papers, 1001.1912's copy): `\titre`, `\auteur` (= `\author`) of `\coord{first}{last}
//!   {mark}` names, `\adresse` (= `\address`) of `\affil{mark}{lines}`, `\email`, the abstracts `\resumefrancais`/
//!   `\frenchabstract` and `\resumeanglais`/`\englishabstract`, and `\legende` for captions.
//! - v1.3 (2023/01/25; 2304.10248's copy): `\titre`, `\auteurs` (= `\author`) of `\auteur{first}{last}{email}{mark}`
//!   names, `\affils` of `\affil{mark}{lines}`, and the abstracts `\resume` (French) and `\abstract` (English).
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
  // The French abstract, then the English one, each as the class heads it ("Résumé –", "Abstract –"), the English
  // one in English.
  DefMacro!(
    "\\lx@gretsi@resume{}",
    "\\lx@add@frontmatter{ltx:abstract}[name={R\\'esum\\'e}]{#1}"
  );
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
    Let!("\\resumefrancais", "\\lx@gretsi@resume");
    DefMacro!(
      "\\resumeanglais{}",
      "\\lx@add@frontmatter{ltx:abstract}[name={Abstract}]{\\begin{texteenanglais}#1\\end{texteenanglais}}"
    );
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
      let mut calls = vec![T_CS!("\\lx@clear@creators")];
      let rewritten = rewrite_calls(names.unlist_ref(), &T_CS!("\\auteur"), 4, &|args| {
        let mut call = vec![T_CS!("\\lx@add@authors@append"), T_BEGIN!()];
        call.extend(marked_name(&args[0], &args[1], &args[3]));
        call.push(T_END!());
        if !is_blank(&args[2]) {
          call.push(T_CS!("\\lx@add@email"));
          call.push(T_BEGIN!());
          call.extend_from_slice(&args[2]);
          call.push(T_END!());
        }
        call
      });
      // (only the calls: the spaces and comments between them are the layout's)
      let mut depth = 0usize;
      let mut in_call = false;
      for t in rewritten {
        if t == T_CS!("\\lx@add@authors@append") || t == T_CS!("\\lx@add@email") {
          in_call = true;
        }
        if in_call {
          calls.push(t);
          match t.get_catcode() {
            Catcode::BEGIN => depth += 1,
            Catcode::END => {
              depth -= 1;
              if depth == 0 {
                in_call = false;
              }
            },
            _ => {},
          }
        }
      }
      Ok(Tokens::new(calls))
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
      "\\resume{}",
      "\\lx@gretsi@resume{\\selectlanguage{french}#1}"
    );
    DefMacro!(
      "\\abstract{}",
      "\\lx@add@frontmatter{ltx:abstract}[name={Abstract}]{\\selectlanguage{english}#1}"
    );
  }
});
