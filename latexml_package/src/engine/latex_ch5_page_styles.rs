use crate::prelude::*;
LoadDefinitions!({
  //======================================================================
  // C.5.3 Page Styles
  //======================================================================
  // Ignored
  NewCounter!("page");
  DefMacro!("\\@mkboth", "\\@gobbletwo");
  DefMacro!("\\ps@empty",
    "\\let\\@mkboth\\@gobbletwo\\let\\@oddhead\\@empty\\let\\@oddfoot\\@empty\
     \\let\\@evenhead\\@empty\\let\\@evenfoot\\@empty");
  DefMacro!("\\ps@plain",
    "\\let\\@mkboth\\@gobbletwo\
     \\let\\@oddhead\\@empty\\def\\@oddfoot{\\reset@font\\hfil\\thepage\
     \\hfil}\\let\\@evenhead\\@empty\\let\\@evenfoot\\@oddfoot");
  Let!("\\@leftmark", "\\@firstoftwo");
  Let!("\\@rightmark", "\\@secondoftwo");

  DefPrimitive!("\\pagestyle{}", None);
  DefPrimitive!("\\thispagestyle{}", None);
  DefPrimitive!("\\markright{}", None);
  DefPrimitive!("\\markboth{}{}", None);
  DefPrimitive!("\\leftmark", None);
  DefPrimitive!("\\rightmark", None);
  DefPrimitive!("\\pagenumbering{}", None);
  DefMacro!("\\twocolumn[]", "#1");
  DefPrimitive!("\\onecolumn", None);

  // Style parameters from Fig. C.3, p.182
  DefRegister!("\\paperheight"     => Dimension!("11in"));
  DefRegister!("\\paperwidth"      => Dimension!("8.5in"));
  DefRegister!("\\textheight"      => Dimension!("550pt"));
  DefRegister!("\\textwidth"       => Dimension!("345pt"));
  DefRegister!("\\topmargin"       => Dimension::new(0));
  DefRegister!("\\headheight"      => Dimension::new(0));
  DefRegister!("\\headsep"         => Dimension::new(0));
  DefRegister!("\\footskip"        => Dimension::new(0));
  DefRegister!("\\footheight"      => Dimension::new(0));
  DefRegister!("\\evensidemargin"  => Dimension::new(0));
  DefRegister!("\\oddsidemargin"   => Dimension::new(0));
  DefRegister!("\\marginparwidth"  => Dimension::new(0));
  DefRegister!("\\marginparsep"    => Dimension::new(0));
  DefRegister!("\\columnwidth"     => Dimension!("345pt"));
  DefRegister!("\\linewidth"       => Dimension!("345pt"));
  DefRegister!("\\baselinestretch" => Dimension::new(0));

  TeX!(
    r"\def\@ifl@t@r#1#2{%
  \ifnum\expandafter\@parse@version@#1//00\@nil<%
        \expandafter\@parse@version@#2//00\@nil
    \expandafter\@secondoftwo
  \else
    \expandafter\@firstoftwo
  \fi}
\def\@parse@version@#1{\@parse@version0#1}
\def\@parse@version#1/#2/#3#4#5\@nil{%
\@parse@version@dash#1-#2-#3#4\@nil
}
\def\@parse@version@dash#1-#2-#3#4#5\@nil{%
  \if\relax#2\relax\else#1\fi#2#3#4 }
"
  );
});
