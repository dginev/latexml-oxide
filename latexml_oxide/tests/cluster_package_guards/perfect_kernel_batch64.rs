//! Red/green guards for perfect-kernel phase-64 batches: IEEEtran's author reference marks linking names to their
//! affiliation blocks, algorithm2e lines inside an `{algorithmic}` listing, the mathtools `XPP` pre-code, and a caption
//! in a minipage beside an `{adjustbox}`.
use latexml::util::test::assert_element;

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

/// 64a: a document's own `\IEEEauthorrefmark` (`\smash{\textsuperscript{\footnotesize\ensuremath{#1}}}`, 2502.16662,
/// 2411.14110) prints the mark under a size switch, past which the mark is read: each marked line of the block goes to
/// the names showing its mark. Repro sectioning-frontmatter/ieee_refmark_sized_marks_link.
#[test]
fn ieee_refmark_sized_marks_link() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ieee_refmark_sized_marks_link.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able<sup xml:id=\"id1\"><Math mode=\"inline\" tex=\"1\" text=\"1\" xml:id=\"m1\">\n          <XMath xml:id=\"m1.1\">\n            <XMTok fontsize=\"80%\" meaning=\"1\" role=\"NUMBER\">1</XMTok>\n          </XMath>\n        </Math></sup></personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ A, Country</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>Bob Baker<sup xml:id=\"id2\"><Math mode=\"inline\" tex=\"2\" text=\"2\" xml:id=\"m2\">\n          <XMath xml:id=\"m2.1\">\n            <XMTok fontsize=\"80%\" meaning=\"2\" role=\"NUMBER\">2</XMTok>\n          </XMath>\n        </Math></sup></personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ B, Country</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>Cy Coe<sup xml:id=\"id3\"><Math mode=\"inline\" tex=\"2\" text=\"2\" xml:id=\"m3\">\n          <XMath xml:id=\"m3.1\">\n            <XMTok fontsize=\"80%\" meaning=\"2\" role=\"NUMBER\">2</XMTok>\n          </XMath>\n        </Math></sup></personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Univ B, Country</contact>\n  </creator>",
  ]);
}

/// 64a: `\IEEEauthorrefmark` is a mark wherever a block is read — `Google, \IEEEauthorrefmark{3}Google DeepMind` is
/// two affiliations (2310.02368), and an address under one author's mark is that author's (2404.05610), while a
/// shared affiliation's address continues it and an address under two marks stays the block's (OD #52(j), #159).
/// Repro sectioning-frontmatter/ieee_refmark_address_and_marks_in_a_line.
#[test]
fn ieee_refmark_address_and_marks_in_a_line() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ieee_refmark_address_and_marks_in_a_line.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ben Steen<sup xml:id=\"id1\">1</sup></personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Microsoft Data &amp; AI<break/>{steen,sund}@microsoft.com</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>Mic Tufo<sup xml:id=\"id2\">2</sup></personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Google</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>Alex Svy<sup xml:id=\"id3\">3</sup></personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\">Google DeepMind</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>Dem Hespe<sup xml:id=\"id4\">4</sup></personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\"><text font=\"italic\" xml:id=\"id5\">Independent</text></contact>\n    <contact name=\"Email: \" role=\"email\">dem.hespe@outlook.com</contact>\n  </creator>",
    "<creator role=\"author\">\n    <contact name=\"Email: \" role=\"email\">{tufo,svy}@google.com</contact>\n  </creator>",
  ]);
}

/// 64a: an ORCID command's digits (`\orcidicon{0000-0001-8953-1075}`) are no name text, so the name list splits
/// (2104.02493, 2407.03625). Repro sectioning-frontmatter/ieee_refmark_orcid_digits_are_no_name.
#[test]
fn ieee_refmark_orcid_digits_are_no_name() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ieee_refmark_orcid_digits_are_no_name.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ole Schu<sup xml:id=\"id1\">1</sup><ref class=\"ltx_href\" href=\"https://orcid.org/0000-0001-8953-1075\">iD</ref></personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\"><text font=\"italic\" xml:id=\"id2\">Lab A</text>, Stuttgart</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>Mark Hahn<sup xml:id=\"id3\">2</sup></personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\"><text font=\"italic\" xml:id=\"id4\">Firm B</text>, Ulm</contact>\n  </creator>",
    "<creator before=\"  \" role=\"author\">\n    <personname>Nic Schein<sup xml:id=\"id5\">1</sup><ref class=\"ltx_href\" href=\"https://orcid.org/0000-0002-5176-6159\">iD</ref></personname>\n    <contact name=\"Affiliation: \" role=\"affiliation\"><text font=\"italic\" xml:id=\"id6\">Lab A</text>, Stuttgart</contact>\n  </creator>",
  ]);
}

/// 64a: an algorithm2e block in an `{algorithmic}` line (`\STATE \For{…}{body}`, 2301.04312) opens its lines in that
/// listing, the nearest one, so the body is a line of it, not bare text in the listing.
/// Repro captions-floats/algorithm2e_block_in_algorithmic_line.
#[test]
fn algorithm2e_block_in_algorithmic_line() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/algorithm2e_block_in_algorithmic_line.tex"
  );
  let xml = convert_clean(tex);
  assert_element(
    &xml,
    "listing",
    &["xml:id=\"alg1.3.1.1.1\""],
    "<listing xml:id=\"alg1.3.1.1.1\"><listingline xml:id=\"alg1.l1\"><tags><tag><text fontsize=\"80%\" xml:id=\"alg1.l1.1\">1:</text></tag><tag role=\"refnum\">1</tag></tags>\u{2002}<Math mode=\"inline\" tex=\"W\\leftarrow 0\" text=\"W leftarrow 0\" xml:id=\"alg1.l1.m1\"><XMath xml:id=\"alg1.l1.m1.1\"><XMApp xml:id=\"alg1.l1.m1.1.1\"><XMTok name=\"leftarrow\" role=\"ARROW\">\u{2190}</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">W</XMTok><XMTok meaning=\"0\" role=\"NUMBER\">0</XMTok></XMApp></XMath></Math>;</listingline><listingline xml:id=\"alg1.l2\"><tags><tag><text fontsize=\"80%\" xml:id=\"alg1.l2.1\">2:</text></tag><tag role=\"refnum\">2</tag></tags>\u{2002}<text font=\"bold\" xml:id=\"alg1.l2.2\">for</text> <emph font=\"italic\" xml:id=\"alg1.l2.3\"><Math mode=\"inline\" tex=\"i=1\" text=\"i = 1\" xml:id=\"alg1.l2.m1\"><XMath xml:id=\"alg1.l2.m1.1\"><XMApp xml:id=\"alg1.l2.m1.1.1\"><XMTok font=\"upright\" meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok role=\"UNKNOWN\">i</XMTok><XMTok font=\"upright\" meaning=\"1\" role=\"NUMBER\">1</XMTok></XMApp></XMath></Math> <text font=\"bold upright\" xml:id=\"alg1.l2.3.1\">to</text> <Math mode=\"inline\" tex=\"n\" text=\"n\" xml:id=\"alg1.l2.m2\"><XMath xml:id=\"alg1.l2.m2.1\"><XMTok role=\"UNKNOWN\">n</XMTok></XMath></Math></emph> <text font=\"bold\" xml:id=\"alg1.l2.4\">do</text></listingline><listingline xml:id=\"alg1.3.1.1.1.1\">\u{2002}<rule height=\"100%\" width=\"1px\"/>\u{2003}<Math mode=\"inline\" tex=\"W\\leftarrow W+1\" text=\"W leftarrow W + 1\" xml:id=\"alg1.m1\"><XMath xml:id=\"alg1.m1.1\"><XMApp xml:id=\"alg1.m1.1.1\"><XMTok name=\"leftarrow\" role=\"ARROW\">\u{2190}</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">W</XMTok><XMApp xml:id=\"alg1.m1.1.1.3\"><XMTok meaning=\"plus\" role=\"ADDOP\">+</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">W</XMTok><XMTok meaning=\"1\" role=\"NUMBER\">1</XMTok></XMApp></XMApp></XMath></Math>  Append <emph font=\"italic\" xml:id=\"alg1.3.1.1.1.1.1\">x</emph><break/></listingline><listingline xml:id=\"alg1.3.1.1.1.2\">end for</listingline><listingline xml:id=\"alg1.l3\"><tags><tag><text fontsize=\"80%\" xml:id=\"alg1.l3.1\">3:</text></tag><tag role=\"refnum\">3</tag></tags>\u{2002}<text font=\"bold\" xml:id=\"alg1.l3.2\">return</text> <Math mode=\"inline\" tex=\"W\" text=\"W\" xml:id=\"alg1.l3.m1\"><XMath xml:id=\"alg1.l3.m1.1\"><XMTok font=\"italic\" role=\"UNKNOWN\">W</XMTok></XMath></Math>;</listingline></listing>",
  );
}

/// 64a: a line algorithm2e's startline opened that closes with no statement is dropped, whoever closes it — the
/// `{algorithmic}` line after `\STATE …\\` (2402.07867), the float's end after the last line.
/// Repro captions-floats/algorithm2e_line_closed_empty_is_dropped.
#[test]
fn algorithm2e_line_closed_empty_is_dropped() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/algorithm2e_line_closed_empty_is_dropped.tex"
  );
  let xml = convert_clean(tex);
  let empty_line = regex::Regex::new(r"<listingline\b[^>]*/>").unwrap();
  assert!(!empty_line.is_match(&xml), "{xml}");
  assert_element(
    &xml,
    "listing",
    &["xml:id=\"alg1.3.1.1.1\""],
    "<listing xml:id=\"alg1.3.1.1.1\"><listingline xml:id=\"alg1.l1\"><tags><tag role=\"refnum\">1</tag></tags>\u{2002}<text font=\"bold\" xml:id=\"alg1.l1.1\">Output:</text> texts.<break/></listingline><listingline xml:id=\"alg1.l2\"><tags><tag role=\"refnum\">2</tag></tags>\u{2002}<text font=\"bold\" xml:id=\"alg1.l2.1\">for</text> <Math mode=\"inline\" tex=\"i=1\" text=\"i = 1\" xml:id=\"alg1.l2.m1\"><XMath xml:id=\"alg1.l2.m1.1\"><XMApp xml:id=\"alg1.l2.m1.1.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">i</XMTok><XMTok meaning=\"1\" role=\"NUMBER\">1</XMTok></XMApp></XMath></Math> <text font=\"bold\" xml:id=\"alg1.l2.2\">do</text></listingline><listingline xml:id=\"alg1.l3\"><tags><tag role=\"refnum\">3</tag></tags>\u{2003}\u{2002}<Math mode=\"inline\" tex=\"x=1\" text=\"x = 1\" xml:id=\"alg1.l3.m1\"><XMath xml:id=\"alg1.l3.m1.1\"><XMApp xml:id=\"alg1.l3.m1.1.1\"><XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok><XMTok meaning=\"1\" role=\"NUMBER\">1</XMTok></XMApp></XMath></Math><break/></listingline><listingline xml:id=\"alg1.l4\"><tags><tag role=\"refnum\">4</tag></tags>\u{2002}<text font=\"bold\" xml:id=\"alg1.l4.1\">end</text> <text font=\"bold\" xml:id=\"alg1.l4.2\">for</text></listingline><listingline xml:id=\"alg1.l5\"><tags><tag role=\"refnum\">5</tag></tags>\u{2002}<text font=\"bold\" xml:id=\"alg1.l5.1\">return</text> <Math mode=\"inline\" tex=\"x\" text=\"x\" xml:id=\"alg1.l5.m1\"><XMath xml:id=\"alg1.l5.m1.1\"><XMTok font=\"italic\" role=\"UNKNOWN\">x</XMTok></XMath></Math></listingline></listing>",
  );
}

/// 64a: `\DeclarePairedDelimiterXPP`'s pre-code holds the command's parameters (`\mathbb{E}_{#1}`, 2310.04475), so it
/// is part of the macro body the arguments substitute into (mathtools.sty:953-988), not spliced in after it.
/// Repro math-parse/mathtools_xpp_precode_parameter.
#[test]
fn mathtools_xpp_precode_parameter() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/math-parse/mathtools_xpp_precode_parameter.tex"
  );
  let xml = convert_clean(tex);
  assert_element(
    &xml,
    "Math",
    &["xml:id=\"p1.m2\""],
    "<Math mode=\"inline\" tex=\"\\mathbb{E}_{s}[x]\" text=\"(E _ s)@(x)\" xml:id=\"p1.m2\">
      <XMath xml:id=\"p1.m2.3\">
        <XMDual xml:id=\"p1.m2.3.1\">
          <XMApp xml:id=\"p1.m2.3.1.1\">
            <XMRef idref=\"p1.m2.1\" xml:id=\"p1.m2.3.1.1.1\"/>
            <XMRef idref=\"p1.m2.2\" xml:id=\"p1.m2.3.1.1.2\"/>
          </XMApp>
          <XMApp xml:id=\"p1.m2.3.1.2\">
            <XMApp xml:id=\"p1.m2.1\">
              <XMTok role=\"SUBSCRIPTOP\" scriptpos=\"post1\"/>
              <XMTok font=\"blackboard\" role=\"UNKNOWN\">E</XMTok>
              <XMTok font=\"italic\" fontsize=\"70%\" role=\"UNKNOWN\">s</XMTok>
            </XMApp>
            <XMWrap xml:id=\"p1.m2.3.1.2.1\">
              <XMTok role=\"OPEN\" stretchy=\"false\">[</XMTok>
              <XMTok font=\"italic\" role=\"UNKNOWN\" xml:id=\"p1.m2.2\">x</XMTok>
              <XMTok role=\"CLOSE\" stretchy=\"false\">]</XMTok>
            </XMWrap>
          </XMApp>
        </XMDual>
      </XMath>
    </Math>",
  );
}

/// 64a: a minipage holding `\caption` and an `{adjustbox}` (whose collectbox `\noindent` opens a paragraph in the
/// block capture, 2312.04535) becomes its table: the paragraph wrapper is unwrapped where the float holds the
/// caption and the box, as a lone captioned panel is the float (OD #447).
/// Repro captions-floats/caption_in_minipage_with_adjustbox.
#[test]
fn caption_in_minipage_with_adjustbox() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/caption_in_minipage_with_adjustbox.tex"
  );
  let xml = convert_clean(tex);
  assert_element(
    &xml,
    "table",
    &["xml:id=\"S0.T1\""],
    "<table class=\"ltx_minipage ltx_pruned_first\" inlist=\"lot\" labels=\"LABEL:t:a\" vattach=\"middle\" width=\"138.0pt\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><toccaption><tag close=\" \">1</tag>First</toccaption><caption><tag close=\": \">Table 1</tag>First</caption><inline-block depth=\"0.0pt\" height=\"4.3pt\" width=\"5.0pt\" xml:id=\"S0.T1.1\" xscale=\"1\" xtranslate=\"0.0pt\" yscale=\"1\" ytranslate=\"0.0pt\"><p xml:id=\"S0.T1.1.1\"><text xml:id=\"S0.T1.1.1.1\">a</text></p></inline-block></table>",
  );
}

/// 64a: `\centering` in the minipage puts `align` on the paragraph the capture unwraps; the box is centred as a box,
/// `ltx_centering` (64a review: the unwrap had dropped it, then an `align` copied onto the box centred only its own
/// text). Repro captions-floats/caption_in_centered_minipage_with_adjustbox.
#[test]
fn caption_in_centered_minipage_with_adjustbox() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/caption_in_centered_minipage_with_adjustbox.tex"
  );
  let xml = convert_clean(tex);
  assert_element(
    &xml,
    "table",
    &["xml:id=\"S0.T1\""],
    "<table class=\"ltx_minipage ltx_pruned_first\" inlist=\"lot\" labels=\"LABEL:t:a\" vattach=\"middle\" width=\"138.0pt\" xml:id=\"S0.T1\"><tags><tag>Table 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Table 1</tag></tags><toccaption class=\"ltx_centering\"><tag close=\" \">1</tag>First</toccaption><caption class=\"ltx_centering\"><tag close=\": \">Table 1</tag>First</caption><inline-block class=\"ltx_centering\" depth=\"0.0pt\" height=\"4.3pt\" width=\"5.0pt\" xml:id=\"S0.T1.1\" xscale=\"1\" xtranslate=\"0.0pt\" yscale=\"1\" ytranslate=\"0.0pt\"><p xml:id=\"S0.T1.1.1\"><text xml:id=\"S0.T1.1.1.1\">a</text></p></inline-block></table>",
  );
}

/// 64a: an author mark that no affiliation answers stays in the name, as LaTeX prints it — an `\IEEEauthorrefmark`
/// whose legend is a `\thanks` (2408.02464; 64a review: read as requests, they had vanished).
/// Repro sectioning-frontmatter/ieee_refmark_unanswered_stays_printed.
#[test]
fn ieee_refmark_unanswered_stays_printed() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ieee_refmark_unanswered_stays_printed.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able<sup xml:id=\"id1\">1</sup></personname>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker<sup xml:id=\"id2\">2</sup></personname>\n    <note class=\"ltx_note_frontmatter ltx_thanks_correspondence\" role=\"thanks\" xml:id=\"id3\"><sup xml:id=\"id3.1\">1</sup>Equal contribution. <sup xml:id=\"id3.2\">2</sup>Corresponding author.</note>\n  </creator>",
  ]);
}

/// 64a: `Ann Able$^*$ and Bob Bee$^1$` with no `$^1$` line: Bob's unanswered mark stays in his name (the RED of 63x).
/// Repro sectioning-frontmatter/author_unmatched_numeric_mark.
#[test]
fn author_unmatched_numeric_mark() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_unmatched_numeric_mark.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able<sup xml:id=\"id1\">*</sup></personname>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Bee<sup xml:id=\"id2\">1</sup></personname>\n  </creator>",
  ]);
}

/// 64b: a name list in a bare group is a wrapper too, and a line opening with "and" (inside its group) continues the
/// names with that separator (2306.02486).
/// Repro sectioning-frontmatter/author_bare_group_names_and_led_line.
#[test]
fn author_bare_group_names_and_led_line() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_bare_group_names_and_led_line.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cat Cole</personname>\n  </creator>",
  ]);
}

/// 64b: marked names in a bare group split as in a declaration group (2312.04684, 2501.04001).
/// Repro sectioning-frontmatter/author_bare_group_marked_names.
#[test]
fn author_bare_group_marked_names() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_bare_group_marked_names.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">Intern.</note>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cat Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
  ]);
}

/// 64b: mnras's `\newauthor{…}` group holding a name list (2409.07518, 2312.07625).
/// Repro sectioning-frontmatter/mnras_newauthor_group_names.
#[test]
fn mnras_newauthor_group_names() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/mnras_newauthor_group_names.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cat Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Dan Dorn</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact>\n  </creator>",
  ]);
}

/// 64b: a `\vspace{…}` after a wrapper of names is spacing, not its tail (2404.02905, 2412.05271).
/// Repro sectioning-frontmatter/author_vspace_after_wrapper_names.
#[test]
fn author_vspace_after_wrapper_names() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_vspace_after_wrapper_names.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cat Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
  ]);
}

/// 64b: spacing before a wrapper of names stays with the first (2512.22234, 2508.05004).
/// Repro sectioning-frontmatter/author_vspace_before_wrapper_names.
#[test]
fn author_vspace_before_wrapper_names() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_vspace_before_wrapper_names.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cat Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Dan Dorn</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact>\n  </creator>",
  ]);
}

/// 64b: a `\quad` beside a wrapper's closing comma is spacing (2407.15815).
/// Repro sectioning-frontmatter/author_quad_at_wrapper_edge.
#[test]
fn author_quad_at_wrapper_edge() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_quad_at_wrapper_edge.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ B</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cat Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Dan Dorn</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"> Univ B</contact>\n  </creator>",
  ]);
}

/// 64b: `\and` inside a wrapper separates names (llncs `\large{A \and B}`, 2402.15967).
/// Repro sectioning-frontmatter/llncs_and_inside_wrapper.
#[test]
fn llncs_and_inside_wrapper() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/llncs_and_inside_wrapper.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname><text fontsize=\"120%\" xml:id=\"id1\">Ann Able<sup xml:id=\"id1.1\">1</sup></text></personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id2\">1</sup>Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname><text fontsize=\"120%\" xml:id=\"id3\">Bob Baker<sup xml:id=\"id3.1\">2</sup></text></personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\"><sup xml:id=\"id4\">2</sup>Univ B</contact>\n  </creator>",
  ]);
}

/// 64b: `\and` inside a group of names (amsart `{\bf {\large A \and B}}`, 2401.03555).
/// Repro sectioning-frontmatter/amsart_and_inside_group.
#[test]
fn amsart_and_inside_group() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/amsart_and_inside_group.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\" and \" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
  ]);
}

/// 64b: a macro printing only a comma is a separator (`\authcomma`, 2403.07809).
/// Repro sectioning-frontmatter/author_separator_macro_comma.
#[test]
fn author_separator_macro_comma() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_separator_macro_comma.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cat Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
  ]);
}

/// 64b: a macro printing ", and " is a separator (tibop-article `\lastand`, 2401.13365).
/// Repro sectioning-frontmatter/author_separator_macro_lastand.
#[test]
fn author_separator_macro_lastand() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_separator_macro_lastand.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able<sup xml:id=\"id1\">1</sup></personname>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker<sup xml:id=\"id2\">2</sup></personname>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cat Cole<sup xml:id=\"id3\">1</sup></personname>\n  </creator>",
  ]);
}

/// 64b: `~\&~` separates names (2601.13599).
/// Repro sectioning-frontmatter/author_tied_ampersand.
#[test]
fn author_tied_ampersand() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_tied_ampersand.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" xml:id=\"id1\">Lead</note>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Cat Cole</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
  ]);
}

/// 64b: a names line opening with a declaration and "and" (`\\\sc and Cat Cole`, 2411.12606). (Bob Baker's small caps
/// from that `\sc` are not kept on his name — a font loss older than this batch.)
/// Repro sectioning-frontmatter/author_declaration_line_opens_with_and.
#[test]
fn author_declaration_line_opens_with_and() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_declaration_line_opens_with_and.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname><text font=\"smallcaps\" xml:id=\"id1\">Ann Able</text></personname>\n    <contact name=\"Note:\u{a0}\" role=\"note\"><text font=\"smallcaps\" xml:id=\"id2\">Univ A</text></contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker<note mark=\"1\" role=\"footnotemark\" xml:id=\"footnotex1\"><tags>\n          <tag>1</tag>\n          <tag role=\"refnum\">1</tag>\n          <tag role=\"typerefnum\">footnote 1</tag>\n        </tags></note></personname>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname><text font=\"smallcaps\" xml:id=\"id3\">Cat Cole</text></personname>\n    <contact name=\"Note:\u{a0}\" role=\"note\"><text font=\"smallcaps\" xml:id=\"id4\">Univ B</text></contact>\n  </creator>",
  ]);
}

/// 64b: a list's closing period before a wrapper's mark (2201.07394); the period itself is trimmed off the last name as
/// any name's trailing punctuation is (63z8 the same).
/// Repro sectioning-frontmatter/author_wrapper_closing_period_before_mark.
#[test]
fn author_wrapper_closing_period_before_mark() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_wrapper_closing_period_before_mark.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>ANN ABLE</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>BOB BAKER</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>CAT COLE</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ B</contact>\n  </creator>",
  ]);
}

/// 64b: a name list closing with `;` (sn-jnl's author lists, 2504.09158; amsart's cautious path). The `;` the PDF
/// prints after "C. Cole" is trimmed with the list's closing punctuation (`strip_glued_marks`), as the closing period
/// is. Repro sectioning-frontmatter/amsart_name_list_closing_semicolon.
#[test]
fn amsart_name_list_closing_semicolon() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/amsart_name_list_closing_semicolon.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>A.\u{a0}Able</personname>\n  </creator>",
    "<creator before=\", \" role=\"author\">\n    <personname>B.\u{a0}Baker</personname>\n  </creator>",
    "<creator before=\" and \" role=\"author\">\n    <personname>C.\u{a0}Cole</personname>\n    <contact name=\"Address:\u{a0}\" role=\"address\">Univ A</contact>\n  </creator>",
  ]);
}

/// 64b review: an author-block delimiter is never rewritten as a separator macro — a class's printing
/// `\renewcommand{\and}{{\normalfont and}}` (ecca.cls:192) still parts its groups, each with its affiliation — and a
/// macro printing ordinary text (`\univ`) stays. Repro sectioning-frontmatter/author_redefined_and_still_parts_groups.
#[test]
fn author_redefined_and_still_parts_groups() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_redefined_and_still_parts_groups.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University of A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">University of B</contact>\n  </creator>",
  ]);
}

/// 64b review r2: a separator macro is written with the spacing its body prints and no more — `AT\amp T Labs` stays
/// "AT&T Labs", `NSF\comma NIH` "NSF,NIH" — while `\amp{}` between names, the source's spaces around it, still parts
/// them. Repro sectioning-frontmatter/author_separator_macro_prints_as_written.
#[test]
fn author_separator_macro_prints_as_written() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_separator_macro_prints_as_written.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <note class=\"ltx_note_frontmatter ltx_thanks_funding\" role=\"thanks\" xml:id=\"id1\">Supported by NSF,NIH.</note>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">AT&amp;T Labs</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">AT&amp;T Labs</contact>\n  </creator>",
  ]);
}

/// 64b review r3: a separator macro whose font is visible (`\textbf{\&}`) is the separator's look, kept as written —
/// the affiliation keeps its bold "&". Repro sectioning-frontmatter/author_separator_macro_visible_font_kept.
#[test]
fn author_separator_macro_visible_font_kept() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_separator_macro_visible_font_kept.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">AT<text font=\"bold\" xml:id=\"id1\">&amp;</text>T Labs</contact>\n  </creator>",
  ]);
}

/// 64b review r3: a separator macro spaced by stretchable glue (`,\hskip 1em plus 1fil`) is read past its whole glue
/// specification, so the comma parts the names. Repro sectioning-frontmatter/author_separator_macro_glue.
#[test]
fn author_separator_macro_glue() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_separator_macro_glue.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Ann Able</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
    "<creator before=\"\u{2003}\u{2003}\" role=\"author\">\n    <personname>Bob Baker</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Univ A</contact>\n  </creator>",
  ]);
}

/// The figure the 64c graphics repros include: a 10bp square as EPS text (repros graphics-tikz/figs/a.eps).
const SQUARE_EPS: &str = "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 10 10\nnewpath 0 0 moveto 10 0 lineto 10 10 lineto 0 10 \
                          lineto closepath fill\nshowpage\n%%EOF\n";

/// The XML of `tex` converted under ar5iv with `files` beside it, which must give no error and no warning.
fn convert_clean_files(tex: &str, files: &[(&str, &str)]) -> String {
  let (log, xml) = latexml::util::test::convert_files_with(tex, files, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  xml
}

/// 64c (HF10 F1): `\graphicspath{{nonexist/}{figs/}}` searches each directory — the DirectoryList's entries no longer
/// run together into "nonexist/figs/" in the digested argument (2608.21961, 2403.18830) — and `\Ginput@path` keeps
/// them braced, as graphics.sty:156 defines it. Repro graphics-tikz/graphicspath_two_directories.
#[test]
fn graphicspath_two_directories() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/graphicspath_two_directories.tex"
  );
  let xml = convert_clean_files(tex, &[("figs/a.eps", SQUARE_EPS)]);
  assert_element(
    &xml,
    "graphics",
    &["graphic=\"a\""],
    "<graphics candidates=\"figs/a.eps\" graphic=\"a\" options=\"width=85.35826pt,keepaspectratio=true\" xml:id=\"p1.g1\"/>",
  );
  assert_element(
    &xml,
    "text",
    &["font=\"typewriter\""],
    "<text font=\"typewriter\" xml:id=\"p1.1.1\">macro:-&gt;{nonexist/}{figs/}</text>",
  );
}

/// 64c (HF10 F2): a graphic named with dots (`b.v0.5_x`) is found with its extension, as Perl's `pathname_findall`
/// reads the whole name (2304.13099, 2608.12208); a name finds only itself plus one extension (`a.old.eps` is not
/// `a`). Repro graphics-tikz/graphic_name_with_dots.
#[test]
fn graphic_name_with_dots() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/graphic_name_with_dots.tex");
  let xml = convert_clean_files(tex, &[
    ("b.v0.5_x.eps", SQUARE_EPS),
    ("b.v0.5_x.old.eps", SQUARE_EPS),
  ]);
  assert_element(
    &xml,
    "graphics",
    &["graphic=\"b.v0.5_x\""],
    "<graphics candidates=\"b.v0.5_x.eps\" graphic=\"b.v0.5_x\" options=\"width=85.35826pt,keepaspectratio=true\" xml:id=\"p1.g1\"/>",
  );
}

/// 64c (HF10 F4): adjustbox's `\adjustimage`/`\adjincludegraphics` call `\Gin@i`, the body of `\includegraphics`
/// (adjustbox.sty:275-280) — no longer read as nothing (2410.09019) — through graphicx's dispatch, so the graphic is
/// sized as `\adjustbox{…}{\includegraphics{…}}` sizes it, not as its file name's text (64c review r1).
/// Repro graphics-tikz/adjustimage_includes_the_graphic.
#[test]
fn adjustimage_includes_the_graphic() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/adjustimage_includes_the_graphic.tex"
  );
  let xml = convert_clean_files(tex, &[("figs/a.eps", SQUARE_EPS)]);
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\"><inline-block depth=\"0.0pt\" height=\"7.2pt\" width=\"7.2pt\" xml:id=\"p1.1\" xscale=\"1\" xtranslate=\"0.0pt\" yscale=\"1\" ytranslate=\"0.0pt\"><p xml:id=\"p1.1.1\"><text xml:id=\"p1.1.1.1\"><graphics candidates=\"figs/a.eps\" cssstyle=\"width:1.004em; height:1.004em\" graphic=\"figs/a.eps\" xml:id=\"p1.g1\"/></text></p></inline-block><inline-block depth=\"0.0pt\" height=\"7.2pt\" width=\"7.2pt\" xml:id=\"p1.2\" xscale=\"1\" xtranslate=\"0.0pt\" yscale=\"1\" ytranslate=\"0.0pt\"><p xml:id=\"p1.2.1\"><text xml:id=\"p1.2.1.1\"><graphics candidates=\"figs/a.eps\" cssstyle=\"width:1.004em; height:1.004em\" graphic=\"figs/a.eps\" xml:id=\"p1.g2\"/></text></p></inline-block></para>",
  );
}

/// 64c (HF10 F5): cuted's `{strip}` keeps its content where it is written (cuted.sty set it in a box only the output
/// routine ships; 2508.07251's teaser figure was lost), a vertical block in which `\captionof` takes the graphic beside
/// it into its figure (64c review r1). Repro captions-floats/cuted_strip_keeps_content.
#[test]
fn cuted_strip_keeps_content() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/cuted_strip_keeps_content.tex"
  );
  let xml = convert_clean_files(tex, &[("figs/a.eps", SQUARE_EPS)]);
  assert_element(
    &xml,
    "figure",
    &["inlist=\"lof\""],
    "<figure align=\"center\" inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><graphics candidates=\"figs/a.eps\" class=\"ltx_centering\" graphic=\"figs/a.eps\" options=\"width=85.35826pt,keepaspectratio=true\" xml:id=\"g1\"/><toccaption><tag close=\" \">1</tag>Teaser caption.</toccaption><caption><tag close=\": \">Figure 1</tag>Teaser caption.</caption></figure>",
  );
}

/// 64c (HF10 F7): AASTeX 7's `\email[show]{…}` takes its option (aastex701.cls:13341), so the address is the email and
/// "[show]" no part of it (2608.21320). Repro sectioning-frontmatter/aastex7_email_show.
#[test]
fn aastex7_email_show() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/aastex7_email_show.tex"
  );
  assert_eq!(creators(&convert_clean(tex)), vec![
    "<creator role=\"author\">\n    <personname>Jane Doe</personname>\n    <contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">Some University</contact>\n    <contact name=\"Email:\u{a0}\" role=\"email\">jane@example.org</contact>\n  </creator>",
  ]);
}

/// 64c review r1: a `{strip}` is a vertical block — its paragraphs end at its edges, the raw `\strip … \endstrip` of a
/// `\newenvironment{widetext}{\strip}{\endstrip}` too. Repro captions-floats/cuted_strip_paragraphs_end_at_its_edges.
#[test]
fn cuted_strip_paragraphs_end_at_its_edges() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/cuted_strip_paragraphs_end_at_its_edges.tex"
  );
  let xml = convert_clean(tex);
  assert_element(
    &xml,
    "para",
    &["xml:id=\"p1\""],
    "<para xml:id=\"p1\"><p xml:id=\"p1.1\">Lead text.</p><p xml:id=\"p1.2\">Para one.</p><p xml:id=\"p1.3\">Para two.</p><p xml:id=\"p1.4\">Tail.</p><p xml:id=\"p1.5\">Wide text.</p><p xml:id=\"p1.6\">After.</p></para>",
  );
}

/// 64c review r1/r2: a name with a graphics extension that names a file is that file alone — `\includegraphics{c.eps}`
/// takes `c.eps` in pdflatex, and a `c.eps.png` listed beside it was what post rendered. Repro
/// graphics-tikz/graphic_exact_name_first.
#[test]
fn graphic_exact_name_first() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/graphics-tikz/graphic_exact_name_first.tex");
  let xml = convert_clean_files(tex, &[
    ("figs/c.eps.png", "not a png"),
    ("figs/c.eps", SQUARE_EPS),
  ]);
  assert_element(
    &xml,
    "graphics",
    &["graphic=\"figs/c.eps\""],
    "<graphics candidates=\"figs/c.eps\" graphic=\"figs/c.eps\" options=\"width=85.35826pt,keepaspectratio=true\" xml:id=\"p1.g1\"/>",
  );
}

/// 64c: a name with a leading `/` that names no file is read under each `\graphicspath` directory, as TeX glues the
/// entry to the name (`figs//a.eps`; 2409.13454's 42 graphics). Repro graphics-tikz/graphicspath_leading_slash_name.
#[test]
fn graphicspath_leading_slash_name() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/graphicspath_leading_slash_name.tex"
  );
  let xml = convert_clean_files(tex, &[("figs/a.eps", SQUARE_EPS)]);
  assert_element(
    &xml,
    "graphics",
    &["graphic=\"/a.eps\""],
    "<graphics candidates=\"figs/a.eps\" graphic=\"/a.eps\" options=\"width=85.35826pt,keepaspectratio=true\" xml:id=\"p1.g1\"/>",
  );
}

/// 64c review r2: the exact name wins over every search directory — `\Gin@getbase` tries `c.eps` over each
/// `\input@path` entry before appending an extension (graphics.sty:211-231), so `figs/c.eps` and not an earlier
/// directory's `other/c.eps.png`. Repro graphics-tikz/graphic_exact_name_over_other_directories.
#[test]
fn graphic_exact_name_over_other_directories() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/graphic_exact_name_over_other_directories.tex"
  );
  let xml = convert_clean_files(tex, &[
    ("other/c.eps.png", "not a png"),
    ("figs/c.eps", SQUARE_EPS),
  ]);
  assert_element(
    &xml,
    "graphics",
    &["graphic=\"c.eps\""],
    "<graphics candidates=\"figs/c.eps\" graphic=\"c.eps\" options=\"width=85.35826pt,keepaspectratio=true\" xml:id=\"p1.g1\"/>",
  );
}

/// 64c review r3: a name with no graphics extension — none, or one no graphics rule reads (`b.v0.5_x`) — lists its
/// `name.ext` files before a bare file of that name, which pdflatex takes only when nothing extended exists
/// (graphics.sty:205-231). Repro graphics-tikz/graphic_bare_name_listed_last.
#[test]
fn graphic_bare_name_listed_last() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/graphics-tikz/graphic_bare_name_listed_last.tex"
  );
  let xml = convert_clean_files(tex, &[
    ("bare/b.v0.5_x.eps", SQUARE_EPS),
    ("bare/b.v0.5_x", SQUARE_EPS),
    ("bare/fig.eps", SQUARE_EPS),
    ("bare/fig", SQUARE_EPS),
  ]);
  assert_element(
    &xml,
    "graphics",
    &["graphic=\"bare/b.v0.5_x\""],
    "<graphics candidates=\"bare/b.v0.5_x.eps,bare/b.v0.5_x\" graphic=\"bare/b.v0.5_x\" options=\"width=85.35826pt,keepaspectratio=true\" xml:id=\"p1.g1\"/>",
  );
  assert_element(
    &xml,
    "graphics",
    &["graphic=\"bare/fig\""],
    "<graphics candidates=\"bare/fig.eps,bare/fig\" graphic=\"bare/fig\" options=\"width=85.35826pt,keepaspectratio=true\" xml:id=\"p1.g2\"/>",
  );
}

/// 64e (HF10 F3): AASTeX's `\gridline` is a row of `\fig` panels (aastex701.cls:12314-12342) — each a child figure
/// with its graphic and sub-caption, a break between rows — no longer read as nothing (2609.21324, 2512.02147).
/// Repro captions-floats/aastex_gridline_panels.
#[test]
fn aastex_gridline_panels() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/captions-floats/aastex_gridline_panels.tex");
  let xml = convert_clean_files(tex, &[("figs/a.eps", SQUARE_EPS)]);
  assert_element(
    &xml,
    "figure",
    &["labels=\"LABEL:f:grid\""],
    "<figure inlist=\"lof\" labels=\"LABEL:f:grid\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"autoref\">Figure\u{a0}1<text xml:id=\"S0.F1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><figure class=\"ltx_figure_panel\" xml:id=\"S0.F1.fig1\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=137.9979pt,keepaspectratio=true\" xml:id=\"S0.F1.g1\"/><caption fontsize=\"80%\">(a) left</caption></figure><figure class=\"ltx_figure_panel\" xml:id=\"S0.F1.fig2\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=137.9979pt,keepaspectratio=true\" xml:id=\"S0.F1.g2\"/><caption fontsize=\"80%\">(b) right</caption></figure><break class=\"ltx_break\"/><figure class=\"ltx_figure_panel\" xml:id=\"S0.F1.fig3\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=103.50105pt,keepaspectratio=true\" xml:id=\"S0.F1.g3\"/><caption>(c)</caption></figure><figure class=\"ltx_figure_panel\" xml:id=\"S0.F1.fig4\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=103.50105pt,keepaspectratio=true\" xml:id=\"S0.F1.g4\"/><caption>(d)</caption></figure><toccaption><tag close=\" \">1</tag>Four panels in two rows.</toccaption><caption><tag close=\": \">Figure 1</tag>Four panels in two rows.</caption></figure>",
  );
}

/// 64e (HF10 F3): a panel with an empty sub-caption has none, and `\boxedfig` is framed (2505.20669).
/// Repro captions-floats/aastex_gridline_empty_caption_boxed.
#[test]
fn aastex_gridline_empty_caption_boxed() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/aastex_gridline_empty_caption_boxed.tex"
  );
  let xml = convert_clean_files(tex, &[("figs/a.eps", SQUARE_EPS)]);
  assert_element(
    &xml,
    "figure",
    &["inlist=\"lof\""],
    "<figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"autoref\">Figure\u{a0}1<text xml:id=\"S0.F1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><figure class=\"ltx_figure_panel\" xml:id=\"S0.F1.fig1\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=207.0021pt,keepaspectratio=true\" xml:id=\"S0.F1.g1\"/></figure><figure class=\"ltx_figure_panel\" framed=\"rectangle\" xml:id=\"S0.F1.fig2\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=103.50105pt,keepaspectratio=true\" xml:id=\"S0.F1.g2\"/></figure><toccaption><tag close=\" \">1</tag>Left: one map. Right: another.</toccaption><caption><tag close=\": \">Figure 1</tag>Left: one map. Right: another.</caption></figure>",
  );
}

/// 64e (HF10 F3): a bare `\fig` in a float is the class's unnumbered panel — it stepped the figure counter as a
/// figure of its own, the float numbered "Figure 3" where the PDF prints 1 (2103.00666).
/// Repro captions-floats/aastex_bare_fig_panels_keep_numbering.
#[test]
fn aastex_bare_fig_panels_keep_numbering() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/aastex_bare_fig_panels_keep_numbering.tex"
  );
  let xml = convert_clean_files(tex, &[("figs/a.eps", SQUARE_EPS)]);
  assert_element(
    &xml,
    "figure",
    &["labels=\"LABEL:f:x\""],
    "<figure inlist=\"lof\" labels=\"LABEL:f:x\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"autoref\">Figure\u{a0}1<text xml:id=\"S0.F1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><figure align=\"center\" class=\"ltx_figure_panel\" xml:id=\"S0.F1.fig1\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=82.8019pt,keepaspectratio=true\" xml:id=\"S0.F1.g1\"/><caption fontsize=\"80%\">(a1)</caption></figure><figure align=\"center\" class=\"ltx_figure_panel\" xml:id=\"S0.F1.fig2\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=82.8019pt,keepaspectratio=true\" xml:id=\"S0.F1.g2\"/><caption fontsize=\"80%\">(a2)</caption></figure><toccaption class=\"ltx_centering\"><tag close=\" \">1</tag>Panel (a1) and (a2).</toccaption><caption class=\"ltx_centering\"><tag close=\": \">Figure 1</tag>Panel (a1) and (a2).</caption></figure>",
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F2\""],
    "<figure inlist=\"lof\" xml:id=\"S0.F2\"><tags><tag>Figure 2</tag><tag role=\"autoref\">Figure\u{a0}2<text xml:id=\"S0.F2.1\"/></tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Figure 2</tag></tags><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=345.0pt,keepaspectratio=true\" xml:id=\"S0.F2.g1\"/><toccaption><tag close=\" \">2</tag>Next.</toccaption><caption><tag close=\": \">Figure 2</tag>Next.</caption></figure>",
  );
}

/// 64e review r1: each `\gridline` stays one row — inside `{center}` (2103.16579) and when its panels add up wider than
/// the float (2609.09897). Repro captions-floats/aastex_gridline_rows_kept.
#[test]
fn aastex_gridline_rows_kept() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/aastex_gridline_rows_kept.tex"
  );
  let xml = convert_clean_files(tex, &[("figs/a.eps", SQUARE_EPS)]);
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    "<figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"autoref\">Figure\u{a0}1<text xml:id=\"S0.F1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><figure align=\"center\" class=\"ltx_figure_panel\" xml:id=\"S0.F1.fig1\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=103.50105pt,keepaspectratio=true\" xml:id=\"S0.F1.g1\"/><caption fontsize=\"80%\">(a)</caption></figure><figure align=\"center\" class=\"ltx_figure_panel\" xml:id=\"S0.F1.fig2\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=103.50105pt,keepaspectratio=true\" xml:id=\"S0.F1.g2\"/><caption fontsize=\"80%\">(b)</caption></figure><break class=\"ltx_break ltx_centering\"/><figure align=\"center\" class=\"ltx_figure_panel\" xml:id=\"S0.F1.fig3\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=103.50105pt,keepaspectratio=true\" xml:id=\"S0.F1.g3\"/><caption fontsize=\"80%\">(c)</caption></figure><figure align=\"center\" class=\"ltx_figure_panel\" xml:id=\"S0.F1.fig4\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=103.50105pt,keepaspectratio=true\" xml:id=\"S0.F1.g4\"/><caption fontsize=\"80%\">(d)</caption></figure><toccaption><tag close=\" \">1</tag>Centered rows.</toccaption><caption><tag close=\": \">Figure 1</tag>Centered rows.</caption></figure>",
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F2\""],
    "<figure inlist=\"lof\" xml:id=\"S0.F2\"><tags><tag>Figure 2</tag><tag role=\"autoref\">Figure\u{a0}2<text xml:id=\"S0.F2.1\"/></tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Figure 2</tag></tags><figure class=\"ltx_figure_panel\" xml:id=\"S0.F2.fig1\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=182.84958pt,keepaspectratio=true\" xml:id=\"S0.F2.g1\"/><caption fontsize=\"80%\">(e)</caption></figure><figure class=\"ltx_figure_panel\" xml:id=\"S0.F2.fig2\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=182.84958pt,keepaspectratio=true\" xml:id=\"S0.F2.g2\"/><caption fontsize=\"80%\">(f)</caption></figure><break class=\"ltx_break\"/><figure class=\"ltx_figure_panel\" xml:id=\"S0.F2.fig3\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=137.9979pt,keepaspectratio=true\" xml:id=\"S0.F2.g3\"/><caption fontsize=\"80%\">(g)</caption></figure><toccaption><tag close=\" \">2</tag>A wide row.</toccaption><caption><tag close=\": \">Figure 2</tag>A wide row.</caption></figure>",
  );
}

/// 64e review r1: `\rotatefig` (a decimal angle) and a bare `\boxedfig` in a float are panels, and a braced `\fig`
/// outside a float is the class's unnumbered panel, not a numbered figure (no AAS class has one), so the next float is
/// Figure 2. Repro captions-floats/aastex_rotatefig_boxedfig_and_outside_fig.
#[test]
fn aastex_rotatefig_boxedfig_and_outside_fig() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/aastex_rotatefig_boxedfig_and_outside_fig.tex"
  );
  let xml = convert_clean_files(tex, &[("figs/a.eps", SQUARE_EPS)]);
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F1\""],
    "<figure inlist=\"lof\" xml:id=\"S0.F1\"><tags><tag>Figure 1</tag><tag role=\"autoref\">Figure\u{a0}1<text xml:id=\"S0.F1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><figure class=\"ltx_figure_panel\" xml:id=\"S0.F1.fig1\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=103.50105pt,angle=22.5,keepaspectratio=true\" xml:id=\"S0.F1.g1\"/><caption fontsize=\"80%\">(r)</caption></figure><figure class=\"ltx_figure_panel\" framed=\"rectangle\" xml:id=\"S0.F1.fig2\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=103.50105pt,keepaspectratio=true\" xml:id=\"S0.F1.g2\"/><caption>(s)</caption></figure><toccaption><tag close=\" \">1</tag>Rotated and boxed.</toccaption><caption><tag close=\": \">Figure 1</tag>Rotated and boxed.</caption></figure>",
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"S0.F2\""],
    "<figure inlist=\"lof\" xml:id=\"S0.F2\"><tags><tag>Figure 2</tag><tag role=\"autoref\">Figure\u{a0}2<text xml:id=\"S0.F2.1\"/></tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Figure 2</tag></tags><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=345.0pt,keepaspectratio=true\" xml:id=\"S0.F2.g1\"/><toccaption><tag close=\" \">2</tag>Next.</toccaption><caption><tag close=\": \">Figure 2</tag>Next.</caption></figure>",
  );
  assert_element(
    &xml,
    "figure",
    &["xml:id=\"fig1\""],
    "<figure xml:id=\"fig1\"><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=103.50105pt,keepaspectratio=true\" xml:id=\"g1\"/><caption fontsize=\"80%\">(o) outside</caption></figure>",
  );
}

/// 64e: AASTeX 5.x's `{plate}` lists with the figures (aastex.cls:1685-1704 `\def\ext@plate{lof}`) — the family binding
/// serves every version, so the plate keeps working though aastex 6+ dropped it (`\ext@plate` was undefined).
/// Repro captions-floats/aastex5_plate_float.
#[test]
fn aastex5_plate_float() {
  let tex =
    include_str!("../../../tools/perfect_kernel/repros/captions-floats/aastex5_plate_float.tex");
  let xml = convert_clean_files(tex, &[("figs/a.eps", SQUARE_EPS)]);
  assert_element(
    &xml,
    "float",
    &["class=\"ltx_float_plate\""],
    "<float class=\"ltx_float_plate\" inlist=\"lof\" xml:id=\"plate1\"><tags><tag>Plate 1</tag><tag role=\"autoref\">Plate\u{a0}1<text xml:id=\"plate1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Plate 1</tag></tags><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=345.0pt,keepaspectratio=true\" xml:id=\"plate1.g1\"/><toccaption><tag close=\" \">1</tag>A plate.</toccaption><caption><tag close=\" \">Plate 1</tag>A plate.</caption></float>",
  );
}

/// 64f: every AASTeX numbers its sections in arabic (aastex.cls 5.x:1076, aastex701.cls:7629) — the binding's revtex4
/// load made them Roman, "I", "I.1", in every AAS paper (Perl the same, KPE #557).
/// Repro sectioning-frontmatter/aastex_sections_arabic.
#[test]
fn aastex_sections_arabic() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/aastex_sections_arabic.tex"
  );
  let xml = convert_clean(tex);
  assert_element(
    &xml,
    "section",
    &["labels=\"LABEL:s:i\""],
    "<section inlist=\"toc\" labels=\"LABEL:s:i\" xml:id=\"S1\"><tags><tag>1</tag><tag role=\"autoref\">section\u{a0}1<text xml:id=\"S1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">\u{a7}1</tag></tags><title><tag close=\" \">1</tag>Introduction</title><subsection inlist=\"toc\" labels=\"LABEL:s:d\" xml:id=\"S1.SS1\"><tags><tag>1.1</tag><tag role=\"autoref\">subsection\u{a0}1.1<text xml:id=\"S1.SS1.1\"/></tag><tag role=\"refnum\">1.1</tag><tag role=\"typerefnum\">\u{a7}1.1</tag></tags><title><tag close=\" \">1.1</tag>Data</title><para xml:id=\"S1.SS1.p1\"><p xml:id=\"S1.SS1.p1.1\">See Section\u{a0}<ref labelref=\"LABEL:s:i\"/> and <ref labelref=\"LABEL:s:d\"/>.</p></para></subsection></section>",
  );
}

/// 64f: under AASTeX 5.x, which has no `\fig`, an author's own `\newcommand{\fig}` is the paper's — the family binding
/// defines the 6+ `\fig` family and `\gridline` only for 6+ (one binding for every version; 64e review r2).
/// Repro captions-floats/aastex5_author_fig_command.
#[test]
fn aastex5_author_fig_command() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/aastex5_author_fig_command.tex"
  );
  let xml = convert_clean_files(tex, &[("figs/a.eps", SQUARE_EPS)]);
  assert_element(
    &xml,
    "figure",
    &["labels=\"LABEL:f2\""],
    "<figure inlist=\"lof\" labels=\"LABEL:f2\" xml:id=\"S0.F2\"><tags><tag>Figure 2</tag><tag role=\"autoref\">Figure\u{a0}2<text xml:id=\"S0.F2.1\"/></tag><tag role=\"refnum\">2</tag><tag role=\"typerefnum\">Figure 2</tag></tags><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=345.0pt,keepaspectratio=true\" xml:id=\"S0.F2.g1\"/><toccaption><tag close=\" \">2</tag>Same as Fig.\u{a0}<ref labelref=\"LABEL:f1\"/><text font=\"italic\" xml:id=\"S0.F2.2\">top</text>.</toccaption><caption><tag close=\": \">Figure 2</tag>Same as Fig.\u{a0}<ref labelref=\"LABEL:f1\"/><text font=\"italic\" xml:id=\"S0.F2.3\">top</text>.</caption></figure>",
  );
}

/// 64f: AASTeX 5.x commands the later classes dropped — `\subsubsubsection` a level-4 heading, `\supportfrom` its text,
/// `\platenum` stepping the counter back (aastex.cls 5.2:1088, :1988, :1691). Repro captions-floats/aastex5_dropped_commands.
/// The plate ids come from the counter, Perl's `plateN` shape: `\platenum` steps it back to 0 before the caption steps it.
#[test]
fn aastex5_dropped_commands() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/aastex5_dropped_commands.tex"
  );
  let xml = convert_clean_files(tex, &[("figs/a.eps", SQUARE_EPS)]);
  assert_element(
    &xml,
    "section",
    &["xml:id=\"S1\""],
    "<section inlist=\"toc\" xml:id=\"S1\"><tags><tag>1</tag><tag role=\"autoref\">section\u{a0}1<text xml:id=\"S1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">\u{a7}1</tag></tags><title><tag close=\" \">1</tag>Intro</title><paragraph inlist=\"toc\" xml:id=\"S1.SS0.SSS0.Px1\"><title>Deep</title><para xml:id=\"S1.SS0.SSS0.Px1.p1\"><p xml:id=\"S1.SS0.SSS0.Px1.p1.1\">Supported by NSF grant.</p></para><float class=\"ltx_float_plate\" inlist=\"lof\" xml:id=\"plate0\"><tags><tag>Plate 5</tag><tag role=\"autoref\">Plate\u{a0}5<text xml:id=\"plate0.1\"/></tag><tag role=\"refnum\">5</tag><tag role=\"typerefnum\">Plate 5</tag></tags><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=345.0pt,keepaspectratio=true\" xml:id=\"plate0.g1\"/><toccaption><tag close=\" \">5</tag>First.</toccaption><caption><tag close=\" \">Plate 5</tag>First.</caption></float><float class=\"ltx_float_plate\" inlist=\"lof\" xml:id=\"plate1\"><tags><tag>Plate 1</tag><tag role=\"autoref\">Plate\u{a0}1<text xml:id=\"plate1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Plate 1</tag></tags><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=345.0pt,keepaspectratio=true\" xml:id=\"plate1.g1\"/><toccaption><tag close=\" \">1</tag>Second.</toccaption><caption><tag close=\" \">Plate 1</tag>Second.</caption></float></paragraph></section>",
  );
}

/// 64f review r1: `\subsubsubsection`/`\supportfrom` are AASTeX 5.x only; in a 6+ paper they are the paper's own, which the
/// family binding leaves undefined for it. Repro sectioning-frontmatter/aastex6_author_subsubsubsection.
#[test]
fn aastex6_author_subsubsubsection() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/aastex6_author_subsubsubsection.tex"
  );
  let xml = convert_clean(tex);
  assert_element(
    &xml,
    "section",
    &["xml:id=\"S1\""],
    "<section inlist=\"toc\" xml:id=\"S1\"><tags><tag>1</tag><tag role=\"autoref\">section\u{a0}1<text xml:id=\"S1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">\u{a7}1</tag></tags><title><tag close=\" \">1</tag>Intro</title><para xml:id=\"S1.p1\"><p xml:id=\"S1.p1.1\"><text font=\"bold\" xml:id=\"S1.p1.1.1\">Deep:</text>\u{a0} Text here. Support: NSF.</p></para></section>",
  );
}

/// 64f review r1-r2: emulateapj loads the aastex binding but is a revtex4 class of its own (emulateapj.cls:165-167) — the
/// author's `\fig`, `\subsubsubsection` and `\supportfrom` are theirs, and the sections are arabic (:676; astro-ph/0503342).
/// Repro captions-floats/emulateapj_author_fig_sections.
#[test]
fn emulateapj_author_fig_sections() {
  let tex = include_str!(
    "../../../tools/perfect_kernel/repros/captions-floats/emulateapj_author_fig_sections.tex"
  );
  let xml = convert_clean_files(tex, &[("figs/a.eps", SQUARE_EPS)]);
  assert_element(
    &xml,
    "section",
    &["xml:id=\"S1\""],
    "<section inlist=\"toc\" labels=\"LABEL:s:i\" xml:id=\"S1\"><tags><tag>1</tag><tag role=\"autoref\">section\u{a0}1<text xml:id=\"S1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">\u{a7}1</tag></tags><title><tag close=\". \">1</tag>Intro</title><toctitle><tag close=\" \">1</tag>Intro</toctitle><figure inlist=\"lof\" labels=\"LABEL:f1\" xml:id=\"S1.F1\"><tags><tag>Figure 1</tag><tag role=\"autoref\">Figure\u{a0}1<text xml:id=\"S1.F1.1\"/></tag><tag role=\"refnum\">1</tag><tag role=\"typerefnum\">Figure 1</tag></tags><graphics candidates=\"figs/a.eps\" graphic=\"figs/a.eps\" options=\"width=345.0pt,keepaspectratio=true\" xml:id=\"S1.F1.g1\"/><toccaption><tag close=\" \">1</tag>One.</toccaption><caption><tag close=\".\u{2014} \">Figure 1</tag>One.</caption></figure><para xml:id=\"S1.p1\"><p xml:id=\"S1.p1.1\">See Fig.\u{a0}<ref labelref=\"LABEL:f1\"/> and Section\u{a0}<ref labelref=\"LABEL:s:i\"/>.</p></para><subsection inlist=\"toc\" labels=\"LABEL:s:m\" xml:id=\"S1.SS1\"><tags><tag>1.1</tag><tag role=\"autoref\">subsection\u{a0}1.1<text xml:id=\"S1.SS1.1\"/></tag><tag role=\"refnum\">1.1</tag><tag role=\"typerefnum\">\u{a7}1.1</tag></tags><title><tag close=\". \">1.1</tag>More</title><toctitle><tag close=\" \">1.1</tag>More</toctitle><para xml:id=\"S1.SS1.p1\"><p xml:id=\"S1.SS1.p1.1\">In <ref labelref=\"LABEL:s:m\"/>.<text font=\"bold\" xml:id=\"S1.SS1.p1.1.1\">Deep:</text>\u{a0} Text here. Support: NSF.</p></para></subsection></section>",
  );
}
