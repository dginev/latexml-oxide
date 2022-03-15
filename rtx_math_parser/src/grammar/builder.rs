use crate::semantics::*;
use marpa::grammar::Grammar as MarpaGrammar;
use marpa::result::Result;
use marpa::tree_builder::TreeBuilder;

#[allow(unused_macros)]
pub fn init_grammar() -> Result<(MarpaGrammar, Actions, TreeBuilder)> {
  // We create a declarative macro language of our own, in the spirit of the Marpa SLIF
  default_registry!();

  
  // Tokens, to be used in rules directly
  // let ws_char = g.string_set(None, "\t\n\r ")?;
  // b.discard(ws_char);

  token!(ATOM ~ "ATOM");
  token!(UNKNOWN ~ "UNKNOWN");
  token!(ID ~ "ID");
  token!(ARRAY ~ "ARRAY");
  token!(NUMBER ~ "NUMBER");
  token!(PUNCT ~ "PUNCT");
  token!(PERIOD ~ "PERIOD");
  token!(RELOP ~ "RELOP");
  // LANGLE          : /RELOP:less-than:\d+/
  //                 | /OPEN:langle:\d+/       
  // RANGLE          : /RELOP:greater-than:\d+/
  //                 | /CLOSE:rangle:\d+/      
  // token!(MIDBAR ~ "VERTBAR");
  //                 | /MIDDLE:\|:\d+/         
  //                 | /MIDDLE:parallel-to:\d+/
  token!(LBRACE = "OPEN:{");
  token!(RBRACE = "CLOSE:}");
  token!(LPAREN = "OPEN:(");
  token!(RPAREN = "CLOSE:)");
  token!(LBRACKET = "OPEN:[");
  token!(RBRACKET = "CLOSE:]");
  token!(METARELOP ~ "METARELOP");
  token!(MODIFIEROP ~ "MODIFIEROP");
  token!(MODIFIER ~ "MODIFIER");
  token!(ARROW ~ "ARROW");
  token!(ADDOP ~ "ADDOP");
  token!(MULOP ~ "MULOP");
  token!(BINOP ~ "BINOP");
  token!(POSTFIX ~ "POSTFIX");
  token!(FUNCTION ~ "FUNCTION");
  token!(OPFUNCTION ~ "OPFUNCTION");
  token!(TRIGFUNCTION ~ "TRIGFUNCTION");
  token!(APPLYOP ~ "APPLYOP");
  token!(COMPOSEOP ~ "COMPOSEOP");
  token!(SUPOP ~ "SUPOP");
  token!(OPEN ~ "OPEN");
  // this should be a rule, rather than a token
  // token!(SCRIPTOPEN = "OPEN:{");
  token!(CLOSE ~ "CLOSE");
  token!(MIDDLE ~ "MIDDLE");
  token!(VERTBAR ~ "VERTBAR");
  token!(SINGLEVERTBAR = "VERTBAR:|");
  token!(BIGOP ~ "BIGOP");
  token!(SUMOP ~ "SUMOP");
  token!(INTOP ~ "INTOP");
  token!(LIMITOP ~ "LIMITOP");
  token!(DIFFOP ~ "DIFFOP");
  token!(OPERATOR ~ "OPERATOR");
  token!(POSTSUBSCRIPT ~ "POSTSUBSCRIPT");
  token!(POSTSUPERSCRIPT ~ "POSTSUPERSCRIPT");
  token!(FLOATSUPERSCRIPT ~ "FLOATSUPERSCRIPT");
  token!(FLOATSUBSCRIPT ~ "FLOATSUBSCRIPT");

  rules!(
  // Factors
  factor   = UNKNOWN | NUMBER;
  // Terms
  term = factor
       | term MULOP factor => infix_apply
       | factor factor => invisible_infix_mulop;
  // Expressions
  expression = term
             | expression ADDOP term => infix_apply
             | ADDOP term => prefix_apply
             | factor ADDOP => postfix_apply;

  // Formula
  formula = expression;
  // Arguments, which are largely related to fencing, and feel "meta" to the operator hierarchy
  tex_argument  = LBRACE formula RBRACE              => circumfix_fenced;

  // Note that we are _EXTENDING_ the original term_argument declaration,
  //                  as at the type of definition we couldn't yet discuss "expression" or "tex_argument"
  factor += LPAREN expression RPAREN          => circumfix_fenced
                | LBRACE expression RBRACE           => circumfix_fenced
                | LBRACKET expression RBRACKET       => circumfix_fenced
  );
                // | term_argument postsuperscript tex_argument  => post_script
                // | term_argument postsubscript tex_argument    => post_script
  start!(formula);

  // Also prepare the tree builder rules here (for now)
  Ok((grammar!(), actions!(), builder!()))
}
