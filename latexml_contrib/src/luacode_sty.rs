//! luacode.sty — Lua-in-LaTeX helpers (Manuel Pégourié-Gonnard), implemented
//! on the engine's `\directlua` texlua bridge (`latexml_engine::lua_bridge`;
//! user directive 2026-08-31 on first-principles LuaTeX-escape support).
//!
//! The real package aborts on non-LuaTeX engines ("LuaTeX is required"), so
//! raw-loading is impossible by design — this binding stands in, mapping the
//! public API onto `\directlua`, which is exactly how the real package
//! implements it on LuaTeX (luacode.sty: `\luacode@dbg@exec` →
//! `\directlua`). The catcode-taming differences between the variants are
//! faithfully reduced to their observable effect: how much expansion the
//! body sees before reaching Lua.
use latexml_package::prelude::*;

LoadDefinitions!({
  // \luadirect{<lua>}: \directlua with luacode's debug wrapper — same
  // evaluation. \luaexec{<lua>}: additionally makes \\, \%, \# usable;
  // the expansion path already yields their character forms.
  DefMacro!("\\luadirect{}", "\\lx@directlua{#1}");
  DefMacro!("\\luaexec{}", "\\lx@directlua{#1}");

  // \luastring{<text>}   — full expansion, then a quoted Lua string literal;
  // \luastringO{<text>}  — expand once;
  // \luastringN{<text>}  — no expansion (detokenized literal).
  // All three reduce to "…escaped…" via the engine's \luaescapestring.
  DefMacro!("\\luastring{}", "\"\\lx@luaescapestring{#1}\"");
  DefMacro!("\\luastringO{}", "\"\\lx@luaescapestring{#1}\"");
  DefMacro!("\\luastringN{}", "\"\\lx@luaescapestring{\\detokenize{#1}}\"");

  // Debug-log toggles: affect console tracing only.
  def_macro_noop("\\LuaCodeDebugOn")?;
  def_macro_noop("\\LuaCodeDebugOff")?;

  // {luacode} / {luacode*}: the body is Lua source, read RAW line-by-line to
  // the matching \end (luacode* fully verbatim; plain luacode allows macros
  // in the body — the corpus's bodies are Lua either way, and any TeX macro
  // in them would have been flattened by \directlua's expansion too), then
  // executed as ONE chunk; whatever it tex.print()s re-enters the input
  // stream here. Mirrors the filecontents capture shape (the \begin group is
  // closed manually, standing in for the consumed \end).
  fn run_luacode_env(env: &str) -> Result<()> {
    let end_marker = s!("\\end{{{env}}}");
    read_raw_line(); // remainder of the \begin{...} line — not Lua
    let mut lines: Vec<String> = Vec::new();
    loop {
      match read_raw_line() {
        Some(line) if !line.contains(end_marker.as_str()) => lines.push(line),
        _ => break,
      }
    }
    let chunk = lines.join("\n");
    // Re-inject the consumed `\end{...}` so the normal environment-close
    // machinery balances the `\begin` (raw capture ate the whole line), then
    // the chunk's output ABOVE it so the printed tokens are read first.
    unread(Tokenize!(TeXString::assembled(s!("\\end{{{env}}}"))));
    match latexml_engine::lua_bridge::lua_exec(&chunk) {
      Ok(out) if !out.is_empty() => {
        unread(Tokenize!(TeXString::assembled(out)));
      },
      Ok(_) => {},
      Err(msg) => {
        Info!("lua", env, s!("{{{env}}} chunk not evaluated: {msg}"));
      },
    }
    Ok(())
  }
  DefPrimitive!(T_CS!("\\luacode"), None, {
    run_luacode_env("luacode")?;
  });
  DefPrimitive!(T_CS!("\\luacode*"), None, {
    run_luacode_env("luacode*")?;
  });
});
