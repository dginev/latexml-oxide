use crate::package::*;

//======================================================================
// TeX Book, Appendix B. p. 360
//
// \choose, et al, already handle above.
// Note that in TeX, all 4 args get digested(!)
// and the choice is made when absorbing!
LoadDefinitions!(_state, {
  DefConstructor!("\\mathchoice Digested Digested Digested Digested", sub[_doc,_args,_state] {
    unimplemented!();
    //   my ($document, $d, $t, $s, $ss, %props) = @_;
    //   my $style  = $props{mathstyle};
    //   my $choice = ($style eq 'display' ? $d
    //     : ($style eq 'text' ? $t
    //       : ($style eq 'script' ? $s
    //         : $ss)));
    //   $document->absorb($choice); },
    // properties => { mathstyle => sub { LookupValue('font')->getMathstyle; } });
  });

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
