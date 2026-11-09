//! Red/green guards for perfect-kernel batch 64zc: an email or web address a class sets in typewriter type (`\url`,
//! `\texttt`, `\ttfamily`) is set in typewriter type, so the `_` and `~` a class reads as other characters print as
//! typed rather than as the roman OT1 font's dot and tilde accents ("ann˙able@a.edu"); one a class sets in the roman font
//! keeps that font and its `_` (user ruling 2026-10-11).
use super::perfect_kernel_batch46::{error_count, warning_count};

/// The XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> String {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// Every creator of `xml`, whole and in document order.
fn creators(xml: &str) -> Vec<String> {
  let mut creators = Vec::new();
  let mut rest = xml;
  while let Some(i) = rest.find("<creator ") {
    let end = i
      + rest[i..]
        .find("</creator>")
        .map_or(0, |j| j + "</creator>".len());
    creators.push(rest[i..end].to_string());
    rest = &rest[end..];
  }
  creators
}

/// 64zc: REVTeX 4.0 sets `\email` and `\homepage` with `\url`, its `\href` fallback (revtex4.cls:4476-4493): '_' and
/// '~' as typed, in typewriter type (1001.1076). Repro sectioning-frontmatter/revtex4_email_address_in_typewriter.
#[test]
fn revtex4_email_address_in_typewriter() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex4_email_address_in_typewriter.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">ann_able@a.edu</text></contact>\n    <contact name=\"URL:\u{a0}\" role=\"url\"><text font=\"typewriter\" xml:id=\"id2\">http://a.edu/~ann_able/</text></contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University of A</contact>\n  </creator>",
  ]);
}

/// 64zc: amsart sets `\email` and `\urladdr` in `\ttfamily` (amsart.cls:538-546): `ann\string_able@a.edu` reads
/// '_' (2101.10462, 2501.04035). Repro sectioning-frontmatter/amsart_email_address_in_typewriter.
#[test]
fn amsart_email_address_in_typewriter() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/amsart_email_address_in_typewriter.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\">University of A</contact>\n    <contact name=\"Email address: \" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">ann_able@a.edu</text></contact>\n    <contact name=\"URL: \" role=\"url\"><text font=\"typewriter\" xml:id=\"id2\">http://a.edu/~ann_able</text></contact>\n  </creator>",
  ]);
}

/// 64zc: elsarticle sets an `\ead` address, a `[url]` one too, in typewriter type (elsarticle.cls:215-219, :232-235).
/// Repro sectioning-frontmatter/elsarticle_ead_address_in_typewriter.
#[test]
fn elsarticle_ead_address_in_typewriter() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/elsarticle_ead_address_in_typewriter.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">ann_able@a.edu</text></contact>\n    <contact name=\"URL:\u{a0}\" role=\"url\"><text font=\"typewriter\" xml:id=\"id2\">http://a.edu/~ann_able</text></contact>\n    <contact name=\"Address:\u{a0}\" role=\"address\">University of A</contact>\n  </creator>",
  ]);
}

/// 64zc control: REVTeX 4.2, which the revtex4 binding also serves, sets its `\email` in the current font
/// (revtex4-2.cls:1980 `\@href`): under T1 the address stays roman, its `_` as typed. Repro
/// sectioning-frontmatter/revtex4_2_t1_email_stays_roman.
#[test]
fn revtex4_2_t1_email_stays_roman() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex4_2_t1_email_stays_roman.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\">ann_able@a.edu</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University of A</contact>\n  </creator>",
  ]);
}

/// 64zc (user ruling 2026-10-11): REVTeX 4.2 sets its `\email` in the current font (revtex4-1.cls:1959-1961 `\@href`),
/// where OT1's 0x5F is the dot accent and pdflatex prints "ann˙able@a.edu"; the address is the author's, so it keeps
/// the roman font and its `_`. Repro sectioning-frontmatter/revtex4_2_ot1_email_keeps_its_underscore.
#[test]
fn revtex4_2_ot1_email_keeps_its_underscore() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex4_2_ot1_email_keeps_its_underscore.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\">ann_able@a.edu</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University of A</contact>\n  </creator>",
  ]);
}

/// 64zc review: the ASCII encoding of an address keeps `\i` and `\j` (its 0x10/0x11 are OT1's dotless ı and ȷ), so
/// `\'{\i}` is "í", not the accent alone. Repro sectioning-frontmatter/address_dotless_i_survives_ascii.
#[test]
fn address_dotless_i_survives_ascii() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/address_dotless_i_survives_ascii.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>María Able</personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\">University of A</contact>\n    <contact name=\"Email address: \" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">maría@a.edu</text></contact>\n  </creator>",
  ]);
}

/// 64zc review: each class's address path keeps its class's font and the `_`: AASTeX 6.x `\email` and `\homepage` in
/// roman (aastex631.cls:6924-6930, revtex4-1.cls:2387-2390; 2301.02349), an svjour3 `\institute` line's `\email` in roman (svjour3.cls:303;
/// 2101.02062), jheppub's `\emailAdd` in typewriter type (jheppub.sty:127-131; 2301.03278), and a labelled
/// `E-mail: \email{…}` author line under REVTeX 4.2 in roman (`email_calls`); AASTeX 5.x's `\email` in typewriter type
/// (aastex.cls:903-905, `\tt`; 1001.0026) and emulateapj's in roman (REVTeX 4.1's `\href`, emulateapj.cls:151-167).
#[test]
fn class_email_paths_keep_their_font_and_underscore() {
  let creators_of = |tex: &str| creators(&convert_clean(tex));
  let aas = |class: &str| {
    format!(
      "\\documentclass{{{class}}}\n\\begin{{document}}\n\\title{{T}}\n\\author{{A}}\n\\email{{ann_able@a.edu}}\n\\homepage{{http://a.edu/~ann_able/}}\n\\affiliation{{U}}\n\\begin{{abstract}}x\\end{{abstract}}\n\\maketitle\nx\n\\end{{document}}\n"
    )
  };
  assert_eq!(creators_of(&aas("aastex")), vec![
    "<creator role=\"author\">\n    <personname>A</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">ann_able@a.edu</text></contact>\n    <contact name=\"URL:\u{a0}\" role=\"url\"><text font=\"typewriter\" xml:id=\"id2\">http://a.edu/~ann_able/</text></contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">U</contact>\n  </creator>",
  ]);
  assert_eq!(creators_of(&aas("emulateapj")), vec![
    "<creator role=\"author\">\n    <personname>A</personname>\n    <contact name=\"Email:\u{a0}\" role=\"email\">ann_able@a.edu</contact>\n    <contact name=\"URL:\u{a0}\" role=\"url\">http://a.edu/~ann_able/</contact>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">U</contact>\n  </creator>",
  ]);
  assert_eq!(
    creators_of(include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/aastex631_email_keeps_its_underscore.tex"
    )),
    vec![
      "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University of A</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\">ann_able@a.edu</contact>\n    <contact name=\"URL:\u{a0}\" role=\"url\">http://a.edu/~ann_able/</contact>\n  </creator>",
    ]
  );
  assert_eq!(
    creators_of(include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/svjour3_institute_email_keeps_its_underscore.tex"
    )),
    vec![
      "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University of A </contact>\n    <contact name=\"E-mail: \" role=\"email\">ann_able@a.edu</contact>\n  </creator>",
      "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Bee</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University of B</contact>\n  </creator>",
    ]
  );
  assert_eq!(
    creators_of(include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/jheppub_emailadd_in_typewriter.tex"
    )),
    vec![
      "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University of A</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\"><text font=\"typewriter\" xml:id=\"id1\">ann_able@a.edu</text></contact>\n  </creator>",
    ]
  );
  assert_eq!(
    creators_of(include_str!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex4_2_author_email_line_keeps_its_underscore.tex"
    )),
    vec![
      "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University of A</contact>\n    <contact name=\"E-mail: \" role=\"email\">ann_able@a.edu</contact>\n  </creator>",
    ]
  );
}

/// 64zc review: the accent of an empty letter (`\~{}`, `\^{}`, `\'{}`) is the current font's, one run with the text
/// around it (Perl Box.pm:31), where it came out font-less between two runs; in typewriter type `\~{}`/`\^{}` are cmtt's
/// `~`/`^` (1001.1367, 1301.6104). Repro fonts-nfss/accent_of_empty_letter_keeps_the_run.
#[test]
fn accent_of_empty_letter_keeps_the_run() {
  let xml = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/fonts-nfss/accent_of_empty_letter_keeps_the_run.tex"
  ));
  let start = xml.find("<p xml:id=\"p1.1\">").expect("the paragraph");
  let end = start + xml[start..].find("</p>").expect("its end") + "</p>".len();
  assert_eq!(
    &xml[start..end],
    "<p xml:id=\"p1.1\"><text font=\"typewriter\" xml:id=\"p1.1.1\">a~b a^b</text> <text font=\"italic\" xml:id=\"p1.1.2\">a˜b</text> <text font=\"sansserif\" xml:id=\"p1.1.3\">a´b</text></p>"
  );
}
