//! Red/green guards for batch 68q: `\unvbox`/`\unvcopy` put a vertical box's list back (tex.web §1110 `unpackage`), so
//! a `\lastbox` peel loop takes the box's lines one by one and ends (§1080) — semantic.sty's `\inference` looped for
//! ever and ran ~315 arXiv papers out of memory (1603.01635, 2506.05832; KNOWN_PERL_ERRORS #840).
//!
//! Every repro is checked against pdflatex (its header holds the oracle).
use super::perfect_kernel_batch57::{RAW, assert_elements};

fn para(id: &'static str, markup: &'static str) -> (&'static str, &'static str, &'static str) {
  ("para", id, markup)
}

/// boxes-groups/unvbox_lastbox_peel_loop: each round `\setbox\pb=\vbox{\unvbox\pb \global\setbox\lb=\lastbox}` takes
/// the last line and leaves the rest; the box's end finishes its last paragraph (§1086), so what follows the spliced
/// lines starts a line of its own. p1 waits for interline glue (R7: pdflatex `[1:h,2]`).
#[test]
fn unvbox_lastbox_peel_loop() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/unvbox_lastbox_peel_loop.tex"),
    RAW,
    (0, 0),
    &[
      para("p2", r##"<para xml:id="p2"><p>[1:h,2:h,3]</p></para>"##),
      para("p3", r##"<para xml:id="p3"><p>[1:h,2:h,3]</p></para>"##),
      para("p4", r##"<para xml:id="p4"><p>[1]</p></para>"##),
      para("p5", r##"<para xml:id="p5"><p>[1:v,2:h,3]</p></para>"##),
      para("p6", r##"<para xml:id="p6"><p>[L2/full]</p></para>"##),
      para(
        "p7",
        r##"<para xml:id="p7"><p>Para one.</p><p>Para two.</p><p>More.</p></para>"##,
      ),
      para(
        "p8",
        r##"<para xml:id="p8"><p>Para one.</p><p>Para two.</p><p>X</p></para>"##,
      ),
      para(
        "p9",
        r##"<para xml:id="p9"><p><inline-block class="ltx_parbox" vattach="middle" width="85.4pt"><p>P1</p><p>P2</p><p>PAfter.</p></inline-block></p></para>"##,
      ),
    ],
  );
}

/// package-bindings/semantic_inference_premises: infernce.sty's `\@adjustPremises` peels the premise lines and
/// resets each to the widest; the rule is a fraction of the premises over the conclusion. `\hskip 7.5pt` (`\@@pEnd`)
/// is set as an en space and a four-per-em space (U+2002, U+2005).
#[test]
fn semantic_inference_premises() {
  assert_elements(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/semantic_inference_premises.tex"
    ),
    RAW,
    (0, 0),
    &[
      // One premise over one conclusion.
      para(
        "p1",
        "<para xml:id=\"p1\"><p><text class=\"ltx_markedasmath\"><Math mode=\"inline\" tex=\"\\frac{\\vtop{\\hbox to20.28589pt{\\hskip 7.5pt$a$\\hskip 7.5pt}\\vskip 1.59998pt}}{\\vbox{\\vskip 1.59998pt\\hbox{\\hskip 7.5pt$b$\\hskip 7.5pt}}}\" text=\"[\u{2002}\u{2005}a] / [\u{2002}\u{2005}b]\" xml:id=\"p1.m1.m1\"><XMath><XMApp><XMTok mathstyle=\"text\" meaning=\"divide\" role=\"FRACOP\"/><XMText vattach=\"top\" width=\"20.3pt\">\u{2002}\u{2005}<Math mode=\"inline\" tex=\"a\" text=\"a\" xml:id=\"p1.m1.m1.m1\"><XMath><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok></XMath></Math></XMText><XMText vattach=\"bottom\">\u{2002}\u{2005}<Math mode=\"inline\" tex=\"b\" text=\"b\" xml:id=\"p1.m1.m1.m2\"><XMath><XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok></XMath></Math></XMText></XMApp></XMath></Math></text></p></para>",
      ),
      // `p & q \\ r`: the two premise lines in order, each reset to the widest (39.85417pt), over the conclusion.
      para(
        "p2",
        "<para xml:id=\"p2\"><p><text class=\"ltx_markedasmath\"><Math mode=\"inline\" tex=\"\\frac{\\vtop{\\hbox to39.85417pt{\\hskip 7.5pt$p$\\hskip 15.00002pt$q$\\hskip 7.5pt}\\hbox to39.85417pt{\\hskip 7.5pt$r$\\hskip 7.5pt}\\vskip 1.59998pt}}{\\vbox{\\vskip 1.59998pt\\hbox{\\hskip 7.5pt$s$\\hskip 7.5pt}}}\" text=\"[\u{2002}\u{2005}p\u{2003}\u{2002}q\u{2002}\u{2005}r] / [\u{2002}\u{2005}s]\" xml:id=\"p2.m1.m1\"><XMath><XMApp><XMTok mathstyle=\"text\" meaning=\"divide\" role=\"FRACOP\"/><XMText><inline-block vattach=\"top\"><p width=\"39.9pt\">\u{2002}\u{2005}<Math mode=\"inline\" tex=\"p\" text=\"p\" xml:id=\"p2.m1.m1.m1\"><XMath><XMTok font=\"italic\" role=\"UNKNOWN\">p</XMTok></XMath></Math>\u{2003}\u{2002}<Math mode=\"inline\" tex=\"q\" text=\"q\" xml:id=\"p2.m1.m1.m2\"><XMath><XMTok font=\"italic\" role=\"UNKNOWN\">q</XMTok></XMath></Math></p><p width=\"39.9pt\">\u{2002}\u{2005}<Math mode=\"inline\" tex=\"r\" text=\"r\" xml:id=\"p2.m1.m1.m3\"><XMath><XMTok font=\"italic\" role=\"UNKNOWN\">r</XMTok></XMath></Math></p></inline-block></XMText><XMText vattach=\"bottom\">\u{2002}\u{2005}<Math mode=\"inline\" tex=\"s\" text=\"s\" xml:id=\"p2.m1.m1.m4\"><XMath><XMTok font=\"italic\" role=\"UNKNOWN\">s</XMTok></XMath></Math></XMText></XMApp></XMath></Math></text></p></para>",
      ),
    ],
  );
}
