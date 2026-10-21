//! `epl2.cls` (and its predecessor `epl.cls`, whose general and bibliography macros it took over: epl2.cls:1328,
//! :1344): the EPL (Europhysics Letters) class of the Italian Physical Society, not in TeX Live and seldom shipped, so
//! the papers fell to OmniBus and stopped at the class's bibliography macros, `\Name` first (740 run-336 papers:
//! 0911.3858, 1904.03227, 1602.05654). Read from epl2.cls v1.1 (2006-2012, as shipped in arXiv 1201.0004), which is
//! written from scratch after article.cls (:13-16); here the article class with the frontmatter routed to the
//! frontmatter API. Rust-only: Perl has no EPL binding. A paper shipping the class gets this binding too, as a binding
//! outranks a raw class.
use latexml_package::prelude::*;

LoadDefinitions!({
  // epl2.cls:109-160: layout options (the column styles, running-head and page-id styles, paper sizes, draft/final),
  // and the `cite`/`bm`/`graphicx` packages the class loads unless told not to (:160 `\ExecuteOptions{…cite,bm,
  // graphicx…}`, :166-181); `figures` implies `graphicx` (:125). `reply` and `comment` (:149-157) mark a Comment or
  // Reply and drop its abstract.
  for option in [
    "doublecol",
    "orcish",
    "tight",
    "ugly",
    "pretty",
    "left",
    "centre",
    "classic",
    "pageid",
    "pageclassic",
    "cite",
    "bm",
    "graphicx",
    "figures",
    "nofigures",
    "a4paper",
    "a5paper",
    "b5paper",
    "letterpaper",
    "legalpaper",
    "executivepaper",
    "draft",
    "final",
    "linenumbers",
  ] {
    DeclareOption!(option, None);
  }
  DeclareOption!("nocite", {
    AssignValue!("epl@nocite" => true, Some(Scope::Global));
  });
  DeclareOption!("nobm", {
    AssignValue!("epl@nobm" => true, Some(Scope::Global));
  });
  DeclareOption!("nographicx", {
    AssignValue!("epl@nographicx" => true, Some(Scope::Global));
  });
  DeclareOption!("reply", {
    AssignValue!("epl@noabstract" => true, Some(Scope::Global));
  });
  DeclareOption!("comment", {
    AssignValue!("epl@noabstract" => true, Some(Scope::Global));
  });
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{article}")?;
  });
  ProcessOptions!();
  LoadClass!("article");
  RequirePackage!("color");
  if !lookup_bool("epl@nographicx") {
    RequirePackage!("graphicx");
  }
  if lookup_bool("epl@nobm") {
    Let!("\\bm", "\\mathbf");
  } else {
    RequirePackage!("bm");
  }
  if !lookup_bool("epl@nocite") {
    RequirePackage!("cite");
  }
  // :167 `\revision{#1}`: an editor's red revision mark
  DefMacro!("\\revision{}", "{\\color{red}{#1}}");

  //======================================================================
  // Frontmatter (:543-760). The class stores each piece and lays them out in `\maketitle`; the kernel's `\title`,
  // `\author` and `\maketitle` stand, as they collect the same pieces.
  // :635-648 the running-head short title and author list: page furniture
  DefMacro!("\\shorttitle{}", "");
  DefMacro!("\\shortauthor{}", "");
  // :639 `\subtitle`, set under the title (:777)
  DefMacro!("\\subtitle{}", "\\lx@add@subtitle{#1}");
  // The kernel's `\author` collects the names; a note that only labels an address (`\thanks{E-mail: \email{…}}`, 159
  // of 170 `\email`s, 1110.0608, 1102.3054) is the author's email contact, named by its label, as an `\email` among
  // the names is.
  DefMacro!(
    "\\lx@epl@noted@address{}{}",
    "\\lx@add@email[name={#1 }]{#2}"
  );
  DefMacro!(
    "\\lx@epl@address@contact{}",
    "\\lx@add@contact[role=email]{#1}"
  );
  Let!("\\lx@epl@author", "\\author");
  DefMacro!("\\author[]{}", sub[(short, names)] {
    let names: Tokens = names;
    Ok(Invocation!(T_CS!("\\lx@epl@author"), vec![short, Some(email_notes_as_emails(names))]))
  }, locked => true);
  // :725 the `\inst` the title prints the names with, `\def\inst##1{\unskip${}^{##1}$…}` (defined only there): it takes
  // back the space before it, so `\author{Zhen Wang,\inst{1,2} Attila Szolnoki,\inst{3} …}` gives each name the marks
  // after its comma (1201.1900; base_utilities.rs `inst_takes_back_space`)
  DefMacro!("\\lx@title@inst{}", "\\unskip${}^{#1}$");
  // :657 `\etal`
  DefMacro!("\\etal", "\\unskip\\ \\emph{et al.}");
  // :1375 `\let\And\and`
  Let!("\\And", "\\and");
  // :682 `\institute{…}`: the authors' institutions, each marked with the `\inst{n}` its authors carry
  // (`\institute{\inst{1} GSI, … \\ \inst{2} Institut für Physik, …}`, 1803.06254), which the title typesets as a
  // superscript (:725 `\def\inst##1{\unskip${}^{##1}$…}`): an affiliation list whose `\inst` marks set the labels the
  // authors' `\inst` marks request. Unmarked, it is one institution, its `\\` a line in it (:684-686, :788), of all the
  // authors (1802.05436, 0806.3107, 1110.0608).
  DefMacro!("\\institute{}", sub[(list)] {
    let list = email_notes_as_emails(list);
    Ok(if has_inst_marks(&list) {
      Invocation!(T_CS!("\\lx@add@affiliations"), vec![None, Some(entries_at_marks(list))])
    } else {
      Invocation!(T_CS!("\\lx@add@affiliation"), vec![Some(Tokenize!("annotate=all")), Some(list)])
    })
  });
  // :1322-1327 the address markup of the institutions: their text
  for cs in [
    "\\orgdivision",
    "\\orgname",
    "\\postcode",
    "\\street",
    "\\city",
    "\\country",
  ] {
    Let!(T_CS!(cs), "\\@iden");
  }
  // :693-705 `\pacs{<code>}{<description>}`, set as "PACS <code> – <description>"
  DefMacro!(
    "\\pacs{}{}",
    "\\lx@add@classification[scheme=pacs,name={PACS~}]{#1 -- #2}"
  );
  // :707 `\abstract{…}`, not an environment. A Comment or Reply prints none (:817 `\ifepl@noabstr`): the author's
  // abstract is then the document's metadata, an `ltx:rdf` annotation, as metadata the PDF does not show is
  // (0907.0855).
  if lookup_bool("epl@noabstract") {
    DefMacro!(
      "\\abstract{}",
      "\\lx@add@frontmatter{ltx:rdf}[property=dcterms:abstract]{#1}"
    );
  } else {
    DefMacro!("\\abstract{}", "\\lx@add@abstract{#1}");
  }
  // :568 `\dedication`, set under the abstract (:800)
  DefMacro!("\\dedication{}", "\\lx@add@pubnote[role=dedication]{#1}");
  // :543-562, :606 the issue data (volume, issue, year, first page, DOI, page id and the dates) the publisher fills
  // in, printed in the banner only by the production layout (:615-634 prints neither): stored, as there. The class's
  // `\renewcommand{\year}` (:545) is not followed: `\year` stays the TeX date parameter the kernel's `\today` reads.
  for cs in [
    "\\vol",
    "\\issue",
    "\\firstpage",
    "\\doi",
    "\\pgid",
    "\\received",
    "\\accepted",
    "\\acceptedinfinalform",
    "\\paperpub",
    "\\onlinepub",
  ] {
    DefMacro!(T_CS!(cs), "{}", "");
  }
  DefMacro!("\\info{}{}{}{}{}", "");
  // :403-439 the title-page spacing adjustments, :854-857 the caption punctuation switches: layout only
  for cs in [
    "\\stretchpretitle",
    "\\stretchpreauthor",
    "\\stretchpreinst",
    "\\stretchpredates",
    "\\stretchprepacs",
    "\\stretchpreabst",
    "\\stretchprerule",
    "\\stretchpostrule",
    "\\stretchfront",
    "\\stretchprebib",
    "\\stretchpostbib",
    "\\stretchtail",
    "\\stretchall",
    "\\shrinkacknowledgments",
    "\\enlargeacknowledgments",
  ] {
    DefMacro!(T_CS!(cs), "{}", "");
  }
  Let!("\\shrinkacknowledgements", "\\shrinkacknowledgments");
  Let!("\\enlargeacknowledgements", "\\enlargeacknowledgments");
  for cs in ["\\nodash", "\\dash", "\\nodot", "\\godot"] {
    DefMacro!(T_CS!(cs), None, "");
  }

  //======================================================================
  // Body
  // :874 `\setcounter{secnumdepth}{0}`: EPL's section headings are not numbered
  Digest!("\\setcounter{secnumdepth}{0}")?;
  // :899-908 `\acknowledgments`, the "∗∗∗" separator its acknowledgements follow (also `\acknowledgements`, `\stars`):
  // the acknowledgements, to the end of the text before the references.
  DefConstructor!("\\acknowledgments", "<ltx:acknowledgements>");
  Let!("\\acknowledgements", "\\acknowledgments");
  Let!("\\stars", "\\acknowledgments");
  // (:1009 `\appendix` letters the sections after it, as the kernel's does)
  // :1057 `{widetext}`, a full-width float of the text it holds: the text, where it stands — a TeX group like the
  // class's environment, not a boxed `#body` DefEnvironment, so display math `$$…$$` inside stays display math
  // (0810.4982: 14 `Script _` errors from a boxed `$$`; the same class as 64ra's mode-less `"#body"` environments).
  // Repro block-model/epl_widetext_dollardollar_is_display.
  RawTeX!("\\newenvironment{widetext}{}{}");
  // :1095-1118 the class's graphics inclusion: one centred figure, or two side by side
  DefMacro!("\\onefigure[]{}", "\\includegraphics*[#1]{#2}");
  DefMacro!(
    "\\twofigures[]{}{}",
    "\\includegraphics*[#1]{#2}\\hfill\\includegraphics*[#1]{#3}"
  );
  // :1135 `{largetable}`, a centred full-width table
  DefMacro!(
    T_CS!("\\begin{largetable}"),
    None,
    "\\begin{table*}\\begin{center}"
  );
  DefMacro!(
    T_CS!("\\end{largetable}"),
    None,
    "\\end{center}\\end{table*}"
  );
  // :1332-1342 the general macros. `\email` is the address in typewriter type where it stands, mostly in a note
  // that labels it (`\thanks{E-mail: \email{…}}`, 159 of 170 calls): a contact of its own lost that label.
  DefMacro!("\\email{}", "\\texttt{#1}");
  DefMacro!("\\upd", "\\mathrm{d}");
  Let!("\\drm", "\\upd");
  DefMacro!("\\ab{}", "\\mathrm{#1}");
  DefMacro!("\\tx{}", "{\\mbox{#1}}");
  DefMacro!("\\vect{}", "\\bm{#1}");
  DefMacro!("\\tens{}", "\\mathsf{#1}");
  DefMacro!("\\un{}", "\\ensuremath{\\unskip\\,\\mathrm{#1}}");
  DefMacro!("\\chem{}", "\\ensuremath{\\mathrm{#1}}");
  DefMacro!("\\mth{}", "\\ensuremath{#1}");
  DefMacro!("\\acro{}", "\\textmd{#1}");

  //======================================================================
  // :1347-1377 the bibliography macros (from epl.cls): a journal reference `\Name{A. Author} \REVIEW{Phys. Rev.
  // Lett.}{80}{1998}{2003}` (or `\Review{…} \Vol{…} \Year{…} \Page{…}`), a book `\Book{…} \Publ{…} \Year{…}`; each
  // `\bibitem` starts a journal reference.
  RawTeX!("\\newif\\ifepl@book\\epl@bookfalse\\newif\\ifepl@gotand\\epl@gotandfalse");
  RawTeX!("\\let\\epl@org@bibitem\\bibitem\\def\\bibitem{\\epl@bookfalse\\epl@org@bibitem}");
  DefMacro!("\\Name{}", "{\\scshape #1},");
  DefMacro!("\\Review{}", "\\epl@bookfalse{\\itshape #1},");
  DefMacro!(
    "\\Vol{}",
    "\\ifepl@book\\unskip, Vol.~{\\bfseries #1}\\else{\\bfseries #1}\\fi"
  );
  DefMacro!("\\Year{}", "\\ifepl@book#1\\else(#1)\\fi");
  DefMacro!(
    "\\Page{}",
    "\\ifepl@book\\unskip,\\ {\\normalfont p.~#1}\\else{\\normalfont #1}\\fi"
  );
  DefMacro!(
    "\\Pages{}{}",
    "\\ifepl@book\\unskip,\\ {\\normalfont pp.~#1-#2}\\else{\\normalfont #1-#2}\\fi"
  );
  DefMacro!("\\Book{}", "\\epl@booktrue{\\itshape #1}");
  DefMacro!("\\Publ{}", "\\ifepl@book\\unskip\\ \\fi{\\normalfont(#1)}");
  DefMacro!(
    "\\Editor{}",
    "\\epl@gotandfalse\\ifepl@book\\unskip,\\ {\\normalfont edited by}\\ {\\scshape #1}\\else{\\scshape #1}\\ {\\normalfont (Editor\\ifepl@gotand s\\fi)},\\fi"
  );
  DefMacro!(
    "\\Section{}",
    "\\ifepl@book\\unskip,\\ {\\normalfont sect.~#1}\\else{\\normalfont sect.~#1}\\fi"
  );
  DefMacro!(
    "\\REVIEW{}{}{}{}",
    "\\Review{#1} \\Vol{#2} \\Year{#3} \\Page{#4}"
  );
  DefMacro!("\\SAME{}{}{}", "\\Vol{#1} \\Year{#2} \\Page{#3}");
});

/// An `\institute` list as one line per entry: an entry begins at its `\inst` mark (or epl2's `\[n]`, :726), a `\\`
/// within it (`… Blegdamsvej 17,\\ DK-2100 Copenhagen \O, Denmark\\ \inst{3}…`, 1803.06254) is a break in the title
/// block's lines, not a new entry, and an entry may follow the one before it on its line (`… e-mail:
/// ziaja@mail.desy.de \inst{2} …`). The spaces after a mark, and a `\\`'s star and space (`\\[2pt]`), are not the
/// entry's; the break is a grouped `{\\}`, which does not end the entry.
fn entries_at_marks(list: Tokens) -> Tokens {
  let inst = T_CS!("\\inst");
  let newline = T_CS!("\\\\");
  let tokens = bracket_marks_as_inst(list.unlist());
  let mut entries: Vec<Token> = Vec::with_capacity(tokens.len());
  let mut depth = 0usize;
  let mut started = false;
  // after a mark: reading its argument, then past it (the spaces next are dropped)
  let (mut in_mark, mut after_mark) = (false, false);
  let mut i = 0;
  while i < tokens.len() {
    let t = tokens[i];
    i += 1;
    let catcode = t.get_catcode();
    if depth == 0
      && catcode == Catcode::SPACE
      && (after_mark
        || !started
        || entries
          .last()
          .is_some_and(|last| last.get_catcode() == Catcode::SPACE))
    {
      continue;
    }
    after_mark = false;
    match catcode {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth = depth.saturating_sub(1);
        if depth == 0 && in_mark {
          in_mark = false;
          after_mark = true;
        }
      },
      _ => {},
    }
    if depth == 0 && t == newline {
      // a line break within the entry, without its `*` and `[<space>]`
      if tokens.get(i) == Some(&T_OTHER!("*")) {
        i += 1;
      }
      let next = tokens[i..]
        .iter()
        .position(|t| t.get_catcode() != Catcode::SPACE)
        .map_or(i, |n| i + n);
      if tokens.get(next) == Some(&T_OTHER!("["))
        && let Some(close) = tokens[next..].iter().position(|t| *t == T_OTHER!("]"))
      {
        i = next + close + 1;
      }
      while entries
        .last()
        .is_some_and(|last| last.get_catcode() == Catcode::SPACE)
      {
        entries.pop();
      }
      entries.extend([T_BEGIN!(), newline, T_END!()]);
      // (nor the spaces after it)
      after_mark = true;
      continue;
    }
    if depth == 0 && t == inst {
      if started {
        while entries
          .last()
          .is_some_and(|last| last.get_catcode() == Catcode::SPACE)
        {
          entries.pop();
        }
        entries.push(newline);
      }
      in_mark = true;
    }
    started = true;
    entries.push(t);
  }
  Tokens::new(entries)
}

/// Whether `list` marks its entries (`\inst{n}`, `\[n]`).
fn has_inst_marks(list: &Tokens) -> bool {
  list
    .unlist_ref()
    .iter()
    .any(|t| *t == T_CS!("\\inst") || *t == T_CS!("\\["))
}

/// epl2's other spelling of the mark, `\[n]` for `\inst{n}` (:726 `\def\[##1]{\inst{##1}}`), as `\inst{n}`.
fn bracket_marks_as_inst(tokens: Vec<Token>) -> Vec<Token> {
  let mut out = Vec::with_capacity(tokens.len());
  let mut i = 0;
  while i < tokens.len() {
    if tokens[i] == T_CS!("\\[")
      && let Some(close) = tokens[i + 1..].iter().position(|t| *t == T_OTHER!("]"))
    {
      out.push(T_CS!("\\inst"));
      out.push(T_BEGIN!());
      out.extend_from_slice(&tokens[i + 1..i + 1 + close]);
      out.push(T_END!());
      i += close + 2;
      continue;
    }
    out.push(tokens[i]);
    i += 1;
  }
  out
}

/// `tokens` with each note that is only a label and an `\email` of an address (`\thanks{E-mail: \email{x@y}}`,
/// `\footnote{\email{E-mail:x@y}}`) replaced by that email, its label the contact's name (`\lx@epl@noted@address`):
/// among the names it is that author's (1110.0608, 1212.3881), in an institute the institute's authors' (1107.1575).
fn email_notes_as_emails(names: Tokens) -> Tokens {
  let tokens = names.unlist();
  let mut out = Vec::with_capacity(tokens.len());
  let mut depth = 0usize;
  let mut i = 0;
  while i < tokens.len() {
    if (tokens[i] == T_CS!("\\thanks") || tokens[i] == T_CS!("\\footnote"))
      && let Some(open) = tokens[i + 1..]
        .iter()
        .position(|t| t.get_catcode() != Catcode::SPACE)
        .map(|n| i + 1 + n)
      && tokens[open].get_catcode() == Catcode::BEGIN
      && let Some(close) = group_end(&tokens, open)
      && let Some((label, address)) = lone_email(&tokens[open + 1..close])
    {
      out.push(T_SPACE!());
      if label.is_empty() {
        out.push(T_CS!("\\email"));
      } else {
        out.push(T_CS!("\\lx@epl@noted@address"));
        out.push(T_BEGIN!());
        out.extend(label);
        out.push(T_END!());
      }
      out.push(T_BEGIN!());
      out.extend(address);
      out.push(T_END!());
      i = close + 1;
      continue;
    }
    match tokens[i].get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.saturating_sub(1),
      _ => {},
    }
    // a bare `\email` after a comma (`\author{V. V. Kisil, \email{kisilv@…}}`, 0907.0855) stands among the names
    // without it
    if depth == 0 && tokens[i] == T_CS!("\\email") {
      while out
        .last()
        .is_some_and(|t: &Token| t.get_catcode() == Catcode::SPACE)
      {
        out.pop();
      }
      if out.last() == Some(&T_OTHER!(",")) {
        out.pop();
      }
      out.push(T_SPACE!());
      // an address in markup (`\email{\href{mailto:…}{…}}`, 0907.0855), which the kernel reads as no address: the
      // author's email contact as written
      if let Some(open) = tokens[i + 1..]
        .iter()
        .position(|t| t.get_catcode() != Catcode::SPACE)
        .map(|n| i + 1 + n)
        && tokens[open].get_catcode() == Catcode::BEGIN
        && let Some(close) = group_end(&tokens, open)
        && tokens[open + 1..close]
          .iter()
          .any(|t| t.get_catcode() == Catcode::CS)
      {
        out.push(T_CS!("\\lx@epl@address@contact"));
        out.extend_from_slice(&tokens[open..=close]);
        i = close + 1;
        continue;
      }
    }
    out.push(tokens[i]);
    i += 1;
  }
  Tokens::new(out)
}

/// The index of the group's closing brace, the group opening at `open`.
fn group_end(tokens: &[Token], open: usize) -> Option<usize> {
  let mut depth = 0usize;
  for (j, t) in tokens.iter().enumerate().skip(open) {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth == 0 {
          return Some(j);
        }
      },
      _ => {},
    }
  }
  None
}

/// The label and address of a note that holds an `\email` and a label only (no other command, at most a few words):
/// `E-mail: \email{x@y}`, or `\email{E-mail:x@y}` (1212.3881).
fn lone_email(note: &[Token]) -> Option<(Vec<Token>, Vec<Token>)> {
  let at = note.iter().position(|t| *t == T_CS!("\\email"))?;
  let open = at
    + 1
    + note[at + 1..]
      .iter()
      .position(|t| t.get_catcode() != Catcode::SPACE)?;
  if note[open].get_catcode() != Catcode::BEGIN {
    return None;
  }
  let close = group_end(note, open)?;
  let mut label: Vec<Token> = Vec::new();
  for t in note[..at].iter().chain(&note[close + 1..]) {
    if t.get_catcode() == Catcode::CS
      || t.get_catcode() == Catcode::BEGIN
      || t.get_catcode() == Catcode::MATH
    {
      return None;
    }
  }
  label.extend(note[..at].iter().copied());
  // a label inside the command's argument, before its address
  let inner = &note[open + 1..close];
  let (inner_label, address) = match inner.iter().rposition(|t| *t == T_OTHER!(":")) {
    Some(colon) => (&inner[..=colon], &inner[colon + 1..]),
    None => (&inner[..0], inner),
  };
  label.extend(inner_label.iter().copied());
  let trim = |tokens: &[Token]| -> Vec<Token> {
    let start = tokens
      .iter()
      .position(|t| t.get_catcode() != Catcode::SPACE)
      .unwrap_or(tokens.len());
    let end = tokens
      .iter()
      .rposition(|t| t.get_catcode() != Catcode::SPACE)
      .map_or(start, |e| e + 1);
    tokens[start..end].to_vec()
  };
  let (label, address) = (trim(&label), trim(address));
  // an address, one word; a label of a few words at most
  if address.is_empty()
    || address.iter().any(|t| t.get_catcode() == Catcode::SPACE)
    || label.len() > 24
  {
    return None;
  }
  Some((label, address))
}
