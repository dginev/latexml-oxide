use crate::prelude::*;
//----------------------------------------------------------------------
// LaTeX; Table 3.10. Delimiters, p.47
//----------------------------------------------------------------------
// The meaning of OPEN/CLOSE tends to depend upon the pairing,
// rather than the individual tokens.
// This meaning is handled in MathParser (for now)



LoadDefinitions!({
  DefMacro!("\\{", r"\ifmmode\lx@math@lbrace\else\lx@text@lbrace\fi", protected => true);
  DefMacro!("\\}", r"\ifmmode\lx@math@rbrace\else\lx@text@rbrace\fi", protected => true);
  DefMath!("\\lx@math@lbrace", None, "{", role => "OPEN",  stretchy => false, alias => "\\{");
  DefMath!("\\lx@math@rbrace", None, "}", role => "CLOSE", stretchy => false, alias => "\\}");
  DefPrimitive!("\\lx@text@lbrace", "{", alias => "\\{",
  font => { shape => "upright" }, bounded => true); // Since not DefMath!
  DefPrimitive!("\\lx@text@rbrace", "}", alias => "\\}",
  font => { shape => "upright" }, bounded => true); // Since not DefMath!
  Let!("\\lbrace", "\\{");
  Let!(&T_CS!("\\lbrack"), T_OTHER!("["));
  Let!("\\rbrace", "\\}");
  Let!(&T_CS!("\\rbrack"), T_OTHER!("]"));
  DefMath!("\\lceil",  None, "\u{2308}", role => "OPEN",  stretchy => false); // LEFT CEILING
  DefMath!("\\rceil",  None, "\u{2309}", role => "CLOSE", stretchy => false); // RIGHT CEILING
  DefMath!("\\lfloor", None, "\u{230A}", role => "OPEN",  stretchy => false); // LEFT FLOOR
  DefMath!("\\rfloor", None, "\u{230B}", role => "CLOSE", stretchy => false); // RIGHT FLOOR

  // Note: We should be using 27E8,27E9, which are "mathematical", not 2329,232A

  DefMath!("\\langle", None, "\u{27E8}", role => "OPEN", stretchy => false); // LEFT-POINTING ANGLE BRACKET
  DefMath!("\\rangle", None, "\u{27E9}", role => "CLOSE", stretchy => false); // RIGHT-POINTING ANGLE BRACKET

  // Not sure these should be defined here, or latex, or even latex compat mode.
  DefMath!("\\lgroup", None, "(", font => { series => "bold" }, role => "OPEN",  stretchy => false);
  DefMath!("\\rgroup", None, ")", font => { series => "bold" }, role => "CLOSE", stretchy => false);
  DefMath!("\\bracevert", None, "|", font => { series => "bold" }, role => "VERTBAR");

  // TeX marks some symbols as delimiters which can be used with \left,\right,
  // but many of which have different grammatical roles otherwise, eg. arrows, <, >.
  // Short of setting up TeX's complicated encoding machinery, I need an explicit
  // mapping.  Unfortunately, this doesn't (yet) support people declaring thier own delimiters!

  // With new treatment of Simple Symbols as just Box's with assigned attributes,
  // we're not getting whatsits, and so we're not looking them up the same way!!!
  // TEMPORARILY (?) hack the Delimiter map
  // foreach my $entry (values %DELIMITER_MAP) {
  //   $DELIMITER_MAP{ $$entry{char} } = $entry; }

  // sub lookup_delimiter {
  //   my ($delim) = @_;
  //   return $DELIMITER_MAP{$delim}; }


  // These originally had Token as parameter, rather than {}..... Why?
  // Note that in TeX, \big{((} will only enlarge the 1st paren!!!
  DefConstructor!("\\big {}",  "#1", bounded => true, font => { size => 1.2 });
  DefConstructor!("\\Big {}",  "#1", bounded => true, font => { size => 1.6 });
  DefConstructor!("\\bigg {}", "#1", bounded => true, font => { size => 2.1 });
  DefConstructor!("\\Bigg {}", "#1", bounded => true, font => { size => 2.6 });

  // sub addDelimiterRole {
  //   my ($document, $role) = @_;
  //   my $current = $document->getNode;
  //   my $delim   = $document->getLastChildElement($current) || $current;
  //   my $delim_role = (($delim && ($delim->nodeType == XML_ELEMENT_NODE) &&
  // $delim->getAttribute('role')) || '<none>');   # if there is some delimiter-like role on the
  // "delimiter", switch it, otherwise, leave it alone!   if ($delim && ($delim_role =~
  // /^(OPEN|MIDDLE|CLOSE|VERTBAR|<none>)$/)) {     ## Maybe we shouldn't switch VERTBAR ?
  //     ## The catch is that occasionally people use a single \Bigl (or whatever)
  //     ## where they should have used a \Big
  //     $document->setAttribute($delim, role => $role); }
  //   return; }

  // TODO:
  // The "m" versions are defined in e-Tex and other places.
  // DefConstructor('\bigl {}', '#1', bounded => true, font => { size => 'big' },
  //   afterConstruct => sub { addDelimiterRole($_[0], 'OPEN'); });
  // DefConstructor('\bigm {}', '#1', bounded => true, font => { size => 'big' },
  //   afterConstruct => sub { addDelimiterRole($_[0], 'MIDDLE'); });
  // DefConstructor('\bigr {}', '#1', bounded => true, font => { size => 'big' },
  //   afterConstruct => sub { addDelimiterRole($_[0], 'CLOSE'); });

  // DefConstructor('\Bigl {}', '#1', bounded => true, font => { size => 'Big' },
  //   afterConstruct => sub { addDelimiterRole($_[0], 'OPEN'); });
  // DefConstructor('\Bigm {}', '#1', bounded => true, font => { size => 'Big' },
  //   afterConstruct => sub { addDelimiterRole($_[0], 'MIDDLE'); });
  // DefConstructor('\Bigr {}', '#1', bounded => true, font => { size => 'Big' },
  //   afterConstruct => sub { addDelimiterRole($_[0], 'CLOSE'); });

  // DefConstructor('\biggl {}', '#1', bounded => true, font => { size => 'bigg' },
  //   afterConstruct => sub { addDelimiterRole($_[0], 'OPEN'); });
  // DefConstructor('\biggm {}', '#1', bounded => true, font => { size => 'bigg' },
  //   afterConstruct => sub { addDelimiterRole($_[0], 'MIDDLE'); });
  // DefConstructor('\biggr {}', '#1', bounded => true, font => { size => 'bigg' },
  //   afterConstruct => sub { addDelimiterRole($_[0], 'CLOSE'); });

  // DefConstructor('\Biggl {}', '#1', bounded => true, font => { size => 'Bigg' },
  //   afterConstruct => sub { addDelimiterRole($_[0], 'OPEN'); });
  // DefConstructor('\Biggm {}', '#1', bounded => true, font => { size => 'Bigg' },
  //   afterConstruct => sub { addDelimiterRole($_[0], 'MIDDLE'); });
  // DefConstructor('\Biggr {}', '#1', bounded => true, font => { size => 'Bigg' },
  //   afterConstruct => sub { addDelimiterRole($_[0], 'CLOSE'); });

  Let!(&T_CS!("\\vert"), T_OTHER!("|"));
  Let!("\\Vert", "\\|");
});
