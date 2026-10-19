//! Red/green guard for batch 77m (silent-loss row #16): a tcolorbox listing box keeps its title, and tcolorbox's
//! `nolol` read-back of a box body is no list-of-listings float. The guards assert the three measures: no Error or
//! Warning, the authored text (title and code) present, and its element.
use latexml::util::test::{assert_element, convert_with};

use super::{
  perfect_kernel_batch46::{error_count, warning_count},
  perfect_kernel_batch57::{RAW, assert_elements},
};

/// `\newtcblisting[auto counter]{…}[1]{title={Listing \thetcbcounter: #1},…}`: the box prints its title above the
/// code (pdflatex: "Listing 1: Numbered One" over "y = 2"); the binding's `\lstnewenvironment` delegation dropped
/// every box option but the counter, the mode and the listing file. The title is the listing's title, at the top.
/// Witnesses 2405.01576, 2409.11413. Repro listings-algorithms/newtcblisting_keeps_its_counted_title.
#[test]
fn newtcblisting_keeps_its_counted_title() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/listings-algorithms/newtcblisting_keeps_its_counted_title.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "float",
      "LSTx1",
      r##"<float class="ltx_lstlisting" xml:id="LSTx1"><toccaption>Listing 1: Numbered One</toccaption><caption>Listing 1: Numbered One</caption><listing class="ltx_lstlisting" data="eSA9IDI=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">y</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>2</listingline></listing></float>"##,
    ),
    ("para", "p1", r##"<para xml:id="p1"><p>After.</p></para>"##),
  ]);
}

/// The `\NewTCBListing` family keeps the title too, with its optional box options applied (2508.16690,
/// 2405.07395). Repro listings-algorithms/new_tcb_listing_keeps_its_title.
#[test]
fn new_tcb_listing_keeps_its_title() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/listings-algorithms/new_tcb_listing_keeps_its_title.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "float",
      "LSTx1",
      r##"<float class="ltx_lstlisting" xml:id="LSTx1"><toccaption>Example 1: First</toccaption><caption>Example 1: First</caption><listing class="ltx_lstlisting" data="YSA9IDE=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">a</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>1</listingline></listing></float>"##,
    ),
    (
      "float",
      "LSTx2",
      r##"<float class="ltx_lstlisting" xml:id="LSTx2"><toccaption>Example 2: Second</toccaption><caption>Example 2: Second</caption><listing class="ltx_lstlisting" data="YiA9IDI=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx2"><text class="ltx_lst_identifier">b</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>2</listingline></listing></float>"##,
    ),
  ]);
}

/// `\newtcbinputlisting[use counter=example]{\examplefile}[3][]{title={Example~\thetcbcounter: #2},listing
/// file={#3},…}` shares the environment's counter through tcolorbox's init processor and keeps its title; the file
/// is read back `nolol`, so it is no `lol` float captioned with its name (1912.06298, 2105.06630, 2301.07825,
/// 2302.14514, 2410.04867, 2405.01480). Repro listings-algorithms/newtcbinputlisting_keeps_its_title.
#[test]
fn newtcbinputlisting_keeps_its_title() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/listings-algorithms/newtcbinputlisting_keeps_its_title.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[
    (
      "float",
      "LSTx1",
      r##"<float class="ltx_lstlisting" xml:id="LSTx1"><toccaption>Example 1: Inline</toccaption><caption>Example 1: Inline</caption><listing class="ltx_lstlisting" data="eCA9IDE=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">x</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>1</listingline></listing></float>"##,
    ),
    (
      "float",
      "LSTx2",
      r##"<float class="ltx_lstlisting" xml:id="LSTx2"><toccaption>Example 2: From file</toccaption><caption>Example 2: From file</caption><listing class="ltx_lstlisting" data="eiA9IDM=" dataencoding="base64" datamimetype="text/plain" dataname="tcbexample.txt"><listingline xml:id="lstnumberx2"><text class="ltx_lst_identifier">z</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>3</listingline></listing></float>"##,
    ),
  ]);
  assert!(!xml.contains("inlist=\"lol\""), "{xml}");
  assert!(!xml.contains("tcbexample.txt</toccaption>"), "{xml}");
}

/// A raw `{tcblisting}` box reads its body back with `\lstinputlisting[nolol,…]{\jobname.listing}`
/// (tcblistings.code.tex:39/:45): the list of listings stays empty and the job's file name is no caption (it was
/// a `<float inlist="lol">` with `<toccaption>` "<job>.listing" inside the box). The box keeps its title and code.
/// Witnesses 2512.18601, 2606.05464, 2606.27251, 2607.12625. Repro
/// listings-algorithms/raw_tcblisting_read_back_is_not_listed.
#[test]
fn raw_tcblisting_read_back_is_not_listed() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/listings-algorithms/raw_tcblisting_read_back_is_not_listed.tex"
  );
  let (log, xml) = convert_with(tex, Some(RAW));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // the box's title part, then its upper part holding the listing
  assert_element(
    &xml,
    "inline-block",
    &["class=\"ltx_minipage\""],
    r##"<inline-block class="ltx_minipage" vattach="bottom" width="31.37em"><p><text color="#FFFFFF">Plain Title</text></p></inline-block>"##,
  );
  assert_element(
    &xml,
    "listing",
    &["dataname=\"t.listing\""],
    r##"<listing class="ltx_lst_language_TeX_LaTeX ltx_lstlisting" data="eCA9IDE=" dataencoding="base64" datamimetype="text/plain" dataname="t.listing"><listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier" color="#000000" font="typewriter" fontsize="90%">x</text><text class="ltx_lst_space" color="#000000" font="typewriter" fontsize="90%"> </text><text color="#000000" font="typewriter" fontsize="90%">=</text><text class="ltx_lst_space" color="#000000" font="typewriter" fontsize="90%"> </text><text color="#000000" font="typewriter" fontsize="90%">1</text></listingline></listing>"##,
  );
  assert!(!xml.contains("<float"), "{xml}");
  assert!(!xml.contains(".listing</toccaption>"), "{xml}");
}

/// A plain `\lstnewenvironment` after a tcolorbox listing box (default `listing and text`) shows its body as code
/// only: the box's execute flag is the box's own and no longer outlives it (the plain body was typeset a second
/// time as a paragraph). Repro listings-algorithms/plain_lstnewenvironment_after_tcb_box_does_not_run_its_body.
#[test]
fn plain_lstnewenvironment_after_a_tcb_box_does_not_run_its_body() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/listings-algorithms/plain_lstnewenvironment_after_tcb_box_does_not_run_its_body.tex"
  );
  let (log, xml) = convert_with(tex, Some(RAW));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  // the box's body is displayed and run (pdflatex prints "AAA" twice), the plain one only displayed
  assert!(xml.contains("<p>AAA</p>"), "{xml}");
  assert!(!xml.contains("<p>BBB</p>"), "{xml}");
  assert_eq!(xml.matches(">BBB</text>").count(), 1, "{xml}");
}

/// An absent `!O{}` is its empty default to the options, as xparse passes it: `\ifx\relax#3\relax` takes the
/// no-title branch (neoschool.cls:5168-5214, its 40 `{code}` boxes); a given one titles its box. Repro
/// listings-algorithms/new_tcb_listing_empty_optional_is_empty.
#[test]
fn new_tcb_listing_empty_optional_is_empty() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/listings-algorithms/new_tcb_listing_empty_optional_is_empty.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><listing class="ltx_lstlisting" data="eCA9IDE=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">x</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>1</listingline></listing></para>"##,
  )]);
  assert_eq!(xml.matches("<caption>").count(), 1, "{xml}");
  assert!(
    xml.contains(concat!(
      "<caption>Listing",
      "\u{a0}",
      "2",
      "\u{a0}",
      "–",
      "\u{a0}",
      "Square Function</caption>"
    )),
    "{xml}"
  );
}

/// A listing box without a counter has an empty `\thetcbcounter` (tcolorbox's `no counter`), so a title naming it
/// prints "code : …" as pdflatex does — not an undefined-macro Error (2605.21082). Repro
/// listings-algorithms/newtcblisting_without_a_counter_has_an_empty_counter.
#[test]
fn newtcblisting_without_a_counter_has_an_empty_counter() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/listings-algorithms/newtcblisting_without_a_counter_has_an_empty_counter.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "float",
    "LSTx1",
    concat!(
      r##"<float class="ltx_lstlisting" xml:id="LSTx1"><toccaption><text font="bold">code"##,
      "\u{a0}",
      r##"</text>: Initial code</toccaption><caption><text font="bold">code"##,
      "\u{a0}",
      r##"</text>: Initial code</caption><listing class="ltx_lstlisting" data="eCA9IDE=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">x</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>1</listingline></listing></float>"##
    ),
  )]);
}

/// listings' `nolol` (listings.sty:1409-1411, `\lst@MakeCaption` :1644-1656) keeps a captioned listing numbered but
/// out of the list of listings; a listing without it is listed (Perl ignores the key, KNOWN_PERL_ERRORS #988).
#[test]
fn nolol_listing_is_numbered_but_not_listed() {
  let (log, xml) = convert_with(
    "\\documentclass{article}\\usepackage{listings}\\begin{document}\n\\begin{lstlisting}[nolol,caption=Bar]\ny\n\\end{\
     lstlisting}\n\\begin{lstlisting}[caption=Baz]\nz\n\\end{lstlisting}\n\\end{document}",
    Some(RAW),
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_element(
    &xml,
    "float",
    &["xml:id=\"LST1\""],
    concat!(
      r##"<float class="ltx_lstlisting" xml:id="LST1"><tags><tag>Listing"##,
      "\u{a0}",
      r##"1</tag><tag role="refnum">1</tag><tag role="typerefnum">Listing 1</tag></tags><toccaption><tag close=" ">1</tag>Bar</toccaption><caption><tag close=": ">Listing"##,
      "\u{a0}",
      r##"1</tag>Bar</caption><listing class="ltx_lstlisting" data="eQ==" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">y</text></listingline></listing></float>"##
    ),
  );
  assert!(
    xml.contains(r#"<float class="ltx_lstlisting" inlist="lol" xml:id="LST2">"#),
    "{xml}"
  );
}

/// A tcolorbox box removes listings' own caption (`listing remove caption`, on by default: `\lst@MakeCaption`
/// gobbled), so a document-wide `\lstset{title=\lstname}` titles neither the environment's listing (its box title
/// stands) nor a raw box's read-back ("<job>.listing"); 2409.11413. Repro
/// listings-algorithms/tcb_box_title_replaces_the_listings_title.
#[test]
fn tcb_box_title_replaces_the_listings_title() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/listings-algorithms/tcb_box_title_replaces_the_listings_title.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[(
    "float",
    "LSTx1",
    concat!(
      r##"<float class="ltx_lstlisting" xml:id="LSTx1"><toccaption>Listing"##,
      "\u{a0}",
      r##"1:"##,
      "\u{a0}",
      r##"Setup</toccaption><caption>Listing"##,
      "\u{a0}",
      r##"1:"##,
      "\u{a0}",
      r##"Setup</caption><listing class="ltx_lstlisting" data="eCA9IDE=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">x</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>1</listingline></listing></float>"##
    ),
  )]);
  assert_eq!(xml.matches("<caption").count(), 1, "{xml}");
  assert_eq!(xml.matches("<float").count(), 1, "{xml}");
}

/// The box's title bar is above its code whatever listings' `captionpos` says (tcolorbox.sty:162 `\tcbtitle`; the box
/// gobbles `\lst@MakeCaption`), while a plain listing's own `title=` still follows `captionpos=b`. 2409.11413
/// (setup.tex:86, 13 boxes), 2402.13846 (2508.16690, 2301.07825: `captionpos=b` plain listings only). Repro listings-algorithms/tcb_box_title_stays_above_under_captionpos_b.
#[test]
fn tcb_box_title_stays_above_under_captionpos_b() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/listings-algorithms/tcb_box_title_stays_above_under_captionpos_b.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "float",
      "LSTx1",
      r##"<float class="ltx_lstlisting" xml:id="LSTx1"><toccaption>Box: Top</toccaption><caption>Box: Top</caption><listing class="ltx_lstlisting" data="YSA9IDE=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">a</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>1</listingline></listing></float>"##,
    ),
    (
      "float",
      "LSTx2",
      r##"<float class="ltx_lstlisting" xml:id="LSTx2"><listing class="ltx_lstlisting" data="YiA9IDI=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx2"><text class="ltx_lst_identifier">b</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>2</listingline></listing><toccaption>Own title</toccaption><caption>Own title</caption></float>"##,
    ),
  ]);
}

/// A leading `o` of a `\NewTCBListing` signature is xparse's no-value marker when absent, so `\IfNoValueTF{#1}` takes
/// its no-value branch (pdflatex "Code", "Code: given"; the arity's empty default printed "Code: "). Repro
/// listings-algorithms/new_tcb_listing_leading_o_absent_is_no_value.
#[test]
fn new_tcb_listing_leading_o_absent_is_no_value() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/listings-algorithms/new_tcb_listing_leading_o_absent_is_no_value.tex"
  );
  let xml = assert_elements(tex, RAW, (0, 0), &[
    (
      "float",
      "LSTx1",
      r##"<float class="ltx_lstlisting" xml:id="LSTx1"><toccaption>Code</toccaption><caption>Code</caption><listing class="ltx_lstlisting" data="YSA9IDE=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">a</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>1</listingline></listing></float>"##,
    ),
    (
      "float",
      "LSTx2",
      r##"<float class="ltx_lstlisting" xml:id="LSTx2"><toccaption>Code: given</toccaption><caption>Code: given</caption><listing class="ltx_lstlisting" data="YiA9IDI=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx2"><text class="ltx_lst_identifier">b</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>2</listingline></listing></float>"##,
    ),
  ]);
  assert!(!xml.contains("NoValue"), "{xml}");
}

/// A leading `O{Default}` / `D[]{Python}` keeps its default when absent (pdflatex "T: Default", "Name (Python)"; the
/// arity's empty default printed "T:" and "Name ()"). Repro
/// listings-algorithms/new_tcb_listing_leading_optional_keeps_its_default.
#[test]
fn new_tcb_listing_leading_optional_keeps_its_default() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/listings-algorithms/new_tcb_listing_leading_optional_keeps_its_default.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[
    (
      "float",
      "LSTx1",
      r##"<float class="ltx_lstlisting" xml:id="LSTx1"><toccaption>T: Given</toccaption><caption>T: Given</caption><listing class="ltx_lstlisting" data="YSA9IDE=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">a</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>1</listingline></listing></float>"##,
    ),
    (
      "float",
      "LSTx2",
      r##"<float class="ltx_lstlisting" xml:id="LSTx2"><toccaption>T: Default</toccaption><caption>T: Default</caption><listing class="ltx_lstlisting" data="YiA9IDI=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx2"><text class="ltx_lst_identifier">b</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>2</listingline></listing></float>"##,
    ),
    (
      "float",
      "LSTx3",
      r##"<float class="ltx_lstlisting" xml:id="LSTx3"><toccaption>Name (Python)</toccaption><caption>Name (Python)</caption><listing class="ltx_lstlisting" data="YyA9IDM=" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx3"><text class="ltx_lst_identifier">c</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>3</listingline></listing></float>"##,
    ),
  ]);
}

/// `\newtcbinputlisting`'s per-use `label=` is the box's phantom code, run at its listing's first line as for a
/// listing environment (it was dropped: a dangling `\ref`). 2405.01480, 2405.02221, 2405.02462, 2405.03285, incgraph-doc
/// (`\examplefile[label={ex:header},…]`; the template also ships, unused by the main file, in 1912.06298, 2105.06630,
/// 2301.07825, 2302.14514, 2410.04867). Repro listings-algorithms/newtcbinputlisting_keeps_its_label.
#[test]
fn newtcbinputlisting_keeps_its_label() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/listings-algorithms/newtcbinputlisting_keeps_its_label.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "float",
    "LSTx1",
    concat!(
      r##"<float class="ltx_lstlisting" xml:id="LSTx1"><toccaption>Example"##,
      "\u{a0}",
      r##"1: Header</toccaption><caption>Example"##,
      "\u{a0}",
      r##"1: Header</caption><listing class="ltx_lstlisting" data="eiA9IDM=" dataencoding="base64" datamimetype="text/plain" dataname="tcblabel.txt"><listingline labels="LABEL:ex:header" xml:id="lstnumberx1"><text class="ltx_lst_identifier">z</text><text class="ltx_lst_space"> </text>=<text class="ltx_lst_space"> </text>3</listingline></listing></float>"##
    ),
  )]);
}

/// `\newtcbinputlisting` follows the box's mode (tcblistingscore.code.tex:195-220): the default `listing and text`
/// typesets the file again after its listing, `text and listing` before it, `text only` alone (after its title),
/// `listing only` not at all (every box was code only, the text part dropped with 0 errors). Same papers as above (`\examplefile`
/// without `listing only`). Repro listings-algorithms/newtcbinputlisting_follows_its_mode.
#[test]
fn newtcbinputlisting_follows_its_mode() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/listings-algorithms/newtcbinputlisting_follows_its_mode.tex"
  );
  let listing = |n: usize, title: &str| {
    format!(
      r##"<float class="ltx_lstlisting" xml:id="LSTx{n}"><toccaption>Mode: {title}</toccaption><caption>Mode: {title}</caption><listing class="ltx_lstlisting" data="XHRleHRiZntUeXBlc2V0fQ==" dataencoding="base64" datamimetype="text/plain" dataname="tcbmode.txt"><listingline xml:id="lstnumberx{n}">\<text class="ltx_lst_identifier">textbf</text>{{<text class="ltx_lst_identifier">Typeset</text>}}</listingline></listing></float>"##
    )
  };
  let xml = assert_elements(tex, RAW, (0, 0), &[
    ("float", "LSTx1", &listing(1, "default")),
    ("float", "LSTx2", &listing(2, "text first")),
    ("float", "LSTx3", &listing(3, "listing alone")),
    // `text only`: the title bar, then the text (no listing to caption)
    (
      "para",
      "p3",
      r##"<para class="ltx_noindent" xml:id="p3"><p>Mode: text alone</p></para>"##,
    ),
  ]);
  // default: listing, text; text first: text, listing; text alone: text; listing alone: listing
  let body = &xml[xml.find("<document").unwrap()..];
  let marks: std::collections::BTreeMap<usize, char> = body
    .match_indices("<text font=\"bold\">Typeset</text>")
    .map(|(at, _)| (at, 'T'))
    .chain(body.match_indices("<float ").map(|(at, _)| (at, 'L')))
    .collect();
  let sequence: String = marks.values().collect();
  assert_eq!(sequence, "LTTLTL", "{xml}");
}

/// A tcolorbox box whose listing is diverted to a file (`\lst@BeginWriteFile`) still runs its text part, and its
/// execute flag is taken before the diverted body returns, so a later plain `\lstnewenvironment` shows its body as
/// code only (pdflatex: "AAA", the "BBB" listing, "After."). Repro
/// listings-algorithms/plain_lstnewenvironment_after_a_diverted_tcb_box_does_not_run_its_body.
#[test]
fn plain_lstnewenvironment_after_a_diverted_tcb_box_does_not_run_its_body() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/listings-algorithms/plain_lstnewenvironment_after_a_diverted_tcb_box_does_not_run_its_body.tex"
  );
  assert_elements(tex, RAW, (0, 0), &[(
    "para",
    "p1",
    r##"<para xml:id="p1"><p>AAA</p><listing class="ltx_lstlisting" data="QkJC" dataencoding="base64" datamimetype="text/plain"><listingline xml:id="lstnumberx1"><text class="ltx_lst_identifier">BBB</text></listingline></listing><p>After.</p></para>"##,
  )]);
}
