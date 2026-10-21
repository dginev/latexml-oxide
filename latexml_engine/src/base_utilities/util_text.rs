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
  let definition = w.get_definition();
  let types: Vec<SymStr> = definition
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
  if types.len() != args.len() {
    return vec![true; args.len()];
  }
  // (0: keyvals or a position keyword, 1: verbatim, 2: shown text)
  let kinds: Vec<u8> = types
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
    .collect();
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
