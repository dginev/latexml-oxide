//! TeX Page
//!
//! Core TeX Implementation for LaTeXML

use crate::prelude::*;
LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Page Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  //======================================================================
  // Parameters for page layout
  //----------------------------------------------------------------------
  // \hoffset          pd is a value added to the default 1-inch left margin.
  // \voffset          pd is a value added to the default 1-inch top margin.
  // \topskip          pg is special glue added before the first box on each page.
  // \pagedepth        iq is the actual depth of the last box on the main page.
  // \pagetotal        iq is the accumulated height of the current page.
  // \maxdepth         pd is the maximum depth of boxes on the main page.
  // \vsize            pd is the desired height of the current page.
  // \pagegoal         iq is the desired height of the current page.
  // \pageshrink       iq is the amount of finite shrinkability in the current page.
  // \pagestretch      iq is the amount of finite stretchability in the current page.
  // \pagefilllstretch iq is the amount of third-order infinite stretchability in the current page.
  // \pagefillstretch  iq is the amount of second-order infinite stretchability in the current page.
  // \pagefilstretch   iq is the amount of first-order infinite stretchability in the current page.

  DefRegister!("\\hoffset", Dimension!("0"));
  DefRegister!("\\voffset", Dimension!("0"));
  DefRegister!("\\topskip", Glue!("10pt"));
  DefRegister!("\\pagedepth", Dimension::new(0));
  DefRegister!("\\pagetotal", Dimension::new(0));
  DefRegister!("\\maxdepth", Dimension!("4pt"));
  DefRegister!("\\vsize", Dimension!("8.9in"));

  // tex.web §982/§987: `page_goal` is `max_dimen` only while the current
  // page is EMPTY; the moment the page builder contributes a box it becomes
  // `\vsize` (minus insertions). Our model has no page builder, so a single
  // standing value must serve every "free space on this page" probe: Perl
  // uses Dimension(0) (fullwidth.sty:243-273 `\fwd@freepagevspace` computes
  // 0−0 and retries `\vfill\eject` forever — SHARED loop), batch 28 used
  // `\maxdimen` (fullwidth's `\ifdimequal{\pagegoal}{\maxdimen}` branch
  // then took `\vsize`, but fillwith.sty:319 `\dim_until_do … \pagegoal -
  // \footskip - \pagetotal …` stacked ~1300 line coffins toward a 16384pt
  // goal and l3coffins' quadratic corner naming ran to the TokenLimit).
  // `\vsize` — the mid-page value — satisfies both: fullwidth's free space
  // is `\vsize − \pagetotal`, fillwith fills one page (~50 coffins with a
  // real `\strutbox`, latex_constructs_rust_only.rs). Guard:
  // `perfect_kernel_batch54::pagegoal_is_vsize_and_strutbox_is_real`.
  DefRegister!("\\pagegoal", Dimension!("8.9in"),
  getter => sub[_args] {
    // Follow `\vsize` (classes set it) unless the document assigned
    // `\pagegoal` itself.
    match lookup_value("pagegoal_assigned") {
      Some(Stored::Dimension(d)) => Some(RegisterValue::Dimension(d)),
      _ => lookup_dimension_cs("\\vsize", true).map(RegisterValue::Dimension),
    }
  },
  setter => sub[value, scope, _args] {
    if let RegisterValue::Dimension(d) = value {
      assign_value("pagegoal_assigned", Stored::Dimension(d), scope);
    }
  });
  DefRegister!("\\pagestretch", Dimension::new(0));
  DefRegister!("\\pagefilstretch", Dimension::new(0));
  DefRegister!("\\pagefillstretch", Dimension::new(0));
  DefRegister!("\\pagefilllstretch", Dimension::new(0));
  DefRegister!("\\pageshrink", Dimension::new(0));

  //======================================================================
  // Usable for things line \clearpage, etc.
  //
  // OXIDIZED_DESIGN #178: a shipped page advances `\c@page` (latex.ltx:15271
  // `\@outputpage`'s `\global\advance\c@page\@ne`; plain.tex `\advancepageno`).
  // Neither engine has an output routine, and Perl's marker leaves the counter
  // at 1 forever, so the standard "pad to page N" idiom
  // `\loop\ifnum\value{page}<N \null\clearpage\repeat` (knowledge.tex:803-809)
  // never ends (Perl hangs; Rust's box-cycle guard made it a Fatal). Counting
  // the page markers is the honest model of a page count. Guard:
  // `perfect_kernel_batch54::clearpage_advances_the_page_counter`.
  DefMacro!(
    "\\lx@newpage",
    "\\lx@newpage@mark\\global\\advance\\c@page\\@ne"
  );
  DefConstructor!("\\lx@newpage@mark", "^<ltx:pagination role='newpage'/>");
});
