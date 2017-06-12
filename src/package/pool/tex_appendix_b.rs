use package::*;
use rtx_core::{BoxOps};
pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);
//
//**********************************************************************
// Plain;  Extracted from Appendix B.
//**********************************************************************
//
//======================================================================
// TeX Book, Appendix B, p. 344
//======================================================================
// \dospecials ??
//
// Normally, the content branch contains the pure structure and meaning of a construct,
// and the presentation is generated from lower level TeX macros that only concern
// themselves with how to display the object.
// Nevertheless, it is sometimes useful to know where the tokens in the presentation branch
// came from;  particularly what their presumed "meaning" is.
// For example, when search-indexing pmml, or providing links to definitions from the pmml.
//
// The following constructor (see how it's used in DefMath), adds meaning attributes
// whereever it seems sensible on the presentation branch, after it has been generated.

// DefConstructor('\@ASSERT@MEANING{}{}', '#2',
//   reversion      => '#2',
//   afterConstruct => sub {
//     my ($document, $whatsit) = @_;
//     my $node    = $document->getNode;              # This should be the wrapper just added.
//     my $meaning = ToString($whatsit->getArg(1));
//     addMeaningRec($document, $node, $meaning);
//     $node; });

//======================================================================
// Properties for plain characters.
// These are allowed in plain text, but need to act a bit special in math.
  DefMathI!('=', None, '=', role => v!("RELOP"),   meaning  => v!("equals"));
  DefMathI!('+', None, '+', role => v!("ADDOP"),   meaning  => v!("plus"));
  DefMathI!('-', None, '-', role => v!("ADDOP"),   meaning  => v!("minus"));
  DefMathI!('*', None, '*', role => v!("MULOP"),   meaning  => v!("times"));
  DefMathI!('/', None, '/', role => v!("MULOP"),   meaning  => v!("divide"));
  DefMathI!('!', None, '!', role => v!("POSTFIX"), meaning  => v!("factorial"));
  DefMathI!(',', None, ',', role => v!("PUNCT"));
  DefMathI!('.', None, '.', role => v!("PERIOD"));
  DefMathI!(';', None, ';', role => v!("PUNCT"));
  DefMathI!('(', None, '(', role => v!("OPEN"),    stretchy => false);
  DefMathI!(')', None, ')', role => v!("CLOSE"),   stretchy => false);
  DefMathI!('[', None, '[', role => v!("OPEN"),    stretchy => false);
  DefMathI!(']', None, ']', role => v!("CLOSE"),   stretchy => false);
  DefMathI!('|', None, '|', role => v!("VERTBAR"), stretchy => false);
  DefMathI!(':', None, ':', role => v!("METARELOP"), name => v!("colon"));    // Seems like good default role
  DefMathI!('<', None, '<', role => v!("RELOP"), meaning => v!("less-than"));
  DefMathI!('>', None, '>', role => v!("RELOP"), meaning => v!("greater-than"));

  //======================================================================
  // TeX Book, Appendix B, p. 351

  // Old style font styles.
  // The trick is to create an empty Whatsit preserved till assimilation (for reversion'ing)
  // but to change the current font used in boxes.
  // (some of these were defined on different pages? or even latex...)
  Tag!("ltx:text", auto_open => true, auto_close => true);

  // // Note that these, unlike \rmfamily, should set the other attributes to the defaults!
  // DefPrimitiveI!("\\rm", undef, undef,
  //   font => { family => 'serif', series => 'medium', shape => 'upright' });
  // DefPrimitiveI!("\\sf", undef, undef,
  //   font => { family => 'sansserif', series => 'medium', shape => 'upright' });
  DefPrimitiveI!("\\bf", noprimitive!(),
    font => Font!(series => "bold", family => "serif", shape => "upright"));
  // DefPrimitiveI!("\\it", undef, undef,
  //   font => { shape => 'italic', family => 'serif', series => 'medium' });
  // DefPrimitiveI!("\\tt", undef, undef,
  //   font => { family => 'typewriter', series => 'medium', shape => 'upright' });
  // // No effect in math for the following 2 ?
  // DefPrimitiveI!("\\sl", undef, undef,
  //   font => { shape => 'slanted', family => 'serif', series => 'medium' });
  // DefPrimitiveI!("\\sc", undef, undef,
  //   font => { shape => 'smallcaps', family => 'serif', series => 'medium' });

  // // Ideally, we should set these sizes from class files
  // AssignValue!("NOMINAL_FONT_SIZE", ObjectStore::Int(10));
  // DefPrimitiveI!("\\tiny",         undef, undef, font => { size => 5 });
  // DefPrimitiveI!("\\scriptsize",   undef, undef, font => { size => 7 });
  // DefPrimitiveI!("\\footnotesize", undef, undef, font => { size => 8 });
  // DefPrimitiveI!("\\small",        undef, undef, font => { size => 9 });
  // DefPrimitiveI!("\\normalsize",   undef, undef, font => { size => 10 });
  // DefPrimitiveI!("\\large",        undef, undef, font => { size => 12 });
  // DefPrimitiveI!("\\Large",        undef, undef, font => { size => 14.4 });
  // DefPrimitiveI!("\\LARGE",        undef, undef, font => { size => 17.28 });
  // DefPrimitiveI!("\\huge",         undef, undef, font => { size => 20.74 });
  // DefPrimitiveI!("\\Huge",         undef, undef, font => { size => 29.8 });

  // DefPrimitiveI!("\\mit", undef, undef, requireMath => 1, font => { family => 'italic' });

  // DefPrimitiveI!("\\frenchspacing",    undef, undef);
  // DefPrimitiveI!("\\nonfrenchspacing", undef, undef);
  // DefMacroI!("\\normalbaselines", undef,
  //   '\lineskip=\normallineskip\baselineskip=\normalbaselineskip\lineskiplimit=\normallineskiplimit');
  // DefMacroI!("\\space", undef, Tokens(T_SPACE));
  // DefMacroI!("\\lq",    undef, "`");
  // DefMacroI!("\\rq",    undef, "'");
  // Let!("\\empty", "\\@empty");
  // DefMacroI!("\\null", undef, '\hbox{}');
  // Let!("\\bgroup",  T_BEGIN!());
  // Let!("\\egroup",  T_END!());
  // Let!("\\endgraf", "\\par");
  // Let!("\\endline", "\\cr");

  // DefPrimitiveI!("\\endline", undef, undef);

  // // Use \r for the newline from TeX!!!
  // DefMacroI!("\\\r", undef, "\\ ");    // \<cr> == \<space> Interesting (see latex.ltx)
  // Let!(T_ACTIVE("\r"), "\\par");       // (or is this just LaTeX?)

  // Let!("\\\t", "\\\r");               // \<tab> == \<space>, also

  Ok(())
}
