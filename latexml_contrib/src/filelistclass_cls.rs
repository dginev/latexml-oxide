use latexml_package::prelude::*;

LoadDefinitions!({
  // Perl: filelistclass.cls.ltxml
  // InputDefinitions('filelistclass', type => 'cls', noltxml => 1, withoptions => 1);
  // Note: handleoptions is false here — the outer dispatch call handles options.
  // withoptions in Perl just means "pass options through", which Rust does differently.
  InputDefinitions!("filelistclass", noltxml => true,
    extension => Some(Cow::Borrowed("cls")));
});
