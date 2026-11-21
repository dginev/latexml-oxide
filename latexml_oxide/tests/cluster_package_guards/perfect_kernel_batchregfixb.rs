//! REGFIXB (v0.8.0 per-paper check, the largest regression cluster): an `\halign` that `\lastbox` took is a box
//! (tex.web §1080), so put back in a paragraph or a box it ends none. Its `before_construct` closed the enclosing
//! `ltx:p`, the table left the `\vphantom` it was copied into, and the box capture around it closed out of order —
//! easytable/easybmat's `\@easy@buildbox` peel, reached since 68q's `\unvbox` splice (34adb486865): ~250 arXiv papers
//! (1705.10408, 2002.06044, 2109.00258, 2106.13444, 1710.10243).
use latexml::util::test::{kpse_has, xml_element};

use super::{
  perfect_kernel_batch46::{convert, error_count, warning_count},
  perfect_kernel_batch57::{RAW, assert_elements},
};

fn para(id: &'static str, markup: &'static str) -> (&'static str, &'static str, &'static str) {
  ("para", id, markup)
}

/// boxes-groups/lastbox_halign_is_a_box: the `\halign` taken straight off its box's list (p1) and off a list
/// `\unvbox` spliced (p2-p6), then copied into a `\vphantom` in a `\vbox` (p1, p2; the two Errors), into bold text in
/// a paragraph of a `\vbox` (p3: the cell keeps the font it was set in), into a math `\vbox`'s phantom (p5), and put
/// by `\box` on a `\vbox`'s vertical list (p6).
#[test]
fn lastbox_halign_is_a_box() {
  assert_elements(
    include_str!("../../../tools/perfect_kernel/repros/boxes-groups/lastbox_halign_is_a_box.tex"),
    RAW,
    (0, 0),
    &[
      para(
        "p1",
        r##"<para xml:id="p1"><p>[h]<inline-block vattach="bottom"><p><text class="ltx_phantom"><tabular><tr><td align="left" class="ltx_nopad_l ltx_nopad_r">a</td></tr></tabular></text></p></inline-block>One.</p></para>"##,
      ),
      para(
        "p2",
        r##"<para xml:id="p2"><p><inline-block><p><text class="ltx_phantom"><tabular vattach="bottom"><tr><td align="left" class="ltx_nopad_l ltx_nopad_r">b</td></tr></tabular></text></p></inline-block>Two.</p></para>"##,
      ),
      para(
        "p3",
        r##"<para vattach="bottom" xml:id="p3"><p><text font="bold">x<tabular vattach="bottom"><tr><td align="left" class="ltx_nopad_l ltx_nopad_r"><text font="medium">c</text></td></tr></tabular></text></p></para>"##,
      ),
      para("p4", r##"<para xml:id="p4"><p>Three.</p></para>"##),
      para(
        "p5",
        r##"<para xml:id="p5"><p><Math mode="inline" tex="\left(\vbox{\vphantom{\halign{#\cr d\cr}}}\right)" text="Array[[[d]]]" xml:id="p5.m1"><XMath><XMDual><XMRef idref="p5.m1.1"/><XMWrap><XMTok role="OPEN" stretchy="true">(</XMTok><XMArray class="ltx_phantom" vattach="bottom" xml:id="p5.m1.1"><XMRow><XMCell align="left" class="ltx_nopad_l ltx_nopad_r"><XMText>d</XMText></XMCell></XMRow></XMArray><XMTok role="CLOSE" stretchy="true">)</XMTok></XMWrap></XMDual></XMath></Math> Four.</p></para>"##,
      ),
      para(
        "p6",
        r##"<para xml:id="p6"><tabular vattach="bottom"><tr><td align="left" class="ltx_nopad_l ltx_nopad_r">e</td></tr></tabular></para>"##,
      ),
      para("p7", r##"<para xml:id="p7"><p>Five.</p></para>"##),
    ],
  );
}

/// package-bindings/easytable_easybmat_lastbox_peel: the real packages (the witnesses' shapes, a `{TAB}` and a
/// `{BMAT}` in math): each table once in its cell's `\vphantom` (hidden, whole) and once shown, in order.
#[test]
fn easytable_easybmat_lastbox_peel() {
  if !kpse_has("easytable.sty") || !kpse_has("easybmat.sty") {
    eprintln!("SKIP: easytable.sty or easybmat.sty is not in this TeX Live");
    return;
  }
  let (log, xml) = convert(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/easytable_easybmat_lastbox_peel.tex"
    ),
    true,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // the ids aside: the elements compared are the tables and their cells
  let mut xml = xml;
  while let Some(at) = xml.find(r#" xml:id=""#) {
    let end = xml[at + 9..].find('"').map_or(xml.len(), |e| at + 10 + e);
    xml.replace_range(at..end, "");
  }
  let cell = r##"<td align="left" class="ltx_nopad_l ltx_nopad_r">"##;
  let math = |x: &str| {
    format!(
      r##"{cell}<Math mode="inline" tex="\displaystyle{{{x}}}" text="{x}"><XMath><XMTok font="italic" role="UNKNOWN">{x}</XMTok></XMath></Math></td>"##
    )
  };
  let phantoms: Vec<String> = xml
    .match_indices(r##"<text class="ltx_phantom""##)
    .filter_map(|(at, _)| xml_element(&xml[at..], "text", &[r##"class="ltx_phantom""##]))
    .collect();
  assert_eq!(
    phantoms,
    [
      format!(
        r##"<text class="ltx_phantom"><tabular vattach="bottom"><tr>{cell}a</td>{cell}b</td></tr><tr>{cell}c</td>{cell}d</td></tr></tabular></text>"##
      ),
      format!(
        r##"<text class="ltx_phantom"><tabular vattach="bottom"><tr>{}{}</tr><tr>{}{}</tr></tabular></text>"##,
        math("e"),
        math("f"),
        math("g"),
        math("h")
      ),
    ],
    "{xml}"
  );
  // every cell in document order: the phantom's table, then the shown one
  let cells: Vec<&str> = xml
    .split("<td ")
    .skip(1)
    .map(|td| {
      let body = &td[td.find('>').map_or(0, |i| i + 1)..];
      match body.find(r#" text=""#) {
        Some(at) if body.trim_start().starts_with("<Math") => {
          body[at + 7..].split('"').next().unwrap_or("")
        },
        _ => body.split('<').next().unwrap_or("").trim(),
      }
    })
    .collect();
  assert_eq!(
    cells,
    [
      "a", "b", "c", "d", "a", "b", "c", "d", "e", "f", "g", "h", "e", "f", "g", "h"
    ],
    "{xml}"
  );
}
