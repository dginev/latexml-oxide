use crate::package::*;
use rtx_core::common::object::Object;
use rtx_core::alignment::read_alignment_template;
use rtx_core::alignment::template::Template;
use std::cell::{RefMut,RefCell};
//======================================================================
// Basic alignment support needed by most environments & commands.
//======================================================================
LoadDefinitions!(outer_state, {
  DefParameterType!(AlignmentTemplate, sub[gullet, _inner, _extra, state] {
    read_alignment_template(gullet, state)
  });

  Tag!("ltx:td", after_close => sub[doc, node, _state] { doc.trim_node_whitespace(node)?; });

  //----------------------------------------------------------------------
  // Primitive column types;
  // This is really LaTeX, but the mechanisms are used behind-the-scenes here, too.
  DefColumnType!("|", sub[_gullet,_args,state] {
    state.current_build_template().unwrap().
      add_between_column(vec![T_CS!("\\vrule"), T_CS!("\\relax")]);
  });
  DefColumnType!("l", sub[_gullet,_args,state] {
    state.current_build_template().unwrap().add_column(Cell {
      after: Some(Tokens!(T_CS!("\\hfil"))), ..Cell::default()});
  });
  DefColumnType!("c", sub[_gullet,_args,state] {
    state.current_build_template().unwrap().add_column(Cell {
      before: Some(Tokens!(T_CS!("\\hfil"))),
      after: Some(Tokens!(T_CS!("\\hfil"))), ..Cell::default()});
  });
  DefColumnType!("r", sub[_gullet,_args,state] {
    let mut template = state.current_build_template().unwrap();
    template.add_column(Cell {
      before: Some(Tokens!(T_CS!("\\hfil"))),
      ..Cell::default()});
  });

  DefColumnType!("p{Dimension}", sub[_gullet,args,state] {
    let width = args.remove(0).expect_dimension();
    state.current_build_template().unwrap().add_column(Cell {
      before: Some(Tokens!(T_CS!("\\vtop"), T_BEGIN!(), T_CS!("\\hbox"), T_BEGIN!())),
      after: Some(Tokens!(T_END!(), T_END!())),
      align: Some(Align::Justify),
      width: Some(width),
      ..Cell::default()});
  });

  DefColumnType!("*{Number}{}", sub[_gullet,args,_state] {
    let n = args.remove(0).expect_number();
    let pattern = args.remove(0).owned_tokens().unwrap();
    let mut tks = Vec::new();
    for _ in 1 ..= n.value_of() {
      tks.extend(pattern.clone().unlist());
    }
    tks
  });

  DefColumnType!("@{}", sub[_gullet,args,state] {
    let filler = args.remove(0).owned_tokens().unwrap();
    state.current_build_template().unwrap().add_between_column(filler.unlist());
  });

  // ----------------------------------------------------------------------
  //  This is where ALL(?) alignments start & finish
  //  This creates the object representing the entire alignment!
  DefConstructor!("\\@start@alignment SkipSpaces", "#alignment",
    reversion => sub[whatsit,_args,state] {
      if let Some(Stored::Digested(alignment)) = whatsit.get_property("alignment").as_deref() {
        if let DigestedData::Alignment(data) = alignment.data() {
          data.borrow().revert(state)
        } else {
          Ok(Tokens!())
        }
      } else {
        Ok(Tokens!())
      }},
    sizer => "#alignment",
    after_digest => sub[stomach,whatsit,state] {
      stomach.bgroup(state);
      if let Some(alignment) = state.lookup_alignment() {
        whatsit.set_property("alignment", Stored::Digested(alignment));
        digest_alignment_body(whatsit, stomach, state)?;
      }
      stomach.egroup(state)?;
    }
  );

  // Seems odd to need both end markers here...
  DefMacro!("\\@finish@alignment", r"\hidden@crcr\@close@alignment");
  DefPrimitive!("\\@close@alignment", None);

  //======================================================================
  // Low-level bits that appear within alignments or \halign

  DefConstructor!("\\cr",   "\n");
  DefConstructor!("\\crcr", "\n");
  // These are useful for reversion of higher-level macros that use alignment
  // internally, but don't use explicit &,\cr in the user markup
  DefConstructor!("\\hidden@cr",    "\n", alias => "");
  DefConstructor!("\\hidden@crcr",  "\n", alias => "");
  DefConstructor!("\\hidden@align", "",   alias => "");

  // Handled directly in alignments, but must be defined as non-macros
  DefPrimitive!("\\noalign", sub[stomach,_args,state] {
      stomach.bgroup(state);
      Error!("unexpected", "\\noalign", stomach, state, "\\noalign cannot be used here");
      Let!(&T_ALIGN!(),          T_RELAX!());
      Let!(&T_CS!("\\noalign"), T_RELAX!());
      Let!(&T_CS!("\\omit"),    T_RELAX!());
      Let!(&T_CS!("\\span"),    T_RELAX!()); });
  DefPrimitive!("\\omit", sub[stomach,_args,state] {
      Error!("unexpected", "\\omit", stomach, state, "\\omit cannot be used here");
      stomach.bgroup(state);
      Let!(&T_ALIGN!(),          T_RELAX!());
      Let!(&T_CS!("\\noalign"), T_RELAX!());
      Let!(&T_CS!("\\omit"),    T_RELAX!());
      Let!(&T_CS!("\\span"),    T_RELAX!()); });
  DefPrimitive!("\\span", sub[stomach,_args,state] {
      stomach.bgroup(state);
      Error!("unexpected", "\\span", stomach, state, "\\span cannot be used here");
      Let!(&T_ALIGN!(),          T_RELAX!());
      Let!(&T_CS!("\\noalign"), T_RELAX!());
      Let!(&T_CS!("\\omit"),    T_RELAX!());
      Let!(&T_CS!("\\span"),    T_RELAX!()); });


  // #######
  // Support for \\[dim] .... TO BE WORKED OUT!
  // NOTE that this does NOT skip spaces before * or []!!!!!
  //  As if: \@alignment@newline OptionalMatch:* [Dimension]
  // Read arguments for \\, namely * and/or [Dimension]
  // BUT optionally do it while skipping spaces (latex style) or not (ams style)
  fn read_newline_args(gullet: &mut Gullet, skipspaces: bool, state: &mut State) -> Result<(bool, Option<Tokens>)> {
    if state.lookup_alignment().is_some() {
      state.local_align_group_count(1000000);
      if skipspaces {
        gullet.skip_spaces(state)?;
      }
      let (mut star, mut optional) = (false,None);
      let mut next_opt = gullet.read_token(state)?;
      if next_opt == Some(T_OTHER!("*")) {
        star = true;
        if skipspaces {
          gullet.skip_spaces(state)?;
        }
        next_opt = gullet.read_token(state)?;
      }
      if next_opt == Some(T_OTHER!("[")) {
        optional = Some(gullet.read_until(&Tokens!(T_OTHER!("]")), state)?);
        next_opt     = None;
      }
      if let Some(next)  = next_opt {
        gullet.unread_one(next);
      }
      state.expire_align_group_count();
      Ok((star, optional))
    } else {
      Err("read_newline_args should only be called with a proper 'Alignment' active in State".into())
    }
  }

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
  DefMacro!("\\@alignment@newline", sub[gullet,_a,state] {
    let (star, optional) = read_newline_args(gullet, true, state)?;
    let mut tokens = vec![T_CS!("\\hidden@cr"), T_BEGIN!()];
    if let Some(opt_tks) = optional {
      tokens.push(T_CS!("\\@alignment@newline@markertall"));
      tokens.push(T_BEGIN!());
      tokens.extend(opt_tks.unlist().into_iter());
      tokens.push(T_END!());
    } else {
      tokens.push(T_CS!("\\@alignment@newline@marker"));
      tokens.push(T_END!());
    }
    Tokens::new(tokens)
  });

  // However, the above will skip spaces --AND a newline! -- looking for [],
  // which is kinda weird in math, since there may be a reasonable math [ in the 1st column!
  // AMS kindly avoids that, by using a special version of \\
  DefMacro!("\\@alignment@newline@noskip", sub[gullet,_a,state] {
    let (star, optional) = read_newline_args(gullet, false, state)?;
    let mut tokens = vec![T_CS!("\\hidden@cr"), T_BEGIN!()];
    if let Some(opt_tks) = optional {
      tokens.push(T_CS!("\\@alignment@newline@markertall"));
      tokens.push(T_BEGIN!());
      tokens.extend(opt_tks.unlist().into_iter());
      tokens.push(T_END!());
    } else {
      tokens.push(T_CS!("\\@alignment@newline@marker"));
      tokens.push(T_END!());
    }
  });

  // These are the markers that produce \\ in the reversion,
  // and (eventually will) add vertical space to the row!
  DefConstructor!("\\@alignment@newline@marker", "",
    reversion => Tokens!(T_CS!("\\\\"), T_CR!()));
  // AND add the spacing to the alignment!!!
  DefConstructor!("\\@alignment@newline@markertall {Dimension}", "",
    after_digest => sub[_stomach,whatsit,state] {
    if let Some(alignment) = state.lookup_alignment() {
      let mut alignment_mut = alignment.alignment_cell().unwrap().borrow_mut();
      let current_row = alignment_mut.current_row_mut().unwrap();
      let padding = if let Some(arg) = whatsit.get_arg(1) {
        if let DigestedData::RegisterValue(RegisterValue::Dimension(v)) = arg.data() {
          *v
        } else { Dimension::new(0) }
      }  else { Dimension::new(0) };
      current_row.set_padding(padding);
    }},
    reversion => sub[whatsit,_args,state] {
      let reverted = whatsit.revert(state)?;
      Ok(Tokens!(T_CS!("\\\\"), T_OTHER!("["), reverted, T_OTHER!("]"), T_CR!()))
    });

  DefMacro!("\\tabularnewline", "\\cr"); // ???


  //======================================================================
  // Various decorations within alignments, rules, headers, etc
  // Like \noalign, takes an arg; handled within alignment processing.
  // But doesn't create a pseudo-row (??? Or does it?; is it still needed?)
  DefConstructor!("\\hidden@noalign{}", "#1",
    reversion  => "",
    properties =>  sub[_stomach, args, _state] {
      // Sometimes, we"re smuggling stuff that needs to be carried into the XML.
      let mut props = stored_map!("alignmentSkippable" => true);
      if let Some(preserve) = args.iter().find(|v_opt| if let Some(ref v) = v_opt {
        v.get_property("alignmentPreserve").is_some()
      } else { false }) {
        props.insert(String::from("alignmentPreserve"), preserve.as_ref().unwrap().into());
      }
      Ok(props) });

  DefMacro!("\\hline", "\\noalign{\\@@alignment@hline}");
  DefConstructor!("\\@@alignment@hline", "",
    after_digest => sub[_stomach,_whatsit,state] {
      if let Some(alignment_stored) = state.lookup_alignment() {
        alignment_stored.alignment_cell().unwrap().borrow_mut()
          .add_line("t", Vec::new());
      }
    },
    properties =>  sub[_stomach, _args, _state] { Ok(stored_map!("isHorizontalRule" => true))},
    sizer      => 0, alias => "\\hline");

  DefMacro!("\\@tabular@begin@heading", sub[_gullet,_args,state] {
    if let Some(alignment_stored) = state.lookup_alignment() {
      alignment_stored.alignment_cell().unwrap().borrow_mut()
        .set_in_tabular_head();
    }});
  DefMacro!("\\@tabular@end@heading", sub[_gullet,_args,state] {
    if let Some(alignment_stored) = state.lookup_alignment() {
      alignment_stored.alignment_cell().unwrap().borrow_mut()
        .unset_in_tabular_head();
    }});


  //======================================================================
  // Math mode in alignment
  // Special forms for $ appearing within alignments.
  // Note that $ within a math alignment (eg array environment),
  // switches to text mode! There's no $$ for display math.
  //
  // This is the "normal" case: $ appearing with an alignment that is in text mode.
  // It's just like regular $, except it doesn't look for $$ (no display math).
  DefPrimitive!("\\@dollar@in@textmode", sub [stomach, (), state] {
    let mathcs = if state.lookup_bool("IN_MATH") { T_CS!("\\@@ENDINLINEMATH") }
      else {T_CS!("\\@@BEGININLINEMATH") };
    stomach.invoke_token(&mathcs, state)
  });

  DefMacro!("\\@row@before", None);
  DefMacro!("\\@row@after", None);
  DefMacro!("\\@column@before", None);
  DefMacro!("\\@column@after", None);


  //======================================================================
  // Multicolumn support
  // DefMacro('\multispan{Number}', sub {
  //     my ($gullet, $span) = @_;
  //     $span = $span->valueOf;
  //     (T_CS('\omit'), map { (T_CS('\span'), T_CS('\omit')) } 1 .. $span - 1); });

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

  // DefMacro('\@multicolumn {Number}  AlignmentTemplate {}', sub {
  //     my ($gullet, $span, $template, $tokens) = @_;
  //     my $column = $template->column(1);
  //     $span = $span->valueOf;
  //     # First part, like \multispan
  //     (T_CS('\omit'), (map { (T_CS('\span'), T_CS('\omit')) } 1 .. $span - 1),
  //       # Next part, just put the template in-line, since it's only used once.
  //       ($column ? beforeCellUnlist($$column{before}) : ()),
  //       $tokens->unlist,
  //       ($column ? afterCellUnlist($$column{after}) : ())); });

  DefConditional!("\\if@in@alignment", sub [gullet, (), state] { state.lookup_alignment().is_some() });

  // DefPrimitive('\@alignment@bindings AlignmentTemplate []', sub {
  //     my ($stomach, $template, $mode) = @_;
  //     alignmentBindings($template, $mode); });

  // This removes trailing whitespace from the current digested list.
  // It is useful as the 1st thing in the rhs template of things like {tabular}.
  // But note that \halign does NOT remove this trailing space!
  DefPrimitive!("\\@@eat@space", sub[stomach,(),_state] {
    let mut save = Vec::new();
    while let Some(tbox) = stomach.box_list.last() {
      if tbox.get_property_bool("alignmentSkippable")
        || tbox.get_property_bool("isFill") {
        save.push(stomach.box_list.pop().unwrap());
      } else if tbox.is_empty() {
        stomach.box_list.pop().unwrap();
      } else { break; }
    }
    stomach.box_list.append(&mut save);
  });


});


//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// And the general alignment processing.
// If the Template is appropriately constructed, either by \halign or various \begin{tabular}
// the body of the alignment is processed the same way.

pub fn alignment_bindings(template: Template, mode: String, properties: HashMap<String,Stored>, gullet: &mut Gullet, state: &mut State) {
  let mode = if mode.is_empty() { state.lookup_string("MODE") } else { mode };
  let is_math    = mode.ends_with("math");
  let (container,rowtype,coltype) = if is_math {
    ("ltx:XMArray","ltx:XMRow","ltx:XMCell")
  }  else {
    ("ltx:tabular","ltx:tr","ltx:td")
  };
let alignment = Alignment::new(AlignmentConfig {
    template: Some(template),
    open_container: Rc::new(
      |document,props,state| document.open_element(container, Some(props), None, state).map(Option::Some)),
    close_container: Rc::new(
      |document,state| document.close_element(container, state) ),
    open_row       : Rc::new(
      |document,props,state| document.open_element(rowtype, Some(props), None, state).and(Ok(()))),
    close_row      : Rc::new(
      |document,state| document.close_element(rowtype, state) ),
    open_column    : Rc::new(
      |document,props,state| document.open_element(coltype, Some(props), None, state).map(Option::Some)),
    close_column   : Rc::new(
      |document,state| document.close_element(coltype, state)),
    is_math,
    properties
  });
  state.assign_alignment(alignment, None);
  // Debug("Halign $alignment: New " . $template->show) if $LaTeXML::DEBUG{halign};
  state.let_i(&T_MATH!(), if is_math { T_CS!("\\@dollar@in@mathmode") } else {T_CS!("\\@dollar@in@textmode")}, None, gullet);
}

pub fn digest_alignment_body(whatsit: &mut Whatsit, stomach: &mut Stomach, state:&mut State) -> Result<()> {
  // Now read & digest the body.
  // Note that the body MUST end with a \cr, and that we've made Special Arrangments
  // with \alignment@cr to recognize the end of the \halign
  state.local_align_group_count(0);
  let alignment_stored = if let Some(alignment) = state.lookup_alignment() {
    alignment
  } else {
    Error!("missing", "alignment", stomach, state, "There is no open alignment structure here");
    return Ok(());
  };
  state.local_reading_alignment(&alignment_stored);
  whatsit.set_property("alignment", Stored::Digested(alignment_stored.clone()));

  // Debug!("Halign {}: BODY Processing...",alignment) if $LaTeXML::DEBUG{halign};
  let mut lastwascr  = false;
  let mut reversion: Vec<Token>  = Vec::new();
  let mut creversion: Vec<Token> = Vec::new();
  let alignment_cell = alignment_stored.alignment_cell().unwrap();
  loop {
    let (cell_opt, next, vtype, hidden) =
      digest_alignment_column(alignment_cell, lastwascr, stomach, state)?;
//     Debug("Halign $alignment: BODY got CELL"
//         . "[" . $alignment->currentRowNumber . "," . $alignment->currentColumnNumber . "]"
//         . ToString($cell) . " ended at " . Stringify($next)) if $LaTeXML::DEBUG{halign};

    if let Some(cell) = cell_opt {
      reversion.extend(trim_column_template(alignment_cell.borrow_mut(), p_revert(cell.clone(), state)?)
        .unlist().into_iter());
      creversion.extend(trim_column_template(alignment_cell.borrow_mut(), c_revert(cell.clone(), state)?)
        .unlist().into_iter());
      extract_alignment_column(alignment_cell.borrow_mut(), cell, state);
    } else {
      // Debug("Halign $alignment: BODY DONE!") if $LaTeXML::DEBUG{halign};
      break;
    }
    lastwascr = false;
    if (vtype.is_none() || vtype.as_ref().unwrap().is_empty()) && (next.is_none()
      || next == Some(T_END!())                         // End of alignment
      || next == Some(T_CS!("\\@close@alignment"))) {   // End of alignment
        alignment_cell.borrow_mut().end_row(stomach, state)?;
        break;
    } else if vtype.as_deref() == Some("align") {
      alignment_cell.borrow_mut().end_column(stomach, state)?;
      if !hidden {
        reversion.push(next.clone().unwrap());              // and record the &
        creversion.push(next.unwrap());                     // and record the &
      }
    } else if vtype.as_deref() == Some("insert") {
      alignment_cell.borrow_mut().end_column(stomach, state)?;
    } else if vtype.as_deref() == Some("cr") || vtype.as_deref() == Some("crcr") {
      alignment_cell.borrow_mut().end_row(stomach, state)?;
      if !hidden {
        reversion.push(next.clone().unwrap());
        creversion.push(next.unwrap());
      } else if vtype.as_deref() == Some("cr") {
        let arg_toks = stomach.get_gullet_mut().read_arg(state)?;
        let arg = stomach.digest(arg_toks, state)?;
        reversion.extend(p_revert(arg.clone(), state)?.unlist().into_iter());
        creversion.extend(c_revert(arg, state)?.unlist().into_iter());
      } else if vtype.as_deref() == Some("crcr") { }
      lastwascr = true;
    } else if let Some(next_tok) = next {    // Note, in case next is \crcr
      Error!("unexpected", next_tok, stomach, state, s!("Column ended with {next_tok}"));
    }
  }
  alignment_cell.borrow_mut().end_row(stomach, state)?;
  alignment_cell.borrow_mut().set_reversion(Tokens!(reversion));
  alignment_cell.borrow_mut().set_content_reversion(Tokens!(creversion));
//   Debug("Halign $alignment: BODY DONE!\n"
//       . "=> " . join(',', map { Stringify($_); } @reversion)) if $LaTeXML::DEBUG{halign};
  state.expire_align_group_count();
  state.expire_reading_alignment();
  Ok(())
}

// Read & digest an alignment column's data,
// accommodating the current template and any special cs's
// Returns the column's digested boxes, the ending token, and it's alignment type.
type DigestedColumn = Result<(Option<Digested>, Option<Token>, Option<String>, bool)>;
pub fn digest_alignment_column(alignment: &RefCell<Alignment>, lastwascr: bool, stomach:&mut Stomach, state: &mut State) -> DigestedColumn {
  stomach.new_local_box_list();
  let ismath = state.lookup_bool("IN_MATH");
  // Scan for leading \omit, skipping over (& saving) \hline.
  //   Debug("Halign $alignment: COLUMN starting scan "
  //       . "(" . ($ismath ? " math" : " text") . ")") if $LaTeXML::DEBUG{halign};
  let mut last_token: Option<Token>;
  let spanning = false;// TODO: Revise spanning use
  loop {   // Outer loop; collects 1 column (possibly multiple spans) return from within!
    // Scan till we get something NOT \omit, \noalign
    last_token = stomach.get_gullet_mut()
      .read_x_token(Some(false),false, state)?;
    if last_token.is_none() { break; }
    let token = last_token.as_ref().unwrap();
    dbg!(token);
    if *token == T_SPACE!()   // Skip leading space.
      || *token == T_CS!("\\par")  // Skip or blank line(?)
      || (lastwascr &&             // Or \crcr following a \cr
         (*token == T_CS!("\\crcr") || *token == T_CS!("\\hidden@crcr"))) {
    } else if *token == T_CS!("\\omit") { // \omit removes template for this column.
//         Debug("Halign $alignment: OMIT at " . Stringify($token)) if $LaTeXML::DEBUG{halign};
      if !alignment.borrow().is_in_row() {
        alignment.borrow_mut().start_row(false, stomach,state)?;
      }
      alignment.borrow_mut().omit_next_column();
    } else if *token == T_CS!("\\noalign") {    // \puts something in vertical list
      // Debug("Halign $alignment: noalign at " . Stringify($token)) if $LaTeXML::DEBUG{halign};
      if alignment.borrow().is_in_row() {
        alignment.borrow_mut().end_row(stomach,state)?;
      }
      alignment.borrow_mut().start_column(true, stomach,state)?;
      alignment.borrow_mut().last_column();
      let next_arg = stomach.get_gullet_mut().read_arg(state)?;
      let r = stomach.digest(next_arg, state)?;
      alignment.borrow_mut().end_row(stomach,state)?;
      stomach.expire_local_box_list();
      return Ok((Some(r), Some(T_CS!("\\cr")), some!("cr"), false)); // Pretend this is a whole row???
    } else if *token == T_CS!("\\hidden@noalign") { // \puts something in vertical list
//         Debug("Halign $alignment: COLUMN invisible noalign") if $LaTeXML::DEBUG{halign};
      let invoked =  stomach.invoke_token(token, state)?;
      stomach.box_list.extend(invoked);
    } else {
      break;
    }
  }
//     Debug("Halign $alignment: COLUMN end scan at " . Stringify($token)) if $LaTeXML::DEBUG{halign};
  if last_token.is_none() || last_token == Some(T_END!()) || last_token == Some(T_CS!("\\@close@alignment")) {
    stomach.expire_local_box_list();
    return Ok((None, last_token, None, false));
  }
  // Next column, unless spanning (then combine columns)
  if spanning {
    // spanning = false;
    alignment.borrow_mut().next_column();
  } else {
    alignment.borrow_mut().start_column(false, stomach, state)?;
  }
  // Push before template,  Marker and put the token back
  // Debug("Halign $alignment: COLUMN preload at "
  //     . Stringify(Tokens($alignment->getColumnBefore, T_MARKER('before-column'), $token)))
  //   if $LaTeXML::DEBUG{halign};
  let to_unread = Tokens!(
    alignment.borrow_mut().get_column_before(), T_MARKER!("before-column"), last_token.unwrap());
  stomach.get_gullet_mut().unread(to_unread);
  while let Some(token) = stomach.get_gullet_mut().read_x_token(Some(false), false, state)? {
    if let Some((_atoken, vtype, hidden)) = gullet::is_column_end(&token, state) {
      if vtype == "span" { // next column, but continue accumulating
        // Debug("Halign $alignment: COLUMN span") if $LaTeXML::DEBUG{halign};
        // spanning = true;
        break;
      } else {
        // Debug("Halign $alignment: COLUMN ended with " . Stringify($token) . "\n"
        //     . "  => " . ToString(List(@LaTeXML::LIST))) if $LaTeXML::DEBUG{halign};
        let current_list = stomach.expire_local_box_list();
        let mut out_list = List::new(current_list, state);
        out_list.mode = Some(if ismath { TexMode::Math } else { TexMode::Text });
        return Ok((Some(out_list.into()),Some(token),Some(String::from(vtype)),hidden));
      }
    } else if token == T_CS!("\\hidden@noalign") { //  \puts something in vertical list
      // Debug("Halign $alignment: COLUMN invisible noalign") if $LaTeXML::DEBUG{halign};
      let invoked = stomach.invoke_token(&token, state)?;
      stomach.box_list.extend(invoked.into_iter());
    } else {  // Else, we're getting some actual content for the column
  //     // Debug!("Halign $alignment: COLUMN invoking " . Stringify($token)) if $LaTeXML::DEBUG{halign};
      let invoked = stomach.invoke_token(&token, state)?;
      stomach.box_list.extend(invoked.into_iter());
  // //     Debug("Halign $alignment: COLUMN " . Stringify($token) . " ==> " . Stringify(List(@LaTeXML::LIST)))
  // //       if $LaTeXML::DEBUG{halign};
    }
  }
  stomach.expire_local_box_list();
  Ok((None, None, None, false))
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
  let mut pre  = alignment.get_column_before().unlist();
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
pub fn extract_alignment_column(mut alignment: RefMut<Alignment>, in_box: Digested, state: &mut State) -> Digested {
  let mut boxes = VecDeque::new();
  boxes.push_back(in_box);
  // let is_math  = in_box.is_math();
  let is_math  = state.lookup_bool("IN_MATH");
  //Note: $n0,$n1 is a VERY round-about way of tracking the column spanning!
  let n0      = state.lookup_int("alignmentStartColumn") as usize + 1;
  let n1      = alignment.current_column_number();
  let colspec = alignment.get_column(n0).unwrap();
  let mut align   = colspec.align.unwrap_or(Align::Left);
  let mut border  = String::new();
  // Peel off any boxes from both sides until we get the "meat" of the column.
  // from this we can establish borders, alignment and emptiness.
  // But we, of course, immediately put them back...
  let mut saveleft  = VecDeque::new();
  let mut saveright = VecDeque::new();
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
      },
      item if front_box.get_property("isHorizontalRule").is_some()
                        || front_box.get_property("alignmentSkippable").is_some()
                        || front_box.get_property("isSpace").is_some()
                        || matches!(item,DigestedData::Comment(_))
                        || front_box.is_empty() => {
              saveleft.push_front(front_box) }
      _ => {
        // put the box back, and terminate left side loop.
        boxes.push_front(front_box);
        break;
      }
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
        if align == Align::Right { align = Align::Center };
        break;
      },
      _ if last_box.get_property("isVerticalRule").is_some() => {
        border.push('r');
      }
      item if last_box.get_property("isHorizontalRule").is_some()
                        || last_box.get_property("alignmentSkippable").is_some()
                        || last_box.get_property("isSpace").is_some()
                        || matches!(item,DigestedData::Comment(_))
                        || last_box.is_empty() => {
        saveright.push_front(last_box);
      },
      _ => {
        // put the box back, and terminate right side loop.
        boxes.push_back(last_box);
        break;
      }
    }
  }
  if align != Align::Justify {
    colspec.width = None;
  }
  // Replacing boxes with the fil padding & vertical rules stripped off
  let mut final_boxes = Vec::from(saveleft);
  final_boxes.extend(boxes.into_iter());
  final_boxes.extend(saveright.into_iter());
  let mut boxes_list = List::new(final_boxes, state);
  boxes_list.mode = Some( if is_math {TexMode::Math} else {TexMode::Text});
  let digested_out = Digested::from(boxes_list);
  // record relevant info in the Alignment.
  colspec.align   = Some(align);
  border = s!("{}{}",colspec.border, border);
  colspec.border  = border;
  colspec.boxes   = Some(digested_out.clone());
  colspec.colspan = Some(n1 - n0 + 1);
//   if ($$alignment{in_tabular_head} || $$alignment{in_tabular_foot}) {
//     $$colspec{thead}{column} = 1; }
//   for (my $i = $n0 + 1 ; $i <= $n1 ; $i++) {
//     my $c = $alignment->getColumn($i);
//     $$c{skipped} = 1 if $c; }
//   Debug("Halign $alignment: INSTALL column " . join(',', map { $_ . "=" . ToString($$colspec{$_}); } sort keys %$colspec)) if $LaTeXML::DEBUG{halign};
  digested_out
}
