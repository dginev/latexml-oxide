use crate::package::*;
LoadDefinitions!(state, {
  //======================================================================
  // C.8.4 Numbering
  //======================================================================
  // For LaTeX documents, We want id's on para, as well as sectional units.
  // However, para get created implicitly on Document construction, rather than
  // explicitly during digestion (via a whatsit), we can't use the usual LaTeX counter mechanism.
  Tag!("ltx:para", after_open => tagsub!(document, node, state, {
    generate_id(document, node, "p", state)?;
  }));

  // DefPrimitive('\newcounter{}[]', sub {
  //     NewCounter(ToString(Expand($_[1])), $_[2] && ToString(Expand($_[2])));
  //     return; });
  // DefPrimitive('\setcounter{}{Number}', sub { SetCounter(ToString(Expand($_[1])), $_[2]); });
  // DefPrimitive('\addtocounter{}{Number}', sub { AddToCounter(ToString(Expand($_[1])), $_[2]); });
  // DefPrimitive('\stepcounter{}',    sub { StepCounter(ToString(Expand($_[1])));    return; });
  // DefPrimitive('\refstepcounter{}', sub { RefStepCounter(ToString(Expand($_[1]))); return; });

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

  // DefMacro('\@arabic{Number}', sub {
  //     ExplodeText(ToString($_[1]->valueOf)); });
  DefMacro!("\\arabic{}", sub[gullet, args, inner_state] {
    unpack!(args => value);
    let ctr_expansion = Expand!(value, gullet, inner_state).to_string();
    let ctr_value = CounterValue!(&ctr_expansion, inner_state).value_of();
    ExplodeText!(ctr_value)
  });

  // DefMacro('\@roman{Number}', sub {
  //     ExplodeText(radix_roman(ToString($_[1]->valueOf))); });
  // DefMacro('\roman{}', sub {
  //     ExplodeText(radix_roman(CounterValue(ToString(Expand($_[1])))->valueOf)); });
  // DefMacro('\@Roman{Number}', sub {
  //     ExplodeText(radix_Roman(ToString($_[1]->valueOf))); });
  // DefMacro('\Roman{}', sub {
  //     ExplodeText(radix_Roman(CounterValue(ToString(Expand($_[1])))->valueOf)); });
  // DefMacro('\@alph{Number}', sub {
  //     ExplodeText(radix_alpha($_[1]->valueOf)); });
  // DefMacro('\alph{}', sub {
  //     ExplodeText(radix_alpha(CounterValue(ToString(Expand($_[1])))->valueOf)); });
  // DefMacro('\@Alph{Number}', sub {
  //     ExplodeText(radix_Alpha($_[1]->valueOf)); });
  // DefMacro('\Alph{}', sub {
  //     ExplodeText(radix_Alpha(CounterValue(ToString(Expand($_[1])))->valueOf)); });

  // our @fnsymbols = ("*", "\x{2020}", "\x{2021}", UTF(0xA7), UTF(0xB6),
  //   "\x{2225}", "**", "\x{2020}\x{2020}", "\x{2021}\x{2021}");
  // DefMacro('\@fnsymbol{Number}', sub {
  //     ExplodeText(radix_format($_[1]->valueOf, @fnsymbols)); });
  // DefMacro('\fnsymbol{}', sub {
  //     ExplodeText(radix_format(CounterValue(ToString(Expand($_[1])))->valueOf, @fnsymbols)); });

});