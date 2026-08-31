//! LuaTeX-escape bridge: evaluate `\directlua` bodies in a persistent
//! external `texlua` interpreter (user directive 2026-08-31: "we can assume a
//! lua interpreter is available on devices with texlive installed").
//!
//! First-principles semantics (LuaTeX manual §2.1, §10.3): `\directlua
//! {<general text>}` runs the (fully expanded) body as a Lua chunk in the
//! engine's Lua state — one state for the whole job, so globals persist
//! across calls — and everything the chunk emits through `tex.print` /
//! `tex.sprint` / `tex.write` is inserted into the TeX input stream and
//! read with CURRENT catcodes (`tex.print` as full lines, `tex.sprint` as
//! a partial line).
//!
//! Strategy: **rebind-as-we-emulate** (docs/perfect_kernel/LUA_REBINDING.md).
//! texlua is Lua + TeX's support libraries (kpse, lpeg, md5, pdfe, …) MINUS
//! the engine — its built-in `tex` table holds only run/initialize/finish —
//! so every `tex.*` touchpoint an author's chunk uses is necessarily OUR
//! shim, and the design question per API is only how deep the shim goes:
//! * **translate** — emissions re-enter our pipeline (`tex.print`/`sprint`
//!   retokenized with current catcodes);
//! * **mirror** — state our engine genuinely has is served LIVE over the
//!   pipe (`tex.count`/`tex.dimen` reads AND writes round-trip to the Rust
//!   State via framed `Q`/`A` messages, `service_query`);
//! * **absorb** — intents with no XML-output meaning are accepted and
//!   dropped (`pdf.set*`, `tex.primitives()`); the node/font/callback layers
//!   (`node.*`, `font.*`, `callback.*` — LuaTeX typesetter internals) stay
//!   out of scope and error visibly rather than pretend.
//!
//! `require` is rebound through texlua's built-in kpse (plus lualibs, whose
//! `file`/`string` extensions lualatex gets via luaotfload), so texmf-shipped
//! Lua modules (newpax, tkz-elements, …) resolve.
//!
//! Process shape: one `texlua` child per conversion thread, spawned on first
//! use with a prelude that loops over length-framed requests on stdin and
//! answers `OK`/`ERR` length-framed responses on stdout. A std
//! `thread_local!` (destructor RUNS at thread exit, unlike `#[thread_local]`)
//! drops the pipes so the child's `io.read` returns nil and it exits.

use std::{
  cell::RefCell,
  io::{BufRead, BufReader, Write},
  process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

/// The Lua-side server loop. `tex.print(...)` arguments follow the LuaTeX
/// convention: an optional leading catcode-table number (ignored here — the
/// text is retokenized with current catcodes on the TeX side either way),
/// then strings / numbers / arrays of strings.
const LUA_PRELUDE: &str = r#"
local buf = {}
local function emit(sep, ...)
  local args = {...}
  local start = 1
  if type(args[1]) == 'number' and #args > 1 then start = 2 end
  for i = start, #args do
    local v = args[i]
    if type(v) == 'table' then
      for _, s in ipairs(v) do buf[#buf+1] = tostring(s) .. sep end
    else
      buf[#buf+1] = tostring(v) .. sep
    end
  end
end
tex = tex or {}
tex.print  = function(...) emit('\n', ...) end
tex.sprint = function(...) emit('', ...) end
tex.write  = tex.sprint
tex.tprint = function(tbl) for _, v in ipairs(tbl) do emit('', table.unpack(v)) end end
-- Live register mirror (rebind-as-we-emulate): texlua has no engine, so
-- tex.count/tex.dimen access round-trips to the Rust State over the pipe as
-- a framed query ('Q <len>' out, 'A <len>' back). Values follow the LuaTeX
-- convention: counts are integers, dimens are scaled points.
local function query(payload)
  io.write(string.format('Q %d\n', #payload))
  io.write(payload)
  io.flush()
  local header = io.read('*l')
  local n = tonumber(header and header:match('%d+')) or 0
  return n > 0 and io.read(n) or ''
end
local function regtable(kind)
  return setmetatable({}, {
    __index = function(_, k)
      return tonumber(query('get' .. kind .. ' ' .. tostring(k))) or 0
    end,
    __newindex = function(_, k, v)
      query('set' .. kind .. ' ' .. tostring(k) .. ' ' .. tostring(math.floor(v)))
    end})
end
local function regsetter(kind)
  return function(a, b, c)
    if a == 'global' then
      query('set' .. kind .. ' global ' .. tostring(b) .. ' ' .. tostring(math.floor(c)))
    else
      query('set' .. kind .. ' ' .. tostring(a) .. ' ' .. tostring(math.floor(b)))
    end
  end
end
tex.count = regtable('count')
tex.dimen = regtable('dimen')
tex.getcount = function(k) return tex.count[k] end
tex.getdimen = function(k) return tex.dimen[k] end
tex.setcount = regsetter('count')
tex.setdimen = regsetter('dimen')
-- Absorb shims: engine intents with no XML-output meaning are accepted and
-- dropped (witnesses in tests::rebound_engine_intents_absorb_and_resolve).
tex.primitives = function() return {} end
tex.error = function(msg) buf[#buf+1] = '' end
pdf = setmetatable({}, {
  __index = function(_, k)
    if type(k) == 'string' and k:find('^get') then return function() return 0 end end
    return function() end
  end})
texio = { write = function() end, write_nl = function() end }
status = setmetatable({}, {__index = function() return 0 end})
token = token or {}
-- Rebind require through texlua's built-in kpse so texmf-shipped Lua modules
-- resolve (newpax, tkz-elements, ...), and load lualibs, which supplies the
-- 'file'/'string'/'table' library extensions lualatex gets via luaotfload.
if kpse and kpse.set_program_name then
  pcall(kpse.set_program_name, 'luatex')
  table.insert(package.searchers or package.loaders, function(name)
    local f = kpse.find_file(name, 'lua')
    if f then return loadfile(f), f end
    return "\n\tno texmf module '" .. tostring(name) .. "' (kpse)"
  end)
  pcall(require, 'lualibs')
end
while true do
  local header = io.read('*l')
  if not header then os.exit(0) end
  local n = tonumber(header)
  if not n then os.exit(1) end
  local chunk = io.read(n)
  io.read('*l')
  buf = {}
  local out, tag
  local f, err = load(chunk)
  if f then
    local ok, rterr = pcall(f)
    if ok then
      out, tag = table.concat(buf), 'OK'
    else
      out, tag = tostring(rterr), 'ERR'
    end
  else
    out, tag = tostring(err), 'ERR'
  end
  io.write(string.format('%s %d\n', tag, #out))
  io.write(out)
  io.flush()
end
"#;

struct LuaProc {
  // Held so the child is reaped at thread exit (Drop closes stdin → the
  // prelude's io.read returns nil → texlua exits).
  child:  Child,
  stdin:  ChildStdin,
  stdout: BufReader<ChildStdout>,
}

thread_local! {
  /// `None` = not yet tried; `Some(None)` = tried and unavailable (warn once);
  /// `Some(Some(proc))` = live interpreter.
  static LUA: RefCell<Option<Option<LuaProc>>> = const { RefCell::new(None) };
}

impl Drop for LuaProc {
  fn drop(&mut self) {
    // Best-effort: closing stdin (dropped with the struct) ends the loop;
    // wait() reaps so no zombie outlives the thread.
    let _ = self.child.kill();
    let _ = self.child.wait();
  }
}

fn spawn() -> Option<LuaProc> {
  // `texlua` takes a script FILE (its `-e` is not lua's inline-chunk flag) —
  // materialize the prelude once per process in the temp dir.
  let prelude_path =
    std::env::temp_dir().join(format!("latexml_lua_prelude_{}.lua", std::process::id()));
  if !prelude_path.exists() {
    std::fs::write(&prelude_path, LUA_PRELUDE).ok()?;
  }
  let mut cmd = Command::new("texlua");
  cmd
    .arg(&prelude_path)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::null());
  // LuaTeX job semantics: chunks resolve relative files against the
  // document's directory (witness newpax/newpax — `newpax.writenewpax
  // ("doc-input1")` reads the PDFs shipped next to the manual).
  let source_dir = latexml_core::state::lookup_string("SOURCEDIRECTORY");
  if !source_dir.is_empty() && std::path::Path::new(&source_dir).is_dir() {
    cmd.current_dir(&source_dir);
  }
  let mut child = cmd.spawn().ok()?;
  let stdin = child.stdin.take()?;
  let stdout = BufReader::new(child.stdout.take()?);
  Some(LuaProc { child, stdin, stdout })
}

/// Service one live tex-state query from the Lua side (the mirror half of
/// rebind-as-we-emulate): `get{count,dimen} <key>` reads a register from the
/// engine State, `set{count,dimen} [global] <key> <value>` writes one. Keys
/// are numeric register indices (`tex.count[255]` → `\count255`) or
/// `\countdef`'d names (`tex.count.scratchcounter` → `\scratchcounter`).
/// Counts answer as integers, dimens in scaled points (LuaTeX convention).
/// Unknown ops or lookup failures answer "0" — the pre-mirror stub value —
/// so a chunk probing state we don't model degrades exactly as before.
fn service_query(payload: &str) -> String {
  use latexml_core::{
    common::{dimension::Dimension, number::Number},
    definition::{argument::ArgWrap, register::RegisterValue},
    state,
  };
  let mut it = payload.split_whitespace();
  let op = it.next().unwrap_or("");
  let mut key = it.next().unwrap_or("");
  let global = key == "global";
  if global {
    key = it.next().unwrap_or("");
  }
  let (cs, params) = match key.parse::<i64>() {
    Ok(n) => {
      let reg = if op.ends_with("dimen") {
        "\\dimen"
      } else {
        "\\count"
      };
      (reg.to_string(), vec![ArgWrap::Number(Number(n))])
    },
    Err(_) => (format!("\\{key}"), Vec::new()),
  };
  match op {
    "getcount" | "getdimen" => match state::lookup_register(&cs, params).ok().flatten() {
      Some(RegisterValue::Number(n)) => n.0.to_string(),
      Some(RegisterValue::Dimension(d)) => d.0.to_string(),
      _ => "0".to_string(),
    },
    "setcount" | "setdimen" => {
      let value: i64 = it.next().and_then(|v| v.parse().ok()).unwrap_or(0);
      let rv = if op == "setdimen" {
        RegisterValue::Dimension(Dimension(value))
      } else {
        RegisterValue::Number(Number(value))
      };
      let scope = global.then_some(state::Scope::Global);
      let _ = state::assign_register(&cs, rv, scope, params);
      String::new()
    },
    _ => "0".to_string(),
  }
}

/// Execute one Lua chunk in this thread's persistent interpreter.
///
/// `Ok(text)` is everything the chunk `tex.print`ed (to be retokenized with
/// current catcodes by the caller); `Err(msg)` is a load/runtime Lua error or
/// bridge unavailability ("texlua not available" exactly once per thread —
/// callers should degrade to a no-op expansion, matching what the document
/// would get from a TeX engine without Lua).
pub fn lua_exec(chunk: &str) -> Result<String, String> {
  LUA.with(|slot| {
    let mut slot = slot.borrow_mut();
    let entry = slot.get_or_insert_with(spawn);
    let Some(proc_) = entry.as_mut() else {
      return Err("texlua not available".to_string());
    };
    let do_io = |p: &mut LuaProc| -> std::io::Result<(String, String)> {
      p.stdin.write_all(format!("{}\n", chunk.len()).as_bytes())?;
      p.stdin.write_all(chunk.as_bytes())?;
      p.stdin.write_all(b"\n")?;
      p.stdin.flush()?;
      // The chunk may interleave 'Q'uery frames (live tex-state reads/writes)
      // before its final OK/ERR frame — service each against engine State.
      loop {
        let mut header = String::new();
        p.stdout.read_line(&mut header)?;
        let mut it = header.trim_end().splitn(2, ' ');
        let tag = it.next().unwrap_or("").to_string();
        let n: usize = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let mut body = vec![0u8; n];
        std::io::Read::read_exact(&mut p.stdout, &mut body)?;
        let body = String::from_utf8_lossy(&body).into_owned();
        if tag == "Q" {
          let answer = service_query(&body);
          p.stdin
            .write_all(format!("A {}\n", answer.len()).as_bytes())?;
          p.stdin.write_all(answer.as_bytes())?;
          p.stdin.flush()?;
          continue;
        }
        return Ok((tag, body));
      }
    };
    match do_io(proc_) {
      Ok((tag, body)) if tag == "OK" => Ok(body),
      Ok((_, body)) => Err(body),
      Err(e) => {
        // Pipe broke (child died): drop it so the next call can respawn.
        *entry = None;
        Err(format!("texlua bridge I/O failed: {e}"))
      },
    }
  })
}

#[cfg(test)]
mod tests {
  use super::lua_exec;

  fn texlua_present() -> bool {
    std::process::Command::new("texlua")
      .arg("--version")
      .output()
      .is_ok_and(|o| o.status.success())
  }

  /// Chunk execution, persistent globals across calls, print vs sprint
  /// line semantics, and Lua-error reporting. Self-skips (green) without a
  /// host texlua — note the CI trimmed-TeX-Live trap: a green run here does
  /// not by itself prove the bridge ran.
  #[test]
  fn bridge_executes_and_persists_state() {
    if !texlua_present() {
      return;
    }
    assert_eq!(lua_exec("x = 6*7").as_deref(), Ok(""));
    assert_eq!(lua_exec("tex.print(x)").as_deref(), Ok("42\n"));
    assert_eq!(
      lua_exec("tex.sprint('a')  tex.sprint('b')").as_deref(),
      Ok("ab")
    );
    // Leading catcode-table number is ignored per LuaTeX convention.
    assert_eq!(lua_exec("tex.sprint(0, 'c')").as_deref(), Ok("c"));
    assert!(lua_exec("this is not lua").is_err());
    // The state survives an error.
    assert_eq!(lua_exec("tex.print(x)").as_deref(), Ok("42\n"));
  }

  /// Rebind-as-we-emulate shims for engine intents texlua itself cannot carry
  /// (its built-in `tex` table holds only run/initialize/finish — every
  /// `tex.*` an author's chunk touches is OUR shim, so the only question is
  /// how deep each shim goes; docs/perfect_kernel/LUA_REBINDING.md).
  /// Witnesses (TL2025 doc corpus, clean-lualatex slice):
  ///   derivative/derivative — `pdf.setmajorversion(2)` backend intent,
  ///     meaningless for XML output → absorb (accept, do nothing);
  ///   abntexto/abntexto + abntexto-uece — `tex.primitives()` feeds a
  ///     listings texcs highlight list → absorb as an empty table (the
  ///     loop body degrades to "no extra highlighted names", no error);
  ///   newpax/newpax — `require("newpax")` resolves a texmf-shipped Lua
  ///     module → rebind require through texlua's built-in kpse (plus
  ///     lualibs, which supplies the `file` library lualatex gets from
  ///     luaotfload; measured 89 ms one-time load).
  #[test]
  fn rebound_engine_intents_absorb_and_resolve() {
    if !texlua_present() {
      return;
    }
    assert_eq!(
      lua_exec("pdf.setmajorversion(2) pdf.setminorversion(0)").as_deref(),
      Ok("")
    );
    assert_eq!(
      lua_exec("for k,v in pairs(tex.primitives()) do tex.print(k, v .. ',') end").as_deref(),
      Ok("")
    );
    // texmf Lua-module resolution — only meaningful where kpse can see the
    // module (trimmed-TL CI hosts self-skip, same trap as the bridge test).
    let newpax_installed = std::process::Command::new("kpsewhich")
      .arg("newpax.lua")
      .output()
      .is_ok_and(|o| o.status.success() && !o.stdout.is_empty());
    if newpax_installed {
      assert_eq!(
        lua_exec("local ok = pcall(require, 'newpax') tex.sprint(ok and 'R1' or 'R0')").as_deref(),
        Ok("R1")
      );
    }
  }
}
