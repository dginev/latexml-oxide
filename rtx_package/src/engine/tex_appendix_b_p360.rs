use crate::prelude::*;

//======================================================================
// TeX Book, Appendix B. p. 360
//
// \choose, et al, already handle above.
// Note that in TeX, all 4 args get digested(!)
// and the choice is made when absorbing!
LoadDefinitions!({
  DefMacro!("\\mathpalette{}{}", r"\mathchoice{#1\displaystyle{#2}}{#1\textstyle{#2}}{#1\scriptstyle{#2}}{#1\scriptscriptstyle{#2}}");

  DefConstructor!("\\phantom{}",
    "?#isMath(<ltx:XMHint width='#width' height='#height' depth='#depth' name='phantom'/>)\
      (<ltx:text class='ltx_phantom'>#1</ltx:text>)");    // !?!?!?!
    // TODO:
    // properties  => { isSpace => 1 },
    // afterDigest => sub {
    //   my $whatsit = $_[1];
    //   my ($w, $h, $d) = $whatsit->getArg(1)->getSize;
    //   $whatsit->setProperties(width => $w, height => $h, depth => $d);
    //   return; });

  DefConstructor!("\\hphantom{}",
    "?#isMath(<ltx:XMHint width='#width' name='hphantom'/>)\
      (<ltx:text class='ltx_phantom'>#1</ltx:text>)");    // !?!?!?!
    // TODO:
    // properties  => { isSpace => 1 },
    // afterDigest => sub {
    //   my $whatsit = $_[1];
    //   my ($w, $h, $d) = $whatsit->getArg(1)->getSize;
    //   $whatsit->setProperties(width => $w, height => $h, depth => $d);
    //   return; });

  DefConstructor!("\\vphantom{}",
    "?#isMath(<ltx:XMHint height='#height' depth='#depth' name='vphantom'/>)\
      (<ltx:text class='ltx_phantom'>#1</ltx:text>)");    // !?!?!?!
    // TODO:
    // properties  => { isSpace => 1 },
    // afterDigest => sub {
    //   my $whatsit = $_[1];
    //   my ($w, $h, $d) = $whatsit->getArg(1)->getSize;
    //   $whatsit->setProperties(width => $w, height => $h, depth => $d);
    //   return; });

  DefConstructor!("\\mathstrut", "?#isMath(<ltx:XMHint name='mathstrut'/>)()",
    properties => { stored_map!("isSpace" => true) });
  DefConstructor!("\\smash{}", "#1");    // well, what?


});
