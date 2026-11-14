use latexml_package::prelude::*;

LoadDefinitions!({
  // pinlabel.sty (MSP's figure labelling: `\labellist \pinlabel <label> [<pos>] <<dx,dy>> at <x> <y> … \endlabellist`
  // before the figure) is read raw: its own label parser and its `\includegraphics` → `\psfig` translation
  // (pinlabel.sty:236, 306-314) run as in TeX, so every label's text is kept with the figure. LaTeXML has no
  // PostScript-coordinate overlay, so the labels follow the image instead of sitting on it. The former stub only
  // warned, and each `\labellist`/`\pinlabel`/`\endlabellist` was undefined: 85 of run 336's first 265k papers
  // (math0412330's seven labels on its conormal-bundle figure among them).
  InputDefinitions!("pinlabel", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // Each driver branch's `\ps@begin` places the picture: the pdfTeX one through graphicx (pinlabel.sty:281-287,
  // `<stem>.pdf`), the DVI one as dvips's `\special{PSfile=…}` (:254-258), which LaTeXML does not render — so a
  // latex+dvips paper (EPS figures, `\pdfoutput=0`) kept the labels and lost the picture (1010.6236 12 → 2 images,
  // 1112.5970 30 → 1). Both include through graphicx here: the file that driver places (DVI: the .ps/.eps/.pdf
  // `\scan@header` found, :566-606), at the psfig size; a side no bounding box sized is left to the image. Only when
  // pinlabel defined its drivers: it stops at :231 when another psfig already defined `\ps@init`, before `\ifdvi`.
  RawTeX!(
    r"\@ifundefined{ifdvi}{}{\def\ps@begin{%
  \edef\@tempa{[\ifdim\@p@swidth sp>\z@ width=\@p@swidth sp,\fi\ifdim\@p@sheight sp>\z@ height=\@p@sheight sp\fi]%
    {\ifdvi\@p@sfile\else\@p@dffile\fi}}%
  \rlap{\smash{\expandafter\@includegraphics@\@tempa}}}}"
  );
});
