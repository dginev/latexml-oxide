//! Red/green guards for batch REGFIXN1, four regressions of the v0.8.0 per-paper check: deluxetable's `\enddata` met by
//! an AASTeX `D` cell's expanding reader, revtex's hyperref preload leaving iftex loaded, `\robustify` of a binding's
//! `\protected` macro, and `\@testopt`'s peek past spaces.
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// class-bindings/aastex_decimals_last_row_at_enddata: after `\decimals` the last row of a `D` column ends at
/// `\enddata` without `\\` (1712.01997, 2104.10697, 2107.00240, 2201.08360, 2308.10484, 2606.18355). The cell's reader
/// expands up to the cell's end, so `\enddata`'s flag reset must be a primitive: `\global\lx@deluxetable@datafalse`
/// had the `\let`'s `\iflx@deluxetable@data` operand expanded into an open conditional (REGFIXJ3 2 errors). Distinct
/// from `perfect_kernel_batch68y::aastex_decimals_*`, rows that end with `\\`.
#[test]
fn aastex_decimals_last_row_at_enddata() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/class-bindings/aastex_decimals_last_row_at_enddata.tex"
  );
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert!(!log.contains("Fatal:"), "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "tbody",
    &[],
    r##"<tbody><tr xml:id="S0.T1.2.2"><td align="left" border="t" xml:id="S0.T1.2.2.1">x</td><td align="right" border="t" class="ltx_norightpad" xml:id="S0.T1.2.2.2"><Math mode="inline" tex="1" text="1" xml:id="S0.T1.m1"><XMath xml:id="S0.T1.m1.1"><XMTok meaning="1" role="NUMBER">1</XMTok></XMath></Math></td><td align="left" border="t" class="ltx_noleftpad" xml:id="S0.T1.2.2.3">.<Math mode="inline" tex="5" text="5" xml:id="S0.T1.m2"><XMath xml:id="S0.T1.m2.1"><XMTok meaning="5" role="NUMBER">5</XMTok></XMath></Math></td></tr><tr xml:id="S0.T1.2.3"><td align="left" border="b" xml:id="S0.T1.2.3.1">y</td><td align="right" border="b" class="ltx_norightpad" xml:id="S0.T1.2.3.2"><Math mode="inline" tex="2" text="2" xml:id="S0.T1.m3"><XMath xml:id="S0.T1.m3.1"><XMTok meaning="2" role="NUMBER">2</XMTok></XMath></Math></td><td align="left" border="b" class="ltx_noleftpad" xml:id="S0.T1.2.3.3">.<Math mode="inline" tex="25" text="25" xml:id="S0.T1.m4"><XMath xml:id="S0.T1.m4.1"><XMTok meaning="25" role="NUMBER">25</XMTok></XMath></Math></td></tr></tbody>"##,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text after.</p></para>"#,
  );
}

/// package-bindings/revtex_hyperref_after_ifpdf_relax: `\let\ifpdf\relax` before the document's hyperref under revtex4-1
/// (1808.09428, 1812.07558, 1904.08912, 1906.04729). The class loads no iftex, so in LaTeX the hyperref load reads
/// iftex.sty, which sets `\ifpdf` again; the binding's preload left iftex marked loaded and the driver test read
/// `\relax` (REGFIXJ3 2 errors). Distinct from the phfnote `hyperrefdefs=defer` guards: the packages the preload loaded,
/// not hyperref's own reload.
#[test]
fn revtex_hyperref_after_ifpdf_relax() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/revtex_hyperref_after_ifpdf_relax.tex"
  );
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert!(!log.contains("Fatal:"), "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">A word.</p></para>"#,
  );
}

/// package-bindings/robustify_a_protected_binding_macro: `\robustify\underline` (2407.21515, 2506.16228, 2511.14592).
/// The binding's `\underline` is a `\protected` macro, robust already, as etoolbox's rebuild of a protected macro is
/// the same macro; the rebuild from its `\meaning` failed (REGFIXJ3 3 errors). A user's `\protected\long` macro stays
/// as defined. Distinct from `perfect_kernel_batchclustfix3::robustify_of_a_native_binding` (closures, constructors)
/// and `perfect_kernel_batch69b::robustify_ltx_protected_native_cite` (`\protect\cite␣`).
#[test]
fn robustify_a_protected_binding_macro() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/package-bindings/robustify_a_protected_binding_macro.tex"
  );
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert!(!log.contains("Fatal:"), "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">A <text class="ltx_underline" xml:id="p1.1.1">word</text> here [a].</p>"#,
  );
}

/// expansion-primitives/testopt_skips_spaces: latex.ltx:1259 `\@testopt` peeks by `\kernel@ifnextchar[`, past spaces
/// (`\DeclareCaptionSubType * [alph]{table}`: 2012.09877, 2102.01489, 2108.10505, 2407.20995, 2601.08142). REGFIXJ3
/// took the space for no option: caption's "No float type '[' defined" and `[alph]table` as text, `Y=[d],[[]`; and
/// past spaces with no `[` the default is inserted, the spaces gone (`Y=[d],[f]`). Perl the same, KNOWN_PERL_ERRORS
/// #1303.
#[test]
fn testopt_skips_spaces() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/expansion-primitives/testopt_skips_spaces.tex"
  );
  let (log, xml) = convert_with(tex, Some("ar5iv.sty"));
  assert!(!log.contains("Fatal:"), "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let shown: Vec<&str> = log.lines().filter(|line| line.starts_with("Y=")).collect();
  assert_eq!(
    shown,
    ["Y=[a],[b]", "Y=[a],[b]", "Y=[c],[e]", "Y=[d],[f]"],
    "{log}"
  );
  assert!(!xml.contains("<para"), "{xml}");
  assert_element(
    &xml,
    "caption",
    &[],
    r#"<caption><tag close=": ">Table 1</tag>T</caption>"#,
  );
}
