use crate::package::*;
LoadDefinitions!(state, {
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
  DefMathI!('=', None, '=', role => "RELOP",   meaning  => "equals");
  DefMathI!('+', None, '+', role => "ADDOP",   meaning  => "plus");
  DefMathI!('-', None, '-', role => "ADDOP",   meaning  => "minus");
  DefMathI!('*', None, '*', role => "MULOP",   meaning  => "times");
  DefMathI!('/', None, '/', role => "MULOP",   meaning  => "divide");
  DefMathI!('!', None, '!', role => "POSTFIX", meaning  => "factorial");
  DefMathI!(',', None, ',', role => "PUNCT");
  DefMathI!('.', None, '.', role => "PERIOD");
  DefMathI!(';', None, ';', role => "PUNCT");
  DefMathI!('(', None, '(', role => "OPEN",    stretchy => false);
  DefMathI!(')', None, ')', role => "CLOSE",   stretchy => false);
  DefMathI!('[', None, '[', role => "OPEN",    stretchy => false);
  DefMathI!(']', None, ']', role => "CLOSE",   stretchy => false);
  DefMathI!('|', None, '|', role => "VERTBAR", stretchy => false);
  DefMathI!(':', None, ':', role => "METARELOP", name => "colon"); // Seems like good default role
  DefMathI!('<', None, '<', role => "RELOP", meaning => "less-han");
  DefMathI!('>', None, '>', role => "RELOP", meaning => "greater-han");

  //======================================================================
  // TeX Book, Appendix B, p. 345

  RawTeX!(
    r###"
    \chardef\active=13
    \chardef\@ne=1
    \chardef\tw@=2
    \chardef\thr@@=3
    \chardef\sixt@@n=16
    \chardef\@cclv=255
    \mathchardef\@cclvi=256
    \mathchardef\@m=1000
    \mathchardef\@M=10000
    \mathchardef\@MM=20000
    \countdef\m@ne=21\relax
    \m@ne=-1
  "###
  );

  // #======================================================================
  // # TeX Book, Appendix B, p. 346
  // RawTeX!(
  //   r###"
  // \countdef\count@=255
  // \toksdef\toks@=0
  // \skipdef\skip@=0
  // \dimendef\dimen@=0
  // \dimendef\dimen@i=1
  // \dimendef\dimen@ii=2
  // \count10=22 % allocates \count registers 23, 24, ...
  // \count11=9 % allocates \dimen registers 10, 11, ...
  // \count12=9 % allocates \skip registers 10, 11, ...
  // \count13=9 % allocates \muskip registers 10, 11, ...
  // \count14=9 % allocates \box registers 10, 11, ...
  // \count15=9 % allocates \toks registers 10, 11, ...
  // \count16=-1 % allocates input streams 0, 1, ...
  // \count17=-1 % allocates output streams 0, 1, ...
  // \count18=3 % allocates math families 4, 5, ...
  // \count19=0 % allocates \language codes 1, 2, ...
  // \count20=255 % allocates insertions 254, 253, ...
  // \countdef\insc@unt=20
  // \countdef\allocationnumber=21
  // \countdef\m@ne=22 \m@ne=-1
  // "###
  // );
  // # Various \count's are set; should we?

  // #======================================================================
  // # TeX Book, Appendix B, p. 347
  // # \wlog ??
  // # From plain.tex
  DefPrimitive!("\\newcount Token", sub[stomach, args, state] {
    unpack_to_token!(args => name);
    DefRegisterI!(name, None, Number::new(0.0));
  });
  DefPrimitive!("\\newdimen Token", sub[stomach, args, state] {
    unpack_to_token!(args => name);
    DefRegisterI!(name, None, Dimension::new(0.0));
  });
  DefPrimitive!("\\newskip Token", sub[stomach, args, state] {
    unpack_to_token!(args => name);
    DefRegisterI!(name, None, Glue::new(0.0));
  });
  DefPrimitive!("\\newmuskip Token", sub[stomach, args, state] {
    unpack_to_token!(args => name);
    DefRegisterI!(name, None, MuGlue::new(0.0));
  });
  AssignValue!("allocated_boxes" => false);
  // DefPrimitive('\newbox    Token', sub {
  //     my $n = LookupValue('allocated_boxes');
  //     AssignValue(allocated_boxes => $n + 1, 'global');
  //     AssignValue("box$n", List());
  //     DefRegisterI($_[1], undef, Number($n)); });
  // DefPrimitive('\newhelp Token {}', sub { AssignValue(ToString($_[1]) => $_[2]); });
  // DefPrimitive('\newtoks Token', sub { DefRegisterI($_[1], undef, Tokens()); });
  // # the next 4 actually work by doing a \chardef instead of \countdef, etc.
  // # which means they actually work quite differently
  // DefRegister('\allocationnumber' => Number(0));
  // DefMacro('\alloc@@ {}', sub {
  //     my ($gullet, $type) = @_;
  //     my $c = 'allocation @' . ToString($type);
  //     my $n = LookupValue($c) || '0';
  //     $n = $n->valueOf if ref $n;
  //     AssignValue($c                  => $n + 1,     'global');
  //     AssignValue('\allocationnumber' => Number($n), 'global'); });
  // DefMacro('\newread Token',     '\alloc@@{read}\global\chardef#1=\allocationnumber');
  // DefMacro('\newwrite Token',    '\alloc@@{write}\global\chardef#1=\allocationnumber');
  // DefMacro('\newfam Token',      '\alloc@@{fam}\global\chardef#1=\allocationnumber');
  // DefMacro('\newlanguage Token', '\alloc@@{language}\global\chardef#1=\allocationnumber');

  // # This implementation is quite wrong
  // DefPrimitive('\newinsert Token', sub { DefRegisterI($_[1], undef, Number(0)); });
  // # \alloc@, \ch@ck

  // # TeX plain uses \newdimen, etc. for these.
  // # Is there any advantage to that?
  // DefRegister('\maxdimen',  Dimension(16383.99999 * 65536));
  // DefRegister('\hideskip',  Glue(-1000 * 65536, '1fill'));
  // DefRegister('\centering', Glue('0pt plus 1000pt minus 1000pt'));
  // DefRegister('\p@',        Dimension(65536));
  // DefRegister('\z@',        Dimension(0));
  // DefRegister('\z@skip',    Glue(0, 0, 0));

  // # First approximation. till I figure out \newbox
  // RawTeX('\newbox\voidb@x');
  // #======================================================================
  // # TeX Book, Appendix B, p. 348

  DefMacro!("\\newif DefToken", sub[gullet, args, state] {
    unpack_to_token!(args => cs);
    DefConditionalI!(cs, None);
  });

  // # See the section Registers & Parameters, above for setting default values.
  // #======================================================================
  // # TeX Book, Appendix B, p. 349
  // # See the section Registers & Parameters, above for setting default values.

  // These are originally defined with \newskip, etc
  DefRegister!("\\smallskipamount", Glue!("3pt plus1pt minus1pt"));
  DefRegister!("\\medskipamount", Glue!("6pt plus2pt minus2pt"));
  DefRegister!("\\bigskipamount", Glue!("12pt plus4pt minus4pt"));
  DefRegister!("\\normalbaselineskip", Glue!("12pt"));
  DefRegister!("\\normallineskip", Glue!("1pt"));
  DefRegister!("\\normallineskiplimit", Dimension!("0pt"));
  DefRegister!("\\jot", Dimension!("3pt"));
  DefRegister!("\\lx@default@jot", LookupRegister!("\\jot"));
  DefRegister!("\\interdisplaylinepenalty", Number!(100));
  DefRegister!("\\interfootnotelinepenalty", Number!(100));

  DefMacro!("\\magstephalf", "1095");
  DefMacro!("\\magstep{}", sub[gullet, args, state] {
    unpack_to_string!(args => mag);
    Explode!(match mag.as_str() {
      "0" => "1000",
      "1" => "1200",
      "2" => "1440",
      "3" => "1728",
      "4" => "2074",
      "5" => "2488",
      _ => ""
    })
  });

  // #======================================================================
  // # TeX Book, Appendix B, p. 350

  // # Font stuff ...
  // RawTeX(<<'EoTeX');
  // \font\tenrm=cmr10
  // \font\sevenrm=cmr7
  // \font\fiverm=cmr5
  // \font\teni=cmmi10
  // \font\seveni=cmmi7
  // \font\fivei=cmmi7
  // \font\tensy=cmsy10
  // \font\sevensy=cmsy7
  // \font\fivesy=cmsy5
  // \font\tenex=cmex10
  // \font\tenbf=cmbx10
  // \font\sevenbf=cmbx7
  // \font\fivebf=cmbx5
  // \font\tensl=cmsl10
  // \font\tentt=cmtt10
  // \font\tenit=cmti10
  // \newfam\itfam
  // \newfam\slfam
  // \newfam\bffam
  // \newfam\ttfam
  // \textfont0=\tenrm\scriptfont0=\sevenrm\scriptscriptfont0=\fiverm
  // \textfont1=\teni\scriptfont1=\seveni\scriptscriptfont1=\fivei
  // \textfont2=\tensy\scriptfont2=\sevensy\scriptscriptfont2=\fivesy
  // \textfont3=\tenex
  // EoTeX
  // # Note: \newfam in math should be font switching(?)

  //======================================================================
  // TeX Book, Appendix B, p. 351

  // Old style font styles.
  // The trick is to create an empty Whatsit preserved till assimilation (for reversion'ing)
  // but to change the current font used in boxes.
  // (some of these were defined on different pages? or even latex...)
  Tag!("ltx:text", auto_open => true, auto_close => true);

  // Note that these, unlike \rmfamily, should set the other attributes to the defaults!
  DefPrimitiveI!("\\rm", noprimitive!(),
    font => Font!(family => "serif", series => "medium", shape => "upright"));
  DefPrimitiveI!("\\sf", noprimitive!(),
    font => Font!(family => "sansserif", series => "medium", shape => "upright"));
  DefPrimitiveI!("\\bf", noprimitive!(),
    font => Font!(series => "bold", family => "serif", shape => "upright"));
  DefPrimitiveI!("\\it", noprimitive!(),
    font => Font!(shape => "italic", family => "serif", series => "medium" ));
  DefPrimitiveI!("\\tt", noprimitive!(),
    font => Font!(family => "typewriter", series => "medium", shape => "upright" ));
  // No effect in math for the following 2 ?
  DefPrimitiveI!("\\sl", noprimitive!(),
    font => Font!(shape => "slanted", family => "serif", series => "medium" ));
  DefPrimitiveI!("\\sc", noprimitive!(),
    font => Font!(shape => "smallcaps", family => "serif", series => "medium" ));

  // Ideally, we should set these sizes from class files
  AssignValue!("NOMINAL_FONT_SIZE", 10);
  DefPrimitiveI!("\\tiny",         noprimitive!(), font => Font!(size => 5 ));
  DefPrimitiveI!("\\scriptsize",   noprimitive!(), font => Font!(size => 7 ));
  DefPrimitiveI!("\\footnotesize", noprimitive!(), font => Font!(size => 8 ));
  DefPrimitiveI!("\\small",        noprimitive!(), font => Font!(size => 9 ));
  DefPrimitiveI!("\\normalsize",   noprimitive!(), font => Font!(size => 10 ));
  DefPrimitiveI!("\\large",        noprimitive!(), font => Font!(size => 12 ));
  DefPrimitiveI!("\\Large",        noprimitive!(), font => Font!(size => 14.4 ));
  DefPrimitiveI!("\\LARGE",        noprimitive!(), font => Font!(size => 17.28 ));
  DefPrimitiveI!("\\huge",         noprimitive!(), font => Font!(size => 20.74 ));
  DefPrimitiveI!("\\Huge",         noprimitive!(), font => Font!(size => 29.8 ));

  DefPrimitiveI!("\\mit", noprimitive!(), require_math => true, font => Font!(family => "italic"));

  DefPrimitiveI!("\\frenchspacing", noprimitive!());
  DefPrimitiveI!("\\nonfrenchspacing", noprimitive!());
  // DefMacroI!("\\normalbaselines", undef,
  //   '\lineskip=\normallineskip\baselineskip=\normalbaselineskip\lineskiplimit=\normallineskiplimit');
  DefMacroI!(T_CS!("\\space"), None, T_SPACE!());
  DefMacroI!(T_CS!("\\lq"), None, T_OTHER!("`"));
  DefMacroI!(T_CS!("\\rq"), None, T_OTHER!("'"));
  Let!("\\empty", "\\@empty");
  //DefMacro!("\\null", "\hbox{}");
  Let!("\\bgroup", T_BEGIN!());
  Let!("\\egroup", T_END!());
  Let!("\\endgraf", "\\par");
  Let!("\\endline", "\\cr");

  DefPrimitiveI!("\\endline", noprimitive!());

  // Use \r for the newline from TeX!!!
  DefMacroI!(T_CS!("\\\r"), None, T_CS!("\\ ")); // \<cr> == \<space> Interesting (see latex.ltx)
  LetI!(&T_ACTIVE!("\r"), T_CS!("\\par")); // (or is this just LaTeX?)

  Let!("\\\t", "\\\r"); // \<tab> == \<space>, also

  //======================================================================
  // TeX Book, Appendix B, p. 352

  DefPrimitive!("\\obeyspaces", sub {
     AssignCatcode!(' ', Catcode::ACTIVE);
     LetI!(&T_ACTIVE!(" "), T_CS!("\\space"));
  });
  // Curiously enough, " " (a space) is ALREADY defined to be the same as "\space"
  // EVEN before it is made active. (see p.380)
  LetI!(&T_ACTIVE!(" "), T_CS!("\\space"));

  DefPrimitive!("\\obeylines", sub {
      AssignCatcode!('\r', Catcode::ACTIVE);
      LetI!(&T_ACTIVE!("\r"), T_CS!("\\@break")); // More appropriate than \par, I think?
  });

  DefConstructor!("\\@break", "<ltx:break/>");

  // RawTeX(<<'EoTeX');
  // \def\loop#1\repeat{\def\body{#1}\iterate}
  // \def\iterate{\body \let\next=\iterate \else\let\next=\relax\fi \next}
  // \let\repeat=\fi
  // EoTeX

  DefMacro!("\\enskip", "\\ifmmode\\@math@enskip\\else\\@text@enskip\\fi");
  // DefConstructorI('\@math@enskip', undef,
  //   "<ltx:XMHint name='enskip' width='#width'/>",
  //   alias => '\enskip',
  //   properties => { isSpace => 1, width => sub { Dimension('0.5em'); } });
  // DefPrimitiveI('\@text@enskip', undef, "\x{2002}", alias => '\enskip');

  DefMacro!("\\enspace", "\\ifmmode\\@math@enspace\\else\\@text@enspace\\fi");
  // DefConstructorI('\@math@enspace', undef,
  //   "<ltx:XMHint name='enskip' width='#width'/>",
  //   alias => '\enspace',
  //   properties => { isSpace => 1, width => sub { Dimension('0.5em'); } });
  // DefPrimitiveI('\@text@enspace', undef, "\x{2002}", alias => '\enspace');

  DefMacro!("\\quad", "\\ifmmode\\@math@quad\\else\\@text@quad\\fi");
  // DefConstructorI('\@math@quad', undef,
  //   "<ltx:XMHint name='quad' width='#width'/>",
  //   alias => '\quad',
  //   properties => { isSpace => 1, width => sub { Dimension('1em'); } });
  // DefPrimitiveI('\@text@quad', undef, "\x{2003}", alias => '\quad');

  // # Conceivably should be treated as punctuation! (but maybe even \quad should !?!)
  DefMacro!("\\qquad", "\\ifmmode\\@math@qquad\\else\\@text@qquad\\fi");
  // DefConstructorI('\@math@qquad', undef,
  //   "<ltx:XMHint name='qquad' width='#width'/>",
  //   alias => '\qquad',
  //   properties => { isSpace => 1, width => sub { Dimension('2em'); } });
  // DefPrimitiveI('\@text@qquad', undef, "\x{2003}\x{2003}", alias => '\qquad');

  DefMacro!("\\thinspace", "\\ifmmode\\@math@thinspace\\else\\@text@thinspace\\fi");
  // DefConstructorI('\@math@thinspace', undef,
  //   "<ltx:XMHint name='thinspace' width='#width'/>",
  //   alias => '\thinspace',
  //   properties => { isSpace => 1, width => sub { Dimension('0.16667em'); } });
  // DefPrimitiveI('\@text@thinspace', undef, "\x{2009}", alias => '\thinspace');

  DefMacro!("\\negthinspace", "\\ifmmode\\@math@negthinspace\\else\\@text@negthinspace\\fi");
  // DefConstructorI('\@math@negthinspace', undef,
  //   "<ltx:XMHint name='negthinspace' width='#width'/>",
  //   alias => '\negthinspace',
  //   properties => { isSpace => 1, width => sub { Dimension('-0.16667em'); } });
  // DefPrimitiveI('\@text@negthinspace', undef, "", alias => '\negthinspace');

  // DefConstructor('\hglue Glue', "?#isMath(<ltx:XMHint name='hglue' width='#width'/>)(\x{2003})",
  //   properties => sub { (isSpace => 1, width => $_[1]); });
  DefPrimitiveI!("\\vglue Glue", noprimitive!());
  DefPrimitiveI!("\\topglue", noprimitive!());
  DefPrimitiveI!("\\nointerlineskip", noprimitive!());
  DefPrimitiveI!("\\offinterlineskip", noprimitive!());

  DefMacro!("\\smallskip", "\\vskip\\smallskipamount");
  DefMacro!("\\medskip", "\\vskip\\medskipamount");
  DefMacro!("\\bigskip", "\\vskip\\bigskipamount");

  //======================================================================
  // TeX Book, Appendix B, p. 353

  DefPrimitiveI!("\\break", noprimitive!());
  DefPrimitiveI!("\\nobreak", noprimitive!());
  DefPrimitiveI!("\\allowbreak", noprimitive!());
  DefMacro!("\\nobreakspace", "\\ifmmode\\math@nobreakspace\\else\\text@nobreakspace\\fi");

  DefPrimitive!("\\text@nobreakspace", sub[stomach, whatsit, state] {
    Tbox::new(String::from("\u{00A0}"), None, None, Tokens!(T_CS!("~")), map!("isSpace" => Stored::Bool(true)), state)
  });

  // DefConstructor!("\\math@nobreakspace", "<ltx:XMHint name='nobreakspace' width='#width'/>",
  //   properties => { isSpace => 1, width => sub { Dimension('0.333em'); } },
  //   alias => '~');
  DefMacro!("~", "\\nobreakspace{}");

  // DefMacroI('\slash', undef, '/');
  // DefPrimitiveI('\filbreak', undef, undef);
  DefMacro!("\\goodbreak", "\\par");
  DefMacro!("\\eject", "\\par\\LTX@newpage");
  Let!("\\newpage", "\\eject");
  // DefConstructor!("\\LTX@newpage", "^<ltx:pagination role='newpage'/>");
  DefMacro!("\\supereject", "\\par\\LTX@newpage");
  DefPrimitiveI!("\\removelastskip", noprimitive!());
  DefMacro!("\\smallbreak", "\\par");
  DefMacro!("\\medbreak", "\\par");
  DefMacro!("\\bigbreak", "\\par");
  DefMacro!("\\line", "\\hbox to \\hsize");
  // DefConstructor('\leftline{}', sub {
  //     alignLine($_[0], $_[1], 'left'); },
  //   bounded => 1);
  // DefConstructor('\rightline{}', sub {
  //     alignLine($_[0], $_[1], 'right'); },
  //   bounded => 1);
  // DefConstructor('\centerline{}', sub {
  //     alignLine($_[0], $_[1], 'center'); },
  //   bounded => 1);

  // sub alignLine {
  //   my ($document, $line, $alignment) = @_;
  //   if ($document->isOpenable('ltx:p')) {
  //     $document->insertElement('ltx:p', $line, class => 'ltx_align_' . $alignment); }
  //   elsif ($document->isOpenable('ltx:text')) {
  //     $document->insertElement('ltx:text', $line, class => 'ltx_align_' . $alignment);
  //     $document->insertElement('ltx:break'); }
  //   else {
  //     $document->absorb($line); }
  //   return; }

  // # These should be 0 width, but perhaps also shifted?
  // DefMacro('\llap{}', '\hbox to 0pt{#1}');
  // DefMacro('\rlap{}', '\hbox to 0pt{#1}');
  // DefMacroI('\m@th', undef, '\mathsurround=0pt ');

  // # \strutbox
  // DefMacroI('\strut', undef, Tokens());
  // RawTeX('\newbox\strutbox');

  // #======================================================================
  // # TeX Book, Appendix B. p. 354

  // # TODO: Not yet done!!
  // # tabbing stuff!!!

  // DefMacroI('\settabs', undef, undef);

  // #======================================================================
  // # TeX Book, Appendix B. p. 355

  // DefPrimitive('\hang', undef);

  // # TODO: \item, \itemitem not done!
  // # This could probably be adopted from LaTeX, if the <itemize> could auto-open
  // # and close!
  // DefConstructor('\item{}',     '#1');
  // DefConstructor('\itemitem{}', '#1');

  // DefMacro('\textindent{}', '#1');

  // # Conceivably this should enclose the next para in a block?
  // # Or add attribute to it? Or...
  // DefPrimitiveI('\narrower', undef, undef);
});
