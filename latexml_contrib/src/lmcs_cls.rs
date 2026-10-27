//! Stub for lmcs.cls (Logical Methods in Computer Science journal class).
//!
//! lmcs.cls is NOT distributed in TeX Live (papers bundle it, but the corpus
//! copies don't reach our search path), so Perl LaTeXML — which ships no lmcs
//! binding — emits `Can't find binding for class lmcs (using OmniBus)` and
//! falls back to **OmniBus**. The base class is the one lmcs.cls:33 loads,
//! `\LoadClass[11pt,reqno]{amsart}` (OXIDIZED_DESIGN_DIVERGENCES #736: the paper's copy, read after the binding, finds
//! it loaded), with OmniBus, the base this binding had before, provided over it; and the class's own theorem set (lmcs.cls:696-761, below)
//! eagerly: a paper whose preamble does `\newtheorem{remark}[thm]{…}` and whose
//! body uses `\begin{thm}`/`\begin{lem}` shares the class's `thm` counter,
//! numbered within the section as pdflatex numbers it (OmniBus's lazy autoloads
//! once supplied `thm`/`lem` with document-wide numbers). Witness 1607.01886. The raw cls relies on pgfmath/tikz/XeTeXLinkBox for
//! ORCID-logo rendering (fails mid-load), so `\lmcsdoi`/`\lmcsheading`/
//! `\lmcsorcid` get content-preserving stubs below. Witness 2305.14448,
//! 2305.19985, 1607.04128 (graphicx).
use latexml_package::{package::omnibus_cls::provide_over_base_class, prelude::*};

LoadDefinitions!({
  LoadClass!("amsart", vec![s!("11pt"), s!("reqno")], Tokens!());
  provide_over_base_class()?;
  RequirePackage!("amsmath");
  RequirePackage!("amssymb");
  RequirePackage!("amsthm");
  // lmcs.cls L33 `\LoadClass[11pt,reqno]{amsart}`. Perl ships no lmcs binding,
  // so it loads OmniBus AND dependency-scans the raw lmcs.cls, which loads
  // amsart.cls.ltxml → ams_support.sty.ltxml. That is where the amsart-family
  // frontmatter macros (`\urladdr`, `\address`, `\email`, `\curraddr`) come
  // from. Without it, `\urladdr{\url{…}}` was undefined (witness 1709.06170,
  // RUST 1 → 0; Perl loads it via the amsart dep). ams_support is frontmatter
  // only — it does not pre-declare theorem envs; those come from the class's
  // theorem set below (1607.01886).
  RequirePackage!("ams_support");
  RequirePackage!("hyperref");
  // lmcs.cls L24 `\usepackage{helvet,cclicenses}` pulls graphicx in
  // transitively: cclicenses → `\RequirePackage{rotating}` (cclicenses.sty
  // L25) → `\RequirePackage{graphicx}` (rotating.sty L62). Perl ships no lmcs
  // binding, so it raw-loads lmcs.cls and gets graphicx that way; LMCS papers
  // therefore use `\includegraphics` WITHOUT their own `\usepackage{graphicx}`.
  // Our stub intercepts the raw cls (which fails mid-load on its tikz/
  // XeTeXLinkBox ORCID machinery), so the transitive graphicx never loaded
  // and `\includegraphics` was undefined where Perl is clean. Supply it
  // directly. Witness 1607.04128 (`\documentclass{lmcs}`, `\includegraphics`
  // with no explicit graphicx load): RUST 1 → 0.
  RequirePackage!("graphicx");
  // xcolor comes with tikz (pgfcore.sty:13); a paper's later `\usepackage[table]{xcolor}` still takes its options (a
  // repeat load applies the options the first lacked, content.rs `apply_new_options_on_reload`), so colortbl and
  // array `m{}`/`b{}` columns work. See ifacconf_cls.rs.
  // lmcs.cls:37-38 `\PassOptionsToPackage{inline}{enumitem}\RequirePackage{enumitem}` (1611.06929's copy): the inline
  // lists (`{enumerate*}`: 1611.06929, 1711.09946, 2007.10842, 2203.09501); the newer copies also pass `shortlabels`
  // (2003.01491's lmcs.cls:45-47, 2604.16477's :51-56), so `\begin{enumerate}[(a)]` numbers "(a)", "(b)". The binding
  // passes both, the union of the versions.
  RequirePackage!("enumitem", options => vec![s!("inline"), s!("shortlabels")]);
  RequirePackage!("etoolbox");
  // lmcs.cls:62-66, 399, 428: tikz (papers draw with it unloaded, 2609.11893), color, thmtools, xparse, mathtools
  // (`\coloneqq`, `\DeclarePairedDelimiter`).
  RequirePackage!("tikz");
  RequirePackage!("color");
  RequirePackage!("thmtools");
  RequirePackage!("xparse");
  RequirePackage!("mathtools");
  // lmcs.cls:696-761: the class's theorem environments, numbered together by section (`{exa}`, `{defi}` were
  // undefined, 2609.11893), with its `defC`/`thmC` styles for theorems carrying a citation (:731-758).
  RawTeX!(
    r"\theoremstyle{plain}
\newtheorem{thm}{Theorem}[section]\newtheorem{cor}[thm]{Corollary}\newtheorem{lem}[thm]{Lemma}
\newtheorem{slem}[thm]{Sublemma}\newtheorem{prop}[thm]{Proposition}
\theoremstyle{definition}
\newtheorem{asm}[thm]{Assumption}\newtheorem{rem}[thm]{Remark}\newtheorem{rems}[thm]{Remarks}
\newtheorem{exa}[thm]{Example}\newtheorem{exas}[thm]{Examples}\newtheorem{defi}[thm]{Definition}
\newtheorem{conv}[thm]{Convention}\newtheorem{conj}[thm]{Conjecture}\newtheorem{prob}[thm]{Problem}
\newtheorem{oprob}[thm]{Open Problem}\newtheorem{oprobs}[thm]{Open Problems}\newtheorem{algo}[thm]{Algorithm}
\newtheorem{obs}[thm]{Observation}\newtheorem{desc}[thm]{Description}\newtheorem{fact}[thm]{Fact}
\newtheorem{qu}[thm]{Question}\newtheorem{oqu}[thm]{Open Question}\newtheorem{pty}[thm]{Property}
\newtheorem{clm}[thm]{Claim}\newtheorem{nota}[thm]{Notation}\newtheorem{com}[thm]{Comment}
\newtheorem{coms}[thm]{Comments}
\newtheoremstyle{defC}{6pt}{6pt}{\normalfont}{}{\bfseries}{{\bfseries .}}{5pt plus 1pt minus 1pt}{\thmname{#1} \thmnumber{#2} \thmnote{\normalfont#3}}
\theoremstyle{defC}
\newtheorem{defiC}[thm]{Definition}\newtheorem{remC}[thm]{Remark}\newtheorem{exaC}[thm]{Example}
\newtheoremstyle{thmC}{6pt}{6pt}{\itshape}{}{\bfseries}{{\bfseries .}}{5pt plus 1pt minus 1pt}{\thmname{#1} \thmnumber{#2} \thmnote{\normalfont#3}}
\theoremstyle{thmC}
\newtheorem{thmC}[thm]{Theorem}\newtheorem{propC}[thm]{Proposition}\newtheorem{lemC}[thm]{Lemma}
\theoremstyle{plain}
\numberwithin{equation}{section}"
  );

  // LMCS publication metadata. Real macros assign internal counters
  // and set up running headers; for HTML rendering we just preserve
  // the args as named notes.
  DefMacro!(
    "\\lmcsdoi{}{}{}",
    "\\@add@frontmatter{ltx:note}[role=lmcs-doi]{Volume #1, Issue #2, Paper #3}"
  );
  // `\lmcsheading{doi}{pages}{line}{line}{submitted}{published}{extra}` (lmcs.cls:499 in 2306.08266's copy, :376 in
  // 1103.0510's LMCS.cls: seven arguments in every version) sets the first page's running head, page furniture but for
  // the two dates, which are the paper's: "Submitted Jun. 16, 2023", "Published Mar. 20, 2024" (2306.08266). Four
  // arguments were read, so the dates were left in the document's first paragraph.
  DefMacro!(
    "\\lmcsheading{}{}{}{}{}{}{}",
    "\\if\\relax\\detokenize{#5}\\relax\\else\\lx@add@date[role=submitted,name={Submitted }]{#5}\\fi\
     \\if\\relax\\detokenize{#6}\\relax\\else\\lx@add@date[role=published,name={Published }]{#6}\\fi"
  );
  // \lmcsorcid{orcid-id} — render as a plain link rather than the
  // tikz/XeTeXLinkBox logo construction. Within an `\author` it is the author's ORCID contact (below).
  DefMacro!("\\lmcsorcid{}", "\\href{https://orcid.org/#1}{ORCID:#1}");

  // Section-numbering and shortauthors/shorttitle helpers used by raw
  // cls header layout. Stub as no-op or pass-through.
  def_macro_noop("\\shorttitle{}")?;
  def_macro_noop("\\shortauthors{}")?;

  // `\dOi` placeholder produced by the raw cls when no \lmcsdoi was
  // declared. Stub to empty so it doesn't appear as red error text.
  def_macro_noop("\\dOi")?;

  // lmcs.cls (1809.04468's copy :252-280, :343; 1403.4094's :490-507), what papers call that the binding lacked (run
  // 340 first errors: `\titlecomment` 445, `\ACMCCS` 223, `\rsuper` 188, `{iteMize}` 22, `\amsclass` 14, `\qEd` 12;
  // witnesses 2103.15292, 1309.5822, 2103.16592, 1107.1351, 1205.4458, cs0510066):
  // - `\titlecomment` (the footnote to the title, :343 `\@footnotetext{\@settitlecomment}`) as a title note;
  // - `\ACMCCS` and `\amsclass`, printed under the abstract as "\ACMCCSname: …" and "\amsclassname: …" (2012 ACM
  //   CCS, 2010 Mathematics Subject Classification; 50 of 56 copies with them define the names), as classifications;
  // - `\rsuper`/`\lsuper`, the affiliation marks after and before an author's name (`\ifsuPer`, set by the class; the
  //   31 newer copies' braced `\MakeLowercase{#1}`, so `\rsuper{A,B}` is "a,b");
  // - `\qEd` (the class's QED box, :491-492) and `{iteMize}{label}`/`{desCription}` (:506-507): the enumerate
  //   package's lists whose items all carry the given label (`\begin{iteMize}{$\bullet$}`), or none; as `{list}`,
  //   since the enumitem the binding loads (as the 2015+ versions do) reads `[$\bullet$]` as a key and numbers the
  //   items instead.
  DefMacro!(
    "\\titlecomment{}",
    "\\@add@frontmatter{ltx:note}[role=titlenote]{#1}"
  );
  RawTeX!(
    r"\def\ACMCCSname{2012 ACM CCS}\def\amsclassname{2010 Mathematics Subject Classification}"
  );
  DefMacro!(
    "\\ACMCCS{}",
    "\\@add@frontmatter{ltx:classification}[scheme=ccs,name={\\ACMCCSname}]{#1}"
  );
  DefMacro!(
    "\\amsclass{}",
    "\\@add@frontmatter{ltx:classification}[scheme=AMS,name={\\amsclassname}]{#1}"
  );
  RawTeX!(
    r"\newif\ifsuPer \suPertrue
\def\rsuper#1{\ifsuPer${\,}^{\MakeLowercase{#1}}$\fi}
\def\lsuper#1{\llap{${}^{\MakeLowercase{#1}\,}$}}
\def\qEd{\ensuremath{\square}}
\newenvironment{iteMize}[1]{\begin{list}{#1}{}}{\end{list}}
\newenvironment{desCription}{\begin{list}{}{}}{\end{list}}"
  );

  // The title page (2306.08266's lmcs.cls:391-450, the 2023 v0.92 class; 1103.0510's LMCS.cls v0.8 keeps amsart's
  // `\author`): `\author[short]{Name}[a,b]` prints the name with the letters raised after it (`\rsuper`, :398-404), and
  // the addresses print as one list after all the names, each led by the next letter, a, b, c, … (`\@setaddresses`,
  // :416-445: `\lsuper{\alph{addressctr}}` when the paper has several authors and several addresses), its e-mail under
  // it. The letters are the link: a name's letters request the addresses printed with them (the `\inst` request of
  // llncs's binding, a frontmatter annotation, so the name is the name alone, FIDELITY_CHARTER §4), and an address is
  // labelled by its place in the list. The papers of the earlier classes write the letters themselves, `Name\rsuper a`
  // and `\address{{\lsuper a}Univ}` (1103.0510, cs0701140, 1202.6352's `{\lsuper{b,c}}`), which mean the same. A paper
  // whose names carry no letters keeps amsart's reading, an address the run of authors' it follows: its own author's
  // when written after each (2108.06944, 1703.06021), every author's when written after them all (2402.11987's one
  // address of two authors). Was: every address on the last author, the letters left as text in a paragraph of their
  // own (2306.08266's eight authors, 2402.09138). An ORCID written in the name (`\lmcsorcid`, :776-791) is the
  // author's ORCID contact, not "ORCID:…" in the name (2306.08266, 2402.09138). Each `\author` starts an author group,
  // so an e-mail that spells no author and follows no lettered address is its author's, as amsart prints it.
  DefMacro!("\\author[]{}[]", sub[(short, name, labels)] {
    let mut labelled = false;
    let mut content = lmcs_author_name(name.unlist_ref(), &mut labelled);
    if let Some(labels) = labels {
      content.extend(inst_requests(labels.unlist_ref(), &mut labelled));
    }
    if labelled {
      assign_value("lx_lmcs_labelled", Stored::Bool(true), Some(Scope::Global));
    }
    let stored = short.clone().unwrap_or_else(|| name.clone());
    Ok(Invocation!(T_CS!("\\lx@lmcs@author"), vec![
      Some(short.unwrap_or_default()),
      Some(name),
      Some(stored),
      Some(Tokens::new(content))
    ]))
  });
  DefMacro!(
    "\\lx@lmcs@author{}{}{}{}",
    "\\def\\@shortauthor{#1}\\def\\@author{#2}\\lx@ams@addto@authors{#3}{#2}\
     \\lx@frontmatter@group@start\\lx@add@author{#4}"
  );
  // An address is labelled by the letters written before it (`{\lsuper a}`, `\lsuper{b,c}`), else, in a paper whose
  // names carry letters, by its place in the list (2306.08266's `[f,g]` naming its sixth and seventh addresses,
  // 2001.03894's `\rsuper{a,c}` and unlettered addresses). Its letter is the mark an e-mail printed under it is read
  // by when the e-mail spells no author (`\lx@address@mark@set`, iopart's `\ead` reading). An address of spacing alone
  // (1202.6352's `\address{\vskip-6 pt}`, the earlier classes' way to letter one address for two authors) is none.
  DefMacro!("\\address[]{}", sub[(name, body)] {
    let place = lookup_int("lx_lmcs_addresses") + 1;
    assign_value("lx_lmcs_addresses", Stored::Int(place), Some(Scope::Global));
    let (letters, rest) = leading_lsuper(body.unlist());
    let items: Vec<Vec<Token>> = match letters {
      Some(letters) => label_items(&letters),
      None if lookup_bool("lx_lmcs_labelled") => vec![vec![T_LETTER!(radix::radix_alpha(place).as_str())]],
      None => Vec::new(),
    };
    let name = name.unwrap_or_default();
    if items.is_empty() {
      return Ok(Invocation!(T_CS!("\\lx@lmcs@address@run"), vec![Some(name), Some(Tokens::new(rest))]));
    }
    let mut mark = vec![T_MATH!(), T_SUPER!(), T_BEGIN!()];
    for (at, item) in items.iter().enumerate() {
      if at > 0 {
        mark.push(T_OTHER!(","));
      }
      mark.extend(item.iter().copied());
    }
    mark.extend([T_END!(), T_MATH!()]);
    // (an address with several letters, `{\\lsuper{b,c}}Univ BC` (1202.6352), is the address of each: one labelled
    // address a letter, so each author its letter names holds it)
    let mut calls = Vec::new();
    for item in &items {
      let label = s!("affiliation:{}", Tokens::new(item.clone()).to_string().trim());
      calls.extend(
        Invocation!(T_CS!("\\lx@lmcs@address@labelled"), vec![
          Some(name.clone()),
          Some(mouth::tokenize_internal(TeXString::assembled(label))),
          Some(Tokens::new(rest.clone()))
        ])
        .unlist(),
      );
    }
    calls.extend(Invocation!(T_CS!("\\lx@address@mark@set"), vec![Some(Tokens::new(mark))]).unlist());
    Ok(Tokens::new(calls))
  });
  DefMacro!(
    "\\lx@lmcs@address@labelled{}{}{}",
    "\\lx@add@address[name={#1},label={#2}]{#3}"
  );
  DefMacro!(
    "\\lx@lmcs@address@run{}{}",
    "\\lx@add@address[name={#1},annotate=run]{#2}\\lx@address@mark@set{}"
  );
  // `\email{a@x, b@y}` prints under the address before it ("e-mail address: …", :438-441): each address of the list is
  // the author it spells, else, under a lettered address, the author its letter names alone, else the group's author
  // (`\lx@add@contact@placed`, as jheppub's `\emailAdd` and iopart's `\ead`; 2402.09138's three addresses of one
  // `\email` for its three authors).
  DefMacro!("\\email[]{}", "\\lx@add@contact@list{\\lx@lmcs@email}{#2}");
  DefMacro!(
    "\\lx@lmcs@email{}",
    "\\lx@add@contact@placed[role=email,owner=sequence,mark=address,name={\\emailaddrname:\\ }]{}{\\lx@address@urlfont{#1}}"
  );
  // `\subjclass[edition]{…}` is the ACM classification (:252-261: `\subjclassname` is the 2012 ACM CCS's, whichever
  // edition is given; the earlier classes' the 1998 ACM Subject Classification, 1103.0510's LMCS.cls:205-215), its name
  // the one the paper's class prints.
  let edition = match crate::shipped_class::shipped_class_text("lmcs")? {
    Some(text)
      if !crate::shipped_class::class_code_contains(&text, "subjclassname@2012")
        && crate::shipped_class::class_code_contains(&text, "subjclassname@1998") =>
    {
      "1998 ACM Subject Classification"
    },
    _ => "2012 ACM CCS",
  };
  RawTeX!(&s!(r"\def\subjclassname{{{edition}}}"));
  DefMacro!(
    "\\lx@ams@subjclass[]{}",
    "\\lx@add@classification[scheme={\\subjclassname},name={\\subjclassname:~}]{#2}"
  );
});

/// An author's name as the frontmatter reads it: each `\rsuper{a,b}` the letters' address requests ([`inst_requests`]),
/// each `\lmcsorcid` the author's ORCID contact. `labelled` is set when a letter is requested.
fn lmcs_author_name(name: &[Token], labelled: &mut bool) -> Vec<Token> {
  let mut out = Vec::with_capacity(name.len());
  let mut at = 0;
  while at < name.len() {
    let t = name[at];
    if t == T_CS!("\\rsuper") {
      let (argument, next) = read_argument(name, at + 1);
      out.extend(inst_requests(&argument, labelled));
      at = next;
      continue;
    }
    out.push(if t == T_CS!("\\lmcsorcid") {
      T_CS!("\\lx@add@orcid")
    } else {
      t
    });
    at += 1;
  }
  out
}

/// `\inst{…}` for each label of a comma list, its letters lowercased as the class's `\MakeLowercase` prints them: a
/// letter or number requests the address it labels; a footnote symbol (`\rsuper{a,*}`, 2012.13129) stays shown.
fn inst_requests(labels: &[Token], labelled: &mut bool) -> Vec<Token> {
  let mut out = Vec::new();
  for item in label_items(labels) {
    *labelled |= item.iter().any(|t| {
      matches!(t.get_catcode(), Catcode::LETTER)
        || t.with_str(|s| s.chars().all(|c| c.is_ascii_digit()))
    });
    out.push(T_CS!("\\inst"));
    out.push(T_BEGIN!());
    out.extend(item);
    out.push(T_END!());
  }
  out
}

/// The labels of a comma list, each without its spaces and its letters lowercased.
fn label_items(labels: &[Token]) -> Vec<Vec<Token>> {
  split_tokens(Tokens::new(labels.to_vec()), vec![SplitDelim::Token(
    T_OTHER!(","),
  )])
  .into_iter()
  .map(|item| {
    item
      .unlist()
      .into_iter()
      .filter(|t| t.get_catcode() != Catcode::SPACE)
      .map(|t| {
        let lower = t.with_str(|s| s.to_ascii_lowercase());
        if t.get_catcode() == Catcode::LETTER && t.with_str(|s| s != lower) {
          T_LETTER!(&lower)
        } else {
          t
        }
      })
      .collect::<Vec<_>>()
  })
  .filter(|item| !item.is_empty())
  .collect()
}

/// The argument of a macro at `at`: a brace group's content, else the one token there; and where it ends.
fn read_argument(tokens: &[Token], at: usize) -> (Vec<Token>, usize) {
  let mut start = at;
  while start < tokens.len() && tokens[start].get_catcode() == Catcode::SPACE {
    start += 1;
  }
  if start >= tokens.len() {
    return (Vec::new(), start);
  }
  if tokens[start].get_catcode() != Catcode::BEGIN {
    return (vec![tokens[start]], start + 1);
  }
  let mut depth = 0usize;
  for (end, t) in tokens.iter().enumerate().skip(start) {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth == 0 {
          return (tokens[start + 1..end].to_vec(), end + 1);
        }
      },
      _ => {},
    }
  }
  (tokens[start + 1..].to_vec(), tokens.len())
}

/// The letters an address opens with (`{\lsuper a}Univ`, `\lsuper{b,c} Univ`, `{\lsuper{a}}Univ`) and the address
/// without them; `None` when it opens with none. An address of spacing alone (`\vskip-6 pt`) is left empty.
fn leading_lsuper(body: Vec<Token>) -> (Option<Vec<Token>>, Vec<Token>) {
  let start = body
    .iter()
    .take_while(|t| t.get_catcode() == Catcode::SPACE)
    .count();
  let mut letters = None;
  let mut rest_at = start;
  if body.get(start) == Some(&T_CS!("\\lsuper")) {
    let (argument, next) = read_argument(&body, start + 1);
    letters = Some(argument);
    rest_at = next;
  } else if body
    .get(start)
    .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
  {
    let (group, next) = read_argument(&body, start);
    let inner = group
      .iter()
      .take_while(|t| t.get_catcode() == Catcode::SPACE)
      .count();
    if group.get(inner) == Some(&T_CS!("\\lsuper")) {
      let (argument, after) = read_argument(&group, inner + 1);
      if group[after..]
        .iter()
        .all(|t| t.get_catcode() == Catcode::SPACE)
      {
        letters = Some(argument);
        rest_at = next;
      }
    }
  }
  let rest: Vec<Token> = body[rest_at..].to_vec();
  // (spacing alone is no address)
  static SPACING: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
      r"\\(?:[vh]skip\s*[-+]?\s*[0-9.]*\s*(?:pt|em|ex|cm|mm|in|bp|sp|pc|dd|cc|mu)|[vh]space\*?\s*\{[^}]*\}|(?:small|med|big)skip|par)",
    )
    .unwrap()
  });
  let spacing = SPACING
    .replace_all(&Tokens::new(rest.clone()).to_string(), "")
    .trim()
    .is_empty();
  if spacing {
    return (letters, Vec::new());
  }
  (letters, rest)
}
