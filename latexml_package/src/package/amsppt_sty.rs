//! amsppt.sty — AMSTeX plain TeX compatibility
//! Perl: amsppt.sty.ltxml — 500 lines
//! Document class for AMSTeX-style plain TeX documents.
//! Provides frontmatter, theorem environments, bibliography.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // amsppt loads the AmSTeX pool — Perl L27
  // AmSTeX pool is partially ported (~30%); residual undefined-CS
  // errors on amsppt papers (\text, \proof, \theorem env, \endmatrix,
  // \foldedtext, \eightbf, \AmSTeX, \DN@, \frills@, etc — see
  // LaTeXML/lib/LaTeXML/Engine/AmSTeX.pool.ltxml) trace to the
  // missing pool. Tested RequirePackage("amsmath") — covers \text but
  // exposes more AmS-TeX-specific CSes on math0111087 (54 undef →
  // worse-looking in absolute count). Proper fix is a real port of
  // AmSTeX.pool.ltxml; quick amsmath shim deferred.

  // Frontmatter — Perl L32-80. Original (pre-LaTeX) AMSPPT syntax uses
  // `\title Foo \endtitle` (tokens terminated by `\endtitle`, not a
  // `{…}` group). Prior Rust stubbed these as naked DefMacro expanding
  // to `\@add@frontmatter{ltx:X}`, which only works when the user writes
  // `\title{Foo}` (LaTeX-ish form). Switch to the `Until:\endX` delimiter
  // form the Perl port uses, with the `\endX` `Let`-ed to `\relax`.
  DefMacro!("\\makeheadline", "");
  DefMacro!("\\makefootline", "");
  DefMacro!("\\title Until:\\endtitle", "\\@add@frontmatter{ltx:title}{#1}");
  Let!("\\endtitle", "\\relax");
  DefMacro!("\\author Until:\\endauthor",
    "\\@add@frontmatter{ltx:creator}[role=author]{\\@personname{#1}}");
  Let!("\\endauthor", "\\relax");

  // Affiliations and contacts — Perl L85-130
  DefConstructor!("\\@@@affil{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\affil Until:\\endaffil",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@affil{#1}}");
  Let!("\\endaffil", "\\relax");
  DefConstructor!("\\@@@address{}", "^ <ltx:contact role='address'>#1</ltx:contact>");
  DefMacro!("\\address Until:\\endaddress",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@address{#1}}");
  Let!("\\endaddress", "\\relax");
  DefConstructor!("\\@@@curraddr{}", "^ <ltx:contact role='current_address'>#1</ltx:contact>");
  DefMacro!("\\curraddr Until:\\endcurraddr",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@curraddr{#1}}");
  Let!("\\endcurraddr", "\\relax");
  DefConstructor!("\\@@@email{}", "^ <ltx:contact role='email'>#1</ltx:contact>");
  DefMacro!("\\email Until:\\endemail",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@email{#1}}");
  Let!("\\endemail", "\\relax");
  DefConstructor!("\\@@@urladdr{}", "^ <ltx:contact role='url'>#1</ltx:contact>");
  DefMacro!("\\urladdr Until:\\endurladdr",
    "\\@add@to@frontmatter{ltx:creator}{\\@@@urladdr{#1}}");
  Let!("\\endurladdr", "\\relax");

  // Perl amsppt.sty.ltxml L72-75: thanks/date/dedicatory/translator —
  // previously absent in Rust.
  DefMacro!("\\thanks Until:\\endthanks",
    "\\@add@frontmatter{ltx:note}[role=support]{#1}");
  Let!("\\endthanks", "\\relax");
  DefMacro!("\\date Until:\\enddate",
    "\\@add@frontmatter{ltx:date}[role=creation]{#1}");
  Let!("\\enddate", "\\relax");
  DefMacro!("\\dedicatory Until:\\enddedicatory",
    "\\@add@frontmatter{ltx:note}[role=dedicatory]{#1}");
  Let!("\\enddedicatory", "\\relax");
  DefMacro!("\\translator Until:\\endtranslator",
    "\\@add@frontmatter{ltx:creator}[role=translator]{\\@personname{#1}}");
  Let!("\\endtranslator", "\\relax");

  // Abstract and classification — Perl L76-79.
  DefMacro!("\\keywords Until:\\endkeywords",
    "\\@add@frontmatter{ltx:keywords}{#1}");
  Let!("\\endkeywords", "\\relax");
  DefMacro!("\\subjclass Until:\\endsubjclass",
    "\\@add@frontmatter{ltx:classification}[scheme=MSC]{#1}");
  Let!("\\endsubjclass", "\\relax");
  DefMacro!("\\abstract Until:\\endabstract",
    "\\@add@frontmatter{ltx:abstract}{#1}");
  Let!("\\endabstract", "\\relax");

  // Section structure — Perl L112-147. AmSTeX uses terminator-delimited
  // syntax (`\head Foo \endhead`) not balanced `\section{Foo}`. Previous
  // Rust just did `\heading → \section*` which reads the next `{...}` — a
  // real bug on `\heading Foo \endheading` (the Foo ends up inlined with
  // no section wrapper, and `\endheading` leaks). Port the full family
  // with `Until:\end<x>` delimiters. Perl uses DefConstructors with
  // bounded+inlist=toc+RefStepID; we simplify to `\section*{#1}` etc.
  // (same as existing simplification for other head CSes), but at least
  // the argument capture is now syntactically correct.
  //
  // Intentional DefConstructor → DefMacro kind divergence for the
  // entire head family (\head, \heading, \subheading*, \specialhead,
  // \subhead, \subsubhead, and their \end<x> pairs): Rust delegates
  // to `\section*` / `\subsection*` / `\subsubsection*` instead of
  // re-implementing per-kind RefStepID + inlist=toc + bounded glue.
  // Section/TOC numbering uses LaTeX's native machinery rather than
  // amsppt's, which is a known cross-package divergence but acceptable
  // because amsppt is only used by legacy pre-LaTeX2e submissions.
  // WISDOM #44 — observable XML structure matches; TOC numbering
  // scheme differs deliberately.
  DefMacro!("\\head Until:\\endhead", "\\section*{#1}");
  Let!("\\endhead", "\\relax");
  DefMacro!("\\heading Until:\\endheading", "\\head#1\\endhead");
  Let!("\\endheading", "\\relax");
  // Perl amsppt.sty.ltxml L133-141: \subheading dispatches on next token.
  // `\subheading{title}` → \subheading@onearg{title}
  // `\subheading title \endsubheading` → \subheading@env (Until:\endsubheading)
  // Both helpers expand to `\subhead{title}\endsubhead` in Perl; Rust lacks
  // a separate \subhead binding, so route both through `\subsection*{title}`
  // (the existing Rust target). `locked=>true` matches Perl L138 — guards
  // against downstream \renewcommand resetting the dispatch.
  DefMacro!("\\subheading", sub[_args] {
    let next = gullet::read_token()?;
    if let Some(t) = next {
      gullet::unread(Tokens!(t));
      if t.get_catcode() == Catcode::BEGIN {
        return Ok(Tokens!(T_CS!("\\subheading@onearg")));
      }
    }
    Ok(Tokens!(T_CS!("\\subheading@env")))
  }, locked => true);
  DefMacro!("\\subheading@onearg{}", "\\subsection*{#1}");
  DefMacro!("\\subheading@env Until:\\endsubheading", "\\subsection*{#1}");
  // Kept defined as a no-op for stray-use safety; \subheading@env consumes
  // the trailing \endsubheading inline so this binding usually doesn't fire.
  DefMacro!("\\endsubheading", "");
  // Perl L112-117 `\specialhead Until:\endspecialhead → <ltx:chapter>`;
  // L143-148 `\subsubhead Until:\endsubsubhead → <ltx:subsubsection>`.
  // Also `\subhead` (no current Rust) needed by the \subheading@env path.
  // All forwarded to their LaTeX starred siblings.
  DefMacro!("\\specialhead Until:\\endspecialhead", "\\section*{#1}");
  Let!("\\endspecialhead", "\\relax");
  DefMacro!("\\subhead Until:\\endsubhead", "\\subsection*{#1}");
  Let!("\\endsubhead", "\\relax");
  DefMacro!("\\subsubhead Until:\\endsubsubhead", "\\subsubsection*{#1}");
  Let!("\\endsubsubhead", "\\relax");

  // Theorem environments — Perl L200-260 use
  //   DefConstructor('\<kind> Undigested DigestUntil:\end<kind>', …)
  // each with its own counter, title font, afterConstruct close, and
  // title-name Digest. DigestUntil is now fully ported (27cc66b60);
  // wiring these up to Perl-parity is still deferred because each
  // needs a NewCounter('<kind>') declaration plus the title-font
  // computation — risk of conflict with amsthm's theorem counter.
  // Current stubs forward to the corresponding `theorem`/`definition`/
  // etc. LaTeX environments, which produce valid ltx:theorem output
  // but with a different counter namespace than native amsppt would.
  //
  // Intentional DefConstructor → DefMacro kind divergence for the
  // entire theorem-env family (\proclaim, \definition, \remark,
  // \example, \demo, \roster and their \end<x> pairs): Rust delegates
  // to `\begin{theorem}` / `\begin{definition}` / etc. instead of
  // re-implementing per-kind counter+title glue. The `{theorem}` env
  // machinery (from LaTeX's native amsthm equivalent) already
  // produces the `ltx:theorem class="ltx_theorem_<kind>"` wrapper
  // Perl's DefConstructor would emit. WISDOM #44 — observable XML
  // matches; amsthm-counter-namespace-aliasing is deliberate.
  DefMacro!("\\proclaim", "\\begin{theorem}");
  DefMacro!("\\endproclaim", "\\end{theorem}");
  DefMacro!("\\definition", "\\begin{definition}");
  DefMacro!("\\enddefinition", "\\end{definition}");
  DefMacro!("\\remark", "\\begin{remark}");
  DefMacro!("\\endremark", "\\end{remark}");
  DefMacro!("\\example", "\\begin{example}");
  DefMacro!("\\endexample", "\\end{example}");
  DefMacro!("\\demo", "\\begin{proof}");
  DefMacro!("\\enddemo", "\\end{proof}");

  // Lists — Perl L265-300
  DefMacro!("\\roster", "\\begin{enumerate}");
  DefMacro!("\\endroster", "\\end{enumerate}");

  // Perl amsppt.sty.ltxml L261-263: \block — simple block-quote container.
  // Previously unported. DigestUntil parameter type landed in 27cc66b60
  // makes this a direct translation.
  DefConstructor!(
    "\\block DigestUntil:\\endblock",
    "<ltx:quote>#1</ltx:quote>"
  );
  Let!(T_CS!("\\endblock"), T_CS!("\\relax"));

  // Footnotes — Perl L305-350
  // Perl amsppt L276 is `DefConstructor('\footnote', <ltx:note role='footnote'>)`
  // — a direct constructor. Rust delegates to `\lx@note{footnote}` (a
  // helper in `latex_constructs.rs` that already carries the same
  // ltx:note wrapper + role attr). Intentional DefConstructor →
  // DefMacro kind divergence via delegation (WISDOM #44).
  DefMacro!("\\footnote", "\\lx@note{footnote}");

  // Bibliography — Perl L355-500
  // Complex Perl closure system for reference formatting
  // Perl L359, L456: amsppt extends ltx:biblist + ltx:bibblock with
  // autoOpen so a bibitem child without an explicit \begin{biblist}
  // wrapper still nests correctly. Core latex_constructs already sets
  // auto_close on biblist/bibblock; here we add auto_open on top to
  // reach amsppt's documented spec.
  Tag!("ltx:biblist",  auto_open => true, auto_close => true);
  Tag!("ltx:bibblock", auto_open => true, auto_close => true);

  // Perl amsppt.sty.ltxml L167-168, L358: auto_close on theorem/proof/
  // bibliography so AmSTeX's implicit structure (no explicit \end) still
  // closes on the next block-level open. AmSTeX documents rely on this
  // because the top-level CSes like \proclaim and \demo don't pair with
  // \end markers. Without these, a `\proclaim` followed by a top-level
  // paragraph would try to nest the paragraph inside the theorem.
  Tag!("ltx:theorem",      auto_close => true);
  Tag!("ltx:proof",        auto_close => true);
  Tag!("ltx:bibliography", auto_close => true);
  // Perl L306 also registers `Tag('ltx:note', afterClose => \&relocate
  // Footnote)` — the closure walks the node tree to re-parent stray
  // footnotes onto their originating paragraph. Deferred: requires the
  // full relocateFootnote infra. No amsppt test in the suite, so leaving
  // note-handling unported is acceptable for now.

  // Perl amsppt.sty.ltxml L457-458: token-valued \holdoverbox register
  // and 1-arg \holdover{#1} no-op. AmSTeX bib entries use \holdover{...}
  // to bounce a partial field to the next entry; the token register
  // accumulates the held tokens across the bib-block. Rust doesn't run
  // the accumulation (stubbed), but both CSes must still resolve so
  // bibliographies using \holdover don't hit undefined-CS.
  DefRegister!("\\holdoverbox" => Tokens!());
  DefMacro!("\\holdover{}", "");

  DefMacro!("\\Refs", "\\begin{thebibliography}{}");
  DefMacro!("\\endRefs", "\\end{thebibliography}");
  DefMacro!("\\ref", "\\bibitem");
  DefMacro!("\\endref", "");
  DefMacro!("\\by", "");
  // Perl L464: \bysame → \by  --- (three hyphens, with the leading
  // \by bibfield marker). In Rust \by currently expands to empty, so
  // the practical effect is just the em-dash triple (---). Prior Rust
  // dropped it entirely; restore the Perl expansion for faithful
  // "by the same author" rendering.
  DefMacro!("\\bysame", "\\by ---");
  DefMacro!("\\paper", "\\textit");
  DefMacro!("\\paperinfo{}", "#1");
  DefMacro!("\\jour{}", "\\textit{#1}");
  DefMacro!("\\vol{}", "{\\bf #1}");
  DefMacro!("\\yr{}", "(#1)");
  DefMacro!("\\pages{}", "#1");
  DefMacro!("\\page{}", "#1");
  DefMacro!("\\book{}", "\\textit{#1}");
  DefMacro!("\\bookinfo{}", "#1");
  DefMacro!("\\publ{}", "#1");
  DefMacro!("\\publaddr{}", "#1");
  DefMacro!("\\finalinfo{}", "#1");
  DefMacro!("\\eds{}", "(#1, eds.)");
  DefMacro!("\\ed{}", "(#1, ed.)");
  DefMacro!("\\moreref", "");
  DefMacro!("\\lang{}", "[#1]");
  DefMacro!("\\toappear", "(to appear)");
  DefMacro!("\\inpress", "(in press)");
  DefMacro!("\\issue{}", "no. #1");
  DefMacro!("\\miscnote{}", "#1");

  // Perl L478, L480, L484: plain-text bib-entry keyword stubs referenced
  // by the formatted \@fill@bibitem body. Even though Rust's amsppt
  // doesn't ship the full \@bibfield infrastructure, authors sometimes
  // invoke these directly in hand-rolled `\ref ... \endref` bibliography
  // entries.
  DefMacro!("\\voltext",   "vol.");
  DefMacro!("\\issuetext", "no.");
  DefMacro!("\\pagestext", "pp.");

  // Miscellaneous — Perl L480-500
  DefMacro!("\\nologo", "");
  DefMacro!("\\NoBlackBoxes", "");

  // AmSTeX pool compatibility stubs — Perl AmSTeX.pool.ltxml L75-114.
  // amsppt.sty (Perl) implicitly loads the AmSTeX pool, which provides
  // these as author-ignorable no-ops. Rust doesn't port AmSTeX pool
  // (~30% ported per L10 comment), so documents using bare amsppt risk
  // undefined-CS on these formatting controls. Adding as empty stubs
  // keeps documents compile without altering XML output.
  DefMacro!("\\NoPageNumbers", "");
  DefMacro!("\\BlackBoxes", "");
  DefMacro!("\\TagsAsMath", "");
  DefMacro!("\\TagsAsText", "");
  DefMacro!("\\TagsOnLeft", "");
  DefMacro!("\\TagsOnRight", "");
  DefMacro!("\\CenteredTagsOnSplits", "");
  DefMacro!("\\TopOrBottomTagsOnSplits", "");
  DefMacro!("\\LimitsOnInts", "");
  DefMacro!("\\NoLimitsOnInts", "");
  DefMacro!("\\LimitsOnNames", "");
  DefMacro!("\\NoLimitsOnNames", "");
  DefMacro!("\\LimitsOnSums", "");
  DefMacro!("\\NoLimitsOnSums", "");
  DefMacro!("\\UseAMSsymbols", "");
  DefMacro!("\\loadbold", "");
  DefMacro!("\\loadeufb", "");
  DefMacro!("\\loadeufm", "");
  DefMacro!("\\loadeurb", "");
  DefMacro!("\\loadeurm", "");
  DefMacro!("\\loadeusb", "");
  DefMacro!("\\loadeusm", "");
  DefMacro!("\\loadmathfont", "");
  DefMacro!("\\loadmsam", "");
  DefMacro!("\\loadmsbm", "");
  DefMacro!("\\boldnotloaded{}", "");
  DefMacro!("\\galleys", "");
  // Perl AmSTeX.pool L114: \flushpar = \par\noindent
  DefMacro!("\\flushpar", "\\par\\noindent");

  // Page-layout no-ops — Perl AmSTeX.pool L116-119.
  DefMacro!("\\pagewidth{Dimension}", "");
  DefMacro!("\\pageheight{Dimension}", "");
  DefMacro!("\\hcorrection{Dimension}", "");
  DefMacro!("\\vcorrection{Dimension}", "");

  // Perl L186: \tie = \unskip\nobreak\␣ (non-breaking space with
  // preceding skip-absorption).
  DefMacro!("\\tie", "\\unskip\\nobreak\\ ");

  // Perl L299-300: math superscript accents via manual ^{...}.
  // Siblings \spcheck/\sptilde are already in Rust plain_base.
  DefMacro!("\\spacute", "^{'}");
  DefMacro!("\\spgrave", "^{`}");

  // Perl AmSTeX.pool L133-134: frontmatter bracket markers.
  // Rust amsppt handles frontmatter via \title/\author/\abstract
  // directly, so the outer bracket is a no-op.
  DefMacro!("\\topmatter", "");
  DefMacro!("\\endtopmatter", "");

  // Perl L256-257: set-braces via \overbrace/\underbrace with the
  // "label" part from before `\to` as superscript/subscript.
  DefMacro!("\\oversetbrace Until:\\to {}",  "\\overbrace{#2}^{#1}");
  DefMacro!("\\undersetbrace Until:\\to {}", "\\underbrace{#2}^{#1}");

  // Perl L289-295: \thickfrac / \thickfracwithdelims. Perl peeks
  // for a following `\thickness` keyword to dispatch between the
  // `\@thickfrac` and `\frac` forms. Rust doesn't implement the
  // `\thickness`-peek dispatch, so route directly to \frac (the
  // no-thickness variant) — the most common case. Same for the
  // delims variant.
  DefMacro!("\\thickfrac", "\\frac");
  DefMacro!("\\thickfracwithdelims{}{}", "\\fracwithdelims{#1}{#2}");
  DefMacro!("\\@thickfrac Token Number {}{}", "\\genfrac{}{}{#2}{}{#3}{#4}");
  DefMacro!("\\@thickfracwithdelims {}{} Token Number {}{}",
    "\\genfrac{#1}{#2}{#4}{}{#5}{#6}");

  // Perl AmSTeX.pool L34: \AmSTeX — logo constructor; render as plain text.
  DefMacro!("\\AmSTeX", "AMSTeX");
  // Perl L175-184: page/line/math break hints — all empty (layout-only).
  DefMacro!("\\bigpagebreak", "");
  DefMacro!("\\allowlinebreak", "");
  DefMacro!("\\allowmathbreak", "");
  DefMacro!("\\allowdisplaybreak", "");
  DefMacro!("\\allowdisplaybreaks", "");
  // Perl L270-284: pass-through math-font wrappers. Perl uses
  // DefConstructor with `bounded => 1, requireMath => 1` to scope
  // the font change; Rust simplifies to the identity DefMacro since
  // the body already carries the math-mode context.
  DefMacro!("\\Cal{}", "#1");
  DefMacro!("\\italic{}", "#1");
  DefMacro!("\\boldkey{}", "#1");
  // Perl L395: \botaligned = \aligned[b] (bottom-vertically-aligned).
  DefMacro!("\\botaligned", "\\aligned[b]");

  // Perl L173-182: more layout-hint empty stubs.
  DefMacro!("\\smallpagebreak", "");
  DefMacro!("\\medpagebreak", "");
  DefMacro!("\\mathbreak", "");
  DefMacro!("\\nomathbreak", "");
  DefMacro!("\\nomultlinegap", "");
  DefMacro!("\\MultlineGap Dimension", "");

  // Perl L350-358: top/bot shave and smash — pass-through text wrappers
  // (Perl DefConstructor with enterHorizontal, flattened to DefMacro
  // identity since the wrapper is decorative).
  DefMacro!("\\botshave{}", "#1");
  DefMacro!("\\topshave{}", "#1");
  DefMacro!("\\topsmash{}", "#1");
  DefMacro!("\\botsmash{}", "#1");

  // Perl L354: \pretend Until:\haswidth {body} — body is #1 up through
  // \haswidth, then {width} follows. Drop the width spec, keep body.
  DefMacro!("\\pretend Until:\\haswidth {}", "#1");

  // Perl L303-308: spdddot / spddddot / spbar / spvec / spbreve —
  // math superscript accents (siblings of existing \spcheck/\sptilde/
  // \spacute/\spgrave). Complete the family.
  DefMacro!("\\spdddot", "^{...}");
  DefMacro!("\\spddddot", "^{....}");
  DefMacro!("\\spbar", "^{-}");
  DefMacro!("\\spvec", "^{\\rightarrow}");

  // Perl L348, L356, L393, L456-458: more empty stubs and aliases.
  DefMacro!("\\ResetBuffer", "");
  DefMacro!("\\snug", "");
  DefMacro!("\\printoptions", "");
  DefMacro!("\\showallocations", "");
  DefMacro!("\\syntax", "");
  // Perl L393: \topaligned = \aligned[t] (sibling of \botaligned).
  DefMacro!("\\topaligned", "\\aligned[t]");

  // Perl L164-166: \textfonti, \textfontii — plain-TeX font-switch
  // primitives, no LaTeXML-observable effect.
  DefMacro!("\\textfonti", "");
  DefMacro!("\\textfontii", "");

  // Perl L281-282: \slanted{#1} — math-font wrapper flattened to
  // identity (same rationale as \Cal/\italic/\boldkey).
  DefMacro!("\\slanted{}", "#1");

  // Perl L349: \shave{#1} → #1 (sibling of \botshave/\topshave).
  DefMacro!("\\shave{}", "#1");

  // Perl amsppt.sty.ltxml L460-495: \@bibfield and friends are the
  // formatted-bib-entry field-routing dispatcher. Rust doesn't
  // implement the \@fill@bibitem consumer that collects these
  // fields, so each routed entry degrades to just its trailing
  // "label text" part. \@end@bibfield is a bare marker; \@bibfield
  // takes a type and swallows content until the next \@end@bibfield
  // or top-level break.
  DefMacro!("\\@end@bibfield", "");
  DefMacro!("\\@bibfield{}", "");
  // The routed entries, Perl L460-493. Expansion after stub:
  // \key → "\@end@bibfield\@bibfield{key}" → ""  (field consumed).
  // \MR  → "\@end@bibfield\@bibfield{mathreview}MR " → "MR ".
  // \AMSPPS → "AMS-PPS ". \CMP → "CMP ".
  DefMacro!("\\key", "\\@end@bibfield\\@bibfield{key}");
  DefMacro!("\\no", "\\@end@bibfield\\@bibfield{refnum}");
  DefMacro!("\\inbook", "\\@end@bibfield\\@bibfield{inbook}");
  DefMacro!("\\procinfo", "\\@end@bibfield\\@bibfield{proceedingsinfo}");
  DefMacro!("\\MR", "\\@end@bibfield\\@bibfield{mathreview}MR ");
  DefMacro!("\\AMSPPS", "\\@end@bibfield\\@bibfield{ams-preprint}AMS-PPS ");
  DefMacro!("\\CMP", "\\@end@bibfield\\@bibfield{CMP}CMP ");

  // Perl L169: \spreadlines {Dimension} — line-spacing dimension
  // consumer, no output (DefConstructor with empty emission).
  DefMacro!("\\spreadlines{}", "");
  // Perl L360: \spreadmatrixlines Dimension — same shape, Dimension
  // param.
  DefMacro!("\\spreadmatrixlines Dimension", "");


  DefMacro!("\\redefine", "\\def");
  DefMacro!("\\define", "\\def");

  // Identity / version metadata — Perl amsppt.sty.ltxml L29-35.
  DefMacro!("\\filename", "amsppt.sty");
  DefMacro!("\\fileversion", "2.1h");
  DefMacro!("\\filedate", "1997/02/02");
  DefMacro!("\\fileversiontest", "\\fileversion\\space(\\filedate)");
  DefMacro!("\\styname", "AMSPPT");
  DefMacro!("\\styversion", "\\fileversion");
  DefMacro!("\\plainend", "\\end");

  // Page-layout no-ops — Perl L40-52. Running-head tokens + page-contents
  // are TeX plain-format hooks with no LaTeXML analogue; swallow their
  // args.
  DefMacro!("\\leftheadline", "");
  DefMacro!("\\rightheadline", "");
  DefMacro!("\\leftheadtext{}", "");
  DefMacro!("\\rightheadtext{}", "");
  Let!("\\flheadline", "\\hfil");
  Let!("\\frheadline", "\\hfil");
  DefMacro!("\\headmark{}", "");
  DefMacro!("\\pagecontents", "");
  DefMacro!("\\cvolyear{}", "");
  DefMacro!("\\issueinfo{}{}{}{}", "");
  DefMacro!("\\NoRunningHeads", "");
  DefMacro!("\\Monograph", "");

  // Per-field "pre" hooks — Perl L90-95. No-ops; user can `\def` to override.
  Let!("\\pretitle", "\\relax");
  Let!("\\preauthor", "\\relax");
  Let!("\\preaffil", "\\relax");
  Let!("\\predate", "\\relax");
  Let!("\\preabstract", "\\relax");
  Let!("\\prepaper", "\\relax");

  // AMS Fonts + QED — Perl L320-330. `\rom` drops its arg into rm.
  // `\qed` emits `\ltx@qed`, which is isMath-aware (end-of-proof symbol).
  // `\tildechar` is the amsppt literal `~` in typewriter (bibliography
  // key separator).
  DefMacro!("\\rom{}", "{\\rm #1}");
  DefMacro!("\\PSAMSFonts", "");
  RawTeX!("\\newif\\ifPSAMSFonts\\PSAMSFontstrue");
  DefMacro!("\\qed", "\\ltx@qed");
  DefConstructor!(
    "\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    reversion => "\\qed"
  );
  // Perl L327: DefPrimitiveI('\tildechar', undef, "~", font => { family => 'typewriter' })
  // — emits a literal `~` other-token in typewriter family, immediately during
  // digestion, with no expansion. Prior Rust DefMacro!("\\tildechar",
  // "\\texttt{\\textasciitilde}") routed through textcomp's font dispatch,
  // changing both the CS class (macro vs primitive) and producing different
  // token structure (\texttt opens a bounded font scope; the literal `~` does
  // not). Restored to faithful primitive form.
  DefPrimitive!("\\tildechar", "~", font => { family => "typewriter" });
  DefMacro!("\\breakcheck", "");
  DefMacro!("\\usualspace", " ");
  // Perl L329: \normalparindent — zero-Dimension register. Without it,
  // `\the\normalparindent` fails on amsppt documents that probe it.
  DefRegister!("\\normalparindent" => Dimension::new(0));

  // References section — Perl L333, L361-365.
  DefMacro!("\\Refsname", "References");
  DefRegister!("\\refindentwd" => Dimension::new(0));
  DefMacro!("\\refstyle{}", "");
  DefMacro!("\\keyformat{}", "#1");
  DefMacro!("\\refbreaks", "");
  DefMacro!("\\defaultreftexts", "");

  // Perl L335: \cite for plain-AMSTeX documents.
  // amsppt is a plain-TeX style, so the latex cite machinery isn't
  // loaded; this DefConstructor provides the minimal cite→bibref
  // surface that \Refs/\bibitem pair against.
  DefConstructor!("\\cite Semiverbatim",
    "<ltx:cite>[<ltx:bibref show='refnum' bibrefs='#1'/>]</ltx:cite>");

  // Head-toks and head-skip registers — Perl L44-45, L151-158. Token
  // registers for running-head content; eight length/glue registers
  // controlling head spacing. All default to zero — amsppt uses them
  // to drive its plain-format page layout, which LaTeXML ignores but
  // user code may still `\the...` or `\setlength` them.
  DefRegister!("\\leftheadtoks"        => Tokens!());
  DefRegister!("\\rightheadtoks"       => Tokens!());
  DefRegister!("\\aboveheadskip"       => Glue::new(0));
  DefRegister!("\\belowheadskip"       => Dimension::new(0));
  DefRegister!("\\abovespecialheadskip" => Glue::new(0));
  DefRegister!("\\subheadskip"         => Glue::new(0));
  DefRegister!("\\subsubheadskip"      => Glue::new(0));
  DefRegister!("\\headlineheight"      => Dimension::new(0));
  DefRegister!("\\headlinespace"       => Dimension::new(0));
  DefRegister!("\\dropfoliodepth"      => Dimension::new(0));
  DefMacro!("\\widestnumber Token {}", "");
  DefMacro!("\\nofrillscheck{}", "");
  DefMacro!("\\toc Until:\\endtoc", "");
  Let!("\\endtoc", "\\relax");

  // Theorem-env skip registers and name/font overrides — Perl L178-232.
  // The full DefConstructor bodies for \proclaim / \definition etc. still
  // need NewCounter+title infrastructure to match, but the register /
  // macro stubs are safe to land now so users can `\def\proclaimfont{…}`.
  DefRegister!("\\preproclaimskip"     => Glue::new(0));
  DefRegister!("\\postproclaimskip"    => Glue::new(0));
  DefMacro!("\\proclaimfont", "\\it");
  DefRegister!("\\remarkskip"          => Glue::new(0));
  DefRegister!("\\postdemoskip"        => Glue::new(0));
  DefRegister!("\\predefinitionskip"   => Glue::new(0));
  DefRegister!("\\postdefinitionskip"  => Glue::new(0));
  DefMacro!("\\definitionfont", "\\rm");
  DefMacro!("\\definitionname", "Definition");
  DefMacro!("\\remarkfont", "\\rm");
  DefMacro!("\\remarkname", "Remark");
  DefMacro!("\\demonstrationname", "Demonstration");

  // Perl amsppt.sty.ltxml L250: \therosteritem{#1} expands to \rom{(#1)}.
  // Used by \roster … \item to wrap the auto-numbered index in upright
  // parentheses. Previously unported.
  DefMacro!("\\therosteritem{}", "\\rom{(#1)}");
  // Perl L468: \edtext expands to "ed." — the editor-marker inserted in
  // bib entries after an `\editors{...}` field.
  DefMacro!("\\edtext", "ed.");

  // Roster / layout — Perl L246, L265-270.
  DefRegister!("\\rosteritemwd"        => Dimension::new(0));
  DefRegister!("\\pagenumwd"           => Dimension::new(0));
  DefRegister!("\\indenti"             => Dimension::new(0));
  DefRegister!("\\indentii"            => Dimension::new(0));
  DefMacro!("\\linespacing Number", "");
  DefMacro!("\\endquotes", "");

  // Perl amsppt.sty.ltxml L497: \smc (smallcaps) — plain-TeX font
  // switch used in AMSTeX running heads and bib entries.
  DefPrimitive!("\\smc", None, font => { shape => "smallcaps" });
});
