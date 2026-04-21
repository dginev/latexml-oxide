use latexml_package::prelude::*;

LoadDefinitions!({
  DefMacro!("\\SetUnicodeOption{}", "");
  DefMacro!("\\unicodevirtual{}", "#1");
  DefMacro!("\\unicodecombine", "");
  // TODO: Perl has a complex \unichar closure that converts hex code to UTF character.
  // Stubbed for now.
  DefMacro!("\\unichar Expanded", "");
  DefMacro!(
    "\\DeclareUnicodeCharacterAsOptional{}{}{}",
    "\\DeclareUnicodeCharacter{#1}{#3}"
  );
  // TODO: Perl redefines \DeclareUnicodeCharacter with hex parsing closure.
  // Keeping the default behavior for now.
  DefMacro!("\\DeclareUnicodeOption[]{}", "");
  DefMacro!("\\LinkUnicodeOptionToPkg{}{}", "");
  DefMacro!("\\PreloadUnicodePage{}", "");
  DefMacro!("\\PrerenderUnicode{}", "");
});
