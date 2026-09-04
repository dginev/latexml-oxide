use std::path::Path;

use once_cell::sync::Lazy;

///**********************************************************************
/// Organized following
///  "`LaTeX`: A Document Preparation System"
///   by Leslie Lamport
///   2nd edition
/// Addison Wesley, 1994
/// Appendix C. Reference Manual
///**********************************************************************
/// NOTE: This will be loaded after `TeX.pool`, so it inherits.
///**********************************************************************
use crate::prelude::*;

// Process-once cached env vars (see WISDOM #56 — getenv hot-path race).
static DUMP_PATH: Lazy<Option<String>> = Lazy::new(|| std::env::var("LATEXML_DUMP_PATH").ok());
static DUMP_DIR: Lazy<Option<String>> = Lazy::new(|| std::env::var("LATEXML_DUMP_DIR").ok());
static INI_MODE: Lazy<bool> = Lazy::new(|| std::env::var_os("LATEXML_INI_MODE").is_some());
static NODUMP: Lazy<bool> = Lazy::new(|| std::env::var_os("LATEXML_NODUMP").is_some());

const DEV_DUMPS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../resources/dumps");

/// Perl `FindFile($format._dump, ...)` parity for the latex dump.
/// Returns true when any `latex.YYYY.dump.txt` is reachable through env
/// overrides, the exe-relative installed layout, the dev-tree path, or
/// the embedded fallback. Used by `LoadFormat('latex')` to decide
/// between the dump branch and the base branch.
fn latex_dump_available() -> bool {
  if *NODUMP {
    return false;
  }
  if let Some(p) = DUMP_PATH.as_deref()
    && Path::new(p).is_file()
  {
    return true;
  }
  let prefer = crate::dump_paths::detect_ambient_texlive_year();
  if let Some(dir) = DUMP_DIR.as_deref()
    && crate::dump_paths::resolve_versioned_in_dir(Path::new(dir), "latex", prefer).is_some()
  {
    return true;
  }
  if let Ok(exe) = std::env::current_exe()
    && let Some(exe_dir) = exe.parent()
  {
    let installed = exe_dir.join("../resources/dumps");
    if crate::dump_paths::resolve_versioned_in_dir(&installed, "latex", prefer).is_some() {
      return true;
    }
    if crate::dump_paths::resolve_versioned_in_dir(exe_dir, "latex", prefer).is_some() {
      return true;
    }
  }
  let dev = Path::new(DEV_DUMPS_DIR);
  if dev.is_dir() && crate::dump_paths::resolve_versioned_in_dir(dev, "latex", prefer).is_some() {
    return true;
  }
  crate::embedded_dumps::embedded_latex_dump(prefer).is_some()
}

LoadDefinitions!({
  //**********************************************************************
  // Organized following
  //  "LaTeX: A Document Preparation System"
  //   by Leslie Lamport
  //   2nd edition
  // Addison Wesley, 1994
  // Appendix C. Reference Manual
  //**********************************************************************
  // NOTE: This will be loaded after TeX.pool.ltxml, so it inherits.
  //**********************************************************************

  // Perl: LaTeX.pool.ltxml — LoadPool('TeX'); LoadFormat('latex');
  LoadPool!("TeX");

  InnerPool!(latex_bootstrap);

  // In `--init=latex.ltx` (dump-build) mode, stop after latex_bootstrap.
  // The same reasoning as in `tex.rs`'s plain branch: pre-loading
  // latex_dump / latex_base / latex_constructs pollutes the snapshot
  // and silences the diff for everything raw latex.ltx defines.
  // `LATEXML_INI_MODE=1` is set by `bin/latexml_oxide.rs` BEFORE
  // `prepare_session`, so this branch fires before latex.rs runs.
  if *INI_MODE {
    return Ok(());
  }

  // Perl `LoadFormat('latex')` strict split:
  //   if dump available: bootstrap → dump → constructs (NO base)
  //   else:              bootstrap → base → constructs (NO dump)
  if !*NODUMP && latex_dump_available() {
    if let Err(e) = crate::latex_dump::load_definitions() {
      Warn!("latex_dump", "load", s!("{}", e));
    }
  } else {
    // DEGRADED raw-load path: no precompiled LaTeX kernel dump is
    // available. This silently fires `latex.ltx` + `expl3-code.tex` raw,
    // which in turn hits known raw-load-only cascades (the expl3-code
    // L33075 codepoint dangling-group, and the `\@expl@pop@filename@@`
    // expl-status desync) that the dump avoids. Running the large canvas
    // here inflates per-paper error counts by ~1000× on expl3-heavy
    // articles (e.g. 2112.11932: 1 → 1003), masquerading as a Rust parity
    // gap when it is really a missing-kernel-dump setup error.
    //
    // The dump is REQUIRED for canvas/parity work. Surface its absence
    // loudly, once per process, unless `LATEXML_NODUMP=1` made it
    // intentional. Not an `Error:`/`Fatal:` (those are reserved for the
    // conversion log and would corrupt canvas error counts) — a plain
    // one-shot stderr banner the operator cannot miss.
    if !*NODUMP {
      crate::dump_paths::warn_degraded_no_dump();
    }
    InnerPool!(latex_base);
  }

  // latex.ltx:22071-22076: under LuaTeX/XeTeX the format inputs
  // `load-unicode-data`, lettering every General_Category L/M code point
  // (load-unicode-data.tex:98-99,134-135). The profile option is a preload,
  // processed before this format (and its dump, which pins U+0080-U+00FF
  // OTHER) loads, so the Latin-1 letters are re-lettered here at the seam;
  // code points >= U+0100 come lazily from `state::lookup_catcode`.
  // Guard: `perfect_kernel_batch56::non_ascii_letters_are_letters_under_luatex`.
  if lookup_bool("LUATEX_PROFILE") {
    ::latexml_core::state::assign_luatex_latin1_letters();
  }

  // Format-layering rule: real LaTeX (INITEX-based) never defines plain.tex's
  // tabbing shorthand `\+` (= `\tabalign`), but our latex format is layered
  // on the plain layer (dump record or raw plain.tex), which does. A stray
  // `\+` in a LaTeX document (author typo, e.g. `\!+\+` for `\!+\!`) then
  // expanded into \tabalign's \halign and detonated the mode-mismatch runaway
  // (102-error TooManyErrors fatal; witness cond-mat0001412 in the 2026-07
  // full-arXiv run) — where pdflatex gives a single undefined-CS error
  // (latex209.def compat: `\let\+\@empty`) and Perl LaTeXML, which never
  // defines `\+` in its TeX.pool, likewise reports one undefined macro.
  // Retract the inherited definition here, at the "latex kernel layer
  // complete" seam (covers both the dump and raw-base branches; skipped in
  // INI_MODE by the early return above, so dump generation is unaffected).
  // Guarded on the body still being plain's bare `\tabalign`, so a user's
  // own pre-\documentclass `\def\+` survives the lazy pool load.
  if let Ok(Some(defn)) = lookup_definition(&T_CS!("\\+"))
    && let Some(ExpansionBody::Tokens(body)) = defn.get_expansion()
    && body.len() == 1
    && body
      .unlist_ref()
      .first()
      .is_some_and(|t| t.text == pin!("\\tabalign"))
  {
    remove_meaning_global(&T_CS!("\\+"));
  }

  InnerPool!(latex_constructs);

  // Rust-only overrides — loaded LAST so they can patch CSes set up by
  // any earlier pool (dump, bootstrap, base, constructs). Without this
  // call, definitions in `latex_constructs_rust_only.rs` (e.g.
  // `\UseRawInputEncoding`, `\ltx@hard@MessageBreak`) never register.
  InnerPool!(latex_constructs_rust_only);

  // Retry any PA/MPA let-aliases whose target wasn't defined at
  // dump-load time (they were queued rather than silently dropped).
  // Classic example: `\let\a=\@tabacckludge` — `\@tabacckludge`
  // itself is defined in latex_constructs (which loads after the
  // dump), so the alias has to wait until now.
  let (applied, skipped) = dump_reader::flush_deferred_aliases();
  if applied + skipped > 0 {
    Info!(
      "latex_dump",
      "deferred",
      s!("deferred aliases: {} applied, {} skipped", applied, skipped)
    );
  }

  // Faithful `\everyjob` emulation (beyond Perl). TeX inserts `\everyjob`'s
  // token list at `main_control` start — right after the format is loaded
  // (tex.web §1030). Our LaTeX "format" is this `LoadFormat('latex')` block, so
  // fire the l3sys job-start hook now, before the document preamble runs. l3sys
  // defers `\c_sys_shell_escape_int`, `\sys_if_shell:*` and the date/time ints
  // into `\__sys_everyjob:n { … }` (expl3-code.tex L8197-8217), i.e. into
  // `\g__sys_everyjob_tl`, which `\__kernel_sys_everyjob:` runs at job start.
  // Perl LaTeXML never fires `\everyjob`, so on the dump/short-circuit path (a
  // texmf expl3.sty NEWER than the embedded dump skips `\input expl3-code.tex`)
  // those constants stay undefined — issue #531 secondary: the newer expl3.sty
  // USES `\sys_if_shell:TF` in its support-file/shell-escape check →
  // `Error:undefined:\sys_if_shell:TF` (reporter nasser1, TL2026 dump 2026-01-19
  // vs texmf 2026-03-20). Firing here gives LIVE values every conversion; the
  // `INI_MODE` early return above means it does NOT run during dump-build, so
  // the date/time constants are never frozen into the dump. Guarded on the hook
  // existing (the latex dump / raw base defines it; a `\csname` fires it whether
  // or not `:`/`_` are catcode-letters, since the name is built char-by-char).
  if lookup_meaning(&T_CS!("\\__kernel_sys_everyjob:")).is_some() {
    let _ = raw_tex(r"\csname __kernel_sys_everyjob:\endcsname");
  }
});
