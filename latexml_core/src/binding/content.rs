use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::borrow::Cow;
use std::collections::VecDeque;
use std::path::Path;
use std::rc::Rc;

use crate::common::arena;
use crate::common::arena::SymStr;
use crate::common::error::*;
use crate::common::font::{Font, Fontmap};
use crate::common::model;
use crate::document::resource::*;
use crate::document::tag::{TagOptionName, TagOptions};
use crate::gullet;
use crate::gullet::do_expand;
use crate::mouth::{Mouth, MouthOptions};
use crate::parameter::{Parameter, Parameters};
use crate::state::let_i;
use crate::state::*;
use crate::stomach::*;
use crate::token::*;
use crate::tokens::Tokens;
use crate::util::pathname::{self, PathnameFindOptions};
// use crate::util::pathname::PathnameFindOptions;
use crate::Digested;

use crate::binding::def::dialect::def_macro;
use crate::definition::expandable::ExpandableOptions;

static QUOTE_WRAPPED: Lazy<Regex> = Lazy::new(|| Regex::new("^\"(.+)\"$").unwrap());

/// Maximum nesting depth for package/class loading to prevent infinite recursion.
/// Perl LaTeXML has no explicit limit but rarely exceeds 20 levels in practice.
const MAX_INPUT_DEPTH: usize = 500;

thread_local! {
  /// Current nesting depth of input_definitions calls.
  static INPUT_DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// a configuration for loading LaTeX definition files (such as .sty, .cls, and their bindings)
pub struct InputDefinitionOptions {
  /// an optional extension (such as "sty")
  pub extension:        Option<Cow<'static, str>>,
  /// package options to pass into the loaded library
  pub options:          Vec<String>,
  /// Tokens to process after the definition is loaded
  pub after:            Tokens,
  /// flag to forbid raw TeX sources
  pub notex:            bool,
  /// flag to forbid errors ?
  pub noerror:          bool,
  /// flag to forbid binding dispatch
  pub noltxml:          bool,
  /// collection of (package) options to process when loading the dependency
  pub withoptions:      Option<Vec<String>>,
  /// flag to handle options, or ignore them
  pub handleoptions:    bool,
  /// flag to process in .cls mode (default: false)
  pub as_class:         bool,
  /// flag to indicate reading the file raw in Gullet
  pub raw:              bool,
  /// flag to allow reloading a previously loaded definitions file
  pub reloadable:       bool,
  /// flag: set @ catcode to LETTER during loading (default true).
  /// Set to false for packages like xy.tex that need @ to stay as OTHER.
  pub at_letter:        bool,
  /// When set, raw-file lookup is restricted to the user-supplied
  /// SEARCHPATHS (SOURCEDIRECTORY + graphicspaths), skipping the
  /// kpsewhich fallback into system texmf. Matches Perl Package.pm's
  /// `searchpaths_only => 1` — enabled by the `localrawstyles` option
  /// to latexml.sty. Perl ref: Package.pm L2135, L2674.
  pub searchpaths_only: bool,
}
impl Default for InputDefinitionOptions {
  fn default() -> Self {
    InputDefinitionOptions {
      extension:        None,
      options:          Vec::new(),
      after:            Tokens!(),
      notex:            false,
      noerror:          false,
      noltxml:          false,
      raw:              false,
      reloadable:       false,
      withoptions:      None,
      handleoptions:    false,
      as_class:         false,
      at_letter:        true,
      searchpaths_only: false,
    }
  }
}

/// TODO: Flesh out with the full infrastructure, incremental functionality for now.
pub fn input_definitions(raw_file: &str, mut options: InputDefinitionOptions) -> Result<()> {
  let name = raw_file.trim();

  // Guard: prevent infinite recursion from circular or runaway package loading.
  // When a binding is missing, raw TeX loading can trigger macro loops.
  let depth = INPUT_DEPTH.with(|d| {
    let current = d.get();
    d.set(current + 1);
    current + 1
  });
  if depth > MAX_INPUT_DEPTH {
    INPUT_DEPTH.with(|d| d.set(d.get() - 1));
    Fatal!(
      Stomach,
      Recursion,
      s!(
        "Package loading depth exceeded {} (loading '{}').\
        This usually means a missing binding causes infinite recursion.",
        MAX_INPUT_DEPTH,
        name
      )
    );
  }

  // Ensure depth cleanup on all exit paths via a guard
  struct InputDepthGuard;
  impl Drop for InputDepthGuard {
    fn drop(&mut self) { INPUT_DEPTH.with(|d| d.set(d.get() - 1)); }
  }
  let _guard = InputDepthGuard;

  // Note: we always need a gullet to expand, and we sometimes need a stomach to load_definitions...
  // so let's make stomach a mandatory option.
  //
  // Snapshot \@currname/\@currext only when handleoptions=true. Perl
  // Package.pm:2549-2550 does the same — both prevname and prevext are
  // gated on options{handleoptions}. The handleoptions=false branch
  // does NOT mutate \@currname/\@currext (mirrors plain LaTeX `\input`
  // semantics; Perl L2580 likewise only mutates them inside its
  // handleoptions=true branch).
  let prevname = if options.handleoptions && lookup_definition(&T_CS!("\\@currname"))?.is_some() {
    gullet::do_expand(T_CS!("\\@currname"))?.to_string()
  } else {
    String::new()
  };
  let prevext = if options.handleoptions && lookup_definition(&T_CS!("\\@currext"))?.is_some() {
    gullet::do_expand(T_CS!("\\@currext"))?.to_string()
  } else {
    String::new()
  };
  // This file will be treated somewhat as if it were a class
  // IF as_class is true
  // OR if it is loaded by such a class, and has withoptions true!!! (yikes)
  if options.handleoptions && options.withoptions.is_some() {
    with_vecdeque("@masquerading@as@class", |vdq_opt| {
      if let Some(vdq) = vdq_opt {
        if vdq.iter().any(|x| {
          if let Stored::String(ref v) = x {
            arena::with(*v, |str| str == prevname)
          } else {
            false
          }
        }) {
          options.as_class = true;
        }
      }
    });
  }
  if options.noltxml {
    options.raw = true; // so it will be read as raw by Gullet.
  }
  let as_type = if options.as_class {
    Cow::Borrowed("cls")
  } else {
    options
      .extension
      .as_ref()
      .cloned()
      .unwrap_or(Cow::Borrowed(""))
  };

  // If loading a class, store class options (Perl Package.pm lines 2561-2564).
  // Perl L2561: `if ($astype eq 'cls' and $options{options})` — only
  // (re)define `\@classoptionslist` when THIS cls load actually carries
  // options. A nested `\LoadClass` with empty options (e.g. amsart →
  // ams_core; our `*_cls.rs` bindings pass `Tokens!()` rather than forwarding
  // the outer options as Perl's `withoptions=>1` does) must NOT clobber the
  // document class's option list — babel iterates `\@classoptionslist`
  // (`\bbl@foreach`, babel.sty L4270) to pick up a GLOBAL language option such
  // as `\documentclass[french]{amsart}` and only then declares/loads that
  // language. Clobbering it to empty silently dropped global babel languages
  // for every bound class. Witness 1911.07001 (`[oneside,french,titlepage]
  // {amsart}` + bare `\usepackage{babel}` → french.ldf must load).
  //
  // DIVERGENCE retained from Perl: still define `\@classoptionslist` as EMPTY
  // for an option-less *document* class so babel's
  // `\csname\ds@\@classoptionslist\endcsname` doesn't run away (the kernel
  // default `\let\@classoptionslist\relax`; witness 2504.00009). We gate that
  // on "no class options recorded yet", so it fires for the first/outermost
  // class load but never clobbers an already-populated list on a nested load.
  if as_type == "cls" {
    for opt in &options.options {
      push_value("class_options", arena::pin(opt))?;
    }
    let class_opts_str = options.options.join(",");
    let have_recorded_opts = lookup_vecdeque("class_options")
      .map(|v| !v.is_empty())
      .unwrap_or(false);
    if !class_opts_str.is_empty() || !have_recorded_opts {
      def_macro(
        T_CS!("\\@classoptionslist"),
        None,
        Tokens!(Explode!(class_opts_str)),
        None,
      )?;
    }
  }

  // Compute the exact name based on the type
  let filename = match &options.extension {
    None => name.to_string(),
    Some(ext) => s!("{}.{}", name, ext),
  };
  // Store the document class filename for xkeyval's isInClassFile check
  if as_type == "cls" && options.handleoptions {
    assign_value(
      "document_class_filename",
      filename.clone(),
      Some(Scope::Global),
    );
  }
  let current_options = options.options.join(",");
  if !current_options.is_empty() {
    if let Some(Stored::String(prevoptions)) = lookup_value(&s!("{filename}_loaded_with_options")) {
      if arena::with(prevoptions, |prev_str| current_options != prev_str) {
        let message = s!(
          "Option clash for file {} with options {:?}, previously loaded with {:?}",
          filename,
          current_options,
          prevoptions
        );
        Info!("unexpected", "options", message);
      }
    }
  }

  // TODO: This needs reorganization, bindings are not found as "files" in rust,
  // we need to have a registry (we don't yet)

  // Perl: early-stop if already loaded (checks request_loaded, name_loaded, etc.)
  // This prevents double-loading and breaks circular loading chains.
  // IMPORTANT: check BEFORE printing "Loading..." message to avoid spurious output.
  //
  // Per OXIDIZED_DESIGN #23: gate on the flag matching the load path
  // we'll actually take. CRITICAL invariant: a binding `<file>.rs` is
  // allowed to call `InputDefinitions(noltxml=>1)` for its same-named
  // raw .sty/.cls/.def AFTER its own `_loaded` flag was set — the raw
  // load gates on `_raw_loaded`, not `_loaded`. Examples: babel_sty
  // → raw babel.sty; cite_sty → raw cite.sty.
  let opt_noltxml = options.noltxml;
  let opt_notex = options.notex;
  // Rust-only `_load_attempted` flag: set in the miss-handler below to
  // prevent retry loops while keeping `_loaded` reserved for genuine
  // binding success. Without this split the `_loaded`-on-miss hack
  // shadowed `require_package`'s `!_loaded && !_raw_loaded`
  // post-call check, disabling `maybe_require_dependencies` for any
  // package that had no binding (e.g. paper-local `jinstpub.sty`).
  let already_handled = |fkey: &str| -> bool {
    if opt_noltxml {
      lookup_bool(&s!("{fkey}_raw_loaded"))
    } else if opt_notex {
      lookup_bool(&s!("{fkey}_loaded")) || lookup_bool(&s!("{fkey}_load_attempted"))
    } else {
      lookup_bool(&s!("{fkey}_loaded"))
        || lookup_bool(&s!("{fkey}_raw_loaded"))
        || lookup_bool(&s!("{fkey}_load_attempted"))
    }
  };
  if !options.reloadable && already_handled(&filename) {
    return Ok(());
  }
  // Also check without extension (Perl checks name_loaded too)
  if !options.reloadable && name != filename && already_handled(name) {
    return Ok(());
  }

  // Mark as loaded, then process the definitions.
  //
  // Suppress the "Loading X definitions" banner when we're inside a
  // nested input_definitions call for the SAME file — that's the
  // pattern where an .ltxml binding (e.g. babel_sty.rs) immediately
  // calls `input_definitions(name, noltxml=true)` to raw-load the
  // texlive .sty. Both frames used to print, producing the confusing
  // duplicate `(Loading "babel.sty" definitions... (Loading
  // "babel.sty" definitions...` trace. Now only the outermost frame
  // announces — tracked per-filename via a state-value marker.
  let banner_key = s!("__loading_banner__{filename}");
  let this_frame_announces = crate::state::with_value(&banner_key, |v| v.is_none());
  if this_frame_announces {
    note_begin(&s!("Loading {:?} definitions", filename));
    crate::state::assign_value(&banner_key, true, Some(crate::state::Scope::Global));
  }

  // Snapshot options.after / options.options BEFORE handleoptions consumes
  // them so the fallback-binding recursive call (Step 3 below) can forward
  // both to the fallback. Without this snapshot, mn1 → mn.cls.ltxml fallback
  // ran with empty options/after and the user's `[epsfig]` was lost (see
  // astro-ph0002213 root cause).
  let original_after = options.after.clone();
  let original_options = options.options.clone();
  // Snapshot the GRANDPARENT's expl3 state BEFORE `\@pushfilename`'s
  // `\ExplSyntaxOff` flips `_` to SUB. The post-load cleanup hook in
  // load_tex_definitions uses this to know whether the calling context
  // was in expl3 mode (so it can skip the `\ExplSyntaxOff` cleanup that
  // would otherwise stick post-`\@popfilename`). Witness cluster:
  // arXiv:2509.05997 / .07893 / .02344, 2510.13206/.13942/.17317
  // (xsavebox + sys_load_backend + l3backend-dvips.def chain — minimal
  // repro: \usepackage{xsavebox}).
  let grandparent_in_expl3 = lookup_catcode('_') == Some(Catcode::LETTER);
  // Strict-LaTeX-kernel order (latex.ltx `\@onefilewithoptions`, L15518-L15519):
  //   \@pushfilename                        % capture OLD \@currname / \@currext
  //   \xdef\@currname{ <new name> }         % then update to NEW
  //
  // We previously set \@currname/\@currext to the new file's name BEFORE
  // calling `before_input_handle_options` (which performs the push). That
  // captured the NEW name in the pushed triple, so `\@currnamestack` never
  // held the empty `{}{}{<catcode>}` initial-state triple that
  // expl3-code.tex's `\__file_tmp:w` recursion uses as its termination
  // sentinel. Result: under raw expl3-code.tex load (LATEXML_NODUMP=1),
  // the recursion ate past `\group_end:` into subsequent
  // `\seq_new:N` / `\cs_new:Npn …` lines, producing the cs_end:
  // cascade documented in .investigation/cs_end_bisect_round22/.
  //
  // Now: push first (uses current/OLD \@currname), then update inside
  // before_input_handle_options (line 756-757). For the
  // `handleoptions == false` path no push happens, so set the names
  // directly here.
  if options.handleoptions {
    before_input_handle_options(&mut options, &prevname, &prevext, name, &as_type)?;
    def_macro(
      T_CS!(s!("\\{}.{}-h@@k", name, as_type)),
      None,
      options.after,
      None,
    )?;
  }
  // No `else` branch: Perl Package.pm L2580-2611 only mutates
  // \@currname/\@currext inside the handleoptions=true block. The
  // handleoptions=false path mirrors plain LaTeX `\input`, which leaves
  // them untouched. Mutating them here breaks \@currnamestack
  // discipline: a subsequent inner \RequirePackage's \@pushfilename
  // captures the leaked name instead of the empty initial-state value,
  // and expl3-code.tex's \__file_tmp:w stack walk over-reads.
  // Witnesses: 0805.4519 (inputenc+ansinew), 1705.00041
  // (\usetikzlibrary{calligraphy}+spath3+expl3).

  if !current_options.is_empty() {
    assign_value(
      &s!("{}_loaded_with_options", filename),
      current_options,
      Some(Scope::Global),
    );
  }

  // Track loaded files in \@filelist BEFORE loading (Perl: Package.pm calls
  // \@addtofilelist before reading the file, so \@filelist is available inside)
  if options.handleoptions && lookup_definition(&T_CS!("\\@addtofilelist"))?.is_some() {
    digest(Tokens!(
      T_CS!("\\@addtofilelist"),
      T_BEGIN!(),
      Explode!(filename),
      T_END!()
    ))?;
  }

  // Skip loading entirely if already loaded (unless reloadable)
  // This prevents double-loading when e.g. smfart calls load_class("amsart")
  // after the binding already set the _loaded flag.
  // Per OXIDIZED_DESIGN #23: gate by the load path's flag — same
  // path-aware logic as the early-skip above. Allows a binding to
  // load its same-named raw counterpart via `noltxml=>1`.
  if !options.reloadable && already_handled(&filename) {
    if this_frame_announces {
      note_end(&s!("Loading {:?} definitions", filename));
      crate::state::assign_value(
        &banner_key,
        crate::common::store::Stored::None,
        Some(crate::state::Scope::Global),
      );
    }
    return Ok(());
  }

  // Catch Fatal errors during binding loading (e.g., token limit exceeded during
  // expl3 kernel loading). Convert to non-fatal so document processing continues.
  let is_binding = if options.noltxml {
    false
  } else {
    match _load_binding(false, &filename, options.reloadable).and_then(|ext| {
      if ext {
        Ok(true)
      } else {
        _load_binding(true, &filename, options.reloadable)
      }
    }) {
      Ok(v) => v,
      Err(e) => {
        Error!(
          "unexpected",
          &filename,
          s!("Error loading binding for '{}': {}", filename, e)
        );
        // Mark as loaded even on error to prevent re-loading via raw path
        assign_value(&s!("{filename}_loaded"), true, Some(Scope::Global));
        false
      },
    }
  };
  let mut is_found_raw = false;
  if is_binding {
    // We found and loaded a binding successfully, mark it as such.
    // Perl Package.pm::loadLTXML L2315-2316 sets TWO flags: `$request`_loaded
    // (e.g. `color.sty_loaded`) AND `$ltxname`_loaded (`color.sty.ltxml_loaded`),
    // where `.ltxml` is the suffix of the Perl binding file. Rust's port
    // keeps only the former — `.ltxml` is not a suffix in the Rust world, so
    // binding-vs-raw-tex distinction is queryable via `*_loaded` directly.
    // See OXIDIZED_DESIGN.md. Callers of the legacy `.ltxml_loaded` form
    // must be migrated to `_loaded`.
    // Per OXIDIZED_DESIGN #23: binding success → `<filename>_loaded`.
    // Raw load tracks separately via `<filename>_raw_loaded` (see
    // load_tex_definitions). The `_found_loaded` Rust-only flag is
    // dropped — read sites check `_loaded || _raw_loaded` instead.
    let loaded_flag = format!("{filename}_loaded");
    assign_value(&loaded_flag, true, Some(Scope::Global));
    // Also set the Perl-equivalent `<filename>.ltxml_loaded` flag —
    // some callers (e.g. require_package's deps-scan gate) need to
    // distinguish binding-loaded from raw-loaded. Without this,
    // paper-bundled .sty files that load natbib (which has a
    // binding) re-trigger deps-scan on natbib.sty, which re-finds
    // \usepackage{natbib} in natbib.sty's own warning text, looping
    // infinitely. Witness 2111.01269 (TIMEOUT from natbib deps loop).
    assign_value(&s!("{filename}.ltxml_loaded"), true, Some(Scope::Global));
    // Perl L2326: Let(T_CS('\ver@'.$trequest), T_CS('\fmtversion'), 'global');
    // Set \ver@name.ext to \fmtversion so LaTeX's \RequirePackage guard works.
    // Without this, \RequirePackage date checks fail and packages get re-loaded.
    if options.handleoptions {
      let ver_cs = T_CS!(s!("\\ver@{}", filename));
      if lookup_definition(&ver_cs).ok().flatten().is_none() {
        let fmtversion_cs = T_CS!("\\fmtversion");
        let_i(&ver_cs, &fmtversion_cs, Some(Scope::Global));
      }
    }
  } else {
    // We're inverting the control flow, because it is near-instant to check whether we have an
    // available binding dispatcher, in both contributed and core binding names
    // Now that we have ensured there is no compiled target of this name, we can start the file
    // system search dance, call to kpsewhich, etc.
    //
    // Perl Package.pm FindFile search order (L2109-2139):
    //   1. .ltxml binding (handled above by load_binding/load_external_binding)
    //   2. Raw TeX in search paths, BUT only if INTERPRETING_DEFINITIONS is true (i.e. we're inside
    //      recursive loading from another raw TeX file)
    //   3. FindFile_fallback — strip version suffixes, find generic .ltxml binding (e.g.
    //      icml2024.sty → icml.sty.ltxml)
    //   4. Raw TeX in search paths (without INTERPRETING_DEFINITIONS gate)
    //   5. kpsewhich
    //
    // This ordering ensures versioned-package fallback bindings take priority
    // over raw .sty files that may contain layout checks (like ICML's \ifdim
    // page-margin checks) that produce spurious warnings.
    let interpreting = lookup_bool_sym(crate::pin!("INTERPRETING_DEFINITIONS"));

    // Step 2: If we're already interpreting raw TeX definitions, look for the file directly.
    // Perl Package.pm L2117-2119: `pathname_find($file, paths => $paths)` —
    // LOCAL PATHS ONLY, no kpsewhich. Rust must mirror this: kpsewhich
    // here would short-circuit Step 3 (fallback ltxml) for any TeX-Live-
    // shipped raw file. Witness: `\RequirePackage{caption3}` from raw
    // floatrow.sty — Perl finds caption3.sty NOT in user paths, falls
    // through to Step 3 → caption.sty.ltxml. Rust with kpsewhich here
    // returned the real caption3.sty from TL, raw-loading it and
    // triggering the `\DeclareCaptionFormat{hang}[#1#2#3\par]{...}`
    // PARAM-leak cascade (arXiv:2506.19291: Rust=30 vs Perl=2).
    let found_raw = if interpreting && !options.notex {
      find_file(
        &filename,
        Some(FindFileOptions {
          forbid_ltxml:      options.noltxml,
          notex:             false,
          ext_type:          options.extension.as_ref().cloned(),
          search_paths_only: true,
        }),
      )
    } else {
      None
    };

    // Step 3: Try fallback (strip version suffixes / dir prefix) before raw TeX.
    // Perl Package.pm L2118-2121: FindFile_fallback.
    //
    // Design policy: bindings ALWAYS win over local raw .sty/.cls files.
    // The `.rs` bindings are hand-tuned for the conversion, so if a
    // fallback name resolves to a registered binding we dispatch there
    // unconditionally. Raw TeX is the last-resort path (Step 4).
    //
    // Two flavors are recorded via [`FallbackKind`] for informational
    // log messages only — both always fire when the binding exists:
    //   - Versioned: suffix/prefix actually stripped (Perl-faithful). Drivers: 1206.0536 (mysvjour3
    //     → svjour3), astro-ph0005021 (./aaspp4 → ./aaspp — aaspp4.sty ships locally with plain-TeX
    //     `\startdata`; the engine's alignment-aware binding still wins, matching Perl).
    //   - BasenameOnly: only directory prefix removed. Rust-specific extension keyed to our
    //     contrib-binding registry. Drivers: 2105.02087 (misc/ieeetran → IEEEtran binding);
    //     2405.18387 (assets/equations → equations binding, because we ship a tuned binding for
    //     this name).
    let found_raw = if found_raw.is_some() {
      found_raw
    } else if !options.noltxml {
      if let Some((fallback, _kind)) = find_file_fallback(name, &as_type) {
        Info!(
          "fallback",
          name,
          s!("Interpreted as versioned package, falling back to {fallback}")
        );
        // Load the fallback binding — use reloadable since we already marked original as "loaded"
        let ext_suffix = if as_type == "sty" { ".sty" } else { ".cls" };
        let fallback_name = fallback.trim_end_matches(ext_suffix).to_string();
        // Forward the original options + after-hook so fallback bindings see
        // user-supplied class/package options (Perl-faithful: in Perl FindFile
        // returns a path and the caller's options/after stay attached to the
        // ORIGINAL `\@currname`-frame). Without this, `\documentstyle[epsfig]{mn1}`
        // fell back to mn.cls.ltxml with empty options → mn.cls's option-handler
        // never saw `epsfig` → `\compat@loadpackages` after-hook never fired
        // → `\psfig` undefined. Witness: astro-ph0002213.
        let fb_result = input_definitions(&fallback_name, InputDefinitionOptions {
          extension: Some(Cow::Borrowed(if as_type == "sty" { "sty" } else { "cls" })),
          options: original_options,
          after: original_after,
          handleoptions: options.handleoptions,
          noerror: true,
          reloadable: true,
          ..InputDefinitionOptions::default()
        });
        if fb_result.is_ok() {
          assign_value(&s!("{filename}_loaded"), true, Some(Scope::Global));
          // NOTE: do NOT set `{filename}.ltxml_loaded` here. The
          // fallback name is a DIFFERENT binding (e.g. article for
          // myclass); the original `{filename}` (myclass.cls) has
          // no binding. Setting it would suppress the downstream
          // deps-scan check that picks up myclass's
          // \RequirePackage{caption}.
        }
        None // fallback handled the loading; no raw file to load
      } else {
        None
      }
    } else {
      None
    };

    // Step 4: Raw TeX in search paths (without INTERPRETING_DEFINITIONS gate)
    // Perl Package.pm L2122-2125
    //
    // Per OXIDIZED_DESIGN #23: gate by `_raw_loaded` only — when a binding
    // explicitly loads its raw counterpart via `noltxml=>1`, the binding's
    // own `_loaded` flag is already set, but we MUST still proceed.
    //
    // EXCEPTION: if Step 3 (fallback ltxml binding) just succeeded, Perl's
    // `if/elsif` flow (Package.pm:2118-2125) RETURNS on success and skips
    // the raw-tex branch entirely. Rust's port uses sequential `let`
    // bindings, so we must explicitly check `_loaded` here. Without this
    // gate, `\RequirePackage{caption2}` loads `caption.sty.ltxml` via
    // `find_file_fallback` (caption2 → caption strips trailing digit) AND
    // then ALSO loads raw `caption2.sty`, which fires its
    // `\@ifpackageloaded{caption}` mutual-exclusivity error. Same pattern
    // applies to any package whose name ends in `[vV]?[-_.\d]+` and whose
    // unsuffixed form has its own .ltxml binding.
    let found_raw = if found_raw.is_some() {
      found_raw
    } else if lookup_bool(&s!("{filename}_loaded")) {
      // Fallback ltxml binding already loaded — don't double-load the raw.
      None
    } else if !options.notex && (options.reloadable || !lookup_bool(&s!("{filename}_raw_loaded"))) {
      // Perl Package.pm L2121-2125 + L2131-2136: combined raw-search
      // step. Tries local paths first, then kpsewhich. Mirrors Perl's
      // Step 4 (`!interpreting` local raw) PLUS Step 5 (kpsewhich
      // unconditionally — note Perl's kpsewhich block lacks the
      // interpreting gate). The previous `!interpreting` guard here
      // was wrong: Step 2 now uses `search_paths_only=true`, so
      // under interpreting=true we still need kpsewhich for raw
      // files that have no fallback ltxml binding.
      find_file(
        &filename,
        Some(FindFileOptions {
          forbid_ltxml:      options.noltxml,
          notex:             false,
          ext_type:          options.extension.as_ref().cloned(),
          search_paths_only: options.searchpaths_only,
        }),
      )
    } else {
      None
    };

    if let Some(file) = found_raw {
      is_found_raw = true;
      // The raw load itself sets `<filename>_raw_loaded` via
      // load_tex_definitions (per OXIDIZED_DESIGN #23). Read sites
      // check `_loaded || _raw_loaded` to detect "any load happened".
      load_tex_definitions(
        &filename,
        &file,
        options.reloadable,
        options.at_letter,
        grandparent_in_expl3,
      )?;
    } else if !lookup_bool(&s!("{filename}_loaded")) && !lookup_bool(&s!("{filename}_raw_loaded")) {
      if options.noerror {
        // With noerror: don't mark as loaded and return Err so callers can
        // try fallback names (e.g. tikzlibrary → pgflibrary). Matches Perl's
        // InputDefinitions which returns undef on not-found even with noerror=>1.
        if this_frame_announces {
          note_end(&s!("Loading {:?} definitions", filename));
          crate::state::assign_value(
            &banner_key,
            crate::common::store::Stored::None,
            Some(crate::state::Scope::Global),
          );
        }
        return Err(s!("File not found: {}", filename).into());
      }
      // Perl Package.pm L2679 / L2715: maybeRequireDependencies($name, $type)
      // is invoked when InputDefinitions returned undef ($success false).
      // We mirror that here in the miss-handler, which is the only point
      // where we know neither binding nor raw load occurred. Doing the
      // dependency-scan BEFORE marking `_load_attempted` keeps the call
      // exactly once-per-package and lets paper-local `.sty` files
      // (e.g. jinstpub.sty bundling natbib + amsmath dependencies) wire
      // up their transitively-bound prerequisites even when raw .sty
      // loading is disabled (`INCLUDE_STYLES=false`, the default).
      let scan_type =
        options
          .extension
          .as_deref()
          .unwrap_or(if options.as_class { "cls" } else { "sty" });
      maybe_require_dependencies(name, scan_type);
      // Rust-only retry guard: prevents re-attempting a missing file in
      // a loop (raw TeX repeatedly calling \RequirePackage). Use a
      // dedicated `_load_attempted` flag — NOT `_loaded` — so the
      // post-input_definitions success check in `require_package`
      // remains honest about whether anything actually loaded.
      assign_value(&s!("{filename}_load_attempted"), true, Some(Scope::Global));
      Warn!(
        "missing_file",
        name,
        s!(
          "Can't find binding or file for '{filename}'. \
          No dispatcher entry and no raw file found on disk."
        )
      );
    }
  }

  if options.handleoptions {
    if is_binding || is_found_raw {
      digest(T_CS!(s!("\\{name}.{as_type}-h@@k")))?;
    }
    // Always restore @currname/@currext and pop filename stack,
    // even when no binding was found, to keep the stack balanced.
    // Note: @popfilename uses \gdef to restore @currname/@currext from the stack,
    // so it takes precedence. We also set them with def_macro as a fallback
    // (matches Perl Package.pm lines 2635-2637).
    if !prevname.is_empty() {
      def_macro(
        T_CS!("\\@currname"),
        None,
        Tokens!(ExplodeText!(prevname)),
        None,
      )?;
    }
    if !prevext.is_empty() {
      def_macro(
        T_CS!("\\@currext"),
        None,
        Tokens!(ExplodeText!(prevext)),
        None,
      )?;
    }
    // Perl-faithful: Package.pm:2637 —
    //   Digest(($pushpop ? T_CS('\@popfilename') : T_CS('\lx@popfilename')));
    // Pair with the dispatched push above. Using `\@popfilename` (dump's
    // expl3-wrapped) when both push/pop are defined; else `\lx@popfilename`
    // (LaTeXML safe internal). The push site re-checks `\@pushfilename` and
    // `\@popfilename` definedness independently (state may have changed
    // mid-load); here we re-check too rather than threading a flag.
    let pop_use_expl = lookup_definition(&T_CS!("\\@pushfilename"))?.is_some()
      && lookup_definition(&T_CS!("\\@popfilename"))?.is_some();
    if pop_use_expl {
      digest(T_CS!("\\@popfilename"))?;
    } else {
      digest(T_CS!("\\lx@popfilename"))?;
    }
    // Verify @currname was correctly restored, and force-fix if not
    let restored_name = if lookup_definition(&T_CS!("\\@currname"))?.is_some() {
      do_expand(T_CS!("\\@currname"))?.to_string()
    } else {
      String::new()
    };
    if !prevname.is_empty() && restored_name != prevname {
      // @popfilename may have popped a stale entry; force correct value
      def_macro(
        T_CS!("\\@currname"),
        None,
        Tokens!(ExplodeText!(prevname)),
        Some(ExpandableOptions {
          scope: Some(Scope::Global),
          ..ExpandableOptions::default()
        }),
      )?;
    }
    if !prevext.is_empty() {
      let restored_ext = if lookup_definition(&T_CS!("\\@currext"))?.is_some() {
        do_expand(T_CS!("\\@currext"))?.to_string()
      } else {
        String::new()
      };
      if restored_ext != prevext {
        def_macro(
          T_CS!("\\@currext"),
          None,
          Tokens!(ExplodeText!(prevext)),
          Some(ExpandableOptions {
            scope: Some(Scope::Global),
            ..ExpandableOptions::default()
          }),
        )?;
      }
    }
    reset_options()?;
  }
  // No handleoptions=false cleanup needed: we never mutated
  // \@currname/\@currext on that path (matching Perl).
  if this_frame_announces {
    note_end(&s!("Loading {:?} definitions", filename));
    crate::state::assign_value(
      &banner_key,
      crate::common::store::Stored::None,
      Some(crate::state::Scope::Global),
    );
  }
  Ok(())
}

/// loads a binding from the main binding dispatcher, if available+found
pub fn load_binding(file: &str) -> Result<bool> { _load_binding(true, file, false) }
/// loads a binding from an external binding dispatcher, if available+found
pub fn load_external_binding(file: &str) -> Result<bool> { _load_binding(false, file, false) }
// in the spirit of Perl's Package::loadLTXML
fn _load_binding(internal: bool, request: &str, reloadable: bool) -> Result<bool> {
  // Perl loadLTXML L2311-2313: skip if already loaded, unless reloadable
  // (e.g. `\inputencoding{cp1251}` re-invokes cp1251.def to re-register
  // DeclareInputText mappings after `set_input_encoding` reset them).
  // OXIDIZED_DESIGN #23: binding load gates ONLY on the binding-specific
  // `_loaded` flag (set on success below). A prior raw load
  // (`_raw_loaded`) does NOT preclude the binding from loading — they
  // are independent paths. Mirrors Perl `loadLTXML` (Package.pm L2311).
  let loaded_key = s!("{request}_loaded");
  if !reloadable && lookup_bool(&loaded_key) {
    return Ok(true);
  }

  // Re-entrance guard for binding loads: track which bindings are
  // currently mid-load on this thread, so that if a binding body
  // transitively calls require_package(SAME_NAME) (e.g. via
  // \citet → OmniBus closure → require_package(natbib) firing
  // during natbib's own ProcessOptions chain), we short-circuit
  // instead of re-entering the dispatcher and looping. Task #260.
  thread_local! {
    static IN_PROGRESS: std::cell::RefCell<rustc_hash::FxHashSet<String>> =
      std::cell::RefCell::new(rustc_hash::FxHashSet::default());
  }
  let request_key = request.to_string();
  let already_loading = IN_PROGRESS.with(|s| s.borrow().contains(&request_key));
  if already_loading {
    Warn!(
      "recursion",
      &request_key,
      s!(
        "Binding-load re-entrance for '{}' (transitive require_package \
         during its own LoadDefinitions). Short-circuiting to break the \
         loop; binding's pending side-effects (DefMacro, Let, etc.) \
         will still complete in the outer frame.",
        request_key
      )
    );
    return Ok(true);
  }

  let taken_dispatcher = if internal {
    get_bindings_dispatch()
  } else {
    get_extra_bindings_dispatch()
  };
  match taken_dispatcher {
    Some(ref dispatcher) => {
      // Perl `Package.pm:loadLTXML L2318` wraps the binding-load body in
      // `local $UNLOCKED = 1`, allowing bindings to override prior
      // (locked) definitions. The guard auto-pops on drop.
      let _unlock_guard = crate::common::local_assignments::local_state_unlocked_guard(true);
      // Mark in-progress for the duration of this dispatcher call.
      IN_PROGRESS.with(|s| {
        s.borrow_mut().insert(request_key.clone());
      });
      struct InProgressGuard(String);
      impl Drop for InProgressGuard {
        fn drop(&mut self) {
          let key = self.0.clone();
          IN_PROGRESS.with(|s| {
            s.borrow_mut().remove(&key);
          });
        }
      }
      let _in_progress_guard = InProgressGuard(request_key.clone());
      let result_opt = dispatcher(request);
      match result_opt {
        Some(result) => {
          // Here and only here we are certain we have binding support.
          // Preemptively mark as loaded to avoid recursion.

          // Mark binding as loaded (raw `<request>_raw_loaded` is tracked
          // separately by load_tex_definitions). Per OXIDIZED_DESIGN #23.
          assign_value(&loaded_key, true, Some(Scope::Global));
          match result {
            Ok(()) => Ok(true),
            Err(e) => Err(e),
          }
        },
        None => Ok(false),
      }
    },
    None => Ok(false),
  }
}

// Factor out handling and passing loading options from input_content,
// to simplify main routine
fn before_input_handle_options(
  options: &mut InputDefinitionOptions,
  prevname: &str,
  prevext: &str,
  name: &str,
  as_type: &str,
) -> Result<()> {
  // Perl-faithful translation of Package.pm:2578-2591:
  //
  //   my $pushpop = LookupDefinition(T_CS('\@pushfilename'))
  //              && LookupDefinition(T_CS('\@popfilename'));
  //   if ($pushpop) {
  //     Digest(Tokens(T_CS('\@pushfilename'),
  //         T_BEGIN, T_END, T_BEGIN, T_END, T_BEGIN, Explode($name), T_END));
  //   } else {
  //     Digest(T_CS('\lx@pushfilename'));
  //   }
  //
  // The 3 trailing brace-arg pairs `{}{}{name}` feed
  // `\@expl@push@filename@aux@@` (which the dump's `\@pushfilename`
  // body chains into) — that aux takes 3 args. Without them it reads
  // 3 garbage tokens from the input stream, corrupting the
  // `\g__hook_name_stack_seq` push. Subsequent `\@popfilename`
  // then sees an empty/corrupt seq, fires `\msg_error:nn{hooks}{extra-pop-label}`,
  // whose `\use:e` (=`\edef`) chain expands `\q_no_value` and triggers
  // recursion-detect. See docs/sandbox_failures_SYNC_STATUS.md
  // "\q_no_value cascade" for the full investigation.
  let push_defined = lookup_definition(&T_CS!("\\@pushfilename"))?.is_some();
  let pop_defined = lookup_definition(&T_CS!("\\@popfilename"))?.is_some();
  if push_defined && pop_defined {
    let mut pushtoks = vec![
      T_CS!("\\@pushfilename"),
      T_BEGIN!(),
      T_END!(),
      T_BEGIN!(),
      T_END!(),
      T_BEGIN!(),
    ];
    pushtoks.extend(Explode!(name));
    pushtoks.push(T_END!());
    digest(Tokens::new(pushtoks))?;
  } else {
    digest(T_CS!("\\lx@pushfilename"))?;
  }

  // For \RequirePackageWithOptions, pass the options from the outer class/style to the inner one.
  if let Some(with_options_to_pass) = options.withoptions.take() {
    if !prevname.is_empty() && has_value(&s!("opt@{}.{}", prevname, prevext)) {
      // Only pass those class options that are declared by the package!
      let mut topass = Vec::new();
      with_vecdeque("@declaredoptions", |vdq_opt| {
        if let Some(declared_options) = vdq_opt {
          for op in with_options_to_pass.into_iter() {
            if declared_options.iter().any(|x| {
              if let Stored::String(val) = x {
                arena::with(*val, |str| str == op)
              } else {
                false
              }
            }) {
              topass.push(op)
            }
          }
        }
      });
      if !topass.is_empty() {
        pass_options(name, as_type, topass)?;
      }
    }
  }
  // Use letter-catcode (`ExplodeText`) for `\@currext` / `\@currname` so
  // they match `\@pkgextension`-style build-time-tokenized macros under
  // `\ifx`. Without this the catcodes diverge — `\@pkgextension` from a
  // compile-time `DefMacro!("\\@pkgextension", "sty")` tokenizes "sty"
  // as letters (default LaTeX catcode 11), but the previous `Explode!`
  // used here produces OTHER catcode tokens, so kvoptions's
  // `\ifx\@currext\@pkgextension` always returned false — vendor
  // `\PackageError{kvoptions}{\ProcessLocalKeyvalOptions is intended
  // for packages only}` then fired on every package that uses kvoptions
  // (rerunfilecheck reaches this via the hyperref backend `.def` chain).
  // Witnesses: arXiv:cond-mat/9611206, math/9904040, math/9904041.
  def_macro(
    T_CS!("\\@currname"),
    None,
    Tokens!(ExplodeText!(name)),
    None,
  )?;
  def_macro(
    T_CS!("\\@currext"),
    None,
    Tokens!(ExplodeText!(as_type)),
    None,
  )?;
  // reset options (Note reset & pass were in opposite order in LoadClass ????)
  reset_options()?;
  pass_options(name, as_type, options.options.clone())?;

  // Note which packages are pretending to be classes.
  if options.as_class {
    push_value("@masquerading@as@class", arena::pin(name))?;
  }
  let current_opt_val = with_vecdeque(&s!("opt@{}.{}", name, as_type), |vdq_opt| match vdq_opt {
    Some(vdq) => {
      let mut pieces = String::new();
      for x in vdq.iter() {
        if let Stored::String(val) = x {
          arena::with(*val, |str| pieces.push_str(str));
        }
        pieces.push(',');
      }
      pieces.pop();
      pieces
    },
    None => String::new(),
  });
  def_macro(
    T_CS!(s!("\\opt@{}.{}", name, as_type)),
    None,
    Tokens!(Explode!(current_opt_val)),
    None,
  )?;
  Ok(())
}

/// configuration for input of a TeX source (content files mostly)
#[derive(Debug, Default, Clone)]
pub struct InputOptions {
  pub noerror:    bool,
  pub reloadable: bool,
  pub file_type:  Option<String>,
}

/// Input for cases when the file (or data)
/// is plain TeX material that is expected to contribute content
/// to the document (as opposed to pure definitions).
///
/// A Mouth is opened onto the file, and subsequent reading
/// and/or digestion will pull Tokens from that Mouth until it is
/// exhausted, or closed.
///
/// In some circumstances it may be useful to provide a string containing
/// the TeX material explicitly, rather than referencing a file.
/// In this case, the `literal` pseudo-protocal may be used.
pub fn input_content(request: &str, options: InputOptions) -> Result<()> {
  let filepath = find_file(request, None);
  match filepath {
    // TODO: type => $options{type}, noltxml => 1
    Some(path) => load_tex_content(&path, options),
    None => {
      // Perl Package.pm L2227-2233: `if (FindFile(...)) { loadTeXContent(...); }
      // elsif (!$options{noerror}) { Error('missing_file', $request, ..., ...); }`
      // Recoverable Error, NOT Fatal. Pre-fix, the Rust port emitted a
      // `fatal!(Package, MissingFile)` that terminated the conversion on any
      // missing-but-non-critical input — over-fatal-izing relative to Perl.
      if !options.noerror {
        Error!(
          "missing_file",
          request,
          format!("Can't find TeX file {request}")
        );
      }
      Ok(())
    },
  }
}

/// This is essentially the `\input` equivalent
///
/// we are most likely expecting to get actual content,
/// (possibly with definitions included, as well)
/// but might actually be getting pure definitions,
/// (like a proper style file)
/// in which case we may really want to load a binding.
/// Note that generic style files (non-latex) often have a .tex extension.
pub fn input(request: &str, options: InputOptions) -> Result<()> {
  // unwrap if in quotes \input{"file name"} — Perl parity:
  // `$request =~ s/^("+)(.+)\g1$/$2/;` (single-pass strip of a matching
  // leading+trailing run of quotes). The previous `while` loop checked
  // the unchanged `request`, which spun forever on any quoted input
  // since the replacement only touches `clean_req`.
  let clean_req = QUOTE_WRAPPED.replace(request, "$1");
  // HEURISTIC! First check if equivalent style file, but only under very specific circumstances
  // if pathname_is_literaldata(request) {
  //   let (dir, name, ftype) = pathname_split(request);
  //   let file = name;
  //   if !ftype.is_empty() {
  //     file += format!(".{}",ftype);
  //   }
  //   let path;
  //   // Firstly, check if we are going to OVERRIDE the requested raw .tex file
  //   // with a latexml binding to a style file.
  //   if ((dir.is_empty() && (ftype.is_empty() || (ftype == "tex"))  // No SPECIFIC directory, but
  // a raw tex file.       // AND, in preamble; SHOULD be style file, OR also if we can't find the
  // raw file.     && (LookupValue!("inPreamble") || !FindFile(file))
  //     && (path = FindFile(name, type => 'sty', notex => 1))) { // AND there IS such a style file
  //     Info!("ignore", request, stomach.get_gullet(),
  //       s!("Ignoring input of tex {}, using package {} instead", request, name));
  //     RequirePackage!(name); // Then override, assuming we'll find name as a package file!
  //     return;
  //   }
  // }
  // // Next special case: If we were currently reading a "known" style or binding file,
  // // then this file, even if .tex, must also be definitions rather than content.!!(?)
  // Check for *.latexml source-level bindings first — these are always handled
  // as definitions regardless of INTERPRETING_DEFINITIONS state.
  // Mirrors Perl's automatic .latexml file loading mechanism.
  if clean_req.ends_with(".latexml") {
    return input_definitions(&clean_req, InputDefinitionOptions::default());
  }
  if lookup_bool_sym(crate::pin!("INTERPRETING_DEFINITIONS")) {
    // Split a binding extension off the request so input_definitions sees
    // (name, extension) — matches Perl Package.pm `FindFile` / `Input`
    // semantics. Without the split, `find_file_fallback` runs with
    // `ext_type=""` and reconstructs `"<base>."` (no extension), which
    // never matches a registered binding. Witness: hep-ph9911514 — the
    // raw-loaded `elsartwb.sty` issues `\input elsart12\@ptsize.sty` →
    // `\input{elsart12.sty}`; the version-strip fallback (elsart12 →
    // elsart) needs `ext_type="sty"` to reconstruct `"elsart.sty"` for
    // the binding lookup. Perl recovers `\ack` cleanly via this path; the
    // earlier Rust port dropped the extension and the fallback never
    // resolved.
    let has_dir = clean_req.contains('/') || clean_req.contains('\\');
    if !has_dir {
      if let Some((stem, ext)) = clean_req.rsplit_once('.') {
        if crate::state::is_binding_extension(ext) {
          return input_definitions(stem, InputDefinitionOptions {
            extension: Some(Cow::Owned(ext.to_string())),
            ..InputDefinitionOptions::default()
          });
        }
      }
    }
    return input_definitions(&clean_req, InputDefinitionOptions::default());
  }
  // Perl Package.pm L2109-2113: FindFile_aux checks for `"$file.ltxml"` in
  // $ltxml_paths BEFORE consulting raw TeX paths. In Rust the bindings are
  // compile-time dispatch tables rather than on-disk .ltxml files, so the
  // equivalent check is: if a binding dispatcher responds to `<name>.tex`,
  // load it (matching `\input harvmac` → `harvmac.tex.ltxml` preference
  // over a local `harvmac.tex`). Skip when the request carries a directory
  // (explicit local path).
  let binding_loaded = {
    let has_dir = clean_req.contains('/') || clean_req.contains('\\');
    // Perl Package.pm:2109-2113 + 2255-2270: when `\input{name}` or
    // `\input{name.<ext>}` resolves to a known binding extension AND a
    // binding for `(name, ext)` is reachable, route to the binding
    // instead of the on-disk raw file. Without this, papers using
    // literal `\input{psfig.sty}` (common 1996-2005 idiom) fail because
    // TL2025 dropped the on-disk file even though Rust has the binding.
    //
    // Extensions handled dynamically via `is_binding_extension`: any
    // extension registered by `latexml_package` or `latexml_contrib`
    // (cls / sty / def / fontmap / ldf / ltx / lua / pool / tex /
    // code.tex / ...) is admitted, gating out `\input{foo.eps}`-style
    // content paths.
    //
    // For .tex / no-extension paths we still use `load_binding` (exact
    // dispatch lookup on `<name>.tex`) — a `<name>.tex` request is
    // semantically "include this content", so suffix-stripping fallback
    // (e.g. `mysetup.tex` → `setup.tex.ltxml`) would surprise more than
    // it helps.
    //
    // For .sty / .cls / .def / etc — the binding-extension cases — we
    // route through `input_definitions`, which gives us the full Step
    // 1 → Step 3 → Step 4 ladder including `find_file_fallback`'s
    // version-suffix strip. This is what makes `\input{psfig.sty}`
    // pick up `psfig_sty.rs` AND `\input{caption2.sty}` fall back to
    // `caption_sty.rs` exactly as Perl Package.pm:2266 does via
    // `RequirePackage($name)`.
    if !has_dir {
      let ext = clean_req.rsplit('.').next().unwrap_or("");
      let no_ext = ext == clean_req.as_ref();
      if no_ext || ext == "tex" {
        let tex_name = if ext == "tex" {
          clean_req.to_string()
        } else {
          s!("{}.tex", clean_req)
        };
        load_binding(&tex_name)? || load_external_binding(&tex_name)?
      } else if crate::state::is_binding_extension(ext) {
        // Route through input_definitions for fallback-aware dispatch.
        // The `name` arg expects no extension, so split it off.
        let name = clean_req
          .strip_suffix(&format!(".{}", ext))
          .unwrap_or(&clean_req)
          .to_string();
        let result = input_definitions(&name, InputDefinitionOptions {
          extension: Some(Cow::Owned(ext.to_string())),
          noerror: true,
          reloadable: true,
          ..InputDefinitionOptions::default()
        });
        // input_definitions returns Err on not-found with noerror=true;
        // treat that as "binding not loaded, fall through to raw".
        result.is_ok()
      } else {
        false
      }
    } else {
      false
    }
  };
  if binding_loaded {
    Ok(())
  } else if let Some(path) = find_file(&clean_req, None) {
    // Found something plausible..
    // let ftype = if pathname_is_literaldata(path) { "tex" } else {
    //   pathname_type(path)
    // };

    //   // Should we be doing anything about options in the next 2 cases?..... I kinda think not,
    // but?   if (ftype == "rs") {                  // it's a LaTeXML binding.
    //     load_latexml(request, path);
    //   }
    //   // Else some sort of "known" definitions type file, but not simply 'tex'
    //   else if (ftype != "tex") && (pathname_is_raw(path)) {
    //     load_tex_definitions(request, path);
    //   } else {
    load_tex_content(&path, options)
  //   }
  } else {
    // Perl heuristic: if the file has no directory, and is a .tex or no extension,
    // try loading it as definitions (which checks for binding dispatchers).
    // This handles cases like \input tcilatex where tcilatex.tex.ltxml exists.
    let has_dir = clean_req.contains('/') || clean_req.contains('\\');
    let ext = clean_req.rsplit('.').next().unwrap_or("");
    let is_tex_like = ext == clean_req.as_ref() || ext == "tex"; // no extension or .tex
    if !has_dir && is_tex_like {
      // Try loading as a .tex binding (e.g. tcilatex → tcilatex.tex)
      let tex_name = if ext == "tex" {
        clean_req.to_string()
      } else {
        s!("{}.tex", clean_req)
      };
      if load_binding(&tex_name)? {
        return Ok(());
      }
    }
    // Couldn't find anything?
    note_status(LogStatus::Missing, Some(request));
    Error!(
      "missing_file",
      request,
      s!("Can't find TeX file {}", request)
    );
    Ok(())
  }
}

fn load_tex_definitions(
  request: &str,
  pathname: &str,
  reloadable: bool,
  at_letter: bool,
  grandparent_in_expl3: bool,
) -> Result<()> {
  // Perl Package.pm L2334: $STATE->getStomach->leaveHorizontal_internal;
  // Defensive cleanup before reading definitions — if we're somehow in
  // horizontal mode while bound to vertical (e.g. after \par-less inline
  // text), repack and flip MODE in-place. No-op in the common case but
  // matches Perl's pre-load state hygiene.
  crate::stomach::leave_horizontal_internal();

  // Snapshot expl3-state at load entry. The cleanup hook below should
  // only restore catcodes if THIS load activated expl3; if the calling
  // context was already in expl3 mode (e.g. tasks.sty has run
  // `\ExplSyntaxOn` and is now `\file_input:n` ing a child file like
  // tasks.cfg), we must preserve the active state for the caller.
  // Without this guard, the nested cleanup would reset `_` and `:` to
  // OTHER/SUB inside the parent's processing, breaking everything past
  // the nested load (e.g. tasks.sty line 817's `\file_input_stop:`).
  // Witness for this exact failure: arXiv:2602.21210, 2604.21347,
  // 2604.22630, 2604.23234, 2604.22528 (tasks.sty + expl3 cluster,
  // Task #20).
  let entered_expl3 = lookup_catcode('_') == Some(Catcode::LETTER);

  if !pathname::is_literaldata(pathname) {
    // We can't analyze literal data's pathnames!
    // let (dir, name, extension) = pathname::split(pathname);

    // Don't load if we've already loaded it before.
    // Note that we'll still load it if we've already loaded only the ltxml version
    // since someone's presumably asking _explicitly_ for the raw TeX version.
    // It's probably even the ltxml version is asking for it!!
    // Of course, now it will be marked and wont get reloaded!
    // Per OXIDIZED_DESIGN #23: raw .sty/.cls/.def load tracks
    // `<request>_raw_loaded`, separate from the binding `<request>_loaded`.
    // This lets a binding .rs load the raw file of the same name without
    // the flags clobbering each other.
    if lookup_bool(&s!("{request}_raw_loaded")) && !reloadable && !pathname::is_reloadable(pathname)
    {
      return Ok(());
    }
    assign_value(&s!("{request}_raw_loaded"), true, Some(Scope::Global));
  }

  // Note that we are reading definitions (and recursive input is assumed also definitions)
  let was_interpreting = lookup_bool_sym(crate::pin!("INTERPRETING_DEFINITIONS"));
  // And that if we're interpreting this TeX file of definitions,
  // we probably should interpret any TeX files IT loads.
  let was_including_styles = lookup_bool("INCLUDE_STYLES");
  assign_value_sym(crate::pin!("INTERPRETING_DEFINITIONS"), true, None);
  // If we're reading in these definitions, probaly will accept included ones?
  // (but not forbid ltxml ?)
  assign_value("INCLUDE_STYLES", true, None);
  // When set, this variable allows redefinitions of locked defns.
  // It is set in before/after methods to allow local rebinding of commands
  // but loading of sources & bindings is typically done in before/after methods of constructors!
  // This re-locks defns during reading of TeX packages.
  local_state_unlocked(false);
  let content_str = lookup_string(&s!("{pathname}_contents"));
  let content = if content_str.is_empty() {
    None
  } else {
    Some(content_str)
  };
  let pathname_mouth = Mouth::create(pathname, MouthOptions {
    fordefinitions: true,
    at_letter,
    notes: true,
    content,
    ..MouthOptions::default()
  })?;

  gullet::reading_from_mouth(pathname_mouth, move || -> Result<()> {
    while let Some(token) = gullet::read_x_token(Some(false), false, None)? {
      if token != T_SPACE!() {
        invoke_token(&token)?;
      }
    }
    Ok(())
  })?;

  // Expl3 scope-exit cleanup: if a raw .sty load activated expl3 catcodes
  // via `\ProvidesExplPackage` or explicit `\ExplSyntaxOn` and forgot to
  // pair it with `\ExplSyntaxOff` (e.g. lipsum.sty, which relies on an
  // `\AtEndOfPackage`-style hook the autoload chain doesn't register),
  // digest `\ExplSyntaxOff` now so the pending `\group_begin:` frame pops
  // and catcodes restore before the next package loads.
  //
  // Perl's `TeX.pool.ltxml` L44-47 acknowledges this as a known edge-
  // case of the `\ProvidesExplPackage` autoload pattern.
  //
  // Skip expl3 / xparse / l3keys2e / expl3-code — those legitimately
  // leave expl3 active for their callers.
  {
    let (_, base, _ext) = pathname::split(pathname);
    let is_expl3_core = matches!(
      base.as_str(),
      "expl3" | "xparse" | "l3keys2e" | "expl3-code"
    );
    // Use grandparent_in_expl3 (snapshotted before `\@pushfilename`)
    // rather than entered_expl3 (snapshotted after the push flipped `_`
    // to SUB). Without this, sub-loads inside an active expl3 frame
    // saw entered_expl3=false (because of the push's `\ExplSyntaxOff`)
    // and over-fired `\ExplSyntaxOff` at exit, which then leaks SUB
    // into the grandparent's continued reading once `\@popfilename`
    // pops the status stack and would otherwise restore `\ExplSyntaxOn`.
    // Witness: `\usepackage{xsavebox}` minimal repro (xsavebox →
    // sys_load_backend → l3backend-dvips.def); arXiv:2509.05997/.07893/
    // .02344, 2510.13206/.13942/.17317.
    if !is_expl3_core
      && !grandparent_in_expl3
      && lookup_catcode('_') == Some(Catcode::LETTER)
      && lookup_definition(&T_CS!("\\ExplSyntaxOff"))?.is_some()
    {
      let _ = invoke_token(&T_CS!("\\ExplSyntaxOff"));
    }
    let _ = entered_expl3; // kept for historical context
  }

  assign_value_sym(
    crate::pin!("INTERPRETING_DEFINITIONS"),
    was_interpreting,
    None,
  );
  assign_value("INCLUDE_STYLES", was_including_styles, None);
  expire_state_unlocked();

  // Perl Package.pm L2376: Let(T_CS('\ver@'.$request), T_CS('\fmtversion'), 'global');
  // Mark the raw .sty/.tex as loaded so LaTeX's `\@ifpackageloaded` and
  // `\RequirePackage` date-version guards work after a raw TeX load. Perl
  // unconditionally Lets here (in contrast to the LTXML loader at line 339,
  // which only Lets when undefined).
  let ver_cs = T_CS!(s!("\\ver@{}", request));
  let_i(&ver_cs, &T_CS!("\\fmtversion"), Some(Scope::Global));

  Ok(())
}

pub fn load_tex_content(path: &str, _options: InputOptions) -> Result<()> {
  // If there is a file-specific declaration file (name_tex.rs), load it first!
  // TODO: is this `.latexml` variation still relevant in the Rust port?
  let _has_binding = if !pathname::is_literaldata(path) {
    let (_dir, base, _ext) = pathname::split(path);
    load_external_binding(&base)? || load_binding(&base)?
  } else {
    false
  };

  // Open a mouth for that TeX content
  let cached = lookup_string(&s!("{path}_contents"));
  let cached_opt = if cached.is_empty() {
    None
  } else {
    Some(cached)
  };
  gullet::open_mouth(
    Mouth::create(path, MouthOptions {
      notes: true,
      content: cached_opt,
      ..MouthOptions::default()
    })?,
    true,
  );
  Ok(())
}

/// Pass the sequence of @options to the package $name (if $ext is 'sty'),
/// or class $name (if $ext is 'cls').
/// Perl Package.pm: PassOptions($name, $ext, @options)
/// Stores options to be processed when the package/class is loaded.
pub fn pass_options(name: &str, ext: &str, options: Vec<String>) -> Result<()> {
  let key = s!("opt@{}.{}", name, ext);
  for opt in options {
    push_value(&key, arena::pin(&opt))?;
  }
  Ok(())
}

/// Perl Package.pm L2430-2465: ProcessOptions / ProcessOptions*
/// `inorder=false` (\ProcessOptions) — execute in declared order, default handler for undeclared
/// `inorder=true` (\ProcessOptions*) — execute in order passed, class options silently skipped
pub fn process_options(inorder: bool, keysets: &[&str]) -> Result<()> {
  let currname_token = T_CS!("\\@currname");
  let currext_token = T_CS!("\\@currext");
  let name = if lookup_definition(&currname_token)?.is_some() {
    do_expand(currname_token)?.to_string()
  } else {
    String::new()
  };
  let ext = if lookup_definition(&currext_token)?.is_some() {
    do_expand(currext_token)?.to_string()
  } else {
    String::new()
  };
  let declared_options: VecDeque<Stored> = lookup_vecdeque("@declaredoptions").unwrap_or_default();
  let opt_key = s!("opt@{}.{}", name, ext);
  let current_options = lookup_vecdeque(&opt_key).unwrap_or_default();
  let class_options = lookup_vecdeque("class_options").unwrap_or_default();

  let collect_syms = |vdq: &VecDeque<Stored>| -> Vec<SymStr> {
    let mut list = Vec::new();
    for item in vdq.iter() {
      match item {
        Stored::String(s) => {
          list.push(*s);
        },
        Stored::Strings(ss) => {
          for s in ss.iter() {
            list.push(*s);
          }
        },
        _ => {},
      }
    }
    list
  };
  let cur_options_list = collect_syms(&current_options);
  let cls_options_list = collect_syms(&class_options);

  if inorder {
    // Perl L2447-2453: ProcessOptions* — execute in the order passed
    // Class options: try executeOption_internal only (no default fallback)
    for option in &cls_options_list {
      let _ = execute_option_internal(*option, keysets)?;
    }
    // Current options: try executeOption, then default handler
    for option in &cur_options_list {
      if !execute_option_internal(*option, keysets)? {
        execute_default_option_internal(*option)?;
      }
    }
  } else {
    // Perl L2454-2461: ProcessOptions — execute in declared order
    let mut cur_set: HashSet<SymStr> = cur_options_list.iter().copied().collect();
    let mut cls_set: HashSet<SymStr> = cls_options_list.iter().copied().collect();

    for option in declared_options.iter() {
      match option {
        Stored::String(content) if cur_set.remove(content) || cls_set.remove(content) => {
          execute_option_internal(*content, keysets)?;
        },
        Stored::Strings(contents) => {
          for content in contents.iter() {
            if cur_set.remove(content) || cls_set.remove(content) {
              execute_option_internal(*content, keysets)?;
            }
          }
        },
        _ => {},
      }
    }
    // Only undeclared CURRENT options go to default handler (not class options).
    // Perl L2460-2461: "foreach my $option (@curroptions)" — class options excluded.
    // Iterate cur_options_list (Vec, ordered) instead of cur_set (HashSet,
    // unordered) so unknown options enter `@unusedoptionlist` in source
    // order. Otherwise `\documentstyle[a,b,c]` produces an arbitrary
    // dispatch order, which breaks paper-local option chains that depend
    // on left-to-right evaluation (e.g. `[aaspp4,tighten]` requires
    // aaspp4's bindings — \tightenlines — to be defined before tighten.sty
    // body fires; driver: astro-ph9707180).
    for option in &cur_options_list {
      if cur_set.contains(option) {
        execute_default_option_internal(*option)?;
      }
    }
  }
  // Now, undefine the handlers
  for option in declared_options.iter() {
    let_i(&T_CS!(s!("\\ds@{}", option)), &T_RELAX!(), None);
  }
  Ok(())
}

fn execute_option_internal(option: SymStr, keysets: &[&str]) -> Result<bool> {
  if let Some((qname, value)) = keyval_option_qname(option, keysets) {
    // Perl Package.pm handles `key=value` package options before normal
    // `\ds@...` lookup when ProcessOptions was given keysets. It digests
    // `\KV@<keyset>@<key>{<value>}`. Rust DefKeyVal entries without code
    // are ordinary macros, so also store the value under the qname for
    // package bindings that apply keyvals after ProcessOptions.
    assign_value(
      &qname,
      Stored::String(arena::pin(value.trim())),
      Some(Scope::Global),
    );
    digest(Tokens!(
      T_CS!(s!("\\{qname}")),
      T_BEGIN!(),
      ExplodeText!(&value),
      T_END!()
    ))?;
    return Ok(true);
  }

  let cs = T_CS!(arena::with(option, |opt| s!("\\ds@{opt}")));
  if lookup_definition(&cs)?.is_some() {
    // Perl Package.pm L2482: `DefMacroI('\CurrentOption', undef, $option)` —
    // tokenizes `$option` via Tokens(Explode($option)) so letters get
    // catcode LETTER and others OTHER. Babel's `\ifx\CurrentOption\bbl@tempa`
    // (where `\bbl@tempa{frenchb}` produces LETTER tokens) only matches when
    // our `\CurrentOption` body has the same catcodes — packing the whole
    // option string into one OTHER-catcode "string" token would make the
    // \ifx silently false. Use SymExplodeText! to split per-character.
    def_macro(
      T_CS!("\\CurrentOption"),
      None,
      Tokens!(SymExplodeText!(option)),
      None,
    )?;

    let unused = match remove_vecdeque("@unusedoptionlist") {
      Some(list) => list
        .into_iter()
        .filter(|item| {
          if let Stored::String(content) = item {
            *content != option
          } else {
            false
          }
        })
        .collect(),
      None => VecDeque::new(),
    };
    assign_value("@unusedoptionlist", Stored::VecDequeStored(unused), None);
    digest(cs)?;
    Ok(true)
  } else {
    Ok(false)
  }
}

fn keyval_option_qname(option: SymStr, keysets: &[&str]) -> Option<(String, String)> {
  if keysets.is_empty() {
    return None;
  }
  let (key, value) = arena::with(option, |opt| {
    opt
      .split_once('=')
      .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
  })?;
  if key.is_empty() {
    return None;
  }
  for keyset in keysets {
    let qname = crate::keyval::keyval_qname("KV", keyset, &key);
    if crate::keyval::keyval_get(&qname, "type").is_some() {
      return Some((qname, value));
    }
  }
  None
}

fn execute_default_option_internal(option: SymStr) -> Result<bool> {
  // Perl Package.pm L2494: `DefMacroI('\CurrentOption', undef, $option)`.
  // Same catcode-faithful tokenization as execute_option_internal.
  def_macro(
    T_CS!("\\CurrentOption"),
    None,
    Tokens!(SymExplodeText!(option)),
    None,
  )?;
  digest(T_CS!("\\default@ds"))?;
  Ok(true)
}

fn reset_options() -> Result<()> {
  assign_value(
    "@declaredoptions",
    Stored::VecDequeStored(VecDeque::new()),
    None,
  );
  let opt_unused_cs = if gullet::do_expand(T_CS!("\\@currext"))?.eq_text("cls") {
    "\\OptionNotUsed"
  } else {
    "\\@unknownoptionerror"
  };
  let_i(&T_CS!("\\default@ds"), &T_CS!(opt_unused_cs), None);
  Ok(())
}

/// Execute a list of options (Perl: ExecuteOptions).
/// Tries each option's \ds@{option} definition; logs unexpected ones.
pub fn execute_options(options: &[&str]) -> Result<()> {
  let mut unhandled = Vec::new();
  for option in options {
    let sym = arena::pin(*option);
    if !execute_option_internal(sym, &[])? {
      unhandled.push(*option);
    }
  }
  for option in &unhandled {
    Info!(
      "unexpected",
      *option,
      s!("Unexpected options passed to ExecuteOptions '{option}'")
    );
  }
  Ok(())
}

pub struct RequireOptions {
  pub options:          Vec<String>,
  pub withoptions:      Option<Vec<String>>,
  pub extension:        Option<Cow<'static, str>>,
  pub searchpaths_only: bool,
  pub as_class:         bool,
  pub noltxml:          Option<bool>,
  pub notex:            Option<bool>,
  pub after:            Tokens,
}
impl Default for RequireOptions {
  fn default() -> Self {
    RequireOptions {
      options:          Vec::new(),
      withoptions:      None,
      extension:        None,
      notex:            None,
      noltxml:          None,
      as_class:         false,
      searchpaths_only: false,
      after:            Tokens!(),
    }
  }
}

/// An opinionated binding for \RequirePackage.
///
/// This (and `FindFile`) needs to evolve a bit to support reading raw .sty (.def, etc) files from
/// the standard texmf directories.  Maybe even use kpsewhich itself (INSTEAD of `pathname_find`
/// ???) Another potentially useful option might be that if we are reading a raw file,
/// perhaps it should just get digested immediately, since it shouldn't contribute any boxes.
pub fn require_package(name: &str, mut options: RequireOptions) -> Result<()> {
  // Perl Package.pm L2671-2672: notex defaults to true unless the user
  // explicitly set it, or INCLUDE_STYLES is true, or noltxml was passed
  // (a raw-only load explicitly requests raw TeX).
  if options.notex.is_none()
    && !lookup_bool("INCLUDE_STYLES")
    && !matches!(options.noltxml, Some(true))
  {
    options.notex = Some(true);
  }
  // Perl Package.pm L2674: top-level \RequirePackage can be limited to
  // local sources via searchpaths_only. Triggered by the `localrawstyles`
  // option to latexml.sty (sets `INCLUDE_STYLES => 'searchpaths'`).
  // Only applies when raw TeX is allowed (notex==false); otherwise the
  // gate is moot since find_file won't search on-disk anyway.
  if !options.searchpaths_only
    && !matches!(options.notex, Some(true))
    && lookup_string("INCLUDE_STYLES") == "searchpaths"
  {
    options.searchpaths_only = true;
  }
  if options.extension.is_none() {
    options.extension = Some("sty".into());
  }
  let result = input_definitions(name, InputDefinitionOptions {
    extension: options.extension,
    handleoptions: true,
    // Pass classes options if we have NONE!
    withoptions: if options.options.is_empty() {
      Some(Vec::new())
    } else {
      None
    }, // fake boolean use, multi-type in latexml... refactor?
    options: options.options,
    as_class: options.as_class,
    noltxml: options.noltxml.unwrap_or(false),
    notex: options.notex.unwrap_or(false),
    searchpaths_only: options.searchpaths_only,
    after: options.after,
    ..InputDefinitionOptions::default()
  });
  // Perl Package.pm L2679 maybeRequireDependencies is invoked from
  // input_definitions's miss-handler. But that handler only runs when
  // the file was NOT found at all. For paper-bundled .sty files that
  // raw-load successfully without a .sty.ltxml binding, we still need
  // to scan transitive \RequirePackage so that bound deps fire too.
  // Mirrors the same fix for load_class (search "cls.ltxml_loaded").
  // Witness 2208.07400 (paper-bundled emnlp2022.sty has
  // \RequirePackage{caption} + others; without scan, \captionsetup
  // and similar are undefined).
  if !lookup_bool(&s!("{name}.sty.ltxml_loaded")) {
    maybe_require_dependencies(name, "sty");
  }
  result
}

/// Perl: `RequirePackage($name, withoptions => 1)` — forward the current
/// package/class's options to the required child package. Reads
/// `\@currname` / `\@currext` to identify the caller, looks up its
/// `opt@<name>.<ext>` options, and passes them explicitly as the child's
/// options list. Mirrors `load_class_with_options` for the package path.
pub fn require_package_with_options(name: &str) -> Result<()> {
  let currname = if lookup_definition(&T_CS!("\\@currname"))?.is_some() {
    do_expand(T_CS!("\\@currname"))?.to_string()
  } else {
    String::new()
  };
  let currext = if lookup_definition(&T_CS!("\\@currext"))?.is_some() {
    do_expand(T_CS!("\\@currext"))?.to_string()
  } else {
    String::new()
  };
  let options: Vec<String> = if !currname.is_empty() {
    let key = s!("opt@{}.{}", currname, currext);
    lookup_vecdeque(&key)
      .unwrap_or_default()
      .iter()
      .filter_map(|item| match item {
        Stored::String(s) => Some(arena::to_string(*s)),
        _ => None,
      })
      .collect()
  } else {
    Vec::new()
  };
  require_package(name, RequireOptions {
    options,
    ..RequireOptions::default()
  })
}

/// Perl Package.pm L2759-2796: maybeRequireDependencies
/// When a package/class file has no binding AND raw TeX loading is disabled,
/// scan the raw file for \RequirePackage/\usepackage/\LoadClass declarations
/// and load any dependencies that DO have bindings. This is a "best effort"
/// fallback that gives us the dependency chain without interpreting raw TeX.
// Strict translation of Perl `Package.pm:maybeRequireDependencies`
// (L2759-L2796). Scan a raw .sty/.cls file for transitive
// `\RequirePackage`, `\usepackage`, and (for classes) `\LoadClass`
// declarations and route them through `require_package` / `load_class`
// so the corresponding bindings get pulled in even when the original
// file has no .ltxml binding.
fn maybe_require_dependencies(file: &str, ext_type: &str) {
  use once_cell::sync::Lazy;
  use regex::Regex;

  // Rust-only re-entrancy guard. Perl avoids this case by other means
  // (the call-site of `maybeRequireDependencies` is the only entry).
  thread_local! { static SCANNING: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
  if SCANNING.with(|s| s.get()) {
    return;
  }
  SCANNING.with(|s| s.set(true));
  struct ResetGuard;
  impl Drop for ResetGuard {
    fn drop(&mut self) { SCANNING.with(|s| s.set(false)); }
  }
  let _guard = ResetGuard;

  // EXECUTED-SET GATE (Rust-only, more-robust-than-Perl). When this file was
  // actually RAW-LOADED, its `\usepackage`/`\RequirePackage` constructors ran
  // for every require the load REACHED — recording `<pkg>.usepackage_executed`.
  // A candidate the regex finds in the text but that is NOT in that set is one
  // whose `\usepackage` never executed — i.e. it sits inside a FALSE `\if…\fi`
  // the raw-load already skipped (e.g. `\ifpdf … \usepackage{hyperref} … \fi`
  // with `\ifpdf` false). Perl never dep-scans a raw-loaded file, so it never
  // anticipates such a package; mirror that by skipping it. The gate applies
  // ONLY when the file raw-loaded (key `<file>.<ext>_raw_loaded`): in the
  // miss-handler / `INCLUDE_STYLES=false` path no constructor ran, so the set
  // is empty and we must NOT filter (keep the anticipation). The flag is
  // cumulative+global, so a candidate that DID execute anywhere is kept — only
  // never-executed ones are dropped, which can never be a false skip.
  // Witnesses 1910.05586 (hyperref in false `\ifpdf` → cleveref "must be loaded
  // after hyperref"), 1804.09301 (xcolor in false `\ifacl@hyperref`).
  let raw_loaded = lookup_bool(&s!("{file}.{ext_type}_raw_loaded"));

  // Perl L2776: `s/%[^\n]*\n//gs` — drop comment AND its trailing newline,
  // replacement is the empty string.
  static COMMENT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"%[^\n]*\n").unwrap());
  // `comment`-package block: `\begin{comment}…\end{comment}` is a verbatim-SKIP
  // environment, so a `\usepackage`/`\RequirePackage` inside it is NEVER loaded
  // by LaTeX. The dep-scan must not anticipate it. Same "more-robust than Perl"
  // rationale as the macro-def-body skip below. Witness 1901.05713: thesis.sty
  // has a commented-out `\usepackage{hyperref}` inside `\begin{comment}`, which
  // the scan otherwise loaded — tripping cleveref's "must be loaded after
  // hyperref" `\AtBeginDocument` order-check (hyperref appears loaded though the
  // author commented it out). `comment` doesn't nest, so a non-greedy match to
  // the first `\end{comment}` is correct.
  static COMMENT_ENV_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)\\begin\s*\{comment\}.*?\\end\s*\{comment\}").unwrap());
  // Perl L2777-2779 runs two separate substitutions, in this order:
  // first `\RequirePackage`, then `\usepackage`. Use two regexes so that
  // collected order matches Perl's call order to `$collect`.
  static REQ_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\\RequirePackage\s*(?:\[([^\]]*)\])?\s*\{([^\}]*)\}").unwrap());
  static USE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\\usepackage\s*(?:\[([^\]]*)\])?\s*\{([^\}]*)\}").unwrap());
  static CLS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\\LoadClass\s*(?:\[([^\]]*)\])?\s*\{([^\}]*)\}").unwrap());
  // Matches a `\newcommand`/`\def`-family DEFINITION HEADER ending exactly at a
  // `{` — i.e. the brace that follows opens the macro BODY. Used to detect a
  // `\usepackage` that lives in a deferred macro body (loads only when the
  // macro is later expanded) vs one inside a load-time conditional.
  static DEF_BODY_HEADER_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
      r"(?s)(?:\\(?:re|provide)?newcommand|\\DeclareRobustCommand)\*?\s*(?:\{\s*\\[A-Za-z@]+\s*\}|\\[A-Za-z@]+)\s*(?:\[[^\]]*\]\s*)*$|\\[egx]?def\s*\\[A-Za-z@]+[^{}]*$",
    )
    .unwrap()
  });
  // The `\def`-family sub-case of DEF_BODY_HEADER_RE, capturing the defined
  // macro NAME (without backslash). A `\def\<m>{… \RequirePackage{P} …}` body
  // is only truly deferred if `\<m>` is never invoked; the AMS-class idiom
  // `\def\@tempa{\RequirePackage{amsmath}}…\@tempa` invokes it immediately and
  // so DOES load P (witness ijnam.cls → amsmath → `{aligned}`, 1911.03415).
  static DEF_NAME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)\\[egx]?def\s*\\([A-Za-z@]+)[^{}]*$").unwrap());

  // Perl L2761: `FindFile($file, type => $type, noltxml => 1)`. `$file`
  // is BARE — `FindFile` glues on `.$type` itself per L2073-2076.
  let raw_path = find_file(
    file,
    Some(FindFileOptions {
      ext_type: Some(Cow::Owned(ext_type.to_string())),
      forbid_ltxml: true, // Perl `noltxml => 1`
      ..FindFileOptions::default()
    }),
  );
  let Some(path) = raw_path else { return };

  // Perl L2762-2766: slurp file. Check filecontents-cache first for the
  // inline-cls/sty case (e.g. `\begin{filecontents}{alggeom.cls}`), then
  // fall through to disk. Without the cache check, papers that bundle
  // their .cls inline via filecontents miss the dep-scan and downstream
  // CSes that the (now-cached) cls would have hand-loaded stay
  // undefined. Witness: arXiv:2604.09738.
  let cached = lookup_string(&s!("{}_contents", path));
  let code = if !cached.is_empty() {
    cached
  } else {
    // Use read (bytes) + lossy UTF-8 conversion so non-UTF-8 cls/sty
    // files (ISO-8859 with vendor copyright headers, e.g. cpc-hepnp.cls
    // with Chinese comments) still get scanned. read_to_string strict
    // UTF-8 validation would error out, leaving \RequirePackage{fancyhdr}
    // and friends silently undiscovered. Witness 2203.16500.
    match std::fs::read(&path) {
      Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
      Err(_) => {
        Warn!(
          "I/O",
          "read",
          s!("Couldn't open {} to scan dependencies, $!", path)
        );
        return;
      },
    }
  };

  // Perl L2776: strip comments (replacement empty).
  let code = COMMENT_RE.replace_all(&code, "");
  // Strip `\begin{comment}…\end{comment}` blocks (see COMMENT_ENV_RE above).
  let code = COMMENT_ENV_RE.replace_all(&code, "");

  // DIVERGENCE FROM PERL (deliberate, more-robust): do NOT dep-load a
  // `\usepackage` / `\RequirePackage` that sits in a DEFERRED macro-definition
  // body (a `\newcommand` / `\def`-family body) — it loads only if that macro
  // is later expanded, which the dep-scan must not eagerly force. Requires
  // inside LOAD-TIME conditionals (`\IfFileExists{X.sty}{\usepackage{X}}`,
  // `\@ifundefined{..}{..}`, `\if…\usepackage…\fi`) DO execute during raw-load
  // and MUST still be picked up. (Perl never force-loads the deferred ones: for
  // a normally raw-loaded file it doesn't dep-scan at all, and the bundled deps
  // it wants in the binding-bypass case run at load.) Witnesses:
  //   - 1506.06200 — categorytheory.sty `\newcommand{\usediagrams}{\usepackage
  //     {diagrams}}` (never invoked) must be SKIPPED (else the `diagrams` stub's
  //     `locked` `\begin{diagram}` shadows the paper's tikz `{diagram}`).
  //   - 1703.03673 — iau.cls `\IfFileExists{amssymb.sty}{…\usepackage{amssymb}…}`
  //     must be KEPT (else `\bigstar` is undefined). An earlier brace-DEPTH
  //     filter wrongly skipped this conditional; the def-body check is precise.
  // A `\usepackage` is deferred iff ANY enclosing `{…}` group is opened directly
  // by a `\newcommand`/`\def` definition header.
  // Is `\<name>` INVOKED (not merely defined) somewhere in the file? A bare
  // control-sequence occurrence not directly preceded by a `\…def`/`\let`
  // introducer counts as an invocation. Used to tell an executed deferred-load
  // idiom (`\def\@tempa{…\RequirePackage{P}…}\@tempa`) from a never-called one.
  let is_invoked = |name: &str| -> bool {
    let pat = s!("\\{name}");
    let mut from = 0usize;
    while let Some(rel) = code[from..].find(&pat) {
      let at = from + rel;
      from = at + pat.len();
      // Full control sequence: next char must not extend the name.
      let after_ok = code[at + pat.len()..]
        .chars()
        .next()
        .is_none_or(|c| !c.is_ascii_alphabetic() && c != '@');
      if !after_ok {
        continue;
      }
      // Not a (re)definition: the chars just before `\name` aren't `…def`/`…let`.
      let before = code[at.saturating_sub(8)..at].trim_end();
      if before.ends_with("def") || before.ends_with("let") {
        continue;
      }
      return true;
    }
    false
  };
  let in_macro_def_body = |start: usize| -> bool {
    let bytes = code.as_bytes();
    let mut stack: Vec<usize> = Vec::new();
    let mut i = 0usize;
    while i < start {
      match bytes[i] {
        b'\\' => {
          i += 2;
          continue;
        },
        b'{' => stack.push(i),
        b'}' => {
          stack.pop();
        },
        _ => {},
      }
      i += 1;
    }
    stack.iter().any(|&ob| {
      let lo = ob.saturating_sub(400);
      let window = code.get(lo..ob).unwrap_or(&code[..ob]);
      // A `\def\<m>{…}` body only defers if `\<m>` is never invoked; an invoked
      // scratch macro (`\@tempa`) runs its body at load, so its require loads.
      if let Some(caps) = DEF_NAME_RE.captures(window) {
        return !is_invoked(&caps[1]);
      }
      // `\newcommand`/`\DeclareRobustCommand` user-command bodies stay deferred
      // (witness 1506.06200 `\newcommand{\usediagrams}{\usepackage{diagrams}}`).
      DEF_BODY_HEADER_RE.is_match(window)
    })
  };
  let top_level = |cap: &regex::Captures| -> bool {
    cap.get(0).map(|m| !in_macro_def_body(m.start())).unwrap_or(true)
  };

  // NOTE — a former Rust-only "conflicting option sets" heuristic was REMOVED
  // here (was: drop a package `\RequirePackage`'d / `\usepackage`'d with two or
  // more DIFFERENT option sets, as the signature of a deferred require inside a
  // `\def`/`\DeclareOption` body, e.g. aa.cls's
  // `\DeclareOption{ascii}{\def\aa@inputenc{\RequirePackage[ascii]{inputenc}}}`
  // …`{utf8}{…[utf8]…}`). It over-fired on a package required in BOTH arms of a
  // load-time `\if…\else…\fi` with different options — mutually exclusive, so
  // exactly one branch loads and the package MUST be kept. Witness: rist.cls's
  // `\ifpdf \RequirePackage[pdftex,…]{hyperref} \else \RequirePackage[dvipdfm,…]
  // {hyperref} \fi` (1912.00781: hyperref dropped → `\url` undefined). The
  // heuristic was also provably REDUNDANT: it only ever saw TOP-LEVEL requires
  // (the captures feeding it were `top_level`-filtered), but aa.cls's inputenc
  // lives in a `\def`/`\DeclareOption` BODY — already dropped by the def-body
  // `top_level` skip below, so it never entered the conflicting set at all. Perl
  // has no such gate (L2767-2774 is a plain dedup); the def-body `top_level` skip
  // plus the executed-set gate cover the legitimate cases faithfully.
  static OPT_SPLIT: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s*,\s*").unwrap());

  // Perl L2767-2774: shared `%dups` map, $collect closure splits on
  // `\s*,\s*` and only enrolls a package once, AND only if its
  // `.sty.ltxml_loaded` flag is unset.
  let mut packages: Vec<(String, Option<String>)> = Vec::new();
  let mut dups: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();
  let mut collect = |pkg_csv: &str, raw_options: Option<&str>| {
    for p in OPT_SPLIT.split(pkg_csv) {
      if p.is_empty() {
        continue;
      }
      // Executed-set gate (see top of fn): when this file raw-loaded, drop a
      // candidate whose `\usepackage` never executed anywhere — it was inside a
      // false conditional the raw-load skipped.
      if raw_loaded && !lookup_bool(&s!("{p}.usepackage_executed")) {
        continue;
      }
      // Perl L2773: `!$dups{$p} && !LookupValue($p . '.sty.ltxml_loaded')`
      if !dups.contains(p) && !lookup_bool(&s!("{p}.sty.ltxml_loaded")) {
        packages.push((p.to_string(), raw_options.map(|s| s.to_string())));
        dups.insert(p.to_string());
      }
    }
  };

  // Perl L2777: `\RequirePackage` first.
  for cap in REQ_RE.captures_iter(&code) {
    if !top_level(&cap) {
      continue;
    }
    collect(&cap[2], cap.get(1).map(|m| m.as_str()));
  }
  // Perl L2778-2779: `\usepackage` second.
  for cap in USE_RE.captures_iter(&code) {
    if !top_level(&cap) {
      continue;
    }
    collect(&cap[2], cap.get(1).map(|m| m.as_str()));
  }

  // Perl L2767/L2781-2782: `@classes` is class-only, NO dup-check.
  let mut classes: Vec<(String, Option<String>)> = Vec::new();
  if ext_type == "cls" {
    for cap in CLS_RE.captures_iter(&code) {
      if !top_level(&cap) {
        continue;
      }
      let class = cap[2].to_string();
      if !class.is_empty() {
        classes.push((class, cap.get(1).map(|m| m.as_str().to_string())));
      }
    }
  }

  // Perl L2784-2785: Info iff EITHER list is non-empty; message lists
  // class names then package names, separated by ',' (no space).
  if !classes.is_empty() || !packages.is_empty() {
    let names: Vec<&str> = classes
      .iter()
      .map(|(n, _)| n.as_str())
      .chain(packages.iter().map(|(n, _)| n.as_str()))
      .collect();
    Info!(
      "dependencies",
      "dependencies",
      s!("Loading dependencies for {}: {}", path, names.join(","))
    );
  }

  // Perl L2786-2789: foreach class — gate by `FindFile($class, type=>'cls',
  // notex=>1)`, then `LoadClass(..., options=>[split ...])`.
  for (class, raw_opts) in classes {
    if find_file(
      &class,
      Some(FindFileOptions {
        ext_type: Some(Cow::Borrowed("cls")),
        notex: true, // Perl `notex => 1`
        ..FindFileOptions::default()
      }),
    )
    .is_some()
    {
      let opts: Vec<String> = raw_opts
        .as_deref()
        .map(|s| OPT_SPLIT.split(s).map(|x| x.to_string()).collect())
        .unwrap_or_default();
      let _ = load_class(&class, opts, Tokens::default());
    }
  }

  // Perl L2790-2793: foreach package — gate by `FindFile($pkg, type=>'sty',
  // notex=>1)`, then `RequirePackage(..., options=>[split ...])`.
  for (pkg, raw_opts) in packages {
    if find_file(
      &pkg,
      Some(FindFileOptions {
        ext_type: Some(Cow::Borrowed("sty")),
        notex: true, // Perl `notex => 1`
        ..FindFileOptions::default()
      }),
    )
    .is_some()
    {
      let opts: Vec<String> = raw_opts
        .as_deref()
        .map(|s| OPT_SPLIT.split(s).map(|x| x.to_string()).collect())
        .unwrap_or_default();
      let _ = require_package(&pkg, RequireOptions {
        options: opts,
        ..RequireOptions::default()
      });
    }
  }
}

pub fn require_resource(mut resource: Resource) {
  if resource.name.is_empty() && resource.content.is_empty() {
    Warn!(
      "expected",
      "resource",
      "Resource must have a resource pathname or content; skipping"
    );
    return;
  }
  if resource.mimetype.is_empty() && !resource.name.is_empty() {
    // Perl Package.pm L3129: `my $ext = pathname_type($resource);` — no
    // case-folding; `$resource_types{$ext}` is a case-sensitive lookup.
    let ext = pathname::extension(&resource.name);
    resource.mimetype = resource_type(&ext);
  }
  if resource.mimetype.is_empty() {
    Warn!(
      "expected",
      "mime-type",
      "Resource must have a mime-type; skipping"
    );
    return;
  }

  // If we've got a document, go ahead & put the resource in.
  // if (document.is_some()) {
  //   document.as_mut().unwrap().add_resource(resource, resource);
  // } else {
  push_pending_resource(resource);
  // }
}

/// Perl: `LoadClass($name, withoptions => 1)` — load a class passing the
/// caller's class options through to the child. Reads `class_options` from
/// state (populated by the outer `\documentclass` invocation) and forwards
/// those as the child's options list, matching Perl Package.pm LoadClass's
/// `withoptions` branch.
pub fn load_class_with_options(name: &str, after: Tokens) -> Result<()> {
  let class_opts = lookup_vecdeque("class_options").unwrap_or_default();
  let options: Vec<String> = class_opts
    .iter()
    .filter_map(|item| match item {
      Stored::String(s) => Some(arena::to_string(*s)),
      _ => None,
    })
    .collect();
  load_class(name, options, after)
}

pub fn load_class(name: &str, options: Vec<String>, after: Tokens) -> Result<()> {
  // Perl Package.pm LoadClass: $options{notex}=1 unless LookupValue('INCLUDE_CLASSES').
  // Defaults to NOT loading raw .cls. Only .cls.ltxml bindings are considered;
  // if the binding is missing, fall through to OmniBus (below). Allowing raw
  // .cls to "succeed" the load prevents the OmniBus fallback that provides
  // generic frontmatter / counter / theorem bindings.
  //
  // PERL-FAITHFUL (2026-05-30): a directory prefix does NOT force a raw .cls
  // load. Perl resolves a path-prefixed class to its basename BINDING when one
  // exists, else OmniBus — it loads `IEEEtran.cls.ltxml` for
  // `\documentclass{misc/ieeetran}` and falls to OmniBus for
  // `\documentclass{JINST-Sample-files/JINST}`. The basename→binding match is
  // handled in the `alternate` search below (it strips the path). Forcing a raw
  // load for any path-prefixed name (the old `&& !has_path_prefix` exception)
  // broke bundled classes whose raw load is semantically incomplete — JINST's
  // begin-document `\author`/`\abstract` checks fire and `\abstract@cs` is left
  // undefined, where Perl (OmniBus) is clean. Witness 1504.01965; the
  // misc/ieeetran case (2105.02087) now matches Perl via the basename binding.
  let notex_default = !lookup_bool("INCLUDE_CLASSES");
  // Perl Package.pm L2690: LoadClass can be limited to local SEARCHPATHS when
  // `localrawclasses` option sets `INCLUDE_CLASSES => 'searchpaths'`.
  let searchpaths_only = !notex_default && lookup_string("INCLUDE_CLASSES") == "searchpaths";

  let result = input_definitions(name, InputDefinitionOptions {
    extension: Some(Cow::Borrowed("cls")),
    options: options.clone(),
    after: after.clone(),
    notex: notex_default,
    searchpaths_only,
    handleoptions: true,
    noerror: true,
    ..InputDefinitionOptions::default()
  });
  // Perl Package.pm L2700-2716: if no direct binding, try a prefix-match fallback.
  // Scan all known cls bindings (longest-first), pick the first whose name is a
  // prefix of the requested class. This catches author-renamed classes like
  //   mysvjour3.cls → ProvidesClass{svjour3} → binding: svjour3
  //   mn2ebis.cls   → starts with "mn2e"   → binding: mn2e
  //   IEEEtranTCOM.cls → starts with "IEEEtran" → binding: IEEEtran
  // Fall through to OmniBus only when nothing matches.
  let will_fallback = (result.is_err()
    || (!lookup_bool(&format!("{name}.cls_loaded"))
      && !lookup_bool(&format!("{name}.cls_raw_loaded"))))
    && name != "OmniBus"
    && name != "article"
    && !lookup_bool("OmniBus.cls_loaded")
    && !lookup_bool("OmniBus.cls_raw_loaded");

  // Perl Package.pm L2679 (LoadClass branch): scan the raw .cls for
  // \usepackage/\RequirePackage/\LoadClass dependencies when no .cls.ltxml
  // binding was found. This matters for unknown classes that nonetheless
  // pull in well-known packages (e.g. ijms-preprint.cls loads amsmath);
  // without it, downstream code like `\eqref{foo_bar}` sees `\eqref` as
  // undefined and the `_` characters then reach the stomach as subscript
  // catcodes, triggering runaway error recovery (arxiv 1003.0934 OOM).
  // Skip deps-scan only when a real `.cls.ltxml` binding has been
  // loaded — that binding is responsible for its own
  // `\RequirePackage` calls. The `cls_loaded` flag is set even for
  // a successful raw .cls load (no binding), so we MUST check the
  // binding-specific `cls.ltxml_loaded` flag instead. Without
  // this, paper-bundled .cls files (e.g. myclass.cls bundling
  // caption + many others, witness 2202.11535) raw-load
  // successfully but their `\RequirePackage` calls do NOT trigger
  // our binding loaders, leaving \captionsetup / \href / \affil
  // undefined.
  //
  // PERL-FAITHFUL ORDER: when we will fall through to an alternate
  // class binding (OmniBus or a prefix-match), DEFER the deps-scan
  // until AFTER the alternate is loaded. Otherwise the deps-scan
  // pulls natbib (et al.) ahead of OmniBus, and OmniBus's later
  // `Let('\lx@OmniBus@saved@bibitem', '\bibitem')` +
  // `DefMacro('\bibitem', ...)` clobbers natbib's `\lx@nat@bibitem`
  // binding — infinite-loop chain on
  // `\bibitem[\protect\citeauthoryear{...}{...}{...}]{key}`.
  // Witness: 1001.1919, 1001.5004, 0809.4358 (statsoc.cls),
  // 0904.3132, 0912.1617 (ectj.cls), 0904.3938 (compositio.cls),
  // 0908.3882 (third-input timeout), 0911.1590 (\tag\textsc cascade).
  // Perl's load order in this branch: warn missing-binding → load
  // alternate (OmniBus) → deps-scan pulls natbib LAST → natbib's Let
  // overrides OmniBus correctly.
  if !lookup_bool(&s!("{name}.cls.ltxml_loaded")) && !will_fallback {
    maybe_require_dependencies(name, "cls");
  }
  if will_fallback {
    note_status(LogStatus::Missing, Some(&format!("{name}.cls")));

    // Perl: @classes = sort { -(length($a) <=> length($b)) } available_cls_names
    //       my ($alternate) = grep { $class =~ /^\Q$_\E/ } @classes;
    // Flatten across ALL registered binding crates (latexml_package +
    // latexml_contrib + any future extensions) so contrib classes like
    // `memoir`, `siamltex`, `scrbook` are eligible alternates too.
    let alternate = {
      let mut sorted: Vec<&str> = crate::state::get_class_binding_names()
        .into_iter()
        .filter(|n| *n != "OmniBus" && *n != name)
        .collect();
      sorted.sort_by_key(|n| std::cmp::Reverse(n.len()));
      // Strict-case prefix first (Perl-faithful: `$class =~ /^\Q$_\E/`), then
      // a case-insensitive fallback for binding entries that differ from the
      // class name ONLY in capitalization (e.g. `WileyNJDv5` class vs a
      // `wileyNJDv5` binding entry, witness 2406.08163). The ci fallback uses
      // FULL equality, NOT prefix: a ci-PREFIX match wrongly fired
      // `AAAI-Std` → `aa` (the 2-char A&A astronomy binding) because
      // `"aaai-std".starts_with("aa")` — but Perl's case-SENSITIVE `/^aa/`
      // against `AAAI-Std` finds nothing, so Perl falls back to OmniBus
      // (which defines `\address`). Full ci-equality keeps the
      // capitalization-only Wiley case while matching Perl's no-match →
      // OmniBus for AAAI-Std. Witness 2008.08548.
      sorted
        .iter()
        .copied()
        .find(|candidate| name.starts_with(candidate))
        .or_else(|| {
          sorted
            .iter()
            .copied()
            .find(|candidate| name.eq_ignore_ascii_case(candidate))
        })
        .or_else(|| {
          // Path-prefixed class (`misc/ieeetran`, `JINST-Sample-files/JINST`):
          // strip the directory and match the basename against a binding, so
          // `misc/ieeetran` → IEEEtran (Perl loads IEEEtran.cls.ltxml for it)
          // while `JINST-Sample-files/JINST` → no basename binding → OmniBus.
          // Case-insensitive FULL equality only (not prefix) — mirrors the
          // capitalization-only ci fallback above and avoids a basename like
          // `AAAI-Std` wrongly prefix-matching the 2-char `aa` binding.
          // Witnesses 1504.01965 (JINST→OmniBus), 2105.02087 (misc/ieeetran).
          let basename = name.rsplit(['/', '\\']).next().unwrap_or(name);
          if basename != name {
            sorted
              .iter()
              .copied()
              .find(|candidate| basename.eq_ignore_ascii_case(candidate))
          } else {
            None
          }
        })
    };

    let target = alternate.unwrap_or("OmniBus");
    Warn!(
      "missing_file",
      name,
      format!("Can't find binding for class {name} (using {target})"),
      "Anticipate undefined macros or environments"
    );
    let loaded = input_definitions(target, InputDefinitionOptions {
      extension: Some(Cow::Borrowed("cls")),
      options,
      after,
      notex: true,
      handleoptions: true,
      noerror: true,
      ..InputDefinitionOptions::default()
    });
    // Perl Package.pm L2715: after loading the alternate class binding, scan
    // the raw class file for \usepackage/\RequirePackage/\LoadClass — the
    // alternate rarely covers all dependencies the renamed class adds.
    // Run for BOTH a real prefix-match alternate AND the pure-OmniBus
    // fallback (the deps were deferred above so their require_package
    // calls fire AFTER OmniBus is in place).
    maybe_require_dependencies(name, "cls");
    return loaded;
  }
  result
}

/// configuration for searching for a file in the local filesystem
#[derive(Default)]
pub struct FindFileOptions {
  // TODO: this is no longer used in find_file, rather a level earlier
  pub forbid_ltxml:      bool,
  pub notex:             bool,
  pub ext_type:          Option<Cow<'static, str>>,
  pub search_paths_only: bool,
}

/// search for a file as prescribed by a `FindFileOptions` configuration
pub fn find_file(file: &str, options: Option<FindFileOptions>) -> Option<String> {
  let options = options.unwrap_or_default();
  if pathname::is_literaldata(file) {
    // If literal protocol return immediately (unless notex!)
    if options.notex {
      None
    } else {
      // TODO: Consider returning a Cow<str> instead to optimize
      Some(file.to_string())
    }
  } else if pathname::is_literaldata(file) || pathname::is_url(file) {
    // If a known special protocol return immediately
    Some(file.to_string())
  } else if let Some(ref ext) = options.ext_type {
    // Otherwise, it's some kind of "real" file, and we might have to search for it
    // Specific type requested? Search for it.
    // Add the extension, if it isn't already there.
    let aux_file = if file.ends_with(ext.as_ref()) {
      file.to_string()
    } else {
      s!("{}.{}", file, ext)
    };
    find_file_aux(&aux_file, &options)
  } else if file.ends_with(".tex") {
    // If no type given, we MAY expect .tex, or maybe NOT!!
    // No requested type, then .tex; Of course, it may already have it!
    find_file_aux(file, &options)
  } else {
    match find_file_aux(&s!("{}.tex", file), &options) {
      None => find_file_aux(file, &options),
      Some(f) => Some(f),
    }
  }
}

/// Perl Package.pm L2141-2210: FindFile_fallback
/// Pure-check variant of [`find_file_fallback`]: strips the same
/// suffix/prefix patterns and reports whether the fallback name has a
/// registered binding, but DOES NOT eagerly invoke the binding's
/// `load_definitions`. Use this from pre-flight existence checks (e.g.
/// `class_cls_via_fallback` in tex_job.rs) where firing the binding's
/// body has side effects (\LoadClass, \RequirePackage) that contaminate
/// the subsequent real load. Witness: astro-ph0002213 — the `\psfig`
/// cluster fix (mn1 → mn fallback was eagerly running mn.cls's
/// `\LoadClass{article}` before the `\documentstyle[epsfig]{mn1}`
/// option-pass-through machinery had a chance to enqueue `epsfig` into
/// `opt@article.cls`).
pub fn find_file_fallback_exists(name: &str, ext_type: &str) -> bool {
  use crate::state::binding_exists;
  use regex::Regex;
  // Mirror find_file_fallback's regex set exactly.
  let suffix_rx = match Regex::new(
    r"(?i)[._-](arx|arxiv|conference|workshop|tmp|alternate|preprint|fixed|[vV]?[-_.\d]+|old|new|final|clean|mine|priv|rev|mod|modified|edited|custom|altered|rtx)$",
  ) {
    Ok(rx) => rx,
    Err(_) => return false,
  };
  let glued_rx = match Regex::new(r"(?i)([vV]?[-_.\d]+|arxiv)$") {
    Ok(rx) => rx,
    Err(_) => return false,
  };
  let prefix_rx = match Regex::new(r"(?i)^((?:rw|my|preprint)[-_.]?)") {
    Ok(rx) => rx,
    Err(_) => return false,
  };
  let basename = pathname::file_name(name);
  let mut base = if basename.is_empty() {
    name.to_string()
  } else {
    basename
  };
  let mut changed = base != name;
  loop {
    if let Some(m) = suffix_rx.find(&base) {
      base = base[..m.start()].to_string();
      changed = true;
      continue;
    }
    if let Some(m) = glued_rx.find(&base) {
      base = base[..m.start()].to_string();
      changed = true;
      continue;
    }
    if let Some(m) = prefix_rx.find(&base) {
      base = base[m.end()..].to_string();
      changed = true;
      continue;
    }
    break;
  }
  if !changed || base.is_empty() || base == name {
    return false;
  }
  binding_exists(&base, ext_type)
}

/// Kind of fallback that matched, returned by [`find_file_fallback`].
///
/// `Versioned` — suffix/prefix stripping changed the basename (Perl
/// FindFile_fallback's core function). Drivers: 1206.0536 (mysvjour3
/// → svjour3), astro-ph0005021 (./aaspp4 → ./aaspp).
///
/// `BasenameOnly` — the *only* change was directory-prefix removal,
/// matching the binding registry by leaf name. This is a Rust-specific
/// extension on top of Perl's FindFile_fallback that exists because
/// our contrib-binding registry is keyed by basename. Drivers:
/// 2105.02087 (misc/ieeetran → IEEEtran binding); 2405.18387
/// (assets/equations → equations binding).
///
/// Both kinds win unconditionally over local raw `.sty`/`.cls` files
/// at the call site (see `input_definitions` Step 3): the `.rs`
/// bindings are hand-tuned for the conversion, so a binding match
/// always supersedes a co-located vendored copy. The variant is
/// preserved for diagnostics and potential future policy tweaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackKind {
  Versioned,
  BasenameOnly,
}

/// Strip version/arxiv suffixes from package names to find existing bindings.
/// Returns the fallback filename (with extension) and which kind of fallback fired.
pub fn find_file_fallback(name: &str, ext_type: &str) -> Option<(String, FallbackKind)> {
  use regex::Regex;
  // Suffixes with separator (Perl @find_fallback_suffixes)
  let suffix_rx = Regex::new(
    r"(?i)[._-](arx|arxiv|conference|workshop|tmp|alternate|preprint|fixed|[vV]?[-_.\d]+|old|new|final|clean|mine|priv|rev|mod|modified|edited|custom|altered|rtx)$"
  ).ok()?;
  // Glued suffixes without separator
  let glued_rx = Regex::new(r"(?i)([vV]?[-_.\d]+|arxiv)$").ok()?;
  // Prefixes. Perl Package.pm L2182: `^((?:rw|my|preprint)[-_.]?)` —
  // separator is OPTIONAL, so `mysvjour3` strips to `svjour3` (not just
  // `mysvjour`). Caught on arxiv 1206.0536 (\documentclass{mysvjour3}).
  let prefix_rx = Regex::new(r"(?i)^((?:rw|my|preprint)[-_.]?)").ok()?;

  // Strip a leading directory path (Perl Package.pm L2167-2170: FindFile_fallback
  // calls `pathname_name($name)` first, so e.g. `\documentclass{./sty/IEEEtran}`
  // routes the basename `IEEEtran` through the binding-name registry. Without
  // this, `IEEEtran.cls.ltxml` is missed because `./sty/IEEEtran.cls.ltxml`
  // never matches the @ltxml_paths registry. Driver paper: arXiv:1308.6663.
  let basename = pathname::file_name(name);
  let mut base = if basename.is_empty() {
    name.to_string()
  } else {
    basename
  };
  let dir_stripped = base != name;
  let mut suffix_stripped = false;
  // Iteratively strip suffixes, then glued, then prefixes.
  //
  // Perl's FindFile_fallback (Package.pm:2174) version-strips the FULL `$file`
  // *with its directory prefix intact*, so the prefix regex `^(rw|my|preprint)`
  // — anchored at the very START of the string — never matches a name that
  // begins with a directory (e.g. `sty/myunits` starts with `sty/`, not `my`).
  // We strip the directory FIRST (to allow a basename-exact binding match like
  // `misc/ieeetran` → IEEEtran), so we must NOT then apply the `^`-anchored
  // prefix strip to the basename, or `sty/myunits` wrongly becomes `units` and
  // loads the stock units.sty instead of the paper-local myunits.sty (which
  // defines `\T`/`\fC`/`\Cm` via its `\defUnit` mechanism). Witness 1702.05093.
  // The suffix/glued strips ARE `$`-anchored, so they still match the tail of a
  // dir-prefixed name in Perl (`./aaspp4` → `./aaspp`); keep applying them.
  loop {
    if let Some(m) = suffix_rx.find(&base) {
      base = base[..m.start()].to_string();
      suffix_stripped = true;
      continue;
    }
    if let Some(m) = glued_rx.find(&base) {
      base = base[..m.start()].to_string();
      suffix_stripped = true;
      continue;
    }
    if !dir_stripped {
      if let Some(m) = prefix_rx.find(&base) {
        base = base[m.end()..].to_string();
        suffix_stripped = true;
        continue;
      }
    }
    break;
  }

  if !suffix_stripped && !dir_stripped {
    return None;
  }
  if base.is_empty() || base == name {
    return None;
  }

  let kind = if suffix_stripped {
    FallbackKind::Versioned
  } else {
    FallbackKind::BasenameOnly
  };

  let fallback_filename = format!("{base}.{ext_type}");
  // Check if fallback binding exists
  if load_binding(&fallback_filename).unwrap_or(false) {
    // Binding exists but was loaded by the check — it's OK, the caller will mark loaded
    Some((fallback_filename, kind))
  } else if load_external_binding(&fallback_filename).unwrap_or(false) {
    Some((fallback_filename, kind))
  } else {
    None
  }
}

fn find_file_aux(file: &str, options: &FindFileOptions) -> Option<String> {
  // If cached, return simple path (it's a key into the cache)
  let cached = lookup_string(&s!("{}_contents", file));
  if !cached.is_empty() {
    return Some(file.to_string());
  }
  if pathname::is_absolute(file) {
    // Perl Package.pm L2089-2093:
    //   if pathname_is_absolute($file) {
    //     if (!$options{noltxml}) {
    //       return $file . '.ltxml' if -f ($file . '.ltxml'); }
    //     return $file if -f $file;
    //     return; }
    if !options.forbid_ltxml {
      let ltxml = s!("{}.ltxml", file);
      if Path::new(&ltxml).exists() {
        return Some(ltxml);
      }
    }
    if Path::new(file).exists() {
      Some(file.to_string())
    } else {
      None
    }
  } else if pathname::is_nasty(file) {
    // If it is a nasty filename, we won't touch it.
    // we DO NOT want to pass this to kpathse or such!
    None
  } else {
    // Note that the strategy is complicated by the fact that
    // (1) we prefer .ltxml bindings, if present
    // (2) those MAY be present in kpsewhich's DB (although our searchpaths take precedence!)
    // (3) BUT we want to avoid kpsewhich if we can, since it's slower
    // (4) depending on switches we may EXCLUDE .ltxml OR raw tex OR allow both.
    let paths: Vec<String> = get_search_paths();
    // let _urlbase = state!().lookup_value("URLBASE");
    // let _nopaths = lookup_bool("REMOTE_REQUEST");
    // let _ltxml_paths: Vec<String> = if nopaths { vec![] } else { paths.clone() };

    // Rust equivalent of Perl's ".ltxml" check: if the binding dispatcher
    // has an entry for this file, consider it "found". This is how Perl's
    // FindFile discovers pgfsys-latexml.def.ltxml etc.
    //
    // Two registry kinds are consulted (in order of cost):
    //  1. The per-call `{file}_binding_available` runtime flag, which packages can set to
    //     pre-announce their availability (used by pgf_sty for `pgfsys-latexml.def`).
    //  2. The compile-time class registry (latexml_package's `BINDINGS`) surfaced via
    //     `state::get_class_binding_names()`. Without this, `find_file("revtex4-1.cls",
    //     notex=true)` returned None for compiled-in bindings — so AIAA.cls's
    //     `\LoadClass{revtex4-1}` was silently skipped, breaking the eager natbib transitive load
    //     (1709.05096 / AIAA → 60s wall-clock SIGABRT in the autoload- trapped-by-abstract loop).
    // Binding-marker fast paths. ONLY fire when caller has requested
    // `notex=true` (i.e. caller wants binding-only search, not a real
    // disk path). Without this gate, raw `\openin` /`\IfFileExists`
    // calls (notex=false) get a literal binding name back as if it were
    // a path — `Mouth::open_file` then fails / produces an empty mouth
    // and `\ifeof` returns true, masking the file as missing. Mirrors
    // Perl `pathname_find`: only `noltxml=>0,notex=>1` returns binding
    // names; the disk-search variant only resolves real files.
    // Triggered by 2026-04-26 t1enc.def-cascade investigation: raw
    // fonttext.ltx's `\input  {t1enc.def}` opens via raw `\openin` /
    // `\IfFileExists`; without this gate find_file returned literal
    // "t1enc.def" → empty mouth → kernel's `\@missingfileerror` → 1M
    // TooManyErrors during latex.ltx dump-build.
    if !options.forbid_ltxml && options.notex {
      if lookup_bool(&s!("{file}_binding_available")) {
        return Some(file.to_string());
      }
      // Check the compile-time binding registries from latexml_package and
      // latexml_contrib for ANY (name, ext) pair that matches `file` (split
      // on the FIRST `.`, mirroring `dispatch()`'s split rule so multi-dot
      // names like `pgfmath.code.tex` resolve as `("pgfmath", "code.tex")`).
      if let Some((base, ext)) = file.split_once('.') {
        // Perl pathname_find L383-389: strict-case first, then case-insensitive
        // fallback (mirrors the dispatcher's lookup). Without this, requests
        // like `find_file("jhep.cls", notex=true)` would miss `("JHEP","cls")`
        // entries that derive from Perl's `JHEP.cls.ltxml` filename.
        let exact = crate::state::get_binding_names()
          .iter()
          .any(|slice| slice.iter().any(|(n, e)| *n == base && *e == ext));
        let nocase = exact
          || crate::state::get_binding_names().iter().any(|slice| {
            slice
              .iter()
              .any(|(n, e)| n.eq_ignore_ascii_case(base) && e.eq_ignore_ascii_case(ext))
          });
        if nocase {
          return Some(file.to_string());
        }
      }
    } else if !options.forbid_ltxml {
      // Narrow notex=false (disk-search) fallback: ONLY honor explicit
      // `<file>_binding_available` runtime flags, NOT the broad compile-time
      // registry. Mirrors Perl `\openin` calling default-args FindFile —
      // see `TeX_FileIO.pool.ltxml:50-64` "we SHOULD find an .ltxml version!"
      //
      // Use case: pgf.sty's pgfsys.code.tex `\pgfutil@InputIfFileExists{\pgfsysdriver}`
      // → `\pgfutil@IfFileExists{pgfsys-latexml.def}` → `\openin` with notex=false.
      // The openin impl in tex_file_io.rs creates an empty Mouth on
      // Mouth::create failure, so \ifeof=false → pgf inputs the driver.
      // pgf_sty.rs sets `pgfsys-latexml.def_binding_available=true` to
      // enable this; without the flag we don't fake-find arbitrary binding
      // names (which would re-introduce the t1enc.def `\@missingfileerror`
      // cascade documented above the notex=true branch).
      if lookup_bool(&s!("{file}_binding_available")) {
        return Some(file.to_string());
      }
    }
    // Perl L2123-2125: `elsif !notex && !interpreting && pathname_find($file,
    // paths=>$paths)` — search local paths for raw TeX.
    // (Rust does not yet honour `INTERPRETING_DEFINITIONS` — minor TODO,
    //  acknowledged in the audit.)
    if !options.notex {
      if let Some(path) = pathname::find(file, PathnameFindOptions {
        paths: Some(paths),
        ..PathnameFindOptions::default()
      }) {
        return Some(path);
      }
    }
    // Perl L2131-2136: build kpsewhich candidate list:
    //   @candidates = ( "$file.ltxml" if !noltxml && !nopaths,
    //                   $file        if !notex );
    //   if (!searchpaths_only) && pathname_kpsewhich(@candidates) → -f $result
    // Perl gates the kpsewhich call only on `!searchpaths_only`; `notex`
    // and `noltxml` instead control which candidate names are tried.
    if options.search_paths_only {
      return None;
    }
    let mut candidates: Vec<String> = Vec::new();
    if !options.forbid_ltxml {
      // Perl `!nopaths` (REMOTE_REQUEST) gate not yet modeled in Rust;
      // we always include the .ltxml candidate when ltxml is allowed.
      candidates.push(s!("{}.ltxml", file));
    }
    if !options.notex {
      candidates.push(file.to_string());
    }
    if candidates.is_empty() {
      return None;
    }
    let refs: Vec<&str> = candidates.iter().map(|s| s.as_str()).collect();
    match pathname::kpsewhich(&refs) {
      // Perl L2136: `(-f $result ? $result : undef)` — re-confirm existence.
      Some(p) if Path::new(&p).exists() => Some(p),
      _ => None,
    }
  }
}

//======================================================================
// Declaring and Adjusting the Document Model.
//======================================================================

pub fn install_tag(tag: &str, mut properties: TagOptions) {
  let tag_ticket = arena::pin(tag);
  with_tag_property_mut(tag_ticket, |options| {
    if properties.auto_open.is_some() {
      options.auto_open = properties.auto_open;
    }
    if properties.auto_close.is_some() {
      options.auto_close = properties.auto_close;
    }
    for name in &TagOptionName::all() {
      if name.is_prepend() {
        options.prepend(name, properties.remove(name));
      } else if name.is_append() {
        options.append(name, properties.remove(name));
      } else {
        // we'll handle the regular ones out of the loop
      }
    }
  });
}

/// Selects the RelaxNG schema defining the XML output language
pub fn select_relaxng_schema(schema: &str, namespaces: Option<HashMap<String, String>>) {
  // What verb here? Set, Choose,...
  model::set_relaxng_schema(schema);
  if let Some(namespaces) = namespaces {
    for (prefix, value) in namespaces {
      model::register_document_namespace(&prefix, Some(&value));
    }
  }
}

pub fn merge_font(font: Font) {
  let new_font = lookup_font().unwrap().merge_ref(&font);
  assign_font(Rc::new(new_font), Some(Scope::Local));
}

/// Like `merge_font` but borrows the font. Saves a clone when the caller
/// has a shared reference (e.g. via Rc) to the font being merged.
pub fn merge_font_ref(font: &Font) {
  let new_font = lookup_font().unwrap().merge_ref(font);
  assign_font(Rc::new(new_font), Some(Scope::Local));
}

/// Define a named color (Perl: DefColor).
/// Stores as color_{name} and also defines \\color@{name} macro.
pub fn def_color(
  name: &str,
  color: &crate::common::color::Color,
  scope: Option<Scope>,
) -> Result<()> {
  use crate::common::color;
  // Check ifglobalcolors — Perl: $scope='global' if lookupDefinition(\ifglobalcolors) &&
  // IfCondition(\ifglobalcolors) Guard with lookup first: xcolor may not be loaded (e.g.
  // colordvi-only documents).
  //
  // ALSO force global when `color_force_global` is set: color_sty.rs's lazy
  // dvipsnam.def loader (recovering a dropped `\usepackage[dvipsnames]{color}`
  // option, e.g. when hyperref preloaded `color` first) fires from WITHIN a
  // grouped digestion (`\textcolor{Blue}{…}` / a listings keywordstyle), so the
  // 68 dvips colors would otherwise be defined in that local group and revert
  // before the next `\color{…}` — making only the FIRST color resolve. Perl
  // loads dvipsnam at the preamble (top level), so its colors persist; the flag
  // reproduces that global effect. Witness 1705.06183.
  let effective_scope = if (lookup_definition(&T_CS!("\\ifglobalcolors"))?.is_some()
    && if_condition(&T_CS!("\\ifglobalcolors"))? == Some(true))
    || lookup_bool("color_force_global")
  {
    Some(Scope::Global)
  } else {
    scope
  };
  // Store in state as "model c1 c2 ..."
  let stored = color.to_stored();
  assign_value(
    &s!("color_{name}"),
    Stored::String(arena::pin(stored)),
    effective_scope,
  );
  // Define \\color@{name} macro for reversion
  // Perl: DefMacroI('\\color@'.$name, undef,
  //   '\relax\relax{model spec}{model}{spec_commas}')
  let model = color.model();
  let comps = color.components();
  let spec_parts: Vec<String> = comps.iter().map(|c| color::format_component(*c)).collect();
  let spec_space = spec_parts.join(" ");
  let spec_comma = spec_parts.join(",");
  let model_spec = s!("\\relax\\relax{{{model} {spec_space}}}{{{model}}}{{{spec_comma}}}");
  def_macro(
    T_CS!(s!("\\\\color@{name}")),
    None,
    crate::mouth::tokenize_internal(&model_spec),
    Some(crate::definition::expandable::ExpandableOptions {
      scope: effective_scope,
      ..Default::default()
    }),
  )?;
  Ok(())
}

/// Define a derived color model (Perl: DefColorModel).
/// Stores conversion functions for a derived color model.
pub fn def_color_model(model: &str, coremodel: &str) {
  assign_value(
    &s!("derived_color_model_{model}"),
    Stored::String(arena::pin(coremodel)),
    Some(Scope::Global),
  );
}

pub fn digest_text(stuff: Tokens) -> Result<Digested> {
  begin_mode("text")?;
  let value = digest(stuff);
  end_mode("text")?;
  value
}

pub fn digest_literal<T: Into<Tokens>>(stuff: T) -> Result<Digested> {
  let stuff: Tokens = stuff.into();
  // Perhaps should do StartSemiverbatim, but is it safe to push a frame? (we might cover over
  // valid changes of state!)
  begin_mode("text")?;

  // Fall back to the global text default if no font is currently assigned —
  // avoids a panic at this hot path (called from e.g. RefStepID / label
  // digestion) when the state's "font" slot hasn't been initialised. Matches
  // the same fallback `assign_value("font", Font::text_default(), …)` that
  // stomach::init uses at startup.
  let font = lookup_font().unwrap_or_else(|| Rc::new(crate::common::font::Font::text_default()));
  assign_font(
    Rc::new(font.merge(fontmap!(encoding => "ASCII"))),
    Some(Scope::Local),
  ); // try to stay as ASCII as possible

  let value = digest(stuff);
  assign_font(font, None);
  end_mode("text")?;
  value
}

pub fn digest_if(token: Token) -> Result<Option<Digested>> {
  if lookup_definition(&token)?.is_some() {
    match digest(Tokens!(token)) {
      Ok(t) => Ok(Some(t)),
      Err(e) => Err(e),
    }
  } else {
    Ok(None)
  }
}

/// Test a conditional `\ifXXX` and return its boolean result (Perl: IfCondition).
/// Looks up the conditional's test closure and invokes it.
pub fn if_condition(if_token: &Token) -> Result<Option<bool>> {
  use crate::definition::conditional::ConditionalType;
  if let Some(defn) = lookup_definition(if_token)? {
    if defn.get_conditional_type() == Some(ConditionalType::If) {
      if let Some(test) = defn.get_test() {
        // Read arguments for the conditional test
        let args = match defn.get_parameters() {
          Some(params) => params.read_arguments(Some(defn.as_ref()))?,
          None => Vec::new(),
        };
        return Ok(Some(test(args)?));
      }
    }
  }
  if x_equals(if_token, &T_CS!("\\iftrue")) {
    return Ok(Some(true));
  }
  if x_equals(if_token, &T_CS!("\\iffalse")) {
    return Ok(Some(false));
  }
  Error!(
    "expected",
    "conditional",
    s!("Expected a conditional, got '{}'", if_token.stringify())
  );
  Ok(None)
}

/// Set the boolean value of a `\newif`-type conditional (Perl: SetCondition).
/// This is only for simple conditionals taking no arguments.
pub fn set_condition(if_token: &Token, value: bool, scope: Option<Scope>) {
  if let Ok(Some(defn)) = lookup_definition(if_token) {
    if defn.get_parameters().is_none() {
      let target = if value {
        T_CS!("\\iftrue")
      } else {
        T_CS!("\\iffalse")
      };
      let_i(if_token, &target, scope);
      return;
    }
  }
  log::error!(
    "Expected a conditional defined by \\newif, got '{}'",
    if_token.stringify()
  );
}

/// Creates a single `Tokens` representing a TeX invocation of the lead `token` over a list of
/// arguments.
///
/// Note: currently this is near the `Mouth` representation of data, and deals purely with tokens.
/// A more generic version of this method may be able to support `ArgWrap` for the argument list.
/// Indeed, the return type may also be lifted to a new generic ArgWrap::Invocation, if there was
/// benefit.
pub fn build_invocation<T: Into<Token>>(token: T, args: Vec<Option<Tokens>>) -> Result<Tokens> {
  let token: Token = token.into();
  build_invocation_token(token, args)
}

/// String entry point for `Invocation` (Perl: `Invocation($token,@args)` with a string
/// first argument). The string is tokenized via `TokenizeInternal`; if it yields a single
/// token, this reduces to `build_invocation` on that token. Otherwise the tokens are
/// treated as an "anonymous macro" containing parameter markers like `#1`, and the
/// arguments are substituted in.
pub fn build_invocation_str(spec: &str, args: Vec<Option<Tokens>>) -> Result<Tokens> {
  let tokens = crate::mouth::tokenize_internal(spec);
  let mut list = tokens.unlist();
  if list.len() > 1 {
    // Treat as anonymous macro
    let cow_args: Vec<Option<Cow<Tokens>>> = args.into_iter().map(|a| a.map(Cow::Owned)).collect();
    return Ok(
      Tokens::new(list)
        .pack_parameters()?
        .substitute_parameters(&cow_args),
    );
  }
  match list.pop() {
    Some(cs) => build_invocation_token(cs, args), // reduce to single token.
    None => {
      Error!(
        "unexpected",
        "invocation",
        s!("Can't invoke empty token spec '{}'", spec)
      );
      Ok(Tokens::default())
    },
  }
}

fn build_invocation_token(token: Token, args: Vec<Option<Tokens>>) -> Result<Tokens> {
  // Note: token may have been \let to another defn!
  if let Some(defn) = lookup_definition(&token)? {
    let mut invoked_tokens = vec![token];
    if let Some(params) = defn.get_parameters() {
      invoked_tokens.extend(params.revert_arguments(args)?);
    }
    Ok(Tokens::new(invoked_tokens))
  } else {
    let message = s!("Can't invoke {:?}; it is undefined", token.stringify());
    token.with_cs_name(|csname| {
      Error!("undefined", csname, message);
      Ok(())
    })?;
    let mut invoked_tokens = vec![token];
    // DefConstructor!(token, convert_latex_args(args.len(), 0),
    // sub { LaTeXML::Core::Stomach::makeError($_[0], 'undefined', token); });
    let wrapped_args: Vec<Token> = args
      .into_iter()
      .flat_map(|arg_opt| {
        let mut wrapped = vec![T_BEGIN!()];
        if let Some(arg) = arg_opt {
          wrapped.extend(arg.unlist());
        }
        wrapped.push(T_END!());
        wrapped
      })
      .collect();
    invoked_tokens.extend(wrapped_args);
    Ok(Tokens::new(invoked_tokens))
  }
}

/// Convert a LaTeX-style argument spec to our Package form.
/// Ie. given $nargs and $optional, being the two optional arguments to
/// something like \newcommand, convert it to the form we use
pub fn convert_latex_args(
  mut nargs: usize,
  optional: Option<Tokens>,
) -> Result<Option<Parameters>> {
  let mut params = Vec::new();
  if let Some(tks) = optional {
    params.push(
      Parameter {
        name: arena::pin_static("Optional"),
        spec: arena::pin(s!("[Default:{}]", tks.clone().untex())),
        extra: vec![tks],
        ..Parameter::default()
      }
      .init()?,
    );
    nargs -= 1;
  }

  for _ in 1..=nargs {
    params.push(
      Parameter {
        name: arena::pin_static("Plain"),
        spec: arena::pin_static("{}"),
        ..Parameter::default()
      }
      .init()?,
    );
  }
  if params.is_empty() {
    Ok(None)
  } else {
    Ok(Some(Parameters::new(params)))
  }
}

/// Two-optional variant of `convert_latex_args` — mirrors Perl's
/// `convert2optArgs` helper in twoopt.sty.ltxml. `\newcommandtwoopt{\cs}[n][d1][d2]{…}`
/// builds a signature of `[Default d1][Default d2]{…}…` where the remaining
/// `n - 2` args are plain required.
pub fn convert_twoopt_args(
  mut nargs: usize,
  opt1: Option<Tokens>,
  opt2: Option<Tokens>,
) -> Result<Option<Parameters>> {
  let mut params = Vec::new();
  if let Some(tks) = opt1 {
    params.push(
      Parameter {
        name: arena::pin_static("Optional"),
        spec: arena::pin(s!("[Default:{}]", tks.clone().untex())),
        extra: vec![tks],
        ..Parameter::default()
      }
      .init()?,
    );
    nargs = nargs.saturating_sub(1);
  }
  if let Some(tks) = opt2 {
    params.push(
      Parameter {
        name: arena::pin_static("Optional"),
        spec: arena::pin(s!("[Default:{}]", tks.clone().untex())),
        extra: vec![tks],
        ..Parameter::default()
      }
      .init()?,
    );
    nargs = nargs.saturating_sub(1);
  }
  for _ in 1..=nargs {
    params.push(
      Parameter {
        name: arena::pin_static("Plain"),
        spec: arena::pin_static("{}"),
        ..Parameter::default()
      }
      .init()?,
    );
  }
  if params.is_empty() {
    Ok(None)
  } else {
    Ok(Some(Parameters::new(params)))
  }
}

/// Decode a codepoint using the fontmap for a given font and/or encoding (Perl: FontDecode).
/// Returns the decoded glyph (if any) and the possibly-adjusted font.
pub fn font_decode(
  code: i32,
  encoding_opt: Option<&str>,
  font_opt: Option<Rc<Font>>,
) -> (Option<char>, Option<Rc<Font>>) {
  if code < 0 {
    return (None, font_opt);
  }
  let font = font_opt.unwrap_or_else(|| lookup_font().unwrap());
  let encoding = match encoding_opt {
    Some(enc) => enc.to_string(),
    None => font
      .get_encoding()
      .map_or_else(|| "OT1".to_string(), |c| c.to_string()),
  };
  let map = load_font_map(&encoding);
  // Check for family-specific map. Use with_value to avoid cloning the
  // Stored envelope when the From<&Stored>→Option<Fontmap> impl only
  // needs the enum variant discriminant + an Rc bump.
  let (effective_map, _effective_enc) = if let Some(family) = font.get_family() {
    let fam_key = s!("{}_{}_fontmap", encoding, family);
    let fam_map: Option<Fontmap> = with_value(&fam_key, |v| v.and_then(|s| s.into()));
    if let Some(fm) = fam_map {
      (Some(fm), s!("{}_{}", encoding, family))
    } else {
      (map, encoding)
    }
  } else {
    (map, encoding)
  };
  let glyph = effective_map
    .as_ref()
    .and_then(|m| m.get(code as usize).copied().flatten());
  // In-math alphanumeric mathstyle handling is done in decodeMathChar instead
  (glyph, Some(font))
}

/// Decode a string using the fontmap for a given encoding (Perl: FontDecodeString).
/// If `implicit` is true, codepoints missing from the map decode to themselves.
pub fn font_decode_string(string: &str, encoding_opt: Option<&str>, implicit: bool) -> String {
  let font = lookup_font().unwrap();
  let encoding = match encoding_opt {
    Some(enc) => enc.to_string(),
    None => font
      .get_encoding()
      .map_or_else(|| "OT1".to_string(), |c| c.to_string()),
  };
  let map = load_font_map(&encoding);
  // Check for family-specific map — same with_value motivation as above.
  let effective_map = if let Some(family) = font.get_family() {
    let fam_key = s!("{}_{}_fontmap", encoding, family);
    let fam_map: Option<Fontmap> = with_value(&fam_key, |v| v.and_then(|s| s.into()));
    fam_map.or(map)
  } else {
    map
  };
  let input_enc = lookup_string("INPUT_ENCODING");
  let map_max: usize = if input_enc == "utf8" { 128 } else { 256 };
  // Also limit for short font maps
  let map_max = if let Some(ref m) = effective_map {
    if m.len() < map_max { m.len() } else { map_max }
  } else {
    map_max
  };

  let mut result = String::new();
  for ch in string.chars() {
    let code = ch as usize;
    if implicit {
      if let Some(ref m) = effective_map {
        if code < map_max {
          if let Some(Some(glyph)) = m.get(code) {
            result.push(*glyph);
          }
        } else {
          result.push(ch);
        }
      } else {
        result.push(ch);
      }
    } else if let Some(ref m) = effective_map {
      if let Some(Some(glyph)) = m.get(code) {
        result.push(*glyph);
      }
    }
  }
  result
}

pub fn load_font_map(encoding: &str) -> Option<Fontmap> {
  let _ = preload_font_map(encoding); // infallible in practice; swallow Result
  // with_value avoids the Stored::clone; the Fontmap extraction is a cheap
  // Rc bump on the inner slice regardless.
  with_value(&s!("{encoding}_fontmap"), |v| v.and_then(|s| s.into()))
}
pub fn preload_font_map(encoding: &str) -> Result<()> {
  // This check is done as a "preload" step for mutability reasons.
  let key = s!("{encoding}_fontmap");
  if has_value(&key) {
    return Ok(());
  }
  let fail_key = s!("{encoding}_fontmap_failed_to_load");
  let failed_flag = lookup_bool(&fail_key);
  if !failed_flag {
    assign_value(&fail_key, true, None); // Stop recursion?
    let _ = input_definitions(&encoding.to_lowercase(), InputDefinitionOptions {
      extension: Some(Cow::Borrowed("fontmap")),
      noerror: true,
      ..InputDefinitionOptions::default()
    });
    if has_value(&s!("{encoding}_fontmap")) {
      // Got map?
      assign_value(&fail_key, false, None);
    } else {
      assign_value(&fail_key, true, Some(Scope::Global));
    }
  }
  Ok(())
}
