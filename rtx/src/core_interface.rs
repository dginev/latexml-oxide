use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use rustc_hash::FxHashMap as HashMap;
use std::path::Path;
use std::rc::Rc;

use rtx_core::common::arena;
use rtx_core::common::error::{self, note_begin, note_end, Result};
use rtx_core::common::DigestionMode;
use rtx_core::common::store::Stored;
use rtx_core::common::model;
use rtx_core::definition::expandable::Expandable;
use rtx_core::digested::Digested;
use rtx_core::document::Document;
use rtx_core::list::List;
use rtx_core::stomach;
use rtx_core::gullet;
use rtx_core::state::{self,Scope};
use rtx_core::token::{Catcode, Token};
use rtx_core::tokens::Tokens;
use rtx_core::util::pathname;
use rtx_core::util::pathname::PathnameFindOptions;
// TODO: Clean up these imports -- what belongs where?
use rtx_core::{fatal, map, s, CharToken, Core, Debug, Explode, Token, T_CS, T_SPACE,model};
use rtx_codegen::LoadModel;
use rtx_math_parser::MathParser;
use rtx_package::{
  input_content, input_definitions, load_model, InputDefinitionOptions, InputOptions,
};

static CLS_EXT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.cls$").unwrap());
static STY_EXT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\.sty$").unwrap());
static LATEX_OPTION_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\[([^\]]*)\]").unwrap());

#[derive(Default)]
pub struct DigestionOptions {
  pub mode: Option<DigestionMode>,
  pub noinitialize: Option<bool>,
  pub preamble: Option<String>,
  pub postamble: Option<String>,
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
    // first, reset the error REPORT singleton
    error::initialize_report();
    // now handle conversion state
    gullet::initialize_gullet();
    stomach::initialize_stomach();
    // should we reset the model also?
    model::initialize_model();
    // let paths = state::search_paths;
    state::assign_value("InitialPreloads", true, Some(Scope::Global));
    for preload in preloads {
      input_definitions(
        &preload,
        InputDefinitionOptions::default())?;
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
    //   $self->withstate::sub {
    //       Fatal('missing_file', $request, undef, "Can't find $mode file $request"); }); } }
    // };
    let digestion_note = s!("Digesting {}", name);
    note_begin(&digestion_note);
    // $self->initializestate::$mode . ".pool", @{ $$self{preload} || [] }) unless
    // $options{noinitialize}; $state::>assignValue(SOURCEFILE      => $request) if
    // (!pathname::is_literaldata($request));
    if let Some(dir) = dir_opt {
      let dir = dir.to_str().unwrap_or(".");
      {

        state::assign_value("SOURCEDIRECTORY", arena::pin(dir), None);
        state::add_search_path(dir.to_string());
      }
    }
    //   if defined $dir && !grep { $_ eq $dir } @{ $state::>lookupValue('SEARCHPATHS') };
    // $state::>unshiftValue(GRAPHICSPATHS => $dir)

    // if defined $dir && !grep { $_ eq $dir } @{ $state::>lookupValue('GRAPHICSPATHS') };

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
    input_content(
      &request,
      InputOptions::default())?;
    // $self->loadPreamble($options{preamble}) if $options{preamble};

    // // Now for the Hacky part for BibTeX!!!
    // if ($mode eq 'BibTeX') {
    //   my $bib = LaTeXML::Pre::BibTeX->newFromGullet($name, $state::>getStomach->getGullet);
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
      let default_model_load = match model!().schema_data {
        None => true,
        Some(ref v) => v.last() == Some(&arena::pin_static("LaTeXML")),
      };
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
            let paths_string = state::with_search_paths(|paths| paths
              .iter()
              .map(String::as_str)
              .collect::<Vec<&str>>()
              .join(","));
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

    let has_rewrites = state::has_value("DOCUMENT_REWRITE_RULES");
    if has_rewrites {
      note_begin("Rewriting");
      document.mark_xmnode_visibility()?;
      document.load_labels_for_rewrite()?;
      // TODO: What is the right way to do rewrites in a daemon-safe manner?
      if let Some(Stored::VecDequeStored(rules)) = state::remove_value("DOCUMENT_REWRITE_RULES") {
        if let Some(root) = document.get_document().get_root_element() {
          // Step 1: copy the rules locally through Rc, to be able to invoke them with mutable
          // (TODO: obviously, this could be avoided if they never needed mutable
          // When do they?)
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

    if ! state::get_nomathparse_flag() {
      let mut parser = MathParser::default();
      parser.parse_math(&mut document)?;
    }
    note_begin("Finalizing");
    document.finalize()?;
    note_end("Finalizing");
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
  //    noinitialize : if defined, it does not initialize state::
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

      if let Some(pathname) = pathname::find(
        &request_base,
        PathnameFindOptions {
          extensions: Some(vec![mode.extension(), String::new()]),
          ..PathnameFindOptions::default()
        },
      ) {
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
      input_content(
        &request,
        InputOptions::default())?;
    }

    if let Some(preamble) = options.preamble {
      self.load_preamble(preamble);
    }

    // Now for the Hacky part for BibTeX!!!
    // if mode == DigestionMode::BibTeX {
    //   let bib = LaTeXML::Pre::BibTeX->newFromGullet($name, $state::>getStomach->getGullet);
    //   LaTeXML::Package::InputContent("literal:" . $bib->toTeX);
    // }

    let list = self.digest_internal()?;
    note_end(&s!("Digesting {} {}", mode, name));
    Ok(list)
  }
}
