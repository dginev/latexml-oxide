use crate::package::*;
use libxml::tree::{Node, NodeType};
use rtx_core::keyvals::KeyvalsConfig;
use std::char::{decode_utf16, REPLACEMENT_CHARACTER};
use std::collections::VecDeque;


// TODO:
// sub isDefinable {
//   my ($token) = @_;
//   return unless $token;
//   my $meaning = LookupMeaning($token);
//   my $name    = $token->getString; $name =~ s/^\\//;
//   return (((!defined $meaning) || ($meaning eq LookupMeaning(T_CS('\relax')))
//         || LookupValue('2.09_COMPATIBILITY'))    # Let redefinitions happen in compatibility mode.
//       && (($name ne 'relax') && ($name !~ /^end/))); }

pub fn reenter_text_mode(vertical_mode: bool, gullet: &mut Gullet, state: &mut State) {
  let mut bindings: VecDeque<Stored> = match state.lookup_value(if vertical_mode {
    "VTEXT_MODE_BINDINGS"
  } else {
    "HTEXT_MODE_BINDINGS"
  }) {
    Some(Stored::VecDequeStored(ref vdq)) => vdq.clone(),
    _ => VecDeque::new(),
  };
  if let Some(Stored::VecDequeStored(ref text_mode_bindings)) =
    state.lookup_value("TEXT_MODE_BINDINGS")
  {
    bindings.extend(text_mode_bindings.clone());
  }
  for binding in bindings {
    if let Stored::Tokens(tks) = binding {
      let mut vec = tks.unlist();
      let new = vec.remove(1);
      state.let_i(&vec[0], new, None, gullet);
    }
  }
}

// Similarly, for metadata appearing within peculiar environments, fonts, etc
// You'll typically want this within a group or bounded=>1.
pub fn neutralize_font(state: &mut State) {
  state.assign_value("font", Font::text_default(), Some(Scope::Local));
  state.assign_value("mathfont", Font::math_default(), Some(Scope::Local));
}

pub fn today(state: &mut State) -> String {
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
  let month =
    month_names[state.lookup_register("\\month", vec![]).unwrap().value_of() as usize - 1];
  let day = state.lookup_register("\\day", vec![]).unwrap().value_of();
  let year = state.lookup_register("\\year", vec![]).unwrap().value_of();
  s!("{} {}, {}", month, day, year)
}

pub fn parse_def_parameters(
  cs: &Token,
  params_in: Tokens,
  state: &mut State,
) -> Result<Option<Parameters>> {
  let mut tokens: VecDeque<Token> = if params_in.is_empty() {
    VecDeque::new()
  } else {
    VecDeque::from(params_in.pack_parameters().unlist())
  };
  // Now, recognize parameters and delimiters.
  let mut params = Vec::new();
  let mut n = 0;
  while let Some(mut t) = tokens.pop_front() {
    let cc = t.get_catcode();
    if cc == Catcode::PARAM || cc == Catcode::ARG {
      if cc == Catcode::PARAM {
        if tokens.is_empty() {
          // Special case: lone # NOT following a numbered parameter
          // Note that we require a { to appear next, but do NOT read it!
          params.push(Parameter::new(
            Cow::Borrowed("RequireBrace"),
            Cow::Borrowed("RequireBrace"),
            state,
          )?);
          break;
        } else {
          n += 1;
          if let Some(t_next) = tokens.pop_front() {
            t = t_next;
          } else {
            unimplemented!(); // hm, this is a bit of a pain to port without making t into an
                              // Option<Token>...
          }
        }
      } else {
        // CC_ARG case, keep looking at this token
        n += 1;
      }
      if n > 0 {
        let t_num = t.with_str(|ts| ts.parse::<i8>()).unwrap_or(-1);
        if t_num != n {
          fatal!(
            ParamSpec,
            Expected,
            s!(
              "Parameters for {:?} not in order. Got {:?}, expected {:?}. in {:?}",
              cs,
              t,
              n,
              params
            )
          );
        }
      }
      // Check for delimiting text following the parameter #n
      let mut delim = Vec::new();
      let mut pc = Catcode::MARKER; // throwaway initial val
      while !tokens.is_empty() {
        let inner_cc = tokens.front().unwrap().get_catcode();
        if inner_cc == Catcode::PARAM || inner_cc == Catcode::ARG {
          break;
        }
        let d = tokens.pop_front().unwrap();
        if !(pc == Catcode::SPACE && inner_cc == Catcode::SPACE) {
          // BUT collapse whitespace!
          delim.push(d);
        }
        pc = inner_cc;
      }
      // Found text that marks the end of the parameter
      if !delim.is_empty() {
        let extra = Tokens::new(delim);
        params.push(
          Parameter {
            name: Cow::Borrowed("Until"),
            spec: Cow::Owned(format!("Until:{extra}")),
            extra: vec![extra],
            ..Parameter::default()
          }
          .init(state)?,
        );
      } else if tokens.len() == 1 && tokens.front().unwrap().get_catcode() == Catcode::PARAM {
        // Special case: trailing sole # => delimited by next opening brace.
        tokens.pop_front();
        params.push(Parameter::new(
          Cow::Borrowed("UntilBrace"),
          Cow::Borrowed("UntilBrace"),
          state,
        )?);
      } else {
        // Nothing? Just a plain parameter.
        params.push(Parameter::new(
          Cow::Borrowed("Plain"),
          Cow::Borrowed("{}"),
          state,
        )?);
      }
    } else {
      // Initial delimiting text is required.
      let mut lit: Vec<Token> = vec![t];
      while !tokens.is_empty() {
        let lit_cc = tokens.front().unwrap().get_catcode();
        if lit_cc == Catcode::PARAM || lit_cc == Catcode::ARG {
          break;
        }
        lit.push(tokens.pop_front().unwrap());
      }
      let expected = Tokens::new(lit);
      params.push(
        Parameter {
          name: Cow::Borrowed("Match"),
          spec: Cow::Owned(s!("Match:{}", expected)),
          extra: vec![expected],
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
    Ok(Some(Parameters::new(params)))
  }
}

pub fn do_def(
  globally: bool,
  stomach: &mut Stomach,
  cs: Token,
  params: Tokens,
  body: Tokens,
  state: &mut State,
) -> Result<()> {
  BindState!(stomach, state);
  let paramlist = parse_def_parameters(&cs, params, state)?;

  let scope = if globally { Some(Scope::Global) } else { None };
  state.install_definition(
    Expandable::new(
      cs,
      paramlist,
      ExpansionBody::Tokens(body),
      Some(ExpandableOptions {
        nopack_parameters: true,
        ..ExpandableOptions::default()
      }),
      state,
    ),
    scope,
  );
  state.after_assignment(stomach.get_gullet_mut());
  Ok(())
}

// Kinda rough: We don't really keep track of modes as carefully as TeX does.
// We'll assume that a box is horizontal if there's anything at all,
// but it's not a vbox (!?!?)
pub fn classify_box(boxnum: Number, state: &State) -> &'static str {
  match state.lookup_value(&s!("box{}", boxnum.value_of())) {
    Some(Stored::Digested(ref d)) => match d.data() {
      DigestedData::Whatsit(ref w)
        if w.borrow().definition == state.lookup_definition(&T_CS!("\\vbox")).unwrap() =>
      {
        "vbox"
      },
      _ => "hbox",
    },
    _ => "",
  }
}

const MATH_CLASS_ROLE: [&str; 8] = ["", "BIGOP", "BINOP", "RELOP", "OPEN", "CLOSE", "PUNCT", ""];
// Is this "fontinfo" stuff sufficient to maintain a math font "family" ??
// What we're really after is a connectio nto a font encoding mapping.
pub fn decode_math_char(
  mut n: u16,
  stomach: &mut Stomach,
  state: &mut State,
) -> (Option<String>, Option<char>) {
  let class: u16 = n / (16 * 256);
  n %= 16 * 256;
  let fam: u16 = n / 256;
  n %= 256;
  let font = state
    .lookup_value(&s!("fontinfo_{}_text", fam))
    .unwrap_or_else(|| {
      state
        .lookup_value(&s!("fontinfo_{}_script", fam))
        .unwrap_or_else(|| {
          state
            .lookup_value(&s!("fontinfo_{}_scriptscript", fam))
            .unwrap_or(&Stored::Bool(false))
        })
    });
  // TODO: This function is called with n=20,000, how is the char cast sensible here? Consult Bruce.
  // TODO: confusing types, the 256 arithmetic implies larger than u8 inputs, what for?
  let c = n as u8 as char;
  // // If no specific class, Lookup properties from a DefMath?
  let charinfo = state.lookup_value(&s!("math_token_attributes_{}", c));
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
      font::decode(n as u8, Some(data.to_string()), false, stomach, state)
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
pub fn read_box_contents(
  gullet: &mut Gullet,
  everybox_opt: Option<Tokens>,
  state: &mut State,
) -> Result<Tokens> {
  while let Some(t) = gullet.read_token(state)? {
    if t.get_catcode() == Catcode::BEGIN {
      break;
    } // Skip till { or \bgroup
  }
  // Now, insert some extra tokens, if any, possibly from \afterassignment
  match state.remove_value("BeforeNextBox") {
    Some(Stored::Tokens(tokens)) => gullet.unread(tokens),
    Some(Stored::Token(token)) => gullet.unread_one(token),
    None | Some(Stored::None) => {},
    Some(other) => panic!("afterAssignment should be a token, got: {}", other),
  };
  // AND, insert any extra tokens passed in, due to everyhbox or everyvbox
  if let Some(everybox) = everybox_opt {
    gullet.unread(everybox);
  }
  Ok(Tokens!())
}

/// Reading a Box's content is crucially dependent on invoking the "{" token and obtaining a
/// digested result Hence it is *always* needed to pair `read_box_contents` with its stomach-level
/// counterpart, `predigest_box_contents`
pub fn predigest_box_contents(
  stomach: &mut Stomach,
  _tokens: ArgWrap,
  state: &mut State,
) -> Result<Option<Digested>> {
  let mut contents = stomach.invoke_token(&T_BEGIN!(), state)?;
  if contents.is_empty() {
    Ok(None)
  } else {
    Ok(Some(contents.remove(0)))
  }
}

pub fn revert_spec(_whatsit: &mut Whatsit, _keyword: &str, _state: &mut State) -> Vec<Token> {
  //   my ($whatsit, $keyword) = @_;
  //   my $value = $whatsit->getProperty($keyword);
  //   return ($value ? (Explode($keyword), Revert($value)) : ()); }
  unimplemented!()
}

pub fn p_revert<T>(arg: T, state: &mut State) -> Result<Tokens>
where T: Sized + Object {
  state.set_dual_branch("presentation");
  let result = arg.revert(state);
  state.expire_dual_branch();
  result
}

pub fn c_revert<T>(arg: T, state: &mut State) -> Result<Tokens>
where T: Sized + Object {
  state.set_dual_branch("content");
  let result = arg.revert(state);
  state.expire_dual_branch();
  result
}

/// This attempts to be a generalize vbox construction;
/// It tries to figure out whether an ltx:inline-block or ltx:para is needed,
/// and attempts to figure out whether sequences of the inserted content
/// need to be explicitly wrapped in some kind of block element (presumably ltx:p).
/// It returns the inserted inner blocks,
/// whether or not they got wrapped by that ltx:inline-block; which it DOESN'T TELL YOU ABOUT!
pub fn insert_block(
  document: &mut Document,
  contents: &Digested,
  mut blockattr: HashMap<String, String>,
  state: &mut State,
) -> Result<Vec<Node>> {
  // Create something like:
  // "<ltx:inline-block vattach='$vattach' height='#height'>#2</ltx:inline-block>"
  let context = document.get_element().unwrap(); // Where we originally start inserting.

  let mut blocktag = "ltx:block";
  let mut iblocktag = "ltx:inline-block";
  if blockattr.remove("para").is_some() {
    blocktag = "ltx:para";
    iblocktag = "ltx:inline-para";
  }
  // Generally, we're going to need some sort of container to hold the contents of the block.
  // Particularly if we're: setting various size & positioning attributes;
  // or can't currently open an ltx:p; or if the current point accepts plain text (#PCDATA).
  // If we're in an inline context, we'll need a ltx:inline-block,  otherwise ltx:block.
  // [Or maybe an ltx:para... when does that happen?]
  let mut newblock: Option<Node> = None;
  let mut remove = Vec::new();
  // drop all empty values
  for (key, val) in &blockattr {
    if val.is_empty() {
      remove.push(key.to_string())
    }
  }
  for key in remove {
    blockattr.remove(&key);
  }
  let hasattr = !blockattr.is_empty();
  if hasattr
    || !document.can_contain_node_somehow(&context, "ltx:p", state)
    || document.can_contain(&context, "#PCDATA", state)
  {
    let tag = if document.can_contain(&context, blocktag, state) {
      blocktag
    } else {
      iblocktag
    };
    let mut attr_arg = blockattr.clone();
    attr_arg.insert("_autoclose".to_string(), "true".to_string());
    newblock = Some(document.open_element(tag, Some(attr_arg), None, state)?);
  }
  // Insert the content for the block, and reduce
  document.set_attribute(
    &mut document.get_element().unwrap(),
    "_vertical_mode_",
    "true",
    state,
  )?; // HACK!!!! (see \hbox)

  document.absorb(contents, None, state)?;
  let absorbed: Vec<Node> = document.get_constructed_nodes().to_vec();
  let mut nodes = VecDeque::from(document.filter_children(document.filter_deletions(absorbed)));

  // Scan the inserted nodes, wrapping sequences of Inline items with a ltx:p
  let mut newnodes = Vec::new();
  while !nodes.is_empty() {
    let qname = document.get_node_qname(&nodes[0], state);
    if qname == arena::pin_static("ltx:break") {
      // ltx:break are superflous, now, unless we're transporting a figure/float
      let bp_name = document.get_node_qname(&nodes[0].get_parent().unwrap(), state);
      if bp_name != arena::pin_static("ltx:figure") && bp_name != arena::pin_static("ltx:float") {
        document.remove_node(&mut nodes.pop_front().unwrap());
        continue;
      }
    }
    let mut inline = Vec::new(); // Collect up sequences of Inline
    while !nodes.is_empty() && state.model.is_node_in_schema_class("Inline", &nodes[0]) {
      inline.push(nodes.pop_front().unwrap());
    }
    if !inline.is_empty() {
      if let Some(wrapped) = document.wrap_nodes("ltx:p", inline, state)? {
        newnodes.push(wrapped);
      }
    } else {
      newnodes.push(nodes.pop_front().unwrap());
    }
  }

  // If we've inserted a wrapper element, close all open elements up to it's parent
  // It may have auto-opened some element to contain it, but leave that open for following material
  // Otherwise, close everything back up to the originally open element (but only if still open!)
  if let Some(ref blocknode) = newblock {
    let block_parent = blocknode.get_parent();
    document.close_to_node(block_parent.as_ref().unwrap(), true, state)?;
  } else {
    document.close_to_node(&context, true, state)?;
  }
  // Check if the ltx:inline-block container is really needed.
  if let Some(mut blocknode) = newblock {
    let mut rows = blocknode.get_child_nodes();
    let mut crows = match rows.first() {
      None => VecDeque::new(),
      Some(n) => VecDeque::from(n.get_child_nodes()),
    };
    if rows.is_empty() {
      // Insertion came up empty?
      document.remove_node(&mut blocknode); // then remove the new block entirely
    } else if rows.len() == 1
      && crows.len() == 1
      && state
        .model
        .with_node_qname(rows.first().unwrap(), |qname| qname == "ltx:p")
      && document.can_contain_sym(
        &blocknode.get_parent().unwrap(),
        state.model.get_node_qname(&crows[0]),
        state,
      )
    // TODO: && (!hasattr || blockattr.keys().any(...
    {
      // Else only 1 item inside...which is an ltx:p with 1 item, if allowed.
      let mut cfirst = crows.pop_front().unwrap();
      for (key, val) in blockattr {
        document.set_attribute(&mut cfirst, &key, &val, state)?;
      }
      document.unwrap_nodes(rows.remove(0))?;
      document.unwrap_nodes(blocknode)?;
    } else if rows.len() == 1
      && document.can_contain_sym(
        &blocknode.get_parent().unwrap(),
        state.model.get_node_qname(&rows[0]),
        state,
      )
    // if allowed.
    // TODO: && (!hasattr || !grep { !$document->canHaveAttribute($rows[0], $_) } keys %blockattr)))
    {
      let mut first = rows.remove(0);
      for (key, val) in blockattr {
        document.set_attribute(&mut first, &key, &val, state)?;
      }
      document.unwrap_nodes(blocknode)?;
    }
  }
  // And return the list of "rows" in the box (in case they need attributes....)
  Ok(newnodes)
}

pub fn cleanup_math(document: &mut Document, mathnode: Node, state: &mut State) -> Result<()> {
  // Cleanup ltx:Math elements; particularly if they aren't "really" math.
  // But record the oddity with class=ltx_markedasmath

  // If the Math ONLY contains XMath/XMText, it apparently isn't math at all!?!
  if document
    .findnodes(
      "ltx:XMath/ltx:*[local-name() != 'XMText']",
      Some(&mathnode),
      state,
    )
    .is_empty()
  {
    // So unwrap down to the contents of the XMText's.
    let xmtexts = mathnode.get_child_nodes().into_iter().flat_map(|child| {
      child
        .get_child_nodes()
        .into_iter()
        .flat_map(|grandhcild| grandhcild.get_child_nodes())
    });
    let mut texts = vec![];
    for mut text in xmtexts {
      text = if text.get_type() == Some(NodeType::ElementNode) {
        // Make sure we've got an element
        text
      } else {
        document.wrap_nodes("ltx:text", vec![text], state)?.unwrap()
      };
      // Now record that it originally was marked as math
      document.add_class(&mut text, "ltx_markedasmath", state)?;
      texts.push(text)
    }
    document.replace_node(mathnode.clone(), texts)?; // and replace the whole Math with the pieces
  } else {
    // Cleanup any remaining XMTexts
    cleanup_xmtext_outer(document, &mathnode, state)?;
  }
  Ok(())
}

// Here's for an inverse case: when an XMText isn't "really" just text
// if it only contains an Math  ORR, a tabular with only Math in the cells?
// First case: pull it back into the math, but in an XMWrap to isolate it for parsing.
// Should we just pull any mixed text math up or only a single Math?
// For the tabular case, convert it to an XMArray.

// Note that normally, we'd do afterClose on ltx:XMText,
// but since the ltx:XMText closes before the outer ltx:Math,
// we would keep cleanup_Math from recognizing the trivial case of
// a single ltx:tabular in an equation (perverse, but people do that).
// So, we put this one on ltx:Math also, and scan for any contained XMText to fixup.

fn cleanup_xmtext_outer(
  document: &mut Document,
  math_node: &Node,
  state: &mut State,
) -> Result<()> {
  for text_node in document.findnodes("descendant::ltx:XMText", Some(math_node), state) {
    cleanup_xmtext(document, text_node, state)?;
  }
  Ok(())
}

fn cleanup_xmtext(document: &mut Document, mut text_node: Node, state: &mut State) -> Result<()> {
  // We're really only interested in reducing nested math, right?
  // But actually also collapsing ltx:XMText/ltx:text
  // Apply "outer" simplifications: remove ltx:text or ltx:p wrappings.

  // A single "simple" element, with a single child
  let mut children;
  loop {
    children = text_node.get_child_nodes();
    if (children.len() != 1)
      || document
        .findnodes(
          "ltx:text | ltx:inline-block[count(*)=1] | ltx:p",
          Some(&text_node),
          state,
        )
        .is_empty()
    {
      break;
    }
    let child = children.pop().unwrap();
    document.copy_node_font(&child, &mut text_node)?;
    for (key, value) in child.get_attributes() {
      // Copy the child's attributes (should Merge!!)
      if key != "xml:id" {
        text_node.set_attribute(&key, &value)?;
      }
    }
    document.unwrap_nodes(child)?;
  }

  // Now apply a simplifying rule for nested Math
  // If the XMText contains a single Math, pull it's content up in
  if children.len() == 1
    && !document
      .findnodes("ltx:Math", Some(&text_node), state)
      .is_empty()
  {
    // Replace XMText by XMWrap/*  (this should preserve the parse?)
    document.rename_node(&mut text_node, "ltx:XMWrap")?; // text_node =
    let first_child = children.pop().unwrap();
    let first_granchildren = first_child.get_child_nodes();
    document.replace_node(
      first_child,
      first_granchildren
        .into_iter()
        .flat_map(|grandchild| grandchild.get_child_nodes())
        .collect(),
    )?;
  // # # RISKY!!!! If SOME nodes are math...
  // # # pull the whole sequence up, unwrap the math and putting the rest back in XMText.
  // # # Even with the XMWrap, this seems to wreak havoc on parsing and structure?
  // # if(document.findnodes('ltx:Math',$text_node)){
  // #   # Replace XMText by XMWrap/*  (this should preserve the parse?)
  // #   $text_node=document.renameNode($text_node,'ltx:XMWrap');
  // #   foreach my $child (@children){
  // #     if($model->getNodeQName($child) eq 'ltx:Math'){
  // #       document.replaceNode($child,map($_->childNodes,$child->childNodes)); }
  // #     else {
  // #       document.wrapNodes('ltx:XMText',$child); }}}
  // If a single tabular that ONLY(?) contains Math, turn into an XMArray
  // Well, a tabular REALLY shouldn't be in math;
  // How much math should determine the switch?
  // [will alignment attributes be lost?]
  } else if children.len() == 1
    && state
      .model
      .with_node_qname(children.first().as_ref().unwrap(), |qname| {
        qname == "ltx:tabular"
      })
  //// Should we ALWAYS do this, or just for some minimal amount of math???
  ////        && !document.findnodes('ltx:tabular/ltx:tr/ltx:td/text()'
  ////                                 .' | ltx:tabular/ltx:tbody/ltx:tr/ltx:td/text()'
  ////                                 .' | ltx:tabular/ltx:tr/ltx:td[not(ltx:Math)]'
  ////                                 .' | ltx:tabular/ltx:tbody/ltx:tr/ltx:td[not(ltx:Math)]',
  ////                                 $text_node)
  {
    unimplemented!(); // TODO
                      // // First step is remove any ltx:tbody from the tabular!
                      // foreach my $tb (document.findnodes('ltx:tabular/ltx:tbody', $text_node)) {
                      //   document.unwrapNodes($tb); }
                      // // Now, we can start replacing tabular=>XMArray, tr=>XMRow, td=>XMCell
                      // my $table = document.renameNode($children[0], 'ltx:XMArray');
                      // foreach my $row ($table->childNodes) {
                      //   $row = document.renameNode($row, 'ltx:XMRow');
                      //   foreach my $cell ($row->childNodes) {
                      //     $cell = document.renameNode($cell, 'ltx:XMCell');
                      //     foreach my $m ($cell->childNodes) {
                      //       if ($model->getNodeQName($m) eq 'ltx:Math') {    // Math cell, unwrap
                      // the Math/XMath layer         document.replaceNode($m,
                      // map { $_->childNodes } $m->childNodes); }       else
                      // {                                           // Otherwise, wrap whatever it
                      // is in an XMText         document.wrapNodes('ltx:
                      // XMText', $m); } } } }
                      // And now we don't need the XMText any more.
                      // foreach my $attr ($text_node->attributes) {    // Copy the child's
                      // attributes (should Merge!!)
                      //   $table->setAttribute($attr->nodeName => $attr->getValue); }
                      // my $newtable = document.unwrapNodes($text_node);
                      // if (my $id = $text_node->getAttribute('xml:id')) {
                      //   document.unRecordID($id);
                      //   document.recordID($id, $newtable); } }
  }
  Ok(())
}

//======================================================================
// A random collection of utility functions.
// [maybe need to do some reorganization?]
// Since this is used for textual tokens, typically to split author lists,
// we don't split within braces or math
#[allow(clippy::while_let_on_iterator)]
pub fn split_tokens(tokens: Tokens, delims: Vec<Token>) -> Vec<Tokens> {
  let mut items = Vec::new();
  let mut toks = Vec::new();
  if !tokens.is_empty() {
    let tokens = tokens.unlist();
    let mut tokens_iter = tokens.into_iter();
    while let Some(t) = tokens_iter.next() {
      if delims.iter().any(|d| d == &t) {
        items.push(Tokens::new(toks.drain(..).collect()));
      } else if t == T_BEGIN!() {
        toks.push(t);
        let mut level = 1;
        while let Some(t) = tokens_iter.next() {
          match t.get_catcode() {
            Catcode::BEGIN => level += 1,
            Catcode::END => level -= 1,
            _ => {},
          }
          toks.push(t);
          if level < 1 {
            // done if balanced.
            break;
          }
        }
      } else if t == T_MATH!() {
        toks.push(t);
        while let Some(t) = tokens_iter.next() {
          let is_math = t.get_catcode() == Catcode::MATH;
          toks.push(t);
          if is_math {
            break;
          }
        }
      } else {
        toks.push(t);
      }
    }
    // last author is in toks, add to items
    items.push(Tokens::new(toks));
  }
  items
}

pub fn and_split(cs: Token, tokens: Tokens) -> Vec<Token> {
  split_tokens(tokens, vec![T_CS!("\\and")])
    .into_iter()
    .flat_map(|t| {
      let mut with_cs = vec![cs.clone(), T_BEGIN!()];
      with_cs.extend(t.unlist());
      with_cs.push(T_END!());
      with_cs
    })
    .collect()
}

/// Converts $tokens to a string in the fashion of \message and others:
/// doubles #, converts to string; optionally adds spaces after control sequences
/// in the spirit of the B Book, "show_token_list" routine, in 292.
pub fn writable_tokens(tokens: &Tokens, _state: &mut State) -> String {
  // unwrap a \noexpand-created \relax to its actual content,
  // to avoid confusing users with a \relax dontexpand
  let mut wv = Vec::new();
  for t in tokens.unlist_ref().iter() {
    let t = t.without_dont_expand_ref();
    match t.code {
      Catcode::CS => {
        wv.push(t.clone());
        wv.push(T_SPACE!());
      },
      Catcode::SPACE => {
        wv.push(T_SPACE!());
      },
      Catcode::PARAM => {
        wv.push(t.clone());
        wv.push(t.clone());
      },
      Catcode::ARG => {
        // B Book, 294. Reduce to param+integer
        wv.push(T_PARAM!());
        wv.push(t.as_other());
      },
      _ => {
        wv.push(t.clone());
      },
    }
  }
  Tokens::new(wv).untex()
}

// sub orNull {
//   return (grep { defined } @_) ? @_ : undef; }

// # Should be a general utility?
// sub stripBraces {
//   my ($tokens) = @_;
//   my @tokens = ($tokens ? $tokens->unlist : ());
//   my @t = ();
//   while (@tokens && ($tokens[0]->getCatcode == CC_SPACE)) {    # Skip leading whitespace
//     shift(@tokens); }
//   # Balanced tokens until $delim
//   my $ntopbraces = 0;
//   while (@tokens) {
//     if (Equals($tokens[0], T_BEGIN)) {                         # If top-level brace
//       $ntopbraces++;
//       my ($level, $t) = (0, undef);
//       while (defined($t = shift(@tokens))) {                   # Read balanced
//         my $cc = $t->getCatcode;
//         $level++ if $cc == CC_BEGIN;
//         $level-- if $cc == CC_END;
//         push(@t, $t);
//         last unless $level; } }
//     else {
//       push(@t, shift(@tokens)); } }
//   while (@t && ($t[-1]->getCatcode == CC_SPACE)) {             # pop off trailing spaces
//     pop(@t); }
//   # Strip outer braces if a single set encloses entire value and not just {}
//   if ($ntopbraces == 1) {
//     shift(@t); pop(@t); }
//   return Tokens(@t); }

/// Support for Key / Value arguments.
// The very basic form is
//   RequiredKeyVals: $keyset
//   OptionalKeyVals: $keyset
// to parse Key-Value pairs from a given keyset (see the 'keyval' package
// documentation for more information). These types of KeyVal
// parameters will return a LaTeXML::Core::KeyVals object, which can then be
// used to access the values of the individual items.
// The difference between the two forms is that RequiredKeyVals expects a set of
// key-value pairs wrapped in T_BEGIN T_END, where as OptionalKeyVals optionally
// expects a set of KeyValue pairs wrapped in T_OTHER('[') T_OTHER(']')
//
// Several extension of the keyval package exist, the most common one we support
// is the xkeyval package. This introduces further variations on the keyval
// arguments parsing, in particular it allows to read keys from more than one
// keyset at once. These can be specified by giving comma-seperated values in
// the keyset argument. By default, a key will only be set in the **first**
// keyset it occurs in. By using
//   RequiredKeyVals+: $keysets
//   OptionalKeyVals+: $keysets
// the key will be set in all keysets instead.
//
// All keys to be parsed with these arguments should be declared using
// DefKeyVal in LaTeXML::Package. By default, an error is thrown if an unknown
// key is encountered. To surpress this behaviour, and instead store all
// undefined keys, use
//   RequiredKeyVals*: $keysets
//   OptionalKeyVals*: $keysets
// instead. The '*' and '+' modifiers can be combined by using:
//   RequiredKeyVals*+: $keysets
//   OptionalKeyVals*+: $keysets
//
// Furthermore, the xkeyval package supports giving prefixes to keys,
//   RequiredKeyVals[*][+]: $prefix|$keysets
//   OptionalKeyVals[*][+]: $prefix|$keysets
//
// Finally, it is possible to specify specific keys to skip when digesting the
// object. This can be achieved using comma-seperated key values in
//   RequiredKeyVals[*][+]: $prefix|$keysets|$skip
//   OptionalKeyVals[*][+]: $prefix|$keysets|$skip

// function to handle all the
#[derive(Default)]
pub struct KVSpec {
  pub star: bool,
  pub plus: bool,
  pub prefix: Option<String>,
  pub keysets: Vec<Option<Parameters>>,
  pub skip: bool,
}
pub fn keyvals_aux(
  gullet: &mut Gullet,
  until: Option<Token>,
  mut spec: KVSpec,
  state: &mut State,
) -> Result<KeyVals> {
  // support both "keysets" and "prefix|keysets"
  // unless (defined($keysets)) {
  //   $keysets = $prefix;
  //   $prefix  = undef;
  // to emulate old behaviour, throw no errors
  // when we have a single keyset and no prefix (or no keyset at all)
  if spec.keysets.is_empty() {
    spec.star = true;
  } else if let Some(ref prefix) = spec.prefix {
    if prefix.find(',').is_none() {
      spec.star = true;
    }
  }

  // create a new set of Key-Value arguments
  let mut keyvals = KeyVals::new(
    KeyvalsConfig {
      prefix: spec.prefix,
      // keysets: spec.keysets, // TODO!
      keysets: Vec::new(),
      set_all: spec.plus,
      set_internals: true,
      skip: spec.skip,
      skip_missing: spec.star,
    },
    state,
  );
  // and read it from the gullet
  if let Some(until_token) = until {
    keyvals.read_from(gullet, until_token, state)?;
  }
  // we still want to make use of the hash
  Ok(keyvals)
}

pub fn uppercase_token(token: Token, state: &State) -> Token {
  either_case_token(token, true, state)
}
pub fn lowercase_token(token: Token, state: &State) -> Token {
  either_case_token(token, false, state)
}

fn either_case_token(token: Token, is_upper: bool, state: &State) -> Token {
  let (chars_count, thischar) = token.with_str(|s| (s.chars().count(), s.chars().next()));
  // DG: new idea, short-circuit if more than 1 char, since our lccode/uccode tables are single
  // char-based (for now?)
  if chars_count != 1 {
    return token;
  }
  let mut result = String::new();
  let cased = if is_upper {
    state.lookup_uccode(thischar.unwrap())
  } else {
    state.lookup_lccode(thischar.unwrap())
  };
  if let Some(code) = cased {
    if code != 0 {
      result.push_str(
        &decode_utf16([code])
          .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
          .collect::<String>(),
      )
    } else {
      result.push(thischar.unwrap());
    }
  } else {
    result.push(thischar.unwrap());
  }
  if token.with_str(|initial_str| initial_str != result) {
    Token::new(result, token.get_catcode())
  } else {
    token
  }
}

/// a candidate for use by \hskip, \hspace, etc... ?
pub fn dimension_to_spaces(dimen: &Dimension, state: &State) -> Cow<'static, str> {
  let fs = state.lookup_font().unwrap().get_size(); // 1 em
  let pt = dimen.pt_value(None);
  let ems = pt / fs.unwrap();
  if ems < 0.01 {
    Cow::Borrowed("")
  } else if ems < 0.17 {
    Cow::Borrowed("\u{2006}")
  }
  // 6em
  else if ems < 0.30 {
    Cow::Borrowed("\u{2005}")
  }
  // 4em
  else if ems < 0.40 {
    Cow::Borrowed("\u{2004}")
  }
  // 3em (same as nbsp?)
  else {
    let n = (ems + 0.3 / 0.333).trunc() as usize; // 10pts per space...?
    Cow::Owned("\u{00A0}".repeat(n))
  }
}

pub fn aligning_environment(
  align: &str,
  class: &str,
  document: &mut Document,
  props: &HashMap<String, Stored>,
  state: &mut State,
) -> Result<()> {
  if let Some(Stored::Digested(body)) = props.get("body") {
    // Add class attribute to new nodes.
    for mut node in insert_block(document, body, HashMap::default(), state)?.into_iter() {
      set_align_or_class(document, &mut node, align, class, state)?;
    }
  }
  Ok(())
}

pub fn set_align_or_class(
  document: &mut Document,
  node: &mut Node,
  align: &str,
  class: &str,
  state: &mut State,
) -> Result<()> {
  let qname = state.model.get_node_qname(node);
  if qname == arena::pin_static("ltx:tag") {
  }
  // HACK
  else if !align.is_empty()
    && state
      .model
      .can_have_attribute(qname, arena::pin_static("align"))
  {
    node.set_attribute("align", align)?;
  } else if !class.is_empty()
    && state
      .model
      .can_have_attribute(qname, arena::pin_static("class"))
  {
    document.add_class(node, class, state)?;
  }
  Ok(())
}

pub fn make_generic_message(
  cmd: &str,
  args: Vec<Tokens>,
  kind: &str,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<()> {
  stomach.bgroup(state);
  state.let_i(
    &T_CS!("\\protect"),
    T_CS!("\\string"),
    None,
    stomach.get_gullet_mut(),
  );
  state.let_i(
    &T_CS!("\\MessageBreak"),
    T_CS!("\\ltx@hard@MessageBreak"),
    None,
    stomach.get_gullet_mut(),
  ); // tricky, we need Expand() to execute it
  let mut message = String::new();
  for arg in args.into_iter() {
    let mut arg_toks = arg.unlist();
    arg_toks.push(T_CS!("\\MessageBreak"));
    let gullet = stomach.get_gullet_mut();
    let arg_str = Expand!(arg_toks, gullet, state).to_string();
    message.push_str(&arg_str);
  }

  stomach.egroup(state)?;
  //   return ('latex', $cmd, $stomach, $message);
  match kind {
    "error" => {
      Error!("latex", cmd, stomach, state, message);
    },
    "warn" => {
      Warn!("latex", cmd, stomach, state, message);
    },
    "info" => {
      Info!("latex", cmd, stomach, state, message);
    },
    _other => panic!("Only call make_generic_message with error|warn|info message kinds."),
  };
  Ok(())
}

/// Utility, not really TeX, but used by LaTeX, AmSTeX...
/// Convert a vertical positioning, optional argument.
///  t = "top", b = "bottom"; default is "middle".
/// Note that the default for vattach attribute is "baseline".
pub fn translate_attachment<T: ToString>(pos: T) -> &'static str {
  //implementor note:
  //  T: AsRef<str> would be more efficient than allocating a string every time
  //  but we first need `Stored` and `Digested` to be capable of that.
  match pos.to_string().as_str() {
    "t" => "top",
    "b" => "bottom",
    _ => "middle",
  } // undef meaning 'baseline'
}
