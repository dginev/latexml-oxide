//! Guards for batch E0 / Z0 of the old frontmatter engine's removal (`docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md`, the
//! user's ruling 2026-11-06: the old engine's code is deleted before the merge). Nothing here changes a conversion:
//! the fence, the token-read count and the pass list read the source, the read-back runs under the census switch. Each
//! pinned count is a debt on the way to the floor, not a permanent allowance: it may only fall, and a fall lowers its pin
//! (the failure message prints the table to paste), so what was paid stays paid. Witnesses of the read-back: 2012.14750,
//! 2201.02056 (overflow positions), 2606.18623, 1507.02068.
use std::{collections::BTreeMap, fs, path::Path};

use latexml::util::test::{convert_with_setup_then, error_count, warning_count};
use latexml_engine::base_utilities::{fm_shadow_json, set_fm_shadow_override};
use regex::Regex;
use serde_json::{Value, json};

const BASE: &str = "latexml_engine/src/base_utilities";

fn repo_path(path: &str) -> std::path::PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(path)
}

/// Every `.rs` file under `path` (a file: itself), repo-relative, sorted.
fn rust_files(path: &str) -> Vec<String> {
  let full = repo_path(path);
  if full.is_file() {
    return vec![path.to_string()];
  }
  let mut out = Vec::new();
  for entry in fs::read_dir(&full).unwrap_or_else(|e| panic!("{path}: {e}")) {
    let entry = entry.expect("a directory entry");
    let name = format!("{path}/{}", entry.file_name().to_string_lossy());
    if entry.path().is_dir() {
      out.extend(rust_files(&name));
    } else if name.ends_with(".rs") {
      out.push(name);
    }
  }
  out.sort();
  out
}

/// `path`'s lines, each up to its first `//`: over-approximate on code, since a string holding `//` is cut there too, and
/// never counts a comment.
fn code_lines(path: &str) -> Vec<String> {
  fs::read_to_string(repo_path(path))
    .unwrap_or_else(|e| panic!("{path}: {e}"))
    .lines()
    .map(|line| line.split("//").next().unwrap_or("").to_string())
    .collect()
}

/// A Rust source as the compiler reads it: no comments, each literal a string or a character, a raw one too, one literal `"<n>"`
/// in `code` ([`RustSource::literals`]'s index), so a guard's pattern reads across lines and nothing in a literal looks like code.
struct RustSource {
  code:     String,
  literals: Vec<String>,
}

/// What a string's or a character's source says, its escapes read; `\` and a line's end join the lines.
fn unescape(body: &str) -> String {
  let mut out = String::new();
  let mut chars = body.chars().peekable();
  while let Some(c) = chars.next() {
    if c != '\\' {
      out.push(c);
      continue;
    }
    match chars.next() {
      Some('n') => out.push('\n'),
      Some('t') => out.push('\t'),
      Some('r') => out.push('\r'),
      Some('0') => out.push('\0'),
      Some('x') => {
        let hex: String = chars.by_ref().take(2).collect();
        out.extend(u8::from_str_radix(&hex, 16).ok().map(char::from));
      },
      Some('u') => {
        let hex: String = chars
          .by_ref()
          .take_while(|c| *c != '}')
          .filter(|c| *c != '{' && *c != '_')
          .collect();
        out.extend(u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32));
      },
      Some('\n') => {
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
          chars.next();
        }
      },
      Some(other) => out.push(other),
      None => {},
    }
  }
  out
}

/// Read `text` ([`RustSource`]); what `#[cfg(test)]` holds is no code of the engine's, so it goes (a fixture spells what it likes).
fn lex_rust(text: &str) -> RustSource {
  let chars: Vec<char> = text.chars().collect();
  let at = |i: usize| chars.get(i).copied();
  let (mut code, mut literals, mut i) = (String::new(), Vec::new(), 0);
  let push = |code: &mut String, literals: &mut Vec<String>, literal: String| {
    code.push_str(&format!("\"{}\"", literals.len()));
    literals.push(literal);
  };
  while let Some(c) = at(i) {
    if c == '/' && at(i + 1) == Some('/') {
      while at(i).is_some_and(|c| c != '\n') {
        i += 1;
      }
    } else if c == '/' && at(i + 1) == Some('*') {
      let (mut depth, mut j) = (1, i + 2);
      while depth > 0 && j < chars.len() {
        if at(j) == Some('/') && at(j + 1) == Some('*') {
          depth += 1;
          j += 2;
        } else if at(j) == Some('*') && at(j + 1) == Some('/') {
          depth -= 1;
          j += 2;
        } else {
          j += 1;
        }
      }
      i = j;
    } else if c == '"' {
      let mut j = i + 1;
      while at(j).is_some_and(|c| c != '"') {
        j += if at(j) == Some('\\') { 2 } else { 1 };
      }
      push(
        &mut code,
        &mut literals,
        unescape(&chars[i + 1..j.min(chars.len())].iter().collect::<String>()),
      );
      i = j + 1;
    } else if c == '\'' && (at(i + 1) == Some('\\') || at(i + 2) == Some('\'')) {
      let mut j = i + 1;
      while at(j).is_some_and(|c| c != '\'') {
        j += if at(j) == Some('\\') { 2 } else { 1 };
      }
      push(
        &mut code,
        &mut literals,
        unescape(&chars[i + 1..j.min(chars.len())].iter().collect::<String>()),
      );
      i = j + 1;
    } else if c.is_alphanumeric() || c == '_' {
      // (a word whole, so that the `r` ending a name opens no raw string)
      let start = i;
      while at(i).is_some_and(|c| c.is_alphanumeric() || c == '_') {
        i += 1;
      }
      let word: String = chars[start..i].iter().collect();
      let mut hashes = 0;
      while at(i + hashes) == Some('#') {
        hashes += 1;
      }
      if matches!(word.as_str(), "r" | "br" | "cr") && at(i + hashes) == Some('"') {
        let body = i + hashes + 1;
        let mut j = body;
        while j < chars.len()
          && !(at(j) == Some('"') && (1..=hashes).all(|n| at(j + n) == Some('#')))
        {
          j += 1;
        }
        push(
          &mut code,
          &mut literals,
          chars[body..j.min(chars.len())].iter().collect(),
        );
        i = j + hashes + 1;
      } else if matches!(word.as_str(), "b" | "c") && matches!(at(i), Some('"' | '\'')) {
        // (its prefix: the literal comes next round)
      } else {
        code.push_str(&word);
      }
    } else {
      code.push(c);
      i += 1;
    }
  }
  // (the literals' text is out of the code, so a brace here is a brace)
  while let Some(start) = code.find("#[cfg(test)]") {
    let tail = &code[start..];
    let ends = match (tail.find('{'), tail.find(';')) {
      (Some(brace), semi) if semi.is_none_or(|semi| brace < semi) => {
        let mut depth = 0usize;
        tail[brace..]
          .char_indices()
          .find_map(|(n, c)| {
            depth += usize::from(c == '{');
            depth -= usize::from(c == '}');
            (depth == 0).then_some(brace + n + 1)
          })
          .unwrap_or(tail.len())
      },
      (_, Some(semi)) => semi + 1,
      _ => tail.len(),
    };
    code.replace_range(start..start + ends, "");
  }
  RustSource { code, literals }
}

fn read_rust(path: &str) -> RustSource {
  lex_rust(&fs::read_to_string(repo_path(path)).unwrap_or_else(|e| panic!("{path}: {e}")))
}

/// Compare the counts with their pins, a debt each: `why` when one rises, a request to lower the pin when one falls. A
/// pin raised shows only in this file's diff, so the rule is in the message: what raises one states its reason, in the
/// commit and in the batch's report, and the reviewer reads it.
fn assert_debts(name: &str, pins: &[(&str, usize)], counts: &BTreeMap<String, usize>, why: &str) {
  let table: String = counts.iter().fold(String::new(), |out, (key, n)| {
    format!("{out}  (\"{key}\", {n}),\n")
  });
  let pinned: BTreeMap<String, usize> = pins.iter().map(|(k, n)| (k.to_string(), *n)).collect();
  let rose: Vec<String> = counts
    .iter()
    .filter(|(key, n)| **n > pinned.get(*key).copied().unwrap_or(0))
    .map(|(key, n)| format!("{key}: {} -> {n}", pinned.get(key).copied().unwrap_or(0)))
    .collect();
  assert!(
    rose.is_empty(),
    "{name} rose: {rose:?}. {why} A raised pin is the exception: the commit and the batch's report state its reason, and \
     the review reads it. The table now:\n{table}"
  );
  assert_eq!(
    pinned, *counts,
    "{name} fell: a debt paid. Lower its pin so it stays paid; the table now:\n{table}"
  );
}

/// What the old reader emits into the API, per file: `\lx@add@`, `\lx@annotate@frontmatter` and `queue_front_matter(`
/// occurrences, and `annotate=` literals. Each is a decision the old engine takes, on the way to deleting it (the first and
/// the last 42 + 4 at int140a; the study's 41 counted otherwise).
/// Stopgaps inside these counts: 97f r2's `\lx@add@contact@placed` in `email_block_calls.rs` (`placed_email_line_calls`,
/// `take_placed_line_emails`; rev126a N2), and its F1 and F3. 108a's six rules (`rows::own_marks_end`,
/// `lines_under_names_read_as_names`, `line_breaks_as_name_separators`, `separators::has_author_block_separator`,
/// `symbols::glued_marks`, `email_lines::address_row_pieces`) added no emission: they feed the existing ones, so only
/// [`OLD_READER_CODE_LINES`] sees that kind of growth.
const OLD_READER_EMISSIONS: &[(&str, usize)] = &[
  ("affiliations.rs lx@add@", 2),
  ("affiliations.rs lx@annotate@frontmatter", 2),
  ("collaborations.rs annotate=", 1),
  ("collaborations.rs lx@add@", 2),
  ("dispatch.rs annotate=", 2),
  ("dispatch.rs lx@add@", 10),
  ("email_block_calls.rs lx@add@", 6),
  ("email_lines.rs lx@add@", 7),
  ("line_kinds.rs annotate=", 1),
  ("line_kinds.rs lx@add@", 4),
  ("name_text.rs lx@add@", 3),
  ("notes.rs lx@add@", 8),
];

/// The old reader's non-blank code lines at int140a: a ceiling, for the growth its emissions don't show (108a's +703
/// lines, no new emission). A ceiling, so no edit that shortens the reader fails on it; **a batch that deletes from the
/// reader lowers it to the new count**, in the same commit, so what was deleted cannot grow back. A sibling directory is no
/// escape from the rest of this fence's reach: its token reads count in `token_reads_only_fall_toward_the_floor`'s
/// catch-all, and its API names and keys in `the_frontmatter_api_stays_perls`.
const OLD_READER_CODE_LINES: usize = 15_404;

/// What the fence counts in the old reader's code: its ways into the API's queue, and a convention's mode.
const OLD_READER_NEEDLES: [&str; 4] = [
  "lx@add@",
  "lx@annotate@frontmatter",
  "queue_front_matter(",
  "annotate=",
];

/// The fence: the old reader takes no new ownership rule.
#[test]
fn the_old_reader_takes_no_new_decision() {
  const WHY: &str = "A new ownership rule lands in `observer/resolver/`, not in the old reader, which is deleted before \
                     the merge: docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md.";
  let dir = format!("{BASE}/author_block");
  let mut counts: BTreeMap<String, usize> = BTreeMap::new();
  let mut lines = 0;
  for file in rust_files(&dir) {
    let short = file.rsplit('/').next().expect("a file name").to_string();
    for code in code_lines(&file) {
      lines += usize::from(!code.trim().is_empty());
      for needle in OLD_READER_NEEDLES {
        let n = code.matches(needle).count();
        if n > 0 {
          *counts.entry(format!("{short} {needle}")).or_default() += n;
        }
      }
    }
  }
  assert_debts(
    "the old reader's emissions",
    OLD_READER_EMISSIONS,
    &counts,
    WHY,
  );
  assert!(
    lines <= OLD_READER_CODE_LINES,
    "the old reader grew to {lines} code lines, over {OLD_READER_CODE_LINES}. {WHY} Raising this ceiling needs the main \
     session's decision."
  );
}

/// The one engine's reach into the old reader, per file and name (user 2026-11-08: "stay focused on having only one engine
/// for frontmatter"). `author_block/mod.rs` re-exports each of its submodules (`pub use affiliations::*`, …) and
/// `base_utilities/mod.rs` re-exports it all (`pub use author_block::*`), so a frontmatter file's `use super::*` reaches every
/// name the old reader exports without spelling `author_block::`: the fence counts the names. A use is a call (`name(`), a
/// path (`::name`, `name::`), a name in a `use` list, or any occurrence of a constant or type name (upper case first). Each
/// pair is a debt: it may only fall. A new one is a decision the old engine takes for the new one; it lands in
/// `observer/resolver/`, on the digested events, instead. Where each name goes when `author_block/` is deleted:
/// docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md, row "`author_block/`'s survivors" (text rules on ink to
/// `frontmatter/classify.rs`, Perl's API-call writers into the emitter, Perl's whole-block fallback to its own module, the
/// decisions translated onto the reading).
const ONE_ENGINE_REACH: &[(&str, usize)] = &[
  ("address_lists.rs is_email_label", 1),
  ("address_lists.rs is_local_char", 1),
  ("address_lists.rs list_owners", 1),
  ("address_lists.rs NameKeys", 3),
  ("address_lists.rs spelled_owner", 1),
  ("insert.rs as_printed", 1),
  ("insert.rs classify_thanks", 1),
  ("insert.rs CUT_ROLES", 5),
  ("insert.rs entry_address_owners", 1),
  ("insert.rs ENTRY_KEY", 7),
  ("insert.rs ENTRY_KINDED", 2),
  ("insert.rs entry_legend_kind", 1),
  ("insert.rs ENTRY_SYMBOL", 2),
  ("insert.rs flag_merged_creators", 1),
  ("insert.rs is_email_label", 4),
  ("insert.rs is_entry_word_mailbox", 1),
  ("insert.rs is_listed_local_part", 1),
  ("insert.rs is_pattern_address", 2),
  ("insert.rs is_role_mailbox", 1),
  ("insert.rs list_owners", 1),
  ("insert.rs lists_local_parts", 2),
  ("insert.rs name_count", 1),
  ("insert.rs NAME_GRADE_SOCIETIES", 1),
  ("insert.rs NameKeys", 11),
  ("insert.rs names_a_contact", 2),
  ("insert.rs opens_a_dedication", 3),
  ("insert.rs own_call_keeps", 1),
  ("insert.rs reads_as_affiliation", 1),
  ("insert.rs SEQUENCE_LABEL", 1),
  ("insert.rs spelled_owner", 9),
  ("insert.rs spelled_unspaced_list", 1),
  ("insert.rs symbol_mark_key", 1),
  ("insert.rs text_prints_name", 1),
  ("observer/resolver/api.rs classify_thanks", 1),
  ("observer/resolver/api.rs is_entry_word_mailbox", 2),
  ("observer/resolver/blocks.rs is_pattern_address", 1),
  ("observer/resolver/labels.rs ADDRESS_GROUP_SEPARATORS", 3),
  ("observer/resolver/labels.rs classify_thanks", 1),
  ("observer/resolver/labels.rs ends_a_listed_run", 2),
  ("observer/resolver/labels.rs has_person_cue", 1),
  ("observer/resolver/labels.rs is_email_label", 4),
  ("observer/resolver/labels.rs is_listed_local_part", 2),
  ("observer/resolver/labels.rs is_local_char", 3),
  ("observer/resolver/labels.rs is_name_qualifier", 3),
  ("observer/resolver/labels.rs names_a_contact", 1),
  ("observer/resolver/labels.rs opens_a_dedication", 1),
  ("observer/resolver/labels.rs opens_with_deceased", 1),
  ("observer/resolver/labels.rs reads_as_affiliation", 1),
  ("observer/resolver/labels.rs UNBRACED_LIST_SEPARATORS", 4),
  ("observer/resolver/labels.rs write_out_unspaced_lists", 2),
  ("observer/resolver/lines.rs collaboration_heads", 1),
  ("observer/resolver/lines.rs is_membership_part", 1),
  ("observer/resolver/lines.rs is_name_qualifier", 2),
  ("observer/resolver/lines.rs name_count", 3),
  ("observer/resolver/lines.rs non_name_word", 1),
  ("observer/resolver/lines.rs reads_as_collaboration", 1),
  (
    "observer/resolver/lines.rs reads_as_listed_collaboration",
    1,
  ),
  ("observer/resolver/mod.rs collaboration_line_framing", 1),
  ("observer/resolver/mod.rs Framing", 1),
  ("observer/resolver/mod.rs lists_local_parts", 1),
  ("observer/resolver/mod.rs NameKeys", 2),
  ("observer/resolver/mod.rs names_correspondence", 5),
  ("observer/resolver/mod.rs spelled_owners", 1),
  ("observer/resolver/net.rs is_name_qualifier", 1),
  ("observer/resolver/placed.rs CUT_LIST", 2),
  ("observer/resolver/post.rs as_printed", 1),
  ("observer/resolver/post.rs entry_address_owners", 2),
  ("observer/resolver/post.rs is_entry_word_mailbox", 1),
  ("observer/resolver/post.rs is_pattern_address", 3),
  ("observer/resolver/post.rs lists_local_parts", 5),
  ("observer/resolver/post.rs NameKeys", 3),
  ("observer/resolver/post.rs own_call_keeps", 1),
  ("observer/resolver/post.rs spelled_owners", 1),
  ("observer/writer/apply.rs classify_thanks", 1),
  ("observer/writer/emit.rs classify_thanks", 1),
  ("observer/writer/emit.rs document_note_call", 2),
  ("observer/writer/emit.rs entry_mark", 1),
  ("observer/writer/emit.rs mark_label", 1),
  ("observer/writer/emit.rs note_body", 1),
  ("observer/writer/emit.rs note_call_end", 2),
  ("observer/writer/emit.rs rewrite_symbol_superscripts", 1),
  ("observer/writer/emit.rs split_wrapped_affiliation_marks", 1),
  ("observer/writer/emit.rs starts_with_affiliation_mark", 1),
  ("observer/writer/emit.rs whole_block_author", 2),
  ("place.rs address_owners", 1),
  ("place.rs author_mark_operands", 2),
  ("place.rs AuthorNote", 1),
  ("place.rs author_notes", 1),
  ("place.rs entry_mark", 2),
  ("place.rs marker_leads", 1),
  ("place.rs mark_label", 2),
  ("place.rs NameKeys", 2),
  ("place.rs names_correspondence", 3),
  ("place.rs queued_name", 2),
  ("place.rs visible_name_text", 3),
  ("queue_digest.rs classify_thanks", 1),
  ("queue_digest.rs is_footnote_symbol_operand", 1),
  ("queue_digest.rs is_symbol_mark", 2),
  ("queue_digest.rs opens_with_deceased", 1),
  ("queue_digest.rs reads_as_affiliation", 1),
  ("queue_digest.rs split_mark_notes", 2),
  ("queue_digest.rs visible_name_text", 1),
  ("queue_reading.rs author_mark_operands", 1),
  ("queue_reading.rs is_mark_command", 2),
  ("queue_reading.rs is_symbol_mark", 2),
  ("queue_reading.rs label_pieces", 1),
  ("queue_reading.rs mark_label", 1),
  ("queue.rs author_mark_position", 1),
  ("queue.rs entry_mark", 1),
  ("queue.rs is_symbol_mark", 1),
  ("relocate.rs author_mark_operands", 1),
  ("relocate.rs classify_thanks", 1),
  ("relocate.rs CUT_LIST", 1),
  ("relocate.rs is_mark_command", 3),
  ("relocate.rs is_ordinal_suffix", 3),
  ("relocate.rs is_spacing", 3),
  ("relocate.rs is_symbol_mark", 1),
  ("relocate.rs leading_spacing_end", 1),
  ("relocate.rs leads_with_mark", 2),
  ("relocate.rs marked_after_name", 1),
  ("relocate.rs marker_leads", 2),
  ("relocate.rs queued_name", 1),
  ("relocate.rs sup_operand_at", 11),
  ("shared_row/emails.rs address_tokens", 1),
  ("shared_row/emails.rs email_addresses", 2),
  ("shared_row/emails.rs email_command_address", 3),
  ("shared_row/emails.rs is_email_command", 2),
  ("shared_row/emails.rs is_email_label", 1),
  ("shared_row/emails.rs lists_local_parts", 3),
  ("shared_row/emails.rs names_a_contact", 1),
  ("shared_row/emails.rs row_address_owners", 1),
  ("shared_row/emails.rs row_address_spelled_owners", 2),
  ("shared_row/emails.rs split_email_label", 2),
  ("shared_row/emails.rs visible_name_text", 1),
  ("shared_row.rs author_mark_operands", 1),
  ("shared_row.rs is_author_mark", 3),
  ("shared_row.rs outside_notes", 1),
  ("shared_row.rs queued_name", 1),
  ("shared_row.rs sup_operand_at", 2),
];

/// The names the old reader exports into `base_utilities`: the `pub`, `pub(crate)` and `pub(in …)` items of `mod.rs` and of
/// each submodule `mod.rs` re-exports with `pub use <submodule>::*`.
fn old_reader_exports() -> std::collections::BTreeSet<String> {
  let dir = format!("{BASE}/author_block");
  let glob = Regex::new(r"pub(?:\([^)]*\))?\s+use\s+([a-z_][a-z0-9_]*)::\*").expect("a regex");
  let item = Regex::new(
    r"(?m)^\s*pub(?:\((?:crate|in [^)]*)\))?\s+(?:fn|const|static|struct|enum|type|trait)\s+([A-Za-z_][A-Za-z0-9_]*)",
  )
  .expect("a regex");
  let mut files = vec![format!("{dir}/mod.rs")];
  for found in glob.captures_iter(&read_rust(&format!("{dir}/mod.rs")).code) {
    let sub = format!("{dir}/{}", &found[1]);
    if repo_path(&format!("{sub}.rs")).is_file() {
      files.push(format!("{sub}.rs"));
    } else {
      files.extend(rust_files(&sub));
    }
  }
  let mut names = std::collections::BTreeSet::new();
  for file in files {
    for found in item.captures_iter(&read_rust(&file).code) {
      names.insert(found[1].to_string());
    }
  }
  names
}

/// How often `code` uses the old reader's `name` (see [`ONE_ENGINE_REACH`] for what counts as a use); `imports` are the byte
/// ranges of `code`'s `use` statements.
fn uses_of(code: &str, imports: &[std::ops::Range<usize>], name: &str) -> usize {
  let word = Regex::new(&format!(r"\b{name}\b")).expect("a regex");
  let constant_or_type = name.chars().next().is_some_and(|c| c.is_ascii_uppercase());
  word
    .find_iter(code)
    .filter(|found| {
      let before = code[..found.start()].trim_end();
      let after = code[found.end()..].trim_start();
      if before.ends_with('.') {
        return false;
      }
      constant_or_type
        || before.ends_with("::")
        || after.starts_with("::")
        || after.starts_with('(')
        || imports.iter().any(|range| range.contains(&found.start()))
    })
    .count()
}

/// The reverse fence: the one engine reaches into the old reader only through the names it reaches today, each a debt.
#[test]
fn the_one_engine_reaches_the_old_reader_only_through_its_survivors() {
  const WHY: &str = "The one engine decides on the digested events (`observer/resolver/`); `author_block/` is deleted after \
                     the flip, and each name `frontmatter/` still uses moves or is translated: \
                     docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md, row \"`author_block/`'s survivors\".";
  let dir = format!("{BASE}/frontmatter");
  let definition =
    Regex::new(r"\b(?:fn|const|static|struct|enum|type|trait|mod)\s+([A-Za-z_][A-Za-z0-9_]*)")
      .expect("a regex");
  let files = rust_files(&dir);
  let sources: Vec<(String, RustSource)> = files
    .iter()
    .map(|file| {
      let short = file
        .strip_prefix(&format!("{dir}/"))
        .unwrap_or(file)
        .to_string();
      (short, read_rust(file))
    })
    .collect();
  let defined_here: std::collections::BTreeSet<String> = sources
    .iter()
    .flat_map(|(_, source)| {
      definition
        .captures_iter(&source.code)
        .map(|found| found[1].to_string())
        .collect::<Vec<_>>()
    })
    .collect();
  let names: Vec<String> = old_reader_exports()
    .into_iter()
    .filter(|name| !defined_here.contains(name))
    .collect();
  let import = Regex::new(r"\b(?:pub(?:\([^)]*\))?\s+)?use\s+[^;]*;").expect("a regex");
  let mut counts: BTreeMap<String, usize> = BTreeMap::new();
  for (short, source) in &sources {
    let imports: Vec<_> = import
      .find_iter(&source.code)
      .map(|found| found.range())
      .collect();
    for name in &names {
      let n = uses_of(&source.code, &imports, name);
      if n > 0 {
        counts.insert(format!("{short} {name}"), n);
      }
    }
  }
  assert_debts(
    "the one engine's reach into the old reader",
    ONE_ENGINE_REACH,
    &counts,
    WHY,
  );
}

/// Perl LaTeXML's Frontmatter API names, generated once from `LaTeXML/lib/LaTeXML` (absent in worktrees and CI, so hard-coded
/// here): `\lx@add@…`, `\lx@annotate@…`, `\lx@clear@…`. Not `\lx@address@…`, `\lx@addtocounter` or `\lx@addframing`,
/// which only share the prefix, nor the bare prefix `\lx@add@`, which Perl builds names from.
const PERL_FRONTMATTER_API: &[&str] = &[
  "lx@add@Preamble@PI",
  "lx@add@abstract",
  "lx@add@address",
  "lx@add@affiliation",
  "lx@add@affiliations",
  "lx@add@altaddress",
  "lx@add@altaffiliation",
  "lx@add@author",
  "lx@add@authors",
  "lx@add@classification",
  "lx@add@contact",
  "lx@add@copyright",
  "lx@add@copyrightholder",
  "lx@add@copyrightyear",
  "lx@add@creator",
  "lx@add@cssclass",
  "lx@add@currentaddress",
  "lx@add@date",
  "lx@add@editor",
  "lx@add@email",
  "lx@add@frontmatter",
  "lx@add@frontmatter@now",
  "lx@add@frontmatter@until",
  "lx@add@keywords",
  "lx@add@note",
  "lx@add@orcid",
  "lx@add@pubnote",
  "lx@add@pubnote@thanks",
  "lx@add@subtitle",
  "lx@add@thanks",
  "lx@add@title",
  "lx@add@toctitle",
  "lx@add@translator",
  "lx@add@url",
  "lx@annotate@frontmatter",
  "lx@annotate@frontmatter@now",
  "lx@clear@creators",
  "lx@clear@frontmatter",
];

/// The API's names outside Perl's: the exceptions, a list that only shrinks. Each retires with the old reader, becomes a
/// key on Perl's own call, or stays with its reason recorded (the classification comes at Z4, Z20-Z22 move the bindings).
const API_EXTENSIONS: &[&str] = &[
  "lx@add@aas@email",
  "lx@add@affiliation@marked",
  "lx@add@affiliation@shared",
  "lx@add@author@keyvals",
  "lx@add@author@split",
  "lx@add@authors@adaptive",
  "lx@add@authors@append",
  "lx@add@authors@list",
  "lx@add@contact@list",
  "lx@add@contact@placed",
  "lx@add@creator@rdf",
  "lx@add@date@halved",
  "lx@add@email@metadata",
  "lx@add@frontmatter@container",
  "lx@add@store@affiliations",
  "lx@add@store@authorblock",
];

/// The keys of `\\lx@add@…[…]` calls Perl has too: its `DefKeyVal('Frontmatter', …)` (`Base_Utility.pool.ltxml:247-254`), what its own
/// calls pass besides (`name=`, `scheme=`, `keys=`: every literal of `LaTeXML/lib/LaTeXML`), and the attributes of the element, as every
/// option reaches `openElement` (`Base_Utility.pool.ltxml:851-852`): `property=` and `content=` on `ltx:rdf` (`lxRDFa.sty.ltxml:190`),
/// `xml:lang=`.
const PERL_DECLARED_KEYS: &[&str] = &[
  "annotate",
  "annotations",
  "class",
  "content",
  "graphic",
  "keys",
  "label",
  "labelref",
  "labelseq",
  "name",
  "property",
  "role",
  "scheme",
  "xml:lang",
];

/// The keys Perl's own code sets on a queued entry's options behind the call (`Base_Utility.pool.ltxml`): what a Rust setter may
/// insert without a row in [`API_KEYS_OUTSIDE_PERLS`].
const PERL_INTERNAL_KEYS: &[&str] = &[
  "_annotation",
  "_annotations",
  "_force_font",
  "_label",
  "_num",
];

/// The API's keys outside Perl's, per key and file: each `_word=` a literal holds (and `owner=`, the one without the `_`), each
/// string constant naming such a key (`CUT_LIST`, which `s!("{CUT_LIST}={key}")` sets: `const _cutlist`), and each key declared in
/// `Frontmatter` that Perl does not declare; since Z2 also every other key of a call's literal options (`[mark]`) and each key a
/// Rust setter inserts into an entry's options outside Perl's own (`insert`, `entry`: pinned with their shape, so the few XML
/// attributes that share it, `_noautoclose`, `_qname`, stay in the count whole). Z2r2 parses them ([`read_api_speech`]), which
/// found what a call passes through its tags (`\lx@add@frontmatter{ltx:rdf}[property=…]`, `[accumulate=true]`, `[xml:lang=#1]`,
/// the rows the old pattern missed): 74 rows, 95 uses became 82, 105, no code changed. A debt like the names. Not a constructor's
/// XML attribute in a template, which is quoted (`_noautoclose='true'`), unless a call's literal options hold it, nor a constant
/// in `latex_constructs/` or `tex_box.rs` (the kernel's own attributes). `_scopebegin=` is pgfsys's own keyval: it shares the
/// shape, and stays pinned so that the count stays whole; so do `_font`, `_xmkey`, `_split_ref` and `_nowrap`, attributes a setter
/// inserts, and the constants naming a DOM mark (`_deposit`, `_inserted`, `_titlepage`, `_preamble`, `_paragraph_break`): this reads
/// the shape, and telling them apart would be a list of exceptions in the reader, not a shorter one here.
const API_KEYS_OUTSIDE_PERLS: &[(&str, usize)] = &[
  ("[accumulate] package/acmart_cls.rs", 2),
  ("[accumulate] package/ieeetran_cls.rs", 1),
  ("[accumulate] package/ims_support_sty.rs", 1),
  ("[mark] lmcs_cls.rs", 1),
  ("[mark] package/iopart_support_sty.rs", 1),
  ("_answers= base_utilities/defs/frontmatter_fields.rs", 1),
  ("_group= base_utilities/defs/frontmatter_fields.rs", 1),
  ("_grouped= package/revtex4_support_sty.rs", 1),
  ("_labelprints= cas_dc_cls.rs", 1),
  ("_labelprints= package/authblk_sty.rs", 1),
  ("_labelprints= package/jheppub_sty.rs", 1),
  ("_mark= package/jheppub_sty.rs", 1),
  ("_markpiece= package/llncs_cls.rs", 1),
  ("_marks= base_utilities/defs/frontmatter_fields.rs", 1),
  ("_markshown= base_utilities/author_block/affiliations.rs", 1),
  // (Z4b: `flag_merged_creators`' own mark of a creator it checked, on a block written whole, Perl's fallback, whose merge its Warning
  // counts: a tabular block is kept whole on purpose alike, OD #980; Z4br2: `author_macros::KEPT_WHOLE`, the `=2` queued entry's and the
  // emitter's)
  ("_merged_checked= base_utilities/defs/author_macros.rs", 1),
  ("_net= frontmatter_stores.rs", 1),
  ("_scopebegin= package/pgfsys_latexml_def.rs", 1),
  ("_sequence= base_utilities/defs/frontmatter_fields.rs", 1),
  ("_share= cas_dc_cls.rs", 1),
  ("_share= package/aa_cls.rs", 1),
  ("_share= package/llncs_cls.rs", 2),
  ("_store= frontmatter_stores.rs", 11),
  ("_unmarked= base_utilities/defs/frontmatter_fields.rs", 1),
  ("_unsplit= frontmatter_stores.rs", 1),
  ("_unsplit= package/revtex4_support_sty.rs", 1),
  ("_withsup= base_utilities/frontmatter/observer/ledger.rs", 2),
  ("_zone= package/aa_support_sty.rs", 2),
  ("_zone= package/elsarticle_cls.rs", 2),
  ("_zone= package/revtex4_support_sty.rs", 2),
  (
    "const _command_form base_utilities/frontmatter/queue_digest.rs",
    1,
  ),
  (
    "const _cutlist base_utilities/author_block/affiliations.rs",
    1,
  ),
  (
    "const _deposit base_utilities/frontmatter/cover_order.rs",
    1,
  ),
  (
    "const _entry base_utilities/author_block/affiliations.rs",
    1,
  ),
  (
    "const _entrysymbol base_utilities/author_block/affiliations.rs",
    1,
  ),
  (
    "const _inserted base_utilities/frontmatter/cover_order.rs",
    1,
  ),
  (
    "const _kinded base_utilities/author_block/affiliations.rs",
    1,
  ),
  // (OD #961's key, with the API that declares it since P2r2, which `author_block/`, `frontmatter/` and the emitter import)
  (
    "const _labelprints base_utilities/defs/frontmatter_api.rs",
    1,
  ),
  ("const _late base_utilities/frontmatter/cover_order.rs", 1),
  (
    "const _late_mark base_utilities/frontmatter/cover_order.rs",
    1,
  ),
  ("const _latecall base_utilities/frontmatter/insert.rs", 1),
  ("const _linefirst base_utilities/frontmatter/insert.rs", 1),
  ("const _linelist base_utilities/frontmatter/insert.rs", 1),
  ("const _linemark base_utilities/frontmatter/insert.rs", 1),
  (
    "const _linemarkshown base_utilities/frontmatter/insert.rs",
    1,
  ),
  ("const _linklabel base_utilities/frontmatter/insert.rs", 1),
  // (LO2, 0 -> 1: an emitted entry's first mark read, a fact kept on its pending entry as Perl keeps its labels there
  // (Base_Utility.pool.ltxml:502-508), where LO1 kept a State flag; internal, never written out)
  ("const _markread base_utilities/defs/author_macros.rs", 1),
  (
    "const _paragraph_break base_utilities/frontmatter/queue.rs",
    1,
  ),
  ("const _preamble base_utilities/frontmatter/place.rs", 1),
  ("const _queued base_utilities/frontmatter/relocate.rs", 1),
  (
    "const _released_store base_utilities/frontmatter/queue_digest.rs",
    1,
  ),
  (
    "const _seqlabel base_utilities/author_block/affiliations.rs",
    1,
  ),
  // (P2r2: the label that prints, of an entry the emitter wrote — `_labelprints` held it beside its flag `true`, a flag and
  // data in one key; an entry's fact in its own key, as Perl keeps `_label` and `_annotations`; OD #980)
  (
    "const _printedlabel base_utilities/defs/frontmatter_api.rs",
    1,
  ),
  ("const _source base_utilities/frontmatter/insert.rs", 1),
  (
    "const _titlepage base_utilities/frontmatter/cover_order.rs",
    1,
  ),
  (
    "declared _labelprints base_utilities/defs/frontmatter_api.rs",
    1,
  ),
  ("declared _net base_utilities/defs/frontmatter_api.rs", 1),
  (
    "declared _printedlabel base_utilities/defs/frontmatter_api.rs",
    1,
  ),
  (
    "declared accumulate base_utilities/defs/frontmatter_api.rs",
    1,
  ),
  (
    "declared command_form base_utilities/defs/frontmatter_api.rs",
    1,
  ),
  ("insert _bymark base_utilities/defs/author_macros.rs", 1),
  ("insert _font base_utilities/frontmatter/insert.rs", 2),
  ("insert _for base_utilities/defs/frontmatter_api.rs", 1),
  (
    "insert _mergednames base_utilities/defs/author_macros.rs",
    1,
  ),
  ("insert _noautoclose algpseudocodex_sty.rs", 1),
  ("insert _nowrap package/latexml_sty/declare.rs", 1),
  (
    "insert _printedbefore base_utilities/defs/author_macros.rs",
    1,
  ),
  ("insert _qname package/axodraw_sty.rs", 1),
  (
    "insert _refmarkshown base_utilities/defs/author_macros.rs",
    1,
  ),
  ("insert _scopebegin package/pgfsys_latexml_def.rs", 3),
  ("insert _split_ref package/amsmath_sty.rs", 1),
  ("insert _textmarks base_utilities/frontmatter/insert.rs", 1),
  ("insert _xmkey package/amsmath_sty.rs", 1),
  ("owner= base_utilities/defs/author_macros.rs", 1),
  ("owner= base_utilities/defs/frontmatter_fields.rs", 4),
  ("owner= lmcs_cls.rs", 1),
  ("owner= package/aas_support_sty.rs", 1),
  ("owner= package/iopart_support_sty.rs", 1),
  ("owner= package/jheppub_sty.rs", 1),
];

/// The crates that may speak the API.
const API_SPEAKERS: [&str; 3] = [
  "latexml_engine/src",
  "latexml_package/src",
  "latexml_contrib/src",
];

/// The keys of `options`, what a call's options hold from its `[` on: to the `]` at its brace depth (`[mark={a]b},foo=1]`), or to
/// the literal's end, an open one, its value made by the code after it (`\lx@add@contact[role=note,mark=`). A key with no value is
/// one (`[role=note,flag]`); a parameter (`#1`) or a format's placeholder standing for keys names none.
fn option_keys(options: &str) -> Vec<String> {
  let (mut keys, mut depth, mut piece) = (Vec::new(), 0usize, String::new());
  let mut take = |piece: &mut String| {
    let key = piece.split('=').next().unwrap_or("").trim().to_string();
    if !key.is_empty() && !key.starts_with(['#', '{']) {
      keys.push(key);
    }
    piece.clear();
  };
  for c in options.chars() {
    match c {
      ']' if depth == 0 => break,
      ',' if depth == 0 => {
        take(&mut piece);
        continue;
      },
      '{' => depth += 1,
      '}' => depth = depth.saturating_sub(1),
      _ => {},
    }
    piece.push(c);
  }
  take(&mut piece);
  keys
}

/// Where a call's options start in `after`, what follows its name, if it has any: right after it, or, for the commands whose first
/// arguments are the tags (`\lx@add@frontmatter{ltx:creator}[…]`, `\lx@annotate@frontmatter{a}{b}[…]`), after those.
fn options_start(name: &str, after: &str) -> Option<usize> {
  let mut at = 0;
  if name.contains("frontmatter") {
    while after[at..].trim_start().starts_with('{') {
      at = after.len() - after[at..].trim_start().len();
      let mut depth = 0usize;
      at += after[at..].char_indices().find_map(|(n, c)| {
        depth += usize::from(c == '{');
        depth = depth.saturating_sub(usize::from(c == '}'));
        (depth == 0).then_some(n + 1)
      })?;
    }
  }
  let trimmed = after[at..].trim_start();
  trimmed
    .starts_with('[')
    .then(|| after.len() - trimmed.len() + 1)
}

/// What one crate source says of the API: the names and the keys outside Perl's.
#[derive(Default)]
struct ApiSpeech {
  names: Vec<String>,
  keys:  BTreeMap<String, usize>,
}

/// Read one source's literals and code for what they say of the API, `short` naming it in a key's row: a call's name and each of
/// its literal options, a `_word=` wherever a literal holds it, a literal of keyvals handed to a call by the code (`role=…`), a
/// setter (`insert(⏎"_foo"`, `entry("_foo")`), a constant, a declaration.
fn read_api_speech(source: &RustSource, short: &str, speech: &mut ApiSpeech) {
  let call = Regex::new(r"\\(lx@(?:add|annotate|clear)@[A-Za-z@]*)").expect("the regex");
  // (each key's own: what follows the `=` is read apart, so `_a=,_b=1`'s second is seen)
  let word = Regex::new(r"(?:^|[^A-Za-z0-9_])(_[a-z][a-z0-9_]*|owner)=").expect("the regex");
  let keyvals =
    Regex::new(r"^(?:annotate|annotations|class|label|labelseq|role)=").expect("the regex");
  let is_key = Regex::new(r"^_[a-z][a-z0-9_]*$").expect("the regex");
  let setter = Regex::new(r#"\b(insert|entry)\(\s*"([0-9]+)""#).expect("the regex");
  let constant =
    Regex::new(r#"\bconst [A-Z][A-Z_0-9]*: &(?:'static )?str = "([0-9]+)""#).expect("the regex");
  let declared = Regex::new(r#"DefKeyVal!\(\s*"([0-9]+)",\s*"([0-9]+)""#).expect("the regex");
  // (a `#[cfg(test)]` module's literals are not the code's)
  let used = Regex::new(r#""([0-9]+)""#).expect("the regex");
  let literal_of =
    |capture: &str| source.literals[capture.parse::<usize>().expect("an index")].as_str();
  for found in used.captures_iter(&source.code) {
    let literal = literal_of(&found[1]);
    let mut options: Vec<&str> = Vec::new();
    for name in call.captures_iter(literal) {
      let whole = name.get(0).expect("the match");
      let found = &name[1];
      // (a bare prefix, as a call test reads it, is no name)
      if !found.ends_with('@') && !PERL_FRONTMATTER_API.contains(&found) {
        speech.names.push(found.to_string());
      }
      let after = &literal[whole.end()..];
      if let Some(start) = options_start(found, after) {
        options.push(&after[start..]);
      }
    }
    if keyvals.is_match(literal) {
      options.push(literal);
    }
    for found in word.captures_iter(literal) {
      let end = found.get(0).expect("the match").end();
      if !matches!(literal[end..].chars().next(), Some('\'' | '=')) {
        *speech
          .keys
          .entry(format!("{}= {short}", &found[1]))
          .or_default() += 1;
      }
    }
    for option in options.into_iter().flat_map(option_keys) {
      if !PERL_DECLARED_KEYS.contains(&option.as_str()) && !seen_as_word_key(&option, literal) {
        *speech
          .keys
          .entry(format!("[{option}] {short}"))
          .or_default() += 1;
      }
    }
  }
  for found in setter.captures_iter(&source.code) {
    let key = literal_of(&found[2]);
    if is_key.is_match(key) && !PERL_INTERNAL_KEYS.contains(&key) {
      *speech
        .keys
        .entry(format!("{} {key} {short}", &found[1]))
        .or_default() += 1;
    }
  }
  if !short.starts_with("latex_constructs/") && short != "tex_box.rs" {
    for found in constant.captures_iter(&source.code) {
      let key = literal_of(&found[1]);
      if is_key.is_match(key) {
        *speech
          .keys
          .entry(format!("const {key} {short}"))
          .or_default() += 1;
      }
    }
  }
  for found in declared.captures_iter(&source.code) {
    let key = literal_of(&found[2]);
    if literal_of(&found[1]) == "Frontmatter" && !PERL_DECLARED_KEYS.contains(&key) {
      *speech
        .keys
        .entry(format!("declared {key} {short}"))
        .or_default() += 1;
    }
  }
}

/// Whether the `_word=` count sees this literal's key already: only a key it takes, which a quote or another `=` does not end.
fn seen_as_word_key(key: &str, literal: &str) -> bool {
  (key.starts_with('_') || key == "owner")
    && literal.contains(&format!("{key}="))
    && literal
      .split(&format!("{key}="))
      .skip(1)
      .all(|after| !matches!(after.chars().next(), Some('\'' | '=')))
}

/// The boundary is Perl's Frontmatter API: a name or a key outside Perl's is the exception, and the list only shrinks. A
/// name built at run time (`\lx@add@{role}`) is not seen: this reads the literals. Parsed, not pattern-matched
/// ([`lex_rust`], [`option_keys`]): the shapes an earlier count missed are `every_shape_of_a_key_is_seen`'s.
#[test]
fn the_frontmatter_api_stays_perls() {
  const WHY: &str = "The engine relies on Perl LaTeXML's Frontmatter API and its macro dialect, and an extension is the \
                     exception, stated with its reason: docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md, \"The boundary is \
                     Perl's Frontmatter API\". Answer with Perl's macros and keys (`\\lx@add@creator`, `\\lx@add@contact`, \
                     `label=`, `annotations=`).";
  let mut speech = ApiSpeech::default();
  for file in API_SPEAKERS
    .iter()
    .flat_map(|crate_src| rust_files(crate_src))
  {
    let short = file
      .split_once("/src/")
      .map_or(file.as_str(), |(_, inner)| inner);
    read_api_speech(&read_rust(&file), short, &mut speech);
  }
  let ApiSpeech { names: mut extensions, keys } = speech;
  extensions.sort();
  extensions.dedup();
  let new: Vec<&String> = extensions
    .iter()
    .filter(|name| !API_EXTENSIONS.contains(&name.as_str()))
    .collect();
  assert!(
    new.is_empty(),
    "a Frontmatter API name outside Perl's: {new:?}. {WHY}"
  );
  assert_eq!(
    extensions, API_EXTENSIONS,
    "an extension is gone from the code: take it from `API_EXTENSIONS` too, so the list only shrinks"
  );
  assert_debts(
    "the API's keys outside Perl's",
    API_KEYS_OUTSIDE_PERLS,
    &keys,
    WHY,
  );
}

/// Every shape of a key revZ2 found the count blind to, read as the API guard reads a file: an open literal up to its end, a
/// valueless key, a bracket inside a brace, a call's options past its tags, a raw literal, a line joined by `\`, rustfmt's setter,
/// `entry`; and what is no key: a parameter, a comment, a test module. Nothing of a source's real text decides it.
#[test]
fn every_shape_of_a_key_is_seen() {
  let keys_of = |code: &str| {
    let mut speech = ApiSpeech::default();
    read_api_speech(&lex_rust(code), "f.rs", &mut speech);
    speech.keys.into_keys().collect::<Vec<_>>().join(" ")
  };
  for (code, seen) in [
    (
      r#"TokenizeInternal!("\\lx@add@contact[role=note,mark=")"#,
      "[mark] f.rs",
    ),
    (r#"T("\\lx@add@contact[role=note,flag]{x}")"#, "[flag] f.rs"),
    (
      r#"T("\\lx@add@contact[mark={a]b},foo=1]")"#,
      "[foo] f.rs [mark] f.rs",
    ),
    (
      r#"T("\\lx@add@frontmatter{ltx:creator}[role=author,foo=1]{x}")"#,
      "[foo] f.rs",
    ),
    (
      r#"T("\\lx@annotate@frontmatter{a}{b}[foo=1]{x}")"#,
      "[foo] f.rs",
    ),
    (r##"T(r#"\lx@add@contact[foo=1]"#)"##, "[foo] f.rs"),
    (
      "T(\"\\\\lx@add@contact[role=note,\\\n    foo=1]\")",
      "[foo] f.rs",
    ),
    (r#"T("\\lx@add@contact[_foo='1']")"#, "[_foo] f.rs"),
    (r#"T("\\lx@add@contact[_a=,_b=1]")"#, "_a= f.rs _b= f.rs"),
    (r#"T("label={x},foo=1")"#, "[foo] f.rs"),
    (
      "attr.insert(\n  \"_foo\".to_string(), v)",
      "insert _foo f.rs",
    ),
    (r#"attr.entry("_foo").or_default()"#, "entry _foo f.rs"),
    (r#"const FOO: &'static str = "_foo";"#, "const _foo f.rs"),
    (
      r#"DefKeyVal!("Frontmatter", "foo", "")"#,
      "declared foo f.rs",
    ),
    (r#"T("\\lx@add@contact[role=note,#1]")"#, ""),
    (r#"T("\\lx@add@contact[role={x}]")"#, ""),
    ("// T(\"\\\\lx@add@contact[foo=1]\")", ""),
    ("/* T(\"\\\\lx@add@contact[foo=1]\") */", ""),
    (
      "#[cfg(test)]\nmod tests { const F: &str = \"\\\\lx@add@contact[foo=1]\"; }",
      "",
    ),
    (r#"attr.insert("_label".to_string(), v)"#, ""),
  ] {
    assert_eq!(keys_of(code), seen, "{code}");
  }
  let mut names = ApiSpeech::default();
  read_api_speech(
    &lex_rust(r#"T("\\lx@add@foo\\lx@add@")"#),
    "f.rs",
    &mut names,
  );
  assert_eq!(names.names, ["lx@add@foo"]);
}

/// Token-spelling reads per area, and in the listed bindings: `bin/tokreads.py`'s count of the removal study plus the shapes
/// it missed, **an over-approximation** (a line of code matching a read's shape, whatever it decides; a string holding `//`
/// is cut there). Debts on the way to the floor: about 60 in the engine (the rehearsal's table, the ledger's stamping, the
/// Perl API boundary), `resolver/` 0, decision code elsewhere 0. Raised at Z2, by the shapes revz01 found the pins blind to and
/// no new read: a name's prefix or suffix, a `TOKEN_…` constant compared, a table of names or tokens (once each), a method on a
/// `to_string()`. The engine's total was 2,264 at Z01: the study's 1,962, plus
/// `title_page_check.rs`, which its list left out (14), plus the helpers' shapes the study did not count (`with_str(`,
/// `cs_in(`, `text_tokens(`, `to_string() ==`, a comparison or a match arm on a quoted `\\…`); and 85 in the rest of
/// `base_utilities/`. `frontmatter (rest)` holds any new file under `frontmatter/`, and a `rest <file>` row each file under
/// `base_utilities/` no area lists, a new one or a sibling directory's too, so a read moved there is no debt paid.
/// Bindings are the study's list, so a binding outside it is not watched.
const TOKEN_READS: &[(&str, usize)] = &[
  ("aa_cls.rs", 5),
  ("aas_support_sty.rs", 17),
  ("acmart_cls.rs", 20),
  ("address_lists", 0),
  ("aistats_sty.rs", 53),
  ("ams_support_sty.rs", 16),
  ("apa7_cls.rs", 7),
  ("authblk_sty.rs", 23),
  // (F1r2, 1288 -> 1285: the membership readout finds its commas by `Token::group_step`, no catcode match)
  ("author_block", 1285),
  ("defs/author_macros", 20),
  // (Z4a, 7 -> 8: `\lx@frontmatter@itemnone` reads its tag argument, an element's name: an `ltx:rdf` passes through, the rest goes)
  ("defs/frontmatter_api", 8),
  ("defs/frontmatter_fields", 21),
  ("defs/general", 8),
  ("epl_cls.rs", 43),
  ("eptcs_cls.rs", 28),
  // (raised by one at Z14: the lexer's own constructor, `\lx@fm@obs@affil@marked`, as the row's others; nothing a document spells;
  // by one at N2r2: `raw_lines`' break arm holding the `\and` family too, two lines, the whatsits' definitions, 2106.06418)
  ("events", 60),
  // (raised from none: moved out of core `token.rs`, unwatched there, for the hot path; a rehearsal's own reads)
  // (Z4a, 4 -> 8: an environment step by its meaning, a CS whose definition is `\begin` or `\end`, a `\let` copy too, what no frame
  // says for a raw environment; the definition's name, not the token's spelling)
  // (LO1: a creator call's `annotations` read as its labels, `FieldCreator::labels` — a keyval's value, argument passing; LO2,
  // 9 -> 8: read by the API's own label cleaning, `clean_frontmatter_labels`, Perl's `cleanFrontmatterLabels`)
  ("field_notes", 8),
  // (26: C1's `queue_reading.rs` keeps a cache of what `author_block`'s readers read off the queue's tokens, and goes with them)
  ("frontmatter (rest)", 26),
  ("frontmatter_stores", 527),
  ("gretsi_cls.rs", 14),
  ("ieeetran_cls.rs", 119),
  ("insert", 14),
  // (from `frontmatter (rest)`, 24 -> 23, at Z2r2: `\alignauthor`, out of the kernel's conventions into the binding's own table; no read)
  ("inst_support_sty.rs", 1),
  ("interspeech_cls.rs", 29),
  ("l_aa_cls.rs", 13),
  ("ledger", 24),
  ("llncs_cls.rs", 32),
  ("osa_journal_sty.rs", 77),
  ("place", 6),
  ("ptephy_cls.rs", 115),
  ("queue", 1),
  ("queue_digest", 16),
  // (raised by one at Z14: the rehearsal's `BOUND_IN_A_BLOCK` table, `\lx@add@affiliation@marked`, `\lx@add@creator` read as digested instead of cut by spelling at
  // the frozen API: the kind of table `FRONTMATTER_SINGLE_ENGINE_PLAN.md` "What already exists" keeps, as Perl's `@authorsplits`)
  ("rehearsal", 6),
  ("relocate", 134),
  // (Z2r2: not the study's lines but what `resolver_reads` counts, a shape no reason to raise it: `text_tokens`, printed text read
  // as tokens by `is_institution` and `NameKeys` (5 with their definitions), a `'\\'` read of a label (2), a call's hook (1).
  // Z14r2: 18, not 9 — the count now sees `is_institution(`, review revZ14: Z14's fall from 15 was the seven `text_tokens` sites
  // it moved behind that name; counted again, with the name's definition and Z14's one new call (`carried`); none of Z14r2's)
  ("resolver", 18),
  ("rest /author_split_tests.rs", 24),
  ("rest /block_insertion.rs", 0),
  ("rest /blocks.rs", 2),
  ("rest /boxes_lists.rs", 0),
  ("rest /defs/mod.rs", 0),
  ("rest /defs/tags_refnums.rs", 0),
  ("rest /fonts_defs.rs", 12),
  ("rest /math_cleanup.rs", 0),
  ("rest /misc.rs", 14),
  ("rest /mod.rs", 0),
  ("rest /tokens.rs", 26),
  ("rest /util_text.rs", 25),
  ("revtex4_support_sty.rs", 12),
  ("shared_row", 13),
  ("shared_row/emails", 65),
  ("sv_support_sty.rs", 35),
  ("webofc_cls.rs", 0),
  ("writer/apply", 7),
  // (a new file, what is left of `writer/plan`'s walk-backs: brace balance and a formula's shifts; the two rows together fell by 25, Z3)
  ("writer/content", 7),
  // (Z4a, a new file: the emitter builds Perl's calls from the reading, its reads a call's own keys and pieces, `text_tokens` and the
  // catcodes of a cut: where a slice's edge falls past an option, a group or an accent's letter, OD #980; Z4ar2, 30 -> 28: an address
  // list's `@` no longer read, its inks are)
  ("writer/emit", 28),
  ("writer/dom+mod", 0),
  ("writer/plan", 85),
  // (LO2r2, 0 -> 4: a row line's cut ends at its address's own first token, which the letters check finds; the tokens between the
  // last letter it keeps and that one print no letter, so their kinds say which it keeps — the spacing, separators, commands and
  // bracket opening the address out, a bracket the cut leaves open closed by its own token: `prefix_tokens`, review revLO2 2)
  ("writer/repart", 4),
];

/// The areas under `frontmatter/`, by path; the first that holds a file takes it.
const FRONTMATTER_AREAS: &[(&str, &str)] = &[
  ("shared_row/emails", "shared_row/emails.rs"),
  ("shared_row", "shared_row.rs"),
  ("relocate", "relocate.rs"),
  ("place", "place.rs"),
  ("queue_digest", "queue_digest.rs"),
  ("queue", "queue.rs"),
  ("insert", "insert.rs"),
  ("address_lists", "address_lists.rs"),
  ("writer/plan", "observer/writer/plan.rs"),
  ("writer/apply", "observer/writer/apply.rs"),
  ("writer/content", "observer/writer/content.rs"),
  ("writer/emit", "observer/writer/emit.rs"),
  ("writer/repart", "observer/writer/repart.rs"),
  ("writer/dom+mod", "observer/writer/"),
  ("resolver", "observer/resolver/"),
  ("events", "observer/events/"),
  ("field_notes", "observer/field_notes.rs"),
  ("rehearsal", "observer/rehearsal.rs"),
  ("ledger", "observer/ledger.rs"),
];

/// The count's areas outside `frontmatter/`, and the bindings.
const OTHER_AREAS: &[(&str, &str)] = &[
  (
    "author_block",
    "latexml_engine/src/base_utilities/author_block",
  ),
  (
    "defs/frontmatter_api",
    "latexml_engine/src/base_utilities/defs/frontmatter_api.rs",
  ),
  (
    "defs/frontmatter_fields",
    "latexml_engine/src/base_utilities/defs/frontmatter_fields.rs",
  ),
  (
    "defs/author_macros",
    "latexml_engine/src/base_utilities/defs/author_macros.rs",
  ),
  (
    "defs/general",
    "latexml_engine/src/base_utilities/defs/general.rs",
  ),
  (
    "frontmatter_stores",
    "latexml_engine/src/frontmatter_stores.rs",
  ),
  ("llncs_cls.rs", "latexml_package/src/package/llncs_cls.rs"),
  ("aa_cls.rs", "latexml_package/src/package/aa_cls.rs"),
  (
    "sv_support_sty.rs",
    "latexml_package/src/package/sv_support_sty.rs",
  ),
  (
    "inst_support_sty.rs",
    "latexml_package/src/package/inst_support_sty.rs",
  ),
  ("webofc_cls.rs", "latexml_contrib/src/webofc_cls.rs"),
  (
    "revtex4_support_sty.rs",
    "latexml_package/src/package/revtex4_support_sty.rs",
  ),
  (
    "aas_support_sty.rs",
    "latexml_package/src/package/aas_support_sty.rs",
  ),
  (
    "ieeetran_cls.rs",
    "latexml_package/src/package/ieeetran_cls.rs",
  ),
  (
    "authblk_sty.rs",
    "latexml_package/src/package/authblk_sty.rs",
  ),
  ("acmart_cls.rs", "latexml_package/src/package/acmart_cls.rs"),
  (
    "ams_support_sty.rs",
    "latexml_package/src/package/ams_support_sty.rs",
  ),
  ("ptephy_cls.rs", "latexml_contrib/src/ptephy_cls.rs"),
  (
    "osa_journal_sty.rs",
    "latexml_contrib/src/osa_journal_sty.rs",
  ),
  ("epl_cls.rs", "latexml_contrib/src/epl_cls.rs"),
  ("eptcs_cls.rs", "latexml_contrib/src/eptcs_cls.rs"),
  ("aistats_sty.rs", "latexml_contrib/src/aistats_sty.rs"),
  (
    "interspeech_cls.rs",
    "latexml_contrib/src/interspeech_cls.rs",
  ),
  ("gretsi_cls.rs", "latexml_contrib/src/gretsi_cls.rs"),
  ("l_aa_cls.rs", "latexml_contrib/src/l_aa_cls.rs"),
  ("apa7_cls.rs", "latexml_contrib/src/apa7_cls.rs"),
];

/// How `resolver/`'s code may read a token's spelling, counted whatever its shape, so that a new spelling rule there goes red: every
/// literal holding a backslash (`name == r"\foo"`, `.contains("\\foo")`, `.eq`, `matches!`, a `const`, an array, a `'\\'`), every
/// token made or named (`T_CS!`, any `T_…!`, `TOKEN_…`, the types, a block's `.tokens`), every method on a `to_string()` (`.chars()`,
/// `.to_lowercase()`: its text is a token list's or a value's, and which is the reader's to say) and every comparison on one, a call's
/// `.content` (`ApiCall::content`, its tokens, reachable from here whatever is done with them), and the engine's own patterns on the
/// code. Not the study's lines but what it holds, each once: the floor is 0, so a shape is no reason to raise it.
fn resolver_reads(source: &RustSource) -> usize {
  let shapes = Regex::new(concat!(
    r"\bT_[A-Z]+!|\bTOKEN_[A-Z_]+|\bTokens?\b|\.tokens\b|\.to_string\(\)\s*\.\s*[a-z_]",
    r"|\.content\b|to_string\(\)\s*(?:==|!=)",
    r"|\.get_cs\(\)|\.cs_name\(\)|\.with_cs_name\(|\.get_string\(\)|\.get_catcode\(\)|\bCatcode::|\bToString\b",
    r"|\bposition_of\(|\bis_cs\(|\.as_cs\(|\bmeaning\(|\bwith_str\(|\bcs_in\(|\btext_tokens\(",
    // (Z14r2: `is_institution`, `text_tokens` behind a name, review revZ14 — Z14's six sites moved behind it read as fewer)
    r"|\bis_institution\(",
  ))
  .expect("the regex");
  backslash_literals(source) + shapes.find_iter(&source.code).count()
}

/// The literals of a source holding a backslash, whatever their shape (a call's argument, a `const`, a table, a `'\\'`, raw or escaped).
fn backslash_literals(source: &RustSource) -> usize {
  let held = Regex::new(r#""([0-9]+)""#).expect("the regex");
  held
    .captures_iter(&source.code)
    .filter(|found| source.literals[found[1].parse::<usize>().expect("an index")].contains('\\'))
    .count()
}

/// Every shape revZ2 found the study's count blind to goes red in `resolver/`, whatever its spelling, and what reads events does not.
#[test]
fn a_spelling_rule_in_the_resolver_counts_however_it_is_written() {
  for rule in [
    r##"if name == r"\foo" {}"##,
    r##"if name == r#"\foo"# {}"##,
    r#"if name.contains("\\foo") {}"#,
    r#"if name.eq("\\foo") {}"#,
    r#"if matches!(x, "\\foo") {}"#,
    r#"const FOO: &str = "\\foo";"#,
    r#"let foo = T_CS!("foo");"#,
    r#"const NAMES: [&str; 2] = ["foo", "\\bar"];"#,
    r#"if x.to_string().chars().next() == None {}"#,
    r#"if x.to_string().to_lowercase() == "foo" {}"#,
    "if x\n  .to_string()\n  .to_lowercase() == \"foo\" {}",
    r"if x == '\\' {}",
    r#"if x == "\x5cfoo" {}"#,
    r#"if x == concat!("\\", "foo") {}"#,
    r"fn f(t: &Token) {}",
    r"let t = block.tokens.len();",
    r#"t.with_cs_name(|n| n == "foo");"#,
    r"if t == *TOKEN_RELAX {}",
    r"if t.get_catcode() == Catcode::Other {}",
    r#"if call.content.to_string() == "foo" {}"#,
    r#"if call.content.iter().any(|t| t.to_string() == "foo") {}"#,
    r#"let s = call.content.to_string(); if s.contains("foo") {}"#,
    r"for t in call.content.unlist() {}",
    r#"if x.to_string() == "foo" {}"#,
    r#"if x.to_string() != "foo" {}"#,
  ] {
    assert!(resolver_reads(&lex_rust(rule)) > 0, "unseen: {rule}");
  }
  for clean in [
    r#"if event.kind == EventKind::Name && event.text.contains("and") {}"#,
    r#"let line = format!("a\nb");"#,
    r"let c = 'x';",
    r"fn f<'a>(x: &'a str) -> &'a str { x }",
    "// if name == \"\\\\foo\" {}",
    "#[cfg(test)]\nmod tests { const A: &str = \"\\\\foo\"; }",
  ] {
    assert_eq!(resolver_reads(&lex_rust(clean)), 0, "{clean}");
  }
}

/// Backslash literals under `observer/`, per file or directory: the part of it that may spell a token's name, whatever the code around
/// them does (the regexes of [`token_reads_only_fall_toward_the_floor`] see a comparison, not `fn spelled(x) { x.contains("\\foo") }`).
/// Debts of what names the kernel's tokens today (the rehearsal's table, the writer's and the events' readers, the ledger's stamping);
/// the zeros stay zero: `policy`, `marks` and `mod` decide, and a decision spells no token; `field_notes` names two kernel definitions
/// (`\begin`, `\end`), not a token's spelling (Z4a, ruled at Z4ar2).
const OBSERVER_BACKSLASH_LITERALS: &[(&str, usize)] = &[
  // (Z14: 84 and 109, the lexer's own constructor and the rehearsal's table rows for `\lx@add@affiliation@marked`, `\lx@add@creator` and `\alignauthor`, as in `TOKEN_READS`;
  // Z4a +2: a note's kind, `\lx@fm@obs@fnnote`, the rehearsal's own name for a footnote it heard; N2r2 +3: `raw_lines` reads the
  // `\and` family's whatsits as it reads a break's, by their definition, 2106.06418)
  ("events", 89),
  // (Z4a, 0 -> 2, ruled at Z4ar2: names two kernel definitions (`\begin`, `\end`), not a token's spelling: an environment step by its
  // meaning, a `\let` copy too, what no frame says for a raw environment)
  ("field_notes", 2),
  ("ledger", 29),
  ("marks", 0),
  ("mod", 0),
  ("policy", 0),
  // (Z14 101 -> 109, the table rows above; Z4a +3: the rehearsal's `\lx@fm@obs@footnote`, the footnote a note step hears, defined and named;
  // N2r2 +7: the bound constructors' `alias`es, the commands they revert as — what a text made of an argument prints, never
  // the rehearsal's name: 2106.06418, 1401.5951, 2106.11154)
  ("rehearsal", 119),
  // (also among `resolver`'s 18 in `TOKEN_READS`: a `'\\'` read of a label, a call's hook)
  ("resolver", 3),
  ("shadow", 7),
  // (Z4a, 81 -> 107: `writer/emit.rs`, the calls it builds: Perl's `\lx@add@authors`, `\lx@annotate@frontmatter@now`, the
  // queue's own `\lx@frontmatter@item*`; it writes them, no decision reads them)
  ("writer", 107),
];

/// revZ2r2: the token-read count's regexes are blind outside `resolver/` to a spelling not written as a comparison. This counts what such
/// a rule cannot do without: a literal holding a backslash, in any file under `observer/`, a new file a row of its own.
#[test]
fn the_observers_spellings_are_counted_however_they_read() {
  for rule in [
    r#"fn spelled(x: &str) -> bool { x.contains("\\foo") }"#,
    r#"const FOO: &str = "\\foo";"#,
    r#"if x.eq("\\foo") {}"#,
    r##"if x == r#"\foo"# {}"##,
    r"if x == '\\' {}",
  ] {
    assert_eq!(backslash_literals(&lex_rust(rule)), 1, "unseen: {rule}");
  }
  for clean in [
    r#"if x.contains("foo") {}"#,
    r#"let line = format!("a\nb");"#,
    "// x.contains(\"\\\\foo\")",
    "#[cfg(test)]\nmod tests { const A: &str = \"\\\\foo\"; }",
  ] {
    assert_eq!(backslash_literals(&lex_rust(clean)), 0, "{clean}");
  }
  let observer = format!("{BASE}/frontmatter/observer");
  let mut counts: BTreeMap<String, usize> = BTreeMap::new();
  for file in rust_files(&observer) {
    let inner = file
      .strip_prefix(&format!("{observer}/"))
      .expect("under observer/");
    let area = inner.split('/').next().expect("a path");
    *counts
      .entry(area.trim_end_matches(".rs").to_string())
      .or_default() += backslash_literals(&read_rust(&file));
  }
  assert_debts(
    "the backslash literals under observer/",
    OBSERVER_BACKSLASH_LITERALS,
    &counts,
    "A decision reads events or digested XML, never a token's spelling: docs/parity/FRONTMATTER_DIGESTED_REFACTOR.md.",
  );
}

/// The token-read check: a decision reads events or digested XML, never a token's spelling.
#[test]
fn token_reads_only_fall_toward_the_floor() {
  const WHY: &str = "Frontmatter decisions read events or digested XML, never token or source spelling: \
                     docs/parity/FRONTMATTER_DIGESTED_REFACTOR.md. The count is over-approximate, as the study's script \
                     is: a line that only looks like a read counts until it is reworded or the code is gone.";
  let read = Regex::new(concat!(
    r"(?:(?:==|!=)\s*&?\*?\s*T_[A-Z]+!)|(?:T_[A-Z]+!\([^)]*\)\s*(?:==|!=))|(?:contains\(&?T_[A-Z]+!)",
    r"|(?:matches!\([^)]*T_[A-Z]+!)|(?:position_of\()|(?:get_catcode\(\))|(?:Catcode::[A-Z]+)|(?:\.get_cs\(\))",
    r"|(?:\.cs_name\(\))|(?:\.get_string\(\))|(?:tokens?\.to_string\(\))|(?:content\.to_string\(\))|(?:\bToString\b)",
    r"|(?:\|\s*T_[A-Z]+!)|(?:Some\(&?T_[A-Z]+!)|(?:\bis_cs\()|(?:\.as_cs\()|(?:meaning\()",
    r"|(?:\bwith_str\()|(?:\bcs_in\()|(?:\btext_tokens\()|(?:to_string\(\)\s*(?:==|!=))",
    r#"|(?:(?:==|!=)\s*&?\*?\s*"\\\\)|(?:"\\\\[^"]*"\s*(?:==|!=|\||=>))"#,
    // (a name's prefix or suffix; a `TOKEN_…` constant compared; a table of names or of tokens, `.contains` read off it; a
    // method on a token list's text)
    r#"|(?:\.(?:starts_with|ends_with|strip_prefix|strip_suffix)\(\s*"\\\\)"#,
    r"|(?:(?:==|!=)\s*&?\*?\s*TOKEN_[A-Z_]+)|(?:TOKEN_[A-Z_]+\s*(?:==|!=))",
    r#"|(?:\[\s*\(?"\\\\)|(?:=\s*&?\[\s*&?T_[A-Z]+!)|(?:\[\s*&?T_[A-Z]+![^\]]*\]\s*\.)"#,
    r"|(?:to_string\(\)\s*\.\s*(?:as_str|contains|ends_with|eq|find|matches|split|starts_with|strip_prefix|strip_suffix|trim))",
  ))
  .expect("the regex");
  // (a line, what rustfmt moves to lines of its own joined to it: a chained call, `.to_string()⏎.contains(…)`, and a table's
  // first row, `&[⏎"\\a",`, so that a table counts once)
  let reads = |file: &str| {
    let mut logical: Vec<String> = Vec::new();
    for line in code_lines(file) {
      match logical.last_mut() {
        Some(last) if line.trim_start().starts_with('.') || last.trim_end().ends_with('[') => {
          last.push_str(line.trim_start())
        },
        _ => logical.push(line),
      }
    }
    logical.iter().filter(|line| read.is_match(line)).count()
  };
  let mut counts: BTreeMap<String, usize> = BTreeMap::new();
  let mut listed: Vec<String> = Vec::new();
  for (area, path) in OTHER_AREAS {
    let files = rust_files(path);
    let n: usize = files.iter().map(|file| reads(file)).sum();
    counts.insert(area.to_string(), n);
    listed.extend(files);
  }
  let frontmatter = format!("{BASE}/frontmatter/");
  for file in rust_files(&format!("{BASE}/frontmatter")) {
    let inner = file.strip_prefix(&frontmatter).expect("under frontmatter/");
    let area = FRONTMATTER_AREAS
      .iter()
      .find(|(_, path)| inner.starts_with(path))
      .map_or("frontmatter (rest)", |(area, _)| area);
    // (`resolver/` is at its floor but for a few: what it holds is counted, in any shape, not what looks like a read)
    *counts.entry(area.to_string()).or_default() += if area == "resolver" {
      resolver_reads(&read_rust(&file))
    } else {
      reads(&file)
    };
    listed.push(file);
  }
  // (the catch-all: a read moved out of an area to a file none lists is still counted)
  for file in rust_files(BASE) {
    if !listed.contains(&file) {
      let rest = file.strip_prefix(BASE).expect("under base_utilities/");
      counts.insert(format!("rest {rest}"), reads(&file));
    }
  }
  assert_debts("the token reads", TOKEN_READS, &counts, WHY);
}

/// What a pass after the writer may change, on the way to deleting the old layers: passes may change kind and text, never
/// who holds an item (the plan's definition of done, item 5). `OwnerDebt` is what breaks that today: the plan's Z8, Z9 pay
/// each one down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PassMay {
  /// nothing: counts, flags or reads
  ChangeNothing,
  /// an item's kind (and with it the words a Kind step takes out)
  ChangeKind,
  /// an item's printed text
  ChangeText,
  /// drop an empty or repeated item, or a line of a note
  Drop,
  /// **moves items between owners**: a debt, witnesses 2511.18438, 2608.22772
  OwnerDebt,
}

/// The passes after `write_observed_frontmatter` in `insert_frontmatter`, in order, each classed. An allow-list and a
/// debt: the three owner-moving ones go before the merge.
const PASSES_AFTER_THE_WRITER: &[(&str, PassMay)] = &[
  ("count_lost_marks", PassMay::ChangeNothing),
  ("join_split_address_contacts", PassMay::ChangeText),
  ("kind_address_lines", PassMay::ChangeKind),
  ("link_entry_addresses", PassMay::OwnerDebt),
  ("drop_empty_contacts", PassMay::Drop),
  ("distribute_address_lists", PassMay::OwnerDebt),
  ("write_role_notes", PassMay::ChangeKind),
  ("show_line_marks", PassMay::ChangeText),
  ("kind_dedication_lines", PassMay::ChangeKind),
  ("kind_legend_contacts", PassMay::ChangeKind),
  // (the URL part moves a link to the block: Z9's; the grade part is a kind)
  ("kind_link_and_grade_contacts", PassMay::OwnerDebt),
  ("drop_repeated_contacts", PassMay::Drop),
  ("flag_merged_creators", PassMay::ChangeNothing),
  ("keep_unclaimed_net_notes", PassMay::Drop),
  // (the content invariant on the placed output, under `=2`: it counts, N2)
  ("check_placed_content", PassMay::ChangeNothing),
  ("read_back_frontmatter", PassMay::ChangeNothing),
];

/// The calls after the writer in `insert_frontmatter`'s body, in order, by the path they are written with: any call of a
/// lower-case name that is no method, so a pass taking other arguments, or in another module (`relocate::new_pass(document)?`),
/// is seen too.
fn calls_after_the_writer() -> Vec<String> {
  let lines = code_lines(&format!("{BASE}/frontmatter/insert.rs"));
  let start = lines
    .iter()
    .position(|l| l.starts_with("pub fn insert_frontmatter("))
    .expect("insert_frontmatter");
  let end = start
    + lines[start..]
      .iter()
      .position(|l| l.starts_with('}'))
      .expect("its end");
  let writer = start
    + lines[start..end]
      .iter()
      .position(|l| l.contains("write_observed_frontmatter("))
      .expect("the writer");
  let call =
    Regex::new(r"(?:^|[^.:\w])((?:[A-Za-z_]\w*::)*[a-z_][a-z_0-9]*)\(").expect("the regex");
  lines[writer + 1..end]
    .iter()
    .flat_map(|line| call.captures_iter(line))
    .map(|c| c[1].to_string())
    .collect()
}

/// The pass-list guard: a new pass after the writer fails until classed, and the owner-moving ones stay named as debts.
#[test]
fn passes_after_the_writer_are_an_allow_list() {
  let calls = calls_after_the_writer();
  // (a pass running twice — after the Kind step reads address lines — is one entry)
  let mut once: Vec<&str> = Vec::new();
  for call in &calls {
    if !once.contains(&call.as_str()) {
      once.push(call);
    }
  }
  let listed: Vec<&str> = PASSES_AFTER_THE_WRITER
    .iter()
    .map(|(name, _)| *name)
    .collect();
  assert_eq!(
    once, listed,
    "a pass after `write_observed_frontmatter` is classed in `PASSES_AFTER_THE_WRITER` by what it may change: kind and \
     text, never who holds an item (docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md, definition of done item 5). An \
     ownership rule lands in `observer/resolver/`."
  );
  assert_eq!(
    PASSES_AFTER_THE_WRITER
      .iter()
      .filter(|(_, may)| *may == PassMay::OwnerDebt)
      .map(|(name, _)| *name)
      .collect::<Vec<_>>(),
    [
      "link_entry_addresses",
      "distribute_address_lists",
      "kind_link_and_grade_contacts"
    ],
    "a debt paid goes from the list; no pass joins it"
  );
  assert_eq!(
    calls.last().map(String::as_str),
    Some("read_back_frontmatter"),
    "the read-back reads the final document"
  );
}

/// `tex` converted under the census: its `readback`, with its Error and Warning counts.
fn read_back(tex: &str) -> (usize, usize, Value) {
  let (log, _, json) = convert_with_setup_then(
    tex,
    Some("ar5iv.sty"),
    || set_fm_shadow_override(Some(true)),
    |_| fm_shadow_json(),
  );
  let dump: Value = serde_json::from_str(&json).expect("the shadow JSON");
  (
    error_count(&log),
    warning_count(&log),
    dump["readback"].clone(),
  )
}

/// A paper's outcome, what the writer's own check and the ledger say, each path its own.
#[test]
fn each_outcome_is_read_back() {
  for (tex, counts, want) in [
    // (the reading and today's agree: the final document reads as the reading)
    (
      r"\documentclass{article}\begin{document}\title{T}\author{Ann Able$^{1}$ \and Bob Baker$^{2}$\\ $^1$Univ A\\ $^2$Univ B}\maketitle Body.\end{document}",
      (0, 0),
      ("same", None, true),
    ),
    // (the writer moved the symbol lines: written, and it reads back)
    (
      r"\documentclass{article}\begin{document}\title{T}\author{Ann Able$^{\dag}$ \and Bob Baker$^{\ddag}$ \and Cat Cole$^{\dag}$\\ \dag Univ A\\ \ddag Univ B}\maketitle Body.\end{document}",
      (0, 0),
      ("written", None, true),
    ),
    // (no author at all: no reading, and the frontmatter was still placed for the title)
    (
      r"\documentclass{article}\begin{document}\title{T}\maketitle Body.\end{document}",
      (0, 0),
      ("no-reading", None, true),
    ),
    // (no frontmatter at all: `insert_frontmatter` never gets to the read-back)
    (
      r"\documentclass{article}\begin{document}Body.\end{document}",
      (0, 0),
      ("no-reading", None, false),
    ),
    // (an error inside the block held: the writer stays out and says why)
    (
      r"\documentclass{article}\begin{document}\title{T}\author{Ann Able\undefinedcommand}\maketitle Body.\end{document}",
      (1, 0),
      ("held-or-skipped", Some("held diagnostics"), true),
    ),
  ] {
    let (errors, warnings, back) = read_back(tex);
    assert_eq!((errors, warnings), counts, "{tex}");
    assert_eq!(
      (
        back["outcome"].as_str().unwrap_or(""),
        back["reason"].as_str(),
        back["reached"].as_bool().unwrap_or(false)
      ),
      want,
      "{tex}: {back}"
    );
    if want.0 == "same" || want.0 == "written" {
      assert_eq!(back["difference"], json!(null), "{tex}: {back}");
    }
  }
}

macro_rules! repro {
  ($name:literal) => {
    include_str!(concat!(
      "../../../tools/perfect_kernel/repros/sectioning-frontmatter/",
      $name,
      ".tex"
    ))
  };
}

/// The reason a paper was declined, one fixture per path a reason takes to the ledger: the re-part's refusals the plan's
/// definition keeps (today's name dropped, more authors by no ruling, read from words alone, no name) and those the
/// plan's batches pay down, the plan's, the re-part that did not read back, the writer's own.
#[test]
fn a_declined_paper_says_why() {
  for (tex, counts, reason) in [
    (
      repro!("author_prefix_marked_name_after_lines"),
      (0, 0),
      "a name of today's the reading makes none",
    ),
    (
      repro!("author_marked_list_with_an_unmarked_name_is_a_reported_merge"),
      (1, 0),
      "an item given more authors by no ruling",
    ),
    (
      repro!("author_wrapped_name_and_institution_kept_as_written"),
      (0, 0),
      "an item read from its words alone",
    ),
    (
      repro!("authblk_address_line_names_no_block"),
      (0, 0),
      "no named creator",
    ),
    (
      repro!("ieee_refmark_orcid_digits_are_no_name"),
      (0, 0),
      "a name today's names do not print",
    ),
    (
      repro!("author_emulateapj_affiliation_line_altaffilmark_stays_with_author"),
      (0, 0),
      "an owner change no ruling decided",
    ),
    (
      repro!("displaced_line_of_several_requested_entries_is_cut_by_the_api"),
      (0, 0),
      "the re-part did not read as the reading",
    ),
    (
      // (Z14r2: `email_own_call_misread_name_word_gives_way` no longer reaches it — its `Italy. email:` piece reads `Italy.`)
      repro!("concede_ink_in_doubt_mark_before_the_piece"),
      (0, 0),
      "a conceded creator's ink",
    ),
  ] {
    let (errors, warnings, back) = read_back(tex);
    assert_eq!((errors, warnings), counts, "{tex}");
    assert_eq!(back["outcome"], "declined", "{tex}: {back}");
    let text = back["reason"].as_str().unwrap_or("");
    assert!(
      text == reason || text.starts_with(&format!("{reason}: ")),
      "{reason}: {back}"
    );
  }
}

/// A paper the writer held that a pass after it changed: `same` by the writer's own check, and the read-back names where
/// the final document differs (the `changed_after` count: a kind or text a pass may change, an owner it may not, the debt
/// Z8 and Z9 pay); so too `written`. An API-only paper's reading is read back though no writer ran (`no-block`, what wave 5
/// opens).
#[test]
fn a_pass_after_the_writer_shows_in_the_read_back() {
  let (errors, warnings, back) = read_back(repro!("email_list_split_order_only"));
  assert_eq!((errors, warnings), (0, 0));
  assert_eq!(back["outcome"], "same", "{back}");
  assert!(back["difference"].is_string(), "{back}");
  let (errors, warnings, back) =
    read_back(repro!("addr_line_one_named_author_beside_unparted_names"));
  assert_eq!((errors, warnings), (0, 0));
  assert_eq!(back["outcome"], "written", "{back}");
  assert!(back["difference"].is_string(), "{back}");
  let (errors, warnings, back) = read_back(repro!("interspeech_author_keys"));
  assert_eq!((errors, warnings), (0, 0));
  assert_eq!(back["outcome"], "no-block", "{back}");
  assert_eq!(back["reached"], true, "{back}");
}
