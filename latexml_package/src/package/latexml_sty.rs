use crate::prelude::*;

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
        }
      }
      // Extract body text from arg 3 (the {} body)
      let body_text = whatsit.get_arg(3)
        .map(|a| { let s = a.to_string(); s.trim_matches('$').trim().to_string() })
        .unwrap_or_default();

      // Generate declaration ID if tag or description present
      // Perl: next_declaration_id via @XMDECL counter → section-scoped "S1.XMD4"
      // TODO: Use proper section-scoped counter via RefStepID when available
      let decl_id = if has_tag || has_description {
        let n = lookup_int("XMDECL_COUNTER") + 1;
        assign_value("XMDECL_COUNTER",
          Stored::from(latexml_core::common::number::Number::new(n as i64)),
          Some(Scope::Global));
        // Get current section prefix for scoped ID
        let section_prefix = state::lookup_value("current_counter")
          .map(|v| {
            let ctr = v.to_string();
            if ctr.is_empty() { String::new() }
            else {
              let num = lookup_int(&format!("\\c@UN{ctr}"));
              if num > 0 { format!("S{num}.") } else { String::new() }
            }
          })
          .unwrap_or_default();
        format!("{section_prefix}XMD{n}")
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
        let xpath = format!(
          "descendant-or-self::*[local-name()='XMTok' and text()='{}']",
          body_text.replace('\'', "&apos;"));
        let mut attrs = FxHashMap::default();
        if !role.is_empty() { attrs.insert("role".to_string(), role); }
        if !name_val.is_empty() { attrs.insert("name".to_string(), name_val); }
        if !meaning.is_empty() { attrs.insert("meaning".to_string(), meaning); }
        if !decl_id.is_empty() { attrs.insert("decl_id".to_string(), decl_id); }
        if !attrs.is_empty() {
          let rewrite = Rewrite::new("math", RewriteOptions {
            xpath: Some(xpath),
            attributes_map: Some(attrs),
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
