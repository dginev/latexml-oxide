use latexml_package::prelude::*;

LoadDefinitions!({
  // Functionally equivalent to https://arxiv.org/macros/equations.sty
  RequirePackage!("subeqn");
  // TODO: Perl defines {eqalignno} and {eqalignno*} DefEnvironments with
  // display_math mode, equation counters, and alignment support.
  // Also defines \eqnarray, \yesnumber, \eqalign, \cases, \eqaligntwo via RawTeX.
  // These are complex alignment environments that would need runtime support.
  // For now, load the raw TeX definitions.
  RawTeX!(
    r"\newif\if@defeqnsw \@defeqnswtrue

\def\yesnumber{\global\@eqnswtrue}

\def\@eqnacr{{\ifnum0=`}\fi\@ifstar{\@yeqnacr}{\@yeqnacr}}
\def\@yeqnacr{\@ifnextchar [{\@xeqnacr}{\@xeqnacr[\z@]}}
\def\@xeqnacr[#1]{\ifnum0=`{\fi}\cr \noalign{\vskip\jot\vskip #1\relax}}

\def\eqalign{\null\,\vcenter\bgroup\openup1\jot \m@th \let\\=\@eqnacr
\ialign\bgroup\strut
\hfil$\displaystyle{##}$&$\displaystyle{{}##}$\hfil\crcr}
\def\endeqalign{\crcr\egroup\egroup\,}

\def\cases{\left\{\,\vcenter\bgroup\normalbaselines\m@th \let\\=\@eqnacr
    \ialign\bgroup$##\hfil$&\quad##\hfil\crcr}
\def\endcases{\crcr\egroup\egroup\right.}"
  );
});
