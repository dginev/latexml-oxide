//! aistats.sty — the AISTATS conference styles (aistats2012.sty, aistats2014.sty … aistats2027.sty, which the versioned
//! fallback strips to `aistats`; aistats2026.sty, and the 2010-2011 aistats2e.sty and its margin variant aistats2eM.sty,
//! by their own names): one family binding (as Perl's icml.sty.ltxml serves the ICML years, icml_support.sty.ltxml:34ff).
//! The paper's copy is read raw, then its title-block setters, which print in place inside `\twocolumn[…]` and never
//! reach `\title`/`\author` (aistats2027.sty:173/248/355, aistats2eM.sty:102/171/252, the same in every year), are the
//! frontmatter's:
//! - `\aistatstitle{T}` is the title (`\title`); a later one prints in place as the copy does (the supplement's
//!   "Supplementary Material" heading, 2610.11580), where a second `\title` would be dropped;
//! - `\aistatsauthor{A₁ \And A₂}` followed by `\aistatsaddress{a₁ \And a₂}` are the authors, each column of the address
//!   under the author of its column when the row has several columns and the address as many (`A₁ \\ a₁ \And A₂ \\ a₂`:
//!   name, affiliation and e-mail lines, as the kernel `\author` reads them; 2604.21432, 2605.03921, 2610.11580,
//!   1002.4058) — also across material between them that prints nothing (`\runningauthor`, `\vspace`, a definition:
//!   2003.03080, 2402.12508, 1302.5348), which runs first; otherwise the names, and the address shared by them (`\lx@add@affiliation@shared`): each column an
//!   affiliation of the row, its e-mail lines the authors' (1806.02924's four names over one Carnegie Mellon address,
//!   2510.09534's one column of names apart by `\quad`), or, marked, each line to the authors whose marks it names
//!   (1910.09056's `$^1$University of British Columbia, $^2$…`, a row of one column). Each `\aistatsauthor` row adds to
//!   the ones before. Their arguments are read where they stand (`in_place`: 1302.5348's `\UMDAddress`).
//!
//! What the PDF shows decides (user ruling 2026-10-06): a submission under review (`\statePaper` 0, no `accepted` or
//! 2027 `preprint` option) prints "Anonymous Author" and "Anonymous Institution" where the copy says so (aistats2020.sty
//! … aistats2027.sty; 2e, 2012, 2014 and 2019 print the names either way). Rebinding `\aistatsauthor` also retires the
//! copy's running-head measurement, whose `\PackageError{Document}{Running heading author exceeds size limitations}`
//! (aistats2026.sty:289-295) the former aistats2026 stub pre-empted with a placeholder `\@runningauthor` (47 papers,
//! stages 15-20). Perl has no aistats binding: the setters print in place there too, or are undefined (3 errors).
//! OXIDIZED_DESIGN_DIVERGENCES #772. Witnesses 1002.4058, 1302.5134, 1302.5348, 1303.0449, 1103.0941 (2e), 1202.1568
//! (2eM), 1102.1492 (2012), 1311.1894 (2014), 1806.02924 (2019), 2604.21432 (2020), 1910.09056 (2022), 2405.15172 (2025),
//! 2605.03921, 2510.09534 (2026), 2610.11580 (2027); 2101.00218 ships aistats2021.sty unused.
use latexml_engine::base_utilities::{distinct_author_marks, has_author_marks};
use latexml_package::{
  prelude::*,
  shipped_class::{class_code_contains, shipped_copy_text},
};

LoadDefinitions!({
  // The style as requested (`aistats2027`, also through the versioned fallback: `fallback_request`, possibly with a
  // directory).
  let fallback_request = lookup_string("fallback_request");
  let request = match fallback_request.rsplit('/').next() {
    Some(name) if name.to_ascii_lowercase().starts_with("aistats") => fallback_request.clone(),
    _ => do_expand(T_CS!("\\@currname"))?.to_string(),
  };
  let shipped = !request.is_empty()
    && find_file(
      &request,
      Some(FindFileOptions {
        ext_type: Some(Cow::Borrowed("sty")),
        forbid_ltxml: true,
        ..Default::default()
      }),
    )
    .is_some();
  let mut anonymous_under_review = false;
  if shipped {
    InputDefinitions!(&request, noltxml => true, extension => Some(Cow::Borrowed("sty")));
    anonymous_under_review =
      shipped_copy_text("sty")?.is_some_and(|text| class_code_contains(&text, "Anonymous Author"));
  }
  assign_value(
    "lx_aistats_anonymous",
    anonymous_under_review,
    Some(Scope::Global),
  );
  // Each setter where the copy defines it (every year's does), or, without a copy, as the family's interface.
  let owns = |cs: &str| !shipped || IsDefined!(&T_CS!(cs));
  if owns("\\aistatstitle") {
    if IsDefined!(&T_CS!("\\aistatstitle")) {
      Let!("\\lx@aistats@title@inplace", "\\aistatstitle");
    } else {
      // (aistats2027.sty:225-226)
      DefMacro!(
        "\\lx@aistats@title@inplace{}",
        "{\\centering{\\Large\\bfseries #1\\par}}"
      );
    }
    // (aistats2022.sty:172 `\gdef\@supplementtitle{#1}`, which its `\makesupplementtitle` prints, :366-368; 1910.09056)
    DefMacro!(
      "\\aistatstitle{}",
      "\\gdef\\@supplementtitle{#1}\\lx@aistats@title{#1}"
    );
    DefMacro!("\\lx@aistats@title{}", sub[(title)] {
      if lookup_bool("lx_aistats_titled") {
        Ok(Invocation!(T_CS!("\\lx@aistats@title@inplace"), vec![Some(title)]))
      } else {
        assign_value("lx_aistats_titled", true, Some(Scope::Global));
        Ok(Invocation!(T_CS!("\\title"), vec![None, Some(title)]))
      }
    });
  }
  if owns("\\aistatsauthor") {
    DefMacro!("\\aistatsauthor{}", sub[(names)] { aistats_authors(names) });
    DefMacro!("\\lx@aistats@pair{}{}", sub[(names, address)] { aistats_pair(names, address) });
  }
  if owns("\\aistatsaddress") {
    DefMacro!("\\aistatsaddress{}", sub[(address)] { aistats_address(address) });
  }
});

/// Whether the copy prints the anonymous block: a submission (`\statePaper` is not `\acceptedPaper`'s 1) to a year
/// whose copy has one.
fn under_review() -> bool {
  lookup_bool("lx_aistats_anonymous")
    && IsDefined!(&T_CS!("\\statePaper"))
    && do_expand(T_CS!("\\statePaper")).is_ok_and(|state| state.to_string().trim() != "1")
}

/// The columns of an author row or an address, split at the copy's column separators outside braces: `\And`, and
/// `\AND`, `\ANDD`, which also start a new row (aistats2027.sty:315-325, the same in every year).
fn columns(tokens: Tokens) -> Vec<Tokens> {
  split_tokens(tokens, vec![
    T_CS!("\\And").into(),
    T_CS!("\\AND").into(),
    T_CS!("\\ANDD").into(),
  ])
  .into_iter()
  .filter(|column| {
    !column
      .unlist_ref()
      .iter()
      .all(|t| t.get_catcode() == Catcode::SPACE)
  })
  .collect()
}

/// Tokens that assign or test a meaning (`\renewcommand{\Uni}{…}`, `\ifx\Uni\relax`, `\string\Uni`): a column of a
/// setter argument holding one is not read in place ([`read_columns`]), as the meaning it names is the one the column
/// itself makes or asks about where it is digested — reading the target in place garbled it (review probe `g_renewin`: "rstSecondFirst"
/// for pdflatex's "Second"; 2511.05163's `\renewcommand{\arraystretch}{1.0}` inside its names).
fn assigns_or_tests(tokens: &[Token]) -> bool {
  tokens.iter().any(|token| {
    token.get_catcode() == Catcode::CS
      && token.with_str(|name| {
        matches!(
          name,
          "\\def"
            | "\\gdef"
            | "\\edef"
            | "\\xdef"
            | "\\let"
            | "\\futurelet"
            | "\\global"
            | "\\newcommand"
            | "\\renewcommand"
            | "\\providecommand"
            | "\\DeclareRobustCommand"
            | "\\ifx"
            | "\\ifdefined"
            | "\\ifcsname"
            | "\\noexpand"
            | "\\string"
            | "\\meaning"
            | "\\csname"
            | "\\detokenize"
            | "\\unexpanded"
        )
      })
  })
}

/// `tokens` as the setter prints them where it stands: each macro without parameters whose meaning the paper made
/// inside a group open now (`\twocolumn[…]`'s, whose `]` ends it; [`meaning_is_group_local`]) replaced by its expansion,
/// as the frontmatter that holds them is digested after that group, where the meaning is gone (1302.5348's
/// `\newcommand{\UMDAddress}{University of Maryland \\ College Park, MD 20742, USA}` between `\aistatsauthor` and
/// `\aistatsaddress`: "undefined"; a global `\Uni` renewed inside the block, review probe `g_globlocal`: the global
/// meaning). A column that assigns or tests a meaning is not read in place ([`read_columns`]), and the body of such a
/// macro that does is not read further (review probe `g_hash`'s `\newcommand{\Uni}{\def\z##1{[##1]}\z{Hash Univ}}`).
/// Other macros stay; a local macro with parameters (`\def\Uni#1{…}`, probe `g_args`) or a local `\let` of a primitive
/// (`\let\Uni\relax`, probe `g_letprim`) is not read in place (SYNC residual).
fn in_place(tokens: Tokens) -> Tokens {
  fn expand(tokens: &[Token], depth: usize, out: &mut Vec<Token>) {
    for token in tokens {
      if depth < 8
        && meaning_is_group_local(token)
        && let Some(Stored::Expandable(definition)) = lookup_meaning(token)
        && definition.get_parameters().is_none()
        && let Some(ExpansionBody::Tokens(body)) = definition.get_expansion()
      {
        if assigns_or_tests(body.unlist_ref()) {
          out.extend_from_slice(body.unlist_ref());
        } else {
          expand(body.unlist_ref(), depth + 1, out);
        }
      } else {
        out.push(*token);
      }
    }
  }
  let mut out = Vec::new();
  expand(tokens.unlist_ref(), 0, &mut out);
  Tokens::new(out)
}

/// `columns` joined by `\And`, the kernel's author separator.
fn joined(columns: Vec<Tokens>) -> Tokens {
  let mut out: Vec<Token> = Vec::new();
  for (i, column) in columns.into_iter().enumerate() {
    if i > 0 {
      out.push(T_CS!("\\And"));
    }
    out.extend(column.unlist());
  }
  Tokens::new(out)
}

/// `\aistatsaddress{a₁ \And a₂}` not paired with the names before it: each column an affiliation of the authors of the
/// row before it (`annotate=run`; columns printed alike, one), as the copy prints them centred under the row — split,
/// not one affiliation "a₁ & a₂" (review probes `a_mismatch`, `a_addrfirst`) —, their e-mail lines the authors' e-mails
/// (`\lx@add@affiliation@shared`); marked, each affiliation goes to the authors whose marks it names (1910.09056's
/// `$^1$University of British Columbia, $^2$…`).
fn aistats_address(address: Tokens) -> Result<Tokens> {
  let address = if under_review() {
    vec![Tokens::new(Explode!("Anonymous Institution"))]
  } else {
    read_columns(address)
  };
  shared_address(address)
}

/// The address columns as one shared address ([`aistats_address`]).
fn shared_address(columns: Vec<Tokens>) -> Result<Tokens> {
  Ok(Invocation!(T_CS!("\\lx@add@affiliation@shared"), vec![
    Some(Tokens::new(Explode!("annotate=run"))),
    Some(joined(columns))
  ]))
}

/// The columns of a setter argument ([`columns`]), each read in place ([`in_place`]) unless it assigns or tests a
/// meaning ([`assigns_or_tests`]) — column by column, so a renewal in one leaves the next read in place (pdflatex's
/// "Second" and "Local Univ" for `\renewcommand{\Uni}{Second}\Uni \And \Glob`); a macro that expands to several
/// columns is split again.
fn read_columns(tokens: Tokens) -> Vec<Tokens> {
  columns(tokens)
    .into_iter()
    .flat_map(|column| {
      if assigns_or_tests(column.unlist_ref()) {
        vec![column]
      } else {
        columns(in_place(column))
      }
    })
    .collect()
}

/// How a command that prints nothing in a title block reads what follows it ([`unprinted`]).
#[derive(Clone, Copy)]
enum Unprinted {
  /// Nothing (`\medskip`, `\par`, a size or font declaration, `\selectfont`, `\begingroup`).
  Bare,
  /// An optional `*`, then that many arguments (`\vspace*{…}`, `\fontsize{…}{…}`, the copy's `\runningauthor{…}`).
  Arguments(usize),
  /// An optional `*` and `[…]` (`\\`).
  LineBreak,
  /// Glue (`\vskip 12pt`, `\vskip\baselineskip`).
  Glue,
  /// An optional `*`, the command, optional `[…]`s, the body (`\newcommand{\cs}[1]{…}`).
  NewCommand,
  /// The command, its parameter text, the body (`\def\cs#1{…}`).
  Def,
  /// The command, an optional `=`, the meaning (`\let\cs\relax`).
  Let,
}

/// The commands that print nothing where they stand between an author row and its address, and how each reads its
/// arguments: vertical space, definitions, size and font declarations, the copy's running-head setters
/// (aistats2027.sty:170-171 `\def\runningauthor#1{\gdef\@runningauthor{#1}}`), groups — what the 2,843 papers with
/// `\aistatsauthor` put there (77o review scan: `\runningauthor` 40 rows, `\vspace` 79, `\medskip`, `\vskip`,
/// `\newcommand`, `\renewcommand\thefootnote`, `\fontsize…\selectfont`, `~\\`, `\begingroup…\endgroup`).
fn unprinted(token: &Token) -> Option<Unprinted> {
  if token.get_catcode() != Catcode::CS {
    return None;
  }
  token.with_str(|name| {
    Some(match name {
      "\\par" | "\\medskip" | "\\smallskip" | "\\bigskip" | "\\relax" | "\\noindent"
      | "\\begingroup" | "\\endgroup" | "\\global" | "\\selectfont" | "\\normalsize"
      | "\\small" | "\\footnotesize" | "\\scriptsize" | "\\tiny" | "\\large" | "\\Large"
      | "\\LARGE" | "\\huge" | "\\Huge" | "\\normalfont" | "\\bf" | "\\it" | "\\rm" | "\\sf"
      | "\\tt" | "\\sc" | "\\sl" | "\\bfseries" | "\\mdseries" | "\\itshape" | "\\upshape"
      | "\\slshape" | "\\scshape" | "\\rmfamily" | "\\sffamily" | "\\ttfamily" => Unprinted::Bare,
      "\\vspace" | "\\addvspace" | "\\runningauthor" | "\\runningtitle" | "\\newlength" => {
        Unprinted::Arguments(1)
      },
      "\\fontsize" | "\\setlength" | "\\addtolength" | "\\setcounter" => Unprinted::Arguments(2),
      "\\\\" | "\\newline" => Unprinted::LineBreak,
      "\\vskip" => Unprinted::Glue,
      "\\newcommand" | "\\renewcommand" | "\\providecommand" | "\\DeclareRobustCommand" => {
        Unprinted::NewCommand
      },
      "\\def" | "\\gdef" | "\\edef" | "\\xdef" => Unprinted::Def,
      "\\let" => Unprinted::Let,
      _ => return None,
    })
  })
}

/// The next token, the spaces before it read into `read`; `None` at the end of the input.
fn next_non_space(read: &mut Vec<Token>) -> Result<Option<Token>> {
  while let Some(token) = read_token()? {
    if token.get_catcode() == Catcode::SPACE {
      read.push(token);
    } else {
      return Ok(Some(token));
    }
  }
  Ok(None)
}

/// The rest of a group whose `{` is read, into `read`: whether its `}` is.
fn read_group_rest(read: &mut Vec<Token>) -> Result<bool> {
  let mut depth = 1usize;
  while let Some(token) = read_token()? {
    read.push(token);
    match token.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth == 0 {
          return Ok(true);
        }
      },
      _ => {},
    }
  }
  Ok(false)
}

/// One argument, a group or a token, into `read`: whether there was one (not a `}` or the end of the input).
fn read_argument(read: &mut Vec<Token>) -> Result<bool> {
  let Some(token) = next_non_space(read)? else {
    return Ok(false);
  };
  read.push(token);
  match token.get_catcode() {
    Catcode::BEGIN => read_group_rest(read),
    Catcode::END => Ok(false),
    _ => Ok(true),
  }
}

/// An optional `*` into `read`.
fn read_star(read: &mut Vec<Token>) -> Result<()> {
  if let Some(token) = next_non_space(read)? {
    if token == T_OTHER!("*") {
      read.push(token);
    } else {
      unread_one(token);
    }
  }
  Ok(())
}

/// An optional `[…]` into `read`: whether its `]` is read (or there is none).
fn read_bracketed(read: &mut Vec<Token>) -> Result<bool> {
  let Some(token) = next_non_space(read)? else {
    return Ok(true);
  };
  if token != T_OTHER!("[") {
    unread_one(token);
    return Ok(true);
  }
  read.push(token);
  let mut depth = 0usize;
  while let Some(token) = read_token()? {
    read.push(token);
    match token.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END if depth == 0 => return Ok(false),
      Catcode::END => depth -= 1,
      _ if depth == 0 && token == T_OTHER!("]") => return Ok(true),
      _ => {},
    }
  }
  Ok(false)
}

/// The arguments a command of [`unprinted`] reads, into `read`: whether they are all there.
fn read_unprinted_arguments(shape: Unprinted, read: &mut Vec<Token>) -> Result<bool> {
  Ok(match shape {
    Unprinted::Bare => true,
    Unprinted::Arguments(count) => {
      read_star(read)?;
      for _ in 0..count {
        if !read_argument(read)? {
          return Ok(false);
        }
      }
      true
    },
    Unprinted::LineBreak => {
      read_star(read)?;
      read_bracketed(read)?
    },
    Unprinted::Glue => {
      // a signed number, then a register or a unit
      let mut next = next_non_space(read)?;
      while let Some(token) = next.filter(|t| {
        t.get_catcode() == Catcode::SPACE
          || (t.get_catcode() == Catcode::OTHER && t.with_str(|s| "+-0123456789.,".contains(s)))
      }) {
        read.push(token);
        next = read_token()?;
      }
      match next {
        Some(token) if token.get_catcode() == Catcode::CS => read.push(token),
        Some(token) if token.get_catcode() == Catcode::LETTER => {
          read.push(token);
          let mut next = read_token()?;
          while let Some(token) = next.filter(|t| t.get_catcode() == Catcode::LETTER) {
            read.push(token);
            next = read_token()?;
          }
          if let Some(token) = next {
            unread_one(token);
          }
        },
        Some(token) => unread_one(token),
        None => return Ok(false),
      }
      true
    },
    Unprinted::NewCommand => {
      read_star(read)?;
      read_argument(read)? && read_bracketed(read)? && read_bracketed(read)? && read_argument(read)?
    },
    Unprinted::Def => {
      let Some(name) = read_token()? else {
        return Ok(false);
      };
      read.push(name);
      loop {
        let Some(token) = read_token()? else {
          return Ok(false);
        };
        read.push(token);
        match token.get_catcode() {
          Catcode::BEGIN => break read_group_rest(read)?,
          Catcode::END => return Ok(false),
          _ => {},
        }
      }
    },
    Unprinted::Let => {
      let Some(name) = read_token()? else {
        return Ok(false);
      };
      read.push(name);
      let Some(mut meaning) = next_non_space(read)? else {
        return Ok(false);
      };
      if meaning == T_OTHER!("=") {
        read.push(meaning);
        let Some(next) = read_token()? else {
          return Ok(false);
        };
        meaning = next;
        if meaning.get_catcode() == Catcode::SPACE {
          read.push(meaning);
          let Some(next) = read_token()? else {
            return Ok(false);
          };
          meaning = next;
        }
      }
      read.push(meaning);
      true
    },
  })
}

/// The `\aistatsaddress{…}` an author row stands over, read ahead through what prints nothing between them: spaces,
/// `~` and paragraph ends (the template's lines), the commands of [`unprinted`] with their arguments, groups (2501.00988's
/// `{\fontsize{9.6pt}{12pt}\selectfont` before it). With the material between, in order, without its spaces, `~`
/// and paragraph ends. `None` — what was read is read again — when something that may print comes first (a character, a
/// command not of [`unprinted`]: a paragraph of text between them keeps the address unpaired), the next
/// `\aistatsauthor`, or the end of the block (the `}` closing the group the row stands in).
fn address_ahead() -> Result<Option<(Vec<Token>, Tokens)>> {
  let mut read: Vec<Token> = Vec::new();
  let mut kept: Vec<Token> = Vec::new();
  let mut depth = 0usize;
  loop {
    let Some(token) = read_token()? else {
      break;
    };
    if token == T_CS!("\\aistatsaddress") {
      return Ok(Some((kept, read_arg(ExpansionLevel::Off)?)));
    }
    read.push(token);
    let code = token.get_catcode();
    if code == Catcode::SPACE
      || token == T_CS!("\\par")
      || (code == Catcode::ACTIVE && token.with_str(|s| s == "~"))
    {
      continue;
    }
    // (a source comment kept under `--comments` stays with the material)
    if code == Catcode::COMMENT {
      kept.push(token);
      continue;
    }
    let start = read.len() - 1;
    let complete = match code {
      Catcode::BEGIN => {
        depth += 1;
        true
      },
      Catcode::END if depth > 0 => {
        depth -= 1;
        true
      },
      _ => match unprinted(&token) {
        Some(shape) => read_unprinted_arguments(shape, &mut read)?,
        None => false,
      },
    };
    if !complete {
      break;
    }
    kept.extend_from_slice(&read[start..]);
  }
  unread_vec(read);
  Ok(None)
}

/// `\aistatsauthor{names}`: the authors, with the `\aistatsaddress{…}` that stands under them when only what prints
/// nothing comes between ([`address_ahead`]), that material first, so a macro it defines is read in place
/// (1302.5348's `\newcommand{\UMDAddress}`; 2003.03080's `\vspace`, 2402.12508's `\runningauthor{…}`, 2602.12181's
/// `\medskip`: [`aistats_pair`]); else the names, one author list across the columns, and an address after other
/// material goes on its own ([`aistats_address`]).
fn aistats_authors(names: Tokens) -> Result<Tokens> {
  if let Some((mut before, address)) = address_ahead()? {
    before
      .extend(Invocation!(T_CS!("\\lx@aistats@pair"), vec![Some(names), Some(address)]).unlist());
    return Ok(Tokens::new(before));
  }
  if under_review() {
    return Ok(Invocation!(T_CS!("\\lx@add@authors@list"), vec![Some(
      Tokens::new(Explode!("Anonymous Author"))
    )]));
  }
  Ok(Invocation!(T_CS!("\\lx@add@authors@list"), vec![Some(
    joined(read_columns(names))
  )]))
}

/// An author row and the address under it: each column of names over the column of the address under it when the row
/// has several columns and the address as many, in as many rows (`\AND`), and the two are not marked — what stands
/// between them aside (2003.03080, 2402.12508, 2602.12181,
/// 2305.16147, 1804.04378, 1302.5348, 1902.01239, 1603.09584) — (`A₁ \\ a₁ \And A₂ \\ a₂`: name, affiliation and
/// e-mail lines, as the kernel `\author` reads them; 2604.21432, 2605.03921, 2610.11580, 1002.4058); else the names,
/// one author list across the columns, and the address after them ([`aistats_address`]) — a row of one column is names
/// the address is printed under together (2510.09534's `Jeong$^\ast$ \quad Zhai$^\ast$ \quad Ročková` over one
/// University of Chicago; 1910.09056's marked names and addresses), as is a row of other columns than its address
/// (1806.02924's four names over one Carnegie Mellon address) or in other rows, and a marked row over a marked address,
/// whose marks pair them (2312.06786's `$^1$Carnegie Mellon University \And $^2$Microsoft Research \AND {…}@… \And …`
/// under four marked names: four columns, two of them e-mails on a second row).
fn aistats_pair(names: Tokens, address: Tokens) -> Result<Tokens> {
  if under_review() {
    let mut block = Explode!("Anonymous Author");
    block.push(T_CS!("\\\\"));
    block.extend(Explode!("Anonymous Institution"));
    return Ok(Invocation!(T_CS!("\\lx@add@authors@list"), vec![Some(
      Tokens::new(block)
    )]));
  }
  // (rows of the two tables alike, and no marks to pair them by: an address is a key the names' marks index only when it
  // shows two or more distinct marks — one mark is a note's, `$^*$equal contribution`, and the PDF still prints column i
  // under author i; 77o r3 review, 1905.11190, 2405.18281)
  let row_breaks = |tokens: &Tokens| {
    tokens
      .unlist_ref()
      .iter()
      .filter(|t| **t == T_CS!("\\AND") || **t == T_CS!("\\ANDD"))
      .count()
  };
  let by_column = row_breaks(&names) == row_breaks(&address)
    && !(has_author_marks(&names) && distinct_author_marks(&address) >= 2);
  let name_columns = read_columns(names);
  let address_columns = read_columns(address);
  if by_column && name_columns.len() > 1 && name_columns.len() == address_columns.len() {
    let mut block: Vec<Token> = Vec::new();
    for (i, (name, place)) in name_columns.into_iter().zip(address_columns).enumerate() {
      if i > 0 {
        block.push(T_CS!("\\And"));
      }
      block.extend(name.unlist());
      if !place.is_empty() {
        block.push(T_CS!("\\\\"));
        block.extend(place.unlist());
      }
    }
    return Ok(Invocation!(T_CS!("\\lx@add@authors@list"), vec![Some(
      Tokens::new(block)
    )]));
  }
  let mut out = Invocation!(T_CS!("\\lx@add@authors@list"), vec![Some(joined(
    name_columns
  ))])
  .unlist();
  out.extend(shared_address(address_columns)?.unlist());
  Ok(Tokens::new(out))
}
