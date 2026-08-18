use std::{path::Path, rc::Rc};

// Top-level re-exports + the `Token!` macro (distinct from
// `latexml_core::token::Token` type imported above).
use latexml_core::{
  CharToken, Core, Debug, Error, Explode, Fatal, T_CS, T_SPACE, Token, fatal, map, s,
};
use latexml_core::{
  common::{
    DigestionMode, arena,
    error::{self, Result, emit_info, emit_warn, note_begin, note_end},
    model,
    store::Stored,
  },
  definition::expandable::Expandable,
  digested::Digested,
  document::Document,
  gullet,
  list::List,
  pin,
  rewrite::{Rewrite, RewriteOptions},
  state::{self, Scope},
  stomach,
  token::{Catcode, Token},
  tokens::Tokens,
  util::{pathname, pathname::PathnameFindOptions},
};
use latexml_math_parser::MathParser;
use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::FxHashMap as HashMap;

// Process-once cached env var (see WISDOM #56 — getenv hot-path race).
static LATEXML_DUMP: Lazy<Option<String>> = Lazy::new(|| std::env::var("LATEXML_DUMP").ok());

/// The latexml-oxide version exposed to bindings as the state value
/// `LATEXML_VERSION` — the Rust analog of Perl's `$LaTeXML::VERSION`. This is
/// **our own** crate version (`latexml_oxide`), not the emulated Perl LaTeXML's,
/// rendered as a bare `X.Y.Z`: Cargo's `_MAJOR`/`_MINOR`/`_PATCH` components drop
/// any `-rc`/pre-release suffix, so a version-gate parser (BookML's `.ltxml`
/// check or the XSLT `b:version-leq`) sees three integer parts. `latexml_contrib`
/// and `latexml_post` can't read this crate's version directly (reverse dep), so
/// it is injected via state at session init and read back where needed.
pub const LATEXML_VERSION: &str = concat!(
  env!("CARGO_PKG_VERSION_MAJOR"),
  ".",
  env!("CARGO_PKG_VERSION_MINOR"),
  ".",
  env!("CARGO_PKG_VERSION_PATCH"),
);
use latexml_package::prelude::{
  InputDefinitionOptions, InputOptions, input_content, input_definitions,
};

/// Perl `Core.pm` L272 `s/^\[([^\]]*)\]//` — the preload option bracket, which
/// comes at the *front* of the spec (`[twocolumn,11pt]article.cls`).
static LATEX_OPTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\[([^\]]*)\]").unwrap());

// Regex for parsing DefMathRewrite calls from .latexml files
// Matches: DefMathRewrite( ... );
static DEF_MATH_REWRITE_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(?s)DefMathRewrite\(([^;]+)\);").unwrap());
// Key-value patterns within DefMathRewrite
static SCOPE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"scope\s*=>\s*'([^']+)'").unwrap());
static MATCH_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"match\s*=>\s*'([^']*)'").unwrap());
static ROLE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"role\s*=>\s*'([^']+)'").unwrap());
static NAME_ATTR_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(?:^|,)\s*name\s*=>\s*'([^']*)'").unwrap());
static MEANING_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"meaning\s*=>\s*'([^']*)'").unwrap());

#[derive(Default)]
pub struct DigestionOptions {
  pub mode:         Option<DigestionMode>,
  pub noinitialize: Option<bool>,
  pub preamble:     Option<String>,
  pub postamble:    Option<String>,
}

pub trait DigestionAPI {
  fn initialize_singletons(&mut self, preloads: Vec<String>) -> Result<()>;
  fn digest(
    &mut self,
    request: String,
    preamble: Option<String>,
    postamble: Option<String>,
    mode: Option<DigestionMode>,
    no_init: bool,
  ) -> Result<Digested>;
  fn digest_setup(
    &mut self,
    request: String,
    preamble: Option<String>,
    postamble: Option<String>,
    mode: Option<DigestionMode>,
  ) -> Result<String>;
  fn digest_file(&mut self, request: String, options: DigestionOptions) -> Result<Digested>;
  fn digest_internal(&mut self) -> Result<Digested>; // used to be "finishDigestion"
  fn convert_file(&mut self, filepath: String) -> Result<Document>;
  /// Streaming (fragmented) conversion: interleaved digest→build with
  /// spill-to-disk, a streaming pass 2, and placeholder-spliced assembly.
  /// The Perl-parity eager path is `digest` + `convert_document`; this is the
  /// sanctioned bounded-memory divergence (OXIDIZED_DESIGN), activated only
  /// via `Config::streaming`.
  fn convert_streaming(
    &mut self,
    request: String,
    preamble: Option<String>,
    postamble: Option<String>,
    mode: Option<DigestionMode>,
    budget: usize,
  ) -> Result<Document>;
  fn convert_document(&mut self, digested: Digested) -> Result<Document>;
  // Mocks
  /// Load preamble content. Perl: Core.pm loadPreamble
  fn load_preamble(&mut self, preamble: String) {
    let content = if preamble == "standard_preamble.tex" {
      "literal:\\documentclass{article}\\begin{document}".to_string()
    } else {
      preamble
    };
    input_content(&content, InputOptions::default()).ok();
  }
  /// Load postamble content. Perl: Core.pm loadPostamble
  fn load_postamble(&mut self, postamble: String) {
    let content = if postamble == "standard_postamble.tex" {
      "literal:\\end{document}".to_string()
    } else {
      postamble
    };
    input_content(&content, InputOptions::default()).ok();
  }
}

/// Parse a preload spec into `(name, ext, options)`.
///
/// Mirrors Perl `Core.pm:initializeState` (regexes
/// `s/^\[([^\]]*)\]//` then `s/\.(\w+)$//`): the option bracket
/// comes at the *front*, e.g. `[ids,mathlexemes]latexml.sty`.
/// Defaults to `ext = "sty"` when the spec has no `.<ext>` suffix.
pub(crate) fn parse_preload_spec(preload: &str) -> (String, String, Vec<String>) {
  let (base, options) = match preload
    .strip_prefix('[')
    .and_then(|rest| rest.find(']').map(|end| (&rest[..end], &rest[end + 1..])))
  {
    Some((opts_str, rest)) => {
      let opts: Vec<String> = opts_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
      (rest.to_string(), opts)
    },
    None => (preload.to_string(), vec![]),
  };
  let (name, ext) = match base.rfind('.') {
    Some(pos) => (base[..pos].to_string(), base[pos + 1..].to_string()),
    None => (base.clone(), String::from("sty")),
  };
  (name, ext, options)
}

/// One guarded digestion step: `digest_next_body(None)` under the salvage
/// policy `digest_internal` has always applied (extracted verbatim so the
/// eager loop and the streaming driver share ONE policy). `Ok(true)` =
/// continue; `Ok(false)` = a Fatal was recovered (announced + latched, boxes
/// salvaged) and digestion must stop; `Err` = a resource fatal that must not
/// be recovered.
/// Can we create files in `dir`? Probes by creating and removing a uniquely
/// named entry — the only answer that is true for the actual operation, since
/// permission bits, read-only mounts, and full filesystems all present
/// differently and `metadata().permissions()` sees none of them reliably.
fn dir_is_writable(dir: &Path) -> bool {
  let probe = dir.join(format!(".latexml-writable-{}", std::process::id()));
  match std::fs::File::create(&probe) {
    Ok(_) => {
      let _ = std::fs::remove_file(&probe);
      true
    },
    Err(_) => false,
  }
}

fn digest_step_guarded(boxes: &mut Vec<Digested>) -> Result<bool> {
  match stomach::digest_next_body(None) {
    Ok(next_bodies) => {
      boxes.extend(next_bodies);
      Ok(true)
    },
    Err(e) => {
      // Re-raise MemoryBudget / wall-clock Timeout (Convert) errors:
      // those are *resource* failures, not recoverable digestion
      // hiccups. Catching them here would silently produce empty
      // output for a runaway-loop paper, masking a real bug and
      // inflating canvas pass rates with empty conversions.
      // R35.A: ensure pathological inputs fail loudly (exit 1+)
      // rather than silently turning into a zero-byte HTML.
      use latexml_core::common::error::{ErrorCategory, ErrorTarget};
      // NOTE the target discrimination on `MemoryBudget`, which is
      // deliberate rather than an oversight. `Timeout`-target is the RSS
      // fuse: real resident memory is already at the ceiling, so
      // continuing means allocating straight into an OOM — there is
      // nothing to do but stop. `Stomach`-target is the box-list ceilings
      // (`box_count_cap` / `box_bytes_budget` / boxing depth), where the
      // salvage below CLEARS the offending accumulation and therefore
      // itself frees the memory; recovering there is safe precisely
      // because the hard RSS backstop above stays non-recoverable
      // underneath it. So the stomach's memory guards stay on the recovery
      // path — a graceful end with as much of the document as was already
      // digested, and the Fatal announced and latched below.
      if matches!(
        (&e.target, &e.category),
        (ErrorTarget::Timeout, ErrorCategory::MemoryBudget)
          | (ErrorTarget::Timeout, ErrorCategory::Convert)
          | (ErrorTarget::Timeout, ErrorCategory::TokenLimit)
          | (ErrorTarget::Timeout, ErrorCategory::PushbackLimit)
      ) {
        emit_warn(
          "recovery",
          "digest_internal",
          &format!(
            "digest_internal: resource failure ({:?}/{:?}) — not recovering",
            e.target, e.category
          ),
        );
        return Err(e);
      }
      // The Err that landed here was raised at Fatal level. We recover
      // BOXES from it (below) — Perl `finishDigestion` L219-220 — but a
      // Fatal-level raise stays FATAL in the document's reported outcome:
      // salvaging content is not licence to reclassify the verdict, and
      // there is deliberately no auto-upgrade to Error severity here (user
      // policy 2026-07-28). The one sanctioned demotion in this codebase is
      // the bibliography's explicit `DEMOTE_FATALS` (`error.rs`), which is
      // opt-in and scoped.
      //
      // `log_fatal()` is the single seam that does BOTH halves: it emits
      // the standard `Fatal:<target>:<category>` line AND latches
      // `LogStatus::Fatal`. This used to be a hand-rolled `log::error!`
      // with a `Fatal:`-prefixed target, avoiding `log_fatal` for fear of
      // "double-incrementing the counter" — unfounded, since the fatal
      // status is a sticky BOOL (`error.rs` `note_status`, guarded by
      // `fatal_status_is_sticky_and_returns_1`). The hand-rolled call used
      // the raw `log`-crate macro, which never reaches `note_status`, so
      // guard fatals raised as a plain `Err` by `stomach::check_timeout`
      // (rather than through `Fatal!`) were never counted at all: the log
      // carried a `Fatal:` line while the run summarised as `Conversion
      // complete: No obvious problems`, status code 0 — "ok" to cortex.
      // Guard: `101_fatal_salvages_partial_document`.
      e.log_fatal();
      emit_warn(
        "recovery",
        "digest_internal",
        &format!("digest_internal: error during recovery digestion: {:?}", e),
      );
      // Recover what the failed body already digested. Without this the
      // "still produce partial output" intent above only worked when the
      // failure landed in a LATER body — a Fatal inside the FIRST one left
      // `boxes` empty and the run wrote a 39-byte empty document, losing a
      // whole paper to one bad construct (arXiv:2508.07407 / ar5iv #556,
      // one pathological `\tikz` picture).
      //
      // Scoped to the STOMACH box-cycle guard, deliberately. That guard
      // fires while the token stream is still healthy — one construct is
      // piling up boxes — so the surrounding document is sound and worth
      // keeping, and the innermost level (the 50k-box repeating window) is
      // exactly the construct to drop.
      //
      // It is NOT extended to the gullet's `Timeout:Recursion`, where the
      // TOKEN stream is the thing looping: measured on arXiv:2605.25400,
      // salvaging there revived a poisoned state that re-entered the same
      // loop during build and turned an 8.7 s fatal into a 2 m 12 s
      // wall-clock timeout writing a ZERO-byte file — strictly worse than
      // the 39-byte stub, for a 1.7 KB gain on the one paper it helped.
      // Same reasoning bars `TooManyErrors`; widening to either needs its
      // own measurement, not an assumption that more salvage is better.
      if matches!(e.target, ErrorTarget::Stomach) {
        let salvaged = stomach::salvage_pending_box_lists(true);
        if !salvaged.is_empty() {
          emit_info(
            "recovery",
            "digest_internal",
            &format!(
              "digest_internal: salvaged {} box(es) digested before the fatal",
              salvaged.len()
            ),
          );
          boxes.extend(salvaged);
        }
      }
      Ok(false)
    },
  }
}

/// The document head shared by the eager and streaming paths: a fresh
/// [`Document`] with the schema model loaded and the preload PIs inserted
/// (the front half of `convert_document`, extracted verbatim).
fn build_document_head(preloads: &[String]) -> Result<Document> {
  let mut document = Document::new();
  {
    // TODO: Can we disentangle the ownership to avoid the clone?
    let paths_stored = state::get_search_paths();
    let schema_paths = paths_stored
      .iter()
      .map(String::as_str)
      .collect::<Vec<&str>>();
    let default_model_load = model::with_schema_data(|schema_opt| match schema_opt {
      None => true,
      Some(v) => v.last() == Some(&pin!("LaTeXML")),
    });
    if default_model_load {
      // Compile-time load of model AND indirect model. Single
      // shared instantiation lives at `crate::load_latexml_default_model`
      // so LTO can keep exactly one `_ModelLoader::build_model` in
      // the final binary (~600 KiB per copy otherwise).
      crate::load_latexml_default_model();
    } else {
      // Eager-load at runtime
      model::load_schema(schema_paths.as_slice())?; // If needed?
    }
    if state::has_search_paths() {
      {
        if state::lookup_bool("INCLUDE_COMMENTS") {
          let paths_string = state::with_search_paths(|paths| {
            paths
              .iter()
              .map(String::as_str)
              .collect::<Vec<&str>>()
              .join(",")
          });
          let attributes = map! {s!("searchpaths") => paths_string};
          document.insert_pi("latexml", Some(attributes))?;
        }
      }
    }
  }

  for preload in preloads {
    if preload.ends_with(".pool") {
      continue;
    }
    // Perl `Core.pm` L268-277 rewrites `$preload` IN PLACE with `s///`, so
    // the option bracket and the `.cls`/`.sty` suffix are gone from the
    // string that becomes the attribute value, and the captured options ride
    // along as a second attribute. `Regex::replace_all` RETURNS a new string
    // rather than mutating its input, so discarding the result — as this loop
    // did until 2026-07-29 — stripped nothing at all:
    // `--preload=[twocolumn,11pt]article.cls` emitted
    // `<?latexml class="[twocolumn,11pt]article.cls"?>` where Perl emits
    // `<?latexml class="article" options="twocolumn,11pt"?>`.
    //
    // Deliberately NOT routed through `parse_preload_spec` above: that
    // splits on the LAST `.` and so would also eat a non-package extension
    // (`--preload=mystyle.tex` -> `package="mystyle"`), while Perl strips
    // only the two literal suffixes and leaves anything else attached. It
    // also trims/drops empty options, where Perl passes `$1` verbatim.
    let mut spec: &str = preload;
    let mut options: &str = "";
    if let Some(bracket) = LATEX_OPTION_REGEX.captures(spec) {
      options = bracket.get(1).map_or("", |m| m.as_str());
      spec = &spec[bracket.get(0).map_or(0, |m| m.end())..];
    }
    let mut attributes: HashMap<String, String> = HashMap::default();
    // Perl's `($options ? (options => $options) : ())`: an empty bracket
    // (`[]name.sty`) is falsy, so it contributes no attribute at all.
    if !options.is_empty() {
      attributes.insert(s!("options"), options.to_string());
    }
    if let Some(class) = spec.strip_suffix(".cls") {
      attributes.insert(s!("class"), class.to_string());
    } else {
      attributes.insert(
        s!("package"),
        spec.strip_suffix(".sty").unwrap_or(spec).to_string(),
      );
    }
    document.insert_pi("latexml", Some(attributes))?;
  }
  Ok(document)
}

/// Load the source-adjacent `.latexml` rewrite-rules file, if present.
/// Perl does this during initialization; we do it post-build so the rules can
/// compile against the built document. Extracted so the streaming path can
/// load rules BEFORE its pass 2 (fragments must see the complete rule set)
/// without `finish_document` loading them a second time.
fn load_source_latexml_rules() {
  // Load .latexml file if it exists alongside the source .tex file.
  // Perl does this automatically during initialization; we do it post-build
  // so the rewrite rules can be compiled against the built document.
  if let Some(Stored::String(source_sym)) = state::lookup_value("SOURCEFILE") {
    let source_path = arena::with(source_sym, |s| s.to_string());
    // Replace .tex extension with .latexml
    let latexml_path = if source_path.ends_with(".tex") {
      source_path.replace(".tex", ".latexml")
    } else {
      format!("{}.latexml", source_path)
    };
    if Path::new(&latexml_path).exists() {
      let _ = load_latexml_file(&latexml_path);
    }
  }
}

/// The whole-document tail shared by both paths: rewrites, `\lxDeclare`
/// application, math parsing, finalize, XMTok-id cleanup (the back half of
/// `convert_document`, extracted verbatim). In streaming mode this runs on
/// the live SPINE — spilled fragments received the same phases fragment-by-
/// fragment in `streaming_pass2` beforehand.
fn finish_document(document: &mut Document) -> Result<()> {
  let has_rewrites = state::has_value("DOCUMENT_REWRITE_RULES");
  if has_rewrites {
    let _gp_rewrite = latexml_core::telemetry::phase(latexml_core::telemetry::Phase::Rewrite);
    note_begin("Rewriting");
    document.mark_xmnode_visibility()?;
    document.load_labels_for_rewrite()?;
    // TODO: What is the right way to do rewrites in a daemon-safe manner?
    if let Some(Stored::VecDequeStored(rules)) = state::remove_value("DOCUMENT_REWRITE_RULES")
      && let Some(root) = document.get_document().get_root_element()
    {
      apply_rewrite_rules(document, rules, &root)?;
    }
    note_end("Rewriting");
  }

  // Apply \lxDeclare declarations: set roles/names/meanings on matching XMTok elements.
  // Must run BEFORE math parsing so the parser sees the updated roles.
  apply_lx_declarations(document, None);

  if !state::get_nomathparse_flag() {
    // Telemetry: count formulae and time the whole Marpa parse pass.
    // Per-formula bucket histogram requires per-call instrumentation
    // inside latexml_math_parser::parser::parse_math; deferred.
    let xmath_count = document.findnodes("//ltx:XMath", None).len() as u32;
    // ADD, not set: `finish_document` runs on the streaming SPINE *after*
    // `streaming_pass2` has already counted every spilled fragment's formulae,
    // so a plain `set` here would clobber the segment tallies and report only
    // the spine's own — near zero on a document that spilled most of itself.
    // Eager is unaffected: the counter starts at 0 and this runs once.
    latexml_core::telemetry::add_formulae(xmath_count);
    let _gp = latexml_core::telemetry::phase(latexml_core::telemetry::Phase::MathParse);
    let mut parser = MathParser::default();
    parser.parse_math(document)?;
    drop(_gp);
    // Post-parse: mark failed XMath nodes as unparsed.
    // The parser's parse_kludge already handles OPEN/CLOSE wrapping + script attachment
    // (parse_kludgeScripts_rec), so we only need to add the unparsed CSS class here.
    if !parser.failed_xmath_ids.is_empty() {
      for mut math_node in document.findnodes("descendant-or-self::ltx:Math[not(@text)]", None) {
        for xmath_child in document.findnodes("ltx:XMath", Some(&math_node)) {
          if parser.failed_xmath_ids.contains(&xmath_child.to_hashable()) {
            document.add_class(&mut math_node, "ltx_math_unparsed")?;
            break;
          }
        }
      }
    }
    // Renumber xml:ids inside parsed XMath subtrees to be sequential in document
    // order. The Marpa parser explores multiple parse alternatives, consuming ID
    // counter slots for pruned nodes. This pass reassigns IDs post-parse.
    renumber_math_ids(document);
    // Fill in \ltx@count@parses markers with actual parse tree counts.
    // Each marker is <ltx:text _parsetrees_marker="true">0</ltx:text>.
    // Find the preceding ltx:Math[@_parsetrees] and copy the count.
    let markers = document.findnodes("//*[@_parsetrees_marker='true']", None);
    for mut marker in markers {
      let count = {
        let preceding = document.findnodes("preceding::ltx:Math[@_parsetrees][1]", Some(&marker));
        preceding
          .into_iter()
          .last()
          .and_then(|m| m.get_attribute("_parsetrees"))
          .unwrap_or_else(|| "0".to_string())
      };
      // Replace the text content with the actual count
      for mut child in marker.get_child_nodes() {
        child.unlink_node();
      }
      let _ = marker.append_text(&count);
      // Remove the marker attribute
      let _ = marker.remove_attribute("_parsetrees_marker");
    }
  }

  // #683 (xworld21): persist NOMINAL_FONT_SIZE as a
  // `<?latexml nominal-font-size="X"?>` processing instruction when it differs
  // from the 10pt default, so post-processing can size font-relative (em)
  // external SVGs correctly (an `em` is `NOMINAL_FONT_SIZE`pt, not always 10pt).
  // Perl does not emit this — NOMINAL_FONT_SIZE is digestion-only (`DEFSIZE`),
  // so this is beyond-Perl. Only a0poster (25), the NNpt class options, and
  // BookML move it off 10, so a normal document's output stays byte-identical
  // (no new PI). `insert_pi` places it before the root, alongside the other
  // `<?latexml …?>` metadata PIs (class/package/graphicspath).
  if let Some(nominal) = state::lookup_float("NOMINAL_FONT_SIZE")
    && (nominal.0 - 10.0).abs() > 1e-6
  {
    let mut attrs = HashMap::default();
    attrs.insert(String::from("nominal-font-size"), nominal.0.to_string());
    document.insert_pi("latexml", Some(attrs))?;
  }

  note_begin("Finalizing");
  document.finalize()?;
  note_end("Finalizing");
  // Perl core produces role="UNKNOWN" for single-letter math tokens.
  // Per-document .latexml files set role="ID" via DefMathRewrite BEFORE parsing.
  // We do NOT apply a blanket conversion — roles are set by rewrite rules only.
  // Cleanup unreferenced xml:ids on XMTok elements generated by the math parser.
  // Must run after finalize (which includes prune_xmduals that may transfer ids).
  document.cleanup_unreferenced_xmtok_ids();
  Ok(())
}

/// Compile and invoke a set of `DefRewrite`/`DefMathRewrite` rules against
/// `root` (extracted verbatim from the eager Rewriting phase). The streaming
/// pass 2 calls this once per FRAGMENT with a snapshot of the rule set — the
/// S2 census showed the production corpus is subtree-local, so per-fragment
/// application is equivalent; `label:`-scoped rules resolve through the
/// fragment's `rewrite_labels`, pre-merged with the spilled-label index.
fn apply_rewrite_rules(
  document: &mut Document,
  rules: std::collections::VecDeque<Stored>,
  root: &libxml::tree::Node,
) -> Result<()> {
  // Step 1: copy the rules locally through Rc, to be able to invoke them with mutable
  // state. (TODO: obviously, this could be avoided if they never needed mutable
  // state. When do they?)
  let mut rewrites = Vec::new();
  for rule in rules {
    if let Stored::Rewrite(mut rewrite_rule) = rule {
      rewrite_rule.compile_clauses(document);
      rewrites.push(rewrite_rule);
    }
  }
  // 31 rules compiled for declare test; XPath matching issue prevents application
  // Step 2: invoke the rewrite rules
  // R35.D instrumentation: print per-rule timing if
  // LATEXML_REWRITE_TIMING=1. Logs BEFORE the rule runs so we can
  // identify the rule that hangs (the timeout watchdog kills
  // mid-rule otherwise).
  let trace_all = std::env::var_os("LATEXML_REWRITE_TIMING").is_some();
  let n_rules = rewrites.len();
  for (idx, mut rewrite_rule) in rewrites.into_iter().enumerate() {
    // Build a useful one-line hint from the rule's options. The
    // Debug impl on RewriteOptions is `<RewriteOptions>` only,
    // so reach into the fields directly.
    let opts = &rewrite_rule.options;
    let mut xpath_hint = format!(
      "select={:?} xpath={:?} regexp={:?} scope={:?} label={:?} clauses={}",
      opts
        .select
        .as_deref()
        .map(|s| s.chars().take(60).collect::<String>()),
      opts
        .xpath
        .as_deref()
        .map(|s| s.chars().take(60).collect::<String>()),
      opts
        .regexp
        .as_deref()
        .map(|s| s.chars().take(60).collect::<String>()),
      opts.scope.as_ref().map(|_| "<scope>"),
      opts.label.as_deref(),
      rewrite_rule.clauses.len(),
    );
    // Dump compiled clauses by op + pattern preview (helps when
    // the options struct itself is empty after compile_clauses
    // moved them into the clauses vec).
    for (ci, c) in rewrite_rule.clauses.iter().enumerate() {
      use std::fmt::Write;
      let _ = write!(
        xpath_hint,
        "\n    [{ci}] op={:?} pat={:?}",
        c.op,
        match &c.pattern {
          latexml_core::rewrite::RewritePattern::String(s) =>
            format!("Str({})", s.chars().take(120).collect::<String>()),
          latexml_core::rewrite::RewritePattern::Tokens(_) => "Tokens(..)".into(),
          latexml_core::rewrite::RewritePattern::Closure(_) => "Closure(..)".into(),
          latexml_core::rewrite::RewritePattern::NodeList(n) => format!("NodeList({})", n.len()),
          _ => "??".into(),
        }
      );
    }
    if trace_all {
      eprintln!(
        "[rewrite-timing] rule #{}/{} START :: {}",
        idx, n_rules, xpath_hint
      );
      // Flush stderr so it appears even if the rule hangs
      use std::io::Write;
      let _ = std::io::stderr().flush();
    }
    let started = std::time::Instant::now();
    rewrite_rule.invoke(document, root)?;
    let elapsed = started.elapsed();
    if trace_all {
      eprintln!(
        "[rewrite-timing] rule #{}/{} END {:.2?}",
        idx, n_rules, elapsed
      );
    } else if elapsed > std::time::Duration::from_secs(5) {
      eprintln!(
        "[rewrite-timing] rule #{}/{} SLOW {:.2?} :: {}",
        idx, n_rules, elapsed, xpath_hint
      );
    }
  }
  Ok(())
}

/// Streaming pass 2: give every spilled fragment the SAME whole-document tail
/// the spine gets — rewrites, `\lxDeclare`, math parsing, per-fragment
/// finalize — then store its final serialized text for the assembly splice.
/// Runs after digestion finished (so the rewrite-rule set is complete) and
/// before `finish_document` consumes the rules for the spine.
/// Root-level id counters (`_ID_counter_*` attrs) must run document-wide:
/// eager mints `id1..idN` across the whole document in one walk, so pass 2
/// seeds each fragment's parse wrapper with the counters carried out of the
/// previous one (`counters` in/out), and the caller seeds the spine's tail
/// from the final state (sweep witness tests/alignment/plainmath.tex, where
/// every fragment restarted at `id1`).
fn streaming_pass2(
  store: &mut latexml_core::sxml::SegmentStore,
  index: &latexml_core::sxml::FragmentIndex,
  node_fonts: &rustc_hash::FxHashMap<u64, latexml_core::common::font::Font>,
  counters: &mut Vec<(String, String)>,
) -> Result<()> {
  use latexml_core::common::error::{ErrorCategory, ErrorTarget};
  // A non-consuming snapshot of the rules: `finish_document` will consume the
  // live entry for the spine afterwards.
  let rules_opt = match state::lookup_value("DOCUMENT_REWRITE_RULES") {
    Some(Stored::VecDequeStored(rules)) => Some(rules),
    _ => None,
  };
  let nomath = state::get_nomathparse_flag();
  let segments: Vec<_> = store.ids().collect();
  // The spilled-label index is the SAME for every fragment, so build it once
  // and share it (Document::rewrite_labels_shared, consulted on a local miss).
  // Copying it into each fragment's own map instead was quadratic: 28,068
  // labels × 459,579 segments on the 131 MB witness = 12.9 billion String
  // allocations, and it dominated pass 2.
  let shared_labels: Option<Rc<rustc_hash::FxHashMap<String, String>>> =
    rules_opt.is_some().then(|| {
      Rc::new(
        index
          .labels()
          .map(|(label, id)| (label.to_string(), id.to_string()))
          .collect::<rustc_hash::FxHashMap<_, _>>(),
      )
    });
  // Rate-limited exactly like the pass-1 telemetry next door (see the
  // `is_power_of_two() || is_multiple_of(65536)` gate in `convert_streaming`):
  // the 131 MB witness spills 459,579 segments, and three ungated `info!` per
  // segment wrote 1.38 M lines — 44% of a 161 MB log — into the in-RAM
  // LOG_BUFFER that pass 1 exists to keep bounded. Dense at the start so short
  // runs and tests still see every line, logarithmic after, 64k floor.
  //
  // The ARGUMENTS must stay inside the gate, not just the macro: each probe
  // reads /proc/self/status, and the `parsed` line re-reads the whole segment
  // off disk purely to print its size.
  fn telemetry_due(n: usize) -> bool { n.is_power_of_two() || n.is_multiple_of(65536) }
  for (seg_idx, seg) in segments.into_iter().enumerate() {
    let due = telemetry_due(seg_idx + 1);
    if store.is_retired(seg) {
      // Inlined into an enclosing segment; its content is processed there.
      continue;
    }
    let meta = store.meta(seg)?.clone();
    // Segments are bounded (one spill run), so plain parsing is as bounded as
    // streaming — and unlike `xmlTextReaderExpand`, it does not mint the
    // libxml2 "default" namespace prefix for default-namespace content
    // (`append_clone`'s doc-comment records the same trap). The streaming
    // TextReader stays reserved for genuinely unbounded files (the post
    // half's pass A over the whole core XML).
    let frag_xml = libxml::parser::Parser::default()
      .parse_string(&store.wrapped_segment(seg)?)
      .map_err(|e| Error {
        target:   ErrorTarget::Internal,
        category: ErrorCategory::Libxml,
        message:  s!("cannot re-parse staged segment {seg}: {e}"),
      })?;
    let mut out = String::new();
    {
      let mut frag = Document::from_xml_document(frag_xml, node_fonts.clone())?;
      if due {
        emit_info(
          "streaming",
          "progress",
          &format!(
            "streaming pass2: segment {seg} ({} KB) parsed; RSS ~{} MB",
            store.read_segment(seg).map(|t| t.len() / 1024).unwrap_or(0),
            latexml_core::watchdog::process_rss_kb().unwrap_or(0) / 1024,
          ),
        );
      }
      frag.scoped_rules_strict = true;
      // A fragment re-emits the (nested) placeholders it contains; only the
      // final assembly resolves them.
      frag.literal_placeholders = true;
      // An ancestor-scoped rewrite covers the whole fragment (field docs).
      frag.fragment_ancestor_ids = meta.ancestors.iter().cloned().collect();
      // Judge the parse wrapper as the segment's REAL parent in schema
      // decisions (empty-`ltx:text` collapse etc.) — see the field docs.
      frag.fragment_parent_qname = meta.parent.as_deref().map(arena::pin);
      // Seed the wrapper root with the carried root-level id counters, so
      // fragment id minting continues the document-wide sequence.
      if let Some(mut root) = frag.get_document().get_root_element() {
        for (key, value) in counters.iter() {
          let _ = root.set_attribute(key, value);
        }
      }
      // No strip needed: pass 1 serializes segments FLAT (`spill_flat`), so
      // the indentation text nodes this used to delete are never created.
      // `strip_indentation_whitespace` unlinked them, and unlink does not
      // free — they were orphaned for the fragment's lifetime.
      // Restore empty text children the parse could not represent (spill-time
      // `_lx_empty_text` markers — see `spill_run`): `<p></p>` must not
      // collapse to `<p/>` across the round-trip.
      for mut marked in frag.findnodes("//*[@_lx_empty_text]", None) {
        let _ = marked.remove_attribute("_lx_empty_text");
        if marked.get_child_nodes().is_empty() {
          // `append_text("")` is a no-op (the fork guards on len > 0), so
          // create a real text node and then empty it in place.
          let _ = marked.append_text("x");
          if let Some(mut text) = marked.get_first_child() {
            let _ = text.set_content("");
          }
        }
      }
      if let Some(rules) = &rules_opt {
        // `Phase::Rewrite` is taken in `finish_document`, which only ever runs
        // on the SPINE — pass 2 calls `apply_rewrite_rules` directly, so every
        // fragment's rewrite work was unattributed.
        let _gp_rewrite = latexml_core::telemetry::phase(latexml_core::telemetry::Phase::Rewrite);
        frag.mark_xmnode_visibility()?;
        frag.load_labels_for_rewrite()?;
        // Share the prebuilt index rather than copying it in; a frag-local
        // label still wins, exactly as the `or_insert_with` copy ensured.
        frag.rewrite_labels_shared = shared_labels.clone();
        if let Some(root) = frag.get_document().get_root_element() {
          apply_rewrite_rules(&mut frag, rules.clone(), &root)?;
        }
      }
      apply_lx_declarations(&mut frag, meta.section_id.as_deref());
      if !nomath {
        // Only probed when the line will actually be emitted — this read used
        // to sit outside the macro, so its /proc cost was paid on every
        // segment even at `--quiet`.
        let rss0 = if due {
          latexml_core::watchdog::process_rss_kb().unwrap_or(0) / 1024
        } else {
          0
        };
        // Same telemetry the EAGER path records (see the `//ltx:XMath` count
        // and `Phase::MathParse` guard above). Streaming recorded neither,
        // while `record_math_parse` inside the parser kept firing — so a
        // streamed job's telemetry.json showed thousands of
        // `math_parse_attempts` against `formulae: 0` and a near-zero
        // `phase_math_parse_us`, systematically, on the biggest papers.
        // Accumulated across segments, since pass 2 parses each separately.
        latexml_core::telemetry::add_formulae(frag.findnodes("//ltx:XMath", None).len() as u32);
        let _gp = latexml_core::telemetry::phase(latexml_core::telemetry::Phase::MathParse);
        let mut parser = MathParser::default();
        parser.parse_math(&mut frag)?;
        drop(_gp);
        if due {
          emit_info(
            "streaming",
            "progress",
            &format!(
              "streaming pass2: segment {seg} math done; RSS {} -> {} MB; arena {} syms",
              rss0,
              latexml_core::watchdog::process_rss_kb().unwrap_or(0) / 1024,
              arena::len(),
            ),
          );
        }
        // Mirror the eager tail: mark failed formulae, renumber math ids
        // (per-Math, so fragment-local by construction).
        if !parser.failed_xmath_ids.is_empty() {
          for mut math_node in frag.findnodes("descendant-or-self::ltx:Math[not(@text)]", None) {
            for xmath_child in frag.findnodes("ltx:XMath", Some(&math_node)) {
              if parser.failed_xmath_ids.contains(&xmath_child.to_hashable()) {
                frag.add_class(&mut math_node, "ltx_math_unparsed")?;
                break;
              }
            }
          }
        }
        renumber_math_ids(&mut frag);
      }
      // The per-fragment share of finalize: XMRef/XMDual pruning and the
      // font/bookkeeping walk. The root-only passes (RDFa prefixes, namespace
      // declarations, schema PI) belong to the SPINE root and must NOT touch
      // a fragment root — they would change its serialization.
      frag.prune_dangling_split_xmrefs()?;
      frag.prune_xmduals()?;
      if let Some(mut root) = frag.get_document().get_root_element() {
        frag.finalize_subtree(&mut root)?;
      }
      frag.cleanup_unreferenced_xmtok_ids();
      // The parsed root is the `_lxfragment` wrapper; the spilled subtrees are
      // its children. Serialize each at the recorded position parameters.
      if let Some(root) = frag.get_document().get_root_element() {
        let _gp_ser = latexml_core::telemetry::phase(latexml_core::telemetry::Phase::Serialize);
        for child in root.get_child_nodes() {
          out.push_str(&frag.serialize_aux(&child, meta.depth, meta.noindent, false));
        }
        // Carry the wrapper's root-level id counters to the next fragment
        // (finalize deliberately leaves the wrapper's bookkeeping attrs in
        // place — see finalize_rec's PostWork).
        counters.clear();
        for (key, value) in root.get_attributes() {
          if key.starts_with("_ID_counter") {
            counters.push((key, value));
          }
        }
      }
    }
    store.finalize_segment(seg, &out)?;
    if due {
      emit_info(
        "streaming",
        "progress",
        &format!(
          "streaming pass2: segment {seg} finalized+dropped; RSS ~{} MB",
          latexml_core::watchdog::process_rss_kb().unwrap_or(0) / 1024
        ),
      );
    }
  }
  Ok(())
}

impl DigestionAPI for Core {
  fn initialize_singletons(&mut self, preloads: Vec<String>) -> Result<()> {
    // reset the error REPORT singleton
    error::initialize_report();
    // Per-conversion notice state in the math parser (the persistent
    // cortex_worker converts many documents per process).
    latexml_math_parser::reset_conversion_notices();
    // Same reason: the only other telemetry reset is `take()`, which the
    // binaries skip entirely when no telemetry sink is configured.
    latexml_core::telemetry::reset();
    // reset localized variables (if_frames, current_token, align state, etc.)
    latexml_core::common::local_assignments::initialize_localized();
    // now handle conversion state
    gullet::initialize_gullet();
    stomach::initialize_stomach();
    // should we reset the model also?
    model::initialize_model();
    // let paths = state::search_paths;
    let dump_path = LATEXML_DUMP.clone();
    // Publish our version (Perl's `$LaTeXML::VERSION`) so bindings can read it —
    // e.g. the Rhai `LaTeXMLVersion()` binding, and the XSLT `LATEXML_VERSION`
    // parameter. Global so it survives the whole conversion.
    state::assign_value("LATEXML_VERSION", LATEXML_VERSION, Some(Scope::Global));
    state::assign_value("InitialPreloads", true, Some(Scope::Global));
    for preload in preloads {
      let (name, ext, options) = parse_preload_spec(&preload);
      let handleoptions = ext == "sty" || ext == "cls";
      // Pass package options via state (Perl: \PassOptionsToPackage equivalent).
      // Match `\PassOptionsToPackage` at latex_constructs.rs L3838-3842: push the
      // `Vec<String>` through `push_value` so it lands as a `Stored::Strings`
      // batch inside the `opt@<name>.<ext>` `VecDequeStored`. The batch shape is
      // what `collect_syms` (binding/content.rs L1157) flattens when
      // `\ProcessOptions*` enumerates declared options; storing a single
      // comma-joined `Stored::String("opt1,opt2")` instead silently bypasses
      // every `DeclareOption!` site, so e.g. dvipsnames/svgnames/x11names
      // palettes never load — visible as `Error:unexpected:Apricot Can't find
      // color named 'Apricot'; assuming Black` on a `[dvipsnames]color.sty`
      // preload.
      if !options.is_empty() {
        let opt_key = format!("opt@{name}.{ext}");
        state::push_value(&opt_key, options)?;
      }
      input_definitions(&name, InputDefinitionOptions {
        extension: Some(ext.into()),
        handleoptions,
        ..InputDefinitionOptions::default()
      })?;
    }
    state::assign_value("InitialPreloads", false, Some(Scope::Global));

    // Load kernel dump AFTER pools (provides TeX/LaTeX macros the pools skipped).
    if let Some(ref dump_path) = dump_path {
      let path = Path::new(dump_path);
      if path.exists() {
        // Rust-native tab-separated format (from --init mode). The
        // Perl-format `dump_loader` was deleted 2026-04-18 (dead code —
        // we never consumed Perl-generated dumps).
        let result = latexml_core::dump_reader::load_native_dump(path);
        match result {
          Ok(count) => {
            eprintln!(
              "[latexml-oxide] Loaded {} kernel definitions from {}",
              count,
              path.display()
            );
          },
          Err(e) => {
            eprintln!("[latexml-oxide] Warning: failed to load dump: {}", e);
          },
        }
      }
    }
    Ok(())
  }

  // TODO: We should choose between this function or digest_file, rather than implement twice,
  // right?
  fn digest(
    &mut self,
    request: String,
    preamble: Option<String>,
    postamble: Option<String>,
    mode: Option<DigestionMode>,
    _no_init: bool,
  ) -> Result<Digested> {
    let digestion_note = self.digest_setup(request, preamble, postamble, mode)?;
    let list = self.digest_internal()?;
    note_end(&digestion_note);
    Ok(list)
  }

  /// The input-side half of [`DigestionAPI::digest`]: canonicalize the
  /// request, seed SOURCEFILE/SOURCEDIRECTORY/SEARCHPATHS/`\jobname`, queue
  /// postamble/source/preamble on the gullet. Returns the progress-note label
  /// the caller must `note_end` when digestion completes. Shared by the eager
  /// path and `convert_streaming` (which drives digestion itself,
  /// fragment-by-fragment).
  fn digest_setup(
    &mut self,
    request: String,
    preamble: Option<String>,
    postamble: Option<String>,
    mode: Option<DigestionMode>,
  ) -> Result<String> {
    let mut _ext = match &mode {
      Some(m) => Some(m.extension()),
      None => Some(DigestionMode::TeX.extension()),
    };
    let mut dir_opt = None;

    // Canonicalize relative paths so `Path::parent()` gives a real directory.
    // `Path::new("foo.tex").parent()` returns `Some("")` (empty string) which
    // poisons SEARCHPATHS / SOURCEDIRECTORY: an empty-string search-path
    // entry resolves files via cwd-name with no normalization, changing the
    // order in which resource files (e.g. `ts1enc.def` vs `t1enc.def`) are
    // discovered. Concrete symptom: TS1 fontmap leaks into control-sequence
    // construction → `cn` characters become `⚮♪` → `\c@cn` undefined →
    // 381-error cascade (paper 0709.2868). Canonicalizing matches Perl's
    // `File::Spec->splitpath` behavior which always yields a real directory.
    let canonical_request = if pathname::is_literaldata(&request) || pathname::is_url(&request) {
      request.clone()
    } else {
      std::fs::canonicalize(&request)
        .ok()
        .and_then(|p| p.to_str().map(String::from))
        .unwrap_or_else(|| request.clone())
    };
    let name = if pathname::is_literaldata(&request) {
      s!("Anonymous String")
    } else if pathname::is_url(&request) {
      request.clone()
    } else {
      let path = Path::new(&canonical_request);
      dir_opt = path.parent();
      match path.file_stem() {
        None => String::from("missing_name"),
        Some(pf) => pf.to_str().unwrap().to_string(),
      }
    };
    // else {
    //   $self->withState(sub {
    //       Fatal('missing_file', $request, undef, "Can't find $mode file $request"); }); } }
    // };
    // Book-scale sources legitimately expand past the arXiv-sized 400M
    // runaway-token backstop; scale it to the input (never lowers, env
    // override wins — see gullet::scale_token_limit_to_source).
    let source_bytes = if pathname::is_literaldata(&request) {
      request.len()
    } else {
      std::fs::metadata(&canonical_request)
        .map(|m| m.len() as usize)
        .unwrap_or(0)
    };
    gullet::scale_token_limit_to_source(source_bytes);
    let digestion_note = s!("Digesting {}", name);
    note_begin(&digestion_note);
    // $self->initializestate::$mode . ".pool", @{ $$self{preload} || [] }) unless
    // $options{noinitialize};
    if !pathname::is_literaldata(&request) {
      state::assign_value("SOURCEFILE", arena::pin(&request), None);
    }
    if let Some(dir) = dir_opt {
      let dir_str = dir.to_str().unwrap_or(".");
      // Perl Core.pm L195-200 unshifts the SOURCE file's directory onto
      // SEARCHPATHS so subsequent `\input`-style lookups resolve relative
      // to the main file. When `canonicalize` succeeded `dir_str` is the
      // absolute parent; if it failed (e.g. file not on disk yet at the
      // time we resolved — unusual for normal latexml CLI invocations
      // but possible for `literal:` etc.) `dir_str` may be empty and an
      // empty entry on SEARCHPATHS is useless. Fall back to CWD in that
      // case so paper-local files (`\input{Chapter/Abstract}` etc.) are
      // discoverable. Witness: arXiv:2604.09744, 2603.04457 (papers
      // bundling subdirectory `\subimport` chains).
      let resolved_dir = if dir_str.is_empty() {
        std::env::current_dir()
          .ok()
          .and_then(|cwd| cwd.to_str().map(String::from))
          .unwrap_or_else(|| ".".to_string())
      } else {
        dir_str.to_string()
      };
      state::assign_value("SOURCEDIRECTORY", arena::pin(&resolved_dir), None);
      // Perl Core.pm L195-200: `$state->unshiftValue(SEARCHPATHS => $dir)`.
      // `unshift` puts the source dir at the FRONT so it's the new "lead"
      // — the same lead `\lx@append@path` reads as the basis for
      // appended subdir paths. Pushing to BACK (`add_search_path`) left
      // the lead as whatever the CLI's `--path` provided (typically
      // `ar5iv-bindings/bindings`); `\subimport{Chapter/}{Abstract}`
      // then appended Chapter/ to ar5iv-bindings/bindings instead of to
      // the paper's directory, and `\input{Abstract}` couldn't resolve.
      // Witness: arXiv:2604.09744, 2603.04457.
      state::search_paths_push_front(resolved_dir);
    }
    //   if defined $dir && !grep { $_ eq $dir } @{ $state->lookupValue('SEARCHPATHS') };
    // $state->unshiftValue(GRAPHICSPATHS => $dir)

    // if defined $dir && !grep { $_ eq $dir } @{ $state->lookupValue('GRAPHICSPATHS') };

    let name_copy = name.clone();
    state::install_definition(
      Stored::Expandable(Rc::new(Expandable {
        cs: T_CS!("\\jobname"),
        paramlist: None,
        expansion: Tokens::new(Explode!(name_copy)).into(),
        ..Expandable::default()
      })),
      None,
    );

    // Reverse order, since last opened is first read!
    // (Perl: Core.pm L154-157 in `digestFile`.)
    if let Some(postamble) = postamble {
      self.load_postamble(postamble);
    }
    input_content(&request, InputOptions::default())?;
    if let Some(preamble) = preamble {
      self.load_preamble(preamble);
    }

    // Now for the Hacky part for BibTeX!!!
    // Perl `Core.pm` L160-162: drain the .bib mouth via the Pre::BibTeX
    // parser, register each entry in the bibtex.rs thread-local
    // registry, and push back a `literal:` wrapper that produces a
    // `\begin{bibtex@bibliography}...\end{bibtex@bibliography}` block
    // for the digester to process.
    if matches!(mode, Some(DigestionMode::BibTeX)) {
      use latexml_engine::pre_bibtex::PreBibTeX;
      let mut bib = PreBibTeX::new_from_gullet(&name);
      match bib.to_tex() {
        Ok(tex) => {
          input_content(&s!("literal:{tex}"), InputOptions::default())?;
        },
        Err(parse_err) => {
          Error!(
            "bibtex",
            "parse_failed",
            s!("Failed to parse BibTeX file {}: {:?}", name, parse_err)
          );
        },
      }
    }

    Ok(digestion_note)
  }

  fn convert_file(&mut self, filepath: String) -> Result<Document> {
    match self.digest_file(filepath, DigestionOptions::default()) {
      Err(e) => Err(e),
      Ok(digested) => self.convert_document(digested),
    }
  }

  fn convert_streaming(
    &mut self,
    request: String,
    preamble: Option<String>,
    postamble: Option<String>,
    mode: Option<DigestionMode>,
    budget: usize,
  ) -> Result<Document> {
    use latexml_core::sxml::{FragmentIndex, SegmentStore};
    let digestion_note = self.digest_setup(request, preamble, postamble, mode)?;
    let mut document = build_document_head(&self.preload)?;
    latexml_core::document::reset_spilled_segment_count();
    // The spill area lives beside the source when that directory is WRITABLE
    // (same volume as the output in the by-far-common layout, so the
    // disk-headroom check measures the filesystem the spill actually
    // consumes), else the system temp dir. The writability test is not
    // hypothetical: auto-activation fires on exactly the large documents that
    // get processed from read-only trees — an arXiv bulk mount, a CI
    // checkout — and creating the directory unconditionally failed the whole
    // conversion there. Literal input has no source directory at all.
    let spill_anchor = match state::lookup_value("SOURCEDIRECTORY") {
      Some(Stored::String(dir)) => {
        let dir = arena::with(dir, |s: &str| std::path::PathBuf::from(s));
        if dir_is_writable(&dir) {
          dir
        } else {
          emit_info(
            "streaming",
            "progress",
            &format!(
              "streaming: {} is not writable; spilling to {} instead",
              dir.display(),
              std::env::temp_dir().display()
            ),
          );
          std::env::temp_dir()
        }
      },
      _ => std::env::temp_dir(),
    };
    let store = SegmentStore::create(&spill_anchor)?;
    // Disk-headroom gate (user requirement 2026-07-29): the spill needs
    // roughly the core-XML size on disk — measured ~12x the source bytes on
    // math-heavy content — and a silent mid-conversion ENOSPC would be the
    // OOM-kill failure mode all over again. Verify up front and raise a
    // Fatal that NAMES the shortfall and the requirement, so the user can
    // free space (or point --dest at a roomier volume) with numbers in hand.
    {
      use latexml_core::common::error::{ErrorCategory, ErrorTarget};
      const SPILL_BYTES_PER_SOURCE_BYTE: u64 = 16; // ~12x measured + slack
      let src_bytes = match state::lookup_value("SOURCEFILE") {
        Some(Stored::String(f)) => {
          arena::with(f, |f| std::fs::metadata(f).map(|m| m.len()).unwrap_or(0))
        },
        _ => 0,
      };
      let need = src_bytes.saturating_mul(SPILL_BYTES_PER_SOURCE_BYTE);
      if let Some(avail) = latexml_core::watchdog::available_disk_bytes(store.dir())
        && avail < need
      {
        stomach::set_fragment_yield_budget(None);
        return Err(Error {
          target:   ErrorTarget::Timeout,
          category: ErrorCategory::MemoryBudget,
          message:  s!(
            "streaming spill needs ~{} MB free under {} but only {} MB is available — free disk space there, or convert with a destination on a roomier volume",
            need / (1024 * 1024),
            store.dir().display(),
            avail / (1024 * 1024)
          ),
        });
      }
    }
    document.set_spill_store(store);
    document.set_defer_root_after_open(true);
    // Pass 1 serializes placeholders literally (nested spills stay nested;
    // the final assembly resolves them recursively — see the field docs).
    document.literal_placeholders = true;
    // Spill segments are an intermediate that pass 2 re-serializes; the
    // indentation pass 1 used to emit was generated, written, read back,
    // parsed into ~40M text nodes and then deleted again. See `spill_flat`.
    document.spill_flat = true;
    let mut index = FragmentIndex::default();
    stomach::set_fragment_yield_budget(Some(budget));
    // Soft-RSS yield: fire regardless of box count once RSS crosses the
    // watermark — the box budget assumes a per-box footprint, and math-dense
    // content blows past it (witness: fuse at 18.8 GB with the box budget
    // untouched). `spill_watermark_bytes` owns the policy, including the
    // `--max-memory=0` case where there is no fuse to divide.
    if let Some(watermark) = stomach::spill_watermark_bytes() {
      stomach::set_fragment_yield_rss_soft_kb(Some(watermark / 1024));
    }

    // Phase clock. A streamed run's cost splits across pass 1 (digest →
    // absorb → spill), pass 2 (per-segment tail) and assembly, and the log
    // carried NO timing at all — the 131 MB witness's 70-minute wall could
    // only be attributed by extrapolating between two segment checkpoints, or
    // by running a paired control binary for another 70 minutes. One elapsed
    // figure per phase seam makes every run self-attributing.
    let phase_clock = std::time::Instant::now();
    // Pass 1: interleaved digest → absorb → spill, at legal seams only.
    let mut fatal_stop = false;
    loop {
      let mut boxes = Vec::new();
      let step = {
        // Streaming had NO telemetry phases at all — every `phase()` guard in
        // the tree is on the eager path — so `TELEMETRY.md`'s "sum of phase
        // wall ≈ total wall, median ≥ 0.92" was silently unmet for every
        // streamed conversion. Reuse the eager phases rather than adding
        // streaming-specific ones: pass 1 IS digest + build, interleaved.
        let _g = latexml_core::telemetry::phase(latexml_core::telemetry::Phase::Digest);
        digest_step_guarded(&mut boxes)
      };
      let yielded = stomach::take_fragment_yielded();
      let mut stopped = match step {
        Ok(keep_going) => !keep_going,
        Err(e) => {
          // Under EAGER digestion a resource fatal (the RSS fuse) is
          // deliberately not recovered — continuing would allocate straight
          // into an OOM. Under STREAMING the calculus inverts: stopping
          // digestion here FREES the box list, the spill below releases the
          // DOM, and the pipeline finishes on already-spilled content — so
          // recovery is not just safe, it is the feature this mode exists
          // for. The Fatal contract still holds: announce + latch the
          // verdict, keep the partial document (user policy 2026-07-28).
          e.log_fatal();
          emit_warn(
            "internal",
            "core_interface",
            &format!(
              "convert_streaming: digestion stopped by a resource fatal ({:?}/{:?}) — keeping the document built so far",
              e.target, e.category
            ),
          );
          // Recover what the interrupted step had digested. Unlike the eager
          // Stomach-target salvage, drop_innermost is FALSE: there is no
          // runaway construct to excise here — the innermost level IS the
          // healthy top-level accumulation, and the budget simply ran out.
          // Safe because everything salvaged is absorbed, spilled and freed
          // immediately below.
          boxes.extend(stomach::salvage_pending_box_lists(false));
          fatal_stop = true;
          true
        },
      };
      if !boxes.is_empty() {
        let digested = Digested::from(List::new(boxes));
        let _g_build = latexml_core::telemetry::phase(latexml_core::telemetry::Phase::Build);
        if let Err(e) = document.absorb(&digested, None) {
          // Same Fatal contract as the eager Build: announce, latch, keep the
          // partial document (recovery is a FEATURE of Fatal).
          e.log_fatal();
          emit_warn(
            "internal",
            "core_interface",
            &format!(
              "convert_streaming: build stopped early ({:?}/{:?}) — keeping the document built so far",
              e.target, e.category
            ),
          );
          fatal_stop = true;
          stopped = true;
        }
      }
      // Perl inserts resources directly once a document exists; the eager
      // path's root-hook drain is deferred here, so fold fresh arrivals in
      // per fragment — mid-digestion consumers (the frontmatter fallback's
      // resource[last()] anchor) depend on them being placed.
      document.process_pending_resources_at_top()?;
      let finishing = stopped || (!yielded && !gullet::has_more_input());
      // Spill policy at the end: a conversion that NEVER yielded fits in RAM
      // whole — spilling it would buy no headroom and cost a full
      // serialize→reparse→reserialize cycle in pass 2 (measured +38% wall
      // time at a roomy cap). But once yields happened, the ceiling is real
      // and the FINAL fragment must spill like every other one: retaining it
      // hands the eager-style spine tail (rewrites, whole-doc math parse,
      // finalize) everything the last fragment held — the witness died in
      // exactly that tail at 24.5 GB after a perfectly bounded pass 1.
      let bounded_mode = stomach::fragment_yield_count() > 0;
      if !finishing || bounded_mode {
        document.spill_closed_subtrees(&mut index)?;
        // Self-healing: entries for nodes that build-time discard paths
        // detached without purging pin whole Digested box trees (see
        // sweep_stale_node_boxes). The threshold keeps the sweep rare and
        // the map bounded; the post-spill spine mark is cheap.
        if document.node_boxes.len() > 1_000_000 {
          document.sweep_stale_node_boxes();
        }
        // Rate-limited: a book-scale run yields MILLIONS of fragments, and
        // every log line lands in the in-RAM LOG_BUFFER — per-fragment
        // telemetry alone wrote a 1.37 GB log on the 131 MB witness,
        // feeding the very creep pass 1 exists to avoid.
        let fragments = stomach::fragment_yield_count();
        // Hand freed spill memory back to the OS. The spilled DOM lives on
        // GLIBC's heap (libxml2 allocates via libc malloc, NOT the Rust
        // global allocator), and glibc keeps freed mid-heap pages mapped —
        // measured on the 131 MB witness: every probed Rust collection flat,
        // C live-heap 4.45 GB peak (heaptrack), yet RSS crept 22→36 GB into
        // the fuse. `malloc_trim(0)` madvises free pages away (glibc ≥2.26
        // releases mid-heap pages too, not just the top). Rate-limited: a
        // trim walks the heap, and a book-scale run yields millions of
        // fragments.
        #[cfg(target_os = "linux")]
        if fragments.is_multiple_of(4096) {
          unsafe {
            libc::malloc_trim(0);
          }
        }
        // The Rust-side analogue: mimalloc (the global allocator) retains
        // freed pages; a forced collect purges them back to the OS. Gated
        // off under dhat-heap, whose tracking allocator replaces mimalloc.
        #[cfg(not(feature = "dhat-heap"))]
        if fragments.is_multiple_of(4096) {
          unsafe {
            libmimalloc_sys::mi_collect(true);
          }
        }
        if fragments.is_power_of_two() || fragments.is_multiple_of(65536) {
          let (index_ids, index_labels, _) = index.sizes();
          // C-heap split (glibc only manages libxml2's allocations — Rust
          // goes through mimalloc): uordblks = live C bytes, fordblks =
          // freed-but-retained. Together with RSS this separates C live
          // growth / C fragmentation / Rust growth.
          #[cfg(target_os = "linux")]
          let (c_live_mb, c_free_mb) = {
            let mi = unsafe { libc::mallinfo2() };
            (mi.uordblks / (1024 * 1024), mi.fordblks / (1024 * 1024))
          };
          #[cfg(not(target_os = "linux"))]
          let (c_live_mb, c_free_mb) = (0usize, 0usize);
          let spine_children = document
            .get_document()
            .get_root_element()
            .map(|r| r.get_child_nodes().len())
            .unwrap_or(0);
          let (mouths, comments) = gullet::queue_sizes();
          emit_info(
            "streaming",
            "progress",
            &format!(
              "streaming: undo {}; mouths {}; comments {}; node_boxes {}",
              state::undo_depth(),
              mouths,
              comments,
              document.node_boxes.len(),
            ),
          );
          emit_info(
            "streaming",
            "progress",
            &format!(
              "streaming: fragment {} absorbed; {} segment(s) staged to disk; RSS ~{} MB; C-live {} MB; C-free {} MB; root-children {}; idstore {}; index {}+{}; arena {}; fonts {}; pending {}",
              fragments,
              latexml_core::document::spilled_segment_count(),
              stomach::last_sampled_rss_kb() / 1024,
              c_live_mb,
              c_free_mb,
              spine_children,
              document.idstore.len(),
              index_ids,
              index_labels,
              arena::len(),
              document.node_fonts.len(),
              document.pending.len(),
            ),
          );
        }
      }
      if finishing {
        break;
      }
    }
    stomach::set_fragment_yield_budget(None);
    stomach::set_fragment_yield_rss_soft_kb(None);
    gullet::flush();
    note_end(&digestion_note);
    let pass1_elapsed = phase_clock.elapsed();
    emit_info(
      "streaming",
      "progress",
      &format!(
        "streaming: PASS 1 done in {:.1?} — {} yield(s), {} segment(s) staged to disk, RSS ~{} MB",
        pass1_elapsed,
        stomach::fragment_yield_count(),
        latexml_core::document::spilled_segment_count(),
        latexml_core::watchdog::process_rss_kb().unwrap_or(0) / 1024,
      ),
    );

    // The ROOT's after-open hooks were DEFERRED during pass 1
    // (`set_defer_root_after_open`): eager semantics guarantee every hook
    // runs with digestion complete, and firing them at fragment 1 was not
    // merely incomplete but harmful — the frontmatter hook consumed EMPTY
    // frontmatter state and marked it done (losing the abstract; sweep
    // witness tests/structure/abstract.tex), and the root-classes hook read
    // a `DOCUMENT_CLASSES` mapping `\maketitle` had not populated yet
    // (dropping `ltx_authors_1line`; gate witness). Dispatch exactly once,
    // now, with digestion finished — the eager timing.
    document.set_defer_root_after_open(false);
    // Late-arrived frontmatter first (canonically positioned after what the
    // mid-digestion insertion already placed), then the root's deferred late
    // hooks (which perform the whole insertion for documents where nothing
    // ran early).
    latexml_engine::base_utilities::insert_late_frontmatter(&mut document)?;
    document.dispatch_deferred_root_hooks()?;
    if fatal_stop {
      // CHEAP partial (Fatal contract under real memory pressure): every
      // further phase allocates while we are AT the ceiling — the first
      // implementation marched from the 18.8 GB cooperative fuse straight
      // into the 24 GB hard watchdog (SIGKILL) doing pass 2. Skip pass 2 and
      // the spine tail: spilled segments splice in their raw pre-finalize
      // form — well-formed XML that still carries `_`-bookkeeping attributes,
      // which a salvaged partial is allowed to. The verdict is already
      // latched Fatal.
      emit_warn(
        "internal",
        "core_interface",
        "convert_streaming: fatal stop — emitting the cheap partial (pass 2 and the finalize tail skipped to avoid allocating at the memory ceiling)",
      );
      // The partial's serialization must still RESOLVE placeholders (raw
      // segments splice recursively) — literal mode was for pass 1 only.
      document.literal_placeholders = false;
      // Flat serialization was for the spill INTERMEDIATE only — the output
      // keeps its formatting.
      document.spill_flat = false;
      return Ok(document);
    }
    // RDFa prefixes used inside spilled content: the finalize scan can no
    // longer see them, so feed the spill-time record in.
    document.add_extra_rdfa_prefixes(index.rdfa_prefixes().map(|(p, _)| p));

    // The rule set must be complete before ANY rewrites run: the source's
    // `.latexml` file loads only now (as in the eager path), and fragments
    // must see it too — which is exactly why pass 1 applies no rewrites.
    load_source_latexml_rules();
    // Pass 2 mutates segments while fragment documents live independently, so
    // it borrows the store OUT of the spine and hands it back for assembly.
    let mut store = document
      .take_spill_store()
      .expect("attached above; nothing detaches it during pass 1");
    let node_fonts = document.node_fonts.clone();
    // Root-level id counters run document-wide (see streaming_pass2's doc):
    // seed pass 2 from the spine root's pass-1 state, and hand the final
    // state back to the root so the spine's own tail continues the sequence.
    let mut counters: Vec<(String, String)> = document
      .get_document()
      .get_root_element()
      .map(|root| {
        root
          .get_attributes()
          .into_iter()
          .filter(|(k, _)| k.starts_with("_ID_counter"))
          .collect()
      })
      .unwrap_or_default();
    let pass2_start = phase_clock.elapsed();
    streaming_pass2(&mut store, &index, &node_fonts, &mut counters)?;
    emit_info(
      "streaming",
      "progress",
      &format!(
        "streaming: PASS 2 done in {:.1?} ({:.1?} cumulative) — {} segment(s), RSS ~{} MB",
        phase_clock.elapsed().saturating_sub(pass2_start),
        phase_clock.elapsed(),
        latexml_core::document::spilled_segment_count(),
        latexml_core::watchdog::process_rss_kb().unwrap_or(0) / 1024,
      ),
    );
    if let Some(mut root) = document.get_document().get_root_element() {
      for (key, value) in &counters {
        let _ = root.set_attribute(key, value);
      }
    }
    // The spine's own rewrite phase must be strict too: its sections are
    // spilled placeholders, so a scope that "isn't here" is in a fragment.
    document.scoped_rules_strict = true;
    // The spine gets the normal whole-document tail; spilled content is
    // invisible to it (placeholders), so root-level passes act on the live
    // root exactly as in the eager path.
    finish_document(&mut document)?;
    // From here serialization splices the processed segments at their
    // placeholders (recursively — pass 1 kept nested spills nested).
    document.literal_placeholders = false;
    // Flat serialization was for the spill INTERMEDIATE only; the spine and
    // the spliced segment text must carry the normal output formatting.
    document.spill_flat = false;
    document.set_spill_store(store);
    Ok(document)
  }

  /// Restriction: convert_document runs on a single thread, and should never try branching out.
  fn convert_document(&mut self, digested: Digested) -> Result<Document> {
    note_begin("Building");
    let mut document = build_document_head(&self.preload)?;
    Debug!("Doc absorb: {:?}", digested);

    // A Build that runs out of budget KEEPS what it has already built. The
    // guard tick inside `absorb` (see `document.rs`) can now raise a resource
    // Fatal mid-build; propagating it with `?` would discard a document that is
    // structurally sound up to the cut and hand the user a 39-byte stub — the
    // same loss `digest_internal` already salvages against on the digestion
    // side. Recovery is a FEATURE of Fatal here (user policy 2026-07-28):
    // announce it, keep the partial document, finish the pipeline gracefully.
    // `log_fatal` both emits the `Fatal:` line and latches the status, so the
    // run still reports as failed and never masquerades as clean.
    if let Err(e) = document.absorb(&digested, None) {
      e.log_fatal();
      emit_warn(
        "internal",
        "core_interface",
        &format!(
          "convert_document: build stopped early ({:?}/{:?}) — keeping the \
         document built so far",
          e.target, e.category
        ),
      );
    }
    note_end("Building");

    load_source_latexml_rules();
    finish_document(&mut document)?;
    Ok(document)
  }

  fn digest_internal(&mut self) -> Result<Digested> {
    let mut boxes = Vec::new();
    while gullet::has_more_input() {
      // Perl finishDigestion L219-220: loop consuming input even after errors.
      if !digest_step_guarded(&mut boxes)? {
        break;
      }
    }
    gullet::flush();
    Ok(Digested::from(List::new(boxes)))
  }

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Mid-level API.

  // options are currently being evolved to accomodate the Daemon:
  //    mode  : the processing mode, ie the pool to preload: TeX or BibTeX
  //    noinitialize : if defined, it does not initialize State.
  //    preamble = names a tex file (or standard_preamble.tex)
  //    postamble = names a tex file (or standard_postamble.tex)

  /// Restriction: `digest_file` runs on a single thread, and should never try branching out.
  fn digest_file(&mut self, mut request: String, options: DigestionOptions) -> Result<Digested> {
    let mut dir = String::new();
    let name;
    // let mut ext = String::new();
    let mode = match options.mode {
      None => DigestionMode::TeX,
      Some(m) => m,
    };

    if pathname::is_literaldata(&request) {
      // ext = mode.extension();
      name = s!("Anonymous String");
    } else if pathname::is_url(&request) {
      // ext = mode.extension();
      name = request.clone();
    } else {
      let ext_str = s!(".{}", mode.extension());
      let request_base = if request.ends_with(&ext_str) {
        request[0..request.len() - ext_str.len()].to_string()
      } else {
        request
      };

      if let Some(pathname) = pathname::find(&request_base, PathnameFindOptions {
        extensions: Some(vec![mode.extension(), String::new()]),
        ..PathnameFindOptions::default()
      }) {
        request = pathname;
        dir = pathname::directory(&request);
        name = pathname::file_stem(&request);
        // Perl Core.pm L195-200 unshifts the SOURCE file's directory onto
        // SEARCHPATHS so subsequent `\input`-style lookups resolve relative
        // to the main file. When the user invokes with a bare filename
        // (e.g. `latexml_oxide neurips_2025.tex` from the paper's cwd),
        // `pathname::find` returns the relative match and `pathname::
        // directory` returns an empty string — which then gets pushed
        // onto SEARCHPATHS as a useless entry. Resolve to CWD in that
        // case so the paper-local Chapter/ etc. are reachable.
        if dir.is_empty()
          && let Ok(cwd) = std::env::current_dir()
        {
          dir = cwd.to_string_lossy().to_string();
        }
      // ext = pathname::extension(&request);
      } else {
        let message = s!("Can't find {} file {} ", mode, request_base);
        fatal!(Core, MissingFile, message);
      }
    }
    note_begin(&s!("Digesting {} {}", mode, name));
    let main_pool = s!("{}.pool", mode);
    let noinitialize = options.noinitialize.unwrap_or(false);
    if !noinitialize {
      let mut preloads = vec![main_pool];
      preloads.extend(self.preload.clone());
      self.initialize_singletons(preloads)?;
    }
    {
      let source_file = if pathname::is_literaldata(&request) {
        None
      } else {
        Some(request.as_str())
      };
      establish_source_context(source_file, &name, &dir);
    }

    // Reverse order, since last opened is first read!
    if let Some(postamble) = options.postamble {
      self.load_postamble(postamble);
    }

    {
      // Make sure the stomach trick is used very *tightly*, always with a surrounding scope.
      input_content(&request, InputOptions::default())?;
    }

    if let Some(preamble) = options.preamble {
      self.load_preamble(preamble);
    }

    // Now for the Hacky part for BibTeX!!!
    // Perl `Core.pm` L160-162: drain the mouth(s) just opened on the
    // .bib file, run the low-level parser to build a registry of
    // BibEntry objects, then push a literal wrapper TeX block that
    // the LaTeX-side digester reads back as
    //   \begin{bibtex@bibliography}
    //     \ProcessBibTeXEntry{<key1>}
    //     ...
    //   \end{bibtex@bibliography}
    if matches!(mode, DigestionMode::BibTeX) {
      use latexml_engine::pre_bibtex::PreBibTeX;
      let mut bib = PreBibTeX::new_from_gullet(&name);
      match bib.to_tex() {
        Ok(tex) => {
          input_content(&s!("literal:{tex}"), InputOptions::default())?;
        },
        Err(parse_err) => {
          Error!(
            "bibtex",
            "parse_failed",
            s!("Failed to parse BibTeX file {}: {:?}", name, parse_err)
          );
        },
      }
    }

    let list = self.digest_internal()?;
    note_end(&s!("Digesting {} {}", mode, name));
    Ok(list)
  }
}

/// Establish the document-global *source context* for a **top-level** document
/// load — `SOURCEFILE`, `SOURCEDIRECTORY`, the front of `SEARCHPATHS`,
/// `GRAPHICSPATHS`, and `\jobname` (Perl Core.pm L195-200). Shared by
/// [`DigestionAPI::digest_file`] (content read from disk) and
/// [`crate::converter::Converter::digest_content_with_provenance`] (content
/// supplied in memory) so the two cannot drift.
///
/// `source_file` is the value for `SOURCEFILE` — the source's path/identity —
/// or `None` for an anonymous/literal source. `jobname` is the bare job name
/// (file stem). `dir` is the source directory (may be empty, e.g. a literal).
///
/// This is for the *main* document only; a nested `\input`/continuation must
/// not reset these document-global values.
pub(crate) fn establish_source_context(source_file: Option<&str>, jobname: &str, dir: &str) {
  if let Some(sf) = source_file {
    state::assign_value("SOURCEFILE", sf.to_string(), None);
  }
  if !dir.is_empty() {
    state::assign_value("SOURCEDIRECTORY", dir.to_string(), None);
  }
  state::search_paths_push_front(dir.to_string());
  // Perl Core.pm L200: unshift GRAPHICSPATHS => $dir unless already present.
  if !state::graphics_paths_contains(dir) {
    state::graphics_paths_push_front(dir.to_string());
  }
  state::install_definition(
    Stored::Expandable(Rc::new(Expandable {
      cs: T_CS!("\\jobname"),
      paramlist: None,
      expansion: Tokens::new(Explode!(jobname)).into(),
      ..Expandable::default()
    })),
    None,
  );
}

/// Load a `.latexml` file alongside a `.tex` source file.
/// Parses `DefMathRewrite(...)` calls and registers them as rewrite rules.
/// Perl loads these automatically; this provides the equivalent for Rust tests.
///
/// Supported patterns:
///   - Single character: `match => 'a'` -> XPath on XMTok text content
///   - Complex patterns (e.g. `\hat{f}`, `f_D`, `f_\WildCard`): skipped
///   - `scope => 'label:...'`: scoped rewrites via label lookup
///   - `attributes => { role => 'FUNCTION' }`: sets role (and optionally name/meaning)
fn load_latexml_file(path: &str) -> Result<()> {
  use latexml_core::rewrite::{RewriteClause, RewriteOperator, RewritePattern};

  let content = match std::fs::read_to_string(path) {
    Ok(c) => c,
    Err(_) => return Ok(()), // File doesn't exist or can't be read
  };

  for cap in DEF_MATH_REWRITE_RE.captures_iter(&content) {
    let body = &cap[1];

    // Extract match pattern
    let match_str = match MATCH_RE.captures(body) {
      Some(m) => m[1].to_string(),
      None => continue, // No match clause, skip
    };

    // Build attributes map from the attributes => { ... } section
    let mut attrs = HashMap::default();
    if let Some(role_cap) = ROLE_RE.captures(body) {
      attrs.insert("role".to_string(), role_cap[1].to_string());
    }
    if let Some(name_cap) = NAME_ATTR_RE.captures(body) {
      attrs.insert("name".to_string(), name_cap[1].to_string());
    }
    if let Some(meaning_cap) = MEANING_RE.captures(body) {
      attrs.insert("meaning".to_string(), meaning_cap[1].to_string());
    }
    if attrs.is_empty() {
      continue; // No attributes to set
    }

    // Check for optional scope
    let scope_str = SCOPE_RE.captures(body).map(|s| s[1].to_string());

    // Use compile_declare_pattern for all patterns (simple + complex).
    // The .latexml match strings use the same format as \lxDeclare body_text:
    //   'f' (simple), 'f_D' (literal subscript), 'f_\WildCard' (wildcard),
    //   '\hat{f}' (accent), "x^{\prime}" (prime).
    let pat = latexml_core::rewrite::declare::compile_declare_pattern(&match_str);
    if pat.xpath.is_empty() {
      continue; // Unrecognized pattern, skip
    }

    // For math mode, append visibility check to XPath
    let xpath = format!("{}[@_pvis and @_cvis]", pat.xpath);

    // Determine select_count based on pattern type
    let select_count = pat.select_count().or(Some(1usize));

    // Build the rewrite rule
    let mut clauses = Vec::new();

    // Add scope clause if present
    if let Some(ref scope) = scope_str {
      clauses.push(RewriteClause::new_uncompiled(
        RewriteOperator::Scope,
        RewritePattern::String(scope.clone()),
      ));
    }

    // Add match clause (pre-compiled as XPath string)
    clauses.push(RewriteClause::new_uncompiled(
      RewriteOperator::Match,
      RewritePattern::String(xpath),
    ));

    // Add attributes clause
    clauses.push(RewriteClause::new_compiled(
      RewriteOperator::Attributes,
      RewritePattern::String(String::new()),
    ));

    let rewrite = Rewrite {
      options: RewriteOptions {
        attributes_map: Some(attrs),
        is_math: true,
        select_count,
        wildcard_paths: pat.wildcard_paths.clone(),
        declare_filter: Some(pat),
        ..RewriteOptions::default()
      },
      clauses,
    };

    state::push_value("DOCUMENT_REWRITE_RULES", rewrite)?;
  }

  Ok(())
}

/// Apply \lxDeclare declarations to the document.
/// Simple fast-path: matches single-token patterns in XMTok elements
/// and sets role/name/meaning attributes.
fn apply_lx_declarations(document: &mut Document, ambient_section: Option<&str>) {
  let decls_str = match state::lookup_value("LATEXML_DECLARATIONS") {
    Some(Stored::String(s)) => arena::with(s, |r| r.to_string()),
    _ => return,
  };
  if decls_str.is_empty() {
    return;
  }

  // Parse declarations:
  // "token_text\trole\tname\tmeaning\tdecl_id\tmatch_font\tscope_prefix".
  // match_font (font_attribute_string of the digested pattern, e.g.
  // "italic"/"bold") makes matching font-aware: a plain italic `$x$` declaration
  // must not annotate a bold `\mathbf{x}` — different fonts denote different
  // meanings; empty when the pattern carried no distinguishing font.
  // scope_prefix carries the section gate for scope=section declarations
  // (INCLUDING untagged ones, which have no decl_id to infer it from).
  let declarations: Vec<(&str, &str, &str, &str, &str, &str, &str)> = decls_str
    .lines()
    .filter_map(|line| {
      let parts: Vec<&str> = line.splitn(7, '\t').collect();
      if parts.len() >= 4 {
        Some((
          parts[0],
          parts[1],
          parts[2],
          parts[3],
          *parts.get(4).unwrap_or(&""),
          *parts.get(5).unwrap_or(&""),
          *parts.get(6).unwrap_or(&""),
        ))
      } else {
        None
      }
    })
    .collect();

  if declarations.is_empty() {
    return;
  }

  // Find all XMTok elements in the document and apply matching declarations.
  // Skip tokens already marked by the rewrite system (_matched) — these were
  // handled by subscript/prime/wildcard patterns which take precedence.
  let xmtoks = document.findnodes("descendant-or-self::ltx:XMTok", None);
  for mut tok in xmtoks {
    if tok.has_attribute("_matched") {
      continue;
    }
    let content = tok.get_content();
    let tok_name = tok.get_attribute("name").unwrap_or_default();
    // Find the section scope of this token (ancestor section's xml:id)
    let tok_scope = {
      let mut scope = String::new();
      let mut cur = tok.get_parent();
      while let Some(p) = cur {
        if p.get_name() == "section" {
          scope = p
            .get_property("id")
            .or_else(|| p.get_attribute("xml:id"))
            .unwrap_or_default();
          break;
        }
        cur = p.get_parent();
      }
      if scope.is_empty() {
        // Streaming fragment: the enclosing section lives on the spine — its
        // id was recorded at spill time (SegmentMeta::section_id).
        scope = ambient_section.unwrap_or_default().to_string();
      }
      scope
    };

    for &(pattern, role, name, meaning, decl_id, match_font, scope_prefix) in &declarations {
      // Match by content text, or by XMTok name attribute (for CS patterns like \circ)
      let matches = content == pattern
        || (!tok_name.is_empty() && pattern.starts_with('\\') && pattern[1..] == tok_name);
      if matches {
        // Font-class check (mirrors declare_node_matches in the rewrite path):
        // discriminate only on the meaningful font *classes* — bold vs not,
        // caligraphic/typewriter family — NOT on an exact font-string match.
        // Exact equality is wrong here because the declaration's digested frame
        // font (e.g. "italic") differs from an upright operator token's font
        // even when they should match (e.g. `$*$` → the ∗ COMPOSEOP token).
        // A plain/italic declaration rejects bold/caligraphic/typewriter tokens
        // (so italic `$x$` skips bold `\mathbf{x}`); a declaration that is
        // itself bold/caligraphic/typewriter requires the token to share it.
        {
          let tf = document.get_node_font(&tok);
          let tok_bold = tf
            .get_series()
            .map(|s| s.as_ref() == "bold")
            .unwrap_or(false);
          let tok_fam = tf.get_family();
          let tok_cal = tok_fam
            .as_ref()
            .map(|f| f.as_ref() == "caligraphic")
            .unwrap_or(false);
          let tok_tt = tok_fam
            .as_ref()
            .map(|f| f.as_ref() == "typewriter")
            .unwrap_or(false);
          let decl_bold = match_font.contains("bold");
          let decl_cal = match_font.contains("caligraphic");
          let decl_tt = match_font.contains("typewriter");
          if tok_bold != decl_bold || tok_cal != decl_cal || tok_tt != decl_tt {
            continue;
          }
        }
        // Scope gate: the explicit scope_prefix (covers UNTAGGED
        // scope=section declarations), falling back to the decl_id's section
        // prefix for older/tagged lines.
        let gate = if !scope_prefix.is_empty() {
          scope_prefix
        } else {
          decl_id.split('.').next().unwrap_or("")
        };
        if (!decl_id.is_empty() || !scope_prefix.is_empty())
          && !gate.is_empty()
          && !tok_scope.is_empty()
          && tok_scope != gate
        {
          continue; // Wrong section — skip this declaration
        }
        if !role.is_empty() {
          let _ = tok.set_attribute("role", role);
        }
        if !name.is_empty() {
          let _ = tok.set_attribute("name", name);
        }
        if !meaning.is_empty() {
          let _ = tok.set_attribute("meaning", meaning);
        }
        if !decl_id.is_empty() {
          let _ = tok.set_attribute("decl_id", decl_id);
        }
        break; // First matching declaration wins
      }
    }
  }
}

/// Fallback parser for unparseable math expressions.
/// Perl: MathParser.pm parse_kludge().
/// Balances OPEN/CLOSE delimiters by wrapping matched groups in XMWrap.
/// Uses document.wrap_nodes for proper namespace handling.
/// Renumber xml:ids inside parsed XMath subtrees so they are sequential in
/// document order. The Marpa parser explores multiple parse alternatives,
/// consuming ID counter slots for pruned nodes (e.g. m1.1, m1.7, m1.12
/// instead of m1.1, m1.2, m1.3). This pure post-processing pass reassigns
/// IDs after all pruning is complete.
///
/// Optimized: single DFS walk per XMath (not XPath), O(1) parent-prefix
/// lookup via ID string parsing, and allocation reuse across Math nodes.
fn renumber_math_ids(document: &mut Document) {
  let xml_ns = "http://www.w3.org/XML/1998/namespace";
  let math_nodes = document.findnodes("descendant-or-self::ltx:Math[@text]", None);

  // Reuse allocations across Math nodes
  let mut id_entries: Vec<(libxml::tree::Node, String)> = Vec::new();
  let mut idref_entries: Vec<(libxml::tree::Node, String)> = Vec::new();
  let mut id_map: rustc_hash::FxHashMap<String, String> = rustc_hash::FxHashMap::default();
  let mut referenced_ids: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();

  for mut math_node in math_nodes {
    let math_id = match math_node.get_attribute_ns("id", xml_ns) {
      Some(id) => id,
      None => continue,
    };

    let xmath_nodes = document.findnodes("ltx:XMath", Some(&math_node));
    for xmath in xmath_nodes {
      id_entries.clear();
      idref_entries.clear();
      id_map.clear();
      referenced_ids.clear();

      // Single DFS walk collects both xml:id and idref nodes in document order
      renumber_collect_dfs(&xmath, xml_ns, &mut id_entries, &mut idref_entries);
      if id_entries.is_empty() {
        continue;
      }

      // Collect all referenced IDs (from XMRef idref attributes)
      for (_, idref) in &idref_entries {
        referenced_ids.insert(idref.clone());
      }

      // Strip xml:id from XMTok elements that are not referenced by any XMRef.
      // The math parser assigns xml:ids to all tokens during parsing, but only
      // structural nodes (XMApp, XMDual) and explicitly referenced tokens need them.
      // Orphan XMTok ids inflate the renumbering counter causing ID gaps.
      {
        let mut stripped = false;
        for (node, id) in &mut id_entries {
          if node.get_name() == "XMTok" && !referenced_ids.contains(id.as_str()) {
            document.unrecord_id(id);
            let _ = node.remove_attribute("xml:id");
            let _ = node.remove_attribute_ns("id", xml_ns);
            id.clear(); // mark for removal
            stripped = true;
          }
        }
        if stripped {
          id_entries.retain(|(_, id)| !id.is_empty());
        }
      }

      if id_entries.is_empty() {
        continue;
      }

      // Build old→new mapping. Flat sequential numbering under the math_id prefix,
      // matching Perl's approach of assigning all IDs at the same level.
      let mut counter = 0u32;
      let mut any_changed = false;
      for (_node, old_id) in &id_entries {
        counter += 1;
        let new_id = format!("{math_id}.{counter}");
        if new_id != *old_id {
          any_changed = true;
        }
        id_map.insert(old_id.clone(), new_id);
      }

      if !any_changed {
        continue;
      }

      // Apply new xml:ids in TWO passes to avoid idstore collisions.
      // A new id like "m1.1" would collide with an old "m1.1" still in the
      // idstore if we interleave unrecord+record. Strip all first, then assign.
      let mut nodes_to_update: Vec<(libxml::tree::Node, String)> = Vec::new();
      for (mut node, old_id) in id_entries.drain(..) {
        if let Some(new_id) = id_map.get(&old_id)
          && new_id != &old_id
        {
          document.unrecord_id(&old_id);
          let _ = node.remove_attribute("xml:id");
          let _ = node.remove_attribute_ns("id", xml_ns);
          nodes_to_update.push((node, new_id.clone()));
        }
      }
      for (mut node, new_id) in nodes_to_update {
        let _ = document.set_attribute(&mut node, "xml:id", &new_id);
      }

      // Update idrefs
      for (mut node, old_idref) in idref_entries.drain(..) {
        if let Some(new_idref) = id_map.get(&old_idref)
          && new_idref != &old_idref
        {
          let _ = node.set_attribute("idref", new_idref);
        }
      }

      // Reset _ID_counter__ on the Math node to the final count
      let _ = math_node.set_attribute("_ID_counter__", &counter.to_string());
    }
  }
}

/// DFS walk collecting nodes with xml:id and idref attributes in document order.
/// Stops at nested `Math` elements (which have their own parsing scope).
fn renumber_collect_dfs(
  node: &libxml::tree::Node,
  xml_ns: &str,
  id_entries: &mut Vec<(libxml::tree::Node, String)>,
  idref_entries: &mut Vec<(libxml::tree::Node, String)>,
) {
  if let Some(id) = node.get_attribute_ns("id", xml_ns) {
    id_entries.push((node.clone(), id));
  }
  if let Some(idref) = node.get_attribute("idref") {
    idref_entries.push((node.clone(), idref));
  }
  for child in node.get_child_elements() {
    // Skip nested Math elements — they have their own parsing scope
    if child.get_name() == "Math" {
      continue;
    }
    renumber_collect_dfs(&child, xml_ns, id_entries, idref_entries);
  }
}

#[cfg(test)]
mod tests {
  use super::{LATEXML_VERSION, parse_preload_spec};

  fn opts(v: &[&str]) -> Vec<String> { v.iter().map(|s| s.to_string()).collect() }

  /// #320: `LATEXML_VERSION` must be a bare `X.Y.Z` (three integer components,
  /// no `-rc`/pre-release) so BookML's version-gate parse works. Guards against a
  /// future "simplify to `env!(\"CARGO_PKG_VERSION\")`" that would re-add `-rc1`.
  #[test]
  fn latexml_version_is_bare_xyz() {
    let parts: Vec<&str> = LATEXML_VERSION.split('.').collect();
    assert_eq!(
      parts.len(),
      3,
      "LATEXML_VERSION must be X.Y.Z, got {LATEXML_VERSION:?}"
    );
    for p in &parts {
      assert!(
        !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()),
        "component {p:?} is not a bare integer in {LATEXML_VERSION:?}"
      );
    }
  }

  #[test]
  fn preload_no_brackets_no_ext() {
    assert_eq!(
      parse_preload_spec("latexml"),
      ("latexml".into(), "sty".into(), opts(&[]))
    );
  }

  #[test]
  fn preload_no_brackets_with_ext() {
    assert_eq!(
      parse_preload_spec("ar5iv.sty"),
      ("ar5iv".into(), "sty".into(), opts(&[]))
    );
    assert_eq!(
      parse_preload_spec("TeX.pool"),
      ("TeX".into(), "pool".into(), opts(&[]))
    );
  }

  #[test]
  fn preload_front_brackets_with_options() {
    // The historical-bug fixture: front-bracket form must produce a real name.
    assert_eq!(
      parse_preload_spec("[ids,mathlexemes]latexml.sty"),
      (
        "latexml".into(),
        "sty".into(),
        opts(&["ids", "mathlexemes"])
      )
    );
    assert_eq!(
      parse_preload_spec("[dvipsnames]color.sty"),
      ("color".into(), "sty".into(), opts(&["dvipsnames"]))
    );
  }

  #[test]
  fn preload_class_with_options() {
    assert_eq!(
      parse_preload_spec("[twocolumn,11pt]article.cls"),
      ("article".into(), "cls".into(), opts(&["twocolumn", "11pt"]))
    );
  }

  #[test]
  fn preload_options_trimmed_and_empty_stripped() {
    assert_eq!(
      parse_preload_spec("[ a , b ,, c ]name.sty"),
      ("name".into(), "sty".into(), opts(&["a", "b", "c"]))
    );
  }

  #[test]
  fn preload_empty_brackets() {
    assert_eq!(
      parse_preload_spec("[]name.sty"),
      ("name".into(), "sty".into(), opts(&[]))
    );
  }

  #[test]
  fn preload_unmatched_bracket_falls_through() {
    // No closing `]` ⇒ treat the whole spec as the base, no options.
    assert_eq!(
      parse_preload_spec("[opt"),
      ("[opt".into(), "sty".into(), opts(&[]))
    );
  }

  #[test]
  fn preload_dot_in_name_uses_last_segment_as_ext() {
    assert_eq!(
      parse_preload_spec("foo.bar.sty"),
      ("foo.bar".into(), "sty".into(), opts(&[]))
    );
  }
}
