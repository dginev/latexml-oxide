pub mod stomach;
pub mod gullet;
pub mod mouth;
pub mod token;
pub mod package;
pub mod document;

use std::path::Path;
use common::{Error, DigestionMode};
use common::model::{Model};
use common::error::*;
use util::pathname::*;
// use core::token;
use state::{State};
use core::stomach::{Stomach};
use core::document::{Document};
use core::package::*;

pub struct Core<'core> {
  pub state : State<'core>,
  preload : Vec<&'core str>,
}
pub struct Digested {
  pub stuff : Option<Vec<String>>,
}

impl Digested {
  pub fn to_string(&self) -> String {
    match self.stuff.clone() {
      Some(s) => "Digested.to_string()".to_string(),
      None => String::new()
    }
  }
  pub fn stringify(&self) -> String {
    match self.stuff.clone() {
      Some(s) => "Digested.stringify()".to_string(),
      None => String::new()
    }
  }
}

impl<'core> Default for Core<'core> {
  fn default() -> Self {
    Core {
      preload : Vec::new(),
      state : State::default()
    }
  }
}

impl<'core> Core<'core> {
  pub fn digest(&mut self, request : String,
    preamble : Option<String>, postamble : Option<String>, mode : Option<DigestionMode>, no_init : bool) 
    -> Result<Digested, Error> {
     
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
        None => None,
        Some(pf) => Some(pf.to_str().unwrap().to_string())
      }
    };
    // else {
    //   $self->withState(sub {
    //       Fatal('missing_file', $request, undef, "Can't find $mode file $request"); }); } }
    // };
    // NoteBegin("Digesting $mode $name");
      // $self->initializeState($mode . ".pool", @{ $$self{preload} || [] }) unless $options{noinitialize};
      // $state->assignValue(SOURCEFILE      => $request) if (!pathname_is_literaldata($request));
      // $state->assignValue(SOURCEDIRECTORY => $dir)     if defined $dir;
      // $state->unshiftValue(SEARCHPATHS => $dir)
      //   if defined $dir && !grep { $_ eq $dir } @{ $state->lookupValue('SEARCHPATHS') };
      // $state->unshiftValue(GRAPHICSPATHS => $dir)

      //   if defined $dir && !grep { $_ eq $dir } @{ $state->lookupValue('GRAPHICSPATHS') };

      // $state->installDefinition(LaTeXML::Core::Definition::Expandable->new(T_CS('\jobname'), undef,
      //     Tokens(Explode($name))));
      // # Reverse order, since last opened is first read!
      // $self->loadPostamble($options{postamble}) if $options{postamble};
      package::input_content(&mut self.state,request.clone());
      // $self->loadPreamble($options{preamble}) if $options{preamble};

      // # Now for the Hacky part for BibTeX!!!
      // if ($mode eq 'BibTeX') {
      //   my $bib = LaTeXML::Pre::BibTeX->newFromGullet($name, $state->getStomach->getGullet);
      //   LaTeXML::Package::InputContent("literal:" . $bib->toTeX); }
      // my $list = $self->finishDigestion;
      let list = self.digest_internal();
      // NoteEnd("Digesting $mode $name");
      // return $list; }); 
    Ok(list)
  }

  pub fn convert_document<'convert>(&'convert mut self, digested : Digested) -> Result<Document, Error> {
    note_begin("Building");
    {
      self.state.model.load_schema(); // If needed?
    }
    let mut document = {Document::new(&mut self.state.model)};
    let paths_opt : Option<Box<Vec<String>>> = {self.state.lookup_value("SEARCHPATHS")};
    match paths_opt {
      None => {},
      Some(paths) => if !paths.is_empty() {
        match self.state.lookup_value("INCLUDE_COMMENTS") {
          Some(ico_flag) => if *ico_flag {
            document.insert_pi("latexml", *paths); },
          None => {} 
        };
      }
    };
    for preload in self.preload.iter() {
      // TODO
      // next if $preload =~ /\.pool$/;
      // my $options = undef;                                 # Stupid perlcritic policy
      // if ($preload =~ s/^\[([^\]]*)\]//) { $options = $1; }
      // if ($preload =~ s/\.cls$//) {
      //   $document->insertPI('latexml', class => $preload, ($options ? (options => $options) : ())); }
      // else {
      //   $preload =~ s/\.sty$//;
      //   $document->insertPI('latexml', package => $preload, ($options ? (options => $options) : ())); } }
    }
    document.absorb(digested);
    note_end("Building");

    // if (my $rules = $state->lookupValue('DOCUMENT_REWRITE_RULES')) {
    //   NoteBegin("Rewriting");
    //   $document->markXMNodeVisibility;
    //   foreach my $rule (@$rules) {
    //     $rule->rewrite($document, $document->getDocument->documentElement); }
    //   NoteEnd("Rewriting"); }

    // LaTeXML::MathParser->new()->parseMath($document) unless $$self{nomathparse};
    note_begin("Finalizing");
    document.finalize(&mut self.state);
    note_end("Finalizing");
    return Ok(document)
  }

  pub fn digest_internal<'digest>(&'digest mut self) -> Digested {
    let mut stuff = Vec::new();
    let state = self.state;
    let stomach : &'digest mut Stomach = state.get_stomach();
    while stomach.get_gullet().has_more_input() {
      stuff.push(stomach.digest_next_body());
    }
    stomach.get_gullet().flush();
    return Digested {
      stuff : Some(stuff)
    }
  }

  // Internal helpers:

}