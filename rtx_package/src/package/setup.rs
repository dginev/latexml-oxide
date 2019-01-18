/// Macros and pool come at the end, so that they load seamlessly

// We need to invoke constructors within constructors. This is only possible with locally passed
// State arguments, IF we have a macro form that explicitly accepts state and has no pseudo-global
// $state in its initialization.

// We also can not decouple the macro variants with an explicit "state" argument,
// as decoupling requires a new macro rule, and respectively a new name.
// So DefMacro could be decoupled into DefMacro_State, which is terrible boilerplate
// and was in fact the first implementation attempt here.
// The current trade-off is to keep the macro variants tightly together, under the same name, and
// only create new names for new macro functions.

#[macro_export]
macro_rules! LoadDefinitions {
  ($outer_state:ident, $body:block) => {
    pub fn load_definitions($outer_state: &mut State, mut outer_stomach: Option<&mut Stomach>) -> Result<()> {
      BindState!($outer_state, outer_stomach);
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
  ($state:ident) => {
    BindState!($state, None)
  };
  ($outer_state:ident, $outer_stomach: expr) => {
    macro_rules! outer_state {
      () => {
        $outer_state
      };
    }
    macro_rules! outer_stomach {
      () => {
        $outer_stomach
      };
    }
  };
}

#[macro_export]
macro_rules! BindInnerState {
  ($state:ident) => {
    BindInnerState!($state, None)
  };
  ($inner_state:ident, $inner_stomach: expr) => {
    macro_rules! inner_state {
      () => {
        $inner_state
      };
    }
    macro_rules! inner_stomach {
      () => {
        $inner_stomach
      };
    }
  };
}

#[macro_export]
macro_rules! bind_state {
  ($st:ident) => {
    bind_state!($st, "outer")
  };
  ($st:ident, $location:expr) => {
    let $st: &mut State = {
      // TODO: If we can manage to make this attribute visible from **outside**,
      // in particular in macros such as beforeproc!() and beforesub!(), we can automate the inner state use
      // entirely
      //
      // #[bound_options(location = "inner")]
      //
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
    bind_state!(st);
    st.assign_mapping("PARAMETER_TYPES", $name, Some(Stored::Parameter($param)))
  };
  ($name:expr, $param:expr, $state_arg:ident) => {
    $state_arg.assign_mapping("PARAMETER_TYPES", $name, Some(Stored::Parameter($param)))
  };
}

#[macro_export]
macro_rules! LoadPool {
  ($name:expr) => {{
    bind_state!(st);
    LoadPool!($name, st)
  }};
  ($name:expr, $state_arg:ident) => {{
    input_definitions(
      $name,
      InputDefinitionOptions {
        extension: Some(String::from("pool")),
        with_stomach: match outer_stomach!().as_mut() {
          None => None,
          Some(st) => Some(st),
        },
        ..InputDefinitionOptions::default()
      },
      $state_arg,
    )?
  }};
}

/// Loader shorthand for pool dependencies
#[macro_export]
macro_rules! InnerPool {
  ($name:ident) => {{
    bind_state!(st);
    InnerPool!($name, st, outer_stomach!())
  }};
  ($name:ident, $state_arg:ident) => {
    InnerPool!($name, $state_arg, outer_stomach!())
  };
  ($name:ident, $state_arg:ident, $stomach:expr) => {{
    match $stomach.as_mut() {
      None => pool::$name::load_definitions($state_arg, None)?,
      Some(st) => pool::$name::load_definitions($state_arg, Some(st))?,
    }
  }};
}

#[macro_export]
macro_rules! RequirePackage {
  ($package:expr, $options:expr) => {{
    bind_state!(st);
    RequirePackage!($package, $options, st)
  }};
  ($package:expr, $options:expr, $state_arg:ident) => {
    require_package($package, $options, $state_arg)
  };
}
macro_rules! LoadClass {
  ($class:expr, $options:expr, $after:expr) => {
    load_class($class, $options, $after, state!())
  };
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
    bind_state!(st);
    DeclareFontMap!($name, $map, $family, st)
  }};
  ($name:expr, $map:expr) => {{
    bind_state!(st);
    DeclareFontMap!($name, $map, st)
  }};
}

#[macro_export]
macro_rules! DefMacroIWO {
  // closure stub
  ($cs:expr, $paramlist:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block, $options:expr) => {{

    bind_state!(st);
    DefMacroIWO!($cs, $paramlist, sub [ $gullet, $args, $inner_state ] $body, $options, st)
  }};
  // with explicit state
  ($cs:expr, $paramlist:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block, $options:expr, $state_arg:ident) => {{
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(move |$gullet, $args, $inner_state| $body)));
    def_macro($cs, $paramlist, expansion_closure, $options, $state_arg);
  }};
  // precompiled
  ($cs:expr, $paramlist:expr, $expansion:expr, $options:expr) => {{
    bind_state!(st);
    def_macro($cs, $paramlist, $expansion, $options, st)
  }};
  // with explicit state
  ($cs:expr, $paramlist:expr, $expansion:expr, $options:expr, $state_arg:ident) => {{
    def_macro($cs, $paramlist, $expansion, $options, $state_arg)
  }};
}

#[macro_export]
macro_rules! DefMacroWO {
  // Rust closure expansion form
  ($proto:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block, $options:expr, $state_arg:ident) => {{
    let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
    let expansion_body : Option<ExpansionBody> =
      Some(ExpansionBody::Closure(Rc::new(move |$gullet: &mut Gullet, $args: Vec<Tokens>, $inner_state:&mut State| $body)));
    // TODO: Also pass in options
    def_macro(cs, paramlist, expansion_body, $options, $state_arg);
  }};
  ($proto:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block, $options:expr) => ({
    bind_state!(st);
    DefMacroWO!($proto, sub [ $gullet, $args, $inner_state ] $body, $options, st)
  });
  // String expansion forms
  ($proto:expr, $expansion:expr, $options:expr) => {{
    bind_state!(st);
    DefMacroWO!($proto, $expansion, $options, st);
  }};
  ($proto:expr, $expansion:expr, $options:expr, $state_arg:ident) => ({
    let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
    let expansion;
    compile_expansion!(expansion, $expansion);
    def_macro(cs, paramlist, expansion, $options, $state_arg);
  });
}

#[macro_export]
macro_rules! DefConditional(
  // test is always a rust closure
  ($proto:expr, sub [$gullet:ident, $args:ident, $inner_state:ident] $body:block) => {{
    bind_state!(st);
    DefConditional!($proto, sub[$gullet, $args, $inner_state] $body, st);
  }};
  ($proto:expr, sub [$gullet:ident, $args:ident, $inner_state:ident] $body:block, $state_arg:ident) => ({
    let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
    DefConditionalI!(cs, paramlist, sub[$gullet, $args, $inner_state] $body, $state_arg)
  });
  // or None
  ($proto:expr) => {{
    bind_state!(st);
    DefConditional!($proto, None, st)
  }};
  ($proto:expr, None) => ({
    bind_state!(st);
    DefConditional!($proto, None, st)
  });
  ($proto:expr, None, $state_arg:ident) => ({
    let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
    DefConditionalI!(cs, paramlist, None, $state_arg)
  });
);

#[macro_export]
macro_rules! DefConditionalI(
  // test is always a rust closure
  ($cs:expr, $paramlist:expr, sub[$gullet:ident, $args:ident, $inner_state:ident] $body:block) => {{
    bind_state!(st);
    DefConditionalI!($cs, $paramlist, $gullet, $args, $inner_state, $body, st)
  }};
  ($cs:expr, $paramlist:expr, sub[$gullet:ident, $args:ident, $inner_state:ident] $body:block, $state_arg:ident) => ({
    let test : ConditionalClosure = Rc::new(move |$gullet, $args, $inner_state| {$body});
    def_conditional($cs, $paramlist, Some(test), ConditionalOptions::default(), $state_arg);
  });
  // or None
  ($cs:expr, $paramlist:expr, None) => {{
    bind_state!(st);
    DefConditionalI!($cs, $paramlist, None, st)
  }};
  ($cs:expr, $paramlist:expr, None, $state_arg:ident) => ({
    def_conditional($cs, $paramlist, None, ConditionalOptions::default(), $state_arg);
  });
);

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
///    isPrefix  : 1 for things like \global, \long, etc.
///    registerType : for parameters (but needs to be worked into `DefParameter`, below).

#[macro_export]
macro_rules! DefPrimitiveII {
  ($cs:expr, $paramlist:expr, sub[$stomach:ident,$args:ident,$inner_state:ident] $body:block) => {{
    bind_state!(st);
    DefPrimitiveII!($cs, $paramlist, sub[$stomach, $args, $inner_state] $body, st)
  }};
  ($cs:expr, $paramlist:expr, sub[$stomach:ident,$args:ident,$inner_state:ident] $body:block, $state_arg:ident) => {
    DefPrimitiveII!($cs, $paramlist, move |$stomach, $args, $inner_state| $body, PrimitiveOptions::default(), $state_arg)
  };
  ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options:expr) => {{
    bind_state!(st);
    DefPrimitiveII!($cs, $paramlist, $compiled_replacement, $options, st)
  }};
  ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => {{
    def_primitive($cs, $paramlist, Rc::new($compiled_replacement), $options, $state_arg);
  }};
}

#[macro_export]
macro_rules! DefPrimitiveIWO(
  ($proto:expr, $compiled_replacement:expr, $options:expr) => ({
    bind_state!(st);
    DefPrimitiveIWO!($proto, $compiled_replacement, $options, st)
  });
  ($proto:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => ({
    let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
    DefPrimitiveII!(cs, paramlist, $compiled_replacement, $options, $state_arg);
  })
);

#[macro_export]
macro_rules! DefRegisterWO {
  ($proto:expr, $value:expr, $options:expr) => {{
    let value = { $value }; // allow to re-borrow state in value macros
    bind_state!(st);
    DefRegisterWO!($proto, value, $options, st)
  }};
  ($proto:expr, $value:expr, $options:expr, $state_arg:ident) => {{
    let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
    DefRegisterI!(cs, paramlist, $value, $options, $state_arg);
  }};
}

#[macro_export]
macro_rules! DefRegisterI {
  ($cs:expr, $paramlist:expr, $value:expr, $($key:ident => $val:expr),*) =>
    (DefRegisterI!($cs, $paramlist, $value, Some(NewDefault!(RegisterOptions, $($key=>$val),*))));
  ($cs:expr, $paramlist:expr, $value:expr, $state_arg:ident, $($key:ident => $val:expr),*) =>
    (DefRegisterI!($cs, $paramlist, $value, Some(NewDefault!(RegisterOptions, $($key=>$val),*)), $state_arg));
  ($cs:expr, $paramlist:expr, $value:expr, $options:expr) => {{
    let value = { $value };
    bind_state!(st);
    DefRegisterI!($cs, $paramlist, value, $options, st)
  }};
  ($cs:expr, $paramlist:expr, $value:expr, $options:expr, $state_arg:ident) => {
    def_register($cs, $paramlist, $value, $options, $state_arg)
  };
}

#[macro_export]
macro_rules! LookupRegister {
  ($cs:expr) => {
    LookupRegister!($cs, Vec::new())
  };
  ($cs:expr, $parameters:expr) => {{
    bind_state!(st);
    LookupRegister!($cs, $parameters, st)
  }};
  ($cs:expr, $parameters:expr, $state_arg: ident) => {
    if let Some(defn) = $state_arg.lookup_register_definition(&T_CS!($cs)) {
      defn.value_of($parameters, $state_arg).unwrap_or_default()
    } else {
      warn!(target:"expected:register", "The control sequence {:?} is not a register", $cs);
      RegisterValue::default()
    }
  }
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

// sub AssignRegister {
//   my ($cs, $value, @parameters) = @_;
//   my $defn;
//   $cs = T_CS($cs) unless ref $cs;
//   if (($defn = $STATE->lookupDefinition($cs)) && $defn->isRegister) {
//     return $defn->setValue($value, @parameters); }
//   else {
//     Warn('expected', 'register', $STATE->getStomach,
//       "The control sequence " . ToString($cs) . " is not a register");
//     return; } }

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
macro_rules! DefConstructorIWO {
  ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options:expr) => {{
    bind_state!(st);
    DefConstructorIWO!($cs, $paramlist, $compiled_replacement, $options, st)
  }};
  ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => {{
    def_constructor($cs, $paramlist, $compiled_replacement, $options, $state_arg);
  }};
}

#[macro_export]
macro_rules! DefConstructorWO(
  ($proto:expr, $replacement:expr, $options:expr) => ({
    bind_state!(st);
    DefConstructorWO!($proto, $replacement, $options, st)
  });
  ($proto:expr, $replacement:expr, $options:expr, $state_arg:ident) => ({
    // check_options("DefConstructor (prototype)", $constructor_options, %options);
    let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
    let compiled_replacement;
    compile_replacement!(compiled_replacement, $replacement);
    DefConstructorIWO!(cs, paramlist, compiled_replacement, $options, $state_arg);
  });
  ($proto:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:block, $options:expr) => {{
    bind_state!(st);
    DefConstructorWO!($proto, $document, $args, $props, $inner_state, $body, $options, st)
  }};

  ($proto:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:block, $options:expr, $state_arg:ident) => ({
    let compiled_replacement : Option<ReplacementClosure> = Some(Rc::new(replacement!($document, $args, $props, $inner_state, $body)));
    let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
    DefConstructorIWO!(cs, paramlist, compiled_replacement, $options, $state_arg);
  });
  // pre-compiled CS with to-be-compiled replacement (see \begin{verbatim})
  (cs [ $cs:expr ], $paramlist:expr, $replacement:expr, $options:expr) => {{
    bind_state!(st);
    DefConstructorWO!(cs[$cs], $paramlist, $replacement, $options, st)
  }};
  (cs [ $cs:expr ], $paramlist:expr, $replacement:expr, $options:expr, $state_arg:ident) => ({
    let cs = T_CS!($cs);
    let compiled_replacement;
    compile_replacement!(compiled_replacement, $replacement);
    DefConstructorIWO!(cs, $paramlist, compiled_replacement, $options, $state_arg);
  })
);

#[macro_export]
macro_rules! TagWO {
  ($tag:expr, $properties:expr) => {{
    bind_state!(st);
    TagWO!($tag, $properties, st)
  }};
  ($tag:expr, $properties:expr, $state_arg:ident) => {
    install_tag($tag, $properties, $state_arg)
  };
}
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
    bind_state!(st);
    DefEnvironmentWO!($proto_raw, $replacement, $options, st)
  }};
  ($proto_raw:expr, $replacement:expr, $options:expr, $state_arg:ident) => ({
  use rtx_core::util::text::*;
  let mut proto = $proto_raw.to_string().trim_start().to_string();
  let name = extract_bracketed(&mut proto, Some(&Delimiter::Brace));

  let compiled_replacement;
  compile_replacement!(compiled_replacement, $replacement);

  let options = $options;
  def_environment(name, None, compiled_replacement, options, $state_arg);
}));

#[macro_export]
macro_rules! DefEnvironmentCWO (
  ($proto_raw:expr, $compiled_replacement:expr, $options:expr) => {{
    bind_state!(st);
    DefEnvironmentCWO!($proto_raw, $compiled_replacement, $options, st)
  }};
  ($proto_raw:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => ({
  use rtx_core::util::text::*;
  let mut proto = $proto_raw.to_string().trim_start().to_string();
  let name = extract_bracketed(&mut proto, Some(&Delimiter::Brace));
  // TODO: What do we do with param lists?
  //let paramlist_str = proto.trim_start().to_string();
  def_environment(name, None, $compiled_replacement, $options, $state_arg);
}));

#[macro_export]
macro_rules! RelaxNGSchema {
  ($name:expr) => {{
    bind_state!(st);
    RelaxNGSchema!($name, st)
  }};
  ($name:expr,$state_arg:ident) => {
    select_relaxng_schema($name.to_string(), None, $state_arg)
  };
}

#[macro_export]
macro_rules! RegisterNamespace(
  ($prefix:expr, $namespace:expr) => {{
    bind_state!(st);
    RegisterNamespace!($prefix, $namespace, st)
  }};
  ($prefix:expr, $namespace:expr,$state_arg:ident) =>
    ($state_arg.model.register_namespace($prefix, Some($namespace.to_string())))
);
#[macro_export]
macro_rules! RegisterDocumentNamespace(
  ($prefix:expr, $namespace:expr) => {{
    bind_state!(st);
    RegisterDocumentNamespace!($prefix, $namespace, st)
  }};
  ($prefix:expr, $namespace:expr,$state_arg:ident) =>
    ($state_arg.model.register_document_namespace($prefix, Some($namespace.to_string())))
);
#[macro_export]
macro_rules! RequireResource(
  ($resource:expr) => {{
    bind_state!(st);
    RequireResource!($resource, st)
  }};
  ($resource:expr,$state_arg:ident) =>
    (require_resource(Resource{name: $resource.to_string(), ..Resource::default()}, $state_arg))
);

// sub DefMath {
//   my ($proto,
//     $presentation, %options) = @_;
//   CheckOptions("DefMath ($proto)", $math_options, %options);
//   DefMathI(parsePrototype($proto), $presentation, %options);
//   return; }
#[macro_export]
macro_rules! DefMathWO {
  ($cstext:expr, $paramlist:expr, $presentation:expr, $options:expr) => {{
    bind_state!(st);
    DefMathWO!($cstext, $paramlist, $presentation, $options, st)
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
    bind_state!(st);
    requireMath!($cs_name, st)
  }};
  ($cs_name:expr, $state_arg:ident) => (
    if !LookupBool!("IN_MATH", $state_arg) {
      warn!(target: "unexpected", "{} should only appear in math mode",$cs_name);
    }
  )
}
#[macro_export]
macro_rules! forbidMath {
  ($cs_name:expr) => ({
    bind_state!(st);
    forbidMath!($cs_name, st)
  });
  ($cs_name:expr, $state_arg:ident) => (
    if LookupBool!("IN_MATH", $state_arg) {
      warn!(target: "unexpected", "{} should not appear in math mode",$cs_name);
    }
  )
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
    bind_state!(st);
    new_counter($ctr, $within, None, st)?
  };
  ($ctr:expr, $within:expr, None, $state_arg:ident) => {
    new_counter($ctr, $within, None, $state_arg)?
  };
  ($ctr:expr, $within:expr, Some($opts:expr)) => {
    bind_state!(st);
    new_counter($ctr, $within, Some($opts), st)?
  };
  ($ctr:expr, $within:expr, Some($opts:expr), $state_arg:ident) => {
    new_counter($ctr, $within, Some($opts), $state_arg)?
  };
}
#[macro_export]
macro_rules! CounterValue {
  ($ctr:expr) => {
    bind_state!(st);
    counter_value($ctr, st)
  };
  ($ctr:expr, $state_arg:ident) => {
    counter_value($ctr, $state_arg)
  };
}
#[macro_export]
macro_rules! SetCounter {
  ($ctr:expr, $value:expr, None) => {
    AssignValue!(&s!("\\c@{}",$ctr), $value, Some(Scope::Global));
    DefMacroI!(T_CS!(s!("\\@{}@ID",$ctr)), None, Tokens::new(Explode!($value.value_of())),
                scope => Some(Scope::Global)
    );
  };
  ($ctr:expr, $value:expr, $gullet:ident) => {
    AssignValue!(&s!("\\c@{}",$ctr), $value, Some(Scope::Global));
    AfterAssignment!();
    DefMacroI!(T_CS!(s!("\\@{}@ID",$ctr)), None, Tokens::new(Explode!($value.value_of())),
                scope => Some(Scope::Global)
    );
  }
}
#[macro_export]
macro_rules! AddToCounter {
  ($ctr:expr, $value:expr, $gullet:ident) => {
    bind_state!(st);
    add_to_counter($ctr, $value, $gullet, st)
  };
  ($ctr:expr, $value:expr, $gullet:ident, $state_arg:ident) => {
    add_to_counter($ctr, $value, $gullet, $state_arg)
  };
}
#[macro_export]
macro_rules! StepCounter {
  ($ctr:expr, $noreset:expr, $gullet:ident) => {
    bind_state!(st);
    step_counter($ctr, $noreset, $gullet, st)
  };
  ($ctr:expr, $noreset:expr, $gullet:ident, $state_arg:ident) => {
    step_counter($ctr, $noreset, $gullet, $state_arg)
  };
}
#[macro_export]
macro_rules! RefStepCounter {
  ($ctr:expr, $noreset:expr, $stomach:ident) => {
    ref_step_counter($ctr, $noreset, $stomach, state!())
  };
  ($ctr:expr, $noreset:expr, $stomach:ident, $state_arg:ident) => {
    ref_step_counter($ctr, $noreset, $stomach, $state_arg)
  };
}
#[macro_export]
macro_rules! RefStepID {
  ($ctr:expr, $stomach:ident) => {
    ref_step_id($ctr, $stomach, state!())
  };
  ($ctr:expr, $stomach:ident, $state_arg:ident) => {
    ref_step_id($ctr, $stomach, $state_arg)
  };
}
#[macro_export]
macro_rules! ResetCounter {
  ($ctr:expr) => {
    reset_counter($ctr, state!())
  };
  ($ctr:expr, $state_arg: ident) => {
    reset_counter($ctr, $state_arg)
  };
}

/// Return $tokens with all tokens expanded
#[macro_export]
macro_rules! Expand {
  ($tokens:expr, $gullet:ident) => {
    do_expand($tokens, $gullet, state!())?
  };
  ($tokens:expr, $gullet:ident, $state_arg:ident) => {
    do_expand($tokens, $gullet, $state_arg)?
  };
}

/// Invocation(<list of Token>); builds a representation of a command sequence invoked on its
/// arguments
#[macro_export]
macro_rules! Invocation {
  ($csname:literal, $args:expr, $gullet:expr) => {
    bind_state!(st);
    Invocation!(T_CS!($csname), $args, $gullet, st)
  };
  ($csname:literal, $args:expr, $gullet:expr, $state_arg:ident) => {
    Invocation!(T_CS!($csname), $args, $gullet, $state_arg)
  };
  ($token:expr, $args:expr, $gullet:expr) => {
    bind_state!(st);
    Invocation!($token, $args, $gullet, st)
  };
  ($token:expr, $args:expr, $gullet:expr, $state_arg:ident) => {
    build_invocation($token, $args.into_iter().map(|arg| arg.into()).collect(), $gullet, $state_arg)
  };
}
#[macro_export]
macro_rules! DefLigature {
  ($regex:expr, $replacement:expr, fontTest => sub[$font:ident] $body:block) => {
    bind_state!(st);
    DefLigature!($regex, $replacement, fontTest => sub[$font]{$body}, st)
  };
  ($regex:expr, $replacement:expr, fontTest => sub[$font:ident] $body:block, $state_arg:ident) => {
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
    bind_state!(st);
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
  ($accent:expr, $combiningchar:expr, $standalonechar:expr) => {
    let mut empty_opts : HashMap<String, Stored> = HashMap::new();
    bind_state!(st);
    DefAccent!($accent, $combiningchar, $standalonechar, empty_opts, st)
  };
  ($accent:expr, $combiningchar:expr, $standalonechar:expr, below => true) => {{
    bind_state!(st);
    DefAccent!($accent, $combiningchar, $standalonechar, map!("below"=>Stored::Bool(true)), st)
  }};
  ($accent:expr, $combiningchar:expr, $standalonechar:expr, $options:expr) => {{
    bind_state!(st);
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
    DefPrimitive!(&format!("{}{{}}",$accent), sub[stomach, letter, inner_state] {
      let invoked = Invocation!(T_CS!($accent), letter.clone(), stomach.get_gullet_mut(), inner_state)?;
      // TODO: check if letter.to_string has artefacts
      crate::package::pool::tex_accents::apply_accent(
        stomach, &letter[0].to_string(), $combiningchar, $standalonechar, Some(invoked), inner_state)?;
      Ok(vec![])
    }, mode => Some(String::from("text")));
  }}
}

//============================================
// User-facing Macros
//============================================
//
#[macro_export]
macro_rules! LookupValue {
  ($name:expr) => {{
    bind_state!(st);
    LookupValue!($name, st)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.lookup_value($name)
  };
}
#[macro_export]
macro_rules! LookupBool {
  ($name:expr) => {{
    {
      bind_state!(st);
      LookupBool!($name, st)
    }
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.lookup_bool($name)
  };
}
#[macro_export]
macro_rules! LookupString {
  ($name:expr) => {{
    bind_state!(st);
    LookupString!($name, st)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.lookup_string($name)
  };
}
#[macro_export]
macro_rules! LookupNumber {
  ($name:expr) => {{
    bind_state!(st);
    LookupNumber!($name, st)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.lookup_number($name)
  };
}
#[macro_export]
macro_rules! LookupTokens {
  ($name:expr) => {{
    bind_state!(st);
    LookupTokens!($name, st)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.lookup_tokens($name)
  };
}
#[macro_export]
macro_rules! AssignValue {
  ($name:expr, $value:expr) => {
    bind_state!(st);
    AssignValue!($name, $value, None, st)
  };
  ($name:expr, $value:expr, $scope:expr) => {
    bind_state!(st);
    AssignValue!($name, $value, $scope, st)
  };
  ($name:expr, $value:expr, $scope:expr, $state_arg:ident) => {
    $state_arg.assign_value($name, $value, $scope)
  };
}
#[macro_export]
macro_rules! RemoveValue {
  ($name:expr) => {{
    bind_state!(st);
    RemoveValue!($name, st)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.remove_value($name)
  };
}
#[macro_export]
macro_rules! PushValue {
  ($name:expr, $values:expr) => {{
    bind_state!(st);
    PushValue!($name, $values, st)
  }};
  ($name:expr, $values:expr, $state_arg:ident) => {
    $state_arg.push_value($name, $values)
  };
}
#[macro_export]
macro_rules! PopValue {
  ($name:expr) => {{
    bind_state!(st);
    PopValue!($name, st)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.pop_value($name)
  };
}
#[macro_export]
macro_rules! UnshiftValue {
  ($name:expr, $values:expr) => {{
    bind_state!(st);
    UnshiftValue!($name, $values, st)
  }};
  ($name:expr, $values:expr,$state_arg:ident) => {
    $state_arg.unshift_value($name, $values)
  };
}
#[macro_export]
macro_rules! ShiftValue {
  ($name:expr) => {{
    bind_state!(st);
    ShiftValue!($name, st)
  }};
  ($name:expr,$state_arg:ident) => {
    $state_arg.shift_value($name)
  };
}
#[macro_export]
macro_rules! LookupMapping {
  ($map:expr, $key:expr) => {{
    bind_state!(st);
    LookupValue!($map, $key, st)
  }};
  ($map:expr, $key:expr, $state_arg:ident) => {
    $state_arg.lookup_mapping($map, $key)
  };
}
#[macro_export]
macro_rules! AssignMapping {
  ($map:expr, $key:expr => $value:expr) => {{
    bind_state!(st);
    AssignMapping!($map, $key => $value, st)
  }};
  ($map:expr, $key:expr => $value:expr, $state_arg:ident) => {
    $state_arg.assign_mapping($map, $key, $value.into())
  };
}
#[macro_export]
macro_rules! LookupMappingKeys {
  ($map:expr) => {{
    bind_state!(st);
    LookupMappingKeys!($map, st)
  }};
  ($map:expr, $state_arg:ident) => {
    $state_arg.lookup_mapping_keys($map)
  };
}
#[macro_export]
macro_rules! LookupCatcode {
  ($char:expr) => {{
    bind_state!(st);
    LookupCatcode!($char, st)
  }};
  ($char:expr, $state_arg:ident) => {
    $state_arg.lookup_catcode($char)
  };
}
#[macro_export]
macro_rules! AssignCatcode {
  ($char:expr, $catcode:expr, $scope:expr) => {{
    bind_state!(st);
    AssignCatcode!($char, $catcode, $scope, st)
  }};
  ($char:expr, $catcode:expr, $scope:expr, $state_arg:ident) => {
    $state_arg.assign_catcode($char, $catcode, $scope)
  };
}
#[macro_export]
macro_rules! LookupMeaning {
  ($name:expr) => {{
    bind_state!(st);
    LookupMeaning!($name, st)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.lookup_meaning($name)
  };
}
#[macro_export]
macro_rules! LookupDefinition {
  ($name:expr) => {{
    bind_state!(st);
    LookupDefinition!($name, st)
  }};
  ($name:expr, $state_arg:ident) => {
    $state_arg.lookup_definition($name)
  };
}
#[macro_export]
macro_rules! InstallDefinition {
  ($name:expr, $definition:expr, $scope:expr) => {{
    bind_state!(st);
    InstallDefinition!($name, $definition, $scope, st)
  }};
  ($name:expr, $definition:expr, $scope:expr, $state_arg:ident) => {
    $state_arg.install_definition($name, $definition, $scope)
  };
}
#[macro_export]
macro_rules! XEquals {
  ($token1:expr, $token2:expr) => {{
    bind_state!(st);
    XEquals!($token1, $token2, st)
  }};
  ($token1:expr, $token2:expr, $state_arg:ident) => {
    $state_arg.x_equals($token1, $token2)
  };
}
#[macro_export]
macro_rules! IsDefined {
  ($name:expr) => {{
    bind_state!(st);
    IsDefined!($name, st)
  }};
  ($name:expr, $state_arg:ident) => {
    is_defined_token($name, $state_arg)
  };
}
#[macro_export]
macro_rules! IsDefinedToken {
  ($name:expr) => {{
    bind_state!(st);
    is_defined_token($name, st)
  }};
}
#[macro_export]
macro_rules! Let {
  ($token1:expr, $token2:expr) => {{
    bind_state!(st);
    Let!($token1, $token2, st)
  }};
  ($token1:expr, $token2:expr, $state_arg:ident) => {{
    LetI!(&T_CS!($token1), T_CS!($token2), $state_arg)
  }};
  ($token1:expr, $token2:expr, $scope:expr, $state_arg:ident) => {{
    LetI!(&T_CS!($token1), T_CS!($token2), $scope, $state_arg)
  }};
}
#[macro_export]
macro_rules! LetI {
  ($token1:expr, $token2:expr) => {{
    bind_state!(st);
    LetI!($token1, $token2, st)
  }};
  ($token1:expr, $token2:expr, $state_arg:ident) => {
    $state_arg.let_i($token1, $token2, None)
  };
  ($token1:expr, $token2:expr, $scope:expr, $state_arg:ident) => {
    $state_arg.let_i($token1, $token2, $scope)
  };
}
#[macro_export]
macro_rules! DigestIf {
  ($token:literal, $stomach:ident) => {{
    bind_state!(st);
    DigestIf!(T_CS!($token), $stomach, st)
  }};
  ($token:literal, $stomach:ident, $state_arg:ident) => {
    digest_if(T_CS!($token), $stomach, $state_arg)
  };
  ($token:expr, $stomach:ident) => {{
    bind_state!(st);
    DigestIf!($token, $stomach, st)
  }};
  ($token:expr, $stomach:ident, $state_arg: ident) => {
    digest_if($token, $stomach, $state_arg)
  };
}
#[macro_export]
macro_rules! AfterAssignment {
  () => {{
    bind_state!(st);
    AfterAssignment!(st)
  }};
  ($state_arg: ident) => {
    $state_arg.after_assignment()
  };
}

// Merge the current font with the style specifications
#[macro_export]
macro_rules! MergeFont {
  ($kv:expr) => {{
    bind_state!(st);
    MergeFont!($kv, st)
  }};
  ($kv:expr, $state_arg:ident) => {
    merge_font($kv, $state_arg)
  };
  ($key:ident => $val:expr) => {{
    bind_state!(st);
    MergeFont!($key => $val, st)
  }};
  ($key:ident => $val:expr, $state_arg:ident) => {
    merge_font(&fontmap!($key => $val), $state_arg)
  };
}

//============================================
// User-facing Argument Parsers, delegating to the stateful *WO variants
//============================================
//

#[macro_export]
macro_rules! DefMacroI(
  // Expansion closure syntax
  ($cs:expr, $paramlist:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block) =>
    (DefMacroIWO!($cs, $paramlist, sub [ $gullet, $args, $inner_state ] $body, None));
  // With explicit state
  ($cs:expr, $paramlist:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block, $state_arg:ident) =>
    (DefMacroIWO!($cs, $paramlist, sub [ $gullet, $args, $inner_state ] $body, None, $state_arg));
  ($cs:expr, $paramlist:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block, $state_arg:ident, $(key:ident=>$val:expr),*) =>
    (DefMacroIWO!($cs, $paramlist, sub [ $gullet, $args, $inner_state ] $body, Some(NewDefaultV!(ExpandableOptions, $($key=>$val),*)), $state_arg));

  // Simple Expression syntax
  ($cs:expr, $paramlist:expr, $expansion:expr) => (DefMacroIWO!($cs, $paramlist, $expansion, None));
  ($cs:expr, $paramlist:expr, $expansion:expr, $($key:ident=>$val:expr),*) =>
    (DefMacroIWO!($cs, $paramlist, $expansion, Some(NewDefaultV!(ExpandableOptions, $($key=>$val),*))));
  // Explicit state
  ($cs:expr, $paramlist:expr, $expansion:expr, $state_arg:ident) => (DefMacroIWO!($cs, $paramlist, $expansion, None, $state_arg));
  ($cs:expr, $paramlist:expr, $expansion:expr, $state_arg:ident, $($key:ident=>$val:expr),*) =>
    (DefMacroIWO!($cs, $paramlist, $expansion, Some(NewDefaultV!(ExpandableOptions, $($key=>$val),*), $state_arg)));
);

#[macro_export]
macro_rules! DefMacro {
  // closure
  ($proto:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block) =>
    (DefMacroWO!($proto, sub[$gullet, $args, $inner_state] $body, None));
  ($proto:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block, $($key:ident=>$val:expr),*) =>
    (DefMacroWO!($proto, sub[$gullet, $args, $inner_state] $body, Some(NewDefaultV!(ExpandableOptions, $($key=>$val),*))));
  // String form
  ($proto:expr, $expansion:expr) => (DefMacroWO!($proto, $expansion, None));
  ($proto:expr, $expansion:expr, $($key:ident=>$val:expr),*) =>
    (DefMacroWO!($proto, $expansion, Some(NewDefaultV!(ExpandableOptions, $($key=>$val),*))));

  // closure; explicit state
  ($proto:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block, $state_arg:ident) =>
    (DefMacroWO!($proto, sub[$gullet, $args, $inner_state] $body, None, $state_arg));
  // string; explicit state
  ($proto:expr, $expansion:expr, $state_arg:ident) => (DefMacroWO!($proto, $expansion, None, $state_arg));
  ($proto:expr, $expansion:expr, $state_arg:ident, $($key:ident=>$val:expr),*) =>
    (DefMacroWO!($proto, $expansion, Some(NewDefault!(ExpandableOptions, $($key=>$val),*), $state_arg)));
}

#[macro_export]
macro_rules! DefRegister {
  ($proto:expr, $value:expr) => (DefRegisterWO!($proto, $value, None));
  ($proto:expr, $value:expr, $state_arg: ident) => (DefRegisterWO!($proto, $value, None, $state_arg));
  ($proto:expr, $value:expr, $($key:ident => $val:expr),*) => (DefRegisterWO!($proto, $value, Some(NewDefault!(RegisterOptions, $($key=>$val),*))));
  ($proto:expr, $value:expr, $state_arg:ident, $($key:ident => $val:expr),*) =>
    (DefRegisterWO!($proto, $value, Some(NewDefault!(RegisterOptions, $($key=>$val),*)), $state_arg));
}

#[macro_export]
macro_rules! DefConstructorI {
  ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block) =>
    (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(replacement!($document, $args, $props, $inner_state, $body))), ConstructorOptions::default()));
  ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block, $($key:ident => $val:expr),*) =>
    (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(replacement!($document, $args, $props, $inner_state, $body))),
      NewDefault!(ConstructorOptions, $($key=>$val),*)));
  // None replacement
  ($cs:expr, $paramlist:expr, None) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(noreplacement!())), NewDefault!(ConstructorOptions)));
  ($cs:expr, $paramlist:expr, None, $($key:ident => $val:expr),*) =>
    (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(noreplacement!())), NewDefault!(ConstructorOptions, $($key=>$val),*)));

  // with explicit state
  ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
    $state_arg:ident) =>
    (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(replacement!($document, $args, $props, $inner_state, $body))),
      ConstructorOptions::default(), $state_arg));
  ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
    $state_arg:ident, $($key:ident => $val:expr),*) =>
    (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(replacement!($document, $args, $props, $inner_state, $body))),
      NewDefault!(ConstructorOptions, $($key=>$val),*), $state_arg));
  // None replacement
  ($cs:expr, $paramlist:expr, None, $state_arg:ident) =>
    (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(noreplacement!())), NewDefault!(ConstructorOptions), $state_arg));
  ($cs:expr, $paramlist:expr, None, $state_arg:ident, $($key:ident => $val:expr),*) =>
    (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(noreplacement!())), NewDefault!(ConstructorOptions, $($key=>$val),*), $state_arg))
}

#[macro_export]
macro_rules! DefConstructor {
  // with implicit state
  // Closure replacement flavors
  ($proto:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block) =>
    (DefConstructorWO!($proto, $document, $args, $props, $inner_state, $body, ConstructorOptions::default()));
  ($proto:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block, $($key:ident => $val:expr),*) =>
    (DefConstructorWO!($proto, $document, $args, $props, $inner_state, $body, NewDefault!(ConstructorOptions, $($key=>$val),*)));
  // String replacement flavors
  ($cs:expr, $replacement:expr) => (DefConstructorWO!($cs, $replacement, ConstructorOptions::default()));
  ($cs:expr, $replacement:expr, $($key:ident => $val:expr),*) =>
    (DefConstructorWO!($cs, $replacement, NewDefault!(ConstructorOptions, $($key => $val),*)));
  // pre-compiled CS with to-be-compiled replacement, (see \begin{verbatim})
  (cs [ $cs:expr ], $paramlist:expr, $replacement:expr) =>
    (DefConstructorWO!(cs[$cs], $paramlist, $replacement, ConstructorOptions::default()));
  (cs [ $cs:expr ], $paramlist:expr, $replacement:expr, $($key:ident => $val:expr),*) =>
    (DefConstructorWO!(cs[$cs], $paramlist, $replacement, NewDefault!(ConstructorOptions, $($key => $val),*)));

  // with explicit state
  // Closure replacement flavors
  ($proto:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block, $state_arg:ident) => (
    DefConstructorWO!($proto, $document, $args, $props, $inner_state, $body, ConstructorOptions::default(), $state_arg));
  ($proto:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block, $state_arg:ident, $($key:ident => $val:expr),*) =>
    (DefConstructorWO!($proto, $document, $args, $props, $inner_state, $body, NewDefault!(ConstructorOptions, $($key=>$val),*), $state_arg));
  // String replacement flavors
  ($cs:expr, $replacement:expr, $state_arg:ident) => (DefConstructorWO!($cs, $replacement, ConstructorOptions::default(), $state_arg));
  ($cs:expr, $replacement:expr, $state_arg:ident, $($key:ident=>$val:expr),*) =>
    (DefConstructorWO!($cs, $replacement, NewDefault!(ConstructorOptions, $($key => $val),*), $state_arg));
  // pre-compiled CS with to-be-compiled replacement, (see \begin{verbatim})
  (cs [ $cs:expr ], $paramlist:expr, $replacement:expr, $state_arg:ident) =>
    (DefConstructorWO!(cs[$cs], $paramlist, $replacement, ConstructorOptions::default(), $state_arg));
  (cs [ $cs:expr ], $paramlist:expr, $replacement:expr, $state_arg:ident, $($key:ident => $val:expr),*) =>
    (DefConstructorWO!(cs[$cs], $paramlist, $replacement, NewDefault!(ConstructorOptions, $($key => $val),*), $state_arg));
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
macro_rules! DefEnvironment(
  // implicit state
  ($proto_raw:expr, $replacement:expr) => (DefEnvironmentWO!($proto_raw, $replacement, ConstructorOptions::default()));
  ($proto_raw:expr, $replacement:expr, $($key:ident => $val:expr),*) =>
    (DefEnvironmentWO!($proto_raw, $replacement, NewDefault!(ConstructorOptions, $($key => $val),*)));
  // explicit state
  ($proto_raw:expr, $replacement:expr, $state_arg:ident) => (DefEnvironmentWO!($proto_raw, $replacement, ConstructorOptions::default(), $state_arg));
  ($proto_raw:expr, $replacement:expr, $($key:ident => $val:expr),*, $state_arg:ident) =>
    (DefEnvironmentWO!($proto_raw, $replacement, NewDefault!(ConstructorOptions, $($key => $val),*, $state_arg)));
);

#[macro_export]
macro_rules! DefEnvironmentC(
  // implicit state
  ($proto_raw:expr, $compiled_replacement:expr) => (DefEnvironmentCWO!($proto_raw, $paramlist, $compiled_replacement, ConstructorOptions::default()));
  ($proto_raw:expr, $compiled_replacement:expr, $($key:ident=>$val:expr),*) =>
    (DefEnvironmentCWO!($proto_raw, $compiled_replacement, NewDefault!(ConstructorOptions, $($key => $val),*)));
  // explicit state
  ($proto_raw:expr, $compiled_replacement:expr, $state_arg:ident) =>
    (DefEnvironmentCWO!($proto_raw, $paramlist, $compiled_replacement, ConstructorOptions::default(), $state_arg));
  ($proto_raw:expr, $compiled_replacement:expr, $($key:ident=>$val:expr),*, $state_arg:ident) =>
    (DefEnvironmentCWO!($proto_raw, $compiled_replacement, NewDefault!(ConstructorOptions, $($key => $val),*), $state_arg));
);

#[macro_export]
macro_rules! DefPrimitive{
  ($proto:expr, sub[$stomach:ident, $whatsit:ident, $inner_state:ident] $body:block) =>
    (DefPrimitiveIWO!($proto, |$stomach, $whatsit, $inner_state| {
      BindInnerState!($inner_state, $stomach); $body}, PrimitiveOptions::default()));
  ($proto:expr, sub[$stomach:ident, $whatsit:ident, $inner_state:ident] $body:block, $($key:ident=>$val:expr),*) =>
    (DefPrimitiveIWO!($proto, |$stomach, $whatsit, $inner_state| {
      BindInnerState!($inner_state, $stomach); $body}, NewDefault!(PrimitiveOptions, $($key=>$val),*)));
  ($proto:expr, $replacement:expr, $options:expr) => ({
    // TODO:
    // let compiled_replacement = || Tbox{text: $replacement, Invocation($options{alias} || $cs, @_[1 .. $#_])); }
    let compiled_replacement = $replacement;
    DefPrimitiveIWO!($proto, compiled_replacement, $options);
  });

  // explicit state
  ($proto:expr, sub[$stomach:ident, $whatsit:ident, $inner_state:ident] $body:block, $state_arg:ident) =>
    (DefPrimitiveIWO!($proto, |$stomach, $whatsit, $inner_state| {$body}, PrimitiveOptions::default(), $state_arg));
  ($proto:expr, sub[$stomach:ident, $whatsit:ident, $inner_state:ident] $body:block, $state_arg:ident, $($key:ident=>$val:expr),*) =>
    (DefPrimitiveIWO!($proto, |$stomach, $whatsit, $inner_state| {$body}, NewDefault!(PrimitiveOptions, $($key=>$val),*), $state_arg));

  ($proto:expr, $replacement:expr, $options:expr, $state_arg:ident) => ({
    // TODO:
    // let compiled_replacement = || Tbox{text: $replacement, Invocation($options{alias} || $cs, @_[1 .. $#_])); }
    let compiled_replacement = $replacement;
    DefPrimitiveIWO!($proto, compiled_replacement, $options, $state_arg);
  });
}

#[macro_export]
macro_rules! DefPrimitiveI{
  ($proto:expr, $compiled_replacement:expr) => (DefPrimitiveIWO!($proto, $compiled_replacement, PrimitiveOptions::default()));
  ($proto:expr, $compiled_replacement:expr, $($key:ident=>$val:expr),*) =>
    (DefPrimitiveIWO!($proto, $compiled_replacement, NewDefault!(PrimitiveOptions, $($key => $val),*)));
  // explicit state
  ($proto:expr, $compiled_replacement:expr, $state_arg:ident) =>
    (DefPrimitiveIWO!($proto,$compiled_replacement, PrimitiveOptions::default(), $state_arg));
  ($proto:expr, $compiled_replacement:expr, $state_arg:ident, $($key:ident=>$val:expr),*) =>
    (DefPrimitiveIWO!($proto, $compiled_replacement, NewDefault!(PrimitiveOptions, $($key => $val),*), $state_arg));
}

#[macro_export]
macro_rules! Tag(
  ($tag:expr,$($key:ident => $val:expr),*) =>
    (TagWO!($tag, NewDefault!(TagOptions, $($key => Some($val)),*)));
  ($tag:expr,$($key:ident => $val:expr),*, $state_arg:ident) =>
    (TagWO!($tag, NewDefault!(TagOptions, $($key => Some($val)),*), $state_arg));
);

#[macro_export]
macro_rules! DefMathI(
  ($text:expr,$paramlist:expr,$presentation:expr) => (
    DefMathWO!($text,$paramlist, $presentation, MathPrimitiveOptions::default()));
  ($text:expr,$paramlist:expr,$presentation:expr, $($key:ident => $val:expr),*) => (
    DefMathWO!($text,$paramlist, $presentation, NewDefaultV!(MathPrimitiveOptions, $($key => $val),*)));
  ($text:expr,$paramlist:expr,$presentation:expr, $($key:ident => $val:expr),*, $state_arg:ident) => (
    DefMathWO!($text,$paramlist, $presentation, NewDefaultV!(MathPrimitiveOptions, $($key => $val),*,$state_arg)));
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
  ($thing:expr) => {
    $thing.revert()?.unlist()
  };
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

macro_rules! Digest {
  ($tokens:expr) => {{
    bind_state!(st);
    Digest!($tokens, st)
  }};
  ($tokens:expr, $state_arg:ident) => {{
    let mut state_stomach = $state_arg.stomach.clone();
    match outer_stomach!().as_mut() {
      Some(st) => (*st).digest($tokens, $state_arg),
      None => state_stomach.borrow_mut().digest($tokens, $state_arg),
    }
  }};
}

macro_rules! DigestText {
  ($tokens:expr) => {
    bind_state!(st);
    let mut state_stomach = st.stomach.clone();
    match outer_stomach!().as_mut() {
      Some(st) => digest_text($tokens, *st, st),
      None => digest_text($tokens, state_stomach.borrow_mut(), st),
    }
  };
  ($tokens:expr, $stomach:ident) => {
    bind_state!(st);
    DigestText!($tokens, $stomach, st)
  };
  ($tokens:expr, $stomach:ident, $state_arg:ident) => {
    digest_text($tokens, $stomach, $state_arg)
  };
}

macro_rules! RawTeX {
  ($text:expr) => {
    bind_state!(st);
    RawTeX!($text, st)
  };
  ($text:expr, $state_arg:ident) => {{
    let mut state_stomach = $state_arg.stomach.clone();
    match outer_stomach!().as_mut() {
      Some(st) => (*st).raw_tex($text, $state_arg)?,
      None => state_stomach.borrow_mut().raw_tex($text, $state_arg)?,
    }
  }};
}

#[macro_export]
macro_rules! Dimension {
  ($number:expr) => {{
    bind_state!(st);
    Dimension!($number, st)
  }};
  ($number:expr, $state_arg:ident) => {
    ::rtx_core::common::dimension::Dimension::new_str($number, $state_arg)?
  };
}

#[macro_export]
macro_rules! DocType {
  ($rootelement:expr, $pubid:expr, $sysid:expr) => {
    bind_state!(st);
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
