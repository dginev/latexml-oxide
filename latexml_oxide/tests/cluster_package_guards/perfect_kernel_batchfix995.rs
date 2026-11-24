//! Red/green guards for issue 995: an image reached outside the source directory (`\subimport*{../children/…}`).
//! `--sitedirectory` keeps its source-relative path when that stays in the site directory and a destination outside it
//! is Fatal (Perl `Post/Graphics.pm` L292-300, `Post.pm` L698-706); a generated `xN` name and a converted image's
//! stem are handed out once per run, not once per split page (Perl's document cache, `Post.pm` L192-200, L687-690).
//! `graphics.rs` `ResourceNames`, `copy_to_destination`; OXIDIZED_DESIGN_DIVERGENCES #1081, #1157.
use std::{path::Path, process::Command};

use super::perfect_kernel_batch46::{error_count, warning_count};

const SITE: &str =
  include_str!("../../../tools/perfect_kernel/repros/loader/image_subimport_sitedirectory.tex");
const SPLIT: &str = include_str!(
  "../../../tools/perfect_kernel/repros/split-navigation/image_names_shared_across_split_pages.tex"
);

/// The issue's layout: `parent/index.tex` (`main`) and `children/child_{1,2}` with their `images/{A,B}.svg`, each
/// SVG holding its own path as its text.
fn layout(root: &Path, main: &str) {
  std::fs::create_dir_all(root.join("parent")).unwrap();
  std::fs::write(root.join("parent/index.tex"), main).unwrap();
  let children = [
    ("child_1", "\\includegraphics{images/A}\n", &["A"][..]),
    (
      "child_2",
      "\\includegraphics{images/A}\n\n\\includegraphics{images/B}\n",
      &["A", "B"][..],
    ),
  ];
  for (child, body, images) in children {
    let dir = root.join("children").join(child);
    std::fs::create_dir_all(dir.join("images")).unwrap();
    std::fs::write(dir.join("index.tex"), body).unwrap();
    for image in images {
      std::fs::write(
        dir.join(format!("images/{image}.svg")),
        svg(&format!("{child}/images/{image}")),
      )
      .unwrap();
    }
  }
}

fn svg(text: &str) -> String {
  format!(
    "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"100\" height=\"50\"><text x=\"5\" y=\"30\">{text}</text></svg>\n"
  )
}

/// Convert `parent/index.tex` with `parent/` as the working directory; the ANSI-stripped stderr.
fn convert(root: &Path, args: &[&str], jobs: Option<&str>) -> String {
  let mut command = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"));
  command
    .args(args)
    .args(["--nocomments", "--timeout=110", "index.tex"])
    .current_dir(root.join("parent"));
  if let Some(jobs) = jobs {
    command.env("LATEXML_RENDER_JOBS", jobs);
  }
  let output = command.output().expect("spawn latexml_oxide");
  String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "")
}

/// Each image a page shows, in order: (its URL, the text of the SVG it reaches, or the PNG's byte length).
fn images_of(page: &Path) -> Vec<(String, String)> {
  let html = std::fs::read_to_string(page).unwrap();
  let url = regex::Regex::new(r#"(?:data|src)="([^"]+\.(?:svg|png))""#).unwrap();
  let text = regex::Regex::new(r">([^<]+)</text>").unwrap();
  url
    .captures_iter(&html)
    .map(|cap| {
      let url = cap[1].to_string();
      let bytes = std::fs::read(page.parent().unwrap().join(&url)).unwrap_or_default();
      let content = std::str::from_utf8(&bytes)
        .ok()
        .and_then(|s| text.captures(s).map(|c| c[1].to_string()))
        .unwrap_or_else(|| format!("png {} bytes", bytes.len()));
      (url, content)
    })
    .collect()
}

/// The image files a run wrote into `dir` (not below it), sorted.
fn images_in(dir: &Path) -> Vec<String> {
  let mut out: Vec<String> = std::fs::read_dir(dir)
    .unwrap()
    .flatten()
    .map(|entry| entry.file_name().to_string_lossy().into_owned())
    .filter(|name| name.ends_with(".svg") || name.ends_with(".png"))
    .collect();
  out.sort();
  out
}

fn ghostscript_present() -> bool {
  Command::new(if cfg!(windows) { "gswin64c" } else { "gs" })
    .arg("--version")
    .output()
    .is_ok_and(|o| o.status.success())
}

/// `a/plot.eps` and `b/plot.eps` below `parent`: one stem, two drawings.
fn plots(parent: &Path) {
  for (dir, drawing) in [
    ("a", "0 0 moveto 20 20 lineto stroke"),
    ("b", "10 10 8 0 360 arc fill"),
  ] {
    std::fs::create_dir_all(parent.join(dir)).unwrap();
    std::fs::write(
      parent.join(dir).join("plot.eps"),
      format!("%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 20 20\n{drawing}\n%%EOF\n"),
    )
    .unwrap();
  }
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
  list
    .iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect()
}

/// `--sitedirectory=..` from `parent/` keeps `../children/child_N/images/*.svg` and copies nothing (base: `x1..x3.svg`,
/// the option ignored); without it the images are copied to `x1..x3.svg` inside `--dest`, as Perl does. Repro
/// loader/image_subimport_sitedirectory.
#[test]
fn batchfix995_sitedirectory_keeps_the_relative_image_path() {
  let root = tempfile::tempdir().unwrap();
  layout(root.path(), SITE);
  let parent = root.path().join("parent");
  let log = convert(
    root.path(),
    &["--sitedirectory=..", "--dest=index.html"],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    images_of(&parent.join("index.html")),
    pairs(&[
      ("../children/child_1/images/A.svg", "child_1/images/A"),
      ("../children/child_2/images/A.svg", "child_2/images/A"),
      ("../children/child_2/images/B.svg", "child_2/images/B"),
    ])
  );
  assert_eq!(images_in(&parent), Vec::<String>::new(), "{log}");

  let log = convert(root.path(), &["--dest=index.html"], None);
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    images_of(&parent.join("index.html")),
    pairs(&[
      ("x1.svg", "child_1/images/A"),
      ("x2.svg", "child_2/images/A"),
      ("x3.svg", "child_2/images/B"),
    ])
  );
  assert_eq!(images_in(&parent), ["x1.svg", "x2.svg", "x3.svg"]);
}

/// A destination outside `--sitedirectory` is Perl's Fatal, and nothing is written (base: converted, the option
/// ignored). Repro loader/image_subimport_sitedirectory.
#[test]
fn batchfix995_destination_outside_the_site_directory_is_fatal() {
  let root = tempfile::tempdir().unwrap();
  layout(root.path(), SITE);
  let log = convert(
    root.path(),
    &["--sitedirectory=../children", "--dest=index.html"],
    None,
  );
  let fatal: Vec<&str> = log.lines().filter(|l| l.starts_with("Fatal:")).collect();
  assert_eq!(
    fatal,
    [
      "Fatal:unexpected:. The destination directory (.) must be within the siteDirectory (../children)"
    ],
    "{log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  let parent = root.path().join("parent");
  assert!(!parent.join("index.html").exists(), "{log}");
  assert_eq!(images_in(&parent), Vec::<String>::new(), "{log}");
}

/// Split pages draw generated names from one sequence: Chapter One shows child_1's `A` as `x1.svg`, Chapter Two
/// child_2's as `x2.svg` (base: both `x1.svg`, which held child_2's — Chapter One showed the wrong image). Repro
/// split-navigation/image_names_shared_across_split_pages, its EPS lines dropped (the next guard has them).
#[test]
fn batchfix995_split_pages_share_the_generated_names() {
  let root = tempfile::tempdir().unwrap();
  layout(
    root.path(),
    &SPLIT
      .lines()
      .filter(|line| !line.contains("plot"))
      .collect::<Vec<_>>()
      .join("\n"),
  );
  let parent = root.path().join("parent");
  let log = convert(
    root.path(),
    &["--split", "--splitat=chapter", "--dest=index.html"],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    images_of(&parent.join("Ch1.html")),
    pairs(&[("x1.svg", "child_1/images/A")])
  );
  assert_eq!(
    images_of(&parent.join("Ch2.html")),
    pairs(&[
      ("x2.svg", "child_2/images/A"),
      ("x3.svg", "child_2/images/B")
    ])
  );
  assert_eq!(images_in(&parent), ["x1.svg", "x2.svg", "x3.svg"]);
}

/// Converted images sharing a stem on different split pages: `a/plot.eps` (Chapter One) keeps `plot.png`, `b/plot.eps`
/// (Chapter Two) takes the next generated name (base: both `plot.png`, Chapter One showing `b`'s drawing). Skips
/// visibly without Ghostscript. Repro split-navigation/image_names_shared_across_split_pages.
#[test]
fn batchfix995_split_pages_keep_converted_stems_apart() {
  if !ghostscript_present() {
    eprintln!("SKIP batchfix995_split_pages_keep_converted_stems_apart: no Ghostscript");
    return;
  }
  let root = tempfile::tempdir().unwrap();
  layout(root.path(), SPLIT);
  let parent = root.path().join("parent");
  plots(&parent);
  let log = convert(
    root.path(),
    &["--split", "--splitat=chapter", "--dest=index.html"],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let one = images_of(&parent.join("Ch1.html"));
  let two = images_of(&parent.join("Ch2.html"));
  let urls =
    |page: &[(String, String)]| page.iter().map(|(url, _)| url.clone()).collect::<Vec<_>>();
  assert_eq!(urls(&one), ["x1.svg", "plot.png"], "{log}");
  // Each page allocates its conversions before its copies (`Graphics::process` phase 1).
  assert_eq!(urls(&two), ["x3.svg", "x4.svg", "x2.png"], "{log}");
  assert_eq!(one[0].1, "child_1/images/A");
  assert_eq!(
    (two[0].1.as_str(), two[1].1.as_str()),
    ("child_2/images/A", "child_2/images/B")
  );
  let png = |name: &str| std::fs::read(parent.join(name)).unwrap();
  assert_ne!(
    png("plot.png"),
    png("x2.png"),
    "a/plot and b/plot are distinct drawings"
  );
}

/// The parallel page render (`LATEXML_RENDER_JOBS=2`, 64+ pages): the second worker cannot see the first's names, so
/// it generates `x1_N` (Perl's `resource_prefix`, OD #1157); without it both wrote `x1.svg`, section 1 showing child_2's
/// `A`. Section 1 shows child_1's `A`, section 70 child_2's.
#[test]
fn batchfix995_parallel_render_workers_generate_disjoint_names() {
  let root = tempfile::tempdir().unwrap();
  let mut tex =
    String::from("\\documentclass{article}\n\\usepackage{graphicx}\n\\begin{document}\n");
  for i in 1..=70 {
    let body = match i {
      1 => "\\includegraphics{../children/child_1/images/A}",
      70 => "\\includegraphics{../children/child_2/images/A}",
      _ => "fine",
    };
    tex.push_str(&format!("\\section{{S{i}}}\n{body}\n\n"));
  }
  tex.push_str("\\end{document}\n");
  layout(root.path(), &tex);
  let parent = root.path().join("parent");
  let log = convert(
    root.path(),
    &["--splitat=section", "--dest=index.html"],
    Some("2"),
  );
  assert!(
    log.contains("parallel page render engaged: 2 worker(s)"),
    "{log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert_eq!(
    images_of(&parent.join("S1.html")),
    pairs(&[("x1.svg", "child_1/images/A")])
  );
  assert_eq!(
    images_of(&parent.join("S70.html")),
    pairs(&[("x1_1.svg", "child_2/images/A")])
  );
  assert_eq!(images_in(&parent), ["x1.svg", "x1_1.svg"]);
}
