use crate::package::*;
use rtx_core::common::object::Object;
use rtx_core::alignment::read_alignment_template;
use rtx_core::alignment::template::Template;
use std::cell::RefCell;
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
    state.current_build_template().unwrap().add_column(Column {
      after: Some(Tokens!(T_CS!("\\hfil"))), ..Column::default()});
  });
  DefColumnType!("c", sub[_gullet,_args,state] {
    state.current_build_template().unwrap().add_column(Column {
      before: Some(Tokens!(T_CS!("\\hfil"))),
      after: Some(Tokens!(T_CS!("\\hfil"))), ..Column::default()});
  });
  DefColumnType!("r", sub[_gullet,_args,state] {
    state.current_build_template().unwrap().add_column(Column {
      before: Some(Tokens!(T_CS!("\\hfil"))),
      ..Column::default()});
  });

  DefColumnType!("p{Dimension}", sub[_gullet,args,state] {
    let width = args.remove(0).expect_dimension();
    state.current_build_template().unwrap().add_column(Column {
      before: Some(Tokens!(T_CS!("\\vtop"), T_BEGIN!(), T_CS!("\\hbox"), T_BEGIN!())),
      after: Some(Tokens!(T_END!(), T_END!())),
      align: Some(Align::Justify),
      width: Some(width),
      ..Column::default()});
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
  DefConstructor!("\\@start@alignment SkipSpaces",
    "#alignment",
    reversion => sub[whatsit,_args,state] {
      if let Some(Stored::Alignment(alignment)) = whatsit.get_property("alignment").as_deref() {
        alignment.borrow().revert(state)
      } else {
        Ok(Tokens!())
      }},
    sizer     => "#alignment",
    after_digest => sub[stomach,whatsit,state] {
      stomach.bgroup(state);
      digest_alignment_body(whatsit, stomach, state)?;
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

});


//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// And the general alignment processing.
// If the Template is appropriately constructed, either by \halign or various \begin{tabular}
// the body of the alignment is processed the same way.

pub fn alignment_bindings(template: Template, mode: String, properties: HashMap<String,Stored>, gullet: &mut Gullet, state: &mut State) {
  let mode = if mode.is_empty() { state.lookup_string("MODE") } else { mode };
  let ismath    = mode.ends_with("math");
  let (container,rowtype,coltype) = if ismath {
    ("ltx:XMArray","ltx:XMRow","ltx:XMCell")
  }  else {
    ("ltx:tabular","ltx:tr","ltx:td")
  };
  let alignment = Alignment::new(AlignmentConfig {
    template: Some(template),
    open_container: Some(Rc::new(
      |document,props,state| document.open_element(container, Some(props), None, state).and(Ok(())))),
    close_container: Some(Rc::new(
      |document,state| document.close_element(container, state).and(Ok(())) )),
    open_row       : Some(Rc::new(
      |document,props,state| document.open_element(rowtype, Some(props), None, state).and(Ok(())))),
    close_row      : Some(Rc::new(
      |document,state| document.close_element(rowtype, state).and(Ok(())) )),
    open_column    : Some(Rc::new(
      |document,props,state| document.open_element(coltype, Some(props), None, state).and(Ok(())))),
    close_column   : Some(Rc::new(
      |document,state| document.close_element(coltype, state).and(Ok(())))),
    is_math        : ismath,
    // properties
    ..AlignmentConfig::default()
  });
  state.assign_value("Alignment", alignment, None);
  // Debug("Halign $alignment: New " . $template->show) if $LaTeXML::DEBUG{halign};
  state.let_i(&T_MATH!(), if ismath { T_CS!("\\@dollar@in@mathmode") } else {T_CS!("\\@dollar@in@textmode")}, None, gullet);
}

pub fn digest_alignment_body(whatsit: &mut Whatsit, stomach: &mut Stomach, state:&mut State) -> Result<()> {
  // Now read & digest the body.
  // Note that the body MUST end with a \cr, and that we've made Special Arrangments
  // with \alignment@cr to recognize the end of the \halign
  let alignment = if let Some(Stored::Alignment(alignment)) = state.lookup_value("Alignment") {
    Rc::clone(alignment)
  } else {
    Error!("missing", "alignment", stomach, state, "There is no open alignment structure here");
    return Ok(());
  };
  let gullet = stomach.get_gullet_mut();
  state.set_align_group_count(0);
  state.set_reading_alignment(&alignment);
  whatsit.set_property("alignment", Stored::Alignment(Rc::clone(&alignment)));
  // THIS IS NOT ENCOURAGED! AVOID THE TECHNIQUE.
  // clone the current whatsit, and set it as the "Alignment" body.
  //
  // TODO: is there a way to avoid the clone? Does this matter in practice?
  // Originally, the same whatsit and alignment object had a w<-->a circular pointing scheme.
  // Now we have a single direction: w --> #alignment(alignment) --> body(w_clone)
  let inner_w = Digested::from(whatsit.clone());
  alignment.borrow_mut().set_body(vec![inner_w], state);

  // Debug!("Halign {}: BODY Processing...",alignment) if $LaTeXML::DEBUG{halign};
  let mut lastwascr  = false;
  let mut reversion  = Vec::new();
  let mut creversion = Vec::new();
  loop {
    let (cell_opt, next, vtype, hidden) = digest_alignment_column(Rc::clone(&alignment), lastwascr, stomach, state);
//     Debug("Halign $alignment: BODY got CELL"
//         . "[" . $alignment->currentRowNumber . "," . $alignment->currentColumnNumber . "]"
//         . ToString($cell) . " ended at " . Stringify($next)) if $LaTeXML::DEBUG{halign};
    if cell_opt.is_none() {
      // Debug("Halign $alignment: BODY DONE!") if $LaTeXML::DEBUG{halign};
      break;
    }
    if let Some(cell) = cell_opt {
      reversion.push(trim_column_template(Rc::clone(&alignment), p_revert(cell.clone(), state)?));
      creversion.push(trim_column_template(Rc::clone(&alignment), c_revert(cell.clone(), state)?));
      extract_alignment_column(Rc::clone(&alignment), vec![cell]);
    }
    lastwascr = false;
//     if (!$type && (!$next
//         || Equals($next, T_END)                           // End of alignment
//         || Equals($next, T_CS('\@close@alignment')))) {   // End of alignment
//       $alignment->endRow();
//       last; }
//     elsif ($type eq 'align') {
//       $alignment->endColumn();
//       if (!$hidden) {
//         push(@reversion,  $next);                         // and record the &
//         push(@creversion, $next); } }                     // and record the &
//     elsif ($type eq 'insert') {
//       $alignment->endColumn(); }
//     elsif (($type eq 'cr') || ($type eq 'crcr')) {
//       $alignment->endRow();
//       if (!$hidden) {
//         push(@reversion,  $next);
//         push(@creversion, $next); }
//       elsif ($type eq 'cr') {
//         my $arg = $stomach->digest($gullet->readArg());
//         push(@reversion,  pRevert($arg));
//         push(@creversion, cRevert($arg)); }
//       elsif ($type eq 'crcr') { }
//       $lastwascr = 1; }   // Note, in case next is \crcr
//     elsif ($next) {
//       Error('unexpected', $next, $stomach, "Column ended with " . Stringify($next)); }
  }
//   $alignment->endRow();
//   $alignment->setReversion(Tokens(@reversion));
//   $alignment->setContentReversion(Tokens(@creversion));
//   Debug("Halign $alignment: BODY DONE!\n"
//       . "=> " . join(',', map { Stringify($_); } @reversion)) if $LaTeXML::DEBUG{halign};
  state.expire_align_group_count();
  state.expire_reading_alignment();
  Ok(())
}

// Read & digest an alignment column's data,
// accommodating the current template and any special cs's
// Returns the column's digested boxes, the ending token, and it's alignment type.
pub fn digest_alignment_column(alignment: Rc<RefCell<Alignment>>, lastwascr: bool, stomach:&mut Stomach, state: &mut State) -> (Option<Digested>, Option<usize>, Option<usize>, Option<bool>) {
  let gullet = stomach.get_gullet_mut();
  let ismath = state.lookup_bool("IN_MATH");
//   local @LaTeXML::LIST = ();
//  // Scan for leading \omit, skipping over (& saving) \hline.
//   Debug("Halign $alignment: COLUMN starting scan "
//       . "(" . ($ismath ? " math" : " text") . ")") if $LaTeXML::DEBUG{halign};
//   my $token;
//   my $spanning = 0;
//   while (1) {   // Outer loop; collects 1 column (possibly multiple spans) return from within!
//    //# Scan till we get something NOT \omit, \noalign
//     while ($token = $gullet->readXToken(0)) {
//       if ($token->equals(T_SPACE)   // Skip leading space.
//         || $token->equals(T_CS('\par'))    # Skip or blank line(?)
//         || ($lastwascr &&                  # Or \crcr following a \cr
//           (Equals($token, T_CS('\crcr')) || Equals($token, T_CS('\hidden@crcr'))))) {
//       }
//       elsif (Equals($token, T_CS('\omit'))) {    # \omit removes template for this column.
//         Debug("Halign $alignment: OMIT at " . Stringify($token)) if $LaTeXML::DEBUG{halign};
//         $alignment->startRow() unless $$alignment{in_row};
//         $alignment->omitNextColumn; }
//       elsif (Equals($token, T_CS('\noalign'))) {    # \puts something in vertical list
//         Debug("Halign $alignment: noalign at " . Stringify($token)) if $LaTeXML::DEBUG{halign};
//         $alignment->endRow()                                        if $$alignment{in_row};
//         $alignment->startColumn(1);
//         $alignment->lastColumn;
//         my $r = $stomach->digest($gullet->readArg);
//         $alignment->endRow();
//         return ($r, T_CS('\cr'), 'cr'), undef; }    # Pretend this is a whole row???
//       elsif (Equals($token, T_CS('\hidden@noalign'))) {    # \puts something in vertical list
//         Debug("Halign $alignment: COLUMN invisible noalign") if $LaTeXML::DEBUG{halign};
//         push(@LaTeXML::LIST, $stomach->invokeToken($token)); }
//       else {
//         last; } }
//     Debug("Halign $alignment: COLUMN end scan at " . Stringify($token)) if $LaTeXML::DEBUG{halign};
//     if (!$token || Equals($token, T_END) || Equals($token, T_CS('\@close@alignment'))) {
//       return (undef, $token, undef, undef); }
//     # Next column, unless spanning (then combine columns)
//     if ($spanning) {
//       $spanning = 0;
//       $alignment->nextColumn; }
//     else {
//       $alignment->startColumn(); }
//     # Push before template,  Marker and put the token back
//     Debug("Halign $alignment: COLUMN preload at "
//         . Stringify(Tokens($alignment->getColumnBefore, T_MARKER('before-column'), $token)))
//       if $LaTeXML::DEBUG{halign};
//     $gullet->unread($alignment->getColumnBefore, T_MARKER('before-column'), $token);
//     while ($token = $gullet->readXToken(0)) {
//       my ($atoken, $type, $hidden) = $gullet->isColumnEnd($token);
//       if ($atoken) {
//         if ($type eq 'span') {    # next column, but continue accumulating
//           Debug("Halign $alignment: COLUMN span") if $LaTeXML::DEBUG{halign};
//           $spanning = 1;
//           last; }
//         else {
//           Debug("Halign $alignment: COLUMN ended with " . Stringify($token) . "\n"
//               . "  => " . ToString(List(@LaTeXML::LIST))) if $LaTeXML::DEBUG{halign};
//           return (List(@LaTeXML::LIST, mode => ($ismath ? 'math' : 'text')),
//             $token, $type, $hidden); } }
//       elsif (Equals($token, T_CS('\hidden@noalign'))) {    # \puts something in vertical list
//         Debug("Halign $alignment: COLUMN invisible noalign") if $LaTeXML::DEBUG{halign};
//         push(@LaTeXML::LIST, $stomach->invokeToken($token)); }
//       else {    # Else, we're getting some actual content for the column
//         Debug("Halign $alignment: COLUMN invoking " . Stringify($token)) if $LaTeXML::DEBUG{halign};
//         push(@LaTeXML::LIST, $stomach->invokeToken($token));
//         Debug("Halign $alignment: COLUMN " . Stringify($token) . " ==> " . Stringify(List(@LaTeXML::LIST)))
//           if $LaTeXML::DEBUG{halign};
//   } } }
  (None, None, None, None)
}

// This attempts to trim off the column template parts from contents of the full column,
// leaving only the author supplied part for a sensible reversion.
// It's not nearly clever enough, given that macros can be in the template,
// but works surprisingly well so far.
// A better alternative might be based on sneaking some Marker tokens/boxes through
// but they would likely interfere with the macros tehmselves.
pub fn trim_column_template(alignment: Rc<RefCell<Alignment>>, tokens: Tokens) -> Tokens {
//   return Tokens(@tokens) if $alignment->currentRow->{pseudorow};
//   my @pre  = $alignment->getColumnBefore->unlist;
//   my @post = $alignment->getColumnAfter->unlist;
//   Debug("Halign $alignment: COLUMN Compare:\n"
//       . "  Column: " . ToString(Tokens(@tokens)) . "\n"
//       . "  Before: " . ToString(Tokens(@pre)) . "\n"
//       . "  After : " . ToString(Tokens(@post)) . "\n") if $LaTeXML::DEBUG{halign};
//   while (scalar(@pre) && scalar(@tokens)) {
//     my $t = shift(@pre);
//     if ($t->equals($tokens[0])) {
//       shift(@tokens); } }
//   while (scalar(@post) && scalar(@tokens)) {
//     my $t = pop(@post);
//     if ($t->equals($tokens[-1])) {
//       pop(@tokens); } }
//   Debug("  Trimmed: " . ToString(Tokens(@tokens))) if $LaTeXML::DEBUG{halign};
  tokens
}

// Given the boxes for an alignment cell,
// extract & remove the various fills and rules from the ends to annotate the cell structure
pub fn extract_alignment_column(alignment: Rc<RefCell<Alignment>>, boxes: Vec<Digested>) -> Vec<Digested> {
// //Note: $n0,$n1 is a VERY round-about way of tracking the column spanning!
//   my $ismath  = $STATE->lookupValue('IN_MATH');
//   my $n0      = (LookupValue('alignmentStartColumn') || 0) + 1;
//   my $n1      = $alignment->currentColumnNumber;
//   my $colspec = $alignment->getColumn($n0);
//   my $align   = $$colspec{align} || 'left';
//   my $border  = '';
//   # Peel off any boxes from both sides until we get the "meat" of the column.
//   # from this we can establish borders, alignment and emptiness.
//   # But we, of course, immediately put them back...
//   my @boxes     = $boxes->unlist;
//   my @saveleft  = ();
//   my @saveright = ();
//   while (@boxes) {
//     if (ref $boxes[0] eq 'LaTeXML::Core::List') {
//       unshift(@boxes, shift(@boxes)->unlist); }
//     elsif ($boxes[0]->getProperty('isFill')) {
//       $align = 'right';
//       shift(@boxes);
//       last; }
//     elsif ($boxes[0]->getProperty('isVerticalRule')) {
//       $border .= 'l';
//       shift(@boxes); }
//     elsif ($boxes[0]->getProperty('isHorizontalRule')
//       || $boxes[0]->getProperty('alignmentSkippable')
//       || (ref $boxes[0] eq 'LaTeXML::Core::Comment')
//       || $boxes[0]->getProperty('isSpace')
//       || IsEmpty($boxes[0])) {
//       push(@saveleft, shift(@boxes)); }
//     else {
//       last; } }
//   while (@boxes) {
//     if (ref $boxes[-1] eq 'LaTeXML::Core::List') {
//       push(@boxes, pop(@boxes)->unlist); }
//     elsif ($boxes[-1]->getProperty('isFill')) {
//       if ($align eq 'right') { $align = 'center'; }
//       pop(@boxes);
//       last; }
//     elsif ($boxes[-1]->getProperty('isVerticalRule')) {
//       $border .= 'r';
//       pop(@boxes); }
//     elsif ($boxes[-1]->getProperty('isHorizontalRule')
//       || $boxes[-1]->getProperty('alignmentSkippable')
//       || (ref $boxes[-1] eq 'LaTeXML::Core::Comment')
//       || $boxes[-1]->getProperty('isSpace')
//       || IsEmpty($boxes[-1])) {
//       unshift(@saveright, pop(@boxes)); }
//     else {
//       last; } }
//   delete $$colspec{width} unless $align eq 'justify';
//   # Replacing boxes with the fil padding & vertical rules stripped off
//   @boxes = (@saveleft, @boxes, @saveright);
//   $boxes = List(@boxes, mode => ($boxes->isMath ? 'math' : 'text'));
//   # record relevant info in the Alignment.
//   $$colspec{align}   = $align;
//   $$colspec{border}  = $border = ($$colspec{border} || '') . $border;
//   $$colspec{boxes}   = $boxes;
//   $$colspec{colspan} = $n1 - $n0 + 1;
//   if ($$alignment{in_tabular_head} || $$alignment{in_tabular_foot}) {
//     $$colspec{thead}{column} = 1; }
//   for (my $i = $n0 + 1 ; $i <= $n1 ; $i++) {
//     my $c = $alignment->getColumn($i);
//     $$c{skipped} = 1 if $c; }
//   Debug("Halign $alignment: INSTALL column " . join(',', map { $_ . "=" . ToString($$colspec{$_}); } sort keys %$colspec)) if $LaTeXML::DEBUG{halign};
  boxes
}
