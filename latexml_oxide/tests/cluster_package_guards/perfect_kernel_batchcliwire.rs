//! CLIWIRE: `latexml_oxide`'s options as Perl's latexmlc applies them (Common/Config.pm, LaTeXML.pm
//! `convert_post`), each checked against latexmlc 0.8.8 on the same host. The options live in the binary's
//! argument handling, so these spawn it. One test per option group (OXIDIZED_DESIGN_DIVERGENCES #1158).

use std::{
  path::{Path, PathBuf},
  process::Command,
};

const DIR: &str = concat!(
  env!("CARGO_MANIFEST_DIR"),
  "/../tools/perfect_kernel/repros/cli-options"
);

/// A run of the binary in its own directory, holding the repro's `.tex` files.
struct Run {
  dir:    tempfile::TempDir,
  code:   Option<i32>,
  stderr: String,
}

impl Run {
  fn file(&self, name: &str) -> String {
    std::fs::read_to_string(self.dir.path().join(name))
      .unwrap_or_else(|e| panic!("read {name}: {e}"))
  }
  fn path(&self, name: &str) -> PathBuf { self.dir.path().join(name) }
  fn ok(&self) -> &Self {
    assert_eq!(self.code, Some(0), "{}", self.stderr);
    assert_eq!(super::error_count(&self.stderr), 0, "{}", self.stderr);
    self
  }
}

fn run_in(dir: tempfile::TempDir, args: &[&str], env: &[(&str, &str)]) -> Run {
  let out = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(args)
    .envs(env.iter().copied())
    .current_dir(dir.path())
    .output()
    .expect("spawn latexml_oxide");
  Run {
    dir,
    code: out.status.code(),
    stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
  }
}

/// `args` over a fresh copy of the repro directory.
fn run(args: &[&str]) -> Run {
  let dir = tempfile::tempdir().expect("tempdir");
  for name in ["sections.tex", "tokens.tex"] {
    std::fs::copy(Path::new(DIR).join(name), dir.path().join(name)).expect("stage the repro");
  }
  run_in(dir, args, &[])
}

fn ids(xml: &str) -> Vec<&str> {
  xml
    .split("xml:id=\"")
    .skip(1)
    .map(|rest| &rest[..rest.find('"').unwrap()])
    .collect()
}

fn warnings(stderr: &str) -> Vec<&str> {
  stderr
    .lines()
    .filter(|l| l.starts_with("Warning:"))
    .collect()
}

/// `--documentid` (Core.pm L48) is in State when Base_Schema defines `\thedocument@ID`: the root carries it and
/// every id is prefixed by it, Perl's ids exactly. latexml.sty's `\lxDocumentID` reaches the root the same way
/// (the `\begin{document}` constructor reads the expanded id tokens, Perl's `ToString($props{id})`).
#[test]
fn documentid_names_the_root_and_prefixes_every_id() {
  let r = run(&["sections.tex", "--dest=out.xml", "--documentid=DOCX"]);
  assert_eq!(ids(&r.ok().file("out.xml")), [
    "DOCX",
    "DOCX.S1",
    "DOCX.S1.p1",
    "DOCX.S1.E1",
    "DOCX.S1.E1.m1",
    "DOCX.S2",
    "DOCX.S2.p1"
  ]);
  let dir = tempfile::tempdir().expect("tempdir");
  std::fs::write(
    dir.path().join("lx.tex"),
    "\\documentclass{article}\n\\usepackage{latexml}\n\\lxDocumentID{FOO}\n\\begin{document}\nHi\n\\end{document}\n",
  )
  .unwrap();
  let r = run_in(dir, &["lx.tex", "--dest=lx.xml"], &[]);
  assert_eq!(ids(&r.ok().file("lx.xml")), ["FOO", "FOO.p1"]);
}

/// `--format=html` is html5 (Config.pm L436); `--format=xhtml` is LaTeXML-xhtml.xsl serialized as XML
/// (`is_html` is false, Writer.pm L47): a well-formed document with its prolog and doctype, as Perl's.
#[test]
fn format_html_is_html5_and_xhtml_is_well_formed_xml() {
  let html = run(&["sections.tex", "--format=html", "--dest=out.html"]);
  let html5 = run(&["sections.tex", "--format=html5", "--dest=out.html"]);
  assert_eq!(html.ok().file("out.html"), html5.ok().file("out.html"));
  assert!(
    html
      .file("out.html")
      .starts_with("<!DOCTYPE html><html lang=\"en\">")
  );

  let xhtml = run(&["sections.tex", "--format=xhtml", "--dest=out.xhtml"]);
  let text = xhtml.ok().file("out.xhtml");
  assert!(
    text.starts_with(
      "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.1 plus MathML \
       2.0//EN\" \"http://www.w3.org/Math/DTD/mathml2/xhtml-math11-f.dtd\">\n<html xmlns=\"http://www.w3.org/1999/xhtml\""
    ),
    "{text}"
  );
  if let Err(e) = libxml::parser::Parser::default().parse_string(&text) {
    panic!("--format=xhtml output is not well-formed XML: {e:?}");
  }
}

/// The serialization follows the format, never the stylesheet's name (Perl `is_html`, LaTeXML.pm L435): a custom
/// sheet named `my-html.xsl` writing `out.xml` keeps its XML prolog and `<p/>`, and any sheet writing `out.html`
/// serializes as HTML.
#[test]
fn stylesheet_output_is_serialized_by_the_format() {
  const SHEET: &str = "<?xml version=\"1.0\"?>\n<xsl:stylesheet version=\"1.0\" \
                       xmlns:xsl=\"http://www.w3.org/1999/XSL/Transform\" \
                       xmlns:ltx=\"http://dlmf.nist.gov/LaTeXML\"><xsl:output method=\"xml\"/>\
                       <xsl:template match=\"/\"><out><xsl:for-each \
                       select=\"//ltx:section\"><p/></xsl:for-each></out></xsl:template></xsl:stylesheet>\n";
  let as_xml = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<out xmlns:ltx=\"http://dlmf.nist.gov/LaTeXML\"><p/><p/></out>\n";
  for (sheet, dest, expect) in [
    ("my.xsl", "out.xml", as_xml),
    ("my-html.xsl", "out.xml", as_xml),
    (
      "my.xsl",
      "out.html",
      "<out xmlns:ltx=\"http://dlmf.nist.gov/LaTeXML\"><p></p><p></p></out>\n",
    ),
  ] {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::copy(
      Path::new(DIR).join("sections.tex"),
      dir.path().join("sections.tex"),
    )
    .unwrap();
    std::fs::write(dir.path().join(sheet), SHEET).unwrap();
    let r = run_in(
      dir,
      &[
        "sections.tex",
        &format!("--stylesheet={sheet}"),
        &format!("--dest={dest}"),
      ],
      &[],
    );
    assert_eq!(r.ok().file(dest), expect, "{sheet} -> {dest}");
  }
}

/// An explicit `--token-limit` is kept as given, as `LATEXML_TOKEN_LIMIT` is: the source size no longer raises a
/// limit below its scaled one (5000 `\textbf{x}` read far more than 1000 tokens).
#[test]
fn token_limit_below_the_default_holds() {
  let r = run(&["tokens.tex", "--dest=out.xml", "--token-limit=1000"]);
  assert!(
    r.stderr.contains("Fatal:Timeout:TokenLimit"),
    "{}",
    r.stderr
  );
}

/// `--quiet` reaches the parallel page render: the workers' Info records go to the log, not to the console
/// (70 pages, 4 workers; a 70-page run printed 211 Info lines under `--quiet`).
#[test]
fn quiet_silences_the_render_workers() {
  let dir = tempfile::tempdir().expect("tempdir");
  let mut tex = String::from("\\documentclass{article}\n\\begin{document}\n");
  for i in 0..70 {
    tex.push_str(&format!("\\section{{S{i}}}Text {i}.\n"));
  }
  tex.push_str("\\end{document}\n");
  std::fs::write(dir.path().join("many.tex"), tex).unwrap();
  let r = run_in(
    dir,
    &["many.tex", "--split", "--dest=out/index.html", "--quiet"],
    &[("LATEXML_RENDER_JOBS", "4")],
  );
  r.ok();
  let log = r.file("many.latexml.log");
  assert!(
    log.contains("parallel page render engaged: 4 worker(s)"),
    "{log}"
  );
  assert!(log.contains("Info:"), "{log}");
  let info: Vec<&str> = r
    .stderr
    .lines()
    .filter(|l| l.starts_with("Info:"))
    .collect();
  assert!(info.is_empty(), "{info:?}");
  assert!(r.path("out/S70.html").is_file());
}

/// `--splitat` and `--splitnaming` take a prefix of their values (Config.pm `_checkOptionValue`): `sec` splits
/// at sections, `labelrel` names the pages by label, as Perl does. An unknown naming is a usage error; `--splitat`
/// is checked only when splitting, and a level Perl does not list still splits there, with a Warning.
#[test]
fn split_options_take_a_prefix_of_their_values() {
  let pages = |r: &Run| {
    let mut names: Vec<String> = std::fs::read_dir(r.path("out"))
      .unwrap()
      .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
      .filter(|n| n.ends_with(".html"))
      .collect();
    names.sort();
    names
  };
  let r = run(&["sections.tex", "--splitat=sec", "--dest=out/index.html"]);
  assert_eq!(pages(r.ok()), ["S1.html", "S2.html", "index.html"]);
  assert!(warnings(&r.stderr).is_empty(), "{}", r.stderr);
  let r = run(&[
    "sections.tex",
    "--splitnaming=labelrel",
    "--dest=out/index.html",
  ]);
  assert_eq!(pages(r.ok()), [
    "index.html",
    "sec_intro.html",
    "sec_results.html"
  ]);

  let r = run(&[
    "sections.tex",
    "--splitnaming=bogus",
    "--dest=out/index.html",
  ]);
  assert_eq!(r.code, Some(2), "{}", r.stderr);
  assert!(r.stderr.contains(
    "Value for --splitnaming, bogus, doesn't match id, idrelative, label, labelrelative"
  ));

  let r = run(&[
    "sections.tex",
    "--nosplit",
    "--splitat=bogus",
    "--dest=out.html",
  ]);
  assert!(warnings(&r.ok().stderr).is_empty(), "{}", r.stderr);
  let r = run(&[
    "sections.tex",
    "--splitat=paragraph",
    "--dest=out/index.html",
  ]);
  assert_eq!(warnings(&r.ok().stderr), [
    "Warning:unexpected:splitat Value for --splitat, paragraph, doesn't match part, chapter, section, subsection, \
     subsubsection; splitting at //ltx:paragraph (--splitpath='//ltx:paragraph' says so)"
  ]);
}

/// `--navigationtoc` is lowercased and checked (Config.pm L472-480): `None` asks for no navigation TOC (the page is
/// the default's), `Context` for the context TOC, and an unknown style is a usage error.
#[test]
fn navigationtoc_is_lowercased_and_checked() {
  let page = |args: &[&str]| {
    let mut all = vec!["sections.tex", "--split", "--dest=out/index.html"];
    all.extend_from_slice(args);
    let r = run(&all);
    r.ok().file("out/S1.html")
  };
  let default = page(&[]);
  assert!(!default.contains("ltx_tocentry"), "{default}");
  assert_eq!(page(&["--navigationtoc=None"]), default);
  let context = page(&["--navtoc=Context"]);
  assert!(
    context.contains("ltx_page_navbar")
      && context.contains("ltx_tocentry ltx_tocentry_section ltx_ref_self"),
    "{context}"
  );
  let r = run(&[
    "sections.tex",
    "--split",
    "--navigationtoc=Bogus",
    "--dest=out/index.html",
  ]);
  assert_eq!(r.code, Some(2), "{}", r.stderr);
  assert!(
    r.stderr
      .contains("Bogus is not a recognized style of navigation TOC"),
    "{}",
    r.stderr
  );
}

/// Perl's spellings of the math options parse: `--keepXMath` is `--xmath`, `--nohackplane1` undoes
/// `--hackplane1` (`hackplane1!`).
#[test]
fn perl_spellings_of_math_options_parse() {
  let keep = run(&["sections.tex", "--keepXMath", "--post", "--dest=out.xml"]);
  let xmath = run(&["sections.tex", "--xmath", "--post", "--dest=out.xml"]);
  assert!(keep.ok().file("out.xml").contains("<XMath"));
  assert_eq!(keep.file("out.xml"), xmath.ok().file("out.xml"));
  let undone = run(&[
    "sections.tex",
    "--hackplane1",
    "--nohackplane1",
    "--dest=out.html",
  ]);
  let plain = run(&["sections.tex", "--dest=out.html"]);
  assert_eq!(undone.ok().file("out.html"), plain.ok().file("out.html"));
}

/// `--nonumbersections` keeps its documented meaning, the sections unnumbered and a reference showing the title
/// (OXIDIZED_DESIGN_DIVERGENCES #1160; Perl's code numbers them, KNOWN_PERL_ERRORS #1330); a later
/// `--numbersections` restores the numbers.
#[test]
fn nonumbersections_unnumbers_the_sections() {
  let off = run(&["sections.tex", "--nonumbersections", "--dest=out.html"]);
  let html = off.ok().file("out.html");
  assert!(!html.contains("ltx_tag_section"), "{html}");
  assert!(
    html.contains("<a href=\"#Sx2\" title=\"Results\" class=\"ltx_ref\"><span class=\"ltx_text ltx_ref_title\">Results</span></a>"),
    "{html}"
  );
  let on = run(&[
    "sections.tex",
    "--nonumbersections",
    "--numbersections",
    "--dest=out.html",
  ]);
  let plain = run(&["sections.tex", "--dest=out.html"]);
  assert_eq!(on.ok().file("out.html"), plain.ok().file("out.html"));
}

/// `--sourcedirectory` is made absolute before post uses it (Perl canonicalizes it, Config.pm L395-397): a
/// relative and an absolute spelling find the graphic and place it alike.
#[test]
fn sourcedirectory_relative_and_absolute_agree() {
  let png = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/alignment/foo.png");
  let outputs: Vec<(String, bool)> = ["relative", "absolute"]
    .into_iter()
    .map(|spelling| {
      let dir = tempfile::tempdir().expect("tempdir");
      std::fs::create_dir_all(dir.path().join("X")).unwrap();
      std::fs::create_dir_all(dir.path().join("S")).unwrap();
      std::fs::copy(png, dir.path().join("S/pic.png")).unwrap();
      std::fs::write(
        dir.path().join("X/doc.tex"),
        "\\documentclass{article}\n\\usepackage{graphicx}\n\\begin{document}\n\\includegraphics{pic}\n\\end{document}\n",
      )
      .unwrap();
      let core = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
        .args(["doc.tex", "--dest=doc.xml"])
        .current_dir(dir.path().join("X"))
        .output()
        .expect("spawn latexml_oxide");
      assert!(core.status.success());
      let source = if spelling == "relative" {
        "S".to_string()
      } else {
        dir.path().join("S").to_string_lossy().into_owned()
      };
      let r = run_in(
        dir,
        &["X/doc.xml", &format!("--sourcedirectory={source}"), "--dest=out/doc.html"],
        &[],
      );
      let html = r.ok().file("out/doc.html");
      let img = html[html.find("<img src=\"").expect("an image") ..].split('"').nth(1).unwrap().to_string();
      let copied = r.path("out").join(&img).is_file();
      (img, copied)
    })
    .collect();
  assert_eq!(outputs, [
    ("pic.png".to_string(), true),
    ("pic.png".to_string(), true)
  ]);
}
