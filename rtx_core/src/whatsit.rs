use std::borrow::Cow;
// use std::cell::RefCell;
use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;
use std::collections::VecDeque;
use std::fmt;
use std::rc::Rc;

use crate::common::dimension::Dimension;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::locator::Locator;
use crate::common::object::Object;
use crate::common::store::Stored;
use crate::definition::expandable::Expandable;
use crate::definition::{Definition, FontDirective, Reversion};
use crate::document::Document;
use crate::list::List;
use crate::state::{get_dual_branch,lookup_font};
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::{BoxOps, Digested, DigestedData, TexMode};

/// Represents a digested object that can generate arbitrary elements in the XML Document.
#[derive(Clone)]
pub struct Whatsit {
  /// arguments
  pub args: Vec<Option<Digested>>,
  /// additional properties, such as font information or sizing
  pub properties: HashMap<String, Stored>,
  /// the definition responsible for creating this object
  pub definition: Rc<dyn Definition>,
  /// cached tokens for reverting back
  ///  (note that the "reversion" _property_ is currently also used)
  pub reversion: Option<Tokens>,
  /// special-case reversion tokens for whatsits representing Dual math structures
  pub dual_reversion: Option<HashMap<String,Tokens>>,
  /// point of origin in the source file
  pub locator: Locator,
}

impl Default for Whatsit {
  fn default() -> Self {
    Whatsit {
      args: Vec::new(),
      properties: HashMap::default(),
      definition: Rc::new(Expandable::default()),
      reversion: None,
      dual_reversion: None,
      locator: Locator::default(),
    }
  }
}
impl PartialEq for Whatsit {
  fn eq(&self, other: &Whatsit) -> bool {
    // identical definition, argument list and body
    *self.definition == *other.definition
      && self.args == other.args
      && if let Some(Stored::Digested(body1)) = self.properties.get("body") {
        if let Some(Stored::Digested(body2)) = other.properties.get("body") {
          *body1 == *body2
        } else {
          false
        }
      } else {
        !other.properties.contains_key("body")
      }
  }
}

impl Whatsit {
  /// checks the "isMath" property was set to true
  pub fn is_math(&self) -> bool {
    match self.properties.get("isMath") {
      Some(&Stored::Bool(v)) => v,
      _ => false,
    }
  }

  /// A Whatsit is empty if it is marked empty, or space-like, or has an empty body.
  pub fn is_empty(&self) -> Result<bool> {
    Ok(
    // 1. A space-like thing
    // 2. An environment-like structure with an empty body
    // TODO: For now it is difficult to pass in a state with an initialized TeX.pool.
    self.get_property_bool("isEmpty")
      || self.get_property_bool("isSpace")
      || (self.get_definition().get_cs_name() == "Begin"
        && match self.get_body()? {
            Some(b) => b.unlist_ref().iter().all(|inner| inner.is_empty().unwrap_or(false)),
            None => true,
        }))
  }
  /// sets a pre-assembled HashMap of properties
  pub fn set_properties(&mut self, props: HashMap<String, Stored>) {
    for (key, value) in props {
      self.properties.insert(key, value);
    }
  }
  /// accessor for the definition which built this Whatsit
  pub fn get_definition(&self) -> Rc<dyn Definition> { Rc::clone(&self.definition) }
  /// accessor for the argument at index `n` (starting from 1)
  pub fn get_arg(&self, n: usize) -> Option<&Digested> {
    match self.args.get(n - 1) {
      Some(Some(opt)) => Some(opt),
      _ => None,
    }
  }
  /// mutably borrow the argument at index `n` (starting from 1)
  pub fn get_arg_mut(&mut self, n: usize) -> Option<&mut Digested> {
    match self.args.get_mut(n - 1) {
      Some(Some(opt)) => Some(opt),
      _ => None,
    }
  }
  /// accessor for the full list of arguments
  pub fn get_args(&self) -> &Vec<Option<Digested>> { &self.args }
  /// Sets the list of arguments for this whatsit (each arg should be `Digested::List`).
  pub fn set_args(&mut self, args: Vec<Option<Digested>>) { self.args = args; }
  /// accessor for the `trailer` property. See `whatsit::set_body`
  pub fn get_trailer(&self) -> Option<Digested> {
    match self.properties.get("trailer") {
      Some(Stored::Digested(trailer)) => Some(trailer.clone()),
      _ => None,
    }
  }
  /// Sets the body of the `whatsit` to the boxes in `body`.
  /// The last box in `body` is assumed to represent the `trailer`, that is the result of the
  /// invocation that closed the environment or math.  It is stored separately in the properties
  /// under "trailer".
  pub fn set_body(&mut self, mut body: Vec<Digested>) {
    let trailer_opt = body.pop();
    let mode = if self.is_math() {
      TexMode::Math
    } else {
      TexMode::Text
    };

    let mut list = List::new(body);
    if self.is_math() {
      list.mode = Some(mode);
    }
    self
      .properties
      .insert(s!("body"), Digested::from(list).into());
    if let Some(digested) = trailer_opt {
      if let DigestedData::Whatsit(ref trailer) = digested.data() {
        // And copy any otherwise undefined properties from the trailer
        let trailer_val = trailer.borrow();
        let props = trailer_val.get_properties();
        for (prop, value) in props {
          self
            .properties
            .entry(prop.to_string())
            .or_insert_with(|| value.clone());
        }
        self
          .properties
          .insert(s!("trailer"), digested.clone().into());
      }
    }
  }

  /// Like Tokens-substituteParameters, but substitutes in the Whatsit's arguments OR properties!
  /// #<digit> is the standard TeX positional argument
  /// # followed by a T_OTHER(propname) specifies the property propname!!
  fn substitute_parameters(&self, spec: Tokens) -> Result<Vec<Token>> {
    // TODO: This is kind of unfortunate -- I am not sure what are the reasonable "entryways" into
    // the Whatsit substituteParameters. For Expandable we now have guarantees that "#,i" has
    // been mapped into a single T_ARG(#i), but not here. so for now run on each call?
    let mut in_toks = VecDeque::from(spec.unlist());
    let args = self.get_args();
    let props = &self.properties;
    let mut result = Vec::new();
    while let Some(token) = in_toks.pop_front() {
      if token.get_catcode() != Catcode::ARG {
        // Non '#'; copy it
        result.push(token);
      } else {
        let arg_opt = token.with_str(|s| {
          let n = s.parse::<usize>().unwrap() - 1;
          if n < 10 {
            args[n].clone()
          } else {
            match props.get(s) {
              Some(Stored::Digested(v)) => Some((*v).clone()),
              Some(other) => {
                panic!("unexpected prop in substitute_parameters, needed Digested, got: {other:?}")
              },
              None => None,
            }
          }
        });
        if let Some(arg) = arg_opt {
          result.extend(arg.revert()?.unlist());
        }
      }
    }
    Ok(result)
  }
}

impl fmt::Debug for Whatsit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Whatsit[")?;
    let mut pieces = Vec::new();
    pieces.push(
      self
        .get_definition()
        .get_cs()
        .with_cs_name(ToString::to_string),
    );
    for arg_opt in self.get_args() {
      if let Some(arg) = arg_opt {
        pieces.push(arg.stringify());
      } else {
        pieces.push(String::new());
      }
    }
    if self.properties.contains_key("body") {
      pieces.push(self.properties.get("body").unwrap().to_string());
      if let Some(trailer) = self.properties.get("trailer") {
        pieces.push(trailer.to_string());
      }
    }
    write!(f, "{}]", pieces.join(","))
  }
}

impl fmt::Display for Whatsit {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.revert().unwrap())
  }
}

impl Object for Whatsit {
  fn get_locator(&self) -> Locator { self.locator }

  fn stringify(&self) -> String { format!("{self:?}") }

  fn revert(&self) -> Result<Tokens> {
    // WARNING: Forbidden knowledge?
    // (2) caching the reversion (which is a big performance boost)
    let saved_opt = if let Some(this_branch) = get_dual_branch() {
      if let Some(ref dual_reversion) = self.dual_reversion {
        dual_reversion.get(this_branch).cloned()
      } else {
        self.reversion.clone()
      }
    } else {
      self.reversion.clone()
    };
    if let Some(saved) = saved_opt {
      return Ok(saved);
    }

    let mut tokens = Vec::new();
    let defn = &self.definition;
    if defn.get_reversion_spec().is_none() {
      if let Some(Stored::Digested(digested)) = self.properties.get("alignment") {
        if let DigestedData::Alignment(alignment) = digested.data() {
          return alignment.borrow().revert();
        }
      }
    }
    // Find the appropriate reversion spec;
    // content_reversion or presntation_reversion if on dual branch
    // or (general) reversion, or the reversion from the definition
    let spec_opt = if let Some(rev) = self.properties.get("reversion") {
      match rev {
        Stored::Tokens(tks) => Some(Cow::Owned(Reversion::Tokens(tks.clone()))),
        Stored::Reversion(rev) => Some(Cow::Borrowed(rev)),
        other => panic!("TODO: Unexpected reversion directive {other:?}"),
      }
    } else {
      defn.get_reversion_spec().map(Cow::Owned)
    };
    let mut is_closure = false;
    match spec_opt.as_deref() {
      Some(Reversion::Closure(spec)) => {
        is_closure = true;
        let spec_tokens = spec(self, self.get_args()).unwrap();
        tokens = self.substitute_parameters(spec_tokens)?;
      },
      Some(Reversion::Tokens(spec)) => {
        if !spec.is_empty() {
          tokens = self.substitute_parameters(spec.clone())?;
        }
      },
      None => {
        if let Some(alias) = defn.get_alias() {
          if !alias.is_empty() {
            tokens.push(T_CS!(alias));
          }
        } else {
          tokens.push(defn.get_cs().into_owned());
        }
        if let Some(parameters) = defn.get_parameters() {
          // TODO: This is a sticking point. Both in terms of type mismatch between
          // revert_arguments and get_args, but much worse with the expectation of
          // passing in a gullet and state for the parameter reversion
          // for now approximate this with some slight of hand ...
          // tokens.extend(parameters.revert_arguments(self.get_args())?);
          //
          // Note 2: I've already had to dance around the T_BEGIN/T_END wrappers with my hacky
          // workaround so maybe worth taking some time and aligning the idea here
          // with `.revert_arguments` to avoid the insanity?
          //
          // GOAL: push(@tokens, $parameters->revertArguments($self->getArgs)); } }
          let args = self
            .get_args()
            .iter()
            .map(|opt| match opt {
              Some(arg) => Some(arg.revert().ok()?),
              None => None,
            })
            .collect();
          tokens.extend(parameters.revert_arguments(args)?)
        }
      },
    };

    if !is_closure {
      if let Some(body) = self.get_body()? {
        tokens.extend(body.revert()?.unlist());
        if let Some(trailer) = self.get_trailer() {
          tokens.extend(trailer.revert()?.unlist());
        }
      }
    }

    // Now cache it, in case it's needed again
    // TODO: DG: We can't yet cache reversions, because we lack mutability on .revert()
    //       should we reorganize? is it worth it?
    //
    // if let Some(this_branch) = state!().get_dual_branch() {
    //   if self.dual_reversion.is_none() {
    //     self.dual_reversion = Some(HashMap::default());
    //   }
    //   self.dual_reversion.as_mut().unwrap()
    //     .insert(this_branch.to_string(), Tokens::new(tokens.clone()));
    // } else {
    //   self.reversion = Some(Tokens::new(tokens.clone()));
    // }
    Ok(Tokens::new(tokens))
  }
}

impl BoxOps for Whatsit {
  fn get_properties(&self) -> &HashMap<String, Stored> { &self.properties }
  fn with_properties<R, FnR>(&self, caller: FnR) -> R
  where FnR: FnOnce(&HashMap<String, Stored>) -> R {
    caller(&self.properties)
  }
  fn get_properties_mut(&mut self) -> &mut HashMap<String, Stored> { &mut self.properties }
  fn get_property(&self, key: &str) -> Option<Cow<Stored>> {
    self.properties.get(key).map(Cow::Borrowed)
  }
  fn get_property_mut(&mut self, key: &str) -> Option<&mut Stored> { self.properties.get_mut(key) }
  fn get_string(&self) -> Result<Cow<str>> {
    Ok(Cow::Owned(self.revert()?.to_string()))
  }

  fn be_absorbed(&self, document: &mut Document) -> Result<Vec<Node>> {
    // Significant time is consumed here, and associated with a specific CS,
    // so we should be profiling as well!
    // Hopefully the csname is the same that was charged in the digestioned phase!

    // my $profiled = $state->lookupValue('PROFILING') && $defn->getCS;
    // LaTeXML::Definition::startProfiling($profiled, 'absorb') if $profiled;
    // info!(target:"whatsit:be_absorbed", "{:?}", self);

    self.definition.do_absorbtion(document, self)
    // LaTeXML::Definition::stopProfiling($profiled, 'absorb') if $profiled;
  }
  fn get_body(&self) -> Result<Option<Digested>> {
    Ok(match self.properties.get("body") {
      Some(Stored::Digested(body)) => Some(body.clone()),
      _ => None,
    })
  }

  fn get_font(&self) -> Result<Option<Cow<Font>>> {
    match self.properties.get("font") {
      Some(Stored::Font(font)) => Ok(Some(Cow::Owned((**font).clone()))),
      Some(Stored::FontDirective(fd)) => match fd {
        FontDirective::Closure(ref code) => Ok(Some(Cow::Owned(code(Some(self))?))),
        FontDirective::Asset(ref asset) => Ok(Some(Cow::Borrowed(asset))),
      },
      _ => Ok(None),
    }
  }

  fn set_font(&mut self, font: Rc<Font>) {
    self
      .properties
      .insert("font".to_string(), Stored::Font(font));
  }

  fn compute_size(
    &self,
    options: HashMap<String, Stored>,
  ) -> Result<(Dimension, Dimension, Dimension)> {
    let defn = self.get_definition();
    if let Some(sizer) = defn.get_sizer() {
      sizer(self)
    } else {
      // Nothing specified? use #body if any, else sum all box args
      let mut boxes = Vec::new();
      if let Some(body_stored) = self.get_property("body") {
        if let Stored::Digested(ref body) = *body_stored {
          boxes.push((*body).clone());
        }
      }
      if boxes.is_empty() {
        // no body
        for arg in self.args.iter().flatten() {
          boxes.extend(arg.unlist());
        }
      }
      let font = if let Stored::Font(ref sf) = *self.get_property("font").unwrap() {
        sf.clone()
      } else {
        lookup_font().unwrap()
      };
      font.compute_boxes_size(&boxes, options)
    }
  }
}
