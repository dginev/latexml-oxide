//! W3RA (PR #829 Rhai round A): a runtime binding's Def* option bag is checked against Perl's option
//! table for its definer (Package.pm CheckOptions :1062-1067) and every key the core carries is
//! honoured. Each row is pinned against Perl `latexml` with the equivalent `.ltxml` binding.
#![cfg(feature = "runtime-bindings")]

use latexml::util::test::{assert_element, convert_files_with};

use crate::perfect_kernel_batch46::{error_count, warning_count};

const HONOURED: &str = r#"DefConstructor("\\fooC", "<ltx:emph>C</ltx:emph>", #{locked: true});
DefPrimitive("\\mkdefs", || {
  DefConstructor("\\fooG", "<ltx:emph>G</ltx:emph>", #{scope: "global"});
  DefConstructor("\\fooL", "<ltx:emph>L</ltx:emph>");
});
DefConstructor("\\fooR{}", "<ltx:emph>#1</ltx:emph>", #{robust: true});
DefPrimitive("\\bfon", || {}, #{font: #{series: "bold"}});
DefMath("\\mysym", "S", #{font: #{family: "typewriter"}, reversion: "\\other"});
DefConditional("\\ifmyflag");
DefPrimitive("\\setflag", || SetCondition("\\ifmyflag", true));
DefConditional("\\ifisx{}", |a| UnTeX(a) == "x");
DefConstructor("\\fooO", "<ltx:emph>O</ltx:emph>", #{outer: true});
"#;

const HONOURED_TEX: &str = r"\documentclass{article}
\usepackage{lxopts}
\begin{document}
\renewcommand\fooC{X}\fooC

{\mkdefs}\fooG\ifdefined\fooL L\else N\fi

\texttt{\meaning\fooR}

{\bfon B}

$\mysym$

\ifmyflag A\else B\fi \myflagtrue\ifmyflag C\else D\fi \myflagfalse\setflag\ifmyflag E\else F\fi
\ifisx{x}Y\else Z\fi\ifisx{y}Y\else Z\fi
\end{document}
";

/// Rows, each red on 8ff0d441504 (the option was dropped, or the call did not exist) and Perl's output now:
/// `locked` survives `\renewcommand` (was `X`); `scope:"global"` outlives the group (was undefined `\fooG`);
/// `robust` makes `\fooR` the `\protect\fooR ` macro (was `\fooR`); DefPrimitive `font:` (was plain `B`);
/// DefMath `font:`/`reversion:` (was italic, `tex="\mysym"`); the test-less DefConditional with
/// `\myflagtrue`/`\myflagfalse`/`SetCondition` (was "Function not found"); a test given Tokens (`UnTeX(a)`).
/// One Warning: `outer` on a DefConstructor, which Perl honours and the core cannot carry yet (it was dropped
/// without a word); Perl 0/0.
#[test]
fn rhai_option_bags_follow_perls_tables() {
  let (log, xml) = convert_files_with(HONOURED_TEX, &[("lxopts.sty.rhai", HONOURED)], None);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 1, "{log}");
  assert!(
    log.lines().any(|line| line
      == "Warning:unsupported:DefConstructor (\\fooO) DefConstructor (\\fooO): option outer is not supported yet; ignored"),
    "{log}"
  );
  for (id, para) in [
    ("p1", r#"<para xml:id="p1"><p><emph>C</emph></p></para>"#),
    ("p2", r#"<para xml:id="p2"><p><emph>G</emph>N</p></para>"#),
    (
      "p3",
      r#"<para xml:id="p3"><p><text font="typewriter">macro:-&gt;\protect \fooR  </text></p></para>"#,
    ),
    (
      "p4",
      r#"<para xml:id="p4"><p><text font="bold">B</text></p></para>"#,
    ),
    (
      "p5",
      r#"<para xml:id="p5"><p><Math mode="inline" tex="\other" text="mysym" xml:id="p5.m1"><XMath><XMTok font="typewriter italic" name="mysym" role="UNKNOWN">S</XMTok></XMath></Math></p></para>"#,
    ),
    ("p6", r#"<para xml:id="p6"><p>BCEYZ</p></para>"#),
  ] {
    assert_element(&xml, "para", &[&format!("xml:id=\"{id}\"")], para);
  }

  // The Error side (red on 8ff0d441504: 0 Errors, and DefConditional with options did not exist): a key outside
  // the definer's table is one `Error:misdefined` each, worded as Perl's, and the definition is still made;
  // DefPrimitive's `nargs` is not in Perl's `$primitive_options`. Perl: 7 Errors, the same lines.
  let bogus = r#"DefMacro("\\fooT", "T", #{bogusopt: true});
DefPrimitive("\\fooP", || {}, #{bogusopt: true});
DefPrimitive("\\fooPn", || {}, #{nargs: 1});
DefConstructor("\\fooK", "<ltx:emph>K</ltx:emph>", #{bogusopt: true});
DefEnvironment("{fooE}", "<ltx:emph>#body</ltx:emph>", #{bogusopt: true});
DefMath("\\fooM", "M", #{bogusopt: true});
DefConditional("\\iffooQ", || true, #{bogusopt: true});
"#;
  let tex = "\\documentclass{article}\n\\usepackage{lxbogus}\n\\begin{document}\n\
             \\fooT\\fooP\\fooPn\\fooK\\begin{fooE}e\\end{fooE}\\iffooQ q\\fi\n\\end{document}\n";
  let (log, xml) = convert_files_with(tex, &[("lxbogus.sty.rhai", bogus)], None);
  assert_eq!(error_count(&log), 7, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  for (op, key) in [
    ("DefMacro (\\fooT)", "bogusopt"),
    ("DefPrimitive (\\fooP)", "bogusopt"),
    ("DefPrimitive (\\fooPn)", "nargs"),
    ("DefConstructor (\\fooK)", "bogusopt"),
    ("DefEnvironment ({fooE})", "bogusopt"),
    ("DefMath (\\fooM)", "bogusopt"),
    ("DefConditional (\\iffooQ)", "bogusopt"),
  ] {
    let line = format!("Error:misdefined:{op} {op} does not accept options:{key}");
    assert!(log.lines().any(|l| l == line), "{line}\n{log}");
  }
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p>T<emph>K</emph><emph>e</emph>q</p></para>"#,
  );
}
