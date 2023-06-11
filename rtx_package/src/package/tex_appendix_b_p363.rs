use crate::package::*;

//======================================================================
// TeX Book, Appendix B. p. 363
LoadDefinitions!(state, {

  DefPrimitive!("\\raggedbottom", None);
  DefPrimitive!("\\normalbottom", None);

  // if the mark is not simple, we add it to the content of the note
  // otherwise, to the attribute.
  DefConstructor!("\\footnote{}{}",
    "^<ltx:note role='footnote' ?#mark(mark='#mark')()>?#prenote(#prenote )()#2</ltx:note>");
    // TODO:
    // mode         => "text", bounded => 1,
    // before_digest => sub { reenterTextMode(1); neutralizeFont(); },
    // after_digest  => sub {
    //   my ($stomach, $whatsit) = @_;
    //   my $mark   = $whatsit->getArg(1);
    //   my $change = 0;
    //   foreach my $token (Revert($mark)) {
    //     unless ($token->getCatcode == CC_LETTER || $token->getCatcode == CC_SPACE ||
    //       $token->getCatcode == CC_OTHER) {
    //       $change = 1; last; } }
    //   $whatsit->setProperty(($change ? "prenote' : "mark') => $mark);
    //   return; });

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
