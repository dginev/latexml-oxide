use crate::prelude::*;

LoadDefinitions!({

  DefPrimitive!("\\vsplit Number Match:to Dimension", sub[(number,_to,_dimension)] {
    // analog to \box for now.
    let box_key   = s!("box{}", number.value_of());
    if let Some(Stored::Digested(stuff)) = lookup_value(&box_key) {
      adjust_box_color(&stuff)?;
      if stuff.is_empty()? { Digested::from(List::default()) } else { stuff }
    } else {
      Digested::from(List::default())
    }
  });

});