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
  DefMacro!(
    "\\tracingall",
    "\\tracingonline=1 \\tracingcommands=2 \\tracingstats=2 \
     \\tracingpages=1 \\tracingoutput=1 \\tracinglostchars=1 \\tracingmacros=2 \
     \\tracingparagraphs=1 \\tracingrestores=1 \\showboxbreadth=\\maxdimen \
     \\showboxdepth=\\maxdimen \\errorstopmode"
  );
  DefMacro!("\\tracingnone", None);
  DefMacro!("\\hideoutput", None);

  // \choose, \brace, \brack moved to math_common.rs (Perl math_common.pool.ltxml L634-642)

  //======================================================================
  // Special Characters.
  // Try to give them some sense in math...
  //
  // Perl plain_base.pool.ltxml L70-77 defines `\#`, `\&`, `\%`, `\$`
  // (and `\_`) as single DefPrimitives whose sub body calls `Box(char,
  // font, undef, T_CS('\#'), role => 'ADDOP|POSTFIX|OPERATOR|…')` —
  // Perl's `Box` internally dispatches on mmode: emitting a Box in text
  // mode and an XMTok (with the attached role) in math mode.
  //
  // Rust splits each character into a trio: a DefMacro that `\ifmmode`-
  // dispatches to either `\lx@math@<name>` (DefMath with role) or
  // `\lx@text@<name>` (DefPrimitive emitting literal char). Kind-wise
  // the audit counts 4 DefPrimitive → DefMacro mismatches (# & % $);
  // the trio structure is more explicit than Perl's Box-dispatch but
  // observationally identical in both modes — same XMTok role + meaning
  // in math, same character in text.
  //
  // Intentional DefPrimitive → DefMacro kind divergence (WISDOM #44).
  // The explicit math/text split is idiomatic Rust — Perl's Box-
  // auto-XMTok-promotion has no direct equivalent in the Rust
  // Primitive API surface.
  DefMacro!("\\#", "\\ifmmode\\lx@math@hash\\else\\lx@text@hash\\fi");
  DefMacro!("\\&", "\\ifmmode\\lx@math@amp\\else\\lx@text@amp\\fi");
  DefMacro!(
    "\\%",
    "\\ifmmode\\lx@math@percent\\else\\lx@text@percent\\fi"
  );
  DefMacro!("\\$", "\\ifmmode\\lx@math@dollar\\else\\lx@text@dollar\\fi");
  DefMacro!(
    "\\_",
    "\\ifmmode\\lx@math@underscore\\else\\lx@text@underscore\\fi"
  );
  DefPrimitive!(T_CS!("\\lx@text@hash"), None, "#",  alias => "\\#");
  DefPrimitive!(T_CS!("\\lx@text@amp"), None, "&",  alias => "\\&");
  DefPrimitive!(T_CS!("\\lx@text@percent"), None, "%",  alias => "\\%");
  DefPrimitive!(T_CS!("\\lx@text@dollar"), None,  "$", alias => "\\$");
  DefPrimitive!(T_CS!("\\lx@text@underscore"), None, "_",  alias => "\\_");

  DefMath!("\\lx@math@hash",  None, "#", alias => "\\#");
  DefMath!("\\lx@math@amp",   None, "&", role  => "ADDOP", meaning => "and", alias => "\\&");
  DefMath!("\\lx@math@percent", None, "%", role  => "POSTFIX", meaning => "percent", alias => "\\%");
  DefMath!("\\lx@math@dollar", None, "\\$", role => "OPERATOR", meaning => "currency-dollar",
    alias => "\\$");
  DefMath!("\\lx@math@underscore", None, "_", alias => "\\_");

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
    use crate::engine::tex_tables::alignment_bindings;
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
    use crate::engine::tex_tables::alignment_bindings;
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
  DefMacro!("\\hidewidth", None);

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
  // From plain.tex
  DefPrimitive!("\\newcount DefToken", sub[(name)] {
    DefRegister!(name, None, Number::new(0), allocate=>"\\count");
  });
  DefPrimitive!("\\newdimen DefToken", sub[(name)] {
    DefRegister!(name, None, Dimension::new(0), allocate=>"\\dimen");
  });
  DefPrimitive!("\\newskip DefToken", sub[(name)] {
    DefRegister!(name, None, Glue::new(0), allocate=>"\\skip");
  });
  DefPrimitive!("\\newmuskip DefToken", sub[(name)] {
    DefRegister!(name, None, MuGlue::new(0), allocate=>"\\muskip");
  });
  AssignValue!("allocated_boxes" => 0);
  DefPrimitive!("\\newbox DefToken", sub[(t)] {
    let n = lookup_int("allocated_boxes");
    AssignValue!("allocated_boxes" => n + 1, Some(Scope::Global));
    // Don't store a value — a newly allocated box is void.
    // classify_box returns "" for None, making \ifvoid true.
    DefRegister!(t, None, Number(n), readonly => true);
  });
  // Perl plain_base.pool.ltxml L213:
  //   \outer\def\newhelp#1#2{\newtoks#1#1\expandafter{\csname#2\endcsname}}
  // allocates a \newtoks register so `#1` becomes defined, then stores
  // the help text. LaTeXML has no errhelp output, so the stored text
  // is irrelevant; what matters is that #1 is installed as a Toks
  // register, otherwise later `\errhelp\defbhelp@` reports undefined.
  // arxiv 1012.3836 (amstex.tex) was the witness.
  DefPrimitive!("\\newhelp DefToken {}", sub[(token, _arg)] {
    DefRegister!(token, None, Tokens!(), allocate => "\\toks");
  });
  DefPrimitive!("\\newtoks DefToken", sub[(name)] {
    DefRegister!(name, None, Tokens!(), allocate=>"\\toks");
  });

  // the next 4 actually work by doing a \chardef instead of \countdef, etc.
  // which means they actually work quite differently
  DefPrimitive!("\\alloc@@ {}", sub[(atype)] {
    let c = s!("allocation @{}", atype);
    let n = state::lookup_int(&c);
    state::assign_value(&c, n + 1, Some(Scope::Global));
    state::assign_register("\\allocationnumber", Number::new(n).into(), Some(Scope::Global), Vec::new())?;
  });
  DefMacro!(
    "\\newread DefToken",
    r"\alloc@@{read}\global\chardef#1=\allocationnumber"
  );
  DefMacro!(
    "\\newwrite DefToken",
    r"\alloc@@{write}\global\chardef#1=\allocationnumber"
  );
  DefMacro!(
    "\\newfam DefToken",
    r"\alloc@@{fam}\global\chardef#1=\allocationnumber"
  );
  DefMacro!(
    "\\newlanguage DefToken",
    r"\alloc@@{language}\global\chardef#1=\allocationnumber"
  );
  // \alloc@ moved to plain_bootstrap.rs (Perl plain_bootstrap.pool.ltxml L32)
  // Perl plain_base.pool.ltxml: \outer\def\newread{\alloc@6\read\chardef\sixt@@n}
  DefMacro!("\\newread", r"\alloc@6\read\chardef\sixt@@n");
  // Perl plain_base.pool.ltxml: \outer\def\newwrite{\alloc@7\write\chardef\sixt@@n}
  DefMacro!("\\newwrite", r"\alloc@7\write\chardef\sixt@@n");

  // This implementation is quite wrong
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

  DefPrimitive!("\\frenchspacing", None);
  DefPrimitive!("\\nonfrenchspacing", None);
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

  DefPrimitive!("\\endline", None);

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

  // Math spacing: medspace, thickspace, and negatives — Perl latex_constructs L2510-2525
  DefPrimitive!("\\medspace", {
    Tbox::new(
      pin!(""),
      None,
      None,
      Tokens!(T_CS!("\\medspace")),
      stored_map!("name" => "medspace", "width" => Dimension::from_str("0.22222em")?,
        "isSpace"=>true),
    )
  });
  DefPrimitive!("\\negmedspace", {
    Tbox::new(
      pin!(""),
      None,
      None,
      Tokens!(T_CS!("\\negmedspace")),
      stored_map!("name" => "negmedspace", "width" => Dimension::from_str("-0.22222em")?,
        "isSpace"=>true),
    )
  });
  DefPrimitive!("\\thickspace", {
    Tbox::new(
      arena::pin_static("\u{2004}"),
      None,
      None,
      Tokens!(T_CS!("\\thickspace")),
      stored_map!("name" => "thickspace", "width" => Dimension::from_str("0.27778em")?,
        "isSpace"=>true),
    )
  });
  DefPrimitive!("\\negthickspace", {
    Tbox::new(
      arena::pin_static("\u{2004}"),
      None,
      None,
      Tokens!(T_CS!("\\negthickspace")),
      stored_map!("name" => "negthickspace", "width" => Dimension::from_str("-0.27778em")?,
        "isSpace"=>true),
    )
  });

  // Perl: plain_base.pool.ltxml L447
  DefPrimitive!("\\hglue Glue", sub[(length)] {
    let s = dimension_to_spaces(length);
    if s.is_empty() { return Ok(Vec::new()); }
    Tbox::new(arena::pin(&s), None, None,
      Invocation!(T_CS!("\\hglue"), vec![length.revert()?]),
      stored_map!("name" => "hglue", "width" => length, "isSpace" => true))
  });
  DefPrimitive!("\\vglue Glue", None);
  DefPrimitive!("\\topglue", None);
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

  DefPrimitive!("\\break", None);
  DefPrimitive!("\\nobreak", None);
  DefPrimitive!("\\nobreakspace", {
    Tbox::new(
      arena::pin_static("\u{00A0}"),
      None,
      None,
      Tokens!(T_ACTIVE!('~')),
      stored_map!("isSpace" => true,
      "width" => Dimension::from_str("0.333em")?),
    )
  });
  // Perl: DefMacro(T_ACTIVE("~"), T_CS('\lx@NBSP'));
  DefMacro!(T_ACTIVE!('~'), None, "\\lx@NBSP");

  DefMacro!("\\slash", "/");
  DefPrimitive!("\\filbreak", None);
  DefMacro!("\\goodbreak", "\\par");
  DefPrimitive!("\\removelastskip", None);
  DefMacro!("\\smallbreak", "\\par");
  DefMacro!("\\medbreak", "\\par");
  DefMacro!("\\bigbreak", "\\par");
  DefMacro!("\\line", "\\hbox to \\hsize");

  // These should be 0 width, but perhaps also shifted?
  DefMacro!("\\llap{}", r"\hbox to 0pt{\hss#1}");
  DefMacro!("\\rlap{}", r"\hbox to 0pt{#1\hss}");
  DefMacro!("\\m@th", "\\mathsurround=0pt ");

  // \strutbox
  DefMacro!("\\strut", None);
  TeX!("\\newbox\\strutbox");
  //======================================================================
  // TeX Book, Appendix B. p. 354

  // Plain TeX tabbing — \settabs stub (no structured tabbing in plain)

  DefMacro!("\\settabs", None);
  //======================================================================
  // TeX Book, Appendix B. p. 355

  DefPrimitive!("\\hang", None);

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

  //======================================================================
  // TeX Book, Appendix B. p. 356

  DefPrimitive!("\\raggedright", None);
  DefPrimitive!("\\raggedleft", None); // this is actually LaTeX
  DefPrimitive!("\\ttraggedright", None);
  // \leavevmode moved to plain_bootstrap.rs (Perl plain_bootstrap.pool.ltxml L43)
  DefMacro!(
    "\\mathhexbox{}{}{}",
    r##"\leavevmode\hbox{$\m@th \mathchar"#1#2#3$}"##
  );
  // math_common + plain_constructs loaded after plain_base by tex.rs
  // (Perl: LoadFormat('plain') → plain_constructs → math_common)

  //----------------------------------------------------------------------
  DefPrimitive!("\\openup Dimension", None);

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

  DefPrimitive!("\\nopagenumbers", None);
  DefMacro!("\\advancepageno", "\\advance\\pageno1\\relax");

  //======================================================================
  // TeX Book, Appendix B. p. 363
  DefPrimitive!("\\raggedbottom", None);
  DefPrimitive!("\\normalbottom", None);

  // Until we can do the "v" properly:
  DefMacro!("\\vfootnote", "\\footnote");
  DefMacro!(
    "\\fo@t",
    r"\ifcat\bgroup\noexpand\next \let\next\f@@t  \else\let\next\f@t\fi \next"
  );
  DefMacro!("\\f@@t", r"\bgroup\aftergroup\@foot\let\next");
  DefMacro!("\\f@t{}", r"#1\@foot");
  DefMacro!("\\@foot", r"\strut\egroup");

  DefPrimitive!("\\footstrut", None);
  DefRegister!("\\footins" => Number::new(0));

  DefPrimitive!("\\topinsert", None);
  DefPrimitive!("\\midinsert", None);
  DefPrimitive!("\\pageinsert", None);
  DefPrimitive!("\\endinsert", None);
  // \topins ?

  //======================================================================
  // TeX Book, Appendix B. p. 364

  // Let's hope nobody is messing with the output routine...
  DefPrimitive!("\\footnoterule", None);

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
