//! Red/green guards for batch 65s: the AMS bindings provide the internals the real packages define and that
//! documents call from their own macros — amsmath's `\bBigg@`/`\big@size`, `\Let@` with `\math@cr`,
//! `\restore@math@cr` and `\default@tag`, `\maketag@@@`, `\ltx@label`, `\@mathmeasure`, `\start@align`,
//! `\intertext@`, `\mdots@`, `\ints@a`, `\spread@equation`; amscd's `\CDat`/`\atdef@`/`\CD@check`; amsfonts'
//! `\newsymbol` — and amscd reads `@ >>>` as amscd.sty does, skipping the spaces before the connector. Repros
//! under `tools/perfect_kernel/repros/ams-internals/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/ams-internals/",
      $name,
      ".tex"
    ))
  };
}

/// The XML of `tex` converted under ar5iv, which must give no error and exactly `warnings` warnings.
fn convert(tex: &str, warnings: usize) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), warnings, "{log}");
  xml
}

/// amsmath.sty:721-739: `\bBigg@{<factor>}<delim>` is the delimiter `<factor>\big@size` tall; its factors 1, 1.5, 2,
/// 2.5 are `\big` … `\Bigg` (here `\bBigg@\tw@[` is `\bigg[`), and a document's `\vast` = `\bBigg@{4}` is larger
/// (witnesses 1412.0919, 2508.13749, 2103.12630). Repro ams-internals/amsmath_bbigg_scaled_delimiter.
#[test]
fn amsmath_bbigg_scales_the_delimiter() {
  let xml = convert(repro!("amsmath_bbigg_scaled_delimiter"), 0);
  for (open, close, size, count) in [("(", ")", "410%", 1), ("[", "]", "210%", 2)] {
    let open_tok =
      format!(r#"<XMTok fontsize="{size}" role="OPEN" stretchy="false">{open}</XMTok>"#);
    let close_tok =
      format!(r#"<XMTok fontsize="{size}" role="CLOSE" stretchy="false">{close}</XMTok>"#);
    assert_eq!(xml.matches(&open_tok).count(), count, "{xml}");
    assert_eq!(xml.matches(&close_tok).count(), count, "{xml}");
  }
  assert!(xml.contains(r#"tex="\bBigg@{4}(x\bBigg@{4})="#), "{xml}");
}

/// amsmath.sty:1176-1186, :1205-1209: a document's `\subalign` sets `\\` to amsmath's `\math@cr` (`\Let@`) before
/// its `\ialign`, so the rows split there (witnesses 2103.14974, 2103.16537, 2103.07815). Repro
/// ams-internals/amsmath_subalign_let_math_cr.
#[test]
fn amsmath_let_math_cr_splits_the_rows() {
  let xml = convert(repro!("amsmath_subalign_let_math_cr"), 0);
  assert_element(
    &xml,
    "XMArray",
    &[r#"xml:id="S0.Ex1.m1.1.1.1.3""#],
    r#"<XMArray vattach="bottom" xml:id="S0.Ex1.m1.1.1.1.3">
                <XMRow xml:id="S0.Ex1.m1.1.1.1.3.1">
                  <XMCell align="right" class="ltx_nopad_r" xml:id="S0.Ex1.m1.1.1.1.3.1.1">
                    <XMTok font="italic" fontsize="49%" role="UNKNOWN">i</XMTok>
                  </XMCell>
                  <XMCell align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="S0.Ex1.m1.1.1.1.3.1.2">
                    <XMTok fontsize="49%" meaning="less-than" role="RELOP">&lt;</XMTok>
                    <XMTok font="italic" fontsize="49%" role="UNKNOWN">n</XMTok>
                  </XMCell>
                </XMRow>
                <XMRow xml:id="S0.Ex1.m1.1.1.1.3.2">
                  <XMCell align="right" class="ltx_nopad_r" xml:id="S0.Ex1.m1.1.1.1.3.2.1">
                    <XMTok font="italic" fontsize="49%" role="UNKNOWN">j</XMTok>
                  </XMCell>
                  <XMCell align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="S0.Ex1.m1.1.1.1.3.2.2">
                    <XMTok fontsize="49%" meaning="less-than" role="RELOP">&lt;</XMTok>
                    <XMTok font="italic" fontsize="49%" role="UNKNOWN">m</XMTok>
                  </XMCell>
                </XMRow>
              </XMArray>"#,
  );
}

/// amsmath.sty:149, :497-509, :1210-1211, :1228, :1724: `\maketag@@@` sets a tag in a box, `\ltx@label` is
/// `\label`, `\start@align\@ne\st@rredfalse\m@ne` starts a numbered `{align}`, and `\mdots@` is the math `\dots`
/// (witnesses 1109.5733, 1904.06459, 2008.12422, 2103.10644, math0012138). Repro
/// ams-internals/amsmath_author_internals.
#[test]
fn amsmath_author_called_internals() {
  let xml = convert(repro!("amsmath_author_internals"), 0);
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">Tag <text xml:id="p1.1.1">[A]</text>.</p>"#,
  );
  assert!(
    xml.contains(r#"<equation labels="LABEL:e2" xml:id="S0.E1">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<equationgroup class="ltx_eqn_align" xml:id="S0.EGx2">"#),
    "{xml}"
  );
  assert!(
    xml.contains(r#"<equation labels="LABEL:e3" xml:id="S0.E3">"#),
    "{xml}"
  );
  assert!(xml.contains(r#"<equation xml:id="S0.E4">"#), "{xml}");
  assert_eq!(
    xml
      .matches(r#"<XMTok name="dots" role="ID">⋯</XMTok>"#)
      .count(),
    1,
    "{xml}"
  );
}

/// `markup` without its ` name="…"` attributes for each of `names`.
fn without_attributes(markup: &str, names: &[&str]) -> String {
  let mut out = markup.to_string();
  for name in names {
    let opener = format!(" {name}=\"");
    while let Some(start) = out.find(&opener) {
      let value = start + opener.len();
      let end = out[value..].find('"').expect("closed attribute") + value + 1;
      out.replace_range(start..end, "");
    }
  }
  out
}

/// The `<equation>` elements of `xml`, without their ids and `tex`/`text` attributes.
fn equations_without_ids(xml: &str) -> Vec<String> {
  xml
    .split("<equation ")
    .skip(1)
    .map(|eq| {
      // The leading space lets the element's first attribute (its `xml:id`) be matched as the others are.
      without_attributes(
        &format!(
          " {}",
          &eq[..eq.find("</equation>").expect("closed equation")]
        ),
        &["xml:id", "idref", "tex", "text"],
      )
    })
    .collect()
}

/// amscd.sty:56-62 reads the connector after `@` as an undelimited argument (`\at@@#1`), so `A @ >f>> B`, `@ VVV`,
/// `@ |`, `@ =` and `@ .` are the connectors without the space; Perl's `\cd@ Token` took the space (KNOWN_PERL_ERRORS
/// #631; witnesses 1403.4347, 2112.10343, 1304.1461). Repro ams-internals/amscd_space_before_connector.
#[test]
fn amscd_skips_spaces_before_the_connector() {
  let xml = convert(repro!("amscd_space_before_connector"), 0);
  let equations = equations_without_ids(&xml);
  assert_eq!(equations.len(), 2, "{xml}");
  assert_eq!(equations[0], equations[1], "{xml}");
  assert_eq!(
    xml.matches(r#"meaning="rightarrow" role="ARROW""#).count(),
    2,
    "{xml}"
  );
  assert_eq!(
    xml.matches(r#"meaning="leftarrow" role="ARROW""#).count(),
    2,
    "{xml}"
  );
}

/// amscd.sty:43-55, :87-97: `\atdef@` defines a document's own connector (written with `\CD@check` and `&&`), and
/// `\CDat` makes `@` read connectors outside `{CD}`, where `\ampersand@` is nothing (witnesses 1408.6469,
/// 1002.0255, math0101071, 0709.3322). Repro ams-internals/amscd_cdat_atdef.
#[test]
fn amscd_cdat_and_atdef() {
  let xml = convert(repro!("amscd_cdat_atdef"), 0);
  assert_eq!(
    xml
      .matches(r#"<XMTok class="ltx_horizontally_stretchy" meaning="rightarrow" role="ARROW" stretchy="true" width="30.0pt">→</XMTok>"#)
      .count(),
    3,
    "{xml}"
  );
  assert!(
    xml.contains(r#"text="X * rightarrow ^ alpha * Y""#),
    "{xml}"
  );
}

/// amsfonts.sty:117-138: `\newsymbol\cs <font><class><hex><hex>` declares an msam/msbm symbol, with amsfonts'
/// obsolete-command warning (witnesses 1302.6049, 2112.15557). Repro ams-internals/amsfonts_newsymbol.
#[test]
fn amsfonts_newsymbol_declares_the_symbol() {
  let xml = convert(repro!("amsfonts_newsymbol"), 2);
  assert_eq!(
    xml
      .matches(r#"<XMTok name="mylneqq" role="RELOP">≨</XMTok>"#)
      .count(),
    1,
    "{xml}"
  );
  assert_eq!(
    xml
      .matches(r#"<XMTok name="mylesssim" role="RELOP">≲</XMTok>"#)
      .count(),
    1,
    "{xml}"
  );
}
