use latexml_core::state::BRACED_ARGUMENT_TYPES;

use crate::prelude::*;

/// bm.sty's `\bm@define` (:307-316) `\xdef`s `\<name>` from its body, so a body that names
/// `\<name>` itself (`\DeclareBoldMathCommand{\nabla}{\nabla}`) makes it bold over its meaning
/// at that moment. Our body stays unexpanded, so that meaning is saved globally under a name of
/// its own, `\lx@bm@saved@<n>@<name>` (a new `<n>` per definition, as each `\xdef` sees the
/// meaning just before it: a second self-referential definition must not save the first's
/// wrapper under the same name), and the body refers to that; a plain `\gdef` would recurse.
fn bm_define(cs: Tokens, body: Tokens) -> Result<Tokens> {
  let Some(target) = cs
    .unlist()
    .into_iter()
    .find(|t| t.get_catcode() != Catcode::SPACE)
  else {
    Error!("expected", "\\bmdefine", "Missing the command to define");
    return Ok(Tokens::new(Vec::new()));
  };
  let mut body = body.unlist();
  let mut out = Vec::new();
  if body.contains(&target) {
    let n = lookup_int("lx@bm@saved@count") + 1;
    assign_value("lx@bm@saved@count", n, Some(Scope::Global));
    let saved = T_CS!(
      &target.with_cs_name(|name| { s!("\\lx@bm@saved@{n}@{}", name.trim_start_matches('\\')) })
    );
    out.extend([T_CS!("\\global"), T_CS!("\\let"), saved, target]);
    for token in body.iter_mut() {
      if *token == target {
        *token = saved;
      }
    }
  }
  out.extend([
    T_CS!("\\gdef"),
    target,
    T_BEGIN!(),
    T_CS!("\\bm"),
    T_BEGIN!(),
  ]);
  out.extend(body);
  out.extend([T_END!(), T_END!()]);
  Ok(Tokens::new(out))
}

/// Whether `token` means `\left`, `\middle` or `\right` (bm.sty:173-174 `\let`s `\left` and `\right` to
/// `\holdinginserts` while it scans, and tests a token's meaning against it, `\ifx\bm@previous\left`, :289).
fn is_left_right(token: &Token) -> bool {
  [T_CS!("\\left"), T_CS!("\\middle"), T_CS!("\\right")]
    .iter()
    .any(|command| token.defined_as(command))
}

/// The tokens of one TeX number after `\delimiter` (tex.web §440-445, at the token level), from `tokens[i]`: an
/// optional sign, then `"` with hexadecimal digits, `'` with octal digits, `` ` `` with its character, decimal
/// digits, or one control sequence (an internal quantity), and one optional space. Returns the index after them.
fn take_number(tokens: &[Token], mut i: usize, out: &mut Vec<Token>) -> usize {
  let text = |i: usize| tokens.get(i).map(ToString::to_string).unwrap_or_default();
  while matches!(text(i).as_str(), "+" | "-" | " ") {
    out.push(tokens[i]);
    i += 1;
  }
  let digit: fn(char) -> bool = match text(i).as_str() {
    "\"" => |c| c.is_ascii_digit() || ('A'..='F').contains(&c),
    "'" => |c| ('0'..='7').contains(&c),
    "`" => {
      out.extend(tokens.get(i..i + 2).unwrap_or(&tokens[i..]));
      return (i + 2).min(tokens.len());
    },
    _ if tokens
      .get(i)
      .is_some_and(|t| matches!(t.get_catcode(), Catcode::CS | Catcode::ACTIVE)) =>
    {
      out.push(tokens[i]);
      return i + 1;
    },
    _ => |c| c.is_ascii_digit(),
  };
  if matches!(text(i).as_str(), "\"" | "'") {
    out.push(tokens[i]);
    i += 1;
  }
  while text(i).chars().next().is_some_and(digit) && text(i).chars().count() == 1 {
    out.push(tokens[i]);
    i += 1;
  }
  if tokens
    .get(i)
    .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
  {
    out.push(tokens[i]);
    i += 1;
  }
  i
}

/// The index of the first token at or after `i` that is not a blank.
fn skip_blanks(tokens: &[Token], mut i: usize) -> usize {
  while tokens
    .get(i)
    .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
  {
    i += 1;
  }
  i
}

/// The index of the `}` closing the group that opens at `tokens[i]`, if it closes.
fn group_end(tokens: &[Token], i: usize) -> Option<usize> {
  let mut depth = 0usize;
  for (j, t) in tokens.iter().enumerate().skip(i) {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth = depth.checked_sub(1)?;
        if depth == 0 {
          return Some(j);
        }
      },
      _ => {},
    }
  }
  None
}

/// The index after one undelimited argument from `tokens[i]` (tex.web §392): blanks, then one token or a balanced
/// group.
fn take_argument(tokens: &[Token], i: usize) -> Option<usize> {
  let i = skip_blanks(tokens, i);
  match tokens.get(i)?.get_catcode() {
    Catcode::BEGIN => group_end(tokens, i).map(|end| end + 1),
    Catcode::END => None,
    _ => Some(i + 1),
  }
}

/// The index after an optional `[…]` argument from `tokens[i]` (latex.ltx `\@ifnextchar[`, after blanks), or `i`
/// when there is none.
fn take_optional(tokens: &[Token], i: usize) -> Option<usize> {
  let open = skip_blanks(tokens, i);
  let is_other = |t: &Token, c: &str| t.get_catcode() == Catcode::OTHER && t.to_string() == c;
  if !tokens.get(open).is_some_and(|t| is_other(t, "[")) {
    return Some(i);
  }
  let mut depth = 0usize;
  for (j, t) in tokens.iter().enumerate().skip(open + 1) {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => depth = depth.checked_sub(1)?,
      _ if depth == 0 && is_other(t, "]") => return Some(j + 1),
      _ => {},
    }
  }
  None
}

/// The argument of a script at `run[i]` and the index after it, or `None` when it cannot be told without expanding
/// it. A braced group gives its content. bm.sty expands the argument as it scans it (`\bm@expand`, with `\protect`
/// `\let` to `\@empty`, bm.sty:166-167, 286-287), so a command's own arguments go with it: a control sequence
/// takes as many arguments as its definition has parameters (`\mathrm{L}`, `\text{L}`, `\frac{1}{2}`: a
/// [`BRACED_ARGUMENT_TYPES`] parameter takes an argument, an optional one its `[…]` when present). An undefined
/// command, `\bgroup`, a parameter of any other kind, or a parameterless macro before a `{` (whose expansion may
/// take it, like a robust command's `\protect\cs␣`) cannot be told.
fn script_argument(run: &[Token], i: usize) -> Option<(Vec<Token>, usize)> {
  let i = skip_blanks(run, i);
  let first = *run.get(i)?;
  match first.get_catcode() {
    Catcode::BEGIN => {
      let end = group_end(run, i)?;
      Some((run[i + 1..end].to_vec(), end + 1))
    },
    Catcode::END => None,
    Catcode::CS | Catcode::ACTIVE => {
      if first.defined_as(&T_CS!("\\bgroup")) || first.defined_as(&T_CS!("\\egroup")) {
        return None;
      }
      let definition = lookup_definition(&first).ok().flatten()?;
      let mut end = i + 1;
      match definition.get_parameters() {
        None => {
          if definition.is_expandable()
            && run
              .get(skip_blanks(run, end))
              .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
          {
            return None;
          }
        },
        Some(parameters) => {
          for parameter in parameters.get_parameters() {
            if parameter.novalue {
              continue;
            }
            let name = to_string(parameter.name);
            end = if BRACED_ARGUMENT_TYPES.contains(&name.as_str()) {
              take_argument(run, end)?
            } else if parameter.optional && name != "OptionalMatch" {
              take_optional(run, end)?
            } else {
              return None;
            };
          }
        },
      }
      Some((run[i..end].to_vec(), end))
    },
    _ => Some((vec![first], i + 1)),
  }
}

/// A run of `\bm`'s argument between its delimiter commands, bolded by `bold` (`\lx@bm@bold`). Its leading scripts
/// attach to the delimiter before them, as bm.sty emits `_`/`^` where they stand (`\bm@add`): `\bm{\left\langle z
/// \right\rangle_{L,R}}` (2107.08152) and `\bm{\left(\Sigma_i^e \right)_{(j,j)}}` (1703.10645) keep their subscript
/// on the fence. bm.sty expands a script's argument before it bolds it (`\bm@expand`), so the argument stays whole
/// ([`script_argument`]: `\right\rangle_\mathrm{L}` bolds `\mathrm{L}`) and is bolded. A prime becomes a bold
/// `^{\prime}` (bm.sty:177 `\def'{\bm@prime\prime\relax}`), joined by a superscript right after it as
/// `\active@math@prime` joins it. A script whose argument cannot be told keeps the rest of the run as its argument,
/// with a Warning naming `cs`.
fn bold_run(out: &mut Vec<Token>, cs: &str, bold: Token, run: Vec<Token>) {
  let is_script = |t: &Token| {
    matches!(t.get_catcode(), Catcode::SUPER | Catcode::SUB)
      || t.defined_as(&T_SUPER!())
      || t.defined_as(&T_SUB!())
  };
  let is_superscript = |t: &Token| t.get_catcode() == Catcode::SUPER || t.defined_as(&T_SUPER!());
  let is_prime = |t: &Token| t.get_catcode() == Catcode::OTHER && t.to_string() == "'";
  let undetermined = |script: &Token, rest: &[Token]| {
    let head = rest
      .iter()
      .find(|t| t.get_catcode() != Catcode::SPACE)
      .map(ToString::to_string)
      .unwrap_or_default();
    Warn!(
      "unexpected",
      cs,
      s!(
        "Cannot tell the argument of {script} (starting {head}) after a \\left, \\middle or \\right in the \
         argument of {cs}; the rest of the argument is taken as it"
      )
    );
  };
  let mut i = 0;
  loop {
    while let Some(&t) = run.get(i)
      && t.get_catcode() == Catcode::SPACE
    {
      out.push(t);
      i += 1;
    }
    match run.get(i) {
      Some(prime) if is_prime(prime) => {
        let mut superscript = Vec::new();
        while run.get(i).is_some_and(is_prime) {
          superscript.push(T_CS!("\\prime"));
          i += 1;
        }
        if let Some(script) = run.get(i).filter(|t| is_superscript(t)) {
          match script_argument(&run, i + 1) {
            Some((argument, next)) => {
              superscript.extend(argument);
              i = next;
            },
            None => {
              undetermined(script, &run[i + 1..]);
              superscript.extend(&run[i + 1..]);
              i = run.len();
            },
          }
        }
        out.extend([T_SUPER!(), T_BEGIN!(), bold, T_BEGIN!()]);
        out.extend(superscript);
        out.extend([T_END!(), T_END!()]);
      },
      Some(&script) if is_script(&script) => {
        out.push(script);
        i += 1;
        // Nothing after the script in the run: its argument follows `\bm`, where bm.sty leaves it.
        if skip_blanks(&run, i) == run.len() {
          return;
        }
        let (argument, next) = script_argument(&run, i).unwrap_or_else(|| {
          undetermined(&script, &run[i..]);
          (run[i..].to_vec(), run.len())
        });
        out.extend([bold, T_BEGIN!()]);
        out.extend(argument);
        out.push(T_END!());
        i = next;
      },
      _ => break,
    }
  }
  let rest = &run[i.min(run.len())..];
  if !rest.is_empty() {
    out.extend([bold, T_BEGIN!()]);
    out.extend(rest);
    out.push(T_END!());
  }
}

/// How the package defining a bold-math command treats its argument.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoldArgument {
  /// bm.sty's `\bm` scans its argument token by token and emits it at the level `\bm` is used at (`\bm@general`,
  /// bm.sty:162-217): a `\left`, `\middle` or `\right` without its partner in the argument stands outside the
  /// bolded runs, without a diagnostic.
  Scanned,
  /// The argument is set in a box or a group of its own: amsbsy's `\boldsymbol` (`\mathchoice{\hbox{$…$}}`,
  /// amsbsy.sty:28-38), the `\bm` of the World Scientific classes (ws-ijmpe.cls:226-230 `\mbox{\boldmath$#1$}`) and
  /// of interact.cls (:134-138), and RevTeX's `\bm` (revtex4-2.cls:7573-7576 `\REV@bm`: the first use is dropped
  /// with "To use \bm, please load the bm package!", later ones are `\relax{…}`). A `\left` paired with its
  /// `\right` inside the argument is the bounded constructor, as before; an unpaired one is TeX's error ("Missing
  /// \right. inserted" at the box's end, "Extra \right." at the next box), set at the outer level as bm.sty would,
  /// with a Warning.
  Grouped,
}

/// Whether the `\left`, `\middle` and `\right` at an argument's top level pair up: each `\middle` and `\right`
/// inside an open `\left`, each `\left` closed.
fn delimiters_pair_up(commands: &[Token]) -> bool {
  let mut open = 0usize;
  for command in commands {
    if command.defined_as(&T_CS!("\\left")) {
      open += 1;
    } else if open == 0 {
      return false;
    } else if command.defined_as(&T_CS!("\\right")) {
      open -= 1;
    }
  }
  open == 0
}

/// The expansion of a bold-math command `cs` (`\bm`, `\boldsymbol`) whose bounded bold constructor is `bold`
/// (`\lx@bm@bold`, Perl's `\bm`, bm.sty.ltxml:20, and `\boldsymbol`, amsbsy.sty.ltxml:23, under their own names).
/// bm.sty's `\bm` (`\bm@general`, bm.sty:162-217) scans its argument token by token and emits the bold result at the
/// level `\bm` is used at, grouping only the argument's own `{…}` groups (`\bm@gr@up`): `\left` and `\right` are kept
/// as they are and the delimiter after them becomes a bold `\delimiter` (`\bm@delimiter`, :416). Here the argument is
/// bolded by the bounded `bold`, but a `\left`, `\middle` or `\right` at the argument's top level is taken out of
/// it: since 65d `\left` opens a `math_left_group` (OXIDIZED_DESIGN #530) that an argument group would close before
/// its `\right` (2204.13665, `\bm{\left\langle} #1 \bm{\right\rangle}`: "\bm Attempt to close a group that
/// switched to mode math", 587 errors). The command and its delimiter stand between the bolded runs, followed by
/// `\lx@bm@delimiter`, which makes the delimiter bold. Only an argument whose delimiters do not pair up is split so:
/// a pair inside it stays in the bounded `bold`, whose group holds the `math_left_group` whole, so the fenced whole
/// keeps its scripts and primes (`\bm{\left(A+B\right)^\mathsf{T}}`, `\bm{x \left( y \right)_\text{L}}`). For a
/// [`BoldArgument::Grouped`] command the split comes with a Warning: TeX's box ends the `\left` there.
/// OXIDIZED_DESIGN #544.
pub fn bold_math(cs: &str, bold: Token, arg: Tokens, argument: BoldArgument) -> Tokens {
  let tokens = arg.unlist();
  let mut depth = 0usize;
  let at_top_level = |depth: &mut usize, token: &Token| {
    match token.get_catcode() {
      Catcode::BEGIN => *depth += 1,
      Catcode::END => *depth = depth.saturating_sub(1),
      _ => {},
    }
    *depth == 0 && is_left_right(token)
  };
  let in_math = lookup_bool("IN_MATH");
  // Perl's `requireMath` on `cs` (Package.pm:1069-1073), the warning naming `cs`.
  requireMath!(cs);
  let commands: Vec<Token> = tokens
    .iter()
    .filter(|t| at_top_level(&mut depth, t))
    .copied()
    .collect();
  // A pair inside the argument stays in the bounded constructor, byte-identical to before: the `math_left_group`
  // opens and closes within its group (`\bm{\left(A+B\right)^\mathsf{T}}` keeps its superscript on the fenced
  // whole).
  let split = in_math && !commands.is_empty() && !delimiters_pair_up(&commands);
  if !split {
    let mut out = vec![bold, T_BEGIN!()];
    out.extend(tokens);
    out.push(T_END!());
    return Tokens::new(out);
  }
  if argument == BoldArgument::Grouped {
    Warn!(
      "unexpected",
      cs,
      s!(
        "\\left, \\middle or \\right without its partner in the argument of {cs}, whose box or group ends it in \
         TeX (\"Missing \\right. inserted\"); set at the outer level"
      )
    );
  }
  depth = 0;
  let mut out = Vec::new();
  let mut run = Vec::new();
  let mut i = 0;
  while let Some(&token) = tokens.get(i) {
    i += 1;
    if !at_top_level(&mut depth, &token) {
      run.push(token);
      continue;
    }
    bold_run(&mut out, cs, bold, std::mem::take(&mut run));
    out.push(token);
    // tex.web §1160 `scan_delimiter` passes blanks and `\relax` by.
    while let Some(&filler) = tokens.get(i)
      && (filler.get_catcode() == Catcode::SPACE || filler.defined_as(&T_CS!("\\relax")))
    {
      out.push(filler);
      i += 1;
    }
    // A brace (or nothing) where the delimiter belongs is the command's own "Missing delimiter".
    let Some(&delimiter) = tokens.get(i) else {
      break;
    };
    if matches!(delimiter.get_catcode(), Catcode::BEGIN | Catcode::END) {
      continue;
    }
    out.push(delimiter);
    i += 1;
    if delimiter.defined_as(&T_CS!("\\delimiter")) {
      i = take_number(&tokens, i, &mut out);
    } else if delimiter.defined_as(&T_CS!("\\Udelimiter")) {
      for _ in 0..3 {
        i = take_number(&tokens, i, &mut out);
      }
    }
    out.push(T_CS!("\\lx@bm@delimiter"));
  }
  bold_run(&mut out, cs, bold, std::mem::take(&mut run));
  Tokens::new(out)
}

/// `\lx@bm@delimiter`: the delimiter `\left`, `\middle` or `\right` just emitted — the current node's last element,
/// as Perl's `augmentDelimiterProperties` finds it (TeX_Math.pool.ltxml:800-823) — is made bold, bm.sty's
/// `\bm@delimiter` (:416). The null delimiter (`XMHint`) has no glyph to bold.
fn bold_delimiter(document: &mut Document) -> Result<()> {
  let current = document.get_node().clone();
  if let Some(mut delimiter) = current.get_last_element_child()
    && delimiter.get_name() != "XMHint"
  {
    let bold = document.get_node_font(&delimiter).merge(Font {
      forcebold: Some(true),
      ..Font::default()
    });
    document.set_node_font(&mut delimiter, &bold)?;
  }
  Ok(())
}

/// Defines `\lx@bm@delimiter` (`bold_delimiter`), which the expansion of every [`bold_math`] command uses after a
/// delimiter it sets at the outer level.
pub fn define_bold_delimiter() -> Result<()> {
  DefConstructor!("\\lx@bm@delimiter", "",
    after_construct => sub[document, _whatsit] { bold_delimiter(document)?; },
    reversion => "");
  Ok(())
}

/// The `\bm` a class defines for itself, setting its argument in a box or group ([`BoldArgument::Grouped`]): the World
/// Scientific classes (ws_journal_cls.rs), interact.cls (interact_cls.rs) and RevTeX (revtex4_support_sty.rs, Perl
/// revtex4_support.sty.ltxml:167). Its bold constructor is bm.sty's `\lx@bm@bold`, reverting as `\bm`; a later
/// `\usepackage{bm}` replaces the definition with bm.sty's.
pub fn define_grouped_bm() -> Result<()> {
  DefConstructor!("\\lx@bm@bold{}", "#1", bounded => true, font => { forcebold => true }, alias => "\\bm");
  DefMacro!("\\bm{}", sub[(arg)] {
    Ok(bold_math("\\bm", T_CS!("\\lx@bm@bold"), arg, BoldArgument::Grouped))
  }, protected => true);
  define_bold_delimiter()
}

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: bm.sty.ltxml
  // Since we're really punting the whole question of what fonts have
  // bold variants of which characters, this should be enough:
  // `\bm` checks the mode itself (`requireMath`), to name `\bm` in its warning.
  DefConstructor!("\\lx@bm@bold{}", "#1", bounded => true, font => { forcebold => true }, alias => "\\bm");
  // bm.sty:219-221 `\DeclareRobustCommand\bm`: unexpandable in an `\edef`.
  DefMacro!("\\bm{}", sub[(arg)] {
    Ok(bold_math("\\bm", T_CS!("\\lx@bm@bold"), arg, BoldArgument::Scanned))
  }, protected => true);
  define_bold_delimiter()?;
  // bm.sty:229 `\def\bmdefine{\DeclareBoldMathCommand[bold]}`, whose `\bm@define` (:307-316)
  // `\xdef`s the command: global, and overwriting one already defined. Perl's `\newcommand`
  // (bm.sty.ltxml:22) kept a `\bmdefine` inside a group local and refused an existing name.
  DefMacro!("\\bmdefine{}{}", sub[(cs, body)] { bm_define(cs, body) });
  Let!("\\boldsymbol", "\\bm");

  // Should we make a distinction between bold & heavy?
  Let!("\\hm",          "\\bm");
  Let!("\\heavysymbol", "\\boldsymbol");
  Let!("\\hmdefine",    "\\bmdefine");
  Let!("\\heavymath",   "\\boldmath");

  // bm.sty L222-227: `\DeclareBoldMathCommand[<bold|heavy>]\<name>{<body>}`
  // declares `\name` as a bold-math wrapper around `body`. We collapse
  // both bold and heavy variants into our single `\bm` wrapper (we don't
  // distinguish bold vs heavy in the XML output anyway, mirroring the
  // \heavysymbol → \boldsymbol Let above). Surpass-Perl: Perl bm.sty.ltxml
  // doesn't carry `\DeclareBoldMathCommand` either, so this is a Rust-only
  // addition. Witness: arxiv-examples/1205.4484 (macros.tex defines
  // \boldlangle / \boldrangle / \boldlvert / \boldrvert via
  // \DeclareBoldMathCommand).
  DefMacro!("\\DeclareBoldMathCommand[]{}{}", sub[(_weight, cs, body)] { bm_define(cs, body) });
});
