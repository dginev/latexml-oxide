use crate::prelude::*;

// Perl: cleveref.sty.ltxml — cleveref cross-referencing
// Provides \cref, \Cref, \crefrange, etc. with type-aware formatting
#[rustfmt::skip]
LoadDefinitions!({
  // Save original \label
  Let!("\\lx@cleverref@save@label", "\\label");
  // cleveref.sty `\label[type]{key}`: the optional type is the label's reference type, `\cref` naming the object by it
  // (`\label[appendixtable]{tab:a}`, "Appendix Table 1" where the table's own type says "table 1"; 2404.11739,
  // 2605.13648). Perl's binding (cleveref.sty.ltxml:20-22) reads and drops it. The type goes on as `\lx@label@type`, as
  // the sites that move a trailing `\label` give it (sect09.rs `\@caption@postlabel`, OXIDIZED_DESIGN #552). Guard
  // `perfect_kernel_batch77s::cleveref_label_type_names_the_reference`.
  DefMacro!("\\lx@cleverref@label[]", sub[(ty)] {
    let mut tokens = Vec::new();
    if let Some(ty) = ty {
      tokens.extend([T_CS!("\\lx@label@type"), T_BEGIN!()]);
      tokens.extend(ty.unlist());
      tokens.push(T_END!());
    }
    tokens.push(T_CS!("\\lx@cleverref@save@label"));
    Tokens::new(tokens)
  });
  DefMacro!("\\lx@label@type{}", sub[(ty)] {
    if ty.unlist_ref().iter().all(|t| t.get_catcode() == Catcode::SPACE) {
      return Ok(Tokens!());
    }
    let mut out = vec![T_CS!("\\lx@cleverref@labeltype"), T_BEGIN!()];
    out.extend(ty.unlist());
    out.push(T_END!());
    Ok(Tokens::new(out))
  });
  // The object the label names takes the type in its cleveref tags (`creftype`, …, the forms `\cref` prints), the
  // object found as `\lx@label` finds it. cleveref keeps the type per label (its `\newlabel{<key>@cref}`); here it is
  // the object's, which one label names. The other forms (`refnum`, `typerefnum`) are the kernel's, unchanged.
  DefConstructor!("\\lx@cleverref@labeltype{}", sub[document, _args, props] {
    let Some(Stored::VecDigested(tags)) = props.get("cref_tags") else {
      return Ok(());
    };
    let raw_step = raw_refstep_of(props);
    if let Some(savenode) = document.float_to_label_for(raw_step.as_ref().map(|step| step.serial)) {
      // (numbered first, as `\lx@label` numbers it: cleveref's `\label` gives the type before the label;
      // 2506.01072 `\newtcbtheorem`, 2505.20088 `[auto counter]`)
      if let Some(step) = raw_step {
        document.take_raw_refstep_number(&step.tags, step.serial)?;
      }
      let target = document.get_node().clone();
      for ((role, _), tag) in CREF_TAG_ROLES.iter().zip(tags.iter()) {
        let xpath = s!("ltx:tags/ltx:tag[@role='{role}']");
        let olds = document.findnodes(&xpath, Some(&target));
        // A form the object's own type has no name for (its empty tag removed: a tcolorbox counter, numbered by the
        // box's raw step, `\newtcbtheorem[crefname={theorem}{theorems}]` with `label type=<box>`, tcolorbox.sty:2270,
        // :955-968; 2506.01072) is the label's type's.
        if olds.is_empty() {
          if let Some(tags_node) = document.findnodes("ltx:tags", Some(&target)).into_iter().next() {
            document.set_node(&tags_node);
            document.absorb(tag, None)?;
          }
          continue;
        }
        for mut old in olds {
          let Some(tags_node) = old.get_parent() else {
            continue;
          };
          // The new tag in the old one's place; a form the type has no name for (an empty tag, removed) leaves none.
          let before = tags_node.get_child_nodes();
          document.set_node(&tags_node);
          document.absorb(tag, None)?;
          for mut new in tags_node.get_child_nodes() {
            if !before.contains(&new) {
              new.unlink_node();
              old.add_prev_sibling(&mut new).ok();
            }
          }
          document.remove_node(old);
        }
      }
      document.set_node(&savenode);
    }
  },
  reversion => "",
  properties => sub[args] {
    let ty = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    let ty = Tokens::new(ExplodeText!(ty.trim()));
    // Each built as `\lx@make@tags` builds the target's (base_utilities.rs `tag_forms`): a text-mode tag in the
    // neutral font, wherever the label stands (an equation, a theorem's italic body).
    let mut tags = Vec::new();
    for (role, formatter) in CREF_TAG_ROLES {
      let mut tokens = TokenizeInternal!(r"\begingroup\let~\lx@tag@texttilde\lx@tag@intags").unlist();
      tokens.push(T_OTHER!("["));
      tokens.extend(ExplodeText!(role));
      tokens.extend([T_OTHER!("]"), T_BEGIN!(), T_CS!(formatter), T_BEGIN!()]);
      tokens.extend(ty.clone().unlist());
      tokens.extend([T_END!(), T_END!(), T_CS!("\\endgroup")]);
      tags.push(Digest!(Tokens::new(tokens))?);
    }
    let mut props = stored_map!("cref_tags" => Stored::VecDigested(tags),
      "alignmentSkippable" => true, "alignmentPreserve" => true);
    // (the object `\lx@label` names after a raw step, `current_raw_refstep`; 2506.01072, 2505.20088)
    if let Some(step) = current_raw_refstep() {
      props.insert("raw_refstep_tags", Stored::Digested(step.tags));
      props.insert("raw_refstep_serial", Stored::Int(step.serial));
    }
    Ok(props)
  });

  // Load the raw cleveref.sty (for language-dependent definitions)
  // Pretend amsmath is loaded to avoid errors. Per OXIDIZED_DESIGN
  // #23, "amsmath is loaded" means EITHER `amsmath.sty_loaded` OR
  // `amsmath.sty_raw_loaded` is set.
  // (a flag set `false`, `latexml_core::state::mark_unloaded`'s — amsmath loaded beyond a class's original,
  // `amsmath_sty::require_amsmath_beyond_original` — is not loaded)
  let ams_loaded = lookup_bool("amsmath.sty_loaded") || lookup_bool("amsmath.sty_raw_loaded");
  assign_value("amsmath.sty_loaded", true, Some(Scope::Local));
  // The pretense includes amsmath's `parentequation` counter (amsmath.sty:1127), which cleveref's amsmath branch reads
  // for every equation label (cleveref.sty:700-707): "\c@parentequation is not a register" otherwise.
  RawTeX!(r"\@ifundefined{c@parentequation}{\newcounter{parentequation}}{}");
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
  // `\crefalias{<counter>}{<type>}` is raw cleveref's (cleveref.sty:1277-1278, `\def\cref@<counter>@alias{<type>}`),
  // read by `cref_type`: a label stepped by the counter is referenced as the type (`\crefalias{section}{appendix}`
  // after `\appendix`, enumitem lists in a theorem through thmtools' hooks). The binding's no-op (Perl's
  // cleveref.sty.ltxml:114-117 alike; KNOWN_PERL_ERRORS #796) kept every such label its counter's name. Witnesses
  // 1706.03576, 2210.04778, 2002.10700 (`\addtotheorempostheadhook[theorem]{\crefalias{thmlisti}{theorem}}`),
  // 1810.01223 (`\crefalias{section}{appsec}`), 2506.14950 (`\crefalias{lemma}{theorem}`).

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
  for (role, formatter) in CREF_TAG_ROLES {
    AssignMapping!("type_tag_formatter", role => formatter);
  }
});

/// cleveref's tag roles (`type_tag_formatter`) and their formatters: the forms of a target's type `\cref` prints.
const CREF_TAG_ROLES: [(&str, &str); 4] = [
  ("creftype", "\\lx@cleverrefnum@@"),
  ("creftypeplural", "\\lx@cleverrefnumplural@@"),
  ("creftypecap", "\\lx@cleverrefnumcap@@"),
  ("creftypepluralcap", "\\lx@cleverrefnumpluralcap@@"),
];

/// Port of raw cleveref's `\@crefname` core (`\newcommand\crefname[3]` →
/// `\@crefname{cref}{type}{sg}{pl}{}`): define `\<prefix>@<type>@name` and
/// `\<prefix>@<type>@name@plural` from the singular/plural arguments. `prefix` is
/// `"cref"` (for `\crefname`) or `"Cref"` (for `\Crefname`). The names are stored as
/// **raw** token bodies (never expanded), so markup such as `\textsc{lemma}` survives,
/// matching cleveref's `\def`. The raw macro's `\toksdef`/`\expandafter` chains are not
/// reproduced — the same clean approach `thmtools_sty.rs` uses for
/// `\declaretheorem[refname=]`. The other variant, when undefined, is derived as
/// cleveref.sty:1296-1325 derives it (`\cref@othervariant`, :1283-1295): `\Cref@<type>@name` is
/// `\MakeUppercase` before the `\crefname` singular (its first token), and `\cref@…` from a lone
/// `\Crefname` is `\MakeLowercase` before it, or the name itself under `capitalise`
/// (`\crefname{appendixtable}{Appendix Table}{…}`: `\Cref` printed no name; 2404.11739).
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
    singular.clone(),
    None,
  )?;
  def_macro(
    T_CS!(s!("\\{}@{}@name@plural", prefix, ctype)),
    None,
    plural.clone(),
    None,
  )?;
  let other = if prefix == "cref" { "Cref" } else { "cref" };
  if !has_meaning(&T_CS!(s!("\\{}@{}@name", other, ctype))) {
    let change_case = if prefix == "cref" {
      Some(T_CS!("\\MakeUppercase"))
    } else if IsDefined!(&T_CS!("\\if@cref@capitalise"))
      && Expand!(TokenizeInternal!(r"\if@cref@capitalise1\else0\fi"))
        .to_string()
        .trim()
        == "1"
    {
      None
    } else {
      Some(T_CS!("\\MakeLowercase"))
    };
    // Both names change case, or neither, as the singular is empty or not (cleveref.sty:1309 `\expandafter\ifx\@tempa
    // \@empty`, the singular alone): `\crefname{x}{}{xs}` derives the plural unchanged.
    let change_case = change_case.filter(|_| !singular.unlist_ref().is_empty());
    // cleveref applies the case change to the name's first token (`\expandafter\@tempd\@tempa`, :1311-1318): a letter
    // is changed alone ("eq." → "Eq."), a macro is read whole by `\MakeUppercase` (`\crefname{proposition}
    // {\nameProposition}…` → "PROPOSITION", 1704.08542). A command that takes its own argument (`\crefname{line}
    // {\text{line}}…`, 2604.22361; 2206.03535) would be split from it by the kernel's grouped `\MakeUppercase` (latex.ltx
    // :22385 `{{…}}`), an error in pdflatex too, but only where that variant is typeset; our tags typeset every variant,
    // so a name led by a command is changed whole, braced.
    let derive = |name: Tokens| -> Tokens {
      match change_case {
        Some(cs) => {
          let mut tokens = vec![cs];
          if name
            .unlist_ref()
            .first()
            .is_some_and(|t| t.code.is_active_or_cs())
          {
            tokens.push(T_BEGIN!());
            tokens.extend(name.unlist());
            tokens.push(T_END!());
          } else {
            tokens.extend(name.unlist());
          }
          Tokens::new(tokens)
        },
        _ => name,
      }
    };
    def_macro(
      T_CS!(s!("\\{}@{}@name", other, ctype)),
      None,
      derive(singular),
      None,
    )?;
    def_macro(
      T_CS!(s!("\\{}@{}@name@plural", other, ctype)),
      None,
      derive(plural),
      None,
    )?;
  }
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

/// Perl: crefType($type) — resolve type alias. cleveref keys an alias by the counter (`\cref@<counter>@alias`,
/// cleveref.sty:1277-1278, read at `\refstepcounter`, :147-170): a type with no alias of its own takes its
/// counter's (`counter_for_type`: an appendix section is stepped by `section`, so `\crefalias{section}{appsec}` after
/// `\appendix` names it, as pdflatex prints).
fn cref_type(ctype: &str) -> String {
  let counter: Option<String> = lookup_mapping("counter_for_type", ctype)
    .as_ref()
    .map(String::from);
  for key in std::iter::once(ctype.to_string()).chain(counter) {
    let alias_cs = s!("\\cref@{}@alias", key);
    if has_meaning(&T_CS!(&alias_cs))
      && let Ok(expanded) = do_expand(Tokens!(T_CS!(&alias_cs)))
    {
      return expanded.to_string();
    }
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
