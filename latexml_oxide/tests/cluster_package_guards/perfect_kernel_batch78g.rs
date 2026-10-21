//! Red/green guards for batch 78g (kernel: a label after a raw counter step names the object the step numbers). A
//! `\refstepcounter` no constructor takes — tcolorbox's in the phantom code it places in the frame picture of a
//! numbered box (tcolorbox.sty:2311, :1154-1161, :1925), a `\put`'s in a `picture` — gives the labels after it its
//! number, as LaTeX's `\@currentlabel` does (latex.ltx:14963-14967): the label goes to the box (the picture may carry
//! it, OXIDIZED_DESIGN_DIVERGENCES #813) or to the listing line it stands in, which takes the step's number, and
//! `\ref` prints pdflatex's number and links to the box. Perl leaves every such label on the enclosing section
//! (KNOWN_PERL_ERRORS #1038). Each guard converts its repro under `tools/perfect_kernel/repros/cross-references/` to
//! HTML (the reference text is the post cross-referencer's, CrossRef.pm:541) and asserts no Error or Warning and the
//! referencing paragraph whole. Witnesses 2505.20088, 2311.09542, 2509.17784, 2602.03411, 2606.06976
//! (`\newtcolorbox[auto counter]`), 2402.06608 (`[blend into=figures]`), 2506.01072 (`\newtcbtheorem`), 2609.20563
//! (`\newtcblisting`).
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/cross-references/",
      $path
    ))
  };
}

/// Convert `tex` to HTML in a directory of its own with the ar5iv preload: the HTML, the conversion free of errors
/// and warnings.
fn html_of(tex: &str) -> String {
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let output = std::process::Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=170",
      "--preload=ar5iv.sty",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  std::fs::read_to_string(workdir.path().join("t.html")).expect("read t.html")
}

/// A reference `<a>` to `#target` printing `text`, with its tooltip `title`.
fn link(target: &str, title: &str, text: &str) -> String {
  format!(
    r##"<a href="#{target}" title="{title}" class="ltx_ref"><span class="ltx_text ltx_ref_tag">{text}</span></a>"##
  )
}

/// A box drawn as a picture, by its id: the `svg` the reference links to.
fn box_svg(html: &str, id: &str) {
  assert!(
    html.contains(&format!(r#"<svg id="{id}" class="ltx_picture""#)),
    "no picture {id} to link to"
  );
}

/// tcolorbox's numbered boxes take their labels and their counter's number: a `\newtcbtheorem` "1", one `[number
/// within=section]` "2.1", a `\newtcolorbox[auto counter]` with `label=` "1" — each a link to its box — while an
/// uncounted `{tcolorbox}[label=…]` keeps the section's "2" (no step: `\@currentlabel` is the section's), as pdflatex
/// prints them. Red (int47a, Perl the same): "See 2, 2, 2 and 2.", all four links to the section. Repro
/// tcb_box_label_takes_the_box_number.
#[test]
fn tcb_box_label_takes_the_box_number() {
  let html = html_of(repro!("tcb_box_label_takes_the_box_number.tex"));
  assert_element(
    &html,
    "p",
    &[r#"id="S2.p5.1""#],
    &format!(
      r#"<p id="S2.p5.1" class="ltx_p">See {}, {}, {} and {}.</p>"#,
      link("S2.p1.pic1", "In 2 Two", "1"),
      link("S2.p2.pic1", "In 2 Two", "2.1"),
      link("S2.p3.pic1", "In 2 Two", "1"),
      link("S2", "2 Two", "2"),
    ),
  );
  for id in ["S2.p1.pic1", "S2.p2.pic1", "S2.p3.pic1"] {
    box_svg(&html, id);
  }
}

/// A delegated listing box (`\newtcblisting[auto counter]`, `\newtcbinputlisting[auto counter]`: the binding's
/// listings, tcblistingscore_code_tex.rs) labels its listing's first line, which takes the box counter's number: "1
/// and 1", as pdflatex prints them. Red (int47a): "See 2 and 2." (the section's number; Perl "2 and 2" with 3 errors).
/// Repro tcb_listing_box_label_takes_the_box_number.
#[test]
fn tcb_listing_box_label_takes_the_box_number() {
  let html = html_of(repro!("tcb_listing_box_label_takes_the_box_number.tex"));
  assert_element(
    &html,
    "p",
    &[r#"id="S2.p1.1""#],
    &format!(
      r#"<p id="S2.p1.1" class="ltx_p">See {} and {}.</p>"#,
      link("lstnumberx1", "In 2 Two", "1"),
      link("lstnumberx2", "In 2 Two", "1"),
    ),
  );
}

/// The mechanism is LaTeX's, not tcolorbox's: a `\refstepcounter` and a `\label` in a `\put` of a `picture` label the
/// picture with the step's number "1"; a `\label` in the next `\put` (a group: the step stayed in its own) and one in
/// a picture without a step keep the section's "2", as pdflatex prints them. The labelled picture stays the picture
/// around its SVG drawing, with its id. Red (int47a, Perl the same): "See 2, 2 and 2.". Repro
/// picture_step_label_takes_the_step_number.
#[test]
fn picture_step_label_takes_the_step_number() {
  let html = html_of(repro!("picture_step_label_takes_the_step_number.tex"));
  assert_element(
    &html,
    "p",
    &[r#"id="S2.p2.1""#],
    &format!(
      r#"<p id="S2.p2.1" class="ltx_p">See {}, {} and {}.</p>"#,
      link("S2.p1.pic1", "In 2 Two", "1"),
      link("S2", "2 Two", "2"),
      link("S2", "2 Two", "2"),
    ),
  );
  box_svg(&html, "S2.p1.pic1");
}

/// Under cleveref, tcolorbox labels a `crefname`d box with its type (`\label[mytheo]{…}`, tcolorbox.sty:2270,
/// :955-968): the box takes its number, then the type's names, so `\cref` prints "theorem 1.1" (cleveref's `~` a
/// no-break space) and `\autoref` "1.1" (no autoref name for the box counter), as pdflatex prints them; the section
/// keeps its own "section" names. Red
/// (int47a): "Theorem 1, section 1, theorem 1" — the label on the section, whose cleveref names it overwrote (Perl
/// "1, section 1, section 1"). Repro tcb_theorem_cref_names_the_box_type.
#[test]
fn tcb_theorem_cref_names_the_box_type() {
  let html = html_of(repro!("tcb_theorem_cref_names_the_box_type.tex"));
  assert_element(
    &html,
    "p",
    &[r#"id="S1.p2.1""#],
    concat!(
      r##"<p id="S1.p2.1" class="ltx_p">See Theorem <a href="#S1.p1.pic1" title="Theorem 1.1 ‣ 1 Intro" class="ltx_ref">"##,
      r##"<span class="ltx_text ltx_ref_tag">1.1</span></a>, <a href="#S1.p1.pic1" title="Theorem 1.1 ‣ 1 Intro" "##,
      r##"class="ltx_ref ltx_refmacro_autoref"><span class="ltx_text ltx_ref_tag">1.1</span></a>, <a "##,
      r##"href="#S1.p1.pic1" title="In 1 Intro" class="ltx_ref"><span class="ltx_text ltx_ref_tag">theorem</span>"##,
      "\u{a0}",
      r##"<span class="ltx_text ltx_ref_tag">1.1</span></a>.</p>"##
    ),
  );
  box_svg(&html, "S1.p1.pic1");
}

/// A listing starts where listings makes `\@currentlabel` its line number's (lstmisc.sty:1228, :1225), so a `\label`
/// in an unnumbered listing after a `\refstepcounter{problem}` does not name the problem: the label stays on its line,
/// with the section's "2" (pdflatex prints the line's 2, a number an unnumbered `ltx:listingline` does not carry — the
/// residual of OXIDIZED_DESIGN_DIVERGENCES #813); a numbered listing's label the line's "1". Red (78g r1): "See 41 and
/// 1." (the problem's step). Repro listing_label_names_no_step_before_it.
#[test]
fn listing_label_names_no_step_before_it() {
  let html = html_of(repro!("listing_label_names_no_step_before_it.tex"));
  assert_element(
    &html,
    "p",
    &[r#"id="S2.p1.4""#],
    &format!(
      r#"<p id="S2.p1.4" class="ltx_p">See {} and {}.</p>"#,
      link("lstnumberx2", "2 Two", "2"),
      link("lstnumberx3", "line 1 ‣ 2 Two", "1"),
    ),
  );
}

/// An amsmath display's `\label` is `\label@in@display` (amsmath.sty:1230-1236), made in the row's tag cell after the
/// cell group restored `\@currentlabel` (:1240-1252, :1621-1627): a `{gather*}` row's label after a step made in its
/// cell is the section's "2"; one after a step made before the display the step's "42"; a numbered row's its own
/// "21"; an `{equation*}`'s, made in the display (:2821-2826), the step's "44" — as pdflatex prints them. The align
/// family and `{multline}` make the display tag only under `\iftag@` (:1833-1843), so an untagged row's label never
/// names the step held at the display's start: in a `\refstepcounter` environment an `{align}`/`{alignat}` untagged
/// row's label is the next tagged row's "22", "23"; a `{multline*}`'s the section's "2" (pdflatex `??`); an
/// `{aligned}` in an `{equation*}` keeps the step's "47"; a minipage in a `{gather*}` cell labels at once with the
/// cell's step, "48" (amsmath's `\@arrayparboxrestore`, :1237-1238). Red (78g r1): "See 41, 42, 21 and 44."; (78g r2)
/// "Also 45, 46, 47, 47 and 47."; (int47a) "See 2, 2, 21 and 2." "Also 22, 23, 2, 2 and 2.". Repro
/// amsmath_row_label_names_the_display_step (review probes rev78g2 al.tex, al2.tex, nest.tex).
#[test]
fn amsmath_row_label_names_the_display_step() {
  let html = html_of(repro!("amsmath_row_label_names_the_display_step.tex"));
  assert_element(
    &html,
    "p",
    &[r#"id="S2.p1.1""#],
    &format!(
      r#"<p id="S2.p1.1" class="ltx_p">See {}, {}, {} and {}.</p>"#,
      link("S2.Ex1", "2 Two", "2"),
      link("S2.Ex3", "In 2 Two", "42"),
      link("S2.E21", "In 2 Two", "21"),
      link("S2.Ex5", "In 2 Two", "44"),
    ),
  );
  assert_element(
    &html,
    "p",
    &[r#"id="S2.p4.2""#],
    &format!(
      r#"<p id="S2.p4.2" class="ltx_p">Also {}, {}, {}, {} and {}.</p>"#,
      link("S2.Ex6", "In 2 Two", "22"),
      link("S2.Ex7", "In 2 Two", "23"),
      link("S2.Ex8", "2 Two", "2"),
      link("S2.Ex9", "In 2 Two", "47"),
      link("S2.Ex10", "In 2 Two", "48"),
    ),
  );
}

/// An unnumbered display (`\[…\]`, `{equation*}`, `$$…$$`) a raw `\refstepcounter` is made in takes the step's number
/// for `\ref` and shows none (an empty number cell); with an `\eqno` too it keeps one `ltx:tags` — its "(B)" shown,
/// `\ref` the step's "43" (`\eqno` leaves `\@currentlabel` alone) — as pdflatex prints them. Red (int47a): "See 2, 2,
/// 2 and (B)."; (78g r1) d:d with two `ltx:tags`, `\ref` "(B)". Repro unnumbered_display_label_takes_the_step_number.
#[test]
fn unnumbered_display_label_takes_the_step_number() {
  let html = html_of(repro!("unnumbered_display_label_takes_the_step_number.tex"));
  assert_element(
    &html,
    "p",
    &[r#"id="S2.p1.1""#],
    &format!(
      r#"<p id="S2.p1.1" class="ltx_p">See {}, {}, {} and {}.</p>"#,
      link("S2.Ex1", "In 2 Two", "21"),
      link("S2.Ex2", "In 2 Two", "41"),
      link("S2.Ex3", "In 2 Two", "42"),
      link("S2.Ex4", "In 2 Two", "43"),
    ),
  );
  let display = |n: usize, tex: &str, mathml: &str, eqno: &str| {
    format!(
      concat!(
        r#"<table id="S2.Ex{n}" class="ltx_equation ltx_eqn_table"><tbody><tr class="ltx_equation ltx_eqn_row "#,
        r#"ltx_align_baseline"><td class="ltx_eqn_cell ltx_eqn_center_padleft"></td><td class="ltx_eqn_cell "#,
        r#"ltx_align_center"><math id="S2.Ex{n}.m1" class="ltx_Math" alttext="{tex}" display="block" "#,
        r#"intent=":literal"><mrow>{mathml}</mrow></math></td><td class="ltx_eqn_cell ltx_eqn_center_padright"></td>"#,
        r#"<td rowspan="1" class="ltx_eqn_cell ltx_eqn_eqno ltx_align_middle ltx_align_right">{eqno}</td></tr></tbody>"#,
        r#"</table>"#
      ),
      n = n,
      tex = tex,
      mathml = mathml,
      eqno = eqno
    )
  };
  let ab = |a: &str, b: &str| format!("<mi>{a}</mi><mo>=</mo><mi>{b}</mi>");
  for (n, a, b) in [(1, "a", "b"), (2, "c", "d"), (3, "e", "f")] {
    let id = format!(r#"id="S2.Ex{n}""#);
    assert_element(
      &html,
      "table",
      &[&id],
      &display(n, &format!("{a}={b}"), &ab(a, b), ""),
    );
  }
  assert_element(
    &html,
    "table",
    &[r#"id="S2.Ex4""#],
    &display(
      4,
      "g=h",
      &ab("g", "h"),
      concat!(
        r#"<span class="ltx_tag ltx_tag_equation ltx_align_right"><math id="S2.Ex4.m2" class="ltx_Math" "#,
        r#"display="inline" intent=":literal"><mrow><mo stretchy="false">(</mo><mi>B</mi><mo stretchy="false">)</mo>"#,
        r#"</mrow></math></span>"#
      ),
    ),
  );
}

/// Two raw steps in one `picture` (a `\refstepcounter` in each of two `\put`s): the first labels the picture with its
/// number "41"; the second, another step's, does not take the picture the first numbered — it climbs past it, as Perl's
/// does, to the section's "2" (pdflatex prints 42, which no element of the second `\put` can carry: the residual of
/// OXIDIZED_DESIGN_DIVERGENCES #813). Red (78g r1): "See 41 and 41." (both on the picture). Repro
/// picture_second_step_label_climbs.
#[test]
fn picture_second_step_label_climbs() {
  let html = html_of(repro!("picture_second_step_label_climbs.tex"));
  assert_element(
    &html,
    "p",
    &[r#"id="S2.p1.1""#],
    &format!(
      r#"<p id="S2.p1.1" class="ltx_p">See {} and {}.</p>"#,
      link("S2.p1.pic1", "In 2 Two", "41"),
      link("S2", "2 Two", "2"),
    ),
  );
  box_svg(&html, "S2.p1.pic1");
}

/// The kernel's `\label` a document saved at `\begin{document}` (`\let\originallabel\label`, the `\labelrel` idiom of
/// 2310.19138) labels at once in an amsmath cell, after the cell's `\refstepcounter`: "a", "b"; the display's own
/// `\label` (amsmath's `\label@in@display`, `\lx@ams@display@label`) on the row r:b numbered is not that step's — it
/// climbs to the equation group, the section's "2" — as pdflatex prints them. Red (78g r1): "See a, b and b."; a draft
/// holding every label in a display gave 2310.19138 "53" for its "a". Repro
/// amsmath_saved_kernel_label_names_the_cell_step.
#[test]
fn amsmath_saved_kernel_label_names_the_cell_step() {
  let html = html_of(repro!("amsmath_saved_kernel_label_names_the_cell_step.tex"));
  assert_element(
    &html,
    "p",
    &[r#"id="S2.p1.1""#],
    &format!(
      r#"<p id="S2.p1.1" class="ltx_p">See {}, {} and {}.</p>"#,
      link("S2.Ex1", "In 2 Two", "a"),
      link("S2.Ex2", "In 2 Two", "b"),
      link("S2.EGx2", "2 Two", "2"),
    ),
  );
}

/// A box in math (`\text{\begin{tcolorbox}…}`, `\mbox{…}`) after a raw `\refstepcounter`: its id does not survive
/// the math's rendering, so the label climbs out of the `ltx:Math` to the display, which takes the step's number when
/// it has none — a `{gather*}` row "41", a `\[…\]` "43", a `\parbox`'s `{gather*}` row "46" (labelled at once,
/// amsmath's `\@arrayparboxrestore`), as pdflatex prints them — or keeps its own (a numbered
/// `{equation}`/`{gather}` row "1", "2"; pdflatex 44, 45, the own-number residual of OXIDIZED_DESIGN_DIVERGENCES
/// #813); an inline formula's climbs to the section "2" (pdflatex 42; Perl and int47a the same). Every `\ref` links to
/// an element the HTML has. Red (78g r3): "See , , ,  and ." (labels on the box pictures inside the math, a Missing
/// Target warning; the parbox's "46"); (int47a, Perl the same) "See 2, 2, 2, 1, 2 and 2.". Repro
/// math_box_label_climbs_to_the_display (review probes rev78g3 tbx.tex, tbx3.tex, pb2.tex y2).
#[test]
fn math_box_label_climbs_to_the_display() {
  let html = html_of(repro!("math_box_label_climbs_to_the_display.tex"));
  assert_element(
    &html,
    "p",
    &[r#"id="S2.p1.2""#],
    &format!(
      r#"<p id="S2.p1.2" class="ltx_p">See {}, {}, {}, {}, {} and {}.</p>"#,
      link("S2.Ex1", "In 2 Two", "41"),
      link("S2", "2 Two", "2"),
      link("S2.Ex2", "In 2 Two", "43"),
      link("S2.E1", "In 2 Two", "1"),
      link("S2.E2", "In 2 Two", "2"),
      link("S2.Ex3", "In 2 Two", "46"),
    ),
  );
  for id in ["S2.Ex1", "S2.Ex2", "S2.E1", "S2.E2", "S2.Ex3"] {
    assert!(
      html.contains(&format!(r#" id="{id}""#)),
      "no element {id} to link to"
    );
  }
}
