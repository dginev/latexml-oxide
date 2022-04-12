use crate::semantics::*;
use marpa::grammar::Grammar as MarpaGrammar;
use marpa::result::Result;
use marpa::tree_builder::TreeBuilder;

#[allow(unused_macros)]
pub fn init_grammar() -> Result<(MarpaGrammar, Actions, TreeBuilder)> {
  // We create a declarative macro language of our own, in the spirit of the Marpa SLIF
  default_registry!();
  // Tokens, to be used in rules directly
  token!(atom ~ "ATOM");
  token!(unknown ~ "UNKNOWN");
  token!(id ~ "ID");
  token!(array ~ "ARRAY");
  token!(number ~ "NUMBER");
  token!(punct ~ "PUNCT");
  token!(period ~ "PERIOD");
  token!(addop ~ "ADDOP");
  token!(mulop ~ "MULOP");
  token!(relop ~ "RELOP");
  token!(elideop ~ "ELIDEOP");
  token!(langle_rel = "RELOP:less-than");
  token!(langle_open = "OPEN:langle");
  token!(langle = [langle_rel langle_open]);
  token!(rangle_rel = "RELOP:greater-than");
  token!(rangle_close = "CLOSE:rangle");
  token!(rangle =[rangle_rel rangle_close]);
  token!(vertbar ~ "VERTBAR");
  token!(singlevertbar = "VERTBAR:|");
  token!(middle_bar = "MIDDLE:|");
  token!(middle_parallel = "MIDDLE:parallel-to");
  token!(midbar = [vertbar middle_bar middle_parallel]);
  token!(lbrace = "OPEN:{");
  token!(rbrace = "CLOSE:}");
  token!(lparen = "OPEN:(");
  token!(rparen = "CLOSE:)");
  token!(lbracket = "OPEN:[");
  token!(rbracket = "CLOSE:]");
  token!(metarelop ~ "METARELOP");
  token!(modifierop ~ "MODIFIEROP");
  token!(modifier ~ "MODIFIER");
  token!(arrow ~ "ARROW");
  token!(binop ~ "BINOP");
  token!(postfix ~ "POSTFIX");
  token!(function ~ "FUNCTION");
  token!(opfunction ~ "OPFUNCTION");
  token!(trigfunction ~ "TRIGFUNCTION");
  token!(applyop ~ "APPLYOP");
  token!(composeop ~ "COMPOSEOP");
  token!(supop ~ "SUPOP");
  token!(open ~ "OPEN");
  token!(close ~ "CLOSE");
  token!(middle ~ "MIDDLE");
  token!(bigop ~ "BIGOP");
  token!(sumop ~ "SUMOP");
  token!(intop ~ "INTOP");
  token!(limitop ~ "LIMITOP");
  token!(diffop ~ "DIFFOP");
  token!(operator ~ "OPERATOR");
  token!(start_postsubscript ~ "start_POSTSUBSCRIPT");
  token!(end_postsubscript ~ "end_POSTSUBSCRIPT");
  token!(start_postsuperscript ~ "start_POSTSUPERSCRIPT");
  token!(end_postsuperscript ~ "end_POSTSUPERSCRIPT");
  token!(start_floatsuperscript ~ "start_FLOATSUPERSCRIPT");
  token!(end_floatsuperscript ~ "end_FLOATSUPERSCRIPT");
  token!(start_floatsubscript ~ "start_FLOATSUBSCRIPT");
  token!(end_floatsubscript ~ "end_FLOATSUBSCRIPT");

  rules!(
    // Factors
    factor_base = unknown | number | id | atom;
    factor = factor_base;
    // Terms
    tight_term = factor
      | tight_term factor => invisible_times;

    term = tight_term
      | term mulop tight_term => infix_apply_nary
      | term mulop tight_term elideop => infix_apply_and_elide;

    // Expressions
    expression = term
      | expression addop term => infix_apply_nary
      | expression addop term elideop => infix_apply_and_elide
      | addop tight_term => prefix_apply
      | factor addop => postfix_apply;

    // Formula
    formula = expression
      | formula relop expression => infix_relation;

    statement = formula
      | statement metarelop formula => infix_relation;

    statements = statement
      | statement punct statement => infix_apply;

    // Extensions, now that we have more category variables defined
    fenced_factor = lbrace expression rbrace    => circumfix_fenced
           | lbracket expression rbracket       => circumfix_fenced
           | lparen formula rparen              => circumfix_fenced;

    factor += fenced_factor;


    // Script content
    postsubscript = start_postsubscript expression end_postsubscript => faux_wrap;
    postsuperscript = start_postsuperscript expression end_postsuperscript => faux_wrap;
    floatsubscript = start_floatsubscript expression end_floatsubscript => faux_wrap;
    floatsuperscript = start_floatsuperscript expression end_floatsuperscript => faux_wrap;

    // Scripted factors -- avoid adding ambiguity in the left-right order of collection
    // first ALL left (=float), then right (=post).
    scripted_factor_l11 = floatsuperscript factor_base => prefix_script;
    scripted_factor_l12 = floatsubscript factor_base => prefix_script;
    scripted_factor_l1 = scripted_factor_l11 | scripted_factor_l12;
    scripted_factor_l2 = floatsuperscript scripted_factor_l12 => prefix_script
      | floatsubscript scripted_factor_l11 => prefix_script;

    scripted_factor_r11 = factor_base postsuperscript => postfix_script
      | scripted_factor_l1 postsuperscript => postfix_script
      | scripted_factor_l2 postsuperscript => postfix_script;
    scripted_factor_r12 = factor_base postsubscript => postfix_script
      | scripted_factor_l1 postsubscript => postfix_script
      | scripted_factor_l2 postsubscript => postfix_script;
    scripted_factor_r1 = scripted_factor_r11 | scripted_factor_r12;
    scripted_factor_r2 = scripted_factor_r12 postsuperscript => postfix_script
      | scripted_factor_r11 postsubscript => postfix_script;
    factor += scripted_factor_l1 | scripted_factor_l2 | scripted_factor_r1 | scripted_factor_r2;

    anyop = addop | mulop | relop | metarelop
      | bigop | sumop | intop
      | limitop | diffop | vertbar;

    anything = statements | anyop
  );
  // | term_argument postsuperscript tex_argument  => post_script
  // | term_argument postsubscript tex_argument    => post_script
  start!(anything);

  // Also prepare the tree builder rules here (for now)
  Ok((grammar!(), actions!(), builder!()))
}
