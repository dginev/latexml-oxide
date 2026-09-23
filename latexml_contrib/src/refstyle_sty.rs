//! refstyle.sty — flexible cross-reference styling (`\newref{type}{...}`,
//! `\figref`, `\vref`, `\Vref`, `\Ref`, ...).
//!
//! The real package is loaded raw, with its options: refstyle.sty and its
//! refstyle.cfg are pure TeX that the engine runs clean — the `\newref`
//! templates define `\figref`/`\secref`/`\eqref`/…, and `\RS@removedef`
//! (`\let\eqref\@undefined`) is seen as undefined by `\@ifundefined`
//! (latex.ltx:1729-1735), so the amsmath `\eqref` redefinition does not fire
//! refstyle's "already defined" error. The former stub (`\newref` a no-op,
//! `\vref` → `\ref`) rested on a mis-probed `\@ifundefined` gap; it left every
//! `\newref`-built command undefined and lacked the internals documents call
//! directly — LyX preambles emit `\RS@ifundefined{subref}{…}` (refstyle.sty:51-57;
//! uspatent/PatentApplicationGuide: an `<ERROR>` in the preamble stranded the
//! title, 3 schema errors). Perl has no refstyle binding (raw only under
//! `--includestyles`, 0 errors on that witness).
//!
//! Witnesses: arXiv:2009.10518, arXiv:1804.06350 (refstyle + amsmath +
//! cleveref), uspatent/PatentApplicationGuide.
//! Guard: `perfect_kernel_batch56::refstyle_loads_raw_with_its_internals`.
use latexml_package::prelude::*;

LoadDefinitions!({
  let opts: Vec<String> = lookup_vecdeque("opt@refstyle.sty")
    .map(|v| v.iter().map(|o| o.to_string()).collect())
    .unwrap_or_default();
  InputDefinitions!("refstyle", noltxml => true, extension => Some(Cow::Borrowed("sty")),
    handleoptions => true, options => opts);
});
