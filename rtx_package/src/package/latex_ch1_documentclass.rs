use crate::package::*;
lazy_static! {
  static ref OPTS_REGEX: Regex = Regex::new(r",\s*").unwrap();
}

LoadDefinitions!(state, {
  // Apparently LaTeX does NOT define \magnification,
  // and babel uses that to determine whether we're runing LaTeX!!!
  Let!("\\magnification", "\\@undefined");
  //**********************************************************************
  // Basic \documentclass & \documentstyle

  DefConditional!("\\if@compatibility", sub [gullet, (), state] {
    state.lookup_bool("2.09_COMPATIBILITY") });
  DefMacro!("\\@compatibilitytrue", "");
  DefMacro!("\\@compatibilityfalse", "");

  Let!("\\@currentlabel", "\\@empty");

  // Let's try just starting with this set (since we've loaded LaTeX)
  AssignValue!("inPreamble", true); // \begin{document} will clear this.

  DefConstructor!("\\documentclass OptionalSemiverbatim SkipSpaces Semiverbatim []",
                  "<?latexml class='#2' ?#1(options='#1')?>",
    after_digest => sub[stomach, whatsit, state] {
      let options: Option<&Digested> = whatsit.get_arg(1);
      let class_opts = match options {
        Some(opts) => OPTS_REGEX.split(&opts.to_string()).map(ToString::to_string).collect(),
        None => Vec::new(),
      };
      load_class(&(whatsit.get_arg(2).unwrap().to_string()),
                class_opts,
                Tokens!(T_CS!("\\AtBeginDocument"), T_CS!("\\warn@unusedclassoptions")),
                stomach,
                state)?;
  });

  AssignValue!("@unusedoptionlist", Stored::VecString(Vec::new()));
  DefPrimitive!("\\warn@unusedclassoptions", sub[stomach,_args,state] {
    if let Some(Stored::VecString(unused)) = state.lookup_value("@unusedoptionlist") {
      if !unused.is_empty() {
        Info!("unexpected", "options", stomach, state,
              "Unused global options: {}",unused.join(","));
        state.assign_value("@unusedoptionlist", Stored::VecString(Vec::new()), None);
      }
    }
  });

  // DefConstructor('\documentstyle OptionalSemiverbatim SkipSpaces Semiverbatim []',
  //   "<?latexml class='#2' ?#1(options='#1') oldstyle='true'?>",
  //   beforeDigest => sub {
  //     Info('unexpected', '\documentstyle', $_[0], "Entering LaTeX 2.09 Compatibility mode");
  //     AssignValue('2.09_COMPATIBILITY' => 1, 'global');
  //     onlyPreamble('\documentstyle'); },
  //   afterDigest => sub {
  //     my ($stomach, $whatsit) = @_;
  //     my $class   = ToString($whatsit->getArg(2));
  //     my $options = $whatsit->getArg(1);
  //     $options = [($options ? split(/,\s*/, ToString($options)) : ())];
  //     # Watch out; In principle, compatibility mode wants a .sty, not a .cls!!!
  //     # But, we'd prefer .cls, since we'll have better bindings.
  //     # And in fact, nobody's likely to write a binding for a .sty that wants to be a class
  // anyway.     # So, we'll just try for a .cls, punting to OmniBus if needed.
  //     # If we start wanting to read style files by default, we'll still need to handle this
  //     # specially, since class (or sty files pretending to be) cover so much more.
  //     LoadClass($class, options => $options, after => Tokens(T_CS('\compat@loadpackages')));
  //     return; });

  // DefPrimitiveI('\compat@loadpackages', undef, sub {
  //     my $name       = ToString(Expand(T_CS('\@currname')));
  //     my $type       = ToString(Expand(T_CS('\@currext')));
  //     my $hadmissing = 0;
  //     foreach my $option (@{ LookupValue('@unusedoptionlist') }) {
  //       if (FindFile($option, type => 'sty')) {
  //         RequirePackage($option); }
  //       else {
  //         $hadmissing = 1;
  //         Info('unexpected', $option, $_[0], "Unexpected option '$option' passed to
  // $name.$type"); } }     # Often, in compatibility mode, the options are used to load what are
  // effectively     # document classes for specific journals, etc that introduce a bunch of new
  // frontmatter!     # To try to recover from this, we'll go ahead & load the OmniBus class.
  //     if ($hadmissing && !LookupValue('OmniBus.cls_loaded')) {
  //       Info('note', 'OmniBus', $_[0], "Loading OmniBus class to attempt to cover missing
  // options");       LoadClass('OmniBus'); }
  //     AssignValue('@unusedoptionlist', []); });

  // sub onlyPreamble {
  //   my ($cs) = @_;
  //   Error('unexpected', $cs, $STATE->getStomach,
  //     "The current command '" . ToString($cs) . "' can only appear in the preamble")
  //     unless LookupValue("inPreamble");
  //   return; }
});
