use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::borrow::Cow;
use std::collections::VecDeque;
use std::path::Path;
use std::rc::Rc;

use crate::util::pathname::{self, PathnameFindOptions};
use crate::common::arena;
use crate::common::error::*;
use crate::common::font::{Font, Fontmap};
use crate::document::resource::*;
use crate::document::tag::{TagOptionName, TagOptions};
use crate::mouth::{Mouth, MouthOptions};
use crate::gullet::do_expand;
use crate::parameter::{Parameter, Parameters};
use crate::tokens::Tokens;
use crate::state::*;
use crate::token::*;
use crate::gullet;
use crate::stomach::*;
use crate::{gullet_mut,model_mut};
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
  mut options: InputDefinitionOptions
) -> Result<()> {
  let name = raw_file.trim();
  // Note: we always need a gullet to expand, and we sometimes need a stomach to load_definitions...
  // so let's make stomach a mandatory option.
  let prevname =
    if options.handleoptions && lookup_definition(&T_CS!("\\@currname"))?.is_some() {
      gullet::do_expand(T_CS!("\\@currname"))?.to_string()
    } else {
      String::new()
    };
  let prevext = if options.handleoptions && lookup_definition(&T_CS!("\\@currext"))?.is_some()
  {
    gullet::do_expand(T_CS!("\\@currext"))?.to_string()
  } else {
    String::new()
  };
  // This file will be treated somewhat as if it were a class
  // IF as_class is true
  // OR if it is loaded by such a class, and has withoptions true!!! (yikes)
  if options.handleoptions && options.withoptions.is_some() {
    with_vecdeque("@masquerading@as@class", |vdq_opt| {
      if let Some(vdq) = vdq_opt {
        if vdq.iter().any(|x| {
          if let Stored::String(ref v) = x {
            arena::with(*v, |str| str == prevname)
          } else {
            false
          }
        }) {
          options.as_class = true;
        }
      }});
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
      lookup_value(&s!("{filename}_loaded_with_options"))
    {
      if arena::with(prevoptions, |prev_str| current_options != prev_str) {
        let message = s!(
          "Option clash for file {} with options {:?}, previously loaded with {:?}",
          filename,
          current_options,
          prevoptions
        );
        Info!("unexpected", "options", message);
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
  )?;
  def_macro(
    T_CS!("\\@currext"),
    None,
    Tokens!(Explode!(as_type)),
    None,
  )?;

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
      &as_type)?;
    def_macro(
      T_CS!(s!("\\{}.{}-h@@k", name, as_type)),
      None,
      options.after,
      None,
      )?;
  }

  if !current_options.is_empty() {
    assign_value(
      &s!("{}_loaded_with_options", filename),
      current_options,
      Some(Scope::Global),
    );
  }

  let is_binding = !options.noltxml
    && (load_external_binding(&filename)?
      || load_binding(&filename)?);
  let mut is_found_raw = false;
  if is_binding {
    // We found and loaded a binding successfully, mark it as such.
    let loaded_flag = format!("{filename}_loaded");
    assign_value(&loaded_flag, true, Some(Scope::Global));
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
      ) {
      is_found_raw = true;
      load_tex_definitions(&filename, &file)?;
    } else if !options.noerror {
      // TODO: Proper missing reports
      Warn!(
        "missing_file",
        name,
        s!("Can't find file for {name}")
      );
      // note_status(missing => $name . ($options{type} ? '.' . $options{type} : ''));
      // # We'll only warn about a missing file of definitions: it may be ignorable or never used.
      // # if there ARE problems, they'll likely produce their own errors!
      // Warn('missing_file', $name, $>getStomach->getGullet,
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
    digest(T_CS!(s!("\\{name}.{as_type}-h@@k")))?;
    if !prevname.is_empty() {
      def_macro(
        T_CS!("\\@currname"),
        None,
        Tokens!(Explode!(prevname)),
        None,
          )?;
    }
    if !prevext.is_empty() {
      def_macro(
        T_CS!("\\@currext"),
        None,
        Tokens!(Explode!(prevext)),
        None,
          )?;
    }
    digest(T_CS!("\\@popfilename"))?;
    reset_options()?; // And reset options afterwards, too.
  }
  note_end(&s!("Loading {:?} definitions", filename));
  Ok(())
}

/// loads a binding from the main binding dispatcher, if available+found
pub fn load_binding(file: &str) -> Result<bool> {
  _load_binding(true, file)
}
/// loads a binding from an external binding dispatcher, if available+found
pub fn load_external_binding(file: &str) -> Result<bool> {
  _load_binding(false, file)
}
// in the spirit of Perl's Package::loadLTXML
fn _load_binding(
  internal: bool,
  request: &str,
  ) -> Result<bool> {
  // avoid double-loads, but be binding-specific
  let loaded_key = s!("{request}_binding_loaded");
  if lookup_bool(&loaded_key) {
    return Ok(true);
  }
  // TODO? || lookup_bool(&s!("{trequest}_loaded"))
  //|| lookup_bool(&s!("{name}_loaded")) || lookup_bool(&s!("{ltxname}_loaded"));

  let taken_dispatcher = {if internal {
    get_bindings_dispatch()
  } else {
    get_extra_bindings_dispatch()
  }};
  match taken_dispatcher {
    Some(ref dispatcher) => {
      let result_opt = dispatcher(request);
      match result_opt {
        Some(result) => {
          // Here and only here we are certain we have binding support.
          // Preemptively mark as loaded to avoid recursion.

          // TODO: is this still true?
          // Note (only!) that the binding version of this was loaded; still could load raw tex!
          assign_value(&loaded_key, true, Some(Scope::Global));
          // if a binding load succeeded, mark the generic request as loaded.
          assign_value(&s!("{request}_loaded"), true, Some(Scope::Global));
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
  as_type: &str
) -> Result<()> {
  // Note: this is trying to emulate the LaTeX 2 (latex.ltx) use of \@pushfilename. For expl3, see
  // expl3.sty.ltxml
  digest(T_CS!("\\@pushfilename"))?;

  // For \RequirePackageWithOptions, pass the options from the outer class/style to the inner one.
  if let Some(with_options_to_pass) = options.withoptions.take() {
    if !prevname.is_empty() && has_value(&s!("opt@{}.{}", prevname, prevext)) {
      // Only pass those class options that are declared by the package!
      let mut topass = Vec::new();
      with_vecdeque("@declaredoptions",|vdq_opt| if let Some(declared_options) = vdq_opt {
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
      });
      if !topass.is_empty() {
        pass_options(name, as_type, topass)?;
      }
    }
  }
  def_macro(
    T_CS!("\\@currname"),
    None,
    Tokens!(Explode!(name)),
    None,
  )?;
  def_macro(
    T_CS!("\\@currext"),
    None,
    Tokens!(Explode!(as_type)),
    None,
  )?;
  // reset options (Note reset & pass were in opposite order in LoadClass ????)
  reset_options()?;
  pass_options(name, as_type, options.options.clone())?;

  // Note which packages are pretending to be classes.
  if options.as_class {
    push_value("@masquerading@as@class", arena::pin(name))?;
  }
  let current_opt_val = with_vecdeque(&s!("opt@{}.{}", name, as_type), |vdq_opt|
    match vdq_opt {
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
      None => String::new()
    });
  def_macro(
    T_CS!(s!("\\opt@{}.{}", name, as_type)),
    None,
    Tokens!(Explode!(current_opt_val)),
    None,
  )?;
  Ok(())
}

/// configuration for input of a TeX source (content files mostly)
#[derive(Debug, Default, Clone)]
pub struct InputOptions {
  pub noerror: bool,
  pub reloadable: bool,
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
  ) -> Result<()> {
  let filepath = find_file(request, None);
  match filepath {
    // TODO: type => $options{type}, noltxml => 1
    Some(path) => load_tex_content(&path, options),
    None => fatal!(Package, MissingFile, request),
    /* TODO:
     * Error("missing_file", request, state!().get_stomach().get_gullet(),
     * "Can't find TeX file "+request, maybeReportSearchPaths())) */
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
  if lookup_bool("INTERPRETING_DEFINITIONS") {
    input_definitions(
      &clean_req,
      InputDefinitionOptions::default())
  } else if let Some(path) = find_file(&clean_req, None) {
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
    load_tex_content(&path, options)
  //   }
  } else {
    // Couldn't find anything?
    note_status(LogStatus::Missing, Some(request));

    // We presumably are trying to input Content; an error if we can't find it (contrast to
    // Definitions)
    Error!(
      "missing_file",
      request,
      s!("Can't find TeX file {}", request)
    );
    //  maybeReportSearchPaths());
    Ok(())
  }
}

fn load_tex_definitions(
  request: &str,
  pathname: &str,
  ) -> Result<()> {
  if !pathname::is_literaldata(pathname) {
    // We can't analyze literal data's pathnames!
    // let (dir, name, extension) = pathname::split(pathname);

    // Don't load if we've already loaded it before.
    // Note that we'll still load it if we've already loaded only the ltxml version
    // since someone's presumably asking _explicitly_ for the raw TeX version.
    // It's probably even the ltxml version is asking for it!!
    // Of course, now it will be marked and wont get reloaded!
    if lookup_bool(&s!("{request}_loaded")) && !pathname::is_reloadable(pathname) {
      return Ok(());
    }
    assign_value(&s!("{request}_loaded"), true, Some(Scope::Global));
  }

  // Note that we are reading definitions (and recursive input is assumed also definitions)
  let was_interpreting = lookup_bool("INTERPRETING_DEFINITIONS");
  // And that if we're interpreting this TeX file of definitions,
  // we probably should interpret any TeX files IT loads.
  let was_including_styles = lookup_bool("INCLUDE_STYLES");
  assign_value("INTERPRETING_DEFINITIONS", true, None);
  // If we're reading in these definitions, probaly will accept included ones?
  // (but not forbid ltxml ?)
  assign_value("INCLUDE_STYLES", true, None);
  // When set, this variable allows redefinitions of locked defns.
  // It is set in before/after methods to allow local rebinding of commands
  // but loading of sources & bindings is typically done in before/after methods of constructors!
  // This re-locks defns during reading of TeX packages.
  set_locked_state();
  let content_str = lookup_string(&s!("{pathname}_contents"));
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
  )?;

  gullet::reading_from_mouth(
    pathname_mouth,
    move || -> Result<()> {
      while let Some(token) =
        gullet::read_x_token(Some(false), false)?
      {
        if token != T_SPACE!() {
          invoke_token(&token)?;
        }
      }
      Ok(())
    },
  )?;

  assign_value("INTERPRETING_DEFINITIONS", was_interpreting, None);
  assign_value("INCLUDE_STYLES", was_including_styles, None);
  Ok(())
}

pub fn load_tex_content(
  path: &str,
  _options: InputOptions,
  ) -> Result<()> {
  // If there is a file-specific declaration file (name_tex.rs), load it first!
  // TODO: is this `.latexml` variation still relevant in the Rust port?
  let _has_binding = if !pathname::is_literaldata(path) {
    let (_dir, base, _ext) = pathname::split(path);
    load_external_binding(&base)? || load_binding(&base)?
  } else {
    false
  };

  // Open a mouth for that TeX content
  let cached = lookup_string(&s!("{path}_contents"));
  let cached_opt = if cached.is_empty() {
    None
  } else {
    Some(cached)
  };
  gullet_mut!().open_mouth(
    Mouth::create(
      path,
      MouthOptions {
        notes: true,
        content: cached_opt,
        ..MouthOptions::default()
      },
      )?,
    true,
  );
  Ok(())
}

/// Pass the sequence of @options to the package $name (if $ext is 'sty'),
/// or class $name (if $ext is 'cls').
fn pass_options(name: &str, ext: &str, options: Vec<String>) -> Result<()> {
  push_value(&s!("opt@{}.{}", name, ext), options)
}

pub fn process_options() -> Result<()> {
  let currname_token = T_CS!("\\@currname");
  let currext_token = T_CS!("\\@currext");
  let name = if lookup_definition(&currname_token)?.is_some() {
    do_expand(currname_token)?.to_string()
  } else {
    String::new()
  };
  let ext = if lookup_definition(&currext_token)?.is_some() {
    do_expand(currext_token)?.to_string()
  } else {
    String::new()
  };
  let declared_options: VecDeque<Stored> = lookup_vecdeque("@declaredoptions")
    .unwrap_or_default();
  let opt_key = s!("opt@{}.{}", name, ext);
  let current_options = lookup_vecdeque(&opt_key).unwrap_or_default();
  let class_options = lookup_vecdeque("class_options").unwrap_or_default();
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
            execute_option_internal(c_str)
          } else {
            Ok(true)
          }
        })?;
      },
      Stored::VecString(contents) => {
        for content in contents.iter() {
          if requested_options.contains(content) {
            requested_options.remove(content); // Remove it, since it's been handled.
            execute_option_internal(content)?;
          }
        }
      },
      _ => {},
    }
  }
  // Now handle any remaining options (eg. default options), in the given order.
  for option in requested_options.iter() {
    execute_default_option_internal(option)?;
  }
  // Now, undefine the handlers?
  for option in declared_options.iter() {
    let_i(
      &T_CS!(s!("\\ds@{}", option)),
      &T_RELAX!(),
      None
    );
  }
  Ok(())
}

fn execute_option_internal(option: &str) -> Result<bool> {
  let cs = T_CS!(s!("\\ds@{}", option));
  if lookup_definition(&cs)?.is_some() {
    def_macro(
      T_CS!("\\CurrentOption"),
      None,
      Tokens!(T_OTHER!(option)),
      None,
      )?;

    let unused = match remove_vecdeque("@unusedoptionlist") {
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
    assign_value("@unusedoptionlist", Stored::VecDequeStored(unused), None);
    digest(cs)?;
    Ok(true)
  } else {
    Ok(false)
  }
}

fn execute_default_option_internal(
  option: &str,
  ) -> Result<bool> {
  def_macro(
    T_CS!("\\CurrentOption"),
    None,
    Tokens!(T_OTHER!(option)),
    None,
  )?;
  digest(T_CS!("\\default@ds"))?;
  Ok(true)
}

fn reset_options() -> Result<()> {
  assign_value(
    "@declaredoptions",
    Stored::VecDequeStored(VecDeque::new()),
    None,
  );
  let opt_unused_cs = if gullet::do_expand(T_CS!("\\@currext"))?.to_string() == "cls" {
    "\\OptionNotUsed"
  } else {
    "\\@unknownoptionerror"
  };
  let_i(&T_CS!("\\default@ds"), &T_CS!(opt_unused_cs), None);
  Ok(())
}

pub struct RequireOptions {
  pub options: Vec<String>,
  pub withoptions: Option<Vec<String>>,
  pub extension: Option<Cow<'static, str>>,
  pub searchpaths_only: bool,
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
      notex: None,
      noltxml: None,
      as_class: false,
      searchpaths_only: false,
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
  ) -> Result<()> {
  // We'll usually disallow raw TeX, unless the option explicitly given, or globally set.
  if options.notex.is_none()
    && !lookup_bool("INCLUDE_STYLES")
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
    }
  )
}

pub fn require_resource(mut resource: Resource) {
  if resource.name.is_empty() && resource.content.is_empty() {
    Warn!(
      "expected",
      "resource",
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
      "Resource must have a mime-type; skipping"
    );
    return;
  }

  // If we've got a document, go ahead & put the resource in.
  // if (document.is_some()) {
  //   document.as_mut().unwrap().add_resource(resource, resource);
  // } else {
  push_pending_resource(resource);
  // }
}

pub fn load_class(
  name: &str,
  _options: Vec<String>,
  after: Tokens,
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
    }
  )
  // if (let success = InputDefinitions($class, type => 'cls', notex => 1, handleoptions => 1,
  // noerror => 1,     %options)) {
  //   return $success; }
  // else {
  //   $>noteStatus(missing => $class . '.cls');
  //   let alternate = 'OmniBus';    # was 'article'
  //   Warn('missing_file', $class, $>getStomach->getGullet,
  //     "Can't find binding for class $class (using $alternate)",
  //     maybeReportSearchPaths());
  // if (let success = InputDefinitions($alternate, type => 'cls', noerror => 1, handleoptions =>
  // 1, %options)) {     return $success; }
  //   else {
  //     Fatal('missing_file', $alternate . '.cls.ltxml', $>getStomach->getGullet,
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
    find_file_aux(&aux_file, &options)
  } else if file.ends_with(".tex") {
    // If no type given, we MAY expect .tex, or maybe NOT!!
    // No requested type, then .tex; Of course, it may already have it!
    find_file_aux(file, &options)
  } else {
    match find_file_aux(&s!("{}.tex", file), &options) {
      None => find_file_aux(file, &options),
      Some(f) => Some(f),
    }
  }
}

fn find_file_aux(file: &str, options: &FindFileOptions) -> Option<String> {
  // If cached, return simple path (it's a key into the cache)
  let cached = lookup_string(&s!("{}_contents", file));
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
    let paths: Vec<String> = get_search_paths();
    // let _urlbase = state!().lookup_value("URLBASE");
    // let _nopaths = lookup_bool("REMOTE_REQUEST");
    // let _ltxml_paths: Vec<String> = if nopaths { vec![] } else { paths.clone() };

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

pub fn install_tag(tag: &str, mut properties: TagOptions) {
  let tag_ticket = arena::pin(tag);
  with_tag_property_mut(tag_ticket, |options| {
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
  }});
}

/// Selects the RelaxNG schema defining the XML output language
pub fn select_relaxng_schema(
  schema: &str,
  namespaces: Option<HashMap<String, String>>,
) {
  // What verb here? Set, Choose,...
  model_mut!().set_relaxng_schema(schema);
  if let Some(namespaces) = namespaces {
    for (prefix, value) in namespaces {
      model_mut!().register_document_namespace(&prefix, Some(&value));
    }
  }
}

pub fn merge_font(font: Font) {
  let new_font = lookup_font().unwrap().merge(font);
  assign_font(Rc::new(new_font), Some(Scope::Local));
}

pub fn digest_text(stuff: Tokens) -> Result<Digested> {
  begin_mode("text")?;
  let value = digest(stuff);
  end_mode("text")?;
  value
}

pub fn digest_literal<T: Into<Tokens>>(
  stuff: T,
  ) -> Result<Digested> {
  let stuff: Tokens = stuff.into();
  // Perhaps should do StartSemiverbatim, but is it safe to push a frame? (we might cover over
  // valid changes of )
  begin_mode("text")?;

  let font = lookup_font().unwrap(); // TODO: raise error if font missing
  assign_font(
    Rc::new(font.merge(fontmap!(encoding => "ASCII"))),
    Some(Scope::Local),
  ); // try to stay as ASCII as possible

  let value = digest(stuff);
  assign_font(font, None);
  end_mode("text")?;
  value
}

pub fn digest_if(
  token: Token,
  ) -> Result<Option<Digested>> {
  if lookup_definition(&token)?.is_some() {
    match digest(Tokens!(token)) {
      Ok(t) => Ok(Some(t)),
      Err(e) => Err(e),
    }
  } else {
    Ok(None)
  }
}

pub fn build_invocation<T: Into<Token>>(
  token: T,
  args: Vec<Option<Tokens>>
) -> Result<Tokens> {
  let token: Token = token.into();
  // Note: token may have been \let to another defn!
  if let Some(defn) = lookup_definition(&token)? {
    let mut invoked_tokens = vec![token];
    let mut reverted_args = if let Some(params) = defn.get_parameters() {
      params.revert_arguments(args)?
    } else {
      Vec::new()
    };
    invoked_tokens.append(&mut reverted_args);
    Ok(Tokens::new(invoked_tokens))
  } else {
    let message = s!("Can't invoke {:?}; it is undefined", token.stringify());
    token.with_cs_name(|csname| { Error!("undefined", csname, message); Ok(()) })?;
    let mut invoked_tokens = vec![token];
    // DefConstructor!(token, convert_latex_args(args.len(), 0),
    // sub { LaTeXML::Core::makeError($_[0], 'undefined', token); });
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

/// Convert a LaTeX-style argument spec to our Package form.
/// Ie. given $nargs and $optional, being the two optional arguments to
/// something like \newcommand, convert it to the form we use
pub fn convert_latex_args(
  mut nargs: usize,
  optional: Option<Tokens>,
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
      .init()?,
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
      .init()?,
    );
  }
  if params.is_empty() {
    Ok(None)
  } else {
    Ok(Some(Parameters::new(params)))
  }
}

pub fn load_font_map(encoding: &str) -> Option<Fontmap> {
  preload_font_map(encoding).expect("preloading font map should succeed.");
  if let Some(map) = lookup_value(&s!("{encoding}_fontmap")) {
    map.into()
  } else {
    None
  }
}
pub fn preload_font_map(encoding: &str) -> Result<()> {
  // This check is done as a "preload" step for mutability reasons.
  let key = s!("{encoding}_fontmap");
  if has_value(&key) {
    return Ok(());
  }
  let fail_key = s!("{encoding}_fontmap_failed_to_load");
  let failed_flag = lookup_bool(&fail_key);
  if !failed_flag {
    assign_value(&fail_key, true, None); // Stop recursion?
    input_definitions(
      &encoding.to_lowercase(),
      InputDefinitionOptions {
        extension: Some(Cow::Borrowed("fontmap")),
        noerror: true,
        ..InputDefinitionOptions::default()
      }
    )?;
    if has_value(&s!("{encoding}_fontmap")) {
      // Got map?
      assign_value(&fail_key, false, None);
    } else {
      assign_value(&fail_key, true, Some(Scope::Global));
    }
  }
  Ok(())
}
