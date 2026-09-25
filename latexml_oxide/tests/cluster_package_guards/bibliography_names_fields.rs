//! Bibliography rendering driven by the style (perfect-kernel round 12, P2/P3).
//! The bibliography is built by the post stage, so every guard converts to
//! HTML — both in one process (`--dest t.html`) and through the split post
//! session the corpus uses (`t.tex → t.xml`, then `--whatsin=xml t.xml`).
use std::process::Command;

use super::perfect_kernel_batch46::{convert_files, error_count, warning_count};

const RAW: &str = "--preload=[rawstyles,rawclasses]latexml.sty";

const GIVEN_BIB: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_given_names.bib");

/// Convert `t.tex` (with side `files`) to HTML in one process. Returns
/// (ANSI-stripped stderr, HTML).
fn convert_html(tex: &str, files: &[(&str, &str)]) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  for (name, content) in files {
    std::fs::write(workdir.path().join(name), content).expect("write side file");
  }
  let output = Command::new(bin)
    .args([
      "t.tex",
      "--dest",
      "t.html",
      "--nocomments",
      "--timeout=110",
      RAW,
    ])
    .current_dir(workdir.path())
    .output()
    .expect("spawn latexml_oxide");
  let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let html = std::fs::read_to_string(workdir.path().join("t.html")).unwrap_or_default();
  (stderr, html)
}

/// Convert `t.tex` to core XML, then post-process that XML alone to HTML in a
/// second process (`--whatsin=xml`, post_sweep.sh's split). Returns the
/// (ANSI-stripped) stderr of both runs and the HTML.
fn convert_split_html(tex: &str, files: &[(&str, &str)]) -> (String, String) {
  let bin = env!("CARGO_BIN_EXE_latexml_oxide");
  let workdir = tempfile::tempdir().expect("create tempdir");
  std::fs::write(workdir.path().join("t.tex"), tex).expect("write t.tex");
  for (name, content) in files {
    std::fs::write(workdir.path().join(name), content).expect("write side file");
  }
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
  let html = std::fs::read_to_string(workdir.path().join("p.html")).unwrap_or_default();
  (stderr, html)
}

/// The rendered entry of `bib_given_names.bib` with `authors` in its first
/// bibblock (`title` as the style's reader cases it).
fn castro_bibitem(authors: &str, title: &str) -> String {
  format!(
    r#"<li id="bib.bib1" class="ltx_bibitem ltx_bib_book"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[1]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">{authors}</span><span class="ltx_text ltx_bib_year"> (2003)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">{title}</span>.
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub</span>.
</span></li>"#
  )
}

const FULL: &str = "Adam-Troy Castro and Toni Morrison";
const INITIALS: &str = "A. Castro and T. Morrison";

fn assert_castro(stderr: &str, html: &str, authors: &str, title: &str) {
  assert_eq!(error_count(stderr), 0, "{stderr}");
  assert_eq!(warning_count(stderr), 0, "{stderr}");
  assert_eq!(html.matches("class=\"ltx_bibitem ").count(), 1, "{html}");
  latexml::util::test::assert_element(
    html,
    "li",
    &[r#"class="ltx_bibitem"#],
    &castro_bibitem(authors, title),
  );
}

/// Given names follow the `.bst` name template: plain.bst:191
/// `{ff~}{vv~}{ll}{, jj}` spells them out, as pdflatex + bibtex print
/// ("Adam-Troy Castro and Toni Morrison"). Perl's `do_name`
/// (MakeBibliography.pm:555-566) always abbreviates. Witnesses bookshelf/spines
/// (bookshelf.bst:29), asmeconf/asmeconf-template (asmeconf.bst:689).
#[test]
fn bst_full_name_template_spells_given_names_out() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_given_names_plain.tex");
  let files = [("bib_given_names.bib", GIVEN_BIB)];
  let (stderr, html) = convert_html(tex, &files);
  assert_castro(&stderr, &html, FULL, "With unclean hands");
  let (stderr, html) = convert_split_html(tex, &files);
  assert_castro(&stderr, &html, FULL, "With unclean hands");
}

/// CONTROL: abbrv.bst:191 `{f.~}{vv~}{ll}{, jj}` asks for initials, which stay
/// Perl's per-word form (bibtex prints "A.-T. Castro": it also abbreviates
/// each hyphen-joined part).
#[test]
fn bst_initials_template_abbreviates_given_names() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_given_names_abbrv.tex");
  let (stderr, html) = convert_html(tex, &[("bib_given_names.bib", GIVEN_BIB)]);
  assert_castro(&stderr, &html, INITIALS, "With unclean hands");
}

/// biblatex spells given names out by default (`giveninits=false`), as
/// pdflatex + biber print; witnesses biblatex-chicago cms-*-sample,
/// windycity, biblatex-fiwi.
#[test]
fn biblatex_default_spells_given_names_out() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_given_names_biblatex.tex");
  let files = [("bib_given_names.bib", GIVEN_BIB)];
  let (stderr, html) = convert_html(tex, &files);
  assert_castro(&stderr, &html, FULL, "With Unclean Hands");
  let (stderr, html) = convert_split_html(tex, &files);
  assert_castro(&stderr, &html, FULL, "With Unclean Hands");
}

/// biblatex's `giveninits` (or its legacy alias `firstinits`,
/// blx-compat.def:224-229) as a package option or through
/// `\ExecuteBibliographyOptions` (a style's .bbx, phys.bbx:49 / lncs.bbx:5,
/// or the preamble) abbreviates. (A per-type `\ExecuteBibliographyOptions[book]`
/// is not modelled; no TL style sets `giveninits` per type.)
#[test]
fn biblatex_giveninits_abbreviates_given_names() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_given_names_biblatex.tex");
  let files = [("bib_given_names.bib", GIVEN_BIB)];
  let package = r"\usepackage{biblatex}";
  for (setting, authors) in [
    (r"\usepackage[giveninits=true]{biblatex}", INITIALS),
    (r"\usepackage[firstinits]{biblatex}", INITIALS),
    (
      r"\usepackage{biblatex}\ExecuteBibliographyOptions{maxnames=3,giveninits}",
      INITIALS,
    ),
  ] {
    let tex = tex.replace(package, setting);
    let (stderr, html) = convert_html(&tex, &files);
    assert_castro(&stderr, &html, authors, "With Unclean Hands");
  }
  let tex = tex.replace(package, r"\usepackage[giveninits=true]{biblatex}");
  let (stderr, html) = convert_split_html(&tex, &files);
  assert_castro(&stderr, &html, INITIALS, "With Unclean Hands");
}

/// The style decides (P2 review): a native style's .bbx, which the binding
/// does not load, still sets its option defaults (ieee.bbx:26-34 `giveninits`,
/// reached from ieee-alphabetic.bbx:13 too); a style's name format prints
/// initials or full names whatever `giveninits` says (apa.bbx:611-631
/// `\namepartgiveni`; biblatex-cse.bbx:61-68 `\namepartgiven` although :37
/// sets `giveninits`). pdflatex + biber: "A.-T. Castro and T. Morrison" (ieee,
/// ieee-alphabetic), "Castro, A.-T., & Morrison, T." (apa), "Castro,
/// Adam-Troy and Morrison, Toni" (biblatex-cse). Witness
/// biblatex-apa/biblatex-apa-test.
#[test]
fn biblatex_style_decides_given_names() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_given_names_style_format.tex");
  let files = [("bib_given_names.bib", GIVEN_BIB)];
  for (style, authors) in [
    ("ieee", INITIALS),
    ("ieee-alphabetic", INITIALS),
    ("biblatex-cse", FULL),
  ] {
    let tex = tex.replace("style=ieee", &format!("style={style}"));
    let (stderr, html) = convert_html(&tex, &files);
    assert_castro(&stderr, &html, authors, "With Unclean Hands");
  }
  // The same rule without a style file, so it runs where biblatex-cse (2025)
  // is not installed (CI's TeX Live): an `author` format that prints
  // `\namepartgiven` wins over `giveninits`.
  let own_format = tex.replace(
    "\\usepackage[style=ieee]{biblatex}",
    "\\usepackage[giveninits]{biblatex}\n\\DeclareNameFormat{author}{\\usebibmacro{name:family-given}\
     {\\namepartfamily}{\\namepartgiven}{\\namepartprefix}{\\namepartsuffix}}",
  );
  assert!(own_format.contains("DeclareNameFormat"), "{own_format}");
  let (stderr, html) = convert_html(&own_format, &files);
  assert_castro(&stderr, &html, FULL, "With Unclean Hands");
  let (stderr, html) = convert_split_html(tex, &files);
  assert_castro(&stderr, &html, INITIALS, "With Unclean Hands");
  // apa is an author-year style: the tag carries the names.
  let tex = tex.replace("style=ieee", "style=apa");
  let (stderr, html) = convert_html(&tex, &files);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(html.matches("class=\"ltx_bibitem ").count(), 1, "{html}");
  latexml::util::test::assert_element(
    &html,
    "li",
    &[r#"class="ltx_bibitem"#],
    r#"<li id="bib.bib1" class="ltx_bibitem ltx_bib_book"><span class="ltx_tag ltx_bib_author-year ltx_role_refnum ltx_tag_bibitem">Castro and Morrison (2003)</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">A. Castro and T. Morrison</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">With Unclean Hands</span>.
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub</span>.
</span></li>"#,
  );
}

/// A name list ending in `others` is comma-separated before "et al.", as
/// Perl's `do_names` (MakeBibliography.pm:568-584) takes the separator from
/// the list's length before popping the `others`: "A. Castro, T. Morrison,
/// et al." (same-host Perl), in the style's given-name form.
#[test]
fn bst_name_list_with_others_is_comma_separated() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_names_etal.tex");
  let bib = include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_names_etal.bib");
  for (style, authors) in [
    ("abbrv", "A. Castro, T. Morrison, "),
    ("plain", "Adam-Troy Castro, Toni Morrison, "),
  ] {
    let tex = tex.replace("{abbrv}", &format!("{{{style}}}"));
    let (stderr, html) = convert_html(&tex, &[("bib_names_etal.bib", bib)]);
    let authors = format!(r#"{authors}<span class="ltx_text ltx_bib_etal">et al.</span>"#);
    assert_castro(&stderr, &html, &authors, "With unclean hands");
  }
}

/// biber's `%` comments in a `.bib` (perfect_kernel_batch56::
/// biber_bib_percent_comments_keep_the_entry) stay comments when the
/// bibliography style is recorded as `biblatex-giveninits` (pre_bibtex.rs).
#[test]
fn biblatex_giveninits_keeps_percent_comment_entries() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/biber_percent_comments.tex")
      .replace("[backend=biber]", "[backend=biber,giveninits]");
  let bib =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/biber_percent_comments.bib");
  let (stderr, html) = convert_html(&tex, &[("biber_percent_comments.bib", bib)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(html.matches("class=\"ltx_bibitem ").count(), 1, "{html}");
  latexml::util::test::assert_element(
    &html,
    "li",
    &[r#"class="ltx_bibitem"#],
    r#"<li id="bib.bib1" class="ltx_bibitem ltx_bib_book"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[1]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">T. Morrison</span><span class="ltx_text ltx_bib_year"> (2004)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Beloved Things</span>.
</span></li>"#,
  );
}

const FIELDS_BIB: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_second_tier_fields.bib");

/// Each entry of `biblatex_second_tier_fields.bib`, whole: `@inbook` (main
/// title nested in the book host, book subtitle), `@inproceedings` (the
/// proceedings' event), `@article` (issue title in the journal host), `@book`
/// (its own main title, a typed `editora`), `@unpublished` (an event of its
/// own), `@periodical` (its issue).
const FIELDS_BIBITEMS: [(&str, &str); 6] = [
  (
    "bib.bib1",
    r#"<li id="bib.bib1" class="ltx_bibitem ltx_bib_inbook"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[1]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Jane Doe</span><span class="ltx_text ltx_bib_year"> (2001)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Chapter Heading</span>.
</span>
<span class="ltx_bibblock">In <span class="ltx_text ltx_bib_maintitle">Collected Alpha Works</span>. <span class="ltx_text ltx_bib_inbook">Volume Name</span>, <span class="ltx_text ltx_bib_subtitle">Bravo Subtitle</span>,
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub</span>, <span class="ltx_text ltx_bib_place">Town</span>.
</span></li>"#,
  ),
  (
    "bib.bib2",
    r#"<li id="bib.bib2" class="ltx_bibitem ltx_bib_inproceedings"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[5]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Rick Roe</span><span class="ltx_text ltx_bib_year"> (2002)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Talk Title</span>.
</span>
<span class="ltx_bibblock">In <span class="ltx_text ltx_bib_inbook">Proceedings Book</span>, <span class="ltx_text ltx_bib_event">Delta Symposium</span> (<span class="ltx_text ltx_bib_place">Echo City</span>), <span class="ltx_text ltx_bib_date">2002-05-06</span>
</span></li>"#,
  ),
  (
    "bib.bib3",
    r#"<li id="bib.bib3" class="ltx_bibitem ltx_bib_article"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[4]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Paul Poe</span><span class="ltx_text ltx_bib_year"> (2003)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Article Title</span>.
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_journal">Journal Name</span>: <span class="ltx_text ltx_bib_issuetitle">Foxtrot Special Issue</span>.
</span></li>"#,
  ),
  (
    "bib.bib4",
    r#"<li id="bib.bib4" class="ltx_bibitem ltx_bib_book"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[2]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Mary Moe</span><span class="ltx_text ltx_bib_year"> (2004)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_maintitle">Hotel Collected Papers</span>. <span class="ltx_text ltx_bib_title">Golf Letters</span>.
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub</span>.
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_editor">Karl Kilo (compiler)</span>
</span></li>"#,
  ),
  (
    "bib.bib5",
    r#"<li id="bib.bib5" class="ltx_bibitem ltx_bib_unpublished"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[6]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Zed Zoe</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">India Lecture</span>.
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_event">Juliet Festival</span> (<span class="ltx_text ltx_bib_place">Lima Hall</span>), <span class="ltx_text ltx_bib_date">2005-06-07</span>.
</span></li>"#,
  ),
  (
    "bib.bib6",
    r#"<li id="bib.bib6" class="ltx_bibitem ltx_bib_periodical"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[3]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_editor">Olga Ode (Ed.)</span><span class="ltx_text ltx_bib_year"> (2006)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Mike Quarterly</span>.
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_issuetitle">November Issue</span>.
</span></li>"#,
  ),
];

fn assert_fields_bibitems(stderr: &str, html: &str) {
  assert_eq!(error_count(stderr), 0, "{stderr}");
  assert_eq!(warning_count(stderr), 0, "{stderr}");
  assert_eq!(html.matches("class=\"ltx_bibitem ").count(), 6, "{html}");
  for (id, bibitem) in FIELDS_BIBITEMS {
    let selector = format!(r#"id="{id}""#);
    latexml::util::test::assert_element(html, "li", &[selector.as_str()], bibitem);
  }
}

/// biblatex's second-tier fields reach the bibliography (P3): `maintitle`,
/// `booksubtitle`, `issuetitle`, `eventtitle`/`venue`/`eventdate` and
/// `editora` with `editoratype` were unprinted `ltx:bib-data`. pdflatex +
/// biber prints each (biblatex standard.bbx:211/260/309/311/578/710); the
/// repro's HTML recall against that PDF goes 39.1 % → 95.7 %. Witnesses
/// biblatex-chicago cms-trad/dates/notes-sample, biblatex-fiwi,
/// biblatex-apa-test.
#[test]
fn biblatex_second_tier_fields_are_printed() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_second_tier_fields.tex");
  let files = [("biblatex_second_tier_fields.bib", FIELDS_BIB)];
  let (stderr, html) = convert_html(tex, &files);
  assert_fields_bibitems(&stderr, &html);
  let (stderr, html) = convert_split_html(tex, &files);
  assert_fields_bibitems(&stderr, &html);
}

const UNITS_BIB: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_title_units_events.bib");

/// Each entry of `biblatex_title_units_events.bib`, whole: `@book` (subtitle
/// and title addon), `@inbook` (main subtitle and addon), `@incollection` and
/// `@inbook` (an event, once), `@inproceedings` salam (book subtitle and
/// addon, and the proceedings' event).
// Known layout residuals pinned here, not intended (DIVERGENCES #299): the
// book-subtitle row's `,` before the place row's `(` ("Symposium, (Aspenäsgarden";
// biblatex prints "Symposium (Aspenäsgarden"), and the event date running into
// the editor with no punctuation ("1968-05-19/1968-05-25 Nils Svartholm (Ed.)";
// the double space there is Perl's row layout, MakeBibliography.pm:734-735).
const UNITS_BIBITEMS: [(&str, &str); 5] = [
  (
    "bib.bib2",
    r#"<li id="bib.bib2" class="ltx_bibitem ltx_bib_book"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[1]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Alice Adams</span><span class="ltx_text ltx_bib_year"> (2001)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Alpha Title</span>. <span class="ltx_text ltx_bib_subtitle">Bravo Subtitle? Charlie Addon</span>.
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub</span>.
</span></li>"#,
  ),
  (
    "bib.bib3",
    r#"<li id="bib.bib3" class="ltx_bibitem ltx_bib_inbook"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[2]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Bob Baker</span><span class="ltx_text ltx_bib_year"> (2002)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Delta Chapter</span>.
</span>
<span class="ltx_bibblock">In <span class="ltx_text ltx_bib_maintitle">Foxtrot Works</span>. <span class="ltx_text ltx_bib_subtitle">Golf Subtitle. Hotel Addon</span>. <span class="ltx_text ltx_bib_inbook">Echo Volume</span>,
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub</span>.
</span></li>"#,
  ),
  (
    "bib.bib4",
    r#"<li id="bib.bib4" class="ltx_bibitem ltx_bib_incollection"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[3]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Carl Clark</span><span class="ltx_text ltx_bib_year"> (2003)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">India Paper</span>.
</span>
<span class="ltx_bibblock">In <span class="ltx_text ltx_bib_inbook">Juliet Collection</span>, <span class="ltx_text ltx_bib_event">Kilo Workshop</span> (<span class="ltx_text ltx_bib_place">Lima Town</span>), <span class="ltx_text ltx_bib_date">2003-04-05</span>
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub</span>.
</span></li>"#,
  ),
  (
    "bib.bib5",
    r#"<li id="bib.bib5" class="ltx_bibitem ltx_bib_inbook"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[4]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Dora Dunn</span><span class="ltx_text ltx_bib_year"> (2004)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Mike Chapter</span>.
</span>
<span class="ltx_bibblock">In <span class="ltx_text ltx_bib_inbook">November Book</span>, <span class="ltx_text ltx_bib_event">Oscar Meeting</span> (<span class="ltx_text ltx_bib_place">Papa City</span>), <span class="ltx_text ltx_bib_date">2004-06-01/2004-06-03</span>
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub</span>.
</span></li>"#,
  ),
  (
    "bib.bib1",
    r#"<li id="bib.bib1" class="ltx_bibitem ltx_bib_inproceedings"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[5]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Abdus Salam</span><span class="ltx_text ltx_bib_year"> (1968)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Weak and Electromagnetic Interactions</span>.
</span>
<span class="ltx_bibblock">In <span class="ltx_text ltx_bib_inbook">Elementary particle theory</span>, <span class="ltx_text ltx_bib_subtitle">Relativistic groups and analyticity. Proceedings of the Eighth Nobel Symposium</span>, (<span class="ltx_text ltx_bib_place">Aspenäsgarden, Lerum</span>), <span class="ltx_text ltx_bib_date">1968-05-19/1968-05-25</span>  <span class="ltx_text ltx_bib_editor">Nils Svartholm (Ed.)</span>,
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Almquist &amp; Wiksell</span>, <span class="ltx_text ltx_bib_place">Stockholm</span>, <span class="ltx_text ltx_bib_pages">pp. 367–377</span>.
</span></li>"#,
  ),
];

fn assert_units_bibitems(stderr: &str, html: &str) {
  assert_eq!(error_count(stderr), 0, "{stderr}");
  assert_eq!(warning_count(stderr), 0, "{stderr}");
  assert_eq!(html.matches("class=\"ltx_bibitem ").count(), 5, "{html}");
  for (id, bibitem) in UNITS_BIBITEMS {
    let selector = format!(r#"id="{id}""#);
    latexml::util::test::assert_element(html, "li", &[selector.as_str()], bibitem);
  }
}

/// A title's subtitle and addon are two units, "Subtitle. Addon"
/// (biblatex.def:3159-3199: the subtitle closes the title unit, `\newunit`,
/// then the addon), for the title, the book title and the main title alike;
/// they had run together ("Relativistic groups and analyticityProceedings of
/// the Eighth Nobel Symposium", biblatex-examples.bib `salam`). The event of
/// an `@incollection`/`@inbook` nests in its book host and prints once; it
/// had printed as the book's title and place and then again.
#[test]
fn biblatex_title_units_and_events_print_once() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_title_units_events.tex");
  let files = [("biblatex_title_units_events.bib", UNITS_BIB)];
  let (stderr, html) = convert_html(tex, &files);
  assert_units_bibitems(&stderr, &html);
  let (stderr, html) = convert_split_html(tex, &files);
  assert_units_bibitems(&stderr, &html);
}

const CLASSIC_BIB: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_host_rows_classic.bib");

/// CONTROL for [`biblatex_second_tier_fields_are_printed`]: classic entries
/// render exactly as before P3 split the incollection "In" row and narrowed the
/// article journal row to the journal host (Perl's FMT_SPEC rows,
/// MakeBibliography.pm:688-744); the tip binary gives the same three bibitems.
#[test]
fn classic_host_rows_are_unchanged() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_host_rows_classic.tex");
  let (stderr, html) = convert_html(tex, &[("bib_host_rows_classic.bib", CLASSIC_BIB)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(html.matches("class=\"ltx_bibitem ").count(), 3, "{html}");
  for (id, bibitem) in [
    (
      "bib.bib1",
      r#"<li id="bib.bib1" class="ltx_bibitem ltx_bib_incollection"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[1]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">A. Able</span><span class="ltx_text ltx_bib_year"> (2001)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Part title</span>.
</span>
<span class="ltx_bibblock">In <span class="ltx_text ltx_bib_inbook">Host Book</span>,  <span class="ltx_text ltx_bib_editor">B. Baker (Ed.)</span>,
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub</span>, <span class="ltx_text ltx_bib_place">Town</span>, <span class="ltx_text ltx_bib_pages">pp. 1–10</span>.
</span></li>"#,
    ),
    (
      "bib.bib2",
      r#"<li id="bib.bib2" class="ltx_bibitem ltx_bib_article"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[2]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">C. Cole</span><span class="ltx_text ltx_bib_year"> (2002)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Paper title</span>.
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_journal">Some Journal</span> <span class="ltx_text ltx_bib_volume">12</span> (<span class="ltx_text ltx_bib_number">3</span>), <span class="ltx_text ltx_bib_pages">pp. 5–9</span>.
</span></li>"#,
    ),
    (
      "bib.bib3",
      r#"<li id="bib.bib3" class="ltx_bibitem ltx_bib_inproceedings"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[3]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">D. Dunn</span><span class="ltx_text ltx_bib_year"> (2003)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Talk</span>.
</span>
<span class="ltx_bibblock">In <span class="ltx_text ltx_bib_inbook">Proc Book</span>,
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_place">Venue City</span>.
</span></li>"#,
    ),
  ] {
    let selector = format!(r#"id="{id}""#);
    latexml::util::test::assert_element(&html, "li", &[selector.as_str()], bibitem);
  }
}

const QUESTION_BIB: &str =
  include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_title_question_period.bib");

/// The two entries of `bib_title_question_period.bib`, whole.
const QUESTION_BIBITEMS: [(&str, &str); 2] = [
  (
    "bib.bib1",
    r#"<li id="bib.bib1" class="ltx_bibitem ltx_bib_book"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[1]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Alice Adams</span><span class="ltx_text ltx_bib_year"> (2001)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">What is X?</span> <span class="ltx_text ltx_bib_subtitle">A survey</span>.
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub</span>.
</span></li>"#,
  ),
  (
    "bib.bib2",
    r#"<li id="bib.bib2" class="ltx_bibitem ltx_bib_book"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[2]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Bob Brown</span><span class="ltx_text ltx_bib_year"> (2002)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Is it so?</span>
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub</span>.
</span></li>"#,
  ),
];

fn assert_question_bibitems(stderr: &str, html: &str) {
  assert_eq!(error_count(stderr), 0, "{stderr}");
  assert_eq!(warning_count(stderr), 0, "{stderr}");
  assert_eq!(html.matches("class=\"ltx_bibitem ").count(), 2, "{html}");
  for (id, bibitem) in QUESTION_BIBITEMS {
    let selector = format!(r#"id="{id}""#);
    latexml::util::test::assert_element(html, "li", &[selector.as_str()], bibitem);
  }
}

/// A title that already ends in a mark takes no added period under biblatex:
/// its punctuation tracker (biblatex.sty:2118-2130 `\blx@addpunct`, :2026
/// `\DeclarePunctuationPairs{period}{}`) drops the period after any mark, so
/// pdflatex + biber print "What is X? A survey" and "Is it so? Pub". Perl's
/// rows add their "." unconditionally (MakeBibliography.pm:527-531) and print
/// "What is X?. A survey".
#[test]
fn title_mark_takes_no_period_biblatex() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/bib_title_question_period.tex");
  let files = [("bib_title_question_period.bib", QUESTION_BIB)];
  let (stderr, html) = convert_html(tex, &files);
  assert_question_bibitems(&stderr, &html);
  let (stderr, html) = convert_split_html(tex, &files);
  assert_question_bibitems(&stderr, &html);
}

/// The same under a `.bst`, by BibTeX's own rule: `add.period$` adds "."
/// only "if the last non-`}` character isn't a `.`, `?`, or `!`"
/// (btxhak.tex:326-329). pdflatex + bibtex (plain.bst) print "What is this?
/// Pub, 2001.", "Is it so? Journal of Y, …" and "Stop! Pub, Inc., 2003." —
/// the last one also the ".." of a publisher ending in an abbreviation.
#[test]
fn title_mark_takes_no_period_bst() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/bib_title_question_period_plain.tex"
  );
  let bib = include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/bib_title_question_period_plain.bib"
  );
  let (stderr, html) = convert_html(tex, &[("bib_title_question_period_plain.bib", bib)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(html.matches("class=\"ltx_bibitem ").count(), 3, "{html}");
  for (id, bibitem) in [
    (
      "bib.bib1",
      r#"<li id="bib.bib1" class="ltx_bibitem ltx_bib_book"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[1]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Alice Adams</span><span class="ltx_text ltx_bib_year"> (2001)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">What is this?</span>
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub</span>.
</span></li>"#,
    ),
    (
      "bib.bib2",
      r#"<li id="bib.bib2" class="ltx_bibitem ltx_bib_article"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[2]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Bob Brown</span><span class="ltx_text ltx_bib_year"> (2002)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Is it so?</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_journal">Journal of Y</span> <span class="ltx_text ltx_bib_volume">3</span>, <span class="ltx_text ltx_bib_pages">pp. 1–10</span>.
</span></li>"#,
    ),
    (
      "bib.bib3",
      r#"<li id="bib.bib3" class="ltx_bibitem ltx_bib_book"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[3]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Carl Cole</span><span class="ltx_text ltx_bib_year"> (2003)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Stop!</span>
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub, Inc.</span>
</span></li>"#,
    ),
  ] {
    let selector = format!(r#"id="{id}""#);
    latexml::util::test::assert_element(&html, "li", &[selector.as_str()], bibitem);
  }
}

/// A shipped biblatex `.bbl` (biber's output, the arXiv submission shape)
/// prints what the `.bib` it came from prints: its records are read back into
/// the BibTeX entry and formatted alike (biblatex_sty.rs `bbl_flush`), so the
/// `.bbl` of `bib_title_question_period.bib` gives the same two bibitems,
/// `subtitle` included. The ar5iv binding's `\bibitem` rebuild
/// (biblatex.sty.ltxml:495-690) dropped the subtitle: "Alice Adams “What is
/// X?” Pub, 2001". pdflatex, reading the same `.bbl`: "Alice Adams. What is X?
/// A survey. Pub, 2001."
#[test]
fn bbl_prints_what_its_bib_prints() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_bbl_subtitle.tex");
  let bbl =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_bbl_subtitle.bbl");
  let files = [("t.bbl", bbl)];
  let (stderr, html) = convert_html(tex, &files);
  assert_question_bibitems(&stderr, &html);
  let (stderr, html) = convert_split_html(tex, &files);
  assert_question_bibitems(&stderr, &html);
}

/// The alphabetic labels of a `.bbl` are biber's, in the `.bbl`'s order, and
/// the citations print them: `labelalpha` + `extraalpha` as alphabetic.bbx:22-25
/// prints them (`\mknumalph`, biblatex.def:476), "Knu84a"/"Knu84b", a label
/// holding TeX typeset ("Böh66" from `B\"{o}h66`) — pdflatex prints "See
/// [Knu84b] and [Ada+01] and [Knu84a] and [Böh66]." with the References in
/// label order. Witnesses arXiv 2605.10053, 2605.14864, 2605.18215,
/// 2605.21199, 1212.4446.
#[test]
fn bbl_keeps_bibers_alphabetic_labels_and_order() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_bbl_alphabetic.tex");
  let bbl =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_bbl_alphabetic.bbl");
  let (stderr, html) = convert_html(tex, &[("t.bbl", bbl)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  assert_eq!(html.matches("class=\"ltx_bibitem ").count(), 4, "{html}");
  latexml::util::test::assert_element(
    &html,
    "p",
    &[r#"class="ltx_p""#],
    r##"<p class="ltx_p">See <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib4" title="Seminumerical Algorithms" class="ltx_ref">Knu84b</a>]</cite> and <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib1" title="On Many Authors" class="ltx_ref">Ada+01</a>]</cite> and <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib3" title="Fundamental Algorithms" class="ltx_ref">Knu84a</a>]</cite> and <cite class="ltx_cite ltx_citemacro_cite">[<a href="#bib.bib2" title="Flow Diagrams" class="ltx_ref">Böh66</a>]</cite>.</p>"##,
  );
  let cited_by = r##"<span class="ltx_bibblock ltx_bib_cited">Cited by: <a href="#p1" title="" class="ltx_ref">p1</a>.
</span>"##;
  for (id, bibitem) in [
    (
      "bib.bib1",
      format!(
        r#"<li id="bib.bib1" class="ltx_bibitem ltx_bib_article"><span class="ltx_tag ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem">[Ada+01]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Alice Adams, Bob Brown, Carl Cole, and Dora Dunn</span><span class="ltx_text ltx_bib_year"> (2001)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">On Many Authors</span>.
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_journal">Journal of Y</span>.
</span>
{cited_by}</li>"#
      ),
    ),
    (
      "bib.bib2",
      format!(
        r#"<li id="bib.bib2" class="ltx_bibitem ltx_bib_book"><span class="ltx_tag ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem">[Böh66]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Corrado Böhm</span><span class="ltx_text ltx_bib_year"> (1966)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Flow Diagrams</span>.
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Pub</span>.
</span>
{cited_by}</li>"#
      ),
    ),
    (
      "bib.bib3",
      format!(
        r#"<li id="bib.bib3" class="ltx_bibitem ltx_bib_book"><span class="ltx_tag ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem">[Knu84a]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Donald E. Knuth</span><span class="ltx_text ltx_bib_year"> (1984)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Fundamental Algorithms</span>.
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Addison-Wesley</span>.
</span>
{cited_by}</li>"#
      ),
    ),
    (
      "bib.bib4",
      format!(
        r#"<li id="bib.bib4" class="ltx_bibitem ltx_bib_book"><span class="ltx_tag ltx_bib_abbrv ltx_role_refnum ltx_tag_bibitem">[Knu84b]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Donald E. Knuth</span><span class="ltx_text ltx_bib_year"> (1984)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Seminumerical Algorithms</span>.
</span>
<span class="ltx_bibblock"> <span class="ltx_text ltx_bib_publisher">Addison-Wesley</span>.
</span>
{cited_by}</li>"#
      ),
    ),
  ] {
    let selector = format!(r#"id="{id}""#);
    latexml::util::test::assert_element(&html, "li", &[selector.as_str()], &bibitem);
  }
}

/// Of a `.bbl`'s datalists, `\printbibliography` prints the default
/// refcontext's (`nyt/global//global/global/global`) once — not every list
/// biber wrote (biblatex-apa's `nyt/apasortcite//…` sorts citations); here the
/// other list comes first, reversed. It printed both, as two bibliographies of
/// four bibitems. Witness arXiv 2605.17646.
#[test]
fn bbl_prints_the_default_refcontext_datalist() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_bbl_two_datalists.tex");
  let bbl =
    include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_bbl_two_datalists.bbl");
  let (stderr, html) = convert_html(tex, &[("t.bbl", bbl)]);
  assert_eq!(
    html.matches(r#"class="ltx_bibliography"#).count(),
    1,
    "{html}"
  );
  assert_question_bibitems(&stderr, &html);
}

/// A `.bbl`'s `[list]` datalist is a biblist (the shorthands
/// `\printshorthands` prints), not the bibliography, even when it comes first
/// and its name reads like the default refcontext's; an entry whose options
/// say `skipbib` (or `dataonly`: biber's related-entry clones) is left out
/// (biblatex.sty:8712-8723).
#[test]
fn bbl_skips_its_biblists_and_skipbib_entries() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_bbl_biblist.tex");
  let bbl = include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_bbl_biblist.bbl");
  let (stderr, html) = convert_html(tex, &[("t.bbl", bbl)]);
  assert!(!html.contains("Skipped title"), "{html}");
  assert_question_bibitems(&stderr, &html);
}

/// A format-2 `.bbl` (`\sortlist`, names as positional parts) reads as its
/// `.bib` does: `van der Berg, Pieter and King, Jr., Martin Luther and
/// others`, the date range rejoined from its parts, the pages range list and
/// the `\verb` doi.
#[test]
fn bbl_format2_reads_as_its_bib() {
  let tex = include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_bbl_format2.tex");
  let bbl = include_str!("../../../tools/perfect_kernel/repros/index-bib/biblatex_bbl_format2.bbl");
  let (stderr, xml) = convert_files(tex, &[("t.bbl", bbl)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  let compact: String = xml.split('\n').map(str::trim).collect();
  for element in [
    "<bib-name role=\"author\"><surname>van der Berg</surname><givenname>Pieter</givenname></bib-name>",
    "<bib-name role=\"author\"><surname>King</surname><givenname>Martin Luther</givenname><lineage>Jr.</lineage></bib-name>",
    "<bib-name role=\"author\"><surname>others</surname></bib-name>",
    "<bib-part role=\"pages\">1–10, 15</bib-part>",
    "<bib-identifier href=\"https://dx.doi.org/10.1000/old%5F1\" id=\"10.1000/old_1\" scheme=\"doi\">Document</bib-identifier>",
    "<bib-date role=\"publication\">1999-05-06/1999-05-08</bib-date>",
  ] {
    assert!(compact.contains(element), "{element}\n{xml}");
  }
  let (stderr, html) = convert_html(tex, &[("t.bbl", bbl)]);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(warning_count(&stderr), 0, "{stderr}");
  latexml::util::test::assert_element(
    &html,
    "li",
    &[r#"id="bib.bib1""#],
    r##"<li id="bib.bib1" class="ltx_bibitem ltx_bib_article"><span class="ltx_tag ltx_bib_key ltx_role_refnum ltx_tag_bibitem">[1]</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_author">Pieter van der Berg, Martin Luther King, <span class="ltx_text ltx_bib_etal">et al.</span></span><span class="ltx_text ltx_bib_year"> (1999)</span>
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_title">Old Format</span>.
</span>
<span class="ltx_bibblock"><span class="ltx_text ltx_bib_journal">J. Old</span>, <span class="ltx_text ltx_bib_pages">pp. 1–10, 15</span>.
</span>
<span class="ltx_bibblock">External Links: <span class="ltx_text ltx_bib_links"><a href="https://dx.doi.org/10.1000/old%5F1" title="" class="ltx_ref doi ltx_bib_external">Document</a></span>
</span>
<span class="ltx_bibblock ltx_bib_cited">Cited by: <a href="#p1" title="" class="ltx_ref">p1</a>.
</span></li>"##,
  );
}

/// Two bare `\thebibliography…\endthebibliography` pairs (no environment, so
/// no group) must neither re-arm the pseudo-`\bibitem` rescue on its own
/// redirection — an unconditional `\let` loop, `Fatal:Timeout:TokenLimit`
/// (KNOWN_PERL_ERRORS #57; `setup_pseudo_bibitem`'s re-arm check,
/// latex_constructs/mod.rs) — nor leave it armed after `\endthebibliography`
/// (its disarm, sect11.rs), where the blank line's `\par` would deposit a stray
/// bibitem outside the list. A biblatex `.bbl` expanded to this shape until
/// round 12 (W6-B), which was this guard's only fixture
/// (`06_cluster_bibliography::cluster_biblatex_two_datalists`); witness arXiv
/// 2605.17646.
#[test]
fn bare_thebibliography_twice_arms_once() {
  let tex = "\\documentclass{article}\n\\begin{document}\nSee \\cite{a} and \\cite{b}.\n\
             \\thebibliography{9}\\bibitem{a}Alpha.\\endthebibliography\n\
             \\thebibliography{9}\\bibitem{b}Beta.\\endthebibliography\n\nAfter.\n\\end{document}\n";
  let (stderr, xml) = super::perfect_kernel_batch46::convert(tex, false);
  assert_eq!(error_count(&stderr), 0, "{stderr}");
  assert_eq!(xml.matches("<bibliography").count(), 2, "{xml}");
  assert_eq!(xml.matches("<bibitem ").count(), 2, "{xml}");
  latexml::util::test::assert_element(
    &xml,
    "para",
    &[],
    r#"<para xml:id="p1"><p>See <cite class="ltx_citemacro_cite">[<bibref bibrefs="a" separator="," yyseparator=","/>]</cite> and <cite class="ltx_citemacro_cite">[<bibref bibrefs="b" separator="," yyseparator=","/>]</cite>.</p></para>"#,
  );
  assert!(xml.contains("<p>After.</p>"), "{xml}");
}
