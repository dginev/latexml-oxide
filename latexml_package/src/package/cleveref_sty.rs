use crate::prelude::*;

// Perl: cleveref.sty.ltxml — cleveref cross-referencing
// Provides \cref, \Cref, \crefrange, etc. with type-aware formatting
#[rustfmt::skip]
LoadDefinitions!({
  // Save original \label
  Let!("\\lx@cleverref@save@label", "\\label");
  DefMacro!("\\lx@cleverref@label[]", "\\lx@cleverref@save@label");

  // Load the raw cleveref.sty (for language-dependent definitions)
  // Pretend amsmath is loaded to avoid errors. Per OXIDIZED_DESIGN
  // #23, "amsmath is loaded" means EITHER `amsmath.sty_loaded` OR
  // `amsmath.sty_raw_loaded` is set.
  let ams_loaded = with_value("amsmath.sty_loaded", |v| v.is_some())
    || with_value("amsmath.sty_raw_loaded", |v| v.is_some());
  assign_value("amsmath.sty_loaded", true, Some(Scope::Local));
  InputDefinitions!("cleveref", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  if !ams_loaded {
    assign_value("amsmath.sty_loaded", Stored::None, Some(Scope::Local));
  }

  // Perl L29-30: AtBeginDocument(sub { Let('\label', '\lx@cleverref@label') })
  // Deferred so any later package's `\label` redefinition lands BEFORE
  // cleveref wraps it; eager Let here would clobber the wrong target.
  at_begin_document(TokenizeInternal!(r"\let\label\lx@cleverref@label"))?;

  // Override raw TeX \crefname/\Crefname/\crefalias with safe stubs.
  // The raw cleveref.sty definitions use complex \expandafter chains and
  // \toksdef that cause token consumption bugs with many calls + blank lines.
  // These stubs store the names for reference formatting without the risky
  // raw TeX expansion chains.
  def_macro_noop("\\crefname{}{}{}")?;
  def_macro_noop("\\Crefname{}{}{}")?;
  def_macro_noop("\\crefalias{}{}")?;

  // Helper: produces literal ~ (tilde) as catcode OTHER text.
  // Needed because {} parameter expands ACTIVE ~ to space.
  DefPrimitive!("\\lx@tilde", "~");

  // \lx@cref: the core constructor for cleveref references
  // Perl: DefConstructor('\lx@cref OptionalMatch:* HyperVerbatim Semiverbatim', ...)
  DefConstructor!("\\lx@cref OptionalMatch:* {} Semiverbatim",
    "<ltx:ref labelref='#label' show='#2' ?#1(class='ltx_nolink')() _force_font='true'/>",
    enter_horizontal => true,
    properties => sub[args] {
      let raw = args[2].as_ref().map(|a| a.to_string()).unwrap_or_default();
      let label = clean_label(raw.trim(), None).to_string();
      Ok(stored_map!("label" => label))
    });

  // \cref, \Cref: main user commands
  // Perl: DefMacro('\cref OptionalMatch:* Semiverbatim', sub { crefMulti(...) })
  DefMacro!("\\cref OptionalMatch:* Semiverbatim", sub[args] {
    let starred = !args[0].is_none();
    let label_tokens = args.into_iter().nth(1).and_then(|a| a.owned_tokens()).unwrap_or_default();
    Ok(cref_multi(starred, label_tokens, true, false)?)
  });
  DefMacro!("\\Cref OptionalMatch:* Semiverbatim", sub[args] {
    let starred = !args[0].is_none();
    let label_tokens = args.into_iter().nth(1).and_then(|a| a.owned_tokens()).unwrap_or_default();
    Ok(cref_multi(starred, label_tokens, true, true)?)
  });

  // \crefrange, \Crefrange
  // \crefrange: Perl uses ~ (ACTIVE) in expansion which becomes space via HyperVerbatim.
  // show="creftypeplural refnum" in Perl output (space, not tilde).
  DefMacro!("\\crefrange OptionalMatch:* Semiverbatim Semiverbatim",
    "\\lx@cref#1{creftypeplural~refnum}{#2}\\crefrangeconjunction\\ref{#3}");
  DefMacro!("\\Crefrange OptionalMatch:* Semiverbatim Semiverbatim",
    "\\lx@cref#1{creftypepluralcap~refnum}{#2}\\crefrangeconjunction\\ref{#3}");

  // Page refs (same as regular refs for now)
  DefMacro!("\\cpageref OptionalMatch:* Semiverbatim", sub[args] {
    let starred = !args[0].is_none();
    let label_tokens = args.into_iter().nth(1).and_then(|a| a.owned_tokens()).unwrap_or_default();
    Ok(cref_multi(starred, label_tokens, true, false)?)
  });
  DefMacro!("\\Cpageref OptionalMatch:* Semiverbatim", sub[args] {
    let starred = !args[0].is_none();
    let label_tokens = args.into_iter().nth(1).and_then(|a| a.owned_tokens()).unwrap_or_default();
    Ok(cref_multi(starred, label_tokens, true, true)?)
  });
  DefMacro!("\\cpagerefrange OptionalMatch:* Semiverbatim Semiverbatim",
    "\\lx@cref#1{creftype~refnum}{#2}\\crefrangeconjunction\\lx@cref#1{creftype~refnum}{#3}");
  DefMacro!("\\Cpagerefrange OptionalMatch:* Semiverbatim Semiverbatim",
    "\\lx@cref#1{creftypecap~refnum}{#2}\\crefrangeconjunction\\lx@cref#1{creftype~refnum}{#3}");

  // Name refs
  DefMacro!("\\namecref Semiverbatim",    "\\lx@cref{creftype}{#1}");
  DefMacro!("\\nameCref Semiverbatim",    "\\lx@cref{creftypecap}{#1}");
  DefMacro!("\\namecrefs Semiverbatim",   "\\lx@cref{creftypeplural}{#1}");
  DefMacro!("\\nameCrefs Semiverbatim",   "\\lx@cref{creftypepluralcap}{#1}");
  DefMacro!("\\lcnamecref Semiverbatim",  "\\lx@cref{creftype}{#1}");
  DefMacro!("\\lcnamecrefs Semiverbatim", "\\lx@cref{creftypeplural}{#1}");

  DefMacro!("\\labelcref Semiverbatim", sub[args] {
    let label_tokens = args.into_iter().next().and_then(|a| a.owned_tokens()).unwrap_or_default();
    Ok(cref_multi(false, label_tokens, false, false)?)
  });
  DefMacro!("\\labelcpageref Semiverbatim", sub[args] {
    let label_tokens = args.into_iter().next().and_then(|a| a.owned_tokens()).unwrap_or_default();
    Ok(cref_multi(false, label_tokens, false, false)?)
  });

  DefPrimitive!("\\crefalias{}{}", sub[(_counter, _ctype)] { Ok(Vec::new()) });

  // Type formatter macros
  DefMacro!("\\lx@cleverrefnum@@{}", sub[args] {
    let ctype = cref_type(&args[0].to_string());
    let cs = s!("\\cref@{}@name", ctype);
    if has_meaning(&T_CS!(&cs)) {
      Ok(Tokens!(T_CS!(&cs)))
    } else {
      Ok(Tokens!())
    }
  });
  DefMacro!("\\lx@cleverrefnumplural@@{}", sub[args] {
    let ctype = cref_type(&args[0].to_string());
    let cs = s!("\\cref@{}@name@plural", ctype);
    if has_meaning(&T_CS!(&cs)) {
      Ok(Tokens!(T_CS!(&cs)))
    } else {
      Ok(Tokens!())
    }
  });
  DefMacro!("\\lx@cleverrefnumcap@@{}", sub[args] {
    let ctype = cref_type(&args[0].to_string());
    let cs = s!("\\Cref@{}@name", ctype);
    if has_meaning(&T_CS!(&cs)) {
      Ok(Tokens!(T_CS!(&cs)))
    } else {
      Ok(Tokens!())
    }
  });
  DefMacro!("\\lx@cleverrefnumpluralcap@@{}", sub[args] {
    let ctype = cref_type(&args[0].to_string());
    let cs = s!("\\Cref@{}@name@plural", ctype);
    if has_meaning(&T_CS!(&cs)) {
      Ok(Tokens!(T_CS!(&cs)))
    } else {
      Ok(Tokens!())
    }
  });

  // Register type_tag_formatter mappings
  AssignMapping!("type_tag_formatter", "creftype" => "\\lx@cleverrefnum@@");
  AssignMapping!("type_tag_formatter", "creftypeplural" => "\\lx@cleverrefnumplural@@");
  AssignMapping!("type_tag_formatter", "creftypecap" => "\\lx@cleverrefnumcap@@");
  AssignMapping!("type_tag_formatter", "creftypepluralcap" => "\\lx@cleverrefnumpluralcap@@");
});

/// Perl: crefType($type) — resolve type alias
fn cref_type(ctype: &str) -> String {
  let alias_cs = s!("\\cref@{}@alias", ctype);
  if has_meaning(&T_CS!(&alias_cs))
    && let Ok(expanded) = do_expand(Tokens!(T_CS!(&alias_cs)))
  {
    return expanded.to_string();
  }
  ctype.to_string()
}

/// Perl: crefMulti($starred, $labels, $showtype, $capitalized)
/// Generates tokens for \cref{label1,label2,...}
/// Trim leading/trailing SPACE tokens from a label group.
fn trim_space_tokens(tokens: Tokens) -> Tokens {
  let mut v = tokens.unlist();
  while v
    .first()
    .map(|t| t.get_catcode() == Catcode::SPACE)
    .unwrap_or(false)
  {
    v.remove(0);
  }
  while v
    .last()
    .map(|t| t.get_catcode() == Catcode::SPACE)
    .unwrap_or(false)
  {
    v.pop();
  }
  Tokens::new(v)
}

/// Split a Semiverbatim label argument on top-level OTHER commas, trimming
/// surrounding spaces and dropping empties. Mirrors Perl cleveref `splitLabels`,
/// which splits the **Tokens** object — NOT a stringified form. Critical for
/// labels whose name contains a control word, e.g.
/// `\cref{… the \SW moduli space …}` (`\SW` is a user `\newcommand`): the space
/// after the control word `\SW` is consumed at tokenization, so the tokens are
/// `… \SW moduli …` (no space token between). Stringifying and re-tokenizing
/// (as the old code did via `\lx@cref{…}{label-string}`) rejoins them into a
/// bogus `\SWmoduli` control sequence that then gets DIGESTED → `undefined`.
/// Keeping the original tokens and splicing them straight into the `\lx@cref`
/// invocation (as Perl's `Invocation(T_CS('\lx@cref'), …, $label_tokens)` does)
/// avoids the round-trip entirely. Witness 1704.05859.
fn split_label_tokens(tokens: Tokens) -> Vec<Tokens> {
  let mut groups: Vec<Tokens> = Vec::new();
  let mut cur: Vec<Token> = Vec::new();
  for t in tokens.unlist() {
    if t.get_catcode() == Catcode::OTHER && t.with_str(|s| s == ",") {
      groups.push(Tokens::new(std::mem::take(&mut cur)));
    } else {
      cur.push(t);
    }
  }
  groups.push(Tokens::new(cur));
  groups
    .into_iter()
    .map(trim_space_tokens)
    .filter(|g| !g.is_empty())
    .collect()
}

fn cref_multi(
  starred: bool,
  label_tokens: Tokens,
  showtype: bool,
  capitalized: bool,
) -> Result<Tokens> {
  let labels = split_label_tokens(label_tokens);
  let n = labels.len();
  let mut out: Vec<Token> = Vec::new();

  // Emit one `\lx@cref [*] {<show>} {<label tokens>}`. The `<show>` is a fixed
  // internal string (no user macros) so tokenizing it is safe; the `<label>`
  // is spliced from the ORIGINAL tokens (never re-tokenized), matching Perl's
  // `Invocation(T_CS('\lx@cref'), $starred, T_OTHER(show), $label)`.
  // `\lx@tilde` carries the inter-word `~` (HyperVerbatim in Perl).
  let emit = |out: &mut Vec<Token>, show: &str, label: &Tokens| {
    out.push(T_CS!("\\lx@cref"));
    if starred {
      out.push(T_OTHER!("*"));
    }
    out.push(T_BEGIN!());
    out.extend(mouth::tokenize_internal(show).unlist());
    out.push(T_END!());
    out.push(T_BEGIN!());
    out.extend(label.clone().unlist());
    out.push(T_END!());
  };

  if n < 2 {
    let show = if showtype {
      if capitalized {
        "creftypecap\\lx@tilde refnum"
      } else {
        "creftype\\lx@tilde refnum"
      }
    } else {
      "refnum"
    };
    let empty = Tokens::new(Vec::new());
    let label = labels.first().unwrap_or(&empty);
    emit(&mut out, show, label);
  } else {
    let show = if showtype {
      if capitalized {
        "creftypepluralcap\\lx@tilde refnum"
      } else {
        "creftypeplural\\lx@tilde refnum"
      }
    } else {
      "refnum"
    };
    emit(&mut out, show, &labels[0]);
    if n == 2 {
      out.push(T_CS!("\\crefpairconjunction"));
      emit(&mut out, "refnum", &labels[1]);
    } else {
      for label in &labels[1..n - 1] {
        out.push(T_CS!("\\crefmiddleconjunction"));
        emit(&mut out, "refnum", label);
      }
      out.push(T_CS!("\\creflastconjunction"));
      emit(&mut out, "refnum", &labels[n - 1]);
    }
  }
  Ok(Tokens::new(out))
}
