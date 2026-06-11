//! TeX Debugging
//!
//! Core TeX Implementation for LaTeXML
static EXCEPTION_MACRO_NAMES_FOR_MEANING: Lazy<Regex> =
  Lazy::new(|| Regex::new("^\\\\(?:(?:un)?expanded|detokenize)$").unwrap());
static LEAD_W_COLON_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\w+):").unwrap());
static UNTIL_SPEC: Lazy<Regex> = Lazy::new(|| Regex::new("^\\w?Until(\\w*):").unwrap());

// TODO: Rethink the numeric juggling here to make sense in our low-level proglang.
static TRACE_MACROS: u8 = 0x1;
static TRACE_COMMANDS: u8 = 0x2;
static _TRACE_ALL: u8 = 0x3; // MACROS | COMMANDS
static _TRACE_PROFILE: u8 = 0x4;

use crate::prelude::*;

LoadDefinitions!({
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Debugging Family of primitive control sequences
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%

  DefConstructor!("\\lx@ERROR{}{}", "<ltx:ERROR class='ltx_#1'>#2</ltx:ERROR>");

  //======================================================================
  // running modes
  //----------------------------------------------------------------------
  // \batchmode        c  acts like pressing Q in response to an error.
  // \errorstopmode    c  switches to normal interaction for processing errors.
  // \nonstopmode      c  acts like pressing R in response to an error.
  // \scrollmode       c  acts like pressing S in response to an error.
  // \pausing          pi if positive, the program halts after every line is read from the input
  // file and waits for a response from the user.

  // These are no-ops; Basically, LaTeXML runs in scrollmode
  DefPrimitive!(T_CS!("\\errorstopmode"), None, None);
  DefPrimitive!(T_CS!("\\scrollmode"), None, None);
  DefPrimitive!(T_CS!("\\nonstopmode"), None, None);
  DefPrimitive!(T_CS!("\\batchmode"), None, None);
  DefRegister!("\\pausing", Number!(0));

  //======================================================================
  // Messages
  //----------------------------------------------------------------------
  // \message          c  writes an expanded token list on the terminal and to the log file.
  // \errmessage       c  displays text on the terminal and interrupts the program.
  // \errhelp          pt is text displayed on the terminal if h is pressed after an \errmessage
  // . \errorcontextlines pi is the number of lines to display on the terminal at an error.

  // Converts $tokens to a string in the fashion of \message and others:
  // doubles #, converts to string; optionally adds spaces after control sequences
  // in the spirit of the B Book, "show_token_list" routine, in 292.
  // [This could be a $tokens->unpackParameters, but for the curious space treatment]
  DefPrimitive!("\\message{}", sub [(message)] {
    if current_verbosity() > -1 {
      Note!(writable_tokens(&do_expand(message)?));
    }
  });

  DefRegister!("\\errhelp", Tokens!());
  DefPrimitive!("\\errmessage{}", sub[(args)] {
    let message = Expand!(args);
    let help = Expand!(Tokens!(T_CS!("\\the"), T_CS!("\\errhelp")));
    Note!(s!("{}: {}", message, help));
  });
  DefRegister!("\\errorcontextlines", Number!(5));

  //======================================================================
  // meaning
  //----------------------------------------------------------------------
  // \meaning          c  adds characters describing a token to the output stream.
  // Not sure about this yet...
  // NOTE: Lots of back-and-forth mangle with definition vs cs; don't do that!
  DefMacro!("\\meaning Token", sub[(token)] {
    let mut meaning = String::from("undefined");
    if let Some(definition) = if token == T_ALIGN!() {
      Some(Stored::Token(token))
    } else {
      lookup_meaning(&token)
    } {
      // First, if this definition is a primitive|conditional|constructor,
      // check to see if it has an alias, which would allow us to work with a token
      // Check for font-defined primitives: \meaning\fiverm => "select font cmr5"
      // Perl: only shows "at Xpt" when explicit "at" or "scaled" was used in \font
      if let Stored::Primitive(_) = &definition {
        let cs_str = token.to_string();
        let key = s!("fontinfo_{}", cs_str);
        // with_value avoids the Stored envelope clone on the Font arm;
        // we only need the font's name string out.
        let name_opt = with_value(&key, |v| match v {
          Some(Stored::Font(f)) => f.name.as_ref().map(|n| n.to_string()),
          _ => None,
        });
        if let Some(name) = name_opt {
          let at_key = s!("fontinfo_at_{}", cs_str);
          let at_info = with_value(&at_key, |v| match v {
            Some(Stored::String(s)) => with(*s, |at| format!(" at {at}")),
            _ => String::new(),
          });
          meaning = format!("select font {}{}", name, at_info);
          return Ok(Tokens::new(Explode!(meaning)));
        }
      }
      let definition : Stored = match definition {
        Stored::Primitive(primitive) =>
          Stored::Token(primitive.get_cs_or_alias().into_owned()),
        Stored::Constructor(constructor) =>
          Stored::Token(constructor.get_cs_or_alias().into_owned()),
        Stored::Conditional(cond) =>
          Stored::Token(cond.get_cs_or_alias().into_owned()),
        other => other
      };

      // Now that we've tried to obtain an expandable definition, do the TeX dance:
      match definition {
        Stored::Token(t) => {
          let cc = t.get_catcode();
          let text = if cc == Catcode::SPACE {
            String::from(" ")
          } else {
            t.to_string()
          };
          meaning = String::from(cc.meaning());
          if !meaning.is_empty() {
            meaning.push(' ');
          }
          meaning.push_str(&text);
        },
        Stored::Register(register) => {
          meaning = register.get_address().to_string();
        },
        Stored::Expandable(expandable) => {
          // short-circuit some troublesome discrepancies with TeX, which end up macros on our end,
          // but \meaning expects as primitives in the CTAN ecosystem.
          let cs = expandable.get_cs_or_alias().to_string();
          // These exceptions could be extended further, as we add more .sty/.cls support
          if EXCEPTION_MACRO_NAMES_FOR_MEANING.is_match(&cs) {
            return Ok(Tokens::new(Explode!(cs)));
          }
          let params = match expandable.get_parameters() {
            Some(ps) => ps.get_parameters(),
            None => Vec::new()
          };
          let mut spec_parts : Vec<SymStr> = Vec::new();
          let mut p_trailer = "";
          // params.iter().map(|param| LEAD_W_COLON_RE.replace(&param.spec,"") ).collect();
          let mut arg_index = 0;
          for param in params.iter() {
            let mut p_spec = pin!("");
            let mut continue_flag = false;
            // TODO: avoiding the allocation is quite painful here, since arena gets into mutability
            // locking
            let spec = to_string(param.spec);
            match spec.as_str() {
              "RequireBrace" => {
                // tex's \meaning prints out the required braces for "\def\a#{}" variants
                p_trailer = "{";
                p_spec    = pin_static("{");
              },
              "UntilBrace" => {
                p_trailer = "{";
                arg_index+=1;
                p_spec = pin(
                  with(p_spec, |p_str| format!("#{arg_index}{p_str}")));
              }
              other if other.starts_with("Match:") => {
                // just match, don't increment arg index
                p_spec = pin(LEAD_W_COLON_RE.replace(other,""));
              },
              other if UNTIL_SPEC.is_match(other) => {
                // implied argument at this slot
                p_spec = pin(LEAD_W_COLON_RE.replace(other,""));
                arg_index +=1 ;
                p_spec = pin(
                  with(p_spec, |p_str| s!("#{arg_index}{p_str}")));
              },
              _other => { // regular parameter, increment
              // skip the latexml-only requirement params, but only here,
              // since Match also have "novalue" set.
                if param.novalue {
                  continue_flag = true;
                } else {
                  arg_index+=1;
                  // ALL parameters — including optional ones — render as a
                  // plain `#N`, matching Perl `\meaning`. Perl's `\meaning`
                  // reflects LaTeXML's internal *parameter count*, not TeX
                  // bracket syntax: an optional-arg command like
                  // `\newcommand{\foo}[2][d]{...}` (or `\cite`) shows
                  // `macro:#1#2->…`, NOT `macro:[#1]#2->…`.
                  //
                  // A prior divergence rendered optional params as `[#N]`
                  // to placate etoolbox `\robustify` (2110.11931). That was
                  // wrong: `\robustify` round-trips a CS through
                  // `\meaning`+`\scantokens`+`\def`, so a literal `[#1]`
                  // becomes a *delimited* parameter — the rebuilt `\cite{x}`
                  // then forward-scans for `[`, swallowing the next
                  // environment's optional arg (e.g. `\begin{figure}[th]`),
                  // and the float collapses with "\caption outside any
                  // known float" / "Can't close environment figure".
                  // Driver: 1908.01908. With plain `#N` (Perl-faithful) the
                  // rebuilt body is still opaque `CODE(...)` garbage in BOTH
                  // engines — `\robustify` cannot reconstruct a closure-
                  // backed primitive — but argument scanning stays
                  // undelimited, exactly as in Perl, so following optional
                  // args are untouched. Perl emits the same `CODE(...)`
                  // text and zero errors.
                  p_spec = pin(s!("#{arg_index}"));
                }
              }
            }
            if !continue_flag {
              spec_parts.push(p_spec);
            }
          }
          let mut spec : String = join(&spec_parts,"");
          spec = spec.replace("{}","");
          spec = spec.replace("Token","");

          let mut prefixes = String::new();
          if expandable.is_protected {
            prefixes.push_str("\\protected");
          }
          if expandable.is_long {
            prefixes.push_str("\\long");
          }
          if expandable.is_outer {
            prefixes.push_str("\\outer");
          }
          if !prefixes.is_empty() {
            prefixes.push(' ');
          }
          let expansion = match expandable.get_expansion() {
            None => String::new(),
            // TODO: How to print closures? This follows Perl's raw pointer format
            Some(ExpansionBody::Closure(exp)) => format!("CODE({:p})", Rc::as_ptr(exp)),
            Some(ExpansionBody::Tokens(tks)) => writable_tokens(tks)
          };
          meaning = format!("{prefixes}macro:{spec}->{expansion}{p_trailer}");
        },
        e => {
          // Handle other Stored variants gracefully (e.g., Register, Constructor, etc.)
          meaning = format!("{e}");
        }
      }
    }
    ExplodeChars!(meaning)
  });

  //======================================================================
  // Showing internal things
  //----------------------------------------------------------------------

  // \show             c  writes a token's definition on the terminal and to the log file.
  // \showbox          c  writes the contents of a box to the log file.
  // \showlists        c  writes information about current lists to the log file.
  // \showthe          c  writes a value on the terminal and to the log file and interrupts the
  // program. \showboxbreadth   pi is the maximum number of items per level written by \showbox
  // and \showlists. \showboxdepth     pi is the maximum level written by \showbox and \showlists.

  // Debugging aids; Ignored!
  DefPrimitive!("\\show Token", sub[(arg)] {
    let lhs = if arg.get_catcode() == Catcode::CS {
      s!("{arg}=")
    } else { String::new() };
    let stuff = Invocation!(T_CS!("\\meaning"), vec![arg]);
    let rhs = writable_tokens(&Expand!(stuff));
    Note!(s!("> {lhs}{rhs}\n{}", get_locator()));
  });
  DefPrimitive!("\\showbox Number", sub[(arg)] {
    let n     = arg.value_of();
    Debug!("Box {n} = {:?}", lookup_value(&s!("box{n}")));
  });
  def_primitive_noop("\\showlists")?;
  def_primitive_noop("\\showthe Token")?;
  DefRegister!("\\showboxbreadth", Number!(5));
  DefRegister!("\\showboxdepth", Number!(3));

  //======================================================================
  // Tracing
  //----------------------------------------------------------------------
  // \tracingcommands   pi if positive, writes commands to the log file.
  //
  // \tracinglostchars  pi if positive, writes characters not in the current font to the log file.
  //
  // \tracingmacros     pi if positive, writes to the log file when expanding macros and
  //                       arguments .
  //
  // \tracingonline     pi if positive, writes diagnostic output to the terminal as
  //                       well as to the log file.
  //
  // \tracingoutput     pi if positive, writes contents of shipped out
  //                       boxes to the log file.
  //
  // \tracingpages      pi if positive, writes the page-cost calculations
  //                       to the log file.
  //
  // \tracingparagraphs pi if positive, writes a summary of the line-breaking
  //                       calculations to the  log file.
  //
  // \tracingrestores   pi if positive, writes save-stack
  //                       details to the log file.
  //
  // \tracingstats      pi if positive, writes memory usage statistics to the log file.
  //
  AssignValue!("tracingmacros"   => Number!(0));
  AssignValue!("tracingcommands" => Number!(0));
  DefRegister!("\\tracingmacros", Number!(0),
  getter => { LookupNumber!("tracingmacros") },
  setter => sub[value,scope,_args] {
    let v = value.value_of();
    AssignValue!("tracingmacros" => v, scope);
    let p : u8 = lookup_int("TRACING") as u8;
    AssignValue!("TRACING" => if v > 0 { p | TRACE_MACROS  } else { p & !TRACE_MACROS });
  });
  DefRegister!("\\tracingcommands", Number!(0),
  getter => { LookupNumber!("tracingcommands") },
  setter => sub[value,scope,_args] {
    let v = value.value_of();
    AssignValue!("tracingcommands" => v, scope);
    let p : u8 = lookup_int("TRACING") as u8;
    AssignValue!("TRACING" => if v > 0 { p | TRACE_COMMANDS  } else { p & !TRACE_COMMANDS });
  });

  DefRegister!("\\tracingonline", Number!(0));
  DefRegister!("\\tracingstats", Number!(0));
  DefRegister!("\\tracingparagraphs", Number!(0));
  DefRegister!("\\tracingpages", Number!(0));
  DefRegister!("\\tracingoutput", Number!(0));
  DefRegister!("\\tracinglostchars", Number!(1));
  DefRegister!("\\tracingrestores", Number!(0));
});
