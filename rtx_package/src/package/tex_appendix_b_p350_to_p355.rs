use crate::package::*;

LoadDefinitions!({
    //======================================================================
  // TeX Book, Appendix B, p. 350

  // Font stuff ...
  RawTeX!(
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
 \textfont3=\tenex");

  // Note: \newfam in math should be font switching(?)

  //======================================================================
  // TeX Book, Appendix B, p. 351

  // Old style font styles.
  // The trick is to create an empty Whatsit preserved till assimilation (for reversion'ing)
  // but to change the current font used in boxes.
  // (some of these were defined on different pages? or even latex...)
  Tag!("ltx:text", auto_open => true, auto_close => true);

  // Note that these, unlike \rmfamily, should set the other attributes to the defaults!
  DefPrimitive!("\\rm", None,
    font => {family => "serif", series => "medium", shape => "upright"});
  DefPrimitive!("\\sf", None,
    font => {family => "sansserif", series => "medium", shape => "upright"});
  DefPrimitive!("\\bf", None,
    font => {series => "bold", family => "serif", shape => "upright"});
  DefPrimitive!("\\it", None,
    font => {shape => "italic", family => "serif", series => "medium" });
  DefPrimitive!("\\tt", None,
    font => {family => "typewriter", series => "medium", shape => "upright" });
  // No effect in math for the following 2 ?
  DefPrimitive!("\\sl", None,
    font => {shape => "slanted", family => "serif", series => "medium" });
  DefPrimitive!("\\sc", None,
    font => {shape => "smallcaps", family => "serif", series => "medium" });

  // Ideally, we should set these sizes from class files
  AssignValue!("NOMINAL_FONT_SIZE", 10);
  DefPrimitive!("\\tiny",         None, font => {size => 5 });
  DefPrimitive!("\\scriptsize",   None, font => {size => 7 });
  DefPrimitive!("\\footnotesize", None, font => {size => 8 });
  DefPrimitive!("\\small",        None, font => {size => 9 });
  DefPrimitive!("\\normalsize",   None, font => {size => 10 });
  DefPrimitive!("\\large",        None, font => {size => 12 });
  DefPrimitive!("\\Large",        None, font => {size => 14.4 });
  DefPrimitive!("\\LARGE",        None, font => {size => 17.28 });
  DefPrimitive!("\\huge",         None, font => {size => 20.74 });
  DefPrimitive!("\\Huge",         None, font => {size => 29.8 });

  DefPrimitive!("\\mit", None, require_math => true, font => {family => "italic"});

  DefPrimitive!("\\frenchspacing", None);
  DefPrimitive!("\\nonfrenchspacing", None);
  DefMacro!("\\normalbaselines",
  r"\lineskip=\normallineskip\baselineskip=\normalbaselineskip\lineskiplimit=\normallineskiplimit");
  DefMacro!(T_CS!("\\space"), None, T_SPACE!());
  DefMacro!(T_CS!("\\lq"), None, T_OTHER!("`"));
  DefMacro!(T_CS!("\\rq"), None, T_OTHER!("'"));
  Let!("\\empty", "\\@empty");
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

  DefConstructor!("\\@break", "<ltx:break/>");

  RawTeX!(
    r"
  \def\loop#1\repeat{\def\body{#1}\iterate}
  \def\iterate{\body \let\next=\iterate \else\let\next=\relax\fi \next}
  \let\repeat=\fi
  ");

  DefPrimitive!("\\enskip", {
    Tbox::new(arena::pin_static("\u{2002}"), None, None, Tokens!(T_CS!("\\enskip")),
    stored_map!("name" => "enskip", "width" => Dimension::from_str("0.5em")?,
      "isSpace"=>true)) });

  DefPrimitive!("\\enspace", {
      Tbox::new(arena::pin_static("\u{2002}"), None, None, Tokens!(T_CS!("\\enspace")),
      stored_map!("name" => "enskip", "width" => Dimension::from_str("0.5em")?,
        "isSpace"=>true)) });

  DefPrimitive!("\\quad", {
      Tbox::new(arena::pin_static("\u{2003}"), None, None, Tokens!(T_CS!("\\quad")),
      stored_map!("name" => "quad", "width" => Dimension::from_str("1em")?,
        "isSpace"=>true)) });

  // Conceivably should be treated as punctuation! (but maybe even \quad should !?!)
  DefPrimitive!("\\qquad", {
      Tbox::new(arena::pin_static("\u{2003}\u{2003}"), None, None, Tokens!(T_CS!("\\qquad")),
      stored_map!("name" => "qquad", "width" => Dimension::from_str("2em")?,
        "isSpace"=>true, "asHint" => true)) });

  DefPrimitive!("\\thinspace", {
      Tbox::new(arena::pin_static("\u{2009}"), None, None, Tokens!(T_CS!("\\thinspace")),
      stored_map!("name" => "thinspace", "width" => Dimension::from_str("0.16667em")?,
        "isSpace"=>true)) });

  DefPrimitive!("\\negthinspace", {
      Tbox::new(arena::pin_static(""), None, None, Tokens!(T_CS!("\\negthinspace")),
      stored_map!("name" => "negthinspace", "width" => Dimension::from_str("-0.16667em")?,
        "isSpace"=>true)) });

  // DefConstructor('\hglue Glue', "?#isMath(<ltx:XMHint name='hglue' width='#width'/>)(\x{2003})",
  //   properties => sub { (stored_map!("isSpace"=>true), width => $_[1]) });
  DefPrimitive!("\\vglue Glue", None);
  DefPrimitive!("\\topglue", None);
  DefPrimitive!("\\nointerlineskip", None);
  DefPrimitive!("\\offinterlineskip", None);

  DefMacro!("\\smallskip", "\\vskip\\smallskipamount");
  DefMacro!("\\medskip", "\\vskip\\medskipamount");
  DefMacro!("\\bigskip", "\\vskip\\bigskipamount");

  //======================================================================
  // TeX Book, Appendix B, p. 353

  DefPrimitive!("\\break", None);
  DefPrimitive!("\\nobreak", None);
  DefPrimitive!("\\allowbreak", None);
  DefPrimitive!("\\nobreakspace", {
    Tbox::new(arena::pin_static("\u{00A0}"), None, None,
      Tokens!(T_ACTIVE!('~')), stored_map!("isSpace" => true,
      "width" => Dimension::from_str("0.333em")?))
  });
  DefMacro!(T_ACTIVE!('~'), None, "\\nobreakspace{}");

  DefMacro!("\\slash", "/");
  DefPrimitive!("\\filbreak", None);
  DefMacro!("\\goodbreak", "\\par");
  DefMacro!("\\eject", "\\par\\LTX@newpage");
  Let!("\\newpage", "\\eject");

  DefConstructor!("\\LTX@newpage", "^<ltx:pagination role='newpage'/>",
  before_digest=>{
    after_assignment();
    Ok(Vec::new())
  });
  DefMacro!("\\supereject", "\\par\\LTX@newpage");
  DefPrimitive!("\\removelastskip", None);
  DefMacro!("\\smallbreak", "\\par");
  DefMacro!("\\medbreak", "\\par");
  DefMacro!("\\bigbreak", "\\par");
  DefMacro!("\\line", "\\hbox to \\hsize");
  DefMacro!("\\leftline Undigested",   r"\ltx@leftline{\hbox{#1}}");
  DefMacro!("\\rightline Undigested",  r"\ltx@rightline{\hbox{#1}}");
  DefMacro!("\\centerline Undigested", r"\ltx@centerline{\hbox{#1}}");
  // TODO:
  //   DefConstructor('\ltx@leftline{}', sub {
  //     alignLine($_[0], $_[1], 'left'); },
  //   alias   => '\leftline',
  //   bounded => 1);
  // DefConstructor('\ltx@rightline{}', sub {
  //     alignLine($_[0], $_[1], 'right'); },
  //   alias   => '\rightline',
  //   bounded => 1);
  // DefConstructor('\ltx@centerline{}', sub {
  //     alignLine($_[0], $_[1], 'center'); },
  //   alias   => '\centerline',
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

  // These should be 0 width, but perhaps also shifted?
  DefMacro!("\\llap{}", r"\hbox to 0pt{\hss#1}");
  DefMacro!("\\rlap{}", r"\hbox to 0pt{#1\hss}");
  DefMacro!("\\m@th", "\\mathsurround=0pt ");

  // \strutbox
  DefMacro!("\\strut", None);
  RawTeX!("\\newbox\\strutbox");

  //======================================================================
  // TeX Book, Appendix B. p. 354

  // TODO: Not yet done!!
  // tabbing stuff!!!

  DefMacro!("\\settabs", None);

  //======================================================================
  // TeX Book, Appendix B. p. 355

  DefPrimitive!("\\hang", None);

  // TODO: \item, \itemitem not done!
  // This could probably be adopted from LaTeX, if the <itemize> could auto-open
  // and close!
  DefMacro!("\\hang",         r"\hangindent\parindent");
  DefMacro!("\\item",         r"\par\hang\textindent");
  DefMacro!("\\itemitem",     r"\par\indent \hangindent2\parindent \textindent");
  DefMacro!("\\textindent{}", r"\indent\llap{#1\enspace}\ignorespaces");
  DefMacro!("\\narrower", r"\advance\leftskip by\parindent\advance\rightskip by\parindent");

});
