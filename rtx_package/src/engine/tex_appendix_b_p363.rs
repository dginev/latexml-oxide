use crate::prelude::*;

//======================================================================
// TeX Book, Appendix B. p. 363
LoadDefinitions!({

  DefPrimitive!("\\raggedbottom", None);
  DefPrimitive!("\\normalbottom", None);

  // if the mark is not simple, we add it to the content of the note
  // otherwise, to the attribute.
  DefConstructor!("\\footnote{}{}",
    "^<ltx:note role='footnote' ?#mark(mark='#mark')()>?#prenote(#prenote )()#2</ltx:note>",
    mode => "text", bounded => true,
    before_digest => sub { reenter_text_mode(true); neutralize_font(); },
    after_digest => sub[whatsit] {
      let mark_clone = whatsit.get_arg(1).cloned();
      if let Some(mark) = mark_clone {
        let mark_tks = mark.revert()?.unlist();
        let mut change = false;
        for token in mark_tks {
          if !matches!(token.get_catcode(), Catcode::LETTER | Catcode::SPACE | Catcode::OTHER) {
            change = true;
            break;
          }
        }
        whatsit.set_property(if change { "prenote" } else {"mark"}, mark);
      }
    }
  );

  // Until we can do the "v" properly:
  DefMacro!("\\vfootnote", "\\footnote");
  DefMacro!("\\fo@t",      r"\ifcat\bgroup\noexpand\next \let\next\f@@t  \else\let\next\f@t\fi \next");
  DefMacro!("\\f@@t",      r"\bgroup\aftergroup\@foot\let\next");
  DefMacro!("\\f@t{}",     r"#1\@foot");
  DefMacro!("\\@foot",     r"\strut\egroup");

  DefPrimitive!("\\footstrut", None);
  DefRegister!("\\footins" => Number::new(0));

  DefPrimitive!("\\topinsert",  None);
  DefPrimitive!("\\midinsert",  None);
  DefPrimitive!("\\pageinsert", None);
  DefPrimitive!("\\endinsert",  None);
  // \topins ?

});
