use crate::package::*;
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

LoadDefinitions!(state, {
  //======================================================================
  // C.8.4 Numbering
  //======================================================================
  // For LaTeX documents, We want id's on para, as well as sectional units.
  // However, para get created implicitly on Document construction, rather than
  // explicitly during digestion (via a whatsit), we can't use the usual LaTeX counter mechanism.
  Tag!("ltx:para", after_open => sub[document, node, state] {
    document.generate_id(node, "p", state)?;
  });

  DefPrimitive!("\\newcounter{}[]", sub[stomach, (cs, default_opt), state] {
    let gullet = stomach.get_gullet_mut();
    let default = if let Some(tks) = default_opt {
      if !tks.is_empty() {
        Expand!(tks, gullet)
      } else {
        Tokens!()
      }
    } else {
      Tokens!()
    };
    let cs_expanded = &Expand!(cs, gullet).to_string();
    NewCounter!(cs_expanded, &default.to_string());
  });
  DefPrimitive!("\\setcounter{}{Number}", sub[stomach, (cs, default), state] {
    let gullet = stomach.get_gullet_mut();
    let cs_expanded = &Expand!(cs, gullet).to_string();
    SetCounter!(cs_expanded, default, stomach, state);
  });
  DefPrimitive!("\\addtocounter{}{Number}", sub[stomach, (cs,default), state] {
    let gullet = stomach.get_gullet_mut();
    let cs_expanded = &Expand!(cs, gullet).to_string();
    AddToCounter!(cs_expanded, default, gullet);
  });
  DefPrimitive!("\\stepcounter{}",    sub[stomach, (cs), state] {
    let gullet = stomach.get_gullet_mut();
    let cs_expanded = &Expand!(cs, gullet).to_string();
    StepCounter!(cs_expanded, false, stomach)?;
  });
  DefPrimitive!("\\refstepcounter{}", sub[stomach, (cs), state] {
    let gullet = stomach.get_gullet_mut();
    let cs_expanded = &Expand!(cs, gullet).to_string();
    RefStepCounter!(cs_expanded, false, stomach)?;
  });

  // DefPrimitive('\@addtoreset{}{}', sub {
  //     my ($stomach, $ctr, $within) = @_;
  //     $ctr    = ToString(Expand($ctr));
  //     $within = ToString(Expand($within));
  //     my $unctr = "UN$ctr";    # UNctr is counter for generating ID's for UN-numbered items.
  //     AssignValue("\\cl\@$within" =>
  //         Tokens(T_CS($ctr), T_CS($unctr),
  //         (LookupValue("\\cl\@$within") ? LookupValue("\\cl\@$within")->unlist : ())),
  //       'global');
  //     # This counter might be doing double duty generating ID's as well, so we may need to patch
  // up.     my $prefix = LookupValue('@ID@prefix@' . $ctr);
  //     if (defined $prefix) {
  //       DefMacroI(T_CS("\\the$ctr\@ID"), undef,
  //         "\\expandafter\\ifx\\csname the$within\@ID\\endcsname\\\@empty"
  //           . "\\else\\csname the$within\@ID\\endcsname.\\fi"
  //           . " $prefix\\csname \@$ctr\@ID\\endcsname",
  //         scope => 'global');
  //       DefMacroI(T_CS("\\\@$ctr\@ID"), undef, "0", scope => 'global'); }
  //     return; });

  DefMacro!("\\value{}", sub[gullet, (value), inner_state] {
    let ctr_expansion = Expand!(value, gullet, inner_state).to_string();
    let ctr_value = CounterValue!(&ctr_expansion, inner_state).value_of();
    ExplodeText!(ctr_value)
  });

  DefMacro!("\\@arabic{Number}", sub[gullet, (number), state] {
    let value = number.value_of();
    ExplodeText!(value.to_string())
  });
  DefMacro!("\\arabic{}", sub[gullet, (value), inner_state] {
    let ctr_expansion = Expand!(value, gullet, inner_state).to_string();
    let ctr_value = CounterValue!(&ctr_expansion, inner_state).value_of();
    ExplodeText!(ctr_value)
  });

  DefMacro!("\\@roman{Number}", sub[gullet, (number), state] {
    let value = number.value_of();
    ExplodeText!(radix::radix_roman(value))
  });
  DefMacro!("\\roman{}", sub[gullet, (token), state] {
    let ctr = Expand!(token, gullet).to_string();
    ExplodeText!(radix::radix_roman(CounterValue!(&ctr).value_of()))
  });
  DefMacro!("\\@Roman{Number}", sub[gullet, (number), state] {
    let value = number.value_of();
    ExplodeText!(radix::radix_up_roman(value))
  });
  DefMacro!("\\Roman{}", sub[gullet, (token), state] {
    let ctr = Expand!(token, gullet).to_string();
    ExplodeText!(radix::radix_up_roman(CounterValue!(&ctr).value_of()))
  });
  DefMacro!("\\@alph{Number}", sub[gullet, (number), state] {
    let value = number.value_of();
    ExplodeText!(radix::radix_alpha(value))
  });
  DefMacro!("\\alph{}", sub[gullet, (token), state] {
    let ctr = Expand!(token, gullet).to_string();
    ExplodeText!(radix::radix_alpha(CounterValue!(&ctr).value_of()))
  });
  DefMacro!("\\@Alph{Number}", sub[gullet, (number), state] {
    let value = number.value_of();
    ExplodeText!(radix::radix_up_alpha(value))
  });
  DefMacro!("\\Alph{}", sub[gullet, (token), state] {
    let ctr = Expand!(token, gullet).to_string();
    ExplodeText!(radix::radix_up_alpha(CounterValue!(&ctr).value_of()))
  });

  DefMacro!("\\@fnsymbol{Number}", sub[gullet, (number), state] {
    ExplodeText!(radix::radix_format_str(number.value_of(), FNSYMBOLS))
  });
  DefMacro!("\\fnsymbol{}", sub[gullet, (token), state] {
    let ctr = Expand!(token, gullet).to_string();
    ExplodeText!(radix::radix_format_str(CounterValue!(&ctr).value_of(), FNSYMBOLS))
  });
});
