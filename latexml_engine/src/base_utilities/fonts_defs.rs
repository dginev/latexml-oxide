//! Shared Rust helpers: text-mode re-entry, `neutralize_font`, font and encoding names, `today`, `\def` parameter
//! parsing, box classes.

use super::*;

//======================================================================
// Shared Rust helper functions (moved from base_functions.rs)
// Perl equivalent: LaTeXML::Package.pm utility exports
//======================================================================

pub fn reenter_text_mode(vertical_mode: bool) {
  let mode_key = if vertical_mode {
    "VTEXT_MODE_BINDINGS"
  } else {
    "HTEXT_MODE_BINDINGS"
  };
  let text_key = "TEXT_MODE_BINDINGS";
  let mode_bindings = checkout_value(mode_key);
  let text_bindings = checkout_value(text_key);
  let mut bindings: VecDeque<&Stored> = match mode_bindings {
    Some(Stored::VecDequeStored(ref vdq)) => vdq.iter().collect::<VecDeque<&Stored>>(),
    _ => VecDeque::new(),
  };
  if let Some(Stored::VecDequeStored(ref vdq)) = text_bindings {
    bindings.extend(vdq.iter());
  }
  for binding in bindings {
    if let Stored::Tokens(tks) = binding {
      let vec = tks.unlist_ref();
      let_i(&vec[0], &vec[1], None);
    }
  }
  if let Some(value) = mode_bindings {
    checkin_value(mode_key, value);
  }
  if let Some(value) = text_bindings {
    checkin_value(text_key, value);
  }
}

// Similarly, for metadata appearing within peculiar environments, fonts, etc
// You'll typically want this within a group or bounded=>1.
/// Reset the text and math fonts to their defaults, **locally**.
///
/// For metadata that must not inherit the font it happens to be written in —
/// a title picked up inside an italic environment, an author in a small-caps
/// box. The assignments are group-local, so use this inside a group (or a
/// `bounded => 1` definition), otherwise the reset outlives the construct that
/// wanted it.
///
/// The NFSS codes follow, as `\reset@font` (`\normalfont`, latex.ltx:14122)
/// sets them where LaTeX resets a note's font (`\@footnotetext`,
/// latex.ltx:17658-17659 `\reset@font\footnotesize`): else a size switch in
/// the note, which ends in `\selectfont`, re-selected the surrounding family,
/// series or shape (`\textit{A\footnote{\footnotesize note}}` came out
/// italic).
///
/// The encoding is `\encodingdefault`, which `\reset@font` selects (`\fontencoding\encodingdefault`,
/// latex.ltx:14113-14122): a document's text encoding — fontenc's last option, fontspec's TU, babel's `\latinencoding`
/// (babel.sty:3936-3938), greek.ldf's `\greekfontencoding` (:151-153) — not the TeX-wide OT1, else a T1 document's
/// footnotes (`\reset@font`, latex.ltx:17659) typeset `<b> x|y` through OT1 as `¡b¿ x—y` (Perl alike,
/// TeX_Box.pool.ltxml:218-221). LaTeXML's tags neutralize too, where a LaTeX label inherits the body's encoding. Plain
/// TeX has no `\encodingdefault`: OT1.
///
/// The family, series and shape are likewise the document's `\familydefault`, `\seriesdefault` and `\shapedefault`
/// that `\reset@font` selects (`\edef\f@family{\familydefault}`, latex.ltx:14115-14117), where they name a font
/// LaTeXML knows: a `\renewcommand\familydefault{\sfdefault}` document's footnotes and tags are sans, as its PDF
/// (CMSS8 footnote text), not Perl's serif `textDefault`. OXIDIZED_DESIGN_DIVERGENCES #779; guard
/// `perfect_kernel_batch77u::family_default_sets_the_body`.
pub fn neutralize_font() {
  let mut font = Font::text_default();
  if let Some(encoding) = encoding_named("\\encodingdefault") {
    font.encoding = Some(Cow::Owned(encoding));
  }
  let components = [
    font_code_named("\\familydefault").and_then(|code| font::lookup_font_family(&code)),
    font_code_named("\\seriesdefault").and_then(|code| font::lookup_font_series(&code)),
    font_code_named("\\shapedefault").and_then(|code| font::lookup_font_shape(&code)),
  ];
  for component in components.into_iter().flatten() {
    font = font.merge_ref(component);
  }
  sync_nfss_font_codes(&font);
  assign_value("font", font, Some(Scope::Local));
  assign_value("mathfont", Font::math_default(), Some(Scope::Local));
}

/// The NFSS code the macro `cs` (`\encodingdefault`, `\familydefault`, …) names, expanded as `\fontencoding` or
/// `\edef\f@family{…}` expands it. The common cases are read without expanding, as they are on every tag: a literal
/// name (`OT1`), or a chain of parameterless macros ending in one (`\familydefault` → `\rmdefault` → `cmr`). `None`
/// when undefined (plain TeX) or empty.
fn font_code_named(cs: &str) -> Option<String> {
  let mut token = T_CS!(cs);
  let mut literal = None;
  for _ in 0..8 {
    let defn = lookup_definition(&token).ok()??;
    let Some(ExpansionBody::Tokens(body)) = defn.get_expansion() else {
      break;
    };
    if defn
      .get_parameters()
      .is_some_and(|p| !p.get_parameters().is_empty())
    {
      break;
    }
    let tokens = body.unlist_ref();
    if tokens
      .iter()
      .all(|t| matches!(t.get_catcode(), Catcode::LETTER | Catcode::OTHER))
    {
      literal = Some(body.to_string());
      break;
    }
    match tokens[..] {
      [next] if matches!(next.get_catcode(), Catcode::CS) => token = next,
      _ => break,
    }
  }
  let code = match literal {
    Some(code) => code,
    None => do_expand(T_CS!(cs)).ok()?.to_string(),
  };
  let code = code.trim();
  (!code.is_empty()).then(|| code.to_string())
}

/// The encoding the macro `cs` (`\encodingdefault`, `\latinencoding`) names ([`font_code_named`]). `None` when
/// undefined (plain TeX) or not an encoding name.
fn encoding_named(cs: &str) -> Option<String> {
  font_code_named(cs).filter(|encoding| encoding != "ASCII" && !encoding.contains('\\'))
}

/// Today's date as LaTeX's `\today` renders it — `Month D, YYYY`.
///
/// Mirrors Perl `TeX_Job.pool.ltxml` L52-55, reading `\year`/`\month`/`\day`
/// from the **value** table where job startup put them — not from the register
/// meanings, which a class file is free to `\def` out from under us. A missing
/// value falls back to a default instead of aborting the conversion; the body
/// comment records that divergence and its witnesses.
pub fn today() -> Result<String> {
  let month_names = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
  ];
  // Mirror Perl TeX_Job.pool.ltxml L52-55:
  //   $MonthNames[LookupValue('\month')->valueOf - 1]
  //     . " " . LookupValue('\day')->valueOf
  //     . ', ' . LookupValue('\year')->valueOf
  // Read from the VALUE table (assigned in tex_job.rs:47-49 at job
  // startup), NOT the register-meaning table. `lookup_register` walks
  // meanings and returns None when a class file (e.g. iopart.cls)
  // `\def`s `\day` for its own purposes, panicking the unwrap on
  // 1208.0134/1702.02270/1705.08909. Intentional Perl divergence:
  // default to 1900-01-01 when a value is missing, where Perl would
  // die on `Can't call method "valueOf" on undef` — keeps the
  // conversion alive when something has clobbered the value table.
  let read = |key: &str, default: i32| -> i32 {
    match lookup_value(key) {
      Some(Stored::Int(n)) => n as i32,
      Some(Stored::Number(n)) => n.value_of() as i32,
      _ => default,
    }
  };
  let m = read("\\month", 1).clamp(1, 12) as usize;
  let month = month_names[m - 1];
  let day = read("\\day", 1);
  let year = read("\\year", 1900);
  Ok(s!("{} {}, {}", month, day, year))
}

pub fn parse_def_parameters(cs: &Token, params_in: Tokens) -> Result<Option<Parameters>> {
  let mut tokens: VecDeque<Token> = VecDeque::from(params_in.pack_parameters()?.unlist());
  // Now, recognize parameters and delimiters.
  let mut params = Vec::new();
  let mut n = 0;
  while let Some(mut t) = tokens.pop_front() {
    let cc = t.get_catcode();
    if cc == Catcode::PARAM || cc == Catcode::ARG {
      if cc == Catcode::PARAM {
        if tokens.is_empty() {
          // Special case: lone # NOT following a numbered parameter
          // Note that we require a { to appear next, but do NOT read it!
          params.push(Parameter::new(
            Cow::Borrowed("RequireBrace"),
            Cow::Borrowed("RequireBrace"),
            None,
          )?);
          break;
        } else {
          n += 1;
          if let Some(t_next) = tokens.pop_front() {
            t = t_next;
          } else {
            unreachable!("tokens.is_empty() was false, so pop_front must return Some");
          }
        }
      } else {
        // CC_ARG case, keep looking at this token
        n += 1;
      }
      if n > 0 {
        let t_num = t.with_str(|ts| ts.parse::<i8>()).unwrap_or(-1);
        if t_num != n {
          fatal!(
            ParamSpec,
            Expected,
            s!(
              "Parameters for {:?} not in order. Got {:?}, expected {:?}. in {:?}",
              cs,
              t,
              n,
              params
            )
          );
        }
      }
      // Check for delimiting text following the parameter #n
      // Every token of the delimiter is kept, consecutive SPACE tokens
      // included (tex.web §473-476 `scan_toks` reads the parameter text with
      // `get_token`, no collapsing; the tokenizer's `skip_blanks` state has
      // already folded file-sourced runs before they get here). Perl
      // TeX_Macro.pool.ltxml L127 drops the second of two adjacent spaces
      // ("BUT collapse whitespace!"), which only ever bites an
      // expansion-built parameter text: expkv.tex L709-712
      // `\ekv@set@was@blank`'s `#1` delimiter carries TWO real spaces
      // (`{ }` through `\ekv@strip@key` twice), so the collapsed `Until:`
      // never matched, the marker dance derailed (`\ekv@stop`/`\ekv@nil`
      // undefined) and the re-scanned tail ran to `Timeout:TokenLimit`
      // (witness tutodoc-en/fr via clrstrip's `\ekvset{clrstrip}{}`;
      // KNOWN_PERL_ERRORS #119; guard
      // `perfect_kernel_batch53::def_delimiter_keeps_adjacent_spaces`).
      let mut delim = Vec::new();
      while !tokens.is_empty() {
        let inner_cc = tokens.front().unwrap().get_catcode();
        if inner_cc == Catcode::PARAM || inner_cc == Catcode::ARG {
          break;
        }
        delim.push(tokens.pop_front().unwrap());
      }
      // Found text that marks the end of the parameter
      if !delim.is_empty() {
        // tex.web §476: a `#` that ends the parameter text (`#{`) appends the `{` to the last delimiter and to the
        // replacement text, so `\def\pt(#1)#{…}` delimits #1 by `){` and gives the brace back after its expansion
        // (the `Until` reader puts a matched delimiter's closing `{` back, `read_until` matches it as tex.web §392
        // does). Perl (TeX_Macro.pool.ltxml:108-112) reads #1 to the first `)` and then requires a `{`: tkz-euclide's
        // `\def\tkz@DefPoint[#1](#2)#{` (tkz-obj-eu-points.tex:82) split `\tkzDefPoint(2*cos(1),3){T}` at
        // `cos(1` and raised "expected {" (witness 2610.26441, 9 Errors with what followed; KNOWN_PERL_ERRORS #1029).
        // Guards `perfect_kernel_batch78d::hash_brace_extends_the_last_delimiter`, `kernel_def_hash_brace_delimiter`.
        if tokens.len() == 1 && tokens.front().unwrap().get_catcode() == Catcode::PARAM {
          tokens.pop_front();
          delim.push(T_BEGIN!());
        }
        let extra = Tokens::new(delim);
        params.push(
          Parameter {
            name: pin!("Until"),
            spec: pin(format!("Until:{extra}")),
            extra: vec![extra],
            ..Parameter::default()
          }
          .init()?,
        );
      } else if tokens.len() == 1 && tokens.front().unwrap().get_catcode() == Catcode::PARAM {
        // Special case: trailing sole # => delimited by next opening brace.
        tokens.pop_front();
        params.push(Parameter::new("UntilBrace", "UntilBrace", None)?);
      } else {
        // Nothing? Just a plain parameter.
        params.push(Parameter::new("Plain", "{}", None)?);
      }
    } else {
      // Initial delimiting text is required.
      let mut lit: Vec<Token> = vec![t];
      while !tokens.is_empty() {
        let lit_cc = tokens.front().unwrap().get_catcode();
        if lit_cc == Catcode::PARAM || lit_cc == Catcode::ARG {
          break;
        }
        lit.push(tokens.pop_front().unwrap());
      }
      let expected = Tokens::new(lit);
      params.push(
        Parameter {
          name: pin!("Match"),
          spec: pin(s!("Match:{expected}")),
          extra: vec![expected],
          novalue: true,
          ..Parameter::default()
        }
        .init()?,
      );
    }
  }
  // return (@params ? LaTeXML::Core::Parameters->new(@params) : undef);
  if params.is_empty() {
    Ok(None)
  } else {
    Ok(Some(Parameters::new(params)))
  }
}

pub fn do_def(globally: bool, cs: Token, params: Tokens, body: Tokens) -> Result<()> {
  let paramlist = parse_def_parameters(&cs, params)?;
  let scope = if globally { Some(Scope::Global) } else { None };
  install_definition(
    Expandable::new(
      cs,
      paramlist,
      Some(ExpansionBody::Tokens(body)),
      Some(ExpandableOptions {
        nopack_parameters: true,
        tex_declared: true,
        ..ExpandableOptions::default()
      }),
    )?,
    scope,
  );
  after_assignment();
  Ok(())
}

// Kinda rough: We don't really keep track of modes as carefully as TeX does.
// We'll assume that a box is horizontal if there's anything at all,
// but it's not a vbox (!?!?)
pub fn classify_box(boxnum: Number) -> Result<&'static str> {
  with_value(&s!("box{}", boxnum.value_of()), |val_opt| {
    Ok(match val_opt {
      Some(Stored::Digested(d)) => match d.data() {
        DigestedData::Whatsit(w)
          if w.borrow().definition == lookup_definition(&T_CS!("\\vbox"))?.unwrap() =>
        {
          "vbox"
        },
        _ => "hbox",
      },
      _ => "",
    })
  })
}
