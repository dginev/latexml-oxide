# Kernel Dump Precompilation Design

## Problem

Processing `latex.ltx` + `expl3-code.tex` at startup takes ~2s and loads 37K lines of TeX. A precompiled dump avoids this overhead. The design goal: `cargo build` should check the TeX Live version and embed the correct dump automatically, using only cargo (build.rs).

## Bootstrap Problem

Generating the dump requires running our engine. The engine is what we're building. `build.rs` cannot invoke `cargo run` (nested cargo is forbidden). Therefore: **build.rs can only embed an existing dump, not generate one from scratch.**

## Design: Text Dump + include_str!

### Architecture

```
resources/dumps/
  latex.dump.txt      # 25,172 entries, ~3.7MB (git-ignored, regenerated via tools/make_formats.sh)
  texlive.version     # kpsewhich version hash (for staleness detection)
  # (plain.dump.txt previously lived here as a checked-in 438-entry file,
  # now superseded by inline compiled plain_dump.rs via dump_codegen.)

latexml_package/build.rs
  → checks resources/dumps/ for cached dumps
  → validates against TeX Live version (kpsewhich --version)
  → generates $OUT_DIR/plain_dump_loader.rs and $OUT_DIR/latex_dump_loader.rs
  → if dump missing: generates no-op stub + prints cargo:warning

latexml_package/src/engine/
  plain_dump.rs       # include!(concat!(env!("OUT_DIR"), "/plain_dump_loader.rs"))
  latex_dump.rs       # include!(concat!(env!("OUT_DIR"), "/latex_dump_loader.rs"))
```

### build.rs Logic (Phase 1)

```rust
fn main() {
  // 1. Check TeX Live availability
  let texlive_version = Command::new("kpsewhich")
    .arg("--version")
    .output()
    .ok()
    .and_then(|o| String::from_utf8(o.stdout).ok());

  // 2. For each dump (plain, latex):
  for (name, dump_file) in [("plain", "plain.dump.txt"), ("latex", "latex.dump.txt")] {
    let dump_path = format!("resources/dumps/{}", dump_file);
    let loader_path = format!("{}/{}_dump_loader.rs", out_dir, name);

    if Path::new(&dump_path).exists() {
      // Embed the dump as include_str! + load_from_str
      fs::write(&loader_path, format!(r#"
        pub fn load_definitions() -> latexml_core::common::error::Result<()> {{
          let content = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/dumps/{dump_file}"));
          let count = latexml_core::dump_reader::load_from_str(content)
            .map_err(|e| latexml_core::common::error::Error::msg(e))?;
          log::info!("Loaded {{}} kernel definitions from {} dump", count);
          Ok(())
        }}
      "#)).unwrap();
    } else {
      // No dump available — generate no-op stub
      fs::write(&loader_path, STUB).unwrap();
      println!("cargo:warning=No {name} kernel dump found. Run: cargo run --release --bin latexml_oxide -- --init={name}.tex");
    }

    println!("cargo:rerun-if-changed=resources/dumps/{dump_file}");
  }

  // 3. Staleness check
  if let Some(version) = texlive_version {
    let version_file = "resources/dumps/texlive.version";
    let cached = fs::read_to_string(version_file).unwrap_or_default();
    if cached.trim() != version.trim() {
      println!("cargo:warning=TeX Live version changed. Regenerate dumps: cargo run --release --bin latexml_oxide -- --init=latex.ltx");
    }
  }
}
```

### Dump Generation (Manual, One-Time)

```bash
# Generate both dumps:
cargo run --release --bin latexml_oxide -- --init=plain.tex
cargo run --release --bin latexml_oxide -- --init=latex.ltx

# This writes:
#   resources/dumps/plain.dump.txt
#   resources/dumps/latex.dump.txt
#   resources/dumps/texlive.version
```

### Why Text Format, Not Compiled Rust

| Approach | plain.tex (438 entries) | latex.ltx (21905 entries) |
|----------|------------------------|--------------------------|
| Compiled Rust (.rs) | 4465 lines, ~1s compile | ~350K lines, ~30s compile |
| Text + include_str! | ~50KB, 0s compile, ~5ms parse | ~8MB, 0s compile, ~50ms parse |

The text format adds negligible runtime cost (50ms one-time parse) but saves 30s of compilation. The Rust source approach for `plain_dump.rs` (current) should be migrated to text format too.

### What Changes

1. **plain_dump.rs**: Replace 4465-line compiled Rust with `include!` (matches latex_dump.rs pattern)
2. **build.rs**: Add TeX Live version check + dump embedding logic
3. **ini_tex.rs**: Write dumps to `resources/dumps/` instead of `src/engine/`
4. **resources/dumps/**: New directory, `.dump.txt` files git-ignored (too large), `texlive.version` checked in
5. **plain.dump.txt**: Small enough to check into git (~50KB)

### Future: Full Automation (Phase 2)

Once the bootstrap problem is solved for initial builds, `build.rs` can optionally:
- Run `latexml_oxide --init=latex.ltx` if the binary already exists in target/
- Use a `[build-dependencies]` approach with a separate dump-gen crate
- Cache dumps in `$CARGO_HOME` across projects

Phase 2 is not needed for the initial implementation.
