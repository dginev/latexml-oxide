use crate::package::*;

LoadDefinitions!(state, {
  // # See http://tex.loria.fr/moteurs/etex_ref.html
  // # Section 3. The new features

  // #======================================================================
  // # 3.1 Additional control over expansion
  // # \protected associates with the next defn
  // # (note that it isn't actually used anywhere).
  // DefPrimitiveI('\protected', undef, sub {
  //     $STATE->setPrefix('protected'); return; }, isPrefix => 1);

  // # \detokenize
  // DefMacro('\detokenize GeneralText', sub { Explode(UnTeX($_[1])); });

  // # \unexpanded
  // # This is like \noexpand, but acts on <general text>
  // # with the peculiarity of how <filler> is expanded beforehand!
  // DefMacro('\unexpanded GeneralText', sub {
  //     my ($gullet, $tokens) = @_;
  //     return $gullet->neutralizeTokens($tokens->unlist); });
  // #======================================================================
  // # 3.2. Provision for re-scanning already read text

  // \readline; like \read, but only spaces & other
  DefMacro!("\\readline Number SkipKeyword:to SkipSpaces Token", sub[gullet, args, state] {
    unpack_to_token!(args => port, token);
    let port = port.to_number();
    let mouth_opt = if let Some(Stored::Mouth(mouth)) = LookupValue!(&s!("input_file:{}",port)) {
      Some(Arc::clone(mouth))
    } else {
      None
    };
    if let Some(mouth) = mouth_opt {
      let raw_line = s!("{}\r", mouth.write().unwrap().read_raw_line(false, state).unwrap_or_default());
      DefMacro!(token, None, Tokens!(Explode!(raw_line)));
    }
  });

  // DefMacro("\scantokens GeneralText", sub {
  //     LaTeXML::Core::Mouth->new(UnTeX($_[1]))->readTokens; });

  // #======================================================================
  // # 3.3 Environmental enquiries

  // our @ETEX_VERSION = (qw(2 .2));
  // DefMacro("\eTeXrevision", sub { Explode($ETEX_VERSION[1]); });
  // DefRegister("\eTeXversion" => Number($ETEX_VERSION[0]));

  // # \currentgrouplevel
  // DefRegister('\currentgrouplevel', Number(0),
  //   readonly => 1,
  //   getter => sub { $STATE->getFrameDepth; });

  // # \currentgrouptype returns group types from 0..16 ; but what IS a "group type"?
  // DefRegister('\currentgrouptype', Number(0), readonly => 1);

  // # \ifcsname stuff \endcsname
  // DefConditional('\ifcsname CSName', sub { defined LookupMeaning($_[1]); });

  // # \ifdefined <token>
  // DefConditional('\ifdefined Token', sub { defined LookupMeaning($_[1]); });

  // # ???
  DefRegister!("\\lastnodetype", Number::new(0.0));

  // #======================================================================
  // # 3.4 Generalization of the \mark concept: a class of \marks
  // # but since we don't manage Pages...

  DefPrimitive!("\\marks Number GeneralText", None);
  DefMacro!("\\topmarks Number", "");
  DefMacro!("\\firstmarks Number", "");
  DefMacro!("\\botmarks Number", "");
  DefMacro!("\\splitfirstmarks Number", "");
  DefMacro!("\\splitbotmarks Number", "");

  // #======================================================================
  // # 3.5 Bi-directional typesetting: the TeX--XeT primitives

  // # Should these simply ouput some unicode direction changers,
  // # [Things like:
  // #  202A;LEFT-TO-RIGHT EMBEDDING;Cf;0;LRE;;;;;N;;;;;
  // #  202B;RIGHT-TO-LEFT EMBEDDING;Cf;0;RLE;;;;;N;;;;;
  // #  202C;POP DIRECTIONAL FORMATTING;Cf;0;PDF;;;;;N;;;;;
  // #  202D;LEFT-TO-RIGHT OVERRIDE;Cf;0;LRO;;;;;N;;;;;
  // #  202E;RIGHT-TO-LEFT OVERRIDE;Cf;0;RLO;;;;;N;;;;;
  // # ]
  // # or do we need to do some more intelligent tracking of modes
  // # and directionality?
  // # Presumably we can't rely on the material itself being directional.

  // By leaving this 0, we're saying "Don't use these features"!
  DefRegister!("\\TeXXeTstate" => Number::new(0.0));

  DefMacro!("\\beginL", "");
  DefMacro!("\\beginR", "");
  DefMacro!("\\endL", "");
  DefMacro!("\\endR", "");

  DefRegister!("\\predisplaydirection" => Number::new(0.0)); // ???

  // #======================================================================
  // # 3.6 Additional debugging features
  // DefRegister('\interactionmode' => Number(0));

  // # Should show all open groups & their type.
  // DefPrimitive('\showgroups', undef);

  // # \showtokens <generaltext>
  // # NOTE Debugging aids are currently IGNORED!
  // DefPrimitive('\showtokens GeneralText', undef);

  // DefRegister('\tracingassigns'    => Number(0));    # ???
  // DefRegister('\tracinggroups'     => Number(0));
  // DefRegister('\tracingifs'        => Number(0));    # ???
  // DefRegister('\tracingscantokens' => Number(0));

  // #======================================================================
  // # 3.7 Miscellaneous primitives

  // # \everyeof
  // # NOTE: These tokens are NOT used anywhere (yet?)
  // DefRegister('\everyeof', Tokens());

  // DefConstructor('\middle Token', '#1',
  //   afterConstruct => sub {
  //     my ($document) = @_;
  //     my $current = $document->getNode;
  //     my $delim = $document->getLastChildElement($current) || $current;
  //     $document->setAttribute($delim, role     => 'MIDDLE');
  //     $document->setAttribute($delim, stretchy => 'true');
  //     return; });

  // # \unless someif
  DefConditional!("\\unless Token", sub [gullet, args, state] {
    unpack_to_token!(args => if_token);
    if let Some(Stored::Conditional(defn)) = state.lookup_definition_stored(&if_token) {
      if defn.conditional_type == ConditionalType::If {
        if let Some(ref closure) = defn.test {
          // Invert the if's test!
          let args = defn.read_arguments(gullet, state)?;
          return Ok(!(closure(gullet, args, state)?));
        }
      }
    }
    let msg = s!("\\unless should not be followed by {}",if_token.stringify());
    Error!("unexpected", if_token, gullet, state, msg);
    false
  });

  // #======================================================================
  // # \numexpr, \dimexpr, \gluexpr, \muexpr
  // # These read tokens doing simple parsing until \relax or the parse fails.
  // # since we don't know where it ends, we can't easily use Parse::RecDescent.
  // # They also act like a Register!
  // # $type is one of Number, Dimension, Glue or MuGlue
  // sub etex_readexpr {
  //   my ($gullet, $type) = @_;
  //   my $value = etex_readexpr_i($gullet, $type, 0);
  //   if (my $token = $gullet->readToken) {    # Skip \relax
  //     $gullet->unread($token) unless $token->equals(T_CS('\relax')); }
  //   return $value; }

  // sub etex_readexpr_i {
  //   my ($gullet, $type, $prec) = @_;
  //   # Read a first value
  //   my $value;
  //   my $token = $gullet->readXNonSpace;
  //   if (!$token) {
  //     return; }
  //   elsif ($token->equals(T_OTHER('('))) {
  //     $value = etex_readexpr_i($gullet, $type, 0);
  //     my $close = $gullet->readXToken;    # close parenthesis should have terminated recursive call
  //     if (!$close || !$close->equals(T_OTHER(')'))) {
  //       Error('expected', ')', $gullet,
  //         "Missing close parenthesis in $type expr.", "Got " . ToString($close)); } }
  //   else {                                # Read core TeX value/register
  //     $gullet->unread($token);
  //     $value = $gullet->readValue($type); }

  //   # Now check for a following operator(s) & operand(s) (respecting precedence)
  //   while (my $next = $gullet->readXNonSpace) {
  //     if ($next->equals(T_CS('\relax'))) {
  //       $gullet->unread($next);           # leave the \relax for top-level to strip off.
  //       last; }
  //     elsif ($next->equals(T_OTHER('+')) && ($prec < 1)) {
  //       $value = $value->add(etex_readexpr_i($gullet, $type, 1)); }
  //     elsif ($next->equals(T_OTHER('-')) && ($prec < 1)) {
  //       $value = $value->subtract(etex_readexpr_i($gullet, $type, 1)); }
  //     elsif ($next->equals(T_OTHER('*')) && ($prec < 2)) {    # multiplier should be pure number
  //       $value = $value->multiply(etex_readexpr_i($gullet, 'Number', 2)); }
  //     elsif ($next->equals(T_OTHER('/')) && ($prec < 2)) {    # denominator should be pure number
  //       $value = $value->divideround(etex_readexpr_i($gullet, 'Number', 2)); }
  //     else {                                                  # anything else, we're done.
  //       $gullet->unread($next);
  //       last; } }
  //   return $value; }

  // DefParameterType('NumExpr',  sub { etex_readexpr($_[0], 'Number'); });
  // DefParameterType('DimExpr',  sub { etex_readexpr($_[0], 'Dimension'); });
  // DefParameterType('GlueExpr', sub { etex_readexpr($_[0], 'Glue'); });
  // DefParameterType('MuExpr',   sub { etex_readexpr($_[0], 'MuGlue'); });

  // DefRegister('\numexpr NumExpr',   Number(0),    getter => sub { $_[0]; });
  // DefRegister('\dimexpr DimExpr',   Dimension(0), getter => sub { $_[0]; });
  // DefRegister('\glueexpr GlueExpr', Glue(0),      getter => sub { $_[0]; });
  // DefRegister('\muexpr MuExpr',     MuGlue(0),    getter => sub { $_[0]; });

  // # Not really sure where this comes from; pdftex?
  // DefRegister('\synctex', Number(0));
  // #======================================================================
});
