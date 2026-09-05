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
  ] {
    DefKeyVal!("caption", key, "");
  }

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
        let state_key = s!("CAPTION_{key}");
        assign_value(
          &state_key,
          Stored::String(pin(value.to_string())),
          None,
        );
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
  def_macro_noop("\\DeclareCaptionLabelFormat{}{}")?;
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
  RawTeX!(
    r"\def\DeclareCaptionOption{\@ifstar{\@gobble\caption@decl@opt}{\caption@decl@opt}}%
\def\caption@decl@opt#1{\define@key{caption}{#1}}%
\def\DeclareCaptionOptionNoValue{\@ifstar{\@gobble\caption@decl@opt@noval}{\caption@decl@opt@noval}}%
\def\caption@decl@opt@noval#1#2{\define@key{caption}{#1}{#2}}%
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
  def_macro_noop("\\SetCaptionDefault{}{}")?;
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

  // Common internal hooks from caption.sty / caption3.sty
  RawTeX!(r"\def\caption@beginex@hook{}");
  RawTeX!(r"\def\caption@xfloat@hook{}");
  RawTeX!(r"\def\caption@xdblfloat@hook{}");
  RawTeX!(r"\def\caption@subtype@hook{}");
  RawTeX!(r"\def\caption@calcmargin@hook{}");
  def_macro_noop("\\ContinuedFloat")?;
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
  DefMacro!("\\captionof", "\\@ifstar{\\@scaptionof}{\\@captionof}");
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
    }
    Ok(Tokens::new(out))
  });
  DefMacro!("\\@scaptionof{}{}", r"\begin{#1*}\@scaption{#2}\end{#1*}");

  def_macro_noop("\\clearcaptionsetup")?;
  def_macro_noop("\\rotcaption")?;
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
