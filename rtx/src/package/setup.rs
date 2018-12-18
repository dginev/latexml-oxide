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
macro_rules! SetupBindingMacros {($state:ident) => (
  let state_stomach = $state.stomach.clone();
  #[allow(unused_macros)]
  //============================================
  // Convenience macros for writing definitions.
  //============================================
  macro_rules! LookupValue {
    ($name:expr) => (LookupValue!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.lookup_value($name))
  }
  macro_rules! LookupBool {
    ($name:expr) => (LookupBool!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.lookup_bool($name))
  }
  macro_rules! LookupString {
    ($name:expr) => (LookupString!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.lookup_string($name))
  }
  macro_rules! LookupNumber {
    ($name:expr) => (LookupNumber!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.lookup_number($name))
  }
  macro_rules! LookupTokens {
    ($name:expr) => (LookupTokens!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.lookup_tokens($name))
  }
  macro_rules! AssignValue {
    ($name:expr, $value:expr) => (AssignValue!($name, $value, None, $state));
    ($name:expr, $value:expr, $scope:expr) => (AssignValue!($name, $value, $scope, $state));
    ($name:expr, $value:expr, $scope:expr, $state_arg:ident) => ($state_arg.assign_value($name, $value, $scope))
  }
  macro_rules! RemoveValue {
    ($name:expr) => (RemoveValue!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.remove_value($name))
  }
  macro_rules! PushValue {
    ($name:expr, $values:expr) => (PushValue!($name, $values, $state));
    ($name:expr, $values:expr, $state_arg:ident) => ($state_arg.push_value($name, $values))
  }
  macro_rules! PopValue  {
    ($name:expr) => (PopValue!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.pop_value($name))
  }
  macro_rules! UnshiftValue {
    ($name:expr, $values:expr) => (UnshiftValue!($name, $values, $state));
    ($name:expr, $values:expr,$state_arg:ident) => ($state_arg.unshift_value($name, $values))
  }
  macro_rules! ShiftValue {
    ($name:expr) => (ShiftValue!($name, $state));
    ($name:expr,$state_arg:ident) => ($state_arg.shift_value($name))
  }
  macro_rules! LookupMapping {
    ($map:expr, $key:expr) => (LookupValue!($map, $key, $state));
    ($map:expr, $key:expr, $state_arg:ident) => ($state_arg.lookup_mapping($map, $key))
  }
  macro_rules! AssignMapping {
    ($map:expr, $key:expr => $value:expr) => (AssignMapping!($map, $key => $value, $state));
    ($map:expr, $key:expr => $value:expr, $state_arg:ident) => ($state_arg.assign_mapping($map, $key, $value.into()))
  }
  macro_rules! LookupMappingKeys {
    ($map:expr) => (LookupMappingKeys!($map, $state));
    ($map:expr, $state_arg:ident) => ($state_arg.lookup_mapping_keys($map))
  }
  macro_rules! LookupCatcode {
    ($char:expr) => (LookupCatcode!($char, $state));
    ($char:expr, $state_arg:ident) => ($state_arg.lookup_catcode($char))
  }
  macro_rules! AssignCatcode {
    ($char:expr, $catcode:expr, $scope:expr) => (AssignCatcode!($char, $catcode, $scope, $state));
    ($char:expr, $catcode:expr, $scope:expr, $state_arg:ident) => ($state_arg.assign_catcode($char, $catcode, $scope));
  }
  macro_rules! LookupMeaning {
    ($name:expr) => (LookupMeaning!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.lookup_meaning($name))
  }
  macro_rules! LookupDefinition {
    ($name:expr) => (LookupDefinition!($name, $state));
    ($name:expr, $state_arg:ident) => ($state_arg.lookup_definition($name))
  }

  macro_rules! InstallDefinition {
    ($name:expr, $definition:expr, $scope:expr) => (InstallDefinition!($name, $definition, $scope, $state));
    ($name:expr, $definition:expr, $scope:expr, $state_arg:ident) => ($state_arg.install_definition($name, $definition, $scope))
  }

  macro_rules! XEquals {
    ($token1:expr, $token2:expr) => (XEquals!($token1, $token2, $state));
    ($token1:expr, $token2:expr, $state_arg:ident) => ($state_arg.x_equals($token1, $token2))
  }

  macro_rules! IsDefined {
    ($name:expr) => (IsDefined!($name, $state));
    ($name:expr, $state_arg:ident) => (is_defined_token($name, $state_arg))
  }
  macro_rules! IsDefinedToken {($name:expr) => (IsDefinedToken!($name, $state))}
  macro_rules! Let {
    ($token1:expr, $token2:expr) => (Let!($token1, $token2, $state));
    ($token1:expr, $token2:expr, $state_arg:ident) => ({
      LetI!(&T_CS!($token1), T_CS!($token2), $state_arg)
    });
    ($token1:expr, $token2:expr, $scope:expr, $state_arg:ident) => ({
      LetI!(&T_CS!($token1), T_CS!($token2), $scope, $state_arg)
    });
  }
  macro_rules! LetI {
    ($token1:expr, $token2:expr) => (LetI!($token1, $token2, $state));
    ($token1:expr, $token2:expr, $state_arg:ident) => ($state_arg.let_i($token1, $token2, None));
    ($token1:expr, $token2:expr, $scope:expr, $state_arg:ident) => ($state_arg.let_i($token1, $token2, $scope));
  }
  macro_rules! Digest {
    ($tokens:expr) => (state_stomach.borrow_mut().digest($tokens, $state));
    ($tokens:expr, $state_arg:ident) => (state_stomach.borrow_mut().digest($tokens, $state_arg));
  }

  macro_rules! DigestText {
    ($stuff:expr) => (state_stomach.borrow_mut().digest_text($stuff, $state));
    ($stuff:expr, $stomach:ident) => (DigestText!($stuff, $stomach, $state));
    ($stuff:expr, $stomach:ident, $state_arg:ident) => (digest_text($stuff, $stomach, $state_arg));
  }

  macro_rules! DigestIf {
    ($token:expr, $stomach:ident) => (DigestIf!($token, $stomach, $state));
    ($token:expr, $stomach:ident, $state_arg: ident) => (digest_if($token, $stomach, $state_arg));
  }

  macro_rules! AfterAssignment {
    () => (AfterAssignment!($state));
    ($state_arg: ident) => ($state_arg.after_assignment());
  }

  // Merge the current font with the style specifications
  macro_rules! MergeFont {
    ($kv:expr) => (MergeFont!($kv, $state));
    ($kv:expr, $state_arg:ident) => (merge_font($kv, $state_arg))
  }

  //======================================================================
  // Defining new Control-sequence Parameter types.
  //======================================================================
  macro_rules! DefParameterType{
    ($name:expr) => (DefParameterType!($name, $state));
    ($name:expr, $key1:ident => $val1:expr)=>(DefParameterType!($name, $key1=>$val1, $state));
    ($name:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr)=>(DefParameterType!($name, $key1=>$val1, $key2=>$val2, $state));
    ($name:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr)=>(DefParameterType!($name, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($name:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr)=>(DefParameterType!($name, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($name:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr)=>(DefParameterType!($name, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));
    ($name:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr,
      $key6:ident=>$val6:expr)=>(DefParameterType!($name, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $key6=>$val6, $state));

    // Explicit state form
    ($name:expr, $state_arg:ident) => (DefParameterTypeWO!($name, Parameter::default(), $state_arg));

    ($name:expr,
     $key1:ident => $val1:expr, $state_arg:ident
    ) => (DefParameterTypeWO!($name, NewDefault!(Parameter,
     name => $name.to_string(),
     $key1 => $val1), $state_arg));

    ($name:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr, $state_arg:ident
    ) => (DefParameterTypeWO!($name, NewDefault!(Parameter,
     name => $name.to_string(),
     $key1 => $val1,
     $key2 => $val2
    ), $state_arg));

    ($name:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr, $state_arg:ident
    ) => (DefParameterTypeWO!($name, NewDefault!(Parameter,
     name => $name.to_string(),
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3
    ), $state_arg));

    ($name:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr, $state_arg:ident
    ) => (DefParameterTypeWO!($name, NewDefault!(Parameter,
     name => $name.to_string(),
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4
    ), $state_arg));

    ($name:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr,
     $key5:ident => $val5:expr, $state_arg:ident
    ) => (DefParameterTypeWO!($name, NewDefault!(Parameter,
     name => $name.to_string(),
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4,
     $key5 => $val5,
    ), $state_arg));

    ($name:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr,
     $key6:ident => $val6:expr, $state_arg:ident
    ) => (DefParameterTypeWO!($name, NewDefault!(Parameter,
     name => $name.to_string(),
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4,
     $key5 => $val5,
     $key6 => $val6
    ), $state_arg));
  }
  macro_rules! DefParameterTypeWO {
    ($name:expr, $param:expr, $state_arg:ident) => ($state_arg.assign_mapping("PARAMETER_TYPES", $name, Some(Stored::Parameter($param))))
  }

  macro_rules! LoadPool {
    ($name:expr) => (LoadPool!($name, $state));
    ($name:expr, $state_arg:ident) => (input_definitions($name,
      InputDefinitionOptions {
        extension: Some(String::from("pool")),
        ..InputDefinitionOptions::default()
      }, $state_arg)?)
  }
  /// Loader shorthand for pool dependencies
  macro_rules! InnerPool {
    ($name:ident) => (InnerPool!($name, $state));
    ($name:ident, $state_arg:ident) => (pool::$name::load_definitions(&mut $state_arg)?)
  }

  macro_rules! RequirePackage{
    ($package:expr, $options:expr) => (RequirePackage!($package, $options, $state));
    ($package:expr, $options:expr, $state_arg:ident) => (require_package($package, $options, $state_arg))
  }
  macro_rules! LoadClass{
    ($class:expr, $options:expr, $after:expr) => (LoadClass!($class, $options, $after, $state));
    ($class:expr, $options:expr, $after:expr, $state_arg:ident) => (load_class($class, $options, $after, $state_arg))
  }

  macro_rules! DeclareFontMap{
    ($name:expr, $map:expr, $family:expr, $state_arg: ident) => (
      let mapname = s!("{}_{}_fontmap",$name, $family);
      let map : Vec<Option<char>> = $map;
      $state_arg.assign_value(&mapname, map, Some(Scope::Global));
    );
    ($name:expr, $map:expr, $state_arg: ident) => (
      let mapname = s!("{}_fontmap",$name);
      let map : Vec<Option<char>> = $map;
      $state_arg.assign_value(&mapname, map, Some(Scope::Global));
    );

    ($name:expr, $map:expr, $family:expr) => (DeclareFontMap!($name, $map, $family, $state));
    ($name:expr, $map:expr) => (DeclareFontMap!($name, $map, $state));
  }

  macro_rules! DefMacroI(
    // With explicit state
    // TODO: Propagate options, such as "locked", etc
    // Expansion closure syntax + explicit state
    ($cs:expr, $paramlist:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block, $state_arg:ident) => {
      let expansion_closure : Option<ExpansionClosure> = Some(Rc::new(move |$gullet, $args, $inner_state| $body));
      def_macro($cs, $paramlist, expansion_closure, $state_arg); };
    ($cs:expr, $paramlist:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block, $key1:ident=>$val1:expr, $state_arg:ident) => {
      let expansion_closure : Option<ExpansionClosure> = Some(Rc::new(move |$gullet, $args, $inner_state| $body));
      def_macro($cs, $paramlist, expansion_closure, $state_arg); };
    // Without explicit state
    // Expansion closure syntax
    ($cs:expr, $paramlist:expr, sub [$gullet:ident, $args:ident, $inner_state:ident ] $body:block) =>
        (DefMacroI!($cs, $paramlist, sub [$gullet, $args, $inner_state] $body, $state));
    ($cs:expr, $paramlist:expr, sub [$gullet:ident, $args:ident, $inner_state:ident ] $body:block, $key1:ident=>$val1:expr) =>
        (DefMacroI!($cs, $paramlist, sub [ $gullet, $args, $inner_state ] $body, $key1=>$val1, $state));

    // With explicit state
    // TODO: Propagate options, such as "locked", etc
    // Simple Expression syntax + explicit state
    ($cs:expr, $paramlist:expr, None, $state_arg:ident) => (def_macro($cs, $paramlist, None, $state_arg));
    ($cs:expr, $paramlist:expr, $expansion:expr, $state_arg:ident) => (def_macro($cs, $paramlist, $expansion, $state_arg));
    ($cs:expr, $paramlist:expr, $expansion:expr, $key1:ident=>$val1:expr, $state_arg:ident) => (def_macro($cs, $paramlist, $expansion, $state_arg));

    // Simple Expression syntax
    ($cs:expr, $paramlist:expr, None) => (DefMacroI!($cs, $paramlist, None, $state));
    ($cs:expr, $paramlist:expr, $expansion:expr) => (DefMacroI!($cs, $paramlist, $expansion, $state));
    ($cs:expr, $paramlist:expr, $expansion:expr, $key1:ident=>$val1:expr) => (DefMacroI!($cs, $paramlist, $expansion, $key1=>$val1, $state));

  );

  macro_rules! DefMacro {
    // Closure form
    ($proto:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block) => (
      DefMacroWO!($proto, sub [$gullet, $args, $inner_state] $body, ExpandableOptions::default(), $state)
    );
    ($proto:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block, $key1:ident=>$val1:expr) => (
      DefMacroWO!($proto, sub[$gullet, $args, $inner_state] $body, NewDefault!(ExpandableOptions, $key1=>$val1))
    );
    // closure; explicit state
    ($proto:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block, $state_arg:ident) => (
      DefMacroWO!($proto, sub[$gullet, $args, $inner_state] $body, ExpandableOptions::default(), $state_arg);
    );
    // String form
    ($proto:expr, $expansion:expr) => (DefMacroWO!($proto, $expansion, ExpandableOptions::default()));
    ($proto:expr, $expansion:expr, $key1:ident=>$val1:expr) =>
      (DefMacroWO!($proto, $expansion, NewDefault!(ExpandableOptions, $key1=>$val1)));
    // string; explicit state
    ($proto:expr, $expansion:expr,$state_arg:ident) => (DefMacroWO!($proto, $expansion, ExpandableOptions::default(), $state_arg));
  }

  macro_rules! DefMacroWO(
    // Rust closure expansion form
    ($proto:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block, $options:expr, $state_arg:ident) => {
      let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
      let expansion_closure : Option<ExpansionClosure> = Some(Rc::new(move |$gullet: &mut Gullet, $args: Vec<Tokens>, $inner_state:&mut State| $body));
      // TODO: Also pass in options
      def_macro(cs, paramlist, expansion_closure, $state_arg);
    };
    ($proto:expr, sub [ $gullet:ident, $args:ident, $inner_state:ident ] $body:block, $options:expr) => (
      DefMacroWO!($proto, sub [ $gullet, $args, $inner_state ] $body, $options, $state));
    // String expansion forms
    ($proto:expr, $expansion:expr, $options:expr) => (DefMacroWO!($proto, $expansion, $options, $state));
    ($proto:expr, $expansion:expr, $options:expr, $state_arg:ident) => ({
      let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
      let expansion;
      compile_expansion!(expansion, $expansion);
      // TODO: Also pass in options
      def_macro(cs, paramlist, expansion, $state_arg);
    });
  );

  macro_rules! DefConditional(
    // test is always a rust closure
    ($proto:expr, sub [$gullet:ident, $args:ident, $inner_state:ident] $body:block) => (DefConditional!($proto, sub[$gullet, $args, $inner_state] $body, $state));
    ($proto:expr, sub [$gullet:ident, $args:ident, $inner_state:ident] $body:block, $state_arg:ident) => ({
      let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
      DefConditionalI!(cs, paramlist, sub[$gullet, $args, $inner_state] $body, $state_arg)
    });
    // or None
    ($proto:expr, None) => (DefConditional!($proto, None, $state));
    ($proto:expr, None, $state_arg:ident) => ({
      let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
      DefConditionalI!(cs, paramlist, None, $state_arg)
    });
  );

  macro_rules! DefConditionalI(
    // test is always a rust closure
    ($cs:expr, $paramlist:expr, sub[$gullet:ident, $args:ident, $inner_state:ident] $body:block) =>
      (DefConditionalI!($cs, $paramlist, $gullet, $args, $inner_state, $body, $state));
    ($cs:expr, $paramlist:expr, sub[$gullet:ident, $args:ident, $inner_state:ident] $body:block, $state_arg:ident) => ({
      let test : ConditionalClosure = Rc::new(move |$gullet, $args, $inner_state| {$body});
      def_conditional($cs, $paramlist, Some(test), ConditionalOptions::default(), $state_arg);
    });
    // or None
    ($cs:expr, $paramlist:expr, None) =>
      (DefConditionalI!($cs, $paramlist, None, $state));
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
  macro_rules! DefPrimitive{
    ($proto:expr, sub[$stomach:ident, $whatsit:ident, $inner_state:ident] $body:block) =>
      (DefPrimitive!($proto, sub[$stomach, $whatsit, $inner_state] $body, $state));
    ($proto:expr, sub[$stomach:ident, $whatsit:ident, $inner_state:ident] $body:block, $key1:ident=>$val1:expr) =>
      (DefPrimitive!($proto, sub[$stomach, $whatsit, $inner_state] $body, $key1=>$val1, $state));

    ($proto:expr, sub[$stomach:ident, $whatsit:ident, $inner_state:ident] $body:block, $state_arg:ident) =>
      (DefPrimitiveIWO!($proto, |$stomach, $whatsit, $inner_state| {$body}, PrimitiveOptions::default(), $state));
    ($proto:expr, sub[$stomach:ident, $whatsit:ident, $inner_state:ident] $body:block, $key1:ident=>$val1:expr, $state_arg:ident) =>
      (DefPrimitiveIWO!($proto, |$stomach, $whatsit, $inner_state| {$body}, NewDefault!(PrimitiveOptions, $key1=>$val1), $state));

    ($proto:expr, $replacement:expr, $options:expr) => (DefPrimitive!($proto, $replacement, $options, $state));
    ($proto:expr, $replacement:expr, $options:expr, $state_arg:ident) => ({
      // TODO:
      // let compiled_replacement = || Tbox{text: $replacement, Invocation($options{alias} || $cs, @_[1 .. $#_])); }
      let compiled_replacement = $replacement;
      DefPrimitiveIWO!($proto, compiled_replacement, $options, $state_arg);
    });
  }

  macro_rules! DefPrimitiveI{
    ($proto:expr, $compiled_replacement:expr) => (DefPrimitiveI!($proto, $compiled_replacement, $state));
    ($proto:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr)=>(DefPrimitiveI!($proto, $compiled_replacement, $key1=>$val1, $state));
    ($proto:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr)=>(DefPrimitiveI!($proto, $compiled_replacement, $key1=>$val1, $key2=>$val2, $state));
    ($proto:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr)=>(DefPrimitiveI!($proto, $compiled_replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($proto:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr)=>(DefPrimitiveI!($proto, $compiled_replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($proto:expr, $compiled_replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr)=>(DefPrimitiveI!($proto, $compiled_replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));

    ($proto:expr, $compiled_replacement:expr, $state_arg:ident) => (DefPrimitiveIWO!($proto,$compiled_replacement, PrimitiveOptions::default(), $state_arg));

    ($proto:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr, $state_arg:ident
    ) => (DefPrimitiveIWO!($proto, $compiled_replacement, NewDefault!(PrimitiveOptions,
      $key1 => $val1
    ), $state_arg));

    ($proto:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr, $state_arg:ident
    ) => (DefPrimitiveIWO!($proto, $compiled_replacement, NewDefault!(PrimitiveOptions,
      $key1 => $val1,
      $key2 => $val2
    ), $state_arg));

    ($proto:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr, $state_arg:ident
    ) => (DefPrimitiveIWO!($proto, $compiled_replacement, NewDefault!(PrimitiveOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3
    ), $state_arg));

    ($proto:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr, $state_arg:ident
    ) => (DefPrimitiveIWO!($proto, $compiled_replacement, NewDefault!(PrimitiveOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4
    ), $state_arg));

    ($proto:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr, $state_arg:ident
    ) => (DefPrimitiveIWO!($proto, $compiled_replacement, NewDefault!(PrimitiveOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4,
      $key5 => $val5
    ), $state_arg));
  }

  macro_rules! DefPrimitiveII{
    ($cs:expr, $paramlist:expr, sub[$stomach:ident,$args:ident,$inner_state:ident] $body:block) =>
      (DefPrimitiveII!($cs, $paramlist, sub[$stomach, $args, $inner_state] $body, $state));
    ($cs:expr, $paramlist:expr, sub[$stomach:ident,$args:ident,$inner_state:ident] $body:block, $state_arg:ident) =>
      (DefPrimitiveII!($cs, $paramlist, move |$stomach, $args, $inner_state| {$body}, PrimitiveOptions::default(), $state_arg));
    ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options:expr) => (DefPrimitiveII!($cs, $paramlist, $compiled_replacement, $options, $state));
    ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => ({
      let options = $options;
      let options_locked = options.locked;
      let scope = options.scope.clone();
      let mut before_digest_env : Vec<BeforeDigestClosure> = Vec::new();

      if options.require_math {
        let cs_name = $cs.get_cs_name().to_owned();
        let require_math_closure = beforeproc!(stomach, state, { requireMath!(cs_name, state) });
        before_digest_env.push(require_math_closure);
      }

      if options.forbid_math {
        let cs_name = $cs.get_cs_name().to_owned();
        let forbid_math_closure = beforeproc!(stomach, state, { forbidMath!(cs_name, state) });
        before_digest_env.push(forbid_math_closure);
      }
      if let Some(ref mode) = options.mode {
        let mode_clone = mode.clone();
        let begin_mode_closure = beforeproc!(stomach, state, { stomach.begin_mode(&mode_clone, state)?; });
        before_digest_env.push(begin_mode_closure);
      }
      if options.bounded {
        let bgroup_closure = beforeproc!(stomach, state, { stomach.bgroup(state); });
        before_digest_env.push(bgroup_closure);
      }
      if let Some(chosen_font) = options.font {
        let merge_font_closure = beforeproc!(stomach, state, {
          MergeFont!(&chosen_font, state);
        });
        before_digest_env.push(merge_font_closure);
      }
      before_digest_env.extend(options.before_digest);

      let mut after_digest_env : Vec<DigestionClosure> = Vec::new();
      after_digest_env.extend(options.after_digest);
      if let Some(ref mode) = options.mode {
        let mode_clone = mode.clone();
        let end_mode_closure = afterproc!(stomach, whatsit, state, { stomach.end_mode(&mode_clone, state)?; });
        after_digest_env.push(end_mode_closure);
      }
      if options.bounded {
        let egroup_closure = afterproc!(stomach, whatsit,state, { stomach.egroup(state)?; });
        after_digest_env.push(egroup_closure);
      }

      $state_arg.install_definition(Primitive{
          cs: $cs.clone(),
          paramlist: $paramlist,
          replacement: Some(Rc::new($compiled_replacement)),
          options: PrimitiveOptions {
            before_digest: before_digest_env,
            after_digest: after_digest_env,
            ..PrimitiveOptions::default()
          }
        },
        scope);
      if options_locked {
        AssignValue!(&s!("{}:locked",$cs.get_cs_name()), true, None, $state_arg);
      }
    })
  }

  macro_rules! DefPrimitiveIWO(
    ($proto:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => ({
      let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
      DefPrimitiveII!(cs, paramlist, $compiled_replacement, $options, $state_arg);
    })
  );

  // my %register_types = (      # [CONSTANT]
  //   'LaTeXML::Common::Number'    => 'Number',
  //   'LaTeXML::Common::Dimension' => 'Dimension',
  //   'LaTeXML::Common::Glue'      => 'Glue',
  //   'LaTeXML::Core::MuGlue'      => 'MuGlue',
  //   'LaTeXML::Core::Tokens'      => 'Tokens',
  //   'LaTeXML::Core::Token'       => 'Token',
  // );

  macro_rules! DefRegister {
    ($proto:expr, $value:expr, $key1:ident => $val1:expr) => (DefRegister!($proto, $value, $key1=>$val1, $state));
    ($proto:expr, $value:expr, $key1:ident => $val1:expr, $key2:ident => $val2:expr) => (DefRegister!($proto, $value, $key1=>$val1, $key2=>$val2, $state));
    ($proto:expr, $value:expr, $key1:ident => $val1:expr, $key2:ident => $val2:expr, $key3:ident => $val3:expr) => (DefRegister!($proto, $value, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($proto:expr, $value:expr, $key1:ident => $val1:expr, $state_arg:ident) => (DefRegister!($proto, $value, Some(NewDefault!(RegisterOptions, $key1=>$val1)), $state_arg));
    ($proto:expr, $value:expr, $key1:ident => $val1:expr, $key2:ident => $val2:expr, $state_arg:ident) => (DefRegister!($proto, $value, Some(NewDefault!(RegisterOptions, $key1=>$val1, $key2=>$val2)), $state_arg));
    ($proto:expr, $value:expr, $key1:ident => $val1:expr, $key2:ident => $val2:expr, $key3:ident=>$val3:expr, $state_arg:ident) => (DefRegister!($proto, $value, Some(NewDefault!(RegisterOptions, $key1=>$val1, $key2=>$val2, $key3=>$val3)), $state_arg));
    ($proto:expr, $value:expr) => (DefRegister!($proto, $value, None, $state));
    ($proto:expr, $value:expr, $options:expr) => (DefRegister!($proto, $value, $options, $state));
    ($proto:expr, $value:expr, $options:expr, $state_arg:ident) => ({
      let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
      DefRegisterI!(cs, paramlist, $value, $options, $state_arg);
    });
  }

  macro_rules! DefRegisterI {
    ($cs:expr, $paramlist:expr, $value:expr, $options:expr, $state_arg:ident) => (def_register($cs, $paramlist, $value, $options, $state_arg));
    ($cs:expr, $paramlist:expr, $value:expr, $options:expr) => (DefRegisterI!($cs, $paramlist, $value, $options, $state));
  }

  // sub LookupRegister {
  //   my ($cs, @parameters) = @_;
  //   my $defn;
  //   $cs = T_CS($cs) unless ref $cs;
  //   if (($defn = $STATE->lookupDefinition($cs)) && $defn->isRegister) {
  //     return $defn->valueOf(@parameters); }
  //   else {
  //     Warn('expected', 'register', $STATE->getStomach,
  //       "The control sequence " . ToString($cs) . " is not a register"); }
  //   return; }

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

  macro_rules! DefConstructorI {
    ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block) => (DefConstructorI!($cs, $paramlist, $compiled_replacement, $state));
    ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident => $val1:expr)=>(DefConstructorI!($cs, $paramlist, sub[$document,$args,$props, $inner_state] $body, $key1=>$val1, $state));
    ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr)=>(DefConstructorI!($cs, $paramlist, sub[$document,$args,$props, $inner_state] $body, $key1=>$val1, $key2=>$val2, $state));
    ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr)=>(DefConstructorI!($cs, $paramlist, sub[$document,$args,$props, $inner_state] $body, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr)=>(DefConstructorI!($cs, $paramlist, sub[$document,$args,$props, $inner_state] $body, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr)=>(DefConstructorI!($cs, $paramlist, sub[$document,$args,$props, $inner_state] $body, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));
    // None replacement
    ($cs:expr, $paramlist:expr, None) => (DefConstructorI!($cs, $paramlist, None, $state));
    ($cs:expr, $paramlist:expr, None, $key1:ident=>$val1:expr) => (DefConstructorI!($cs, $paramlist, None, $key1=>$val1, $state));
    ($cs:expr, $paramlist:expr, None, $key1:ident=>$val1:expr, $key2:ident=>$val2:expr) => (DefConstructorI!($cs, $paramlist, None, $key1=>$val1, $key2=>$val2, $state));

    // with explicit state:
    ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block, $state_arg:ident
    ) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(replacement!($document, $args, $props, $inner_state, $body))), ConstructorOptions::default(), $state_arg));
    ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident => $val1:expr,
      $state_arg:ident
    ) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(replacement!($document, $args, $props, $inner_state, $body))), NewDefault!(ConstructorOptions,
      $key1 => $val1
    ),$state_arg));

    ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr,
      $state_arg:ident
    ) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(replacement!($document, $args, $props, $inner_state, $body))), NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2
    ), $state_arg));

    ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr,
      $key3:ident => $val3:expr,
      $state_arg:ident
    ) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(replacement!($document, $args, $props, $inner_state, $body))), NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3
    ), $state_arg));

    ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $state_arg:ident
    ) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(replacement!($document, $args, $props, $inner_state, $body))), NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4
    ), $state_arg));

    ($cs:expr, $paramlist:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr,
      $state_arg:ident
    ) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(replacement!($document, $args, $props, $inner_state, $body))), NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4,
      $key5 => $val5
    ), $state_arg));

    // None replacement
    ($cs:expr, $paramlist:expr, None, $state_arg:ident) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(noreplacement!())), NewDefault!(ConstructorOptions), $state_arg));
    ($cs:expr, $paramlist:expr, None,
    $key1:ident => $val1:expr,
    $state_arg:ident
    ) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(noreplacement!())), NewDefault!(ConstructorOptions, $key1=>$val1), $state_arg));
    ($cs:expr, $paramlist:expr, None,
    $key1:ident => $val1:expr,
    $key2:ident => $val2:expr,
    $state_arg:ident
    ) => (DefConstructorIWO!($cs, $paramlist, Some(Rc::new(noreplacement!())), NewDefault!(ConstructorOptions, $key1=>$val1, $key2=>$val2), $state_arg));
  }

  macro_rules! DefConstructorIWO {
    ($cs:expr, $paramlist:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => (
    {
      use rtx_core::definition::constructor::Constructor;
      let options = $options;
      // TODO: This won't work, as we can only invoke method calls on paramlist in runtime
      //*rtx_codegen::constructable::NARGS = $paramlist.get_num_args();
      if options.locked {
        $state_arg.assign_value(&s!("{}:locked",$cs.get_cs_name()), true, None)
      }
      let scope = options.scope.clone();
      let constructor = Constructor {
        cs: $cs,
        paramlist: $paramlist,
        replacement: $compiled_replacement,
        options: options};

      $state_arg.install_definition(constructor, scope);
   })
  }

  macro_rules! DefConstructor (
    // Code replacement flavors
   ($proto:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block) => (DefConstructor!($proto, sub [ $document, $args, $props, $inner_state ] $body, $state));
   ($proto:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident => $val1:expr) => (DefConstructor!($proto, sub [ $document, $args, $props, $inner_state ] $body, $key1 => $val1, $state));
    ($proto:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr) => (DefConstructor!($proto, sub [ $document, $args, $props, $inner_state ] $body, $key1 => $val1, $key2=>$val2, $state));
    ($proto:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr,
      $key3:ident => $val3:expr) => (DefConstructor!($proto, sub [ $document, $args, $props, $inner_state ] $body, $key1 => $val1, $key2=>$val2, $key3=>$val3, $state));
    // with explicit state
    ($proto:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block, $state_arg:ident) => (
      DefConstructorWO!($proto, $document, $args, $props, $inner_state, $body, ConstructorOptions::default(), $state_arg));
    ($proto:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident => $val1:expr, $state_arg:ident ) => (
      DefConstructorWO!($proto, $document, $args, $props, $inner_state, $body, NewDefault!(ConstructorOptions,$key1=>$val1), $state_arg));
    ($proto:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr, $state_arg:ident ) => (
      DefConstructorWO!($proto, $document, $args, $props, $inner_state, $body, NewDefault!(ConstructorOptions,$key1=>$val1, $key2=>$val2), $state_arg));
    ($proto:expr, sub [ $document:ident, $args:ident, $props:ident, $inner_state:ident ] $body:block,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr,
      $key3:ident => $val3:expr, $state_arg:ident ) => (
      DefConstructorWO!($proto, $document, $args, $props, $inner_state, $body, NewDefault!(ConstructorOptions,$key1=>$val1, $key2=>$val2, $key3=>$val3), $state_arg));

    // String replacement flavors
    ($cs:expr, $replacement:expr) => (DefConstructor!($cs, $replacement, $state));
    ($cs:expr, $replacement:expr,
      $key1:ident => $val1:expr)=>(DefConstructor!($cs, $replacement, $key1=>$val1, $state));
    ($cs:expr, $replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr)=>(DefConstructor!($cs, $replacement, $key1=>$val1, $key2=>$val2, $state));
    ($cs:expr, $replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr)=>(DefConstructor!($cs, $replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($cs:expr, $replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr)=>(DefConstructor!($cs, $replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($cs:expr, $replacement:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr)=>(DefConstructor!($cs, $replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));

    // with explicit state:
    ($cs:expr, $replacement:expr, $state_arg:ident) => (DefConstructorWO!($cs, $replacement, ConstructorOptions::default(), $state_arg));
    ($cs:expr, $replacement:expr, $key1:ident=>$val1:expr, $state_arg:ident) =>
      (DefConstructorWO!($cs, $replacement, NewDefault!(ConstructorOptions, $key1 => $val1), $state_arg));
    ($cs:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr, $state_arg:ident
    ) => (DefConstructorWO!($cs, $replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2
    ), $state_arg));

    ($cs:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr, $state_arg:ident
    ) => (DefConstructorWO!($cs, $replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3
    ), $state_arg));

    ($cs:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr, $state_arg:ident
    ) => (DefConstructorWO!($cs, $replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4
    ), $state_arg));

    ($cs:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr, $state_arg:ident
    ) => (DefConstructorWO!($cs, $replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4,
      $key5 => $val5
    ), $state_arg));

    // Closure replacement flavors:
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr)=>(
        DefConstructor!($cs, $document, $args, $props, $inner_state, $body,
                        $state));
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident => $val1:expr)=>(
        DefConstructor!($cs, $document, $args, $props, $inner_state, $body,
                        $key1=>$val1, $state));
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr)=>(
        DefConstructor!($cs, $document, $args, $props, $inner_state, $body,
                        $key1=>$val1, $key2=>$val2, $state));
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr)=>(
        DefConstructor!($cs, $document, $args, $props, $inner_state, $body,
                        $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr)=>(
        DefConstructor!($cs, $document, $args, $props, $inner_state, $body,
                        $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr)=>(
        DefConstructor!($cs, $document, $args, $props, $inner_state, $body,
                        $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));
    // Closure replacement, explicit state
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr, $state_arg:ident) => (
      DefConstructorWO!($cs, $document, $args, $props, $inner_state, $body, ConstructorOptions::default(), $state_arg)
    );
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr, $key1:ident=>$val1:expr, $state_arg:ident) => (
      let replacement = replacement!($document, $args, $props, $body);
      DefConstructorWO!($cs, replacement, NewDefault!(ConstructorOptions, $key1 => $val1), $state_arg)
    );
    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr, $state_arg:ident
    ) => (
      DefConstructorWO!($cs, $document, $args, $props, $inner_state, $body, NewDefault!(ConstructorOptions,
        $key1 => $val1,
        $key2 => $val2),
      $state_arg));

    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr, $state_arg:ident
    ) => (
      DefConstructorWO!($cs, $document, $args, $props, $inner_state, $body, NewDefault!(ConstructorOptions,
        $key1 => $val1,
        $key2 => $val2,
        $key3 => $val3
      ), $state_arg));

    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr, $state_arg:ident
    ) => (
      DefConstructorWO!($cs, $document, $args, $props, $inner_state, $body, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4
    ), $state_arg));

    ($cs:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr, $state_arg:ident
    ) => (
      DefConstructorWO!($cs, $document, $args, $props, $inner_state, $body, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4,
      $key5 => $val5
    ), $state_arg));
  );

  macro_rules! DefConstructorWO(
    ($proto:expr, $replacement:expr, $options:expr, $state_arg:ident) => ({
      // check_options("DefConstructor (prototype)", $constructor_options, %options);
      let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
      let compiled_replacement;
      compile_replacement!(compiled_replacement, $replacement);
      DefConstructorIWO!(cs, paramlist, compiled_replacement, $options, $state_arg);
    });
    ($proto:expr, $document:ident, $args:ident, $props:ident, $inner_state:ident, $body:block, $options:expr, $state_arg:ident) => ({
      let compiled_replacement : Option<ReplacementClosure> = Some(Rc::new(replacement!($document, $args, $props, $inner_state, $body)));
      let (cs, paramlist) = parse_prototype($proto, $state_arg)?;
      DefConstructorIWO!(cs, paramlist, compiled_replacement, $options, $state_arg);
    });
  );
  //=====================================================================
  // Define a LaTeX environment
  // Note that the body of the environment is treated is the 'body' parameter in the constructor.
  macro_rules! DefEnvironment(
    ($proto_raw:expr, $replacement:expr) => (DefEnvironment!($proto_raw, $replacement, $state));
    ($proto_raw:expr, $replacement:expr,
      $key1:ident=>$val1:expr) => (DefEnvironment!($proto_raw, $replacement, $key1=>$val1, $state));
    ($proto_raw:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr) => (DefEnvironment!($proto_raw, $replacement, $key1=>$val1, $key2=>$val2, $state));
    ($proto_raw:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr) => (DefEnvironment!($proto_raw, $replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($proto_raw:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr) => (DefEnvironment!($proto_raw, $replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($proto_raw:expr, $replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr) => (DefEnvironment!($proto_raw, $replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));

    // with explicit state:
    ($proto_raw:expr, $replacement:expr, $state_arg:ident) => (DefEnvironmentWO!($proto_raw, $replacement, ConstructorOptions::default(), $state_arg));

    ($proto_raw:expr, $replacement:expr,
     $key1:ident => $val1:expr, $state_arg:ident
    ) => (DefEnvironmentWO!($proto_raw, $replacement, NewDefault!(ConstructorOptions,
     $key1 => $val1), $state_arg));

    ($proto_raw:expr, $replacement:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr, $state_arg:ident
    ) => (DefEnvironmentWO!($proto_raw, $replacement, NewDefault!(ConstructorOptions,
     $key1 => $val1,
     $key2 => $val2
    ), $state_arg));

    ($proto_raw:expr, $replacement:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr, $state_arg:ident
    ) => (DefEnvironmentWO!($proto_raw, $replacement, NewDefault!(ConstructorOptions,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3
    ), $state_arg));

    ($proto_raw:expr, $replacement:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr, $state_arg:ident
    ) => (DefEnvironmentWO!($proto_raw, $replacement, NewDefault!(ConstructorOptions,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4
    ), $state_arg));

    ($proto_raw:expr, $replacement:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr,
     $key5:ident => $val5:expr, $state_arg:ident
    ) => (DefEnvironmentWO!($proto_raw, $replacement, NewDefault!(ConstructorOptions,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4,
     $key5 => $val5
    ), $state_arg));
  );

  macro_rules! DefEnvironmentC(
    ($proto_raw:expr, $compiled_replacement:expr) => (DefEnvironmentC!($proto_raw, $compiled_replacement, $state));
    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr) => (DefEnvironmentC!($proto_raw, $compiled_replacement, $key1=>$val1, $state));
    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr) => (DefEnvironmentC!($proto_raw, $compiled_replacement, $key1=>$val1, $key2=>$val2, $state));
    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr) => (DefEnvironmentC!($proto_raw, $compiled_replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr) => (DefEnvironmentC!($proto_raw, $compiled_replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr) => (DefEnvironmentC!($proto_raw, $compiled_replacement, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));

    // with explicit state:
    ($proto_raw:expr, $compiled_replacement:expr, $state_arg:ident) => (DefEnvironmentCWO!($proto_raw, $paramlist, $compiled_replacement, ConstructorOptions::default()));
    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $state_arg:ident
    ) => (DefEnvironmentCWO!($proto_raw, $compiled_replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1
    ), $state_arg));

    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $state_arg:ident
    ) => (DefEnvironmentCWO!($proto_raw, $compiled_replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2
    ), $state_arg));

    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $state_arg:ident
    ) => (DefEnvironmentCWO!($proto_raw, $compiled_replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3
    ), $state_arg));

    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $state_arg:ident
    ) => (DefEnvironmentCWO!($proto_raw, $compiled_replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4
    ), $state_arg));

    ($proto_raw:expr, $compiled_replacement:expr,
      $key1:ident=>$val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr,
      $state_arg:ident
    ) => (DefEnvironmentCWO!($proto_raw, $compiled_replacement, NewDefault!(ConstructorOptions,
      $key1 => $val1,
      $key2 => $val2,
      $key3 => $val3,
      $key4 => $val4,
      $key5 => $val5
    ), $state_arg));
  );
  macro_rules! DefEnvironmentI{
    ($name_raw:expr, $paramlist:expr, $compiled_replacement:expr, $cc_copy:expr, $options:expr) =>
      (DefEnvironmentI!($name_raw, $paramlist, $compiled_replacement, $cc_copy, $options, $state));
    ($name_raw:expr, $paramlist:expr, $compiled_replacement:expr, $cc_copy:expr, $options:expr, $state_arg:ident) => ({
      use rtx_core::stomach::Stomach;
      use rtx_core::whatsit::Whatsit;
      use rtx_core::definition::constructor::Constructor;
      let name = $name_raw.to_string();
      let options = $options;
      let begin_name = s!("\\begin{{{}}}",&name);
      let end_name = s!("\\end{{{}}}",&name);
      // This is for the common case where the environment is opened by \begin{env}
      // let sizer = inferSizer($options.sizer, $options.reversion);
      let mut before_digest_env : Vec<BeforeDigestClosure> = Vec::new();
      match &options.mode {
        Some(ref mode) => {
          let bmode = mode.clone();
          let mode_closure = Rc::new(move |stomach: &mut Stomach, state: &mut State| {
            stomach.begin_mode(&bmode, state)?;
            Ok(Vec::new())
          });
          before_digest_env.push(mode_closure);
        },
        None => {
          let bgroup_closure = beforeproc!(stomach, state, {stomach.bgroup(state);});
          before_digest_env.push(bgroup_closure);
        }
      };
      if options.require_math {
        let require_name = begin_name.clone();
        let require_math_closure = beforeproc!(stomach, state, { requireMath!(require_name, state) });
        before_digest_env.push(require_math_closure);
      }
      if options.forbid_math {
        let forbid_name = begin_name.clone();
        let forbid_math_closure = beforeproc!(stomach, state, { forbidMath!(forbid_name, state) });
        before_digest_env.push(forbid_math_closure);
      }

      let env_name = name.clone();
      let current_environment_closure = beforeproc!(stomach, state, {
        AssignValue!("current_environment", env_name.clone(), None, state);
        let body = T_LETTER!(env_name.clone());
        DefMacroI!(T_CS!("\\@currenvir"), None, body.clone(), state);
      });
      before_digest_env.push(current_environment_closure);

      if let Some(chosen_font) = options.font {
        let merge_font_closure = beforeproc!(stomach, state, {
          MergeFont!(&chosen_font.clone(), state);
        });
        before_digest_env.push(merge_font_closure);
      }
      before_digest_env.extend(options.before_digest);

      let push_frame_closure = Rc::new(|_document: &mut Document, _whatsit: &Whatsit, state: &mut State| {
        state.push_frame();
      });
      let mut before_construct_with_frame : Vec<ConstructionClosure> = vec![push_frame_closure];
      before_construct_with_frame.extend(options.before_construct);

      let mut after_construct_with_frame : Vec<ConstructionClosure> = options.after_construct;

      let pop_frame_closure = Rc::new(|_document: &mut Document, _whatsit: &Whatsit, state: &mut State| {
        state.pop_frame();
      });
      after_construct_with_frame.push(pop_frame_closure);

      let begin_name_constructor = Rc::new(Constructor {
          cs: T_CS!(begin_name),
          paramlist: $paramlist,
          replacement: $compiled_replacement,
          options: ConstructorOptions {
            nargs: options.nargs,
            before_digest: before_digest_env,
            after_digest: options.after_digest_begin,
            after_digest_body: options.after_digest_body,
            before_construct: before_construct_with_frame,
            // Curiously, it's the \begin whose afterConstruct gets called.
            after_construct: after_construct_with_frame,
            capture_body: true,
            properties: options.properties.clone(),
            // (defined $options{reversion} ? (reversion => $options{reversion}) : ()),
            // (defined $sizer ? (sizer => $sizer) : ()),
            // ), $options{scope});
            ..ConstructorOptions::default()
          }});
      $state_arg.install_definition(begin_name_constructor, options.scope.clone());


      let mut after_digest_env = options.after_digest;
      let unexpected_end_closure = Rc::new(|_stomach: &mut Stomach, _whatsit: &mut Whatsit, state: &mut State| {
        // let env = LookupValue!("current_environment", $state_arg);
        //     Error('unexpected', "\\end{$name}", $_[0],
        //       "Can't close environment $name",
        //       "Current are "
        //         . join(', ', state->lookupStackedValues('current_environment')))
        //       unless $env && $name eq $env;
        //     return; },
        Ok(Vec::new())
      });
      after_digest_env.push(unexpected_end_closure);

      match options.mode {
        Some(mode) => {
          let emode = mode.clone();
          let emode_closure = Rc::new(move |stomach: &mut Stomach, _whatsit: &mut Whatsit, state: &mut State| {
            stomach.end_mode(&emode, state)?;
            Ok(Vec::new())
          });
          after_digest_env.push(emode_closure);
        },
        None => {
          let egroup_closure = Rc::new(|stomach: &mut Stomach, _whatsit: &mut Whatsit, state: &mut State| {
            stomach.egroup(state)?;
            Ok(Vec::new())
          });
          after_digest_env.push(egroup_closure);
        }
      };

      let end_envname_constructor = Rc::new(Constructor {
        cs: T_CS!(end_name),
        replacement: None,
        paramlist: None,
        options: ConstructorOptions {
          before_digest: options.before_digest_end,
          after_digest: after_digest_env,
          ..ConstructorOptions::default()
        }
      });
      $state_arg.install_definition(end_envname_constructor, options.scope.clone());

      // For the uncommon case opened by \csname env\endcsname
      let name_constructor = Rc::new(Constructor{
        cs: T_CS!(s!("\\{}", &name)),
        paramlist: $paramlist,
        replacement: $cc_copy,
        // beforeDigest => flatten(($options{requireMath} ? (sub { requireMath($name); }) : ()),
        //   ($options{forbidMath} ? (sub { forbidMath($name); })              : ()),
        //   ($mode                ? (sub { $_[0]->beginMode($mode); })        : ()),
        //   ($options{font}       ? (sub { MergeFont(%{ $options{font} }); }) : ()),
        //   $options{beforeDigest}),
        // afterDigest     => flatten($options{afterDigestBegin}),
        // afterDigestBody => flatten($options{afterDigestBody}),
        // beforeConstruct => flatten(sub { state->pushFrame; }, $options{beforeConstruct}),
        // Curiously, it's the \begin whose afterConstruct gets called.
        // afterConstruct => flatten($options{afterConstruct}, sub { state->popFrame; }),
        options: ConstructorOptions {
          nargs: options.nargs,
          capture_body: true,
          properties: options.properties.clone(),
          // (defined $options{reversion} ? (reversion => $options{reversion}) : ()),
          // (defined $sizer ? (sizer => $sizer) : ()),
          // ), $options{scope});
          ..ConstructorOptions::default()
        }
      });
      $state_arg.install_definition(name_constructor, options.scope.clone());

      let end_name_constructor = Rc::new(Constructor {
        cs: T_CS!(s!("\\end{}",&name)),
        paramlist: None,
        replacement: Some(Rc::new(|document, whatsit, properties, state|{
          let env = state.lookup_value("current_environment");
          // Error('unexpected', "\\end{$name}", $_[0],
          //   "Can't close environment $name",
          //   "Current are "
          //     . join(', ', state->lookupStackedValues('current_environment')))
          //   unless $env && $name eq $env;
          Ok(()) })),
        // beforeDigest => flatten($options{beforeDigestEnd}),
        // afterDigest  => flatten($options{afterDigest},
        //   ($mode ? (sub { $_[0]->endMode($mode); }) : ())),
        // ), $options{scope});
        options: ConstructorOptions::default()
      });
      $state_arg.install_definition(end_name_constructor, options.scope);

      if options.locked {
        AssignValue!(&s!("\\begin{{{}}}:locked",&name), true, None, $state_arg);
        AssignValue!(&s!("\\end{{{}}}:locked",&name)  , true, None, $state_arg);
        AssignValue!(&s!("\\{}:locked",&name)       , true, None, $state_arg);
        AssignValue!(&s!("\\end{}:locked",&name)    , true, None, $state_arg);
      }
    })
  }

  macro_rules! Tag {
    ($tag:expr, $key1:ident => $val1:expr)=>(Tag!($tag, $key1=>$val1, $state));
    ($tag:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr)=>(Tag!($tag, $key1=>$val1, $key2=>$val2, $state));
    ($tag:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr)=>(Tag!($tag, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($tag:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr)=>(Tag!($tag, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($tag:expr, $key1:ident => $val1:expr,
      $key2:ident=>$val2:expr,
      $key3:ident=>$val3:expr,
      $key4:ident=>$val4:expr,
      $key5:ident=>$val5:expr)=>(Tag!($tag, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));

    // with explicit state:
    ($tag:expr,
     $key1:ident => $val1:expr,
     $state_arg:ident
    ) => (TagWO!($tag, NewDefault!(TagOptions,
     $key1 => Some($val1)), $state_arg));

    ($tag:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $state_arg:ident
    ) => (TagWO!($tag, NewDefault!(TagOptions,
     $key1 => Some($val1),
     $key2 => Some($val2)
    ), $state_arg));

    ($tag:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $state_arg:ident
    ) => (TagWO!($tag, NewDefault!(TagOptions,
     $key1 => Some($val1),
     $key2 => Some($val2),
     $key3 => Some($val3)
    ), $state_arg));

    ($tag:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr,
     $state_arg:ident
    ) => (TagWO!($tag, NewDefault!(TagOptions,
     $key1 => Some($val1),
     $key2 => Some($val2),
     $key3 => Some($val3),
     $key4 => Some($val4)
    ),$state_arg));

    ($tag:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr,
     $key5:ident => $val5:expr,
     $state_arg:ident
    ) => (TagWO!($tag, NewDefault!(TagOptions,
     $key1 => Some($val1),
     $key2 => Some($val2),
     $key3 => Some($val3),
     $key4 => Some($val4),
     $key5 => Some($val5)
    ),$state_arg));
  }

  macro_rules! TagWO {
    ($tag:expr, $properties:expr, $state_arg:ident) => (install_tag($tag, $properties, $state_arg))
  }
  // sub DocType {
  //   my ($rootelement, $pubid, $sysid, %namespaces) = @_;
  //   let model = state->getModel;
  //   $model->setDocType($rootelement, $pubid, $sysid);
  //   foreach let prefix (keys %namespaces) {
  //     $model->registerDocumentNamespace($prefix => $namespaces{$prefix}); }
  //   return; }


  macro_rules! DefEnvironmentWO (
    ($proto_raw:expr, $replacement:expr, $options:expr, $state_arg:ident) => ({
    use rtx_core::util::text::*;
    let mut proto = $proto_raw.to_string().trim_start().to_string();
    let name = extract_bracketed(&mut proto, Some(&Delimiter::Brace));

    let compiled_replacement;
    compile_replacement!(compiled_replacement, $replacement);
    let cc_copy;
    compile_replacement!(cc_copy, $replacement);

    let options = $options;

    DefEnvironmentI!(name, None, compiled_replacement, cc_copy, options, $state_arg);
  }));

  macro_rules! DefEnvironmentCWO (
    ($proto_raw:expr, $compiled_replacement:expr, $options:expr, $state_arg:ident) => ({
    use rtx_core::util::text::*;
    let mut proto = $proto_raw.to_string().trim_start().to_string();
    let name = extract_bracketed(&mut proto, Some(&Delimiter::Brace));
    // TODO: What do we do with param lists?
    //let paramlist_str = proto.trim_start().to_string();
    DefEnvironmentI!(name, None, $compiled_replacement, $compiled_replacement, $options, $state_arg);
  }));


  macro_rules! RelaxNGSchema{
    ($name:expr) => (RelaxNGSchema!($name, $state));
    ($name:expr,$state_arg:ident) => (select_relaxng_schema($name.to_string(), None, $state_arg))
  }
  macro_rules! RegisterNamespace(
    ($prefix:expr, $namespace:expr) => (RegisterNamespace!($prefix, $namespace, $state));
    ($prefix:expr, $namespace:expr,$state_arg:ident) => ($state_arg.model.register_namespace($prefix, Some($namespace.to_string())))
  );
  macro_rules! RegisterDocumentNamespace(
    ($prefix:expr, $namespace:expr) => (RegisterDocumentNamespace!($prefix, $namespace, $state));
    ($prefix:expr, $namespace:expr,$state_arg:ident) => ($state_arg.model.register_document_namespace($prefix, Some($namespace.to_string())))
  );
  macro_rules! RequireResource(
    ($resource:expr) => (RequireResource!($resource, $state));
    ($resource:expr,$state_arg:ident) => (require_resource(Resource{name: $resource.to_string(), ..Resource::default()}, $state_arg))
  );

  // sub DefMath {
  //   my ($proto,
  //     $presentation, %options) = @_;
  //   CheckOptions("DefMath ($proto)", $math_options, %options);
  //   DefMathI(parsePrototype($proto), $presentation, %options);
  //   return; }
  macro_rules! DefMathI(
    ($text:expr,$paramlist:expr,$presentation:expr,
      $key1:ident => $val1:expr)=>(DefMathI!($text, $paramlist, $presentation, $key1=>$val1, $state));
    ($text:expr,$paramlist:expr,$presentation:expr,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr)=>(DefMathI!($text, $paramlist, $presentation, $key1=>$val1, $key2=>$val2, $state));
    ($text:expr,$paramlist:expr,$presentation:expr,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr,
      $key3:ident => $val3:expr)=>(DefMathI!($text, $paramlist, $presentation, $key1=>$val1, $key2=>$val2, $key3=>$val3, $state));
    ($text:expr,$paramlist:expr,$presentation:expr,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr,
      $key3:ident => $val3:expr,
      $key4:ident => $val4:expr)=>(DefMathI!($text, $paramlist, $presentation, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $state));
    ($text:expr,$paramlist:expr,$presentation:expr,
      $key1:ident => $val1:expr,
      $key2:ident => $val2:expr,
      $key3:ident => $val3:expr,
      $key4:ident => $val4:expr,
      $key5:ident => $val5:expr)=>(DefMathI!($text, $paramlist, $presentation, $key1=>$val1, $key2=>$val2, $key3=>$val3, $key4=>$val4, $key5=>$val5, $state));

    // with explicit state:
    ($text:expr,$paramlist:expr,$presentation:expr,
     $key1:ident => $val1:expr,
     $state_arg:ident
    ) => (DefMathWO!($text,$paramlist, $presentation, NewDefaultV!(MathPrimitiveOptions,
     $key1 => $val1),$state_arg));

    ($text:expr,$paramlist:expr,$presentation:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $state_arg:ident
    ) => (DefMathWO!($text,$paramlist, $presentation, NewDefaultV!(MathPrimitiveOptions,
     $key1 => $val1,
     $key2 => $val2
    ), $state_arg));

    ($text:expr,$paramlist:expr,$presentation:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $state_arg:ident
    ) => (DefMathWO!($text,$paramlist, $presentation, NewDefaultV!(MathPrimitiveOptions,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3
    ),$state_arg));

    ($text:expr,$paramlist:expr,$presentation:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr,
     $state_arg:ident
    ) => (DefMathWO!($text,$paramlist, $presentation, NewDefaultV!(MathPrimitiveOptions,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4
    ), $state_arg));

    ($text:expr,$paramlist:expr,$presentation:expr,
     $key1:ident => $val1:expr,
     $key2:ident => $val2:expr,
     $key3:ident => $val3:expr,
     $key4:ident => $val4:expr,
     $key5:ident => $val5:expr,
     $state_arg:ident
    ) => (DefMathWO!($text,$paramlist, $presentation, NewDefaultV!(MathPrimitiveOptions,
     $key1 => $val1,
     $key2 => $val2,
     $key3 => $val3,
     $key4 => $val4,
     $key5 => $val5
    ), $state_arg));
  );

  macro_rules! DefMathWO {
    ($cstext:expr, $paramlist:expr, $presentation:expr, $options:expr, $state_arg:ident) => ({
      let mut options = $options;
      let cs = T_CS!($cstext.to_string());
      let presentation = $presentation.to_string();
      // Can't defer parsing parameters since we need to know number of args!
      // $paramlist = parseParameters($paramlist, $cs) if defined $paramlist && !ref $paramlist;
      let paramlist : Option<Parameters> = $paramlist;
      let nargs = match paramlist {
        Some(plist) => plist.get_num_args(),
        None => 0
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
        let mut math_attr_hash : HashMap<String, String> = HashMap::new();
        transfer_opt_default!(name, options, math_attr_hash);
        transfer_opt_default!(meaning, options, math_attr_hash);
        transfer_opt_default!(omcd, options, math_attr_hash);
        transfer_opt_default!(role, options, math_attr_hash);
        transfer_opt_default!(mathstyle, options, math_attr_hash);
        transfer_default!(stretchy, options, math_attr_hash);
        $state_arg.assign_value(&s!("math_token_attributes_{}",csname), math_attr_hash, Some(Scope::Global));
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
      if nargs == 0 {// && !grep { !$$simpletoken_options{$_} } keys %options) {
        defmath_prim!(cs, paramlist, $presentation.to_string(), options, $state_arg);
      }

      // else {
      //   defmath_cons($cs, $paramlist, $presentation, %options); }
      // AssignValue($csname . ":locked" => 1) if $options{locked};
    })
  }

  macro_rules! defmath_prim {
    ($cs:expr, $_paramlist:expr, $presentation:expr, $options:expr, $state_arg:ident) => ({
    let mut prim_options = $options;
    prim_options.locked = false;
    prim_options.font = None;
    let scope = prim_options.scope.clone();
    let reqfont = prim_options.font.clone().unwrap_or_else(Font::default);
    $state_arg.install_definition(MathPrimitive{
      cs: $cs.clone(),
      paramlist: None, // never any parameters, this is intentional
      replacement: Some(Rc::new(move |stomach, args, state| {
        // let locator    = $stomach->getGullet->getLocator;
        let mut properties = HashMap::new(); // TODO: sync with perl master here
        properties.insert(s!("mode"), Stored::String(String::from("math")));
        // TODO: Improve font precision here, the defaults may not belong in this lookup
        let font = state.lookup_font().unwrap_or_else(|| Rc::new(Font::default())).merge(&reqfont).specialize(&$presentation);
        let font = Rc::new(font);
        // foreach my $key (keys %properties) {
        //   my $value = $properties{$key};
        //   if (ref $value eq 'CODE') {
        //     $properties{$key} = &$value(); } }
        info!("defmath_prim: {}, tokens: {:?}", &$presentation, $cs);
        Ok(vec![Digested::TBox(Rc::new( // TODO: Can we reduce boilerplate?
          Tbox{ text: $presentation, tokens: Tokens!($cs.clone()), font, properties, ..Tbox::default()}
        ))])
      })),
      options: prim_options,
      ..MathPrimitive::default()
      }, scope);
    })
  }

  macro_rules! requireMath {
    ($cs_name:expr, $state_arg:ident) => (
      if !LookupBool!("IN_MATH", $state_arg) {
        warn!(target: "unexpected", "{} should only appear in math mode",$cs_name);
      }
    )
  }
  macro_rules! forbidMath {
    ($cs_name:expr) => (forbidMath!($cs_name, $state));
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

  macro_rules! NewCounter {
    ($ctr:expr) => (NewCounter!($ctr, "", None, $state));
    ($ctr:expr, $within:expr) => (NewCounter!($ctr, $within, None, $state));
    ($ctr:expr, $within:expr, None, $state_arg:ident) => (new_counter($ctr, $within, None, $state_arg)?);

    // with options
    ($ctr:expr, $within:expr, $key1:ident => $val1:expr) => (NewCounter!($ctr, $within, $key1 => $val1, $state));
    ($ctr:expr, $within:expr, $key1:ident => $val1:expr, $state_arg: ident) =>
     (new_counter($ctr, $within, Some(NewDefault!(NewCounterOptions, $key1=>$val1)), $state_arg)?);
    ($ctr:expr, $within:expr, $key1:ident => $val1:expr, $key2:ident => $val2:expr) => (NewCounter!($ctr, $within, $key1=>$val1, $key2=>$val2, $state));
    ($ctr:expr, $within:expr, $key1:ident => $val1:expr, $key2:ident => $val2:expr, $state_arg: ident) =>
     (new_counter($ctr, $within, Some(NewDefault!(NewCounterOptions, $key1=>$val1, $key2=>$val2)), $state_arg)?);
  }

  macro_rules! CounterValue {
    ($ctr:expr) => (counter_value($ctr, $state));
    ($ctr:expr, $state_arg:ident) => (counter_value($ctr, $state_arg));
  }

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

  macro_rules! AddToCounter {
    ($ctr:expr, $value:expr, $gullet:ident) => (AddToCounter!($ctr, $value, $gullet, $state));
    ($ctr:expr, $value:expr, $gullet:ident, $state_arg:ident) => (add_to_counter($ctr, $value, $gullet, $state_arg));
  }

  macro_rules! StepCounter {
    ($ctr:expr, $noreset:expr, $gullet:ident) => (StepCounter!($ctr, $noreset, $gullet, $state));
    ($ctr:expr, $noreset:expr, $gullet:ident, $state_arg:ident) => (step_counter($ctr, $noreset, $gullet, $state_arg));
  }

  macro_rules! RefStepCounter {
    ($ctr:expr, $noreset:expr, $gullet:ident) => (RefStepCounter!($ctr, $noreset, $gullet, $state));
    ($ctr:expr, $noreset:expr, $gullet:ident, $state_arg:ident) => (ref_step_counter($ctr, $noreset, $gullet, $state_arg));
  }

  /// Return $tokens with all tokens expanded
  macro_rules! Expand {
    ($tokens:expr, $gullet:ident) => (Expand!($tokens, $gullet, $state));
    ($tokens:expr, $gullet:ident, $state_arg:ident) => (do_expand($tokens, $gullet, $state_arg));
  }

  /// Invocation(<list of Token>); builds a representation of a command sequence invoked on its
  /// arguments
  macro_rules! Invocation {
    ($token:expr, $args:expr, $gullet:expr) => {
      Invocation!($token, $args, $gullet, $state)
    };
    ($token:expr, $args:expr, $gullet:expr, $state_arg:ident) => {
      build_invocation($token, $args.into_iter().map(|arg| arg.into()).collect(), $gullet, $state_arg)
    };
  }

  macro_rules! DefLigature {
    ($regex:expr, $replacement:expr, fontTest => sub[$font:ident] $body:block) => (DefLigature!($regex, $replacement, fontTest => sub[$font]{$body}, $state));
    ($regex:expr, $replacement:expr, fontTest => sub[$font:ident] $body:block, $state_arg:ident) => {
      let regex_compiled = Regex::new($regex).unwrap();
      let test_closure : Option<FontTestClosure> = Some(Rc::new(move |$font| $body));
      $state_arg.unshift_value("TEXT_LIGATURES",vec![
        Ligature {regex: $regex.to_string(),
          code: Rc::new(move |text| regex_compiled.replace_all(text, $replacement).to_string()),
          font_test: test_closure }]);
    };
    ($regex:expr, $replacement:expr) => (DefLigature!($regex, $replacement, $state));
    ($regex:expr, $replacement:expr, $state_arg:ident) => {
      let regex_compiled = Regex::new($regex).unwrap();
      $state_arg.unshift_value("TEXT_LIGATURES",vec![
        Ligature {regex: $regex.to_string(),
          code: Rc::new(move |text| regex_compiled.replace_all(text, $replacement).to_string()),
          font_test: None }]);

    }
  }

  // Defines an accent command using a combining char that follows the
  // 1st char of the argument.  In cases where there is no argument, $standalonechar is used.
  macro_rules! DefAccent {
    ($accent:expr, $combiningchar:expr, $standalonechar:expr) => {
      let mut empty_opts : HashMap<String, Stored> = HashMap::new();
      DefAccent!($accent, $combiningchar, $standalonechar, empty_opts, $state);
    };
    ($accent:expr, $combiningchar:expr, $standalonechar:expr, below => true) => (DefAccent!($accent, $combiningchar, $standalonechar, map!("below"=>Stored::Bool(true)), $state));
    ($accent:expr, $combiningchar:expr, $standalonechar:expr, $options:expr) => (DefAccent!($accent, $combiningchar, $standalonechar, $options, $state));
    ($accent:expr, $combiningchar:expr, $standalonechar:expr, $options:expr, $state_arg: ident) => {
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
        crate::package::pool::tex_accents::apply_accent(stomach, &letter[0].to_string(), $combiningchar, $standalonechar, Some(invoked), inner_state)?;
        Ok(vec![])
      }, mode => Some(String::from("text")));
    }
  }

  macro_rules! RawTeX {
    ($text:expr) => (RawTeX!($text, $state));
    ($text:expr, $state_arg:ident) => ({
      state_stomach.borrow_mut().raw_tex($text, $state_arg)?
    });
  }
)}
