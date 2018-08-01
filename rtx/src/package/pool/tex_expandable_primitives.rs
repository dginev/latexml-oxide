use package::*;

// Hmm... I wonder, should getString itself be dealing with escapechar?
fn escapechar(state: &State) -> String {
  let code = match state.lookup_register("\\escapechar", Vec::new()) {
    Some(RegisterValue::Number(v)) => v.value_of(),
    _ => -1,
  };
  if code >= 0 && code <= 255 {
    let char_code = (code as u8) as char;
    char_code.to_string()
  } else {
    String::new()
  }
}

pub fn load_definitions(state: &mut State) -> Result<()> {
  SetupBindingMacros!(state);

  DefConditional!("\\ifx Token Token", sub[gullet, args, inner_state] {
    if let Some(token1) = args[0].tokens.first() {
      if let Some(token2) = args[1].tokens.first() {
        let xequals = XEquals!(token1, token2, inner_state);
        Ok(xequals)
      } else {
        Ok(false)
      }
    } else {
      Ok(false)
    }
  });

  // DefParameterType!("CSName", reader => Rc::new(|gullet: &mut Gullet, _inner:
  // Vec<Option<Parameters>>, _extra: Vec<Token>, state: &mut State| {     let cs = escapechar();
  //     let s;
  //     // keep newlines from having \n inside!
  //     while (($token = $gullet->readXToken(1)) && (($s = $token->getString) ne "\\endcsname")) {
  //       my $cc = $token->getCatcode;
  //       if ($cc == CC_CS) {
  //         if (defined $STATE->lookupDefinition($token)) {
  //           Error("unexpected", $token, $gullet,
  //             "The control sequence " . ToString($token) . " should not appear between \\csname
  // and \\endcsname"); }         else {
  //           Error("undefined", $token, $gullet, "The token " . Stringify($token) . " is not
  // defined"); } }       // Keep newlines from having \n!
  //       $cs .= ($cc == CC_SPACE ? " " : $s); }
  //     T_CS($cs); });

  // DefMacro!("\\csname CSName", sub {
  //     my ($gullet, $token) = @_;
  //     Let($token, "\relax") unless defined LookupMeaning($token);
  //     $token; });

  DefPrimitive!("\\endcsname", sub[stomach, whatsit, state] {
    error!(target: "unexpected:\\endcsname", "Extra \\endcsname");
    Ok(Vec::new())
  });

  // DefMacro("\\expandafter Token Token", sub {
  //     my ($gullet, $tok, $xtok) = @_;
  //     my $defn;
  //     if (defined($defn = $STATE->lookupExpandable($xtok))) {
  //       // Note that IF expandafter ends up expanding a \the in an \edef,
  //       // that it Overrides the implicit noexpand that \edef would normally use for\the!!
  //       local $LaTeXML::NOEXPAND_THE = undef;
  //       my $x = $defn->invoke($gullet);
  //       ($tok, ($x ? @{$x} : ())); }    // Expand $xtok ONCE ONLY!
  //     else {
  //       ($tok, $xtok); } });

  Ok(())
}
