//! Frontmatter class-binding fixtures.
//!
//! Structured, well-rendered author blocks across conference/journal classes.
//! Witnesses are open arXiv HTML "front matter" reports; each fix is described
//! in its binding. `<personname>` counts use the default-namespace
//! serialization (bare tag names).
//!
//! Split out of `06_cluster_regressions`; shares its helpers via
//! [`mod cluster`](cluster).

mod cluster;
use cluster::{convert_to_xml, convert_to_xml_contrib, convert_to_xml_contrib_clean};

/// True if the `<creator>` whose `<personname>` is `name` carries an `email`
/// contact whose text contains `mail`.
fn creator_has_email(xml: &str, name: &str, mail: &str) -> bool {
  xml.split("<creator").skip(1).any(|rest| {
    let block = rest.split("</creator>").next().unwrap_or("");
    block.contains(&format!("<personname>{name}</personname>"))
      && block.contains("role=\"email\"")
      && block.contains(mail)
  })
}

/// True if the `<creator>` whose `<personname>` is `name` has a block containing
/// `needle` (a contact text/attribute substring).
fn creator_block_contains(xml: &str, name: &str, needle: &str) -> bool {
  xml.split("<creator").skip(1).any(|rest| {
    let block = rest.split("</creator>").next().unwrap_or("");
    block.contains(&format!("<personname>{name}</personname>")) && block.contains(needle)
  })
}

/// acmart `\author[F. Poli]{Federico Poli}`: the real class is `\author[2][]`
/// (optional running-head short name + full name). The name must render, and
/// the `[F. Poli]` optarg must NOT leak as a `[` creator. Witness 2405.08372.
#[test]
fn frontmatter_acmart_author_optarg() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_acmart_author_optarg.tex");
  assert!(
    x.contains("Federico Poli"),
    "acmart author name missing:\n{x}"
  );
  assert!(
    !x.contains("<personname>[") && !x.contains("<personname> ["),
    "acmart `[short]` optarg leaked as a bracket creator:\n{x}"
  );
}
/// html_feedback#6885 (arXiv:2608.07766, acmart PACMHCI): the ACM pubnotes
/// (`\acmJournal`/`\acmVolume`/`\acmDOI`/`\ccsdesc`) must be frontmatter SIBLINGS
/// of the title, not nested inside it. The reported "journal info appears in
/// title" was the OLD deployed corpus binary; HEAD matches Perl 0.8.8 exactly —
/// a clean `<title>` with the pubnotes as document children. Pins that so they
/// cannot regress back into the title element.
#[test]
fn frontmatter_acmart_pubnotes_not_in_title() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_acmart_pubnotes_not_in_title.tex");
  // The title is self-contained — a nested <pubnote> would break this exact close.
  assert!(
    x.contains("<title>The Real Title</title>"),
    "acmart title must not absorb the journal/DOI/CCS pubnotes:\n{x}"
  );
  // …and the pubnotes still render, as siblings of the title.
  assert!(
    x.contains("role=\"journal\">PACMHCI</pubnote>")
      && x.contains("role=\"doi\">10.1145/3710969</pubnote>"),
    "acmart journal/DOI pubnotes missing from frontmatter:\n{x}"
  );
}
/// html_feedback#4276 (arXiv:2406.15288, article + biblatex): argumentless biblatex
/// preamble macros — `\renewbibmacro`/`\DeclareNameAlias`/`\addbibresource` — must be
/// consumed, not rendered as literal text above the title (the reader saw them printed
/// at the top on the old deployed binary). The native biblatex binding
/// (`latexml_contrib/src/biblatex_sty.rs`) defines them, so they leave a clean `<title>`
/// and no macro-name text. Deployed-lag; pins it fixed. (The paper's `\externaldocument`
/// from xr is dropped here to keep the fixture warning-free — xr is unimplemented and
/// only warns; it does not leak either.)
#[test]
fn frontmatter_biblatex_xr_preamble_no_leak() {
  // biblatex is a native contrib binding — the real fix — so use the contrib harness,
  // matching the CLI that arxiv runs. The fixture converts with zero errors/warnings.
  let x = convert_to_xml_contrib(
    "tests/cluster_regressions/frontmatter_biblatex_xr_preamble_no_leak.tex",
  );
  assert!(
    x.contains("<title font=\"bold\">The Real Title</title>")
      || x.contains("<title>The Real Title</title>"),
    "document title missing/garbled:\n{x}"
  );
  for cs in ["renewbibmacro", "DeclareNameAlias", "addbibresource"] {
    assert!(
      !x.contains(cs),
      "preamble macro \\{cs} leaked as text into the document:\n{x}"
    );
  }
}
/// html_feedback#6242 (arXiv:2510.02340): two `\textsuperscript{n}Affil` institutions
/// on one space-separated affiliation line — `\textsuperscript{1}University A
/// \textsuperscript{2}University B` — must split into TWO affiliations so each attaches
/// to its authors by number, not merge into one under a single author. Reuses the
/// `\thanks`-abuse splitter (never breaks a superscript glued inside an institution
/// name). Refines OXIDIZED_DESIGN #52(f); Rust surpasses (Perl emits no structured
/// creators for this superscript idiom at all).
#[test]
fn frontmatter_multi_affil_superscript() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_multi_affil_superscript.tex");
  // Both authors present…
  assert!(
    x.contains("<personname>Alice</personname>") && x.contains("<personname>Bob</personname>"),
    "authors missing:\n{x}"
  );
  // …two SEPARATE affiliations, each a standalone contact…
  assert_eq!(
    x.matches("role=\"affiliation\"").count(),
    2,
    "expected exactly two split affiliations:\n{x}"
  );
  assert!(
    x.contains(">University A") && x.contains(">University B<"),
    "the two numbered institutions did not both survive:\n{x}"
  );
  // …and NOT merged into one (the reported canary).
  assert!(
    !x.contains("University A University B"),
    "the two superscript affiliations merged into one:\n{x}"
  );
}
/// html_feedback#6588 (arXiv:2606.01317, ACL superscript block): 10 authors, each
/// `\textbf{Name\textsuperscript{N}}`, then five affiliations one per
/// `\textsuperscript{N}`, COMMA-separated across two `\\` lines. Each author must map
/// to its numbered affiliation, and — the reported "affiliation notes" defect — the
/// institution's trailing comma separator must NOT cling to the name
/// ("The University of Hong Kong," was wrong). Rust already surpasses Perl here (Perl
/// loses every author name). OXIDIZED_DESIGN #52.
#[test]
fn frontmatter_superscript_affil_comma() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_superscript_affil_comma.tex");
  assert_eq!(
    x.matches("role=\"author\"").count(),
    10,
    "expected 10 author creators:\n{x}"
  );
  // The reported canary: no affiliation carries a trailing comma separator.
  for stray in [
    "The University of Hong Kong,",
    "Shandong University,",
    "Carnegie Mellon University,",
    "National University of Singapore,",
  ] {
    assert!(
      !x.contains(stray),
      "affiliation kept its trailing comma separator ({stray:?}):\n{x}"
    );
  }
  // …the clean names are all present…
  for name in [
    "The University of Hong Kong",
    "Shandong University",
    "The Hong Kong University of Science and Technology",
  ] {
    assert!(x.contains(name), "affiliation {name:?} missing:\n{x}");
  }
  // …and the superscript marks still map each author to the right institution.
  let by_name = |name: &str| -> String {
    x.split("<creator")
      .find(|b| b.contains(&format!("<personname>{name}</personname>")))
      .unwrap_or("")
      .to_string()
  };
  assert!(
    by_name("Qi HU").contains("The University of Hong Kong"),
    "Qi HU (\\textsuperscript{{1}}) not linked to Hong Kong:\n{x}"
  );
  assert!(
    by_name("Pengji Zhang").contains("Carnegie Mellon University"),
    "Pengji Zhang (\\textsuperscript{{3}}) not linked to CMU:\n{x}"
  );
  assert!(
    by_name("Lin Zhang").contains("The Hong Kong University of Science and Technology"),
    "Lin Zhang (\\textsuperscript{{5}}) not linked to HKUST:\n{x}"
  );
}
/// html_feedback#1361 + #1362 (arXiv:2401.03955, ttm.sty — an ijcai97.sty
/// derivative): the IJCAI author idiom packs names, `\affiliations` and a
/// comma-separated `\emails` list into ONE `\author{}`. Neither engine's default
/// splitter recognises those section markers, so the email list is shredded into
/// phantom author creators (13 for 7 authors) and the affiliation is dropped —
/// same on Perl 0.8.8 (parity). The fix delegates any `\author{}` carrying an
/// `\affiliations`/`\emails` marker to the shared sectioned-author machinery
/// (also used by `ijcai_sty`), which splits names / affiliations / emails and
/// attaches the n-th email to the n-th author. Rust surpasses Perl.
#[test]
fn frontmatter_ijcai_affiliations_emails() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_ijcai_affiliations_emails.tex");
  // Exactly the seven real authors — the email list must NOT become creators.
  assert_eq!(
    x.matches("role=\"author\"").count(),
    7,
    "IJCAI author block: expected exactly 7 author creators (the \\emails list must \
     not become phantom authors):\n{x}"
  );
  for name in [
    "Vijay Ekambaram",
    "Arindam Jati",
    "Nam H. Nguyen",
    "Pankaj Dayama",
    "Chandra Reddy",
    "Wesley M.\u{a0}Gifford", // `Wesley M.~Gifford` — the ~ tie is a NBSP
    "Jayant Kalagnanam",
  ] {
    assert!(
      x.contains(&format!("<personname>{name}</personname>")),
      "IJCAI author block: author {name:?} missing as its own creator:\n{x}"
    );
  }
  // No email address promoted to a phantom author.
  assert!(
    !x.contains("<personname>arindam.jati") && !x.contains("<personname>nnguyen"),
    "IJCAI author block: an email address leaked into a <personname>:\n{x}"
  );
  // The `\affiliations` payload survives as a structured affiliation contact…
  assert!(
    x.contains("role=\"affiliation\"") && x.contains("IBM Research"),
    "IJCAI author block: the \\affiliations \"IBM Research\" was dropped:\n{x}"
  );
  // …and the emails are email contacts, not names.
  assert!(
    x.contains("role=\"email\"") && x.contains("vijaye12@in.ibm.com"),
    "IJCAI author block: the \\emails list is not structured as email contacts:\n{x}"
  );
}
/// html_feedback#6880 (arXiv:2605.23553, IEEEtran journal): the "all authors, then
/// all affiliations keyed by `\textsuperscript{N}`" block — the harder variation of
/// #6242. A comma-list superscript (`\textsuperscript{1,2}`) links ONE author to
/// TWO affiliations; the affiliations sit one-per-`\\`-line, each led by its own
/// `\textsuperscript{N}`; and `\\[1em]` spacing separates the groups. The reporter's
/// deployed binary — and Perl 0.8.8 — scrambled it: `[1em]` leaked as literal text
/// and the affiliation lines ("University of Pisa", "Italy") became phantom authors
/// (9 creators instead of 6). HEAD's OXIDIZED_DESIGN #52 author-splitter maps each
/// author to its affiliation(s) by number; Rust surpasses Perl. Deployed-lag; this
/// pins the correct mapping so it cannot regress back.
#[test]
fn frontmatter_ieeetran_journal_superscript_affil() {
  let x =
    convert_to_xml("tests/cluster_regressions/frontmatter_ieeetran_journal_superscript_affil.tex");
  // Exactly the six real authors — no affiliation fragment promoted to a creator.
  assert_eq!(
    x.matches("role=\"author\"").count(),
    6,
    "IEEEtran journal: expected exactly 6 author creators (affiliation lines must \
     not become phantom authors):\n{x}"
  );
  for name in [
    "Davide Cosimo",
    "Davide Costa",
    "Riccardo Costanzi",
    "Filippo Campagnaro",
    "Andrea Caiti",
    "Michele Zorzi",
  ] {
    assert!(
      x.contains(&format!("<personname>{name}</personname>")),
      "IEEEtran journal: author {name:?} missing as its own creator:\n{x}"
    );
  }
  // No affiliation fragment promoted to a phantom author, and no `\\[1em]` leak.
  for phantom in [
    "<personname>University of Pisa",
    "<personname>Italy</personname>",
    "<personname>Dept. of Information",
  ] {
    assert!(
      !x.contains(phantom),
      "IEEEtran journal: affiliation fragment leaked as a phantom author ({phantom}):\n{x}"
    );
  }
  assert!(
    !x.contains("[1em]"),
    "IEEEtran journal: `\\\\[1em]` leaked as literal text:\n{x}"
  );
  // Comma-list `\textsuperscript{1,2}`: the first author (Cosimo) links to BOTH
  // affiliation 1 (Pisa) AND affiliation 2 (Naval).
  let cosimo = x
    .split("<creator")
    .find(|b| b.contains("<personname>Davide Cosimo</personname>"))
    .unwrap_or("");
  assert!(
    cosimo.contains("University of Pisa") && cosimo.contains("Naval Support"),
    "IEEEtran journal: comma-list superscript {{1,2}} must link Cosimo to BOTH \
     Pisa and Naval:\n{cosimo}"
  );
}
/// html_feedback#6255 (googledeepmind, authblk): a single `\author{A, B, C}` comma
/// list is authblk's one-author-arg form, so it stayed as one merged
/// `<personname>A, B, C</personname>`. The DEFAULT `\author` already splits a comma
/// list into separate creators; authblk's `\author` routed a no-`\and` list to
/// `\lx@add@creator` (single) instead. Routing a comma list to `\lx@add@authors`
/// too fixes the inconsistency. OXIDIZED_DESIGN #52(h).
#[test]
fn frontmatter_authblk_comma_list() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_authblk_comma_list.tex");
  for name in ["Alice One", "Bob Two", "Carol Three"] {
    assert!(
      x.contains(&format!("<personname>{name}</personname>")),
      "author {name:?} must be its own creator, not merged:\n{x}"
    );
  }
  assert_eq!(
    x.matches("role=\"author\"").count(),
    3,
    "authblk comma list must split into three separate creators:\n{x}"
  );
  assert!(
    !x.contains("<personname>Alice One, Bob"),
    "the comma list stayed welded into one personname:\n{x}"
  );
}
/// A shared author email line must not bunch every address under the last author.
/// Distributed (`a@x, b@y, c@z`) → email i to author i; grouped brace-expansion
/// (`{a,b,c}@dom`) → expand then distribute (and NOT glue into the affiliation);
/// a single shared address → the lead (first) author. OXIDIZED_DESIGN #52(j).
#[test]
fn frontmatter_shared_email_distribution() {
  // 1. Distributed list, one address per author.
  let d = convert_to_xml("tests/cluster_regressions/frontmatter_email_distributed.tex");
  for (name, mail) in [
    ("Alice", "alice@mit.edu"),
    ("Bob", "bob@cmu.edu"),
    ("Carol", "carol@mit.edu"),
  ] {
    assert!(
      creator_has_email(&d, name, mail),
      "distributed: {name} must carry {mail}:\n{d}"
    );
  }
  // 2. Grouped {a,b,c}@dom expands per author; the affiliation stays clean.
  let g = convert_to_xml("tests/cluster_regressions/frontmatter_email_grouped.tex");
  for (name, mail) in [
    ("Alice", "alice@mit.edu"),
    ("Bob", "bob@mit.edu"),
    ("Carol", "carol@mit.edu"),
  ] {
    assert!(
      creator_has_email(&g, name, mail),
      "grouped: {name} must carry expanded {mail}:\n{g}"
    );
  }
  // The grouped email must NOT be glued into an affiliation (a clean affiliation
  // carries no '@').
  for aff in g.split("role=\"affiliation\"").skip(1) {
    let text = aff.split("</contact>").next().unwrap_or("");
    assert!(
      !text.contains('@'),
      "grouped email leaked into the affiliation text:\n{g}"
    );
  }
  // 3. A single shared address lands on the lead (first) author, not the last.
  let s = convert_to_xml("tests/cluster_regressions/frontmatter_email_single_shared.tex");
  assert!(
    creator_has_email(&s, "Alice", "contact@lab.org"),
    "single shared: lead author Alice must carry contact@lab.org:\n{s}"
  );
  assert!(
    !creator_has_email(&s, "Bob", "contact@lab.org"),
    "single shared: must not also land on a trailing author:\n{s}"
  );
}
/// arXiv/html_feedback#46 (witness 2308.06214v1, amsart): all `\author` declared up
/// front, THEN one `\address`/`\email` pair each. LaTeXML's default "attach contact
/// to the preceding creator" bunches every address+email under the LAST author
/// (Perl 0.8.8 too — SHARED). A clean N×m grid (3 authors × {address,email}) must
/// redistribute pair i to author i. The interleaved control (each author followed by
/// its own contacts) is already correct and must stay untouched.
/// OXIDIZED_DESIGN #140 / KNOWN_PERL_ERRORS #104.
#[test]
fn frontmatter_amsart_upfront_contact_distribution() {
  let up = convert_to_xml("tests/cluster_regressions/frontmatter_amsart_upfront_46.tex");
  let authors = [
    ("Peter Feller", "peter.feller@math.ch", "ETH Zurich"),
    (
      "Diana Hubbard",
      "diana.hubbard@brooklyn.cuny.edu",
      "Brooklyn College",
    ),
    (
      "Hannah Turner",
      "hannah.turner@math.gatech.edu",
      "Georgia Institute",
    ),
  ];
  for (name, mail, addr) in authors {
    assert!(
      creator_has_email(&up, name, mail),
      "up-front: {name} must carry its own email {mail}:\n{up}"
    );
    assert!(
      creator_block_contains(&up, name, addr),
      "up-front: {name} must carry its own address {addr}:\n{up}"
    );
  }
  // Tight guard: no author may carry another author's email.
  for (name, ..) in authors {
    for (_, other_mail, _) in authors.iter().filter(|(n, ..)| *n != name) {
      assert!(
        !creator_has_email(&up, name, other_mail),
        "up-front: {name} must NOT carry another author's email {other_mail}:\n{up}"
      );
    }
  }

  // Interleaved control: already correct; the redistribution pass must not disturb it.
  let il = convert_to_xml("tests/cluster_regressions/frontmatter_amsart_interleaved_46.tex");
  for (name, mail, addr) in authors {
    assert!(
      creator_has_email(&il, name, mail) && creator_block_contains(&il, name, addr),
      "interleaved: {name} must keep its own {mail} / {addr}:\n{il}"
    );
    for (_, other_mail, _) in authors.iter().filter(|(n, ..)| *n != name) {
      assert!(
        !creator_has_email(&il, name, other_mail),
        "interleaved: {name} must NOT gain another author's email {other_mail}:\n{il}"
      );
    }
  }
}
/// arXiv:2403.16405 (IEEEtran conference): a `\author{}` grid where `\and` starts a
/// COLUMN and top-level `\\` a ROW within it is linearized column-major (declaration
/// order), scrambling the row-major reading order shown in the PDF/arXiv metadata.
/// `Zhao\\Ding \and Chen\\Kong \and Huang\\Zhang` must emit creators row-major: Zhao,
/// Chen, Huang, Ding, Kong, Zhang. Perl 0.8.8 also mis-orders/drops this grid
/// (SHARED-FAILURE); Rust surpasses (OXIDIZED_DESIGN #127 / KNOWN_PERL_ERRORS #94).
/// The single-row control must NOT be reordered (tight-guard proof).
#[test]
fn frontmatter_ieee_author_grid_transpose() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_ieee_author_grid_transpose.tex");
  // Creators come out in row-major reading order.
  let want = ["Zhao", "Chen", "Huang", "Ding", "Kong", "Zhang"];
  let mut last = 0usize;
  for name in want {
    let pos = x
      .find(&format!("<personname>{name}</personname>"))
      .unwrap_or_else(|| panic!("author {name} missing:\n{x}"));
    assert!(
      pos >= last,
      "author {name} out of row-major order (column-major not transposed):\n{x}"
    );
    last = pos;
  }
  // Control: a plain single-row `\and` list keeps its DECLARED order (no transpose).
  let c = convert_to_xml("tests/cluster_regressions/frontmatter_ieee_author_row_no_transpose.tex");
  let (a, b, cc) = (
    c.find("<personname>Alice</personname>"),
    c.find("<personname>Bob</personname>"),
    c.find("<personname>Carol</personname>"),
  );
  assert!(
    a.is_some() && a < b && b < cc,
    "single-row \\and list was wrongly reordered:\n{c}"
  );
}
/// html_feedback#1021 (arXiv:2403.11905): an author carries a doubly-nested
/// inline-math superscript (`$^\text{$...$}$`) as an affiliation marker. The
/// author-marker (withsup) branch `\let`s `^` onto an annotation primitive that
/// read a bare `{}` operand, grabbing only the leading `\text` and orphaning its
/// `{...}` argument; inside the marker's own inline math the stray `{...}` left a
/// brace-group frame on top, so the closing `$` fired `\lx@end@inline@math`
/// against it — a cascade of "Attempt to end mode math" that garbled every
/// creator. Both engines erred (SHARED-FAILURE); the Rust marker now reads a FULL
/// superscript operand, keeping `\text{...}` and its nested `$...$` whole. Reading
/// the operand undigested also collapses the empty math, so the marker links the
/// author to the shared affiliation instead of rendering. OXIDIZED_DESIGN #129.
#[test]
fn frontmatter_nested_math_author_marker() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_nested_math_author_marker.tex");
  // Zero-error gate (convert_to_xml) already guards the "Attempt to end mode
  // math" cascade; assert the markup is clean and the creators did NOT merge.
  assert!(
    x.contains("<personname>Alice</personname>") && x.contains("<personname>Bob</personname>"),
    "both creators must survive as clean, separate personnames:\n{x}"
  );
  assert_eq!(
    x.matches("role=\"author\"").count(),
    2,
    "expected exactly two author creators, not a merged blob:\n{x}"
  );
  // The nested-math markers link both authors to the shared affiliation.
  assert_eq!(
    x.matches("role=\"affiliation\"").count(),
    2,
    "each author should carry the shared affiliation contact:\n{x}"
  );
  assert!(
    x.contains("Shared University"),
    "the shared affiliation text must survive:\n{x}"
  );
}
/// html_feedback#1021 F2 residual (arXiv:2403.11905): `\and` must be a HARD author
/// boundary in the superscript-marker branch. When a 2nd/3rd author's marker is
/// delivered by a macro (no literal `^` on that segment), the old flat
/// `\and`/`\quad`/`\\` split appended the marker-less segment to the PREVIOUS
/// author, collapsing `Alice\mk$^{1}$ \and Bob\mk \and Carol\mk` into one merged
/// `<personname>`. Grouping on `\and` first (append bounded to the group) keeps
/// them separate. OXIDIZED_DESIGN #52(g).
#[test]
fn frontmatter_and_hard_author_boundary() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_and_hard_author_boundary.tex");
  for name in ["Alice", "Bob", "Carol"] {
    assert!(
      x.contains(&format!("<personname>{name}</personname>")),
      "author {name} must be its own clean creator, not merged:\n{x}"
    );
  }
  assert_eq!(
    x.matches("role=\"author\"").count(),
    3,
    "expected exactly three separate author creators across the \\and boundaries:\n{x}"
  );
  // The reported canary: the three names must not weld into one personname.
  assert!(
    !x.contains("AliceBob") && !x.contains("BobCarol"),
    "\\and-separated authors merged into one personname:\n{x}"
  );
}
/// html_feedback#6637 construct 1 (OXIDIZED_DESIGN #52): co-authors separated by
/// `\hspace{len}` (a "regular" poor-man's separator LaTeXML's `\and`/`\quad`
/// splitter otherwise misses) must split into distinct creators, not weld into
/// one `<personname>`. `\hspace`'s length argument must not leak as text.
#[test]
fn frontmatter_hspace_author_split() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_hspace_author_split.tex");
  for name in ["Alice Alpha", "Bob Beta", "Carol Gamma"] {
    assert!(
      x.contains(&format!("<personname>{name}</personname>")),
      "\\hspace-separated author {name} must be its own creator:\n{x}"
    );
  }
  assert_eq!(
    x.matches("role=\"author\"").count(),
    3,
    "expected three creators split on \\hspace:\n{x}"
  );
  // The reported canary: names must not weld, and the length must not leak.
  assert!(
    !x.contains("AlphaBob") && !x.contains("BetaCarol") && !x.contains("1cm"),
    "\\hspace authors merged or the length leaked as text:\n{x}"
  );
}
/// html_feedback#6637 construct 2 (OXIDIZED_DESIGN #52): a footnote-SYMBOL author
/// superscript (`$^{*}$`, `$^{\dagger}$`, `\textsuperscript{*}` — equal-
/// contribution / corresponding notes, never affiliation numbers) must render as
/// a visible `<sup>`, not be consumed into an unmatched `affiliation:*` label and
/// silently dropped. All three marker spellings are covered; a plain author stays
/// plain, and every author still splits on `\and`.
#[test]
fn frontmatter_symbol_superscript_mark() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_symbol_superscript_mark.tex");
  // Each marked author keeps a visible superscript symbol…
  assert!(
    x.contains("<personname>Dana Delta<sup>*</sup>"),
    "literal $^{{*}}$ mark dropped from Dana Delta:\n{x}"
  );
  assert!(
    x.contains("<personname>Evan Echo<sup>†</sup>"),
    "$^{{\\dagger}}$ mark dropped from Evan Echo:\n{x}"
  );
  assert!(
    x.contains("<personname>Fiona Foxtrot<sup>*</sup>"),
    "\\textsuperscript{{*}} mark dropped from Fiona Foxtrot:\n{x}"
  );
  // …the unmarked author stays plain…
  assert!(
    x.contains("<personname>Gina Golf</personname>"),
    "unmarked author Gina Golf must stay plain:\n{x}"
  );
  // …all four split, and the mark never became an affiliation.
  assert_eq!(
    x.matches("role=\"author\"").count(),
    4,
    "expected four authors:\n{x}"
  );
  assert!(
    !x.contains("role=\"affiliation\""),
    "a symbol mark was misread as an affiliation:\n{x}"
  );
}
/// html_feedback#6637 combined (arXiv:2506.06941, "The Illusion of Thinking",
/// plain article): six authors separated by `\hspace{0.5cm}`/`\\`, one lead with
/// two `\thanks`, a second lead with a literal `$^{*}$`, trailing "Apple"
/// affiliation. Both engines (Perl 0.8.8 == HEAD before this fix) welded all six
/// names into one `<personname>` with "Apple" glued on and Mirzadeh's `$^{*}$`
/// dropped. With `\hspace`→separator + symbol-mark→visible-sup (removing the only
/// affiliation-marker trigger), the block takes the clean no-marker branch: all
/// six split, Mirzadeh keeps his `∗`, Shojaee keeps his thanks, and "Apple"
/// becomes the last author's affiliation. OXIDIZED_DESIGN #52.
#[test]
fn frontmatter_thanks_literal_mark_mix() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_thanks_literal_mark_mix.tex");
  // All six authors are their own creators (was one welded blob).
  for name in [
    "Parshin Shojaee",
    "Keivan Alizadeh",
    "Maxwell Horton",
    "Samy Bengio",
    "Mehrdad Farajtabar",
  ] {
    assert!(
      x.contains(&format!("<personname>{name}")),
      "author {name} missing as a distinct creator:\n{x}"
    );
  }
  assert_eq!(
    x.matches("role=\"author\"").count(),
    6,
    "expected six split authors:\n{x}"
  );
  // Mirzadeh's literal $^{*}$ survives as a visible superscript.
  assert!(
    x.contains("<personname>Iman Mirzadeh<sup>*</sup>"),
    "Mirzadeh's literal $^{{*}}$ mark was dropped:\n{x}"
  );
  // Shojaee's two \thanks attach to HIS creator (not the welded blob).
  assert!(
    x.contains("Equal contribution.") && x.contains("Work done during an internship at Apple."),
    "Shojaee's \\thanks notes were lost:\n{x}"
  );
  // "Apple" is the trailing affiliation, not glued into a name.
  assert!(
    x.contains("role=\"affiliation\">Apple"),
    "\"Apple\" must be the last author's affiliation, not welded into a name:\n{x}"
  );
  assert!(
    !x.contains("FarajtabarApple") && !x.contains("[0.5cm]"),
    "\"Apple\" welded onto a name or the \\\\[0.5cm] length leaked:\n{x}"
  );
}
/// html_feedback#6614 (arXiv:2606.08234, ACL): a `\author{Name\quad Name… \\
/// \textsuperscript{n}Affil}` block must keep SHORT author names as authors, not
/// reclassify them as affiliations. "Min Xu" (7 tokens) tripped the old `p < 8`
/// superscript-position proxy and was demoted to an `Affiliation:`; the
/// length-independent name-before-marker rule keeps all four authors while the
/// marker-led affiliation lines stay affiliations.
#[test]
fn frontmatter_acl_quad_authors_short_name() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_acl_quad_authors.tex");
  // All four authors survive as personnames — including the short "Min Xu".
  for name in [
    "Tanush Swaminathan",
    "Runmin Jiang",
    "Letian Zhang",
    "Min Xu",
  ] {
    assert!(
      x.contains(&format!("<personname>{name}")),
      "author {name} missing as a personname:\n{x}"
    );
  }
  // "Min Xu" must NOT be demoted to an affiliation (the reported canary).
  assert!(
    !x.contains("role=\"affiliation\">Min Xu"),
    "short author Min Xu misclassified as an affiliation:\n{x}"
  );
  // The marker-led affiliation lines still render as affiliations.
  assert!(
    x.contains("Carnegie Mellon University") && x.contains("Allen Institute"),
    "affiliations dropped:\n{x}"
  );
  // The `\\[5pt]` optional length must not leak as literal text.
  assert!(
    !x.contains("[5pt]"),
    "\\\\[5pt] optional length leaked as text:\n{x}"
  );
}
/// html_feedback#6870 (arXiv:2312.14226, aistats2024): `\renewcommand{\abstractname}
/// {\centering {\large Abstract}}` — the abstract heading must read "Abstract", not
/// leak the alignment declaration as literal text `\centeringAbstract`. Both Perl and
/// Rust digested `\centering`'s constructor reversion into the `name=` string; the
/// designated hook `\format@title@abstract` now neutralizes alignment declarations
/// during name extraction (mirrors the `titlepage` `Let('\centering','\relax')`
/// precedent). Surpass-Perl divergence — see OXIDIZED_DESIGN_DIVERGENCES #121.
#[test]
fn frontmatter_abstract_centering_name() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_abstract_centering_name.tex");
  assert!(
    x.contains("name=\"Abstract\""),
    "abstract name must be the clean text \"Abstract\":\n{x}"
  );
  assert!(
    !x.contains("\\centering"),
    "alignment declaration leaked as literal text into the abstract name:\n{x}"
  );
}
/// html_feedback#61 (arXiv:2308.06262, neurips_2023): the `\author` block bolds
/// only its second name line (`\textbf{…}`) and relies on the class's block-level
/// `\bf` to bold the rest — which LaTeXML doesn't emulate, so line 1 rendered plain
/// and line 2 bold (incoherent). A `font="bold"` that wraps an ENTIRE personname is
/// presentational author-block styling, not semantic; it is now unwrapped so every
/// author renders coherently. Surpass-Perl divergence — see
/// OXIDIZED_DESIGN_DIVERGENCES #122. (Plain `article` reproduces the parse — no
/// neurips dependency.) `\textbf{Zhou Zhao}\footnotemark[2]` additionally guards that
/// a trailing reference marker (`<ltx:note>`) does not block the unwrap — witness
/// "Zhou Zhao" in arXiv 2507.06670.
#[test]
fn frontmatter_neurips_author_bold_coherent() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_neurips_author_bold_coherent.tex");
  // The plain-`\textbf` authors survive as plain personnames…
  for name in ["Fanqing Meng", "Wenqi Shao", "Kaipeng Zhang", "Yu Qiao"] {
    assert!(
      x.contains(&format!("<personname>{name}</personname>")),
      "author {name} must be a plain personname (whole-name bold unwrapped):\n{x}"
    );
  }
  // …the bold author WITH a trailing footnotemark is also unwrapped (the marker
  // stays inside the personname, the bold does not).
  assert!(
    x.contains("<personname>Zhou Zhao") && x.contains("role=\"footnotemark\""),
    "bold author with a trailing footnotemark marker was not unwrapped:\n{x}"
  );
  // …and none carries a whole-name bold wrapper.
  assert!(
    !x.contains("<personname><text font=\"bold\">"),
    "a whole-personname bold wrapper survived (incoherent with the plain lines):\n{x}"
  );
}
/// A multi-line `\author` block whose first line ends with a trailing `\quad \\`
/// (arXiv 2507.06670, acl): the leaked `\\` heads the next `\quad`-group, so its
/// first author ("Carol Three" here; "Ruiqi Li" in the paper) landed on an EMPTY
/// names_line and was demoted to a bogus affiliation with an empty `<personname/>`.
/// Dropping empty `\\`-pieces up front keeps every line-2 author an author.
#[test]
fn frontmatter_multiline_author_leading_break() {
  let x =
    convert_to_xml("tests/cluster_regressions/frontmatter_multiline_author_leading_break.tex");
  // All four authors — including the first of the second line — are personnames.
  for name in ["Alice One", "Bob Two", "Carol Three", "Dan Four"] {
    assert!(
      x.contains(&format!("<personname>{name}</personname>")),
      "author {name} missing as a personname:\n{x}"
    );
  }
  // The reported canary: no empty personname, and the line-2 first author is NOT
  // demoted to an affiliation.
  assert!(
    !x.contains("<personname/>") && !x.contains("<personname></personname>"),
    "a leading `\\\\` produced an empty personname:\n{x}"
  );
  assert!(
    !x.contains("role=\"affiliation\">Carol Three"),
    "line-2 first author Carol Three misclassified as an affiliation:\n{x}"
  );
  // The genuine affiliation still attaches.
  assert!(x.contains("Some University"), "affiliation dropped:\n{x}");
}
/// IEEEtran `\author{\IEEEauthorblockN{…}\IEEEauthorblockA{…}\and …}`: each
/// block is one creator; the `1\textsuperscript{st}` ordinals must not be
/// misread as affiliation markers and drop every author. Witness 2602.05517.
#[test]
fn frontmatter_ieee_authorblock() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_ieee_authorblock.tex");
  assert!(
    x.contains("Alice Smith"),
    "IEEE authorblock author 1 missing:\n{x}"
  );
  assert!(
    x.contains("Bob Jones"),
    "IEEE authorblock author 2 missing:\n{x}"
  );
  assert!(
    x.matches("<personname>").count() >= 2,
    "IEEE authorblock must yield >=2 creators, got {}:\n{x}",
    x.matches("<personname>").count()
  );
}

/// An author-attached `\thanks` becomes a MARKED `<ltx:note role="thanks">` (not an
/// inline `<ltx:contact role="thanks">`), carrying semantic class hooks so a theme can
/// style each content kind. Surpass, OXIDIZED_DESIGN #156. Witnesses arXiv 2512.24601
/// (correspondence), 1510.02728 (funding).
#[test]
fn cluster_author_thanks_marked_note() {
  let x = convert_to_xml("tests/cluster_regressions/author_thanks_marked_note.tex");
  // No author-scope thanks CONTACT survives — every thanks is a <note> now.
  let thanks_contacts = x
    .split("<contact")
    .skip(1)
    .filter(|seg| {
      seg
        .split('>')
        .next()
        .unwrap_or("")
        .contains(r#"role="thanks""#)
    })
    .count();
  assert_eq!(
    thanks_contacts, 0,
    "an author \\thanks is still an inline role=thanks contact:\n{x}"
  );
  // Each \thanks is a <note role="thanks"> with the right content-kind class hook.
  for (needle, kind) in [
    ("Correspondence to", "ltx_thanks_correspondence"),
    ("supported by NSF", "ltx_thanks_funding"),
    ("contributed equally", "ltx_thanks_contribution"),
    ("Now at Acme", "ltx_thanks_address"),
    ("Warm thanks", "ltx_thanks_note"),
  ] {
    // find the note element carrying this text, assert it is a role=thanks note with the class.
    let has = x.split("<note").skip(1).any(|seg| {
      let block = seg.split("</note>").next().unwrap_or("");
      block.contains(needle) && block.contains("role=\"thanks\"") && block.contains(kind)
    });
    assert!(
      has,
      "\\thanks {needle:?} not a role=thanks <note> classed {kind}:\n{x}"
    );
  }
  // The mark-bearing notes live inside creators (not detached).
  assert!(
    x.matches("<note class=\"ltx_note_frontmatter").count() == 5,
    "expected 5 frontmatter thanks notes:\n{x}"
  );
}

/// IEEEtran `\author{\IEEEauthorblockN{…}\IEEEauthorblockA{…} \{…\}@host}`: a bare
/// email trailing after `\IEEEauthorblockA` must attach to the creator (as an
/// affiliation), NOT leak into the document body as the first `<p>`. GENUINE-RUST-ONLY
/// (Perl bundles the whole `\author` arg into `<personname>`, so it never leaks).
/// Fixed by `wrap_bare_author_block_text` in `ieeetran_cls.rs`. Witness arXiv 1901.07768.
#[test]
fn frontmatter_ieee_authorblock_trailing_email() {
  let x =
    convert_to_xml("tests/cluster_regressions/frontmatter_ieee_authorblock_trailing_email.tex");
  // The leak symptom: the email opened the document body as its first paragraph.
  assert!(
    !x.contains("<p>{anuja") && !x.contains("<p>\n{anuja"),
    "trailing bare email leaked into the document body as a <p>:\n{x}"
  );
  // The email must instead live inside the creator's frontmatter (an affiliation).
  assert!(
    creator_block_contains(
      &x,
      "Anuja Meetoo Appavoo, Seth Gilbert, and Kian-Lee Tan",
      "@comp.nus.edu.sg"
    ),
    "trailing email is not attached to the creator's frontmatter:\n{x}"
  );
  // The genuine body paragraph is still present and correct.
  assert!(
    x.contains("Body paragraph here."),
    "body paragraph missing:\n{x}"
  );
}
/// IEEEtran `\IEEEmembership{Senior Member, IEEE}` inside a flat comma author
/// list must not become a phantom "Senior Member, IEEE" creator. Witness
/// 2508.00603 (html_feedback#4539: reader saw a stray "," between authors). The
/// comma-split leaves EMPTY name pieces where each `\IEEEmembership`/" and " sat;
/// those must not surface as empty `<personname/>` creators, and a trailing
/// `\thanks` must attach to the preceding real author, not to a nameless creator.
#[test]
fn frontmatter_ieee_membership_no_phantom() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_ieee_membership.tex");
  assert!(
    x.contains("Alice Smith") && x.contains("Bob Jones"),
    "IEEE authors missing:\n{x}"
  );
  assert!(
    !x.contains("<personname>Senior Member") && !x.contains("<personname>Member, IEEE"),
    "IEEEmembership leaked as a phantom creator:\n{x}"
  );
  // No empty personname creators from the comma-split membership/" and " gaps.
  assert!(
    !x.contains("<personname/>") && !x.contains("<personname></personname>"),
    "comma-split left an empty <personname/> creator:\n{x}"
  );
  // Exactly the two real authors.
  assert_eq!(
    x.matches("<creator role=\"author\"").count() + x.matches("<creator before").count(),
    2,
    "expected exactly 2 author creators (Alice, Bob):\n{x}"
  );
  // The \thanks note is not stranded on a nameless creator.
  assert!(
    !x.contains("<personname/>\n") || !x.contains("Funding note"),
    "the \\thanks funding note stranded on an empty creator:\n{x}"
  );
}
/// IEEEtran lazy single-`\author` block with `\\[1em]` row breaks (witness
/// 2605.23553, arXiv/html_feedback; KNOWN_PERL_ERRORS #75). The optional-length
/// `[1em]` on `\\` must be consumed, not leak as literal `[1em]` text; and its
/// removal keeps the following `\textsuperscript` at the front of the
/// affiliation line so the comma-bearing address is NOT reclassified into
/// phantom "University of Pisa"/"Italy" author creators. Beyond-Perl: Perl 0.8.8
/// garbles this identically (its own `\lx@add@authors` flags the gap).
#[test]
fn frontmatter_ieee_linebreak_optarg() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_ieee_linebreak_optarg.tex");
  // The `\\[1em]` length must not leak as literal text anywhere.
  assert!(
    !x.contains("[1em]"),
    "\\\\[1em] optional length leaked as text:\n{x}"
  );
  // The three authors are clean personnames.
  assert!(
    x.contains("<personname>Alice Smith")
      && x.contains("<personname>Bob Jones")
      && x.contains("<personname>Carol White"),
    "IEEE lazy-block authors missing/unstructured:\n{x}"
  );
  // The affiliation lines stay affiliations, not comma-split into phantom authors.
  assert!(
    !x.contains("<personname>University of Pisa")
      && !x.contains("<personname>Italy")
      && !x.contains("<personname>University of Padua"),
    "affiliation address commas mis-split into phantom authors:\n{x}"
  );
  // The affiliation content still survives (attached as affiliation contacts).
  assert!(
    x.contains("University of Pisa") && x.contains("University of Padua"),
    "affiliation content dropped:\n{x}"
  );
  // The shared \texttt{} email line becomes its own email contact, shown once —
  // not welded into an affiliation's text.
  assert!(
    x.contains("role=\"email\"") && x.contains("alice@unipi.it"),
    "shared email line not routed to an email contact:\n{x}"
  );
  // The email must not be glued onto an affiliation's content (the pre-fix bug
  // was `…Padua, Italy<text font=\"typewriter\">alice@…` inside role=affiliation).
  assert!(
    !x.contains("Italy<text"),
    "email welded into affiliation text instead of its own contact:\n{x}"
  );
}
/// Modern Interspeech.cls `\name[affiliation={1,*}]{First}{Last}` (2-arg): the
/// author renders as "First Last"; the `[affiliation=…]` optarg must not leak a
/// `[` creator or `\name`. Interspeech2024 resolves here by version-stripping.
/// Witness 2406.11727.
#[test]
fn frontmatter_interspeech2024_name() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/frontmatter_interspeech2024_name.tex");
  assert!(
    x.contains("Alice Smith"),
    "Interspeech author 1 missing:\n{x}"
  );
  assert!(
    x.contains("Bob Jones"),
    "Interspeech author 2 missing:\n{x}"
  );
  assert!(!x.contains("\\name"), "Interspeech `\\name` leaked:\n{x}");
  assert!(
    !x.contains("<personname>["),
    "Interspeech optarg leaked as bracket:\n{x}"
  );
}
/// czipreprint `\author[1]{…}` / `\author*[1,2]{…}` (starred = corresponding):
/// the star must be peeked via `\@ifstar`, not baked into the signature (which
/// would break the plain form → `]Name` leak). Witness 2508.00826.
#[test]
fn frontmatter_czipreprint_author_star() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/frontmatter_czipreprint_author.tex");
  assert!(
    x.contains("Alice Smith"),
    "czipreprint plain author missing:\n{x}"
  );
  assert!(
    x.contains("Bob Jones"),
    "czipreprint starred author missing:\n{x}"
  );
  assert!(
    !x.contains("<personname>]"),
    "czipreprint `[n]` optarg leaked a `]`:\n{x}"
  );
}
/// ceurart `\author[affil]{name}[orcid=…,email=…]` — the modern per-author
/// keyval bracket (ceurart.cls L1247 `\RenewDocumentCommand\author{O{} m O{}}`).
/// The binding didn't define `\author`, so the trailing `[keyval]` fell through
/// OmniBus's `\author[]{}` unconsumed and LEAKED as raw `orcid=…, email=…` text.
/// The 3-arg form now routes the name to a creator and parses the keyval into
/// role=orcid/email notes. arXiv/html_feedback#6650, witness 2511.11770.
#[test]
fn frontmatter_ceurart_author_orcid_keyval() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/ceurart_author_orcid_keyval.tex");
  // The keyval no longer leaks as literal `key=value` text.
  assert!(
    !x.contains("orcid=") && !x.contains("email="),
    "ceurart author keyval leaked as raw text:\n{x}"
  );
  // The author renders, and the orcid/email VALUES are preserved as notes.
  assert!(
    x.contains("Alice Smith"),
    "ceurart author name missing:\n{x}"
  );
  assert!(
    x.contains("role=\"orcid\"") && x.contains("0000-0003-0583-6969"),
    "ceurart orcid value not preserved as a note:\n{x}"
  );
  assert!(
    x.contains("role=\"email\"") && x.contains("alice@example.org"),
    "ceurart email value not preserved as a note:\n{x}"
  );
}
/// spconf.sty / INTERSPEECH2021.sty single-arg `\name{Author1$^1$, Author2$^2$}`
/// on `\documentclass{article}`: the name list becomes structured creators
/// rather than being stashed and dropped. Witness 2309.14838, 2405.13379.
#[test]
fn frontmatter_spconf_name() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/frontmatter_spconf_name.tex");
  assert!(x.contains("Alice Smith"), "spconf author 1 missing:\n{x}");
  assert!(x.contains("Bob Jones"), "spconf author 2 missing:\n{x}");
}
/// spconf.sty `\begin{keywords}…\end{keywords}` — the "Index Terms" block, a
/// bare `\def\keywords`/`\def\endkeywords` pair (spconf.sty L211-214), not a
/// `\newenvironment`. It must become structured `ltx:keywords` frontmatter
/// with the label in `@name`, not bounce as an undefined environment (94 papers
/// in sandbox-arxiv-2605, 49 in 2606 — the single largest `undefined` what).
/// Witnesses 2605.00480, 2605.00698, 2605.00721, 2605.01187.
#[test]
fn frontmatter_spconf_keywords() {
  let x = convert_to_xml_contrib_clean("tests/cluster_regressions/frontmatter_spconf_keywords.tex");
  assert!(
    x.contains("<keywords"),
    "spconf keywords did not become ltx:keywords frontmatter:\n{x}"
  );
  assert!(
    x.contains("Speech recognition, deep learning"),
    "spconf keyword list missing:\n{x}"
  );
  assert!(
    x.contains("name=\"Index Terms:"),
    "spconf keywords label not carried in @name:\n{x}"
  );
  // The label is an attribute, never inline content of the block.
  assert!(
    !x.contains(">Index Terms"),
    "spconf `Index Terms` label leaked into the content:\n{x}"
  );
}
/// spconf's `\keywords` is argument-less, so the braced `\keywords{a, b}` form
/// is legal too. Routed to the bare environment opener it would find no
/// `\endkeywords` and scan to EOF, dragging the whole body into
/// `<ltx:keywords>`; the `{`-peek (Perl's `\keywords@onearg`, IEEEtran.cls.ltxml
/// L398-404) must close the block after the argument.
#[test]
fn frontmatter_spconf_keywords_braced() {
  let x = convert_to_xml_contrib_clean(
    "tests/cluster_regressions/frontmatter_spconf_keywords_braced.tex",
  );
  // The `@name` separator is a `~` tie (U+00A0), so match around it.
  assert!(
    x.contains("<keywords name=\"Index Terms:")
      && x.contains(">Speech recognition, deep learning</keywords>"),
    "braced spconf `\\keywords{{…}}` did not close after its argument:\n{x}"
  );
  assert!(
    x.contains("<section"),
    "the document body was swallowed into the keywords block:\n{x}"
  );
}
/// spconf.sty `\twoauthors{N1}{A1}{N2}{A2}` (L183-190) — the side-by-side
/// two-author title block. Each pair must become a creator with its own
/// affiliation instead of an undefined-token `<ltx:ERROR/>`.
/// Witnesses 2605.05692, 2605.18923, 2605.26747.
#[test]
fn frontmatter_spconf_twoauthors() {
  let x =
    convert_to_xml_contrib_clean("tests/cluster_regressions/frontmatter_spconf_twoauthors.tex");
  assert!(
    x.contains("<personname>Alice Smith</personname>"),
    "twoauthors author 1 is not a creator:\n{x}"
  );
  assert!(
    x.contains("<personname>Bob Jones</personname>"),
    "twoauthors author 2 is not a creator:\n{x}"
  );
  assert!(
    x.contains("University A") && x.contains("University B"),
    "twoauthors affiliations missing:\n{x}"
  );
}
/// atlasdoc `\AtlasTitle{…}` / `\AtlasAbstract{…}` / `\AtlasOrcid[orcid]{Name}`:
/// the frontmatter macros of the (very large, unbound) ATLAS class must not leak
/// as literal text — the title/abstract render and the collaboration author
/// names show. Witness 2508.20929. (Full author-list-as-creators is out of scope
/// for this minimal frontmatter binding — the list is `\input` in the body.)
#[test]
fn frontmatter_atlasdoc_title() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/frontmatter_atlasdoc_title.tex");
  assert!(
    x.contains("heavy neutral leptons"),
    "AtlasTitle text missing:\n{x}"
  );
  assert!(
    !x.contains("\\AtlasTitle") && !x.contains("\\AtlasAbstract") && !x.contains("\\AtlasOrcid"),
    "Atlas frontmatter macro leaked as raw text:\n{x}"
  );
  assert!(x.contains("Aad"), "AtlasOrcid author name missing:\n{x}");
}
/// jmlr.cls `\author{ \Name{N} \Email{E} \\ ... \addr Affiliation }`: the
/// structured sub-macros must build one clean creator per `\Name` (name →
/// personname, `\Email` → contact[email], the trailing `\addr` block →
/// contact[affiliation]), not cram everything into one personname or split the
/// affiliation's commas into phantom "Foo"/"FL" authors. `\nametag` must not
/// leak. Witness 2410.16138.
#[test]
fn frontmatter_jmlr_structured_author() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/frontmatter_jmlr_name.tex");
  assert!(
    x.contains("<personname>Alice Smith</personname>"),
    "jmlr author 1 not a clean personname:\n{x}"
  );
  assert!(
    x.contains("<personname>Bob Jones</personname>"),
    "jmlr author 2 not a clean personname:\n{x}"
  );
  assert!(
    !x.contains("\\Name") && !x.contains("\\nametag") && !x.contains("\\addr"),
    "jmlr author sub-macro leaked as raw text:\n{x}"
  );
  assert!(
    x.contains("role=\"email\"") && x.contains("alice@example.edu"),
    "jmlr email not structured:\n{x}"
  );
  assert!(
    x.contains("role=\"affiliation\"") && x.contains("Department of Computer Science"),
    "jmlr affiliation not structured:\n{x}"
  );
  assert!(
    !x.contains("<personname>Foo") && !x.contains("<personname>FL"),
    "jmlr affiliation commas mis-split into phantom authors:\n{x}"
  );
}
/// MRM.cls (Wiley `\author[idx]{name}{orcid}` family): the author name renders,
/// the ORCID becomes a linked contact, `\address`/`\state`/`\country` don't leak
/// (`\state` is deliberately absent from OmniBus), and `\corres`/`\finfo` are
/// preserved as notes. Witness 2509.13644.
#[test]
fn frontmatter_mrm_author() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/frontmatter_mrm_author.tex");
  assert!(
    x.contains("<personname>Jakob Asslander*</personname>"),
    "MRM author name missing/unstructured:\n{x}"
  );
  assert!(
    !x.contains("\\state")
      && !x.contains("\\orcid")
      && !x.contains("\\corres")
      && !x.contains("\\authormark"),
    "MRM frontmatter macro leaked as raw text:\n{x}"
  );
  assert!(
    x.contains("role=\"orcid\"") && x.contains("0000-0003-2288-038X"),
    "MRM ORCID not a structured contact:\n{x}"
  );
  assert!(
    x.contains("Center for Biomedical Imaging"),
    "MRM affiliation content missing:\n{x}"
  );
}

/// Springer-Nature `sn-jnl.cls` with a single **shared unnumbered** `\affil`
/// (the witness's shape): all three authors render as clean personnames, and
/// the one affiliation must attach to the author creators as a
/// `<contact role="affiliation">`, NOT float off as a top-level
/// `<note role="affiliation">` dagger orphan attached to nobody.
/// arXiv/html_feedback#534, witness 2204.04741.
#[test]
fn frontmatter_sn_jnl_shared_affil() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/frontmatter_sn_jnl_affil.tex");
  assert!(
    x.matches("<personname>").count() >= 3,
    "sn-jnl: expected 3 author personnames:\n{x}"
  );
  assert!(
    x.contains("Asare") && x.contains("Nagappan") && x.contains("Asokan"),
    "sn-jnl author names missing:\n{x}"
  );
  // The affiliation is a structured contact attached to the authors...
  assert!(
    x.contains("<contact")
      && x.contains("role=\"affiliation\"")
      && x.contains("University of Waterloo"),
    "sn-jnl shared affiliation is not a structured <contact role=affiliation>:\n{x}"
  );
  // ...on ALL three authors (a single shared \affil → annotate=all), not just
  // the last creator...
  assert!(
    x.matches("role=\"affiliation\"").count() >= 3,
    "sn-jnl shared affiliation did not attach to all three authors:\n{x}"
  );
  // ...not an orphaned top-level note (the pre-fix bug).
  assert!(
    !x.contains("<note role=\"affiliation\""),
    "sn-jnl affiliation leaked as an orphaned top-level note:\n{x}"
  );
}

/// Springer-Nature `sn-jnl.cls` with `\abstract{...}` written BEFORE
/// `\maketitle` (the shape every sn-jnl paper uses). The abstract is a *locked*
/// deferred-frontmatter macro (`\lx@add@abstract`, latex_constructs `\abstract`
/// L5153), so `\maketitle` must flush it in schema-canonical order — the
/// document `<title>` FIRST, the `<abstract>` after it — not emit the abstract
/// immediately at its call site (which lands it before the still-deferred
/// title). Perl locks the core `\abstract` so the raw class's `\def\abstract`
/// cannot pull it back to an immediate emit; the binding must not override it
/// either. arXiv/html_feedback#3436, witness 2411.11158, 2306.11901.
#[test]
fn frontmatter_sn_jnl_abstract_after_title() {
  let x =
    convert_to_xml_contrib_clean("tests/cluster_regressions/frontmatter_sn_jnl_abstract_order.tex");
  let title = x.find("<title>").expect("document <title> present");
  let abstract_ = x.find("<abstract").expect("<abstract> present");
  assert!(
    title < abstract_,
    "sn-jnl abstract must render AFTER the title, not before it \
     (title@{title}, abstract@{abstract_}):\n{x}"
  );
  // The abstract text is preserved…
  assert!(
    x.contains("This is the abstract."),
    "sn-jnl abstract body missing:\n{x}"
  );
  // …and the author frontmatter still flushes with the title block.
  assert!(
    x.contains("<personname>") && x.contains("Asare"),
    "sn-jnl author frontmatter missing:\n{x}"
  );
}

/// Same defect via the *environment* form `\begin{abstract}…\end{abstract}`
/// (rather than the braced `\abstract{…}`). In LaTeXML the abstract environment
/// is NOT backed by the raw-LaTeX `\abstract`/`\endabstract` pair — the kernel
/// binds `\begin{abstract}`→`\lx@begin@abstract` and `\end{abstract}`→
/// `\lx@end@abstract` directly (latex_constructs L5147-5148), both deferred — so
/// the env form routes to the same frontmatter accumulator as the braced form and
/// must likewise land after the title. Removing the binding's `{abstract}`
/// override could only *regress* this if it clobbered those kernel bindings; it
/// does not, and this guard proves it. arXiv/html_feedback#3436.
#[test]
fn frontmatter_sn_jnl_abstract_env_after_title() {
  let x = convert_to_xml_contrib_clean(
    "tests/cluster_regressions/frontmatter_sn_jnl_abstract_env_order.tex",
  );
  let title = x.find("<title>").expect("document <title> present");
  let abstract_ = x.find("<abstract").expect("<abstract> present");
  assert!(
    title < abstract_,
    "sn-jnl env-form abstract must render AFTER the title, not before it \
     (title@{title}, abstract@{abstract_}):\n{x}"
  );
  assert!(
    x.contains("This is the env-form abstract."),
    "sn-jnl env-form abstract body missing:\n{x}"
  );
  assert!(
    x.contains("<personname>") && x.contains("Asare"),
    "sn-jnl env-form author frontmatter missing:\n{x}"
  );
}

/// LNCS `llncs.cls`: several authors all bound to ONE `\institute` via
/// `\inst{1}`, whose body is the shared affiliation line followed by a single
/// shared `\email{\{a,b\}@host}` covering every author. Both the affiliation
/// AND the email are `\lx@annotate@frontmatter` contacts that must attach to
/// EACH creator carrying the `affiliation:1` label (the email inherits that
/// label from the enclosing pending affiliation entry, Perl
/// `Base_Utility.pool.ltxml` L498-500), not land on a single author. And the
/// pending affiliation stub must not surface as an empty
/// `<contact role="affiliation"/>`. arXiv/html_feedback#6881, witness 2608.11332.
#[test]
fn frontmatter_llncs_institute_shared_email() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_llncs_shared_email.tex");
  assert!(
    x.matches("<personname>").count() >= 2,
    "llncs: expected 2 author personnames:\n{x}"
  );
  assert!(
    x.contains("Alice Smith") && x.contains("Bob Jones"),
    "llncs author names missing:\n{x}"
  );
  // The shared affiliation attaches to BOTH authors...
  assert!(
    x.matches(">Some University</contact>").count() >= 2,
    "llncs shared affiliation did not attach to both authors:\n{x}"
  );
  // ...and so does the shared email...
  assert!(
    x.matches("role=\"email\"").count() >= 2,
    "llncs shared email did not attach to both authors:\n{x}"
  );
  assert!(
    x.matches(">{alice,bob}@univ.edu</contact>").count() >= 2,
    "llncs shared email content missing on both authors:\n{x}"
  );
  // ...with no empty pending-stub affiliation leaking through.
  assert!(
    !x.contains("role=\"affiliation\"/>") && !x.contains("role=\"affiliation\"></contact>"),
    "llncs pending affiliation stub leaked as an empty contact:\n{x}"
  );
}

/// LNCS shape (a): well-formed `\and`-separated institutions, each with an
/// internal `\\` break between its name and its own `\email`. Each author's
/// `\inst{N}` binds it to the N-th institution; the per-institution email
/// inherits that affiliation's label. Guards that the `\and`-split path stays
/// faithful beside the single-institute fix. Witness 2605.16562.
#[test]
fn frontmatter_llncs_and_separated_institutes() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_llncs_and_institutes.tex");
  assert!(
    x.matches("<personname>").count() >= 2,
    "llncs \\and: expected 2 author personnames:\n{x}"
  );
  assert!(
    x.contains("University of Foo") && x.contains("Institute of Bar"),
    "llncs \\and affiliations dropped:\n{x}"
  );
  // Each institution's email lands (one per author), so both addresses appear as
  // structured emails and neither institution is split off the wrong author.
  assert!(
    x.matches("role=\"email\"").count() >= 2
      && x.contains(">alice@foo.edu</contact>")
      && x.contains(">bob@bar.edu</contact>"),
    "llncs \\and per-institution emails not structured onto both authors:\n{x}"
  );
  assert!(
    !x.contains("role=\"affiliation\"/>"),
    "llncs \\and split leaked an empty affiliation stub:\n{x}"
  );
}

/// LNCS shape (c): ONE block, no `\and`, hand-typed `$^N$` superscript labels
/// with a `\quad` separator — the lazy multi-institution form the shared
/// `\lx@add@affiliations` parser splits by superscript (a beyond-Perl
/// improvement; upstream dumps the whole block on the first author only). The
/// superscript-presence dispatch must still route here, NOT to the single-
/// affiliation path. Witness 2606.19939.
#[test]
fn frontmatter_llncs_lazy_superscript_institutes() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_llncs_lazy_superscripts.tex");
  assert!(
    x.matches("<personname>").count() >= 2,
    "llncs lazy: expected 2 author personnames:\n{x}"
  );
  // Both institutions survive as affiliation content (the block was split, not
  // collapsed into one creator or dropped).
  assert!(
    x.contains("University of Foo") && x.contains("Institute of Bar"),
    "llncs lazy superscript institutions dropped:\n{x}"
  );
  assert!(
    x.contains("role=\"affiliation\""),
    "llncs lazy affiliations are not structured contacts:\n{x}"
  );
}

/// `sn-jnl.cls` with **numbered** `\author[N]` / `\affil[N]`: both affiliations
/// render as structured `<contact role="affiliation">` (matched to authors by
/// the id label), not orphaned notes. Guards the general numbered form beside
/// the shared form above.
#[test]
fn frontmatter_sn_jnl_numbered_affil() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/frontmatter_sn_jnl_affil_numbered.tex");
  assert!(
    x.contains("University of Foo") && x.contains("University of Bar"),
    "sn-jnl numbered affiliation content dropped:\n{x}"
  );
  assert!(
    x.contains("<contact") && x.contains("role=\"affiliation\""),
    "sn-jnl numbered affiliations are not structured contacts:\n{x}"
  );
  assert!(
    !x.contains("<note role=\"affiliation\""),
    "sn-jnl numbered affiliation leaked as an orphaned top-level note:\n{x}"
  );
}

/// A single creator whose authors cite the same institution via different
/// `\inst` lists (`Alice\inst{1}` + `Bob\inst{2,1}`, both citing institution 1)
/// must receive that affiliation ONCE, not once per citing author. The dedup
/// lives in `relocate_annotations` (base_utilities.rs): duplicate labels within
/// a creator's `_annotations` previously cloned the affiliation contact per
/// citation. Witness arXiv 2603.23669 (html_feedback frontmatter report): the
/// two-author creators rendered "Mila … Mila … McGill … McGill".
#[test]
fn frontmatter_inst_affiliation_dedup() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_inst_affiliation_dedup.tex");
  let mila = x.matches(">Mila Institute").count();
  assert_eq!(
    mila, 1,
    "affiliation cited by two authors was not deduped (got {mila}× Mila Institute):\n{x}"
  );
  // Both distinct institutions still present, as structured affiliation contacts.
  assert!(
    x.contains(">Uni Montreal") && x.contains("role=\"affiliation\""),
    "the second (distinct) affiliation was dropped:\n{x}"
  );
}

/// A figure injected into the title via `\g@addto@macro\@maketitle{…}` must
/// survive and register its `\label`.
///
/// LaTeXML redefines `\maketitle` to deposit its own captured frontmatter and
/// then `\global\let\@maketitle\relax` — discarding `\@maketitle` wholesale (the
/// source even notes "we can't yet emulate that"). So content a document appends
/// to `\@maketitle` (a teaser figure, an epigraph) was silently dropped, and any
/// `\ref` to a `\label` inside it rendered the raw internal key
/// "LABEL:fig:teaser". Both engines shared this blind spot (Perl drops it too).
///
/// Fix (surpass-Perl, OXIDIZED_DESIGN #124, KNOWN_PERL_ERRORS #90): `\@maketitle`
/// is predefined empty (so `\g@addto@macro` appends cleanly) and `\maketitle`
/// now deposits its accumulated content in a title-neutralized group before
/// relaxing it. Witness arXiv:2506.23854 (html_feedback#4281).
#[test]
fn frontmatter_maketitle_injected_figure_survives() {
  let x = convert_to_xml("tests/cluster_regressions/maketitle_injected_figure.tex");
  // The injected figure now exists AND carries its label (before the fix there
  // was no teaser figure at all, hence no `labels="LABEL:fig:teaser"`).
  assert!(
    x.contains(r#"labels="LABEL:fig:teaser""#),
    "the figure injected into \\@maketitle was dropped (no teaser figure/label):\n{x}"
  );
  assert!(
    x.contains("Teaser caption for the drums scene."),
    "the injected figure's caption was lost:\n{x}"
  );
  // The graphics candidate rode along too.
  assert!(
    x.contains("teaser.png"),
    "the injected figure's graphics were lost:\n{x}"
  );
}

/// `titlepic` variant of the injected-figure case above. `titlepic.sty` does not
/// APPEND to `\@maketitle` (`\g@addto@macro`); it stores its argument in
/// `\@titlepic` and REDEFINES `\@maketitle` wholesale (`\renewcommand`) to inject
/// `{\centering\@titlepic\par}` between the author block and the abstract. That is
/// a different path into the same machinery OXIDIZED_DESIGN #124 repaired, so it
/// gets its own guard: the teaser `\captionof{figure}`+`\label` must survive,
/// register its label, and take figure number 1 — otherwise every `\ref` renders
/// the raw "LABEL:fig:…" and the real second figure shifts to Figure 1.
///
/// New witness arXiv:2606.25280 (html_feedback#6675). The production ar5iv (Perl)
/// still drops it — this is a Rust-supersedes-Perl behavior, locked here.
#[test]
fn frontmatter_titlepic_redefined_maketitle_figure_survives() {
  let x = convert_to_xml("tests/cluster_regressions/frontmatter_titlepic_teaser_figure.tex");
  // Teaser figure survived and carries its label (dropped before #124).
  assert!(
    x.contains(r#"labels="LABEL:fig:teaser""#),
    "titlepic teaser figure dropped (no label):\n{x}"
  );
  assert!(
    x.contains("Teaser flock caption"),
    "titlepic teaser caption lost:\n{x}"
  );
  // Numbered as the first figure (xml:id S0.F1) — so the body `\ref{fig:teaser}`
  // resolves to "Figure 1", not the raw key, and the real figure stays Figure 2.
  assert!(
    x.contains(r#"labels="LABEL:fig:teaser" xml:id="S0.F1""#),
    "titlepic teaser figure not numbered first (expected xml:id S0.F1):\n{x}"
  );
  assert!(
    x.contains(r#"labels="LABEL:fig:second""#),
    "the real second figure lost its label:\n{x}"
  );
}
/// arXiv/html_feedback#1396 (+ the fairmeta.cls family: #662/#3512/#4707/#4971/
/// #5035/#5466). fairmeta.cls — the Meta/FAIR pre-print template — links authors
/// to institutions by superscript marks: `\author[1,2,*]{Name}`,
/// `\affiliation[1]{Inst}`, `\contribution[*]{Note}`. The shared meta-class
/// binding used to DROP the `[mark]` optarg, so every author lost its institution
/// and affiliations became detached document notes (the reported defect). The fix
/// routes the marks through LaTeXML's author-annotation / contact-label plan (the
/// authblk idiom, `\lx@add@creator[annotations]` + `\lx@add@contact[label]`), so
/// each institution/contribution attaches to the authors that cite its mark.
/// Verified byte-identical to Perl LaTeXML; covers the two sibling classes
/// (selfevolagent/openmoss) via `meta_class::install_meta_class_frontmatter`.
#[test]
fn frontmatter_fairmeta_author_affiliation_1396() {
  let x = convert_to_xml_contrib("tests/cluster_regressions/fairmeta_author_affiliation_1396.tex");
  // Four author creators, none leaked as a `[` bracket (the #4707/#5466 canary).
  assert_eq!(
    x.matches("role=\"author\"").count(),
    4,
    "expected 4 author creators:\n{x}"
  );
  assert!(
    !x.contains("<personname>[</personname>"),
    "an author name leaked as a `[` optarg bracket:\n{x}"
  );
  // Each institution attaches to exactly the authors that cite its mark…
  assert!(
    creator_block_contains(&x, "Alex Havrilla", ">Meta<"),
    "Alex Havrilla (mark 1) lost affiliation Meta:\n{x}"
  );
  assert!(
    creator_block_contains(&x, "Alex Havrilla", "Georgia Institute of Technology"),
    "Alex Havrilla (mark 2) lost affiliation Georgia Institute of Technology:\n{x}"
  );
  assert!(
    creator_block_contains(&x, "Yuqing Du", "UC Berkeley"),
    "Yuqing Du (mark 4) lost affiliation UC Berkeley:\n{x}"
  );
  assert!(
    creator_block_contains(&x, "Maksym Zhuravinskyi", "StabilityAI"),
    "Maksym Zhuravinskyi (mark 3) lost affiliation StabilityAI:\n{x}"
  );
  assert!(
    creator_block_contains(&x, "Eric Hambro", ">Meta<"),
    "Eric Hambro (mark 1) lost affiliation Meta:\n{x}"
  );
  // …contributions attach to their authors by symbolic mark (#4707 "credit
  // assignment")…
  assert!(
    creator_block_contains(&x, "Alex Havrilla", "Work done during Meta internship"),
    "Alex Havrilla (mark *) lost its contribution note:\n{x}"
  );
  assert!(
    creator_block_contains(&x, "Eric Hambro", "Work done while at Meta"),
    "Eric Hambro (mark **) lost its contribution note:\n{x}"
  );
  // …and marks are no longer dropped: Yuqing (mark 4 only) must NOT carry Meta,
  // and no affiliation is left as a detached document-level note.
  assert!(
    !creator_block_contains(&x, "Yuqing Du", ">Meta<"),
    "Yuqing Du (mark 4) wrongly carries Meta — marks not honored:\n{x}"
  );
}
