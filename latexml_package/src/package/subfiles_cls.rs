use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl ships only `subfiles.sty.ltxml`. Author TeX commonly writes
  // `\documentclass{subfiles}{main}` — in Perl that falls through to the
  // sty binding via LaTeXML's cls→sty lookup chain. Rust has an explicit
  // BINDINGS registry, so we wire the same fallthrough by hand: load
  // OmniBus as the outer class skeleton, then pull in the fully-ported
  // `subfiles.sty` logic (fake `\begin{document}` / `\end{document}`,
  // nesting counter, `\subfile` = `\input`). This replaces the previous
  // punt-with-Error stub that silently dropped subfiles semantics.
  // subfiles.cls:38-40 `\LoadClass`es the main file's class, unknown here: OmniBus's guess, which keeps the kernel's
  // `\chapter` as for any unknown class (OXIDIZED_DESIGN #179 (c); 1906.09200's chapters are subfiles of a memoir book).
  assign_value("OmniBus:fallback", true, Some(Scope::Global));
  load_class("OmniBus", Vec::new(), Tokens!())?;
  RequirePackage!("subfiles");
});
