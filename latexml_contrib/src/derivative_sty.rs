//! Binding for `derivative.sty` — Simon Jensen's expl3 package of derivative and
//! differential operators (`\odv`, `\pdv`, `\mdv`, `\fdv`, `\adv`, `\jdv` and the
//! matching differentials `\odif`, `\pdif`, `\mdif`, `\fdif`, `\adif`, `\jdif`,
//! plus user-declared ones via `\DeclareDerivative`/`\DeclareDifferential`).
//!
//! There is no Perl binding, and physics.sty only covers the overlap
//! (`\dv`/`\pdv`/`\fdv`/`\dd`), so a paper's derivative-only commands — most
//! commonly `\mdif` (material differential) — leaked as undefined control
//! sequences without raw TeX loading. The package is ~1800 lines of expl3 that
//! our engine already executes correctly (verified: raw-loading it under
//! `--includestyles` converts cleanly), so — exactly like `commath_sty` — the
//! faithful binding just force-raw-loads the real `.sty` rather than
//! reimplementing it. `noltxml` bypasses the default "raw loading is off" gate;
//! the `\usepackage` options (e.g. `[italic=true]`) are forwarded by
//! `InputDefinitions`. Reading a host texmf `.sty` via kpathsea is in scope
//! (CLAUDE.md Guiding Principles). Witness html_feedback#6876 (arXiv:2311.15365v3,
//! `[italic=true]`, 36 uses of `\mdif`).
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("derivative", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});
