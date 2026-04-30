use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Functionally equivalent to https://arxiv.org/macros/equations.sty
  RequirePackage!("subeqn");

  // Perl equations.sty.ltxml L18: DefRegister('\@stequation' => Tokens()).
  DefRegister!("\\@stequation" => Tokens!());

  use latexml_package::engine::latex_constructs::{
    after_equation, before_equation, prepare_equation_counter,
  };

  // Perl L20-40: {eqalignno} — display_math env wrapping an alignment,
  // numbered equations.
  DefEnvironment!(
    "{eqalignno}",
    "<ltx:equation xml:id='#id'>#tags<ltx:Math mode='display'>\
     <ltx:XMath>#body</ltx:XMath></ltx:Math></ltx:equation>",
    mode => "display_math",
    before_digest => {
      prepare_equation_counter(stored_map!("numbered" => true, "preset" => true));
      before_equation()?;
      gullet::unread_one(T_CS!("\\@start@alignment"));
      Let!(T_MATH!(), "\\lx@dollar@in@mathmode");
    },
    after_digest_body => sub[whatsit] {
      after_equation(Some(whatsit))?;
      gullet::unread_one(T_CS!("\\@finish@alignment"));
    },
    locked => true
  );

  // Perl L41-60: {eqalignno*} — same but no equation counter preset.
  DefEnvironment!(
    "{eqalignno*}",
    "<ltx:equation xml:id='#id'>#tags<ltx:Math mode='display'>\
     <ltx:XMath>#body</ltx:XMath></ltx:Math></ltx:equation>",
    mode => "display_math",
    before_digest => {
      before_equation()?;
      gullet::unread_one(T_CS!("\\@start@alignment"));
      Let!(T_MATH!(), "\\lx@dollar@in@mathmode");
    },
    after_digest_body => sub[whatsit] {
      after_equation(Some(whatsit))?;
      gullet::unread_one(T_CS!("\\@finish@alignment"));
    },
    locked => true
  );

  // Perl L62-124: Full RawTeX block — eqnarray, eqalign, cases, eqaligntwo,
  // plus eqaligntwo* \@namedef. Previous Rust stub shipped only a partial
  // extract.
  RawTeX!(
    r"\newif\if@defeqnsw \@defeqnswtrue

% John Hobby's version to fix up the spacing.
\def\eqnarray{\stepcounter{equation}\let\@currentlabel=\theequation
\if@defeqnsw\global\@eqnswtrue\else\global\@eqnswfalse\fi
\global\@eqnswtrue
\tabskip\@centering\let\\=\@eqncr
$$\halign to \displaywidth\bgroup\hfil\global\@eqcnt\z@
  $\displaystyle\tabskip\z@{##}$&\global\@eqcnt\@ne
  \hfil$\displaystyle{{}##{}}$\hfil
  &\global\@eqcnt\tw@ $\displaystyle{##}$\hfil
  \tabskip\@centering&\llap{##}\tabskip\z@\cr}

\def\yesnumber{\global\@eqnswtrue}

\def\@@eqncr{\let\@tempa\relax\global\advance\@eqcnt by \@ne
    \ifcase\@eqcnt \def\@tempa{& & & &}\or \def\@tempa{& & &}\or
     \def\@tempa{& &}\or \def\@tempa{&}\else\fi
     \@tempa \if@eqnsw\@eqnnum\stepcounter{equation}\fi
     \if@defeqnsw\global\@eqnswtrue\else\global\@eqnswfalse\fi
     \global\@eqcnt\z@\cr}

\def\@eqnacr{{\ifnum0=`}\fi\@ifstar{\@yeqnacr}{\@yeqnacr}}
\def\@yeqnacr{\@ifnextchar [{\@xeqnacr}{\@xeqnacr[\z@]}}
\def\@xeqnacr[#1]{\ifnum0=`{\fi}\cr \noalign{\vskip\jot\vskip #1\relax}}

\def\eqalign{\null\,\vcenter\bgroup\openup1\jot \m@th \let\\=\@eqnacr
\ialign\bgroup\strut
\hfil$\displaystyle{##}$&$\displaystyle{{}##}$\hfil\crcr}
\def\endeqalign{\crcr\egroup\egroup\,}

\def\cases{\left\{\,\vcenter\bgroup\normalbaselines\m@th \let\\=\@eqnacr
    \ialign\bgroup$##\hfil$&\quad##\hfil\crcr}
\def\endcases{\crcr\egroup\egroup\right.}

\def\eqaligntwo{\stepcounter{equation}\let\@currentlabel=\theequation
\if@defeqnsw\global\@eqnswtrue\else\global\@eqnswfalse\fi
\let\\=\@eqncr
$$\displ@y \tabskip\@centering \halign to \displaywidth\bgroup
  \global\@eqcnt\m@ne\hfil
  $\@lign\displaystyle{##}$\tabskip\z@skip&\global\@eqcnt\z@
  $\@lign\displaystyle{{}##}$\hfil\qquad&\global\@eqcnt\@ne
  \hfil$\@lign\displaystyle{##}$&\global\@eqcnt\tw@
  $\@lign\displaystyle{{}##}$\hfil\tabskip\@centering&
  \llap{\@lign##}\tabskip\z@skip\crcr}

\def\endeqaligntwo{\@@eqncr\egroup
      \global\advance\c@equation\m@ne$$\global\@ignoretrue}

\@namedef{eqaligntwo*}{\@defeqnswfalse\eqaligntwo}
\@namedef{endeqaligntwo*}{\endeqaligntwo}
"
  );
});
