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
  let decl = src.find("struct Cli").expect("`struct Cli` present in the binary");
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
  let re = Regex::new(&format!(r"\bcli\s*\.\s*{}\b", regex::escape(field)))
    .expect("valid regex");
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
