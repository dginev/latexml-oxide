//! `LoadDefinitions!` part 1: `\lx@ifundefined`, the dash/space primitives, CSS-class and attribute setters, the
//! splitting macros.

use super::*;

pub(in crate::base_utilities) fn load() -> Result<()> {
  //======================================================================
  // LaTeX has a very particular, but useful, notion of "Undefined",
  //    so let's get that squared away at the outset; it's useful for TeX, too!
  //
  // Naturally, it uses \csname to check, which ends up DEFINING the possibly undefined macro as
  // \relax
  // Perl Base_Utility.pool.ltxml L23-31
  DefMacro!("\\lx@ifundefined{}{}{}", sub[(name, if_token, else_token)] {
    let cs = T_CS!(s!("\\{}", Expand!(name).to_string()));
    // Autoload triggers (declared via `def_autoload` at tex.rs:238-247)
    // install a closure under the trigger CS so the package auto-loads on
    // first invocation. Perl scopes the equivalent `DefAutoload` entries
    // to `OmniBus.cls.ltxml`, so for non-OmniBus papers Perl sees these
    // CSes as truly undefined. Mirror Perl by treating an unfired
    // autoload trigger as "undefined" in `\@ifundefined`. We must NOT
    // overwrite the trigger CS with `\relax` in this case (the kernel
    // `\csname X\endcsname \ifx \relax` idiom DOES overwrite, but doing
    // so here would destroy the autoload — subsequent use of the trigger
    // CS would no-op instead of loading its package). Driver:
    // arXiv:2507.23241v1 (smfart.cls) — line 373's
    // `\@ifundefined{numberwithin}` branches wrong when our preloaded
    // `\numberwithin` autoload makes the CS "look" defined, then the
    // `\@gobbletwo` branch eats `\ifx \relax` and orphans `\else` / `\fi`.
    // An autoload TRIGGER counts as "undefined" only while UNFIRED — i.e. its
    // target package has not yet loaded. `def_autoload` stores the package name
    // (`.sty`) as a String; once `<pkg>.sty_loaded`/`_raw_loaded` is set the
    // trigger CS holds the package's real definition and must read as DEFINED
    // (fixes `\@ifundefined{align}` after `\usepackage{amsmath}`). `.pool`
    // triggers keep the legacy Bool form and stay "undefined until used".
    let is_autoload = cs.with_cs_name(|cs_name| {
      let flagged = match lookup_value(&s!("{cs_name}:autoload")) {
        Some(Stored::String(pkg_sym)) => {
          let pkg = with(pkg_sym, |s| s.to_string());
          !lookup_bool(&s!("{pkg}.sty_loaded"))
            && !lookup_bool(&s!("{pkg}.sty_raw_loaded"))
        },
        Some(Stored::Bool(b)) => b,
        _ => false,
      };
      // The flag is only meaningful while the CS still HOLDS the trigger
      // definition. The kernel dump redefines some trigger CSes itself
      // (`\ProvidesExplPackage` et al. are real latex.ltx macros) — then
      // the CS is genuinely defined regardless of the package flag. Compare
      // definition identity against the snapshot def_autoload stored.
      flagged
        && match lookup_value(&s!("{cs_name}:autoload_trigger")) {
          Some(Stored::Expandable(trigger)) => with_meaning(&cs, |m| {
            matches!(m, Some(Stored::Expandable(current)) if Rc::ptr_eq(current, &trigger))
          }),
          None => true, // legacy trigger without snapshot: old behavior
          _ => false,
        }
    });
    if IsDefined!(&cs) && !is_autoload {
      Ok(else_token)
    } else {
      // No `\relax` pollution of the probed name. latex.ltx:1729-1737 defines
      // `\@ifundefined` with `\ifcsname` precisely so a probe leaves the name
      // undefined (the `\csname…\endcsname\relax` form at :1738 is only the
      // pre-`\ifcsname` fallback, and Perl Base_Utility.pool:23-31 still
      // pollutes). polyglossia's gloss-*.ldf:591 `\@ifundefined
      // {initiate@active@char}{\input{babelsh.def}}{}` then hands babelsh.def:1
      // `\ifx\initiate@active@char\@undefined\else\bbl@afterfi\endinput\fi` a
      // `\relax`-valued name, so the shorthand file early-exits with
      // `\bbl@afterfi` undefined (hang, sample; 19 gloss files load it this
      // way). Guard: `perfect_kernel_batch54::ifundefined_does_not_define_the_name`.
      Ok(if_token)
    }
  }, locked=>true);
  // \@ifundefined is a LaTeX-kernel macro, but our amsppt / amssymb
  // bindings invoke it from Plain-TeX/AMSTeX context too — and those
  // contexts don't load latex_constructs (which has the same Let).
  // Surfacing it here makes it available regardless of which constructs
  // pool is loaded. Surpasses Perl's Base_Utility.pool.ltxml gap.
  Let!("\\@ifundefined", "\\lx@ifundefined");

  // Dash and space primitives used by ligatures and other mechanisms.
  // Perl Base_Utility.pool.ltxml L44-45
  DefPrimitive!("\\lx@endash", {
    Tbox::new(
      pin!("\u{2013}"),
      None,
      None,
      Tokens!(T_CS!("\\lx@endash")),
      SymHashMap::default(),
    )
  });
  // Perl Base_Utility.pool.ltxml L46-47
  DefPrimitive!("\\lx@emdash", {
    Tbox::new(
      pin!("\u{2014}"),
      None,
      None,
      Tokens!(T_CS!("\\lx@emdash")),
      SymHashMap::default(),
    )
  });
  // Perl Base_Utility.pool.ltxml L50-52: stand-in for T_ACTIVE('~').
  DefPrimitive!("\\lx@NBSP", {
    Tbox::new(
      pin!("\u{00A0}"),
      None,
      None,
      Tokens!(T_ACTIVE!('~')),
      // `~` is a penalty and glue (latex.ltx:9411-9419): `\unskip` removes the glue.
      stored_map!("isSpace" => true, "isSkip" => true, "width" => Dimension::from_str("0.333em")?),
    )
  }, locked => true);
  // Perl Base_Utility.pool.ltxml L53-55
  DefPrimitive!("\\lx@nobreakspace", {
    Tbox::new(
      pin!("\u{00A0}"),
      None,
      None,
      Tokens!(T_CS!("\\lx@nobreakspace")),
      // `~` is a penalty and glue (latex.ltx:9411-9419): `\unskip` removes the glue.
      stored_map!("isSpace" => true, "isSkip" => true, "width" => Dimension::from_str("0.333em")?),
    )
  });

  // Perl Base_Utility.pool.ltxml L57-65
  DefPrimitive!("\\lx@ignorehardspaces", {
    let mut boxes = Vec::new();
    while let Some(token) = read_x_token(None, false, None)? {
      boxes = invoke_token(&token)?;
      if boxes.is_empty() {
        break;
      }
      while !boxes.is_empty() {
        if match boxes[0].get_property("isSpace") {
          Some(Cow::Borrowed(Stored::Bool(space_bool))) => *space_bool,
          Some(Cow::Owned(Stored::Bool(ref space_bool))) => *space_bool,
          _ => false,
        } {
          boxes = boxes[1..].to_vec();
        } else {
          break;
        }
      }
      if !boxes.is_empty() {
        break;
      }
    }
    Ok(boxes)
  });

  // Perl Base_Utility.pool.ltxml L42 (PR #2767)
  DefMacro!("\\lx@strip@braces{}", sub[(arg)] {
    Ok(arg.strip_braces())
  });

  // Perl Base_Utility.pool.ltxml L85-87 (renamed from \@ADDCLASS in PR #2767)
  DefConstructor!("\\lx@add@cssclass Semiverbatim", sub[document,args] {
      document.add_class(&mut document.get_element().unwrap(),
        &args[0].as_ref().unwrap().to_string())?;
    }, sizer => 0);

  // Perl Base_Utility.pool.ltxml L101-103 (PR #2767)
  DefConstructor!("\\lx@set@attribute Semiverbatim {}", sub[document,args] {
      let key = args[0].as_ref().map(ToString::to_string).unwrap_or_default();
      let value = args[1].as_ref().map(ToString::to_string).unwrap_or_default();
      if let Some(mut element) = document.get_element() {
        document.set_attribute(&mut element, &key, &value)?;
      }
    }, sizer => 0);

  // Perl Base_Utility.pool.ltxml (PR #2767): split #3 on the delimiter
  // tokens in #2, invoking #1 on each piece.
  // DefMacro('\lx@splitting{}{}{}', ...)
  DefMacro!("\\lx@splitting{}{}{}", sub[(op, delimiters, tokens)] {
    // Note: Perl tests `$delimiters ?` — a Tokens object is always truthy,
    // so the split always applies (an empty delimiter list yields one piece).
    let delims: Vec<SplitDelim> = delimiters.unlist().into_iter().map(SplitDelim::from).collect();
    let mut result: Vec<Token> = Vec::new();
    let line_break = T_CS!("\\\\");
    for (delimiter, piece) in split_tokens_delimited(tokens, delims) {
      // Rust-only (OXIDIZED_DESIGN_DIVERGENCES #804): a piece after a `\\` delimiter (a split at line breaks, which no
      // Perl binding makes: llncs's `\inst` list, webofc's `\author`) starts without the break's `*` and `[…]`, which
      // are no text (`B\inst{2}\\*[2mm] C\inst{1}` made a creator "*[2mm] C"; the rev78e review's finding 6)
      let piece = if delimiter == [line_break] {
        let with_break: Vec<Token> = std::iter::once(line_break).chain(piece.unlist()).collect();
        let mut stripped = strip_linebreak_options(Tokens::new(with_break)).unlist();
        stripped.remove(0);
        while stripped.first() == Some(&T_SPACE!()) {
          stripped.remove(0);
        }
        Tokens::new(stripped)
      } else {
        piece
      };
      result.extend(op.unlist_ref().iter().copied());
      result.push(T_BEGIN!());
      result.extend(piece.unlist());
      result.push(T_END!());
    }
    Ok(Tokens::new(result))
  });

  // Rust-only (OXIDIZED_DESIGN_DIVERGENCES #805): `\lx@split@addresses{op}{list}` — `op{address}` for each address of a
  // class's address list (jheppub's `\emailAdd{a@x, b@y}`, iopart's `\ead{a@x and b@y}`); a list in plain text that
  // is a list by `email_addresses` (a braced group of local parts before a shared domain, `\ead{\{yangqq, yangjh\}
  // @ipp.ac.cn}`, 2607.04241; a semicolon list) each of its addresses written out, as an author block's line gives
  // them, not "{yangqq" and "yangjh}@ipp.ac.cn"; `op{list}` whole when it is no list of addresses (2011.05971's
  // `\emailAdd{nikolay.bobev, anthony.charles, valentin.reys @kuleuven.be}`, the bare local parts no address).
  DefMacro!("\\lx@split@addresses{}{}", sub[(op, list)] {
    // (at the commas and at an " and ": `\ead{cpires@fisica.ufpb.br and psilva@fisica.ufpb.br}`, 0710.2104, 1003.0073)
    let pieces: Vec<Tokens> = split_tokens(list.clone(), vec![
      SplitDelim::Token(T_OTHER!(",")),
      SplitDelim::Tokens(Tokens::new(vec![
        T_SPACE!(),
        T_LETTER!("a"),
        T_LETTER!("n"),
        T_LETTER!("d"),
        T_SPACE!(),
      ])),
    ])
    .into_iter()
    .filter(|piece| !piece.is_empty())
    .collect();
    let addresses = pieces.iter().all(|piece| address_text(piece).contains('@'));
    // (only a list of plain addresses is written out — their letters, the printed braces `\{ \}` and `\_`, the spacing
    // and line breaks between them and an e-mail command's address font, `\mailto` (1105.4742's `\ead{\mailto{idan.oren@…}
    // \\ \mailto{\ \ uzy.smilansky@…}}`): a braced group of local parts, a semicolon list, 0711.4000's `leticia@…;
    // jorge@…` —, so a list in other markup keeps it: `\href{mailto:a@x,b@y}{a@x, b@y}` one contact, rev78f probe j06)
    let plain = list.unlist_ref().iter().all(|t| {
      t.get_catcode() != Catcode::CS
        || is_spacing_command(t)
        || is_line_separator(t)
        || is_email_wrapper(t)
        || t.with_str(|name| matches!(name, "\\{" | "\\}" | "\\_" | "\\lbrace" | "\\rbrace" | "\\,"))
    });
    let pieces = if pieces.len() > 1 && addresses {
      pieces
    } else {
      // (a shared domain's addresses each written out, as an author block's line gives them, `email_line_calls`)
      match email_addresses(&list) {
        Some(each) if each.len() > 1 && plain => each
          .iter()
          .map(|address| Tokens::new(text_tokens(address)))
          .collect(),
        _ => vec![list],
      }
    };
    let mut result: Vec<Token> = Vec::new();
    for piece in pieces {
      result.extend(op.unlist_ref().iter().copied());
      result.push(T_BEGIN!());
      result.extend(piece.unlist());
      result.push(T_END!());
    }
    Ok(Tokens::new(result))
  });
  Ok(())
}
