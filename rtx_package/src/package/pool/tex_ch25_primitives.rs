use crate::package::*;

LoadDefinitions!(state, {
//======================================================================
// Horizontal Mode primitives in Ch.25, pp.285--287

// The following cause tex to start a new paragraph -- they switch to horizontal mode.
// <horizontal command> = <letter> | <other> | \char | <chardef token>
//    | \noboundary | \unhbox | \unhcopy | \valign | \vrule
//    | \hskip | \hfil | \hfill | \hss | \hfilneg
//    | \accent | \discretionary | \- | \<space> | $

// # a candidate for use by \hskip, \hspace, etc... ?
// sub DimensionToSpaces {
//   my ($dimen) = @_;
//   my $fs      = LookupValue('font')->getSize;    # 1 em
//   my $pt      = $dimen->ptValue;
//   my $ems     = $pt / $fs;
//   if    ($ems < 0.01) { return; }
//   elsif ($ems < 0.17) { return Box("\x{2006}"); }    # 6/em
//   elsif ($ems < 0.30) { return Box("\x{2005}"); }    # 4/em
//   elsif ($ems < 0.40) { return Box("\x{2004}"); }    # 3/em (same as nbsp?)
//   else {
//     my $n = int(($ems + 0.3) / 0.333);               # 10pts per space...?
//     return Box((UTF(0xA0) x $n)); } }

// DefPrimitiveI('\noboundary', undef, undef);
// DefMacro('\hskip Glue',   '\ifmmode\@math@hskip #1\relax\else\@text@hskip #1\relax\fi');
// DefMacro('\mskip MuGlue', '\ifmmode\@math@mskip #1\relax\else\@text@mskip #1\relax\fi');

// DefConstructor('\@math@hskip Glue',
//   "<ltx:XMHint width='#1'/>",
//   alias => '\hskip',
//   properties => sub { (width => $_[1], isSpace => 1); }
// );
// DefPrimitive('\@text@hskip Glue', sub {
//     my ($stomach, $length) = @_;
//     DimensionToSpaces($length); },
//   alias => '\hskip');
// DefConstructor('\@math@mskip MuGlue',
//   "<ltx:XMHint width='#1'/>",
//   alias => '\mskip',
//   properties => sub { (width => $_[1], isSpace => 1); }
// );
// DefPrimitive('\@text@mskip MuGlue', sub {
//     my ($stomach, $length) = @_;
//     DimensionToSpaces($length); },
//   alias => '\mskip');

// DefPrimitiveI('\hss', undef, undef);
// DefConstructorI('\hfil', undef, "?#isMath(<ltx:XMHint name='hfil'/>)( )",
//   properties => { isSpace => 1, isFill => 1 });
// DefConstructorI('\hfill', undef, "?#isMath(<ltx:XMHint name='hfill'/>)( )",
//   properties => { isSpace => 1, isFill => 1 });
// DefPrimitiveI('\hfilneg', undef, undef);

// \lower <dimen> <box>
// \raise <dimen> <box>
// But <box> apparently must really explicitly be an \hbox, \vbox or \vtop (?)
// OR something that expands into one!!

DefConstructor!("\\lower Dimension MoveableBox",
  "<ltx:text yoffset='#y'  _noautoclose='1'>#2</ltx:text>",
  after_digest => after_digest!(stomach, whatsit, state, {
    whatsit.set_property("y", Number(whatsit.get_arg(1).unwrap().value_of() * -1.0));
  })
);
// DefConstructor('\raise Dimension MoveableBox',
//   "<ltx:text yoffset='#y' _noautoclose='1'>#2</ltx:text>",
//   afterDigest => sub {
//     $_[1]->setProperty(y => $_[1]->getArg(1)); });

// \unhbox<8bit>, \unhcopy<8bit>
// DefPrimitive('\unhbox Number', sub {
//     my $box   = 'box' . $_[1]->valueOf;
//     my $stuff = LookupValue($box);
//     AssignValue($box, undef);
//     (defined $stuff ? $stuff->unlist : ()); });
// DefPrimitive('\unhcopy Number', sub {
//     my $box   = 'box' . $_[1]->valueOf;
//     my $stuff = LookupValue($box);
//     (defined $stuff ? $stuff->unlist : ()); });

// \vrule
// \valign ???

DefMacro!("\\vspace{}", "\\vskip#1\\relax");
// \indent, \noindent, \par; see above.

DefMacro!("\\discretionary{}{}{}", "#3"); // No hyphenation here!
DefPrimitive!("\\-", None);
DefPrimitive!("\\setlanguage Number", None);
});
