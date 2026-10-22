//! Red/green guards for batch 78t (quality batch 17, fidelity audit 4's numbering classes): subfigure's `\subfigure[]`
//! and uncaptioned panels, sub-captions that are not the document's `\caption`, subfigure.cfg, a `\label` in an amsmath
//! `{equation}` with `\nonumber`, a `\label` on an algorithm2e line. Each guard converts its repro under
//! `tools/perfect_kernel/repros/`, asserts the diagnostics by count and the labelled elements whole, their refnums
//! those of pdflatex's `.aux`.
use latexml::util::test::{assert_element, convert_with};

use super::perfect_kernel_batch46::{error_count, warning_count};

const PRELOAD: &str = "ar5iv.sty";

macro_rules! repro {
  ($path:literal) => {
    include_str!(concat!("../../../tools/perfect_kernel/repros/", $path))
  };
}

/// Convert `tex`: 0 errors, 0 warnings.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = convert_with(tex, Some(PRELOAD));
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  xml
}

/// A sub-float's tags: its printed label `(x)` in the sub-caption size, and the refnum.
fn panel_tags(id: &str, letter: &str, refnum: &str) -> String {
  format!(
    "<tags><tag><text fontsize=\"80%\" xml:id=\"{id}.1\">({letter})</text></tag><tag role=\"refnum\">{refnum}</tag></tags>"
  )
}

/// subfigure.sty:208-262: `\subfigure` steps its counter whatever its arguments, an explicit `[]` prints the "(a)"
/// caption, a `\subfigure{…}` none, `[list][caption]` captions with the second (pdflatex "See 1(a), 1(b), 1(c), 1.").
/// Red (int57a): "1, 1, 1(a), 1", no panel number without caption text (KNOWN_PERL_ERRORS #1125). Witnesses 1507.00722,
/// 1808.10020. Repro captions-floats/subfigure_empty_optional_is_a_numbered_panel.
#[test]
fn subfigure_empty_optional_is_a_numbered_panel() {
  let xml = convert_clean(repro!(
    "captions-floats/subfigure_empty_optional_is_a_numbered_panel.tex"
  ));
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1.sf1\""],
    &format!(
      "<figure class=\"ltx_figure_panel\" labels=\"LABEL:a\" xml:id=\"S0.F1.sf1\">{}\
       <rule height=\"28.5pt\" width=\"28.5pt\"/><toccaption><tag close=\" \">(a)</tag></toccaption>\
       <caption><tag close=\" \"><text fontsize=\"80%\" xml:id=\"S0.F1.sf1.2\">(a)</text></tag></caption></figure>",
      panel_tags("S0.F1.sf1", "a", "1(a)")
    ),
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1.sf2\""],
    &format!(
      "<figure class=\"ltx_figure_panel\" labels=\"LABEL:b\" xml:id=\"S0.F1.sf2\">{}\
       <rule height=\"28.5pt\" width=\"28.5pt\"/></figure>",
      panel_tags("S0.F1.sf2", "b", "1(b)")
    ),
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1.sf3\""],
    &format!(
      "<figure class=\"ltx_figure_panel\" labels=\"LABEL:c\" xml:id=\"S0.F1.sf3\">{}\
       <rule height=\"28.5pt\" width=\"28.5pt\"/><toccaption><tag close=\" \">(c)</tag>Third panel</toccaption>\
       <caption><tag close=\" \"><text fontsize=\"80%\" xml:id=\"S0.F1.sf3.2\">(c)</text></tag>\
       <text fontsize=\"80%\" xml:id=\"S0.F1.sf3.3\">Third panel</text></caption></figure>",
      panel_tags("S0.F1.sf3", "c", "1(c)")
    ),
  );
}

/// subfigure.sty:279-290: the sub-caption is `\@make<sub>caption`'s, never the document's `\caption`, so a
/// `\def\caption#1{\splitter #1@@}` does not read it (pdflatex 0 errors, "See 1(a), 1(b), 1."). Red (int57a): 1 Error
/// and a Fatal `Mouth:EoF` (KNOWN_PERL_ERRORS #1126; Perl 55 Errors). Witness 1808.10020 (48 Errors). Repro
/// captions-floats/subfigure_caption_is_not_the_documents_caption.
#[test]
fn subfigure_caption_is_not_the_documents_caption() {
  let xml = convert_clean(repro!(
    "captions-floats/subfigure_caption_is_not_the_documents_caption.tex"
  ));
  for (n, letter, label, text) in [(1, "a", "a", "Left panel."), (2, "b", "b", "Right panel.")] {
    let id = format!("S0.F1.sf{n}");
    assert_element(
      &xml,
      "figure",
      &[&format!("xml:id=\"{id}\"")],
      &format!(
        "<figure class=\"ltx_figure_panel\" labels=\"LABEL:{label}\" xml:id=\"{id}\">{}\
         <rule height=\"28.5pt\" width=\"28.5pt\"/><toccaption><tag close=\" \">({letter})</tag>{text}</toccaption>\
         <caption><tag close=\" \"><text fontsize=\"80%\" xml:id=\"{id}.2\">({letter})</text></tag>\
         <text fontsize=\"80%\" xml:id=\"{id}.3\">{text}</text></caption></figure>",
        panel_tags(&id, letter, &format!("1({letter})"))
      ),
    );
  }
  assert!(
    xml.contains("<caption><tag close=\": \">Figure 1</tag>Main. More.</caption>"),
    "{xml}"
  );
}

/// subfigure.sty:205-208 reads subfigure.cfg, whose `TABTOPCAP` keeps a sub-table after its table's `\caption` from
/// stepping the table counter ahead (pdflatex "See 1, 1(a), 1(b)."), its caption above the tabular. Red (int57a): 2(a),
/// 2(b) (KNOWN_PERL_ERRORS #1127). Repro captions-floats/subtable_after_the_caption_is_numbered_by_it.
#[test]
fn subtable_after_the_caption_is_numbered_by_it() {
  let xml = convert_clean(repro!(
    "captions-floats/subtable_after_the_caption_is_numbered_by_it.tex"
  ));
  for (n, letter, cell, text) in [(1, "a", "x", "Left"), (2, "b", "y", "Right")] {
    let id = format!("S0.T1.st{n}");
    assert_element(
      &xml,
      "table",
      &[&format!("xml:id=\"{id}\"")],
      &format!(
        "<table class=\"ltx_figure_panel\" labels=\"LABEL:t{letter}\" xml:id=\"{id}\">{}\
         <toccaption><tag close=\" \">({letter})</tag>{text}</toccaption>\
         <caption><tag close=\" \"><text fontsize=\"80%\" xml:id=\"{id}.2\">({letter})</text></tag>\
         <text fontsize=\"80%\" xml:id=\"{id}.3\">{text}</text></caption>\
         <tabular vattach=\"middle\" xml:id=\"{id}.4\"><tbody><tr xml:id=\"{id}.4.1\">\
         <td align=\"center\" xml:id=\"{id}.4.1.1\">{cell}</td></tr></tbody></tabular></table>",
        panel_tags(&id, letter, &format!("1({letter})"))
      ),
    );
  }
}

/// subfig.sty:350-411: the sub-caption is `\sf@subcaption`'s (caption3's `\caption@make`), not the document's
/// `\caption`, and `[list][caption]` captions with the second (pdflatex 0 errors, "See 1a, 1b, 1c, 1."). Red (int57a):
/// 1 Error and a Fatal (KNOWN_PERL_ERRORS #1126; the list entry as the caption, #1130). Repro
/// captions-floats/subfig_caption_is_not_the_documents_caption.
#[test]
fn subfig_caption_is_not_the_documents_caption() {
  let xml = convert_clean(repro!(
    "captions-floats/subfig_caption_is_not_the_documents_caption.tex"
  ));
  for (n, letter, list, text) in [
    (1, "a", "Left panel.", "Left panel."),
    (2, "b", "Listed", "Right panel."),
    (3, "c", "", ""),
  ] {
    let id = format!("S0.F1.sf{n}");
    assert_element(
      &xml,
      "figure",
      &[&format!("xml:id=\"{id}\"")],
      &format!(
        "<figure class=\"ltx_figure_panel\" labels=\"LABEL:{letter}\" xml:id=\"{id}\">\
         <tags><tag>({letter})</tag><tag role=\"refnum\">1{letter}</tag></tags>\
         <rule height=\"28.5pt\" width=\"28.5pt\"/><toccaption><tag close=\" \">{letter}</tag>{list}</toccaption>\
         <caption><tag close=\" \">({letter})</tag>{text}</caption></figure>"
      ),
    );
  }
}

/// Every `<equation>` in document order as `labels@xml:id:<tags>…</tags>`, `-` for one without tags.
fn equation_heads(xml: &str) -> String {
  let re = regex::Regex::new(r#"(?s)<equation ([^>]*)>\s*(<tags>.*?</tags>)?"#).unwrap();
  let labels = regex::Regex::new(r#"labels="([^"]*)""#).unwrap();
  let id = regex::Regex::new(r#"xml:id="([^"]*)""#).unwrap();
  let space = regex::Regex::new(r"\s*\n\s*").unwrap();
  re.captures_iter(xml)
    .map(|c| {
      let label = labels
        .captures(&c[1])
        .map_or("", |l| l.get(1).unwrap().as_str());
      let id = id
        .captures(&c[1])
        .map_or("", |i| i.get(1).unwrap().as_str());
      let tags = c.get(2).map_or("-".to_string(), |t| {
        space.replace_all(t.as_str(), "").to_string()
      });
      format!("{label}@{id}:{tags}")
    })
    .collect::<Vec<_>>()
    .join("|")
}

/// amsmath.sty:2924-2932, :1269-1275, :2821-2830: in an amsmath `{equation}` the number is stepped at the start and a
/// `\nonumber` steps the counter back, so its `\label`, made at the end, names the stepped number — the next equation's
/// (pdflatex "See 1, 2, 2, 2, X, 3."); a `\nonumber` after a `\tag` changes nothing. Red (int57a): b and c the section's
/// "1", t unnumbered (KNOWN_PERL_ERRORS #1128). Witness 1306.2116 (`eq:covariate`, PDF 17). Repro
/// cross-references/ams_equation_nonumber_label_names_the_stepped_number.
#[test]
fn ams_equation_nonumber_label_names_the_stepped_number() {
  let xml = convert_clean(repro!(
    "cross-references/ams_equation_nonumber_label_names_the_stepped_number.tex"
  ));
  let tags = |n: &str| format!("<tags><tag>({n})</tag><tag role=\"refnum\">{n}</tag></tags>");
  let reference = |n: &str| format!("<tags><tag role=\"refnum\">{n}</tag></tags>");
  assert_eq!(
    equation_heads(&xml),
    format!(
      "LABEL:a@S1.E1:{}|LABEL:b@S1.Ex1:{}|LABEL:c@S1.Ex2:{}|@S1.Ex3:-|LABEL:d@S1.E2:{}|LABEL:t@S1.Ex4:{}|\
       LABEL:e@S1.E3:{}",
      tags("1"),
      reference("2"),
      reference("2"),
      tags("2"),
      tags("X"),
      tags("3")
    ),
    "{xml}"
  );
}

/// algorithm2e.sty:1657-1667: `\nl` steps `AlgoLine` and makes `\@currentlabel` `\theAlgoLine` (globally), so a
/// `\label` after the line's number names the line — in the line, after its `\;`, at the start of a block (pdflatex
/// "See lines 2, 4, 8 and 9 of 1; 2.": a label at the next algorithm's start names that one). Red (int57a): the
/// algorithm's "1" for all four (KNOWN_PERL_ERRORS #1129).
/// Witnesses 2406.05437 (`step1`, PDF 14), 1602.05866 (`algline:mainloop`, PDF 8). Repro
/// listings-algorithms/algorithm2e_line_label_names_the_line.
#[test]
fn algorithm2e_line_label_names_the_line() {
  let xml = convert_clean(repro!(
    "listings-algorithms/algorithm2e_line_label_names_the_line.tex"
  ));
  assert_element(
    &xml,
    "listingline",
    &["labels=\"LABEL:l2\""],
    "<listingline labels=\"LABEL:l2\" xml:id=\"algorithm1.1\"><tags>\
     <tag><text font=\"bold\" xml:id=\"algorithm1.1.1\">2</text></tag><tag role=\"refnum\">2</tag></tags>\
     Second line ;</listingline>",
  );
  assert_element(
    &xml,
    "listingline",
    &["labels=\"LABEL:l4\""],
    "<listingline labels=\"LABEL:l4\" xml:id=\"algorithm1.2\"><tags>\
     <tag><text font=\"bold\" xml:id=\"algorithm1.2.1\">4</text></tag><tag role=\"refnum\">4</tag></tags>\u{2002}\
     <rule height=\"100%\" width=\"1px\"/>\u{2003}then part;</listingline>",
  );
  assert_element(
    &xml,
    "listingline",
    &["labels=\"LABEL:l8\""],
    "<listingline labels=\"LABEL:l8\" xml:id=\"algorithm1.3\"><tags>\
     <tag><text font=\"bold\" xml:id=\"algorithm1.3.1\">8</text></tag><tag role=\"refnum\">8</tag></tags>\
     Eighth line;</listingline>",
  );
  assert_element(
    &xml,
    "listingline",
    &["labels=\"LABEL:l9\""],
    "<listingline labels=\"LABEL:l9\" xml:id=\"algorithm1.4\"><tags>\
     <tag><text font=\"bold\" xml:id=\"algorithm1.4.1\">9</text></tag><tag role=\"refnum\">9</tag></tags>\
     <text font=\"bold\" xml:id=\"algorithm1.4.2\">while</text> <emph font=\"italic\" xml:id=\"algorithm1.4.3\">cond</emph> \
     <text font=\"bold\" xml:id=\"algorithm1.4.4\">do</text></listingline>",
  );
  // The other lines keep their printed numbers alone, 1 to 11 in order, then the second algorithm's 1.
  let printed: Vec<String> =
    regex::Regex::new(r#"<tag><text font="bold" xml:id="[^"]*">(\d+)</text></tag>"#)
      .unwrap()
      .captures_iter(&xml)
      .map(|c| c[1].to_string())
      .collect();
  assert_eq!(
    printed,
    ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "1"],
    "{xml}"
  );
  // A label at the next algorithm's start names that algorithm (:2560-2561 `\refstepcounter{algocfline}`), not the
  // previous one's last line (2010.00626's `alg:F_cycle`, PDF 2).
  assert!(
    xml.contains(
      "<float class=\"ltx_algorithm\" framed=\"top\" labels=\"LABEL:alg2\" xml:id=\"algorithm2\">"
    ),
    "{xml}"
  );
}

/// subfigure.sty:232-233: a panel is its body's width (`\setbox\@tempboxa\hbox{#4}\@tempdima=\wd\@tempboxa`), the
/// sub-caption a `\parbox` of it, so captioned panels share a row as their bodies fit (pdflatex: Figure 1 one row of
/// four, Figure 2 "(a) Left" and "(b) Right" then "(c) Wide", Table 1 one row). Red (int58a 5 breaks, 78t r1 6): a
/// captioned panel was measured with its caption, a paragraph of the column's width, and took a row of its own; a
/// `\label` in a panel's body makes no box, where its whatsit is sized by its key's text (0907.1044). Witnesses
/// 2006.08949 (Figure 2), cond-mat9707225, astro-ph0205132, 0907.1044. Repro
/// captions-floats/subfigure_panel_is_as_wide_as_its_body.
#[test]
fn subfigure_panel_is_as_wide_as_its_body() {
  let xml = convert_clean(repro!(
    "captions-floats/subfigure_panel_is_as_wide_as_its_body.tex"
  ));
  let panel = |n: u32, letter: &str, width: &str, text: &str| {
    let labels = if n == 1 {
      " labels=\"LABEL:fig:left-panel-of-two\""
    } else {
      ""
    };
    format!(
      "<figure class=\"ltx_figure_panel\"{labels} xml:id=\"S0.F2.sf{n}\">{}<rule height=\"28.5pt\" width=\"{width}\"/>\
       <toccaption><tag close=\" \">({letter})</tag>{text}</toccaption>\
       <caption><tag close=\" \"><text fontsize=\"80%\" xml:id=\"S0.F2.sf{n}.2\">({letter})</text></tag>\
       <text fontsize=\"80%\" xml:id=\"S0.F2.sf{n}.3\">{text}</text></caption></figure>",
      panel_tags(&format!("S0.F2.sf{n}"), letter, &format!("2({letter})"))
    )
  };
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F2\""],
    &format!(
      "<figure inlist=\"lof\" xml:id=\"S0.F2\"><tags><tag>Figure 2</tag><tag role=\"refnum\">2</tag>\
       <tag role=\"typerefnum\">Figure 2</tag></tags>{}{}<break class=\"ltx_break\"/>{}\
       <toccaption><tag close=\" \">2</tag>Two.</toccaption><caption><tag close=\": \">Figure 2</tag>Two.</caption>\
       </figure>",
      panel(1, "a", "155.3pt", "Left"),
      panel(2, "b", "155.3pt", "Right"),
      panel(3, "c", "276.0pt", "Wide")
    ),
  );
  // Figure 1's four panels and Table 1's two each share a row: that break is the only one.
  assert_eq!(xml.matches("<break").count(), 1, "{xml}");
}

/// The last `\newlabel` of a key wins (latex.ltx `\@newl@bel`; pdflatex "See 1, 1, 3."): a `\label` takes its key
/// from the earlier elements labelled with it that contain it or that it contains
/// (`Document::release_earlier_label_holders`; holders apart keep it, the post-processor picks the later in document
/// order), where the post scan's registration, the outer element first, let an earlier label on an element inside a
/// later one win. Red (int58a
/// "See 1a, 1, 2."; 78t r1 "See 1a, 3, 3."; Perl "See 1(a), 1, 2.", KNOWN_PERL_ERRORS #1131). Witness 2001.00288
/// (`\label{algo2}` at a block's start and after `\caption`). Repro cross-references/the_later_label_wins.
#[test]
fn the_later_label_wins() {
  let xml = convert_clean(repro!("cross-references/the_later_label_wins.tex"));
  // Each key has one holder.
  for key in ["x", "y", "z"] {
    assert_eq!(
      xml.matches(&format!("labels=\"LABEL:{key}\"")).count(),
      1,
      "{key}: {xml}"
    );
  }
  assert!(
    xml.contains("<figure inlist=\"lof\" labels=\"LABEL:x\" xml:id=\"S0.F1\">"),
    "{xml}"
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1.sf1\""],
    "<figure inlist=\"lof\" xml:id=\"S0.F1.sf1\"><tags><tag><text fontsize=\"90%\" xml:id=\"S0.F1.sf1.1\">(a)</text>\
     </tag><tag role=\"refnum\">1a</tag></tags><rule height=\"28.5pt\" width=\"28.5pt\"/>\
     <toccaption><tag close=\" \">a</tag>Panel</toccaption><caption><tag close=\" \">\
     <text fontsize=\"90%\" xml:id=\"S0.F1.sf1.2\">(a)</text></tag>\
     <text fontsize=\"90%\" xml:id=\"S0.F1.sf1.3\">Panel</text></caption></figure>",
  );
  assert!(
    xml.contains(
      "<float class=\"ltx_algorithm\" framed=\"top\" labels=\"LABEL:y\" xml:id=\"algorithm1\">"
    ),
    "{xml}"
  );
  assert_element(
    &xml,
    "listingline",
    &["xml:id=\"algorithm1.3.3\""],
    "<listingline xml:id=\"algorithm1.3.3\"><tags><tag><text font=\"bold\" xml:id=\"algorithm1.3.3.1\">3</text></tag>\
     <tag role=\"refnum\">3</tag></tags><text font=\"bold\" xml:id=\"algorithm1.3.3.2\">begin</text></listingline>",
  );
  assert!(
    xml.contains("<float class=\"ltx_algorithm\" framed=\"top\" xml:id=\"algorithm2\">"),
    "{xml}"
  );
  assert_element(
    &xml,
    "listingline",
    &["labels=\"LABEL:z\""],
    "<listingline labels=\"LABEL:z\" xml:id=\"algorithm2.1\"><tags>\
     <tag><text font=\"bold\" xml:id=\"algorithm2.1.1\">3</text></tag><tag role=\"refnum\">3</tag></tags>\
     Line;</listingline>",
  );
}

/// The rows of the float `id`: its sub-floats' letters (the last letter of their ids, `sf1` → `a`), `|` for each
/// `ltx:break`, the panels marked `ltx_figure_panel` in capitals.
fn float_rows(xml: &str, id: &str) -> String {
  let start = xml
    .find(&format!("xml:id=\"{id}\">"))
    .unwrap_or_else(|| panic!("no {id}: {xml}"));
  let mut rows = Vec::new();
  // The float's own tag is open: depth 1 until its `</figure>`/`</table>`.
  let mut depth = 1;
  let mut rest = &xml[start..];
  while let Some(at) = rest.find('<') {
    rest = &rest[at..];
    let tag_end = rest.find('>').map_or(rest.len(), |end| end + 1);
    let tag = &rest[..tag_end];
    rest = &rest[tag_end..];
    if tag.starts_with("</figure") || tag.starts_with("</table") {
      depth -= 1;
      if depth == 0 {
        break;
      }
    } else if tag.starts_with("<break") && depth == 1 {
      rows.push("|".to_string());
    } else if tag.starts_with("<figure") || tag.starts_with("<table") {
      if !tag.ends_with("/>") {
        depth += 1;
      }
      if let Some(n) = tag
        .split(&format!("xml:id=\"{id}.sf"))
        .nth(1)
        .and_then(|r| r.split('"').next())
        .and_then(|n| n.parse::<u8>().ok())
      {
        let letter = char::from(b'a' + n - 1);
        rows.push(if tag.contains("ltx_figure_panel") {
          letter.to_ascii_uppercase().to_string()
        } else {
          letter.to_string()
        });
      }
    }
  }
  rows.join(" ")
}

/// A sub-figure body `{⏎ \rule{0.475\textwidth}{1cm}⏎ \label{a1}⏎}` is as wide as its rule (pdflatex `\wd\@tempboxa`
/// 170.54pt with or without the label: latex.ltx's `\label` is `\@bsphack…\@esphack`), and four `0.23\textwidth` panels
/// with ` \label`s and `\hfill`s share a row (pdflatex: both figures one row, "See 1(a), 1(b), 2(a), 2(d)."): the
/// panel's stated width leaves its labels and its glue out (`note_sub_float_body_width`, 78t r5). Red (78t r2: both
/// spaces counted, Figure 1 one panel per row, Figure 2 3+1; int60a one per row). Witnesses 2511.21020 Figs 7-8, 1607.02761, 2211.12798
/// `fig:dataset`, 1601.02116 `fig:EpsW-E`. Repro captions-floats/a_label_between_spaces_counts_one.
#[test]
fn a_label_between_spaces_counts_one() {
  let xml = convert_clean(repro!(
    "captions-floats/a_label_between_spaces_counts_one.tex"
  ));
  let panel = |n: u32, letter: &str, text: &str| {
    format!(
      "<figure align=\"center\" class=\"ltx_figure_panel\" labels=\"LABEL:a{n}\" xml:id=\"S0.F1.sf{n}\">{}\
       <rule height=\"28.5pt\" width=\"163.9pt\"/><toccaption><tag close=\" \">({letter})</tag>{text}</toccaption>\
       <caption><tag close=\" \"><text fontsize=\"80%\" xml:id=\"S0.F1.sf{n}.2\">({letter})</text></tag>\
       <text fontsize=\"80%\" xml:id=\"S0.F1.sf{n}.3\">{text}</text></caption></figure>",
      panel_tags(&format!("S0.F1.sf{n}"), letter, &format!("1({letter})"))
    )
  };
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "<figure inlist=\"lof\" labels=\"LABEL:f1\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag>\
       <tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags>{}{}\
       <toccaption class=\"ltx_centering\"><tag close=\" \">1</tag>A label between two spaces.</toccaption>\
       <caption class=\"ltx_centering\"><tag close=\": \">Figure 1</tag>A label between two spaces.</caption></figure>",
      panel(1, "a", "One"),
      panel(2, "b", "Two")
    ),
  );
  assert_eq!(float_rows(&xml, "S0.F2"), "A B C D", "{xml}");
  assert_eq!(xml.matches("<break").count(), 0, "{xml}");
}

/// epsfig's `\psfig` is sized by its keys and its file (Perl epsfig.sty.ltxml:37 `sizer => \&image_graphicx_sizer`),
/// so three `\subfigure[…]{\psfig{file=…,width=0.4\textwidth}}` panels are two and one, as latex + dvips set them, and
/// a pair of uncaptioned ones are both panels. Red (78t r2: a box of 0 width, Figure 1 one row of three; int60a one
/// per row, Figure 2's panels unmarked). Witnesses hep-ex0703011, cond-mat9707225, 1610.06413 Figs 2, 4,
/// nlin0606018. Repro captions-floats/psfig_panels_are_as_wide_as_their_keys.
#[test]
fn psfig_panels_are_as_wide_as_their_keys() {
  let xml = convert_clean(repro!(
    "captions-floats/psfig_panels_are_as_wide_as_their_keys.tex"
  ));
  let panel = |n: u32, letter: &str, text: &str| {
    format!(
      "<figure align=\"center\" class=\"ltx_figure_panel\" xml:id=\"S0.F1.sf{n}\">{}\
       <graphics graphic=\"panel\" options=\"width=137.9979pt\" xml:id=\"S0.F1.sf{n}.g1\"/>\
       <toccaption><tag close=\" \">({letter})</tag>{text}</toccaption>\
       <caption><tag close=\" \"><text fontsize=\"80%\" xml:id=\"S0.F1.sf{n}.2\">({letter})</text></tag>\
       <text fontsize=\"80%\" xml:id=\"S0.F1.sf{n}.3\">{text}</text></caption></figure>",
      panel_tags(&format!("S0.F1.sf{n}"), letter, &format!("1({letter})"))
    )
  };
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    &format!(
      "<figure inlist=\"lof\" labels=\"LABEL:f1\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag>\
       <tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags>{}{}<break class=\"ltx_break\"/>{}\
       <toccaption class=\"ltx_centering\"><tag close=\" \">1</tag>Three psfig panels.</toccaption>\
       <caption class=\"ltx_centering\"><tag close=\": \">Figure 1</tag>Three psfig panels.</caption></figure>",
      panel(1, "a", "One"),
      panel(2, "b", "Two"),
      panel(3, "c", "Three")
    ),
  );
  assert_eq!(float_rows(&xml, "S0.F2"), "A B", "{xml}");
}

/// A sub-figure body of no width without its glue (`\subfigure[]{⏎\mbox{}}`; a graphic whose file we did not find, as
/// 1408.2843 Fig 1's `recomb.eps` shipping only `recomb.pdf`) is measured again with its glue, so two such panels share
/// a row as pdflatex sets them (both sub-captions on one baseline): `note_sub_float_body_width` falls back to the box
/// only when the body is 0 wide with its glue too. Red (78t r5: one panel per row, the captioned box measured; int61a
/// one row). Review rev78t5 finding 1. Repro captions-floats/subfigure_panels_of_no_known_width_share_a_row.
#[test]
fn subfigure_panels_of_no_known_width_share_a_row() {
  let xml = convert_clean(repro!(
    "captions-floats/subfigure_panels_of_no_known_width_share_a_row.tex"
  ));
  assert_eq!(float_rows(&xml, "S0.F1"), "A B", "{xml}");
  assert_eq!(xml.matches("<break").count(), 0, "{xml}");
}
