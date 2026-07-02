use crate::prelude::*;

/// Metadata for a compiled \lxDeclare pattern.
/// Contains the XPath, pattern type for Rust-side filtering, and wildcard info.
pub struct DeclarePattern {
  pub xpath:          String,
  /// "simple", "subscript", "prime", "accent"
  pub pattern_type:   &'static str,
  /// Base token text for subscript/prime/accent base matching (e.g. "x")
  pub base_text:      Option<String>,
  /// For literal subscripts: the subscript content text (e.g. "1")
  pub sub_text:       Option<String>,
  /// For accent patterns: the accent name (e.g. "hat")
  pub accent_name:    Option<String>,
  #[allow(dead_code)]
  pub has_wildcard:   bool,
  pub wildcard_paths: Option<Vec<Vec<usize>>>,
}

/// Generate an XPath text predicate for a base token specification.
fn base_text_predicate(base: &str) -> String {
  if base.starts_with('\\') {
    let cmd = base.trim_start_matches('\\');
    if let Some(inner) = cmd
      .strip_prefix("mathcal{")
      .and_then(|s| s.strip_suffix('}'))
    {
      format!("@font='caligraphic' and text()='{inner}'")
    } else {
      match cmd {
        "varepsilon" => "@meaning='varepsilon'".to_string(),
        _ => format!("@meaning='{cmd}'"),
      }
    }
  } else {
    format!("text()='{}'", base.replace('\'', "&apos;"))
  }
}

/// Compile a \lxDeclare body_text into pattern metadata.
/// Handles both wildcard and non-wildcard patterns.
///
/// Perl: compile_match1 digests tokens to DOM, then domToXPath.
/// Rust: pattern-match on body_text string and generate broad XPath
/// with Rust-side filtering criteria (avoids XPath nested predicate bug).
///
/// Font-awareness is deliberately NOT baked into these XPaths: the serialized
/// `@font` attribute is only finalized after math parsing, so a rewrite-time
/// `@font='…'` predicate matches nothing (and would silently break the
/// wildcard/subscript/prime rewrites). Font discrimination happens Rust-side
/// instead — `declare_node_matches` (rewrite path, via the resolved `_font`
/// id) and `apply_lx_declarations` (post-rewrite fast path, via match_font).
pub fn compile_declare_pattern_pub(body_text: &str) -> DeclarePattern {
  compile_declare_pattern(body_text)
}

fn compile_declare_pattern(body_text: &str) -> DeclarePattern {
  // === Subscript patterns ===
  // IMPORTANT: Rewrites run BEFORE math parsing. The pre-parsed DOM has:
  //   <XMTok>x</XMTok> <XMApp role="POSTSUBSCRIPT"><XMTok>n</XMTok></XMApp>
  // NOT the post-parsed: <XMApp><XMTok role="SUBSCRIPTOP"/><XMTok>x</XMTok><XMTok>n</XMTok></XMApp>
  // Match the BASE XMTok, with select_count=2 to include the POSTSUBSCRIPT sibling.
  // Rust-side filtering verifies the sibling structure.

  // Wildcard: x_\WildCard, \varepsilon_\WildCard, \mathcal{T}_\WildCard
  if let Some(base) = body_text.strip_suffix("_\\WildCard") {
    let base = base.trim().to_string();
    let base_pred = base_text_predicate(&base);
    return DeclarePattern {
      // Match the base XMTok; Rust-side filter checks POSTSUBSCRIPT sibling
      xpath:          format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
      pattern_type:   "subscript",
      base_text:      Some(base),
      sub_text:       None,
      accent_name:    None,
      has_wildcard:   true,
      // Wildcard = child 1 of sibling 2 (the content of POSTSUBSCRIPT XMApp)
      wildcard_paths: Some(vec![vec![2, 1]]),
    };
  }
  // Braced wildcard subscripts: x_{\WildCard}, x_{\WildCard,\WildCard}
  if body_text.contains("_{\\WildCard")
    && let Some(idx) = body_text.find("_{")
  {
    let base = body_text[..idx].trim().to_string();
    let base_pred = base_text_predicate(&base);
    let brace_content = &body_text[idx + 2..body_text.len().saturating_sub(1)];
    let nwilds = brace_content.matches("\\WildCard").count();
    let wpaths = if nwilds <= 1 {
      vec![vec![2, 1]] // child 1 of sibling 2 (POSTSUBSCRIPT content)
    } else {
      (1..=nwilds).map(|i| vec![2, 1, i]).collect()
    };
    return DeclarePattern {
      xpath:          format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
      pattern_type:   "subscript",
      base_text:      Some(base),
      sub_text:       None,
      accent_name:    None,
      has_wildcard:   true,
      wildcard_paths: Some(wpaths),
    };
  }
  // Literal subscript: x_1, x_{1}, x_{2n-1}
  // Pre-parsed: XMTok[x] + XMApp[POSTSUBSCRIPT, XMTok[1]]
  if let Some((base, sub)) = parse_subscript_literal(body_text) {
    let base_pred = format!("text()='{}'", base.replace('\'', "&apos;"));
    return DeclarePattern {
      xpath:          format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
      pattern_type:   "literal_subscript",
      base_text:      Some(base),
      sub_text:       Some(sub),
      accent_name:    None,
      has_wildcard:   false,
      wildcard_paths: None,
    };
  }

  // === Accent patterns ===
  // Wildcard accent: \hat{\WildCard}, \widehat{\WildCard}
  for accent in &[
    "hat", "widehat", "tilde", "bar", "vec", "dot", "ddot", "check", "breve",
  ] {
    let pattern = format!("\\{accent}{{\\WildCard}}");
    if body_text == pattern {
      return DeclarePattern {
        // Broad: match any XMApp. Rust filters by accent name in first child.
        xpath:          "descendant-or-self::*[local-name()='XMApp']".to_string(),
        pattern_type:   "accent",
        base_text:      None,
        sub_text:       None,
        accent_name:    Some(accent.to_string()),
        has_wildcard:   true,
        // Wildcard = child 2 (base content) of the accent XMApp
        wildcard_paths: Some(vec![vec![1, 2]]),
      };
    }
  }
  // Literal accent: \hat{x}, \widehat{x}
  for accent in &[
    "hat", "widehat", "tilde", "bar", "vec", "dot", "ddot", "check", "breve",
  ] {
    if let Some(rest) = body_text.strip_prefix(&format!("\\{accent}{{"))
      && let Some(inner) = rest.strip_suffix('}')
      && !inner.contains("WildCard")
    {
      return DeclarePattern {
        xpath:          "descendant-or-self::*[local-name()='XMApp']".to_string(),
        pattern_type:   "accent",
        base_text:      Some(inner.to_string()),
        sub_text:       None,
        accent_name:    Some(accent.to_string()),
        has_wildcard:   false,
        wildcard_paths: None,
      };
    }
  }

  // === Prime pattern ===
  // x^{\prime} → after parsing: XMApp[SUPERSCRIPTOP, XMTok(x), XMTok(prime)]
  // Match the XMApp with SUPERSCRIPTOP and base text.
  if let Some(base) = body_text.strip_suffix("^{\\prime}") {
    let base = base.trim().to_string();
    if !base.is_empty() && !base.contains('\\') {
      let base_pred = format!("text()='{}'", base.replace('\'', "&apos;"));
      return DeclarePattern {
        // Pre-parsed: XMTok[x] + XMApp[POSTSUPERSCRIPT, XMTok[prime]]
        xpath:          format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
        pattern_type:   "prime",
        base_text:      Some(base),
        sub_text:       None,
        accent_name:    None,
        has_wildcard:   false,
        wildcard_paths: None,
      };
    }
  }
  // Also handle raw prime: x'
  if body_text.ends_with('\'') && body_text.len() > 1 {
    let base = body_text[..body_text.len() - 1].trim().to_string();
    if !base.is_empty() && !base.contains('\\') {
      let base_pred = format!("text()='{}'", base.replace('\'', "&apos;"));
      return DeclarePattern {
        // Pre-parsed: XMTok[x] + XMApp[POSTSUPERSCRIPT, XMTok[prime]]
        xpath:          format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
        pattern_type:   "prime",
        base_text:      Some(base),
        sub_text:       None,
        accent_name:    None,
        has_wildcard:   false,
        wildcard_paths: None,
      };
    }
  }

  // === Bare math symbol command, e.g. "\pi", "\alpha", "\cpi" ===
  // Perl digests $\pi$ and matches the resulting XMTok via domToXPath. In our
  // pre-parse DOM the symbol carries a `name` attribute equal to the control
  // sequence (DefMath sets `name => <cs>`), so keying the match on @name is the
  // string-pattern equivalent and lets \lxDeclare target symbol commands.
  if let Some(cmd) = body_text.strip_prefix('\\')
    && !cmd.is_empty()
    && cmd.chars().all(|c| c.is_ascii_alphabetic())
  {
    return DeclarePattern {
      xpath:          format!(
        "descendant-or-self::*[local-name()='XMTok' and @name='{}']",
        cmd
      ),
      pattern_type:   "simple",
      base_text:      None,
      sub_text:       None,
      accent_name:    None,
      has_wildcard:   false,
      wildcard_paths: None,
    };
  }

  // === Fallback: simple token pattern ===
  // For single characters/words without special structure, match as XMTok by text.
  // This handles DefMathRewrite match strings like 'a', 'f', 'x', etc.
  if !body_text.is_empty() && !body_text.contains('\\') {
    return DeclarePattern {
      xpath:          format!(
        "descendant-or-self::*[local-name()='XMTok' and text()='{}']",
        body_text.replace('\'', "&apos;")
      ),
      pattern_type:   "simple",
      base_text:      None,
      sub_text:       None,
      accent_name:    None,
      has_wildcard:   false,
      wildcard_paths: None,
    };
  }

  // Truly unrecognized pattern (e.g. complex TeX commands without matching rules)
  DeclarePattern {
    xpath:          String::new(),
    pattern_type:   "unknown",
    base_text:      None,
    sub_text:       None,
    accent_name:    None,
    has_wildcard:   false,
    wildcard_paths: None,
  }
}

/// Parse a literal (non-wildcard) subscript pattern like "x_1" or "x_{2n-1}".
/// Returns (base, subscript_content) if recognized.
fn parse_subscript_literal(body_text: &str) -> Option<(String, String)> {
  if body_text.contains("WildCard") {
    return None;
  }
  // Check for _ subscript
  let idx = body_text.find('_')?;
  let base = body_text[..idx].trim().to_string();
  if base.is_empty() {
    return None;
  }
  let sub = body_text[idx + 1..].trim();
  // Strip braces: {1} → 1, {2n-1} → 2n-1
  let sub = sub
    .strip_prefix('{')
    .and_then(|s| s.strip_suffix('}'))
    .unwrap_or(sub);
  Some((base, sub.to_string()))
}

LoadDefinitions!({
  // Perl latexml.sty.ltxml L31-35: ids/noids and comments/nocomments expose
  // two well-known boolean knobs to the document author. Both state keys
  // (GENERATE_IDS, INCLUDE_COMMENTS) are read elsewhere in Rust (document.rs
  // L459 and mouth.rs L358/L696/L889 respectively), so the options were
  // functional but unreachable until wired here.
  DeclareOption!("ids", {
    AssignValue!("GENERATE_IDS"     => true,  Scope::Global);
  });
  DeclareOption!("noids", {
    AssignValue!("GENERATE_IDS"     => false, Scope::Global);
  });
  DeclareOption!("comments", {
    AssignValue!("INCLUDE_COMMENTS" => true,  Scope::Global);
  });
  DeclareOption!("nocomments", {
    AssignValue!("INCLUDE_COMMENTS" => false, Scope::Global);
  });

  // 'nobibtex': used for arXiv-like build harnesses where only ".bbl" is available
  // (bibtex will not be ran). 'bibtex' is the default (try bib, fall back to bbl).
  DeclareOption!("bibtex", {
    AssignValue!(
      "BIB_CONFIG",
      Stored::Strings(Rc::new([pin("bib"), pin("bbl")])),
      Scope::Global
    );
  });
  DeclareOption!("nobibtex", {
    AssignValue!(
      "BIB_CONFIG",
      Stored::Strings(Rc::new([pin("bbl")])),
      Scope::Global
    );
  });

  // Perl L57-59: bibconfig KeyVal — comma-separated bib config values.
  DefKeyVal!("LTXML", "bibconfig", "Semiverbatim");

  // Perl L63-86: Image scaling options — saved as processing instructions
  // via \lx@save@parameter at \begin{document} time. Perl's user-facing
  // keyval name is lowercase `dpi` but the internal PI is uppercase `DPI`
  // (Perl: `$STATE->assignValue(DPI => ...)`). Keep the keyval name
  // lowercase to match Perl user-facing — the uppercase `DPI` mismatch
  // meant `\usepackage[dpi=144]{latexml}` silently missed the keyval.
  DefKeyVal!("LTXML", "dpi", "Number");
  DefKeyVal!("LTXML", "magnify", "Number");
  DefKeyVal!("LTXML", "upsample", "Number");
  DefKeyVal!("LTXML", "zoomout", "Number");

  // Perl L87-98: Limit options — set global limits for infinite-loop protection.
  // These are DefKeyVal with code closures; since our macro doesn't support code,
  // we define them as DeclareOption and handle in ProcessOptions.
  DefKeyVal!("LTXML", "tokenlimit", "Number");
  DefKeyVal!("LTXML", "iflimit", "Number");
  DefKeyVal!("LTXML", "absorblimit", "Number");
  DefKeyVal!("LTXML", "pushbacklimit", "Number");

  // Lexeme serialization for math formulas
  DeclareOption!("mathlexemes", {
    AssignValue!("LEXEMATIZE_MATH" => true, Scope::Global);
  });

  // Math parser speculation (e.g. possible function detection)
  // Perl: DeclareOption('mathparserspeculate', sub { AssignValue('MATHPARSER_SPECULATE' => 1,
  // 'global'); });
  DeclareOption!("mathparserspeculate", {
    AssignValue!("MATHPARSER_SPECULATE" => true, Scope::Global);
  });
  DeclareOption!("nomathparserspeculate", {
    AssignValue!("MATHPARSER_SPECULATE" => false, Scope::Global);
  });

  // Header guessing for tabular environments
  DeclareOption!("guesstabularheaders", {
    AssignValue!("GUESS_TABULAR_HEADERS" => true, Scope::Global);
  });
  DeclareOption!("noguesstabularheaders", {
    AssignValue!("GUESS_TABULAR_HEADERS" => false, Scope::Global);
  });

  // Styling options (Perl PR #2767)
  DeclareOption!("authorsoneline", {
    assign_mapping(
      "DOCUMENT_CLASSES",
      "ltx_authors_1line",
      Some(Stored::Bool(true)),
    );
    assign_mapping("DOCUMENT_CLASSES", "ltx_authors_multiline", None::<Stored>);
  });
  DeclareOption!("authorsmultiline", {
    assign_mapping(
      "DOCUMENT_CLASSES",
      "ltx_authors_multiline",
      Some(Stored::Bool(true)),
    );
    assign_mapping("DOCUMENT_CLASSES", "ltx_authors_1line", None::<Stored>);
  });

  // Finer control over which (if any) raw .sty/.cls files to include
  DeclareOption!("rawstyles", {
    AssignValue!("INCLUDE_STYLES"  => true, Scope::Global);
  });
  DeclareOption!("localrawstyles", {
    AssignValue!("INCLUDE_STYLES"  => "searchpaths", Scope::Global);
  });
  DeclareOption!("norawstyles", {
    AssignValue!("INCLUDE_STYLES"  => false,             Scope::Global);
  });
  DeclareOption!("rawclasses", {
    AssignValue!("INCLUDE_CLASSES" => true,             Scope::Global);
  });
  DeclareOption!("localrawclasses", {
    AssignValue!("INCLUDE_CLASSES" => "searchpaths", Scope::Global);
  });
  DeclareOption!("norawclasses", {
    AssignValue!("INCLUDE_CLASSES" => false, Scope::Global);
  });

  // Perl latexml.sty.ltxml L34-41: tracing / profiling options manipulate
  // a TRACING bitmap via TRACE_ALL / TRACE_PROFILE constants. Rust hasn't
  // wired the bitmap constants (no TRACE_ALL/TRACE_PROFILE symbols in the
  // state module), so stub these as no-op option declarations. The
  // observable effect is that `\usepackage[tracing]{latexml}` etc. simply
  // load latexml.sty without throwing an "unknown option" error; tracing
  // actually kicks in via the CLI `--verbose`/`--profile` flags, not
  // package options. Prevents load-time errors for documents that include
  // these flags defensively.
  DeclareOption!("tracing", None);
  DeclareOption!("notracing", None);
  DeclareOption!("profiling", None);
  DeclareOption!("noprofiling", None);

  // Perl latexml.sty.ltxml L43-44: breakuntex / nobreakuntex toggle the
  // SUPPRESS_UNTEX_LINEBREAKS boolean, which controls whether the `\\`
  // backslash-newline reversion in `tex=` attributes inserts a real
  // line break or is suppressed. Default breakuntex=true (Perl omits the
  // flag by default; documents explicitly passing nobreakuntex enable
  // SUPPRESS).
  DeclareOption!("breakuntex", {
    AssignValue!("SUPPRESS_UNTEX_LINEBREAKS" => false, Scope::Global);
  });
  DeclareOption!("nobreakuntex", {
    AssignValue!("SUPPRESS_UNTEX_LINEBREAKS" => true, Scope::Global);
  });

  ProcessOptions!(keysets => ["LTXML"]);

  // Process bibconfig keyval from options passed to latexml.sty.
  // Perl handles this via \setkeys{LTXML}{...} in the default option handler.
  // ProcessOptions with the LTXML keyset now stores package keyvals here;
  // keep the legacy extraction as a fallback for older call paths.
  if let Some(opts) = lookup_vecdeque("opt@latexml.sty") {
    for opt in opts.iter() {
      let opt_str = opt.to_string();
      if let Some(val) = opt_str.strip_prefix("bibconfig=") {
        assign_value(
          "KV@LTXML@bibconfig",
          Stored::String(pin(val.trim())),
          Some(Scope::Global),
        );
      }
    }
  }

  // Apply bibconfig from keyvals (Perl L57-59: code closure)
  // bibconfig=bbl,bib means try bbl first, fall back to bib
  if let Some(v) = lookup_value("KV@LTXML@bibconfig") {
    let config_str = v.to_string();
    let configs: Vec<_> = config_str.split(',').map(|s| pin(s.trim())).collect();
    if !configs.is_empty() {
      assign_value(
        "BIB_CONFIG",
        Stored::Strings(Rc::from(configs)),
        Some(Scope::Global),
      );
    }
  }

  // Apply limit options from keyvals (Perl L87-98)
  if let Some(v) = lookup_value("KV@LTXML@tokenlimit") {
    let limit = v.to_string().trim().parse::<usize>().unwrap_or(0);
    if limit > 0 {
      set_token_limit(Some(limit));
    }
  }
  if let Some(v) = lookup_value("KV@LTXML@iflimit") {
    let limit = v.to_string().trim().parse::<usize>().unwrap_or(0);
    if limit > 0 {
      assign_value("if_limit", Stored::from(limit as i64), Some(Scope::Global));
    }
  }
  if let Some(v) = lookup_value("KV@LTXML@absorblimit") {
    let limit = v.to_string().trim().parse::<usize>().unwrap_or(0);
    if limit > 0 {
      assign_value(
        "absorb_limit",
        Stored::from(limit as i64),
        Some(Scope::Global),
      );
    }
  }
  if let Some(v) = lookup_value("KV@LTXML@pushbacklimit") {
    let limit = v.to_string().trim().parse::<usize>().unwrap_or(0);
    if limit > 0 {
      set_pushback_limit(Some(limit));
    }
  }

  // Save image scaling parameters as processing instructions.
  // Perl: DefKeyVal with code => AtBeginDocument(\lx@save@parameter{key}{value})
  // Perl stores state under uppercase `DPI` but the keyval is lowercase
  // `dpi`, so lookup uses the keyval (user-facing) name, and the PI emits
  // under the uppercase Perl-internal convention for DPI only.
  for (kv_name, pi_name) in &[
    ("dpi", "DPI"),
    ("magnify", "magnify"),
    ("upsample", "upsample"),
    ("zoomout", "zoomout"),
  ] {
    let key = s!("KV@LTXML@{}", kv_name);
    if let Some(v) = lookup_value(&key) {
      let val = v.to_string().trim().to_string();
      if !val.is_empty() {
        assign_value(
          &s!("PI@latexml@{}", pi_name),
          Stored::String(pin(&val)),
          Some(Scope::Global),
        );
      }
    }
  }

  DefConditional!("\\iflatexml", { true });

  // Perl: NewCounter('@XMDECL', 'section', idprefix => 'XMD');
  // Counter for \lxDeclare IDs, resets per-section (subordinate to section).
  NewCounter!("@XMDECL", "section", idprefix => "XMD");

  // ======================================================================
  // Define the Declare keyval family for \lxDeclare
  DefKeyVal!("Declare", "role", "");
  DefKeyVal!("Declare", "name", "");
  DefKeyVal!("Declare", "meaning", "");
  DefKeyVal!("Declare", "tag", "");
  DefKeyVal!("Declare", "scope", "");
  DefKeyVal!("Declare", "description", "");
  DefKeyVal!("Declare", "nowrap", "");
  DefKeyVal!("Declare", "label", "");
  DefKeyVal!("Declare", "trace", "");
  // Perl: DefKeyVal('Declare', 'replace', 'UndigestedKey') — the replacement
  // pattern is kept as raw tokens and digested at rewrite time (see the
  // replace-closure in \lxDeclare's afterConstruct).
  DefKeyVal!("Declare", "replace", "UndigestedKey");

  // \lxFcn / \lxID / \lxPunct — math-mode role hints (Perl latexml.sty.ltxml).
  // Wrap the argument in <ltx:XMWrap role='...'> so the math grammar
  // treats it as the named role for that occurrence only. `requireMath`
  // forces math context (errors if invoked outside math); `reversion =>
  // '#1'` round-trips just the body (no role wrapper) to TeX; `alias =>
  // ''` suppresses the constructor name in the reversion path.
  // Perl latexml.sty.ltxml L160-163: \lxRegisterNamespace{prefix}{uri}
  // — dynamic XML namespace registration for foreign attributes. Perl's
  // DefPrimitive calls RegisterNamespace(prefix => uri). Rust has
  // latexml_core::common::model::register_namespace exposed; wire it up
  // to the CS so documents using \lxRegisterNamespace{my}{http://…}
  // can then set foreign attributes like my:data='value'.
  DefPrimitive!("\\lxRegisterNamespace {} Semiverbatim", sub[(prefix, uri)] {
    let prefix_str = prefix.to_string();
    let uri_str = uri.to_string();
    model::register_namespace(&prefix_str, Some(&uri_str));
    Ok(Vec::new())
  });

  // Perl latexml.sty.ltxml L236-238: \lxRequireResource[options]{name}
  // adds a document resource (CSS/JS/…). Perl invocation:
  //   RequireResource(ToString(path), ($kv ? $kv->getHash : ()))
  // where the kv hash can carry `type` (mime-type) and `media`. Rust's
  // require_resource takes a `Resource{name, mimetype, media, content}`;
  // the infra lives in latexml_core::binding::content.
  DefPrimitive!("\\lxRequireResource OptionalKeyVals {}", sub[(kv, path)] {
    let name = path.to_string();
    let mimetype = kv.as_ref()
      .and_then(|k| k.get_value("type"))
      .map(|v| v.to_string())
      .unwrap_or_default();
    let media = kv.as_ref()
      .and_then(|k| k.get_value("media"))
      .map(|v| v.to_string())
      .unwrap_or_default();
    require_resource(
      Resource {
        name, mimetype, media, content: String::new(),
      });
    Ok(Vec::new())
  });

  // Perl latexml.sty.ltxml (PR #2767): \lxKeywords{text} — add keywords to
  // the frontmatter.
  DefMacro!("\\lxKeywords{}", "\\lx@add@keywords[name={keywords}]{#1}");

  // Perl latexml.sty.ltxml L249-250: \lxContextTOC — emits a TOC element
  // with format='context'. The matching ltx:TOC schema element already
  // flows through the native schema; previously missing in Rust.
  DefConstructor!("\\lxContextTOC", "<ltx:TOC format='context'/>");

  // Perl latexml.sty.ltxml L166-167: \lxAddClass{class} adds a CSS class
  // to the current element. Rust had this CS completely missing, so
  // documents using `\lxAddClass{ltx_highlight}` hit undefined-CS.
  DefConstructor!("\\lxAddClass Semiverbatim", "",
  after_construct => sub[document, whatsit] {
    let class_tok = whatsit.get_arg(1);
    if let Some(cls) = class_tok {
      let class_str = cls.to_string();
      if let Some(mut element) = document.get_element() {
        let _ = document.add_class(&mut element, &class_str);
      }
    }
  });

  // Perl latexml.sty.ltxml L182-185: \lxWithClass{class}{body} — wraps
  // body in a node with the given CSS class. Perl's getAnnotatableNode
  // detects text-node context and opens <ltx:text> if needed, then
  // addClass on the resulting container. Rust approximates: always
  // wrap in <ltx:text class='#1'>#2</ltx:text>. This is correct for
  // text-mode callers (the common case); in math mode the result
  // diverges (Perl wouldn't wrap, Rust adds an ltx:text inside XMath).
  // No test exercises \lxWithClass, so the approximation is
  // acceptable until the filter_children/absorb pipeline can be
  // wired.
  DefConstructor!(
    "\\lxWithClass Semiverbatim {}",
    "<ltx:text class='#1'>#2</ltx:text>"
  );

  DefConstructor!("\\lxFcn{}", "<ltx:XMWrap role='FUNCTION'>#1</ltx:XMWrap>",
    require_math => true, reversion => "#1", alias => "");
  DefConstructor!("\\lxID{}", "<ltx:XMWrap role='ID'>#1</ltx:XMWrap>",
    require_math => true, reversion => "#1", alias => "");
  DefConstructor!("\\lxPunct{}", "<ltx:XMWrap role='PUNCT'>#1</ltx:XMWrap>",
    require_math => true, reversion => "#1", alias => "");

  // Perl latexml.sty.ltxml L342-350: \lxMathTweak RequiredKeyVals {} —
  // the general form behind \lxFcn/\lxID/\lxPunct. Perl's comment says
  // "same as \lx@math@tweak"; the engine actually has \lx@math@tweaked
  // (base_xmath.rs L527) with the full RequiredKeyVals {} shape and
  // the xmath_copy_keyvals after_digest hook. Let-alias the user-
  // facing name at to the internal one so docs can write
  // `\lxMathTweak{role=POSTFIX}{@}` and get the expected XMWrap.
  Let!("\\lxMathTweak", "\\lx@math@tweaked");

  // \lxDeclare — declare semantic roles for math tokens
  // Perl: latexml.sty.ltxml lines 462-568
  // Creates <declare> elements and rewrite rules for math token annotation.
  // Complex patterns with \WildCard are NOT yet supported.
  DefConstructor!("\\lxDeclare OptionalMatch:* OptionalKeyVals:Declare {}", "",
  mode => "restricted_horizontal",
  reversion => "",
  before_digest => { neutralize_font(); },
  after_digest => sub[whatsit] {
    // Extract role/name/meaning from KeyVals arg (arg index 2 = keyvals)
    let mut role = String::new();
    let mut name_val = String::new();
    let mut meaning = String::new();
    let mut has_tag = false;
    let mut has_description = false;
    let mut tag_text = String::new();
    let mut description_text = String::new();
    // Perl: replace => $kv->getValue('replace') — an UndigestedKey, i.e. raw
    // tokens kept for digestion at replacement time (Core/Rewrite.pm
    // compile_replacement). Capture them (undigested) as an owned local so the
    // keyvals borrow is released before the whatsit is mutated below.
    let mut replace_tks_opt: Option<Tokens> = None;
    if let Some(kv_arg) = whatsit.get_arg(2)
      && let DigestedData::KeyVals(kv) = kv_arg.data() {
        let hash = kv.get_hash_digested();
        replace_tks_opt = kv.get_value("replace").and_then(|a| a.revert().ok());
        if let Some(v) = hash.get("role") { role = v.clone(); }
        if let Some(v) = hash.get("name") { name_val = v.clone(); }
        if let Some(v) = hash.get("meaning") { meaning = v.clone(); }
        if let Some(v) = hash.get("tag") { has_tag = true; tag_text = v.clone(); }
        if let Some(v) = hash.get("description") { has_description = true; description_text = v.clone(); }
        // Store scope option for rewrite rule creation in afterConstruct
        if let Some(v) = hash.get("scope") {
          whatsit.set_property("scope_opt", Stored::from(v.clone()));
        }
      }
    if let Some(replace_tks) = replace_tks_opt {
      whatsit.set_property("replace_tokens", Stored::Tokens(replace_tks));
    }
    // Extract body text from arg 3 (the {} body)
    let body_text = whatsit.get_arg(3)
      .map(|a| { let s = a.to_string(); s.trim_matches('$').trim().to_string() })
      .unwrap_or_default();
    // Capture the digested pattern's font (Perl's domToXPath includes @font in
    // the match, so e.g. an italic `$x$` declaration does NOT match a bold
    // `\mathbf{x}` — fonts carry mathematical meaning). Only \lxDeclare has a
    // digested body to read this from; the .latexml DefMathRewrite loader path
    // (string matches) keeps its font-agnostic behavior via match_font=None.
    let match_font = whatsit
      .get_arg(3)
      .and_then(|a| a.get_font().ok().flatten())
      .map(|f| f.font_attribute_string())
      .filter(|s| !s.is_empty());
    if let Some(ref font_str) = match_font {
      whatsit.set_property("match_font", Stored::from(font_str.clone()));
    }

    // Generate declaration ID if tag or description present
    // Perl: next_declaration_id() → StepCounter('@XMDECL'), return \the@XMDECL@ID
    // Counter @XMDECL is subordinate to section, so it resets per-section:
    //   S1.XMD1, S1.XMD2, ..., S2.XMD1, S2.XMD2, ...
    let decl_id = if has_tag || has_description {
      step_counter("@XMDECL", false)?;
      // Perl: DefMacroI(\@@XMDECL@ID, ..., LookupRegister(\c@@XMDECL)->valueOf)
      // then: ToString(Expand(\the@XMDECL@ID))

      do_expand(T_CS!("\\the@XMDECL@ID"))
        .ok().map(|t| t.to_string().trim().to_string())
        .unwrap_or_default()
    } else {
      String::new()
    };

    // Store properties on the whatsit for constructor body and afterConstruct
    whatsit.set_property("role", Stored::from(role.clone()));
    whatsit.set_property("name", Stored::from(name_val.clone()));
    whatsit.set_property("meaning", Stored::from(meaning.clone()));
    whatsit.set_property("body_text", Stored::from(body_text.clone()));
    whatsit.set_property("decl_id", Stored::from(decl_id.clone()));
    if has_description || has_tag {
      let desc = if !description_text.is_empty() { description_text } else { tag_text };
      whatsit.set_property("description", Stored::from(desc));
    }

    // Store in LATEXML_DECLARATIONS for math parser string-based lookup
    if !body_text.is_empty() && (!role.is_empty() || !name_val.is_empty() || !meaning.is_empty()) {
      let key = "LATEXML_DECLARATIONS";
      let mut decls: Vec<String> = match lookup_value(key) {
        Some(Stored::String(s)) => {
          let s_str = with(s, |r| r.to_string());
          if s_str.is_empty() { Vec::new() } else { s_str.split('\n').map(String::from).collect() }
        },
        _ => Vec::new(),
      };
      // Line format: body_text \t role \t name \t meaning \t decl_id \t match_font.
      // The trailing match_font makes apply_lx_declarations font-aware (a plain
      // italic `$x$` must not annotate a bold `\mathbf{x}`), mirroring the
      // font-aware rewrite path (declare_node_matches). Empty when the pattern
      // carried no distinguishing font.
      let match_font_field = match_font.as_deref().unwrap_or("");
      decls.push(format!(
        "{}\t{}\t{}\t{}\t{}\t{}",
        body_text, role, name_val, meaning, decl_id, match_font_field));
      // Mathcode decoding for single-char bodies
      if body_text.chars().count() == 1 {
        let ch = body_text.chars().next().unwrap();
        if let Some(mathcode) = lookup_mathcode(&ch.to_string())
          && mathcode > 0 {
            let decoded_pos = (mathcode % 256) as u8;
            let decoded_fam = (mathcode / 256) % 16;
            let font_key = format!("textfont_{decoded_fam}");
            if let Some(Stored::Token(ref ftok)) = lookup_value(&font_key) {
              // Extract encoding before calling font::decode — decode may
              // trigger preload_font_map → assign_value, and with_font_info
              // holds a State borrow while its closure runs (see
              // mathchar.rs fix for 0711.4787 RefCell panic pattern).
              let mut encoding_opt: Option<String> = with_font_info(ftok, |fontinfo| {
                if let Some(Stored::Font(info)) = fontinfo.unwrap_or(None) {
                  info.encoding.as_ref().map(|s| s.to_string())
                } else {
                  None
                }
              });
              // Fallback (mirror mathchar.rs L862-887): when `fontinfo_<cs>`
              // didn't round-trip through the dump as a `Stored::Font`, but
              // its `font_shared_key_<cs>` pointer DID, derive encoding from
              // the font name via `decode_fontname`. Without this, dump-mode
              // \lxDeclare doesn't add the alternate codepoint pattern (e.g.
              // `*` → `∗`) and overrides on \ast etc. silently fail.
              if encoding_opt.is_none() {
                let shared_key = with_value(
                  &format!("font_shared_key_{}", ftok.with_str(ToString::to_string)),
                  |v| match v {
                    Some(Stored::String(s)) => with(*s, |str| Some(str.to_string())),
                    _ => None,
                  },
                );
                if let Some(sk) = shared_key
                  && let Some(name) = sk.strip_prefix("fontinfo_") {
                    let props = font::decode_fontname(name, None, None);
                    if let Some(props) = props {
                      encoding_opt = props.encoding.as_ref().map(|s| s.to_string());
                    }
                  }
              }
              if let Some(encoding) = encoding_opt {
                let decoded =
                  font::decode(decoded_pos, Some(encoding), false);
                if let Some(dc) = decoded {
                  let ds = dc.to_string();
                  if ds != body_text {
                    // Same 6-field shape (empty decl_id, trailing match_font)
                    // so apply_lx_declarations parses it uniformly.
                    decls.push(format!(
                      "{}\t{}\t{}\t{}\t\t{}",
                      ds, role, name_val, meaning, match_font.as_deref().unwrap_or("")));
                  }
                }
              }
            }
          }
      }
      assign_value(key, Stored::String(pin(decls.join("\n"))), Some(Scope::Global));
    }
  },
  after_construct => sub[document, whatsit] {
    // Perl: createDeclarationRewrite — create rewrite rule AND <declare> element
    let role = whatsit.get_property("role").map(|v| v.to_string()).unwrap_or_default();
    let name_val = whatsit.get_property("name").map(|v| v.to_string()).unwrap_or_default();
    let meaning = whatsit.get_property("meaning").map(|v| v.to_string()).unwrap_or_default();
    let body_text = whatsit.get_property("body_text").map(|v| v.to_string()).unwrap_or_default();
    let decl_id = whatsit.get_property("decl_id").map(|v| v.to_string()).unwrap_or_default();
    // Perl createDeclarationRewrite: a `replace=` declaration provides a
    // replacement for the matched expression instead of adding attributes
    // (the two are mutually exclusive). Recover the raw replacement tokens.
    let replace_tokens: Option<Tokens> = whatsit
      .get_property("replace_tokens")
      .and_then(|v| if let Stored::Tokens(t) = v.as_ref() { Some(t.clone()) } else { None });

    // Create <ltx:declare> element if id is set (tag or description present)
    if !decl_id.is_empty() {
      let desc = whatsit.get_property("description").map(|v| v.to_string()).unwrap_or_default();
      // Perl: floatToElement('ltx:declare') positions at a container that accepts <declare>
      let saved = document.float_to_element("ltx:declare", false)?;
      let mut attrs_map = HashMap::default();
      attrs_map.insert("xml:id".to_string(), decl_id.clone());
      let _decl_node = document.open_element("ltx:declare", Some(attrs_map), None)?;
      if !desc.is_empty() {
        // Insert description text in <ltx:text>
        let _text_node = document.open_element("ltx:text", None, None)?;
        // Add text content directly to the current node
        let font = lookup_font().unwrap_or_default();
        document.open_text(&desc, &font)?;
        document.close_element("ltx:text")?;
      }
      document.close_element("ltx:declare")?;
      if let Some(ref save) = saved {
        document.set_node(save);
      }
    }

    // Create rewrite rule
    let has_annotation = !role.is_empty() || !name_val.is_empty() || !meaning.is_empty();
    if !body_text.is_empty() && (has_annotation || replace_tokens.is_some()) {
      use latexml_core::rewrite::{Rewrite, RewriteOptions};
      use rustc_hash::FxHashMap;
      // Perl: getDeclarationScope — resolve scope=section to current section ID
      // Use decl_id prefix (e.g. "S1" from "S1.XMD1") since it's computed in afterDigest
      // where \thesection@ID is correct. In afterConstruct, it may be stale.
      let scope_val = whatsit.get_property("scope_opt").map(|v| v.to_string()).unwrap_or_default();
      let rewrite_scope = if scope_val == "section" {
        // Extract section prefix from decl_id (e.g. "S1" from "S1.XMD1")
        let section_id = if !decl_id.is_empty() {
          decl_id.split('.').next().unwrap_or("").to_string()
        } else {
          // Fallback: use the node's ancestor section id
          let mut node = document.get_node().clone();
          let mut sid = String::new();
          loop {
            if node.get_name() == "section" {
              if let Some(id) = node.get_property("xml:id").or_else(|| node.get_property("id")) {
                sid = id;
              }
              break;
            }
            match node.get_parent() {
              Some(p) => node = p,
              None => break,
            }
          }
          sid
        };
        if !section_id.is_empty() {
          Some(Scope::Named(pin(format!("id:{section_id}"))))
        } else { None }
      } else { None };
      let mut attrs = FxHashMap::default();
      if !role.is_empty() { attrs.insert("role".to_string(), role); }
      if !name_val.is_empty() { attrs.insert("name".to_string(), name_val); }
      if !meaning.is_empty() { attrs.insert("meaning".to_string(), meaning); }
      if !decl_id.is_empty() { attrs.insert("decl_id".to_string(), decl_id); }
      // Compile pattern: determine XPath, type, filters, wildcard paths.
      // Font-awareness is applied Rust-side (declare_node_matches for the
      // rewrite path, apply_lx_declarations for the post-rewrite fast path) —
      // NOT baked into the XPath, since the serialized `@font` attribute isn't
      // finalized until after math parsing (see compile_declare_pattern).
      let has_wildcard = body_text.contains("WildCard");
      let pat = if body_text.contains('_') || body_text.contains('\\') || body_text.contains('\'') {
        compile_declare_pattern(&body_text)
      } else {
        // Simple single-token pattern: match XMTok by text; the "simple"
        // filter in declare_node_matches rejects non-matching fonts.
        DeclarePattern {
          xpath: format!(
            "descendant-or-self::*[local-name()='XMTok' and text()='{}']",
            body_text.replace('\'', "&apos;")),
          pattern_type: "simple",
          base_text: None,
          sub_text: None,
          accent_name: None,
          has_wildcard: false,
          wildcard_paths: None,
        }
      };
      if pat.xpath.is_empty() {
        // Unrecognized pattern — skip
      } else {
        // Store pattern metadata in attrs for Rust-side filtering in Select handler
        attrs.insert("_declare_type".to_string(), pat.pattern_type.to_string());
        if let Some(ref base) = pat.base_text {
          attrs.insert("_declare_base".to_string(), base.clone());
        }
        if let Some(ref sub) = pat.sub_text {
          attrs.insert("_declare_sub".to_string(), sub.clone());
        }
        if let Some(ref accent) = pat.accent_name {
          attrs.insert("_declare_accent".to_string(), accent.clone());
        }
        if has_wildcard {
          attrs.insert("_wildcard_pattern".to_string(), "1".to_string());
        }
        // Pattern types determine select_count:
        // Subscript/prime patterns match base XMTok + POSTSUBSCRIPT/POSTSUPERSCRIPT sibling
        // (select_count=2, pre-parsed DOM). Accent patterns match the single XMApp.
        let select_count = match pat.pattern_type {
          "literal_subscript" | "prime" | "subscript" => Some(2usize),
          "accent" => Some(1usize),
          _ => None,
        };
        // Perl createDeclarationRewrite: `replace` and `attributes` are
        // mutually exclusive. A `replace=` declaration digests its replacement
        // pattern at rewrite time (compile_replacement) rather than marking
        // attributes on the matched node.
        let rewrite = if let Some(replace_tks) = replace_tokens {
          use latexml_core::rewrite::RewriteReplaceClosure;
          use std::rc::Rc;
          let closure: RewriteReplaceClosure = Rc::new(move |document, _nodes| {
            // Perl Core/Rewrite.pm::compile_replacement (Tokens branch, as
            // fixed by upstream b17cc621): digest the pattern in
            // restricted_horizontal mode with a neutral font; for a math
            // rule, unwrap the outer List (Rust's digest() always wraps its
            // result in a List — the same shape Perl's changed autosimplify
            // now produces), take its single body, then getBody and absorb.
            begin_mode("restricted_horizontal")?;
            neutralize_font();
            let mut rbox = digest(replace_tks.clone())?;
            end_mode("restricted_horizontal")?;
            let unwrapped = if let DigestedData::List(l) = rbox.data() {
              let items = l.borrow().unlist();
              if items.len() == 1 { Some(items[0].clone()) } else { None }
            } else {
              None
            };
            if let Some(u) = unwrapped {
              rbox = u;
            }
            if let Some(body) = rbox.get_body()? {
              rbox = body;
            }
            document.absorb(&rbox, None)?;
            Ok(())
          });
          Rewrite::new("math", RewriteOptions {
            xpath: Some(pat.xpath),
            replace: Some(closure),
            wildcard_paths: pat.wildcard_paths,
            select_count,
            scope: rewrite_scope,
            ..RewriteOptions::default()
          })
        } else {
          Rewrite::new("math", RewriteOptions {
            xpath: Some(pat.xpath),
            attributes_map: Some(attrs),
            wildcard_paths: pat.wildcard_paths,
            select_count,
            scope: rewrite_scope,
            ..RewriteOptions::default()
          })
        };
        unshift_value("DOCUMENT_REWRITE_RULES", vec![rewrite]);
      }
    }
  });

  // Perl latexml.sty.ltxml L300-307: user-facing aliases for
  // \lx@alignment@begin@heading / \lx@alignment@end@heading, which
  // bracket a run of tabular heading rows. The table-foot aliases
  // point at the same two CSes (the Perl convention uses head/foot
  // for clarity; both just toggle the in_tabular_head flag).
  Let!("\\lxBeginTableHead", "\\lx@alignment@begin@heading");
  Let!("\\lxEndTableHead", "\\lx@alignment@end@heading");
  Let!("\\lxBeginTableFoot", "\\lx@alignment@begin@heading");
  Let!("\\lxEndTableFoot", "\\lx@alignment@end@heading");

  // Perl latexml.sty.ltxml L310-313: \lxTableColumnHead — mirrors
  // \lxTableRowHead below but flips thead_in_column instead of
  // thead_in_row on the current column spec.
  def_primitive(
    T_CS!("\\lxTableColumnHead"),
    None,
    Some(PrimitiveBody::Closure(Rc::new(|_args| {
      if let Some(alignment) = lookup_alignment()
        && let Some(data) = alignment.alignment_cell()
        && let Some(col) = data.borrow_mut().current_column()
      {
        col.thead_in_column = true;
      }
      Ok(Vec::new())
    }))),
    PrimitiveOptions::default(),
  )?;

  // Perl: DefMacroI('\lxTableRowHead', undef, sub { $alignment->currentColumn->{thead}{row} = 1 })
  // Marks the current column as a row header in alignment/tabular contexts.
  // Usage: >{\lxTableRowHead} in column spec with array.sty
  def_primitive(
    T_CS!("\\lxTableRowHead"),
    None,
    Some(PrimitiveBody::Closure(Rc::new(|_args| {
      if let Some(alignment) = lookup_alignment()
        && let Some(data) = alignment.alignment_cell()
        && let Some(col) = data.borrow_mut().current_column()
      {
        col.thead_in_row = true;
      }
      Ok(Vec::new())
    }))),
    PrimitiveOptions::default(),
  )?;

  // Perl latexml.sty L354-371: \lxDefMath{\name}[nargs][optional]{presentation}[keyvals]
  // Defines a math macro with semantic annotations (name, meaning, role, etc.)
  DefPrimitive!("\\lxDefMath {} [Number] [] {} OptionalKeyVals:XMath", sub[(cs, nargs, opt, presentation, params_opt)] {
    let cs_name = cs.to_string();
    let n = nargs.value_of() as usize;
    // Extract semantic properties from keyvals.
    // Perl L368 always sets `revert_as => 'context'` so source-export
    // emits the user-defined CS rather than expanding the presentation
    // template (matches the convention for user-defined math macros).
    let mut opts = MathPrimitiveOptions {
      revert_as: Some(Cow::Borrowed("context")),
      ..Default::default()
    };
    if let Some(kv) = params_opt.as_ref() {
      if let Some(v) = kv.get_value("name") { opts.name = Some(v.to_string()); }
      if let Some(v) = kv.get_value("meaning") { opts.meaning = Some(v.to_string()); }
      if let Some(v) = kv.get_value("role") { opts.role = Some(v.to_string()); }
      if let Some(v) = kv.get_value("cd") { opts.omcd = Some(v.to_string()); }
      if let Some(v) = kv.get_value("alias") { opts.alias = Some(v.to_string()); }
    }
    // Perl also extracts `scope` / detects `tag`/`description` keyvals to allocate
    // a decl_id and Digest a follow-up `\@lxDefMathDeclare` invocation. Neither is
    // wired through the Rust DefPrimitive shim yet — needs `next_declaration_id`
    // helper + the \@lxDefMathDeclare constructor port. Track separately.
    // Build parameter spec for n args
    use latexml_core::common::def_parser::parse_parameters;
    let params = if n > 0 {
      let spec = (0..n).map(|_| "{}").collect::<Vec<_>>().join("");
      parse_parameters(&spec, &T_CS!(&cs_name), true)?
    } else {
      None
    };
    // Create the math definition
    let presentation_str = presentation.to_string();
    def_math(
      T_CS!(&cs_name),
      params,
      presentation_str,
      opts,
    )?;
  });

  // Perl latexml.sty L106-108: \URL[text]{href}
  DefConstructor!("\\URL[] Verbatim",
    "<ltx:ref href='#href'>?#1(#1)(#href)</ltx:ref>",
    enter_horizontal => true,
    properties => sub[_args] {
      let mut href_str = _args
        .get(1)
        .and_then(|a| a.as_ref())
        .map(|a| a.to_string())
        .unwrap_or_default();
      // Perl: CleanURL — strip whitespace/newlines from URLs
      href_str = href_str.replace(['\n', '\r'], "").trim().to_string();
      Ok(stored_map!("href" => href_str))
    }
  );

  // Perl latexml.sty.ltxml L122-134: the LaTeXML-logo trio.
  // \LaTeXML expands to \LaTeXML@logo, which lays out a stylized
  // nested-ltx:text pattern (the classic Lamport-style kerning). The
  // Perl `sizer` closure is specific to LaTeXML-Post typesetting layout
  // and not called by the Rust compile-time binding pipeline — omit.
  DefMacro!("\\LaTeXML", "\\LaTeXML@logo");
  DefConstructor!("\\LaTeXML@logo",
    "<ltx:text class='ltx_LaTeXML_logo'>\
       <ltx:text cssstyle='letter-spacing:-0.2em; margin-right:0.1em'>\
         L\
         <ltx:text cssstyle='font-variant:small-caps;' yoffset='0.4ex'>a</ltx:text>\
         T\
         <ltx:text cssstyle='font-variant:small-caps;font-size:120%' yoffset='-0.2ex'>e</ltx:text>\
       </ltx:text>\
       <ltx:text cssstyle='font-size:90%' yoffset='-0.2ex'>XML</ltx:text>\
     </ltx:text>",
    enter_horizontal => true);

  // Perl latexml.sty.ltxml L136-139: \LaTeXMLversion / \LaTeXMLrevision
  // expand to $LaTeXML::VERSION / $LaTeXML::Version::REVISION via
  // ExplodeText. Rust's DefMacro! proc-macro requires a literal body —
  // CARGO_PKG_VERSION can't be env!()'d through it — so we hard-code
  // the latexml_package crate version (kept in sync by humans). Revision
  // is left empty (no git rev exposed at runtime); that makes
  // \LaTeXMLfullversion collapse to just the version string via the
  // `\ifx\expandafter.\LaTeXMLrevision.` guard.
  DefMacro!("\\LaTeXMLversion", "0.4.0");
  def_macro_noop("\\LaTeXMLrevision")?;
  DefMacro!(
    "\\LaTeXMLfullversion",
    "\\LaTeXML (\\LaTeXMLversion\\expandafter\\ifx\\expandafter.\\LaTeXMLrevision.\\else; rev.~\\LaTeXMLrevision\\fi)"
  );

  // Perl latexml.sty.ltxml L227-230: \lxRef{label}{text} — like hyperref's
  // \hyperref but straightforward. Emits <ltx:ref labelref='label'>text</ref>
  // with enter_horizontal so a bare \lxRef between paragraphs doesn't
  // leak out of <ltx:p> (same mode-leak class as hyperref \url cycle 87).
  // CleanLabel normalizes the label for the labelref attribute.
  DefConstructor!("\\lxRef Semiverbatim {}",
    "<ltx:ref labelref='#label'>#2</ltx:ref>",
    enter_horizontal => true,
    properties => sub[args] {
      unpack_opt_ref!(args => label_opt);
      let label = label_opt.as_ref().unwrap().to_string();
      Ok(stored_map!("label" => Stored::String(pin(clean_label(&label, None)))))
    }
  );

  // Perl latexml.sty.ltxml L209-222: \lxAddAnnotation / \lxWithAnnotation
  // add RDFa-ish annotations to the current / enclosing node via the
  // `addAnnotations` helper. That helper isn't ported to Rust yet (see
  // latexml_sty.rs:855 "Track separately" for \@lxDefMathDeclare, same
  // family). Ship arg-consuming stubs so documents invoking
  // \lxAddAnnotation{key=val,...} or \lxWithAnnotation{…}{body} don't
  // hit undefined-CS. The {body} arg passes through for \lxWithAnnotation
  // so the visible content is preserved; the annotation itself is dropped.
  def_macro_noop("\\lxAddAnnotation RequiredKeyVals")?;
  DefMacro!("\\lxWithAnnotation RequiredKeyVals {}", "#2");

  // Perl latexml.sty.ltxml L514-528: \lxRefDeclaration OptionalKeyVals:Declare {}
  // — refers declarations from another document point to labels at the
  // call site, via createDeclarationRewrite + the Declaration_ state
  // registry. Neither helper is ported. Stub as arg-consuming no-op so
  // documents don't hit undefined-CS; annotations won't actually rewrite
  // but the prose renders cleanly.
  def_macro_noop("\\lxRefDeclaration OptionalKeyVals:Declare {}")?;

  // Perl latexml.sty.ltxml L145: \lxDocumentID{id} sets the top-level
  // document's xml:id via a plain TeX `\def` of the internal
  // \thedocument@ID command that \begin{document}'s constructor
  // consults for its `id` property.
  DefMacro!("\\lxDocumentID{}", "\\def\\thedocument@ID{#1}");

  // Perl latexml.sty.ltxml L148: \LXMID{id}{math} associates an
  // identifier with the given math expression. Thin wrapper around
  // the internal \lx@xmarg constructor already emitted elsewhere.
  DefMacro!("\\LXMID{}{}", "\\lx@xmarg{#1}{#2}");

  // Perl latexml.sty.ltxml L153: \LXMRef{id} refers to the math
  // expression associated with id. Thin wrapper around \lx@xmref.
  DefMacro!("\\LXMRef{}", "\\lx@xmref{#1}");

  // Perl latexml.sty L109-116: acronym shortcuts. Prior Rust stopped at
  // \XML / \SGML / \HTML — the remaining \XHTML / \XSLT / \CSS / \MathML
  // / \OpenMath were missing, so documents using them hit undefined-CS
  // errors.
  DefMacro!("\\XML", "\\textsc{xml}");
  DefMacro!("\\SGML", "\\textsc{sgml}");
  DefMacro!("\\HTML", "\\textsc{html}");
  DefMacro!("\\XHTML", "\\textsc{xhtml}");
  DefMacro!("\\XSLT", "\\textsc{xslt}");
  DefMacro!("\\CSS", "\\textsc{css}");
  DefMacro!("\\MathML", "\\texttt{MathML}");
  DefMacro!("\\OpenMath", "\\texttt{OpenMath}");

  // Diagnostic constructor: emits a marker that gets filled with the Marpa parse tree count
  // for the preceding formula, after math parsing completes.
  // Usage: $x^2$ \ltx@count@parses → becomes the count of grammar trees.
  // The math parser sets _parsetrees on each Math element, then a post-parse step
  // in core_interface fills in the markers.
  DefConstructor!("\\ltx@count@parses",
    "<ltx:text class='ltx_count_parses' _parsetrees_marker='true'>0</ltx:text>",
    enter_horizontal => true);

  // Perl latexml.sty.ltxml L263-289: {lxNavbar} / {lxHeader} / {lxFooter}
  // envs accumulate body content into a `navigation` list that
  // insertNavigation (ltx:document afterClose) splices under an
  // <ltx:navigation> wrapper. Rust has no afterClose hook yet and no
  // PushValue-based list accumulator plumbed through the post-pipeline,
  // so a faithful hoisted-navigation output isn't possible yet. Stub
  // as inline-logical-block wrappers that keep body content visible
  // in-flow and prevent undefined-env errors when documents invoke
  // \begin{lxNavbar}.../\begin{lxHeader}.../\begin{lxFooter}... .
  // Intentional divergence: navigation content appears in flow rather
  // than hoisted to a dedicated <ltx:navigation> container. Revisit
  // when the Tag()/afterClose + PushValue list-accumulator machinery
  // is ported.
  // Perl all three envs run `beforeDigest => sub { AssignValue(inPreamble => 0); }`
  // so body content digests as document text even when the env is
  // invoked from the preamble (same pattern as jheppub affiliation
  // and standalone.sty's \@standalone@start@input).
  DefEnvironment!("{lxNavbar}",
    "<ltx:inline-logical-block class='ltx_page_navbar'>#body</ltx:inline-logical-block>",
    before_digest => { AssignValue!("inPreamble" => false); });
  DefEnvironment!("{lxHeader}",
    "<ltx:inline-logical-block class='ltx_page_header'>#body</ltx:inline-logical-block>",
    before_digest => { AssignValue!("inPreamble" => false); });
  DefEnvironment!("{lxFooter}",
    "<ltx:inline-logical-block class='ltx_page_footer'>#body</ltx:inline-logical-block>",
    before_digest => { AssignValue!("inPreamble" => false); });
});
