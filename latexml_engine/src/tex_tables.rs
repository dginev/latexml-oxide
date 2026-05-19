//! TeX Tables
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
use latexml_core::alignment::read_alignment_template;
use latexml_core::alignment::template::TemplateConfig;
use std::cell::{RefCell, RefMut};
use std::collections::VecDeque;


/// DEP-18 helper for empty-body `DefMacro!("\\cs[opt-spec]", "")` stubs.
fn def_macro_noop(proto: &str) -> Result<()> {
  let (cs_tok, params) = parse_prototype(proto, true)?;
  let body = mouth::tokenize_internal("");
  def_macro(cs_tok, params, ExpansionBody::Tokens(body), None)?;
  Ok(())
}

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Tables Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  //----------------------------------------------------------------------
  DefParameterType!(AlignmentTemplate, sub[_inner, _extra] {
    read_alignment_template()
  });

  // This is where ALL alignments start & finish
  // This creates the object representing the entire alignment!
  // NOTE: Perl does NOT have SkipSpaces here. Adding it causes handle_template
  // to fire for the outer alignment during parameter parsing, corrupting the token stream
  // when \begin{aligned} is nested inside \begin{align}.
  DefConstructor!("\\lx@begin@alignment", "#alignment",
    reversion => sub[whatsit,_args] {
      if let Some(Stored::Digested(alignment)) = whatsit.get_property("alignment").as_deref() {
        if let DigestedData::Alignment(data) = alignment.data() {
          data.borrow().revert()
        } else {
          Ok(Tokens!())
        }
      } else {
        Ok(Tokens!())
      }},
    sizer => "#alignment",
    after_digest => sub[whatsit] {
      bgroup();
      if let Some(alignment) = lookup_alignment() {
        whatsit.set_property("alignment", Stored::Digested(alignment));
        digest_alignment_body(whatsit)?;
      }
      egroup()?;
    }
  );

  // Seems odd to need both end markers here...
  DefMacro!("\\lx@end@alignment", r"\lx@hidden@crcr\lx@close@alignment");
  DefPrimitive!("\\lx@close@alignment", None);

  // & gives an error except within the right context
  // (which should redefine it!)
  DefConstructor!("&", {
    Error!("unexpected", "&", "Stray alignment \"&\"");
  });

  Tag!("ltx:td", after_close => sub[doc, node] { doc.trim_node_whitespace(node)?; });

  //----------------------------------------------------------------------
  // Primitive column types;
  // This is really LaTeX, but the mechanisms are used behind-the-scenes here, too.
  DefColumnType!("|", {
    with_current_build_template(|template_opt| {
      template_opt
        .unwrap()
        .add_between_column(vec![T_CS!("\\vrule"), T_CS!("\\relax")])
    });
  });
  // Perl: l/c/r column types do NOT set `align` explicitly.
  // Alignment is derived from \hfil fills during extractAlignmentColumn:
  //   \hfil after  → left,  \hfil before+after → center,  \hfil before → right
  DefColumnType!("l", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        after: Some(Tokens!(T_CS!("\\hfil"))),
        ..Cell::default()
      })
    });
  });
  DefColumnType!("c", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens!(T_CS!("\\hfil"))),
        after: Some(Tokens!(T_CS!("\\hfil"))),
        ..Cell::default()
      })
    });
  });
  DefColumnType!("r", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(Cell {
        before: Some(Tokens!(T_CS!("\\hfil"))),
        ..Cell::default()
      })
    });
  });

  // Perl: p{Dimension} — before wraps content in \vtop{\hbox to <width>\relax{...}}.
  // Width flows through \hbox BoxSpecification (not as cell attribute).
  // vattach flows through \vtop → insertBlock.
  // Perl: before => Tokens(\vtop, {, \hbox, T_LETTER('t'), T_LETTER('o'), $_[1]->revert, \relax, {)
  DefColumnType!("p{Dimension}", sub[(width)] {
    // Build before tokens: \vtop { \hbox to <dim> \relax {
    let mut before = vec![
      T_CS!("\\vtop"), T_BEGIN!(), T_CS!("\\hbox"),
    ];
    before.extend(ExplodeText!("to"));
    before.extend(width.revert()?.unlist());
    before.push(T_CS!("\\relax"));
    before.push(T_BEGIN!());
    with_current_build_template(|template_opt| template_opt.unwrap().add_column(Cell {
      before: Some(Tokens::new(before)),
      after: Some(Tokens!(T_END!(), T_END!())),
      align: Some(Align::Justify),
      vattach: Some("top".to_string()),
      ..Cell::default()}));
  });

  DefColumnType!("*{Number}{}", sub[(n,pattern)] {
    let mut tks = Vec::new();
    for _ in 1 ..= n.value_of() {
      tks.extend_from_slice(pattern.unlist_ref());
    }
    tks
  });

  // Perl TeX_Tables L81-86: @{} disables intercolumn before and after
  DefColumnType!("@{}", sub[(filler)] {
    with_current_build_template(|template_opt| {
      let t = template_opt.unwrap();
      t.disable_intercolumn();
      t.add_between_column(filler.unlist());
      t.disable_intercolumn();
    }); });

  //======================================================================
  // Table Line endings
  //----------------------------------------------------------------------
  // \cr               c  is a visible command which ends one row in a table.
  // \crcr             c  is an alternate to \cr.
  // \everycr          pt holds tokens inserted after every \cr or nonredundent \crcr.
  //\tabskip          pg is optional glue put between columns in a table.
  DefConstructor!("\\cr", "\n");
  DefConstructor!("\\crcr", "\n");
  // These are useful for reversion of higher-level macros that use alignment
  // internally, but don't use explicit &,\cr in the user markup
  DefConstructor!("\\lx@hidden@cr",    "\n", alias => "");
  DefConstructor!("\\lx@hidden@crcr",  "\n", alias => "");
  DefConstructor!("\\lx@hidden@align", "",   alias => "");

  DefRegister!("\\everycr", Tokens!());
  DefRegister!("\\tabskip", Glue!("0"));

  //======================================================================
  // Aligment exceptions
  //----------------------------------------------------------------------
  // \noalign          c  inserts vertical mode material after a \cr in a table.
  // \omit             c  is used in the body of a table to change an entry's template from the one
  // in the preamble. \span             c  combines adjacent entries in a table into a single
  // entry.
  DefPrimitive!("\\noalign", {
    // Perl `DefPrimitiveI('\noalign', ...)`: bgroup() + error + lets.
    // We MATCH Perl's bgroup but then read+discard the `{...}` body and
    // egroup so the body's closing `}` doesn't leak past the bgroup
    // frame. Without this body-consume, the user's `\hline`
    // (= `\noalign{\@@alignment@hline}`) hitting the bad path (cell
    // content rather than INNER 1's noalign branch) leaves the bgroup
    // pushed AND its `{` pushes a SECOND frame, while only the body's
    // `}` pops one — the `\noalign`'s primitive-bgroup leaks. The leak
    // then cascades into \vtop/\hbox mode-end mismatches in p{}-column
    // arrays (REG-1 / math0403005).
    bgroup();
    Error!("unexpected", "\\noalign", "\\noalign cannot be used here");
    // Consume the `{...}` body so its `}` matches our bgroup frame.
    if let Some(tok) = gullet::read_token()? {
      if tok.get_catcode() == Catcode::BEGIN {
        let _ = gullet::read_balanced(ExpansionLevel::Off, false, false)?;
      } else {
        gullet::unread_one(tok);
      }
    }
    egroup()?;
    Let!(&T_ALIGN!(), T_RELAX!());
    Let!(&T_CS!("\\noalign"), T_RELAX!());
    Let!(&T_CS!("\\omit"), T_RELAX!());
    Let!(&T_CS!("\\span"), T_RELAX!());
  });
  DefPrimitive!("\\omit", {
    Error!("unexpected", "\\omit", "\\omit cannot be used here");
    bgroup();
    Let!(&T_ALIGN!(), T_RELAX!());
    Let!(&T_CS!("\\noalign"), T_RELAX!());
    Let!(&T_CS!("\\omit"), T_RELAX!());
    Let!(&T_CS!("\\span"), T_RELAX!());
  });
  DefPrimitive!("\\span", {
    bgroup();
    Error!("unexpected", "\\span", "\\span cannot be used here");
    Let!(&T_ALIGN!(), T_RELAX!());
    Let!(&T_CS!("\\noalign"), T_RELAX!());
    Let!(&T_CS!("\\omit"), T_RELAX!());
    Let!(&T_CS!("\\span"), T_RELAX!());
  });

  //======================================================================
  // Horizontal alignments
  //----------------------------------------------------------------------
  // \halign           c  begins the horizontal alignment of material (i.e., makes a table
  // containing rows).

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Now, for \halign itself
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // See \lxX@noalign for some \noalign cases
  // See \lx@alignment@multicolumn for cases of \span,\omit
  // See alignmentBindings for default bindings
  // But also see others for different handling of (eg) open@row, etc.
  // Probably we have to handle these cases by more generic default code
  // and appropriate tweaks of alignment data????

  // Algorithm:
  //   open@alignment
  //     Loop while read_column
  //======================================================================

  // Perl TeX_Tables L159-184: \halign BoxSpecification
  DefConstructor!("\\halign BoxSpecification", "#alignment",
  reversion => sub[whatsit, _args] {
    let mut tks = vec![T_CS!("\\halign")];
    if let Some(spec) = whatsit.get_arg(1) {
      tks.extend(spec.revert()?.unlist());
    }
    tks.push(T_BEGIN!());
    if let Some(Stored::Tokens(template_tks)) = whatsit.get_property("template_tokens").as_deref() {
      tks.extend_from_slice(template_tks.unlist_ref());
    }
    tks.push(T_CS!("\\cr"));
    if let Some(Stored::Digested(alignment_d)) = whatsit.get_property("alignment").as_deref() {
      if let DigestedData::Alignment(data) = alignment_d.data() {
        tks.extend(data.borrow().revert()?.unlist());
      }
    }
    tks.push(T_END!());
    Ok(Tokens::new(tks))
  },
  bounded => true,
  leave_horizontal => true,
  sizer => sub[whatsit] {
    if let Some(Stored::Digested(alignment_d)) = whatsit.get_property("alignment").as_deref() {
      let (w, h, d, _, _, _) = alignment_d.clone().get_size(None)?;
      Ok((w, h, d))
    } else {
      Ok((Dimension::default(), Dimension::default(), Dimension::default()))
    }
  },
  before_construct => sub[document, _whatsit] {
    document.maybe_close_element("ltx:p")?;
  },
  after_digest => sub[whatsit] {
    whatsit.set_property("mode", Stored::from("internal_vertical"));
    begin_mode("restricted_horizontal")?;
    let template = parse_halign_template(whatsit)?;
    // Get width from BoxSpecification 'to' key
    let width_attr: Option<String> = {
      let spec = whatsit.get_arg(1);
      if let Some(ArgWrap::Dimension(w)) = GetKeyVal!(spec, "to") {
        Some(w.to_attribute())
      } else {
        None
      }
    };
    let mut xml_attrs = HashMap::default();
    if let Some(w) = width_attr {
      xml_attrs.insert(String::from("width"), w);
    }
    alignment_bindings(template, String::new(), SymHashMap::default(), xml_attrs);
    // Mark as \halign — first column CAN get ltx_nopad_l (unlike LaTeX
    // tabular). with_value avoids the Stored::clone on the Digested
    // variant; the inner Rc<Digested> + RefCell<Alignment> mutation
    // still works fine through the borrow.
    state::with_value("Alignment", |v| {
      if let Some(Stored::Digested(ref d)) = v {
        if let latexml_core::digested::DigestedData::Alignment(ref alignment) = d.data() {
          alignment.borrow_mut().is_halign = true;
        }
      }
    });
    digest_alignment_body(whatsit)?;
    end_mode("restricted_horizontal")?;
    decrement_align_group_count(); // Balance the opening { OUTSIDE of the masking of ALIGN_STATE
  });

  def_macro_noop("\\lx@alignment@row@before")?;
  def_macro_noop("\\lx@alignment@row@after")?;
  def_macro_noop("\\lx@alignment@column@before")?;
  def_macro_noop("\\lx@alignment@column@after")?;

  //======================================================================
  // Vertical alignments
  //----------------------------------------------------------------------
  // \valign           c  begins the vertical alignment of material (i.e., makes a table containing
  // columns).

  // Implement ???
  // DefMacro('\vrule','\relax');
  def_macro_noop("\\valign")?;

  // VERY tricky (and mostly Wrong).
  // The issue is for \\ to look ahead for * and [],
  // Eventually we'll expand into \cr (which should be preceded by the RHS of the template)
  // BUT it should NOT trigger the template if it bumps into a &
  // which happens when the 1st column of an alignment is empty.
  // In proper LaTeX this is inhibited by a curious construct
  //   {\ifnum0='}
  // and possibly by proper tracking of a Master Counter !?!?!?
  // But we're not there (yet)

  // This is the internal macro for \\[dim] used by LaTeX for various arrays, tabular, etc
  DefMacro!("\\lx@alignment@newline", {
    let (_star, optional) = read_newline_args(true)?;
    let mut tokens = vec![T_CS!("\\lx@hidden@cr"), T_BEGIN!()];
    if let Some(opt_tks) = optional {
      tokens.push(T_CS!("\\lx@alignment@newline@markertall"));
      tokens.push(T_BEGIN!());
      tokens.extend(opt_tks.unlist());
      tokens.push(T_END!());
    } else {
      tokens.push(T_CS!("\\lx@alignment@newline@marker"));
    }
    tokens.push(T_END!());
    Tokens::new(tokens)
  });
  // However, the above will skip spaces --AND a newline! -- looking for [],
  // which is kinda weird in math, since there may be a reasonable math [ in the 1st column!
  // AMS kindly avoids that, by using a special version of \\
  DefMacro!("\\lx@alignment@newline@noskip", {
    let (_star, optional) = read_newline_args(false)?;
    let mut tokens = vec![T_CS!("\\lx@hidden@cr"), T_BEGIN!()];
    if let Some(opt_tks) = optional {
      tokens.push(T_CS!("\\lx@alignment@newline@markertall"));
      tokens.push(T_BEGIN!());
      tokens.extend(opt_tks.unlist());
      tokens.push(T_END!());
    } else {
      tokens.push(T_CS!("\\lx@alignment@newline@marker"));
    }
    tokens.push(T_END!());
    Tokens::new(tokens)
  });
  // These are the markers that produce \\ in the reversion,
  // and (eventually will) add vertical space to the row!
  DefConstructor!("\\lx@alignment@newline@marker", "",
    reversion => Tokens!(T_CS!("\\\\"), T_CR!()));
  // AND add the spacing to the alignment!!!
  DefConstructor!("\\lx@alignment@newline@markertall {Dimension}", "",
  after_digest => sub[whatsit] {
  if let Some(alignment) = lookup_alignment() {
    let mut alignment_mut = alignment.alignment_cell().unwrap().borrow_mut();
    let current_row = alignment_mut.current_row_mut().unwrap();
    let padding = if let Some(arg) = whatsit.get_arg(1) {
      if let DigestedData::RegisterValue(RegisterValue::Dimension(v)) = arg.data() {
        *v
      } else { Dimension::new(0) }
    }  else { Dimension::new(0) };
    current_row.set_padding(padding);
  }},
  reversion => sub[whatsit,_args] {
    let arg_reverted = whatsit.get_arg(1)
      .map(|a| a.revert())
      .unwrap_or_else(|| Ok(Tokens!()))?;
    Ok(Tokens!(T_CS!("\\\\"), T_OTHER!("["), arg_reverted, T_OTHER!("]"), T_CR!()))
  });

  // Perl: \lx@intercol is our replacement for LaTeX's \@acol for intercolumn space
  DefMacro!("\\lx@intercol", "");
  // Perl: Candidates for binding \lx@intercol for LaTeX tabular or math arrays
  DefConstructor!("\\lx@text@intercol", sub[document, _args, props] {
    if let Some(width) = props.get("width") {
      let dim: Option<Dimension> = width.into();
      if let Some(d) = dim {
        let s = crate::tex_glue::dimension_to_spaces(d);
        if !s.is_empty() {
          document.absorb_string(&s, &SymHashMap::default())?;
        }
      }
    }
  },
  reversion => Tokens!(T_CS!("\\lx@intercol")),
  properties => {
    let w = match state::lookup_register("\\tabcolsep", Vec::new())? {
      Some(RegisterValue::Dimension(d)) => d,
      Some(RegisterValue::Glue(g)) => Dimension::new(g.value_of()),
      _ => Dimension::default(),
    };
    stored_map!("width" => w, "isSpace" => true)
  });
  DefConstructor!("\\lx@math@intercol", "",
  reversion => Tokens!(T_CS!("\\lx@intercol")),
  properties => {
    let w = match state::lookup_register("\\arraycolsep", Vec::new())? {
      Some(RegisterValue::Dimension(d)) => d,
      Some(RegisterValue::Glue(g)) => Dimension::new(g.value_of()),
      _ => Dimension::default(),
    };
    stored_map!("width" => w, "isSpace" => true)
  });

  //======================================================================
  // Various decorations within alignments, rules, headers, etc

  // Like \noalign, takes an arg; handled within alignment processing.
  // But doesn't create a pseudo-row (??? Or does it?; is it still needed?)
  DefConstructor!("\\lx@hidden@noalign{}", "#1",
    reversion  => "",
    properties =>  sub[args] {
      // Sometimes, we're smuggling stuff that needs to be carried into the XML.
      let mut props = stored_map!("alignmentSkippable" => true);
      // Check if any arg (or child of a List arg) has alignmentPreserve.
      // This propagates the property from e.g. \label (inside a List) to the
      // \lx@hidden@noalign whatsit, so the alignment absorption code knows
      // to absorb this whatsit even in skippable cells.
      'outer: for v in args.iter().flatten() {
        if v.get_property("alignmentPreserve").is_some() {
          props.insert("alignmentPreserve", Stored::Bool(true));
          break;
        }
        // Also check children of List args
        for child in v.unlist_ref() {
          if child.get_property_bool("alignmentPreserve") {
            props.insert("alignmentPreserve", Stored::Bool(true));
            break 'outer;
          }
        }
      }
      Ok(props) });

  // NOTE: this engine override gets clobbered by the dump-load (latex.dump.txt's
  // M-line for `\hline` is the latex.ltx macro form `\noalign{\ifnum0=`}\fi
  // \hrule\@height\arrayrulewidth\futurelet\reserved@a\@xhline`). The override
  // is re-applied at the end of `latex_constructs.rs::load_definitions` so the
  // engine version wins post-dump. Recovers ~50 tabular-using tests.
  DefMacro!("\\hline", "\\noalign{\\@@alignment@hline}");
  DefConstructor!("\\@@alignment@hline", "",
    after_digest => sub[_whatsit] {
      if let Some(alignment_stored) = lookup_alignment() {
        alignment_stored.alignment_cell().unwrap().borrow_mut()
          .add_line("t", Vec::new());
      }
    },
    properties =>  { Ok(stored_map!("isHorizontalRule" => true))},
    sizer      => 0, alias => "\\hline");

  // Perl: DefMacroI('\lx@alignment@begin@heading', undef, sub { ... in_tabular_head = 1 })
  DefMacro!("\\lx@alignment@begin@heading", {
    if let Some(alignment_stored) = lookup_alignment() {
      alignment_stored
        .alignment_cell()
        .unwrap()
        .borrow_mut()
        .set_in_tabular_head();
    }
  });
  // Perl: DefMacroI('\lx@alignment@end@heading', undef, sub { ... in_tabular_head = 0 })
  DefMacro!("\\lx@alignment@end@heading", {
    if let Some(alignment_stored) = lookup_alignment() {
      alignment_stored
        .alignment_cell()
        .unwrap()
        .borrow_mut()
        .unset_in_tabular_head();
    }
  });
  // Deprecated aliases (Base_Deprecated.pool.ltxml)
  Let!("\\@tabular@begin@heading", "\\lx@alignment@begin@heading");
  Let!("\\@tabular@end@heading", "\\lx@alignment@end@heading");

  //======================================================================
  // Multicolumn support
  // Perl: DefRegisterI('\lx@alignment@ncolumns', undef, Dimension(0), getter => sub { ... })
  DefRegister!("\\lx@alignment@ncolumns", Number::new(0),
    getter => {
      if let Some(alignment_stored) = lookup_alignment() {
        let data = alignment_stored.alignment_cell().unwrap();
        let borrowed = data.borrow();
        Number::new(borrowed.get_template().get_columns().len() as i64)
      } else {
        Number::new(0)
      }
    }
  );
  // Perl: DefRegisterI('\lx@alignment@column', undef, Dimension(0), getter => sub { ... })
  DefRegister!("\\lx@alignment@column", Number::new(0),
    getter => {
      if let Some(alignment_stored) = lookup_alignment() {
        let data = alignment_stored.alignment_cell().unwrap();
        let borrowed = data.borrow();
        Number::new(borrowed.current_column_number() as i64)
      } else {
        Number::new(0)
      }
    }
  );

  // Perl: DefMacro('\lx@alignment@multicolumn {Number} AlignmentTemplate {}', sub { ... })
  // Expands to \omit + (span-1) × (\span \omit) + before_cell_tokens + body + after_cell_tokens
  DefMacro!("\\lx@alignment@multicolumn {Number} AlignmentTemplate {}", sub[(span, template, body)] {
    let n = span.value_of();
    let column = template.get_columns().first();
    // First part, like \multispan
    let mut tks = vec![T_CS!("\\omit")];
    for _ in 1..n {
      tks.push(T_CS!("\\span"));
      tks.push(T_CS!("\\omit"));
    }
    // Next part: put the template in-line, since it's only used once.
    // beforeCellUnlist: reorder $ and \hfil (move \hfil before $)
    if let Some(col) = column {
      if let Some(before) = &col.before {
        tks.extend(before_cell_unlist(before.unlist_ref().to_vec()));
      }
    }
    tks.extend(body.unlist_ref().iter().copied());
    // afterCellUnlist: reorder $ and \hfil (move \hfil after $)
    if let Some(col) = column {
      if let Some(after) = &col.after {
        tks.extend(after_cell_unlist(after.unlist_ref().to_vec()));
      }
    }
    Ok(Tokens::new(tks))
  });

  DefConditional!("\\if@in@lx@alignment", { lookup_alignment().is_some() });

  // TODO:
  // DefPrimitive('\@alignment@bindings AlignmentTemplate []', sub {
  //     my ($stomach, $template, $mode) = @_;
  //     alignmentBindings($template, $mode); });

  // This removes trailing whitespace from the current digested list.
  // It is useful as the 1st thing in the rhs template of things like {tabular}.
  // But note that \halign does NOT remove this trailing space!
  DefPrimitive!("\\lx@column@trimright", {
    let mut save = Vec::new();
    while let Some(tbox) = pop_box_list() {
      if tbox.get_property_bool("alignmentSkippable") || tbox.get_property_bool("isFill") {
        save.push(tbox);
      } else if !tbox.is_empty()? {
        push_box_list(tbox);
        break;
      }
    }
    if !save.is_empty() {
      extend_box_list(save);
    }
    Ok(Vec::new())
  });
});

//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// And the general alignment processing.
// If the Template is appropriately constructed, either by \halign or various \begin{tabular}
// the body of the alignment is processed the same way.

pub fn alignment_bindings(
  template: Template,
  mode: String,
  mut properties: SymHashMap<Stored>,
  xml_attributes: HashMap<String, String>,
) {
  let mode = if mode.is_empty() {
    state::lookup_string_from_sym(pin!("MODE"))
  } else {
    mode
  };
  let is_math = mode.ends_with("math");
  let (container, rowtype, coltype) = if is_math {
    ("ltx:XMArray", "ltx:XMRow", "ltx:XMCell")
  } else {
    ("ltx:tabular", "ltx:tr", "ltx:td")
  };
  // Perl alignmentBindings L254: $properties{strut} = LookupRegister('\baselineskip');
  if !properties.contains_key("strut") {
    if let Ok(Some(bs)) = state::lookup_register("\\baselineskip", Vec::new()) {
      properties.insert("strut", bs.into());
    }
  }
  let alignment = Alignment::new(AlignmentConfig {
    template: Some(template),
    open_container: Rc::new(|document, props| {
      document
        .open_element(container, Some(props), None)
        .map(Option::Some)
    }),
    close_container: Rc::new(|document| document.close_element(container)),
    open_row: Rc::new(|document, props| {
      let str_props: HashMap<String, String> =
        props.into_iter().map(|(k, v)| (k, v.to_string())).collect();
      document
        .open_element(rowtype, Some(str_props), None)
        .and(Ok(()))
    }),
    close_row: Rc::new(|document| document.close_element(rowtype)),
    open_column: Rc::new(|document, props| {
      document
        .open_element(coltype, Some(props), None)
        .map(Option::Some)
    }),
    close_column: Rc::new(|document| document.close_element(coltype)),
    is_math,
    properties,
    xml_attributes,
  });
  assign_alignment(alignment, None);
  // Debug("Halign $alignment: New " . $template->show) if $LaTeXML::DEBUG{halign};
  state::let_i(
    &T_MATH!(),
    &if is_math {
      T_CS!("\\lx@dollar@in@mathmode")
    } else {
      T_CS!("\\lx@dollar@in@textmode")
    },
    None,
  );
}

pub fn digest_alignment_body(whatsit: &mut Whatsit) -> Result<()> {
  // Now read & digest the body.
  // Note that the body MUST end with a \cr, and that we've made Special Arrangments
  // with \alignment@cr to recognize the end of the \halign
  local_align_group_count(0);
  let alignment_stored = if let Some(alignment) = lookup_alignment() {
    alignment
  } else {
    Error!(
      "missing",
      "alignment",
      "There is no open alignment structure here"
    );
    return Ok(());
  };
  local_reading_alignment(&alignment_stored);
  whatsit.set_property("alignment", Stored::Digested(alignment_stored.clone()));

  // Debug!("Halign {}: BODY Processing...",alignment) if $LaTeXML::DEBUG{halign};
  let mut lastwascr = false;
  let mut reversion: Vec<Token> = Vec::new();
  let mut creversion: Vec<Token> = Vec::new();
  let alignment_cell = alignment_stored.alignment_cell().unwrap();
  loop {
    let (cell_opt, next, vtype, hidden) = digest_alignment_column(alignment_cell, lastwascr)?;
    //     Debug("Halign $alignment: BODY got CELL"
    //         . "[" . $alignment->currentRowNumber . "," . $alignment->currentColumnNumber . "]"
    //         . ToString($cell) . " ended at " . Stringify($next)) if $LaTeXML::DEBUG{halign};

    if let Some(cell) = cell_opt {
      reversion.extend(
        trim_column_template(alignment_cell.borrow_mut(), p_revert(cell.clone())?).unlist(),
      );
      creversion.extend(
        trim_column_template(alignment_cell.borrow_mut(), c_revert(cell.clone())?).unlist(),
      );
      extract_alignment_column(alignment_cell.borrow_mut(), cell)?;
    } else {
      // Debug("Halign $alignment: BODY DONE!") if $LaTeXML::DEBUG{halign};
      break;
    }
    lastwascr = false;
    // Perl L319-320: $next->defined_as(T_END) — recognizes \egroup as alignment end
    let next_is_end = next
      .as_ref()
      .map(|t| t.defined_as(&T_END!()))
      .unwrap_or(false)
      || next == Some(T_CS!("\\lx@close@alignment"));
    if (vtype.is_none() || vtype.as_ref().unwrap().is_empty()) && (next.is_none() || next_is_end) {
      // End of alignment
      alignment_cell.borrow_mut().end_row()?;
      break;
    } else if vtype.as_deref() == Some("align") {
      alignment_cell.borrow_mut().end_column()?;
      if !hidden {
        reversion.push(next.unwrap()); // and record the &
        creversion.push(next.unwrap()); // and record the &
      }
    } else if vtype.as_deref() == Some("insert") {
      alignment_cell.borrow_mut().end_column()?;
    } else if vtype.as_deref() == Some("cr") || vtype.as_deref() == Some("crcr") {
      alignment_cell.borrow_mut().end_row()?;
      if !hidden {
        reversion.push(next.unwrap());
        creversion.push(next.unwrap());
      } else if vtype.as_deref() == Some("cr") {
        let arg_toks = gullet::read_arg(ExpansionLevel::Off)?;
        let arg = stomach::digest(arg_toks)?;
        reversion.extend(p_revert(arg.clone())?.unlist());
        creversion.extend(c_revert(arg)?.unlist());
      } else if vtype.as_deref() == Some("crcr") {
      }
      lastwascr = true;
    } else if let Some(next_tok) = next {
      // Note, in case next is \crcr
      Error!("unexpected", next_tok, s!("Column ended with {next_tok}"));
    }
  }
  alignment_cell.borrow_mut().end_row()?;
  alignment_cell
    .borrow_mut()
    .set_reversion(Tokens!(reversion));
  alignment_cell
    .borrow_mut()
    .set_content_reversion(Tokens!(creversion));
  //   Debug("Halign $alignment: BODY DONE!\n"
  //       . "=> " . join(',', map { Stringify($_); } @reversion)) if $LaTeXML::DEBUG{halign};
  expire_align_group_count();
  expire_reading_alignment();
  Ok(())
}

// Read & digest an alignment column's data,
// accommodating the current template and any special cs's
// Returns the column's digested boxes, the ending token, and it's alignment type.
type DigestedColumn = Result<(Option<Digested>, Option<Token>, Option<String>, bool)>;
pub fn digest_alignment_column(alignment: &RefCell<Alignment>, lastwascr: bool) -> DigestedColumn {
  new_local_box_list();
  let ismath = state::lookup_bool_sym(pin!("IN_MATH"));
  // Scan for leading \omit, skipping over (& saving) \hline.
  //   Debug("Halign $alignment: COLUMN starting scan "
  //       . "(" . ($ismath ? " math" : " text") . ")") if $LaTeXML::DEBUG{halign};
  // Declared without initializer — Perl resets this to undef at the
  // start of every OUTER iteration (see L742), so the initial value
  // is genuinely dead. The compiler tracks definite-assignment for us.
  let mut last_token: Option<Token>;
  let mut spanning = false;
  loop {
    // Outer loop; collects 1 column (possibly multiple spans) return from within!
    // Scan till we get something NOT \omit, \noalign.
    // Perl TeX_Tables.pool.ltxml L371-396: `$token` is set per readXToken call,
    // so when the gullet returns undef (mouth exhausted), `!$token` triggers
    // the early-return `return (undef, $token, undef, undef);`. In Rust the
    // INNER 1 `while let Some(xtoken) = read()` branch only updates last_token
    // on Some — when read returns None, last_token would stay at its prior
    // value (e.g. a content token from a previous OUTER iteration), the
    // `last_token.is_none()` check below would skip, and we'd re-feed the
    // (column_before, marker, last_token) bundle into an empty gullet,
    // looping infinitely. Reset to None per Perl's per-iteration semantics.
    last_token = None;
    while let Some(xtoken) = gullet::read_x_token(Some(false), false, None)? {
      last_token = Some(xtoken);
      let token = last_token.as_ref().unwrap();
      // Skip leading space. Skip \par or blank line(?). Or \crcr following a \cr
      if *token == T_SPACE!()
        || *token == T_CS!("\\par")
        || (lastwascr && (*token == T_CS!("\\crcr") || *token == T_CS!("\\lx@hidden@crcr")))
      {
      } else if *token == T_CS!("\\omit") {
        // \omit removes template for this column.
        //         Debug("Halign $alignment: OMIT at " . Stringify($token)) if
        // $LaTeXML::DEBUG{halign};
        if !alignment.borrow().is_in_row() {
          alignment.borrow_mut().start_row(false)?;
        }
        alignment.borrow_mut().omit_next_column();
      } else if *token == T_CS!("\\noalign") {
        // \puts something in vertical list
        // Debug("Halign $alignment: noalign at " . Stringify($token)) if $LaTeXML::DEBUG{halign};
        if alignment.borrow().is_in_row() {
          alignment.borrow_mut().end_row()?;
        }
        alignment.borrow_mut().start_column(true)?;
        alignment.borrow_mut().last_column();
        let next_arg = gullet::read_arg(ExpansionLevel::Off)?;
        let r = stomach::digest(next_arg)?;
        alignment.borrow_mut().end_row()?;
        expire_local_box_list();
        return Ok((Some(r), Some(T_CS!("\\cr")), some!("cr"), false)); // Pretend this is a whole
      // row???
      } else if *token == T_CS!("\\lx@hidden@noalign") {
        // \puts something in vertical list
        //         Debug("Halign $alignment: COLUMN invisible noalign") if $LaTeXML::DEBUG{halign};
        let invoked = stomach::invoke_token(token)?;
        extend_box_list(invoked);
      } else {
        break;
      }
    }
    //     Debug("Halign $alignment: COLUMN end scan at " . Stringify($token)) if
    // $LaTeXML::DEBUG{halign};
    // Perl L395: $token->defined_as(T_END) — recognizes \egroup as column end
    let last_is_end = last_token
      .as_ref()
      .map(|t| t.defined_as(&T_END!()))
      .unwrap_or(false)
      || last_token == Some(T_CS!("\\lx@close@alignment"));
    if last_token.is_none() || last_is_end {
      expire_local_box_list();
      return Ok((None, last_token, None, false));
    }
    // Next column, unless spanning (then combine columns)
    if spanning {
      spanning = false;
      alignment.borrow_mut().next_column()?;
    } else {
      alignment.borrow_mut().start_column(false)?;
    }
    // Push before template,  Marker and put the token back
    let to_unread = Tokens!(
      alignment.borrow_mut().get_column_before(),
      T_MARKER!("before-column"),
      last_token.unwrap()
    );
    // eprintln!("Halign: COLUMN preload at {}", to_unread.stringify());
    gullet::unread(to_unread);
    while let Some(token) = gullet::read_x_token(Some(false), false, None)? {
      if let Some((_atoken, vtype, hidden)) = gullet::is_column_end(&token) {
        if vtype == "span" {
          // next column, but continue accumulating
          // Debug("Halign $alignment: COLUMN span") if $LaTeXML::DEBUG{halign};
          spanning = true;
          break;
        } else {
          // Debug("Halign $alignment: COLUMN ended with " . Stringify($token) . "\n"
          //     . "  => " . ToString(List(@LaTeXML::LIST))) if $LaTeXML::DEBUG{halign};
          let current_list = expire_local_box_list();
          let mut out_list = List::new(current_list);
          out_list.mode = Some(if ismath { TexMode::Math } else { TexMode::Text });
          return Ok((
            Some(out_list.into()),
            Some(token),
            Some(String::from(vtype)),
            hidden,
          ));
        }
      // DG: Note this block is commented out as clippy warned it has the exact identical logic
      // as the "all other cases" else that follows it
      //
      // } else if token == T_CS!("\\lx@hidden@noalign") { //  \puts something in vertical list
      //   // Debug("Halign $alignment: COLUMN invisible noalign") if $LaTeXML::DEBUG{halign};
      //   let invoked = stomach.invoke_token(&token)?;
      //   stomach.box_list.extend(invoked.into_iter());
      } else {
        // Else, we're getting some actual content for the column
        let invoked = stomach::invoke_token(&token)?;
        extend_box_list(invoked);
        // eprintln!("Halign: COLUMN {} ==> {}",token.stringify(),
        // List::new(stomach.box_list.clone()).stringify()); //       if
        // $LaTeXML::DEBUG{halign};
      }
    }
  }
}

// This attempts to trim off the column template parts from contents of the full column,
// leaving only the author supplied part for a sensible reversion.
// It's not nearly clever enough, given that macros can be in the template,
// but works surprisingly well so far.
// A better alternative might be based on sneaking some Marker tokens/boxes through
// but they would likely interfere with the macros tehmselves.
pub fn trim_column_template(mut alignment: RefMut<Alignment>, tokens: Tokens) -> Tokens {
  if let Some(row) = alignment.current_row() {
    if row.is_pseudo() {
      return tokens;
    }
  }
  let mut pre = alignment.get_column_before().unlist();
  let mut post = alignment.get_column_after().unlist();
  //   Debug("Halign $alignment: COLUMN Compare:\n"
  //       . "  Column: " . ToString(Tokens(@tokens)) . "\n"
  //       . "  Before: " . ToString(Tokens(@pre)) . "\n"
  //       . "  After : " . ToString(Tokens(@post)) . "\n") if $LaTeXML::DEBUG{halign};
  let mut tks_vec = tokens.unlist();
  while !pre.is_empty() && !tks_vec.is_empty() {
    let t = pre.remove(0);
    if let Some(tks_first) = tks_vec.first() {
      if t == *tks_first {
        tks_vec.remove(0);
      }
    }
  }
  while !post.is_empty() && !tks_vec.is_empty() {
    let t = post.pop().unwrap();
    if let Some(tks_last) = tks_vec.last() {
      if t == *tks_last {
        tks_vec.pop();
      }
    }
  }
  //   Debug("  Trimmed: " . ToString(Tokens(@tokens))) if $LaTeXML::DEBUG{halign};
  Tokens::new(tks_vec)
}
// Given the boxes for an alignment cell,
// extract & remove the various fills and rules from the ends to annotate the cell structure
pub fn extract_alignment_column(
  mut alignment: RefMut<Alignment>,
  in_box: Digested,
) -> Result<Digested> {
  let mut boxes = VecDeque::new();
  boxes.extend(in_box.unlist());
  let is_math = state::lookup_bool_sym(pin!("IN_MATH"));
  //Note: $n0,$n1 is a VERY round-about way of tracking the column spanning!
  let n0 = lookup_int("alignmentStartColumn") as usize + 1;
  let n1 = alignment.current_column_number();

  // --- Read phase: extract values from colspec, then drop the borrow ---
  let (initial_align, tabskip_clone, is_omitted, has_before_fill, has_after_fill, old_border);
  {
    let colspec = match alignment.get_column(n0) {
      Some(c) => c,
      None => {
        return Ok(Digested::default());
      },
    };
    initial_align = colspec.align.clone().unwrap_or(Align::Left);
    tabskip_clone = colspec.tabskip;
    is_omitted = colspec.omitted;
    old_border = colspec.border.clone();
    has_before_fill = colspec
      .before
      .as_ref()
      .map(|b| {
        b.unlist_ref()
          .iter()
          .any(|t| *t == T_CS!("\\hfil") || *t == T_CS!("\\hfill"))
      })
      .unwrap_or(false);
    has_after_fill = colspec
      .after
      .as_ref()
      .map(|a| {
        a.unlist_ref()
          .iter()
          .any(|t| *t == T_CS!("\\hfil") || *t == T_CS!("\\hfill"))
      })
      .unwrap_or(false);
  } // colspec borrow dropped

  let mut align = initial_align;
  let mut border = String::new();
  let mut saveleft = VecDeque::new();
  let mut saveright = VecDeque::new();
  let mut lspaces: Vec<Digested> = Vec::new();
  let mut rspaces: Vec<Digested> = Vec::new();
  // Perl L487-489: lspaces to transfer to previous column when vrule found
  let mut prev_rspaces_transfer: Option<Vec<Digested>> = None;

  // Perl L476-477: add tabskip as spacing text to lspaces
  // Create a Tbox with Unicode space characters directly, rather than
  // going through \hskip digestion (which can produce nbsp via constructor).
  if let Some(skip) = &tabskip_clone {
    if skip.value_of() != 0 {
      let dim = Dimension::new(skip.value_of());
      let spaces = crate::tex_glue::dimension_to_spaces(dim);
      if !spaces.is_empty() {
        let tbox = Tbox {
          text: arena::pin(&spaces),
          font: lookup_font().unwrap_or_default(),
          ..Tbox::default()
        };
        lspaces.push(Digested::from(tbox));
      }
    }
  }
  // Determine expected alignment from template fills, as a fallback for when
  // the trailing fill box is lost during digestion (known issue with nested \hbox groups).
  let expected_from_template = match (has_before_fill, has_after_fill) {
    (true, true) => Some(Align::Center),
    (false, true) => Some(Align::Left),
    (true, false) => Some(Align::Right),
    (false, false) => None,
  };
  // --- Scan phase: peel boxes from both sides ---
  while let Some(front_box) = boxes.pop_front() {
    match front_box.data() {
      DigestedData::List(_) => {
        for fbox in front_box.unlist().into_iter().rev() {
          boxes.push_front(fbox);
        }
      },
      _ if front_box.get_property("isFill").is_some() => {
        align = Align::Right;
        break;
      },
      _ if front_box.get_property("isVerticalRule").is_some() => {
        border.push('l');
        // Perl L487-489: space before | ? move lspaces to previous column's rspaces
        if !lspaces.is_empty() {
          prev_rspaces_transfer = Some(std::mem::take(&mut lspaces));
        }
      },
      _ if front_box.get_property("isSpace").is_some()
        && front_box.get_property("isVerticalSpace").is_none() =>
      {
        lspaces.push(front_box);
      },
      item
        if front_box.get_property("isHorizontalRule").is_some()
          || front_box.get_property("alignmentSkippable").is_some()
          || matches!(item, DigestedData::Comment(_))
          || front_box.is_empty()? =>
      {
        saveleft.push_front(front_box)
      },
      _ => {
        boxes.push_front(front_box);
        break;
      },
    }
  }
  while let Some(last_box) = boxes.pop_back() {
    match last_box.data() {
      DigestedData::List(_) => {
        for lbox in last_box.unlist().into_iter() {
          boxes.push_back(lbox);
        }
      },
      _ if last_box.get_property("isFill").is_some() => {
        if align == Align::Right {
          align = Align::Center
        };
        break;
      },
      _ if last_box.get_property("isVerticalRule").is_some() => {
        border.push('r');
        rspaces.clear(); // Perl L508: discard spacing after rule
      },
      _ if last_box.get_property("isSpace").is_some()
        && last_box.get_property("isVerticalSpace").is_none() =>
      {
        rspaces.insert(0, last_box);
      },
      item
        if last_box.get_property("isHorizontalRule").is_some()
          || last_box.get_property("alignmentSkippable").is_some()
          || matches!(item, DigestedData::Comment(_))
          || last_box.is_empty()? =>
      {
        saveright.push_front(last_box);
      },
      _ => {
        boxes.push_back(last_box);
        break;
      },
    }
  }
  // Fallback: if only one fill was found but the template expects two, use template's alignment.
  if !is_omitted {
    if let Some(expected) = expected_from_template {
      if (align == Align::Right && expected == Align::Center)
        || (align == Align::Left && expected != Align::Left)
      {
        align = expected;
      }
    }
  }
  // Replacing boxes with the fil padding & vertical rules stripped off
  let mut final_boxes = Vec::from(saveleft);
  final_boxes.extend(boxes);
  final_boxes.extend(saveright);
  let mut boxes_list = List::new(final_boxes);
  boxes_list.mode = Some(if is_math {
    TexMode::Math
  } else {
    TexMode::Text
  });
  let digested_out = Digested::from(boxes_list);

  // --- Write phase: apply to previous column (Perl L487-489) ---
  if let Some(transfer) = prev_rspaces_transfer {
    if n0 >= 2 {
      if let Some(prev_col) = alignment.get_column(n0 - 1) {
        // Perl: $$prev{rspaces} = List(($$prev{rspaces} || ()), @lspaces)
        let mut all_rspaces = Vec::new();
        if let Some(existing) = prev_col.rspaces.take() {
          all_rspaces.extend(existing.unlist());
        }
        all_rspaces.extend(transfer);
        prev_col.rspaces = Some(List::new(all_rspaces).into());
      }
    }
  }

  // --- Write phase: apply to current column ---
  let in_thead = alignment.is_in_tabular_head();
  {
    let colspec = alignment.get_column(n0).unwrap();
    let is_justify = align == Align::Justify;
    colspec.align = Some(align);
    if !is_justify {
      colspec.width = None;
    }
    let new_border = s!("{}{}", old_border, border);
    colspec.border = new_border;
    colspec.boxes = Some(digested_out.clone());
    // Perl L526-527: store lspaces/rspaces
    if !lspaces.is_empty() {
      colspec.lspaces = Some(List::new(lspaces).into());
    }
    if !rspaces.is_empty() {
      colspec.rspaces = Some(List::new(rspaces).into());
    }
    colspec.colspan = Some(if n1 >= n0 { n1 - n0 + 1 } else { 1 });
    // Perl L530-534: mark thead columns
    if in_thead {
      colspec.thead_in_column = true;
    }
  }
  // Mark skipped (spanned) columns
  for i in (n0 + 1)..=n1 {
    if let Some(c) = alignment.get_column(i) {
      c.skipped = true;
    }
  }
  Ok(digested_out)
}

// #######
// Support for \\[dim] .... TO BE WORKED OUT!
// NOTE that this does NOT skip spaces before * or []!!!!!
//  As if: \lx@alignment@newline OptionalMatch:* [Dimension]
// Read arguments for \\, namely * and/or [Dimension]
// BUT optionally do it while skipping spaces (latex style) or not (ams style)
fn read_newline_args(skipspaces: bool) -> Result<(bool, Option<Tokens>)> {
  if lookup_alignment().is_some() {
    local_align_group_count(1000000);
    if skipspaces {
      gullet::skip_spaces()?;
    }
    let (mut star, mut optional) = (false, None);
    let mut next_opt = gullet::read_token()?;
    if next_opt == Some(T_OTHER!("*")) {
      star = true;
      if skipspaces {
        gullet::skip_spaces()?;
      }
      next_opt = gullet::read_token()?;
    }
    if next_opt == Some(T_OTHER!("[")) {
      optional = Some(gullet::read_until(&Tokens!(T_OTHER!("]")))?);
      next_opt = None;
    }
    if let Some(next) = next_opt {
      gullet::unread_one(next);
    }
    expire_align_group_count();
    Ok((star, optional))
  } else {
    Err("read_newline_args should only be called with a proper 'Alignment' active in state".into())
  }
}

// Recognise "implicit alignment tab" CS tokens, i.e. `\let\amp=&`.
// Real TeX treats `\amp` (whose meaning is the `&` char-token with
// catcode ALIGN) as an alignment tab in `\halign` preambles and
// bodies — see texbook.tex p.~277 ("implicit characters") and
// tex.web @<Manufacture a control...@> (cur_cmd dispatch by meaning,
// not by token catcode).
fn is_implicit_align(t: &Token) -> bool {
  if t.get_catcode() != Catcode::CS {
    return false;
  }
  matches!(
    latexml_core::state::lookup_meaning(t),
    Some(Stored::Token(tt)) if tt.get_catcode() == Catcode::ALIGN
  )
}

// `\cr` / `\crcr` `\let`-equivalents. Less common in the wild than
// implicit `&` but covered by the same Knuth-TeX semantics. The body
// path uses the analogous `gullet::is_column_end` which does meaning-
// equality against the COLUMN_ENDS table — keep both code paths in
// sync. Two shapes of implicit-CR are observed:
//
//   - `\let\rowEnd=\cr` while `\cr` is a Constructor (LaTeXML's normal
//     state): meaning of `\rowEnd` becomes `Stored::Constructor` with
//     the same `.cs` as `\cr`. Use meaning-equality against
//     `lookup_meaning(\cr)`.
//   - `\let\rowEnd=<token-CS>`: meaning is `Stored::Token(<\cr>)`. Use
//     the by-name fallback (matches when no engine binding has shipped
//     a proper `\cr` Constructor / Primitive yet).
fn is_implicit_cr(t: &Token) -> bool {
  if t.get_catcode() != Catcode::CS {
    return false;
  }
  let defn = latexml_core::state::lookup_meaning(t);
  let Some(defn) = defn else { return false; };
  // Meaning-equality (handles `\let \rowEnd \cr` where `\cr` is a
  // Constructor / Primitive — the LaTeXML-default state).
  for cr_cs in &[T_CS!("\\cr"), T_CS!("\\crcr")] {
    if let Some(cr_defn) = latexml_core::state::lookup_meaning(cr_cs) {
      if defn == cr_defn {
        return true;
      }
    }
  }
  // Fallback: meaning IS the CS token `\cr` / `\crcr` (raw alias form).
  if let Stored::Token(tt) = defn {
    if tt.get_catcode() == Catcode::CS {
      return tt.with_str(|s| s == "\\cr" || s == "\\crcr");
    }
  }
  false
}

// Perl TeX_Tables L187-240: Parse an \halign style alignment template from Gullet
pub fn parse_halign_template(whatsit: &mut Whatsit) -> Result<Template> {
  let t = gullet::read_non_space()?;
  // Perl L190: $t->defined_as(T_BEGIN) — checks \let aliases like \bgroup
  if !t
    .as_ref()
    .map(|t| t.defined_as(&T_BEGIN!()))
    .unwrap_or(false)
  {
    Error!("expected", "\\bgroup", "Missing \\halign box");
    // Put back the token we consumed so it can be handled elsewhere
    if let Some(tok) = t {
      gullet::unread(Tokens::from(tok));
    }
    return Ok(Template::default());
  }
  let mut before = true; // true if we're before a # in current column
  let mut pre: Vec<Token> = Vec::new();
  let mut post: Vec<Token> = Vec::new();
  let mut cols: Vec<Cell> = Vec::new();
  let mut repeated = false;
  let mut nonreps: Vec<Cell> = Vec::new();
  let mut tokens: Vec<Token> = Vec::new();
  // Perl L197-198: track tabskip per column
  let mut tabskip = match lookup_register("\\tabskip", Vec::new())? {
    Some(RegisterValue::Glue(g)) => g,
    _ => Glue::new(0),
  };
  let mut nexttabskip = tabskip;
  // Only expand certain things; See TeX book p.238
  local_align_group_count(1000000);
  while let Some(t) = gullet::read_token()? {
    let cc = t.get_catcode();
    if t == T_CS!("\\tabskip") {
      // Read the tabskip assignment
      gullet::read_keyword(&["="])?;
      nexttabskip = gullet::read_glue()?;
    } else if t == T_CS!("\\span") {
      // ex-span-ded next token
      let expanded = gullet::read_x_token(Some(false), false, None)?;
      if let Some(xt) = expanded {
        gullet::unread_one(xt);
      }
    } else if cc == Catcode::PARAM {
      // Found the template's column slot
      before = false;
      tokens.push(t);
    } else if cc == Catcode::ALIGN || t == T_CS!("\\cr") || t == T_CS!("\\crcr")
      || is_implicit_align(&t) || is_implicit_cr(&t) {
      // End the column
      if before {
        // Leading & means repeated columns
        repeated = true;
        nonreps = std::mem::take(&mut cols);
      } else {
        // Finished column spec; add it
        cols.push(Cell {
          before: if pre.is_empty() {
            None
          } else {
            Some(Tokens::new(before_cell_unlist(std::mem::take(&mut pre))))
          },
          after: if post.is_empty() {
            None
          } else {
            Some(Tokens::new(after_cell_unlist(std::mem::take(&mut post))))
          },
          tabskip: if tabskip.value_of() != 0 {
            Some(tabskip)
          } else {
            None
          },
          ..Cell::default()
        });
        tabskip = nexttabskip;
        pre.clear();
        post.clear();
        before = true;
      }
      if cc != Catcode::ALIGN && !is_implicit_align(&t) {
        break; // \cr or \crcr (explicit or implicit) ends the template
      }
      tokens.push(t);
    } else if before {
      // Other random tokens go into the column's pre-template
      if !pre.is_empty() || cc != Catcode::SPACE {
        pre.push(t);
      }
      tokens.push(t);
    } else {
      // Or the post-template
      if !post.is_empty() || cc != Catcode::SPACE {
        post.push(t);
      }
      tokens.push(t);
    }
  }
  expire_align_group_count();
  // Store the template's token representation for reversion
  let template_tokens = Tokens::new(tokens.clone());
  whatsit.set_property("template_tokens", Stored::Tokens(template_tokens));
  // Now create & return the template object
  let template = if repeated {
    Template::new(TemplateConfig {
      columns: Some(nonreps),
      repeated: cols,
      tokens: Some(tokens),
      ..TemplateConfig::default()
    })
  } else {
    Template::new(TemplateConfig {
      columns: Some(cols),
      tokens: Some(tokens),
      ..TemplateConfig::default()
    })
  };
  Ok(template)
}

// Perl TeX_Tables L735-745: beforeCellUnlist
// Reorder `$ \hfil` → `\hfil $` in template tokens
fn before_cell_unlist(tokens: Vec<Token>) -> Vec<Token> {
  let mut toks: VecDeque<Token> = tokens.into();
  let mut result = Vec::new();
  while let Some(t) = toks.pop_front() {
    if t == T_MATH!() {
      if let Some(hfil) = toks.pop_front_if(|next| *next == T_CS!("\\hfil")) {
        result.push(hfil); // push \hfil
        toks.push_front(t); // put $ back
        continue;
      }
    }
    result.push(t);
  }
  result
}

// Perl TeX_Tables L747-757: afterCellUnlist
// Reorder `\hfil $` → `$ \hfil` in template tokens (reverse scan)
fn after_cell_unlist(tokens: Vec<Token>) -> Vec<Token> {
  let mut toks: VecDeque<Token> = tokens.into();
  let mut result: VecDeque<Token> = VecDeque::new();
  while let Some(t) = toks.pop_back() {
    if t == T_MATH!() {
      if let Some(hfil) = toks.pop_back_if(|prev| *prev == T_CS!("\\hfil")) {
        result.push_front(hfil); // push \hfil to front
        toks.push_back(t); // put $ back
        continue;
      }
    }
    result.push_front(t);
  }
  result.into()
}

#[cfg(test)]
mod tests {
  use super::*;
  use latexml_core::state::{State, StateOptions, set_state};

  fn setup() {
    // T_CS!, T_MATH! need a thread-local State for string interning.
    set_state(State::new(StateOptions::default()));
  }

  #[test]
  fn before_cell_unlist_reorders_dollar_hfil() {
    setup();
    // `$ \hfil foo` → `\hfil $ foo`
    let input = vec![T_MATH!(), T_CS!("\\hfil"), T_LETTER!("f")];
    let got = before_cell_unlist(input);
    assert_eq!(got, vec![T_CS!("\\hfil"), T_MATH!(), T_LETTER!("f")]);
  }

  #[test]
  fn before_cell_unlist_no_hfil_after_dollar_is_identity() {
    setup();
    let input = vec![T_MATH!(), T_LETTER!("x")];
    let got = before_cell_unlist(input.clone());
    assert_eq!(got, input);
  }

  #[test]
  fn before_cell_unlist_empty_stays_empty() {
    setup();
    assert!(before_cell_unlist(vec![]).is_empty());
  }

  #[test]
  fn before_cell_unlist_hfil_without_dollar_is_identity() {
    setup();
    let input = vec![T_CS!("\\hfil"), T_LETTER!("x")];
    let got = before_cell_unlist(input.clone());
    assert_eq!(got, input);
  }

  #[test]
  fn after_cell_unlist_reorders_hfil_dollar() {
    setup();
    // `foo \hfil $` → `foo $ \hfil`
    let input = vec![T_LETTER!("f"), T_CS!("\\hfil"), T_MATH!()];
    let got = after_cell_unlist(input);
    assert_eq!(got, vec![T_LETTER!("f"), T_MATH!(), T_CS!("\\hfil")]);
  }

  #[test]
  fn after_cell_unlist_no_hfil_before_trailing_dollar_is_identity() {
    setup();
    let input = vec![T_LETTER!("x"), T_MATH!()];
    let got = after_cell_unlist(input.clone());
    assert_eq!(got, input);
  }

  #[test]
  fn after_cell_unlist_empty_stays_empty() {
    setup();
    assert!(after_cell_unlist(vec![]).is_empty());
  }

  #[test]
  fn before_and_after_are_inverses_on_canonical_layout() {
    // Running after then before on the "cell form" returns the "template form",
    // and vice versa. Verify round-trip for the canonical 3-token pattern.
    setup();
    let template = vec![T_MATH!(), T_CS!("\\hfil"), T_LETTER!("x")];
    let cell = before_cell_unlist(template.clone()); // \hfil $ x
    let back = after_cell_unlist(cell); // $ \hfil x
    assert_eq!(back, template);
  }
}
