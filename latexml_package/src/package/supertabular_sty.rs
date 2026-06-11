use crate::prelude::*;

#[rustfmt::skip]
LoadDefinitions!({
  // Perl: supertabular.sty.ltxml

  DefPrimitive!("\\@supertabular@bindings [Dimension] AlignmentTemplate", sub[(width, template)] {
    // tabular_bindings is now in latex_constructs, re-exported via prelude
    let mut props = SymHashMap::default();
    props.insert("guess_headers", Stored::Bool(false));
    let mut xml_attrs = HashMap::default();
    if width.value_of() != 0 {
      xml_attrs.insert(s!("width"), width.to_attribute());
    }
    tabular_bindings(template, props, xml_attrs)?;
    Ok(())
  });

  // Environment \begin{supertabular}{pattern} ... \end{supertabular}
  DefMacro!("\\supertabular{}",
    r"\@supertabular@start\@supertabular@bindings{#1}\@@supertabular{#1}\lx@begin@alignment\@supertabular@head");
  DefMacro!("\\endsupertabular",
    r"\@supertabular@tail\lx@end@alignment\@end@tabular\@supertabular@finish");

  DefConstructor!("\\@@supertabular Undigested DigestedBody",
    "#2",
    reversion => r"\begin{supertabular}{#1}#2\end{supertabular}",
    before_digest => { bgroup(); },
    mode => "restricted_horizontal");

  // Environment \begin{supertabular*}{width}{pattern} ... \end{supertabular*}
  DefMacro!("\\csname supertabular*\\endcsname {Dimension}{}",
    r"\@supertabular@start\@supertabular@bindings[#1]{#2}\@@supertabular@{#1}{#2}\lx@begin@alignment\@supertabular@head");
  DefMacro!("\\csname endsupertabular*\\endcsname",
    r"\@supertabular@tail\lx@end@alignment\@end@tabular\@supertabular@finish");

  DefConstructor!("\\@@supertabular@ {Dimension} Undigested DigestedBody",
    "#3",
    reversion => r"\begin{supertabular*}{#1}{#2}#3\end{supertabular*}",
    before_digest => { bgroup(); },
    mode => "restricted_horizontal");

  // mpsupertabular variants
  DefMacro!("\\mpsupertabular{}",
    r"\@supertabular@start\@supertabular@bindings{#1}\@@supertabular{#1}\lx@begin@alignment\@supertabular@head");
  DefMacro!("\\endmpsupertabular",
    r"\@supertabular@tail\lx@end@alignment\@end@tabular\@supertabular@finish");

  DefMacro!("\\csname mpsupertabular*\\endcsname {Dimension}{}",
    r"\@supertabular@start\@supertabular@bindings[#1]{#2}\@@supertabular@{#1}{#2}\lx@begin@alignment\@supertabular@head");
  DefMacro!("\\csname endmpsupertabular*\\endcsname",
    r"\@supertabular@tail\lx@end@alignment\@end@tabular\@supertabular@finish");

  // ======================================================================
  // Table headings/footers: store tokens for later replay via .into()

  DefPrimitive!("\\tablehead{}", sub[(tokens)] {
    assign_value("SUPERTABULAR_HEAD", Stored::Tokens(tokens), Some(Scope::Global));
    Ok(())
  });
  DefPrimitive!("\\tablefirsthead{}", sub[(tokens)] {
    assign_value("SUPERTABULAR_FIRSTHEAD", Stored::Tokens(tokens), Some(Scope::Global));
    Ok(())
  });
  DefPrimitive!("\\tabletail{}", sub[(tokens)] {
    assign_value("SUPERTABULAR_TAIL", Stored::Tokens(tokens), Some(Scope::Global));
    Ok(())
  });
  DefPrimitive!("\\tablelasttail{}", sub[(tokens)] {
    assign_value("SUPERTABULAR_LASTTAIL", Stored::Tokens(tokens), Some(Scope::Global));
    Ok(())
  });

  // Emit the head/tail: body returns Vec<Token> directly (not Result)
  DefMacro!("\\@supertabular@head", sub[_args] {
    let head = lookup_value("SUPERTABULAR_FIRSTHEAD")
      .or_else(|| lookup_value("SUPERTABULAR_HEAD"));
    if let Some(Stored::Tokens(ref toks)) = head {
      let mut result = vec![T_CS!("\\lx@alignment@begin@heading")];
      result.extend_from_slice(toks.unlist_ref());
      result.push(T_CS!("\\lx@alignment@end@heading"));
      result
    } else {
      Vec::new()
    }
  });

  DefMacro!("\\@supertabular@tail", sub[_args] {
    let tail = lookup_value("SUPERTABULAR_LASTTAIL")
      .or_else(|| lookup_value("SUPERTABULAR_TAIL"));
    if let Some(Stored::Tokens(ref toks)) = tail {
      let mut result = vec![T_CS!("\\lx@alignment@begin@heading")];
      result.extend_from_slice(toks.unlist_ref());
      result.push(T_CS!("\\lx@alignment@end@heading"));
      result
    } else {
      Vec::new()
    }
  });

  // ======================================================================
  // Captions

  RawTeX!(r"\newif\if@topcaption\@topcaptiontrue");

  DefMacro!("\\topcaption",    r"\@topcaptiontrue\tablecaption");
  DefMacro!("\\bottomcaption", r"\@topcaptionfalse\tablecaption");

  DefPrimitive!("\\tablecaption []{}", sub[(toccaption, caption)] {
    if let Some(tc) = toccaption {
      assign_value("SUPERTABULAR_TOCCAPTION", Stored::Tokens(tc), None);
    }
    assign_value("SUPERTABULAR_CAPTION", Stored::Tokens(caption), None);
    Ok(())
  });

  DefMacro!("\\@supertabular@topcaption",    r"\if@topcaption\@supertabular@docaption\fi");
  DefMacro!("\\@supertabular@bottomcaption", r"\if@topcaption\else\@supertabular@docaption\fi");

  DefMacro!("\\@supertabular@docaption", sub[_args] {
    let cap = lookup_value("SUPERTABULAR_CAPTION");
    let toccap = lookup_value("SUPERTABULAR_TOCCAPTION");
    if let Some(Stored::Tokens(ref c)) = cap {
      let mut result: Vec<Token> = vec![T_CS!("\\@caption"), T_BEGIN!()];
      result.extend(Explode!("table"));
      result.push(T_END!());
      if let Some(Stored::Tokens(ref tc)) = toccap
        && !tc.is_empty() {
          result.push(T_OTHER!("["));
          result.extend_from_slice(tc.unlist_ref());
          result.push(T_OTHER!("]"));
        }
      result.push(T_BEGIN!());
      result.extend_from_slice(c.unlist_ref());
      result.push(T_END!());
      result
    } else {
      Vec::new()
    }
  });

  // ======================================================================

  DefPrimitive!("\\@supertabular@clear", sub[_args] {
    assign_value("SUPERTABULAR_TOCCAPTION", false, Some(Scope::Global));
    assign_value("SUPERTABULAR_CAPTION", false, Some(Scope::Global));
    Ok(())
  });

  DefMacro!("\\@supertabular@start",  r"\begin{table}\@supertabular@topcaption");
  DefMacro!("\\@supertabular@finish", r"\@supertabular@bottomcaption\end{table}\@supertabular@clear");

  DefMacro!("\\shrinkheight{Dimension}", None);
});
