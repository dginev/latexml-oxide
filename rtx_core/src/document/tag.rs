use std::rc::Rc;
use libxml::tree::Node;

use Digested;
use state::State;
use document::Document;

pub type TagConstructionClosure = Rc<Fn(&mut Document, Node, Option<Digested>, &mut State)>;

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
    vec![AfterOpen,
         AfterOpenEarly,
         AfterOpenLate,
         AfterClose,
         AfterCloseEarly,
         AfterCloseLate]
  }
  pub fn is_prepend(&self) -> bool {
    use self::TagOptionName::*;
    match *self {
      AfterOpenEarly | AfterCloseEarly => true,
      _ => false
    }
  }
  pub fn is_append(&self) -> bool {
    use self::TagOptionName::*;
    match *self {
      AfterOpen | AfterClose | AfterOpenLate | AfterCloseLate => true,
      _ => false
    }
  }
}

#[derive(Clone)]
pub struct TagOptions {
  pub auto_open: bool,
  pub auto_close: bool,
  pub after_open: Vec<TagConstructionClosure>,
  pub after_close: Vec<TagConstructionClosure>,
  pub after_open_early: Vec<TagConstructionClosure>,
  pub after_close_early: Vec<TagConstructionClosure>,
  pub after_open_late: Vec<TagConstructionClosure>,
  pub after_close_late: Vec<TagConstructionClosure>,
}
impl Default for TagOptions {
  fn default() -> Self {
    TagOptions {
      auto_open: true,
      auto_close: true,
      after_open: Vec::new(),
      after_close: Vec::new(),
      after_open_early: Vec::new(),
      after_open_late: Vec::new(),
      after_close_early: Vec::new(),
      after_close_late: Vec::new(),
    }
  }
}
#[allow(dead_code)]
impl TagOptions {
  pub fn get(&self, name: &TagOptionName) -> &Vec<TagConstructionClosure> {
    use self::TagOptionName::*;
    match *name {
      AfterOpen => &self.after_open,
      AfterOpenEarly => &self.after_open_early,
      AfterOpenLate => &self.after_open_late,
      AfterClose => &self.after_close,
      AfterCloseEarly => &self.after_close_early,
      AfterCloseLate => &self.after_close_late
    }
  }

  pub fn get_mut(&mut self, name: &TagOptionName) -> &mut Vec<TagConstructionClosure> {
    use self::TagOptionName::*;
    match *name {
      AfterOpen => &mut self.after_open,
      AfterOpenEarly => &mut self.after_open_early,
      AfterOpenLate => &mut self.after_open_late,
      AfterClose => &mut self.after_close,
      AfterCloseEarly => &mut self.after_close_early,
      AfterCloseLate => &mut self.after_close_late
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
      AfterCloseLate => &mut self.after_close_late
    };
    field.drain(..).collect()
  }

  pub fn set(&mut self, name: &TagOptionName, value: Vec<TagConstructionClosure>) {
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
    match *name {
      AfterOpen => {
        let drained : Vec<TagConstructionClosure> = self.after_open.drain(..).collect();
        value.extend(drained);
        self.after_open = value;
      },
      AfterOpenEarly => {
        let drained : Vec<TagConstructionClosure> = self.after_open_early.drain(..).collect();
        value.extend(drained);
        self.after_open_early = value;
      },
      AfterOpenLate => {
        let drained : Vec<TagConstructionClosure> = self.after_open_late.drain(..).collect();
        value.extend(drained);
        self.after_open_late = value;
      },
      AfterClose => {
        let drained : Vec<TagConstructionClosure> = self.after_close.drain(..).collect();
        value.extend(drained);
        self.after_close = value;
      },
      AfterCloseEarly => {
        let drained : Vec<TagConstructionClosure> = self.after_close_early.drain(..).collect();
        value.extend(drained);
        self.after_close_early = value;
      },
      AfterCloseLate => {
        let drained : Vec<TagConstructionClosure> = self.after_close_late.drain(..).collect();
        value.extend(drained);
        self.after_close_late = value;
      },
    }
  }

  pub fn append(&mut self, name: &TagOptionName, value: Vec<TagConstructionClosure>) {
    use self::TagOptionName::*;
    match *name {
      AfterOpen => {
        self.after_open.extend(value);
      },
      AfterOpenEarly => {
        self.after_open_early.extend(value);
      },
      AfterOpenLate => {
        self.after_open_late.extend(value);
      },
      AfterClose => {
        self.after_close.extend(value);
      },
      AfterCloseEarly => {
        self.after_close_early.extend(value);
      },
      AfterCloseLate => {
        self.after_close_late.extend(value);
      },
    }
  }
}
