//! Red/green guards for batch LOOPFIX, run 397's alias-cycle loop Fatals (OXIDIZED_DESIGN_DIVERGENCES #1100): a binding
//! that called another user-level command by name, or aliased two unconditionally, where the real package keeps them
//! apart, so a class's or paper's own `\let`/`\def` closed a loop. Repros `tools/perfect_kernel/repros/package-bindings/
//! alias_cycle_*`. Each guard is one binding's code path: a delegation through a saved internal (apacite, slashed), an
//! alias under its package's test (amsmath), a dispatch kept to its own load (hyperref under revtex), `\protected`
//! commands (physics).
use latexml::util::test::convert_with;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The counts `(errors, warnings)` of `log`, which must hold no Fatal.
fn counts(log: &str) -> (usize, usize) {
  assert!(!log.contains("Fatal:"), "{log}");
  (error_count(log), warning_count(log))
}

/// apacite's citation forms call natbib's private copies (`\lx@natbib@citep`, …), so agujournal2018.cls:1140-1143's
/// `\let\cite\shortcite \let\citep\cite \let\citet\citeA` is no cycle (1909.02387, 2212.08776: 180 s timeouts).
#[test]
fn apacite_forms_survive_the_class_lets() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/alias_cycle_apacite_shortcite_let_citep.tex"
    ),
    None,
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  let p = concat!(
    r#"<p>Text <cite class="ltx_citemacro_citep">(<bibref bibrefs="k" separator=";" show="AuthorsPhrase1Year" "#,
    r#"yyseparator=",">"#,
    "\n          <bibrefphrase>, </bibrefphrase>\n        </bibref>)</cite> and ",
    r#"<cite class="ltx_citemacro_citet"><bibref bibrefs="k" separator=";" show="Authors Phrase1YearPhrase2" "#,
    r#"yyseparator=",">"#,
    "\n          <bibrefphrase>(</bibrefphrase>\n          <bibrefphrase>)</bibrefphrase>\n",
    "        </bibref></cite>.</p>"
  );
  assert!(xml.contains(p), "{xml}");
}

/// amsmath.sty:318-345 lets `\sum@` and its eleven siblings only while `\coprod` is still a `\mathchar`: an inlined
/// AmS-TeX `\def\sum{\DOTSB\sum@\slimits@}` keeps its own `\sum@` (math0612243, 0705.4126, math0112238: PushbackLimit).
#[test]
fn amsmath_keeps_an_inlined_amstex_sum() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/alias_cycle_amsmath_inlined_amstex_sum.tex"
    ),
    None,
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  let sum = concat!(
    "<XMApp>\n              <XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/>\n",
    "              <XMTok mathstyle=\"text\" meaning=\"sum\" name=\"sum@\" role=\"SUMOP\" scriptpos=\"post1\">∑</XMTok>\n",
    "              <XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">i</XMTok>\n            </XMApp>"
  );
  assert!(xml.contains(sum), "{xml}");
}

/// The revtex classes' hyperref preload leaves the kernel's `\refstepcounter` (its dispatch through `\H@refstepcounter`
/// is the document's own hyperref load's), so the shipped revtex 4.0 ltxutil.sty:682 `\let\H@refstepcounter
/// \refstepcounter` is no cycle (astro-ph0112270). The cleveref witnesses that need the dispatch load hyperref
/// themselves (2403.15855, 2207.13707).
#[test]
fn revtex_preload_leaves_ltxutil_refstepcounter_alone() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/alias_cycle_revtex_ltxutil_h_refstepcounter.tex"
    ),
    None,
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  let section = concat!(
    "<section inlist=\"toc\" xml:id=\"S1\">\n    <tags>\n      <tag>I.</tag>\n",
    "      <tag role=\"autoref\">section\u{a0}I</tag>\n      <tag role=\"refnum\">I</tag>\n",
    "      <tag role=\"typerefnum\">§I</tag>\n    </tags>\n",
    "    <title><tag close=\" \">I.</tag>A</title>\n    <toctitle><tag close=\" \">I</tag>A</toctitle>\n",
    "    <para xml:id=\"S1.p1\">\n      <p>x</p>\n    </para>\n  </section>"
  );
  assert!(xml.contains(section), "{xml}");
}

/// physics.sty's `\DeclareDocumentCommand`s are `\protected`, so `\pdv` in ifthen's `\equal` edef (algpseudocode's
/// `\Function`) stays itself, where it re-expanded inside its own helper (2001.08539, 2505.01628: PushbackLimit).
/// Slashed's `\lx@slashed` calls the kernel's `\not` saved at load, so an author's `\def\not#1{\slashed{#1}}` is no
/// cycle (2504.21590).
#[test]
fn physics_commands_are_protected_in_an_edef() {
  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/alias_cycle_physics_pdv_in_edef.tex"
    ),
    None,
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  let head = concat!(
    r#"<text font="bold">function</text> <text font="smallcaps">augment</text>(<Math mode="inline" "#,
    r#"tex="\partialderivative{x}{y}" text="(partial-derivative@(y))@(x)" xml:id="algx1.l1.m1">"#,
    "\n          <XMath>\n            <XMDual>\n              <XMApp>\n                <XMApp>\n",
    "                  <XMTok meaning=\"partial-derivative\"/>"
  );
  assert!(xml.contains(head), "{xml}");
  assert!(
    xml.contains("<text font=\"bold\">end</text> <text font=\"bold\">function</text>"),
    "{xml}"
  );

  let (log, xml) = convert_with(
    include_str!(
      "../../../tools/perfect_kernel/repros/package-bindings/alias_cycle_slashed_author_not.tex"
    ),
    None,
  );
  assert_eq!(counts(&log), (0, 0), "{log}");
  let p = concat!(
    r#"<p><Math mode="inline" tex="\not{p}" text="not-p" xml:id="p1.m1">"#,
    "\n        <XMath>\n          <XMTok font=\"italic\" name=\"not-p\" role=\"UNKNOWN\">p̸</XMTok>\n",
    "        </XMath>\n      </Math></p>"
  );
  assert!(xml.contains(p), "{xml}");
}
