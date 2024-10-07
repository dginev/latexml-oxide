use crate::prelude::*;

//**********************************************************************
// C.13 Lengths, Spaces and Boxes
//**********************************************************************

//////////
//  Complete to here
//  [except for NOTE'd entries, of course]
//////////

// TODO:
// sub raisedSizer {
//   my ($box, $y) = @_;
//   my ($w, $h, $d) = $box->getSize;
//   my $z = Dimension(0);
//   $h = $h->add($y)->larger($z);
//   $d = $d->subtract($y)->larger($z);
//   return ($w, $h, $d); }

LoadDefinitions!({
  //======================================================================
  // C.13.1 Length
  //======================================================================
  // \fill
  DefMacro!("\\stretch{}", "0pt plus #1fill\\relax");

  DefPrimitive!("\\@check@length DefToken", sub[(cs)] {
    match lookup_definition(&cs)? {
      None => {
        let message = s!("'{}' is not a length; defining it now", cs.stringify());
        Warn!("undefined", cs, message);
        DefRegister!(cs, None, Dimension::new(0));
      },
      Some(defn) => if !defn.is_register() {
        let message = s!("'{}' length was expected, got {:?} instead of register.",
          cs.to_string(), defn.register_type());
        Error!("misdefined", cs, message);
      }
    };
  });

  DefPrimitive!("\\newlength DefToken", sub[(cs)] {
    DefRegister!(cs, None, Glue::new(0), allocate => "\\skip");
    Ok(vec![])
  });

  DefPrimitive!("\\setlength {Variable}{Dimension}", sub[(variable,length)] {
    if let ArgWrap::RegisterDefinition(dbox) = variable {
      let (rtoken, params) = *dbox;
      let defn = rtoken.to_register()
        .expect("if a Variable parameter provides a token, it must have a Register definition.");
      defn.set_value(length.into(), None, params);
    }
    Ok(Vec::new())
  });
  DefPrimitive!("\\addtolength {Variable}{Dimension}", sub[(variable,length)] {
    if let ArgWrap::RegisterDefinition(dbox) = variable {
      let (rtoken, params) = *dbox;
      let defn = rtoken.to_register()
        .expect("if a Variable parameter provides a token, it must have a Register definition.");
      // TODO: can we avoid cloning the params?
      let oldlength = defn.value_of(params.clone()).unwrap_or_default();
      defn.set_value(oldlength.add(length), None, params);
    }
    Ok(Vec::new())
  });

  DefMacro!(
    "\\@settodim{}{}{}",
    "\\setbox\\@tempboxa\\hbox{{#3}}#2#1\\@tempboxa\\setbox\\@tempboxa\\box\\voidb@x"
  );
  DefMacro!("\\settoheight", "\\@settodim\\ht");
  DefMacro!("\\settodepth", "\\@settodim\\dp");
  DefMacro!("\\settowidth", "\\@settodim\\wd");
  DefMacro!(r"\@settopoint{}", r"\divide#1\p@\multiply#1\p@");

  DefRegister!("\\fill", Glue!("0pt plus 1fill"));

  //======================================================================
  // C.13.2 Space
  //======================================================================

  DefPrimitive!("\\hspace OptionalMatch:* {Dimension}", sub[(_star,length)] {
    let s = dimension_to_spaces(length);
    if !s.is_empty() {
      let length_tokens = length.revert()?;

      let tokens = Invocation!(T_CS!("\\hskip"), vec![length_tokens]);
      Tbox::new(arena::pin(&s), None, None, tokens,
        stored_map!("width" => length, "isSpace" => true));
    }
  });

  DefPrimitive!("\\vspace OptionalMatch:* {}", None);
  DefPrimitive!("\\addvspace {}", None);
  DefPrimitive!("\\addpenalty {}", None);
  DefPrimitive!("\\@endparenv", None);

  //======================================================================
  // C.13.3 Boxes
  //======================================================================
  // Can't really get these?
  DefMacro!("\\height", "0pt");
  DefMacro!("\\totalheight", "0pt");
  DefMacro!("\\depth", "0pt");
  DefMacro!("\\width", "0pt");

  DefConstructor!("\\mbox {}", "<ltx:text _noautoclose='1'>#1</ltx:text>",
    mode => "text",
    bounded => true,
    sizer => "#1",
    before_digest => {
      reenter_text_mode(false); }
  );

  // our %makebox_alignment = (l => 'left', r => 'right', s => 'justified');
  DefMacro!("\\makebox", "\\@ifnextchar(\\pic@makebox\\@makebox");
  DefConstructor!("\\@makebox[Dimension][]{}",
    "<ltx:text ?#width(width='#width') ?#align(align='#align') _noautoclose='1'>#3</ltx:text>",
    mode         => "text", bounded => true, alias => "\\makebox", sizer => "#3",
    before_digest => {
      reenter_text_mode(false); }
    // properties   => sub[args] {
    //   let arg1 = &args[0];
    //   let arg2 = &args[1];
    //   (($_[2] ? (align => $makebox_alignment{ ToString($_[2]) }) : ()),
    //     ($_[1] ? (width => $_[1]) : ())) }
  );

  DefRegister!("\\fboxrule", Dimension!(".4pt"));
  DefRegister!("\\fboxsep", Dimension!("3pt"));

  // Peculiar special case!
  //  These are nominally text mode macros. However, there is a somewhat common idiom:
  //     $ ... \framebox{$operator$} ... $
  // in which case the operator gets boxed and really should be treated as a math object.
  // (and ultimately converted to mml:menclose)
  // So, we need to switch to text mode, as usual, but FIRST note whether we started in math mode!
  // Afterwards, if we were in math mode, and the content is math, we'll convert the whole thing
  // to a framed math object.
  // Second special issue:
  //   Although framebox doesn't allow flowed content inside, it is also somewhat common
  // to put a vbox or some other block construct inside.
  // Seemingly, the ultimate html gets somewhat tangled (browser bugs?)
  // At any rate, since we're wrapping with an ltx:text, we'll try to unwrap it,
  // if the contents are a single child that can handle the framing.

  DefMacro!("\\fbox{}", "\\@framebox{#1}");
  DefMacro!("\\framebox", "\\@ifnextchar(\\pic@framebox\\@framebox");
  // DefConstructor("\\@framebox[Dimension][]{}",
  //   "?#mathframe(<ltx:XMArg enclose='box'>#inner</ltx:XMArg>)"
  //     . "(<ltx:text ?#width(width='#width') ?#align(align='#align')"
  //     . " framed='rectangle' framecolor='#framecolor'"
  //     . " _noautoclose='1'>#3</ltx:text>)",
  //   alias => '\framebox', sizer => '#3',
  //   beforeDigest => sub {
  //     my ($stomach) = @_;
  //     my $wasmath = LookupValue('IN_MATH');
  //     $stomach->beginMode('text');
  //     AssignValue(FRAME_IN_MATH => $wasmath); },
  //   properties => sub {
  //     (($_[2] ? (align => $makebox_alignment{ ToString($_[2]) }) : ()),
  //       framecolor => LookupValue('font')->getColor,
  //       ($_[1] ? (width => $_[1]) : ())); },
  //   afterDigest => sub {
  //     my ($stomach, $whatsit) = @_;
  //     my $wasmath = LookupValue('FRAME_IN_MATH');
  //     my $arg     = $whatsit->getArg(3);
  //     $stomach->endMode('text');
  //     if ($wasmath && $arg->isMath) {
  //       $whatsit->setProperties(mathframe => 1, inner => $arg->getBody); }
  //     return; },
  //   afterConstruct => sub {
  //     my ($document, $whatsit) = @_;
  //     my $node = $document->getNode->lastChild;
  //     # If the generated node, has only a single (non space) child
  //     my @c = grep { ($_->nodeType != XML_TEXT_NODE) || ($_->textContent =~ /[^\s\n]/) }
  //       $node->childNodes;
  //     my $model = $document->getModel;
  //     # and that child can have the framed attribute
  //     if ((scalar(@c) == 1)
  //       && $document->canHaveAttribute($model->getNodeQName($c[0]), 'framed')) {
  //       # unwrap, copying the attributes
  //       $document->unwrapNodes($node);
  //       foreach my $k (qw(width align framed)) {
  //         if (my $v = $node->getAttribute($k)) {
  //           $document->setAttribute($c[0], $k => $v); } } } }
  // );

  AssignValue!("SAVEBOX", 100);
  TeX!(r#"""\def\newsavebox#1{\@ifdefinable{#1}{\newbox#1}}
  \DeclareRobustCommand\savebox[1]{%
    \@ifnextchar(%)
      {\@savepicbox#1}{\@ifnextchar[{\@savebox#1}{\sbox#1}}}%
  \DeclareRobustCommand\sbox[2]{\setbox#1\hbox{%
    \color@setgroup#2\color@endgroup}}
  \def\@savebox#1[#2]{%
    \@ifnextchar [{\@isavebox#1[#2]}{\@isavebox#1[#2][c]}}
  \long\def\@isavebox#1[#2][#3]#4{%
    \sbox#1{\@imakebox[#2][#3]{#4}}}
  \def\@savepicbox#1(#2,#3){%
    \@ifnextchar[%]
      {\@isavepicbox#1(#2,#3)}{\@isavepicbox#1(#2,#3)[]}}
  \long\def\@isavepicbox#1(#2,#3)[#4]#5{%
    \sbox#1{\@imakepicbox(#2,#3)[#4]{#5}}}
  \def\lrbox#1{%
    \edef\reserved@a{%
      \endgroup
      \setbox#1\hbox{%
        \begingroup\aftergroup}%
          \def\noexpand\@currenvir{\@currenvir}%
          \def\noexpand\@currenvline{\on@line}}%
    \reserved@a
      \@endpefalse
      \color@setgroup
        \ignorespaces}
  \def\endlrbox{\unskip\color@endgroup}
  \DeclareRobustCommand\usebox[1]{\leavevmode\copy #1\relax}
  """#);

  // DefMacro!(T_CS!("\\begin{lrbox}"), '{Token}', "\@begin@lrbox #1");
  // DefPrimitive!("\\end{lrbox}", primtiveproc!( args, {stomach.egroup()?; }));
  // DefPrimitive!("\\@begin@lrbox Token", sub {
  //     my ($stomach, $token) = @_;
  //     $stomach->bgroup;
  //     my $box = List($stomach->digestNextBody());
  //     AssignValue('box' . ToString($token), $box); });

  // DefPrimitive!("\\usebox {Register}", sub {
  //     my ($defn) = @{ $_[1] };
  //     return Box() unless $defn && ($defn ne 'missing');
  //     my $value = $defn->valueOf()->valueOf;
  //     LookupValue('box' . $value) || Box(); });

  // A soft sorta \par that only closes an ltx:p, but not ltx:para
  DefConstructor!("\\lx@parboxnewline[]", sub[document, _args, _props] {
    document.maybe_close_element("ltx:p")?;
  });

  // //NOTE: There are 2 extra arguments (See LaTeX Companion, p.866)
  // //for height and inner-pos.  We're ignoring them, for now, though.
  // DefConstructor('\parbox[][Dimension][]{Dimension}{}', sub {
  //     my ($document, $attachment, $b, $c, $width, $body, %props) = @_;
  //     insertBlock($document, $body,
  //       width   => $width,
  //       vattach => $props{vattach},
  //       class   => 'ltx_parbox');
  //     return; },
  //   sizer      => '#5',
  //   properties => sub {
  //     (width => $_[4],
  //       vattach => translateAttachment($_[1]),
  //       height  => $_[2]); },
  //   mode => 'text', bounded => 1,
  //   beforeDigest => sub {
  // AssignRegister('\hsize' => $_[4]);
  //     Let('\\\\', '\lx@parboxnewline'); });

  DefMacro!("\\@parboxrestore", "");
  DefConditional!("\\if@minipage");
  // DefEnvironment('{minipage}[][][]{Dimension}', sub {
  //     my ($document, $attachment, $b, $c, $width, %props) = @_;
  //     my $vattach = translateAttachment($attachment);
  //     insertBlock($document, $props{body},
  //       para    => 1,
  //       width   => $width,
  //       vattach => $vattach,
  //       class   => 'ltx_minipage');
  //     return; },
  //   mode => 'text',
  //   properties => sub { (
  //       width   => $_[4],
  //       vattach => translateAttachment($_[1])); },
  //   beforeDigest => sub {
  //     Digest(T_CS('\@minipagetrue'));
  // AssignRegister('\hsize' => $_[4]);
  //     // this conflicts (& not needed?) with insertBlock
  //     Let('\\\\', '\lx@parboxnewline'); });

  // DefConstructor("\\rule[Dimension]{Dimension}{Dimension}",
  //   "<ltx:rule ?#offset(yoffset='#offset') width='#width' height='#height'/>",
  //   properties => sub { (offset => $_[1], width => $_[2], height => $_[3]) });
  DefConstructor!("\\raisebox{Dimension}[Dimension][Dimension]{}",
    "<ltx:text yoffset='#1' _noautoclose='1'>#4</ltx:text>",
    mode         => "text", bounded => true,
    before_digest => {
      reenter_text_mode(false); }
    // TODO
    // sizer        => sub { raisedSizer($_[0]->getArg(4), $_[0]->getArg(1)); }
  );
});
