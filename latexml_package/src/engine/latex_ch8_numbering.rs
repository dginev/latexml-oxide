use crate::prelude::*;
const FNSYMBOLS: &[&str] = &[
  "*",
  "\u{2020}",
  "\u{2021}",
  "\u{00A7}",
  "\u{00B6}",
  "\u{2225}",
  "**",
  "\u{2020}\u{2020}",
  "\u{2021}\u{2021}",
];

LoadDefinitions!({
  //======================================================================
  // C.8.4 Numbering
  //======================================================================
  // For LaTeX documents, We want id's on para, as well as sectional units.
  // However, para get created implicitly on Document construction, rather than
  // explicitly during digestion (via a whatsit), we can't use the usual LaTeX counter mechanism.
  Tag!("ltx:para", after_open => sub[document, node] {
    document.generate_id(node, "p")?;
  });

  DefPrimitive!("\\newcounter{}[]", sub[(cs, default_opt)] {
    let default = if let Some(tks) = default_opt {
      if !tks.is_empty() {
        Expand!(tks)
      } else {
        Tokens!()
      }
    } else {
      Tokens!()
    };
    let cs_expanded = &Expand!(cs).to_string();
    NewCounter!(cs_expanded, &default.to_string());
  });
  DefPrimitive!("\\setcounter{}{Number}", sub[(cs, default)] {
    let cs_expanded = &Expand!(cs).to_string();
    SetCounter!(cs_expanded, default);
  });
  DefPrimitive!("\\addtocounter{}{Number}", sub[(cs,default)] {
    let cs_expanded = &Expand!(cs).to_string();
    AddToCounter!(cs_expanded, default);
  });
  DefPrimitive!("\\stepcounter{}",    sub[(cs)] {
    let cs_expanded = &Expand!(cs).to_string();
    StepCounter!(cs_expanded, false)?;
  });
  DefPrimitive!("\\refstepcounter{}", sub[(cs)] {
    let cs_expanded = &Expand!(cs).to_string();
    RefStepCounter!(cs_expanded, false)?;
  });

  // Perl latex_constructs.pool.ltxml: addtoCounterReset + defCounterID
  DefPrimitive!("\\@addtoreset{}{}", sub[(ctr, within)] {
    let ctr_str = Expand!(ctr).to_string();
    let within_str = Expand!(within).to_string();
    let unctr = s!("UN{}", ctr_str);
    let reg = s!("\\cl@{}", within_str);
    // Prepend ctr and UNctr to the counter reset list for 'within'
    let prev = state::lookup_tokens(&reg).unwrap_or_default();
    let mut toks = vec![T_CS!(ctr_str.clone()), T_CS!(unctr)];
    toks.extend(prev.unlist());
    state::assign_value(&reg, Stored::Tokens(Tokens::new(toks)), None);
  });

  // Perl: latex_constructs.pool.ltxml \@removefromreset
  DefPrimitive!("\\@removefromreset{}{}", sub[(ctr, within)] {
    let ctr_str = Expand!(ctr).to_string();
    let within_str = Expand!(within).to_string();
    let reg = s!("\\cl@{}", within_str);
    if let Some(prev) = state::lookup_tokens(&reg) {
      let ctr_cs = T_CS!(ctr_str.clone());
      let unctr_cs = T_CS!(s!("UN{}", ctr_str));
      let filtered: Vec<Token> = prev.unlist().into_iter()
        .filter(|t| *t != ctr_cs && *t != unctr_cs)
        .collect();
      state::assign_value(&reg, Stored::Tokens(Tokens::new(filtered)), None);
    }
  });

  // Perl: latex_constructs.pool.ltxml \counterwithin
  DefPrimitive!("\\counterwithin OptionalMatch:* {}{}", sub[(star, ctr, within)] {
    let ctr_str = Expand!(ctr).to_string();
    let within_str = Expand!(within).to_string();
    // Add ctr to reset list of within
    let unctr = s!("UN{}", ctr_str);
    let reg = s!("\\cl@{}", within_str);
    let prev = state::lookup_tokens(&reg).unwrap_or_default();
    let mut toks = vec![T_CS!(ctr_str.clone()), T_CS!(unctr)];
    toks.extend(prev.unlist());
    state::assign_value(&reg, Stored::Tokens(Tokens::new(toks)), None);
    if star.is_none() {
      // Redefine \thectr to include \thewithin prefix
      let the_ctr = T_CS!(s!("\\the{}", ctr_str));
      let expansion = s!("\\the{}.\\arabic{{{}}}", within_str, ctr_str);
      let _ = def_macro(the_ctr, None,
        Some(ExpansionBody::from(expansion)),
        Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))));
      // defCounterID with within
      let prefix = state::lookup_string(&s!("@ID@prefix@{}", ctr_str));
      let clean_prefix = if prefix.is_empty() { ctr_str.clone() } else { prefix };
      let ctr_for_id = ctr_str.clone();
      let within_for_id = within_str.clone();
      let thectrid = s!("\\the{}@ID", ctr_str);
      let _ = def_macro(T_CS!(thectrid), None,
        Some(ExpansionBody::Closure(Rc::new(move |_args| {
          Ok(mouth::tokenize_internal(&s!(
            "\\expandafter\\ifx\\csname the{}@ID\\endcsname\\@empty\\else\\csname the{}@ID\\endcsname.\\fi {}\\csname @{}@ID\\endcsname",
            within_for_id, within_for_id, clean_prefix, ctr_for_id
          )))
        }))),
        Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))));
    }
  });

  // Perl: latex_constructs.pool.ltxml \counterwithout
  DefPrimitive!("\\counterwithout OptionalMatch:* {}{}", sub[(star, ctr, within)] {
    let ctr_str = Expand!(ctr).to_string();
    let within_str = Expand!(within).to_string();
    // Remove ctr from reset list of within
    let reg = s!("\\cl@{}", within_str);
    if let Some(prev) = state::lookup_tokens(&reg) {
      let ctr_cs = T_CS!(ctr_str.clone());
      let unctr_cs = T_CS!(s!("UN{}", ctr_str));
      let filtered: Vec<Token> = prev.unlist().into_iter()
        .filter(|t| *t != ctr_cs && *t != unctr_cs)
        .collect();
      state::assign_value(&reg, Stored::Tokens(Tokens::new(filtered)), None);
    }
    if star.is_none() {
      // Redefine \thectr without prefix
      let the_ctr = T_CS!(s!("\\the{}", ctr_str));
      let expansion = s!("\\arabic{{{}}}", ctr_str);
      let _ = def_macro(the_ctr, None,
        Some(ExpansionBody::from(expansion)),
        Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))));
      // defCounterID without within — redefine \thectr@ID
      let prefix = state::lookup_string(&s!("@ID@prefix@{}", ctr_str));
      let clean_prefix = if prefix.is_empty() { ctr_str.clone() } else { prefix };
      let ctr_for_id = ctr_str.clone();
      let thectrid = s!("\\the{}@ID", ctr_str);
      let _ = def_macro(T_CS!(thectrid), None,
        Some(ExpansionBody::Closure(Rc::new(move |_args| {
          Ok(mouth::tokenize_internal(&s!(
            "{}\\csname @{}@ID\\endcsname", clean_prefix, ctr_for_id
          )))
        }))),
        Some(NewDefault!(ExpandableOptions, scope => Some(Scope::Global))));
    }
  });

  DefMacro!("\\cl@@ckpt", "\\@elt{page}");

  DefMacro!("\\value{}", sub[(value)] {
    T_CS!(s!("\\c@{}", Expand!(value)))
  });

  DefMacro!("\\@arabic{Number}", sub[(number)] {
    ExplodeText!(number.value_of().to_string())
  });
  DefMacro!("\\arabic{}", sub[(value)] {
    let ctr_expansion = Expand!(value).to_string();
    let ctr_value = CounterValue!(&ctr_expansion).value_of();
    ExplodeText!(ctr_value)
  });

  DefMacro!("\\@roman{Number}", sub[(number)] {
    ExplodeText!(radix::radix_roman(number.value_of()))
  });
  DefMacro!("\\roman{}", sub[(token)] {
    let ctr = Expand!(token).to_string();
    ExplodeText!(radix::radix_roman(CounterValue!(&ctr).value_of()))
  });
  DefMacro!("\\@Roman{Number}", sub[(number)] {
    ExplodeText!(radix::radix_up_roman(number.value_of()))
  });
  DefMacro!("\\Roman{}", sub[(token)] {
    let ctr = Expand!(token).to_string();
    ExplodeText!(radix::radix_up_roman(CounterValue!(&ctr).value_of()))
  });
  DefMacro!("\\@alph{Number}", sub[(number)] {
    ExplodeText!(radix::radix_alpha(number.value_of()))
  });
  DefMacro!("\\alph{}", sub[(token)] {
    let ctr = Expand!(token).to_string();
    ExplodeText!(radix::radix_alpha(CounterValue!(&ctr).value_of()))
  });
  DefMacro!("\\@Alph{Number}", sub[(number)] {
    ExplodeText!(radix::radix_up_alpha(number.value_of()))
  });
  DefMacro!("\\Alph{}", sub[(token)] {
    let ctr = Expand!(token).to_string();
    ExplodeText!(radix::radix_up_alpha(CounterValue!(&ctr).value_of()))
  });

  DefMacro!("\\@fnsymbol{Number}", sub[(number)] {
    ExplodeText!(radix::radix_format_str(number.value_of(), FNSYMBOLS))
  });
  DefMacro!("\\fnsymbol{}", sub[(token)] {
    let ctr = Expand!(token).to_string();
    ExplodeText!(radix::radix_format_str(CounterValue!(&ctr).value_of(), FNSYMBOLS))
  });
});
