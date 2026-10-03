//! The TeX box family — `\box`, `\copy`, `\lastbox`, `\vsplit`, `\unhbox`, `\unvbox`, `\unhcopy`, `\unvcopy` and box
//! sizes — as one collection (user 2026-10-02): each repro in tools/perfect_kernel/repros/boxes-groups/box_primitives_*
//! holds one case per paragraph, checked against pdflatex, its header naming every case's status and root (design
//! agent_reports/2026-10-02_box_family_design.md, R1-R6). The tests assert the GREEN cases as whole elements; a RED
//! case joins when its root is fixed.

use super::perfect_kernel_batch57::{RAW, assert_elements};

fn case(id: &'static str, markup: &'static str) -> (&'static str, &'static str, &'static str) {
  ("para", id, markup)
}

/// `\lastbox` (tex.web §1080) and the list-tail registers (§424).
#[test]
fn lastbox() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/box_primitives_lastbox.tex"),
    RAW,
    (0, 0),
    &[
      case(
        "p1",
        r##"<para xml:id="p1"><p>Kept line.Next paragraph.</p></para>"##,
      ),
      case(
        "p2",
        r##"<para xml:id="p2"><p>Also kept.</p><p>Inner.</p></para>"##,
      ),
      case("p3", r##"<para xml:id="p3"><p>[47.28606pt]</p></para>"##),
      case("p8", r##"<para xml:id="p8"><p>[1]</p></para>"##),
      case("p9", r##"<para xml:id="p9"><p>[0]</p></para>"##),
      case("p10", r##"<para xml:id="p10"><p>[0.0pt]</p></para>"##),
      case("p11", r##"<para xml:id="p11"><p>[0.0pt] done.</p></para>"##),
      case("p12", r##"<para xml:id="p12"><p>[h]</p></para>"##),
      case("p13", r##"<para xml:id="p13"><p>[void]</p></para>"##),
      case("p14", r##"<para xml:id="p14"><p>[3.0pt]</p></para>"##),
      case("p15", r##"<para xml:id="p15"><p>[void]</p></para>"##),
      case("p16", r##"<para xml:id="p16"><p>[void]</p></para>"##),
      case("p18", r##"<para xml:id="p18"><p>[3.0pt]</p></para>"##),
      case("p19", r##"<para xml:id="p19"><p>[0.0pt]</p></para>"##),
      case("p20", r##"<para xml:id="p20"><p>[other]</p></para>"##),
      case("p21", r##"<para xml:id="p21"><p>[void/xy]</p></para>"##),
      case(
        "p22",
        r##"<para xml:id="p22"><p>[void/<Math mode="inline" tex="x" text="x" xml:id="p22.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math>]</p></para>"##,
      ),
      case("p23", r##"<para xml:id="p23"><p>[void]</p></para>"##),
      case("p24", r##"<para xml:id="p24"><p>[other]</p></para>"##),
      case(
        "p27",
        r##"<para xml:id="p27"><p>[other,5.2778pt]</p></para>"##,
      ),
      case("p28", r##"<para xml:id="p28"><p>[other]</p></para>"##),
      case("p29", r##"<para xml:id="p29"><p>[void]</p></para>"##),
      case(
        "p31",
        r##"<para xml:id="p31"><p>[6.83331pt,7.08336pt]</p></para>"##,
      ),
      case("p32", r##"<para xml:id="p32"><p>[void]</p></para>"##),
    ],
  );
}

/// `\vsplit` (tex.web §968-977).
#[test]
fn vsplit() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/box_primitives_vsplit.tex"),
    RAW,
    (0, 0),
    &[
      case(
        "p3",
        r##"<para class="ltx_noindent" xml:id="p3"><p vattach="bottom">Three</p></para>"##,
      ),
      case("p4", r##"<para xml:id="p4"><p>[5.0pt,10.0pt]</p></para>"##),
      case(
        "p5",
        r##"<para vattach="top" xml:id="p5"><p>Alpha</p></para>"##,
      ),
      case(
        "p6",
        r##"<para xml:id="p6"><p>Beta</p><p>Gamma</p></para>"##,
      ),
      case("p7", r##"<para xml:id="p7"><p>[1]</p></para>"##),
      case(
        "p9",
        r##"<para xml:id="p9"><p>F1</p><p>F2</p><p>F3</p></para>"##,
      ),
      case(
        "p10",
        r##"<para vattach="bottom" xml:id="p10"><p>N3</p></para>"##,
      ),
      case(
        "p11",
        r##"<para xml:id="p11"><p>[20.0pt,5.0pt,10.0pt]</p></para>"##,
      ),
      case(
        "p12",
        r##"<para xml:id="p12"><p>[24.0pt,22.0pt]</p></para>"##,
      ),
      case(
        "p13",
        r##"<para xml:id="p13"><p>[15.0pt,22.0pt]</p></para>"##,
      ),
      case(
        "p14",
        r##"<para xml:id="p14"><p>[5.0pt,1.0pt,10.0pt]</p></para>"##,
      ),
      case(
        "p15",
        r##"<para xml:id="p15"><p>[10.0pt,5.0pt]</p></para>"##,
      ),
      case(
        "p16",
        r##"<para xml:id="p16"><p>[22.0pt,0.0pt]</p></para>"##,
      ),
      case(
        "p17",
        r##"<para xml:id="p17"><p>[10.0pt,10.0pt]</p></para>"##,
      ),
      case(
        "p21",
        r##"<para xml:id="p21"><p>[10.0pt,32.0pt]</p></para>"##,
      ),
      case("p22", r##"<para xml:id="p22"><p>[6.83331pt]</p></para>"##),
      case("p23", r##"<para xml:id="p23"><p>[6.83331pt]</p></para>"##),
      case("p25", r##"<para xml:id="p25"><p>[22.0pt]</p></para>"##),
      case("p26", r##"<para xml:id="p26"><p>[22.0pt]</p></para>"##),
      case("p29", r##"<para xml:id="p29"><p>[28.33344pt]</p></para>"##),
    ],
  );
}

/// The unpacking primitives (§1110) and box sizes (§1083, `\vtop` §1087).
#[test]
fn unpack() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/box_primitives_unpack.tex"),
    RAW,
    (0, 0),
    &[
      case("p3", r##"<para xml:id="p3"><p>[ab]</p></para>"##),
      case("p5", r##"<para xml:id="p5"><p>[1]</p></para>"##),
      case(
        "p6",
        r##"<para xml:id="p6"><p>[50.0pt,6.83331pt,full]</p></para>"##,
      ),
      case(
        "p10",
        r##"<para xml:id="p10"><p>[6.83331pt,43.16669pt]</p></para>"##,
      ),
      case(
        "p11",
        r##"<para xml:id="p11"><p>[6.83331pt,23.94444pt]</p></para>"##,
      ),
      case(
        "p12",
        r##"<para xml:id="p12"><p>[28.83331pt,1.94444pt]</p></para>"##,
      ),
      case(
        "p13",
        r##"<para xml:id="p13"><p>[0.0pt,11.83331pt]</p></para>"##,
      ),
      case(
        "p14",
        r##"<para xml:id="p14"><p>[0.0pt,20.77776pt]</p></para>"##,
      ),
      case(
        "p17",
        r##"<para xml:id="p17"><p>[0.0pt,6.83331pt]</p></para>"##,
      ),
      case("p18", r##"<para xml:id="p18"><p>[12.0pt]</p></para>"##),
      case(
        "p19",
        r##"<para xml:id="p19"><p>[0.0pt,12.0pt]</p></para>"##,
      ),
      case("p20", r##"<para xml:id="p20"><p>[5.2778pt]</p></para>"##),
      case(
        "p23",
        r##"<para xml:id="p23"><p>[11.91666pt,6.91666pt]</p></para>"##,
      ),
    ],
  );
}
