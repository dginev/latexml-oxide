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
//! Document Lua is untrusted input: the prelude sandboxes it to TeX's defaults
//! (`shell_escape=f`, `openout_any=p`) before any chunk runs, each refusal an
//! `Error:lua:sandbox` (OXIDIZED_DESIGN_DIVERGENCES #1081).
//!
//! Process shape: one `texlua` child per conversion thread, spawned on first
//! use with a prelude that loops over length-framed requests on stdin and
//! answers `OK`/`ERR` length-framed responses on stdout, each header carrying
//! the nonce the bridge sent as the child's first line. A std
//! `thread_local!` (destructor RUNS at thread exit, unlike `#[thread_local]`)
//! drops the pipes so the child's `io.read` returns nil and it exits.

use std::{
  cell::RefCell,
  io::{BufRead, BufReader, Write},
  process::{ChildStdin, ChildStdout, Command, Stdio},
};

/// The Lua-side server loop. `tex.print(...)` arguments follow the LuaTeX
/// convention: an optional leading catcode-table number (ignored here — the
/// text is retokenized with current catcodes on the TeX side either way),
/// then strings / numbers / arrays of strings.
const LUA_PRELUDE: &str = r#"
-- The sandbox's own helpers, held before any other code runs: document code can
-- replace a global or a library field (string.find, the file methods), never
-- these locals (OXIDIZED_DESIGN_DIVERGENCES #1081).
local type, tostring, tonumber, error, select, next, pcall, ipairs, pairs, assert =
  type, tostring, tonumber, error, select, next, pcall, ipairs, pairs, assert
local setmetatable = setmetatable
local s_find, s_sub, s_gsub, s_gmatch, s_format, s_match =
  string.find, string.sub, string.gsub, string.gmatch, string.format, string.match
local t_concat, t_unpack, t_insert = table.concat, table.unpack, table.insert
local m_floor, os_exit = math.floor, os.exit
-- The protocol's pipe ends and the C functions that use them: the document
-- reaches neither (io.stdout, io.write and print are pointed away below).
local stdin, stdout = io.stdin, io.stdout
local f_read, f_write, f_flush = stdin.read, stdout.write, stdout.flush
-- The bridge's first line, read before any other code loads: every frame header carries it, and the bridge rejects
-- a header without it, so bytes that reach the pipe by any other route are a broken protocol, never a frame.
local NONCE = f_read(stdin, '*l') or ''
-- A frame's length is the bytes that reach the pipe: Windows texlua writes
-- stdout in text mode, each '\n' as '\r\n' (the bridge drops the added '\r').
local CRLF = s_sub(package.config, 1, 1) == '\\'
local function wire_len(s)
  if CRLF then
    local _, newlines = s_gsub(s, '\n', '')
    return #s + newlines
  end
  return #s
end
-- One write per frame, so no frame lands inside another. Built by concatenation: '%s' would ask the string
-- metatable's __tostring, which document code can set.
local function frame(tag, payload)
  f_write(stdout, tag .. ' ' .. NONCE .. ' ' .. s_format('%d', wire_len(payload)) .. '\n' .. payload)
  f_flush(stdout)
end
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
tex.tprint = function(tbl) for _, v in ipairs(tbl) do emit('', t_unpack(v)) end end
-- Live register mirror (rebind-as-we-emulate): texlua has no engine, so
-- tex.count/tex.dimen access round-trips to the Rust State over the pipe as
-- a framed query ('Q <len>' out, 'A <len>' back). Values follow the LuaTeX
-- convention: counts are integers, dimens are scaled points.
local function query(payload)
  frame('Q', payload)
  local header = f_read(stdin, '*l')
  local n = tonumber(header and s_match(header, '%d+')) or 0
  return n > 0 and f_read(stdin, n) or ''
end
local function regtable(kind)
  return setmetatable({}, {
    __index = function(_, k)
      return tonumber(query('get' .. kind .. ' ' .. tostring(k))) or 0
    end,
    __newindex = function(_, k, v)
      query('set' .. kind .. ' ' .. tostring(k) .. ' ' .. tostring(m_floor(v)))
    end})
end
local function regsetter(kind)
  return function(a, b, c)
    if a == 'global' then
      query('set' .. kind .. ' global ' .. tostring(b) .. ' ' .. tostring(m_floor(c)))
    else
      query('set' .. kind .. ' ' .. tostring(a) .. ' ' .. tostring(m_floor(b)))
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
    if type(k) == 'string' and s_find(k, '^get') then return function() return 0 end end
    return function() end
  end})
texio = { write = function() end, write_nl = function() end }
status = setmetatable({}, {__index = function() return 0 end})
token = token or {}
-- require('texio') and the like answer the shims, not texlua's originals (whose texio.write reached the pipe).
package.loaded.texio, package.loaded.pdf, package.loaded.status, package.loaded.token = texio, pdf, status, token
package.loaded.node = nil
-- Sandbox (OXIDIZED_DESIGN_DIVERGENCES #1081): document Lua is untrusted input,
-- held to TeX's defaults for it, shell_escape=f (no process runs) and
-- openout_any=p (kpathsea's kpse_out_name_ok: a write only to a relative path
-- with no '..' part and no dotfile), checked on every path component and through
-- no symbolic link; reads stay open (openin_any=a). texlua's
-- --safer/--nosocket/--shell-escape do not restrict a script, so the prelude
-- does it before any document chunk runs. A denied call reports a 'D' frame (an
-- Error a pcall cannot swallow) and raises a Lua error.
-- While the prelude loads lualibs (texmf code, trusted like a format), the
-- originals still answer; the wrappers it captures deny once the loop starts.
local booting = true
local function deny(what)
  if booting then error(what .. ' is not allowed', 3) end
  local msg = what .. ' is not allowed (document Lua runs sandboxed: no programs, writes only below the job directory)'
  frame('D', msg)
  error(msg, 3)
end
local function denier(what, original)
  return function(...)
    if booting and original then return original(...) end
    deny(what)
  end
end
-- A name for a refusal message: the document's value is never asked to describe itself, and a long one is cut.
local function shown(name)
  if type(name) ~= 'string' then return '<' .. type(name) .. '>' end
  if #name > 256 then return s_sub(name, 1, 256) .. '... (' .. s_format('%d', #name) .. ' bytes)' end
  return name
end
local symlinkattributes = lfs and lfs.symlinkattributes
-- kpse_out_name_ok under openout_any=p, decided here rather than by texlua's
-- kpse.out_name_ok, which follows the host's texmf.cnf (openout_any=a there
-- allows anything) and checks only the last component for a dotfile.
local function out_name_ok(name)
  if type(name) ~= 'string' or name == '' or s_find(name, '\0', 1, true) then return false end
  local path = name
  if CRLF then
    -- Windows: '\' separates too; a ':' is a drive or an alternate data stream; '\\?\' and '\\.\' are device paths.
    if s_find(name, '^[\\/][\\/][%?%.][\\/]') or s_find(name, ':', 1, true) then return false end
    path = s_gsub(name, '\\', '/')
  elseif s_find(name, '\\', 1, true) then
    -- Unix: '\' is a filename character; the name checked must be the name the system opens.
    return false
  end
  if s_sub(path, 1, 1) == '/' then return false end
  local prefix = nil
  for part in s_gmatch(path, '[^/]+') do
    if part == '..' or (s_sub(part, 1, 1) == '.' and part ~= '.' and part ~= '.tex') then return false end
    -- (Windows drops a component's trailing dots and spaces: 'a.' opens 'a')
    if CRLF and part ~= '.' and s_find(part, '[%. ]$') then return false end
    -- No component may be a symbolic link: one made by the source tree could point anywhere.
    prefix = prefix and (prefix .. '/' .. part) or part
    if symlinkattributes and symlinkattributes(prefix, 'mode') == 'link' then return false end
  end
  return true
end
local function writes(mode) return type(mode) == 'string' and s_find(mode, '[wa+]') ~= nil end
for _, name in ipairs({'execute', 'exec', 'spawn', 'kpsepopen', 'remove', 'rename', 'tmpname', 'tmpdir', 'setenv'}) do
  os[name] = denier('os.' .. name, os[name])
end
io.popen = denier('io.popen', io.popen)
local io_open, io_output, io_input = io.open, io.output, io.input
io.open = function(name, mode)
  if writes(mode) and not out_name_ok(name) then deny('writing ' .. shown(name)) end
  return io_open(name, mode)
end
io.output = function(file)
  if type(file) == 'string' and not out_name_ok(file) then deny('writing ' .. shown(file)) end
  return io_output(file)
end
if gzip and gzip.open then
  local gzip_open = gzip.open
  gzip.open = function(name, mode)
    if writes(mode) and not out_name_ok(name) then deny('writing ' .. shown(name)) end
    return gzip_open(name, mode)
  end
end
-- mplib takes its file finder in the options table: a copy, read raw, carries the
-- sandbox's finder, whatever the document's table does on assignment.
if mplib and mplib.new then
  local mplib_new = mplib.new
  mplib.new = function(options)
    if options ~= nil and type(options) ~= 'table' then deny('mplib.new with a ' .. type(options)) end
    local copy = {}
    if options then for k, v in next, options do copy[k] = v end end
    local find = copy.find_file
    if type(find) ~= 'function' then find = nil end
    copy.find_file = function(name, mode, kind)
      local found = name
      if find then found = find(name, mode, kind) end
      if writes(mode) and found and not out_name_ok(found) then
        frame('D', 'writing ' .. shown(found) .. ' is not allowed (document Lua runs sandboxed)')
        return nil
      end
      return found
    end
    return mplib_new(copy)
  end
end
-- img needs the engine's backend: texlua ends on its calls (img.new, img.scan, img.write: a crash).
if img then
  for name, value in pairs(img) do
    if type(value) == 'function' then img[name] = denier('img.' .. name, value) end
  end
end
if lfs then
  local read_only = {attributes = true, currentdir = true, dir = true, isdir = true, isfile = true,
    readlink = true, shortname = true, symlinkattributes = true}
  for name, value in pairs(lfs) do
    if type(value) == 'function' and not read_only[name] then lfs[name] = denier('lfs.' .. name, value) end
  end
end
package.loadlib = denier('package.loadlib', package.loadlib)
local c_path = package.cpath
package.cpath = ''
local function is_native(name)
  return name == 'ffi' or s_find(name, '^socket') ~= nil or s_find(name, '^mime') ~= nil
end
for _, name in ipairs({'ffi', 'socket', 'mime'}) do _G[name] = nil end
for _, registry in ipairs({package.loaded, package.preload}) do
  for name in pairs(registry) do
    if type(name) == 'string' and is_native(name) then registry[name] = nil end
  end
end
if lua then lua.getbytecode, lua.setbytecode, lua.bytecode = nil, nil, nil end
-- Text chunks only: crafted bytecode can break out of any Lua-level sandbox.
local base_load, base_loadfile = load, loadfile
-- (an env passed as nil is still an env: forward it only when given)
local function load_text(chunk, name, mode, ...)
  if select('#', ...) > 0 then return base_load(chunk, name, 't', ...) end
  return base_load(chunk, name, 't')
end
local function loadfile_text(file, mode, ...)
  if select('#', ...) > 0 then return base_loadfile(file, 't', ...) end
  return base_loadfile(file, 't')
end
load = load_text
loadstring = function(chunk, name) return base_load(chunk, name, 't') end
loadfile = loadfile_text
dofile = function(file) return assert(loadfile_text(file))() end
-- require: the preload table, then Lua files on package.path and (below) the
-- texmf tree via kpse, all text-only; a C module or a network library is denied.
local searchpath = package.searchpath
local preload = (package.searchers or package.loaders)[1]
local searchers = {
  function(name)
    if type(name) == 'string' and is_native(name) then
      return function() deny('require "' .. shown(name) .. '"') end
    end
    return preload(name)
  end,
  function(name)
    local file, err = searchpath(name, package.path)
    if not file then return err end
    return assert(loadfile_text(file)), file
  end,
}
package.searchers, package.loaders = searchers, searchers
searchers[3] = function(name)
  if searchpath(name, c_path) or searchpath(s_match(name, '^[^.]*'), c_path) then
    return function() deny('loading the C module "' .. shown(name) .. '"') end
  end
  return "\n\tno C modules (document Lua runs sandboxed)"
end
-- The protocol's pipe is the bridge's alone: the document's standard output and
-- input are the null device (its output reaches TeX through tex.print), set before
-- lualibs loads, whose helpers keep what they see (inspect keeps print).
local null_in = io_open(CRLF and 'NUL' or '/dev/null', 'r')
io_output(io.stderr)
if null_in then io_input(null_in) end
io.stdout, io.stdin = io.stderr, null_in
print = function() end
-- Rebind require through texlua's built-in kpse so texmf-shipped Lua modules
-- resolve (newpax, tkz-elements, ...), and load lualibs, which supplies the
-- 'file'/'string'/'table' library extensions lualatex gets via luaotfload.
if kpse and kpse.set_program_name then
  pcall(kpse.set_program_name, 'luatex')
  local find_file = kpse.find_file
  t_insert(searchers, function(name)
    -- LuaTeX's own kpse loader reads a dotted name as a path (pgf.gd.trees.library).
    local f = find_file((s_gsub(name, '%.', '/')), 'lua') or find_file(name, 'lua')
    if f then return assert(loadfile_text(f)), f end
    return "\n\tno texmf module '" .. shown(name) .. "' (kpse)"
  end)
  pcall(require, 'lualibs')
end
-- debug reaches the upvalues the wrappers above hold (the raw io.open, ...).
debug, package.loaded.debug = nil, nil
-- An error value's text, asked of no metamethod (tostring would run the document's __tostring).
local function error_text(value)
  local kind = type(value)
  if kind == 'string' then return value end
  if kind == 'number' then return s_format('%.14g', value) end
  return '(error object is a ' .. kind .. ' value)'
end
booting = false
while true do
  local header = f_read(stdin, '*l')
  if not header then os_exit(0) end
  local n = tonumber(header)
  if not n then os_exit(1) end
  local chunk = f_read(stdin, n)
  f_read(stdin, '*l')
  buf = {}
  local out, tag
  local f, err = load_text(chunk)
  if f then
    local ok, rterr = pcall(f)
    if ok then
      out, tag = t_concat(buf), 'OK'
    else
      out, tag = error_text(rterr), 'ERR'
    end
  else
    out, tag = error_text(err), 'ERR'
  end
  frame(tag, out)
end
"#;

struct LuaProc {
  /// The child, registered so a timeout or memory exit (which runs no destructor) kills it too; dropping the guard
  /// kills and reaps it.
  child:   Option<latexml_core::watchdog::KillOnExit>,
  stdin:   ChildStdin,
  stdout:  BufReader<ChildStdout>,
  /// A private prelude copy (`Prelude::Own`), removed with the process.
  prelude: Option<std::path::PathBuf>,
  /// This interpreter's frame nonce (`session_nonce`), sent as its first line.
  nonce:   String,
}

/// This thread's interpreter for the current conversion (`reset_session` ends it: a persistent worker converts its
/// papers on one thread, and LuaTeX starts each job with a fresh Lua state in the job's directory).
#[derive(Default)]
struct LuaSlot {
  proc_:       Option<LuaProc>,
  /// texlua could not be started (reported once per conversion as "texlua not available").
  unavailable: bool,
  /// Interpreters that ended mid-conversion (a crash, a broken frame); each is respawned up to `MAX_RESTARTS`.
  ended:       u32,
}

/// Restarts per conversion after an interpreter ends; past them, later chunks are not evaluated.
const MAX_RESTARTS: u32 = 3;
/// The longest frame the bridge reads: a length past it is a broken protocol, not output.
const MAX_FRAME: usize = 64 << 20;
/// The refusals a chunk reports one by one, by count and by bytes; the rest are counted in one summary Error.
const MAX_DENIALS: usize = 20;
const MAX_DENIAL_BYTES: usize = 64 << 10;

thread_local! {
  static LUA: RefCell<LuaSlot> = RefCell::new(LuaSlot::default());
}

/// Ends this thread's interpreter and forgets its restarts: called at each conversion's start
/// (`Converter::initialize_session`), so no Lua state or working directory carries over to the next paper.
pub fn reset_session() { LUA.with(|slot| *slot.borrow_mut() = LuaSlot::default()); }

impl Drop for LuaProc {
  fn drop(&mut self) {
    // The guard kills and reaps the child, so no zombie outlives the thread.
    drop(self.child.take());
    if let Some(path) = self.prelude.take() {
      let _ = std::fs::remove_file(path);
    }
  }
}

fn content_hash(text: &str) -> u64 {
  text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
    (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
  })
}

/// Where the prelude script is: the file shared by this user's processes, or a private copy the
/// process removes when its interpreter ends.
enum Prelude {
  Shared(std::path::PathBuf),
  Own(std::path::PathBuf),
}

/// The prelude script on disk: `latexml_lua_prelude_<uid>_<content hash>.lua` in the temp dir, shared
/// by this user's processes and used only when it is a regular file (not a link) of this user holding
/// exactly the prelude — the temp dir is world-writable, and a planted or truncated file would run as
/// this user. Written aside under a name no other writer uses (a pid and a counter; a name left by an
/// earlier process of the same pid is skipped) and renamed into place, so a concurrent reader never
/// sees a partial file; where the shared name cannot be taken, the private copy is used and removed
/// with the interpreter (`LuaProc`'s drop).
fn prelude_file() -> Option<Prelude> {
  use std::io::Write;
  static WRITES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
  let dir = std::env::temp_dir();
  let hash = content_hash(LUA_PRELUDE);
  let (own, mut file) = (0..64).find_map(|_| {
    let own = dir.join(format!(
      "latexml_lua_prelude_{hash:016x}.{}.{}.lua",
      std::process::id(),
      WRITES.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o644);
    match options.open(&own) {
      Ok(file) => Some(Some((own, file))),
      Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => None,
      Err(_) => Some(None),
    }
  })??;
  if file.write_all(LUA_PRELUDE.as_bytes()).is_err() {
    let _ = std::fs::remove_file(&own);
    return None;
  }
  drop(file);
  #[cfg(unix)]
  let owner = {
    use std::os::unix::fs::MetadataExt;
    std::fs::symlink_metadata(&own).ok()?.uid()
  };
  #[cfg(not(unix))]
  let owner = 0;
  let shared = dir.join(format!("latexml_lua_prelude_{owner}_{hash:016x}.lua"));
  let is_ours = |path: &std::path::Path| {
    std::fs::symlink_metadata(path).is_ok_and(|meta| {
      #[cfg(unix)]
      let same_owner = std::os::unix::fs::MetadataExt::uid(&meta) == owner;
      #[cfg(not(unix))]
      let same_owner = true;
      meta.is_file() && same_owner
    })
  };
  if is_ours(&shared) && std::fs::read(&shared).is_ok_and(|bytes| bytes == LUA_PRELUDE.as_bytes()) {
    let _ = std::fs::remove_file(&own);
    return Some(Prelude::Shared(shared));
  }
  if std::fs::rename(&own, &shared).is_ok() {
    return Some(Prelude::Shared(shared));
  }
  Some(Prelude::Own(own))
}

/// The environment variables the texlua child keeps: the program and library search (`PATH`, `LD_LIBRARY_PATH`,
/// `DYLD_*`, Windows' system ones), kpathsea's (`TEXMF*`, `KPATHSEA*`, the `*INPUTS`/`*FONTS` format paths), the user's home (`TEXMFHOME`), the
/// locale, the reproducible-date clock and the temp dir.
fn child_env_allowed(name: &str) -> bool {
  const EXACT: &[&str] = &[
    "PATH",
    "HOME",
    "LANG",
    "LANGUAGE",
    "SOURCE_DATE_EPOCH",
    "FORCE_SOURCE_DATE",
    "TMPDIR",
    "TMP",
    "TEMP",
    "SystemRoot",
    "SYSTEMROOT",
    "windir",
    "SystemDrive",
    "SYSTEMDRIVE",
    "USERPROFILE",
    "HOMEDRIVE",
    "HOMEPATH",
    "APPDATA",
    "LOCALAPPDATA",
    "PATHEXT",
    "TEXCONFIG",
    "TEXFORMATS",
    "TEXPOOL",
    "TEXPSHEADERS",
    "TEXFONTMAPS",
    "LD_LIBRARY_PATH",
  ];
  EXACT.contains(&name)
    || name.starts_with("DYLD_")
    || name.starts_with("LC_")
    || name.starts_with("TEXMF")
    || name.starts_with("KPATHSEA")
    || name.ends_with("INPUTS")
    || name.ends_with("FONTS")
}

/// A per-interpreter frame nonce: 128 bits of SipHash keyed by the standard library's random `RandomState` keys,
/// over the pid, the clock and a counter. The prelude reads it before any other code loads; a chunk never sees it.
fn session_nonce() -> String {
  use std::hash::{BuildHasher, Hasher};
  static SPAWNS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
  let state = std::collections::hash_map::RandomState::new();
  let count = SPAWNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
  let clock = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map_or(0, |since| since.as_nanos());
  let word = |half: u64| {
    let mut hasher = state.build_hasher();
    hasher.write_u64(half);
    hasher.write_u32(std::process::id());
    hasher.write_u128(clock);
    hasher.write_u64(count);
    hasher.finish()
  };
  format!("{:016x}{:016x}", word(0), word(1))
}

fn spawn() -> Option<LuaProc> {
  // `texlua` takes a script FILE (its `-e` is not lua's inline-chunk flag) — materialize the
  // prelude in the temp dir, named by its owner and content: one file shared by this user's processes
  // and version-safe (a per-process name was never removed, and one-shot runs and test processes left ~98,000
  // of them, exhausting `/tmp`'s inodes).
  let (prelude_path, prelude) = match prelude_file()? {
    Prelude::Shared(path) => (path, None),
    Prelude::Own(path) => (path.clone(), Some(path)),
  };
  let mut cmd = Command::new("texlua");
  cmd
    .arg(&prelude_path)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .stderr(Stdio::null())
    // Untrusted Lua sees only what the texmf lookup and the locale need, not the worker's environment.
    .env_clear()
    .envs(std::env::vars_os().filter(|(name, _)| name.to_str().is_some_and(child_env_allowed)));
  // LuaTeX job semantics: chunks resolve relative files against the
  // document's directory (witness newpax/newpax — `newpax.writenewpax
  // ("doc-input1")` reads the PDFs shipped next to the manual).
  let source_dir = latexml_core::state::lookup_string("SOURCEDIRECTORY");
  if !source_dir.is_empty() && std::path::Path::new(&source_dir).is_dir() {
    cmd.current_dir(&source_dir);
  }
  let Ok(mut child) = cmd.spawn() else {
    if let Some(path) = prelude {
      let _ = std::fs::remove_file(path);
    }
    return None;
  };
  let (Some(stdin), Some(stdout)) = (child.stdin.take(), child.stdout.take()) else {
    let _ = child.kill();
    let _ = child.wait();
    if let Some(path) = prelude {
      let _ = std::fs::remove_file(path);
    }
    return None;
  };
  let mut proc_ = LuaProc {
    child: Some(latexml_core::watchdog::kill_on_exit(child)),
    stdin,
    stdout: BufReader::new(stdout),
    prelude,
    nonce: session_nonce(),
  };
  let line = format!("{}\n", proc_.nonce);
  if proc_.stdin.write_all(line.as_bytes()).is_err() {
    return None;
  }
  Some(proc_)
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
/// bridge unavailability ("texlua not available" — callers degrade to a no-op
/// expansion, matching what the document would get from a TeX engine without
/// Lua). Each call the sandbox refused, and an interpreter that ended mid-chunk
/// (its output lost), is raised as an `Error` (the outer `Err`, should that
/// Error escalate to a Fatal).
pub fn lua_exec(chunk: &str) -> latexml_core::common::error::Result<Result<String, String>> {
  let framed = exec_framed(chunk);
  use crate::prelude::*;
  for denial in framed.denials.shown {
    Error!("lua", "sandbox", denial);
  }
  if framed.denials.more > 0 {
    Error!(
      "lua",
      "sandbox",
      format!(
        "{} more calls the sandbox refused in this chunk are not listed",
        framed.denials.more
      )
    );
  }
  if let Some(ended) = framed.ended {
    Error!("lua", "interpreter", ended);
  }
  Ok(framed.result)
}

/// One chunk's run: its result, the sandbox's refusals ('D' frames, OXIDIZED_DESIGN_DIVERGENCES #1081) — returned
/// apart, as a pcall in the chunk can swallow the Lua error, never the record — and the report of an interpreter
/// that ended during it.
struct Framed {
  result:  Result<String, String>,
  denials: Denials,
  ended:   Option<String>,
}

/// A chunk's refusals: the first `MAX_DENIALS` (within `MAX_DENIAL_BYTES`) kept, the rest counted.
#[derive(Default)]
struct Denials {
  shown: Vec<String>,
  bytes: usize,
  more:  usize,
}

impl Denials {
  fn push(&mut self, denial: String) {
    if self.shown.len() < MAX_DENIALS && self.bytes + denial.len() <= MAX_DENIAL_BYTES {
      self.bytes += denial.len();
      self.shown.push(denial);
    } else {
      self.more += 1;
    }
  }
}

/// A frame header's tag and length: `<tag> <nonce> <length>`, with this interpreter's nonce (a header without it
/// came from no prelude frame, OXIDIZED_DESIGN_DIVERGENCES #1081) and a length the bridge reads.
fn parse_header<'h>(header: &'h str, nonce: &str) -> Result<(&'h str, usize), FrameError> {
  let mut fields = header.trim_end().splitn(3, ' ');
  let (tag, sent, len) = (fields.next(), fields.next(), fields.next());
  match (tag, len.map(str::parse::<usize>)) {
    (Some(tag), Some(Ok(n))) if sent == Some(nonce) && n <= MAX_FRAME => Ok((tag, n)),
    _ if sent == Some(nonce) => Err(FrameError::Protocol(format!(
      "a frame header '{}'",
      header.trim_end()
    ))),
    _ => Err(FrameError::Protocol(
      "output that is not a frame (no session nonce)".to_string(),
    )),
  }
}

/// Why a frame exchange failed: the pipe (the child ended) or a frame no prelude writes.
enum FrameError {
  Io(std::io::Error),
  Protocol(String),
}

impl From<std::io::Error> for FrameError {
  fn from(e: std::io::Error) -> Self { FrameError::Io(e) }
}

/// Sends `chunk` and reads frames up to its final `OK`/`ERR`, servicing `Q`ueries and collecting `D`enials.
fn exchange(
  p: &mut LuaProc,
  chunk: &str,
  denials: &mut Denials,
) -> Result<(bool, String), FrameError> {
  use std::io::Read;
  p.stdin.write_all(format!("{}\n", chunk.len()).as_bytes())?;
  p.stdin.write_all(chunk.as_bytes())?;
  p.stdin.write_all(b"\n")?;
  p.stdin.flush()?;
  // The chunk may interleave 'Q'uery frames (live tex-state reads/writes)
  // before its final OK/ERR frame — service each against engine State.
  loop {
    let mut header = String::new();
    (&mut p.stdout).take(64).read_line(&mut header)?;
    if header.is_empty() {
      return Err(FrameError::Io(std::io::ErrorKind::UnexpectedEof.into()));
    }
    let (tag, n) = parse_header(&header, &p.nonce)?;
    let mut body = vec![0u8; n];
    p.stdout.read_exact(&mut body)?;
    let mut body = String::from_utf8_lossy(&body).into_owned();
    if cfg!(windows) {
      // Undo the text-mode stdout: drop the `\r` written before each `\n` (a `\r\n` the chunk printed came
      // through as `\r\r\n`, and stays `\r\n`).
      body = body.replace("\r\n", "\n");
    }
    match tag {
      "D" => denials.push(body),
      "Q" => {
        let answer = service_query(&body);
        p.stdin
          .write_all(format!("A {}\n", answer.len()).as_bytes())?;
        p.stdin.write_all(answer.as_bytes())?;
        p.stdin.flush()?;
      },
      "OK" => return Ok((true, body)),
      "ERR" => return Ok((false, body)),
      _ => return Err(FrameError::Protocol(format!("a frame tagged '{tag}'"))),
    }
  }
}

fn exec_framed(chunk: &str) -> Framed {
  let mut framed = Framed {
    result:  Ok(String::new()),
    denials: Denials::default(),
    ended:   None,
  };
  // (a speculative run is undone, the interpreter's globals could not be: `\\author{First\\luaexec{n=(n or 0)+1}}`, which a second
  // `\\author` replaces, left its count for the body's `\\luaexec{tex.print(n or 0)}`, lualatex's 0)
  if latexml_core::gullet::in_speculative_digestion() {
    framed.result = Err("a speculative run evaluates no Lua".to_string());
    return framed;
  }
  LUA.with(|slot| {
    let mut slot = slot.borrow_mut();
    if slot.unavailable {
      framed.result = Err("texlua not available".to_string());
      return;
    }
    if slot.ended > MAX_RESTARTS {
      framed.result = Err(format!(
        "texlua ended {} times in this conversion",
        slot.ended
      ));
      return;
    }
    if slot.proc_.is_none() {
      slot.proc_ = spawn();
      if slot.proc_.is_none() {
        slot.unavailable = true;
        framed.result = Err("texlua not available".to_string());
        return;
      }
    }
    // Windows texlua reads stdin in text mode: a `\r\n` arrives as `\n`, one byte short of the frame's count, so
    // it is sent as the `\n` it would read (the prelude's `wire_len` covers the other direction).
    let chunk: std::borrow::Cow<str> = if cfg!(windows) {
      chunk.replace("\r\n", "\n").into()
    } else {
      chunk.into()
    };
    let Some(proc_) = slot.proc_.as_mut() else {
      return;
    };
    framed.result = match exchange(proc_, &chunk, &mut framed.denials) {
      Ok((true, body)) => Ok(body),
      Ok((false, body)) => Err(body),
      Err(error) => {
        // The interpreter ended (a crash, `os.exit`) or broke the protocol: its output is lost and its state gone.
        // Dropping it kills and reaps the child; the next chunk starts a fresh one, up to MAX_RESTARTS times.
        slot.proc_ = None;
        slot.ended += 1;
        let why = match error {
          FrameError::Io(e) => format!("the texlua interpreter ended ({e})"),
          FrameError::Protocol(what) => format!("the texlua interpreter sent {what}"),
        };
        let next = if slot.ended > MAX_RESTARTS {
          "; later chunks are not evaluated"
        } else {
          "; the next chunk starts a fresh interpreter"
        };
        framed.ended = Some(format!(
          "{why}: this chunk's output and the Lua state are lost{next}"
        ));
        Err(why)
      },
    };
  });
  framed
}

#[cfg(test)]
mod tests {
  fn lua_exec(chunk: &str) -> Result<String, String> { super::exec_framed(chunk).result }

  /// A frame header is accepted only with this interpreter's nonce and a length within `MAX_FRAME`; bytes without
  /// the nonce (a header a chunk forged, or stray output) are a broken protocol whatever their tag.
  #[test]
  fn frame_headers_carry_the_session_nonce() {
    use super::{FrameError, parse_header};
    let nonce = super::session_nonce();
    assert_eq!(nonce.len(), 32);
    assert_ne!(nonce, super::session_nonce());
    let accepted = |header: &str| match parse_header(header, &nonce) {
      Ok((tag, n)) => Some(format!("{tag} {n}")),
      Err(FrameError::Protocol(_)) => None,
      Err(FrameError::Io(e)) => panic!("{e}"),
    };
    assert_eq!(accepted(&format!("D {nonce} 7\n")).as_deref(), Some("D 7"));
    assert_eq!(
      accepted(&format!("OK {nonce} 0\r\n")).as_deref(),
      Some("OK 0")
    );
    assert_eq!(accepted("OK 7\n"), None);
    assert_eq!(accepted("D 0123456789abcdef0123456789abcdef 7\n"), None);
    assert_eq!(accepted(&format!("OK {nonce} 99999999999\n")), None);
    assert_eq!(accepted(&format!("OK {nonce}\n")), None);
  }

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
