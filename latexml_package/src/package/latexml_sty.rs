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
  // Perl: latexml.sty.ltxml lines 462-512
  // Fast-path implementation: handles simple single-token patterns only.
  // Complex patterns with \WildCard are NOT yet supported.
  {
    use latexml_core::common::def_parser::parse_parameters;
    let lxdeclare_params = parse_parameters(
      "OptionalKeyVals:Declare Undigested", &T_CS!("\\lxDeclare"), true)?;
    def_primitive(
    T_CS!("\\lxDeclare"),
    lxdeclare_params,
    Some(PrimitiveBody::Closure(Rc::new(|args| {
      use latexml_core::definition::argument::ArgWrap;

      // Extract role/name/meaning from KeyVals arg
      let mut role = String::new();
      let mut name_val = String::new();
      let mut meaning = String::new();
      if let ArgWrap::KV(ref kv) = args[0] {
        if let Some(v) = kv.get_value("role") { role = v.to_string(); }
        if let Some(v) = kv.get_value("name") { name_val = v.to_string(); }
        if let Some(v) = kv.get_value("meaning") { meaning = v.to_string(); }
      }

      // Extract the token text from the body (arg[1] is Tokens)
      let body_text = match &args[1] {
        ArgWrap::Tokens(toks) => {
          let s = toks.to_string();
          s.trim_matches('$').trim().to_string()
        },
        _ => String::new(),
      };

      if !body_text.is_empty() && (!role.is_empty() || !name_val.is_empty() || !meaning.is_empty()) {
        // Generate declaration ID (Perl: next_declaration_id via @XMDECL counter)
        let decl_id = {
          let counter = match lookup_value("XMDECL_COUNTER") {
            Some(Stored::Number(n)) => n.value_of() + 1,
            _ => 1,
          };
          assign_value("XMDECL_COUNTER",
            Stored::from(latexml_core::common::number::Number::new(counter)),
            Some(Scope::Global));
          // Perl generates IDs like "S1.XMD4" using \the@XMDECL@ID.
          // We use a simplified form here.
          format!("XMD{counter}")
        };

        // Store as "token_text\trole\tname\tmeaning" in LATEXML_DECLARATIONS
        let key = "LATEXML_DECLARATIONS";
        let mut decls: Vec<String> = match lookup_value(key) {
          Some(Stored::String(s)) => {
            let s_str = arena::with(s, |r| r.to_string());
            if s_str.is_empty() { Vec::new() } else { s_str.split('\n').map(String::from).collect() }
          },
          _ => Vec::new(),
        };
        let decl = format!("{}\t{}\t{}\t{}", body_text, role, name_val, meaning);
        decls.push(decl);
        // Perl: \lxDeclare digests the $...$ body in math mode, producing the
        // mathcode-decoded glyph. If the body_text is a single char with a mathcode,
        // also store the declaration under the decoded character so it matches
        // after mathcode processing (e.g. * → ∗).
        if body_text.chars().count() == 1 {
          let ch = body_text.chars().next().unwrap();
          if let Some(mathcode) = state::lookup_mathcode(&ch.to_string()) {
            if mathcode > 0 {
              let decoded_pos = (mathcode % 256) as u8;
              let decoded_fam = (mathcode / 256) % 16 ;
              // Look up the font encoding for this family to decode the character
              let _style = "text";
              let font_key = format!("textfont_{decoded_fam}");
              if let Some(Stored::Token(ref ftok)) = state::lookup_value(&font_key) {
                state::with_font_info(ftok, |fontinfo| {
                  if let Some(Stored::Font(ref info)) = fontinfo.unwrap_or(None) {
                    if let Some(ref encoding) = info.encoding {
                      if let Some(decoded_char) = latexml_core::common::font::decode(decoded_pos, Some(encoding.to_string()), false) {
                        let decoded_str = decoded_char.to_string();
                        if decoded_str != body_text {
                          let decoded_decl = format!("{}\t{}\t{}\t{}", decoded_str, role, name_val, meaning);
                          decls.push(decoded_decl);
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

        // Also create a Rewrite rule for the XML rewrite phase (matching Perl's
        // createDeclarationRewrite in afterConstruct). This applies role/name/meaning
        // attributes to matching XMTok elements in the final XML tree.
        use latexml_core::rewrite::{Rewrite, RewriteOptions};
        use rustc_hash::FxHashMap;
        let xpath = format!(
          "descendant-or-self::*[local-name()='XMTok' and text()='{}']",
          body_text.replace('\'', "&apos;"));
        let mut attrs = FxHashMap::default();
        if !role.is_empty() { attrs.insert("role".to_string(), role); }
        if !name_val.is_empty() { attrs.insert("name".to_string(), name_val); }
        if !meaning.is_empty() { attrs.insert("meaning".to_string(), meaning); }
        // Perl: createDeclarationRewrite adds decl_id pointing to the <declare> element
        attrs.insert("decl_id".to_string(), decl_id);
        if !attrs.is_empty() {
          let rewrite = Rewrite::new("math", RewriteOptions {
            xpath: Some(xpath),
            attributes_map: Some(attrs),
            ..RewriteOptions::default()
          });
          // Perl: UnshiftValue (prepend) — declarations go IN FRONT of other rules
          unshift_value("DOCUMENT_REWRITE_RULES", vec![rewrite]);
        }
      }

      Ok(Vec::new())
    }))),
    PrimitiveOptions::default(),
  )?;
  }

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
