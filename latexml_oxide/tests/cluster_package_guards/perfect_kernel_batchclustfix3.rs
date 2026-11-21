//! Red/green guards for batch CLUSTFIX3, four more Error roots of run 397 where TeX is clean: amsmath's `\[`/`\]` as
//! `{equation*}`, etoolbox's `\robustify` of a native binding, the native pgfkeys accessors over a paper's own pgf 2.00
//! `pgfkeys.code.tex`, and doublespace's `\setstretch`. Repros under `tools/perfect_kernel/repros/`.
use latexml::util::test::{assert_element, convert_files_with, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The counts `(errors, warnings)` of `log`, which must hold no Fatal.
fn counts(log: &str) -> (usize, usize) {
  assert!(!log.contains("Fatal:"), "{log}");
  (error_count(log), warning_count(log))
}

/// amsmath.sty:2950-2951 makes `\[`/`\]` `\begin{equation*}`/`\end{equation*}`, so `\[ … \end{equation*}` and
/// `\begin{equation*} … \]` pair (math0306159, math0401181, math0508341; OD #1091). Before: "Can't close environment
/// equation*", the second pair's `\]` ran the kernel's cheap intertext, and a `\tag` in `\[` was dropped without a
/// diagnostic (2008.12392, 2409.02265). A matched `\[ … \]` is unchanged.
/// Repro ams-internals/amsmath_display_brackets_pair_with_equation_star.
#[test]
fn amsmath_display_brackets() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/ams-internals/amsmath_display_brackets_pair_with_equation_star.tex"
    ),
    None,
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_eq!(xml.matches("<equation ").count(), 4, "{xml}");
  for (id, tex) in [("S0.Ex1", "a+b=c."), ("S0.Ex2", "x=y."), ("S0.Ex3", "p=q.")] {
    let head = format!("<equation xml:id=\"{id}\">\n      <Math mode=\"display\" tex=\"{tex}\"");
    assert!(xml.contains(&head), "{xml}");
  }
  let tagged = concat!(
    "<equation xml:id=\"S0.Ex4\">\n      <tags>\n        <tag>(T)</tag>\n",
    "        <tag role=\"refnum\">T</tag>\n      </tags>\n      <Math mode=\"display\" tex=\"r=s\""
  );
  assert!(xml.contains(tagged), "{xml}");
  for words in [
    "We have",
    "and more text.",
    "then text.",
    "end, and a tag:",
    "done.",
  ] {
    assert!(xml.contains(&format!("<p>{words}</p>")), "{xml}");
  }
}

/// The `<p>` texts and the `<equation>` ids with their `tex`, in order and joined by ` | `, and the count of `<tags>`.
fn display_run(xml: &str) -> (String, usize) {
  let mut run = Vec::new();
  for line in xml.lines().map(str::trim) {
    if let Some(rest) = line.strip_prefix("<equation xml:id=\"") {
      run.push(rest.split('"').next().unwrap_or_default().to_string());
    } else if let Some(rest) = line.strip_prefix("<Math mode=\"display\" tex=\"") {
      run.push(rest.split('"').next().unwrap_or_default().to_string());
    } else if line.starts_with("<p>") || line.starts_with("<p xml:id=") {
      let text = line.split_once('>').map_or("", |(_, t)| t);
      run.push(text.trim_end_matches("</p>").to_string());
    }
  }
  (run.join(" | "), xml.matches("<tags>").count())
}

/// A `$$` inside amsmath's `\[ … \]` (= `{equation*}`) ends the display and the next `$$` opens one that `\]` closes
/// (tex.web §1193-1197, §1138; amsmath.sty:2794, 2813): two unnumbered displays, the text between them kept. The
/// kernel's cheap intertext (`\lx@eDM@in@equation`, sect07.rs) closed and reopened `{equation}` whatever display was
/// open: "Can't close environment equation" / "equation*" and a spurious numbered equation per display (2606.13867,
/// a regression from #929; the written-out `{equation*}` erred before, as in Perl, KNOWN_PERL_ERRORS). OD #1091.
/// Repro ams-internals/amsmath_display_brackets_dollar_dollar.
#[test]
fn amsmath_display_brackets_dollar_dollar() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/ams-internals/amsmath_display_brackets_dollar_dollar.tex"
    ),
    None,
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  let expected = concat!(
    "First | S0.Ex1 | a=b | S0.Ex2 | =c. | Then | S0.Ex3 | x=y | text kept | S0.Ex4 | =z. | ",
    "Also | S0.Ex5 | p=q | S0.Ex6 | =r. | Done."
  );
  assert_eq!(display_run(&xml), (expected.to_string(), 0), "{xml}");
}

/// The numbered `{equation}` path of the same cheap intertext is unchanged: the first half's number is retracted, the
/// second half carries it, and `$$ text $$` keeps the text between them. Companion of
/// `amsmath_display_brackets_dollar_dollar`; repro ams-internals/amsmath_equation_dollar_dollar_keeps_its_number.
#[test]
fn amsmath_equation_dollar_dollar_keeps_its_number() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/ams-internals/amsmath_equation_dollar_dollar_keeps_its_number.tex"
    ),
    None,
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  let expected =
    "Text | S0.Ex1 | a=b | text kept | S0.E1 | =c. | Then | S0.Ex2 | p=q | S0.E2 | =r. | Done.";
  assert_eq!(display_run(&xml), (expected.to_string(), 2), "{xml}");
  for n in ["1", "2"] {
    let tags = format!(
      "<tags>\n        <tag>({n})</tag>\n        <tag role=\"refnum\">{n}</tag>\n      </tags>"
    );
    assert!(
      xml.contains(&format!("<equation xml:id=\"S0.E{n}\">\n      {tags}")),
      "{xml}"
    );
  }
}

/// etoolbox's `\robustify` of a command that is a macro in LaTeX and a native binding here is a no-op (it is robust
/// already): `\uparrow`, `\sum`, `\it`, `\uline` had "not a macro" (1508.06145, 1702.00233), and `\robustify{\ref}`
/// left `\ref` an undefined `\etb@resrvda` (wiley-article.cls:771-775, 1812.10085), the reference lost. A TeX
/// primitive or a register keeps etoolbox's Error, as pdflatex's (`\hbox`, a constructor here, `\relax`,
/// `\parindent`; OD #1092). Repros package-bindings/robustify_of_a_native_binding and
/// robustify_of_a_tex_primitive_errs.
#[test]
fn robustify_of_a_native_binding() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/robustify_of_a_native_binding.tex"
    ),
    None,
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  let p = concat!(
    r#"<p>See Section"#,
    "\u{a0}",
    r#"<ref labelref="LABEL:s:a"/> here. Text <text font="italic">slanted</text>, "#,
    r#"<text class="ltx_ulem_uline">under</text> and <Math mode="inline" "#,
    r#"tex="\sum_{i}x_{i}\uparrow" "#
  );
  assert!(xml.contains(p), "{xml}");

  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/robustify_of_a_tex_primitive_errs.tex"
    ),
    None,
  );
  assert_eq!(counts(&log), (3, 0), "{log}");
  for message in [
    r"\hbox not a macro",
    r"\relax undefined",
    r"\parindent not a macro",
  ] {
    assert!(
      log.contains(&format!("Package etoolbox Error: {message}")),
      "{log}"
    );
  }
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>Box x.</p></para>"#,
  );
}

/// The native pgfkeys accessors (pgfkeys_code_tex.rs) are installed only over a file of TeX Live's shape: a paper's
/// own pgf 2.00 `pgfkeys.code.tex` (delimited `\pgfkeys@ifcsname`, no `\pgfkeys@firstoftwo`; 1108.0463, 1010.2550)
/// keeps its raw code. Before: 6 Errors (`\pgfkeys@secondoftwo` undefined, Extra `\endcsname`, …) and "[A:x]" lost.
/// Repro tikz-pgf/pgf2_pgfkeys (its stand-in files beside it).
#[test]
fn pgfkeys_natives_skip_a_pgf2_pgfkeys() {
  let (log, xml) = convert_files_with(
    include_str!("../../../tools/perfect_kernel/repros/tikz-pgf/pgf2_pgfkeys/pgf2_pgfkeys.tex"),
    &[
      (
        "pgfkeys.sty",
        include_str!("../../../tools/perfect_kernel/repros/tikz-pgf/pgf2_pgfkeys/pgfkeys.sty"),
      ),
      (
        "pgfkeys.code.tex",
        include_str!("../../../tools/perfect_kernel/repros/tikz-pgf/pgf2_pgfkeys/pgfkeys.code.tex"),
      ),
    ],
    None,
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>Text [A:x] end.</p></para>"#,
  );
}

/// doublespace.sty:66 (TeX Live has none; the copy quant-ph0004056 ships) defines `\setstretch`: undefined before
/// (physics0003041, quant-ph0005132). Repro package-bindings/doublespace_setstretch.
#[test]
fn doublespace_setstretch() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/doublespace_setstretch.tex"
    ),
    None,
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p>Text one.</p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p>Text two.</p></para>"#,
  );
}
