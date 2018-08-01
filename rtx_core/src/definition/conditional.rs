use common::error::*;
use common::object::Object;
use definition::{BeforeDigestClosure, ConditionalClosure, Definition, DigestionClosure};
use document::Document;
use gullet::Gullet;
use parameter::Parameters;
use state::Scope;
use state::State;
use std::rc::Rc;
use stomach::Stomach;
use token::*;
use tokens::Tokens;
use whatsit::Whatsit;
use Digested;

// Conditional control sequences; Expandable
//   Expand enough to determine true/false, then maybe skip
//   record a flag somewhere so that \else or \fi is recognized
//   (otherwise, they should signal an error)
#[derive(Debug, Clone)]
pub enum ConditionalType {
  If,
  Else,
  Or,
  Fi,
}

impl ConditionalType {
  pub fn from(cs: &str) -> Self {
    use self::ConditionalType::*;
    match cs {
      "\\if" => If,
      "\\else" => Else,
      "\\or" => Or,
      "\\fi" => Fi,
      _ => If,
    }
  }
}

// This is ONLY used for \ifcase.
pub struct ConditionalOptions {
  pub scope: Option<Scope>,
  pub locked: Option<bool>,
  pub skipper: Option<bool>,
}
impl Default for ConditionalOptions {
  fn default() -> Self {
    ConditionalOptions {
      scope: None,
      locked: None,
      skipper: None,
    }
  }
}

#[derive(Clone)]
pub struct Conditional {
  pub cs: Token,
  pub paramlist: Option<Parameters>,
  pub test: Option<ConditionalClosure>,
  pub conditional_type: Option<ConditionalType>,
  pub locked: Option<bool>,
  pub skipper: Option<bool>,
}
impl Default for Conditional {
  fn default() -> Self {
    Conditional {
      cs: T_CS!(s!("Conditional")),
      paramlist: None,
      test: None,
      conditional_type: None,
      locked: None,
      skipper: None,
    }
  }
}
impl PartialEq for Conditional {
  fn eq(&self, other: &Conditional) -> bool { self.cs == other.cs }
}

impl Object for Conditional {}
impl Definition for Conditional {
  // sub new {
  //   my ($class, $cs, $parameters, $test, %traits) = @_;
  //   my $source = $STATE->getStomach->getGullet->getMouth;
  //   return bless { cs => $cs, parameters => $parameters, test => $test,
  //     locator      => "from " . $source->getLocator(-1),
  //     isExpandable => 1,
  //     %traits }, $class; }

  // Note that although conditionals are Expandable,
  // they are NOT defined as macros, so they don't need to handle doInvocation,
  fn invoke(&self, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> {
    // A real conditional must have condition_type set
    if let Some(ref cond_type) = self.conditional_type {
      use self::ConditionalType::*;
      match *cond_type {
        If => self.invoke_conditional(gullet, state),
        Else => self.invoke_else(gullet, state),
        Or => self.invoke_else(gullet, state),
        Fi => self.invoke_fi(gullet, state),
      };
    } else {
      error!(
        target: &s!("unexpected:{}", self.cs), //$gullet,
        "Unknown conditional control sequence {:?}",
        state.current_token
      );
    }
    Ok(Tokens!())
  }
  // TODO:
  fn get_parameters(&self) -> &Option<Parameters> { &self.paramlist }
  fn get_cs(&self) -> Token { self.cs.clone() }

  fn get_cs_name(&self) -> String { self.cs.get_cs_name() }

  fn get_locator(&self) -> String { String::from("Locator is TODO") }

  // Not implemented for expandable
  fn invoke_primitive(
    &self,
    _gullet: &mut Stomach,
    _caller: Rc<Definition>,
    _state: &mut State,
  ) -> Result<Vec<Digested>>
  {
    Ok(Vec::new())
  }
  fn before_digest(&self) -> Option<&Vec<BeforeDigestClosure>> { None }
  fn after_digest(&self) -> Option<&Vec<DigestionClosure>> { None }
  fn do_absorbtion(
    &self,
    _document: &mut Document,
    _whatsit: &Whatsit,
    _state: &mut State,
  ) -> Result<()>
  {
    fatal!(
      Definition,
      Unexpected,
      "do_absorbtion on Conditional should never be called!"
    );
  }
}

impl Conditional {
  pub fn invoke_conditional(&self, gullet: &mut Gullet, state: &mut State) -> Result<()> {
    // TODO!!! Implement in full
    // Keep a stack of the conditionals we are processing.
    // let mut ifid = state.lookup_int("if_count").unwrap_or(0);
    // ifid += 1;
    // state.assign_int("if_count", ifid, Some(Scope::Global));
    //   local $LaTeXML::IFFRAME = { token => $LaTeXML::CURRENT_TOKEN, start => $gullet->getLocator,
    //     parsing => 1, elses => 0, ifid => $ifid };
    //   $STATE->unshiftValue(if_stack => $LaTeXML::IFFRAME);

    let args = self.read_arguments(gullet, state)?;
    //   $$LaTeXML::IFFRAME{parsing} = 0;    # Now, we're done parsing the Test clause.
    //   my $tracing = $STATE->lookupValue('TRACINGCOMMANDS');
    //   print STDERR '{' . $self->tracingCSName . "} [#$ifid]\n" if $tracing;
    //   print STDERR $self->tracingArgs(@args) . "\n" if $tracing && @args;
    if let Some(ref test) = self.test {
      if (test)(gullet, args, state)? {
        // print STDERR "{true}\n" if $tracing; }
      } else {
        let to = self.skip_conditional_body(gullet, -1);
        // print STDERR "{false} [skipped to " . ToString($to) . "]\n" if $tracing; } }
      }
    } else {
      // If there's no test, it must be the Special Case, \ifcase
      //     my $num = $args[0]->valueOf;
      //     if ($num > 0) {
      //       my $to = skipConditionalBody($gullet, $num);
      //       print STDERR "{$num} [skipped to " . ToString($to) . "]\n" if $tracing; } }
    }
    Ok(())
  }

  // #======================================================================
  // # Support for conditionals:

  // # Skipping for conditionals
  // #   0 : skip to \fi
  // #  -1 : skip to \else, if any, or \fi
  // #   n : skip to n-th \or, if any, or \else, if any, or \fi.

  // # NOTE that there are 2 kinds of "nested" ifs.
  // #  \if's inside the body of either the true or false branch
  // # are easily skipped by tracking a level of if nesting and skipping over the
  // # same number of \fi as you find \if.
  // #  \if's that get expanded while evaluating the test clause itself
  // # are considerably trickier. There's a frame on the if-stack for this \if
  // # that's above the one we're currently processing; typically the \else & \fi
  // # may still remain, but we need to either evaluate them a normal
  // # if we're continuing to follow the true branch, or skip oever them if
  // # we're trying to find the \else for the false branch.
  // # The danger is mistaking the \else that's associated with the test clause's \if
  // # and taking it for the \else that we're skipping to!
  // # Canonical example:
  // #   \if\ifx AA XY junk \else blah \fi True \else False \fi
  // # The inner \ifx should expand to "XY junk", since A==A
  // # Return the token we've skipped to, and the frame that this applies to.
  fn skip_conditional_body(&self, gullet: &mut Gullet, nskips: i32) {
    //   my $level = 1;
    //   my $n_ors = 0;
    //   my $start = $gullet->getLocator;
    //   # NOTE: Open-coded manipulation of if_stack!
    //   # [we're only reading tokens & looking up, so State shouldn't change behind our backs]
    //   my $stack = $STATE->lookupValue('if_stack');
    //   while (1) {
    //     my ($t, $cond_type) = $gullet->readNextConditional;
    //     last unless $cond_type;
    //     if ($cond_type eq 'if') {    #  Found a \ifxx of some sort
    //       $level++; }
    //     elsif ($cond_type eq 'fi') {    #  Found a \fi
    //       if ($$stack[0] ne $LaTeXML::IFFRAME) {
    //         # But is it for a condition nested in the test clause?
    //         shift(@$stack); }           # then DO pop that conditional's frame; it's DONE!
    //       elsif (!--$level) {           # If no more nesting, we're done.
    //         shift(@$stack);             # Done with this frame
    //         return $t; } }              # AND Return the finishing token.
    //     elsif ($level > 1) {            # Ignore \else,\or nested in the body.
    //     }
    //     elsif (($cond_type eq 'or') && (++$n_ors == $nskips)) {
    //       return $t; }
    //     elsif (($cond_type eq 'else') && $nskips
    //       # Found \else and we're looking for one?
    //       # Make sure this \else is NOT for a nested \if that is part of the test clause!
    //       && ($$stack[0] eq $LaTeXML::IFFRAME)) {
    //       # No need to actually call elseHandler, but note that we've seen an \else!
    //       $$stack[0]{elses} = 1;
    //       return $t; } }    # } #}
    //   Error('expected', '\fi', $gullet, "Missing \\fi or \\else, conditional fell off end",
    //     "Conditional started at $start");
    return;
  }

  pub fn invoke_else(&self, gullet: &mut Gullet, state: &mut State) -> Result<()> { Ok(()) }
  //   my $stack = $STATE->lookupValue('if_stack');
  //   if (!($stack && $$stack[0])) {    # No if stack entry ?
  //     Error('unexpected', $LaTeXML::CURRENT_TOKEN, $gullet,
  //       "Didn't expect a " . Stringify($LaTeXML::CURRENT_TOKEN)
  //         . " since we seem not to be in a conditional");
  //     return; }
  //   elsif ($$stack[0]{parsing}) {     # Defer expanding the \else if we're still parsing the test
  //     return [T_CS('\relax'), $LaTeXML::CURRENT_TOKEN]; }
  //   elsif ($$stack[0]{elses}) {       # Already seen an \else's at this level?
  //     Error('unexpected', $LaTeXML::CURRENT_TOKEN, $gullet,
  //       "Extra " . Stringify($LaTeXML::CURRENT_TOKEN),
  // "already saw \\else for " . Stringify($$stack[0]{token}) . " [" . $$stack[0]{ifid} . "] at " .
  // $$stack[0]{start});     return; }
  //   else {
  //     local $LaTeXML::IFFRAME = $$stack[0];
  //     my $t = skipConditionalBody($gullet, 0);
  //     print STDERR '{' . ToString($LaTeXML::CURRENT_TOKEN) . '}'
  //       . " [for " . ToString($$LaTeXML::IFFRAME{token}) . " #" . $$LaTeXML::IFFRAME{ifid}
  //       . " skipping to " . ToString($t) . "]\n"
  //       if $STATE->lookupValue('TRACINGCOMMANDS');
  //     return; } }

  pub fn invoke_fi(&self, gullet: &mut Gullet, state: &mut State) -> Result<()> { Ok(()) }
  //   my ($self, $gullet) = @_;
  //   my $stack = $STATE->lookupValue('if_stack');
  //   if (!($stack && $$stack[0])) {    # No if stack entry ?
  //     Error('unexpected', $LaTeXML::CURRENT_TOKEN, $gullet,
  //       "Didn't expect a " . Stringify($LaTeXML::CURRENT_TOKEN)
  //         . " since we seem not to be in a conditional");
  //     return; }
  //   elsif ($$stack[0]{parsing}) {     # Defer expanding the \else if we're still parsing the test
  //     return [T_CS('\relax'), $LaTeXML::CURRENT_TOKEN]; }
  //   else {                            # "expand" by removing the stack entry for this level
  //     local $LaTeXML::IFFRAME = $$stack[0];
  //     $STATE->shiftValue('if_stack');    # Done with this frame
  //     print STDERR '{' . ToString($LaTeXML::CURRENT_TOKEN) . '}'
  // . " [for " . Stringify($$LaTeXML::IFFRAME{token}) . " #" . $$LaTeXML::IFFRAME{ifid} .
  // "]\n"       if $STATE->lookupValue('TRACINGCOMMANDS');
  //     return; } }
}
