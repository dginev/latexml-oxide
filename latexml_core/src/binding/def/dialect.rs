use once_cell::sync::Lazy;
use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;
use std::rc::Rc;

use regex::Regex;

// use crate::common::error::*;
use crate::binding::content::merge_font;
use crate::binding::counter::dialect::step_counter;
use crate::binding::def::traits::{IntoDigestedResult, IntoOption};
use crate::common::arena;
use crate::common::arena::SymHashMap;
use crate::common::error::*;
use crate::common::font::Font;
use crate::common::number::Number;
use crate::common::numeric_ops::NumericOps;
use crate::definition::conditional::{Conditional, ConditionalOptions, ConditionalType};
use crate::definition::constructor::{Constructor, ConstructorOptions};
use crate::definition::expandable::{Expandable, ExpandableOptions};
use crate::definition::math_primitive::{MathPrimitive, MathPrimitiveOptions};
use crate::definition::primitive::{Primitive, PrimitiveOptions};
use crate::definition::register::{
  Register, RegisterGetterClosure, RegisterSetterClosure, RegisterType, RegisterValue,
};
use crate::definition::{
  BeforeDigestClosure, ConditionalClosure, ConstructionClosure, Definition, DigestionClosure,
  ExpansionBody, FontDirective, PrimitiveBody, ReplacementClosure, Reversion, SizingClosure,
};
use crate::document::Document;
use crate::gullet;
use crate::mouth;
use crate::parameter::Parameters;
use crate::state::*;
use crate::stomach::*;
use crate::tbox::Tbox;
use crate::token::*;
use crate::tokens::Tokens;
use crate::whatsit::Whatsit;
use crate::Digested;

const MATH_CONSTRUCTOR_ATTRIBUTES: &[&str] = &[
  "name",
  "meaning",
  "omcd",
  "decl_id",
  "mathstyle",
  "lpadding",
  "rpadding",
];

/// regex for the prefix of a conditional command sequence
pub static CONDITIONAL_CS_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^\\(?:if(.*)|unless)$").unwrap());
/// regex for the prefix of a protocol (such as literal:)
pub static LEADING_PROTOCOL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\w+:").unwrap());
/// regex for a trailing slash (trivial, but aids replacement of said slash)
pub static TRAILING_SLASH_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"/$").unwrap());
/// regex for one-or-more spaces
pub static SPACES_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());
/// regex for ${}^{label}$
pub static DIRTY_ID_IDIOM_RE: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"\$\{\}\^\{(?P<label>[^\}]*)\}\$").unwrap());
/// regex for characters not expected in a usual id attribute
pub static NON_ID_CHARSET_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[^\w_\-.]+").unwrap());
/// regex for a strange noisy TeX `\\~{}`
pub static TILDE_NOISE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\~\{\}").unwrap());
/// regex for a TeX argument specifier or any command sequence
pub static HAS_ARG_OR_CS: Lazy<Regex> = Lazy::new(|| Regex::new(r"#\d|\\.").unwrap());
/// regex for the usual argument placeholders `#1`-`#9`
pub static ARG_HOLE: Lazy<Regex> = Lazy::new(|| Regex::new(r"#(\d)").unwrap());

/// Is defined in the `LaTeX`-y sense of also not being let to \relax.
pub fn is_defined(name: &str) -> bool {
  let cs = T_CS!(name);
  is_defined_token(&cs)
}

/// Token variant of `is_defined`. Defined in the LaTeX-y sense of also not being let to \relax.
pub fn is_defined_token(cs: &Token) -> bool {
  match lookup_meaning(cs) {
    Some(store) => match store {
      Stored::Token(_) => true,
      Stored::Expandable(ref m) => m.get_cs_name() != "\\relax",
      Stored::Primitive(ref m) => m.get_cs_name() != "\\relax",
      Stored::Constructor(ref m) => m.get_cs_name() != "\\relax",
      Stored::Register(ref m) => m.get_cs_name() != "\\relax",
      other => panic!("TODO: unexpected case for is_defined_token, got: {other:?}"),
    },
    _ => false,
  }
}

/// Check if the `token` is not yet defined, or let to `\relax`
pub fn is_definable(token: &Token) -> bool {
  let meaning = lookup_meaning(token);
  token.with_str(|name| name != "\\relax" && !name.starts_with("\\end"))
    && (meaning.is_none()
      || (meaning == lookup_meaning(&TOKEN_RELAX))
      || lookup_bool("2.09_COMPATIBILITY"))
}

/// unconditionally wraps a CS token around a string
// TODO: this was more useful in Perl, maybe we should remove?
pub fn coerce_cs(t: &str) -> Token { T_CS!(t) }

//======================================================================
// Defining Conditional Control Sequences.
//======================================================================
/// Define a conditional control sequence.
///
/// Its processing takes place in the Gullet.
/// The test is applied to the arguments (if any),
/// which determines which branch is executed.
/// If the test is undefined, the conditional is a "user defined" one;
/// Two additional primitives are defined \footrue and \foofalse;
/// the test is then determined by the most recently called of those.
///
/// If you supply a skipper instead of a test, it is also applied to the arguments
/// and should skip to the right place in the following \or, \else, \fi.
pub fn def_conditional(
  cs: Token,
  paramlist: Option<Parameters>,
  test: Option<ConditionalClosure>,
  options: ConditionalOptions,
) -> Result<()> {
  let locked_key_opt = if let Some(true) = options.locked {
    Some(arena::with(cs.get_sym(), |cs_name| s!("{cs_name}:locked")))
  } else {
    None
  };
  if cs.with_str(|cs_name| matches!(cs_name, "\\fi" | "\\else" | "\\or" | "\\unless")) {
    install_definition(
      Conditional {
        cs,
        paramlist,
        test,
        conditional_type: cs.with_str(|cs_name| ConditionalType::from(cs_name)),
        skipper: options.skipper,
      },
      options.scope,
    )
  } else {
    let name_opt = cs.with_str(|custom| {
      CONDITIONAL_CS_RE
        .captures(custom)
        .map(|captures| captures.get(1).map_or("", |m| m.as_str()).to_string())
    });
    if let Some(name) = name_opt {
      if !name.is_empty() && name != "case" && test.is_none() {
        // user-defined conditional, like with \newif
        // Note: setting up these macros is compile-time expensive, maybe there is some way to
        // avoid...
        // Note: the double clones are technically correct Rust if annoying to write and read.
        //       first, we want to capture a cloned value of cs, to be able to keep using cs here.
        // second, each invocation of the conditional macro needs to create new tokens to
        // return,       hence a clone is required on each call.
        def_macro(
          T_CS!(s!("\\{}true", name)),
          None,
          Tokens!(T_CS!("\\let"), cs, T_CS!("\\iftrue")),
          None,
        )?;
        def_macro(
          T_CS!(s!("\\{}false", name)),
          None,
          Tokens!(T_CS!("\\let"), cs, T_CS!("\\iffalse")),
          None,
        )?;
        let_i(&cs, &T_CS!("\\iffalse"), None);
      } else {
        //  For \ifcase, the parameter list better be a single Number !!
        install_definition(
          Conditional {
            cs,
            paramlist,
            test,
            conditional_type: ConditionalType::If,
            skipper: options.skipper,
          },
          options.scope,
        );
      }
    } else {
      let message = s!(
        "The conditional {} is being defined but doesn't start with \\if",
        cs
      );
      Error!("misdefined", cs, message);
    }
  }

  if let Some(locked_key) = locked_key_opt {
    assign_value(&locked_key, true, Some(Scope::Global));
  }
  Ok(())
}

/// Defines the macro expansion for a command sequence.
///
/// A macro control sequence that reads parameters
/// as specified by `paramlist` and is expanded during macro expansion time in the `Gullet`.
/// See `ExpansionBody` for the possible kinds of `expansion` material.
pub fn def_macro<T: Into<Option<ExpansionBody>>>(
  cs: Token,
  paramlist: Option<Parameters>,
  expansion: T,
  options_opt: Option<ExpandableOptions>,
) -> Result<()> {
  let expansion_opt: Option<ExpansionBody> = expansion.into();
  // TODO: The None case could be refactored to feel much cleaner.
  // For now it's equivalent to Tokens!()
  let mut options = options_opt.unwrap_or_default();
  let scope = options.scope.take();
  if options.mathactive && cs.with_str(|s| s.len()) == 1 {
    assign_mathcode(
      cs.with_str(|cstr| cstr.chars().next().unwrap()),
      0x8000u16,
      scope,
    );
  }
  let locked_key_opt = if options.locked {
    Some(format!("{cs}:locked"))
  } else {
    None
  };
  let defcs = if options.robust {
    def_robust_cs(cs, options.locked, options.scope)?
  } else {
    cs
  };
  install_definition(
    Expandable::new(defcs, paramlist, expansion_opt, Some(options))?,
    scope,
  );
  if let Some(locked_key) = locked_key_opt {
    assign_value(&locked_key, true, Some(Scope::Global));
  }
  Ok(())
}

/// configuration for creating a new Register
#[derive(Default)]
pub struct RegisterOptions {
  /// closure to obtain the current register value
  pub getter:   Option<RegisterGetterClosure>,
  /// closure to set the current register value
  pub setter:   Option<RegisterSetterClosure>,
  /// is this register meant as read-only? (default: false)
  pub readonly: bool,
  /// an optional name for the register (default: the cs)
  pub address:  Option<String>,
  /// an optional allocation for the register (default: None)
  pub allocate: Option<String>,
}

/// Defines a register with an initial value.
///
/// (a Number, Dimension, Glue, MuGlue or Tokens --- I haven't handled Box's yet).
/// Usually, the `prototype` is just the control sequence,
/// but registers are also handled by prototypes like `\count{Number}`. `DefRegister` arranges
/// that the register value can be accessed when a numeric, dimension, ... value is being read,
/// and also defines the control sequence for assignment.
pub fn def_register<T: Into<RegisterValue>>(
  cs: Token,
  parameters: Option<Parameters>,
  value: T,
  options: Option<RegisterOptions>,
) -> Result<()> {
  let mut options: RegisterOptions = options.unwrap_or_default();
  let value: RegisterValue = value.into();
  let has_address_option = options.address.is_some();
  let mut address = match options.address.take() {
    Some(v) => v,
    None => match options.allocate {
      Some(v) => allocate_register(&v)?.unwrap_or_default(),
      None => String::new(),
    },
  };
  // by adding this check here, we no longer need to use Register::new in the Rust version
  if address.is_empty() {
    address = cs.to_string();
  }
  // Assign, but do not RE-assign
  if !has_address_option || !has_value(&address) {
    assign_value(&address, value.clone(), Some(Scope::Global));
  }

  let register_type: RegisterType = (&value).into();
  install_definition(
    Register {
      cs,
      address,
      parameters,
      register_type,
      readonly: options.readonly,
      getter: options.getter,
      setter: options.setter,
      default: Some(value),
      value: None,
      role: None,
      locator: gullet::get_locator(),
      mathglyph: None,
    },
    Some(Scope::Global),
  );
  Ok(())
}

/// Defines a primitive control sequence
///
/// A primitive is processed during
/// digestion (in the  `Stomach`), after macro expansion but before Construction time.
/// Primitive control sequences generate Boxes or Lists, generally
/// containing basic Unicode content, rather than structured XML.
/// Primitive control sequences are also executed for side effect during digestion,
/// effecting changes to the `State`.
pub fn def_primitive(
  cs: Token,
  paramlist: Option<Parameters>,
  compiled_replacement: Option<PrimitiveBody>,
  options: PrimitiveOptions,
) -> Result<()> {
  let options_locked = options.locked;
  let scope = options.scope;
  let mut before_digest_env: Vec<BeforeDigestClosure> = Vec::new();
  let cs_name = cs.with_cs_name(ToString::to_string);

  if options.require_math {
    let cs_name_cloned = cs_name.clone();
    let require_math_closure = before_digest_simple!({ requireMath!(cs_name_cloned) });
    before_digest_env.push(require_math_closure);
  }

  if options.forbid_math {
    let cs_name_cloned = cs_name.clone();
    let forbid_math_closure = before_digest_simple!({ forbidMath!(cs_name_cloned) });
    before_digest_env.push(forbid_math_closure);
  }
  if let Some(ref mode) = options.mode {
    let mode_clone = mode.clone();
    let begin_mode_closure = before_digest_simple!({
      begin_mode(&mode_clone)?;
    });
    before_digest_env.push(begin_mode_closure);
  } else if options.bounded {
    let bgroup_closure = before_digest_simple!({
      bgroup();
    });
    before_digest_env.push(bgroup_closure);
  }
  if let Some(chosen_font_directive) = options.font {
    let merge_font_closure = before_digest_simple!({
      if let FontDirective::Asset(ref chosen_font) = chosen_font_directive {
        merge_font((**chosen_font).clone());
      }
    });
    before_digest_env.push(merge_font_closure);
  }
  before_digest_env.extend(options.before_digest);

  let mut after_digest_env: Vec<DigestionClosure> = options.after_digest;
  if let Some(ref mode) = options.mode {
    let mode_clone = mode.clone();
    let end_mode_closure: DigestionClosure = after_digest_simple!(_whatsit, {
      end_mode(&mode_clone)?;
    });
    after_digest_env.push(end_mode_closure);
  } else if options.bounded {
    let egroup_closure: DigestionClosure = after_digest_simple!(_whatsit, {
      egroup()?;
    });
    after_digest_env.push(egroup_closure);
  }
  //  Not sure robust entirely makes sense for Primitives, other than LaTeXML vs LaTeX mismatch
  let defcs = if options.robust {
    def_robust_cs(cs, options.locked, scope)?
  } else {
    cs
  };

  install_definition(
    Primitive {
      cs: defcs,
      paramlist,
      replacement: compiled_replacement,
      before_digest: before_digest_env,
      after_digest: after_digest_env,
      alias: options.alias,
      nargs: options.nargs,
      is_prefix: options.is_prefix,
      reversion: options.reversion,
    },
    scope,
  );
  if options_locked {
    assign_value(&s!("{}:locked", cs_name), true, Some(Scope::Global));
  }
  Ok(())
}

/// Advanced math replacements require a XMDual representation
pub fn def_math_dual(
  cs: Token,
  paramlist: Option<Parameters>,
  presentation: String,
  options: MathPrimitiveOptions,
) -> Result<()> {
  let (cont_cs_str, pres_cs_str) =
    cs.with_str(|csname| (s!("{csname}@content"), s!("{csname}@presentation")));
  let cont_cs = T_CS!(cont_cs_str);
  let pres_cs = T_CS!(pres_cs_str);
  let defcs = if options.robust {
    def_robust_cs(cs, options.locked, options.scope)?
  } else {
    cs
  };
  let presentation_toks = mouth::tokenize_internal(&presentation);

  // Make the original CS expand into a DUAL invoking a presentation macro and content constructor
  let captured_role = options.role.clone();
  let captured_revert_as = options.revert_as.clone();
  let captured_cont_cs = cont_cs;
  let captured_pres_cs = pres_cs;
  let captured_pres = presentation.clone();
  install_definition(
    Expandable::new(
      defcs,
      paramlist.clone(),
      Some(ExpansionBody::Closure(Rc::new(move |args| {
        let args_opt_tks = args
          .into_iter()
          .map(|arg| arg.into())
          .collect::<Vec<Option<Tokens>>>();
        let (cargs, pargs) = dualize_arglist(&captured_pres, args_opt_tks)?;

        let mut dtks = vec![T_CS!("\\lx@dual")];
        // optional keyval arg
        if captured_role.is_some() || captured_revert_as.is_some() {
          dtks.push(T_OTHER!("["));
          if let Some(ref role) = captured_role {
            dtks.extend(vec![T_OTHER!("role"), T_OTHER!("="), T_OTHER!(role)]);
            if let Some(ref _revert_as) = captured_revert_as {
              dtks.push(T_OTHER!(","));
            }
          }
          if let Some(ref revert_as) = captured_revert_as {
            dtks.extend(vec![
              T_OTHER!("revert_as"),
              T_OTHER!("="),
              T_OTHER!(revert_as),
            ]);
          }
          dtks.push(T_OTHER!("]"));
        }
        // end optional keyval arg
        dtks.push(T_BEGIN!());
        dtks.push(captured_cont_cs);
        dtks.push(T_BEGIN!());
        for carg in cargs.into_iter().flatten() {
          // if let Some(carg) = carg_opt {
          dtks.extend(carg.unlist());
          //} else {}
          // TODO: we can't push an empty tokens in the flat setup. Is this a problem?
        }
        dtks.push(T_END!());
        dtks.push(T_END!());
        dtks.push(T_BEGIN!());
        dtks.push(captured_pres_cs);
        dtks.push(T_BEGIN!());
        for parg in pargs.into_iter().flatten() {
          // if let Some(parg) = parg_opt {
          dtks.extend(parg.unlist());
          //} else {} // TODO: we can't push an empty tokens in the flat setup. Is this a problem?
        }
        dtks.push(T_END!());
        dtks.push(T_END!());

        Ok(Tokens::new(dtks))
      }))),
      Some(ExpandableOptions {
        protected: options.protected,
        ..ExpandableOptions::default()
      }),
    )?,
    options.scope,
  );

  // Make the presentation macro.
  install_definition(
    Expandable::new(
      pres_cs,
      paramlist.clone(),
      Some(ExpansionBody::Tokens(presentation_toks)),
      Some(ExpandableOptions {
        protected: options.protected,
        ..ExpandableOptions::default()
      }),
    )?,
    options.scope,
  );

  // content: Make the content constructor
  // content: build the replacement closure
  let nargs = paramlist
    .as_ref()
    .map(|pl| pl.get_parameters().len())
    .unwrap_or(0);
  let content_closure: ReplacementClosure = if nargs == 0 {
    Rc::new(|document, _args, props| {
      let mut attrs = HashMap::default();
      for key in ["role", "scriptpos", "stretchy"] {
        if let Some(v) = props.get(key) {
          attrs.insert(key.to_owned(), v.to_string());
        }
      }
      for key in MATH_CONSTRUCTOR_ATTRIBUTES {
        if let Some(v) = props.get(key) {
          attrs.insert(key.to_string(), v.to_string());
        }
      }
      document.insert_element("ltx:XMTok", Vec::new(), Some(attrs))?;
      Ok(())
    })
  } else {
    Rc::new(|document, args, props| {
      let mut app_attrs = HashMap::default();
      for key in ["role", "scriptpos"] {
        if let Some(v) = props.get(key) {
          app_attrs.insert(key.to_owned(), v.to_string());
        }
      }
      document.open_element("ltx:XMApp", Some(app_attrs), None)?;
      let mut op_attrs = HashMap::default();
      if let Some(v) = props.get("operator_stretchy") {
        op_attrs.insert("stretchy".to_owned(), v.to_string());
      }
      if let Some(v) = props.get("operator_role") {
        op_attrs.insert("role".to_owned(), v.to_string());
      }
      if let Some(v) = props.get("operator_scriptpos") {
        op_attrs.insert("scriptpos".to_owned(), v.to_string());
      }
      for key in MATH_CONSTRUCTOR_ATTRIBUTES {
        if let Some(v) = props.get(key) {
          op_attrs.insert(key.to_string(), v.to_string());
        }
      }
      // operator
      document.insert_element("ltx:XMTok", Vec::new(), Some(op_attrs))?;
      // arguments
      // TODO: options.reorder?
      for arg in args.iter().flatten() {
        document.absorb(arg, None)?;
      }
      document.close_element("ltx:XMApp")?;
      Ok(())
    })
  };
  // content: install the constructor
  let mut content_constructor = Constructor {
    cs: cont_cs,
    paramlist,
    replacement: Some(content_closure),
    ..Constructor::default()
  };
  let scope = options.scope;
  transfer_common_constructor_options(&cs, &presentation, options, &mut content_constructor);
  install_definition(content_constructor, scope);
  Ok(())
}

/// EXPERIMENT: Introduce an intermediate case for simple symbols
/// Define a primitive that will create a Tbox with the appropriate set of XMTok attributes.
pub fn def_math_primitive(
  cs: Token,
  _paramlist: Option<Parameters>,
  presentation: String,
  options: MathPrimitiveOptions,
) {
  let scope = options.scope;
  let reqfont_opt = options.font.clone();
  let moved_options = options.clone();

  install_definition(
    MathPrimitive {
      cs,
      paramlist: None, // never any parameters, this is intentional
      replacement: Some(Rc::new(move |_args| {
        let locator = gullet::get_locator();
        let mut properties = moved_options.clone();
        properties.mode = Some(String::from("math"));
        let state_font = lookup_font().unwrap();
        let font = Rc::new(if let Some(ref reqfont) = reqfont_opt {
          let this_reqfont = reqfont.get_font(None)?;
          state_font
            .merge((*this_reqfont).clone())
            .specialize(&presentation)
        } else {
          state_font.specialize(&presentation)
        });

        Ok(vec![Digested::from(Tbox {
          text: arena::pin(&presentation),
          tokens: Tokens!(cs),
          font,
          properties: properties.to_hash_stored(),
          locator,
        })])
      })),
      options,
      ..MathPrimitive::default()
    },
    scope,
  );
}

/// Uses of DefMath without arguments, but with constructor-like options, are realized via a
/// `Constructor` definition
pub fn def_math_constructor(
  cs: Token,
  paramlist: Option<Parameters>,
  presentation: String,
  mut options: MathPrimitiveOptions,
) -> Result<()> {
  // TODO: do we need to do anything about digesting the presentation?
  let nargs = paramlist
    .as_ref()
    .map(|pl| pl.get_parameters().len())
    .unwrap_or(0);
  // let csname_alias = if options.alias.is_none() && options.robust {
  //   Some(String::from(cs.get_cs_name()))
  // } else {
  //   None
  // };
  let defcs = if options.robust {
    def_robust_cs(cs, options.locked, options.scope)?
  } else {
    cs
  };
  if options.reversion.is_none() && nargs == 0 && options.alias.is_none() {
    if options.revert_as.is_none()
      || options.revert_as == Some(Cow::Borrowed("content"))
      || options.revert_as == Some(Cow::Borrowed("context"))
    {
      // TODO :&& (($LaTeXML::DUAL_BRANCH || 'content') eq 'content'))
      options.reversion = Some(Reversion::Tokens(Tokens!(cs)));
    } else {
      // TODO: This differs from the Perl, where `presentation` comes in as Tokens
      //       we have it come in as a `String`,
      //       so need to tokenize when reusing it as a reversion.
      options.reversion = Some(Reversion::Tokens(Tokens::new(Explode!(presentation))));
    }
  }
  let presentation_for_sizer = presentation.clone();
  let presentation_for_replacement = presentation.clone();
  let is_mathstyle = options.mathstyle.is_some();
  let mathstyle_for_font = options.mathstyle.clone();
  let presentation_for_font = presentation.clone();
  options.font = Some(FontDirective::Closure(if is_mathstyle {
    Rc::new(move |_whatsit| {
      Ok(
        lookup_font()
          .unwrap()
          .merge(Font {
            mathstyle: mathstyle_for_font
              .as_ref()
              .map(|ms| Cow::Owned(ms.to_owned())),
            ..Font::default()
          })
          .specialize(&presentation_for_font),
      )
    })
  } else {
    Rc::new(move |_whatsit| Ok(lookup_font().unwrap().specialize(&presentation_for_font)))
  }));
  let compiled_replacement: Option<ReplacementClosure> = Some(if nargs == 0 {
    // If trivial presentation, allow it in Text
    Rc::new(
      move |document: &mut Document, _, props: &SymHashMap<Stored>| {
        let mut attrs = HashMap::default();
        for key in ["role", "scriptpos", "stretchy"] {
          if let Some(v) = props.get(key) {
            attrs.insert(key.to_owned(), v.to_string());
          }
        }
        for key in MATH_CONSTRUCTOR_ATTRIBUTES {
          if let Some(v) = props.get(key) {
            attrs.insert(key.to_string(), v.to_string());
          }
        }
        let font_opt = match props.get("font") {
          Some(Stored::Font(f)) => Some(Cow::Borrowed(&**f)),
          Some(Stored::FontDirective(FontDirective::Closure(code))) => {
            Some(Cow::Owned(code(None)?))
          },
          Some(Stored::FontDirective(FontDirective::Asset(font))) => Some(Cow::Borrowed(&**font)),
          _ => None,
        };
        if let Some(font) = font_opt {
          document.open_element("ltx:XMTok", Some(attrs), Some(&font))?;
        } else {
          document.open_element("ltx:XMTok", Some(attrs), None)?;
        }
        document.absorb_string(&presentation_for_replacement, props)?;
        document.close_element("ltx:XMTok")?;

        Ok(())
      },
    )
  } else {
    Rc::new(
      move |document: &mut Document, args: &Vec<Option<Digested>>, props: &SymHashMap<Stored>| {
        let mut attrs = HashMap::default();
        for key in ["role", "scriptpos", "stretchy"] {
          if let Some(v) = props.get(key) {
            attrs.insert(key.to_owned(), v.to_string());
          }
        }
        let font_opt = match props.get("font") {
          Some(Stored::Font(f)) => Some(Cow::Borrowed(&**f)),
          Some(Stored::FontDirective(FontDirective::Closure(code))) => {
            Some(Cow::Owned(code(None)?))
          },
          Some(Stored::FontDirective(FontDirective::Asset(font))) => Some(Cow::Borrowed(&**font)),
          _ => None,
        };
        if let Some(ref font) = font_opt {
          document.open_element("ltx:XMApp", Some(attrs), Some(font))?;
        } else {
          document.open_element("ltx:XMApp", Some(attrs), None)?;
        }
        // operator
        let mut op_attrs = HashMap::default();
        if let Some(role) = props.get("operator_role") {
          op_attrs.insert(String::from("role"), role.to_string());
        }
        if let Some(stretchy) = props.get("operator_stretchy") {
          op_attrs.insert(String::from("stretchy"), stretchy.to_string());
        }
        if let Some(scriptpos) = props.get("operator_scriptpos") {
          op_attrs.insert(String::from("scriptpos"), scriptpos.to_string());
        }
        for key in MATH_CONSTRUCTOR_ATTRIBUTES {
          if let Some(v) = props.get(key) {
            op_attrs.insert(key.to_string(), v.to_string());
          }
        }
        if let Some(font) = font_opt {
          document.open_element("ltx:XMTok", Some(op_attrs), Some(&font))?;
        } else {
          document.open_element("ltx:XMTok", Some(op_attrs), None)?;
        }
        document.absorb_string(&presentation_for_replacement, props)?;
        document.close_element("ltx:XMTok")?;
        // arguments
        for arg in args {
          document.open_element("ltx:XMArg", None, None)?;
          if let Some(arg_v) = arg {
            document.absorb(arg_v, None)?;
          }
          document.close_element("ltx:XMArg")?;
        }

        document.close_element("ltx:XMApp")?;
        Ok(())
      },
    )
  });
  let sizer: Option<SizingClosure> = Some(Rc::new(move |_| {
    Ok(Font::math_default().compute_string_size(&presentation_for_sizer, SymHashMap::default()))
  }));

  // let mut prop_options = options.clone();
  let mut constructor = Constructor {
    cs: defcs,
    paramlist,
    replacement: compiled_replacement,
    nargs: Some(nargs),
    sizer,
    // capture_body: options.capture_body,
    // outer
    // long
    ..Constructor::default()
  };
  let scope = options.scope;
  transfer_common_constructor_options(&cs, &presentation, options, &mut constructor);
  install_definition(constructor, scope);
  Ok(())
}

fn infer_sizer(
  sizer: Option<&SizingClosure>,
  reversion: Option<&Reversion>,
) -> Option<SizingClosure> {
  match sizer {
    Some(closure) => Some(Rc::clone(closure)),
    None => match reversion {
      Some(Reversion::Tokens(tks)) => (*tks).to_string().as_str().into_option(),
      _ => None,
    },
  }
}

fn def_robust_cs(cs: Token, locked: bool, scope: Option<Scope>) -> Result<Token> {
  let cs_str = cs.with_str(|cstr| format!("{cstr} "));
  let defcs = T_CS!(cs_str);
  let return_cs = defcs;
  let expansion = Tokens!(T_CS!("\\protect"), defcs);
  let options = ExpandableOptions {
    locked,
    robust: true,
    ..ExpandableOptions::default()
  };
  // scope should be \x@protect?
  install_definition(
    Expandable::new(cs, None, expansion.into(), Some(options))?,
    scope,
  );
  Ok(return_cs)
}

/// Binding definition connecting a TeX command sequence with a structured XML output.
///
/// The Constructor is where LaTeXML really starts getting interesting;
/// invoking the control sequence will generate an arbitrary XML
/// fragment in the document tree.  More specifically: during digestion, the arguments
/// will be read and digested, creating a `Whatsit` to represent the object. During
/// absorption by the `Document`, the `Whatsit` will generate the XML fragment according
/// to the `compiled_replacement`.
pub fn def_constructor(
  cs: Token,
  paramlist: Option<Parameters>,
  compiled_replacement: Option<ReplacementClosure>,
  options: ConstructorOptions,
) {
  // TODO: This won't work, as we can only invoke method calls on paramlist in runtime
  //*latexml_codegen::constructable::NARGS = $paramlist.get_num_args();
  let scope = options.scope;
  let cs_name = cs.with_cs_name(ToString::to_string);
  let locked_key_opt = if options.locked {
    Some(s!("{cs_name}:locked"))
  } else {
    None
  };

  let mut before_digest_closures: Vec<BeforeDigestClosure> = Vec::new();

  if options.require_math {
    let cs_name_cloned = cs_name.clone();
    let require_math_closure = before_digest_simple!({ requireMath!(cs_name_cloned) });
    before_digest_closures.push(require_math_closure);
  }
  if options.forbid_math {
    let cs_name_cloned = cs_name;
    let forbid_math_closure = before_digest_simple!({ forbidMath!(cs_name_cloned) });
    before_digest_closures.push(forbid_math_closure);
  }
  if let Some(ref mode) = options.mode {
    let mode_clone = mode.clone();
    let begin_mode_closure = before_digest_simple!({
      begin_mode(&mode_clone)?;
    });
    before_digest_closures.push(begin_mode_closure);
  } else if options.bounded {
    let bgroup_closure = before_digest_simple!({
      bgroup();
    });
    before_digest_closures.push(bgroup_closure);
  }
  // DG: The situations with Fonts in Constructors appears rather complex?
  //  LaTeXML seems to currently rely on both the top-level "font" option but *also*
  //  has code checking for a second-tier "properties => { font => VALUE}" option
  //  Can we consolidate into a single, top-level, font handler?
  match options.font {
    Some(FontDirective::Asset(chosen_font)) => {
      let merge_font_closure = before_digest_simple!({
        merge_font((*chosen_font).clone());
      });
      before_digest_closures.push(merge_font_closure);
    },
    Some(FontDirective::Closure(font_closure)) => {
      let execute_font_closure = before_digest_simple!({
        merge_font(font_closure(None)?);
      });
      before_digest_closures.push(execute_font_closure);
    },
    None => {},
  };
  before_digest_closures.extend(options.before_digest);

  let mut after_digest_closures: Vec<DigestionClosure> = options.after_digest;
  if let Some(ref mode) = options.mode {
    let mode_clone = mode.clone();
    let end_mode_closure: DigestionClosure = after_digest_simple!(_whatsit, {
      end_mode(&mode_clone)?;
    });
    after_digest_closures.push(end_mode_closure);
  } else if options.bounded {
    let egroup_closure: DigestionClosure = after_digest_simple!(_whatsit, {
      egroup()?;
    });
    after_digest_closures.push(egroup_closure);
  }

  let constructor = Constructor {
    cs,
    paramlist,
    replacement: compiled_replacement,
    before_digest: before_digest_closures,
    after_digest: after_digest_closures,
    before_construct: options.before_construct,
    after_construct: options.after_construct,
    nargs: options.nargs,
    alias: options.alias,
    sizer: infer_sizer(options.sizer.as_ref(), options.reversion.as_ref()),
    reversion: options.reversion,
    capture_body: options.capture_body,
    properties: options.properties,
    // outer
    // long
    ..Constructor::default()
  };
  install_definition(constructor, scope);

  if let Some(locked_key) = locked_key_opt {
    assign_value(&locked_key, true, Some(Scope::Global));
  }
}

/// Defines an Environment that generates a specific XML fragment.
///
/// `compiled_replacement` is of the same form as for DefConstructor, but will generally include
/// reference to the `#body` property.
/// Upon encountering a `\begin{env}`:  the mode is switched, if needed, else a new group is opened;
/// then the environment name is noted; the beforeDigest hook is run.
/// Then the Whatsit representing the begin command (but ultimately the whole environment) is
/// created and the `after_digest_begin` hook is run.
/// Next, the body will be digested and collected until the balancing `\end{env}`.
/// Then, any `after_digest` hook is run, the environment is ended, finally the mode is ended or the
/// group is closed.  The body and `\end{env}` whatsit are added to the `\begin{env}`'s whatsit as
/// body and trailer, respectively.
pub fn def_environment(
  name: String,
  paramlist: Option<Parameters>,
  compiled_replacement: Option<ReplacementClosure>,
  options: ConstructorOptions,
) {
  // This is for the common case where the environment is opened by \begin{env}
  let begin_name = s!("\\begin{{{name}}}");
  let end_name = s!("\\end{{{name}}}");
  let mut before_digest_env: Vec<BeforeDigestClosure> = Vec::new();
  if options.require_math {
    let require_name = begin_name.clone();
    let require_math_closure = before_digest_simple!({ requireMath!(require_name) });
    before_digest_env.push(require_math_closure);
  }
  if options.forbid_math {
    let forbid_name = begin_name.clone();
    let forbid_math_closure = before_digest_simple!({ forbidMath!(forbid_name) });
    before_digest_env.push(forbid_math_closure);
  }
  let bgroup_closure = before_digest_simple!({
    bgroup();
  });
  before_digest_env.push(bgroup_closure);
  let atbegin_key = s!("@environment@{name}@atbegin");
  let atbegin_hook_closure = before_digest_simple!({
    if let Some(b) = lookup_tokens(&atbegin_key) {
      vec![digest(b.unlist())?]
    } else {
      Vec::new()
    }
  });

  before_digest_env.push(atbegin_hook_closure);
  if let Some(ref mode) = options.mode {
    let bmode = mode.clone();
    let mode_closure = before_digest_simple!({
      set_mode(&bmode)?;
    });
    before_digest_env.push(mode_closure);
  }

  let env_name = name.clone();
  let current_environment_closure = before_digest_simple!({
    assign_value("current_environment", env_name.clone(), None);
    let body = T_LETTER!(env_name.clone());
    def_macro(
      T_CS!("\\@currenvir"),
      None,
      Some(ExpansionBody::Tokens(Tokens!(body))),
      None,
    )?;
  });
  before_digest_env.push(current_environment_closure);

  if let Some(chosen_font_directive) = options.font {
    let merge_font_closure = before_digest_simple!({
      if let FontDirective::Asset(ref chosen_font) = chosen_font_directive {
        merge_font((**chosen_font).clone());
      }
    });
    before_digest_env.push(merge_font_closure);
  }
  before_digest_env.extend(options.before_digest);

  let push_frame_closure = Rc::new(|_document: &mut Document, _whatsit: &Whatsit| {
    push_frame();
    Ok(())
  });
  let mut before_construct_with_frame: Vec<ConstructionClosure> = vec![push_frame_closure];
  before_construct_with_frame.extend(options.before_construct);

  let mut after_construct_with_frame: Vec<ConstructionClosure> = options.after_construct;

  let pop_frame_closure = Rc::new(|_document: &mut Document, _whatsit: &Whatsit| {
    pop_frame()?;
    Ok(())
  });
  after_construct_with_frame.push(pop_frame_closure);

  let begin_name_constructor = Rc::new(Constructor {
    cs:                T_CS!(begin_name),
    paramlist:         paramlist.clone(),
    replacement:       compiled_replacement.clone(),
    nargs:             options.nargs,
    before_digest:     before_digest_env,
    after_digest:      options.after_digest_begin,
    after_digest_body: options.after_digest_body,
    before_construct:  before_construct_with_frame,
    // Curiously, it's the \begin whose afterConstruct gets called.
    after_construct:   after_construct_with_frame,
    capture_body:      true,
    properties:        options.properties.clone(),
    // (defined $options{reversion} ? (reversion => $options{reversion}) : ()),
    // (defined $sizer ? (sizer => $sizer) : ()),
    // ), $options{scope});
    sizer:             infer_sizer(options.sizer.as_ref(), options.reversion.as_ref()),
    reversion:         options.reversion,
    alias:             options.alias,
  });
  install_definition(begin_name_constructor, options.scope);

  let mut after_digest_env = options.after_digest.clone();
  let name_clone = name.to_string();
  let end_name_clone = end_name.to_string();
  let unexpected_end_closure = after_digest_simple!(_whatsit, {
    let env = lookup_string("current_environment");
    if env.is_empty() || name_clone != env {
      let message1 = s!("Can't close environment {}", name_clone);
      let message2 = s!(
        "Current are {} ",
        with_stacked_values("current_environment", |vals| vals
          .iter()
          .map(|x| s!("{:?}", x))
          .collect::<Vec<String>>()
          .join(", "))
      );
      Error!("unexpected", end_name_clone, message1, message2);
    }
    Ok(Vec::new())
  });
  after_digest_env.push(unexpected_end_closure);

  match options.mode {
    Some(mode) => {
      let emode = mode;
      let emode_closure = Rc::new(move |_whatsit: &mut Whatsit| {
        end_mode(&emode)?;
        Ok(Vec::new())
      });
      after_digest_env.push(emode_closure);
    },
    None => {
      let egroup_closure = Rc::new(|_whatsit: &mut Whatsit| {
        egroup()?;
        Ok(Vec::new())
      });
      after_digest_env.push(egroup_closure);
    },
  };

  let mut before_digest_for_endenv = options.before_digest_end;
  let atend_key = s!("@environment@{name}@atend");
  let atend_hook_closure = before_digest_simple!({
    if let Some(e) = lookup_tokens(&atend_key) {
      vec![digest(e.unlist())?]
    } else {
      Vec::new()
    }
  });
  before_digest_for_endenv.push(atend_hook_closure);

  let end_envname_constructor = Rc::new(Constructor {
    cs: T_CS!(end_name),
    replacement: None,
    paramlist: None,
    before_digest: before_digest_for_endenv,
    after_digest: after_digest_env,
    ..Constructor::default() // TODO ? fill in missing ones
  });
  install_definition(end_envname_constructor, options.scope);

  // For the uncommon case opened by \csname env\endcsname
  let name_constructor = Rc::new(Constructor {
    cs: T_CS!(s!("\\{}", &name)),
    paramlist,
    replacement: compiled_replacement,
    // beforeDigest => flatten(($options{requireMath} ? (sub { requireMath($name); }) : ()),
    //   ($options{forbidMath} ? (sub { forbidMath($name); })              : ()),
    //   ($mode                ? (sub { $_[0]->beginMode($mode); })        : ()),
    //   ($options{font}       ? (sub { MergeFont(%{ $options{font} }); }) : ()),
    //   $options{beforeDigest}),
    // afterDigest     => flatten($options{afterDigestBegin}),
    // afterDigestBody => flatten($options{afterDigestBody}),
    // beforeConstruct => flatten(sub { >pushFrame; }, $options{beforeConstruct}),
    // Curiously, it's the \begin whose afterConstruct gets called.
    // afterConstruct => flatten($options{afterConstruct}, sub { >popFrame; }),
    nargs: options.nargs,
    capture_body: true,
    properties: options.properties.clone(),
    // (defined $options{reversion} ? (reversion => $options{reversion}) : ()),
    // (defined $sizer ? (sizer => $sizer) : ()),
    // ), $options{scope});
    ..Constructor::default()
  });
  install_definition(name_constructor, options.scope);
  let end_name = s!("\\end{}", &name);
  let mut after_digest_end = options.after_digest;
  after_digest_end.push(after_digest_simple!(_whatsit, {
    egroup()?;
  }));

  let end_name_constructor = Constructor {
    cs: T_CS!(end_name),
    paramlist: None,
    replacement: None,
    after_digest: after_digest_end,
    // beforeDigest => flatten($options{beforeDigestEnd}),
    //   ($mode ? (sub { $_[0]->endMode($mode); }) : ())),
    // ), $options{scope});
    ..Constructor::default()
  };
  install_definition(Rc::new(end_name_constructor), options.scope);

  if options.locked {
    assign_value(
      &s!("\\begin{{{}}}:locked", &name),
      true,
      Some(Scope::Global),
    );
    assign_value(&s!("\\end{{{}}}:locked", &name), true, Some(Scope::Global));
    assign_value(&s!("\\{}:locked", &name), true, Some(Scope::Global));
    assign_value(&s!("\\end{}:locked", &name), true, Some(Scope::Global));
  }
}

//======================================================================
// Support for XMDual

// Perhaps it would be better to use a label(-like) indirection here,
// so all ID's can stay in the desired format?
pub fn get_xmarg_id() -> Result<Tokens> {
  step_counter("@lx@xmarg", false)?;
  def_macro(
    T_CS!("\\@@lx@xmarg@ID"),
    None,
    Tokens!(Explode!(lookup_register("\\c@@lx@xmarg", Vec::new())?
      .unwrap()
      .value_of())),
    Some(ExpandableOptions {
      scope: Some(Scope::Global),
      ..ExpandableOptions::default()
    }),
  )?;
  gullet::do_expand(T_CS!("\\the@lx@xmarg@ID"))
}

type ArgsUnpacked = Vec<Option<Tokens>>;
/// Flesh out two dual (mathematical) forms of a given list of arguments.
///
/// Given a list of Tokens (to be expanded into mathematical objects)
/// return two lists
///   (1) The Tokens' wrapped in an XMAarg, with an ID added
///   (2) a corresponding list of Tokens creating XMRef's to those IDs
///
/// Ah, but there are complications!!!
/// On the one hand, arguments may be hidden, never appearing on the presentation side
/// (all will be passed to the content side); This argues for putting the XMArg's on the content
/// side. OTOH, they ought to be on the presentation side, so that they can be expanded & digested
/// in the proper context they will be presented, and pick up all the styling (font size,
/// displaystyle..) I don't know how to work around the latter, so we'll put args on the
/// presentation side, UNLESS they are hidden, in which case they'll be on the content side.
/// So, how do we know if they're hidden? We'll scan the presentation for #\d, that's how!
pub fn dualize_arglist(
  presentation: &str,
  args: Vec<Option<Tokens>>,
) -> Result<(ArgsUnpacked, ArgsUnpacked)> {
  let mut used = HashMap::default();
  for cap in ARG_HOLE.captures_iter(presentation) {
    // Get the args that were actually used!
    let argi = cap.get(1).unwrap().as_str();
    let entry = used.entry(argi.parse::<usize>().expect(argi)).or_insert(0);
    *entry += 1;
  }
  let mut cargs = Vec::new();
  let mut pargs = Vec::new();
  for (index, arg_opt) in args.into_iter().enumerate() {
    match arg_opt {
      None => {
        pargs.push(None);
        cargs.push(None);
      },
      Some(arg) if arg.unlist_ref().is_empty() => {
        pargs.push(Some(arg.clone()));
        cargs.push(Some(arg));
      },
      Some(arg_toks) => {
        if used.get(&(1 + index)).unwrap_or(&0) > &0 {
          // used in presentation?
          let id = get_xmarg_id()?;
          pargs.push(Some(Tokens!(
            T_CS!("\\lx@xmarg"),
            T_BEGIN!(),
            id.clone().unlist(),
            T_END!(),
            T_BEGIN!(),
            arg_toks.unlist(),
            T_END!()
          ))); // put XMArg in presentation
          cargs.push(Some(Tokens!(
            T_CS!("\\lx@xmref"),
            T_BEGIN!(),
            id.unlist(),
            T_END!()
          )));
        } else {
          // Hidden arg, put XMArg in content.
          let id = get_xmarg_id()?;
          cargs.push(Some(Tokens!(
            T_CS!("\\lx@xmarg"),
            T_BEGIN!(),
            id.clone().unlist(),
            T_END!(),
            T_BEGIN!(),
            arg_toks.unlist(),
            T_END!()
          )));
          pargs.push(Some(Tokens!(
            T_CS!("\\lx@xmref"),
            T_BEGIN!(),
            id.unlist(),
            T_END!()
          )));
        }
      },
    }
  }
  Ok((cargs, pargs))
}

/// Define a Mathematical symbol or function.
///
/// There are two sets of cases:
///  (1) If the presentation appears to be TeX code, we create an XMDual,
/// since the presentation may end up with structure, etc.
///  (2) But if the presentation is a simple string, or unicode,
/// it is just the content of the symbol; even if the function takes arguments.
// ALSO
//  arrange that the operator token gets cs="$cs"
// ALSO
//  Possibly some trick with SUMOP/INTOP affecting limits ?
//  Well, not exactly, but....
// HMM.... Still fishy.
// When to make a dual ?
// If the $presentation seems to be TeX (ie. it involves #1... but not ONLY!)
pub fn def_math(
  cs: Token,
  paramlist: Option<Parameters>,
  presentation: String,
  mut options: MathPrimitiveOptions,
) -> Result<()> {
  // Can't defer parsing parameters since we need to know number of args!
  // $paramlist = parseParameters($paramlist, $cs) if defined $paramlist && !ref $paramlist;

  let nargs = match paramlist {
    Some(ref plist) => plist.get_num_args(),
    None => 0,
  };
  let csname = cs.with_str(ToString::to_string);
  let name_opt = {
    let name = match options.name {
      Some(ref name) => Cow::Owned(name.to_owned()),
      None => {
        let mut inferred_name = match options.alias {
          Some(ref alias) => Cow::Owned(alias.to_owned()),
          None => Cow::Borrowed(&csname),
        };
        if inferred_name.starts_with('\\') {
          inferred_name = Cow::Owned(inferred_name.replacen('\\', "", 1))
        }
        inferred_name
      },
    };
    let meaning_check = options
      .meaning
      .as_ref()
      .map_or_else(|| Cow::Owned(String::new()), Cow::Borrowed);
    if (*name == presentation) || (name.is_empty()) || *name == *meaning_check {
      None
    } else {
      Some(name.into_owned())
    }
  };
  options.name = name_opt;
  if nargs == 0 && options.role.is_none() {
    options.role = Some(String::from("UNKNOWN"))
  }
  if nargs > 0 && options.operator_role.is_none() {
    options.operator_role = Some(String::from("UNKNOWN"))
  }
  if options.hide_content_reversion {
    options.revert_as = Some(Cow::Borrowed("context"));
  }

  let locked = options.locked;
  // Store some data for introspection
  // defmath_introspective(cs, paramlist, presentation, options);

  // If single character, handle with a rewrite rule
  if csname.len() == 1 {
    let mut math_attr_hash: HashMap<String, String> = HashMap::default();
    transfer_opt_default!(name, options, math_attr_hash);
    transfer_opt_default!(meaning, options, math_attr_hash);
    transfer_opt_default!(omcd, options, math_attr_hash);
    transfer_opt_default!(decl_id, options, math_attr_hash);
    transfer_opt_default!(role, options, math_attr_hash);
    transfer_opt_default!(replace, options, math_attr_hash);
    transfer_opt_default!(mathstyle, options, math_attr_hash);
    transfer_opt_default!(stretchy, options, math_attr_hash);
    assign_value(
      &s!("math_token_attributes_{}", csname),
      math_attr_hash,
      Some(Scope::Global),
    );
  }
  // If the macro involves arguments,
  // we will create an XMDual to separate simple content application
  // from the (likely) convoluted presentation.
  else if HAS_ARG_OR_CS.is_match(&presentation) {
    // TODO: Are the code variants still applicable in Rust?
    //((ref presentation eq "CODE")
    // || ((ref presentation) && grep { $_->equals(T_PARAM) } presentation->unlist)
    // || ((ref presentation) && (grep { $_->isExecutable } presentation->unlist)))
    def_math_dual(cs, paramlist, presentation, options)?;
  }
  // EXPERIMENT: Introduce an intermediate case for simple symbols
  // Define a primitive that will create a Box with the appropriate set of XMTok attributes.
  else if nargs == 0 && !options.has_complex_option() {
    def_math_primitive(cs, paramlist, presentation, options);
  } else {
    def_math_constructor(cs, paramlist, presentation, options)?;
  }
  if locked {
    assign_value(&format!("{csname}:locked"), true, Some(Scope::Global));
  }
  Ok(())
}

/// Transfers the common MathPrimitive options to a (ideally freshly instantiated) Constructor.
fn transfer_common_constructor_options(
  cs: &Token,
  presentation: &str,
  options: MathPrimitiveOptions,
  cons: &mut Constructor,
) {
  let cs_str = cs.with_str(ToString::to_string);
  let mut properties = options.to_hash_stored();
  cons.alias = Some(options.alias.unwrap_or_else(|| cs_str.to_owned()));
  if let Some(sizer) = infer_sizer(options.sizer.as_ref(), options.reversion.as_ref()) {
    cons.sizer = Some(sizer);
  }
  if let Some(reversion) = options.reversion {
    cons.reversion = Some(reversion);
  }
  //
  // before_digest
  //
  let mut before_digest_closures: Vec<BeforeDigestClosure> = vec![before_digest_simple!({
    requireMath!(cs_str);
  })];
  if !options.nogroup {
    before_digest_closures.push(before_digest_simple!({
      bgroup();
    }));
  }
  if let Some(font) = options.font {
    before_digest_closures.push(before_digest_simple!({
      if let FontDirective::Asset(ref chosen_font) = font {
        merge_font((**chosen_font).clone());
      }
    }));
  }
  before_digest_closures.extend(options.before_digest);
  cons.before_digest = before_digest_closures;
  //
  // after_digest
  //
  let mut after_digest_closures = options.after_digest;
  if !options.nogroup {
    after_digest_closures.push(after_digest_simple!(_args, {
      egroup()?;
    }));
  }
  cons.after_digest = after_digest_closures;
  cons.before_construct = options.before_construct;
  cons.after_construct = options.after_construct;
  let presentation_for_font = presentation.to_owned();
  properties.insert(
    "font",
    Stored::FontDirective(FontDirective::Closure(
      if let Some(mathstyle) = options.mathstyle {
        Rc::new(move |_whatsit| {
          Ok(
            lookup_font()
              .unwrap()
              .merge(Font {
                mathstyle: Some(Cow::Owned(mathstyle.clone())),
                ..Font::default()
              })
              .specialize(&presentation_for_font),
          )
        })
      } else {
        Rc::new(move |_whatsit| Ok(lookup_font().unwrap().specialize(&presentation_for_font)))
      },
    )),
  );

  cons.properties = Rc::new(move |_args| Ok(properties.clone()));
}

//======================================================================
// Allocated registers.
// We ASSUME the same set of \count positions used by TeX & LaTeX
// for recording the next available position in \count,\dimen,\skip,\muskip.

pub fn allocate_register(rtype: &str) -> Result<Option<String>> {
  let addr = match rtype {
    "\\count" => "\\count10",
    "\\dimen" => "\\count11",
    "\\skip" => "\\count12",
    "\\muskip" => "\\count13",
    "\\box" => "\\count14",
    "\\toks" => "\\count15",
    _ => "",
  };
  if !addr.is_empty() {
    // addr is a Register but MUST be stored as \count<#>
    if let Some(n) = lookup_number(addr) {
      let next = n.value_of() + 1;
      assign_value(addr, Number::new(next), Some(Scope::Global));
      Ok(Some(format!("{rtype}{next}")))
    } else {
      Ok(None)
    }
  } else {
    Error!(
      "misdefined",
      rtype,
      format!("Type {rtype} is not an allocated register type")
    );
    Ok(None)
  }
}
