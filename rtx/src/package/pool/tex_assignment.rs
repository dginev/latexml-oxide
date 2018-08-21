use package::*;

pub fn load_definitions(core_state: &mut State) -> Result<()> {
  SetupBindingMacros!(core_state);

  //======================================================================
  // Assignment, TeXBook Ch.24, p.275
  //======================================================================
  // <assignment> = <non-macro assignment> | <macro assignment>

  //======================================================================
  // Macros
  // See Chapter 24, p.275-276
  // <macro assignment> = <definition> | <prefix><macro assignment>
  // <definition> = <def><control sequence><definition text>
  // <def> = \def | \gdef | \edef | \xdef
  // <definition text> = <register text><left brace><balanced text><right brace>

  // sub parseDefParameters {
  //   my ($cs, $params) = @_;
  //   my @tokens = $params->unlist;
  //   // Now, recognize parameters and delimiters.
  //   my @params = ();
  //   my $n      = 0;
  //   while (@tokens) {
  //     my $t = shift(@tokens);
  //     if ($t->getCatcode == CC_PARAM) {
  //       if (!@tokens) {    // Special case: lone // NOT following a numbered parameter
  //                         // Note that we require a { to appear next, but do NOT read it!
  //         push(@params, LaTeXML::Core::Parameter->new('RequireBrace', 'RequireBrace')); }
  //       else {
  //         $n++; $t = shift(@tokens);
  //         Fatal('expected', "#$n", $STATE->getStomach,
  //           "Parameters for '" . ToString($cs) . "' not in order in " . ToString($params))
  //           unless (defined $t) && ($n == (ord($t->getString) - ord('0')));
  //         // Check for delimiting text following the parameter //n
  //         my @delim = ();
  //         my ($pc, $cc) = (-1, 0);
  //         while (@tokens && (($cc = $tokens[0]->getCatcode) != CC_PARAM)) {
  //           my $d = shift(@tokens);
  //           push(@delim, $d) unless $cc == $pc && $cc == CC_SPACE;    // BUT collapse whitespace!
  //           $pc = $cc; }
  //         // Found text that marks the end of the parameter
  //         if (@delim) {
  //           my $expected = Tokens(@delim);
  //           push(@params, LaTeXML::Core::Parameter->new('Until',
  //               'Until:' . ToString($expected),
  //               extra => [$expected])); }
  //         // Special case: trailing sole // => delimited by next opening brace.
  //         elsif ((scalar(@tokens) == 1) && ($tokens[0]->getCatcode == CC_PARAM)) {
  //           shift(@tokens);
  //           push(@params, LaTeXML::Core::Parameter->new('UntilBrace', 'UntilBrace')); }
  //         // Nothing? Just a plain parameter.
  //         else {
  //           push(@params, LaTeXML::Core::Parameter->new('Plain', '{}')); } } }
  //     else {
  //       // Initial delimiting text is required.
  //       my @lit = ($t);
  //       while (@tokens && ($tokens[0]->getCatcode != CC_PARAM)) {
  //         push(@lit, shift(@tokens)); }
  //       my $expected = Tokens(@lit);
  //       push(@params, LaTeXML::Core::Parameter->new('Match',
  //           'Match:' . ToString($expected),
  //           extra   => [$expected],
  //           novalue => 1)); }
  //   }
  //   return (@params ? LaTeXML::Core::Parameters->new(@params) : undef); }

  // sub do_def {
  //   my ($globally, $expanded, $gullet, $cs, $params, $body) = @_;
  //   if (!$cs) {
  //     Error('expected', 'Token', $gullet, "Expected definition token");
  //     return; }
  //   elsif (!$params) {
  //     Error('misdefined', $cs, $gullet, "Expected definition parameter list");
  //     return; }
  //   $params = parseDefParameters($cs, $params);
  //   if ($expanded) {
  //     local $LaTeXML::NOEXPAND_THE = 1;
  //     $body = Expand($body); }
  //   $STATE->installDefinition(LaTeXML::Core::Definition::Expandable->new($cs, $params, $body),
  //     ($globally ? 'global' : undef));
  //   AfterAssignment();
  //   return; }

  // DefPrimitive('\def  SkipSpaces Token UntilBrace {}', sub { do_def(0, 0, @_); }, locked => 1);
  // DefPrimitive('\gdef SkipSpaces Token UntilBrace {}', sub { do_def(1, 0, @_); }, locked => 1);
  // DefPrimitive('\edef SkipSpaces Token UntilBrace {}', sub { do_def(0, 1, @_); }, locked => 1);
  // DefPrimitive('\xdef SkipSpaces Token UntilBrace {}', sub { do_def(1, 1, @_); }, locked => 1);

  // <prefix> = \global | \long | \outer
  // See Stomach.pm & Stomach.pm
  DefPrimitive!("\\global",sub[stomach, args, state] { state.set_prefix("global");  Ok(vec![])}, is_prefix => true);
  DefPrimitive!("\\long",  sub[stomach, args, state] { state.set_prefix("long");    Ok(vec![])}, is_prefix => true);
  DefPrimitive!("\\outer", sub[stomach, args, state] { state.set_prefix("outer");   Ok(vec![])}, is_prefix => true);


  // <let assignment> = \futurelet <control sequence><token><token>
  //  | \let<control sequence><equals><one optional space><token>
  DefPrimitive!("\\let Token SkipMatch:= Skip1Space Token", sub[stomach, args, state] {
    unpack_to_token!(args => token1, token2);
    state.let_i(&token1, token2, None); 
    Ok(Vec::new())
   });

  DefMacro!("\\futurelet Token Token Token", sub[gullet, args, state] {
      unpack_to_token!(args => cs, token1, token2);
      state.let_i(&cs, token2.clone(), None);
      Ok(Tokens!(token1, token2))
  });

  DefRegister!("\\catcode Number", Number::new(0),
    getter => Some(Rc::new(|args, state| {
      let num : i32 = args[0].to_number().value_of();
      let code : Catcode = state.lookup_catcode((num as u8) as char).unwrap_or(Catcode::OTHER);
      let code : u8 = code.into();
      Number::new(code.into()).into()
    })),
    setter => Some(Rc::new(|value, args, state| {
      let c_char = (args[0].to_number().value_of() as u8) as char;
      let c_code = From::from(value.value_of() as u8);
      state.assign_catcode(c_char, c_code, None);
    }))
  );

  Ok(())
}
