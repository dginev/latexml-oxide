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
  token!(postsubscript ~ "POSTSUBSCRIPT");
  token!(postsuperscript ~ "POSTSUPERSCRIPT");
  token!(floatsuperscript ~ "FLOATSUPERSCRIPT");
  token!(floatsubscript ~ "FLOATSUBSCRIPT");

rules!(
    // Factors
    factor = unknown | number;
    // Terms
    term = factor
      | term mulop factor => infix_apply
      | factor factor => invisible_infix_mulop;
    // Expressions
    expression = term
      | expression addop term => infix_apply
      | addop term => prefix_apply
      | factor addop => postfix_apply;

    // Formula
    formula = expression
      | formula relop expression => infix_apply;

    // // Note that we are _EXTENDING_ the original term_argument declaration,
    // //                  as at the type of definition we couldn't yet discuss "expression" or "tex_argument"
    // factor += lparen expression rparen          => circumfix_fenced
    //               | lbrace expression rbrace           => circumfix_fenced
    //               | lbracket expression rbracket       => circumfix_fenced
    //               | lbrace formula rbrace              => circumfix_fenced

    // anyop = addop | mulop | relop | metarelop
    //   | bigop | sumop | intop
    //   | limitop | diffop;

    // todotoken = atom | id | array | punct | period  | vertbar
    // | lbrace | rbrace | lparen | rparen | lbracket | rbracket
    // | modifierop | modifier | arrow | binop
    // | postfix | function | opfunction | trigfunction | applyop
    // | composeop | supop | middle
    // | operator | postsubscript | postsuperscript
    // | floatsuperscript | floatsubscript
    // | langle_open
    // | rangle_close;
    // Inaccessible tokens?
    // | langle | rangle | midbar | open | close | middle_parallel
    // | singlevertbar | middle_bar |  langle_rel | rangle_rel

    anything = formula //| anyop | todotoken
  );
  // | term_argument postsuperscript tex_argument  => post_script
  // | term_argument postsubscript tex_argument    => post_script
  start!(anything);

  // Also prepare the tree builder rules here (for now)
  Ok((grammar!(), actions!(), builder!()))
}
