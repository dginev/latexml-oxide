//! TeX Glue
//! 
//! Core TeX Implementation for LaTeXML
use crate::prelude::*;
LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Glue Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // Inserting, removing glue
  //----------------------------------------------------------------------
  // \hskip            c  inserts horizontal glue in a horizontal or math list.
  // \vskip            c  inserts vertical glue in a vertical list.
  // \unskip           c  removes a glue item from the current list.

  // \hskip handled similarly to \kern
  // \hskip can be ignored in certain situations...

  // DefConstructor!("\\hskip Glue", sub[_document, _args] {
  //     let parent = document.get_node();
  //     if ($document->getNodeQName($parent) eq 'svg:g') {
  //       if (my $x = $length->pxValue) {
  //         # HACK HACK HACK
  //         my $transform = $parent->getAttribute('transform');
  //         $parent->setAttribute(transform => ($transform ? $transform . ' ' : '') . "translate($x,0)");
  //     } }
  //     elsif (inSVG()) {
  //       Warn('unexpected', 'kern', $_[0], "Lost hskip in SVG " . ToString($length)); }
  //     else {
  //       $document->absorb(DimensionToSpaces($length)); } },
  //   properties => sub {
  //     my ($stomach, $length) = @_;
  //     (width => $length, isSpace => 1); 
  // });
  //======================================================================
  // If this is the right solution...
  // then we also should put the desired spacing on a style attribute?!?!?!
  DefConstructor!("\\vskip Glue", sub[document, args, _props] {
    unref!(args => length);
    let length = length.pt_value(None);

    if length > 10.0 {    // Or what!?!?!?!
      if document.is_closeable("ltx:para").is_some() {
        document.close_element("ltx:para")?;
      } else if document.is_openable("ltx:break") {
        document.insert_element("ltx:break", Vec::new(), None)?;
      }
    }},
     // TODO: "height" property
    properties => {stored_map!("isSpace" => true, "isVerticalSpace" => true, "isBreak" => true)}
  );

  DefPrimitive!("\\unskip", {
    // pop until a non-empty box is found
    while let Some(last_box) = pop_box_list() {
      if !last_box.is_empty()? {
        push_box_list(last_box);
        break;
      }
    }
  });
  
  //======================================================================
  // Horizontal skips
  //----------------------------------------------------------------------
  // \hfil             d  inserts first order infinitely stretchable horizontal glue in a horizontal or math list.
  // \hfill            d  inserts second order infinitely stretchable horizontal glue in a horizontal or math list.
  // \hfilneg          d  cancels the stretchability of \hfil.
  // \hss              d  inserts infinitely stretchable and shrinkable horizontal glue in a horizontal or math list.
  DefPrimitive!("\\hss", None);
  DefPrimitive!("\\hfilneg", None);
  DefPrimitive!("\\hfil", {
    Tbox::new(arena::pin_static(" "), None, None, Tokens!(T_CS!("\\hfil")),
    stored_map!("isSpace" => true, "isFill" => true))});
  DefPrimitive!("\\hfill", {
    Tbox::new(arena::pin_static(" "), None, None, Tokens!(T_CS!("\\hfill")),
    stored_map!("isSpace" => true, "isFill" => true)) });

  //======================================================================
  // Vertical skips
  //----------------------------------------------------------------------
  // \vfil             d  inserts first order infinitely stretchable vertical glue in a vertical list.
  // \vfill            d  inserts second order infinitely stretchable vertical glue in a vertical list.
  // \vfilneg          d  cancels the stretchability of \vfil.
  // \vss              d  insert infinitely stretchable and shrinkable vertical glue in a vertical list.
  
  // Stuff to ignore for now...
  DefPrimitive!("\\vfil", None);
  DefPrimitive!("\\vfill", None);
  DefPrimitive!("\\vss", None);
  DefPrimitive!("\\vfilneg", None);

  //======================================================================
  // Lastskip
  //----------------------------------------------------------------------
  // \lastskip         iq is 0.0 pt or the last glue or muglue on the current list.

  DefRegister!("\\lastskip", Glue::new(0), readonly => true);
});