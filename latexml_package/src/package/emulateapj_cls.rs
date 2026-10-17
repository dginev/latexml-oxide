use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: emulateapj.cls.ltxml — Seems to be equivalent to aastex.
  // Perl `LoadClass('aastex', withoptions => 1)` forwards the current class
  // options; Rust's `load_class_with_options` (latexml_core::binding::content)
  // reads `class_options` from state and passes them through. Previous
  // `load_class("aastex", Vec::new(), ...)` silently dropped the user's
  // `\documentclass[...]{emulateapj}` options before they reached aastex.
  load_class_with_options("aastex", Tokens!())?;
  RequireResource!("ltx-apj.css");
  RequirePackage!("emulateapj");
  // emulateapj.cls is a revtex4 class of its own (:165-167 `\LoadClass…{\@revtex@cls}`), not an AASTeX version: it has
  // no AASTeX 6 `\fig` family or `\gridline` (which the aastex binding defines for a 6+ class only, aastex_cls.rs), so a
  // paper's own `\fig` is its own (astro-ph/0503342 `\newcommand{\fig}`; KNOWN_PERL_ERRORS #557). The 5.x
  // `\subsubsubsection`/`\supportfrom` stay, the family's union (user 2026-10-10).
  for command in ["\\fig", "\\leftfig", "\\rightfig", "\\boxedfig", "\\rotatefig", "\\gridline"] {
    Let!(T_CS!(command), "\\@undefined");
  }
  // …nor AASTeX 5.x's `{plate}` (emulateapj.cls defines none; aastex_cls.rs `undefine_plate`).
  aastex_cls::undefine_plate();
  // …and unlocked: emulateapj_sty.rs locks the `\fig` it found defined, which a 6+ `aastex.cls` the paper ships leaves
  // (aastex_cls.rs), and a lock on an undefined `\fig` would refuse the paper's own (64f review r3).
  assign_value("\\fig:locked", Stored::None, Some(Scope::Global));
  // emulateapj's deluxetable is a longtable opened at the environment's `\begin` (emulateapj.cls:1430-1460, `{table*}`
  // and `{longtable*}` :1464-1495), not AASTeX's table whose data `\startdata#1\enddata` holds (aastex701.cls:11384,
  // deluxetable_sty.rs): `\tablehead` (:1329-1345) sets its heading rows in the table, the rows after it are rows with
  // or without a `\startdata`, which only sets the caption of a table without a head (:1324), and `\enddata` only ends
  // the last row (:1497 `\def\enddata{\\[\lastrowheight@]…}`), so rows after it are rows too and a second one is
  // harmless. Here the data opens at `\tablehead` or `\startdata`, whichever comes first, and closes at the
  // environment's end (with the rule `\tablehead`'s `\endfoot` sets, :1344-1345), so the `\tablecaption`/`\tablewidth`
  // set before them (stored, :1294, :1359) stay out of it as before. Perl's AASTeX table (deluxetable.sty.ltxml:35-43)
  // left a head's `\\ \cline`, the rows of a table without `\startdata`, and the rows after `\enddata` out of the
  // alignment (1411.3719, 1909.11243, 1002.1848, 1902.00588; 1506.07185, 1705.08932: a table ended without `\enddata`;
  // KNOWN_PERL_ERRORS #665, #874). Guard: `perfect_kernel_batch68y::emulateapj_deluxetable_is_open_from_its_head`.
  // emulateapj's mark is text, `\tablenotemark{#1}` (:1424-1427) and `\footnotetext[#1]` outside a table (:1319), not
  // AASTeX's math (aas_support_sty.rs); in the table the notes are kept for its end (:1424-1427 `\DT@p@ftn`), as the
  // `\tablecomments`/`\tablerefs` lists are (:1436-1437, deluxetable_sty.rs), so a row holding only them (after the
  // last `\enddata`) is an empty row, dropped. A `\label` there is the table's (`\@currentlabel`, the caption's), kept
  // for the end too, so it makes no row either.
  DefMacro!("\\tablenotetext{}{}", "\\lx@deluxe@tablenotetext{#1}{#2}");
  DefMacro!("\\lx@eapj@tablenotetext{}{}", r"\g@addto@macro\tblnote@list{\lx@deluxe@tablenotetext{#1}{#2}}");
  DefMacro!("\\lx@eapj@label{}",
    r"\iflx@deluxetable@data\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi
{\g@addto@macro\lx@eapj@labels{\lx@eapj@saved@label{#1}}}{\lx@eapj@saved@label{#1}}");
  DefMacro!("\\lx@eapj@begin@deluxetable",
    r"\let\tablenotetext\lx@eapj@tablenotetext\let\lx@eapj@saved@label\label\let\label\lx@eapj@label
\gdef\lx@eapj@labels{}\let\tablehead\lx@eapj@tablehead\let\startdata\lx@eapj@open@data\let\enddata\lx@eapj@enddata");
  DefMacro!("\\lx@eapj@open@data",
    r"\iflx@deluxetable@data\expandafter\@gobble\else\expandafter\@firstofone\fi
{\global\lx@deluxetable@datatrue\bgroup\@deluxetable@bindings\@@deluxetabular\lx@begin@alignment\hline\hline}");
  DefMacro!("\\lx@eapj@tablehead{}",
    r"\lx@eapj@open@data\lx@alignment@begin@heading#1\\\hline\lx@alignment@end@heading");
  DefMacro!("\\lx@eapj@enddata", r"\iflx@deluxetable@data\crcr\fi");
  DefMacro!("\\lx@eapj@end@deluxetable",
    r"\iflx@deluxetable@data\expandafter\@firstofone\else\expandafter\@gobble\fi
{\crcr\hline\global\lx@deluxetable@datafalse\lx@end@alignment\@end@deluxetabular\egroup}\lx@eapj@labels
\spew@tblnotes");
  DefMacro!("\\deluxetable{}",
    r"\set@deluxetable@template{#1}\def\@deluxetable@header{}\lx@eapj@begin@deluxetable\begin{table}");
  DefMacro!(T_CS!("\\deluxetable*"), "{}",
    r"\set@deluxetable@template{#1}\def\@deluxetable@header{}\lx@eapj@begin@deluxetable\begin{table*}");
  DefMacro!("\\enddeluxetable", r"\lx@eapj@end@deluxetable\end{table}");
  DefMacro!(T_CS!("\\enddeluxetable*"), None, r"\lx@eapj@end@deluxetable\end{table*}");
  // emulateapj loads REVTeX 4.1 when it is installed (emulateapj.cls:76-78, :151-167; copies since 11/10/2009, 1601.00007's
  // too), whose `\email`/`\homepage` set the address with `\href`, in the current font (revtex4-1.cls:1959-1961,
  // 2383-2391): roman, as pdflatex prints them (CMR8, also under `[revtex4]`), not AASTeX 5.x's `\tt` or REVTeX 4.0's
  // `\url` that the aastex binding it loads installs. The `_` is kept (user ruling 2026-10-11).
  DefMacro!("\\email[]{}", "\\lx@add@email{\\lx@address@textfont{#2}}");
  DefMacro!("\\homepage[] Semiverbatim", "\\lx@add@url[name={#1}]{\\lx@address@textfont{#2}}");
});
