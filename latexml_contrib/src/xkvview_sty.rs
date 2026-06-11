use latexml_package::prelude::*;

LoadDefinitions!({
  // Load xkeyval first (provides key definition infrastructure)
  RequirePackage!("xkeyval");
  // Note: we do NOT load the raw xkvview.sty — its TeX macros require
  // internal \XKVV@ variables that our Rust keyval system doesn't set.
  // Instead we implement \xkvview directly as a constructor below.

  // Enable xkvview metadata tracking for key definitions from this point on.
  // Only keys defined AFTER this flag is set will appear in \xkvview output.
  assign_value("XKVVIEW_TRACKING", true, Some(Scope::Global));

  // Override \xkvview to build a table from registered keyval metadata.
  // The Perl xkvview.sty.ltxml intercepts \define@key etc. to store metadata,
  // then the raw TeX xkvview.sty generates a longtable wrapped in \ttfamily.
  // In Rust, we have already stored metadata in keyval::define(), so we generate
  // the XML directly, passing a typewriter Font to absorb_string so the document's
  // font system auto-wraps cell content in <text font="typewriter">.
  {
    let replacement: ReplacementClosure = Rc::new(
      |document: &mut Document, _args: &Vec<Option<Digested>>, props: &SymHashMap<Stored>| {
        use latexml_core::keyval;

        let entries = keyval::enumerate_keyvals();
        if entries.is_empty() {
          return Ok(());
        }

        // Get id from counter (set in after_digest)
        let id_str = match props.get("id") {
          Some(Stored::String(s)) => to_string(*s),
          _ => String::new(),
        };

        // Open <table inlist="lot" xml:id="S0.T1">
        let mut table_attrs = HashMap::default();
        table_attrs.insert("inlist".to_string(), "lot".to_string());
        if !id_str.is_empty() {
          table_attrs.insert("xml:id".to_string(), id_str);
        }
        document.open_element("ltx:table", Some(table_attrs), None)?;

        // Absorb tags (from ref_step_counter, set in after_digest)
        if let Some(Stored::Digested(tags)) = props.get("tags") {
          document.absorb(tags, None)?;
        }

        // Create typewriter font for text wrapping.
        // The document's open_text() compares this against the parent's serif font
        // and auto-opens <text font="typewriter"> elements.
        let tt_font = Font {
          family: Some(Cow::Borrowed("typewriter")),
          ..Font::text_default()
        };
        let tt_props = stored_map!(
          "font" => Stored::Font(Rc::new(tt_font))
        );

        // Open <tabular>
        document.open_element("ltx:tabular", None, None)?;

        // Header row
        document.open_element("ltx:thead", None, None)?;
        document.open_element("ltx:tr", None, None)?;
        for header in &["Key", "Prefix", "Family", "Type", "Default"] {
          let mut td_attrs = HashMap::default();
          td_attrs.insert("align".to_string(), "left".to_string());
          td_attrs.insert("thead".to_string(), "column".to_string());
          document.open_element("ltx:td", Some(td_attrs), None)?;
          document.absorb_string(header, &tt_props)?;
          document.close_element("ltx:td")?;
        }
        document.close_element("ltx:tr")?;
        document.close_element("ltx:thead")?;

        // Body rows
        document.open_element("ltx:tbody", None, None)?;
        let num_entries = entries.len();
        for (i, entry) in entries.iter().enumerate() {
          document.open_element("ltx:tr", None, None)?;
          let border = if i == 0 {
            Some("t")
          } else if i == num_entries - 1 {
            Some("b")
          } else {
            None
          };
          let values = [
            &entry.key,
            &entry.prefix,
            &entry.keyset,
            &entry.kind,
            &entry.default,
          ];
          for val in &values {
            let mut td_attrs = HashMap::default();
            td_attrs.insert("align".to_string(), "left".to_string());
            if let Some(b) = border {
              td_attrs.insert("border".to_string(), b.to_string());
            }
            document.open_element("ltx:td", Some(td_attrs), None)?;
            document.absorb_string(val, &tt_props)?;
            document.close_element("ltx:td")?;
          }
          document.close_element("ltx:tr")?;
        }
        document.close_element("ltx:tbody")?;

        // Close tabular and table
        document.close_element("ltx:tabular")?;
        document.close_element("ltx:table")?;

        Ok(())
      },
    );
    let cs = T_CS!("\\xkvview");
    let paramlist = parse_parameters("{}", &cs, true)?;

    let mut opts = ConstructorOptions::default();
    // Step the table counter during digestion to generate xml:id and <tags>.
    // Mirrors Perl: the raw TeX xkvview.sty wraps content in a longtable
    // environment which implicitly steps the table counter via \caption.
    opts.after_digest.push(Rc::new(|whatsit: &mut Whatsit| {
      let counter_props = ref_step_counter("table", false)?;
      if let Some(tags) = counter_props.get("tags") {
        whatsit.set_property("tags", tags.clone());
      }
      if let Some(id) = counter_props.get("id") {
        whatsit.set_property("id", id.clone());
      }
      Ok(Vec::new())
    }));

    def_constructor(cs, paramlist, Some(replacement), opts);
  }

  // Override \define@key to also store xkvview metadata.
  // The Perl xkvview.sty.ltxml saves the original and wraps it.
  // In Rust, keyval::define() already stores metadata (kind, prefix, keyset, key)
  // so we just need to make sure the raw TeX xkvview.sty doesn't break things.
  // The key definitions go through xkeyval_sty.rs → keyval::define() which now
  // stores all metadata automatically. No additional overrides needed.
});
