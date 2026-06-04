use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: longtable.sty.ltxml
  // NOTE: The way the headers & footers are captured causes trailing \hlines
  // to generate empty rows.

  // Environment \begin{longtable}[align]{pattern} ... \end{longtable}
  DefMacro!("\\longtable[]{}",
    r"\lx@longtable@bindings{#2}\@@longtable[#1]{#2}\lx@begin@alignment");
  DefMacro!("\\endlongtable",
    r"\lx@end@alignment\@end@tabular");
  // {longtable*} is defined in revtex4-1 to be able to span a two column document
  DefMacro!("\\csname longtable*\\endcsname []{}",
    r"\lx@longtable@bindings{#2}\@@longtable[#1]{#2}\lx@begin@alignment");
  DefMacro!("\\csname endlongtable*\\endcsname",
    r"\lx@end@alignment\@end@tabular");

  DefMacro!("\\@gobble@optional[]", None);

  DefConstructor!("\\@@longtable [] Undigested DigestedBody",
    "<ltx:table xml:id='#id' inlist='lot' labels='#label'>#tags?#headcaption(<ltx:caption>#headcaption</ltx:caption>)?#headtoccaption(<ltx:toccaption>#headtoccaption</ltx:toccaption>)#3?#footcaption(<ltx:caption>#footcaption</ltx:caption>)?#foottoccaption(<ltx:toccaption>#foottoccaption</ltx:toccaption>)</ltx:table>",
    reversion => r"\begin{longtable}[#1]{#2}#3\end{longtable}",
    before_digest => {
      bgroup();
      state::let_i(&T_CS!("\\pagebreak"), &T_CS!("\\@gobble@optional"), None);
    },
    after_digest => sub[whatsit] {
      // Insert properties from LONGTABLE_PROPERTIES
      if let Some(Stored::HashStored(ref map)) = lookup_value("LONGTABLE_PROPERTIES") {
        for (k, v) in map.iter() {
          arena::with(*k, |key| whatsit.set_property(key, v.clone()));
        }
      }
      // Insert head captions (from \endfirsthead or \endhead)
      let head_captions = lookup_value("LONGTABLE_HEAD_CAPTIONS")
        .or_else(|| lookup_value("LONGTABLE_CAPTIONS"));
      if let Some(Stored::VecDigested(ref captions)) = head_captions {
        if captions.len() >= 2 {
          if !captions[1].to_string().is_empty() {
            whatsit.set_property("headcaption", captions[1].clone());
          }
          if !captions[0].to_string().is_empty() {
            whatsit.set_property("headtoccaption", captions[0].clone());
          }
        }
      }
      // Insert foot captions
      if let Some(Stored::VecDigested(ref captions)) = lookup_value("LONGTABLE_FOOT_CAPTIONS") {
        if captions.len() >= 2 {
          if !captions[1].to_string().is_empty() {
            whatsit.set_property("footcaption", captions[1].clone());
          }
          if !captions[0].to_string().is_empty() {
            whatsit.set_property("foottoccaption", captions[0].clone());
          }
        }
      }
      // Insert label
      if let Some(Stored::String(label)) = lookup_value("LONGTABLE_LABEL") {
        whatsit.set_property("label", arena::to_string(label));
      }
      // Reinsert head/foot rows into alignment
      if let Some(alignment) = lookup_alignment() {
        if let Some(data) = alignment.alignment_cell() {
          let mut al = data.borrow_mut();
          let head = std::mem::take(&mut al.head_rows);
          let foot = std::mem::take(&mut al.foot_rows);
          if !head.is_empty() {
            al.prepend_rows(head);
          }
          if !foot.is_empty() {
            al.append_rows(foot);
          }
        }
      }
      Ok(Vec::new())
    },
    mode => "restricted_horizontal");

  DefPrimitive!("\\lx@longtable@bindings AlignmentTemplate", sub[(template)] {
    longtable_bindings(template)?;
    Ok(())
  });

  // These macros appear within the longtable, at the beginning.
  // They cut off the previous lines to be used as headers or footers.
  DefMacro!("\\lx@longtable@endfirsthead", r"\crcr\noalign{\lx@longtable@grab{FIRSTHEAD}}");
  DefMacro!("\\lx@longtable@endhead",      r"\crcr\noalign{\lx@longtable@grab{HEAD}}");
  DefMacro!("\\lx@longtable@endfoot",      r"\crcr\noalign{\lx@longtable@grab{FOOT}}");
  DefMacro!("\\lx@longtable@endlastfoot",  r"\crcr\noalign{\lx@longtable@grab{LASTFOOT}}");
  // Real longtable `\kill` is `\LT@echunk` — it ENDS the current row (the row
  // is measured for widths then discarded). Model that faithfully: end the row
  // via `\crcr` (which closes the column's `\vtop{\hbox{…` boxing through the
  // normal cr path, exactly like `\\`), with a flag that tells the alignment
  // driver to drop the just-ended row. The older `\crcr\noalign{marker}` form
  // (still defined below for compatibility) leaked the cell box / popped the
  // wrong row; routing through this flag avoids both. Witness 2010.09763.
  DefPrimitive!("\\lx@longtable@kill@flag", sub[_args] {
    assign_value("LONGTABLE_KILL_NEXT", true, Some(Scope::Global));
    Ok(())
  });
  DefMacro!("\\lx@longtable@kill",         r"\lx@longtable@kill@flag\crcr");

  DefPrimitive!("\\lx@longtable@grab{}", sub[(name_arg)] {
    let name = name_arg.to_string();
    if let Some(alignment) = lookup_alignment() {
      if let Some(data) = alignment.alignment_cell() {
        let mut al = data.borrow_mut();
        // Remove all preceding rows and mark columns as thead.
        let mut rows = Vec::new();
        while let Some(mut row) = al.remove_row() {
          for col in row.get_columns_mut() {
            col.thead_in_column = true;
          }
          rows.push(row);
        }
        rows.reverse(); // restore original order (remove_row pops from back)
        if name == "FIRSTHEAD" || (name == "HEAD" && al.head_rows.is_empty()) {
          al.head_rows = rows;
          if let Some(caption) = lookup_value("LONGTABLE_CAPTIONS") {
            assign_value("LONGTABLE_CAPTIONS", Stored::None, Some(Scope::Global));
            assign_value("LONGTABLE_HEAD_CAPTIONS", caption, Some(Scope::Global));
          }
        } else if name == "LASTFOOT" || (name == "FOOT" && al.foot_rows.is_empty()) {
          al.foot_rows = rows;
          if let Some(caption) = lookup_value("LONGTABLE_CAPTIONS") {
            assign_value("LONGTABLE_CAPTIONS", Stored::None, Some(Scope::Global));
            assign_value("LONGTABLE_FOOT_CAPTIONS", caption, Some(Scope::Global));
          }
        }
      }
    }
    Ok(())
  });

  DefConstructor!("\\lx@longtable@kill@marker", "", reversion => "\\kill",
    after_digest => sub[_args] {
      if let Some(alignment) = lookup_alignment() {
        if let Some(data) = alignment.alignment_cell() {
          data.borrow_mut().remove_row();
        }
      }
      Ok(Vec::new())
    });

  // Caption gets redefined.
  DefMacro!("\\lx@longtable@caption[]{}",
    r"\lx@longtable@caption@{\lx@format@toctitle@@{table}{\ifx.#1.#2\else#1\fi}}{\lx@format@title@@{table}{#2}}");
  DefPrimitive!("\\lx@longtable@caption@{}{}", sub[(toccap, cap)] {
    // Perl: AssignValue(LONGTABLE_CAPTIONS => [DigestText($toccap), DigestText($cap)], 'global')
    let toccap_digested = digest_text(toccap)?;
    let cap_digested = digest_text(cap)?;
    let captions = Stored::VecDigested(vec![toccap_digested, cap_digested]);
    assign_value("LONGTABLE_CAPTIONS", captions, Some(Scope::Global));
    Ok(())
  });
  DefPrimitive!("\\lx@longtable@label Semiverbatim", sub[(label)] {
    // Perl: AssignValue(LONGTABLE_LABEL => CleanLabel(ToString($label)), 'global')
    let label = clean_label(&label.to_string(), None).into_owned();
    assign_value("LONGTABLE_LABEL", Stored::String(arena::pin(label)), Some(Scope::Global));
    Ok(())
  });

  // Not used, but must be defined.
  TeX!(r"\newskip\LTleft \LTleft=0pt plus 1fill
\newskip\LTright \LTright=0pt plus 1fill
\newskip\LTpre \LTpre=12pt plus 4pt minus 4pt
\newskip\LTpost \LTpost=12pt plus 4pt minus 4pt
\newdimen\LTcapwidth \LTcapwidth=4in
\newcount\LTchunksize \LTchunksize=200
\newcount\LT@cols
\newcount\LT@rows
");
  state::let_i(&T_CS!("\\c@LTchunksize"), &T_CS!("\\LTchunksize"), None);

  TeX!(r"\newbox\LT@head
\newbox\LT@firsthead
\newbox\LT@foot
\newbox\LT@lastfoot
\newbox\LT@gbox
");

  state::let_i(&T_CS!("\\setlongtables"), &T_CS!("\\relax"), None);
});

fn longtable_bindings(template: Template) -> Result<()> {
  let mut props = SymHashMap::default();
  props.insert("guess_headers", Stored::Bool(false));
  tabular_bindings(template, props, HashMap::default())?;
  state::let_i(
    &T_CS!("\\endfirsthead"),
    &T_CS!("\\lx@longtable@endfirsthead"),
    None,
  );
  state::let_i(&T_CS!("\\endhead"), &T_CS!("\\lx@longtable@endhead"), None);
  state::let_i(&T_CS!("\\endfoot"), &T_CS!("\\lx@longtable@endfoot"), None);
  state::let_i(
    &T_CS!("\\endlastfoot"),
    &T_CS!("\\lx@longtable@endlastfoot"),
    None,
  );
  state::let_i(&T_CS!("\\caption"), &T_CS!("\\lx@longtable@caption"), None);
  state::let_i(&T_CS!("\\label"), &T_CS!("\\lx@longtable@label"), None);
  state::let_i(&T_CS!("\\kill"), &T_CS!("\\lx@longtable@kill"), None);

  assign_value("LONGTABLE_LABEL", Stored::None, Some(Scope::Global));
  assign_value("LONGTABLE_CAPTIONS", Stored::None, Some(Scope::Global));
  assign_value("LONGTABLE_HEAD_CAPTIONS", Stored::None, Some(Scope::Global));
  assign_value("LONGTABLE_FOOT_CAPTIONS", Stored::None, Some(Scope::Global));
  assign_value("LONGTABLE_HEAD", Stored::None, Some(Scope::Global));
  assign_value("LONGTABLE_FOOT", Stored::None, Some(Scope::Global));

  // properties happen too late!!! - do RefStepCounter now
  let props = ref_step_counter("table", false)?;
  assign_value(
    "LONGTABLE_PROPERTIES",
    Stored::HashStored(props),
    Some(Scope::Global),
  );

  Ok(())
}
