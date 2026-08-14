//! Merged font-map / character-decode guards.
//!
//! Auto-consolidated test binary: each former file is an inline `mod`
//! below, body preserved verbatim, merged into one link unit for CI
//! economy. All members are subprocess- or few-conversion tests, so
//! co-locating them in one process stays far under the RSS fuse.

mod cluster;

mod caret_charcode {
  //! `` `^^@ `` (backtick + caret-notation char) must read the code point, like Perl.
  //!
  //! Root cause (found while triaging 2111.00584 → xintexpr/xinttrig): Rust set
  //! NUL's (`\^^@`, U+0000) DEFAULT catcode to 9 (IGNORE, per the TeXbook),
  //! whereas Perl LaTeXML uses 12 (OTHER). With IGNORE, the `^^@`-notation char
  //! is *dropped* during tokenization, so the alphabetic constant `` `^^@ ``
  //! skipped to the next token (e.g. `\relax`) and returned its code (114)
  //! instead of 0. xint's `\romannumeral`&&@` expansion idiom (`&&@` is `^^@`
  //! with `&` at catcode 7) relies on `` `^^@ `` == 0.
  //!
  //! Fix: NUL default catcode → 12 (OTHER), matching Perl. An explicit
  //! `\catcode`^^Q=9` is still honored (only the default changes); stray raw NUL
  //! bytes become harmless OTHER chars stripped at XML serialization (no bogus
  //! `\uninger`-style CS, no invalid-XML NUL in output).
  //!
  //! Dump-independent.
  use latexml::util::test::convert_fixture;

  #[test]
  fn backtick_caret_notation_reads_charcode() {
    let r = convert_fixture("tests/cluster_regressions/caret_charcode.tex");
    let out = r.result.expect("conversion produced no result");
    let xml = out.to_string();

    assert!(
      xml.contains("value is [0]"),
      "`` `^^@ `` must read code 0 (got output without `[0]`): NUL default catcode \
       regressed to IGNORE? — relevant excerpt: {:?}",
      xml.split("value is").nth(1).map(|s| &s[..s.len().min(40)])
    );
    assert!(
      xml.contains("Second [1]"),
      "`` `^^A `` must read code 1 (^^A == char 1)"
    );
  }
}

mod line_fontmap {
  //! Picture-mode `line10` chars must have nonzero width, or LaTeX-2.09
  //! `\@sline`-style drawing loops never terminate.
  //!
  //! Root cause of the canvas_3 OOM cluster (math0102053, math0102089,
  //! math0212126, math0504436, math0506088, math0604321): plain-TeX papers
  //! inline LaTeX 2.09 picture mode, whose `\@whiledim` loop advances by the
  //! width of an `\hbox{\@linefnt\@getlinechar(x,y)}`. Real TeX gets nonzero
  //! widths (2.5–10pt) from `line10.tfm`; without a fontmap, `FontDecode`
  //! dropped the char → empty box → 0pt → unbounded box accumulation (~1.9M
  //! boxes / 4.5GB RSS). Perl LaTeXML ships no `line` fontmap and OOMs the same
  //! way; `line_fontmap.rs`/`lcircle_fontmap.rs` fix this at the root through
  //! the architecture's own mechanism (a surpass-Perl reliability fix with no
  //! control-flow divergence). Modern latex.ltx guards this exact hazard
  //! (`\ifdim\wd\@linechar=\z@ \setbox\@linechar\hbox{.}\@badlinearg\fi`) but
  //! these old documents inline the unguarded 2.09 macros.
  //!
  //! Dump-independent (plain-TeX input; the fontmap binding is compiled in).
  use latexml::util::test::convert_fixture;

  #[test]
  fn line_font_chars_have_nonzero_width_and_loops_terminate() {
    let r = convert_fixture("tests/cluster_regressions/line_font_picture.tex");

    let out = r
      .result
      .unwrap_or_else(|| {
        panic!(
          "conversion produced no result (status_code={}) — the \\@whiledim line \
           loop likely ran away again",
          r.status_code
        )
      })
      .to_string();
    assert!(
      out.contains("LINEWIDTH-OK") && !out.contains("LINEWIDTH-ZERO"),
      "a line10 \\char box measured 0pt wide — the `line` fontmap regressed \
       (zero width re-opens the \\@sline infinite-loop OOM cluster)"
    );
    assert!(
      out.contains("LOOP-DONE"),
      "the \\whiledim drawing loop did not complete"
    );
    assert!(
      !r.log.contains("PushbackLimit")
        && !r.log.contains("runaway")
        && !r.log.contains("Infinite digestion loop"),
      "a runaway guard fired — the drawing loop is no longer terminating"
    );
  }
}

mod ding_family_fontmap {
  //! A font selected by FAMILY must still decode through that family's fontmap.
  //!
  //! `bbding.sty` reaches its glyphs by family, not by encoding: `\dingfamily`
  //! is `\fontencoding{U}\fontfamily{ding}\selectfont`, and neither Perl nor
  //! Rust ships a `u.fontmap`. Perl's `\selectfont` handles this with an
  //! explicit hack (latex_constructs.pool.ltxml L5207-5209) — if the family is
  //! not a known typeface, try `LoadFontMap($family)` and, on success,
  //! `MergeFont(encoding => $family)`. Rust was missing that branch, so every
  //! `\@chooseSymbol{N}` fell through to the OT1 fallback and emitted OT1 slot
  //! N's TEXT character: `\XSolidBrush` (`'045`) became a literal `%` and
  //! `\Checkmark` (`'041`) became `!`. `ding_fontmap.rs` was dead code —
  //! nothing ever set the `ding` encoding.
  //!
  //! Witness 2503.04421 (ICLR submission): 28 cells across its two main
  //! results tables — the "pretrained?" column of both — rendered `%`/`!`
  //! instead of ✗/✓, inverting the tables' meaning for a reader. The paper
  //! converted with status 0 and zero `Error:` lines, so nothing but a
  //! fidelity check catches this.
  //!
  //! The fixture spells out `\fontencoding{U}\fontfamily{ding}\selectfont`
  //! rather than loading `bbding.sty`, because `bbding` ships in
  //! `texlive-fonts-extra`, which CI deliberately does not install — a fixture
  //! that quietly lost its package would assert nothing. (It did: the first
  //! version of this test went red on CI with an empty `Marks:` paragraph and
  //! `Warning:missing_file:bbding`, while passing locally.) The expansion is
  //! verbatim what `\dingfamily` produces, and the engine reads no `.fd` file
  //! for it — the `ding` fontmap is compiled in — so the raw form needs no TeX
  //! Live package at all.
  //!
  //! Golden glyphs verified against Perl LaTeXML 0.8.8 on the same host.
  //! Dump-independent (the fontmap bindings are compiled in).
  use latexml::util::test::convert_fixture;

  /// The text of the first paragraph starting with `label`, up to the next tag.
  fn paragraph_text(out: &str, label: &str) -> String {
    let start = out
      .find(label)
      .unwrap_or_else(|| panic!("no {label:?} paragraph in the conversion output"))
      + label.len();
    let rest = &out[start..];
    let end = rest.find('<').unwrap_or(rest.len());
    rest[..end].trim().to_string()
  }

  #[test]
  fn ding_family_glyphs_decode_through_the_family_fontmap() {
    let r = convert_fixture("tests/fonts/ding_family_fontmap.tex");
    let out = r
      .result
      .unwrap_or_else(|| {
        panic!(
          "conversion produced no result (status_code={})",
          r.status_code
        )
      })
      .to_string();

    // \Checkmark \CheckmarkBold \XSolid \XSolidBold \XSolidBrush
    // = ding slots '041 '042 '043 '044 '045 = U+2713..U+2717.
    // Pre-fix this read "! \" # $ %" — OT1 slots 33..37.
    assert_eq!(
      paragraph_text(&out, "Marks:"),
      "\u{2713} \u{2714} \u{2715} \u{2716} \u{2717}",
      "bbding marks did not decode through ding.fontmap — the \\selectfont \
       family-as-encoding branch regressed, and \\XSolidBrush is silently a \
       literal OT1 character again (witness 2503.04421)"
    );

    // Slots far from the ASCII range, so a fallback cannot coincidentally match:
    // \ScissorRight \Phone \Envelope \HandRight \PencilRight = '001 '010 '014 '021 '027.
    assert_eq!(
      paragraph_text(&out, "Slots:"),
      "\u{2702} \u{260E} \u{2709} \u{1F599} \u{270F}",
      "low ding slots did not decode through ding.fontmap"
    );

    // The family-derived encoding must survive a nested font switch: the
    // witness reaches these glyphs from inside \textbf{} in table cells, and a
    // `MergeFont` that the inner switch discards would decode to OT1 again.
    // Asserted on the surrounding text, not on element names, so the check
    // does not also pin \textbf/\emph markup.
    for (context, glyph) in [("bold", '\u{2713}'), ("italic", '\u{2717}')] {
      let expected = format!("{context} {glyph}");
      assert!(
        out.contains(&expected),
        "expected {expected:?} — ding glyph U+{:04X} was lost inside a nested \
         font switch",
        glyph as u32
      );
    }
  }
}

mod char_font_decode {
  //! `\char` must decode with Perl's `FontDecode` semantics — including in math.
  //!
  //! Perl's `\char` calls `FontDecode` (`TeX_Character.pool.ltxml` L32-36), which
  //! defaults the encoding to OT1 when the current font carries none:
  //! `$encoding = $font->getEncoding || 'OT1'` (`Package.pm` L2877). Its sibling
  //! `FontDecodeString` (`Package.pm` L2906) deliberately has **no** such
  //! default, and `font::decode_str` is the port of that one — so calling it for
  //! `\char` inherited the wrong half of a deliberate Perl asymmetry.
  //!
  //! That only bites in math mode, because `Font::math_default()` sets
  //! `encoding: None` on purpose: `$\char65$` produced the empty string where
  //! Perl produces `A`. Text mode was unaffected, which is why it survived.
  //!
  //! Second defect in the same line: the code was cast `as u8`, so an
  //! out-of-range `\char300` wrapped onto slot 300 & 0xFF = 44 and printed `,`.
  //! Perl guards `$code < 0` and then indexes the map, so out-of-range yields no
  //! glyph at all.
  //!
  //! All expectations verified against same-host Perl LaTeXML 0.8.8, which
  //! renders this fixture as `MATH[A] TEXT[A]`, `OOBMATH[] OOBTEXT[]`,
  //! `SYMBOL[A] ACCENT[´]`.
  use latexml::util::test::convert_fixture;

  /// The text between `NAME[` and its `]`, with all markup stripped.
  fn marker(out: &str, name: &str) -> String {
    let open = format!("{name}[");
    let start = out
      .find(&open)
      .unwrap_or_else(|| panic!("marker {name}[ absent from the conversion output"))
      + open.len();
    let rest = &out[start..];
    let end = rest
      .find(']')
      .unwrap_or_else(|| panic!("marker {name}[ is never closed"));
    // Strip tags: math lands in <math>…</math> wrappers, text does not.
    let inner = regex_lite_strip_tags(&rest[..end]);
    inner.split_whitespace().collect::<Vec<_>>().join(" ")
  }

  /// Minimal tag stripper — avoids pulling a regex dep into a test.
  fn regex_lite_strip_tags(s: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for c in s.chars() {
      match c {
        '<' => depth += 1,
        '>' => depth = depth.saturating_sub(1),
        _ if depth == 0 => out.push(c),
        _ => {},
      }
    }
    out
  }

  #[test]
  fn char_decodes_through_ot1_in_math_and_does_not_wrap_out_of_range() {
    let r = convert_fixture("tests/cluster_regressions/char_font_decode.tex");
    let out = r
      .result
      .unwrap_or_else(|| {
        panic!(
          "conversion produced no result (status_code={})",
          r.status_code
        )
      })
      .to_string();

    // The regression: math mode fell through to an empty encoding.
    assert_eq!(
      marker(&out, "MATH"),
      "A",
      "`$\\char65$` did not decode — `\\char` is using FontDecodeString \
       semantics again (no OT1 default) and silently drops the glyph in math, \
       where Font::math_default() leaves the encoding unset"
    );
    assert_eq!(marker(&out, "TEXT"), "A", "`\\char65` broke in text mode");
    assert_eq!(
      marker(&out, "SYMBOL"),
      "A",
      "`$\\symbol{{65}}$` did not decode — \\symbol routes through \\char"
    );

    // Out of range must yield NOTHING, not a wrapped slot. `,` is the specific
    // pre-fix answer (300 & 0xFF == 44), so name it in the message.
    for m in ["OOBMATH", "OOBTEXT"] {
      let got = marker(&out, m);
      assert!(
        got.is_empty(),
        "`\\char300` produced {got:?} instead of nothing — the code is being \
         truncated to u8 again (300 & 0xFF = 44 = `,`)"
      );
    }

    // A slot whose OT1 entry is an accent still routes through decode_str, which
    // is what carries Perl's multi-char map entries (NBSP-prefixed combining
    // marks, T1's "SS"). Pins that the fix did not bypass that wrapper.
    assert_eq!(
      marker(&out, "ACCENT"),
      "\u{00B4}",
      "`\\char19` lost its OT1 acute accent — the multi-char/combining-mark \
       handling in font::decode_str is being bypassed"
    );
  }

  /// `\DeclareSymbolFont`'s encoding argument must be expanded before storage.
  ///
  /// Perl reads it as `ExpandedPartially` (latex_constructs.pool.ltxml L2664)
  /// because `fontmath.ltx` and most font packages write
  /// `\DeclareSymbolFont{operators}{\encodingdefault}{\rmdefault}{m}{n}`. Read
  /// unexpanded, the literal `\encodingdefault` is stored as the encoding and
  /// every dependent `\DeclareMathSymbol` looks up a fontmap of that name,
  /// finds none, and falls back to the raw code — silently, with no diagnostic.
  ///
  /// Verified against same-host Perl 0.8.8, which yields `A` and `Γ`.
  #[test]
  fn symbol_font_encoding_argument_is_expanded_before_storage() {
    let r = convert_fixture("tests/cluster_regressions/symbolfont_encoding_expansion.tex");
    let out = r
      .result
      .unwrap_or_else(|| {
        panic!(
          "conversion produced no result (status_code={})",
          r.status_code
        )
      })
      .to_string();

    // Slot 65 is a decoy: an un-decoded raw code 65 IS ASCII `A`, so it looks
    // right either way. Asserted anyway so a regression that breaks BOTH slots
    // is distinguishable from one that breaks only the non-ASCII path.
    assert_eq!(
      marker(&out, "DECOY"),
      "A",
      "slot 65 of the declared symbol font did not decode"
    );

    // Slot 0 is the tell: `Γ` through OT1, U+FFFD when the encoding was stored
    // as the unexpanded string `\encodingdefault`.
    assert_eq!(
      marker(&out, "TELL"),
      "\u{0393}",
      "slot 0 decoded to something other than `Γ` — `\\DeclareSymbolFont` is \
       storing its encoding argument unexpanded again, so the fontmap lookup \
       misses and the raw code leaks through (pre-fix this was U+FFFD)"
    );
  }
}

mod declaremathalphabet_props {
  //! `\DeclareMathAlphabet` maps NFSS codes to abstract font properties.
  //!
  //! Perl runs its three arguments through `lookupTeXFont`
  //! (`Common/Font.pm` L230-239) — the same family/series/shape tables
  //! `\selectfont` consults — before storing them (`latex_constructs.pool.ltxml`
  //! L2677-2687). Rust stored the raw NFSS codes, so
  //! `\DeclareMathAlphabet{\mysf}{OT1}{cmss}{m}{n}` emitted `font="cmss m n"`
  //! where Perl emits `font="sansserif"`, and the MathML then carried
  //! `mathvariant="normal"` for every declared alphabet — a sansserif, bold or
  //! italic alphabet rendered upright.
  //!
  //! `font::lookup_tex_font` was already a faithful port of `lookupTeXFont`
  //! with **no callers** — the same dead-helper shape as `ding_fontmap.rs`
  //! before the `\selectfont` fix.
  //!
  //! Blast radius is document- and package-declared alphabets only: the stock
  //! `\mathsf`/`\mathbf`/`\mathit`/`\mathrm` are bound directly in the pool and
  //! are unaffected (asserted below, so a future change cannot quietly route
  //! them through the broken path). Verified against same-host Perl 0.8.8,
  //! which emits `sansserif`, `bold`, `italic` for the declared trio.
  use crate::cluster::convert_to_xml;

  /// The `font=` attribute of every `<XMTok>`, in document order.
  fn tok_fonts(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    for seg in xml.split("<XMTok").skip(1) {
      let Some(end) = seg.find('>') else { continue };
      let tag = &seg[..end];
      if let Some(i) = tag.find("font=\"") {
        let rest = &tag[i + 6..];
        if let Some(j) = rest.find('"') {
          out.push(rest[..j].to_string());
        }
      }
    }
    out
  }

  #[test]
  fn declared_math_alphabets_carry_abstract_font_properties() {
    let xml = convert_to_xml("tests/cluster_regressions/declaremathalphabet_props.tex");
    assert_eq!(
      tok_fonts(&xml),
      vec!["sansserif", "bold", "italic"],
      "\\DeclareMathAlphabet stored raw NFSS codes again — Perl maps them \
       through lookupTeXFont, and the raw form leaks into the XML as e.g. \
       `font=\"cmss m n\"`, which costs the alphabet its MathML variant"
    );
  }

  #[test]
  fn stock_math_alphabets_are_unaffected() {
    // \mathsf/\mathbf/\mathit are bound directly in the pool, so they were
    // already correct — pinned so a future change cannot quietly reroute them
    // through the declaration path this test's sibling protects.
    let xml = convert_to_xml("tests/cluster_regressions/stock_math_alphabets.tex");
    assert_eq!(
      tok_fonts(&xml),
      vec!["sansserif", "bold", "italic"],
      "a stock math alphabet lost its abstract font property"
    );
  }
}

mod amsb_multichar_slots {
  //! A fontmap slot holding TWO characters must survive into the output.
  //!
  //! Perl fontmaps may store a multi-character string in a slot — the six `msbm`
  //! negated relations are base + U+0338 COMBINING LONG SOLIDUS OVERLAY
  //! (`amsb.fontmap.ltxml` L20, L21, L23). Rust's slot is a single
  //! `Option<char>`, so such entries live in a companion
  //! `DeclareFontMapMultichar!` table that `font::decode_str` consults first.
  //! `AMSb` had no such table, so these six slots decoded to the EMPTY STRING.
  //!
  //! Scope, checked rather than assumed: the familiar spellings `\nleqslant`,
  //! `\ngeqslant`, `\nleqq`, `\ngeqq`, `\nsubseteqq`, `\nsupseteqq` are bound by
  //! `amssymb` directly to explicit Unicode and never reach the fontmap — they
  //! were correct all along and are NOT a guard for this. Only direct slot
  //! access under the encoding exercises it.
  //!
  //! Expectations verified against same-host Perl LaTeXML 0.8.8.
  use crate::cluster::convert_to_xml;

  /// Text inside `NAME[...]`, markup stripped, as codepoints.
  fn marker_chars(xml: &str, name: &str) -> Vec<u32> {
    let open = format!("{name}[");
    let start = xml
      .find(&open)
      .unwrap_or_else(|| panic!("marker {name}[ absent"))
      + open.len();
    let rest = &xml[start..];
    let end = rest
      .find(']')
      .unwrap_or_else(|| panic!("marker {name}[ unclosed"));
    let mut out = String::new();
    let mut depth = 0usize;
    for c in rest[..end].chars() {
      match c {
        '<' => depth += 1,
        '>' => depth = depth.saturating_sub(1),
        _ if depth == 0 => out.push(c),
        _ => {},
      }
    }
    out.chars().map(|c| c as u32).collect()
  }

  #[test]
  fn amsb_multichar_slots_keep_their_combining_overlay() {
    let xml = convert_to_xml("tests/cluster_regressions/amsb_multichar_slots.tex");

    // Control first: a single-char slot on the same path. If this fails the
    // whole decode path broke, not the multichar table.
    assert_eq!(
      marker_chars(&xml, "OK"),
      vec![0x1D538],
      "the AMSb single-char path itself regressed (slot 65 should be 𝔸)"
    );

    for (marker, base) in [
      ("S10", 0x2A7Du32),
      ("S11", 0x2A7E),
      ("S20", 0x2266),
      ("S21", 0x2267),
      ("S34", 0x2AC5),
      ("S35", 0x2AC6),
    ] {
      assert_eq!(
        marker_chars(&xml, marker),
        vec![base, 0x0338],
        "AMSb slot behind {marker} lost its U+0338 overlay — the \
         DeclareFontMapMultichar! table is not being consulted, and the slot \
         decodes to the empty string (Perl gives the pair)"
      );
    }
  }
}

mod multichar_slot_paths {
  //! Every decode path must honour a multi-character fontmap slot.
  //!
  //! Perl's `FontDecode` returns `$$map[$code]` whole, so a slot holding two
  //! characters just works on every path. Rust splits the representation: the
  //! array slot is a single `Option<char>` and multi-char entries live in a
  //! `_fontmap_multichar` side table. Only `font::decode_str` consults that
  //! table, so any caller of the single-`char` `font::decode` silently drops the
  //! second character.
  //!
  //! T2B slot 128 is U+04F6 + U+0336 COMBINING LONG STROKE OVERLAY. Dropping the
  //! overlay leaves U+04F6 — a *different* letter.
  //!
  //! There was also an ORDERING defect: `lookup_multichar_override` read the
  //! table from state but ran *before* `decode`, which is what triggers the map
  //! load. `\DeclareTextSymbol` decodes in the preamble and bakes the result
  //! into a primitive body, so it hit the table before it existed and froze the
  //! stripped letter for the whole document.
  //!
  //! Expectations verified against same-host Perl LaTeXML 0.8.8.
  use crate::cluster::convert_to_xml;

  fn marker_chars(xml: &str, name: &str) -> Vec<u32> {
    let open = format!("{name}[");
    let start = xml
      .find(&open)
      .unwrap_or_else(|| panic!("marker {name}[ absent"))
      + open.len();
    let rest = &xml[start..];
    let end = rest
      .find(']')
      .unwrap_or_else(|| panic!("marker {name}[ unclosed"));
    let mut out = String::new();
    let mut depth = 0usize;
    for c in rest[..end].chars() {
      match c {
        '<' => depth += 1,
        '>' => depth = depth.saturating_sub(1),
        _ if depth == 0 => out.push(c),
        _ => {},
      }
    }
    // Drop layout whitespace; only the glyph matters.
    out
      .chars()
      .map(|c| c as u32)
      .filter(|c| *c != 0x0A && *c != 0x20)
      .collect()
  }

  #[test]
  fn declare_text_symbol_keeps_a_multichar_slot() {
    let xml = convert_to_xml("tests/cluster_regressions/multichar_slot_paths.tex");
    assert_eq!(
      marker_chars(&xml, "TS"),
      vec![0x04F6, 0x0336],
      "\\DeclareTextSymbol dropped the U+0336 overlay — either it is back on \
       font::decode instead of decode_str, or the multichar table is again being \
       consulted before the fontmap binding is loaded (it decodes in the \
       preamble, so it must preload the map itself)"
    );
  }

  #[test]
  fn symbol_keeps_a_multichar_slot() {
    // Control: the same slot at document time, when the map is already loaded.
    // Worked before the fix; a failure here means the whole path broke.
    let xml = convert_to_xml("tests/cluster_regressions/multichar_slot_paths.tex");
    assert_eq!(
      marker_chars(&xml, "SYM"),
      vec![0x04F6, 0x0336],
      "\\symbol lost a multichar slot"
    );
  }

  /// KNOWN REMAINING GAP — pinned, not endorsed.
  ///
  /// `\DeclareMathSymbol` routes through `mathchar.rs`, which calls the
  /// single-`char` `font::decode` and stores the result in a
  /// `glyph: Option<char>` field. Carrying a pair there needs that field to
  /// become a string — a real type change through the math-char pipeline, so it
  /// is deliberately out of scope here rather than done badly.
  ///
  /// Perl gives `[U+04F6, U+0336]`. When this is fixed, THIS TEST WILL FAIL —
  /// that is the intent. Update it to the Perl value and delete this note.
  #[test]
  fn declare_math_symbol_still_drops_the_overlay_known_gap() {
    let xml = convert_to_xml("tests/cluster_regressions/multichar_slot_paths.tex");
    assert_eq!(
      marker_chars(&xml, "MS"),
      vec![0x04F6],
      "the math-char decode path changed — if it now yields [0x04F6, 0x0336] \
       this gap is FIXED: update this test to expect the pair and remove it from \
       the known-gap list"
    );
  }
}
