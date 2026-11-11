//! eptcs.cls (Electronic Proceedings in Theoretical Computer Science; v1.5 2010/11/26, v1.6 2020/12/20, v1.7 2022/05/20
//! — the same title page in each): the papers ship the class (it is not in TeX Live) and it is read raw, its packages
//! loaded as it loads them (`\LoadClass[letterpaper,11pt,twoside]{article}`, hyperref, mathptmx, eptcsdata.tex when
//! the paper has one). Perl has no binding.
//!
//! Its title page (eptcs.cls:160-191) sets `\@author` in a centred `tabular`, so `\and` starts a column of its own and
//! `\institute{…}` and `\email{…}`, written inside `\author` after the names, are each `\\` and a line under them:
//! `\author{Ann Able \institute{Univ A} \email{ann@a.edu} \and Bob Baker \institute{Univ B}}`. Read as the kernel's
//! author line, the institute and e-mail stayed in the name ("Ilaria De Crescenzo Università degli Studi di Salerno
//! Dipartimento di Informatica", 1408.5969; 1909.07676, 2406.16409, 2109.08295's "Jens Lehmann SDA Research Group, … \{grubenmann,
//! jens.lehmann\}@cs.uni-bonn.de"). The binding writes the block as the class prints it and gives it whole to the
//! reading (`\lx@add@author@split`, OXIDIZED_DESIGN_DIVERGENCES #984): each cell a column — cut where the block holds a
//! separator meaning the kernel's `\and`, written, held in a macro or a `\let` copy (the separator query, #995; P2-4,
//! OD #991 (8)), and followed by that separator's own tokens, which the reading cuts at — its names
//! (`Tobias Grubenmann and Jens Lehmann`, 2109.08295) and each `\institute` and `\email` a `\\` line under them; the
//! reading places each line — an institute every name's of the column, an e-mail the author it spells, else, a list of
//! one address per name in order, the name at its place (1101.4424's `g.stavrinos@… \qquad tassiana98@…`), else the
//! column's only author, else the block's (FIDELITY_CHARTER §4). A cell that opens with a `tabular` of its
//! own — the names as its columns, a row of e-mails under them, and lines under the table (1407.7928) — or with an
//! environment of the paper's own that opens one (2409.15014's `{authors}{5}`, its `\institute` and `\email` lines
//! spanning the columns) is read the same way: a column's names its cell, a full row's cell a line under its column,
//! any other line under the table every name's. A block with none of these is the kernel's.
use latexml_package::prelude::{
  tokens_environment_end as environment_end, tokens_read_argument as read_argument,
  tokens_unit_end as unit_end, *,
};

const EPTCS: crate::shipped_class::RawRead = crate::shipped_class::RawRead {
  shipped: true,
  ..crate::shipped_class::RawRead::new("eptcs", "cls")
};

LoadDefinitions!({
  if !crate::shipped_class::read_raw(&EPTCS)? {
    return Ok(());
  }
  let_i(&T_CS!("\\lx@eptcs@kernel@author"), &T_CS!("\\author"), None);
  DefMacro!("\\author[]{}", sub[(short, body)] {
    let Some(cells) = eptcs_structured(&body)? else {
      return Ok(Invocation!(T_CS!("\\lx@eptcs@kernel@author"), vec![short, Some(body)]));
    };
    let calls = eptcs_author_calls(cells)?;
    Ok(Invocation!(T_CS!("\\lx@eptcs@author"), vec![Some(body), Some(calls)]))
  });
  DefMacro!(
    "\\lx@eptcs@author{}{}",
    "\\def\\@author{#1}\\lx@clear@creators[role=author]#2"
  );
  // (outside an `\author`, after it: the last author's, as the class prints the line under the names before it)
  DefMacro!("\\institute{}", "\\lx@add@affiliation{#1}");
  // eptcs.cls:191 `\newcommand{\email}[1]{\\{\footnotesize\ttfamily #1}}`
  DefMacro!("\\email{}", "\\lx@add@email{\\lx@address@urlfont{#1}}");
});

/// The author separators of the class's `\@maketitle` by spelling, `\and` and its kernel aliases: the cut where the
/// separator query reads nothing ([`cells`]).
fn and_delims() -> Vec<SplitDelim> {
  vec![
    SplitDelim::Token(T_CS!("\\and")),
    SplitDelim::Token(T_CS!("\\And")),
    SplitDelim::Token(T_CS!("\\AND")),
  ]
}

/// A cell of the block: the separator's own tokens written before it (none for the first), and its tokens.
struct Cell {
  separator: Vec<Token>,
  tokens:    Vec<Token>,
}

/// The block's cells (its `\\[…]` options stripped), where it is the class's: an `\institute` or `\email` in it, or a
/// cell opening with a table of names; `None` for a block the kernel's author line reads.
fn eptcs_structured(body: &Tokens) -> Result<Option<Vec<Cell>>> {
  let calls = body
    .unlist_ref()
    .iter()
    .any(|t| *t == T_CS!("\\institute") || *t == T_CS!("\\email"));
  let cells = cells(&strip_linebreak_options(body.clone()))?;
  let structured = calls
    || cells
      .iter()
      .any(|cell| leading_table(&cell.tokens).is_some());
  Ok(structured.then_some(cells))
}

/// The block's cells, cut where it holds a separator meaning the kernel's `\and` — eptcs.cls's column break (its
/// `\@maketitle` sets `\@author` in a `tabular[t]{c}`, :170-174, which latex.ltx's `\and` closes, opening the next):
/// written, held in a macro of the document's, or a `\let` copy, as the separator query reads them by meaning
/// (`argument_separators`, OXIDIZED_DESIGN_DIVERGENCES #995; a separator of another meaning — a class's or the
/// document's renewed `\and`, amsmath's `\And` — prints no column, trap T8). A separator is cut only where its own tokens
/// are whole units of the block's top level (no group or environment holds them, [`unit_end`]) and print nothing else;
/// one a macro holds together with names stays in its cell, which the reading cuts at its `Sep(And)` (#993): the runs of
/// `kernel_separator_runs`, the cut the institute lists of inst_support and aa share (P2-2). Where the
/// query reads nothing (`None`: inside a rehearsal, over its allowance), the cut is by spelling, at `\and`, `\And`,
/// `\AND` and an alias of `\and`'s own definition (`split_tokens`), as before P2-4. The spaces at a cell's ends are its
/// readers' to drop ([`leading_table`], [`cell_items`]).
fn cells(body: &Tokens) -> Result<Vec<Cell>> {
  let Some(runs) = kernel_separator_runs(body)? else {
    return Ok(
      split_tokens_delimited(body.clone(), and_delims())
        .into_iter()
        .map(|(separator, tokens)| Cell {
          separator,
          tokens: tokens.unlist(),
        })
        .collect(),
    );
  };
  let tokens = body.unlist_ref();
  let mut cells = Vec::new();
  let mut separator = Vec::new();
  let mut from = 0;
  for (start, end) in runs {
    cells.push(Cell {
      separator: std::mem::replace(&mut separator, tokens[start..end].to_vec()),
      tokens:    tokens[from..start].to_vec(),
    });
    from = end;
  }
  cells.push(Cell {
    separator,
    tokens: tokens[from..].to_vec(),
  });
  Ok(cells)
}

/// The block as the class prints it, for the reading: each cell a column of names and the lines under them (an
/// `\institute` or `\email` call is a `\\` line of its own, eptcs.cls:188-190 `\newcommand{\institute}[1]{…\\…}`,
/// :191 `\newcommand{\email}[1]{\\…}`), each column after the separator written before it, and the lines under every
/// name of a table, each a `_share=authors` entry.
#[derive(Default)]
struct Block {
  /// each column: the separator written before it, its names and lines
  columns:   Vec<(Vec<Token>, Vec<Token>)>,
  shared:    Vec<Vec<Token>>,
  /// the separator of the cell being read, for the column it opens
  separator: Vec<Token>,
}

impl Block {
  /// The cell next read follows `separator` (none for the first).
  fn follows(&mut self, separator: Vec<Token>) { self.separator = separator; }

  /// A new column holding `names`, after the cell's separator, or a column of a table after another column of it
  /// (a table's own `&` columns, which the binding reads as names: `\and`); its place.
  fn add_names(&mut self, names: Vec<Token>) -> usize {
    let separator = match std::mem::take(&mut self.separator) {
      separator if separator.is_empty() => vec![T_CS!("\\and")],
      separator => separator,
    };
    self.columns.push((separator, names));
    self.columns.len() - 1
  }

  /// A line under the names of column `at`.
  fn line_under(&mut self, at: usize, line: Vec<Token>) {
    let column = &mut self.columns[at].1;
    column.push(T_CS!("\\\\"));
    column.extend(line);
  }

  /// The block's whole list to the reading (`\lx@add@author@split`, as llncs's `\author`, OD #984), which cuts it at its
  /// separators' `Sep(And)` (#993): each column after the separator its cell follows, as written (a literal `\and`, a
  /// macro holding it, a `\let` copy), or a table's `\and`; then each line every name shares, an affiliation (e-mail
  /// calls in it are lines of the row, placed by the reading: the address the author it spells, a list in order, else
  /// the block's — FIDELITY_CHARTER §4).
  fn calls(self) -> Result<Tokens> {
    let mut list = Vec::new();
    for (at, (separator, column)) in self.columns.into_iter().enumerate() {
      if at > 0 {
        list.push(T_SPACE!());
        list.extend(separator);
        list.push(T_SPACE!());
      }
      list.extend(column);
    }
    let mut out = Invocation!(T_CS!("\\lx@add@author@split"), vec![
      None,
      Some(Tokens::new(list))
    ])
    .unlist();
    for line in self.shared {
      let options = mouth::tokenize_internal("labelseq={affiliation},_share={authors}");
      out.extend(
        Invocation!(T_CS!("\\lx@add@affiliation"), vec![
          Some(options),
          Some(Tokens::new(line))
        ])
        .unlist(),
      );
    }
    Ok(Tokens::new(out))
  }
}

/// A call the class prints as a line of its own (`\institute{…}`, `\email{…}`), written as it is called.
fn call_line(cs: Token, argument: Vec<Token>) -> Vec<Token> {
  let mut line = vec![cs, T_BEGIN!()];
  line.extend(argument);
  line.push(T_END!());
  line
}

/// An item of a cell: a line of text (after a `\\` or not), an `\institute`'s argument or an `\email`'s.
enum Item {
  Text {
    after_break: bool,
    line:        Vec<Token>,
  },
  Institute(Vec<Token>),
  Email(Vec<Token>),
}

/// The calls the block makes: each cell ([`cells`]) read in turn.
fn eptcs_author_calls(cells: Vec<Cell>) -> Result<Tokens> {
  let mut block = Block::default();
  for Cell { separator, tokens } in cells {
    block.follows(separator);
    match leading_table(&tokens) {
      Some(table) => read_table(&mut block, table),
      None => read_cell(&mut block, cell_items(&tokens)),
    }
  }
  block.calls()
}

/// A cell's items in order (an environment in it, and a group, one unit of its text).
fn cell_items(cell: &[Token]) -> Vec<Item> {
  let mut items = Vec::new();
  let mut line: Vec<Token> = Vec::new();
  let mut after_break = false;
  let push_line = |items: &mut Vec<Item>, line: &mut Vec<Token>, after_break: bool| {
    let taken = std::mem::take(line);
    if !is_blank(&taken) {
      items.push(Item::Text {
        after_break,
        line: trim_spaces(taken),
      });
    }
  };
  let mut at = 0;
  while at < cell.len() {
    let t = cell[at];
    if t == T_CS!("\\\\") {
      push_line(&mut items, &mut line, after_break);
      after_break = true;
      at += 1;
    } else if t == T_CS!("\\institute") || t == T_CS!("\\email") {
      push_line(&mut items, &mut line, after_break);
      let (argument, next) = read_argument(cell, at + 1);
      items.push(if t == T_CS!("\\institute") {
        Item::Institute(argument)
      } else {
        Item::Email(argument)
      });
      after_break = true;
      at = next;
    } else {
      let next = unit_end(cell, at);
      line.extend_from_slice(&cell[at..next]);
      at = next;
    }
  }
  push_line(&mut items, &mut line, after_break);
  items
}

/// A cell read as the class prints it: a column, its first line the names and each `\institute`, `\email` and `\\`
/// line a line under them, each decided by the reading (an e-mail list, an affiliation, else a row of names of its own).
fn read_cell(block: &mut Block, items: Vec<Item>) {
  let mut column: Option<usize> = None;
  for item in items {
    let line = match item {
      Item::Institute(argument) => call_line(T_CS!("\\institute"), argument),
      Item::Email(argument) => call_line(T_CS!("\\email"), argument),
      Item::Text { line, .. } => line,
    };
    match column {
      Some(at) => block.line_under(at, line),
      None => column = Some(block.add_names(line)),
    }
  }
}

/// A table opening a cell: its rows, and what follows its end in the cell.
struct Table {
  rows: Vec<Row>,
  tail: Vec<Token>,
}

/// A row of a table: its cells (with how many columns each spans), or an `\institute`/`\email` line across it.
enum Row {
  Cells(Vec<(Vec<Token>, usize)>),
  Institute(Vec<Token>),
  Email(Vec<Token>),
}

/// The environments that set their content as a table: the tabular family, and an environment of the document's own
/// whose begin code opens one (2409.15014's `\newenvironment{authors}[1]{…\begin{tabular}[t]{…}}`), with the number
/// of arguments each takes after its name (an optional one first, if any, is read as one).
fn table_arguments(name: &str) -> Option<usize> {
  match name {
    "tabular" | "array" => Some(1),
    "tabular*" | "tabularx" | "tabulary" => Some(2),
    _ => {
      let definition = lookup_definition(&T_CS!(s!("\\{name}"))).ok().flatten()?;
      let Some(ExpansionBody::Tokens(begin)) = definition.get_expansion() else {
        return None;
      };
      let text = begin.to_string();
      (text.contains("{tabular}") || text.contains("{tabular*}")).then(|| {
        definition
          .get_parameters()
          .map_or(0, |p| p.get_parameters().len())
      })
    },
  }
}

/// The table a cell opens with (after spaces), when it is one ([`table_arguments`]).
fn leading_table(cell: &[Token]) -> Option<Table> {
  let start = cell
    .iter()
    .take_while(|t| t.get_catcode() == Catcode::SPACE)
    .count();
  if cell.get(start) != Some(&T_CS!("\\begin")) {
    return None;
  }
  let (name_tokens, mut at) = read_argument(cell, start + 1);
  let name = Tokens::new(name_tokens).to_string();
  let nargs = table_arguments(&name)?;
  let end = environment_end(cell, start, &name)?;
  // (its arguments: a position, `[t]`, then the given number of groups)
  at += cell[at..]
    .iter()
    .take_while(|t| t.get_catcode() == Catcode::SPACE)
    .count();
  if cell.get(at) == Some(&T_OTHER!("[")) {
    at = cell[at..end.0]
      .iter()
      .position(|t| *t == T_OTHER!("]"))
      .map_or(at, |close| at + close + 1);
  }
  for _ in 0..nargs {
    at = read_argument(cell, at).1;
  }
  let body = &cell[at.min(end.0)..end.0];
  Some(Table {
    rows: table_rows(body),
    tail: cell[end.1..].to_vec(),
  })
}

/// A table's rows: split at its `\\`, each `\institute`/`\email` a row of its own, each row's cells at its `&`.
fn table_rows(body: &[Token]) -> Vec<Row> {
  let mut rows = Vec::new();
  let mut line: Vec<Token> = Vec::new();
  let push_cells = |rows: &mut Vec<Row>, line: &mut Vec<Token>| {
    let taken = std::mem::take(line);
    let cells: Vec<(Vec<Token>, usize)> =
      split_tokens(Tokens::new(taken), vec![SplitDelim::Token(T_ALIGN!())])
        .into_iter()
        .map(|cell| multicolumn(cell.unlist()))
        .collect();
    if cells.iter().any(|(cell, _)| !is_blank(cell)) {
      rows.push(Row::Cells(cells));
    }
  };
  let mut at = 0;
  while at < body.len() {
    let t = body[at];
    if t == T_CS!("\\\\") {
      push_cells(&mut rows, &mut line);
      at += 1;
    } else if t == T_CS!("\\institute") || t == T_CS!("\\email") {
      push_cells(&mut rows, &mut line);
      let (argument, next) = read_argument(body, at + 1);
      rows.push(if t == T_CS!("\\institute") {
        Row::Institute(argument)
      } else {
        Row::Email(argument)
      });
      at = next;
    } else {
      let next = unit_end(body, at);
      line.extend_from_slice(&body[at..next]);
      at = next;
    }
  }
  push_cells(&mut rows, &mut line);
  rows
}

/// A cell's content and how many columns it spans: a `\multicolumn{n}{spec}{content}`'s, else the cell itself.
fn multicolumn(cell: Vec<Token>) -> (Vec<Token>, usize) {
  let start = cell
    .iter()
    .take_while(|t| t.get_catcode() == Catcode::SPACE)
    .count();
  if cell.get(start) == Some(&T_CS!("\\multicolumn")) {
    let (span, at) = read_argument(&cell, start + 1);
    let (_, at) = read_argument(&cell, at);
    let (content, _) = read_argument(&cell, at);
    let span = Tokens::new(span).to_string().trim().parse().unwrap_or(1);
    return (content, span);
  }
  (cell, 1)
}

/// A table of names read as the class prints it: its first row the names, a cell of each column its names; a full row
/// under them a line under each column, unless it reads as names, a row of names of its own; any other row a line
/// under every name of the table, a `_share=authors` entry; and after the table, what stands beside it the last name's
/// (2409.15014's `\thanks` after `\end{authors}`, its mark printed after "Verena Wolf") and each line under it every
/// name's (1407.7928's institute lines).
fn read_table(block: &mut Block, table: Table) {
  let mut columns: Vec<usize> = Vec::new();
  for row in table.rows {
    match row {
      Row::Institute(line) => block.shared.push(call_line(T_CS!("\\institute"), line)),
      Row::Email(line) => block.shared.push(call_line(T_CS!("\\email"), line)),
      Row::Cells(cells) => {
        let full = !columns.is_empty()
          && cells.len() == columns.len()
          && cells.iter().all(|(_, span)| *span == 1);
        let names_row = columns.is_empty()
          || (full
            && cells.iter().all(|(cell, _)| {
              let tokens = Tokens::new(cell.clone());
              is_blank(cell) || !(line_is_email_list(&tokens) || reads_as_affiliation(&tokens))
            }));
        if names_row && (columns.is_empty() || full) {
          columns = cells
            .into_iter()
            .map(|(cell, _)| block.add_names(trim_spaces(cell)))
            .collect();
        } else if full {
          for ((cell, _), &column) in cells.into_iter().zip(&columns) {
            if !is_blank(&cell) {
              block.line_under(column, trim_spaces(cell));
            }
          }
        } else {
          let mut line: Vec<Token> = Vec::new();
          for (cell, _) in cells {
            if !is_blank(&cell) {
              if !line.is_empty() {
                line.push(T_SPACE!());
              }
              line.extend(trim_spaces(cell));
            }
          }
          block.shared.push(line);
        }
      },
    }
  }
  for item in cell_items(&table.tail) {
    match item {
      Item::Institute(line) => block.shared.push(call_line(T_CS!("\\institute"), line)),
      Item::Email(line) => block.shared.push(call_line(T_CS!("\\email"), line)),
      Item::Text { after_break: false, line } => {
        if let Some((_, last)) = block.columns.last_mut() {
          last.extend(line);
        }
      },
      Item::Text { line, .. } => block.shared.push(line),
    }
  }
}

/// The spacing a cell's line may hold beside its text: no text of its own.
fn is_spacing(t: &Token) -> bool {
  t.get_catcode() == Catcode::SPACE
    || [
      "\\smallskip",
      "\\medskip",
      "\\bigskip",
      "\\hline",
      "\\par",
      "\\centering",
      "\\noindent",
      "\\relax",
    ]
    .iter()
    .any(|cs| *t == T_CS!(*cs))
}

/// Whether a line prints nothing: spacing alone.
fn is_blank(line: &[Token]) -> bool { line.iter().all(is_spacing) }

/// A line without the spacing at its ends.
fn trim_spaces(line: Vec<Token>) -> Vec<Token> {
  let start = line.iter().take_while(|t| is_spacing(t)).count();
  let end = line.len()
    - line[start..]
      .iter()
      .rev()
      .take_while(|t| is_spacing(t))
      .count();
  line[start..end].to_vec()
}
