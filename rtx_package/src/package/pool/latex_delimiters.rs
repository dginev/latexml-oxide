use crate::package::*;
//----------------------------------------------------------------------
// LaTeX; Table 3.10. Delimiters, p.47
//----------------------------------------------------------------------
// The meaning of OPEN/CLOSE tends to depend upon the pairing,
// rather than the individual tokens.
// This meaning is handled in MathParser (for now)
LoadDefinitions!(state, {
DefMacro!("\\{", r"\ifmmode\lx@math@lbrace\else\lx@text@lbrace\fi", protected => true);
DefMacro!("\\}", r"\ifmmode\lx@math@rbrace\else\lx@text@rbrace\fi", protected => true);
DefMath!("\\lx@math@lbrace", None, "{", role => "OPEN",  stretchy => false, alias => "\\{");
DefMath!("\\lx@math@rbrace", None, "}", role => "CLOSE", stretchy => false, alias => "\\}");
DefPrimitive!("\\lx@text@lbrace", "{", alias => "\\{",
  font => { shape => "upright" }, bounded => true);    // Since not DefMath!
DefPrimitive!("\\lx@text@rbrace", "}", alias => "\\}",
  font => { shape => "upright" }, bounded => true);    // Since not DefMath!
Let!("\\lbrace", "\\{");
Let!(&T_CS!("\\lbrack"), T_OTHER!("["));
Let!("\\rbrace", "\\}");
Let!(&T_CS!("\\rbrack"), T_OTHER!("]"));
DefMath!("\\lceil",  None, "\u{2308}", role => "OPEN",  stretchy => false);    // LEFT CEILING
DefMath!("\\rceil",  None, "\u{2309}", role => "CLOSE", stretchy => false);    // RIGHT CEILING
DefMath!("\\lfloor", None, "\u{230A}", role => "OPEN",  stretchy => false);    // LEFT FLOOR
DefMath!("\\rfloor", None, "\u{230B}", role => "CLOSE", stretchy => false);    // RIGHT FLOOR
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

// # This duplicates in slightly different way what DefMath has put together.
// our %DELIMITER_MAP =
//   ('(' => { char => "(", lrole => 'OPEN', rrole => 'CLOSE' },
//   ')'          => { char => ")",        lrole => 'OPEN',    rrole => 'CLOSE' },
//   '['          => { char => "[",        lrole => 'OPEN',    rrole => 'CLOSE' },
//   ']'          => { char => "]",        lrole => 'OPEN',    rrole => 'CLOSE' },
//   '\{'         => { char => "{",        lrole => 'OPEN',    rrole => 'CLOSE' },
//   '\}'         => { char => "}",        lrole => 'OPEN',    rrole => 'CLOSE' },
//   '\lfloor'    => { char => "\u{230A}", lrole => 'OPEN',    rrole => 'CLOSE', name => 'lfloor' },
//   '\rfloor'    => { char => "\u{230B}", lrole => 'OPEN',    rrole => 'CLOSE', name => 'rfloor' },
//   '\lceil'     => { char => "\u{2308}", lrole => 'OPEN',    rrole => 'CLOSE', name => 'lceil' },
//   '\rceil'     => { char => "\u{2309}", lrole => 'OPEN',    rrole => 'CLOSE', name => 'rceil' },
//   '\langle'    => { char => "\u{27E8}", lrole => 'OPEN',    rrole => 'CLOSE', name => 'langle' },
//   '\rangle'    => { char => "\u{27E9}", lrole => 'OPEN',    rrole => 'CLOSE', name => 'rangle' },
//   '<'          => { char => "\u{27E8}", lrole => 'OPEN',    rrole => 'CLOSE', name => 'langle' },
//   '>'          => { char => "\u{27E9}", lrole => 'OPEN',    rrole => 'CLOSE', name => 'rangle' },
//   '/'          => { char => "/",        lrole => 'MULOP',   rrole => 'MULOP' },
//   '\backslash' => { char => UTF(0x5C),  lrole => 'MULOP',   rrole => 'MULOP', name => 'backslash' },
//   '|'          => { char => "|",        lrole => 'VERTBAR', rrole => 'VERTBAR' },
//   '\|'         => { char => "\u{2225}", lrole => 'VERTBAR', rrole => 'VERTBAR' },
//   '\uparrow'   => { char => "\u{2191}", lrole => 'OPEN', rrole => 'CLOSE', name => 'uparrow' },   # ??
//   '\Uparrow'   => { char => "\u{21D1}", lrole => 'OPEN', rrole => 'CLOSE', name => 'Uparrow' },   # ??
//   '\downarrow' => { char => "\u{2193}", lrole => 'OPEN', rrole => 'CLOSE', name => 'downarrow' }, # ??
//   '\Downarrow' => { char => "\u{21D3}", lrole => 'OPEN', rrole => 'CLOSE', name => 'Downarrow' }, # ??
//   '\updownarrow' => { char => "\u{2195}", lrole => 'OPEN', rrole => 'CLOSE', name => 'updownarrow' }, # ??
//   '\Updownarrow' => { char => "\u{21D5}", lrole => 'OPEN', rrole => 'CLOSE', name => 'Updownarrow' }, # ??
//   );

// # With new treatment of Simple Symbols as just Box's with assigned attributes,
// # we're not getting whatsits, and so we're not looking them up the same way!!!
// # TEMPORARILY (?) hack the Delimiter map
// foreach my $entry (values %DELIMITER_MAP) {
//   $DELIMITER_MAP{ $$entry{char} } = $entry; }

// sub lookup_delimiter {
//   my ($delim) = @_;
//   return $DELIMITER_MAP{$delim}; }

// This is a little messier than you'd think.
// These effectively create a group between the \left,\right.
// And this also gives us a single list of things to parse separately.
// Since \left,\right are TeX, primitives and must be paired up,
// we use a bit of macro trickery to simulate.
// [The \@hidden@bgroup/egroup keep from putting a {} into the UnTeX]
// HOWEVER, an additional complication is that it is a common mistake to omit the balancing \right!
// Using an \egroup (or hidden) makes it hard to recover, so use a special egroup
DefMacro!("\\left XToken", r"\@left #1\@hidden@bgroup");
// # Like \@hidden@egroup, but softer about missing \left
// DefConstructor('\right@hidden@egroup', '',
//   afterDigest => sub {
//     my ($stomach) = @_;
//     if ($STATE->isValueBound('MODE', 0)    # Last stack frame was a mode switch!?!?!
//       || $STATE->lookupValue('groupNonBoxing')) {    # or group was opened with \begingroup
//       Error('unexpected', '\right', undef, "Unbalanced \\right, no balancing \\left."); }
//     else {
//       $stomach->egroup; } },
//   reversion => '');

DefMacro!("\\right XToken", r"\right@hidden@egroup\@right #1");

// DefConstructor('\@left Token',
//   "?#char(<ltx:XMTok role='#role' name='#name' stretchy='#stretchy'>#char</ltx:XMTok>)"
//     . "(?#hint(<ltx:XMHint/>)(#1))",
//   afterDigest => sub { my ($stomach, $whatsit) = @_;
//     my $arg   = $whatsit->getArg(1);
//     my $delim = ToString($arg);
//     if ($delim eq '.') {
//       $whatsit->setProperty(hint => 1); }
//     elsif (my $entry = $DELIMITER_MAP{$delim}) {
//       $whatsit->setProperties(role => $$entry{lrole},
//         char     => $$entry{char},
//         name     => $$entry{name},
//         stretchy => 'true');
//       $whatsit->setFont($arg->getFont()); }
//     elsif (($arg->getProperty('role') || '') eq 'OPEN') {
//       $arg->setProperty(stretchy => 'true'); }
//     else {
//       Warn('unexpected', $delim, $stomach,
//         "Missing delimiter; '.' inserted"); }
//     return; },
//   alias => '\left');
// DefConstructor('\@right Token',
//   "?#char(<ltx:XMTok role='#role' name='#name' stretchy='#stretchy'>#char</ltx:XMTok>)"
//     . "(?#hint(<ltx:XMHint/>)(#1))",
//   afterDigest => sub { my ($stomach, $whatsit) = @_;
//     my $arg   = $whatsit->getArg(1);
//     my $delim = ToString($arg);
//     if ($delim eq '.') {
//       $whatsit->setProperty(hint => 1); }
//     elsif (my $entry = $DELIMITER_MAP{$delim}) {
//       $whatsit->setProperties(role => $$entry{rrole},
//         char     => $$entry{char},
//         name     => $$entry{name},
//         stretchy => 'true');
//       $whatsit->setFont($arg->getFont()); }
//     elsif (($arg->getProperty('role') || '') eq 'CLOSE') {
//       $arg->setProperty(stretchy => 'true'); }
//     else {
//       Warn('unexpected', $delim, $stomach,
//         "Missing delimiter; '.' inserted)"); }
//     return; },
//   alias => '\right');

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
//   my $delim_role = (($delim && ($delim->nodeType == XML_ELEMENT_NODE) && $delim->getAttribute('role')) || '<none>');
//   # if there is some delimiter-like role on the "delimiter", switch it, otherwise, leave it alone!
//   if ($delim && ($delim_role =~ /^(OPEN|MIDDLE|CLOSE|VERTBAR|<none>)$/)) {
//     ## Maybe we shouldn't switch VERTBAR ?
//     ## The catch is that occasionally people use a single \Bigl (or whatever)
//     ## where they should have used a \Big
//     $document->setAttribute($delim, role => $role); }
//   return; }

// # The "m" versions are defined in e-Tex and other places.
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
