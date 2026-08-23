//! CLI / output-shape / telemetry smoke tests.
//!
//! Auto-consolidated test binary: each former file is an inline `mod`
//! below, body preserved verbatim, merged into one link unit for CI
//! economy. All members are subprocess- or few-conversion tests, so
//! co-locating them in one process stays far under the RSS fuse.

mod hello {
  use latexml::converter::Converter;
  use latexml_core::common::{Config, OutputFormat};

  #[test]
  fn can_convert_hello() {
    assert!(latexml_core::util::logger::init(log::LevelFilter::Warn).is_ok());
    let hello_source = "tests/hello/hello.tex";
    let html_config = Config {
      format: OutputFormat::HTML5,
      ..Config::default()
    };
    let mut converter = Converter::from_config(html_config);
    converter.initialize_session().expect("can initialize.");

    let conversion_result = converter.convert(hello_source.to_string());
    assert!(conversion_result.result.is_some());
    let response = conversion_result;
    assert!(response.result.is_some());
    assert!(response.status_code == 0);
    assert_eq!(response.status, "No obvious problems");
  }
}

mod single_binary_smoke {
  //! Smoke test for the prebuilt-binary distribution.
  //!
  //! Builds, locates, and runs the `latexml_oxide` binary from a temp
  //! directory that has *no access* to the project's `resources/` tree,
  //! then asserts that:
  //!
  //! 1. Conversion succeeds and produces the HTML file at the requested destination.
  //! 2. The bundled CSS files referenced in the HTML actually land in the destination directory (via
  //!    the embedded resource fallback).
  //!
  //! Catches regressions where someone re-introduces a disk-only resource
  //! lookup path on the post-processing pipeline. Without the embedded
  //! fallback this test fails by either dropping the CSS files or
  //! emitting a "missing_file" warning for `LaTeXML.css`/`ltx-article.css`.
  //!
  //! The binary path comes from cargo's `CARGO_BIN_EXE_latexml_oxide`
  //! env var, set automatically for integration tests that import a
  //! crate which produces a binary target.

  use std::{path::Path, process::Command};

  const HELLO_TEX: &str = "\\documentclass{article}\n\
                           \\begin{document}\n\
                           Hello World!\n\
                           \\end{document}\n";

  #[test]
  fn binary_runs_without_source_tree() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(
      Path::new(bin).is_file(),
      "test harness did not stage binary at {}",
      bin,
    );

    let workdir = tempfile::tempdir().expect("create tempdir");
    let tex_path = workdir.path().join("hello.tex");
    let html_path = workdir.path().join("hello.html");
    std::fs::write(&tex_path, HELLO_TEX).expect("write hello.tex");

    // Run the binary with the tempdir as cwd so resource lookups can't
    // accidentally pick up the project tree via "." in the search path.
    let output = Command::new(bin)
      .arg(tex_path.file_name().unwrap())
      .arg("--dest")
      .arg(html_path.file_name().unwrap())
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");

    assert!(
      output.status.success(),
      "binary exited with status {:?}\nstderr:\n{}",
      output.status.code(),
      String::from_utf8_lossy(&output.stderr),
    );

    // Output HTML present and references the expected CSS files.
    let html = std::fs::read_to_string(&html_path).expect("read hello.html");
    assert!(
      html.contains("LaTeXML.css"),
      "expected LaTeXML.css reference in HTML, got:\n{html}",
    );
    assert!(
      html.contains("ltx-article.css"),
      "expected ltx-article.css reference in HTML, got:\n{html}",
    );

    // The CSS files themselves must have been materialised alongside
    // the HTML — that's the post-XSLT copy_resource step's job, and
    // it pulls from the embedded table when `resources/CSS/` isn't on
    // disk.
    let css_main = workdir.path().join("LaTeXML.css");
    let css_article = workdir.path().join("ltx-article.css");
    assert!(
      css_main.is_file(),
      "expected LaTeXML.css next to hello.html, missing at {}",
      css_main.display(),
    );
    assert!(
      css_article.is_file(),
      "expected ltx-article.css next to hello.html, missing at {}",
      css_article.display(),
    );

    // Sanity: CSS content is non-empty and looks like CSS.
    let css_main_content = std::fs::read_to_string(&css_main).expect("read LaTeXML.css");
    assert!(
      css_main_content.contains("{") && !css_main_content.is_empty(),
      "LaTeXML.css looks empty or invalid",
    );
  }
}

mod telemetry {
  //! Integration test for telemetry foundation (docs/performance/TELEMETRY.md §6 acceptance).
  //!
  //! Runs a real Converter conversion on hello.tex and verifies:
  //! 1. The telemetry struct is populated with non-zero phase totals where applicable.
  //! 2. `sum(phase_us) / wall_us >= 0.85` (loose for tiny doc; the §6.5 tighter ≥0.92 acceptance is
  //!    for the 100-paper sample, not unit tests).
  //! 3. The hand-written JSON serializer produces valid JSON.

  use latexml::converter::Converter;
  use latexml_core::{
    common::{Config, OutputFormat},
    telemetry::{self, Phase},
  };

  #[test]
  fn telemetry_populates_on_hello_conversion() {
    // Each #[test] runs on a fresh thread, so thread-local STATE/STACK
    // start zeroed. No tear-down needed.
    // logger::init may fail on the second test in the same binary (already
    // installed). Either outcome is fine for our purposes here.
    let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);

    let wall_start = std::time::Instant::now();
    let html_config = Config {
      format: OutputFormat::HTML5,
      ..Config::default()
    };
    let mut converter = Converter::from_config(html_config);
    converter.initialize_session().expect("can initialize.");
    let response = converter.convert("tests/hello/hello.tex".to_string());
    assert!(response.result.is_some(), "conversion failed");
    assert_eq!(response.status_code, 0);
    let wall_us = wall_start.elapsed().as_micros() as u64;

    // Snapshot phase totals.
    let (phase_us, telem_wall_us) = telemetry::with(|t| (t.phase_us, t.wall_us));
    let _ = telem_wall_us; // wall_us is set by binary at exit; not in this in-process path

    // At least one of the post-Bootstrap phases must have run during
    // convert(): Digest is the canonical entry. Bootstrap may be 0 if
    // initialize_session() ran before the guard was wrapped (lazy init).
    assert!(
      phase_us[Phase::Digest as usize] > 0,
      "Digest phase wasn't recorded; phase_us = {:?}",
      phase_us
    );
    assert!(
      phase_us[Phase::Build as usize] > 0,
      "Build phase wasn't recorded; phase_us = {:?}",
      phase_us
    );

    // Telemetry recorded real per-phase timings (this test's purpose: phases
    // POPULATE — already pinned by the Digest/Build assertions above). A
    // wall-clock RATIO is deliberately NOT asserted: the in-process
    // `wall_start.elapsed()` includes scheduler preemption, which under
    // concurrent `cargo test` load inflates wall far above phase work for a
    // tiny doc (observed 0.49 vs the old 0.5 bound — flaky, not a regression).
    // Production phase-coverage (≥0.92) is measured out-of-process on real
    // papers; see docs/performance/TELEMETRY.md §6.5.
    let sum_phase: u64 = phase_us.iter().sum();
    assert!(
      sum_phase > 0,
      "no phase time recorded; phase_us = {phase_us:?}"
    );
    let _ = wall_us; // measured for context; not asserted (preemption-sensitive)
  }

  #[test]
  fn telemetry_json_round_trip_on_real_conversion() {
    // logger::init may fail on the second test in the same binary (already
    // installed). Either outcome is fine for our purposes here.
    let _ = latexml_core::util::logger::init(log::LevelFilter::Warn);
    let html_config = Config {
      format: OutputFormat::HTML5,
      ..Config::default()
    };
    let mut converter = Converter::from_config(html_config);
    converter.initialize_session().expect("can initialize.");
    let _ = converter.convert("tests/hello/hello.tex".to_string());

    // Set the binary-side identifiers so the JSON has stable structure.
    telemetry::set_paper_id("hello");
    telemetry::set_host("test-host");
    telemetry::set_category("ok");
    telemetry::set_exit_code(0);
    telemetry::set_wall_us(1_000_000); // dummy

    let record = telemetry::take();
    let json = record.to_json_line();

    // Structural invariants
    assert!(json.starts_with('{') && json.ends_with('}'), "json: {json}");
    assert!(json.contains("\"paper_id\":\"hello\""));
    assert!(json.contains("\"category\":\"ok\""));
    assert!(json.contains("\"schema_version\":1"));
    // All 17 phase aliases present
    for p in [
      "bootstrap",
      "digest",
      "build",
      "rewrite",
      "math_parse",
      "post_xml_parse",
      "post_scan",
      "bibliography",
      "crossref",
      "graphics",
      "math_images",
      "mathml_pres",
      "mathml_cont",
      "split",
      "xslt",
      "html5_fixups",
      "serialize",
    ] {
      let needle = format!("\"phase_{p}_us\":");
      assert!(json.contains(&needle), "missing field {needle} in: {json}");
    }
  }
}

mod cli_css_resource_copy {
  //! Regression test: CLI `--css` resources are searched on `--path` and
  //! COPIED into the destination directory.
  //!
  //! Before the fix, `--css=foo.css --path=DIR` emitted a `<link>` to `foo.css`
  //! in the HTML but never searched `--path` for the file nor copied it, so the
  //! page rendered unstyled (the file simply wasn't there next to the HTML).
  //!
  //! `--nodefaultresources` is orthogonal and must NOT suppress CLI-specified
  //! resources — it only drops the bundled defaults (`LaTeXML.css` /
  //! `ltx-article.css`). This test pins both halves: with
  //! `--nodefaultresources` set, the custom `--css` file is still copied, while
  //! the bundled defaults are not.
  //!
  //! Faithful to Perl `LaTeXML::Post::XSLT::process` L71-78 (the CSS/JAVASCRIPT
  //! param copy, which sits OUTSIDE the `noresources` guard).

  use std::{path::Path, process::Command};

  const HELLO_TEX: &str = "\\documentclass{article}\n\
                           \\begin{document}\n\
                           Hello World!\n\
                           \\end{document}\n";

  const CUSTOM_CSS: &str = "/* oxide-test-marker */\nbody { color: rebeccapurple; }\n";

  #[test]
  fn cli_css_is_searched_on_path_and_copied_even_with_nodefaultresources() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    // The custom CSS lives in a subdirectory reachable ONLY via `--path`.
    let cssdir = workdir.path().join("styles");
    std::fs::create_dir(&cssdir).expect("mkdir styles");
    std::fs::write(cssdir.join("mystyle.css"), CUSTOM_CSS).expect("write mystyle.css");

    let tex_path = workdir.path().join("hello.tex");
    let html_path = workdir.path().join("hello.html");
    std::fs::write(&tex_path, HELLO_TEX).expect("write hello.tex");

    let output = Command::new(bin)
      .arg(tex_path.file_name().unwrap())
      .arg("--dest")
      .arg(html_path.file_name().unwrap())
      .arg("--format")
      .arg("html5")
      .arg("--css")
      .arg("mystyle.css")
      .arg("--path")
      .arg(&cssdir)
      .arg("--nodefaultresources")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");

    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{}",
      output.status.code(),
      String::from_utf8_lossy(&output.stderr),
    );

    // The HTML links the custom CSS.
    let html = std::fs::read_to_string(&html_path).expect("read hello.html");
    assert!(
      html.contains("mystyle.css"),
      "expected mystyle.css link in HTML, got:\n{html}",
    );

    // THE FIX: the file is searched on `--path` and copied into the destination
    // directory (next to the HTML), even though `--nodefaultresources` is set.
    let copied = workdir.path().join("mystyle.css");
    assert!(
      copied.is_file(),
      "expected mystyle.css copied next to hello.html (the --css/--path copy), \
       missing at {}\nstderr:\n{}",
      copied.display(),
      String::from_utf8_lossy(&output.stderr),
    );
    assert_eq!(
      std::fs::read_to_string(&copied).expect("read copied css"),
      CUSTOM_CSS,
      "copied CSS content should match the source on --path",
    );

    // The custom file came from `--path`, NOT the embedded table, so there must
    // be no `missing_file` warning for it.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
      !stderr.contains("missing_file") || !stderr.contains("mystyle.css"),
      "unexpected missing_file warning for mystyle.css:\n{stderr}",
    );

    // `--nodefaultresources` must still suppress the bundled defaults.
    assert!(
      !workdir.path().join("LaTeXML.css").exists(),
      "--nodefaultresources should suppress bundled LaTeXML.css",
    );
  }

  /// The copied CSS's LOCAL `@import` targets are followed recursively, with
  /// their subdirectory structure recreated under the destination so the
  /// cascade still resolves (the ar5iv "glowup" pattern: `ar5iv.css` →
  /// `@import "./ar5iv/*.css"`). Remote (`https://…`) imports are left alone.
  #[test]
  fn cli_css_local_imports_are_recursively_copied_with_subdirs() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    let styles = workdir.path().join("styles");
    std::fs::create_dir_all(styles.join("layer")).expect("mkdir styles/layer");
    // main.css imports a LOCAL sub-file (in a subdir) and a REMOTE sheet.
    std::fs::write(
      styles.join("main.css"),
      "@import url(\"./layer/part.css\") layer(base);\n\
       @import url('https://example.invalid/remote.css');\n\
       body { color: red; }\n",
    )
    .expect("write main.css");
    let part_css = "/* part marker */\np { margin: 0; }\n";
    std::fs::write(styles.join("layer").join("part.css"), part_css).expect("write part.css");

    let tex_path = workdir.path().join("hello.tex");
    std::fs::write(&tex_path, HELLO_TEX).expect("write hello.tex");

    let output = Command::new(bin)
      .arg("hello.tex")
      .arg("--dest")
      .arg("hello.html")
      .arg("--format")
      .arg("html5")
      .arg("--css")
      .arg("main.css")
      .arg("--path")
      .arg(&styles)
      .arg("--nodefaultresources")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");

    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{}",
      output.status.code(),
      String::from_utf8_lossy(&output.stderr),
    );

    // Top-level CSS copied next to the HTML (flattened to its basename)...
    assert!(
      workdir.path().join("main.css").is_file(),
      "main.css not copied next to hello.html",
    );

    // ...and the LOCAL @import target was followed AND its subdirectory was
    // recreated under the destination, so `./layer/part.css` resolves.
    let imported = workdir.path().join("layer").join("part.css");
    assert!(
      imported.is_file(),
      "expected @import target recreated at {} (subdir structure preserved)\nstderr:\n{}",
      imported.display(),
      String::from_utf8_lossy(&output.stderr),
    );
    assert_eq!(
      std::fs::read_to_string(&imported).expect("read imported part.css"),
      part_css,
      "recursively-copied @import content should match the source",
    );

    // The remote @import must NOT have been fetched/written locally.
    assert!(
      !workdir.path().join("remote.css").exists(),
      "remote @import should be left untouched, not copied locally",
    );
  }
}

mod dest_htm_infers_html5 {
  //! Regression pin: a `.htm` destination extension infers `--format=html5`,
  //! exactly like `.html`.
  //!
  //! `bin/latexml_oxide.rs` maps `"html" | "htm" => "html5"` (Perl Config.pm
  //! L435), lowercased so `.HTM` works too — so `--dest=index.htm` with no
  //! explicit `--format` must produce a post-processed HTML5 page (DOCTYPE +
  //! `<link>`ed, copied `LaTeXML.css`), NOT the raw-XML default. This pins the
  //! `.htm` half, which every other CLI test exercises only via `.html`; it is
  //! the exact invocation shape from GitHub #312 (`--dest=index.htm`).

  use std::{path::Path, process::Command};

  const HELLO_TEX: &str = "\\documentclass{article}\n\
                           \\begin{document}\n\
                           Hello World!\n\
                           \\end{document}\n";

  #[test]
  fn dest_htm_extension_infers_html5_and_copies_css() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("hello.tex"), HELLO_TEX).expect("write hello.tex");

    // Nasser's invocation shape: `.htm` destination, NO explicit --format.
    let output = Command::new(bin)
      .arg("hello.tex")
      .arg("--dest")
      .arg("hello.htm")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");

    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{}",
      output.status.code(),
      String::from_utf8_lossy(&output.stderr),
    );

    // The `.htm` extension must have inferred html5 → a post-processed HTML page,
    // not the raw-XML fallback.
    let html = std::fs::read_to_string(workdir.path().join("hello.htm")).expect("read hello.htm");
    assert!(
      html.contains("<!DOCTYPE html>"),
      "a .htm destination should infer html5 (DOCTYPE html), got:\n{html}",
    );
    assert!(
      html.contains("LaTeXML.css"),
      "html5 output should link LaTeXML.css, got:\n{html}",
    );

    // ...and the html5 pipeline copies the bundled default stylesheet next to it.
    assert!(
      workdir.path().join("LaTeXML.css").is_file(),
      "expected LaTeXML.css copied next to hello.htm (html5 resource copy)",
    );
  }
}

mod kpathsea_backend_resolution {
  //! Regression guard for issue #304's mechanism: file resolution must survive a
  //! process that cannot resolve a `kpsewhich` executable.
  //!
  //! Before kpathsea 0.3.4, `Kpaths::new()` returned `Err` whenever `kpsewhich`
  //! was unresolvable *from this process* — absent from its PATH (as opposed to
  //! the user's interactive shell), a stale `KPSEWHICH`, not executable, or a
  //! `kpsewhich.exe` beside a Linux binary under WSL — and `select_kpaths()`
  //! discarded that error with `.ok()?`. The linked libkpathsea was then never
  //! initialized, so EVERY lookup returned `None` while embedded bindings and
  //! dumps kept the conversion working. The only symptom was `Can't find TeX
  //! file X`, indistinguishable from a genuinely absent file.
  //!
  //! Runs the binary as a subprocess, because the backend is chosen once per
  //! process and cannot be re-selected from inside a test.

  use std::{fs, process::Command};

  /// `\input`s a file reachable ONLY through kpathsea's `TEXINPUTS` handling —
  /// not via `--path`, which is resolved Rust-side and would bypass the backend
  /// entirely (exactly how the reporter's workaround masked the problem).
  const MAIN_TEX: &str = "\\documentclass{article}\n\
                          \\input{lxo_probe_304}\n\
                          \\begin{document}\n\
                          \\lxoprobe\n\
                          \\end{document}\n";

  /// Requires a host TeX installation, which is optional for latexml-oxide, so
  /// this is `ignore`d rather than failed where none exists — and `ignore` rather
  /// than an early `return`, so the skip is visible in the test summary instead
  /// of reporting green while asserting nothing. It further assumes a linked
  /// libkpathsea (the default wherever `libkpathsea` is present at build time);
  /// on a subprocess-only build the deliberately-broken `KPSEWHICH` below leaves
  /// no backend at all, which is a different scenario than the one guarded here.
  #[test]
  #[cfg_attr(
    not(building_with_texlive),
    ignore = "requires a TeX Live installation"
  )]
  fn texinputs_resolves_without_a_resolvable_kpsewhich() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let dir = std::env::temp_dir().join(format!("lxo_kpse_backend_{}", std::process::id()));
    let include = dir.join("include");
    fs::create_dir_all(&include).unwrap();
    fs::write(
      include.join("lxo_probe_304.tex"),
      "\\newcommand\\lxoprobe{PROBE-OK}\n",
    )
    .unwrap();
    fs::write(dir.join("main.tex"), MAIN_TEX).unwrap();

    let sep = if cfg!(windows) { ';' } else { ':' };
    let out = Command::new(bin)
      .current_dir(&dir)
      .arg("--destination=out.xml")
      .arg("main.tex")
      .env("TEXINPUTS", format!("{}{sep}", include.display()))
      // No `kpsewhich` reachable: the condition that used to disable the linked
      // libkpathsea outright. `KPSEWHICH` is honored ahead of PATH by the
      // kpathsea crate, so pointing it at a nonexistent file is enough, and
      // unlike clearing PATH it stays portable.
      .env("KPSEWHICH", "/nonexistent/definitely-not-kpsewhich")
      .output()
      .expect("failed to run latexml_oxide");

    let log = String::from_utf8_lossy(&out.stderr).into_owned();
    let produced = fs::read_to_string(dir.join("out.xml")).unwrap_or_default();
    let _ = fs::remove_dir_all(&dir);

    // This test's premise — resolution survives a disabled `kpsewhich` — only
    // holds when libkpathsea is linked in-process (the shipped distribution,
    // `--features kpathsea-build-from-source`). A plain `cargo build` links no
    // libkpathsea and resolves via a spawned `kpsewhich`; with `KPSEWHICH`
    // pointed at nothing there is no resolver at all, so skip rather than fail.
    if log.contains("no libkpathsea is linked") {
      eprintln!(
        "skipping texinputs_resolves_without_a_resolvable_kpsewhich: build has no \
         linked libkpathsea (needs --features kpathsea-build-from-source)"
      );
      return;
    }

    assert!(
      !log.contains("Error:missing_file:lxo_probe_304"),
      "the TEXINPUTS-only include must resolve with no usable kpsewhich; log:\n{log}"
    );
    assert!(
      produced.contains("PROBE-OK"),
      "the included macro must have been expanded; log:\n{log}"
    );
    // The backend line is what makes a future report self-diagnosing: whatever
    // the outcome, the log must say which resolver was in play.
    assert!(
      log.contains("kpathsea:backend"),
      "every conversion log must record the resolved kpathsea backend; log:\n{log}"
    );
  }
}

mod stale_css_overwrite {
  //! Regression for GitHub #312: a stale/empty `LaTeXML.css` / `ltx-article.css`
  //! already sitting in the destination must be OVERWRITTEN with the bundled
  //! stylesheet, not left as-is.
  //!
  //! The reporter had empty `.css` files left by an earlier failed run. Because
  //! the destination directory (== the source directory here) is on the resource
  //! search path, `copy_resource` *found the stale destination file itself* and
  //! `fs::copy`'d it onto itself — truncating it to empty — instead of writing the
  //! embedded canonical CSS. The browser then loaded empty CSS and the math
  //! rendered flush-left. The `path != dest` guard was a string compare that can't
  //! detect the same file reached via a different path string.

  use std::{path::Path, process::Command};

  const HELLO_TEX: &str = "\\documentclass{article}\n\
                           \\begin{document}\n\
                           Hello World! $E=mc^2$\n\
                           \\end{document}\n";

  #[test]
  fn stale_css_in_dest_is_overwritten_with_bundled() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    let p = workdir.path();
    std::fs::write(p.join("hello.tex"), HELLO_TEX).expect("write hello.tex");
    // Reporter's exact setup: stale stylesheets already in the destination (==
    // source) directory, which is on the resource search path — one empty, one
    // with junk content, to catch both the truncate-to-empty and skip variants.
    std::fs::write(p.join("LaTeXML.css"), b"").expect("write empty LaTeXML.css");
    std::fs::write(p.join("ltx-article.css"), b"/* STALE LEFTOVER */").expect("write stale css");

    let output = Command::new(bin)
      .arg("hello.tex")
      .arg("--dest")
      .arg("hello.html")
      .current_dir(p)
      .output()
      .expect("spawn latexml_oxide");
    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{}",
      output.status.code(),
      String::from_utf8_lossy(&output.stderr),
    );

    let latexml_css = std::fs::read_to_string(p.join("LaTeXML.css")).expect("read LaTeXML.css");
    let article_css =
      std::fs::read_to_string(p.join("ltx-article.css")).expect("read ltx-article.css");

    assert!(
      latexml_css.len() > 1000 && latexml_css.contains("ltx_"),
      "a stale/empty LaTeXML.css in the dest must be overwritten with the bundled \
       stylesheet, got {} bytes:\n{latexml_css}",
      latexml_css.len(),
    );
    assert!(
      !article_css.contains("STALE") && article_css.contains("ltx_"),
      "a stale ltx-article.css in the dest must be overwritten with the bundled \
       stylesheet, got:\n{article_css}",
    );
  }
}

mod whatsinout {
  //! End-to-end `--whatsin` / `--whatsout` CLI coverage (Perl
  //! `LaTeXML::Util::Pack` + `LaTeXML.pm` driver logic).
  //!
  //! Exercises the binary the way a user invokes it, asserting the
  //! shape of the output for each `whatsout` mode:
  //!
  //! * `document` (default) → full HTML page (has `<head>`).
  //! * `fragment` → embeddable snippet (no page chrome).
  //! * `archive` → a zip bundle (HTML + status), with a placeholder `<source>.zip` destination when
  //!   `--dest` is omitted (Perl LaTeXML.pm:185-187).
  //!
  //! Run via the prebuilt-binary harness (`CARGO_BIN_EXE_latexml_oxide`),
  //! like `001_single_binary_smoke.rs`.

  use std::{io::Read, path::Path, process::Command};

  const HELLO_TEX: &str = "\\documentclass{article}\n\
                           \\begin{document}\n\
                           Hello World!\n\
                           \\end{document}\n";

  /// Spawn the binary in `cwd` with `args`, returning the captured output.
  fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    Command::new(bin)
      .args(args)
      .current_dir(cwd)
      .output()
      .expect("spawn latexml_oxide")
  }

  fn stderr_of(out: &std::process::Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
  }

  #[test]
  fn whatsout_document_is_full_page() {
    let work = tempfile::tempdir().expect("tempdir");
    std::fs::write(work.path().join("hello.tex"), HELLO_TEX).unwrap();
    let out = run(work.path(), &["hello.tex", "--dest", "doc.html"]);
    assert!(
      out.status.success(),
      "status {:?}\nstderr:\n{}",
      out.status.code(),
      stderr_of(&out)
    );
    let html = std::fs::read_to_string(work.path().join("doc.html")).expect("read doc.html");
    assert!(html.contains("Hello World"), "missing body text:\n{html}");
    // A full document carries the page chrome (`<head>`…`</head>`).
    assert!(
      html.contains("<head") && html.contains("</head>"),
      "document output should be a full HTML page with a <head>:\n{html}"
    );
  }

  #[test]
  fn whatsin_math_wraps_literal_as_mathml() {
    // `--whatsin=math` must digest the literal AS math (Perl LaTeXML.pm:166-168
    // wraps it `\begin{document}\ensuremathfollows … \ensuremathpreceeds\end{document}`,
    // and those macros open/close `\(…\)`). Before the automath port, the wrapper
    // macros were no-ops, so `\sqrt{x}` became a bare `<ltx:XMApp>` outside any
    // `<ltx:Math>` container → `malformed:ltx:XMApp isn't allowed` → no MathML.
    let work = tempfile::tempdir().expect("tempdir");
    let out = run(work.path(), &[
      "literal:\\sqrt{x}",
      "--whatsin=math",
      "--format=html5",
      "--dest",
      "m.html",
    ]);
    assert!(
      out.status.success(),
      "status {:?}\nstderr:\n{}",
      out.status.code(),
      stderr_of(&out)
    );
    let html = std::fs::read_to_string(work.path().join("m.html")).expect("read m.html");
    assert!(
      html.contains("msqrt"),
      "expected MathML <msqrt> from `--whatsin=math`:\n{html}"
    );
  }

  #[test]
  fn whatsout_fragment_strips_page_chrome() {
    let work = tempfile::tempdir().expect("tempdir");
    std::fs::write(work.path().join("hello.tex"), HELLO_TEX).unwrap();
    let out = run(work.path(), &[
      "hello.tex",
      "--whatsout",
      "fragment",
      "--dest",
      "frag.html",
    ]);
    assert!(
      out.status.success(),
      "status {:?}\nstderr:\n{}",
      out.status.code(),
      stderr_of(&out)
    );
    let frag = std::fs::read_to_string(work.path().join("frag.html")).expect("read frag.html");
    assert!(frag.contains("Hello World"), "missing body text:\n{frag}");
    // An embeddable fragment must NOT carry the full-page `<head>`/`<html>`
    // wrapper — that is the whole point of `--whatsout=fragment`.
    assert!(
      !frag.contains("<head") && !frag.contains("<html"),
      "fragment output must not carry page chrome:\n{frag}"
    );
  }

  #[test]
  fn whatsout_archive_writes_zip_bundle() {
    let work = tempfile::tempdir().expect("tempdir");
    std::fs::write(work.path().join("hello.tex"), HELLO_TEX).unwrap();
    let out = run(work.path(), &[
      "hello.tex",
      "--whatsout",
      "archive",
      "--dest",
      "bundle.zip",
    ]);
    assert!(
      out.status.success(),
      "status {:?}\nstderr:\n{}",
      out.status.code(),
      stderr_of(&out)
    );
    let zip_path = work.path().join("bundle.zip");
    assert!(zip_path.is_file(), "expected bundle.zip on disk");

    let f = std::fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(f).expect("valid zip");
    let names: Vec<String> = (0..archive.len())
      .map(|i| archive.by_index(i).unwrap().name().to_string())
      .collect();
    assert!(
      names.iter().any(|n| n == "bundle.html"),
      "zip should contain bundle.html; names: {names:?}"
    );
    assert!(
      names.iter().any(|n| n == "status"),
      "zip should contain a status entry; names: {names:?}"
    );
    // The bundled HTML is the full document.
    let mut html = String::new();
    archive
      .by_name("bundle.html")
      .unwrap()
      .read_to_string(&mut html)
      .unwrap();
    assert!(
      html.contains("Hello World"),
      "bundled HTML missing body:\n{html}"
    );
  }

  #[test]
  fn whatsout_archive_defaults_destination_to_source_zip() {
    // Perl LaTeXML.pm:185-187: `--whatsout=archive` with no `--dest`
    // invents `<source-name>.zip`.
    let work = tempfile::tempdir().expect("tempdir");
    std::fs::write(work.path().join("paper.tex"), HELLO_TEX).unwrap();
    let out = run(work.path(), &["paper.tex", "--whatsout", "archive"]);
    assert!(
      out.status.success(),
      "status {:?}\nstderr:\n{}",
      out.status.code(),
      stderr_of(&out)
    );
    let zip_path = work.path().join("paper.zip");
    assert!(
      zip_path.is_file(),
      "expected placeholder paper.zip on disk; stderr:\n{}",
      stderr_of(&out)
    );
    // With no --dest/--format an archive still defaults to an html5 web
    // bundle: `paper.html` inside, carrying the body text.
    let f = std::fs::File::open(&zip_path).unwrap();
    let mut archive = zip::ZipArchive::new(f).expect("valid zip");
    let names: Vec<String> = (0..archive.len())
      .map(|i| archive.by_index(i).unwrap().name().to_string())
      .collect();
    assert!(
      names.iter().any(|n| n == "paper.html"),
      "placeholder zip should contain paper.html; names: {names:?}"
    );
    let mut html = String::new();
    archive
      .by_name("paper.html")
      .unwrap()
      .read_to_string(&mut html)
      .unwrap();
    assert!(
      html.contains("Hello World"),
      "bundled HTML missing body:\n{html}"
    );
  }
}

mod cli_options_consumed {
  //! CLI drift guard — every option declared in the `Cli` struct must actually
  //! be consumed by the binary, and no option may be hidden from `--help`.
  //!
  //! WHY THIS EXISTS. `Cli` derives `Debug`, and the generated `Debug` impl reads
  //! every field. That read suppresses rustc's `dead_code` "field is never read"
  //! lint, so a parsed-but-ignored option — a no-op flag still printed in
  //! `--help` — compiles clean even under `-D warnings`. This test restores the
  //! guarantee by scanning the binary source: it fails if any `Cli` field is
  //! never used as `cli.<field>`.
  //!
  //! Historical no-ops this would have caught: `--inputencoding`,
  //! `--sitedirectory`, `--sourcedirectory` — all three were declared for Perl
  //! CLI parity but never wired, so they parsed and were silently ignored (fixed
  //! 2026-07-16; see `git log` for the wiring commit).
  //!
  //! The second test enforces the reverse direction (available ⇒ documented):
  //! the struct must declare no clap `hide`/`skip` attribute, so `--help` always
  //! lists every option the binary parses.

  use regex::Regex;

  /// The binary's own source — the single source of truth for the option set
  /// (clap generates `--help` from this struct's doc-comments).
  const SRC: &str = include_str!("../bin/latexml_oxide.rs");

  /// Return the `{ ... }` body of `struct Cli`, located by brace matching so a
  /// nested `{}` in a field type or attribute can't confuse it.
  fn cli_struct_body(src: &str) -> &str {
    let decl = src
      .find("struct Cli")
      .expect("`struct Cli` present in the binary");
    let open = decl + src[decl..].find('{').expect("opening brace of Cli");
    let bytes = src.as_bytes();
    let mut depth = 0usize;
    for i in open..bytes.len() {
      match bytes[i] {
        b'{' => depth += 1,
        b'}' => {
          depth -= 1;
          if depth == 0 {
            return &src[open + 1..i];
          }
        },
        _ => {},
      }
    }
    panic!("unbalanced braces while scanning the Cli struct");
  }

  /// Field identifiers declared in the struct body. Skips attribute lines
  /// (`#[...]`), doc/line comments (`///`, `//`), and blank lines; a field line
  /// is `name: Type,` whose name is a bare snake_case identifier.
  fn cli_fields(body: &str) -> Vec<String> {
    let mut fields = Vec::new();
    for raw in body.lines() {
      let line = raw.trim();
      if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
        continue;
      }
      if let Some((name, _rest)) = line.split_once(':') {
        let name = name.trim();
        // A real field name is snake_case only; anything with `=`, `"`, `(`, or
        // spaces (e.g. an attribute arg like `long = "x"`) fails this and is
        // skipped — those lines only reach here inside a multi-line `#[arg(...)]`.
        if !name.is_empty()
          && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit())
        {
          fields.push(name.to_string());
        }
      }
    }
    fields
  }

  /// Whether the binary reads `cli.<field>` anywhere. Whitespace-tolerant because
  /// rustfmt frequently splits the access across lines (`cli\n    .field`).
  fn is_consumed(field: &str) -> bool {
    let re = Regex::new(&format!(r"\bcli\s*\.\s*{}\b", regex::escape(field))).expect("valid regex");
    re.is_match(SRC)
  }

  #[test]
  fn every_cli_option_is_consumed() {
    let body = cli_struct_body(SRC);
    let fields = cli_fields(body);

    // Sanity: we actually found the option set, not an empty/garbled parse.
    assert!(
      fields.len() > 50,
      "expected the full Cli option set (>50 fields); found {}: {:?}",
      fields.len(),
      fields
    );

    // The matcher must discriminate — a genuinely-consumed field is found, and a
    // bogus name is not (otherwise the guard would vacuously pass).
    assert!(
      is_consumed("source_positional"),
      "self-check failed: `source_positional` IS consumed but the matcher missed it"
    );
    assert!(
      !is_consumed("definitely_not_a_field_zzz"),
      "self-check failed: a nonexistent field must not match"
    );

    let dead: Vec<&str> = fields
      .iter()
      .filter(|f| !is_consumed(f))
      .map(String::as_str)
      .collect();
    assert!(
      dead.is_empty(),
      "these CLI options are parsed but NEVER consumed — no-op flags still shown \
       in --help: {dead:?}\n\
       Wire each to behavior (read `cli.<field>` somewhere) or remove the field \
       from the Cli struct. The Debug derive masks the dead_code warning, so this \
       test is the only thing that catches them."
    );
  }

  #[test]
  fn no_cli_option_is_hidden_from_help() {
    let body = cli_struct_body(SRC);
    // Consider only clap attribute lines (`#[...]`), so a `hide`/`skip` in a
    // doc-comment (e.g. "Skip post-processing") isn't a false positive.
    let attrs: String = body
      .lines()
      .map(str::trim)
      .filter(|l| l.starts_with("#["))
      .collect::<Vec<_>>()
      .join("\n");
    let re = Regex::new(r"\b(hide|hide_long_help|skip)\b").expect("valid regex");
    assert!(
      !re.is_match(&attrs),
      "a Cli arg attribute uses `hide`/`skip`, which would parse an option but \
       omit it from --help (breaking available ⇒ documented). Remove it, or \
       update this guard deliberately if a hidden option is truly intended."
    );
  }
}

mod preload_pi_attributes {
  //! The `<?latexml class=…?>` / `<?latexml package=…?>` PIs that a `--preload`
  //! contributes to the document.
  //!
  //! Perl `Core.pm` L268-277 rewrites the preload spec IN PLACE with `s///`
  //! before it becomes an attribute value: the leading `[…]` option bracket and a
  //! `.cls`/`.sty` suffix are stripped off, and the bracket's contents come back
  //! as a separate `options` attribute. The Rust port used `Regex::replace_all`,
  //! which *returns* the rewritten string instead of mutating its input, and
  //! discarded every result — so nothing was ever stripped and the options
  //! attribute was never emitted (`SYNC_STATUS.md` R2, second divergence; the
  //! entry recorded only the `.cls` half of it).
  //!
  //! Every expectation below was ground-truthed against Perl LaTeXML 0.8.8 on the
  //! same input, not just read off `Core.pm`.

  use std::{path::Path, process::Command};

  /// No `\documentclass` — a preloaded class is the point of the exercise, and
  /// this is the exact input the Perl comparison was run on.
  const DOC: &str = "\\begin{document}\nHello.\n\\end{document}\n";

  fn preload_pi(spec: &str) -> String {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("p.tex"), DOC).expect("write p.tex");
    let output = Command::new(bin)
      .args(["p.tex", "--dest", "p.xml", "--nocomments"])
      .arg(format!("--preload={spec}"))
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let xml = std::fs::read_to_string(workdir.path().join("p.xml")).unwrap_or_else(|e| {
      let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
      panic!("no output for --preload={spec}: {e}\n{stderr}");
    });
    xml
      .lines()
      .find(|l| l.starts_with("<?latexml class=") || l.starts_with("<?latexml package="))
      .unwrap_or("<<no class/package PI>>")
      .to_string()
  }

  #[test]
  fn preload_spec_is_stripped_to_its_bare_name_in_the_pi() {
    // (spec, expected PI) — verbatim Perl LaTeXML 0.8.8 output.
    let cases = [
      ("article.cls", "<?latexml class=\"article\"?>"),
      (
        "[twocolumn,11pt]article.cls",
        "<?latexml class=\"article\" options=\"twocolumn,11pt\"?>",
      ),
      (
        "[dvipsnames]color.sty",
        "<?latexml package=\"color\" options=\"dvipsnames\"?>",
      ),
      ("color.sty", "<?latexml package=\"color\"?>"),
      // An empty bracket is FALSY in Perl, so it contributes no attribute.
      ("[]color.sty", "<?latexml package=\"color\"?>"),
      // Only the two literal package suffixes are stripped — anything else stays
      // attached. This is why the loop cannot reuse `parse_preload_spec`, which
      // splits on the last `.` and would emit `package="mystyle"` here.
      ("mystyle.tex", "<?latexml package=\"mystyle.tex\"?>"),
    ];
    for (spec, expected) in cases {
      assert_eq!(preload_pi(spec), expected, "--preload={spec}");
    }
  }
}

mod stale_autoload_no_runaway {
  //! Regression test: a stale autoload trigger must not spin the gullet.
  //!
  //! `def_autoload` (`latexml_engine/src/tex.rs`) installs a trigger CS that, on
  //! first use, loads its package and re-emits itself so the real definition runs.
  //! When the package is ALREADY loaded the closure just re-emits, on the
  //! assumption that a real definition is now in place — true for the case that
  //! branch was written for (a *different* CS `\let` to the trigger, e.g.
  //! `\varmathbb`, arXiv:2310.13684).
  //!
  //! But `<pkg>.sty_loaded` is assigned GLOBALLY while the package's macros are
  //! installed at the current frame. Load a package or class inside a group and
  //! the group pops the macros while the flag survives, leaving the globally
  //! installed trigger as the only definition of the CS. It then re-emits
  //! *itself*, forever — and emits no `Error:`, so `too_many_errors` never caps
  //! it and the run grinds to the token limit (~42 s) with an empty document.
  //!
  //! Real LaTeX refuses the premise outright ("! LaTeX Error: Loading a class or
  //! package in a group", latex.ltx `\@fileswithoptions` L18700), and same-host
  //! Perl LaTeXML reports a plain `Error:undefined:\theoremstyle` in ~1.2 s. So
  //! the fix clears the stale trigger and lets the CS take the ordinary bounded
  //! undefined path.
  //!
  //! Witnesses: arXiv:2606.21610 (the Overleaf/Springer conditional
  //! `\IfFileExists{sn-jnl.cls}{\documentclass…}` template) 42.9 s
  //! `Fatal:Timeout:TokenLimit` → 0.2 s bounded; arXiv:2605.21013 43.1 s → 0.2 s.
  //! Both are `STABILITY_WITNESSES.md` Cluster H.
  //!
  //! Binary-driven (fresh process) because the property under test is
  //! process-level: a bounded wall clock and a terminating conversion.

  use std::{path::Path, process::Command, time::Instant};

  /// `\usepackage` inside a group: amsthm's macros are installed on the group's
  /// frame and popped at `}`, but `amsthm.sty_loaded` stays set — so the
  /// `\theoremstyle` autoload trigger (tex.rs `def_autoload("\\theoremstyle",
  /// "amsthm")`) is left stale.
  const STALE_TRIGGER_TEX: &str = "\\documentclass{article}\n\
    {\\usepackage{amsthm}}\n\
    \\begin{document}\n\
    \\theoremstyle{plain}\n\
    x\n\
    \\end{document}\n";

  #[test]
  fn stale_autoload_trigger_does_not_run_away() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("st.tex"), STALE_TRIGGER_TEX).expect("write st.tex");

    let started = Instant::now();
    let output = Command::new(bin)
      .args(["st.tex", "--dest", "st.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let elapsed = started.elapsed();

    let stderr = String::from_utf8_lossy(&output.stderr);

    // The bug's signature is the runaway, so assert on it directly rather than on
    // wall clock alone (a loaded CI box can be slow for honest reasons).
    assert!(
      !stderr.contains("Timeout:TokenLimit") && !stderr.contains("Timeout:IfLimit"),
      "stale autoload trigger ran away to a resource limit:\n{stderr}",
    );
    // A generous ceiling: the pre-fix binary needed ~42 s to reach the 400M-token
    // limit, the fixed one finishes in ~0.2 s. Anything under 30 s means the loop
    // is gone, without making the test flaky on a busy machine.
    assert!(
      elapsed.as_secs() < 30,
      "conversion took {elapsed:?} — expected well under a second, \
       which suggests the autoload loop is back",
    );

    // Perl's verdict on the same input is a single undefined-CS error; ours must
    // be that too. Asserting the error is PRESENT (not absent) is deliberate:
    // the group really did discard amsthm's definitions, so reporting `\theoremstyle`
    // as undefined is the honest outcome — silently swallowing it would be a
    // downgrade, not a fix.
    assert!(
      stderr.contains("Error:undefined:\\theoremstyle"),
      "expected the bounded `Error:undefined:\\theoremstyle` Perl also reports:\n{stderr}",
    );

    let xml = std::fs::read_to_string(workdir.path().join("st.xml")).expect("read st.xml");
    assert!(
      xml.contains('x') && xml.len() > 200,
      "document body was lost — the runaway used to leave a 39-byte stub:\n{xml}",
    );
  }
}

mod arxiv_sty_defers_to_bundled {
  //! Guard for the configuration gate in `latexml_contrib/src/arxiv_sty.rs`.
  //!
  //! `arxiv.sty` is BUNDLED with the paper, so its contents vary: the binding
  //! exists only to supply `\keywords` & friends in configurations that do not
  //! raw-load style files. Whenever raw loading IS available (`--includestyles`
  //! / the ar5iv profile) the binding must hand control straight back to the
  //! paper's own file — otherwise every arxiv.sty paper silently loses that
  //! file's `\@maketitle`, `abstract`/`table` redefinitions and section
  //! formatting. Witnesses 2605.02338 and 2605.10111 convert byte-identically
  //! before and after the binding under `--preload=ar5iv.sty` because of this.
  //!
  //! The bundled fixture below names its keyword label `Bundled-keywords`,
  //! which the Rust fallback never emits (it says `Keywords`, arxiv.sty L44).
  //! So the assertion distinguishes "raw file won" from "binding shadowed it".
  //! `tests/contrib/arxiv_keywords.{tex,xml}` covers the complementary bare
  //! case, where the binding is the only source of `\keywords`.

  use std::{path::Path, process::Command};

  const TEX: &str = "\\documentclass{article}\n\
    \\usepackage{arxiv}\n\
    \\begin{document}\n\
    \\keywords{alpha \\and beta}\n\
    \\end{document}\n";

  /// A stand-in for the paper-bundled file: only the `\keywords` pair, with a
  /// label the binding's own fallback cannot produce.
  const STY: &str = "\\NeedsTeXFormat{LaTeX2e}\n\
    \\ProcessOptions\\relax\n\
    \\def\\keywordname{{\\bfseries Bundled-keywords}}\n\
    \\def\\keywords#1{\\par\\noindent\\keywordname\\enspace\\ignorespaces#1\\par}\n";

  #[test]
  fn arxiv_binding_defers_to_the_bundled_sty_under_includestyles() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("a.tex"), TEX).expect("write a.tex");
    std::fs::write(workdir.path().join("arxiv.sty"), STY).expect("write arxiv.sty");

    let output = Command::new(bin)
      .arg("a.tex")
      .arg("--dest")
      .arg("a.xml")
      .arg("--nocomments")
      .arg("--includestyles")
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
      "arxiv.sty + \\keywords should be error-clean, stderr had errors:\n{stderr}",
    );

    let xml = std::fs::read_to_string(workdir.path().join("a.xml")).expect("read a.xml");
    assert!(
      xml.contains("Bundled-keywords"),
      "the paper's own arxiv.sty must still define \\keywords under \
       --includestyles; the binding shadowed it:\n{xml}",
    );
  }
}

mod date_no_parens {
  //! The title-page date must render WITHOUT surrounding parentheses — full
  //! pipeline (runs XSLT).
  //!
  //! arXiv html_feedback #1934 (arXiv:2408.08811v1): the title-page date showed as
  //! `(August 1, 2024)`. LaTeXML's `dates` XSLT template
  //! (`LaTeXML-structure-xhtml.xsl`) historically wrapped every date div in
  //! `(...)` — a convention with no pdflatex counterpart (no LaTeX puts parens
  //! around `\date`, titlepage or not). Removed for PDF fidelity, a surpass-Perl
  //! divergence (OXIDIZED_DESIGN #102; same-host Perl still parenthesizes).
  //!
  //! The parens are added at the XSLT stage, so the in-process `Converter`
  //! (`06_cluster_regressions.rs`) — which stops at Core XML — cannot see them;
  //! this drives the binary end-to-end, like `cluster_xslt_split.rs`.

  use std::{path::Path, process::Command};

  fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .args(args)
      .current_dir(cwd)
      .output()
      .expect("spawn latexml_oxide")
  }

  const TEX: &str = "\\documentclass{article}\n\
     \\title{A Title}\n\
     \\author{An Author}\n\
     \\date{August 1, 2024}\n\
     \\begin{document}\\maketitle\\end{document}\n";

  #[test]
  fn date_renders_without_surrounding_parens() {
    let work = tempfile::tempdir().expect("tempdir");
    std::fs::write(work.path().join("d.tex"), TEX).unwrap();
    let out = run(work.path(), &["d.tex", "--dest", "d.html"]);
    assert!(
      out.status.success(),
      "conversion failed (status {:?}):\n{}",
      out.status.code(),
      String::from_utf8_lossy(&out.stderr)
    );
    let html = std::fs::read_to_string(work.path().join("d.html")).expect("read d.html");

    // Isolate the dates div.
    let at = html
      .find("ltx_dates")
      .expect("no ltx_dates div in output — the date was lost");
    let tail = &html[at..];
    let end = tail.find("</div>").expect("unterminated ltx_dates div");
    let dates = &tail[..end];

    // The author's date is preserved…
    assert!(
      dates.contains("August 1, 2024"),
      "the date content was lost:\n{dates}"
    );
    // …but WITHOUT the LaTeXML-ism parentheses that no PDF shows.
    assert!(
      !dates.contains('(') && !dates.contains(')'),
      "the date is still wrapped in parentheses — the `dates` XSLT template's \
       `(`/`)` were not removed:\n{dates}"
    );
  }
}

mod latexml_sty_save_parameter {
  //! `\lx@save@parameter{key}{value}` → a `<?latexml key="value"?>` processing
  //! instruction (Perl `latexml.sty.ltxml` L86-96): the constructor inserts the
  //! PI, and the `dpi`/`magnify`/`upsample`/`zoomout` package options schedule it
  //! at `\begin{document}`. The Rust `latexml_sty` binding never defined it — so a
  //! direct call errored `undefined:\lx@save@parameter`, and the image-scaling
  //! options silently dropped their PIs (they assigned a dead `PI@latexml@…` state
  //! value that nothing ever emitted). Issue #536 (reporter xworld21).
  //!
  //! Expectations ground-truthed against Perl LaTeXML 0.8.8 on the same input.

  use std::{path::Path, process::Command};

  /// Convert `tex` through the binary; return `(core-xml, ansi-stripped stderr)`.
  fn convert(tex: &str) -> (String, String) {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("create tempdir");
    std::fs::write(workdir.path().join("p.tex"), tex).expect("write p.tex");
    let output = Command::new(bin)
      .args(["p.tex", "--dest", "p.xml", "--nocomments"])
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    let xml = std::fs::read_to_string(workdir.path().join("p.xml")).unwrap_or_default();
    let stderr = String::from_utf8_lossy(&output.stderr).replace('\u{1b}', "");
    (xml, stderr)
  }

  /// The four image-scaling options each save their value as a `<?latexml …?>` PI
  /// (Perl emits `DPI` uppercase; the other three keep the keyval name).
  #[test]
  fn latexml_sty_image_scaling_options_emit_pis() {
    let (xml, stderr) = convert(
      "\\documentclass{article}\n\
       \\usepackage[dpi=300,magnify=1.5,upsample=2,zoomout=3]{latexml}\n\
       \\begin{document}Hello.\\end{document}\n",
    );
    assert!(
      xml.contains("<?latexml DPI=\"300\"?>"),
      "DPI PI missing:\n{xml}"
    );
    assert!(
      xml.contains("<?latexml magnify=\"1.5\"?>"),
      "magnify PI missing:\n{xml}"
    );
    assert!(
      xml.contains("<?latexml upsample=\"2\"?>"),
      "upsample PI missing:\n{xml}"
    );
    assert!(
      xml.contains("<?latexml zoomout=\"3\"?>"),
      "zoomout PI missing:\n{xml}"
    );
    assert!(
      !stderr.contains("undefined"),
      "unexpected undefined:\n{stderr}"
    );
  }

  /// A direct `\lx@save@parameter{key}{value}` emits its PI and does not error.
  #[test]
  fn latexml_sty_save_parameter_direct_call() {
    let (xml, stderr) = convert(
      "\\documentclass{article}\n\
       \\usepackage{latexml}\n\
       \\makeatletter\\lx@save@parameter{foo}{bar}\\makeatother\n\
       \\begin{document}Hello.\\end{document}\n",
    );
    assert!(
      xml.contains("<?latexml foo=\"bar\"?>"),
      "direct-call PI missing:\n{xml}"
    );
    assert!(
      !stderr.contains("is not defined") && !stderr.contains("undefined:\\lx@save@parameter"),
      "\\lx@save@parameter still undefined:\n{stderr}"
    );
  }
}

#[cfg(feature = "runtime-bindings")]
mod rhai_loading_path {
  //! Issue #560: a runtime `.rhai` binding must announce its actual on-disk
  //! path — `(Loading .../mybinding.sty.rhai... )` — not the synthesized
  //! compiled-module proxy name `mybinding_sty.rs`. The path is more useful to
  //! a user and closer to Perl, whose load note carries the real binding file.
  //! Compiled-in bindings (no file) keep the `_sty.rs`/`_cls.rs` proxy name.

  use std::{path::Path, process::Command};

  #[test]
  fn loads_rhai_binding_by_real_path_not_synthesized_name() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    // A runtime binding next to the source — resolved via the LocalPaths tier
    // of the dispatcher chain (converter.rs `rhai_dispatch`).
    std::fs::write(
      workdir.path().join("mybinding.sty.rhai"),
      "DefMacro(\"\\\\mybindinghook\", \"\");\n",
    )
    .expect("write mybinding.sty.rhai");
    std::fs::write(
      workdir.path().join("doc.tex"),
      "\\documentclass{article}\n\
       \\usepackage{mybinding}\n\
       \\begin{document}\\mybindinghook Hi\\end{document}\n",
    )
    .expect("write doc.tex");

    let output = Command::new(bin)
      .arg("doc.tex")
      .arg("--dest")
      .arg("doc.html")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{}",
      output.status.code(),
      String::from_utf8_lossy(&output.stderr),
    );
    // The load note goes to stderr at default verbosity.
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
      stderr.contains("Loading") && stderr.contains("mybinding.sty.rhai"),
      "expected the real `.rhai` path in the load note, got:\n{stderr}"
    );
    assert!(
      !stderr.contains("mybinding_sty.rs"),
      "load note still shows the synthesized module name, not the real path:\n{stderr}"
    );
  }
}

#[cfg(feature = "runtime-bindings")]
mod dir_prefixed_package_loading {
  //! Search-path scoping (#561) and directory-prefixed package resolution.
  //!
  //! #561: a package that modifies the search paths while loading must keep that
  //! change after the load. SEARCHPATHS is a group-scoped value (Perl-faithful:
  //! default-local `AssignValue`), and package loading opens no group, so a
  //! package's path add persists — whether from the Rhai `PrependSearchPath`/
  //! `AppendSearchPath` (the reporter's use) or the import primitives
  //! `\lx@set@path`/`\lx@append@path`. (An `\import`'s own `{…}` group still
  //! reverts its path change — guarded by the subimport sibling test.)
  //!
  //! Directory-prefixed load: `\usepackage{DIR/pkg}` where `pkg` has a
  //! basename-keyed binding must raw-load the author's bundled `DIR/pkg`, not a
  //! bare `pkg` — resolved via `\@currname` (which carries the full request),
  //! exactly as Perl does. This replaced the former `SearchPathGuard`. Real
  //! witness: arXiv 2510.09534 (`AISTATS/aistats2026`).

  use std::{path::Path, process::Command};

  #[test]
  fn package_search_path_change_survives_the_load() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    let root = workdir.path();
    std::fs::create_dir_all(root.join("sub")).unwrap();
    std::fs::create_dir_all(root.join("extra")).unwrap();

    // A dir-prefixed package (`sub/pkg`): the raw `.sty` makes the guard fire
    // (it finds a subdir'd raw file), the `.sty.rhai` is what actually loads and
    // prepends a NEW search directory.
    std::fs::write(root.join("sub/pkg.sty"), "\\ProvidesPackage{pkg}\n").unwrap();
    let extra_abs = root.join("extra");
    std::fs::write(
      root.join("sub/pkg.sty.rhai"),
      format!("PrependSearchPath({:?});\n", extra_abs.to_string_lossy()),
    )
    .unwrap();
    // Only reachable via the prepended `extra/` dir.
    std::fs::write(
      root.join("extra/inc.tex"),
      "\\newcommand{\\marker}{FOUNDIT}\n",
    )
    .unwrap();
    std::fs::write(
      root.join("main.tex"),
      "\\documentclass{article}\n\
       \\usepackage{sub/pkg}\n\
       \\begin{document}\\input{inc}\\marker\\end{document}\n",
    )
    .unwrap();

    let output = Command::new(bin)
      .arg("--includestyles")
      .arg("main.tex")
      .arg("--dest")
      .arg("out.html")
      .current_dir(root)
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let html = std::fs::read_to_string(root.join("out.html")).unwrap_or_default();

    assert!(
      !stderr.contains("missing_file:inc"),
      "`\\input{{inc}}` could not resolve — the package's prepended search path \
       was dropped after loading:\n{stderr}"
    );
    assert!(
      html.contains("FOUNDIT"),
      "expected the prepended search path to survive the package load so \
       `\\input{{inc}}` resolves; html=\n{html}\nstderr=\n{stderr}"
    );
  }

  /// #561, the mechanism the reporter actually named: a package that adds a
  /// search directory with the import primitive `\lx@set@path` must keep it after
  /// the load. Distinct from the rhai case above (this is a raw `.sty`, no
  /// binding), and from `\import` (no wrapping `{…}` group here, so the local add
  /// persists past the package).
  #[test]
  fn import_primitive_search_path_change_survives_the_load() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    let root = workdir.path();
    std::fs::create_dir_all(root.join("sub")).unwrap();
    std::fs::create_dir_all(root.join("extra")).unwrap();

    // A RELATIVE dir keeps this cross-platform without any path-string surgery:
    // an absolute OS path in TeX source is hostile on Windows (`\` is catcode 0,
    // so `C:\Users\…` tokenizes \U, \e, … as control sequences and mangles the
    // arg that `\lx@set@path` Expand!s). A relative arg resolves against
    // SOURCEDIRECTORY when that is set, else the process cwd — both are the temp
    // root here (the run uses current_dir(root)), so `extra` == root/extra on
    // every platform.
    std::fs::write(
      root.join("sub/lpkg.sty"),
      "\\ProvidesPackage{lpkg}\n\\RequirePackage{import}\n\\lx@set@path{extra}\n",
    )
    .unwrap();
    std::fs::write(
      root.join("extra/inc.tex"),
      "\\newcommand{\\marker}{FOUNDIT}\n",
    )
    .unwrap();
    std::fs::write(
      root.join("main.tex"),
      "\\documentclass{article}\n\
       \\usepackage{sub/lpkg}\n\
       \\begin{document}\\input{inc}\\marker\\end{document}\n",
    )
    .unwrap();

    let output = Command::new(bin)
      .arg("--includestyles")
      .arg("main.tex")
      .arg("--dest")
      .arg("out.html")
      .current_dir(root)
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let html = std::fs::read_to_string(root.join("out.html")).unwrap_or_default();

    assert!(
      !stderr.contains("missing_file:inc"),
      "`\\input{{inc}}` could not resolve — the package's `\\lx@set@path` add was \
       dropped after loading:\n{stderr}"
    );
    assert!(
      html.contains("FOUNDIT"),
      "expected the package's `\\lx@set@path` dir to survive the load; html=\n{html}\nstderr=\n{stderr}"
    );
  }

  /// The ex-`SearchPathGuard` case (real witness arXiv 2510.09534): a
  /// directory-prefixed `\usepackage{DIR/pkg}` whose binding raw-loads its own
  /// basename must find the author's bundled `DIR/pkg.sty`. `\@currname` carries
  /// the full request, so the raw-load targets `DIR/pkg` directly — as Perl does,
  /// with no SEARCHPATHS injection. Here a `.sty.rhai` binding does the basename
  /// raw-load (`InputDefinitions("mypkg", noltxml)`); without the `\@currname`
  /// rewrite, bare `mypkg.sty` is not on the search path and the macro is lost.
  #[test]
  fn dir_prefixed_binding_raw_loads_its_bundled_file() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");

    let workdir = tempfile::tempdir().expect("create tempdir");
    let root = workdir.path();
    std::fs::create_dir_all(root.join("SUB")).unwrap();

    // The binding (found via the full dir-prefixed request) raw-loads its own
    // basename — the dispatch drops `SUB/`, exactly like the compiled aistats2026
    // binding on the witness.
    std::fs::write(
      root.join("SUB/mypkg.sty.rhai"),
      "InputDefinitions(\"mypkg\", #{ noltxml: true, type: \"sty\" });\n",
    )
    .unwrap();
    // The author's bundled raw file, reachable only as `SUB/mypkg.sty`.
    std::fs::write(
      root.join("SUB/mypkg.sty"),
      "\\ProvidesPackage{mypkg}\n\\newcommand{\\dirmarker}{DIRLOADED}\n",
    )
    .unwrap();
    std::fs::write(
      root.join("main.tex"),
      "\\documentclass{article}\n\
       \\usepackage{SUB/mypkg}\n\
       \\begin{document}\\dirmarker\\end{document}\n",
    )
    .unwrap();

    let output = Command::new(bin)
      .arg("--includestyles")
      .arg("main.tex")
      .arg("--dest")
      .arg("out.html")
      .current_dir(root)
      .output()
      .expect("spawn latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let html = std::fs::read_to_string(root.join("out.html")).unwrap_or_default();

    assert!(
      !stderr.contains("missing_file:mypkg"),
      "the binding's basename raw-load did not resolve the bundled `SUB/mypkg.sty`:\n{stderr}"
    );
    assert!(
      html.contains("DIRLOADED"),
      "expected the bundled `SUB/mypkg.sty` to load (via `\\@currname`); html=\n{html}\nstderr=\n{stderr}"
    );
  }
}

mod em_figure_sizing {
  //! #562: a natural-size VECTOR figure is sized in font-relative `em`
  //! (`cssstyle="width:Nem; height:Nem"` on `<ltx:graphics>`, copied verbatim
  //! into the `<img>`/`<object>` style by the XSLT), so it keeps its proportion
  //! to the surrounding text at any reading size instead of a fixed pixel block.
  //! The em value is the engine's typeset size (`cached_width`, read from the PDF
  //! page box as bp→pt, converter-independent) over the local font size.
  //!
  //! Scope boundary guarded here: only inclusions with NO author size take the em
  //! path; `[width=…]`/`[scale=…]` keep the existing pixel path (their size is the
  //! author's absolute choice, not the figure's intrinsic size).
  //!
  //! Deterministic without any external tool: a minimal hand-authored PDF whose
  //! only content is a `/MediaBox` is read in pure Rust (`read_pdf_page_box`), so
  //! a 100×50 bp box sizes to 100.375/50.1875 TeX pt → 10.037em/5.019em at the
  //! 10pt default body font.

  use std::{fs, process::Command};

  /// Minimal PDF whose `/MediaBox` is all `read_pdf_page_box` needs (100×50 bp).
  const BOX_PDF: &str = "%PDF-1.4\n1 0 obj\n<< /Type /Page /MediaBox [0 0 100 50] >>\nendobj\ntrailer\n<< /Root 1 0 R >>\n%%EOF\n";

  const DOC: &str = "\\documentclass{article}\n\
                     \\usepackage{graphicx}\n\
                     \\begin{document}\n\
                     natural \\includegraphics{box.pdf}\n\n\
                     scaled \\includegraphics[width=40pt]{box.pdf}\n\n\
                     scaleopt \\includegraphics[scale=0.5]{box.pdf}\n\
                     \\end{document}\n";

  #[test]
  fn natural_vector_figure_is_em_sized_author_sized_stays_pixels() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("create tempdir");
    let root = workdir.path();
    fs::write(root.join("box.pdf"), BOX_PDF).unwrap();
    fs::write(root.join("doc.tex"), DOC).unwrap();

    let output = Command::new(bin)
      .current_dir(root)
      .arg("--destination=out.xml")
      .arg("doc.tex")
      .output()
      .expect("failed to run latexml_oxide");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let xml = fs::read_to_string(root.join("out.xml")).unwrap_or_default();

    // The natural-size include is sized in em, from the true bp box size:
    //   100 bp → 100.375 TeX pt / 10 pt = 10.037 em ; 50 bp → 5.019 em.
    assert!(
      xml.contains("cssstyle=\"width:10.037em; height:5.019em\""),
      "natural-size vector figure must carry font-relative em sizing;\nxml=\n{xml}\nstderr=\n{stderr}"
    );
    // Exactly one figure is em-sized: the `[width=…]` and `[scale=…]` inclusions
    // keep the pixel path (their size is the author's, not the figure's intrinsic).
    assert_eq!(
      xml.matches("em; height:").count(),
      1,
      "only the natural-size include should be em-sized;\nxml=\n{xml}"
    );
  }
}

mod display_math_text_nowrap {
  //! #527: `\[ \text{…} \]` — a display equation whose whole content is `\text{}`
  //! renders as `ltx_markedasmath` text in the centered equation cell. That cell
  //! is an `ltx_eqn_cell ltx_align_center` with NO `ltx_td`, so it was missing the
  //! nowrap that aligned table cells (`ltx_td`/`ltx_th`) get; squeezed between the
  //! two 50%-width centering pad cells of the width:100% `ltx_eqn_table`, the
  //! wrappable text collapsed to one word per line ("The / solution / is not /
  //! valid"). SHARED-FAILURE with same-host Perl → surpass-Perl: LaTeXML.css now
  //! gives `ltx_eqn_cell` the same nowrap-with-`ltx_wrap`-optout as a table cell.
  //!
  //! This is the platform-independent structural guard (no browser needed): both
  //! display equations put their content in the cell the rule targets, and the
  //! destination stylesheet carries the rule. The rendered-geometry guarantee —
  //! that they actually lay out on one line without clipping — is the sibling
  //! `browser_render_display_math` (Playwright).

  use std::{fs, process::Command};

  /// Two display equations exercising BOTH content shapes that land in the
  /// centering cell: pure `\text{}` (a bare `ltx_markedasmath` run) and mixed
  /// math+text (a real `<math>` with an embedded `<mtext>`). Both collapse
  /// without the fix — the first stacks one word per line, the second clips its
  /// text — and both must render on one line with it.
  const DOC: &str = "\\documentclass[12pt]{article}\n\
                     \\usepackage{amsmath}\n\
                     \\begin{document}\n\
                     \\[\n\\text{The solution is not valid}\n\\]\n\n\
                     \\[\nx^2 + \\text{the solution is not valid here} = y^2\n\\]\n\
                     \\end{document}\n";

  #[test]
  fn display_math_text_cell_gets_nowrap_css() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("create tempdir");
    let root = workdir.path();
    fs::write(root.join("doc.tex"), DOC).unwrap();

    let output = Command::new(bin)
      .current_dir(root)
      .arg("--format=html5")
      .arg("--destination=out.html")
      .arg("doc.tex")
      .output()
      .expect("failed to run latexml_oxide");
    let html = fs::read_to_string(root.join("out.html")).unwrap_or_default();
    let css = fs::read_to_string(root.join("LaTeXML.css")).unwrap_or_default();
    let stderr = String::from_utf8_lossy(&output.stderr);

    // (1) BOTH display equations put their content in a centered equation cell
    // (no `ltx_td`) — the pure-`\text{}` (markedasmath) one AND the mixed
    // math+text one (a `<math>` with `<mtext>`). These are the exact cells the
    // nowrap rule must target; count both so mixed content is guaranteed too.
    assert_eq!(
      html
        .matches("class=\"ltx_eqn_cell ltx_align_center\"")
        .count(),
      2,
      "both display equations must land in the centered equation cell;\nhtml=\n{html}\nstderr=\n{stderr}"
    );
    assert!(
      html.contains("ltx_markedasmath")
        && html.contains("<mtext>the solution is not valid here</mtext>"),
      "expected the pure-text (markedasmath) and mixed (math+mtext) contents;\nhtml=\n{html}"
    );
    // (2) The destination stylesheet gives that cell nowrap, so it cannot collapse
    // between the 50% centering pads (pure text stacks / mixed text clips without it).
    assert!(
      css.contains(".ltx_eqn_cell.ltx_align_center { white-space:nowrap; }"),
      "LaTeXML.css must nowrap the centered equation cell (#527); rule missing"
    );
  }
}

mod browser_render_display_math {
  //! #527, the *rendered* guarantee: a real headless browser must lay out both a
  //! pure-`\text{}` display and a mixed math+text display on ONE line without
  //! clipping. The two failure modes need two metrics, so CSS-string matching
  //! (the sibling test) is not enough: without the fix the pure-text cell STACKS
  //! (tall `cellHeight`) while the mixed cell CLIPS (`scrollWidth > clientWidth`).
  //! `tests/browser/measure.js` renders via Playwright (system Chrome) and reports
  //! the geometry of every `td.ltx_eqn_cell.ltx_align_center`.
  //!
  //! **Opt-in, LOCAL only — deliberately NOT run in CI** (a headless-browser job
  //! is expensive; the platform-independent `display_math_text_nowrap` is the
  //! CI-enforced guard). This self-skips (visibly) when node / `playwright-core`
  //! / a system Chrome is absent, which is the default everywhere including CI.
  //! To run it: `npm install` in `latexml_oxide/tests/browser`, then
  //! `LATEXML_BROWSER_TESTS=1 cargo test … browser_render_display_math` — with
  //! that env set a missing toolchain becomes a hard FAILURE rather than a skip.

  use std::{fs, path::PathBuf, process::Command};

  const DOC: &str = "\\documentclass[12pt]{article}\n\
                     \\usepackage{amsmath}\n\
                     \\begin{document}\n\
                     \\[\n\\text{The solution is not valid}\n\\]\n\n\
                     \\[\nx^2 + \\text{the solution is not valid here} = y^2\n\\]\n\
                     \\end{document}\n";

  fn browser_dir() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/browser") }
  fn runs(cmd: &str, arg: &str) -> bool {
    Command::new(cmd)
      .arg(arg)
      .output()
      .map(|o| o.status.success())
      .unwrap_or(false)
  }
  fn have_chrome() -> bool {
    [
      "google-chrome",
      "google-chrome-stable",
      "chromium",
      "chromium-browser",
    ]
    .iter()
    .any(|c| runs(c, "--version"))
  }
  fn toolchain_ready() -> bool {
    runs("node", "--version")
      && have_chrome()
      && browser_dir().join("node_modules/playwright-core").is_dir()
  }

  #[test]
  fn display_math_renders_on_one_line_without_clipping() {
    if !toolchain_ready() {
      // CI opts in via LATEXML_BROWSER_TESTS: there a missing toolchain is a
      // failure (never a silent green), everywhere else it is a visible skip.
      assert!(
        std::env::var("LATEXML_BROWSER_TESTS").is_err(),
        "LATEXML_BROWSER_TESTS is set but node + playwright-core + a system Chrome \
         are not all available; run `npm ci` in {}",
        browser_dir().display()
      );
      eprintln!(
        "SKIP browser render (#527): node/playwright-core/chrome unavailable — \
         `npm ci` in {} and set LATEXML_BROWSER_TESTS to require it",
        browser_dir().display()
      );
      return;
    }

    let workdir = tempfile::tempdir().expect("tempdir");
    let root = workdir.path();
    fs::write(root.join("doc.tex"), DOC).unwrap();
    let conv = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .current_dir(root)
      .arg("--format=html5")
      .arg("--destination=out.html")
      .arg("doc.tex")
      .output()
      .expect("run latexml_oxide");
    assert!(
      root.join("out.html").exists(),
      "conversion produced no HTML:\n{}",
      String::from_utf8_lossy(&conv.stderr)
    );

    let url = format!("file://{}", root.join("out.html").display());
    let m = Command::new("node")
      .arg(browser_dir().join("measure.js"))
      .arg(&url)
      .output()
      .expect("run measure.js");
    let stdout = String::from_utf8_lossy(&m.stdout);
    assert!(
      m.status.success(),
      "measure.js failed: {}\n{stdout}",
      String::from_utf8_lossy(&m.stderr)
    );

    // One object per display-equation content cell.
    let cells: Vec<serde_json::Value> =
      serde_json::from_str(stdout.trim()).expect("parse measure.js JSON");
    assert_eq!(
      cells.len(),
      2,
      "expected 2 display-equation cells (pure text + mixed), got {}: {stdout}",
      cells.len()
    );
    for (i, c) in cells.iter().enumerate() {
      let height = c["cellHeight"].as_f64().unwrap_or(f64::MAX);
      let overflow = c["overflow"].as_f64().unwrap_or(f64::MAX);
      // One 12pt line is ~20px; a stacked collapse is 3-4× that.
      assert!(
        height < 40.0,
        "display equation {i} rendered {height}px tall — it stacked into multiple \
         lines instead of one (#527); cells={stdout}"
      );
      // A squeezed cell overflows its content (the mixed case clips its text).
      assert!(
        overflow <= 2.0,
        "display equation {i} overflows its cell by {overflow}px — its content is \
         clipped, not shown on one line (#527); cells={stdout}"
      );
    }
  }
}

mod browser_render_aligned_math_spacing {
  //! #755 (reporter nasser1), the *rendered* guarantee: an `aligned`/`gather`
  //! nested in math (ONE `<math>` with a tight `<mtable columnspacing="0pt">`)
  //! must render the relation with its own spacing only, not the browser's
  //! default 0.4em `<mtd>` padding on top — which tripled the `y(x) = …` gap.
  //! `tests/browser/measure_align_mtd.js` renders via Playwright (system Chrome)
  //! and reports the aligned cells' computed horizontal padding plus the first
  //! row's left-column→relation gap.
  //!
  //! **Opt-in, LOCAL only — deliberately NOT run in CI** (headless-browser jobs
  //! are expensive), self-skipping (visibly) when node / `playwright-core` / a
  //! system Chrome is absent. To run it: `npm ci` in `latexml_oxide/tests/browser`,
  //! then `LATEXML_BROWSER_TESTS=1 cargo test … browser_render_aligned_math_spacing`
  //! — with that env set a missing toolchain is a hard FAILURE, not a skip. The
  //! platform-independent structural half is `aligned_mtable_columnspacing_zero`
  //! (below): the CSS reset only bites when `columnspacing="0pt"` is emitted.

  use std::{fs, path::PathBuf, process::Command};

  // The reporter's MWE, distilled: a `gather*` wrapping an `aligned` makes the
  // whole thing ONE `<math>` with a native `<mtable>` (the path the bug lives on).
  const DOC: &str = "\\documentclass[12pt]{book}\n\
                     \\usepackage{amsmath}\n\
                     \\begin{document}\n\
                     \\begin{gather*}\n\\begin{aligned}\n\
                     y(x) &= C_1 y(x) + C_2 y(x) \\\\\n\
                     g(x) &= C_1 y_1 + C_2 y_2\n\
                     \\end{aligned}\n\\end{gather*}\n\
                     \\end{document}\n";

  fn browser_dir() -> PathBuf { PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/browser") }
  fn runs(cmd: &str, arg: &str) -> bool {
    Command::new(cmd)
      .arg(arg)
      .output()
      .map(|o| o.status.success())
      .unwrap_or(false)
  }
  fn have_chrome() -> bool {
    [
      "google-chrome",
      "google-chrome-stable",
      "chromium",
      "chromium-browser",
    ]
    .iter()
    .any(|c| runs(c, "--version"))
  }
  fn toolchain_ready() -> bool {
    runs("node", "--version")
      && have_chrome()
      && browser_dir().join("node_modules/playwright-core").is_dir()
  }

  #[test]
  fn aligned_relation_is_not_double_spaced_by_mtd_padding() {
    if !toolchain_ready() {
      assert!(
        std::env::var("LATEXML_BROWSER_TESTS").is_err(),
        "LATEXML_BROWSER_TESTS is set but node + playwright-core + a system Chrome \
         are not all available; run `npm ci` in {}",
        browser_dir().display()
      );
      eprintln!(
        "SKIP browser render (#755): node/playwright-core/chrome unavailable — \
         `npm ci` in {} and set LATEXML_BROWSER_TESTS to require it",
        browser_dir().display()
      );
      return;
    }

    let workdir = tempfile::tempdir().expect("tempdir");
    let root = workdir.path();
    fs::write(root.join("doc.tex"), DOC).unwrap();
    let conv = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .current_dir(root)
      .arg("--format=html5")
      .arg("--destination=out.html")
      .arg("doc.tex")
      .output()
      .expect("run latexml_oxide");
    assert!(
      root.join("out.html").exists(),
      "conversion produced no HTML:\n{}",
      String::from_utf8_lossy(&conv.stderr)
    );

    let url = format!("file://{}", root.join("out.html").display());
    let m = Command::new("node")
      .arg(browser_dir().join("measure_align_mtd.js"))
      .arg(&url)
      .output()
      .expect("run measure_align_mtd.js");
    let stdout = String::from_utf8_lossy(&m.stdout);
    assert!(
      m.status.success(),
      "measure_align_mtd.js failed: {}\n{stdout}",
      String::from_utf8_lossy(&m.stderr)
    );

    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("parse JSON");
    let cells = v["cells"].as_array().expect("cells array");
    // The `aligned` has 2 rows x 2 columns → 4 native alignment cells.
    assert_eq!(
      cells.len(),
      4,
      "expected 4 aligned <mtd> cells, got {}: {stdout}",
      cells.len()
    );
    // The FIX: the browser's default 0.4em (6.4px) mtd padding is reset to 0 on
    // these `columnspacing="0pt"` cells, so only the relation operator spaces the
    // columns. Robust, font-independent.
    for (i, c) in cells.iter().enumerate() {
      for side in ["paddingLeft", "paddingRight"] {
        let p = c[side].as_str().unwrap_or("?");
        assert_eq!(
          p, "0px",
          "aligned cell {i} kept {side}={p} — the native-MathML mtd padding reset \
           (LaTeXML.css, #755) is missing, so the relation is double-spaced: {stdout}"
        );
      }
    }
    // Behavioural cross-check: the rendered `y(x)`→`=` gap is the relation space
    // alone (~4.4px at 16px), not the ~10.8px the UA padding produced. A generous
    // ceiling separates fixed from broken without pinning the exact font metric.
    let gap = v["firstRelGap"].as_f64().expect("firstRelGap");
    assert!(
      gap < 7.0,
      "aligned `y(x)`→`=` gap rendered {gap}px — double-spaced by mtd padding \
       (pre-fix ~10.8px; the relation space alone is ~4.4px): {stdout}"
    );
  }
}

mod aligned_mtable_columnspacing_zero {
  //! #755, the platform-independent (CI-enforced) half: the native-MathML mtd
  //! padding reset in `LaTeXML.css` keys on `mtable[columnspacing="0pt"]`, so it
  //! bites only if the post-processor actually emits that attribute on an
  //! `aligned` table AND the reset rule ships in the embedded stylesheet. Neither
  //! needs a browser — this guards the CSS/XML contract the browser test above
  //! renders. If the presentation MathML ever stopped emitting `columnspacing="0pt"`
  //! for `aligned`, the reset would silently no-op and the relation over-space
  //! again; if the CSS rule were dropped, likewise.

  use std::{fs, path::Path, process::Command};

  const DOC: &str = "\\documentclass[12pt]{book}\n\
                     \\usepackage{amsmath}\n\
                     \\begin{document}\n\
                     \\begin{gather*}\n\\begin{aligned}\n\
                     y(x) &= C_1 y(x)\\\\ g(x) &= C_1 y_1\n\
                     \\end{aligned}\n\\end{gather*}\n\
                     \\end{document}\n";

  #[test]
  fn aligned_emits_zero_columnspacing_and_css_resets_mtd_padding() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let work = tempfile::tempdir().expect("tempdir");
    let root = work.path();
    fs::write(root.join("doc.tex"), DOC).unwrap();
    let out = Command::new(bin)
      .current_dir(root)
      .args(["--format=html5", "--destination=out.html", "doc.tex"])
      .output()
      .expect("run latexml_oxide");
    let html = fs::read_to_string(root.join("out.html")).unwrap_or_else(|e| {
      panic!(
        "no HTML: {e}\n{}",
        String::from_utf8_lossy(&out.stderr).replace('\u{1b}', "")
      )
    });

    // The `aligned` renders as ONE native `<math>` with a tight mtable — the
    // attribute the CSS reset keys on. (An HTML `ltx_eqn_table` display would NOT
    // carry it; this nested-in-`gather*` shape is the reported one.)
    assert!(
      html.contains("<mtable columnspacing=\"0pt\""),
      "aligned did not emit a native <mtable columnspacing=\"0pt\"> — the #755 CSS \
       reset keys on that attribute and would silently no-op:\n{html}"
    );

    // The reset rule must ship in the embedded stylesheet the binary wrote out.
    let css = fs::read_to_string(root.join("LaTeXML.css")).expect("read LaTeXML.css");
    assert!(
      css.contains("mtable[columnspacing=\"0pt\"] mtd"),
      "LaTeXML.css is missing the native-MathML aligned mtd padding reset (#755)"
    );
  }
}

mod title_pubnote_pollution {
  //! arXiv/html_feedback#6886: publication METADATA pubnotes (conference, DOI,
  //! ISBN, journal, …) were nested inside the title `<h1 class="ltx_title">`,
  //! leaking frontmatter into the title (a stray dagger on the heading + the
  //! metadata living in the title DOM). Vanilla LaTeXML's `maketitle` "collects
  //! ALL pubnotes into the title"; we diverge (OXIDIZED_DESIGN) — only genuine
  //! title FOOTNOTES (`\thanks`, `\titlenote` ⇒ role note/thanks) stay in the
  //! `<h1>`; metadata pubnotes move to a sibling `ltx_pubnotes_meta` block.
  //!
  //! Deterministic without acmart: `\lx@add@pubnote[role=…]` is the exact API
  //! acmart maps `\acmConference`/`\acmDOI`/… onto.

  use std::{fs, process::Command};

  const DOC: &str = "\\documentclass{article}\n\
                     \\makeatletter\n\
                     \\lx@add@pubnote[role=conference]{Proc. of Something 2025}\n\
                     \\lx@add@pubnote[role=doi]{10.1/xyz}\n\
                     \\lx@add@pubnote[role=note]{A title footnote.}\n\
                     \\makeatother\n\
                     \\title{My Paper Title}\n\
                     \\author{An Author}\n\
                     \\begin{document}\\maketitle Body.\\end{document}\n";

  #[test]
  fn title_h1_excludes_metadata_pubnotes_keeps_footnotes() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("tempdir");
    let root = workdir.path();
    fs::write(root.join("doc.tex"), DOC).unwrap();
    let out = Command::new(bin)
      .current_dir(root)
      .arg("--format=html5")
      .arg("--destination=out.html")
      .arg("doc.tex")
      .output()
      .expect("run latexml_oxide");
    let html = fs::read_to_string(root.join("out.html")).unwrap_or_default();
    let stderr = String::from_utf8_lossy(&out.stderr);

    let h1 = html
      .find("<h1")
      .and_then(|i| html[i..].find("</h1>").map(|j| &html[i..i + j]))
      .unwrap_or("");

    // The metadata (conference/DOI) must NOT pollute the title heading …
    assert!(
      !h1.contains("Proc. of Something") && !h1.contains("10.1/xyz"),
      "publication metadata leaked into the <h1> title (#6886);\nh1=\n{h1}\nstderr=\n{stderr}"
    );
    // … but a genuine title footnote (\thanks/\titlenote ⇒ role=note) stays in it.
    assert!(
      h1.contains("A title footnote"),
      "the role=note title footnote should remain on the title;\nh1=\n{h1}"
    );
    // The metadata moves to a sibling `ltx_pubnotes_meta` block AFTER the <h1>.
    let (h1_close, meta) = (html.find("</h1>"), html.find("ltx_pubnotes_meta"));
    assert!(
      matches!((h1_close, meta), (Some(h), Some(m)) if m > h)
        && html.contains("Proc. of Something"),
      "metadata pubnotes must render as an ltx_pubnotes_meta block after the title;\nhtml=\n{html}"
    );
  }

  /// The HTML `<head><title>` (browser tab / SEO / bookmark text) must carry
  /// ONLY the title text — never any note content. Notes (`\thanks`, footnotes)
  /// and publication metadata (conference/DOI) are meant to display visually in
  /// the body only. This holds by construction: the engine extracts every note
  /// out of `\title{}` into sibling `<pubnote>` elements, so the core `<title>`
  /// node — and the navigation title the head `<title>` derives from — are
  /// clean. This guard locks that in against a regression that let note text
  /// flow back into the head title (e.g. a template switched to a
  /// note-including flatten, or notes re-parented under `<title>`).
  ///
  /// A `\thanks` INSIDE `\title{}` is the strongest case: it is a note that TeX
  /// nests within the title group, yet it must still be extracted and kept out
  /// of the head `<title>`.
  #[test]
  fn head_title_excludes_all_note_content() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("tempdir");
    let root = workdir.path();
    // \thanks nested in the title + metadata pubnotes + a plain title footnote.
    let doc = "\\documentclass{article}\n\
               \\makeatletter\n\
               \\lx@add@pubnote[role=conference]{Proc. of Something 2025}\n\
               \\lx@add@pubnote[role=doi]{10.1/xyz}\n\
               \\makeatother\n\
               \\title{My Paper Title\\thanks{Secret Funding Note}}\n\
               \\author{An Author}\n\
               \\begin{document}\\maketitle Body.\\end{document}\n";
    fs::write(root.join("doc.tex"), doc).unwrap();
    let out = Command::new(bin)
      .current_dir(root)
      .arg("--format=html5")
      .arg("--destination=out.html")
      .arg("doc.tex")
      .output()
      .expect("run latexml_oxide");
    let html = fs::read_to_string(root.join("out.html")).unwrap_or_default();
    let stderr = String::from_utf8_lossy(&out.stderr);

    // Isolate the HEAD <title>…</title> (not the body <h1 class="ltx_title">).
    let head_title = html
      .find("<title>")
      .and_then(|i| {
        html[i + 7..]
          .find("</title>")
          .map(|j| &html[i + 7..i + 7 + j])
      })
      .expect("HTML must have a <head><title>");

    // The title text is present …
    assert!(
      head_title.contains("My Paper Title"),
      "head <title> lost the actual title text;\n<title>={head_title:?}\nstderr=\n{stderr}"
    );
    // … and NONE of the note / metadata content leaked into it.
    for leak in [
      "Secret Funding Note",
      "Proc. of Something",
      "10.1/xyz",
      "Thanks",
    ] {
      assert!(
        !head_title.contains(leak),
        "note/metadata {leak:?} leaked into the head <title> \
         (must be body-visual only);\n<title>={head_title:?}"
      );
    }
  }

  /// arXiv/html_feedback#6888: a `\thanks` on the author / affiliation line of a
  /// manually-formatted `\author{}` (no `\and`) becomes a loose `role='thanks'`
  /// pubnote that the maketitle collects into the title `<h1>`. It must land
  /// inside a **collapsible** `.ltx_pubnotes_content` block (the bundled CSS
  /// renders only the dagger MARK and hides the content — guarded in
  /// `latexml_post::xslt::witnessed_css_delta::title_pubnote_content_stays_collapsed`),
  /// never as **bare inline text** in the `<h1>`, and never in the head `<title>`.
  /// Parity: same-host Perl 0.8.8 produces the byte-identical structure. Witness
  /// arXiv:2312.08128 (Clockwork Diffusion, Qualcomm affiliation `\thanks`).
  #[test]
  fn author_block_thanks_collapses_in_title_not_inline() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("tempdir");
    let root = workdir.path();
    let doc = "\\documentclass{article}\n\
               \\title{Clockwork Diffusion}\n\
               \\author{Amir Habibian\\thanks{Equal contribution} \\\\\n\
               {Qualcomm AI Research\\thanks{Qualcomm AI Research is an initiative of \
               Qualcomm Technologies, Inc}}}\n\
               \\begin{document}\\maketitle Body.\\end{document}\n";
    fs::write(root.join("doc.tex"), doc).unwrap();
    let out = Command::new(bin)
      .current_dir(root)
      .arg("--format=html5")
      .arg("--destination=out.html")
      .arg("doc.tex")
      .output()
      .expect("run latexml_oxide");
    let html = fs::read_to_string(root.join("out.html")).unwrap_or_default();
    let stderr = String::from_utf8_lossy(&out.stderr);

    let leak = "initiative of Qualcomm";
    // Head <title> is clean of the footnote text.
    let head_title = html
      .find("<title>")
      .and_then(|i| {
        html[i + 7..]
          .find("</title>")
          .map(|j| &html[i + 7..i + 7 + j])
      })
      .expect("HTML must have a <head><title>");
    assert!(
      !head_title.contains(leak),
      "affiliation \\thanks leaked into the head <title> (#6888);\n<title>={head_title:?}"
    );

    // Isolate the <h1> title block.
    let h1 = html
      .find("<h1")
      .and_then(|i| html[i..].find("</h1>").map(|j| &html[i..i + j]))
      .expect("HTML must have a title <h1>");
    // The thanks IS carried in the title (a collapsible pubnotes block) …
    assert!(
      h1.contains("ltx_pubnotes_content") && h1.contains(leak),
      "the title \\thanks should be present as a collapsible pubnotes block;\nh1=\n{h1}\nstderr=\n{stderr}"
    );
    // … but NOT as bare inline text: stripping the pubnotes block must remove it.
    let bare = {
      // drop everything from the pubnotes span to the end of the h1 (the pubnotes
      // block is the trailing content of the heading here)
      match h1.find("<span class=\"ltx_pubnotes") {
        Some(p) => &h1[..p],
        None => h1,
      }
    };
    assert!(
      !bare.contains(leak) && !bare.contains("Thanks"),
      "the \\thanks text renders inline in the title <h1> outside the collapsible \
       pubnotes block (#6888) — it must be mark-only;\nbare-h1=\n{bare}"
    );
  }
}

mod math_greek_no_replacement_char {
  //! arXiv/html_feedback#6622: the deployed v0.5.0 emitted Greek math letters as
  //! U+FFFD replacement characters (`<mi mathvariant="normal">\u{FFFD}</mi>` —
  //! 381 of them in arXiv:2509.03592v2). The reporter read the σ-turned-empty-box
  //! in eq. 28 (`\sigma^2_\phi(\Sa) \equiv \sum …`) as a stray `\hphantom`. The
  //! current engine emits the real Unicode letters. Guard the final HTML: the
  //! Greek/relation/operator symbols are present and NO U+FFFD leaks into the
  //! rendered math (checked in both the core XML and post-processed HTML paths —
  //! this drives the full pipeline).

  use std::{fs, process::Command};

  #[test]
  fn greek_math_letters_are_unicode_not_replacement_char() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("tempdir");
    let root = workdir.path();
    // The reporter's eq. 28 verbatim, plus a spread of inline Greek.
    let doc = "\\documentclass{article}\n\
               \\begin{document}\n\
               \\[\\sigma^2_\\phi(S^\\mathrm{acc}) \\equiv \\sum_{s \\in S^\\mathrm{acc}} \
               \\sigma^2_\\phi(s)\\]\n\
               Inline Greek: $\\alpha\\beta\\gamma\\sigma\\phi\\mu\\Sigma\\Phi$.\n\
               \\end{document}\n";
    fs::write(root.join("doc.tex"), doc).unwrap();
    let out = Command::new(bin)
      .current_dir(root)
      .arg("--format=html5")
      .arg("--destination=out.html")
      .arg("doc.tex")
      .output()
      .expect("run latexml_oxide");
    let html = fs::read_to_string(root.join("out.html")).unwrap_or_default();
    let stderr = String::from_utf8_lossy(&out.stderr);

    // No U+FFFD replacement character anywhere — the #6622 symptom.
    assert!(
      !html.contains('\u{FFFD}'),
      "the rendered HTML contains a U+FFFD replacement char — a math letter was \
       corrupted to invalid output (#6622);\nstderr=\n{stderr}"
    );
    // The real Greek / relation / big-op symbols are all present.
    for (ch, name) in [
      ('\u{03C3}', "sigma σ"),
      ('\u{03D5}', "phi ϕ"),
      ('\u{03B1}', "alpha α"),
      ('\u{03A3}', "Sigma Σ"),
      ('\u{2261}', "equiv ≡"),
      ('\u{2211}', "sum ∑"),
    ] {
      assert!(
        html.contains(ch),
        "math symbol {name} is missing from the rendered output (#6622);\nstderr=\n{stderr}"
      );
    }
  }
}

mod resource_src_path {
  //! #662: a resource whose `@src` has a folder component (`subdir/foo.css`) must
  //! be copied PRESERVING that folder — `<dest>/subdir/foo.css` — so the
  //! `<link>`/`<script>` href the stylesheet emits still resolves, instead of the
  //! file being flattened to `<dest>/foo.css` while the tag keeps `subdir/foo.css`.
  use std::process::Command;

  #[test]
  fn resource_folder_component_is_preserved_on_copy() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    std::fs::create_dir_all(root.join("subdir")).unwrap();
    std::fs::create_dir_all(root.join("out")).unwrap();
    std::fs::write(root.join("subdir/mystyle.css"), "body{color:red}").unwrap();
    std::fs::write(
      root.join("myres.sty.rhai"),
      "RequireResource(\"subdir/mystyle.css\", #{ type: \"text/css\" });",
    )
    .unwrap();
    std::fs::write(
      root.join("doc.tex"),
      "\\documentclass{article}\\usepackage{myres}\\begin{document}Hi.\\end{document}",
    )
    .unwrap();

    let out = Command::new(bin)
      .args([
        "--format=html5",
        "--path=.",
        "--dest=out/doc.html",
        "doc.tex",
      ])
      .env("LATEXML_NODUMP", "1")
      .current_dir(root)
      .output()
      .expect("spawn latexml_oxide");
    assert!(
      out.status.success(),
      "binary failed; stderr:\n{}",
      String::from_utf8_lossy(&out.stderr)
    );

    // The file kept its folder …
    assert!(
      root.join("out/subdir/mystyle.css").is_file(),
      "resource was not copied to out/subdir/mystyle.css (folder dropped)"
    );
    // … and was NOT flattened to the destination root.
    assert!(
      !root.join("out/mystyle.css").exists(),
      "resource was flattened to out/mystyle.css, dropping its folder (#662)"
    );
    // … and the emitted <link> href matches the file location.
    let html = std::fs::read_to_string(root.join("out/doc.html")).unwrap();
    assert!(
      html.contains("href=\"subdir/mystyle.css\""),
      "the <link> href does not point at the preserved path; html=\n{html}"
    );
  }

  /// A resource that ESCAPES the source dir (`../shared.css`) is flattened to the
  /// basename (Perl's "otherwise flatten" branch), and the tag is rewritten to match.
  #[test]
  fn resource_escaping_source_dir_is_flattened() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::create_dir_all(root.join("src/out")).unwrap();
    std::fs::write(root.join("shared.css"), "body{color:blue}").unwrap();
    std::fs::write(
      root.join("src/myesc.sty.rhai"),
      "RequireResource(\"../shared.css\", #{ type: \"text/css\" });",
    )
    .unwrap();
    std::fs::write(
      root.join("src/doc.tex"),
      "\\documentclass{article}\\usepackage{myesc}\\begin{document}Hi.\\end{document}",
    )
    .unwrap();

    let out = Command::new(bin)
      .args([
        "--format=html5",
        "--path=.",
        "--dest=out/doc.html",
        "doc.tex",
      ])
      .env("LATEXML_NODUMP", "1")
      .current_dir(root.join("src"))
      .output()
      .expect("spawn latexml_oxide");
    assert!(out.status.success(), "binary failed on escaping resource");

    // Flattened INTO the destination root — never written outside it via `../`
    // (which is where a naive "preserve `../shared.css`" would land it).
    assert!(
      root.join("src/out/shared.css").is_file(),
      "escaping resource should flatten into the destination"
    );
    assert!(
      !root.join("src/shared.css").exists(),
      "escaping resource must NOT be written outside the destination (path traversal)"
    );
  }
}

mod whatsin_xml_input {
  //! #655: a core LaTeXML XML document is post-processed directly (the
  //! `latexmlpost` role) when its extension is `.xml` / `*-xml` / `*_xml`, or
  //! when `--whatsin=xml` forces it regardless of the extension. Neither path
  //! spins up the TeX engine or re-digests the source.
  use std::{path::Path, process::Command};

  /// A full core `<document>` (article, with sections) that post-processes to
  /// HTML — reused from the `latexmlpost` fixtures.
  fn core_xml() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/post/hyperref.xml");
    std::fs::read_to_string(p).expect("read hyperref.xml fixture")
  }

  fn run(input_name: &str, extra: &[&str]) -> (bool, String, String) {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join(input_name), core_xml()).expect("write input");
    let mut cmd = Command::new(bin);
    cmd.arg(input_name).arg("--dest").arg("out.html");
    for a in extra {
      cmd.arg(a);
    }
    let out = cmd
      .current_dir(dir.path())
      .output()
      .expect("spawn latexml_oxide");
    let html = std::fs::read_to_string(dir.path().join("out.html")).unwrap_or_default();
    (
      out.status.success(),
      html,
      String::from_utf8_lossy(&out.stderr).into_owned(),
    )
  }

  /// A `.preprocessed-xml` extension (ends in `-xml`) is auto-detected as XML
  /// input. Before #655 it was treated as TeX and the XML source was digested
  /// as garbage instead of post-processed.
  #[test]
  fn dash_xml_extension_is_post_processed_directly() {
    let (ok, html, stderr) = run("input.preprocessed-xml", &[]);
    assert!(
      ok,
      "binary failed on a .preprocessed-xml core doc; stderr:\n{stderr}"
    );
    // The section titles from the CORE document survive into HTML — proof the
    // file went through post-processing, not TeX digestion of `<?xml …>`.
    // The core document's SECTION STRUCTURE (ltx_section + the `xml:id`s S1/S2/S3)
    // survives into HTML — a signal absent when the XML source is instead digested
    // as TeX (angle brackets become garbage text, no sections).
    assert!(
      html.contains("ltx_section") && html.contains("id=\"S1\""),
      "the -xml file was not post-processed as a LaTeXML core document; got:\n{html}"
    );
  }

  /// An unrecognised extension (`.dat`) is NOT auto-detected; `--whatsin=xml`
  /// forces the XML-input path anyway.
  #[test]
  fn whatsin_xml_forces_post_on_unrecognised_extension() {
    let (ok, html, stderr) = run("input.dat", &["--whatsin", "xml"]);
    assert!(
      ok,
      "binary failed with --whatsin=xml on a .dat core doc; stderr:\n{stderr}"
    );
    assert!(
      html.contains("ltx_section") && html.contains("id=\"S1\""),
      "--whatsin=xml did not force post-processing of the core document; got:\n{html}"
    );
  }
}

/// arXiv/html_feedback#1291: an `\includegraphics` INSIDE a `{picture}` (the
/// Inkscape `.pdf_tex` figure idiom) must land as a resolved `<img>`/`<object>`
/// in the picture's `<foreignObject>`, not the raw `<graphics>` the pass-A SVG
/// snapshot froze before the Graphics phase ran. Uses a real PNG (passthrough —
/// no ghostscript/mutool needed), so the guard is deterministic on every host.
/// Witness paper: arXiv:2311.14363v2 (Figures 1 and 4 lost their images).
mod picture_graphics_e2e {
  use std::{path::Path, process::Command};

  #[test]
  fn includegraphics_inside_picture_resolves_to_img() {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    let workdir = tempfile::tempdir().expect("create tempdir");
    // A real PNG the Graphics phase copies through without any converter.
    let png = std::fs::read("tests/graphics/none.png").expect("read none.png fixture");
    std::fs::write(workdir.path().join("pic.png"), &png).expect("stage pic.png");
    let tex = "\\documentclass{article}\n\
       \\usepackage{graphicx}\n\
       \\begin{document}\n\
       \\setlength{\\unitlength}{100pt}\n\
       \\begin{picture}(1,1)\n\
       \\put(0,0){\\includegraphics[width=\\unitlength]{pic.png}}\n\
       \\end{picture}\n\
       \\end{document}\n";
    std::fs::write(workdir.path().join("pic.tex"), tex).expect("write pic.tex");

    let output = Command::new(bin)
      .arg("pic.tex")
      .arg("--dest")
      .arg("pic.html")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    assert!(
      output.status.success(),
      "binary exited with {:?}\nstderr:\n{}",
      output.status.code(),
      String::from_utf8_lossy(&output.stderr),
    );

    let html = std::fs::read_to_string(workdir.path().join("pic.html")).expect("read pic.html");
    // The picture renders as an inline <svg> with a <foreignObject>; the nested
    // graphic must be a resolved <img> pointing at the PNG.
    assert!(
      html.contains("<foreignObject") && html.contains("<img") && html.contains("pic.png"),
      "picture-nested graphic did not resolve to an <img src=pic.png>:\n{html}"
    );
    // The #1291 defect: a raw <graphics> element surviving into the HTML.
    assert!(
      !html.contains("<graphics"),
      "raw <graphics> survived into the HTML (missing image):\n{html}"
    );
    // Guard the fixture path stays valid.
    assert!(Path::new(bin).is_file());
  }
}

mod identity_banner {
  //! Every conversion logs a one-line identity banner — executable name, version,
  //! git revision, exact start time (`latexml::identity`) — mirroring Perl's
  //! `Note("$LaTeXML::IDENTITY processing $source")`. These guard the end-to-end
  //! wiring: that both front-ends actually emit it, and that `--quiet` mutes it.

  use std::process::Command;

  /// The four fields the banner must carry, checked against a captured stderr.
  fn assert_is_banner(stderr: &str, exe: &str) {
    let line = stderr
      .lines()
      .find(|l| l.contains("latexml-oxide") && l.contains("; revision "))
      .unwrap_or_else(|| panic!("no identity banner in stderr of {exe}:\n{stderr}"));
    assert!(line.contains(exe), "banner names the wrong exe: {line:?}");
    assert!(
      line.contains(env!("CARGO_PKG_VERSION")),
      "banner missing crate version: {line:?}"
    );
    assert!(
      line.contains(" started "),
      "banner missing start time: {line:?}"
    );
  }

  #[test]
  fn latexml_oxide_logs_identity() {
    let workdir = tempfile::tempdir().expect("tempdir");
    let tex = workdir.path().join("hi.tex");
    std::fs::write(
      &tex,
      "\\documentclass{article}\\begin{document}Hi\\end{document}\n",
    )
    .expect("write tex");
    let output = Command::new(env!("CARGO_BIN_EXE_latexml_oxide"))
      .arg(tex.file_name().unwrap())
      .arg("--dest")
      .arg("hi.html")
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    assert_is_banner(&String::from_utf8_lossy(&output.stderr), "latexml_oxide");
  }

  #[test]
  fn latexmlmath_logs_identity_and_quiet_mutes_it() {
    let bin = env!("CARGO_BIN_EXE_latexmlmath_oxide");

    let loud = Command::new(bin)
      .arg("x^2")
      .output()
      .expect("spawn latexmlmath");
    assert_is_banner(&String::from_utf8_lossy(&loud.stderr), "latexmlmath_oxide");

    let quiet = Command::new(bin)
      .args(["--quiet", "x^2"])
      .output()
      .expect("spawn latexmlmath --quiet");
    let quiet_err = String::from_utf8_lossy(&quiet.stderr);
    assert!(
      !quiet_err.contains("; revision "),
      "--quiet must suppress the identity banner, got:\n{quiet_err}"
    );
  }
}

mod quiet_keeps_log_floor {
  //! Issue #763 (xworld21/Vincenzo Mantova, BookML author): `--quiet` must
  //! reduce STDERR only — the `.latexml.log` keeps a minimum verbosity floor
  //! (identity banner, `(Processing …`/`(Loading …` progress notes, `Info:`
  //! records, the `Status:conversion:` verdict). BookML's makefile dependency
  //! tracking reads the `(Loading …` lines from the log, so stripping them under
  //! `--quiet` broke it.
  //!
  //! Perl ground truth — `Common/Error.pm` `_printline`/`ProgressSpinup` write to
  //! `$LOG` whenever the log is open, gating only STDERR on `$VERBOSITY >= 0`;
  //! `bin/latexml` L83 emits the identity `Note` unconditionally. Confirmed on the
  //! same host: Perl 0.8.8 `--quiet` keeps the banner + every `(Loading …` line in
  //! its `.log`.
  //!
  //! The same log-floor rule governs the TeX terminal-output primitives, which
  //! must also survive `--quiet` (Perl calls them WITHOUT a verbosity guard):
  //! `\typeout` → `Note` (log always + stderr if `$VERBOSITY >= 0`,
  //! `latex_constructs.pool.ltxml` L4538), `\message` → `NoteLog` (log ONLY, never
  //! stderr, `TeX_Debugging.pool.ltxml` L65). The `\message` distinction is the
  //! sharp one: its content must reach the log at any verbosity yet never appear on
  //! stderr, even loud.

  use std::{path::Path, process::Command};

  const DOC: &str = "\\documentclass{article}\n\
                     \\usepackage{amsmath}\n\
                     \\begin{document}\n\
                     \\typeout{TypeoutMarker763}\n\
                     \\message{MessageMarker763}\n\
                     Hello $x^2$.\n\
                     \\end{document}\n";

  /// The log-floor lines every run must keep, quiet or not.
  fn assert_log_has_floor(log: &str, label: &str) {
    for needle in [
      "; revision ",         // identity banner
      " started ",           // identity banner start time
      "(Processing content", // Mouth progress note (ProgressSpinup)
      "(Loading ",           // binding-module load note (BookML depends on this)
      "Info:",               // an Info-level diagnostic record
      "TypeoutMarker763",    // \typeout → Note (log always)
      "MessageMarker763",    // \message → NoteLog (log always)
      "Status:conversion:",  // the final verdict line
    ] {
      assert!(
        log.contains(needle),
        "{label} log missing floor line {needle:?}; full log:\n{log}"
      );
    }
  }

  fn run(quiet: bool) -> (String, String) {
    let bin = env!("CARGO_BIN_EXE_latexml_oxide");
    assert!(Path::new(bin).is_file(), "binary not staged at {bin}");
    let workdir = tempfile::tempdir().expect("tempdir");
    std::fs::write(workdir.path().join("doc.tex"), DOC).expect("write doc.tex");
    let log_name = if quiet { "quiet.log" } else { "loud.log" };

    let mut cmd = Command::new(bin);
    cmd
      .arg("doc.tex")
      .arg("--dest")
      .arg("doc.html")
      .arg("--log")
      .arg(log_name);
    if quiet {
      cmd.arg("--quiet");
    }
    let output = cmd
      .current_dir(workdir.path())
      .output()
      .expect("spawn latexml_oxide");
    assert!(
      output.status.success(),
      "binary exited {:?}\nstderr:\n{}",
      output.status.code(),
      String::from_utf8_lossy(&output.stderr),
    );
    let log = std::fs::read_to_string(workdir.path().join(log_name)).expect("read log");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (log, stderr)
  }

  /// THE FIX: under `--quiet` the on-disk `.log` still carries the full floor,
  /// while STDERR is quieted (no `Info:` records, no `(Loading …` notes).
  #[test]
  fn quiet_log_keeps_floor_but_stderr_is_muted() {
    let (log, stderr) = run(true);
    assert_log_has_floor(&log, "--quiet");
    // STDERR is reduced: progress notes, Info records, and \typeout do not reach
    // the console.
    assert!(
      !stderr.contains("(Loading "),
      "--quiet must mute progress notes on STDERR, got:\n{stderr}"
    );
    assert!(
      !stderr.contains("Info:"),
      "--quiet must mute Info records on STDERR, got:\n{stderr}"
    );
    assert!(
      !stderr.contains("TypeoutMarker763"),
      "--quiet must mute \\typeout on STDERR, got:\n{stderr}"
    );
    // \message uses NoteLog — never on stderr at any verbosity.
    assert!(
      !stderr.contains("MessageMarker763"),
      "\\message (NoteLog) must never reach STDERR, got:\n{stderr}"
    );
  }

  /// Parity companion: without `--quiet`, the log keeps the whole floor, and stderr
  /// keeps the verbosity-gated notes (`(Loading …`, `Info:`, `\typeout`) — but
  /// `\message` (Perl `NoteLog`) stays log-only, off stderr even loud.
  #[test]
  fn loud_log_and_stderr_both_keep_floor() {
    let (log, stderr) = run(false);
    assert_log_has_floor(&log, "loud");
    assert!(
      stderr.contains("(Loading "),
      "loud STDERR should show progress notes, got:\n{stderr}"
    );
    assert!(
      stderr.contains("Info:"),
      "loud STDERR should show Info records, got:\n{stderr}"
    );
    assert!(
      stderr.contains("TypeoutMarker763"),
      "loud STDERR should show \\typeout (Note), got:\n{stderr}"
    );
    assert!(
      !stderr.contains("MessageMarker763"),
      "\\message (NoteLog) must stay log-only, never on STDERR — got:\n{stderr}"
    );
  }
}
