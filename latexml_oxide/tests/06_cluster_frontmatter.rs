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
/// IEEEtran `\IEEEmembership{Senior Member, IEEE}` inside a flat comma author
/// list must not become a phantom "Senior Member, IEEE" creator. Witness
/// 2508.00603.
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
