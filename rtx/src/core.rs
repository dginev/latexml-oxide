use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::borrow::Cow;
use std::path::Path;
use std::rc::Rc;

use rtx_core::common::error::{note_begin, note_end, Result};
use rtx_core::common::DigestionMode;
use rtx_core::definition::expandable::Expandable;
use rtx_core::document::Document;
use rtx_core::list::List;
use rtx_core::state::{Scope, Stored}; // State
use rtx_core::token::{Catcode, Token};
use rtx_core::tokens::Tokens;
use rtx_core::util::pathname;
use rtx_core::util::pathname::PathnameFindOptions;
// TODO: Clean up these imports -- what belongs where?
use rtx_core::{fatal, map, s, Core, Debug, Digested, Explode, T_CS, T_OTHER, T_SPACE};

use rtx_codegen::LoadModel;
use rtx_math_parser::MathParser;
use rtx_package::{input_content, input_definitions, load_model, InputDefinitionOptions};

lazy_static! {
  static ref CLS_EXT_REGEX: Regex = Regex::new(r"\.cls$").unwrap();
  static ref STY_EXT_REGEX: Regex = Regex::new(r"\.sty$").unwrap();
  static ref LATEX_OPTION_REGEX: Regex = Regex::new(r"^\[([^\]]*)\]").unwrap();
}

#[derive(Default)]
pub struct DigestionOptions {
  pub mode: Option<DigestionMode>,
  pub noinitialize: Option<bool>,
  pub preamble: Option<String>,
  pub postamble: Option<String>,
}

pub trait DigestionAPI {
  fn initialize_state(&mut self, preloads: Vec<String>) -> Result<()>;
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
  fn initialize_state(&mut self, preloads: Vec<String>) -> Result<()> {
    self.state.initialize_stomach();
    // let paths = state.search_paths;
    self.state.assign_value("InitialPreloads", true, Some(Scope::Global));
    for preload in preloads {
      input_definitions(
        &preload,
        InputDefinitionOptions::default(),
        &mut self.stomach.borrow_mut(),
        &mut self.state,
      )?;
    }
    self.state.assign_value("InitialPreloads", false, Some(Scope::Global));
    Ok(())
  }

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
      Some(s!("Anonymous String"))
    } else if pathname::is_url(&request) {
      Some(request.clone())
    } else {
      let path = Path::new(&request);
      // _ext = match path.extension() {
      //   Some(pe) => Some(pe.to_str().unwrap().to_string()),
      //   None => None,
      // };
      dir_opt = path.parent();
      match path.file_stem() {
        None => Some(s!("missing_name")),
        Some(pf) => Some(pf.to_str().unwrap().to_string()),
      }
    };
    // else {
    //   $self->withState(sub {
    //       Fatal('missing_file', $request, undef, "Can't find $mode file $request"); }); } }
    // };
    let digestion_note = s!("Digesting {}", name.as_ref().unwrap());
    note_begin(&digestion_note);
    // $self->initializeState($mode . ".pool", @{ $$self{preload} || [] }) unless
    // $options{noinitialize}; $state->assignValue(SOURCEFILE      => $request) if
    // (!pathname::is_literaldata($request));
    if let Some(dir) = dir_opt {
      let dir = dir.to_str().unwrap_or(".");
      self.state.assign_value("SOURCEDIRECTORY", dir, None);
      self.state.search_paths.push_front(dir.to_string());
    }
    //   if defined $dir && !grep { $_ eq $dir } @{ $state->lookupValue('SEARCHPATHS') };
    // $state->unshiftValue(GRAPHICSPATHS => $dir)

    // if defined $dir && !grep { $_ eq $dir } @{ $state->lookupValue('GRAPHICSPATHS') };

    // $state->installDefinition(LaTeXML::Definition::Expandable->new(T_CS!('\jobname'), undef,
    //     Tokens(Explode($name))));
    // // Reverse order, since last opened is first read!

    // $self->loadPostamble($options{postamble}) if $options{postamble};
    input_content(self, &request)?;
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

  fn convert_document(&mut self, digested: Digested) -> Result<Document> {
    note_begin("Building");

    let state = &mut self.state;
    let schema_paths = state.search_paths.iter().map(String::as_str).collect::<Vec<&str>>();
    let default_model_load = match state.model.schema_data {
      None => true,
      Some(ref v) => v.last() == Some(&String::from("LaTeXML")),
    };
    if default_model_load {
      // Compile-time load of model AND indirect model
      load_model!(state, "LaTeXML");
    } else {
      // Eager-load at runtime
      state.model.load_schema(schema_paths.as_slice()); // If needed?
    }

    let mut document = Document::new();
    if !state.search_paths.is_empty() {
      {
        if let Some(&Stored::Bool(ico_flag)) = state.lookup_value("INCLUDE_COMMENTS") {
          if ico_flag {
            let paths_string = state.search_paths.iter().map(String::as_str).collect::<Vec<&str>>().join(",");
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
    document.absorb(digested, state)?;
    note_end("Building");

    let has_rewrites = state.lookup_value("DOCUMENT_REWRITE_RULES").is_some();
    if has_rewrites {
      note_begin("Rewriting");
      document.mark_xmnode_visibility(state)?;
      document.load_labels_for_rewrite(state);
      // TODO: What is the right way to do rewrites in a daemon-safe manner?
      if let Some(Stored::VecDequeStored(rules)) = state.remove_value("DOCUMENT_REWRITE_RULES") {
        let root = document.get_document().get_root_element().unwrap();
        // Step 1: copy the rules locally through Rc, to be able to invoke them with mutable state.
        // (TODO: obviously, this could be avoided if they never needed mutable state. When do they?)
        let mut rewrites = Vec::new();
        for rule in rules {
          if let Stored::Rewrite(mut rewrite_rule) = rule {
            rewrite_rule.compile_clauses(&mut document);
            rewrites.push(rewrite_rule); // clone the Rc
          }
        }
        // Step 2: invoke the rewrite rules
        for mut rewrite_rule in rewrites {
          rewrite_rule.invoke(&mut document, &root, state)?;
        }
      }
      note_end("Rewriting");
    }

    if !state.nomathparse {
      let mut parser = MathParser::default();
      parser.parse_math(&mut document, state)?;
    }
    note_begin("Finalizing");
    document.finalize(state)?;
    note_end("Finalizing");
    Ok(document)
  }

  fn digest_internal(&mut self) -> Result<Digested> {
    let mut boxes = Vec::new();
    let state = &mut self.state;

    while self.stomach.borrow_mut().get_gullet_mut().has_more_input() {
      let next_bodies: Vec<Digested> = self.stomach.borrow_mut().digest_next_body(None, state)?;
      // info!(target:"core:digest_next_body", "\n{:?}\n----\n",next_bodies);
      boxes.extend(next_bodies);
    }
    self.stomach.borrow_mut().get_gullet_mut().flush(state);
    List::new(boxes).into()
  }

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Mid-level API.

  // options are currently being evolved to accomodate the Daemon:
  //    mode  : the processing mode, ie the pool to preload: TeX or BibTeX
  //    noinitialize : if defined, it does not initialize State.
  //    preamble = names a tex file (or standard_preamble.tex)
  //    postamble = names a tex file (or standard_postamble.tex)

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
        request.to_string()
      };

      if let Some(pathname) = pathname::find(
        &request_base,
        PathnameFindOptions {
          types: Some(vec![mode.extension(), String::new()]),
          ..PathnameFindOptions::default()
        },
      ) {
        request = pathname;
        dir = pathname::directory(&request);
        name = pathname::file_name(&request);
      // ext = pathname::extension(&request);
      } else {
        let message = s!("Can't find {} file {} ", mode, request);
        fatal!(Core, MissingFile, self, None, message);
      }
    }

    note_begin(&s!("Digesting {} {}", mode, name));
    let main_pool = s!("{}.pool", mode);
    let noinitialize = options.noinitialize.unwrap_or(false);
    if !noinitialize {
      let mut preloads = vec![main_pool];
      preloads.extend(self.preload.clone());
      self.initialize_state(preloads)?;
    }

    if !pathname::is_literaldata(&request) {
      self.state.assign_value("SOURCEFILE", request.clone(), None);
    }
    if !dir.is_empty() {
      self.state.assign_value("SOURCEDIRECTORY", dir.clone(), None);
    }
    self.state.search_paths.push_front(dir.clone());
    self.state.graphics_paths.push_front(dir);

    let name_copy = name.clone();
    self.state.install_definition(
      Stored::Expandable(Rc::new(Expandable {
        cs: T_CS!("\\jobname"),
        paramlist: None,
        expansion: Tokens::new(Explode!(name_copy)).into(),
        ..Expandable::default()
      })),
      None,
    );

    // Reverse order, since last opened is first read!
    if let Some(postamble) = options.postamble {
      self.load_postamble(postamble);
    }

    input_content(self, &request)?;

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
