//! `latex_constructs` section 8: C.8 Definitions, Numbering and Programming
//!
//! Split verbatim from the single `LoadDefinitions!` body (2026-09-03); the
//! definitions run in the original order from `super::load_definitions`.

use super::*;

#[rustfmt::skip]
pub(crate) fn load() -> Result<()> {
  // ======================================================================
  // C.8 Definitions, Numbering and Programming
  // ======================================================================

  //**********************************************************************
  // C.8 Definitions, Numbering and Programming
  //**********************************************************************

  //======================================================================
  // C.8.1 Defining Commands
  //======================================================================

  // \@tabacckludge body lives in `latex_constructs_rust_only.rs` (Perl
  // has it in latex_base.pool.ltxml L357; the rust_only.rs copy is the
  // dump-path override per its docstring section 7).
  // latex.ltx L10007 — `\let\a=\@tabacckludge`. The dump carries
  // `\a` as an E record (the serializer captured
  // `\@tabacckludge`'s body with a `\@changed@cmd` wrapper, not a
  // pure Let-alias), which the outer M-gate rejects as a
  // public-CS Expandable (expl3-cascade safety). Neither the
  // PA/MPA gate relaxation nor the deferred-alias retry pass in
  // `dump_reader.rs` applies, so we keep this alias hand-written
  // to match latex.ltx source. Inside a `tabbing` environment,
  // tabbing_bindings() overrides this local to `\@tabbing@accent`.
  // Found in arxiv 1611.05395.
  //
  // **Lazy-pool-load guard (2026-05-01)**: in Perl, the kernel
  // `\let\a=\@tabacckludge` runs at engine init, BEFORE user TeX.
  // If the user defines `\def\a{\alpha}` in their preamble (a
  // common Greek-letter abbreviation), the user assignment runs
  // LATER in TeX's normal "later wins" semantics and overrides the
  // kernel Let cleanly. In Rust, `latex_constructs` runs at
  // `\documentclass`-time (lazy-pool-load —
  // `wisdom_lazy_pool_load.md`), AFTER the user preamble — so this
  // bare `Let!` would clobber the user's `\def\a{\alpha}` and
  // route subsequent `\a` invocations through the kernel
  // `\@changed@cmd → \+ → \tabalign` chain that triggers a
  // `\halign`/`\hbox` mode-mismatch runaway. Witness:
  // hep-th0005268, see `wisdom_tabalign_math_runaway.md`.
  //
  // Guard: only Let `\a` if it's not already user-defined. The
  // dump E record was rejected, so `\a` would be undefined here
  // unless the user defined it.
  if !has_meaning(&T_CS!("\\a")) {
    Let!("\\a", "\\@tabacckludge");
  }

  DefPrimitive!("\\newcommand OptionalMatch:* SkipSpaces DefToken [Number][]{}",
  sub[(_star,cs_token,nargs,opt,body)] {
    let nargs = nargs.value_of() as usize;
    let (definable, plain_origin) = is_definable_latex(&cs_token)?;
    if !definable {
      if !has_value(&s!("{}:locked", cs_token.to_string())) { // not locked, inform.
        let message = s!("Ignoring redefinition (\\newcommand) of {}", cs_token.stringify());
        Info!("ignore", cs_token, message);
      }
      return Ok(vec![]);
    }
    let macro_args = convert_latex_args(nargs, opt)?;
    // When the CS came from the plain pool, bypass any `<cs>:locked`
    // guard so latex.ltx can layer over plain. RAII guard ensures the
    // lock state is restored even if `DefMacro!` errors. No-op when
    // the CS was previously undefined.
    let _unlock = plain_origin.then(
      || local_state_unlocked_guard(true));
    DefMacro!(cs_token, macro_args, body);
  });

  DefPrimitive!("\\renewcommand OptionalMatch:* DefToken [Number][]{}",
  sub[(_star, cs, nargs_num, opt, body)] {
    let nargs = nargs_num.value_of() as usize;
    let macro_args = convert_latex_args(nargs, opt)?;
    DefMacro!(cs, macro_args, body);
  });

  // low-level implementation of both \newcommand and \renewcommand depends on \@argdef
  // and robustness upgrades are often realized via redefining \l@ngrel@x
  // Perl latex_constructs.pool.ltxml L2591-2604
  DefPrimitive!("\\@argdef DefToken [Number]{}", sub[(cs, nargs, body)] {
    let macro_args = convert_latex_args(nargs.value_of() as usize, None)?;
    DefMacro!(cs, macro_args, body);
  });
  DefPrimitive!("\\@xargdef DefToken [Number][]{}", sub[(cs, nargs, opt, body)] {
    let macro_args = convert_latex_args(nargs.value_of() as usize, opt)?;
    DefMacro!(cs, macro_args, body);
  });
  // Perl L2597-2602: \@yargdef checks if arg2 equals \tw@ (2) for optional arg type
  DefPrimitive!("\\@yargdef DefToken DefToken {}{}", sub[(cs, type_tok, nargs_toks, body)] {
    let nargs_str = nargs_toks.to_string();
    let nargs: usize = nargs_str.trim().parse().unwrap_or(0);
    let has_optional = type_tok.with_str(|s| s.contains('2'))
      || x_equals(&type_tok, &T_CS!("\\tw@"));
    let opt = if has_optional { Some(Tokens!()) } else { None };
    let macro_args = convert_latex_args(nargs, opt)?;
    DefMacro!(cs, macro_args, body);
  });
  DefPrimitive!("\\@reargdef DefToken [Number]{}", sub[(cs, nargs, body)] {
    let macro_args = convert_latex_args(nargs.value_of() as usize, None)?;
    DefMacro!(cs, macro_args, body);
  });

  DefPrimitive!("\\providecommand OptionalMatch:* DefToken [Number][]{}",
  sub[(_star, cs, nargs, opt, body)] {
    // Use `is_definable_latex` (honors the `:autoload` flag) rather than the bare
    // `IsDefinable!`, for the same reason as `\newcommand`/`\newenvironment`: an
    // autoload TRIGGER (e.g. `\align`→amsmath from `def_autoload`) appears defined
    // but is genuinely undefined in Perl (its `DefAutoload` lives in OmniBus, not
    // loaded for typical papers), so `\providecommand{\align}{…}` must DEFINE it
    // there. See [[project_newenvironment_autoload_clobber]].
    if is_definable_latex(&cs)?.0 {
      let nargs = nargs.value_of() as usize;
      let cs_args = convert_latex_args(nargs, opt)?;
      DefMacro!(cs, cs_args, body);
    }
  });

  // Crazy; define \cs in terms of \cs[space] !!!
  DefPrimitive!("\\DeclareRobustCommand OptionalMatch:* SkipSpaces DefToken [Number][]{}",
  sub[(_star,cs,nargs,opt,body)] {
    let nargs = nargs.value_of() as usize;
    let cs_args = convert_latex_args(nargs, opt)?;
    DefMacro!(cs, cs_args, body, robust => true);
  });

  DefPrimitive!("\\MakeRobust DefToken", sub[(cs)] {
    let mungedcs = T_CS!(cs.with_str(|cstr| s!("{cstr} ")));
    // only if defined but not yet robust
    if LookupDefinition!(&cs).is_some() &&
       LookupDefinition!(&mungedcs).is_none() {
      Let!(&mungedcs, &cs);
      DefMacro!(cs, None, Tokens!(T_CS!("\\protect"),mungedcs));
    }
  });

  // \CheckCommand validates but doesn't define — absorb and ignore args
  def_primitive_noop("\\CheckCommand OptionalMatch:* SkipSpaces DefToken [Number][]{}")?;
  // Font encoding subset declaration — ignored in our context
  def_primitive_noop("\\DeclareEncodingSubset{}{}{}")?;

  //------------------------------------------------------------
  // The following commands define encoding-specific expansions
  // or glyphs.  The control-sequence is defined to use the expansion for
  // the current encoding, if any, or the default expansion (for encoding "?").
  // We don't want to redefine control-sequence if it already has a definition:
  // It may be that we've already defined it to expand into the above conditional.
  // But more importantly, we don't want to override a hand-written definition (if any).
  // Perl latex_constructs.pool.ltxml:2588-2591/2602-2605: the bare command
  // is the CALL-TIME encoding dispatcher (`\<cf@encoding>\cs`, else `\?\cs`),
  // exactly as `\DeclareTextSymbol` below installs it. The port froze the bare
  // command to the FIRST encoding body instead, so lgrenc.def:278's
  // `\ProvideTextCommand{\textbetasymbol}{LGR}{\LGR@TextSymbolUnavailable…}`
  // stuck and textalpha's later `normalize-symbols` override of
  // `\LGR\textbetasymbol` never reached it: "character beta symbol not
  // available" ×~200 across the greek-fontenc manuals (char-list,
  // hyperref-with-greek, alphabeta-doc). Guard:
  // `perfect_kernel_batch54::provide_text_command_dispatches_on_encoding`.
  // `\fi`-free: the chosen command must see its ARGUMENT next. With Perl's
  // `…\else\csname…\endcsname\fi` shape an argument-taking text command
  // (`\accperispomeni{a}`) read the `\fi` as its argument and the real
  // `{a}` followed the mark ("͂α" — the mark BEFORE the letter).
  // Perl keeps an existing bare definition (a hand-written macro must win,
  // and the kernel's Unicode glyph primitives `\textdegree` … beat a fontmap
  // slot lookup — the whole 30_encoding family depends on that); real
  // `\DeclareTextCommand` always redefines. One carve-out: a CONTROL SYMBOL
  // (`\<`, `\>`) that is a non-expandable primitive gives way —
  // textalpha.sty:187-188 declares the breathings on `\<`/`\>`, and `\>` is
  // otherwise the math `\mskip` primitive, which in text turned
  // `\>'\textalpha` into a space + `'`.
  fn text_command_may_define(cs: &Token) -> Result<bool> {
    let is_control_symbol =
      cs.with_str(|s| s.len() == 2 && !s.ends_with(|c: char| c.is_alphabetic()));
    Ok(match lookup_definition(cs)? {
      None => true,
      Some(defn) => is_control_symbol && !defn.is_expandable(),
    })
  }
  fn def_text_command_dispatcher(cs: &Token, cs_str: &str) -> Result<()> {
    DefMacro!(*cs, None, Some(s!(
      r"\expandafter\ifx\csname\cf@encoding\string{cs_str}\endcsname\relax\expandafter\@firstoftwo\else\expandafter\@secondoftwo\fi{{\csname?\string{cs_str}\endcsname}}{{\csname\cf@encoding\string{cs_str}\endcsname}}"
    ).into()));
    Ok(())
  }
  //------------------------------------------------------------
  // `locked => true` on the `\Declare...`/`\Provide...` text primitives
  // below: a raw-loaded package may `\def\DeclareTextSymbol{...}` to
  // route through its own TeX-level dispatch (e.g. mathtext.sty's
  // `\DeclareTextMathSymbol` chain). When the package's chain depends
  // on LaTeX kernel internals we don't fully implement
  // (`\@changed@tmcmd`, `\@tmchar@`, `\csname\cf@encoding\string<cs>
  // \endcsname → \chardef` resolution), the override produces an
  // unbounded macro-expansion loop. Perl LaTeXML has no binding for
  // mathtext.sty and skips it entirely (`Warning:missing_file`); we
  // raw-load instead (default `INCLUDE_STYLES=true` + ar5iv
  // parity), so the override fires and breaks. Locking keeps our
  // primitive in place even under raw-load; the package's `\def` is
  // logged as Info:ignore and skipped. Witness: paper 2305.16331 +
  // `\u\i` under `mathtext + T2A`.
  DefPrimitive!("\\DeclareTextCommand DefToken {}[Number][]{}",
  sub[(cs, encoding, nargs, opts, expansion)] {
    let cs_str = cs.to_string();
    let nargs = nargs.value_of() as usize;
    let encoding_str = Expand!(encoding).to_string();
    let ecs = T_CS!(s!("\\{encoding_str}{cs_str}"));
    let ecs_args = convert_latex_args(nargs, opts)?;
    DefMacro!(ecs, ecs_args, expansion);
    if text_command_may_define(&cs)? {
      def_text_command_dispatcher(&cs, &cs_str)?;
    }
  }, locked => true);

  DefMacro!(
    "\\DeclareTextCommandDefault DefToken",
    "\\DeclareTextCommand{#1}{?}"
  );

  DefPrimitive!("\\ProvideTextCommand DefToken {}[Number][]{}",
  sub[(cs, encoding, nargs, opts, expansion)] {
    let cs_str = cs.to_string();
    let nargs = nargs.value_of() as usize;
    let encoding_str = Expand!(encoding).to_string();
    let ecs = T_CS!(s!("\\{encoding_str}{cs_str}"));
    if !IsDefined!(&ecs) { // If not already defined...
      let ecs_args = convert_latex_args(nargs, opts.clone())?;
      DefMacro!(ecs, ecs_args, expansion.clone());
    }
    if IsDefinable!(&cs) || text_command_may_define(&cs)? {
      def_text_command_dispatcher(&cs, &cs_str)?;
    }
  }, locked => true);

  DefMacro!(
    "\\ProvideTextCommandDefault DefToken",
    "\\ProvideTextCommand{#1}{?}"
  );

  // #------------------------------------------------------------

  DefPrimitive!("\\DeclareTextSymbol DefToken {}{Number}", sub[(cs, encoding, code)] {
    // Perl `latex_constructs.pool.ltxml:2671-2682`:
    //   if (isDefinableLaTeX($cs)) {
    //     DefMacroI($cs, undef,
    //       '\expandafter\ifx\csname\cf@encoding\string'.$css.'\endcsname\relax
    //          \csname?\string'.$css.'\endcsname
    //        \else\csname\cf@encoding\string'.$css.'\endcsname\fi');
    //   }
    //   my $ecs = T_CS('\\'.$encoding.$css);
    //   $STATE->installDefinition(CharDef->new($ecs, 'restricted_horizontal', $code, $encoding));
    //
    // The bare `\cs` is ALWAYS defined as a dispatch-chain macro that
    // resolves to `\<encoding>\cs` via `\csname`. The encoding-specific
    // `\<encoding>\cs` carries the actual glyph (as a CharDef in Perl;
    // as a `PrimitiveBody::String` primitive here — both produce a
    // single char on invocation).
    //
    // Witness for why this matters: paper 2305.16331 + mathtext.sty +
    // `\u\i` under T2A. mathtext.sty raw-loads and `\def\DeclareTextSymbol`
    // redefines our handler. T2A's `\DeclareTextSymbol{\i}{T2A}{25}` then
    // runs mathtext's chain (`\DeclareTextMathSymbol`), which redefines
    // `\i` to `\T2A-tmcmd \i \T2A\i \T2Amath\i`. That chain bottoms out
    // at `\csname\cf@encoding\string\i\endcsname = \T2A\i` — but only if
    // `\T2A\i` is a real definition. The prior happy-path bypassed both
    // the macro chain AND assumed `\i` would never be re-routed; under
    // mathtext's override `\i`'s chain looped, growing pushback past
    // the 4 GiB OOM boundary. With this Perl-faithful chain in place,
    // mathtext's override produces an equivalent chain that terminates
    // at the same `\T2A\i` primitive.
    // `u8::try_from`, not `as u8`: Perl hands the Number straight to
    // `CharDef->new` and the decode indexes the map, so an out-of-range code
    // yields no glyph. A truncating cast wrapped it onto a valid slot instead.
    let code_value = u8::try_from(code.value_of()).ok();
    let cs_str = cs.to_string();
    let encoding_str = Expand!(encoding).to_string();
    let ecs = T_CS!(s!("\\{encoding_str}{cs_str}"));
    // `decode_str`, not `decode`: a fontmap slot may hold MORE than one
    // character. Perl's `FontDecode` returns the whole string
    // (`$$map[$code]`), and Rust splits that across a single-`char` array plus
    // the `_fontmap_multichar` side table — which only `decode_str` consults,
    // along with T1's `SS` and the NBSP-prefixed combining accents. Decoding
    // with `decode` silently dropped the second character: T2B slot 128 came
    // out as `Ӷ` (U+04F6) instead of `Ӷ̶` (U+04F6 U+0336), i.e. a DIFFERENT
    // letter with its stroke removed, where Perl keeps the pair.
    if let Some(replacement_value) =
      code_value.and_then(|c| font::decode_str(c, Some(encoding_str), false))
    {
      // Encoding-specific carries the actual glyph.
      def_primitive(ecs, None, Some(PrimitiveBody::String(replacement_value)),
        PrimitiveOptions::default())?;
    } else if IsDefinable!(&ecs) {
      // Can't decode: install no-op fallback so downstream chains find
      // *something* to resolve to. Witness arXiv:1802.05444 / tipa T3.
      DefMacro!(ecs, None, Tokens!());
    }
    // Bare `\cs` always becomes a dispatch chain (matches Perl).
    if IsDefinable!(&cs) {
      DefMacro!(cs, None, Some(s!(
        r"\expandafter\ifx\csname\cf@encoding\string{cs_str}\endcsname\relax\csname?\string{cs_str}\endcsname\else\csname\cf@encoding\string{cs_str}\endcsname\fi"
      ).into()));
    }
  }, locked => true);

  // Perl `latex_constructs.pool.ltxml:2684-2688`:
  //   DefPrimitive('\DeclareTextSymbolDefault DefToken {}', sub {
  //     my ($stomach, $cs, $encoding) = @_;
  //     $encoding = ToString(Expand($encoding));
  //     DefMacroI(T_CS('\?' . ToString($cs)), undef,
  //               T_CS('\' . $encoding . ToString($cs))); });
  //
  // Registers the `\?<cs>` → `\<encoding><cs>` alias used by the
  // `DeclareTextSymbol` fallback (line 5662 above). Without this side
  // effect, tipa-style `\DeclareTextSymbolDefault\textrhookrevepsilon{T3}`
  // leaves `\textrhookrevepsilon` undefined for any encoding other than
  // T3, so a paper that loads tipa but is typeset in T1 errors.
  DefPrimitive!("\\DeclareTextSymbolDefault DefToken {}", sub[(cs, encoding)] {
    let cs_str = cs.to_string();
    let encoding_str = Expand!(encoding).to_string();
    // ltoutenc.dtx: `\DeclareTextSymbolDefault{\cs}{enc}` is
    // `\DeclareTextCommandDefault{\cs}{\UseTextSymbol{enc}\cs}` — the symbol
    // is typeset IN that encoding. Perl's bare `\<enc>\cs` alias only works
    // for numeric-slot symbols; lgrenc.def:190 `\DeclareTextCommand{\textsigma}
    // {LGR}{s\noboundary}` needs the LGR fontmap active or it prints a Latin
    // `s` (textalpha under T1; witnesses arXiv:2603.02703, 2604.09141).
    let alias_cs = T_CS!(s!("\\?{cs_str}"));
    DefMacro!(alias_cs, None, Some(s!("\\UseTextSymbol{{{encoding_str}}}{{{cs_str}}}").into()));
  }, locked => true);

  //------------------------------------------------------------
  // `\DeclareTextAccent{\cs}{enc}{slot}` — Perl ignores it (Base_Utility
  // `ignoredDefinition`), which left greek-fontenc's breathings and accents
  // (lgrenc.def:439-470 `\accdasia`, `\accperispomeni`, …) undefined:
  // teubner.sty:165 `\let\~\accperispomeni` then made `\~` itself undefined
  // (teubner-doc 1→87), textalpha's breathings `\<`/`\>` errored. OXIDIZED_DESIGN
  // #184: the encoding-specific command appends the COMBINING mark(s) the
  // slot's standalone glyph stands for (the kernel's own accent combiner map,
  // plus the Greek diacritics table below); the bare command becomes the
  // encoding dispatcher. A bare command that already exists (the kernel
  // accents `\'`, `\"`, `\~` …, `\DefAccent`'d natively) is left alone, as is
  // its encoding-specific slot, so the native accent path with its dotless-i
  // and typewriter rules stays in charge. Guard:
  // `perfect_kernel_batch54::declare_text_accent_defines_greek_diacritics`.
  DefPrimitive!("\\DeclareTextAccent DefToken {}{Number}", sub[(cs, encoding, code)] {
    if IsDefined!(&cs) {
      return Ok(vec![]);
    }
    let cs_str = cs.to_string();
    let encoding_str = Expand!(encoding).to_string();
    let ecs = T_CS!(s!("\\{encoding_str}{cs_str}"));
    let standalone = u8::try_from(code.value_of()).ok()
      .and_then(|c| font::decode_str(c, Some(encoding_str.clone()), false))
      .map(|sym| with(sym, |s| s.to_string()))
      .unwrap_or_default();
    let combining: Option<&str> = match standalone.as_str() {
      // Greek diacritics (LGR standalone glyph → combining sequence)
      "\u{1FEF}" => Some("\u{0300}"),          // varia
      "\u{1FFD}" | "\u{0384}" => Some("\u{0301}"), // oxia / tonos
      "\u{1FC0}" => Some("\u{0342}"),          // perispomeni
      "\u{00A8}" => Some("\u{0308}"),          // dialytika
      "\u{1FBF}" | "\u{2019}" => Some("\u{0313}"), // psili (`>`, slot 62)
      "\u{1FFE}" | "\u{201B}" => Some("\u{0314}"), // dasia (`<`, slot 60)
      "\u{1FCE}" => Some("\u{0313}\u{0301}"), // psili oxia
      "\u{1FCD}" => Some("\u{0313}\u{0300}"), // psili varia
      "\u{1FCF}" => Some("\u{0313}\u{0342}"), // psili perispomeni
      "\u{1FDE}" => Some("\u{0314}\u{0301}"), // dasia oxia
      "\u{1FDD}" => Some("\u{0314}\u{0300}"), // dasia varia
      "\u{1FDF}" => Some("\u{0314}\u{0342}"), // dasia perispomeni
      "\u{0385}" | "\u{1FEE}" => Some("\u{0308}\u{0301}"), // dialytika tonos / oxia
      "\u{1FED}" => Some("\u{0308}\u{0300}"), // dialytika varia
      "\u{1FC1}" => Some("\u{0308}\u{0342}"), // dialytika perispomeni
      "\u{02D8}" => Some("\u{0306}"),          // vrachy (breve)
      "\u{00AF}" => Some("\u{0304}"),          // macron
      _ => None,
    };
    let combining: Option<String> = combining.map(str::to_owned).or_else(|| {
      lookup_mapping("accent_combiner_above", &standalone)
        .or_else(|| lookup_mapping("accent_combiner_below", &standalone))
        .map(|v| v.to_string())
    });
    let marks = match combining {
      Some(marks) => marks,
      None => {
        Info!("unexpected", "DeclareTextAccent",
          s!("No combining form for accent {cs_str} in {encoding_str} (slot {}); it is dropped", code.value_of()));
        String::new()
      },
    };
    let body = mouth::tokenize_internal(TeXString::assembled(s!("#1{marks}")));
    def_macro(ecs, parse_parameters("{}", &ecs, true)?, ExpansionBody::Tokens(body), None)?;
    def_text_command_dispatcher(&cs, &cs_str)?;
  }, locked => true);
  DefPrimitive!("\\DeclareTextAccentDefault{}{}", None, locked => true);

  // TL2023+ kernel per-codepoint case-mapping declarations (ltmiscen:
  // `\DeclareUppercaseMapping{"0390}{\accdialytikatonos{\textiota}}` etc.)
  // — fine-tuning hints for `\MakeUppercase`/`\MakeLowercase`. LaTeXML
  // does Unicode-aware casing internally, so these are ignored exactly
  // like the `DeclareText*` font-slot family above (Perl has no handler
  // either — candidate upstream). The override matters beyond fidelity:
  // the kernel's own definitions ARE captured in the latex dump, so
  // without a native handler here the `\ifdefined` guards in e.g.
  // greek-fontenc's `lgrenc.def` pass and the raw expl3 kernel bodies
  // execute — hitting the raw-load expl3 catcode gap
  // (docs/EXPL3_CATCODE_GAP_2026-06-08.md) and spraying `Script _` +
  // undefined-accent errors at load time (witness: 81_babel greek_test
  // on TL2026, 87 errors → 0). Constructs load AFTER the dump applies
  // (strict-LoadFormat order), so these natively supersede the dumped
  // kernel macros. Args are read unexpanded, which also keeps babel's
  // active `"` shorthand inert inside the `{"03B0}` codepoint groups.
  DefPrimitive!("\\DeclareUppercaseMapping{}{}", None, locked => true);
  DefPrimitive!("\\DeclareLowercaseMapping{}{}", None, locked => true);
  DefPrimitive!("\\DeclareTitlecaseMapping{}{}", None, locked => true);

  DefMacro!("\\fontencoding{}", "\\lx@fontencoding{#1}");
  // Perl `latex_constructs.pool.ltxml:27-28`:
  //   DefMacroI('\f@encoding',  undef, sub { ExplodeText(LookupValue('font')->getEncoding); });
  //   DefMacroI('\cf@encoding', undef, sub { ExplodeText(LookupValue('font')->getEncoding); });
  // Perl's `undef->getEncoding` and `ExplodeText(undef)` both quietly degrade
  // to empty output instead of crashing; Rust must mirror that. Earlier code
  // chained `.unwrap().get_encoding().unwrap()`, which panicked on text-only
  // CSes used in math mode (e.g. `\i`, sandbox papers 0802.1100, 0811.2815,
  // 0901.4716, 0904.1706, 0905.1491).
  //
  // BUT a plain `.unwrap_or_default()` (→ "") on a live-font-with-no-encoding
  // diverges from Perl: a real Perl Font ALWAYS carries an encoding
  // (`Common/Font.pm:331` `encoding => $enc || 'OT1'`, `$DEFENCODING='OT1'`),
  // so `LookupValue('font')->getEncoding` is never empty when a font exists.
  // Rust's `Font::math_default()` deliberately leaves `encoding: None`
  // (font.rs:588 — char decoding differs), so the empty fallback leaked an
  // empty encoding name into the `\@changed@cmd`/`\@current@cmd` glyph lookup
  // `\csname\cf@encoding\string<cs>\endcsname` whenever a text-symbol CS
  // (`\i`, `\j`, accents) was expanded under the math font: "" builds the
  // bogus CS named "<cs>" (undefined) instead of the real `\<enc>\<cs>` glyph.
  // Mirror Perl: font present → its encoding, or OT1 when its slot is None
  // (Perl's always-OT1 default); no font at all → "" (Perl `undef->getEncoding`).
  //
  // NOTE: this does NOT fix the SHARED hyperref hang on 2004.08143
  // (`pdfauthor={…Mar{\'\i}n…}`) — there the font encoding is "ASCII" (set by
  // `beginSemiverbatim`), not None, so the OT1 fallback never triggers; that
  // loop reproduces in Perl too (see docs/parity/KNOWN_PERL_ERRORS.md, "text-symbol
  // CS in a Semiverbatim option").
  DefMacro!("\\f@encoding", {
    ExplodeText!(
      LookupFont!()
        .map(|f| f
          .get_encoding()
          .map(|e| e.to_string())
          .unwrap_or_else(|| "OT1".to_string()))
        .unwrap_or_default()
    )
  });
  DefMacro!("\\cf@encoding", {
    ExplodeText!(
      LookupFont!()
        .map(|f| f
          .get_encoding()
          .map(|e| e.to_string())
          .unwrap_or_else(|| "OT1".to_string()))
        .unwrap_or_default()
    )
  });

  // #------------------------------------------------------------
  // ltoutenc.dtx `\@text@composite`: an accent applied to a declared
  // argument uses the composite (`\<enc>\cs-<char>`) instead of the generic
  // mark — lgrenc.def:530-700 (`\DeclareTextComposite{\accdasia}{LGR}{a}
  // {129}`, `\DeclareTextCompositeCommand{\>}{LGR}{'}{\accpsilioxia}` from
  // textalpha.sty:189-194). Perl ignores both; the accent bodies installed by
  // `\DeclareTextAccent` above consult the composite first (OXIDIZED_DESIGN
  // #184). The key is `\string` of the argument's first token, as in the
  // kernel (`\csname\string#1-\string#2\endcsname`).
  // The first composite declared for `\<enc>\cs` wraps it (ltoutenc.dtx
  // `\@text@composite`): the original body moves to `\<enc>\cs@orig`, and
  // `\<enc>\cs{#1}` takes the composite when one exists for `#1`.
  fn wrap_text_command_for_composites(encoding_str: &str, cs_str: &str) -> Result<()> {
    let ecs = T_CS!(s!("\\{encoding_str}{cs_str}"));
    let orig = T_CS!(s!("\\{encoding_str}{cs_str}@orig"));
    if IsDefined!(&orig) || !IsDefined!(&ecs) {
      return Ok(());
    }
    Let!(orig, ecs);
    // `\lx@text@composite@key{#1}` is the `\string` of the argument's first
    // token, and EMPTY for an empty argument (`\accpsili{}` typesets the bare
    // mark; a raw `\string#1` would stringify the `\endcsname`).
    let body = mouth::tokenize_internal(TeXString::assembled(s!(
      "\\expandafter\\ifx\\csname {encoding_str}\\string{cs_str}-\\lx@text@composite@key{{#1}}\\endcsname\\relax\\expandafter\\@firstoftwo\\else\\expandafter\\@secondoftwo\\fi{{\\csname {encoding_str}\\string{cs_str}@orig\\endcsname{{#1}}}}{{\\csname {encoding_str}\\string{cs_str}-\\lx@text@composite@key{{#1}}\\endcsname}}"
    )));
    def_macro(
      ecs,
      parse_parameters("{}", &ecs, true)?,
      ExpansionBody::Tokens(body),
      None,
    )?;
    Ok(())
  }
  DefMacro!("\\lx@text@composite@key{}", sub[(arg)] {
    Ok(match arg.unlist_ref().first() {
      Some(t) => Tokens::new(Explode!(t.to_string())),
      None => Tokens!(),
    })
  });
  DefPrimitive!("\\DeclareTextComposite DefToken {} Undigested {Number}", sub[(cs, encoding, ch, code)] {
    let encoding_str = Expand!(encoding).to_string();
    let cs_str = cs.to_string();
    let key = T_CS!(s!("\\{encoding_str}{cs_str}-{}", ch.to_string()));
    if let Some(glyph) = u8::try_from(code.value_of()).ok()
      .and_then(|c| font::decode_str(c, Some(encoding_str.clone()), false))
    {
      def_primitive(key, None, Some(PrimitiveBody::String(glyph)), PrimitiveOptions::default())?;
      wrap_text_command_for_composites(&encoding_str, &cs_str)?;
    }
  }, locked => true);
  DefPrimitive!("\\DeclareTextCompositeCommand DefToken {} Undigested Undigested", sub[(cs, encoding, ch, cmd)] {
    let encoding_str = Expand!(encoding).to_string();
    let cs_str = cs.to_string();
    let key = T_CS!(s!("\\{encoding_str}{cs_str}-{}", ch.to_string()));
    def_macro(key, None, ExpansionBody::Tokens(cmd), None)?;
    wrap_text_command_for_composites(&encoding_str, &cs_str)?;
  }, locked => true);

  def_primitive_noop("\\UndeclareTextCommand{}{}")?;
  // Perl `latex_constructs.pool.ltxml:2642`:
  //   DefMacro('\UseTextSymbol{}{}', '{\fontencoding{#1}#2}');
  //
  // Perl's body verbatim is `{\fontencoding{#1}#2}`, and that shape CANNOT
  // TERMINATE when it is reached from a pure-expansion collect loop: `{` and
  // `\fontencoding` are non-expandable, so the loop gathers them without
  // executing, the font encoding never changes, and the inner `#2` re-enters
  // `\@changed@cmd` under the same encoding — forever.
  //
  // The live trigger is `beginSemiverbatim` (Perl `State.pm:597`, faithfully
  // ported at `state.rs:2691`) merging `encoding => 'ASCII'` — a stay-ASCII
  // neutralization, not a real LaTeX text encoding. So inside a Semiverbatim
  // argument (a `\cite` key, a `\usepackage` option value) `\cf@encoding` is
  // `ASCII`, `\ASCII\i` is undefined, and the `?`-fallback spins.
  //
  // Perl has the SAME looping shape available and simply reaches it less
  // often: measured 2026-07-26 against a format-equipped Perl 0.8.8 built with
  // `cpanm --build-arg formats .`, its own `latex_dump.pool.ltxml` carries
  // `\?\i -> \UseTextSymbol{OT1}\i` (72 `UseTextSymbol` records). So the dump
  // is NOT the differentiator — an earlier draft of this comment claimed it
  // was, and that was wrong. What is measured, on that one Perl install:
  //   * `\usepackage[pdfauthor={Mar{\'\i}n}]{hyperref}` — Perl HANGS (exit 124)
  //     exactly as we did: SHARED, the KNOWN_PERL_ERRORS entry.
  //   * `\cite{garc<U+00ED>a2024key}` under `[OT1]{fontenc}` — Perl converts
  //     cleanly in 0.89 s (`bibrefs="garcía2024key"`) while we looped:
  //     GENUINE-RUST-ONLY. Something in our `\cite`-key read reaches the
  //     encoding dispatch where Perl's does not; that residual delta is not
  //     yet pinned and is worth its own look.
  //
  // Either way the loop is a property of this macro's SHAPE, so fix it here:
  // resolve to the direct glyph — which is precisely what Perl's own
  // `\DeclareTextSymbolDefault` (`latex_constructs.pool.ltxml:2684-2688`,
  // ported above) makes `\?<cs>` expand to — and keep Perl's literal body as
  // the fallback when no such glyph exists. The observable result therefore
  // matches Perl wherever Perl terminates, and terminates where Perl does not.
  //
  // Witness 2606.11784 (`\usepackage[OT1]{fontenc}` + a literal `í` U+00ED in
  // a `\cite` key, mapped onto the text-symbol chain by the `.dfu`):
  // `Fatal:Timeout:PushbackLimit` with no output before, 0 errors / 519 KB
  // after. Also breaks the SHARED hang 2004.08143 — a surpass-Perl reliability
  // win, using the cure KNOWN_PERL_ERRORS "text-symbol CS in a Semiverbatim
  // argument" prescribes.
  // The encoding-specific command runs INSIDE its encoding (ltoutenc.dtx
  // `\UseTextSymbol` = `{\fontencoding{#1}\selectfont #2}`): a
  // `\DeclareTextCommand` body of plain letters (`s\noboundary`) decodes
  // through that fontmap. Resolving the encoding-specific CS directly (not
  // `#2`) keeps the dispatcher from re-entering (the PushbackLimit above).
  // A glyph primitive (`\DeclareTextSymbol`) is returned BARE — a `\cite`
  // key or hyperref option is read as Semiverbatim, where a group would not
  // be executed (fixture encoding/textsymbol_semiverbatim, 2606.11784).
  DefMacro!("\\UseTextSymbol{}{}", sub[(enc, cs)] {
    let enc_str = Expand!(enc).to_string();
    let cs_str = cs.to_string();
    let ecs = T_CS!(s!("\\{enc_str}{cs_str}"));
    match lookup_definition(&ecs)? {
      Some(defn) if defn.is_expandable() => Ok(Tokens!(
        T_BEGIN!(), T_CS!("\\fontencoding"), T_BEGIN!(), Explode!(enc_str), T_END!(),
        T_CS!("\\selectfont"), ecs, T_END!()
      )),
      Some(_) => Ok(Tokens!(ecs)),
      None => {
        let mut toks = vec![T_BEGIN!(), T_CS!("\\fontencoding"), T_BEGIN!()];
        toks.extend(Explode!(enc_str));
        toks.push(T_END!());
        toks.extend(cs.unlist());
        toks.push(T_END!());
        Ok(Tokens::new(toks))
      },
    }
  });
  DefMacro!("\\UseTextAccent{}{}", "{\\fontencoding{#1}#2{#3}}");

  // Perl: DefPrimitive('\DeclareMathAccent DefToken {}{} {Number}', ...)
  // latex_constructs.pool.ltxml:2702-2709. Perl always calls DefMathI even
  // when FontDecode returns undef (DefMathI normalizes `$presentation = ''`
  // when undef, Package.pm:1609). Earlier Rust skipped def_math when glyph
  // is None — that left the CS undefined for unknown encodings (e.g.
  // `\DeclareMathAccent{\widecheck}{\mathalpha}{mathx}{"71}` with no
  // mathx font map → \widecheck undefined → 1806.02506-style 1-error
  // cluster). Mirror Perl: always install, fall back to empty presentation.
  DefPrimitive!("\\DeclareMathAccent DefToken {}{} {Number}",
  sub[(cs, kind, class, code)] {
    let class_str = class.to_string();
    let encoding = lookup_value(&s!("fontdeclaration@{}", class_str))
      .and_then(|v| {
        if let Stored::Font(ref f) = v {
          f.get_encoding().map(|e| e.to_string())
        } else {
          None
        }
      })
      .unwrap_or(class_str);
    let (glyph, _font) = font_decode(code.value_of() as i32, Some(&encoding), None);
    let presentation = glyph.map(|c| c.to_string()).unwrap_or_default();
    let paramlist = parse_parameters("Digested", &cs, true)?;
    let opts = MathPrimitiveOptions{
      operator_role: Some("OVERACCENT".to_string()),
      ..Default::default()};
    def_math(cs, paramlist, presentation, opts)?;
    // Perl: return AddToPreamble('\DeclareMathAccent', $cs, $kind, $class, $code);
    // AddToPreamble returns Digest(Invocation(\lx@add@Preamble@PI, Invocation(\DeclareMathAccent, ...)))
    // The primitive must RETURN this digested result so it gets absorbed by the document.
    let preamble_text = format!("\\DeclareMathAccent{}{{{}}}{{{}}}{{{}}}",
      cs.with_str(|s| s.to_string()), kind, class, code.value_of());
    let preamble_toks = build_invocation(
      T_CS!("\\lx@add@Preamble@PI"),
      vec![Some(Tokens::new(Explode!(preamble_text)))])?;
    let digested = digest(preamble_toks)?;
    Ok(vec![digested])
  });

  // Perl: DefPrimitive('\DeclareMathSymbol DefToken SkipSpaces DefToken {}{Number}', ...)
  // my $symboltype_roles = { '\mathord' => 'ID', '\mathop' => 'BIGOP', '\mathbin' => 'BINOP',
  //   '\mathrel' => 'RELOP', '\mathopen' => 'OPEN', '\mathclose' => 'CLOSE', '\mathpunct' => 'PUNCT' };
  // locked: prevents raw TeX dump from overriding with version that errors on redefinition
  DefPrimitive!("\\DeclareMathSymbol DefToken SkipSpaces DefToken {}{Number}",
  sub[(cs, sym_type, fontkind, code)] {
    let mut encoding = fontkind.to_string();
    if let Some(Stored::Font(ref decl)) = lookup_value(&s!("fontdeclaration@{}", encoding))
      && let Some(enc) = decl.get_encoding() {
        encoding = enc.to_string();
      }
    let (glyph, _font) = font_decode(code.value_of() as i32, Some(&encoding), None);
    let role = match sym_type.to_string().as_str() {
      "\\mathord"  => Some("ID"),
      "\\mathop"   => Some("BIGOP"),
      "\\mathbin"  => Some("BINOP"),
      "\\mathrel"  => Some("RELOP"),
      "\\mathopen" => Some("OPEN"),
      "\\mathclose"=> Some("CLOSE"),
      "\\mathpunct"=> Some("PUNCT"),
      _ => None,
    };
    // Perl Package.pm L2761: `DefMathI($cs, undef, $glyph, role => $role)` —
    // called unconditionally, even when FontDecode returns `undef` (e.g. the
    // encoding's `.fontmap.ltxml` isn't shipped with LaTeXML, like "U").
    // Fall back to the raw codepoint so the CS is defined; better to render
    // an ASCII stand-in than to cascade into Error:undefined for the
    // command itself. arxiv 1011.1955 hits this with
    //   \DeclareSymbolFont{AMSb}{U}{msb}{m}{n}
    //   \DeclareMathSymbol{\Z}{\mathalpha}{AMSb}{"5A}
    // where no u.fontmap exists.
    //
    // XML-validity guard: slots in the C0 control range (0x00-0x1F, minus
    // tab/LF/CR) are NOT valid XML 1.0 characters and break downstream
    // libxml2 parsing of the serialized document (XPath aborts mid-tree
    // when it encounters one, so post-processor `find_node_by_id` returns
    // None for any id past the bad byte — manifested as the
    // `Error:expected:id Cannot find a node` cluster on 1501.05180,
    // where `\DeclareMathSymbol\onto\mathrel{latex-font msa}{"10}`
    // emits `<XMTok>\x10</XMTok>` with no msa fontmap installed).
    // Render U+FFFD (REPLACEMENT CHARACTER) for forbidden control chars
    // when no fontmap mapping is available — visually surfaces the
    // missing-fontmap case without poisoning the XML.
    fn xml_safe_char(codepoint: u32) -> String {
      if let Some(c) = char::from_u32(codepoint) {
        // XML 1.0 §2.2 valid Char: #x9 | #xA | #xD | [#x20-#xD7FF] |
        // [#xE000-#xFFFD] | [#x10000-#x10FFFF]
        let cp = c as u32;
        let valid = cp == 0x09
          || cp == 0x0A
          || cp == 0x0D
          || (0x20..=0xD7FF).contains(&cp)
          || (0xE000..=0xFFFD).contains(&cp)
          || (0x10000..=0x10FFFF).contains(&cp);
        if valid {
          c.to_string()
        } else {
          "\u{FFFD}".to_string()
        }
      } else {
        String::new()
      }
    }
    let presentation = match glyph {
      Some(ch) => ch.to_string(),
      None => xml_safe_char(code.value_of() as u32),
    };
    let mut opts = MathPrimitiveOptions::default();
    if let Some(r) = role {
      opts.role = Some(r.to_string());
    }
    def_math(cs, None, presentation, opts)?;
  });

  def_primitive_noop("\\DeclareMathDelimiter{}{}{}{}")?;
  def_primitive_noop("\\DeclareMathRadical{}{}{}{}{}")?;
  // latex.ltx `\DeclareMathVersion{name}` registers a math version a later
  // `\mathversion{name}` may select (oz.sty:34/:70, iwonamath.sty:110 with an
  // expl3 name, askmaps `sans`, zed `zed`); the noop left every declared
  // version "Unknown" (Perl latex_constructs.pool:2658 identical; 5 docs,
  // ozguide 28). The name is recorded fully expanded; no font change is
  // modelled beyond `bold`/`normal`. Guard:
  // `perfect_kernel_batch54::declared_math_versions_are_selectable`.
  DefPrimitive!("\\DeclareMathVersion Expanded", sub[(name)] {
    let name = name.to_string().trim().to_string();
    assign_value(&s!("MATH_VERSION_{name}"), Stored::Bool(true), Some(Scope::Global));
    Ok(())
  });
  def_primitive_noop("\\DeclarePreloadSizes{}{}{}{}{}")?;

  // The next font declaration commands are based on
  // http://tex.loria.fr/general/new/fntguide.html
  // we ignore font encoding
  // Perl: latex_constructs.pool.ltxml L2664 —
  // `\DeclareSymbolFont{} ExpandedPartially {}{}{}`. The encoding argument is
  // `ExpandedPartially` because `fontmath.ltx` and most font packages write
  // `\DeclareSymbolFont{operators}{\encodingdefault}{\rmdefault}{m}{n}`. Read
  // unexpanded, the literal string `\encodingdefault` lands in
  // `fontdeclaration@operators`, and every dependent `\DeclareMathSymbol` /
  // `\DeclareMathAccent` then looks up a fontmap by that name and finds none.
  DefPrimitive!("\\DeclareSymbolFont{} ExpandedPartially {}{}{}",
  sub[(name, enc, family, series, shape)] {
    AssignValue!(&s!("fontdeclaration@{}", name),
      fontmap!(family => family.to_string(),
        series   => series.to_string(),
        shape    => shape.to_string(),
        encoding => enc.to_string()
      )
    );
    // latex209.def L272-292 declares math fonts then uses
    //   `\DeclareRobustCommand\it{\normalfont\itshape\mathgroup\symitalic}`.
    // `\mathgroup` is `Let` to `\fam`, which expects a number. Plain TeX
    // would `\newfam\itfam` + `\mathchardef\symitalic=\itfam`. Stub `\sym<name>`
    // as a Let to `\z@` so `\fam\symitalic` parses cleanly (selecting fam 0,
    // since LaTeXML doesn't track active math fams). Without this, papers
    // using revtex 3.x + `{\it ...}` in math contexts hit `\symitalic`
    // undefined errors. Witness: cond-mat9911130, math0007178, hep-th9912229.
    let sym_cs = T_CS!(s!("\\sym{}", name));
    if !IsDefined!(&sym_cs) {
      Let!(&sym_cs, "\\z@");
    }
  });
  DefPrimitive!("\\DeclareSymbolFontAlphabet {Token} {}", sub[(cs, name)] {
    let fontkey = s!("fontdeclarations@{}", name.to_string());
    let font : Option<Font> = match lookup_value(&fontkey) { Some(Stored::Font(value)) => {
      Some((*value).clone())
    } _ => {
      None
    }};
    DefPrimitive!(cs, None, None, font => font);
  });

  // Perl latex_constructs.pool.ltxml L2764: defines the new CS as \relax.
  // Without a body, papers like 0706.2748 (`\DeclareFixedFont{\mytabfont}...`)
  // hit "T_CS[\mytabfont] is not defined" when later invoked.
  DefPrimitive!("\\DeclareFixedFont{}{}{}{}{}{}", sub[(cs, _enc, _fam, _ser, _sh, _sz)] {
    let cs_tok = T_CS!(cs.to_string());
    def_macro(cs_tok, None, Tokens!(T_CS!("\\relax")), None)?;
  });
  def_primitive_noop("\\DeclareErrorFont{}{}{}{}{}")?;
  // Font declaration stubs (Perl latex_constructs.pool.ltxml)
  def_primitive_noop("\\DeclareFontShape{}{}{}{}{}{}")?;
  def_primitive_noop("\\DeclareFontFamily{}{}{}")?;
  def_primitive_noop("\\DeclareSizeFunction{}{}")?;
  def_primitive_noop("\\DeclareMathSizes{}{}{}{}")?;
  // \newmathalphabet — pre-LaTeX2e (NFSS 1.0) math-alphabet declarator.
  // Effectively a no-op for XML output. NOTE the `, None)` arm is the
  // "discard everything" mock — `, None, None)` (3 args) would not
  // match any DefMacro arm and silently fail to register the CS.
  def_macro_noop("\\newmathalphabet{}{}{}")?;
  // \new@internalmathalphabet — obsolete LaTeX 2.09 kernel macro for
  // defining math alphabets, superseded by \DeclareSymbolFontAlphabet.
  // Used by old (1992-93) hep-th papers (~11 papers in stage-4 of the
  // 100k warning corpus, witness: arXiv:hep-th9211047 — \new@internalmathalphabet
  // \mathsf\sffam{cmss}{m}{n}). Stub with the same 5-arg signature as
  // \DeclareMathAlphabet so the args are consumed cleanly instead of
  // leaking into the document body via the auto-undefined-as-ERROR path.
  // SHARED-FAILURE with Perl (both fatal on this macro without a stub).
  def_macro_noop("\\new@internalmathalphabet{}{}{}{}{}")?;

  // LaTeX 2.09 size aliases (`\vpt`…`\xxvpt`) are intentionally NOT defined
  // here. Perl `latex_base.pool.ltxml:142-153` defines them, but they do NOT
  // survive into the dumped `latex.ltx` snapshot (only the `\@vpt`…`\@xxvpt`
  // dimensions do), so at runtime Perl leaves `\vpt`…`\xxvpt` *undefined*:
  // a 1990s hep-th paper that USES `\xpt` gets `Error:undefined:\xpt` in
  // Perl (verified), i.e. a SHARED error — not a Rust-only gap. A previous
  // port stubbed them as no-ops "to help those papers", but that (a) masked
  // the SHARED Perl error and, worse, (b) made the CS already-defined so a
  // paper's own `\newcommand{\vpt}{\tilde\varphi}` (a perfectly valid user
  // macro — `\vpt`/`\xpt` are NOT reserved in LaTeX 2e) was silently
  // dropped, then the now-empty `\vpt` left its `^`/`_` to re-attack the
  // previous atom for a spurious "Double/­triple sub/superscript". Witness
  // 1801.08339 (`\newcommand{\vpt}{\tilde{\varphi}}`, then `c^3\vpt^\circ` →
  // Rust double-superscript, Perl clean). Faithful parity: leave them
  // undefined, exactly like the Perl runtime.
  // DeclareMathAlphabet: define math font command if not already defined
  // Perl: latex_constructs.pool.ltxml L2677-2687. The arguments are NFSS
  // codes, and Perl maps them to LaTeXML's abstract font properties through
  // `lookupTeXFont` (Common/Font.pm L230-239) — the same
  // family/series/shape tables `\selectfont` consults. Storing the raw codes
  // instead put them straight into the XML: `\DeclareMathAlphabet{\mysf}{OT1}
  // {cmss}{m}{n}` emitted `font="cmss m n"` where Perl emits
  // `font="sansserif"`, and MathML then carried `mathvariant="normal"` for
  // every such alphabet — so a declared sansserif/bold/italic alphabet
  // rendered upright. `lookup_tex_font` was already a faithful port of
  // `lookupTeXFont`, with no callers.
  //
  // Only DOCUMENT- and PACKAGE-declared alphabets were affected: the stock
  // `\mathsf`/`\mathbf`/`\mathit`/`\mathrm` are bound directly in the pool and
  // never route through here. Font packages (fouriernc, mathpazo, newtxmath, …)
  // do declare their own.
  DefPrimitive!("\\DeclareMathAlphabet{}{}{}{}{}", sub[(cs, _enc, family, series, shape)] {
    let cs_tok = T_CS!(cs.to_string());
    // We won't override this, e.g. \mathrm by fouriernc.sty
    if IsDefined!(&cs_tok) {
      let csname = cs.to_string();
      let message = s!("Ignoring redefinition (\\DeclareMathAlphabet) of '{csname}'");
      Info!("ignore", csname, message);
    } else {
      let font : Option<Font> = Some(font::lookup_tex_font(
        &family.to_string(), &series.to_string(), &shape.to_string()));
      DefPrimitive!(cs_tok, None, None, font => font);
    }
  });

  DefMacro!("\\cdp@list", "\\@empty");
  Let!("\\cdp@elt", "\\relax");
  DefPrimitive!("\\DeclareFontEncoding{}{}{}", sub[(encoding, x, y)] {
    // Perl: AddToMacro(\cdp@list, \cdp@elt{enc}{family}{series}{shape})
    let cdp_cs = T_CS!("\\cdp@list");
    let enc_toks = encoding.clone().unlist();
    let mut cdp_tokens_vec = vec![T_CS!("\\cdp@elt"), T_BEGIN!()];
    cdp_tokens_vec.extend(enc_toks);
    cdp_tokens_vec.extend(vec![
      T_END!(),
      T_BEGIN!(), T_CS!("\\default@family"), T_END!(),
      T_BEGIN!(), T_CS!("\\default@series"), T_END!(),
      T_BEGIN!(), T_CS!("\\default@shape"), T_END!(),
    ]);
    let cdp_tokens = Tokens::new(cdp_tokens_vec);
    AddToMacro!(cdp_cs, cdp_tokens);

    let e = Expand!(encoding);
    DefMacro!(T_CS!("\\LastDeclaredEncoding"), None, e.clone());
    DefMacro!(T_CS!(s!("\\T@{}", e)), None, x);
    DefMacro!(T_CS!(s!("\\M@{}", e)), None, Tokens!(T_CS!("\\default@M"), y.unlist()));
    // LaTeX kernel ltoutenc.dtx defines `\<encoding>-cmd #1#2{#2}` as part
    // of \DeclareFontEncoding — the "switch to encoding-specific CS"
    // dispatcher used by `\DeclareTextCommand` bodies. Without this, every
    // `\i`-style symbol expansion hits an undefined `\T1-cmd` cascade
    // that re-injects the original CS into the input and infinite-loops
    // (driver: 2306.16410 — paper hangs in token-limit when reading
    // `\citep{surís2023vipergpt}` after `\usepackage[T1]{fontenc}`,
    // because `\i` expands to `\T1-cmd \i \T1\i` which loops back to
    // `\i` when `\T1-cmd` isn't defined. 2402.01687 (sigplan) was a
    // suspected second driver of the same UTF-8-in-bib-key pattern,
    // never confirmed — check it first if this cascade resurfaces).
    let enc_cmd = s!("\\{}-cmd", e);
    DefMacro!(T_CS!(enc_cmd), "{}{}", "#2");

    // Perl `latex_constructs.pool.ltxml:2781-2783`:
    //   if (my $path = $encoding_str && FindFile(lc($encoding_str)."enc", type=>"dfu")) {
    //     InputDefinitions($path); }
    // Without this, `\DeclareFontEncoding{TS1}` (textcomp.sty.ltxml tail)
    // never loads `ts1enc.dfu` and `\DeclareUnicodeCharacter` mappings
    // from the .dfu are missing — surfacing as bogus undefineds when
    // user input contains TS1 glyphs that should resolve via the .dfu.
    //
    // BUT: skip the .dfu load when `\DeclareUnicodeCharacter` has been
    // `\let` to `\@undefined` (the signal `\UseRawInputEncoding` uses
    // to disable inputenc-style mappings — latex.ltx L18271). The .dfu
    // body is nothing but `\DeclareUnicodeCharacter{...}` calls, and
    // each one would emit "undefined CS" if we proceeded. Real pdflatex
    // tolerates this because `\DeclareFontEncoding` does not auto-load
    // the .dfu — only inputenc.sty does. Witness: arXiv:2509.22585
    // (revtex4-2 + `\UseRawInputEncoding` + `\usepackage[T1]{fontenc}`).
    use latexml_core::binding::content::{
      FindFileOptions, InputDefinitionOptions, find_file, input_definitions,
    };
    let duc_defined = has_meaning(&T_CS!("\\DeclareUnicodeCharacter"));
    let enc_str = e.to_string();
    if duc_defined && !enc_str.is_empty() {
      let dfu_name = format!("{}enc", enc_str.to_lowercase());
      if find_file(&dfu_name, Some(FindFileOptions {
        forbid_ltxml: true,
        notex: false,
        ext_type: Some(Cow::Borrowed("dfu")),
        search_paths_only: false,
      })).is_some() {
        let opts = InputDefinitionOptions {
          extension: Some(Cow::Borrowed("dfu")),
          noltxml:   true,
          raw:       true,
          ..Default::default()
        };
        let _ = input_definitions(&dfu_name, opts);
      }
    }
  });

  DefMacro!("\\LastDeclaredEncoding", None, None);
  // utf8.def:253-265 — the octet arithmetic behind `\DeclareUnicodeCharacter`,
  // KERNEL macros since latex.ltx:22224 `\input{utf8.def}` at format time.
  // The engine decodes UTF-8 natively, but packages that build their own UTF-8
  // sequences `\let` these internals: paresse-utf8.sty:203-204
  // `\global\let\GA@parse@UTFviii@a=\parse@UTFviii@a` (paresse-eng/-fra 3/6
  // errors; Perl utf8.def.ltxml omits them too, KPE #163). Pure
  // `\uccode`/`\count@` arithmetic, verbatim. Guard:
  // `perfect_kernel_batch54::utf8_octet_parsers_are_defined`.
  RawTeX!(
    r##"\gdef\parse@UTFviii@a#1{%
     \@tempcnta\count@
     \divide\count@ 64
     \@tempcntb\count@
     \multiply\count@ 64
     \advance\@tempcnta-\count@
     \advance\@tempcnta 128
     \uccode`#1\@tempcnta
     \count@\@tempcntb}
\gdef\parse@UTFviii@b#1#2#3#4{%
     \advance\count@ "#10\relax
     \uccode`#3\count@
     \uppercase{\gdef\UTFviii@tmp{#2#3#4}}}"##
  );

  // \DeclareUnicodeCharacter — from utf8.def / latex_constructs
  // Maps a hex codepoint to an expansion, making the character active.
  DefPrimitive!("\\DeclareUnicodeCharacter Expanded {}", sub[(hexcode, expansion)] {
    let hex_str = hexcode.to_string();
    let hex_str = hex_str.trim();
    if hex_str.chars().all(|c| c.is_ascii_hexdigit()) && !hex_str.is_empty() {
      if let Ok(cp) = u32::from_str_radix(hex_str, 16) {
        if cp <= 0x10FFFF {
          if let Some(ch) = char::from_u32(cp) {
            AssignCatcode!(ch, Catcode::ACTIVE);
            DefMacro!(T_ACTIVE!(ch), None, expansion);
          }
        } else {
          Error!("unexpected", hex_str,
            s!("{} too large for Unicode. Values between 0 and 10FFFF are permitted.", hex_str));
        }
      }
    } else {
      Error!("unexpected", hex_str,
        s!("Non-hex value {} in \\DeclareUnicodeCharacter", hex_str));
    }
  });

  def_primitive_noop("\\DeclareFontSubstitution{}{}{}{}")?;
  def_primitive_noop("\\DeclareFontEncodingDefaults{}{}")?;
  DefMacro!("\\LastDeclaredEncoding", None, None);

  def_primitive_noop("\\SetSymbolFont{}{}{}{}{}{}")?;
  def_primitive_noop("\\SetMathAlphabet{}{}{}{}{}{}")?;
  def_primitive_noop("\\addtoversion{}{}")?;
  def_primitive_noop("\\TextSymbolUnavailable{}")?;

  // LaTeX3 ltcmd: \NewCommandCopy and \DeclareCommandCopy
  // These are semantic \let equivalents from the 2023+ LaTeX kernel.
  // Not in Perl LaTeXML (too new), but needed for modern packages (tcolorbox, etc.).
  //
  // ltcmd defines them as `\NewDocumentCommand \NewCommandCopy { m m }` —
  // both args are mandatory, accepting either `\foo` (bare token) or
  // `{\foo}` (brace-wrapped token). Real arxmliv usage is the brace
  // \NewCommandCopy / \DeclareCommandCopy / \ShowCommand defined in
  // `latex_constructs_rust_only.rs` (modern LaTeX 2023+ kernel CSes;
  // not in Perl). Moved there for single-source-of-truth and to keep
  // bug fix (brace-wrapped token unwrap; arXiv:2510.20194 witness) from
  // being overridden by an older copy.

  TeX!(
    r#"""
  \DeclareSymbolFont{operators}   {OT1}{cmr} {m}{n}
  \DeclareSymbolFont{letters}     {OML}{cmm} {m}{it}
  \DeclareSymbolFont{symbols}     {OMS}{cmsy}{m}{n}
  \DeclareSymbolFont{largesymbols}{OMX}{cmex}{m}{n}
  """#
  );
  // Perl: latex_constructs.pool.ltxml L5759-5764 — picture font stubs
  DefPrimitive!("\\OMX", None, font => { family => "cmex10" });
  DefPrimitive!("\\tenln", None, font => { family => "line10" });
  DefPrimitive!("\\tenlnw", None, font => { family => "linew10" });
  DefPrimitive!("\\tencirc", None, font => { family => "lcircle10" });
  DefPrimitive!("\\tencircw", None, font => { family => "lcirclew10" });

  // Perl latex_constructs.pool.ltxml L2814-2832: uclclist members are
  // DefPrimitiveI(..., robust=>1) — Expandable wrapper expanding to
  // `\protect <cs-munged>` (Rust `def_robust_cs`), with the munged CS
  // as the primitive emitting the Unicode char. `\MakeUppercase`'s
  // case-mapping pipeline reads `\protect <cs>` pairs; see
  // `lx_read_and_change_case` protect-branch + `\lx@prepare@case@mapping`.
  DefPrimitive!("\\OE", "\u{0152}", robust => true); // LATIN CAPITAL LIGATURE OE
  DefPrimitive!("\\oe", "\u{0153}", robust => true); // LATIN SMALL LIGATURE OE
  DefPrimitive!("\\AE", "\u{00C6}", robust => true); // LATIN CAPITAL LETTER AE
  DefPrimitive!("\\ae", "\u{00E6}", robust => true); // LATIN SMALL LETTER AE
  DefPrimitive!("\\AA", "\u{00C5}", robust => true); // LATIN CAPITAL LETTER A WITH RING ABOVE
  DefPrimitive!("\\aa", "\u{00E5}", robust => true); // LATIN SMALL LETTER A WITH RING ABOVE
  DefPrimitive!("\\O",  "\u{00D8}", robust => true); // LATIN CAPITAL LETTER O WITH STROKE
  DefPrimitive!("\\o",  "\u{00F8}", robust => true); // LATIN SMALL LETTER O WITH STROKE
  DefPrimitive!("\\L",  "\u{0141}", robust => true); // LATIN CAPITAL LETTER L WITH STROKE
  DefPrimitive!("\\l",  "\u{0142}", robust => true); // LATIN SMALL LETTER L WITH STROKE
  DefPrimitive!("\\ss", "\u{00DF}", robust => true); // LATIN SMALL LETTER SHARP S
  DefPrimitive!("\\dh", "\u{00F0}", robust => true); // eth
  DefPrimitive!("\\DH", "\u{00D0}", robust => true); // Eth (looks same as \DJ!)
  DefPrimitive!("\\dj", "\u{0111}", robust => true); // d with stroke
  DefPrimitive!("\\DJ", "\u{0110}", robust => true); // D with stroke (looks same as \DH!)
  DefPrimitive!("\\ng", "\u{014B}", robust => true);
  DefPrimitive!("\\NG", "\u{014A}", robust => true);
  DefPrimitive!("\\th", "\u{00FE}", robust => true);
  DefPrimitive!("\\TH", "\u{00DE}", robust => true);

  DefPrimitive!("\\newenvironment OptionalMatch:* {}[Number][]{}{}",
  sub[(_star_opt, name, nargs, opt, begin, end)] {
    let name = { Expand!(name).to_string() };
    let name_cs = T_CS!(format!("\\{name}"));
    // Use `is_definable_latex` (not a bare `IsDefined!`) so an autoload TRIGGER
    // for `\<name>` (installed by `def_autoload`, e.g. `\align`→amsmath) does NOT
    // block the user's `\newenvironment`. Perl's analogous `DefAutoload` entries
    // live in OmniBus.cls.ltxml (not loaded for typical papers), so there `\align`
    // is genuinely undefined and `\newenvironment{align}{…}` SUCCEEDS. `\newcommand`
    // already uses this check; `\newenvironment` must match. Witness 1907.04260
    // (iopart + amssymb): `\newenvironment{align}{\begin{eqnarray}}{\end{eqnarray}}`
    // was silently ignored because amssymb→amsfonts left the `\align` autoload
    // trigger in place; the doc then ran amsmath's `align`, and a following
    // `\cases`-equation desynced math mode → 71-error cascade. Perl: 0 (its `align`
    // is the author's eqnarray wrapper).
    let (definable, _plain_origin) = is_definable_latex(&name_cs)?;
    if !definable {
      let is_locked = lookup_bool(&s!("\\{}:locked",name)) ||
       lookup_bool(&s!("\\begin{{{}}}:locked",name));
      if !is_locked {
        let message = s!("Ignoring redefinition (\\newenvironment) of Environment {:?}", name);
        Info!("ignore", name, message);
      }
    } else {
      // TODO: can we convince DefMacro! this is not a second mutable borrow of state::
      let converted_args = convert_latex_args(nargs.value_of() as usize, opt)?;
      let end_name_cs = T_CS!(s!("\\end{}",name));
      DefMacro!(name_cs, converted_args, begin);
      DefMacro!(end_name_cs, None, end);
    }
    Ok(Vec::new())
  });

  DefPrimitive!("\\renewenvironment OptionalMatch:* {}[Number][]{}{}",
  sub[(_star, name, nargs, opt, begin, end)] {
    let name = Expand!(name).to_string();
    let is_locked = lookup_bool(&s!("\\{}:locked",name)) ||
       lookup_bool(&s!("\\begin{{{}}}:locked",name));
    if !is_locked {
      let name_cs = T_CS!(s!("\\{}",name));
      let end_name_cs = T_CS!(s!("\\end{}",name));
      let converted_args = convert_latex_args(nargs.value_of() as usize, opt)?;

      DefMacro!(name_cs, converted_args, begin);
      DefMacro!(end_name_cs, None, end);
    }
    Ok(Vec::new())
  });

  //======================================================================
  // C.8.3 Theorem-like Environments
  //======================================================================
  AssignValue!("thm@swap" => 0i64);
  DefRegister!("\\thm@style"         => Tokens!(T_OTHER!("plain")));
  DefRegister!("\\thm@headfont"      => Tokens!(T_CS!("\\bfseries")));
  DefRegister!("\\thm@notefont"      => Tokens!(T_CS!("\\the"), T_CS!("\\thm@headfont")));
  DefRegister!("\\thm@bodyfont"      => Tokens!(T_CS!("\\itshape")));
  DefRegister!("\\thm@headformatter" => Tokens!());
  DefRegister!("\\thm@headpunct"     => Tokens!());
  DefRegister!("\\thm@styling"       => Tokens!());
  DefRegister!("\\thm@headstyling"   => Tokens!());
  DefRegister!("\\thm@prework"       => Tokens!());
  DefRegister!("\\thm@postwork"      => Tokens!());
  DefRegister!("\\thm@symbol"        => Tokens!());
  DefRegister!("\\thm@numbering"     => Tokens!(T_CS!("\\arabic")));

  DefPrimitive!("\\th@plain", {
    assign_register(
      "\\thm@bodyfont",
      RegisterValue::Tokens(Tokens!(T_CS!("\\itshape"))),
      None,
      vec![],
    )?;
    assign_register(
      "\\thm@headstyling",
      RegisterValue::Tokens(Tokens!(T_CS!("\\lx@makerunin"))),
      None,
      vec![],
    )?;
  });

  DefMacro!("\\lx@makerunin", "\\@ADDCLASS{ltx_runin}");
  DefMacro!("\\lx@makeoutdent", "\\@ADDCLASS{ltx_outdent}");

  DefMacro!("\\@thmcountersep", ".");
  def_macro_noop("\\thm@doendmark")?;

  init_savable_theorem_parameters(vec![
    "\\thm@bodyfont",
    "\\thm@headpunct",
    "\\thm@styling",
    "\\thm@headstyling",
    "thm@swap",
  ]);

  // Activate the default style.
  RawTeX!("\\th@plain");

  Tag!("ltx:theorem", auto_close => true);
  Tag!("ltx:proof",   auto_close => true);

  // The extra leading `[]` absorbs (and DISCARDS) a style optional some
  // classes accept there — aomart.cls L676-679 rewraps \newtheorem so
  // `\newtheorem[{}\it]{thm}{Theorem}[section]` is valid input, but its
  // wrapper is a no-op on a LOCKED CS in both engines, so our unmodified
  // signature grabbed `[` as the theorem NAME, defining an environment
  // called `[` whose csname form clobbered `\[` — every later display
  // math opened a spurious theorem (aomsample ×2, 89 of 101 errs; Perl
  // 0.8.8 shares byte-identically; pdflatex clean). The class's own
  // handler discards the optional (`\@aom@newthm@[#1]{\@xnthm\relax}`),
  // so discarding is the ground truth. Standard forms start with `{` and
  // never match the optional. KNOWN_PERL_ERRORS #82.
  DefPrimitive!("\\newtheorem OptionalMatch:* [] {}[]{}[]", sub[(flag, _style_opt, thmset, otherthmset, typ, reset)] {
    define_new_theorem(
      flag.filter(|f| !f.is_empty()),
      thmset,
      otherthmset.filter(|t| !t.is_empty()),
      if typ.is_empty() { None } else { Some(typ) },
      reset.filter(|t| !t.is_empty()),
      None, // \newtheorem body font comes from \theoremstyle, not a per-theorem arg
    )?;
    // Reset these!
    assign_register("\\thm@prework",
      RegisterValue::Tokens(Tokens!()), None, vec![])?;
    assign_register("\\thm@postwork",
      RegisterValue::Tokens(Tokens!()), None, vec![])?;
  });

  //======================================================================
  // C.8.4 Numbering
  //======================================================================
  // For LaTeX documents, We want id's on para, as well as sectional units.
  // However, para get created implicitly on Document construction, rather than
  // explicitly during digestion (via a whatsit), we can't use the usual LaTeX counter mechanism.
  Tag!("ltx:para", after_open => sub[document, node] {
    document.generate_id(node, "p")?;
  });

  // \newcounter moved to latex_bootstrap.rs (Perl latex_bootstrap.pool.ltxml L51-53,
  // locked => 1) so it's available before the dump and constructs phases.
  DefPrimitive!("\\setcounter{}{Number}", sub[(cs, default)] {
    let cs_expanded = &Expand!(cs).to_string();
    SetCounter!(cs_expanded, default);
  });
  DefPrimitive!("\\addtocounter{}{Number}", sub[(cs,default)] {
    let cs_expanded = &Expand!(cs).to_string();
    AddToCounter!(cs_expanded, default);
  });
  DefPrimitive!("\\stepcounter{}",    sub[(cs)] {
    let cs_expanded = &Expand!(cs).to_string();
    StepCounter!(cs_expanded, false)?;
  });
  DefPrimitive!("\\refstepcounter{}", sub[(cs)] {
    let cs_expanded = &Expand!(cs).to_string();
    RefStepCounter!(cs_expanded, false)?;
  });
  // latex.ltx:14978 `\def\labelformat#1{\expandafter\def\csname p@#1\endcsname##1}`
  // — kernel since 2019-10-01 (varioref only re-exports it). Was undefined in
  // Rust AND Perl (KPE #160): contract.sty:978 probes it with
  // `\scr@ifundefinedorrelax{labelformat}` and fell back to its pre-2019
  // `\p@sentence`=`\expandafter\p@@sentence` prefix, whose one-token grab of
  // `\thesentence`'s expansion left `{sentence}` behind and ended
  // `\refstepcounter`'s label with `\arabic}` ("You can't use } after \the"
  // ×3 per sentence; contract-example-en 44 errors). The refnum formatter
  // (`\lx@@therefnum@@`, base_utilities.rs) already applies the argument-taking
  // `\p@<ctr>` this defines exactly as latex.ltx:14976's `\@currentlabel` does.
  // Guard: `perfect_kernel_batch54::labelformat_is_a_kernel_macro`.
  DefMacro!(
    "\\labelformat{}",
    r"\expandafter\def\csname p@#1\endcsname##1"
  );

  // Perl latex_constructs.pool.ltxml: addtoCounterReset + defCounterID
  DefPrimitive!("\\@addtoreset{}{}", sub[(ctr, within)] {
    let ctr_str = Expand!(ctr).to_string();
    let within_str = Expand!(within).to_string();
    let unctr = s!("UN{}", ctr_str);
    let reg = s!("\\cl@{}", within_str);
    // Prepend ctr and UNctr to the counter reset list for 'within'
    let prev = lookup_tokens(&reg).unwrap_or_default();
    let mut toks = vec![T_CS!(ctr_str), T_CS!(unctr)];
    toks.extend(prev.unlist());
    assign_value(&reg, Stored::Tokens(Tokens::new(toks)), None);
    sync_reset_list_macro(&within_str)?;
  });

  // Perl: latex_constructs.pool.ltxml \@removefromreset
  DefPrimitive!("\\@removefromreset{}{}", sub[(ctr, within)] {
    let ctr_str = Expand!(ctr).to_string();
    let within_str = Expand!(within).to_string();
    let reg = s!("\\cl@{}", within_str);
    if let Some(prev) = lookup_tokens(&reg) {
      let unctr_cs = T_CS!(s!("UN{}", ctr_str));
      let ctr_cs = T_CS!(ctr_str);
      let filtered: Vec<Token> = prev.unlist().into_iter()
        .filter(|t| *t != ctr_cs && *t != unctr_cs)
        .collect();
      assign_value(&reg, Stored::Tokens(Tokens::new(filtered)), None);
      sync_reset_list_macro(&within_str)?;
    }
  });

  // Perl: latex_constructs.pool.ltxml \counterwithin
  DefPrimitive!("\\counterwithin OptionalMatch:* {}{}", sub[(star, ctr, within)] {
    let ctr_str = Expand!(ctr).to_string();
    let within_str = Expand!(within).to_string();
    // Add ctr to reset list of within
    let unctr = s!("UN{}", ctr_str);
    let reg = s!("\\cl@{}", within_str);
    let prev = lookup_tokens(&reg).unwrap_or_default();
    let mut toks = vec![T_CS!(ctr_str.clone()), T_CS!(unctr)];
    toks.extend(prev.unlist());
    assign_value(&reg, Stored::Tokens(Tokens::new(toks)), None);
    sync_reset_list_macro(&within_str)?;
    if star.is_none() {
      // Redefine \thectr to include \thewithin prefix
      let the_ctr = T_CS!(s!("\\the{}", ctr_str));
      let expansion = s!("\\the{}.\\arabic{{{}}}", within_str, ctr_str);
      let _ = def_macro(the_ctr, None,
        Some(ExpansionBody::from(expansion)),
        Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))));
      // defCounterID with within
      let prefix = lookup_string(&s!("@ID@prefix@{}", ctr_str));
      let clean_prefix = if prefix.is_empty() { ctr_str.clone() } else { prefix };
      let ctr_for_id = ctr_str.clone();
      let within_for_id = within_str;
      let thectrid = s!("\\the{}@ID", ctr_str);
      let _ = def_macro(T_CS!(thectrid), None,
        Some(ExpansionBody::Closure(Rc::new(move |_args| {
          Ok(mouth::tokenize_internal(TeXString::assembled(s!(
            "\\expandafter\\ifx\\csname the{}@ID\\endcsname\\@empty\\else\\csname the{}@ID\\endcsname.\\fi {}\\csname @{}@ID\\endcsname",
            within_for_id, within_for_id, clean_prefix, ctr_for_id
          ))))
        }))),
        Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))));
    }
  });

  // Perl: latex_constructs.pool.ltxml \counterwithout
  DefPrimitive!("\\counterwithout OptionalMatch:* {}{}", sub[(star, ctr, within)] {
    let ctr_str = Expand!(ctr).to_string();
    let within_str = Expand!(within).to_string();
    // Remove ctr from reset list of within
    let reg = s!("\\cl@{}", within_str);
    if let Some(prev) = lookup_tokens(&reg) {
      let ctr_cs = T_CS!(ctr_str.clone());
      let unctr_cs = T_CS!(s!("UN{}", ctr_str));
      let filtered: Vec<Token> = prev.unlist().into_iter()
        .filter(|t| *t != ctr_cs && *t != unctr_cs)
        .collect();
      assign_value(&reg, Stored::Tokens(Tokens::new(filtered)), None);
      sync_reset_list_macro(&within_str)?;
    }
    if star.is_none() {
      // Redefine \thectr without prefix
      let the_ctr = T_CS!(s!("\\the{}", ctr_str));
      let expansion = s!("\\arabic{{{}}}", ctr_str);
      let _ = def_macro(the_ctr, None,
        Some(ExpansionBody::from(expansion)),
        Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))));
      // defCounterID without within — redefine \thectr@ID
      let prefix = lookup_string(&s!("@ID@prefix@{}", ctr_str));
      let clean_prefix = if prefix.is_empty() { ctr_str.clone() } else { prefix };
      let ctr_for_id = ctr_str.clone();
      let thectrid = s!("\\the{}@ID", ctr_str);
      let _ = def_macro(T_CS!(thectrid), None,
        Some(ExpansionBody::Closure(Rc::new(move |_args| {
          Ok(mouth::tokenize_internal(TeXString::assembled(s!(
            "{}\\csname @{}@ID\\endcsname", clean_prefix, ctr_for_id
          ))))
        }))),
        Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))));
    }
  });

  DefMacro!("\\cl@@ckpt", "\\@elt{page}");

  DefMacro!("\\value{}", sub[(value)] {
    let name = Expand!(value).to_string();
    // `\newtheorem{lemma}[theorem]{Lemma}` shares theorem's counter — no
    // \c@lemma is created, only the `counter_for_type` mapping. So
    // `\value{lemma}` would expand to the undefined `\c@lemma`. Resolve
    // through the mapping first. Driver: 2101.03928
    // `\setcounter{lemmathreesets}{\value{lemma}}` (llncs paper).
    let resolved = match lookup_mapping("counter_for_type", &name) {
      Some(Stored::String(s)) => with(s, |s| s.to_string()),
      _ => name,
    };
    T_CS!(s!("\\c@{resolved}"))
  });

  DefMacro!("\\@arabic{Number}", sub[(number)] {
    ExplodeText!(number.value_of().to_string())
  });
  // latex.ltx L15715: \def\two@digits#1{\ifnum#1<10 0\fi\number#1}
  // Zero-pad a number to at least two digits (date/time formatting helper,
  // used by \today and many class/package date macros). Faithful literal
  // port of the kernel `\def`. Was undefined in Rust → packages calling it
  // directly errored and cascaded (witness 2206.12768: undefined:\two@digits
  // → expected:<relationaltoken> in the surrounding \ifnum). Perl defines it
  // via the kernel; minimal repro `\two@digits{7}` → Rust nothing / Perl 07.
  DefMacro!("\\two@digits{}", r"\ifnum#1<10 0\fi\number#1");
  // latex.ltx L6977-6982: \@removeelement{elt}{list}{\cmd} — remove the
  // comma-list element `elt` from `list`, store the result in `\cmd`.
  // Faithful literal port of the kernel `\def` (nested delimited macros over
  // `\reserved@a`/`\reserved@b`). Was undefined in Rust; Perl defines it via
  // the kernel. Used by grfext and other comma-list-manipulating packages
  // (witness 2309.13586 chain). `\@empty` already exists; `\reserved@a/b` are
  // scratch macros defined inline by the body.
  DefMacro!(
    "\\@removeelement{}{}{}",
    r"\def\reserved@a##1,#1,##2\reserved@a{##1,##2\reserved@b}\def\reserved@b##1,\reserved@b##2\reserved@b{\ifx,##1\@empty\else##1\fi}\edef#3{\expandafter\reserved@b\reserved@a,#2,\reserved@b,#1,\reserved@a}"
  );
  // latex.ltx L7634: \protected\def\leavevmode@ifvmode{\ifvmode\expandafter\indent\fi}
  // Emit \indent only when currently in vertical mode (used by \enspace and
  // by `\vcenter`/box helpers). Was undefined in Rust; Perl defines it via the
  // kernel. `\protected` (matched here) so it survives \edef/serialization
  // unexpanded — it surfaced inside serialized .bbl math (witness 2312.14913).
  // Faithful port; \ifvmode/\indent already exist.
  DefMacro!("\\leavevmode@ifvmode", r"\ifvmode\expandafter\indent\fi", protected => true);
  // latex.ltx L14276-14285: \@starttoc{ext} — input the .ext toc-style file
  // and (re)open it for writing the \contentsline entries. Faithful literal
  // port; in LaTeXML the file I/O is in-memory-cached (\openout/\@input) and
  // the actual TOC is built from the captured \contentsline entries during
  // post-processing, so this just lets packages that drive custom lists via
  // \@starttoc{loa}/etc. run without erroring. Was undefined in Rust; Perl
  // defines it via the kernel (witness 2211.02345). All deps
  // (\@input/\newwrite/\if@filesw/\@nobreakfalse) already exist.
  DefMacro!(
    "\\@starttoc{}",
    r"\begingroup\makeatletter\@input{\jobname.#1}\if@filesw\expandafter\newwrite\csname tf@#1\endcsname\immediate\openout \csname tf@#1\endcsname \jobname.#1\relax\fi\@nobreakfalse\endgroup"
  );
  DefMacro!("\\arabic{}", sub[(value)] {
    let ctr_expansion = Expand!(value).to_string();
    let ctr_value = CounterValue!(&ctr_expansion).value_of();
    ExplodeText!(ctr_value)
  });

  DefMacro!("\\@roman{Number}", sub[(number)] {
    ExplodeText!(radix::radix_roman(number.value_of()))
  });
  DefMacro!("\\roman{}", sub[(token)] {
    let ctr = Expand!(token).to_string();
    ExplodeText!(radix::radix_roman(CounterValue!(&ctr).value_of()))
  });
  DefMacro!("\\@Roman{Number}", sub[(number)] {
    ExplodeText!(radix::radix_up_roman(number.value_of()))
  });
  DefMacro!("\\Roman{}", sub[(token)] {
    let ctr = Expand!(token).to_string();
    ExplodeText!(radix::radix_up_roman(CounterValue!(&ctr).value_of()))
  });
  DefMacro!("\\@alph{Number}", sub[(number)] {
    ExplodeText!(radix::radix_alpha(number.value_of()))
  });
  DefMacro!("\\alph{}", sub[(token)] {
    let ctr = Expand!(token).to_string();
    ExplodeText!(radix::radix_alpha(CounterValue!(&ctr).value_of()))
  });
  DefMacro!("\\@Alph{Number}", sub[(number)] {
    ExplodeText!(radix::radix_up_alpha(number.value_of()))
  });
  DefMacro!("\\Alph{}", sub[(token)] {
    let ctr = Expand!(token).to_string();
    ExplodeText!(radix::radix_up_alpha(CounterValue!(&ctr).value_of()))
  });

  DefMacro!("\\@fnsymbol{Number}", sub[(number)] {
    ExplodeText!(radix::radix_format_str(number.value_of(), FNSYMBOLS))
  });
  DefMacro!("\\fnsymbol{}", sub[(token)] {
    let ctr = Expand!(token).to_string();
    ExplodeText!(radix::radix_format_str(CounterValue!(&ctr).value_of(), FNSYMBOLS))
  });

  Ok(())
}
