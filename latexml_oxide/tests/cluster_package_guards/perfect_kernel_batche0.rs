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
  ("dispatch.rs queue_front_matter(", 1),
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
const OLD_READER_CODE_LINES: usize = 15_420;

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

/// The keys of `\\lx@add@…[…]` calls Perl has too, by its `DefKeyVal('Frontmatter', …)` (`Base_Utility.pool.ltxml:247-254`).
const PERL_DECLARED_KEYS: &[&str] = &[
  "annotate",
  "annotations",
  "class",
  "graphic",
  "label",
  "labelref",
  "labelseq",
  "role",
];

/// The API's keys outside Perl's, per key and file: each `_word=` an API call sets (and `owner=`, the one without the `_`),
/// each string constant naming such a key (`CUT_LIST`, which `s!("{CUT_LIST}={key}")` sets: `const _cutlist`), and each key
/// declared in `Frontmatter` that Perl does not declare. A debt like the names. Not a constructor's XML attribute, which is
/// quoted (`_noautoclose='true'`), nor a constant in `latex_constructs/` or `tex_box.rs` (the kernel's own attributes).
/// `_scopebegin=` is pgfsys's own keyval: it shares the shape, and stays pinned so that the count stays whole.
const API_KEYS_OUTSIDE_PERLS: &[(&str, usize)] = &[
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
  (
    "const _labelprints base_utilities/author_block/affiliations.rs",
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
    "declared accumulate base_utilities/defs/frontmatter_api.rs",
    1,
  ),
  (
    "declared command_form base_utilities/defs/frontmatter_api.rs",
    1,
  ),
  ("declared content base_utilities/defs/frontmatter_api.rs", 1),
  ("declared name base_utilities/defs/frontmatter_api.rs", 1),
  (
    "declared property base_utilities/defs/frontmatter_api.rs",
    1,
  ),
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

/// The boundary is Perl's Frontmatter API: a name or a key outside Perl's is the exception, and the list only shrinks. A
/// name built at run time (`\lx@add@{role}`) is not seen: this reads the literals.
#[test]
fn the_frontmatter_api_stays_perls() {
  const WHY: &str = "The engine relies on Perl LaTeXML's Frontmatter API and its macro dialect, and an extension is the \
                     exception, stated with its reason: docs/parity/FRONTMATTER_SINGLE_ENGINE_PLAN.md, \"The boundary is \
                     Perl's Frontmatter API\". Answer with Perl's macros and keys (`\\lx@add@creator`, `\\lx@add@contact`, \
                     `label=`, `annotations=`).";
  let name = Regex::new(r"\\+(lx@(?:add|annotate|clear)@[A-Za-z@]*)").expect("the regex");
  let key =
    Regex::new(r"(?:^|[^A-Za-z0-9_])(_[a-z][a-z0-9_]*|owner)=(?:[^'=]|$)").expect("the regex");
  let constant =
    Regex::new(r#"\bconst [A-Z][A-Z_0-9]*: &str = "(_[a-z][a-z0-9_]*)""#).expect("the regex");
  let declared = Regex::new(r#"DefKeyVal!\(\s*"Frontmatter",\s*"([^"]+)""#).expect("the regex");
  let mut extensions: Vec<String> = Vec::new();
  let mut keys: BTreeMap<String, usize> = BTreeMap::new();
  for file in API_SPEAKERS
    .iter()
    .flat_map(|crate_src| rust_files(crate_src))
  {
    for code in code_lines(&file) {
      for found in name.captures_iter(&code) {
        // (a bare prefix, as a call test reads it, is no name)
        let found = &found[1];
        if !found.ends_with('@') && !PERL_FRONTMATTER_API.contains(&found) {
          extensions.push(found.to_string());
        }
      }
      let short = file
        .split_once("/src/")
        .map_or(file.as_str(), |(_, inner)| inner);
      for found in key.captures_iter(&code) {
        *keys.entry(format!("{}= {short}", &found[1])).or_default() += 1;
      }
      if !short.starts_with("latex_constructs/") && short != "tex_box.rs" {
        for found in constant.captures_iter(&code) {
          *keys
            .entry(format!("const {} {short}", &found[1]))
            .or_default() += 1;
        }
      }
      for found in declared.captures_iter(&code) {
        if !PERL_DECLARED_KEYS.contains(&&found[1]) {
          *keys
            .entry(format!("declared {} {short}", &found[1]))
            .or_default() += 1;
        }
      }
    }
  }
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

/// Token-spelling reads per area, and in the listed bindings: `bin/tokreads.py`'s count of the removal study plus the shapes
/// it missed, **an over-approximation** (a line of code matching a read's shape, whatever it decides; a string holding `//`
/// is cut there). Debts on the way to the floor: about 60 in the engine (the rehearsal's table, the ledger's stamping, the
/// Perl API boundary), `resolver/` 0, decision code elsewhere 0. The engine's total is 2,264 at Z01: the study's 1,962, plus
/// `title_page_check.rs`, which its list left out (14), plus the helpers' shapes the study did not count (`with_str(`,
/// `cs_in(`, `text_tokens(`, `to_string() ==`, a comparison or a match arm on a quoted `\\…`); and 85 in the rest of
/// `base_utilities/`. `frontmatter (rest)` holds any new file under `frontmatter/`, and a `rest <file>` row each file under
/// `base_utilities/` no area lists, a new one or a sibling directory's too, so a read moved there is no debt paid.
/// Bindings are the study's list, so a binding outside it is not watched.
const TOKEN_READS: &[(&str, usize)] = &[
  ("aa_cls.rs", 5),
  ("aas_support_sty.rs", 17),
  ("acmart_cls.rs", 18),
  ("address_lists", 0),
  ("aistats_sty.rs", 52),
  ("ams_support_sty.rs", 16),
  ("apa7_cls.rs", 7),
  ("authblk_sty.rs", 23),
  ("author_block", 1259),
  ("defs/author_macros", 19),
  ("defs/frontmatter_api", 4),
  ("defs/frontmatter_fields", 20),
  ("defs/general", 9),
  ("epl_cls.rs", 39),
  ("eptcs_cls.rs", 26),
  ("events", 51),
  // (raised from none: moved out of core `token.rs`, unwatched there, for the hot path; a rehearsal's own reads)
  ("field_notes", 4),
  ("frontmatter (rest)", 22),
  ("frontmatter_stores", 485),
  ("gretsi_cls.rs", 14),
  ("ieeetran_cls.rs", 114),
  ("insert", 14),
  ("inst_support_sty.rs", 0),
  ("interspeech_cls.rs", 27),
  ("l_aa_cls.rs", 13),
  ("ledger", 21),
  ("llncs_cls.rs", 32),
  ("osa_journal_sty.rs", 75),
  ("place", 5),
  ("ptephy_cls.rs", 95),
  ("queue", 0),
  ("queue_digest", 12),
  ("rehearsal", 3),
  ("relocate", 131),
  ("resolver", 12),
  ("rest /author_split_tests.rs", 14),
  ("rest /block_insertion.rs", 0),
  ("rest /blocks.rs", 1),
  ("rest /boxes_lists.rs", 0),
  ("rest /defs/mod.rs", 0),
  ("rest /defs/tags_refnums.rs", 0),
  ("rest /fonts_defs.rs", 12),
  ("rest /math_cleanup.rs", 0),
  ("rest /misc.rs", 13),
  ("rest /mod.rs", 0),
  ("rest /tokens.rs", 26),
  ("rest /util_text.rs", 19),
  ("revtex4_support_sty.rs", 12),
  ("shared_row", 13),
  ("shared_row/emails", 60),
  ("sv_support_sty.rs", 36),
  ("webofc_cls.rs", 0),
  ("writer/apply", 1),
  ("writer/dom+mod", 0),
  ("writer/plan", 119),
  ("writer/repart", 0),
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
    r#"|(?:(?:==|!=)\s*&?\*?\s*"\\\\)|(?:"\\\\[^"]*"\s*(?:==|!=|\||=>))"#
  ))
  .expect("the regex");
  let reads = |file: &str| {
    code_lines(file)
      .iter()
      .filter(|line| read.is_match(line))
      .count()
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
    *counts.entry(area.to_string()).or_default() += reads(&file);
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
      repro!("email_own_call_misread_name_word_gives_way"),
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
