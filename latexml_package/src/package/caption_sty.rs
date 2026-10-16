use latexml_core::{definition::ExpansionBody, mouth::tokenize};

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
      // A typed `\captionsetup[type]`/`[type][sub]` stores its keys for that type only (`\caption@setup@options`,
      // caption3.sty:252-262): subfig's `\captionsetup[subfloat]{labelformat=empty}` must not empty every
      // figure's label, now that the main captions read these settings (`caption_format_tokens`).
      let typed = !type_prefix.is_empty() || !sub_prefix.is_empty();
      for (key, value) in kv.get_pairs() {
        // The value's tokens, and its text for the keyword values (`labelsep=period`, `font={small,it}`), read back
        // without welding (`untex`).
        let value_tokens = value.clone().owned_tokens().unwrap_or_default();
        let text_value = caption_value(value_tokens.clone().untex_string().as_str());
        let prefix = s!("CAPTION_{type_prefix}{sub_prefix}");
        // caption3.sty:842-848: `font+=`/`labelfont+=`/`textfont+=` add to the font a `font=` sets anew.
        if let Some(base) = key.strip_suffix('+') {
          let previous = [s!("{prefix}{base}"), s!("CAPTION_{base}")]
            .iter()
            .map(|k| lookup_string(k))
            .find(|v| !v.is_empty())
            .unwrap_or_default();
          let joined = if previous.is_empty() { text_value.clone() } else { s!("{previous},{text_value}") };
          assign_value(&s!("{prefix}{base}"), Stored::String(pin(joined)), None);
          continue;
        }
        let key = match key.as_str() {
          "type*" if !typed => "type",
          // caption3.sty:774-775: `labelseparator` is `labelsep`.
          "labelseparator" => "labelsep",
          other => other,
        };
        assign_value(&s!("{prefix}{key}"), Stored::String(pin(text_value.clone())), None);
        if key == "name" {
          assign_value(&s!("{prefix}name@tokens"), Stored::Tokens(value_tokens.clone()), None);
        }
        if !typed {
          match key {
            // caption3.sty:1347-1350: `name=` names the current float type (`\@namedef{\@captype name}`); outside a
            // float there is no type to name.
            "name" if has_meaning(&T_CS!("\\@captype")) => {
              let captype = do_expand(T_CS!("\\@captype"))?.to_string();
              let mut call = vec![T_CS!("\\lx@caption@setname"), T_BEGIN!()];
              call.extend(ExplodeText!(captype.trim()));
              call.push(T_END!());
              call.push(T_BEGIN!());
              call.extend(value_tokens.clone().unlist());
              call.push(T_END!());
              Digest!(Tokens::new(call))?;
            },
            // caption3.sty:1336-1345: `figurename=`/`tablename=`/`listfigurename=`/`listtablename=` set the names
            // as newfloat's `\newfloat@setname` does (`\caption@SetName`), so babel's `\captions<language>` at
            // `\begin{document}` keeps them.
            "figurename" | "tablename" | "listfigurename" | "listtablename" => {
              let name = key.strip_suffix("name").unwrap_or(key);
              let mut call = vec![T_CS!("\\lx@caption@SetName"), T_BEGIN!()];
              call.extend(ExplodeText!(name));
              call.push(T_END!());
              call.push(T_BEGIN!());
              call.extend(value_tokens.clone().unlist());
              call.push(T_END!());
              Digest!(Tokens::new(call))?;
            },
            _ => {},
          }
        }
        // `type=`/`type*=` are `\setcaptiontype` → `\caption@settype` (caption.sty:283-284,
        // :288-297, :300-303), which a float's begin also runs, so a `\ContinuedFloat` after
        // `\captionsetup{type=figure}` in a minipage continues, and one before it is dropped.
        // It sets `\@captype` (:313), which a `\subcaptionbox` or `\caption` in that minipage
        // reads: unset, they built `\c@\@captype` (2609.06557, 32742; KNOWN_PERL_ERRORS #504).
        if (key == "type" || key == "type*") && !typed {
          let mut call = vec![T_CS!("\\caption@settype"), T_BEGIN!()];
          call.extend(value.clone().owned_tokens().unwrap_or_default().unlist());
          call.push(T_END!());
          Digest!(Tokens::new(call))?;
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
  // caption3.sty:1349-1350 `\caption@setname{type}{name}` = `\@namedef{<type>name}{name}`.
  RawTeX!(r"\def\lx@caption@setname#1#2{\@namedef{#1name}{#2}}");
  // caption3.sty:1344-1345 `\caption@SetName` is newfloat.sty:179-188 `\newfloat@setname`: the name is set, and added
  // to the current language's `\captions<language>` (babel, polyglossia), which `\begin{document}` runs and which
  // would otherwise reset it ("Figure" again under `[english]{babel}`).
  RawTeX!(r"\def\lx@caption@SetName#1#2{\lx@caption@setname{#1}{#2}\begingroup
  \ifcsname languagename\endcsname\ifcsname captions\languagename\endcsname
  \expandafter\g@addto@macro\csname captions\languagename\endcsname{\lx@caption@setname{#1}{#2}}\fi\fi
  \endgroup}");
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
\DeclareCaptionLabelSeparator{newline}{\captionnewline}%
\DeclareCaptionLabelSeparator{endash}{\space\textendash\space}%
\DeclareCaptionLabelSeparator{gobble}{ }%
\DeclareCaptionLabelSeparator{gobbletwo}{}%
\providecommand*\captionnewline{\\\relax}%
\def\caption@labelsep@crlist{newline}%
\newcommand*\caption@LabelSeparatorWithNewline[1]{\edef\caption@labelsep@crlist{\caption@labelsep@crlist,#1}}");
  // caption3.sty (2023, v2.4d) :767 also defines `\caption@lsep@default`; a
  // loaded caption3 is detected by babel-hungarian through it —
  // magyar.ldf:1882-1898 `\ifx\caption@lsep\caption@lsep@default
  // \caption@setdefaultlabelsep{period}\fi` calls a caption3 internal REMOVED
  // in 2023 only when both are undefined (two undefined cs `\ifx` true), so
  // without this definition the seeding above sent elteikthesis/elteiktdk into
  // the deprecated call (RUST-ONLY: Perl seeds no separators). Guard:
  // `perfect_kernel_batch56::caption_lsep_default_keeps_magyar_off_the_removed_internal`.
  RawTeX!(r"\newcommand*\caption@lsep@default{\caption@labelseparator@default\caption@labelsep}");
  // caption3.sty:830-836 `\DeclareCaptionFont[<package>]{name}{code}`: `\caption@font@<name>`, a one-argument macro
  // (the value of `name=value`, as `color=blue`) a font option (`font=`, `labelfont=`, `textfont=`) runs
  // (`caption_font_tokens`). The predefined ones, caption3.sty:860-950 and :1365-1375; the spacing ones are layout.
  // Perl's no-op (caption.sty.ltxml:73) keeps every caption font out (KNOWN_PERL_ERRORS #795).
  RawTeX!(r"\def\DeclareCaptionFont{\@ifnextchar[\lx@caption@declfont{\lx@caption@declfont[]}}
\def\lx@caption@declfont[#1]#2#3{\expandafter\long\expandafter\def\csname caption@font@#2\endcsname##1{#3}}
\DeclareCaptionFont{normalcolor}{\normalcolor}\DeclareCaptionFont{color}{\color{#1}}%
\DeclareCaptionFont{normalfont}{\normalfont}\DeclareCaptionFont{up}{\upshape}\DeclareCaptionFont{it}{\itshape}%
\DeclareCaptionFont{sl}{\slshape}\DeclareCaptionFont{sc}{\scshape}\DeclareCaptionFont{md}{\mdseries}%
\DeclareCaptionFont{bf}{\bfseries}\DeclareCaptionFont{rm}{\rmfamily}\DeclareCaptionFont{sf}{\sffamily}%
\DeclareCaptionFont{tt}{\ttfamily}\DeclareCaptionFont{tiny}{\tiny}\DeclareCaptionFont{scriptsize}{\scriptsize}%
\DeclareCaptionFont{footnotesize}{\footnotesize}\DeclareCaptionFont{small}{\small}%
\DeclareCaptionFont{normalsize}{\normalsize}\DeclareCaptionFont{large}{\large}\DeclareCaptionFont{Large}{\Large}%
\DeclareCaptionFont{LARGE}{\LARGE}\DeclareCaptionFont{huge}{\huge}\DeclareCaptionFont{Huge}{\Huge}%
\DeclareCaptionFont{smaller}{}\DeclareCaptionFont{larger}{}\DeclareCaptionFont{none}{}%
\DeclareCaptionFont{normal}{\normalcolor\normalfont\normalsize}\DeclareCaptionFont{default}{}%
\DeclareCaptionFont{hypersetup}{}\DeclareCaptionFont{sansmath}{}\DeclareCaptionFont{singlespacing}{}%
\DeclareCaptionFont{stretch}{}\DeclareCaptionFont{onehalfspacing}{}\DeclareCaptionFont{doublespacing}{}");
  // caption3.sty:701-711 `\DeclareCaptionFormat*?{name}[short]?{code}`: the
  // star and the optional must be consumed too — a 2-arg no-op left the
  // starred form's `{code}` with its `#1#2#3` in the stream (nostarch.cls:856;
  // Perl's `{}{}` no-op fails the same way). The declaration itself has no
  // rendering here (captions are structural).
  DefMacro!("\\DeclareCaptionFormat OptionalMatch:* {} [] {}", "");
  // caption3.sty L432: `\DeclareCaptionTextFormat{name}{body}` — sibling
  // of `\DeclareCaptionFormat` for text-only caption-format definers.
  // caption3.sty:787-799 `\DeclareCaptionTextFormat{name}{code}`: `\caption@textformat@<name>`, applied to the caption
  // text (`textformat=`, `caption_format_tokens`); `simple` and `period` predefined. caption3's `empty` keeps only the
  // text's `\label` (:792-797), done by `caption_format_tokens`. Perl's no-op dropped the period (KNOWN_PERL_ERRORS #795).
  RawTeX!(r"\def\DeclareCaptionTextFormat#1#2{\expandafter\long\expandafter\def\csname caption@textformat@#1\endcsname##1{#2}}
\DeclareCaptionTextFormat{simple}{#1}\DeclareCaptionTextFormat{period}{#1.}");
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
  // caption3.sty:399: its options processing ends by marking them processed, leaving `\@curroptions` as it was.
  DefMacro!("\\caption@ProcessOptions OptionalMatch:* {}", sub[_args] {
    key_options_processed()?;
    Ok(Tokens::default())
  });
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
  // The kernel's `\@dblarg` (KNOWN_PERL_ERRORS #468; caption.sty's `\caption` keeps latex.ltx's): a
  // package's `\let\@caption` to a `#1[#2]#3` macro finds its `[` (cond-mat0307356).
  DefMacro!("\\caption",
    r"\lx@donecaptiontrue\@ifundefined{@captype}{\maybe@@generic@caption}{\@ifstar{\@scaption}{\expandafter\@dblarg\expandafter{\expandafter\@caption\expandafter{\@captype}}}}"
  );
  // caption.sty:213-235: the starred caption is made through `\caption@setfloatcapt` too.
  DefMacro!("\\@scaption{}", "\\lx@setfloatcapt{\\lx@@caption{#1}}");

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
  // caption — the type's continuation state set (`\lx@caption@settype`) before the float
  // `\@captionof@` wraps the caption in, whose begin then opens nothing (`begin_float`, the
  // `\lx@caption@wrapper` one-shot) and sets `\@captype` inside its own group, as Perl's
  // `beforeFloat`. Not `\caption@settype` here: `\@captype` would stay set in the enclosing float,
  // whose `after_float` then rescues the other type's counters — a figure with `\caption` and a
  // `\captionof{table}` lost its number and every later figure and table shifted (2605.19656,
  // 2605.20199; guard `perfect_kernel_gemini::captionof_in_a_float_keeps_the_float_type`). A typed
  // `\caption` (`\maybe@@generic@caption`) reaches `\@captionof` with the type
  // `\captionsetup{type=…}` set, and sets none itself, as caption's `\caption`.
  DefMacro!("\\captionof", "\\@ifstar{\\lx@caption@of\\@scaptionof}{\\lx@caption@of\\@captionof}");
  // The type is expanded once, as `\caption@@settype`'s `\edef` (caption.sty:309).
  RawTeX!(r"\def\lx@caption@of#1#2{\edef\lx@caption@of@type{#2}\expandafter\lx@caption@of@\expandafter{\lx@caption@of@type}#1}
\def\lx@caption@of@#1#2{\lx@caption@settype{#1}#2{#1}}");
  // caption.sty:288-298 `\setcaptiontype[*]{type}[options]`: `\caption@settype`, in the group it
  // stands in (the options are typographic, `\caption@setoptions` a no-op here). 2605.20199's
  // figure sets its type back after a `\captionof{table}`.
  DefMacro!("\\setcaptiontype OptionalMatch:* {}[]", "\\caption@settype{#2}");
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
  // cleveref's `\label[type]{key}` after `\captionof`: the optional type is read and dropped, the key is the label — as
  // `\@caption@postlabel` does (the mechanism of 2404.11739; OXIDIZED_DESIGN #552). Guard
  // `perfect_kernel_batch65m::cleveref_label_type_after_caption`.
  DefMacro!("\\@captionof@postlabel{}{}{} SkipMatch:\\label [] Semiverbatim", r"\@captionof@{#1}{#2}{#3\label{#5}}");
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
  // So a verbatim-bodied type's caption is wrapped in the generic `{@float}{<type>}` instead (below),
  // in a group that sets `\@captype` to its type: `\@@add@caption@counters` steps `\@captype`, which
  // inside a figure was `figure` — "Listing 0" tagged "Figure 2" (guard
  // `perfect_kernel_gemini::captionof_verbatim_type_numbers_its_own_counter`) — and the group keeps
  // the type from the enclosing float's `after_float`. Non-verbatim types keep Perl's wrapper, since
  // that is what gives an unfloated `\captionof{figure}` its container. OXIDIZED_DESIGN #89.
  DefMacro!("\\@captionof@{}{}{}", sub[(ty, opt, text)] {
    let name = ty.to_string();
    // A verbatim-bodied type's own environment would read the rest of the input as its body, so its
    // wrapper is the kernel's generic `{@float}{<type>}` (Perl's `\@float`): a float of that type, which
    // `collapse_float` folds into a captionless figure or keeps beside a main caption — its `\label`
    // then names it (57bu re-review: in a group of its own the label fell to the section and the
    // listing's counters leaked to the next listing).
    let wrapper: Vec<Token> = if VERBATIM_BODY_ENVS.contains(&name.trim()) {
      let mut env = ExplodeText!("@float");
      env.push(T_END!());
      env.push(T_BEGIN!());
      env.extend(ExplodeText!(name.trim()));
      env
    } else {
      ExplodeText!(name.trim())
    };
    // The type, set in a group of its own around the wrapper: as `\caption@settype`, for an
    // environment that sets none (a non-float `\captionof{myfig}`, whose two errors are then
    // pdflatex's "No counter" and "No float type"), and never past the group. Before the wrapper's
    // `\begin`, which may read arguments (`\captionof{subfigure}`: its width, 2605.30420).
    let mut out = vec![T_CS!("\\begingroup"), T_CS!("\\def"), T_CS!("\\@captype"), T_BEGIN!()];
    out.extend(ExplodeText!(name.trim()));
    out.push(T_END!());
    out.push(T_CS!("\\lx@caption@wrapper"));
    out.push(T_CS!("\\begin"));
    out.push(T_BEGIN!());
    out.extend(wrapper.iter().cloned());
    out.push(T_END!());
    out.push(T_CS!("\\@caption@"));
    for arg in [&ty, &opt, &text] {
      out.push(T_BEGIN!());
      out.extend(arg.clone().unlist());
      out.push(T_END!());
    }
    out.push(T_CS!("\\end"));
    out.push(T_BEGIN!());
    out.extend(if VERBATIM_BODY_ENVS.contains(&name.trim()) {
      ExplodeText!("@float")
    } else {
      ExplodeText!(name.trim())
    });
    out.push(T_END!());
    out.push(T_CS!("\\lx@caption@wrapper@done"));
    out.push(T_CS!("\\endgroup"));
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
  // caption3.sty:850-852 `\caption@setfont{kind}{value}` defines `\caption<kind>` from the font
  // options (font/labelfont/textfont/size); floatrow's `\flrow@setfont` (floatrow.sty:1021-1023)
  // then `\let`s its `\floatfont` to `\caption@tempa`, used by every float (:478, :775, :820-822).
  // The font itself is presentation (as `\DeclareCaptionFont` and `\captionfont` here), so the
  // macro is defined empty — its first line; the former no-op left `\caption@tempa` undefined:
  // `\floatfont` undefined and an `ltx:ERROR` in every floatrow float (kaytannollista-latexia: 120,
  // floatrow-rus, makecell-rus). Witness 2504.00326.
  DefMacro!("\\caption@setfont{}{}", "\\expandafter\\let\\csname caption#1\\endcsname\\@empty");
  // caption.sty:392-395 `\phantomcaption` is `\caption@refstepcounter\@captype` (outside a
  // float type, caption's error — nothing here): the float's counter steps and it gets its tag and
  // id, but no caption, toccaption or list entry. In a float that is the caption counters' step,
  // less the list entry (`\lx@caption@phantom@nolist`), marked as `\caption` marks it
  // (`\lx@donecaptiontrue`, sect09): a sub-float after it must not pre-increment the float's
  // counter again (2503.21681: a figure opening with `\phantomcaption` then `{subfigure}`s is
  // Figure 4 with 4a, 4b, as pdflatex, not 5a, 5b); with `\@captype` set outside one
  // (`\captionsetup{type=…}`), the counter step alone. subcaption's bare `\subfloat{…}`
  // (subcaption.sty:293-300) and subfig's (subfig.sty:348-349, :391) number a sub-float this way.
  // Was a no-op stub (witness 2503.21681). Guard
  // `perfect_kernel_gemini::bare_subfloat_has_a_phantom_caption`.
  DefMacro!("\\phantomcaption", sub[_args] {
    Ok(if !has_meaning(&T_CS!("\\@captype")) {
      Tokens!()
    } else if lookup_bool("lx@in@float") {
      Tokens!(
        T_CS!("\\lx@donecaptiontrue"),
        T_CS!("\\@@add@caption@counters"),
        T_CS!("\\lx@caption@phantom@nolist")
      )
    } else {
      Tokens!(T_CS!("\\refstepcounter"), T_BEGIN!(), T_CS!("\\@captype"), T_END!())
    })
  });
  // The list entry `\@@add@caption@counters` recorded for the float, keyed exactly as it stores
  // it (`<captype>_inlist`, the captype untrimmed, latex_constructs sect09).
  DefPrimitive!("\\lx@caption@phantom@nolist", {
    let captype = do_expand(T_CS!("\\@captype"))?.to_string();
    remove_value(&s!("{captype}_inlist"));
  });
  // subcaption.sty:252 `\phantomsubcaption` = `\setcaptionsubtype*\phantomcaption`: the sub-type's
  // counter (`sub<captype>`) steps — `\phantomcaption` itself inside a sub-float, whose
  // `\@captype` is already the sub-type.
  DefMacro!("\\phantomsubcaption", sub[_args] {
    if !has_meaning(&T_CS!("\\@captype")) {
      return Ok(Tokens!());
    }
    let captype = do_expand(T_CS!("\\@captype"))?.to_string();
    Ok(if captype.trim().starts_with("sub") {
      Tokens!(T_CS!("\\phantomcaption"))
    } else {
      Tokens!(T_CS!("\\refstepcounter"), T_BEGIN!(), Explode!("sub"), T_CS!("\\@captype"), T_END!())
    })
  });
  // caption.sty:633-638 `\subcaptionsetup*[<type>]{<options>}` = `\captionsetup*[sub<type>]{<options>}`: the
  // options of the sub-captions (of every type when `[<type>]` is absent: `[sub]`, which subcaption's
  // `\captionsetup[sub]` reads too). Run 340: 148 first errors (witnesses 2508.10437, 2406.09241, 2406.12144).
  DefMacro!("\\subcaptionsetup OptionalMatch:* []{}", "\\captionsetup#1[sub#2]{#3}");
  // caption.sty:396-413 `\captionlistentry*[<type>]{<entry>}`: the list-of entry of a float that has no
  // `\caption`; unstarred it steps the counter as a caption would (`\caption@refstepcounter`, :402-410),
  // so a `\label` after it refers to the float. In a float that is `\phantomcaption`'s step keeping the
  // float's list entry (an `[<type>]` steps that counter, as `\caption@refstepcounter@`); starred, only
  // the entry. The entry's text is the List of Figures' only and has no place in the document (the
  // list is built from the floats). Outside a float with no `[<type>]`, caption's
  // `\caption@OutsideFloat` error. In subcaption a sub-float's entry steps the sub-counter
  // (caption.sty:660 `\let\captionlistentry\caption@sublistentry`), which `\@captype` is here.
  // Run 340: 207 first errors (witnesses 2604.10403, 2103.15767, 2103.01737).
  DefMacro!("\\captionlistentry OptionalMatch:* []{}", sub[(star, captype, _entry)] {
    if star.is_some() {
      return Ok(Tokens!());
    }
    if let Some(captype) = captype {
      let mut tokens = vec![T_CS!("\\refstepcounter"), T_BEGIN!()];
      tokens.extend(captype.unlist());
      tokens.push(T_END!());
      return Ok(Tokens::new(tokens));
    }
    Ok(if !has_meaning(&T_CS!("\\@captype")) {
      Error!("unexpected", "\\captionlistentry", "\\captionlistentry outside float");
      Tokens!()
    } else if lookup_bool("lx@in@float") {
      Tokens!(T_CS!("\\lx@donecaptiontrue"), T_CS!("\\@@add@caption@counters"))
    } else {
      Tokens!(T_CS!("\\refstepcounter"), T_BEGIN!(), T_CS!("\\@captype"), T_END!())
    })
  });
  // caption.sty:415-428 `\captiontext*[<number>]{<text>}`: the float's caption typeset without a counter
  // step or a list entry — unstarred the caption (`\caption@@make`), starred the same unboxed
  // (caption3.sty:1661 `\caption@@@make`): the label, the separator and the text, as papers set a panel's "(a)" over its graphic with
  // `\captiontext*{}` after a `\phantomcaption`. `[<number>]` is the counter's value for it, in a group
  // (`\caption@@@text`). The label is the type's `\lx@fnum@@`, as a caption's; a sub-float's
  // (`\@captype` a sub-type in subcaption's groups and sub-floats: caption.sty:661
  // `\let\captiontext\caption@subtext`) is its "(a)". Outside a float, caption's `\caption@OutsideFloat`
  // error, the arguments dropped. Run 340: 10 first errors (witnesses 2407.10358, 2409.07679, 2309.05387).
  DefMacro!("\\captiontext OptionalMatch:* []{}", sub[(_star, number, text)] {
    if !has_meaning(&T_CS!("\\@captype")) {
      Error!("unexpected", "\\captiontext", "\\captiontext outside float");
      return Ok(Tokens!());
    }
    let captype = do_expand(T_CS!("\\@captype"))?.to_string();
    let captype = captype.trim();
    let mut tokens = vec![T_CS!("\\begingroup")];
    if let Some(number) = number {
      tokens.push(T_CS!(&s!("\\c@{captype}")));
      tokens.push(T_OTHER!("="));
      tokens.extend(number.unlist());
      tokens.push(T_CS!("\\relax"));
    }
    tokens.extend([T_CS!("\\lx@fnum@@"), T_BEGIN!()]);
    tokens.extend(Explode!(captype));
    tokens.push(T_END!());
    // With text, the label's separator before it, the ": " of a kernel caption (`\format@title@<type>`, sect09.rs),
    // here as text: what `\captiontext` sets is text in the float, not its `ltx:caption`. Without text, the label
    // alone (caption3's empty-text form, the "(a)" over a panel).
    if !text.is_empty() {
      tokens.extend(ExplodeText!(": "));
      tokens.extend(text.unlist());
    }
    tokens.push(T_CS!("\\endgroup"));
    Ok(Tokens::new(tokens))
  });

  // caption3.sty:1468-1483, :1661-1715: caption formats a float's caption itself — the label is the label format
  // applied to the type's name and number (`\caption@fnum`), the separator the label separator's, the fonts and the
  // text format caption's options — where the kernel's `\@makecaption` sets "Figure 1: text". The kernel's caption
  // hook (`\lx@format@caption@@`, sect09.rs) takes the settings `\captionsetup` and the package options store
  // (`caption_format_tokens`). Perl's binding stores them and reads none (caption.sty.ltxml:62-68; KNOWN_PERL_ERRORS
  // #795), so `labelsep=period`, `name=Fig.`, `labelformat=parens`, `labelfont=bf` made no difference.
  DefMacro!("\\lx@format@caption@@{}{}", sub[(ty, text)] { caption_format_tokens(ty, text) });
  // caption3.sty:1468-1472 `\caption@fnum`: the label format's name argument is `\@nameuse{<type>name}`, empty when
  // the type has none.
  RawTeX!(r"\def\lx@caption@fnum#1#2{\expandafter\def\csname fnum@#2\endcsname{\csname caption@labelformat@#1\endcsname{\ifcsname #2name\endcsname\csname #2name\endcsname\fi}{\lx@the@@{#2}}}}
\def\lx@caption@settitle#1#2{\@namedef{format@title@#1}##1{\lx@tag[][#2]{\lx@fnum@@{#1}}##1}}
\def\lx@caption@addfont#1#2{\@ifundefined{#1}{\@namedef{#1}{}}{}\expandafter\l@addto@macro\csname #1\endcsname{#2}}");

  // caption3.sty:1774-1806 `\DeclareCaptionSubType*[<numbering>]{<type>}`: the sub-type `sub<type>`, its counter
  // reset by the type's (`\newcounter{sub<type>}[<type>]`) unless a package made it — caption's error, the
  // numbering still set — its number `\<numbering>{sub<type>}` referenced after the type's (`\p@sub<type>`) or, starred,
  // printed after it with a dot (`\thefigure.\alph{subfigure}`). caption3's empty `\sub<type>name` (:1791) is left
  // out, as subcaption_sty.rs does (a defined name makes the `typerefnum` tag). The TOC entry (`\l@sub<type>`, :1795) is
  // the list's, built here from the floats. Was a no-op in subcaption_sty.rs and undefined with caption alone (39
  // papers' first error in run 336: 1202.5818, 1604.05412, 2002.03129, 2106.01798, 1706.00437, 2405.16148, 1907.05468,
  // 2512.12259); Perl has no binding for it either (KNOWN_PERL_ERRORS #795).
  RawTeX!(r"\newcommand*\caption@subtypelist{}
\def\caption@ifdefined@subtype#1{\edef\lx@caption@tempa{\noexpand\in@{,#1,}{,\caption@subtypelist,}}\lx@caption@tempa\ifin@\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi}
\def\lx@caption@addsubtype#1{\caption@ifdefined@subtype{#1}{}{\edef\caption@subtypelist{\ifx\caption@subtypelist\@empty\else\caption@subtypelist,\fi#1}}}
\newcommand*\DeclareCaptionSubType{\@ifstar{\caption@declaresubtype\@firstoftwo}{\caption@declaresubtype\@secondoftwo}}
\newcommand*\caption@declaresubtype[1]{\@testopt{\caption@@declaresubtype#1}{alph}}
\def\caption@@declaresubtype#1[#2]#3{\@ifundefined{c@#3}{\caption@Error{No float type '#3' defined}}{\caption@@@declaresubtype{#1}{#2}{#3}{sub#3}}}
\newcommand*\caption@@@declaresubtype[4]{%
  \caption@ifdefined@subtype{#4}{}{%
    \@ifundefined{c@#4}%
      {\lx@caption@newsubcounter{#4}{#3}%
       \@namedef{#4autorefname}{\csname #3autorefname\endcsname}%
       \@namedef{ext@#4}{\csname ext@#3\endcsname}%
       \lx@caption@addsubtype{#4}}%
      {\caption@subtypesource{#4}\caption@Error{The counter `#4' was already defined by\MessageBreak\caption@subtype@source}}}%
  #1{\@namedef{p@#4}{}\@namedef{the#4}{\csname the#3\endcsname.\@nameuse{#2}{#4}}}%
    {\@namedef{p@#4}{\csname the#3\endcsname}\@namedef{the#4}{\@nameuse{#2}{#4}}}%
  \@namedef{theH#4}{\csname theH#3\endcsname.\arabic{#4}}}
\def\caption@subtypesource#1{\def\caption@subtype@source{an unknown document class or package}%
  \ifcsname sf@counterlist\endcsname\edef\lx@caption@tempa{\noexpand\in@{,#1,}{,\sf@counterlist,}}\lx@caption@tempa
    \ifin@\def\caption@subtype@source{\noexpand\newsubfloat (offered by the subfig package)}\fi\fi}
\@onlypreamble\DeclareCaptionSubType");

  // caption3.sty:1788 `\newcounter{sub<type>}[<type>]`, its ids within the type's as subcaption's (`sf` for a
  // sub-figure, subcaption_sty.rs): `s` and the type's initial.
  DefPrimitive!("\\lx@caption@newsubcounter{}{}", sub[(sub_type, parent)] {
    let (sub_type, parent) = (sub_type.to_string(), parent.to_string());
    let idprefix = s!("s{}", parent.trim().chars().next().map(|c| c.to_ascii_lowercase()).unwrap_or('s'));
    NewCounter!(sub_type.trim(), parent.trim(), idprefix => &idprefix, idwithin => parent.trim());
  });

  // caption.sty:171-172: the package options are `\captionsetup` keys (`\caption@ProcessOptions*{caption}`), with the
  // caption v1 options (:114-147, valid only here): a font shape or series for the label (`bf` is
  // `labelfont+=bf`), a size for the caption (`small`), the rest layout. Ignored before, so
  // `\usepackage[labelsep=period]{caption}` kept "Figure 1:".
  RawTeX!(r"\def\lx@caption@optionsetup#1{\captionsetup{#1}}");
  DeclareOption!(None, {
    let option = current_option_text()?;
    let option = option.trim();
    let setup = match option {
      "up" | "it" | "sl" | "sc" | "md" | "bf" | "rm" | "sf" | "tt" => s!("labelfont+={option}"),
      // caption.sty:124-129: a size option is the whole caption font (`\def\captionfont{\small}`), replacing `font=`.
      "scriptsize" | "footnotesize" | "small" | "normalsize" | "large" | "Large" => s!("font={option}"),
      "normal" | "isu" | "hang" | "center" | "centerlast" | "nooneline" | "ruled" | "flushleft"
      | "flushright" | "oneline" | "ignoreLTcapwidth" => String::new(),
      // The option's own tokens, unexpanded, as `\caption@ProcessOptions*` passes them on (caption.sty:171-172):
      // `figurename=\textsc{Fig}` is the name, not `\textsc`'s expansion.
      _ => {
        Digest!(r"\expandafter\lx@caption@optionsetup\expandafter{\CurrentOption}")?;
        String::new()
      },
    };
    if !setup.is_empty() {
      Digest!(tokenize(TeXString::assembled(s!(r"\captionsetup{{{setup}}}"))))?;
    }
  });
  ProcessKeyOptions!();
});

/// `\CurrentOption`'s text, unexpanded: the option list was read once — `\@pass@ptions` builds `\opt@caption.sty` by a
/// `\protected@xdef` (latex.ltx:18514), which caption.sty:172 `\caption@ProcessOptions*` reads through caption3.sty:398
/// `\@ptionlist` (our engine's path is `\ProcessOptions`' `\protected@edef`, latex.ltx:18557) — and caption hands it to
/// its keys by `\@expandtwoargs` (caption3.sty:374-399), another `\protected@edef` (latex.ltx:1381-1382) that leaves it
/// as it is — so the v1-keyword match reads the macro's body. Expanding it ran a robust command's body
/// at package load (`\DeclareRobustCommand\rb{\laterdefined}` + `[figurename=\rb]{caption}`: an undefined-command
/// Error pdflatex does not give). Guard: `perfect_kernel_batch68h::caption_option_key_is_read_unexpanded`.
pub fn current_option_text() -> Result<String> {
  Ok(match lookup_definition(&T_CS!("\\CurrentOption"))? {
    Some(defn) => match defn.get_expansion() {
      Some(ExpansionBody::Tokens(body)) => body.to_string(),
      _ => String::new(),
    },
    None => String::new(),
  })
}

/// A `\captionsetup` value as text: one pair of enclosing braces is keyval's grouping (`font={small,bf}`).
fn caption_value(value: &str) -> String {
  let value = value.trim();
  match value.strip_prefix('{').and_then(|v| v.strip_suffix('}')) {
    Some(inner)
      if inner.chars().filter(|c| *c == '{').count()
        == inner.chars().filter(|c| *c == '}').count() =>
    {
      inner.trim().to_string()
    },
    _ => value.to_string(),
  }
}

/// A caption setting for float type `ty`: the type's own (`\captionsetup[figure]{…}`), else the general one.
fn caption_setting(ty: &str, key: &str) -> Option<String> {
  [s!("CAPTION_{ty}_{key}"), s!("CAPTION_{key}")]
    .iter()
    .map(|k| lookup_string(k))
    .find(|v| !v.is_empty())
}

/// caption3.sty:852-858: a font option's value is a list of caption fonts (`font={small,it}`, `labelfont=bf`,
/// `font={color=blue}`), each `\caption@font@<name>{<value>}`; an undeclared one is caption's error.
fn caption_font_tokens(spec: &str) -> Result<Vec<Token>> {
  let mut tokens = Vec::new();
  let mut depth = 0i32;
  let mut items = vec![String::new()];
  for c in spec.chars() {
    match c {
      '{' => depth += 1,
      '}' => depth -= 1,
      ',' if depth == 0 => {
        items.push(String::new());
        continue;
      },
      _ => {},
    }
    if let Some(last) = items.last_mut() {
      last.push(c);
    }
  }
  for item in items {
    // keyval strips one pair of braces around an item (`font={{stretch=1},footnotesize}`, 1706.05280).
    let item = caption_value(&item);
    let (name, value) = match item.split_once('=') {
      Some((name, value)) => (name.trim().to_string(), caption_value(value)),
      None => (item.trim().to_string(), String::new()),
    };
    if name.is_empty() {
      continue;
    }
    let font = T_CS!(&s!("\\caption@font@{name}"));
    if is_defined_token(&font) {
      tokens.push(font);
      tokens.push(T_BEGIN!());
      tokens.extend(tokenize(TeXString::assembled(value)).unlist());
      tokens.push(T_END!());
    } else {
      Error!("undefined", &name, &s!("Undefined font `{name}'"));
    }
  }
  Ok(tokens)
}

/// caption3.sty:792-797 `textformat=empty`: the caption text is dropped, its `\label`s kept.
fn caption_labels_only(text: Tokens) -> Tokens {
  let tokens = text.unlist();
  let mut kept = Vec::new();
  let mut i = 0;
  while i < tokens.len() {
    if tokens[i] == T_CS!("\\label") {
      kept.push(tokens[i]);
      i += 1;
      let mut depth = 0;
      while i < tokens.len() {
        let token = tokens[i];
        kept.push(token);
        i += 1;
        match token.get_catcode() {
          Catcode::BEGIN => depth += 1,
          Catcode::END => {
            depth -= 1;
            if depth <= 0 {
              break;
            }
          },
          _ if depth == 0 => break,
          _ => {},
        }
      }
    } else {
      i += 1;
    }
  }
  Tokens::new(kept)
}

/// caption3.sty:1713-1748 `\caption@ifempty`: a caption text is empty when nothing is left once its spaces, a leading
/// `\ignorespaces` and its `\label{…}`, `\index{…}` and `\glossary{…}` are taken away (`\caption{\label{f3}}`,
/// pdflatex "Figure 3" with no separator); a group, even an empty one, is text.
fn caption_text_is_empty(text: &Tokens) -> bool {
  let tokens = text.unlist_ref();
  let mut i = 0;
  let skip_spaces = |i: &mut usize| {
    while tokens
      .get(*i)
      .is_some_and(|t| t.get_catcode() == Catcode::SPACE)
    {
      *i += 1;
    }
  };
  loop {
    skip_spaces(&mut i);
    let Some(token) = tokens.get(i) else {
      return true;
    };
    i += 1;
    if *token == T_CS!("\\ignorespaces") {
      continue;
    }
    if *token == T_CS!("\\label") || *token == T_CS!("\\index") || *token == T_CS!("\\glossary") {
      // `\@gobbletwo`: the command and its argument, a group or one token.
      skip_spaces(&mut i);
      if tokens
        .get(i)
        .is_some_and(|t| t.get_catcode() == Catcode::BEGIN)
      {
        let mut depth = 0;
        while let Some(t) = tokens.get(i) {
          i += 1;
          match t.get_catcode() {
            Catcode::BEGIN => depth += 1,
            Catcode::END => {
              depth -= 1;
              if depth == 0 {
                break;
              }
            },
            _ => {},
          }
        }
      } else {
        i += 1;
      }
      continue;
    }
    return false;
  }
}

/// A float's caption as caption formats it (caption3.sty:1661-1715), from its settings for the type: the type's name
/// (`name=`), its label format (`labelformat=`, the type's `\fnum@<type>` for `original`/`default`), its label
/// separator (`labelsep=`, none for an empty caption, :1680-1683), its fonts (`font=` for the whole caption,
/// `labelfont=` for the label, `textfont=` for the text) and its text format (`textformat=`). With none set the
/// caption is the class's (`\lx@format@title@@`). Sub-captions keep their packages' formats (subcaption_sty.rs,
/// subfig_sty.rs).
fn caption_format_tokens(ty: Tokens, text: Tokens) -> Result<Tokens> {
  let ty_name = ty.to_string().trim().to_string();
  let text_empty = caption_text_is_empty(&text);
  let get = |key: &str| caption_setting(&ty_name, key);
  let format = get("labelformat").filter(|f| f != "default" && f != "original");
  let separator = get("labelsep").filter(|s| s != "default");
  let (font, label_font, text_font) = (get("font"), get("labelfont"), get("textfont"));
  // caption3.sty:1680-1683: an empty text also takes the `simple` text format (`textformat=period` adds no ".").
  let text_format = get("textformat").filter(|f| f != "simple" && f != "default" && !text_empty);
  let name = lookup_tokens(&s!("CAPTION_{ty_name}_name@tokens"));
  let mut tokens = Vec::new();
  let mut line_break: Vec<Token> = Vec::new();
  // No type (a caption whose type an argument scan lost) has nothing to format.
  let passthrough = ty_name.is_empty()
    || ty_name.starts_with("sub")
    || (format.is_none()
      && separator.is_none()
      && font.is_none()
      && label_font.is_none()
      && text_font.is_none()
      && text_format.is_none()
      && name.is_none()
      && !text_empty);
  let explode = |s: &str| ExplodeText!(s);
  let braced = |tokens: &mut Vec<Token>, inner: Vec<Token>| {
    tokens.push(T_BEGIN!());
    tokens.extend(inner);
    tokens.push(T_END!());
  };
  if !passthrough {
    tokens.push(T_CS!("\\begingroup"));
    if let Some(name) = name {
      tokens.push(T_CS!("\\lx@caption@setname"));
      braced(&mut tokens, explode(&ty_name));
      braced(&mut tokens, name.unlist());
    }
    let font_tokens = font
      .as_deref()
      .map(caption_font_tokens)
      .transpose()?
      .unwrap_or_default();
    if font.is_some() || label_font.is_some() {
      let mut fonts = font_tokens.clone();
      fonts.extend(
        label_font
          .as_deref()
          .map(caption_font_tokens)
          .transpose()?
          .unwrap_or_default(),
      );
      tokens.push(T_CS!("\\lx@caption@addfont"));
      braced(&mut tokens, explode(&s!("fnum@font@{ty_name}")));
      braced(&mut tokens, fonts);
    }
    if font.is_some() || text_font.is_some() {
      let mut fonts = font_tokens;
      fonts.extend(
        text_font
          .as_deref()
          .map(caption_font_tokens)
          .transpose()?
          .unwrap_or_default(),
      );
      tokens.push(T_CS!("\\lx@caption@addfont"));
      braced(&mut tokens, explode(&s!("format@title@font@{ty_name}")));
      braced(&mut tokens, fonts);
    }
    if let Some(format) = format {
      if is_defined(&s!("\\caption@labelformat@{format}")) {
        tokens.push(T_CS!("\\lx@caption@fnum"));
        braced(&mut tokens, explode(&format));
        braced(&mut tokens, explode(&ty_name));
      } else {
        Error!(
          "undefined",
          &format,
          &s!("Undefined label format `{format}'")
        );
      }
    }
    // caption3.sty:1680-1683: an empty caption text (`caption_text_is_empty`) has no separator. A separator that
    // breaks the line (`\caption@LabelSeparatorWithNewline`, caption3.sty:769, :782: `newline`) is the text's first
    // line break, the tag's `close` being text.
    let separator_cs = if text_empty {
      Some(Vec::new())
    } else if let Some(separator) = separator {
      if is_defined(&s!("\\caption@lsep@{separator}")) {
        let crlist = do_expand(T_CS!("\\caption@labelsep@crlist"))?.to_string();
        if crlist.split(',').any(|item| item.trim() == separator) {
          line_break = vec![T_CS!(&s!("\\caption@lsep@{separator}"))];
          Some(Vec::new())
        } else {
          Some(vec![T_CS!(&s!("\\caption@lsep@{separator}"))])
        }
      } else {
        Error!(
          "undefined",
          &separator,
          &s!("Undefined label separator `{separator}'")
        );
        None
      }
    } else {
      None
    };
    if let Some(separator_cs) = separator_cs {
      tokens.push(T_CS!("\\lx@caption@settitle"));
      braced(&mut tokens, explode(&ty_name));
      braced(&mut tokens, separator_cs);
    }
  }
  let text = match text_format.filter(|_| !passthrough) {
    Some(f) if f == "empty" => caption_labels_only(text),
    Some(f) if is_defined(&s!("\\caption@textformat@{f}")) => {
      let mut wrapped = vec![T_CS!(&s!("\\caption@textformat@{f}"))];
      braced(&mut wrapped, text.unlist());
      Tokens::new(wrapped)
    },
    Some(f) => {
      Error!("undefined", &f, &s!("Undefined text format `{f}'"));
      text
    },
    None => text,
  };
  tokens.push(T_CS!("\\lx@format@title@@"));
  braced(&mut tokens, ty.unlist());
  line_break.extend(text.unlist());
  braced(&mut tokens, line_break);
  if !passthrough {
    tokens.push(T_CS!("\\endgroup"));
  }
  Ok(Tokens::new(tokens))
}

/// caption.sty:504-511, :519-533: a float continues only the type stepped last
/// (`\continuedfloat@captype`, set on every real step), else caption's error; then
/// `continuedfloat` steps, the float's next step keeps its number when `suppress_step`
/// (`\caption@setcontinued`, a global flag the float's next step uses up, :590-599, or the next
/// float's begin clears, :173-192, :300-303;
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

/// A sub-caption's label (`\fnum@sub<type>`): its label format applied to the sub-float's
/// `number` (caption3.sty:734-737). The format is the first non-empty `\captionsetup` setting among
/// `keys` (state keys, the most specific first: `CAPTION_sub<type>_labelformat`, then the package's
/// family — subcaption's `[sub]`, subfig's `[subfloat]`), else the package's `default` (`parens` for
/// both: subcaption.sty:218-222, subfig.sty:285-288). An undefined format is caption3's
/// `\caption@Error{Undefined label format}`, the label keeping the default.
pub fn sub_label_tokens(keys: &[String], default: &str, number: Tokens) -> Result<Tokens> {
  let format = keys
    .iter()
    .map(|key| lookup_string(key))
    .find(|value| !value.is_empty())
    .unwrap_or_else(|| default.to_string());
  let mut formatter = T_CS!(&s!("\\caption@labelformat@{}", format.trim()));
  if !is_defined_token(&formatter) {
    Error!(
      "undefined",
      format.trim(),
      &s!("Undefined label format `{}'", format.trim())
    );
    formatter = T_CS!(&s!("\\caption@labelformat@{default}"));
  }
  let mut tokens = vec![formatter, T_BEGIN!(), T_END!(), T_BEGIN!()];
  tokens.extend(number.unlist());
  tokens.push(T_END!());
  Ok(Tokens::new(tokens))
}
