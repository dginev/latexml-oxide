use latexml_package::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Functionally equivalent to https://arxiv.org/macros/equations.sty
  RequirePackage!("subeqn");

  // Perl equations.sty.ltxml L18: DefRegister('\@stequation' => Tokens()).
  DefRegister!("\\@stequation" => Tokens!());

  use latexml_package::engine::latex_constructs::{
    after_equation, before_equation, prepare_equation_counter, set_eqnarray_default_eqnsw, set_eqnarray_eqnsw,
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
      unread_one(T_CS!("\\@start@alignment"));
      Let!(T_MATH!(), "\\lx@dollar@in@mathmode");
    },
    after_digest_body => sub[whatsit] {
      after_equation(Some(whatsit))?;
      unread_one(T_CS!("\\@finish@alignment"));
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
      unread_one(T_CS!("\\@start@alignment"));
      Let!(T_MATH!(), "\\lx@dollar@in@mathmode");
    },
    after_digest_body => sub[whatsit] {
      after_equation(Some(whatsit))?;
      unread_one(T_CS!("\\@finish@alignment"));
    },
    locked => true
  );

  // Perl L62-124: eqalign, cases, eqaligntwo, eqaligntwo* (and `\@@eqncr` for eqaligntwo's raw `\halign`).
  // equations.sty:128-160 redefines eqnarray's begin (John Hobby's spacing) and `\@@eqncr` so that the rows start
  // with `\if@defeqnsw` — false in its eqnarray*, whose rows are unnumbered unless they hold `\yesnumber`. The
  // kernel's eqnarray is kept (its `\@@eqncr` numbering, sect07.rs) and given that switch
  // (`set_eqnarray_default_eqnsw`); the copy's raw `$$\halign` begin left the kernel's `\endeqnarray` closing a
  // display it had not opened (hep-ph9904268: 8 "\end@eqnarray" Errors, 16 equation references wrong; the
  // ar5iv-bindings Perl copy, equations.sty.ltxml:66, gives 7; KNOWN_PERL_ERRORS #1119). Guards
  // `perfect_kernel_batch78s::{equations_sty_eqnarray_numbers_as_latex, equations_sty_yesnumber_in_eqnarray_star}`.
  DefPrimitive!("\\lx@equations@unnumbered", {
    set_eqnarray_default_eqnsw(false);
  });
  DefPrimitive!("\\lx@equations@yesnumber", {
    set_eqnarray_eqnsw(true);
  });
  Let!("\\lx@equations@eqnarray", "\\eqnarray");
  DefMacro!("\\eqnarray", "\\if@defeqnsw\\else\\lx@equations@unnumbered\\fi\\lx@equations@eqnarray");
  // equations.sty:140-152 `\yesnumber[<number>]` prints `(<number>)` and leaves the counter alone.
  DefMacro!("\\yesnumber", "\\@ifnextchar[{\\@yesnumber}{\\global\\@eqnswtrue\\lx@equations@yesnumber}");
  DefMacro!("\\@yesnumber[]",
    "\\global\\@eqnswtrue\\lx@equations@yesnumber\\lx@equation@settag{\\def\\theequation{#1}\\lx@make@tags{equation}}");
  RawTeX!(
    r"\newif\if@defeqnsw \@defeqnswtrue
\@namedef{eqnarray*}{\@defeqnswfalse\eqnarray}
\@namedef{endeqnarray*}{\endeqnarray}

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
