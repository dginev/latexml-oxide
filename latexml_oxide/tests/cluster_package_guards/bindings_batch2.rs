//! Guards for the second bindings batch of 2026-11-13 (issues #787, #773): one table-driven test per issue, each repro
//! converted once in-process and checked whole, with exact Error/Warning counts.

use latexml::util::test::{convert_with, error_count, kpse_has, warning_count};

macro_rules! repro {
  ($dir:literal, $path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $dir,
      "/",
      $path
    ))
  };
}

/// Every element `<tag …>…</tag>` of `xml`, whole, in document order.
fn elements(xml: &str, tag: &str) -> Vec<String> {
  let (open, close) = (format!("<{tag}"), format!("</{tag}>"));
  let mut found = Vec::new();
  let mut rest = xml;
  while let Some(at) = rest.find(&open) {
    let tail = &rest[at..];
    let end = tail.find(&close).map_or(tail.len(), |e| e + close.len());
    found.push(tail[..end].to_string());
    rest = &tail[end..];
  }
  found
}

/// Issue #787: caption's `justification=` is the caption's `text-align` (its body's last alignment command; `justify`
/// for an empty one), replacing the class a `\centering` of its float gives it, and `skip=`/`belowskip=` its margins on
/// the sides caption.sty:251-258 puts them, `position=auto` read as caption3.sty:1034-1045 reads it when the caption
/// begins (the mode, then the list), a longtable's by its rows (caption.sty:1169). Red on base 6df4632fb1: every
/// `<caption>` without class or cssstyle; on bf059b707e (review revCAP): alignment classes beside the float's
/// `ltx_centering`, a `margin-top:0.0pt`, five auto positions read top, ragged2e's and a declared justification ignored;
/// on df5149f093 (review revCAP2): a stale auto mark (listings' Listing 1 `margin-top:9.0pt` after a sub-caption or a
/// stray `\caption`), no `singlelinecheck` centring, an alignment-free justification left to the float's `ltx_centering`.
/// Each row: its repro's Error count (Warnings 0) and every `<caption>` whole.
#[test]
fn caption_justification_and_skip() {
  if ![
    "caption.sty",
    "longtable.sty",
    "ragged2e.sty",
    "subcaption.sty",
    "listings.sty",
  ]
  .iter()
  .all(|f| kpse_has(f))
  {
    return;
  }
  let rows: [(&str, usize, &[&str]); 9] = [
    (
      repro!("captions-floats", "caption_justification_and_skip.tex"),
      0,
      &[
        "<caption cssstyle=\"text-align:center;width:fit-content;margin-left:auto;margin-right:auto;margin-bottom:20.0pt\"><tag close=\": \">Table 1</tag>Results</caption>",
        "<caption cssstyle=\"text-align:center;width:fit-content;margin-left:auto;margin-right:auto\"><tag close=\": \">Table 2</tag>After</caption>",
      ],
    ),
    (
      repro!(
        "captions-floats",
        "caption_justification_and_skip_sides.tex"
      ),
      0,
      &[
        "<caption cssstyle=\"text-align:left;width:fit-content;margin-left:auto;margin-right:auto;margin-top:4.0pt\"><tag close=\": \">Figure 1</tag>Bottom figure</caption>",
        "<caption cssstyle=\"text-align:right;width:fit-content;margin-left:auto;margin-right:auto;margin-top:6.0pt\"><tag close=\": \">Table 1</tag>Top table</caption>",
        "<caption cssstyle=\"text-align:center;width:fit-content;margin-left:auto;margin-right:auto;margin-bottom:4.0pt\">Starred</caption>",
      ],
    ),
    // Review revCAP B1: the float's `\centering` (`{center}`, a minipage's) no longer classes the caption.
    (
      repro!(
        "captions-floats",
        "caption_justification_replaces_the_float_alignment.tex"
      ),
      0,
      &[
        "<caption cssstyle=\"text-align:left;width:fit-content;margin-left:auto;margin-right:auto;margin-top:3.0pt\"><tag close=\": \">Figure 1</tag>Centred float.</caption>",
        "<caption cssstyle=\"text-align:left;width:fit-content;margin-left:auto;margin-right:auto;margin-top:3.0pt\">In center.</caption>",
        "<caption cssstyle=\"text-align:left;width:fit-content;margin-left:auto;margin-right:auto;margin-top:3.0pt\"><tag close=\": \">Figure 2</tag>In a minipage.</caption>",
        "<caption cssstyle=\"text-align:center;width:fit-content;margin-left:auto;margin-right:auto;margin-bottom:10.0pt\"><tag close=\": \">Table 1</tag>Top table.</caption>",
        "<caption cssstyle=\"text-align:center;width:fit-content;margin-left:auto;margin-right:auto;margin-top:10.0pt\"><tag close=\": \">Table 2</tag>Bottom table.</caption>",
        "<caption cssstyle=\"text-align:justify;width:fit-content;margin-left:auto;margin-right:auto;margin-top:3.0pt\"><tag close=\": \">Figure 3</tag>Justified.</caption>",
        "<caption cssstyle=\"text-align:justify;width:fit-content;margin-left:auto;margin-right:auto;margin-top:3.0pt\"><tag close=\": \">Figure 4</tag>Ragged2e justified.</caption>",
      ],
    ),
    // Review revCAP B2: margin-top is a bottom caption, margin-bottom a top one.
    (
      repro!(
        "captions-floats",
        "caption_auto_position_reads_the_mode.tex"
      ),
      0,
      &[
        "<caption cssstyle=\"margin-top:5.0pt\"><tag close=\": \">Figure 1</tag>Mini after par.</caption>",
        "<caption cssstyle=\"margin-bottom:5.0pt\"><tag close=\": \">Figure 2</tag>Mini first.</caption>",
        "<caption cssstyle=\"margin-bottom:5.0pt\"><tag close=\": \">Figure 3</tag>First.</caption>",
        "<caption cssstyle=\"margin-top:5.0pt\"><tag close=\": \">Figure 4</tag>Second.</caption>",
        "<caption cssstyle=\"margin-top:5.0pt\"><tag close=\": \">Figure 5</tag>After par.</caption>",
        "<caption cssstyle=\"margin-bottom:5.0pt\"><tag close=\": \">Figure 6</tag>After vspace.</caption>",
        "<caption cssstyle=\"margin-top:5.0pt\"><tag close=\": \">Table 1</tag>Captionof in a float.</caption>",
        "<caption cssstyle=\"margin-top:5.0pt\"><tag close=\": \">Figure 7</tag>Inline.</caption>",
        "<caption cssstyle=\"margin-top:5.0pt\"><tag close=\": \">Table 2</tag>LT bottom.</caption>",
        "<caption cssstyle=\"margin-bottom:5.0pt\"><tag close=\": \">Table 3</tag>LT top.</caption>",
      ],
    ),
    // Review revCAP B3: ragged2e's `RaggedRight`, a declared justification ending in `\raggedright`.
    (
      repro!("captions-floats", "caption_ragged2e_justification.tex"),
      0,
      &[
        "<caption cssstyle=\"text-align:left;width:fit-content;margin-left:auto;margin-right:auto\"><tag close=\": \">Figure 1</tag>Fig cap.</caption>",
        "<caption cssstyle=\"text-align:left;width:fit-content;margin-left:auto;margin-right:auto;margin-top:12.0pt\"><tag close=\": \">Table 1</tag>Tab cap.</caption>",
        "<caption cssstyle=\"text-align:left;width:fit-content;margin-left:auto;margin-right:auto;margin-top:12.0pt\"><tag close=\": \">Table 2</tag>Captionof in figure.</caption>",
      ],
    ),
    // Review revCAP2 q5: the auto mark is its caption's; a sub-caption drops it, listings' caption reads its own.
    (
      repro!(
        "captions-floats",
        "caption_auto_mark_belongs_to_its_caption.tex"
      ),
      0,
      &[
        "<caption><tag close=\" \"><text fontsize=\"90%\">(a)</text></tag><text fontsize=\"90%\">Sub after X.</text></caption>",
        "<caption cssstyle=\"margin-bottom:9.0pt\"><tag close=\": \">Listing 1</tag>Listing cap.</caption>",
        "<caption cssstyle=\"margin-bottom:9.0pt\"><tag close=\": \">Listing 2</tag>Second listing.</caption>",
      ],
    ),
    // The inline caption of a `\caption` outside a float (caption's error) drops the mark too.
    (
      repro!(
        "captions-floats",
        "caption_auto_mark_dropped_by_a_stray_caption.tex"
      ),
      1,
      &[
        "<caption cssstyle=\"margin-bottom:9.0pt\"><tag close=\": \">Listing 1</tag>Listing cap.</caption>",
      ],
    ),
    // Review revCAP2 p6: `singlelinecheck` on (the default) centres a short caption; `false`/`slc=off` do not.
    (
      repro!(
        "captions-floats",
        "caption_singlelinecheck_centres_a_short_caption.tex"
      ),
      0,
      &[
        "<caption cssstyle=\"text-align:left;width:fit-content;margin-left:auto;margin-right:auto\"><tag close=\": \">Figure 1</tag>Short on.</caption>",
        "<caption cssstyle=\"text-align:left\"><tag close=\": \">Table 1</tag>Short off.</caption>",
        "<caption cssstyle=\"text-align:left\"><tag close=\": \">Figure 2</tag>Off by slc.</caption>",
      ],
    ),
    // Review revCAP2 q3: a justification body with no alignment command is justified text.
    (
      repro!(
        "captions-floats",
        "caption_justification_without_alignment_is_justified.tex"
      ),
      0,
      &[
        "<caption cssstyle=\"text-align:justify;width:fit-content;margin-left:auto;margin-right:auto\"><tag close=\": \">Figure 1</tag>A caption.</caption>",
      ],
    ),
  ];
  for (tex, errors, expected) in rows {
    let (log, xml) = convert_with(tex, None);
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (errors, 0),
      "{log}"
    );
    assert_eq!(elements(&xml, "caption"), expected, "{xml}");
  }
}
