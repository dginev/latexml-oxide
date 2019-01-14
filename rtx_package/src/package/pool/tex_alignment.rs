use crate::package::*;

LoadDefinitions!(state, {
  //======================================================================
  // If this is the right solution...
  // then we also should put the desired spacing on a style attribute?!?!?!
  DefConstructor!("\\vskip Glue", sub[document, args, props, state] {
    // $length = $length->ptValue;
    // if ($length > 10) {    # Or what!?!?!?!
    //   if ($document->isCloseable('ltx:para')) {
    //     $document->closeElement('ltx:para'); }
    //   elsif ($document->isOpenable('ltx:break')) {
    //     $document->insertElement('ltx:break'); } }
    // return; },

  });
  // properties => { isSpace => 1, isVerticalSpace => 1 });
});
