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
