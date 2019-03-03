use log::*;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use rtx_core::common::error::*;
use rtx_core::common::font::Font;
use rtx_core::document::resource::*;
use rtx_core::document::tag::{TagOptionName, TagOptions};
use rtx_core::gullet::Gullet;
use rtx_core::mouth::{Mouth, MouthOptions};
use rtx_core::parameter::{Parameter, ParameterExtra, Parameters};
use rtx_core::state::{Scope, State, Stored};
use rtx_core::stomach::Stomach;
use rtx_core::token::*;
use rtx_core::tokens::Tokens;
use rtx_core::util::pathname;
use rtx_core::util::pathname::PathnameFindOptions;
use rtx_core::{Core, Digested};

use crate::package::pool;
use super::*;
use super::def_dialect::def_macro;

pub fn load_external_binding(file: &str, state: &mut State, mut with_stomach: Option<&mut Stomach>) -> Result<bool> {
  let taken_dispatcher = state.extra_bindings_dispatch.take();
  match taken_dispatcher {
    Some(ref dispatcher) => {
      let result_opt = match with_stomach {
        None => dispatcher(&file, state, None),
        Some(ref mut st) => dispatcher(&file, state, Some(st)),
      };
      match result_opt {
        Some(result) => match result {
          Ok(()) => true,
          Err(e) => return Err(e),
        },
        None => false,
      }
    },
    None => false,
  };

  let is_contrib: bool = match taken_dispatcher {
    Some(ref dispatcher) => {
      note_begin(&s!("Loading {:?} definitions", file));
      let result_opt = match with_stomach {
        None => dispatcher(&file, state, None),
        Some(ref mut st) => dispatcher(&file, state, Some(st)),
      };
      note_end(&s!("Loading {:?} definitions", file));
      match result_opt {
        Some(result) => match result {
          Ok(()) => true,
          Err(e) => return Err(e),
        },
        None => false,
      }
    },
    None => false,
  };
  state.extra_bindings_dispatch = taken_dispatcher;

  Ok(is_contrib)
}

/// TODO: Flesh out with the full infrastructure, incremental functionality for now.
pub fn input_definitions(raw_file: &str, options: InputDefinitionOptions, mut state: &mut State) -> Result<()> {
  let name = raw_file.trim();
  // let prevname = if options.handleoptions {
  //   if let state.lookup_definition(&T_CS!("\\@currname")).is_some() {
  //     Digest!(T_CS!("\@currname")).to_string()
  //   }
  // let prevext = options.handleoptions && state.lookup_definition(T_CS!('\@currext')) &&
  // ToString(Digest(T_CS!('\@currext')));

  // Compute the exact name based on the type
  let filename = match options.extension {
    None => name.to_string(),
    Some(ext) => s!("{}.{}",name, ext),
  };
  let as_type = if options.as_class { "cls" } else { options.extension.unwrap_or("") };

  let mut with_stomach = options.with_stomach;
  let loaded_flag = filename.clone() + "_loaded";
  {
    // Only load definitions once
    if let Some(&Stored::Bool(flag)) = state.lookup_value(&loaded_flag) {
      if flag {
        // do nothing if we've loaded before
        return Ok(());
      }
    }
  }

  // Mark as loaded, then process the definitions
  note_begin(&s!("Loading {:?} definitions", filename));
  state.assign_value(&loaded_flag, true, Some(Scope::Global));
  def_macro(T_CS!("\\@currname"),None, Tokens!(Explode!(name)), None, state);
  def_macro(T_CS!("\\@currext"), None, Tokens!(Explode!(as_type)), None, state);

  let is_contrib = match with_stomach {
    None => load_external_binding(&filename, state, None)?,
    Some(ref mut stomach_mut) => load_external_binding(&filename, state, Some(stomach_mut))?,
  };

  if !is_contrib {
    match filename.as_ref() {
      "TeX.pool" => pool::tex::load_definitions(&mut state, with_stomach)?,
      "LaTeX.pool" => pool::latex::load_definitions(&mut state, with_stomach)?,
      "eTeX.pool" => pool::etex::load_definitions(&mut state, with_stomach)?,
      "pdfTeX.pool" => pool::pdftex::load_definitions(&mut state, with_stomach)?,
      "article.cls" => pool::article_cls::load_definitions(&mut state, with_stomach)?,
      "alltt.sty" => pool::alltt_sty::load_definitions(&mut state, with_stomach)?,
      "amsmath.sty" => pool::amsmath_sty::load_definitions(&mut state, with_stomach)?,
      "amsthm.sty" => pool::amsthm_sty::load_definitions(&mut state, with_stomach)?,
      "comment.sty" => pool::comment_sty::load_definitions(&mut state, with_stomach)?,
      "IEEEtran.cls" => pool::ieeetran_cls::load_definitions(&mut state, with_stomach)?,
      "url.sty" => pool::url_sty::load_definitions(&mut state, with_stomach)?,
      "verbatim.sty" => pool::verbatim_sty::load_definitions(&mut state, with_stomach)?,
      "fontenc.sty"  => pool::fontenc_sty::load_definitions(&mut state, with_stomach)?,
      "inputenc.sty"  => pool::inputenc_sty::load_definitions(&mut state, with_stomach)?,
      "textcomp.sty"  => pool::textcomp_sty::load_definitions(&mut state, with_stomach)?,
      other => fatal!(Package, Unknown, s!("TODO: unknown binding {:?}, can't load", other)),
    };
  }
  note_end(&s!("Loading {:?} definitions", filename));
  Ok(())
}

pub fn input_content(core: &mut Core, request: &str) -> Result<()> {
  match find_file(request, None, &mut core.state) {
    // TODO: type => $options{type}, noltxml => 1
    Some(path) => load_tex_content(core, &path),
    None => fatal!(Package, MissingFile, request),
    /* TODO:
     * Error("missing_file", request, state.get_stomach().get_gullet(),
     * "Can't find TeX file "+request, maybeReportSearchPaths(state))) */
  }
}

pub fn input(file: String, gullet: &mut Gullet, state: &mut State) {
  unimplemented!();
}

pub fn load_tex_content(core: &mut Core, path: &str) -> Result<()> {
  // If there is a file-specific declaration file (name_tex.rs), load it first!
  // let namespace = path;
  // state.extra_bindings_dispatch
  if !pathname::is_literaldata(path) {
    let (dir, base, ext) = pathname::split(path);
    load_external_binding(&base, &mut core.state, Some(&mut core.stomach.borrow_mut()))?;
  }
  // TODO: Caching
  // content => LookupValue($pathname . '_contents')

  // Open a mouth for that TeX content
  core.stomach.borrow_mut().get_gullet_mut().open_mouth(
    Mouth::create(
      path,
      MouthOptions {
        notes: true,
        ..MouthOptions::default()
      },
      &mut core.state,
    )?,
    true,
  );
  Ok(())
}

pub fn process_options(stomach: &mut Stomach, state: &mut State) -> Result<()> {
  let currname_token = T_CS!("\\@currname");
  let currext_token = T_CS!("\\@currext");
  let gullet = stomach.get_gullet_mut();
  let name = if state.lookup_definition(&currname_token).is_some() {
    do_expand(currname_token, gullet, state)?.to_string()
  } else { 
    String::new()
  };
  let ext  = if state.lookup_definition(&currext_token).is_some() {
    do_expand(currext_token, gullet, state)?.to_string()
  } else { 
    String::new()
  };
  let empty_vdq = VecDeque::new(); // convenience for unwrapping empty

  let declared_options : VecDeque<Stored> = state.lookup_vecdeque("@declaredoptions").unwrap_or(&empty_vdq).clone();
  let opt_key = dbg!(s!("opt@{}.{}", name, ext));
  let current_options =state.lookup_vecdeque(&opt_key).unwrap_or(&empty_vdq);
  let class_options = state.lookup_vecdeque("class_options").unwrap_or(&empty_vdq);
  // Execute options in declared order (unless \ProcessOptions*)

  // TODO: processing options, not yet supported
  // if ($options{inorder}) {    # Execute options in the order passed in (eg. \ProcessOptions*)
  //   foreach my $option (@class_options) {    # process global options, but no error
  //     if    (executeOption_internal($option))        { }
  //     elsif (executeDefaultOption_internal($option)) { } }
  // for option in current_options.iter() {
  //   if execute_option_internal(option)        { }
  //   else if execute_default_option_internal(option)) { } 
  // } }
  // else {                                    
  let mut requested_options : HashSet<String> = HashSet::new();
  for option in current_options.iter() {
    if let Stored::String(content) = option {
      requested_options.insert(content.to_string());
    }
  }
  for option in class_options.iter() {
    if let Stored::String(content) = option {
      requested_options.insert(content.to_string());
    }
  }
  dbg!(&requested_options);
  dbg!(&declared_options);

  // Execute options in declared order (eg. \ProcessOptions)
  for option in declared_options.iter() {
    if let Stored::String(content) = option {
      if requested_options.contains(content)  {
        requested_options.remove(content); // Remove it, since it's been handled.
        execute_option_internal(content, stomach, state)?; 
      }
    }
  }
  // Now handle any remaining options (eg. default options), in the given order.
  for option in requested_options.iter() {
    execute_default_option_internal(option, stomach, state)?; 
  }
  // Now, undefine the handlers?
  for option in declared_options.iter() {
    state.let_i(&T_CS!(&s!("\\ds@{}", option)), T_CS!("\\relax"), None);
  }
  Ok(())
}


fn execute_option_internal(option: &str, stomach: &mut Stomach, state: &mut State) -> Result<bool> {
  let cs = T_CS!(&s!("\\ds@{}",option));
  if state.lookup_definition(&cs).is_some() {
    def_macro(T_CS!("\\CurrentOption"), None, Tokens!(T_OTHER!(option)), None, state);
    
    let unused = match state.remove_vecdeque("@unusedoptionlist") {
      Some(list) => list.into_iter().filter(|item| if let Stored::String(content) = item { content != option} else { false }).collect(),
      None => VecDeque::new()
    };
    state.assign_value("@unusedoptionlist", Stored::VecDequeStored(unused), None);
    stomach.digest(cs, state)?;
    Ok(true)
  } else {
    Ok(false)
  }
}

fn execute_default_option_internal(option: &str, stomach: &mut Stomach, state: &mut State) -> Result<bool> {
  def_macro(T_CS!("\\CurrentOption"), None, Tokens!(T_OTHER!(option)), None, state);
  stomach.digest(T_CS!("\\default@ds"), state)?;
  Ok(true)
}


pub struct RequireOptions<'a> {
  pub options: Vec<String>,
  pub withoptions: bool,
  pub extension: Option<&'static str>,
  pub as_class: bool,
  pub noltxml: bool,
  pub notex: bool,
  pub raw: bool,
  pub after: bool,
  pub with_stomach: Option<&'a mut Stomach>,
}
impl<'a> Default for RequireOptions<'a> {
  fn default() -> Self {
    RequireOptions {
      options: Vec::new(),
      withoptions: false,
      extension: None,
      as_class: false,
      noltxml: false,
      notex: true,
      raw: false,
      after: false,
      with_stomach: None,
    }
  }
}

/// This (and `FindFile`) needs to evolve a bit to support reading raw .sty (.def, etc) files from
/// the standard texmf directories.  Maybe even use kpsewhich itself (INSTEAD of `pathname_find`
/// ???) Another potentially useful option might be that if we are reading a raw file,
/// perhaps it should just get digested immediately, since it shouldn't contribute any boxes.
pub fn require_package(name: &str, mut options: RequireOptions, state: &mut State) -> Result<()> {
  if options.raw {
    options.raw = false;
    // Warn('deprecated', 'raw', $STATE->getStomach->getGullet,
    //   "RequirePackage option raw is obsolete; it is not needed");
  }

  // We'll usually disallow raw TeX, unless the option explicitly given, or globally set.
  // $options{notex} = 1
  //   if !defined $options{notex} && !LookupValue('INCLUDE_STYLES') && !$options{noltxml};
  if options.extension.is_none() {
    options.extension = Some("sty");
  }
  // TODO: Ideally we want to use the same struct for the RequirePackage options as for the
  // InputDefinitions options
  input_definitions(
    name,
    InputDefinitionOptions {
      extension: options.extension,
      handleoptions: true,
      with_stomach: options.with_stomach,
      // Pass classes options if we have NONE!
      withoptions: options.options,
      ..InputDefinitionOptions::default()
    },
    state,
  )
}

pub fn require_resource(mut resource: Resource, state: &mut State) {
  if resource.name.is_empty() && resource.content.is_empty() {
    warn!(target: "expected:resource", "Resource must have a resource pathname or content; skipping");
    return;
  }
  if resource.mimetype.is_empty() && !resource.name.is_empty() {
    let ext = pathname::extension(&resource.name);
    resource.mimetype = resource_type(&ext);
  }
  if resource.mimetype.is_empty() {
    warn!(target: "expected:mime-type", "Resource must have a mime-type; skipping");
    return;
  }

  // If we've got a document, go ahead & put the resource in.
  // if (state.document.is_some()) {
  //   state.document.as_mut().unwrap().add_resource(resource, resource);
  // } else {
  state.pending_resources.push(resource);
  // }
}

pub fn load_class(name: &str, options: Vec<String>, after: Tokens, with_stomach: Option<&mut Stomach>, state: &mut State) -> Result<()> {
  input_definitions(
    name,
    InputDefinitionOptions {
      extension: Some("cls"),
      after,
      notex: true,
      handleoptions: true,
      noerror: true,
      with_stomach,
      ..InputDefinitionOptions::default()
    },
    state,
  )
  // if (let success = InputDefinitions($class, type => 'cls', notex => 1, handleoptions => 1,
  // noerror => 1,     %options)) {
  //   return $success; }
  // else {
  //   $STATE->noteStatus(missing => $class . '.cls');
  //   let alternate = 'OmniBus';    # was 'article'
  //   Warn('missing_file', $class, $STATE->getStomach->getGullet,
  //     "Can't find binding for class $class (using $alternate)",
  //     maybeReportSearchPaths());
  // if (let success = InputDefinitions($alternate, type => 'cls', noerror => 1, handleoptions =>
  // 1, %options)) {     return $success; }
  //   else {
  //     Fatal('missing_file', $alternate . '.cls.ltxml', $STATE->getStomach->getGullet,
  //       "Can't find binding for class $alternate (installation error)");
  //     return; } } }
}

pub struct FindFileOptions {
  forbid_ltxml: bool,
  raw: bool,
  notex: bool,
  ext_type: Option<String>,
}

impl Default for FindFileOptions {
  fn default() -> Self {
    FindFileOptions {
      forbid_ltxml: false,
      raw: false,
      notex: false,
      ext_type: None,
    }
  }
}

pub fn find_file(file: &str, options: Option<FindFileOptions>, state: &mut State) -> Option<String> {
  let mut options = options.unwrap_or_default();
  if options.raw {
    options.raw = false;
    warn!(target: "deprecated:raw", "FindFile option raw is deprecated; it is not needed");
  }

  if pathname::is_literaldata(file) {
    // If literal protocol return immediately (unless notex!)
    if options.notex {
      None
    } else {
      // TODO: Consider returning a Cow<str> instead to optimize
      Some(file.to_string())
    }
  } else if pathname::is_literaldata(file) || pathname::is_url(file) {
    // If a known special protocol return immediately
    Some(file.to_string())
  } else if let Some(ref ext) = options.ext_type {
    // Otherwise, it's some kind of "real" file, and we might have to search for it
    // Specific type requested? Search for it.
    // Add the extension, if it isn't already there.
    let aux_file = if file.ends_with(ext) { file.to_string() } else { s!("{}.{}", file, ext) };
    find_file_aux(&aux_file, &options, state)
  } else if file.ends_with(".tex") {
    // If no type given, we MAY expect .tex, or maybe NOT!!
    // No requested type, then .tex; Of course, it may already have it!
    find_file_aux(file, &options, state)
  } else {
    match find_file_aux(&s!("{}.tex", file), &options, state) {
      None => find_file_aux(file, &options, state),
      Some(f) => Some(f),
    }
  }
}

pub fn find_file_aux(file: &str, options: &FindFileOptions, state: &mut State) -> Option<String> {
  // If cached, return simple path (it's a key into the cache)
  let cached = state.lookup_string(&s!("{}_contents", file));
  if !cached.is_empty() {
    Some(file.to_string())
  } else if pathname::is_absolute(file) {
    // And if we've got an absolute path,
    if !options.forbid_ltxml && Path::new(&s!("{}.ltxml", file)).exists() {
      // No need to search, just check if it exists.
      Some(s!("{}.ltxml", file))
    } else if Path::new(file).exists() {
      // No need to search, just check if it exists.
      Some(file.to_string())
    } else {
      // otherwise we're never going to find it.
      None
    }
  } else if pathname::is_nasty(file) {
    // If it is a nasty filename, we won't touch it.
    // we DO NOT want to pass this to kpathse or such!
    None
  } else {
    // Note that the strategy is complicated by the fact that
    // (1) we prefer .ltxml bindings, if present
    // (2) those MAY be present in kpsewhich's DB (although our searchpaths take precedence!)
    // (3) BUT we want to avoid kpsewhich if we can, since it's slower
    // (4) depending on switches we may EXCLUDE .ltxml OR raw tex OR allow both.
    let paths: Vec<String> = state.search_paths.iter().cloned().collect();
    let urlbase = state.lookup_value("URLBASE");
    let nopaths = state.lookup_bool("REMOTE_REQUEST");
    let ltxml_paths: Vec<String> = if nopaths { vec![] } else { paths.clone() };

    // If we're looking for ltxml, look within our paths & installation first (faster than kpse)
    if !options.forbid_ltxml {
      if let Some(path) = pathname::find(
        &s!("{}.ltxml", file),
        NewDefaultV!(PathnameFindOptions, paths => ltxml_paths, installation_subdir => "Package"),
      ) {
        return Some(path);
      }
    }
    // If we're looking for TeX, look within our paths & installation first (faster than kpse)
    if !options.notex {
      if let Some(path) = pathname::find(file, NewDefaultV!(PathnameFindOptions, paths => paths)) {
        return Some(path);
      }
    }
    // Otherwise, pass on to kpsewhich
    // Depending on flags, maybe search for ltxml in texmf or for plain tex in ours!
    // The main point, though, is to we make only ONE (more) call.
    // return if grep { pathname::is_nasty($_) } @$paths;    // SECURITY! No nasty paths in cmdline
    //       // Do we need to sanitize these environment variables?
    // my $kpsewhich = which($ENV{LATEXML_KPSEWHICH} || 'kpsewhich');
    // local $ENV{TEXINPUTS} = join($Config::Config{'path_sep'},
    //   @$paths, $ENV{TEXINPUTS} || $Config::Config{'path_sep'});
    // my @candidates = (((!$options{noltxml} && !$nopaths) ? ("$file.ltxml") : ()),
    //   (!$options{notex} ? ($file) : ()));
    // if (my $result = pathname::kpsewhich(@candidates)) {
    //   return (-f $result ? $result : undef); }
    // if ($urlbase && ($path = url_find($file, urlbase => $urlbase))) {
    //   return $path; }
    // return; }
    info!("No path found for: {:?}", file);
    None
  }
}

//======================================================================
// Declaring and Adjusting the Document Model.
//======================================================================

pub fn install_tag(tag: &str, mut properties: TagOptions, state: &mut State) {
  let mut options = state.tag_properties.entry(tag.to_string()).or_insert_with(TagOptions::default);
  if properties.auto_open.is_some() {
    options.auto_open = properties.auto_open;
  }
  if properties.auto_close.is_some() {
    options.auto_close = properties.auto_close;
  }

  for name in &TagOptionName::all() {
    if name.is_prepend() {
      options.prepend(name, properties.remove(name));
    } else if name.is_append() {
      options.append(name, properties.remove(name));
    } else {
      // we'll handle the regular ones out of the loop
    }
  }
}

pub struct InputDefinitionOptions<'a> {
  pub extension: Option<&'static str>,
  pub options: Vec<String>,
  pub after: Tokens,
  pub notex: bool,
  pub noerror: bool,
  pub noltxml: bool,
  pub withoptions: Vec<String>,
  pub handleoptions: bool,
  pub as_class: bool,
  pub with_stomach: Option<&'a mut Stomach>,
}
impl<'a> Default for InputDefinitionOptions<'a> {
  fn default() -> Self {
    InputDefinitionOptions {
      extension: None,
      options: Vec::new(),
      after: Tokens!(),
      notex: false,
      noerror: false,
      noltxml: false,
      withoptions: Vec::new(),
      handleoptions: false,
      as_class: false,
      with_stomach: None,
    }
  }
}

// Selects the RelaxNG schema defining the XML output language
pub fn select_relaxng_schema(schema: String, namespaces: Option<HashMap<String, String>>, state: &mut State) {
  // What verb here? Set, Choose,...
  let model = &mut state.model;
  model.set_relaxng_schema(schema);
  if let Some(namespaces) = namespaces {
    for (prefix, value) in namespaces {
      model.register_document_namespace(&prefix, Some(value));
    }
  }
  return;
}

pub fn merge_font(font: Font, state: &mut State) {
  let new_font = match state.lookup_font() {
    Some(ref f) => f.merge(font),
    _ => Font::text_default().merge(font),
  };
  state.assign_value("font", new_font, Some(Scope::Local));
}

pub fn digest_text(stuff: Tokens, stomach: &mut Stomach, state: &mut State) -> Result<Digested> {
  stomach.begin_mode("text", state)?;
  let value = stomach.digest(stuff, state);
  stomach.end_mode("text", state)?;
  value
}

pub fn digest_literal<T: Into<Tokens>>(stuff: T, stomach: &mut Stomach, state: &mut State) -> Result<Digested> {
  let stuff: Tokens = stuff.into();
  // Perhaps should do StartSemiverbatim, but is it safe to push a frame? (we might cover over
  // valid changes of state!)
  stomach.begin_mode("text", state)?;

  let font = state.lookup_font().unwrap(); // TODO: raise error if font missing
  state.assign_value("font", font.merge(fontmap!(encoding => "ASCII")), Some(Scope::Local)); // try to stay as ASCII as possible

  let value = stomach.digest(stuff, state);
  state.assign_value("font", font, None); // TODO: maybe we need .assign_font ?
  stomach.end_mode("text", state)?;
  value
}

pub fn digest_if(token: Token, stomach: &mut Stomach, state: &mut State) -> Result<Option<Digested>> {
  if let Some(defn) = state.lookup_definition(&token) {
    match stomach.digest(Tokens!(token), state) {
      Ok(t) => Ok(Some(t)),
      Err(e) => Err(e),
    }
  } else {
    Ok(None)
  }
}

pub fn build_invocation<T: Into<Token>>(token: T, args: Vec<Tokens>, gullet: &mut Gullet, state: &mut State) -> Result<Tokens> {
  let token: Token = token.into();
  // Note: token may have been \let to another defn!
  if let Some(defn) = state.lookup_definition(&token) {
    let mut invoked_tokens = vec![token];
    let mut reverted_args = if let Some(params) = defn.get_parameters() {
      params.revert_arguments(args, gullet, state)?
    } else {
      Vec::new()
    };
    invoked_tokens.append(&mut reverted_args);
    Ok(Tokens::new(invoked_tokens))
  } else {
    error!(
      target: &s!("undefined:{}", token.get_cs_name()),
      "Can't invoke {:?}; it is undefined", token
    );
    let mut invoked_tokens = vec![token];
    // DefConstructorI!(token, convert_latex_args(args.len(), 0),
    // sub { LaTeXML::Core::Stomach::makeError($_[0], 'undefined', token); });
    let mut wrapped_args: Vec<Token> = args
      .into_iter()
      .flat_map(|arg| {
        let mut wrapped = vec![T_BEGIN!()];
        wrapped.append(&mut arg.unlist());
        wrapped.push(T_END!());
        wrapped
      })
      .collect();
    invoked_tokens.append(&mut wrapped_args);
    Ok(Tokens::new(invoked_tokens))
  }
}

pub fn do_expand<T: Into<Tokens>>(mut tokens: T, outer_gullet: &mut Gullet, outer_state: &mut State) -> Result<Tokens> {
  let mut tokens: Tokens = tokens.into();
  outer_gullet.reading_from_mouth(
    Mouth::default(),
    outer_state,
    move |expand_gullet: &mut Gullet, expand_state: &mut State| -> Result<Tokens> {
      expand_gullet.unread(tokens);
      let mut expanded = Vec::new();
      while let Some(t) = expand_gullet.read_x_token(false, false, expand_state)? {
        expanded.push(t);
      }
      Ok(Tokens::new(expanded))
    },
  )
}

/// Convert a LaTeX-style argument spec to our Package form.
/// Ie. given $nargs and $optional, being the two optional arguments to
/// something like \newcommand, convert it to the form we use
pub fn convert_latex_args(mut nargs: usize, optional: Option<Tokens>, state: &mut State) -> Result<Option<Parameters>> {
  let mut params = Vec::new();
  if let Some(tks) = optional {
    params.push(
      Parameter {
        name: s!("Optional"),
        spec: s!("[Default:{}]", tks.untex(state)),
        extra: vec![ParameterExtra::Token(tks.into()), ParameterExtra::ParametersOption(None)],
        ..Parameter::default()
      }
      .init(state)?,
    );
    nargs -= 1;
  }

  for _ in 1..=nargs {
    params.push(
      Parameter {
        name: s!("Plain"),
        spec: "{}".to_string(),
        ..Parameter::default()
      }
      .init(state)?,
    );
  }
  if params.is_empty() {
    Ok(None)
  } else {
    Ok(Some(Parameters { params }))
  }
}
