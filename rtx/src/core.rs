use regex::{Captures, Regex};
use std::path::Path;
use std::rc::Rc;

use rtx_core::common::error::*;
use rtx_core::common::DigestionMode;
use rtx_core::definition::expandable::Expandable;
use rtx_core::document::Document;
use rtx_core::list::List;
use rtx_core::state::{Scope, Stored}; // State
use rtx_core::util::pathname;
use rtx_core::util::pathname::FindOptions;
use rtx_core::{Core, Digested};

use crate::math_parser::MathParser;
use crate::package::*;

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
  fn load_preamble(&mut self, preamble: String) {}
  fn load_postamble(&mut self, preamble: String) {}
}

impl DigestionAPI for Core {
  fn initialize_state(&mut self, preloads: Vec<String>) -> Result<()> {
    self.state.initialize_stomach();
    // let paths = state.lookup_value("SEARCHPATHS");
    self
      .state
      .assign_value("InitialPreloads", true, Some(Scope::Global));
    for preload in preloads {
      input_definitions(&preload, InputDefinitionOptions::default(), &mut self.state)?;
    }
    self
      .state
      .assign_value("InitialPreloads", false, Some(Scope::Global));
    Ok(())
  }

  fn digest(
    &mut self,
    request: String,
    _preamble: Option<String>,
    _postamble: Option<String>,
    _mode: Option<DigestionMode>,
    _no_init: bool,
  ) -> Result<Digested>
  {
    // let mut ext = match mode {
    //   Some(m) => Some(m.extension()),
    //   None => Some(DigestionMode::TeX.extension()),
    // };
    // let mut dir = None;
    let name = if pathname::is_literaldata(&request) {
      Some(s!("Anonymous String"))
    } else if pathname::is_url(&request) {
      Some(request.clone())
    } else {
      let path = Path::new(&request);
      // ext = match path.extension() {
      //   Some(pe) => Some(pe.to_str().unwrap().to_string()),
      //   None => None,
      // };
      // dir = path.parent();
      match path.file_stem() {
        None => Some(s!("missing_name")),
        Some(pf) => Some(pf.to_str().unwrap().to_string()),
      }
    };
    // else {
    //   $self->withState(sub {
    //       Fatal('missing_file', $request, undef, "Can't find $mode file $request"); }); } }
    // };
    let digestion_note = s!("Digesting {}", &name.clone().unwrap());
    note_begin(&digestion_note);
    // $self->initializeState($mode . ".pool", @{ $$self{preload} || [] }) unless
    // $options{noinitialize}; $state->assignValue(SOURCEFILE      => $request) if
    // (!pathname::is_literaldata($request)); $state->assignValue(SOURCEDIRECTORY => $dir)
    // if defined $dir; $state->unshiftValue(SEARCHPATHS => $dir)
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

    let mut state = &mut self.state;
    let search_paths = match state.lookup_value("SEARCHPATHS") {
      Some(&Stored::VecString(ref paths)) => Some(paths.clone()),
      _ => None,
    };
    // Compile-time load of model AND indirect model
    load_model!(&mut state, "LaTeXML");
    // Was:
    // state.model.load_schema(search_paths.clone()); // If needed?

    let mut document = Document::new();
    if search_paths.is_none() || !search_paths.as_ref().unwrap().is_empty() {
      {
        if let Some(&Stored::Bool(ico_flag)) = state.lookup_value("INCLUDE_COMMENTS") {
          if ico_flag {
            let paths_string = search_paths.as_ref().unwrap().join(",");
            let attributes = map!{s!("paths") => paths_string};
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
        let attributes = map!{s!("class") => preload.to_string()};
        document.insert_pi("latexml", Some(attributes))?;
      } else {
        STY_EXT_REGEX.replace_all(preload, "");
        let attributes = map!{s!("package") => preload.to_string()};
        document.insert_pi("latexml", Some(attributes))?;
      }
    }
    document.absorb(digested, state)?;
    note_end("Building");

    // if (my $rules = $state->lookupValue('DOCUMENT_REWRITE_RULES')) {
    //   NoteBegin("Rewriting");
    //   $document->markXMNodeVisibility;
    //   foreach my $rule (@$rules) {
    //     $rule->rewrite($document, $document->getDocument->documentElement); }
    //   NoteEnd("Rewriting"); }

    if !state.nomathparse {
      let mut parser = MathParser::default();
      parser.parse_math(&mut document, state)?;
    }
    note_begin("Finalizing");
    document.finalize(&mut state)?;
    note_end("Finalizing");
    Ok(document)
  }

  fn digest_internal(&mut self) -> Result<Digested> {
    let mut boxes = Vec::new();
    let mut state = &mut self.state;

    while self.stomach.borrow().get_gullet().has_more_input() {
      let mut next_bodies: Vec<Digested> =
        self.stomach.borrow_mut().digest_next_body(false, state)?;
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
    let mut name = String::new();
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
        FindOptions {
          types: Some(vec![mode.extension(), String::new()]),
          ..FindOptions::default()
        },
      ) {
        request = pathname;
        dir = pathname::directory(&request);
        name = pathname::file_name(&request);
      // ext = pathname::extension(&request);
      } else {
        error!(
          target: &s!("Fatal:missing_file:{}", request_base),
          "Can't find {} file {} ",
          mode,
          request
        );
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
      self
        .state
        .assign_value("SOURCEDIRECTORY", dir.clone(), None);
    }
    self.state.search_paths.push_front(dir.clone());
    self.state.graphics_paths.push_front(dir.clone());

    let name_copy = name.clone();
    self.state.install_definition(
      Stored::Expandable(Rc::new(Expandable {
        cs: T_CS!("\\jobname"),
        paramlist: None,
        expansion: SimpleExpansion!(Tokens::new(Explode!(name_copy))),
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
