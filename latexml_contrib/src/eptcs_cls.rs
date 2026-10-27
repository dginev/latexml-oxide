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
//! jens.lehmann\}@cs.uni-bonn.de"). The binding reads the block as the class prints it: each `\and` cell a row of names
//! (`Tobias Grubenmann and Jens Lehmann`, 2109.08295), every institute under the row each name's affiliation, and an
//! e-mail the author it spells, else, a list of one address per name in order, the name at its place, else the row's
//! only author, else the block's (FIDELITY_CHARTER §4; `row_email_owners`). A cell that opens with a `tabular` of its
//! own — the names as its columns, a row of e-mails under them, and lines under the table (1407.7928) — or with an
//! environment of the paper's own that opens one (2409.15014's `{authors}{5}`, its `\institute` and `\email` lines
//! spanning the columns) is read the same way: a column's names its cell, a full row's cell a line under its column,
//! any other line under the table every name's. A block with none of these is the kernel's.
use latexml_package::prelude::*;

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
    if !eptcs_structured(&body) {
      return Ok(Invocation!(T_CS!("\\lx@eptcs@kernel@author"), vec![short, Some(body)]));
    }
    let calls = eptcs_author_calls(body.clone())?;
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

/// The author separators of the class's `\@maketitle`, `\and` and its kernel aliases.
fn and_delims() -> Vec<SplitDelim> {
  vec![
    SplitDelim::Token(T_CS!("\\and")),
    SplitDelim::Token(T_CS!("\\And")),
    SplitDelim::Token(T_CS!("\\AND")),
  ]
}

/// Whether the block is the class's: an `\institute` or `\email` in it, or a cell opening with a table of names.
fn eptcs_structured(body: &Tokens) -> bool {
  body
    .unlist_ref()
    .iter()
    .any(|t| *t == T_CS!("\\institute") || *t == T_CS!("\\email"))
    || split_tokens(body.clone(), and_delims())
      .iter()
      .any(|cell| leading_table(cell.unlist_ref()).is_some())
}

/// One author: the name, and the contact calls digested with it.
struct Author {
  name:     Vec<Token>,
  contacts: Vec<Token>,
}

/// The authors of the block, in order, and the calls of the contacts the block shares.
#[derive(Default)]
struct Block {
  authors: Vec<Author>,
  shared:  Vec<Token>,
}

impl Block {
  /// The names of a names line, each a new author; their places.
  fn add_names(&mut self, line: Tokens) -> Vec<usize> {
    let mut row = Vec::new();
    for name in split_author_line(line) {
      row.push(self.authors.len());
      self.authors.push(Author {
        name:     name.unlist(),
        contacts: Vec::new(),
      });
    }
    row
  }

  /// An affiliation under the names at `row`: each name's.
  fn affiliation(&mut self, row: &[usize], line: Vec<Token>) -> Result<()> {
    // (one before any name is the block's)
    if row.is_empty() {
      let options = mouth::tokenize_internal("label=addresses:block");
      self.shared.extend(
        Invocation!(T_CS!("\\lx@add@affiliation"), vec![
          Some(options),
          Some(Tokens::new(line))
        ])
        .unlist(),
      );
      return Ok(());
    }
    let call = Invocation!(T_CS!("\\lx@add@affiliation"), vec![
      None,
      Some(Tokens::new(line))
    ])
    .unlist();
    for &at in row {
      self.authors[at].contacts.extend(call.iter().copied());
    }
    Ok(())
  }

  /// An e-mail line under the names at `row`: the row's only author's; else each address the name it spells, or the
  /// name at its place in a list in their order, else the block's.
  fn email(&mut self, row: &[usize], line: Vec<Token>) -> Result<()> {
    let call = |address: Tokens, block: bool| -> Result<Vec<Token>> {
      let address = Invocation!(T_CS!("\\lx@address@urlfont"), vec![Some(address)]);
      let options = block.then(|| mouth::tokenize_internal("label=addresses:block"));
      Ok(Invocation!(T_CS!("\\lx@add@email"), vec![options, Some(address)]).unlist())
    };
    let line = Tokens::new(line);
    if let [only] = row {
      self.authors[*only].contacts.extend(call(line, false)?);
      return Ok(());
    }
    let names: Vec<Tokens> = row
      .iter()
      .map(|&at| Tokens::new(self.authors[at].name.clone()))
      .collect();
    match row_email_owners(&names, &line)? {
      Some(addresses) => {
        for (address, owner) in addresses {
          match owner {
            Some(place) => self.authors[row[place]]
              .contacts
              .extend(call(address, false)?),
            None => self.shared.extend(call(address, true)?),
          }
        }
      },
      None => self.shared.extend(call(line, true)?),
    }
    Ok(())
  }

  /// The `\lx@add@author` calls, then the shared contacts.
  fn calls(self) -> Result<Tokens> {
    let mut out = Vec::new();
    for author in self.authors {
      let mut content = author.name;
      content.extend(author.contacts);
      out.extend(
        Invocation!(T_CS!("\\lx@add@author"), vec![
          None,
          Some(Tokens::new(content))
        ])
        .unlist(),
      );
    }
    out.extend(self.shared);
    Ok(Tokens::new(out))
  }
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

/// The calls the block makes: each `\and` cell read in turn.
fn eptcs_author_calls(body: Tokens) -> Result<Tokens> {
  let mut block = Block::default();
  for cell in split_tokens(strip_linebreak_options(body), and_delims()) {
    let cell = cell.unlist();
    match leading_table(&cell) {
      Some(table) => read_table(&mut block, table)?,
      None => read_cell(&mut block, cell_items(&cell))?,
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

/// A cell read as the class prints it: its first line the names, an `\institute` and an `\email` lines under them, a
/// `\\` line under them an e-mail when it is a list of addresses, an affiliation when it reads as one, else names of a
/// row of their own (as the llncs binding reads its author paragraph).
fn read_cell(block: &mut Block, items: Vec<Item>) -> Result<()> {
  let mut row: Vec<usize> = Vec::new();
  // (whether a line under the names came since they did, so that the next names start a new row)
  let mut under = false;
  for item in items {
    match item {
      Item::Institute(line) => {
        block.affiliation(&row, line)?;
        under = true;
      },
      Item::Email(line) => {
        block.email(&row, line)?;
        under = true;
      },
      Item::Text { after_break, line } => {
        let tokens = Tokens::new(line.clone());
        if after_break && !row.is_empty() && line_is_email_list(&tokens) {
          block.email(&row, line)?;
          under = true;
        } else if after_break && !row.is_empty() && reads_as_affiliation(&tokens) {
          block.affiliation(&row, line)?;
          under = true;
        } else {
          let names = block.add_names(tokens);
          if under || row.is_empty() {
            row = names;
            under = false;
          } else {
            row.extend(names);
          }
        }
      },
    }
  }
  Ok(())
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

/// Where the environment begun at `start` ends: the place of its `\end` and the place after `\end{name}`.
fn environment_end(cell: &[Token], start: usize, name: &str) -> Option<(usize, usize)> {
  let mut depth = 0usize;
  let mut at = start;
  while at < cell.len() {
    if cell[at] == T_CS!("\\begin") || cell[at] == T_CS!("\\end") {
      let (argument, next) = read_argument(cell, at + 1);
      if Tokens::new(argument).to_string() == name {
        if cell[at] == T_CS!("\\begin") {
          depth += 1;
        } else {
          depth -= 1;
          if depth == 0 {
            return Some((at, next));
          }
        }
      }
      at = next;
    } else {
      at += 1;
    }
  }
  None
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
/// under every name of the table; and after the table, what stands beside it the last name's (2409.15014's `\thanks`
/// after `\end{authors}`, its mark printed after "Verena Wolf") and each line under it every name's.
fn read_table(block: &mut Block, table: Table) -> Result<()> {
  let mut columns: Vec<Vec<usize>> = Vec::new();
  let mut all: Vec<usize> = Vec::new();
  let line_under = |block: &mut Block, names: &[usize], line: Vec<Token>| -> Result<()> {
    if line_is_email_list(&Tokens::new(line.clone())) {
      block.email(names, line)
    } else {
      block.affiliation(names, line)
    }
  };
  for row in table.rows {
    match row {
      Row::Institute(line) => block.affiliation(&all.clone(), line)?,
      Row::Email(line) => block.email(&all.clone(), line)?,
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
            .map(|(cell, _)| block.add_names(Tokens::new(trim_spaces(cell))))
            .collect();
          all.extend(columns.iter().flatten().copied());
        } else if full {
          for ((cell, _), column) in cells.into_iter().zip(columns.clone()) {
            if !is_blank(&cell) {
              line_under(block, &column, trim_spaces(cell))?;
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
          line_under(block, &all.clone(), line)?;
        }
      },
    }
  }
  for item in cell_items(&table.tail) {
    match item {
      Item::Institute(line) => block.affiliation(&all.clone(), line)?,
      Item::Email(line) => block.email(&all.clone(), line)?,
      Item::Text { after_break: false, line } => {
        if let Some(&last) = all.last() {
          block.authors[last].name.extend(line);
        }
      },
      Item::Text { line, .. } => line_under(block, &all.clone(), line)?,
    }
  }
  Ok(())
}

/// Where the unit at `at` ends: a group, an environment (`\begin{x}…\end{x}`), else the one token.
fn unit_end(tokens: &[Token], at: usize) -> usize {
  let t = tokens[at];
  if t.get_catcode() == Catcode::BEGIN {
    return read_argument(tokens, at).1;
  }
  if t == T_CS!("\\begin") {
    let (name, _) = read_argument(tokens, at + 1);
    if let Some((_, end)) = environment_end(tokens, at, &Tokens::new(name).to_string()) {
      return end;
    }
  }
  at + 1
}

/// The argument of a macro at `at`: a brace group's content, else the one token there; and where it ends.
fn read_argument(tokens: &[Token], at: usize) -> (Vec<Token>, usize) {
  let mut start = at;
  while start < tokens.len() && tokens[start].get_catcode() == Catcode::SPACE {
    start += 1;
  }
  if start >= tokens.len() {
    return (Vec::new(), start);
  }
  if tokens[start].get_catcode() != Catcode::BEGIN {
    return (vec![tokens[start]], start + 1);
  }
  let mut depth = 0usize;
  for (end, t) in tokens.iter().enumerate().skip(start) {
    match t.get_catcode() {
      Catcode::BEGIN => depth += 1,
      Catcode::END => {
        depth -= 1;
        if depth == 0 {
          return (tokens[start + 1..end].to_vec(), end + 1);
        }
      },
      _ => {},
    }
  }
  (tokens[start + 1..].to_vec(), tokens.len())
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
