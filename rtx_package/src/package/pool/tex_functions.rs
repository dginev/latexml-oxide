use std::collections::VecDeque;
use libxml::tree::Node;
use crate::package::*;

pub fn reenter_text_mode(vertical_mode: bool, state: &mut State) {
  let bindings_val = if vertical_mode {
    state.lookup_value("VTEXT_MODE_BINDINGS")
  } else {
    state.lookup_value("HTEXT_MODE_BINDINGS")
  };

  let mut bindings: VecDeque<Stored> = match bindings_val {
    Some(Stored::VecDequeStored(ref vdq)) => vdq.clone(),
    _ => VecDeque::new(),
  };
  if let Some(Stored::VecDequeStored(ref text_mode_bindings)) = state.lookup_value("TEXT_MODE_BINDINGS") {
    bindings.extend(text_mode_bindings.clone());
  }
  for binding in bindings {
    if let Stored::Tokens(tks) = binding {
      let vec = tks.unlist();
      state.let_i(&vec[0], vec[1].clone(), None);
    }
  }
  return;
}

pub fn only_preamble(cs: &str, stomach: &mut Stomach, state: &mut State) {
  if !state.lookup_bool("inPreamble") {
    Error!("unexpected", cs, stomach, state, "The current command can only appear in the preamble");
  }
}

pub fn today(state: &State) -> String {
  let month_names = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
  ];
  let month = month_names[state.lookup_register("\\month", vec![]).unwrap().value_of() as usize - 1];
  let day = state.lookup_register("\\day", vec![]).unwrap().value_of();
  let year = state.lookup_register("\\year", vec![]).unwrap().value_of();
  s!("{} {}, {}", month, day, year)
}

pub fn parse_def_parameters(cs: &Token, params_in: Tokens, state: &mut State) -> Result<Option<Parameters>> {
  let mut tokens: VecDeque<Token> = if params_in.is_stub() {
    VecDeque::new() // handle default tokens making their way into here, they are ignorable
  } else {
    VecDeque::from(params_in.unlist())
  };
  // Now, recognize parameters and delimiters.
  let mut params = Vec::new();
  let mut n = 0;
  while let Some(mut t) = tokens.pop_front() {
    if t.get_catcode() == Catcode::PARAM {
      if tokens.is_empty() {
        // Special case: lone # NOT following a numbered parameter
        // Note that we require a { to appear next, but do NOT read it!
        params.push(Parameter::new("RequireBrace", "RequireBrace", state)?);
      } else {
        n += 1;
        t = tokens.pop_front().unwrap();
        // TODO: Double-check we're not missing cases from the original:
        //       ($n == (ord($t->getString) - ord('0'))
        let t_num = t.get_string().parse::<i32>().unwrap_or(-1);
        if t_num != n {
          fatal!(ParamSpec, Expected, s!("Parameters for {:?} not in order in {:?}", cs, params));
        }
        // Check for delimiting text following the parameter #n
        let mut delim = Vec::new();
        let mut pc = Catcode::MARKER; // throwaway initial val
        let mut cc;
        while !tokens.is_empty() && (tokens.front().unwrap().get_catcode() != Catcode::PARAM) {
          let d = tokens.pop_front().unwrap();
          cc = d.get_catcode();
          if !(cc == pc && cc == Catcode::SPACE) {
            // BUT collapse whitespace!
            delim.push(d);
          }
          pc = cc;
        }
        // Found text that marks the end of the parameter
        if !delim.is_empty() {
          let expected = Tokens::new(delim);
          params.push(
            Parameter {
              name: s!("Until"),
              spec: s!("Until:{}", expected),
              extra: expected.into(),
              ..Parameter::default()
            }
            .init(state)?,
          );
        } else if tokens.len() == 1 && tokens.front().unwrap().get_catcode() == Catcode::PARAM {
          // Special case: trailing sole # => delimited by next opening brace.
          tokens.pop_front();
          params.push(Parameter::new("UntilBrace", "UntilBrace", state)?);
        } else {
          // Nothing? Just a plain parameter.
          params.push(Parameter::new("Plain", "{}", state)?);
        }
      }
    } else {
      // Initial delimiting text is required.
      let mut lit: Vec<Token> = vec![t];
      while !tokens.is_empty() && (tokens.front().unwrap().get_catcode() != Catcode::PARAM) {
        lit.push(tokens.pop_front().unwrap());
      }
      let expected = Tokens::new(lit);
      params.push(
        Parameter {
          name: s!("Match"),
          spec: s!("Match:{}", expected),
          extra: expected.into(),
          novalue: true,
          ..Parameter::default()
        }
        .init(state)?,
      );
    }
  }
  // return (@params ? LaTeXML::Core::Parameters->new(@params) : undef);
  if params.is_empty() {
    Ok(None)
  } else {
    Ok(Some(Parameters { params }))
  }
}

pub fn do_def(globally: bool, expanded: bool, stomach: &mut Stomach, args: Vec<Tokens>, state: &mut State) -> Result<Vec<Digested>> {
  BindState!(stomach, state);
  unpack!(args => cs, params, body);
  // ensure params is empty if it contains only the default token
  // TODO: is this a flaw of parameter parsing?
  let params = if params.is_stub() { Tokens!() } else { params };
  let cs: Token = cs.into();
  let paramlist = parse_def_parameters(&cs, params, state)?;
  if expanded {
    state.noexpand_the = true;
    let gullet = stomach.get_gullet_mut();
    body = Expand!(body, gullet, state);
  }
  let scope = if globally { Some(Scope::Global) } else { None };
  state.install_definition(
    Expandable {
      cs,
      paramlist,
      expansion: body.into(),
      ..Expandable::default()
    },
    scope,
  );
  AfterAssignment!();
  Ok(Vec::new())
}

// Kinda rough: We don't really keep track of modes as carefully as TeX does.
// We'll assume that a box is horizontal if there's anything at all,
// but it's not a vbox (!?!?)
pub fn classify_box(boxnum: Token, state: &State) -> &'static str {
  let boxnum: Number = boxnum.to_number();
  match state.lookup_value(&s!("box{}", boxnum.value_of())) {
    Some(Stored::Digested(ref d)) => match **d {
      Digested::Whatsit(ref w) if w.borrow().definition == state.lookup_definition(&T_CS!("\\vbox")).unwrap() 
        => "vbox",
      _ => "hbox"
    },
    _ => "",
  }
}

const MATH_CLASS_ROLE: [&str; 8] = ["", "BIGOP", "BINOP", "RELOP", "OPEN", "CLOSE", "PUNCT", ""];
// Is this "fontinfo" stuff sufficient to maintain a math font "family" ??
// What we're really after is a connectio nto a font encoding mapping.
pub fn decode_math_char(mut n: u16, state: &State) -> (Option<String>, Option<char>) {
  let class: u16 = n / (16 * 256);
  n %= 16 * 256;
  let fam: u16 = n / 256;
  n %= 256;
  let font = state.lookup_value(&s!("fontinfo_{}_text", fam)).unwrap_or_else(|| {
    state
      .lookup_value(&s!("fontinfo_{}_script", fam))
      .unwrap_or_else(|| state.lookup_value(&s!("fontinfo_{}_scriptscript", fam)).unwrap_or(&Stored::Bool(false)))
  });
  // TODO: This function is called with n=20,000, how is the char cast sensible here? Consult Bruce.
  let c = n as u8 as char; // TODO: confusing types, the 256 arithmetic implies larger than u8 inputs, what for?
                           // // If no specific class, Lookup properties from a DefMath?
  let charinfo = state.lookup_value(&s!("math_token_attributes_{}", c));
  let fontinfo = state.lookup_value(&s!("fontinfo_{}", font.to_string()));
  let mut role = MATH_CLASS_ROLE[class as usize];

  if role.is_empty() {
    if let Some(Stored::HashString(ref info)) = charinfo {
      role = &info[role];
    }
  }
  let role_opt = if role.is_empty() { None } else { Some(role.to_string()) };
  let font_opt = if let Some(Stored::Font(ref info)) = fontinfo {
    if let Some(ref data) = info.encoding {
      font::decode(n as u8, Some(data.to_string()), false, state)
    } else {
      Some(c)
    }
  } else {
    None
  };

  (role_opt, font_opt)
}

// Risky: I think this needs to be digested as a body to work like TeX (?)
// but parameter think's it's just parsing from gullet...
pub fn read_box_contents(gullet: &mut Gullet, everybox_opt: Option<Tokens>, state: &mut State) -> Result<Tokens> {
  while let Some(t) = gullet.read_token(state) {
    if t == T_BEGIN!() {
      break;
    } // Skip till { or \bgroup
  }
  // Now, insert some extra tokens, if any, possibly from \afterassignment
  if let Some(token) = state.lookup_tokens("BeforeNextBox") {
    state.assign_value("BeforeNextBox", None, Some(Scope::Global));
    gullet.unread(token);
  }
  // AND, insert any extra tokens passed in, due to everyhbox or everyvbox
  if let Some(everybox) = everybox_opt {
    gullet.unread(everybox);
  }
  Ok(Tokens!())
}

/// Reading a Box's content is crucially dependent on invoking the "{" token and obtaining a digested result
/// Hence it is *always* needed to pair `read_box_contents` with its stomach-level counterpart, `predigest_box_contents`
pub fn predigest_box_contents(stomach: &mut Stomach, _tokens: Tokens, state: &mut State) -> Result<Option<Digested>> {
  let mut contents = stomach.invoke_token(&T_BEGIN!(), state)?;
  if contents.is_empty() {
    Ok(None)
  } else {
    Ok(Some(contents.remove(0)))
  }
}

pub fn revert_spec(whatsit: &mut Whatsit, keyword: &str, state: &mut State) -> Vec<Token> {
  //   my ($whatsit, $keyword) = @_;
  //   my $value = $whatsit->getProperty($keyword);
  //   return ($value ? (Explode($keyword), Revert($value)) : ()); }
  unimplemented!()
}

/// This attempts to be a generalize vbox construction;
/// It tries to figure out whether an ltx:inline-block or ltx:para is needed,
/// and attempts to figure out whether sequences of the inserted content
/// need to be explicitly wrapped in some kind of block element (presumably ltx:p).
/// It returns the inserted inner blocks,
/// whether or not they got wrapped by that ltx:inline-block; which it DOESN'T TELL YOU ABOUT!
pub fn insert_block(document: &mut Document, contents: Digested, mut blockattr: HashMap<String, String>, state: &mut State) -> Result<Vec<Node>> {
  // Create something like:
  // "<ltx:inline-block vattach='$vattach' height='#height'>#2</ltx:inline-block>"
  let context = document.get_element().unwrap();    // Where we originally start inserting.
  
  let mut blocktag  = "ltx:block";
  let mut iblocktag = "ltx:inline-block";
  if blockattr.get("para").is_some() {
    blocktag  = "ltx:para";
    iblocktag = "ltx:inline-para";
    blockattr.remove("para");
  }
  // Generally, we're going to need some sort of container to hold the contents of the block.
  // Particularly if we're: setting various size & positioning attributes;
  // or can't currently open an ltx:p; or if the current point accepts plain text (#PCDATA).
  // If we're in an inline context, we'll need a ltx:inline-block,  otherwise ltx:block.
  // [Or maybe an ltx:para... when does that happen?]
  let mut newblock : Option<Node> = None;
  let mut unwrap   = 0;
  let mut remove = vec![];
  // drop all empty values
  for (key, val) in &blockattr {
    if val.is_empty() {
      remove.push(key.to_string())
    }
  }
  for key in remove {
    blockattr.remove(&key);
  }
  
  if blockattr.is_empty() || !document.can_contain_node_somehow(&context, "ltx:p", state) || document.can_contain(&context, "#PCDATA", state) {
    let tag =  if document.can_contain(&context, blocktag, state) {
      blocktag } else { iblocktag };
    let mut attr_arg = blockattr.clone();
    attr_arg.insert("_autoclose".to_string(), "true".to_string());
    newblock = Some(document.open_element(tag, Some(attr_arg), None, state)?); 
  }
  // Insert the content for the block, and reduce
  document.set_attribute(&mut document.get_element().unwrap(), "_vertical_mode_", "true")?;    // HACK!!!! (see \hbox)
  
  document.absorb(contents, state)?;
  let absorbed = document.drain_constructed_nodes();
  let mut nodes = document.filter_children(document.filter_deletions(absorbed));

  // Scan the inserted nodes, wrapping sequences of Inline items with a ltx:p
  let mut newnodes = Vec::new();
  while !nodes.is_empty() {
    if state.model.get_node_qname(nodes.first().as_ref().unwrap()) == "ltx:break" {    // ltx:break are superflous, now.
      document.remove_node(nodes.remove(0));
      continue;
    }
    let mut inline = Vec::new(); // Collect up sequences of Inline
    while !nodes.is_empty() && state.model.is_node_in_schema_class("Inline", nodes.first().unwrap()) {
      inline.push(nodes.remove(0));
    }
    if !inline.is_empty() {
      if let Some(wrapped) = document.wrap_nodes("ltx:p", inline, state)? {
        newnodes.push(wrapped);
      }
    } else {
      newnodes.push(nodes.remove(0));
    }
  }

  // If we've inserted a wrapper element, close all open elements up to it's parent
  // It may have auto-opened some element to contain it, but leave that open for following material
  // Otherwise, close everything back up to the originally open element (but only if still open!)
  if let Some(ref blocknode) = newblock {
    document.close_to_node(blocknode.get_parent().as_ref().unwrap(), true, state)?;
  } else {
    document.close_to_node(&context, true, state)?;
  }
  // Check if the ltx:inline-block container is really needed.
  if let Some(blocknode) = newblock {
    let mut rows = blocknode.get_child_nodes();
    if rows.is_empty() {    // Insertion came up empty?
      document.remove_node(blocknode); // then remove the new block entirely
    } else if rows.len() == 1 {// Else only 1 item inside, then flatten
      let mut first = rows.pop().unwrap();
      let first_name = state.model.get_node_qname(&first);
      if document.can_contain(blocknode.get_parent().as_ref().unwrap(), &first_name, state)    // if allowed.
        && (!blockattr.is_empty()
         || !blockattr.keys().any(|attr|
               document.can_node_have_attribute(rows.first().unwrap(), attr, state)))
      {
        for (key,val) in blockattr { 
          document.set_attribute(&mut first, &key, &val)?;
        }
        document.unwrap_nodes(blocknode)?; 
      }
    }
  }

  // And return the list of "rows" in the box (in case they need attributes....)
  Ok(newnodes)
}
