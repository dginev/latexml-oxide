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
    if let Some(inner) = cmd.strip_prefix("mathcal{").and_then(|s| s.strip_suffix('}')) {
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
/// Public entry point for the .latexml file loader.
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
      xpath: format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
      pattern_type: "subscript",
      base_text: Some(base),
      sub_text: None,
      accent_name: None,
      has_wildcard: true,
      // Wildcard = child 1 of sibling 2 (the content of POSTSUBSCRIPT XMApp)
      wildcard_paths: Some(vec![vec![2, 1]]),
    };
  }
  // Braced wildcard subscripts: x_{\WildCard}, x_{\WildCard,\WildCard}
  if body_text.contains("_{\\WildCard") {
    if let Some(idx) = body_text.find("_{") {
      let base = body_text[..idx].trim().to_string();
      let base_pred = base_text_predicate(&base);
      let brace_content = &body_text[idx + 2..body_text.len().saturating_sub(1)];
      let nwilds = brace_content.matches("\\WildCard").count();
      let wpaths = if nwilds <= 1 {
        vec![vec![2, 1]]  // child 1 of sibling 2 (POSTSUBSCRIPT content)
      } else {
        (1..=nwilds).map(|i| vec![2, 1, i]).collect()
      };
      return DeclarePattern {
        xpath: format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
        pattern_type: "subscript",
        base_text: Some(base),
        sub_text: None,
        accent_name: None,
        has_wildcard: true,
        wildcard_paths: Some(wpaths),
      };
    }
  }
  // Literal subscript: x_1, x_{1}, x_{2n-1}
  // Pre-parsed: XMTok[x] + XMApp[POSTSUBSCRIPT, XMTok[1]]
  if let Some((base, sub)) = parse_subscript_literal(body_text) {
    let base_pred = format!("text()='{}'", base.replace('\'', "&apos;"));
    return DeclarePattern {
      xpath: format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
      pattern_type: "literal_subscript",
      base_text: Some(base),
      sub_text: Some(sub),
      accent_name: None,
      has_wildcard: false,
      wildcard_paths: None,
    };
  }

  // === Accent patterns ===
  // Wildcard accent: \hat{\WildCard}, \widehat{\WildCard}
  for accent in &["hat", "widehat", "tilde", "bar", "vec", "dot", "ddot", "check", "breve"] {
    let pattern = format!("\\{accent}{{\\WildCard}}");
    if body_text == pattern {
      return DeclarePattern {
        // Broad: match any XMApp. Rust filters by accent name in first child.
        xpath: "descendant-or-self::*[local-name()='XMApp']".to_string(),
        pattern_type: "accent",
        base_text: None,
        sub_text: None,
        accent_name: Some(accent.to_string()),
        has_wildcard: true,
        // Wildcard = child 2 (base content) of the accent XMApp
        wildcard_paths: Some(vec![vec![1, 2]]),
      };
    }
  }
  // Literal accent: \hat{x}, \widehat{x}
  for accent in &["hat", "widehat", "tilde", "bar", "vec", "dot", "ddot", "check", "breve"] {
    if let Some(rest) = body_text.strip_prefix(&format!("\\{accent}{{")) {
      if let Some(inner) = rest.strip_suffix('}') {
        if !inner.contains("WildCard") {
          return DeclarePattern {
            xpath: "descendant-or-self::*[local-name()='XMApp']".to_string(),
            pattern_type: "accent",
            base_text: Some(inner.to_string()),
            sub_text: None,
            accent_name: Some(accent.to_string()),
            has_wildcard: false,
            wildcard_paths: None,
          };
        }
      }
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
        xpath: format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
        pattern_type: "prime",
        base_text: Some(base),
        sub_text: None,
        accent_name: None,
        has_wildcard: false,
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
        xpath: format!("descendant-or-self::*[local-name()='XMTok' and {base_pred}]"),
        pattern_type: "prime",
        base_text: Some(base),
        sub_text: None,
        accent_name: None,
        has_wildcard: false,
        wildcard_paths: None,
      };
    }
  }

  // === Fallback: simple token pattern ===
  // For single characters/words without special structure, match as XMTok by text.
  // This handles DefMathRewrite match strings like 'a', 'f', 'x', etc.
  if !body_text.is_empty() && !body_text.contains('\\') {
    return DeclarePattern {
      xpath: format!(
        "descendant-or-self::*[local-name()='XMTok' and text()='{}']",
        body_text.replace('\'', "&apos;")),
      pattern_type: "simple",
      base_text: None,
      sub_text: None,
      accent_name: None,
      has_wildcard: false,
      wildcard_paths: None,
    };
  }

  // Truly unrecognized pattern (e.g. complex TeX commands without matching rules)
  DeclarePattern {
    xpath: String::new(),
    pattern_type: "unknown",
    base_text: None,
    sub_text: None,
    accent_name: None,
    has_wildcard: false,
    wildcard_paths: None,
  }
}

/// Parse a literal (non-wildcard) subscript pattern like "x_1" or "x_{2n-1}".
/// Returns (base, subscript_content) if recognized.
fn parse_subscript_literal(body_text: &str) -> Option<(String, String)> {
  if body_text.contains("WildCard") { return None; }
  // Check for _ subscript
  let idx = body_text.find('_')?;
  let base = body_text[..idx].trim().to_string();
  if base.is_empty() { return None; }
  let sub = body_text[idx + 1..].trim();
  // Strip braces: {1} → 1, {2n-1} → 2n-1
  let sub = sub.strip_prefix('{').and_then(|s| s.strip_suffix('}')).unwrap_or(sub);
  Some((base, sub.to_string()))
}

LoadDefinitions!({
  // 'nobibtex': used for arXiv-like build harnesses where only ".bbl" is available
  // (bibtex will not be ran). 'bibtex' is the default (try bib, fall back to bbl).
  DeclareOption!("bibtex", {
    AssignValue!("BIB_CONFIG", Stored::Strings(Rc::new([arena::pin("bib"), arena::pin("bbl")])),
      Scope::Global);
  });
  DeclareOption!("nobibtex", {
    AssignValue!("BIB_CONFIG", Stored::Strings(Rc::new([arena::pin("bbl")])), Scope::Global);
  });

  // bibconfig KeyVal: comma-separated list of bib config values
  // e.g. \usepackage[bibconfig=bib,bbl]{latexml}
  // TODO: DefKeyVal!("LTXML", "bibconfig", "Semiverbatim", "", code => ...)
  // For now, the bibtex/nobibtex options cover the main use cases.

  // Lexeme serialization for math formulas
  DeclareOption!("mathlexemes", {
    AssignValue!("LEXEMATIZE_MATH" => true, Scope::Global);
  });

  // Math parser speculation (e.g. possible function detection)
  // Perl: DeclareOption('mathparserspeculate', sub { AssignValue('MATHPARSER_SPECULATE' => 1, 'global'); });
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

  ProcessOptions!();

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

  // \lxDeclare — declare semantic roles for math tokens
  // Perl: latexml.sty.ltxml lines 462-568
  // Creates <declare> elements and rewrite rules for math token annotation.
  // Complex patterns with \WildCard are NOT yet supported.
  DefConstructor!("\\lxDeclare OptionalMatch:* OptionalKeyVals:Declare {}", "",
    mode => "restricted_horizontal",
    reversion => "",
    after_digest => sub[whatsit] {
      // Extract role/name/meaning from KeyVals arg (arg index 2 = keyvals)
      let mut role = String::new();
      let mut name_val = String::new();
      let mut meaning = String::new();
      let mut has_tag = false;
      let mut has_description = false;
      let mut tag_text = String::new();
      let mut description_text = String::new();
      if let Some(kv_arg) = whatsit.get_arg(2) {
        if let DigestedData::KeyVals(ref kv) = kv_arg.data() {
          let hash = kv.get_hash_digested();
          if let Some(v) = hash.get("role") { role = v.to_string(); }
          if let Some(v) = hash.get("name") { name_val = v.to_string(); }
          if let Some(v) = hash.get("meaning") { meaning = v.to_string(); }
          if let Some(v) = hash.get("tag") { has_tag = true; tag_text = v.to_string(); }
          if let Some(v) = hash.get("description") { has_description = true; description_text = v.to_string(); }
          // Store scope option for rewrite rule creation in afterConstruct
          if let Some(v) = hash.get("scope") {
            whatsit.set_property("scope_opt", Stored::from(v.to_string()));
          }
        }
      }
      // Extract body text from arg 3 (the {} body)
      let body_text = whatsit.get_arg(3)
        .map(|a| { let s = a.to_string(); s.trim_matches('$').trim().to_string() })
        .unwrap_or_default();

      // Generate declaration ID if tag or description present
      // Perl: next_declaration_id() → StepCounter('@XMDECL'), return \the@XMDECL@ID
      // Counter @XMDECL is subordinate to section, so it resets per-section:
      //   S1.XMD1, S1.XMD2, ..., S2.XMD1, S2.XMD2, ...
      let decl_id = if has_tag || has_description {
        step_counter("@XMDECL", false)?;
        // Perl: DefMacroI(\@@XMDECL@ID, ..., LookupRegister(\c@@XMDECL)->valueOf)
        // then: ToString(Expand(\the@XMDECL@ID))
        let id = gullet::do_expand(T_CS!("\\the@XMDECL@ID"))
          .ok().map(|t| t.to_string().trim().to_string())
          .unwrap_or_default();
        id
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
            let s_str = arena::with(s, |r| r.to_string());
            if s_str.is_empty() { Vec::new() } else { s_str.split('\n').map(String::from).collect() }
          },
          _ => Vec::new(),
        };
        decls.push(format!("{}\t{}\t{}\t{}", body_text, role, name_val, meaning));
        // Mathcode decoding for single-char bodies
        if body_text.chars().count() == 1 {
          let ch = body_text.chars().next().unwrap();
          if let Some(mathcode) = state::lookup_mathcode(&ch.to_string()) {
            if mathcode > 0 {
              let decoded_pos = (mathcode % 256) as u8;
              let decoded_fam = (mathcode / 256) % 16;
              let font_key = format!("textfont_{decoded_fam}");
              if let Some(Stored::Token(ref ftok)) = state::lookup_value(&font_key) {
                state::with_font_info(ftok, |fontinfo| {
                  if let Some(Stored::Font(ref info)) = fontinfo.unwrap_or(None) {
                    if let Some(ref encoding) = info.encoding {
                      if let Some(dc) = latexml_core::common::font::decode(decoded_pos, Some(encoding.to_string()), false) {
                        let ds = dc.to_string();
                        if ds != body_text {
                          decls.push(format!("{}\t{}\t{}\t{}", ds, role, name_val, meaning));
                        }
                      }
                    }
                  }
                });
              }
            }
          }
        }
        assign_value(key, Stored::String(arena::pin(decls.join("\n"))), Some(Scope::Global));
      }
    },
    after_construct => sub[document, whatsit] {
      // Perl: createDeclarationRewrite — create rewrite rule AND <declare> element
      let role = whatsit.get_property("role").map(|v| v.to_string()).unwrap_or_default();
      let name_val = whatsit.get_property("name").map(|v| v.to_string()).unwrap_or_default();
      let meaning = whatsit.get_property("meaning").map(|v| v.to_string()).unwrap_or_default();
      let body_text = whatsit.get_property("body_text").map(|v| v.to_string()).unwrap_or_default();
      let decl_id = whatsit.get_property("decl_id").map(|v| v.to_string()).unwrap_or_default();

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
      if !body_text.is_empty() && (!role.is_empty() || !name_val.is_empty() || !meaning.is_empty()) {
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
            Some(Scope::Named(arena::pin(format!("id:{section_id}"))))
          } else { None }
        } else { None };
        let mut attrs = FxHashMap::default();
        if !role.is_empty() { attrs.insert("role".to_string(), role); }
        if !name_val.is_empty() { attrs.insert("name".to_string(), name_val); }
        if !meaning.is_empty() { attrs.insert("meaning".to_string(), meaning); }
        if !decl_id.is_empty() { attrs.insert("decl_id".to_string(), decl_id); }
        // Compile pattern: determine XPath, type, filters, wildcard paths
        let has_wildcard = body_text.contains("WildCard");
        let pat = if body_text.contains('_') || body_text.contains('\\') || body_text.contains('\'') {
          compile_declare_pattern(&body_text)
        } else {
          // Simple single-token pattern: match XMTok by text
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
          let rewrite = Rewrite::new("math", RewriteOptions {
            xpath: Some(pat.xpath),
            attributes_map: Some(attrs),
            wildcard_paths: pat.wildcard_paths,
            select_count,
            scope: rewrite_scope,
            ..RewriteOptions::default()
          });
          unshift_value("DOCUMENT_REWRITE_RULES", vec![rewrite]);
        }
      }
    });

  // Perl: DefMacroI('\lxTableRowHead', undef, sub { $alignment->currentColumn->{thead}{row} = 1 })
  // Marks the current column as a row header in alignment/tabular contexts.
  // Usage: >{\lxTableRowHead} in column spec with array.sty
  def_primitive(
    T_CS!("\\lxTableRowHead"),
    None,
    Some(PrimitiveBody::Closure(Rc::new(|_args| {
      if let Some(alignment) = lookup_alignment() {
        if let Some(data) = alignment.alignment_cell() {
          if let Some(col) = data.borrow_mut().current_column() {
            col.thead_in_row = true;
          }
        }
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
    // Extract semantic properties from keyvals
    let mut opts = MathPrimitiveOptions::default();
    if let Some(kv) = params_opt.as_ref() {
      if let Some(v) = kv.get_value("name") { opts.name = Some(v.to_string()); }
      if let Some(v) = kv.get_value("meaning") { opts.meaning = Some(v.to_string()); }
      if let Some(v) = kv.get_value("role") { opts.role = Some(v.to_string()); }
      if let Some(v) = kv.get_value("cd") { opts.omcd = Some(v.to_string()); }
      if let Some(v) = kv.get_value("alias") { opts.alias = Some(v.to_string()); }
    }
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
      let mut href_str = _args.get(1).and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      // Perl: CleanURL — strip whitespace/newlines from URLs
      href_str = href_str.replace(['\n', '\r'], "").trim().to_string();
      Ok(stored_map!("href" => href_str))
    }
  );

  // Perl latexml.sty L109-111
  DefMacro!("\\XML", "\\textsc{xml}");
  DefMacro!("\\SGML", "\\textsc{sgml}");
  DefMacro!("\\HTML", "\\textsc{html}");
});
