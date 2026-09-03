use crate::prelude::*;

mod declare;
use declare::*;

LoadDefinitions!({
  // Perl latexml.sty.ltxml L31-35: ids/noids and comments/nocomments expose
  // two well-known boolean knobs to the document author. Both state keys
  // (GENERATE_IDS, INCLUDE_COMMENTS) are read elsewhere in Rust (document.rs
  // L459 and mouth.rs L358/L696/L889 respectively), so the options were
  // functional but unreachable until wired here.
  DeclareOption!("ids", {
    AssignValue!("GENERATE_IDS"     => true,  Scope::Global);
  });
  DeclareOption!("noids", {
    AssignValue!("GENERATE_IDS"     => false, Scope::Global);
  });
  DeclareOption!("comments", {
    AssignValue!("INCLUDE_COMMENTS" => true,  Scope::Global);
  });
  DeclareOption!("nocomments", {
    AssignValue!("INCLUDE_COMMENTS" => false, Scope::Global);
  });

  // 'nobibtex': used for arXiv-like build harnesses where only ".bbl" is available
  // (bibtex will not be ran). 'bibtex' is the default (try bib, fall back to bbl).
  DeclareOption!("bibtex", {
    AssignValue!(
      "BIB_CONFIG",
      Stored::Strings(Rc::new([pin("bib"), pin("bbl")])),
      Scope::Global
    );
  });
  DeclareOption!("nobibtex", {
    AssignValue!(
      "BIB_CONFIG",
      Stored::Strings(Rc::new([pin("bbl")])),
      Scope::Global
    );
  });

  // Perl L57-59: bibconfig KeyVal — comma-separated bib config values.
  DefKeyVal!("LTXML", "bibconfig", "Semiverbatim");

  // Perl L63-86: Image scaling options — saved as processing instructions
  // via \lx@save@parameter at \begin{document} time. Perl's user-facing
  // keyval name is lowercase `dpi` but the internal PI is uppercase `DPI`
  // (Perl: `$STATE->assignValue(DPI => ...)`). Keep the keyval name
  // lowercase to match Perl user-facing — the uppercase `DPI` mismatch
  // meant `\usepackage[dpi=144]{latexml}` silently missed the keyval.
  DefKeyVal!("LTXML", "dpi", "Number");
  DefKeyVal!("LTXML", "magnify", "Number");
  DefKeyVal!("LTXML", "upsample", "Number");
  DefKeyVal!("LTXML", "zoomout", "Number");

  // Perl L87-98: Limit options — set global limits for infinite-loop protection.
  // These are DefKeyVal with code closures; since our macro doesn't support code,
  // we define them as DeclareOption and handle in ProcessOptions.
  DefKeyVal!("LTXML", "tokenlimit", "Number");
  DefKeyVal!("LTXML", "iflimit", "Number");
  DefKeyVal!("LTXML", "absorblimit", "Number");
  DefKeyVal!("LTXML", "pushbacklimit", "Number");

  // Lexeme serialization for math formulas
  DeclareOption!("mathlexemes", {
    AssignValue!("LEXEMATIZE_MATH" => true, Scope::Global);
  });

  // Math parser speculation (e.g. possible function detection)
  // Perl: DeclareOption('mathparserspeculate', sub { AssignValue('MATHPARSER_SPECULATE' => 1,
  // 'global'); });
  DeclareOption!("mathparserspeculate", {
    AssignValue!("MATHPARSER_SPECULATE" => true, Scope::Global);
  });
  DeclareOption!("nomathparserspeculate", {
    AssignValue!("MATHPARSER_SPECULATE" => false, Scope::Global);
  });

  // Header guessing for tabular environments
  DeclareOption!("guesstabularheaders", {
    AssignValue!("GUESS_TABULAR_HEADERS" => true, Scope::Global);
  });
  DeclareOption!("noguesstabularheaders", {
    AssignValue!("GUESS_TABULAR_HEADERS" => false, Scope::Global);
  });

  // Styling options (Perl PR #2767)
  DeclareOption!("authorsoneline", {
    assign_mapping(
      "DOCUMENT_CLASSES",
      "ltx_authors_1line",
      Some(Stored::Bool(true)),
    );
    assign_mapping("DOCUMENT_CLASSES", "ltx_authors_multiline", None::<Stored>);
  });
  DeclareOption!("authorsmultiline", {
    assign_mapping(
      "DOCUMENT_CLASSES",
      "ltx_authors_multiline",
      Some(Stored::Bool(true)),
    );
    assign_mapping("DOCUMENT_CLASSES", "ltx_authors_1line", None::<Stored>);
  });

  // Finer control over which (if any) raw .sty/.cls files to include
  DeclareOption!("rawstyles", {
    AssignValue!("INCLUDE_STYLES"  => true, Scope::Global);
  });
  DeclareOption!("localrawstyles", {
    AssignValue!("INCLUDE_STYLES"  => "searchpaths", Scope::Global);
  });
  DeclareOption!("norawstyles", {
    AssignValue!("INCLUDE_STYLES"  => false,             Scope::Global);
  });
  DeclareOption!("rawclasses", {
    AssignValue!("INCLUDE_CLASSES" => true,             Scope::Global);
  });
  DeclareOption!("localrawclasses", {
    AssignValue!("INCLUDE_CLASSES" => "searchpaths", Scope::Global);
  });
  DeclareOption!("norawclasses", {
    AssignValue!("INCLUDE_CLASSES" => false, Scope::Global);
  });

  // Opt-in LuaTeX profile (user decision 2026-08-31, perfect-kernel
  // mission): `\usepackage[luatex]{latexml}` declares the DOCUMENT as
  // LuaLaTeX-authored. Engine-detection probes flip to LuaTeX and the
  // `\directlua` escape becomes available under its REAL name (elsewhere
  // deliberately absent — it is the detection probe babel & friends use).
  // The default engine identity (pdfTeX-model) is untouched without this
  // option. Rust-only option; divergence documented as OXIDIZED_DESIGN #168.
  DeclareOption!("luatex", {
    AssignValue!("LUATEX_PROFILE" => true, Scope::Global);
    RawTeX!(
      r"\let\iftutex\iftrue \let\ifluatex\iftrue \let\ifpdftex\iffalse \let\ifLuaTeX\iftrue \let\ifPDFTeX\iffalse"
    );
    RawTeX!(r"\let\directlua\lx@directlua \let\luaescapestring\lx@luaescapestring");
    RawTeX!(r"\def\luatexversion{121} \def\luatexrevision{0} \def\directluaversion{1}");
    // l3sys froze its engine identity at FORMAT-build time (expl3-code.tex:7846-7861:
    // `\str_const:Ne \c_sys_engine_str {\cs_if_exist:NT \tex_luatexversion:D {luatex} …}`
    // and one `\__sys_const:nn {sys_if_engine_<e>}` per engine), when
    // `\luatexversion` did not exist yet, so `\sys_if_engine_luatex:TF` stayed FALSE
    // under this profile and polyglossia's gloss-latin.ldf:125 took its XeTeX branch
    // (`\newXeTeXintercharclass` ×12; hang, sample). Re-derive the constants the
    // way :7860 does, with the engine now known; `\c_sys_engine_exec_str` (:7868-7884)
    // and `\c_sys_engine_format_str` (:7886-7916, LaTeX2e format) follow.
    // Guard: `perfect_kernel_batch54::l3sys_engine_identity_under_luatex_profile`.
    RawTeX!(
      r"\ExplSyntaxOn
        \tl_gset:Nn \c_sys_engine_str { luatex }
        \tl_map_inline:nn { { pdftex } { ptex } { uptex } { xetex } }
          {
            \cs_gset_eq:cN { sys_if_engine_ #1 :T }  \use_none:n
            \cs_gset_eq:cN { sys_if_engine_ #1 :F }  \use:n
            \cs_gset_eq:cN { sys_if_engine_ #1 :TF } \use_ii:nn
            \cs_gset_eq:cN { sys_if_engine_ #1 _p: } \c_false_bool
          }
        \cs_gset_eq:cN { sys_if_engine_luatex :T }  \use:n
        \cs_gset_eq:cN { sys_if_engine_luatex :F }  \use_none:n
        \cs_gset_eq:cN { sys_if_engine_luatex :TF } \use_i:nn
        \cs_gset_eq:cN { sys_if_engine_luatex _p: } \c_true_bool
        \tl_gset:Nn \c_sys_engine_exec_str { luatex }
        \tl_gset:Nn \c_sys_engine_format_str { lualatex }
        \ExplSyntaxOff"
    );
    // The lualatex FORMAT surface: latex.ltx:107-110 enables the LuaTeX
    // extra primitives and :896-1058 (= ltluatex.dtx `2ekernel`) defines the
    // allocators every LuaTeX-branch package assumes — luaotfload.sty:38
    // `\input ltluatex` (a plain \input, bypassing the binding registry),
    // luacolor.sty:161-166 `\setattribute`, tuenc.def:77 `\newprotectedluacmd`,
    // babel's `\prehyphenchar`. Without them the first such name was the
    // first error of 17 lualatex-oracle manuals (sunpath ×2, highlightx,
    // creationboites, tkz-bernoulli, tabularray-abnt, asmeconf, minted-code,
    // abntexto ×2, homework-jp, DEMO-BFHSciPoster, beamer-amurmaple,
    // mathfont-symbol-list, greek-fontenc ×2, biblatex-german-legal).
    // Guard: `perfect_kernel_batch54::ltluatex_format_surface_under_luatex_profile`.
    //
    // Primitives (LuaTeX manual §3.4 attributes, §3.5 catcode tables, §5.2
    // hyphenation): `\attributedef` allocates into a private `\attribute`
    // bank (NOT `\count`: attributes 0,1,2… would alias the page registers);
    // the catcode-table switches scan their <number> and do nothing (catcode
    // tables are engine state with no XML meaning); `\luadef` binds a Lua
    // function slot the bridge cannot run → the command is a no-op.
    DefPrimitive!("\\attributedef SkipSpaces Token SkipSpaces SkipMatch:=", sub[(cs)] {
      shorthand_def(cs, "\\attribute", Number::new(0).into())
    });
    DefPrimitive!("\\initcatcodetable Number", sub[(_n)] {});
    DefPrimitive!("\\savecatcodetable Number", sub[(_n)] {});
    DefPrimitive!("\\catcodetable Number", sub[(_n)] {});
    DefPrimitive!("\\luadef SkipSpaces Token SkipSpaces SkipMatch:= Number", sub[(cs, _n)] {
      clear_prefixes();
      let _ = def_macro(cs, None, ExpansionBody::Tokens(Tokens!()), None);
    });
    // LuaTeX manual §8: the PDF backend primitives that accompany
    // `\directlua` — `\pdfvariable <name>` (a token-list/int variable:
    // multimedia.sty:30 `\edef\beamer@pdfpageattr{\pdfvariable pageattr}`),
    // `\pdfextension <name>…`, `\pdffeedback <name>`. The name keyword is
    // consumed; the value is empty (hitszbeamer, vocaltract).
    DefMacro!(T_CS!("\\pdfvariable"), None, {
      let _ = read_keyword(&[
        "pageattr",
        "pagesattr",
        "pageresources",
        "compresslevel",
        "objcompresslevel",
        "decimaldigits",
        "gentounicode",
        "horigin",
        "vorigin",
        "linkmargin",
        "destmargin",
        "threadmargin",
        "xformmargin",
        "pkresolution",
        "pkfixeddpi",
        "inclusionerrorlevel",
        "ignoreunknownimages",
        "gamma",
        "imageapplygamma",
        "imagegamma",
        "imagehicolor",
        "imageaddfilename",
        "uniqueresname",
        "majorversion",
        "minorversion",
        "pagebox",
        "inclusioncopyfonts",
        "recorderfilename",
        "suppressoptionalinfo",
        "omitcidset",
        "omitcharset",
        "omitinfodict",
        "omitmediabox",
        "omitprocset",
        "fontattr",
        "info",
        "catalog",
        "names",
        "trailer",
        "trailerid",
        "xformattr",
        "xformresources",
        "retval",
        "lastlink",
        "lastannot",
        "lastobj",
        "lastxform",
        "lastximage",
        "lastximagepages",
      ])?;
      Vec::new()
    });
    DefMacro!(T_CS!("\\pdffeedback"), None, {
      let _ = read_keyword(&[
        "lastlink",
        "lastannot",
        "lastobj",
        "lastxform",
        "lastximage",
        "lastximagepages",
        "retval",
        "colorstackinit",
        "creationdate",
        "fontname",
        "fontobjnum",
        "fontsize",
        "pageref",
        "xformname",
        "ximagebbox",
        "version",
        "revision",
      ])?;
      vec![T_OTHER!("0")]
    });
    DefPrimitive!("\\pdfextension Token", sub[(_name)] {});
    // tuenc.def:106-121 `\DeclareUnicodeAccent{\cs}[{enc}]{code}` (tipauni.sty
    // :349+ `\DeclareUnicodeAccent{\textsyllabic}{TU}{"0329}`): the accent
    // appends `\char code` to its argument (NBSP base when empty). The
    // format here stays 8-bit (no `\UnicodeEncodingName`), so the command is
    // declared as the encoding DEFAULT rather than for `TU`.
    RawTeX!(
      r#"\def\add@unicode@accent#1#2{\if\relax\detokenize{#2}\relax^^a0\else#2\fi\char#1\relax}
\def\DeclareUnicodeAccent#1#2{\edef\reserved@a{#2}\def\reserved@b{TU}%
  \ifx\reserved@a\reserved@b\expandafter\lx@DeclareUnicodeAccent@iii\else\expandafter\lx@DeclareUnicodeAccent@ii\fi{#1}{#2}}
\def\lx@DeclareUnicodeAccent@iii#1#2#3{\DeclareTextCommandDefault{#1}{\add@unicode@accent{#3}}}
\def\lx@DeclareUnicodeAccent@ii#1#2{\DeclareTextCommandDefault{#1}{\add@unicode@accent{#2}}}"#
    );
    DefRegister!("\\prehyphenchar"     => Number::new(0));
    DefRegister!("\\posthyphenchar"    => Number::new(0));
    DefRegister!("\\preexhyphenchar"   => Number::new(0));
    DefRegister!("\\postexhyphenchar"  => Number::new(0));
    DefRegister!("\\hyphenationbounds" => Number::new(0));
    // latex.ltx:896-1058 verbatim, minus the UnicodeData.txt letter-catcode
    // loop (:946-996, a catcode-table fill) and the two `\directlua` calls
    // (:1038 `lua.name[…]`, :1056 `require("ltluatex")`). The four kernel
    // tables are `\chardef`'d directly: this option runs at preload time,
    // before the LaTeX pool defines `\e@alloc`.
    RawTeX!(
      r#"\ifx\e@alloc@attribute@count\@undefined
  \countdef\e@alloc@attribute@count=258
  \e@alloc@attribute@count=\z@
\fi
\def\newattribute#1{%
  \e@alloc\attribute\attributedef
    \e@alloc@attribute@count\m@ne\e@alloc@top#1%
}
\def\setattribute#1#2{#1=\numexpr#2\relax}
\def\unsetattribute#1{#1=-"7FFFFFFF\relax}
\ifx\e@alloc@ccodetable@count\@undefined
  \countdef\e@alloc@ccodetable@count=259
  \e@alloc@ccodetable@count=\z@
\fi
\def\newcatcodetable#1{%
  \e@alloc\catcodetable\chardef
    \e@alloc@ccodetable@count\m@ne{"8000}#1%
  \initcatcodetable\allocationnumber
}
\chardef\catcodetable@initex=1
\chardef\catcodetable@string=2
\chardef\catcodetable@latex=3
\chardef\catcodetable@atletter=4
\e@alloc@ccodetable@count=4
\ifx\e@alloc@luafunction@count\@undefined
  \countdef\e@alloc@luafunction@count=260
  \e@alloc@luafunction@count=\z@
\fi
\def\newluafunction{%
  \e@alloc\luafunction\e@alloc@chardef
    \e@alloc@luafunction@count\m@ne\e@alloc@top
}
\def\newluacmd{%
  \e@alloc\luafunction\luadef
    \e@alloc@luafunction@count\m@ne\e@alloc@top
}
\def\newprotectedluacmd{%
  \e@alloc\luafunction{\protected\luadef}
    \e@alloc@luafunction@count\m@ne\e@alloc@top
}
\ifx\e@alloc@whatsit@count\@undefined
  \countdef\e@alloc@whatsit@count=261
  \e@alloc@whatsit@count=\z@
\fi
\def\newwhatsit#1{%
  \e@alloc\whatsit\e@alloc@chardef
    \e@alloc@whatsit@count\m@ne\e@alloc@top#1%
}
\ifx\e@alloc@bytecode@count\@undefined
  \countdef\e@alloc@bytecode@count=262
  \e@alloc@bytecode@count=\z@
\fi
\def\newluabytecode#1{%
  \e@alloc\luabytecode\e@alloc@chardef
    \e@alloc@bytecode@count\m@ne\e@alloc@top#1%
}
\ifx\e@alloc@luachunk@count\@undefined
  \countdef\e@alloc@luachunk@count=263
  \e@alloc@luachunk@count=\z@
\fi
\def\newluachunkname#1{%
  \e@alloc\luachunk\e@alloc@chardef
    \e@alloc@luachunk@count\m@ne\e@alloc@top#1%
}
\def\now@and@everyjob#1{%
  \everyjob\expandafter{\the\everyjob
    #1%
  }%
  #1%
}
\attributedef\attributezero=0 "#
    );
    // The LuaTeX parameter surface documents commonly poke once they take
    // the LuaTeX branch (LuaTeX manual §2.6/§3): line-breaking and error-
    // suppression knobs — engine tuning with no XML-content meaning.
    DefRegister!("\\hyphenationmin"       => Number::new(-1));
    DefRegister!("\\outputmode"           => Number::new(1));
    DefRegister!("\\suppresslongerror"    => Number::new(0));
    DefRegister!("\\suppressoutererror"   => Number::new(0));
    DefRegister!("\\suppressmathparerror" => Number::new(0));
    DefRegister!("\\luacopyinputnodes"    => Number::new(0));
    DefRegister!("\\matheqnogapstep"      => Number::new(1000));
    DefRegister!("\\breakafterdirmode"    => Number::new(0));
    // `\matheqdirmode` (LuaTeX manual §6, math direction of equations):
    // minim-math.tex:19 sets it (lettrine-demo-arabic). Guard:
    // `perfect_kernel_batch54::luatex_profile_defines_matheqdirmode`.
    DefRegister!("\\matheqdirmode"        => Number::new(0));
    // `\pagewidth`/`\pageheight` (LuaTeX manual §4.3, the page dimensions
    // pdfTeX calls `\pdfpagewidth`): incgraph.sty reads them under LuaTeX.
    DefRegister!("\\pagewidth"            => Dimension!("614.295pt"));
    DefRegister!("\\pageheight"           => Dimension!("794.96999pt"));
    // luapstricks: under this profile pstricks.tex (L443-471) takes its
    // LuaTeX branch, routing `\pstverb`/`\pstVerb`/`\pstheader`/
    // `\c@lor@to@ps` through tokens that luapstricks.lua creates via
    // `token.set_lua` (L4212/4255/4259/4300) — a node-layer module our
    // bridge cannot run. Absorb the 4-token surface (optional keywords up
    // to the brace group; `\luaPSTbox` voids the box it scans). Raw
    // pstricks.sty loads dominate, so these live HERE, not in the
    // pstricks_tex binding. Sweep-11: 13 oracle-clean docs, 8 with
    // `\luaPST` as sole error — witnesses bardiag/bardiag1, psgo/psgomanual.
    DefMacro!("\\luaPST UntilBrace {}", "");
    DefMacro!("\\luaPSTheader UntilBrace {}", "");
    DefMacro!("\\luaPSTcolor UntilBrace {}", "");
    RawTeX!(r"\def\luaPSTbox{\setbox\voidb@x}");
    // XeTeX/LuaTeX: `\suppressfontnotfounderror=1` silences font-not-found.
    // Written by luatexja-preset.sty:501, iffont.sty:45-73, and
    // fontspec-luatex.sty:486. Engine knob, no XML meaning — but it must be
    // a REGISTER: a macro stub would leave the trailing `=1` to typeset.
    DefRegister!("\\suppressfontnotfounderror" => Number::new(0));
    // Direction primitives (LuaTeX §6): `\textdir TLT` etc. are PRIMITIVES
    // consuming a three-letter direction keyword — not macros producing one
    // (defining them as `{TLT}` typeset the keyword: `TLTTLT` text leaked
    // into the babel guard-test paragraph). TLT (left-to-right) is the only
    // direction this engine models → absorb the keyword, emit nothing.
    RawTeX!(
      r"\def\pagedir#1#2#3{} \def\bodydir#1#2#3{} \def\pardir#1#2#3{} \def\textdir#1#2#3{} \def\mathdir#1#2#3{} \def\linedir#1#2#3{}"
    );
    // Format parity: the lualatex FORMAT ships hyphenation patterns, so
    // `\bbl@luapatterns` is already defined when babel.def loads. That makes
    // babel.def L1135 skip the patterns-only first `\input luababel.def`
    // (which `\endinput`s at L195, and whose loaded-flag would suppress the
    // real second input at babel.def L2285) — so the single in-document
    // load runs luababel's Lua API part (L196+, `Babel.locale_props` &co).
    // Patterns are engine hyphenation state with no XML meaning → no-op
    // body. Guard: cluster_package_guards::luatex_babel_api.
    DefMacro!("\\bbl@luapatterns{}{}", "");
  });

  // Perl latexml.sty.ltxml L34-41: tracing / profiling options manipulate
  // a TRACING bitmap via TRACE_ALL / TRACE_PROFILE constants. Rust hasn't
  // wired the bitmap constants (no TRACE_ALL/TRACE_PROFILE symbols in the
  // state module), so stub these as no-op option declarations. The
  // observable effect is that `\usepackage[tracing]{latexml}` etc. simply
  // load latexml.sty without throwing an "unknown option" error; tracing
  // actually kicks in via the CLI `--verbose`/`--profile` flags, not
  // package options. Prevents load-time errors for documents that include
  // these flags defensively.
  DeclareOption!("tracing", None);
  DeclareOption!("notracing", None);
  DeclareOption!("profiling", None);
  DeclareOption!("noprofiling", None);

  // Perl latexml.sty.ltxml L43-44: breakuntex / nobreakuntex toggle the
  // SUPPRESS_UNTEX_LINEBREAKS boolean, which controls whether the `\\`
  // backslash-newline reversion in `tex=` attributes inserts a real
  // line break or is suppressed. Default breakuntex=true (Perl omits the
  // flag by default; documents explicitly passing nobreakuntex enable
  // SUPPRESS).
  DeclareOption!("breakuntex", {
    AssignValue!("SUPPRESS_UNTEX_LINEBREAKS" => false, Scope::Global);
  });
  DeclareOption!("nobreakuntex", {
    AssignValue!("SUPPRESS_UNTEX_LINEBREAKS" => true, Scope::Global);
  });

  ProcessOptions!(keysets => ["LTXML"]);

  // Process bibconfig keyval from options passed to latexml.sty.
  // Perl handles this via \setkeys{LTXML}{...} in the default option handler.
  // ProcessOptions with the LTXML keyset now stores package keyvals here;
  // keep the legacy extraction as a fallback for older call paths.
  if let Some(opts) = lookup_vecdeque("opt@latexml.sty") {
    for opt in opts.iter() {
      let opt_str = opt.to_string();
      if let Some(val) = opt_str.strip_prefix("bibconfig=") {
        assign_value(
          "KV@LTXML@bibconfig",
          Stored::String(pin(val.trim())),
          Some(Scope::Global),
        );
      }
    }
  }

  // Apply bibconfig from keyvals (Perl L57-59: code closure)
  // bibconfig=bbl,bib means try bbl first, fall back to bib
  if let Some(v) = lookup_value("KV@LTXML@bibconfig") {
    let config_str = v.to_string();
    let configs: Vec<_> = config_str.split(',').map(|s| pin(s.trim())).collect();
    if !configs.is_empty() {
      assign_value(
        "BIB_CONFIG",
        Stored::Strings(Rc::from(configs)),
        Some(Scope::Global),
      );
    }
  }

  // Apply limit options from keyvals (Perl L87-98)
  if let Some(v) = lookup_value("KV@LTXML@tokenlimit") {
    let limit = v.to_string().trim().parse::<usize>().unwrap_or(0);
    if limit > 0 {
      set_token_limit(Some(limit));
    }
  }
  if let Some(v) = lookup_value("KV@LTXML@iflimit") {
    let limit = v.to_string().trim().parse::<usize>().unwrap_or(0);
    if limit > 0 {
      assign_value("if_limit", Stored::from(limit as i64), Some(Scope::Global));
    }
  }
  if let Some(v) = lookup_value("KV@LTXML@absorblimit") {
    let limit = v.to_string().trim().parse::<usize>().unwrap_or(0);
    if limit > 0 {
      assign_value(
        "absorb_limit",
        Stored::from(limit as i64),
        Some(Scope::Global),
      );
    }
  }
  if let Some(v) = lookup_value("KV@LTXML@pushbacklimit") {
    let limit = v.to_string().trim().parse::<usize>().unwrap_or(0);
    if limit > 0 {
      set_pushback_limit(Some(limit));
    }
  }

  // Perl latexml.sty.ltxml L86-96: \lx@save@parameter{key}{value} records a
  // conversion parameter as a <?latexml key="value"?> processing instruction
  // (issue #536, reporter xworld21). It is a public-ish `@`-internal hook — a
  // document or package may call it directly — and the image-scaling options
  // below schedule it. The Rust binding previously never defined it, so a direct
  // call errored `undefined:\lx@save@parameter` and the options' PIs were
  // silently dropped (the earlier code assigned a `PI@latexml@…` state value
  // that nothing consumed). Mirror `\graphicspath`'s insertPI constructor.
  DefConstructor!("\\lx@save@parameter{}{}", sub[document, args, _props] {
    let key = args.first().and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
    if !key.is_empty() {
      let value = args.get(1).and_then(|a| a.as_ref()).map(|a| a.to_string()).unwrap_or_default();
      let mut attrs = HashMap::default();
      attrs.insert(key, value);
      document.insert_pi("latexml", Some(attrs))?;
    }
  });
  // Image-scaling options → a PI each, deferred to \begin{document} exactly as
  // Perl (AtBeginDocument(\lx@save@parameter{PI_NAME}{value})). The keyval name
  // is lowercase `dpi` but the PI name is uppercase `DPI` (Perl
  // `$STATE->assignValue(DPI => …)`); the other three keep their name.
  for (kv_name, pi_name) in &[
    ("dpi", "DPI"),
    ("magnify", "magnify"),
    ("upsample", "upsample"),
    ("zoomout", "zoomout"),
  ] {
    if let Some(v) = lookup_value(&s!("KV@LTXML@{}", kv_name)) {
      let val = v.to_string().trim().to_string();
      if !val.is_empty() {
        at_begin_document(Invocation!(T_CS!("\\lx@save@parameter"), vec![
          Some(mouth::tokenize_internal(TeXString::assembled(
            pi_name.to_string()
          ))),
          Some(mouth::tokenize_internal(TeXString::assembled(val))),
        ]))?;
      }
    }
  }

  DefConditional!("\\iflatexml", { true });

  // Perl: NewCounter('@XMDECL', 'section', idprefix => 'XMD');
  // Counter for \lxDeclare IDs, resets per-section (subordinate to section).
  NewCounter!("@XMDECL", "section", idprefix => "XMD");

  // ======================================================================
  // Define the Declare keyval family for \lxDeclare
  DefKeyVal!("Declare", "role", "");
  DefKeyVal!("Declare", "name", "");
  DefKeyVal!("Declare", "meaning", "");
  DefKeyVal!("Declare", "tag", "");
  DefKeyVal!("Declare", "scope", "");
  DefKeyVal!("Declare", "description", "");
  DefKeyVal!("Declare", "nowrap", "");
  DefKeyVal!("Declare", "label", "");
  DefKeyVal!("Declare", "trace", "");
  // Perl: DefKeyVal('Declare', 'replace', 'UndigestedKey') — the replacement
  // pattern is kept as raw tokens and digested at rewrite time (see the
  // replace-closure in \lxDeclare's afterConstruct).
  DefKeyVal!("Declare", "replace", "UndigestedKey");

  // \lxFcn / \lxID / \lxPunct — math-mode role hints (Perl latexml.sty.ltxml).
  // Wrap the argument in <ltx:XMWrap role='...'> so the math grammar
  // treats it as the named role for that occurrence only. `requireMath`
  // forces math context (errors if invoked outside math); `reversion =>
  // '#1'` round-trips just the body (no role wrapper) to TeX; `alias =>
  // ''` suppresses the constructor name in the reversion path.
  // Perl latexml.sty.ltxml L160-163: \lxRegisterNamespace{prefix}{uri}
  // — dynamic XML namespace registration for foreign attributes. Perl's
  // DefPrimitive calls RegisterNamespace(prefix => uri). Rust has
  // latexml_core::common::model::register_namespace exposed; wire it up
  // to the CS so documents using \lxRegisterNamespace{my}{http://…}
  // can then set foreign attributes like my:data='value'.
  DefPrimitive!("\\lxRegisterNamespace {} Semiverbatim", sub[(prefix, uri)] {
    let prefix_str = prefix.to_string();
    let uri_str = uri.to_string();
    model::register_namespace(&prefix_str, Some(&uri_str));
    Ok(Vec::new())
  });

  // Perl latexml.sty.ltxml L236-238: \lxRequireResource[options]{name}
  // adds a document resource (CSS/JS/…). Perl invocation:
  //   RequireResource(ToString(path), ($kv ? $kv->getHash : ()))
  // where the kv hash can carry `type` (mime-type) and `media`. Rust's
  // require_resource takes a `Resource{name, mimetype, media, content}`;
  // the infra lives in latexml_core::binding::content.
  DefPrimitive!("\\lxRequireResource OptionalKeyVals {}", sub[(kv, path)] {
    let name = path.to_string();
    let mimetype = kv.as_ref()
      .and_then(|k| k.get_value("type"))
      .map(|v| v.to_string())
      .unwrap_or_default();
    let media = kv.as_ref()
      .and_then(|k| k.get_value("media"))
      .map(|v| v.to_string())
      .unwrap_or_default();
    require_resource(
      Resource {
        name, mimetype, media, content: String::new(),
      });
    Ok(Vec::new())
  });

  // Perl latexml.sty.ltxml (PR #2767): \lxKeywords{text} — add keywords to
  // the frontmatter.
  DefMacro!("\\lxKeywords{}", "\\lx@add@keywords[name={keywords}]{#1}");

  // Perl latexml.sty.ltxml L249-250: \lxContextTOC — emits a TOC element
  // with format='context'. The matching ltx:TOC schema element already
  // flows through the native schema; previously missing in Rust.
  DefConstructor!("\\lxContextTOC", "<ltx:TOC format='context'/>");

  // Perl latexml.sty.ltxml L166-167: \lxAddClass{class} adds a CSS class
  // to the current element. Rust had this CS completely missing, so
  // documents using `\lxAddClass{ltx_highlight}` hit undefined-CS.
  DefConstructor!("\\lxAddClass Semiverbatim", "",
  after_construct => sub[document, whatsit] {
    let class_tok = whatsit.get_arg(1);
    if let Some(cls) = class_tok {
      let class_str = cls.to_string();
      if let Some(mut element) = document.get_element() {
        let _ = document.add_class(&mut element, &class_str);
      }
    }
  });

  // Perl latexml.sty.ltxml L182-185: \lxWithClass{class}{body} — wraps
  // body in a node with the given CSS class. Perl's getAnnotatableNode
  // detects text-node context and opens <ltx:text> if needed, then
  // addClass on the resulting container. Rust approximates: always
  // wrap in <ltx:text class='#1'>#2</ltx:text>. This is correct for
  // text-mode callers (the common case); in math mode the result
  // diverges (Perl wouldn't wrap, Rust adds an ltx:text inside XMath).
  // No test exercises \lxWithClass, so the approximation is
  // acceptable until the filter_children/absorb pipeline can be
  // wired.
  DefConstructor!(
    "\\lxWithClass Semiverbatim {}",
    "<ltx:text class='#1'>#2</ltx:text>"
  );

  DefConstructor!("\\lxFcn{}", "<ltx:XMWrap role='FUNCTION'>#1</ltx:XMWrap>",
    require_math => true, reversion => "#1", alias => "");
  DefConstructor!("\\lxID{}", "<ltx:XMWrap role='ID'>#1</ltx:XMWrap>",
    require_math => true, reversion => "#1", alias => "");
  DefConstructor!("\\lxPunct{}", "<ltx:XMWrap role='PUNCT'>#1</ltx:XMWrap>",
    require_math => true, reversion => "#1", alias => "");

  // Perl latexml.sty.ltxml L342-350: \lxMathTweak RequiredKeyVals {} —
  // the general form behind \lxFcn/\lxID/\lxPunct. Perl's comment says
  // "same as \lx@math@tweak"; the engine actually has \lx@math@tweaked
  // (base_xmath.rs L527) with the full RequiredKeyVals {} shape and
  // the xmath_copy_keyvals after_digest hook. Let-alias the user-
  // facing name at to the internal one so docs can write
  // `\lxMathTweak{role=POSTFIX}{@}` and get the expected XMWrap.
  Let!("\\lxMathTweak", "\\lx@math@tweaked");

  // \lxDeclare — declare semantic roles for math tokens
  // Perl: latexml.sty.ltxml lines 462-568
  // Creates <declare> elements and rewrite rules for math token annotation.
  // Complex patterns with \WildCard are NOT yet supported.
  DefConstructor!("\\lxDeclare OptionalMatch:* OptionalKeyVals:Declare {}", "",
  mode => "restricted_horizontal",
  reversion => "",
  before_digest => { neutralize_font(); },
  after_digest => sub[whatsit] {
    // Extract role/name/meaning from KeyVals arg (arg index 2 = keyvals)
    let mut role = String::new();
    let mut name_val = String::new();
    let mut meaning = String::new();
    let mut has_tag = false;
    let mut has_description = false;
    let mut tag_text = String::new();
    let mut description_text = String::new();
    // Perl: replace => $kv->getValue('replace') — an UndigestedKey, i.e. raw
    // tokens kept for digestion at replacement time (Core/Rewrite.pm
    // compile_replacement). Capture them (undigested) as an owned local so the
    // keyvals borrow is released before the whatsit is mutated below.
    let mut replace_tks_opt: Option<Tokens> = None;
    let mut nowrap_flag = false;
    let mut tag_digested: Option<Digested> = None;
    let mut description_digested: Option<Digested> = None;
    if let Some(kv_arg) = whatsit.get_arg(2)
      && let DigestedData::KeyVals(kv) = kv_arg.data() {
        let hash = kv.get_hash_digested();
        replace_tks_opt = kv.get_value("replace").and_then(|a| a.revert().ok());
        // Perl: nowrap => defined $kv->getValue('nowrap') — presence flag that
        // routes setAttributes_wild to mark the non-wildcard base instead of
        // wrapping the matched span in an XMDual. (Read here, before the
        // set_property below ends the kv borrow.)
        nowrap_flag = kv.get_value("nowrap").is_some();
        // DIGESTED tag/description values for normalizeDeclareKeys below —
        // a description like `$x$: a real variable` contains a real math box
        // that must survive to the <ltx:declare> term tag (Perl inserts the
        // boxes; the term Math is then subject to the declaration rewrites).
        tag_digested = kv.get_value_digested("tag").cloned();
        description_digested = kv.get_value_digested("description").cloned();
        if let Some(v) = hash.get("role") { role = v.clone(); }
        if let Some(v) = hash.get("name") { name_val = v.clone(); }
        if let Some(v) = hash.get("meaning") { meaning = v.clone(); }
        if let Some(v) = hash.get("tag") { has_tag = true; tag_text = v.clone(); }
        if let Some(v) = hash.get("description") { has_description = true; description_text = v.clone(); }
        // Store scope option for rewrite rule creation in afterConstruct
        if let Some(v) = hash.get("scope") {
          whatsit.set_property("scope_opt", Stored::from(v.clone()));
        }
      }
    if let Some(replace_tks) = replace_tks_opt {
      whatsit.set_property("replace_tokens", Stored::Tokens(replace_tks));
    }
    if nowrap_flag {
      whatsit.set_property("nowrap", Stored::from("1".to_string()));
    }
    // Perl normalizeDeclareKeys: synthesize term/short/description for the
    // <ltx:declare> element (declare.rs; splitDeclareTag splits at ':').
    normalize_declare_keys(whatsit, tag_digested.as_ref(), description_digested.as_ref());
    // Extract body text from arg 3 (the {} body)
    let body_text = whatsit.get_arg(3)
      .map(|a| { let s = a.to_string(); s.trim_matches('$').trim().to_string() })
      .unwrap_or_default();
    // Capture the digested pattern's font (Perl's domToXPath includes @font in
    // the match, so e.g. an italic `$x$` declaration does NOT match a bold
    // `\mathbf{x}` — fonts carry mathematical meaning). Only \lxDeclare has a
    // digested body to read this from; the .latexml DefMathRewrite loader path
    // (string matches) keeps its font-agnostic behavior via match_font=None.
    let match_font = whatsit
      .get_arg(3)
      .and_then(|a| a.get_font().ok().flatten())
      .map(|f| f.font_attribute_string())
      .filter(|s| !s.is_empty());
    if let Some(ref font_str) = match_font {
      whatsit.set_property("match_font", Stored::from(font_str.clone()));
    }

    // Generate declaration ID if tag or description present
    let decl_id = if has_tag || has_description {
      next_declaration_id()?
    } else {
      String::new()
    };

    // Store properties on the whatsit for constructor body and afterConstruct
    whatsit.set_property("role", Stored::from(role.clone()));
    whatsit.set_property("name", Stored::from(name_val.clone()));
    whatsit.set_property("meaning", Stored::from(meaning.clone()));
    whatsit.set_property("body_text", Stored::from(body_text.clone()));
    whatsit.set_property("decl_id", Stored::from(decl_id.clone()));
    if has_description || has_tag {
      let desc = if !description_text.is_empty() { description_text } else { tag_text };
      whatsit.set_property("description", Stored::from(desc));
    }

    // Register in the LATEXML_DECLARATIONS fast-path table (declare.rs).
    if !body_text.is_empty() && (!role.is_empty() || !name_val.is_empty() || !meaning.is_empty()) {
      let scope_opt_val = whatsit
        .get_property("scope_opt")
        .map(|v| v.to_string())
        .unwrap_or_default();
      record_declaration_lines(
        &body_text, &role, &name_val, &meaning, &decl_id,
        match_font.as_deref(), &scope_opt_val)?;
    }
  },
  after_construct => sub[document, whatsit] {
    // Perl: createDeclarationRewrite — create rewrite rule AND <declare> element
    let role = whatsit.get_property("role").map(|v| v.to_string()).unwrap_or_default();
    let name_val = whatsit.get_property("name").map(|v| v.to_string()).unwrap_or_default();
    let meaning = whatsit.get_property("meaning").map(|v| v.to_string()).unwrap_or_default();
    let body_text = whatsit.get_property("body_text").map(|v| v.to_string()).unwrap_or_default();
    let decl_id = whatsit.get_property("decl_id").map(|v| v.to_string()).unwrap_or_default();
    // Perl createDeclarationRewrite: a `replace=` declaration provides a
    // replacement for the matched expression instead of adding attributes
    // (the two are mutually exclusive). Recover the raw replacement tokens.
    let replace_tokens: Option<Tokens> = whatsit
      .get_property("replace_tokens")
      .and_then(|v| if let Stored::Tokens(t) = v.as_ref() { Some(t.clone()) } else { None });

    // Emit the <ltx:declare> element (declare.rs, Perl L474-485) from the
    // digested *_boxes properties normalizeDeclareKeys stored.
    if !decl_id.is_empty() {
      let unpack = |key: &str| match whatsit.get_property(key).as_deref() {
        Some(Stored::VecDigested(v)) => Some(v.clone()),
        _ => None,
      };
      emit_declare_element(
        document,
        &decl_id,
        unpack("term_boxes"),
        unpack("short_boxes"),
        unpack("desc_boxes"),
      )?;
    }

    // Perl createDeclarationRewrite (declare.rs): build + UNSHIFT the rule.
    // A tag-only declaration (decl_id, no role/name/meaning) still creates one.
    let has_annotation =
      !role.is_empty() || !name_val.is_empty() || !meaning.is_empty() || !decl_id.is_empty();
    if !body_text.is_empty() && (has_annotation || replace_tokens.is_some()) {
      let scope_val = whatsit.get_property("scope_opt").map(|v| v.to_string()).unwrap_or_default();
      let rewrite_scope = get_declaration_scope(document, &scope_val, &decl_id);
      create_declaration_rewrite(
        rewrite_scope,
        role,
        name_val,
        meaning,
        decl_id,
        &body_text,
        whatsit.get_property("nowrap").is_some(),
        replace_tokens,
      );
    }
  });

  // Perl latexml.sty.ltxml L300-307: user-facing aliases for
  // \lx@alignment@begin@heading / \lx@alignment@end@heading, which
  // bracket a run of tabular heading rows. The table-foot aliases
  // point at the same two CSes (the Perl convention uses head/foot
  // for clarity; both just toggle the in_tabular_head flag).
  Let!("\\lxBeginTableHead", "\\lx@alignment@begin@heading");
  Let!("\\lxEndTableHead", "\\lx@alignment@end@heading");
  Let!("\\lxBeginTableFoot", "\\lx@alignment@begin@heading");
  Let!("\\lxEndTableFoot", "\\lx@alignment@end@heading");

  // Perl latexml.sty.ltxml L310-313: \lxTableColumnHead — mirrors
  // \lxTableRowHead below but flips thead_in_column instead of
  // thead_in_row on the current column spec.
  def_primitive(
    T_CS!("\\lxTableColumnHead"),
    None,
    Some(PrimitiveBody::Closure(Rc::new(|_args| {
      if let Some(alignment) = lookup_alignment()
        && let Some(data) = alignment.alignment_cell()
        && let Some(col) = data.borrow_mut().current_column()
      {
        col.thead_in_column = true;
      }
      Ok(Vec::new())
    }))),
    PrimitiveOptions::default(),
  )?;

  // Perl: DefMacroI('\lxTableRowHead', undef, sub { $alignment->currentColumn->{thead}{row} = 1 })
  // Marks the current column as a row header in alignment/tabular contexts.
  // Usage: >{\lxTableRowHead} in column spec with array.sty
  def_primitive(
    T_CS!("\\lxTableRowHead"),
    None,
    Some(PrimitiveBody::Closure(Rc::new(|_args| {
      if let Some(alignment) = lookup_alignment()
        && let Some(data) = alignment.alignment_cell()
        && let Some(col) = data.borrow_mut().current_column()
      {
        col.thead_in_row = true;
      }
      Ok(Vec::new())
    }))),
    PrimitiveOptions::default(),
  )?;

  // Perl latexml.sty L354-371: \lxDefMath{\name}[nargs][optional]{presentation}[keyvals]
  // Defines a math macro with semantic annotations (name, meaning, role, etc.)
  // Perl latexml.sty.ltxml L385-405: \@lxDefMathDeclare{id}{description} —
  // the declare-element half of a tagged \lxDefMath. Perl passes the raw
  // keyvals and derives term/short/description via normalizeDeclareKeys; the
  // Rust shim pre-resolves the description tokens (tag || description) and
  // digests them as the ltx:text content (so embedded math renders — and its
  // tokens are subject to the declaration rewrites, like any document math).
  DefConstructor!("\\@lxDefMathDeclare {} {}", "",
  mode => "restricted_horizontal",
  reversion => "",
  after_construct => sub[document, whatsit] {
    let id = whatsit.get_arg(1).map(|a| a.to_string()).unwrap_or_default();
    if !id.is_empty() {
      let desc = whatsit.get_arg(2).map(|d| vec![d.clone()]);
      emit_declare_element(document, &id, None, None, desc)?;
    }
  });

  DefPrimitive!("\\lxDefMath {} [Number] [] {} OptionalKeyVals:XMath", sub[(cs, nargs, opt, presentation, params_opt)] {
    let cs_name = cs.to_string();
    let n = nargs.value_of() as usize;
    // Extract semantic properties from keyvals.
    // Perl L368 always sets `revert_as => 'context'` so source-export
    // emits the user-defined CS rather than expanding the presentation
    // template (matches the convention for user-defined math macros).
    let mut opts = MathPrimitiveOptions {
      revert_as: Some(Cow::Borrowed("context")),
      ..Default::default()
    };
    // Perl L374-380: tag/description ⇒ allocate a decl_id (next_declaration_id),
    // pass it to DefMathI (every use-site token/dual-op then carries decl_id at
    // digestion), and Digest a follow-up \@lxDefMathDeclare{id}{desc} whose
    // whatsit emits the <ltx:declare> element.
    let mut desc_tks_opt: Option<Tokens> = None;
    let mut needs_id = false;
    if let Some(kv) = params_opt.as_ref() {
      if let Some(v) = kv.get_value("name") { opts.name = Some(v.to_string()); }
      if let Some(v) = kv.get_value("meaning") { opts.meaning = Some(v.to_string()); }
      if let Some(v) = kv.get_value("role") { opts.role = Some(v.to_string()); }
      if let Some(v) = kv.get_value("cd") { opts.omcd = Some(v.to_string()); }
      if let Some(v) = kv.get_value("alias") { opts.alias = Some(v.to_string()); }
      let tag_tks: Option<Tokens> = kv.get_value("tag").and_then(|a| a.revert().ok());
      let desc_tks: Option<Tokens> = kv.get_value("description").and_then(|a| a.revert().ok());
      needs_id = tag_tks.is_some() || desc_tks.is_some();
      desc_tks_opt = desc_tks.or(tag_tks);
    }
    let mut declare_box: Option<Digested> = None;
    if needs_id {
      let id = next_declaration_id()?;
      if !id.is_empty() {
        opts.decl_id = Some(id.clone());
        let mut inv: Vec<Token> = vec![T_CS!("\\@lxDefMathDeclare"), T_BEGIN!()];
        inv.extend(ExplodeText!(&id));
        inv.push(T_END!());
        inv.push(T_BEGIN!());
        if let Some(ref d) = desc_tks_opt {
          inv.extend(d.unlist_ref().iter().cloned());
        }
        inv.push(T_END!());
        declare_box = Some(digest(Tokens::new(inv))?);
      }
    }
    // Build parameter spec for n args
    use latexml_core::common::def_parser::parse_parameters;
    let params = if n > 0 {
      let spec = (0..n).map(|_| "{}").collect::<Vec<_>>().join("");
      parse_parameters(&spec, &T_CS!(&cs_name), true)?
    } else {
      None
    };
    // Create the math definition
    let presentation_str = presentation.to_string();
    def_math(
      T_CS!(&cs_name),
      params,
      presentation_str,
      opts,
    )?;
    // Perl: return Digest(Invocation('\@lxDefMathDeclare', $id, $params))
    declare_box.map(|b| vec![b]).unwrap_or_default()
  });

  // Perl latexml.sty L106-108: \URL[text]{href}
  DefConstructor!("\\URL[] Verbatim",
    "<ltx:ref href='#href'>?#1(#1)(#href)</ltx:ref>",
    enter_horizontal => true,
    properties => sub[_args] {
      let mut href_str = _args
        .get(1)
        .and_then(|a| a.as_ref())
        .map(|a| a.to_string())
        .unwrap_or_default();
      // Perl: CleanURL — strip whitespace/newlines from URLs
      href_str = href_str.replace(['\n', '\r'], "").trim().to_string();
      Ok(stored_map!("href" => href_str))
    }
  );

  // Perl latexml.sty.ltxml L122-134: the LaTeXML-logo trio.
  // \LaTeXML expands to \LaTeXML@logo, which lays out a stylized
  // nested-ltx:text pattern (the classic Lamport-style kerning). The
  // Perl `sizer` closure is specific to LaTeXML-Post typesetting layout
  // and not called by the Rust compile-time binding pipeline — omit.
  DefMacro!("\\LaTeXML", "\\LaTeXML@logo");
  DefConstructor!("\\LaTeXML@logo",
    "<ltx:text class='ltx_LaTeXML_logo'>\
       <ltx:text cssstyle='letter-spacing:-0.2em; margin-right:0.1em'>\
         L\
         <ltx:text cssstyle='font-variant:small-caps;' yoffset='0.4ex'>a</ltx:text>\
         T\
         <ltx:text cssstyle='font-variant:small-caps;font-size:120%' yoffset='-0.2ex'>e</ltx:text>\
       </ltx:text>\
       <ltx:text cssstyle='font-size:90%' yoffset='-0.2ex'>XML</ltx:text>\
     </ltx:text>",
    enter_horizontal => true);

  // Perl latexml.sty.ltxml L136-139: \LaTeXMLversion / \LaTeXMLrevision expand
  // to $LaTeXML::VERSION / $LaTeXML::Version::REVISION via ExplodeText. The Rust
  // analog of $LaTeXML::VERSION is the LATEXML_VERSION state value, seeded at
  // session init from the latexml_oxide *binary's* own three-part crate version
  // (core_interface.rs) — the same source the Rhai `LaTeXMLVersion()` binding and
  // the XSLT `LATEXML_VERSION` parameter read. latexml.sty is loaded at runtime (a
  // user `\usepackage{latexml}`, never dumped) and session init sets the value
  // before any package loads, so reading it here yields the running binary's
  // version. (Issue #541: this used to hard-code a stale "0.4.0" — latexml_package
  // cannot env!() the binary's version, and latexml_contrib's 0.4.0 is not it.)
  // Revision is left empty (no git rev exposed at runtime); that makes
  // \LaTeXMLfullversion collapse to just the version string via the
  // `\ifx\expandafter.\LaTeXMLrevision.` guard.
  def_macro(
    T_CS!("\\LaTeXMLversion"),
    None,
    mouth::tokenize_internal(TeXString::assembled(lookup_string("LATEXML_VERSION"))),
    None,
  )?;
  def_macro_noop("\\LaTeXMLrevision")?;
  DefMacro!(
    "\\LaTeXMLfullversion",
    "\\LaTeXML (\\LaTeXMLversion\\expandafter\\ifx\\expandafter.\\LaTeXMLrevision.\\else; rev.~\\LaTeXMLrevision\\fi)"
  );

  // Perl latexml.sty.ltxml L227-230: \lxRef{label}{text} — like hyperref's
  // \hyperref but straightforward. Emits <ltx:ref labelref='label'>text</ref>
  // with enter_horizontal so a bare \lxRef between paragraphs doesn't
  // leak out of <ltx:p> (same mode-leak class as hyperref \url cycle 87).
  // CleanLabel normalizes the label for the labelref attribute.
  DefConstructor!("\\lxRef Semiverbatim {}",
    "<ltx:ref labelref='#label'>#2</ltx:ref>",
    enter_horizontal => true,
    properties => sub[args] {
      unpack_opt_ref!(args => label_opt);
      let label = label_opt.as_ref().unwrap().to_string();
      Ok(stored_map!("label" => Stored::String(pin(clean_label(&label, None)))))
    }
  );

  // Perl latexml.sty.ltxml L209-222: \lxAddAnnotation / \lxWithAnnotation
  // add RDFa-ish annotations to the current / enclosing node via the
  // `addAnnotations` helper. That helper isn't ported to Rust yet (see
  // latexml_sty.rs:855 "Track separately" for \@lxDefMathDeclare, same
  // family). Ship arg-consuming stubs so documents invoking
  // \lxAddAnnotation{key=val,...} or \lxWithAnnotation{…}{body} don't
  // hit undefined-CS. The {body} arg passes through for \lxWithAnnotation
  // so the visible content is preserved; the annotation itself is dropped.
  def_macro_noop("\\lxAddAnnotation RequiredKeyVals")?;
  DefMacro!("\\lxWithAnnotation RequiredKeyVals {}", "#2");

  // Perl latexml.sty.ltxml L514-528: \lxRefDeclaration OptionalKeyVals:Declare {}
  // — refers declarations from another document point to labels at the
  // call site, via createDeclarationRewrite + the Declaration_ state
  // registry. Neither helper is ported. Stub as arg-consuming no-op so
  // documents don't hit undefined-CS; annotations won't actually rewrite
  // but the prose renders cleanly.
  def_macro_noop("\\lxRefDeclaration OptionalKeyVals:Declare {}")?;

  // Perl latexml.sty.ltxml L145: \lxDocumentID{id} sets the top-level
  // document's xml:id via a plain TeX `\def` of the internal
  // \thedocument@ID command that \begin{document}'s constructor
  // consults for its `id` property.
  DefMacro!("\\lxDocumentID{}", "\\def\\thedocument@ID{#1}");

  // Perl latexml.sty.ltxml L148: \LXMID{id}{math} associates an
  // identifier with the given math expression. Thin wrapper around
  // the internal \lx@xmarg constructor already emitted elsewhere.
  DefMacro!("\\LXMID{}{}", "\\lx@xmarg{#1}{#2}");

  // Perl latexml.sty.ltxml L153: \LXMRef{id} refers to the math
  // expression associated with id. Thin wrapper around \lx@xmref.
  DefMacro!("\\LXMRef{}", "\\lx@xmref{#1}");

  // Perl latexml.sty L109-116: acronym shortcuts. Prior Rust stopped at
  // \XML / \SGML / \HTML — the remaining \XHTML / \XSLT / \CSS / \MathML
  // / \OpenMath were missing, so documents using them hit undefined-CS
  // errors.
  DefMacro!("\\XML", "\\textsc{xml}");
  DefMacro!("\\SGML", "\\textsc{sgml}");
  DefMacro!("\\HTML", "\\textsc{html}");
  DefMacro!("\\XHTML", "\\textsc{xhtml}");
  DefMacro!("\\XSLT", "\\textsc{xslt}");
  DefMacro!("\\CSS", "\\textsc{css}");
  DefMacro!("\\MathML", "\\texttt{MathML}");
  DefMacro!("\\OpenMath", "\\texttt{OpenMath}");

  // Diagnostic constructor: emits a marker that gets filled with the Marpa parse tree count
  // for the preceding formula, after math parsing completes.
  // Usage: $x^2$ \ltx@count@parses → becomes the count of grammar trees.
  // The math parser sets _parsetrees on each Math element, then a post-parse step
  // in core_interface fills in the markers.
  DefConstructor!("\\ltx@count@parses",
    "<ltx:text class='ltx_count_parses' _parsetrees_marker='true'>0</ltx:text>",
    enter_horizontal => true);

  // Perl latexml.sty.ltxml L263-289: {lxNavbar} / {lxHeader} / {lxFooter}
  // envs accumulate body content into a `navigation` list that
  // insertNavigation (ltx:document afterClose) splices under an
  // <ltx:navigation> wrapper. Rust has no afterClose hook yet and no
  // PushValue-based list accumulator plumbed through the post-pipeline,
  // so a faithful hoisted-navigation output isn't possible yet. Stub
  // as inline-logical-block wrappers that keep body content visible
  // in-flow and prevent undefined-env errors when documents invoke
  // \begin{lxNavbar}.../\begin{lxHeader}.../\begin{lxFooter}... .
  // Intentional divergence: navigation content appears in flow rather
  // than hoisted to a dedicated <ltx:navigation> container. Revisit
  // when the Tag()/afterClose + PushValue list-accumulator machinery
  // is ported.
  // Perl all three envs run `beforeDigest => sub { AssignValue(inPreamble => 0); }`
  // so body content digests as document text even when the env is
  // invoked from the preamble (same pattern as jheppub affiliation
  // and standalone.sty's \@standalone@start@input).
  DefEnvironment!("{lxNavbar}",
    "<ltx:inline-logical-block class='ltx_page_navbar'>#body</ltx:inline-logical-block>",
    before_digest => { AssignValue!("inPreamble" => false); });
  DefEnvironment!("{lxHeader}",
    "<ltx:inline-logical-block class='ltx_page_header'>#body</ltx:inline-logical-block>",
    before_digest => { AssignValue!("inPreamble" => false); });
  DefEnvironment!("{lxFooter}",
    "<ltx:inline-logical-block class='ltx_page_footer'>#body</ltx:inline-logical-block>",
    before_digest => { AssignValue!("inPreamble" => false); });
});
