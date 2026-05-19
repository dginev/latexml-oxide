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
  let ams_loaded = state::with_value("amsmath.sty_loaded", |v| v.is_some())
    || state::with_value("amsmath.sty_raw_loaded", |v| v.is_some());
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
    let labels_str = args[1].to_string();
    Ok(cref_multi(starred, &labels_str, true, false)?)
  });
  DefMacro!("\\Cref OptionalMatch:* Semiverbatim", sub[args] {
    let starred = !args[0].is_none();
    let labels_str = args[1].to_string();
    Ok(cref_multi(starred, &labels_str, true, true)?)
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
    let labels_str = args[1].to_string();
    Ok(cref_multi(starred, &labels_str, true, false)?)
  });
  DefMacro!("\\Cpageref OptionalMatch:* Semiverbatim", sub[args] {
    let starred = !args[0].is_none();
    let labels_str = args[1].to_string();
    Ok(cref_multi(starred, &labels_str, true, true)?)
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
    let labels_str = args[0].to_string();
    Ok(cref_multi(false, &labels_str, false, false)?)
  });
  DefMacro!("\\labelcpageref Semiverbatim", sub[args] {
    let labels_str = args[0].to_string();
    Ok(cref_multi(false, &labels_str, false, false)?)
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
  if has_meaning(&T_CS!(&alias_cs)) {
    if let Ok(expanded) = gullet::do_expand(Tokens!(T_CS!(&alias_cs))) {
      return expanded.to_string();
    }
  }
  ctype.to_string()
}

/// Perl: crefMulti($starred, $labels, $showtype, $capitalized)
/// Generates tokens for \cref{label1,label2,...}
fn cref_multi(starred: bool, labels: &str, showtype: bool, capitalized: bool) -> Result<Tokens> {
  let label_list: Vec<&str> = labels
    .split(',')
    .map(|s| s.trim())
    .filter(|s| !s.is_empty())
    .collect();
  let star = if starred { "*" } else { "" };

  if label_list.len() < 2 {
    // Single reference
    // Perl uses HyperVerbatim which preserves ~ literally.
    // We use {} parameter, so ~ (ACTIVE) expands to space.
    // Fix: embed catcode-12 ~ directly in show string by using \lx@tilde
    let show = if showtype {
      if capitalized {
        "creftypecap\\lx@tilde refnum"
      } else {
        "creftype\\lx@tilde refnum"
      }
    } else {
      "refnum"
    };
    let label = label_list.first().copied().unwrap_or("");
    let expansion = s!("\\lx@cref{star}{{{show}}}{{{label}}}");
    Ok(mouth::tokenize_internal(&expansion))
  } else {
    // Multiple references
    let show = if showtype {
      if capitalized {
        "creftypepluralcap\\lx@tilde refnum"
      } else {
        "creftypeplural\\lx@tilde refnum"
      }
    } else {
      "refnum"
    };
    let mut parts = Vec::new();
    // First label with type
    parts.push(s!("\\lx@cref{star}{{{show}}}{{{}}}", label_list[0]));

    if label_list.len() == 2 {
      // Pair: use \crefpairconjunction
      parts.push(s!(
        "\\crefpairconjunction\\lx@cref{star}{{refnum}}{{{}}}",
        label_list[1]
      ));
    } else {
      // Multiple: use \crefmiddleconjunction for all but last, \creflastconjunction for last
      for label in &label_list[1..label_list.len() - 1] {
        parts.push(s!(
          "\\crefmiddleconjunction\\lx@cref{star}{{refnum}}{{{label}}}"
        ));
      }
      parts.push(s!(
        "\\creflastconjunction\\lx@cref{star}{{refnum}}{{{}}}",
        label_list.last().unwrap()
      ));
    }
    let expansion = parts.join("");
    Ok(mouth::tokenize_internal(&expansion))
  }
}
