//! TeX Job
//!
//! Core TeX Implementation for LaTeXML

use chrono::prelude::*;
use once_cell::sync::Lazy;

use crate::prelude::*;

/// Set once `\documentstyle` has applied latex209.def's definitions (`\lx@latex@ix@definitions`): a later class
/// declaration after an OmniBus fallback does not apply them again.
const LATEX_IX_DEFINITIONS_APPLIED: &str = "lx@latex@ix@definitions@applied";

// Process-once cached env var (see WISDOM #56 — getenv hot-path race).
static SOURCE_DATE_EPOCH: Lazy<Option<String>> =
  Lazy::new(|| std::env::var("SOURCE_DATE_EPOCH").ok());

/// `SOURCE_DATE_EPOCH` (Unix seconds), when set and numeric: the reproducible-build clock Perl reads in
/// TeX_Job.pool.ltxml:42.
fn source_date_epoch_env() -> Option<i64> {
  SOURCE_DATE_EPOCH
    .as_deref()
    .and_then(|epoch| epoch.trim().parse::<i64>().ok())
}

/// Sets the job's clock — `\year`, `\month`, `\day`, `\time` (globally) — to the instant `epoch` (Unix seconds) on the
/// UTC clock, as Perl TeX_Job.pool.ltxml:42-46 does for `SOURCE_DATE_EPOCH` (`gmtime`). `\today` and
/// `\pdfcreationdate` read these registers. Returns whether `epoch` is a representable instant (the clock is left
/// untouched when it is not).
pub fn assign_date_registers(epoch: i64) -> bool {
  match DateTime::from_timestamp(epoch, 0) {
    Some(utc) => {
      assign_clock_registers(&utc);
      true
    },
    None => false,
  }
}

fn assign_clock_registers<Tz: TimeZone>(dt: &DateTime<Tz>) {
  AssignValue!("\\day", Number!(dt.day()), Scope::Global);
  AssignValue!("\\month", Number!(dt.month()), Scope::Global);
  AssignValue!("\\year", Number!(dt.year()), Scope::Global);
  AssignValue!(
    "\\time",
    Number!(60 * dt.hour() + dt.minute()),
    Scope::Global
  );
}

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
  // Perl TeX_Job.pool.ltxml:42-46: SOURCE_DATE_EPOCH (reproducible builds) on the UTC clock (`gmtime`), else the
  // local clock (`localtime`).
  if !source_date_epoch_env().is_some_and(assign_date_registers) {
    assign_clock_registers(&Local::now());
  }

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
    if !lookup_bool_sym(pin!("INTERPRETING_DEFINITIONS")) {
      leave_horizontal()?;
      flush();
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
  //   1. <class>.sty exists  → input_definitions("article", cls, handleoptions=true, options=opts)
  //                          + require_package(class, as_class=true, after=\compat@loadpackages)
  //   2. <class>.cls exists  → load_class(class, opts, after=\compat@loadpackages)
  //   3. neither             → input_definitions("OmniBus", cls, handleoptions=true, options=opts,
  //      after=\compat@loadpackages)
  //                          + require_package(class, as_class=true)
  //
  // Critical Perl semantics (matching latex_constructs.pool.ltxml:97-129):
  //   * `handleoptions => 1` makes the cls's `\DeclareOption`/`\ProcessOptions` consume the
  //     `<opts>` and route leftovers onto `@unusedoptionlist`.
  //   * `after => \compat@loadpackages` runs *after* the cls finishes its option-processing — at
  //     that point unmet options sit on the unused list and `\compat@loadpackages`
  //     (`latex_constructs.rs:2502`) walks them, RequirePackage's any that resolve, and triggers
  //     OmniBus when anything went unmet. That is what lets `\documentstyle[paspconf] {article}`
  //     transitively load `aas_macros.sty.ltxml` to define `\affil` / `\altaffilmark` /
  //     `\acknowledgments` etc.
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
    use latexml_core::binding::content::{
      find_file, find_file_fallback_exists, FindFileOptions, load_class,
    };
    // Perl latex_constructs.pool.ltxml:79 — `$class =~ s/\s+//g;`. Strip ALL
    // whitespace (not just leading/trailing as a bare `.trim()` would), so a
    // multi-line `\documentstyle[...]{<newline>article}` — whose post-`{`
    // newline becomes an embedded space token in the Semiverbatim arg — still
    // matches its binding registration. Mirrors the sibling `\documentclass`
    // fix (latex_constructs.rs); a whitespace-bearing name otherwise misses
    // the binding and falls through to OmniBus.
    let class: String = class_tks
      .to_string()
      .chars()
      .filter(|c| !c.is_ascii_whitespace())
      .collect();

    // A second class declaration loads no class: latex.ltx's `\documentclass` has made itself `\@twoclasseserror`
    // (:18617-18618, :18944-18947), which `\documentstyle`'s calls too (:18622-18624), "Two \documentclass or
    // \documentstyle commands". A first declaration no binding and no file answered (the OmniBus fallback, branch 3
    // below; `sect01.rs` `\documentclass`) does not count (main-session ruling 2026-10-19): astro-ph9812142's line 1
    // `#\documentstyle[psfig,epsf]{/disk/3/roud/home/Texstuff/aa}` names a path no tree has (the `#` is no comment:
    // pdflatex's "You can't use macro parameter character #", then the missing class stops it), and its line 2
    // `\documentstyle[psfig,epsf]{aa}` loads the real class (aa's `\thesaurus`, `\offprints`, bibitem tags).
    if lookup_bool(crate::latex_constructs::CLASS_DECLARED) {
      return Ok(TokenizeInternal!(
        r"\@latex@error{Two \noexpand\documentclass or \noexpand\documentstyle commands}{The document may only declare one class.}"));
    }
    assign_value(crate::latex_constructs::CLASS_DECLARED, true, Some(Scope::Global));

    let pool = if class == "amsppt" { "AmSTeX" } else { "LaTeX" };
    input_definitions(pool, InputDefinitionOptions {
      extension: Some(Cow::Borrowed("pool")),
      ..InputDefinitionOptions::default()
    })?;

    assign_value("2.09_COMPATIBILITY", true, Some(Scope::Global));

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

    // latex.ltx:18623 `\documentstyle` inputs latex209.def before the class (`\lx@latex@ix@definitions`), and the
    // class's end runs latex209.def's `\@documentclasshook` (latex.ltx:18721), ahead of `\compat@loadpackages`.
    // Once per document: after a first declaration's OmniBus fallback (above), applying it again would save the
    // settled `\normalsize` as `\original@normalsize` and make the two call each other (astro-ph9812142). The second
    // class's load is again under the 2e-command traps' `\@gobble`, until its own `\@documentclasshook`.
    if lookup_bool(LATEX_IX_DEFINITIONS_APPLIED) {
      digest(TokenizeInternal!(r"\let\@latex@e@error\@gobble"))?;
    } else {
      assign_value(LATEX_IX_DEFINITIONS_APPLIED, true, Some(Scope::Global));
      digest(Tokens!(T_CS!("\\lx@latex@ix@definitions")))?;
    }

    // Perl L132-135 `compatDefinitions` — pre-bind LaTeX 2.09 dimensions.
    // Perl helper redefines `\@maxsep` and `\@dblmaxsep`; if these come
    // from another file and are already defined, redef is harmless.
    let zero_dim = Stored::Number(Number::new(0));
    assign_value("\\@maxsep", zero_dim.clone(), Some(Scope::Global));
    assign_value("\\@dblmaxsep", zero_dim, Some(Scope::Global));

    // Now that LaTeX pool is loaded, undefine `\magnification` so babel
    // detects "running under LaTeX" correctly. The latex_constructs
    // pool-load no longer does this unconditionally; deferring to
    // \documentclass/\documentstyle keeps plain TeX papers (whose first
    // line is often `\magnification=\magstep1`) functional under the
    // cortex_worker's eager LaTeX.pool preload. See sibling change in
    // latex_constructs.rs `\documentclass` after_digest.
    if let Some(undef_meaning) = lookup_meaning(&T_CS!("\\@undefined")) {
      assign_meaning(&T_CS!("\\magnification"), undef_meaning, Some(Scope::Global));
    }

    // Comma-list to Vec<String>. Whitespace-strip per Perl
    // TrimmedCommaList. Empty entries dropped.
    let opts_vec: Vec<String> = options_opt.as_ref()
      .map(|t| t.to_string())
      .unwrap_or_default()
      .split(',')
      .map(|s| s.trim().to_string())
      .filter(|s| !s.is_empty())
      .collect();

    // Perl uses `notex => !LookupValue('INCLUDE_CLASSES')`, but its FindFile prefers a `.ltxml` binding whatever
    // `notex` says (Package.pm:2126-2129; only `noltxml` gates it). These probes ask for bindings, which Rust's
    // `find_file` consults only under `notex` — so they always set it: with raw classes on (INCLUDE_CLASSES set, the
    // arXiv profile since 62k) they missed `mn.cls`'s binding and fell to OmniBus (`\ifoldfss` undefined,
    // `{keywords}` lost; astro-ph0008081, astro-ph0105519). A paper-local raw `.sty` is probed below, only when no
    // binding answers, so a shipped `mn.sty` yields to the `mn` binding (astro-ph0602372).
    let notex = true;
    // Probe `.sty` then `.cls`, AND fall back to version-stripping
    // (`find_file_fallback`) so e.g. `\documentstyle{aipproc2}` resolves
    // to aipproc.sty.ltxml — matching Perl's `FindFile` which consults
    // the binding-name registry plus the `\d+`-suffix fallback. Without
    // this, versioned class names go to OmniBus → aipproc.sty.ltxml is
    // never loaded → `\epsfsize` etc. that aipproc.sty's
    // `RequirePackage('psfig'→'epsfig')` chain provides stay undefined.
    // Check `<class>.sty` via binding registry (notex), version-strip
    // fallback, AND disk-only probe (`forbid_ltxml: true`) so paper-local
    // `<class>.sty` files (e.g. `jinstpub.sty`, with no .cls counterpart)
    // can be loaded as a class. Disk probe is gated on the absence of
    // ANY `<class>.cls` binding (exact OR via version-strip fallback) so
    // that `\documentstyle{mn1}` falls back to `mn.cls.ltxml` (Perl-faithful
    // priority) instead of the local `mn1.sty` whose raw load is suppressed
    // by the default `INCLUDE_STYLES=false` gate. Witness: astro-ph0002213
    // (\documentstyle[epsfig]{mn1} + local mn1.sty + \begin{keywords}/\psfig).
    let class_sty_binding = find_file(
      &format!("{}.sty", class),
      Some(FindFileOptions { notex, ..Default::default() }),
    ).is_some();
    // Pre-flight existence checks must NOT eagerly load the fallback
    // binding (its body would run \LoadClass / \RequirePackage with no
    // option-pass-through, contaminating the actual `\documentstyle`
    // load that happens further below). Fix for the `\psfig` cluster
    // (12 papers) — astro-ph0002213 R=1→0 with this gate change.
    let class_sty_fallback = find_file_fallback_exists(&class, "sty");
    let class_cls_binding_exact = find_file(
      &format!("{}.cls", class),
      Some(FindFileOptions { notex, ..Default::default() }),
    ).is_some();
    let class_cls_via_fallback = find_file_fallback_exists(&class, "cls");
    let class_sty_via_disk = !class_sty_binding
      && !class_sty_fallback
      && !class_cls_binding_exact
      && !class_cls_via_fallback
      && find_file(
        &class,
        Some(FindFileOptions { ext_type: Some(Cow::Borrowed("sty")), forbid_ltxml: true, ..Default::default() }),
      ).is_some();
    // With INCLUDE_CLASSES set (`rawclasses`, the arXiv profile's since 67v, or `localrawclasses`), a raw `<class>.cls`
    // no binding answers is the class (Perl latex_constructs.pool.ltxml:115-118 `FindFile(type=>'cls', notex=>!INCLUDE_CLASSES)`
    // takes the binding first, then the file), searched locally only under `searchpaths`.
    let class_cls_raw = !class_cls_binding_exact
      && !class_cls_via_fallback
      && lookup_bool("INCLUDE_CLASSES")
      && find_file(
        &class,
        Some(FindFileOptions {
          ext_type: Some(Cow::Borrowed("cls")),
          forbid_ltxml: true,
          search_paths_only: lookup_string("INCLUDE_CLASSES") == "searchpaths",
          ..Default::default()
        }),
      ).is_some();
    let class_sty_found = class_sty_binding || class_sty_via_disk || class_sty_fallback;
    let class_cls_found = !class_sty_found && (class_cls_binding_exact
      || class_cls_via_fallback || class_cls_raw);

    // (OXIDIZED_DESIGN #179: the "has chapters" answer is stored where the class's load ends, before the style options)
    crate::latex_constructs::store_chapter_after_latex_ix_class_load(true);
    let after = Tokens!(
      T_CS!("\\@documentclasshook"),
      T_CS!("\\lx@latex@ix@store@chapters"),
      T_CS!("\\compat@loadpackages")
    );

    if class_sty_found {
      // Branch 1 — class is actually a `.sty` (e.g. spackap, aipproc,
      // kluwer): load article.cls under it, then RequirePackage(class).
      // Pin extension to "sty" so the version-stripping fallback (e.g.
      // aipproc2 → aipproc) prefers the binding registry's `.sty` entry
      // instead of also probing for a same-name `.cls` (which has a
      // different DefMacro signature for `\author` etc. that conflicts
      // with the `.sty` body — root cause for nucl-th0010030).
      // Choose article.cls vs OmniBus.cls as the underlying class. When
      // the `.sty` was found via binding registry or version-strip
      // fallback, the user's intent is "load this binding as the
      // document class" — article.cls is the right base. When the
      // `.sty` was found ONLY via paper-local disk-probe (no binding /
      // no fallback), the file is an arbitrary user style that doesn't
      // know about LaTeXML's frontmatter conventions; OmniBus is the
      // right base because it provides broad fallback coverage
      // (`\citeauthoryear` autoload trigger, generic `\affil`, etc.).
      // Mirrors Perl's flow for paper-local-only `\documentstyle`
      // classes (verified via `--verbose` on astro-ph0510540 and
      // astro-ph0008100 — Perl loads OmniBus for both).
      let underlying = if class_sty_via_disk { "OmniBus" } else { "article" };
      input_definitions(underlying, InputDefinitionOptions {
        extension: Some(Cow::Borrowed("cls")),
        options: opts_vec.clone(),
        handleoptions: true,
        noerror: true,
        ..InputDefinitionOptions::default()
      })?;
      // When the .sty was found ONLY via paper-local disk-probe (no
      // binding, no fallback), pass `notex=false` to allow the raw
      // load. Otherwise the default `INCLUDE_STYLES=false` gate inside
      // require_package would force `notex=true` and suppress the
      // load. Mirrors the `\compat@loadpackages` fix from commit
      // bb4cf2e17. Witness: astro-ph0008100
      // (\documentstyle[PASJadd]{PASJ95} + uppercase-named PASJ95.STY).
      let notex_for_require = if class_sty_via_disk { Some(false) } else { None };
      require_package(&class, RequireOptions {
        options: opts_vec,
        extension: Some(Cow::Borrowed("sty")),
        notex: notex_for_require,
        after,
        ..RequireOptions::default()
      })?;
    } else if class_cls_found {
      // Branch 2 — `<class>.cls` exists: load it as the document class.
      load_class(&class, opts_vec, after)?;
      crate::latex_constructs::make_missing_float_counters()?;
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
      // No binding and no file answered: a later `\documentstyle` may still declare the class (above).
      assign_value(crate::latex_constructs::CLASS_DECLARED, false, Some(Scope::Global));
    }

    Ok(Tokens!())
  });

  // What latex209.def (TL 2025 v0.55, read whole) defines that the native `\documentstyle` path lacks and a 2.09
  // class or document uses. The size switch first: `\normalsize` is the class's `\@normalsize` (:138-139), which a
  // 2.09 class defines (`\def\@normalsize{\@setsize\normalsize{12pt}\xpt\@xpt}` then `\normalsize`; witnesses
  // hep-ex0107077 dp_delphi.cls, 0802.1120 klu.cls, astro-ph0009372 cupconf.cls: "The font size command \normalsize is
  // not defined"), and at the class's end `\@documentclasshook` (:124-137) settles the pair: `\normalsize` the
  // class's `\@normalsize`, else `\@normalsize` the class's `\normalsize`, else the kernel's placeholder again.
  // `\@normalsize` is undefined until then, as in latex.ltx (only its own `\@documentclasshook`, :18348, gives it a
  // meaning; latex_constructs/sect13.rs's default stands for that hook), so a class defining neither meets
  // "undefined", not a `\normalsize`/`\@normalsize` loop. Then the 2.09 registers and macros a class reads
  // (:234-236 `\@cla`, `\@clb`, `\mscount`; :213 `\@lquote`; :229-233 `\SLiTeX`; :342 `\load`; :336-338 the old
  // font scaling words; :356-367 the old size commands `\vpt`…`\xxvpt`) and latexsym (:307). The LaTeX2e-command
  // traps come first, as in the file (:79-123): `\usepackage`, `\listfiles`, `\lrbox` and `\@xargdef` call
  // `\@latex@e@error` before their own meaning, a `\@gobble` until the class's end, where `\@documentclasshook`
  // makes it latex209.def's "LaTeX2e command … in LaTeX 2.09 document" error (:137) — so a `\usepackage` after
  // `\documentstyle` errs as in pdflatex and still loads the package (user ruling 2026-10-17; no paper of the 250 `\documentstyle` papers in the census sample
  // uses it — 0806.0904 redefines `\usepackage` itself); and
  // the `\@ifdefinable` wrapper lets a 2.09 document define the 2e-only names it lists (`\th`, `\SS`, the guillemets).
  // The `\ensuremath` trap (:88) is held: our bindings call `\ensuremath` where their 2.09-era originals do not (AASTeX 4
  // aaspp4.sty `\def\fd{\hbox{$.\!\!^{\rm d}$}}` against aas_support_sty.rs `\ensuremath{\@fd}`; REVTeX 3,
  // emulateapj), so the trap would fire on binding code pdflatex never runs (astro-ph9606143, astro-ph9701025,
  // cond-mat9906271, astro-ph9809336, astro-ph9806372) — it lands once binding internals use a kernel-internal
  // ensure-math (SYNC row). OXIDIZED_DESIGN_DIVERGENCES #725.
  // Not applied, the native engine standing for them: the missing-file and obsolete-file fallbacks (:140-168, the
  // `.sty`-as-class branches below), the byte catcodes
  // (:214-228), `\verb`, the font-selection redefinitions (`\rm`…`\em`, `\@setfontsize`, the symbol fonts and math
  // alphabets, `\DeclareOldFontCommand`, :273-335), tracefnt (:273), the marks and the output-routine resets. Perl's
  // `\documentstyle` (latex_constructs.pool.ltxml:97-129) applies none of it (KNOWN_PERL_ERRORS #925). Guard
  // `perfect_kernel_batch70b::documentstyle_class_sets_normalsize_through_at_normalsize`.
  DefMacro!(
    "\\lx@latex@ix@definitions",
    r"\def\@tempa#1#2{\expandafter\let\csname @@\string#1\endcsname#1%
\edef#1{\noexpand\@latex@e@error{\noexpand#2}\expandafter\noexpand\csname @@\string#1\endcsname}}%
\@tempa\usepackage\usepackage\@tempa\listfiles\listfiles
\@tempa\lrbox{\begin{lrbox}}\@tempa\@xargdef{\newcommand{cmd}[args][def]}%
\let\@latex@e@error\@gobble
\def\@latex@e@error@#1{\@latexerr{LaTeX2e command \string#1\space in LaTeX 2.09 document}{This is a LaTeX 2.09
document, but it contains \string#1.^^JIf you want to use the new features of LaTeX2e, your document^^Jshould begin
with \string\documentclass\space rather than \string\documentstyle}}%
\let\@old@ifdefinable\@ifdefinable
\long\def\@ifdefinable#1{\def\@tempa####1#1####2#1####3#1####4\@tempa{\def\@latex@e@commands{####1####2}####3\expandafter
\@firstofone\else\expandafter\@old@ifdefinable\expandafter#1\fi}\expandafter\@tempa\@latex@e@commands
#1\iftrue#1\iffalse#1\@tempa}%
\let\@@ifdefinable\@ifdefinable
\def\@latex@e@commands{\usepackage\listfiles\ensuremath\LaTeXe\lrbox\th\dh\ng\dj\TH\DH\NG\DJ\k\r\SS
\guillemetleft\guillemetright\guillemotleft\guillemotright\guilsinglleft\guilsinglright\quotedblbase\quotesinglbase}%
\let\original@normalsize\normalsize\def\normalsize{\@normalsize}\let\@normalsize\@undefined
\def\@documentclasshook{\def\@tempa{\@normalsize}\ifx\normalsize\@tempa\let\normalsize\@normalsize\fi
\ifx\@normalsize\@undefined\let\@normalsize\normalsize\fi
\ifx\normalsize\@undefined\let\normalsize\original@normalsize\fi\let\@latex@e@error\@latex@e@error@}
\ifx\@cla\@undefined\newcount\@cla\fi\ifx\@clb\@undefined\newcount\@clb\fi\ifx\mscount\@undefined\newcount\mscount\fi
\def\@lquote{\leavevmode{\kern\z@}`}
\DeclareRobustCommand{\SLiTeX}{{\normalfont S\kern -.06em{\scshape l\kern -.035emi}\kern -.06em\TeX}}
\let\load\@gobbletwo
\def\@halfmag{ scaled \magstephalf}\def\@magscale#1{ scaled \magstep#1 }\def\@ptscale#1{ scaled #100 }
\def\vpt{\edef\f@size{\@vpt}\rm}\def\vipt{\edef\f@size{\@vipt}\rm}\def\viipt{\edef\f@size{\@viipt}\rm}
\def\viiipt{\edef\f@size{\@viiipt}\rm}\def\ixpt{\edef\f@size{\@ixpt}\rm}\def\xpt{\edef\f@size{\@xpt}\rm}
\def\xipt{\edef\f@size{\@xipt}\rm}\def\xiipt{\edef\f@size{\@xiipt}\rm}\def\xivpt{\edef\f@size{\@xivpt}\rm}
\def\xviipt{\edef\f@size{\@xviipt}\rm}\def\xxpt{\edef\f@size{\@xxpt}\rm}\def\xxvpt{\edef\f@size{\@xxvpt}\rm}
\RequirePackage{latexsym}"
  );

  // OXIDIZED_DESIGN #179: the 2.09 class's end stores its "has chapters" answer (`store_chapter_after_latex_ix_class_load`).
  DefPrimitive!("\\lx@latex@ix@store@chapters", {
    crate::latex_constructs::store_chapter_after_latex_ix_class_load(false);
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
