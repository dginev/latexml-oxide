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

  // \crefname{type}{singular}{plural} / \Crefname{...}: register the cleveref type
  // name so \cref/\Cref render "<name> <num>". A faithful reimplementation of raw
  // cleveref's \@crefname core (see cref_define_name) replacing the former no-op
  // stubs. The raw macros use \toksdef/\expandafter chains that mis-consumed tokens
  // here; this clean port avoids them (the same approach thmtools_sty.rs already
  // uses for \declaretheorem[refname=]). An explicit \crefname now populates
  // \cref@<type>@name, so it wins over the theorem-heading fallback (OXIDIZED_DESIGN #131).
  DefPrimitive!("\\crefname{}{}{}", sub[(type_arg, sg, pl)] {
    cref_define_name("cref", type_arg, sg, pl)?;
    Ok(Vec::new())
  });
  DefPrimitive!("\\Crefname{}{}{}", sub[(type_arg, sg, pl)] {
    cref_define_name("Cref", type_arg, sg, pl)?;
    Ok(Vec::new())
  });
  // \crefalias is defined below as a 2-arg primitive; leave it as-is.
  def_macro_noop("\\crefalias{}{}")?;

  // Helper: produces the literal `~` (U+007E) as text — a CS so an active `~`
  // can't collapse to a space during tokenization, and a direct Tbox string so
  // it does NOT go through the font map that would turn a `~` char into the
  // tilde accent ˜ (U+02DC). The literal `DefPrimitive!(_, "~")` form reverts the
  // Tbox to its own CS token (`\lx@tilde`), which then leaked into the `tex=`
  // attribute of a `\cref` inside math, where Perl reverts a plain `~`. Give the
  // Tbox a `~` reversion instead so `tex=` matches Perl while the `show`
  // attribute stays U+007E (html_feedback#6876).
  DefPrimitive!("\\lx@tilde", sub[_args] {
    Tbox::new(
      pin_static("~"),
      None,
      None,
      Tokens!(T_OTHER!("~")),
      SymHashMap::default(),
    )
  });

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

  // Type formatter macros. `theorem_fallback` (the singular creftype/creftypecap
  // variants) supplies the surpass-Perl auto-naming — see cleverref_type_name.
  DefMacro!("\\lx@cleverrefnum@@{}", sub[args] {
    Ok(cleverref_type_name(&args[0].to_string(), "cref", "name", true))
  });
  DefMacro!("\\lx@cleverrefnumplural@@{}", sub[args] {
    Ok(cleverref_type_name(&args[0].to_string(), "cref", "name@plural", false))
  });
  DefMacro!("\\lx@cleverrefnumcap@@{}", sub[args] {
    Ok(cleverref_type_name(&args[0].to_string(), "Cref", "name", true))
  });
  DefMacro!("\\lx@cleverrefnumpluralcap@@{}", sub[args] {
    Ok(cleverref_type_name(&args[0].to_string(), "Cref", "name@plural", false))
  });

  // Register type_tag_formatter mappings
  AssignMapping!("type_tag_formatter", "creftype" => "\\lx@cleverrefnum@@");
  AssignMapping!("type_tag_formatter", "creftypeplural" => "\\lx@cleverrefnumplural@@");
  AssignMapping!("type_tag_formatter", "creftypecap" => "\\lx@cleverrefnumcap@@");
  AssignMapping!("type_tag_formatter", "creftypepluralcap" => "\\lx@cleverrefnumpluralcap@@");
});

/// Port of raw cleveref's `\@crefname` core (`\newcommand\crefname[3]` →
/// `\@crefname{cref}{type}{sg}{pl}{}`): define `\<prefix>@<type>@name` and
/// `\<prefix>@<type>@name@plural` from the singular/plural arguments. `prefix` is
/// `"cref"` (for `\crefname`) or `"Cref"` (for `\Crefname`). The names are stored as
/// **raw** token bodies (never expanded), so markup such as `\textsc{lemma}` survives,
/// matching cleveref's `\def`. The raw macro's `\toksdef`/`\expandafter` chains are not
/// reproduced — the same clean approach `thmtools_sty.rs` uses for
/// `\declaretheorem[refname=]`. Like that precedent, the cross-variant `\MakeUppercase`
/// derivation (deriving `\Cref@…` from a lone `\crefname`) is not reproduced; provide
/// `\Crefname` explicitly for the capitalised form.
fn cref_define_name(
  prefix: &str,
  type_arg: Tokens,
  singular: Tokens,
  plural: Tokens,
) -> Result<()> {
  let ctype = do_expand(type_arg)?.to_string();
  let ctype = ctype.trim();
  def_macro(
    T_CS!(s!("\\{}@{}@name", prefix, ctype)),
    None,
    singular,
    None,
  )?;
  def_macro(
    T_CS!(s!("\\{}@{}@name@plural", prefix, ctype)),
    None,
    plural,
    None,
  )?;
  Ok(())
}

/// Resolve a cleveref type-name control sequence (`\cref@<type>@name`,
/// `\Cref@<type>@name@plural`, …) to the tokens the `type_tag_formatter` emits.
///
/// When the primary CS is undefined and `theorem_fallback` is set, fall back to the
/// theorem's stored heading `\lx@name@<type>`. This is a **surpass-Perl** divergence
/// (OXIDIZED_DESIGN #131): real cleveref patches `\@ynthm`/`\@xnthm`/`\@othm` so that
/// `\newtheorem{arch}{Architecture}` auto-registers "Architecture" as the cref name,
/// but LaTeXML's `\newtheorem` (`define_new_theorem`) is a native primitive that never
/// routes through those patches — so Perl and Rust alike leave `\cref@arch@name`
/// undefined and `\cref{...}` renders bare "1" instead of "Architecture 1". LaTeXML
/// already stores the heading as `\lx@name@<type>`, so the singular creftype/creftypecap
/// formatters reuse it, matching the PDF. Only the *singular* names get the fallback:
/// cleveref's theorem patches set only `cref@<type>@name@preamble` (never `@plural`).
/// The heading is emitted verbatim; cleveref's first-letter `capitalize` case transform
/// is not reproduced, so a lowercase-`\cref` under the default (non-`capitalize`) option
/// keeps the heading's own case.
///
/// `\lx@name@<type>` is also set by `\floatname`/`\newfloat` (`float_sty.rs`), so custom
/// floats get the same auto-naming — matching real cleveref; standard figure/table/equation
/// keep their raw-cleveref primary name (fallback stays dormant). An explicit `\crefname`
/// (now a real definition — see `cref_define_name`) populates the primary CS, so it wins
/// over this fallback. Witness arXiv 2305.10391 (`\usepackage[capitalize,…]{cleveref}` +
/// `\newtheorem{arch}{Architecture}`).
fn cleverref_type_name(
  type_arg: &str,
  prefix: &str,
  suffix: &str,
  theorem_fallback: bool,
) -> Tokens {
  let ctype = cref_type(type_arg);
  let cs = s!("\\{}@{}@{}", prefix, ctype, suffix);
  if has_meaning(&T_CS!(&cs)) {
    return Tokens!(T_CS!(&cs));
  }
  if theorem_fallback {
    let name_cs = s!("\\lx@name@{}", ctype);
    if has_meaning(&T_CS!(&name_cs)) {
      return Tokens!(T_CS!(&name_cs));
    }
  }
  Tokens!()
}

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
    out.extend(mouth::tokenize_internal(TeXString::assembled(show.to_string())).unlist());
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
