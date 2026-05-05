use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use rustc_hash::FxHashMap as HashMap;
use std::path::Path;
use std::rc::Rc;

use latexml_core::common::DigestionMode;
use latexml_core::common::arena;
use latexml_core::common::error::{self, Result, note_begin, note_end};
use latexml_core::common::model;
use latexml_core::common::store::Stored;
use latexml_core::definition::expandable::Expandable;
use latexml_core::digested::Digested;
use latexml_core::document::Document;
use latexml_core::gullet;
use latexml_core::list::List;
use latexml_core::rewrite::{Rewrite, RewriteOptions};
use latexml_core::state::{self, Scope};
use latexml_core::stomach;
use latexml_core::token::{Catcode, Token};
use latexml_core::tokens::Tokens;
use latexml_core::util::pathname;
use latexml_core::util::pathname::PathnameFindOptions;
// TODO: Clean up these imports -- what belongs where?
use latexml_codegen::LoadModel;
use latexml_core::{CharToken, Core, Debug, Explode, T_CS, T_SPACE, Token, fatal, map, s};
use latexml_math_parser::MathParser;

// Process-once cached env var (see WISDOM #56 — getenv hot-path race).
static LATEXML_DUMP: Lazy<Option<String>> = Lazy::new(|| std::env::var("LATEXML_DUMP").ok());
use latexml_package::prelude::{
  InputDefinitionOptions, InputOptions, input_content, input_definitions,
};

static CLS_EXT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.cls$").unwrap());
static STY_EXT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.sty$").unwrap());
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
  fn digest_file(&mut self, request: String, options: DigestionOptions) -> Result<Digested>;
  fn digest_internal(&mut self) -> Result<Digested>; // used to be "finishDigestion"
  fn convert_file(&mut self, filepath: String) -> Result<Document>;
  fn convert_document(&mut self, digested: Digested) -> Result<Document>;
  // Mocks
  /// Load preamble content. Perl: Core.pm loadPreamble
  fn load_preamble(&mut self, preamble: String) {
    let content = if preamble == "standard_preamble.tex" {
      "literal:\\documentclass{article}\\begin{document}".to_string()
    } else {
      preamble
    };
    crate::core_interface::input_content(&content, InputOptions::default()).ok();
  }
  /// Load postamble content. Perl: Core.pm loadPostamble
  fn load_postamble(&mut self, postamble: String) {
    let content = if postamble == "standard_postamble.tex" {
      "literal:\\end{document}".to_string()
    } else {
      postamble
    };
    crate::core_interface::input_content(&content, InputOptions::default()).ok();
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

impl DigestionAPI for Core {
  fn initialize_singletons(&mut self, preloads: Vec<String>) -> Result<()> {
    // reset the error REPORT singleton
    error::initialize_report();
    // reset localized variables (if_frames, current_token, align state, etc.)
    latexml_core::common::local_assignments::initialize_localized();
    // now handle conversion state
    gullet::initialize_gullet();
    stomach::initialize_stomach();
    // should we reset the model also?
    model::initialize_model();
    // let paths = state::search_paths;
    let dump_path = LATEXML_DUMP.clone();
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
      let path = std::path::Path::new(dump_path);
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
    let mut _ext = match mode {
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
    let digestion_note = s!("Digesting {}", name);
    note_begin(&digestion_note);
    // $self->initializestate::$mode . ".pool", @{ $$self{preload} || [] }) unless
    // $options{noinitialize};
    if !pathname::is_literaldata(&request) {
      state::assign_value("SOURCEFILE", arena::pin(&request), None);
    }
    if let Some(dir) = dir_opt {
      let dir = dir.to_str().unwrap_or(".");
      {
        state::assign_value("SOURCEDIRECTORY", arena::pin(dir), None);
        state::add_search_path(dir.to_string());
      }
    }
    //   if defined $dir && !grep { $_ eq $dir } @{ $state->lookupValue('SEARCHPATHS') };
    // $state->unshiftValue(GRAPHICSPATHS => $dir)

    // if defined $dir && !grep { $_ eq $dir } @{ $state->lookupValue('GRAPHICSPATHS') };

    let name_copy = name;
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

    // // Now for the Hacky part for BibTeX!!!
    // if ($mode eq 'BibTeX') {
    //   my $bib = LaTeXML::Pre::BibTeX->newFromGullet($name, $state->getStomach->getGullet);
    //   LaTeXML::Package::InputContent("literal:" . $bib->toTeX); }

    let list = self.digest_internal()?;
    note_end(&digestion_note);

    Ok(list)
  }

  fn convert_file(&mut self, filepath: String) -> Result<Document> {
    match self.digest_file(filepath, DigestionOptions::default()) {
      Err(e) => Err(e),
      Ok(digested) => self.convert_document(digested),
    }
  }

  /// Restriction: convert_document runs on a single thread, and should never try branching out.
  fn convert_document(&mut self, digested: Digested) -> Result<Document> {
    note_begin("Building");
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
        Some(v) => v.last() == Some(&arena::pin_static("LaTeXML")),
      });
      if default_model_load {
        // Compile-time load of model AND indirect model
        load_model!("LaTeXML");
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

    for preload in &self.preload {
      if preload.ends_with(".pool") {
        continue;
      }
      let mut options: Option<String> = None;
      LATEX_OPTION_REGEX.replace_all(preload, |refs: &Captures| -> String {
        options = Some(refs.get(1).map_or("", |m| m.as_str()).to_string());
        String::new()
      });
      if preload.ends_with(".cls") {
        CLS_EXT_REGEX.replace_all(preload, "");
        let attributes = map! {s!("class") => preload.to_string()};
        document.insert_pi("latexml", Some(attributes))?;
      } else {
        STY_EXT_REGEX.replace_all(preload, "");
        let attributes = map! {s!("package") => preload.to_string()};
        document.insert_pi("latexml", Some(attributes))?;
      }
    }
    Debug!("Doc absorb: {:?}", digested);

    document.absorb(&digested, None)?;
    note_end("Building");

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

    let has_rewrites = state::has_value("DOCUMENT_REWRITE_RULES");
    if has_rewrites {
      let _gp_rewrite = latexml_core::telemetry::phase(latexml_core::telemetry::Phase::Rewrite);
      note_begin("Rewriting");
      document.mark_xmnode_visibility()?;
      document.load_labels_for_rewrite()?;
      // TODO: What is the right way to do rewrites in a daemon-safe manner?
      if let Some(Stored::VecDequeStored(rules)) = state::remove_value("DOCUMENT_REWRITE_RULES") {
        if let Some(root) = document.get_document().get_root_element() {
          // Step 1: copy the rules locally through Rc, to be able to invoke them with mutable
          // state. (TODO: obviously, this could be avoided if they never needed mutable
          // state. When do they?)
          let mut rewrites = Vec::new();
          for rule in rules {
            if let Stored::Rewrite(mut rewrite_rule) = rule {
              rewrite_rule.compile_clauses(&mut document);
              rewrites.push(rewrite_rule);
            }
          }
          // 31 rules compiled for declare test; XPath matching issue prevents application
          // Step 2: invoke the rewrite rules
          for mut rewrite_rule in rewrites {
            rewrite_rule.invoke(&mut document, &root)?;
          }
        }
      }
      note_end("Rewriting");
    }

    // Apply \lxDeclare declarations: set roles/names/meanings on matching XMTok elements.
    // Must run BEFORE math parsing so the parser sees the updated roles.
    apply_lx_declarations(&mut document);

    if !state::get_nomathparse_flag() {
      // Telemetry: count formulae and time the whole Marpa parse pass.
      // Per-formula bucket histogram requires per-call instrumentation
      // inside latexml_math_parser::parser::parse_math; deferred.
      let xmath_count = document.findnodes("//ltx:XMath", None).len() as u32;
      latexml_core::telemetry::set_formulae(xmath_count);
      let _gp = latexml_core::telemetry::phase(latexml_core::telemetry::Phase::MathParse);
      let mut parser = MathParser::default();
      parser.parse_math(&mut document)?;
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
      renumber_math_ids(&mut document);
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

    note_begin("Finalizing");
    document.finalize()?;
    note_end("Finalizing");
    // Perl core produces role="UNKNOWN" for single-letter math tokens.
    // Per-document .latexml files set role="ID" via DefMathRewrite BEFORE parsing.
    // We do NOT apply a blanket conversion — roles are set by rewrite rules only.
    // Cleanup unreferenced xml:ids on XMTok elements generated by the math parser.
    // Must run after finalize (which includes prune_xmduals that may transfer ids).
    document.cleanup_unreferenced_xmtok_ids();
    Ok(document)
  }

  fn digest_internal(&mut self) -> Result<Digested> {
    let mut boxes = Vec::new();
    while gullet::has_more_input() {
      // Perl finishDigestion L219-220: loop consuming input even after errors.
      // Catch Fatal errors (TooManyErrors, etc.) so we can still produce partial output.
      match stomach::digest_next_body(None) {
        Ok(next_bodies) => boxes.extend(next_bodies),
        Err(e) => {
          log::warn!("digest_internal: error during recovery digestion: {:?}", e);
          break;
        },
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
      if !pathname::is_literaldata(&request) {
        state::assign_value("SOURCEFILE", request.clone(), None);
      }
      if !dir.is_empty() {
        state::assign_value("SOURCEDIRECTORY", dir.clone(), None);
      }
      state::search_paths_push_front(dir.clone());
      // Perl Core.pm L200:
      //   $state->unshiftValue(GRAPHICSPATHS => $dir)
      //     if !grep { $_ eq $dir } @{ $state->lookupValue('GRAPHICSPATHS') };
      if !state::graphics_paths_contains(&dir) {
        state::graphics_paths_push_front(dir);
      }
      state::install_definition(
        Stored::Expandable(Rc::new(Expandable {
          cs: T_CS!("\\jobname"),
          paramlist: None,
          expansion: Tokens::new(Explode!(name)).into(),
          ..Expandable::default()
        })),
        None,
      );
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
    // if mode == DigestionMode::BibTeX {
    //   let bib = LaTeXML::Pre::BibTeX->newFromGullet($name, $state->getStomach->getGullet);
    //   LaTeXML::Package::InputContent("literal:" . $bib->toTeX);
    // }

    let list = self.digest_internal()?;
    note_end(&s!("Digesting {} {}", mode, name));
    Ok(list)
  }
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
    let pat = latexml_package::package::latexml_sty::compile_declare_pattern_pub(&match_str);
    if pat.xpath.is_empty() {
      continue; // Unrecognized pattern, skip
    }

    // Add declare metadata for Rust-side filtering
    attrs.insert("_declare_type".to_string(), pat.pattern_type.to_string());
    if let Some(ref b) = pat.base_text {
      attrs.insert("_declare_base".to_string(), b.clone());
    }
    if let Some(ref s) = pat.sub_text {
      attrs.insert("_declare_sub".to_string(), s.clone());
    }
    if let Some(ref a) = pat.accent_name {
      attrs.insert("_declare_accent".to_string(), a.clone());
    }

    // For math mode, append visibility check to XPath
    let xpath = format!("{}[@_pvis and @_cvis]", pat.xpath);

    // Determine select_count based on pattern type
    let select_count = match pat.pattern_type {
      "literal_subscript" | "prime" | "subscript" => Some(2usize),
      "accent" => Some(1usize),
      _ => Some(1usize),
    };

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
        wildcard_paths: pat.wildcard_paths,
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
fn apply_lx_declarations(document: &mut Document) {
  let decls_str = match state::lookup_value("LATEXML_DECLARATIONS") {
    Some(Stored::String(s)) => arena::with(s, |r| r.to_string()),
    _ => return,
  };
  if decls_str.is_empty() {
    return;
  }

  // Parse declarations: "token_text\trole\tname\tmeaning\tdecl_id" per line
  let declarations: Vec<(&str, &str, &str, &str, &str)> = decls_str
    .lines()
    .filter_map(|line| {
      let parts: Vec<&str> = line.splitn(5, '\t').collect();
      if parts.len() >= 4 {
        Some((
          parts[0],
          parts[1],
          parts[2],
          parts[3],
          *parts.get(4).unwrap_or(&""),
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
      scope
    };

    for &(pattern, role, name, meaning, decl_id) in &declarations {
      // Match by content text, or by XMTok name attribute (for CS patterns like \circ)
      let matches = content == pattern
        || (!tok_name.is_empty() && pattern.starts_with('\\') && pattern[1..] == tok_name);
      if matches {
        // Check scope: if decl_id has a section prefix (e.g. "S1" from "S1.XMD1"),
        // only apply to tokens within that section
        if !decl_id.is_empty() {
          if let Some(section_prefix) = decl_id.split('.').next() {
            if !section_prefix.is_empty() && !tok_scope.is_empty() && tok_scope != section_prefix {
              continue; // Wrong section — skip this declaration
            }
          }
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
  let mut id_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
  let mut referenced_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

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
        if let Some(new_id) = id_map.get(&old_id) {
          if new_id != &old_id {
            document.unrecord_id(&old_id);
            let _ = node.remove_attribute("xml:id");
            let _ = node.remove_attribute_ns("id", xml_ns);
            nodes_to_update.push((node, new_id.clone()));
          }
        }
      }
      for (mut node, new_id) in nodes_to_update {
        let _ = document.set_attribute(&mut node, "xml:id", &new_id);
      }

      // Update idrefs
      for (mut node, old_idref) in idref_entries.drain(..) {
        if let Some(new_idref) = id_map.get(&old_idref) {
          if new_idref != &old_idref {
            let _ = node.set_attribute("idref", new_idref);
          }
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
  use super::parse_preload_spec;

  fn opts(v: &[&str]) -> Vec<String> { v.iter().map(|s| s.to_string()).collect() }

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
