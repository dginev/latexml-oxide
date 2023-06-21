use crate::package::*;
//----------------------------------------------------------------------
// LaTeX; Table 3.10. Delimiters, p.47
//----------------------------------------------------------------------
// The meaning of OPEN/CLOSE tends to depend upon the pairing,
// rather than the individual tokens.
// This meaning is handled in MathParser (for now)

/// A shorthand data structure for delimiter metadata
pub struct DelimeterMeta {
  char: char,
  left_role: &'static str,
  right_role: &'static str,
  name: Option<&'static str>,

}
/// This duplicates in slightly different way what DefMath has put together.
pub static DELIMITER_MAP : Lazy<HashMap<&'static str, DelimeterMeta>> = Lazy::new(|| raw_map!(
  "(" => DelimeterMeta{char: '(', left_role: "OPEN", right_role: "CLOSE", name:None},
  ")" => DelimeterMeta{char: ')', left_role: "OPEN", right_role: "CLOSE", name:None},
  "[" => DelimeterMeta{char: '[', left_role: "OPEN", right_role: "CLOSE", name:None},
  "]" => DelimeterMeta{ char: ']', left_role: "OPEN", right_role: "CLOSE", name:None},
  "\\{" => DelimeterMeta{ char: '{', left_role: "OPEN", right_role: "CLOSE", name:None},
  "\\}" => DelimeterMeta{ char: '}', left_role: "OPEN", right_role: "CLOSE", name:None},
  "\\lfloor"=> DelimeterMeta{ char: '\u{230A}',
                left_role: "OPEN", right_role: "CLOSE", name: Some("lfloor") },
  "\\rfloor"=> DelimeterMeta{ char: '\u{230B}',
                left_role: "OPEN", right_role: "CLOSE", name: Some("rfloor") },
  "\\lceil" => DelimeterMeta{ char: '\u{2308}',
                left_role: "OPEN", right_role: "CLOSE", name: Some("lceil") },
  "\\rceil" => DelimeterMeta{ char: '\u{2309}',
                left_role: "OPEN", right_role: "CLOSE", name: Some("rceil") },
  "\\langle"=> DelimeterMeta{ char: '\u{27E8}',
                left_role: "OPEN", right_role: "CLOSE", name: Some("langle") },
  "\\rangle"=> DelimeterMeta{ char: '\u{27E9}',
                left_role: "OPEN",  right_role: "CLOSE", name: Some("rangle") },
  "<"      => DelimeterMeta{ char: '\u{27E8}',
                left_role: "OPEN", right_role: "CLOSE", name: Some("langle") },
  ">"      => DelimeterMeta{ char: '\u{27E9}',
                left_role: "OPEN", right_role: "CLOSE", name: Some("rangle") },
  "/"      => DelimeterMeta{ char: '/', left_role: "MULOP",   right_role: "MULOP", name: None },
  "\\backslash" => DelimeterMeta{ char: '\u{005C}',
                left_role: "MULOP",   right_role: "MULOP", name: Some("backslash") },
  "|"      => DelimeterMeta{ char: '|',
                left_role: "VERTBAR", right_role: "VERTBAR", name: None },
  "\\|"     => DelimeterMeta{ char: '\u{2225}',
                left_role: "VERTBAR", right_role: "VERTBAR", name: None },
  "\\uparrow"   => DelimeterMeta{ char: '\u{2191}',
                    left_role: "OPEN", right_role: "CLOSE", name: Some("uparrow") },
  "\\Uparrow"   => DelimeterMeta{ char: '\u{21D1}',
                    left_role: "OPEN", right_role: "CLOSE", name: Some("Uparrow") },
  "\\downarrow" => DelimeterMeta{ char: '\u{2193}',
                    left_role: "OPEN", right_role: "CLOSE", name: Some("downarrow") },
  "\\Downarrow" =>  DelimeterMeta{ char: '\u{21D3}',
                    left_role: "OPEN", right_role: "CLOSE", name: Some("Downarrow") },
  "\\updownarrow" => DelimeterMeta{ char: '\u{2195}',
                    left_role: "OPEN", right_role: "CLOSE", name: Some("updownarrow") },
  "\\Updownarrow" => DelimeterMeta{ char: '\u{21D5}',
                    left_role: "OPEN", right_role: "CLOSE", name: Some("Updownarrow") }
));


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

  // This is a little messier than you'd think.
  // These effectively create a group between the \left,\right.
  // And this also gives us a single list of things to parse separately.
  // Since \left,\right are TeX, primitives and must be paired up,
  // we use a bit of macro trickery to simulate.
  // [The \@hidden@bgroup/egroup keep from putting a {} into the UnTeX]
  // HOWEVER, an additional complication is that it is a common mistake to omit the balancing
  // \right! Using an \egroup (or hidden) makes it hard to recover, so use a special egroup
  DefMacro!("\\left XToken", r"\@left #1\@hidden@bgroup");
  // Like \@hidden@egroup, but softer about missing \left
  DefConstructor!("\\right@hidden@egroup", "",
    after_digest => {
      if state!().is_value_bound("MODE", Some(0)) // Last stack frame was a mode switch!?!?!
        || state!().lookup_bool("groupNonBoxing") { // or group was opened with \begingroup
        Error!("unexpected", "\\right", "Unbalanced \\right, no balancing \\left."); }
      else {
        stomach_mut!().egroup()?;
      }
    },
    reversion => None);

  DefMacro!("\\right XToken", r"\right@hidden@egroup\@right #1");

  DefConstructor!("\\@left Token",
    "?#char(<ltx:XMTok role='#role' name='#name' stretchy='#stretchy'>#char</ltx:XMTok>)\
      (?#hint(<ltx:XMHint/>)(#1))",
    after_digest => sub[whatsit] {
      let delim = whatsit.get_arg(1).map(ToString::to_string).unwrap_or_default();
      if delim == "." {
        whatsit.set_property("hint", true); }
      else if let Some(entry) = DELIMITER_MAP.get(delim.as_str()) {
        whatsit.set_property("role", entry.left_role);
        whatsit.set_property("char", entry.char);
        whatsit.set_property("name", entry.name);
        whatsit.set_property("stretchy", true);
        // TODO: Should we have more Rc<> wrappers over Font?
        whatsit.set_font(Rc::new(
          whatsit.get_arg(1).unwrap().get_font()?.unwrap().into_owned()
        ));
      }
      else if whatsit.get_arg(1).unwrap().get_property_string("role") == "OPEN" {
        whatsit.get_arg_mut(1).unwrap().set_property("stretchy", true);
      } else {
        Warn!("unexpected", delim,
          "Missing delimiter; '.' inserted");
      }
      Ok(Vec::new())
    },
    alias => "\\left");
  DefConstructor!("\\@right Token",
    "?#char(<ltx:XMTok role='#role' name='#name' stretchy='#stretchy'>#char</ltx:XMTok>)\
      (?#hint(<ltx:XMHint/>)(#1))",
    after_digest => sub[whatsit] {
      let delim = whatsit.get_arg(1).map(ToString::to_string).unwrap_or_default();
      if delim == "." {
        whatsit.set_property("hint", true); }
      else if let Some(entry) = DELIMITER_MAP.get(delim.as_str()) {
        whatsit.set_property("role", entry.right_role);
        whatsit.set_property("char", entry.char);
        whatsit.set_property("name", entry.name);
        whatsit.set_property("stretchy", true);
        // TODO: Should we have more Rc<> wrappers over Font?
        whatsit.set_font(Rc::new(
          whatsit.get_arg(1).unwrap().get_font()?.unwrap().into_owned()
        ));
      }
      else if whatsit.get_arg(1).unwrap().get_property_string("role") == "CLOSE" {
        whatsit.get_arg_mut(1).unwrap().set_property("stretchy", true);
      } else {
        Warn!("unexpected", delim,
          "Missing delimiter; '.' inserted");
      }
      Ok(Vec::new())
    },
    alias => "\\right");

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
