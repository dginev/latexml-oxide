use libxml::tree::Node;
use rustc_hash::FxHashMap as HashMap;
use std::rc::Rc;

use crate::Digested;
use crate::common::error::*;
use crate::document::Document;

/// Perl's Tag afterOpen/afterClose closures receive ($document, $node, $box).
/// The $box is the Digested (whatsit) associated with the node, retrieved via getNodeBox.
pub type TagConstructionClosure =
  Rc<dyn Fn(&mut Document, &mut Node, Option<&Digested>) -> Result<()>>;
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
    matches!(*self, AfterOpenEarly | AfterCloseEarly)
  }
  pub fn is_append(&self) -> bool {
    use self::TagOptionName::*;
    matches!(
      *self,
      AfterOpen | AfterClose | AfterOpenLate | AfterCloseLate
    )
  }
}

#[derive(Clone, Default)]
pub struct TagOptions {
  pub auto_open:         Option<bool>,
  pub auto_close:        Option<bool>,
  pub after_open:        Option<Vec<TagConstructionClosure>>,
  pub after_close:       Option<Vec<TagConstructionClosure>>,
  pub after_open_early:  Option<Vec<TagConstructionClosure>>,
  pub after_close_early: Option<Vec<TagConstructionClosure>>,
  pub after_open_late:   Option<Vec<TagConstructionClosure>>,
  pub after_close_late:  Option<Vec<TagConstructionClosure>>,
}

impl TagOptions {
  pub fn get(&self, name: &TagOptionName) -> Option<&Vec<TagConstructionClosure>> {
    use self::TagOptionName::*;
    match *name {
      AfterOpen => self.after_open.as_ref(),
      AfterOpenEarly => self.after_open_early.as_ref(),
      AfterOpenLate => self.after_open_late.as_ref(),
      AfterClose => self.after_close.as_ref(),
      AfterCloseEarly => self.after_close_early.as_ref(),
      AfterCloseLate => self.after_close_late.as_ref(),
    }
  }

  pub fn get_mut(&mut self, name: &TagOptionName) -> Option<&mut Vec<TagConstructionClosure>> {
    use self::TagOptionName::*;
    match *name {
      AfterOpen => self.after_open.as_mut(),
      AfterOpenEarly => self.after_open_early.as_mut(),
      AfterOpenLate => self.after_open_late.as_mut(),
      AfterClose => self.after_close.as_mut(),
      AfterCloseEarly => self.after_close_early.as_mut(),
      AfterCloseLate => self.after_close_late.as_mut(),
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
      Some(ref mut vec) => std::mem::take(vec),
      None => Vec::new(),
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
    let drained: Vec<TagConstructionClosure> = match self.get_mut(name) {
      Some(vec) => std::mem::take(vec),
      None => Vec::new(),
    };

    value.extend(drained);

    self.set(name, Some(value));
  }

  pub fn append(&mut self, name: &TagOptionName, value: Vec<TagConstructionClosure>) {
    if self.get(name).is_none() {
      // initialize if needed
      self.set(name, Some(Vec::new()));
    }

    if let Some(vec) = self.get_mut(name) {
      vec.extend(value);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn tag_option_name_all_has_six() {
    let all = TagOptionName::all();
    assert_eq!(all.len(), 6);
  }

  #[test]
  fn tag_option_name_is_prepend_vs_append() {
    assert!(TagOptionName::AfterOpenEarly.is_prepend());
    assert!(TagOptionName::AfterCloseEarly.is_prepend());
    assert!(!TagOptionName::AfterOpen.is_prepend());
    assert!(!TagOptionName::AfterClose.is_prepend());

    assert!(TagOptionName::AfterOpen.is_append());
    assert!(TagOptionName::AfterClose.is_append());
    assert!(TagOptionName::AfterOpenLate.is_append());
    assert!(TagOptionName::AfterCloseLate.is_append());
    assert!(!TagOptionName::AfterOpenEarly.is_append());
    assert!(!TagOptionName::AfterCloseEarly.is_append());
  }

  #[test]
  fn tag_option_name_prepend_and_append_are_disjoint() {
    // Every variant is either prepend OR append, not both.
    for name in TagOptionName::all() {
      let p = name.is_prepend();
      let a = name.is_append();
      assert!(
        p != a,
        "variant classification should be exactly one of prepend/append"
      );
    }
  }

  #[test]
  fn tag_options_default_all_none() {
    let t = TagOptions::default();
    assert!(t.auto_open.is_none());
    assert!(t.auto_close.is_none());
    assert!(t.after_open.is_none());
    assert!(t.after_close.is_none());
    assert!(t.after_open_early.is_none());
    assert!(t.after_close_early.is_none());
    assert!(t.after_open_late.is_none());
    assert!(t.after_close_late.is_none());
  }

  #[test]
  fn tag_options_get_returns_stored_vec() {
    let mut t = TagOptions::default();
    // Initially all None.
    assert!(t.get(&TagOptionName::AfterOpen).is_none());
    // After set, get returns the slot.
    t.after_open = Some(Vec::new());
    assert!(t.get(&TagOptionName::AfterOpen).is_some());
    assert_eq!(t.get(&TagOptionName::AfterOpen).unwrap().len(), 0);
  }

  #[test]
  fn tag_options_get_independent_per_variant() {
    // Setting one slot doesn't populate others.
    let t = TagOptions {
      after_open: Some(Vec::new()),
      ..Default::default()
    };
    assert!(t.get(&TagOptionName::AfterOpen).is_some());
    assert!(t.get(&TagOptionName::AfterClose).is_none());
    assert!(t.get(&TagOptionName::AfterOpenLate).is_none());
    assert!(t.get(&TagOptionName::AfterCloseEarly).is_none());
  }
}
