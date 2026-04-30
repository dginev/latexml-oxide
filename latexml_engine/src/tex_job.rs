//! TeX Job
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
use chrono::prelude::*;
use once_cell::sync::Lazy;

// Process-once cached env var (see WISDOM #56 — getenv hot-path race).
static SOURCE_DATE_EPOCH: Lazy<Option<String>> =
  Lazy::new(|| std::env::var("SOURCE_DATE_EPOCH").ok());

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
  let dt: DateTime<Local> = if let Some(epoch_str) = SOURCE_DATE_EPOCH.as_deref() {
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
  // Perl latex_constructs.pool.ltxml:97-129 (`\documentstyle` afterDigest).
  // LaTeX 2.09 compat shim. Three branches mirroring Perl strictly:
  //
  //   1. <class>.sty exists  → input_definitions("article", cls,
  //                            handleoptions=true, options=opts)
  //                          + require_package(class, as_class=true,
  //                            after=\compat@loadpackages)
  //   2. <class>.cls exists  → load_class(class, opts,
  //                            after=\compat@loadpackages)
  //   3. neither             → input_definitions("OmniBus", cls,
  //                            handleoptions=true, options=opts,
  //                            after=\compat@loadpackages)
  //                          + require_package(class, as_class=true)
  //
  // Critical Perl semantics (matching latex_constructs.pool.ltxml:97-129):
  //   * `handleoptions => 1` makes the cls's `\DeclareOption`/`\ProcessOptions`
  //     consume the `<opts>` and route leftovers onto `@unusedoptionlist`.
  //   * `after => \compat@loadpackages` runs *after* the cls finishes its
  //     option-processing — at that point unmet options sit on the unused
  //     list and `\compat@loadpackages` (`latex_constructs.rs:2502`) walks
  //     them, RequirePackage's any that resolve, and triggers OmniBus when
  //     anything went unmet. That is what lets `\documentstyle[paspconf]
  //     {article}` transitively load `aas_macros.sty.ltxml` to define
  //     `\affil` / `\altaffilmark` / `\acknowledgments` etc.
  // The latex_dump unconditionally redefines `\documentstyle` with the
  // kernel-style `\input{latex209.def}\documentclass` form (Perl
  // latex_dump.pool.ltxml entry for `\documentstyle`). In Perl that
  // version is itself overridden by latex_constructs.pool.ltxml's
  // DefConstructor; in our Rust port `latex_constructs.rs` doesn't
  // redefine `\documentstyle`, so without intervention any autoload-of-
  // LaTeX-pool path (e.g. `\newcommand` before `\documentstyle`)
  // replaces our impl with the dump's, breaking the
  // `\compat@loadpackages` after-hook that dispatches `[epsf]` etc.
  // from `@unusedoptionlist`. Witness: hep-th9912229
  // (`\newcommand` before `\documentstyle[12pt,epsf]`).
  //
  // Workaround: register the impl under a stable backup name
  // `\lx@documentstyle@impl` and `\let \documentstyle = \lx@documentstyle@impl`
  // at the end of `\@load@latex@pool` so we restore our impl after
  // every LaTeX pool load.
  DefMacro!("\\lx@documentstyle@impl[]{}", sub[(options_opt, class_tks)] {
    use latexml_core::binding::content::{find_file, find_file_fallback, FindFileOptions, load_class};
    let class = class_tks.to_string();
    let class = class.trim().to_string();

    let pool = if class == "amsppt" { "AmSTeX" } else { "LaTeX" };
    input_definitions(pool, InputDefinitionOptions {
      extension: Some(Cow::Borrowed("pool")),
      ..InputDefinitionOptions::default()
    })?;

    state::assign_value("2.09_COMPATIBILITY", true, Some(Scope::Global));

    // Perl TeX.pool.ltxml:60-65 — when the class triggers the AmSTeX pool
    // (only `amsppt` today), Perl LoadPool's AmSTeX and *re-emits*
    // `\documentstyle{class}` so the AmSTeX-pool-defined `\documentstyle`
    // (amstex.rs L40) takes over. Critically, Perl never loads the LaTeX
    // pool (`latex_constructs.pool.ltxml`) on the amsppt path, so its
    // `Let('\magnification', '\@undefined')` (L56) never fires and plain
    // TeX's `\magnification = \magstep N` keeps working. The Rust
    // implementation previously fell through to the article.cls +
    // RequirePackage chain unconditionally, which loads latex.ltx and
    // re-binds `\magnification` to `\@undefined` — an amsppt-only
    // regression. Mirror Perl: for amsppt, just RequirePackage(amsppt,
    // as_class=true) and return. amstex.rs's `\documentstyle` does the
    // same thing in the same way.
    if pool == "AmSTeX" {
      let _ = require_package(&class, RequireOptions {
        notex: Some(true),
        as_class: true,
        ..RequireOptions::default()
      });
      return Ok(Tokens!());
    }

    // Perl L132-135 `compatDefinitions` — pre-bind LaTeX 2.09 dimensions.
    // Perl helper redefines `\@maxsep` and `\@dblmaxsep`; if these come
    // from another file and are already defined, redef is harmless.
    let zero_dim = Stored::Number(latexml_core::common::number::Number::new(0));
    state::assign_value("\\@maxsep", zero_dim.clone(), Some(Scope::Global));
    state::assign_value("\\@dblmaxsep", zero_dim, Some(Scope::Global));

    // Comma-list to Vec<String>. Whitespace-strip per Perl
    // TrimmedCommaList. Empty entries dropped.
    let opts_vec: Vec<String> = options_opt.as_ref()
      .map(|t| t.to_string())
      .unwrap_or_default()
      .split(',')
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect();

    // Perl uses `notex => !LookupValue('INCLUDE_CLASSES')` which defaults to
    // `notex = true` — FindFile consults the @ltxml_paths binding registry
    // as well as the filesystem.
    let notex = !state::lookup_bool("INCLUDE_CLASSES");
    // Probe `.sty` then `.cls`, AND fall back to version-stripping
    // (`find_file_fallback`) so e.g. `\documentstyle{aipproc2}` resolves
    // to aipproc.sty.ltxml — matching Perl's `FindFile` which consults
    // the binding-name registry plus the `\d+`-suffix fallback. Without
    // this, versioned class names go to OmniBus → aipproc.sty.ltxml is
    // never loaded → `\epsfsize` etc. that aipproc.sty's
    // `RequirePackage('psfig'→'epsfig')` chain provides stay undefined.
    let class_sty_found = find_file(
      &format!("{}.sty", class),
      Some(FindFileOptions { notex, ..Default::default() }),
    ).is_some()
    || find_file_fallback(&class, "sty").is_some();
    let class_cls_found = !class_sty_found && (find_file(
      &format!("{}.cls", class),
      Some(FindFileOptions { notex, ..Default::default() }),
    ).is_some()
    || find_file_fallback(&class, "cls").is_some());

    let after = Tokens!(T_CS!("\\compat@loadpackages"));

    if class_sty_found {
      // Branch 1 — class is actually a `.sty` (e.g. spackap, aipproc,
      // kluwer): load article.cls under it, then RequirePackage(class).
      input_definitions("article", InputDefinitionOptions {
        extension: Some(Cow::Borrowed("cls")),
        options: opts_vec.clone(),
        handleoptions: true,
        noerror: true,
        ..InputDefinitionOptions::default()
      })?;
      require_package(&class, RequireOptions {
        options: opts_vec,
        as_class: true,
        after,
        ..RequireOptions::default()
      })?;
    } else if class_cls_found {
      // Branch 2 — `<class>.cls` exists: load it as the document class.
      load_class(&class, opts_vec, after)?;
    } else {
      // Branch 3 — neither sty nor cls found. Load OmniBus to provide the
      // wide AAS/elsevier/etc. coverage, then attempt the user-named class
      // as a package (will likely no-op via missing_file warn).
      input_definitions("OmniBus", InputDefinitionOptions {
        extension: Some(Cow::Borrowed("cls")),
        options: opts_vec.clone(),
        handleoptions: true,
        noerror: true,
        after,
        ..InputDefinitionOptions::default()
      })?;
      require_package(&class, RequireOptions {
        options: opts_vec,
        as_class: true,
        ..RequireOptions::default()
      })?;
    }

    Ok(Tokens!())
  });

  // Initial alias `\documentstyle = \lx@documentstyle@impl`. The latex_dump
  // may overwrite this on autoload; `\@load@latex@pool` (tex.rs) re-applies
  // the Let after every pool load so our impl wins regardless.
  Let!("\\documentstyle", "\\lx@documentstyle@impl");
});

// The \today macro's implementation lives in base_utilities::today()
// (used by the Today! macro in prelude/setup_binding_language.rs). An
// earlier parallel version here was dead code and has been removed; if
// ever needed, the base_utilities version is the canonical one.
