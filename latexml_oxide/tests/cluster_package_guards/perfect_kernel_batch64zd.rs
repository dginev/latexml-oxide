//! Red/green guards for batch 64zd: a `\documentstyle` option that is neither declared nor a style file makes
//! `\compat@loadpackages` load OmniBus after the class (Perl latex_constructs.pool.ltxml:137-153). Two fixes. REVTeX 3's
//! own options (revtex.sty v3.0/v3.1 `\ds@<option>`) and its distribution's option files (`version2`, `prbbib`, …) are
//! declared by revtex3_support, so a REVTeX 3 paper is not missing them (user ruling 2026-10-10). And OmniBus, when a
//! missing option does load it, loads provide-only: it defines what is still undefined and replaces nothing the class
//! defines (`ProvideOnlyGuard`, user ruling 2026-10-11). Both keep REVTeX's author–address grouping, which OmniBus's
//! `\author` broke (hep-th9610203, cond-mat9403039). Repros `tools/perfect_kernel/repros/sectioning-frontmatter/`.
use latexml::util::test::assert_element;

use super::perfect_kernel_batch46::{error_count, warning_count};

/// The log and XML of `tex` converted under ar5iv, which must give no error and no warning.
fn convert_clean(tex: &str) -> (String, String) {
  let (log, xml) = latexml::util::test::convert_with(tex, Some("ar5iv.sty"));
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warning_count(&log), 0, "{log}");
  (log, xml)
}

/// Every `<creator>` element of `xml`, whole, with the indentation between its tags removed.
fn creators(xml: &str) -> Vec<String> {
  xml
    .split("<creator ")
    .skip(1)
    .map(|rest| {
      let element = &rest[..rest.find("</creator>").expect("closed creator") + "</creator>".len()];
      let lines: Vec<&str> = element.lines().map(str::trim).collect();
      format!("<creator {}", lines.concat())
    })
    .collect()
}

fn author_with(name: &str, contact: &str) -> String {
  format!(r#"<creator role="author"><personname>{name}</personname>{contact}</creator>"#)
}

fn address(text: &str) -> String {
  format!("<contact name=\"Address:\u{a0}\" role=\"address\">{text}</contact>")
}

fn affiliation(text: &str) -> String {
  format!("<contact name=\"Affiliation:\u{a0}\" role=\"affiliation\">{text}</contact>")
}

/// A creator after the first, set off by two em spaces.
fn later(creator: String) -> String {
  creator.replacen("<creator ", "<creator before=\"\u{2003}\u{2003}\" ", 1)
}

/// Whether the log reports OmniBus loaded for missing options, and how.
fn omnibus_note(log: &str) -> Option<&str> {
  log
    .lines()
    .find(|line| line.contains("OmniBus") && line.contains("missing options"))
}

/// `\documentstyle[aps]{revtex}` (hep-th9610203, cond-mat9403039): `aps` is REVTeX's, so no OmniBus, and the address
/// after `\author{Ann Able and Bob Baker}` is both authors'.
#[test]
fn revtex3_omnibus_author_group_address() {
  let (log, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex3_omnibus_author_group_address.tex"
  ));
  assert_eq!(omnibus_note(&log), None, "{log}");
  assert_eq!(
    creators(&xml),
    vec![
      author_with("Ann Able", &address("Univ X, City")),
      later(author_with("Bob Baker", &address("Univ X, City")))
    ],
    "{xml}"
  );
}

/// Two author groups under revtex.sty's own options `[aps,prl,preprint,eqsecnum,tighten,floats]`: each group with its
/// address, and `eqsecnum`'s equations numbered within sections (aps.sty:358-365), as pdflatex gives (1.1), (2.1).
#[test]
fn revtex3_real_options_two_groups_eqsecnum() {
  let (log, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex3_real_options_two_groups_eqsecnum.tex"
  ));
  assert_eq!(omnibus_note(&log), None, "{log}");
  assert_eq!(
    creators(&xml),
    vec![
      author_with("Ann Able", &address("Univ X, City")),
      later(author_with("Bob Baker", &address("Univ X, City"))),
      later(author_with("Cy Cole", &address("Univ Y, Town"))),
    ],
    "{xml}"
  );
  for (label, id, number, lhs, rhs) in [
    ("e1", "S1.E1", "1.1", "a", "b"),
    ("e2", "S2.E1", "2.1", "c", "d"),
  ] {
    assert_element(
      &xml,
      "equation",
      &[&format!(r#"xml:id="{id}""#)],
      &format!(
        "<equation labels=\"LABEL:{label}\" xml:id=\"{id}\">
          <tags>
            <tag>({number})</tag>
            <tag role=\"autoref\">Equation\u{a0}{number}<text xml:id=\"{id}.1\"/></tag>
            <tag role=\"refnum\">{number}</tag>
          </tags>
          <Math mode=\"display\" tex=\"{lhs}={rhs}\" text=\"{lhs} = {rhs}\" xml:id=\"{id}.m1\">
            <XMath xml:id=\"{id}.m1.1\">
              <XMApp xml:id=\"{id}.m1.1.1\">
                <XMTok meaning=\"equals\" role=\"RELOP\">=</XMTok>
                <XMTok font=\"italic\" role=\"UNKNOWN\">{lhs}</XMTok>
                <XMTok font=\"italic\" role=\"UNKNOWN\">{rhs}</XMTok>
              </XMApp>
            </XMath>
          </Math>
        </equation>"
      ),
    );
  }
}

/// REVTeX 2 under `[preprint,aps,version2]{revtex}` (hep-th9610203, cond-mat9403039): `version2` is a REVTeX 3
/// distribution file, so no OmniBus, and each `{instit}` is the affiliation of the whole author group before it.
#[test]
fn revtex2_version2_instit_author_group() {
  let (log, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex2_version2_instit_author_group.tex"
  ));
  assert_eq!(omnibus_note(&log), None, "{log}");
  assert_eq!(
    creators(&xml),
    vec![
      author_with("Ann Able", &affiliation("Univ X, City")),
      later(author_with("Bob Baker", &affiliation("Univ X, City"))),
      later(author_with("Cy Cole", &affiliation("Univ Y, Town"))),
    ],
    "{xml}"
  );
}

/// A bound class with an unknown option (`[aps,myjournalx]{revtex}`): OmniBus is loaded, provide-only, and the class's
/// `\author`/`\address` stay, so both author groups keep their addresses.
#[test]
fn omnibus_provide_only_keeps_bound_class_grouping() {
  let (log, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/omnibus_provide_only_keeps_bound_class_grouping.tex"
  ));
  assert!(
    omnibus_note(&log).is_some_and(|note| note.contains("provide-only")),
    "{log}"
  );
  assert_eq!(
    creators(&xml),
    vec![
      author_with("Ann Able", &address("Univ X, City")),
      later(author_with("Bob Baker", &address("Univ X, City"))),
      later(author_with("Cy Cole", &address("Univ Y, Town"))),
    ],
    "{xml}"
  );
}

/// `\documentstyle[myjournalx]{article}`: article has no `\address`, so provide-only OmniBus still gives it, the
/// recovery Perl's `\compat@loadpackages` is for.
#[test]
fn omnibus_provide_only_provides_missing_address() {
  let (log, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/omnibus_provide_only_provides_missing_address.tex"
  ));
  assert!(
    omnibus_note(&log).is_some_and(|note| note.contains("provide-only")),
    "{log}"
  );
  assert_eq!(
    creators(&xml),
    vec![author_with("Ann Able", &address("Univ X, City"))],
    "{xml}"
  );
}

/// An option revtex.sty does not declare that is a style file (`epsf`) still loads as a package; with `aps` declared
/// nothing is missing, so no OmniBus, and the address stays the author's.
#[test]
fn revtex3_package_option_still_loads() {
  let (log, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex3_package_option_still_loads.tex"
  ));
  assert_eq!(omnibus_note(&log), None, "{log}");
  assert_eq!(
    creators(&xml),
    vec![author_with("Ann Able", &address("Univ X, City"))],
    "{xml}"
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">Text 56.9055pt.</p></para>"#,
  );
}

/// epsfig's `\epsffile`/`\epsfbox` include the graphic at `\epsfxsize`, then reset it (epsfig.sty:61-100;
/// cond-mat9908394, whose `\epsffile` figures OmniBus's epsf used to supply).
#[test]
fn epsfig_epsffile_includes_the_graphic() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/graphics-axodraw/epsfig_epsffile_includes_the_graphic.tex"
  ));
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1">
      <graphics graphic="x.eps" options="width=56.9pt" xml:id="p1.g1"/>
      <graphics graphic="x.eps" xml:id="p1.g2"/>
    </para>"#,
  );
}

/// AASTeX's `{references}` keeps its title and cite style (aas_support.sty.ltxml:289-293; astro-ph9708112), which
/// OmniBus's `\references` used to supply over the binding's bare one.
#[test]
fn aas_references_environment_has_its_title() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/aas_references_environment_has_its_title.tex"
  ));
  assert_element(
    &xml,
    "bibliography",
    &[],
    r#"<bibliography citestyle="authoryear" inlist="toc" xml:id="bib">
      <title>References</title>
      <biblist>
        <bibitem key="a" xml:id="bib.bib1">
          <tags>
            <tag role="number">1</tag>
            <tag role="refnum">(1)</tag>
            <tag role="key">a</tag>
          </tags>
          <bibblock> Able, A. 1990, ApJ, 1, 1
</bibblock>
        </bibitem>
      </biblist>
    </bibliography>"#,
  );
}

/// OmniBus's `\bibitem` extends the class's (natbib for a `[\protect\citeauthoryear…]` label, else the saved
/// `\bibitem`), so provide-only OmniBus still installs it (cond-mat9808262).
#[test]
fn omnibus_citeauthoryear_bibitem_under_provide_only() {
  let (log, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/index-bib/omnibus_citeauthoryear_bibitem_under_provide_only.tex"
  ));
  assert!(
    omnibus_note(&log).is_some_and(|note| note.contains("provide-only")),
    "{log}"
  );
  assert_element(
    &xml,
    "tags",
    &[],
    r#"<tags>
      <tag role="number">1</tag>
      <tag role="year">1990</tag>
      <tag role="authors">Able</tag>
      <tag role="refnum">Able (1990)</tag>
      <tag role="key">a</tag>
    </tags>"#,
  );
  assert_element(
    &xml,
    "bibitem",
    &[r#"xml:id="bib.bib2""#],
    r#"<bibitem key="b" xml:id="bib.bib2">
      <tags>
        <tag role="number">2</tag>
        <tag role="year">1991</tag>
        <tag role="authors">Baker</tag>
        <tag role="refnum">Baker (1991)</tag>
        <tag role="key">b</tag>
      </tags>
      <bibblock> Baker, B. 1991, Journal.
</bibblock>
    </bibitem>"#,
  );
}

/// A REVTeX author list with marks splits into its names: a letter mark `$^{a}$`, a `\cite` note mark, a surname set
/// in math with its mark, a byline's "by" (cond-mat9402093, cond-mat9611093, hep-ph9507223, hep-ex9910016).
#[test]
fn revtex_marked_author_list_splits_each_name() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_marked_author_list_splits_each_name.tex"
  ));
  let creators = creators(&xml);
  assert_eq!(creators.len(), 4, "{xml}");
  assert_eq!(
    creators[..3],
    [
      author_with(
        r#"Ann Able<sup xml:id="id1"><text font="italic" xml:id="id1.1">a</text></sup>"#,
        &address("Univ X, City")
      ),
      later(author_with(
        r#"Bob Baker<cite class="ltx_citemacro_cite">[<bibref bibrefs="bb" separator="," show="Number" yyseparator=","/>]</cite>"#,
        &address("Univ X, City")
      )),
      later(author_with("Cy Cole", &address("Univ Y, Town"))),
    ],
    "{xml}"
  );
  assert_eq!(
    creators[3],
    later(author_with(
      "D. <Math mode=\"inline\" tex=\"{\\rm Dorn}^{*}\" text=\"Dorn ^ *\" xml:id=\"m2\"><XMath xml:id=\"m2.1\">\
       <XMApp xml:id=\"m2.1.1\"><XMTok role=\"SUPERSCRIPTOP\" scriptpos=\"post1\"/><XMTok role=\"UNKNOWN\">Dorn</XMTok>\
       <XMTok fontsize=\"70%\" meaning=\"times\" role=\"MULOP\">\u{2217}</XMTok></XMApp></XMath></Math>",
      &address("Univ Y, Town")
    )),
    "{xml}"
  );
}

/// REVTeX 3's numbering: sections roman, subsections lettered and referenced with their section ("II A"),
/// subsubsections arabic ("II A 1"); in the appendix subsections arabic ("A 1") and equations A1 (aps.sty:220-227,
/// 358-365; aps10.sty:178-186).
#[test]
fn revtex3_section_numbers_as_aps() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex3_section_numbers_as_aps.tex"
  ));
  assert_eq!(
    refnums(&xml),
    [
      "I",
      "II",
      "II\u{2009}A",
      "II\u{2009}A\u{2009}1",
      "A",
      "A\u{2009}1",
      "A1"
    ],
    "{xml}"
  );
}

/// Provide-only OmniBus declines what the class defines, also through the bindings it requires: under
/// `[aps,myjunkx]{revtex}` the live `\author` is the revtex binding's, not inst_support's (review 64zd round 1).
#[test]
fn omnibus_provide_only_keeps_bound_class_commands() {
  let (log, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/omnibus_provide_only_keeps_bound_class_commands.tex"
  ));
  assert!(
    omnibus_note(&log).is_some_and(|note| note.contains("provide-only")),
    "{log}"
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1"><text font="typewriter" xml:id="p1.1.1">macro:#1#2-&gt;\lx@add@authors@append [annotations={#1}]{#2}\lx@author@after {#2}</text></p></para>"#,
  );
}

/// elsart's labelled `\author[a]`/`\address[a]` under a missing option: each author with the address of its label
/// (inst_support's `\author`, required by OmniBus, had replaced elsart's).
#[test]
fn omnibus_provide_only_labelled_elsart_addresses() {
  let (log, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/omnibus_provide_only_labelled_elsart_addresses.tex"
  ));
  assert!(
    omnibus_note(&log).is_some_and(|note| note.contains("provide-only")),
    "{log}"
  );
  assert_eq!(
    creators(&xml),
    vec![
      author_with("Ann Able", &address("Univ X, City")),
      later(author_with("Bob Baker", &address("Univ Y, Town")))
    ],
    "{xml}"
  );
}

/// `\documentstyle[foobarx]{report}`: provide-only OmniBus does not load article over report, which numbers sections
/// and equations within chapters.
#[test]
fn omnibus_provide_only_keeps_report_numbering() {
  let (log, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/omnibus_provide_only_keeps_report_numbering.tex"
  ));
  assert!(
    omnibus_note(&log).is_some_and(|note| note.contains("provide-only")),
    "{log}"
  );
  assert_eq!(refnums(&xml), ["1", "1.1", "1.1"], "{xml}");
}

/// An unnumbered appendix steps the section counter (aps.sty:280-295 `\@ssect`): its equation is (A1), the next
/// appendix B with (B1) (cond-mat0006498).
#[test]
fn revtex3_unnumbered_appendix_steps_section() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex3_unnumbered_appendix_steps_section.tex"
  ));
  assert_eq!(
    refnums(&xml),
    ["I", "1.1", "1.1a", "1.1b", "1.2", "A1", "B", "B1"],
    "{xml}"
  );
}

/// Bare-asterisk marks after the names' commas (quant-ph0003094): three authors, each keeping its mark.
#[test]
fn revtex_asterisk_marked_authors_split() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/revtex_asterisk_marked_authors_split.tex"
  ));
  let address = address("Norman Bridge Laboratory, Pasadena");
  assert_eq!(
    creators(&xml),
    vec![
      author_with("S. F. Pereira*", &address),
      later(author_with("Z. Y. Ou**", &address)),
      later(author_with("H. J. Kimble", &address)),
    ],
    "{xml}"
  );
}

/// Every `<tag role="refnum">` text of `xml`, in order.
fn refnums(xml: &str) -> Vec<&str> {
  xml
    .split(r#"<tag role="refnum">"#)
    .skip(1)
    .map(|rest| &rest[..rest.find("</tag>").expect("closed tag")])
    .collect()
}

/// World Scientific's sprocl: each `\author` line appends, each `\address` goes to the authors given since the one
/// before (sprocl.sty:18-21; hep-ph9611287).
#[test]
fn ws_sprocl_author_groups() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ws_sprocl_author_groups.tex"
  ));
  let oregon = affiliation("Institute of Theoretical Science, University of Oregon");
  assert_eq!(
    creators(&xml),
    vec![
      author_with("N. G. DESHPANDE", &oregon),
      later(author_with("B. DUTTA", &oregon)),
      later(author_with(
        "E. KEITH",
        &affiliation("Department of Physics, University of California")
      )),
    ],
    "{xml}"
  );
  assert_element(
    &xml,
    "abstract",
    &[],
    r#"<abstract inlist="toc" name="Abstract" xml:id="abstract1"><p xml:id="abstract1.1">An abstract.</p></abstract>"#,
  );
}

/// aas2pp4.sty, AASTeX 4's two-column preprint substyle, is the aaspp binding (astro-ph9703136).
#[test]
fn aas2pp4_is_aaspp4() {
  let (log, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/aas2pp4_is_aaspp4.tex"
  ));
  assert_eq!(omnibus_note(&log), None, "{log}");
  assert_eq!(
    creators(&xml),
    vec![
      author_with("Ann Able", &affiliation("Univ X, City")),
      later(author_with("Bob Baker", &affiliation("Univ Y, Town"))),
    ],
    "{xml}"
  );
}

/// Initials glued to a surname in capitals are a name (`M.CACCIA`, hep-ex9910019).
#[test]
fn author_caps_glued_initials_split() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_caps_glued_initials_split.tex"
  ));
  let milano = affiliation("Dipartimento di Fisica, Milano");
  assert_eq!(
    creators(&xml),
    vec![
      author_with("M.CACCIA", &milano),
      later(author_with("R.CAMPAGNOLO", &milano)),
      later(author_with("C.MERONI", &milano)),
    ],
    "{xml}"
  );
}

/// In an article author block, a name line with an author's `\thanks` after an affiliation opens the next author
/// (hep-lat9404004).
#[test]
fn author_note_name_line_opens_next_author() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_note_name_line_opens_next_author.tex"
  ));
  let author = |name: &str, id: &str, note: &str, place: &str| {
    format!(
      "<personname>{name}</personname><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" \
       xml:id=\"{id}\">{note}</note>{}",
      affiliation(place)
    )
  };
  assert_eq!(
    creators(&xml),
    vec![
      format!(
        r#"<creator role="author">{}</creator>"#,
        author(
          "Ann Able",
          "id1",
          "E-mail: ann@x.fi",
          "Univ X, Helsinki, Finland,"
        )
      ),
      later(format!(
        r#"<creator role="author">{}</creator>"#,
        author(
          "Bob Baker",
          "id2",
          "E-mail: bob@y.uk",
          "Univ Y, Liverpool, UK,"
        )
      )),
      later(format!(
        r#"<creator role="author">{}</creator>"#,
        author(
          "Cy Cole",
          "id3",
          "E-mail: cy@x.fi",
          "Univ X, Helsinki, Finland."
        )
      )),
    ],
    "{xml}"
  );
}

/// A doubled separator, `\& and`, parts the names as one does (astro-ph9612009).
#[test]
fn author_doubled_separator_ampersand_and() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_doubled_separator_ampersand_and.tex"
  ));
  let toronto = affiliation("Department of Astronomy, University of Toronto");
  assert_eq!(
    creators(&xml),
    vec![
      author_with("David Schade", &toronto),
      later(author_with("L. Felipe Barrientos", &toronto)),
      later(author_with("Omar López-Cruz", &toronto)),
    ],
    "{xml}"
  );
}

/// AASTeX 3's `{planotable}` ends its own data, with no `\enddata` (aaspptwo.sty:252-256; astro-ph9502038).
#[test]
fn aas_planotable_ends_its_data() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/aas_planotable_ends_its_data.tex"
  ));
  assert_element(
    &xml,
    "tabular",
    &[r#"xml:id="S0.T1.2""#],
    r#"<tabular xml:id="S0.T1.2">
      <thead>
        <tr xml:id="S0.T1.2.1">
          <td align="center" border="tt" thead="column" xml:id="S0.T1.2.1.1">A</td>
          <td align="center" border="tt" thead="column" xml:id="S0.T1.2.1.2">B</td>
        </tr>
      </thead>
      <tbody>
        <tr xml:id="S0.T1.2.2">
          <td align="left" border="t" cssstyle="padding-bottom: 0.0pt" xml:id="S0.T1.2.2.1">1</td>
          <td align="left" border="t" cssstyle="padding-bottom: 0.0pt" xml:id="S0.T1.2.2.2">2</td>
        </tr>
        <tr xml:id="S0.T1.2.3">
          <td align="left" border="b" xml:id="S0.T1.2.3.1">3</td>
          <td align="left" border="b" xml:id="S0.T1.2.3.2"><text xml:id="S0.T1.2.3.2.1"><tabular class="ltx_markedasmath" vattach="bottom" xml:id="S0.T1.2.3.2.1.1">
            <tr xml:id="S0.T1.2.3.2.1.1.1">
              <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="S0.T1.2.3.2.1.1.1.1">x</td>
            </tr>
            <tr xml:id="S0.T1.2.3.2.1.1.2">
              <td align="left" class="ltx_nopad_l ltx_nopad_r" xml:id="S0.T1.2.3.2.1.1.2.1">y</td>
            </tr>
          </tabular></text></td>
        </tr>
      </tbody>
    </tabular>"#,
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p2""#],
    r#"<para xml:id="p2"><p xml:id="p2.1">After the table.</p></para>"#,
  );
}

/// AASTeX 4's `{planotable}` after an explicit `\enddata` ends once (aaspp4.sty `\let\endplanotable=\enddeluxetable`).
#[test]
fn aas4_planotable_with_enddata() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/aas4_planotable_with_enddata.tex"
  ));
  let row = |n: &str, cells: [&str; 3], border: &str| {
    format!(
      r#"<tr xml:id="S0.T1.3.{n}"><td align="left" border="{border}" cssstyle="padding-bottom: 0.0pt" xml:id="S0.T1.3.{n}.1">{}</td><td align="center" border="{border}" cssstyle="padding-bottom: 0.0pt" xml:id="S0.T1.3.{n}.2">{}</td><td align="center" border="{border}" cssstyle="padding-bottom: 0.0pt" xml:id="S0.T1.3.{n}.3">{}</td></tr>"#,
      cells[0], cells[1], cells[2]
    )
  };
  assert_element(
    &xml,
    "tabular",
    &[r#"xml:id="S0.T1.3""#],
    &format!(
      r#"<tabular xml:id="S0.T1.3"><thead><tr xml:id="S0.T1.3.1"><td align="center" border="tt" thead="column" xml:id="S0.T1.3.1.1">Name</td><td align="center" border="tt" thead="column" xml:id="S0.T1.3.1.2">A</td><td align="center" border="tt" thead="column" xml:id="S0.T1.3.1.3">B</td></tr></thead><tbody>{}{}</tbody></tabular>"#,
      row("2", ["X", "1", "2"], "t"),
      row("3", ["Y", "3", "4"], "b")
    ),
  );
  assert_element(
    &xml,
    "para",
    &[r#"xml:id="p1""#],
    r#"<para xml:id="p1"><p xml:id="p1.1">After table.</p></para>"#,
  );
}

/// A package OmniBus required under provide-only is unmarked when the guard ends, so a later `\usepackage{graphicx}`
/// loads it whole: graphicx's keyval `\includegraphics` (review 64zd round 2).
#[test]
fn omnibus_provide_only_package_loads_whole_later() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/omnibus_provide_only_package_loads_whole_later.tex"
  ));
  assert_element(
    &xml,
    "p",
    &[r#"xml:id="p1.1""#],
    r#"<p xml:id="p1.1">A <graphics graphic="w0" options="width=56.9055pt,keepaspectratio=true" xml:id="p1.g1"/> B</p>"#,
  );
}

/// Under plain names, an affiliation line with a `\footnote` stays an affiliation (review 64zd round 2, fn5.tex).
#[test]
fn author_noted_affiliation_stays_affiliation() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_noted_affiliation_stays_affiliation.tex"
  ));
  let places = [
    affiliation("Department of Physics"),
    affiliation("Ohio State"),
    affiliation("Columbus OH"),
  ]
  .concat();
  assert_eq!(
    creators(&xml),
    vec![
      author_with("Ann Able", &places),
      later(author_with("Bob Baker", &places))
    ],
    "{xml}"
  );
  assert!(xml.contains("<pubnote>Visiting.</pubnote>"), "{xml}");
}

/// A sprocl address led by an author's mark goes to the authors showing it (review 64zd round 2, ws1.tex).
#[test]
fn ws_sprocl_marked_addresses() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/ws_sprocl_marked_addresses.tex"
  ));
  assert_eq!(
    creators(&xml),
    vec![
      author_with(
        r#"A. ABLE<sup xml:id="id1"><text font="italic" xml:id="id1.1">a</text></sup>"#,
        &affiliation("Univ X, City")
      ),
      later(author_with(
        r#"B. BAKER<sup xml:id="id2"><text font="italic" xml:id="id2.1">b</text></sup>"#,
        &affiliation("Univ Y, Town")
      )),
    ],
    "{xml}"
  );
}

/// A `\footnote`d affiliation under `\thanks` names stays an affiliation: a noted line opens an author only with the
/// first names' note command (review 64zd round 3, au5.tex).
#[test]
fn author_other_note_command_stays_affiliation() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/author_other_note_command_stays_affiliation.tex"
  ));
  let author = |name: &str, id: &str, note: &str, places: &[&str]| {
    let places: String = places.iter().map(|p| affiliation(p)).collect();
    format!(
      "<personname>{name}</personname><note class=\"ltx_note_frontmatter ltx_thanks_note\" role=\"thanks\" \
       xml:id=\"{id}\">{note}</note>{places}"
    )
  };
  assert_eq!(
    creators(&xml),
    vec![
      format!(
        r#"<creator role="author">{}</creator>"#,
        author("Ann Able", "id1", "x", &["Dept", "Ohio State"])
      ),
      later(format!(
        r#"<creator role="author">{}</creator>"#,
        author("Bob Baker", "id2", "y", &["Dept Y"])
      )),
    ],
    "{xml}"
  );
}

/// AASTeX's `\authoremail` before `\author` is the email of the author it spells, never a creator of its own
/// (astro-ph9803018).
#[test]
fn aas_authoremail_goes_to_its_author() {
  let (_, xml) = convert_clean(include_str!(
    "../../../tools/perfect_kernel/repros/sectioning-frontmatter/aas_authoremail_goes_to_its_author.tex"
  ));
  let univ = affiliation("Univ X, City");
  assert_eq!(
    creators(&xml),
    vec![
      author_with("Ann Able", &univ),
      later(author_with(
        "Bob Baker",
        &format!("{univ}<contact name=\"Email:\u{a0}\" role=\"email\">bbaker@univ.edu</contact>")
      )),
    ],
    "{xml}"
  );
}
