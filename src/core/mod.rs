pub mod token;

use common::{OutputFormat, Error, DigestionMode};
use util::pathname::*;
// use core::token;
use state::{State};

pub struct Core {
  pub state : State,
}
pub struct Digested {
  pub stuff : String,
}

impl Digested {
  pub fn to_string(&self) -> String {
    self.stuff.clone()
  }
  pub fn stringify(&self) -> String {
    self.stuff.clone()
  }
}

impl Default for Core {
  fn default() -> Self {
    Core {
      state : State {
        verbosity : 0,
        status_code: 0,
        map : Vec::new()
      }
    }
  }
}

impl Core {
  pub fn digest(&self, request : String,
    preamble : Option<String>, postamble : Option<String>, mode : Option<DigestionMode>, no_init : bool) 
    -> Result<Digested, Error> {
     
    let mut ext = match mode {
      Some(m) => m.extension(),
      None => DigestionMode::TeX.extension()
    };
    let name = if pathname_is_literaldata(&request) {
      "Anonymous String".to_string() }
    else if pathname_is_url(&request) {
      request.clone()
    } else {
      request.clone()
    };
    // else {
      
    // else {
    //   self.fatal('missing_file', $request, undef, "Can't find $mode file $request"); }); } }
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
      // LaTeXML::Package::InputContent($request);
      // $self->loadPreamble($options{preamble}) if $options{preamble};

      // # Now for the Hacky part for BibTeX!!!
      // if ($mode eq 'BibTeX') {
      //   my $bib = LaTeXML::Pre::BibTeX->newFromGullet($name, $state->getStomach->getGullet);
      //   LaTeXML::Package::InputContent("literal:" . $bib->toTeX); }
      // my $list = $self->finishDigestion;
      // NoteEnd("Digesting $mode $name");
      // return $list; }); 
    Ok(Digested{ stuff: request})
  }

  pub fn convert_document(&self, digested : Digested) -> Result<String, Error> {
    Ok(digested.stuff)
  }

  // Internal helpers:

}