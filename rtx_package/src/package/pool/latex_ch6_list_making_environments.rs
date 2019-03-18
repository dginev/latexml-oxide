use crate::package::*;

LoadDefinitions!(state, {
  //======================================================================
  // C.6.2 List-Making environments
  //======================================================================
  Tag!("ltx:item",        auto_close => true, auto_open => true);
  Tag!("ltx:inline-item", auto_close => true, auto_open => true);

  // These are for the (not quite legit) case where \item appears outside
  // of an itemize, enumerate, etc, environment.
  // DefCon('\item[]',
  //   "<ltx:item>?&defined(#1)(<ltx:tags><ltx:tag>#1</ltx:tag></ltx:tags>)");
  // DefCon('\subitem[]',
  //   "<ltx:item>?&defined(#1)(<ltx:tags><ltx:tag>#1</ltx:tag></ltx:tags>)");
  // DefCon('\subsubitem[]',
  //   "<ltx:item>?&defined(#1)(<ltx:tags><ltx:tag>#1</ltx:tag></ltx:tags>)");

  // Or maybe best just to do \par ?
  DefMacro!("\\item[]",       "\\par");
  DefMacro!("\\subitem[]",    "\\par");
  DefMacro!("\\subsubitem[]", "\\par");

  AssignValue!("@itemlevel" => 0, Some(Scope::Global));
  AssignValue!("enumlevel"  => 0, Some(Scope::Global));
  AssignValue!("@desclevel" => 0, Some(Scope::Global));
  // protection against lower-level code...
  DefConditional!("\\if@noitemarg");
  DefMacro!("\\@item",      "\\item");   // Hopefully no circles...
  DefMacro!("\\@itemlabel", "");         // Maybe needs to be same as \item will be using?

  // These counters are only used for id's of the various itemize, enumerate, etc elements
  NewCounter!("@itemizei",   "section",      idprefix => "I");
  NewCounter!("@itemizeii",  "@itemizei",   idprefix => "I");
  NewCounter!("@itemizeiii", "@itemizeii",  idprefix => "I");
  NewCounter!("@itemizeiv",  "@itemizeiii", idprefix => "I");
  NewCounter!("@itemizev",   "@itemizeiv",  idprefix => "I");
  NewCounter!("@itemizevi",  "@itemizev",   idprefix => "I");

  // id, but NO refnum (et.al) attributes on itemize \\item ...
  // unless the optional tag argument was given!
  // We"ll make the <ltx:tag> from either the optional arg, or from \\labelitemi..
  DefMacro!("\\itemize@item", "\\par\\itemize@item@");
  DefConstructor!("\\itemize@item@ OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[stomach, args, state] {
      unpack_to_string!(args=>tag);
      ref_step_item_counter(&tag, stomach, state) });
  DefConstructor!("\\inline@itemize@item OptionalUndigested",
    "<ltx:inline-item xml:id='#id'>#tags",
    properties => sub[stomach, args, state] {
      unpack_to_string!(args=>tag);
      ref_step_item_counter(&tag, stomach, state) });

  DefMacro!("\\enumerate@item", "\\par\\enumerate@item@");
  DefConstructor!("\\enumerate@item@ OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[stomach, args, state] {
      unpack_to_string!(args=>tag);
      ref_step_item_counter(&tag, stomach, state) });
  DefConstructor!("\\inline@enumerate@item OptionalUndigested",
    "<ltx:inline-item xml:id='#id'>#tags",
    properties => sub[stomach, args, state] {
      unpack_to_string!(args=>tag);
      ref_step_item_counter(&tag, stomach, state) });

  DefMacro!("\\description@item", "\\par\\description@item@");
  DefConstructor!("\\description@item@ OptionalUndigested",
    "<ltx:item xml:id='#id' itemsep='#itemsep'>#tags",
    properties => sub[stomach, args, state] {
      unpack_to_string!(args=>tag);
      ref_step_item_counter(&tag, stomach, state) });
  DefConstructor!("\\inline@description@item OptionalUndigested",
    "<ltx:inline-item xml:id='#id'>#tags",
    properties => sub[stomach, args, state] {
      unpack_to_string!(args=>tag);
      ref_step_item_counter(&tag, stomach, state) });

  DefEnvironment!("{itemize}",
    "<ltx:itemize xml:id='#id'>#body</ltx:itemize>",
    properties => { BeginItemize!("itemize", "@item") },
    before_digest_end => { Digest!("\\par")?; },
    locked => true,
    mode => "text"
  );
  DefEnvironment!("{enumerate}",
    "<ltx:enumerate  xml:id='#id'>#body</ltx:enumerate>",
    properties => { BeginItemize!("enumerate", "enum") },
    before_digest_end => { Digest!("\\par")?; },
    locked => true,
    mode => "text"
  );
  DefEnvironment!("{description}",
    "<ltx:description  xml:id='#id'>#body</ltx:description>",
    before_digest => { Let!("\\makelabel", "\\descriptionlabel"); },
    properties => { BeginItemize!("description", "@desc") },
    before_digest_end => { Digest!("\\par")?; },
    locked => true,
    mode => "text"
  );

  DefMacro!("\\makelabel{}", "#1");
  //----------------------------------------------------------------------
  // Basic itemize bits
  // Fake counter for itemize to give id's to ltx:item.
  NewCounter!("@itemi",   "", idwithin => "@itemizei", idprefix => "i");
  NewCounter!("@itemii",  "", idwithin => "@itemi",    idprefix => "i");
  NewCounter!("@itemiii", "", idwithin => "@itemii",   idprefix => "i");
  NewCounter!("@itemiv",  "", idwithin => "@itemiii",  idprefix => "i");
  NewCounter!("@itemv",   "", idwithin => "@itemiv",   idprefix => "i");
  NewCounter!("@itemvi",  "", idwithin => "@itemv",    idprefix => "i");
  // These are empty to make the "refnum" go away.
  DefMacro!("\\the@itemi",   "");
  DefMacro!("\\the@itemii",  "");
  DefMacro!("\\the@itemiii", "");
  DefMacro!("\\the@itemiv",  "");
  DefMacro!("\\the@itemv",   "");
  DefMacro!("\\the@itemvi",  "");

  // Formatted item tags.
  // Really should be in the class file, but already was here.
  DefMacro!("\\labelitemi",   "\\textbullet");
  DefMacro!("\\labelitemii",  "\\normalfont\\bfseries \\textendash");
  DefMacro!("\\labelitemiii", "\\textasteriskcentered");
  DefMacro!("\\labelitemiv",  "\\textperiodcentered");

  // Make the fake counters point to the real labels
  DefMacro!("\\label@itemi",   "\\labelitemi");
  DefMacro!("\\label@itemii",  "\\labelitemii");
  DefMacro!("\\label@itemiii", "\\labelitemiii");
  DefMacro!("\\label@itemiv",  "\\labelitemiv");

  // These hookup latexml"s tagging to normal latex"s \labelitemi...
  DefMacro!("\\fnum@@itemi",   "{\\makelabel{\\label@itemi}}");
  DefMacro!("\\fnum@@itemii",  "{\\makelabel{\\label@itemii}}");
  DefMacro!("\\fnum@@itemiii", "{\\makelabel{\\label@itemiii}}");
  DefMacro!("\\fnum@@itemiv",  "{\\makelabel{\\label@itemiv}}");

  // These define the typerefnum form, for out-of-context \ref"s
  // Better would language sensitive!
  DefMacro!("\\lx@poormans@ordinal{}", sub[gullet, args, state] {
    unpack_to_token!(args => ctr);
    let pm_ordinal_suffices = ["th", "st", "nd", "rd", "th", "th", "th", "th", "th", "th"];
    let mut ctr_str      = CounterValue!(ctr.get_string()).value_of().to_string();
    let last_char = ctr_str.chars().last().unwrap_or('.');
    if last_char.is_ascii_digit() {
      ctr_str.push_str(pm_ordinal_suffices[last_char.to_digit(10).unwrap() as usize]);
    }
    T_OTHER!(ctr_str)
  });
  DefMacro!("\\itemtyperefname", "item");
  DefMacro!("\\itemcontext",     "\\space in \\@listcontext");
  DefMacro!("\\itemcontext",     "");
  // Probably would help to give a bit more context for the ii & higher?
  DefMacro!("\\typerefnum@@itemi", "\\lx@poormans@ordinal{@itemi} \\itemtyperefname \\itemcontext");
  DefMacro!("\\typerefnum@@itemii", "\\lx@poormans@ordinal{@itemii} \\itemtyperefname \\itemcontext");
  DefMacro!("\\typerefnum@@itemiii", "\\lx@poormans@ordinal{@itemiii} \\itemtyperefname \\itemcontext");
  DefMacro!("\\typerefnum@@itemiv", "\\lx@poormans@ordinal{@itemiv} \\itemtyperefname \\itemcontext");
  //----------------------------------------------------------------------
  // Basic enumeration bits

  // Class file should have
  //  NewCounter for enumi,...,
  //  define \labelenumi,... and probably \p@enumii...

  // How the refnums look... (probably should be in class file, but already here)
  DefMacro!("\\p@enumi",  "");
  DefMacro!("\\p@enumii",  "\\theenumi");
  DefMacro!("\\p@enumiii", "\\theenumi(\\theenumii)");
  DefMacro!("\\p@enumiv",  "\\p@enumii\\theenumiii");

  // Formatting of item tags (probably should be in the class file, but already here)
  DefMacro!("\\labelenumi",   "\\theenumi.");
  DefMacro!("\\labelenumii",  "(\\theenumii)");
  DefMacro!("\\labelenumiii", "\\theenumiii.");
  DefMacro!("\\labelenumiv",  "\\theenumiv.");

  // These hookup latexml"s tagging to normal latex"s \labelenummi...
  DefMacro!("\\fnum@enumi",   "{\\makelabel{\\labelenumi}}");
  DefMacro!("\\fnum@enumii",  "{\\makelabel{\\labelenumii}}");
  DefMacro!("\\fnum@enumiii", "{\\makelabel{\\labelenumiii}}");
  DefMacro!("\\fnum@enumiv",  "{\\makelabel{\\labelenumiv}}");

  // These define the typerefnum form, for out-of-context \ref's
  DefMacro!("\\enumtyperefname",    "item");
  DefMacro!("\\typerefnum@enumi",   "\\enumtyperefname~\\p@enumi\\theenumi \\itemcontext");
  DefMacro!("\\typerefnum@enumii",  "\\enumtyperefname~\\p@enumii\\theenumii \\itemcontext");
  DefMacro!("\\typerefnum@enumiii", "\\enumtyperefname~\\p@enumiii\\theenumiii \\itemcontext");
  DefMacro!("\\typerefnum@enumiv",  "\\enumtyperefname~\\p@enumiv\\theenumiv \\itemcontext");
  //DefMacro!("\\typerefnum@enumi",   None, "\enumtyperefname~\p@enumi\labelenumi \itemcontext");
  //DefMacro!("\\typerefnum@enumii",  None, "\enumtyperefname~\p@enumii\labelenumii \itemcontext");
  //DefMacro!("\\typerefnum@enumiii", None, "\enumtyperefname~\p@enumiii\labelenumiii \itemcontext");
  //DefMacro!("\\typerefnum@enumiv",  None, "\enumtyperefname~\p@enumiv\labelenumiv \itemcontext");

  //----------------------------------------------------------------------
  // Basic description list bits
  // Fake counter for itemize to give id"s to ltx:item.
  NewCounter!("@desci",   "", idwithin => "@itemizei", idprefix => "i");
  NewCounter!("@descii",  "", idwithin => "@desci",    idprefix => "i");
  NewCounter!("@desciii", "", idwithin => "@descii",   idprefix => "i");
  NewCounter!("@desciv",  "", idwithin => "@desciii",  idprefix => "i");
  NewCounter!("@descv",   "", idwithin => "@desciv",   idprefix => "i");
  NewCounter!("@descvi",  "", idwithin => "@descv",    idprefix => "i");
  // No refnum"s here, either
  DefMacro!("\\the@desci",   "");
  DefMacro!("\\the@descii",  "");
  DefMacro!("\\the@desciii", "");
  DefMacro!("\\the@desciv",  "");
  DefMacro!("\\the@descv",   "");
  DefMacro!("\\the@descvi",  "");
  // These hookup latexml"s numbering to normal latex"s
  // Umm.... but they"re not normally used, since \item usually gets an argument!
  DefMacro!("\\descriptionlabel{}", "\\normalfont\\bfseries #1");
  DefMacro!("\\fnum@@desci",   "{\\descriptionlabel{}}");
  DefMacro!("\\fnum@@descii",  "{\\descriptionlabel{}}");
  DefMacro!("\\fnum@@desciii", "{\\descriptionlabel{}}");
  DefMacro!("\\fnum@@desciv",  "{\\descriptionlabel{}}");

  DefMacro!("\\desctyperefname", "item");

  // Blech
  for lvl in &["@itemi", "@itemii", "@itemiii", "@itemiv", "@itemv", "@itemvi", "enumi", "enumii",
              "enumiii", "enumiv", "@desci", "@descii", "@desciii", "@desciv", "@descv", "@descvi"] {
    DefMacroI!(T_CS!(s!("\\{}name", lvl)), None, T_CS!("\\itemtyperefname"));
  }
});
