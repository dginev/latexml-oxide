//! textpos: the raw package, with an absolute block placed where it is written.
//!
//! In absolute mode (`[absolute]`, `\textblockorigin`) textpos collects each block in `\TP@holdbox`
//! (textpos.sty:372-376) and draws the hold box only from a shipout hook (`shipout/background` or
//! `/foreground`, :389-397; `\EveryShipout` on old kernels). LaTeXML ships no pages, so the hook never fires and
//! every absolute block was lost, without a diagnostic (Perl has no textpos binding and loses them too). A block is
//! content the author placed on the page; with one page and no coordinates, it is placed where it is written —
//! in a margin note it stays in the note (OXIDIZED_DESIGN_DIVERGENCES #416). What is placed is the block's text box
//! at its natural size (`\TP@textbox`, :257): the hold box holds it inside textpos's zero-size positioning boxes
//! (:326-371), which a renderer draws over the text around it. The hold box is emptied as the block is placed, so
//! the end-of-document flush (:92) finds nothing; `discardcontent` (no `\ifTP@displayholdbox`) and `noshowtext`
//! (no `\ifTP@showtext`; pdflatex draws an empty labelled frame) place nothing, nor does a block that shows nothing
//! (`typesets_content`, as eso-pic's one-shot overlay): autonum.sty:159-173 captures each display environment's
//! `\\` and `\label` in an absolute block at (0,0) holding an empty `equation`, which became an empty numbered
//! equation after the abstract (2605.31413). A block of rules alone is placed decoration and is dropped too, as
//! eso-pic's rule-only overlay is. Firing the shipout hooks at the end
//! instead would move every block out of reading order and run every package's page furniture (eso-pic,
//! draftwatermark, background). Witnesses: pdfcomment example ×3, stubs_ex, niepraschk-eso-pic, ftc-notebook.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("textpos", noltxml => true, extension => Some(Cow::Borrowed("sty")));
  // `\box`, unless the box shows nothing.
  DefPrimitive!("\\lx@TP@placebox Number", sub[(number)] {
    match remove_value(&s!("box{}", number.value_of())) {
      Some(Stored::Digested(block)) if typesets_content(&block) => Ok(vec![block]),
      _ => Ok(Vec::new()),
    }
  });
  RawTeX!(
    r"\newbox\lx@TP@textbox
\let\lx@TP@endtextblock\TP@endtextblock
\def\TP@endtextblock#1#2#3#4{\ifTP@abspos\setbox\lx@TP@textbox\copy\TP@textbox\fi
  \lx@TP@endtextblock{#1}{#2}{#3}{#4}%
  \ifTP@abspos\global\setbox\TP@holdbox\box\voidb@x
    \ifTP@displayholdbox\ifTP@showtext\lx@TP@placebox\lx@TP@textbox\fi\fi
  \fi}"
  );
});
