//! Per-class frontmatter binding guards.
//!
//! Auto-consolidated test binary: each former file is an inline `mod`
//! below, body preserved verbatim, merged into one link unit for CI
//! economy. All members are subprocess- or few-conversion tests, so
//! co-locating them in one process stays far under the RSS fuse.

mod cluster;

mod fairmeta_frontmatter {
  //! Regression test: the `fairmeta.cls` (FAIR / Meta pre-print class) binding
  //! captures the class's custom frontmatter into the XML.
  //!
  //! `fairmeta.cls` defines its whole frontmatter interface
  //! (\author/\affiliation/\contribution/\metadata/\correspondence/\abstract) in
  //! the class BODY, which OmniBus does not raw-load — so without a binding those
  //! commands are `Error:undefined` and the metadata is silently dropped
  //! (ar5iv #520/#567/#576). The `fairmeta_cls` contrib binding routes them
  //! through `\@add@frontmatter` / `\lx@add@abstract`, and links `\author`/
  //! `\affiliation`/`\contribution` by their superscript MARK via LaTeXML's
  //! author-annotation plan (arXiv/html_feedback#1396; see
  //! `06_cluster_frontmatter::frontmatter_fairmeta_author_affiliation_1396`).
  //!
  //! Driven through the BINARY (fresh process, one conversion) rather than the
  //! in-process `tests/contrib` harness on purpose: loading a
  //! `LoadClass!("OmniBus")` contrib `.cls` and then `reset_thread_engine`-ing
  //! between files (as `can_contrib` does) reads a pre-reset `SymStr` from an
  //! unresettable `pin!` cache and aborts — the documented one-conversion-per-
  //! thread contract (see `latexml_core::reset_thread_engine`). A fresh process
  //! per conversion is exactly how production (cortex_worker) runs, so it is both
  //! faithful and safe. This is why the ~100 other contrib `.cls` bindings carry
  //! no in-process fixture.

  use std::{path::Path, process::Command};

  const FAIRMETA_TEX: &str = "\\documentclass{fairmeta}\n\
    \\title{A FAIR Preprint}\n\
    \\author[1,*]{Jane Doe}\n\
    \\author[2]{John Roe}\n\
    \\affiliation[1]{Meta AI}\n\
    \\affiliation[2]{Some University}\n\
    \\contribution[*]{Equal contribution}\n\
    \\metadata[Date]{January 2026}\n\
    \\correspondence{jane@example.org}\n\
    \\abstract{We study the frontmatter of a preprint class.}\n\
    \\begin{document}\n\
    \\maketitle\n\
    Body text with a \\nm{model name}.\n\
    \\beginappendix\n\
    \\section{Extra material}\n\
    More content.\n\
    \\end{document}\n";

  #[test]
  fn fairmeta_frontmatter_is_captured() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    let tex_path = workdir.path().join("fm.tex");
    std::fs::write(&tex_path, FAIRMETA_TEX).expect("write fm.tex");

    // No --preload/--includestyles: the class's tcolorbox[most] raw-loads its
    // library files (and their pgf dependencies) itself — that raw read turns
    // INCLUDE_STYLES on locally, and the hardened load guard (a deferred miss no
    // longer poisons a later load) lets pgfcore load in the nicematrix→tcolorbox
    // require order. Matches pdflatex; no ar5iv config needed.
    let output = Command::new(bin)
      .arg("fm.tex")
      .arg("--dest")
      .arg("fm.xml")
      .arg("--nocomments")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{stderr}",
      output.status.code(),
    );

    // The conversion must be error-clean: the whole point is that the custom
    // frontmatter commands are DEFINED (were Error:undefined before the binding).
    assert!(
      !stderr.contains("Error:") && !stderr.contains("Fatal:"),
      "expected an error-clean conversion, stderr had errors:\n{stderr}",
    );

    let xml = std::fs::read_to_string(workdir.path().join("fm.xml")).expect("read fm.xml");

    // Both authors survive as document creators (the class accumulates authors;
    // \lx@add@author must NOT collapse to the last one).
    for author in ["Jane Doe", "John Roe"] {
      assert!(
        xml.contains(author),
        "author {author} missing from frontmatter:\n{xml}"
      );
    }
    // Affiliations, contribution, date, correspondence land as frontmatter notes.
    for (role, text) in [
      ("affiliation", "Meta AI"),
      ("affiliation", "Some University"),
      ("contribution", "Equal contribution"),
      ("correspondence", "jane@example.org"),
    ] {
      assert!(
        xml.contains(&format!("role=\"{role}\"")),
        "missing frontmatter note role={role}:\n{xml}",
      );
      assert!(
        xml.contains(text),
        "missing frontmatter text {text:?}:\n{xml}"
      );
    }
    // \metadata[Date]{...} keys the note by its label.
    assert!(
      xml.contains("January 2026"),
      "metadata date value missing:\n{xml}"
    );
    // The superscript marks LINK author→institution/contribution (#1396): Jane Doe
    // [1,*] must carry Meta AI + the contribution, John Roe [2] Some University.
    let jane = xml
      .split("<creator")
      .find(|b| b.contains("<personname>Jane Doe</personname>"))
      .and_then(|b| b.split("</creator>").next())
      .unwrap_or("");
    assert!(
      jane.contains("Meta AI") && jane.contains("Equal contribution"),
      "Jane Doe [1,*] lost her linked Meta AI affiliation / contribution:\n{xml}"
    );
    assert!(
      !jane.contains("Some University"),
      "Jane Doe wrongly carries John's affiliation — marks not honored:\n{xml}"
    );
    // \abstract{...} reaches the abstract element.
    assert!(
      xml.contains("<abstract") && xml.contains("frontmatter of a preprint class"),
      "abstract not captured:\n{xml}",
    );
    // \beginappendix opens an appendix; \nm{...} passes its content through.
    assert!(
      xml.contains("<appendix"),
      "\\beginappendix did not open an appendix:\n{xml}"
    );
    assert!(xml.contains("model name"), "\\nm content missing:\n{xml}");
  }
}

mod selfevolagent_frontmatter {
  //! Regression test: the `selfevolagent.cls` binding captures the class's custom
  //! frontmatter into the XML (ar5iv #556).
  //!
  //! selfevolagent.cls is a near-identical sibling of fairmeta.cls — the same
  //! class-body `\addtolist`-based frontmatter interface, `Error:undefined`
  //! without a binding (an unknown `.cls` body is not raw-loaded). See
  //! `92_fairmeta_frontmatter.rs` for why this is driven through the binary rather
  //! than the in-process `tests/contrib` harness.
  //!
  //! (The full paper 2508.07407 also hits an unrelated box-loop `Stomach:Recursion`
  //! in its `paradigms.tex` content — a separate, paper-specific issue outside the
  //! frontmatter binding's scope.)

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{selfevolagent}\n\
    \\title{Self-Evolving Agents: A Survey}\n\
    \\author[1,*]{Ana Lee}\n\
    \\author[2]{Bo Chen}\n\
    \\affiliation[1]{EvoAgentX}\n\
    \\affiliation[2]{A University}\n\
    \\contribution[*]{Equal Contributor}\n\
    \\metadata[Github]{https://github.com/EvoAgentX/x}\n\
    \\correspondence{ana@example.org}\n\
    \\abstract{A survey of self-evolving agents.}\n\
    \\begin{document}\n\
    \\maketitle\n\
    Body text.\n\
    \\beginappendix\n\
    \\section{More}\n\
    Appendix content.\n\
    \\end{document}\n";

  #[test]
  fn selfevolagent_frontmatter_is_captured() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("s.tex"), TEX).expect("write s.tex");

    let output = Command::new(bin)
      .arg("s.tex")
      .arg("--dest")
      .arg("s.xml")
      .arg("--nocomments")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{stderr}",
      output.status.code(),
    );
    assert!(
      !stderr.contains("Error:") && !stderr.contains("Fatal:"),
      "expected an error-clean conversion, stderr had errors:\n{stderr}",
    );

    let xml = std::fs::read_to_string(workdir.path().join("s.xml")).expect("read s.xml");
    for author in ["Ana Lee", "Bo Chen"] {
      assert!(xml.contains(author), "author {author} missing:\n{xml}");
    }
    assert!(
      xml.contains("role=\"affiliation\"") && xml.contains("EvoAgentX"),
      "affiliation missing:\n{xml}"
    );
    assert!(
      xml.contains("role=\"contribution\""),
      "contribution missing:\n{xml}"
    );
    // The marks LINK author→affiliation/contribution (#1396): Ana Lee [1,*] carries
    // EvoAgentX + the contribution; Bo Chen [2] carries A University, not EvoAgentX.
    let ana = xml
      .split("<creator")
      .find(|b| b.contains("<personname>Ana Lee</personname>"))
      .and_then(|b| b.split("</creator>").next())
      .unwrap_or("");
    assert!(
      ana.contains("EvoAgentX") && ana.contains("Equal Contributor"),
      "Ana Lee [1,*] lost her linked affiliation / contribution:\n{xml}"
    );
    assert!(
      xml.contains("role=\"correspondence\""),
      "correspondence missing:\n{xml}"
    );
    // \metadata's arbitrary label renders as note CONTENT ("Github: …"), not a role.
    assert!(
      xml.contains("Github: https://github.com/EvoAgentX/x"),
      "metadata label:value missing:\n{xml}"
    );
    assert!(
      xml.contains("<abstract") && xml.contains("survey of self-evolving agents"),
      "abstract missing:\n{xml}"
    );
    assert!(
      xml.contains("<appendix"),
      "\\beginappendix did not open an appendix:\n{xml}"
    );
  }
}

mod openmoss_frontmatter {
  //! Regression test: the `openmoss.cls` binding captures the class's frontmatter
  //! and defines its colour helpers (ar5iv #605).
  //!
  //! openmoss.cls is a third sibling of fairmeta.cls / selfevolagent.cls — the same
  //! class-body `\addtolist` frontmatter plus an `openmossblue` colour used by the
  //! paper's `\textcolor{openmossblue}` / `\openmossblue{…}`. Without a binding the
  //! commands and colour are `Error:undefined` (an unknown `.cls` body is not
  //! raw-loaded). See `92_fairmeta_frontmatter.rs` for why this is binary-driven.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{openmoss}\n\
    \\title{World Action Models}\n\
    \\author[1,*]{Ann Poe}\n\
    \\affiliation[1]{OpenMOSS}\n\
    \\contribution[*]{Lead}\n\
    \\checkdata[Github Repo]{https://github.com/OpenMOSS/x}\n\
    \\correspondence{ann@example.org}\n\
    \\abstract{A survey of world action models.}\n\
    \\begin{document}\n\
    \\maketitle\n\
    \\textcolor{openmossblue}{blue text} and \\openmossblue{helper}.\n\
    \\beginappendix\n\
    \\section{More}\n\
    Appendix.\n\
    \\end{document}\n";

  #[test]
  fn openmoss_frontmatter_and_colors() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("o.tex"), TEX).expect("write o.tex");

    let output = Command::new(bin)
      .arg("o.tex")
      .arg("--dest")
      .arg("o.xml")
      .arg("--nocomments")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{stderr}",
      output.status.code(),
    );
    assert!(
      !stderr.contains("Error:") && !stderr.contains("Fatal:"),
      "expected an error-clean conversion, stderr had errors:\n{stderr}",
    );

    let xml = std::fs::read_to_string(workdir.path().join("o.xml")).expect("read o.xml");
    assert!(xml.contains("Ann Poe"), "author missing:\n{xml}");
    assert!(
      xml.contains("role=\"affiliation\"") && xml.contains("OpenMOSS"),
      "affiliation missing:\n{xml}"
    );
    assert!(
      xml.contains("role=\"correspondence\""),
      "correspondence missing:\n{xml}"
    );
    // \checkdata's arbitrary label ("Github Repo", with a space) renders as content.
    assert!(
      xml.contains("Github Repo: https://github.com/OpenMOSS/x"),
      "checkdata missing:\n{xml}"
    );
    assert!(
      xml.contains("<abstract") && xml.contains("survey of world action models"),
      "abstract missing:\n{xml}"
    );
    assert!(
      xml.contains("<appendix"),
      "\\beginappendix did not open an appendix:\n{xml}"
    );
    // The openmossblue colour resolved (not the "Can't find color" fallback).
    assert!(xml.contains("blue text"), "colored text missing:\n{xml}");
  }
}

mod agujournal_acronyms {
  //! Regression test: the agujournal2019.cls binding provides the end-matter
  //! `{acronyms}` / `{notation}` description lists and the `{sidewaystable}` float
  //! (ar5iv #538).
  //!
  //! agujournal2019.cls defines `{acronyms}`/`{notation}` in its (non-raw-loaded)
  //! body via `\def\acronyms{…\def\acro##1{\item[##1]}}`, and pulls in
  //! `{sidewaystable}` from `\RequirePackage{rotating}`. Without the binding
  //! covering these, `\acro`/`{acronyms}`/`{notation}`/`{sidewaystable}` were
  //! `Error:undefined` and the sideways `\caption` leaked ("outside any known
  //! float"). Binary-driven (`LoadClass!("OmniBus")`; see 92_fairmeta_frontmatter).

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{agujournal2019}\n\
    \\begin{document}\n\
    \\section{Body}\n\
    Text.\n\
    \\begin{sidewaystable}\n\
    \\caption{A wide table}\n\
    \\begin{tabular}{ll}a & b\\end{tabular}\n\
    \\end{sidewaystable}\n\
    \\begin{acronyms}\n\
    \\acro{CME} Coronal Mass Ejection\n\
    \\acro{ENA} Energetic Neutral Atom\n\
    \\end{acronyms}\n\
    \\begin{notation}\n\
    \\notation{r} radial distance\n\
    \\end{notation}\n\
    \\end{document}\n";

  #[test]
  fn agujournal_acronyms_notation_and_sideways() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("a.tex"), TEX).expect("write a.tex");

    let output = Command::new(bin)
      .arg("a.tex")
      .arg("--dest")
      .arg("a.xml")
      .arg("--nocomments")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{stderr}",
      output.status.code(),
    );
    assert!(
      !stderr.contains("Error:") && !stderr.contains("Fatal:"),
      "expected an error-clean conversion, stderr had errors:\n{stderr}",
    );

    let xml = std::fs::read_to_string(workdir.path().join("a.xml")).expect("read a.xml");
    // {acronyms}/{notation} render as description lists carrying their items.
    assert!(
      xml.matches("<description").count() >= 2,
      "expected 2 description lists (acronyms + notation):\n{xml}",
    );
    for token in [
      "CME",
      "Coronal Mass Ejection",
      "radial distance",
      "A wide table",
    ] {
      assert!(xml.contains(token), "missing {token:?}:\n{xml}");
    }
  }
}

mod scrartcl_titlehead {
  //! Regression test: the scrartcl (KOMA) binding captures `\titlehead` — a
  //! full-width header rendered ABOVE the title in KOMA's `\maketitle` — as
  //! frontmatter, instead of leaving it `Error:undefined` and dropping its
  //! content (ar5iv #498, arXiv:2305.01582).
  //!
  //! Neither Perl nor upstream LaTeXML binds `\titlehead` (no scrartcl.cls.ltxml),
  //! so both errored + dropped the banner; this is a surpass-Perl content
  //! recovery, a sibling of the existing `\subject`/`\publishers` frontmatter
  //! notes in `scrartcl_cls.rs`. Binary-driven because scrartcl `LoadClass!`es
  //! OmniBus (see `92_fairmeta_frontmatter` for why in-process aborts).

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{scrartcl}\n\
    \\newcommand\\giturl{github.com/example/repo}\n\
    \\titlehead{\\begin{center}\\emph{MySoftware} \\hspace{1in} \\giturl\\end{center}}\n\
    \\title{A KOMA Title}\n\
    \\author{Ann Poe}\n\
    \\begin{document}\n\
    \\maketitle\n\
    \\section{Body}\n\
    Text.\n\
    \\end{document}\n";

  #[test]
  fn scrartcl_titlehead_captured() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("s.tex"), TEX).expect("write s.tex");

    let output = Command::new(bin)
      .arg("s.tex")
      .arg("--dest")
      .arg("s.xml")
      .arg("--nocomments")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{stderr}",
      output.status.code(),
    );
    assert!(
      !stderr.contains("Error:") && !stderr.contains("Fatal:"),
      "expected an error-clean conversion (no \\titlehead undefined), stderr:\n{stderr}",
    );

    let xml = std::fs::read_to_string(workdir.path().join("s.xml")).expect("read s.xml");
    assert!(
      xml.contains("role=\"titlehead\""),
      "expected a role=titlehead frontmatter note:\n{xml}"
    );
    for token in ["MySoftware", "github.com/example/repo"] {
      assert!(
        xml.contains(token),
        "titlehead content {token:?} missing:\n{xml}"
      );
    }
  }
}

mod omnibus_agu_authors {
  //! An UNKNOWN class using the AGU `agujournal2019` author interface —
  //! `\authors{A\affil{1}, B\affil{2}, and C\affil{3}}` with numbered
  //! `\affiliation{n}{text}` blocks — must land as proper document creators +
  //! affiliation contacts, NOT a side-margin `<note role="authors">` with the
  //! affiliation bodies leaking into the flow (arXiv/html_feedback #6901, witness
  //! 2608.15512, `\documentclass[preprint]{agujournal2019}`).
  //!
  //! Fixed in OmniBus (the generic unknown-class fallback), so ANY unknown class
  //! with this interface benefits. An unknown class name (`foo`) resolves to
  //! OmniBus regardless of whether a `foo.cls` is on disk (`load_class` never
  //! raw-loads a class — it defaults `notex => true`), so its `\authors` /
  //! `\affil` / `\affiliation` bindings are exactly what run for the witness's
  //! `agujournal2019` too (that contrib binding `LoadClass!("OmniBus")`s and
  //! defines none of these itself).
  //!
  //! Resilient dispatch: `\affil{n}`/`\affiliation{n}{..}` take AGU's numbered
  //! superscript-ref path only when the label is all-digits; a non-numeric
  //! `\affil{Dept…}`/`\affiliation{Dept…}` (authblk/AAS convention) still routes
  //! to the plain affiliation-text form.
  use crate::cluster::convert_to_xml;

  #[test]
  fn agu_authors_become_creators_not_a_note() {
    // The fixture uses the AGU interface verbatim (witness 2608.15512): a
    // comma+"and"-separated \authors list, `\affil{n}` marks, `\affiliation{n}{}`
    // blocks, and a \thanks on the first author. `\documentclass{foo}` is an
    // unknown class → OmniBus (never raw-loaded), the same path the witness's
    // agujournal2019 contrib binding reaches via LoadClass!("OmniBus").
    let xml = convert_to_xml("tests/cluster_regressions/frontmatter_agu_authors.tex");

    // The four authors are SEPARATE document creators, each with a personname —
    // never one merged blob and never a side-margin note.
    assert!(
      !xml.contains("role=\"authors\""),
      "authors still land in a <note role=\"authors\"> instead of creators:\n{xml}"
    );
    let creators = xml.matches("role=\"author\"").count();
    assert_eq!(
      creators, 4,
      "expected 4 separate author creators, found {creators}:\n{xml}"
    );
    for name in [
      "Keshav Aggarwal",
      "R. K. Choudhary",
      "Abhirup Datta",
      "Anshuman Sharma",
    ] {
      assert!(xml.contains(name), "author {name:?} missing:\n{xml}");
    }
    // The literal joining word "and" before the last author is a separator, not
    // part of the name.
    assert!(
      !xml.contains("and Anshuman"),
      "the joining \"and\" leaked into the last author name:\n{xml}"
    );

    // Each numbered \affiliation{n}{text} is captured as an affiliation contact —
    // its text must NOT leak into the body flow as a bare paragraph.
    assert!(
      xml.contains("role=\"affiliation\""),
      "no affiliation contact captured:\n{xml}"
    );
    for aff in [
      "Indian Institute of Technology Indore",
      "Physical Research Laboratory",
      "Some Other Place",
    ] {
      assert!(
        xml.contains(aff),
        "affiliation text {aff:?} missing:\n{xml}"
      );
    }
    // The affiliation body leaked into a leading <p> in the buggy baseline; the
    // first institute text must sit inside frontmatter, never a standalone para.
    assert!(
      !xml.contains("<p>Indian Institute of Technology Indore"),
      "affiliation text leaked into the body flow as a paragraph:\n{xml}"
    );
    // The first author's \thanks survives as its own note on that creator.
    assert!(
      xml.contains("Corresponding author"),
      "first author's \\thanks note was dropped:\n{xml}"
    );
  }

  #[test]
  fn authblk_style_affiliation_text_still_works() {
    // Resilience: a non-numeric argument keeps the authblk/AAS convention where
    // the argument IS the affiliation text (`\affil{Dept…}`/`\affiliation{Dept…}`),
    // NOT a numbered superscript reference. `bar` is another unknown class →
    // OmniBus; the numeric dispatch must not disturb this common path.
    let xml = convert_to_xml("tests/cluster_regressions/frontmatter_authblk_affil.tex");
    // Both non-numeric affiliation texts are captured verbatim as affiliation
    // contacts — not swallowed as superscript labels or dropped.
    for text in ["Analytical Engine Institute", "Department of Computation"] {
      assert!(
        xml.contains(text),
        "authblk-style affiliation text {text:?} missing:\n{xml}"
      );
    }
    assert!(
      xml.contains("role=\"affiliation\""),
      "no affiliation contact for authblk-style text:\n{xml}"
    );
  }
}

mod omnibus_orcid_running_heads {
  //! sandbox-arxiv-2606 study (OXIDIZED_DESIGN #160): the safe slice of the
  //! OmniBus journal-frontmatter vocabulary extension. Perl's OmniBus leaves
  //! `\orcid` / `\lefttitle` / `\righttitle` undefined (→ Error), even though
  //! `\orcid` maps cleanly to the existing `\lx@add@orcid` helper and the
  //! running heads are presentational. We capture the ORCID (surpass-Perl,
  //! like `scrartcl`'s `\titlehead`) and no-op the running heads. `\orcid`
  //! appears both as `\orcid{id}` (pasj02, 2606.00213) and `\orcid[name]{id}`
  //! (aas-style), so the binding accepts the leading optional. Binary-driven
  //! because an unknown `.cls` `LoadClass!`es OmniBus (see 92_fairmeta_frontmatter).

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{someunknownjournal}\n\
    \\lefttitle{Left Running Head}\n\
    \\righttitle{Right Running Head}\n\
    \\title{A Study of Things}\n\
    \\author{Ada Lovelace\\orcid{0000-0002-2709-7338}}\n\
    \\begin{document}\n\
    \\maketitle\n\
    \\section{Body}\n\
    Text.\n\
    \\end{document}\n";

  #[test]
  fn omnibus_captures_orcid_and_drops_running_heads() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("a.tex"), TEX).expect("write a.tex");

    let output = Command::new(bin)
      .arg("a.tex")
      .arg("--dest")
      .arg("a.xml")
      .arg("--nocomments")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{stderr}",
      output.status.code(),
    );
    // \orcid / \lefttitle / \righttitle must no longer be undefined-errors.
    assert!(
      !stderr.contains("Error:") && !stderr.contains("Fatal:"),
      "expected an error-clean conversion, stderr had errors:\n{stderr}",
    );

    let xml = std::fs::read_to_string(workdir.path().join("a.xml")).expect("read a.xml");
    // The ORCID is captured as a real frontmatter contact, not dropped.
    assert!(
      xml.contains("role=\"orcid\"") && xml.contains("orcid.org/0000-0002-2709-7338"),
      "expected a captured ORCID contact:\n{xml}",
    );
    // The running heads are presentational — dropped, never leaked into the body.
    for leaked in ["Left Running Head", "Right Running Head"] {
      assert!(
        !xml.contains(leaked),
        "running-head {leaked:?} must be dropped, not emitted:\n{xml}",
      );
    }
  }
}
