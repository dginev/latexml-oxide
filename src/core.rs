use regex::{Regex, Captures};
use std::path::Path;
use rtx_core::common::{Error, DigestionMode};
// use common::model::{Model};
use rtx_core::{Core, Digested, TexMode};
use rtx_core::common::error::*;
use rtx_core::util::pathname::*;
use rtx_core::state::{Scope, ObjectStore}; // State
// use rtx_core::stomach::Stomach;
use rtx_core::document::Document;
// use rtx_core::tbox::TBox;
use rtx_core::list::List;
use package::*;


pub trait DigestionAPI {
  fn initialize_state(&mut self, preloads: Vec<String>);
  fn digest(&mut self, request: String, preamble: Option<String>, postamble: Option<String>, mode: Option<DigestionMode>, no_init: bool) -> Result<Digested, Error>;
  fn convert_file(&mut self, filepath: String) -> Result<Document, Error>;
  fn convert_document(&mut self, digested: Digested) -> Result<Document, Error>;
  fn digest_internal(&mut self) -> Digested;
}

impl DigestionAPI for Core {
  fn initialize_state(&mut self, preloads: Vec<String>) {
    self.stomach.initialize(); // The current Stomach;
    // let paths = state.lookup_value("SEARCHPATHS");
    self.state.assign_value("InitialPreloads",
                            ObjectStore::Bool(true),
                            Some(Scope::Global));
    for preload in preloads {
      match input_definitions(preload, InputDefinitionOptions::default(), &mut self.state) {
        Ok(_) => {}
        Err(_) => {} // TODO
      }
    }
    self.state.assign_value("InitialPreloads",
                            ObjectStore::Bool(false),
                            Some(Scope::Global));
  }

  fn digest(&mut self, request: String, _preamble: Option<String>, _postamble: Option<String>, _mode: Option<DigestionMode>, _no_init: bool) -> Result<Digested, Error> {

    // let mut ext = match mode {
    //   Some(m) => Some(m.extension()),
    //   None => Some(DigestionMode::TeX.extension()),
    // };
    // let mut dir = None;
    let name = if pathname_is_literaldata(&request) {
      Some("Anonymous String".to_string())
    } else if pathname_is_url(&request) {
      Some(request.clone())
    } else {
      let path = Path::new(&request);
      // ext = match path.extension() {
      //   Some(pe) => Some(pe.to_str().unwrap().to_string()),
      //   None => None,
      // };
      // dir = path.parent();
      match path.file_stem() {
        None => Some("missing_name".to_string()),
        Some(pf) => Some(pf.to_str().unwrap().to_string()),
      }
    };
    // else {
    //   $self->withState(sub {
    //       Fatal('missing_file', $request, undef, "Can't find $mode file $request"); }); } }
    // };
    let digestion_note = "Digesting ".to_string() + &name.clone().unwrap();
    note_begin(&digestion_note);
    // $self->initializeState($mode . ".pool", @{ $$self{preload} || [] }) unless $options{noinitialize};
    // $state->assignValue(SOURCEFILE      => $request) if (!pathname_is_literaldata($request));
    // $state->assignValue(SOURCEDIRECTORY => $dir)     if defined $dir;
    // $state->unshiftValue(SEARCHPATHS => $dir)
    //   if defined $dir && !grep { $_ eq $dir } @{ $state->lookupValue('SEARCHPATHS') };
    // $state->unshiftValue(GRAPHICSPATHS => $dir)

    // if defined $dir && !grep { $_ eq $dir } @{ $state->lookupValue('GRAPHICSPATHS') };

    // $state->installDefinition(LaTeXML::Definition::Expandable->new(T_CS!('\jobname'), undef,
    //     Tokens(Explode($name))));
    // # Reverse order, since last opened is first read!
    // $self->loadPostamble($options{postamble}) if $options{postamble};
    match input_content(self, request.clone()) {
      Ok(_) => {}
      Err(e) => println_stderr!("Failed to input content: {:?}", e),
    };
    // $self->loadPreamble($options{preamble}) if $options{preamble};

    // # Now for the Hacky part for BibTeX!!!
    // if ($mode eq 'BibTeX') {
    //   my $bib = LaTeXML::Pre::BibTeX->newFromGullet($name, $state->getStomach->getGullet);
    //   LaTeXML::Package::InputContent("literal:" . $bib->toTeX); }

    let list = self.digest_internal();
    note_end(&digestion_note);
    // return $list; });
    Ok(list)
  }

  fn convert_file(&mut self, filepath: String) -> Result<Document, Error> {
    match self.digest(filepath, None, None, None, false) {
      Err(e) => Err(e),
      Ok(digested) => self.convert_document(digested),
    }
  }

  fn convert_document(&mut self, digested: Digested) -> Result<Document, Error> {
    note_begin("Building");

    let mut state = &mut self.state;
    state.model.load_schema(); // If needed?
    let mut document = Document::new();
    {
      if let Some(&ObjectStore::VecString(ref paths)) = state.lookup_value("SEARCHPATHS") {
        if !paths.is_empty() {
          {
            if let Some(&ObjectStore::Bool(ico_flag)) = state.lookup_value("INCLUDE_COMMENTS") {
              if ico_flag {
                let paths_string = paths.join(",");
                document.insert_pi("latexml", vec!["paths".to_string()], vec![paths_string]);
              }
            }
          }
        }
      }
    };
    lazy_static! {
      static ref POOL_EXT_REGEX : Regex = Regex::new(r"\.pool$").unwrap();
      static ref CLS_EXT_REGEX : Regex = Regex::new(r"\.cls$").unwrap();
      static ref STY_EXT_REGEX : Regex = Regex::new(r"\.sty$").unwrap();
      static ref LATEX_OPTION_REGEX : Regex = Regex::new(r"^\[([^\]]*)\]").unwrap();
    }
    for preload in &self.preload {
      if POOL_EXT_REGEX.is_match(preload) {
        continue;
      }
      let mut options: Option<String> = None;
      LATEX_OPTION_REGEX.replace_all(preload, |refs: &Captures| -> String {
        options = Some(refs.at(1).unwrap_or("").to_string());
        String::new()
      });
      if CLS_EXT_REGEX.is_match(preload) {
        CLS_EXT_REGEX.replace_all(preload, "");
        document.insert_pi("latexml", vec!["class".to_string()], vec![preload.to_string()]);
      } else {
        STY_EXT_REGEX.replace_all(preload, "");
        document.insert_pi("latexml", vec!["package".to_string()], vec![preload.to_string()]);
      }
    }
    document.absorb(digested, state);
    note_end("Building");

    // if (my $rules = $state->lookupValue('DOCUMENT_REWRITE_RULES')) {
    //   NoteBegin("Rewriting");
    //   $document->markXMNodeVisibility;
    //   foreach my $rule (@$rules) {
    //     $rule->rewrite($document, $document->getDocument->documentElement); }
    //   NoteEnd("Rewriting"); }

    // LaTeXML::MathParser->new()->parseMath($document) unless $$self{nomathparse};
    note_begin("Finalizing");
    document.finalize(&mut state);
    note_end("Finalizing");
    Ok(document)
  }

  fn digest_internal(&mut self) -> Digested {
    let mut boxes = Vec::new();
    let mut state = &mut self.state;

    while self.stomach.get_gullet().has_more_input() {
      let next_bodies: Vec<Digested> = self.stomach.digest_next_body(false, state);
      for body in next_bodies {
        boxes.push(body);
      }
    }
    self.stomach.get_gullet().flush();
    Digested::ListObj(List { boxes: boxes, mode: TexMode::Text })
  }

  // Internal helpers:
}
