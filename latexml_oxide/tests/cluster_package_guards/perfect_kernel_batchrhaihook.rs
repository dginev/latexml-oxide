//! RHAIHOOK: a Rhai hook body takes Perl's arguments (DG 2026-11-18, Rhai mature in v0.8.0): `beforeDigest` and
//! `beforeDigestEnd` `($stomach)`, a constructor's `afterDigest`/`afterDigestBegin`/`afterDigestBody`
//! `($stomach, $whatsit)`, `properties` `($stomach, @args)` (Constructor.pm:99-117), a primitive's `afterDigest`
//! `($stomach)` (Primitive.pm:79), a DefMath's `afterDigest` `($stomach, $whatsit)` (a Constructor: Package.pm:1711).
//! The repro `rhai-bindings/afterdigest_stomach_first` (red: `setProperty ((), …)`).
#![cfg(feature = "runtime-bindings")]

use latexml::util::test::{assert_element, convert_files_with};

use crate::perfect_kernel_batch46::{error_count, warning_count};

/// Every hook in the Perl form (each paragraph opens with a letter: a Rhai constructor in vertical mode drops the
/// space after it, a separate gap); each records its handles' types, or sets the property its template reads.
const LXH_PERL: &str = r#"DefConstructor("\\hA{}", "<ltx:text class='#c'>#1</ltx:text>", #{
  beforeDigest: |stomach| AssignValue("hk:bd", type_of(stomach), "global"),
  properties: |stomach, x| #{ k: "k-" + ToString(x) + "-" + type_of(stomach) },
  afterDigest: |stomach, whatsit|
    whatsit.setProperty("c", type_of(stomach) + "-" + ToString(whatsit.getArg(1)) + "-" + whatsit.getProperty("k"))
});
DefEnvironment("{hE}{}", "<ltx:text class='#c'>#body</ltx:text>", #{
  afterDigestBegin: |stomach, whatsit| whatsit.setProperty("c", "b-" + ToString(whatsit.getArg(1))),
  afterDigestBody: |stomach, whatsit|
    whatsit.setProperty("c", whatsit.getProperty("c") + "-body-" + type_of(stomach)),
  beforeDigestEnd: |stomach| AssignValue("hk:bde", type_of(stomach), "global"),
  afterDigest: |stomach, whatsit| AssignValue("hk:ad", type_of(stomach) + "+" + type_of(whatsit), "global")
});
DefPrimitive("\\hP", |stomach| { return; }, #{
  afterDigest: |stomach| AssignValue("hk:pad", type_of(stomach), "global")
});
DefMath("\\hM{}", "hM", #{
  afterDigest: |stomach, whatsit| AssignValue("hk:mad", type_of(stomach) + "+" + type_of(whatsit), "global")
});
DefConstructor("\\hQ{}", "<ltx:text class='#c'>#1</ltx:text>");
LookupDefinition("\\hQ").pushAfterDigest(|stomach, whatsit| whatsit.setProperty("c", "q-" + ToString(whatsit.getArg(1))));
DefMacro("\\hshow", |gullet| ToString(LookupValue("hk:bd")) + " " + ToString(LookupValue("hk:bde")) + " "
  + ToString(LookupValue("hk:ad")) + " " + ToString(LookupValue("hk:pad")) + " " + ToString(LookupValue("hk:mad")));
"#;

const LXH_PERL_TEX: &str = r"\documentclass{article}
\usepackage{lxhp}
\begin{document}
A \hA{x} \begin{hE}{y}z\end{hE} \hP\hQ{w}

\setbox0\hbox{$\hM{v}$}\hshow
\end{document}
";

/// The Perl form of every hook option Perl passes a stomach: no deprecation Warning, the whatsit and arguments in
/// their Perl places. Red on the base: `Error:script:afterDigest Function not found: setProperty ((), …)` for each
/// two-parameter hook, the `properties` argument in the stomach slot (`class="k--"`); round 0 gave DefMath's
/// `afterDigest` the stomach alone (`Stomach+()`).
#[test]
fn rhai_hooks_take_perls_arguments() {
  let (log, xml) = convert_files_with(LXH_PERL_TEX, &[("lxhp.sty.rhai", LXH_PERL)], None);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p>A <text class="Stomach-x-k-x-Stomach">x</text> <text class="b-y-body-Stomach">z</text> <text class="q-w">w</text></p></para>"#,
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p2\""],
    r#"<para xml:id="p2"><p>Stomach Stomach Stomach+Whatsit Stomach Stomach+Whatsit</p></para>"#,
  );
}

/// The old forms, each used twice: the whatsit alone, the arguments alone as TeX strings.
const LXH_OLD: &str = r#"DefConstructor("\\oA{}", "<ltx:text class='#c'>#1</ltx:text>", #{
  properties: |x| #{ k: "k-" + x },
  afterDigest: |whatsit| whatsit.setProperty("c", "a-" + ToString(whatsit.getArg(1)) + "-" + whatsit.getProperty("k"))
});
DefEnvironment("{oE}{}", "<ltx:text class='#c'>#body</ltx:text>", #{
  afterDigestBegin: |whatsit| whatsit.setProperty("c", "b-" + ToString(whatsit.getArg(1))),
  afterDigestBody: |whatsit| whatsit.setProperty("c", whatsit.getProperty("c") + "-body")
});
DefEnvironment("{eE}", "<ltx:text class='e'>#body</ltx:text>", #{
  afterDigestBody: |stomach, whatsit| { throw "body trouble"; },
  beforeDigestEnd: |stomach| { throw "end trouble"; }
});
DefMath("\\oM", "oM", #{ meaning: "om", afterDigest: |whatsit| whatsit.setProperty("role", "FUNCTION") });
"#;

const LXH_OLD_TEX: &str = r"\documentclass{article}
\usepackage{lxho}
\begin{document}
A \oA{x}\oA{y} \begin{oE}{u}z\end{oE}\begin{oE}{v}t\end{oE} \begin{eE}s\end{eE}

$\oM$
\end{document}
";

/// The old one-parameter forms keep working this release, each Warned `deprecated` once per binding load (four
/// Warnings for eight calls), naming the Perl form; a failing hook's `Error:script` names that hook (both were
/// labelled `afterDigest`/`beforeDigest` on the base). DefMath's `|whatsit|` (a Constructor in Perl) gets the whatsit
/// (round 0: `Error:script … setProperty (Stomach, …)` and `role="UNKNOWN"`).
#[test]
fn rhai_old_hook_forms_warn_deprecated_once() {
  let (log, xml) = convert_files_with(LXH_OLD_TEX, &[("lxho.sty.rhai", LXH_OLD)], None);
  assert_eq!(error_count(&log), 2, "{log}");
  assert_eq!(warning_count(&log), 4, "{log}");
  for (old, new) in [
    ("afterDigest |whatsit|", "|stomach, whatsit|"),
    ("afterDigestBegin |whatsit|", "|stomach, whatsit|"),
    ("afterDigestBody |whatsit|", "|stomach, whatsit|"),
    ("properties |args|", "|stomach, args|"),
  ] {
    let line = format!("Warning:deprecated:{old} {old} is deprecated; use {new}");
    assert_eq!(log.matches(&line).count(), 1, "{line}\n{log}");
  }
  assert!(log.contains("Error:script:afterDigestBody"), "{log}");
  assert!(log.contains("Error:script:beforeDigestEnd"), "{log}");
  assert!(
    xml.contains(r#"<XMTok meaning="om" role="FUNCTION">oM</XMTok>"#),
    "{xml}"
  );
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p>A <text class="a-x-k-x">x</text><text class="a-y-k-y">y</text> <text class="b-u-body">z</text><text class="b-v-body">t</text> <text class="e">s</text></p></para>"#,
  );
}

/// Two bindings each using two old forms: each binding Warns each alias once (round 0 keyed a digestion-time
/// Warning by the running script, none outside a body, so the second binding's was swallowed).
#[test]
fn rhai_deprecated_hook_forms_warn_once_per_binding() {
  let lxa = r#"DefConstructor("\\ta{}", "<ltx:text class='#c'>#1</ltx:text>", #{ afterDigest: |w| w.setProperty("c", "ta") });
DefMacro("\\ma{}", |x| "MA" + x);"#;
  let lxb = r#"DefConstructor("\\tb{}", "<ltx:text class='#c'>#1</ltx:text>", #{ afterDigest: |w| w.setProperty("c", "tb") });
DefMacro("\\mb{}", |x| "MB" + x);"#;
  let tex = r"\documentclass{article}
\usepackage{lxa}
\usepackage{lxb}
\begin{document}
\ta{x} \tb{y} \ma{1} \mb{2}
\end{document}
";
  let (log, xml) = convert_files_with(tex, &[("lxa.sty.rhai", lxa), ("lxb.sty.rhai", lxb)], None);
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 4, "{log}");
  for old in ["afterDigest |whatsit|", "DefMacro |args|"] {
    let line = format!("Warning:deprecated:{old} {old} is deprecated");
    assert_eq!(log.matches(&line).count(), 2, "{line}\n{log}");
  }
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    r#"<para xml:id="p1"><p><text class="ta">x</text><text class="tb">y</text>MA1 MB2</p></para>"#,
  );
}
