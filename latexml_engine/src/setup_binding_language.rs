#[macro_export]
macro_rules! LoadDefinitions {
  ($body:block) => {
    pub fn load_definitions() -> Result<()> {
      $body
      Ok(())
    }
  };
}

//======================================================================
// Defining new Control-sequence Parameter types.
//======================================================================
#[macro_export]
macro_rules! DefParameterTypeWO {
  ($name:ident, $param:expr_2021) => {
    assign_mapping(
      "PARAMETER_TYPES",
      stringify!($name),
      Some(Stored::Parameter(Rc::new($param))),
    )
  };
}

#[macro_export]
macro_rules! LoadPool {
  ($name:expr_2021) => {{
    input_definitions($name, InputDefinitionOptions {
      extension: Some(Cow::Borrowed("pool")),
      ..InputDefinitionOptions::default()
    })?
  }};
}

#[macro_export]
macro_rules! InputDefinitions {
  ($name:expr_2021) => {{
    input_definitions($name, InputDefinitionOptions::default())?
  }};
  ($name: expr_2021, $($key:ident => $value:expr_2021),*) => {
    input_definitions($name, NewDefault!(InputDefinitionOptions, $($key => $value),*))?
  }
}

/// Loader shorthand for pool dependencies. Mirrors Perl `LoadPool` —
/// honors `<name>.pool_loaded` (the Rust port's analogue of Perl's
/// `<name>.pool.ltxml_loaded` from `Package.pm::loadLTXML`
/// L2311-2316; Rust drops the `.ltxml` suffix since the suffix has
/// no meaning in our world — see existing reads at
/// `tex_file_io.rs:199`, `mathchar.rs:889`). Already-loaded pools
/// are skipped; the flag is set before the body runs (matches Perl
/// L2315: `AssignValue(... _loaded => 1, 'global')`).
///
/// To force a re-load (Perl `latex_constructs.pool.ltxml` L19-20
/// pattern: `AssignValue('<name>.pool.ltxml_loaded' => undef);
/// LoadPool('<name>')`), clear the flag first via
/// `state::assign_value("<name>.pool_loaded", Stored::None,
/// Some(Scope::Global))` and then call `InnerPool!(<name>)`.
#[macro_export]
macro_rules! InnerPool {
  ($name:ident) => {{
    let __pool_flag = concat!(stringify!($name), ".pool_loaded");
    if !$crate::prelude::lookup_bool(__pool_flag) {
      $crate::prelude::state::assign_value(
        __pool_flag,
        true,
        Some($crate::prelude::state::Scope::Global),
      );
      $crate::$name::load_definitions()?;
    }
  }};
}

#[macro_export]
macro_rules! RequirePackage {
  ($package:expr_2021) => {{
    require_package($package, RequireOptions::default())?
  }};
  ($package:expr_2021, $options:expr_2021) => {{
    require_package($package, $options)?
  }};
  ($package:expr_2021, $($key:ident => $val:expr_2021),*) => {{
    let require_package_options = NewDefault!(RequireOptions, $($key=>$val),*);
    require_package($package, require_package_options)?
  }};
}
#[macro_export]
macro_rules! LoadClass {
  ($class:expr_2021) => {{ load_class($class, Vec::new(), Tokens!())? }};
  ($class:expr_2021, $options:expr_2021, $after:expr_2021) => {{ load_class($class, $options, $after)? }};
}

#[macro_export]
macro_rules! DeclareFontMap {
  ($name:expr_2021, $map:expr_2021, $family:expr_2021) => {{
    let mapname = s!("{}_{}_fontmap", $name, $family);
    let map: Rc<[Option<char>]> = $map;
    state::assign_value(&mapname, map, Some(Scope::Global));
  }};
  ($name:expr_2021, $map:expr_2021) => {{
    let mapname = s!("{}_fontmap", $name);
    let map: Rc<[Option<char>]> = $map;
    state::assign_value(&mapname, map, Some(Scope::Global));
  }};
}

/// Declare multi-char overrides for font map positions that need more than one character.
/// Usage: `DeclareFontMapMultichar!("T2B", { 0x80 => "\u{04F6}\u{0336}", 0x91 => "C\u{0337}" });`
#[macro_export]
macro_rules! DeclareFontMapMultichar {
  ($name:expr_2021, { $($pos:expr_2021 => $str:expr_2021),* $(,)? }) => {{
    let mapname = s!("{}_fontmap_multichar", $name);
    let map: HashMap<String, String> = [
      $( ($pos.to_string(), $str.to_string()), )*
    ].into_iter().collect();
    state::assign_value(&mapname, map, Some(Scope::Global));
  }};
}

#[macro_export]
macro_rules! FindFile {
  ($name:expr_2021) => {
    find_file($name, None)
  };
  ($name:expr_2021, type => $ext:literal) => {
    find_file(
      $name,
      Some(FindFileOptions {
        ext_type: Some(Cow::Borrowed($ext)),
        ..FindFileOptions::default()
      }),
    )
  };
}

// ======================================================================
// Color
#[macro_export]
macro_rules! LookupColor {
  ($name:expr_2021) => {{
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
//   $scope = 'global' if $state->lookupDefinition(T_CS('\ifglobalcolors')) &&
// IfCondition(T_CS('\ifglobalcolors'));   AssignValue('color_' . $name => $color, $scope);
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

#[macro_export]
macro_rules! DefRewrite {
  ($($input:tt)+) => {{
    let rewrite_options = defi_opts!(@munch ($($input)*) -> {RewriteOptions,});
    push_value("DOCUMENT_REWRITE_RULES",
      Rewrite::new("text", rewrite_options))?;
  }};
}

#[macro_export]
macro_rules! DefMathRewrite {
  ($($input:tt)+) => {{
    let rewrite_options = defi_opts!(@munch ($($input)*) -> {RewriteOptions,});
    push_value("DOCUMENT_REWRITE_RULES",
      Rewrite::new("math", rewrite_options))?;
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

macro_rules! DefMathLigature {
  ($pattern:literal, $replacement:literal, $($input:tt)+) => {{
    let attr = defi_opts!(@munch ($($input)*) -> {MathLigatureOptions,});
    let chars    = $pattern.chars().rev().collect::<Vec<_>>();
    let ntomatch = chars.len();
    let matcher : Option<LigatureMatcher> = Some(Rc::new(
      move |_document: &mut Document, node_opt: &mut Node| {
      let mut node : Node;
      let mut node_mut = node_opt;
      for c in chars.iter() {
        if model::with_node_qname(node_mut, |qname| qname != "ltx:XMTok") ||
           node_mut.get_content() != c.to_string() {
          return Ok(None);
        }
        if let Some(sibling) = node_mut.get_prev_sibling() {
          node = sibling;
          node_mut = &mut node;
        } else {
          return Ok(None);
        }
      }
      if ntomatch > 0 {
        Ok(Some((ntomatch, $replacement.to_string(), attr.clone())))
      } else {
        Ok(None)
      }
    }));
    let id = generate_ligature_id();
    unshift_value("MATH_LIGATURES", vec![Ligature {
      id,
      matcher,
      code: None,
      font_test: None,
      regex: None,
    }]);
  }};
  (matcher => sub[$document:ident, $node:ident] $code:block) => {{
    let id = generate_ligature_id();
    let matcher : Option<LigatureMatcher> = Some(Rc::new(
      |$document: &mut Document, $node: &mut Node| $code));
      unshift_value("MATH_LIGATURES", vec![Ligature {
        id,
        matcher,
        code: None,
        font_test: None,
        regex: None,
      }]);
  }};
}

#[macro_export]
macro_rules! DefConditional(
  // test with explicit arguments can get typed at compile-time
  ($proto:literal, sub [( $($var:ident),* )]
    $body:block $($input:tt)*) => {{
    compile_prototype_for_typed_conditional!($proto, sub [ ( $($var),* )]
      $body $($input)*)
  }};
  // test is always a rust closure
  ($proto:literal, sub [$args:ident]
    $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConditionalOptions,});
    let (cs, paramlist) = parse_prototype!($proto);
    let test : ConditionalClosure = Rc::new(move |mut $args| {
      $body.into_bool_result() });
    def_conditional(cs, paramlist, Some(test), options)?;
  }};
  // other shorthand cases
  ($proto:literal, $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConditionalOptions,});
    let (cs, paramlist) = parse_prototype!($proto);
    let test : ConditionalClosure = Rc::new(move |_args| {
      $body.into_bool_result() });
    def_conditional(cs, paramlist, Some(test), options)?;
  }};
  ($proto:literal $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConditionalOptions,});
    let (cs, paramlist) = parse_prototype!($proto);
    def_conditional(cs, paramlist, None, options)?;
  }};
  // internal, just declare CS
  ($cs:ident, None  $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConditionalOptions,});
    def_conditional($cs, None, None, options)?;
  }};
);

/// Small helper used by TypedMacro / TypedPrimitive / … to fix the
/// array size at compile time. Exported so downstream crates
/// (`latexml_contrib`) can expand the typed-`sub[(…)]` syntax too.
#[macro_export]
macro_rules! count {
  () => (0usize);
  ( $x:tt $($xs:tt)* ) => (1usize + $crate::count!($($xs)*));
}

#[macro_export]
macro_rules! TypedConditional {
  ($cs:literal, $these_parameters:ident,
      sub [( $($var:ident),* ):($($ptype:ident),*)]
      $body:block $($input:tt)*) => {{

    let options = defi_opts!(@munch ($($input)*) -> {ConditionalOptions,});
    let closure : ConditionalClosure =  Rc::new(
    move |args: Vec<ArgWrap>| {
      let [$($var),*] : [_; $crate::count!($($var)*)] = args.try_into().unwrap();
      $(
          let $var: parameter_rust_type!($ptype) = match $var.try_into() {
            Ok(v) => v,
            Err(e) => {
              Error!("expected", "argument", e);
              <parameter_rust_type!($ptype)>::default()
            }
          };
      )*
      $body.into_bool_result()
    });
    def_conditional(T_CS!($cs), $these_parameters, Some(closure), options)?;
  }};
}

// sub IfCondition {
//   my ($if, @args) = @_;
//   my $gullet = $state->getStomach->getGullet;
//   $if = coerceCS($if);
//   my ($defn, $test);
//   if (($defn = $state->lookupDefinition($if))
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
//   if (($defn = $state->lookupDefinition($if)) && (($$defn{conditional_type} || '') eq 'if')
//     && !$defn->getParameters) {
//     Let($if, ($value ? T_CS('\iftrue') : T_CS('\iffalse')), $scope) }
//   else {
//     Error('expected', 'conditional', $state->getStomach,
//       "Expected a conditional defined by \\newif, got '" . ToString($if) . "'"); }
//   return; }

/// Define a primitive control sequence.
///
/// Primitives are executed in the Stomach.
/// The $replacement should be a sub which returns nothing, or a list of `Box`'s or `Whatsit`'s.
/// The options are:
///    is_prefix  : 1 for things like \global, \long, etc.
///    registerType : for parameters (but needs to be worked into `DefParameter`, below).
#[macro_export]
macro_rules! DefPrimitive {
  // Case: simple literal replacement
  // Perl: Box($string, font, locator, $current_token) — reversion is the CS token
  ($proto:literal, $replacement:literal $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    let (cs, params) = parse_prototype!($proto);
    let cs_for_closure = cs;
    let closure : PrimitiveBody = PrimitiveBody::Closure(Rc::new(
      move | _args: Vec<ArgWrap>| {
      Tbox::new(arena::pin_static($replacement), None, None,
        Tokens!(cs_for_closure), SymHashMap::default())
        .into_digested_result()
    }));
    def_primitive(cs, params, Some(closure), options)?;
  }};
  // closure with literal prototype
  ($proto:literal, sub[( $($var:ident),* )]
    $body:block $($input:tt)*) => {{
    compile_prototype_for_typed_primitive!($proto, sub [( $($var),* )]
      $body $($input)*)
  }};
  // Case: closure pattern replacement
  ($proto:expr_2021, sub[$args:ident]
    $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    let (cs, params) = parse_prototype!($proto);
    let closure : PrimitiveBody = PrimitiveBody::Closure(Rc::new(
      move |$args: Vec<ArgWrap>| {
        $body.into_digested_result()
      }));
    def_primitive(cs, params, Some(closure), options)?;
  }};
  // Case: cs-noparams with closure pattern replacement
  ($cs:expr_2021, None, sub[$args:ident]
    $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    let closure = PrimitiveBody::Closure(Rc::new(
      move |$args: Vec<ArgWrap>| {
        $body.into_digested_result()
      }));
    def_primitive($cs, None, Some(closure), options)?;
  }};
  // Case: cs-noparams with closure block
  ($cs:expr_2021, None, $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    let closure = PrimitiveBody::Closure(Rc::new(
      move |_args: Vec<ArgWrap>| {
        $body.into_digested_result()
      }));
    def_primitive($cs, None, Some(closure), options)?;
  }};
  // Case: cs-noparams with replacement expr
  ($cs:expr_2021, None, $body:literal, $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    let pbody = PrimitiveBody::String(arena::pin_static($body));
    def_primitive($cs, None, Some(pbody), options)?;
  }};
  // Case: no replacement
  ($proto:literal, None $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    let (cs, params) = parse_prototype!($proto);
    def_primitive(cs, params, None, options)?;
  }};
  // Case: no params, no replacement
  ($cs:expr_2021, None, None $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    def_primitive($cs, None, None, options)?;
  }};

  // Case: closure block with implicit arguments
  ($proto:expr_2021, $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    let (cs, params) = parse_prototype!($proto);
    let closure = PrimitiveBody::Closure(Rc::new(
      move |_args: Vec<ArgWrap>| {
        $body.into_digested_result()
      }));
    def_primitive(cs, params, Some(closure), options)?;
  }};
  // Case: direct closure provided (for reasons of reusing the same closure in several definitions)
  ($proto:expr_2021, $replacement_closure:expr_2021, $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    def_primitive($proto, $replacement_closure, options)?;
  }};
  ($proto:expr_2021, $replacement_closure:expr_2021) => {{
    let (cs, params) = parse_prototype!($proto);
    def_primitive(cs, params, $replacement_closure, PrimitiveOptions::default())?;
  }};
  // Case: a literal prototype mapped to nothing, will simply eat args and drop.
  ($proto:literal) => {{
    let (cs, params) = parse_prototype!($proto);
    def_primitive(cs, params, None, PrimitiveOptions::default())?;
  }};
}

#[macro_export]
macro_rules! TypedPrimitive {
  ($cs:literal, $these_parameters:ident ,
      sub [( $($var:ident),* ):($($ptype:ident),*)]
      $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {PrimitiveOptions,});
    let replacement_closure =  PrimitiveBody::Closure(Rc::new(
    move |args: Vec<ArgWrap>| {
      let [$($var),*] : [_; $crate::count!($($var)*)] = args.try_into().unwrap();
      $(
        // TODO: How do we fine-tune the match body based on whether we have an Infallible try_into?
        #[allow(warnings)]
        let $var: parameter_rust_type!($ptype) = match $var.try_into() {
          Ok(v) => v,
          Err(e) => {
            Error!("expected", "argument", e);
            <parameter_rust_type!($ptype)>::default()
          }
        };
      )*
      $body.into_digested_result()
    }));
    def_primitive(T_CS!($cs), $these_parameters, Some(replacement_closure), options)?;
  }};
}

#[macro_export]
macro_rules! LookupRegister {
  ($cs:expr_2021) => {
    LookupRegister!($cs, Vec::new())
  };
  ($cs:expr_2021, $parameters:expr_2021) => {{
    let params = { $parameters };
    let defn_opt = { state::lookup_register_definition(&T_CS!($cs)) };
    if let Some(defn) = defn_opt {
      defn.value_of(params).unwrap_or_default()
    } else {
      let message = s!("The control sequence {:?} is not a register", $cs);
      Warn!("expected", "register", message);
      RegisterValue::default()
    }
  }};
}

#[macro_export]
macro_rules! LookupRegisterOrDefault {
  ($cs:expr_2021) => {
    LookupRegisterOrDefault!($cs, Vec::new())
  };
  ($cs:expr_2021, $parameters:expr_2021) => {
    if let Some(defn) = { state::lookup_register_definition(&T_CS!($cs)) } {
      defn.value_of($parameters).unwrap_or_default()
    } else {
      RegisterValue::default()
    }
  };
}

// sub LookupDimension {
//   my ($cs) = @_;
//   my $defn;
//   $cs = T_CS($cs) unless ref $cs;
//   if (my $defn = $state->lookupDefinition($cs)) {
//     if ($defn->isRegister) {    # Easy (and proper) case.
//       return $defn->valueOf; }
//     else {
//       $state->getStomach->getGullet->readingFromMouth(LaTeXML::Core::Mouth->new(), sub { # start
// with empty mouth           my ($gullet) = @_;
//           $gullet->unread($cs);    # but put back tokens to be read
//           return $gullet->readDimension; }); } }
//   else {
//     Warn('expected', 'register', $state->getStomach,
//       "The control sequence " . ToString($cs) . " is not a register"); }
//   return Dimension(0); }

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
//   mode            : causes a switch into the given mode during the Whatsit building in the
// stomach.   reversion       : a string representing the preferred TeX form of the invocation.
//   beforeDigest    : code to be executed (in the stomach) before parsing & constructing the
// Whatsit.                     Can be used for changing modes, beginning groups, etc.
//   afterDigest     : code to be executed (in the stomach) after parsing & constructing the
// Whatsit.                     useful for setting Whatsit properties,
//   properties      : a hashref listing default values of properties to assign to the Whatsit.
//                     These properties can be used in the constructor.
#[macro_export]
macro_rules! DefConstructor {
  // Closure replacement flavors
  ($proto:literal, sub [ $document:ident, $args:ident, $props:ident ]
    $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    let compiled_replacement : Option<ReplacementClosure> = Some(Rc::new(
      replacement!($document, $args, $props, $body)));
    let (cs, params) = parse_prototype!($proto);
    def_constructor(cs, params, compiled_replacement, options);
  }};
  ($proto:literal, sub [ $document:ident, $args:ident ]
    $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    let _props : ConstructorOptions;
    let compiled_replacement : Option<ReplacementClosure> = Some(Rc::new(
      replacement!($document, $args, _props, $body)));
    let (cs, params) = parse_prototype!($proto);
    def_constructor(cs, params, compiled_replacement, options);
  }};
  ($proto:literal, sub [ $document:ident] $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    let _props : ConstructorOptions;
    let _args : Option<Parameters> = None;
    let compiled_replacement : Option<ReplacementClosure> = Some(Rc::new(
      replacement!($document, _args, _props, $body)));
    let (cs, params) = parse_prototype!($proto);
    def_constructor(cs, params, compiled_replacement, options);
  }};
  ($proto:literal, $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    let _props : ConstructorOptions;
    let _args : Option<Parameters> = None;
    let compiled_replacement : Option<ReplacementClosure> = Some(Rc::new(
      replacement!(_document, _args, _props, $body)));
    let (cs, params) = parse_prototype!($proto);
    def_constructor(cs, params, compiled_replacement, options);
  }};

  // Literal replacement flavors
  ($proto:expr_2021, $replacement:literal) => {{
    let (cs, params) = parse_prototype!($proto);
    let compiled_replacement;
    compile_replacement!(compiled_replacement, $replacement);
    def_constructor(cs, params, compiled_replacement, ConstructorOptions::default());
  }};
  // Pre-parsed prototype flavors
  // Pre-parsed prototype; Closure replacement flavors
  ($cs:expr_2021, $parameters:expr_2021, sub [ $document:ident, $args:ident, $props:ident]
    $body:block, $($input:tt)+) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    let compiled_replacement : Option<ReplacementClosure>= Some(Rc::new(
      replacement!($document, $args, $props, $body)));
    def_constructor($cs, $parameters, compiled_replacement, options);
  }};
  ($cs:expr_2021, $parameters:expr_2021, sub [ $document:ident, $args:ident, $props:ident]
    $body:block) => {{
    let compiled_replacement : Option<ReplacementClosure>= Some(Rc::new(
      replacement!($document, $args, $props, $body)));
    def_constructor($cs, $parameters, compiled_replacement, ConstructorOptions::default());
  }};
  //  Pre-parsed prototype; Literal replacement flavors
  ($cs:expr_2021, $parameters:expr_2021, $replacement:literal) => {{
    let compiled_replacement;
    compile_replacement!(compiled_replacement, $replacement);
    def_constructor($cs, $parameters, compiled_replacement, ConstructorOptions::default());
  }};
  ($cs:expr_2021, $parameters:expr_2021, None) => {{
    def_constructor($cs, $parameters, $replacement, ConstructorOptions::default());
  }};

  // Optioned flavors come last due to :tt munching
  ($cs:expr_2021, $parameters:expr_2021, $replacement:literal, $($input:tt)+) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    let compiled_replacement;
    compile_replacement!(compiled_replacement, $replacement);
    def_constructor($cs, $parameters, compiled_replacement, options);
  }};
  ($cs:expr_2021, $parameters:expr_2021, None, $($input:tt)+) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    def_constructor($cs, $parameters, None, options);
  }};
  ($proto:expr_2021, $replacement:literal, $($input:tt)+) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    let (cs, params) = parse_prototype!($proto);
    let compiled_replacement;
    compile_replacement!(compiled_replacement, $replacement);
    def_constructor(cs, params, compiled_replacement, options);
  }};
}

/// A macro that uses latexml_codegen to expand a prototype at compile time,
/// and initialize all parameters with the in-scope state::at runtime.
/// Returns a (cs, parameters) pair.
#[macro_export]
macro_rules! parse_prototype(
  // literals get handled at compile time
  ($proto:literal) => {{
    let (cs, params_opt) : (Token,Option<Parameters>) = compile_prototype!($proto);
    match params_opt {
      Some(params) => (cs, Some(params.init()?)),
      None => (cs, None),
    }
  }};
  ($proto:literal) => {{
    let (cs, params_opt) : (Token,Option<Parameters>) = compile_prototype!($proto);
    match params_opt {
      Some(params) => (cs, Some(params.init()?)),
      None => (cs, None),
    }
  }};
  // expressions get handled at runtime
  ($proto:expr_2021) => {{
    latexml_core::common::def_parser::parse_prototype($proto, true)? }};
);

#[macro_export]
macro_rules! DefEnvironmentWO (
  ($proto_raw:expr_2021, $replacement:expr_2021, $options:expr_2021) => ({
  use latexml_core::util::text::*;
  let mut proto = $proto_raw.to_string().trim_start().to_string();
  let name = extract_bracketed(&mut proto, Some(&Delimiter::Brace)).unwrap_or_default();
  let paramlist_str = proto.trim_start().to_string();
  let paramlist = if paramlist_str.is_empty() {
    None
  } else {
    let cs = T_CS!(s!("\\{}", &name));
    parse_parameters(&paramlist_str, &cs, true)?
  };
  let compiled_replacement;
  compile_replacement!(compiled_replacement, $replacement);

  let options = $options;
  def_environment(name, paramlist, compiled_replacement, options);
}));

#[macro_export]
macro_rules! DefEnvironmentIWO (
  ($proto_raw:expr_2021, $compiled_replacement:expr_2021, $options:expr_2021) => ({
  use latexml_core::util::text::*;
  let mut proto = $proto_raw.to_string().trim_start().to_string();
  let name = extract_bracketed(&mut proto, Some(&Delimiter::Brace)).unwrap_or_default();
  let paramlist_str = proto.trim_start().to_string();
  let paramlist = if paramlist_str.is_empty() {
    None
  } else {
    let cs = T_CS!(s!("\\{}", &name));
    parse_parameters(&paramlist_str, &cs, true)?
  };
  def_environment(name, paramlist, $compiled_replacement, $options);
}));

#[macro_export]
macro_rules! RelaxNGSchema {
  ($name:expr_2021) => {{ select_relaxng_schema($name, None) }};
}

#[macro_export]
macro_rules! RegisterNamespace(
  ($prefix:expr_2021, $namespace:expr_2021) => {
    model::register_namespace($prefix, Some($namespace));
  };
  ($prefix:expr_2021 => $namespace:expr_2021) => {
    RegisterNamespace!($prefix, $namespace)
  };
);
#[macro_export]
macro_rules! RegisterDocumentNamespace(
  ($prefix:expr_2021, $namespace:expr_2021) => {
    model::register_document_namespace($prefix, Some($namespace))
  }
);
#[macro_export]
macro_rules! RequireResource(
  ($resource:expr_2021) => {
    require_resource(Resource{name: $resource.to_string(), ..Resource::default()})
  }
);

#[macro_export]
macro_rules! defi_math {
  ($cstext:expr_2021, $paramlist:expr_2021, $presentation:expr_2021, $options:expr_2021) => {{
    let options = $options;
    let cs = T_CS!($cstext.to_string());
    let presentation = $presentation.to_string();
    let paramlist: Option<Parameters> = $paramlist;
    def_math(cs, paramlist, presentation, options)?;
  }};
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
macro_rules! CounterValue {
  ($ctr:expr_2021) => {{ counter_value($ctr)? }};
  ($ctr:expr_2021) => {
    counter_value($ctr)?
  };
}

#[macro_export]
macro_rules! AddToCounter {
  ($ctr:expr_2021, $value:expr_2021) => {
    add_to_counter($ctr, $value.into())?
  };
}
#[macro_export]
macro_rules! StepCounter {
  ($ctr:expr_2021, $noreset:expr_2021) => {
    step_counter($ctr, $noreset)
  };
}

/// convenience macro for `api::counter_dialect::ref_step_counter`
#[macro_export]
macro_rules! RefStepCounter {
  ($ctr:expr_2021) => {
    ref_step_counter($ctr, false)
  };
  ($ctr:expr_2021, $noreset:expr_2021) => {
    ref_step_counter($ctr, $noreset)
  };
}

/// convenience macro for `api::counter_dialect::ref_step_id`
#[macro_export]
macro_rules! RefStepID {
  ($ctr:expr_2021) => {
    ref_step_id($ctr)
  };
}
/// convenience macro for `api::counter_dialect::ref_current_id`
/// Recycles the last ID without incrementing — useful when a previous
/// ID-ed item got pruned and we want to reuse its identifier.
#[macro_export]
macro_rules! RefCurrentID {
  ($ctr:expr_2021) => {
    ref_current_id($ctr)
  };
}
#[macro_export]
macro_rules! ResetCounter {
  ($ctr:literal) => {{ reset_counter(&T_OTHER!($ctr))? }};
  ($ctr:expr_2021) => {{ reset_counter($ctr)? }};
  ($ctr:expr_2021) => {
    reset_counter($ctr)?
  };
}

/// Return `tokens` with all tokens expanded
#[macro_export]
macro_rules! Expand {
  ($tokens:expr_2021) => {
    do_expand($tokens)?
  };
}

#[macro_export]
macro_rules! Input {
  ($arg:expr_2021) => {
    ::latexml_core::binding::content::input($arg, InputOptions::default())?
  };
}

// /// Return `tokens` with all partial expandsion
// macro_rules! ExpandPartially {
//   ($tokens:expr) => {
//     do_expand_partially($tokens)?
//   };
// }

/// Builds a representation of a single command sequence invoked on a
/// `Vec<Token>` of its arguments.
/// A leading string argument is tokenized (`TokenizeInternal`); if it yields a single
/// token it is interpreted as that command sequence, otherwise the tokens are treated
/// as an "anonymous macro" containing parameter markers like `#1`, with the arguments
/// substituted in (Perl `Invocation`, Package.pm).
#[macro_export]
macro_rules! Invocation {
  ($csname:literal) => {{ Invocation!($csname, vec![None]) }};
  ($csname:literal, $args:expr_2021) => {{
    build_invocation_str($csname, $args.into_iter().map(Into::into).collect())?
  }};
  ($token:expr_2021) => {
    Invocation!($token, vec![None])
  };
  ($token:expr_2021, $args:expr_2021) => {
    build_invocation($token, $args.into_iter().map(Into::into).collect())?
  };
}
#[macro_export]
macro_rules! DefLigature {
  ($regex:expr_2021, $replacement:expr_2021, fontTest => sub[$font:ident] $body:block) => {
    let regex_compiled = Regex::new($regex).unwrap();
    let test_closure: Option<FontTestClosure> = Some(Rc::new(move |$font| $body));
    let new_ligature_id = generate_ligature_id();
    unshift_value("TEXT_LIGATURES", vec![Ligature {
      id:        new_ligature_id,
      regex:     Some($regex.to_string()),
      code:      Some(Rc::new(move |text| {
        regex_compiled.replace_all(text, $replacement).to_string()
      })),
      font_test: test_closure,
      matcher:   None,
    }]);
  };
  ($regex:expr_2021, $replacement:expr_2021) => {
    let regex_compiled = Regex::new($regex).unwrap();
    let new_ligature_id = generate_ligature_id();
    unshift_value("TEXT_LIGATURES", vec![Ligature {
      id:        new_ligature_id,
      regex:     Some($regex.to_string()),
      code:      Some(Rc::new(move |text| {
        regex_compiled.replace_all(text, $replacement).to_string()
      })),
      font_test: None,
      matcher:   None,
    }]);
  };
}

// Defines an accent command using a combining char that follows the
// 1st char of the argument.  In cases where there is no argument, `standalonechar` is used.
#[macro_export]
macro_rules! DefAccent {
  ($accent:literal, $combiningchar:expr_2021, $standalonechar:expr_2021) => {{
    DefAccent!($accent, $combiningchar, $standalonechar, HashMap::default())
  }};
  ($accent:literal, $combiningchar:expr_2021, $standalonechar:expr_2021, below => true) => {{
    DefAccent!($accent, $combiningchar, $standalonechar, map!("below"=>Stored::Bool(true)))
  }};
  ($accent:literal, $combiningchar:expr_2021, $standalonechar:expr_2021, $options:expr_2021) => {{
    let mut options : HashMap<String, Stored> = $options;
    if !options.contains_key("above") &&
      !options.get("below").map(|v| matches!(v, Stored::Bool(true))).unwrap_or(false) {
        options.insert("above".to_string(), Stored::Bool(true));
    }
    // Used for converting a char used as an above-accent to a combining char (See \accent)
    if options.get("above").map(|v| matches!(v, Stored::Bool(true))).unwrap_or(false) {
      assign_mapping("accent_combiner_above", $standalonechar, Some($combiningchar));
    } else {
      assign_mapping("accent_combiner_below", $standalonechar, Some($combiningchar));
    }
    let plain_param = Some(Parameters::new(vec![Parameter {
      name: arena::pin_static("Plain"), spec: arena::pin_static("{}"), ..Parameter::default()
      }.init()?
    ]));
    def_macro(T_CS!($accent), plain_param, ExpansionBody::Tokens(Tokens!(
        T_CS!("\\lx@applyaccent"), T_OTHER!($accent),
        T_OTHER_CHAR!($combiningchar), T_OTHER!($standalonechar),
        T_BEGIN!(), T_ARG!(1), T_END!())),
      Some(ExpandableOptions{protected: true, ..ExpandableOptions::default()}))?;
  }};
}

//============================================
// User-facing Macros
//============================================
//
#[macro_export]
macro_rules! LookupBool {
  ($name:expr_2021) => {{ state::lookup_bool($name) }};
}
#[macro_export]
macro_rules! LookupFont {
  () => {{ state::lookup_font() }};
}
#[macro_export]
macro_rules! LookupString {
  ($name:expr_2021) => {{ state::lookup_string($name) }};
}
#[macro_export]
macro_rules! LookupNumber {
  ($name:expr_2021) => {{ state::lookup_number($name) }};
}
#[macro_export]
macro_rules! LookupTokens {
  ($name:expr_2021) => {{ state::lookup_tokens($name) }};
}
#[macro_export]
macro_rules! AssignValue {
  ($name:expr_2021 => $value:expr_2021) => {
    AssignValue!($name, $value)
  };
  ($name:expr_2021 => $value:expr_2021, $scope:expr_2021) => {
    AssignValue!($name, $value, $scope)
  };
  ($name:expr_2021, $value:expr_2021) => {{ state::assign_value($name, $value, None) }};
  ($name:expr_2021, $value:expr_2021, $scope:expr_2021) => {{ state::assign_value($name, $value, $scope) }};
  ($name:expr_2021, $value:expr_2021, $scope:expr_2021) => {
    state::assign_value($name, $value, $scope)
  };
}

#[macro_export]
macro_rules! AssignMapping {
  ($map:expr_2021, $key:expr_2021 => $value:expr_2021) => {
    assign_mapping($map, $key, $value.into())
  };
}
#[macro_export]
macro_rules! AssignMeaning {
  ($key:expr_2021, $val:expr_2021) => {
    AssignMeaning!($key, $val, None)
  };
  ($key:expr_2021, $val:expr_2021, $scope: expr_2021) => {{ assign_meaning($key, $val, $scope) }};
}

#[macro_export]
macro_rules! LookupCatcode {
  ($c:expr_2021) => {{ state::lookup_catcode($c) }};
}
#[macro_export]
macro_rules! AssignCatcode {
  ($name:expr_2021 => $value:expr_2021) => {
    AssignCatcode!($name, $value)
  };
  ($c:expr_2021, $catcode:expr_2021) => {{ AssignCatcode!($c, $catcode, None) }};
  ($c:expr_2021, $catcode:expr_2021, $scope:expr_2021) => {{ assign_catcode($c, $catcode, $scope) }};
}
#[macro_export]
macro_rules! LookupMeaning {
  ($name:expr_2021) => {
    state::lookup_meaning($name)
  };
}
#[macro_export]
macro_rules! LookupDefinition {
  ($name:expr_2021) => {
    state::lookup_definition($name)?
  };
}
#[macro_export]
macro_rules! InstallDefinition {
  ($name:expr_2021, $definition:expr_2021, $scope:expr_2021) => {
    state::install_definition($name, $definition, $scope)
  };
}
#[macro_export]
macro_rules! XEquals {
  ($token1:expr_2021, $token2:expr_2021) => {
    state::x_equals($token1, $token2)
  };
}
#[macro_export]
macro_rules! IsDefined {
  ($name:expr_2021) => {
    is_defined_token($name)
  };
}
#[macro_export]
macro_rules! IsDefinedToken {
  ($name:expr_2021) => {{ is_defined_token($name) }};
}
#[macro_export]
macro_rules! IsDefinable {
  ($token: expr_2021) => {
    is_definable($token)
  };
}

#[macro_export]
macro_rules! Let {
  ($token1:literal, $token2:literal) => {
    state::let_i(&T_CS!($token1), &T_CS!($token2), None)
  };
  ($token1:literal, $token2:literal, None) => {
    state::let_i(&T_CS!($token1), &T_CS!($token2), None)
  };
  ($token1:literal, $token2:literal, $scope:expr_2021) => {
    state::let_i(&T_CS!($token1), &T_CS!($token2), Some($scope))
  };
  // half-packaged args
  ($token1:literal, $token2:expr_2021) => {
    state::let_i(&T_CS!($token1), &$token2, None)
  };
  ($token1:expr_2021, $token2:literal) => {
    state::let_i(&$token1, &T_CS!($token2), None)
  };
  // internal form, pre-packaged arguments
  ($token1:expr_2021, $token2:expr_2021) => {
    state::let_i(&$token1, &$token2, None)
  };
  ($token1:expr_2021, $token2:expr_2021, None) => {
    state::let_i(&$token1, &$token2, None)
  };
  ($token1:expr_2021, $token2:expr_2021, $scope:expr_2021) => {
    state::let_i(&$token1, &$token2, Some($scope))
  };
}

#[macro_export]
macro_rules! DigestIf {
  ($token:literal) => {
    digest_if(T_CS!($token))
  };
  ($token:expr_2021) => {
    digest_if($token)
  };
}

/// Merge the current font with the style specifications.
///
/// Supports three forms, all mirroring Perl `MergeFont(...)`:
/// - `MergeFont!(font_expr)` — pass a pre-built `Font` (or fontmap) expression.
/// - `MergeFont!(family => "typewriter")` — single `key => value`.
/// - `MergeFont!(family => "math", shape => "italic")` — multi-key, faithful to Perl's
///   `MergeFont(family => 'math', shape => 'italic')`.
#[macro_export]
macro_rules! MergeFont {
  ($kv:expr_2021) => {
    merge_font($kv)
  };
  ($($key:ident => $val:expr_2021),+ $(,)?) => {
    merge_font(fontmap!($($key => $val),+))
  };
}

//============================================
// User-facing Argument Parsers
//============================================
//
// There is a lot of "Do What I Mean" logic going on here, to allow binding writers to thoughtlessly
// use a single DefMacro!() and have:
// - the macro machinery auto-wrap the correct union type containers,
// - auto-compile the various string replacements and prototypes into their rust data structures
// - auto-build the ExpansionOptions data structure from a Perl-like syntax, and validate it along
//   the way
//  we're taking things a few pegs further than LaTeXML, as DefMacroI syntax is *included* in
// DefMacro,  and we have a several places where we get compile-time speedups by pre-tokenizing into
// Rust Tokens objects / Replacement closures

/// Defines a macro, a binding analog to `\def`
///
/// A `prototype` will be parsed into a command sequence and a list of parameters.
/// Any macro arguments will be substituted for parameter indicators (eg #1)
/// in the `Tokens` or tokenized string and the result is used as the expansion
/// of the control sequence. If a closure is used, it is called at expansion time
/// and should return a list of tokens as its result.
#[macro_export]
macro_rules! DefMacro {
  // simplest case - mock macro that discards everything.
  ($proto:literal, None) => {
    let (cs, params) = parse_prototype!($proto);
    def_macro(cs, params, None, None)?;
  };
  // closure with literal prototype
  ($prototype:literal, sub [( $($var:ident),* )]
    $body:block $($input:tt)*) => {
    compile_prototype_for_typed_macro!($prototype, sub [ ( $($var),* ) ]
      $body $($input)*)
  };
  // closure, general form
  ($proto:expr_2021, sub [$args:ident]
    $body:block $($input:tt)*) => {
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    let (cs, params) = parse_prototype!($proto);
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(
      Rc::new(move |$args| $body.into_tokens_result())));
    def_macro(cs, params, expansion_closure, Some(options))?;
  };
  ($proto:expr_2021, $body:block $($input:tt)*) => {
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    let (cs, params) = parse_prototype!($proto);
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |_args| $body.into_tokens_result()
    )));
    def_macro(cs, params, expansion_closure, Some(options))?;
  };
  // String; implicit state
  ($proto:literal, $expansion:literal $($input:tt)*) => {
    let mut options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    options.nopack_parameters = true; // compile_expansion! already packs parameters at compile time
    let (cs, params) = parse_prototype!($proto);
    let compiled_expansion;
    compile_expansion!(compiled_expansion, $expansion);
    def_macro(cs, params, compiled_expansion, Some(options))?;
  };
  // Internal-level use
  ($cs:expr_2021, $parameters:literal, $expansion: literal) => {
    let cs = $cs;
    let params = parse_parameters($parameters, &cs, true)?;
    let compiled_expansion;
    compile_expansion!(compiled_expansion, $expansion);
    def_macro(cs, params, compiled_expansion, None)?;
  };
  // ($cs:expr, $params_str:literal, sub [$args] $body) — non-classical CS
  // names (expl3 `\__foo:nn`, `\g_x_int`, etc.) where the leading
  // `parse_prototype!` token-tokenizer can't represent the colon/underscore
  // letters. Caller hands us the CS as an expression (typically `T_CS!(...)`)
  // plus a parameters string. We invoke `parse_parameters` at runtime to
  // build the Parameters from the literal — same shape as the
  // ($proto:literal, sub [$args] $body) arm.
  ($cs:expr_2021, $parameters:literal, sub [$args:ident]
    $body:block $($input:tt)*) => {
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    let cs_expr = $cs;
    let parsed_params = parse_parameters($parameters, &cs_expr, true)?;
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |$args| $body.into_tokens_result()
    )));
    def_macro(cs_expr, parsed_params, expansion_closure, Some(options))?;
  };
  // Same as above, but with parenthesized named args:
  //   DefMacro!(T_CS!("\\__foo:nn"), "{}{}", sub[(case, cp)] { ... });
  ($cs:expr_2021, $parameters:literal, sub [( $($var:ident),* )]
    $body:block $($input:tt)*) => {
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    let cs_expr = $cs;
    let parsed_params = parse_parameters($parameters, &cs_expr, true)?;
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |args: Vec<ArgWrap>| {
        let mut iter = args.into_iter();
        $( let $var = iter.next().unwrap_or(ArgWrap::None); )*
        $body.into_tokens_result()
      }
    )));
    def_macro(cs_expr, parsed_params, expansion_closure, Some(options))?;
  };
  ($cs:expr_2021, $parameters:expr_2021, sub [$args:ident]
    $body:block $($input:tt)*) => {
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |$args| $body.into_tokens_result()
    )));
    def_macro($cs, $parameters, expansion_closure, Some(options))?;
  };
  ($cs:literal, None, $expansion:literal) => {
    let compiled_expansion;
    compile_expansion!(compiled_expansion, $expansion);
    def_macro(T_CS!($cs), None, compiled_expansion, Some(ExpandableOptions {
      nopack_parameters: true, ..ExpandableOptions::default()
    }))?;
  };
  ($cs:literal, None, $expansion:literal, $($input:tt)*) => {
    let mut options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    options.nopack_parameters = true; // compile_expansion! already packs parameters
    let compiled_expansion;
    compile_expansion!(compiled_expansion, $expansion);
    def_macro(T_CS!($cs), None, compiled_expansion, Some(options))?;
  };
  ($cs:literal, None, $expansion:expr_2021) => {
    def_macro(T_CS!($cs), None, $expansion, None)?;
  };
  ($cs:expr_2021, None, $expansion:literal) => {
    let compiled_expansion;
    compile_expansion!(compiled_expansion, $expansion);
    def_macro($cs, None, compiled_expansion, Some(ExpandableOptions {
      nopack_parameters: true, ..ExpandableOptions::default()
    }))?;
  };
  ($cs:expr_2021, None, $body:block) => {
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |_args| $body.into_tokens_result()
    )));
    def_macro($cs, None, expansion_closure, None)?;
  };
  ($cs:expr_2021, None, $expansion:expr_2021) => {
    def_macro($cs, None, $expansion, None)?;
  };
  ($cs:expr_2021, None, $expansion:literal, $($input:tt)+) => {
    let compiled_expansion;
    compile_expansion!(compiled_expansion, $expansion);
    let mut options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    options.nopack_parameters = true; // compile_expansion! already packs parameters
    def_macro($cs, None, compiled_expansion, Some(options))?;
  };
  ($cs:expr_2021, None, $expansion:expr_2021, $($input:tt)+) => {
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    def_macro($cs, None, $expansion, Some(options))?;
  };
  // the triple expr case should be near the end, as it matches too many cases.
  // It's an internal use of DefMacro e.g. with 3 variable name arguments
  ($cs:expr_2021, $parameters:expr_2021, $expansion:expr_2021) => {{
    def_macro($cs, $parameters, $expansion, None)?;
  }};
  // The least-specified option-parsing cases come last due to the TT munchers accepting any inputs
  ($proto:literal, None $($input:tt)*) => {
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    let (cs, params) = parse_prototype!($proto);
    def_macro(cs, params, None, Some(options))?;
  };
  ($cs:expr_2021, $replacement:expr_2021, $expansion:expr_2021, $($input:tt)*) => {
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    def_macro($cs, $replacement, $expansion, Some(options))?;
  };
}

#[macro_export]
macro_rules! TypedMacro {
  ( $cs:literal, $these_parameters:ident,
    sub [( $($var:ident),* ):($($ptype:ident),*)]
    $body:block $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ExpandableOptions,});
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |args: Vec<ArgWrap>| {
        let [$($var),*] : [_; $crate::count!($($var)*)] = args.try_into().unwrap();
        $(
          // TODO: How do we fine-tune the match body based on whether we have an Infallible try_into?
          #[allow(warnings)]
          let $var: parameter_rust_type!($ptype) = match $var.try_into() {
            Ok(v) => v,
            Err(e) => {
              Error!("expected", "argument", e);
              <parameter_rust_type!($ptype)>::default()
            }
          };
        )*
        $body.into_tokens_result()
      }
    )));
    def_macro(T_CS!($cs), $these_parameters, expansion_closure, Some(options))?;
  }};
}

/// Defines a register with `value` as the initial value
///
/// (a Number, Dimension, Glue, MuGlue or Tokens --- I haven't handled Box's yet).
/// Usually, the `prototype` is just the control sequence,
/// but registers are also handled by prototypes like `\count{Number}`. `DefRegister` arranges
/// that the register value can be accessed when a numeric, dimension, ... value is being read,
/// and also defines the control sequence for assignment.
#[macro_export]
macro_rules! DefRegister {
  ($proto:expr_2021 => $value:expr_2021) => {{
    let (cs, params) = parse_prototype!($proto);
    defi_register!(cs, params, $value, None);
  }};
  ($proto:expr_2021, $value:expr_2021) => {{
    let (cs, params) = parse_prototype!($proto);
    defi_register!(cs, params, $value, None);
  }};
  ($cs:expr_2021, None, $value:expr_2021) => {{
    defi_register!($cs, None, $value, None);
  }};
  // Option parsers are more lenient, should be at the end of the list of patterns
  ($cs:expr_2021, None, $value:expr_2021, $($input:tt)+) => {{
    let options = defi_opts!(@munch ($($input)*) -> {RegisterOptions,});
    defi_register!($cs, None, $value, Some(options));
  }};
  ($proto:expr_2021, $value:expr_2021, $($input:tt)+) => {{
    let (cs, params) = parse_prototype!($proto);
    let options = defi_opts!(@munch ($($input)*) -> {RegisterOptions,});
    defi_register!(cs, params, $value, Some(options));
  }};
  ($proto:expr_2021 => $value:expr_2021, $($input:tt)+) => {{
    let (cs, params) = parse_prototype!($proto);
    let options = defi_opts!(@munch ($($input)*) -> {RegisterOptions,});
    defi_register!(cs, params, $value, Some(options));
  }};
}

#[macro_export]
macro_rules! defi_register {
  ($cs:expr_2021, $paramlist:expr_2021, $value:expr_2021, $options:expr_2021) => {{
    let value = { $value };
    def_register($cs, $paramlist, value, $options)?
  }};
}

#[macro_export]
macro_rules! NewCounter {
  ($ctr:expr_2021) => (new_counter($ctr, "", None)?);
  ($ctr:expr_2021, $within:expr_2021) => (new_counter($ctr, $within, None)?);
  ($ctr:expr_2021, $within:expr_2021, $($key:ident => $val:expr_2021),*) => (
    new_counter($ctr, $within, Some(NewDefault!(NewCounterOptions, $($key=>$val),*)))?);
}

//=====================================================================
// Define a LaTeX environment
// Note that the body of the environment is treated is the 'body' parameter in the constructor.
#[macro_export]
macro_rules! DefEnvironment {
  // entry points (this is where a macro call starts):
  ($proto:literal, sub[$document:ident, $args:ident, $props:ident]
    $body:block, $($input:tt)+ ) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    DefEnvironmentIWO!($proto,
      Some(Rc::new(|$document: &mut Document, $args: &Vec<Option<Digested>>,
        $props: &SymHashMap<Stored>| $body
      )),
      options);
  }};
  ($proto:literal, sub[$document:ident, $args:ident, $props:ident]
    $body:block) => {{
    let options = ConstructorOptions::default();
    DefEnvironmentIWO!($proto,
      Some(Rc::new(|$document: &mut Document, $args: &Vec<Option<Digested>>,
        $props: &SymHashMap<Stored>| $body
      )),
      options);
  }};
  ($proto:literal, $replacement:expr_2021) => {
    DefEnvironmentWO!($proto, $replacement, ConstructorOptions::default());
  };
  ($proto:literal, $replacement:expr_2021, $($input:tt)* ) => {{
    let options = defi_opts!(@munch ($($input)*) -> {ConstructorOptions,});
    //                              ^^^^^^^^^^^^    ^^^^^^^^^^^^^^^^^^^^
    //                                 input       output
    DefEnvironmentWO!($proto, $replacement, options);
  }};
}

#[macro_export]
macro_rules! Tag {
  ($tag:expr_2021, $($input:tt)*) => {{
    let options = defi_opts!(@munch ($($input)*) -> {TagOptions,});
    install_tag($tag, options);
  }}
}

#[macro_export]
macro_rules! DefMath(
  ($proto:literal,$presentation:literal) => {{
    let (cs, paramlist) = parse_prototype!($proto);
    let defmath_options = MathPrimitiveOptions::default();
    defi_math!(cs,paramlist, $presentation, defmath_options);
  }};
  ($proto:literal,$presentation:literal, $($input:tt)*) => {{
    let (cs, paramlist) = parse_prototype!($proto);
    let defmath_options = defi_opts!(@munch ($($input)*) -> {MathPrimitiveOptions,});
    defi_math!(cs,paramlist, $presentation, defmath_options);
  }};
  ($text:expr_2021,$paramlist:expr_2021,$presentation:expr_2021) => (
    defi_math!($text,$paramlist, $presentation, MathPrimitiveOptions::default()));
  ($text:expr_2021,$paramlist:expr_2021,$presentation:expr_2021, $($input:tt)*) => {{
    let defmath_options = defi_opts!(@munch ($($input)*) -> {MathPrimitiveOptions,});
    defi_math!($text,$paramlist, $presentation, defmath_options);
  }};
);

#[macro_export]
macro_rules! DefParameterType {
  ($name:ident, $($key:ident => $value:expr_2021),*)=>(
    DefParameterTypeWO!($name, NewDefault!(Parameter, name => arena::pin_static(stringify!($name)),
    $($key=>$value),*)));
  // with reader as explicit sub
  ($name:ident, sub[$inner:ident, $extra:ident] $body:block) => (
    DefParameterTypeWO!($name, NewDefault!(Parameter, reader =>
      reader!($inner, $extra, $body))));
  // fully advanced version, including e.g. inner sub[] patterns for before_digest, after_digest,...
  ($name:ident, sub[$inner:ident, $extra:ident] $body:block, $($input:tt)+) => (
      let mut paramtype_options = defi_opts!(@munch ($($input)*) -> {Parameter,});
      paramtype_options.reader = reader!($inner, $extra, $body);
      paramtype_options.name = arena::pin_static(stringify!($name));
      DefParameterTypeWO!($name, paramtype_options));
}

#[macro_export]
macro_rules! DefColumnType {
  ($proto:literal, sub[$args:ident] $body:block) => {
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |$args| $body.into_tokens_result()
    )));
    DefColumnType!($proto, expansion_closure)
  };

  ($prototype:literal, sub [( $($var:ident),* )]
    $body:block) => {{
    compile_prototype_for_typed_columntype!($prototype, sub [ ( $($var),* ) ] $body)
  }};
  ($proto:literal, $body:block) => {
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |_args| $body.into_tokens_result()
    )));
    DefColumnType!($proto, expansion_closure)
  };
  ($proto:literal, $expansion_closure:ident) => {
    let mut c_chars = $proto.chars();
    if let Some(first_c) = c_chars.next() {
      let mut c_chars_peek = c_chars.peekable();
      while c_chars_peek.peek() == Some(&' ') {
        c_chars_peek.next();
      }
      let proto = parse_parameters(&c_chars_peek.collect::<String>(), &T_RELAX!(), true)?;
      def_macro(
        T_CS!(s!("\\NC@rewrite@{first_c}")),
        proto,
        $expansion_closure,
        None
      )?;
    } else {
      Warn!(
        "expected",
        "character",
        "Expected Column specifier"
      );
    }
  };
}

#[macro_export]
macro_rules! TypedColumntype {
  ($first_c:literal, $these_parameters:ident,
      sub [( $($var:ident),* ):($($ptype:ident),*)]
      $body:block $($input:tt)*) => {{
    let expansion_closure: Option<ExpansionBody> = Some(ExpansionBody::Closure(Rc::new(
      move |args: Vec<ArgWrap>| {
        let [$($var),*] : [_; $crate::count!($($var)*)] = args.try_into().unwrap();
        $(
          // TODO: How do we fine-tune the match body based on whether we have an Infallible try_into?
          #[allow(warnings)]
          let $var: parameter_rust_type!($ptype) = match $var.try_into() {
            Ok(v) => v,
            Err(e) => {
              Error!("expected", "argument", e);
              <parameter_rust_type!($ptype)>::default()
            }
          };
        )*
        $body.into_tokens_result()
      }
    )));
    def_macro(
      T_CS!(concat!("\\NC@rewrite@",$first_c)),
      $these_parameters,
      expansion_closure,
      None
    )?;
  }}
}

// Reverts an object into TeX code, as a Tokens list, that would create it.
// Note that this is not necessarily the original TeX.
#[macro_export]
macro_rules! Revert {
  ($thing:literal) => {
    Explode!($thing)
  };
  ($thing:expr_2021) => {{ $thing.revert()?.unlist() }};
}

#[macro_export]
macro_rules! GetKeyVal {
  ($keyval_opt:expr_2021, $key:expr_2021) => {
    match $keyval_opt {
      Some(digested) => match digested.data() {
        DigestedData::KeyVals(keyval) => keyval.get_value($key),
        _ => None,
      },
      _ => None,
    }
  };
}

#[macro_export]
macro_rules! GetKeyVals {
  ($keyval:expr_2021) => {
    match $keyval_opt {
      Some(Digested::KeyVals(keyval)) => keyval.get_key_vals(),
      _ => None,
    }
  };
}

/// Defines a new KeyVal Parameter in the given `keyset`, `key`
/// and with optional prefix `option.prefix`.
/// For descriptions of further parameters, see `keyval::define`.
#[macro_export]
macro_rules! DefKeyVal {
  ($keyset:expr_2021, $key:expr_2021, $vtype:expr_2021) => {{
    ::latexml_core::keyval::define(KeyvalConfig {
      prefix: "KV",
      keyset: $keyset,
      key: $key,
      vtype: $vtype,
      default: None,
      ..KeyvalConfig::default()
    })?;
  }};
  ($keyset:expr_2021, $key:expr_2021, $vtype:expr_2021, $default:expr_2021) => {{
    ::latexml_core::keyval::define(KeyvalConfig {
      prefix: "KV",
      keyset: $keyset,
      key: $key,
      vtype: $vtype,
      default: Some($default),
      ..KeyvalConfig::default()
    })?;
  }};
  ($keyset:expr_2021, $key:expr_2021, $vtype:expr_2021, $default:expr_2021, $options:tt) => {{
    // TODO: explicit $options with prefix logic — for now ignore options and use default prefix
    log::warn!(
      "DefKeyVal with explicit options not fully ported, ignoring options for {}/{}",
      $keyset,
      $key
    );
    ::latexml_core::keyval::define(KeyvalConfig {
      prefix: "KV",
      keyset: $keyset,
      key: $key,
      vtype: $vtype,
      default: Some($default),
      ..KeyvalConfig::default()
    })?;
  }};
}

#[macro_export]
macro_rules! Digest {
  ($string:literal) => {{
    let tokenized;
    compile_tokenize_internal!(tokenized, $string);
    stomach::digest(tokenized)
  }};

  ($tokens:expr_2021) => {{ stomach::digest($tokens) }};
}

#[macro_export]
macro_rules! DigestText {
  ($tokens:expr_2021) => {{ digest_text($tokens) }};
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
  ($string:expr_2021) => {
    mouth::tokenize($string)
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
  ($string:expr_2021) => {
    mouth::tokenize_internal($string)
  };
}

#[macro_export]
macro_rules! RawTeX {
  ($text:expr_2021) => {
    ::latexml_core::stomach::raw_tex($text)?;
  };
}

#[macro_export]
macro_rules! TeX {
  ($text:literal) => {
    let tokenized;
    compile_tokenize_internal!(tokenized, $text);
    ::latexml_core::stomach::digest(tokenized)?;
  };
}

#[macro_export]
macro_rules! Dimension {
  ($number:expr_2021) => {
    Dimension::new_f64(Dimension::spec_to_f64($number)?)
  };
}

#[macro_export]
macro_rules! Glue {
  ($spec:expr_2021) => {{ Glue::new_spec($spec, None, None, None, None) }};
}

#[macro_export]
macro_rules! MuGlue {
  ($spec:expr_2021) => {{ MuGlue::new_spec($spec, None, None, None, None) }};
}

/// Register document namespaces. Replaces the old `DocType!` macro (DTD not supported in Rust
/// port). The root element, public ID, and system ID arguments are accepted for compatibility but
/// ignored.
#[macro_export]
macro_rules! RegisterDocumentNamespaces {
  ($rootelement:expr_2021, $pubid:expr_2021, $sysid:expr_2021) => {
    // No-op: DTD schema type not supported. Arguments retained for documentation/compatibility.
  };
  ($rootelement:expr_2021, $pubid:expr_2021, $sysid:expr_2021, $namespaces:expr_2021) => {{
    for (prefix, value) in $namespaces.iter() {
      model::register_document_namespace(prefix, Some(value));
    }
  }};
}

#[macro_export]
macro_rules! Today {
  () => {{ today()? }};
}

#[macro_export]
macro_rules! DeclareOption {
  (None, $tokenized:ident) => {
    let cs = String::from("\\default@ds");
    def_macro(T_CS!(cs), None, $tokenized, None)?;
  };
  (None, $(sub)? $body:block) => {
    let cs = String::from("\\default@ds");
    // block case, create a primitive
    let code: PrimitiveBody =
      PrimitiveBody::Closure(Rc::new(move |_args| $body.into_digested_result()));
    def_primitive(T_CS!(cs), None, Some(code), PrimitiveOptions::default())?;
  };
  ($option:expr_2021, $tex:literal) => {
    let tokenized;
    compile_tokenize_internal!(tokenized, $tex);
    state::push_value("@declaredoptions", $option)?;
    let cs = s!("\\ds@{}", $option);
    // literal case, create a macro
    def_macro(T_CS!(cs), None, tokenized, None)?;
  };
  ($option:expr_2021, $tokenized:ident) => {
    state::push_value("@declaredoptions", $option.to_string())?;
    let cs = s!("\\ds@{}", $option);
    // literal case, create a macro
    def_macro(T_CS!(cs), None, $tokenized, None)?;
  };
  ($option:expr_2021, $(sub)? $body:block) => {
    state::push_value("@declaredoptions", $option)?;
    let cs = s!("\\ds@{}", $option);
    // block case, create a primitive
    let code: PrimitiveBody =
      PrimitiveBody::Closure(Rc::new(move |_args| $body.into_digested_result()));
    def_primitive(T_CS!(cs), None, Some(code), PrimitiveOptions::default())?;
  };
}

#[macro_export]
macro_rules! ProcessOptions {
  // ProcessOptions!() — non-star, declared order (inorder=false)
  () => {
    process_options(false, &[])?;
  };
  // ProcessOptions!(*) — star variant, in-order processing (inorder=true)
  (*) => {
    process_options(true, &[])?;
  };
  // ProcessOptions!(keysets => ["LTXML"]) — Perl
  // ProcessOptions(inorder => 1, keysets => ['LTXML'])
  (keysets => [$($keyset:expr_2021),+ $(,)?]) => {
    process_options(true, &[$($keyset),+])?;
  };
}

#[macro_export]
macro_rules! AddToMacro {
  ($cs:literal, $tokens:literal) => {{
    let into_cs = T_CS!($cs);
    let into_tokens = TokenizeInternal!($tokens);
    AddToMacro!(into_cs, into_tokens);
  }};
  ($cs:ident, $tokens:ident) => {{
    match state::lookup_definition(&$cs)? {
      None => {
        // Perl: InputDefinitions pre-defines hooks, but not all paths go through it.
        // When the CS is undefined (e.g., babel's \AtEndOfPackage hooks), create a new
        // expandable with the given tokens instead of just warning.
        def_macro(
          $cs.clone(),
          None,
          ExpansionBody::Tokens($tokens.clone()),
          Some(ExpandableOptions {
            scope: Some(Scope::Global),
            nopack_parameters: true,
            ..ExpandableOptions::default()
          }),
        )?;
      },
      Some(defn) if !defn.is_expandable() => {
        let message = s!("{} is not an expandable control sequence", $cs);
        let message2 = "Ignoring addition";
        Warn!("unexpected", $cs, message, message2);
      },
      Some(defn) => {
        let mut expansion = match defn.get_expansion() {
          // the .clone() call is again avoidable with a careful refactor via e.g. using
          // `.remove_definition` from state (as we're redefining the macro again), and then
          // use a `.remove_expansion` call on defn?
          Some(ExpansionBody::Tokens(tokens)) => tokens.clone().unlist(),
          Some(ExpansionBody::Closure(_)) => {
            let message = s!(
              "{} has a closure body, AddToMacro will *override* with an ExpandableBody[Tokens] ! \
              This is usually in error!",
              $cs
            );
            Warn!("unexpected", "ExpandableBody[Closure]", message);
            Vec::new()
          },
          None => Vec::new(),
        };
        expansion.extend($tokens.unlist());
        // Perl Package.pm:2527 — `local $UNLOCKED = 1` allows the
        // append-redefinition to bypass `:locked` (only an addition,
        // never a replacement of the body's intent).
        let _unlock_guard =
          ::latexml_core::common::local_assignments::local_state_unlocked_guard(true);
        def_macro(
          $cs,
          None,
          ExpansionBody::Tokens(Tokens!(expansion)),
          Some(ExpandableOptions {
            scope: Some(Scope::Global),
            nopack_parameters: true,
            ..ExpandableOptions::default()
          }),
        )?;
      },
    }
  }};
}

#[macro_export]
macro_rules! BeginItemize {
  ($t:literal, $c:literal) => {{ begin_itemize($t, Some($c), BeginItemizeOptions::default()) }};
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
  (@munch ($(,)?) -> {$kind:ident, $([$id:ident @ $body:expr_2021])+ } ) => {
    $kind {
      $($id: $body),*,
      ..$kind::default()
    }
  };
  // reversion: Option<Reversion>
  // DG: this is currently problematic - we seem to have two kinds of reversions, one working after
  //     digestion, and one *not*
  (@munch ( $(,)? reversion $(:)?$(=>)? sub[$whatsit:ident, $args:ident] $body:block $($next:tt)*)
  -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@reversion (sub[$whatsit,$args] $body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? reversion $(:)?$(=>)?
    sub[$arg:ident, $inner:ident, $extra:ident] $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@reversion (sub[$arg,$inner,$extra] $body $($next)*) -> {$kind, $([ $key @ $val ])*})
  };
  (@munch ( $(,)? reversion $(:)?$(=>)? $(sub)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@reversion (sub $body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  // reversion => None means "empty reversion" (disable reversion entirely)
  // Perl: reversion => Tokens() — produces empty tex= attribute
  (@munch ( $(,)? reversion $(:)?$(=>)? None, $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
      [ reversion @ Some(Reversion::Tokens(Tokens!())) ] })
  };
  (@munch ( $(,)? reversion $(:)?$(=>)? None)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ()
      -> {$kind, $( [ $key @ $val ] )*
        [ reversion @ Some(Reversion::Tokens(Tokens!())) ] })
  };
  (@munch ( $(,)? reversion $(:)?$(=>)? $tokens:expr_2021, $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
      [ reversion @ $tokens.into_option() ] })
  };
  (@munch ( $(,)? reversion $(:)?$(=>)? $tokens:expr_2021)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ reversion @ $tokens.into_option() ] })
  };

  // sizer: Option<SizingClosure>
  (@munch ( $(,)? sizer $(:)?$(=>)? sub[$whatsit_arg:ident]
    $body:block $($next:tt)*) ->
  {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )* [
      sizer @ Some(sizersub!($whatsit_arg, $body)) ]})
  };
  (@munch ( $(,)? sizer $(:)?$(=>)? $body:block $($next:tt)*) ->
  {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )* [
      sizer @ Some(sizersub!(_whatsit_arg, $body)) ]})
  };
  (@munch ( $(,)? sizer $(:)?$(=>)? $tokens:expr_2021, $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
      [ sizer @ $tokens.into_option() ] })
  };
  (@munch ( $(,)? sizer $(:)?$(=>)? $tokens:expr_2021)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ sizer @ $tokens.into_option() ] })
  };
  // select: literal string
  (@munch ( $(,)? select $(:)?$(=>)? $tokens:expr_2021, $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
      [ select @ $tokens.into_option() ] })
  };
  (@munch ( $(,)? select $(:)?$(=>)? $tokens:expr_2021)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ select @ $tokens.into_option() ] })
  };
  // select_count: literal number
  (@munch ( $(,)? select_count $(:)?$(=>)? $tokens:expr_2021, $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
      [ select_count @ $tokens.into_option() ] })
  };
  (@munch ( $(,)? select_count $(:)?$(=>)? $tokens:expr_2021)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )*
      [ select_count @ $tokens.into_option() ] })
  };
  // replace: sub
  (@munch ( $(,)? replace $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@replace (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? replace $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@replace ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  // xpath: literal string
  (@munch ( $(,)? xpath $(:)?$(=>)? $tokens:expr_2021, $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
      [ xpath @ $tokens.into_option() ] })
  };
  (@munch ( $(,)? xpath $(:)?$(=>)? $tokens:expr_2021)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ xpath @ $tokens.into_option() ] })
  };

  // mode : Option<TexMode>
  (@munch ( $(,)? mode $(:)?$(=>)? $literal:literal $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
      [ mode @ $literal.into_option() ] })
  };
  // alias : Option<String>
  (@munch ( $(,)? alias $(:)?$(=>)? $literal:literal $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
      [ alias @ $literal.into_option() ] })
  };
  // scope: Option<Scope>
  (@munch ( $(,)? scope $(:)?$(=>)? $scope:expr_2021)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ scope @ $scope.into_option() ] })
  };
  (@munch ( $(,)? scope $(:)?$(=>)? $scope:expr_2021, $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
      [ scope @ $scope.into_option() ] })
  };
  // font: Font
  (@munch ( $(,)? font $(:)?$(=>)? sub [ $font_whatsit:ident]
    $body:block $($next:tt)*) -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )*
      [ font @ Some(FontDirective::Closure(Rc::new(move |$font_whatsit| $body))) ] })
  };
  (@munch ( $(,)? font $(:)?$(=>)? { $($fkey:ident => $fvalue:literal),* } $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )*
      [ font @ FontDirective!($($fkey => $fvalue),*) ] })
  };
  (@munch ( $(,)? font $(:)?$(=>)? $body:block $($next:tt)*) ->
  {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )*
      [ font @ Some(FontDirective::Closure(Rc::new(move |_font_whatsit| $body))) ] })
  };
  (@munch ( $(,)? font $(:)?$(=>)? $props:ident $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )*
      [ font @ $props.map(|v| FontDirective::Asset(Rc::new(v))) ] })
  };
  // properties: PropertiesClosure
  (@munch ( $(,)? properties $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )*
      [ properties @ properties!($body) ] })
  };
  (@munch ( $(,)? properties $(:)?$(=>)?
      sub[$args:ident] $body:block $($next:tt)*)
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )*
      [ properties @ properties!($args, $body) ] })
  };
  (@munch ( $(,)? properties $(:)?$(=>)? $var:ident $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )*
      [ properties @ properties!($var) ] })
  };

  // before_digest_end: Vec<BeforeDigestClosure>
  (@munch ( $(,)? before_digest_end $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@before_digest_end (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? before_digest_end $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@before_digest_end ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };


  // before_digest: Vec<BeforeDigestClosure>
  (@munch ( $(,)? before_digest $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@before_digest (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? before_digest $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@before_digest ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };

  // after_digest: Vec<DigestionClosure>
  (@munch ( $(,)? after_digest $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_digest (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_digest $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_digest ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };

  // after_digest_begin: Vec<DigestionClosure>
  (@munch ( $(,)? after_digest_begin $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_digest_begin (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_digest_begin $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_digest_begin ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };

  // after_digest_body: Vec<DigestionClosure>
  (@munch ( $(,)? after_digest_body $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_digest_body (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_digest_body $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_digest_body ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };

  // before_construct: Vec<ConstructionClosure>
  (@munch ( $(,)? before_construct $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@before_construct (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? before_construct $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@before_construct ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };


  // after_construct: Vec<ConstructionClosure>
  (@munch ( $(,)? after_construct $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_construct (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_construct $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_construct ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_construct $(:)?$(=>)? $var:ident $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [after_construct @ $var]})
  };

  // getter: RegisterGetterClosure
  (@munch ( $(,)? getter $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@getter (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? getter $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@getter ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  // setter: RegisterSetterClosure
  (@munch ( $(,)? setter $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@setter (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? setter $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@setter ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  // after_open: Option<Vec<TagConstructionClosure>>
  (@munch ( $(,)? after_open $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_open (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_open $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_open ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  // after_open_late: Option<Vec<TagConstructionClosure>>
  (@munch ( $(,)? after_open_late $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_open_late (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_open_late $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_open_late ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  // after_close: Option<Vec<TagConstructionClosure>>
  (@munch ( $(,)? after_close $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_close (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_close $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_close ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  // after_close_late: Option<Vec<TagConstructionClosure>>
  (@munch ( $(,)? after_close_late $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_close_late (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? after_close_late $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@after_close_late ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  // auto_open: Option<bool>
  (@munch ( $(,)? auto_open $(:)?$(=>)? $auto:literal $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )* [ auto_open @ $auto.into() ]})
  };
  // auto_close: Option<bool>
  (@munch ( $(,)? auto_close $(:)?$(=>)? $auto:literal $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )* [ auto_close @ $auto.into() ]})
  };

  // DefParameterType options
  // predigest: Vec<ReaderPredigestClosure>
  (@munch ( $(,)? predigest $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@predigest (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@munch ( $(,)? predigest $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@predigest ($body $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  // digested_reversion: Option<DigestedReversionClosure>
  // Perl equivalent: the `reversion` option on DefParameterType, which receives the raw value.
  // Allows parameter types to control reversion formatting from the structured digested data.
  (@munch ( $(,)? digested_reversion $(:)?$(=>)? sub $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@digested_reversion (sub $($next)*) -> {$kind, $( [ $key @ $val ] )*})
  };
  (@digested_reversion (sub[$arg:ident] $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )*
      [digested_reversion @ Some(Rc::new({
        move |$arg: &Digested| -> Result<Tokens> { $body }
      }))]})
  };
  // reversion

  // semiverbatim
  (@munch ( $(,)? semiverbatim $(:)?$(=>)? Some($value:expr_2021))
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch () -> {$kind, $( [ $key @ $val ] )*
      [ semiverbatim @ Some($value) ] })
  };
  (@munch ( $(,)? semiverbatim $(:)?$(=>)? None)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch () -> {$kind, $( [ $key @ $val ] )*
      [ semiverbatim @ None ] })
  };
  (@munch ( $(,)? semiverbatim $(:)?$(=>)? $value:expr_2021, $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $( [ $key @ $val ] )*
      [ semiverbatim @ $value ] })
};
  // ligature options
  // role: literal string
  (@munch ( $(,)? role $(:)?$(=>)? $literal:literal, $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
      [ role @ Some($literal.to_string()) ] })
  };
  (@munch ( $(,)? role $(:)?$(=>)? $literal:literal)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ role @ Some($literal.to_string()) ] })
  };
  // meaning: literal string
  (@munch ( $(,)? meaning $(:)?$(=>)? $literal:literal, $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
      [ meaning @ Some($literal.to_string()) ] })
  };
  (@munch ( $(,)? meaning $(:)?$(=>)? $literal:literal)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )*
      [ meaning @ Some($literal.to_string()) ] })
  };
  // name: literal string
  (@munch ( $(,)? name $(:)?$(=>)? $literal:literal, $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
      [ name @ Some($literal.to_string()) ] })
  };
  (@munch ( $(,)? name $(:)?$(=>)? $literal:literal)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ name @ Some($literal.to_string()) ] })
  };
  // for register
  // address: Option<String>
  (@munch ( $(,)? address $(:)?$(=>)? $idval:expr_2021, $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
      [ address @ Some($idval.to_string()) ] })
  };
  (@munch ( $(,)? address $(:)?$(=>)? $idval:expr_2021)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ name @ Some($idval.to_string()) ] })
  };
  // allocate: Option<String>
  (@munch ( $(,)? allocate $(:)?$(=>)? $idval:expr_2021, $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
      [ allocate @ Some($idval.to_string()) ] })
  };
  (@munch ( $(,)? allocate $(:)?$(=>)? $idval:expr_2021)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [allocate @ Some($idval.to_string()) ]})
  };
  // for defmath
  // stretchy: bool
  (@munch ( $(,)? stretchy $(:)?$(=>)? $flag:literal, $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )* [ stretchy @ Some($flag) ] })
  };
  (@munch ( $(,)? stretchy $(:)?$(=>)? $flag:literal)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ stretchy @ Some($flag) ] })
  };
  // operator_role: string
  (@munch ( $(,)? operator_role $(:)?$(=>)? $flag:literal, $($next:tt)*)
  -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
  defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
    [ operator_role @ Some($flag.to_string()) ] })
  };
  (@munch ( $(,)? operator_role $(:)?$(=>)? $flag:literal)
  -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
  defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )*
    [ operator_role @ Some($flag.to_string()) ] })
  };
  // operator_stretchy: bool
  (@munch ( $(,)? operator_stretchy $(:)?$(=>)? $flag:literal, $($next:tt)*)
  -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
  defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
    [ operator_stretchy @ Some($flag) ] })
  };
  (@munch ( $(,)? operator_stretchy $(:)?$(=>)? $flag:literal)
  -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
  defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ operator_stretchy @ Some($flag) ] })
  };
  // scriptpos: string
  (@munch ( $(,)? scriptpos $(:)?$(=>)? $flag:literal, $($next:tt)*)
  -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
  defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
    [ scriptpos @ Some($flag.to_string()) ] })
  };
  (@munch ( $(,)? scriptpos $(:)?$(=>)? $flag:literal)
  -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
  defi_opts!(@munch ()  -> {$kind, $( [ $key @ $val ] )* [ scriptpos @ Some($flag.to_string()) ] })
  };
  // mathstyle: Option<String>
  (@munch ( $(,)? mathstyle $(:)?$(=>)? $literal:literal, $($next:tt)*)
  -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
  defi_opts!(@munch ($($next)*)  -> {$kind, $( [ $key @ $val ] )*
    [ mathstyle @ Some($literal.to_string()) ] })
  };
  (@munch ( $(,)? mathstyle $(:)?$(=>)? $literal:literal)
  -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
  defi_opts!(@munch ()
    -> {$kind, $( [ $key @ $val ] )*
      [ mathstyle @ Some($literal.to_string()) ] })
  };
  // misc ident with literal value
  (@munch ( $(,)? $id:ident $(:)?$(=>)? $lit:literal $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [$id @ $lit]})
  };
  // misc ident with block value
  (@munch ( $(,)? $id:ident $(:)?$(=>)? $body:block $($next:tt)*)
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [$id @ $body]})
  };

  //-- aux
  // Closure parsers

  (@before_digest_end ($body:block $($next:tt)* )
                  -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [before_digest_end @ before_digest!($body)]})
  };
  (@before_digest ($(sub)? $body:block $($next:tt)* )
                  -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [before_digest @ before_digest!($body)]})
  };
  (@before_digest ($(sub)? $body:block $($next:tt)* ) -> {$kind:ident,
    $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [before_digest @ before_digest!($body)]})
  };
  (@after_digest (
    sub[$whatsit:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_digest @ after_digest!($whatsit, $body)]})
  };
  (@after_digest (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_digest @ after_digest!(_whatsit,$body)]})
  };

  (@after_digest_begin (
    sub[$whatsit:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_digest_begin @ after_digest!($whatsit, $body)]})
  };
  (@after_digest_begin (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_digest_begin @ after_digest!(_whatsit, $body)]})
  };

  (@after_digest_body (
    sub[$whatsit:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_digest_body @ after_digest!($whatsit, $body)]})
  };
  (@after_digest_body (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_digest_body @ after_digest!( whatsit, $body)]})
  };


  (@before_construct (
    sub[$doc:ident, $whatsit:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [before_construct @ construct!($doc, $whatsit, $body)]})
  };
  (@before_construct (
    sub[$doc:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [before_construct @ construct!($doc, _whatsit, $body)]})
  };
  (@before_construct (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [before_construct @ construct!(_document, _whatsit, $body)]})
  };

  (@after_construct (
    sub[$doc:ident, $whatsit:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_construct @ construct!($doc, $whatsit, $body)]})
  };
  (@after_construct (
    sub[$doc:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_construct @ construct!($doc, _whatsit, $body)]})
  };
  (@after_construct (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_construct @ construct!(_document, _whatsit, $body)]})
  };

  (@getter (
    sub[$args:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [getter @ getter!($args, $body)]})
  };
  (@getter (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [getter @ getter!(_args,$body)]})
  };

  (@setter (
    sub[$value:ident, $scope:ident, $args:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [setter @ setter!($value, $scope, $args, $body)]})
  };
  (@setter (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [setter @ setter!(value, args, $body)]})
  };
  // 3-argument form: sub[document, node, whatsit] { ... }
  (@after_open (
    sub[$document:ident, $node:ident, $whatsit:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_open @ Some(tagsub!($document, $node, $whatsit, $body)) ]})
  };
  // 2-argument form: sub[document, node] { ... }
  (@after_open (
    sub[$document:ident, $node:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_open @ Some(tagsub!($document, $node, $body)) ]})
  };
  (@after_open (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_open @ Some(tagsub!(document, node, $body)) ]})
  };
  // 3-argument form
  (@after_open_late (
    sub[$document:ident, $node:ident, $whatsit:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_open_late @ Some(tagsub!($document, $node, $whatsit, $body)) ]})
  };
  // 2-argument form
  (@after_open_late (
    sub[$document:ident, $node:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_open_late @ Some(tagsub!($document, $node, $body)) ]})
  };
  (@after_open_late (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_open_late @ Some(tagsub!(document, node, $body)) ]})
  };
  // 3-argument form: sub[document, node, whatsit] { ... }
  // Matches Perl's afterClose => sub { my ($document, $node, $whatsit) = @_; ... }
  (@after_close (
    sub[$document:ident, $node:ident, $whatsit:ident] $body:block $($next:tt)* )
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_close @ Some(tagsub!($document, $node, $whatsit, $body)) ]})
  };
  // 2-argument form: sub[document, node] { ... }
  (@after_close (
    sub[$document:ident, $node:ident] $body:block $($next:tt)* )
    -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_close @ Some(tagsub!($document, $node, $body)) ]})
  };
  (@after_close (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_close @ Some(tagsub!(document, node, $body)) ]})
  };
  // 3-argument form
  (@after_close_late (
    sub[$document:ident, $node:ident, $whatsit:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_close_late @ Some(tagsub!($document, $node, $whatsit, $body)) ]})
  };
  // 2-argument form
  (@after_close_late (
    sub[$document:ident, $node:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_close_late @ Some(tagsub!($document, $node, $body)) ]})
  };
  (@after_close_late (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [after_close_late @ Some(tagsub!(document, node, $body)) ]})
  };

  (@replace ($body:block $($next:tt)* )
                  -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [replace @ rewrite_replace_sub!($body)]})
  };
  (@replace (sub [$document_arg:ident, $node_arg:ident]
    $body:block $($next:tt)* )
                  -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])* [replace @
      rewrite_replace_sub!($document_arg, $node_arg, $body)]})
  };
  (@reversion (sub [$whatsit:ident, $args:ident] $body:block $($next:tt)* )
                  -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [reversion @ reversion_digested!($whatsit, $args, $body)]})
  };
  (@reversion (sub [$args:ident, $inner:ident, $extra:ident] $body:block $($next:tt)*)
                  -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [reversion @ reversion!($args, $inner, $extra, $body)]})
  };
  (@reversion (sub $body:block $($next:tt)*)
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [reversion @ reversion!(_args, _inner, _extra, $body)]})
  };
  (@predigest (
    sub[$whatsit:ident, $extra:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [predigest @ predigest!($whatsit, $extra, $body)]})
  };
  (@predigest (
    sub[$whatsit:ident] $body:block $($next:tt)* )
      -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [predigest @ predigest!($whatsit, $body)]})
  };
  (@predigest (
    $body:block $($next:tt)* ) -> {$kind:ident, $([$key:ident @ $val:expr_2021])*}) => {
    defi_opts!(@munch ($($next)*) -> {$kind, $([$key @ $val])*
      [predigest @ predigest!( _whatsit,$body)]})
  };
}
