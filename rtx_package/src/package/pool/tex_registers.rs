use crate::package::*;

//======================================================================
// Registers & Parameters
// See Chapter 24, Summary of Vertical Mode
// Define a whole mess of useless registers here ...
// Values are from Appendix B, pp. 348-349 (for whatever its worth)
//======================================================================
LoadDefinitions!(state, {
  //======================================================================
  // Integer registers; TeXBook p. 272-273

  // DefRegister('\tracingmacros', Number(0),
  //   getter => sub { Number(LookupValue('TRACINGMACROS') || 0); },
  //   setter => sub { AssignValue(TRACINGMACROS => $_[0]->valueOf); });
  // DefRegister('\tracingcommands', Number(0),
  //   getter => sub { Number(LookupValue('TRACINGCOMMANDS')); },
  //   setter => sub { AssignValue(TRACINGCOMMANDS => $_[0]->valueOf); });

  for (key, val) in [
    ("pretolerance", 100),
    ("tolerance", 200),
    ("hbadness", 1000),
    ("vbadness", 1000),
    ("linepenalty", 10),
    ("hyphenpenalty", 50),
    ("exhyphenpenalty", 50),
    ("binoppenalty", 700),
    ("relpenalty", 500),
    ("clubpenalty", 150),
    ("widowpenalty", 150),
    ("displaywidowpenalty", 50),
    ("brokenpenalty", 100),
    ("predisplaypenalty", 10000),
    ("postdisplaypenalty", 0),
    ("interlinepenalty", 0),
    ("floatingpenalty", 0),
    ("outputpenalty", 0),
    ("doublehyphendemerits", 10000),
    ("finalhyphendemerits", 5000),
    ("adjdemerits", 10000),
    ("looseness", 0),
    ("pausing", 0),
    ("holdinginserts", 0),
    ("tracingonline", 0),
    ("tracingstats", 0),
    ("tracingparagraphs", 0),
    ("tracingpages", 0),
    ("tracingoutput", 0),
    ("tracinglostchars", 1),
    ("tracingrestores", 0),
    ("language", 0),
    ("uchyph", 1),
    ("lefthyphenmin", 0),
    ("righthyphenmin", 0),
    ("globaldefs", 0),
    ("defaulthyphenchar", 45),
    ("defaultskewchar", -1),
    ("escapechar", 92),
    ("endlinechar", 0),
    ("newlinechar", -1),
    ("maxdeadcycles", 0),
    ("hangafter", 0),
    ("fam", -1),
    ("mag", 1000),
    ("magnification", 1000),
    ("delimiterfactor", 0),
    ("time", 0),
    ("day", 0),
    ("month", 0),
    ("year", 0),
    ("showboxbreadth", 5),
    ("showboxdepth", 3),
    ("errorcontextlines", 5),
  ]
  .iter()
  {
    DefRegister!(&s!("\\{}", key), Number!(*val));
  }

  // # Most of these are ignored, but...
  // DefMacro('\tracingall',
  //   '\tracingonline=1 \tracingcommands=2 \tracingstats=2'
  // . ' \tracingpages=1 \tracingoutput=1 \tracinglostchars=1'
  // . ' \tracingmacros=2 \tracingparagraphs=1 \tracingrestores=1'
  //     . ' \showboxbreadth=\maxdimen \showboxdepth=\maxdimen \errorstopmode');

  // # This may mess up Daemon state?
  // { my ($sec, $min, $hour, $mday, $mon, $year) = localtime();
  //   AssignValue('\day'   => Number($mday),             'global');
  //   AssignValue('\month' => Number($mon + 1),          'global');
  //   AssignValue('\year'  => Number(1900 + $year),      'global');
  //   AssignValue('\time'  => Number(60 * $hour + $min), 'global'); }

  // our @MonthNames = (qw( January February March April May June
  //     July August September October November December));

  // # Return a string for today's date.
  // sub today {
  //   return $MonthNames[LookupValue('\month')->valueOf - 1]
  //     . " " . LookupValue('\day')->valueOf
  //     . ', ' . LookupValue('\year')->valueOf; }

  // # Read-only Integer registers
  // {
  //   my %ro_iparms = (lastpenalty => 0, inputlineno => 0, badness => 0);
  //   foreach my $p (keys %ro_iparms) {
  //     DefRegister("\\$p", Number($ro_iparms{$p}), readonly => 1); }
  // }

  // # Special integer registers (?)
  // # <special integer> = \spacefactor | \prevgraf | \deadcycles | \insertpenalties
  // {
  //   my %sp_iparms = (spacefactor => 0, prevgraf => 0, deadcycles => 0, insertpenalties => 0);
  //   foreach my $p (keys %sp_iparms) {
  //     DefRegister("\\$p", Number($sp_iparms{$p})); }
  // }
});
