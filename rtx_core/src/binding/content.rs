use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::borrow::Cow;
use std::collections::VecDeque;
use std::path::Path;

use crate::common::arena;
use crate::common::error::*;
use crate::common::font::{Font, Fontmap};
use crate::common::object::Object;
use crate::document::resource::*;
use crate::document::tag::{TagOptionName, TagOptions};
use crate::gullet::Gullet;
use crate::mouth::{Mouth, MouthOptions};
use crate::parameter::{Parameter, Parameters};
use crate::state::{Scope, State, Stored};
use crate::stomach::Stomach;
use crate::token::*;
use crate::util::pathname::{self, PathnameFindOptions};
use crate::*;
// use crate::util::pathname::PathnameFindOptions;
use crate::Digested;

use crate::binding::def::dialect::def_macro;

static QUOTE_WRAPPED: Lazy<Regex> = Lazy::new(|| Regex::new("^\"(.+)\"$").unwrap());

/// a configuration for loading LaTeX definition files (such as .sty, .cls, and their bindings)
pub struct InputDefinitionOptions {
  /// an optional extension (such as "sty")
  pub extension: Option<Cow<'static, str>>,
  /// package options to pass into the loaded library
  pub options: Vec<String>,
  /// Tokens to process after the definition is loaded
  pub after: Tokens,
  /// flag to forbid raw TeX sources
  pub notex: bool,
  /// flag to forbid errors ?
  pub noerror: bool,
  /// flag to forbid binding dispatch
  pub noltxml: bool,
  ///
  pub withoptions: Option<Vec<String>>,
  /// flag to handle options, or ignore them
  pub handleoptions: bool,
  /// flag to process in .cls mode (default: false)
  pub as_class: bool,
  /// flag to indicate reading the file raw in Gullet
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

/// TODO: Flesh out with the full infrastructure, incremental functionality for now.
pub fn input_definitions(
  raw_file: &str,
  mut options: InputDefinitionOptions,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<()> {
  let name = raw_file.trim();
  // Note: we always need a gullet to expand, and we sometimes need a stomach to load_definitions...
  // so let's make stomach a mandatory option.
  let prevname =
    if options.handleoptions && state.lookup_definition(&T_CS!("\\@currname")).is_some() {
      let gullet = stomach.get_gullet_mut();
      gullet.do_expand(T_CS!("\\@currname"), state)?.to_string()
    } else {
      String::new()
    };
  let prevext = if options.handleoptions && state.lookup_definition(&T_CS!("\\@currext")).is_some()
  {
    let gullet = stomach.get_gullet_mut();
    gullet.do_expand(T_CS!("\\@currext"), state)?.to_string()
  } else {
    String::new()
  };
  // This file will be treated somewhat as if it were a class
  // IF as_class is true
  // OR if it is loaded by such a class, and has withoptions true!!! (yikes)
  if options.handleoptions && options.withoptions.is_some() {
    if let Some(vdq) = state.lookup_vecdeque("@masquerading@as@class") {
      if vdq.iter().any(|x| {
        if let Stored::String(ref v) = x {
          arena::with(*v, |str| str == prevname)
        } else {
          false
        }
      }) {
        options.as_class = true;
      }
    }
  }
  if options.noltxml {
    options.raw = true; // so it will be read as raw by Gullet.
  }
  let as_type = if options.as_class {
    Cow::Borrowed("cls")
  } else {
    options
      .extension
      .as_ref()
      .cloned()
      .unwrap_or(Cow::Borrowed(""))
  };

  // Compute the exact name based on the type
  let filename = match &options.extension {
    None => name.to_string(),
    Some(ext) => s!("{}.{}", name, ext),
  };
  let current_options = options.options.join(",");
  if !current_options.is_empty() {
    if let Some(Stored::String(prevoptions)) =
      state.lookup_value(&s!("{filename}_loaded_with_options"))
    {
      if arena::with(*prevoptions, |prev_str| current_options != prev_str) {
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

  // TODO: This needs reorganization, bindings are not found as "files" in rust,
  // we need to have a registry (we don't yet)

  // Mark as loaded, then process the definitions
  note_begin(&s!("Loading {:?} definitions", filename));
  def_macro(
    T_CS!("\\@currname"),
    None,
    Tokens!(Explode!(name)),
    None,
    state,
  );
  def_macro(
    T_CS!("\\@currext"),
    None,
    Tokens!(Explode!(as_type)),
    None,
    state,
  );

  // TODO: Is this inaccurate with latexml? It only sets the macros if the file is found, we set
  // them *always*, as a matter of course TODO: This *IS* inaccurate with the Package.pm
  // InputDefinitions, revisit at the right time and make sure it matches line-by-line (including
  // the subordinated methods)
  if options.handleoptions {
    before_input_handle_options(
      &mut options,
      &prevname,
      &prevext,
      name,
      &as_type,
      stomach,
      state,
    )?;
    def_macro(
      T_CS!(s!("\\{}.{}-h@@k", name, as_type)),
      None,
      options.after,
      None,
      state,
    );
  }

  if !current_options.is_empty() {
    state.assign_value(
      &s!("{}_loaded_with_options", filename),
      current_options,
      Some(Scope::Global),
    );
  }

  let is_binding = !options.noltxml
    && (load_external_binding(&filename, stomach, state)?
      || load_binding(&filename, stomach, state)?);
  let mut is_found_raw = false;
  if is_binding {
    // We found and loaded a binding successfully, mark it as such.
    let loaded_flag = format!("{filename}_loaded");
    state.assign_value(&loaded_flag, true, Some(Scope::Global));
  } else {
    // We're inverting the control flow, because it is near-instant to check whether we have an
    // available binding dispatcher, in both contributed and core binding names
    // Now that we have ensured there is no compiled target of this name, we can start the file
    // system search dance, call to kpsewhich, etc.
    //
    // Find the file to load
    // TODO options.search_paths_only
    if let Some(file) = find_file(
      &filename,
      Some(FindFileOptions {
        forbid_ltxml: options.noltxml,
        notex: options.notex,
        ext_type: options.extension.as_ref().cloned(),
        search_paths_only: false,
      }),
      state,
    ) {
      is_found_raw = true;
      load_tex_definitions(&filename, &file, stomach, state)?;
    } else if !options.noerror {
      // TODO: Proper missing reports
      Warn!(
        "missing_file",
        name,
        stomach,
        state,
        s!("Can't find file for {name}")
      );
      // STATE.note_status(missing => $name . ($options{type} ? '.' . $options{type} : ''));
      // # We'll only warn about a missing file of definitions: it may be ignorable or never used.
      // # if there ARE problems, they'll likely produce their own errors!
      // Warn('missing_file', $name, $STATE->getStomach->getGullet,
      //   "Can't find "
      //     . ($options{notex} ? "binding for " : "")
      //     . (($options{type} && $definition_name{ $options{type} }) || 'definitions') . ' '
      //     . $name,
      //   "Anticipate undefined macros or environments",
      //   maybeReportSearchPaths()); }
    }
  }

  if (is_binding || is_found_raw) && options.handleoptions {
    // after_input_handle_options ?
    stomach.digest(T_CS!(s!("\\{name}.{as_type}-h@@k")), state)?;
    if !prevname.is_empty() {
      def_macro(
        T_CS!("\\@currname"),
        None,
        Tokens!(Explode!(prevname)),
        None,
        state,
      );
    }
    if !prevext.is_empty() {
      def_macro(
        T_CS!("\\@currext"),
        None,
        Tokens!(Explode!(prevext)),
        None,
        state,
      );
    }
    stomach.digest(T_CS!("\\@popfilename"), state)?;
    reset_options(stomach.get_gullet_mut(), state)?; // And reset options afterwards, too.
  }
  note_end(&s!("Loading {:?} definitions", filename));
  Ok(())
}

/// loads a binding from the main binding dispatcher, if available+found
pub fn load_binding(file: &str, stomach: &mut Stomach, state: &mut State) -> Result<bool> {
  _load_binding(true, file, stomach, state)
}
/// loads a binding from an external binding dispatcher, if available+found
pub fn load_external_binding(file: &str, stomach: &mut Stomach, state: &mut State) -> Result<bool> {
  _load_binding(false, file, stomach, state)
}
// in the spirit of Perl's Package::loadLTXML
fn _load_binding(
  internal: bool,
  request: &str,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<bool> {
  // avoid double-loads, but be binding-specific
  let loaded_key = s!("{request}_binding_loaded");
  if state.lookup_bool(&loaded_key) {
    return Ok(true);
  }
  // TODO? || state.lookup_bool(&s!("{trequest}_loaded"))
  //|| state.lookup_bool(&s!("{name}_loaded")) || state.lookup_bool(&s!("{ltxname}_loaded"));

  let taken_dispatcher = if internal {
    state.bindings_dispatch.as_ref().map(Rc::clone)
  } else {
    state.extra_bindings_dispatch.as_ref().map(Rc::clone)
  };
  match taken_dispatcher {
    Some(ref dispatcher) => {
      let result_opt = dispatcher(request, stomach, state);
      match result_opt {
        Some(result) => {
          // Here and only here we are certain we have binding support.
          // Preemptively mark as loaded to avoid recursion.

          // TODO: is this still true?
          // Note (only!) that the binding version of this was loaded; still could load raw tex!
          state.assign_value(&loaded_key, true, Some(Scope::Global));
          // if a binding load succeeded, mark the generic request as loaded.
          state.assign_value(&s!("{request}_loaded"), true, Some(Scope::Global));
          match result {
            Ok(()) => Ok(true),
            Err(e) => Err(e),
          }
        },
        None => Ok(false),
      }
    },
    None => Ok(false),
  }
}

// Factor out handling and passing loading options from input_content,
// to simplify main routine
fn before_input_handle_options(
  options: &mut InputDefinitionOptions,
  prevname: &str,
  prevext: &str,
  name: &str,
  as_type: &str,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<()> {
  // Note: this is trying to emulate the LaTeX 2 (latex.ltx) use of \@pushfilename. For expl3, see
  // expl3.sty.ltxml
  stomach.digest(T_CS!("\\@pushfilename"), state)?;

  // For \RequirePackageWithOptions, pass the options from the outer class/style to the inner one.
  if let Some(with_options_to_pass) = options.withoptions.take() {
    if !prevname.is_empty() && state.has_value(&s!("opt@{}.{}", prevname, prevext)) {
      // Only pass those class options that are declared by the package!
      if let Some(declared_options) = state.lookup_vecdeque("@declaredoptions") {
        let mut topass = Vec::new();
        for op in with_options_to_pass.into_iter() {
          if declared_options.iter().any(|x| {
            if let Stored::String(val) = x {
              arena::with(*val, |str| str == op)
            } else {
              false
            }
          }) {
            topass.push(op)
          }
        }
        if !topass.is_empty() {
          pass_options(name, as_type, topass, state)
        }
      }
    }
  }
  def_macro(
    T_CS!("\\@currname"),
    None,
    Tokens!(Explode!(name)),
    None,
    state,
  );
  def_macro(
    T_CS!("\\@currext"),
    None,
    Tokens!(Explode!(as_type)),
    None,
    state,
  );
  // reset options (Note reset & pass were in opposite order in LoadClass ????)
  let gullet = stomach.get_gullet_mut();
  reset_options(gullet, state)?;
  pass_options(name, as_type, options.options.clone(), state);

  // Note which packages are pretending to be classes.
  if options.as_class {
    state.push_value("@masquerading@as@class", arena::pin(name));
  }
  let current_opt_val = match state.lookup_vecdeque(&s!("opt@{}.{}", name, as_type)) {
    Some(vdq) => {
      let mut pieces = String::new();
      for x in vdq.iter() {
        if let Stored::String(val) = x {
          arena::with(*val, |str| pieces.push_str(str));
        }
        pieces.push(',');
      }
      pieces.pop();
      pieces
    },
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

/// configuration for input of a TeX source (content files mostly)
#[derive(Debug, Default, Clone)]
pub struct InputOptions {
  pub noerror: bool,
  pub file_type: Option<String>,
}

/// used for cases when the file (or data)
/// is plain TeX material that is expected to contribute content
/// to the document (as opposed to pure definitions).
/// A Mouth is opened onto the file, and subsequent reading
/// and/or digestion will pull Tokens from that Mouth until it is
/// exhausted, or closed.
///
/// In some circumstances it may be useful to provide a string containing
/// the TeX material explicitly, rather than referencing a file.
/// In this case, the `literal` pseudo-protocal may be used.

pub fn input_content(
  request: &str,
  options: InputOptions,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<()> {
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

/// This is essentially the `\input` equivalent;
/// we are most likely expecting to get actual content,
/// (possibly with definitions included, as well)
/// but might actually be getting pure definitions,
/// (like a proper style file)
/// in which case we may really want to load a binding.
/// Note that generic style files (non-latex) often have a .tex extension.
pub fn input(
  request: &str,
  options: InputOptions,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<()> {
  // unwrap if in quotes \input{"file name"}
  let mut clean_req = Cow::Borrowed(request);
  while request.starts_with('"') && request.ends_with('"') {
    clean_req = Cow::Owned(QUOTE_WRAPPED.replace(&clean_req, "$1").into_owned());
  }
  // HEURISTIC! First check if equivalent style file, but only under very specific circumstances
  // if pathname_is_literaldata(request) {
  //   let (dir, name, ftype) = pathname_split(request);
  //   let file = name;
  //   if !ftype.is_empty() {
  //     file += format!(".{}",ftype);
  //   }
  //   let path;
  //   // Firstly, check if we are going to OVERRIDE the requested raw .tex file
  //   // with a latexml binding to a style file.
  //   if ((dir.is_empty() && (ftype.is_empty() || (ftype == "tex"))  // No SPECIFIC directory, but
  // a raw tex file.       // AND, in preamble; SHOULD be style file, OR also if we can't find the
  // raw file.     && (LookupValue!("inPreamble") || !FindFile(file))
  //     && (path = FindFile(name, type => 'sty', notex => 1))) { // AND there IS such a style file
  //     Info!("ignore", request, stomach.get_gullet(),
  //       s!("Ignoring input of tex {}, using package {} instead", request, name));
  //     RequirePackage!(name); // Then override, assuming we'll find name as a package file!
  //     return;
  //   }
  // }
  // // Next special case: If we were currently reading a "known" style or binding file,
  // // then this file, even if .tex, must also be definitions rather than content.!!(?)
  if state.lookup_bool("INTERPRETING_DEFINITIONS") {
    input_definitions(
      &clean_req,
      InputDefinitionOptions::default(),
      stomach,
      state,
    )
  } else if let Some(path) = find_file(&clean_req, None, state) {
    // Found something plausible..
    // let ftype = if pathname_is_literaldata(path) { "tex" } else {
    //   pathname_type(path)
    // };

    //   // Should we be doing anything about options in the next 2 cases?..... I kinda think not,
    // but?   if (ftype == "rs") {                  // it's a LaTeXML binding.
    //     load_rtx(request, path);
    //   }
    //   // Else some sort of "known" definitions type file, but not simply 'tex'
    //   else if (ftype != "tex") && (pathname_is_raw(path)) {
    //     load_tex_definitions(request, path);
    //   } else {
    load_tex_content(&path, options, stomach, state)
  //   }
  } else {
    // Couldn't find anything?
    state.note_status("missing", request);

    // We presumably are trying to input Content; an error if we can't find it (contrast to
    // Definitions)
    let gullet = stomach.get_gullet();
    Error!(
      "missing_file",
      request,
      gullet,
      state,
      s!("Can't find TeX file {}", request)
    );
    //  maybeReportSearchPaths());
    Ok(())
  }
}

fn load_tex_definitions(
  request: &str,
  pathname: &str,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<()> {
  if !pathname::is_literaldata(pathname) {
    // We can't analyze literal data's pathnames!
    // let (dir, name, extension) = pathname::split(pathname);

    // Don't load if we've already loaded it before.
    // Note that we'll still load it if we've already loaded only the ltxml version
    // since someone's presumably asking _explicitly_ for the raw TeX version.
    // It's probably even the ltxml version is asking for it!!
    // Of course, now it will be marked and wont get reloaded!
    if state.lookup_bool(&s!("{request}_loaded")) && !pathname::is_reloadable(pathname) {
      return Ok(());
    }
    state.assign_value(&s!("{request}_loaded"), true, Some(Scope::Global));
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
  let content_str = state.lookup_string(&s!("{pathname}_contents"));
  let content = if content_str.is_empty() {
    None
  } else {
    Some(content_str)
  };
  let pathname_mouth = Mouth::create(
    pathname,
    MouthOptions {
      fordefinitions: true,
      notes: true,
      content,
      ..MouthOptions::default()
    },
    state,
  )?;

  stomach.reading_from_mouth(
    pathname_mouth,
    state,
    move |i_stomach, i_state| -> Result<()> {
      while let Some(token) =
        i_stomach
          .get_gullet_mut()
          .read_x_token(Some(false), false, i_state)?
      {
        if token != T_SPACE!() {
          i_stomach.invoke_token(&token, i_state)?;
        }
      }
      Ok(())
    },
  )?;

  state.assign_value("INTERPRETING_DEFINITIONS", was_interpreting, None);
  state.assign_value("INCLUDE_STYLES", was_including_styles, None);
  Ok(())
}

pub fn load_tex_content(
  path: &str,
  _options: InputOptions,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<()> {
  // If there is a file-specific declaration file (name_tex.rs), load it first!
  // TODO: is this `.latexml` variation still relevant in the Rust port?
  let _has_binding = if !pathname::is_literaldata(path) {
    let (_dir, base, _ext) = pathname::split(path);
    load_external_binding(&base, stomach, state)? || load_binding(&base, stomach, state)?
  } else {
    false
  };

  // Open a mouth for that TeX content
  let cached = state.lookup_string(&s!("{path}_contents"));
  let cached_opt = if cached.is_empty() {
    None
  } else {
    Some(cached)
  };
  stomach.get_gullet_mut().open_mouth(
    Mouth::create(
      path,
      MouthOptions {
        notes: true,
        content: cached_opt,
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
fn pass_options(name: &str, ext: &str, options: Vec<String>, state: &mut State) {
  state.push_value(&s!("opt@{}.{}", name, ext), options);
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
  let ext = if state.lookup_definition(&currext_token).is_some() {
    do_expand(currext_token, gullet, state)?.to_string()
  } else {
    String::new()
  };
  let empty_vdq = VecDeque::new(); // convenience for unwrapping empty

  let declared_options: VecDeque<Stored> = state
    .lookup_vecdeque("@declaredoptions")
    .unwrap_or(&empty_vdq)
    .clone();
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
  let mut requested_options: HashSet<String> = HashSet::default();
  for option in current_options.iter() {
    match option {
      Stored::String(content) => {
        requested_options.insert(arena::to_string(*content));
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
        requested_options.insert(arena::to_string(*content));
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
        arena::with(*content, |c_str| {
          if requested_options.contains(c_str) {
            requested_options.remove(c_str); // Remove it, since it's been handled.
            execute_option_internal(c_str, stomach, state)
          } else {
            Ok(true)
          }
        })?;
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
    state.let_i(
      &T_CS!(s!("\\ds@{}", option)),
      T_RELAX!(),
      None,
      stomach.get_gullet_mut(),
    );
  }
  Ok(())
}

fn execute_option_internal(option: &str, stomach: &mut Stomach, state: &mut State) -> Result<bool> {
  let cs = T_CS!(s!("\\ds@{}", option));
  if state.lookup_definition(&cs).is_some() {
    def_macro(
      T_CS!("\\CurrentOption"),
      None,
      Tokens!(T_OTHER!(option)),
      None,
      state,
    );

    let unused = match state.remove_vecdeque("@unusedoptionlist") {
      Some(list) => list
        .into_iter()
        .filter(|item| {
          if let Stored::String(content) = item {
            *content != arena::pin(option)
          } else {
            false
          }
        })
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

fn execute_default_option_internal(
  option: &str,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<bool> {
  def_macro(
    T_CS!("\\CurrentOption"),
    None,
    Tokens!(T_OTHER!(option)),
    None,
    state,
  );
  stomach.digest(T_CS!("\\default@ds"), state)?;
  Ok(true)
}

fn reset_options(gullet: &mut Gullet, state: &mut State) -> Result<()> {
  state.assign_value(
    "@declaredoptions",
    Stored::VecDequeStored(VecDeque::new()),
    None,
  );
  let opt_unused_cs = if do_expand(T_CS!("\\@currext"), gullet, state)?.to_string() == "cls" {
    "\\OptionNotUsed"
  } else {
    "\\@unknownoptionerror"
  };
  state.let_i(&T_CS!("\\default@ds"), T_CS!(opt_unused_cs), None, gullet);
  Ok(())
}

pub struct RequireOptions {
  pub options: Vec<String>,
  pub withoptions: Option<Vec<String>>,
  pub extension: Option<Cow<'static, str>>,
  pub as_class: bool,
  pub noltxml: Option<bool>,
  pub notex: Option<bool>,
  pub after: Tokens,
}
impl Default for RequireOptions {
  fn default() -> Self {
    RequireOptions {
      options: Vec::new(),
      withoptions: None,
      extension: None,
      as_class: false,
      noltxml: None,
      notex: None,
      after: Tokens!(),
    }
  }
}

/// This (and `FindFile`) needs to evolve a bit to support reading raw .sty (.def, etc) files from
/// the standard texmf directories.  Maybe even use kpsewhich itself (INSTEAD of `pathname_find`
/// ???) Another potentially useful option might be that if we are reading a raw file,
/// perhaps it should just get digested immediately, since it shouldn't contribute any boxes.
pub fn require_package(
  name: &str,
  mut options: RequireOptions,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<()> {
  // We'll usually disallow raw TeX, unless the option explicitly given, or globally set.
  if options.notex.is_none()
    && !state.lookup_bool("INCLUDE_STYLES")
    && !matches!(options.noltxml, Some(true))
  {
    options.notex = Some(true);
  }
  if options.extension.is_none() {
    options.extension = Some("sty".into());
  }
  // TODO: Ideally we want to use the same struct for the RequirePackage options as for the
  // InputDefinitions options
  input_definitions(
    name,
    InputDefinitionOptions {
      extension: options.extension,
      handleoptions: true,
      // Pass classes options if we have NONE!
      withoptions: if options.options.is_empty() {
        Some(Vec::new())
      } else {
        None
      }, // fake boolean use, multi-type in latexml... refactor?
      options: options.options,
      as_class: options.as_class,
      noltxml: options.noltxml.unwrap_or(false),
      notex: options.notex.unwrap_or(false),
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
    Warn!(
      "expected",
      "mime-type",
      None,
      state,
      "Resource must have a mime-type; skipping"
    );
    return;
  }

  // If we've got a document, go ahead & put the resource in.
  // if (state.document.is_some()) {
  //   state.document.as_mut().unwrap().add_resource(resource, resource);
  // } else {
  state.pending_resources.push(resource);
  // }
}

pub fn load_class(
  name: &str,
  _options: Vec<String>,
  after: Tokens,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<()> {
  input_definitions(
    name,
    InputDefinitionOptions {
      extension: Some(Cow::Borrowed("cls")),
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

/// configuration for searching for a file in the local filesystem
#[derive(Default)]
pub struct FindFileOptions {
  // TODO: this is no longer used in find_file, rather a level earlier
  pub forbid_ltxml: bool,
  pub notex: bool,
  pub ext_type: Option<Cow<'static, str>>,
  pub search_paths_only: bool,
}

/// search for a file as prescribed by a `FindFileOptions` configuration
pub fn find_file(
  file: &str,
  options: Option<FindFileOptions>,
  state: &mut State,
) -> Option<String> {
  let options = options.unwrap_or_default();
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
    let aux_file = if file.ends_with(ext.as_ref()) {
      file.to_string()
    } else {
      s!("{}.{}", file, ext)
    };
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

fn find_file_aux(file: &str, options: &FindFileOptions, state: &mut State) -> Option<String> {
  // If cached, return simple path (it's a key into the cache)
  let cached = state.lookup_string(&s!("{}_contents", file));
  if !cached.is_empty() {
    Some(file.to_string())
  } else if pathname::is_absolute(file) {
    // And if we've got an absolute path,
    if Path::new(file).exists() {
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
    let _urlbase = state.lookup_value("URLBASE");
    let nopaths = state.lookup_bool("REMOTE_REQUEST");
    let _ltxml_paths: Vec<String> = if nopaths { vec![] } else { paths.clone() };

    // TODO: DG: What do we do instead here? A YAML equivalent with an interpreter? Nothing?
    // If we're looking for ltxml, look within our paths & installation first (faster than kpse)
    // if !options.forbid_ltxml {
    //   if let Some(path) = pathname::find(
    //     &s!("{}.ltxml", file),
    //     PathnameFindOptions {
    //       paths: Some(ltxml_paths),
    //       installation_subdir: Some(String::from("Package")),
    //       ..PathnameFindOptions::default()
    //     },
    //   ) {
    //     return Some(path);
    //   }
    // }
    // If we're looking for TeX, look within our paths & installation first (faster than kpse)
    if !options.notex {
      if let Some(path) = pathname::find(
        file,
        PathnameFindOptions {
          paths: Some(paths),
          ..PathnameFindOptions::default()
        },
      ) {
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
    pathname::kpsewhich(&[file])
  }
}

//======================================================================
// Declaring and Adjusting the Document Model.
//======================================================================

pub fn install_tag(tag: &str, mut properties: TagOptions, state: &mut State) {
  let mut options = state
    .tag_properties
    .entry(arena::pin(tag))
    .or_insert_with(TagOptions::default);
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

/// Selects the RelaxNG schema defining the XML output language
pub fn select_relaxng_schema(
  schema: &str,
  namespaces: Option<HashMap<String, String>>,
  state: &mut State,
) {
  // What verb here? Set, Choose,...
  let model = &mut state.model;
  model.set_relaxng_schema(schema);
  if let Some(namespaces) = namespaces {
    for (prefix, value) in namespaces {
      model.register_document_namespace(&prefix, Some(&value));
    }
  }
}

pub fn merge_font(font: Font, state: &mut State) {
  let new_font = state.lookup_font().unwrap().merge(font);
  state.assign_font(Rc::new(new_font), Some(Scope::Local));
}

pub fn digest_text(stuff: Tokens, stomach: &mut Stomach, state: &mut State) -> Result<Digested> {
  stomach.begin_mode("text", state)?;
  let value = stomach.digest(stuff, state);
  stomach.end_mode("text", state)?;
  value
}

pub fn digest_literal<T: Into<Tokens>>(
  stuff: T,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<Digested> {
  let stuff: Tokens = stuff.into();
  // Perhaps should do StartSemiverbatim, but is it safe to push a frame? (we might cover over
  // valid changes of state!)
  stomach.begin_mode("text", state)?;

  let font = state.lookup_font().unwrap(); // TODO: raise error if font missing
  state.assign_font(
    Rc::new(font.merge(fontmap!(encoding => "ASCII"))),
    Some(Scope::Local),
  ); // try to stay as ASCII as possible

  let value = stomach.digest(stuff, state);
  state.assign_font(font, None);
  stomach.end_mode("text", state)?;
  value
}

pub fn digest_if(
  token: Token,
  stomach: &mut Stomach,
  state: &mut State,
) -> Result<Option<Digested>> {
  if let Some(_defn) = state.lookup_definition(&token) {
    match stomach.digest(Tokens!(token), state) {
      Ok(t) => Ok(Some(t)),
      Err(e) => Err(e),
    }
  } else {
    Ok(None)
  }
}

pub fn build_invocation<T: Into<Token>>(
  token: T,
  args: Vec<Option<Tokens>>,
  gullet: &mut Gullet,
  state: &mut State,
) -> Result<Tokens> {
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
    token.with_cs_name(|csname| Error!("undefined", csname, gullet, state, message));
    let mut invoked_tokens = vec![token];
    // DefConstructor!(token, convert_latex_args(args.len(), 0),
    // sub { LaTeXML::Core::Stomach::makeError($_[0], 'undefined', token); });
    let mut wrapped_args: Vec<Token> = args
      .into_iter()
      .flat_map(|arg_opt| {
        let mut wrapped = vec![T_BEGIN!()];
        if let Some(arg) = arg_opt {
          wrapped.append(&mut arg.unlist());
        }
        wrapped.push(T_END!());
        wrapped
      })
      .collect();
    invoked_tokens.append(&mut wrapped_args);
    Ok(Tokens::new(invoked_tokens))
  }
}

pub fn do_expand<T: Into<Tokens>>(
  tokens: T,
  outer_gullet: &mut Gullet,
  outer_state: &mut State,
) -> Result<Tokens> {
  outer_gullet.do_expand(tokens, outer_state)
}

/// Convert a LaTeX-style argument spec to our Package form.
/// Ie. given $nargs and $optional, being the two optional arguments to
/// something like \newcommand, convert it to the form we use
pub fn convert_latex_args(
  mut nargs: usize,
  optional: Option<Tokens>,
  state: &mut State,
) -> Result<Option<Parameters>> {
  let mut params = Vec::new();
  if let Some(tks) = optional {
    params.push(
      Parameter {
        name: Cow::Borrowed("Optional"),
        spec: Cow::Owned(s!("[Default:{}]", tks.clone().untex())),
        extra: vec![tks],
        ..Parameter::default()
      }
      .init(state)?,
    );
    nargs -= 1;
  }

  for _ in 1..=nargs {
    params.push(
      Parameter {
        name: Cow::Borrowed("Plain"),
        spec: Cow::Borrowed("{}"),
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

pub fn load_font_map<'a>(encoding: &'a str, state: &'a State) -> Option<&'a Fontmap> {
  if let Some(map) = state.lookup_value(&s!("{encoding}_fontmap")) {
    map.into()
  } else {
    None
  }
}
pub fn preload_font_map(encoding: &str, stomach: &mut Stomach, state: &mut State) -> Result<()> {
  // This check is done as a "preload" step for mutability reasons.
  let sym = arena::pin(s!("{encoding}_fontmap"));
  if state.lookup_value_sym(&sym).is_some() {
    return Ok(());
  }
  let fail_key = s!("{encoding}_fontmap_failed_to_load");
  let failed_flag = state.lookup_bool(&fail_key);
  if !failed_flag {
    state.assign_value(&fail_key, true, None); // Stop recursion?
    input_definitions(
      &encoding.to_lowercase(),
      InputDefinitionOptions {
        extension: Some(Cow::Borrowed("fontmap")),
        noerror: true,
        ..InputDefinitionOptions::default()
      },
      stomach,
      state,
    )?;
    if let Some(_map) = state.lookup_value(&s!("{encoding}_fontmap")) {
      // Got map?
      state.assign_value(&fail_key, false, None);
    } else {
      state.assign_value(&fail_key, true, Some(Scope::Global));
    }
  }
  Ok(())
}
