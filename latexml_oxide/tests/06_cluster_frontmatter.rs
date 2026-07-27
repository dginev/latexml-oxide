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
