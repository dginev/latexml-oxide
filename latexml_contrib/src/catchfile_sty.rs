//! catchfile.sty (H. Oberdiek) — `\CatchFileDef` / `\CatchFileEdef`.
//!
//! catchfile.sty:251-296: `\CatchFileDef\cs{file}{setup}` opens a group,
//! runs `setup` (catcode changes, `\endlinechar`), reads the whole file's
//! tokens under those catcodes into a global scratch macro via `\everyeof`,
//! closes the group and `\let`s `\cs` to it at the outer level — unexpanded
//! for `\CatchFileDef`, `\xdef`-expanded with a trailing `\space` for
//! `\CatchFileEdef` (L251-261). A missing file defines `\cs` empty and
//! errors (L240-245; ar5iv's catchfile.sty.ltxml defines it empty too).
//!
//! The setup argument is what makes the read faithful: codehigh's
//! `\dochighinput` reads a `.sty` with `\catcode`\#=12` so parameter
//! characters survive as text (fontscale-code, cistercian manuals), makron.sty
//! L61 reads `\jobname.runs` for a counter (arXiv 1611.01359), mnras tables
//! (arXiv 2210.08043). Guard:
//! `perfect_kernel_batch54::catchfiledef_reads_under_setup_catcodes_and_edef_expands`.
use latexml_core::{binding::content::find_file, mouth::Mouth};
use latexml_package::prelude::*;

LoadDefinitions!({
  DefMacro!(
    "\\CatchFileDef DefToken {}{}",
    r"\begingroup#3\relax\lx@catchfile@slurp{#2}\endgroup\let#1\CatchFile@gtemp"
  );
  DefMacro!(
    "\\CatchFileEdef DefToken {}{}",
    r"\begingroup#3\relax\lx@catchfile@slurp{#2}\xdef\CatchFile@gtemp{\CatchFile@gtemp\space}\endgroup\let#1\CatchFile@gtemp"
  );
  // Read the file under the CURRENT catcodes (the setup ran in this group)
  // into the global `\CatchFile@gtemp`, as catchfile's `\CatchFile@Do` does.
  DefPrimitive!("\\lx@catchfile@slurp{}", sub[(path)] {
    let path_str = path.to_string();
    let body = match find_file(&path_str, None).and_then(|disk| std::fs::read(&disk).ok()) {
      Some(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
      None => {
        Warn!("missing_file", &path_str, s!("CatchFile: File `{path_str}' not found"));
        String::new()
      },
    };
    let tokens = if body.is_empty() {
      Tokens::new(Vec::new())
    } else {
      Mouth::new(&body, None)?.read_tokens()
    };
    def_macro(
      T_CS!("\\CatchFile@gtemp"),
      None,
      tokens,
      Some(ExpandableOptions { scope: Some(Scope::Global), long: true, ..Default::default() }),
    )?;
    Ok(())
  });
});
