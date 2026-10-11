//! setspace.sty — line spacing
//! Perl: setspace.sty.ltxml
use latexml_core::common::error::emit_error;

use crate::prelude::*;

LoadDefinitions!({
  // setspace.sty:304-363: `\setstretch` sets `\baselinestretch` and re-runs the current size, whose
  // switch sets `\baselineskip` = leading × `\baselinestretch` (58c); the spacing commands are its
  // 10pt stretches (the bindings' sizes are size10.clo's). `\singlespacing`'s `\vskip\baselineskip`
  // correction is omitted. Perl's are no-ops (setspace.sty.ltxml). Witnesses 2605.18633, 2605.01923.
  DefMacro!("\\setstretch{}", "\\def\\baselinestretch{#1}\\@currsize");
  DefMacro!("\\setspace@singlespace", "1");
  DefMacro!("\\SetSinglespace{}", "\\def\\setspace@singlespace{#1}");
  DefMacro!("\\singlespacing", "\\setstretch{\\setspace@singlespace}");
  // setspace.sty:337-363: the stretch for the class's `\@ptsize` (10/11/12pt), 10pt's without one.
  DefMacro!(
    "\\onehalfspacing",
    "\\setstretch{1.25}\\ifx\\@ptsize\\@undefined\\else\\ifcase\\@ptsize\\relax\\setstretch{1.25}\\or\\setstretch{1.213}\\or\\setstretch{1.241}\\fi\\fi"
  );
  DefMacro!(
    "\\doublespacing",
    "\\setstretch{1.667}\\ifx\\@ptsize\\@undefined\\else\\ifcase\\@ptsize\\relax\\setstretch{1.667}\\or\\setstretch{1.618}\\or\\setstretch{1.655}\\fi\\fi"
  );
  // setspace.sty:418-422: a float's text is single-spaced (the kernel's `\@floatboxreset` hook).
  DefMacro!(
    "\\lx@floatbox@reset",
    "\\reset@font\\def\\baselinestretch{\\setspace@singlespace}\\normalsize"
  );
  // setspace.sty:442-480: a footnote's text is single-spaced too — the leading alone (the
  // kernel's `\size@update` arithmetic), the note's font untouched (notes are unsized). Appended, as
  // other packages hook a note's text too (biblatex's `blx@footnote`).
  RawTeX!(
    r"\g@addto@macro\lx@note@reset{\baselineskip\f@baselineskip\relax\baselineskip\setspace@singlespace\baselineskip}"
  );
  // setspace.sty:294-295: the package options run the spacing command at the package's end.
  DeclareOption!("onehalfspacing", "\\AtEndOfPackage{\\onehalfspacing}");
  DeclareOption!("doublespacing", "\\AtEndOfPackage{\\doublespacing}");
  ProcessOptions!();
  def_macro_noop("\\setdisplayskipstretch{}")?;
  def_macro_noop("\\restore@spacing")?;

  // {spacing}{}: the body-wrapping variant. Keep BOUND_MODE vertical so `$$`
  // inside it still enters display math. Witness 2305.08368:
  // `\begin{spacing}{1.25}` wrapping the whole body made `$$x_1=...$$` fall
  // through the `$` handler's vertical-only check (tex_math.rs:447) → 199
  // `Error:unexpected:_` cascades. Perl-faithful binding is `#body` only, but
  // the default Package.pm mode is `restricted_horizontal`; in Rust we have
  // to make `internal_vertical` explicit so paragraphs and display math
  // survive the wrap.
  //
  // Every one ends with `\par` (setspace.sty:489-548, `\restore@spacing` :516-523), so its last
  // paragraph closes before the end and the text after it starts a new one (a transparent `#body`
  // left it open, 57l); `{spacing}` begins with `\par` (:525-526) and `{singlespace}` with
  // `\vskip` (:489-495), which end the paragraph before them, while `{onehalfspace}` and
  // `{doublespace}` begin with `\begingroup` alone (:534-548) and go on in it (KPE #320).
  // Each sets its stretch in its group (setspace.sty:489-548: `\setstretch{#1}`,
  // `{\setspace@singlespace}`, `\onehalfspacing`, `\doublespacing`).
  DefEnvironment!("{spacing}{}", "#body", mode => "internal_vertical",
    after_digest_begin => sub[whatsit] {
      if let Some(stretch) = whatsit.get_arg(1) {
        let mut tokens = vec![T_CS!("\\setstretch"), T_BEGIN!()];
        tokens.extend(stretch.revert()?.unlist());
        tokens.push(T_END!());
        digest(Tokens::new(tokens))?;
      }
    },
    before_digest_end => { leave_horizontal()?; },
    // The begin code opens a `\begingroup` (a semi_simple_group) of its own: setspace.sty:525-547 `\begingroup`.
    tex_group => { TexGroup::SemiSimple });
  DefEnvironment!("{singlespace}", "#body", mode => "internal_vertical",
    after_digest_begin => { Digest!("\\setstretch{\\setspace@singlespace}")?; },
    before_digest_end => { leave_horizontal()?; });
  // The paragraph these two close was opened outside their group, which a mode frame cannot end
  // (ARCHITECTURE_THEMES 1), so the `\par` is the document's: the body goes on in the current
  // paragraph and its `<p>` closes after it, where the text that follows starts a new one.
  DefEnvironment!("{onehalfspace}", sub[document, _args, props] {
    body_closing_its_paragraph(document, props)
  }, after_digest_begin => { Digest!("\\onehalfspacing")?; },
    // The begin code opens a `\begingroup` (a semi_simple_group) of its own: setspace.sty:525-547 `\begingroup`.
    tex_group => { TexGroup::SemiSimple });
  DefEnvironment!("{doublespace}", sub[document, _args, props] {
    body_closing_its_paragraph(document, props)
  }, after_digest_begin => { Digest!("\\doublespacing")?; },
    // The begin code opens a `\begingroup` (a semi_simple_group) of its own: setspace.sty:525-547 `\begingroup`.
    tex_group => { TexGroup::SemiSimple });
  // Standalone-switch overrides: some papers (witness 2310.08233 IEEEtran)
  // use `\singlespace` as a SWITCH inside an arg-grabbing context such as
  // `\title{\singlespace ...}`. DefEnvironment binds `\singlespace` to the
  // env-begin CS, which pushes a (restricted_)horizontal mode-switch group.
  // That group cannot be cleanly popped at arg-close, leaking into the
  // following `\@add@frontmatter@now`. Override the env-begin CS (and its
  // sibling `\endsinglespace`) with plain noops so standalone use is inert.
  // For `\begin{singlespace}...\end{singlespace}` blocks: \begin/\end track
  // env scope themselves; the begin/end CSes are just hooks. Without the
  // hooks the body still renders (it is just literal #body), so the env
  // form remains correct.
  def_macro_noop("\\singlespace")?;
  def_macro_noop("\\endsinglespace")?;
  // `\onehalfspace`/`\doublespace` as switches: setspace.sty:534-547 makes each a
  // `\begingroup` and the spacing, ended by its `\end…` (`\restore@spacing`'s
  // `\endgroup`, :516-523). The spacing is presentational and the group is not
  // modelled: a real `\begingroup` there now keeps every word (65c, guard
  // perfect_kernel_batch65c::unclosed_nested_environments_keep_their_text) but nests
  // a later sibling environment inside the earlier one at `\end{document}`; the
  // checked no-op keeps the author's sibling structure (content and semantics ahead
  // of PDF fidelity). What the group does to TeX is reported, where TeX reports
  // it: when the group a switch opened in ends with the switch still open.
  // - A brace or box group: TeX's "Extra }, or forgotten \endgroup" — `{\doublespace
  //   text}` (witness 2310.08233's shape), a tabular cell, `\footnote{…}`, and an
  //   environment whose body is a box: its frame sets `\hsize` (a float's
  //   `\@xfloat`/`\@endfloatbox`, `{minipage}`, subcaption's `{subfigure}`, wrapfig,
  //   float.sty's `{algorithm}`), and its end's `}` meets the switch's group.
  // - Another environment: its `\end`'s `\endgroup` closes the switch's group, not the
  //   environment's, which stays open until the next `\end` — the enclosing
  //   environment's: LaTeX "\begin{itemize} on input line N ended by \end{quote}",
  //   or "… by \end{document}" (`{quote}`, an `\item`'s list, `{center}`). The closer
  //   named is right when no group sits between the environment and the next one out;
  //   with one between (`{\begin{itemize}…}`, a `\parbox` or `{minipage}` around it,
  //   review 5 probes p6/p14/p15/p5) TeX says "Extra }" instead, and a plain group
  //   around the environment is no error in TeX (RED
  //   boxes-groups/plain_group_around_itemize_doublespace). The `\hsize` test is a
  //   proxy: an author's `\hsize=` in a non-box environment, or `{list}`'s inner
  //   frame, reads as a box (wording only, the count is right: probes p21, p4).
  // - A plain `\begingroup … \endgroup`: the `\endgroup` closes the switch's group and
  //   the plain group stays open: the switch re-arms on the enclosing group (Perl
  //   `\aftergroup` order, stomach.rs `pop_stack_frame`), so a plain group inside
  //   `{quote}` still makes the quote's error, and at the document level TeX only
  //   logs "(\end occurred inside a group at level 1)": no error (Perl errs).
  // An `\end…` closes the innermost switch of its own group (a frame-local stack, so
  // of two switches one `\end…` leaves the outer open and named: review 6 probe h4). One met in a nested
  // group closes nothing here, so the switch's group still reports. That errs toward
  // flagging: a switch closed by `\end<name>` from inside a nested plain group still
  // reports, where TeX's `\endgroup`s pair off cleanly (review 4 probe r9, pdflatex 0:
  // RED boxes-groups/doublespace_end_in_a_nested_plain_group).
  // One report per group, however many switches it holds (q6: 120 `\doublespace` in
  // one `{quote}` are one LaTeX error); it names the innermost. A group digested twice
  // (a `\section` title, for the heading and the toc) reports twice, like every error
  // there (Perl and ours alike: RED sectioning-frontmatter/section_title_errors_reported_twice).
  // At the document level the switch runs to `\end{document}` with no error, as in TeX.
  // OXIDIZED_DESIGN #520.
  for (name, spacing) in [
    ("onehalfspace", "\\onehalfspacing"),
    ("doublespace", "\\doublespacing"),
  ] {
    DefPrimitive!(T_CS!(s!("\\{name}")), None, {
      // A switch read from a token list (`\footnote{…}`, a `\section` title) is placed
      // where the document reads that list.
      let at = get_location();
      let at = if at.contains("Anonymous String") {
        get_source_location().unwrap_or_default()
      } else {
        at
      };
      open_setspace_switch(name, at)?;
      Digest!(T_CS!(spacing))?;
    });
    DefPrimitive!(T_CS!(s!("\\end{name}")), None, {
      // A switch opened in this group: its `\endgroup` closes the innermost one, whatever
      // its name (setspace.sty:516-523); the next one out names the report, the last one
      // closes the check.
      if is_value_bound("lx@setspace@open", Some(0)) {
        let mut stack = lookup_string("lx@setspace@stack")
          .split('\u{2}')
          .filter(|entry| !entry.is_empty())
          .map(str::to_string)
          .collect::<Vec<_>>();
        stack.pop();
        let (open, at) = match stack.last().and_then(|top| top.split_once('\u{1}')) {
          Some((open, at)) => (Stored::String(pin(open)), Stored::String(pin(at))),
          None => (Stored::None, Stored::None),
        };
        assign_value("lx@setspace@open", open, Some(Scope::Local));
        assign_value("lx@setspace@open@at", at, Some(Scope::Local));
        assign_value(
          "lx@setspace@stack",
          Stored::String(pin(stack.join("\u{2}"))),
          Some(Scope::Local),
        );
      }
    });
  }
  // Run before the group pops (`beforeAfterGroup`), where the frame still says what
  // it is. The tokens are digested from a string of their own, so the report is
  // placed at the switch (where it opened), its object the group's closer.
  DefPrimitive!(T_CS!("\\lx@setspace@check"), None, {
    let name = lookup_string("lx@setspace@open");
    if name.is_empty() || lookup_bool("lx@ending@document") {
      return Ok(Vec::new());
    }
    let switch_at = lookup_string("lx@setspace@open@at");
    // An environment's frame binds `current_environment` (the `{abstract}` binding
    // captures its body in a frame of its own, `\lx@add@frontmatter@until`, so it
    // reads as a box: "Extra }", where LaTeX reports "\begin{abstract} … ended by").
    let environment =
      is_value_bound("current_environment", Some(0)).then(|| lookup_string("current_environment"));
    let extra_brace = || {
      (
        s!("}}"),
        s!(
          "Extra }}, or forgotten \\endgroup: \\{name}'s \\begingroup (setspace.sty:534-547) is still open"
        ),
      )
    };
    let (object, message) = match environment {
      // A box environment: its frame sets `\hsize` (latex.ltx `\@xfloat`
      // `\hsize\columnwidth`, `\@iiiminipage` `\hsize #4`), its end closes the box.
      Some(_) if is_value_bound("\\hsize", Some(0)) => extra_brace(),
      Some(environment) => {
        // The environment stays open until the next `\end`: the enclosing one's.
        let outer = match value_in_frame("current_environment", 1) {
          Some(Stored::String(outer)) => to_string(outer),
          _ => String::new(),
        };
        let closer = if outer.is_empty() {
          s!("\\end{{document}}")
        } else {
          s!("\\end{{{outer}}}")
        };
        (
          s!("\\end{{{environment}}}"),
          s!(
            "\\end{{{environment}}} closes \\{name}'s \\begingroup (setspace.sty:534-547) instead of \
             {{{environment}}}'s group; {{{environment}}} stays open (LaTeX: \\begin{{{environment}}} … ended by {closer})"
          ),
        )
      },
      None if lookup_bool("groupNonBoxing") => {
        // A plain `\begingroup`: its `\endgroup` closes the innermost switch's group, and
        // the plain group stands open in its place: every switch of this group re-arms on
        // the enclosing one, outermost first (review 7 probes k1, k3).
        assign_value(
          "lx@setspace@rearm",
          Stored::String(pin(lookup_string("lx@setspace@stack"))),
          Some(Scope::Global),
        );
        push_value(
          "afterGroup",
          Stored::Tokens(Tokens::new(vec![T_CS!("\\lx@setspace@rearm")])),
        )?;
        return Ok(Vec::new());
      },
      None => extra_brace(),
    };
    let at = if switch_at.is_empty() {
      String::new()
    } else {
      s!("\n\t{switch_at} (the switch)")
    };
    emit_error(
      "unexpected",
      &object,
      &s!("{message}{at}\n\tIn latexml_package/src/package/setspace_sty.rs\n"),
    );
  });
  DefPrimitive!(T_CS!("\\lx@setspace@rearm"), None, {
    let rearm = lookup_string("lx@setspace@rearm");
    for entry in rearm.split('\u{2}') {
      if let Some((name, at)) = entry.split_once('\u{1}') {
        open_setspace_switch(name, at.to_string())?;
      }
    }
  });
});

/// A setspace environment's body, then the `\par` its end runs (`\restore@spacing`, setspace.sty:516)
/// as the closing of the current `<p>`.
fn body_closing_its_paragraph(document: &mut Document, props: &SymHashMap<Stored>) -> Result<()> {
  if let Some(Stored::Digested(body)) = props.get("body") {
    document.absorb(body, None)?;
  }
  document.maybe_close_element("ltx:p")?;
  Ok(())
}

/// A setspace switch (`\onehalfspace`, `\doublespace`) opened, or still open as a
/// plain group it was opened in ends (`\lx@setspace@rearm`), in a group other than
/// the document's: record it and where it opened (the innermost switch names the
/// report), and arm the group's end check once per group (`\lx@setspace@check`,
/// run before the group pops). The group stacks its open switches
/// (`lx@setspace@stack`), the innermost closed by an `\end…` in the group.
fn open_setspace_switch(name: &str, at: String) -> Result<()> {
  let at_document_level = is_value_bound("current_environment", Some(0))
    && lookup_string("current_environment") == "document";
  if !at_document_level {
    assign_value(
      "lx@setspace@open",
      Stored::String(pin(name)),
      Some(Scope::Local),
    );
    assign_value(
      "lx@setspace@open@at",
      Stored::String(pin(&at)),
      Some(Scope::Local),
    );
    let mut stack = if is_value_bound("lx@setspace@stack", Some(0)) {
      lookup_string("lx@setspace@stack")
    } else {
      String::new()
    };
    if !stack.is_empty() {
      stack.push('\u{2}');
    }
    stack.push_str(&s!("{name}\u{1}{at}"));
    assign_value(
      "lx@setspace@stack",
      Stored::String(pin(&stack)),
      Some(Scope::Local),
    );
    if !is_value_bound("lx@setspace@armed", Some(0)) {
      assign_value("lx@setspace@armed", true, Some(Scope::Local));
      unshift_value("beforeAfterGroup", vec![T_CS!("\\lx@setspace@check")]);
    }
  }
  Ok(())
}
