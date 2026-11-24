//! Red/green guards for issue 995: an image reached outside the source directory (`\subimport*{../children/…}`).
//! `--sitedirectory` keeps its source-relative path when that stays in the site directory and a destination outside it
//! is Fatal (Perl `Post/Graphics.pm` L292-300, `Post.pm` L698-706); a generated `xN` name and a converted image's
//! stem are handed out once per destination directory from Perl's resource cache (`Post.pm` L192-200, L1517-1548:
//! `LaTeXML-oxide.cache`), named for every page before any is rendered, so the serial and the parallel render agree, a
//! rerun keeps the names and a second document continues them. A copy is written at its lexical path, never through a
//! link out of the site directory; a converted image is reused by every page.
//! `graphics.rs` `Graphics::plan_names`, `resource_cache.rs`; OXIDIZED_DESIGN_DIVERGENCES #1081, #1157.
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
const NAV: &str = include_str!(
  "../../../tools/perfect_kernel/repros/split-navigation/image_names_title_images_in_navigation.tex"
);
const TWO_DOCS: &str = include_str!(
  "../../../tools/perfect_kernel/repros/loader/image_names_two_documents_one_folder.tex"
);
const EDITED: &str =
  include_str!("../../../tools/perfect_kernel/repros/loader/image_edited_keeps_its_name.tex");
const CORRUPT: &str =
  include_str!("../../../tools/perfect_kernel/repros/loader/image_names_corrupt_cache_entry.tex");
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

/// The tree a run wrote below `dir`: each file's path and bytes.
fn tree(dir: &Path) -> Vec<(String, Vec<u8>)> {
  fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
      let path = entry.path();
      if path.is_dir() {
        walk(root, &path, out);
      } else {
        let name = path
          .strip_prefix(root)
          .unwrap()
          .to_string_lossy()
          .into_owned();
        out.push((name, std::fs::read(&path).unwrap()));
      }
    }
  }
  let mut out = Vec::new();
  walk(dir, dir, &mut out);
  out.sort();
  out
}

/// The sources a page's images show (each `img` file matched by its bytes; `?url` when none matches).
fn shown(page: &Path, sources: &[(String, Vec<u8>)]) -> Vec<String> {
  let html = std::fs::read_to_string(page).unwrap();
  regex::Regex::new(r#"<img src="([^"]+)""#)
    .unwrap()
    .captures_iter(&html)
    .map(|cap| cap[1].to_string())
    .filter(|url| !url.starts_with("data:"))
    .map(|url| {
      let bytes = std::fs::read(page.parent().unwrap().join(&url)).unwrap_or_default();
      sources
        .iter()
        .find(|(_, source)| *source == bytes)
        .map_or(format!("?{url}"), |(name, _)| name.clone())
    })
    .collect()
}

/// 72 sections, an image in each title (which CrossRef copies into the neighbours' navigation) and one in each body:
/// at `LATEXML_RENDER_JOBS` 1, 2 and 4 the trees are byte-identical, `LaTeXML-oxide.cache` included, and every page shows
/// its own two images; a rerun into the same destination at another job count changes nothing (round 2, which had
/// each worker replay the pages before its range, missed the navigation copies: 13 pages showed another section's
/// image). The names are the page order's: S<k> shows `x(2k-1).png` and `x(2k).png`. Repro
/// split-navigation/image_names_title_images_in_navigation.
#[test]
fn batchfix995_every_job_count_names_images_alike() {
  let root = tempfile::tempdir().unwrap();
  let (src, img) = (root.path().join("src"), root.path().join("img"));
  std::fs::create_dir_all(&src).unwrap();
  std::fs::create_dir_all(&img).unwrap();
  std::fs::write(src.join("index.tex"), NAV).unwrap();
  let mut sources = Vec::new();
  for k in 1..=72u32 {
    for (kind, h) in [("t", 1), ("b", 2)] {
      let name = format!("{kind}{k}.png");
      std::fs::write(img.join(&name), png(k, h)).unwrap();
      sources.push((name, png(k, h)));
    }
  }
  let run = |out: &str, jobs: &str| {
    let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .args([
        "--nocomments",
        "--timeout=110",
        "--splitat=section",
        &format!("--dest={out}/index.html"),
        "index.tex",
      ])
      .env("LATEXML_RENDER_JOBS", jobs)
      .current_dir(&src)
      .output()
      .expect("spawn latexml_oxide");
    let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
    if jobs != "1" {
      assert!(
        log.contains(&format!("parallel page render engaged: {jobs} worker(s)")),
        "{log}"
      );
    }
    src.join(out)
  };
  let serial = run("o1", "1");
  for k in 1..=72u32 {
    let shown = shown(&serial.join(format!("S{k}.html")), &sources);
    for own in [format!("t{k}.png"), format!("b{k}.png")] {
      assert!(shown.contains(&own), "S{k} shows {shown:?}");
    }
    assert!(
      !shown.iter().any(|s| s.starts_with('?')),
      "S{k} shows {shown:?}"
    );
  }
  let html = std::fs::read_to_string(serial.join("S40.html")).unwrap();
  assert!(html.contains("src=\"x79.png\"") && html.contains("src=\"x80.png\""));
  let reference = tree(&serial);
  assert!(
    reference
      .iter()
      .any(|(name, _)| name == "LaTeXML-oxide.cache")
  );
  assert!(
    !reference
      .iter()
      .any(|(name, _)| name.starts_with("LaTeXML-oxide.cache-"))
  );
  for jobs in ["2", "4"] {
    assert!(
      tree(&run(&format!("o{jobs}"), jobs)) == reference,
      "jobs {jobs}"
    );
  }
  // A rerun into the serial run's destination, on four workers.
  assert!(tree(&run("o1", "4")) == reference, "the rerun");
}

/// Documents converted into one folder: `a.tex` (child_1) names its image `x1.svg`, then `b.tex` (child_2) continues
/// with `x2.svg` and `x3.svg` (base: both `x1.svg`, `a.html` showing child_2's image). child_1's image changed, `a.tex`
/// again writes it into `x1.svg`, the name its source has (round 3: `x4.svg`), leaving `b.html` right. Perl's
/// `LaTeXML-oxide.cache` counter (`Post.pm` L192-200), the key carrying the content digest in place of Perl's
/// timestamp check. Repro loader/image_names_two_documents_one_folder.
#[test]
fn batchfix995_documents_in_one_folder_keep_their_own_images() {
  let root = tempfile::tempdir().unwrap();
  layout(root.path(), "");
  let parent = root.path().join("parent");
  std::fs::write(parent.join("a.tex"), TWO_DOCS).unwrap();
  std::fs::write(parent.join("b.tex"), TWO_DOCS.replace("child_1", "child_2")).unwrap();
  let run = |doc: &str| {
    let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .args([
        "--nocomments",
        "--timeout=110",
        &format!("--dest={doc}.html"),
        &format!("{doc}.tex"),
      ])
      .current_dir(&parent)
      .output()
      .expect("spawn latexml_oxide");
    let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  };
  run("a");
  run("b");
  assert_eq!(
    images_of(&parent.join("a.html")),
    pairs(&[("x1.svg", "child_1/images/A")])
  );
  let b = pairs(&[
    ("x2.svg", "child_2/images/A"),
    ("x3.svg", "child_2/images/B"),
  ]);
  assert_eq!(images_of(&parent.join("b.html")), b);
  std::fs::write(
    root.path().join("children/child_1/images/A.svg"),
    svg("child_1/images/A changed"),
  )
  .unwrap();
  run("a");
  assert_eq!(
    images_of(&parent.join("a.html")),
    pairs(&[("x1.svg", "child_1/images/A changed")])
  );
  assert_eq!(images_of(&parent.join("b.html")), b);
}

/// Eight documents converted at one moment into a new folder, each with two images of its own, three times: every
/// process ends on `LaTeXML-oxide.cache` (creating it is retried while another holds it: the first round fell back to
/// memory in 3 of 5 trials, `database is locked`, and two pages showed one `x1.png`), every name is handed out once
/// (one `IMMEDIATE` transaction per plan), every page shows its own images, and no SQLite side file is left. The
/// processes wait on one start signal (a FIFO, on Unix). `sqlite_store.rs`, `resource_cache.rs`.
#[test]
fn batchfix995_concurrent_documents_in_one_folder_never_share_a_name() {
  for trial in 0..3 {
    concurrent_documents(&tempfile::tempdir().unwrap(), trial);
  }
}

fn concurrent_documents(root: &tempfile::TempDir, trial: usize) {
  let parent = root.path().join("parent");
  std::fs::create_dir_all(&parent).unwrap();
  for doc in 0..8 {
    let dir = root.path().join(format!("children/c{doc}"));
    std::fs::create_dir_all(&dir).unwrap();
    for image in ["A", "B"] {
      std::fs::write(
        dir.join(format!("{image}.svg")),
        svg(&format!("c{doc}/{image}")),
      )
      .unwrap();
    }
    std::fs::write(
      parent.join(format!("d{doc}.tex")),
      format!(
        "\\documentclass{{article}}\n\\usepackage{{graphicx}}\n\\begin{{document}}\n\
         \\includegraphics{{../children/c{doc}/A}}\n\\includegraphics{{../children/c{doc}/B}}\n\\end{{document}}\n"
      ),
    )
    .unwrap();
  }
  // The start signal: each process waits for its line on a FIFO the test holds open for writing, so none starts before
  // the last is spawned.
  #[cfg(unix)]
  let signal = {
    let fifo = root.path().join("start");
    assert!(
      Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .is_ok_and(|s| s.success())
    );
    let writer = std::fs::OpenOptions::new()
      .read(true)
      .write(true)
      .open(&fifo)
      .unwrap();
    (fifo, writer)
  };
  let children: Vec<_> = (0..8)
    .map(|doc| {
      let args = [
        "--nocomments".to_string(),
        "--timeout=110".to_string(),
        format!("--dest=d{doc}.html"),
        format!("d{doc}.tex"),
      ];
      #[cfg(unix)]
      let mut command = {
        let mut command = Command::new("sh");
        command
          .arg("-c")
          .arg("read go < \"$0\" && exec \"$@\"")
          .arg(&signal.0)
          .arg(env!("CARGO_BIN_EXE_latexml_oxide"));
        command
      };
      #[cfg(not(unix))]
      let mut command = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"));
      command
        .args(args)
        .current_dir(&parent)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("spawn latexml_oxide")
    })
    .collect();
  #[cfg(unix)]
  {
    use std::io::Write;
    std::thread::sleep(std::time::Duration::from_millis(300));
    (&signal.1).write_all(&b"go\n".repeat(8)).unwrap();
  }
  for child in children {
    let output = child.wait_with_output().unwrap();
    let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert_eq!(
      (error_count(&log), warning_count(&log)),
      (0, 0),
      "trial {trial}: {log}"
    );
  }
  let mut urls = Vec::new();
  for doc in 0..8 {
    let shown = images_of(&parent.join(format!("d{doc}.html")));
    assert_eq!(
      shown
        .iter()
        .map(|(_, text)| text.as_str())
        .collect::<Vec<_>>(),
      [format!("c{doc}/A"), format!("c{doc}/B")],
      "trial {trial}: d{doc}"
    );
    urls.extend(shown.into_iter().map(|(url, _)| url));
  }
  urls.sort();
  urls.dedup();
  assert_eq!(urls.len(), 16, "trial {trial}: {urls:?}");
  assert_eq!(images_in(&parent).len(), 16);
  let side_files: Vec<String> = std::fs::read_dir(&parent)
    .unwrap()
    .flatten()
    .map(|entry| entry.file_name().to_string_lossy().into_owned())
    .filter(|name| name.starts_with("LaTeXML-oxide.cache-"))
    .collect();
  assert!(side_files.is_empty(), "trial {trial}: {side_files:?}");
}

/// `LaTeXML-oxide.cache` and its SQLite side files stay out of a `.zip` destination and an EPUB (as Perl keeps its
/// `LaTeXML.cache` out: `Pack/Zip.pm` `ARCHIVE_EXT_EXCLUDE`, `Manifest/Epub.pm` L205), also when a render worker is
/// killed (`LATEXML_RENDER_WORKER_TEST_ABORT=signal`: the fleet is abandoned, the serial render redoes the pages, a
/// Warning): workers only look names up, the parent closes the cache before packing, and a side file left by a killed
/// process is excluded by name (`pack::tests`). Every image is packed.
#[test]
fn batchfix995_packages_leave_the_cache_out_when_a_worker_is_killed() {
  let root = tempfile::tempdir().unwrap();
  let (src, img) = (root.path().join("src"), root.path().join("img"));
  std::fs::create_dir_all(&src).unwrap();
  std::fs::create_dir_all(&img).unwrap();
  std::fs::write(src.join("index.tex"), NAV).unwrap();
  for k in 1..=72u32 {
    std::fs::write(img.join(format!("t{k}.png")), png(k, 1)).unwrap();
    std::fs::write(img.join(format!("b{k}.png")), png(k, 2)).unwrap();
  }
  for dest in ["t.zip", "t.epub"] {
    let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .args([
        "--nocomments",
        "--timeout=110",
        "--splitat=section",
        &format!("--dest={dest}"),
        "index.tex",
      ])
      .env("LATEXML_RENDER_JOBS", "2")
      .env("LATEXML_RENDER_WORKER_TEST_ABORT", "signal")
      .current_dir(&src)
      .output()
      .expect("spawn latexml_oxide");
    let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    assert_eq!(error_count(&log), 0, "{log}");
    let archive = zip::ZipArchive::new(std::fs::File::open(src.join(dest)).unwrap()).unwrap();
    let names: Vec<String> = archive
      .file_names()
      .filter_map(|name| name.ok().map(String::from))
      .collect();
    assert!(
      !names.iter().any(|n| n.contains("LaTeXML-oxide.cache")),
      "{dest}: {names:?}"
    );
    assert_eq!(
      names.iter().filter(|n| n.ends_with(".png")).count(),
      144,
      "{dest}: {names:?}"
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

fn imagemagick_present() -> bool {
  Command::new("convert")
    .arg("-version")
    .output()
    .is_ok_and(|o| o.status.success())
}

/// An edited image keeps its name: a copy at its kept path (`figs/a.png`), a converted EPS at its stem (`p.png`) and a
/// turned PNG at its generated `x1.png` are written again with the new pictures (round 3: `x1.png`, `x2.png`, `x3.png`,
/// `out/figs/a.png` keeping the old picture), as Perl copies a simply scaled image to its kept path every run
/// (`Post/Graphics.pm` L292-300). In place (`--dest` in the source folder) `figs/a.png` stays linked where it is, with
/// no copy of it. A name's owner is its source, not its bytes (`resource_cache.rs`). The EPS needs Ghostscript and the
/// turn ImageMagick: each is left out, visibly, where its tool is absent. Repro loader/image_edited_keeps_its_name.
#[test]
fn batchfix995_an_edited_image_keeps_its_name() {
  let (gs, im) = (ghostscript_present(), imagemagick_present());
  if !gs || !im {
    eprintln!(
      "SKIP part of batchfix995_an_edited_image_keeps_its_name: Ghostscript {gs}, ImageMagick {im}"
    );
  }
  let root = tempfile::tempdir().unwrap();
  let src = root.path().join("src");
  std::fs::create_dir_all(src.join("figs")).unwrap();
  let tex: Vec<&str> = EDITED
    .lines()
    .filter(|line| (gs || !line.contains("p.eps")) && (im || !line.contains("angle")))
    .collect();
  std::fs::write(src.join("p.tex"), tex.join("\n")).unwrap();
  let draw = |v: u32| {
    std::fs::write(src.join("figs/a.png"), png(3 + v, 2)).unwrap();
    std::fs::write(src.join("figs/r.png"), png(2, 3 + v)).unwrap();
    std::fs::write(
      src.join("figs/p.eps"),
      format!(
        "%!PS-Adobe-3.0 EPSF-3.0\n%%BoundingBox: 0 0 20 20\n{} setlinewidth 0 0 moveto 20 20 lineto stroke\n%%EOF\n",
        v * 4 + 1
      ),
    )
    .unwrap();
  };
  let mut expected = vec!["figs/a.png"];
  expected.extend(gs.then_some("p.png"));
  expected.extend(im.then_some("x1.png"));
  let generated: Vec<&str> = expected[1..].to_vec();
  for dest in ["out/p.html", "p.html"] {
    draw(0);
    let page = src.join(dest);
    let dir = page.parent().unwrap().to_path_buf();
    let run = || {
      let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
        .args([
          "--nocomments",
          "--timeout=110",
          &format!("--dest={dest}"),
          "p.tex",
        ])
        .current_dir(&src)
        .output()
        .expect("spawn latexml_oxide");
      let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
      assert_eq!(
        (error_count(&log), warning_count(&log)),
        (0, 0),
        "{dest}: {log}"
      );
      let urls: Vec<String> = images_of(&page).into_iter().map(|(url, _)| url).collect();
      assert_eq!(urls, expected, "{dest}: {log}");
      generated
        .iter()
        .map(|name| std::fs::read(dir.join(name)).unwrap())
        .collect::<Vec<_>>()
    };
    let first = run();
    draw(1);
    let second = run();
    for (name, (before, after)) in generated.iter().zip(first.iter().zip(&second)) {
      assert_ne!(before, after, "{dest}: {name} holds the edited image");
    }
    assert_eq!(
      std::fs::read(dir.join("figs/a.png")).unwrap(),
      png(4, 2),
      "{dest}: figs/a.png is the edited image"
    );
    assert_eq!(images_in(&dir), generated, "{dest}");
  }
  let mut figs: Vec<String> = std::fs::read_dir(src.join("figs"))
    .unwrap()
    .flatten()
    .map(|entry| entry.file_name().to_string_lossy().into_owned())
    .collect();
  figs.sort();
  assert_eq!(figs, ["a.png", "p.eps", "r.png"]);
}

/// A corrupt or planted `LaTeXML-oxide.cache` decides no file a run writes: entries set to `Ch1.html||` and
/// `.hidden/evil||` are names no PNG could have been given, so each is ignored with a Warning naming the cache and the
/// images are named again (round 3: a PNG written over `Ch1.html`, `.hidden/evil` created). Perl writes only names it
/// generates (`Post/Graphics.pm` L249-260 reuse an existing file). Repro loader/image_names_corrupt_cache_entry.
#[test]
fn batchfix995_a_corrupt_cache_entry_decides_no_file() {
  let root = tempfile::tempdir().unwrap();
  let (src, img) = (root.path().join("src"), root.path().join("img"));
  std::fs::create_dir_all(&src).unwrap();
  std::fs::create_dir_all(&img).unwrap();
  std::fs::write(src.join("index.tex"), CORRUPT).unwrap();
  std::fs::write(src.join("a.png"), png(4, 4)).unwrap();
  std::fs::write(img.join("b.png"), png(4, 5)).unwrap();
  std::fs::write(img.join("c.png"), png(5, 5)).unwrap();
  let run = || {
    let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .args([
        "--nocomments",
        "--timeout=110",
        "--split",
        "--splitat=chapter",
        "--dest=index.html",
        "index.tex",
      ])
      .current_dir(&src)
      .output()
      .expect("spawn latexml_oxide");
    String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "")
  };
  let shown = || {
    ["Ch1.html", "Ch2.html"]
      .map(|page| images_of(&src.join(page)))
      .to_vec()
  };
  let log = run();
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let before = shown();
  let urls: Vec<Vec<&str>> = before
    .iter()
    .map(|page| page.iter().map(|(url, _)| url.as_str()).collect())
    .collect();
  assert_eq!(urls, [vec!["a.png", "x1.png"], vec!["x2.png"]]);
  {
    let conn = rusqlite::Connection::open(src.join("LaTeXML-oxide.cache")).unwrap();
    let keys: Vec<String> = conn
      .prepare("SELECT key FROM kv WHERE key LIKE 'LaTeXML::Post::Graphics:%' ORDER BY key")
      .unwrap()
      .query_map([], |row| row.get(0))
      .unwrap()
      .collect::<Result<_, _>>()
      .unwrap();
    assert_eq!(keys.len(), 3, "{keys:?}");
    for (key, value) in keys.iter().zip(["Ch1.html||", ".hidden/evil||"]) {
      conn
        .execute("UPDATE kv SET value = ?1 WHERE key = ?2", [value, key])
        .unwrap();
    }
  }
  let log = run();
  let warnings: Vec<&str> = log.lines().filter(|l| l.starts_with("Warning:")).collect();
  assert_eq!(error_count(&log), 0, "{log}");
  assert_eq!(warnings.len(), 2, "{log}");
  assert!(
    warnings.iter().all(|w| w.contains("LaTeXML-oxide.cache")),
    "{warnings:?}"
  );
  assert_eq!(shown(), before);
  assert!(
    std::fs::read_to_string(src.join("Ch1.html"))
      .unwrap()
      .contains("<html")
  );
  assert!(!src.join(".hidden").exists());
}

/// `a.tex` and `b.tex` converted into one folder, `b.tex` meeting the cache `hold` makes unusable (Warning, then the
/// run in memory): `b.html` takes names whose files do not exist (`x2.svg`, `x3.svg`) and `a.html` keeps its image.
/// Repro loader/image_names_cache_unusable.
fn unusable_cache(hold: impl FnOnce(&Path) -> Option<Box<dyn FnOnce()>>) {
  let root = tempfile::tempdir().unwrap();
  layout(root.path(), "");
  let parent = root.path().join("parent");
  std::fs::write(parent.join("a.tex"), TWO_DOCS).unwrap();
  std::fs::write(parent.join("b.tex"), TWO_DOCS.replace("child_1", "child_2")).unwrap();
  let run = |doc: &str| {
    let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .args([
        "--nocomments",
        "--timeout=60",
        &format!("--dest={doc}.html"),
        &format!("{doc}.tex"),
      ])
      .current_dir(&parent)
      .output()
      .expect("spawn latexml_oxide");
    String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "")
  };
  let log = run("a");
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  let a = pairs(&[("x1.svg", "child_1/images/A")]);
  assert_eq!(images_of(&parent.join("a.html")), a);
  let Some(release) = hold(&parent.join("LaTeXML-oxide.cache")) else {
    return;
  };
  let start = std::time::Instant::now();
  let log = run("b");
  let took = start.elapsed();
  release();
  let warnings: Vec<&str> = log.lines().filter(|l| l.starts_with("Warning:")).collect();
  assert_eq!((error_count(&log), warnings.len()), (0, 1), "{log}");
  assert!(warnings[0].starts_with("Warning:I/O:cache"), "{log}");
  assert!(took < std::time::Duration::from_secs(40), "{took:?}");
  assert_eq!(
    images_of(&parent.join("b.html")),
    pairs(&[
      ("x2.svg", "child_2/images/A"),
      ("x3.svg", "child_2/images/B")
    ])
  );
  assert_eq!(images_of(&parent.join("a.html")), a);
}

/// The cache made read-only between the two documents: `b.tex`'s transaction fails and runs again in memory, which
/// skips the names whose files exist (round 4 decided the skip before the transaction: both pages showed `x1.svg`,
/// holding `b`'s image). `SqliteStore::immediate`, `Graphics::resolve_names`.
#[cfg(unix)]
#[test]
fn batchfix995_a_read_only_cache_overwrites_no_image() {
  use std::os::unix::fs::PermissionsExt;
  unusable_cache(|cache| {
    std::fs::set_permissions(cache, std::fs::Permissions::from_mode(0o444)).unwrap();
    if std::fs::OpenOptions::new().write(true).open(cache).is_ok() {
      eprintln!("SKIP: a read-only file is writable here (root)");
      return None;
    }
    Some(Box::new(|| {}))
  });
}

/// Another process holds the cache (`BEGIN EXCLUSIVE`) for the whole of `b.tex`'s run: it waits 10 s, warns and goes
/// on in memory, well inside `--timeout=60` (round 4 waited 120 s: `Fatal:timeout:wallclock`). `sqlite_store.rs`
/// `BUSY_TIMEOUT`.
#[test]
fn batchfix995_a_locked_cache_waits_less_than_the_conversion() {
  unusable_cache(|cache| {
    let conn = rusqlite::Connection::open(cache).unwrap();
    conn
      .execute_batch("BEGIN EXCLUSIVE; INSERT OR REPLACE INTO kv VALUES('_held_', '1');")
      .unwrap();
    Some(Box::new(move || conn.execute_batch("COMMIT").unwrap()))
  });
}

/// A planted `_source_` that is a device, or a path out of the destination: the conversion ends promptly (round 4
/// hashed `/dev/zero` for ever), reports the device once, and writes nothing out of the folder; a planted entry of
/// an image's own is reported once. `Graphics::vouch`, `resource_cache::place`.
#[cfg(unix)]
#[test]
fn batchfix995_a_planted_source_is_never_read_for_ever() {
  let root = tempfile::tempdir().unwrap();
  let src = root.path().join("src");
  std::fs::create_dir_all(src.join("figs")).unwrap();
  std::fs::write(
    src.join("p.tex"),
    "\\documentclass{article}\\usepackage{graphicx}\\begin{document}\n\
     Fig \\includegraphics{figs/a.png} \\includegraphics{figs/b.png}\n\\end{document}\n",
  )
  .unwrap();
  let (a, b) = (png(4, 4), png(4, 5));
  std::fs::write(src.join("figs/a.png"), a).unwrap();
  std::fs::write(src.join("figs/b.png"), &b).unwrap();
  let run = || {
    let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .args(["--nocomments", "--timeout=60", "--dest=p.html", "p.tex"])
      .current_dir(&src)
      .output()
      .expect("spawn latexml_oxide");
    String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "")
  };
  let digest = |file: &Path| {
    latexml_post::graphics_cache::SourceDigest::of_file(file)
      .unwrap()
      .content()
      .iter()
      .map(|b| format!("{b:02x}"))
      .collect::<String>()
  };
  {
    // Each image's bytes said to be in a file another source owns, whose source is a device or outside the folder.
    let conn = rusqlite::Connection::open(src.join("LaTeXML-oxide.cache")).unwrap();
    conn
      .execute_batch(
        "CREATE TABLE kv(key TEXT PRIMARY KEY, value TEXT NOT NULL) WITHOUT ROWID; PRAGMA user_version = 1;",
      )
      .unwrap();
    for (image, name, source) in [
      ("figs/a.png", "evil.png", "/dev/zero"),
      ("figs/b.png", "evil2.png", "../outside.png"),
    ] {
      let h = digest(&src.join(image));
      for (key, value) in [
        (format!("_content_copy|{h}|"), name.to_string()),
        (format!("_owner_{name}"), "copy|other".to_string()),
        (format!("_holds_{name}"), format!("copy|{h}")),
        (format!("_source_{name}"), source.to_string()),
      ] {
        conn
          .execute("INSERT OR REPLACE INTO kv VALUES(?1, ?2)", [&key, &value])
          .unwrap();
      }
    }
    // And `a.png`'s own entry a page, its kept path another source's: looked up again for a generated name, the
    // entry is reported once (round 4: twice).
    let key = format!(
      "LaTeXML::Post::Graphics:figs/a.png.png||{}",
      digest(&src.join("figs/a.png"))
    );
    for (key, value) in [
      (key.as_str(), "p.html||"),
      ("_owner_figs/a.png", "copy|zzz"),
    ] {
      conn
        .execute("INSERT OR REPLACE INTO kv VALUES(?1, ?2)", [key, value])
        .unwrap();
    }
  }
  std::fs::write(root.path().join("outside.png"), &b).unwrap();
  let start = std::time::Instant::now();
  let log = run();
  assert!(start.elapsed() < std::time::Duration::from_secs(40));
  let warnings: Vec<&str> = log.lines().filter(|l| l.starts_with("Warning:")).collect();
  assert_eq!((error_count(&log), warnings.len()), (0, 2), "{log}");
  assert!(
    warnings[0].contains(" names p.html for figs/a.png"),
    "{log}"
  );
  assert!(warnings[1].contains("the source /dev/zero"), "{log}");
  let urls: Vec<String> = images_of(&src.join("p.html"))
    .into_iter()
    .map(|(url, _)| url)
    .collect();
  assert_eq!(urls, ["x1.png", "figs/b.png"]);
  assert!(!src.join("evil.png").exists() && !src.join("evil2.png").exists());
  assert_eq!(std::fs::read(root.path().join("outside.png")).unwrap(), b);
}
