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
//! What this bridge reproduces: the Lua LANGUAGE state + the string-emitting
//! half of the `tex` library — which is what documentation-corpus uses of
//! `\directlua` overwhelmingly are (compute something, print tokens back).
//! What it deliberately does NOT reproduce: the node/font/callback layers
//! (`node.*`, `font.*`, `callback.*` — print-shaping machinery manipulating
//! LuaTeX's typesetter internals, out of scope for XML output) and reads of
//! live TeX state (`tex.count[...]` etc. — a chunk that ASKS the TeX side
//! for values gets stub zeros; wiring real register mirroring is future
//! work, tracked in docs/perfect_kernel/CLUSTERS.md).
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
tex.count = setmetatable({}, {__index = function() return 0 end,
                              __newindex = function() end})
tex.dimen = setmetatable({}, {__index = function() return 0 end,
                              __newindex = function() end})
tex.error = function(msg) buf[#buf+1] = '' end
texio = { write = function() end, write_nl = function() end }
status = setmetatable({}, {__index = function() return 0 end})
token = token or {}
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
  let prelude_path = std::env::temp_dir().join(format!(
    "latexml_lua_prelude_{}.lua",
    std::process::id()
  ));
  if !prelude_path.exists() {
    std::fs::write(&prelude_path, LUA_PRELUDE).ok()?;
  }
  let mut child = Command::new("texlua")
    .arg(&prelude_path)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    .spawn()
    .ok()?;
  let stdin = child.stdin.take()?;
  let stdout = BufReader::new(child.stdout.take()?);
  Some(LuaProc {
    child,
    stdin,
    stdout,
  })
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
      p.stdin
        .write_all(format!("{}\n", chunk.len()).as_bytes())?;
      p.stdin.write_all(chunk.as_bytes())?;
      p.stdin.write_all(b"\n")?;
      p.stdin.flush()?;
      let mut header = String::new();
      p.stdout.read_line(&mut header)?;
      let mut it = header.trim_end().splitn(2, ' ');
      let tag = it.next().unwrap_or("").to_string();
      let n: usize = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
      let mut body = vec![0u8; n];
      std::io::Read::read_exact(&mut p.stdout, &mut body)?;
      Ok((tag, String::from_utf8_lossy(&body).into_owned()))
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
}
