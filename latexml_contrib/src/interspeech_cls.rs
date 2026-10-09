//! Interspeech.cls (the ISCA template a paper ships; Interspeech2024.cls and other versioned names fall back here) on
//! OmniBus: the packages the class requires, its one-author-per-call `\name`/`\author` with keys, its `\address` block
//! or numbered `\affiliation`s, `\email`, and its camera-ready switch, as the frontmatter.
//! Witnesses 2409.08589, 2409.08711, 2605.02715, 2406.11727, 2506.00350.
use std::borrow::Cow;

use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  // The packages the class requires (Interspeech.cls:55-68 of 2024, :66-81 of 2026; witnesses 2409.08589,
  // 2605.02715), in its order: a paper's `\color` and siunitx `S` column come from them. xcolor comes with siunitx
  // (its binding loads it, as Perl's does; the 2026 class requires it too), and a later `[table]{xcolor}` still loads
  // colortbl, whose `\rowcolor` and array's `m{}` columns convert cleanly. Only the 2026 class adds tikz; it is left to
  // the papers that draw, which load it themselves.
  RequirePackage!("graphicx");
  RequirePackage!("amssymb");
  RequirePackage!("amsmath");
  RequirePackage!("bm");
  RequirePackage!("textcomp");
  RequirePackage!("booktabs");
  RequirePackage!("caption", options => vec![s!("textfont=it"), s!("tableposition=top")]);
  RequirePackage!("siunitx");
  RequirePackage!("xspace");
  RequirePackage!("url");
  RequirePackage!("lipsum");
  RequirePackage!("tipa");
  RequirePackage!("enumitem");
  RequirePackage!("xkeyval");
  RequirePackage!("calc");
  RequirePackage!("xcolor");
  RequirePackage!("lineno", options => vec![s!("switch")]);
  RequirePackage!("hyperref");

  // The camera-ready switch: `\interspeechcameraready` (Interspeech2024.cls:108-112, the 2025 class alike) or the
  // 2026 class's `cameraready` option (:13-15). Without it the class prints "Anonymous submission to Interspeech <year>"
  // where the authors go, and neither their affiliations nor their emails (`\anonname`, `\anonaddress`, `\anonemail`);
  // shown as the PDF shows it (ruling 2026-10-06). The class decides where `\maketitle` typesets them, and a paper may
  // name its authors or set the switch anywhere before it, so the frontmatter they make is collected and added there
  // (or at the end of a document without `\maketitle`); what comes after is added as it comes.
  let request = lookup_string("fallback_request");
  // The paper's own copy of the class, as text (a copy need not be UTF-8).
  let class = request
    .rsplit('/')
    .next()
    .filter(|name| !name.is_empty())
    .unwrap_or("Interspeech");
  let source = find_file(
    class,
    Some(FindFileOptions {
      ext_type: Some(Cow::Borrowed("cls")),
      forbid_ltxml: true,
      ..Default::default()
    }),
  )
  .and_then(|path| std::fs::read(path).ok())
  .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
  .unwrap_or_default();
  let code_lines = || {
    source
      .lines()
      .map(|line| line.split('%').next().unwrap_or_default().trim())
  };
  // The anonymous note as the copy words it (`Anonymous submission to Interspeech 2025`, INTERSPEECH2023.cls:168-174
  // `… INTERSPEECH 2023`), else with the year of the class's name.
  let year: String = class.chars().filter(char::is_ascii_digit).collect();
  let anonymous = code_lines()
    .find(|code| code.starts_with("Anonymous submission"))
    .map(str::to_string)
    .unwrap_or_else(|| {
      if year.is_empty() {
        s!("Anonymous submission to Interspeech")
      } else {
        s!("Anonymous submission to Interspeech {year}")
      }
    });
  // The switch starts as the copy sets it, its last setting at top level winning. The template's is off, and copies edited
  // to start it on print the authors with no switch in the paper: all 11 papers of a 146-paper sample that set none,
  // with `\interspeechfinaltrue` on a line of its own (2406.08914, 2506.01138, …) or a call of `\interspeechcameraready`
  // after its definition (2406.07291's Interspeech2024.cls:110-113; the definition's own lines are neither).
  let starts_final = code_lines()
    .filter_map(|code| match code {
      "\\interspeechfinaltrue" | "\\camerareadytrue" | "\\interspeechcameraready" => Some(true),
      "\\interspeechfinalfalse" | "\\camerareadyfalse" => Some(false),
      _ => None,
    })
    .next_back()
    .unwrap_or(false);
  // (`\ifcameraready` is the 2026 class's switch, :52-54, which papers test around their acknowledgements)
  RawTeX!(
    r"\newif\ifinterspeechfinal\def\interspeechcameraready{\global\interspeechfinaltrue}\newif\ifcameraready"
  );
  if starts_final {
    RawTeX!(r"\interspeechfinaltrue\camerareadytrue");
  }
  DeclareOption!("cameraready", "\\camerareadytrue\\interspeechfinaltrue");
  ProcessOptions!();
  // One author per call, given and family name apart, keys in the optional argument: `\name[affiliation={1,*}]{First}
  // {Last}` (Interspeech2024.cls:91; witness 2406.11727, whose single-argument binding read `[` as the name) and
  // `\author[…]{First}{Last}` (2025 :95-110, witness 2409.08589; 2026 :146-209, witness 2605.02715, whose last names
  // fell into the body). The class prints `First Last` with the affiliation marks, `*` for equal contribution and `**`
  // for the corresponding author, and links the ORCID; it ends the title block with a note for each of those two marks
  // (2026 :338-352), here a note of each author so marked.
  raw_tex(&format!(
    r"\newif\iflx@is@equal\newif\iflx@is@corresponding\newif\iflx@is@decided\newif\iflx@is@marked
\let\lx@is@authors\@empty\let\lx@is@frontmatter\@empty
\def\lx@is@decide{{\iflx@is@decided\else\global\lx@is@decidedtrue
  \ifcameraready\global\interspeechfinaltrue\fi
  \ifinterspeechfinal\lx@is@authors\lx@is@frontmatter
    \iflx@is@equal\lx@add@thanks[label={{*}}]{{These authors contributed equally.}}\fi
    \iflx@is@corresponding\lx@add@thanks[label={{**}}]{{indicates the corresponding author.}}\fi
  \else\@add@frontmatter{{ltx:note}}[role=authors]{{{anonymous}}}\fi
  \global\let\lx@is@authors\@empty\global\let\lx@is@frontmatter\@empty\fi}}
\AtBeginDocument{{\let\lx@is@maketitle\maketitle\def\maketitle{{\lx@is@decide\lx@is@maketitle}}}}
\AtEndDocument{{\lx@is@decide}}"
  ))?;
  DefMacro!("\\name []{}", sub[(keys, given)] { Ok(interspeech_author(keys, given)?) });
  DefMacro!("\\author []{}", sub[(keys, given)] { Ok(interspeech_author(keys, given)?) });
  // (the authors are added before the rest, as the class prints them before their affiliations whatever the order in
  // the source)
  DefMacro!(
    "\\lx@is@frontmatter@add{}",
    "\\iflx@is@decided\\ifinterspeechfinal#1\\fi\\else\\g@addto@macro\\lx@is@frontmatter{#1}\\fi"
  );
  DefMacro!(
    "\\lx@is@author@add{}",
    "\\iflx@is@decided\\ifinterspeechfinal#1\\fi\\else\\g@addto@macro\\lx@is@authors{#1}\\fi"
  );
  DefMacro!(
    "\\lx@is@creator{}{}",
    "\\lx@add@creator[role=author,annotations={#1}]{#2}"
  );
  DefMacro!("\\lx@is@author@named []{}{}", sub[(keys, given, family)] {
    let mut name = given.unlist();
    name.push(T_SPACE!());
    name.extend(family.unlist());
    interspeech_author_entry(keys, Tokens::new(name))
  });
  // The affiliations: one `\address` block, of `\\` lines marked `$^1$`, `$^2$` for marked authors (2024 :265, 2026
  // :216; 2406.11727) or else one affiliation over its lines (2401.07506), or one `\affiliation{department}
  // {institution}{place}` per affiliation, numbered as the class numbers it and printed `$^n$department, institution,
  // place` without an empty department (2025 :115-129).
  DefMacro!("\\address{}", sub[(block)] {
    // (an unmarked block is the affiliation of all the authors; 2401.07506)
    let mut entry = if leads_with_mark(&block) {
      vec![T_CS!("\\lx@add@affiliations")]
    } else {
      TokenizeInternal!("\\lx@add@affiliation[annotate=all]").unlist()
    };
    entry.push(T_BEGIN!());
    entry.extend(block.unlist());
    entry.push(T_END!());
    Ok(Invocation!(T_CS!("\\lx@is@frontmatter@add"), vec![Some(Tokens::new(entry))]))
  });
  // (a paper whose authors carry no affiliation marks gets its affiliations as an unmarked list, each the affiliation of
  // all the authors, as in 2401.07506, 2406.02167; the marked test is made when the entries are added, after every
  // author; and the class copies that take keys, `\affiliation[nocounter]{…}{…}{…}`, print a `nocounter` one
  // unnumbered, the affiliation of all, 2506.10653 Interspeech.cls:152-176)
  RawTeX!(
    r"\@ifundefined{c@affcounter}{\newcounter{affcounter}}{}
\def\lx@is@affiliation@numbered#1#2#3{\stepcounter{affcounter}%
  \edef\lx@is@entry{\noexpand\lx@is@frontmatter@add{\noexpand\lx@is@affiliation@entry{\theaffcounter}{\unexpanded{\lx@is@place{#1}{#2}{#3}}}}}%
  \lx@is@entry}
\def\lx@is@affiliation@entry#1#2{\iflx@is@marked\lx@add@contact[label={#1},role=affiliation]{#2}\else
  \lx@add@affiliation[annotate=all]{#2}\fi}
\def\lx@is@affiliation@unnumbered#1#2#3{\lx@is@frontmatter@add{\lx@add@affiliation[annotate=all]{\lx@is@place{#1}{#2}{#3}}}}
\def\lx@is@place#1#2#3{\if\relax\detokenize{#1}\relax\else#1, \fi#2, #3}"
  );
  DefMacro!("\\affiliation[]{}{}{}", sub[(keys, department, institution, place)] {
    let unnumbered = keys.is_some_and(|keys| {
      split_top_level(keys.unlist(), &T_OTHER!(","))
        .iter()
        .any(|item| Tokens::new(item.clone()).to_string().trim().starts_with("nocounter"))
    });
    let form = if unnumbered { "\\lx@is@affiliation@unnumbered" } else { "\\lx@is@affiliation@numbered" };
    Ok(Invocation!(T_CS!(form), vec![Some(department), Some(institution), Some(place)]))
  });
  DefMacro!(
    "\\email{}",
    "\\lx@is@frontmatter@add{\\@add@frontmatter{ltx:note}[role=email]{#1}}"
  );
  // INTERSPEECH2023.cls L160: `\def\ninept{\def\baselinestretch{0.95}
  // \let\normalsize\small\normalsize}` — 9-point text mode. Layout
  // adjustment, semantically irrelevant for our XML output. Witness
  // 2312.05730.
  def_macro_noop("\\ninept")?;
  // Vectors and matrices are bold, not arrow-accented (Interspeech.cls:152-153 of 2025, :227-228 of 2026).
  DefMacro!("\\vec{}", "\\ensuremath{\\bm{{#1}}}");
  DefMacro!("\\mat{}", "\\vec{#1}");
  DefMacro!(
    "\\thanks{}",
    "\\@add@frontmatter{ltx:note}[role=thanks]{#1}"
  );
  DefMacro!(
    "\\keywords{}",
    "\\@add@frontmatter{ltx:classification}[scheme=keywords]{#1}"
  );
  DefMacro!(
    "\\copyrightnotice{}",
    "\\@add@frontmatter{ltx:note}[role=copyright]{#1}"
  );
});

/// `\name[keys]{First}{Last}` / `\author[keys]{First}{Last}`, or, when no braced family name follows, a whole author
/// list in the one argument, its names marked `$^1$` (2506.00350; the class takes the next token as the family name,
/// and pdflatex prints the list).
fn interspeech_author(keys: Option<Tokens>, given: Tokens) -> Result<Tokens> {
  let next = read_non_space()?;
  if let Some(token) = next {
    unread_one(token);
  }
  if next.is_some_and(|t| t.get_catcode() == Catcode::BEGIN) {
    Ok(Invocation!(T_CS!("\\lx@is@author@named"), vec![
      keys,
      Some(given)
    ]))
  } else {
    // (the names' marks link them to the numbered affiliations: `Xixin Wu$^{1,3,*}$`)
    let tokens = given.unlist_ref();
    let marked = tokens.iter().enumerate().any(|(i, t)| {
      (t.get_catcode() == Catcode::SUPER || *t == T_CS!("\\textsuperscript"))
        && tokens[i + 1..]
          .iter()
          .find(|t| t.get_catcode() != Catcode::BEGIN && t.get_catcode() != Catcode::SPACE)
          .is_some_and(|t| {
            t.get_catcode() != Catcode::CS
              && t.with_str(|text| text.chars().all(char::is_alphanumeric))
          })
    });
    let mut out = Invocation!(T_CS!("\\lx@is@author@add"), vec![Some(Invocation!(
      T_CS!("\\lx@add@authors"),
      vec![Some(given)]
    ))])
    .unlist();
    if marked {
      out.extend([T_CS!("\\global"), T_CS!("\\lx@is@markedtrue")]);
    }
    Ok(Tokens::new(out))
  }
}

/// One author's frontmatter, added when the document begins: the creator with the class's marks — the
/// `affiliation={…}` items, `*` for `equalcontribution`, `**` for `correspondingauthor` — and the ORCID link
/// (Interspeech.cls of 2026 :129-209). A mark the class sets in math (`affiliation={1^\dagger}`, `{1{^\dagger}{^\#}}`,
/// 2406.08931, 2605.04749) is split into its items, a symbol one in math.
fn interspeech_author_entry(keys: Option<Tokens>, name: Tokens) -> Result<Tokens> {
  let mut marks: Vec<Vec<Token>> = Vec::new();
  let mut orcid: Option<Vec<Token>> = None;
  let (mut equal, mut corresponding) = (false, false);
  for item in split_top_level(keys.map(Tokens::unlist).unwrap_or_default(), &T_OTHER!(",")) {
    let (key, value) = match item.iter().position(|t| *t == T_OTHER!("=")) {
      Some(i) => (item[..i].to_vec(), Some(item[i + 1..].to_vec())),
      None => (item, None),
    };
    match Tokens::new(key).to_string().trim() {
      "affiliation" => marks.extend(mark_items(value.unwrap_or_default())),
      "equalcontribution" => equal = true,
      "correspondingauthor" => corresponding = true,
      "orcid" => orcid = value.map(strip_spaces),
      _ => {},
    }
  }
  // (an author marked only by symbols names no numbered affiliation)
  let marked = marks.iter().any(|mark| {
    mark.iter().all(|t| {
      t.get_catcode() != Catcode::CS && t.with_str(|text| text.chars().all(char::is_alphanumeric))
    })
  });
  if equal {
    marks.push(vec![T_OTHER!("*")]);
  }
  if corresponding {
    marks.push(vec![T_OTHER!("*"), T_OTHER!("*")]);
  }
  let mut annotations: Vec<Token> = Vec::new();
  for (i, mark) in marks.into_iter().enumerate() {
    if i > 0 {
      annotations.push(T_OTHER!(","));
    }
    if mark.iter().any(|t| t.get_catcode() == Catcode::CS) {
      annotations.extend([T_CS!("\\ensuremath"), T_BEGIN!()]);
      annotations.extend(mark);
      annotations.push(T_END!());
    } else {
      annotations.extend(mark);
    }
  }
  let mut entry = Invocation!(T_CS!("\\lx@is@creator"), vec![
    Some(Tokens::new(annotations)),
    Some(name)
  ])
  .unlist();
  if let Some(orcid) = orcid {
    entry.extend(
      Invocation!(T_CS!("\\lx@add@orcid"), vec![
        None,
        Some(Tokens::new(orcid))
      ])
      .unlist(),
    );
  }
  let mut out = Invocation!(T_CS!("\\lx@is@author@add"), vec![Some(Tokens::new(entry))]).unlist();
  if marked {
    out.push(T_CS!("\\global"));
    out.push(T_CS!("\\lx@is@markedtrue"));
  }
  if equal {
    out.push(T_CS!("\\global"));
    out.push(T_CS!("\\lx@is@equaltrue"));
  }
  if corresponding {
    out.push(T_CS!("\\global"));
    out.push(T_CS!("\\lx@is@correspondingtrue"));
  }
  Ok(Tokens::new(out))
}

/// `tokens` split at the top-level `separator`s, each piece without its outer spaces.
fn split_top_level(tokens: Vec<Token>, separator: &Token) -> Vec<Vec<Token>> {
  let mut items = vec![Vec::new()];
  let mut depth = 0;
  for token in tokens {
    match token.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth -= 1,
      _ => {},
    }
    if depth == 0 && token == *separator {
      items.push(Vec::new());
    } else {
      items.last_mut().unwrap().push(token);
    }
  }
  items
    .into_iter()
    .map(strip_spaces)
    .filter(|item| !item.is_empty())
    .collect()
}

/// `tokens` without leading and trailing spaces, and without one pair of braces around the whole.
fn strip_spaces(tokens: Vec<Token>) -> Vec<Token> {
  let start = tokens
    .iter()
    .position(|t| t.get_catcode() != Catcode::SPACE)
    .unwrap_or(tokens.len());
  let end = tokens
    .iter()
    .rposition(|t| t.get_catcode() != Catcode::SPACE)
    .map_or(start, |i| i + 1);
  let inner = &tokens[start..end];
  match inner {
    [open, body @ .., close]
      if open.get_catcode() == Catcode::BEGIN
        && close.get_catcode() == Catcode::END
        && split_top_level_depth_ok(body) =>
    {
      body.to_vec()
    },
    _ => inner.to_vec(),
  }
}

/// Whether `body` is balanced on its own (so braces around it enclose all of it).
fn split_top_level_depth_ok(body: &[Token]) -> bool {
  let mut depth = 0i32;
  for t in body {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth < 0 {
          return false;
        }
      },
      _ => {},
    }
  }
  depth == 0
}

/// The marks in an `affiliation` value: its comma-separated items, each script in them (`^\dagger`, `{^\#}`) an item
/// of its own, and a number glued to a symbol two (`1*`, `2\dagger`, `*1`, `\#1`: 2406.07909, 2409.15974): each run
/// of letters and digits, each run of other characters (`**`), each control sequence.
fn mark_items(value: Vec<Token>) -> Vec<Vec<Token>> {
  // 0: letters and digits, 1: other characters, 2: a control sequence (a run of its own)
  let kind = |token: &Token| match token.get_catcode() {
    Catcode::CS | Catcode::ACTIVE => 2,
    _ if token.with_str(|text| text.chars().all(char::is_alphanumeric)) => 0,
    _ => 1,
  };
  let mut items: Vec<Vec<Token>> = vec![Vec::new()];
  for token in strip_spaces(value) {
    match token.get_catcode() {
      Catcode::BEGIN | Catcode::END | Catcode::MATH => {},
      Catcode::SUPER | Catcode::SPACE => items.push(Vec::new()),
      _ if token == T_OTHER!(",") => items.push(Vec::new()),
      _ => {
        let item = items.last_mut().unwrap();
        if item
          .last()
          .is_some_and(|last| kind(last) != kind(&token) || kind(&token) == 2)
        {
          items.push(Vec::new());
        }
        items.last_mut().unwrap().push(token);
      },
    }
  }
  items.into_iter().filter(|item| !item.is_empty()).collect()
}
