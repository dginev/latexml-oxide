//! GA1: an undefined `\c@<ctr>` is the error TeX and Perl raise; the counter leniency lives only where Perl's is.
//! Perl's `generateErrorStub` (State.pm:532-553) has no counter branch; its "Counter '<ctr>' was not defined; assuming
//! 0" is `CounterValue`'s (Package.pm:712), the counter readers', ours `counter::dialect::counter_value`. A new module:
//! no existing one holds the undefined-control-sequence stub's rows.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{convert_with, error_count, warning_count};

/// A whole element: its tag, identifying attributes, the element.
type Element = (&'static str, &'static [&'static str], &'static str);
/// One repro: its file, the exact Error and Warning counts, the "assuming 0" Warnings among them, the first Error's head
/// (empty for none), and whole elements.
type Row = (
  &'static str,
  usize,
  usize,
  usize,
  &'static str,
  &'static [Element],
);

const ROWS: &[Row] = &[
  // The undefined `\c@nosuch` read by `\the`: Perl's two errors (pdflatex's two `!`), no "assuming 0" Warning; "Value: 0."
  // as Perl and the PDF.
  (
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/undefined_counter_register_is_an_error.tex"
    ),
    2,
    0,
    0,
    "Error:undefined:\\c@nosuch",
    &[("p", &[], r#"<p>Value: 0.</p>"#)],
  ),
  // `\arabic{nosuch}` keeps `CounterValue`'s Warning; `\value{nosuch}` read as a number is the undefined control
  // sequence (an Error, as Perl) and "Missing number" (a Warning, as Perl).
  (
    include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/counter_readers_keep_their_leniency.tex"
    ),
    1,
    2,
    1,
    "Error:undefined:\\c@nosuch",
    &[("p", &[], "<p>Reader: 0.\nValue: 0.</p>")],
  ),
  // subfig's `\ContinuedFloat` in a float type without a sub-float counter (subfig.sty:587's `\@ifundefined` guard):
  // the number steps back, no sub-counter is touched, nothing is reported. Witness 1910.02851.
  (
    include_str!(
      "../../../tools/perfect_kernel/repros/captions-floats/subfig_continuedfloat_in_a_float_without_subfloats.tex"
    ),
    0,
    0,
    0,
    "",
    &[
      (
        "float",
        &[r#"xml:id="algorithm1""#],
        r#"<float class="ltx_float_algorithm" framed="top" inlist="loa" labels="LABEL:alg:a" placement="H" xml:id="algorithm1">
          <tags><tag><text font="bold">Algorithm 1</text></tag><tag role="refnum">1</tag>
          <tag role="typerefnum">Algorithm 1</tag></tags>
          <toccaption><tag close=" ">1</tag>Factorization</toccaption>
          <caption><tag close=" "><text font="bold">Algorithm 1</text></tag>Factorization</caption>
          <p framed="topbottom">First part.</p></float>"#,
      ),
      (
        "float",
        &[r#"xml:id="algorithm1a""#],
        r#"<float class="ltx_float_algorithm" framed="top" inlist="loa" labels="LABEL:alg:b" placement="H" xml:id="algorithm1a">
          <tags><tag><text font="bold">Algorithm 1</text></tag><tag role="refnum">1</tag>
          <tag role="typerefnum">Algorithm 1</tag></tags>
          <toccaption><tag close=" ">1</tag>Factorization (continued)</toccaption>
          <caption><tag close=" "><text font="bold">Algorithm 1</text></tag>Factorization (continued)</caption>
          <p framed="topbottom">Second part.</p></float>"#,
      ),
      (
        "para",
        &[r#"xml:id="p1""#],
        r#"<para xml:id="p1"><p>See <ref labelref="LABEL:alg:a"/> and <ref labelref="LABEL:alg:b"/>.</p></para>"#,
      ),
    ],
  ),
  // tabularray's own counters (tabularray.sty:185-194): defined at the binding's load, the table's size set before its
  // cells, each cell's place set at its template (`\lx@tblr@cell@open`); read by `\the`, `\the…` and `\arabic`,
  // inside a cell's text and in a `\textbf`; back to 0 after the table, as pdflatex prints. Witness: the manual
  // non-decimal-units (26 errors on GA1).
  (
    include_str!(
      "../../../tools/perfect_kernel/repros/alignment-bindings/tabularray_rownum_colnum_counters.tex"
    ),
    0,
    0,
    0,
    "",
    &[
      (
        "tabular",
        &[],
        r#"<tabular vattach="middle"><tbody>
          <tr><td align="left">a1.1</td><td align="left">b1.2/2.3</td><td align="left">c1-3</td></tr>
          <tr><td align="left">d2.1</td><td align="left">e<text font="bold">2.2</text></td>
          <td align="left">f2.3</td></tr></tbody></tabular>"#,
      ),
      ("p", &[], "<p>after: 0.0/0.0</p>"),
    ],
  ),
];

/// Repros macro-state/undefined_counter_register_is_an_error (GA0 finding 1), macro-state/
/// counter_readers_keep_their_leniency and captions-floats/subfig_continuedfloat_in_a_float_without_subfloats
/// (witness 1910.02851), alignment-bindings/tabularray_rownum_colnum_counters (GA1b): exact counts, the first Error,
/// whole elements.
#[test]
fn an_undefined_counter_register_is_the_error_tex_raises() {
  for (tex, errors, warnings, assuming, first_error, elements) in ROWS {
    let (stderr, xml) = convert_with(tex, None);
    let head = tex.lines().next().unwrap_or_default();
    assert_eq!(error_count(&stderr), *errors, "{head}\n{stderr}");
    assert_eq!(warning_count(&stderr), *warnings, "{head}\n{stderr}");
    let assumed = stderr
      .lines()
      .filter(|l| l.starts_with("Warning:") && l.contains("was not defined; assuming 0"));
    assert_eq!(
      assumed.count(),
      *assuming,
      "{head}: only a counter reader says \"assuming 0\"\n{stderr}"
    );
    if !first_error.is_empty() {
      let first = stderr
        .lines()
        .find(|l| l.starts_with("Error:"))
        .unwrap_or_default();
      assert!(
        first.starts_with(first_error),
        "{head}: first error {first:?}\n{stderr}"
      );
    }
    for (tag, attrs, element) in *elements {
      assert_element(&xml, tag, attrs, element);
    }
  }
}

/// The `Warn!` calls in `generate_error_stub`'s body, comment lines left out. The stub raises Perl's errors only
/// (State.pm:532-553): a leniency there would silence an undefined control sequence by its name, in every context.
fn stub_warnings(source: &str) -> usize {
  let start = source
    .find("pub fn generate_error_stub(")
    .expect("generate_error_stub is in state.rs");
  let body = &source[start..];
  let end = body.find("\n}\n").expect("the stub's closing brace");
  body[..end]
    .lines()
    .filter(|line| !line.trim_start().starts_with("//"))
    .map(|line| line.matches("Warn!(").count())
    .sum()
}

/// Fence F4 (GA0): `generate_error_stub` holds no `Warn!` (GA1 deleted its `\c@<ctr>` branch).
#[test]
fn the_undefined_stub_raises_errors_only() {
  assert_eq!(
    stub_warnings(include_str!("../../../latexml_core/src/state.rs")),
    0,
    "generate_error_stub warns: Perl's generateErrorStub (State.pm:532-553) raises Error:undefined only. A leniency \
     belongs to the reader Perl gives it (CounterValue, Package.pm:712: counter::dialect::counter_value), not to the stub"
  );
}
