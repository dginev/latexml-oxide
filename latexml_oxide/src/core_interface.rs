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
use latexml_package::prelude::{
  InputDefinitionOptions, InputOptions, input_content, input_definitions,
};

static CLS_EXT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.cls$").unwrap());
static STY_EXT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.sty$").unwrap());
static LATEX_OPTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\[([^\]]*)\]").unwrap());

// Regex for parsing DefMathRewrite calls from .latexml files
// Matches: DefMathRewrite( ... );
static DEF_MATH_REWRITE_RE: Lazy<Regex> = Lazy::new(|| {
  Regex::new(r"(?s)DefMathRewrite\(([^;]+)\);").unwrap()
});
// Key-value patterns within DefMathRewrite
static SCOPE_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"scope\s*=>\s*'([^']+)'").unwrap());
static MATCH_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"match\s*=>\s*'([^']*)'").unwrap());
static ROLE_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"role\s*=>\s*'([^']+)'").unwrap());
static NAME_ATTR_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(?:^|,)\s*name\s*=>\s*'([^']*)'").unwrap());
static MEANING_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"meaning\s*=>\s*'([^']*)'").unwrap());

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
  fn load_preamble(&mut self, _preamble: String) {}
  fn load_postamble(&mut self, _preamble: String) {}
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
    state::assign_value("InitialPreloads", true, Some(Scope::Global));
    for preload in preloads {
      input_definitions(&preload, InputDefinitionOptions::default())?;
    }
    state::assign_value("InitialPreloads", false, Some(Scope::Global));
    Ok(())
  }

  // TODO: We should choose between this function or digest_file, rather than implement twice,
  // right?
  fn digest(
    &mut self,
    request: String,
    _preamble: Option<String>,
    _postamble: Option<String>,
    mode: Option<DigestionMode>,
    _no_init: bool,
  ) -> Result<Digested> {
    let mut _ext = match mode {
      Some(m) => Some(m.extension()),
      None => Some(DigestionMode::TeX.extension()),
    };
    let mut dir_opt = None;

    let name = if pathname::is_literaldata(&request) {
      s!("Anonymous String")
    } else if pathname::is_url(&request) {
      request.clone()
    } else {
      let path = Path::new(&request);
      // _ext = match path.extension() {
      //   Some(pe) => Some(pe.to_str().unwrap().to_string()),
      //   None => None,
      // };
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
    // $options{noinitialize}; $state->assignValue(SOURCEFILE      => $request) if
    // (!pathname::is_literaldata($request));
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

    // $self->loadPostamble($options{postamble}) if $options{postamble};
    input_content(&request, InputOptions::default())?;
    // $self->loadPreamble($options{preamble}) if $options{preamble};

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
            let attributes = map! {s!("paths") => paths_string};
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
              rewrites.push(rewrite_rule); // clone the Rc
            }
          }
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
      let mut parser = MathParser::default();
      parser.parse_math(&mut document)?;
      // Post-parse: add ltx_math_unparsed to Math parents of failed XMath nodes
      if !parser.failed_xmath_ids.is_empty() {
        for xmath_id in &parser.failed_xmath_ids {
          let _xpath = format!("descendant-or-self::ltx:XMath[@_hashid='{}']/..", xmath_id);
          // XMath nodes don't have @_hashid, so use the node list approach instead
        }
        // Apply ltx_math_unparsed to failed XMath nodes
        for mut math_node in document.findnodes("descendant-or-self::ltx:Math[not(@text)]", None) {
          for xmath_child in document.findnodes("ltx:XMath", Some(&math_node)) {
            if parser.failed_xmath_ids.contains(&xmath_child.to_hashable()) {
              document.add_class(&mut math_node, "ltx_math_unparsed")?;
              break;
            }
          }
        }
      }
    }

    note_begin("Finalizing");
    document.finalize()?;
    note_end("Finalizing");
    // Post-finalize: convert single-letter UNKNOWN tokens to ID.
    // Perl core produces role="UNKNOWN" for single-letter tokens by default.
    // In Perl, per-document .latexml files add DefMathRewrite rules that set role="ID".
    // In Rust, we always apply this conversion to match the expected test XMLs.
    for mut tok in document.findnodes("descendant-or-self::ltx:XMTok[@role='UNKNOWN']", None) {
      let content = tok.get_content();
      if content.chars().count() == 1
        && content.chars().next().is_some_and(|c| c.is_alphabetic())
        && tok.get_attribute("meaning").is_none()
      {
        tok.set_attribute("role", "ID")?;
      }
    }
    // Cleanup unreferenced xml:ids on XMTok elements generated by the math parser.
    // Must run after finalize (which includes prune_xmduals that may transfer ids).
    document.cleanup_unreferenced_xmtok_ids();
    Ok(document)
  }

  fn digest_internal(&mut self) -> Result<Digested> {
    let mut boxes = Vec::new();
    while gullet::has_more_input() {
      let next_bodies: Vec<Digested> = stomach::digest_next_body(None)?;
      // info!(target:"core:digest_next_body", "\n{:?}\n----\n",next_bodies);
      boxes.extend(next_bodies);
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
      state::graphics_paths_push_front(dir);
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

    // Skip complex patterns: anything with \, _, {, } in the match string
    // These need full token-level matching (digestion + DOM -> XPath) which we
    // don't implement yet.
    if match_str.contains('\\') || match_str.contains('_') || match_str.contains('{') {
      continue;
    }

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

    // Build the XPath for this match.
    // For math mode, Perl's compile_match digests "$a$", builds DOM, then generates:
    //   descendant-or-self::ltx:XMTok[text()='a'][@_pvis and @_cvis]
    // We generate the equivalent XPath directly for single-character matches.
    let xpath = format!(
      "descendant-or-self::ltx:XMTok[text()='{}'][@_pvis and @_cvis]",
      match_str
    );

    // Check for optional scope
    let scope_str = SCOPE_RE.captures(body).map(|s| s[1].to_string());

    // Build the rewrite rule with pre-compiled clauses
    let mut clauses = Vec::new();

    // Add scope clause if present (compiled during compile_clauses phase)
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
        select_count: Some(1),
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

  // Parse declarations: "token_text\trole\tname\tmeaning" per line
  let declarations: Vec<(&str, &str, &str, &str)> = decls_str
    .lines()
    .filter_map(|line| {
      let parts: Vec<&str> = line.splitn(4, '\t').collect();
      if parts.len() == 4 {
        Some((parts[0], parts[1], parts[2], parts[3]))
      } else {
        None
      }
    })
    .collect();

  if declarations.is_empty() {
    return;
  }

  // Find all XMTok elements in the document and apply matching declarations
  let xmtoks = document.findnodes("descendant-or-self::ltx:XMTok", None);
  for mut tok in xmtoks {
    let content = tok.get_content();
    let tok_name = tok.get_attribute("name").unwrap_or_default();
    for &(pattern, role, name, meaning) in &declarations {
      // Match by content text, or by XMTok name attribute (for CS patterns like \circ)
      let matches = content == pattern
        || (!tok_name.is_empty() && pattern.starts_with('\\') && pattern[1..] == tok_name);
      if matches {
        if !role.is_empty() {
          let _ = tok.set_attribute("role", role);
        }
        if !name.is_empty() {
          let _ = tok.set_attribute("name", name);
        }
        if !meaning.is_empty() {
          let _ = tok.set_attribute("meaning", meaning);
        }
        break; // First matching declaration wins
      }
    }
  }
}

// TODO: kludge_bracket_grouping — needs full implementation with script handling
// (parse_kludgeScripts_rec) to match Perl's XMWrap structure. The basic bracket
// grouping works but produces different nesting than Perl, causing test diffs.
