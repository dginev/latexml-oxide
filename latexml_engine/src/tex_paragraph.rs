//! TeX Paragraph
//!
//! Core TeX Implementation for LaTeXML
use latexml_core::document::helpers::prune_empty_para;

use crate::prelude::*;

/// Helper used by `\leftline`/`\rightline`/`\centerline` and friends.
/// Perl `TeX_Paragraph.pool.ltxml:75` `sub alignLine`.
pub fn align_line(
  document: &mut Document,
  line: &[Option<Digested>],
  alignment: &str,
) -> Result<()> {
  if document.is_openable("ltx:p") {
    let line_content = line.iter().filter_map(|c| c.as_ref()).collect();
    document.insert_element(
      "ltx:p",
      line_content,
      Some(string_map!("class" => s!("ltx_align_{alignment}"))),
    )?;
  } else if document.is_openable("ltx:text") {
    let line_content = line.iter().filter_map(|c| c.as_ref()).collect();
    document.insert_element(
      "ltx:text",
      line_content,
      Some(string_map!("class" => s!("ltx_align_{alignment}"))),
    )?;
    document.insert_element("ltx:break", Vec::new(), None)?;
  } else if let Some(Some(line_content)) = line.first() {
    document.absorb(line_content, None)?;
  }
  Ok(())
}

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Paragraph Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // Spacing tweaks
  //----------------------------------------------------------------------
  // \ignorespaces           c  makes TeX read and expand tokens but do nothing until a nonspace
  // token is reached. \noboundary             c  if present, breaks ligatures and kerns.
  // \vadjust                c  inserts a vertical list between two lines in a paragraph.

  def_primitive_noop("\\ignorespaces SkipSpaces")?;
  def_primitive_noop("\\noboundary")?;
  // \vadjust<filler>{<vertical mode material>}
  // Note: \vadjust ignores in vertical mode...
  DefPrimitive!("\\vadjust {}", sub[(arg)] { push_tokens("vAdjust", arg); });

  //======================================================================
  // Basic Paragraph
  //----------------------------------------------------------------------
  // \everypar               pt holds tokens added at the beginning of every paragraph.
  // \indent                 c  begins a new paragraph indented by \parindent.
  // \noindent               c  begins a new paragraph that is not indented.
  // \par                    c  is an explicit command to end a paragraph.
  DefRegister!("\\everypar", Tokens!());
  // These determine whether the _next_ paragraph gets indented!
  // thus it needs \par to check whether such indentation has been set.
  DefConstructor!("\\indent", sub[document] {
    if let Some(mut node) = document.get_element() {
      let tag = document::get_node_qname(&node);
      let para_tag = pin_static("ltx:para");
      if tag == para_tag {
        node.set_attribute("class","ltx_indent")?;
      } else if document::sym_can_contain_somehow(tag, para_tag).is_some() {
        // Perversely ignore indent on 1st para after sectioning titles (Perl parity)
        let prev_is_title = node.get_last_child().map(|prev| {
          let prev_qname = document::get_node_qname(&prev);
          with(prev_qname, |s| s == "ltx:title" || s == "ltx:toctitle")
        }).unwrap_or(false);
        if prev_is_title {
          document.open_element("ltx:para", None, None)?;
        } else {
          document.open_element("ltx:para", Some(string_map!("class"=>"ltx_indent")), None)?;
        }
      }
      // Otherwise ignore.
    }
  },
  properties => { stored_map!("isSpace" => true) },
  // Perl: enterHorizontal => 1
  before_digest => { enter_horizontal(); });
  DefConstructor!("\\noindent", sub[document] {
    if let Some(mut node) = document.get_element() {
      let tag = document::get_node_qname(&node);
      let para_tag = pin_static("ltx:para");
      if tag == para_tag {
        node.set_attribute("class","ltx_noindent")?;
      } else if document::sym_can_contain_somehow(tag, para_tag ).is_some() {
        // Used in a position where a paragraph can be started, start
        document.open_element("ltx:para", Some(string_map!("class"=>"ltx_noindent")), None)?;
      }
      // Otherwise ignore.
    }
  },
  properties => { stored_map!("isSpace" => true) },
  // Perl: enterHorizontal => 1
  before_digest => { enter_horizontal(); });

  // <ltx:para> represents a Logical Paragraph, whereas <ltx:p> is a `physical paragraph'.
  // A para can contain both p and displayed equations and such.

  // Remember; \par _closes_, not opens, paragraphs!
  // Here, we want to close both an open p and para (if either are open).
  // NOTE Also that the whole inPreamble bit is, I think, overused.
  // For example, \par should be a NOOP in vertical mode, and that would generally make it
  // ignored in the preamble.
  let mut skippable_props: SymHashMap<Stored> = SymHashMap::default();
  skippable_props.insert("alignmentSkippable", Stored::Bool(true));

  DefConstructor!("\\lx@normal@par",
    sub[document, _args, props] {
      if !prop_bool!(props, "inPreamble") {
        document.maybe_close_element("ltx:p")?;
        let element = document.get_element();
        if let Some(mut node) = element {
          let qname = document::get_node_qname(&node);
          // Only set on the para about to close, if unknown!
          if qname == pin_static("ltx:para") && node.get_attribute("class").is_none() {
            let class_sym = prop_str!(props,"class");
            if class_sym != pin!("") {
              let class_s = with(class_sym, |s| s.to_string());
              document.set_attribute(&mut node, "class", &class_s)?;
            }
          } else if qname == pin_static("ltx:figure") {
            // insert breaks in figures, for vertically separating subfigures
            document.insert_element("ltx:break",Vec::new(), None)?;
          }
        }
        if !prop_bool!(props, "internal_par") {
          document.maybe_close_element("ltx:para")?;
        }
      }
    },
    before_digest => {
      // Perl: combine any digested horizontal material into a horizontal List
      let mode = lookup_string_from_sym(pin!("MODE"));
      let bound = lookup_string_from_sym(pin!("BOUND_MODE"));
      if mode == "horizontal" && bound.ends_with("vertical") {
        // Perl: $stomach->repackHorizontal;
        repack_horizontal();
        assign_value_inplace_sym(pin!("MODE"), bound); // Resume vertical/internal_vertical
      }
      assign_value("parshape", Stored::None, None);
      assign_value("interlinepenalties", Stored::None, None);
    },
    after_digest => sub[whatsit] {
      whatsit.set_property("mode", lookup_string_from_sym(pin!("MODE")));
      // When invoked by leave_horizontal: no reversion, don't close ltx:para
      if LookupBool!("INTERNAL_PAR") {
        whatsit.set_property("internal_par", true);
        whatsit.set_property("reversion", Tokens!());
      }
      let in_preamble = LookupBool!("inPreamble");
      if in_preamble {
        whatsit.set_property("inPreamble", true);
        Ok(Vec::new())
      } else {
        if let Some(c) = lookup_value("next_para_class") {
          // Check if flags were set by prior \par:
          whatsit.set_property("class", c);
          { assign_value("next_para_class", Stored::None, None); }
        }
        // Per eTeX spec, \interlinepenalties (like \parshape) is reset after each paragraph.
        { assign_value("interlinepenalties", Stored::None, None); }
        // Fish out flags for next ltx:para, to be used when the next \par closes:
        // `\parindent` is normally defined; if it isn't (None), don't assume zero
        // and force noindent — skip the override. Witness: 1502.07281.
        if lookup_register("\\parindent", Vec::new())?.is_some_and(|r| r.value_of() == 0) {
          // respect \parindent if no overrides are given
          { assign_value("next_para_class", "ltx_noindent", None); }
        }
        // Vertical adjustments
        match remove_value("vAdjust") { Some(Stored::Tokens(vadj)) => {
          assign_value("vAdjust", Tokens!(), Some(Scope::Global));
          Ok(vec![ Digest!(vadj)? ])
        } _ => {
          Ok(Vec::new())
        }}
      }
    },
    properties => skippable_props,
    alias => "\\par"
  );
  Let!("\\par", "\\lx@normal@par");
  Tag!("ltx:para", auto_close => true, auto_open => true,
    after_close => sub[document, node] {
      prune_empty_para(document, node)?;
  });
  Tag!("ltx:p", auto_close => true, auto_open => true,
    after_close => sub[document, node] {
      document.trim_node_whitespace(node)?;
  });

  //======================================================================
  // Paragraph Shape
  //----------------------------------------------------------------------
  // \parshape               iq specifies an arbitrary paragraph shape.
  // Acts like a Number register (returns count of lines when read).
  // Setter reads n pairs of dimensions and stores them in state.
  DefRegister!("\\parshape", Number::new(0),
    getter => sub[_args] {
      with_value("parshape", |val_opt| {
        if let Some(Stored::VecDequeStored(shape)) = val_opt {
          Some(RegisterValue::Number(Number::new((shape.len() / 2) as i64)))
        } else {
          Some(RegisterValue::Number(Number::new(0)))
        }
      })
    },
    setter => sub[value, _scope, _args] {
      let n_val = value.value_of();
      let n = if n_val < 0 { 0 } else { n_val } as usize;
      let mut shape = VecDeque::new();
      for _ in 0..n {
        let indent = read_dimension().unwrap_or_default();
        let length = read_dimension().unwrap_or_default();
        shape.push_back(Stored::Dimension(indent));
        shape.push_back(Stored::Dimension(length));
      }
      assign_value(
        "parshape",
        if n > 0 { Stored::VecDequeStored(shape) } else { Stored::None },
        Some(Scope::Global),
      );
    }
  );
  //======================================================================
  // Paragraph Shape
  //----------------------------------------------------------------------
  // \prevgraf               iq is the number of lines in the paragraph most recently completed or
  // partially completed. \spacefactor            iq controls interword spacing.
  // \emergencystretch       pd is glue used in the third pass made for bad paragraphs.
  // \hangindent             pd is the amount of hanging indentation.
  // \hsize                  pd is the width of normal lines in a paragraph.
  // \lineskiplimit          pd is the cutoff used to select between \baselineskip and \lineskip.
  // \parindent              pd is the width of indentation at the beginning of a paragraph.
  // \baselineskip           pg is glue added between lines to keep their baselines consistently
  // spaced. \leftskip               pg is glue added at the left of every line in a paragraph.
  // \rightskip              pg is glue added at the right of every line in a paragraph.
  // \lineskip               pg is alternate interline glue used if the \baselineskip glue is not
  // feasible   . \parskip                pg is extra glue put between paragraphs.
  // \parfillskip            pg is glue which finishs the last line of a paragraph.
  // \spaceskip              pg is alternate interword glue.
  // \xspaceskip             pg is alternate intersentence glue.
  // \adjdemerits            pi holds the demerits for visually incompatible adjacent lines.
  // \doublehyphendemerits   pi holds the demerits added if two consecutive lines end with
  // discretionary breaks. \finalhyphendemerits    pi holds the demerits added if the penultimate
  // line in a paragraph ends with a discretionary break. \hangafter              pi is the number
  // of lines before hanging indentation changes. \looseness              pi tells TeX to try and
  // increase or decrease the number of lines in a paragraph.

  // \tolerance              pi is the acceptable \badness of lines after hyphenation.
  // \pretolerance           pi is the acceptable \badness of lines in a paragraph before
  // hyphenation is attempted.
  DefRegister!("\\spacefactor", Number!(0));
  DefRegister!("\\prevgraf", Number!(0));
  DefRegister!("\\emergencystretch", Dimension!("0"));
  DefRegister!("\\hangindent", Dimension!("0"));
  DefRegister!("\\hsize", Dimension!("6.5in"));
  DefRegister!("\\lineskiplimit", Dimension!("0"));
  DefRegister!("\\parindent", Dimension!("20pt"));
  DefRegister!("\\baselineskip", Glue!("12pt"));
  DefRegister!("\\leftskip", Glue!("0"));
  DefRegister!("\\rightskip", Glue!("0"));
  DefRegister!("\\lineskip", Glue!("1pt"));
  DefRegister!("\\parskip", Glue!("0pt plus 1pt"));
  DefRegister!("\\parfillskip", Glue!("0pt plus 1fil"));
  DefRegister!("\\spaceskip", Glue!("0"));
  DefRegister!("\\xspaceskip", Glue!("0"));
  DefRegister!("\\adjdemerits", Number!(10000));
  DefRegister!("\\doublehyphendemerits", Number!(10000));
  DefRegister!("\\finalhyphendemerits", Number!(5000));
  DefRegister!("\\hangafter", Number!(0));
  DefRegister!("\\looseness", Number!(0));
  DefRegister!("\\tolerance", Number!(200));
  DefRegister!("\\pretolerance", Number!(100));
});
