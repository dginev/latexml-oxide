use std::collections::HashMap;
use std::rc::Rc;
use libxml::tree::Node;

use common::error::*;
use Digested;
use state::State;
use document::Document;

pub type TagConstructionClosure = Rc<Fn(&mut Document, &mut Node, &mut State) -> Result<()>>;
pub type TagData = (String, Option<HashMap<String, String>>, Digested);

// Specify the properties of a Node tag.
pub enum TagOptionName {
  AfterOpen,
  AfterOpenEarly,
  AfterOpenLate,
  AfterClose,
  AfterCloseEarly,
  AfterCloseLate,
}

#[allow(dead_code)]
impl TagOptionName {
  pub fn all() -> Vec<TagOptionName> {
    use self::TagOptionName::*;
    vec![
      AfterOpen,
      AfterOpenEarly,
      AfterOpenLate,
      AfterClose,
      AfterCloseEarly,
      AfterCloseLate,
    ]
  }
  pub fn is_prepend(&self) -> bool {
    use self::TagOptionName::*;
    match *self {
      AfterOpenEarly | AfterCloseEarly => true,
      _ => false,
    }
  }
  pub fn is_append(&self) -> bool {
    use self::TagOptionName::*;
    match *self {
      AfterOpen | AfterClose | AfterOpenLate | AfterCloseLate => true,
      _ => false,
    }
  }
}

#[derive(Clone)]
pub struct TagOptions {
  pub auto_open: Option<bool>,
  pub auto_close: Option<bool>,
  pub after_open: Option<Vec<TagConstructionClosure>>,
  pub after_close: Option<Vec<TagConstructionClosure>>,
  pub after_open_early: Option<Vec<TagConstructionClosure>>,
  pub after_close_early: Option<Vec<TagConstructionClosure>>,
  pub after_open_late: Option<Vec<TagConstructionClosure>>,
  pub after_close_late: Option<Vec<TagConstructionClosure>>,
}
impl Default for TagOptions {
  fn default() -> Self {
    TagOptions {
      auto_open: None,
      auto_close: None,
      after_open: None,
      after_close: None,
      after_open_early: None,
      after_open_late: None,
      after_close_early: None,
      after_close_late: None,
    }
  }
}
#[allow(dead_code)]
impl TagOptions {
  pub fn get(&self, name: &TagOptionName) -> &Option<Vec<TagConstructionClosure>> {
    use self::TagOptionName::*;
    match *name {
      AfterOpen => &self.after_open,
      AfterOpenEarly => &self.after_open_early,
      AfterOpenLate => &self.after_open_late,
      AfterClose => &self.after_close,
      AfterCloseEarly => &self.after_close_early,
      AfterCloseLate => &self.after_close_late,
    }
  }

  pub fn get_mut(&mut self, name: &TagOptionName) -> &mut Option<Vec<TagConstructionClosure>> {
    use self::TagOptionName::*;
    match *name {
      AfterOpen => &mut self.after_open,
      AfterOpenEarly => &mut self.after_open_early,
      AfterOpenLate => &mut self.after_open_late,
      AfterClose => &mut self.after_close,
      AfterCloseEarly => &mut self.after_close_early,
      AfterCloseLate => &mut self.after_close_late,
    }
  }

  pub fn remove(&mut self, name: &TagOptionName) -> Vec<TagConstructionClosure> {
    use self::TagOptionName::*;
    let field = match *name {
      AfterOpen => &mut self.after_open,
      AfterOpenEarly => &mut self.after_open_early,
      AfterOpenLate => &mut self.after_open_late,
      AfterClose => &mut self.after_close,
      AfterCloseEarly => &mut self.after_close_early,
      AfterCloseLate => &mut self.after_close_late,
    };
    match field {
      Some(ref mut vec) => vec.drain(..).collect(),
      None => Vec::new()
    }
  }

  pub fn set(&mut self, name: &TagOptionName, value: Option<Vec<TagConstructionClosure>>) {
    use self::TagOptionName::*;
    match *name {
      AfterOpen => {
        self.after_open = value;
      },
      AfterOpenEarly => {
        self.after_open_early = value;
      },
      AfterOpenLate => {
        self.after_open_late = value;
      },
      AfterClose => {
        self.after_close = value;
      },
      AfterCloseEarly => {
        self.after_close_early = value;
      },
      AfterCloseLate => {
        self.after_close_late = value;
      },
    }
  }

  pub fn prepend(&mut self, name: &TagOptionName, mut value: Vec<TagConstructionClosure>) {
    use self::TagOptionName::*;
    {// scoping the borrow for "field"
      let drained : Vec<TagConstructionClosure> = match self.get_mut(name) {
        Some(vec) => vec.drain(..).collect(),
        None => Vec::new()
      };
      value.extend(drained);
    }
    // is there a briefer syntax for the assignment?
    self.set(name, Some(value));
  }

  pub fn append(&mut self, name: &TagOptionName, value: Vec<TagConstructionClosure>) {
    use self::TagOptionName::*;
    {
      // initialize
      if self.get(name).is_none() {
        self.set(name,Some(Vec::new()));
      }
    }
    // set
    if let Some(vec) = self.get_mut(name) {
      vec.extend(value);
    }
  }
}
