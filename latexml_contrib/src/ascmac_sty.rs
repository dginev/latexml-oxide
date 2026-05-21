//! Stub for ascmac.sty — a Japanese LaTeX2e add-on (part of `jsclasses`
//! family) that provides boxed environments such as `{itembox}`,
//! `{screen}`, `{shadebox}`, `{boxnote}`. Not in standard TeX Live.
//!
//! Witness: 2601.09339 (`\usepackage{ascmac, fancybox}` +
//! `\begin{itembox}[l]{title}` body `\end{itembox}`). We treat the
//! boxed envs as transparent paragraphs and preserve the title as
//! ltx:note, so author content is not dropped.
use latexml_package::prelude::*;

LoadDefinitions!({
  // {itembox}[align]{title} ... \end{itembox}
  // Render as a transparent block; preserve title via ltx:note.
  DefEnvironment!(
    "{itembox}[]{}",
    "<ltx:para><ltx:p><ltx:note role='itembox-title'>#2</ltx:note>#body</ltx:p></ltx:para>",
    mode => "internal_vertical"
  );
  // {screen}, {boxnote}, {shadebox} take no args.
  DefEnvironment!(
    "{screen}",
    "<ltx:para><ltx:p>#body</ltx:p></ltx:para>",
    mode => "internal_vertical"
  );
  DefEnvironment!(
    "{boxnote}",
    "<ltx:para><ltx:p>#body</ltx:p></ltx:para>",
    mode => "internal_vertical"
  );
  DefEnvironment!(
    "{shadebox}",
    "<ltx:para><ltx:p>#body</ltx:p></ltx:para>",
    mode => "internal_vertical"
  );
});
