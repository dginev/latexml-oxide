use crate::package::*;

LoadDefinitions!({
  // TODO:
  // #======================================================================
  // # \choose & friends, also need VERY special argument handling

  // # After digesting the \choose (or whatever), grab the previous and following material
  // # and store as args in the whatsit.

  // # Increment the mathstyle stored in any boxes & whatsits.
  // # The tricky part is to know when NOT to increment!
  // # \displaystyle, constructors that set their own specific style,...
  // # And, any collateral adjustments that had been done in digestion depending on mathstyle
  // # WONT be adjusted!
  // # We don't have a clear API to find the displayable Boxes within;
  // # and we don't have a good handle on grouping...

  // # ARGH!!!!!!!!! RETHINK!!!!!!
  // sub adjustMathstyle {
  //   my ($outerstyle, $adjusted, @boxes) = @_;
  //   foreach my $box (@boxes) {
  //     next unless defined $box;
  //     next if $$adjusted{$box};    # since we do args AND props, be careful not to adjust twice!
  //     $$adjusted{$box} = 1;
  //     my $r = ref $box;
  //     next unless $r && ($r !~ /(?:SCALAR|HASH|ARRAY|CODE|REF|GLOB|LVALUE)/) && $r->isaBox;
  //     return if $box->getProperty('explicit_mathstyle');
  //     next   if $box->getProperty('own_mathstyle');

  //     if ($r eq 'LaTeXML::Core::Box') {
  //       adjustMathStyle_internal($outerstyle, $box); }
  //     elsif ($r eq 'LaTeXML::Core::List') {
  //       adjustMathstyle($outerstyle, $adjusted, $box->unlist); }
  //     elsif ($r eq 'LaTeXML::Core::Whatsit') {
  //       my $style = adjustMathStyle_internal($outerstyle, $box) || $outerstyle;
  //       # now recurse on contained boxes (args AND properties!)
  //       adjustMathstyle($style, $adjusted, $box->getArgs);
  //       adjustMathstyle($style, $adjusted, values %{ $box->getPropertiesRef }); } }
  //   return; }

  // # Heursitic;
  // # we're wanting to adjust the style AS IF the numerator had been already in the next mathstyle
  // # This isn't the same as just shifting the mathstyle!
  // # we're sorta trying to infer WHY the box has a given style...?
  // our %mathstyle_adjust_map = (
  //   display => { display => 'text', text => 'script', script => 'script', scriptscript => 'scriptscript' },
  //   text => { display => 'text', text => 'script', script => 'scriptscript', scriptscript => 'scriptscript' },
  //   script => { display => 'display', text => 'text', script => 'scriptscript', scriptscript => 'scriptscript' },
  //   scriptscript => { display => 'display', text => 'text', script => 'scriptscript', scriptscript => 'scriptscript' });

  // sub adjustMathStyle_internal {
  //   my ($outerstyle, $box) = @_;
  //   $outerstyle = 'display' unless $outerstyle;
  //   if (my $font = $box->getFont) {
  //     my $origstyle = $font->getMathstyle || 'display';
  //     my $newstyle  = $mathstyle_adjust_map{$outerstyle}{$origstyle};
  //     $box->setFont($font->merge(mathstyle => $newstyle));
  //     if (my $recstyle = $box->getProperty('mathstyle')) {    # And adjust here, if recorded.
  //       $box->setProperty(mathstyle => $newstyle);
  //       return $newstyle; } }
  //   return; }

  // sub fracSizer {
  //   my ($numerator, $denominator) = @_;
  //   my $w = $numerator->getWidth->larger($denominator->getWidth);
  //   my $d = $denominator->getTotalHeight->multiply(0.5);
  //   my $h = $numerator->getTotalHeight->add($d);
  //   return ($w, $h, $d); }

  // # \lx@generalized@over{reversion}{keyvals}{top}{bottom}
  // # keyvals: role,meaning, left,right, thickness
  // DefConstructor('\lx@generalized@over Undigested RequiredKeyVals',
  //   "?#needXMDual("
  //     . "<ltx:XMDual>"
  //     . "<ltx:XMApp>"
  //     . "<ltx:XMRef _xmkey='#xmkey0'/>"
  //     . "<ltx:XMRef _xmkey='#xmkey1'/>"
  //     . "<ltx:XMRef _xmkey='#xmkey2'/>"
  //     . "</ltx:XMApp>"
  //     . "<ltx:XMWrap>"
  //     . "#left)()"
  //     . "<ltx:XMApp>"
  //     . "<ltx:XMTok _xmkey='#xmkey0' role='#role' meaning='#meaning' mathstyle='#mathstyle' thickness='#thickness'/>"
  //     . "<ltx:XMArg _xmkey='#xmkey1'>#top</ltx:XMArg>"
  //     . "<ltx:XMArg _xmkey='#xmkey2'>#bottom</ltx:XMArg>"
  //     . "</ltx:XMApp>"
  //     . "?#needXMDual(#right"
  //     . "</ltx:XMWrap>"
  //     . "</ltx:XMDual>)()",
  //   afterDigest => sub {
  //     my ($stomach, $whatsit) = @_;
  //     my $kv = $whatsit->getArg(2);
  //     # Really, we want the mathstyle that was in effect BEFORE the group starting the numerator!
  //     # (there could be a \displaystyle INSIDE the numerator, but that's not the one we want)
  //     # Of course the group that started the numerator may be the start of the Math, itself!
  //     # AND, the numerator, which was already digested, needs it's mathstyle ADJUSTED
  //     my $font = ($state::>isValueBound('MODE', 0)    # Last stack frame was a mode switch!?!?!
  //       ? $state::>lookupValue('font')                # then just use whatever font we've got
  //       : ($state::>isValueBound('font', 0)           # else if font was set in numerator
  //           && $state::>valueInFrame('font', 1))
  //         || $state::>lookupValue('font')             # then just use whatever font we've got
  //     );
  //     my $style     = $font->getMathstyle;
  //     my $role      = ToString($kv->getValue('role'));
  //     my $meaning   = ToString($kv->getValue('meaning'));
  //     my $thickness = ToString($kv->getValue('thickness'));
  //     $role    = 'FRACOP' unless $role;
  //     $meaning = 'divide' if (!$meaning) && ($thickness ne '0pt');
  //     # Unfortunately, the numerator's already digested! We have to adjust it's mathstyle
  //     my @top = $stomach->regurgitate;
  //     # really have to pass +/-1, +/-2 etc..!
  //     adjustMathstyle($style, {}, @top);
  //     MergeFont(fraction => 1);
  //     my @bot     = $stomach->digestNextBody();
  //     my $closing = pop(@bot);    # We'll leave whatever closed the list (endmath, endgroup...)
  //     $whatsit->setProperties(
  //       top       => List(@top, mode => 'math'),
  //       bottom    => List(@bot, mode => 'math'),
  //       role      => $role,
  //       meaning   => $meaning,
  //       thickness => $thickness,
  //       mathstyle => $style);
  //     if ($kv->getValue('left') || $kv->getValue('right')) {
  //       $whatsit->setProperties(needXMDual => 1,
  //         xmkey0 => LaTeXML::Package::getXMArgID(),
  //         xmkey1 => LaTeXML::Package::getXMArgID(),
  //         xmkey2 => LaTeXML::Package::getXMArgID()); }
  //     return $closing; },    # and leave the closing bit, whatever it is.
  //   properties => sub { %{ $_[2]->getKeyVals }; },
  //   sizer      => sub { fracSizer($_[0]->getProperty('top'), $_[0]->getProperty('bottom')); },
  //   reversion  => sub {
  //     my ($whatsit) = @_;
  //     (Revert($whatsit->getProperty('top')),
  //       $whatsit->getArg(1)->unlist,
  //       Revert($whatsit->getProperty('bottom'))); });

  // DefMacro('\choose',
  //   '\lx@generalized@over{\choose}{meaning=binomial,thickness=0pt,left=\@left(,right=\@right)}');
  // DefMacro('\brace',
  //   '\lx@generalized@over{\brace}{thickness=0pt,left=\@left\{,right=\@right\}}');
  // DefMacro('\brack',
  //   '\lx@generalized@over{\brack}{thickness=0pt,left=\@left[,right=\@right]}');
  // DefMacro('\atop',
  //   '\lx@generalized@over{\atop}{thickness=0pt}');
  // DefMacro('\atopwithdelims Token Token',
  //   '\lx@generalized@over{\atopwithdelims #1 #2}{thickness=0pt,left={\@left#1},right={\@right#2}}');
  // DefMacro('\over',
  //   '\lx@generalized@over{\over}{meaning=divide}');
  // DefMacro('\overwithdelims Token Token',
  //   '\lx@generalized@over{\overwithdelims #1 #2}{left={\@left#1},right={\@right#2},meaning=divide}');
  // # My thinking was that this is a "fraction" providing the dimension is > 0!
  // DefMacro('\above Dimension',
  //   '\lx@generalized@over{\above #1}{meaning=divide,thickness=#1}');
  // DefMacro('\abovewithdelims Token Token Dimension',
  // '\lx@generalized@over{\abovewithdelims #1 #2 #3}{left={\@left#1},right={\@right#2},meaning=divide,thickness=#3}');


  //======================================================================
  DefPrimitive!("\\cal", None);
  // TODO:  font => { family => 'caligraphic', series => 'medium', shape => 'upright' });

  // In principle, <ltx:emph> is a nice markup for emphasized.
  // Unfortunately, TeX really just treats it as a font switch.
  // Something like:  \em et.al. \rm more stuff
  // works in TeX, but in our case, since there is no explicit {},
  // the <ltx:emph> stays open!  Ugh!
  // This could still be made to work, but merge font would
  // need to look at any open <ltx:emph>, and then somehow close it!
  DefPrimitive!("\\em", None,
  before_digest => {
    let font = LookupFont!().unwrap();
    let shape = font.get_shape().unwrap_or(&Cow::Borrowed(""));
    let shapevariant = if shape == "italic" { "normal" } else { "italic" };
    AssignValue!("font", font.merge(fontmap!(shape => shapevariant)), Some(Scope::Local));
  });

  // Change math font while still in text!
  DefPrimitive!("\\boldmath", None);
  //  beforeDigest => sub { AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold => 1),
  // 'local'); }, forbidMath => 1);
  DefPrimitive!("\\unboldmath", None);
  // TODO:
  // beforeDigest => sub { AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold => 0),
  // 'local'); }, forbidMath => 1);
});
