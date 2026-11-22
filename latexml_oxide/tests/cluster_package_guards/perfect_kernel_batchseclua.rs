//! Red/green guards for batch SECLUA: untrusted input stays in the job. Document Lua runs sandboxed (the
//! `lua_bridge.rs` prelude: TeX's `shell_escape=f`, `openout_any=p`): a call lualatex refuses too returns lualatex's
//! values with a Warning (batch LUAEXEC), one only the sandbox refuses is an Error; an image copy never leaves
//! `--dest` (`graphics.rs` `copy_to_destination`, Perl `Post/Graphics.pm` L292-300).
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

/// The sandbox's refusals in `log` that only it makes (Errors), in order.
fn denials(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter_map(|line| line.strip_prefix("Error:lua:sandbox "))
    .map(|line| line.split(" is not allowed").next().unwrap())
    .collect()
}

/// The sandbox's refusals in `log` that lualatex makes too (Warnings), in order.
fn refusals(log: &str) -> Vec<&str> {
  log
    .lines()
    .filter_map(|line| line.strip_prefix("Warning:lua:sandbox "))
    .map(|line| line.split(" was ").next().unwrap())
    .collect()
}

/// A refused program call answers as lualatex `-no-shell-escape`: `os.execute("…")` returns nil and "All command
/// execution disabled." and the document goes on (its `else` branch), `os.execute()` (pdftexcmds'
/// `\pdf@shellescape`) returns 0 silently; one Warning, no program runs. Base: 2 Errors, both chunks lost. Witnesses
/// the TL manuals derivative, wheelchart, xassoccnt_doc (+6). Repro luatex-profile/lua_sandbox_refused_program_goes_on.
#[test]
fn batchluaexec_refused_program_answers_as_lualatex() {
  if !texlua_present() {
    return;
  }
  let root = tempfile::tempdir().unwrap();
  std::fs::write(
    root.path().join("t.tex"),
    repro!("luatex-profile", "lua_sandbox_refused_program_goes_on"),
  )
  .unwrap();
  let log = convert_at(root.path(), "t.tex", "t.xml");
  let xml = std::fs::read_to_string(root.path().join("t.xml")).unwrap_or_default();
  assert_eq!(
    refusals(&log),
    ["os.execute \"echo ran >ran.txt\""],
    "{log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 1), "{log}");
  assert!(
    xml.contains("<p>[refused: All command execution disabled.]\n[0]</p>"),
    "{xml}"
  );
  assert_eq!(files_below(root.path()), ["t.tex", "t.xml"]);
}

/// A chunk that uses a refused call's nil and fails is lost, as under lualatex (rc=1, "attempt to index a nil value"):
/// `io.popen(cmd):read()` and an `io.open` write openout_any=p refuses, each an Error naming the refused call beside
/// its Warning; the next chunk runs. Base: a Warning and an Info each, 0 Errors. Repro
/// luatex-profile/lua_sandbox_refused_then_failed.
#[test]
fn batchluaexec_refused_then_failed_is_an_error() {
  if !texlua_present() {
    return;
  }
  let root = tempfile::tempdir().unwrap();
  let job = root.path().join("job");
  std::fs::create_dir(&job).unwrap();
  std::fs::write(
    job.join("t.tex"),
    repro!("luatex-profile", "lua_sandbox_refused_then_failed"),
  )
  .unwrap();
  let log = convert_at(root.path(), "job/t.tex", "job/t.xml");
  let xml = std::fs::read_to_string(job.join("t.xml")).unwrap_or_default();
  let sandbox: Vec<&str> = log
    .lines()
    .filter(|line| line.contains(":lua:sandbox "))
    .collect();
  assert_eq!(
    sandbox,
    [
      "Warning:lua:sandbox io.popen \"echo hi\" was not run (document Lua runs no programs, as lualatex without \
       shell escape)",
      "Error:lua:sandbox the chunk failed after the sandbox refused io.popen \"echo hi\": [string \"local h = \
       io.popen(\"echo hi\") tex.sprint(h:re...\"]:1: attempt to index a nil value (local 'h')",
      "Warning:lua:sandbox writing ../refused_then_failed.txt was refused (document Lua writes only below the job \
       directory, as lualatex under openout_any=p)",
      "Error:lua:sandbox the chunk failed after the sandbox refused writing ../refused_then_failed.txt: [string \
       \"local f = io.open(\"../refused_then_failed.txt...\"]:1: attempt to index a nil value (local 'f')",
    ],
    "{log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (2, 2), "{log}");
  assert!(xml.contains("<p>A[]\nB[]\nC[alive]</p>"), "{xml}");
  assert_eq!(files_below(root.path()), ["job/t.tex", "job/t.xml"]);
}

/// Each severity has its own budget of 20 listed refusals and its own summary: 40 `io.output` writes outside the job
/// (only the sandbox refuses them) are 20 Errors and one summary Error; after 30 `os.execute` Warnings, 30 such writes
/// still list 20 Errors by name. Base: the Warnings filled the list, and the mixed chunk's writes were one summary
/// Error. Repro luatex-profile/lua_sandbox_refusal_budgets.
#[test]
fn batchluaexec_refusal_budget_per_severity() {
  if !texlua_present() {
    return;
  }
  let root = tempfile::tempdir().unwrap();
  let job = root.path().join("job");
  std::fs::create_dir(&job).unwrap();
  std::fs::write(
    job.join("t.tex"),
    repro!("luatex-profile", "lua_sandbox_refusal_budgets"),
  )
  .unwrap();
  let log = convert_at(root.path(), "job/t.tex", "job/t.xml");
  let xml = std::fs::read_to_string(job.join("t.xml")).unwrap_or_default();
  let strict = |name: &str| {
    format!(
      "Error:lua:sandbox writing ../{name} is not allowed (document Lua runs sandboxed: no programs, writes only \
       below the job directory)"
    )
  };
  let program = "Warning:lua:sandbox os.execute \"exit 0\" was not run (document Lua runs no programs, as lualatex \
                 without shell escape)"
    .to_string();
  let more = |severity: &str, n: usize| {
    format!(
      "{severity}:lua:sandbox {n} more calls the sandbox refused in this chunk are not listed"
    )
  };
  let mut expected: Vec<String> = vec![strict("budget_strict.txt"); 20];
  expected.push(more("Error", 20));
  expected.extend(vec![program; 20]);
  expected.extend(vec![strict("budget_mixed.txt"); 20]);
  expected.push(more("Error", 10));
  expected.push(more("Warning", 10));
  let sandbox: Vec<&str> = log
    .lines()
    .filter(|line| line.contains(":lua:sandbox "))
    .collect();
  assert_eq!(sandbox, expected, "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (42, 21), "{log}");
  assert!(xml.contains("[strict:40][programs:30 writes:30]"), "{xml}");
  assert_eq!(files_below(root.path()), ["job/t.tex", "job/t.xml"]);
}

/// `font.current()` (hologo's glyph test, `font.fonts[font.current()]`) raises a Lua error instead of ending texlua,
/// which has no fonts: the chunk is not evaluated (an Info) and the next chunk sees the state an earlier one set.
/// Base: `[] [nil]`, an `Error:lua:interpreter`. Witness the TL manual derivative. Repro
/// luatex-profile/lua_font_current_keeps_the_interpreter.
#[test]
fn batchluaexec_font_current_keeps_the_interpreter() {
  if !texlua_present() {
    return;
  }
  let root = tempfile::tempdir().unwrap();
  std::fs::write(
    root.path().join("t.tex"),
    repro!("luatex-profile", "lua_font_current_keeps_the_interpreter"),
  )
  .unwrap();
  let log = convert_at(root.path(), "t.tex", "t.xml");
  let xml = std::fs::read_to_string(root.path().join("t.xml")).unwrap_or_default();
  assert_eq!((error_count(&log), warning_count(&log)), (0, 0), "{log}");
  assert!(
    log.contains("font.current needs the TeX engine (texlua has no fonts)"),
    "{log}"
  );
  assert!(xml.contains("<p>[] [5]</p>"), "{xml}");
}

/// No program runs and no native code loads: `os.execute`/`io.popen`/`os.exec`/`os.spawn`/`os.kpsepopen` return
/// lualatex's values (a Warning each), `require` of a network library and of a C module on `package.cpath` (`./?.so`;
/// `.\?.dll` on Windows) and `package.loadlib` are refused (an Error each, pcalled). Base: `X[768]`, the shell ran.
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
  // The C module texlua's own `package.cpath` finds in the job directory: `./?.so`, `.\?.dll` on Windows.
  let cmod = if cfg!(windows) { "cmod.dll" } else { "cmod.so" };
  std::fs::write(root.path().join(cmod), b"").unwrap();
  let log = convert_at(root.path(), "t.tex", "t.xml");
  let xml = std::fs::read_to_string(root.path().join("t.xml")).unwrap_or_default();
  assert_eq!(
    denials(&log),
    [
      "require \"socket\"",
      "loading the C module \"cmod\"",
      "package.loadlib"
    ],
    "{log}"
  );
  assert_eq!(
    refusals(&log),
    [
      "os.execute \"exit 3\"",
      "io.popen \"exit 3\"",
      "os.exec \"touch ran.txt\"",
      "os.spawn \"touch ran.txt\"",
      "os.kpsepopen \"touch ran.txt\""
    ],
    "{log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (3, 5), "{log}");
  assert!(
    xml.contains(
      "X[nil] Y\n[none] Z\nW[nil:All command execution disabled. / nil:All command execution disabled. / nil:all command execution is disabled]"
    ),
    "{xml}"
  );
  assert_eq!(files_below(root.path()), [cmod, "t.tex", "t.xml"]);
}

/// Writes follow kpathsea's `openout_any=p`: `out.txt` in the job directory is written and read back; `../`, an
/// absolute path and a dotfile are refused as lualatex refuses them (`io.open` returns nil, a Warning each), and
/// nothing appears there. Base: all three written. Repro
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
  let absolute = format!("writing {}", absolute.to_string_lossy().replace('\\', "/"));
  assert_eq!(
    refusals(&log),
    [
      "writing ../sandbox_escape.txt",
      absolute.as_str(),
      "writing .sandbox_escape"
    ],
    "{log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 3), "{log}");
  assert!(xml.contains("W[ok] R[hello] [nil nil nil]"), "{xml}");
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

/// The checks hold their helpers from boot: an `mplib` instance whose options table swallows assignments (made
/// first, in a fresh interpreter: lmplib keeps a finder across instances) still writes through the sandbox's finder; a
/// replaced `string.find` no longer disables the write check; a file-handle `write` that filters out `D` frames does
/// not hide a refusal (an Error or a Warning). Base: the two files written outside the job, the `os.execute` refusal
/// unreported. Repro
/// luatex-profile/lua_sandbox_helpers_held_at_boot.
#[test]
fn batchseclua_sandbox_helpers_are_held_at_boot() {
  if !texlua_present() {
    return;
  }
  let root = tempfile::tempdir().unwrap();
  let job = root.path().join("job");
  std::fs::create_dir(&job).unwrap();
  std::fs::write(
    job.join("t.tex"),
    repro!("luatex-profile", "lua_sandbox_helpers_held_at_boot"),
  )
  .unwrap();
  let log = convert_at(root.path(), "job/t.tex", "job/t.xml");
  let xml = std::fs::read_to_string(job.join("t.xml")).unwrap_or_default();
  assert_eq!(denials(&log), ["writing ../mp_escape.txt"], "{log}");
  assert_eq!(
    refusals(&log),
    ["writing ../find_escape.txt", "os.execute \"exit 3\""],
    "{log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (1, 2), "{log}");
  assert!(
    xml.contains("[mp:done] [find:refused] [hidden:nil]"),
    "{xml}"
  );
  assert_eq!(files_below(root.path()), ["job/t.tex", "job/t.xml"]);
}

/// A write through a symbolic link the source tree ships — a linked directory or a linked file, both pointing beside
/// the job — is refused; a plain file is written. Base: both written through the links. Repro
/// luatex-profile/lua_sandbox_no_symlink_writes.
#[cfg(unix)]
#[test]
fn batchseclua_lua_writes_follow_no_symlink() {
  if !texlua_present() {
    return;
  }
  let root = tempfile::tempdir().unwrap();
  let (job, beside) = (root.path().join("job"), root.path().join("beside"));
  std::fs::create_dir(&job).unwrap();
  std::fs::create_dir(&beside).unwrap();
  std::fs::write(beside.join("kept.txt"), "kept").unwrap();
  std::os::unix::fs::symlink(&beside, job.join("lnkdir")).unwrap();
  std::os::unix::fs::symlink(beside.join("kept.txt"), job.join("lnkfile")).unwrap();
  std::fs::write(
    job.join("t.tex"),
    repro!("luatex-profile", "lua_sandbox_no_symlink_writes"),
  )
  .unwrap();
  let log = convert_at(root.path(), "job/t.tex", "job/t.xml");
  let xml = std::fs::read_to_string(job.join("t.xml")).unwrap_or_default();
  assert_eq!(
    denials(&log),
    ["writing lnkdir/via_dir.txt", "writing lnkfile"],
    "{log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (2, 0), "{log}");
  assert!(
    xml.contains("[dir:refused] [file:refused] [plain:ok]"),
    "{xml}"
  );
  assert_eq!(
    std::fs::read_dir(&beside).unwrap().count(),
    1,
    "only kept.txt beside the job"
  );
  assert_eq!(
    std::fs::read_to_string(beside.join("kept.txt")).unwrap(),
    "kept"
  );
  assert!(job.join("plain.txt").is_file());
}

/// The protocol pipe is the bridge's alone: a chunk's `io.write`/`print` of a frame header (a forged 100 TB length,
/// a forged `ERR`) reach the null device, and the conversion goes on. `img.new` (texlua has no backend; it crashed)
/// is refused; an `os.exit` ends the interpreter — an Error, its output lost — and the next chunk runs in a fresh one.
/// The interpreter's environment is the allowlist: a variable outside it reads nil, `PATH` is kept. Base: the forged
/// length aborted the conversion. Repro luatex-profile/lua_interpreter_restarts.
#[test]
fn batchseclua_lua_interpreter_restarts_and_owns_its_pipe() {
  if !texlua_present() {
    return;
  }
  let root = tempfile::tempdir().unwrap();
  std::fs::write(
    root.path().join("t.tex"),
    repro!("luatex-profile", "lua_interpreter_restarts"),
  )
  .unwrap();
  let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(["t.tex", "--dest", "t.xml", "--nocomments", "--timeout=110"])
    .env("LATEXML_SECLUA_PROBE", "visible")
    .current_dir(root.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  let xml = std::fs::read_to_string(root.path().join("t.xml")).unwrap_or_default();
  assert!(output.status.success(), "{log}");
  assert_eq!(denials(&log), ["img.new"], "{log}");
  assert_eq!(
    log
      .lines()
      .filter(|line| line.starts_with("Error:lua:interpreter "))
      .count(),
    1,
    "{log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (2, 0), "{log}");
  assert!(
    xml.contains("A[]\nB[]\nC[false]\nD\nE[alive]\nF[nil/set]"),
    "{xml}"
  );
}

/// A tar source's link is unpacked only when its target, resolved through the links already made, stays inside the
/// source tree: an internal symbolic link and an internal hard link are read; a symbolic link and a hard link to a file
/// beside the archive, and a symbolic link through an internal one to `..`, are skipped, a counted Warning each.
/// Base: every link was skipped, and the warnings went to stderr uncounted.
#[cfg(unix)]
#[test]
fn batchseclua_tar_links_stay_inside_the_source() {
  let root = tempfile::tempdir().unwrap();
  std::fs::write(root.path().join("beside.tex"), "BESIDEWORD").unwrap();
  let tex = "\\documentclass{article}\\begin{document}[\\InputIfFileExists{in.tex}{}{none}][\\InputIfFileExists{sub/up.tex}{}{none}][\\InputIfFileExists{hin.tex}{}{none}][\\InputIfFileExists{lnk.tex}{}{none}][\\InputIfFileExists{hard.tex}{}{none}][\\InputIfFileExists{sub/self/esc.tex}{}{none}]\\end{document}\n";
  let mut builder = tar::Builder::new(Vec::new());
  for (name, body) in [("t.tex", tex), ("inside.tex", "INSIDEWORD")] {
    let mut header = tar::Header::new_gnu();
    header.set_size(body.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
      .append_data(&mut header, name, body.as_bytes())
      .unwrap();
  }
  let beside = root.path().join("beside.tex");
  for (kind, name, target) in [
    (
      tar::EntryType::Symlink,
      "in.tex",
      PathBuf::from("inside.tex"),
    ),
    (
      tar::EntryType::Symlink,
      "sub/up.tex",
      PathBuf::from("../inside.tex"),
    ),
    (tar::EntryType::Link, "hin.tex", PathBuf::from("inside.tex")),
    (tar::EntryType::Symlink, "lnk.tex", beside.clone()),
    (tar::EntryType::Link, "hard.tex", beside.clone()),
    (tar::EntryType::Symlink, "sub/self", PathBuf::from("..")),
    (
      tar::EntryType::Symlink,
      "sub/self/esc.tex",
      PathBuf::from("../beside.tex"),
    ),
  ] {
    let mut header = tar::Header::new_gnu();
    header.set_entry_type(kind);
    header.set_size(0);
    builder.append_link(&mut header, name, target).unwrap();
  }
  std::fs::write(root.path().join("p.tar"), builder.into_inner().unwrap()).unwrap();
  let log = convert_at(root.path(), "p.tar", "p.xml");
  let xml = std::fs::read_to_string(root.path().join("p.xml")).unwrap_or_default();
  let skipped: Vec<&str> = log
    .lines()
    .filter_map(|line| line.strip_prefix("Warning:archive:link the archive entry '"))
    .map(|line| line.split('\'').next().unwrap())
    .collect();
  assert_eq!(
    skipped,
    ["hard.tex", "lnk.tex", "sub/self/esc.tex"],
    "{log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 3), "{log}");
  assert!(
    xml.contains("[INSIDEWORD\n][INSIDEWORD\n][INSIDEWORD\n][none][none][none]"),
    "{xml}"
  );
  assert!(!xml.contains("BESIDEWORD"), "{xml}");
}

/// The bridge accepts only frames that carry its nonce and builds them without a metamethod: a string `__tostring`
/// that fails no longer hides a refusal, and a forged `D`, `W` or `G` header through lualibs' `inspect` (which held the
/// original `print`), `io.write` or `require('texio').write` reaches no pipe, so no refusal is invented. Base: the
/// first refusal unreported and the raw `texio` write a broken frame. Repro luatex-profile/lua_sandbox_frames_authenticated.
#[test]
fn batchseclua_lua_frames_are_authenticated() {
  if !texlua_present() {
    return;
  }
  let root = tempfile::tempdir().unwrap();
  let job = root.path().join("job");
  std::fs::create_dir(&job).unwrap();
  std::fs::write(
    job.join("t.tex"),
    repro!("luatex-profile", "lua_sandbox_frames_authenticated"),
  )
  .unwrap();
  let log = convert_at(root.path(), "job/t.tex", "job/t.xml");
  let xml = std::fs::read_to_string(job.join("t.xml")).unwrap_or_default();
  assert_eq!(
    refusals(&log),
    ["writing ../escape_a.txt", "writing ../escape_b.txt"],
    "{log}"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 2), "{log}");
  assert!(!log.contains("forged"), "{log}");
  assert!(xml.contains("[a:true] [b:true] [texio:true]"), "{xml}");
  assert_eq!(files_below(root.path()), ["job/t.tex", "job/t.xml"]);
}

/// On Unix a backslash is a filename character: a write to `a\b`, a link the source tree ships pointing beside the
/// job, is refused (the check no longer reads it as `a/b`). Base: written through the link. Repro
/// luatex-profile/lua_sandbox_backslash_names.
#[cfg(unix)]
#[test]
fn batchseclua_lua_backslash_names_are_refused() {
  if !texlua_present() {
    return;
  }
  let root = tempfile::tempdir().unwrap();
  let (job, beside) = (root.path().join("job"), root.path().join("beside"));
  std::fs::create_dir(&job).unwrap();
  std::fs::create_dir(&beside).unwrap();
  std::os::unix::fs::symlink(beside.join("t.txt"), job.join("a\\b")).unwrap();
  std::fs::write(
    job.join("t.tex"),
    repro!("luatex-profile", "lua_sandbox_backslash_names"),
  )
  .unwrap();
  let log = convert_at(root.path(), "job/t.tex", "job/t.xml");
  let xml = std::fs::read_to_string(job.join("t.xml")).unwrap_or_default();
  assert_eq!(denials(&log), ["writing a\\b"], "{log}");
  assert_eq!((error_count(&log), warning_count(&log)), (1, 0), "{log}");
  assert!(xml.contains("[bs:refused] [plain:ok]"), "{xml}");
  assert_eq!(
    std::fs::read_dir(&beside).unwrap().count(),
    0,
    "nothing beside the job"
  );
}

/// Forty refusals of a 1 MiB name: 20 listed, each name cut at 256 bytes, and one summary for the rest, Warnings as
/// lualatex refuses each too; no Fatal (the Errors' own budget: `batchluaexec_refusal_budget_per_severity`). Base: every refusal carried the whole name (a reviewer's 80 × 50 MB ran the bridge out of memory). Repro
/// luatex-profile/lua_sandbox_many_refusals.
#[test]
fn batchseclua_lua_refusals_are_bounded() {
  if !texlua_present() {
    return;
  }
  let root = tempfile::tempdir().unwrap();
  std::fs::write(
    root.path().join("t.tex"),
    repro!("luatex-profile", "lua_sandbox_many_refusals"),
  )
  .unwrap();
  let log = convert_at(root.path(), "t.tex", "t.xml");
  let xml = std::fs::read_to_string(root.path().join("t.xml")).unwrap_or_default();
  let refusals: Vec<&str> = log
    .lines()
    .filter(|line| line.starts_with("Warning:lua:sandbox "))
    .collect();
  assert_eq!(refusals.len(), 21, "{log}");
  assert!(
    refusals[..20]
      .iter()
      .all(|line| line.len() < 512 && line.contains("(1048579 bytes)"))
  );
  assert_eq!(
    refusals[20],
    "Warning:lua:sandbox 20 more calls the sandbox refused in this chunk are not listed"
  );
  assert_eq!((error_count(&log), warning_count(&log)), (0, 21), "{log}");
  assert!(!log.contains("Fatal:"), "{log}");
  assert!(xml.contains("[denied:40]"), "{xml}");
}

/// A chunk spinning in `while true do end` past `--timeout`: the wall-clock exit kills the registered interpreter,
/// which no longer outlives the conversion (it ran on at 100 % CPU with PPID 1). Repro
/// luatex-profile/lua_spinning_chunk_ends_with_timeout.
#[cfg(target_os = "linux")]
#[test]
fn batchseclua_lua_interpreter_ends_with_the_timeout() {
  if !texlua_present() {
    return;
  }
  let root = tempfile::tempdir().unwrap();
  std::fs::write(
    root.path().join("t.tex"),
    repro!("luatex-profile", "lua_spinning_chunk_ends_with_timeout"),
  )
  .unwrap();
  let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
    .args(["t.tex", "--dest", "t.xml", "--nocomments", "--timeout=20"])
    .current_dir(root.path())
    .output()
    .expect("spawn latexml_oxide");
  let log = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
  assert!(log.contains("Fatal:timeout:wallclock"), "{log}");
  let pid = std::fs::read_to_string(root.path().join("texlua.pid")).unwrap();
  let stat = Path::new("/proc").join(pid.trim()).join("stat");
  // Killed before the exit; at most a zombie until it is reaped.
  let running = (0..50).all(|_| {
    let alive = std::fs::read_to_string(&stat).is_ok_and(|text| {
      text
        .rsplit(')')
        .next()
        .is_some_and(|rest| !rest.trim_start().starts_with('Z'))
    });
    if alive {
      std::thread::sleep(std::time::Duration::from_millis(100));
    }
    alive
  });
  assert!(!running, "texlua {pid} outlived the conversion");
}
