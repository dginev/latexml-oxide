//! resphilosophica.cls (Res Philosophica): the class is loaded raw; its `\thanks` text is printed at the end under
//! the label "Acknowledgments" (`\enddoc@text`, resphilosophica.cls:436-449: `\textbf{Acknowledgments\quad}
//! \@setthanks`), which the frontmatter's thanks notes carry as their name instead of LaTeXML's "Thanks:" (user ruling
//! 2026-10-03: a class label before content we keep is content). Witness resphilosophica/rpsample. Guard
//! `perfect_kernel_batch61::resphilosophica_thanks_are_acknowledgments`.
use latexml_package::prelude::*;

LoadDefinitions!({
  InputDefinitions!("resphilosophica", noltxml => true, extension => Some(Cow::Borrowed("cls")));
  RawTeX!(
    r"\def\lx@pubnote@thanks@name{Acknowledgments~}\def\lx@contact@thanks@name{Acknowledgments~}"
  );
});
