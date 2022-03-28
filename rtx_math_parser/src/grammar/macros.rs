// Simple helper for hashset creation
// Source: https://riptutorial.com/rust/example/4149/create-a-hashset-macro
#[macro_export]
macro_rules! set {
    ( $( $x:expr ),* ) => {  // Match zero or more comma delimited items
        {
            use std::collections::HashSet;
            let mut temp_set = HashSet::new();  // Create a mutable HashSet
            $(
                temp_set.insert($x); // Insert each item matched into the HashSet
            )*
            temp_set // Return the populated HashSet
        }
    };
}

#[macro_export]
macro_rules! unpack {
  ($args:ident => $var:ident) => (count_unpack!(0usize, $args => $var));
  ($args:ident => $($var:ident),*) => (count_unpack!(0usize, $args => $($var),*));
}

#[macro_export]
macro_rules! count_unpack {
  ($index:expr, $args:ident => $var:ident) => (
    let $var = $args.remove(0);
  );
  ($index:expr, $args:ident => $var:ident,$($tail:ident),*) => {
    count_unpack!($index,$args => $var);
    count_unpack!(1usize+$index, $args => $($tail),*)
  }
}

#[macro_export]
macro_rules! registry {
  ($grammar:ident, $actions:ident, $builder:ident) => {
    let lexeme_sep = $grammar.literal_string(None, ":")?;
    // Lexical terminals, to be used as constituents of complex token definitions
    // must not be declared with the TreeBuilder
    let digit = $grammar.char_range(None, '0', '9')?;
    let lex_char = $grammar.inverse_string_set(None, ":\t\n\r ")?;
    let lex_plus = $grammar.plus(None, lex_char)?;
    let d_plus = $grammar.plus(None, digit)?;
    // let ws_char = $grammar.string_set(None, "\t\n\r ")?;
    let ws_char = $grammar.literal_string(None, " ")?;

    macro_rules! grammar {
      () => {
        $grammar
      };
    }
    macro_rules! actions {
      () => {
        $actions
      };
    }
    macro_rules! builder {
      () => {
        $builder
      };
    }
    macro_rules! lexeme_sep {
      () => {
        lexeme_sep
      };
    }
    macro_rules! lex_plus {
      () => {
        lex_plus
      };
    }
    macro_rules! d_plus {
      () => {
        d_plus
      };
    }
    macro_rules! ws_char {
      () => {
        ws_char
      };
    }
  };
}

#[macro_export]
macro_rules! default_registry {
  () => {
    let mut g = MarpaGrammar::new().unwrap();
    let mut actions = Actions::default();
    // tree builder from marpa crate (should we move to an in-house builder combined with actions?)
    let mut builder = TreeBuilder::new();
    registry!(g, actions, builder);
  };
}

#[macro_export]
macro_rules! register {
  ($rule:ident, $($arg:ident)+ => $call:ident) => {
    #[allow(unused_variables)]
    actions!().register(
        #[allow(unused_variables)]
        $rule.rule(),
        ::std::sync::Arc::new($call))
  };
  ($rule:ident, $($arg:ident)+ => $body:block) => {
    #[allow(unused_variables)]
    actions!().register(
        $rule.rule(),
        ::std::sync::Arc::new(|rule_id: i32, mut args: Vec<Option<Tree>>| {
          #[allow(unused_variables)]
          unpack!(args => $($arg),+);
          Some($body)
        }))
  };
  ($rule:ident, $($arg:ident)+) => { };
}

#[macro_export]
macro_rules! rule {
  ($name:ident = $($parts:ident)+ $(=> $action:block)?$(=> $fn:ident)?) => {
    let $name = match grammar!().rule(None, &[$($parts),+]) {
      Ok(r) => r,
      Err(e) => panic!("Failed to instantiate rule \"{} ={}\" ({:?})", stringify!($name), stringify!($($parts),+), e)
    };
    builder!().rule($name.rule());
    register!($name, $($parts)+ $(=> $action)?$(=> $fn)?);
  };
  ($name:ident = $($parts:ident)+ $(=> $action:block)?$(=> $fn:ident)? | $($($moreparts:ident)+ $(=> $moreaction:block)?$(=> $morefn:ident)?)|+) => {
    let $name = match grammar!().rule(None, &[$($parts),+]) {
      Ok(r) => r,
      Err(e) => panic!("Failed to instantiate rule \"{} ={}\" ({:?})", stringify!($name), stringify!($($parts),+), e)
    };
    builder!().rule($name.rule());
    register!($name, $($parts)+ $(=> $action)?$(=> $fn)?);
    rule!($name += $($($moreparts)+ $(=> $moreaction)?$(=> $morefn)?)|+);
  };
  // continuations for | clauses
  ($name:ident += $($parts:ident)+$(=> $action:block)?$(=> $fn:ident)?) => {
    let subrule = match grammar!().rule(Some($name), &[$($parts),+]) {
      Ok(r) => r,
      Err(e) => panic!("Failed to instantiate subrule \"{} = {}\" ({:?})", stringify!($name), stringify!($($parts),+), e)
    };
    builder!().rule(subrule.rule());
    register!(subrule, $($parts)+ $(=> $action)?$(=> $fn)?);
  };
  ($name:ident += $($parts:ident)+ $(=> $action:block)?$(=> $fn:ident)? |
    $($($moreparts:ident)+ $(=> $moreaction:block)?$(=> $morefn:ident)?)|+) => {
    let subrule = match grammar!().rule(Some($name), &[$($parts),+]) {
      Ok(r) => r,
      Err(e) => panic!("Failed to instantiate subrule \"{} = {}\" ({:?})", stringify!($name), stringify!($($parts),+), e)
    };
    builder!().rule(subrule.rule());
    register!(subrule, $($parts)+ $(=> $action)?$(=> $fn)?);
    rule!($name += $($($moreparts)+ $(=> $moreaction)?$(=> $morefn)?)|+);
  };
}

#[macro_export]
macro_rules! rules {
  ($($name:ident $op:tt $($($parts:ident)+ $(=> $action:block)?$(=> $fn:ident)?)|+);+) => {
    $(
      rule!($name $op $($($parts)+ $(=> $action)?$(=> $fn)?)|+)
    );+
  };
}

#[macro_export]
macro_rules! token {
  ($name:ident = $literal:literal) => {
    let literal_piece = grammar!().literal_string(None, $literal)?;
    let $name = grammar!().rule(None, &[literal_piece, lexeme_sep!(), d_plus!(),ws_char!()])?;
    builder!().token($name.rule());
  };
  ($name:ident ~ $literal:literal) => {
    let literal_piece = grammar!().literal_string(None, $literal)?;
    let $name = grammar!().rule(None, &[literal_piece, lexeme_sep!(), lex_plus!(), lexeme_sep!(), d_plus!(),ws_char!()])?;
    builder!().token($name.rule());
  };
  ($name:ident = [ $($part:ident)+ ]) => {
    let $name = grammar!().alternative(None, &[$($part),+])?;
    builder!().token($name.rule());
  };
}

#[macro_export]
macro_rules! start {
  ($top:ident) => {
    builder!().discard(ws_char!().rule());
    grammar!().set_start($top)?;
  };
}
