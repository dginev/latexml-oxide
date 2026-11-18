//! Red/green guards for batch SECLUA: untrusted input stays in the job. Document Lua runs sandboxed (the
//! `lua_bridge.rs` prelude: TeX's `shell_escape=f`, `openout_any=p`), each refusal an Error; an image copy never
//! leaves `--dest` (`graphics.rs` `copy_to_destination`, Perl `Post/Graphics.pm` L292-300).
//! OXIDIZED_DESIGN_DIVERGENCES #1081. The Lua guards self-skip without a host `texlua`.
use std::{
  path::{Path, PathBuf},
  process::Command,
};

use super::perfect_kernel_batch46::{error_count, warning_count};

macro_rules! repro {
  ($dir:literal, $name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/",
      $dir,
      "/",
      $name,
      ".tex"
    ))
  };
}

fn texlua_present() -> bool {
  Command::new("texlua")
    .arg("--version")
    .output()
    .is_ok_and(|o| o.status.success())
}

/// Convert `source` (relative to `root`) to `dest` with `root` as the working directory; the ANSI-stripped stderr.
fn convert_at(root: &Path, source: &str, dest: &str) -> String {
  let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args([source, "--dest", dest, "--nocomments", "--timeout=110"])
    .current_dir(root)
    .output()
    .expect("spawn latexml_oxide");
  String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "")
}

/// Every file below `dir`, relative to it, sorted; the conversion's own log aside.
fn files_below(dir: &Path) -> Vec<String> {
  fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
      let path: PathBuf = entry.path();
      if path.is_dir() {
        walk(&path, root, out);
      } else if !path.to_string_lossy().ends_with(".latexml.log") {
        out.push(
          path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/"),
        );
      }
    }
  }
  let mut out = Vec::new();
  walk(dir, dir, &mut out);
  out.sort();
  out
}

/// The sandbox's refusals in `log`, in order.
fn denials(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter_map(|line| line.strip_prefix("Error:lua:sandbox "))
    .map(|line| line.split(" is not allowed").next().unwrap())
    .collect()
}

/// No program runs and no native code loads: `os.execute`/`io.popen` (pcalled, so only the record shows the refusal),
/// `require` of a network library and of a C module on `./?.so`, `package.loadlib`. Base: `X[768]`, the shell ran.
/// Repro luatex-profile/lua_sandbox_no_programs.
#[test]
fn batchseclua_lua_runs_no_program() {
  if !texlua_present() {
    return;
  }
  let root = tempfile::tempdir().unwrap();
  std::fs::write(
    root.path().join("t.tex"),
    repro!("luatex-profile", "lua_sandbox_no_programs"),
  )
  .unwrap();
  std::fs::write(root.path().join("cmod.so"), b"").unwrap();
  let log = convert_at(root.path(), "t.tex", "t.xml");
  let xml = std::fs::read_to_string(root.path().join("t.xml")).unwrap_or_default();
  assert_eq!(
    denials(&log),
    [
      "os.execute",
      "io.popen",
      "require \"socket\"",
      "loading the C module \"cmod\"",
      "package.loadlib"
    ],
    "{log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (5, 0), "{log}");
  assert!(xml.contains("X[nil] Y\n[none] Z"), "{xml}");
}

/// Writes follow kpathsea's `openout_any=p`: `out.txt` in the job directory is written and read back; `../`, an
/// absolute path and a dotfile are refused, and nothing appears there. Base: all three written. Repro
/// luatex-profile/lua_sandbox_paranoid_writes (its absolute path pointed into the test's own directory).
#[test]
fn batchseclua_lua_writes_stay_in_the_job_directory() {
  if !texlua_present() {
    return;
  }
  let root = tempfile::tempdir().unwrap();
  let job = root.path().join("job");
  std::fs::create_dir(&job).unwrap();
  let absolute = root.path().join("absolute_escape.txt");
  let tex = repro!("luatex-profile", "lua_sandbox_paranoid_writes").replace(
    "/tmp/sandbox_escape.txt",
    &absolute.to_string_lossy().replace('\\', "/"),
  );
  std::fs::write(job.join("t.tex"), tex).unwrap();
  let log = convert_at(root.path(), "job/t.tex", "job/t.xml");
  let xml = std::fs::read_to_string(job.join("t.xml")).unwrap_or_default();
  assert_eq!(denials(&log).len(), 3, "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (3, 0), "{log}");
  assert!(xml.contains("W[ok] R[hello]"), "{xml}");
  assert_eq!(files_below(root.path()), [
    "job/out.txt",
    "job/t.tex",
    "job/t.xml"
  ]);
}

/// An image reached as `../../x/y` from the source is copied as `x1.png` inside `--dest` (Perl's
/// `generateResourcePathname` when `pathname_is_contained` fails), never to `<dest>/../../x/y.png`; the witness is
/// the TL manual ijsra, whose class's images resolve to `../../../tex/latex/…`. Repro
/// loader/image_copy_outside_source_stays_in_dest.
#[test]
fn batchseclua_image_copy_stays_in_dest() {
  let root = tempfile::tempdir().unwrap();
  let source_dir = root.path().join("src/a/b");
  std::fs::create_dir_all(&source_dir).unwrap();
  std::fs::create_dir_all(root.path().join("src/x")).unwrap();
  std::fs::create_dir_all(root.path().join("out/d")).unwrap();
  std::fs::write(
    source_dir.join("t.tex"),
    repro!("loader", "image_copy_outside_source_stays_in_dest"),
  )
  .unwrap();
  std::fs::write(
    root.path().join("src/x/y.png"),
    include_bytes!("../../../latexml_post/tests/fixtures/tile.png"),
  )
  .unwrap();
  // The source named by its absolute path, as the witness was: the image is then `../../x/y.png` from it.
  let source = source_dir.join("t.tex").to_string_lossy().into_owned();
  let log = convert_at(root.path(), &source, "out/d/t.html");
  assert_eq!(error_count(&log), 0, "{log}");
  let written: Vec<String> = files_below(root.path())
    .into_iter()
    .filter(|path| !path.starts_with("src/") && !path.ends_with(".css") && !path.ends_with(".js"))
    .collect();
  assert_eq!(written, ["out/d/t.html", "out/d/x1.png"], "{log}");
  let html = std::fs::read_to_string(root.path().join("out/d/t.html")).unwrap();
  assert!(html.contains("src=\"x1.png\""), "{html}");
}
