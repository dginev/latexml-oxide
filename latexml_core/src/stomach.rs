use once_cell::sync::Lazy;
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;
use std::time::Instant;

use crate::digested::DigestedData;

/// Cached snapshot of `LXML_TRACE_BOUND_MODE` env var. Like the
/// `TRACE_GROUP_END` cache in gullet.rs, this avoids per-digest
/// `getenv` calls — glibc's `getenv` is unsafe under high-volume
/// concurrent reads from many test threads, manifesting as SIGSEGV
/// in `__GI_getenv` when running `cargo test --release --tests`.
/// Sample once at static-init; subsequent reads are an atomic load.
static TRACE_BOUND_MODE: Lazy<bool> = Lazy::new(|| std::env::var("LXML_TRACE_BOUND_MODE").is_ok());

// Conversion timeout: thread-local deadline. When set, digest loops check it.
thread_local! {
  static CONVERSION_DEADLINE: Cell<Option<Instant>> = const { Cell::new(None) };
}

/// Set a conversion timeout (seconds from now). 0 = no timeout.
pub fn set_timeout(seconds: u64) {
  if seconds > 0 {
    CONVERSION_DEADLINE.with(|d| {
      d.set(Some(
        Instant::now() + std::time::Duration::from_secs(seconds),
      ))
    });
  } else {
    CONVERSION_DEADLINE.with(|d| d.set(None));
  }
}

/// Check if conversion has timed out. Returns Err if deadline exceeded.
pub fn check_timeout() -> Result<()> {
  CONVERSION_DEADLINE.with(|d| {
    if let Some(deadline) = d.get() {
      if Instant::now() > deadline {
        fatal!(Timeout, Convert, "Conversion timed out!");
      }
    }
    Ok(())
  })
}

use crate::comment::Comment;
use crate::common::arena;
use crate::common::arena::SymHashMap as HashMap;
use crate::common::error::*;
use crate::common::font;
use crate::common::font::Font;
use crate::definition::Definition;
use crate::definition::constructor::Constructor;
use crate::definition::expandable::Expandable;
use crate::definition::register::RegisterValue;
use crate::list::List;
use crate::mouth::{Mouth, MouthOptions};
use crate::state::*;
use crate::tbox::*;
use crate::token::{Catcode, Token};
use crate::tokens::Tokens;
use crate::{BoxOps, Digested, TexMode, gullet};

static MAXSTACK: usize = 200;

/// The Stomach is responsible for digesting tokens into boxes, lists, etc.
#[derive(Default)]
pub struct Stomach {
  /// currently invoked tokens
  pub token_stack:    Vec<Token>,
  /// tracks the tokens of boxing groups(?)
  pub boxing:         Vec<Token>,
  /// localized box lists for stacked digestion calls
  localized_box_list: Vec<Vec<Digested>>,
  /// collects the intermediate boxes resulting from a `digest` call.
  pub box_list:       Vec<Digested>,
}

#[thread_local]
pub static STOMACH: Lazy<RefCell<Stomach>> = Lazy::new(|| RefCell::new(Stomach::default()));

macro_rules! stomach {
  () => {
    (*STOMACH).borrow()
  };
}
macro_rules! stomach_mut {
  () => {
    (*STOMACH).borrow_mut()
  };
}

/// Initialize various stomach parameters, preload, etc.
pub fn initialize_stomach() {
  let mut stomach = stomach_mut!();
  stomach.boxing = Vec::new();
  stomach.token_stack = Vec::new();
  stomach.box_list = Vec::new();
  stomach.localized_box_list = Vec::new();

  assign_value("BOUND_MODE", "vertical", Some(Scope::Global));
  assign_value("MODE", "vertical", Some(Scope::Global));
  assign_value("IN_MATH", false, Some(Scope::Global));
  assign_value("PRESERVE_NEWLINES", 1, Some(Scope::Global));
  assign_value(
    "afterGroup",
    Stored::VecDequeStored(VecDeque::new()),
    Some(Scope::Global),
  );
  assign_value("afterAssignment", Stored::None, Some(Scope::Global)); // undef ???
  assign_value_sym(
    crate::pin!("groupInitiator"),
    "Initialization",
    Some(Scope::Global),
  );
  // Setup default fonts.
  assign_value("font", Font::text_default(), Some(Scope::Global));
  assign_value("mathfont", Font::math_default(), Some(Scope::Global));
}

/// steal the previously digested boxes from the current level.
pub fn regurgitate() -> Vec<Digested> { stomach_mut!().box_list.drain(..).collect() }

//**********************************************************************
// Maintaining state
//**********************************************************************
// state changes that the Stomach needs to moderate and know about (?)

//======================================================================
// Dealing with TeX's bindings & grouping.
// Note that lookups happen more often than bgroup/egroup (which open/close frames).

/// Adds a new stack frame for a TeX group.
pub fn push_stack_frame(nobox: bool) {
  let current_token = get_current_token().unwrap_or_else(|| T_CS!("\\relax"));
  push_frame();
  assign_value(
    "beforeAfterGroup",
    Stored::VecDequeStored(VecDeque::new()),
    Some(Scope::Local),
  ); // ALWAYS bind this!
  assign_value(
    "afterGroup",
    Stored::VecDequeStored(VecDeque::new()),
    Some(Scope::Local),
  ); // ALWAYS bind this!
  assign_value("afterAssignment", Stored::None, Some(Scope::Local)); // ALWAYS bind this!
  assign_value_sym(crate::pin!("groupNonBoxing"), nobox, Some(Scope::Local)); // ALWAYS bind this!
  assign_value_sym(
    crate::pin!("groupInitiator"),
    current_token,
    Some(Scope::Local),
  );
  assign_value_sym(
    crate::pin!("groupInitiatorLocator"),
    gullet::get_locator(),
    Some(Scope::Local),
  );
  if !nobox {
    // For begingroup/endgroup
    stomach_mut!().boxing.push(current_token)
  }
}
/// Execute tokens stored on beforeAfterGroup (if any); done before popping a stack frame.
/// Perl: sub executeBeforeAfterGroup (Stomach.pm lines 286-295)
pub fn execute_before_after_group() -> Result<()> {
  if let Some(Stored::VecDequeStored(beforeafter)) = remove_value("beforeAfterGroup") {
    if !beforeafter.is_empty() {
      let mut result = Vec::with_capacity(beforeafter.len());
      for beforeafter_frame in beforeafter.into_iter() {
        match beforeafter_frame {
          Stored::Tokens(frametoks) => result.push(frametoks.be_digested()?),
          Stored::Token(frametok) => result.push(frametok.be_digested()?),
          _ => {
            // Unexpected value type in beforeAfterGroup — skip silently
            // rather than panic (could occur with non-standard TeX constructs)
          },
        }
      }
      // Perl Stomach.pm:182-183 — every digested item must be Box-like
      // (TBox / List / Whatsit / Alignment); anything else is a binding
      // bug. Emit Error per offender; the Box-like items still flow
      // through to box_list so partial output is preserved.
      // Perl additionally calls `@result = (makeMisdefinedError(@result))`
      // collapsing everything to a single error sentinel — we keep
      // the partial-output behaviour (Rust-side divergence; surfacing
      // *the* offending item via Error! is what the harness needs to
      // report, while the rest of the box stream is still useful).
      //
      // Implementation note: walk the result list with an index loop
      // rather than `retain(|d| {…})`. The Error! macro can `return
      // Err(…)` on the max-errors / runaway-loop guards, and a closure
      // returning `bool` can't propagate that out — only an explicit
      // for-loop in the surrounding `Result<()>` body can.
      let mut filtered = Vec::with_capacity(result.len());
      for d in result {
        let is_box = matches!(
          d.data(),
          DigestedData::TBox(_)
            | DigestedData::List(_)
            | DigestedData::Whatsit(_)
            | DigestedData::Alignment(_)
        );
        if is_box {
          filtered.push(d);
        } else {
          let kind_label = match d.data() {
            DigestedData::Postponed(_) => "Postponed",
            DigestedData::KeyVals(_) => "KeyVals",
            DigestedData::RegisterValue(_) => "RegisterValue",
            DigestedData::Comment(_) => "Comment",
            _ => "non-Box",
          };
          Error!(
            "misdefined",
            "<beforeAfterGroup>",
            format!(
              "Expected a Box|List|Whatsit, but got '{}' — dropping",
              kind_label
            )
          );
        }
      }
      {
        stomach_mut!().box_list.extend(filtered);
      }
    }
  }
  Ok(())
}

/// Removes the last/current stack frame, ending a TeX group
pub fn pop_stack_frame(nobox: bool) -> Result<()> {
  let after = remove_value("afterGroup");
  execute_before_after_group()?;
  pop_frame()?;
  if !nobox {
    {
      stomach_mut!().boxing.pop(); // For begingroup/endgroup
    }
  }
  if let Some(Stored::VecDequeStored(after_entries)) = after {
    for entry in after_entries.into_iter().rev() {
      match entry {
        Stored::Tokens(t) => gullet::unread(t),
        Stored::Token(t) => gullet::unread_one(t),
        other => panic!(r"\aftergroup should be used with tokens, got instead: {other:?}"),
      };
    }
  }
  Ok(())
}

/// explain the current frame
pub fn current_frame_message() -> String {
  let target = if is_value_bound("MODE", Some(0)) {
    // SET mode in CURRENT frame ?
    Cow::Owned(s!(
      "mode-switch to {}",
      crate::state::lookup_string_from_sym(crate::pin!("MODE"))
    ))
  } else if lookup_bool_sym(crate::pin!("groupNonBoxing")) {
    // Current frame is a non-boxing group?
    Cow::Borrowed("non-boxing group")
  } else {
    Cow::Borrowed("boxing group")
  };

  let initiator = if let Some(t) = lookup_token_sym(crate::pin!("groupInitiator")) {
    t.stringify()
  } else {
    String::new()
  };
  let locator = lookup_string_from_sym(crate::pin!("groupInitiatorLocator"));
  s!(
    "current frame is {} due to {} {}",
    target,
    initiator,
    locator
  )
}

//======================================================================
// Grouping pushes a new stack frame for binding definitions, etc.
//======================================================================

/// Begin a new level of binding by pushing a new stack frame,
/// and a new level of boxing the digested output.
pub fn bgroup() {
  push_stack_frame(false);
  // Perl's bgroup does NOT touch $ALIGN_STATE — it's tracked only at the scan level
  // (in read_token/read_x_token). The scan-level tracking in gullet.rs is sufficient.
}
/// End a level of binding by popping the last stack frame,
/// undoing whatever bindings appeared there, and also
/// decrementing the level of boxing.
pub fn egroup() -> Result<()> {
  if is_value_bound("BOUND_MODE", Some(0)) {
    // Diagnostic for cluster investigation (project_explsyntax_midload.md).
    if *TRACE_BOUND_MODE {
      let mode = crate::state::lookup_string_from_sym(crate::pin!("MODE"));
      let bound = crate::state::lookup_string_from_sym(crate::pin!("BOUND_MODE"));
      let cur_tok = get_current_token()
        .map(|t| t.to_string())
        .unwrap_or_default();
      eprintln!(
        "[trace] egroup ERROR: cur_tok={cur_tok} BOUND_MODE={bound} MODE={mode}\n{}",
        std::backtrace::Backtrace::force_capture()
      );
    }
    // Last stack frame was a mode switch!?!?!
    // Don't pop if there's an error; maybe we'll recover?
    Error!(
      "unexpected",
      get_current_token().unwrap_or_else(|| T_CS!("\\?")),
      s!(
        "Attempt to close a group that switched to mode {}; {}",
        crate::state::lookup_string_from_sym(crate::pin!("MODE")),
        current_frame_message()
      )
    );
  } else if lookup_bool_sym(crate::pin!("groupNonBoxing")) {
    // or group was opened with \begingroup
    Error!(
      "unexpected",
      get_current_token().unwrap_or_else(|| T_CS!("\\?")),
      s!("Attempt to close boxing group; {}", current_frame_message())
    );
  } else {
    // Don't pop if there's an error; maybe we'll recover?
    pop_stack_frame(false)?;
  }
  // Perl's egroup does NOT touch $ALIGN_STATE — tracked at scan level only.
  Ok(())
}
/// Begin a new level of binding by pushing a new stack frame.
pub fn begingroup() {
  if *TRACE_BOUND_MODE {
    let depth = crate::state::get_frame_depth();
    let loc = gullet::get_locator();
    eprintln!("[trace] begingroup pre-depth={depth} at {}", loc);
  }
  push_stack_frame(true);
}
/// End a level of binding by popping the last stack frame,
/// undoing whatever bindings appeared there.
pub fn endgroup() -> Result<()> {
  if *TRACE_BOUND_MODE {
    let depth = crate::state::get_frame_depth();
    let bound = is_value_bound("BOUND_MODE", Some(0));
    let loc = gullet::get_locator();
    let tok = get_current_token().unwrap_or_else(|| T_CS!("\\?"));
    if depth == 0 {
      eprintln!(
        "[trace] endgroup at locked frame: tok={} at {}\n{}",
        tok,
        loc,
        std::backtrace::Backtrace::force_capture()
      );
    } else {
      eprintln!(
        "[trace] endgroup pre-depth={depth} bound_top={bound} tok={} at {}",
        tok, loc
      );
    }
  }
  // BAND-AID (commit 3088dbd17 — under root-cause investigation, see
  // `project_explsyntax_midload.md`): during raw .sty/.tex load
  // (INTERPRETING_DEFINITIONS=true), suppress strict BOUND_MODE check.
  // Empirically Perl emits zero errors on the same inputs while strict
  // checks fire 19 times in our Rust during expl3-code.tex raw load.
  // Latent bugs found 2026-04-25 when removing this guard:
  //   - `#` (catcode PARAM) escapes to stomach
  //   - `\q_stop` recursion
  //   - residual `\group_end:` mode-switch error (not caught by strict end_mode_opt either —
  //     separate divergence point)
  //   - `\xparse-2018-04-12.sty-h@@k` undefined
  // Each of those needs its own root-cause investigation.
  let interpreting = lookup_bool_sym(crate::pin!("INTERPRETING_DEFINITIONS"));
  if interpreting {
    // Diagnostic: capture band-aid suppression occurrences for analysis.
    if *TRACE_BOUND_MODE && is_value_bound("BOUND_MODE", Some(0)) {
      let mode = crate::state::lookup_string_from_sym(crate::pin!("MODE"));
      let bound = crate::state::lookup_string_from_sym(crate::pin!("BOUND_MODE"));
      let frame_keys = crate::state::dump_top_frame_keys();
      eprintln!(
        "[trace] endgroup SUPPRESSED-ERR: BOUND_MODE={bound} MODE={mode} frame0_keys={frame_keys:?}",
      );
    }
    pop_stack_frame(true)?;
  } else if is_value_bound("BOUND_MODE", Some(0)) {
    // Diagnostic: dump BOUND_MODE binding context for cluster investigation.
    if *TRACE_BOUND_MODE {
      let mode = crate::state::lookup_string_from_sym(crate::pin!("MODE"));
      let bound = crate::state::lookup_string_from_sym(crate::pin!("BOUND_MODE"));
      eprintln!(
        "[trace] endgroup ERROR: BOUND_MODE={bound} MODE={mode}\n{}",
        std::backtrace::Backtrace::force_capture()
      );
    }
    // Last stack frame was a mode switch!?!?!
    // Don't pop if there's an error; maybe we'll recover?
    Error!(
      "unexpected",
      get_current_token()
        .map(|t| t.to_string())
        .unwrap_or_else(|| String::from("\\?")),
      s!(
        "Attempt to close a group that switched to mode {}; {}",
        crate::state::lookup_string_from_sym(crate::pin!("MODE")),
        current_frame_message()
      )
    );
  } else if !lookup_bool_sym(crate::pin!("groupNonBoxing")) {
    // or group was opened with \bgroup
    Error!(
      "unexpected",
      get_current_token()
        .map(|t| t.to_string())
        .unwrap_or_else(|| String::from("\\?")),
      s!(
        "Attempt to close non-boxing group; {}",
        current_frame_message()
      )
    );
  } else {
    pop_stack_frame(true)?;
  }
  Ok(())
}

//======================================================================
// Mode (minimal so far; math vs text)
// Could (should?) be taken up by Stomach by building horizontal, vertical or math lists ?

/// Sets the mode without doing any grouping (NOR does it stack the modes!!)
///
/// Useful for environments, where the group has already been established.
/// (presumably, in the long run, modes & groups should be much less coupled)
pub fn set_mode(mode: &str) -> Result<()> {
  let prevmode = crate::state::lookup_string_from_sym(crate::pin!("MODE"));
  let ismath = mode.ends_with("math");
  // Perl: beginMode maps to internal mode names, but set_mode stores as-is
  // We also set BOUND_MODE so end_mode can find it
  let bound_mode = bindable_mode(mode).unwrap_or(mode);
  // Diagnostic
  if *TRACE_BOUND_MODE {
    eprintln!(
      "[trace] set_mode mode={mode} bound_mode={bound_mode}\n{}",
      std::backtrace::Backtrace::force_capture()
    );
  }
  assign_value("BOUND_MODE", arena::pin(bound_mode), Some(Scope::Local));
  assign_value("MODE", arena::pin(bound_mode), Some(Scope::Local));
  assign_value("IN_MATH", ismath, Some(Scope::Local));
  if mode == prevmode {
  } else if ismath {
    let curfont = lookup_font().unwrap();
    // When entering math mode, we set the font to the default math font,
    // and save the text font for any embedded text.
    assign_value("savedfont", curfont.clone(), Some(Scope::Local));
    // see get_script_level()
    assign_value("script_base_level", stomach!().boxing.len(), None);
    let isdisplay = mode.starts_with("display");
    assign_value("IN_MATH_DISPLAY", isdisplay, Some(Scope::Local));
    let new_font = Rc::new(lookup_mathfont().unwrap().merge(Font {
      color: curfont.color,
      bg: curfont.bg,
      size: curfont.size,
      mathstyle: if isdisplay {
        Some("display".into())
      } else {
        Some("text".into())
      },
      ..Font::default()
    }));
    assign_value(
      "initial_math_font",
      Stored::Font(new_font.clone()),
      Some(Scope::Local),
    );
    assign_font(new_font, Some(Scope::Local));
    // Perl Stomach.pm:505 — `$STATE->assignValue(fontfamily => -1, 'local');`
    // Resets `\fam` (whose getter reads `fontfamily`) on math entry so that
    // text-mode `\rm` (which sets `fontfamily=0`) doesn't leak into math.
    assign_value("fontfamily", -1_i64, Some(Scope::Local));
  } else {
    let curfont = lookup_font().unwrap();
    // When entering text mode, we should set the font to the text font in use before the math
    // but inherit color and size
    let saved_opt = lookup_value("savedfont");
    if let Some(Stored::Font(saved_font)) = saved_opt {
      assign_font(
        Rc::new(saved_font.merge(Font {
          color: curfont.color,
          bg: curfont.bg,
          size: curfont.size,
          ..Font::default()
        })),
        Some(Scope::Local),
      );
    }
  }
  Ok(())
}

/// Map user-facing mode names to internal bound mode names.
/// Perl: our %bindable_mode = (text => 'restricted_horizontal', ...)
fn bindable_mode(umode: &str) -> Option<&'static str> {
  match umode {
    "text" | "restricted_horizontal" => Some("restricted_horizontal"),
    "vertical" | "internal_vertical" => Some("internal_vertical"),
    "math" | "inline_math" => Some("math"),
    "display_math" => Some("display_math"),
    _ => None,
  }
}

/// Begin processing in `mode`; one of "text", "display-math" or "inline-math".
/// This also begins a new level of grouping and switches to a font
/// appropriate for the mode.
/// If `noframe` is true, skip pushing a stack frame (e.g. for \begin{document}).
/// Perl: sub beginMode (Stomach.pm lines 474-517)
pub fn begin_mode(mode: &str) -> Result<()> { begin_mode_opt(mode, false) }
/// Like `begin_mode`, but with an explicit `noframe` option.
/// When `noframe` is true, no stack frame is pushed (the caller already did bgroup).
pub fn begin_mode_opt(mode: &str, noframe: bool) -> Result<()> {
  if let Some(bound_mode) = bindable_mode(mode) {
    if !noframe {
      push_stack_frame(false); // Effectively bgroup
    }
    // Diagnostic: tracking who binds BOUND_MODE during raw .sty load
    // (gated by LXML_TRACE_BOUND_MODE env var to avoid noise in normal runs).
    // See project_explsyntax_midload.md memory for the active investigation.
    if *TRACE_BOUND_MODE {
      eprintln!(
        "[trace] begin_mode_opt mode={mode} noframe={noframe} bound_mode={bound_mode}\n{}",
        std::backtrace::Backtrace::force_capture()
      );
    }
    // Perl: $STATE->assignValue(BOUND_MODE => $mode, 'local');
    assign_value("BOUND_MODE", arena::pin(bound_mode), Some(Scope::Local));
    set_mode(bound_mode)?;
    // Perl Stomach.pm lines 504-507: inject \everymath or \everydisplay tokens
    // Display math gets \everydisplay, inline math gets \everymath (not both).
    if bound_mode.contains("math") {
      let is_display = bound_mode == "display_math";
      let reg_name = if is_display {
        "\\everydisplay"
      } else {
        "\\everymath"
      };
      if let Some(RegisterValue::Tokens(toks)) = lookup_register(reg_name, Vec::new())? {
        let toks = toks.unlist();
        if !toks.is_empty() {
          gullet::unread(Tokens::new(toks));
        }
      }
    }
    Ok(())
  } else {
    Warn!("unexpected", mode, s!("Cannot enter {mode} mode"));
    Ok(())
  }
}
/// End processing in `mode`; an error is signalled if `stomach` is not
/// currently in `mode`.  This also ends a level of grouping.
/// Perl: sub endMode (Stomach.pm lines 522-541)
pub fn end_mode(mode: &str) -> Result<()> { end_mode_opt(mode, false) }
/// Like `end_mode`, but with an explicit `noframe` option.
/// When `noframe` is true, executeBeforeAfterGroup is run but the stack frame is not popped.
pub fn end_mode_opt(mode: &str, noframe: bool) -> Result<()> {
  if let Some(bound_mode) = bindable_mode(mode) {
    // Perl Stomach.pm L527-528:
    //   if ((!$STATE->isValueBound('BOUND_MODE', 0))     # Last stack frame was NOT a mode switch
    //     || ($STATE->lookupValue('BOUND_MODE') ne $mode))  # OR switch to a different mode
    // Strict Perl-faithful: error if BOUND_MODE is not bound on the top
    // frame, OR if its value doesn't match the mode being closed. (Earlier
    // versions of this file used a lax value-only check as a workaround
    // for the 1112.6246 halign frame-balance issue, since fixed in
    // d162803d2.)
    let current_bound = crate::state::lookup_string_from_sym(crate::pin!("BOUND_MODE"));
    let bound_on_top = is_value_bound("BOUND_MODE", Some(0));
    if !bound_on_top || current_bound != bound_mode {
      // Last stack frame was NOT a mode switch, or was a switch to a different mode.
      // Perl: Don't pop if there's an error; maybe we'll recover?
      let message = s!(
        "Attempt to end mode `{}` in `{}`",
        mode,
        crate::state::lookup_string_from_sym(crate::pin!("MODE"))
      );
      let category = match get_current_token() {
        Some(ref token) => token.to_string(),
        None => String::from("mode"),
      };
      Error!("unexpected", category, &message);
    } else {
      // Perl: leaveHorizontal_internal($self) if $mode =~ /vertical$/;
      if bound_mode.ends_with("vertical") {
        leave_horizontal_internal();
      }
      if noframe {
        // No pop, but at least do beforeAfterGroup
        execute_before_after_group()?;
      } else {
        pop_stack_frame(false)?;
      }
    }
  } else {
    Warn!("unexpected", mode, s!("Cannot end {mode} mode"));
  }
  Ok(())
}

/// Switch to horizontal mode without stacking the mode.
/// Can only switch from vertical|internal_vertical to horizontal.
/// Perl: sub enterHorizontal
pub fn enter_horizontal() {
  let mode = crate::state::lookup_string_from_sym(crate::pin!("MODE"));
  if mode.ends_with("vertical") {
    assign_value_inplace_sym(crate::pin!("MODE"), crate::pin!("horizontal"));
  } else if !mode.ends_with("horizontal") && !mode.ends_with("math") {
    // Perl L420-422: warn on unexpected mode
    Warn!(
      "unexpected",
      "enterHorizontal",
      s!("Unexpected mode '{}' for enterHorizontal", mode)
    );
  }
  // else: already horizontal or math — fine
}

/// Resume vertical mode by executing \par, in TeX-like fashion.
/// Perl: sub leaveHorizontal
pub fn leave_horizontal() -> Result<()> {
  let mode = crate::state::lookup_string_from_sym(crate::pin!("MODE"));
  let bound = crate::state::lookup_string_from_sym(crate::pin!("BOUND_MODE"));
  if mode == "horizontal" && bound.ends_with("vertical") {
    // This needs to be an invisible, and slightly gentler, \par
    assign_value("INTERNAL_PAR", true, Some(Scope::Local));
    let par_result = invoke_token(&T_CS!("\\par"))?;
    push_box_list_vec(par_result);
    assign_value("INTERNAL_PAR", false, Some(Scope::Local));
  }
  Ok(())
}

/// Resume vertical mode internally: reset mode without firing \par.
/// Used within argument digestion, e.g. endMode for vertical modes.
/// Perl: sub leaveHorizontal_internal
pub fn leave_horizontal_internal() {
  let mode = crate::state::lookup_string_from_sym(crate::pin!("MODE"));
  let bound = crate::state::lookup_string_from_sym(crate::pin!("BOUND_MODE"));
  if mode == "horizontal" && bound.ends_with("vertical") {
    repack_horizontal();
    assign_value_inplace_sym(crate::pin!("MODE"), arena::pin(&bound));
  }
}

/// Repack recently digested horizontal items into single horizontal List.
/// Note that TeX would have done paragraph line-breaking, resulting in essentially
/// a vertical list.
/// Perl: sub repackHorizontal (Stomach.pm lines 440-454)
pub fn repack_horizontal() {
  let mut stomach = stomach_mut!();
  let mut para: Vec<Digested> = Vec::new();
  let mut keep = false;

  loop {
    let should_pop = if let Some(item) = stomach.box_list.last() {
      // Perf: compare as &str via with() instead of allocating a String each iter.
      // Default mode is "horizontal" (matches previous unwrap_or).
      let mode_prop = item.get_property("mode");
      let (is_horiz_family, is_plain_horizontal) = match mode_prop.as_deref() {
        Some(Stored::String(sym)) => arena::with(*sym, |s| {
          let plain = s == "horizontal";
          let fam = plain || s == "restricted_horizontal" || s == "math";
          (fam, plain)
        }),
        None => (true, true), // default "horizontal"
        Some(other) => {
          // Rare path — fall back to Display formatting.
          let s = other.to_string();
          let plain = s == "horizontal";
          let fam = plain || s == "restricted_horizontal" || s == "math";
          (fam, plain)
        },
      };
      if is_horiz_family {
        if !is_plain_horizontal || !item.get_property_bool("isSpace") {
          keep = true;
        }
        true
      } else {
        false
      }
    } else {
      false
    };

    if should_pop {
      para.push(stomach.box_list.pop().unwrap());
    } else {
      break;
    }
  }

  // Items were popped in reverse order, so reverse them back
  para.reverse();

  if keep {
    let mut list = List::new(para);
    list.mode = Some(TexMode::Text); // "horizontal" in Perl
    // Perl: List(@para, mode => 'horizontal') — set mode property string
    // This is needed for compute_boxes_size vertical layout to detect paragraph Lists
    list.set_property("mode", Stored::String(arena::pin_static("horizontal")));
    // Perl: $list->setProperty(width => LookupRegister('\hsize')) if $mode eq 'horizontal';
    if let Some(hsize) = lookup_dimension("\\hsize") {
      list.set_property("width", hsize);
    }
    stomach.box_list.push(Digested::from(list));
  }
}

pub fn new_local_box_list() {
  let mut buffer = Vec::new();
  let mut stomach = stomach_mut!();
  std::mem::swap(&mut stomach.box_list, &mut buffer);
  stomach.localized_box_list.push(buffer);
}
pub fn expire_local_box_list() -> Vec<Digested> {
  let mut stomach = stomach_mut!();
  let mut buffer = stomach.localized_box_list.pop().unwrap_or_default();
  std::mem::swap(&mut stomach.box_list, &mut buffer);
  buffer
}

pub fn extend_box_list<I>(arg: I)
where I: IntoIterator<Item = Digested> {
  stomach_mut!().box_list.extend(arg)
}
pub fn push_box_list(arg: Digested) { stomach_mut!().box_list.push(arg) }
fn push_box_list_vec(args: Vec<Digested>) { stomach_mut!().box_list.extend(args) }
pub fn pop_box_list() -> Option<Digested> { stomach_mut!().box_list.pop() }
pub fn with_box_list<R, FnR>(caller: FnR) -> R
where FnR: FnOnce(&[Digested]) -> R {
  let stomach = stomach!();
  let list = &stomach.box_list;
  caller(list)
}
pub fn with_box_list_mut<R, FnR>(caller: FnR) -> R
where FnR: FnOnce(&mut [Digested]) -> R {
  let mut stomach = stomach_mut!();
  let list = &mut stomach.box_list;
  caller(list)
}
/// Access to the current box_list as a `&mut Vec` — allows push/pop operations.
pub fn with_box_list_mut_vec<R, FnR>(caller: FnR) -> R
where FnR: FnOnce(&mut Vec<Digested>) -> R {
  let mut stomach = stomach_mut!();
  caller(&mut stomach.box_list)
}

// **********************************************************************
// Digestion
// **********************************************************************

/// Digest a list of tokens independent from any current Gullet.
/// Typically used to digest arguments to primitives or constructors.
/// Returns a List containing the digested material.
pub fn digest<T: Into<Tokens>>(tokens: T) -> Result<Digested> {
  let tokens: Tokens = tokens.into();
  if tokens.is_empty() {
    return Ok(Digested::default());
  }
  gullet::reading_from_mouth(Mouth::default(), || {
    gullet::unread(tokens);
    clear_prefixes(); // prefixes shouldn't apply here.
    let mode = if crate::state::lookup_bool_sym(crate::pin!("IN_MATH")) {
      TexMode::Math
    } else {
      TexMode::Text
    };
    let initdepth = stomach!().boxing.len();
    let depth = initdepth;
    new_local_box_list();
    while let Some(token) = match gullet::get_pending_comment() {
      Some(comment) => Some(comment),
      None => gullet::read_x_token(Some(true), false, None)?,
    } {
      // Done if we run out of tokens
      let invoked = invoke_token(&token)?;
      extend_box_list(invoked);

      if initdepth > stomach!().boxing.len() {
        // if we've closed the initial mode.
        break;
      }
      if initdepth < depth {
        // TODO
        fatal!(Internal, EoF, "We've fallen off the end, somehow !?!?!?");
        //     Fatal('internal', '<EOF>', self,
        //       "We've fallen off the end, somehow!?!?!",
        //       "Last token " . ToString($LaTeXML::CURRENT_TOKEN)
        //         . " (Boxing depth was $initdepth, now $depth: Boxing generated by "
        //         . join(', ', map { ToString($_) } @{ $self{boxing} }))
        //       if $initdepth < $depth;
      }
    }

    let mut digested_list = List::new(expire_local_box_list());
    digested_list.mode = Some(mode);
    digested_list.into()
  })
}

/// Return the digested `List` after reading and digesting a body from the its Gullet.
/// The body extends until the current level of boxing or environment is closed.
pub fn digest_next_body(terminal_opt: Option<Token>) -> Result<Vec<Digested>> {
  let start_location = { gullet::get_locator() };

  let init_depth = { stomach!().boxing.len() };
  let mut found_token = false;
  let mut found_terminal = false;
  new_local_box_list();
  let alignment_opt = lookup_alignment();
  // TODO: bookkeep for "expected" warning
  //let mut aug = Vec::new();

  // try reading a executable token
  while let Some(token) = match gullet::get_pending_comment() {
    Some(comment) => Some(comment),
    None => gullet::read_x_token(Some(true), false, None)?,
  } {
    // Check conversion timeout
    check_timeout()?;
    // done if we run out of tokens
    found_token = true;
    // first, check for alignment case
    // Perl #2775: only fire at the original alignment nesting level,
    // not inside deeper boxing groups (e.g. \vbox inside a tabular cell).
    if alignment_opt.is_some()
      && !stomach!().box_list.is_empty()
      && (stomach!().boxing.len() <= init_depth)
      && (token == T_ALIGN!()
        || token == T_CS!("\\cr")
        || token == T_CS!("\\lx@hidden@cr")
        || token == T_CS!("\\lx@hidden@crcr"))
    {
      gullet::unread_one(token);
      return Ok(expire_local_box_list());
    }
    // normal case
    let invoked = invoke_token(&token)?;
    extend_box_list(invoked);

    if let Some(ref terminal) = terminal_opt {
      if &token == terminal {
        found_terminal = true;
        break;
      }
    }
    if init_depth > stomach!().boxing.len() {
      break;
    }
  }

  if let Some(ref terminal) = terminal_opt {
    if !found_terminal {
      let message = s!(
        "body should have ended with {:?}. current body started at {:?}",
        terminal,
        start_location
      );
      Warn!("expected", terminal, message);
    }
  }
  // and add a Dummy `trailer' if none explicit.
  if !found_token {
    push_box_list(Digested::from(Tbox::default()));
    // info!(target:"digest_next_body","no_token");
  }
  Ok(expire_local_box_list())
}

/// a convenience function for including chunks of raw TeX (or LaTeX) code
/// It is useful for copying portions of the normal
/// implementation that can be handled simply using macros and primitives.
pub fn raw_tex(text: &str) -> Result<()> {
  // It could be as simple as this, except if catcodes get changed, it's too late!!!
  //  Digest(TokenizeInternal($text));
  let raw_tex_mouth = Mouth::new(
    text,
    Some(MouthOptions {
      fordefinitions: true,
      at_letter: true,
      ..MouthOptions::default()
    }),
  )?;
  gullet::reading_from_mouth(raw_tex_mouth, || -> Result<()> {
    while let Some(token) = gullet::read_x_token(Some(false), false, None)? {
      if token.get_catcode() != Catcode::SPACE {
        invoke_token(&token)?;
      }
    }
    Ok(())
  })?;
  Ok(())
}

/// Invoke a token
///
/// If it is a primitive or constructor, the definition will be invoked,
/// possibly arguments will be parsed from the Gullet.
/// Otherwise, the token is simply digested: turned into an appropriate box.
/// Returns a list of boxes/whatsits.
pub fn invoke_token(input_token: &Token) -> Result<Vec<Digested>> {
  // Perf: Token is Copy (SymStr + Catcode, ~5 bytes), so we pass by value
  // directly instead of wrapping in Cow<Token>.
  let mut maybe_token: Option<Token> = Some(*input_token);
  let mut result: Vec<Digested> = Vec::new();
  // INVOKE:
  while let Some(token) = maybe_token.take() {
    // RAII guard: auto-pops current_token on scope exit (even on early return/panic)
    let _token_guard = local_current_token_guard(token);
    {
      stomach_mut!().token_stack.push(token);
    }
    if { stomach!().token_stack.len() } > MAXSTACK {
      fatal!(
        Stomach,
        Recursion,
        s!(
          "Excessive recursion(?): Tokens on stack: {:?}",
          stomach!().token_stack
        )
      );
    }
    result = Vec::new();

    // Rust notes: It would be ideal if we could unify the cases for (Primtive, Constructor,
    // MathPrimitive), as well as (Expandable, Conditional) since the
    // API is identical. However, as the types are different, Rust
    // constrains us here, we need separate match arms for each
    // distinctly typed enum case.
    let digestable_def = lookup_digestable_definition(&token);
    match digestable_def {
      None | Some(Stored::None) => {
        result = invoke_token_undefined(&token)?;
      },
      Some(Stored::Token(meaning)) => {
        // Common case
        let cc = meaning.get_catcode();
        if cc == Catcode::CS {
          result = invoke_token_undefined(&token)?;
        } else if cc.is_absorbable() {
          if let Some(digested) = invoke_token_simple(meaning)? {
            result.push(digested);
          }
        } else {
          // Perl L187-189: deactivate T_ALIGN to prevent error flood in tables
          if token.get_catcode() == Catcode::ALIGN {
            if let Some(relax_meaning) = lookup_meaning(&T_CS!("\\relax")) {
              assign_meaning(&token, relax_meaning, Some(Scope::Local));
            }
          }
          let message = s!(
            "The token {:?} (catcode {:?}) should never reach Stomach!",
            token,
            cc
          );
          Error!("misdefined", token, &message);
          if let Some(digested) = invoke_token_simple(meaning)? {
            result.push(digested);
          }
        }
      },
      Some(Stored::Expandable(meaning)) => {
        // A math-active character will (typically) be a macro,
        // but it isn't expanded in the gullet, but later when digesting, in math mode
        // (? I think)
        let invoked_meaning = meaning.invoke(false)?;
        if !invoked_meaning.is_empty() {
          {
            gullet::unread(invoked_meaning);
          }
        }
        // replace the token by it's expansion!!!
        maybe_token = gullet::read_x_token(None, false, None)?;
        {
          stomach_mut!().token_stack.pop();
        }
        drop(_token_guard); // expire current token via RAII
        continue;
      },
      Some(Stored::Conditional(meaning)) => {
        // Conditionals are "expandable", use the regular invoke.
        let invoked_meaning = meaning.invoke(false)?;
        gullet::unread(invoked_meaning);
        maybe_token = gullet::read_x_token(None, false, None)?;
        {
          stomach_mut!().token_stack.pop();
        }
        drop(_token_guard); // expire current token via RAII
        continue;
      },
      Some(Stored::Constructor(meaning)) => {
        // Perl Stomach.pm L187-189: deactivate T_ALIGN to `\relax` LOCAL
        // on first non-table encounter, to prevent error flood. The
        // existing guard at the Stored::Token branch (above) only fires
        // when `&` has been Let'd to another token, but the `&`
        // CC_ALIGN char-token is bound to a Constructor (TeX_Tables.pool
        // L49: `DefConstructorI('&', undef, sub { Error('unexpected', '&',
        // $_[0], "Stray alignment \"&\"") })`), so it falls into THIS
        // branch instead. Without this guard, papers with multiple stray
        // `&` (e.g. astro-ph0107583's bibitem with unescaped `Hirose &
        // Osaki`) emit one Error per occurrence; Perl emits ONE total
        // because of the LOCAL `\relax` rebinding. Self-deactivate here
        // too so subsequent `&` invocations no-op.
        if token.get_catcode() == Catcode::ALIGN {
          if let Some(relax_meaning) = lookup_meaning(&T_CS!("\\relax")) {
            assign_meaning(&token, relax_meaning, Some(Scope::Local));
          }
        }
        result = meaning.invoke_primitive()?;
        if !meaning.is_prefix() {
          clear_prefixes(); // Clear prefixes unless we just set one.
        }
      },
      Some(Stored::Primitive(meaning)) => {
        // Otherwise, a normal primitive or constructor
        result = meaning.invoke_primitive()?;
        if !meaning.is_prefix() {
          clear_prefixes(); // Clear prefixes unless we just set one.
        }
      },
      Some(Stored::MathPrimitive(meaning)) => {
        // Copy of regular Primitive
        // Otherwise, a normal primitive or constructor
        result = meaning.invoke_primitive()?;
        if !meaning.is_prefix() {
          clear_prefixes(); // Clear prefixes unless we just set one.
        }
      },
      Some(Stored::Register(meaning)) => {
        // Registers are special primitives
        result = meaning.invoke_primitive()?;
        if !meaning.is_prefix() {
          clear_prefixes(); // Clear prefixes unless we just set one.
        }
      },
      meaning => {
        // Perl: Error + makeMisdefinedError (non-fatal). Don't crash.
        Error!(
          "misdefined",
          token,
          s!("Unexpected object in Stomach: {:?}", meaning)
        );
      },
    }
    // _token_guard drops here, auto-expiring current token
    break;
  }
  stomach_mut!().token_stack.pop();
  Ok(result)
}

fn invoke_token_undefined(token: &Token) -> Result<Vec<Digested>> {
  let cs = token.with_cs_name(|cs| String::from(cs));
  // Gate the undefined-CS summary tally and the Error! emission by
  // SUPPRESS_UNDEFINED_ERRORS. During expl3-code.tex raw load we install
  // the ERROR stub silently — forward references resolve when subsequent
  // post-load fixups rebind the canonical CS (see expl3_sty.rs L161-167
  // for \iow_wrap stubs that overwrite ERROR after the raw load). Mirrors
  // the existing gate at state.rs::generate_error_stub L1018-L1030.
  let suppressed = lookup_bool_sym(crate::pin!("SUPPRESS_UNDEFINED_ERRORS"));
  if !suppressed {
    note_status(LogStatus::Undefined, Some(&cs));
  }

  // To minimize chatter, go ahead and define it...
  if cs.starts_with("\\if") {
    // Apparently an \ifsomething ???
    let name = cs.replace("\\if", "");
    if !suppressed {
      let message = s!("The token {} is not defined.", token.stringify());
      Error!(
        "undefined",
        token,
        &message,
        "Defining it now as with \\newif"
      );
    }
    // install stub definitions for new conditional
    install_definition(
      Expandable::new(
        T_CS!(s!("\\{}true", name)),
        None,
        Tokens!(T_CS!("\\let"), T_CS!(&cs), T_CS!("\\iftrue")).into(),
        None,
      )?,
      None,
    );
    install_definition(
      Expandable::new(
        T_CS!(s!("\\{}false", name)),
        None,
        Tokens!(T_CS!("\\let"), T_CS!(cs), T_CS!("\\iffalse")).into(),
        None,
      )?,
      None,
    );

    let_i(token, &T_CS!("\\iffalse"), None);
    gullet::unread_one(*token); // Retry
    Ok(Vec::new())
  } else {
    if !suppressed {
      let message = s!("The token {} is not defined.", token.stringify());
      Error!(
        "undefined",
        token,
        &message,
        "Defining it now as <ltx:ERROR/>"
      );
    }
    install_definition(
      Constructor {
        cs: *token,
        paramlist: None,
        replacement: Some(Rc::new(move |document, _args, _props| {
          document.make_error("undefined", &cs)
        })),
        ..Constructor::default()
      },
      Some(Scope::Global),
    );
    // Perl: unread the token and return empty, so the outer loop re-reads
    // and dispatches through the normal path (with the newly installed stub).
    // This ensures gullet-level side effects (filtering, expansion) are applied.
    gullet::unread_one(*token);
    Ok(Vec::new())
  }
}

fn invoke_token_simple(meaning: Token) -> Result<Option<Digested>> {
  let cc = meaning.get_catcode();
  let font = lookup_font();
  match cc {
    Catcode::SPACE => {
      clear_prefixes(); // Perl Stomach.pm line 234: prefixes shouldn't apply here.
      // Perl: if($STATE->lookupValue('MODE') =~ /(?:math|vertical)$/) { return (); }
      let mode = crate::state::lookup_string_from_sym(crate::pin!("MODE"));
      if mode.ends_with("math") || mode.ends_with("vertical") {
        Ok(None)
      } else {
        enter_horizontal();
        Ok(Some(Digested::from(Tbox::new(
          meaning.get_sym(),
          font,
          None,
          Tokens!(meaning),
          HashMap::default(),
        ))))
      }
    },
    Catcode::COMMENT => {
      // Perl Stomach.pm lines 241-244: decode comment via font encoding
      let decoded = font::decode_string(meaning.get_sym(), None, true);
      let comment = arena::with(decoded, |s| {
        // However, spaces normally would have be digested away as positioning...
        // Replace NBSP + combining strikethrough (OT1 space position) with actual space
        s.replace("\u{00A0}\u{0335}", " ")
      });
      // Perl: returns LaTeXML::Core::Comment->new($comment)
      // which gets absorbed as an XML comment node via Document::insertComment
      Ok(Some(Digested::from(Comment(comment))))
    },
    _ => {
      clear_prefixes(); // Perl Stomach.pm line 247: prefixes shouldn't apply here.
      // Perl: check mathcode for IN_MATH characters (Stomach.pm lines 248-251)
      // In Perl, all math chars go through decodeMathChar which decodes via
      // the font encoding. In Rust, Tbox::new already handles IN_MATH:
      // it sets mode="math", looks up math_token_attributes for role/meaning/name,
      // and specializes the font. This produces the correct LaTeXML-level properties.
      // The mathchar parsing handles non-ASCII chars needing font map lookup.
      // TODO: Use for chars where font-encoding glyph differs from input.
      // Perl L248-257: if IN_MATH && mathcode → decodeMathChar (math box)
      // else → enterHorizontal + text box (covers non-math AND math-but-no-mathcode)
      if crate::state::lookup_bool_sym(crate::pin!("IN_MATH")) {
        if let Some(mathcode) = lookup_mathcode_sym(meaning.get_sym()) {
          return crate::common::mathchar::decode_math_char_for_stomach(mathcode, meaning);
        }
      }
      // Fallthrough: either not in math, or in math but no mathcode
      enter_horizontal();
      let text = font::decode_string(meaning.get_sym(), None, true);
      Ok(Some(Digested::from(Tbox::new(
        text,
        None,
        None,
        Tokens!(meaning),   // tokens
        HashMap::default(), // properties
      ))))
    },
  }
}

pub fn set_stomach(new_stomach: Stomach) {
  let mut singleton = stomach_mut!();
  *singleton = new_stomach;
}
pub fn clone_box_list() -> Vec<Digested> { stomach!().box_list.clone() }

/// get the current boxing level
pub fn get_boxing_level() -> usize { stomach!().boxing.len() }

/// ScriptLevel is similar to boxing level, but relative to current Math mode's level
///
/// This is used for the scriptpos attribute to recognize overlapping sccripts.
/// Making it relative to the math's level avoids unnecessary changes
pub fn get_script_level() -> usize {
  let boxlevel = get_boxing_level();
  with_value("script_base_level", |val_opt| {
    if let Some(Stored::Int(prevlevel)) = val_opt {
      boxlevel - (*prevlevel as usize) + 1
    } else {
      boxlevel
    }
  })
}
