//! The markup the VTeX IMS journal classes share: arximspdf.cls / arxstspdf.cls (Annals of Probability/Statistics)
//! and arxbj.cls (Bernoulli) define the same structured bibliography (`{barticle}`/… with `\b*` fields; arxbj.cls:
//! 2599-2897), the same `{pf}` proof and automatic `\qed` (arximspdf.cls:1417-1434 = arxbj.cls:1428-1445) and the same
//! `\tablewidth` table notes (arximspdf.cls:1729-1790, arxbj.cls:1665-1809). Their bindings load this beside their own
//! options, math and frontmatter.
//!
//! Bibliography note: these classes use `natbib` + `\bibitem`/`thebibliography`, so a bibitem produces `ltx:bibitem >
//! ltx:bibblock` (Flow.model). The structured `ltx:bib-*` vocabulary is schema-valid only inside `ltx:bibentry` (the
//! BibTeX path), NOT inside `bibblock`, so the `\b*` field macros are PASSTHROUGH text (readable + schema-valid), not
//! mapped to `ltx:bib-*`.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  RequirePackage!("bm");        // \boldsymbol
  Let!("\\bolds", "\\boldsymbol");

  // ---- structured bibliography (passthrough) -------------------------------
  // Entry environments: transparent (swallow the optional [type] arg, pass body).
  for env in [
    "barticle","bbook","bincollection","binproceedings","binbook","bproceedings",
    "btechreport","bmanual","bmastersthesis","bphdthesis","bbooklet",
    "bunpublished","bmisc",
  ] {
    def_macro_noop(&format!("\\{env}[]"))?;
    def_macro_noop(&format!("\\end{env}"))?;
  }
  // Field markup macros: emit their content as text.
  for m in [
    "bsnm","bfnm","binits","bparticle","bsuffix",
    "btitle","bjournal","bbooktitle","bseries","bvolume","byear","bpages",
    "bedition","bpublisher","baddress","blocation","borganization","binstitution",
    "bschool","btype","bnumber","bchapter","bhowpublished","bnote","banumber",
    "bisbn","betal",
  ] {
    def_macro_identity(&format!("\\{m}{{}}"))?;
  }
  // An author or editor prints surname and initials, the first name gobbled (1203.0186's arxbj.cls:2633
  // `\bbl@bauthor`), and `\bid{mr=…,doi=…}` and `\bmrnumber{MR…}` print the MathReviews number through `\MR` (:2614
  // `\get@MR`), the other keys nothing (:2652-2656; 1205.6055's arximspdf.cls:2777-2784 adds pmid, pmcid, mid):
  // 1003.1189's 46 `mr=` ids printed nothing and its authors "Abramovich, FelixF.".
  RequirePackage!("keyval");
  RawTeX!(r"\def\bauthor#1{{\let\bfnm\@gobble#1}}
\def\beditor#1{{\let\bfnm\@gobble#1}}
\def\lx@ims@getMR#1R#2 #3\end{\MR{#2}}
\define@key{bid}{mr}{\lx@ims@getMR MR#1 \end\ignorespaces}
\define@key{bid}{doi}{}\define@key{bid}{pubmed}{}\define@key{bid}{pii}{}\define@key{bid}{issn}{}
\define@key{bid}{pmid}{}\define@key{bid}{pmcid}{}\define@key{bid}{mid}{}
\def\bid#1{\setkeys{bid}{#1}}
\def\bmrnumber#1{\lx@ims@getMR#1 \end\ignorespaces}");
  DefMacro!("\\AND", "and ");
  def_macro_noop("\\bptok{}")?;
  def_macro_noop("\\bptnote{}")?;
  DefMacro!("\\MR{}", "MR#1");         // MathReviews id, as text
  def_macro_noop("\\endbibitem")?;

  // ---- proofs --------------------------------------------------------------
  // `{pf}` (arximspdf.cls:1428-1434, vtexthm) runs `\proofname\proof@sep` ("Proof.") in before its body and ends with
  // `\@qed`, the class's `\qed` (`\theqed`, a □ ending the line; :1417-1426); `{pf*}{<name>}` names it. `\noqed` drops
  // the next automatic □; `\upqed`/`\rightqed`/`\qedbreak` only move it. The classes have no `{proof}` and load no
  // amsthm, so `pf` as `\begin{proof}` was undefined (12 arximspdf papers of run 329: 1205.6055, 1104.1047, 1001.4028;
  // arxbj 1003.1189), and `\upqed` undefined in 5. Defined outright, as the classes do (`\@namedef{endpf*}`), over an
  // earlier support binding's `{pf}` (elsart_support's `\@ifundefined{pf}`, which arxbj loads first).
  DefMacro!("\\proofname", "Proof");
  DefConstructor!("\\lx@ims@proof{}", "<ltx:proof><ltx:title class='ltx_runin'>#1</ltx:title>");
  DefConstructor!("\\lx@ims@endproof", sub[document, _args] {
    document.maybe_close_element("ltx:proof")?;
  });
  RawTeX!(r"\def\theqed{\ensuremath{\square}}
\def\qed{\theqed}
\let\@qed\qed
\def\noqed{\let\sv@qed\@qed\def\@qed{\global\let\@qed\sv@qed}}
\def\pf{\par\lx@ims@proof{\proofname.}}\def\endpf{\@qed\par\lx@ims@endproof}
\@namedef{pf*}#1{\par\lx@ims@proof{#1.}}\@namedef{endpf*}{\@qed\par\lx@ims@endproof}");
  for cs in ["upqed","qedbreak","rightqed"] { def_macro_noop(&format!("\\{cs}"))?; }

  // ---- tables (arximspdf.cls:1729-1790, 1838-1848) ---------------------------
  // `\tablewidth` is the classes' dimen for a table's caption and notes (11 arximspdf papers of run 329 set it;
  // arxbj 1203.0186).
  RawTeX!(r"\newdimen\tablewidth \tablewidth\textwidth");
  // `\tabnotetext[<mark>]{<label>}{<text>}` sets a table note below the tabular; `\tabnoteref[<mark>]{<label>}` and
  // `\tabnotemark[<mark>]{<label>}` print its mark in a cell (:1745-1764). The note's text is kept as a paragraph of the
  // table, and an explicit `[<mark>]` as a superscript on both sides; the class's automatic numbering is not modelled.
  DefMacro!("\\tabnotetext[]{}{}", "\\par\\lx@ims@tabnotemark{#1}#3\\par");
  DefMacro!("\\tabnoteref[]{}", "\\lx@ims@tabnotemark{#1}");
  DefMacro!("\\tabnotemark[]{}", "\\lx@ims@tabnotemark{#1}");
  RawTeX!(r"\def\lx@ims@tabnotemark#1{\if\relax\detokenize{#1}\relax\else\textsuperscript{#1}\fi}");

  // ---- the URL, attribute and script helpers of the VTeX classes (arximspdf.cls, arxstspdf.cls, arxbj.cls) ---------
  // `\doiurl{<doi>}`, `\arxivurl{<id>}` (0911.2919's copy :3057-3140, as arxstspdf.cls:2902-2975) and `\surl{<url>}`
  // (:3140-3162): a link to `http://dx.doi.org/<doi>`, `http://arxiv.org/abs/<id>` or `http://<url>`, printing the
  // argument; a full URL links as given, and a `\\` in it is only a break hint. 92 of 100 sampled copies define them
  // (run 340 first errors: `\doiurl` 288, `\arxivurl` 264, `\surl` 55; 1309.4864, 1309.5877, 1309.1380).
  DefMacro!("\\doiurl{}", sub[(arg)] { class_url_link("http://dx.doi.org/", "http://", arg) });
  DefMacro!("\\arxivurl{}", sub[(arg)] { class_url_link("http://arxiv.org/abs/", "http://", arg) });
  DefMacro!("\\surl{}", sub[(arg)] { class_url_link("http://", "://", arg) });
  // :336-338 `\setattribute[…]{group}{key}{value}` and `\getattribute{group}{key}` (89 first errors, 1107.3284);
  // `\tsup`/`\tsub` (`\textsuperscript`/`\textsubscript`, 1107.4192); `\OrigBibText…\endOrigBibText`, the source text
  // of a reference kept for production (`\long\def\OrigBibText#1\endOrigBibText{}`; 12 of the sampled papers).
  RawTeX!(
    r"\def\setattribute{\@ifnextchar[\@setattribute{\@setattribute[]}}
\def\@setattribute[#1]#2#3#4{\expandafter\gdef\csname #2@#3\endcsname{#4}}
\def\getattribute#1#2{\csname #1@#2\endcsname}
\let\tsup\textsuperscript
\let\tsub\textsubscript
\ifx\OrigBibText\undefined \long\def\OrigBibText#1\endOrigBibText{}\fi"
  );

  // ---- keywords (arxbj.cls:2490-2541 of 1106.2467's copy; arximspdf.cls:2404-2450 of 1107.3284's) -----------------
  // `{keyword}[class=<class>]` with `\kwd[<sep>]{<text>}`: a list of entries joined by `\sep` (none before the first,
  // an explicit `[<sep>]` in its place: `\kwd[Primary ]{60F05}\kwd[; secondary ]{62G20}`), headed by the class's
  // `\keyword@<class>` (`\setattribute{keyword}{KWD}{Keywords:}`, `{AMS}{AMS 2000 subject classifications:}`; each
  // class binding sets its own heading and `\sep`). The blocks add up, as the classes keep each (arxbj's
  // `\keyword@box`, arximspdf's `\keyword@toks`): a keywords block per `{keyword}`, and the AMS block the paper's
  // subject classification. Each `\kwd` had started the list afresh (elsart's `{keyword}`: arxbj 1106.2467, 0811.0643
  // lost every entry) or a second block replaced the first (arximspdf's: the AMS codes of 1107.3284 and 1309.4864).
  RequirePackage!("keyval");
  DefMacro!(T_CS!("\\begin{keyword}"), None, "\\begingroup\\keyword");
  DefMacro!(T_CS!("\\end{keyword}"), None, "\\endkeyword\\endgroup");
  RawTeX!(r"\def\keyword{\@ifnextchar[\lx@ims@keyword{\lx@ims@keyword[class=KWD]}}
\define@key{lx@ims@keyword}{class}{\def\keyword@class{#1}}
\def\lx@ims@AMS{AMS}
\def\lx@ims@keyword[#1]{\def\keyword@class{KWD}\setkeys{lx@ims@keyword}{#1}\let\kwd@sep\@empty
\ifx\keyword@class\lx@ims@AMS\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi
{\lx@add@frontmatter@until{ltx:classification}[scheme=AMS,name={\csname keyword@\keyword@class\endcsname~}]{\lx@end@keywords}}
{\lx@add@frontmatter@until{ltx:keywords}[accumulate=true,name={\csname keyword@\keyword@class\endcsname~}]{\lx@end@keywords}}}
\def\endkeyword{\lx@end@keywords}
\def\sep{\unskip, }
\DeclareRobustCommand*\kwd{\@ifnextchar[\@kwd{\@kwd[\kwd@sep]}}
\def\@kwd[#1]#2{\unskip#1{#2}\let\kwd@sep\sep}
\setattribute{keyword}{KWD}{Keywords:}
\setattribute{keyword}{AMS}{AMS 2000 subject classifications:}");

  // ---- theorems (vtexthm.sty, which the classes carry: arxbj.cls:1302-1345 of 1106.2467's copy, arximspdf.cls:
  // 1287-1337 of 1107.3284's) ---------------------------------------------------------------------------------------
  // Every `\newtheorem` also defines an unnumbered `{<name>*}` twin (`\@namedef{#1*}{…\@thmstar{…}}`), and
  // `\newproclaim` is `\newtheorem` in the class's other style (:1418-1420). Starred statements: arxbj 15 of the 45
  // sampled papers (`{cnd*}`, `{assumptions*}` 1205.2947, `{Comments*}` 1106.2467, `{rem*}` 1307.5990, `{thm*}`
  // 1309.7754); arximspdf `{hy*}` 1107.3284. The theorems are amsthm's, borrowed (left unloaded, so a paper's own
  // `\usepackage{amsthm}` is a real load, as after the class in LaTeX; theorem_sty.rs).
  require_package_borrowed("amsthm")?;
  RawTeX!(
    r"\let\lx@vtex@newtheorem\newtheorem
\def\lx@vtex@thm{\@ifstar{\lx@vtex@newtheorem*}{\lx@vtex@thm@}}
\def\lx@vtex@thm@#1{\@ifnextchar[{\lx@vtex@othm{#1}}{\lx@vtex@nthm{#1}}}
\def\lx@vtex@othm#1[#2]#3{\lx@vtex@newtheorem{#1}[#2]{#3}\lx@vtex@star{#1}{#3}}
\def\lx@vtex@nthm#1#2{\@ifnextchar[{\lx@vtex@xnthm{#1}{#2}}{\lx@vtex@newtheorem{#1}{#2}\lx@vtex@star{#1}{#2}}}
\def\lx@vtex@xnthm#1#2[#3]{\lx@vtex@newtheorem{#1}{#2}[#3]\lx@vtex@star{#1}{#2}}
\def\lx@vtex@star#1#2{\@ifundefined{#1*}{\lx@vtex@newtheorem*{#1*}{#2}}{}}
\def\newtheorem{\theoremstyle{plain}\lx@vtex@thm}
\def\newproclaim{\theoremstyle{plain}\lx@vtex@thm}"
  );
  // The value lists both classes define, `\setvaluelist{list}{a,b,…}` and `\getitemvalue{list}{n}` (arxbj.cls
  // :495-502, 0811.0643; arximspdf.cls :533-540, 1203.0565, which numbers its equations by one).
  RawTeX!(
    r"\def\setvaluelist#1#2{\@tempcnta=0\relax\@for\@curr@val:=#2\do{\advance\@tempcnta by1\relax\expandafter\protected@xdef\csname #1@item@\the\@tempcnta\endcsname{\@curr@val}}\expandafter\protected@xdef\csname #1@item@0\endcsname{\the\@tempcnta}}
\def\getitemvalue#1#2{\csname #1@item@#2\endcsname}"
  );
});

/// `\href{<base><arg>}{<arg>}`, or `\href{<arg>}{<arg>}` when the argument is already a full URL (it contains `full`:
/// `http://` for `\doiurl`/`\arxivurl`, `://` for `\surl`), dropping the `\\` break hints.
fn class_url_link(base: &'static str, full: &'static str, arg: Tokens) -> Result<Tokens> {
  let text: Vec<Token> = arg
    .unlist()
    .into_iter()
    .filter(|t| *t != T_CS!("\\\\"))
    .collect();
  let full = Tokens::new(text.clone()).untex().contains(full);
  let mut out = vec![T_CS!("\\href"), T_BEGIN!()];
  if !full {
    out.extend(Tokenize!(base).unlist());
  }
  out.extend(text.iter().copied());
  out.extend([T_END!(), T_BEGIN!()]);
  out.extend(text);
  out.push(T_END!());
  Ok(Tokens::new(out))
}
