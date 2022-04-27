use rtx_core::state::State;

pub fn start_appendices(kind: &str, state: &mut State) { begin_appendices(kind, state) }

// Class files should define \@appendix to call this as startAppendices('section') or chapter...
// counter is also the element name!

pub fn begin_appendices(counter: &str, state: &mut State) {
  unimplemented!();
  // Let('\lx@save@theappendex',    '\the' . $counter,         'global');
  // Let('\lx@save@theappendex@ID', '\the' . $counter . '@ID', 'global');
  // Let('\lx@save@appendix',       T_CS('\\' . $counter),     'global');
  // Let('\lx@save@@appendix',      T_CS('\@appendix'),        'global');
  // AssignMapping('BACKMATTER_ELEMENT', 'ltx:appendix' => 'ltx:' . $counter);
  // if (LookupDefinition(T_CS('\c@chapter'))    # Has \chapter defined
  //   && ($counter ne 'chapter')) {             # And appendices are below the chapter level.
  //   NewCounter($counter, 'chapter', idprefix => 'A');
  //   DefMacroI('\the' . $counter, undef, '\thechapter.\Alph{' . $counter . '}', scope => 'global'); }
  // else {
  //   NewCounter($counter, 'document', idprefix => 'A');
  //   DefMacroI('\the' . $counter, undef, '\Alph{' . $counter . '}', scope => 'global'); }
  // AssignMapping('counter_for_type', appendix => $counter);
  // Let(T_CS('\\' . $counter), T_CS('\@@appendix'), 'global');
  // Let(T_CS('\@appendix'),    T_CS('\relax'),      'global');
}

pub fn end_appendices(state: &mut State) {
  unimplemented!();
  // if (my $counter = LookupMapping('BACKMATTER_ELEMENT', 'ltx:appendix')) {
  //   $counter =~ s/^ltx://;
  //   Let('\the' . $counter,         '\lx@save@theappendex',    'global');
  //   Let('\the' . $counter . '@ID', '\lx@save@theappendex@ID', 'global');
  //   Let(T_CS('\\' . $counter),     '\lx@save@appendix',       'global');
  //   Let(T_CS('\@appendix'),        '\lx@save@@appendix',      'global'); }
}
