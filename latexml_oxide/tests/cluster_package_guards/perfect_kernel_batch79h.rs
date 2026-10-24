//! Red/green guards for batch 79h (complete breakage, continued: user 2026-10-24): references into another document
//! through xr print what pdflatex prints — the other document's text typeset as TeX, and `\cref`/`\autoref` naming its
//! type. Each guard converts its repro under `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the
//! element whole.
use latexml::util::test::{assert_element, convert_files_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// The other document's `.aux`, as pdflatex writes it for a document with hyperref and cleveref: an IEEEtran-style
/// `\mbox` section number, each label's cleveref record (`<label>@cref`) and its hyperref anchor.
const SUPP_AUX: &str = r"\relax
\newlabel{sec:sa}{{\mbox  {II-A}}{1}{Sub}{subsection.2.1}{}}
\newlabel{sec:sa@cref}{{[subsection][1][2]\mbox  {II-A}}{[1][1][]1}{}{}{}}
\newlabel{fig:s1}{{S1}{1}{A caption}{figure.1}{}}
\newlabel{fig:s1@cref}{{[figure][1][]S1}{[1][1][]1}{}{}{}}
\newlabel{eq:s2}{{S2}{1}{Sub}{equation.2}{}}
\newlabel{eq:s2@cref}{{[equation][2][]S2}{[1][1][]1}{}{}{}}
";

/// `\cref`, `\Cref` and `\autoref` to labels xr imported print the type's name before the number, as pdflatex prints
/// them ("See fig. S1, Equation (S2), subsection II-A, Figure S1 and Section II-A"): cleveref's from the label's
/// `@cref` record (its type, named by this document's `\crefname`s, and its number in the type's label format),
/// hyperref's from the anchor's type (`subsection.2.1`). The text is typeset as TeX, so `\mbox {II-A}` prints "II-A".
/// A label of this document stays a reference post resolves. Red (int79c): the `\cref`s and `\autoref`s were missing
/// labels, the `\ref` printed "\mbox II-A". Witnesses 2412.14256, 2508.18808, 2608.12136 (`\cref`), 2402.05535
/// (`\autoref*`). Repro package-bindings/xr_external_cref_autoref.
#[test]
fn xr_cref_and_autoref_name_the_other_documents_types() {
  let (log, xml) = convert_files_with(
    repro!("package-bindings/xr_external_cref_autoref.tex"),
    &[("supp.aux", SUPP_AUX)],
    Some(PRELOAD),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "ref",
    &["labelref=\"LABEL:S-fig:s1\"", "show=\"creftype~creflabel\""],
    "<ref class=\"ltx_ref_external\" labelref=\"LABEL:S-fig:s1\" show=\"creftype~creflabel\">fig.\u{a0}S1</ref>",
  );
  assert_element(
    &xml,
    "ref",
    &["labelref=\"LABEL:S-eq:s2\""],
    "<ref class=\"ltx_ref_external\" labelref=\"LABEL:S-eq:s2\" show=\"creftypecap~creflabel\">Equation\u{a0}(S2)</ref>",
  );
  assert_element(
    &xml,
    "ref",
    &["labelref=\"LABEL:S-sec:sa\"", "show=\"autoref\""],
    "<ref class=\"ltx_refmacro_autoref ltx_ref_external\" labelref=\"LABEL:S-sec:sa\" show=\"autoref\">subsection\u{a0}<text xml:id=\"S1.p1.1.1\">II-A</text></ref>",
  );
  assert_element(
    &xml,
    "ref",
    &["labelref=\"LABEL:S-fig:s1\"", "show=\"autoref\""],
    "<ref class=\"ltx_refmacro_autoref ltx_nolink ltx_ref_external\" labelref=\"LABEL:S-fig:s1\" show=\"autoref\">Figure\u{a0}S1</ref>",
  );
  assert!(
    xml.contains("<ref class=\"ltx_ref_external\" labelref=\"LABEL:S-sec:sa\">II-A</ref>"),
    "{xml}"
  );
  assert_element(
    &xml,
    "ref",
    &["labelref=\"LABEL:sec:main\""],
    r#"<ref labelref="LABEL:sec:main" show="creftype~creflabel"/>"#,
  );
}
