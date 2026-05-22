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
use once_cell::sync::Lazy;
use std::path::Path;

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
  if let Some(p) = DUMP_PATH.as_deref() {
    if Path::new(p).is_file() {
      return true;
    }
  }
  let prefer = crate::dump_paths::detect_ambient_texlive_year();
  if let Some(dir) = DUMP_DIR.as_deref() {
    if crate::dump_paths::resolve_versioned_in_dir(Path::new(dir), "latex", prefer).is_some() {
      return true;
    }
  }
  if let Ok(exe) = std::env::current_exe() {
    if let Some(exe_dir) = exe.parent() {
      let installed = exe_dir.join("../resources/dumps");
      if crate::dump_paths::resolve_versioned_in_dir(&installed, "latex", prefer).is_some() {
        return true;
      }
      if crate::dump_paths::resolve_versioned_in_dir(exe_dir, "latex", prefer).is_some() {
        return true;
      }
    }
  }
  let dev = Path::new(DEV_DUMPS_DIR);
  if dev.is_dir()
    && crate::dump_paths::resolve_versioned_in_dir(dev, "latex", prefer).is_some()
  {
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
    InnerPool!(latex_base);
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
  let (applied, skipped) = latexml_core::dump_reader::flush_deferred_aliases();
  if applied + skipped > 0 {
    Info!(
      "latex_dump", "deferred",
      s!("deferred aliases: {} applied, {} skipped", applied, skipped)
    );
  }
});
