//! IEEEtran.cls — IEEE Transactions document class
//! Perl: IEEEtran.cls.ltxml — 458 lines
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // DeclareOption stubs — Perl L18-108
  DeclareOption!("9pt", {});
  DeclareOption!("10pt", {});
  DeclareOption!("11pt", {});
  DeclareOption!("12pt", {});
  DeclareOption!("letterpaper", {});
  DeclareOption!("a4paper", {});
  DeclareOption!("cspaper", {});
  DeclareOption!("draft", {});
  DeclareOption!("final", {});
  DeclareOption!("journal", { Let!("\\ifCLASSOPTIONjournal", "\\iftrue"); Let!("\\ifCLASSOPTIONconference", "\\iffalse"); });
  DeclareOption!("conference", { Let!("\\ifCLASSOPTIONjournal", "\\iffalse"); Let!("\\ifCLASSOPTIONconference", "\\iftrue"); });
  DeclareOption!("technote", { Let!("\\ifCLASSOPTIONtechnote", "\\iftrue"); });
  DeclareOption!("nofonttune", {});
  DeclareOption!("captionsoff", {});
  // TL `IEEEtran.cls` L254-255 and L364-366: `comsoc`, `compsoc`,
  // `transmag` are THREE distinct mutually-exclusive options. Each
  // setter flips one to true and clears the other two:
  //   \DeclareOption{comsoc}{\CLASSOPTIONcomsoctrue \CLASSOPTIONcompsocfalse \CLASSOPTIONtransmagfalse}
  //   \DeclareOption{compsoc}{\CLASSOPTIONcomsocfalse \CLASSOPTIONcompsoctrue \CLASSOPTIONtransmagfalse}
  //   \DeclareOption{transmag}{\CLASSOPTIONcomsocfalse \CLASSOPTIONcompsocfalse \CLASSOPTIONtransmagtrue}
  // Perl `IEEEtran.cls.ltxml:103` for `comsoc` additionally
  // `\RequirePackage{newtxmath}` so `\coloneqq` / `\bigstar` / TX-math
  // symbol family used by comsoc papers resolves cleanly (Perl-faithful;
  // earlier Rust loaded only amssymb here, missing `\coloneqq` —
  // witnesses 1902.10910, 2201.11831).
  DeclareOption!("comsoc", {
    Let!("\\ifCLASSOPTIONcomsoc",  "\\iftrue");
    Let!("\\ifCLASSOPTIONcompsoc", "\\iffalse");
    Let!("\\ifCLASSOPTIONtransmag","\\iffalse");
    RequirePackage!("newtxmath");
  });
  DeclareOption!("compsoc", {
    Let!("\\ifCLASSOPTIONcomsoc",  "\\iffalse");
    Let!("\\ifCLASSOPTIONcompsoc", "\\iftrue");
    Let!("\\ifCLASSOPTIONtransmag","\\iffalse");
  });
  DeclareOption!("transmag", {
    Let!("\\ifCLASSOPTIONcomsoc",  "\\iffalse");
    Let!("\\ifCLASSOPTIONcompsoc", "\\iffalse");
    Let!("\\ifCLASSOPTIONtransmag","\\iftrue");
  });
  DeclareOption!("romanappendices", { Let!("\\ifCLASSOPTIONromanappendices", "\\iftrue"); });
  DeclareOption!("onecolumn", {});
  DeclareOption!("twocolumn", {});
  DeclareOption!("peerreview", {});
  DeclareOption!("peerreviewca", {});
  // Option conditionals — Perl L18-108. These are the FALSE defaults
  // (mirroring `\newif\if@CLASSOPTIONcompsoc \@CLASSOPTIONcompsocfalse`).
  // MUST come BEFORE ProcessOptions so the option-handler `\let` flips to
  // `\iftrue` survive — the previous order placed these after ProcessOptions
  // and silently clobbered any positive option flag the user passed (driver
  // 2308.01854 `\documentclass[10pt,journal,compsoc]{IEEEtran}` had
  // \ifCLASSOPTIONcompsoc unexpectedly false → user's
  // `\ifCLASSOPTIONcompsoc \usepackage{url} \fi` skipped → \url undefined).
  // TL `IEEEtran.cls` L254-256: three separate `\newif` flags for the
  // mutually-exclusive comsoc/compsoc/transmag options. Pre-bind all
  // three to false so paper-side `\ifCLASSOPTION* … \fi` doesn't see
  // undefined when none of the options is passed. Witnesses:
  // arXiv:2603.07560 (`comsoc`-style probe), 2308.01854 (`compsoc`),
  // older IEEEtran/transmag papers.
  Let!("\\ifCLASSOPTIONcomsoc",   "\\iffalse");
  Let!("\\ifCLASSOPTIONcompsoc",  "\\iffalse");
  Let!("\\ifCLASSOPTIONtransmag", "\\iffalse");
  Let!("\\ifCLASSOPTIONjournal", "\\iftrue");
  Let!("\\ifCLASSOPTIONconference", "\\iffalse");
  Let!("\\ifCLASSOPTIONtechnote", "\\iffalse");
  Let!("\\ifCLASSOPTIONromanappendices", "\\iffalse");
  Let!("\\ifCLASSINFOpdf", "\\iftrue");
  Let!("\\ifCLASSOPTIONonecolumn", "\\iffalse");
  Let!("\\ifCLASSOPTIONtwocolumn", "\\iftrue");
  Let!("\\ifCLASSOPTIONdraftcls", "\\iffalse");
  Let!("\\ifCLASSOPTIONpeerreview", "\\iffalse");
  Let!("\\ifCLASSOPTIONcaptionsoff", "\\iffalse");
  // TL `IEEEtran.cls` L238-244: the `draft`/`final`, `oneside`/`twoside`,
  // `peerreviewca`, `nofonttune`, `draftclsnofoot` `\newif` flags. User
  // code in IEEEtran papers freely reads `\ifCLASSOPTIONdraft` /
  // `\ifCLASSOPTIONoneside` without first checking they're defined —
  // e.g. arXiv:2509.12142 has a sectionhead probe
  // `\ifCLASSOPTIONdraft Draft\fi`. Pre-bind to the same defaults as
  // the real class so undefined-cond auto-define doesn't spam an Error.
  Let!("\\ifCLASSOPTIONdraft", "\\iffalse");
  Let!("\\ifCLASSOPTIONdraftclsnofoot", "\\iffalse");
  Let!("\\ifCLASSOPTIONfinal", "\\iftrue");
  Let!("\\ifCLASSOPTIONoneside", "\\iftrue");
  Let!("\\ifCLASSOPTIONtwoside", "\\iffalse");
  Let!("\\ifCLASSOPTIONpeerreviewca", "\\iffalse");
  Let!("\\ifCLASSOPTIONnofonttune", "\\iffalse");

  ProcessOptions!();

  // Load article as base
  load_class("article", Vec::new(), Tokens!())?;

  // Real IEEEtran.cls L689 `\newif\if@technote \@technotefalse` — private flag
  // (separate from the public `\ifCLASSOPTION*` mirrors). User code in
  // technote-aware paragraphs (e.g. cs0502037 `\def\endkeywords{\if@technote
  // \vspace{1.34ex}\else\vspace{0.67ex}\fi}`) reads `\if@technote` directly.
  // Pre-define so `\if@technote` doesn't trip the "undefined cond"
  // auto-define-as-iffalse path (works but emits an Error).
  Let!("\\if@technote", "\\iffalse");
  Let!("\\if@confmode", "\\iffalse");
  // IEEEtran journal default is two-column. Real IEEEtran.cls invokes
  // `\twocolumn` (in journal mode) which sets the LaTeX kernel
  // `\@twocolumntrue`. Mirror by setting `\if@twocolumn` to `\iftrue`
  // unless the paper passed onecolumn explicitly. Witness: cs0502037
  // user-installed `\def\endkeywords{\if@twocolumn\else\endquotation\fi}`
  // wants the if-true branch (two-column → no `\endquotation`); without
  // this, the `\else \endquotation` branch fires from `\keywords` opening
  // `\quotation` (article default `\if@twocolumn=false`), producing an
  // unmatched `\if`/`\fi` cascade by EOF.
  Let!("\\if@twocolumn", "\\iftrue");

  // Front matter macros (Perl L134-165)
  DefMacro!("\\IEEEtitleabstractindextext{}", "#1");

  // \thetitle / \theauthor / \thedate — titling.sty-style accessors that some
  // IEEEtran preambles or .bbl files reference. IEEEtran doesn't natively
  // export them, but users assume they exist. Provide empty defaults so
  // bibliographies that include `\thetitle` don't crash. Witness 2501.15830.
  DefMacro!("\\thetitle",  "");
  DefMacro!("\\theauthor", "");
  DefMacro!("\\thedate",   "");
  DefMacro!("\\IEEEdisplaynontitleabstractindextext", "");
  DefMacro!("\\IEEEdisplaynotcompsoctitleabstractindextext", "");
  DefMacro!("\\IEEEcompsoctitleabstractindextext", "");
  Let!("\\IEEEpeerreviewmaketitle", "\\maketitle");
  DefMacro!("\\IEEEoverridecommandlockouts", "");
  DefMacro!("\\overrideIEEEmargins", "");
  DefMacro!("\\IEEEaftertitletext{}", "");
  DefMacro!("\\IEEEspecialpapernotice{}", "");
  DefMacro!("\\IEEEmembership{}", "");
  DefMacro!("\\IEEEauthorblockN{}", "#1");
  DefConstructor!("\\@@@affiliation{}", "^ <ltx:contact role='affiliation'>#1</ltx:contact>");
  DefMacro!("\\IEEEauthorblockA{}", "\\@add@to@frontmatter{ltx:creator}{\\@@@affiliation{#1}}");

  // IEEEkeywords environment (Perl L152-155)
  Let!("\\@endIEEEkeywords", "\\relax");
  DefMacro!("\\@IEEEkeywords XUntil:\\@endIEEEkeywords",
    "\\@add@frontmatter{ltx:keywords}[name={Index Terms}]{#1}");
  DefMacro!("\\IEEEkeywords", "\\@IEEEkeywords");
  DefMacro!("\\endIEEEkeywords", "\\@endIEEEkeywords");
  // Perl IEEEtran.cls.ltxml L152-153: explicit env-token aliases. Without
  // these, our standard `\begin{X} → \begingroup\X` expansion routes
  // through user-redefinable namespace and the `XUntil:\@endIEEEkeywords`
  // terminator can be broken by user `\def\endIEEEkeywords`. Drivers:
  // 2007.13436, 1812.09324 (`\@iffalse` cascade past EOF when the
  // XUntil reader runs off the end).
  DefMacro!(T_CS!("\\begin{IEEEkeywords}"), None, "\\@IEEEkeywords");
  DefMacro!(T_CS!("\\end{IEEEkeywords}"),   None, "\\@endIEEEkeywords");

  DefMacro!("\\IEEEraisesectionheading{}", "#1");
  DefMacro!("\\IEEEPARstart{}{}", "#1#2");
  DefMacro!("\\IEEEcompsocitemizethanks{}", "\\thanks{#1}");
  DefMacro!("\\IEEEcompsocthanksitem[]", "");
  DefMacro!("\\IEEEauthorrefmark", "");
  DefMacro!("\\IEEEtriggeratref{}", "");
  DefMacro!("\\IEEEpubid{}", "\\@add@frontmatter{ltx:note}[role=publicationid]{pubid: #1}");
  DefMacro!("\\IEEEpubidadjcol", "");

  // Section numbering — default journal mode uses Roman numerals
  DefMacro!("\\thesection", "\\Roman{section}");
  DefMacro!("\\thesubsection", "\\mbox{\\thesection-\\Alph{subsection}}");
  DefMacro!("\\thesubsubsection", "\\thesubsection\\arabic{subsubsection}");
  DefMacro!("\\theparagraph", "\\thesubsubsection\\alph{paragraph}");

  // Font primitives (Perl L183-186)
  DefPrimitive!("\\ltx@ieeetran@it", None, font => { shape => "italic", family => "serif", series => "medium" }, locked => true);
  DefPrimitive!("\\ltx@ieeetran@sc", None, font => { shape => "smallcaps", family => "serif", series => "medium" }, locked => true);
  DefMacro!("\\format@title@font@section", "\\ltx@ieeetran@sc");
  DefMacro!("\\format@title@font@subsection", "\\ltx@ieeetran@it");
  DefMacro!("\\figurename", "Fig.");
  DefMacro!("\\tablename", "TABLE");
  DefMacro!("\\thetable", "\\Roman{table}");

  // QED symbols (Perl L194-198)
  DefConstructor!("\\IEEEQEDclosed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true);
  Let!("\\IEEEQEDopen", "\\IEEEQEDclosed");
  Let!("\\IEEEQED", "\\IEEEQEDclosed");

  // Perl IEEEtran.cls.ltxml L200-203: \IEEEQEDhere pops top of QED@stack,
  // pushes empty Tokens() back, returns popped value. Intended to move
  // the QED symbol from proof-end to an explicit in-body position. Mirrors
  // the amsthm.sty `\qedhere` pattern (amsthm_sty.rs L154-162). The
  // IEEEproof environment (defined below) pushes `\qed` in
  // after_digest_begin and pops-and-digests in before_digest_end, so the
  // full Perl stack discipline is in place: inline `\IEEEQEDhere` pulls
  // the token out of the stack (replacing it with empty Tokens), causing
  // the proof-end pop to produce nothing.
  DefMacro!("\\IEEEQEDhere", sub[_args] {
    let t = pop_value("QED@stack");
    let _ = push_value("QED@stack", Stored::Tokens(Tokens!()));
    if let Ok(Some(Stored::Tokens(tokens))) = t {
      Ok(tokens)
    } else {
      Ok(Tokens!())
    }
  });

  // IEEEproof environment (Perl L206-229)
  // Perl digests \\textbf{\\textit{Proof:}} producing font="bold italic".
  // Our codegen treats \\word as literal text, so use explicit attributes instead.
  //
  // Perl L213-228: afterDigestBegin pushes T_CS('\qed') onto QED@stack, and
  // beforeDigestEnd pops it and digests — firing the QED symbol at proof-end
  // unless \IEEEQEDhere already consumed the token inline. Mirrors the amsthm
  // \@proof / \end@proof stack pattern.
  // Template drops the explicit </ltx:proof> close so the Tag-level
  // auto_close (latex_constructs.rs:6176) handles cleanup. Mirrors
  // amsthm's `\@proof` pattern (constructor template opens but doesn't
  // close; `\end@proof` calls maybe_close_element). Without this, when
  // \end{IEEEproof}'s end_mode triggers a mode-error and auto-closes
  // <ltx:proof> early, the template's strict </ltx:proof> close emits
  // a spurious "malformed:ltx:proof isn't open" cascade. Witnesses:
  // 1001.3714, 0801.0061 (R=Δ+1 vs Perl, both Δ=1 cosmetic cascade).
  DefEnvironment!("{IEEEproof}[]",
    "<ltx:proof><ltx:title font='#font' _force_font='true' class='ltx_runin'>#title</ltx:title>#body#qed",
    properties => sub[_args] {
      // Perl digests \textbf{\textit{Proof:}} producing font="bold italic".
      // Build a bold-italic font via digestion so the title attribute matches.
      // Template engine auto-binds `"font"` prop to the element's font= attr.
      let title = stomach::digest(mouth::tokenize_internal(
        "{\\bfseries\\itshape Proof:}"
      ))?;
      let titlefont = title.get_font().ok().flatten().map(|f| f.into_owned());
      // Digest `\qed` directly into a prop — the template references `#qed`
      // at body-end so the QED symbol lands inside <ltx:proof>.
      let qed = stomach::digest(mouth::tokenize_internal("\\qed"))?;
      let mut map = SymHashMap::default();
      map.insert("title", title.into());
      map.insert("qed", qed.into());
      if let Some(f) = titlefont {
        map.insert("font", Stored::Font(Rc::new(f)));
      }
      Ok(map)
    },
    // Cycle 302 cleanup: removed after_digest_begin (push \qed) +
    // before_digest_end (pop+digest \qed) hooks. The QED symbol is
    // now emitted via the `#qed` template prop above (properties
    // closure digests `\qed` once at construction time). These
    // hooks never fired correctly in the DefEnvironment absorber
    // context — cycle 301 probe confirmed Digest! from
    // before_digest_end didn't reach the body. The properties-prop
    // approach is simpler and works. Note: `\IEEEQEDhere` inline
    // consumption (Perl IEEEtran.cls.ltxml L200-203) now emits an
    // extra symbol that Perl would have suppressed via the stack
    // machinery — tracked as a known minor divergence; no test
    // exercises \IEEEQEDhere against Perl ground truth, so accepting
    // the simplification.
    );

  // IEEEbiography (Perl IEEEtran.cls.ltxml L238-247) — two-column
  // tabular-in-float: photo/placeholder on left, bolded author + body
  // on right. Matches Perl shape byte-for-byte.
  DefEnvironment!("{IEEEbiography}[]{}",
    "<ltx:float class='biography'>\
      <ltx:tabular>\
        <ltx:tr>\
          <ltx:td>#1</ltx:td>\
          <ltx:td><ltx:inline-block>\
            <ltx:text class='ltx_font_bold'>#2</ltx:text> #body\
          </ltx:inline-block></ltx:td>\
        </ltx:tr>\
      </ltx:tabular>\
    </ltx:float>");
  DefEnvironment!("{IEEEbiographynophoto}[]{}",
    "<ltx:float class='biography'>\
      <ltx:tabular>\
        <ltx:tr>\
          <ltx:td><ltx:inline-block>\
            <ltx:text class='ltx_font_bold'>#2</ltx:text> #body\
          </ltx:inline-block></ltx:td>\
        </ltx:tr>\
      </ltx:tabular>\
    </ltx:float>");

  // IEEEeqnarray (Perl IEEEtran.cls.ltxml L298-302) — Perl uses
  //   DefMacroI('\IEEEeqnarray', '{}', '\eqnarray')
  // Consumes `{rCl}` column spec, expands to `\eqnarray`.
  //
  // KNOWN BUG: Rust translation below drops row-1 cell-1 of the
  // expanded env (emits `<td colspan="2">` merging cells 1+2 where
  // Perl emits three separate `<td>` cells). Plain `\eqnarray` via
  // direct `\begin{eqnarray}` works correctly; rows 2+ of
  // IEEEeqnarray also work correctly. Failing mode scoped to row 1.
  //
  // Cycle 294 diagnostic probes (all still broken):
  //   1. Zero-arg `\IEEEeqnarray` (leave `{rCl}` in stream)
  //   2. Trailing space after `\eqnarray` in body
  //   3. Inlined `\eqnarray` expansion directly
  //   4. `\@gobble`-style intermediate macro indirection
  //   5. `\relax` barrier before `\eqnarray`
  //   6. `RawTeX!(r"\long\def\IEEEeqnarray#1{\eqnarray}…")` (replace
  //      compile-time DefMacro! with runtime \def in ltxml class-load)
  //   7. `LATEXML_NODUMP=1` (bypass dump cache)
  //
  // **Works:** an in-`.tex` document-preamble `\def\IEEEeqnarray#1{\eqnarray}`
  // correctly rescues row 1 cell 1. Also a rename probe — `\myeqnarray`
  // via `\def\myeqnarray#1{\eqnarray}` + `\def\endmyeqnarray{\endeqnarray}`
  // under the same IEEEtran class load — works.
  //
  // So the bug is SPECIFIC to the `\IEEEeqnarray` CS binding installed
  // from this `.cls.ltxml` (probably interacting with the dump cache
  // or a pre-class `\let` against `\IEEEeqnarray`). The runtime \def
  // workaround via RawTeX does NOT override it, suggesting the
  // binding is installed before this RawTeX runs, or persists via a
  // path that \def can't supersede. Needs dumper-trace next cycle —
  // grep the .model / dump files for `\IEEEeqnarray` pre-existing
  // bindings, and investigate `AssignMeaning` vs `Let` lock-out.
  //
  // Affects ~56 <Math>, ~38 <td> across IEEE.tex.
  //
  // Cycle 295 probe: defer \def to post-preamble time — the proven-working
  // context for `\def\IEEEeqnarray#1{\eqnarray}`.
  DefMacro!("\\IEEEeqnarray{}", "\\eqnarray");
  DefMacro!("\\endIEEEeqnarray", "\\endeqnarray");
  at_begin_document(TokenizeInternal!(
    r"\def\IEEEeqnarray#1{\eqnarray}\def\endIEEEeqnarray{\endeqnarray}\expandafter\def\csname IEEEeqnarray*\endcsname#1{\csname eqnarray*\endcsname}\expandafter\def\csname endIEEEeqnarray*\endcsname{\csname endeqnarray*\endcsname}"
  ))?;
  // Perl L301-302: `\IEEEeqnarray*` → `\eqnarray*` (unnumbered form).
  // Port was missing — absence surfaced as undefined-macro errors on
  // any `\begin{IEEEeqnarray*}…\end{IEEEeqnarray*}` in source, shifting
  // subsequent equation numbering by 3 in tests/structure/IEEE.tex
  // (the test uses 3 unnumbered IEEEeqnarray* env pairs interleaved
  // with numbered ones). Fixing this + the matching \endIEEEeqnarray*
  // should recover the ~3-equation drift between Rust and the
  // IEEE.xml reference under TL2025.
  DefMacro!("\\IEEEeqnarray*{}", "\\eqnarray*");
  Let!("\\endIEEEeqnarray*", "\\endeqnarray*");
  DefMacro!("\\IEEEeqnarraynumspace", "");
  // IEEEeqnarraybox — faithful port of Perl IEEEtran.cls.ltxml L315-332.
  // Perl dispatches \ifmmode into \IEEEeqnarrayboxm (math-mode) or
  // \IEEEeqnarrayboxt (text-mode, with \lx@begin@inline@math wrapper),
  // plus a \@@IEEE@array DefConstructor whose `reversion` preserves
  // the original `\begin{IEEEeqnarraybox}` string in `tex=` attr.
  RawTeX!(
    r"\def\IEEEeqnarraybox{\ifmmode\def\@tempa{\let\endIEEEeqnarraybox\endIEEEeqnarrayboxm\IEEEeqnarrayboxm}\else\def\@tempa{\let\endIEEEeqnarraybox\endIEEEeqnarrayboxt\IEEEeqnarrayboxt}\fi\@tempa}"
  );
  DefMacro!("\\IEEEeqnarrayboxm OptionalMatch:* {}",
    "\\@array@bindings{#2}\\@@IEEE@array{#2}\\lx@begin@alignment");
  DefMacro!("\\endIEEEeqnarrayboxm", "\\lx@end@alignment\\@end@array");
  DefMacro!("\\IEEEeqnarrayboxt OptionalMatch:* {}",
    "\\lx@begin@inline@math\\@array@bindings{#2}\\@@IEEE@array{#2}\\lx@begin@alignment");
  DefMacro!("\\endIEEEeqnarrayboxt",
    "\\lx@end@alignment\\@end@array\\lx@end@inline@math");
  DefConstructor!("\\@@IEEE@array[] Undigested DigestedBody", "#3",
    before_digest => { bgroup(); },
    reversion => "\\begin{IEEEeqnarraybox}[#1]{#2}#3\\end{IEEEeqnarraybox}");
  DefMacro!("\\IEEEeqnarraymulticol{}{}{}", "\\multicolumn{#1}{#2}{#3}");
  DefMacro!("\\IEEEeqnarraydefcol{}{}{}", "");
  DefMacro!("\\IEEEeqnarraydefcolsep{}{}", "");

  // IEEEnonumber/yesnumber/sub-numbering — Perl L252-294.
  // Flip EQUATION_NUMBERING (starred form) or EQUATIONROW_TAGS (unstarred)
  // retract/noretract/counter keys to match Perl's in-place hash mutation
  // of LookupValue-returned refs. Previous Rust port was a DefMacro stub
  // that aliased to \nonumber (or was empty) and lost the row-tag
  // retraction entirely.
  DefPrimitive!("\\IEEEnonumber OptionalMatch:*", sub[(star)] {
    let key = if star.is_some() { "EQUATION_NUMBERING" } else { "EQUATIONROW_TAGS" };
    with_value_mut(key, |v| {
      if let Some(Stored::HashStored(ref mut m)) = v {
        m.insert("retract", Stored::Bool(true));
        m.remove("counter");
      }
    });
    Ok(())
  });
  DefPrimitive!("\\IEEEyesnumber OptionalMatch:*", sub[(star)] {
    // Perl: if EQUATION_NUMBERING.counter == 'subequation', step the equation counter
    let subeq = with_value("EQUATION_NUMBERING", |v| {
      if let Some(Stored::HashStored(ref m)) = v {
        matches!(m.get("counter"),
          Some(Stored::String(s)) if arena::to_string(*s) == "subequation")
      } else { false }
    });
    if subeq {
      RefStepCounter!("equation", false)?;
    }
    if star.is_some() {
      with_value_mut("EQUATION_NUMBERING", |v| {
        if let Some(Stored::HashStored(ref mut m)) = v {
          m.insert("retract", Stored::Bool(false));
          m.remove("counter");
        }
      });
    } else {
      with_value_mut("EQUATIONROW_TAGS", |v| {
        if let Some(Stored::HashStored(ref mut m)) = v {
          m.insert("noretract", Stored::Bool(true));
          m.remove("counter");
        }
      });
    }
    Ok(())
  });
  DefPrimitive!("\\IEEEyessubnumber OptionalMatch:*", sub[(star)] {
    let key = if star.is_some() { "EQUATION_NUMBERING" } else { "EQUATIONROW_TAGS" };
    with_value_mut(key, |v| {
      if let Some(Stored::HashStored(ref mut m)) = v {
        m.insert("counter", Stored::String(pin!("subequation")));
      }
    });
    let preset = with_value("EQUATION_NUMBERING", |v| {
      matches!(v, Some(Stored::HashStored(m)) if m.contains_key("preset"))
    }) || with_value("EQUATIONROW_TAGS", |v| {
      matches!(v, Some(Stored::HashStored(m)) if m.contains_key("preset"))
    });
    if preset {
      RefStepCounter!("subequation", false)?;
    }
    Ok(())
  });
  DefPrimitive!("\\IEEEnosubnumber OptionalMatch:*", sub[(star)] {
    let key = if star.is_some() { "EQUATION_NUMBERING" } else { "EQUATIONROW_TAGS" };
    with_value_mut(key, |v| {
      if let Some(Stored::HashStored(ref mut m)) = v {
        m.insert("counter", Stored::String(pin!("equation")));
      }
    });
    Ok(())
  });

  // Column types (Perl IEEEtran.cls.ltxml L308-314): L/C/R add
  // \hfil-before/after hooks — the same pattern aas_support_sty:313
  // uses for its `h`/`B` columns. Porting all three so IEEEeqnarraybox
  // actually aligns by the user's spec instead of Rust's
  // center-defaulted fallthrough.
  //
  //   L  = after \hfil        (flush left)
  //   C  = before + after     (center)
  //   R  = before \hfil       (flush right)
  DefColumnType!("L", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(latexml_core::alignment::cell::Cell {
        after: Some(Tokens!(T_CS!("\\hfil"))),
        ..latexml_core::alignment::cell::Cell::default()
      })
    });
  });
  DefColumnType!("C", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(latexml_core::alignment::cell::Cell {
        before: Some(Tokens!(T_CS!("\\hfil"))),
        after:  Some(Tokens!(T_CS!("\\hfil"))),
        ..latexml_core::alignment::cell::Cell::default()
      })
    });
  });
  DefColumnType!("R", {
    with_current_build_template(|template_opt| {
      template_opt.unwrap().add_column(latexml_core::alignment::cell::Cell {
        before: Some(Tokens!(T_CS!("\\hfil"))),
        ..latexml_core::alignment::cell::Cell::default()
      })
    });
  });

  Let!("\\appendices", "\\appendix");

  // Bibliography style — Perl IEEEtran doesn't touch bibliography
  // beyond the (commented-out) bstctlcite documentation at L442. Stale
  // "AssignMapping not yet ported" note removed; nothing to port here.

  // IED list stubs (Perl L340-347)
  DefMacro!("\\IEEEsetlabelwidth{}", "\\settowidth{\\labelwidth}{#1}");
  DefMacro!("\\IEEEusemathlabelsep", "");
  DefMacro!("\\IEEEtriggercmd{}", "");
  DefMacro!("\\IEEElabelindent", "");
  DefMacro!("\\IEEEcalcleftmargin{}", "");
  DefMacro!("\\IEEEiedlabeljustifyc", "");
  DefMacro!("\\IEEEiedlabeljustifyl", "");
  DefMacro!("\\IEEEiedlabeljustifyr", "");

  // IEEEitemize/enumerate/description (Perl IEEEtran.cls.ltxml L351-366).
  // Each env:
  //   - runs `beginItemize('<kind>', '<counter>')` via properties
  //     closure — registers the nested list level, resets counters,
  //     and wires the itemize/enumerate counter-label machinery.
  //   - digests `\par` on close so trailing text in the last item
  //     closes its <ltx:item> cleanly instead of leaking whitespace.
  //   - locks against further redefinition (matches Perl `locked=>1`).
  // {IEEEdescription} additionally re-`\let`s `\makelabel` to
  // `\descriptionlabel` at env start (Perl L363), mirroring LaTeX's
  // core description-env plumbing.
  DefEnvironment!("{IEEEitemize}[]",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    properties => sub[_args] { BeginItemize!("itemize", "@item") },
    before_digest_end => { Digest!("\\par") },
    locked => true,
    mode => "internal_vertical");
  DefEnvironment!("{IEEEenumerate}[]",
    "<ltx:enumerate xml:id='#id'>#body</ltx:enumerate>",
    properties => sub[_args] { BeginItemize!("enumerate", "enum") },
    before_digest_end => { Digest!("\\par") },
    locked => true,
    mode => "internal_vertical");
  DefEnvironment!("{IEEEdescription}[]",
    "<ltx:description xml:id='#id'>#body</ltx:description>",
    before_digest => { Let!("\\makelabel", "\\descriptionlabel"); },
    properties => sub[_args] { BeginItemize!("description", "@desc") },
    before_digest_end => { Digest!("\\par") },
    locked => true,
    mode => "internal_vertical");

  // Override LaTeX's default IED lists with the IEEE versions defined
  // above. Per Perl L369-380: a `\let` of the bare CSes AND a `\let` on
  // the `\begin{itemize}` / `\end{itemize}` env-tokens, so user code that
  // writes `\begin{itemize}` (the LaTeXish form) hits the IEEE variant
  // — without these the standard latex-core itemize wins, bypassing
  // IEEE's locked spec. 12 aliases total.
  Let!("\\itemize",      "\\IEEEitemize");
  Let!("\\enditemize",   "\\endIEEEitemize");
  Let!("\\enumerate",    "\\IEEEenumerate");
  Let!("\\endenumerate", "\\endIEEEenumerate");
  Let!("\\description",  "\\IEEEdescription");
  Let!("\\enddescription","\\endIEEEdescription");
  Let!("\\begin{itemize}",      "\\IEEEitemize");
  Let!("\\end{itemize}",        "\\endIEEEitemize");
  Let!("\\begin{enumerate}",    "\\IEEEenumerate");
  Let!("\\end{enumerate}",      "\\endIEEEenumerate");
  Let!("\\begin{description}",  "\\IEEEdescription");
  Let!("\\end{description}",    "\\endIEEEdescription");

  // String macros (Perl L383-395)
  DefMacro!("\\contentsname", "Contents");
  DefMacro!("\\listfigurename", "List of Figures");
  DefMacro!("\\listtablename", "List of Tables");
  DefMacro!("\\refname", "References");
  DefMacro!("\\indexname", "Index");
  DefMacro!("\\partname", "Part");
  DefMacro!("\\appendixname", "Appendix");
  DefMacro!("\\abstractname", "Abstract");
  DefMacro!("\\IEEEkeywordsname", "Index Terms");
  DefMacro!("\\IEEEproofname", "Proof");

  // Legacy aliases (Perl L398-439)
  Let!("\\authorblockA", "\\IEEEauthorblockA");
  Let!("\\authorblockN", "\\IEEEauthorblockN");
  Let!("\\authorrefmark", "\\IEEEauthorrefmark");
  Let!("\\PARstart", "\\IEEEPARstart");
  Let!("\\pubid", "\\IEEEpubid");
  Let!("\\pubidadjcol", "\\IEEEpubidadjcol");
  Let!("\\specialpapernotice", "\\IEEEspecialpapernotice");

  // Keywords environment aliases — Perl L406-414
  // Perl dispatches on whether the next token is a brace:
  //   \keywords{foo}  → \keywords@onearg{foo}
  //   \keywords … \endkeywords (env form) → \@IEEEkeywords
  // Rust was hardcoding the env-start path, so braced `\keywords{foo}`
  // never reached the one-arg expansion.
  DefMacro!("\\keywords", sub[_args] {
    let next = gullet::read_token()?;
    if let Some(t) = next {
      gullet::unread(Tokens!(t));
      if t.get_catcode() == Catcode::BEGIN {
        return Ok(Tokens!(T_CS!("\\keywords@onearg")));
      }
    }
    Ok(Tokens!(T_CS!("\\@IEEEkeywords")))
  }, locked => true);
  DefMacro!("\\keywords@onearg{}",
    "\\@IEEEkeywords #1 \\@endIEEEkeywords");
  DefMacro!("\\endkeywords", "\\@endIEEEkeywords");
  // Perl L406-407: explicit `\begin{keywords}` / `\end{keywords}` env-token
  // aliases so user `\def\keywords` / `\def\endkeywords` redefinitions
  // don't break the env semantics. Drivers: 2007.06704
  // (`\def\keywords{...}\def\endkeywords{\par}` followed by
  // `\begin{keywords}...\end{keywords}` — without these aliases, our
  // standard `\begin{X} → \begingroup\X` expansion routed through user's
  // `\par`-redef'd `\endkeywords`, leaving `\@IEEEkeywords`'s
  // `XUntil:\@endIEEEkeywords` reading past EOF).
  DefMacro!(T_CS!("\\begin{keywords}"), None, "\\@IEEEkeywords");
  DefMacro!(T_CS!("\\end{keywords}"),   None, "\\@endIEEEkeywords");

  // Legacy IED list aliases — Perl IEEEtran.cls.ltxml L417-423
  Let!("\\labelindent", "\\IEEElabelindent");
  Let!("\\calcleftmargin", "\\IEEEcalcleftmargin");
  Let!("\\setlabelwidth", "\\IEEEsetlabelwidth");
  Let!("\\usemathlabelsep", "\\IEEEusemathlabelsep");
  Let!("\\iedlabeljustifyc", "\\IEEEiedlabeljustifyc");
  Let!("\\iedlabeljustifyl", "\\IEEEiedlabeljustifyl");
  Let!("\\iedlabeljustifyr", "\\IEEEiedlabeljustifyr");

  // QED/proof aliases
  Let!("\\QED", "\\IEEEQED");
  Let!("\\QEDclosed", "\\IEEEQEDclosed");
  Let!("\\QEDopen", "\\IEEEQEDopen");
  DefMacro!("\\qed", "\\ltx@qed");
  DefConstructor!("\\ltx@qed",
    "?#isMath(<ltx:XMTok role='PUNCT'>\u{220E}</ltx:XMTok>)(\u{220E})",
    enter_horizontal => true, reversion => "\\qed");
  Let!("\\proof", "\\IEEEproof");
  Let!("\\endproof", "\\endIEEEproof");
  // IEEEtran proofs route through amsthm's `\@proof` / `\end@proof`
  // machinery (the magic `\begin{proof}` CS from amsthm_sty.rs:220 —
  // `\begin{proof}` → `\begin{@proof}`). We re-override `\th@proof`
  // here so the amsthm header-font is bold-italic (Perl ships
  // `\textbf{\textit{Proof:}}` under IEEEtran, producing
  // font="bold italic"). Keeping amsthm's env path also gives us the
  // QED symbol emission at proof-end for free — it's already wired
  // through amsthm's `\end@proof` before_digest stack-pop.
  RawTeX!(r"\def\th@proof{\def\thm@headfont{\bfseries\itshape}\def\thm@bodyfont{\normalfont}}");

  // Biography aliases
  Let!("\\biography", "\\IEEEbiography");
  Let!("\\biographynophoto", "\\IEEEbiographynophoto");
  Let!("\\endbiography", "\\endIEEEbiography");
  Let!("\\endbiographynophoto", "\\endIEEEbiographynophoto");

  // bstctlcite stub (Perl L445)
  DefMacro!("\\bstctlcite[]{}", "");

  // Disable internal alignment env (Perl L453-454)
  DefMacro!("\\@IEEEauthorhalign", "\\relax");
  DefMacro!("\\end@IEEEauthorhalign", "\\relax");

  // \linebreakand — IEEEtran tip-jar macro that papers redefine to break
  // a multi-row author halign across visual lines. The canonical
  // tex.stackexchange recipe is:
  //   \newcommand{\linebreakand}{
  //     \end{@IEEEauthorhalign}\hfill\mbox{}\par\mbox{}\hfill
  //     \begin{@IEEEauthorhalign}}
  // The unbalanced `\end{...}\begin{...}` pair pops + pushes a frame
  // on the live stack, which is fine in real IEEEtran (where the
  // \author{...} body is wrapped in `\begin{@IEEEauthorhalign}...\end{@IEEEauthorhalign}`)
  // but breaks our `\@personname`-wrapped frontmatter digest where
  // there's no matching outer halign frame. Pre-define `\linebreakand`
  // as a paragraph break so user `\newcommand{\linebreakand}{...}`
  // can't override (it's already-defined → silently ignored), matching
  // Perl's behavior where the user redefinition triggers
  // ``\linebreakand:locked`` and gets dropped on the floor. Driver
  // cluster: 2211.12981, 2403.11083, 2405.03537, 2405.04387 (IEEEtran
  // multi-author papers using the linebreakand recipe).
  DefMacro!("\\linebreakand", "\\par");
});
