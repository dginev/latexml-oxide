#[macro_export]
macro_rules! LoadDefinitions {
  ($outer_state:ident, $body:block) => {
    LoadDefinitions!(outer_stomach, $outer_state, $body);
  };
  ($outer_stomach:ident, $outer_state:ident, $body:block) => {
    pub fn load_definitions($outer_stomach: &mut Stomach, $outer_state: &mut State) -> Result<()> {
      BindState!($outer_stomach, $outer_state);
      {
        $body
      }
      Ok(())
    }
  };
}

//=================================================
// Variable capture games -- capture a given $state
//    into a set of convenience macros for brief syntax
//
//=================================================

#[macro_export]
macro_rules! BindState {
  ($state_arg:ident) => {
    #[allow(unused_macros)]
    macro_rules! outer_state {
      () => {
        $state_arg
      };
    }
  };
  ($outer_stomach:ident, $state_arg:ident) => {
    #[allow(unused_macros)]
    macro_rules! outer_stomach {
      () => {
        $outer_stomach
      };
    }
    #[allow(unused_macros)]
    macro_rules! outer_state {
      () => {
        $state_arg
      };
    }
  };
}

#[macro_export]
macro_rules! BindInnerState {
  ($inner_state:ident) => {
    #[allow(unused_macros)]
    macro_rules! inner_state {
      () => {
        $inner_state
      };
    }
    start_state_frame!();
  };
  ($inner_stomach:ident, $inner_state:ident) => {
    macro_rules! inner_stomach {
      () => {
        $inner_stomach
      };
    }
    #[allow(unused_macros)]
    macro_rules! inner_state {
      () => {
        $inner_state
      };
    }
    start_state_frame!();
  };
}

#[macro_export]
macro_rules! start_state_frame {
  () => {{
    #[derive(StartStateFrame)]
    struct _SFrame;
  }};
}
#[macro_export]
macro_rules! end_state_frame {
  () => {{
    #[derive(EndStateFrame)]
    struct _EFrame;
  }};
}

#[macro_export]
macro_rules! WithInnerState {
  ($body: block, $inner_state:ident) => {{
    BindInnerState!($inner_state);
    let macro_out = $body;
    end_state_frame!();
    macro_out
  }};
  ($body: block, $stomach:ident, $inner_state:ident) => {{
    BindInnerState!($stomach, $inner_state);
    let macro_out = $body;
    end_state_frame!();
    macro_out
  }};
}

#[macro_export]
macro_rules! bind_state {
  ($st:ident) => {
    let $st: &State = {
      #[derive(BoundState)]
      struct _Bound;
      state!()
    };
  };
  ($stmch:ident, $st:ident) => {
    let $stmch: &Stomach = {
      #[derive(BoundState)]
      struct _Bound;
      stomach!()
    };
    let $st: &State = {
      #[derive(BoundState)]
      struct _Bound;
      state!()
    };
  };
}

#[macro_export]
macro_rules! bind_state_mut {
  ($st:ident) => {
    let $st: &mut State = {
      #[derive(BoundState)]
      struct _Bound;
      state!()
    };
  };
  ($stmch:ident, $st:ident) => {
    let $stmch: &mut Stomach = {
      #[derive(BoundState)]
      struct _Bound;
      stomach!()
    };
    let $st: &mut State = {
      #[derive(BoundState)]
      struct _Bound;
      state!()
    };
  };
}

//======================================================================
// Defining new Control-sequence Parameter types.
//======================================================================
#[macro_export]
macro_rules! DefParameterTypeWO {
  ($name:expr, $param:expr) => {
    bind_state_mut!(st);
    st.assign_mapping("PARAMETER_TYPES", $name, Some(Stored::Parameter(Rc::new($param))))
  };
  ($name:expr, $param:expr, $state_arg:ident) => {
    $state_arg.assign_mapping("PARAMETER_TYPES", $name, Some(Stored::Parameter(Rc::new($param))))
  };
}

#[macro_export]
macro_rules! LoadPool {
  ($name:expr) => {{
    bind_state_mut!(stmch, st);
    LoadPool!($name, stmch, st)
  }};
  ($name:expr, $stomach_arg:ident, $state_arg:ident) => {{
    input_definitions(
      $name,
      InputDefinitionOptions {
        extension: Some("pool"),
        ..InputDefinitionOptions::default()
      },
      $stomach_arg,
      $state_arg,
    )?
  }};
}

#[macro_export]
macro_rules! InputDefinitions {
  ($name:expr) => {{
    bind_state_mut!(st);
    input_definitions($name, InputDefinitionOptions::default(), st)?
  }};
  ($name: expr, $($key:ident => $value:expr)*) => {
    bind_state_mut!(stmch, st);
    input_definitions($name, NewDefault!(InputDefinitionOptions, $($key => $value),*), stmch, st)?
  }
}

/// Loader shorthand for pool dependencies
#[macro_export]
macro_rules! InnerPool {
  ($name:ident) => {{
    bind_state_mut!(stmch, st);
    InnerPool!($name, stmch, st)
  }};
  ($name:ident, $state_arg:ident) => {
    InnerPool!($name, stomach!(), $state_arg)
  };
  ($name:ident, $stomach_arg:ident, $state_arg:ident) => {{
    pool::$name::load_definitions($stomach_arg, $state_arg)?
  }};
}

#[macro_export]
macro_rules! RequirePackage {
  ($package:expr, $options:expr) => {{
    bind_state_mut!(st);
    RequirePackage!($package, $options, st)
  }};
  ($package:expr, $options:expr, $state_arg:ident) => {
    require_package($package, $options, $state_arg)
  };
}
macro_rules! LoadClass {
  ($class:expr, $options:expr, $after:expr) => {{
    bind_state_mut!(st);
    load_class($class, $options, $after, st)
  }};
  ($class:expr, $options:expr, $after:expr, $state_arg:ident) => {
    load_class($class, $options, $after, $state_arg)
  };
}

#[macro_export]
macro_rules! DeclareFontMap {
  ($name:expr, $map:expr, $family:expr, $state_arg: ident) => {{
    let mapname = s!("{}_{}_fontmap", $name, $family);
    let map: Vec<Option<char>> = $map;
    $state_arg.assign_value(&mapname, map, Some(Scope::Global));
  }};
  ($name:expr, $map:expr, $state_arg: ident) => {{
    let mapname = s!("{}_fontmap", $name);
    let map: Vec<Option<char>> = $map;
    $state_arg.assign_value(&mapname, map, Some(Scope::Global));
  }};
  ($name:expr, $map:expr, $family:expr) => {{
    bind_state_mut!(st);
    DeclareFontMap!($name, $map, $family, st)
  }};
  ($name:expr, $map:expr) => {{
    bind_state_mut!(st);
    DeclareFontMap!($name, $map, st)
  }};
}

#[macro_export]
macro_rules! LoadFontMap {
  ($encoding: expr) => {{
    bind_state!(st);
    st.load_font_map($encoding)
  }};
}

// ======================================================================
// Color
#[macro_export]
macro_rules! LookupColor {
  ($name:expr) => {{
    bind_state!(st);
    if let Some(color) = LookupValue!(&s!("color_{}", $name)) {
      color.to_string()
    } else {
      Error!("undefined", $name, s!("color '{}' is undefined...", $name));
      "Black";
    }
  }};
}

// sub DefColor {
//   my ($name, $color, $scope) = @_;
//   return unless ref $color;
//   my ($model, @spec) = @$color;
//   $scope = 'global' if $STATE->lookupDefinition(T_CS('\ifglobalcolors')) && IfCondition(T_CS('\ifglobalcolors'));
//   AssignValue('color_' . $name => $color, $scope);
//   # We could store these pieces separately,or in a list for above,
//   # so that extract could use them more reasonably?
//   # This is perhaps too xcolor specific?
//   DefMacroI('\\\\color@' . $name, undef,
//     '\relax\relax{' . join(' ', $model, @spec) . '}{' . $model . '}{' . join(',', @spec) . '}',
//     scope => $scope);
//   return; }

// # Need 3 things for Derived Models:
// #   derivedfrom  : the core model that this model is "derived from"
// #   convertto    : code to convert to the (a) core model
// #   convertfrom  : code to convert from the core model
// sub DefColorModel {
//   my ($model, $coremodel, $tocore, $fromcore) = @_;
//   AssignValue('derived_color_model_' . $model => [$coremodel, $tocore, $fromcore], 'global');
//   return; }

macro_rules! DefRewrite {
  ($($input:tt)+) => {{
    let rewrite_options = defi_opts!(@munch ($($input)*) -> {RewriteOptions,});
    bind_state!(st);
    PushValue!("DOCUMENT_REWRITE_RULES",
      Rewrite::new("text", rewrite_options));
  }};
}

macro_rules! DefMathRewrite {
  ($($input:tt)+) => {{
    let rewrite_options = defi_opts!(@munch ($($input)*) -> {RewriteOptions,});
    bind_state!(st);
    PushValue!("DOCUMENT_REWRITE_RULES",
      Rewrite::new("math", rewrite_options));
  }};
}

// #======================================================================
// # Defining "Ligatures" rules that act on the DOM
// # These are actually a sort of rewrite that is applied while the doom
// # is being constructed, in particular as each node is closed.

// my $ligature_options = {    # [CONSTANT]
//   fontTest => 1 };

// sub DefLigature {
//   my ($regexp, $replacement, %options) = @_;
//   CheckOptions("DefLigature", $ligature_options, %options);
//   UnshiftValue('TEXT_LIGATURES',
//     { regexp => $regexp,
//       code => sub { $_[0] =~ s/$regexp/$replacement/g; $_[0]; },
//       %options });
//   return; }

// my $old_math_ligature_options = {};                                                     # [CONSTANT]
// my $math_ligature_options     = { matcher => 1, role => 1, name => 1, meaning => 1 };   # [CONSTANT]

// sub DefMathLigature {
//   if ((scalar(@_) % 2) == 1) {                                                          # Old style!
//     my ($matcher, %options) = @_;
//     Info('deprecated', 'ligature', undef, "Old style arguments to DefMathLigature; please update");
//     CheckOptions("DefMathLigature", $old_math_ligature_options, %options);
//     UnshiftValue('MATH_LIGATURES', { old_style => 1, matcher => $matcher }); }          # Install it...
//   else {                                                                                # new style!
//     my (%options) = @_;
//     my $matcher = $options{matcher};
//     delete $options{matcher};
//     my ($pattern) = grep { !$$math_ligature_options{$_} } keys %options;
//     my $replacement = $pattern && $options{$pattern};
//     delete $options{$pattern} if $replacement;
//     CheckOptions("DefMathLigature", $math_ligature_options, %options);    # Check remaining options
//     if ($matcher && $pattern) {
//       Error('misdefined', 'MathLigature', undef,
//         "DefMathLigature only gets one of matcher or pattern=>replacement keywords");
//       return; }
//     elsif ($pattern) {
//       my @chars    = reverse(split(//, $pattern));
//       my $ntomatch = scalar(@chars);
//       my %attr     = %options;
//       $matcher = sub {
//         my ($document, $node) = @_;
//         foreach my $char (@chars) {
//           return unless
//             ($node
//             && ($document->getModel->getNodeQName($node) eq 'ltx:XMTok')
//             && (($node->textContent || '') eq $char));
//           $node = $node->previousSibling; }
//         return ($ntomatch, $replacement, %attr); }; }
//     elsif (!$matcher) {
//       Error('misdefined', 'MathLigature', undef,
//         "DefMathLigature missing matcher or pattern=>replacement keywords");
//       return; }
//     UnshiftValue('MATH_LIGATURES', { matcher => $matcher }); }    # Install it...
//   return; }

#[macro_export]
macro_rules! DefConditional(
  // test is always a rust closure
  ($proto:literal, sub [$gullet:ident, $args:ident, $inner_state:ident] $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConditionalOptions,});
    let (cs, paramlist) = parse_prototype!($proto);
    let test : ConditionalClosure = Rc::new(move |$gullet, $args, $inner_state| { WithInnerState!($body, $inner_state).into_bool_result() });
    defi_conditional!(cs, paramlist, Some(test), options);
  }};
  ($proto:literal, $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConditionalOptions,});
    let (cs, paramlist) = parse_prototype!($proto);
    let test : ConditionalClosure = Rc::new(move |gullet, args, inner_state| { WithInnerState!($body, inner_state).into_bool_result() });
    defi_conditional!(cs, paramlist, Some(test), options);
  }};
  ($proto:literal $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConditionalOptions,});
    let (cs, paramlist) = parse_prototype!($proto);
    defi_conditional!(cs, paramlist, None, options);
  }};
  // internal, just declare CS
  ($cs:ident, None  $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConditionalOptions,});
    defi_conditional!($cs, None, None, options);
  }}
);

#[macro_export]
macro_rules! defi_conditional {
  ($cs:expr, $paramlist:expr, $test:expr, $options:expr) => {{
    bind_state_mut!(st);
    defi_conditional!($cs, $paramlist, $test, $options, st);
  }};
  ($cs:expr, $paramlist:expr, $test:expr, $options:expr, $state_arg:ident) => {{
    def_conditional($cs, $paramlist, $test, $options, $state_arg);
  }};
}

// sub IfCondition {
//   my ($if, @args) = @_;
//   my $gullet = $STATE->getStomach->getGullet;
//   $if = coerceCS($if);
//   my ($defn, $test);
//   if (($defn = $STATE->lookupDefinition($if))
//     && (($$defn{conditional_type} || '') eq 'if') && ($test = $defn->getTest)) {
//     return &$test($gullet, @args); }
//   elsif (XEquals($if, T_CS('\iftrue'))) {
//     return 1; }
//   elsif (XEquals($if, T_CS('\iffalse'))) {
//     return 0; }
//   else {
//     Error('expected', 'conditional', $gullet,
//       "Expected a conditional, got '" . ToString($if) . "'");
//     return; } }

// # Used only for regular \newif type conditions
// sub SetCondition {
//   my ($if, $value, $scope) = @_;
//   my ($defn, $test);
//   # We'll accept any conditional \ifxxx, providing it takes no arguments
//   if (($defn = $STATE->lookupDefinition($if)) && (($$defn{conditional_type} || '') eq 'if')
//     && !$defn->getParameters) {
//     Let($if, ($value ? T_CS('\iftrue') : T_CS('\iffalse')), $scope) }
//   else {
//     Error('expected', 'conditional', $STATE->getStomach,
//       "Expected a conditional defined by \\newif, got '" . ToString($if) . "'"); }
//   return; }

///======================================================================
/// Define a primitive control sequence.
///======================================================================
/// Primitives are executed in the Stomach.
/// The $replacement should be a sub which returns nothing, or a list of `Box`'s or `Whatsit`'s.
/// The options are:
///    is_prefix  : 1 for things like \global, \long, etc.
///    registerType : for parameters (but needs to be worked into `DefParameter`, below).

#[macro_export]
macro_rules! DefPrimitive {
  // Case: simple literal replacement
  ($proto:literal, $replacement:literal $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    let (cs, params) = parse_prototype!($proto);
    let replacement_closure = Rc::new(|stomach: &mut Stomach, args: Vec<Tokens>, inner_state: &mut State| {
      Tbox::new($replacement.to_string(), None, None, Tokens!(), HashMap::new(), inner_state).into_digested_result()
    });
    defi_primitive!(cs, params, replacement_closure, options);
  }};
  // Case: closure pattern replacement
  ($proto:expr, sub[$stomach_arg:ident, $args:ident, $state_arg:ident] $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    let (cs, params) = parse_prototype!($proto);
    let replacement_closure = Rc::new(move |$stomach_arg: &mut Stomach, $args: Vec<Tokens>, $state_arg: &mut State| {
      WithInnerState!($body, $stomach_arg, $state_arg).into_digested_result()
    });
    defi_primitive!(cs, params, replacement_closure, options);
  }};
  // Case: cs-noparams with closure pattern replacement
  ($cs:expr, None, sub[$stomach_arg:ident, $args:ident, $state_arg:ident] $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    let replacement_closure = Rc::new(move |$stomach_arg: &mut Stomach, $args: Vec<Tokens>, $state_arg: &mut State| {
      WithInnerState!($body, $stomach_arg, $state_arg).into_digested_result()
    });
    defi_primitive!($cs, None, replacement_closure, options);
  }};
  // Case: no replacement
  ($proto:literal, None $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    let (cs, params) = parse_prototype!($proto);
    defi_primitive!(cs, params, Rc::new(noprimitive!()), options);
  }};
  // Case: no params, no replacement
  ($cs:expr, None, None $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    defi_primitive!($cs, None, Rc::new(noprimitive!()), options);
  }};

  // Case: closure block with implicit arguments
  ($proto:expr, $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    let (cs, params) = parse_prototype!($proto);
    let replacement_closure =  Rc::new(move |stomach: &mut Stomach, args: Vec<Tokens>, state: &mut State| {
      WithInnerState!($body, stomach, state).into_digested_result()
    });
    defi_primitive!(cs, params, replacement_closure, options);
  }};
  // Case: direct closure provided (for reasons of reusing the same closure in several definitions)
  ($proto:expr, $replacement_closure:expr, $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    defi_primitive!($proto, $replacement_closure, options);
  }};
  ($proto:expr, $replacement_closure:expr) => {{
    let (cs, params) = parse_prototype!($proto);
    defi_primitive!(cs, params, $replacement_closure, PrimitiveOptions::default());
  }};
}

#[macro_export]
macro_rules! defi_primitive(
  ($cs:expr, $params:expr, $compiled_replacement:expr, $options:expr) => {{
    bind_state_mut!(st);
    defi_primitive!($cs, $params, $compiled_replacement, $options, st)
  }};
  ($cs:expr, $params:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => {{
    def_primitive($cs, $params, $compiled_replacement, $options, $state_arg);
  }};
);

#[macro_export]
macro_rules! LookupRegister {
  ($cs:expr) => {
    LookupRegister!($cs, Vec::new())
  };
  ($cs:expr, $parameters:expr) => {{
    bind_state_mut!(st);
    LookupRegister!($cs, $parameters, st)
  }};
  ($cs:expr, $parameters:expr, $state_arg: ident) => {
    if let Some(defn) = $state_arg.lookup_register_definition(&T_CS!($cs)) {
      defn.value_of($parameters, $state_arg).unwrap_or_default()
    } else {
      let message = s!("The control sequence {:?} is not a register", $cs);
      Warn!("expected", "register", None, $state_arg, message);
      RegisterValue::default()
    }
  };
}

#[macro_export]
macro_rules! LookupRegisterOrDefault {
  ($cs:expr) => {
    LookupRegisterOrDefault!($cs, Vec::new())
  };
  ($cs:expr, $parameters:expr) => {{
    bind_state_mut!(st);
    LookupRegisterOrDefault!($cs, $parameters, st)
  }};
  ($cs:expr, $parameters:expr, $state_arg: ident) => {
    if let Some(defn) = $state_arg.lookup_register_definition(&T_CS!($cs)) {
      defn.value_of($parameters, $state_arg).unwrap_or_default()
    } else {
      RegisterValue::default()
    }
  };
}

// sub LookupDimension {
//   my ($cs) = @_;
//   my $defn;
//   $cs = T_CS($cs) unless ref $cs;
//   if (my $defn = $STATE->lookupDefinition($cs)) {
//     if ($defn->isRegister) {    # Easy (and proper) case.
//       return $defn->valueOf; }
//     else {
//       $STATE->getStomach->getGullet->readingFromMouth(LaTeXML::Core::Mouth->new(), sub { # start with empty mouth
//           my ($gullet) = @_;
//           $gullet->unread($cs);    # but put back tokens to be read
//           return $gullet->readDimension; }); } }
//   else {
//     Warn('expected', 'register', $STATE->getStomach,
//       "The control sequence " . ToString($cs) . " is not a register"); }
//   return Dimension(0); }

#[macro_export]
macro_rules! AssignRegister {
  ($cs:literal, $value:expr) => {{
    bind_state!(stmch, st);
    AssignRegister!($cs, $value, Vec::new(), st);
  }};
  ($cs:literal, $value:expr, $args:expr, $state_arg: ident) => {{
    if let Some(defn) = $state_arg.lookup_register_definition(&T_CS!($cs)) {
      defn.borrow_mut().set_value($value, $args, $state_arg);
    } else {
      let message = s!("The control sequence {} is not a register", $cs);
      Warn!("expected", "register", None, $state_arg, message);
    }
  }};
}

//======================================================================
// Define a constructor control sequence.
//======================================================================
// The arguments, if any, will be collected and processed in the Stomach, and
// a Whatsit will be constructed.
// It is the Whatsit that will be processed in the Document: It is responsible
// for constructing XML Nodes.  The $replacement should be a sub which inserts nodes,
// or a string specifying a constructor pattern (See somewhere).
//
// Options are:
//   bounded         : any side effects of before/after daemans are bounded; they are
//                     automatically enclosed by bgroup/egroup pair.
//   mode            : causes a switch into the given mode during the Whatsit building in the stomach.
//   reversion       : a string representing the preferred TeX form of the invocation.
//   beforeDigest    : code to be executed (in the stomach) before parsing & constructing the Whatsit.
//                     Can be used for changing modes, beginning groups, etc.
//   afterDigest     : code to be executed (in the stomach) after parsing & constructing the Whatsit.
//                     useful for setting Whatsit properties,
//   properties      : a hashref listing default values of properties to assign to the Whatsit.
//                     These properties can be used in the constructor.

#[macro_export]
macro_rules! DefConstructor {
  // Closure replacement flavors
  ($proto:literal, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    let compiled_replacement : Option<ReplacementClosure> = Some(Rc::new(replacement!($document, $args, $props, $inner_state, $body)));
    let (cs, params) = parse_prototype!($proto);
    defi_constr!(cs, params, compiled_replacement, options);
  }};
  // Literal replacement flavors
  ($proto:expr, $replacement:literal) => {{
    let (cs, params) = parse_prototype!($proto);
    let compiled_replacement;
    compile_replacement!(compiled_replacement, $replacement);
    defi_constr!(cs, params, compiled_replacement, ConstructorOptions::default());
  }};
  // Pre-parsed prototype flavors
  // Pre-parsed prototype; Closure replacement flavors
  ($cs:literal, $parameters:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block, $($input:tt)+) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    defi_constr!($proto, $document, $args, $props, $inner_state, $body, options);
  }};
  ($cs:literal, $parameters:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block) => {{
    let compiled_replacement = Some(Rc::new(replacement!($document, $args, $props, $inner_state, $body)));
    defi_constr!($cs, $parameters, compiled_replacement, ConstructorOptions::default());
  }};
  //  Pre-parsed prototype; Literal replacement flavors
  ($cs:expr, $parameters:expr, $replacement:literal) => {{
    let compiled_replacement;
    compile_replacement!(compiled_replacement, $replacement);
    defi_constr!($cs, $parameters, compiled_replacement, ConstructorOptions::default());
  }};
  ($cs:expr, $parameters:expr, None) => {{
    defi_constr!($cs, $parameters, $replacement, ConstructorOptions::default());
  }};

  // Optioned flavors come last due to :tt munching
  ($cs:expr, $parameters:expr, $replacement:literal, $($input:tt)+) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    let compiled_replacement;
    compile_replacement!(compiled_replacement, $replacement);
    defi_constr!($cs, $parameters, compiled_replacement, options);
  }};
  ($cs:expr, $parameters:expr, None, $($input:tt)+) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    defi_constr!($cs, $parameters, None, options);
  }};
  ($proto:expr, $replacement:literal, $($input:tt)+) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    let (cs, params) = parse_prototype!($proto);
    let compiled_replacement;
    compile_replacement!(compiled_replacement, $replacement);
    defi_constr!(cs, params, compiled_replacement, options);
  }};
}

/// Internal auxiliary, only purpose is to bind state, then call the api::def_constructor
/// function, where the interior construction logic resides.
#[macro_export]
macro_rules! defi_constr {
  ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options:expr) => {{
    bind_state_mut!(st);
    defi_constr!($cs, $paramlist, $compiled_replacement, $options, st);
  }};
  ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => {
    def_constructor($cs, $paramlist, $compiled_replacement, $options, $state_arg);
  };
}

/// Internal auxiliary, taking a prototype string (possibly through a variable) and
/// invoking the state
#[macro_export]
macro_rules! parse_prototype(
  ($proto:expr) => {{
    bind_state_mut!(st);
    parse_prototype($proto, st)?
  }};
  ($proto:expr, $state_arg:ident) => {{
    parse_prototype($proto, $state_arg)?
  }};
);

// sub DocType {
//   my ($rootelement, $pubid, $sysid, %namespaces) = @_;
//   let model = state->getModel;
//   $model->setDocType($rootelement, $pubid, $sysid);
//   foreach let prefix (keys %namespaces) {
//     $model->registerDocumentNamespace($prefix => $namespaces{$prefix}); }
//   return; }

#[macro_export]
macro_rules! DefEnvironmentWO (
  ($proto_raw:expr, $replacement:expr, $options:expr) => {{
    bind_state_mut!(st);
    DefEnvironmentWO!($proto_raw, $replacement, $options, st)
  }};
  ($proto_raw:expr, $replacement:expr, $options:expr, $state_arg:ident) => ({
  use rtx_core::util::text::*;
  let mut proto = $proto_raw.to_string().trim_start().to_string();
  let name = extract_bracketed(&mut proto, Some(&Delimiter::Brace)).unwrap_or_default();
  let compiled_replacement;
  compile_replacement!(compiled_replacement, $replacement);

  let options = $options;
  def_environment(name, None, compiled_replacement, options, $state_arg);
}));

#[macro_export]
macro_rules! DefEnvironmentIWO (
  ($proto_raw:expr, $compiled_replacement:expr, $options:expr) => {{
    bind_state_mut!(st);
    DefEnvironmentIWO!($proto_raw, $compiled_replacement, $options, st)
  }};
  ($proto_raw:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => ({
  use rtx_core::util::text::*;
  let mut proto = $proto_raw.to_string().trim_start().to_string();
  let name = extract_bracketed(&mut proto, Some(&Delimiter::Brace)).unwrap_or_default();
  // TODO: What do we do with param lists?
  //let paramlist_str = proto.trim_start().to_string();
  def_environment(name, None, $compiled_replacement, $options, $state_arg);
}));

#[macro_export]
macro_rules! RelaxNGSchema {
  ($name:expr) => {{
    bind_state_mut!(st);
    RelaxNGSchema!($name, st)
  }};
  ($name:expr,$state_arg:ident) => {
    select_relaxng_schema($name.to_string(), None, $state_arg)
  };
}

#[macro_export]
macro_rules! RegisterNamespace(
  ($prefix:expr, $namespace:expr) => {{
    bind_state_mut!(st);
    RegisterNamespace!($prefix, $namespace, st)
  }};
  ($prefix:expr, $namespace:expr,$state_arg:ident) =>
    ($state_arg.model.register_namespace($prefix, Some($namespace.to_string())))
);
#[macro_export]
macro_rules! RegisterDocumentNamespace(
  ($prefix:expr, $namespace:expr) => {{
    bind_state_mut!(st);
    RegisterDocumentNamespace!($prefix, $namespace, st)
  }};
  ($prefix:expr, $namespace:expr,$state_arg:ident) =>
    ($state_arg.model.register_document_namespace($prefix, Some($namespace.to_string())))
);
#[macro_export]
macro_rules! RequireResource(
  ($resource:expr) => {{
    bind_state_mut!(st);
    RequireResource!($resource, st)
  }};
  ($resource:expr,$state_arg:ident) =>
    (require_resource(Resource{name: $resource.to_string(), ..Resource::default()}, $state_arg))
);

#[macro_export]
macro_rules! defi_math {
  ($cstext:expr, $paramlist:expr, $presentation:expr, $options:expr) => {{
    bind_state_mut!(st);
    defi_math!($cstext, $paramlist, $presentation, $options, st)
  }};
  ($cstext:expr, $paramlist:expr, $presentation:expr, $options:expr, $state_arg:ident) => {{
    let mut options = $options;
    let cs = T_CS!($cstext.to_string());
    let presentation = $presentation.to_string();
    // Can't defer parsing parameters since we need to know number of args!
    // $paramlist = parseParameters($paramlist, $cs) if defined $paramlist && !ref $paramlist;
    let paramlist: Option<Parameters> = $paramlist;
    let nargs = match paramlist {
      Some(ref plist) => plist.get_num_args(),
      None => 0,
    };
    let csname = cs.get_string().to_string();
    let mut name = options.alias.clone().unwrap_or_else(|| csname.clone());
    if name.starts_with('\\') {
      name = name.replacen('\\', "", 1)
    }
    if let Some(options_name) = options.name {
      name = options_name;
    }
    let name_opt = if (name == presentation) || (name.is_empty()) || (options.meaning == Some(name.clone())) {
      None
    } else {
      Some(name)
    };
    options.name = name_opt;
    if nargs == 0 && options.role.is_none() {
      options.role = Some(s!("UNKNOWN"))
    }
    if nargs > 0 && options.operator_role.is_none() {
      options.operator_role = Some(s!("UNKNOWN"))
    }

    // Store some data for introspection
    // defmath_introspective(cs, $paramlist, presentation, %options);

    // If single character, handle with a rewrite rule
    if csname.len() == 1 {
      // WAS: defmath_rewrite!($cs, options);
      // No, do NOT make mathactive; screws up things like babel french, or... ?
      // EXPERIMENT: store XMTok attributes for if this char ends up a Math Token.
      // But only some DefMath options make sense!
      // let rw_options = { name => 1, meaning => 1, omcd => 1, role => 1, mathstyle => 1, stretchy => 1 }; # (well, mathstyle?)
      // CheckOptions("DefMath reimplemented as DefRewrite ($csname)", $rw_options, %options);
      let mut math_attr_hash: HashMap<String, String> = HashMap::new();
      transfer_opt_default!(name, options, math_attr_hash);
      transfer_opt_default!(meaning, options, math_attr_hash);
      transfer_opt_default!(omcd, options, math_attr_hash);
      transfer_opt_default!(role, options, math_attr_hash);
      transfer_opt_default!(mathstyle, options, math_attr_hash);
      transfer_default!(stretchy, options, math_attr_hash);
      $state_arg.assign_value(&s!("math_token_attributes_{}", csname), math_attr_hash, Some(Scope::Global));
    }
    // TODO:
    // // If the presentation is complex, and involves arguments,
    // // we will create an XMDual to separate content & presentation.
    // elsif ((ref presentation eq "CODE")
    //   || ((ref presentation) && grep { $_->equals(T_PARAM) } presentation->unlist)
    //   || (!(ref presentation) && (presentation =~ /\//\d|\\./))
    //   || ((ref presentation) && (grep { $_->isExecutable } presentation->unlist))) {
    //   defmath_dual($cs, $paramlist, presentation, %options); }

    // EXPERIMENT: Introduce an intermediate case for simple symbols
    // Define a primitive that will create a Box with the appropriate set of XMTok attributes.
    if nargs == 0 {
      // && !grep { !$$simpletoken_options{$_} } keys %options) {
      def_math_primitive(cs, paramlist, $presentation.to_string(), options, $state_arg);
    }

    // else {
    //   defmath_cons($cs, $paramlist, $presentation, %options); }
    // AssignValue($csname . ":locked" => 1) if $options{locked};
  }};
}

#[macro_export]
macro_rules! requireMath {
  ($cs_name:expr) => {{
    bind_state_mut!(st);
    requireMath!($cs_name, st)
  }};
  ($cs_name:expr, $state_arg:ident) => {
    if !LookupBool!("IN_MATH", $state_arg) {
      let message = s!("{} should only appear in math mode", $cs_name);
      Warn!("unexpected", "mode", None, $state_arg, message);
    }
  };
}
#[macro_export]
macro_rules! forbidMath {
  ($cs_name:expr) => {{
    bind_state_mut!(st);
    forbidMath!($cs_name, st)
  }};
  ($cs_name:expr, $state_arg:ident) => {
    if LookupBool!("IN_MATH", $state_arg) {
      let message = s!("{} should not appear in math mode", $cs_name);
      Warn!("unexpected", "mode", None, $state_arg, message);
    }
  };
}

//======================================================================
// Counters
//======================================================================
// This is modelled on LaTeX's counter mechanisms, but since it also
// provides support for ID's, even where there is no visible reference number,
// it is defined in genera.
// These id's should be both unique, and parallel the visible reference numbers
// (as much as possible).  Also, for consistency, we add id's to unnumbered
// document elements (eg from \section*); this requires an additional counter
// (eg. UNsection) and  mechanisms to track it.

// Defines a new counter named $ctr.
// If $within is defined, $ctr will be reset whenever $within is incremented.
// Keywords:
//  idprefix : specifies a prefix to be used in sting ID's for document structure elements
//           counted by this counter.  Ie. subsection 3 in section 2 might get: id="S2.SS3"
//  idwithin : specifies that the ID is composed from $idwithin's ID,, even though
//           the counter isn't numbered within it.  (mainly to avoid duplicated ids)
//   nested : a list of counters that correspond to scopes which are "inside" this one.
//           Whenever any definitions scoped to this counter are deactivated,
//           the inner counter's scopes are also deactivated.
//           NOTE: I'm not sure this is even a sensible implementation,
//           or why inner should be different than the counters reset by incrementing this counter.
#[macro_export]
macro_rules! NewCounterWO {
  ($ctr:expr, $within:expr, None) => {
    bind_state_mut!(stmch, st);
    new_counter($ctr, $within, None, stmch, st)?
  };
  ($ctr:expr, $within:expr, None, $state_arg:ident) => {
    new_counter($ctr, $within, None, $state_arg)?
  };
  ($ctr:expr, $within:expr, Some($opts:expr)) => {
    bind_state_mut!(stmch, st);
    new_counter($ctr, $within, Some($opts), stmch, st)?
  };
  ($ctr:expr, $within:expr, Some($opts:expr), $stomach_arg:ident, $state_arg:ident) => {
    new_counter($ctr, $within, Some($opts), $stomach_arg, $state_arg)?
  };
}
#[macro_export]
macro_rules! CounterValue {
  ($ctr:expr) => {{
    bind_state_mut!(st);
    counter_value($ctr, st)
  }};
  ($ctr:expr, $state_arg:ident) => {
    counter_value($ctr, $state_arg)
  };
}
#[macro_export]
macro_rules! SetCounter {
  ($ctr:expr, $value:expr, None) => {
    AssignValue!(&s!("\\c@{}",$ctr), $value, Some(Scope::Global));
    DefMacro!(T_CS!(s!("\\@{}@ID",$ctr)), None, Tokens::new(Explode!($value.value_of())),
                scope => Some(Scope::Global)
    );
  };
  ($ctr:expr, $value:expr, $gullet:ident) => {
    AssignValue!(&s!("\\c@{}",$ctr), $value, Some(Scope::Global));
    AfterAssignment!();
    DefMacro!(T_CS!(s!("\\@{}@ID",$ctr)), None, Tokens::new(Explode!($value.value_of())),
                scope => Some(Scope::Global)
    );
  }
}
#[macro_export]
macro_rules! AddToCounter {
  ($ctr:expr, $value:expr, $gullet:ident) => {{
    bind_state_mut!(st);
    add_to_counter($ctr, $value, $gullet, st)
  }};
  ($ctr:expr, $value:expr, $gullet:ident, $state_arg:ident) => {
    add_to_counter($ctr, $value, $gullet, $state_arg)
  };
}
#[macro_export]
macro_rules! StepCounter {
  ($ctr:expr, $noreset:expr, $stomach:ident) => {{
    bind_state_mut!(st);
    step_counter($ctr, $noreset, $stomach, st)
  }};
  ($ctr:expr, $noreset:expr, $stomach:ident, $state_arg:ident) => {
    step_counter($ctr, $noreset, $stomach, $state_arg)
  };
}
#[macro_export]
macro_rules! RefStepCounter {
  ($ctr:expr, $noreset:expr, $stomach:ident) => {{
    bind_state_mut!(st);
    ref_step_counter($ctr, $noreset, $stomach, st)
  }};
  ($ctr:expr, $noreset:expr, $stomach:ident, $state_arg:ident) => {
    ref_step_counter($ctr, $noreset, $stomach, $state_arg)
  };
}
#[macro_export]
macro_rules! RefStepID {
  ($ctr:expr) => {{
    bind_state_mut!(stmch, st);
    ref_step_id($ctr, stmch, st)
  }};
  ($ctr:expr, $stomach:ident) => {{
    bind_state_mut!(st);
    ref_step_id($ctr, $stomach, st)
  }};
  ($ctr:expr, $stomach:ident, $state_arg:ident) => {
    ref_step_id($ctr, $stomach, $state_arg)
  };
}
#[macro_export]
macro_rules! ResetCounter {
  ($ctr:expr) => {{
    bind_state_mut!(st);
    reset_counter($ctr, st)
  }};
  ($ctr:expr, $state_arg: ident) => {
    reset_counter($ctr, $state_arg)
  };
}

/// Return $tokens with all tokens expanded
#[macro_export]
macro_rules! Expand {
  ($tokens:expr, $gullet:ident) => {{
    bind_state_mut!(st);
    do_expand($tokens, $gullet, st)?
  }};
  ($tokens:expr, $gullet:ident, $state_arg:ident) => {
    do_expand($tokens, $gullet, $state_arg)?
  };
}

/// Invocation(<list of Token>); builds a representation of a command sequence invoked on its
/// arguments
#[macro_export]
macro_rules! Invocation {
  ($csname:literal, $args:expr, $gullet:expr) => {{
    bind_state_mut!(st);
    Invocation!(T_CS!($csname), $args, $gullet, st)
  }};
  ($csname:literal, $args:expr, $gullet:expr, $state_arg:ident) => {
    Invocation!(T_CS!($csname), $args, $gullet, $state_arg)
  };
  ($token:expr, $args:expr, $gullet:expr) => {{
    bind_state_mut!(st);
    Invocation!($token, $args, $gullet, st)
  }};
  ($token:expr, $args:expr, $gullet:expr, $state_arg:ident) => {
    build_invocation($token, $args.into_iter().map(Into::into).collect(), $gullet, $state_arg)
  };
}
#[macro_export]
macro_rules! DefLigature {
  ($regex:expr, $replacement:expr, fontTest => sub[$font:ident] $body:block) => {
    bind_state_mut!(st);
    DefLigature!($regex, $replacement, fontTest => sub[$font]{$body}, st)
  };
  ($regex:expr, $replacement:expr, fontTest => sub[$font:ident] $body:block, $state_arg:ident) => {
    #[allow(clippy::trivial_regex)]
    let regex_compiled = Regex::new($regex).unwrap();
    let test_closure: Option<FontTestClosure> = Some(Rc::new(move |$font| $body));
    $state_arg.unshift_value(
      "TEXT_LIGATURES",
      vec![Ligature {
        regex: $regex.to_string(),
        code: Rc::new(move |text| regex_compiled.replace_all(text, $replacement).to_string()),
        font_test: test_closure,
      }],
    );
  };
  ($regex:expr, $replacement:expr) => {{
    bind_state_mut!(st);
    DefLigature!($regex, $replacement, st)
  }};
  ($regex:expr, $replacement:expr, $state_arg:ident) => {
    let regex_compiled = Regex::new($regex).unwrap();
    $state_arg.unshift_value(
      "TEXT_LIGATURES",
      vec![Ligature {
        regex: $regex.to_string(),
        code: Rc::new(move |text| regex_compiled.replace_all(text, $replacement).to_string()),
        font_test: None,
      }],
    );
  };
}

// Defines an accent command using a combining char that follows the
// 1st char of the argument.  In cases where there is no argument, $standalonechar is used.
#[macro_export]
macro_rules! DefAccent {
  ($accent:expr, $combiningchar:expr, $standalonechar:expr) => {{
    let mut empty_opts : HashMap<String, Stored> = HashMap::new();
    bind_state_mut!(st);
    DefAccent!($accent, $combiningchar, $standalonechar, empty_opts, st)
  }};
  ($accent:expr, $combiningchar:expr, $standalonechar:expr, below => true) => {{
    bind_state_mut!(st);
    DefAccent!($accent, $combiningchar, $standalonechar, map!("below"=>Stored::Bool(true)), st)
  }};
  ($accent:expr, $combiningchar:expr, $standalonechar:expr, $options:expr) => {{
    bind_state_mut!(st);
    DefAccent!($accent, $combiningchar, $standalonechar, $options, st)
  }};
  ($accent:expr, $combiningchar:expr, $standalonechar:expr, $options:expr, $state_arg: ident) => {{
    if $options.get("below").is_none() {
      $options.entry(String::from("above")).or_insert(Stored::Bool(true));
    }
    // Used for converting a char used as an above-accent to a combining char (See \accent)
    if $options.get("above").is_some() {
      $state_arg.assign_mapping("accent_combiner_above", $standalonechar, Some($combiningchar));
    } else {
      $state_arg.assign_mapping("accent_combiner_below", $standalonechar, Some($combiningchar));
    }
    let accent_proto = format!("{}{{}}",$accent);

    DefPrimitive!(&accent_proto, sub[stomach, letter, inner_state] {
      let invoked = Invocation!(T_CS!($accent), letter.clone(), stomach.get_gullet_mut(), inner_state)?;
      // TODO: check if letter.to_string has artefacts
      crate::package::pool::tex_accents::apply_accent(
        stomach, &letter[0].to_string(), $combiningchar, $standalonechar, Some(invoked), inner_state)?;
      Ok(vec![])
    }, mode => "text");
  }};
}

//============================================
// User-facing Macros
//============================================
//
#[macro_export]
macro_rules! LookupValue {
  ($name:expr) => {{
    bind_state!(st);
    st.lookup_value($name)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.lookup_value($name)
  };
}
#[macro_export]
macro_rules! LookupBool {
  ($name:expr) => {{
    bind_state!(st);
    st.lookup_bool($name)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.lookup_bool($name)
  };
}
#[macro_export]
macro_rules! LookupFont {
  () => {{
    bind_state!(st);
    st.lookup_font()
  }};
  ($state_arg:ident) => {
    $state_arg.lookup_font()
  };
}
#[macro_export]
macro_rules! LookupString {
  ($name:expr) => {{
    bind_state!(st);
    st.lookup_string($name)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.lookup_string($name)
  };
}
#[macro_export]
macro_rules! LookupNumber {
  ($name:expr) => {{
    bind_state!(st);
    st.lookup_number($name)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.lookup_number($name)
  };
}
#[macro_export]
macro_rules! LookupTokens {
  ($name:expr) => {{
    bind_state!(st);
    st.lookup_tokens($name)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.lookup_tokens($name)
  };
}
#[macro_export]
macro_rules! AssignValue {
  ($name:expr => $value:expr) => {
    AssignValue!($name, $value)
  };
  ($name:expr => $value:expr, $scope:expr) => {
    AssignValue!($name, $value, $scope)
  };
  ($name:expr, $value:expr) => {{
    bind_state_mut!(st);
    st.assign_value($name, $value, None)
  }};
  ($name:expr, $value:expr, $scope:expr) => {{
    bind_state_mut!(st);
    st.assign_value($name, $value, $scope)
  }};
  ($name:expr, $value:expr, $scope:expr, $state_arg:ident) => {
    $state_arg.assign_value($name, $value, $scope)
  };
}
#[macro_export]
macro_rules! RemoveValue {
  ($name:expr) => {{
    bind_state_mut!(st);
    st.remove_value($name)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.remove_value($name)
  };
}
#[macro_export]
macro_rules! PushValue {
  ($name:expr => $values:expr) => {{
    bind_state_mut!(st);
    st.push_value($name, $values)
  }};
  ($name:expr, $values:expr) => {{
    bind_state_mut!(st);
    st.push_value($name, $values)
  }};
  ($name:expr, $values:expr, $state_arg:ident) => {
    $state_arg.push_value($name, $values)
  };
}
#[macro_export]
macro_rules! PopValue {
  ($name:expr) => {{
    bind_state_mut!(st);
    st.pop_value($name)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.pop_value($name)
  };
}
#[macro_export]
macro_rules! UnshiftValue {
  ($name:expr, $values:expr) => {{
    bind_state_mut!(st);
    st.unshift_value($name, $values)
  }};
  ($name:expr, $values:expr,$state_arg:ident) => {
    $state_arg.unshift_value($name, $values)
  };
}
#[macro_export]
macro_rules! ShiftValue {
  ($name:expr) => {{
    bind_state_mut!(st);
    st.shift_value($name)
  }};
  ($name:expr,$state_arg:ident) => {
    $state_arg.shift_value($name)
  };
}
#[macro_export]
macro_rules! LookupMapping {
  ($map:expr, $key:expr) => {{
    bind_state_mut!(st);
    st.lookup_mapping($map, $key)
  }};
  ($map:expr, $key:expr, $state_arg:ident) => {
    $state_arg.lookup_mapping($map, $key)
  };
}
#[macro_export]
macro_rules! AssignMapping {
  ($map:expr, $key:expr => $value:expr) => {{
    bind_state_mut!(st);
    AssignMapping!($map, $key => $value, st)
  }};
  ($map:expr, $key:expr => $value:expr, $state_arg:ident) => {
    $state_arg.assign_mapping($map, $key, $value.into())
  };
}
#[macro_export]
macro_rules! AssignMeaning {
  ($key:expr, $val:expr) => {
    AssignMeaning!($key, $val, None)
  };
  ($key:expr, $val:expr, $scope: expr) => {{
    bind_state_mut!(st);
    st.assign_meaning($key, $val, $scope)
  }};
  ($key:expr, $val:expr, $scope: expr, $state_arg:ident) => {
    $state_arg.assign_meaning($key, $val, $scope)
  };
}

#[macro_export]
macro_rules! LookupMappingKeys {
  ($map:expr) => {{
    bind_state_mut!(st);
    LookupMappingKeys!($map, st)
  }};
  ($map:expr, $state_arg:ident) => {
    $state_arg.lookup_mapping_keys($map)
  };
}
#[macro_export]
macro_rules! LookupCatcode {
  ($c:expr) => {{
    bind_state_mut!(st);
    st.lookup_catcode($c)
  }};
  ($c:expr, $state_arg:ident) => {
    $state_arg.lookup_catcode($c)
  };
}
#[macro_export]
macro_rules! AssignCatcode {
  ($name:expr => $value:expr) => {
    AssignCatcode!($name, $value)
  };
  ($c:expr, $catcode:expr) => {{
    bind_state_mut!(st);
    AssignCatcode!($c, $catcode, None, st)
  }};
  ($c:expr, $catcode:expr, $scope:expr) => {{
    bind_state_mut!(st);
    AssignCatcode!($c, $catcode, $scope, st)
  }};
  ($c:expr, $catcode:expr, $scope:expr, $state_arg:ident) => {
    $state_arg.assign_catcode($c, $catcode, $scope)
  };
}
#[macro_export]
macro_rules! LookupMeaning {
  ($name:expr) => {{
    bind_state_mut!(st);
    LookupMeaning!($name, st)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.lookup_meaning($name)
  };
}
#[macro_export]
macro_rules! LookupDefinition {
  ($name:expr) => {{
    bind_state_mut!(st);
    LookupDefinition!($name, st)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.lookup_definition($name)
  };
}
#[macro_export]
macro_rules! InstallDefinition {
  ($name:expr, $definition:expr, $scope:expr) => {{
    bind_state_mut!(st);
    InstallDefinition!($name, $definition, $scope, st)
  }};
  ($name:expr, $definition:expr, $scope:expr, $state_arg:ident) => {
    $state_arg.install_definition($name, $definition, $scope)
  };
}
#[macro_export]
macro_rules! XEquals {
  ($token1:expr, $token2:expr) => {{
    bind_state_mut!(st);
    XEquals!($token1, $token2, st)
  }};
  ($token1:expr, $token2:expr, $state_arg:ident) => {
    $state_arg.x_equals($token1, $token2)
  };
}
#[macro_export]
macro_rules! IsDefined {
  ($name:expr) => {{
    bind_state_mut!(st);
    IsDefined!($name, st)
  }};
  ($name:expr, $state_arg:ident) => {
    is_defined_token($name, $state_arg)
  };
}
#[macro_export]
macro_rules! IsDefinedToken {
  ($name:expr) => {{
    bind_state_mut!(st);
    is_defined_token($name, st)
  }};
}
#[macro_export]
macro_rules! IsDefinable {
  ($token: expr) => {{
    bind_state_mut!(st);
    IsDefinable!($token, st)
  }};
  ($token: expr, $state_arg: ident) => {
    is_definable($token, $state_arg)
  };
}

#[macro_export]
macro_rules! Let {
  ($token1:literal, $token2:literal) => {{
    bind_state_mut!(st);
    st.let_i(&T_CS!($token1), T_CS!($token2), None);
  }};
  // half-packaged args
  ($token1:literal, $token2:expr) => {{
    bind_state_mut!(st);
    st.let_i(&T_CS!($token1), $token2, None);
  }};
  ($token1:expr, $token2:literal) => {{
    bind_state_mut!(st);
    st.let_i($token1, T_CS!($token2), None);
  }};
  // internal form, pre-packaged arguments
  ($token1:expr, $token2:expr) => {{
    bind_state_mut!(st);
    st.let_i($token1, $token2, None);
  }};
  ($token1:expr, $token2:expr, $scope:expr) => {{
    bind_state_mut!(st);
    st.let_i($token1, $token2, $scope);
  }};
}

#[macro_export]
macro_rules! DigestIf {
  ($token:literal, $stomach:ident) => {{
    bind_state_mut!(st);
    DigestIf!(T_CS!($token), $stomach, st)
  }};
  ($token:literal, $stomach:ident, $state_arg:ident) => {
    digest_if(T_CS!($token), $stomach, $state_arg)
  };
  ($token:expr, $stomach:ident) => {{
    bind_state_mut!(st);
    DigestIf!($token, $stomach, st)
  }};
  ($token:expr, $stomach:ident, $state_arg: ident) => {
    digest_if($token, $stomach, $state_arg)
  };
}
#[macro_export]
macro_rules! AfterAssignment {
  () => {{
    bind_state_mut!(st);
    st.after_assignment()
  }};
  ($state_arg: ident) => {
    $state_arg.after_assignment()
  };
}

/// Merge the current font with the style specifications
#[macro_export]
macro_rules! MergeFont {
  ($kv:expr) => {{
    bind_state_mut!(st);
    MergeFont!($kv, st)
  }};
  ($kv:expr, $state_arg:ident) => {
    merge_font($kv, $state_arg)
  };
  ($key:ident => $val:expr) => {{
    bind_state_mut!(st);
    MergeFont!($key => $val, st)
  }};
  ($key:ident => $val:expr, $state_arg:ident) => {
    merge_font(fontmap!($key => $val), $state_arg)
  };
}

//============================================
// User-facing Argument Parsers
//============================================
//
// There is a lot of "Do What I Mean" logic going on here, to allow binding writers to thoughtlessly use a single DefMacro!()
// and have:
// - the macro machinery auto-wrap the correct union type containers,
// - auto-compile the various string replacements and prototypes into their rust data structures
// - auto-build the ExpansionOptions data structure from a Perl-like syntax, and validate it along the way
//  we're taking things a few pegs further than LaTeXML, as DefMacroI syntax is *included* in DefMacro,
//  and we have a several places where we get compile-time speedups by pre-tokenizing into Rust Tokens objects / Replacement closures
#[macro_export]
macro_rules! DefMacro {
  // closure
  ($proto:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    let (cs, params) = parse_prototype!($proto);
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |$gullet, $args, $inner_state| WithInnerState!($body, $inner_state).into_tokens_result()
    )));
    defi_macro!(cs, params, expansion_closure, Some(options));
  }};
  ($proto:expr, $body:block) => {{
    let (cs, params) = parse_prototype!($proto);
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |gullet, args, inner_state| WithInnerState!($body, inner_state).into_tokens_result()
    )));
    defi_macro!(cs, params, expansion_closure, None);
  }};
  // String; implicit state
  ($proto:literal, $expansion:literal $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    let (cs, params) = parse_prototype!($proto);
    let compiled_expansion;
    compile_expansion!(compiled_expansion, $expansion);
    defi_macro!(cs, params, compiled_expansion, Some(options));
  }};
  // Internal-level use
  ($cs:expr, $parameters:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |$gullet, $args, $inner_state| WithInnerState!($body, $inner_state).into_tokens_result()
    )));
    defi_macro!($cs, $parameters, expansion_closure, Some(options));
  }};
  ($cs:literal, None, $expansion:literal) => {{
    let compiled_expansion;
    compile_expansion!(compiled_expansion, $expansion);
    defi_macro!(T_CS!($cs), None, compiled_expansion, None);
  }};
  ($cs:literal, None, $expansion:literal, $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    let compiled_expansion;
    compile_expansion!(compiled_expansion, $expansion);
    defi_macro!(T_CS!($cs), None, compiled_expansion, None);
  }};
  ($cs:expr, None, $expansion:literal) => {{
    let compiled_expansion;
    compile_expansion!(compiled_expansion, $expansion);
    defi_macro!($cs, None, compiled_expansion, None);
  }};
  ($cs:expr, None, $expansion:expr) => {{
    defi_macro!($cs, None, $expansion, None);
  }};
  ($cs:expr, None, $expansion:expr, $state_arg:ident) => {{ // explicit state, for nested macro factories
    def_macro($cs, None, $expansion, None, $state_arg);
  }};
  ($cs:expr, None, $expansion:literal, $($input:tt)+) => {{
    let compiled_expansion;
    compile_expansion!(compiled_expansion, $expansion);
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    defi_macro!($cs, None, compiled_expansion, Some(options));
  }};
  ($cs:expr, None, $expansion:expr, $($input:tt)+) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    defi_macro!($cs, None, $expansion, Some(options));
  }};
  // the triple expr case should be near the end, as it matches too many cases.
  // It's an internal use of DefMacro e.g. with 3 variable name arguments
  ($cs:expr, $replacement:expr, $expansion:expr) => {{
    defi_macro!($cs, $replacement, $expansion, None);
  }};

  // The least-specified option-parsing cases come last due to the TT munchers accepting any inputs
  ($proto:literal, None $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    let (cs, params) = parse_prototype!($proto);
    defi_macro!(cs, params, None, Some(options));
  }};

}
/// Internal auxiliary, only purpose is to bind state, then call the api::def_macro
/// function, where the interior macro installation logic resides.
#[macro_export]
macro_rules! defi_macro {
  ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options:expr) => {{
    bind_state_mut!(st);
    defi_macro!($cs, $paramlist, $compiled_replacement, $options, st);
  }};
  ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => {
    def_macro($cs, $paramlist, $compiled_replacement, $options, $state_arg);
  };
}

#[macro_export]
macro_rules! DefRegister {
  ($proto:expr => $value:expr) => {{
    let (cs, params) = parse_prototype!($proto);
    defi_register!(cs, params, $value, None);
  }};
  ($proto:expr, $value:expr) => {{
    let (cs, params) = parse_prototype!($proto);
    defi_register!(cs, params, $value, None);
  }};
  ($cs:expr, None, $value:expr) => {{
    defi_register!($cs, None, $value, None);
  }};
  // Option parsers are more lenient, should be at the end of the list of patterns
  ($cs:expr, None, $value:expr, $($input:tt)+) => {{
    let options = defi_opts!(@munch ($($input)*) -> {RegisterOptions,});
    defi_register!($cs, None, $value, Some(options));
  }};
  ($proto:expr, $value:expr, $($input:tt)+) => {{
    let (cs, params) = parse_prototype!($proto);
    let options = defi_opts!(@munch ($($input)*) -> {RegisterOptions,});
    defi_register!(cs, params, $value, Some(options));
  }};
  ($proto:expr => $value:expr, $($input:tt)+) => {{
    let (cs, params) = parse_prototype!($proto);
    let options = defi_opts!(@munch ($($input)*) -> {RegisterOptions,});
    defi_register!(cs, params, $value, Some(options));
  }};
}

/// Internal auxiliary, only purpose is to bind state, then call the api::def_macro
/// function, where the interior macro installation logic resides.
#[macro_export]
macro_rules! defi_register {
  ($cs:expr, $paramlist:expr, $value:expr, $options:expr) => {{
    let value = {
      {
        $value
      }
    }; // allow to reborrow state.
    bind_state_mut!(st);
    def_register($cs, $paramlist, value, $options, st)
  }};
  ($cs:expr, $paramlist:expr, $value:expr, $options:expr, $state_arg:ident) => {
    let value = {
      {
        $value
      }
    }; // allow to reborrow state.
    def_register($cs, $paramlist, value, $options, $state_arg)
  };
}

#[macro_export]
macro_rules! NewCounter {
  ($ctr:expr) => (NewCounterWO!($ctr, "", None));
  ($ctr:expr, $within:expr) => (NewCounterWO!($ctr, $within, None));
  ($ctr:expr, $within:expr, $($key:ident => $val:expr),*) => (NewCounterWO!($ctr, $within, Some(NewDefault!(NewCounterOptions, $($key=>$val),*))));
  // with state
  ($ctr:expr, $state_arg:ident) => (NewCounterWO!($ctr, "", None, $state_arg));
  ($ctr:expr, $within:expr, $state_arg:ident) => (NewCounterWO!($ctr, $within, None, $state_arg));
  ($ctr:expr, $within:expr, $($key:ident => $val:expr),*, $state_arg:ident) =>
    (NewCounterWO!($ctr, $within, Some(NewDefault!(NewCounterOptions, $($key=>$val),*)), $state_arg))
}

//=====================================================================
// Define a LaTeX environment
// Note that the body of the environment is treated is the 'body' parameter in the constructor.
#[macro_export]
macro_rules! DefEnvironment {
  // entry points (this is where a macro call starts):
  ($proto:literal, sub[$document:ident, $args:ident, $props:ident, $state_arg:ident] $body:block, $($input:tt)+ ) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    DefEnvironmentIWO!($proto,
      Some(Rc::new(|$document: &mut Document, $args: &Vec<Option<Digested>>, $props: &HashMap<String, Stored>, $state_arg: &mut State| {
        WithInnerState!($body, $state_arg)
      })),
      options);
  }};
  ($proto:literal, $replacement:expr) => {
    DefEnvironmentWO!($proto, $replacement, ConstructorOptions::default());
  };
  ($proto:literal, $replacement:expr, $($input:tt)* ) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    //                              ^^^^^^^^^^^^    ^^^^^^^^^^^^^^^^^^^^
    //                                 input       output
    DefEnvironmentWO!($proto, $replacement, options);
  }};
}

#[macro_export]
macro_rules! Tag {
  ($tag:expr, $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {TagOptions,});
    bind_state_mut!(st);
    install_tag($tag, options, st);
  }}
}

#[macro_export]
macro_rules! DefMath(
  ($text:expr,$paramlist:expr,$presentation:expr) => (
    defi_math!($text,$paramlist, $presentation, MathPrimitiveOptions::default()));
  ($text:expr,$paramlist:expr,$presentation:expr, $($key:ident => $val:expr),*) => (
    defi_math!($text,$paramlist, $presentation, NewDefaultV!(MathPrimitiveOptions, $($key => $val),*)));
  ($text:expr,$paramlist:expr,$presentation:expr, $($key:ident => $val:expr),*, $state_arg:ident) => (
    defi_math!($text,$paramlist, $presentation, NewDefaultV!(MathPrimitiveOptions, $($key => $val),*,$state_arg)));
);

#[macro_export]
macro_rules! DefParameterType {
  ($name:literal) => (DefParameterTypeWO!($name, NewDefault!(Parameter, name => $name.to_string())));
  ($name:literal, $state_arg:ident) => (DefParameterTypeWO!($name, NewDefault!(Parameter, name => $name.to_string()), $state_arg));
  ($name:literal, $($key:ident => $value:expr),*)=>(DefParameterTypeWO!($name, NewDefault!(Parameter, name => $name.to_string(), $($key=>$value),*)));
  ($name:literal, $($key:ident => $value:expr),*, $state_arg:ident)=>
    (DefParameterTypeWO!($name, NewDefault!(Parameter, name => $name.to_string(), $($key=>$value),*), $state_arg));
  // with reader as explicit sub
  ($name:literal, sub[$gullet:ident, $inner:ident, $extra:ident, $inner_state:ident] $body:block) => (
    DefParameterTypeWO!($name, NewDefault!(Parameter, reader => reader!($gullet, $inner, $extra, $inner_state, $body))));
  ($name:literal, sub[$gullet:ident, $inner:ident, $extra:ident, $inner_state:ident] $body:block, $($key:ident => $value:expr),*) => (
    DefParameterTypeWO!($name, NewDefault!(Parameter, reader => reader!($gullet, $inner, $extra, $inner_state, $body),
      name => $name.to_string(),  $($key=>$value),*)));
  ($name:literal, sub[$gullet:ident, $inner:ident, $extra:ident, $inner_state:ident] $body:block, $($key:ident => $value:expr),*) => (
    DefParameterTypeWO!($name, NewDefault!(Parameter, reader => reader!($gullet, $inner, $extra, $inner_state, $body),
      name => $name.to_string(),  $($key=>$value),*), $state_arg));
}

// Reverts an object into TeX code, as a Tokens list, that would create it.
// Note that this is not necessarily the original TeX.
#[macro_export]
macro_rules! Revert {
  ($thing:literal) => {
    Explode!($thing)
  };
  ($thing:expr) => {{
    bind_state_mut!(st);
    $thing.revert(st)?.unlist()
  }};
}

#[macro_export]
macro_rules! GetKeyVal {
  ($keyval_opt:expr, $key:expr) => {
    match $keyval_opt {
      Some(Digested::KeyVals(keyval)) => keyval.get_value($key),
      _ => None,
    }
  };
}

#[macro_export]
macro_rules! GetKeyVals {
  ($keyval:expr) => {
    match $keyval_opt {
      Some(Digested::KeyVals(keyval)) => keyval..get_key_vals(),
      _ => None,
    }
  };
}

#[macro_export]
macro_rules! Digest {
  ($string:literal) => {{
    bind_state_mut!(stmch, st);
    let tokenized;
    compile_tokenize_internal!(tokenized, $string);
    stmch.digest(tokenized, st)
  }};

  ($tokens:expr) => {{
    bind_state_mut!(stmch, st);
    stmch.digest($tokens, st)
  }};
  ($tokens:expr, $state_arg:ident) => {{
    let mut state_stomach = $state_arg.stomach.clone();
    let mut state_stomach_mut = state_stomach.borrow_mut();
    state_stomach_mut.digest($tokens, $state_arg)
  }};
}

#[macro_export]
macro_rules! DigestText {
  ($tokens:expr) => {
    bind_state_mut!(st);
    digest_text($tokens, outer_stomach!(), st),
  };
  ($tokens:expr, $stomach:ident) => {
    bind_state_mut!(st);
    DigestText!($tokens, $stomach, st)
  };
  ($tokens:expr, $stomach:ident, $state_arg:ident) => {
    digest_text($tokens, $stomach, $state_arg)
  };
}

/// Tokenize($string); Tokenizes the string using the standard cattable, returning a
/// LaTeXML::Core::Tokens
#[macro_export]
macro_rules! Tokenize {
  ($string:literal) => {{
    let tokenized;
    compile_tokenize!(tokenized, $string);
    tokenized
  }};
  ($string:expr) => {
    mouth::tokenize($string, None)
  };
  ($string:expr, None) => {
    mouth::tokenize($string, None)
  };
  ($string:expr, $state_arg:ident) => {
    mouth::tokenize($string, Some($state_arg))
  };
}

/// TokenizeInternal($string); Tokenizes the string using the internal cattable, returning a
/// LaTeXML::Core::Tokens
#[macro_export]
macro_rules! TokenizeInternal {
  ($string:literal) => {{
    let tokenized;
    compile_tokenize_internal!(tokenized, $string);
    tokenized
  }};
  ($string:expr) => {
    mouth::tokenize_internal($string, None)
  };
}

#[macro_export]
macro_rules! RawTeX {
  ($text:literal) => {
    bind_state_mut!(stmch, st);
    let tokenized: Tokens;
    compile_tokenize_internal!(tokenized, $text);
    stmch.digest(tokenized, st)?;
  };
  ($text:expr) => {
    bind_state_mut!(st);
    RawTeX!($text, st)
  };
  ($text:literal, $state_arg:ident) => {{
    let tokenized: Tokens;
    compile_tokenize_internal!(tokenized, $text);
    outer_stomach!().digest(tokenized, $state_arg)?;
  }};
  ($text:expr, $state_arg:ident) => {{
    let mut state_stomach = $state_arg.stomach.clone();
    outer_stomach!().raw_tex($text, $state_arg)?;
  }};
}

#[macro_export]
macro_rules! Dimension {
  ($number:expr) => {{
    bind_state!(st);
    Dimension!($number, st)
  }};
  ($number:expr, $state_arg:ident) => {
    Dimension::new(Dimension::spec_to_f32($number, $state_arg)?)
  };
}

#[macro_export]
macro_rules! Glue {
  ($spec:expr) => {{
    bind_state!(st);
    Glue::new(Glue::spec_to_f32($spec, st)?)
  }};
}

#[macro_export]
macro_rules! MuGlue {
  ($spec:expr) => {{
    bind_state!(st);
    MuGlue::new(MuGlue::spec_to_f32($spec, None)?)
  }};
}

#[macro_export]
macro_rules! DocType {
  ($rootelement:expr, $pubid:expr, $sysid:expr) => {
    bind_state_mut!(st);
    let mut namespaces: HashMap<String, String> = HashMap::new();
    DocType!($rootelement, $pubid, $sysid, namespaces, st)
  };
  ($rootelement:expr, $pubid:expr, $sysid:expr, $namespaces:expr, $state_arg:ident) => {{
    let mut model = &mut $state_arg.model;
    model.set_doc_type($rootelement.to_string(), $pubid.to_string(), $sysid.to_string());
    for (prefix, value) in $namespaces.iter() {
      model.register_document_namespace(prefix, Some(value.to_string()));
    }
  }};
}

#[macro_export]
macro_rules! Today {
  () => {{
    bind_state_mut!(st);
    today(st)
  }};
}

#[macro_export]
macro_rules! SetPrefix {
  ($prefix:literal) => {{
    bind_state_mut!(st);
    st.set_prefix($prefix);
  }};
}

#[macro_export]
macro_rules! DeclareOption {
  ($option:expr, None) => {
    bind_state_mut!(st);
    DeclareOption!($option, sub[stomach, state] {}, st)
  };
  (None, sub $body:block) => {
    bind_state_mut!(st);
    DeclareOption!(None, sub[stomach, state] $body, st)
  };
  (None, sub[$state:ident] $body:block) => {
    bind_state_mut!(st);
    DeclareOption!(None, sub[stomach, $state] $body, st)
  };
  (None, sub[$stomach:ident, $state:ident] $body:block) => {
    bind_state_mut!(st);
    DeclareOption!(None, sub[$stomach, $state] $body, st)
  };
  ($option:expr, sub $body:block) => {
    bind_state_mut!(st);
    DeclareOption!($option, sub[stomach, state] $body, st)
  };
  ($option:expr, sub[$state:ident] $body:block) => {
    bind_state_mut!(st);
    DeclareOption!($option, sub[stomach, $state] $body, st)
  };
  ($option:expr, sub[$stomach:ident, $state:ident] $body:block) => {
    bind_state_mut!(st);
    DeclareOption!($option, sub[$stomach, $state] $body, st)
  };
  (None, sub[$stomach:ident, $inner_state:ident] $body:block, $outer_state: ident) => {
    let cs = String::from("\\default@ds");
    // block case, create a primitive
    let code: PrimitiveClosure = Rc::new(move |$stomach, _args, $inner_state|
      WithInnerState!($body, $stomach, $inner_state).into_digested_result()
    );
    def_primitive(T_CS!(cs), None, code, PrimitiveOptions::default(), $outer_state);
  };
  ($option:expr, sub[$stomach:ident, $inner_state:ident] $body:block, $outer_state: ident) => {
    $outer_state.push_value("@declaredoptions", $option);
    let cs = s!("\\ds@{}", $option);
    // block case, create a primitive
    let code: PrimitiveClosure = Rc::new(move |$stomach, _args, $inner_state|
      WithInnerState!($body, $stomach, $inner_state).into_digested_result()
    );
    def_primitive(T_CS!(cs), None, code, PrimitiveOptions::default(), $outer_state);
  }
}

#[macro_export]
macro_rules! ProcessOptions {
  ($gullet:ident) => {{
    bind_state_mut!(st);
    process_options($gullet, st)?;
  }};
}

#[macro_export]
macro_rules! AddToMacro {
  ($cs:literal, $tokens:literal) => {{
    bind_state_mut!(stmch, st);
    let cs = T_CS!($cs);
    let tokens = TokenizeInternal!($tokens);
    // Needs error checking!
    let defn = st.lookup_definition(&cs);
    if defn.is_none() || !defn.as_ref().unwrap().is_expandable() {
      let message = s!("{} is not an expandable control sequence", cs);
      let message2 = "Ignoring addition";
      Warn!("unexpected", cs, stmch, st, message, message2);
    } else {
      let mut expansion = match defn.unwrap().get_expansion() {
        // the .clone() call is again avoidable with a careful refactor via e.g. using `.remove_definition` from state
        // (as we're redefining the macro again), and then use a `.remove_expansion` call on defn?
        Some(ExpansionBody::Tokens(tokens)) => tokens.clone().unlist(),
        Some(ExpansionBody::Closure(_)) => {
          let message = s!(
            "{} has a closure body, AddToMacro will *override* with an ExpandableBody::Tokens ! This is usually in error!",
            cs
          );
          Warn!("unexpected", "ExpandableBody::Closure", stmch, st, message);
          Vec::new()
        },
        None => Vec::new(),
      };
      expansion.extend(tokens.unlist());
      def_macro(
        cs,
        None,
        ExpansionBody::Tokens(Tokens!(expansion)),
        Some(ExpandableOptions {
          scope: Some(Scope::Global),
          ..ExpandableOptions::default()
        }),
        st,
      );
    }
  }};
}

#[macro_export]
macro_rules! BeginItemize {
  ($itype:literal, $counter:literal) => {{
    bind_state_mut!(stmch, st);
    begin_itemize($itype, Some($counter), false, stmch, st)
  }};
}

//
// Tricks:
// $(:)?$(=>)?   allow for any of "key:val", "key => val" and even "key :=> val".
//
#[macro_export]
macro_rules! defi_opts {
  // input is empty: time to output (with optional trailing comma allowed )
  (@munch ($(,)?) -> {$kind:ident,}) => {
    $kind::default()
  };
  (@munch ($(,)?) -> {$kind:ident, $([$id:ident @ $body:expr])+ } ) => {
    $kind {
      $($id: $body),*,
      ..$kind::default()
    }
  };
  // reversion: Option<Reversion>
  (@munch ( $(,)? reversion $(:)?$(=>)? $tokens:expr, $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )* [ reversion @ $tokens.into_option() ] })
  };
  (@munch ( $(,)? reversion $(:)?$(=>)? $tokens:expr) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ reversion @ $tokens.into_option() ] })
  };

  // sizer: Option<SizingClosure>
  (@munch ( $(,)? sizer $(:)?$(=>)? $tokens:expr, $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )* [ sizer @ $tokens.into_option() ] })
  };
  (@munch ( $(,)? sizer $(:)?$(=>)? $tokens:expr) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ sizer @ $tokens.into_option() ] })
  };
  // select: literal string
  (@munch ( $(,)? select $(:)?$(=>)? $tokens:expr, $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )* [ select @ $tokens.into_option() ] })
  };
  (@munch ( $(,)? select $(:)?$(=>)? $tokens:expr) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ select @ $tokens.into_option() ] })
  };
  // select: literal number
  (@munch ( $(,)? select_count $(:)?$(=>)? $tokens:expr, $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )* [ select_count @ $tokens.into_option() ] })
  };
  (@munch ( $(,)? select_count $(:)?$(=>)? $tokens:expr) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ select_count @ $tokens.into_option() ] })
  };
  // replace: sub
  (@munch ( $(,)? replace $(:)?$(=>)? sub $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@replace (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? replace $(:)?$(=>)? $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@replace ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };


  // mode : Option<TexMode>
  (@munch ( $(,)? mode $(:)?$(=>)? $literal:literal $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )* [ mode @ $literal.into_option() ] })
  };
  // alias : Option<String>
  (@munch ( $(,)? alias $(:)?$(=>)? $literal:literal $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )* [ alias @ $literal.into_option() ] })
  };
  // scope: Option<Scope>
  (@munch ( $(,)? scope $(:)?$(=>)? $scope:expr) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ scope @ $scope.into_option() ] })
  };
  (@munch ( $(,)? scope $(:)?$(=>)? $scope:expr, $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )* [ scope @ $scope.into_option() ] })
  };
  // font: Font
  (@munch ( $(,)? font $(:)?$(=>)? { $($fkey:ident => $fvalue:literal),* } $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )* [ font @ Font!($($fkey => $fvalue),*) ] })
  };
  (@munch ( $(,)? font $(:)?$(=>)? $props:ident $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )* [ font @ $props ] })
  };

  // properties: PropertiesClosure
  (@munch ( $(,)? properties $(:)?$(=>)? $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )* [ properties @ properties!($body) ] })
  };
  (@munch ( $(,)? properties $(:)?$(=>)?
      sub[$stomach_arg:ident, $args:ident, $state_arg:ident] $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )* [ properties @ properties!($stomach_arg, $args, $state_arg, $body) ] })
  };
  (@munch ( $(,)? properties $(:)?$(=>)? $var:ident $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )* [ properties @ properties!($var) ] })
  };

  // before_digest_end: Vec<BeforeDigestClosure>
  (@munch ( $(,)? before_digest_end $(:)?$(=>)? sub $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@before_digest_end (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? before_digest_end $(:)?$(=>)? $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@before_digest_end ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };


  // before_digest: Vec<BeforeDigestClosure>
  (@munch ( $(,)? before_digest $(:)?$(=>)? sub $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@before_digest (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? before_digest $(:)?$(=>)? $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@before_digest ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };

  // after_digest: Vec<DigestionClosure>
  (@munch ( $(,)? after_digest $(:)?$(=>)? sub $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@after_digest (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_digest $(:)?$(=>)? $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@after_digest ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };

  // after_digest_begin: Vec<DigestionClosure>
  (@munch ( $(,)? after_digest_begin $(:)?$(=>)? sub $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@after_digest_begin (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_digest_begin $(:)?$(=>)? $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@after_digest_begin ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };

  // before_construct: Vec<ConstructionClosure>
  (@munch ( $(,)? before_construct $(:)?$(=>)? sub $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@before_construct (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? before_construct $(:)?$(=>)? $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@before_construct ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };


  // after_construct: Vec<ConstructionClosure>
  (@munch ( $(,)? after_construct $(:)?$(=>)? sub $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@after_construct (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_construct $(:)?$(=>)? $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@after_construct ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_construct $(:)?$(=>)? $var:ident $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [after_construct @ $var]})
  };

  // getter: RegisterGetterClosure
  (@munch ( $(,)? getter $(:)?$(=>)? sub $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@getter (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? getter $(:)?$(=>)? $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@getter ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  // setter: RegisterSetterClosure
  (@munch ( $(,)? setter $(:)?$(=>)? sub $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@setter (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? setter $(:)?$(=>)? $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@setter ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  // after_open: Option<Vec<TagConstructionClosure>>
  (@munch ( $(,)? after_open $(:)?$(=>)? sub $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@after_open (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_open $(:)?$(=>)? $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@after_open ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  // after_open_late: Option<Vec<TagConstructionClosure>>
  (@munch ( $(,)? after_open_late $(:)?$(=>)? sub $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@after_open_late (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_open_late $(:)?$(=>)? $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@after_open_late ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  // after_close: Option<Vec<TagConstructionClosure>>
  (@munch ( $(,)? after_close $(:)?$(=>)? sub $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@after_close (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_close $(:)?$(=>)? $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@after_close ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  // auto_open: Option<bool>
  (@munch ( $(,)? auto_open $(:)?$(=>)? $auto:literal $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )* [ auto_open @ $auto.into() ]})
  };
  // auto_close: Option<bool>
  (@munch ( $(,)? auto_close $(:)?$(=>)? $auto:literal $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )* [ auto_close @ $auto.into() ]})
  };

  // misc "id" with literal value
  (@munch ( $(,)? $id:ident $(:)?$(=>)? $literal:literal $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [$id @ $literal]})
  };
  // misc "id" with block value
  (@munch ( $(,)? $id:ident $(:)?$(=>)? $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [$id @ $body]})
  };

  //-- aux
  // Closure parsers

  (@before_digest_end ($body:block $($next:tt)* )
                  -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [before_digest_end @ before_digest!($body)]})
  };
  (@before_digest_end (sub [$stomach_arg:ident, $state_arg: ident] $body:block $($next:tt)* )
                  -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [before_digest_end @ before_digest!($stomach_arg, $state_arg, $body)]})
  };

  (@before_digest (sub [$stomach_arg:ident, $state_arg: ident] $body:block $($next:tt)* )
                  -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [before_digest @ before_digest!($stomach_arg, $state_arg, $body)]})
  };
  (@before_digest ($body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [before_digest @ before_digest!($body)]})
  };
  (@after_digest (
    sub[$stomach_arg:ident, $whatsit:ident, $state_arg: ident] $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [after_digest @ after_digest!($stomach_arg, $whatsit, $state_arg, $body)]})
  };
  (@after_digest (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [after_digest @ after_digest!(stomach, whatsit, state, $body)]})
  };

  (@after_digest_begin (
    sub[$stomach_arg:ident, $whatsit:ident, $state_arg: ident] $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [after_digest_begin @ after_digest!($stomach_arg, $whatsit, $state_arg, $body)]})
  };
  (@after_digest_begin (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [after_digest_begin @ after_digest!(stomach, whatsit, state, $body)]})
  };

  (@before_construct (
    sub[$doc:ident, $whatsit:ident, $state_arg: ident] $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [before_construct @ construct!($doc, $whatsit, $state_arg, $body)]})
  };
  (@before_construct (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [before_construct @ construct!(document, whatsit, state, $body)]})
  };

  (@after_construct (
    sub[$doc:ident, $whatsit:ident, $state_arg: ident] $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [after_construct @ construct!($doc, $whatsit, $state_arg, $body)]})
  };
  (@after_construct (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [after_construct @ construct!(document, whatsit, state, $body)]})
  };

  (@getter (
    sub[$args:ident, $state_arg: ident] $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [getter @ getter!($args, $state_arg, $body)]})
  };
  (@getter (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [getter @ getter!(args, state, $body)]})
  };

  (@setter (
    sub[$value:ident, $args:ident, $state_arg: ident] $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [setter @ setter!($value, $args, $state_arg, $body)]})
  };
  (@setter (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [setter @ setter!(value, args, state, $body)]})
  };
  (@after_open (
    sub[$document:ident, $node:ident, $state_arg: ident] $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [after_open @ Some(tagsub!($document, $node, $state_arg, $body)) ]})
  };
  (@after_open (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [after_open @ Some(tagsub!(document, node, state, $body)) ]})
  };
  (@after_open_late (
    sub[$document:ident, $node:ident, $state_arg: ident] $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [after_open_late @ Some(tagsub!($document, $node, $state_arg, $body)) ]})
  };
  (@after_open_late (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [after_open_late @ Some(tagsub!(document, node, state, $body)) ]})
  };
  (@after_close (
    sub[$document:ident, $node:ident, $state_arg: ident] $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [after_close @ Some(tagsub!($document, $node, $state_arg, $body)) ]})
  };
  (@after_close (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [after_close @ Some(tagsub!(document, node, state, $body)) ]})
  };

  (@replace ($body:block $($next:tt)* )
                  -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [replace @ rewrite_replace_sub!($body)]})
  };
  (@replace (sub [$document_arg:ident, $node_arg:ident, $state_arg: ident] $body:block $($next:tt)* )
                  -> {$kind:ident, $([$key:ident @ $val:expr])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [replace @ rewrite_replace_sub!($document_arg, $node_arg, $state_arg, $body)]})
  };

}
