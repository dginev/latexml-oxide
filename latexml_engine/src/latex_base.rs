//! latex_base — LaTeX kernel base definitions (infrastructure, no constructors)
//!
//! Perl: latex_base.pool.ltxml (865 lines)
//! Loaded BEFORE latex_dump in the Perl loading order.
//! Contains DefMacro, Let, DefPrimitive, DefRegister, DefConditional, RawTeX —
//! NO DefConstructor or DefEnvironment (those are in latex_constructs).
//!
//! This file collects base definitions that were previously scattered across
//! latex_other_in_appendices.rs, latex_ch*.rs, and latex_semi_undocumented.rs.
//! Definitions are ordered to match the Perl file's section structure.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  //======================================================================
  // C.0 Preliminaries & Shorthands
  // Perl: latex_base.pool.ltxml lines 31-114
  //======================================================================

  // \@empty — fundamental LaTeX macro, expands to nothing.
  // \lx@empty is already available from Base_Schema (loaded with TeX pool).
  Let!("\\@empty", "\\lx@empty");

  // RawTeX utilities — Perl: latex_base L55-64 + latex_other_in_appendices
  TeX!(
    r"
    \def\@namedef#1{\expandafter\def\csname #1\endcsname}
    \def\@nameuse#1{\csname #1\endcsname}
    \def\@cons#1#2{\begingroup\let\@elt\relax\xdef#1{#1\@elt #2}\endgroup}
    \let\@arrayparboxrestore\relax
    \def\@car#1#2\@nil{#1}
    \def\@cdr#1#2\@nil{#2}
    \def\@carcube#1#2#3#4\@nil{#1#2#3}
    \def\nfss@text#1{{\mbox{#1}}}
    \def\@sect#1#2#3#4#5#6[#7]#8{}
    "
  );

  // The following Perl-latex_constructs entries previously lived here
  // and were misplaced; all have been relocated to latex_constructs.rs
  // (Phase 25 cleanup, 2026-04-27):
  //   * `\@elt` (Perl L1018) — at latex_constructs.rs:3848
  //   * `\@begindocumenthook` (Perl L5510)
  //   * `\@qend`, `\@qrelax`, `\@spaces`, `\@sptoken` (Perl L5536-5539)
  //   * Float-page stubs (Perl L1015-1028): `\@topnewpage`, `\@next`,
  //     `\@xnext`, `\@freelist`, `\@currbox`, `\@toplist`, `\@botlist`,
  //     `\@midlist`, `\@currlist`, `\@deferlist`, `\@dbltoplist`,
  //     `\@dbldeferlist`, `\@startcolumn`
  //   * Aux file stubs (Perl L5796-5800): `\bibdata`, `\bibcite`,
  //     `\citation`, `\contentsline`, `\newlabel`
  //   * `\@preamblecmds` (Perl L5511)
  //   * `\nocorrlist`, `\text@command`, `\check@nocorr@`,
  //     `\ifmaybe@ic`, `\maybe@ic`, `\maybe@ic@`, `\sw@slant`,
  //     `\fix@penalty` (Perl L5814-5826)
  //   * `\@finalstrut` (Perl L4857)

  // Perl L35-36
  Let!("\\@pushfilename", "\\lx@pushfilename");
  Let!("\\@popfilename", "\\lx@popfilename");

  // Perl L38
  DefMacro!("\\@ehc", "I can't help");

  // Perl L40-47: gobble/firstof/secondof macros
  DefMacro!("\\@gobble{}", None);
  DefMacro!("\\@gobbletwo{}{}", None);
  DefMacro!("\\@gobblefour{}{}{}{}", None);
  // Perl latex.ltx uses `\long\def\@firstofone#1{#1}` etc., overriding the
  // closure-version defined in latex_base.pool.ltxml L46-48. The dump
  // therefore captures these as token-list bodies (see latex_dump.pool.ltxml
  // L3771 `\@thirdofthree T(A(3))`). Using token-list form here matches
  // Perl's end-state AND lets these CSes survive dump-only mode dump loading.
  DefMacro!("\\@firstofone{}", "#1");
  Let!("\\@iden", "\\@firstofone");
  DefMacro!("\\@firstoftwo{}{}", "#1");
  DefMacro!("\\@secondoftwo{}{}", "#2");
  DefMacro!("\\@thirdofthree{}{}{}", "#3");
  // Perl L48: `\@expandtwoargs{}{}{}` — closure body. Closures can't be
  // serialized into the dump, but `_base.rs` is always loaded before
  // the dump, so the closure is always available. Dump's add-only
  // policy then skips any same-named dump entry.
  DefMacro!("\\@expandtwoargs{}{}{}", sub[(first,second,third)] {
    let mut tks = first.unlist();
    tks.push(T_BEGIN!());
    tks.append(&mut Expand!(second).unlist());
    tks.push(T_END!());
    tks.push(T_BEGIN!());
    tks.append(&mut Expand!(third).unlist());
    tks.push(T_END!());
    tks });

  // Perl L50-52: `\@makeother` — closure.
  DefMacro!("\\@makeother {}", sub[(arg)] {
    let arg_str = arg.to_string();
    let mut arg_chars = arg_str.chars();
    let arg_c = match arg_chars.next() {
      Some('\\') => arg_chars.next().unwrap(),
      Some(other) => other,
      None => {
        Warn!("expected","character","\\@makeother called on empty argument?");
        return Ok(Tokens!());
      }};
    assign_catcode(arg_c, Catcode::OTHER, Some(Scope::Local));
  });

  // Perl L55-64: @namedef, @nameuse, @cons, @car, @cdr, obeycr/restorecr
  TeX!(
    r"{\catcode`\^^M=13 \gdef\obeycr{\catcode`\^^M13 \def^^M{\\\relax}%
    \@gobblecr}%
    {\catcode`\^^M=13 \gdef\@gobblecr{\@ifnextchar
    \@gobble\ignorespaces}}%
    \gdef\restorecr{\catcode`\^^M5 }}"
  );

  // Perl L73-90: strip@pt, sanitize, dospecials
  TeX!(
    r"\begingroup
  \catcode`P=12
  \catcode`T=12
  \lowercase{
    \def\x{\def\rem@pt##1.##2PT{##1\ifnum##2>\z@.##2\fi}}}
  \expandafter\endgroup\x
  \def\strip@pt{\expandafter\rem@pt\the}
  \def\strip@prefix#1>{}
  \def\@sanitize{\@makeother\ \@makeother\\\@makeother\$\@makeother\&%
  \@makeother\#\@makeother\^\@makeother\_\@makeother\%\@makeother\~}
  \def \@onelevel@sanitize #1{%
    \edef #1{\expandafter\strip@prefix
            \meaning #1}%
  }
  \def\dospecials{\do\ \do\\\do\{\do\}\do\$\do\&%
    \do\#\do\^\do\_\do\%\do\~}"
  );

  // Perl L92-114: \nfss@catcodes
  DefMacro!(
    "\\nfss@catcodes",
    r###"\makeatletter
    \catcode`\ 9%
    \catcode`\^^I9%
    \catcode`\^^M9%
    \catcode`\\\z@
    \catcode`\{\@ne
    \catcode`\}\tw@
    \catcode`\#6%
    \catcode`\^7%
    \catcode`\%14%
    \@makeother\<%
    \@makeother\>%
    \@makeother\*%
    \@makeother\.%
    \@makeother\-%
    \@makeother\/%
    \@makeother\[%
    \@makeother\]%
    \@makeother\`%
    \@makeother\'%
    \@makeother\"%
    "###
  );

  // Perl L116-127: dimension/skip shorthands + special chars
  DefMacro!("\\@height", None, "height");
  DefMacro!("\\@width", None, "width");
  DefMacro!("\\@depth", None, "depth");
  DefMacro!("\\@minus", None, "minus");
  DefMacro!("\\@plus", None, "plus");
  DefMacro!("\\hb@xt@", None, "\\hbox to");
  DefMacro!("\\hmode@bgroup", None, "\\leavevmode\\bgroup");

  DefMacro!(T_CS!("\\@backslashchar"), None, T_OTHER!("\\"));
  DefMacro!(T_CS!("\\@percentchar"), None, T_OTHER!("%"));
  DefMacro!(T_CS!("\\@charlb"), None, T_LETTER!("{"));
  DefMacro!(T_CS!("\\@charrb"), None, T_LETTER!("}"));

  // Perl L129-153: font size macros
  DefMacro!(T_CS!("\\@vpt"), None, T_OTHER!("5"));
  DefMacro!(T_CS!("\\@vipt"), None, T_OTHER!("6"));
  DefMacro!(T_CS!("\\@viipt"), None, T_OTHER!("7"));
  DefMacro!(T_CS!("\\@viiipt"), None, T_OTHER!("8"));
  DefMacro!(T_CS!("\\@ixpt"), None, T_OTHER!("9"));
  DefMacro!("\\@xpt", "10");
  DefMacro!("\\@xipt", "10.95");
  DefMacro!("\\@xiipt", "12");
  DefMacro!("\\@xivpt", "14.4");
  DefMacro!("\\@xviipt", "17.28");
  DefMacro!("\\@xxpt", "20.74");
  DefMacro!("\\@xxvpt", "24.88");
  // LaTeX 209 size aliases
  DefMacro!("\\vpt", r"\edef\f@size{\@vpt}\rm");
  DefMacro!("\\vipt", r"\edef\f@size{\@vipt}\rm");
  DefMacro!("\\viipt", r"\edef\f@size{\@viipt}\rm");
  DefMacro!("\\viiipt", r"\edef\f@size{\@viiipt}\rm");
  DefMacro!("\\ixpt", r"\edef\f@size{\@ixpt}\rm");
  DefMacro!("\\xpt", r"\edef\f@size{\@xpt}\rm");
  DefMacro!("\\xipt", r"\edef\f@size{\@xipt}\rm");
  DefMacro!("\\xiipt", r"\edef\f@size{\@xiipt}\rm");
  DefMacro!("\\xivpt", r"\edef\f@size{\@xivpt}\rm");
  DefMacro!("\\xviipt", r"\edef\f@size{\@xviipt}\rm");
  DefMacro!("\\xxpt", r"\edef\f@size{\@xxpt}\rm");
  DefMacro!("\\xxvpt", r"\edef\f@size{\@xxvpt}\rm");

  //======================================================================
  // C.1.3 Fragile Commands
  // Perl: latex_base.pool.ltxml lines 177-237
  //======================================================================
  TeX!(
    r"
\def\@ignorefalse{\global\let\if@ignore\iffalse}
\def\@ignoretrue {\global\let\if@ignore\iftrue}
\def\zap@space#1 #2{%
  #1%
  \ifx#2\@empty\else\expandafter\zap@space\fi
  #2}
\def\@unexpandable@protect{\noexpand\protect\noexpand}
\def\x@protect#1{%
   \ifx\protect\@typeset@protect\else
      \@x@protect#1%
   \fi
}
\def\@x@protect#1\fi#2#3{%
   \fi\protect#1%
}
\let\@typeset@protect\relax
\def\set@display@protect{\let\protect\string}
\def\set@typeset@protect{\let\protect\@typeset@protect}
\def\protected@edef{%
   \let\@@protect\protect
   \let\protect\@unexpandable@protect
   \afterassignment\restore@protect
   \edef
}
\def\protected@xdef{%
   \let\@@protect\protect
   \let\protect\@unexpandable@protect
   \afterassignment\restore@protect
   \xdef
}
\def\unrestored@protected@xdef{%
   \let\protect\@unexpandable@protect
   \xdef
}
\def\restore@protect{\let\protect\@@protect}
\set@typeset@protect
\def\@nobreakfalse{\global\let\if@nobreak\iffalse}
\def\@nobreaktrue {\global\let\if@nobreak\iftrue}
\@nobreakfalse

\newif\ifv@
\newif\ifh@
\newif\ifdt@p
\newif\if@pboxsw
\newif\if@rjfield
\newif\if@firstamp
\newif\if@negarg
\newif\if@ovt
\newif\if@ovb
\newif\if@ovl
\newif\if@ovr
\newdimen\@ovxx
\newdimen\@ovyy
\newdimen\@ovdx
\newdimen\@ovdy
\newdimen\@ovro
\newdimen\@ovri
\newif\if@noskipsec \@noskipsectrue
"
  );

  //======================================================================
  // C.3. Sentences and Paragraphs
  // Perl: latex_base.pool.ltxml lines 248-277
  //======================================================================
  // C.3.1 Making Sentences (Perl L255-256)
  DefMacro!("\\fmtname", "LaTeX2e");
  DefMacro!("\\fmtversion", "2018/12/01");

  // C.3.2 Making Paragraphs (Perl L261-263)
  Let!("\\@@par", "\\par");
  DefMacro!("\\@par", r"\let\par\@@par\par");
  DefMacro!("\\@restorepar", r"\def\par{\@par}");

  // C.3.3 Footnotes (Perl L268-273)
  NewCounter!("footnote");
  DefMacro!("\\thefootnote", "\\arabic{footnote}");
  NewCounter!("mpfootnote");
  DefMacro!("\\thempfn", "\\thefootnote");
  DefMacro!("\\thempfootnote", "\\arabic{mpfootnote}");
  DefRegister!("\\footnotesep" => Dimension::new(0));

  //======================================================================
  // C.4 Sectioning and Table of Contents
  // Perl: latex_base.pool.ltxml lines 279-300
  //======================================================================
  // C.4.2 The Appendix (Perl L287-288)
  // `\appendixname` is also in Perl latex_constructs L5783 (Perl-faithful
  // dup); Rust mirrors with a 2nd entry in latex_constructs.rs (~L9055).
  DefMacro!("\\appendixname", "Appendix");
  DefMacro!("\\appendixesname", "Appendixes");

  // C.4.3 Table of Contents — label macros (Perl L294-296)
  DefMacro!("\\contentsname", "Contents");
  DefMacro!("\\listfigurename", "List of Figures");
  DefMacro!("\\listtablename", "List of Tables");

  // C.4.4 Style registers (Perl L300)
  NewCounter!("tocdepth");

  // C.5.1 Document Class — page registers (Perl L309-311)
  DefRegister!("\\columnsep"     => Dimension::new(0));
  DefRegister!("\\columnseprule" => Dimension::new(0));
  DefRegister!("\\mathindent"    => Dimension::new(0));
  // C.5.1 secnumdepth (Perl L312)
  NewCounter!("secnumdepth");

  // C.5.2 Packages — version parsing helpers (Perl L317-331)
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
  \if\relax#2\relax\else#1\fi#2#3#4 }"
  );

  //======================================================================
  // C.5 Classes, Packages and Page Styles
  // Perl: latex_base.pool.ltxml lines 302-347
  //======================================================================
  // \columnsep, \columnseprule, \mathindent — still in latex_constructs.rs (C.5)
  // NewCounter('secnumdepth') — still in latex_constructs.rs (C.5)

  // C.5.4 Title Page mark stubs (Perl L343-347)
  DefMacro!("\\sectionmark{}", "");
  DefMacro!("\\subsectionmark{}", "");
  DefMacro!("\\subsubsectionmark{}", "");
  DefMacro!("\\paragraphmark{}", "");
  DefMacro!("\\subparagraphmark{}", "");

  //======================================================================
  // C.8.1 Defining Commands
  // Perl: latex_base.pool.ltxml lines 350-368
  //======================================================================
  // Perl L357
  DefMacro!("\\@tabacckludge {}", "\\csname\\string#1\\endcsname");

  // Perl L359-368: DeclareTextAccent family (no-op stubs)
  DefPrimitive!("\\DeclareTextAccent DefToken {}{}", None);
  DefPrimitive!("\\DeclareTextAccentDefault{}{}", None);
  DefPrimitive!("\\DeclareTextComposite{}{}{}{}", None);
  DefPrimitive!("\\DeclareTextCompositeCommand{}{}{}{}", None);

  //======================================================================
  // C.9.1 Figures and Tables — float parameters
  // Perl: latex_base.pool.ltxml lines 384-417
  //======================================================================
  // Perl L391-392
  DefPrimitive!("\\flushbottom",      None);
  DefPrimitive!("\\suppressfloats[]", None);

  // Perl L394-403: float counters and fractions
  NewCounter!("topnumber");
  DefMacro!("\\topfraction", "0.25");
  NewCounter!("bottomnumber");
  DefMacro!("\\bottomfraction", "0.25");
  NewCounter!("totalnumber");
  DefMacro!("\\textfraction", "0.25");
  DefMacro!("\\floatpagefraction", "0.25");
  NewCounter!("dbltopnumber");
  DefMacro!("\\dbltopfraction", "0.7");
  DefMacro!("\\dblfloatpagefraction", "0.25");

  // Perl L404-414: float separators and extents
  DefRegister!("\\floatsep"        => Glue!("12.0pt plus 2.0pt minus 2.0pt"));
  DefRegister!("\\textfloatsep"    => Glue!("20.0pt plus 2.0pt minus 4.0pt"));
  DefRegister!("\\intextsep"       => Glue!("12.0pt plus 2.0pt minus 2.0pt"));
  DefRegister!("\\dblfloatsep"     => Glue!("12.0pt plus 2.0pt minus 2.0pt"));
  DefRegister!("\\dbltextfloatsep" => Glue!("20.0pt plus 2.0pt minus 4.0pt"));
  DefRegister!("\\@fptop"          => Glue::new(0));
  DefRegister!("\\@fpsep"          => Glue::new(0));
  DefRegister!("\\@fpbot"          => Glue::new(0));
  DefRegister!("\\@dblfptop"       => Glue::new(0));
  DefRegister!("\\@dblfpsep"       => Glue::new(0));
  DefRegister!("\\@dblfpbot"       => Glue::new(0));

  // Perl L415-417: figure rules (Lets to \relax)
  Let!("\\topfigrule", "\\relax");
  Let!("\\botfigrule", "\\relax");
  Let!("\\dblfigrule", "\\relax");

  //======================================================================
  // C.11.4 Splitting the input / C.13 Boxes
  // Perl: latex_base.pool.ltxml lines 454-486
  //======================================================================
  // \DeclareRobustCommand (Perl L454-456) — closure body uses
  // convert_latex_args from prelude
  DefPrimitive!("\\DeclareRobustCommand OptionalMatch:* SkipSpaces DefToken [Number][]{}",
  sub[(_star,cs,nargs,opt,body)] {
    let nargs = nargs.value_of() as usize;
    let cs_args = convert_latex_args(nargs, opt)?;
    DefMacro!(cs, cs_args, body, robust => true);
  });

  // savebox RawTeX block (Perl L457-486)
  TeX!(
    r#"""\def\newsavebox#1{\@ifdefinable{#1}{\newbox#1}}
  \DeclareRobustCommand\savebox[1]{%
    \@ifnextchar(%)
      {\@savepicbox#1}{\@ifnextchar[{\@savebox#1}{\sbox#1}}}%
  \DeclareRobustCommand\sbox[2]{\setbox#1\hbox{%
    \color@setgroup#2\color@endgroup}}
  \def\@savebox#1[#2]{%
    \@ifnextchar [{\@isavebox#1[#2]}{\@isavebox#1[#2][c]}}
  \long\def\@isavebox#1[#2][#3]#4{%
    \sbox#1{\@imakebox[#2][#3]{#4}}}
  \def\@savepicbox#1(#2,#3){%
    \@ifnextchar[%]
      {\@isavepicbox#1(#2,#3)}{\@isavepicbox#1(#2,#3)[]}}
  \long\def\@isavepicbox#1(#2,#3)[#4]#5{%
    \sbox#1{\@imakepicbox(#2,#3)[#4]{#5}}}
  \def\lrbox#1{%
    \edef\reserved@a{%
      \endgroup
      \setbox#1\hbox{%
        \begingroup\aftergroup}%
          \def\noexpand\@currenvir{\@currenvir}%
          \def\noexpand\@currenvline{\on@line}}%
    \reserved@a
      \@endpefalse
      \color@setgroup
        \ignorespaces}
  \def\endlrbox{\unskip\color@endgroup}
  \DeclareRobustCommand\usebox[1]{\leavevmode\copy #1\relax}
  """#
  );

  //======================================================================
  // Error/Warning/Info infrastructure
  // Perl: latex_base.pool.ltxml lines 516-593
  //======================================================================
  // `\ltx@hard@MessageBreak` moved to `latex_constructs_rust_only.rs`
  // (Rust-only; not in Perl latex_*.pool.ltxml).
  // Perl-parity: `\@onlypreamble`, `\GenericError/Warning/Info` are
  // closure-backed primitives defined in `latex_constructs.pool.ltxml`
  // (L5645-5648), not latex_base. Relocated there 2026-04-18 so they
  // survive the dump/base mutual-exclusivity flip.

  Let!("\\MessageBreak", "\\relax");
  TeX!(
    r"\gdef\PackageError#1#2#3{%
       \GenericError{%
           (#1)\@spaces\@spaces\@spaces\@spaces
        }{%
           Package #1 Error: #2%
        }{%
           See the #1 package documentation for explanation.%
        }{#3}%
     }
     \def\PackageWarning#1#2{%
       \GenericWarning{%
           (#1)\@spaces\@spaces\@spaces\@spaces
        }{%
           Package #1 Warning: #2%
        }%
     }
     \def\PackageWarningNoLine#1#2{%
       \PackageWarning{#1}{#2\@gobble}}
     \def\PackageInfo#1#2{%
       \GenericInfo{%
           (#1) \@spaces\@spaces\@spaces
        }{%
           Package #1 Info: #2%
        }%
     }
     \def\PackageNote#1#2{%
       \GenericWarning{%
           (#1) \@spaces\@spaces\@spaces
        }{%
           Package #1 Info: #2%
        }%
     }
     \def\PackageNoteNoLine#1#2{\PackageNote{#1}{#2\@gobble}}
     \def\ClassError#1#2#3{%
       \GenericError{%
           (#1) \space\@spaces\@spaces\@spaces
        }{%
           Class #1 Error: #2%
        }{%
           See the #1 class documentation for explanation.%
        }{#3}%
     }
     \def\ClassWarning#1#2{%
       \GenericWarning{%
           (#1) \space\@spaces\@spaces\@spaces
        }{%
           Class #1 Warning: #2%
        }%
     }
     \def\ClassWarningNoLine#1#2{%
       \ClassWarning{#1}{#2\@gobble}}
     \def\ClassInfo#1#2{%
       \GenericInfo{%
           (#1) \space\space\@spaces\@spaces
        }{%
           Class #1 Info: #2%
        }%
     }
     \def\@latex@error#1#2{%
       \GenericError{%
           \space\space\space\@spaces\@spaces\@spaces
        }{%
           LaTeX Error: #1%
        }{%
           See the LaTeX manual or LaTeX Companion for explanation.%
        }{#2}%
     }
     \def\@latex@warning#1{%
       \GenericWarning{%
           \space\space\space\@spaces\@spaces\@spaces
        }{%
           LaTeX Warning: #1%
        }%
     }
     \def\@latex@warning@no@line#1{%
       \@latex@warning{#1\@gobble}}
     \def\@latex@info#1{%
       \GenericInfo{%
           \@spaces\@spaces\@spaces
        }{%
           LaTeX Info: #1%
        }%
     }
     \def\@latex@info@no@line#1{%
       \@latex@info{#1\@gobble}}
     "
  );
  // `\hexnumber@`, `\on@line`, `\@warning`, `\@@warning`,
  // `\G@refundefinedtrue`, `\@nomath`, `\@font@warning` moved to
  // latex_constructs.rs (Perl L5653-5666). `\@latexbug` moved to
  // `latex_constructs_rust_only.rs`.

  //======================================================================
  // Perl: latex_base.pool.ltxml lines 601-608
  // Math chardef constants and fontenc load list
  //======================================================================
  TeX!(
    r"\chardef\@xxxii=32
    \mathchardef\@Mi=10001
    \mathchardef\@Mii=10002
    \mathchardef\@Miii=10003
    \mathchardef\@Miv=10004
    \def\@fontenc@load@list{\@elt{T1}\@elt{OT1}}"
  );

  // Perl L622-628: temp macros and script ratios
  DefMacro!("\\@tempa", None);
  DefMacro!("\\@tempb", None);
  DefMacro!("\\@tempc", None);
  DefMacro!("\\@gtempa", None);

  DefMacro!("\\defaultscriptratio", None, ".7");
  DefMacro!("\\defaultscriptscriptratio", None, ".5");

  //======================================================================
  // Perl: latex_base.pool.ltxml lines 630-802
  // Large RawTeX block: registers, conditionals, iteration, lists
  //======================================================================
  TeX!(
    r"
    \long\def\loop#1\repeat{%
      \def\iterate{#1\relax\expandafter\iterate\fi}%
      \iterate%
      \let\iterate\relax}
    \newdimen\@ydim
    \let\@@hyph=\-
    \newbox\@arstrutbox
    \newbox\@begindvibox
    \newcount\@botnum
    \newdimen\@botroom
    \newcount\@chclass
    \newcount\@chnum
    \newdimen\@clnht
    \newdimen\@clnwd
    \newdimen\@colht
    \newcount\@colnum
    \newdimen\@colroom
    \newbox\@curfield
    \newbox\@curline
    \newcount\@currtype
    \newcount\@curtab
    \newcount\@curtabmar
    \newbox\@dashbox
    \newcount\@dashcnt
    \newdimen\@dashdim
    \newcount\@dbltopnum
    \newdimen\@dbltoproom
    \let\@dischyph=\-
    \newcount\@enumdepth
    \newcount\@floatpenalty
    \newdimen\@fpmin
    \newcount \@fpstype
    \newcount\@highpenalty
    \newcount\@hightab
    \newbox\@holdpg
    \newinsert \@kludgeins
    \newcount\@lastchclass
    \newbox\@leftcolumn
    \newbox\@linechar
    \newdimen\@linelen
    \newcount\@lowpenalty
    \newdimen\@maxdepth
    \newcount\@medpenalty
    \newdimen\@mparbottom \@mparbottom\z@
    \newinsert\@mpfootins
    \newcount\@mplistdepth
    \newcount\@multicnt
    \newcount\@nxttabmar
    \newbox\@outputbox
    \newdimen\@pagedp
    \newdimen\@pageht
    \newbox\@picbox
    \newdimen\@picht
    \newdimen \@reqcolroom
    \newskip\@rightskip \@rightskip \z@skip
    \newcount\@savsf
    \newdimen\@savsk
    \newcount\@secpenalty
    \def\@sqrt[#1]{\root #1\of}
    \newbox\@tabfbox
    \newcount\@tabpush
    \newdimen \@textfloatsheight
    \newdimen\@textmin
    \newcount\@topnum
    \newdimen\@toproom
    \newcount\@xarg
    \newdimen\@xdim
    \newcount\@yarg
    \newdimen\@ydim
    \newcount\@yyarg
    \newtoks\every@math@size
    \newif \if@fcolmade
    \newdimen\lower@bound
    \newcount\par@deathcycles
    \newdimen\upper@bound
    \newif\if@insert
    \newif\if@colmade
    \newif\if@specialpage   \@specialpagefalse
    \newif\if@firstcolumn   \@firstcolumntrue
    \newif\if@twocolumn     \@twocolumnfalse
    \newif\if@twoside       \@twosidefalse
    \newif\if@reversemargin \@reversemarginfalse
    \newif\if@mparswitch    \@mparswitchfalse
    \newcount\col@number    \@ne
    \newread\@inputcheck
    \newwrite\@unused
    \newwrite\@mainaux
    \newwrite\@partaux
    \let\@auxout=\@mainaux
    \openout\@mainaux\jobname.aux
    \newcount\@clubpenalty
    \@clubpenalty \clubpenalty
    \newif\if@filesw \@fileswtrue
    \newif\if@partsw \@partswfalse
    \def\@tempswafalse{\let\if@tempswa\iffalse}
    \def\@tempswatrue{\let\if@tempswa\iftrue}
    \let\if@tempswa\iffalse
    \newcount\@tempcnta
    \newcount\@tempcntb
    \newif\if@tempswa
    \newdimen\@tempdima
    \newdimen\@tempdimb
    \newdimen\@tempdimc
    \newbox\@tempboxa
    \newskip\@tempskipa
    \newskip\@tempskipb
    \newtoks\@temptokena
    \newskip\@flushglue \@flushglue = 0pt plus 1fil
    \newif\if@afterindent\@afterindenttrue
    \newbox\rootbox

    \newcount\@eqcnt
    \newcount\@eqpen
    \newif\if@eqnsw\@eqnswtrue
    \newskip\@centering
    \@centering = 0pt plus 1000pt
    \let\@eqnsel=\relax

     \long\def\@whilenum#1\do #2{\ifnum #1\relax #2\relax\@iwhilenum{#1\relax
          #2\relax}\fi}
     \long\def\@iwhilenum#1{\ifnum #1\expandafter\@iwhilenum
              \else\expandafter\@gobble\fi{#1}}
     \long\def\@whiledim#1\do #2{\ifdim #1\relax#2\@iwhiledim{#1\relax#2}\fi}
     \long\def\@iwhiledim#1{\ifdim #1\expandafter\@iwhiledim
             \else\expandafter\@gobble\fi{#1}}
     \long\def\@whilesw#1\fi#2{#1#2\@iwhilesw{#1#2}\fi\fi}
     \long\def\@iwhilesw#1\fi{#1\expandafter\@iwhilesw
              \else\@gobbletwo\fi{#1}\fi}
    \def\@nnil{\@nil}
    \def\@fornoop#1\@@#2#3{}
    \long\def\@for#1:=#2\do#3{%
      \expandafter\def\expandafter\@fortmp\expandafter{#2}%
      \ifx\@fortmp\@empty \else
        \expandafter\@forloop#2,\@nil,\@nil\@@#1{#3}\fi}
    \long\def\@forloop#1,#2,#3\@@#4#5{\def#4{#1}\ifx #4\@nnil \else
           #5\def#4{#2}\ifx #4\@nnil \else#5\@iforloop #3\@@#4{#5}\fi\fi}
    \long\def\@iforloop#1,#2\@@#3#4{\def#3{#1}\ifx #3\@nnil
           \expandafter\@fornoop \else
          #4\relax\expandafter\@iforloop\fi#2\@@#3{#4}}
    \def\@tfor#1:={\@tf@r#1 }
    \long\def\@tf@r#1#2\do#3{\def\@fortmp{#2}\ifx\@fortmp\space\else
        \@tforloop#2\@nil\@nil\@@#1{#3}\fi}
    \long\def\@tforloop#1#2\@@#3#4{\def#3{#1}\ifx #3\@nnil
           \expandafter\@fornoop \else
          #4\relax\expandafter\@tforloop\fi#2\@@#3{#4}}
    \long\def\@break@tfor#1\@@#2#3{\fi\fi}
    \def\remove@to@nnil#1\@nnil{}
    \def\remove@angles#1>{\set@simple@size@args}
    \def\remove@star#1*{#1}
    \def\@defaultunits{\afterassignment\remove@to@nnil}

    \newif\ifmath@fonts \math@fontstrue
    \newbox\@labels
    \newif\if@inlabel \@inlabelfalse
    \newif\if@newlist   \@newlistfalse
    \newif\if@noparitem \@noparitemfalse
    \newif\if@noparlist \@noparlistfalse
    \newif\if@noitemarg \@noitemargfalse
    \newif\if@nmbrlist  \@nmbrlistfalse

    \def\glb@settings{}%
    "
  );

  //======================================================================
  // Perl L809: \loggingall
  //======================================================================
  // `\loggingoutput`, `\tracingfonts`, `\showoverfull`, `\showoutput`
  // moved to latex_constructs.rs (Perl L5676-5679); only `\loggingall`
  // belongs here per Perl L809.
  DefMacro!("\\loggingall", None);
  // `\wlog` moved to `latex_constructs_rust_only.rs`.

  //======================================================================
  // Perl L829-863: Expl3 / L3 hook stubs + kernel conditionals
  // (moved from latex_semi_undocumented.rs)
  //======================================================================

  // Perl-parity: `\@ifnextchar`, `\kernel@ifnextchar`, `\@ifnext` are
  // defined in latex_constructs.pool.ltxml L5687 (closure-backed, can't
  // round-trip through the dump). Relocated there 2026-04-18.

  // Perl-parity: `\makeatletter` / `\makeatother` are defined in
  // latex_constructs.pool.ltxml L5765-5766. Relocated there 2026-04-18.

  // L3 hook stubs — Perl latex_base L829-855
  // Perl L829: DefMacroI(T_CS('\hook_gput_code:nnn'), '{}{}{}', '');
  // Use the `DefMacroI`-style branch (`$cs:expr, $parameters:literal,
  // $expansion:literal`) via `T_CS!` so the CS name is built as a single
  // pre-tokenized Token — bypassing the string-prototype tokenizer which
  // would otherwise split on `_` (SUB) and `:` (OTHER) under default
  // catcodes.
  DefMacro!(T_CS!("\\hook_gput_code:nnn"), "{}{}{}", "");
  DefMacro!("\\NewHook{}", None);
  DefMacro!("\\NewReversedHook{}", None);
  DefMacro!("\\NewMirroredHookPair{}{}", None);
  DefMacro!("\\ActivateGenericHook{}", None);
  DefMacro!("\\DisableGenericHook{}", None);
  DefMacro!("\\AddToHook{}[]{}", None);
  DefMacro!("\\AddToHookNext{}{}", None);
  DefMacro!("\\ClearHookNext{}", None);
  DefMacro!("\\RemoveFromHook{}[]", None);
  DefMacro!("\\SetDefaultHookLabel{}", None);
  DefMacro!("\\PushDefaultHookLabel{}", None);
  DefMacro!("\\PopDefaultHookLabel", None);
  DefMacro!("\\UseHook{}", None);
  DefMacro!("\\UseOneTimeHook{}", None);
  DefMacro!("\\ShowHook{}", None);
  DefMacro!("\\LogHook{}", None);
  DefMacro!("\\DebugHooksOn", None);
  DefMacro!("\\DebugHooksOff", None);
  DefMacro!("\\DeclareHookRule{}{}{}{}", None);
  DefMacro!("\\DeclareDefaultHookRule{}{}{}", None);
  DefMacro!("\\ClearHookRule{}{}{}", None);
  DefMacro!("\\IfHookEmptyTF{}{}{}", "#3");
  DefMacro!("\\IfHookExistsTF{}{}{}", "#3");
  DefMacro!("\\MakeTextLowercase", "\\lowercase");
  DefMacro!("\\MakeTextUppercase", "\\uppercase");

  // Perl latex_base L856-862: kernel conditionals and Lets
  DefConditional!("\\if@includeinrelease");
  Let!("\\@kernel@after@enddocument", "\\@empty");
  Let!("\\@kernel@after@enddocument@afterlastpage", "\\@empty");
  Let!("\\@kernel@before@begindocument", "\\@empty");
  Let!("\\@kernel@after@begindocument", "\\@empty");
  Let!("\\conditionally@traceon", "\\@empty");
  Let!("\\conditionally@traceoff", "\\@empty");

  //======================================================================
  // Additional base definitions not in Perl's latex_base but needed early
  //======================================================================

  // Perl-parity: `\check@mathfonts`, `\fontsize`, `\@setfontsize` are
  // defined in latex_constructs.pool.ltxml L5670-5673 (serialize-able
  // token-list bodies). Relocated there 2026-04-18 — required to fix
  // the `\fontsize` undefined error under LATEXML_DUMP_ONLY=1 where
  // the dump reader's @-internal safety filter rejects public-CS
  // macros whose definitions normally come from `_base.rs`.

  // Class internals used by raw TeX classes
  // `\@bls` moved to `latex_constructs_rust_only.rs`.
  // `\@maxlistdepth` also in `latex_constructs_rust_only.rs`.

  // LaTeX kernel: \nofiles
  DefMacro!("\\nofiles", "\\@fileswfalse");

  // The following Perl-latex_constructs entries previously lived here
  // and were misplaced; all have been relocated to latex_constructs.rs
  // (Phase 25 cleanup, 2026-04-27):
  //   * Aux file stubs (Perl L5796-5800)
  //   * `\ignorespacesafterend`/`\mathgroup`/`\mathalpha` (Perl L5803-5805)
  //   * Hyphenation `\newlanguage\l@*` (Perl L5836-5886)
  //   * `\extrafloats` (Perl L5825) lives in latex_constructs_rust_only.rs
});
