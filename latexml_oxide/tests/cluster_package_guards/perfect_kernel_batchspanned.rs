//! Guards for SPANNED: a math array's cell covered by a `rowspan` above has one XML layout, the table builder's — it is
//! omitted (Perl Alignment.pm:336) — so post's MathML emits every `XMCell` and skips none. `\bordermatrix` builds its
//! fence columns to match (OD #1171); Perl's post `@spanned` (MathML.pm:448-470) drops real cells (KNOWN_PERL_ERRORS
//! #1347). Every repro is checked against pdflatex (its header holds the oracle).
use latexml::{
  post::{PostOptions, run_post_processing_logged},
  util::test::{convert_with_then, error_count, warning_count},
};

/// Each `<mtable>` of `html` as its rows, each `<mtd>` as `attrs|text` (its attributes, then its text with the tags
/// stripped and whitespace dropped) — the cell layout a browser lays out.
fn mtables(html: &str) -> Vec<Vec<Vec<String>>> {
  let re_table = regex::Regex::new(r"(?s)<(?:m:)?mtable[^>]*>(.*?)</(?:m:)?mtable>").unwrap();
  let re_row = regex::Regex::new(r"(?s)<(?:m:)?mtr[^>]*>(.*?)</(?:m:)?mtr>").unwrap();
  let re_cell = regex::Regex::new(r"(?s)<(?:m:)?mtd( [^>]*?)?(?:/>|>(.*?)</(?:m:)?mtd>)").unwrap();
  let re_tag = regex::Regex::new(r"<[^>]*>|\s").unwrap();
  re_table
    .captures_iter(html)
    .map(|t| {
      re_row
        .captures_iter(&t[1])
        .map(|r| {
          re_cell
            .captures_iter(&r[1])
            .map(|c| {
              let attrs = c.get(1).map_or("", |a| a.as_str().trim());
              let text = re_tag.replace_all(c.get(2).map_or("", |b| b.as_str()), "");
              format!("{attrs}|{text}")
            })
            .collect()
        })
        .collect()
    })
    .collect()
}

/// alignment/bordermatrix_mathml_covered_cells (RED on rc6: the rows below r_1 kept the two empty `ltx_nopad` cells
/// the fences cover, 5 `<mtd>`, so every later cell sat one column right) and alignment/multirow_math_array_mathml_cells
/// (GREEN on rc6 and after: a `\multirow` keeps every real cell, where Perl's `@spanned` loses c, d, b). One conversion
/// each, post's MathML in process; 0 E and 0 W.
#[test]
fn math_rowspan_covered_cells_are_omitted_once() {
  let bordermatrix: &[&[&[&str]]] = &[
    &[
      &[
        "|",
        r#"class="ltx_nopad"|"#,
        "|c1",
        "|c2",
        r#"class="ltx_nopad"|"#,
      ],
      &[
        "|r1",
        r#"class="ltx_nopad" rowspan="2"|("#,
        "|a11",
        "|a12",
        r#"class="ltx_nopad" rowspan="2"|)"#,
      ],
      &["|r2", "|a21", "|a22"],
    ],
    &[
      &[
        "|",
        r#"class="ltx_nopad"|"#,
        "|c1",
        "|c2",
        r#"class="ltx_nopad"|"#,
      ],
      &[
        "|r1",
        r#"class="ltx_nopad" rowspan="3"|("#,
        "|1",
        "|2",
        r#"class="ltx_nopad" rowspan="3"|)"#,
      ],
      &["|r2", "|3", "|4"],
      &["|r3", "|5", "|6"],
    ],
    &[
      &["|", r#"class="ltx_nopad"|"#, "|c1", r#"class="ltx_nopad"|"#],
      &[
        "|r1",
        r#"class="ltx_nopad" rowspan="1"|("#,
        "|7",
        r#"class="ltx_nopad" rowspan="1"|)"#,
      ],
    ],
  ];
  let multirow: &[&[&[&str]]] = &[
    &[&[r#"rowspan="2"|x"#, "|a", "|b"], &["|c", "|d"], &[
      "|e", "|f", "|g",
    ]],
    &[&["|a", r#"rowspan="2"|x"#, "|b"], &["|c", "|d"]],
    &[&[r#"columnspan="2" rowspan="2"|x"#, "|b"], &["|d"], &[
      "|e", "|f", "|g",
    ]],
  ];
  for (name, tex, expected) in [
    (
      "bordermatrix",
      include_str!(
        "../../../tools/perfect_kernel/repros/alignment/bordermatrix_mathml_covered_cells.tex"
      ),
      bordermatrix,
    ),
    (
      "multirow",
      include_str!(
        "../../../tools/perfect_kernel/repros/alignment/multirow_math_array_mathml_cells.tex"
      ),
      multirow,
    ),
  ] {
    let (log, _, (post_log, html)) = convert_with_then(tex, None, |xml| {
      let opts = PostOptions {
        pmml: true,
        default_resources: false,
        graphicimages: false,
        ..PostOptions::default()
      };
      let post = run_post_processing_logged(xml, &opts);
      (post.log, post.html)
    });
    let log = format!("{log}{post_log}");
    assert_eq!(error_count(&log), 0, "{name}: {log}");
    assert_eq!(warning_count(&log), 0, "{name}: {log}");
    assert_eq!(mtables(&html), expected, "{name}: {html}");
  }
}
