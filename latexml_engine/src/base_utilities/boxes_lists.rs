//! Box contents predigestion and list simplification; revert helpers.

use super::*;

/// Stomach-level counterpart to `read_box_contents`.
///
/// Perl: readBoxContents calls $stomach->beginMode($mode), then reads/digests tokens
/// Predigest box contents by invoking T_BEGIN, which triggers
/// the stomach's bgroup/egroup mechanism to properly handle the box body.
///
/// Perl's `List()` simplification (List.pm line 41-44):
/// When a vertical-mode List has exactly one non-empty item and that item's mode
/// is also vertical, return the item directly instead of wrapping in a List.
/// This enables `is_vbox` property propagation for nested \vbox/\vtop.
pub fn predigest_box_contents(tokens: ArgWrap) -> Result<Option<Digested>> {
  // Default: use the CURRENT mode (legacy behavior, e.g. for `{Body}` arguments
  // that aren't \vbox/\vtop/\hbox-flavored). For VBoxContents / HBoxContents,
  // call predigest_box_contents_in_mode("internal_vertical" / "restricted_horizontal")
  // explicitly so we mirror Perl's `readBoxContents(..., $mode)` (TeX_Box.pool.ltxml L133).
  let mode = lookup_string_from_sym(pin!("MODE"));
  predigest_box_contents_in_mode(tokens, &mode)
}

/// Perl-faithful body-digest for VBoxContents / HBoxContents parameters.
/// Mirrors `readBoxContents($gullet, $everybox, $mode)` exactly:
/// pushes a fresh frame for `$mode`, digests tokens until matching T_END,
/// pops the frame.
///
/// The mode-aware variant matters when `\vtop` is invoked inside a `p{}`
/// column body (alignment is in horizontal mode, but the `\vtop`'s VBox
/// content must be digested in internal_vertical mode so `\@startpbox`'s
/// `\vtop\bgroup ...` doesn't try to close groups in the wrong mode).
/// Witness: 2210.13325 `\begin{tabular}{|p{1cm}|...}` — pre-fix Rust
/// inherited the surrounding horizontal mode and emitted `\vtop`
/// errors; Perl uses 'internal_vertical' here regardless of where the
/// `\vtop` was invoked.
pub fn predigest_box_contents_in_mode(tokens: ArgWrap, mode: &str) -> Result<Option<Digested>> {
  predigest_list_in_mode(tokens, mode, false)
}

/// The material of a `\vadjust`, read live from the input after its `{` as the box loop reads a box's (tex.web §1099:
/// `scan_left_brace`, then `main_control` builds the group's internal vertical list), so catcode changes inside it
/// act on what follows — a short verbatim `|\ifx|` prints `\ifx` — and it is built where it is read (a box register,
/// a macro, a counter as they are there). The mode is `inline_internal_vertical`, which does not end the paragraph
/// around it; a paragraph the material begins ends at the group's end (§1100 `insert_group`: `end_graf`), as the
/// primitive `\lx@normal@par` (KNOWN_PERL_ERRORS #441; repro boxes-groups/vadjust_material_is_read_live).
pub fn predigest_insert_group_contents() -> Result<Option<Digested>> {
  predigest_list_in_mode(ArgWrap::None, "inline_internal_vertical", true)
}

fn predigest_list_in_mode(
  _tokens: ArgWrap,
  mode: &str,
  end_graf: bool,
) -> Result<Option<Digested>> {
  // Perl: readBoxContents calls beginMode($mode) / endMode($mode) around the body reading.
  // This creates a scoped frame where enterHorizontal can change MODE inplace.
  // When endMode is called, leaveHorizontal_internal detects MODE='horizontal' with
  // BOUND_MODE ending in 'vertical', triggers repackHorizontal, then pops the frame.
  //
  // NOTE: read_box_contents already consumed the opening { or \bgroup (§403 `scan_left_brace`, `is_left_brace`).
  // invoke_token(T_BEGIN) pushes a synthetic group frame. The matching } or \egroup
  // in the content will pop this frame, since \egroup is \let to T_END and
  // invoke_token handles it via the standard group-closing mechanism.
  // Perl: $stomach->beginMode($mode) — push a new frame for this box content scope
  // VERTICAL box modes (`\vbox`/`\vtop`/p{}-cell VBoxContents): faithful port of
  // Perl `readBoxContents` (TeX_Box.pool.ltxml L139-160). ONE mode frame as the
  // group; a loop that STOPS at the matching `}` WITHOUT processing it, so
  // `end_mode` runs `leave_horizontal_internal` (repack — capturing the *inner*
  // `\hsize`) BEFORE `pop_stack_frame` restores it. The `invoke_token(&T_BEGIN!())`
  // path used for other modes lets the body's `}` egroup-restore `\hsize` before
  // the post-hoc repack, so a `\vbox{\hsize=W …para…}` would wrap at the OUTER
  // `\hsize` (the Cluster G `\narrow`-hang; rotated p{}-cell mis-sizing —
  // SYNC_STATUS 1610.00974 step-3). hbox/math don't paragraph-wrap → the simpler
  // `invoke_token(T_BEGIN)` path below. (A tabular inside a `\vbox`/`\vtop` is
  // correctly SKIPPED by the repack because `\@@tabular`/`\halign` mark the result
  // box `internal_vertical`.)
  // RESTRICTED HORIZONTAL (`\\hbox`/`\\mbox`/`\\makebox`… contents) takes the
  // same one-frame loop (batch 54n). tex.web §1083 `begin_box` pushes the
  // nest and the save level TOGETHER for `\\hbox\\bgroup` and §1100 `package`
  // pops both; Perl's one frame is that single push. The former two-frame
  // shortcut (`begin_mode` + the `{` primitive's own `bgroup`) made the box
  // depend on an `\\egroup` to close its synthetic group, and ulem's
  // open-here/close-there word boxes (examdesign.cls:186-200 `\\UL@start` =
  // `\\setbox\\UL@box\\hbox\\bgroup…\\bgroup`, `\\UL@stop` = `\\egroup\\egroup`) around a
  // `\\makebox` whose argument digests in isolation met the wrong frame
  // ("\\egroup Attempt to close a group that switched to mode
  // restricted_horizontal"; examdesign examplea/b/c, Perl shares it). Guard:
  // `perfect_kernel_batch54::hbox_reader_is_one_frame`.
  if mode.ends_with("vertical") || mode == "restricted_horizontal" {
    begin_mode(mode)?;
    // A vertical box's own list (`VBoxContents`), where `\unvbox` puts a box's lines back (tex_box.rs `vlist_of`).
    if mode == "internal_vertical" {
      assign_frame_value("lx@vbox@list", true);
    }
    // The box's OWN frame (tex.web §1068 `handle_right_brace` dispatches on
    // `cur_group`, the group itself): Perl keys the terminal on the save-stack
    // DEPTH (`$level >= getFrameDepth`, TeX_Box.pool.ltxml:172), which a
    // crossing — a `\begingroup` opened inside the box after a frame below
    // it was popped, pgf's node text over our bound `\pgfsys@*` seam — can
    // satisfy with a `\begingroup` frame on top, so `end_mode` then failed
    // ("\hbox Attempt to end mode restricted_horizontal", modernposter; the
    // msc/dsptricks cascades). A `}` that meets another frame is invoked
    // instead: `egroup` reports and drops it (tex.web §1069
    // `extra_right_brace`) and the box goes on to its own `}`.
    let own_frame = current_frame_id();
    new_local_box_list(); // Perl: local @LaTeXML::LIST = ()
    // The box's contents are digested outside any number scan in progress
    // (`gullet::NumberScan::suspend`, as `stomach::digest` does): this loop is
    // a stomach entry of its own.
    let _outside_scan = NumberScan::suspend();
    loop {
      let next = match get_pending_comment() {
        Some(comment) => Some(comment),
        None => read_x_token(Some(true), false, None)?,
      };
      let Some(token) = next else { break };
      // The box's own `}` (its frame on top) ends the loop and is closed by
      // `end_mode`, NOT egroup; any other `}` is processed.
      if is_right_brace(&token) && current_frame_id() == own_frame {
        break;
      }
      check_timeout()?; // runaway guard (mirrors the canonical group-digest loop)
      extend_box_list(invoke_token(&token)?);
    }
    // tex.web §679 `append_to_vlist`: the interline glue between a vertical box's lines follows the
    // `\baselineskip`, `\lineskip` and `\lineskiplimit` in force inside it (`\offinterlineskip` in the box), read
    // before its group ends; Perl's `List` records the `\baselineskip` after it (List.pm:52-53). Its sizing and
    // `\vsplit` (tex_inserts.rs) stack the lines with them (OXIDIZED_DESIGN_DIVERGENCES #421; short-math-guide's
    // `symlist` columns split as its PDF).
    let interline: Vec<(&str, Dimension)> = if mode.ends_with("vertical") {
      [
        ("baseline", "\\baselineskip"),
        ("lineskip", "\\lineskip"),
        ("lineskiplimit", "\\lineskiplimit"),
      ]
      .into_iter()
      .filter_map(|(key, register)| {
        lookup_register_quiet(register).map(|value| (key, Dimension::new(value.value_of())))
      })
      .collect()
    } else {
      Vec::new()
    };
    // §1100 `insert_group`: `end_graf` before the group's list is packaged. The primitive `\lx@normal@par` also when
    // the list ends in vertical mode (where `end_graf` does nothing): it closes what the material opened, so a box
    // set in it (`\vadjust{\box0}`) is no part of the paragraph after.
    if end_graf {
      extend_box_list(invoke_token(&T_CS!("\\lx@normal@par"))?);
    }
    // Perl: $stomach->endMode($mode) — leave_horizontal_internal (repack with the
    // still-in-scope inner `\hsize`) THEN pop_stack_frame (restores `\hsize`).
    end_mode(mode)?;
    let mode_tex = if lookup_bool_sym(pin!("IN_MATH")) {
      TexMode::Math
    } else {
      TexMode::Text
    };
    let mut digested_list = List::new(expire_local_box_list());
    digested_list.mode = Some(mode_tex);
    for (key, value) in interline {
      digested_list
        .properties
        .insert(key, Stored::Dimension(value));
    }
    let mut item: Digested = digested_list.into();
    // Perl: List(@LaTeXML::LIST, mode => $mode)
    item.set_property("mode", Stored::String(pin(mode)));
    // Perl's single-item `List()` simplification is a vertical-mode rule.
    return Ok(Some(if mode.ends_with("vertical") {
      simplify_vertical_list(item)
    } else {
      item
    }));
  }
  if mode.ends_with("vertical") || mode.ends_with("horizontal") {
    begin_mode(mode)?;
  }
  let mut contents = invoke_token(&T_BEGIN!())?;
  if contents.is_empty() {
    // Perl: $stomach->endMode($mode)
    if mode.ends_with("vertical") || mode.ends_with("horizontal") {
      end_mode(mode)?;
    }
    Ok(None)
  } else {
    let mut item = contents.remove(0);
    // Perl's endMode triggers leaveHorizontal_internal → repackHorizontal
    // when enterHorizontal changed MODE to 'horizontal' inplace within this frame.
    // Check the condition BEFORE endMode pops the frame.
    let post_mode = lookup_string_from_sym(pin!("MODE"));
    let bound_mode = lookup_string_from_sym(pin!("BOUND_MODE"));
    if post_mode == "horizontal"
      && bound_mode.ends_with("vertical")
      && has_only_simple_horizontal_content(&item)
    {
      repack_horizontal_in_list(&mut item);
      // Restore MODE like leave_horizontal_internal does
      assign_value_inplace_sym(pin!("MODE"), pin(&bound_mode));
    }
    // Perl: $stomach->endMode($mode) — pop the frame
    if mode.ends_with("vertical") || mode.ends_with("horizontal") {
      end_mode(mode)?;
    }
    // Set the mode property on the resulting item (matching Perl's List(@boxes, mode => $mode))
    if !mode.is_empty() {
      item.set_property("mode", Stored::String(pin(mode)));
    }
    // Apply Perl's List() single-item simplification for vertical modes.
    // In Perl, List(@boxes, mode=>'internal_vertical') returns the single box
    // directly when @boxes has 1 element and the box's mode is also vertical.
    // This is critical for nested \vbox/\vtop: the inner box's `is_vbox` property
    // must be visible to the outer box's constructor.
    Ok(Some(simplify_vertical_list(item)))
  }
}

/// Check if a List contains only simple horizontal content (TBoxes,
/// Comments, or sub-Lists whose mode is horizontal/restricted_horizontal/
/// math). This guards against repack being triggered for cases like
/// `\vtop{\begin{tabular}...}` where the tabular processing leaks
/// MODE='horizontal' but the content (an Alignment/Whatsit) should NOT
/// be paragraph-wrapped.
///
/// Inline brace-groups `{...}` inside running text produce sub-Lists
/// (one per group) whose mode is `restricted_horizontal`. Without
/// accepting those, e.g. `\vbox{\small\bfseries hello {,} world}` would
/// fail the repack gate and be measured as 3 separate vertical lines.
/// Witness: aistats2026.sty's `\def\And{\unskip{,}\enspace}` in the
/// `\@runningauthor` body — every author separator emits a `{,}` group
/// that fragments the vbox into many short rows, making `\ht\autrun`
/// far exceed 10pt and triggering the class's `\PackageError{Document}
/// {Running heading author exceeds size limitations}` (driver paper:
/// arXiv:2602.11863).
fn has_only_simple_horizontal_content(item: &Digested) -> bool {
  if let DigestedData::List(l) = item.data() {
    let list = l.borrow();
    let non_empty: Vec<_> = list
      .boxes
      .iter()
      .filter(|b| !b.get_property_bool("isEmpty"))
      .collect();
    non_empty.iter().all(|b| match b.data() {
      DigestedData::TBox(_) | DigestedData::Comment(_) => true,
      DigestedData::List(sub) => {
        // Inline brace-groups `{...}` in horizontal context digest to
        // a sub-List with no `mode` property (implicit hbox). Accept
        // those plus sub-Lists explicitly tagged as horizontal-flavour.
        // Reject `mode=vertical|internal_vertical` (structural
        // sub-vboxes) and any other tagged mode.
        let sub_mode = sub
          .borrow()
          .properties
          .get("mode")
          .map(|v| v.to_string())
          .unwrap_or_default();
        matches!(
          sub_mode.as_str(),
          "" | "horizontal" | "restricted_horizontal" | "math"
        )
      },
      _ => false,
    })
  } else {
    false
  }
}

/// Replicates Perl's repackHorizontal() within a List's children.
///
/// Perl (Stomach.pm lines 442-456): In readBoxContents, after digesting box content
/// in vertical mode, repackHorizontal groups consecutive horizontal-mode items
/// from @LaTeXML::LIST into a single List(@para, mode => 'horizontal') with
/// width set to \hsize. This enables compute_boxes_size to do paragraph wrapping.
///
/// Without this, \vbox{hop} measures each character individually (width=5.55pt)
/// instead of wrapping as a paragraph at \hsize (width=469.75pt).
fn repack_horizontal_in_list(item: &mut Digested) {
  if let DigestedData::List(l) = item.data() {
    let mut list = l.borrow_mut();
    let children = std::mem::take(&mut list.boxes);
    let mut result: Vec<Digested> = Vec::new();
    let mut para: Vec<Digested> = Vec::new();
    let mut keep = false;

    for child in children {
      let child_mode = child
        .get_property("mode")
        .map(|v| v.to_string())
        .unwrap_or_else(|| "horizontal".to_string());
      // Empty-string mode means "implicit hbox" — produced by inline
      // brace-groups `{...}` in horizontal context. Treat as horizontal
      // so the surrounding running text doesn't get fragmented.
      let effective_mode = if child_mode.is_empty() {
        "horizontal"
      } else {
        child_mode.as_str()
      };
      if effective_mode == "horizontal"
        || effective_mode == "restricted_horizontal"
        || effective_mode == "math"
      {
        // Perl: $keep = 1 if ($mode ne 'horizontal') || !$item->getProperty('isSpace');
        if effective_mode != "horizontal" || !child.get_property_bool("isSpace") {
          keep = true;
        }
        para.push(child);
      } else {
        // Flush accumulated horizontal items as a horizontal List
        if keep {
          let horiz_list = make_horizontal_list(std::mem::take(&mut para));
          result.push(Digested::from(horiz_list));
        } else {
          result.extend(std::mem::take(&mut para));
        }
        keep = false;
        result.push(child);
      }
    }
    // Flush remaining horizontal items
    if keep {
      let horiz_list = make_horizontal_list(para);
      result.push(Digested::from(horiz_list));
    } else {
      result.extend(para);
    }
    list.boxes = result;
  }
}

/// Create a horizontal List with mode='horizontal' and width=\hsize.
/// Perl: push(@LaTeXML::LIST, List(@para, mode => 'horizontal')) if $keep;
/// Perl: $list->setProperty(width => LookupRegister('\hsize')) if $mode eq 'horizontal';
fn make_horizontal_list(para: Vec<Digested>) -> List {
  let mut list = List::new(para);
  list.mode = Some(TexMode::Text);
  list.set_property("mode", Stored::String(pin!("horizontal")));
  if let Some(hsize) = lookup_dimension("\\hsize") {
    list.set_property("width", Stored::Dimension(hsize));
  }
  list
}

/// Perl's List() single-item simplification for vertical modes.
///
/// Perl (List.pm line 41-44):
/// ```perl
/// if ((scalar(@boxes) == 1)
///     && (!$mode || ($mode !~ /vertical$/)
///         || (($boxes[0]->getProperty('mode')||'') =~ /vertical$/))) {
///     return $boxes[0]; }   # Simplify!
/// ```
///
/// When a List in vertical mode contains a single non-empty item whose mode is also
/// vertical, return that item directly. This is critical for nested \vbox/\vtop:
/// the inner \vbox Whatsit has `is_vbox = true` set by after_digest, and the outer
/// \vtop constructor needs to see this property to skip double insertBlock wrapping.
fn simplify_vertical_list(item: Digested) -> Digested {
  // Only simplify if the item is a List
  let is_vertical_list = match item.data() {
    DigestedData::List(l) => {
      let list = l.borrow();
      // Check if the List's mode property indicates vertical
      list
        .properties
        .get("mode")
        .map(|m| m.ends_with_text("vertical"))
        .unwrap_or(false)
    },
    _ => false,
  };
  if !is_vertical_list {
    return item;
  }

  // Extract the List's boxes, filtering out empty marker items (isEmpty property)
  let non_empty: Vec<Digested> = match item.data() {
    DigestedData::List(l) => {
      let list = l.borrow();
      list
        .boxes
        .iter()
        .filter(|b| !b.get_property_bool("isEmpty"))
        .cloned()
        .collect()
    },
    _ => unreachable!(),
  };

  // Perl simplification: single non-empty item whose mode is also vertical
  if non_empty.len() == 1 {
    let single = &non_empty[0];
    let child_is_vertical = match single.data() {
      DigestedData::List(l) => l
        .borrow()
        .properties
        .get("mode")
        .map(|m| m.ends_with_text("vertical"))
        .unwrap_or(false),
      DigestedData::Whatsit(w) => {
        // Check whatsit's mode property (set by DefConstructor mode => "internal_vertical")
        w.borrow()
          .get_property("mode")
          .map(|m| m.ends_with_text("vertical"))
          .unwrap_or(false)
      },
      _ => false,
    };
    if child_is_vertical {
      return non_empty.into_iter().next().unwrap();
    }
  }
  item
}

/// Perl: revertSpec($whatsit, $keyword)
/// If whatsit has property $keyword, return Explode($keyword) ++ Revert($value)
pub fn revert_spec(whatsit: &Whatsit, keyword: &str) -> Vec<Token> {
  match whatsit.get_property(keyword) {
    Some(value) => {
      // Explode the keyword + value strings into T_OTHER tokens. `pin_char`
      // uses a stack-buffer encode_utf8 and skips the per-char
      // `c.to_string()` heap alloc the previous version did.
      let mut tokens: Vec<Token> = keyword.chars().map(|c| CharToken!(c)).collect();
      let val_str = value.to_attribute();
      tokens.extend(val_str.chars().map(|c| CharToken!(c)));
      tokens
    },
    _ => Vec::new(),
  }
}

pub fn p_revert<T>(arg: T) -> Result<Tokens>
where T: Sized + Object {
  set_dual_branch("presentation");
  let result = arg.revert();
  expire_dual_branch();
  result
}

pub fn c_revert<T>(arg: T) -> Result<Tokens>
where T: Sized + Object {
  set_dual_branch("content");
  let result = arg.revert();
  expire_dual_branch();
  result
}
