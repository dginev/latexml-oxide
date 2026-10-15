//! Stub for asme2ej.cls (ASME journal class).
use latexml_package::prelude::*;

LoadDefinitions!({
  LoadClass!("OmniBus");
  RequirePackage!("amsmath");
  // NO eager amsthm preload (the real asme2ej.cls loads neither amsmath nor
  // amsthm — only ifthen/times/mathptm/ifpdf — and defines its OWN trivlist
  // `\proof`/`\endproof` at L1242-1245). Eager-loading amsthm here is the
  // eager-preload anti-pattern: a paper that does `\let\proof\relax` (to clear
  // the class's conflicting proof) and THEN `\usepackage{amsthm}` to install
  // amsthm's proof env — exactly the documented asme2ej idiom — finds amsthm
  // "already loaded" (so its `\usepackage` is a no-op), leaving `\proof`
  // relaxed → `{proof}` undefined where Perl (which raw-loads the class, never
  // pre-loading amsthm) is clean. Let the document's own `\usepackage{amsthm}`
  // be the first/real load. Witness 2102.03856 (`\let\proof\relax` +
  // `\usepackage{...,amsthm}` + `\begin{proof}`).
  // Eager xcolor preload removed for Perl parity: it makes a later document
  // xcolor[table] load a no-op, so colortbl/array never load and array m{}/b{}
  // columns break (Unrecognized tabular template -> Extra alignment tab). The
  // document loads xcolor itself; color/definecolor stay via hyperref->color.
  // See ifacconf_cls.rs and SYNC_STATUS (eager-xcolor cluster).
  RequirePackage!("hyperref");

  // ASME-specific frontmatter — preserve author content.
  DefMacro!(
    "\\setauthorname{}",
    "\\@add@frontmatter{ltx:note}[role=authorname]{#1}"
  );
  DefMacro!(
    "\\manuscriptnotenumber{}",
    "\\@add@frontmatter{ltx:note}[role=manuscriptno]{#1}"
  );
  DefMacro!(
    "\\confname{}",
    "\\@add@frontmatter{ltx:note}[role=conference]{#1}"
  );
  DefMacro!(
    "\\confyear{}",
    "\\@add@frontmatter{ltx:note}[role=year]{#1}"
  );

  // asme2ej.cls:301-311 (1212.4243's copy): the class's Helvetica fonts, which papers set headings and tables in
  // (`\tensfb` 3 papers of the sample: 1805.06800).
  RawTeX!(
    r"\DeclareFixedFont{\ninsfb}{OT1}{phv}{b}{n}{9}\DeclareFixedFont{\ninsf}{OT1}{phv}{m}{n}{9}
\DeclareFixedFont{\egtsf}{OT1}{phv}{m}{n}{8}\DeclareFixedFont{\egtsfb}{OT1}{phv}{b}{n}{8}
\DeclareFixedFont{\sevsf}{OT1}{phv}{m}{n}{7}\DeclareFixedFont{\bannerfnt}{OT1}{phv}{b}{n}{10}
\DeclareFixedFont{\pnumfnt}{OT1}{phv}{b}{n}{18}\DeclareFixedFont{\twlsfb}{OT1}{phv}{b}{n}{12}
\DeclareFixedFont{\tensfb}{OT1}{phv}{b}{n}{10}\DeclareFixedFont{\tensf}{OT1}{phv}{m}{n}{10}
\DeclareFixedFont{\phvtf}{OT1}{phv}{b}{n}{18}"
  );
  // asme2ej.cls:1218-1221 `{acknowledgment}`, the acknowledgements under their heading (93 papers of the inventory:
  // 1011.2269, 1012.2685, 1311.1922).
  DefEnvironment!("{acknowledgment}",
    "<ltx:acknowledgements name='Acknowledgements'>#body</ltx:acknowledgements>",
    mode => "internal_vertical");
  // asme2ej.cls:1209-1216 `{nomenclature}`: under the heading "Nomenclature", each `\entry{symbol}{meaning}` a
  // paragraph, the symbol, a quad, the meaning (33 and 31 papers: 1212.4243, 1403.0296).
  RawTeX!(
    r"\newenvironment{nomenclature}{\def\entry##1##2{\par\noindent{##1}\quad\ignorespaces##2\par}\section*{Nomenclature}}{\par}"
  );
  // asme2ej.cls L1242-1245 defines its OWN trivlist `proof` environment.
  // Since we LoadClass("OmniBus") (not the real .cls), port it so asme2ej
  // papers that use `\begin{proof}` WITHOUT loading amsthm still render (as
  // Perl does, raw-loading the class). A paper that DOES want amsthm's proof
  // uses the documented `\let\proof\relax \let\endproof\relax` idiom first,
  // which clears this so its later `\usepackage{amsthm}` can install amsthm's
  // version (witness 2102.03856).
  RawTeX!(
    r"\def\proof{\@ifnextchar[{\@optargproof}{\@proof}}
\def\@proof{\trivlist \item[\hskip \labelsep{\it Proof.}]}
\def\@optargproof[#1]{\trivlist \item[\hskip \labelsep{\it #1.}]}
\def\endproof{\endtrivlist}"
  );
  // asme2ej.cls:1246 (in the same `\AtEndOfClass`): `\qed`, a `\Box` at the right (4 papers: 1610.03468); the class
  // backs up over the last line to set it there, print layout.
  RawTeX!(r"\def\qed{\unskip\nobreak\hfill$\Box$}");
});
