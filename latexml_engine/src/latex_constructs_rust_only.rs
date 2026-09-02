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
//! The file ends with a **retraction record**: the MathSciNet `\cprime` transliteration family was
//! briefly always-on here and was removed 2026-07-27. The comment is kept because it carries the
//! witnesses and the reasoning — do not re-add the family.
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
  // \IfFormatAtLeastTF is deliberately NOT redefined here. latex.ltx L18405
  // defines it as `\@ifl@t@r\fmtversion`, and both halves survive the dump
  // (`\@ifl@t@r` + `\@parse@version@`, `\fmtversion` = the real per-TL-year
  // date, issue #739). The former always-true 3-arg stub (kept for witness
  // 2408.03197 / 2408.04893, greek-fontenc.def probing the macro, back when
  // the Let to `\@ifl@t@r@released` dangled) answered YES to any date, so
  // KOMA's `\IfLTXAtLeastTF{<KOMA year+2>/…}` (scrartcl.cls L2028-2035 via
  // scrbase.sty L127) warned "Your are using a KOMA-Script version, that has
  // not been tested with LaTeX version" on every KOMA document. Guard:
  // `perfect_kernel_batch53::ifformatatleast_compares_real_fmtversion`.
  Let!("\\IfFileAtLeastTF",    r"\@ifl@t@r");

  // \UseRawInputEncoding — latex.ltx L18268-18324 defines this kernel CS
  // for legacy 8-bit-encoding compat (used by papers that pre-date the
  // 2018-04-01 default switch to UTF-8). Upstream `\let`s it to `\relax`
  // after first use (L18324); the raw definition is a catcode-mangling
  // loop we must not run. Papers like 1711.09157 and 2403.19280 invoke it
  // at line 1 col 1, BEFORE `\documentclass` — `latex_kernel::
  // autoload_latex_kernel` pulls the format in for them. Define as a
  // no-op so the legacy preamble compiles silently; the encoding-switching
  // behaviour is irrelevant for our XML pipeline.
  Let!("\\UseRawInputEncoding", r"\relax");

  // \DocumentMetadata{<keyval>} — LaTeX 2024 kernel command for PDF
  // accessibility metadata. Author calls it BEFORE `\documentclass`;
  // `latex_kernel::autoload_latex_kernel` loads the LaTeX pool on the
  // undefined CS so this stub is in place by the time it is expanded.
  // The kvopts inside are PDF-only and semantically irrelevant for XML
  // output — gobble the brace group. Witness 2305.08034.
  // Absorb the metadata keyvals but FLIP the kernel's declared flag
  // (latex.ltx L9167/L9173: `\IfDocumentMetadataTF` starts as
  // `\@secondoftwo`, `\DocumentMetadata` lets it to `\@firstoftwo`) —
  // classes guard on it (`\NeedsDocumentMetadata`, ltx-talk.cls L32:
  // "This file needs \DocumentMetadata" ×12 docs).
  // Real `\DocumentMetadata` ALWAYS `\RequirePackage{latex-lab-testphase-latest}`
  // (documentmetadata-support.ltx:72), which always `\RequirePackage{tagpdf}`
  // (latex-lab-testphase-latest.sty:39) — the `tagging=` key only toggles
  // activation, not loading. So load our tagpdf binding here too: its
  // role-namespace props are document content (the tagpdf manual iterates
  // `\g__tag_role_NS_pdf_prop`, tagpdf.tex:2163; on an undefined prop
  // `\prop_map_inline:cn` loses its `\prg_break_point:Nn` and the trailing
  // `\prop_map_break:` runs to EOF — 39-byte XML + Fatal). Guard:
  // `perfect_kernel_batch54::documentmetadata_loads_tagpdf`.
  DefMacro!("\\DocumentMetadata{}",
    "\\global\\let\\IfDocumentMetadataTF\\@firstoftwo\\global\\let\\IfDocumentMetadataT\\@firstofone\\global\\let\\IfDocumentMetadataF\\@gobble\\RequirePackage{tagpdf}");
  // `\DocumentMetadata{tagging=on}` activates the kernel's latex-lab
  // tagging project, whose user surface (`\tagpdfsetup` etc.) exists
  // WITHOUT tagpdf.sty ever loading (tagpdf manuals; tex-vpat). Our XML is
  // the accessible structure — absorb here at kernel level, mirroring the
  // tagpdf_sty.rs binding.
  def_macro_noop("\\tagpdfsetup{}")?;
  // tagpdf.sty `\tagtool{<keyvals>}` (per-structure tagging tweaks) and
  // latex-lab-testphase-block's `\DebugBlocksOn/Off` (debug output for the
  // block tagging code) — both PDF-structure-only, like `\tagpdfsetup`.
  // Witness: tagpdf manual (tagpdf.tex:18,127).
  def_macro_noop("\\tagtool{}")?;
  def_macro_noop("\\DebugBlocksOn")?;
  def_macro_noop("\\DebugBlocksOff")?;
  def_macro_noop("\\tagstructbegin{}")?;
  def_macro_noop("\\tagstructend")?;
  def_macro_noop("\\tagmcbegin{}")?;
  def_macro_noop("\\tagmcend")?;
  // The expl3 layer of the same API — classes written for the tagging
  // project call these directly (ltx-talk.cls uses struct/mc pairs,
  // suspend/resume, tool, get).
  def_macro_noop("\\tag_struct_begin:n{}")?;
  def_macro_noop("\\tag_struct_end:")?;
  def_macro_noop("\\tag_mc_begin:n{}")?;
  def_macro_noop("\\tag_mc_end:")?;
  def_macro_noop("\\tag_suspend:n{}")?;
  def_macro_noop("\\tag_resume:n{}")?;
  def_macro_noop("\\tag_tool:n{}")?;
  def_macro_noop("\\tag_get:n{}")?;

  // Unicode-engine math-code primitives (LuaTeX/XeTeX; LuaTeX manual §7.3).
  // Raw font-setup files probe-and-set them when the engine claims Unicode
  // support (fontsetup's fspdefault.tex under unicode-math; keytheorems/
  // elpres manuals — sweep-16 tail, 4 bundles). Assignments are engine
  // font-table bookkeeping with no XML meaning: absorb the full assignment
  // syntax faithfully so the token stream stays balanced.
  //
  // Defined HERE (post-dump), NOT in pdftex.rs (pre-format): latex.ltx
  // PROBES `\ifx\Umathcode\@undefined` to pick the TU/Unicode branch
  // (L14662), so a pre-format definition makes the FORMAT BUILD masquerade
  // as a Unicode engine — the dump then bakes `\UnicodeEncodingName`=TU and
  // babel-greek raw-loads tuenc-greek.def instead of LGR (greek_test).
  // Same probe-safety class as the \directlua prohibition
  // (project_lua_bridge_directive).
  //   \Umathcode <num>[=]<class><fam><ucode>  /  \Umathchardef <cs>[=]<c><f><u>
  DefPrimitive!("\\Umathcode Number SkipMatch:= Number Number Number", sub[(_a,_b,_c,_d)] {});
  DefPrimitive!("\\Umathchardef DefToken SkipSpaces SkipMatch:= Number Number Number", sub[(cs,_c,_f,_u)] {
    let _ = def_macro(cs, None, ExpansionBody::Tokens(Tokens!()), None);
  });
  DefPrimitive!("\\Umathcharnumdef DefToken SkipSpaces SkipMatch:= Number", sub[(cs,_n)] {
    let _ = def_macro(cs, None, ExpansionBody::Tokens(Tokens!()), None);
  });
  DefPrimitive!("\\Udelcode Number SkipMatch:= Number Number", sub[(_a,_b,_c)] {});

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

  // `\*` — invisible-times (U+2062, MULOP). Perl `TeX.pool.ltxml:7124`:
  //   DefMathI('\*', undef, "\x{2062}", role=>'MULOP', name=>'', meaning=>'times');
  // and our plain layer mirrors it at plain_base.rs:119. Perl's LaTeX
  // emulation NEVER raw-loads latex.ltx's `\DeclareRobustCommand\*`
  // (the discretionary-multiplication `\discretionary{\thinspace\the
  // \textfont2\char2}{}{}`), so `\*` stays the invisible-times DefMath
  // in the latex context. Rust DOES raw-load latex.ltx in
  // `latex_bootstrap`, whose `\*` discretionary clobbers the TeX.pool
  // DefMath and gets baked into the latex dump (as an Expandable macro
  // that simply vanishes in math mode). Re-establish the Perl-faithful
  // invisible-times definition here, post-dump, so `$a\*b$` yields
  // `⁢(a,b)` and — crucially — `$a_\beta\*_\alpha$` makes `\*` the BASE
  // of the second subscript (matching Perl's `<XMTok meaning="times">⁢`
  // base), instead of vanishing and letting `_\alpha` re-attack the
  // prior `_\beta` → spurious "Double subscript". Witness 1909.03262.
  DefMath!("\\*", None, "\u{2062}", role => "MULOP", name => "", meaning => "times");

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
  def_macro_noop("\\@listi")?;
  def_macro_noop("\\@listii")?;
  def_macro_noop("\\@listiii")?;
  def_macro_noop("\\@listiv")?;
  def_macro_noop("\\@listv")?;
  def_macro_noop("\\@listvi")?;
  // size1x.clo:220 `\let\@listI\@listi` — the pristine copy that a class's
  // size commands restore (`\let\@listi\@listI`, size10.clo:53,
  // tufte-common.def:374). Witness: tikz-network manual (tufte-book raw).
  Let!("\\@listI", "\\@listi");
  // `\strutbox` (latex.ltx:12596-12599, rebuilt by `\set@fontsize` from
  // `\baselineskip`): box registers do not survive the dump and our size
  // commands are primitives, so give the 10pt default here — `.7`/`.3` of
  // the 12pt `\baselineskip`. Left void, every `\strut`-based height was 0
  // (fillwith stacked ~200 line coffins instead of ~50 → TokenLimit).
  RawTeX!(r"\setbox\strutbox\hbox{\vrule\@height.7\baselineskip\@depth.3\baselineskip\@width\z@}");

  // expl3 `\tl_set_rescan:Nnn` core (expl3-code.tex:3758-3790): real eTeX
  // sets `\everyeof` to a marker, `\scantokens` the string, and lets
  // `\__tl_rescan:NNw #1#2#3 <marker>` capture EVERYTHING the pseudo-file
  // yielded as a delimited argument — PARAM tokens included — before
  // `\group_end:` and `#1 #2 {#3}`. Our `\scantokens` cannot insert the
  // `\everyeof` payload (wiring it loops the l3doc family — settled dead-end,
  // PLANS P15 / etex.rs), so the delimited scan ran to the pseudo-file end,
  // `read_until` unread the collected tokens and a rescanned macro MEANING
  // (`\cs_meaning:N` → `\long macro:#1#2#3->…`) leaked its `#`s into
  // digestion: substances.sty:452 `\tl_set_rescan:Nnx … {\cs_meaning:N #1}`
  // (substances manual, 720 `misdefined:#`). Perl identical (eTeX.pool
  // `\everyeof` unused; Gullet.pm:683 unread-on-miss). Do the rescan
  // atomically instead: tokenize the string under the CURRENT (group-local,
  // caller-configured) catcodes exactly as the pseudo-file would, and hand
  // the tokens to the unchanged `\__tl_rescan:NNw` protocol with the
  // marker appended — no mouth, no EOF, nothing for a nested scan to cross.
  // `\prg_do_nothing:` stays in front: `\tl_set:No` o-expands it away.
  // The dispatcher's own rule (:3782-3790): content without the
  // `\newlinechar` character is single-line and rescans under
  // `\endlinechar=-1` (no trailing end-of-line token) — a local assignment
  // inside the caller's group, as in expl3.
  DefMacro!("\\__tl_set_rescan:nNN {} Token Token", sub[(content, setter, target)] {
    let text = writable_tokens(&content);
    let register_int = |cs: &str| -> i64 {
      lookup_definition(&T_CS!(cs)).ok().flatten()
        .and_then(|d| d.value_of(Vec::new()))
        .map(|v| v.value_of()).unwrap_or(-1)
    };
    let newline = register_int("\\newlinechar");
    let multi = (0..=255).contains(&newline) && text.contains(newline as u8 as char);
    if !multi {
      assign_register("\\endlinechar", RegisterValue::Number(Number::new(-1)), None, Vec::new())?;
    }
    let mut mouth = Mouth::new(&text, None)?;
    let mut toks = vec![T_CS!("\\__tl_rescan:NNw"), setter, target, T_CS!("\\prg_do_nothing:")];
    while let Some(t) = mouth.read_token() {
      if t.get_catcode() != Catcode::COMMENT {
        toks.push(t);
      }
    }
    toks.extend(do_expand(T_CS!("\\c__tl_rescan_marker_tl"))?.unlist());
    Tokens::new(toks)
  });

  //======================================================================
  // 4. Misc Rust-side stubs
  //======================================================================
  // `\@latexbug` — kernel macro used to mark would-be bug reports.
  // No-op stub.
  def_macro_noop("\\@latexbug")?;

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
  def_macro_noop("\\thebibliography@ID")?;

  //======================================================================
  // 5. Modern LaTeX kernel (2023+) — `\NewCommandCopy`/`\DeclareCommandCopy`/
  //    `\ShowCommand` from `ltcmd.dtx` (semantic-let equivalents).
  //
  // Not in Perl LaTeXML (too new), but needed for modern packages
  // (tcolorbox, etc.).
  //
  // ltcmd defines these as `\NewDocumentCommand … { m m }` — both args
  // are mandatory and accept either a bare token (`\foo`) or a
  // brace-wrapped token (`{\foo}`). Real arxmliv usage is the brace form:
  // `\NewCommandCopy{\origsum}{\sum}` (witness: arXiv:2510.20194
  // various.sty L296). Earlier `Token Token` spec consumed `{` as the
  // first token and `\origsum` as the second, producing `\let { = \origsum`
  // (no-op), leaving `\origsum` undefined and yielding 100+ cascade errors.
  // Use `{}{}` (brace-mandatory) and unwrap to the contained Token.
  //======================================================================
  DefPrimitive!("\\NewCommandCopy{}{}", sub[(new_arg, old_arg)] {
    let new_tok = new_arg.unlist().into_iter().next().ok_or("\\NewCommandCopy: empty new arg")?;
    let old_tok = old_arg.unlist().into_iter().next().ok_or("\\NewCommandCopy: empty old arg")?;
    let_i(&new_tok, &old_tok, None);
  });
  DefPrimitive!("\\DeclareCommandCopy{}{}", sub[(new_arg, old_arg)] {
    let new_tok = new_arg.unlist().into_iter().next().ok_or("\\DeclareCommandCopy: empty new arg")?;
    let old_tok = old_arg.unlist().into_iter().next().ok_or("\\DeclareCommandCopy: empty old arg")?;
    let_i(&new_tok, &old_tok, None);
  });
  def_macro_noop("\\ShowCommand Token")?;

  //======================================================================
  // 6. Modern LaTeX (2015+) extras
  //======================================================================
  // `\extrafloats{N}` — request N extra float slots (no-op in LaTeXML).
  DefPrimitive!("\\extrafloats{}", None);

  // `\wlog{...}` — write to log only (no-op in LaTeXML).
  def_macro_noop("\\wlog{}")?;

  // `\Gin@driver` — pre-defined empty so graphics.sty doesn't error
  // when loaded from disk (LaTeXML doesn't run a Backend driver).
  // Not in Perl source; pure Rust hotfix.
  def_macro_noop("\\Gin@driver")?;

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
  // 8. C.4 label-macros — dump-path coverage
  //
  // Perl latex_base L287-288, L294-296 defines these label macros.
  // Under the dump path (LoadFormat mutual exclusivity) latex_base.rs
  // is SKIPPED, and the dump (resources/dumps/latex.dump.txt) does
  // NOT capture these CSes (raw latex.ltx doesn't define them).
  // Pre-define here so dump-path runs find them too. NODUMP path
  // already gets them from latex_base.rs. Either way, definitions
  // are Perl-faithful values.
  //======================================================================
  DefMacro!("\\appendixname",   "Appendix");
  DefMacro!("\\appendixesname", "Appendixes");
  DefMacro!("\\contentsname",   "Contents");
  DefMacro!("\\listfigurename", "List of Figures");
  DefMacro!("\\listtablename",  "List of Tables");

  // C.5.1 page registers (Perl latex_base L309-311) — same dump-path
  // coverage rationale.
  DefRegister!("\\columnsep"     => Dimension::new(0));
  DefRegister!("\\columnseprule" => Dimension::new(0));
  DefRegister!("\\mathindent"    => Dimension::new(0));

  // C.3.3 footnote counters (Perl latex_base L268-273) — same dump-path
  // coverage rationale. NewCounter is idempotent under dump path so
  // counter-creation is safe.
  NewCounter!("footnote");
  DefMacro!("\\thefootnote", "\\arabic{footnote}");
  NewCounter!("mpfootnote");
  DefMacro!("\\thempfn", "\\thefootnote");
  DefMacro!("\\thempfootnote", "\\arabic{mpfootnote}");
  DefRegister!("\\footnotesep" => Dimension::new(0));

  // C.4.4 / C.5.1 NewCounters (Perl latex_base L300, L312) — dump-path
  // coverage. \@startsection's SetCounter to 3 (in latex_constructs.rs)
  // requires the counter to exist beforehand.
  NewCounter!("tocdepth");
  NewCounter!("secnumdepth");

  // C.5.2 version parsing (Perl latex_base L317-331) — dump-path coverage.
  TeX!(
    r"\def\@ifl@t@r#1#2{%
  \ifnum\expandafter\@parse@version@#1//00\@nil<%
        \expandafter\@parse@version@#2//00\@nil
    \expandafter\@secondoftwo
  \else
    \expandafter\@firstoftwo
  \fi}
\def\@parse@version@#1{\@parse@version0#1}
\def\@parse@version#1/#2/#3#4#5\@nil{%
\@parse@version@dash#1-#2-#3#4\@nil
}
\def\@parse@version@dash#1-#2-#3#4#5\@nil{%
  \if\relax#2\relax\else#1\fi#2#3#4 }"
  );
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
    if lookup_bool_sym(pin!("IN_MATH")) {
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
    skip_spaces()?;
    // Real LaTeX `\filecontents` `\edef`s the filename argument, so
    // `\begin{filecontents}{\jobname-acro.tex}` writes — and a later
    // `\input`/`\loadglsentries` finds — the file under the EXPANDED name
    // (e.g. `root-acro.tex`). Read with Full expansion (not Off) so `\jobname`
    // and friends are resolved; otherwise the content is cached under the
    // literal key `\jobname-acro.tex_contents` and the lookup for
    // `root-acro.tex_contents` misses, leaving the file "not found" and any
    // entries it defines (glossaries acronyms, etc.) undefined. Witness
    // 1905.05350 (`\begin{filecontents}{\jobname-acro.tex}` …
    // `\loadglsentries[\acronymtype]{\jobname-acro}`).
    // filecontents 2019+ takes a leading optional (`[overwrite, nosearch,
    // noheader]{file}` — algxpar-doc L18); without skipping it the CACHE KEY
    // became "[" and the payload was lost. Absorb it before the filename.
    let _opts = read_optional(None)?;
    skip_spaces()?;
    let filename_toks = read_arg(ExpansionLevel::Full)?;
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
    read_raw_line();
    // Read raw lines until the end marker (whole-line match; see
    // capture_raw_lines_until).
    let (captured, _terminator) = capture_raw_lines_until(&[end_marker]);
    lines.extend(captured);
    let n = lines.len();
    Info!("note", "filecontents", s!("Cached filecontents for {filename} ({n} lines)"));
    vfs_store(&filename, &lines.join("\n"));
    Ok(())
  }
  // The \filecontents primitive reads filename + raw lines until \end{filecontents}.
  // When called via \begin{filecontents}, \begin opens a group first, so we manually
  // close the group after caching, matching the \end that was consumed.
  DefPrimitive!("\\filecontents", {
    cache_filecontents("\\end{filecontents}", false)?;
    endgroup()?;
  });
  DefPrimitive!("\\lx@filecontents@star", {
    cache_filecontents("\\end{filecontents*}", true)?;
    endgroup()?;
  });
  assign_meaning(
    &T_CS!("\\filecontents*"),
    lookup_meaning(&T_CS!("\\lx@filecontents@star")).unwrap_or(Stored::None),
    Some(Scope::Global),
  );
  def_macro_noop("\\endfilecontents")?;
  assign_meaning(
    &T_CS!("\\endfilecontents*"),
    lookup_meaning(&T_CS!("\\endfilecontents")).unwrap_or(Stored::None),
    Some(Scope::Global),
  );

    //======================================================================
  // `\cprime` / `\Cprime` / `\cdprime` / `\Cdprime` — REMOVED from the
  // always-on set 2026-07-27 (maintainer decision). They belong to
  // `mathscinet.sty`, which defines them (`mathscinet_sty.rs`), and a
  // definition that is always live can shadow an author's own — the same
  // hazard that kept `\Dbar` package-only from the start.
  //
  // The stub was justified by four papers regaining `undefined:\cprime`
  // without it (2605.00173/.00186/.00190/.00305). Three of those four were
  // artifacts of a defect since fixed: we digested EVERY entry of a `.bib`
  // library, so we met `\cprime` in entries `bibtex(1)` never copies into
  // the `.bbl`. Since #416 we digest only the CITED entries, which removes
  // the trigger structurally rather than papering over it.
  //
  // What a paper needs now is what the real toolchain needs: load
  // `mathscinet` (or `amsrefs`, which requires it), or carry the definition
  // in its own `.bib` `@preamble` — which executes, and is guarded by
  // `bib_preamble_defines_macros_for_the_whole_bibliography`. Witness
  // 2605.11579 (17 uses) is covered by its `@preamble` and is unaffected.
  // See OXIDIZED_DESIGN #78 and `docs/parity/BIBLIOGRAPHY_WORKLIST.md`.
  //======================================================================

  //======================================================================
  // 9. The class-level sectioning workers a raw class or document reaches
  // through `\secdef` (KNOWN_PERL_ERRORS #141). latex.ltx defines none of
  // `\@part`/`\@spart`/`\@chapter`/`\@schapter` — article.cls:281-311 and
  // book.cls:439-475 do — and the bindings that replace those classes never
  // did, so a document that re-`\newcommand`s `\part`/`\chapter` the way the
  // classes write them (`\secdef\@part\@spart`: source3body.tex:96/119 in
  // interface3 + source3, then `\newcounter{chapter}` and its own `\chapter`)
  // errored `undefined:\@part` + one `undefined:\chapter` per chapter (2 → 101
  // errors, fatal). The kernel `\@sect` (latex.ltx:16205; Perl latex_base
  // stubs it to nothing) is the same shape one level down. All route to our
  // `\@startsection` dispatcher with the `[toc]{title}` / `*{title}` tail it
  // reads. `\@ssect` cannot be given (it never sees the heading's name).
  // Guard: `perfect_kernel_batch54::secdef_part_and_chapter_workers_exist`.
  //======================================================================
  TeX!(
    r"
    \def\@part[#1]#2{\@startsection{part}{-1}{}{}{}{}[#1]{#2}}
    \def\@spart#1{\@startsection{part}{-1}{}{}{}{}*{#1}}
    \def\@chapter[#1]#2{\@startsection{chapter}{0}{}{}{}{}[#1]{#2}}
    \def\@schapter#1{\@startsection{chapter}{0}{}{}{}{}*{#1}}
    \def\@sect#1#2#3#4#5#6[#7]#8{\@startsection{#1}{#2}{#3}{#4}{#5}{#6}[#7]{#8}}
    "
  );
});
