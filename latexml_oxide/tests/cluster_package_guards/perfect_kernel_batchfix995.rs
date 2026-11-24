//! Red/green guards for issue 995: an image reached outside the source directory (`\subimport*{../children/…}`).
//! `--sitedirectory` keeps its source-relative path when that stays in the site directory and a destination outside it
//! is Fatal (Perl `Post/Graphics.pm` L292-300, `Post.pm` L698-706); a generated `xN` name and a converted image's
//! stem are handed out once per run, not once per split page (Perl's document cache, `Post.pm` L192-200, L687-690).
//! A copy is written at its lexical path, never through a link out of the site directory; a converted image is reused
//! by every page; the parallel page render names images as the serial one does.
//! `graphics.rs` `ResourceNames`, `copy_destination`, `reserve_names`; OXIDIZED_DESIGN_DIVERGENCES #1081, #1157.
use std::{path::Path, process::Command};

use super::perfect_kernel_batch46::{error_count, warning_count};

const SITE: &str =
  include_str!("../../../tools/perfect_kernel/repros/loader/image_subimport_sitedirectory.tex");
const SPLIT: &str = include_str!(
  "../../../tools/perfect_kernel/repros/split-navigation/image_names_shared_across_split_pages.tex"
);
const REUSE: &str = include_str!(
  "../../../tools/perfect_kernel/repros/split-navigation/converted_image_reused_across_split_pages.tex"
);
#[cfg(unix)]
const LINKED: &str = include_str!(
  "../../../tools/perfect_kernel/repros/loader/image_copy_through_linked_destination.tex"
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

/// The parallel page render (64+ pages) names every image as the serial render does: a worker names the images of
/// the pages before its range first (`Graphics::reserve_names`). Round 1 gave a worker after the first `x<i>_N`, so the
/// names followed the fleet size (2 workers: S70 `x1_2.svg`); before it both workers wrote `x1.svg`, section 1
/// showing child_2's `A`. S40 reuses S1's copy across the worker boundary. `LATEXML_RENDER_JOBS` 1, 2 and 4.
#[test]
fn batchfix995_parallel_render_names_images_as_the_serial_render() {
  let mut tex =
    String::from("\\documentclass{article}\n\\usepackage{graphicx}\n\\begin{document}\n");
  for i in 1..=70 {
    let body = match i {
      1 => "\\includegraphics{../children/child_1/images/A}",
      40 => {
        "\\includegraphics{../children/child_1/images/A}\n\\includegraphics{../children/child_2/images/B}"
      },
      70 => "\\includegraphics{../children/child_2/images/A}",
      _ => "fine",
    };
    tex.push_str(&format!("\\section{{S{i}}}\n{body}\n\n"));
  }
  tex.push_str("\\end{document}\n");
  for jobs in ["1", "2", "4"] {
    let root = tempfile::tempdir().unwrap();
    layout(root.path(), &tex);
    let parent = root.path().join("parent");
    let log = convert(
      root.path(),
      &["--splitat=section", "--dest=index.html"],
      Some(jobs),
    );
    if jobs != "1" {
      assert!(
        log.contains(&format!("parallel page render engaged: {jobs} worker(s)")),
        "{log}"
      );
    }
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    assert_eq!(
      images_of(&parent.join("S1.html")),
      pairs(&[("x1.svg", "child_1/images/A")]),
      "jobs {jobs}"
    );
    assert_eq!(
      images_of(&parent.join("S40.html")),
      pairs(&[
        ("x1.svg", "child_1/images/A"),
        ("x2.svg", "child_2/images/B")
      ]),
      "jobs {jobs}"
    );
    assert_eq!(
      images_of(&parent.join("S70.html")),
      pairs(&[("x3.svg", "child_2/images/A")]),
      "jobs {jobs}"
    );
    assert_eq!(
      images_in(&parent),
      ["x1.svg", "x2.svg", "x3.svg"],
      "jobs {jobs}"
    );
  }
}

/// A converted image is reused by every split page (Perl's `cacheLookup`, `Post/Graphics.pm` L249-260): `fig.png`
/// turned on three chapters is one `x1.png`, `other.png` turned is `x2.png`, `other.png` as it is a copy (round 1:
/// `x1.png`, `x2.png`, `x3.png` holding one image, and `x4.png`). Skips visibly without ImageMagick. Repro
/// split-navigation/converted_image_reused_across_split_pages.
#[test]
fn batchfix995_split_pages_reuse_a_converted_image() {
  if !Command::new("convert")
    .arg("-version")
    .output()
    .is_ok_and(|o| o.status.success())
  {
    eprintln!("SKIP batchfix995_split_pages_reuse_a_converted_image: no ImageMagick");
    return;
  }
  let root = tempfile::tempdir().unwrap();
  let parent = root.path().join("parent");
  std::fs::create_dir_all(&parent).unwrap();
  std::fs::write(parent.join("index.tex"), REUSE).unwrap();
  std::fs::write(parent.join("fig.png"), png(3, 2)).unwrap();
  std::fs::write(parent.join("other.png"), png(5, 4)).unwrap();
  let log = convert(
    root.path(),
    &["--split", "--splitat=chapter", "--dest=index.html"],
    None,
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let urls = |page: &str| {
    images_of(&parent.join(page))
      .into_iter()
      .map(|(url, _)| url)
      .collect::<Vec<_>>()
  };
  assert_eq!(urls("Ch1.html"), ["x1.png", "other.png"]);
  assert_eq!(urls("Ch2.html"), ["x1.png"]);
  assert_eq!(urls("Ch3.html"), ["x1.png", "x2.png"]);
  assert_eq!(images_in(&parent), [
    "fig.png",
    "other.png",
    "x1.png",
    "x2.png"
  ]);
}

/// A `w`×`h` gray PNG (stored deflate blocks, so no encoder is needed).
fn png(w: u32, h: u32) -> Vec<u8> {
  fn chunk(out: &mut Vec<u8>, kind: &[u8], data: &[u8]) {
    out.extend((data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend(kind);
    out.extend(data);
    let crc = out[start..].iter().fold(!0u32, |mut crc, &byte| {
      crc ^= u32::from(byte);
      for _ in 0..8 {
        crc = if crc & 1 == 1 {
          (crc >> 1) ^ 0xEDB8_8320
        } else {
          crc >> 1
        };
      }
      crc
    });
    out.extend((!crc).to_be_bytes());
  }
  let raw: Vec<u8> = (0..h)
    .flat_map(|y| std::iter::once(0).chain((0..w).map(move |x| ((x + y) * 40) as u8)))
    .collect();
  let (mut a, mut b) = (1u32, 0u32);
  for &byte in &raw {
    a = (a + u32::from(byte)) % 65521;
    b = (b + a) % 65521;
  }
  let mut zlib = vec![0x78, 0x01, 0x01];
  zlib.extend((raw.len() as u16).to_le_bytes());
  zlib.extend((!(raw.len() as u16)).to_le_bytes());
  zlib.extend(&raw);
  zlib.extend(((b << 16) | a).to_be_bytes());
  let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
  let mut header = Vec::new();
  header.extend(w.to_be_bytes());
  header.extend(h.to_be_bytes());
  header.extend([8, 0, 0, 0, 0]);
  chunk(&mut out, b"IHDR", &header);
  chunk(&mut out, b"IDAT", &zlib);
  chunk(&mut out, b"IEND", &[]);
  out
}

/// `--sitedirectory` with a destination reached through a symbolic link (`site/out` → `elsewhere/deep/out`): the copy
/// is written at its lexical path (Perl's `pathname_copy` canonicalizes first), so `../children/c.svg` from `site/out`
/// is the source itself and nothing is written (round 1: `elsewhere/deep/children/c.svg`, the system resolving `..`
/// through the link); and a link inside the destination leading out of both (`out/sub` → `outside`) takes a generated
/// name instead of writing through it (Perl writes through; OXIDIZED_DESIGN_DIVERGENCES #1081). Repro
/// loader/image_copy_through_linked_destination.
#[cfg(unix)]
#[test]
fn batchfix995_linked_destination_writes_nothing_outside_the_site_directory() {
  let root = tempfile::tempdir().unwrap();
  let (site, elsewhere) = (root.path().join("site"), root.path().join("elsewhere"));
  for dir in [
    "site/src/sub",
    "site/children",
    "elsewhere/deep/out",
    "outside",
  ] {
    std::fs::create_dir_all(root.path().join(dir)).unwrap();
  }
  std::os::unix::fs::symlink(elsewhere.join("deep/out"), site.join("out")).unwrap();
  std::os::unix::fs::symlink(root.path().join("outside"), elsewhere.join("deep/out/sub")).unwrap();
  std::fs::write(site.join("children/c.svg"), svg("children/c")).unwrap();
  std::fs::write(site.join("src/sub/d.svg"), svg("src/sub/d")).unwrap();
  std::fs::write(site.join("src/index.tex"), LINKED).unwrap();
  let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .arg(format!("--sitedirectory={}", site.display()))
    .arg(format!("--dest={}", site.join("out/index.html").display()))
    .args(["--nocomments", "--timeout=110", "index.tex"])
    .current_dir(site.join("src"))
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let urls: Vec<String> = images_of(&site.join("out/index.html"))
    .into_iter()
    .map(|(url, _)| url)
    .collect();
  assert_eq!(urls, ["../children/c.svg", "x1.svg"]);
  assert!(!elsewhere.join("deep/children").exists(), "{log}");
  assert_eq!(
    std::fs::read_dir(root.path().join("outside"))
      .unwrap()
      .count(),
    0
  );
  assert_eq!(images_in(&elsewhere.join("deep/out")), ["x1.svg"]);
}
