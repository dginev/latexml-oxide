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

/// Perl `FindFile($format._dump, ...)` parity for the latex dump.
/// Mirrors `latex_dump::resolve_dump_path` (defined in
/// `OUT_DIR/latex_dump_loader.rs`). Returns true if `latex.dump.txt`
/// is reachable — used by `LoadFormat('latex')` to decide between the
/// dump branch and the base branch.
fn latex_dump_available() -> bool {
  if let Ok(p) = std::env::var("LATEXML_DUMP_PATH") {
    if std::path::Path::new(&p).is_file() {
      return true;
    }
  }
  if let Ok(dir) = std::env::var("LATEXML_DUMP_DIR") {
    if std::path::Path::new(&dir).join("latex.dump.txt").is_file() {
      return true;
    }
  }
  if let Ok(exe) = std::env::current_exe() {
    if let Some(exe_dir) = exe.parent() {
      if exe_dir.join("../resources/dumps/latex.dump.txt").is_file() {
        return true;
      }
      if exe_dir.join("latex.dump.txt").is_file() {
        return true;
      }
    }
  }
  let dev = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../resources/dumps/latex.dump.txt"
  );
  std::path::Path::new(dev).is_file()
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
  if std::env::var_os("LATEXML_INI_MODE").is_some() {
    return Ok(());
  }

  // Perl `LoadFormat('latex')` strict split:
  //   if dump available: bootstrap → dump → constructs (NO base)
  //   else:              bootstrap → base → constructs (NO dump)
  if std::env::var_os("LATEXML_NODUMP").is_none() && latex_dump_available() {
    if let Err(e) = crate::engine::latex_dump::load_definitions() {
      log::warn!("latex_dump: {}", e);
    }
  } else {
    InnerPool!(latex_base);
  }

  InnerPool!(latex_constructs);

  // Rust-only hotfix overrides — entries needed by the Rust port that
  // are NOT in any Perl `latex_{base,bootstrap,constructs}.pool.ltxml`.
  // Loaded LAST so it can `Let!` against `\@ifpackageloaded` etc. that
  // `latex_constructs` just installed.
  InnerPool!(latex_constructs_rust_only);

  // Retry any PA/MPA let-aliases whose target wasn't defined at
  // dump-load time (they were queued rather than silently dropped).
  // Classic example: `\let\a=\@tabacckludge` — `\@tabacckludge`
  // itself is defined in latex_constructs (which loads after the
  // dump), so the alias has to wait until now.
  let (applied, skipped) = latexml_core::dump_reader::flush_deferred_aliases();
  if applied + skipped > 0 {
    log::info!(
      "[latex_dump] deferred aliases: {} applied, {} skipped",
      applied,
      skipped
    );
  }
});
