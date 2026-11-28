//! Regression test: a recoverable Fatal must not throw away the document.
//!
//! `digest_internal` (`latexml_oxide/src/core_interface.rs`) deliberately keeps
//! consuming input after a recoverable Fatal so it can "still produce partial
//! output" — Perl's `finishDigestion` L219-220. That intent silently only
//! worked when the failure landed in a LATER body: `digest_next_body`
//! accumulates into the stomach's `box_list` and hands it back only on the
//! success path, so a Fatal inside the FIRST body left the caller's `boxes`
//! empty and the run wrote a **39-byte empty document**.
//!
//! One pathological `\tikz` picture therefore cost a whole paper. Witnesses,
//! all ar5iv user reports and all previously 0-byte:
//!   * 2508.07407 (#556) → 31 KB (title/authors/abstract recovered)
//!   * 2405.19920 (#522) → 1.82 MB, 6 sections + 80 bibitems — essentially the
//!     complete paper, where same-host Perl produces **nothing** in 5 minutes
//!   * 2501.10235 (#551) → 1.7 KB
//!
//! `stomach::salvage_pending_box_lists` unwinds the stranded levels. For the
//! runaway guards (`Stomach:Recursion`) the innermost level IS the pathology —
//! a repeating window grown past 50k boxes — so it is dropped and the suspended
//! outer levels are kept: drop the offending construct, keep the document.

/// Text before, then the `calc`-coordinate `\tikz` picture that drives the
/// box-cycle guard (reduced from arXiv:2508.07407), then text after.
const RECURSION_TEX: &str = "\\documentclass{article}\n\
    \\usepackage{tikz}\n\
    \\usetikzlibrary{shapes.symbols,calc,positioning}\n\
    \\begin{document}\n\
    \\section{Before the bad picture}\n\
    UNIQUEMARKERBEFORE some ordinary prose that must survive.\n\
    \n\
    \\tikz[baseline=(env.base),node distance=4mm]{%\n\
      \\node[cloud, draw, inner sep=13pt, minimum width=40mm, minimum height=20mm] (env) {Env};\n\
      \\node[circle, draw, minimum size=6mm] (A1) at ($(env.west)+(10mm,6mm)$) {};\n\
      \\node[circle, draw, minimum size=6mm] (A2) at ($(env.east)+(-10mm,6mm)$) {};\n\
      \\node[circle, draw, minimum size=6mm] (A3) at ($(env.north)+(0,-24mm)$) {};\n\
      \\draw[->, thick] (A1) -- (A2);\n\
      \\draw[->, thick] (A2) -- (A3);\n\
      \\draw[->, thick] (A3) -- (A1);\n\
    }\n\
    \n\
    \\end{document}\n";

#[test]
fn recoverable_fatal_keeps_the_already_digested_document() {
  let (stderr, xml, status, status_code) =
    latexml::util::test::convert_with_status(RECURSION_TEX, None);

  // The Fatal MUST still be reported — salvaging partial output is not a
  // licence to downgrade the diagnostic. If a future fix makes this input
  // convert outright the assertion below still holds and this one should be
  // revisited deliberately, not deleted.
  assert!(
    stderr.contains("Fatal:") || xml.contains("UNIQUEMARKERBEFORE"),
    "expected either the Fatal to be reported or the document to convert:\n{stderr}",
  );

  // The point of the test: content digested BEFORE the pathological construct
  // survives. Pre-fix this file was 39 bytes with the prose gone.
  assert!(
    xml.contains("UNIQUEMARKERBEFORE"),
    "prose preceding the runaway construct was lost — the whole document was \
       thrown away by one bad picture (rec.xml is {} bytes):\n{xml}",
    xml.len(),
  );
  assert!(
    xml.len() > 400,
    "output is a {}-byte stub, so nothing was salvaged:\n{xml}",
    xml.len(),
  );

  // ...and the SUMMARY must agree with the log. Recovering boxes is NOT a
  // licence to reclassify the verdict: a Fatal-level raise stays Fatal in the
  // document's reported outcome (user policy 2026-07-28), and the graceful
  // salvage below is a *feature* of that Fatal, not a downgrade of it.
  //
  // `digest_internal` used to emit its recovered Fatal with the raw
  // `log::error!` macro rather than `Error::log_fatal`, so nothing reached
  // `note_status` and the tally stayed empty: this very input printed
  // `Fatal:Stomach:Recursion` and then signed off with "Conversion complete:
  // No obvious problems" — status code 0, i.e. "ok" to cortex (which reads
  // `get_status_code`) and clean to any check that does not scrape the log. A
  // run that reports a Fatal and summarises as problem-free is the false
  // negative CLAUDE.md forbids outright.
  // The verdict is the conversion's status and code (`ConversionResponse::{status, status_code}`; the CLI prints them
  // as "Conversion failed: …", its position as the run's final line is `119_final_status_report`'s guard). Both
  // directions, so neither seam can drift from the other again: a `Fatal:` in the log REQUIRES the fatal verdict
  // (code 3, the code cortex reads), and no `Fatal:` forbids it.
  if stderr.contains("Fatal:") {
    // "1 fatal error" alone: the Fatal prints one line, as Perl's does; the salvage path no longer repeats it as a
    // `Warning:recovery:digest_internal` (#792).
    assert_eq!(
      status, "1 fatal error",
      "the log reports a Fatal, so the status must be exactly \"1 fatal error\" — recovering boxes is not a \
         licence to reclassify the verdict.\n{stderr}",
    );
    assert_eq!(
      status_code, 3,
      "the fatal verdict's code:\n  {status}\n{stderr}"
    );
  } else {
    assert!(
      !status.contains("fatal") && status_code < 3,
      "the status claims a fatal that never appears in the log:\n  {status} ({status_code})\n{stderr}",
    );
  }

  // And the runaway's own boxes must NOT be grafted in: the guard trips at
  // 50k repeated boxes, so salvaging that level would produce a vast garbage
  // document rather than a small honest one.
  assert!(
    xml.len() < 2_000_000,
    "output is {} bytes — the runaway box window looks like it was salvaged \
       into the document instead of dropped",
    xml.len(),
  );
}

/// A `TooManyErrors` Fatal keeps everything digested before it: Perl's
/// `hardYankProcessing` rescues the innermost `@LaTeXML::LIST` for every Fatal
/// (Common/Error.pm:336-338) and latexmlc builds it (LaTeXML.pm:251-259). The
/// run wrote a 39-byte stub (27 TeX Live manuals, e.g. acro-manual, 120 KB in
/// Perl). The input after the Fatal stays unread, as in Perl, and the Fatal
/// stays the run's verdict.
#[test]
fn too_many_errors_keeps_the_already_digested_document() {
  let (stderr, xml, status, status_code) = latexml::util::test::convert_with_status(
    include_str!("../../../tools/perfect_kernel/repros/loader/too_many_errors_partial.tex"),
    None,
  );

  assert_eq!(
    stderr
      .lines()
      .filter(|l| l.starts_with("Fatal:TooManyErrors:MaxLimit(100)"))
      .count(),
    1,
    "{stderr}"
  );
  assert!(
    status.contains("errors; 1 fatal error") && status_code == 3,
    "{status} ({status_code})\n{stderr}"
  );
  assert!(xml.contains("<p>KEEPMEBEFOREMARKER prose.</p>"), "{xml}");
  assert!(!xml.contains("KEEPMEAFTERMARKER"), "{xml}");
}

type SalvageRow<'a> = (
  &'a str,
  &'a str,
  &'a [(&'a str, &'a str)],
  &'a str,
  usize,
  usize,
  &'a [&'a str],
);

/// Every digestion Fatal but the resource and token-loop guards keeps the text of every pending level digested before
/// it (`core_interface::finish_digestion_fatal`; Perl keeps the innermost list, Common/Error.pm:336-338, OD #251); the
/// run stays Fatal and reads nothing past it. A `document` environment is one body, so a Fatal in it kept no text at all
/// before DIGFATAL. (row, repro, its side files, the `Fatal:` line's head, `Error:` and `Warning:` lines, every `<p>`)
/// A delimited argument run off the end of input is no Fatal since EOFPUTBACK: `error_boundary`'s `Until:` rows.
#[test]
fn a_digestion_fatal_keeps_the_text_before_it() {
  const KEPT: &[&str] = &["<p>First paragraph.</p>", "<p>Before </p>"];
  let rows: &[SalvageRow] = &[
    (
      // a binding's own Fatal in a macro body (the RHODR shape)
      "a binding's Fatal",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_keeps_text_before.tex"
      ),
      &[(
        "lxmbf.sty.rhai",
        include_str!("../../../tools/perfect_kernel/repros/rhai-bindings/lxmbf.sty.rhai"),
      )],
      "Fatal:unexpected:Internal lxmbf gives up",
      0,
      0,
      KEPT,
    ),
    (
      // a Stomach-target Fatal that is no runaway guard keeps the innermost level too (synthetic: 1509.08999's and
      // 1807.06367's fire in the preamble, where there is no text to keep; 1311.6759 no longer reaches it)
      "Stomach:Misdefined",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/unbalanced_definition_keeps_text_before.tex"
      ),
      &[(
        "lxubd.sty.rhai",
        include_str!("../../../tools/perfect_kernel/repros/rhai-bindings/lxubd.sty.rhai"),
      )],
      "Fatal:Stomach:Misdefined Expansion of '\\lxubdx' has unbalanced {}",
      0,
      0,
      KEPT,
    ),
    (
      // a getter's Fatal stashed by the input's last token is the digestion's tail's, not the build's (the reviewer's
      // `tail/last.tex`: INT081RC6 built nothing and warned "build stopped early")
      "a stashed Fatal at the end of input",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/register_getter_fatal_at_end_of_input_keeps_text_before.tex"
      ),
      &[(
        "lxrgf.sty.rhai",
        include_str!("../../../tools/perfect_kernel/repros/rhai-bindings/lxrgf.sty.rhai"),
      )],
      "Fatal:unexpected:Internal lxrgf gives up",
      0,
      0,
      KEPT,
    ),
    (
      // the token-loop guards keep nothing: the salvage revived the loop in the build (2605.25400)
      "Timeout:Recursion",
      include_str!(
        "../../../tools/perfect_kernel/repros/expansion-primitives/expansion_loop_keeps_nothing.tex"
      ),
      &[],
      "Fatal:Timeout:Recursion Infinite expansion loop: a window of 2 token(s) repeated 100+ times",
      0,
      0,
      &[],
    ),
    (
      // the input-level limit is one of them (hep-th0409095, 1503.05624)
      "Mouth:Recursion",
      include_str!(
        "../../../tools/perfect_kernel/repros/expansion-primitives/scantokens_self_recursion_keeps_nothing.tex"
      ),
      &[],
      "Fatal:Mouth:Recursion TeX capacity exceeded, sorry [text input levels=15]",
      0,
      0,
      &[],
    ),
  ];
  let mut failures = Vec::new();
  for (name, tex, files, fatal, want_errors, want_warnings, paras) in rows {
    let (log, xml, status, code) = latexml::util::test::convert_files_with_status(tex, files, None);
    let fatals: Vec<&str> = log.lines().filter(|l| l.starts_with("Fatal:")).collect();
    let errors = log.lines().filter(|l| l.starts_with("Error:")).count();
    let warnings = log.lines().filter(|l| l.starts_with("Warning:")).count();
    let found = latexml::util::test::xml_elements(&xml, "p", &[]);
    let want: Vec<String> = paras
      .iter()
      .map(|p| latexml::util::test::normalize_markup(p))
      .collect();
    if fatals.len() != 1
      || !fatals[0].starts_with(fatal)
      || errors != *want_errors
      || warnings != *want_warnings
      || code != 3
      || !status.ends_with("1 fatal error")
      || found != want
    {
      failures.push(format!(
        "{name}: fatals {fatals:#?} (want one starting {fatal:?}), {errors} Error: lines (want {want_errors}), {warnings} Warning: \
         lines (want {want_warnings}), status {status:?} ({code}), paragraphs {found:#?} (want {want:#?})\n{xml}\n{log}"
      ));
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

type ClosedRow<'a> = (
  &'a str,
  &'a str,
  usize,
  usize,
  &'a [(&'a str, &'a [&'a str])],
);

/// A digestion Fatal closes each pending box list as the construct that opened it would (`stomach::close_pending_list_as`,
/// OD #251): a constructor's last argument or its body as its whatsit, an alignment's cell as its cell, a frontmatter
/// entry as its entry (placed by the document's end fallback). One row per closer: the constructor's argument (a
/// footnote), its body (a list; inline and display math), the frontmatter's `@now` entry under `\lx@frontmatterhere`'s
/// unfinished flush (an author), its `@until` entry (an abstract alone), the alignment cell (a tabular). A construct that
/// closes nothing of its own clears the closers inside it (an item's label); a closed whatsit is not `UNENDED` (a note's
/// `\par` and display); a caption's list entry counts the words the HTML hides, or keeps them in place outside a float.
/// A Fatal outside any construct keeps the text as it was: `a_digestion_fatal_keeps_the_text_before_it`'s rows. (row,
/// repro, `Error:` and `Warning:` lines, every element of each tag; one `Fatal:` line each, status 3)
#[test]
fn a_digestion_fatal_closes_each_unfinished_construct() {
  let rows: &[ClosedRow] = &[
    (
      "a footnote: the note",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_in_a_footnote.tex"
      ),
      0,
      0,
      &[("p", &[
        r#"<p>First paragraph.</p>"#,
        r#"<p>Before<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>Note text </note></p>"#,
      ])],
    ),
    (
      "an unfinished list: the itemize",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_in_an_unfinished_list.tex"
      ),
      0,
      0,
      &[
        ("itemize", &[
          r#"<itemize xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.p1"><p>One.</p></para></item><item xml:id="S0.I1.i2"><tags><tag>•</tag><tag role="typerefnum">2nd item</tag></tags><para xml:id="S0.I1.i2.p1"><p>Two</p></para></item></itemize>"#,
        ]),
        ("p", &[
          r#"<p>First paragraph.</p>"#,
          r#"<p>One.</p>"#,
          r#"<p>Two</p>"#,
        ]),
      ],
    ),
    (
      "inline math: the formula",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_in_inline_math.tex"
      ),
      0,
      0,
      &[("p", &[
        r#"<p>First paragraph.</p>"#,
        r#"<p>Before <Math mode="inline" tex="a+b" text="a + b" xml:id="p2.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math></p>"#,
      ])],
    ),
    (
      "display math: the equation",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_in_display_math.tex"
      ),
      0,
      0,
      &[
        ("equation", &[
          r#"<equation xml:id="S0.Ex1"><Math mode="display" tex="a+b" text="a + b" xml:id="S0.Ex1.m1"><XMath><XMApp><XMTok meaning="plus" role="ADDOP">+</XMTok><XMTok font="italic" role="UNKNOWN">a</XMTok><XMTok font="italic" role="UNKNOWN">b</XMTok></XMApp></XMath></Math></equation>"#,
        ]),
        ("p", &[r#"<p>First paragraph.</p>"#, r#"<p>Before</p>"#]),
      ],
    ),
    (
      // (the Warning is the rehearsal's own, which the Fatal ends too: INT081RC6 printed it)
      "an author: the creator",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_in_an_author.tex"
      ),
      0,
      1,
      &[
        ("title", &[r#"<title>T</title>"#]),
        ("creator", &[
          r#"<creator role="author"><personname>Ann Author</personname></creator>"#,
        ]),
        ("p", &[r#"<p>First paragraph.</p>"#]),
      ],
    ),
    (
      "an abstract alone: the abstract",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_in_an_abstract.tex"
      ),
      0,
      0,
      &[
        ("abstract", &[
          r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p>Abs</p></abstract>"#,
        ]),
        ("p", &[r#"<p>First paragraph.</p>"#, r#"<p>Abs</p>"#]),
      ],
    ),
    (
      "a tabular cell: the table",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_in_a_tabular_cell.tex"
      ),
      0,
      0,
      &[
        ("tabular", &[
          r#"<tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td></tr><tr><td align="left">c</td><td align="left">d</td></tr></tbody></tabular>"#,
        ]),
        ("p", &[
          r#"<p>First paragraph.</p>"#,
          r#"<p>Before <tabular vattach="middle"><tbody><tr><td align="left">a</td><td align="left">b</td></tr><tr><td align="left">c</td><td align="left">d</td></tr></tbody></tabular></p>"#,
        ]),
      ],
    ),
    (
      // a construct closing nothing of its own clears the closers inside it: `\item`'s label, digested in its
      // properties, stays text after the item before it (round 0 made it that item's second `<tags>`)
      "an item's label: its text in the list",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_in_an_item_label.tex"
      ),
      0,
      0,
      &[("itemize", &[
        r#"<itemize xml:id="S0.I1"><item xml:id="S0.I1.i1"><tags><tag>•</tag><tag role="typerefnum">1st item</tag></tags><para xml:id="S0.I1.i1.p1"><p>One.</p></para><para cssstyle="--ltx-skip-before:8.0pt" xml:id="S0.I1.i1.p2"><p>B</p></para></item></itemize>"#,
      ])],
    ),
    (
      // the same in a `{description}` (round 0: `<tags>` after the item's `<para>`, schema-invalid)
      "a description item's label: its text in the list",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_in_a_description_label.tex"
      ),
      0,
      0,
      &[("description", &[
        r#"<description xml:id="S0.I1"><item xml:id="S0.I1.ix1"><tags><tag><text font="bold">Lab</text></tag><tag role="typerefnum">item Lab</tag></tags><para xml:id="S0.I1.ix1.p1"><p>One.</p></para><para cssstyle="--ltx-skip-before:8.0pt" xml:id="S0.I1.ix1.p2"><p><text font="bold">La</text></p></para></item></description>"#,
      ])],
    ),
    (
      // the closed note is not `UNENDED` (OD #542): a `\par` in it does not close it (round 0 cut it there)
      "a footnote with a paragraph break: the whole note",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_after_a_par_in_a_footnote.tex"
      ),
      0,
      0,
      &[("p", &[
        r#"<p>First paragraph.</p>"#,
        r#"<p>Before<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>Note one. Note two </note></p>"#,
      ])],
    ),
    (
      // nor does a display in it
      "a footnote with a display: the whole note",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_after_display_math_in_a_footnote.tex"
      ),
      0,
      0,
      &[("note", &[
        r#"<note mark="1" role="footnote" xml:id="footnote1"><tags><tag>1</tag><tag role="refnum">1</tag><tag role="typerefnum">footnote 1</tag></tags>Note one <equation xml:id="S0.Ex1"><Math mode="display" tex="x" text="x" xml:id="S0.Ex1.m1"><XMath><XMTok font="italic" role="UNKNOWN">x</XMTok></XMath></Math></equation>note two </note>"#,
      ])],
    ),
    (
      // a caption's list entry, which the HTML does not show, counts the words kept only there (RED for the caption)
      "a caption: its list entry, counted",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_in_a_caption_red.tex"
      ),
      0,
      1,
      &[("figure", &[
        r#"<figure placement="h" xml:id="fig1"><p>X</p><toccaption><tag close=" ">1</tag>Cap </toccaption></figure>"#,
      ])],
    ),
    (
      // outside a float the entry waits for a caption that never comes: its text stays where it stands (round 0 dropped
      // it); the base's Error for its `<tag>` in a `<p>` (RED for the caption)
      "a caption outside a float: its text kept",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_in_a_caption_outside_a_float_red.tex"
      ),
      1,
      0,
      &[("inline-block", &[
        r#"<inline-block class="ltx_minipage" vattach="middle" width="142.3pt"><p>X <tag close=" ">1</tag>Mini</p></inline-block>"#,
      ])],
    ),
    (
      // an unfinished MATH construct keeps the leading math it owns, the rest goes back in order (FATALSALVAGE): the
      // body closer, after `\pmb{$\Gamma$}` left an inline math open (cond-mat9807003's shape; INT081TA kept the
      // eqnarray and the text inside an unparsed inline formula, E=7)
      "unfinished inline math: the formula, then the eqnarray and the text",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_after_unfinished_inline_math.tex"
      ),
      6,
      4,
      &[
        ("equation", &[
          "<equation xml:id=\"S0.E1\"><tags><tag>(1)</tag><tag role=\"autoref\">Equation\u{a0}1</tag><tag role=\"refnum\">1</tag></tags><Math mode=\"inline\" tex=\"\\displaystyle a\" text=\"a\" xml:id=\"S0.E1.m1\"><XMath><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok></XMath></Math><Math mode=\"inline\" tex=\"\\displaystyle=\" text=\"=\" xml:id=\"S0.E1.m2\"><XMath><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok></XMath></Math><Math mode=\"inline\" tex=\"\\displaystyle\\pmb{$\\alpha$}\" text=\"alpha\" xml:id=\"S0.E1.m3\"><XMath><XMTok font=\"bold italic\" name=\"alpha\" role=\"UNKNOWN\">α</XMTok></XMath></Math>.       </equation>",
        ]),
        ("p", &[
          "<p>variations in <Math mode=\"inline\" tex=\"\\pmb{$\\Gamma$}\" text=\"Gamma\" xml:id=\"p1.m1\"><XMath><XMTok font=\"blackboard bold\" name=\"Gamma\" role=\"UNKNOWN\">Γ</XMTok></XMath></Math><text class=\"ltx_markedasmath\">,</text></p>",
          "<p>alpha beta gamma words after. </p>",
        ]),
      ],
    ),
    (
      // the last-argument closer: `\bgroup` leaves the `multlined` cell's argument open (INT081TA: the text and the
      // section inside the formula, E=5)
      "an unfinished math argument: the equation, then the text and the section",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_after_an_unfinished_math_argument.tex"
      ),
      4,
      0,
      &[
        ("equation", &[
          "<equation><Math mode=\"display\" tex=\"\\begin{multlined}a{+b\\\\&#10;c\\end{equation}Textafter\\end{multlined}}\" text=\"multlined * T * e * x * t * a * f * t * e * r\" xml:id=\"p1.m1\"><XMath><XMApp><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMArray name=\"multlined\"><XMRow><XMCell align=\"right\"><XMApp><XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">b</XMTok></XMApp></XMCell></XMRow><XMRow><XMCell align=\"right\"><XMTok font=\"italic\" role=\"UNKNOWN\">c</XMTok></XMCell></XMRow></XMArray><XMTok font=\"italic\" role=\"UNKNOWN\">T</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">e</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">t</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">f</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">t</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">e</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">r</XMTok></XMApp></XMath></Math></equation>",
        ]),
        ("p", &[
          "<p>First paragraph.</p>",
          "<p>box<text font=\"italic\">here</text>.</p>",
          "<p><text font=\"italic\">Morewords</text></p>",
        ]),
        ("title", &[
          "<title font=\"italic\"><tag close=\" \"><text font=\"upright\">1</text></tag>Next</title>",
        ]),
      ],
    ),
    (
      // the math-cell closer: `\bgroup` leaves the `multlined` cell open (2203.12624's shape; INT081TA as above)
      "an unfinished math cell: the equation, then the text and the section",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_after_an_unfinished_math_cell.tex"
      ),
      4,
      0,
      &[
        ("equation", &[
          "<equation><Math mode=\"display\" tex=\"\\begin{multlined}\\end{multlined}\" text=\"a + b * c * &#10; * T * e * x * t * a * f * t * e * r\" xml:id=\"p1.m1\"><XMath><XMDual><XMApp><XMRef idref=\"p1.m1.1\"/><XMRef idref=\"p1.m1.2\"/><XMApp><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMRef idref=\"p1.m1.3\"/><XMRef idref=\"p1.m1.4\"/><XMRef idref=\"p1.m1.5\"/><XMRef idref=\"p1.m1.6\"/><XMRef idref=\"p1.m1.7\"/><XMRef idref=\"p1.m1.8\"/><XMRef idref=\"p1.m1.9\"/><XMRef idref=\"p1.m1.10\"/><XMRef idref=\"p1.m1.11\"/><XMRef idref=\"p1.m1.12\"/><XMRef idref=\"p1.m1.13\"/><XMRef idref=\"p1.m1.14\"/></XMApp></XMApp><XMArray name=\"multlined\"><XMRow><XMCell align=\"right\"><XMApp><XMTok meaning=\"plus\" role=\"ADDOP\" xml:id=\"p1.m1.1\">+</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.2\">a</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.3\">b</XMTok></XMApp></XMCell></XMRow><XMRow><XMCell align=\"right\"><XMApp><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.4\">c</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.5\"></XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.6\">T</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.7\">e</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.8\">x</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.9\">t</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.10\">a</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.11\">f</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.12\">t</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.13\">e</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.14\">r</XMTok></XMApp></XMCell></XMRow></XMArray></XMDual></XMath></Math></equation>",
        ]),
        ("p", &[
          "<p>First paragraph.</p>",
          "<p>box<text font=\"italic\">here</text>.</p>",
          "<p><text font=\"italic\">Morewords</text></p>",
        ]),
        ("title", &[
          "<title font=\"italic\"><tag close=\" \"><text font=\"upright\">1</text></tag>Next</title>",
        ]),
      ],
    ),
    (
      // RED (row R FATALSALVAGE-TEXTINMATH): Perl's `isMath` is false for the box `\text`, `\mbox` or `\hbox` makes
      // in math (Box.pm:81), so the closed formula ends at it and the rest is handed back as text; no word is lost
      "math after a \\text in the formula: split at the text (RED)",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_in_math_after_text_red.tex"
      ),
      0,
      0,
      &[("p", &[
        "<p>First paragraph.</p>",
        "<p>Before <Math mode=\"inline\" tex=\"a\" text=\"a\" xml:id=\"p2.m1\"><XMath><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok></XMath></Math> if <text font=\"italic\">b</text></p>",
      ])],
    ),
    (
      // a construct closed among the handed-back boxes opens into the part its closer filled, the `\parbox`'s text, not
      // its `[t]` and width (round 2 spliced every argument as text: "<Math>a+</Math>tinner words"); the formula ends at
      // the box (row R FATALSALVAGE-TEXTINMATH)
      "a parbox in an mbox in math: its text, no option text",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_in_a_parbox_in_math.tex"
      ),
      0,
      0,
      &[("p", &[
        "<p>First paragraph.</p>",
        "<p>Before <Math mode=\"inline\" tex=\"a+\" text=\"limit-from@(a, +)\" xml:id=\"p2.m1\"><XMath><XMApp><XMTok meaning=\"limit-from\"/><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok><XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok></XMApp></XMath></Math>inner words </p>",
      ])],
    ),
    (
      // the same with keyvals: `\rotatebox[origin=c]{90}` (round 2: "origin=c90.0inner words")
      "a keyval box in an mbox in math: its text, no keyval text",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_in_a_keyval_box_in_math.tex"
      ),
      0,
      0,
      &[("p", &[
        "<p>First paragraph.</p>",
        "<p>Before <Math mode=\"inline\" tex=\"a+\" text=\"limit-from@(a, +)\" xml:id=\"p2.m1\"><XMath><XMApp><XMTok meaning=\"limit-from\"/><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok><XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok></XMApp></XMath></Math>inner words </p>",
      ])],
    ),
    (
      // RED (row R FATALSALVAGE-MATHARGS): a math construct opens into its last argument alone, so the root's index is
      // lost (pdflatex prints ∛x; int kept the whole formula; round 2 spliced the index as text)
      "a root after a \\text in math: its radicand alone (RED)",
      include_str!(
        "../../../tools/perfect_kernel/repros/rhai-bindings/digest_fatal_in_a_root_after_text_in_math.tex"
      ),
      0,
      0,
      &[("p", &[
        "<p>First paragraph.</p>",
        "<p>Before <Math mode=\"inline\" tex=\"a\" text=\"a\" xml:id=\"p2.m1\"><XMath><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok></XMath></Math> if <text font=\"italic\">x</text></p>",
      ])],
    ),
  ];
  let files: &[(&str, &str)] = &[(
    "lxmbf.sty.rhai",
    include_str!("../../../tools/perfect_kernel/repros/rhai-bindings/lxmbf.sty.rhai"),
  )];
  let mut failures = Vec::new();
  for (name, tex, want_errors, want_warnings, elements) in rows {
    let (log, xml, status, code) = latexml::util::test::convert_files_with_status(tex, files, None);
    let fatals: Vec<&str> = log.lines().filter(|l| l.starts_with("Fatal:")).collect();
    let errors = log.lines().filter(|l| l.starts_with("Error:")).count();
    let warnings = log.lines().filter(|l| l.starts_with("Warning:")).count();
    let mut wrong = Vec::new();
    for (tag, want) in *elements {
      let found = latexml::util::test::xml_elements(&xml, tag, &[]);
      let want: Vec<String> = want
        .iter()
        .map(|e| latexml::util::test::normalize_markup(e))
        .collect();
      if found != want {
        wrong.push(format!("<{tag}>: {found:#?} (want {want:#?})"));
      }
    }
    if fatals.len() != 1
      || !fatals[0].starts_with("Fatal:unexpected:Internal lxmbf gives up")
      || errors != *want_errors
      || warnings != *want_warnings
      || code != 3
      || !status.ends_with("1 fatal error")
      || !wrong.is_empty()
    {
      failures.push(format!(
        "{name}: fatals {fatals:#?}, {errors} Error: lines (want {want_errors}), {warnings} Warning: lines (want \
         {want_warnings}), status {status:?} ({code}), {}\n{xml}\n{log}",
        wrong.join("; ")
      ));
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

type HandedBackRow<'a> = (
  &'a str,
  &'a str,
  usize,
  &'a [(&'a str, &'a [&'a str])],
  Option<Vec<String>>,
);

/// A `TooManyErrors` Fatal inside an unfinished math construct: the construct keeps the leading math it owns and the
/// boxes after it go back to the enclosing list (`stomach::split_off_after_leading_math`), and the build of what is kept
/// runs to its end. Once the digestion's end took the cap's Fatal, an `Error!` of the build no longer raises it again
/// (`error::spend_too_many_errors`; Perl's `Error` returns before its cap test after a Fatal, Common/Error.pm:362): every
/// one returned `Err(TooManyErrors)` and the build stopped at its first dropped Error (2203.12624). (row, repro, `Warning:`
/// lines, every element of each tag, every `<p>` where its text is right; 101 `Error:` lines, one `Fatal:`, status 3, no
/// "build stopped early"; the text the math-cell row digested in math is RED:
/// `an_unfinished_math_cell_hands_back_its_text_as_text_red`)
#[test]
fn a_too_many_errors_fatal_in_unfinished_math_keeps_what_follows() {
  let rows: &[HandedBackRow] = &[
    (
      // the body closer after `\pmb{$\Gamma$}` (cond-mat9807003; INT081TA: one `<Math text="absent">`)
      "unfinished inline math: the eqnarray and the text after it",
      include_str!(
        "../../../tools/perfect_kernel/repros/loader/too_many_errors_in_unfinished_inline_math.tex"
      ),
      4,
      &[("equation", &[
        "<equation xml:id=\"S0.E1\"><tags><tag>(1)</tag><tag role=\"autoref\">Equation\u{a0}1</tag><tag role=\"refnum\">1</tag></tags><Math mode=\"inline\" tex=\"\\displaystyle a\" text=\"a\" xml:id=\"S0.E1.m1\"><XMath><XMTok font=\"italic\" role=\"UNKNOWN\">a</XMTok></XMath></Math><Math mode=\"inline\" tex=\"\\displaystyle=\" text=\"=\" xml:id=\"S0.E1.m2\"><XMath><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok></XMath></Math><Math mode=\"inline\" tex=\"\\displaystyle\\pmb{$\\alpha$}\" text=\"alpha\" xml:id=\"S0.E1.m3\"><XMath><XMTok font=\"bold italic\" name=\"alpha\" role=\"UNKNOWN\">α</XMTok></XMath></Math>.       </equation>",
      ])],
      // (the 101st Error, in the 95th line's `x_1`, is the Fatal: the input after it stays unread)
      Some(vec![
        r#"<p>variations in <Math mode="inline" tex="\pmb{$\Gamma$}" text="Gamma" xml:id="p1.m1"><XMath><XMTok font="blackboard bold" name="Gamma" role="UNKNOWN">Γ</XMTok></XMath></Math><text class="ltx_markedasmath">,</text></p>"#.to_string(),
        format!("<p>alpha beta gamma words after. {}word x</p>", "word x_1 here ".repeat(94)),
      ]),
    ),
    (
      // the cell closer, then the build past the cap (2203.12624; INT081TA: the formula alone, "build stopped early")
      "an unfinished math cell: the equation, the text and the section after it",
      include_str!(
        "../../../tools/perfect_kernel/repros/loader/too_many_errors_after_an_unfinished_math_cell.tex"
      ),
      0,
      &[
        ("equation", &[
          "<equation><Math mode=\"display\" tex=\"\\begin{multlined}\\end{multlined}\" text=\"a + b * c * &#10; * S * e * t * t * i * n * g\" xml:id=\"p1.m1\"><XMath><XMDual><XMApp><XMRef idref=\"p1.m1.1\"/><XMRef idref=\"p1.m1.2\"/><XMApp><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMRef idref=\"p1.m1.3\"/><XMRef idref=\"p1.m1.4\"/><XMRef idref=\"p1.m1.5\"/><XMRef idref=\"p1.m1.6\"/><XMRef idref=\"p1.m1.7\"/><XMRef idref=\"p1.m1.8\"/><XMRef idref=\"p1.m1.9\"/><XMRef idref=\"p1.m1.10\"/><XMRef idref=\"p1.m1.11\"/><XMRef idref=\"p1.m1.12\"/></XMApp></XMApp><XMArray name=\"multlined\"><XMRow><XMCell align=\"right\"><XMApp><XMTok meaning=\"plus\" role=\"ADDOP\" xml:id=\"p1.m1.1\">+</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.2\">a</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.3\">b</XMTok></XMApp></XMCell></XMRow><XMRow><XMCell align=\"right\"><XMApp><XMTok meaning=\"times\" role=\"MULOP\">\u{2062}</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.4\">c</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.5\"></XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.6\">S</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.7\">e</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.8\">t</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.9\">t</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.10\">i</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.11\">n</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m1.12\">g</XMTok></XMApp></XMCell></XMRow></XMArray></XMDual></XMath></Math></equation>",
        ]),
        ("title", &[
          "<title font=\"italic\"><tag close=\" \"><text font=\"upright\">1</text></tag>Next</title>",
        ]),
      ],
      // (its text was digested in the cell's display math: RED, `an_unfinished_math_cell_hands_back_its_text_as_text_red`)
      None,
    ),
  ];
  let mut failures = Vec::new();
  for (name, tex, want_warnings, elements, want_paras) in rows {
    let (log, xml, status, code) = latexml::util::test::convert_with_status(tex, None);
    let fatals: Vec<&str> = log.lines().filter(|l| l.starts_with("Fatal:")).collect();
    let errors = log.lines().filter(|l| l.starts_with("Error:")).count();
    let warnings = log.lines().filter(|l| l.starts_with("Warning:")).count();
    let mut wrong = Vec::new();
    for (tag, want) in *elements {
      let found = latexml::util::test::xml_elements(&xml, tag, &[]);
      let want: Vec<String> = want
        .iter()
        .map(|e| latexml::util::test::normalize_markup(e))
        .collect();
      if found != want {
        wrong.push(format!("<{tag}>: {found:#?} (want {want:#?})"));
      }
    }
    if let Some(want) = want_paras {
      let want: Vec<String> = want
        .iter()
        .map(|e| latexml::util::test::normalize_markup(e))
        .collect();
      let found = latexml::util::test::xml_elements(&xml, "p", &[]);
      if found != want {
        wrong.push(format!("<p>: {found:#?} (want {want:#?})"));
      }
    }
    if fatals.len() != 1
      || !fatals[0].starts_with("Fatal:TooManyErrors:MaxLimit(100)")
      || errors != 101
      || warnings != *want_warnings
      || code != 3
      || !status.ends_with("1 fatal error")
      || log.contains("build stopped early")
      || !wrong.is_empty()
    {
      failures.push(format!(
        "{name}: fatals {fatals:#?}, {errors} Error: lines (want 101), {warnings} Warning: lines (want \
         {want_warnings}), status {status:?} ({code}), {}\n{xml}\n{log}",
        wrong.join("; ")
      ));
    }
  }
  assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

/// RED (row R FATALSALVAGE-MATHTEXT): the text after an unfinished math cell was digested in the cell's display math, so
/// the boxes handed back are math atoms and in-math text: "the text after" builds as `thetextafter` in italic, its `$q$`
/// as a bare `<XMText>` outside any formula, and the section's lines as one formula's atoms. pdflatex reads it as text
/// (its "Missing } inserted" closes the cell): `<p>Setting <Math>q</Math> the text after.</p>`, `<p>Words with
/// <Math>x</Math> inline. …`. A salvage cannot digest it again; this is the right reading it should reach.
#[test]
#[ignore = "RED: row R FATALSALVAGE-MATHTEXT (text digested in an unfinished cell's display math)"]
fn an_unfinished_math_cell_hands_back_its_text_as_text_red() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/loader/too_many_errors_after_an_unfinished_math_cell.tex"
  );
  let (_, xml, ..) = latexml::util::test::convert_with_status(tex, None);
  let paras = latexml::util::test::xml_elements(&xml, "p", &[]);
  let xmtext_in_math: usize = latexml::util::test::xml_elements(&xml, "Math", &[])
    .iter()
    .map(|math| math.matches("<XMText").count())
    .sum();
  let ok = paras.first().map(String::as_str) == Some("<p>First paragraph.</p>")
    && paras.get(1).is_some_and(|p| {
      p.starts_with("<p>Setting <Math mode=\"inline\"") && p.ends_with(" the text after.</p>")
    })
    && paras
      .iter()
      .any(|p| p.starts_with("<p>Words with <Math mode=\"inline\""))
    && xml.matches("<XMText").count() == xmtext_in_math;
  assert!(ok, "{paras:#?}\n{xml}");
}
