//! TeX Job
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
use chrono::prelude::*;

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Job Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // The current Job
  //----------------------------------------------------------------------
  // \jobname          c  is the underlying file name for a job.
  // \time             pi holds the current time in minutes after midnight (0-1439).
  // \day              pi holds the current day of the month (1-31).
  // \month            pi holds the current month of the year (1-12).
  // \year             pi holds the current year (e.g., 2000).
  // \mag              pi holds the magnification ratio times 1000.
  DefMacro!(T_CS!("\\jobname"), None, Tokens!()); // Set to the filename by initialization
  DefRegister!("\\time", Number!(0));
  DefRegister!("\\day", Number!(0));
  DefRegister!("\\month", Number!(0));
  DefRegister!("\\year", Number!(0));
  DefRegister!("\\mag", Number!(1000));

  // TODO: This may mess up Daemon state? Reinit when setting jobname?
  // Respect SOURCE_DATE_EPOCH env var for reproducible builds (like Perl)
  let dt: DateTime<Local> = if let Ok(epoch_str) = std::env::var("SOURCE_DATE_EPOCH") {
    if let Ok(epoch) = epoch_str.trim().parse::<i64>() {
      DateTime::from_timestamp(epoch, 0)
        .map(|utc| utc.with_timezone(&Local))
        .unwrap_or_else(Local::now)
    } else {
      Local::now()
    }
  } else {
    Local::now()
  };
  AssignValue!("\\day", Number!(dt.day()), Scope::Global);
  AssignValue!("\\month", Number!(dt.month()), Scope::Global);
  AssignValue!("\\year", Number!(dt.year()), Scope::Global);
  AssignValue!(
    "\\time",
    Number!(60 * dt.hour() + dt.minute()),
    Scope::Global
  );

  //======================================================================
  // Random Job related things
  //----------------------------------------------------------------------
  // \end              c  terminates the current job.
  // \everyjob         pt holds tokens which are inserted at the start of every job.
  // \deadcycles       iq is the number of times \output was called since the last \shipout.
  // \maxdeadcycles    pi is the maximum allowed value of \deadcycles before an error is generated.
  // Perl: $stomach->leaveHorizontal; $stomach->getGullet->flush;
  DefPrimitive!("\\lx@end@document", {
    // When called during package/definition loading (e.g., expl3's error handler
    // calls \tex_end:D via \msg_fatal), ignore it. Package errors should not
    // terminate the entire document processing.
    if !state::lookup_bool_sym(pin!("INTERPRETING_DEFINITIONS")) {
      leave_horizontal()?;
      gullet::flush();
    }
    // else: silently ignore during definition loading
  });
  Let!("\\end", "\\lx@end@document");
  // Save the TeX primitive \end under \@@end so that expl3's primitive rename
  // (\__kernel_primitive:NN \end \tex_end:D) gets the real primitive, not
  // LaTeX's \end{environment} handler which consumes {} arguments.
  // In Perl, @@end is the saved TeX \end (latex.ltx saves it).
  Let!("\\@@end", "\\lx@end@document");

  DefRegister!("\\everyjob", Tokens!());
  DefRegister!("\\deadcycles", Number!(0));
  DefRegister!("\\maxdeadcycles", Number!(0));

  //======================================================================
  // Dumping
  //----------------------------------------------------------------------
  // \dump             c  outputs a format file in INITEX; otherwise it is equivalent to \end.

  DefMacro!("\\dump", {
    Warn!("unexpected", "dump", "Do not know how to \\dump yet, sorry");
  });

  // TODO: load_dump
  // TODO: load_latex

  //======================================================================
  // LaTeX 2.09 compatibility
  //----------------------------------------------------------------------
  // Perl: TeX.pool.ltxml L60-65
  // \documentstyle[opts]{class} — LaTeX 2.09 command.
  // Perl loads LaTeX.pool then re-queues \documentstyle token.
  // Since our \documentclass already loads LaTeX.pool on first encounter,
  // we can simply redirect \documentstyle → \documentclass.
  // Perl: TeX.pool.ltxml L60-65
  // Reads [options]{class}, loads LaTeX (or AmSTeX) pool, then re-emits
  // \documentclass [options]{class} for the now-loaded LaTeX pool to handle.
  DefMacro!("\\documentstyle[]{}", sub[(options_opt, class_tks)] {
    let class = class_tks.to_string();
    let pool = if class == "amsppt" { "AmSTeX" } else { "LaTeX" };
    input_definitions(pool, InputDefinitionOptions {
      extension: Some(Cow::Borrowed("pool")),
      ..InputDefinitionOptions::default()
    })?;

    state::assign_value("2.09_COMPATIBILITY", true, Some(Scope::Global));

    // In LaTeX 2.09, options are both class options AND packages to load.
    // First load the class, then try to load each option as a package.
    let mut result = Tokens!(T_CS!("\\documentclass"));
    if let Some(ref opts) = options_opt {
      let opts: &Tokens = opts;
      result = Tokens!(result, T_OTHER!("["), opts.clone(), T_OTHER!("]"));
    }
    result = Tokens!(result, T_BEGIN!(), class_tks, T_END!());

    // After class loads, try each option as a package (Perl \compat@loadpackages
    // in latex_constructs.pool.ltxml:137-154).
    //
    // Emit `\RequirePackage{opt}` unconditionally for non-kernel options.
    // `require_package` internally tries: compiled binding dispatcher →
    // external .ltxml → raw .sty (INTERPRETING_DEFINITIONS-gated) →
    // find_file_fallback (strips version suffixes: aaspp4 → aaspp,
    // emulateapj5 → emulateapj) → raw .sty again → kpsewhich. A missing
    // option (e.g. `anonymous`) ends at a note-level log, not an error.
    //
    // This replaced an earlier `\IfFileExists{opt.sty}` wrapper and then a
    // Rust-level `find_file + find_file_fallback` pre-check. Both had the
    // same structural bug: compiled-in Rust bindings (like emulateapj_sty.rs)
    // don't surface on the filesystem, so the pre-check missed them and
    // skipped emitting \RequirePackage — leaving `\documentstyle[emulateapj]
    // {article}` with ~15 undefined macros (astro-ph0009148 class). Perl's
    // FindFile covers compiled-in .ltxml because they're real files in
    // @ltxml_paths; Rust has no equivalent probe so we just let
    // require_package make the decision.
    //
    // Skip-list below is the subset of LaTeX 2.09 class options that are
    // NEVER packages (size, layout, mode). These are consumed by
    // article.cls's option handler directly; emitting \RequirePackage for
    // them would fail spuriously.
    if let Some(opts) = options_opt {
      let opts_str = opts.to_string();
      for opt in opts_str.split(',') {
        let opt = opt.trim();
        if opt.is_empty() { continue; }
        if opt.ends_with("pt") || opt == "landscape" || opt == "twocolumn"
          || opt == "onecolumn" || opt == "draft" || opt == "final"
          || opt == "preprint" || opt == "tighten" || opt == "manuscript" {
          continue;
        }
        result = Tokens!(result,
          T_CS!("\\RequirePackage"), T_BEGIN!(), Explode!(opt), T_END!()
        );
      }
    }
    Ok(result)
  });
});

// The \today macro's implementation lives in base_utilities::today()
// (used by the Today! macro in prelude/setup_binding_language.rs). An
// earlier parallel version here was dead code and has been removed; if
// ever needed, the base_utilities version is the canonical one.
