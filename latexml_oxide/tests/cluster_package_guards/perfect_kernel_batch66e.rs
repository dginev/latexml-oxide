//! Red/green guards for batch 66e: AMS-TeX's `\tag` (amstex.tex:587-605 in a display, also after `\endmultline`,
//! :903; :629-640, 718-739, 797 in `\align`, `\alignat`, `\gather`), with the markup amsmath's `\tag` gives. Repros
//! `tools/perfect_kernel/repros/amstex/amstex_tag_with_math_text.tex`, `amstex/amstex_align_row_tags.tex`,
//! `amstex/amstex_plain_input_tags.tex`.
use super::perfect_kernel_batch46::{error_count, warning_count};

fn convert_counted(tex: &str) -> (String, String) {
  latexml::util::test::convert_with(tex, Some("ar5iv.sty"))
}

/// The `ltx:equation` with this id, up to its end (an equation holds no other).
fn equation<'a>(xml: &'a str, id: &str) -> &'a str {
  let open = format!("<equation xml:id=\"{id}\">");
  let start = xml
    .find(&open)
    .unwrap_or_else(|| panic!("no {open}\n{xml}"));
  let end = xml[start..]
    .find("</equation>")
    .map_or(xml.len(), |e| start + e);
  &xml[start..end]
}

/// Assert that the equation `id` holds `tags`, its whole `ltx:tags`.
fn assert_tags_on(xml: &str, id: &str, tags: &str) {
  let eq = equation(xml, id);
  assert!(eq.contains(tags), "{id}: {tags}\n{eq}");
}

/// AMS-TeX's `\tag#1$$` takes the tag text up to the display's closing `$$` and sets it as text
/// (`\tagform@` = `\hbox{\rm(#1)}`), so a `$'$` in it is inline math, not the display's end: `\let\tag\eqno` (Perl's
/// too, KNOWN_PERL_ERRORS #643) closed the display there ("Missing $", 1012.3760, 1112.1423; `\tag {$*$}` 1108.6264).
/// The tags are amsmath's `\tag{…}` ones (`\tag{1.2$'$}`); a quoted tag is set without parentheses
/// (`\maketag@@@"#1"#2\maketag@{\hbox{\rm#1}}`), the `\tag*` form.
#[test]
fn amstex_display_tag_is_a_text_tag() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amstex_tag_with_math_text.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // each display's tags on its own equation
  for (id, tags) in [
    (
      "equationx1",
      "<tags>\n        <tag>(1.2<sup xml:id=\"equationx1.1\"><text font=\"italic\" xml:id=\"equationx1.1.1\">\u{2032}</text></sup>)</tag>\n        <tag role=\"refnum\">1.2<sup xml:id=\"equationx1.2\"><text font=\"italic\" xml:id=\"equationx1.2.1\">\u{2032}</text></sup></tag>\n      </tags>",
    ),
    (
      "equationx3",
      "<tags>\n        <tag>(2)</tag>\n        <tag role=\"refnum\">2</tag>\n      </tags>",
    ),
    (
      "equationx4",
      "<tags>\n        <tag>A</tag>\n        <tag role=\"refnum\">A</tag>\n      </tags>",
    ),
  ] {
    assert_tags_on(&xml, id, tags);
  }
  // `\tag {$*$}`: the tag's inline math, as in amsmath's `\tag{$*$}`
  for tag in [
    "<tags>\n        <tag>(<Math mode=\"inline\" tex=\"*\" text=\"*\" xml:id=\"equationx2.m2\">",
    "<tag role=\"refnum\"><Math mode=\"inline\" tex=\"*\" text=\"*\" xml:id=\"equationx2.m3\">",
  ] {
    assert_tags_on(&xml, "equationx2", tag);
  }
  assert_eq!(xml.matches("<tags>").count(), 4, "{xml}");
  assert_eq!(xml.matches("<equation ").count(), 4, "{xml}");
  // the displays end where they end: one display Math each
  assert_eq!(xml.matches("<Math mode=\"display\"").count(), 4, "{xml}");
}

/// Inside `\align`, AMS-TeX's `\tag` opens the row's tag cell (`&&`, amstex.tex:633-635): the tag runs to the row's
/// end, `\\` or `\endalign` (1012.3760: `\tag 6.16\\`, `&=\sum_\alpha … \tag 2.1` before `\endalign`), a `$'$` in it
/// is inline math, and the tag is the row's: the lone-`aligned` rearrangement gives each row equation its own.
#[test]
fn amstex_align_row_tags_run_to_the_row_end() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amstex_align_row_tags.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (id, tags) in [
    (
      "equationx1X",
      "<tags>\n          <tag>(6.16)</tag>\n          <tag role=\"refnum\">6.16</tag>\n        </tags>",
    ),
    (
      "equationx1Xa",
      "<tags>\n          <tag>(6.17<sup xml:id=\"equationx1Xa.1\"><text font=\"italic\" xml:id=\"equationx1Xa.1.1\">\u{2032}</text></sup>)</tag>\n          <tag role=\"refnum\">6.17<sup xml:id=\"equationx1Xa.2\"><text font=\"italic\" xml:id=\"equationx1Xa.2.1\">\u{2032}</text></sup></tag>\n        </tags>",
    ),
    (
      "equationx1Xb",
      "<tags>\n          <tag>(2.1)</tag>\n          <tag role=\"refnum\">2.1</tag>\n        </tags>",
    ),
  ] {
    assert!(
      xml.contains(&format!("<equation xml:id=\"{id}\">\n        {tags}")),
      "{id}: {tags}\n{xml}"
    );
  }
  assert_eq!(xml.matches("<tags>").count(), 3, "{xml}");
  assert!(
    xml.contains("<equationgroup xml:id=\"equationx1\">\n      <equation "),
    "{xml}"
  );
  assert!(!xml.contains("_tags"), "{xml}");
}

/// amstex.tex:590-593: `\TagsAsMath` sets the tag in math (`\tagform@{$\m@th#1$}`), `\TagsAsText` back in text; the
/// amsppt binding's no-op stubs hid the pool's switches.
#[test]
fn amstex_tags_as_math_switch() {
  let (log, xml) = convert_counted(
    "\\input amstex\n\\documentstyle{amsppt}\n\\document\n\\TagsAsMath\n$$\na=b\\tag 1.2'\n$$\n\\TagsAsText\n$$\nc=d\\tag 3\n$$\n\\enddocument\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // the whole tags element: both tags in math, the parentheses outside it
  assert_tags_on(
    &xml,
    "equationx1",
    r#"<tags>
        <tag>(<Math mode="inline" tex="1.2^{\prime}" text="1.2 ^ prime" xml:id="equationx1.m2">
            <XMath xml:id="equationx1.m2.1">
              <XMApp xml:id="equationx1.m2.1.1">
                <XMTok role="SUPERSCRIPTOP" scriptpos="post1"/>
                <XMTok meaning="1.2" role="NUMBER">1.2</XMTok>
                <XMTok fontsize="70%" name="prime" role="SUPOP">′</XMTok>
              </XMApp>
            </XMath>
          </Math>)</tag>
        <tag role="refnum"><Math mode="inline" tex="1.2^{\prime}" text="1.2 ^ prime" xml:id="equationx1.m3">
            <XMath xml:id="equationx1.m3.1">
              <XMApp xml:id="equationx1.m3.1.1">
                <XMTok role="SUPERSCRIPTOP" scriptpos="post1"/>
                <XMTok meaning="1.2" role="NUMBER">1.2</XMTok>
                <XMTok fontsize="70%" name="prime" role="SUPOP">′</XMTok>
              </XMApp>
            </XMath>
          </Math></tag>
      </tags>"#,
  );
  assert_tags_on(
    &xml,
    "equationx2",
    "<tags>\n        <tag>(3)</tag>\n        <tag role=\"refnum\">3</tag>\n      </tags>",
  );
}

/// A plain AMS-TeX document (`\input amstex`, `\input amsppt.sty`, no `\documentstyle`) never loads the LaTeX pool,
/// so `\tag`'s float path is the AmSTeX pool's own: 66e r1 kept it in the LaTeX pool ("\lx@equation@settag@float
/// undefined" and "<ltx:tags> isn't allowed in <ltx:XMath>" per tag; math9412232, hep-th9308082, hep-th9206021,
/// hep-th9111004, 1102.5649 TooManyErrors). The tags are the ones a `\documentstyle{amsppt}` document gets, `(1)`
/// with AMS-TeX's `\tagform@` parentheses.
#[test]
fn plain_amstex_tags_need_no_latex_pool() {
  let (log, xml) = convert_counted(include_str!(
    "../../../tools/perfect_kernel/repros/amstex/amstex_plain_input_tags.tex"
  ));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(!log.contains("loading LaTeX.pool"), "{log}");
  assert_tags_on(
    &xml,
    "id1.2",
    "<tags>\n        <tag>(1)</tag>\n        <tag role=\"refnum\">1</tag>\n      </tags>",
  );
  assert_tags_on(
    &xml,
    "id1.4",
    "<tags>\n        <tag>(1.2<sup xml:id=\"id1.4.2\"><text font=\"italic\" xml:id=\"id1.4.2.1\">\u{2032}</text></sup>)</tag>\n        <tag role=\"refnum\">1.2<sup xml:id=\"id1.4.3\"><text font=\"italic\" xml:id=\"id1.4.3.1\">\u{2032}</text></sup></tag>\n      </tags>",
  );
  assert!(
    xml.contains("<equation xml:id=\"id1.6.1\">\n        <tags>\n          <tag>(2.1)</tag>\n          <tag role=\"refnum\">2.1</tag>\n        </tags>"),
    "{xml}"
  );
  assert_eq!(xml.matches("<tags>").count(), 3, "{xml}");
  assert!(!xml.contains("_tags"), "{xml}");
}

/// A row's tag ends at every AMS-TeX row ender, not only `\\`: `\displaybreak`, `\allowdisplaybreak` (amstex.tex:613-614
/// `\crcr\noalign`), `\vspace` (`\vspace@`, amstex.rs), and a macro whose expansion starts with `\\` (`\def\nl{\\}`).
/// The tag was "(1 c=d)". (pdftex errs on a row ender right after a tag here: the measuring pass's tag cell is
/// `\eat@{##}`, amstex.tex:655; the recovery keeps the tag to its number.)
#[test]
fn amstex_row_tag_ends_at_every_row_ender() {
  let (log, xml) = convert_counted(
    "\\input amstex\n\\input amsppt.sty\n\\document\n\\def\\nl{\\\\}\n\
     $$\\align a&=b\\tag 1\\displaybreak c&=d\\\\ e&=f\\endalign$$\n\
     $$\\align a&=b\\tag 2\\allowdisplaybreak c&=d\\endalign$$\n\
     $$\\align a&=b\\tag 4\\vspace{2pt} c&=d\\endalign$$\n\
     $$\\align a&=b\\tag 5\\nl c&=d\\endalign$$\n\\enddocument\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (id, n) in [
    ("id1.1.1", 1),
    ("id1.2.1", 2),
    ("id1.3.1", 4),
    ("id1.4.1", 5),
  ] {
    assert_tags_on(
      &xml,
      id,
      &format!(
        "<tags>\n          <tag>({n})</tag>\n          <tag role=\"refnum\">{n}</tag>\n        </tags>"
      ),
    );
  }
  assert_eq!(xml.matches("<tags>").count(), 4, "{xml}");
  // `\intertext` ends the row too (amstex.tex:615-618); the intertext itself inside `$$\align` is a separate gap
  // (one "<ltx:p> isn't allowed in <ltx:XMArg>", before 66e too; RED repro amstex/amstex_align_intertext)
  let (log, xml) = convert_counted(
    "\\input amstex\n\\input amsppt.sty\n\\document\n$$\\align a&=b\\tag 3\\intertext{and} c&=d\\endalign$$\n\\enddocument\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(
    log.contains("<ltx:p> isn't allowed in <ltx:XMArg>"),
    "{log}"
  );
  assert_tags_on(
    &xml,
    "id1.1.1",
    "<tags>\n          <tag>(3)</tag>\n          <tag role=\"refnum\">3</tag>\n        </tags>",
  );
}

/// The row test needs an alignment opened inside the display's own math: a `$$` display in an `\halign` cell is no
/// row, so `\tag 1\matrix a\cr b\endmatrix` reads to the `$$` (the matrix in the tag, as TeX does; `\TagsAsMath` sets
/// it in math) instead of stopping at the matrix's `\cr`.
#[test]
fn amstex_display_tag_in_an_halign_cell_reads_to_the_display_end() {
  let (log, xml) = convert_counted(
    "\\input amstex\n\\documentstyle{amsppt}\n\\document\n\\TagsAsMath\n\\halign{#\\cr\n\
     \\vbox{\\hsize=10cm $$x\\tag 1\\matrix a\\cr b\\endmatrix$$}\\cr}\n\\enddocument\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_tags_on(
    &xml,
    "equationx1",
    "<tags>\n                <tag>(<Math mode=\"inline\" tex=\"1\\begin{matrix}a\\cr b\\end{matrix}\"",
  );
  assert_eq!(xml.matches("<tags>").count(), 1, "{xml}");
}

/// `\tag#1$$` is not `\long`: a `\par`, at any depth, ends the call with TeX's runaway error, and a `}` that closes a
/// group opened before the `\tag` with "Argument of \tag has an extra }" (tex.web §392, §395); TeX drops the call, so
/// the tag is dropped. (The `}` then ends the display: the second error.)
#[test]
fn amstex_tag_argument_errors() {
  let (log, xml) = convert_counted(
    "\\input amstex\n\\input amsppt.sty\n\\document\n{$$ a=b\\tag 1}$$ x $$\n\\enddocument\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (2, 0), "{log}");
  assert!(log.contains("Argument of \\tag has an extra }"), "{log}");
  assert!(log.contains("Extra }, or forgotten $"), "{log}");
  assert!(!xml.contains("<tags>"), "{xml}");
  for tex in [
    "\\input amstex\n\\input amsppt.sty\n\\document\n$$ c=d\\tag 2\\par$$\n\\enddocument\n",
    "\\input amstex\n\\input amsppt.sty\n\\document\n$$ c=d\\tag{2\\par$$\n\\enddocument\n",
  ] {
    let (log, xml) = convert_counted(tex);
    assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
    assert!(
      log.contains("Paragraph ended before \\tag was complete"),
      "{log}"
    );
    assert!(!xml.contains("<tags>"), "{xml}");
  }
}

/// `\def\tag#1$$` strips the braces of a delimited argument that is one group (tex.web §400): `\tag{"A"}$$` is the
/// quoted form; with a space before the `$$` the group stays, and so does a row's tag cell (`\tag{"D"}\\`), set as
/// `("D")`.
#[test]
fn amstex_tag_braced_argument_is_stripped() {
  let (log, xml) = convert_counted(
    "\\input amstex\n\\input amsppt.sty\n\\document\n$$ a=b\\tag{\"A\"}$$\n$$ a=b\\tag{\"C\"} $$\n\
     $$\\align a&=b\\tag{\"D\"}\\\\ c&=d\\endalign$$\n\\enddocument\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_tags_on(
    &xml,
    "id1.1",
    "<tags>\n        <tag>A</tag>\n        <tag role=\"refnum\">A</tag>\n      </tags>",
  );
  assert_tags_on(
    &xml,
    "id1.2",
    "<tags>\n        <tag>(\u{201D}C\u{201D})</tag>\n        <tag role=\"refnum\">\u{201D}C\u{201D}</tag>\n      </tags>",
  );
  assert_tags_on(
    &xml,
    "id1.3.1",
    "<tags>\n          <tag>(\u{201D}D\u{201D})</tag>\n          <tag role=\"refnum\">\u{201D}D\u{201D}</tag>\n        </tags>",
  );
}

/// In an equation row amsmath's `\tag` sets (`in_equation`: a LaTeX `equation`, an `align` row) AMS-TeX's `\tag`
/// hands its text to it (`\lx@amstex@ams@tag`): the equation is retracted (`S0.Ex1`, not numbered), a row's tag
/// ends at its `\\`, and a quoted tag is the `\tag*` form. (pdflatex cannot `\input amstex`: amstex.tex:474
/// `\fontdimen13\tenex`.)
#[test]
fn amstex_tag_in_latex_equation() {
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\input amstex\n\\begin{document}\n\
     \\begin{equation} a=b \\tag 3.1$'$ \\end{equation}\n\
     \\begin{align} a&=b \\tag 4\\\\ c&=d \\tag \"Q\" \\end{align}\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_tags_on(
    &xml,
    "S0.Ex1",
    "<tags>\n        <tag>(3.1<sup xml:id=\"S0.Ex1.1\"><text font=\"italic\" xml:id=\"S0.Ex1.1.1\">\u{2032}</text></sup>)</tag>\n        <tag role=\"refnum\">3.1<sup xml:id=\"S0.Ex1.2\"><text font=\"italic\" xml:id=\"S0.Ex1.2.1\">\u{2032}</text></sup></tag>\n      </tags>",
  );
  assert_tags_on(
    &xml,
    "S0.Ex2",
    "<tags>\n          <tag>(4)</tag>\n          <tag role=\"refnum\">4</tag>\n        </tags>",
  );
  assert_tags_on(
    &xml,
    "S0.Ex3",
    "<tags>\n          <tag>Q</tag>\n          <tag role=\"refnum\">Q</tag>\n        </tags>",
  );
  assert_eq!(xml.matches("<tags>").count(), 3, "{xml}");
}
