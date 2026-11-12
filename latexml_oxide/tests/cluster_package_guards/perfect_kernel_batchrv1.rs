//! Guards for RV1: a definer reverts to the user-level macro, never a `\lx@…` internal (QA41 audit, BM Q3.0). A Perl
//! `sub { (); }` body is a closure returning nothing, not an undef body's empty box; an undef body's box reverts its
//! arguments (Primitive.pm:74-76); an internal constructor carries its macro's `alias`. One table: each repro converted
//! once in-process, its exact Error and Warning counts, and every `<Math>` element whole whose `tex=` names `select`.

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batchcr2::elements,
};

macro_rules! repro {
  ($name:literal) => {
    (
      $name,
      include_str!(concat!(
        "../../../tools/perfect_kernel/repros/reversion/",
        $name,
        ".tex"
      )),
    )
  };
}

/// (repro, Errors, Warnings, the `tex=` substring selecting the Math elements, those elements whole in order).
type Row = (
  (&'static str, &'static str),
  usize,
  usize,
  &'static str,
  &'static [&'static str],
);

const ROWS: &[Row] = &[
  // tikz-cd: each node's Math is the cell as written (1301.4725, 1305.4700, 1504.00121).
  (
    repro!("inpgf_ignorespaces_tikzcd_node"),
    0,
    0,
    "tex=\"{",
    &[
      "<Math mode=\"inline\" tex=\"{X}\" text=\"X\" xml:id=\"p1.pic1.m1\">\n                      <XMath xml:id=\"p1.pic1.m1.1\">\n                        <XMTok font=\"italic\" role=\"UNKNOWN\">X</XMTok>\n                      </XMath>\n                    </Math>",
      "<Math mode=\"inline\" tex=\"{Y^{\\prime}}\" text=\"Y ^ prime\" xml:id=\"p1.pic1.m2\">\n                      <XMath xml:id=\"p1.pic1.m2.1\">\n                        <XMApp xml:id=\"p1.pic1.m2.1.1\">\n                          <XMTok role=\"SUPERSCRIPTOP\" scriptpos=\"post2\"/>\n                          <XMTok font=\"italic\" role=\"UNKNOWN\">Y</XMTok>\n                          <XMTok fontsize=\"70%\" name=\"prime\" role=\"SUPOP\">′</XMTok>\n                        </XMApp>\n                      </XMath>\n                    </Math>",
    ],
  ),
  // A pgf `matrix of math nodes`: every cell; a node's own `$x+y$` was already clean.
  (
    repro!("inpgf_ignorespaces_pgf_matrix_of_math_nodes"),
    0,
    0,
    "tex=\"",
    &[
      "<Math mode=\"inline\" tex=\"{a_{1}}\" text=\"a _ 1\" xml:id=\"p1.pic1.m1\">\n                      <XMath xml:id=\"p1.pic1.m1.1\">\n                        <XMApp xml:id=\"p1.pic1.m1.1.1\">\n                          <XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post2\"/>\n                          <XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok>\n                          <XMTok fontsize=\"70%\" meaning=\"1\" role=\"NUMBER\">1</XMTok>\n                        </XMApp>\n                      </XMath>\n                    </Math>",
      "<Math mode=\"inline\" tex=\"{b}\" text=\"b\" xml:id=\"p1.pic1.m2\">\n                      <XMath xml:id=\"p1.pic1.m2.1\">\n                        <XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok>\n                      </XMath>\n                    </Math>",
      "<Math mode=\"inline\" tex=\"{c}\" text=\"c\" xml:id=\"p1.pic1.m3\">\n                      <XMath xml:id=\"p1.pic1.m3.1\">\n                        <XMTok font=\"italic\" role=\"UNKNOWN\">c</XMTok>\n                      </XMath>\n                    </Math>",
      "<Math mode=\"inline\" tex=\"{d^{2}}\" text=\"d ^ 2\" xml:id=\"p1.pic1.m4\">\n                      <XMath xml:id=\"p1.pic1.m4.1\">\n                        <XMApp xml:id=\"p1.pic1.m4.1.1\">\n                          <XMTok role=\"SUPERSCRIPTOP\" scriptpos=\"post2\"/>\n                          <XMTok font=\"italic\" role=\"UNKNOWN\">d</XMTok>\n                          <XMTok fontsize=\"70%\" meaning=\"2\" role=\"NUMBER\">2</XMTok>\n                        </XMApp>\n                      </XMath>\n                    </Math>",
      "<Math mode=\"inline\" tex=\"x+y\" text=\"x + y\" xml:id=\"p1.pic1.m5\">\n                <XMath xml:id=\"p1.pic1.m5.1\">\n                  <XMApp xml:id=\"p1.pic1.m5.1.1\">\n                    <XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok>\n                    <XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok>\n                    <XMTok font=\"italic\" role=\"UNKNOWN\">y</XMTok>\n                  </XMApp>\n                </XMath>\n              </Math>",
    ],
  ),
  // `\centering`/`\raggedright`/`\raggedleft` (and ragged2e's `\Centering`) in math, as Perl's constructors revert
  // (1312.5886, 2008.09739, 2107.14385).
  (
    repro!("aligning_switch_in_math_reverts_to_the_users_macro"),
    0,
    0,
    "@add@",
    &[
      "<Math mode=\"display\" tex=\"\\centering x=1\\@add@centering\" text=\"x = 1\" xml:id=\"S0.E1.m1\">\n        <XMath xml:id=\"S0.E1.m1.1\">\n          <XMApp xml:id=\"S0.E1.m1.1.1\">\n            <XMTok align=\"center\" meaning=\"equals\" role=\"RELOP\">=</XMTok>\n            <XMTok align=\"center\" font=\"italic\" role=\"UNKNOWN\">x</XMTok>\n            <XMTok align=\"center\" meaning=\"1\" role=\"NUMBER\">1</XMTok>\n          </XMApp>\n        </XMath>\n      </Math>",
      "<Math mode=\"display\" tex=\"\\raggedright y=2\\@add@raggedright\" text=\"y = 2\" xml:id=\"S0.E2.m1\">\n        <XMath xml:id=\"S0.E2.m1.1\">\n          <XMApp xml:id=\"S0.E2.m1.1.1\">\n            <XMTok class=\"ltx_align_left\" meaning=\"equals\" role=\"RELOP\">=</XMTok>\n            <XMTok class=\"ltx_align_left\" font=\"italic\" role=\"UNKNOWN\">y</XMTok>\n            <XMTok class=\"ltx_align_left\" meaning=\"2\" role=\"NUMBER\">2</XMTok>\n          </XMApp>\n        </XMath>\n      </Math>",
      "<Math mode=\"display\" tex=\"\\raggedleft z=3\\@add@raggedleft\" text=\"z = 3\" xml:id=\"S0.E3.m1\">\n        <XMath xml:id=\"S0.E3.m1.1\">\n          <XMApp xml:id=\"S0.E3.m1.1.1\">\n            <XMTok class=\"ltx_align_right\" meaning=\"equals\" role=\"RELOP\">=</XMTok>\n            <XMTok class=\"ltx_align_right\" font=\"italic\" role=\"UNKNOWN\">z</XMTok>\n            <XMTok class=\"ltx_align_right\" meaning=\"3\" role=\"NUMBER\">3</XMTok>\n          </XMApp>\n        </XMath>\n      </Math>",
      "<Math mode=\"display\" tex=\"\\centering w=4\\@add@centering\" text=\"w = 4\" xml:id=\"S0.E4.m1\">\n        <XMath xml:id=\"S0.E4.m1.1\">\n          <XMApp xml:id=\"S0.E4.m1.1.1\">\n            <XMTok align=\"center\" meaning=\"equals\" role=\"RELOP\">=</XMTok>\n            <XMTok align=\"center\" font=\"italic\" role=\"UNKNOWN\">w</XMTok>\n            <XMTok align=\"center\" meaning=\"4\" role=\"NUMBER\">4</XMTok>\n          </XMApp>\n        </XMath>\n      </Math>",
      "<Math mode=\"inline\" tex=\"a\\centering b\\@add@centering\" text=\"a * b\" xml:id=\"p1.m1\">\n        <XMath xml:id=\"p1.m1.1\">\n          <XMApp xml:id=\"p1.m1.1.1\">\n            <XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok>\n            <XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok>\n            <XMTok align=\"center\" font=\"italic\" role=\"UNKNOWN\">b</XMTok>\n          </XMApp>\n        </XMath>\n      </Math>",
    ],
  ),
  // `\shortintertext` inside `aligned` errs as pdflatex's 2 `!`; its row reverts to `\intertext` (KNOWN_PERL_ERRORS
  // #1227; 1301.0750, 1906.05085, 2107.14777).
  (
    repro!("intertext_in_aligned_reverts_to_intertext"),
    2,
    0,
    "intertext",
    &[
      "<Math tex=\"\\displaystyle\\intertext{and}\" text=\"[and]\" xml:id=\"S0.E1Xa.m1\">\n            <XMath xml:id=\"S0.E1Xa.m1.1\">\n              <p class=\"ltx_intertext\" xml:id=\"S0.E1Xa.m1.1.1\">and</p>\n            </XMath>\n          </Math>",
      "<Math tex=\"\\displaystyle\\intertext{and}\" text=\"[and]\" xml:id=\"S0.E1Xa.m2\">\n                <XMath xml:id=\"S0.E1Xa.m2.1\">\n                  <p class=\"ltx_intertext\" xml:id=\"S0.E1Xa.m2.1.1\">and</p>\n                </XMath>\n              </Math>",
      "<Math tex=\"\\displaystyle\\intertext{or}g=h,\" text=\"[or] * g = h\" xml:id=\"S0.E2Xa.m1\">\n            <XMath xml:id=\"S0.E2Xa.m1.2\">\n              <XMDual xml:id=\"S0.E2Xa.m1.2.1\">\n                <XMRef idref=\"S0.E2Xa.m1.1\" xml:id=\"S0.E2Xa.m1.2.1.1\"/>\n                <XMWrap xml:id=\"S0.E2Xa.m1.2.1.2\">\n                  <XMApp xml:id=\"S0.E2Xa.m1.1\">\n                    <XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok>\n                    <XMApp xml:id=\"S0.E2Xa.m1.1.2\">\n                      <XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok>\n                      <p class=\"ltx_intertext\" xml:id=\"S0.E2Xa.m1.1.2.2\">or</p>\n                      <XMTok font=\"italic\" role=\"UNKNOWN\">g</XMTok>\n                    </XMApp>\n                    <XMTok font=\"italic\" role=\"UNKNOWN\">h</XMTok>\n                  </XMApp>\n                  <XMTok role=\"PUNCT\">,</XMTok>\n                </XMWrap>\n              </XMDual>\n            </XMath>\n          </Math>",
      "<Math tex=\"\\displaystyle\\intertext{or}g\" text=\"[or] * g\" xml:id=\"S0.E2Xa.m2\">\n                <XMath xml:id=\"S0.E2Xa.m2.1\">\n                  <XMApp xml:id=\"S0.E2Xa.m2.1.1\">\n                    <XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok>\n                    <p class=\"ltx_intertext\" xml:id=\"S0.E2Xa.m2.1.1.2\">or</p>\n                    <XMTok font=\"italic\" role=\"UNKNOWN\">g</XMTok>\n                  </XMApp>\n                </XMath>\n              </Math>",
    ],
  ),
  // bibunits' `\cite` in math: its `\lx@bibunits@…` primitives leave no box.
  (
    repro!("bibunits_cite_in_math_reverts_to_cite"),
    0,
    0,
    "cite",
    &[
      "<Math mode=\"inline\" tex=\"x_{\\cite[cite]{[\\@@bibref{}{foo}{}{}]}}\" text=\"x _ [[]]\" xml:id=\"p1.m1\">\n        <XMath xml:id=\"p1.m1.1\">\n          <XMApp xml:id=\"p1.m1.1.1\">\n            <XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/>\n            <XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok>\n            <XMText xml:id=\"p1.m1.1.1.3\"><cite class=\"ltx_citemacro_cite\"><text fontsize=\"70%\" xml:id=\"p1.m1.1.1.3.1\">[</text><bibref bibrefs=\"foo\" class=\"ltx_cite_sort\" inlist=\"bu1\" separator=\",\" yyseparator=\",\"/><text fontsize=\"70%\" xml:id=\"p1.m1.1.1.3.2\">]</text></cite></XMText>\n          </XMApp>\n        </XMath>\n      </Math>",
      "<Math mode=\"inline\" tex=\"y_{\\cite[cite]{[\\@@bibref{}{foo}{}{}]}}\" text=\"y _ [[]]\" xml:id=\"p1.m2\">\n        <XMath xml:id=\"p1.m2.1\">\n          <XMApp xml:id=\"p1.m2.1.1\">\n            <XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/>\n            <XMTok font=\"italic\" role=\"UNKNOWN\">y</XMTok>\n            <XMText xml:id=\"p1.m2.1.1.3\"><cite class=\"ltx_citemacro_cite\"><text fontsize=\"70%\" xml:id=\"p1.m2.1.1.3.1\">[</text><bibref bibrefs=\"foo\" class=\"ltx_cite_sort\" inlist=\"bibliography bu1\" separator=\",\" yyseparator=\",\"/><text fontsize=\"70%\" xml:id=\"p1.m2.1.1.3.2\">]</text></cite></XMText>\n          </XMApp>\n        </XMath>\n      </Math>",
    ],
  ),
  // An undef body's box reverts its argument (`\mark{x}`); `\extrafloats`, latex.ltx's macro, leaves none.
  (
    repro!("undef_primitive_reverts_its_arguments"),
    0,
    0,
    "tex=\"",
    &[
      "<Math mode=\"inline\" tex=\"a\\markright{x}b\" text=\"a * b\" xml:id=\"p1.m1\">\n        <XMath xml:id=\"p1.m1.1\">\n          <XMApp xml:id=\"p1.m1.1.1\">\n            <XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok>\n            <XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok>\n            <XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok>\n          </XMApp>\n        </XMath>\n      </Math>",
      "<Math mode=\"inline\" tex=\"cd\" text=\"c * d\" xml:id=\"p1.m2\">\n        <XMath xml:id=\"p1.m2.1\">\n          <XMApp xml:id=\"p1.m2.1.1\">\n            <XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok>\n            <XMTok font=\"italic\" role=\"UNKNOWN\">c</XMTok>\n            <XMTok font=\"italic\" role=\"UNKNOWN\">d</XMTok>\n          </XMApp>\n        </XMath>\n      </Math>",
    ],
  ),
];

#[test]
fn batchrv1_reversion_rows() {
  let mut failures = Vec::new();
  for &((name, tex), errors, warnings, select, expected) in ROWS {
    let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
    let got: Vec<String> = elements(&xml, "Math")
      .into_iter()
      .filter(|math| math.contains(select))
      .collect();
    if (error_count(&log), warning_count(&log)) != (errors, warnings) || got != expected {
      failures.push(format!(
        "{name}: E/W {:?} (expected {:?})\n{got:#?}\n{log}",
        (error_count(&log), warning_count(&log)),
        (errors, warnings)
      ));
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
