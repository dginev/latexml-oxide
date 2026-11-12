//! Red/green guards for batch QA47A (BM Q3.3, "fire the kernel's hooks wherever LaTeX fires them"): `\begin{document}`
//! runs the kernel's `\@kernel@before@begindocument` / `\@kernel@after@begindocument` (latex.ltx:9511, :9513), so the
//! generic `cmd/<name>/<hook>` hooks patch TeX macros as pdflatex does, and a command LaTeXML binds takes the hook
//! with one counted Warning instead of a false Error. A table of its own: no hooks table exists yet. Each row's
//! oracle (pdflatex `!` count and text, Perl's) is in its repro header; the RED shapes stay catalog repros
//! (`command_hook_on_an_optional_argument_command_red`, `command_hook_on_a_tex_primitive_red`).
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// One repro: its source, its exact (errors, warnings), the Warnings it names, and whole elements
/// `(tag, attributes, element)`.
struct Row {
  name:     &'static str,
  tex:      &'static str,
  counts:   (usize, usize),
  messages: &'static [&'static str],
  elements: &'static [(&'static str, &'static str, &'static str)],
}

const ROWS: &[Row] = &[
  // A user macro and a kept kernel macro (`\refstepcounter`) take their hooks: before, after, both (red: "[]").
  Row {
    name:     "generic_command_hooks_run",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/generic_command_hooks_run.tex"
    ),
    counts:   (0, 0),
    messages: &[],
    elements: &[("p", "xml:id=\"p1.1\"", r#"<p xml:id="p1.1">X¡1¿[bar].</p>"#)],
  },
  // Every kind of TeX macro the kernel patches: robust, `\NewDocumentCommand` (its prefixes read under
  // `\escapechar` `/`), delimited, kept kernel macros, and a hook added in the body inside a group (red: "[]").
  Row {
    name:     "command_hooks_on_tex_macros",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/command_hooks_on_tex_macros.tex"
    ),
    counts:   (0, 0),
    messages: &[],
    elements: &[
      (
        "p",
        "xml:id=\"p1.1\"",
        r#"<p xml:id="p1.1">Re Xf Dg <text font="typewriter" xml:id="p1.1.1">T</text> [rxdDt].</p>"#,
      ),
      (
        "p",
        "xml:id=\"A1.p1.1\"",
        r#"<p xml:id="A1.p1.1">[rxdDtp].</p>"#,
      ),
    ],
  },
  // The safe side of the Error boundary: pdflatex 0 `!`; a constructor, two locked macros, a macro reading
  // LaTeXML's argument types and a Rust-bodied one take no patch, one Warning each, the text kept (red: a false
  // Error on `\emph` once the kernel list ran; the lost "Before." uncounted before).
  Row {
    name:     "command_hook_on_a_binding_is_counted",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/command_hook_on_a_binding_is_counted.tex"
    ),
    counts:   (0, 5),
    messages: &[
      "cmd/emph/before: LaTeXML's binding of \\emph takes no generic hook",
      "cmd/section/before: LaTeXML's binding of \\section takes no generic hook",
      "cmd/footnote/before: LaTeXML's binding of \\footnote takes no generic hook",
      "cmd/item/before: LaTeXML's binding of \\item takes no generic hook",
      "cmd/arabic/before: LaTeXML's binding of \\arabic takes no generic hook",
    ],
    elements: &[
      (
        "p",
        "xml:id=\"S1.p1.1\"",
        r#"<p xml:id="S1.p1.1">A<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>N</note> <emph font="italic" xml:id="S1.p1.1.1">b</emph> c 1.</p>"#,
      ),
      (
        "p",
        "xml:id=\"S1.I1.i1.p1.1\"",
        r#"<p xml:id="S1.I1.i1.p1.1">Text.</p>"#,
      ),
    ],
  },
  // `\document`'s other kernel hooks are one-time: code added to them in the body runs at once (a stale RED repro,
  // GREEN since int160a; QA46).
  Row {
    name:     "atbegindocument_in_the_body_runs_now",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/atbegindocument_in_the_body_runs_now.tex"
    ),
    counts:   (0, 0),
    messages: &[],
    elements: &[(
      "p",
      "xml:id=\"p1.1\"",
      r#"<p xml:id="p1.1">A LateZ B EndZ C OtherZ</p>"#,
    )],
  },
  // `\meaning`'s prefixes print in `\escapechar`, as its body's names do (red: "\protected\long macro:…").
  Row {
    name:     "meaning_prefixes_follow_escapechar",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/meaning_prefixes_follow_escapechar.tex"
    ),
    counts:   (0, 0),
    messages: &[],
    elements: &[(
      "text",
      "xml:id=\"p1.1.1\"",
      r#"<text font="typewriter" xml:id="p1.1.1">/protected/long macro:#1-&gt;/relax #1|macro:-&gt;/relax </text>"#,
    )],
  },
];

/// Every row converted once, in-process: exact counts, the Warnings named, the whole elements.
#[test]
fn batchqa47a_command_hook_rows() {
  for row in ROWS {
    let (log, xml) = convert_with(row.tex, Some("ar5iv.sty"));
    let name = row.name;
    assert_eq!(error_count(&log), row.counts.0, "{name}: errors\n{log}");
    assert_eq!(warning_count(&log), row.counts.1, "{name}: warnings\n{log}");
    for message in row.messages {
      assert!(
        log
          .lines()
          .any(|l| l.starts_with("Warning:hook:") && l.contains(message)),
        "{name}: no Warning {message:?}\n{log}"
      );
    }
    for (tag, attrs, element) in row.elements {
      assert_element(&xml, tag, &[attrs], element);
    }
  }
}
