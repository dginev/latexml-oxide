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
    generate_id(document, node, "p", state)?;
  });

  DefPrimitive!("\\newcounter{}[]", sub[stomach, args, state] {
    unpack_opt!(args => cs_opt, default_opt);
    let cs = cs_opt.owned_tokens().unwrap();
    let gullet = stomach.get_gullet_mut();
    let default = if !default_opt.is_empty() {
      Expand!(default_opt.owned_tokens().unwrap(), gullet)
    } else {
      Tokens!()
    };
    let cs_expanded = &Expand!(cs, gullet).to_string();
    NewCounter!(cs_expanded, &default.to_string());
  });
  DefPrimitive!("\\setcounter{}{Number}", sub[stomach, args, state] {
    unpack!(args=>cs, default);
    let gullet = stomach.get_gullet_mut();
    let cs_expanded = &Expand!(cs, gullet).to_string();
    let default = default.to_number();
    SetCounter!(cs_expanded, default, stomach, state);
  });
  DefPrimitive!("\\addtocounter{}{Number}", sub[stomach, args, state] {
    unpack!(args=>cs, default);
    let gullet = stomach.get_gullet_mut();
    let cs_expanded = &Expand!(cs, gullet).to_string();
    // TODO: Continue here: The {Number} parameter type should be expanded already,
    //       I need to carefully study the differences with LaTeXML and smooth the edges
    //
    let default = Expand!(default, gullet).to_number();
    AddToCounter!(cs_expanded, default, gullet);
  });
  DefPrimitive!("\\stepcounter{}",    sub[stomach, args, state] {
    unpack!(args=>cs);
    let gullet = stomach.get_gullet_mut();
    let cs_expanded = &Expand!(cs, gullet).to_string();
    StepCounter!(cs_expanded, false, stomach)?;
  });
  DefPrimitive!("\\refstepcounter{}", sub[stomach, args, state] {
    unpack!(args=>cs);
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

  DefMacro!("\\value{}", sub[gullet, args, inner_state] {
    unpack!(args => value);
    let ctr_expansion = Expand!(value, gullet, inner_state).to_string();
    let ctr_value = CounterValue!(&ctr_expansion, inner_state).value_of();
    ExplodeText!(ctr_value)
  });

  DefMacro!("\\@arabic{Number}", sub[gullet, args, state] {
    let number = args.remove(0).to_number();
    let value = number.value_of();
    ExplodeText!(value.to_string())
  });
  DefMacro!("\\arabic{}", sub[gullet, args, inner_state] {
    unpack!(args => value);
    let ctr_expansion = Expand!(value, gullet, inner_state).to_string();
    let ctr_value = CounterValue!(&ctr_expansion, inner_state).value_of();
    ExplodeText!(ctr_value)
  });

  DefMacro!("\\@roman{Number}", sub[gullet, args, state] {
    unpack_to_number!(args => number);
    let value = number.value_of();
    ExplodeText!(radix::radix_roman(value))
  });
  DefMacro!("\\roman{}", sub[gullet, args, state] {
    unpack!(args => token);
    let ctr = Expand!(token, gullet).to_string();
    ExplodeText!(radix::radix_roman(CounterValue!(&ctr).value_of()))
  });
  DefMacro!("\\@Roman{Number}", sub[gullet, args, state] {
    unpack_to_number!(args => number);
    let value = number.value_of();
    ExplodeText!(radix::radix_up_roman(value))
  });
  DefMacro!("\\Roman{}", sub[gullet, args, state] {
    unpack!(args => token);
    let ctr = Expand!(token, gullet).to_string();
    ExplodeText!(radix::radix_up_roman(CounterValue!(&ctr).value_of()))
  });
  DefMacro!("\\@alph{Number}", sub[gullet, args, state] {
    unpack_to_number!(args => number);
    let value = number.value_of();
    ExplodeText!(radix::radix_alpha(value))
  });
  DefMacro!("\\alph{}", sub[gullet, args, state] {
    unpack!(args => token);
    let ctr = Expand!(token, gullet).to_string();
    ExplodeText!(radix::radix_alpha(CounterValue!(&ctr).value_of()))
  });
  DefMacro!("\\@Alph{Number}", sub[gullet, args, state] {
    unpack_to_number!(args => number);
    let value = number.value_of();
    ExplodeText!(radix::radix_up_alpha(value))
  });
  DefMacro!("\\Alph{}", sub[gullet, args, state] {
    unpack!(args => token);
    let ctr = Expand!(token, gullet).to_string();
    ExplodeText!(radix::radix_up_alpha(CounterValue!(&ctr).value_of()))
  });

  DefMacro!("\\@fnsymbol{Number}", sub[gullet, args, state] {
    unpack_to_number!(args => number);
    ExplodeText!(radix::radix_format_str(number.value_of(), FNSYMBOLS))
  });
  DefMacro!("\\fnsymbol{}", sub[gullet, args, state] {
    unpack!(args => token);
    let ctr = Expand!(token, gullet).to_string();
    ExplodeText!(radix::radix_format_str(CounterValue!(&ctr).value_of(), FNSYMBOLS))
  });
});
