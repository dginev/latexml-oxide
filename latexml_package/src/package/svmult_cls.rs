use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: svmult.cls.ltxml

  // Generally ignorable options
  for option in [
    "nospthms", "vecphys", "vecarrow", "norunningheads", "referee", "oribibl",
    "chaprefs", "footinfo", "openany", "sechang", "deutsch", "francais",
    // These could affect numbering...
    "numart", "book", "envcountresetchap", "envcountresetsect", "envcountsame",
    "envcountchap",
    "natbib",
  ].iter() {
    DeclareOption!(*option, None);
  }

  // Other options could load sv<option>.clo
  // Anything else gets passed to book.
  DeclareOption!(None, {
    Digest!("\\PassOptionsToClass{\\CurrentOption}{book}")?;
  });

  ProcessOptions!();
  // Perl svmult.cls.ltxml L34: LoadClass('book', withoptions => 1)
  load_class_with_options("book", Tokens!())?;
  RequirePackage!("sv_support");
  RequirePackage!("url");

  // \title with optional * to affect numbering (* => numart, none => book)
  DefMacro!("\\title OptionalMatch:* {}",
    "\\if.#1.\\else\\def\\thesection{\\arabic{section}}\\fi\
     \\lx@add@title{#2}");

  //======================================================================
  // Upright greek letters
  DefMath!("\\ualpha", "\u{03B1}", font => { shape => "upright", forceshape => true });
  DefMath!("\\ubeta",  "\u{03B2}", font => { shape => "upright", forceshape => true });
  DefMath!("\\uchi",   "\u{03C7}", font => { shape => "upright", forceshape => true });
  DefMath!("\\udelta", "\u{03B4}", font => { shape => "upright", forceshape => true });
  DefMath!("\\ugamma", "\u{03B3}", font => { shape => "upright", forceshape => true });
  DefMath!("\\umu",    "\u{03BC}", font => { shape => "upright", forceshape => true });
  DefMath!("\\unu",    "\u{03BD}", font => { shape => "upright", forceshape => true });
  DefMath!("\\upi",    "\u{03C0}", font => { shape => "upright", forceshape => true });
  DefMath!("\\utau",   "\u{03C4}", font => { shape => "upright", forceshape => true });

  // Italic var-Greek letters
  DefMath!("\\varDelta",   "\u{0394}", font => { shape => "italic" });
  DefMath!("\\varGamma",   "\u{0393}", font => { shape => "italic" });
  DefMath!("\\varLambda",  "\u{039B}", font => { shape => "italic" });
  DefMath!("\\varOmega",   "\u{03A9}", font => { shape => "italic" });
  DefMath!("\\varPhi",     "\u{03A6}", font => { shape => "italic" });
  DefMath!("\\varPi",      "\u{03A0}", font => { shape => "italic" });
  DefMath!("\\varPsi",     "\u{03A8}", font => { shape => "italic" });
  DefMath!("\\varSigma",   "\u{03A3}", font => { shape => "italic" });
  DefMath!("\\varTheta",   "\u{0398}", font => { shape => "italic" });
  DefMath!("\\varUpsilon", "\u{03A5}", font => { shape => "italic" });
  DefMath!("\\varXi",      "\u{039E}", font => { shape => "italic" });

  // Blackboard bold letters (identical \bbbX cluster to llncs_cls.rs).
  // Perl: DefPrimitiveI('\bbbc', undef, "\x{2102}");
  // DP audit flags 13 DefPrimitiveI↔DefConstructor mismatches here as
  // structural pattern — see llncs_cls.rs for the shared rationale
  // (Rust's DefConstructor is the idiomatic shape for literal-glyph
  // output with explicit horizontal-mode entry).
  DefConstructor!("\\bbbc",   "\u{2102}",   enter_horizontal => true);
  DefConstructor!("\\bbbf",   "\u{1D53D}",  enter_horizontal => true);
  DefConstructor!("\\bbbh",   "\u{210D}",   enter_horizontal => true);
  DefConstructor!("\\bbbk",   "\u{1D542}",  enter_horizontal => true);
  DefConstructor!("\\bbbm",   "\u{1D544}",  enter_horizontal => true);
  DefConstructor!("\\bbbn",   "\u{2115}",   enter_horizontal => true);
  DefConstructor!("\\bbbone", "\u{1D7D9}",  enter_horizontal => true);
  DefConstructor!("\\bbbp",   "\u{2119}",   enter_horizontal => true);
  DefConstructor!("\\bbbq",   "\u{211A}",   enter_horizontal => true);
  DefConstructor!("\\bbbr",   "\u{211D}",   enter_horizontal => true);
  DefConstructor!("\\bbbs",   "\u{1D54A}",  enter_horizontal => true);
  DefConstructor!("\\bbbt",   "\u{1D54B}",  enter_horizontal => true);
  DefConstructor!("\\bbbz",   "\u{2124}",   enter_horizontal => true);

  // Math operators
  DefMath!("\\getsto", "\u{21C6}", role => "ARROW");
  DefMath!("\\lid",    "\u{2266}", role => "RELOP", meaning => "less-than-or-equals");
  DefMath!("\\gid",    "\u{2267}", role => "RELOP", meaning => "greater-than-or-equals");
  DefMath!("\\grole",  "\u{2277}", role => "RELOP", meaning => "greater-than-or-less-than");
  Let!("\\qedsymbol", "\\qed");

  // Special signs and characters
  DefMacro!("\\D", "\\mathrm{d}");
  DefMacro!("\\E", "\\mathrm{e}");
  Let!("\\eul", "\\E");
  DefMacro!("\\I", "{\\rm i}");
  Let!("\\imag", "\\I");

  // Size and style macros
  DefMacro!("\\partsize",    "\\Large");
  DefMacro!("\\partstyle",   "\\bfseries\\boldmath");
  DefMacro!("\\chapsize",    "\\Large");
  DefMacro!("\\chapstyle",   "\\bfseries\\boldmath");
  DefMacro!("\\secsize",     "\\large");
  DefMacro!("\\secstyle",    "\\bfseries\\boldmath");
  DefMacro!("\\subsecsize",  "\\normalsize");
  DefMacro!("\\subsecstyle", "\\bfseries\\boldmath");

  def_macro_noop("\\chaptermark{}")?;
  def_macro_noop("\\sectionmark{}")?;
  def_macro_noop("\\subsectionmark{}")?;
  DefMacro!("\\tocauthorstyle",   "\\itshape");
  DefMacro!("\\toctitlestyle",    "\\bfseries");
  DefMacro!("\\tocaftauthskip",   "\\z@");

  DefMacro!("\\preface{}",   "\\chapter*{#1}");
  DefMacro!("\\prefacename", "Preface");

  DefMacro!("\\propositionname", "Proposition");

  // TOC registers
  DefRegister!("\\tocchpnum"         => Dimension::new(0));
  DefRegister!("\\tocsecnum"         => Dimension!("18pt"));
  DefRegister!("\\tocsectotal"       => Dimension::new(0));
  DefRegister!("\\tocsubsecnum"      => Dimension::new(0));
  DefRegister!("\\tocsubsectotal"    => Dimension::new(0));
  DefRegister!("\\tocsubsubsecnum"   => Dimension::new(0));
  DefRegister!("\\tocsubsubsectotal" => Dimension::new(0));
  DefRegister!("\\tocparanum"        => Dimension::new(0));
  DefRegister!("\\tocparatotal"      => Dimension::new(0));
  DefRegister!("\\tocsubparanum"     => Dimension::new(0));

  def_macro_noop("\\dominitoc")?;
  def_macro_noop("\\calctocindent")?;

  def_macro_noop("\\clearheadinfo")?;
  def_macro_noop("\\clearemptydoublepage")?;
  // Springer Multi-author authors use \orcid for ORCID identifier.
  // Witness 2408.17087, 2411.17645 (2 svmult papers).
  DefMacro!("\\orcid{}",
    "\\@add@frontmatter{ltx:note}[role=orcid]{#1}");
});
