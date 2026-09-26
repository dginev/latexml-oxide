//! Batch 56js: doc.sty's change history and index prologue reach the HTML,
//! and `\index`/`\glossary` entries are read as makeindex reads them
//! (OXIDIZED_DESIGN_DIVERGENCES #318; KNOWN_PERL_ERRORS #281, #282). The
//! lists are built by the post stage, so each guard converts to core XML and
//! post-processes that XML to HTML in a second process, as the corpus does.
use std::process::Command;

use latexml::util::test::{assert_element, rng_error_count};

use super::perfect_kernel_batch46::{error_count, warning_count};

const RAW: &str = "--preload=[rawstyles,rawclasses]latexml.sty";

const DOC_CHANGES: &str =
  include_str!("../../../tools/perfect_kernel/repros/index/doc_changes_index_prologue.tex");
const QUOTE_VERB: &str =
  include_str!("../../../tools/perfect_kernel/repros/index/index_quote_verb_lshort.tex");
const ENTRY_WRITERS: &str =
  include_str!("../../../tools/perfect_kernel/repros/index/doc_index_entry_writers.tex");
const STRING_VERB: &str =
  include_str!("../../../tools/perfect_kernel/repros/index/index_string_verb_amsldoc.tex");
const MAKEGLOS: &str =
  include_str!("../../../tools/perfect_kernel/repros/index/glossary_list_env_makeglos.tex");
const RUNNING_TEXT: &str =
  include_str!("../../../tools/perfect_kernel/repros/index/glossary_running_text_keywords.tex");
const CHANGES_URL: &str =
  include_str!("../../../tools/perfect_kernel/repros/index/changes_url_moving_argument.tex");
const UNBALANCED: &str =
  include_str!("../../../tools/perfect_kernel/repros/index/changes_unbalanced_discarded.tex");
const STANDIN_PLACEMENT: &str =
  include_str!("../../../tools/perfect_kernel/repros/index/glossary_standin_in_item_and_cell.tex");
const GLOSSARY_DROPPED: &str =
  include_str!("../../../tools/perfect_kernel/repros/index/glossary_digests_argument_guitar.tex");
const MARKS_CLEANED: &str = include_str!(
  "../../../tools/perfect_kernel/repros/index/indexmark_cleaned_from_titles_captions.tex"
);

/// Convert `t.tex` to core XML, then post-process that XML alone to HTML.
/// Returns the ANSI-stripped stderr of both runs, the core XML and the HTML.
fn convert_xml_then_html(tex: &str) -> (String, String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  let core = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.xml",
      "--nocomments",
      "--timeout=110",
      RAW,
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn core");
  let post = Command::new(bin)
    .args([
      "--whatsin=xml",
      "t.xml",
      "--dest",
      "p.html",
      "--timeout=110",
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn post");
  let stderr = format!(
    "{}{}",
    String::from_utf8_lossy(&core.stderr),
    String::from_utf8_lossy(&post.stderr)
  )
  .replace('\u{1b}', "");
  let xml = std::fs::read_to_string(workdir.path().join("t.xml")).unwrap_or_default();
  let html = std::fs::read_to_string(workdir.path().join("p.html")).unwrap_or_default();
  (stderr, xml, html)
}

fn assert_clean_and_valid(stderr: &str, xml: &str) {
  assert_eq!(error_count(stderr), 0, "{stderr}");
  assert_eq!(warning_count(stderr), 0, "{stderr}");
  if let Some(invalid) = rng_error_count(xml) {
    assert_eq!(invalid, 0, "core XML is schema-invalid\n{xml}");
  }
}

/// doc.sty's `\changes` is a `\glossary` entry in doc's makeindex characters
/// (`v1.0>!!=General:>…`, doc.sty:626-650) and `\PrintChanges` inputs
/// `\jobname.gls`, which only makeindex writes (doc.sty:680). The entries were
/// dropped with `Warning:unexpected:glossary` and nothing was typeset. Now
/// the `.gls` stand-in runs doc's own `theglossary` (its "Change History"
/// heading) around a `glo` list MakeIndex fills: `v1.0 › General: › Alpha
/// stable release`, as pdflatex + `makeindex -s gglo.ist` print it (General
/// first: its sort key is `!`).
#[test]
fn doc_change_history_is_typeset() {
  let (stderr, xml, html) = convert_xml_then_html(DOC_CHANGES);
  assert_clean_and_valid(&stderr, &xml);
  assert_element(
    &html,
    "h2",
    &[r#"class="ltx_title ltx_title_section""#],
    r#"<h2 class="ltx_title ltx_title_section">Change History</h2>"#,
  );
  assert_element(
    &html,
    "li",
    &[r#"id="glo.v10""#],
    r##"<li id="glo.v10" class="ltx_indexentry"><span class="ltx_indexphrase">v1.0</span>
<ul class="ltx_indexlist">
<li id="glo.v10." class="ltx_indexentry"><span class="ltx_indexphrase">General:</span>
<ul class="ltx_indexlist">
<li id="glo.v10..Alphastablerelease" class="ltx_indexentry"><span class="ltx_indexphrase">Alpha stable release</span><span class="ltx_indexrefs"><span class="ltx_text"> </span><a href="#p1" title="" class="ltx_ref">p1</a></span></li></ul></li></ul></li>"##,
  );
  // A change inside `\begin{macro}{\foo}` is filed under the macro:
  // `v1.1>foo=\verb!*+\foo+:>…` — the `!` quotes the `*` (makeindex `quote`)
  // and the `:` after the `\verb` run is roman again.
  assert_element(
    &html,
    "li",
    &[r#"id="glo.v11.foo""#],
    r##"<li id="glo.v11.foo" class="ltx_indexentry"><span class="ltx_indexphrase"><code class="ltx_verbatim ltx_font_typewriter">\foo</code>:</span>
<ul class="ltx_indexlist">
<li id="glo.v11.foo.Bravorevisedstyles" class="ltx_indexentry"><span class="ltx_indexphrase">Bravo revised styles</span><span class="ltx_indexrefs"><span class="ltx_text"> </span><span class="ltx_ref ltx_ref_self">Document</span></span></li></ul></li>"##,
  );
  assert!(!stderr.contains("unexpected:glossary"), "{stderr}");
}

/// `\PrintIndex` inputs `\jobname.ind` (doc.sty:624). The stand-in runs
/// doc.sty's own `\theindex` — our `{theindex}` constructor would print
/// `\indexname` and drop it — so the index prologue (doc.sty:583-597) is
/// typeset, followed by the `idx` list.
#[test]
fn doc_index_prologue_is_typeset() {
  let (stderr, xml, html) = convert_xml_then_html(DOC_CHANGES);
  assert_clean_and_valid(&stderr, &xml);
  assert_element(
    &html,
    "div",
    &[r#"id="Sx2.p1""#],
    r#"<div id="Sx2.p1" class="ltx_para">
<p class="ltx_p">Numbers written in italic refer to the page
where the corresponding entry is described;
numbers underlined refer to the
definition; numbers in roman refer to the
pages
where the entry is used.</p>
</div>"#,
  );
}

/// doc.sty writes its usage and definition entries as
/// `\index{\@gtempa\actualchar\verb\quotechar*\verbatimchar\bslash\@gtempa
/// \verbatimchar…}` (doc.sty:1054-1093): the `\write` expands the macros, so
/// makeindex sees `foo=\verb!*+\foo+|…` and prints `\verb*+\foo+`. The run
/// was absorbed unexpanded (key `foo=*\verbatimchar\bslash\@gtempa…`, every
/// doc.sty manual: source2e, sunpath, circledtext); both marks of `\foo` are
/// one entry now, with hypdoc's encaps as their styles.
#[test]
fn doc_index_entries_read_their_macros() {
  let (stderr, xml, html) = convert_xml_then_html(DOC_CHANGES);
  assert_clean_and_valid(&stderr, &xml);
  assert!(!xml.contains("verbatimchar"), "{xml}");
  assert_element(
    &html,
    "section",
    &[r#"id="idx""#],
    r##"<section id="idx" class="ltx_index ltx_list_idx">
<ul class="ltx_indexlist">
<li id="idx.foo" class="ltx_indexentry"><span class="ltx_indexphrase"><code class="ltx_verbatim ltx_font_typewriter">\foo</code></span><span class="ltx_indexrefs"><span class="ltx_text"> </span><span class="ltx_text ltx_font_hdpindex"><span class="ltx_ref ltx_ref_self">Document</span></span>, <span class="ltx_text ltx_font_hdclindex"><a href="#p1" title="" class="ltx_ref">p1</a></span></span></li></ul>
</section>"##,
  );
}

/// makeindex's quote character makes the next character literal and is
/// dropped — including inside a `\verb` run: lshort's `\index{^@\verb"|^"|}`
/// is `\verb|^|` and amsldoc's `\index{"|@\verb"*+"\"|+}` is `\verb*+\|+`
/// (a quoted `\` before a live quote), while `\"u`, the escape rule, stays an
/// umlaut. The run took `"` as its delimiter: entries `|^`, `|_`, `*+¨`.
#[test]
fn index_verb_reads_makeindex_quotes() {
  let (stderr, xml, html) = convert_xml_then_html(QUOTE_VERB);
  assert_clean_and_valid(&stderr, &xml);
  for (id, entry) in [("idx.", r"^"), ("idx.a", r"_"), ("idx.b", r"\|")] {
    assert_element(
      &html,
      "li",
      &[&format!(r#"id="{id}""#)],
      &format!(
        r##"<li id="{id}" class="ltx_indexentry"><span class="ltx_indexphrase"><code class="ltx_verbatim ltx_font_typewriter">{entry}</code></span><span class="ltx_indexrefs"><span class="ltx_text"> </span><a href="#p1" title="" class="ltx_ref">p1</a></span></li>"##
      ),
    );
  }
  assert_element(
    &html,
    "li",
    &[r#"id="idx.Muller""#],
    r##"<li id="idx.Muller" class="ltx_indexentry"><span class="ltx_indexphrase">Müller</span><span class="ltx_indexrefs"><span class="ltx_text"> </span><a href="#p1" title="" class="ltx_ref">p1</a></span></li>"##,
  );
}

/// Each `\index` entry is split in the characters its writer used: a plain
/// `alpha@\textit{Alpha}` in a doc.sty document is makeindex's `@` (the
/// tcolorbox documentation library writes such entries in keytheorems-doc
/// and csvsimple-l3, where doc's `=` split nothing and the unsplit sort key
/// raised `Script _`), while `beta=\textsf{Beta}` (nlctdoc.cls's literal `=`,
/// testidx-manual printed `bib2gls=bib2gls`) and `\DescribeMacro`'s entry
/// are doc's. A robust
/// command in the macro-assembled `\verb` body is written by its name and
/// not run (lthooks.dtx `\DescribeMacro{\g__hook_\meta{hook}_code_prop}`
/// ran `\meta`'s internals: 65 errors on source2e's lthooks), and a `_` in a
/// change is underscore.sty's text underscore (lt3graph): read inside
/// `\changes`'s `\@sanitize` it is an other character, which the roman font
/// prints through OT1's slot as `˙`.
#[test]
fn index_entries_follow_their_writers() {
  let (stderr, xml, html) = convert_xml_then_html(ENTRY_WRITERS);
  assert_clean_and_valid(&stderr, &xml);
  assert_element(
    &html,
    "li",
    &[r#"id="idx.alpha""#],
    r##"<li id="idx.alpha" class="ltx_indexentry"><span class="ltx_indexphrase"><span class="ltx_text ltx_font_italic">Alpha</span></span><span class="ltx_indexrefs"><span class="ltx_text"> </span><a href="#p1" title="" class="ltx_ref">p1</a></span></li>"##,
  );
  assert_element(
    &html,
    "li",
    &[r#"id="idx.beta""#],
    r##"<li id="idx.beta" class="ltx_indexentry"><span class="ltx_indexphrase"><span class="ltx_text ltx_font_sansserif">Beta</span></span><span class="ltx_indexrefs"><span class="ltx_text"> </span><a href="#p1" title="" class="ltx_ref">p1</a></span></li>"##,
  );
  assert_element(
    &html,
    "li",
    &[r#"id="idx.foolanglenamerangle""#],
    r##"<li id="idx.foolanglenamerangle" class="ltx_indexentry"><span class="ltx_indexphrase"><code class="ltx_verbatim ltx_font_typewriter">\foo\meta {name}</code></span><span class="ltx_indexrefs"><span class="ltx_text"> </span><span class="ltx_text ltx_font_hdclindex"><a href="#p1" title="" class="ltx_ref">p1</a></span></span></li>"##,
  );
  assert_element(
    &html,
    "li",
    &[r#"id="glo.v20..Renamedgraphputtographget""#],
    r##"<li id="glo.v20..Renamedgraphputtographget" class="ltx_indexentry"><span class="ltx_indexphrase">Renamed <span class="ltx_text ltx_font_typewriter">\graph_put</span> to graph_get</span><span class="ltx_indexrefs"><span class="ltx_text"> </span><a href="#p1" title="" class="ltx_ref">p1</a></span></li>"##,
  );
}

/// amsldoc.cls:87-92 writes `\string\verb\quotechar*\verbatimchar…`: the
/// written `\verb` is the command when the `.ind` file is read back, so
/// `\cn{alpha}` is `\verb*+\alpha+` — the `\string` stringified the run's
/// brace instead. `\cn{\\*}` carries the control symbol `\*` into the
/// macro-assembled body, where amsldoc.cls:213's `\def\*#1` must not run.
#[test]
fn index_string_verb_is_the_command() {
  let (stderr, xml, html) = convert_xml_then_html(STRING_VERB);
  assert_clean_and_valid(&stderr, &xml);
  assert_element(
    &html,
    "section",
    &[r#"id="idx""#],
    r##"<section id="idx" class="ltx_index">
<h2 class="ltx_title ltx_title_index">Index</h2>

<ul class="ltx_indexlist">
<li id="idx." class="ltx_indexentry"><span class="ltx_indexphrase"><code class="ltx_verbatim ltx_font_typewriter">\\*</code></span><span class="ltx_indexrefs"><span class="ltx_text"> </span><a href="#p1" title="" class="ltx_ref">p1</a></span></li>
<li id="idx.alpha" class="ltx_indexentry"><span class="ltx_indexphrase"><code class="ltx_verbatim ltx_font_typewriter">\alpha</code></span><span class="ltx_indexrefs"><span class="ltx_text"> </span><a href="#p1" title="" class="ltx_ref">p1</a></span></li></ul>
</section>"##,
  );
}

/// makeglos's `theglossary` is `\section*` + `description`
/// (makeglos.sty:10-12), where an `ltx:index` may not stand: the stand-in's
/// list follows the environment.
#[test]
fn glossary_list_follows_a_list_environment() {
  let (stderr, xml, html) = convert_xml_then_html(MAKEGLOS);
  assert_clean_and_valid(&stderr, &xml);
  assert_element(
    &html,
    "section",
    &[r#"id="glo""#],
    r##"<section id="glo" class="ltx_index ltx_list_glo">
<ul class="ltx_indexlist">
<li id="glo.glossaryAlistofwordswithexplanations" class="ltx_indexentry"><span class="ltx_indexphrase">glossary:A list of words with explanations</span><span class="ltx_indexrefs"><span class="ltx_text"> </span><a href="#Sx1.p2" title="Glossary" class="ltx_ref"><span class="ltx_text ltx_ref_title">Glossary</span></a></span></li></ul>
</section>"##,
  );
}

/// After `\makeglossary`, `\glossary` in running text is an invisible mark
/// in list `glo`, no longer dropped with `Warning:unexpected:glossary`; the
/// HTML head keywords stay the document's index phrases (the webpage
/// stylesheet took every `ltx:indexphrase`, which added an empty keyword for
/// math phrases).
#[test]
fn glossary_in_running_text_is_a_mark() {
  let (stderr, xml, html) = convert_xml_then_html(RUNNING_TEXT);
  assert_clean_and_valid(&stderr, &xml);
  assert_element(
    &xml,
    "indexmark",
    &[r#"inlist="glo""#],
    r##"<indexmark inlist="glo">
        <indexphrase key="x the unknown"><Math mode="inline" tex="x" text="x" xml:id="p1.m2">
            <XMath>
              <XMTok font="italic" role="UNKNOWN">x</XMTok>
            </XMath>
          </Math> the unknown</indexphrase>
      </indexmark>"##,
  );
  // A void element: the whole `<meta …>` tag is the element.
  assert_eq!(html.matches(r#"name="keywords""#).count(), 1, "{html}");
  assert!(
    html.contains(r#"<meta name="keywords" lang="en" content="alpha">"#),
    "{html}"
  );
}

/// `\url` is robust (hyperref.sty:4801 `\DeclareRobustCommand*`): doc.sty's
/// `\changes` `\protected@edef` (doc.sty:627) must leave it for the typeset
/// entry. As a plain macro its reader was expanded into the entry (ltsect.dtx
/// v1.1b: mode errors and a runaway that ended source2e early).
#[test]
fn url_in_a_change_entry_stays_a_url() {
  let (stderr, xml, html) = convert_xml_then_html(CHANGES_URL);
  assert_clean_and_valid(&stderr, &xml);
  assert_element(
    &html,
    "li",
    &[r#"id="glo.v11b""#],
    r##"<li id="glo.v11b" class="ltx_indexentry"><span class="ltx_indexphrase">v1.1b</span>
<ul class="ltx_indexlist">
<li id="glo.v11b." class="ltx_indexentry"><span class="ltx_indexphrase">General:</span>
<ul class="ltx_indexlist">
<li id="glo.v11b..Preventprotrusionhttpstexstackexchangecomq17278510109" class="ltx_indexentry"><span class="ltx_indexphrase">Prevent protrusion (<a href="https://tex.stackexchange.com/q/172785/10109" title="" class="ltx_ref ltx_url ltx_font_typewriter">https://tex.stackexchange.com/q/172785/10109</a>)</span><span class="ltx_indexrefs"><span class="ltx_text"> </span><a href="#p1" title="" class="ltx_ref">p1</a></span></li></ul></li></ul></li>"##,
  );
}

/// Without `\makeglossary`, `\glossary` is latex.ltx:17742's
/// `\@bsphack\begingroup\@sanitize\@index`: the entry is read and dropped,
/// never typeset. guitar.dtx:397-399's `\changes` text `\protected@edef`s to
/// `\def{\relax}`, which digesting the entry ran ("Missing control sequence
/// inserted"); the arXiv papers that call `\glossary` in running text
/// (cs/9809003, math/9608214, nucl-th/9311001) call no `\makeglossary` either.
#[test]
fn glossary_is_dropped_without_makeglossary() {
  let (stderr, xml, _html) = convert_xml_then_html(GLOSSARY_DROPPED);
  assert_clean_and_valid(&stderr, &xml);
  assert!(!xml.contains("<indexmark"), "{xml}");
  assert_element(&xml, "p", &[], "<p>Text more.</p>");
}

/// An entry whose groups do not balance is discarded with
/// `Warning:malformed:indexentry` (Perl pool:4332-4336), as makeindex rejects
/// it: ltmath.dtx v1.2i's `\cs{\bslash}` writes `\cs{\}`, and read as a phrase it
/// ran the entry's reader off its end.
#[test]
fn unbalanced_entry_is_discarded() {
  let (stderr, xml, html) = convert_xml_then_html(UNBALANCED);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 1, "{stderr}");
  assert_eq!(
    stderr.matches("Warning:malformed:indexentry").count(),
    1,
    "{stderr}"
  );
  assert_eq!(xml.matches("<indexmark").count(), 1, "{xml}");
  assert_element(
    &html,
    "section",
    &[r#"id="glo""#],
    r##"<section id="glo" class="ltx_index ltx_list_glo">
<ul class="ltx_indexlist">
<li id="glo.v13" class="ltx_indexentry"><span class="ltx_indexphrase">v1.3</span>
<ul class="ltx_indexlist">
<li id="glo.v13." class="ltx_indexentry"><span class="ltx_indexphrase">General:</span>
<ul class="ltx_indexlist">
<li id="glo.v13..Keptentry" class="ltx_indexentry"><span class="ltx_indexphrase">Kept entry</span><span class="ltx_indexrefs"><span class="ltx_text"> </span><a href="#p1" title="" class="ltx_ref">p1</a></span></li></ul></li></ul></li></ul>
</section>"##,
  );
}

/// The stand-in's list placeholder where no open element admits an
/// `ltx:index`: after an `\item`'s text or inside a tabular cell it goes after
/// the list or table (`Error:malformed:ltx:index isn't allowed in <ltx:p>` /
/// `<ltx:td>` before), and an inline input ends its paragraph first.
#[test]
fn standin_list_is_placed_where_an_index_may_stand() {
  let (stderr, xml, _html) = convert_xml_then_html(STANDIN_PLACEMENT);
  assert_clean_and_valid(&stderr, &xml);
  assert_eq!(xml.matches("<index ").count(), 3, "{xml}");
  for id in ["glo", "gloa", "glob"] {
    assert_element(
      &xml,
      "index",
      &[&format!(r#"xml:id="{id}""#)],
      &format!(r#"<index lists="glo" xml:id="{id}"/>"#),
    );
  }
  assert!(
    !xml.contains("<td align=\"left\">cell Glossary prologue text and epilogue<index"),
    "{xml}"
  );
}

/// Perl `Scan::cleanNode` (`Scan.pm` L206-214): the title, caption and tag
/// copies Scan stores lose their `ltx:indexmark`s, so neither an `\index` nor
/// a `\glossary` (after `\makeglossary`) phrase reaches a list-of-figures
/// entry ("1Cap icap" before), a link's tooltip ("1 Title gsec isec" before)
/// or an `\eqref` ("(A t)" before). The missing space in "1Cap" (Perl and
/// pdflatex: "1 Cap") is older: captions are stored as text, which drops the
/// tag's `close`.
#[test]
fn index_marks_stay_out_of_stored_titles_and_captions() {
  let (stderr, xml, html) = convert_xml_then_html(MARKS_CLEANED);
  assert_clean_and_valid(&stderr, &xml);
  assert_element(
    &html,
    "nav",
    &[r#"class="ltx_TOC ltx_list_lof ltx_toc_lof""#],
    r##"<nav class="ltx_TOC ltx_list_lof ltx_toc_lof"><h6 class="ltx_title ltx_title_contents">List of Figures</h6>
<ol class="ltx_toclist">
<li class="ltx_tocentry ltx_tocentry_figure"><a href="#S1.F1" title="Figure 1" class="ltx_ref"><span class="ltx_text ltx_ref_title">1Cap</span></a></li>
<li class="ltx_tocentry ltx_tocentry_figure"><a href="#S1.F2" title="Figure 2" class="ltx_ref"><span class="ltx_text ltx_ref_title">2Plot</span></a></li>
</ol></nav>"##,
  );
  assert_element(
    &html,
    "li",
    &[r#"id="idx.isec""#],
    r##"<li id="idx.isec" class="ltx_indexentry"><span class="ltx_indexphrase">isec</span><span class="ltx_indexrefs"><span class="ltx_text"> </span><a href="#S1" title="1 Title" class="ltx_ref"><span class="ltx_text ltx_ref_tag">§1</span></a></span></li>"##,
  );
  assert_element(
    &html,
    "span",
    &[r#"class="ltx_text ltx_font_bold""#],
    r##"<span class="ltx_text ltx_font_bold">See (<a href="#S1.Ex1" title="In 1 Title" class="ltx_ref"><span class="ltx_text ltx_ref_tag">A</span></a>).</span>"##,
  );
}
