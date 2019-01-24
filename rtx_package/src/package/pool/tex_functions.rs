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
