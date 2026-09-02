use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl pgfplots.sty.ltxml — port handles the same InputDefinitions
  // flow that pulls in the raw pgfplots.sty on top of our pgf-latexml
  // shim. Perl L24 marks `\pgfplots@iffileexists` `locked => 1` so the
  // raw-TeX load can't clobber our \IfFileExists alias; Rust now mirrors.
  //
  // Still unported (Perl L27-33): compat-mode detection + autoset to
  // `mostrecent`. That requires Expand of `\pgfk@/pgfplots/compat/*` CSes
  // from the raw-sty body — safe to omit since pgfplots defaults to a
  // usable compat level without the autoset.
  DefMacro!("\\pgfplots@iffileexists", "\\IfFileExists", locked => true);
  // pgfplotscore.code.tex:74-89: `\def\pgfplotsenablelua{0}` is pgfplots' own
  // boot-time switch for its Lua backend. Under the `[luatex]` profile
  // `\directlua` exists (the external-texlua bridge), so `lua support=auto`
  // (:64-70) took the LuaTeX path whose `tex.enableprimitives` bootstrap
  // (ltluatex.tex:63, :117) cannot reach this engine — `undefined:
  // \pgfplotsglobalretval`, `\pgfplotsutil@savecatcodetable` on every axis
  // (colorblind_doc, srdp-mathematik, sunpath; Perl has no such profile). The
  // Lua backend is only a coordinate accelerator: output is identical. Guard:
  // `perfect_kernel_batch54::pgfplots_lua_backend_is_off_under_luatex_profile`.
  DefMacro!("\\pgfplotsenablelua", "0");
  InputDefinitions!("pgfplots", noltxml => true, extension => Some(Cow::Borrowed("sty")));
});

// (Resolved 2026-04-25) `\pgfplots@curplotlist`/`\pgfplots@curlegend`
// undefined-CS cluster traced to a Rust core bug, NOT a pgfplots-shim
// issue. `\pgfplots@pop@next@legend` (raw pgfplots.code.tex L5813-5827)
// uses the `\def\foo{{\globaldefs=1 \let\x=\relax}}` idiom to make
// local lets globally effective. Rust's `assign_internal` (state.rs)
// only matched `Stored::Int(1)` for the `\globaldefs` magic, but the
// register stores as `Stored::Number(1)` — so the override silently
// failed and the lets popped on group exit, leaving the CSes undefined
// and `\pgfplots@createlegend` looping at the digest wall-clock cap.
