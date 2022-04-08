use std::borrow::Cow;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use rtx_core::common::error::*;
use rtx_core::common::font::Font;
use rtx_core::common::object::Object;
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
use rtx_core::{Digested};

use super::def_dialect::def_macro;
use super::*;
use crate::package::pool;

lazy_static! {
  static ref QUOTE_WRAPPED : Regex = Regex::new("^\"(.+)\"$").unwrap();
}

pub fn load_external_binding(file: &str, state: &mut State, mut stomach: &mut Stomach) -> Result<bool> {
  let taken_dispatcher = state.extra_bindings_dispatch.take();
  match taken_dispatcher {
    Some(ref dispatcher) => {
      let result_opt = dispatcher(file, stomach, state);
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
      // these notes should be inside the dispatch, since it may not have anything to load
      // note_begin(&s!("Loading {:?} definitions", file));
      let result_opt = dispatcher(file, stomach, state);
      //note_end(&s!("Loading {:?} definitions", file));
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
pub fn input_definitions(raw_file: &str, mut options: InputDefinitionOptions, mut stomach: &mut Stomach, mut state: &mut State) -> Result<()> {
  let name = raw_file.trim();
  // Note: we always need a gullet to expand, and we sometimes need a stomach to load_definitions... so let's make stomach a mandatory option.
  let prevname = if options.handleoptions && state.lookup_definition(&T_CS!("\\@currname")).is_some() {
    let gullet = stomach.get_gullet_mut();
    do_expand(T_CS!("\\@currname"), gullet, state)?.to_string()
  } else {
    String::new()
  };
  let prevext = if options.handleoptions && state.lookup_definition(&T_CS!("\\@currext")).is_some() {
    let gullet = stomach.get_gullet_mut();
    do_expand(T_CS!("\\@currext"), gullet, state)?.to_string()
  } else {
    String::new()
  };
  // This file will be treated somewhat as if it were a class
  // IF as_class is true
  // OR if it is loaded by such a class, and has withoptions true!!! (yikes)
  if options.handleoptions && options.withoptions.is_some() {
    if let Some(vdq) = state.lookup_vecdeque("@masquerading@as@class") {
      if vdq.iter().any(|x| if let Stored::String(ref v) = x { v == &prevname } else { false }) {
        options.as_class = true;
      }
    }
  }
  if options.noltxml {
    options.raw = true; // so it will be read as raw by Gullet.
  }
  let as_type = if options.as_class { "cls" } else { options.extension.unwrap_or("") };

  // Compute the exact name based on the type
  let filename = match options.extension {
    None => name.to_string(),
    Some(ext) => s!("{}.{}", name, ext),
  };
  let current_options = options.options.join(",");
  if !current_options.is_empty() {
    if let Some(Stored::String(prevoptions)) = state.lookup_value(&s!("{}_loaded_with_options", filename)) {
      if &current_options != prevoptions {
        let message = s!(
          "Option clash for file {} with options {:?}, previously loaded with {:?}",
          filename,
          current_options,
          prevoptions
        );
        Info!("unexpected", "options", stomach, state, message);
      }
    }
  }

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
  def_macro(T_CS!("\\@currname"), None, Tokens!(Explode!(name)), None, state);
  def_macro(T_CS!("\\@currext"), None, Tokens!(Explode!(as_type)), None, state);

  // TODO: Is this inaccurate with latexml? It only sets the macros if the file is found, we set them *always*, as a matter of course
  if options.handleoptions {
    input_handle_options(&mut options, &prevname, &prevext, name, as_type, stomach, state)?;
    def_macro(T_CS!(s!("\\{}.{}-h@@k", name, as_type)), None, options.after, None, state);
  }
  if !current_options.is_empty() {
    state.assign_value(&s!("{}_loaded_with_options", filename), current_options, Some(Scope::Global));
  }

  let is_contrib = load_external_binding(&filename, state, stomach)?;
  let mut is_binding = true;
  if !is_contrib {
    match filename.as_ref() {
      "TeX.pool" => pool::tex::load_definitions(stomach, state)?,
      "LaTeX.pool" => pool::latex::load_definitions(stomach, state)?,
      "eTeX.pool" => pool::etex::load_definitions(stomach, state)?,
      "pdfTeX.pool" => pool::pdftex::load_definitions(stomach, state)?,
      "article.cls" => pool::article_cls::load_definitions(stomach, state)?,
      "alltt.sty" => pool::alltt_sty::load_definitions(stomach, state)?,
      "amsmath.sty" => pool::amsmath_sty::load_definitions(stomach, state)?,
      "amsthm.sty" => pool::amsthm_sty::load_definitions(stomach, state)?,
      "comment.sty" => pool::comment_sty::load_definitions(stomach, state)?,
      "IEEEtran.cls" => pool::ieeetran_cls::load_definitions(stomach, state)?,
      "url.sty" => pool::url_sty::load_definitions(stomach, state)?,
      "verbatim.sty" => pool::verbatim_sty::load_definitions(stomach, state)?,
      "fontenc.sty" => pool::fontenc_sty::load_definitions(stomach, state)?,
      "inputenc.sty" => pool::inputenc_sty::load_definitions(stomach, state)?,
      "textcomp.sty" => pool::textcomp_sty::load_definitions(stomach, state)?,
      other => is_binding = false,
    };
  }
  if is_binding {
    // We found and loaded a binding successfully, mark it as such.
    state.assign_value(&loaded_flag, true, Some(Scope::Global));
  } else {
    // We're inverting the control flow, because it is near-instant to check whether we have an available
    // binding dispatcher, in both contributed and core binding names
    // Now that we have ensured there is no compiled target of this name, we can start the file system search dance,
    // call to kpsewhich, etc.
    //
    if let Some(absolute_filename) = pathname::kpsewhich(&[&filename]) {
      load_tex_definitions(&filename, &absolute_filename, stomach, state)?;
    } else {
      fatal!(Package, Unknown, s!("TODO: unknown binding {:?}, can't load", filename))
    }
  }

  note_end(&s!("Loading {:?} definitions", filename));
  Ok(())
}

// Factor out handling and passing loading options from input_content,
// to simplify main routine
fn input_handle_options(
  options: &mut InputDefinitionOptions,
  prevname: &str,
  prevext: &str,
  name: &str,
  as_type: &str,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<()> {
  // For \RequirePackageWithOptions, pass the options from the outer class/style to the inner one.
  if let Some(with_options_to_pass) = options.withoptions.take() {
    if !prevname.is_empty() && state.has_value(&s!("opt@{}.{}", prevname, prevext)) {
      // Only pass those class options that are declared by the package!
      if let Some(declared_options) = state.lookup_vecdeque("@declaredoptions") {
        let mut topass = Vec::new();
        for op in with_options_to_pass.into_iter() {
          if declared_options
            .iter()
            .any(|x| if let Stored::String(val) = x { val == &op } else { false })
          {
            topass.push(op)
          }
        }
        if !topass.is_empty() {
          pass_options(name, as_type, topass, state)
        }
      }
    }
  }
  def_macro(T_CS!("\\@currname"), None, Tokens!(Explode!(name)), None, state);
  def_macro(T_CS!("\\@currext"), None, Tokens!(Explode!(as_type)), None, state);
  // reset options (Note reset & pass were in opposite order in LoadClass ????)
  let gullet = stomach.get_gullet_mut();
  reset_options(gullet, state)?;
  pass_options(name, as_type, options.options.clone(), state); // passed explicit options.
                                                               // Note which packages are pretending to be classes.
  if options.as_class {
    state.push_value("@masquerading@as@class", name);
  }
  let current_opt_val = match state.lookup_vecdeque(&s!("opt@{}.{}", name, as_type)) {
    Some(vdq) => vdq
      .iter()
      .map(|x| if let Stored::String(val) = x { val } else { "" })
      .collect::<Vec<&str>>()
      .join(","), // this is so painful, why can't we .join on a VecDeque?
    None => String::new(),
  };
  def_macro(
    T_CS!(s!("\\opt@{}.{}", name, as_type)),
    None,
    Tokens!(Explode!(current_opt_val)),
    None,
    state,
  );
  Ok(())
}
#[derive(Debug, Default, Clone)]
pub struct InputOptions {
  pub noerror: bool,
  pub file_type: Option<String>,
}
pub fn input_content(request: &str, options: InputOptions, stomach: &mut Stomach, state :&mut State) -> Result<()> {
  let filepath = find_file(request, None, state);
  match filepath {
    // TODO: type => $options{type}, noltxml => 1
    Some(path) => load_tex_content(&path, options, stomach, state),
    None => fatal!(Package, MissingFile, request),
    /* TODO:
     * Error("missing_file", request, state.get_stomach().get_gullet(),
     * "Can't find TeX file "+request, maybeReportSearchPaths(state))) */
  }
}

pub fn input(mut request: &str, options: InputOptions, stomach: &mut Stomach, state: &mut State) -> Result<()> {
  //  // unwrap if in quotes \input{"file name"}
  // while request.starts_with('"') && request.ends_with('"') {
  //   request = QUOTE_WRAPPED.replace(request);
  // }
  // // HEURISTIC! First check if equivalent style file, but only under very specific circumstances
  // if pathname_is_literaldata(request) {
  //   let (dir, name, ftype) = pathname_split(request);
  //   let file = name;
  //   if !ftype.is_empty() {
  //     file += format!(".{}",ftype);
  //   }
  //   let path;
  //   // Firstly, check if we are going to OVERRIDE the requested raw .tex file
  //   // with a latexml binding to a style file.
  //   if ((dir.is_empty() && (ftype.is_empty() || (ftype == "tex"))  // No SPECIFIC directory, but a raw tex file.
  //       // AND, in preamble; SHOULD be style file, OR also if we can't find the raw file.
  //     && (LookupValue!("inPreamble") || !FindFile(file))
  //     && (path = FindFile(name, type => 'sty', notex => 1))) { // AND there IS such a style file
  //     Info!("ignore", request, stomach.get_gullet(),
  //       s!("Ignoring input of tex {}, using package {} instead", request, name));
  //     RequirePackage!(name); // Then override, assuming we'll find name as a package file!
  //     return;
  //   }
  // }
  // // Next special case: If we were currently reading a "known" style or binding file,
  // // then this file, even if .tex, must also be definitions rather than content.!!(?)
  // if state.lookup_bool("INTERPRETING_DEFINITIONS") {
  //   input_definitions(request);
  // }
  if let Some(path) = find_file(request, None, state) { // Found something plausible..
  //   let ftype = if pathname_is_literaldata(path) { "tex" } else {
  //     pathname_type(path)
  //   };

  //   // Should we be doing anything about options in the next 2 cases?..... I kinda think not, but?
  //   if (ftype == "rs") {                  // it's a LaTeXML binding.
  //     load_rtx(request, path);
  //   }
  //   // Else some sort of "known" definitions type file, but not simply 'tex'
  //   else if (ftype != "tex") && (pathname_is_raw(path)) {
  //     load_tex_definitions(request, path);
  //   } else {
      load_tex_content(&path, options, stomach, state)
  //   }
  } else { // Couldn't find anything?
    state.note_status("missing");//, request);
    // We presumably are trying to input Content; an error if we can't find it (contrast to Definitions)
    let gullet = stomach.get_gullet();
    Error!("missing_file", request, gullet, state,
       s!("Can't find TeX file {}", request));
      //  maybeReportSearchPaths());
    Ok(())
  }
}

fn load_tex_definitions(request: &str, pathname: &str, stomach: &mut Stomach, state: &mut State) -> Result<()> {
  if !pathname::is_literaldata(pathname) {
    // We can't analyze literal data's pathnames!
    let (dir, name, extension) = pathname::split(pathname);
    // Don't load if we've already loaded it before.
    // Note that we'll still load it if we've already loaded only the ltxml version
    // since someone's presumably asking _explicitly_ for the raw TeX version.
    // It's probably even the ltxml version is asking for it!!
    // Of course, now it will be marked and wont get reloaded!
    if state.lookup_bool(&s!("{}_loaded", request)) {
      return Ok(());
    }
    state.assign_value(&s!("{}_loaded", request), true, Some(Scope::Global));
  }

  // Note that we are reading definitions (and recursive input is assumed also definitions)
  let was_interpreting = state.lookup_bool("INTERPRETING_DEFINITIONS");
  // And that if we're interpreting this TeX file of definitions,
  // we probably should interpret any TeX files IT loads.
  let was_including_styles = state.lookup_bool("INCLUDE_STYLES");
  state.assign_value("INTERPRETING_DEFINITIONS", true, None);
  // If we're reading in these definitions, probaly will accept included ones?
  // (but not forbid ltxml ?)
  state.assign_value("INCLUDE_STYLES", true, None);
  // When set, this variable allows redefinitions of locked defns.
  // It is set in before/after methods to allow local rebinding of commands
  // but loading of sources & bindings is typically done in before/after methods of constructors!
  // This re-locks defns during reading of TeX packages.
  state.unlocked = false;
  let content_str = state.lookup_string(&s!("{}_contents", pathname));
  let content = if content_str.is_empty() { None } else { Some(content_str) };
  let mut pathname_mouth = Mouth::create(
    pathname,
    MouthOptions {
      fordefinitions: true,
      notes: true,
      content,
      ..MouthOptions::default()
    },
    state,
  )?;

  stomach.reading_from_mouth(pathname_mouth, state, move |i_stomach, i_state| -> Result<()> {
    while let Some(token) = i_stomach.get_gullet_mut().read_x_token(false, false, i_state)? {
      if token != T_SPACE!() {
        i_stomach.invoke_token(&token, i_state)?;
      }
    }
    Ok(())
  })?;

  state.assign_value("INTERPRETING_DEFINITIONS", was_interpreting, None);
  state.assign_value("INCLUDE_STYLES", was_including_styles, None);
  Ok(())
}

pub fn load_tex_content(path: &str, options: InputOptions, stomach: &mut Stomach, state: &mut State) -> Result<()> {
  // If there is a file-specific declaration file (name_tex.rs), load it first!
  // let namespace = path;
  // state.extra_bindings_dispatch
  if !pathname::is_literaldata(path) {
    let (dir, base, ext) = pathname::split(path);
    load_external_binding(&base, state, stomach)?;
  }
  // TODO: Caching
  // content => LookupValue($pathname . '_contents')

  // Open a mouth for that TeX content
  stomach.get_gullet_mut().open_mouth(
    Mouth::create(
      path,
      MouthOptions {
        notes: true,
        ..MouthOptions::default()
      },
      state,
    )?,
    true,
  );
  Ok(())
}

/// Pass the sequence of @options to the package $name (if $ext is 'sty'),
/// or class $name (if $ext is 'cls').
fn pass_options(name: &str, ext: &str, options: Vec<String>, state: &mut State) { state.push_value(&s!("opt@{}.{}", name, ext), options); }

pub fn process_options(stomach: &mut Stomach, state: &mut State) -> Result<()> {
  let currname_token = T_CS!("\\@currname");
  let currext_token = T_CS!("\\@currext");
  let gullet = stomach.get_gullet_mut();
  let name = if state.lookup_definition(&currname_token).is_some() {
    do_expand(currname_token, gullet, state)?.to_string()
  } else {
    String::new()
  };
  let ext = if state.lookup_definition(&currext_token).is_some() {
    do_expand(currext_token, gullet, state)?.to_string()
  } else {
    String::new()
  };
  let empty_vdq = VecDeque::new(); // convenience for unwrapping empty

  let declared_options: VecDeque<Stored> = state.lookup_vecdeque("@declaredoptions").unwrap_or(&empty_vdq).clone();
  let opt_key = s!("opt@{}.{}", name, ext);
  let current_options = state.lookup_vecdeque(&opt_key).unwrap_or(&empty_vdq);
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
  let mut requested_options: HashSet<String> = HashSet::new();
  for option in current_options.iter() {
    match option {
      Stored::String(content) => {
        requested_options.insert(content.to_string());
      },
      Stored::VecString(contents) => {
        for content in contents.iter() {
          requested_options.insert(content.to_string());
        }
      },
      _ => {},
    }
  }
  for option in class_options.iter() {
    match option {
      Stored::String(content) => {
        requested_options.insert(content.to_string());
      },
      Stored::VecString(contents) => {
        for content in contents.iter() {
          requested_options.insert(content.to_string());
        }
      },
      _ => {},
    }
  }

  // Execute options in declared order (eg. \ProcessOptions)
  for option in declared_options.iter() {
    match option {
      Stored::String(content) => {
        if requested_options.contains(content) {
          requested_options.remove(content); // Remove it, since it's been handled.
          execute_option_internal(content, stomach, state)?;
        }
      },
      Stored::VecString(contents) => {
        for content in contents.iter() {
          if requested_options.contains(content) {
            requested_options.remove(content); // Remove it, since it's been handled.
            execute_option_internal(content, stomach, state)?;
          }
        }
      },
      _ => {},
    }
  }
  // Now handle any remaining options (eg. default options), in the given order.
  for option in requested_options.iter() {
    execute_default_option_internal(option, stomach, state)?;
  }
  // Now, undefine the handlers?
  for option in declared_options.iter() {
    state.let_i(&T_CS!(s!("\\ds@{}", option)), T_CS!("\\relax"), None);
  }
  Ok(())
}

fn execute_option_internal(option: &str, stomach: &mut Stomach, state: &mut State) -> Result<bool> {
  let cs = T_CS!(s!("\\ds@{}", option));
  if state.lookup_definition(&cs).is_some() {
    def_macro(T_CS!("\\CurrentOption"), None, Tokens!(T_OTHER!(option)), None, state);

    let unused = match state.remove_vecdeque("@unusedoptionlist") {
      Some(list) => list
        .into_iter()
        .filter(|item| if let Stored::String(content) = item { content != option } else { false })
        .collect(),
      None => VecDeque::new(),
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

fn reset_options(gullet: &mut Gullet, state: &mut State) -> Result<()> {
  state.assign_value("@declaredoptions", Stored::VecDequeStored(VecDeque::new()), None);
  let opt_unused_cs = if do_expand(T_CS!("\\@currext"), gullet, state)?.to_string() == "cls" {
    "\\OptionNotUsed"
  } else {
    "\\@unknownoptionerror"
  };
  state.let_i(&T_CS!("\\default@ds"), T_CS!(opt_unused_cs), None);
  Ok(())
}

pub struct RequireOptions {
  pub options: Vec<String>,
  pub withoptions: Option<Vec<String>>,
  pub extension: Option<&'static str>,
  pub as_class: bool,
  pub noltxml: bool,
  pub notex: bool,
  pub raw: bool,
  pub after: Tokens,
}
impl Default for RequireOptions {
  fn default() -> Self {
    RequireOptions {
      options: Vec::new(),
      withoptions: None,
      extension: None,
      as_class: false,
      noltxml: false,
      notex: true,
      raw: false,
      after: Tokens!(),
    }
  }
}

/// This (and `FindFile`) needs to evolve a bit to support reading raw .sty (.def, etc) files from
/// the standard texmf directories.  Maybe even use kpsewhich itself (INSTEAD of `pathname_find`
/// ???) Another potentially useful option might be that if we are reading a raw file,
/// perhaps it should just get digested immediately, since it shouldn't contribute any boxes.
pub fn require_package(name: &str, mut options: RequireOptions, stomach: &mut Stomach, state: &mut State) -> Result<()> {
  if options.raw {
    options.raw = false;
    Warn!(
      "deprecated",
      "raw",
      stomach,
      state,
      "RequirePackage option raw is obsolete; it is not needed"
    );
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
      // Pass classes options if we have NONE!
      withoptions: if options.options.is_empty() { Some(Vec::new()) } else { None }, // fake boolean use, multi-type in latexml... refactor?
      options: options.options,
      as_class: options.as_class,
      noltxml: options.noltxml,
      notex: options.notex,
      raw: options.raw,
      after: options.after,
      ..InputDefinitionOptions::default()
    },
    stomach,
    state,
  )
}

pub fn require_resource(mut resource: Resource, state: &mut State) {
  if resource.name.is_empty() && resource.content.is_empty() {
    Warn!(
      "expected",
      "resource",
      None,
      state,
      "Resource must have a resource pathname or content; skipping"
    );
    return;
  }
  if resource.mimetype.is_empty() && !resource.name.is_empty() {
    let ext = pathname::extension(&resource.name);
    resource.mimetype = resource_type(&ext);
  }
  if resource.mimetype.is_empty() {
    Warn!("expected", "mime-type", None, state, "Resource must have a mime-type; skipping");
    return;
  }

  // If we've got a document, go ahead & put the resource in.
  // if (state.document.is_some()) {
  //   state.document.as_mut().unwrap().add_resource(resource, resource);
  // } else {
  state.pending_resources.push(resource);
  // }
}

pub fn load_class(name: &str, options: Vec<String>, after: Tokens, stomach: &mut Stomach, state: &mut State) -> Result<()> {
  input_definitions(
    name,
    InputDefinitionOptions {
      extension: Some("cls"),
      after,
      notex: true,
      handleoptions: true,
      noerror: true,
      ..InputDefinitionOptions::default()
    },
    stomach,
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

#[derive(Default)]
pub struct FindFileOptions {
  forbid_ltxml: bool,
  raw: bool,
  notex: bool,
  ext_type: Option<String>,
}

pub fn find_file(file: &str, options: Option<FindFileOptions>, state: &mut State) -> Option<String> {
  let mut options = options.unwrap_or_default();
  if options.raw {
    options.raw = false;
    Warn!("deprecated", "raw", None, state, "FindFile option raw is deprecated; it is not needed");
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
    // Info!("No path found for: {:?}", file);
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

pub struct InputDefinitionOptions {
  pub extension: Option<&'static str>,
  pub options: Vec<String>,
  pub after: Tokens,
  pub notex: bool,
  pub noerror: bool,
  pub noltxml: bool,
  pub withoptions: Option<Vec<String>>,
  pub handleoptions: bool,
  pub as_class: bool,
  pub raw: bool,
}
impl Default for InputDefinitionOptions {
  fn default() -> Self {
    InputDefinitionOptions {
      extension: None,
      options: Vec::new(),
      after: Tokens!(),
      notex: false,
      noerror: false,
      noltxml: false,
      raw: false,
      withoptions: None,
      handleoptions: false,
      as_class: false,
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
}

pub fn merge_font(font: Font, state: &mut State) {
  let new_font = state.lookup_font().unwrap().merge(font);
  state.assign_value("font", Arc::new(new_font), Some(Scope::Local));
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
      params.revert_arguments(args, state)?
    } else {
      Vec::new()
    };
    invoked_tokens.append(&mut reverted_args);
    Ok(Tokens::new(invoked_tokens))
  } else {
    let message = s!("Can't invoke {:?}; it is undefined", token.stringify());
    Error!("undefined", token.get_cs_name(), gullet, state, message);
    let mut invoked_tokens = vec![token];
    // DefConstructor!(token, convert_latex_args(args.len(), 0),
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
    Ok(Some(Parameters::new(params)))
  }
}
