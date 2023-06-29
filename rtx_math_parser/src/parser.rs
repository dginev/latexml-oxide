use libxml::tree::{Node, NodeType};
use once_cell::sync::Lazy;
use regex::Regex;
use rustc_hash::FxHashMap as HashMap;
use std::borrow::Cow;
use std::io::Cursor;
use string_interner::symbol::SymbolU32;

use rtx_core::common::arena;
use rtx_core::common::error::{note_begin, note_end, note_progress, Result};
use rtx_core::common::xml::*;
use rtx_core::document::{get_node_qname,sym_can_have_attribute,with_node_qname,Document};
use rtx_core::{map, raw_map, s, static_map, Error, Fatal, fatal};

use crate::grammar::builder::init_grammar;
use crate::pragmatics::ValidationPragmatics;
use crate::semantics::*;
use crate::util::{filter_hints, node_to_grammar_lexemes};
use marpa::lexer::byte_scanner::*;
use marpa::parser::*;
use marpa::tree_builder::TreeBuilder;

static PREFIX_ALIAS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
  static_map!(
    "SUPERSCRIPTOP" => "^",
    "SUBSCRIPTOP" => "_",
    "times" => "*",
    "equals" => "=",
    "less-than" => "<",
    "greater-than" => ">",
    "less-than-or-equals" => "<=",
    "greater-than-or-equals" => ">=",
    "much-less-than" => "<<",
    "much-greater-than" => ">>",
    "plus" => "+",
    "minus" => "-",
    "divide" => "/")
});
// Put infix, along with `binding power'
static IS_INFIX: Lazy<HashMap<String, usize>> = Lazy::new(|| {
  map!(
  "METARELOP" => 1,
  "RELOP"         => 2,    "ARROW"       => 2,
  "ADDOP"         => 10,   "MULOP"       => 100, "FRACOP" => 100,
  "SUPERSCRIPTOP" => 1000, "SUBSCRIPTOP" => 1000)
});
static PRE_DIGITS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^pre\d+$").unwrap());

pub struct MathParser {
  // grammar: MarpaGrammar,
  actions: Actions,
  builder: TreeBuilder,
  engine: Parser,
  pub expert_pragmatics: Vec<ValidationPragmatics>,
  pub student_pragmatics: Vec<ValidationPragmatics>,
  passed: HashMap<SymbolU32, usize>,
  failed: HashMap<SymbolU32, usize>,
  unknowns: HashMap<SymbolU32, usize>,
  // punctuation: HashMap<String, usize>,
  // lostnodes: HashMap<String, Node>,
  // idrefs: Vec<(String, Node)>,
  maybe_functions: HashMap<SymbolU32, usize>,
  n_parsed: usize,
  // strict: bool,
  // warned: bool,
  // xnode: Option<Node>,
}
impl Default for MathParser {
  fn default() -> Self {
    let (grammar, actions, builder) = init_grammar().unwrap();
    let engine = Parser::with_grammar(grammar.unwrap());
    MathParser {
      engine,
      actions,
      builder,
      expert_pragmatics: ValidationPragmatics::expert_defaults(),
      student_pragmatics: ValidationPragmatics::student_defaults(),
      passed: HashMap::default(),
      failed: HashMap::default(),
      unknowns: HashMap::default(),
      maybe_functions: HashMap::default(),
      // punctuation: HashMap::default(),
      // lostnodes: HashMap::default(),
      // idrefs: Vec::new(),
      n_parsed: 0,
      // strict: true,
      // warned: false,
      // xnode: None,
    }
  }
  // sub new {
  //   my ($class, %options) = @_;
  //   require LaTeXML::MathGrammar;

  //   my $internalparser = LaTeXML::MathGrammar->new();
  //   Fatal("expected", "MathGrammar", undef,
  //     "Compilation of Math Parser grammar failed") unless $internalparser;

  //   my $self = bless { internalparser => $internalparser }, $class;
  //   return $self; }
}

// ================================================================================

impl MathParser {
  pub fn parse_math(&mut self, document: &mut Document) -> Result<()> {
    self.clear();
    self.cleanup_scripts(document);
    let xmath_selector = "descendant-or-self::ltx:XMath[not(ancestor::ltx:XMath)]";
    let xmath_nodes = document.findnodes(xmath_selector, None);

    if !xmath_nodes.is_empty() {
      note_begin("Math Parsing");
      note_progress(&s!("{:?} formulae ...", xmath_nodes.len()));
      for math in xmath_nodes {
        self.parse(math, document)?;
      }

      //     note_progress("\nMath parsing succeeded:"
      //         . join('', map { "\n   $_: "
      // . colorizeString(self.passed{$_} . "/" . (self.passed{$_} +
      // $$self{failed}{$_}), ($$self{failed}{$_} == 0 ? 'success' : 'warning')) }
      //           grep { self.passed{$_} + $$self{failed}{$_} }
      //           keys %{ self.passed }) . "\n");

      //     if (my @unk = keys %{ $$self{unknowns} }) {
      // note_progress("Symbols assumed as simple identifiers (with # of
      // occurences):\n   " . join(', ', map { "'" .
      // colorizeString("$_", 'warning') . "' ($$self{unknowns}{$_})" } sort @unk) .
      // "\n");       if (!$state->lookupValue('MATHPARSER_SPECULATE')) {
      // note_progress("Set MATHPARSER_SPECULATE to speculate on possible
      // notations.\n"); } } if (my @funcs = keys %{
      // $$self{maybe_functions} }) { note_progress("Possibly used as
      // functions?\n  " . join(', ', map { "'$_'
      // ($$self{maybe_functions}{$_}/$$self{unknowns}{$_} usages)" }
      // sort @funcs) . "\n"); }

      note_end("Math Parsing");
    }
    Ok(())
  }

  // This is a rather peculiar cleanup that needs to be done to manage ids &
  // idrefs Before parsing, sub/superscripts are represented by an
  // operator-less XMApp with the script itself as the only child. Ideally,
  // upon parsing these get merged, combined and disappear into proper XMApp of
  // an appropriate operator on the base and scripts.  Unless there is a parse
  // failure, in which case they remain.
  // The problem comes from various XMDual constructs where an XMRef refers to
  // the script XMApp. It can occur that one branch parses and the other fails:
  // This can leave a reference to the script XMApp which no longer exists!
  // To solve this, we find & replace all references to such script XMApps by an
  // explicit XMApp with the XMRef refering to the script itself, not the
  // XMApp. (make sense?)
  pub fn cleanup_scripts(&mut self, _document: &Document) {
    // unimplemented!();
  }
  // sub cleanupScripts {
  //   my ($self, $document) = @_;
  //   foreach my $app ($document->findnodes(
  // 'descendant-or-self::ltx:XMApp[@xml:id and
  // contains(@role,"SCRIPT")]')) {     my $role  = $app->getAttribute('role');
  //     my $appid = $app->getAttribute('xml:id');
  //     if ($role =~ /^(?:PRE|POST|FLOAT)(:?SUB|SUPER)SCRIPT$/) {
  // my @refs = $document->findnodes("descendant-or-self::ltx:XMRef[\@idref
  // = '$appid']");       if (scalar(@refs)) {
  // print STDERR "\nREPLACING SCRIPT REF: found " . scalar(@refs) . "
  // references to " . ToString($app) . "\n"; my $script =
  // $app->firstChild; my ($scriptref) =
  // LaTeXML::Package::createXMRefs($document, $script);
  //         $document->unRecordID($appid);    # no longer refers to the app
  //         $app->removeAttribute('xml:id');
  //         # Copy all attributes, EXCEPT xml:id
  //         my %attr = map { (getQName($_) => $_->getValue) }
  //           grep { $_->nodeType == XML_ATTRIBUTE_NODE } $app->attributes;
  // # Now, replace each ref to the script application by an application
  // to a ref to the script.         foreach my $ref (@refs) {
  //           $document->replaceTree(['ltx:XMApp', {%attr}, $scriptref], $ref); }
  //       } } }
  //   return; }

  // ================================================================================
  pub fn clear(&mut self) {
    self.passed = raw_map!(arena::pin_static("ltx:XMath") => 0, arena::pin_static("ltx:XMArg") => 0,
      arena::pin_static("ltx:XMWrap") => 0);
    self.failed = raw_map!(arena::pin_static("ltx:XMath") => 0,
      arena::pin_static("ltx:XMArg") => 0,
      arena::pin_static("ltx:XMWrap") => 0 );
    self.unknowns = HashMap::default();
    self.maybe_functions = HashMap::default();
    self.n_parsed = 0;
  }
  // our %EXCLUDED_PRETTYNAME_ATTRIBUTES = (fontsize => 1, opacity => 1);

  // sub token_prettyname {
  //   my ($node) = @_;
  //   my $name = $node->getAttribute('name');
  //   if (defined $name) { }
  //   elsif ($name = $node->textContent) {
  //     my $font = $LaTeXML::MathParser::DOCUMENT->getNodeFont($node);
  //     my %attr = $font->relativeTo(LaTeXML::Common::Font->textDefault);
  //     my $desc = join(' ', map { ToString($attr{$_}{value}) }
  //         (grep { !$EXCLUDED_PRETTYNAME_ATTRIBUTES{$_} } (sort keys %attr)));
  //     $name .= "{$desc}" if $desc; }
  //   else {
  //     $name = Stringify($node); }    # what else ????
  //   return $name; }

  // sub note_unknown {
  //   my ($self, $node) = @_;
  //   my $name = token_prettyname($node);
  //   $$self{unknowns}{$name}++;
  //   return; }

  // debugging utility, should be somewhere handy.
  // sub printNode {
  //   my ($node) = @_;
  //   if (ref $node eq 'ARRAY') {
  //     my ($tag, $attr, @children) = @$node;
  //     my @keys = sort keys %$attr;
  //     return "<$tag"
  //       . (@keys ? ' ' . join(' ', map { "$_='$$attr{$_}'" } @keys) : '')
  //       . (@children
  //       ? ">\n" . join('', map { printNode($_) } @children) . "</$tag>"
  //       : '/>')
  //       . "\n"; }
  //   else {
  //     return ToString($node); } }

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Parser
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Top-level per-formula parse.
  // We do a depth-first traversal of the content of the XMath element,
  // since various sub-elements (XMArg & XMWrap) act as containers of
  // nominally complete subexpressions.
  // We do these first for two reasons.
  // Firstly, since after parsing, the parent will be rebuilt from the result,
  // we lose the node "identity"; ie. we can't find the child to replace it!
  // Secondly, in principle (although this isn't used yet), parsing the
  // child could reveal something interesting about it; say, it's effective role.
  // Then, this information could be used when parsing the parent.
  // In fact, this could work the other way too; parsing the parent could tell
  // us something about what the child must be....
  fn parse(&mut self, mut xnode: Node, document: &mut Document) -> Result<()> {
    // This bit for debugging....
    // foreach my $n ($document->findnodes("descendant-or-self::*[\@xml:id]",
    // $xnode)) {     my $id = $n->getAttribute('xml:id');
    //     $LaTeXML::MathParser::IDREFS{$id} = $n; }
    if let Some(result) = self.parse_rec(&mut xnode, "Anything,", document)? {
      // Add text representation to the containing Math element.
      let mut p = xnode.get_parent().unwrap();
      // This is a VERY screwy situation? How can the parent be a document fragment??
      // This has got to be a LibXML bug???
      if p.get_type() == Some(NodeType::DocumentFragNode) {
        let child_nodes = p.get_child_nodes();
        if child_nodes.len() == 1 {
          p = child_nodes[0].clone();
        } else {
          fatal!(
            XMath,
            Malformed,
            "XMath node has DOCUMENT_FRAGMENT for parent!"
          );
          // xnode,
        }
      }
      // HACK: replace XMRef's to stray trailing punctution
      //     foreach my $id (keys %$LaTeXML::MathParser::PUNCTUATION) {
      //       my $r = $$LaTeXML::MathParser::PUNCTUATION{$id}->cloneNode;
      //       $r->removeAttribute('xml:id');
      // foreach my $n ($document->findnodes("descendant-or-self::ltx:XMRef[\@idref='$id']",
      // $p)) {         $document->replaceTree($r, $n); } }
      //     foreach my $id (keys %$LaTeXML::MathParser::LOSTNODES) {
      //       my $repid = $$LaTeXML::MathParser::LOSTNODES{$id};
      //       # but the replacement my have been replaced as well!
      //       while (my $reprepid = $$LaTeXML::MathParser::LOSTNODES{$repid}) {
      //         $repid = $reprepid; }
      //       if ($document->findnodes("descendant-or-self::*[\@xml:id='$id']")
      // &&
      // !$document->findnodes("descendant-or-self::*[\@xml:id='$repid']")) {
      // # Do nothing if the node never actually got replaced (parse ultimately
      // failed?)       }
      //       else {
      // foreach my $n
      // ($document->findnodes("descendant-or-self::ltx:XMRef[\@idref='$id']", $p)) {
      // $document->setAttribute($n, idref => $repid); } } }
      p.set_attribute("text", &text_form(&result, document))?;
    }
    Ok(())
  }

  // Recursively parse a node with some internal structure
  // by first parsing any structured children, then it's content.
  fn parse_rec(
    &mut self,
    node: &mut Node,
    rule_opt: &str,
    document: &mut Document,
  ) -> Result<Option<Node>> {
    self.parse_children(node, document)?;
    // This will only handle 1 layer nesting (successfully?)
    // Note that this would have been found by the top level xpath,
    // but we've got to worry about node identity: the parent is being rebuilt
    for nested in document.findnodes("descendant::ltx:XMath", Some(node)) {
      self.parse(nested, document)?;
    }
    let tag = get_node_qname(node);
    let rule = if let Some(requested_rule) = node.get_attribute("rule") {
      requested_rule
    } else {
      rule_opt.to_owned()
    };

    if rule == "kludge" {
      self.parse_kludge(node, document);
      Ok(None)
    } else if let Some(mut result) = self.parse_single(node, document, &rule)? {
      *self.passed.entry(tag).or_insert(0) += 1;
      if tag == arena::pin_static("ltx:XMath") {
        // Replace the content of XMath with parsed result
        self.n_parsed += 1;
        note_progress(&s!("[{}]", self.n_parsed));
        for el_node in element_nodes(node) {
          document.unrecord_node_ids(&el_node);
        }
        // unbindNode followed by (append|replace)Tree (which removes ID's) should
        // be safe
        for mut child in node.get_child_nodes() {
          child.unbind_node();
        }
        document.append_tree(node, vec![result])?;
        let mut new_element_children = element_nodes(node);
        result = new_element_children.remove(0);
      } else {
        // Replace the whole node for XMArg, XMWrap; preserve some attributes
        //ProgressStep() if ($$self{progress}++ % $MATHPARSE_PROGRESS_QUANTUM) == 0;
        // Copy all attributes
        let resultid = p_get_attribute(&result, "id");
        let mut attr = node.get_attributes();

        // add to result, even allowing modification of xml node, since we're committed.
        // [Annotate converts node to array which messes up clearing the id!]
        let rtag = get_node_qname(&result);
        // TODO: Is this needed in a world where `result` is always a `Node` ?
        // // // Make sure font is "Appropriate", if we're creating a new token (yuck)
        // // if ($isarr && $attr{_font} && ($rtag eq 'ltx:XMTok')) {
        // // my $content = join('', @$result[2 .. $#$result]);
        // // if ((!defined $content) || ($content eq '')) {
        // //   delete $attr{_font}; }    # No font needed
        // // elsif (my $font = $document->decodeFont($attr{_font})) {
        // //   delete $attr{_font};
        // //   $attr{font} = $font->specialize($content); } }
        // // } else {
        attr.remove("_font");
        // TODO: See the namespaced attribute issue for libxml's wrapper:
        //  https://github.com/KWARC/rust-libxml/issues/104
        // until then, HACK ids.
        let newid = attr.remove("id");
        if let Some(ref nid) = newid {
          attr.insert(String::from("xml:id"), nid.to_owned());
        }
        for (key, value) in attr {
          if !(key.starts_with('_')
            || sym_can_have_attribute(rtag, arena::pin(&key)))
          {
            continue;
          }
          if key == "xml:id" {
            // Since we're moving the id...bookkeeping
            document.unrecord_id(&value);
            node.remove_attribute("xml:id")?;
          }
          // TODO: is the array/XM case still relevant?
          // if ($isarr) { $$result[1]{$key} = $value; } else {
          document.set_attribute(&mut result, &key, &value)?;
          // }
        }
        result = document
          .replace_tree(result, node)?
          .expect("replacing the tree should always work.");
        // Danger: the above code replaced the id on the parsed result with the one from XMArg,..
        // If there are any references to `resultid`, we need to point them to `newid`!
        if let Some(rid) = resultid {
          if let Some(nid) = newid {
            if rid != nid {
              for mut tref in document.findnodes(&s!("//*[@idref='{}']", rid), None) {
                tref.set_attribute("idref", &nid)?;
              }
            }
          }
        }
      }
      Ok(Some(result))
    } else {
      // TODO:
      // self.parse_kludge(node, document);
      // ProgressStep() if ($$self{progress}++ % $MATHPARSE_PROGRESS_QUANTUM) == 0;
      // $$self{failed}{$tag}++;
      Ok(None)
    }
  }

  // Depth first parsing of XMArg nodes.
  fn parse_children(
    &mut self,
    node: &mut Node,
    document: &mut Document,
  ) -> Result<()> {
    for mut child in element_nodes(node) {
      let tag = get_node_qname(&child);
      if tag == arena::pin_static("ltx:XMArg") || tag == arena::pin_static("ltx:XMWrap") {
        self.parse_rec(&mut child, "Anything", document)?;
      } else if tag == arena::pin_static("ltx:XMApp")
        || tag == arena::pin_static("ltx:XMArray")
        || tag == arena::pin_static("ltx:XMRow")
        || tag == arena::pin_static("ltx:XMCell")
        || tag == arena::pin_static("ltx:XMDual")
      {
        self.parse_children(&mut child, document)?;
      }
    }
    Ok(())
  }

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Low-Level hack parsing when "real" parsing fails;
  // Two issues cause generated Presentation MathML to be really bad:
  // (1) not having mrow/mfenced structures wrapping OPEN...CLOSE sequences
  //     throws off MathML's stretchiness treatment of the fences
  //     (they're all the same size; big)
  // (2) un-attached sub/superscripts won't position correctly,
  //     unless they're attached to something plausible.
  // NOTE: we should be able to optionally switch this off.
  // Especially, when we want to try alternative parse strategies.
  fn parse_kludge(&self, _node: &mut Node, _document: &mut Document) {
    unimplemented!();
  }

  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Low-level Parser: parse a single expression
  //%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
  // Convert to textual form for processing by MathGrammar
  fn parse_single(
    &mut self,
    mathnode: &mut Node,
    document: &mut Document,
    _rule: &str,
  ) -> Result<Option<Node>> {
    let mut idx = 0;
    let mut content_nodes = filter_hints(mathnode.get_child_nodes());
    if content_nodes.is_empty() {
      Ok(None) // nothing to parse
    } else if content_nodes.len() == 1 {
      // single node, just return
      Ok(Some(content_nodes.remove(0)))
    } else {
      let (lexemes, mut nodes) = node_to_grammar_lexemes(mathnode, &mut idx);
      if let Ok(Some(parse_tree)) = self.parse_lexemes(lexemes, &nodes, document) {
        //START reparent: the reparenting used to be in `parse_rec` in Perl. Is this a good place?
        // Replace the content of XMath with parsed result
        // unbindNode followed by (append|replace)Tree (which removes ID's) should be safe
        for child_el in element_nodes(mathnode) {
          document.unrecord_node_ids(&child_el);
        }
        for mut node in mathnode.get_child_nodes() {
          node.unlink();
        }
        let new_xml_tree = parse_tree.into_xmath(mathnode, &mut nodes, document)?;
        document.append_tree(mathnode, vec![new_xml_tree])?;
        let result = element_nodes(mathnode).remove(0);
        //END reparent.
        Ok(Some(result))
      } else {
        Ok(None)
      }
    }
    //   # Failure? No result or uparsed lexemes remain.
    //   # NOTE: Should do script hack??
    //   if ((!defined $result) || $unparsed) {
    //     $self->failureReport($document, $mathnode, $rule, $unparsed, @nodes);
    //     return; }
    //   # Success!
    //   else {
    // if (@punct) {    # create a trivial XMDual to treat the punctuation as
    // presentation       $result = ['ltx:XMDual', {},
    //         LaTeXML::Package::createXMRefs($document, $result),
    // ['ltx:XMWrap', {}, $result, @punct]]; }    # or perhaps: Apply,
    // punctuated???     if ($LaTeXML::MathParser::DEBUG) {
    //       print STDERR "\n=>" . printNode($result) . "\n" . ('=' x 60) . "\n"; }
    //     return $result; } }
  }

  pub fn parse_marpa(
    &mut self,
    input: &str,
    nodes: &[Node],
    document: &mut Document,
  ) -> Result<XM> {
    let parse_result = self
      .engine
      .run_recognizer(ByteScanner::new(Cursor::new(input)))?;
    let mut parses = Vec::new();
    let mut ok_trees = 0;
    let mut pruned_trees = 0;
    for val in parse_result {
      match self.actions.get_tree(
        self.builder.clone(),
        val,
        self.expert_pragmatics.as_slice(),
        ActionContext { nodes, document },
      ) {
        Ok(tree_opt) => {
          if let Some(tree) = tree_opt {
            // eprintln!("-- we found a tree: {:?}", tree);
            ok_trees += 1;
            // ignore semantically pruned parses
            parses.push(tree);
          }
        },
        Err(_prune_err) => {
          pruned_trees += 1;
        }, // bookkeep the prune reasons?
      }
    }
    if ok_trees + pruned_trees > 100 {
      // let warning1 = format!(
      //   "WARNING! too many marpa trees: {:?}, accepted as semantic trees: {:?}",
      //   ok_trees + pruned_trees,
      //   ok_trees
      // );
      // let warning2 = format!("         on input: {:?}", input);
      // eprintln!("\n{}", Yellow.bold().paint(warning1));
      // eprintln!("{}\n", Yellow.paint(warning2));
    }

    match parses.len() {
      0 => Err("Failed to find any parse".into()),
      1 => Ok(parses.into_iter().next().unwrap()),
      2 | 3 => Ok(XM::Choices(parses)),
      _more => {
        // Loop over the various soft pruning algorithms available, until we are at 3 trees or less
        let mut reduced_forest = XM::Choices(parses);
        for pragma in self.student_pragmatics.iter() {
          reduced_forest = reduced_forest.soft_prune_choices(*pragma);
          match reduced_forest {
            XM::Choices(ref trees) => match trees.len() {
              2 | 3 => break, //reduced sufficiently, return
              _more => {},    // keep trying to reduce
            },
            _ => break, // reduced sufficiently, return
          };
        }
        Ok(reduced_forest)
      },
    }
  }

  pub fn parse_lexemes(
    &mut self,
    lexemes: Vec<String>,
    nodes: &[Node],
    document: &mut Document,
  ) -> Result<Option<XM>> {
    let mut input_string: String = lexemes.join(" ");
    // Add a trailing space, in an attempt to work with
    // a rules!() macro that has a Hard expectation of a space char following EVERY token.
    // this - counterintuitively- allows a simple macro definition AND a simple parse tree.
    input_string.push(' ');
    if let Ok(parse_tree) = self.parse_marpa(&input_string, nodes, document) {
      Ok(Some(parse_tree))
    } else {
      Ok(None)
    }
  }
}

//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// Conversion to a less ambiguous, mostly-prefix form.
// Mostly for debugging information?
// Note that the nodes are true libXML nodes, already absorbed into the document
//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
pub fn text_form(node: &Node, document: &mut Document) -> String {
  textrec(node, None, None, document)
}

// ================================================================================
// Some more XML utilities, but math specific (?)

// Get the Token's  meaning, else name, else content, else role
fn get_token_meaning(node_opt: &Node, document: &Document) -> Option<String> {
  let node = realize_xmnode(node_opt, document);
  if let Some(x) = p_get_attribute(&node, "meaning") {
    Some(x)
  } else if let Some(x) = p_get_attribute(&node, "name") {
    Some(x)
  } else {
    let text_content = node.get_content();
    if !text_content.is_empty() {
      Some(text_content)
    } else {
      p_get_attribute(&node, "role")
    }
  }
}

fn textrec(
  node_opt: &Node,
  outer_bp_opt: Option<usize>,
  outer_name_opt: Option<&str>,
  document: &Document,
) -> String {
  let node = realize_xmnode(node_opt, document);
  let tag = get_node_qname(&node);
  let outer_bp = outer_bp_opt.unwrap_or(0);
  let outer_name = outer_name_opt.unwrap_or("");
  // If node has meaning, that's the text form.
  let meaning_opt = match p_get_attribute(&node, "meaning") {
    Some(m) => Some(m),
    None => p_get_attribute(&node, "name"),
  };
  if let Some(meaning) = meaning_opt {
    return match PREFIX_ALIAS.get(meaning.as_str()) {
      Some(m) => m.to_string(),
      None => meaning,
    };
  }
  if tag == arena::pin_static("ltx:XMApp") {
    let mut args = element_nodes(&node);
    if args.is_empty() {
      // Error!("expected","arguments" ...);
      unimplemented!();
    }
    let arg_node = args.remove(0);
    let op = realize_xmnode(&arg_node, document);
    if let Some(app_role) = node.get_attribute("role") {
      if app_role == "FLOATSUBSCRIPT" {
        return String::from("_") + &textrec(&op, None, None, document);
      } else if app_role == "FLOATSUPERSCRIPT" {
        return String::from("^") + &textrec(&op, None, None, document);
      }
    }

    let name = if with_node_qname(&op, |name| name == "ltx:XMTok") {
      get_token_meaning(&op, document).unwrap_or_else(|| "unknown".to_owned())
    } else {
      String::new()
    };
    let (bp, string) = textrec_apply(&name, &op, args, document);
    if (bp < outer_bp) || ((bp == outer_bp) && (name != outer_name)) {
      format!("({string})")
    } else {
      string
    }
  } else if tag == arena::pin_static("ltx:XMDual") {
    let children = element_nodes(&node);
    let content = children
      .first()
      .expect("XMDual should always have 2 child elements.");
    textrec(content, Some(outer_bp), Some(outer_name), document) // Just send out the
                                                                        // semantic form
                                                                        // Fall back to
                                                                        // presentation, if
                                                                        // content has poor
                                                                        // semantics (eg. from
                                                                        // replacement patterns)
                                                                        // TODO
                                                                        // return ($text =~
                                                                        // /^\(*Unknown/ ?
                                                                        // textrec($presentation,
                                                                        // $outer_bp, $outer_name)
                                                                        // : $text); }
  } else if tag == arena::pin_static("ltx:XMTok") {
    let name = match get_token_meaning(&node, document) {
      Some(meaning) => meaning,
      None => s!("Unknown"),
    };
    match PREFIX_ALIAS.get(name.as_str()) {
      Some(v) => v.to_string(),
      None => name,
    }
  } else if tag == arena::pin_static("ltx:XMWrap") || tag == arena::pin_static("ltx:XMCell") {
    // ??
    element_nodes(&node)
      .into_iter()
      .map(|child| textrec(&child, None, None, document))
      .collect::<Vec<_>>()
      .join("@")
  } else if tag == arena::pin_static("ltx:XMArg") {
    let args = element_nodes(&node);
    if args.is_empty() {
      // Error!("expected","arguments" ...);
      unimplemented!();
    }
    args
      .iter()
      .map(|arg| textrec(arg, None, None, document))
      .collect::<Vec<_>>()
      .join("")
  } else if tag == arena::pin_static("ltx:XMArray") {
    textrec_array(&node)
  } else {
    s!("[{}]", p_get_value(&node))
  }
}

fn textrec_apply(
  name: &str,
  op: &Node,
  args: Vec<Node>,
  document: &Document,
) -> (usize, String) {
  let role = op
    .get_attribute("role")
    .unwrap_or_else(|| "Unknown".to_string());
  if (role == "SUBSCRIPTOP" || role == "SUPERSCRIPTOP")
    && PRE_DIGITS_RE.is_match(&op.get_attribute("scriptpos").unwrap_or_default())
  {
    // Note that this will likely get parenthesized due to high bp
    let mut inner = textrec(op, None, None, document);
    if let Some(arg2) = args.get(1) {
      inner.push(' ');
      inner.push_str(&textrec(arg2, None, None, document));
    }
    if let Some(arg1) = args.get(0) {
      inner.push(' ');
      inner.push_str(&textrec(arg1, None, None, document));
    }
    (5000, inner)
  } else if let Some(bp) = IS_INFIX.get(&role) {
    // A sub/superscript with a meaning probably should be prefix
    if (role == "SUBSCRIPTOP" || role == "SUPERSCRIPTOP") && op.has_attribute("meaning") {
      (
        500,
        format!(
          "{}@({})",
          textrec(op, Some(10000), Some(name), document),
          args
            .iter()
            .map(|a| textrec(a, None, None, document))
            .collect::<Vec<_>>()
            .join(", ")
        ),
      )
    } else {
      // Format as infix.
      let textrec_op = textrec(op, None, None, document);
      let rec_form = if args.len() == 1 {
        // unless a single arg; then prefix.
        textrec_op + " " + &textrec(&args[0], Some(*bp), Some(name), document)
      } else {
        args
          .iter()
          .map(|a| textrec(a, Some(*bp), Some(name), document))
          .collect::<Vec<_>>()
          .join(&(" ".to_string() + &textrec_op + " "))
      };
      (*bp, rec_form)
    }
  } else if role == "POSTFIX" {
    (
      10000,
      textrec(&args[0], Some(10000), Some(name), document)
        + &textrec(op, None, None, document),
    )
  } else if name == "multirelation" {
    let joined = args
      .iter()
      .map(|a| textrec(a, Some(2), Some(name), document))
      .collect::<Vec<_>>()
      .join(" ");
    (2, joined)
  } else {
    (
      500,
      textrec(op, Some(10000), Some(name), document)
        + "@("
        + &args
          .iter()
          .map(|a| textrec(a, None, None, document))
          .collect::<Vec<_>>()
          .join(", ")
        + ")",
    )
  }
}

fn textrec_array(_node: &Node) -> String {
  // my $name = $node->getAttribute('meaning') || $node->getAttribute('name')
  // || 'Array';   my @rows = ();
  //   foreach my $row (element_nodes($node)) {
  // push(@rows, '[' . join(', ', map { ($_->firstChild ?
  // textrec($_->firstChild) : '') } element_nodes($row)) . ']'); } return $name
  // . '[' . join(', ', @rows) . ']';
  unimplemented!()
}

//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// Cute! Were it NOT for Sub/Superscripts, the whole parsing process only
// builds a new superstructure around the sequence of token nodes in the input.
// Thus, any internal structure is unchanged.
//  They get re-parented, but if the parse fails, we've only got to put them
// BACK into the original node, to recover the original arrangment!!!
// Thus, we don't have to clone, and deal with namespace duplication.
// ...
// EXCEPT, as I said, for sub/superscripts!!!!
//

//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// Constructors used in grammar
// All the tree construction in the grammar should come through these
// operations. We avoid mucking with the actual XML nodes (both to avoid
// modifying the original tree until we have a successful parse, and to avoid
// XML::LibXML cloning nightmares) We are converting XML nodes to array
// representation: [$tag, {%attr},@children] This means any inspection of
// nodes has to recognize that  * node may be in XML vs ARRAY representation
// * node may be an XMRef to another node whose properties are the ones we
// should use.
//
// Also, when we are examining a node's properties (roles, fences, script
// positioning, etc) we should be careful to check for XMRef indirection and
// examine the properties of the node that was referred to.
// HOWEVER, we should construct our parse tree using (a clone of) the XMRef
// node, rather than (a clone of) the referred to node, so as to preserve
// identity.
//%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%
// We're currently keeping the id's on the nodes as they get cloned,
// since they'll (maybe) replace the unparsed nodes.
// However, if we consider multiple parses or preserving both parsed & unparsed,
// we may have to do some adaptation and id shifting.
// ================================================================================

// ================================================================================
// Low-level accessors

// The following accessors work on both the LibXML and ARRAY representations
// but they do NOT automatically dereference XMRef!
pub fn p_get_value(node: &Node) -> String {
  let node_type = node.get_type();
  if node_type == Some(NodeType::ElementNode) {
    let x = node.get_content();
    if !x.is_empty() {
      // get content, or fall back to name
      x
    } else {
      match node.get_attribute("name") {
        Some(name) => name,
        None => String::new(),
      }
    }
  } else {
    node.get_content()
  }
}

//================================================================================

pub fn realize_xmnode<'a>(node: &'a Node, document: &'a Document) -> Cow<'a, Node> {
  if with_node_qname(node, |name| name == "ltx:XMRef")
  {
    if let Some(idref) = node.get_attribute("idref") {
      // Can it happen that $realnode is, itself, an XMRef?
      // Then we should recurse recurse!
      if let Some(realnode) = document.lookup_id(&idref) {
        return Cow::Borrowed(realnode);
      } else {
        let message = s!("Cannot find a node with xml:id='{}'", idref);
        // TODO:
        // LaTeXML::MathParser::IDREFS{$idref}
        // ? "Previously bound to " .
        // ToString($LaTeXML::MathParser::IDREFS{$idref})           : ()));
        let err = || {Error!("expected", "id", message); Ok(()) };
        err().ok();
        //       return ['ltx:ERROR', {}, "Missing XMRef idref=$idref"]; } }
      }
    }
  }
  Cow::Borrowed(node)
}

fn p_get_attribute(item: &Node, key: &str) -> Option<String> {
  //   elsif (ref $item eq 'ARRAY') {
  //     return $$item[1]{$key}; }
  if item.get_type() == Some(NodeType::ElementNode) {
    item.get_attribute(key)
  } else {
    None
  }
}
