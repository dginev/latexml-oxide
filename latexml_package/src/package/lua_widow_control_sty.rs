//! lua-widow-control.sty — LuaTeX widow/orphan control. Page-layout only, and
//! its engine half is Lua: `\__lwc_enable:` and friends are created by the Lua
//! module loaded at lua-widow-control.sty:120-123, which no reader here runs, so
//! under the luatex profile the raw package leaves them undefined for the
//! `.code` keys (:232-238) and hooks (:174) that call them (homework-demo-*,
//! jwjournal-demo-cn, abntexto ×2). The user surface (`\lwcsetup`,
//! `\lwcenable`/`\lwcdisable`, `\iflwc`, `\lwcemergencystretch`,
//! `\lwcdisablecmd`) is kept with no layout effect. Outside the luatex profile
//! the package's own `no-luatex` critical (:68-70) is reported instead.
//! Guard: `perfect_kernel_batch56::lua_widow_control_surface`.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  if !lookup_bool("LUATEX_PROFILE") {
    Error!("latex", "lua-widow-control",
      "Package lua-widow-control Error: LuaTeX is REQUIRED for lua-widow-control (lua-widow-control.sty:68)");
    return Ok(());
  }
  RequirePackage!("expl3");
  RawTeX!(r"\newif\iflwc
\ExplSyntaxOn
\cs_set_protected:Npn \__lwc_enable: { \lwctrue }
\cs_set_protected:Npn \__lwc_disable: { \lwcfalse }
\prg_set_conditional:Npnn \__lwc_if_enabled: { p, T, F, TF }
  { \iflwc \prg_return_true: \else \prg_return_false: \fi }
\cs_set_eq:NN \lwcenable \__lwc_enable:
\cs_set_eq:NN \lwcdisable \__lwc_disable:
\cs_set_protected:Npn \lwcsetup #1 { }
\cs_set_protected:Npn \lwcemergencystretch { }
\cs_set_protected:Npn \lwcdisablecmd #1 { }
\ExplSyntaxOff
\lwctrue");
});
