//! `latex_constructs` section 2: C.2 The Structure of the Document
//!
//! Split verbatim from the single `LoadDefinitions!` body (2026-09-03); the
//! definitions run in the original order from `super::load_definitions`.

use super::*;

#[rustfmt::skip]
pub(crate) fn load() -> Result<()> {
  // ======================================================================
  // C.2 The Structure of the Document
  // ======================================================================

  //**********************************************************************
  // C.2. The Structure of the Document
  //**********************************************************************
  //   prepended files (using filecontents environment)
  //   preamble (starting with \documentclass)
  //   \begin{document}
  //    text
  //   \end{document}

  // Perl: PushValue('@at@begin@document', $_[1]->unlist)
  // latex.ltx:18901 `\DeclareRobustCommand\AtBeginDocument{\AddToHook
  // {begindocument}}` (and `\AtEndDocument` = `\AddToHook{enddocument}`):
  // the TeX-level macro IS the L3 hook under the current default label, so
  // `\RemoveFromHook{begindocument}[l3doc]` (source2edoc.cls:12) can cancel
  // l3doc.cls:511's `\AtBeginDocument{\MakeShortVerb\"}`. Perl
  // (latex_constructs.pool.ltxml:296-297, 93f875a6) keeps a private
  // `@at@begin@document` store that ignores the label and that
  // `\RemoveFromHook` never sees, so the shortverb stayed live and
  // `"` scanned across ltoutenc.dtx's macrocode blocks (base/source2e, 30
  // errors; SHARED, pdflatex clean). With the hook system loaded, route
  // through `\AddToHook` (honouring an explicit `[label]`); the private
  // store stays the fallback for the format-less path and for the Rust
  // bindings' `AtBeginDocument()` helper (prelude.rs:28), which fires just
  // before the L3 `begindocument` hook (below).
  // Guard: `perfect_kernel_batch56::atbegindocument_joins_the_l3_begindocument_hook`.
  DefMacro!("\\AtBeginDocument[]{}", sub[(label, rules)] {
    at_document_hook("begindocument", "@at@begin@document", label, rules)
  });
  DefMacro!("\\AtEndDocument[]{}", sub[(label, rules)] {
    at_document_hook("enddocument", "@at@end@document", label, rules)
  });

  // Like  "<ltx:document xml:id='#id'>#body</ltx:document>",
  // But more complicated due to id, at begin/end document and so forth.
  // AND, lower-level so that we can cope with common errors at document end.
  DefConstructor!(T_CS!("\\begin{document}"), None, sub[document, _args, props] {
    let id = prop_str!(props,"id");
    // Already (auto) created?
    match document.findnode("/ltx:document", None) { Some(mut docel) => {
      if id != pin!("") {
        let id_s = with(id, |s| s.to_string());
        document.set_attribute(&mut docel, "xml:id", &id_s)?;
      }
    } _ => {
      let props = with(id, |id_str| string_map!("xml:id" => id_str));
      document.open_element("ltx:document", Some(props), None)?;
    }}
  },
  after_digest => sub[whatsit] {
    // Perl: beginMode('internal_vertical', 1) — noframe=1
    // Begin internal_vertical mode WITHOUT pushing a stack frame, keeping level=0
    begin_mode_opt("internal_vertical", true)?;
    // we need to re-bind in order to nest calls to the binding macro machinery
    DefMacro!("\\@currenvir", "document");
    assign_value("current_environment", "document", None);
    let expanded_id = Expand!(T_CS!("\\thedocument@ID"));
    whatsit.set_property("id", expanded_id);
    Let!("\\@nodocument", "\\relax", Scope::Global);
    // Clear \everypar at document start (Perl `AssignRegister('\everypar',
    // Tokens(), 'global')`, latex_constructs.pool L319). `\everypar` is a REGISTER,
    // so it must be cleared via `assign_register` (which writes the register
    // definition's value that `\the\everypar`/`lookup_register` read), NOT
    // `assign_value` (a separate State value slot the register never consults).
    // Raw-loading modern `ltpara` leaves the register holding the para-hook token
    // list `\g__para_standard_everypar_tl`; the old `assign_value` clear did not
    // actually empty it, so `\the\everypar` in the body still expanded to that
    // unmodelled hook. Nothing read the register in the body before, so this was
    // latent; it matters for any code that fires `\everypar` (algorithm2e numbering).
    assign_register("\\everypar", RegisterValue::Tokens(Tokens!()), Some(Scope::Global), Vec::new())?;
    // Perl #2798: at \begin{document}, make the fill widths consistent —
    //   \columnwidth = \hsize = \linewidth = \textwidth
    // (\columnwidth/\linewidth otherwise keep their 6in=433.62pt DefRegister
    // default, which is wrong for the default 345pt article \textwidth).
    if let Some(textwidth) = lookup_register("\\textwidth", Vec::new())? {
      assign_register("\\columnwidth", textwidth.clone(), None, Vec::new())?;
      assign_register("\\hsize", textwidth.clone(), None, Vec::new())?;
      assign_register("\\linewidth", textwidth, None, Vec::new())?;
    }
    let mut boxes = Vec::new();
    // Rust-only divergence (OXIDIZED_DESIGN "Frontmatter / locked-macro protection
    // at begin-document"; reproducer docs/reproducers/frontmatter_maketitle_double.tex):
    // digest the begin-document hook lists (\AtBeginDocument via @at@begin@document,
    // and @document@preamble@atend) with the state RE-LOCKED.
    //
    // These hooks carry RAW document/package TeX. A constructor's after-digest runs
    // state-UNLOCKED (execute_after_digest, definition.rs) so bindings can rebind/load
    // within their own before/after methods — but that unlock leaks into this nested
    // raw-TeX digest, letting a raw `\def`/`\let` of a binding-LOCKED macro (e.g.
    // `\AtBeginDocument{\def\maketitle{...}}`) silently win. LaTeXML deliberately owns
    // `\maketitle` (locked) so that `\title`/`\author` produce SEMANTIC frontmatter and
    // the class's visual `\@maketitle` reconstruction is suppressed; when a class's raw
    // redefinition slips the lock, the title is emitted twice (once semantic, once
    // visual). Perl shares this bug on an inline `\AtBeginDocument` (both engines double
    // vs pdflatex's single); it only escapes on acl.sty because its lock incidentally
    // holds for a raw-loaded `.sty`. Re-locking here enforces the intended rule — a
    // binding-locked macro is never redefined from the raw TeX layer — generally, for
    // every begin-document hook, so LaTeXML's own `\maketitle` runs and the frontmatter
    // is emitted exactly once (matching Perl's acl output, incl. `ltx_authors_1line`,
    // and surpassing Perl on the inline case). Scope is narrow: only these two nested
    // hook digests, not the general before/after-digest unlock.
    // Leave the preamble (`inPreamble=0`) only AFTER the begindocument hooks below —
    // the pre-#2846 latex.ltx placement (`\@preamblecmds` disables the \@onlypreamble
    // commands at L9522, AFTER firing the begindocument hook at L9512). So
    // `\RequirePackage`/`\usepackage` deferred to \AtBeginDocument runs while still
    // `inPreamble=1` and stays legal. Paragraph-breaking inside \AtBeginDocument
    // (upstream #2754) does NOT depend on this timing: `\par` (tex_paragraph.rs) closes
    // a paragraph whenever one is being built in the `document` environment — the hooks
    // run with `current_environment=document` — so a blank line after some
    // \AtBeginDocument text (horizontal mode) breaks a paragraph even while
    // `inPreamble=1`. That makes upstream #2846's early clear — and the
    // `inBeginDocumentHook` guard-decouple it forced (#2848) — unnecessary; one flag,
    // one transition. Ground truth: pdflatex
    // AND same-host Perl accept both a paragraph-splitting and a `\RequirePackage`-
    // loading \AtBeginDocument (reproducers atbegindocument_requirepackage.tex + the
    // #2754 book example; corpus witnesses arXiv:2605.00022 / 2605.00119).
    // KNOWN_PERL_ERRORS #43.
    // `\document` is `\@onlypreamble` and its hooks are `\UseOneTimeHook`s
    // (latex.ltx:9512/9537): a SECOND `\begin{document}` — ltnews.tex:236/296
    // and l3news.tex:109/177 `\renewenvironment{document}` then `\input` each
    // issue file with its own `\begin{document}` — fires nothing. Re-firing
    // ran csquotes' end-preamble block twice, whose hooks it `\undef`s after
    // use (csquotes.sty:2434-2446 `\csq@hook@nomultilang`/`@hyperref`; Perl
    // pool:304-335 re-fires too). Guard:
    // `perfect_kernel_batch54::second_begin_document_fires_no_hooks`.
    let first_begin = lookup_bool("inPreamble");
    if first_begin
      && let Some(ops) = lookup_tokens("@document@preamble@atend") {
      local_state_unlocked(false);
      let r = digest(ops);
      expire_state_unlocked();
      boxes.push(r?);
    }
    // Fire the L3 hook system for begindocument/before, then begindocument.
    // Modern LaTeX (with expl3) fires these in order at \begin{document}:
    //   1. \hook_use:n {begindocument/before}  — pre-init hook, STILL preamble
    //   2. @at@begin@document + \hook_use:n {begindocument}  — \AtBeginDocument,
    //      STILL preamble (latex.ltx disables \@onlypreamble only afterwards)
    //   3. leave the preamble  (inPreamble=0, AFTER the hooks — see below)
    // begindocument/before therefore fires while inPreamble=1: it carries
    // last-minute preamble setup (deferred `\RequirePackage`, translations.sty's
    // language initialiser) that must precede leaving the preamble — firing it
    // after inPreamble=0 wrongly rejects a deferred `\RequirePackage`.
    // Driver for begindocument/before: translations.sty L73-85 wraps its
    //   `\def\@trnslt@current@language{\languagename}` initialiser in
    //   `\AddToHook{begindocument/before}{…}`. Without this dispatch the
    //   .trsl dictionary loads (queued via `\AtBeginDocument`) inside the
    //   subsequent begindocument firing reference an undefined CS.
    // Witnesses: stage-2/3 of 100k warning corpus (2603.25051, 2604.07448).
    //
    // NOTE: this is a Rust-only deviation from Perl (Perl does not fire a
    // begindocument hook dispatch), but it's load-bearing because our raw
    // expl3-code.tex load path *does* define `\hook_use:n` and enqueues
    // real hook code against it. Keep until the kernel-parity direction
    // either (a) stops loading raw expl3-code.tex, or (b) ports l3hooks
    // natively with storage. See SYNC_STATUS.md "l3hooks parity".
    // Hook code runs RE-LOCKED like the sibling lists above: Perl runs every
    // begin-document hook under `$UNLOCKED=0` (State.pm:502-514 ignores a
    // redefinition of a `:locked` cs) and never fires the L3 hook at all, so a
    // binding-locked kernel macro must not be replaceable from here —
    // polyglossia.sty:1442-1456 `\cs_set:Npn \@caption #1 [#2] #3` in the
    // `begindocument` hook overrode our locked `\@caption` and its `[`-scan
    // overshot every figure (beamerdarkthemes guide, 101 caption errors).
    if first_begin && lookup_definition(&T_CS!("\\hook_use:n"))?.is_some() {
      local_state_unlocked(false);
      let r = digest(Tokens!(
        T_CS!("\\hook_use:n"),
        T_BEGIN!(),
        T_LETTER!("b"),
        T_LETTER!("e"),
        T_LETTER!("g"),
        T_LETTER!("i"),
        T_LETTER!("n"),
        T_LETTER!("d"),
        T_LETTER!("o"),
        T_LETTER!("c"),
        T_LETTER!("u"),
        T_LETTER!("m"),
        T_LETTER!("e"),
        T_LETTER!("n"),
        T_LETTER!("t"),
        T_OTHER!("/"),
        T_LETTER!("b"),
        T_LETTER!("e"),
        T_LETTER!("f"),
        T_LETTER!("o"),
        T_LETTER!("r"),
        T_LETTER!("e"),
        T_END!()
      ));
      expire_state_unlocked();
      boxes.push(r?);
    }
    // latex.ltx `\document` L9472: right after begindocument/before, the
    // kernel loads the expl3 BACKEND — `\@expl@sys@load@backend@@` →
    // `\sys_load_backend:n {}` unless one was chosen — raw-inputting
    // `l3backend-<driver>.def`, which defines the `\__color_backend_*` /
    // `\__pdf_backend_*` function families that l3color / l3pdf / raw
    // package code call at runtime. The embedded dump cannot carry these:
    // backend selection is a job-start decision (exactly why the kernel
    // defers it to `\document`), so without this firing every dump-mode
    // conversion left them undefined (witnesses: prettytok / spath3 manuals,
    // `Error:undefined:\__color_backend_reset:`; TL doc corpus 2026-08-31).
    //
    // The kernel's `\@expl@sys@load@backend@@` (expl3.ltx:130-134) is
    // `\str_if_exist:NF \c_sys_backend_str { \sys_load_backend:n {} }`;
    // `\lx@sys@load@backend` (latex_constructs_rust_only.rs) is that guard
    // with the DVI case named: in our default `\pdfoutput=0` state a
    // `\documentclass[pdftex]` option maps to `pdfmode` (expl3-code.tex:8352)
    // and `\__sys_load_backend_check:N` (:7992) rejects it with a counted
    // "inconsistent … using 'dvips'" (elpres, scidoc), so DVI names `dvips`
    // outright; a document that set `\pdfoutput=1` (commath, isorot, thinsp,
    // webguide, quantum-bibliographystyle-demo) gets the blank auto-select
    // (pdftex), and one whose preamble already loaded a backend
    // (quantum-template) is left alone. All RUST-ONLY: Perl never fires the
    // loader. Guards: `perfect_kernel_batch56::{backend_load_names_the_dvi_backend,
    // backend_load_follows_pdfoutput_and_prior_choice}`.
    if first_begin && lookup_definition(&T_CS!("\\lx@sys@load@backend"))?.is_some() {
      boxes.push(digest(Tokens!(T_CS!("\\lx@sys@load@backend")))?);
    } else if first_begin && lookup_definition(&T_CS!("\\@expl@sys@load@backend@@"))?.is_some() {
      boxes.push(digest(Tokens!(T_CS!("\\@expl@sys@load@backend@@")))?);
    }
    // @at@begin@document (\AtBeginDocument) + the begindocument hook. `inPreamble`
    // is STILL 1 here (we leave the preamble only after this block), so a deferred
    // `\RequirePackage`/`\usepackage` remains legal (corpus witnesses
    // arXiv:2605.00022 / 2605.00119: inconsolata.sty →
    // `\AtBeginDocument{...\usepackage{upquote}}` → upquote.sty's `\RequirePackage
    // {textcomp}`). A blank line here still splits paragraphs (fixes #2754): the hooks
    // run inside the `document` environment, so `\par` is active (only the RAW preamble,
    // where `document` is not yet on the env stack, no-ops it) regardless of
    // `inPreamble` (tex_paragraph.rs).
    // Raw `#`-bearing `\AtBeginDocument` chunks (see `at_document_hook`) run
    // first — before the L3 hook that holds the later, `#`-free raw chunks.
    if first_begin && let Some(ops) = lookup_tokens("@at@begin@document@rawparam") {
      local_state_unlocked(false);
      let r = digest(ops);
      expire_state_unlocked();
      boxes.push(r?);
    }
    // Order: the L3 `begindocument` hook (raw packages' `\AtBeginDocument`,
    // routed there since batch 56i) fires FIRST, then the bindings' private
    // `@at@begin@document` store — bindings outrank raw: cleveref_sty.rs
    // defers `\let\label\lx@cleverref@label` here, and the raw cleveref.sty:66
    // hook's `\def\label{\@ifnextchar[\label@optarg\label@noarg}` must not
    // shadow it (its `\cref@override@label@type` `[#1][#2]` scan ran to EOF:
    // crossreftools_driver, test-autonum fatal; RUST-ONLY, Perl has no raw
    // cleveref). Guard: `perfect_kernel_batch56::binding_begin_document_code_outranks_raw_hook`.
    if first_begin && lookup_definition(&T_CS!("\\hook_use:n"))?.is_some() {
      // Build the Tokens explicitly: `Tokenize!` runs at the runtime
      // catcode regime where `:` is OTHER (not LETTER), which would
      // truncate the CS to `\hook_use` and emit `:n` as plain text.
      // That leaks `_use:n` + arg-text into the document body.
      local_state_unlocked(false); // see the begindocument/before block
      let r = digest(Tokens!(
        T_CS!("\\hook_use:n"),
        T_BEGIN!(),
        T_LETTER!("b"),
        T_LETTER!("e"),
        T_LETTER!("g"),
        T_LETTER!("i"),
        T_LETTER!("n"),
        T_LETTER!("d"),
        T_LETTER!("o"),
        T_LETTER!("c"),
        T_LETTER!("u"),
        T_LETTER!("m"),
        T_LETTER!("e"),
        T_LETTER!("n"),
        T_LETTER!("t"),
        T_END!()
      ));
      expire_state_unlocked();
      boxes.push(r?);
    }
    if first_begin && let Some(ops) = lookup_tokens("@at@begin@document") {
      local_state_unlocked(false);
      let r = digest(ops);
      expire_state_unlocked();
      boxes.push(r?);
    }
    // Leave the preamble now — AFTER the begindocument hooks (the `\@preamblecmds`
    // point, latex.ltx L9522). This single transition both (a) disables the
    // \@onlypreamble commands, so from here the onlyPreamble guard rejects body-level
    // `\RequirePackage`/`\usepackage` (matching pdflatex + Perl), and (b) restores the
    // pre-#2846 placement — `\AtBeginDocument` above ran while still `inPreamble=1`.
    // `\par` paragraph-breaking is governed by mode + the `document` env, not this flag.
    assign_value("inPreamble", false, None);
    // latex.ltx:9525 `\UseOneTimeHook{begindocument/end}` — AFTER the
    // preamble is left. jwjournal.cls:643-650 wraps the whole body in a `+b`
    // environment from this hook (and closes it from `enddocument`); without
    // it the markdown body typeset raw (`##` reaching the stomach, 4 docs).
    // Guard: `perfect_kernel_batch54::begindocument_end_and_enddocument_hooks_fire`.
    // UNREAD onto the main stream rather than digest in a string mouth: a
    // `+b` environment opened from this hook must read its body from the
    // document file, not from the hook's own (empty) mouth.
    if first_begin && lookup_definition(&T_CS!("\\hook_use:n"))?.is_some() {
      unread(hook_use_tokens("begindocument/end"));
    }
    // Preamble cleanup: force `\ExplSyntaxOff` if `_` is still LETTER at
    // document start. Mirrors LaTeX2e kernel's preamble cleanup (latex.ltx
    // L7122 `\bool_if:NTF \l__kernel_expl_bool { \ExplSyntaxOff } ...`) —
    // packages like mhchem.sty end with an unmatched final `\ExplSyntaxOn`
    // (see mhchem.sty tail, "legacy" block), and LaTeX's kernel relies on
    // this scheduled cleanup to restore catcodes before the document body.
    // Without this, `\sum_{...}` tokenizes as the CS `\sum_` (letter `_`)
    // rather than `\sum` + `_` + `{...}`.
    if lookup_catcode('_') == Some(Catcode::LETTER)
      && lookup_definition(&T_CS!("\\ExplSyntaxOff"))?.is_some()
    {
      boxes.push(digest(Tokens!(T_CS!("\\ExplSyntaxOff")))?);
    }
    // Fire babel language activation AFTER all hooks (including babel's own
    // \selectlanguage call). This runs even if babel's hook code has errors.
    // Use T_CS! directly since @ is OTHER catcode at \begin{document} time.
    if lookup_definition(&T_CS!("\\lx@babel@activate@mainlang"))?.is_some() {
      boxes.push(digest(Tokens!(T_CS!("\\lx@babel@activate@mainlang")))?);
    }
    // @document@preamble@afterend runs after the \@preamblecmds point (both
    // inPreamble=0 and the hook window closed above), so onlyPreamble commands
    // here are already disabled — matching latex.ltx's begindocument/end.
    if first_begin
      && let Some(ops) = lookup_tokens("@document@preamble@afterend") {
      boxes.push(digest(ops)?);
    }
    whatsit.set_font(lookup_font().unwrap()); // Start w/ whatever font was last selected.
    leave_horizontal_internal();
    boxes
  });

  // \document is used directly in e.g. expl3.sty
  Let!("\\document", "\\begin{document}", Scope::Global);

  /// `\hook_use:n {name}` as explicit tokens: `Tokenize!` runs under the
  /// runtime catcodes, where `:` is OTHER and `/` must stay OTHER inside the
  /// name.
  fn hook_use_tokens(name: &str) -> Tokens {
    let mut toks = vec![T_CS!("\\hook_use:n"), T_BEGIN!()];
    for c in name.chars() {
      toks.push(if c.is_ascii_alphabetic() {
        T_LETTER!(c.to_string())
      } else {
        T_OTHER!(c.to_string())
      });
    }
    toks.push(T_END!());
    Tokens::new(toks)
  }

  DefConstructor!(T_CS!("\\end{document}"), None, sub[document,_args,_props] {
      document.close_element("ltx:document")?;
    },
    before_digest => {
      let mut boxes : Vec<Digested> = Vec::new();
      if let Some(ops) = lookup_tokens("@at@end@document") {
        boxes.push(digest(ops)?);
      }
      // latex.ltx:15257 `\UseOneTimeHook{enddocument}` (the lthooks slot; the
      // legacy `\AtEndDocument` list above is `@at@end@document`).
      if lookup_definition(&T_CS!("\\hook_use:n"))?.is_some() {
        local_state_unlocked(false);
        let r = digest(hook_use_tokens("enddocument"));
        expire_state_unlocked();
        boxes.push(r?);
      }
      // Should we try to indent the last paragraph? If so, it goes like this:
      boxes.push(digest(T_CS!("\\lx@normal@par"))?);
      // Pop unclosed groups and environments back to the document frame
      // so endMode's strict BOUND_MODE check sees the right frame at the
      // top. Mirrors Perl latex_constructs.pool.ltxml L350-374. Without
      // this loop, papers with a dangling `\begingroup` inside the body
      // (e.g. `\providecommand{\href}[2]{#2}\begingroup\raggedright
      // \begin{thebibliography}{99}`) trigger
      // "Attempt to end mode `internal_vertical` in `internal_vertical`"
      // because the top frame is the dangling group, not the document.
      // Note: Rust port omits Perl's if_stack handling — Rust's gullet
      // does not maintain an explicit if_stack value.
      let top_is_document = is_value_bound("current_environment", Some(0))
        && lookup_string("current_environment") == "document";
      if !top_is_document {
        let mut popped_lines: Vec<String> = Vec::new();
        while !(is_value_bound("current_environment", Some(0))
          && lookup_string("current_environment") == "document")
          && get_frame_depth() > 0
        {
          let initiator = lookup_string("groupInitiator");
          let initiator = if initiator.is_empty() {
            "<unknown>".to_string()
          } else {
            initiator
          };
          let env_bound = is_value_bound("current_environment", Some(0));
          let env_name = if env_bound {
            lookup_string("current_environment")
          } else {
            String::new()
          };
          if !env_name.is_empty() {
            popped_lines.push(s!("Environment {env_name} opened by {initiator}"));
          } else {
            popped_lines.push(s!("Group opened by {initiator}"));
          }
          pop_frame()?;
        }
        let detail = if popped_lines.is_empty() {
          String::new()
        } else {
          s!("\n{}", popped_lines.join("\n"))
        };
        Warn!(
          "unexpected",
          "\\end{document}",
          s!(
            "Attempt to end document with open groups, environments or conditionals{detail}"
          )
        );
      }
      // Perl: endMode('internal_vertical', 1) — noframe=1
      // End mode without popping stack frame (executes beforeAfterGroup).
      //
      // Skip the end-mode call if the document frame was never opened (e.g.
      // AmsTeX papers using `\input amstex` + `\documentstyle{amsppt}` +
      // `\enddocument` — they never call `\begin{document}`, so no
      // `internal_vertical` mode-frame exists to close). Recognized by:
      // current_environment is bound-on-top with value "document". Without
      // this guard, AmsTeX papers fail with `Attempt to end mode
      // 'internal_vertical' in 'vertical'` at the very last token.
      let in_document_env = is_value_bound("current_environment", Some(0))
        && lookup_string("current_environment") == "document";
      if in_document_env {
        end_mode_opt("internal_vertical", true)?;
      }
      flush();
      boxes
  });

  // \enddocument is used directly in e.g. standalone.cls
  Let!("\\enddocument", "\\end{document}", Scope::Global);

  Ok(())
}

/// `\AtBeginDocument[label]{code}` / `\AtEndDocument`: `\AddToHook{hook}
/// [label]{code}` when the L3 hook system is loaded, else the private store.
fn at_document_hook(
  hook: &str,
  store: &str,
  label: Option<Tokens>,
  rules: Tokens,
) -> Result<Tokens> {
  // INTERIM (K3 correctness item, KERNEL_CAPABILITIES.md): a parameter-
  // bearing chunk under a package label (pm-isomath.sty's
  // `\AtBeginDocument{…\NewDocumentCommand\vectorsymbol{s m}{…#2…}}`)
  // takes lthooks' labeled `\exp_args:Nx` cleanup path (latex.ltx:5375,
  // 5401-5416), whose x-expansion our gullet does not yet reproduce: a
  // `\noexpand`-family `\tl_if_empty:nTF` surfaced inside the
  // `\csname g__hook_…` (euclideangeometry-man 100× + Fatal, sweep #41).
  // Until the gullet is faithful there, such chunks stay in the private
  // store (pre-56i behaviour); `#`-free chunks and explicitly labeled ones
  // go through lthooks, which is what `\RemoveFromHook` and the top-level-
  // last order need. Guard: `perfect_kernel_batch56::hashful_begin_document_chunk_under_a_package_label`.
  let has_param = rules
    .unlist_ref()
    .iter()
    .any(|t| t.get_catcode() == Catcode::PARAM);
  if has_param && label.is_none() && hook == "begindocument" {
    // K7 (KERNEL_CAPABILITIES.md): latex.ltx keeps ONE FIFO `\@begindocumenthook`
    // (`\AtBeginDocument` = `\AddToHook{begindocument}`). A `#`-bearing chunk
    // cannot go through lthooks' cleanup here (K3), so it is stored under a
    // fresh `\lx@bdhook@N` macro — a no-parameter body keeps its `#` tokens
    // verbatim, exactly what the private store replayed — and THAT `#`-free
    // name is what lthooks receives, at the chunk's registration position:
    // alphabeta.sty:103-699's `#`-chunk before hep-math-font.sty:150-202's
    // `#`-free one (hep-paper-documentation; a separate first-fired store
    // inverted the opposite order, italian.ldf:156's nested `##1` hook running
    // before verifica.cls:65's earlier class hook — Gemini T5). Without
    // lthooks (plain-derived formats) the private store keeps the old order.
    // Guards: `perfect_kernel_batch56::{raw_hashful_begin_document_chunk_keeps_fifo_order,
    // begin_document_hooks_run_in_registration_order}`.
    if lookup_meaning(&T_CS!("\\hook_gput_code:nnn")).is_some() {
      use std::sync::atomic::{AtomicUsize, Ordering};
      static BDHOOK_SEQ: AtomicUsize = AtomicUsize::new(0);
      let n = BDHOOK_SEQ.fetch_add(1, Ordering::Relaxed);
      let cs = T_CS!(s!("\\lx@bdhook@{n}"));
      DefMacro!(cs, None, rules, scope => Some(Scope::Global));
      let mut out = vec![T_CS!("\\AddToHook"), T_BEGIN!()];
      out.extend(ExplodeText!(hook));
      out.push(T_END!());
      out.push(T_BEGIN!());
      out.push(cs);
      out.push(T_END!());
      return Ok(Tokens::new(out));
    }
    push_value("@at@begin@document@rawparam", rules)?;
    return Ok(Tokens!());
  }
  if lookup_meaning(&T_CS!("\\hook_gput_code:nnn")).is_some() && (label.is_some() || !has_param) {
    let mut out = vec![T_CS!("\\AddToHook"), T_BEGIN!()];
    out.extend(ExplodeText!(hook));
    out.push(T_END!());
    if let Some(label) = label {
      out.push(T_OTHER!("["));
      out.extend(label.unlist());
      out.push(T_OTHER!("]"));
    }
    out.push(T_BEGIN!());
    out.extend(rules.unlist());
    out.push(T_END!());
    Ok(Tokens::new(out))
  } else {
    push_value(store, rules)?;
    Ok(Tokens!())
  }
}
