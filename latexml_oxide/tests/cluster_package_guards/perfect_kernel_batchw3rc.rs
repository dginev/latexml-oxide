//! W3RC (PR #829 Rhai round C): Number, Dimension, Glue and MuGlue are typed handles with Perl's constructors and
//! methods (Common/Number.pm, Dimension.pm, Glue.pm, Core/MuGlue.pm), and the register calls take and return them
//! (Package.pm DefRegister :1335, LookupRegister :1360, LookupDimension :1373, AssignRegister :1397). Each row is
//! pinned against Perl `latexml` with the equivalent `.ltxml` binding.
#![cfg(feature = "runtime-bindings")]

use latexml::util::test::{assert_element, convert_files_with};

use crate::perfect_kernel_batch46::{error_count, warning_count};

const LXRC: &str = r#"DefRegister("\\probeskip", Glue("1pt plus 2pt"));
DefRegister("\\probedim", Dimension("3pt"));
DefRegister("\\probesp", Dimension(65536));
DefRegister("\\probemu", MuGlue("3mu plus 1mu"));
DefRegister("\\probenum", Number(5));
DefRegister("\\probero", Dimension("2pt"), #{readonly: true});
DefRegister("\\probeget", Dimension(0), #{getter: || Dimension("4pt")});
DefRegister("\\probeset", Dimension(0), #{setter: |v, s| AssignValue("lxrc:set", ToString(v), "global")});
DefRegister("\\sglue", "1pt plus 2pt");
DefRegister("\\smu", "3mu plus 1mu");
DefRegister("\\parskip", "2pt");
DefRegister("\\stoks", Tokenize("ab"));
DefMacro("\\dimarg{Dimension}", |gullet, d| d.add(Dimension("1pt")));
DefMacro("\\showlook", |gullet| {
  let d = LookupRegister("\\probedim"); let g = LookupRegister("\\probeskip");
  let m = LookupRegister("\\probemu"); let n = LookupRegister("\\probenum");
  let parts = [type_of(d), type_of(g), type_of(m), type_of(n),
    d.add(Dimension("1pt")), d - Dimension("1pt"), d * 2, d.divide(2), -d,
    g.add(Glue("1pt plus 1fil")), g.multiply(2), g.negate(),
    m + MuGlue("1mu"), n + Number(2), n.multiply(3),
    d.valueOf(), d.ptValue(), n.valueOf(), g.valueOf(), ToString(g),
    if d > Dimension("2pt") { "gt" } else { "le" },
    LookupDimension("\\probedim"), LookupDimension("5pt"), LookupRegister("\\probeget"),
    Glue("3pt plus 1fil") - Dimension("1pt"), Glue("1pt plus 1fil") - Glue("1pt plus 2fil"),
    MuGlue("3mu plus 1mu") + MuGlue("1mu plus 1fil")];
  let out = "";
  for p in parts { if out != "" { out += " "; } out += `${p}`; }
  Explode(out)
});
DefMacro("\\sib", |gullet| {
  let parts = [type_of(LookupRegister("\\sglue")), type_of(LookupRegister("\\smu")),
    type_of(LookupRegister("\\parskip")), type_of(LookupRegister("\\stoks")), type_of(LookupValue("\\probedim")),
    LookupRegister("\\count", [255]), Equals(Dimension("1pt"), Dimension("1pt")), Equals(Dimension("1pt"), Glue("1pt")),
    !!Dimension(0), LookupRegisterOrDefault("\\nosuchreg")];
  let out = "";
  for p in parts { if out != "" { out += " "; } out += `${p}`; }
  Explode(out)
});
DefPrimitive("\\setit", |stomach| {
  AssignRegister("\\probedim", Dimension("7pt"));
  AssignRegister("\\probenum", Number(9), "global");
  AssignRegister("\\probeskip", Glue("2pt minus 1pt"));
});
DefMacro("\\showset", |gullet| { let v = LookupValue("lxrc:set"); Explode(if defined(v) { v } else { "none" }) });
"#;

const LXRC_TEX: &str = r"\documentclass{article}
\usepackage{lxrc}
\begin{document}
\the\probeskip{} \the\probedim{} \the\probesp{} \the\probemu{} \the\probenum{} \the\probeget

\showlook

{\setit\the\probedim{} \the\probeskip}{} \the\probedim{} \the\probenum{} \the\probeskip

\probeset=6pt \showset{} \probero=5pt \the\probero{} \advance\probeskip by 1pt plus 1fil\relax \the\probeskip

\the\sglue{} \the\smu{} \the\parskip{} \the\stoks{} \dimarg{3pt} \count255=7 \sib
\end{document}
";

/// The clean side (0 Errors; 1 Warning, Perl's, for the readonly `\probero=5pt`), as Perl's `.ltxml` twin prints
/// except where TeX decides. Red on bd0962a211e, which had no `Glue`/`MuGlue`/`Number` constructor and no
/// `LookupRegister` ("Function not found"), and whose `DefRegister("\\probeskip", "1pt plus 2pt")` printed `0.0pt`.
/// p1: each register type defined; a `getter` closure. p2: `LookupRegister`'s typed handles and their arithmetic;
/// Glue + `1pt plus 1fil` is TeX's `2.0pt plus 1.0fil` (Perl's Glue.pm `add` compares the fill names as numbers:
/// `2.0pt plus 3.0pt`), and so are OD #1030's other three, as pdftex's `\advance` prints them (Perl `2.0pt`, `0.0pt`,
/// `4.0mu plus 1.0mu`); `ptValue` is Rhai's float `3.0` (Perl `3`). p3: `AssignRegister`, local and global.
/// p4: a `setter` closure; `\advance` of a Glue register by fil glue. p5: a string value is
/// read for its declared type (Glue, MuGlue; `\parskip` stays Glue); a Tokens register; a `{Dimension}` argument is a
/// Dimension; `LookupValue` of a register's address and `LookupRegister`'s parameters; `Equals`; a handle is true;
/// `LookupRegisterOrDefault` of no register is 0.
#[test]
fn rhai_typed_registers() {
  let (log, xml) = convert_files_with(LXRC_TEX, &[("lxrc.sty.rhai", LXRC)], None);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 1, "{log}");
  assert!(log.contains("Warning:unexpected:\\probero"), "{log}");
  for (id, para) in [
    (
      "p1",
      "1.0pt plus 2.0pt 3.0pt 1.0pt 3.0mu plus 1.0mu 5 4.0pt",
    ),
    (
      "p2",
      "Dimension Glue MuGlue Number 4.0pt 2.0pt 6.0pt 1.5pt -3.0pt 2.0pt plus 1.0fil 2.0pt plus 4.0pt -1.0pt plus \
       -2.0pt 4.0mu plus 1.0mu 7 15 196608 3.0 5 65536 1.0pt plus 2.0pt gt 3.0pt 5.0pt 4.0pt 2.0pt plus 1.0fil 0.0pt \
       plus -1.0fil 4.0mu plus 1.0fil",
    ),
    ("p3", "7.0pt 2.0pt minus 1.0pt 3.0pt 9 1.0pt plus 2.0pt"),
    ("p4", "6.0pt 2.0pt 2.0pt plus 1.0fil"),
    (
      "p5",
      "1.0pt plus 2.0pt 3.0mu plus 1.0mu 2.0pt ab 4.0pt Glue MuGlue Glue Tokens Dimension 7 true false true 0",
    ),
  ] {
    assert_element(
      &xml,
      "para",
      &[&format!("xml:id=\"{id}\"")],
      &format!("<para xml:id=\"{id}\"><p>{para}</p></para>"),
    );
  }
}

const LXRC_ERR: &str = r#"DefRegister("\\probebad", Dimension("3 furlongs"));
DefRegister("\\probebadg", Glue("1pt plus"));
DefRegister("\\probeopt", Dimension("1pt"), #{bogus: 1});
DefRegister("\\probestr", "3 furlongs");
DefRegister("\\probeunit", "5");
DefRegister("\\probeint", Number(1));
DefRegister("\\probeint", "5");
DefMacro("\\look", |gullet| { let v = LookupRegister("\\relax"); AssignRegister("\\relax", Dimension("1pt"));
  let u = LookupRegister("\\MDodotkern");
  Explode(if defined(v) || defined(u) { "def" } else { "undef" }) });
"#;

const LXRC_ERR_TEX: &str = r"\documentclass{article}
\usepackage{lxrcerr}
\begin{document}
\the\probebad{} \the\probebadg{} \the\probeopt{} \the\probestr{} \look{} \the\probeunit{} \the\probeint
\end{document}
";

/// The Error side. A spec the constructor cannot read is an Error and 0 (red on bd0962a211e: the string form read
/// `3 furlongs` as 0.0pt with E=0); Perl reads `Dimension('3 furlongs')` as 3sp (a Perl `isn't numeric` warning) and
/// Warns `Missing Glue specification` for `Glue('1pt plus')`. An option outside Perl's `$register_options` is Perl's
/// `Error:misdefined`. `LookupRegister`/`AssignRegister` of no register Warn as Perl's (3 W) and give `()`, an
/// undefined CS included (mathdots `\MDodotkern`; red on 7b3b2d70f02: no Warning). A fresh register's unitless string
/// is an Error and 0 (red on 7b3b2d70f02: 5sp, E=0; Perl errs at `\the`); an existing register's type wins (5).
#[test]
fn rhai_typed_registers_error_side() {
  let (log, xml) = convert_files_with(LXRC_ERR_TEX, &[("lxrcerr.sty.rhai", LXRC_ERR)], None);
  assert_eq!(error_count(&log), 5, "{log}");
  assert_eq!(warning_count(&log), 3, "{log}");
  for line in [
    "Error:unexpected:3 furlongs Dimension(\"3 furlongs\"): not a Dimension specification; using 0",
    "Error:unexpected:1pt plus Glue(\"1pt plus\"): not a Glue specification; using 0",
    "Error:misdefined:DefRegister (\\probeopt) DefRegister (\\probeopt) does not accept options:bogus",
    "Warning:expected:register The control sequence '\\relax' is not a register",
    "Warning:expected:register The control sequence '\\MDodotkern' is not a register",
    "Error:unexpected:5 DefRegister (\\probeunit): \"5\" has no unit, so no register type; write Number(5) or a \
     unit; using Dimension 0",
  ] {
    assert!(log.contains(line), "{line}\n{log}");
  }
  assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>0.0pt 0.0pt 1.0pt 0.0pt undef 0.0pt 5</p></para>"#,
  );
}
