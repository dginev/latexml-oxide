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


  // Implement ???
  // DefMacro('\vrule','\relax');
  DefMacro!("\\valign", None);

  DefMacro!("\\vspace{}", "\\vskip#1\\relax");
  // \indent, \noindent, \par; see above.

});
