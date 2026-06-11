//! Main `.tex` discovery for directory inputs.
//!
//! Equivalent to the binary's `--whatsin=directory` mode: given a
//! source directory, return the path of the file the converter should
//! treat as the top-level entrypoint. Lifted from
//! `bin/latexml_oxide.rs` so embedders (e.g. `ar5iv-editor`,
//! `cortex_worker`) can run the same heuristic without shelling out.
//!
//! Detection order:
//!  1. `00README.json` "sources" entry with `usage == "toplevel"` (modern arXiv format).
//!  2. `00README.XXX` line tagged `toplevelfile` (legacy arXiv). Lines tagged `ignore` exclude the
//!     named file from later heuristic scanning (Perl `Pack.pm::detect_source` `unlink`s them; we
//!     filter the candidate list instead — safer if the directory isn't a sandbox).
//!  3. Pack.pm-derived likelihood scoring across every `.tex` / `.txt` / `.ltx` (and, as a
//!     fallback, every long-extensioned or extension-less file). Files vetoed by `\input` /
//!     `\include` references are excluded; `\documentclass` / `\documentstyle` and Mac-classic
//!     markers boost the score; bibtex / metafont / %auto-ignore / withdrawal sentinels disqualify.
//!
//! Returns the absolute path of the chosen entry as a `PathBuf`. The
//! `Err` arm carries a Perl-canonical `Fatal:invalid:not_tex_source`
//! style message (matching what the binary printed before this lift).

use std::path::{Path, PathBuf};

use once_cell::sync::Lazy;
use regex::Regex;

/// Discover the main `.tex` file inside `dir`, mirroring the binary's
/// `--whatsin=directory` heuristic. The returned path is rooted at
/// `dir`. Errors carry a human-readable diagnostic on `Fatal:` failure
/// modes (PDF mis-named as TeX, only `%auto-ignore` files, no TeX at
/// all).
pub fn find_main_tex(dir: &Path) -> Result<PathBuf, String> {
  // Phase I.1: Check 00README.json (2025 arXiv format)
  // Format: { "sources": [{"filename": "main.tex", "usage": "toplevel"}, ...] }
  if let Some(filename) = parse_readme_json(dir) {
    let main_path = dir.join(&filename);
    if main_path.exists() {
      return Ok(main_path);
    }
  }

  // Phase I.1.2: Check 00README.XXX (legacy arXiv format).
  // Two directive kinds supported (Perl Pack.pm L82-97):
  //   `<name> toplevelfile` → shortcut, return directly.
  //   `<name> ignore`       → exclude `<name>` from heuristic
  //                            candidate scanning below.
  // Perl additionally `unlink`s the ignored path; we instead carry
  // the names through as a Phase-I.2 filter set — safer (no
  // filesystem mutation in case the directory isn't a sandbox).
  let mut ignored_names: rustc_hash::FxHashSet<PathBuf> = rustc_hash::FxHashSet::default();
  let readme_xxx = dir.join("00README.XXX");
  if readme_xxx.exists()
    && let Ok(content) = std::fs::read_to_string(&readme_xxx)
  {
    for line in content.lines() {
      let parts: Vec<&str> = line.split_whitespace().collect();
      if parts.len() < 2 {
        continue;
      }
      match parts[1] {
        "toplevelfile" => {
          let main_path = dir.join(parts[0]);
          if main_path.exists() {
            return Ok(main_path);
          }
        },
        "ignore" => {
          ignored_names.insert(dir.join(parts[0]));
        },
        _ => {},
      }
    }
  }

  // Phase I.2: Heuristic detection (ported from arXiv::FileGuess via Pack.pm)
  let mut tex_files: Vec<PathBuf> = Vec::new();
  collect_tex_files(dir, &mut tex_files, false);
  if !ignored_names.is_empty() {
    tex_files.retain(|p| !ignored_names.contains(p));
  }
  let candidates_before_pdf_filter = tex_files.len();
  tex_files.retain(|p| !is_pdf_magic(p));
  if tex_files.is_empty() && candidates_before_pdf_filter > 0 {
    return Err(s(
      "Fatal:invalid:not_tex_source PDF magic detected in source file (no TeX-format files in archive)",
    ));
  }
  if tex_files.is_empty() {
    collect_tex_files(dir, &mut tex_files, true);
    tex_files.retain(|p| !is_pdf_magic(p));
    if !ignored_names.is_empty() {
      tex_files.retain(|p| !ignored_names.contains(p));
    }
  }
  if tex_files.is_empty() {
    return Err(s("No .tex files found in directory"));
  }

  // Score each file: likelihood 0-3 (Perl: Main_TeX_likelihood)
  let mut likelihood: rustc_hash::FxHashMap<PathBuf, f32> = rustc_hash::FxHashMap::default();
  // (vetoed_path, vetoer_path) — the veto is honored at filtering
  // time only when the vetoer's score >= the vetee's. Prevents a
  // 2-line wrapper file (e.g. `\input{main}`) from removing the
  // documentclass-bearing main.tex. Witness 2307.13586.
  let mut vetoed: Vec<(PathBuf, PathBuf)> = Vec::new();
  let mut had_auto_ignore = false;

  for tex_file in &tex_files {
    if !tex_file.exists() {
      continue;
    }
    let Ok(raw) = std::fs::read(tex_file) else {
      continue;
    };
    let content = String::from_utf8_lossy(&raw);
    let mut maybe_tex = false;
    let mut maybe_tex_priority = false;
    let mut maybe_tex_priority2 = false;
    let mut determined = false;

    for (lineno, raw_line) in content.lines().enumerate() {
      let lineno1 = lineno + 1;
      // Perl L117-120: early-line checks (first 10-12 lines)
      if lineno1 <= 10
        && (RE_AUTOIGNORE.is_match(raw_line)
          || RE_TEXINFO.is_match(raw_line)
          || RE_AUTOINCLUDE.is_match(raw_line))
      {
        likelihood.insert(tex_file.clone(), 0.0);
        if RE_AUTOIGNORE.is_match(raw_line) {
          had_auto_ignore = true;
        }
        determined = true;
        break;
      }
      if lineno1 <= 12
        && let Some(cap) = RE_FORMAT_HINT.captures(raw_line)
      {
        let fmt = &cap[1];
        if fmt == "latex209" || fmt == "biglatex" || fmt == "latex" || fmt == "LaTeX" {
          likelihood.insert(tex_file.clone(), 3.0);
        } else {
          likelihood.insert(tex_file.clone(), 1.0);
        }
        determined = true;
        break;
      }
      // Perl L128: strip ONE `%`-comment up to the next `\r`. `\r`-aware
      // so bare-`\r` line-ended files (read as one big "line" in Perl
      // because `$/=\n`) preserve subsequent `\r\documentclass` chunks.
      let stripped: std::borrow::Cow<str> = RE_STRIP_COMMENT.replacen(raw_line, 1, "");
      let line: &str = &stripped;

      if RE_DOCCLASS.is_match(line) {
        likelihood.insert(tex_file.clone(), 3.0);
        determined = true;
        break;
      }
      if RE_MAYBE_TEX.is_match(line) {
        maybe_tex = true;
      }
      // Perl L133-148: \input/\include → veto the included file
      if let Some(cap) = RE_INPUT_INCLUDE.captures(line) {
        maybe_tex = true;
        let mut vetoed_name = cap[1].to_string();
        if RE_AMSTEX.is_match(&vetoed_name) {
          likelihood.insert(tex_file.clone(), 2.0);
          determined = true;
          break;
        }
        if !vetoed_name.contains('.') {
          vetoed_name = vetoed_name.trim_end().to_string() + ".tex";
        }
        let base_dir = tex_file.parent().unwrap_or(dir);
        vetoed.push((base_dir.join(&vetoed_name), tex_file.clone()));
      }
      if RE_END_BYE.is_match(line) {
        maybe_tex_priority = true;
      }
      if RE_END_BYE2.is_match(line) {
        maybe_tex_priority2 = true;
      }
      if RE_MAC_TEX.is_match(line) {
        likelihood.insert(tex_file.clone(), 1.0);
        determined = true;
        break;
      }
      if RE_METAFONT.is_match(line) {
        likelihood.insert(tex_file.clone(), 0.0);
        determined = true;
        break;
      }
      if RE_BIBTEX.is_match(raw_line) {
        likelihood.insert(tex_file.clone(), 0.0);
        determined = true;
        break;
      }
      if RE_UUENCODE.is_match(raw_line) {
        if maybe_tex_priority {
          likelihood.insert(tex_file.clone(), 2.0);
        } else if maybe_tex {
          likelihood.insert(tex_file.clone(), 1.0);
        } else {
          likelihood.insert(tex_file.clone(), 0.0);
        }
        determined = true;
        break;
      }
      if RE_WITHDRAWN.is_match(line) {
        likelihood.insert(tex_file.clone(), 0.0);
        determined = true;
        break;
      }
    }
    if !determined {
      let score = if maybe_tex_priority {
        2.0
      } else if maybe_tex_priority2 {
        1.5
      } else if maybe_tex {
        1.0
      } else {
        0.0
      };
      likelihood.insert(tex_file.clone(), score);
    }
  }

  for (vetee, vetoer) in &vetoed {
    let vetee_score = likelihood.get(vetee).copied().unwrap_or(0.0);
    let vetoer_score = likelihood.get(vetoer).copied().unwrap_or(0.0);
    if vetoer_score >= vetee_score {
      likelihood.remove(vetee);
    }
  }

  let mut candidates: Vec<PathBuf> = likelihood
    .keys()
    .filter(|f| likelihood[*f] > 0.0)
    .cloned()
    .collect();
  candidates.sort_by(|a, b| likelihood[b].partial_cmp(&likelihood[a]).unwrap());

  if candidates.is_empty() {
    if had_auto_ignore {
      // Perl-faithful: process %auto-ignore sources as normal (the
      // `%` is a comment, the rest is empty → empty XML output, no
      // Fatal). Witness: 2307.10758 (12-byte `%auto-ignore` source) —
      // Perl reports "Conversion complete: No obvious problems"; the
      // old Rust path turned 90 wp4 corpus entries into hard
      // failures. Cortex_worker has a sibling fix; both must stay in
      // sync. Prefer the dirname-matching .tex (arxiv convention
      // `<id>/<id>.tex`), else the first available.
      let dir_name = dir.file_name().and_then(|s| s.to_str()).unwrap_or_default();
      let auto_ignore_main = tex_files
        .iter()
        .find(|p| {
          p.file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(|stem| stem == dir_name)
        })
        .cloned()
        .or_else(|| tex_files.first().cloned());
      if let Some(p) = auto_ignore_main {
        return Ok(p);
      }
    }
    return Err(s("No viable .tex files found in directory"));
  }

  let max_score = likelihood[&candidates[0]];
  candidates.retain(|f| (likelihood[f] - max_score).abs() < f32::EPSILON);

  if candidates.len() > 1 {
    let min_depth = candidates
      .iter()
      .map(|f| f.strip_prefix(dir).unwrap_or(f).components().count())
      .min()
      .unwrap_or(0);
    candidates.retain(|f| f.strip_prefix(dir).unwrap_or(f).components().count() == min_depth);
  }

  if candidates.len() > 1 {
    let pdf_candidates: Vec<PathBuf> = candidates
      .iter()
      .filter(|f| {
        std::fs::read(f).ok().is_some_and(|raw| {
          let c = String::from_utf8_lossy(&raw);
          c.contains("\\includegraphics")
            && (c.contains(".pdf") || c.contains(".png") || c.contains(".jpg"))
        })
      })
      .cloned()
      .collect();
    if !pdf_candidates.is_empty() {
      candidates = pdf_candidates;
    }
  }

  if candidates.len() > 1 {
    let bbl_candidates: Vec<PathBuf> = candidates
      .iter()
      .filter(|f| f.with_extension("bbl").exists())
      .cloned()
      .collect();
    if !bbl_candidates.is_empty() {
      candidates = bbl_candidates;
    }
  }

  if candidates.len() > 1 {
    let common: Vec<PathBuf> = candidates
      .iter()
      .filter(|f| {
        f.file_name().is_some_and(|n| {
          let n = n.to_str().unwrap_or("");
          n == "main.tex" || n == "ms.tex" || n == "paper.tex"
        })
      })
      .cloned()
      .collect();
    if !common.is_empty() {
      candidates = common;
    }
  }

  candidates.sort();
  Ok(
    candidates
      .into_iter()
      .next()
      .expect("non-empty after filtering"),
  )
}

// ---------------------------------------------------------------------------
// Internal helpers.
// ---------------------------------------------------------------------------

fn s(msg: &str) -> String { msg.to_string() }

// Perl Pack.pm L25 TEX_EXT = qr/\.(?:[tT](:?[eE][xX]|[xX][tT])|ltx|LTX)$/
// → .tex, .txt, .ltx (case-insensitive). The `fallback` arm matches Perl
// Pack/Dir.pm L47: `!/\./ || /\.[^.]{4,}$/` — extension-less or extension
// ≥4 chars, used when nothing TeX-shaped surfaces in the strict pass.
fn collect_tex_files(dir: &Path, files: &mut Vec<PathBuf>, fallback: bool) {
  if let Ok(entries) = std::fs::read_dir(dir) {
    for entry in entries.flatten() {
      let path = entry.path();
      if path.is_dir() {
        collect_tex_files(&path, files, fallback);
      } else if !fallback {
        if path.extension().is_some_and(|e| {
          let e = e.to_ascii_lowercase();
          e == "tex" || e == "txt" || e == "ltx"
        }) {
          files.push(path);
        }
      } else {
        let ext_opt = path.extension().and_then(|e| e.to_str());
        let keep = match ext_opt {
          None => true,
          Some(ext) => ext.len() >= 4,
        };
        if keep {
          files.push(path);
        }
      }
    }
  }
}

// Skip files whose magic bytes identify them as PDF (e.g. arXiv source
// archives that contain a PDF mis-named with a `.tex` extension).
pub fn is_pdf_magic(path: &Path) -> bool {
  let mut buf = [0u8; 5];
  if let Ok(mut f) = std::fs::File::open(path) {
    use std::io::Read;
    if f.read(&mut buf).is_ok_and(|n| n == 5) {
      return &buf == b"%PDF-";
    }
  }
  false
}

/// Parse `00README.json` in `dir` and return the "filename" of the
/// toplevel source. Perl Pack.pm L68-80: scans `sources[]` for the
/// entry tagged `usage == "toplevel"`. Minimal hand-rolled JSON
/// scanner — we don't pull a full JSON dep just for this.
fn parse_readme_json(dir: &Path) -> Option<String> {
  let content = std::fs::read_to_string(dir.join("00README.json")).ok()?;
  let sources_start = content.find("\"sources\"")?;
  let rest = &content[sources_start..];
  let arr_start = rest.find('[')?;
  let arr_end = rest.find(']')?;
  let arr = &rest[arr_start + 1..arr_end];

  for obj_str in arr.split('}') {
    if !obj_str.contains("\"toplevel\"") {
      continue;
    }
    if let Some(fn_pos) = obj_str.find("\"filename\"") {
      let after_key = &obj_str[fn_pos + 10..];
      let after_key = after_key.trim_start();
      let after_key = after_key.strip_prefix(':')?;
      let after_key = after_key.trim_start();
      let after_key = after_key.strip_prefix('"')?;
      let mut result = String::new();
      for ch in after_key.chars() {
        match ch {
          '"' => break,
          '\\' => continue,
          c => result.push(c),
        }
      }
      if !result.is_empty() {
        return Some(result);
      }
    }
  }
  None
}

// ---------------------------------------------------------------------------
// Pre-compiled regexes used by `find_main_tex`. Parking these as module-
// level `Lazy<Regex>` keeps a single instance per process and avoids the
// per-call recompile that nesting them inside the function caused.
// ---------------------------------------------------------------------------

static RE_AUTOIGNORE: Lazy<Regex> = Lazy::new(|| Regex::new(r"%auto-ignore").unwrap());
static RE_TEXINFO: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\input texinfo").unwrap());
static RE_AUTOINCLUDE: Lazy<Regex> = Lazy::new(|| Regex::new(r"%auto-include").unwrap());
static RE_FORMAT_HINT: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\r?%&(\S+)").unwrap());
static RE_DOCCLASS: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(?:^|\r)\s*\\document(?:style|class)").unwrap());
static RE_MAYBE_TEX: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"(?:^|\r)\s*\\(?:font|magnification|input|def|special|baselineskip|begin)").unwrap()
});
static RE_INPUT_INCLUDE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\\(?:input|include)(?:\s+|\{)([^ \}]+)").unwrap());
static RE_END_BYE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(?:^|\r)\s*\\(?:end|bye)(?:\s|$)").unwrap());
static RE_END_BYE2: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\(?:end|bye)(?:\s|$)").unwrap());
static RE_MAC_TEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\\input *(?:harv|lanl)mac|\\input\s+phyzzx").unwrap());
static RE_METAFONT: Lazy<Regex> = Lazy::new(|| Regex::new(r"beginchar\(").unwrap());
static RE_BIBTEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(?i)(?:^|\r)@(?:book|article|inbook|unpublished)\{").unwrap());
static RE_UUENCODE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^begin \d{1,4}\s+\S+\r?$").unwrap());
static RE_WITHDRAWN: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"paper deliberately replaced by what little").unwrap());
static RE_AMSTEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^amstex$").unwrap());
// Strip a single `%`-comment, stopping at the next `\r` so bare-`\r`
// line-ended files (Mac classic) preserve post-comment `\documentclass`.
static RE_STRIP_COMMENT: Lazy<Regex> = Lazy::new(|| Regex::new(r"%[^\r]*").unwrap());

#[cfg(test)]
mod tests {
  use tempfile::tempdir;

  use super::*;

  fn write(dir: &Path, name: &str, body: &str) { std::fs::write(dir.join(name), body).unwrap(); }

  #[test]
  fn picks_documentclass_over_input_only_file() {
    let d = tempdir().unwrap();
    write(
      d.path(),
      "main.tex",
      "\\documentclass{article}\n\\begin{document}x\\end{document}",
    );
    write(d.path(), "intro.tex", "\\input{intro_body}\n");
    let pick = find_main_tex(d.path()).unwrap();
    assert_eq!(pick.file_name().unwrap(), "main.tex");
  }

  #[test]
  fn errors_when_directory_has_no_tex() {
    let d = tempdir().unwrap();
    let err = find_main_tex(d.path()).unwrap_err();
    assert!(err.contains("No .tex files"));
  }

  #[test]
  fn prefers_main_over_other_documentclass_files() {
    let d = tempdir().unwrap();
    write(d.path(), "draft.tex", "\\documentclass{article}\n");
    write(d.path(), "main.tex", "\\documentclass{article}\n");
    let pick = find_main_tex(d.path()).unwrap();
    assert_eq!(pick.file_name().unwrap(), "main.tex");
  }

  #[test]
  fn readme_xxx_toplevelfile_directive_short_circuits() {
    let d = tempdir().unwrap();
    // Two candidate documents — main.tex would normally win,
    // but the 00README directive points at draft.tex.
    write(d.path(), "draft.tex", "\\documentclass{article}\n");
    write(d.path(), "main.tex", "\\documentclass{article}\n");
    write(d.path(), "00README.XXX", "draft.tex toplevelfile\n");
    let pick = find_main_tex(d.path()).unwrap();
    assert_eq!(pick.file_name().unwrap(), "draft.tex");
  }

  #[test]
  fn readme_xxx_ignore_directive_excludes_candidate() {
    let d = tempdir().unwrap();
    // Without a directive, `main.tex` would tie with `old.tex`
    // and the "common name" heuristic prefers `main.tex`. With
    // an `ignore` directive on main.tex, the heuristic falls back
    // to `old.tex`.
    write(d.path(), "old.tex", "\\documentclass{article}\n");
    write(d.path(), "main.tex", "\\documentclass{article}\n");
    write(d.path(), "00README.XXX", "main.tex ignore\n");
    let pick = find_main_tex(d.path()).unwrap();
    assert_eq!(pick.file_name().unwrap(), "old.tex");
  }

  #[test]
  fn readme_xxx_mixed_directives() {
    let d = tempdir().unwrap();
    write(d.path(), "intro.tex", "\\documentclass{article}\n");
    write(d.path(), "paper.tex", "\\documentclass{article}\n");
    write(d.path(), "junk.tex", "\\documentclass{article}\n");
    // `paper.tex toplevelfile` short-circuits the heuristic; the
    // ignore line is unreachable in this test but exercises the
    // parser's multi-line/multi-kind handling.
    write(
      d.path(),
      "00README.XXX",
      "junk.tex ignore\npaper.tex toplevelfile\n",
    );
    let pick = find_main_tex(d.path()).unwrap();
    assert_eq!(pick.file_name().unwrap(), "paper.tex");
  }
}
