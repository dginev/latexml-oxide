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
DefPrimitive("\\rsc", || { RefStepCounter("lxrdc"); return; });
DefMacro("\\cv", || `${CounterValue("lxrdc")}`);
DefMacro("\\rid", || RefStepCounter("lxrdd").id);
DefPrimitive("\\unmean", || Let("\\lxa", "\\lxundefined"));
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
/// the `2` follows TeX and KNOWN_PERL_ERRORS #412, not Perl (Perl 0.8.8 prints 1, Package.pm:756-760);
/// `NewCounter`'s `idprefix` (`X1`; was "Function not found").
/// p3: `Let` (was `AssignMeaning`, now its deprecated alias) to an undefined token undefines (U; was a no-op, D); a named scope is not the current one (U;
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

const LXTHIN: &str = r##"RawTeX("\\def\\lxwrap#1{(#1)}\\def\\lxhook{X}\\newcount\\lxc \\lxc=7 \\def\\lxa{A}\\let\\lxb\\lxa");
NewCounter("lxk");
LoadPool("LaTeX");
ProcessKeyOptions();
DeclareFontMap("LXE", ["a", (), "c"], #{family: "lxfam"});
DeclareFontMapMultichar("LXE", #{"128": "ab"});
DeclareFontMapMultichar("LXE", "lxfam", #{"128": "cd"});
AddToMacro("\\lxhook", "Y");
DefMacro("\\inv", || Invocation("\\lxwrap", [Tokenize("B")]));
DefMacro("\\invanon", || Invocation("#1-#2", Tokenize("a"), Tokenize("b")));
DefMacro("\\lxinfo", || {
  let r = [IsEmpty(Tokenize("{}")), IsEmpty(Tokenize("a")), Equals(T_CS("\\lxa"), T_CS("\\lxb")),
    Equals(T_CS("\\lxa"), T_CS("\\relax")), Equals("1", 1), CleanLabel(" ab "), CleanID("x:y"), UTF(65),
    roman(4), Roman(9), Explode("a b"), ExplodeText("cd"), LookupRegisterOrDefault("\\lxc"),
    defined(FindFile("article", #{type: "cls"})), DigestIf("\\lxundefd") == ()];
  let s = ""; for x in r { s += `${x} `; } s
});
DefPrimitive("\\lxset", || { SetCounter("lxk", 5); Input("lxin"); });
DefEnvironment("{lxlist}", "<ltx:itemize>#body</ltx:itemize>",
  #{properties: || BeginItemize("itemize", "@item")});
DefLigature("lxq", "Q!", #{fontTest: |font| font.getFamily() != "typewriter"});
"##;

const LXTHIN_TEX: &str = r"\documentclass{article}
\usepackage{lxthin}
\begin{document}
\inv{} \invanon{} \lxhook

\lxinfo

\lxset\thelxk

\begin{lxlist}\item one\item two\end{lxlist}

lxq \texttt{lxq}
\end{document}
";

/// W3RD1b: the thin wrappers, each "Function not found" on e82b4171a6f (the binding stopped loading at the first,
/// `LoadPool`), and `DefLigature`'s `fontTest`. p1: `Invocation` of a CS and of an anonymous `#1-#2` macro,
/// `AddToMacro`. p2: `IsEmpty` (braces are empty, a letter is not), `Equals` (`\let` to one meaning; not `\relax`;
/// `"1"` and `1` as Perl's string compare), `CleanLabel`, `CleanID`, `UTF`, `roman`/`Roman`, `Explode`/`ExplodeText`,
/// `LookupRegisterOrDefault`, `FindFile`, `DigestIf` of an undefined CS. p3: `Input`, `SetCounter`. The list:
/// `BeginItemize`'s properties number the items. p5: the ligature applies outside typewriter only. `LoadPool`,
/// `ProcessKeyOptions` and the font-map calls run clean (0 E / 0 W).
#[test]
fn rhai_thin_wrappers() {
  let (log, xml) = convert_files_with(
    LXTHIN_TEX,
    &[("lxthin.sty.rhai", LXTHIN), ("lxin.tex", "IN\n")],
    None,
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  for (id, para) in [
    ("p1", r#"<para xml:id="p1"><p>(B) a-b XY</p></para>"#),
    (
      "p2",
      r#"<para xml:id="p2"><p>true false true false true LABEL:ab x..y A iv IX a b cd 7 true true</p></para>"#,
    ),
    ("p3", r#"<para xml:id="p3"><p>IN 5</p></para>"#),
    (
      "p5",
      r#"<para xml:id="p5"><p>Q! <text font="typewriter">lxq</text></p></para>"#,
    ),
  ] {
    assert_element(&xml, "para", &[&format!("xml:id=\"{id}\"")], para);
  }
  assert_element(
    &xml,
    "item",
    &["xml:id=\"S0.I1.i2\""],
    "<item xml:id=\"S0.I1.i2\"><tags><tag>\u{2022}</tag><tag role=\"typerefnum\">2nd item</tag></tags><para \
     xml:id=\"S0.I1.i2.p1\"><p>two</p></para></item>",
  );
}

const LXRD2: &str = r#"DefConstructor("\\lxw{}", "<ltx:text class='#c'>#1</ltx:text>", #{
  afterDigest: |whatsit| whatsit.setProperty("c", "w" + ToString(whatsit.getArg(1)) + (whatsit.getArg(2) == ())),
  afterConstruct: |document, whatsit| document.absorbString("/" + whatsit.getProperty("c") + whatsit.getProperties().c)});
DefMath("\\lxm{}", "M", #{reversion: |whatsit, x| "\\lxm{" + x + ToString(whatsit.getArg(1)) + "}"});
DefConstructor("\\lxo{}", |document, x, props| {
  let n = document.openElement("ltx:text", #{class: props.c});
  n.setAttribute("title", type_of(n));
  document.absorb(x);
  let c = document.closeElement("ltx:text");
  document.absorbString(c.getAttribute("class") + " " + type_of(document.maybeCloseElement("ltx:emph")));
}, #{properties: #{c: "pc"}});
DefPrimitive("\\lxs", || { AssignValue("lx:n", 3); AssignValue("lx:t", true, "global"); Let("\\lxr", "\\relax"); });
DefMacro("\\lxv", || `${LookupNumber("lx:n")} ${LookupBool("lx:t")} ${LookupValue("lx:n") + 1}`);
DefAccent("\\lxacc", "́", "´", #{below: false});
DefPrimitive("\\lxam", || { AssignMeaning(T_CS("\\lxbf"), LookupDefinition("\\textbf"));
  AssignMeaning(T_CS("\\lxch"), T_LETTER("a")); });
DefPrimitive("\\lxag", || AssignMeaning(T_CS("\\lxgl"), T_LETTER("g"), "global"));
DefPrimitive("\\lxun", || AssignMeaning(T_CS("\\lxbf"), LookupDefinition("\\lxnone")));
"#;

const LXRD2_TEX: &str = r"\documentclass{article}
\usepackage{lxrdtwo}
\begin{document}
\lxw{a}

$\lxm{b}$

\lxo{X}

\lxs\lxv{} \ifx\lxr\relax R\fi{} \lxacc{e}

\lxam\lxbf{b} \lxch\ifx\lxch a=\fi{} {\lxag}\lxgl{} \lxun\ifdefined\lxbf D\else U\fi
\end{document}
";

/// W3RD2a (#716): Perl's forms, each "Function not found" or a wrong arity on a1b6c0b827d (the binding stopped at the
/// first). p1: a hook receives the whatsit positionally — `afterDigest` as `|whatsit|` (Perl `($stomach, $whatsit)`,
/// no stomach handle), `afterConstruct` as `|document, whatsit|` (Constructor.pm:139) — with Whatsit.pm's `getArg`
/// (the digested argument; a missing one `()`), `getProperty`, `getProperties`. p2: a reversion closure is Perl's
/// `($whatsit, @args)` (Whatsit.pm:121). p3: `openElement` takes attributes and returns the node, `closeElement` the
/// node closed, `maybeCloseElement` `()` when nothing closes (Document.pm:787-866); a constructor body's trailing
/// argument is the properties map (Perl's `%props`, Constructor.pm:137). p4: `AssignValue` stores a number and a bool
/// (`LookupNumber`/`LookupBool` read them), `Let`, `DefAccent`'s option form (TeX_Character.pool.ltxml:92). p5 (W3RF):
/// `AssignMeaning` is Perl's `$STATE->assignMeaning` (State.pm:380): a `LookupDefinition` handle, a Token, `()`, a
/// scope; Perl and pdflatex "b a= g U", 0 E/W; red on e094f346e9d (E=6, W=1: the old `Let` alias).
#[test]
fn rhai_perl_forms_716() {
  let (log, xml) = convert_files_with(LXRD2_TEX, &[("lxrdtwo.sty.rhai", LXRD2)], None);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  for (id, para) in [
    (
      "p1",
      r#"<para xml:id="p1"><p><text class="watrue">a</text>/watruewatrue</p></para>"#,
    ),
    (
      "p3",
      r#"<para xml:id="p3"><p><text class="pc" title="Node">X</text>pc ()</p></para>"#,
    ),
    ("p4", "<para xml:id=\"p4\"><p>3 true 4 R \u{e9}</p></para>"),
    (
      "p5",
      r#"<para xml:id="p5"><p><text font="bold">b</text> a= g U</p></para>"#,
    ),
  ] {
    assert_element(&xml, "para", &[&format!("xml:id=\"{id}\"")], para);
  }
  assert!(xml.contains(r#"tex="\lxm{bb}""#), "{xml}");
}

const LXRD2_OLD: &str = r#"DefConstructor("\\lxa{}", "<ltx:text class='#c'>#1</ltx:text>", #{
  afterDigest: || { whatsit().setProperty("c", whatsit().argString(1) + whatsit().propertyString("none"));
    whatsit().setProperty("d", whatsit().argString(1)); }});
DefConstructor("\\lxb{}", |document, x| {
  document.openElement("ltx:text"); document.setAttribute(document.getNode(), "class", document.getProperty("c") + document.getProperties().c);
  document.absorb(x); document.closeElement("ltx:text"); }, #{properties: #{c: "q"}});
DefPrimitive("\\lxs", || { assign_value("lx:a", "A"); assign_global("lx:b", "B"); AssignString("lx:s", "S");
  AssignNumber("lx:n", 3); AssignBool("lx:t", true); AssignMeaning("\\lxm", "\\relax"); });
DefMacro("\\lxv", || lookup_value("lx:a") + lookup_value("lx:b") + LookupValue("lx:s") + LookupNumber("lx:n")
  + LookupBool("lx:t"));
DefAccent("\\lxacc", "́", "´", false);
DefAccent("\\lxbad", "́", "´", #{bogus: true});
DefMacro("\\lxom{}", |x| "(" + x + x.len() + ")");
DefPrimitive("\\lxop{}", |v| AssignValue("lx:op", v + "!", "global"));
DefMacro("\\lxopv", || LookupValue("lx:op"));
DefMath("\\lxor{}", "R", #{reversion: |x| "\\lxor{" + x + "}"});
"#;

const LXRD2_OLD_TEX: &str = r"\documentclass{article}
\usepackage{lxrdold}
\begin{document}
\lxa{a}\lxa{b} \lxb{c}\lxb{d}

\lxs\lxs\lxv{} \ifx\lxm\relax R\fi{} \lxacc{e}

\lxom{ab}\lxom{c} \lxop{v}\lxopv{} $\lxor{q}$
\end{document}
";

/// W3RD2a: the names #716 replaces stay one release as aliases; each Warns `deprecated` once per binding load, naming
/// its replacement, however often it runs (here twice), and keeps its old result. `DefAccent` reads no option, as Perl's (TeX_Character.pool.ltxml:92-100). Red on a1b6c0b827d: no
/// Warning (W=0), and `#{bogus: true}` "Function not found". The space before `\lxb` is dropped as Perl drops it before
/// an imperative constructor's `openElement` (RED `rhai-bindings/imperative_constructor_drops_space_red.tex`). W3RD2b: a macro
/// or primitive body declaring 1..=nargs parameters, no leading gullet/stomach, gets its arguments as strings, Warned
/// with the fix; so does an old-form reversion `|x|` (red on 8ee92a55a49: W=13, the
/// reversion `tex="\lxor{Whatsit}"`-garbage; on a1b6c0b827d E=1 for `#{bogus: true}`). W3RF: `AssignMeaning` is Perl's
/// again; only its string meaning (the old `Let` form) is the alias.
#[test]
fn rhai_deprecated_aliases_warn_once() {
  let (log, xml) = convert_files_with(LXRD2_OLD_TEX, &[("lxrdold.sty.rhai", LXRD2_OLD)], None);
  assert_eq!(error_count(&log), 0, "{log}");
  let deprecated = [
    ("whatsit", "the hook's whatsit argument"),
    ("whatsit.argString", "whatsit.getArg(n)"),
    ("whatsit.propertyString", "whatsit.getProperty(key)"),
    (
      "document.getProperty",
      "the constructor body's trailing properties map",
    ),
    (
      "document.getProperties",
      "the constructor body's trailing properties map",
    ),
    ("assign_value", "AssignValue(key, value, \"local\")"),
    ("assign_global", "AssignValue(key, value, \"global\")"),
    ("lookup_value", "LookupValue(key)"),
    ("AssignString", "AssignValue(key, value[, scope])"),
    ("AssignNumber", "AssignValue(key, value[, scope])"),
    ("AssignBool", "AssignValue(key, value[, scope])"),
    (
      "AssignMeaning(cs, string)",
      "Let(cs, other[, scope]), or AssignMeaning(T_CS(cs), LookupDefinition(other)[, scope])",
    ),
    (
      "DefAccent",
      "DefAccent(accent, combining, standalone, #{below: true})",
    ),
    (
      "DefMacro |args|",
      "|gullet, args|, the arguments as Tokens (name each: |gullet, _x|)",
    ),
    (
      "DefPrimitive |args|",
      "|stomach, args|, the arguments as Tokens (name each: |stomach, _x|)",
    ),
    (
      "reversion |args|",
      "|whatsit, args| (name each: |whatsit, _x|)",
    ),
  ];
  assert_eq!(warning_count(&log), deprecated.len(), "{log}");
  for (old, new) in deprecated {
    let line = format!("Warning:deprecated:{old} {old} is deprecated; use {new}");
    assert_eq!(log.matches(&line).count(), 1, "{line}\n{log}");
  }
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p><text class="a">a</text><text class="b">b</text><text class="qq">c</text><text class="qq">d</text></p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p2\""],
    "<para xml:id=\"p2\"><p>ABS3true R \u{e9}</p></para>",
  );
  assert!(xml.contains("<p>(ab2)(c1) v! <Math"), "{xml}");
  assert!(xml.contains(r#"tex="\lxor{q}""#), "{xml}");
  // W3RD2bF (revW3i B1): the old test of a GetKeyVal value against a string (`w != ""`, the base's mykeyval.sty.rhai)
  // compares string forms, `()` as "", each Warned; red on 34740a129a8: a stray "Width: " for the omitted key, W=0.
  // W3RD2bF2 (revW3j B1, DG ruling 9): `()` equals "" in both orders, as Perl's `undef eq ''` (Perl Warns
  // `uninitialized`): `"" != w` and `s != ()` are false (red on dd79f1fc1b2: "R:" and "U2[]" by the left-only rule,
  // `defined` "Function not found"); `defined(v)` is Perl's defined test, true for `""`.
  let kv = include_str!("../keyval_rhai/mykeyval.sty.rhai");
  let old = kv
    .replace("if defined(w) {", "if w != \"\" && w != \"false\" {")
    .replace("if defined(h) {", "if h != \"\" && h != \"false\" {")
    + LXU;
  assert_eq!(old.matches("!= \"false\"").count(), 2);
  let (log, xml) = convert_files_with(
    "\\documentclass{article}\\usepackage{mykeyval}\\begin{document}\\begin{KVstruct}[height=2]X\\end{KVstruct}\n\n\
     \\lxuA{x} \\lxuA[width=2cm]{y} \\lxuB{}\\end{document}",
    &[("mykeyval.sty.rhai", &old)],
    None,
  );
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 2, "{log}");
  for line in [
    "Warning:deprecated:() == string () == string is deprecated; use defined(value) or !!value",
    "Warning:deprecated:value == string value == string is deprecated; use ToString(value) == string",
  ] {
    assert!(log.contains(line), "{line}\n{log}");
  }
  assert!(xml.contains("<p>&amp;Height: 2X</p>"), "{xml}");
  assert!(xml.contains("<p>E;xR:2cmL:2cmD;yDefined: []</p>"), "{xml}");
}

/// The p2/p3 shapes of revW3j: `()` against a string on either side, `defined`.
const LXU: &str = r#"
DefConstructor("\\lxuA OptionalKeyVals:foo {}", |document, kv, b| {
  let w = GetKeyVal(kv, "width");
  if "" != w { document.absorbString("R:" + w); }
  if w != "" { document.absorbString("L:" + w); }
  if "" == w && w == "" && !defined(w) { document.absorbString("E;"); }
  if defined(w) { document.absorbString("D;"); }
  document.absorbString(b);
});
DefConstructor("\\lxuB{}", |document, a| {
  let s = ToString(a);
  if () != s { document.absorbString("U[" + s + "]"); }
  if s != () { document.absorbString("U2[" + s + "]"); }
  if defined(s) && !defined(()) { document.absorbString("Defined: [" + s + "]"); }
});
"#;

const LXRD3: &str = r#"DefMacro("\\lxpt{}", |gullet, a| type_of(a) + "/" + type_of(gullet));
DefMacro("\\lxpa{}{}", |gullet, a, b| Tokens(b, a));
DefMacro("\\lxpg", |gullet| Tokens(T_CS("\\textbf"), T_BEGIN(), gullet.readArg(), T_END()));
DefPrimitive("\\lxpp{}", |stomach, a| AssignValue("lx:pp", Tokens(a, stomach.getGullet().readArg()), "global"));
DefMacro("\\lxppv", || LookupValue("lx:pp"));
DefKeyVal("LXK", "k", "");
DefMacro("\\lxkv OptionalKeyVals:LXK {}", |gullet, kv, body|
  "(" + GetKeyVal(kv, "k") + "/" + GetKeyVals(kv).k.len() + "/" + type_of(kv) + "/" + GetKeyVal(kv, "z") + ")" + body);
DefConstructor("\\lxkc RequiredKeyVals:LXK", |document, kv| {
  document.absorbString(GetKeyVal(kv, "k")); document.absorbString("/" + type_of(GetKeyVal(kv, "k")));
  document.absorbString(GetKeyVal(kv, "z")); });
DefParameterType("LxUntil", |gullet, delim| gullet.readUntil(delim));
DefMacro("\\lxpu LxUntil:!", |gullet, a| "[" + a + "]");
DefParameterType("LxNone", |gullet| (), #{optional: true});
DefMacro("\\lxpn LxNone", |gullet, a| if a == () { "N" } else { "Y" });
DefParameterType("LxSemi", |gullet| gullet.readArg(), #{semiverbatim: true});
DefMacro("\\lxps LxSemi", |gullet, a| Tokens(T_CS("\\texttt"), T_BEGIN(), a, T_END()));
DefParameterType("LxUnd", |gullet| gullet.readArg(), #{undigested: true});
DefConstructor("\\lxpc LxUnd", |document, a| document.absorbString(type_of(a)));
DefMacro("\\lxt OptionalKeyVals:LXK", |gullet, kv| if !!GetKeyVal(kv, "k") { "T" } else { "F" });
DefMacro("\\lxtv", || { let s = ""; for v in ["", "0", 0, 0.0, "a", Tokenize(""), T_OTHER("0"), [], #{}, ()] {
  s += if !v { "F" } else { "T" }; } s });
DefMacro("\\lxz{}", || "Z");
DefPrimitive("\\lxzp{}", || AssignValue("lx:z", "P", "global"));
DefMacro("\\lxzv", || LookupValue("lx:z"));
DefColumnType("Q{}", || "c");
DefConstructor("\\lxws{}", "<ltx:text class='#c'>#1</ltx:text>", #{afterDigest: |whatsit| whatsit.setProperty("c", "s" + whatsit),
  afterConstruct: |document, whatsit| document.absorbString("/" + whatsit + ToString(whatsit))});
DefParameterType("LxN", |gullet| gullet.readArg(), #{nargs: 1});
DefMacro("\\lxpq LxN", |gullet, a| a);
"#;

const LXRD3_TEX: &str = r"\documentclass{article}
\usepackage{lxrdthree}
\begin{document}
\lxpt{a} \lxpa{x}{y} \lxpg{B} \lxpp{P}{Q}\lxppv

\lxkv[k=v1,k=v2]{b} \lxkc{k=w}

\lxpu ab!c \lxpn{} \lxps{a_b} \lxpc{u}

\lxt{} \lxt[k=] \lxt[k=0] \lxt[k=v] \lxtv

\lxz{a} \lxzp{b}\lxzv{} \begin{tabular}{Q{x}}t\end{tabular} \lxws{w} \lxpq{N}
\end{document}
";

const LXRD3_ERR: &str = r#"DefParameterType("LxReq", |gullet| ());
DefMacro("\\lxpr LxReq", |gullet, a| "R");
DefParameterType("LxBad", |gullet| gullet.readArg(), #{bogus: 1});
DefMath("\\lxself{}", "S", #{reversion: |whatsit, x| "\\lxself{" + whatsit + "}"});
"#;

/// W3RD2b: a body gets Perl's arguments — a macro `($gullet, @args)` (Expandable.pm:74), a primitive `($stomach,
/// @args)` (Primitive.pm:78), each argument Tokens or a `KeyVals`; `GetKeyVal`/`GetKeyVals` return the value (Tokens
/// read, a box digested; `()` missing; KeyVals.pm:31-37); `DefParameterType`'s reader gets the gullet and the type's
/// extra (Parameter.pm:88) with Perl's options. Red on 8ee92a55a49: arguments were strings, `Stomach`/`getGullet`,
/// `DefParameterType` "Function not found". Error side: a required type whose reader returns `()` (Perl
/// Parameter.pm:94 `Error:expected`) and an option outside `$parameter_options` (`Error:misdefined`), 1 each in Perl.
/// W3RD2bF (revW3i): p4 `!!v`/`!v` are Perl's `if ($v)` — undef, "", "0", 0 false, every object true, so a set key
/// even empty or `0` (Perl `latexml`: the same F T T T, FFFFTTTTTF; on 34740a129a8 "Function not found: !"). p5: a
/// body declaring no parameter on a definition with arguments is Perl's `sub {…}`, unwarned (macro, primitive, column
/// type); a whatsit in a string is its reversion, as `ToString($whatsit)` (was the first argument, Warned
/// `deprecated:reversion`, and `ToString` the type name); `DefParameterType`'s `nargs` is accepted unread (was
/// Warned unsupported) — W=5 on 34740a129a8.
#[test]
fn rhai_closure_args_and_parameter_types() {
  let (log, xml) = convert_files_with(LXRD3_TEX, &[("lxrdthree.sty.rhai", LXRD3)], None);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  for (id, para) in [
    (
      "p1",
      r#"<para xml:id="p1"><p>Tokens/Gullet yx <text font="bold">B</text> PQ</p></para>"#,
    ),
    (
      "p2",
      r#"<para xml:id="p2"><p>(v2/2/KeyVals/)b w/Digested</p></para>"#,
    ),
    (
      "p3",
      r#"<para xml:id="p3"><p>[ab]c N <text font="typewriter">a_b</text> Tokens</p></para>"#,
    ),
    (
      "p4",
      r#"<para xml:id="p4"><p>F T T T FFFFTTTTTF</p></para>"#,
    ),
    (
      "p5",
      r#"<para xml:id="p5"><p>Z P <tabular vattach="middle"><tbody><tr><td align="center">t</td></tr></tbody></tabular> <text class="slxwsw">w</text>/\lxws{w}\lxws{w} N</p></para>"#,
    ),
  ] {
    assert_element(&xml, "para", &[&format!("xml:id=\"{id}\"")], para);
  }
  let (log, xml) = convert_files_with(
    "\\documentclass{article}\\usepackage{lxrderr}\\begin{document}\\lxpr $\\lxself{c}$\\end{document}",
    &[("lxrderr.sty.rhai", LXRD3_ERR)],
    None,
  );
  // W3RD2bF: a reversion printing its own whatsit, which Perl reverts without end, is an Error per reversion (the
  // math's `tex` and its parse), the whatsit printing "" (p5 is the safe side: a whatsit in any other hook).
  assert_eq!(error_count(&log), 4, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_eq!(
    log
      .matches("Error:script:reversion a reversion's own whatsit in a string reverts it again")
      .count(),
    2,
    "{log}"
  );
  assert!(xml.contains(r#"tex="\lxself{}""#), "{xml}");
  assert!(log.contains("Error:expected:LxReq"), "{log}");
  assert!(
    log.contains("Error:misdefined:DefParameterType LxBad"),
    "{log}"
  );
  assert!(xml.contains("<p>R<Math"), "{xml}");
}
