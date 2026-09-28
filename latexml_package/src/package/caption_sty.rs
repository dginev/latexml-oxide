use crate::prelude::*;

/// Environments whose body is read VERBATIM from the raw input, so `\captionof`
/// must not open one to host its caption — the terminator it would need is in
/// the token stream, where such an environment never looks. See
/// `\@captionof@` below.
const VERBATIM_BODY_ENVS: &[&str] = &[
  "lstlisting",
  "lstlisting*",
  "verbatim",
  "verbatim*",
  "Verbatim",
  "Verbatim*",
  "BVerbatim",
  "LVerbatim",
  "SaveVerbatim",
  "minted",
  "alltt",
];

#[rustfmt::skip]
LoadDefinitions!({
  // caption3.sty:209: `\DeclareCaptionOption` defines its keys with keyval's `\define@key`.
  RequirePackage!("keyval");
  // caption.sty:170 `\let\AtCaptionPackage\@firstofone`: code a class hands it runs
  // at once, caption being loaded (langscibook.cls; class census 2026-09-24).
  Let!("\\AtCaptionPackage", "\\@firstofone");
  // Perl: caption.sty.ltxml
  // Basically all of this is ignorable (other than needing the macros defined).
  // In principle, we could make use of some of the fonts...

  // Perl L24-59: DefKeyVal declarations for caption package
  DefKeyVal!("caption", "format", "", "");
  DefKeyVal!("caption", "indentation", "Dimension", "0pt");
  DefKeyVal!("caption", "labelformat", "", "default");
  DefKeyVal!("caption", "labelsep", "", "");
  DefKeyVal!("caption", "textformat", "", "");
  DefKeyVal!("caption", "justification", "", "");
  DefKeyVal!("caption", "singlelinecheck", "", "");
  DefKeyVal!("caption", "font", "", "");
  DefKeyVal!("caption", "labelfont", "", "");
  DefKeyVal!("caption", "textfont", "", "");
  DefKeyVal!("caption", "font+", "", "");
  DefKeyVal!("caption", "labelfont+", "", "");
  DefKeyVal!("caption", "textfont+", "", "");
  DefKeyVal!("caption", "margin", "Dimension", "0pt");
  DefKeyVal!("caption", "margin*", "Dimension", "0pt");
  DefKeyVal!("caption", "minmargin", "Dimension", "0pt");
  DefKeyVal!("caption", "maxmargin", "Dimension", "0pt");
  DefKeyVal!("caption", "parskip", "Dimension", "0pt");
  DefKeyVal!("caption", "width", "Dimension", "0pt");
  DefKeyVal!("caption", "oneside", "", "");
  DefKeyVal!("caption", "twoside", "", "");
  DefKeyVal!("caption", "hangindent", "Dimension", "0pt");
  DefKeyVal!("caption", "style", "", "");
  DefKeyVal!("caption", "skip", "Dimension", "0pt");
  DefKeyVal!("caption", "position", "", "");
  DefKeyVal!("caption", "figureposition", "", "");
  DefKeyVal!("caption", "tableposition", "", "");
  DefKeyVal!("caption", "list", "", "");
  DefKeyVal!("caption", "listformat", "", "");
  DefKeyVal!("caption", "name", "", "");
  DefKeyVal!("caption", "type", "", "");
  // caption.sty:284: `type*` sets the type as `type` does, without the anchor.
  DefKeyVal!("caption", "type*", "", "");
  // Additional caption.sty options not in Perl's pre-registration list.
  // Rust-only divergence paired with `21e730e71e` Info→Warn promotion.
  for key in [
    "compatibility", "calcmargin", "ignoreLTcapwidth",
    "captionlinewidth", "subrefformat",
    "subskip", "belowskip", "aboveskip",
    "rule", "tableposition", "labelseparator",
    "options", "ruled", "boxed",
    "above", "below", "outside", "inside",
    "centerlast", "centering", "raggedright", "raggedleft",
    // caption.sty/caption3.sty (TL 2025) `\DeclareCaptionOption` keys missing
    // above: `hypcap` warned in 3 papers of arXiv 2605 (2605.14865, the NeurIPS
    // and ACL templates' `\usepackage[hypcap=true]{caption}`).
    "box", "boxcolor", "boxsep", "calcwidth", "config", "debug", "figurename",
    "figurewithin", "FPlist", "FPref", "hypcap", "hypcapspace", "indent",
    "indention", "list-entry", "listfigurename", "listof", "listtablename",
    "listtype", "listtype+", "lofdepth", "lotdepth", "parbox", "parindent",
    "size", "slc", "strut", "tablename", "tablewithin", "within",
  ] {
    DefKeyVal!("caption", key, "");
  }

  // A key `\DeclareCaptionOption` declared (see below), which `\captionsetup` runs.
  DefPrimitive!("\\lx@caption@declared{}", sub[(key)] {
    assign_value(&s!("CAPTION_DECLARED_{}", key.to_string()), Stored::Bool(true), Some(Scope::Global));
  });

  // Perl L62-68: \captionsetup stores key-value pairs as CAPTION_{key}
  // in state. Perl uses `RequiredKeyVals:caption` so brace-nested and
  // quoted values parse correctly; the prior Rust version accepted
  // `{}` and manually split on `,`, which mis-parsed values containing
  // commas inside braces (e.g. `font={normal,bold}`).
  // Perl L62-68: \captionsetup stores key-value pairs as CAPTION_{key}
  // in state. Supports optional * and single or double optional argument:
  // \captionsetup*[<type>][<subtype>]{<keyvals>}
  // Used by bicaption, subcaption, etc. (e.g. \captionsetup[figure][bi-second]{name=Figure})
  DefPrimitive!(
    "\\captionsetup OptionalMatch:* [] [] RequiredKeyVals:caption",
    sub[(_star, type_opt, subtype_opt, kv)] {
      let type_prefix = type_opt
        .as_ref()
        .map(|t| t.to_string())
        .filter(|s| !s.is_empty())
        .map(|s| format!("{s}_"))
        .unwrap_or_default();
      let sub_prefix = subtype_opt
        .as_ref()
        .map(|st| st.to_string())
        .filter(|s| !s.is_empty())
        .map(|s| format!("{s}_"))
        .unwrap_or_default();
      for (key, value) in kv.get_pairs() {
        if !type_prefix.is_empty() || !sub_prefix.is_empty() {
          let state_key = s!("CAPTION_{type_prefix}{sub_prefix}{key}");
          assign_value(
            &state_key,
            Stored::String(pin(value.to_string())),
            None,
          );
        }
        let state_key = s!("CAPTION_{}", if key == "type*" { "type" } else { key.as_str() });
        assign_value(
          &state_key,
          Stored::String(pin(value.to_string())),
          None,
        );
        // `type=`/`type*=` are `\setcaptiontype` → `\caption@settype` (caption.sty:283-284,
        // :288-297, :300-303), which a float's begin also runs, so a `\ContinuedFloat` after
        // `\captionsetup{type=figure}` in a minipage continues, and one before it is dropped.
        if (key == "type" || key == "type*") && type_prefix.is_empty() && sub_prefix.is_empty() {
          engine::latex_constructs::begin_float_continuation(&value.to_string());
        }
      }
      // caption's `\captionsetup` sets its keys (`\caption@setkeys{caption}`, caption3.sty:244-
      // 259): a key a document or package declared with `\DeclareCaptionOption` runs its code. A
      // typed `\captionsetup[type]`/`[type][sub]` only stores them for that type
      // (`\caption@setup@options`, :252-262): bicaption's `\captionsetup[bi-second]{bi-second}`
      // (sjtuthesis.cls:730-735, cquthesis.cls:341-350) must not rename every figure.
      let untyped = type_prefix.is_empty() && sub_prefix.is_empty();
      for (key, value) in kv.get_pairs().filter(|_| untyped) {
        if lookup_bool(&s!("CAPTION_DECLARED_{key}")) {
          let mut call = vec![T_CS!(s!("\\KV@caption@{key}")), T_BEGIN!()];
          call.extend(value.clone().owned_tokens().unwrap_or_default().unlist());
          call.push(T_END!());
          Digest!(Tokens::new(call))?;
        }
      }
    }
  );
  def_macro_noop("\\DeclareCaptionStyle{}[]{}")?;
  // caption3.sty:1753-1756: the public float-type API lazy-loads newfloat and
  // delegates (`\DeclareCaptionType[opts]{type}[singular][listname]`);
  // pygmentex.sty:23 `\DeclareCaptionType{pygcode}[Listagem][Lista de
  // listagens]` (pygmentex ×2, hvpygmentex). Perl's caption.sty.ltxml omits it.
  // Guard: `perfect_kernel_batch54::declare_caption_type_makes_a_float`.
  RawTeX!(r"\newcommand\DeclareCaptionType{\RequirePackage{newfloat}\DeclareFloatingEnvironment}");
  // caption3.sty:730-739: a label format is a two-argument macro (name, number) the caption label is
  // built with (`labelformat=<name>`); the predefined ones, less `autodot`; `original` (the default)
  // is its fallback, `simple` (:740-745), there being no kernel `\fnum@<type>` of its own to keep.
  // subcaption's sub-captions read theirs (`\fnum@sub<type>`, subcaption_sty.rs).
  RawTeX!(r"\def\DeclareCaptionLabelFormat#1#2{%
\expandafter\long\expandafter\def\csname caption@labelformat@#1\endcsname##1##2{#2}}
\DeclareCaptionLabelFormat{empty}{}
\DeclareCaptionLabelFormat{simple}{\bothIfFirst{#1}{\nobreakspace}#2}
\DeclareCaptionLabelFormat{parens}{\bothIfFirst{#1}{\nobreakspace}(#2)}
\DeclareCaptionLabelFormat{brace}{\bothIfFirst{#1}{\nobreakspace}#2)}
\DeclareCaptionLabelFormat{unnumbered}{#1}
\DeclareCaptionLabelFormat{original}{\bothIfFirst{#1}{\nobreakspace}#2}
\let\caption@labelformat@default\caption@labelformat@original");
  // `\DeclareCaptionLabelSeparator{name}{body}` — caption3.sty L289 stores
  // body in `\caption@lsep@<name>`. floatrow.sty L1185 lets its
  // `\DeclareFloatSeparators` to this, and its option `capbesidesep=<name>`
  // looks up `\caption@lsep@<name>` via `\@ifundefined`. A no-op stub
  // makes every floatrow separator option fire
  // `Error:latex:\GenericError Package floatrow Error: Undefined float
  // separator '<name>'`. Witness 2403.03161 (capbesidesep=quad).
  // The `*` form (caption3 L780 `\DeclareCaptionLabelSeparator*{quad}{\quad}`)
  // sets a "no autobreak" flag; HTML rendering ignores autobreaks so
  // accepting the same body for both forms is fine.
  DefMacro!("\\DeclareCaptionLabelSeparator OptionalMatch:* {}{}",
    "\\expandafter\\def\\csname caption@lsep@#2\\endcsname{#3}");
  // Standard caption3.sty separators (L304-307 + L780). Pre-register
  // so floatrow's `capbesidesep=<std-name>` resolves without needing
  // the raw caption3.sty to load. Witness 2403.03161.
  RawTeX!(
    r"\DeclareCaptionLabelSeparator{none}{}%
\DeclareCaptionLabelSeparator{colon}{: }%
\DeclareCaptionLabelSeparator{period}{. }%
\DeclareCaptionLabelSeparator{space}{ }%
\DeclareCaptionLabelSeparator{quad}{\quad}%
\DeclareCaptionLabelSeparator{newline}{\\}%
\DeclareCaptionLabelSeparator{endash}{ -- }");
  // caption3.sty (2023, v2.4d) :767 also defines `\caption@lsep@default`; a
  // loaded caption3 is detected by babel-hungarian through it —
  // magyar.ldf:1882-1898 `\ifx\caption@lsep\caption@lsep@default
  // \caption@setdefaultlabelsep{period}\fi` calls a caption3 internal REMOVED
  // in 2023 only when both are undefined (two undefined cs `\ifx` true), so
  // without this definition the seeding above sent elteikthesis/elteiktdk into
  // the deprecated call (RUST-ONLY: Perl seeds no separators). Guard:
  // `perfect_kernel_batch56::caption_lsep_default_keeps_magyar_off_the_removed_internal`.
  RawTeX!(r"\newcommand*\caption@lsep@default{\caption@labelseparator@default\caption@labelsep}");
  def_macro_noop("\\DeclareCaptionFont{}{}")?;
  // caption3.sty:701-711 `\DeclareCaptionFormat*?{name}[short]?{code}`: the
  // star and the optional must be consumed too — a 2-arg no-op left the
  // starred form's `{code}` with its `#1#2#3` in the stream (nostarch.cls:856;
  // Perl's `{}{}` no-op fails the same way). The declaration itself has no
  // rendering here (captions are structural).
  DefMacro!("\\DeclareCaptionFormat OptionalMatch:* {} [] {}", "");
  // caption3.sty L432: `\DeclareCaptionTextFormat{name}{body}` — sibling
  // of `\DeclareCaptionFormat` for text-only caption-format definers.
  def_macro_noop("\\DeclareCaptionTextFormat{}{}")?;
  // caption3.sty L955-959: `\DeclareCaptionJustification[<pkg>]{<name>}{<body>}`
  // defines `\caption@justification@<name>` (the body) AND lets
  // `\caption@hj@<name>` equal it. The `\caption@hj@<name>` macros are
  // probed by other packages — notably floatrow.sty L1169
  // (`\@ifundefined{caption@hj@#1}` for `objectset=centering`/`raggedright`);
  // a pure no-op leaves them undefined → `Package floatrow Error: Undefined
  // object setting` (witness 1504.02564, 1608.07117, 1704.01862,
  // 1708.07230, 1712.06479). Faithfully define `\caption@hj@<name>` to the
  // body (collapsing caption3's justification@→hj@ \let into one \@namedef).
  // The optional `[<pkg>]` arg (caption3 L1361 `[ragged2e]{Justified}{...}`)
  // is consumed and ignored — it only triggers package-autoload, moot here.
  RawTeX!(r"\def\DeclareCaptionJustification{\@ifnextchar[\lx@caption@decljust@opt{\lx@caption@decljust@opt[]}}");
  RawTeX!(r"\def\lx@caption@decljust@opt[#1]#2#3{\@namedef{caption@hj@#2}{#3}\@namedef{caption@justification@#2}{#3}}");
  // Seed the standard justifications caption3.sty declares at load time
  // (L964-969) so they exist even when a paper never re-declares them.
  RawTeX!(r"\DeclareCaptionJustification{justified}{}%
\DeclareCaptionJustification{centering}{\centering}%
\DeclareCaptionJustification{centerfirst}{\centering}%
\DeclareCaptionJustification{centerlast}{\centering}%
\DeclareCaptionJustification{raggedleft}{\raggedleft}%
\DeclareCaptionJustification{raggedright}{\raggedright}");
  // caption3.sty:221-236: \DeclareCaptionOption delegates to \define@key{caption}
  // bicaption.sty:71-76 uses \DeclareCaptionOption{bi-swap}[1]{\caption@set@bool\bicaption@ifswap{#1}}
  // The star only undefines the key at the end of the declaring package (`\caption@teststar
  // \caption@declareoption\AtEndOfPackage\@gobble`); the binding's star branch gobbled its own
  // helper, leaving the key name and code in the document (KPE #374). A declared key is
  // recorded (`\lx@caption@declared`) so `\captionsetup` runs its code, as caption's does.
  RawTeX!(
    r"\def\DeclareCaptionOption{\@ifstar{\caption@decl@opt}{\caption@decl@opt}}%
\def\caption@decl@opt#1{\lx@caption@declared{#1}\define@key{caption}{#1}}%
\def\DeclareCaptionOptionNoValue{\@ifstar{\caption@decl@opt@noval}{\caption@decl@opt@noval}}%
\def\caption@decl@opt@noval#1#2{\lx@caption@declared{#1}\define@key{caption}{#1}{#2}}%
\providecommand*\bicaption@ifswap{\@secondoftwo}%
\providecommand*\bicaption@ifslc{\@firstoftwo}"
  );
  def_macro_noop("\\DeclareCaptionPackage{}")?;
  // caption3.sty internals that user code or extension packages
  // (e.g. caption-style extensions, fltrace, ccaption) sometimes
  // reach for. All no-ops — caption-package internals are
  // typesetting-only and have no body-content effect:
  //   * `\SetCaptionDefault{name}{body}` — set default value for
  //     a named caption option (5 R-stage papers).
  //   * `\caption@ifundefined{cs}{then}{else}` — internal version
  //     of `\@ifundefined`. Treat as undefined (always run `\else`).
  //   * `\caption@ExecuteOptions[opt-list]` — internal option-
  //     execution helper. No-op.
  // caption3.sty:446-457 `\SetCaptionDefault{name}{value}` →
  // `\caption@@set{name}{name@default}{value}` (:443-445) = define
  // `\caption@<name>@default` as `\caption@<name>@<value>`. bicaption.sty:92
  // `\SetCaptionDefault{biseparator}{none}` must bind
  // `\caption@biseparator@default`, or :132 `\caption@set{biseparator}
  // {default}` raises "Undefined biseparator `default'" (shtthesis).
  // The `*`-form (`\edef`), `\caption@maparg` aliasing, `\caption@checkdecl`
  // and the `\@onlypreamble` restriction (:448) are omitted (keyword values only). Guard:
  // `perfect_kernel_batch56::setcaptiondefault_binds_the_default`.
  RawTeX!(r"\def\SetCaptionDefault#1#2{\expandafter\def\csname caption@#1@default\expandafter\endcsname\expandafter{\csname caption@#1@#2\endcsname}}");
  // caption3.sty:67-75: \caption@ifundefined\cs{then:undefined}{else:defined}
  // bicaption.sty:379 calls \caption@ifundefined\caption@LT@setup{\providecommand*\caption@LT@setup{}}
  RawTeX!(
    r"\newcommand*\caption@ifundefined[1]{%
  \ifdefined#1%
    \ifx#1\relax \expandafter\expandafter\expandafter\@firstoftwo
    \else \expandafter\expandafter\expandafter\@secondoftwo \fi
  \else \expandafter\@firstoftwo \fi}"
  );
  // caption3.sty:130-139: boolean option setter
  // bicaption.sty:76 calls \caption@set@bool\bicaption@ifswap{#1}
  RawTeX!(
    r"\def\caption@set@bool#1#2{%
  \caption@ifinlist{#2}{1,true,yes,on}%
    {\let#1\@firstoftwo}%
    {\let#1\@secondoftwo}}
\def\caption@ifinlist#1#2#3#4{%
  \in@{,#1,}{,#2,}\ifin@#3\else#4\fi}
\def\caption@ExecuteOptions#1#2{\caption@setkeys{#1}{#2}}
\def\caption@Error#1{\PackageError{caption}{#1}{}}"
  );
  // caption.sty L184-185 call these as part of package-init bootstrap of
  // the caption3 backend (`\caption@SetupOptions{caption}{\caption@setkeys...}`
  // / `\caption@ProcessOptions*{caption}`). Our binding intercepts
  // caption.sty before caption3.sty raw-loads, so these caption3
  // internals are undefined. No-op stubs are safe — option setup is
  // typesetting-only, and `\captionsetup` (handled above) already
  // stores keyvals as `CAPTION_<key>` state regardless of this
  // bootstrap chain. Witness clusters: ~5 R-stage papers each.
  def_macro_noop("\\caption@SetupOptions{}{}")?;
  def_macro_noop("\\caption@ProcessOptions OptionalMatch:* {}")?;
  // \caption@IfPackageLoaded{pkg}[date]{body}{else} (caption.sty L700-702
  // + L703-708). caption.sty self-registers conditional adapters for
  // float / hyperref / longtable / ... — our XML pipeline doesn't need
  // any of those adapters, so always take the `else` branch as if the
  // package is not loaded.
  DefMacro!("\\caption@IfPackageLoaded{}[]{}{}", "#4");
  def_macro_noop("\\caption@@IfPackageLoaded{}[]{}{}")?;
  // caption3.sty L564: \DeclareCaptionBox{name}{body} defines a
  // "caption@box@<name>" macro via \@namedef. We don't render caption
  // box layouts; gobble both args.
  def_macro_noop("\\DeclareCaptionBox{}{}")?;
  // caption3.sty L573: \DeclareCaptionListFormat{name}{body}
  def_macro_noop("\\DeclareCaptionListFormat{}{}")?;
  // caption3.sty:1595 `\providecommand*\caption@prepareslc{}` — an empty
  // hook that other packages extend (hep-bibliography.sty:108
  // `\g@addto@macro\caption@prepareslc{…}` under `\AtBeginDocument`; the 9
  // hep-* docs). The emulation stands in for caption3.sty, so it carries the
  // hook. Guard: `perfect_kernel_batch54::caption_prepareslc_hook_is_defined`.
  DefMacro!("\\caption@prepareslc", "");

  // caption3 internals used by raw-loaded sibling packages like
  // floatrow.sty. Real `\caption@setkeys [opt] {family} {kvs}` calls
  // `\setkeys{family}{kvs}` with caption-specific error handling
  // (caption3_2020-10-26.sty L337-360). Stub to a plain `\setkeys`
  // — drops the optional error-handler context but preserves
  // keyval-processing semantics. Witness cluster: papers using
  // `\usepackage{floatrow}` which raw-loads its body containing
  // `\caption@setkeys{...}{...}` calls.
  DefMacro!("\\caption@setkeys[]{}{}", "\\setkeys{#2}{#3}");
  // `\undefine@key` removes a keyval. Real keyval.sty defines it
  // post-2018; xkeyval too. Both Perl LaTeXML's keyval.sty.ltxml
  // hand-port and our Rust binding pre-date that and don't include
  // it. Stub as a no-op — keyval removal is mostly an authoring
  // hygiene issue; missing it means stale keys linger but no
  // tokenization breakage. Witness: same floatrow chain.
  def_macro_noop("\\undefine@key{}{}")?;

  DefMacro!("\\bothIfFirst{}{}", sub[(first, second)] {
    if first.is_empty() { Ok(Tokens!()) } else {
      let mut result = first.unlist();
      result.extend(second.unlist());
      Ok(Tokens::new(result))
    }
  });

  DefMacro!("\\bothIfSecond{}{}", sub[(first, second)] {
    if second.is_empty() { Ok(Tokens!()) } else {
      let mut result = first.unlist();
      result.extend(second.unlist());
      Ok(Tokens::new(result))
    }
  });

  // caption3.sty:1048-1051: caption hook definitions.
  // bicaption.sty:128 appends to \caption@beginhook with \g@addto@macro.
  RawTeX!(r"\def\caption@beginhook{}");
  RawTeX!(r"\def\caption@endhook{}");
  RawTeX!(r"\def\AtBeginCaption#1{\g@addto@macro\caption@beginhook{#1}}");
  RawTeX!(r"\def\AtEndCaption#1{\g@addto@macro\caption@endhook{#1}}");

  // caption.sty:1208: \caption@LT@setup for longtable integration.
  // bicaption.sty:386 patches \caption@LT@setup with \g@addto@macro.
  RawTeX!(r"\def\caption@LT@setup{}");

  // caption.sty:600: \caption@dblarg duplicates single argument [arg]{arg}.
  // bicaption.sty:261,265,361,365 uses \caption@dblarg.
  RawTeX!(r"\def\caption@dblarg{\@dblarg}");

  // caption.sty:300-320 `\caption@settype{type}` — the internal a caption-aware
  // package calls to declare the float type of the box it builds (wrapstuff.sty
  // :2382-2384, with `\caption@clearmargin` and `\caption@setoptions{wrap type}`;
  // latex-via-exemplos' ex15-wrapstuff). Its effect in this model is
  // `\@captype` (caption.sty:313 `\let\@captype\caption@tempa`, after an `\edef`);
  // the margin reset (:751) and the per-type option set (caption3.sty:326) are
  // typographic (`\caption@setoptions` is the existing no-op below). Undefined
  // here, the raw package's calls stood as errors and the caption lost its type. It also
  // clears a pending continuation and makes the type the current float's (:302,
  // `\caption@clrflags`; `\lx@caption@settype`, `latex_constructs::begin_float_continuation`).
  RawTeX!(r"\def\caption@settype#1{\edef\@captype{#1}\expandafter\lx@caption@settype\expandafter{\@captype}}");
  DefPrimitive!("\\lx@caption@settype{}", sub[(float_type)] {
    engine::latex_constructs::begin_float_continuation(&float_type.to_string());
  });
  RawTeX!(r"\def\caption@clearmargin{}");
  // Common internal hooks from caption.sty / caption3.sty
  RawTeX!(r"\def\caption@beginex@hook{}");
  RawTeX!(r"\def\caption@xfloat@hook{}");
  RawTeX!(r"\def\caption@xdblfloat@hook{}");
  RawTeX!(r"\def\caption@subtype@hook{}");
  RawTeX!(r"\def\caption@calcmargin@hook{}");
  // caption.sty:496-536 `\continuedfloat[*]` (`\ContinuedFloat`): the float continues the one
  // stepped before it — the same number, the sub-float letters going on — its parts counted in
  // `continuedfloat`, whose `\@alph` value suffixes the ids (caption's `\theH<type>` append,
  // :530-532) while `\thecontinuedfloat` (empty by default) suffixes the number. caption
  // suppresses the float counter's next step (`\caption@setcontinued`, :192, then
  // `\caption@@refcounter`, :557-568): the kernel's `step_float_counter` keeps the number for a
  // pending continuation of its type (`lx@float@continued`), and runs `\lx@float@stepped` on every
  // real step of the current float's own counter (caption's `\caption@reset@continuedfloat`,
  // :501-503, :577-579).
  // Perl's binding is a no-op (caption.sty.ltxml:91; KNOWN_PERL_ERRORS #362).
  // Witness 2605.17685; repro captions-floats/continuedfloat_keeps_the_number_and_the_letters.
  RawTeX!(r"\newcounter{continuedfloat}\let\c@ContinuedFloat\c@continuedfloat
\def\thecontinuedfloat{\theContinuedFloat}\let\theContinuedFloat\@empty
\providecommand\l@addto@macro[2]{\edef#1{\unexpanded\expandafter{#1#2}}}
\newcommand*\continuedfloat@captype{??}
\def\lx@float@stepped#1{\xdef\continuedfloat@captype{#1}\global\c@continuedfloat\z@}
\def\lx@caption@addto@continued#1{\expandafter\l@addto@macro\csname the#1\endcsname\thecontinuedfloat
  \@ifundefined{the#1@ID}{}{\expandafter\l@addto@macro\csname the#1@ID\endcsname{\@alph\c@continuedfloat}}}
\def\continuedfloat{\@ifstar{\lx@caption@continuedfloat*}{\lx@caption@continuedfloat}}
\def\ContinuedFloat{\continuedfloat}");
  DefPrimitive!("\\lx@caption@continuedfloat OptionalMatch:*", sub[(star)] {
    // `\caption@iftype` (caption.sty:514-518): the float's `\@captype`, or the `type` a
    // `\captionsetup{type=…}` declared (as `\maybe@@generic@caption` reads it).
    let captype = if has_meaning(&T_CS!("\\@captype")) {
      do_expand(T_CS!("\\@captype"))?.to_string()
    } else {
      lookup_string("CAPTION_type")
    };
    if captype.is_empty() {
      Error!("unexpected", "\\continuedfloat", "\\continuedfloat outside float");
    } else {
      if star.is_some() {
        // `\continuedfloat*` (:516): the float steps now, a new number, then continues.
        engine::latex_constructs::step_float_counter(&captype)?;
      }
      continue_float(&captype, true)?;
    }
  });
  // caption.sty:535-536, called by subfig's `\ContinuedFloat` (subfig.sty:581-590), which steps
  // the counter back itself and restores its sub-counter from `sub<type>@save`. Witness 2605.17685.
  DefPrimitive!("\\caption@ContinuedFloat{}", sub[(captype)] {
    continue_float(&captype.to_string(), false)?;
  });
  // caption.sty L: `\providecommand*\nextfloat{...}` — used to mark
  // sub-caption float continuation. Gobble safely (visual-only).
  // Witness 2202.03356.
  def_macro_noop("\\nextfloat")?;
  def_macro_noop("\\ProcessOptionsWithKV{}")?;

  def_macro_noop("\\captionfont")?;
  def_macro_noop("\\captionsize")?;

  DefRegister!("\\captionparindent"  => Dimension::new(0));
  DefRegister!("\\captionindent"     => Dimension::new(0));
  DefRegister!("\\captionhangindent" => Dimension::new(0));
  DefRegister!("\\captionmargin"     => Dimension::new(0));
  DefRegister!("\\captionwidth"      => Dimension::new(0));

  // Override \caption to support \caption* (starred form)
  // caption.sty:454-487 `\captionbox[list]{caption}[width][inner]{content}`
  // — content with its caption in a box (below by default). Same idiom as
  // subcaption's `\subcaptionbox`; the width/inner position are layout only.
  // Witness tikz-mirror-lens (both bindings lacked it).
  DefMacro!("\\captionbox []{}[][]{}",
    "\\begingroup#5\\caption{#2}\\ifx.#1.\\else\\lx@caption@addinlist{#1}\\fi\\endgroup");
  DefConstructor!("\\lx@caption@addinlist{}", "", properties => sub[args] {
    let list = args[0].as_ref().map(|a| a.to_string()).unwrap_or_default();
    Ok(stored_map!("inlist" => list))
  });
  DefMacro!("\\caption",
    r"\lx@donecaptiontrue\@ifundefined{@captype}{\maybe@@generic@caption}{\@ifstar{\@scaption}{\expandafter\@caption\expandafter{\@captype}}}"
  );
  DefMacro!("\\@scaption{}", "\\@@caption{#1}");

  // \captionof — fake a caption in any context.
  //
  // Perl caption.sty.ltxml L110-115 routes through the `CAPTION_type` state
  // value set by `\captionsetup{type=…}`: when the author has declared a
  // float type, `\maybe@@generic@caption` expands to `\@captionof{type}`
  // so the caption digests inside the proper environment; otherwise it
  // falls through to `\@@generic@caption`. Rust previously hardcoded the
  // fallback, silently dropping the captionsetup type.
  DefMacro!("\\maybe@@generic@caption", sub[_args] {
    if let Some(Stored::String(t)) = lookup_value("CAPTION_type") {
      let ty = with(t, |s| s.to_string());
      if !ty.is_empty() {
        let mut out = vec![T_CS!("\\@captionof"), T_BEGIN!()];
        out.extend(ExplodeText!(&ty));
        out.push(T_END!());
        return Ok(Tokens::new(out));
      }
    }
    Ok(Tokens!(T_CS!("\\@@generic@caption")))
  });
  // caption.sty:389-391: `\captionof` is `\caption@of`, `\setcaptiontype*{<type>}` then the
  // caption — the type set (`\lx@caption@settype`) before the float `\@captionof@` wraps the
  // caption in, whose begin then opens nothing (`begin_float`, the `\lx@caption@wrapper` one-shot).
  // A typed `\caption` (`\maybe@@generic@caption`) reaches `\@captionof` with the type
  // `\captionsetup{type=…}` set, and sets none itself, as caption's `\caption`.
  DefMacro!("\\captionof", "\\@ifstar{\\lx@caption@of\\@scaptionof}{\\lx@caption@of\\@captionof}");
  // The type is expanded once, as `\caption@@settype`'s `\edef` (caption.sty:309).
  RawTeX!(r"\def\lx@caption@of#1#2{\edef\lx@caption@of@type{#2}\expandafter\lx@caption@of@\expandafter{\lx@caption@of@type}#1}
\def\lx@caption@of@#1#2{\lx@caption@settype{#1}#2{#1}}");
  // `\@captionof@`'s wrapper float: its begin is not a new type (`begin_float`, one-shot) — and
  // cleared after its `\end` whether or not the begin reached a float (an undefined or non-float
  // environment), so it cannot skip a later float's type. Guard
  // `perfect_kernel_batch56::continuedfloat_captionof_wrapper_does_not_leak`.
  DefPrimitive!("\\lx@caption@wrapper", {
    assign_value("lx@float@captionof", true, Some(Scope::Global));
  });
  DefPrimitive!("\\lx@caption@wrapper@done", {
    assign_value("lx@float@captionof", false, Some(Scope::Global));
  });
  DefMacro!("\\@captionof{}[]{}", r"\@ifnextchar\label{\@captionof@postlabel{#1}{#2}{#3}}{\@captionof@{#1}{#2}{#3}}");
  DefMacro!("\\@captionof@postlabel{}{}{} SkipMatch:\\label Semiverbatim", r"\@captionof@{#1}{#2}{#3\label{#4}}");
  // Perl wraps the caption in the named environment — "it isn't necessarily IN
  // a figure or any float, so we'll wrap it in an otherwise empty one!"
  // (`caption.sty.ltxml` L124-125) — and that is FATAL when the environment
  // reads its body verbatim. `\captionof{lstlisting}{…}` expands to
  // `\begin{lstlisting}…\end{lstlisting}`, but listings scans the raw INPUT for
  // its terminator, never the token stream, so it finds no `\end{lstlisting}`
  // and swallows the rest of the file: the document tail — `\bibliography`
  // included — comes out as line-numbered listing text. Witness 2606.08339,
  // where one such line costs the whole bibliography (0 entries; 30 once this
  // construct stops running away). pdflatex renders that paper correctly, and
  // real caption.sty never opens the environment at all — `\caption@of` is
  // `\setcaptiontype*{#2}#1` (caption.sty L391), i.e. it only sets the type.
  //
  // So for a verbatim-bodied type, emit just the caption. `\@caption@` carries
  // the type through for numbering and the construct is normally already
  // inside a float (it is in the witness), which is what pdflatex shows.
  // Non-verbatim types keep Perl's wrapper, since that is what gives an
  // unfloated `\captionof{figure}` its container. OXIDIZED_DESIGN #89.
  DefMacro!("\\@captionof@{}{}{}", sub[(ty, opt, text)] {
    let name = ty.to_string();
    let mut out = Vec::new();
    if !VERBATIM_BODY_ENVS.contains(&name.trim()) {
      out.push(T_CS!("\\lx@caption@wrapper"));
      out.push(T_CS!("\\begin"));
      out.push(T_BEGIN!());
      out.extend(ExplodeText!(name.trim()));
      out.push(T_END!());
    }
    out.push(T_CS!("\\@caption@"));
    for arg in [&ty, &opt, &text] {
      out.push(T_BEGIN!());
      out.extend(arg.clone().unlist());
      out.push(T_END!());
    }
    if !VERBATIM_BODY_ENVS.contains(&name.trim()) {
      out.push(T_CS!("\\end"));
      out.push(T_BEGIN!());
      out.extend(ExplodeText!(name.trim()));
      out.push(T_END!());
      out.push(T_CS!("\\lx@caption@wrapper@done"));
    }
    Ok(Tokens::new(out))
  });
  DefMacro!("\\@scaptionof{}{}", r"\begin{#1*}\@scaption{#2}\end{#1*}");

  // caption3.sty:275-280 `\clearcaptionsetup*[option]{type}`; Perl's argument-less
  // (caption.sty.ltxml:130) left the type as text (KPE #316).
  def_macro_noop("\\clearcaptionsetup OptionalMatch:* []{}")?;
  // No `\rotcaption`: caption.sty redefines it only when rotating is loaded (caption.sty:1284,
  // `\caption@IfPackageLoaded{rotating}`), as a caption; rotating_sty.rs defines it. Perl's no-op
  // (caption.sty.ltxml:131) replaced rotating's, losing the caption (KPE #321).
  def_macro_noop("\\showcaptionsetup[]{}")?;

  // \caption@ifinlist{val}{csv-list}{then}{else} — caption3.sty L87.
  // Returns `then` if val matches one of the comma-separated list items,
  // else `else`. Used by floatrow (`\caption@ifinlist{#1}{0,false,no,off}{...}{...}`)
  // and by caption-key parsing. Witness 2405.18938.
  DefMacro!("\\caption@ifinlist{}{}", sub[(val, list)] {
    let v_str = val.to_string();
    let v = v_str.trim();
    let l_str = list.to_string();
    let found = l_str.split(',').any(|item| item.trim() == v);
    Ok(if found {
      Tokens!(T_CS!("\\@firstoftwo"))
    } else {
      Tokens!(T_CS!("\\@secondoftwo"))
    })
  });

  // \caption@setposition{value} — caption3.sty L1007. Sets the caption
  // position. We don't materialize caption-position logic; stub as
  // no-op so floatrow-style position setters don't crash.
  def_macro_noop("\\caption@setposition{}")?;

  // \caption@set@bool{cs}{value} — caption3.sty L131. Defines `cs` as
  // `\@firstoftwo` if value is in {1,true,yes,on}, `\@secondoftwo` for
  // {0,false,no,off}, else error. We don't model caption boolean state
  // (caption settings don't affect XML output), so stub the dispatch
  // — \let the CS to \@secondoftwo by default. Witness 2408.09623,
  // 2408.12461, 2409.01528.
  DefMacro!("\\caption@set@bool DefToken {}", sub[(cs, value)] {
    let val = value.to_string();
    let truthy = matches!(val.trim(), "1" | "true" | "yes" | "on");
    let target_name = if truthy { "\\@firstoftwo" } else { "\\@secondoftwo" };
    let_i(&cs, &T_CS!(target_name), None);
    Ok(Tokens!())
  });
  // \caption@setbool{name} — wraps caption@set@bool by building \caption@if<name>.
  DefMacro!("\\caption@setbool{}{}",
    "\\expandafter\\caption@set@bool\\csname caption@if#1\\endcsname{#2}");
  // \caption@ifbool{name} — \@nameuse{caption@if<name>} dispatch helper.
  DefMacro!("\\caption@ifbool{}", "\\@nameuse{caption@if#1}");

  // \caption@setoptions{name} (caption3.sty L325-333) — apply the
  // named option setup if defined, else do nothing. Used by floatrow
  // (line 473) and various caption-extension packages. Stub as no-op
  // since the actual option dictionary `\caption@opt@<name>` isn't
  // populated under our digestion model. Witness 2412.15378 (floatrow).
  def_macro_noop("\\caption@setoptions{}")?;
  // \caption@@make — internal caption-rendering hook used by float
  // wrappers. No-op for our XML pipeline (caption text is emitted via
  // ltx:caption regardless of formatting). Witness 2412.15378.
  DefMacro!("\\caption@@make{}{}", "#2");
  // caption3.sty L850 defines \caption@setfont{kind}{value} — used
  // internally to apply font options (font/labelfont/textfont/size).
  // Font formatting is irrelevant in our XML output; gobble args.
  // Witness 2504.00326.
  def_macro_noop("\\caption@setfont{}{}")?;
  // \phantomcaption (caption package, originally subcaption) — adds an
  // invisible caption for layout reasons; we don't need spacing in XML
  // output, so stub as no-op. Witness 2503.21681.
  def_macro_noop("\\phantomcaption")?;
  def_macro_noop("\\phantomsubcaption")?;
});

/// caption.sty:504-511, :519-533: a float continues only the type stepped last
/// (`\continuedfloat@captype`, set on every real step), else caption's error; then
/// `continuedfloat` steps, the float's next step keeps its number when `suppress_step`
/// (`\caption@setcontinued`, a global flag until the next float begins, :173-192, :300-303;
/// subfig steps the counter back itself), and — once per float — the number and the ids take the
/// suffixes (`\caption@@@continuedfloat`, which gobbles itself).
fn continue_float(captype: &str, suppress_step: bool) -> Result<()> {
  let last = do_expand(T_CS!("\\continuedfloat@captype"))?.to_string();
  if last != captype {
    Error!(
      "unexpected",
      "\\ContinuedFloat",
      s!("Continued `{captype}' after `{last}'")
    );
    return Ok(());
  }
  step_counter("continuedfloat", false)?;
  if suppress_step {
    assign_value(
      "lx@float@continued",
      captype.to_string(),
      Some(Scope::Global),
    );
  }
  if !lookup_bool("lx@caption@continued") {
    assign_value("lx@caption@continued", true, Some(Scope::Local));
    let mut tokens = vec![T_CS!("\\lx@caption@addto@continued"), T_BEGIN!()];
    tokens.extend(Explode!(captype));
    tokens.push(T_END!());
    unread(Tokens::new(tokens));
  }
  Ok(())
}
