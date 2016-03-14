#[macro_use] pub mod token;
pub mod stomach;
pub mod gullet;
pub mod mouth;
pub mod definition;
pub mod parameter;
pub mod package;
pub mod tbox;
pub mod list;
pub mod document;
pub mod whatsit;

use regex::{Regex, Captures};
use std::path::Path;
use common::{Error, DigestionMode};
// use common::model::{Model};
use common::error::*;
use util::pathname::*;
// use core::token;
use state::{State, Scope};
use core::stomach::{Stomach};
use core::document::{Document};
use core::tbox::TBox;
use core::list::List;
use core::package::*;

pub struct Core {
  pub state : State,
  pub stomach : Stomach,
  pub preload : Vec<String>,
}
pub trait Digested {
  fn unlist(&self) -> Vec<&TBox>;
  fn to_string(&self) -> String {
    "Vec<TBox> for now ".to_string()
  }
  fn stringify(&self) -> String {
    "Vec<TBox> for now ".to_string()
  }
}

impl Default for Core {
  fn default() -> Self {
    Core {
      preload : Vec::new(),
      stomach : Stomach::default(),
      state : State::new()
    }
  }
}

impl Core {
  pub fn initialize_state(&mut self, preloads: Vec<String>) {
    self.stomach.initialize(); // The current Stomach;
    // let paths = state.lookup_value("SEARCHPATHS");
    self.state.assign_value("InitialPreloads", Box::new(true), &Scope::Global);
    for preload in preloads.into_iter() {
      // TODO
      match package::input_definitions(self, preload) {
        Ok(_) => {},
        Err(_) => {}, // TODO
      }
    }
    self.state.assign_value("InitialPreloads", Box::new(false), &Scope::Global);
  }

  pub fn digest(&mut self, request : String,
    preamble : Option<String>, postamble : Option<String>, mode : Option<DigestionMode>, no_init : bool)
    -> Result<Box<Digested>, Error> {

    let mut ext = match mode {
      Some(m) => Some(m.extension()),
      None => Some(DigestionMode::TeX.extension())
    };
    let mut dir = None;
    let name = if pathname_is_literaldata(&request) {
      Some("Anonymous String".to_string())
    } else if pathname_is_url(&request) {
      Some(request.clone())
    } else {
      let path = Path::new(&request);
      ext = match path.extension() {
        Some(pe) => Some(pe.to_str().unwrap().to_string()),
        None => None
      };
      dir = path.parent();
      match path.file_stem() {
        None => Some("missing_name".to_string()),
        Some(pf) => Some(pf.to_str().unwrap().to_string())
      }
    };
    // else {
    //   $self->withState(sub {
    //       Fatal('missing_file', $request, undef, "Can't find $mode file $request"); }); } }
    // };
    note_begin("Digesting ".to_string() +&name.clone().unwrap());
      // $self->initializeState($mode . ".pool", @{ $$self{preload} || [] }) unless $options{noinitialize};
      // $state->assignValue(SOURCEFILE      => $request) if (!pathname_is_literaldata($request));
      // $state->assignValue(SOURCEDIRECTORY => $dir)     if defined $dir;
      // $state->unshiftValue(SEARCHPATHS => $dir)
      //   if defined $dir && !grep { $_ eq $dir } @{ $state->lookupValue('SEARCHPATHS') };
      // $state->unshiftValue(GRAPHICSPATHS => $dir)

      //   if defined $dir && !grep { $_ eq $dir } @{ $state->lookupValue('GRAPHICSPATHS') };

      // $state->installDefinition(LaTeXML::Core::Definition::Expandable->new(T_CS!('\jobname'), undef,
      //     Tokens(Explode($name))));
      // # Reverse order, since last opened is first read!
      // $self->loadPostamble($options{postamble}) if $options{postamble};
      match package::input_content(self, request.clone()) {
        Ok(_) => {},
        Err(e) => println_stderr!("Failed to input content: {:?}", e)
      };
      // $self->loadPreamble($options{preamble}) if $options{preamble};

      // # Now for the Hacky part for BibTeX!!!
      // if ($mode eq 'BibTeX') {
      //   my $bib = LaTeXML::Pre::BibTeX->newFromGullet($name, $state->getStomach->getGullet);
      //   LaTeXML::Package::InputContent("literal:" . $bib->toTeX); }

      let list = self.digest_internal();
      note_end("Digesting ".to_string()+ &name.clone().unwrap());
      // return $list; });
    Ok(list)
  }

  pub fn convert_file<'convert>(&'convert mut self, filepath : String) -> Result<Document, Error> {
    match self.digest(filepath, None, None, None, false) {
      Err(e) => Err(e),
      Ok(digested) => self.convert_document(digested)
    }
  }

  pub fn convert_document<'convert>(&'convert mut self, digested : Box<Digested>) -> Result<Document, Error> {
    note_begin("Building".to_string());

    let mut state = &mut self.state;
    state.model.load_schema(); // If needed?
    let mut document = Document::new();
    let paths_opt : Option<Box<Vec<String>>> = state.lookup_value("SEARCHPATHS");
    match paths_opt {
      None => {},
      Some(paths) => if !paths.is_empty() {
        match state.lookup_value("INCLUDE_COMMENTS") {
          Some(ico_flag) => if *ico_flag {
            let paths_string = paths.join(",");
            document.insert_pi("latexml", "paths", &paths_string, None); },
          None => {}
        };
      }
    };
    let pool_ext_regex = Regex::new(r"\.pool$").unwrap();
    let cls_ext_regex = Regex::new(r"\.cls$").unwrap();
    let sty_ext_regex = Regex::new(r"\.sty$").unwrap();
    let latex_option_regex = Regex::new(r"^\[([^\]]*)\]").unwrap();
    for preload in self.preload.iter() {
      if pool_ext_regex.is_match(preload) {
        continue;
      }
      let mut options : Option<String> = None;
      latex_option_regex.replace_all(preload, |refs: &Captures| -> String {
         options = Some(refs.at(1).unwrap_or("").to_string());
         String::new()
      });
      if cls_ext_regex.is_match(preload) {
        cls_ext_regex.replace_all(preload, "");
        document.insert_pi("latexml", "class", preload, options);
      } else {
        sty_ext_regex.replace_all(preload, "");
        document.insert_pi("latexml", "package", preload, options);
      }
    }
    document.absorb(digested);
    note_end("Building".to_string());

    // if (my $rules = $state->lookupValue('DOCUMENT_REWRITE_RULES')) {
    //   NoteBegin("Rewriting");
    //   $document->markXMNodeVisibility;
    //   foreach my $rule (@$rules) {
    //     $rule->rewrite($document, $document->getDocument->documentElement); }
    //   NoteEnd("Rewriting"); }

    // LaTeXML::MathParser->new()->parseMath($document) unless $$self{nomathparse};
    note_begin("Finalizing".to_string());
    document.finalize(&mut state);
    note_end("Finalizing".to_string());
    return Ok(document)
  }

  pub fn digest_internal(&mut self) -> Box<Digested> {
    let mut boxes = Vec::new();
    let mut state = &mut self.state;

    while self.stomach.get_gullet().has_more_input() {
      for body in self.stomach.digest_next_body(false, state) {
        boxes.push(body);
      }
    }
    self.stomach.get_gullet().flush();
    Box::new(List { boxes : boxes })
  }

  // Internal helpers:

}