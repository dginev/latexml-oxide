//! `latex_constructs` section 1: C.1 Commands and Environments
//!
//! Split verbatim from the single `LoadDefinitions!` body (2026-09-03); the
//! definitions run in the original order from `super::load_definitions`.

use super::*;

#[rustfmt::skip]
/// OXIDIZED_DESIGN #179, the "has chapters" step: after a class has loaded,
/// the kernel-level `\chapter` (Perl pool:557, locked) is retracted unless
/// the class made a chapter counter (Perl's own probe, pool:690) or the class
/// is the OmniBus fallback. Runs at `\documentclass` AND at `\LoadClass`
/// return (sect05.rs): a class body's probe `\@ifundefined{chapter}` after
/// its `\LoadClass{article}` (aguplus.cls:524 "figcaps may only be used with
/// article-like classes") must already see article's answer, as in LaTeX
/// where article never defines one. The lock is released with the
/// definition so a class that builds its own chapters afterwards
/// (source3body.tex:100-123 `\newcounter{chapter}` + `\newcommand\chapter`)
/// is not "Ignoring redefinition of \chapter" (KNOWN_PERL_ERRORS #141).
/// Guard: `perfect_kernel_batch56::loadclass_return_retracts_kernel_chapter`.
pub(crate) fn retract_kernel_chapter_if_chapterless() -> Result<()> {
  if lookup_definition(&T_CS!("\\c@chapter"))?.is_none()
    && !lookup_bool("OmniBus.cls_loaded")
    && !lookup_bool("OmniBus.cls.ltxml_loaded")
  {
    let_i(&T_CS!("\\chapter"), &T_CS!("\\@undefined"), Some(Scope::Global));
    assign_value("\\chapter:locked", false, Some(Scope::Global));
  }
  Ok(())
}

pub(crate) fn load() -> Result<()> {
  // ======================================================================
  // C.1 Commands and Environments
  // ======================================================================

  // Perl latex_constructs.pool.ltxml L45-48 — page counter + early Lets.
  // These are redundant with `NewCounter!("page")` later in the file
  // (L3791) and the `Let!("\\nobreakspace", ...)` later (L4723), but
  // Perl-faithful: Perl runs them here in the early "C.1 Commands and
  // Environments" block so any subsequent dump-load (which writes via
  // `assign_internal` and bypasses :locked) cannot leave these CSes
  // pointing at raw plain.tex bodies.
  Let!("\\nobreakspace", "\\lx@nobreakspace");

  //======================================================================
  // Just to pass test t/alignment/halignatt
  // Perl latex_constructs.pool.ltxml L51 — redundant with plain_base.rs
  // (Perl plain_base.pool.ltxml L147), but Perl-faithful.
  def_macro_noop("\\hidewidth")?;

  // Perl-faithful: LaTeX does NOT define `\magnification`, and babel
  // uses this to detect whether it's running under LaTeX (vs plain
  // TeX). The unconditional Let here would also kill plain-TeX papers
  // whose first line is `\magnification=\magstep1` — fine in Perl
  // because the Perl driver doesn't pre-load latex_constructs for plain
  // TeX sources, but the Rust cortex_worker eagerly preloads
  // `LaTeX.pool` for the `\UseRawInputEncoding`-before-`\documentclass`
  // case (2407.00348). To keep both working, defer the Let until
  // `\documentclass` actually fires (see `\documentclass`
  // after_digest below). Witness: 15 wp4 papers like 2305.09030 ship
  // `\magnification=\magstep1` and previously failed with
  // `Error:undefined:\magnification` under the worker.
  //
  // `latex_kernel::autoload_latex_kernel` is a second route into the same
  // state (LaTeX.pool loaded in a plain-TeX document), so the deferral is
  // load-bearing there too. It cannot fire ON `\magnification` itself:
  // latex.ltx does not define it, so it is absent from the kernel dump that
  // the autoload's membership test consults — checked when that hook landed.
  Let!("\\@empty", "\\lx@empty");
  Let!("\\@ifundefined", "\\lx@ifundefined");
  //**********************************************************************
  // Basic \documentclass & \documentstyle

  DefConditional!("\\if@compatibility", { lookup_bool("2.09_COMPATIBILITY") });
  def_macro_noop("\\@compatibilitytrue")?;
  def_macro_noop("\\@compatibilityfalse")?;

  Let!("\\@currentlabel", "\\@empty");
  DefMacro!("\\@currdir", "./");

  // Defensive `\let \@halignto \@empty` for the kernel macro that's
  // initialized inline inside \tabular / \tabular* / \array setup
  // (`\let\@halignto\@empty` at the top of each), but referenced
  // unprotected inside \edef expansions. If the inline init doesn't
  // fire before the edef-referencing macro runs (e.g. `\@array`
  // called outside a tabular context), the reference becomes
  // undefined and trips `Error:undefined:\\@halignto` cascade.
  // Witness: 2306.10481 (4 errors), 2307.05820 (22 errors), plus
  // 2 other stage-2 papers using \begin{array} outside standard wrapping.
  Let!("\\@halignto", "\\@empty");

  // Defensive `\let \@arrayright \@empty` for array.sty's macro
  // (array.sty L: `\let\@arrayright\@empty`). Referenced by revtex's
  // ltxutil.sty (`\def\endarray{...\@arrayright...}`) BEFORE array.sty
  // has loaded. Without this default, raw-loading ltxutil while in
  // revtex's class init reads \@arrayright as undefined, defines it
  // as <ltx:ERROR/>, and the resulting bad <ltx:ERROR/> token propagates
  // through `\endarray` causing 60GB+ OOM cascades. Witness 2305.18141.
  Let!("\\@arrayright", "\\@empty");
  AssignValue!("inPreamble", true); // \begin{document} will clear this.

  DefConstructor!("\\documentclass PackageOptions SkipSpaces ExpandedSemiverbatim []",
                  "<?latexml class='#2' ?#options(options='#options')?>",
    properties => sub[args] { Ok(sect05::options_pi_spelling(args[0].as_ref())) },
    after_digest => sub[whatsit] {
      // Now that we know we're a LaTeX document, undefine `\magnification`
      // (babel's plain-TeX vs LaTeX discriminator). Deferred from the
      // pool-load site above so plain-TeX papers that never run
      // `\documentclass` keep `\magnification` as a usable register.
      if let Some(undef_meaning) = lookup_meaning(&T_CS!("\\@undefined")) {
        assign_meaning(&T_CS!("\\magnification"), undef_meaning, Some(Scope::Global));
      }
      // Revert the digested option list rather than `to_string` it: the
      // digested form drops the braces of a braced key value, so
      // `thesis={type=dr,dr=rernat}` (DEMO-TUDaPhD) came out as
      // `thesis=type=dr,dr=rernat` — split at its inner comma and, once
      // recorded in `\@raw@classoptionslist`, mis-parsed by l3keys.
      let options: Option<&Digested> = whatsit.get_arg(1);
      let class_opts = match options {
        Some(opts) => split_trim_options(&sect05::protected_xdef_options(opts.revert()?)?.untex()),
        None => Vec::new(),
      };
      // Perl LaTeX.pool.ltxml:57 — `$class =~ s/\s+//g;`. Strip ALL
      // whitespace from the class name before LoadClass. A multi-line
      // `\documentclass[...]{<newline>revtex4}` makes the Semiverbatim arg
      // ` revtex4` (the newline right after `{` becomes a leading space
      // token); unstripped, the name fails to match the `revtex4` binding
      // registration and falls through to OmniBus — which lacks revtex4's
      // `\email [] Semiverbatim`, so an `\email{a_b@…}` with `_` then errors
      // "Script _ can only appear in math mode". Witness 1601.06734.
      let class_name: String = whatsit
        .get_arg(2)
        .unwrap()
        .to_string()
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect();
      load_class(&class_name,
                class_opts,
                Tokens!(T_CS!("\\AtBeginDocument"), T_CS!("\\warn@unusedclassoptions")))?;
      // OXIDIZED_DESIGN #179: `\chapter` exists only where the class made a
      // chapter counter. latex.ltx defines no `\chapter`; article/scrartcl
      // never do, so `\@ifundefined{chapter}` is how packages decide
      // (blindtext.sty:243 `\blinddocument`, hvfloat's 50 test docs,
      // coseoul, xassoccnt: the kernel-level `\chapter` — Perl pool:557,
      // locked — took the chapter branch and every doc errored
      // `undefined:\thechapter`). The `\c@chapter` probe is Perl's own
      // "has chapters" test (pool:690). An unknown class (OmniBus fallback)
      // may well have chapters — OmniBus autoloads book.cls on `\thechapter`
      // (arXiv:2602.10407) — so it keeps the kernel `\chapter`.
      retract_kernel_chapter_if_chapterless()?;
      Ok(())
  });

  AssignValue!("@unusedoptionlist", Stored::Strings(Rc::new([])));
  // The group-local record of `\begin{document}`'s decision (divergence
  // #232, see `\begin`/`\end` below): `\lx@document@indirected` runs inside
  // the `\begingroup` a renewed `document` gets, `\lx@document@direct` inside
  // the constructor's group when the original `\document` ran.
  DefPrimitive!("\\lx@document@indirected", {
    assign_value("document:indirected", Stored::Bool(true), None);
    Ok(Vec::new())
  });
  DefPrimitive!("\\lx@document@direct", {
    assign_value("document:indirected", Stored::Bool(false), None);
    Ok(Vec::new())
  });
  DefPrimitive!("\\warn@unusedclassoptions", {
    if let Some(Stored::Strings(unused)) = lookup_value("@unusedoptionlist")
      && !unused.is_empty()
    {
      Info!(
        "unexpected",
        "options",
        "Unused global options: {}",
        with_many(&unused, |u| u.join(","))
      );
      assign_value("@unusedoptionlist", Stored::Strings(Rc::new([])), None);
    }
  });

  // Perl latex_constructs.pool.ltxml:137-154:
  //   DefPrimitiveI('\compat@loadpackages', undef, sub {
  //       my $hadmissing = 0;
  //       foreach my $option (@{ LookupValue('@unusedoptionlist') }) {
  //         if (FindFile($option, type => 'sty')) { RequirePackage($option); }
  //         else { $hadmissing = 1; Info('unexpected', $option, ...); } }
  //       if ($hadmissing && !LookupValue('OmniBus.cls_loaded')) {
  //         Info('note', 'OmniBus', ...); LoadClass('OmniBus'); }
  //       AssignValue('@unusedoptionlist', []); });
  //
  // Scheduled via `after => Tokens(T_CS('\compat@loadpackages'))` when the
  // LaTeX-2.09-compat \documentstyle finishes its class load. Consumes the
  // unused options that the class (e.g. article.cls) didn't recognise and
  // routes each to \RequirePackage via Rust's `require_package` sub — which
  // includes `find_file_fallback` (version-suffix stripping, e.g.
  // `aaspp4` → `aaspp.sty.ltxml`). This is what lets
  // `\documentstyle[aaspp4]{article}` load aas_support transitively and
  // define \affil / \altaffilmark / \acknowledgments etc. that ~49 astro-ph
  // papers in the 10k sandbox need (docs/archive/SANDBOX_TRIAGE_2026-05-21.md Class A).
  //
  // Current \documentstyle implementation lives in tex_job.rs as a DefMacro
  // whose body mirrors Perl's three afterDigest branches. It no longer emits
  // one `\RequirePackage` per option inline; this primitive is the route for
  // class-unconsumed options after `\ProcessOptions`.
  DefPrimitive!("\\compat@loadpackages", {
    use latexml_core::binding::content::{find_file, find_file_fallback};
    let unused_list: Vec<String> = match lookup_value("@unusedoptionlist") {
      // `\OptionNotUsed` uses `state::push_value` which converts `Strings`
      // → `VecDequeStored` on first push. Either form may be live here.
      Some(Stored::Strings(rc)) => rc.iter().map(|s| with(*s, |s| s.to_string())).collect(),
      Some(Stored::VecDequeStored(vdq)) => vdq
        .iter()
        .filter_map(|item| match item {
          Stored::String(s) => Some(with(*s, |s| s.to_string())),
          _ => None,
        })
        .collect(),
      _ => Vec::new(),
    };
    let mut had_missing = false;
    for opt in &unused_list {
      // Perl `FindFile($option, type=>'sty')` defaults to consulting the
      // binding-registry AND disk SEARCHPATHS. Use TWO calls: first
      // `notex: true` so compiled-in Rust bindings (e.g. psfig_sty.rs,
      // when paspconf-class doc loads `[psfig]` as an unused option)
      // are considered "found" and `RequirePackage` is fired. Then a
      // disk-only call (notex: false, ext_type: "sty") to pick up
      // paper-local `<opt>.sty` files (e.g. `ysc.sty` in an arXiv
      // submission shipping its own class-option). Without the second
      // call, ~50 astro-ph papers errored on `\plotone`/`\plottwo` etc.
      // even though the paper-local sty defines them. Min repro:
      // `\documentstyle[mysty]{article}` with local mysty.sty.
      use latexml_core::binding::content::FindFileOptions;
      let found_binding = find_file(
        &format!("{opt}.sty"),
        Some(FindFileOptions {
          notex: true,
          ..Default::default()
        }),
      )
      .is_some();
      let found_fallback = !found_binding && find_file_fallback(opt, "sty").is_some();
      let found_disk = !found_binding
        && !found_fallback
        && find_file(
          opt,
          Some(FindFileOptions {
            ext_type: Some("sty".into()),
            forbid_ltxml: true,
            ..Default::default()
          }),
        )
        .is_some();
      if found_binding || found_fallback || found_disk {
        // When the file was found ONLY via paper-local disk-probe (no
        // .sty.ltxml binding and no version-strip fallback), we must
        // explicitly enable raw TeX loading because the default
        // `INCLUDE_STYLES=false` gate inside `require_package` would
        // otherwise force `notex=true` and suppress the raw load. Without
        // this, `\documentstyle[<opt>]{<class>}` with paper-local
        // `<opt>.sty` (e.g. `newpasp.sty` in astro-ph0009248) fired
        // RequirePackage but never actually loaded, leaving `\affil`,
        // `\references`, etc. undefined.
        let opts = if found_disk {
          RequireOptions {
            notex: Some(false),
            ..RequireOptions::default()
          }
        } else {
          RequireOptions::default()
        };
        require_package(opt, opts)?;
      } else {
        had_missing = true;
        Info!(
          "unexpected",
          opt,
          "Unexpected option '{}' passed via \\documentstyle",
          opt
        );
      }
    }
    if had_missing && !lookup_bool("OmniBus.cls_loaded") {
      Info!(
        "note",
        "OmniBus",
        "Loading OmniBus class to attempt to cover missing options"
      );
      load_class("OmniBus", Vec::new(), Tokens!())?;
    }
    assign_value(
      "@unusedoptionlist",
      Stored::Strings(Rc::new([])),
      Some(Scope::Global),
    );
  });

  // onlyPreamble (Perl helper) — flag-only; the actual Error emission when
  // used outside preamble is a future polish (the mis-use cascade already
  // surfaces downstream Errors today).

  AssignValue!("current_environment", String::new(), Some(Scope::Global));
  def_macro_noop("\\@currenvir")?;
  // Note: LaTeX kernel defines \def\f#1{\def\@currenvir{#1}} but this is just
  // a kernel internal that gets overridden by user \newcommand{\f}{...}.
  // We do NOT define \f here — use \lx@setcurrenvir instead (matching Perl).
  // The old DefPrimitive!("\\f{}", ...) was a bug: primitives can't be overridden
  // by \newcommand, so \newcommand{\f}{\mathcal{F}} would silently fail, and
  // $\f$ would eat the closing $ as an argument, corrupting the mode stack.

  DefPrimitive!(
  "\\lx@setcurrenvir{}", sub[(env)] {
    let env_string = env.to_string();
    DefMacro!(T_CS!("\\@currenvir"), None, env);
    AssignValue!("current_environment", env_string);
  });
  Let!("\\@currenvline", "\\@empty");

  // Perl: latex_constructs.pool.ltxml line 190. Perl's body string ends
  // with a STRAY `}` — a transcription artifact from copying the LaTeX
  // kernel's `\def\@checkend#1{...\fi}` (the final `}` closes the `\def`,
  // it is NOT part of the replacement text). Standard LaTeX `\@checkend`
  // has no trailing brace. Perl's lenient gullet silently tolerates the
  // extra `}`, but ours raises `Error:unexpected:} Attempt to close
  // boxing group` when the brace pops a `\begingroup` frame. `\@checkend`
  // is only ever reached when a package redefines `\end` to call it (e.g.
  // extract.sty's `\def\end#1{\csname end#1\endcsname\@checkend{#1}...
  // \endgroup}`); there the stray `}` collided with extract's wrapping
  // `\begingroup`, producing one boxing-group error per environment.
  // Dropping the artifact matches standard-LaTeX semantics. Witness
  // 2007.09971 (IEEEtran+extract under ar5iv: 41 errors -> clean). See
  // KNOWN_PERL_ERRORS.md.
  DefMacro!(
    "\\@checkend{}",
    r"\def\reserved@a{#1}\ifx\reserved@a\@currenvir \else\@badend{#1}\fi"
  );

  // latex.ltx:15346-15364 `\begin#1`: `\UseHook{env/#1/before}` OUTSIDE the
  // group, then inside it `\@currenvir` and `\UseHook{env/#1/begin}` before
  // `\csname #1\endcsname`. The kernel's environment hooks live in
  // lthooks' own store (`\AddToHook{env/X/before}` — real lthooks under
  // the raw-loaded latexml.sty; the no-op `\UseHook` of latex_base.rs
  // otherwise), which `\begin`/`\end` never consulted: only the
  // `@environment@X@*` PushValue store written by etoolbox's
  // `\AtBeginEnvironment` & co. fired. functional.sty's
  // `\AddToHook{env/demohigh/before}{\MyDeleteShortVerb}` (functional
  // manual, 10 errors: the live shortvrb `|` re-read by codehigh as `\verb`)
  // and 39 tex/latex packages use the kernel form. SHARED (Perl's `\begin`
  // is the same), pdflatex clean. DefEnvironment-managed environments fire
  // `begin`/`end` from their constructors (dialect.rs). Guard:
  // `perfect_kernel_batch56::kernel_env_hooks_fire_around_environments`.
  DefMacro!("\\begin{}", sub[(env)] {
    let name = Expand!(env.clone()).to_string();
    let begin_name = format!("\\begin{{{name}}}");
    let before_opt = lookup_tokens(&format!("@environment@{name}@beforebegin"));
    let after_opt  = lookup_tokens(&format!("@environment@{name}@atbegin"));
    // The kernel's own token shapes: bare `\UseHook{env/#1/<point>}` for
    // before/begin/after (latex.ltx:15347/15362/15391) and, for `end`,
    // `\romannumeral\IfHookEmptyTF{env/#1/end}{\expandafter\z@}{\z@\UseHook
    // {env/#1/end}}` (:15386-15388): an empty hook vanishes at EXPANSION time,
    // so nothing sits between an alignment's last cell and `\endtabular`'s
    // implicit `\crcr` (a trailing `\multicolumn` row leaked its group when
    // an unexpandable token stood there).
    let use_hook = |point: &str| -> Vec<Token> {
      // Emitted only when the L3 hook system exists and reports code for
      // the hook (its own `\IfHookEmptyTF`, dialect.rs): with no hook there
      // is nothing to fire, and a line-reading environment (comment.sty)
      // must not see extra tokens at its `\end`.
      if !env_hook_has_code(&name, point) {
        return Vec::new();
      }
      let hook = |v: &mut Vec<Token>| {
        v.push(T_CS!("\\UseHook"));
        v.push(T_BEGIN!());
        v.extend(ExplodeText!(s!("env/{name}/{point}")));
        v.push(T_END!());
      };
      let mut v = Vec::new();
      if point == "end" {
        v.push(T_CS!("\\romannumeral"));
        v.push(T_CS!("\\IfHookEmptyTF"));
        v.push(T_BEGIN!());
        v.extend(ExplodeText!(s!("env/{name}/end")));
        v.push(T_END!());
        v.extend([T_BEGIN!(), T_CS!("\\expandafter"), T_CS!("\\z@"), T_END!()]);
        v.push(T_BEGIN!());
        v.push(T_CS!("\\z@"));
        hook(&mut v);
        v.push(T_END!());
      } else {
        hook(&mut v);
      }
      v
    };

    // LaTeX's own indirection for the ONE environment whose `\begin{…}`
    // is a binding constructor here: `\begin{document}` is `\begingroup
    // \document` (latex.ltx:15346-15362), so a `\document` the document
    // redefined — ltnews.tex:236 / l3news.tex:109 `\renewenvironment
    // {document}` around the `\input` of every issue, tools-overview.tex:59
    // `\def\document{\TO@document\maketitle…}` — is what a later
    // `\begin{document}` runs. Both engines took the constructor
    // regardless (Perl's `\begin` looks up `\begin{document}` first), so an
    // issue's `\end{document}` ended the whole newsletter (ltnews 3.9 %
    // recall, l3news 12.6 %). While `\document` is still the binding's alias
    // (`\let\document\begin{document}`, `\ifx`-equal), nothing changes.
    // Beyond Perl, user-approved: OXIDIZED_DESIGN_DIVERGENCES #232. Guard:
    // `cluster_package_guards::document_indirection::*`.
    // The ORIGINAL `\document` eats `\begin`'s group itself
    // (`\@execute@begin@hook{document}` `\endgroup`s it, latex.ltx
    // :15356-15361), so the constructor path opens none; a RENEWED
    // `document` is a normal environment and gets its
    // `\begingroup`/`\endgroup` below.
    let user_document = name == "document"
      && is_user_document_macro(&T_CS!("\\document"), &T_CS!("\\lx@orig@document"));
    if user_document {
      // A renewed `document` IS a normal environment: `\begin{document}` =
      // `\begingroup\document`, `\end{document}` = `\enddocument\endgroup`
      // (latex.ltx `\begin`/`\end`). Without the group every issue's
      // `\let\saved@addtocontents\addtocontents` + `\renewcommand` of the
      // l3news driver LEAKED into the next `\input` issue, until
      // `\saved@addtocontents` captured its own wrapper (a self-recursive
      // macro → `PushbackLimit`). Only the ORIGINAL `\document` eats
      // `\begin`'s group itself.
      let mut tks = before_opt.map(Tokens::unlist).unwrap_or_default();
      tks.extend(use_hook("before"));
      tks.push(T_CS!("\\begingroup"));
      // The end side follows this decision (see `\end`), recorded INSIDE the
      // group it opens (`document:indirected`, group-local): a class that
      // wraps BOTH macros around the originals (standalone.cls) must not have
      // its begin wrapper skipped (self-referential here) and its end wrapper
      // run, and a nested complete document (exam-n.cls:1348
      // `\includequestion`: `\let\document\@empty \let\enddocument\endinput
      // \input{…}`) must not leave a stale decision for the enclosing one.
      tks.push(T_CS!("\\lx@document@indirected"));
      tks.extend(Invocation!(T_CS!("\\lx@setcurrenvir"), vec![env]).unlist());
      if let Some(after) = after_opt {
        tks.extend(after.unlist());
      }
      tks.extend(use_hook("begin"));
      tks.push(T_CS!("\\document"));
      return Ok(Tokens::new(tks));
    }
    if is_defined(&begin_name) {
      let mut tks = before_opt.map(Tokens::unlist).unwrap_or_default();
      tks.extend(use_hook("before"));
      tks.push(T_CS!(begin_name));
      if name == "document" {
        // Inside the constructor's own group: this document opened none.
        tks.push(T_CS!("\\lx@document@direct"));
      }
      Ok(Tokens::new(tks)) // Magic cs!
    } else {
      let token = T_CS!(format!("\\{name}"));
      if !is_defined_token(&token) {
        // this creates {name} , {{ and }} are escapes in Rust's `format` macro
        let undef = format!("{{{name}}}");
        let message = s!("The environment {} is not defined.", undef);
        Error!("undefined", &undef, message);
        note_status(LogStatus::Undefined, Some(&undef));
        // Perl latex_constructs.pool.ltxml L207-208 installs a dummy
        // Constructor for `\<name>` whose body is `makeError('undefined',
        // $undef)` — emitting `<ltx:ERROR class='undefined'>{name}</ltx:ERROR>`
        // as a visible marker (and NOT a counted Error — the env-undefined
        // Error above is already logged). The earlier Rust port used a no-op
        // DefMacro instead, which kept the error COUNT right but silently
        // dropped the ERROR element from the output (Perl emits it). Mirror
        // Perl faithfully via the same make_error constructor that
        // `generate_error_stub` installs for undefined commands — count stays
        // 1 (witness 0810.4249: still Rust=Perl=1), output now has the marker.
        install_undefined_error_constructor(token, &undef);
      }
      let mut out_tokens = before_opt.map(Tokens::unlist).unwrap_or_default();
      out_tokens.extend(use_hook("before"));
      out_tokens.push(T_CS!("\\begingroup"));
      if let Some(after) = after_opt {
        out_tokens.extend(after.unlist());
      }
      out_tokens.extend(Invocation!(T_CS!("\\lx@setcurrenvir"), vec![env]).unlist());
      out_tokens.extend(use_hook("begin"));
      out_tokens.push(token);
      Ok(Tokens::new(out_tokens))
    }
  });

  // latex.ltx:15384-15393 `\end` ends with the epilogue
  // `\if@ignore\@ignorefalse\ignorespaces\fi` AFTER the `env/#1/after` hook.
  // Perl (latex_constructs.pool.ltxml:216-231) drops it; noindentafter.sty:44
  // `\def\nia@afterendenv#1\ignorespaces\fi{…}` — installed by
  // `\NoIndentAfterEnv` through `\AfterEndEnvironment` — is DELIMITED by
  // exactly those two tokens and otherwise scans away the rest of the
  // document (pkgloader manual: `\NoIndentAfterEnv{itemize}` swallowed the
  // `\DocInput` macrocode, 72 `misdefined:#` + Fatal; Perl fails the same
  // way). `\if@ignore` is false except after display math, so the group is
  // normally skipped whole.
  DefMacro!("\\end {}", sub[(env)]{
    let name = Expand!(env).to_string();
    let before = lookup_tokens(&s!("@environment@{name}@atend"));
    let after = lookup_tokens(&s!("@environment@{name}@afterend"));
    // The kernel's own token shapes: bare `\UseHook{env/#1/<point>}` for
    // before/begin/after (latex.ltx:15347/15362/15391) and, for `end`,
    // `\romannumeral\IfHookEmptyTF{env/#1/end}{\expandafter\z@}{\z@\UseHook
    // {env/#1/end}}` (:15386-15388): an empty hook vanishes at EXPANSION time,
    // so nothing sits between an alignment's last cell and `\endtabular`'s
    // implicit `\crcr` (a trailing `\multicolumn` row leaked its group when
    // an unexpandable token stood there).
    let use_hook = |point: &str| -> Vec<Token> {
      // Emitted only when the L3 hook system exists and reports code for
      // the hook (its own `\IfHookEmptyTF`, dialect.rs): with no hook there
      // is nothing to fire, and a line-reading environment (comment.sty)
      // must not see extra tokens at its `\end`.
      if !env_hook_has_code(&name, point) {
        return Vec::new();
      }
      let hook = |v: &mut Vec<Token>| {
        v.push(T_CS!("\\UseHook"));
        v.push(T_BEGIN!());
        v.extend(ExplodeText!(s!("env/{name}/{point}")));
        v.push(T_END!());
      };
      let mut v = Vec::new();
      if point == "end" {
        v.push(T_CS!("\\romannumeral"));
        v.push(T_CS!("\\IfHookEmptyTF"));
        v.push(T_BEGIN!());
        v.extend(ExplodeText!(s!("env/{name}/end")));
        v.push(T_END!());
        v.extend([T_BEGIN!(), T_CS!("\\expandafter"), T_CS!("\\z@"), T_END!()]);
        v.push(T_BEGIN!());
        v.push(T_CS!("\\z@"));
        hook(&mut v);
        v.push(T_END!());
      } else {
        hook(&mut v);
      }
      v
    };
    let mut t = T_CS!(s!("\\end{{{name}}}"));
    let mut out_tokens = Vec::new();
    // The `\end{document}` half of the indirection (see `\begin`): a
    // user-redefined `\enddocument` runs in place of the constructor (no
    // `\endgroup`: see `\begin` — the document environment opens no group);
    // the binding's own definition keeps the constructor.
    // … \endgroup`); the binding's alias keeps the constructor.
    // Symmetric with `\begin`: the user's `\enddocument` runs when the
    // user's `\document` ran, or when `\document` was left alone and only
    // `\enddocument` was redefined (tools-overview.tex:65). standalone.cls
    // wraps both; its begin wrapper is self-referential and skipped, so its
    // end wrapper (`\endstandalone`, closing a minipage the skipped begin
    // never opened) is skipped too.
    let indirected = name == "document" && lookup_bool("document:indirected");
    let user_enddocument = name == "document"
      && is_user_document_macro(&T_CS!("\\enddocument"), &T_CS!("\\lx@orig@enddocument"))
      && (indirected || x_equals(&T_CS!("\\document"), &T_CS!("\\lx@orig@document")));
    let orig_end = x_equals(&T_CS!("\\enddocument"), &T_CS!("\\lx@orig@enddocument"));
    if indirected && !user_enddocument && !orig_end {
      // The begin opened the group and `\enddocument` is a closure, not a
      // user macro — exam-n's `\let\enddocument\endinput`, whose `\endinput`
      // stops the included file after this line while `\end`'s tokens are
      // already expanded, as in TeX. LaTeX's `\end{document}` =
      // `\enddocument\endgroup`: run it, then close the begin's group.
      // With the ORIGINAL `\enddocument` (a package wrapped `\document`
      // around the original — dblfnote, etoolbox's `\AtEndPreamble` — and
      // left the end alone) the finalizer runs as on the constructor path:
      // in TeX the original never returns, so `\end`'s `\endgroup` is never
      // reached, and here an explicit one met the document constructor's
      // mode frame above the begin's group ("\endgroup Attempt to close a
      // group that switched to mode internal_vertical" at `\end{document}`,
      // seven manuals in sweep 83: yafoot-man, guitartabs, zanabazr, …).
      let mut tks = before.map(Tokens::unlist).unwrap_or_default();
      tks.extend(use_hook("end"));
      tks.push(T_CS!("\\enddocument"));
      tks.push(T_CS!("\\par"));
      tks.push(T_CS!("\\endgroup"));
      tks.extend(use_hook("after"));
      if let Some(afterend_toks) = after {
        tks.extend(afterend_toks.unlist())
      }
      return Ok(Tokens::new(tks));
    }
    if user_enddocument {
      let mut tks = before.map(Tokens::unlist).unwrap_or_default();
      tks.extend(use_hook("end"));
      tks.push(T_CS!("\\enddocument"));
      // `\par` first: an issue's last paragraph is still open, and this
      // engine's group bookkeeping refuses to close a group that switched
      // mode ("Attempt to close a group that switched to mode horizontal");
      // TeX ends that paragraph at the next page/vertical material anyway.
      tks.push(T_CS!("\\par"));
      tks.push(T_CS!("\\endgroup"));
      tks.extend(use_hook("after"));
      if let Some(afterend_toks) = after {
        tks.extend(afterend_toks.unlist())
      }
      return Ok(Tokens::new(tks));
    }
    if is_defined_token(&t) {
      // Magic CS! (its constructor fires `env/NAME/end` inside the group)
      out_tokens.push(t);
      out_tokens.extend(use_hook("after"));
      if let Some(afterend_toks) = after {
        out_tokens.extend(afterend_toks.unlist())
      }
    } else {
      // latex.ltx:15386-15391: `env/#1/end` inside the group before
      // `\end#1`, `env/#1/after` after the `\endgroup`.
      out_tokens = before.map(Tokens::unlist).unwrap_or_default();
      out_tokens.extend(use_hook("end"));
      t = T_CS!(s!("\\end{name}"));
      if is_defined_token(&t) {
        out_tokens.push(t);
      }
      out_tokens.push(T_CS!("\\endgroup"));
      out_tokens.extend(use_hook("after"));
      if let Some(afterend_toks) = after {
        out_tokens.extend(afterend_toks.unlist())
      }
    }
    out_tokens.extend([
      T_CS!("\\if@ignore"),
      T_CS!("\\@ignorefalse"),
      T_CS!("\\ignorespaces"),
      T_CS!("\\fi"),
    ]);
    Ok(Tokens::new(out_tokens))
  });

  TeX!(
    r"
\def\@ignorefalse{\global\let\if@ignore\iffalse}
\def\@ignoretrue {\global\let\if@ignore\iftrue}
\def\zap@space#1 #2{%
  #1%
  \ifx#2\@empty\else\expandafter\zap@space\fi
  #2}
\def\@unexpandable@protect{\noexpand\protect\noexpand}
\def\x@protect#1{%
   \ifx\protect\@typeset@protect\else
      \@x@protect#1%
   \fi
}
\def\@x@protect#1\fi#2#3{%
   \fi\protect#1%
}
\let\@typeset@protect\relax
\def\set@display@protect{\let\protect\string}
\def\set@typeset@protect{\let\protect\@typeset@protect}
\def\protected@edef{%
   \let\@@protect\protect
   \let\protect\@unexpandable@protect
   \afterassignment\restore@protect
   \edef
}
\def\protected@xdef{%
   \let\@@protect\protect
   \let\protect\@unexpandable@protect
   \afterassignment\restore@protect
   \xdef
}
\def\unrestored@protected@xdef{%
   \let\protect\@unexpandable@protect
   \xdef
}
\def\restore@protect{\let\protect\@@protect}
\set@typeset@protect
\def\@nobreakfalse{\global\let\if@nobreak\iffalse}
\def\@nobreaktrue {\global\let\if@nobreak\iftrue}
\@nobreakfalse

\newif\ifv@
\newif\ifh@
\newif\ifdt@p
\newif\if@pboxsw
\newif\if@rjfield
\newif\if@firstamp
\newif\if@negarg
\newif\if@ovt
\newif\if@ovb
\newif\if@ovl
\newif\if@ovr
\newdimen\@ovxx
\newdimen\@ovyy
\newdimen\@ovdx
\newdimen\@ovdy
\newdimen\@ovro
\newdimen\@ovri
\newif\if@noskipsec \@noskipsectrue
"
  );

  //======================================================================
  // C.1.4 Declarations
  //======================================================================
  // actual implementation later.
  //======================================================================
  // C.1.5 Invisible Commands
  //======================================================================
  // actual implementation later.

  //======================================================================
  // C.1.6 The \\ Command
  //======================================================================
  // In math, \\ is just a formatting hint, unless within an array, cases, .. environment.
  // Perl: DefConstructor('\lx@newline OptionalMatch:* [Glue]', sub { ... });
  // Complex constructor that checks document context:
  //   - in math: insert <ltx:XMHint name='newline'/>
  //   - no context or _CaptureBlock_: skip
  //   - ltx:p with parent _CaptureBlock_: maybeCloseElement('ltx:p')
  //   - can contain ltx:break: insert <ltx:break/>
  DefConstructor!("\\lx@newline OptionalMatch:* [Glue]", sub[document, args] {
    if lookup_bool_sym(pin!("IN_MATH")) {
      document.insert_element("ltx:XMHint", Vec::new(), Some(map!("name" => s!("newline"))))?;
    } else {
      // OXIDIZED_DESIGN surpass-Perl (#722): the optional [Glue] of `\\[20pt]` is the
      // extra vertical space LaTeX inserts at the break. Perl parses it and drops it
      // (ltx:break has no spacing slot in the schema). We PRESERVE it as a themeable CSS
      // custom property `--ltx-break-space` on the break; NO default rule consumes it, so
      // default rendering is byte-identical to a plain break — a theme (ar5iv) may map it
      // to margin/padding. Plain `\\` has no [Glue] (arg absent) so it stays attribute-free.
      // args[1] is the [Glue] (args[0] is the OptionalMatch:* star).
      let break_attrs = args
        .get(1)
        .and_then(|a| a.as_ref())
        .map(|g| g.to_attribute())
        .filter(|v| !v.is_empty() && v != "0.0pt" && v != "0pt")
        .map(|v| map!("cssstyle" => s!("--ltx-break-space:{v}")));
      if let Some(context) = document.get_element() {
        let tag = document::get_node_qname(&context);
        let capture_block = pin!("ltx:_CaptureBlock_");
        if tag == capture_block {
          // skip, if in insertBlock
        } else if tag == pin!("ltx:p") {
          // Close <p> if parent is _CaptureBlock_
          if let Some(parent) = context.get_parent() {
            if document::get_node_qname(&parent) == capture_block {
              document.maybe_close_element("ltx:p")?;
            } else if document::can_contain(&context, "ltx:break") {
              document.insert_element("ltx:break", Vec::new(), break_attrs)?;
            }
          }
        } else if document::can_contain(&context, "ltx:break") {
          document.insert_element("ltx:break", Vec::new(), break_attrs)?;
        }
      }
      // else: no context => skip
    }
  },
    reversion => Tokens!(T_CS!("\\\\"), T_CR!()),
    properties => { stored_map!("isBreak" => true) },
  );
  Let!("\\\\", "\\lx@newline");

  DefConstructor!("\\newline", "?#isMath(<ltx:XMHint name='newline'/>)(<ltx:break/>)",
    reversion  => Tokens!(T_CS!("\\newline"), T_CR!()),
    properties => { Ok(stored_map!("isBreak" => true)) },
  );

  Let!("\\@normalcr", "\\\\");
  Let!("\\@normalnewline", "\\newline");
  // latex.ltx:9247-9266: `\@normalcr` → `\@xnewline` → `\@newline[glue]` →
  // `\@gnewline`, whose raw body (inherited from the dump) dereferences the
  // scratch `\reserved@e{\reserved@f#1}` that only `\@normalcr` itself sets.
  // `\\` is native here, but raw classes enter the chain directly —
  // brief.cls:496 `\@nobreakcr` (`\stopbreaks`, :485) — and a parbox body
  // holding it is re-expanded during sizing, where the scratch macro is
  // undefined (ntgclass brief-sample, 2nd letter). Route the chain to the
  // native newline, as `\\`/`\@normalcr` already are. Guard:
  // `perfect_kernel_batch56::raw_newline_chain_is_the_native_newline`.
  DefMacro!("\\@xnewline", "\\@ifnextchar[\\@newline{\\lx@newline}");
  DefMacro!("\\@newline[]", "\\lx@newline[#1]");
  DefMacro!("\\@gnewline{}", "\\lx@newline");
  // NOTE: Activating this binding messes up an \afterassign test,
  //       so it may be best left disabled.
  // PushValue!("TEXT_MODE_BINDINGS" => Tokens!(T_CS!("\\\\"), T_CS!("\\@normalcr")));

  def_macro_noop("\\@nolnerr")?;
  DefMacro!(
    "\\@centercr",
    r"\ifhmode\unskip\else\@nolnerr\fi\par\@ifstar{\nobreak\@xcentercr}\@xcentercr"
  );
  DefMacro!(
    "\\@xcentercr",
    r"\addvspace{-\parskip}\@ifnextchar[\@icentercr\ignorespaces"
  );
  DefMacro!("\\@icentercr[]", "\\vskip #1\\ignorespaces");

  Ok(())
}

/// Is `cs` (`\document` / `\enddocument`) a macro the DOCUMENT gave it, which
/// `\begin{document}` / `\end{document}` should run (OXIDIZED_DESIGN_DIVERGENCES
/// #232)? Three things are NOT that: the binding's own definition (`\ifx`-equal
/// to the ORIGINAL `\begin{document}`/`\end{document}`, kept as
/// `\lx@orig@document`/`\lx@orig@enddocument` in `sect02.rs` — docmute and
/// subfiles redefine the `\end{document}` CS itself to end an `\input` file
/// while `\enddocument` keeps the original, so the current CS is no yardstick);
/// the kernel's re-let of `\document` to `\@notprerr` once the document has
/// begun (latex.ltx:1228 `\@onlypreamble`, sect02.rs), which every nested
/// `\begin{document}` of an `\input` complete document meets; and a wrapper
/// whose body contains the control sequence ITSELF — standalone.cls:1073-1083
/// captures the kernel macro's body with `\toks@\expandafter{\document …}`,
/// which for this engine's unexpandable alias is the token `\document` (a
/// self-loop), so such a wrapper keeps the constructor.
fn is_user_document_macro(cs: &Token, alias: &Token) -> bool {
  if x_equals(cs, alias) || x_equals(cs, &T_CS!("\\@notprerr")) {
    return false;
  }
  match lookup_definition(cs) {
    Ok(Some(defn)) if defn.is_expandable() => match defn.get_expansion() {
      Some(ExpansionBody::Tokens(body)) => !body.unlist_ref().iter().any(|t| t == cs),
      // A macro with no body at all — `\renewenvironment{document}{…}{}`'s
      // empty end code — is a user macro too.
      None => true,
      Some(ExpansionBody::Closure(_)) => false,
    },
    _ => false,
  }
}
