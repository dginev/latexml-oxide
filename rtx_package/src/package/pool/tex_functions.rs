use std::collections::VecDeque;
use crate::package::*;

pub fn reenter_text_mode(vertical_mode: bool, state: &mut State) {
  BindState!(state);
  let bindings_val = if vertical_mode {
    LookupValue!("VTEXT_MODE_BINDINGS")
  } else {
    LookupValue!("HTEXT_MODE_BINDINGS")
  };

  let mut bindings: VecDeque<Stored> = match bindings_val {
    Some(Stored::VecDequeStored(ref vdq)) => vdq.clone(),
    _ => VecDeque::new(),
  };
  if let Some(Stored::VecDequeStored(ref text_mode_bindings)) = LookupValue!("TEXT_MODE_BINDINGS") {
    bindings.extend(text_mode_bindings.clone());
  }
  for binding in bindings {
    if let Stored::Tokens(tks) = binding {
      let vec = tks.unlist();
      LetI!(&vec[0], vec[1].clone());
    }
  }
  return;
}

pub fn only_preamble(cs: &str, state: &mut State) {
  if !state.lookup_bool("inPreamble") {
    let category_object = s!("unexpected:{}", cs);
    error!(target: &category_object, "The current command can only appear in the preamble");
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
  BindState!(state);
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
  BindState!(state);
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
  AfterAssignment!(state);
  Ok(Vec::new())
}


// Kinda rough: We don't really keep track of modes as carefully as TeX does.
// We'll assume that a box is horizontal if there's anything at all,
// but it's not a vbox (!?!?)
pub fn classify_box(boxnum: Token, state: &State) -> &'static str {
  let boxnum : Number = boxnum.to_number();
  match state.lookup_value(&s!("box{}", boxnum.value_of())) {
    Some(Stored::Digested(ref d)) => match **d {
      Digested::Whatsit(ref w) if Rc::ptr_eq(&w.borrow().definition, &state.lookup_definition(&T_CS!("\\vbox")).unwrap()) => "vbox" ,
      _ => "hbox"
    },
    _ => "",
  }
}

const MATH_CLASS_ROLE : [&str; 8] = ["", "BIGOP", "BINOP", "RELOP", "OPEN", "CLOSE", "PUNCT", ""];
// Is this "fontinfo" stuff sufficient to maintain a math font "family" ??
// What we're really after is a connectio nto a font encoding mapping.
pub fn decode_math_char(mut n: u16, state: &State) -> (Option<String>, Option<char>) {
  let class : u16 = n / (16 * 256);
  n %= 16 * 256;
  let fam : u16  = n / 256;
  n %= 256;
  let font  = state.lookup_value(&s!("fontinfo_{}_text",fam)).unwrap_or_else(||
    state.lookup_value(&s!("fontinfo_{}_script",fam)).unwrap_or_else(|| 
      state.lookup_value(&s!("fontinfo_{}_scriptscript",fam)).unwrap_or(&Stored::Bool(false))
    )
  );
  // TODO: This function is called with n=20,000, how is the char cast sensible here? Consult Bruce.
  let c = n as u8 as char; // TODO: confusing types, the 256 arithmetic implies larger than u8 inputs, what for?
  // // If no specific class, Lookup properties from a DefMath?
  let charinfo = state.lookup_value(&s!("math_token_attributes_{}",c));
  let fontinfo = state.lookup_value(&s!("fontinfo_{}", font.to_string()));
  let mut role = MATH_CLASS_ROLE[class as usize];
  
  if role.is_empty() {
    if let Some(Stored::HashString(ref info)) = charinfo {
      role = &info[role];
    }
  }
  let role_opt = if role.is_empty() {
    None
  } else {
    Some(role.to_string())
  };
  let font_opt = if let Some(Stored::Font(ref info)) = fontinfo {
    if let Some(ref data) = info.encoding {
      font::decode(n as u8, Some(data.to_string()), false, state)
    } else {
      Some(c)
    }
  } else { None };
  
  (role_opt,font_opt)
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
  if let Some(ref token) = state.lookup_tokens("BeforeNextBox") {
    state.assign_value("BeforeNextBox", None, Some(Scope::Global));
    gullet.unread(token);
  }
  // AND, insert any extra tokens passed in, due to everyhbox or everyvbox
  if let Some(everybox) = everybox_opt {
    gullet.unread(&everybox);
  }
  Ok(Tokens!())
}

pub fn predigest_box_contents(stomach: &mut Stomach, _tokens: Tokens, state: &mut State) -> Result<Option<Digested>> {
  let mut contents = stomach.invoke_token(&T_BEGIN!(), state)?;
  Ok(Some(contents.remove(0)))
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
pub fn insert_block(document: &mut Document, contents: Tokens, blockattr: HashMap<String, String>) -> Result<()> {
  unimplemented!();
  // my ($document, $contents, %blockattr) = @_;
  // # Create something like:
  // # "<ltx:inline-block vattach='$vattach' height='#height'>#2</ltx:inline-block>"
  // my $model   = $document->getModel;
  // my $context = $document->getElement;    # Where we originally start inserting.

  // my $blocktag  = 'ltx:block';
  // my $iblocktag = 'ltx:inline-block';
  // if ($blockattr{para}) {
  //   $blocktag  = 'ltx:para';
  //   $iblocktag = 'ltx:inline-para';
  //   delete $blockattr{para}; }
  // # Generally, we're going to need some sort of container to hold the contents of the block.
  // # Particularly if we're: setting various size & positioning attributes;
  // # or can't currently open an ltx:p; or if the current point accepts plain text (#PCDATA).
  // # If we're in an inline context, we'll need a ltx:inline-block,  otherwise ltx:block.
  // # [Or maybe an ltx:para... when does that happen?]
  // my $newblock = undef;
  // my $unwrap   = 0;
  // map { ($blockattr{$_} || delete $blockattr{$_}) } keys %blockattr;
  // my $hasattr = scalar(keys %blockattr);
  // if ($hasattr || !$document->canContainSomehow($context, 'ltx:p') || $document->canContain($context, '#PCDATA')) {
  //   my $tag = ($document->canContain($context, $blocktag)
  //     ? $blocktag
  //     : $iblocktag);
  //   $newblock = $document->openElement($tag, '_autoclose' => 1, %blockattr); }
  // ## I think this option isn't really needed.... try to simplify
  // ## elsif ($document->canContainSomehow($context, 'ltx:para')) {
  // ## $newblock = $document->openElement('ltx:para', '_autoclose' => 1, %blockattr); }
  // # Insert the content for the block, and reduce

  // $document->setAttribute($document->getElement, '_vertical_mode_' => 1);    # HACK!!!! (see \hbox)
  // my @nodes = $document->filterChildren($document->filterDeletions($document->absorb($contents)));

  // # Scan the inserted nodes, wrapping sequences of Inline items with a ltx:p
  // my @newnodes = ();
  // while (@nodes) {
  //   if ($model->getNodeQName($nodes[0]) eq 'ltx:break') {    # ltx:break are superflous, now.
  //     $document->removeNode(shift(@nodes));
  //     next; }
  //   my @n;                                                   # Collect up sequences of Inline
  //   while (@nodes && ($model->isInSchemaClass('Inline', $nodes[0]))) {
  //     push(@n, shift(@nodes)); }
  //   if (@n) {
  //     push(@newnodes, $document->wrapNodes('ltx:p', @n)); }
  //   else {
  //     push(@newnodes, shift(@nodes)); } }

  // # If we've inserted a wrapper element, close all open elements up to it's parent
  // # It may have auto-opened some element to contain it, but leave that open for following material
  // # Otherwise, close everything back up to the originally open element (but only if still open!)
  // if ($newblock) {
  //   $document->closeToNode($newblock->parentNode, 1); }
  // else {
  //   $document->closeToNode($context, 1); }
  // # Check if the ltx:inline-block container is really needed.
  // if ($newblock) {
  //   my @rows = $newblock->childNodes;
  //   if (scalar(@rows) < 1) {    # Insertion came up empty?
  //     $document->removeNode($newblock); }    # then remove the new block entirely
  //   elsif ($unwrap ||
  //     ((scalar(@rows) == 1)                  # Else only 1 item inside, then flatten
  //       && $document->canContain($newblock->parentNode, $rows[0])    # if allowed.
  //       && (!$hasattr || !grep { !$document->canHaveAttribute($rows[0], $_) } keys %blockattr))) {
  //     map { $document->setAttribute($rows[0], $_ => $blockattr{$_}) } keys %blockattr;
  //     $document->unwrapNodes($newblock); } }

  // # And return the list of "rows" in the box (in case they need attributes....)
  // return @newnodes; }
}