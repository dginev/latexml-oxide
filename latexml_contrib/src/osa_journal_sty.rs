//! The Optical Society of America's submission styles for the article class: `opex3.sty` (Optics Express, v1.0
//! 2003), `ol2.sty` (Optics Letters, v1.0 2006), `osajnl.sty` (the OSA journals, v2.0 2004) and its successor
//! `osajnl2.sty` (v2.1 2006), loaded as `\documentclass{article}\usepackage{opex3}`. Not in TeX Live and often not
//! shipped, so the papers stopped at `\address` (759 run-336 papers: 1006.2188, 0901.2165, 0901.2997, 0901.2793); a
//! shipped copy, read raw, typeset the title, authors and addresses into the text, leaving no title or authors. They
//! share one document syntax (cited from the copies in 0901.2165 (opex3.sty:43-67), 0901.2997 (ol2.sty:38-89),
//! 0707.0735 (osajnl.sty:51-90), 0901.2793 (osajnl2.sty:47-86)): `\title`, `\author` (the names, marked `$^1$` by
//! affiliation), `\address`/`\affil` (the affiliations, marked alike), `\email`, `\homepage`, `\ocis` (`\pacs`), an
//! `{abstract}`, the reference list as `{thebibliography}` or `{References}`, and the journal abbreviations of the
//! references; one binding serves them, routing the frontmatter to the frontmatter API. Rust-only: Perl has no OSA
//! binding.
use latexml_package::prelude::*;

/// The styles' common definitions; `name` is the style loaded.
pub(crate) fn osa_journal_style(name: &str) -> Result<()> {
  // opex3.sty:27-33, ol2.sty:22-26, osajnl.sty:32-39, osajnl2.sty:32-35: the packages loaded (the page geometry and
  // fonts aside)
  RequirePackage!("color");
  RequirePackage!("graphicx");
  match name {
    "opex3" => {},
    // osajnl.sty:32 `\RequirePackage{overcite}`, its citations as superscripts
    "osajnl" => {
      RequirePackage!("overcite");
    },
    _ => {
      RequirePackage!("cite");
    },
  }
  // opex3.sty:55 `\newcommand\url[1]{\textcolor{blue}{\underline{#1}}}`: a URL, the link here
  RequirePackage!("url");
  // The styles' names and formats: "Fig." (opex3.sty:84, ol2.sty:91, osajnl.sty:93, osajnl2.sty:89), "Table"
  // (ol2.sty:93, osajnl.sty:95, osajnl2.sty:91), a caption "Fig. 1. text" (`\@makecaption`'s `#1. #2`, opex3.sty:94,
  // ol2.sty:98, osajnl.sty:104, osajnl2.sty:100), the references headed "References and links" in opex3 (:86),
  // "References" in the others (ol2.sty:122, osajnl.sty:115, osajnl2.sty:111); headings numbered "1." (`\@seccntformat`,
  // opex3.sty:156, ol2.sty:159, osajnl.sty:153, osajnl2.sty:149), subsections "1.1" in opex3 (:157-158), "A" in ol2
  // (:160-161), "1.A" in osajnl and osajnl2 (osajnl.sty:154-155, osajnl2.sty:150-151). (The `{abstract}` label, opex3's
  // "Abstract:" (:70), is the kernel's abstract name; the copyright line it adds, :71-72, is the publisher's, not
  // kept.)
  DefMacro!("\\figurename", "Fig.");
  DefMacro!("\\tablename", "Table");
  if name == "opex3" {
    DefMacro!("\\refname", "References and links");
  } else {
    DefMacro!("\\refname", "References");
  }
  DefMacro!(
    "\\format@title@figure{}",
    "\\lx@tag[][. ]{\\lx@fnum@@{figure}}#1"
  );
  DefMacro!(
    "\\format@title@table{}",
    "\\lx@tag[][. ]{\\lx@fnum@@{table}}#1"
  );
  DefMacro!(
    "\\format@title@section{}",
    "\\lx@tag[][.\\space]{\\thesection}#1"
  );
  DefMacro!(
    "\\format@title@subsection{}",
    "\\lx@tag[][.\\space]{\\thesubsection}#1"
  );
  DefMacro!(
    "\\format@title@subsubsection{}",
    "\\lx@tag[][.\\space]{\\thesubsubsection}#1"
  );
  match name {
    "opex3" => {
      RawTeX!(
        "\\renewcommand\\thesubsection{\\thesection.\\arabic{subsection}}\\renewcommand\\thesubsubsection{\\thesubsection.\\arabic{subsubsection}}"
      );
    },
    "ol2" => {
      RawTeX!(
        "\\renewcommand\\thesubsection{\\Alph{subsection}}\\renewcommand\\thesubsubsection{\\arabic{subsubsection}}"
      );
    },
    _ => {
      RawTeX!(
        "\\renewcommand\\thesubsection{\\thesection.\\Alph{subsection}}\\renewcommand\\thesubsubsection{\\thesubsection.\\arabic{subsubsection}}"
      );
    },
  }
  // The kernel's `\title` collects the title, and `\maketitle` (blanked by ol2.sty:87 and osajnl2.sty:84, where
  // `\title` typesets at once) makes the frontmatter as anywhere.
  // opex3.sty:46 `\OEauthor` (ol2.sty:41, osajnl2.sty:50 `\OSAJNLauthor`), `\let\author` to it (opex3.sty:63): each
  // call typesets its names where it stands, so each adds its authors (`\author{D. L. Boiko}\address{…}\email{…}
  // \author{N. J. Gunther}…`, 0906.4563), split as the article `\author` splits them, their `$^n$` marks requesting the
  // affiliations so labelled; a footnote symbol in a mark is a mark of its own (`S. Lacour$^{1,2*}$` is marked 1, 2
  // and *, 0901.2165). `\@author` is the names, as the kernel's `\author` keeps them. Locked as the kernel's is.
  DefMacro!("\\author{}", sub[(names)] {
    let names: Tokens = names;
    let mut call = TokenizeInternal!(r"\def\@author").unlist();
    call.push(T_BEGIN!());
    call.extend(names.unlist_ref().iter().copied());
    call.push(T_END!());
    // (the group's start, for the contacts placed after it: `\lx@frontmatter@group@start`)
    call.push(T_CS!("\\lx@frontmatter@group@start"));
    call.extend(
      Invocation!(T_CS!("\\lx@add@authors@list"), vec![Some(marks_before_commas(
        symbol_marks_apart(names)
      ))])
      .unlist(),
    );
    Ok(Tokens::new(call))
  }, locked => true);
  // opex3.sty:64-65 `\let\affil\OEaddress \let\address\OEaddress` (ol2.sty:66-67, osajnl2.sty:75-76; ol2.sty:88 and
  // osajnl2.sty:85 also `\def\affiliation{\address}`): the affiliations, a line each, labelled by the `$^n$` marks the
  // names carry (`\address{$^1$LESIA, … \\ $^2$School of Physics, …}`, 0901.2165), or the institution of the authors
  // before it (a second address of theirs too). An email line in it, a line or the piece of one from an inner mark on
  // (`$^*$Corresponding author: x@y`, 0705.1274, 1006.2188, 1204.5397), is the emails: see `email_line`.
  DefMacro!("\\address{}", sub[(lines)] {
    let mut affiliations: Vec<Token> = Vec::new();
    let mut emails: Vec<Token> = Vec::new();
    for line in top_level_lines(lines) {
      // (a piece marked with several labels a line of its own for each: its next piece is not part of it)
      let mut kept: Vec<Vec<Token>> = vec![Vec::new()];
      for piece in pieces_at_marks(line) {
        if let Some(calls) = email_line(&piece)? {
          emails.extend(calls);
          continue;
        }
        let mut copies = lines_per_label(piece);
        if copies.len() == 1 {
          kept.last_mut().into_iter().for_each(|k| k.append(&mut copies[0]));
        } else {
          kept.extend(copies);
          kept.push(Vec::new());
        }
      }
      for line in kept {
        if line.iter().any(|t| t.get_catcode() != Catcode::SPACE) {
          if !affiliations.is_empty() {
            affiliations.push(T_CS!("\\\\"));
          }
          affiliations.extend(line);
        }
      }
    }
    let mut calls = Vec::new();
    if !affiliations.is_empty() {
      calls.extend(
        Invocation!(T_CS!("\\lx@add@affiliation@marked"), vec![
          Some(TokenizeInternal!("annotate=run")),
          Some(Tokens::new(affiliations))
        ])
        .unlist(),
      );
    }
    calls.extend(emails);
    Ok(Tokens::new(calls))
  });
  Let!("\\affil", "\\address");
  Let!("\\affiliation", "\\address");
  // opex3.sty:52 `\OEemail`: the email line, text often labelled and marked (`\email{$^*$Corresponding author: …}`,
  // 0901.2165), so not the verbatim address `\lx@add@email` reads: see `email_line`. Any other is the block's email,
  // placed by the kernel.
  DefMacro!("\\email{}", sub[(line)] {
    let line: Tokens = line;
    Ok(match email_line(line.unlist_ref())? {
      Some(calls) => Tokens::new(calls),
      None => Invocation!(T_CS!("\\lx@add@contact@placed"), vec![
        Some(TokenizeInternal!("role=email")),
        Some(Tokens!()),
        Some(line)
      ]),
    })
  });
  // opex3.sty:57 `\homepage`: the group's or block's web page, placed as an email is (1004.2645's institute page was
  // the last author's)
  DefMacro!(
    "\\homepage{}",
    "\\lx@add@contact@placed[role=url]{}{\\url{#1}}"
  );
  // opex3.sty:60, :67 `\let\pacs\ocis`: the paper's OCIS (Optics Classification and Indexing Scheme) codes
  DefMacro!(
    "\\ocis{}",
    "\\lx@add@classification[scheme=ocis,name={OCIS codes:~}]{#1}"
  );
  Let!("\\pacs", "\\ocis");
  // opex3.sty:75-78 `{abstract*}`, the abstract of a paper exempt from the copyright line `{abstract}` adds
  DefMacro!(T_CS!("\\begin{abstract*}"), None, "\\begin{abstract}");
  DefMacro!(T_CS!("\\end{abstract*}"), None, "\\end{abstract}");
  // The reference lists the styles define with both ends: `{References}` (opex3.sty:148, :151), `{OEReflinks}`
  // (:147, :150), `{OSAJNLReferences}` (ol2.sty:151, :154; osajnl2.sty:140, :144). (`\OSAJNLReflinks` is closed by an
  // `\endOSAJNLlinks` and `\references` by nothing, ol2.sty:152, :155, osajnl2.sty:143: neither is an environment.)
  for env in ["References", "OEReflinks", "OSAJNLReferences"] {
    DefMacro!(
      T_CS!(&format!("\\begin{{{env}}}")),
      None,
      "\\begin{thebibliography}{}"
    );
    DefMacro!(
      T_CS!(&format!("\\end{{{env}}}")),
      None,
      "\\end{thebibliography}"
    );
  }
  // ol2.sty:89, osajnl.sty:90, osajnl2.sty:86 `\def\onlinecite{\citeonline}` (cite.sty's citation without brackets)
  if name != "opex3" {
    Let!("\\onlinecite", "\\citeonline");
  }
  // opex3.sty:237-273 (ol2.sty:240-276, osajnl2.sty:230-266 alike): the journal abbreviations of the references, the
  // ones every shipped version has. The style `\def`s them, so a paper's own `\def` or `\renewcommand` replaces one
  // (60 of 751 OSA papers) and a `\newcommand` keeps the style's, as in pdflatex (where it is an error too). The later
  // opex3.sty's additions (`\boe`, `\ome`, `\aop`, `\as`, `\jocn`, `\opn`, `\pr`; 1406.6489 :245-280) are left
  // out: no OSA paper cites them, and papers define those names for themselves (0909.4968 `\newcommand{\as}`,
  // 1104.3859 `\as` for arcseconds, 1104.0806 `\pr` for Phys. Rev.), which a binding's earlier definition would hide.
  for (cs, journal) in [
    ("\\opex", " Opt.\\ Express "),
    ("\\ao", " Appl.\\  Opt.\\ "),
    ("\\ap", " Appl.\\  Phys.\\ "),
    ("\\apa", " Appl.\\  Phys.\\ A "),
    ("\\apb", " Appl.\\  Phys.\\ B "),
    ("\\apl", " Appl.\\ Phys.\\ Lett.\\ "),
    ("\\apj", " Astrophys.\\ J.\\ "),
    ("\\bell", " Bell Syst.\\ Tech.\\ J.\\ "),
    ("\\jqe", " IEEE J.\\ Quantum Electron.\\ "),
    (
      "\\assp",
      " IEEE Trans.\\ Acoust.\\ Speech Signal Process.\\ ",
    ),
    ("\\aprop", " IEEE Trans.\\ Antennas Propag.\\ "),
    ("\\mtt", " IEEE Trans.\\ Microwave Theory Tech.\\ "),
    ("\\iovs", " Invest.\\ Ophthalmol.\\ Visual\\ Sci.\\ "),
    ("\\jcp", " J.\\ Chem.\\ Phys.\\ "),
    ("\\jmo", " J.\\ Mod.\\ Opt.\\ "),
    ("\\jon", "J.\\ Opt.\\ Netw.\\ "),
    ("\\josa", " J.\\ Opt.\\ Soc.\\ Am.\\ "),
    ("\\josaa", " J.\\ Opt.\\ Soc.\\ Am.\\ A "),
    ("\\josab", " J.\\ Opt.\\ Soc.\\ Am.\\ B "),
    ("\\jpp", " J.\\ Phys.\\ (Paris) "),
    ("\\nat", " Nature (London) "),
    ("\\oc", " Opt.\\ Commun.\\ "),
    ("\\ol", " Opt.\\ Lett.\\ "),
    ("\\pl", " Phys.\\ Lett.\\ "),
    ("\\pra", " Phys.\\ Rev.\\ A "),
    ("\\prb", " Phys.\\ Rev.\\ B "),
    ("\\prc", " Phys.\\ Rev.\\ C "),
    ("\\prd", " Phys.\\ Rev.\\ D "),
    ("\\pre", " Phys.\\ Rev.\\ E "),
    ("\\prl", " Phys.\\ Rev.\\ Lett.\\ "),
    ("\\rmp", " Rev.\\ Mod.\\ Phys.\\ "),
    ("\\pspie", " Proc.\\ SPIE\\ "),
    ("\\sjqe", " Sov.\\ J.\\ Quantum Electron.\\ "),
    ("\\vr", " Vision Res.\\ "),
    ("\\cleo", " {\\it Conference on Lasers and Electro-Optics }"),
    ("\\assl", " {\\it Advanced Solid State Lasers }"),
    ("\\tops", " Trends in Optics and Photonics "),
  ] {
    DefMacro!(T_CS!(cs), None, Tokenize!(journal));
  }
  Ok(())
}

/// Whether `t` is a footnote symbol as a mark writes it: `*`, `\ast`, the daggers, `\S`, `\P`, `\star`, `\#`.
fn is_symbol_mark(t: &Token) -> bool {
  *t == T_OTHER!("*")
    || [
      "\\ast",
      "\\dag",
      "\\dagger",
      "\\ddag",
      "\\ddagger",
      "\\S",
      "\\P",
      "\\star",
      "\\#",
      "\\sharp",
    ]
    .iter()
    .any(|cs| *t == T_CS!(*cs))
}

/// `names` with each mark written after the comma that ends its name moved before it: `E. Hirose,$ ^{1,4,*} $
/// K. Kawabe,$ ^{2} $` (0909.0010, 1501.05707, 0812.1160) marks Hirose and Kawabe, where the names split at the comma
/// gave each mark to the name after it; a bare symbol so (`Leibrandt,* Michael J. Thorpe`, 1104.1413) as a superscript.
fn marks_before_commas(names: Tokens) -> Tokens {
  let tokens = names.unlist();
  let mut out: Vec<Token> = Vec::with_capacity(tokens.len());
  let mut depth = 0usize;
  let mut i = 0;
  while i < tokens.len() {
    let t = tokens[i];
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    if depth == 0 && t == T_OTHER!(",") {
      let at = skip_spaces_at(&tokens, i + 1);
      // a math mark `$ ^{…} $`, its spaces in it
      if tokens
        .get(at)
        .is_some_and(|m| m.get_catcode() == Catcode::MATH)
        && tokens
          .get(skip_spaces_at(&tokens, at + 1))
          .is_some_and(|m| m.get_catcode() == Catcode::SUPER)
        && let Some(close) = tokens[at + 1..]
          .iter()
          .position(|m| m.get_catcode() == Catcode::MATH)
          .map(|n| at + 1 + n)
      {
        out.extend_from_slice(&tokens[at..=close]);
        out.push(t);
        i = close + 1;
        continue;
      }
      // a bare footnote symbol (`David R. Leibrandt,* Michael J. Thorpe`, 1104.1413), as the superscript it marks
      let symbols = tokens[at..]
        .iter()
        .take_while(|m| is_symbol_mark(m) && **m != T_CS!("\\#"))
        .count();
      if symbols > 0 {
        out.extend([T_MATH!(), T_SUPER!(), T_BEGIN!()]);
        out.extend_from_slice(&tokens[at..at + symbols]);
        out.extend([T_END!(), T_MATH!(), t]);
        i = at + symbols;
        continue;
      }
    }
    out.push(t);
    i += 1;
  }
  Tokens::new(out)
}

/// `names` with each footnote symbol written into a numbered superscript mark made a mark of its own: `$^{1,2*}$` as
/// `$^{1,2,*}$`, `$^{*1}$` as `$^{*,1}$`, so that it requests affiliations 1 and 2 and the line marked `*` (0901.2165,
/// 1011.1336, 1208.1364, 1310.5176).
fn symbol_marks_apart(names: Tokens) -> Tokens {
  let tokens = names.unlist();
  let mut out = Vec::with_capacity(tokens.len() + 4);
  let mut i = 0;
  while i < tokens.len() {
    let t = tokens[i];
    out.push(t);
    i += 1;
    let is_mark = t.get_catcode() == Catcode::SUPER || t == T_CS!("\\textsuperscript");
    if !is_mark
      || tokens
        .get(i)
        .is_none_or(|t| t.get_catcode() != Catcode::BEGIN)
    {
      continue;
    }
    // the mark's group, its symbols set apart from what comes before them
    let mut depth = 0usize;
    while i < tokens.len() {
      let u = tokens[i];
      match u.get_catcode() {
        Catcode::BEGIN => depth += 1,
        Catcode::END => depth -= 1,
        _ => {},
      }
      if depth == 1
        && is_symbol_mark(&u)
        && out.last().is_some_and(|last| {
          *last != T_OTHER!(",") && last.get_catcode() != Catcode::BEGIN && !is_symbol_mark(last)
        })
      {
        out.push(T_OTHER!(","));
      }
      out.push(u);
      i += 1;
      // and from what comes after them (`$^{*1}$`, `$^{\ast 2}$`: 1310.5176, 1506.01336)
      if depth == 1 && is_symbol_mark(&u) {
        let next = tokens[i..]
          .iter()
          .find(|t| t.get_catcode() != Catcode::SPACE);
        if next.is_some_and(|n| {
          *n != T_OTHER!(",") && n.get_catcode() != Catcode::END && !is_symbol_mark(n)
        }) {
          out.push(T_OTHER!(","));
        }
      }
      if depth == 0 {
        break;
      }
    }
  }
  Tokens::new(out)
}

/// An affiliation line marked with several labels (`$^{1,2}$Department …`, 1406.0824) as one line for each: an entry
/// takes one label (Perl's `\lx@set@frontmatter@label`, Base_Utility.pool.ltxml:568, as the kernel's), so the
/// author marked 2 had no affiliation.
fn lines_per_label(line: Vec<Token>) -> Vec<Vec<Token>> {
  let start = skip_spaces_at(&line, 0);
  let Some((operand, end)) =
    mark_at(&line, start).filter(|_| line[start].get_catcode() == Catcode::MATH)
  else {
    return vec![line];
  };
  let labels: Vec<&[Token]> = operand.split(|t| *t == T_OTHER!(",")).collect();
  if labels.len() < 2
    || labels
      .iter()
      .any(|label| label.iter().all(|t| t.get_catcode() == Catcode::SPACE))
  {
    return vec![line];
  }
  labels
    .into_iter()
    .map(|label| {
      let mut copy = vec![T_MATH!(), T_SUPER!(), T_BEGIN!()];
      copy.extend_from_slice(label);
      copy.extend([T_END!(), T_MATH!()]);
      copy.extend_from_slice(&line[end..]);
      copy
    })
    .collect()
}

/// The lines of `tokens` at its top-level `\\`.
fn top_level_lines(tokens: Tokens) -> Vec<Vec<Token>> {
  let mut lines = vec![Vec::new()];
  let mut depth = 0usize;
  for t in tokens.unlist() {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    // (a blank line between the lines too: 1109.5493)
    if depth == 0 && (t == T_CS!("\\\\") || t == T_CS!("\\par")) {
      lines.push(Vec::new());
    } else if let Some(line) = lines.last_mut() {
      line.push(t);
    }
  }
  lines
}

/// Whether `t` is the character `c`, whatever its category (babel's French makes `:` and `;` active: 1210.1049).
fn is_char(t: &Token, c: &str) -> bool {
  matches!(
    t.get_catcode(),
    Catcode::OTHER | Catcode::ACTIVE | Catcode::LETTER
  ) && t.with_str(|s| s == c)
}

/// Whether the tokens hold an `@`.
fn has_at(tokens: &[Token]) -> bool { tokens.iter().any(|t| t.with_str(|s| s == "@")) }

/// The first index from `i` that is not a space.
fn skip_spaces_at(tokens: &[Token], mut i: usize) -> usize {
  while tokens
    .get(i)
    .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
  {
    i += 1;
  }
  i
}

/// The index after the group opening at `open`, or after the one token there.
fn after_argument(tokens: &[Token], open: usize) -> Option<usize> {
  if tokens.get(open)?.get_catcode() != Catcode::BEGIN {
    return Some(open + 1);
  }
  let mut depth = 0usize;
  for (j, t) in tokens.iter().enumerate().skip(open) {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth == 0 {
          return Some(j + 1);
        }
      },
      _ => {},
    }
  }
  None
}

/// The operand of the mark at `i` in `tokens`, and the index after it: `$^…$`, `^…`,
/// `\textsuperscript{…}`, a footnote symbol in math (`$*$`, 1412.3226) or bare (`*peter.banzer@…`, `**3794…`,
/// 1004.2645, 2001.05892).
fn mark_at(tokens: &[Token], i: usize) -> Option<(Vec<Token>, usize)> {
  let t = tokens.get(i)?;
  let (operand, end): (Vec<Token>, usize) = if t.get_catcode() == Catcode::MATH {
    let close = i
      + 1
      + tokens[i + 1..]
        .iter()
        .position(|t| t.get_catcode() == Catcode::MATH)?;
    let inner = &tokens[i + 1..close];
    match inner.first() {
      Some(sup) if sup.get_catcode() == Catcode::SUPER => {
        let operand = &inner[1..];
        let operand = match operand {
          [open, rest @ .., close]
            if open.get_catcode() == Catcode::BEGIN && close.get_catcode() == Catcode::END =>
          {
            rest
          },
          _ => operand,
        };
        (operand.to_vec(), close + 1)
      },
      _ if !inner.is_empty() && inner.iter().all(is_symbol_mark) => (inner.to_vec(), close + 1),
      _ => return None,
    }
  } else if t.get_catcode() == Catcode::SUPER || *t == T_CS!("\\textsuperscript") {
    let open = skip_spaces_at(tokens, i + 1);
    let end = after_argument(tokens, open)?;
    let operand = if tokens[open].get_catcode() == Catcode::BEGIN {
      &tokens[open + 1..end - 1]
    } else {
      &tokens[open..end]
    };
    (operand.to_vec(), end)
  } else if is_symbol_mark(t) && *t != T_CS!("\\#") {
    // (bare, `\#` is the address's own character: `\email{\#able@x.org}`)
    let n = tokens[i..].iter().take_while(|t| is_symbol_mark(t)).count();
    (tokens[i..i + n].to_vec(), i + n)
  } else {
    return None;
  };
  Some((operand, end))
}

/// An email line: its mark, its label, its addresses.
struct EmailLine {
  mark:      Option<Vec<Token>>,
  label:     Vec<Token>,
  addresses: Vec<Address>,
}

/// One address of an email line: its mark, if any, and the address.
struct Address {
  mark:    Option<Vec<Token>>,
  address: Vec<Token>,
}

/// An email line (opex3.sty:52 `\OEemail`; or a line of an `\address`): an optional mark, an optional short label (a
/// few words ending in `:`, no command or math: `Corresponding author:`, `e-mail:`), then addresses separated by
/// commas or semicolons, each perhaps marked (`*a@x, **b@y`). `None` for anything else — an affiliation naming an
/// address in its text (`… Milano, Italy (stefano.longhi@polimi.it)`, 1411.7144) is no email line.
fn email_line_parts(line: &[Token]) -> Option<EmailLine> {
  if !has_at(line) {
    return None;
  }
  let start = skip_spaces_at(line, 0);
  let (line_mark, rest) = match mark_at(line, start) {
    Some((mark, end)) => (Some(mark), end),
    None => (None, start),
  };
  // the words before the first one naming an address are the label
  let mut words: Vec<(usize, usize)> = Vec::new();
  let mut i = skip_spaces_at(line, rest);
  while i < line.len() {
    let mut j = i;
    while j < line.len() && line[j].get_catcode() != Catcode::SPACE {
      j = if line[j].get_catcode() == Catcode::BEGIN {
        after_argument(line, j)?
      } else {
        j + 1
      };
    }
    words.push((i, j));
    i = skip_spaces_at(line, j);
  }
  let first = words.iter().position(|&(a, b)| has_at(&line[a..b]))?;
  let label_words = &words[..first];
  let label_ok = label_words.len() <= 4
    && label_words.iter().all(|&(a, b)| {
      line[a..b].iter().all(|t| {
        (matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER) || is_char(t, ":"))
          && t.with_str(|s| s != "(")
      })
    })
    && label_words
      .last()
      .is_none_or(|&(_, b)| is_char(&line[b - 1], ":"));
  if !label_ok {
    return None;
  }
  let mut label: Vec<Token> = Vec::new();
  for &(a, b) in label_words {
    label.extend_from_slice(&line[a..b]);
    label.push(T_SPACE!());
  }
  // the addresses, at the top-level commas and semicolons
  let tail = &line[words[first].0..];
  let mut items: Vec<Vec<Token>> = vec![Vec::new()];
  let mut depth = 0usize;
  for t in tail {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    if depth == 0 && (is_char(t, ",") || is_char(t, ";")) {
      items.push(Vec::new());
    } else if let Some(item) = items.last_mut() {
      item.push(*t);
    }
  }
  let mut addresses = Vec::new();
  for item in items {
    let Some(address) = address_item(&item) else {
      // a bare "and" between the last two (`$^2$a@x, and $^3$b@y`, 1406.0824)
      if is_and_word(&item) {
        continue;
      }
      return None;
    };
    addresses.push(address);
  }
  if addresses.is_empty() {
    return None;
  }
  Some(EmailLine {
    mark: line_mark,
    label,
    addresses,
  })
}

/// Whether `tokens` is the word "and", or only spaces.
fn is_and_word(tokens: &[Token]) -> bool {
  let word: String = tokens
    .iter()
    .filter(|t| t.get_catcode() != Catcode::SPACE)
    .map(|t| t.to_string())
    .collect();
  word.is_empty() || word == "and"
}

/// The declarations a style puts around an address (`{\color{blue}$^*$x@y}`, 1303.3906; `\it`): not its text.
fn skip_declarations(tokens: &[Token], mut i: usize) -> usize {
  loop {
    i = skip_spaces_at(tokens, i);
    match tokens.get(i) {
      Some(t) if *t == T_CS!("\\color") => {
        let Some(end) = after_argument(tokens, skip_spaces_at(tokens, i + 1)) else {
          return i;
        };
        i = end;
      },
      Some(t)
        if [
          "\\it",
          "\\bf",
          "\\rm",
          "\\sl",
          "\\tt",
          "\\sf",
          "\\em",
          "\\small",
          "\\footnotesize",
        ]
        .iter()
        .any(|cs| *t == T_CS!(*cs)) =>
      {
        i += 1;
      },
      _ => return i,
    }
  }
}

/// One address of an email line ([`email_line_parts`]): a mark before it (`$^*$`, `*`, `\textit{$^3$}` of 1306.0672),
/// or after it (`x@y$^\dag$`, 1003.2212), the address itself, one word, and a parenthesis after it (`x@y (A. Able)`),
/// set aside with the item's enclosing group and declarations.
fn address_item(item: &[Token]) -> Option<Address> {
  let mut tokens: Vec<Token> = item.to_vec();
  // the group the whole item is
  loop {
    let start = skip_spaces_at(&tokens, 0);
    let end = tokens
      .iter()
      .rposition(|t| t.get_catcode() != Catcode::SPACE)?
      + 1;
    if tokens[start].get_catcode() == Catcode::BEGIN && after_argument(&tokens, start) == Some(end)
    {
      tokens = tokens[start + 1..end - 1].to_vec();
    } else {
      break;
    }
  }
  let at = skip_declarations(&tokens, 0);
  let (mut mark, from) = match mark_at(&tokens, at) {
    Some((mark, end)) => (Some(mark), skip_spaces_at(&tokens, end)),
    // a mark inside a font command (`\textit{$^3$}yinjuan@…`)
    None => match tokens.get(at) {
      // (only with its braced argument: a bare `\textasteriskcentered able@x.edu` is the address's text, and a
      // command without a brace after it sliced backwards, a panic)
      Some(t)
        if t.get_catcode() == Catcode::CS
          && t.with_str(|s| s.starts_with("\\text") || s == "\\emph")
          && tokens
            .get(skip_spaces_at(&tokens, at + 1))
            .is_some_and(|t| t.get_catcode() == Catcode::BEGIN) =>
      {
        let open = skip_spaces_at(&tokens, at + 1);
        let end = after_argument(&tokens, open)?;
        let inner = &tokens[open + 1..end.saturating_sub(1)];
        match mark_at(inner, skip_spaces_at(inner, 0)) {
          Some((operand, e)) if skip_spaces_at(inner, e) == inner.len() => {
            (Some(operand), skip_spaces_at(&tokens, end))
          },
          _ => (None, at),
        }
      },
      _ => (None, at),
    },
  };
  let mut address = tokens[from..].to_vec();
  // a parenthesis after the address, naming its owner (`x@y (A. Able)`): no part of the address the contact links to
  if let Some(space) = address
    .iter()
    .position(|t| t.get_catcode() == Catcode::SPACE)
    && let Some(open) = address[space..]
      .iter()
      .position(|t| t.get_catcode() != Catcode::SPACE)
      .map(|n| space + n)
    && address[open] == T_OTHER!("(")
    && address
      .iter()
      .rposition(|t| t.get_catcode() != Catcode::SPACE)
      .is_some_and(|last| address[last] == T_OTHER!(")"))
  {
    address.truncate(space);
  }
  while address
    .last()
    .is_some_and(|t| t.get_catcode() == Catcode::SPACE || *t == T_OTHER!("."))
  {
    address.pop();
  }
  // a mark after the address (`\email{hyunchul.nha@qatar.tamu.edu$^\dag$}`, 1003.2212)
  if mark.is_none()
    && let Some(open) = address
      .iter()
      .position(|t| t.get_catcode() == Catcode::MATH)
    && let Some((operand, end)) = mark_at(&address, open)
    && end == address.len()
  {
    mark = Some(operand);
    address.truncate(open);
  }
  // an address is one word that names one
  if !has_at(&address) || address.iter().any(|t| t.get_catcode() == Catcode::SPACE) {
    return None;
  }
  Some(Address { mark, address })
}

/// The calls of an email line ([`email_line_parts`]), the label each contact's name, each address placed by the kernel
/// (`\lx@add@contact@placed`): with the author carrying its mark, else the one its local part spells, else the group's
/// only author, else the block's.
fn email_line(line: &[Token]) -> Result<Option<Vec<Token>>> {
  let Some(EmailLine {
    mark: line_mark,
    label,
    addresses,
  }) = email_line_parts(line)
  else {
    return Ok(None);
  };
  let mut calls = Vec::new();
  for Address { mark, address } in addresses {
    let mut keyvals = TokenizeInternal!("role=email").unlist();
    if !label.is_empty() {
      keyvals.extend(TokenizeInternal!(",name=").unlist());
      keyvals.push(T_BEGIN!());
      keyvals.extend(label.iter().copied());
      keyvals.push(T_END!());
    }
    let mark = mark.or_else(|| line_mark.clone()).unwrap_or_default();
    calls.extend(
      Invocation!(T_CS!("\\lx@add@contact@placed"), vec![
        Some(Tokens::new(keyvals)),
        Some(Tokens::new(mark)),
        Some(Tokens::new(address))
      ])
      .unlist(),
    );
  }
  Ok(Some(calls))
}

/// The pieces of an `\address` line at its inner marks: `$^1$Institute … Switzerland $^*$Corresponding author: x@y`
/// (1204.5397) is an affiliation and an email line.
fn pieces_at_marks(line: Vec<Token>) -> Vec<Vec<Token>> {
  let mut pieces: Vec<Vec<Token>> = vec![Vec::new()];
  let mut depth = 0usize;
  let mut i = 0;
  while i < line.len() {
    let t = line[i];
    let starts_mark = depth == 0
      && (t.get_catcode() == Catcode::MATH
        && line
          .get(i + 1)
          .is_some_and(|n| n.get_catcode() == Catcode::SUPER)
        || t.get_catcode() == Catcode::SUPER
          && (i == 0 || line[i - 1].get_catcode() != Catcode::MATH)
        || t == T_CS!("\\textsuperscript"));
    // (a superscript glued to the letter or digit before it is no mark: `5$^{th}$ floor`)
    let glued = i > 0 && matches!(line[i - 1].get_catcode(), Catcode::LETTER | Catcode::OTHER);
    if starts_mark
      && !glued
      && pieces
        .last()
        .is_some_and(|p| p.iter().any(|t| t.get_catcode() != Catcode::SPACE))
    {
      pieces.push(Vec::new());
    }
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    if let Some(piece) = pieces.last_mut() {
      piece.push(t);
    }
    i += 1;
  }
  pieces
}

/// `opex3.sty`
pub mod opex3_sty {
  use super::*;
  LoadDefinitions!({
    osa_journal_style("opex3")?;
  });
}

/// `ol2.sty`
pub mod ol2_sty {
  use super::*;
  LoadDefinitions!({
    osa_journal_style("ol2")?;
  });
}

/// `osajnl.sty`
pub mod osajnl_sty {
  use super::*;
  LoadDefinitions!({
    osa_journal_style("osajnl")?;
  });
}

/// `osajnl2.sty`
pub mod osajnl2_sty {
  use super::*;
  LoadDefinitions!({
    osa_journal_style("osajnl2")?;
  });
}
