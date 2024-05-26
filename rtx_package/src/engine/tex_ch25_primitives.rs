use crate::prelude::*;

LoadDefinitions!({
  //======================================================================
  // Horizontal Mode primitives in Ch.25, pp.285--287

  // The following cause tex to start a new paragraph -- they switch to horizontal mode.
  // <horizontal command> = <letter> | <other> | \char | <chardef token>
  //    | \noboundary | \unhbox | \unhcopy | \valign | \vrule
  //    | \hskip | \hfil | \hfill | \hss | \hfilneg
  //    | \accent | \discretionary | \- | \<space> | $

  DefPrimitive!("\\noboundary", None);



  // DefPrimitive('\mskip MuGlue', sub {
  //     my ($stomach, $length) = @_;
  //     my $s = DimensionToSpaces($length);
  //     Box($s, undef, undef, Invocation(T_CS('\mskip'), $length),
  //       width => $length, isSpace => 1); });
  // DefPrimitive('\mkern MuGlue', sub {
  //     my ($stomach, $length) = @_;
  //     my $s = DimensionToSpaces($length);
  //     Box($s, undef, undef, Invocation(T_CS('\mkern'), $length),
  //       width => $length, isSpace => 1); });

  // \lower <dimen> <box>
  // \raise <dimen> <box>
  // But <box> apparently must really explicitly be an \hbox, \vbox or \vtop (?)
  // OR something that expands into one!!

  DefConstructor!("\\lower Dimension MoveableBox",
  // TODO: SVG
  // "?&inSVG()(<svg:g transform='#transform' _noautoclose='1'>#2</svg:g>)\
  // (<ltx:text yoffset='#y'  _noautoclose='1'>#2</ltx:text>)",
  "<ltx:text yoffset='#y'  _noautoclose='1'>#2</ltx:text>",
    // sizer => sub { raisedSizer($_[0]->getArg(2), $_[0]->getArg(1)->negate); },
    after_digest => sub[whatsit] {
      let y         = Dimension(-whatsit.get_arg(1).unwrap().value_of());
      let ypx       = y.px_value(None);
      let transform = if ypx != 0.0 { s!("translate(0,{ypx})") } else { String::new() };
      whatsit.set_property("y", y);
      whatsit.set_property("transform", transform);
    }
  );

  DefConstructor!("\\raise Dimension MoveableBox",
  // TODO: SVG
  // "?&inSVG()(<svg:g transform='#transform' _noautoclose='1'>#2</svg:g>)"
  //   . "(<ltx:text yoffset='#y'  _noautoclose='1'>#2</ltx:text>)",
  "<ltx:text yoffset='#y'  _noautoclose='1'>#2</ltx:text>",
  //sizer       => sub { raisedSizer($_[0]->getArg(2), $_[0]->getArg(1)); },
  after_digest => sub[whatsit] {
    let y         = Dimension(whatsit.get_arg(1).unwrap().value_of());
    let ypx       = y.px_value(None);
    let transform = if ypx != 0.0 { s!("translate(0,{ypx})") } else { String::new() };
    whatsit.set_property("y", y);
    whatsit.set_property("transform", transform);
  });

  // Implement ???
  // DefMacro('\vrule','\relax');
  DefMacro!("\\valign", None);

  DefMacro!("\\vspace{}", "\\vskip#1\\relax");
  // \indent, \noindent, \par; see above.

});
