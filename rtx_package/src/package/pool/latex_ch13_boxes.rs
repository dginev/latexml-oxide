use crate::package::*;

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

LoadDefinitions!(state, {
  //======================================================================
  // C.13.1 Length
  //======================================================================
  // \fill
  DefMacro!("\\stretch{}", "0pt plus #1fill\\relax");

  DefPrimitive!("\\@check@length DefToken", sub[stomach, args, state] {
    unpack_to_token!(args => cs);
    match state.lookup_definition(&cs) {
      None => {
        let message = s!("'{}' is not a length; defining it now", cs.stringify());
        Warn!("undefined", cs, stomach, state, message);
        DefRegister!(cs, None, Dimension::new(0.0));
      },
      Some(defn) => if !defn.is_register() {
        let message = s!("'{}' length was expected, got {:?} instead of register.", cs.to_string(), defn.register_type());
        Error!("misdefined", cs, stomach, state, message);
      }
    };
  });

  DefPrimitive!("\\newlength DefToken", sub[stomach, args, inner_state] {
    unpack_to_token!(args => cs);
    DefRegister!(cs, None, Glue::new(0.0));
    Ok(vec![])
  });
  DefMacro!("\\setlength{}{}", "\\@check@length{#1}#1#2\\relax");
  DefMacro!("\\addtolength{}{}", "\\@check@length{#1}\\advance#1 #2\\relax");

  DefMacro!(
    "\\@settodim{}{}{}",
    "\\setbox\\@tempboxa\\hbox{{#3}}#2#1\\@tempboxa\\setbox\\@tempboxa\\box\\voidb@x"
  );
  DefMacro!("\\settoheight", "\\@settodim\\ht");
  DefMacro!("\\settodepth", "\\@settodim\\dp");
  DefMacro!("\\settowidth", "\\@settodim\\wd");

  // Assuming noone tries to get clever with figuring out the allocation of
  // numbers, these become simple DefRegister's
  DefPrimitive!("\\newcount DefToken", sub[stomach, args, state] {
    unpack_to_token!(args => name);
    DefRegister!(name, None, Number::new(0.0));
  });
  DefPrimitive!("\\newdimen DefToken", sub[stomach, args, state] {
    unpack_to_token!(args => name);
    DefRegister!(name, None, Dimension::new(0.0));
  });
  DefPrimitive!("\\newskip DefToken", sub[stomach, args, state] {
    unpack_to_token!(args => name);
    DefRegister!(name, None, Glue::new(0.0));
  });
  DefPrimitive!("\\newmuskip DefToken", sub[stomach, args, state] {
    unpack_to_token!(args => name);
    DefRegister!(name, None, MuGlue::new(0.0));
  });
  DefPrimitive!("\\newtoks DefToken", sub[stomach, args, state] {
    unpack_to_token!(args => name);
    DefRegister!(name, None, Tokens!());
  });

  // DefRegister!("\\fill", Glue(0, "1fill"));

  //======================================================================
  // C.13.2 Space
  //======================================================================
  DefMacro!(
    "\\hspace  OptionalMatch:* {Dimension}",
    "\\ifmmode\\@math@hskip #2\\relax\\else\\@text@hskip #2\\relax\\fi"
  );

  DefPrimitive!("\\vspace OptionalMatch:* {}", None);
  DefPrimitive!("\\addvspace {}", None);
  DefPrimitive!("\\addpenalty {}", None);
  // \hfill, \vfill

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
    // sizer => "#1", // TODO
    before_digest => sub[stomach, state] { reenter_text_mode(false, state); }
  );

  // our %makebox_alignment = (l => 'left', r => 'right', s => 'justified');
  DefMacro!("\\makebox", "\\@ifnextchar(\\pic@makebox\\@makebox");
  // DefConstructor!("\\@makebox[Dimension][]{}",
  //   "<ltx:text ?#width(width='#width') ?#align(align='#align') _noautoclose='1'>#3</ltx:text>",
  //   mode         => "text", bounded => 1, alias => "\\makebox", sizer => "#3",
  //   beforeDigest => sub { reenterTextMode(); },
  //   properties   => sub {
  //     (($_[2] ? (align => $makebox_alignment{ ToString($_[2]) }) : ()),
  //       ($_[1] ? (width => $_[1]) : ())) });

  let dimp4pt = Dimension!(".4pt");
  let dim3pt = Dimension!("3pt");
  DefRegister!("\\fboxrule", dimp4pt);
  DefRegister!("\\fboxsep", dim3pt);

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
  // DefPrimitive!("\\newsavebox DefToken", sub {
  //     my $n = LookupValue('SAVEBOX') + 1;
  //     AssignValue(SAVEBOX => $n, 'global');
  //     DefRegisterI($_[1], undef, Number($n));
  //     AssignValue('box' . $n, List()); });

  // DefPrimitive!("\\sbox {Register} {}", sub {
  //     my ($defn) = @{ $_[1] };
  //     my $value;
  //     if ($defn && ref $defn) {
  //       $value = $defn->valueOf();
  //       if (ref $value) {
  //         $value = $value->valueOf;
  //       }
  //     } else {
  //       Error('expected', '<definition>', undef, "\\sbox expected a definition, was missing");
  //     }
  //     my $contents = Digest($_[2]);
  //     AssignValue('box' . $value, $contents); return; });

  DefMacro!("\\savebox{}", "\\@ifnextchar({\\pic@savebox#1}{\\@savebox#1}");
  // DefPrimitive!("\\@savebox DefToken[][]{}", sub {
  //     my ($defn, @args) = @{ LookupDefinition($_[1]) };
  //     my $value = $defn->valueOf(@args);
  //     AssignValue('box' . $value, Digest($_[4])); return; });
  // DefPrimitive!("\\@savebox Register [][]{}", sub {
  //     my ($defn)   = @{ $_[1] };
  //     my $value    = $defn->valueOf()->valueOf;
  //     my $contents = Digest($_[4]);
  //     #    AssignValue('box' . $value, Digest($_[4])); return; });
  //     AssignValue('box' . $value, $contents); return; });

  // DefMacro!(T_CS!("\\begin{lrbox}"), '{Token}', "\@begin@lrbox #1");
  // DefPrimitive!("\\end{lrbox}", primtiveproc!(stomach, args, state, {stomach.egroup(state)?; }));
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
  DefConstructor!("\\lx@parboxnewline[]", sub[document, args, props, state] {
    document.maybe_close_element("ltx:p", state)?;
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
  //     AssignValue('\hsize' => $_[4]);
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
  //     AssignValue('\hsize' => $_[4]);
  //     // this conflicts (& not needed?) with insertBlock
  //     Let('\\\\', '\lx@parboxnewline'); });

  // DefConstructor("\\rule[Dimension]{Dimension}{Dimension}",
  //   "<ltx:rule ?#offset(yoffset='#offset') width='#width' height='#height'/>",
  //   properties => sub { (offset => $_[1], width => $_[2], height => $_[3]) });
  // DefConstructor("\\raisebox{Dimension}[Dimension][Dimension]{}",
  //   "<ltx:text yoffset='#1' _noautoclose='1'>#4</ltx:text>",
  //   mode         => 'text', bounded => 1,
  //   beforeDigest => sub { reenterTextMode(); },
  //   sizer        => sub { raisedSizer($_[0]->getArg(4), $_[0]->getArg(1)); });
});
