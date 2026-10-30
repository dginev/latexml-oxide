//! Small token/text utilities and the front matter debug and ink helpers (`unitlength_sp`, `already_reported`,
//! `digested_to_text`, `ink_text`, `shown_arguments`).

use super::*;

/// `\unitlength` in sp, exact (65536, 1pt, when it is undefined): the picture
/// unit every `{picture}` and pict2e coordinate is multiplied by (Perl
/// `picScale`, latex_constructs.pool.ltxml:4896-4921). A skip that calc's
/// `\setlength` put there is its natural width (the register coerces it).
pub fn unitlength_sp() -> Result<i64> {
  Ok(match lookup_register("\\unitlength", Vec::new())? {
    Some(value) => Dimension::from(value).value_of(),
    None => 65536,
  })
}

/// Has this diagnostic already been reported? Marks `key` reported either way.
///
/// Port of Perl's report-once diagnostic idiom, e.g. `\selectfont`
/// (latex_constructs.pool.ltxml L5210-5211):
///
/// ```perl
/// elsif (!LookupValue("reported_unrecognized_font_family_$family")) {
///   AssignValue("reported_unrecognized_font_family_$family", 1, 'global');
///   Info('unexpected', $family, $_[0], "Unrecognized font family '$family'."); }
/// ```
///
/// So a caller reads `if !already_reported(&key) { Info!(…) }`, mirroring the
/// Perl `elsif`. An unrecognized font family/series/shape — or a missing font
/// encoding — is announced once per document, not once per occurrence: the
/// font switch that provokes it typically sits inside a macro that a document
/// expands in every table cell (witness 2503.04421, whose `\XSolidBrush`
/// column produced 28 identical `Info:unexpected:ding` lines).
///
/// Global scope, as in Perl: the suppression has to outlive the group the
/// diagnostic fired in, or a grouped `{\dingfamily …}` reports again each time.
pub fn already_reported(key: &str) -> bool {
  if lookup_bool(key) {
    return true;
  }
  assign_value(key, true, Some(Scope::Global));
  false
}

/// Join token groups with a conjunction token between them.
///
/// Port of Perl `JoinTokens($conjunction, @things)` (Base_Utility.pool.ltxml L142-148).
#[allow(dead_code)]
pub fn join_tokens(conjunction: &Tokens, things: Vec<Tokens>) -> Tokens {
  if things.is_empty() {
    return Tokens::new(vec![]);
  }
  let mut result: Vec<Token> = Vec::new();
  let mut first = true;
  for thing in things {
    if !first {
      result.extend_from_slice(conjunction.unlist_ref());
    }
    result.extend(thing.unlist());
    first = false;
  }
  Tokens::new(result)
}

//======================================================================
// Front Matter machinery (Perl Base_Utility.pool.ltxml, PR #2767)
//======================================================================

/// Perl: showFrontmatter($entry) — debug formatter for a frontmatter entry.
pub(super) fn show_frontmatter(entry: &TagData) -> String {
  let mut attrs: Vec<String> = entry.attr.iter().map(|(k, v)| s!("{k}={v}")).collect();
  attrs.sort();
  let content: String = entry
    .content
    .iter()
    .map(|c| match c {
      TagContent::PlaceKeeper => "place_keeper".to_string(),
      TagContent::Box(d) => d.to_string(),
      TagContent::Entry(e) => show_frontmatter(e),
    })
    .collect();
  s!("{}  [{}] {}", entry.tag, attrs.join(","), content)
}

/// The tags `\lx@make@tags{type}` collects into an `ltx:tags` (Perl Base_Utility.pool.ltxml:975-983), one per declared
/// role (`type_tag_formatter`), in role order, each with whether it is printed where it is built: the role-less tag is
/// the target's own label (`\item[Espa~na]`, a bibliography entry's `\@biblabel`), the others are the forms only a
/// reference prints (refnum, typerefnum, autoref, the cleveref forms). A caller that typesets those only where a
/// reference prints them (sect11.rs `\lx@bibitem`, the `\b@key` a `\cite` prints) takes them apart here.
pub fn tag_forms(ttype: &Tokens) -> Result<Vec<(bool, Tokens)>> {
  // Pull the (role -> formatter) pairs out of HashStored via with_value
  // so we don't clone the whole hashmap envelope; the per-role tokens
  // are Copy and the per-role String arm just dereferences a SymStr.
  let role_formatters: Vec<(String, Option<Token>)> =
    with_value("type_tag_formatter", |v| match v {
      Some(Stored::HashStored(formatters)) => {
        let keys_sym: Vec<_> = formatters.keys().copied().collect();
        let mut sorted_keys: Vec<String> = with_many(&keys_sym, |keys| {
          keys.into_iter().map(str::to_owned).collect()
        });
        sorted_keys.sort();
        sorted_keys
          .into_iter()
          .map(|role| {
            let ft = match formatters.get(&role) {
              Some(Stored::Token(t)) => Some(*t),
              Some(Stored::String(sym)) => Some(Token!(sym * sym, Catcode::CS)),
              _ => None,
            };
            (role, ft)
          })
          .collect()
      },
      _ => Vec::new(),
    });
  let mut forms = Vec::new();
  for (role, formatter_opt) in role_formatters {
    let Some(formatter_token) = formatter_opt else {
      continue;
    };
    // A tag only a reference prints (`type_tag_deferred`) holds its diagnostics (`\lx@tag@intags@held`).
    let constructor = if with_mapping("type_tag_deferred", &role, |held| held.is_some()) {
      "\\lx@tag@intags@held"
    } else {
      "\\lx@tag@intags"
    };
    let invoked_tag = Invocation!(T_CS!(constructor), vec![
      Tokens!(T_OTHER!(role.as_str())),
      build_invocation(formatter_token, vec![Some(ttype.clone())])?
    ]);
    // A tag a reference prints (refnum, typerefnum, autoref, the cleveref forms) is built where its target is (an
    // eqnarray row's in its math group, where a document may rebind `~`: vdm.sty:135-138, 202-205
    // `\everymath{\let~\hook}`, a `\vbox{\ialign…}` that re-stepped the equation, an endless recursion on 1601.02132)
    // and printed in text. Its `~` is the kernel's no-break space, LaTeX's `\nobreakspace`, which babel's `~` also is
    // except in a language that makes it a shorthand; such a shorthand, or a document's own text `~`, no longer reaches
    // these tags (OXIDIZED_DESIGN_DIVERGENCES #98 addendum). The role-less tag is the target's own label, printed where
    // it is built (`\item[Espa~na]`), and keeps the document's `~`. Guard
    // `perfect_kernel_batch63::autoref_tag_tilde_is_the_kernel_space`.
    // A reference form re-reads the label's source, which the role-less tag typesets (`\lx@reread@copy`): an
    // `\item[$\hat\mathbf{e}$]`'s typerefnum, a `\tag{$\hat\mathbf{t}$}`'s refnum give no second
    // OXIDIZED_DESIGN_DIVERGENCES #721 Warning (guard `perfect_kernel_batch71a::undelimited_argument_warning_once_per_occurrence`).
    if role.is_empty() {
      forms.push((true, invoked_tag));
    } else {
      let mut wrapped = vec![
        T_CS!("\\begingroup"),
        T_CS!("\\lx@reread@copy"),
        T_CS!("\\let"),
        T_ACTIVE!('~'),
        T_CS!("\\lx@tag@texttilde"),
      ];
      wrapped.extend(invoked_tag.unlist());
      wrapped.push(T_CS!("\\endgroup"));
      forms.push((false, Tokens::new(wrapped)));
    }
  }
  Ok(forms)
}

/// The authors the frontmatter has digested so far (`num_ltx:creator{author}`): the last one digests with the label
/// `author:N` for this N, by which a contact digested after it can name it or the ones before it (amsppt's per-block
/// addresses, amsppt_sty.rs `\lx@amsppt@paired`).
pub fn digested_author_count() -> i64 { lookup_mapping_int("num_ltx:creator", "author") }

/// Perl: LookupMapping returning a number (0 when absent).
pub(super) fn lookup_mapping_int(map: &str, key: &str) -> i64 {
  match lookup_mapping(map, key) {
    Some(Stored::Int(n)) => n,
    Some(Stored::Number(n)) => n.0,
    _ => 0,
  }
}

// (the count of empty held tags a target has yet to settle is the job's deferred diagnostics', reset with them)
pub use latexml_core::util::logger::{
  empty_held_tags_pending, note_empty_held_tag, take_empty_held_tag,
};

/// The `\lx@tag@intags` whatsit a held tag digested to (`\lx@tag@intags@held`): the digested box itself, or the
/// first such whatsit of its list.
pub(super) fn held_tag_whatsit(tag: &Digested) -> Option<Digested> {
  match tag.data() {
    DigestedData::Whatsit(w) if w.borrow().get_definition().get_cs_name() == "\\lx@tag@intags" => {
      Some(tag.clone())
    },
    DigestedData::List(l) => l.borrow().boxes.iter().find_map(held_tag_whatsit),
    _ => None,
  }
}

/// Walk a `Digested` and concatenate its text content (for attribute use,
/// matching Perl's `setAttribute(..., DigestText(...))` semantics). Tbox
/// children contribute their text; nested Lists recurse; `\hskip`-style
/// Whatsits (which are side-effect-only constructors with no text content)
/// fall back to `dimension_to_spaces(width)` instead of reverting to the
/// macro name. All other Whatsits use their normal `get_string` path.
/// (Rust-only helper, previously in latex_constructs; moved here since
/// digest_front_matter needs it for the creator `before` separators.)
pub fn digested_to_text(d: &Digested) -> Result<String> {
  let mut out = String::new();
  match d.data() {
    DigestedData::TBox(b) => out.push_str(&b.borrow().get_string()?),
    DigestedData::List(l) => {
      for child in l.borrow().boxes.iter() {
        out.push_str(&digested_to_text(child)?);
      }
    },
    DigestedData::Whatsit(w) => {
      let w = w.borrow();
      match w.get_property("width").as_deref() {
        Some(Stored::Dimension(width)) => {
          // The whatsit's OWN font — see tex_glue::dimension_to_spaces.
          let font = w.get_font()?;
          out.push_str(&crate::tex_glue::dimension_to_spaces(
            *width,
            font.as_deref(),
          ));
        },
        _ => {
          out.push_str(&w.get_string()?);
        },
      }
    },
    _ => out.push_str(&d.to_string()),
  }
  Ok(out)
}

/// Whether digested material typesets something a reader sees: a character that is not a space, an image (a whatsit
/// carrying a `graphic` or image `candidates`), or a cross-reference or citation, whose text the post-processor
/// fills in (`fills_generated_text`), anywhere in it — through lists, the whatsit arguments it shows
/// (`shown_arguments`, as [`ink_text`] reads them) and bodies, and alignment cells. Paragraph and spacing commands,
/// rules and empty boxes are not content, nor is an argument LaTeX only tests: the position letter of
/// `\begin{tabular}[t]`, `\parbox[t]`, `{minipage}[t]`, `\makebox[w][l]` or `\shortstack[l]` (a `Position` parameter,
/// base_parameter_types.rs) — counted, the deproc title code's `\begin{tabular}[t]{l}\bf\rule{\z@}{24pt}…` with an
/// emptied `\@author` (iclr*_conference.sty:83-93, arxiv.sty, colm, corl, nips) kept its `\@maketitle` deposit, an empty
/// strut table before the first section (HF14; 94 of 6,000 sampled papers, 2610.20409, 2310.10873).
/// A whatsit's own string is its reversion (Whatsit.pm `toString`), so a digested `\noindent\par`
/// stringifies as source; this walks the boxes instead. Used to keep a speculative typesetting (a
/// class's `\maketitle` replay, a one-shot page overlay) only when it shows something.
pub fn typesets_content(d: &Digested) -> bool {
  match d.data() {
    DigestedData::TBox(b) => b
      .borrow()
      .get_string()
      .is_ok_and(|s| s.chars().any(|c| !c.is_whitespace())),
    DigestedData::List(l) => l.borrow().boxes.iter().any(typesets_content),
    DigestedData::Whatsit(w) => {
      let w = w.borrow();
      // (an image: graphics.sty's `\@includegraphics` records its `graphic`, graphicx's `\@includegraphicx` its
      // `path`, both the image's `candidates`; its file name is no shown argument)
      w.get_property("graphic").is_some()
        || w.get_property("candidates").is_some()
        || fills_generated_text(&w)
        // A tabular or `\halign` keeps its rows in its `alignment` property (tex_tables.rs): a class's `\maketitle`
        // that is only a table typesets it (simplecv's header, brandeis-problemset's title table; dropped since 59p).
        || matches!(w.get_property("alignment").as_deref(), Some(Stored::Digested(a)) if typesets_content(a))
        || w
          .get_args()
          .iter()
          .zip(shown_arguments(&w))
          .any(|(arg, shown)| shown && arg.as_ref().is_some_and(typesets_content))
        || w
          .get_body()
          .ok()
          .flatten()
          .is_some_and(|body| typesets_content(&body))
    },
    DigestedData::Alignment(a) => a.borrow().rows().iter().any(|row| {
      row
        .get_columns()
        .iter()
        .any(|cell| cell.boxes.as_ref().is_some_and(typesets_content))
    }),
    _ => false,
  }
}

/// Whether digested material prints nothing: every item in it is one known to leave no ink ([`first_ink`] finds
/// none). The inverse of [`typesets_content`], failing safe: what is not known to print nothing is ink. A frontmatter
/// note is dropped as one with no ink only on this test, the XML's (frontmatter_api.rs) and the frontmatter observer's
/// (the rehearsal's note event, `Event::inkless`) alike (OXIDIZED_DESIGN_DIVERGENCES #916): a false "no ink" loses the
/// note's content, a false "ink" keeps a blank note.
pub fn leaves_no_ink(d: &Digested) -> bool { first_ink(d).is_none() }

/// The first item of digested material not known to print nothing ([`leaves_no_ink`]), described: its command, or its
/// text. Known to print nothing are
/// - a box of white space (an interword space, a tie's no-break space) and a box Perl marks `isSpace` (glue, a kern,
///   `\quad`, a phantom, `\indent`/`\noindent`: Box.pm:33-38 and the pool's `isSpace` properties), but a leader;
/// - a rule with no width or no height and depth (a strut);
/// - a line or page break, `\par`, a penalty, `\leavevmode` (`INKLESS_COMMANDS`);
/// - a transparent wrapper whose arguments (but options and a position keyword) and body print nothing: a plain box or
///   a formula (`INKLESS_WRAPPERS`: `\makebox[1em]{}`, `\raisebox{1ex}{}`, `$ $`), or a constructor whose whole
///   construction is a template printing only its arguments or body (`prints_only_arguments`: `\mbox{}`, a font
///   switch, `$\mathbb{}$`).
///
/// Anything else is ink: a constructor's own text, with or without a font (revsymb's `\openone`, `"1"` in the
/// blackboard font; `\dots`, `\LaTeX`, `$\vdots$`), an element of its own (`\textsuperscript{}`'s `ltx:sup`, a framed
/// box `\fbox{}`), a closure-built construction (`\hypertarget{a}{}`) or one with a construction hook (`\emph{}`), an
/// empty template standing for what it does not show (the observer rehearsal's `\lx@fm@obs@url`, its footnote mark), a
/// leader (`\hrulefill`), an image, a cross-reference, an alignment, an undefined command's error marker — and any
/// item, blank or not, whose font carries a background colour: `\colorbox{c}{…}` is `\hbox{\pagecolor{c}…}`
/// (color.sty, color_sty.rs), whose `\pagecolor` leaves an empty box in the coloured font, and pdflatex prints the
/// coloured block (`\colorbox{red}{}`, `\colorbox{yellow}{\hspace{2em}}`, inside `\mbox{}` too).
pub fn first_ink(d: &Digested) -> Option<String> {
  if d
    .get_font()
    .ok()
    .flatten()
    .is_some_and(|font| font.get_background().is_some())
  {
    return Some(format!("a background colour: {d}"));
  }
  if is_spacing_item(d) || is_blank_item(d) {
    return None;
  }
  match d.data() {
    DigestedData::TBox(b) => {
      let b = b.borrow();
      match b.get_string() {
        Ok(s) => Some(s.to_string()),
        Err(_) => Some(b.to_string()),
      }
    },
    DigestedData::List(l) => l.borrow().boxes.iter().find_map(first_ink),
    DigestedData::Whatsit(w) => {
      let w = w.borrow();
      let definition = w.get_definition();
      let cs = definition.get_cs_name();
      let truthy = |key: &str| {
        w.get_property(key)
          .is_some_and(|v| !matches!(*v, Stored::Bool(false) | Stored::Int(0)))
      };
      let dimension = |key: &str| match w.get_property(key).as_deref() {
        Some(Stored::Dimension(d)) => d.value_of(),
        _ => 0,
      };
      if INKLESS_COMMANDS.contains(&cs.as_ref()) {
        return None;
      }
      // (a rule shows with a width and a height or depth: LaTeX's `\rule` its `width`, `height` and `depth`
      // (sect12.rs); a primitive rule its `cached_*` sizes, a `\vrule` of no stated height or depth the running
      // one, of width 0 `invisible` (tex_box.rs, OXIDIZED_DESIGN #176))
      let shows = match cs.as_ref() {
        "\\rule" => Some(dimension("width") != 0 && dimension("height") + dimension("depth") != 0),
        "\\vrule" => Some(
          !truthy("invisible")
            && (w.get_property("rheight").is_none()
              || w.get_property("rdepth").is_none()
              || dimension("cached_height") + dimension("cached_depth") != 0),
        ),
        "\\hrule" => Some(dimension("cached_height") + dimension("cached_depth") != 0),
        _ => None,
      };
      if let Some(shows) = shows {
        return shows.then(|| cs.to_string());
      }
      let body = w.get_body().ok().flatten();
      let wrapper = INKLESS_WRAPPERS.contains(&cs.as_ref())
        || definition
          .replacement_template()
          .is_some_and(prints_only_arguments);
      if !wrapper {
        return Some(cs.to_string());
      }
      // (every argument but its options and a position keyword: a verbatim one too, which a wrapper may print though
      // `shown_arguments` does not count it — failing safe, as for jhep's `\href`, 0704.3719)
      let kinds = argument_kinds(&w);
      w.get_args()
        .iter()
        .enumerate()
        .filter(|(i, _)| kinds.as_ref().is_none_or(|k| k.get(*i) != Some(&0)))
        .find_map(|(_, arg)| arg.as_ref().and_then(first_ink))
        .or_else(|| body.as_ref().and_then(first_ink))
    },
    DigestedData::Alignment(_) => Some("an alignment".to_string()),
    DigestedData::Postponed(t) => Some(t.to_string()),
    DigestedData::KeyVals(_) | DigestedData::RegisterValue(_) | DigestedData::Comment(_) => None,
  }
}

/// Whether digested material is spacing, printing no ink but a gap between what it stands between ([`first_ink`]; the
/// frontmatter label reader's separator, `queue_digest::label_ink`): a box of white space (an interword space, a tie's
/// no-break space, math glue `\,` in a formula) and a box or whatsit Perl marks `isSpace` (Box.pm:33-38 and the pool's
/// `isSpace` properties: `\!`'s zero-width-space box, glue, a kern, `\quad`, `\hspace`, a phantom), but a leader
/// (`\hrulefill`, ink though Perl marks some `isSpace`) and what prints no gap ([`is_blank_item`]).
pub fn is_spacing_item(d: &Digested) -> bool {
  if is_blank_item(d) {
    return false;
  }
  let truthy = is_space_marked(d);
  match d.data() {
    DigestedData::TBox(b) => {
      truthy
        || b
          .borrow()
          .get_string()
          .is_ok_and(|s| s.chars().all(char::is_whitespace))
    },
    DigestedData::Whatsit(w) => {
      truthy && !LEADERS.contains(&w.borrow().get_definition().get_cs_name().as_ref())
    },
    _ => false,
  }
}

/// Whether digested material prints nothing and no gap — no ink ([`first_ink`]) and no spacing ([`is_spacing_item`]):
/// a box of no text (`\-`'s), a kern Perl marks `isSpace` of zero width (the italic correction
/// `\/`, `width => 0em`, TeX_Fonts.pool.ltxml:162, a kern tex.web §1113; `\kern0pt`), and glue of no natural width,
/// stretch or shrink (`\hskip0pt`, `\hspace{0pt}`: `is_zero_glue`). `\textsuperscript{1\-2}`, `1\/2` and
/// `1\hskip0pt 2` print "12", the label 12 (review rev92b_r3 P1 and rev92b_r4 S1: read as spacing, the labels 1 and
/// 2, its `12` line on no name).
pub fn is_blank_item(d: &Digested) -> bool {
  if is_zero_glue(d) {
    return true;
  }
  match d.data() {
    DigestedData::TBox(b)
      if b.borrow().get_string().is_ok_and(|s| s.is_empty()) && !is_space_marked(d) =>
    {
      true
    },
    DigestedData::TBox(_) | DigestedData::Whatsit(_) => {
      is_space_marked(d)
        && d
          .get_property("isKern")
          .is_some_and(|v| !matches!(*v, Stored::Bool(false) | Stored::Int(0)))
        && matches!(d.get_property("width").as_deref(), Some(Stored::Dimension(w)) if w.value_of() == 0)
    },
    _ => false,
  }
}

/// Whether digested material is glue whose natural width, stretch and shrink are all zero ([`is_blank_item`]): an
/// `\hskip` whatsit by its glue argument (tex_glue.rs), an `\hspace` box by the glue it states (`glue`, sect12.rs).
fn is_zero_glue(d: &Digested) -> bool {
  let zero = |g: &Glue| g.skip == 0 && g.plus.unwrap_or(0) == 0 && g.minus.unwrap_or(0) == 0;
  match d.data() {
    DigestedData::Whatsit(w) => {
      let w = w.borrow();
      w.get_definition().get_cs_name().trim_end() == "\\hskip"
        && matches!(
          w.get_args().first().and_then(Option::as_ref).map(Digested::data),
          Some(DigestedData::RegisterValue(RegisterValue::Glue(g))) if zero(g)
        )
    },
    DigestedData::TBox(_) => {
      matches!(d.get_property("glue").as_deref(), Some(Stored::Glue(g)) if zero(g))
    },
    _ => false,
  }
}

/// Whether Perl marks digested material `isSpace` (Box.pm:33-38).
fn is_space_marked(d: &Digested) -> bool {
  d.get_property("isSpace")
    .is_some_and(|v| !matches!(*v, Stored::Bool(false) | Stored::Int(0)))
}

/// The text a pgf/TikZ picture shows (pgfsys-latexml.def's `\lxSVG@insertpicture`, an `<ltx:picture>` and its
/// `<svg:svg>`), `None` for any other whatsit: the boxes its text nodes insert — `\pgfsys@hbox`'s box (`thebox`), the
/// one way pgf typesets text in a picture (a `\node{…}`, `\pgftext`, `\pgfsys@hboxsynced`; pgfsys.code.tex) —, in their
/// order. Everything else a pgf picture holds draws: an SVG path's data and style (`\lxSVG@drawpath@unclipped{M 5.53 0
/// C …}{fill:none}`), colours, transforms, scopes. A frontmatter mark's printed label reads only the text
/// (`queue_digest::label_ink`, the observer's `events::script_ink`): 2605.02769's TikZ ORCID logo in
/// `\inst{\ref{leiden}\orcidicon{…}}` holds none (its path data, parted at its spaces, was the marks M, C, 0, L, Z on
/// every name — review rev92b_r1 B1), a circled letter `\tikz…\node[circle,draw](c){a};` prints its "a" (review
/// rev92b_r2 B2: a picture read as no text lost it, its line a continuation of the one before). LaTeX's `picture`
/// environment is read as any material is (`\put(0,0){a}`'s text is an argument).
pub fn picture_text(w: &Whatsit) -> Option<Vec<Digested>> {
  fn text_nodes(d: &Digested, nodes: &mut Vec<Digested>) {
    match d.data() {
      DigestedData::List(l) => {
        for child in l.borrow().boxes.iter() {
          text_nodes(child, nodes);
        }
      },
      DigestedData::Whatsit(w) => whatsit_text_nodes(&w.borrow(), nodes),
      DigestedData::Alignment(a) => {
        for row in a.borrow().rows() {
          for cell in row.get_columns() {
            if let Some(boxes) = cell.boxes.as_ref() {
              text_nodes(boxes, nodes);
            }
          }
        }
      },
      _ => {},
    }
  }
  fn whatsit_text_nodes(w: &Whatsit, nodes: &mut Vec<Digested>) {
    if w.get_definition().get_cs_name().trim_end() == "\\pgfsys@hbox" {
      if let Some(Stored::Digested(text)) = w.get_property("thebox").as_deref() {
        nodes.push(text.clone());
      }
      return;
    }
    if let Some(Stored::Digested(a)) = w.get_property("alignment").as_deref() {
      text_nodes(a, nodes);
    }
    for arg in w.get_args().iter().flatten() {
      text_nodes(arg, nodes);
    }
    if let Ok(Some(body)) = w.get_body() {
      text_nodes(&body, nodes);
    }
  }
  (w.get_definition().get_cs_name().trim_end() == "\\lxSVG@insertpicture").then(|| {
    let mut nodes = Vec::new();
    for arg in w.get_args().iter().flatten() {
      text_nodes(arg, &mut nodes);
    }
    nodes
  })
}

/// Whether a frontmatter mark's label reader keeps the character `c` of printed text, `before` the last it kept
/// (`queue_digest::label_ink`, the observer's `events::script_ink`): a format character (Unicode Cf: the zero-width
/// space, the joiners, the byte-order mark) prints nothing and is dropped — `\up{\!1,2}`'s zero-width space was an
/// empty mark on each name, 2209.14855 (review rev92b_r1 B1) —; a combining mark (M) is part of the character before
/// it, kept — `\={x}` prints "x̄", an accent with no precomposed form (`\=`, `\~`, `\v`, `\u`, `\H`, `\b`, `\d` over an
/// `x`), a label of its own, not `x` (review rev92b_r2 B1: the mark dropped, `x̄` matched an `x` line) — and dropped
/// only with no base: first, or after white space or a label separator (`,`). Any other character is kept.
pub fn keeps_label_char(c: char, before: Option<char>) -> bool {
  if is_format_char(c) {
    return false;
  }
  !is_combining_mark(c) || before.is_some_and(|b| b != ',' && !b.is_whitespace())
}

/// The combining characters a math accent whatsit prints on its base, from the glyph its constructor carries
/// (`accent`, set for an `OVERACCENT` or `UNDERACCENT` operator, latexml_core `transfer_common_constructor_options`):
/// `\bar`'s U+00AF over is U+0304, `\hat`'s `^` U+0302, `\vec`'s → U+20D7, a combining glyph itself (`\mathring`'s
/// U+030A), `\underline`'s U+00AF under U+0332; `None` for anything else, an accent whose glyph has no combining form
/// (`\overbrace`'s ⏞) among them. The frontmatter label readers (`queue_digest::label_ink`, the observer's
/// `events::script_ink`) write it after the base: `$^{\bar{\dag}}$` is the label `†̄`, no `\dag`'s `†`, and
/// `$^{\hat a}$` `â`, no `a` (review rev92b_r4 S2; `\={x}` digests to `x̄` already).
pub fn accent_combining_form(w: &Whatsit) -> Option<String> {
  let accent = match w.get_property("accent").as_deref() {
    Some(accent @ Stored::String(_)) => accent.to_string(),
    _ => return None,
  };
  let under = w
    .get_property("operator_role")
    .is_some_and(|role| role.to_string() == "UNDERACCENT");
  let form = |c: char| -> Option<char> {
    if is_combining_mark(c) {
      return Some(c);
    }
    Some(match (c, under) {
      ('^' | '\u{02C6}', false) => '\u{0302}',
      ('~' | '\u{02DC}', false) => '\u{0303}',
      ('\u{00AF}', false) => '\u{0304}',
      ('\u{02D8}', false) => '\u{0306}',
      ('\u{02D9}', false) => '\u{0307}',
      ('\u{00A8}', false) => '\u{0308}',
      ('\u{02C7}', false) => '\u{030C}',
      ('\u{00B4}', false) => '\u{0301}',
      ('`', false) => '\u{0300}',
      ('\u{2192}', false) => '\u{20D7}',
      ('\u{2190}', false) => '\u{20D6}',
      ('\u{2194}', false) => '\u{20E1}',
      ('\u{00AF}', true) => '\u{0332}',
      ('~' | '\u{02DC}', true) => '\u{0330}',
      ('\u{2192}', true) => '\u{20EF}',
      ('\u{2190}', true) => '\u{20EE}',
      _ => return None,
    })
  };
  accent.chars().map(form).collect()
}

/// Whether a character is a format character (Unicode Cf), printing nothing ([`keeps_label_char`]).
pub fn is_format_char(c: char) -> bool {
  static FORMAT: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^\p{Cf}$").unwrap());
  FORMAT.is_match(c.encode_utf8(&mut [0; 4]))
}

/// Whether a character is a combining mark (Unicode M), printed on the character before it ([`keeps_label_char`]).
pub fn is_combining_mark(c: char) -> bool {
  static MARK: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"^\p{M}$").unwrap());
  MARK.is_match(c.encode_utf8(&mut [0; 4]))
}

/// Whether a constructor's replacement `template` ([`Definition::replacement_template`]) prints nothing of its own,
/// only its arguments and body ([`first_ink`]): every piece of it is one of these
/// - an argument (`#1`) or the body (`#body`), absorbed;
/// - white space;
/// - an `ltx:text` or `ltx:emph` element (opened or closed) whose attributes only set the font or how the element
///   closes ([`FONT_ONLY_ATTRIBUTES`]: `<ltx:text _noautoclose='1'>#1</ltx:text>`, `\emph`'s `_force_font`);
/// - a conditional both of whose branches are such (`\emph`'s `?#isMath(…)(…)`);
///
/// and one of them absorbs an argument or the body. Anything else is something of its own: literal text (revsymb's `\openone`,
/// `"1"`), another element (`<ltx:sup>#1</ltx:sup>`, a rule, a framed or coloured `ltx:text`), an empty element, a
/// processing instruction, an attribute set on the node around it, a property or function value, a template that
/// does not parse — and an empty template, which stands for what it does not show.
fn prints_only_arguments(template: &str) -> bool {
  use latexml_core::binding::def::replacement::{
    AttrPair, ReplacementOp, Value, parse_replacement,
  };
  fn font_only(attrs: &[AttrPair]) -> bool {
    attrs.iter().all(|pair| match pair {
      AttrPair::KeyValue { key, .. } => FONT_ONLY_ATTRIBUTES.contains(&key.as_str()),
      AttrPair::Conditional { then_attrs, else_attrs, .. } => {
        font_only(then_attrs) && font_only(else_attrs)
      },
    })
  }
  fn transparent(ops: &[ReplacementOp]) -> bool {
    ops.iter().all(|op| match op {
      ReplacementOp::Text { text } => text.chars().all(char::is_whitespace),
      ReplacementOp::AbsorbValue { value: Value::Arg(_) } => true,
      ReplacementOp::AbsorbValue { value: Value::Prop(name) } => name == "body",
      ReplacementOp::OpenElement {
        qname,
        attrs,
        self_closing: false,
        ..
      } => matches!(qname.as_str(), "ltx:text" | "ltx:emph") && font_only(attrs),
      ReplacementOp::CloseElement { qname } => matches!(qname.as_str(), "ltx:text" | "ltx:emph"),
      ReplacementOp::Conditional { then_ops, else_ops, .. } => {
        transparent(then_ops) && transparent(else_ops)
      },
      _ => false,
    })
  }
  // (and absorbs one: a template printing nothing at all stands for what it does not show — the frontmatter
  // observer's rehearsal's `\lx@fm@obs@url`, `\lx@fm@obs@fnmark`, `""` in place of a URL or a footnote mark —, a doubt)
  fn absorbs(ops: &[ReplacementOp]) -> bool {
    ops.iter().any(|op| match op {
      ReplacementOp::AbsorbValue { .. } => true,
      ReplacementOp::Conditional { then_ops, else_ops, .. } => {
        absorbs(then_ops) || absorbs(else_ops)
      },
      _ => false,
    })
  }
  parse_replacement(template).is_ok_and(|ops| transparent(&ops) && absorbs(&ops))
}

/// The attributes a transparent wrapper's element may carry ([`prints_only_arguments`]): its font, and how it closes.
const FONT_ONLY_ATTRIBUTES: &[&str] = &["font", "_font", "_force_font", "_noautoclose"];

/// Commands that print nothing, whatever their arguments ([`first_ink`]): `\par`, breaks and penalties.
const INKLESS_COMMANDS: &[&str] = &[
  "\\par",
  "\\penalty",
  "\\\\",
  "\\lx@newline",
  "\\lx@parboxnewline",
  "\\@normalcr",
  "\\newline",
  "\\linebreak",
  "\\nolinebreak",
  "\\pagebreak",
  "\\nopagebreak",
  "\\leavevmode",
  "\\lx@normal@par",
  // (the frontmatter observer's rehearsal's `\\`, `\newline`, `\linebreak`, `\par` and `\label`, observer/rehearsal.rs)
  "\\lx@fm@obs@break",
  "\\lx@fm@obs@label",
];

/// Leaders, ink though Perl marks some `isSpace` ([`first_ink`]).
const LEADERS: &[&str] = &[
  "\\hrulefill",
  "\\dotfill",
  "\\leaders",
  "\\cleaders",
  "\\xleaders",
];

/// Wrappers that print only their arguments and body ([`first_ink`]) though their construction is no font-only
/// template ([`prints_only_arguments`] reads `\mbox`, `\text`, `\smash` and the font switches): an unframed box,
/// built by a closure (`\hbox`, `\vbox`, `\vtop`) or placed by its template's size or offset
/// (`\@makebox`, `\raisebox`) — a box's colour is its content's font's, which [`first_ink`] reads —, and a formula.
const INKLESS_WRAPPERS: &[&str] = &[
  "\\@makebox",
  "\\hbox",
  "\\vbox",
  "\\vtop",
  "\\raisebox",
  "\\lx@begin@inline@math",
  "\\lx@begin@display@math",
];

/// Whether the whatsit `w` is a cross-reference or a citation, which prints text the post-processor generates (the
/// label's number, the cited work's name and year) though its arguments are keys, never shown ([`shown_arguments`]):
/// a reference carries the `label` its `labelref` names (`\ref`, `\pageref`, `\eqref`, `\autoref`, `\nameref`,
/// cleveref's `\lx@cref`, ntheorem's `\thref`), a citation its `bibrefs` (`\@@bibref`, which every `\cite` form reaches).
/// `\label` records its `label` too, but prints nothing: an alignment may skip it (`alignmentSkippable`). A class's
/// `\@maketitle` printing only `\ref{s1}` keeps its deposit, as pdflatex prints "1" (77o review probe `m1_depref`).
fn fills_generated_text(w: &Whatsit) -> bool {
  w.get_property("bibrefs").is_some()
    || (w.get_property("label").is_some() && w.get_property("alignmentSkippable").is_none())
}

/// The text digested material typesets, as [`typesets_content`] walks it — characters of its boxes, through lists,
/// whatsit arguments and bodies, alignment rows and cells; never a whatsit's reversion (its source), and only the
/// arguments it shows (`shown_arguments`): an image's file name and options are not text on the page
/// (`\includegraphics{\@logo}`), nor is a link's target (`\href{\@url}{Home}` prints "Home"). What a speculative
/// title-page run printed (sect05.rs `speculative_title_page_run`; review probe `logo`).
pub fn ink_text(d: &Digested, out: &mut String) {
  match d.data() {
    DigestedData::TBox(b) => {
      if let Ok(s) = b.borrow().get_string() {
        out.push_str(&s);
      }
    },
    DigestedData::List(l) => {
      for child in l.borrow().boxes.iter() {
        ink_text(child, out);
      }
    },
    DigestedData::Whatsit(w) => {
      let w = w.borrow();
      if let Some(Stored::Digested(a)) = w.get_property("alignment").as_deref() {
        ink_text(a, out);
      }
      for (arg, shown) in w.get_args().iter().zip(shown_arguments(&w)) {
        if shown && let Some(arg) = arg {
          ink_text(arg, out);
        }
      }
      if let Ok(Some(body)) = w.get_body() {
        ink_text(&body, out);
      }
    },
    DigestedData::Alignment(a) => {
      for row in a.borrow().rows() {
        for cell in row.get_columns() {
          if let Some(boxes) = cell.boxes.as_ref() {
            ink_text(boxes, out);
          }
        }
      }
    },
    _ => {},
  }
}

/// The constructors that print a verbatim argument as it is — a URL, an e-mail address, a DOI —, each with which of its
/// arguments it shows (by position), for [`shown_arguments`]: llncs.cls's `\url` (llncs_cls.rs), hyperref's
/// `\nolinkurl`, aas's `\@@email`, the OmniBus `\lx@doi`, natbib's `\harvardurl`. A constructor's replacement is
/// compiled, so what it puts in its element cannot be read off the whatsit; a verbatim argument is otherwise a key
/// (`\ref`, `\pageref`, `\eqref`, `\cite`, `\nameref`, `\autoref`, `\label`), a link target or a file name, which the
/// page does not show. url.sty's and hyperref's `\url`/`\path` and the bindings' `\email`s are macros: what they print
/// reaches a constructor of its own, whose text argument is shown (`\lx@url@url@nolink`'s, `\lx@hyper@url@`'s `#5`), as
/// aas's `\doi{…}`'s is. A constructor printing one of two verbatim arguments names it: `("\\href", &[false, true])` would show the
/// text, not the URL (jhep's two-verbatim `\href`, a binding class's, which never meets the raw-class stores).
const VERBATIM_PRINTERS: &[(&str, &[bool])] = &[
  ("\\url", &[true]),
  ("\\nolinkurl", &[true]),
  ("\\@@email", &[true]),
  ("\\lx@doi", &[true]),
  ("\\harvardurl", &[true]),
];

/// Which arguments of the whatsit `w` it shows as text, for [`ink_text`], by their declared parameter types: none of an
/// image (a whatsit carrying a `graphic` or image `candidates`: its arguments are the file and its options); else a
/// verbatim argument (`Semiverbatim`, `Verbatim`, `HyperVerbatim`: a file name, a URL, a label) only when the whatsit
/// is one that prints it ([`VERBATIM_PRINTERS`]: `\url{…}`, `\nolinkurl{…}`, the arguments its entry names) — never a
/// reference's or a citation's key (`\ref{\@x}` prints the label's number, review probe `ctx3`), and
/// `\href{<url>}{<text>}` and `\hypertarget{<name>}{<text>}` print the text —, and never keyvals (options) nor a
/// position keyword (`Position`, `OptionalPosition`: the `[t]` of `\begin{tabular}[t]`, base_parameter_types.rs). Every
/// argument when its parameters cannot be matched to them. Also what `\textcircled` circles (sect13.rs
/// `typeset_text`).
pub(crate) fn shown_arguments(w: &Whatsit) -> Vec<bool> {
  let args = w.get_args();
  // (graphics.sty's `\@includegraphics` records its `graphic`, graphicx's `\@includegraphicx` its `path`; both the
  // image's `candidates`)
  if w.get_property("graphic").is_some() || w.get_property("candidates").is_some() {
    return vec![false; args.len()];
  }
  let Some(kinds) = argument_kinds(w) else {
    return vec![true; args.len()];
  };
  let definition = w.get_definition();
  let cs = definition.get_cs();
  let shown: &[bool] = VERBATIM_PRINTERS
    .iter()
    .find(|(name, _)| *cs == T_CS!(*name))
    .map_or(&[], |(_, shown)| shown);
  kinds
    .iter()
    .enumerate()
    .map(|(i, kind)| *kind == 2 || (*kind == 1 && shown.get(i).copied().unwrap_or(false)))
    .collect()
}

/// The kind of each argument of the whatsit `w` by its parameter's type — 0: keyvals (options) or a position keyword
/// (`Position`, `OptionalPosition`), 1: verbatim (`Semiverbatim`, `Verbatim`, `HyperVerbatim`), 2: any other —; none
/// when its parameters cannot be matched to its arguments ([`shown_arguments`], [`first_ink`]).
fn argument_kinds(w: &Whatsit) -> Option<Vec<u8>> {
  let types: Vec<SymStr> = w
    .get_definition()
    .get_parameters()
    .map(|params| {
      params
        .get_parameters()
        .into_iter()
        .filter(|p| !p.novalue)
        .map(|p| p.name)
        .collect()
    })
    .unwrap_or_default();
  (types.len() == w.get_args().len()).then(|| {
    types
      .iter()
      .map(|t| {
        with(*t, |name| {
          if name.contains("KeyVals") || name.ends_with("Position") {
            0
          } else if name.contains("erbatim") {
            1
          } else {
            2
          }
        })
      })
      .collect()
  })
}

#[cfg(test)]
mod tests {
  use super::prints_only_arguments;

  /// A constructor's template prints only its arguments when it absorbs one and adds nothing but white space and
  /// font-only `ltx:text`/`ltx:emph` (OD #916): `\mbox`, `\emph`, a math alphabet; revsymb's `\openone` (`"1"`), an
  /// element of its own, a frame, a background, an empty template and a property value are something of their own.
  #[test]
  fn template_prints_only_its_arguments() {
    for template in [
      "#1",
      "<ltx:text _noautoclose='1'>#1</ltx:text>",
      "?#isMath(<ltx:text _force_font='1'>#1)(<ltx:emph _force_font='1'>#1)",
      "#body",
    ] {
      assert!(prints_only_arguments(template), "{template}");
    }
    for template in [
      "1",
      "",
      "<ltx:sup>#1</ltx:sup>",
      "<ltx:text framed='rectangle'>#1</ltx:text>",
      "<ltx:text backgroundcolor='#color'>#1</ltx:text>",
      "<ltx:text width='#width' _noautoclose='1'>#3</ltx:text>",
      "<ltx:rule width='#1'/>",
      "#1x",
      "#name",
    ] {
      assert!(!prints_only_arguments(template), "{template}");
    }
  }
}
