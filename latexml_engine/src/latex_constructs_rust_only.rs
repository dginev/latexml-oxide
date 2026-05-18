//! latex_constructs_rust_only — Rust-side hotfix overrides for LaTeX-format CSes.
//!
//! Holds bindings present in the Rust port but **not** in any of Perl's three
//! `latex_{base,bootstrap,constructs}.pool.ltxml` files. Anything that lives
//! here is a hotfix tracked separately so that the corresponding Rust
//! "engine/latex_*.rs" siblings stay byte-for-byte parity with the Perl
//! source.
//!
//! Loaded LAST in `latex.rs`'s `LoadFormat('latex')` chain, after
//! `latex_constructs`, so every entry can rely on:
//! * The dump (or `latex_base.rs` under NODUMP) having installed raw LaTeX-kernel CSes.
//! * `latex_constructs.rs` having registered its own definitions (which some entries here `Let!`
//!   against — e.g. `\IfPackageLoadedTF ↦ \@ifpackageloaded`).
//!
//! Categories (in source order below):
//! 1. Modern LaTeX kernel CSes added post-2020 (the `\If…AtLeast/LoadedTF` family). LaTeX2e re-Lets
//!    these from the kernel; the Perl source predates them, so they need an explicit override here.
//! 2. LaTeXML-internal helpers that the engine code expects to exist (`\ltx@hard@MessageBreak`,
//!    `\ltx@ifclassloaded`, `\ltx@ifpackageloaded`).
//! 3. List internals not in Perl source (`\@bls`, `\@listi`-`\@listvi`, `\@maxlistdepth`). The dump
//!    captures the values raw `latex.ltx` installs; these are defensive overrides for the NODUMP
//!    path.
//! 4. Misc Rust-side stubs (`\@latexbug`, `\maybe@end@title`, `\thebibliography@ID` empty default).
//!
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  //======================================================================
  // 1. Modern LaTeX kernel — `\If…AtLeast/LoadedTF` family
  //
  // Latex.ltx L15252-15256: LaTeX3-style aliases for the file-load
  // tracking commands. The dump captures these as `Lt(...)` self-let
  // entries that don't actually replay because we filter same-target
  // aliases in `dump_writer`. Re-establish here post-dump.
  //======================================================================
  Let!("\\IfPackageLoadedTF",  r"\@ifpackageloaded");
  Let!("\\IfClassLoadedTF",    r"\@ifclassloaded");
  Let!("\\IfPackageAtLeastTF", r"\@ifpackagelater");
  Let!("\\IfClassAtLeastTF",   r"\@ifclasslater");
  // \IfFormatAtLeastTF{<date>}{<true>}{<false>}: alias to
  // `\@ifl@t@r@released` — but that name isn't captured by the dump,
  // so the Let creates a dangling alias and downstream usage errors
  // (witness 2408.03197 — greek-fontenc.def probes the macro).
  // Define directly as a 3-arg gobble that always takes the "true"
  // branch (we don't model format dates). Witness 2408.03197, 2408.04893.
  DefMacro!("\\IfFormatAtLeastTF{}{}{}", "#2");
  Let!("\\IfFileAtLeastTF",    r"\@ifl@t@r");

  // \UseRawInputEncoding — latex.ltx L18268-18324 defines this kernel CS
  // for legacy 8-bit-encoding compat (used by papers that pre-date the
  // 2018-04-01 default switch to UTF-8). Upstream `\let`s it to `\relax`
  // after first use (L18324). LaTeXML's dump skips this section, so the
  // CS arrives undefined — papers like 1711.09157 with
  // `\UseRawInputEncoding` at line 1 col 1 fail with
  // `Error:undefined:\UseRawInputEncoding`. Define as a no-op so the
  // legacy preamble compiles silently; the encoding-switching behaviour
  // is irrelevant for our XML pipeline.
  Let!("\\UseRawInputEncoding", r"\relax");

  //======================================================================
  // 2. LaTeXML-internal helpers
  //======================================================================
  // \ltx@hard@MessageBreak — emit a hard newline in error/warning text.
  // Used by `\GenericError`/`\GenericWarning`; the dump-loader's
  // let-target safety filter can clobber it under certain orderings,
  // so define here post-dump as well.
  DefMacro!("\\ltx@hard@MessageBreak", None, "^^J");

  // Kernel argument-gobbling macros — defensive re-declaration. These
  // are defined in latex_base.rs L65 (and Perl's latex_dump.pool.ltxml
  // L2063 has them) but our current Rust latex.dump.txt is missing
  // M-records for them (dump-build coverage gap). When dump load is the
  // active LoadFormat branch, latex_base is NOT loaded — so \@gobble
  // stays undefined. Re-declare here so they're always available
  // regardless of dump completeness. Witness: 2512.06027 (and ~2 v6
  // papers) — textcomp.sty raw-load calls \@gobble at L74 and crashes.
  DefMacro!("\\@gobble{}",          None);
  DefMacro!("\\@gobbletwo{}{}",     None);
  DefMacro!("\\@gobblefour{}{}{}{}", None);

  // LaTeXML aliases for the file-loaded predicates.
  Let!("\\ltx@ifpackageloaded", r"\@ifpackageloaded");
  Let!("\\ltx@ifclassloaded",   r"\@ifclassloaded");

  //======================================================================
  // 3. List internals — defensive NODUMP-path overrides
  //
  // Raw LaTeX classes (article.cls etc.) define these; the dump captures
  // the kernel's `\def`s. Under `LATEXML_NODUMP=1` (no dump load) the
  // bindings would be missing, so we install no-op fallbacks here.
  //======================================================================
  DefRegister!("\\@bls"          => Dimension!("12pt"));
  DefRegister!("\\@maxlistdepth" => Number::new(6));

  // \tracingstacklevels / \@nil / \@expl@str@if@eq@@nnTF moved to
  // latex_bootstrap.rs — must be defined BEFORE the dump loads (the
  // dump's latexrelease replay probes them).
  //
  // KNOWN ISSUE — papers that pin latexrelease to an older release
  // via \RequirePackage[YYYY-MM-DD]{latexrelease} (e.g. 2503.21471)
  // trip a cascade of undefined helpers (\@expl@str@if@eq@@nnTF,
  // \@expl@cs@to@str@@N, \robust@command@act, \ExpandArgs, ...)
  // because our \IncludeInRelease always runs the body. The proper
  // fix is date-aware IncludeInRelease (skip rollback blocks unless
  // release_date < block_date). Stubbing individual helpers gets us
  // partway but the cascade has many tendrils — defer to a focused
  // IncludeInRelease refactor.

  // List formatting macros from article.cls / report.cls / book.cls.
  // No-ops because LaTeXML handles list formatting via CSS.
  DefMacro!("\\@listi",   "");
  DefMacro!("\\@listii",  "");
  DefMacro!("\\@listiii", "");
  DefMacro!("\\@listiv",  "");
  DefMacro!("\\@listv",   "");
  DefMacro!("\\@listvi",  "");

  //======================================================================
  // 4. Misc Rust-side stubs
  //======================================================================
  // `\@latexbug` — kernel macro used to mark would-be bug reports.
  // No-op stub.
  DefMacro!("\\@latexbug", "");

  // `\maybe@end@title` — Constructor that closes ltx:titlepage if open.
  // Used by Rust's titling pipeline; not directly mirrored in Perl.
  DefConstructor!("\\maybe@end@title", sub[document, _args, _props] {
    if document.is_closeable("ltx:titlepage").is_some() {
      document.close_element("ltx:titlepage")?;
    }
  });

  // `\thebibliography@ID` — initial empty default. Per-bibliography
  // value is reassigned at \begin{thebibliography} time (see
  // latex_constructs.rs `\bibliography` constructor).
  DefMacro!("\\thebibliography@ID", "");

  //======================================================================
  // 5. Modern LaTeX kernel (2023+) — `\NewCommandCopy`/`\DeclareCommandCopy`/
  //    `\ShowCommand` from `ltcmd.dtx` (semantic-let equivalents).
  //
  // Not in Perl LaTeXML (too new), but needed for modern packages
  // (tcolorbox, etc.).
  //======================================================================
  DefPrimitive!("\\NewCommandCopy Token Token", sub[(new_cs, old_cs)] {
    state::let_i(&new_cs, &old_cs, None);
  });
  DefPrimitive!("\\DeclareCommandCopy Token Token", sub[(new_cs, old_cs)] {
    state::let_i(&new_cs, &old_cs, None);
  });
  DefMacro!("\\ShowCommand Token", "");

  //======================================================================
  // 6. Modern LaTeX (2015+) extras
  //======================================================================
  // `\extrafloats{N}` — request N extra float slots (no-op in LaTeXML).
  DefPrimitive!("\\extrafloats{}", None);

  // `\wlog{...}` — write to log only (no-op in LaTeXML).
  DefMacro!("\\wlog{}", "");

  // `\Gin@driver` — pre-defined empty so graphics.sty doesn't error
  // when loaded from disk (LaTeXML doesn't run a Backend driver).
  // Not in Perl source; pure Rust hotfix.
  DefMacro!("\\Gin@driver", "");

  // `\@tabacckludge` simplified body — Perl-faithful body lives in
  // latex_base.rs (Perl L357: `\csname\string#1\endcsname`). Under
  // the dump path latex_base.rs is skipped and the dump-captured
  // body uses the latex.ltx `\@changed@cmd`-wrapped form which
  // emits in-math warnings via `\@inmathwarn` and routes through
  // `\cf@encoding` lookup. That chain doesn't expand cleanly under
  // Rust's expansion model (encoding tests cp1250/cp852/latin2/
  // latin4/latin10 break with it). Override here so the dump-path
  // body matches latex_base.rs's simpler Perl-faithful form.
  DefMacro!("\\@tabacckludge {}", "\\csname\\string#1\\endcsname");

  //======================================================================
  // Sections 8-10 (label-macros, page registers, footnote counters,
  // tocdepth/secnumdepth counters, version-parsing \@ifl@t@r body) were
  // previously defined here for dump-path coverage. As of 2026-05-18,
  // `latex_constructs.rs` covers all of them and runs in BOTH dump and
  // NODUMP paths, so these defensive overrides were redundant. Removed
  // to eliminate triple-definition (latex_base.rs + latex_constructs.rs
  // + rust_only.rs).
  //======================================================================
  // 7a. Defensive NODUMP-path overrides for raw-LaTeX-kernel CSes
  //
  // Perl gets these from raw `latex.ltx` load (dump captures them).
  // Rust adds explicit overrides so the NODUMP path keeps working.
  //======================================================================
  // `\@@appendix` — body of `\appendix` after `\@startsection` chain.
  // Perl uses it as a Let target (latex_constructs.pool.ltxml:694) but
  // doesn't define it; the value comes from raw latex.ltx.
  DefMacro!("\\@@appendix", "\\@startsection{appendix}{0}{}{}{}{}");

  // `\textperiodcentered` — middle dot. Perl uses it as `\labelitemiv`'s
  // body (latex_constructs.pool.ltxml:1584) but doesn't define it (sister
  // entries `\textbullet`, `\textdaggerdbl`, `\textparagraph`,
  // `\textsection` ARE in latex_constructs:5404-5408 — Perl is missing
  // this one specifically).
  DefPrimitive!("\\textperiodcentered", "\u{00B7}"); // MIDDLE DOT

  //======================================================================
  // 7. Rust helper used by `\newlength` (latex_constructs.rs)
  //======================================================================
  // `\@check@length` — verify a CS is a length register; if not, define
  // it as a Dimension(0) and warn. Mirrors the role of internal kernel
  // checks done implicitly by Perl LaTeXML via DefRegister probing.
  DefPrimitive!("\\@check@length DefToken", sub[(cs)] {
    match lookup_definition(&cs)? {
      None => {
        let message = s!("'{}' is not a length; defining it now", cs.stringify());
        Warn!("undefined", cs, message);
        DefRegister!(cs, None, Dimension::new(0));
      },
      Some(defn) => if !defn.is_register() {
        let message = s!("'{}' length was expected, got {:?} instead of register.",
          cs.to_string(), defn.register_type());
        Error!("misdefined", cs, message);
      }
    };
  });

  //======================================================================
  // 7b. `\@ensuremath` — Rust-only inner helper for `\ensuremath`
  //
  // Perl's `\ensuremath` is a single DefMacro doing the math-mode dance
  // directly. Rust splits into `\ensuremath → \protect\@ensuremath` (in
  // latex_constructs.rs, parity with Perl L2133) plus this `\@ensuremath`
  // body so the `\protect` mechanism preserves the call until digestion.
  //======================================================================
  // protected => true prevents read_x_token(fully_expand=false) from
  // expanding this (needed for lx_change_case_tokens to preserve
  // \ensuremath{} content unchanged).
  DefMacro!("\\@ensuremath{}", sub[(stuff)] {
    if state::lookup_bool_sym(pin!("IN_MATH")) {
      stuff.unlist()
    } else {
      let mut result = vec![T_MATH!()];
      result.extend(stuff.unlist());
      result.push(T_MATH!());
      result
    }
  }, protected => true);

  //======================================================================
  // 8. {filecontents}/{filecontents*} environments — Rust impl
  //
  // Perl uses Semiverbatim DefConstructor for begin{filecontents}; Rust
  // implements via DefPrimitive that reads raw lines until end-marker
  // and caches the content for later \input. Helper fn defined here so
  // the migration is self-contained.
  //======================================================================
  fn cache_filecontents(end_marker: &str, header_star: bool) -> Result<()> {
    gullet::skip_spaces()?;
    let filename_toks = gullet::read_arg(ExpansionLevel::Off)?;
    let filename = filename_toks.to_string();
    // Perl latex_constructs L4316-4353: header comments match Perl's
    // three-line preamble. The \jobname line is synthesized as `\jobname`
    // (unexpanded literal) rather than the digested jobname — our tests
    // don't exercise a specific date and we don't want to leak
    // compile-time state into the dump-like content cache.
    let mut lines: Vec<String> = vec![
      format!("%% LaTeX2e file `{filename}'"),
      if header_star {
        "%% generated by the `filecontents*' environment".to_string()
      } else {
        "%% generated by the `filecontents' environment".to_string()
      },
      "%% from source `\\jobname' on YYYY/MM/DD.".to_string(),
    ];
    if !header_star { lines.push("%%".to_string()); }
    // Discard remainder of \begin{filecontents} line
    gullet::read_raw_line();
    // Read raw lines until end marker
    loop {
      match gullet::read_raw_line() {
        Some(line) if !line.contains(end_marker) => lines.push(line),
        _ => break,
      }
    }
    let n = lines.len();
    let content = lines.join("\n");
    Info!("note", "filecontents", s!("Cached filecontents for {filename} ({n} lines)"));
    state::assign_value(&s!("{filename}_contents"), Stored::from(content), Some(Scope::Global));
    Ok(())
  }
  // The \filecontents primitive reads filename + raw lines until \end{filecontents}.
  // When called via \begin{filecontents}, \begin opens a group first, so we manually
  // close the group after caching, matching the \end that was consumed.
  DefPrimitive!("\\filecontents", {
    cache_filecontents("\\end{filecontents}", false)?;
    stomach::endgroup()?;
  });
  DefPrimitive!("\\lx@filecontents@star", {
    cache_filecontents("\\end{filecontents*}", true)?;
    stomach::endgroup()?;
  });
  state::assign_meaning(
    &T_CS!("\\filecontents*"),
    state::lookup_meaning(&T_CS!("\\lx@filecontents@star")).unwrap_or(Stored::None),
    Some(Scope::Global),
  );
  DefMacro!("\\endfilecontents", "");
  state::assign_meaning(
    &T_CS!("\\endfilecontents*"),
    state::lookup_meaning(&T_CS!("\\endfilecontents")).unwrap_or(Stored::None),
    Some(Scope::Global),
  );
});
