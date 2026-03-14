//! TeX Tables
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
use latexml_core::alignment::read_alignment_template;
use latexml_core::alignment::template::TemplateConfig;
use std::cell::{RefCell, RefMut};
use std::collections::VecDeque;

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
  DefConstructor!("\\lx@begin@alignment SkipSpaces", "#alignment",
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
      tks.extend(pattern.clone().unlist());
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
    bgroup();
    Error!("unexpected", "\\noalign", "\\noalign cannot be used here");
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
        tks.extend(template_tks.clone().unlist());
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
      digest_alignment_body(whatsit)?;
      end_mode("restricted_horizontal")?;
      decrement_align_group_count(); // Balance the opening { OUTSIDE of the masking of ALIGN_STATE
    });

  DefMacro!("\\lx@alignment@row@before", None);
  DefMacro!("\\lx@alignment@row@after", None);
  DefMacro!("\\lx@alignment@column@before", None);
  DefMacro!("\\lx@alignment@column@after", None);

  //======================================================================
  // Vertical alignments
  //----------------------------------------------------------------------
  // \valign           c  begins the vertical alignment of material (i.e., makes a table containing
  // columns).

  // Implement ???
  // DefMacro('\vrule','\relax');
  DefMacro!("\\valign", None);

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
      tokens.push(T_END!());
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
    let reverted = whatsit.revert()?;
    Ok(Tokens!(T_CS!("\\\\"), T_OTHER!("["), reverted, T_OTHER!("]"), T_CR!()))
  });

  // Perl: \lx@intercol is our replacement for LaTeX's \@acol for intercolumn space
  DefMacro!("\\lx@intercol", "");
  // Perl: Candidates for binding \lx@intercol for LaTeX tabular or math arrays
  DefConstructor!("\\lx@text@intercol", sub[document, _args, props] {
    if let Some(width) = props.get("width") {
      let dim: Option<Dimension> = (&*width).into();
      if let Some(d) = dim {
        let s = dimension_to_spaces(d);
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
      // Sometimes, we"re smuggling stuff that needs to be carried into the XML.
      let mut props = stored_map!("alignmentSkippable" => true);
      if let Some(preserve) = args.iter().find(|v_opt| if let Some(ref v) = v_opt {
        v.get_property("alignmentPreserve").is_some()
      } else { false }) {
        props.insert("alignmentPreserve", preserve.as_ref().unwrap().into());
      }
      Ok(props) });

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

  DefMacro!("\\@tabular@begin@heading", {
    if let Some(alignment_stored) = lookup_alignment() {
      alignment_stored
        .alignment_cell()
        .unwrap()
        .borrow_mut()
        .set_in_tabular_head();
    }
  });
  DefMacro!("\\@tabular@end@heading", {
    if let Some(alignment_stored) = lookup_alignment() {
      alignment_stored
        .alignment_cell()
        .unwrap()
        .borrow_mut()
        .unset_in_tabular_head();
    }
  });

  //======================================================================
  // Multicolumn support
  // DefRegisterI('\@alignment@ncolumns', undef, Dimension(0),
  //   getter => sub {
  //     if (my $alignment = LookupValue('Alignment')) {
  //       Number(scalar($alignment->getTemplate->columns)); }
  //     else { Number(0); } });
  // DefRegisterI('\@alignment@column', undef, Dimension(0),
  //   getter => sub {
  //     if (my $alignment = LookupValue('Alignment')) {
  //       Number($alignment->currentColumnNumber); }
  //     else { Number(0); } });

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
  properties: SymHashMap<Stored>,
  xml_attributes: HashMap<String, String>,
) {
  let mode = if mode.is_empty() {
    state::lookup_string("MODE")
  } else {
    mode
  };
  let is_math = mode.ends_with("math");
  let (container, rowtype, coltype) = if is_math {
    ("ltx:XMArray", "ltx:XMRow", "ltx:XMCell")
  } else {
    ("ltx:tabular", "ltx:tr", "ltx:td")
  };
  let alignment = Alignment::new(AlignmentConfig {
    template: Some(template),
    open_container: Rc::new(|document, props| {
      document
        .open_element(container, Some(props), None)
        .map(Option::Some)
    }),
    close_container: Rc::new(|document| document.close_element(container)),
    open_row: Rc::new(|document, props| {
      document
        .open_element(rowtype, Some(props), None)
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
    if (vtype.is_none() || vtype.as_ref().unwrap().is_empty())
      && (next.is_none() || next == Some(T_END!()) || next == Some(T_CS!("\\lx@close@alignment")))
    {
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
  let ismath = lookup_bool("IN_MATH");
  // Scan for leading \omit, skipping over (& saving) \hline.
  //   Debug("Halign $alignment: COLUMN starting scan "
  //       . "(" . ($ismath ? " math" : " text") . ")") if $LaTeXML::DEBUG{halign};
  let mut last_token: Option<Token> = None;
  let mut spanning = false;
  loop {
    // Outer loop; collects 1 column (possibly multiple spans) return from within!
    // Scan till we get something NOT \omit, \noalign
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
    if last_token.is_none()
      || last_token == Some(T_END!())
      || last_token == Some(T_CS!("\\lx@close@alignment"))
    {
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
  let is_math = lookup_bool("IN_MATH");
  //Note: $n0,$n1 is a VERY round-about way of tracking the column spanning!
  let n0 = lookup_int("alignmentStartColumn") as usize + 1;
  let n1 = alignment.current_column_number();
  let colspec = match alignment.get_column(n0) {
    Some(c) => c,
    None => {
      // Column doesn't exist — return the input unchanged
      return Ok(Digested::default());
    }
  };
  // Perl: $align = $$colspec{align} || 'left' — default is left, fills override
  let mut align = colspec.align.unwrap_or(Align::Left);
  let mut border = String::new();
  // Peel off any boxes from both sides until we get the "meat" of the column.
  // from this we can establish borders, alignment and emptiness.
  // But we, of course, immediately put them back...
  let mut saveleft = VecDeque::new();
  let mut saveright = VecDeque::new();
  let mut lspaces: Vec<Digested> = Vec::new();
  let mut rspaces: Vec<Digested> = Vec::new();
  // Perl L476-477: add tabskip as \hskip to lspaces
  if let Some(skip) = &colspec.tabskip {
    if skip.value_of() != 0 {
      let hskip_toks = Tokens!(
        T_CS!("\\hskip"),
        ExplodeText!(&skip.to_string()),
        T_CS!("\\relax")
      );
      let hskip_digested = stomach::digest(hskip_toks)?;
      lspaces.push(hskip_digested);
    }
  }
  // Determine expected alignment from template fills, as a fallback for when
  // the trailing fill box is lost during digestion (known issue with nested \hbox groups).
  let has_before_fill = colspec
    .before
    .as_ref()
    .map(|b| b.unlist_ref().iter().any(|t| *t == T_CS!("\\hfil") || *t == T_CS!("\\hfill")))
    .unwrap_or(false);
  let has_after_fill = colspec
    .after
    .as_ref()
    .map(|a| a.unlist_ref().iter().any(|t| *t == T_CS!("\\hfil") || *t == T_CS!("\\hfill")))
    .unwrap_or(false);
  let expected_from_template = match (has_before_fill, has_after_fill) {
    (true, true) => Some(Align::Center),
    (false, true) => Some(Align::Left),
    (true, false) => Some(Align::Right),
    (false, false) => None,
  };
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
        // Perl L487-489: space before | ? move lspaces to previous column
        // (deferred until after we release colspec borrow)
        lspaces.clear(); // discard lspaces when border found
      },
      _ if front_box.get_property("isSpace").is_some() => {
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
        // put the box back, and terminate left side loop.
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
      _ if last_box.get_property("isSpace").is_some() => {
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
        // put the box back, and terminate right side loop.
        boxes.push_back(last_box);
        break;
      },
    }
  }
  // Fallback: if only one fill was found but the template expects two, use template's alignment.
  // This handles cells where the trailing \hfil was lost during digestion (e.g. after \hbox groups).
  // Skip for omitted columns (multicolumn case) — they have their own fills, not the parent template's.
  if !colspec.omitted {
    if let Some(expected) = expected_from_template {
      if (align == Align::Right && expected == Align::Center)
        || (align == Align::Left && expected != Align::Left)
      {
        align = expected;
      }
    }
  }
  if align != Align::Justify {
    colspec.width = None;
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
  // record relevant info in the Alignment.
  colspec.align = Some(align);
  border = s!("{}{}", colspec.border, border);
  colspec.border = border;
  colspec.boxes = Some(digested_out.clone());
  // Perl L526-527: store lspaces/rspaces
  if !lspaces.is_empty() {
    colspec.lspaces = Some(List::new(lspaces).into());
  }
  if !rspaces.is_empty() {
    colspec.rspaces = Some(List::new(rspaces).into());
  }
  colspec.colspan = Some(if n1 >= n0 { n1 - n0 + 1 } else { 1 });
  //   if ($$alignment{in_tabular_head} || $$alignment{in_tabular_foot}) {
  //     $$colspec{thead}{column} = 1; }
  //   for (my $i = $n0 + 1 ; $i <= $n1 ; $i++) {
  //     my $c = $alignment->getColumn($i);
  //     $$c{skipped} = 1 if $c; }
  //   Debug("Halign $alignment: INSTALL column " . join(',', map { $_ . "=" .
  // ToString($$colspec{$_}); } sort keys %$colspec)) if $LaTeXML::DEBUG{halign};
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

// Perl TeX_Tables L187-240: Parse an \halign style alignment template from Gullet
fn parse_halign_template(whatsit: &mut Whatsit) -> Result<Template> {
  let t = gullet::read_non_space()?;
  if t.as_ref().map(|t| t.get_catcode()) != Some(Catcode::BEGIN) {
    Error!("expected", "\\bgroup", "Missing \\halign box");
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
    } else if cc == Catcode::ALIGN
      || t == T_CS!("\\cr")
      || t == T_CS!("\\crcr")
    {
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
      if cc != Catcode::ALIGN {
        break; // \cr or \crcr ends the template
      }
      tokens.push(t);
    } else if before {
      // Other random tokens go into the column's pre-template
      if !pre.is_empty() || cc != Catcode::SPACE {
        pre.push(t.clone());
      }
      tokens.push(t);
    } else {
      // Or the post-template
      if !post.is_empty() || cc != Catcode::SPACE {
        post.push(t.clone());
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
      if let Some(next) = toks.front() {
        if *next == T_CS!("\\hfil") {
          result.push(toks.pop_front().unwrap()); // push \hfil
          toks.push_front(t); // put $ back
          continue;
        }
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
      if let Some(prev) = toks.back() {
        if *prev == T_CS!("\\hfil") {
          result.push_front(toks.pop_back().unwrap()); // push \hfil to front
          toks.push_back(t); // put $ back
          continue;
        }
      }
    }
    result.push_front(t);
  }
  result.into()
}
