use crate::prelude::*;
use latexml_core::alignment::template::TemplateConfig;

LoadDefinitions!({
  //======================================================================
  // C.10.1 The tabbing Environment
  // Perl: latex_constructs.pool.ltxml lines 3554-3651
  //======================================================================

  DefRegister!("\\tabbingsep" => Dimension::new(0));

  // Main entry: \tabbing → \par\@tabbing@bindings\@@tabbing\lx@begin@alignment
  DefMacro!("\\tabbing", "\\par\\@tabbing@bindings\\@@tabbing\\lx@begin@alignment");
  DefMacro!("\\endtabbing", "\\lx@end@alignment\\@end@tabbing\\par");

  DefPrimitive!("\\@end@tabbing", sub [_args] {
    egroup()?;
  });

  DefConstructor!("\\@@tabbing SkipSpaces DigestedBody", "#1",
    reversion => "\\begin{tabbing}#1\\end{tabbing}",
    before_digest => sub {
      bgroup();
    },
    mode => "internal_vertical"
  );

  // Wrapper macros that expand to marker + & (column separator)
  DefMacro!("\\@tabbing@tabset", "\\@tabbing@tabset@marker&");
  DefMacro!("\\@tabbing@nexttab", "\\@tabbing@nexttab@marker&");
  DefMacro!("\\@tabbing@newline OptionalMatch:* [Dimension]", "\\@tabbing@newline@marker\\cr");
  DefMacro!("\\@tabbing@kill", "\\@tabbing@kill@marker\\cr\\@tabbing@start@tabs");

  // Marker constructors
  DefConstructor!("\\@tabbing@tabset@marker", "",
    reversion => "\\=",
    properties => { Ok(stored_map!("alignmentSkippable" => true)) }
  );
  DefConstructor!("\\@tabbing@nexttab@marker", "",
    reversion => "\\>",
    properties => { Ok(stored_map!("alignmentSkippable" => true)) }
  );
  DefConstructor!("\\@tabbing@newline@marker", "",
    reversion => "\\\\"
  );
  DefConstructor!("\\@tabbing@kill@marker", "",
    reversion => "\\kill",
    after_digest => sub [_whatsit] {
      // Perl: LookupValue('Alignment')->removeRow
      if let Some(alignment_stored) = lookup_alignment() {
        if let Some(alignment_cell) = alignment_stored.alignment_cell() {
          alignment_cell.borrow_mut().remove_row();
        }
      }
    },
    properties => { Ok(stored_map!("alignmentSkippable" => true)) }
  );

  // Tab tracking
  state::assign_value(
    "tabbing_start_tabs",
    Stored::Tokens(Tokens!()),
    Some(Scope::Global),
  );

  DefMacro!("\\@tabbing@start@tabs", sub [_args] {
    if let Some(Stored::Tokens(toks)) = state::lookup_value("tabbing_start_tabs") {
      toks
    } else {
      Tokens!()
    }
  });

  // \+ increments tab start by adding \> to tabbing_start_tabs
  DefPrimitive!("\\@tabbing@increment", sub [_args] {
    let mut tabs = if let Some(Stored::Tokens(toks)) = state::lookup_value("tabbing_start_tabs") {
      toks.unlist().to_vec()
    } else {
      Vec::new()
    };
    tabs.push(T_CS!("\\>"));
    state::assign_value(
      "tabbing_start_tabs",
      Stored::Tokens(Tokens::new(tabs)),
      Some(Scope::Global),
    );
  });

  // \- decrements tab start by removing first element from tabbing_start_tabs
  DefPrimitive!("\\@tabbing@decrement", sub [_args] {
    let tabs = if let Some(Stored::Tokens(toks)) = state::lookup_value("tabbing_start_tabs") {
      let mut v = toks.unlist().to_vec();
      if !v.is_empty() {
        v.remove(0);
      }
      v
    } else {
      Vec::new()
    };
    state::assign_value(
      "tabbing_start_tabs",
      Stored::Tokens(Tokens::new(tabs)),
      Some(Scope::Global),
    );
  });

  // Stubs for unimplemented features (matching Perl)
  DefPrimitive!("\\@tabbing@untab", sub [_args] { /* NOT HANDLED — see Perl note */ });
  DefPrimitive!("\\@tabbing@flushright", sub [_args] { /* NOT HANDLED */ });
  DefPrimitive!("\\@tabbing@hfil", sub [_args] { /* NOT HANDLED */ });
  DefPrimitive!("\\@tabbing@pushtabs", sub [_args] { /* NOT HANDLED */ });
  DefPrimitive!("\\@tabbing@poptabs", sub [_args] { /* NOT HANDLED */ });

  // Accent redirect: \a{x} → \@tabbing@x (looks up the accent by name)
  DefMacro!("\\@tabbing@accent{}", sub [args] {
    let accent = args[0].to_string();
    Tokens::new(vec![T_CS!(&format!("\\@tabbing@{accent}"))])
  });

  // Default definitions for \pushtabs/\poptabs/\kill (outside tabbing)
  DefMacro!("\\pushtabs", "");
  DefMacro!("\\poptabs", "");
  DefMacro!("\\kill", "");

  // The binding primitive that sets up the alignment
  DefPrimitive!("\\@tabbing@bindings", sub [_args] {
    tabbing_bindings()?;
  });

  // Internals of tabbing for program.sty compatibility
  DefMacro!("\\@startfield", "\\global\\setbox\\@curfield\\hbox\\bgroup\\color@begingroup");
  DefMacro!("\\@stopfield", "\\color@endgroup\\egroup");
  DefMacro!("\\@contfield", "\\global\\setbox\\@curfield\\hbox\\bgroup\\color@begingroup\\unhbox\\@curfield");
  DefMacro!("\\@addfield", "\\global\\setbox\\@curline\\hbox{\\unhbox\\@curline\\unhbox\\@curfield}");
});

/// Perl: tabbingBindings() — sets up alignment with repeated template and rebinds control chars
fn tabbing_bindings() -> Result<()> {
  // Template: repeated column with before=\lx@text@intercol, after=\hfil\lx@text@intercol
  let col = Cell {
    before: Some(Tokens::new(vec![T_CS!("\\lx@text@intercol")])),
    after: Some(Tokens::new(vec![T_CS!("\\hfil"), T_CS!("\\lx@text@intercol")])),
    empty: true,
    ..Cell::default()
  };
  let template = Template::new(TemplateConfig {
    repeated: vec![col],
    ..TemplateConfig::default()
  });

  let mut xml_attrs = HashMap::default();
  xml_attrs.insert(String::from("class"), String::from("ltx_tabbing"));

  let alignment = Alignment::new(AlignmentConfig {
    template: Some(template),
    open_container: Rc::new(|document, props| {
      document
        .open_element("ltx:tabular", Some(props), None)
        .map(Option::Some)
    }),
    close_container: Rc::new(|document| document.close_element("ltx:tabular")),
    open_row: Rc::new(|document, props| {
      let str_props: HashMap<String, String> = props.into_iter()
        .map(|(k, v)| (k, v.to_string()))
        .collect();
      document
        .open_element("ltx:tr", Some(str_props), None)
        .and(Ok(()))
    }),
    close_row: Rc::new(|document| document.close_element("ltx:tr")),
    open_column: Rc::new(|document, props| {
      document
        .open_element("ltx:td", Some(props), None)
        .map(Option::Some)
    }),
    close_column: Rc::new(|document| document.close_element("ltx:td")),
    is_math: false,
    properties: SymHashMap::default(),
    xml_attributes: xml_attrs,
  });
  assign_alignment(alignment, None);

  // Rebind control characters within tabbing
  // Perl: Let("\\=", '\@tabbing@tabset') etc.
  state::let_i(&T_CS!("\\="), &T_CS!("\\@tabbing@tabset"), None);
  state::let_i(&T_CS!("\\>"), &T_CS!("\\@tabbing@nexttab"), None);
  state::let_i(&T_CS!("\\\\"), &T_CS!("\\@tabbing@newline"), None);
  state::let_i(&T_CS!("\\kill"), &T_CS!("\\@tabbing@kill"), None);
  state::let_i(&T_CS!("\\+"), &T_CS!("\\@tabbing@increment"), None);
  state::let_i(&T_CS!("\\-"), &T_CS!("\\@tabbing@decrement"), None);
  state::let_i(&T_CS!("\\<"), &T_CS!("\\@tabbing@untab"), None);
  // Save accent definitions before rebinding \' and \`
  state::let_i(&T_CS!("\\@tabbing@'"), &T_CS!("\\'"), None);
  state::let_i(&T_CS!("\\@tabbing@`"), &T_CS!("\\`"), None);
  state::let_i(&T_CS!("\\a"), &T_CS!("\\@tabbing@accent"), None);
  // Rebind \' and \` to tabbing-specific (flush right / hfil)
  state::let_i(&T_CS!("\\'"), &T_CS!("\\@tabbing@flushright"), None);
  state::let_i(&T_CS!("\\`"), &T_CS!("\\@tabbing@hfil"), None);
  state::let_i(&T_CS!("\\pushtabs"), &T_CS!("\\@tabbing@pushtabs"), None);
  state::let_i(&T_CS!("\\poptabs"), &T_CS!("\\@tabbing@poptabs"), None);

  Ok(())
}
