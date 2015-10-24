pub mod stomach;
pub mod gullet;
pub mod mouth;
pub mod token;
pub mod package;

use std::path::Path;
use common::{Error, DigestionMode};
use util::pathname::*;
// use core::token;
use state::{State};
use core::stomach::{Stomach};
use core::package::*;

pub struct Core {
  pub state : State,
}
pub struct Digested {
  pub stuff : Option<Vec<String>>,
}

impl Digested {
  pub fn to_string(&self) -> String {
    match self.stuff.clone() {
      Some(s) => "some".to_string(),
      None => String::new()
    }
  }
  pub fn stringify(&self) -> String {
    match self.stuff.clone() {
      Some(s) => "some".to_string(),
      None => String::new()
    }
  }
}

impl Default for Core {
  fn default() -> Self {
    Core {
      state : State {
        stomach : Stomach::default(),
        verbosity : 0,
        status_code: 0,
        map : Vec::new()
      }
    }
  }
}

impl Core {
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

  pub fn convert_document(&self, digested : Digested) -> Result<String, Error> {
    Ok(digested.to_string())
  }

  pub fn digest_internal(&mut self) -> Digested {
    let mut stuff = Vec::new();
    let stomach : &mut Stomach = self.state.get_stomach();
    while stomach.get_gullet().get_mouth().has_more_input() {
      stuff.push(stomach.digest_next_body());
    }
    stomach.get_gullet().flush();
    return Digested {
      stuff : Some(stuff)
    }
  }

  // Internal helpers:

}