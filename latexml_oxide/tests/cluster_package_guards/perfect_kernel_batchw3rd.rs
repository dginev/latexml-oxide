//! W3RD (PR #829 Rhai round D): diagnostics as Perl's (Common/Error.pm), the Package.pm options Rhai dropped, and the
//! thin wrappers over existing core calls. Each row is pinned against Perl `latexml` with the equivalent `.ltxml`
//! binding, or against TeX where Perl does not check.
#![cfg(feature = "runtime-bindings")]

use latexml::util::test::{assert_element, convert_files_with};

use crate::perfect_kernel_batch46::{error_count, warning_count};

const LXRD: &str = r#"Tag("ltx:emph", #{afterOpen: |doc, node| node.setAttribute("class", "ao"),
  "afterClose:late": [|d, n| n.setAttribute("title", "c1")]});
NewCounter("lxrdc");
NewCounter("lxrdd", "", #{idprefix: "X"});
DefPrimitive("\\rsc", || { RefStepCounter("lxrdc"); });
DefMacro("\\cv", || `${CounterValue("lxrdc")}`);
DefMacro("\\rid", || RefStepCounter("lxrdd").id);
DefPrimitive("\\unmean", || AssignMeaning("\\lxa", "\\lxundefined"));
DefPrimitive("\\named", || Let("\\lxn", "\\relax", "lxscope"));
DefConditional("\\iflxtwo{}{}", || true);
AtBeginDocument(|| "\\gdef\\lxabd{ABD}");
DefConstructor("\\lxhk", "<ltx:text>H</ltx:text>",
  #{afterDigest: [|| AssignValue("lxrd:h1", "1", "global"), || AssignValue("lxrd:h2", "2", "global")]});
DefMacro("\\hk", || LookupValue("lxrd:h1") + LookupValue("lxrd:h2"));
RequirePackage("lxrdp", #{options: ["optA"]});
"#;

const LXRDP: &str = r#"DeclareOption("optA", || RawTeX("\\gdef\\lxopt{OA}"));
DeclareOption("optB", || RawTeX("\\gdef\\lxopt{OB}"));
ProcessOptions(#{inorder: true});
"#;

const LXRD_TEX: &str = r"\documentclass{article}
\usepackage{lxrd}
\begin{document}
\emph{E}

{\renewcommand\refstepcounter[1]{\stepcounter{#1}\stepcounter{#1}}\rsc}\cv{} \rid

\def\lxa{A}\unmean\ifdefined\lxa D\else U\fi{} \named\ifdefined\lxn D\else U\fi{} \iflxtwo{a}{b}T\else F\fi

\lxabd{} \lxhk\hk{} \lxopt
\end{document}
";

/// The clean side, 0 Errors / 0 Warnings; each row red on 5a709726503 (Perl's output with the equivalent `.ltxml`):
/// p1: `Tag` `afterOpen` and an array under `afterClose:late` run (were dropped); the 2-parameter closure is called
/// with Perl's 3 arguments `($document, $node, $box)` (Rhai required the exact count).
/// p2: `RefStepCounter` steps through `\refstepcounter`'s meaning, as `RefStepCounter!` (2; was 1, the kernel step);
/// `NewCounter`'s `idprefix` (`X1`; was "Function not found").
/// p3: `AssignMeaning` to an undefined token undefines (U; was a no-op, D); a named scope is not the current one (U;
/// was dropped to local, D); a test closure declaring no parameters takes the conditional's two (T; was an Error).
/// p4: `AtBeginDocument` of a closure (ABD), an array of `afterDigest` closures (both run, 12), `RequirePackage`'s
/// `options` and `ProcessOptions`' `inorder` (OA) — each "Function not found" or dropped before.
#[test]
fn rhai_package_options_and_core_calls() {
  let (log, xml) = convert_files_with(
    LXRD_TEX,
    &[("lxrd.sty.rhai", LXRD), ("lxrdp.sty.rhai", LXRDP)],
    None,
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  for (id, para) in [
    (
      "p1",
      r#"<para xml:id="p1"><p><emph class="ao" font="italic" title="c1">E</emph></p></para>"#,
    ),
    ("p2", r#"<para xml:id="p2"><p>2 X1</p></para>"#),
    ("p3", r#"<para xml:id="p3"><p>U U T</p></para>"#),
    ("p4", r#"<para xml:id="p4"><p>ABD H12 OA</p></para>"#),
  ] {
    assert_element(&xml, "para", &[&format!("xml:id=\"{id}\"")], para);
  }
}

const LXRD_ERR: &str = r#"DefPrimitive("\\badcodes", || { AssignCatcode("Q", 16); AssignDelcode("Q", -1); AssignDelcode("(", 164608); });
DefPrimitive("\\diags", || { Warn("mycat", "myobj", "a warning", ["wd1", "wd2"]); Error("mycat", "myobj", "an error", "ed1");
  Info("mycat", "myobj", "an info", "id1"); Tag("ltx:p", #{bogus: 1}); });
DefPrimitive("\\fatal", || Fatal("mycat", "myobj", "a fatal", "fd1"));
"#;

const LXRD_ERR_TEX: &str = r"\documentclass{article}
\usepackage{lxrderr}
\begin{document}
\badcodes\the\catcode`\Q

\diags
\end{document}
";

const LXRD_FATAL_TEX: &str = r"\documentclass{article}
\usepackage{lxrderr}
\begin{document}
\fatal
\end{document}
";

/// The Error side of the round's boundary. e1 (revW3e): an `Assign*code` value the table cannot hold is an Error
/// naming the call (TeX: "Invalid code (16), should be at most 15"; a delcode above "FFFF is the u16 table's limit,
/// `macro-state/delcode_holds_24_bits_red.tex`); red on f528beee1fb, which dropped all three silently (E=0).
/// e2: `Warn`/`Error`/`Info` take Perl's trailing `@details` (a string or an array, one line each; Error.pm:360/:378;
/// "Function not found" on 5a709726503); a `Tag` key outside Perl's table is an Error (Package.pm:2017; was ignored).
#[test]
fn rhai_diagnostics_error_side() {
  let (log, xml) = convert_files_with(LXRD_ERR_TEX, &[("lxrderr.sty.rhai", LXRD_ERR)], None);
  assert_eq!(error_count(&log), 5, "{log}");
  assert_eq!(warning_count(&log), 1, "{log}");
  for line in [
    "Warning:mycat:myobj a warning",
    "\twd1\n\twd2",
    "Error:mycat:myobj an error",
    "\ted1",
    "Info:mycat:myobj an info",
    "\tid1",
    "Error:misdefined:Tag (ltx:p) Tag (ltx:p) does not accept options:bogus",
  ] {
    assert!(log.contains(line), "{line}\n{log}");
  }
  for call in [
    "AssignCatcode(\"Q\", 16)",
    "AssignDelcode(\"Q\", -1)",
    "AssignDelcode(\"(\", 164608)",
  ] {
    assert!(
      log.contains(&format!(
        "Error:unexpected:{} {call}",
        call.split('(').next().unwrap()
      )),
      "{call}\n{log}"
    );
  }
  assert_element(&xml, "para", &[], r#"<para xml:id="p1"><p>11</p></para>"#);
  // e3: `Fatal` logs Perl's `Fatal:<cat>:<obj> <msg>` with its details and ends the run (was
  // `Fatal:Internal:Unexpected  mycat:myobj a fatal`).
  let (log, _) = convert_files_with(LXRD_FATAL_TEX, &[("lxrderr.sty.rhai", LXRD_ERR)], None);
  assert_eq!(error_count(&log), 1, "{log}");
  // (the core's Fatal header ends in a space, so every Fatal line has two before its message)
  assert!(log.contains("Fatal:mycat:myobj  a fatal\n\tfd1"), "{log}");
}
