// plain_bootstrap — Bootstrap code for reading plain.tex for LaTeXML.
// Corresponds to Perl Engine/plain_bootstrap.pool.ltxml.
//
// Loaded BEFORE the plain dump. Contains stubs that override plain.tex's
// own allocation mechanisms with LaTeXML's versions.
use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // INITEX letter/digit char-code initialization. Per plain.tex L112-113
  // INITEX (the initial TeX engine) sets `\uccode\x=\X`, `\uccode\X=\X`,
  // `\lccode\x=\x`, `\lccode\X=\x` for every letter; mathcodes for
  // digits & letters per plain.tex Table 17.2; sfcode 999 for letters.
  // Previously this loop lived only in `plain_base.rs` (NODUMP path).
  // With `plain_dump` loaded instead, the initialization was missing —
  // the dump-build snapshot is taken BEFORE plain.tex's raw load, and
  // plain.tex itself doesn't (re)assign these codes (it assumes INITEX
  // has). So with the dump active, `\uccode` for letters returned 0 and
  // `\MakeUppercase` produced lowercase output. Mirror INITEX here in
  // `plain_bootstrap` (which runs in BOTH paths AND before the
  // dump-build snapshot) — the resulting codes land in the snapshot
  // baseline and don't need to round-trip through the dump diff.
  // NOTE: mathcodes (digits and letters) are deliberately NOT set here.
  // plain_base.rs sets them via 0x7100+letter / 0x7030+digit (class 7
  // variable family). With the dump path active these mathcodes route
  // input through `decode_math_char`, which preempts the `DefMath`-
  // based per-letter handlers in base_xmath that produce the expected
  // unicode-substituted output (e.g. `\mathbb{A}` → 𝔸, `\mathbb{0}` →
  // 𝟘). Without a full Stored::Font roundtrip in the dump these paths
  // diverge — 22_fonts::bbold_test regressed when letter/digit
  // mathcodes were eagerly set in this bootstrap.
  // lccode/uccode/sfcode for letters DO need this initialization so
  // `\MakeUppercase` (which consults uccode, not mathcode) works
  // outside math mode in the dump path.
  for letter in b'A'..=b'Z' {
    assign_lccode(letter, letter + 32, Some(Scope::Global));
    assign_uccode(letter, letter, Some(Scope::Global));
    assign_sfcode(letter as char, 999u16, Some(Scope::Global));

    assign_lccode(letter + 32, letter + 32, Some(Scope::Global));
    assign_uccode(letter + 32, letter, Some(Scope::Global));
  }

  // Perl: plain_bootstrap.pool.ltxml L19-27 — CSS-based TeX logo
  DefConstructor!("\\TeX", "<ltx:text class='ltx_TeX_logo' cssstyle='letter-spacing:-0.2em; margin-right:0.2em'
  >T<ltx:text cssstyle='font-variant:small-caps;font-size:120%;' yoffset='-0.2ex'
  >e</ltx:text>X</ltx:text>",
    locked => true,
    enter_horizontal => true,
    sizer => sub[_whatsit] { Ok((Dimension!("1.9em"), Dimension!("1.6ex"), Dimension!("0.5ex"))) });

  //======================================================================
  // Perl: plain_bootstrap.pool.ltxml L32-33
  // Use LaTeXML's register allocation to avoid allocating same slot twice
  DefMacro!("\\alloc@{}{}{}{}{}", r"\lx@alloc@{#2}{\count1#1}{#3}{#5}", locked => true);
  DefMacro!("\\ch@ck{}{}{}", None, locked => true);

  //======================================================================
  // Perl: plain_bootstrap.pool.ltxml L37-40
  //   DefPrimitive('\newif DefToken', sub { DefConditionalI($cs, undef); }, locked => 1);
  // Use LaTeXML's conditional machinery. State side-effect → DefPrimitive
  // (stomach level), not DefMacro (expansion level).
  DefPrimitive!("\\newif DefToken", sub[(cs)] {
    def_conditional(cs, None, None, ConditionalOptions::default())?;
  }, locked => true);

  //======================================================================
  // Perl: plain_bootstrap.pool.ltxml L43
  DefPrimitive!("\\leavevmode", { enter_horizontal(); });
});
