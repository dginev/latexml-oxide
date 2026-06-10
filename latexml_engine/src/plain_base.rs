//! plain_base — Perl: plain_base.pool.ltxml
//!
//! Core plain TeX definitions (Appendix B of The TeXbook)
use crate::prelude::*;


LoadDefinitions!({
  // Perl: plain_base.pool.ltxml — definitions only (no LoadPool calls)
  // bootstrap/dump/constructs are loaded by tex.rs (= LoadFormat('plain'))

  //**********************************************************************
  // Plain;  Extracted from Appendix B.
  //**********************************************************************

  // Remember, we're assigning a NUMBER (codepoint) to a CHARACTER!
  {
    for digit in 0..10 {
      assign_mathcode(
        (b'0' + digit) as char,
        0x7030 + (digit as u16),
        Some(Scope::Global),
      );
    }
    for letter in b'A'..=b'Z' {
      //FYI: 0x20 == 32
      assign_lccode(letter, letter + 32, Some(Scope::Global));
      assign_uccode(letter, letter, Some(Scope::Global));
      assign_mathcode(
        letter as char,
        0x7100 + (letter as u16),
        Some(Scope::Global),
      );
      assign_sfcode(letter as char, 999u16, Some(Scope::Global));

      assign_lccode(letter + 32, letter + 32, Some(Scope::Global));
      assign_uccode(letter + 32, letter, Some(Scope::Global));
      assign_mathcode(
        (letter + 32) as char,
        0x7100 + ((letter + 32) as u16),
        Some(Scope::Global),
      );
    }
  }
  DefRegister!("\\magnification", Number!(1000));
  // \bye moved to plain_constructs.rs (Perl: plain_constructs.pool.ltxml L285)

  // Most of these are ignored, but...
  // Actually, latex.ltx's definition (Perl plain_base.pool.ltxml L29-65).
  // \showoverfull and \loggingoutput are defined as no-ops in Knuth's
  // plain.tex; \tracingall references them so they must exist.
  // \tracingfonts and \showoutput are also ignored debug primitives —
  // Perl declares them in latex_constructs.pool.ltxml L5677-5679 alongside
  // the others. Co-locating here so plain.tex users also get them.
  def_macro_noop("\\hideoutput")?;
  def_macro_noop("\\showoverfull")?;
  def_macro_noop("\\loggingoutput")?;
  def_macro_noop("\\tracingfonts")?;
  def_macro_noop("\\showoutput")?;
  DefMacro!(
    "\\loggingall",
    r"\tracingstats\tw@
      \tracingpages\@ne
      \tracinglostchars\thr@@
      \tracingparagraphs\@ne
      \tracinggroups\@ne
      \tracingifs\@ne
      \tracingscantokens\@ne
      \tracingnesting\@ne
      \errorcontextlines\maxdimen
      \ifdefined\tracingstacklevels \tracingstacklevels\maxdimen \fi
      \noexpand \loggingoutput
      \tracingmacros\tw@
      \tracingcommands\thr@@
      \tracingrestores\@ne
      \tracingassigns\@ne"
  );
  DefMacro!("\\tracingall", r"\showoverfull\loggingall");
  DefMacro!(
    "\\tracingnone",
    r"\tracingassigns\z@
      \tracingrestores\z@
      \tracingonline\z@
      \tracingcommands\z@
      \showboxdepth\m@ne
      \showboxbreadth\m@ne
      \tracingoutput\z@
      \errorcontextlines\m@ne
      \ifdefined\tracingstacklevels \tracingstacklevels\z@ \fi
      \tracingnesting\z@
      \tracingscantokens\z@
      \tracingifs\z@
      \tracinggroups\z@
      \tracingparagraphs\z@
      \tracingmacros\z@
      \tracinglostchars\@ne
      \tracingpages\z@
      \tracingstats\z@"
  );

  // \choose, \brace, \brack moved to math_common.rs (Perl math_common.pool.ltxml L634-642)

  //======================================================================
  // Special Characters.
  // Try to give them some sense in math...
  //
  // \#, \&, \%, \$, \_ math/text dispatch family moved to
  // plain_constructs.rs (which runs in BOTH NODUMP and DUMP paths).
  // The DefPrimitive closures here would have been dump-skipped, leaving
  // them undefined on the dump path; the dispatch macros call into them.
  // Putting them in plain_constructs ensures dump-path math-mode `\&`
  // routes through `\lx@math@amp` (ADDOP XMTok) instead of decaying to
  // the dump's CharDef-38 register which the math parser mishandles.
  // Mirror Perl `plain_base.pool.ltxml:L70-77` semantically (Perl uses
  // single Box-dispatch DefPrimitives — Rust's explicit math/text split
  // is the WISDOM #44 documented divergence).

  // Discretionary times; just treat as invisible ?
  // INVISIBLE TIMES (or MULTIPLICATION SIGN = 00D7)
  DefMath!("\\*", None, "\u{2062}", role => "MULOP", name => "", meaning => "times");

  // If an XMWrap (presumably from \mathop, \mathbin, etc)
  // has multiple children, ALL are XMTok, within a restricted set of roles,
  // we want to concatenate the text content into a single XMTok.
  DefMathRewrite!(xpath => concat!("descendant-or-self::ltx:XMWrap[",
    // Only XMWrap's from the above class of operators
    "(@role='OP' or @role='BIGOP' or @role='RELOP' ",
    "or @role='ADDOP' or @role='MULOP' or @role='BINOP' ",
    "or @role='OPEN' or @role='CLOSE')",
    " and count(child::*) > 1 ",
    // with only XMTok as children with the roles in (roughly) the same set
    " and not(child::*[local-name() != 'XMTok'])",
    " and not(ltx:XMTok[",
    "@role!='OP' and @role!='BIGOP' and @role!='RELOP' and @role!='METARELOP' ",
    "and @role!='ADDOP' and @role!='MULOP' and @role!='BINOP' ",
    "and @role!='OPEN' and @role!='CLOSE'",
    "])]"),
  replace => sub[document, nodes] {
    // Perl: `$node->cloneNode(0)` — SHALLOW clone (attributes only, no
    // children). Rust's `Node::clone` is an Rc clone (same underlying node),
    // so we build a fresh XMTok and carry attributes across.
    let node = nodes.pop().unwrap();
    let content = node.get_content();
    let doc = document.get_document();
    let mut replacement = libxml::tree::Node::new("XMTok", None, doc)?;
    for (k, v) in node.get_attributes() {
      replacement.set_attribute(&k, &v)?;
    }
    if !content.is_empty() {
      replacement.append_text(&content)?;
    }
    document.get_node_mut().add_child(&mut replacement)?;
  });

  // Ligatures moved to tex_fonts.rs (Perl: TeX_Fonts.pool.ltxml L335-365).
  // Perl plain_base.pool.ltxml L108-109: `robust => 1` wraps these in the
  // standard LaTeX \protect/<cs-munged> pair so \MakeUppercase / \edef /
  // moving-argument contexts see the frozen CS instead of substituting the
  // raw glyph mid-traversal (WISDOM #40 accented-letter cluster).
  DefPrimitive!("\\i", "\u{0131}", robust => true); // LATIN SMALL LETTER DOTLESS I
  DefPrimitive!("\\j", "\u{0237}", robust => true);

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Alignment code
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  //======================================================================
  // Low-level bits that appear within alignments or \halign

  // "Initialized" alignment; presets spacing, but since we're ignoring it anyway...
  Let!("\\ialign", "\\halign");
  // Overlapping alignments.
  // Perl: plain_base.pool.ltxml L121-137
  DefMacro!(
    "\\oalign{}",
    r"\@@oalign{\lx@begin@alignment#1\lx@end@alignment}"
  );
  DefConstructor!("\\@@oalign{}", "#1",
  reversion => "\\oalign{#1}", bounded => true, mode => "text",
  before_digest => sub {
    use crate::tex_tables::alignment_bindings;
    use latexml_core::alignment::parse_alignment_template;
    if let Ok(template) = parse_alignment_template("l") {
      alignment_bindings(template, String::new(), SymHashMap::default(), HashMap::default());
    }
  });

  // Lines lie on top of each other.
  DefMacro!(
    "\\ooalign{}",
    r"\@@ooalign{\lx@begin@alignment#1\lx@end@alignment}"
  );
  DefConstructor!("\\@@ooalign{}", "#1",
  reversion => "\\ooalign{#1}", bounded => true, mode => "text",
  before_digest => sub {
    use crate::tex_tables::alignment_bindings;
    use latexml_core::alignment::parse_alignment_template;
    if let Ok(template) = parse_alignment_template("l") {
      alignment_bindings(template, String::new(), SymHashMap::default(), HashMap::default());
    }
  });

  DefConstructor!(
    "\\buildrel Until:\\over {}",
    "<ltx:XMApp role='RELOP'>\
    <ltx:XMTok role='SUPERSCRIPTOP' scriptpos='#scriptpos'/>\
    <ltx:XMArg>#2</ltx:XMArg>\
    <ltx:XMArg>#1</ltx:XMArg>\
    </ltx:XMApp>",
    // Perl: properties => { scriptpos => sub { "mid" . $_[0]->getBoxingLevel; } }
    properties => { stored_map!("scriptpos" => s!("mid{}", stomach::get_boxing_level())) }
  );
  def_macro_noop("\\hidewidth")?;

  //======================================================================
  // TeX Book, Appendix B, p. 344
  //======================================================================
  TeX!(r"\outer\def^^L{\par}");
  DefMacro!(
    "\\dospecials",
    r"\do\ \do\\\do\{\do\}\do\$\do\&\do\#\do\^\do\^^K\do\_\do\^^A\do\%\do\~"
  );

  //======================================================================
  // TeX Book, Appendix B, p. 345
  TeX!(
    r"\chardef\active=13
    \chardef\@ne=1
    \chardef\tw@=2
    \chardef\thr@@=3
    \chardef\sixt@@n=16
    \chardef\@cclv=255
    \mathchardef\@cclvi=256
    \mathchardef\@m=1000
    \mathchardef\@M=10000
    \mathchardef\@MM=20000
    \mathchardef\cdotp=25089
    \mathchardef\ldotp=24890
    \mathchardef\intop=4946
    \mathchardef\ointop=4936
    \countdef\m@ne=21\relax
    \m@ne=-1"
  );
  //======================================================================
  // TeX Book, Appendix B, p. 346
  TeX!(
    r"\countdef\count@=255
  \toksdef\toks@=0
  \skipdef\skip@=0
  \dimendef\dimen@=0
  \dimendef\dimen@i=1
  \dimendef\dimen@ii=2
  \count10=22
  \count11=9
  \count12=9
  \count13=9
  \count14=9
  \count15=9
  \count16=-1
  \count17=-1
  \count18=3
  \count19=0
  \count20=255
  \countdef\insc@unt=20
  \countdef\allocationnumber=21
  \countdef\m@ne=22 \m@ne=-1"
  );
  // Various \count's are set; should we?
  //======================================================================
  // TeX Book, Appendix B, p. 347
  DefPrimitive!("\\wlog{}", sub[(arg)] {
    NoteLog!(Expand!(arg).to_string());
    Ok(Vec::new())
  }, locked => true);
  // From plain.tex (Perl `plain_base.pool.ltxml` L207-218 RawTeX block).
  // Raw `\outer\def` bodies matching Perl exactly so the dump captures
  // these as serializable Token bodies, not opaque Rust closures.
  TeX!(
    r"\outer\def\newcount{\alloc@0\count\countdef\insc@unt}
\outer\def\newdimen{\alloc@1\dimen\dimendef\insc@unt}
\outer\def\newskip{\alloc@2\skip\skipdef\insc@unt}
\outer\def\newmuskip{\alloc@3\muskip\muskipdef\@cclv}
\outer\def\newbox{\alloc@4\box\chardef\insc@unt}
\outer\def\newhelp#1#2{\newtoks#1#1\expandafter{\csname#2\endcsname}}
\outer\def\newtoks{\alloc@5\toks\toksdef\@cclv}
\outer\def\newread{\alloc@6\read\chardef\sixt@@n}
\outer\def\newwrite{\alloc@7\write\chardef\sixt@@n}
\outer\def\newfam{\alloc@8\fam\chardef\sixt@@n}
\outer\def\newlanguage{\alloc@9\language\chardef\@cclvi}"
  );

  // Perl plain_base.pool.ltxml L222: `\newinsert` is closure-backed
  // (the only one in this group). DefRegister with no scope hint
  // matches `DefRegisterI($_[1], undef, Number(0))`.
  DefPrimitive!("\\newinsert Token", sub[(t)] {
    DefRegister!(t, None, Number::new(0));
  });
  // \ch@ck moved to plain_bootstrap.rs (Perl plain_bootstrap.pool.ltxml L33)

  // TeX plain uses \newdimen, etc. for these.
  // Is there any advantage to that?
  // note: rust complains about the 16_383.99999 having excessive precision, hence simplifying
  DefRegister!("\\maxdimen", Dimension::new_f64(16383.99999 * UNITY_F64));
  DefRegister!("\\hideskip", Glue!("-1000pt plus 1fill"));
  DefRegister!("\\centering", Glue!("0pt plus 1000pt minus 1000pt"));
  DefRegister!("\\p@", Dimension::new(UNITY));
  DefRegister!("\\z@", Dimension::new(0));
  DefRegister!("\\z@skip", Glue::new(0));

  // Spacing stuff
  DefConstructor!("\\@", "");

  // First approximation. till I figure out \newbox
  TeX!(r"\newbox\voidb@x");

  //======================================================================
  // TeX Book, Appendix B, p. 348

  // \newif moved to plain_bootstrap.rs (Perl plain_bootstrap.pool.ltxml L37-40)

  // See the section Registers & Parameters, above for setting default values.
  //======================================================================
  // TeX Book, Appendix B, p. 349
  // See the section Registers & Parameters, above for setting default values.

  // These are originally defined with \newskip, etc
  DefRegister!("\\smallskipamount", Glue!("3pt plus1pt minus1pt"));
  DefRegister!("\\medskipamount", Glue!("6pt plus2pt minus2pt"));
  DefRegister!("\\bigskipamount", Glue!("12pt plus4pt minus4pt"));
  DefRegister!("\\normalbaselineskip", Glue!("12pt"));
  DefRegister!("\\normallineskip", Glue!("1pt"));
  DefRegister!("\\normallineskiplimit", Dimension!("0pt"));
  DefRegister!("\\jot", Dimension!("3pt"));

  let jot_val = LookupRegister!("\\jot");

  DefRegister!("\\lx@default@jot", jot_val);
  DefRegister!("\\interdisplaylinepenalty", Number(100));
  DefRegister!("\\interfootnotelinepenalty", Number(100));

  DefMacro!("\\magstephalf", "1095");
  DefMacro!("\\magstep{}", sub[(mag)] {
    Explode!(match mag.to_string().as_str() {
      "0" => "1000",
      "1" => "1200",
      "2" => "1440",
      "3" => "1728",
      "4" => "2074",
      "5" => "2488",
      _ => ""
    })
  });

  //======================================================================
  // TeX Book, Appendix B, p. 350

  // Font stuff ...
  TeX!(
    r"\font\tenrm=cmr10
  \font\tenrm=cmr10
  \font\sevenrm=cmr7
  \font\fiverm=cmr5
  \font\teni=cmmi10
  \font\seveni=cmmi7
  \font\fivei=cmmi7
  \font\tensy=cmsy10
  \font\sevensy=cmsy7
  \font\fivesy=cmsy5
  \font\tenex=cmex10
  \font\tenbf=cmbx10
  \font\sevenbf=cmbx7
  \font\fivebf=cmbx5
  \font\tensl=cmsl10
  \font\tentt=cmtt10
  \font\tenit=cmti10
  \newfam\itfam
  \newfam\slfam
  \newfam\bffam
  \newfam\ttfam
 \textfont0=\tenrm\scriptfont0=\sevenrm\scriptscriptfont0=\fiverm
 \textfont1=\teni\scriptfont1=\seveni\scriptscriptfont1=\fivei
 \textfont2=\tensy\scriptfont2=\sevensy\scriptscriptfont2=\fivesy
 \textfont3=\tenex"
  );

  // Note: \newfam in math should be font switching(?)

  //======================================================================
  // TeX Book, Appendix B, p. 351

  // Old style font styles.
  // The trick is to create an empty Whatsit preserved till assimilation (for reversion'ing)
  // but to change the current font used in boxes.
  // (some of these were defined on different pages? or even latex...)

  // Ideally, we should set these sizes from class files
  AssignValue!("NOMINAL_FONT_SIZE", 10);

  // Perl plain_base.pool.ltxml L371 (shadowed L369's declarative form):
  //   DefPrimitiveI('\mit', undef, sub {
  //     if (LookupValue('IN_MATH')) {
  //       MergeFont(family => 'math', shape => 'italic'); }
  //     return; });
  // Perl plain_base.pool.ltxml L369: DefPrimitiveI('\mit', undef, undef,
  //   requireMath => 1, font => { family => 'italic' });
  // (the current Perl shape requires math AND sets a font option, no closure).
  // Rust simplification: use closure + guarded MergeFont, but we still need
  // require_math => true to match Perl's "error when used outside math" check.
  DefPrimitive!("\\mit", {
    if state::lookup_bool_sym(pin!("IN_MATH")) {
      MergeFont!(family => "math", shape => "italic");
    }
  }, require_math => true);

  def_primitive_noop("\\frenchspacing")?;
  def_primitive_noop("\\nonfrenchspacing")?;
  DefMacro!(
    "\\normalbaselines",
    r"\lineskip=\normallineskip\baselineskip=\normalbaselineskip\lineskiplimit=\normallineskiplimit"
  );
  DefMacro!(T_CS!("\\space"), None, T_SPACE!());
  DefMacro!(T_CS!("\\lq"), None, T_OTHER!("`"));
  DefMacro!(T_CS!("\\rq"), None, T_OTHER!("'"));
  Let!("\\empty", "\\lx@empty");
  DefMacro!("\\null", "\\hbox{}");
  Let!("\\bgroup", T_BEGIN!());
  Let!("\\egroup", T_END!());
  Let!("\\endgraf", "\\par");
  Let!("\\endline", "\\cr");

  def_primitive_noop("\\endline")?;

  // Use \r for the newline from TeX!!!
  DefMacro!(T_CS!("\\\r"), None, T_CS!("\\ ")); // \<cr> == \<space> Interesting (see latex.ltx)
  Let!(&T_ACTIVE!('\r'), T_CS!("\\par")); // (or is this just LaTeX?)

  Let!("\\\t", "\\\r"); // \<tab> == \<space>, also

  //======================================================================
  // TeX Book, Appendix B, p. 352

  DefPrimitive!("\\obeyspaces", {
    AssignCatcode!(' ', Catcode::ACTIVE);
    Let!(&T_ACTIVE!(' '), T_CS!("\\space"));
  });
  // Curiously enough, " " (a space) is ALREADY defined to be the same as "\space"
  // EVEN before it is made active. (see p.380)
  Let!(&T_ACTIVE!(' '), T_CS!("\\space"));

  DefPrimitive!("\\obeylines", {
    AssignCatcode!('\r', Catcode::ACTIVE);
    Let!(&T_ACTIVE!('\r'), T_CS!("\\@break")); // More appropriate than \par, I think?
  });

  DefConstructor!("\\@break", "<ltx:break/>",
    properties => { stored_map!("isBreak" => true) });

  TeX!(
    r"
  \def\loop#1\repeat{\def\body{#1}\iterate}
  \def\iterate{\body \let\next=\iterate \else\let\next=\relax\fi \next}
  \let\repeat=\fi
  "
  );

  DefPrimitive!("\\enskip", {
    Tbox::new(
      arena::pin_static("\u{2002}"),
      None,
      None,
      Tokens!(T_CS!("\\enskip")),
      stored_map!("name" => "enskip", "width" => Dimension::from_str("0.5em")?,
      "isSpace"=>true),
    )
  });

  DefPrimitive!("\\enspace", {
    Tbox::new(
      arena::pin_static("\u{2002}"),
      None,
      None,
      Tokens!(T_CS!("\\enspace")),
      stored_map!("name" => "enskip", "width" => Dimension::from_str("0.5em")?,
        "isSpace"=>true),
    )
  });

  DefPrimitive!("\\quad", {
    Tbox::new(
      arena::pin_static("\u{2003}"),
      None,
      None,
      Tokens!(T_CS!("\\quad")),
      stored_map!("name" => "quad", "width" => Dimension::from_str("1em")?,
        "isSpace"=>true),
    )
  });

  // Conceivably should be treated as punctuation! (but maybe even \quad should !?!)
  DefPrimitive!("\\qquad", {
    Tbox::new(
      arena::pin_static("\u{2003}\u{2003}"),
      None,
      None,
      Tokens!(T_CS!("\\qquad")),
      stored_map!("name" => "qquad", "width" => Dimension::from_str("2em")?,
        "isSpace"=>true),
    )
  });

  DefPrimitive!("\\thinspace", {
    Tbox::new(
      arena::pin_static("\u{2009}"),
      None,
      None,
      Tokens!(T_CS!("\\thinspace")),
      stored_map!("name" => "thinspace", "width" => Dimension::from_str("0.16667em")?,
        "isSpace"=>true),
    )
  });

  DefPrimitive!("\\negthinspace", {
    Tbox::new(
      pin!(""),
      None,
      None,
      Tokens!(T_CS!("\\negthinspace")),
      stored_map!("name" => "negthinspace", "width" => Dimension::from_str("-0.16667em")?,
        "isSpace"=>true),
    )
  });

  // \medspace / \negmedspace / \thickspace / \negthickspace moved to
  // latex_constructs.rs (Perl latex_constructs.pool.ltxml L2510-2525).
  // Plain TeX has no medspace/thickspace family — they appear once LaTeX
  // C.7.7 Spacing is loaded.

  // Perl: plain_base.pool.ltxml L447
  DefPrimitive!("\\hglue Glue", sub[(length)] {
    let s = dimension_to_spaces(length);
    if s.is_empty() { return Ok(Vec::new()); }
    Tbox::new(arena::pin(&s), None, None,
      Invocation!(T_CS!("\\hglue"), vec![length.revert()?]),
      stored_map!("name" => "hglue", "width" => length, "isSpace" => true))
  });
  def_primitive_noop("\\vglue Glue")?;
  def_primitive_noop("\\topglue")?;
  // Perl: DefMacroI('\nointerlineskip',undef,'\prevdepth-1000\p@');
  DefMacro!("\\nointerlineskip", r"\prevdepth-1000\p@");
  // Perl: DefMacroI('\offinterlineskip',undef, '\baselineskip-1000\p@\lineskip\z@
  // \lineskiplimit\maxdimen');
  DefMacro!(
    "\\offinterlineskip",
    r"\baselineskip-1000\p@\lineskip\z@ \lineskiplimit\maxdimen"
  );

  DefMacro!("\\smallskip", "\\vskip\\smallskipamount");
  DefMacro!("\\medskip", "\\vskip\\medskipamount");
  DefMacro!("\\bigskip", "\\vskip\\bigskipamount");

  //======================================================================
  // TeX Book, Appendix B, p. 353

  def_primitive_noop("\\break")?;
  def_primitive_noop("\\nobreak")?;
  // \nobreakspace not in Perl plain_base — defined as `\lx@nobreakspace`
  // in base_utilities.rs (Perl Base_Utility.pool.ltxml:53), then Let'd
  // in latex_constructs.rs (Perl latex_constructs.pool.ltxml:48). Plain
  // format leaves it undefined, matching Perl.
  // Perl: DefMacro(T_ACTIVE("~"), T_CS('\lx@NBSP'));
  // `protected => true`: keep active `~` UNEXPANDED in partial
  // expansion (`\write`'s `XGeneralText`, …). Without it, the `~`
  // baked into a written aux file becomes the literal CS name
  // `\lx@NBSP`, which on re-read with `@`=OTHER splits to `\lx` +
  // `@NBSP`. See `plain_constructs.rs` `\&` for the parallel
  // dispatch-macro case.
  DefMacro!(T_ACTIVE!('~'), None, "\\lx@NBSP", protected => true);

  DefMacro!("\\slash", "/");
  def_primitive_noop("\\filbreak")?;
  DefMacro!("\\goodbreak", "\\par");
  def_primitive_noop("\\removelastskip")?;
  DefMacro!("\\smallbreak", "\\par");
  DefMacro!("\\medbreak", "\\par");
  DefMacro!("\\bigbreak", "\\par");
  DefMacro!("\\line", "\\hbox to \\hsize");

  // These should be 0 width, but perhaps also shifted?
  DefMacro!("\\llap{}", r"\hbox to 0pt{\hss#1}");
  DefMacro!("\\rlap{}", r"\hbox to 0pt{#1\hss}");
  DefMacro!("\\m@th", "\\mathsurround=0pt ");
  // fontmath.ltx L521: \def\n@space{\nulldelimiterspace\z@ \m@th}
  // Zero the null-delimiter space and mathsurround for hand-built math
  // delimiter boxes (\big/\Big… and many package delimiter helpers use it).
  // Rust defines \big etc. without going through fontmath.ltx's `\n@space`
  // path, so the macro itself was missing — packages/documents calling it
  // directly errored (witness 2206.12768 et al.). Perl defines it via the
  // kernel. Faithful literal port. \z@/\m@th/\nulldelimiterspace all exist.
  DefMacro!("\\n@space", r"\nulldelimiterspace\z@ \m@th");

  // \strutbox
  def_macro_noop("\\strut")?;
  TeX!("\\newbox\\strutbox");
  //======================================================================
  // TeX Book, Appendix B. p. 354

  // Plain TeX tabbing — \settabs stub (no structured tabbing in plain)

  def_macro_noop("\\settabs")?;
  //======================================================================
  // TeX Book, Appendix B. p. 355

  def_primitive_noop("\\hang")?;

  // Plain TeX \item/\itemitem — simple hanging indent with \textindent.
  // No auto-opening <itemize> — plain TeX doesn't have structured lists.
  DefMacro!("\\hang", r"\hangindent\parindent");
  DefMacro!("\\item", r"\par\hang\textindent");
  DefMacro!(
    "\\itemitem",
    r"\par\indent \hangindent2\parindent \textindent"
  );
  DefMacro!("\\textindent{}", r"\indent\llap{#1\enspace}\ignorespaces");
  DefMacro!(
    "\\narrower",
    r"\advance\leftskip by\parindent\advance\rightskip by\parindent"
  );

  //----------------------------------------------------------------------
  // Actually from LaTeX; Table 3.2. Non-English Symbols, p.39
  // Perl: plain_base.pool.ltxml L521-533. The following shouldn't appear
  // in math. latex_constructs.pool.ltxml L2814-2824 re-emits these with
  // `robust => 1` once LaTeX kernel is loaded.
  DefPrimitive!("\\OE", "\u{0152}"); // LATIN CAPITAL LIGATURE OE
  DefPrimitive!("\\oe", "\u{0153}"); // LATIN SMALL LIGATURE OE
  DefPrimitive!("\\AE", "\u{00C6}"); // LATIN CAPITAL LETTER AE
  DefPrimitive!("\\ae", "\u{00E6}"); // LATIN SMALL LETTER AE
  DefPrimitive!("\\AA", "\u{00C5}"); // LATIN CAPITAL LETTER A WITH RING ABOVE
  DefPrimitive!("\\aa", "\u{00E5}"); // LATIN SMALL LETTER A WITH RING ABOVE
  DefPrimitive!("\\O", "\u{00D8}"); // LATIN CAPITAL LETTER O WITH STROKE
  DefPrimitive!("\\o", "\u{00F8}"); // LATIN SMALL LETTER O WITH STROKE
  DefPrimitive!("\\ss", "\u{00DF}"); // LATIN SMALL LETTER SHARP S

  //======================================================================
  // TeX Book, Appendix B. p. 356

  def_primitive_noop("\\raggedright")?;
  def_primitive_noop("\\raggedleft")?; // this is actually LaTeX
  def_primitive_noop("\\ttraggedright")?;
  // \leavevmode moved to plain_bootstrap.rs (Perl plain_bootstrap.pool.ltxml L43)
  DefMacro!(
    "\\mathhexbox{}{}{}",
    r##"\leavevmode\hbox{$\m@th \mathchar"#1#2#3$}"##
  );
  // math_common + plain_constructs loaded after plain_base by tex.rs
  // (Perl: LoadFormat('plain') → plain_constructs → math_common)

  //======================================================================
  // TeX Book, Appendix B. p. 357
  // Perl: plain_base.pool.ltxml L537-543
  Let!("\\sp", T_SUPER!());
  Let!("\\sb", T_SUB!());
  Let!("\\:", "\\>");
  // Earlier (L423) `\<TAB>` was Let to `\<CR>`; this overrides it with a
  // 1em-wide NBSP Box. Perl: DefPrimitiveI("\\\t", undef, sub { Box(UTF(0xA0), ...) });
  DefPrimitive!("\\\t", {
    Tbox::new(
      arena::pin_static("\u{00A0}"),
      None,
      None,
      Tokens!(T_CS!("\\\t")),
      stored_map!("isSpace" => true, "width" => Dimension::from_str("1em")?),
    )
  });

  //----------------------------------------------------------------------
  def_primitive_noop("\\openup Dimension")?;

  // What should this do? (needs to work with alignments..)
  // see https://www.tug.org/TUGboat/tb07-1/tb14beet.pdf
  // Perl: DefMacro('\displaylines{}', '\halign{\hbox to\displaywidth{...}\crcr#1\crcr}')
  DefMacro!(
    "\\displaylines{}",
    r"\halign{\hbox to\displaywidth{$\hfil\displaystyle##\hfil$}\crcr#1\crcr}"
  );

  DefRegister!("\\pageno"   => Number::new(0));
  DefRegister!("\\headline" => Tokens!());
  DefRegister!("\\footline" => Tokens!());
  DefMacro!("\\folio", "1"); // What else?

  def_primitive_noop("\\nopagenumbers")?;
  DefMacro!("\\advancepageno", "\\advance\\pageno1\\relax");

  //======================================================================
  // TeX Book, Appendix B. p. 363
  def_primitive_noop("\\raggedbottom")?;
  def_primitive_noop("\\normalbottom")?;

  // Until we can do the "v" properly:
  DefMacro!("\\vfootnote", "\\footnote");
  DefMacro!(
    "\\fo@t",
    r"\ifcat\bgroup\noexpand\next \let\next\f@@t  \else\let\next\f@t\fi \next"
  );
  DefMacro!("\\f@@t", r"\bgroup\aftergroup\@foot\let\next");
  DefMacro!("\\f@t{}", r"#1\@foot");
  DefMacro!("\\@foot", r"\strut\egroup");

  def_primitive_noop("\\footstrut")?;
  DefRegister!("\\footins" => Number::new(0));

  def_primitive_noop("\\topinsert")?;
  def_primitive_noop("\\midinsert")?;
  def_primitive_noop("\\pageinsert")?;
  def_primitive_noop("\\endinsert")?;
  // \topins ?

  //======================================================================
  // TeX Book, Appendix B. p. 364

  // Let's hope nobody is messing with the output routine...
  def_primitive_noop("\\footnoterule")?;

  //======================================================================
  // End of TeX Book definitions.
  //======================================================================

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Stuff that probably doesn't belong here (LaTeX? or nowhere?)
  //DefMacro('\vspace{}', '\vskip#1\relax');

  //======================================================================
  // In principle, <ltx:emph> is a nice markup for emphasized.
  // Unfortunately, TeX really just treats it as a font switch.
  // Something like:  \em et.al. \rm more stuff
  // works in TeX, but in our case, since there is no explicit {},
  // the <ltx:emph> stays open!  Ugh!
  // This could still be made to work, but merge font would
  // need to look at any open <ltx:emph>, and then somehow close it!
  DefPrimitive!("\\em", None,
  before_digest => {
    let font = LookupFont!().unwrap();
    let shape = font.get_shape().unwrap_or(&Cow::Borrowed(""));
    let shapevariant = if shape == "italic" { "normal" } else { "italic" };
    AssignValue!("font", font.merge(fontmap!(shape => shapevariant)), Some(Scope::Local));
  });

  // Change math font while still in text!
  // Perl: AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold => 1), 'local')
  DefPrimitive!("\\boldmath", None,
    before_digest => {
      let mf = state::lookup_mathfont().unwrap_or_else(|| Rc::new(Font::math_default()));
      let merged = mf.merge(Font { forcebold: Some(true), ..Font::default() });
      state::assign_value("mathfont", Stored::Font(Rc::new(merged)), Some(Scope::Local));
    },
    forbid_math => true);
  // Perl: AssignValue(mathfont => LookupValue('mathfont')->merge(forcebold => 0), 'local')
  DefPrimitive!("\\unboldmath", None,
    before_digest => {
      let mf = state::lookup_mathfont().unwrap_or_else(|| Rc::new(Font::math_default()));
      let merged = mf.merge(Font { forcebold: Some(false), ..Font::default() });
      state::assign_value("mathfont", Stored::Font(Rc::new(merged)), Some(Scope::Local));
    },
    forbid_math => true);
});
// non_typewriter/non_typewriter_t1 moved to tex_fonts.rs (Perl: TeX_Fonts.pool.ltxml L338-344)
