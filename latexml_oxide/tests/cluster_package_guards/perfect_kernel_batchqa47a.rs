//! Red/green guards for batch QA47A (BM Q3.3, "fire the kernel's hooks wherever LaTeX fires them"): `\begin{document}`
//! runs the kernel's `\@kernel@before@begindocument` / `\@kernel@after@begindocument` (latex.ltx:9511, :9513), so the
//! generic `cmd/<name>/<hook>` hooks patch TeX macros as pdflatex does, and a command LaTeXML binds takes the hook
//! with one counted Warning (an Info for a class's or package's own) instead of a false Error. A table of its own: no
//! hooks table exists yet, nor the shared Error-boundary table on this base. Each row's
//! oracle (pdflatex `!` count and text, Perl's) is in its repro header; the RED shapes stay catalog repros
//! (`command_hook_on_an_optional_argument_command_red`, `command_hook_on_a_tex_primitive_red`,
//! `command_hook_on_makecaption_caption_repeat_without_options_red`).
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

/// One repro: its source, its exact (errors, warnings), whole diagnostic lines it prints, and whole elements
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
      r"Warning:hook:cmd/emph/before LaTeXML's binding of \emph takes no generic hook; its code does not run",
      r"Warning:hook:cmd/section/before LaTeXML's binding of \section takes no generic hook; its code does not run",
      r"Warning:hook:cmd/footnote/before LaTeXML's binding of \footnote takes no generic hook; its code does not run",
      r"Warning:hook:cmd/item/before LaTeXML's binding of \item takes no generic hook; its code does not run",
      r"Warning:hook:cmd/arabic/before LaTeXML's binding of \arabic takes no generic hook; its code does not run",
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
  // Who owns the code on a bound command's hook sets the severity: the document's (top level, its own label,
  // `\AddToHookNext`, the body, a `\newtheorem` command's hook, named without "LaTeXML's binding of") a Warning; a
  // package's alone an Info, as its redefinition of a locked command (red on QA47A r1: the package's a Warning, each
  // message repeating its hook name; on r2's first build `\AddToHookNext`'s an Info).
  Row {
    name:     "command_hook_owner_sets_its_severity",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/command_hook_owner_sets_its_severity.tex"
    ),
    counts:   (0, 5),
    messages: &[
      r"Info:hook:cmd/section/before LaTeXML's binding of \section takes no generic hook; its code does not run",
      r"Warning:hook:cmd/emph/before LaTeXML's binding of \emph takes no generic hook; its code does not run",
      r"Warning:hook:cmd/arabic/before LaTeXML's binding of \arabic takes no generic hook; its code does not run",
      r"Warning:hook:cmd/item/before LaTeXML's binding of \item takes no generic hook; its code does not run",
      r"Warning:hook:cmd/thm/before \thm takes no generic hook; its code does not run",
      r"Warning:hook:cmd/footnote/before LaTeXML's binding of \footnote takes no generic hook; its code does not run",
    ],
    elements: &[
      (
        "p",
        "xml:id=\"S1.p1.1\"",
        r#"<p xml:id="S1.p1.1">A<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>N</note> <emph font="italic" xml:id="S1.p1.1.1">b</emph> c 1.</p>"#,
      ),
      (
        "p",
        "xml:id=\"Thmthm1.p1.1\"",
        r#"<p xml:id="Thmthm1.p1.1"><text font="italic" xml:id="Thmthm1.p1.1.1">T.</text></p>"#,
      ),
    ],
  },
  // The Error side: a hook on a register (no command to patch) is the kernel's Error, as pdflatex's two `!` (Perl 0);
  // the text kept (red on QA47Abase: 0 errors, the hooks never applied; revQA47A S1).
  Row {
    name:     "command_hook_on_a_register_errs",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/command_hook_on_a_register_errs.tex"
    ),
    counts:   (2, 0),
    messages: &[
      r"Error:errmessage:\errmessage LaTeX hooks Error: Generic hooks cannot be added to '\mylen'.",
      r"Error:errmessage:\errmessage LaTeX hooks Error: Generic hooks cannot be added to '\mycnt'.",
    ],
    elements: &[(
      "p",
      "xml:id=\"p1.1\"",
      r#"<p xml:id="p1.1">Text 0.0pt.</p>"#,
    )],
  },
  // caption.sty:270 lets `\@makecaption` to caption's, so a hook on it patches under babel-spanish's active `>` (red on
  // int163a: the kernel's body kept, 1 Error; 2401.04032's asmeconf.cls:598-604). CAP1.
  Row {
    name:     "command_hook_on_makecaption_under_caption",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/command_hook_on_makecaption_under_caption.tex"
    ),
    counts:   (0, 0),
    messages: &[],
    elements: &[
      (
        "text",
        "xml:id=\"p1.1.1\"",
        r#"<text font="typewriter" xml:id="p1.1.1">\long macro:#1#2-&gt;\UseHookWithArguments {cmd/@makecaption/before}{2}{#1}{#2}\lx@caption@ORI@makecaption {#1}{#2}</text>"#,
      ),
      (
        "caption",
        "class=\"ltx_centering\"",
        r#"<caption class="ltx_centering"><tag close=": ">Figura 1</tag>Una figura.</caption>"#,
      ),
    ],
  },
  // The other route: caption's full load after its kernel alone (floatrow's caption3, then subcaption) lets it then,
  // `\lx@caption@full@load` (red on int163a: 1 Error). CAP1.
  Row {
    name:     "command_hook_on_makecaption_caption_after_caption3",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/command_hook_on_makecaption_caption_after_caption3.tex"
    ),
    counts:   (0, 0),
    messages: &[],
    elements: &[(
      "caption",
      "class=\"ltx_centering\"",
      r#"<caption class="ltx_centering"><tag close=": ">Figura 1</tag>Una figura.</caption>"#,
    )],
  },
  // The Error side, as pdflatex's one `!`: the kernel's `\@makecaption` without caption, and with caption's kernel alone
  // (caption3 lets nothing; the binding's `\lx@caption@kernel@only` branch). CAP1.
  Row {
    name:     "command_hook_on_makecaption_kernel_errs",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/command_hook_on_makecaption_kernel_errs.tex"
    ),
    counts:   (1, 0),
    messages: &[
      r"Error:errmessage:\errmessage LaTeX hooks Error: Generic hooks cannot be added to '\@makecaption'.",
    ],
    elements: &[(
      "caption",
      "class=\"ltx_centering\"",
      r#"<caption class="ltx_centering"><tag close=": ">Figura 1</tag>Una figura.</caption>"#,
    )],
  },
  Row {
    name:     "command_hook_on_makecaption_caption3_alone_errs",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/command_hook_on_makecaption_caption3_alone_errs.tex"
    ),
    counts:   (1, 0),
    messages: &[
      r"Error:errmessage:\errmessage LaTeX hooks Error: Generic hooks cannot be added to '\@makecaption'.",
    ],
    elements: &[(
      "caption",
      "class=\"ltx_centering\"",
      r#"<caption class="ltx_centering"><tag close=": ">Figura 1</tag>Una figura.</caption>"#,
    )],
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
  // A label naming a package the document only queried is the document's code: `\@ifpackagelater` leaves its `\ver@`
  // `\relax`, no load (`state::file_loaded`; red on int167a: `Info:hook:cmd/emph/before`, 2 warnings). F14.
  Row {
    name:     "command_hook_a_queried_package_label_is_the_documents",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/command_hook_a_queried_package_label_is_the_documents.tex"
    ),
    counts:   (0, 3),
    messages: &[
      r"Warning:hook:cmd/emph/before LaTeXML's binding of \emph takes no generic hook; its code does not run",
      r"Warning:hook:cmd/arabic/before LaTeXML's binding of \arabic takes no generic hook; its code does not run",
      r"Warning:hook:cmd/footnote/before LaTeXML's binding of \footnote takes no generic hook; its code does not run",
    ],
    elements: &[(
      "p",
      "xml:id=\"p1.1\"",
      r#"<p xml:id="p1.1">A <emph font="italic" xml:id="p1.1.1">b</emph> 0<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>N</note>.</p>"#,
    )],
  },
  // A hook added from `begindocument/end` patches before its code is stored: its owner read after the hooks, a
  // package's alone an Info, the document's (beside a package's too) a Warning, on any TeX Live (red on KOMA1's base:
  // 3 warnings; KOMA-Script 3.49 scrartcl.cls:3306-3313 +1 Warning on every KOMA document).
  Row {
    name:     "command_hook_added_at_begindocument_end",
    tex:      include_str!(
      "../../../tools/perfect_kernel/repros/macro-state/command_hook_added_at_begindocument_end.tex"
    ),
    counts:   (0, 2),
    messages: &[
      r"Info:hook:cmd/@startsection/before LaTeXML's binding of \@startsection takes no generic hook; its code does not run",
      r"Warning:hook:cmd/footnote/before LaTeXML's binding of \footnote takes no generic hook; its code does not run",
      r"Warning:hook:cmd/emph/before LaTeXML's binding of \emph takes no generic hook; its code does not run",
    ],
    elements: &[(
      "p",
      "xml:id=\"S1.p1.1\"",
      r#"<p xml:id="S1.p1.1">A<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>N</note> <emph font="italic" xml:id="S1.p1.1.1">b</emph> c.</p>"#,
    )],
  },
];

/// Every row converted once, in-process: exact counts, the whole diagnostic lines, the whole elements.
#[test]
fn batchqa47a_command_hook_rows() {
  for row in ROWS {
    let (log, xml) = convert_with(row.tex, Some("ar5iv.sty"));
    let name = row.name;
    assert_eq!(error_count(&log), row.counts.0, "{name}: errors\n{log}");
    assert_eq!(warning_count(&log), row.counts.1, "{name}: warnings\n{log}");
    for message in row.messages {
      assert!(
        log.lines().any(|l| l == *message),
        "{name}: no line {message:?}\n{log}"
      );
    }
    for (tag, attrs, element) in row.elements {
      assert_element(&xml, tag, &[attrs], element);
    }
  }
}
