//! Red/green guards for batch 66e: AMS-TeX's `\tag` (amstex.tex:587-605 in a display, also after `\endmultline`,
//! :903; :629-640, 718-739, 797 in `\align`, `\alignat`, `\gather`), with the markup amsmath's `\tag` gives. Repros
//! `tools/perfect_kernel/repros/amstex/amstex_tag_with_math_text.tex`, `amstex/amstex_align_row_tags.tex`,
//! `amstex/amstex_plain_input_tags.tex`, `amstex/amstex_row_tag_runaway_group.tex`,
//! `amstex/amstex_row_tag_open_group_in_enclosing_group.tex`.
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

/// Assert that the `ltx:tags` of the equation `id` is `tags`, the whole element (lines compared trimmed).
fn assert_tags_on(xml: &str, id: &str, tags: &str) {
  let eq = equation(xml, id);
  let tags = tags.lines().map(str::trim).collect::<Vec<_>>().join("\n");
  assert_eq!(element(eq, "<tags>", "</tags>"), tags, "{id}\n{eq}");
}

/// `xml` from the first `open` to the end of its `close`, each line trimmed: a whole element, indentation-free.
fn element(xml: &str, open: &str, close: &str) -> String {
  let start = xml.find(open).unwrap_or_else(|| panic!("no {open}\n{xml}"));
  let end = xml[start..].find(close).map_or_else(
    || panic!("no {close} after {open}\n{xml}"),
    |e| start + e + close.len(),
  );
  xml[start..end]
    .lines()
    .map(str::trim)
    .collect::<Vec<_>>()
    .join("\n")
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
  assert_tags_on(
    &xml,
    "equationx2",
    "<tags>\n<tag>(<Math mode=\"inline\" tex=\"*\" text=\"*\" xml:id=\"equationx2.m2\">\n<XMath xml:id=\"equationx2.m2.1\">\n<XMTok meaning=\"times\" role=\"MULOP\">\u{2217}</XMTok>\n</XMath>\n</Math>)</tag>\n\
     <tag role=\"refnum\"><Math mode=\"inline\" tex=\"*\" text=\"*\" xml:id=\"equationx2.m3\">\n<XMath xml:id=\"equationx2.m3.1\">\n<XMTok meaning=\"times\" role=\"MULOP\">\u{2217}</XMTok>\n</XMath>\n</Math></tag>\n</tags>",
  );
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
/// `\eat@{##}`, amstex.tex:655; the recovery keeps the tag to its number, OXIDIZED_DESIGN_DIVERGENCES #575.)
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
  // the whole `ltx:tags`: "(1 matrix)" in math, both tags ({IT}: the invisible times)
  assert_eq!(
    element(equation(&xml, "equationx1"), "<tags>", "</tags>"),
    r#"<tags>
<tag>(<Math mode="inline" tex="1\begin{matrix}a\cr b\end{matrix}" text="1 * matrix@(Array[[a], [b]])" xml:id="equationx1.m2">
<XMath xml:id="equationx1.m2.2">
<XMApp xml:id="equationx1.m2.2.1">
<XMTok meaning="times" role="MULOP">{IT}</XMTok>
<XMTok meaning="1" role="NUMBER">1</XMTok>
<XMDual xml:id="equationx1.m2.2.1.3">
<XMApp xml:id="equationx1.m2.2.1.3.1">
<XMTok meaning="matrix"/>
<XMRef idref="equationx1.m2.1" xml:id="equationx1.m2.2.1.3.1.2"/>
</XMApp>
<XMArray xml:id="equationx1.m2.1">
<XMRow xml:id="equationx1.m2.1.1">
<XMCell align="center" xml:id="equationx1.m2.1.1.1">
<XMTok font="italic" role="UNKNOWN">a</XMTok>
</XMCell>
</XMRow>
<XMRow xml:id="equationx1.m2.1.2">
<XMCell align="center" xml:id="equationx1.m2.1.2.1">
<XMTok font="italic" role="UNKNOWN">b</XMTok>
</XMCell>
</XMRow>
</XMArray>
</XMDual>
</XMApp>
</XMath>
</Math>)</tag>
<tag role="refnum"><Math mode="inline" tex="1\begin{matrix}a\cr b\end{matrix}" text="1 * matrix@(Array[[a], [b]])" xml:id="equationx1.m3">
<XMath xml:id="equationx1.m3.2">
<XMApp xml:id="equationx1.m3.2.1">
<XMTok meaning="times" role="MULOP">{IT}</XMTok>
<XMTok meaning="1" role="NUMBER">1</XMTok>
<XMDual xml:id="equationx1.m3.2.1.3">
<XMApp xml:id="equationx1.m3.2.1.3.1">
<XMTok meaning="matrix"/>
<XMRef idref="equationx1.m3.1" xml:id="equationx1.m3.2.1.3.1.2"/>
</XMApp>
<XMArray xml:id="equationx1.m3.1">
<XMRow xml:id="equationx1.m3.1.1">
<XMCell align="center" xml:id="equationx1.m3.1.1.1">
<XMTok font="italic" role="UNKNOWN">a</XMTok>
</XMCell>
</XMRow>
<XMRow xml:id="equationx1.m3.1.2">
<XMCell align="center" xml:id="equationx1.m3.1.2.1">
<XMTok font="italic" role="UNKNOWN">b</XMTok>
</XMCell>
</XMRow>
</XMArray>
</XMDual>
</XMApp>
</XMath>
</Math></tag>
</tags>"#
      .replace("{IT}", "\u{2062}"),
    "{xml}"
  );
  assert_eq!(xml.matches("<tags>").count(), 1, "{xml}");
}

/// `\tag#1$$` is not `\long`: a `\par`, at any depth, ends the call with TeX's runaway error (tex.web §396). A `}`
/// that closes a group opened before the `\tag` raises "Argument of \tag has an extra }", is backed up behind an
/// inserted `\par` and the scan goes on (§395), so the `\par` raises the runaway error too: pdftex reports both on
/// `{$$ a=b\tag 1}$$ x $$`, then "Missing $ inserted" and "Display math should end with $$" for the `\par` in the
/// display; ours ignores a `\par` in math (as Perl), so the `}` ends the display with "Extra }, or forgotten $": 3
/// errors for pdftex's 4. TeX drops the call, so the tag is dropped; `a=b` and `x` survive, each in its display.
/// The `\par` in the first display's `tex="a=b\par"` is the one §395's recovery inserted before the `}`, not the
/// author's; it is harmless (a `\par` in math is ignored, as in Perl).
#[test]
fn amstex_tag_argument_errors() {
  let (log, xml) = convert_counted(
    "\\input amstex\n\\input amsppt.sty\n\\document\n{$$ a=b\\tag 1}$$ x $$\n\\enddocument\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (3, 0), "{log}");
  let first = log
    .find("Argument of \\tag has an extra }")
    .unwrap_or_else(|| panic!("{log}"));
  let second = log
    .find("Paragraph ended before \\tag was complete")
    .unwrap_or_else(|| panic!("{log}"));
  let third = log
    .find("Extra }, or forgotten $")
    .unwrap_or_else(|| panic!("{log}"));
  assert!(first < second && second < third, "{log}");
  assert!(!xml.contains("<tags>"), "{xml}");
  assert_eq!(
    element(&xml, "<equation xml:id=\"id1.1\">", "</equation>"),
    "<equation xml:id=\"id1.1\">\n<Math mode=\"display\" tex=\"a=b\\par\" text=\"a = b\" xml:id=\"id1.1.1\">\n\
     <XMath xml:id=\"id1.1.1.1\">\n<XMApp xml:id=\"id1.1.1.1.1\">\n\
     <XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok>\n<XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok>\n\
     <XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok>\n</XMApp>\n</XMath>\n</Math>\n</equation>",
    "{xml}"
  );
  assert_eq!(
    element(&xml, "<equation xml:id=\"id1.2\">", "</equation>"),
    "<equation xml:id=\"id1.2\">\n<Math mode=\"display\" tex=\"x\" text=\"x\" xml:id=\"id1.2.1\">\n\
     <XMath xml:id=\"id1.2.1.1\">\n<XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok>\n</XMath>\n</Math>\n</equation>",
    "{xml}"
  );
  assert_eq!(xml.matches("<equation ").count(), 2, "{xml}");
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
    assert!(
      xml.contains("<Math mode=\"display\" tex=\"c=d\\par\" text=\"c = d\""),
      "{xml}"
    );
  }
}

/// A row's `\tag` takes no argument (`\def\tag{\global\tag@true…&&}` in `\align`, amstex.tex:633-635; `\def\tag{&}`
/// in `\gather`, :797), so it raises no runaway or "extra }" error: a `\par` outside the tag's own groups, or a `}`
/// closing a group opened before it, ends the tag cell, and the row goes on; a `\par` inside the tag's own group stays
/// in its text when the group closes before the row ends (`\tag{3\par}\\`). pdftex errs on each shape: `\align`
/// reads its body as a non-`\long` argument (amstex.tex:667, :687), "Paragraph ended before \align was complete" and
/// the display-math errors; `a&={b\tag 2} c` cascades to 100 errors and no PDF. OXIDIZED_DESIGN_DIVERGENCES #575.
#[test]
fn amstex_row_tag_cell_ends_at_par_and_extra_brace() {
  let (log, xml) = convert_counted(
    "\\input amstex\n\\input amsppt.sty\n\\document\n\
     $$\\align a&=b\\tag{3\\par}\\\\ c&=d\\endalign$$\n\n\
     $$\\align a&=b\\tag 4\\par\\\\ c&=d\\endalign$$\n\n\
     $$\\align a&={b\\tag 2} c\\\\ e&=f\\endalign$$\n\\enddocument\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (id, n, row) in [
    (
      "id1.1.1",
      "3",
      "<Math tex=\"\\displaystyle a=b\" text=\"a = b\"",
    ),
    (
      "id2.1.1",
      "4",
      "<Math tex=\"\\displaystyle a=b\\par\" text=\"a = b\"",
    ),
    (
      "id3.1.1",
      "2",
      "<Math tex=\"\\displaystyle a={b}c\" text=\"a = b * c\"",
    ),
  ] {
    assert_tags_on(
      &xml,
      id,
      &format!(
        "<tags>\n          <tag>({n})</tag>\n          <tag role=\"refnum\">{n}</tag>\n        </tags>"
      ),
    );
    assert!(equation(&xml, id).contains(row), "{id}: {row}\n{xml}");
  }
  for (id, row) in [
    ("id1.1.2", "<Math tex=\"\\displaystyle c=d\" text=\"c = d\""),
    ("id2.1.2", "<Math tex=\"\\displaystyle c=d\" text=\"c = d\""),
    ("id3.1.2", "<Math tex=\"\\displaystyle e=f\" text=\"e = f\""),
  ] {
    let eq = equation(&xml, id);
    assert!(
      eq.contains(row) && !eq.contains("<tags>"),
      "{id}: {row}\n{xml}"
    );
  }
  assert_eq!(xml.matches("<tags>").count(), 3, "{xml}");
  assert_eq!(xml.matches("<equation ").count(), 6, "{xml}");
}

/// A row's or environment's end read while a group of a row's `\tag` is open (`\tag{5\\ c&=d\endalign$$`) stops the
/// tag's scan there: TeX's `\cr` inside the cell's braces is "Missing } inserted" (tex.web §1126-1127, once per open
/// group), and the scan never reads past that end. The tag keeps its text with its groups closed, and the row, the
/// alignment, an enclosing group (`{\it …}`, `\vbox{…}`), the paragraphs after it and `\enddocument` are read as they
/// are. A `\par` inside a group of the tag that closes before the row's end stays in the text (`\tag{{3\par}5\\`); a
/// `\par` in a group still open there is TeX's runaway, and the input goes on from that `\par` (`\tag{5\par 6\\`).
/// The scan stops at the first such end at any depth (`\tag{{5\\ 6}\par`): the `}` after the row's end is the
/// next row's, where a stray `}` closes the display, as without a tag (`$$\align a&=b\\ 6} c&=d\endalign$$`, one
/// error). r3 read the rest of the document into the tag or to the file's end; r4 read past the alignment's end when
/// an enclosing group's `}` came first, and kept a `\par` or a row end recorded in a nested group that closed. pdftex
/// errs on each shape and recovers none ("Paragraph ended before \align was complete" and the display-math errors; the
/// `\vbox` one stops with "You can't use `\end' in internal vertical mode", no PDF). OXIDIZED_DESIGN_DIVERGENCES
/// #575; repros `amstex/amstex_row_tag_runaway_group`, `amstex/amstex_row_tag_open_group_in_enclosing_group`.
#[test]
fn amstex_row_tag_open_group_stops_at_the_row_end() {
  const MISSING: &str = "Missing } inserted before the row's end in \\tag";
  const RUNAWAY: &str = "Paragraph ended before \\tag was complete";
  const STRAY: &str = "Attempt to close a group that switched to mode display_math";
  let tags = |n: &str| format!("<tags>\n<tag>({n})</tag>\n<tag role=\"refnum\">{n}</tag>\n</tags>");
  let para = |id: &str, text: &str| {
    format!("<para xml:id=\"{id}\">\n<p xml:id=\"{id}.1\">{text}</p>\n</para>")
  };
  let later = "\n\nLater {para}graph.\n\nLast paragraph.\n\\enddocument\n";
  let align = "$$\\align a&=b\\tag{5\\\\ c&=d\\endalign$$";
  // (body, end, errors, the tagged row's equation id and tag, its math, the paragraphs after it, equations)
  type Case<'a> = (
    String,
    &'a str,
    Vec<(&'a str, usize)>,
    Option<(&'a str, &'a str, &'a str)>,
    Vec<(&'a str, &'a str)>,
    usize,
  );
  let cases: Vec<Case> = vec![
    (
      align.to_string(),
      later,
      vec![(MISSING, 1)],
      Some(("id1.1.1", "5", "a=b")),
      vec![("id2", "Later paragraph."), ("id3", "Last paragraph.")],
      2,
    ),
    (
      align.to_string(),
      "\n\\enddocument\n",
      vec![(MISSING, 1)],
      Some(("id1.1.1", "5", "a=b")),
      vec![],
      2,
    ),
    (
      format!("{{\\it Before {align} inside}} after."),
      later,
      vec![(MISSING, 1)],
      Some(("id1.2.1", "5", "a=b")),
      vec![("id2", "Later paragraph."), ("id3", "Last paragraph.")],
      2,
    ),
    (
      format!("\\vbox{{Before {align} inside}} after."),
      later,
      vec![(MISSING, 1)],
      Some(("id1.1.2.1", "5", "a=b")),
      vec![
        ("id2", "after."),
        ("id3", "Later paragraph."),
        ("id4", "Last paragraph."),
      ],
      2,
    ),
    (
      "$$\\align a&=b\\tag{{3\\par}5\\\\ c&=d\\endalign$$".to_string(),
      later,
      vec![(MISSING, 1)],
      Some(("id1.1.1", "35", "a=b")),
      vec![("id2", "Later paragraph."), ("id3", "Last paragraph.")],
      2,
    ),
    (
      "$$\\align a&=b\\tag{5\\par 6\\\\ c&=d\\endalign$$".to_string(),
      later,
      vec![(RUNAWAY, 1)],
      Some(("id1.1.1", "5", "a=b\\par 6")),
      vec![("id2", "Later paragraph."), ("id3", "Last paragraph.")],
      2,
    ),
    (
      "$$\\align a&=b\\tag{{5\\\\ 6}\\par c&=d\\endalign$$".to_string(),
      later,
      vec![(MISSING, 2), (STRAY, 1)],
      None,
      vec![("id2", "Later paragraph."), ("id3", "Last paragraph.")],
      1,
    ),
  ];
  for (body, end, errors, tagged, paras, equations) in cases {
    let tex = format!("\\input amstex\n\\input amsppt.sty\n\\document\n{body}{end}");
    let (log, xml) = convert_counted(&tex);
    let total: usize = errors.iter().map(|(_, n)| n).sum();
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (total, 0),
      "{body}\n{log}"
    );
    for (message, n) in errors {
      assert_eq!(log.matches(message).count(), n, "{body}: {message}\n{log}");
    }
    if let Some((id, n, math)) = tagged {
      assert_tags_on(&xml, id, &tags(n));
      let row = format!("<Math tex=\"\\displaystyle {math}\"");
      assert!(equation(&xml, id).contains(&row), "{body}: {row}\n{xml}");
      let second = format!("{}2", &id[..id.len() - 1]);
      let eq = equation(&xml, &second);
      assert!(
        eq.contains("<Math tex=\"\\displaystyle c=d\" text=\"c = d\"") && !eq.contains("<tags>"),
        "{body}\n{xml}"
      );
    }
    assert_eq!(
      xml.matches("<tags>").count(),
      usize::from(tagged.is_some()),
      "{body}\n{xml}"
    );
    assert_eq!(
      xml.matches("<equation ").count(),
      equations,
      "{body}\n{xml}"
    );
    for (id, text) in paras {
      assert_eq!(
        element(&xml, &format!("<para xml:id=\"{id}\">"), "</para>"),
        para(id, text),
        "{body}\n{xml}"
      );
    }
    assert!(xml.trim_end().ends_with("</document>"), "{body}\n{xml}");
  }
  // the enclosing group survives the tag: `inside` is in it, `after.` is not
  let (_, xml) = convert_counted(&format!(
    "\\input amstex\n\\input amsppt.sty\n\\document\n{{\\it Before {align} inside}} after.{later}"
  ));
  assert_eq!(
    element(&xml, "<p xml:id=\"id1.3\">", "</p>"),
    "<p xml:id=\"id1.3\"><text font=\"italic\" xml:id=\"id1.3.1\">inside</text> after.</p>",
    "{xml}"
  );
  let (_, xml) = convert_counted(&format!(
    "\\input amstex\n\\input amsppt.sty\n\\document\n\\vbox{{Before {align} inside}} after.{later}"
  ));
  assert_eq!(
    element(
      &xml,
      "<logical-block vattach=\"bottom\" xml:id=\"id1\">",
      "</logical-block>"
    )
    .lines()
    .filter(|l| l.starts_with("<p ")
      || l.starts_with("<logical-block")
      || l.starts_with("</logical-block"))
    .collect::<Vec<_>>()
    .join("\n"),
    "<logical-block vattach=\"bottom\" xml:id=\"id1\">\n<p xml:id=\"id1.1.1\">Before</p>\n\
     <p xml:id=\"id1.1.3\">inside</p>\n</logical-block>",
    "{xml}"
  );
}

/// The document's end (amsppt's `\enddocument`, plain's `\bye`, both `\outer` in TeX) is never read into a tag's text:
/// in a row it ends the row and the environment (`$$\align a&=b\\ c&=d\tag{5 e\enddocument` without its `\endalign`:
/// "Missing } inserted" for the tag's open group, as at any row end), in the `$$` form it is TeX's "Forbidden control
/// sequence found while scanning use of \tag" and the call is dropped (tex.web §336-339). r5 read `\enddocument` into
/// the row's tag (the second row lost, "Mouth is unexpectedly already closed"). The file's end without either still
/// raises "File ended while scanning use of \tag". In each shape the paragraph before the display is kept and the
/// document ends. pdftex: `$$\align a&=b\tag{5 c\enddocument` 3 errors ("Forbidden control sequence found while
/// scanning use of \align", "Missing $ inserted", "Display math should end with $$"), `$$ a=b\tag{5 c\enddocument` 3
/// (the first `… use of \tag`). The open row's last cells at `\enddocument` are dropped with or without a tag (RED
/// `amstex/amstex_align_unclosed_at_enddocument`). OXIDIZED_DESIGN_DIVERGENCES #575.
#[test]
fn amstex_tag_stops_at_the_document_end() {
  const MISSING: &str = "Missing } inserted before the row's end in \\tag";
  const FORBIDDEN: &str = "Forbidden control sequence found while scanning use of \\tag";
  const FILE_ENDED: &str = "File ended while scanning use of \\tag";
  const UNCLOSED: &str = "Closing tag \"ltx:document\" whose open descendents do not auto-close";
  // (body, errors, the equation id and its math)
  type Case<'a> = (&'a str, Vec<(&'a str, usize)>, (&'a str, &'a str));
  let cases: [Case; 6] = [
    (
      "$$\\align a&=b\\\\ c&=d\\tag{5 e\n\\enddocument\n",
      vec![(MISSING, 1)],
      ("id2.1.1", "<Math tex=\"\\displaystyle a=b\" text=\"a = b\""),
    ),
    (
      "$$\\align a&=b\\\\ c&=d\\tag{5 e\n\\bye\n",
      vec![(MISSING, 1)],
      ("id2.1.1", "<Math tex=\"\\displaystyle a=b\" text=\"a = b\""),
    ),
    (
      "$$\\align a&=b\\tag{5 c\n",
      vec![(FILE_ENDED, 1)],
      ("id2.1.1", "<Math tex=\"\\displaystyle a\" text=\"a\""),
    ),
    (
      "$$ a=b\\tag{5 c\n\\enddocument\n",
      vec![(FORBIDDEN, 1), (UNCLOSED, 1)],
      // the display never ends: `\enddocument` is read in it, as without the tag
      (
        "id2.1",
        "<Math mode=\"display\" tex=\"a=b\\enddocument\" text=\"a = b\"",
      ),
    ),
    (
      "$$ a=b\\tag 5 c\n\\bye\n",
      vec![(FORBIDDEN, 1)],
      ("id2.1", "<Math mode=\"display\" tex=\"a=b\" text=\"a = b\""),
    ),
    (
      "$$ a=b\\tag{5 c\n",
      vec![(FILE_ENDED, 1)],
      ("id2.1", "<Math mode=\"display\" tex=\"a=b\" text=\"a = b\""),
    ),
  ];
  for (body, errors, (id, math)) in cases {
    let tex =
      format!("\\input amstex\n\\input amsppt.sty\n\\document\nBefore paragraph.\n\n{body}");
    let (log, xml) = convert_counted(&tex);
    let total: usize = errors.iter().map(|(_, n)| n).sum();
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (total, 0),
      "{body}\n{log}"
    );
    for (message, n) in errors {
      assert_eq!(log.matches(message).count(), n, "{body}: {message}\n{log}");
    }
    assert_eq!(
      element(&xml, "<para xml:id=\"id1\">", "</para>"),
      "<para xml:id=\"id1\">\n<p xml:id=\"id1.1\">Before paragraph.</p>\n</para>",
      "{body}\n{xml}"
    );
    assert!(equation(&xml, id).contains(math), "{body}: {math}\n{xml}");
    // no tag holds the document's end
    assert!(!xml.contains("<tag"), "{body}\n{xml}");
    assert!(xml.trim_end().ends_with("</document>"), "{body}\n{xml}");
  }
}

/// Inline math inside a tag is tracked, so the `$` closing it is never the start of `$$`: `\tag{$a$$b$}\\` is two
/// inline maths in a row's tag (r5 read the `$$` inside the tag's group as a display end and stopped after `$a`, with
/// 10 errors), as is `\tag $a$$b$\\` in a row's tag cell (r5: 8 errors) and `\tag{$a$$b$}$$` in a display. pdftex: 0 errors on all
/// three, the tag "(ab)". Only the `$$` form's own level matches `$$` as tokens (`\def\tag#1$$`).
#[test]
fn amstex_tag_inline_maths_are_not_a_display_end() {
  let tag = |ids: [&str; 4]| {
    let math = |id: &str, c: &str| {
      format!(
        "<Math mode=\"inline\" tex=\"{c}\" text=\"{c}\" xml:id=\"{id}\">\n<XMath xml:id=\"{id}.1\">\n\
         <XMTok font=\"italic\" role=\"UNKNOWN\">{c}</XMTok>\n</XMath>\n</Math>"
      )
    };
    format!(
      "<tags>\n<tag>({}{})</tag>\n<tag role=\"refnum\">{}{}</tag>\n</tags>",
      math(ids[0], "a"),
      math(ids[1], "b"),
      math(ids[2], "a"),
      math(ids[3], "b")
    )
  };
  let row = ["id1.1.1.1", "id1.1.1.2", "id1.1.1.3", "id1.1.1.4"];
  for (body, id, ids, equations) in [
    (
      "$$\\align a&=b\\tag{$a$$b$}\\\\ c&=d\\endalign$$",
      "id1.1.1",
      row,
      2,
    ),
    (
      "$$\\align a&=b\\tag $a$$b$\\\\ c&=d\\endalign$$",
      "id1.1.1",
      row,
      2,
    ),
    (
      "$$ a=b\\tag{$a$$b$}$$",
      "id1.1",
      ["id1.1.2", "id1.1.3", "id1.1.4", "id1.1.5"],
      1,
    ),
  ] {
    let (log, xml) = convert_counted(&format!(
      "\\input amstex\n\\input amsppt.sty\n\\document\n{body}\n\\enddocument\n"
    ));
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "{body}\n{log}"
    );
    assert_tags_on(&xml, id, &tag(ids));
    assert_eq!(xml.matches("<tags>").count(), 1, "{body}\n{xml}");
    assert_eq!(
      xml.matches("<equation ").count(),
      equations,
      "{body}\n{xml}"
    );
    if equations == 2 {
      let eq = equation(&xml, "id1.1.2");
      assert!(
        eq.contains("<Math tex=\"\\displaystyle c=d\" text=\"c = d\"") && !eq.contains("<tags>"),
        "{body}\n{xml}"
      );
    }
  }
}

/// A LaTeX equation row with the AmSTeX pool loaded is a row too: no `\tag#1$$` governs it, so a `}` closing a group
/// opened before an unbraced `\tag` ends the tag (amsmath's, `\lx@amstex@ams@tag`) without the `$$` form's "extra }"
/// and runaway errors (r3 raised two per tag and put a `\par` in the row). A braced `\tag{…}` there is amsmath's
/// before this test (`amstex_braced_tag_in_latex_equation_is_amsmaths`).
#[test]
fn amstex_tag_in_latex_equation_row_ends_at_extra_brace() {
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\NoBlackBoxes\n\\begin{document}\n\
     \\begin{align} a&={b\\tag 2} c\\\\ e&=f\\end{align}\n\
     \\begin{equation} {a\\tag 6} b\\end{equation}\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  for (id, n, math) in [
    (
      "S0.Ex1",
      "2",
      "<Math tex=\"\\displaystyle a={b}c\" text=\"a = b * c\"",
    ),
    (
      "S0.Ex2",
      "6",
      "<Math mode=\"display\" tex=\"{a}b\" text=\"a * b\"",
    ),
  ] {
    assert_tags_on(
      &xml,
      id,
      &format!("<tags>\n<tag>({n})</tag>\n<tag role=\"refnum\">{n}</tag>\n</tags>"),
    );
    assert!(equation(&xml, id).contains(math), "{id}: {math}\n{xml}");
  }
  // the align's second row keeps its own number
  assert_tags_on(
    &xml,
    "S0.E1",
    "<tags>\n<tag>(1)</tag>\n<tag role=\"refnum\">1</tag>\n</tags>",
  );
  assert!(
    equation(&xml, "S0.E1").contains("<Math tex=\"\\displaystyle e=f\""),
    "{xml}"
  );
  assert_eq!(xml.matches("<equation ").count(), 3, "{xml}");
  assert_eq!(xml.matches("<tags>").count(), 3, "{xml}");
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

/// A LaTeX document that loads the AmSTeX pool (`\NoBlackBoxes` autoloads it, tex.rs; `\input amstex` too) has
/// AMS-TeX's `\tag`; in an equation row a braced `\tag{3}` (`\tag*{4}`) is amsmath's, its argument only: `a=b` stays
/// in the equation's math (the `\eqno` reading of Perl's `\let\tag\eqno` and 66e r2's both read `3 a=b` into the tag).
#[test]
fn amstex_braced_tag_in_latex_equation_is_amsmaths() {
  let (log, xml) = convert_counted(
    "\\documentclass{article}\n\\usepackage{amsmath}\n\\NoBlackBoxes\n\\begin{document}\n\
     \\begin{equation}\\tag{3} a=b\\end{equation}\n\
     \\begin{align} a&=b \\tag*{4}\\\\ c&=d \\tag 5\\end{align}\n\\end{document}\n",
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    element(&xml, "<equation xml:id=\"S0.Ex1\">", "</equation>"),
    "<equation xml:id=\"S0.Ex1\">\n<tags>\n<tag>(3)</tag>\n<tag role=\"refnum\">3</tag>\n</tags>\n\
     <Math mode=\"display\" tex=\"a=b\" text=\"a = b\" xml:id=\"S0.Ex1.m1\">\n<XMath xml:id=\"S0.Ex1.m1.1\">\n\
     <XMApp xml:id=\"S0.Ex1.m1.1.1\">\n<XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok>\n\
     <XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok>\n<XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok>\n\
     </XMApp>\n</XMath>\n</Math>\n</equation>",
    "{xml}"
  );
  for (id, tag, row) in [
    (
      "S0.Ex2",
      "<tag>4</tag>\n          <tag role=\"refnum\">4</tag>",
      "a=b",
    ),
    (
      "S0.Ex3",
      "<tag>(5)</tag>\n          <tag role=\"refnum\">5</tag>",
      "c=d",
    ),
  ] {
    assert_tags_on(
      &xml,
      id,
      &format!("<tags>\n          {tag}\n        </tags>"),
    );
    let math = format!("<Math tex=\"\\displaystyle {row}\"");
    assert!(equation(&xml, id).contains(&math), "{id}: {math}\n{xml}");
  }
  assert_eq!(xml.matches("<tags>").count(), 3, "{xml}");
}
