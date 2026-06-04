#!/usr/bin/env python3
"""Live smoke harness for `latexml_oxide --server` (the editor LSP).

Usage:
    tools/lsp_smoke.py <path-to-latexml_oxide-binary> [scenario ...]

Scenarios (default: all):
    basic      initialize -> convert -> shutdown/exit
    preempt    convert superseded by a newer same-project convert ->
               "cancelled" for the stale id, real HTML for the winner;
               malformed convert answered with -32602
    multifile  project-root detection (find_main_tex), unsaved-buffer
               overlay (didOpen text visible in the preview without
               saving), per-file diagnostics attribution

Exit code 0 only if every requested scenario passes. This is a manual /
CI-extra tool (needs the built binary + a TeX Live tree), not part of
`cargo test`.
"""
import json
import os
import subprocess
import sys
import tempfile
import threading
import time

PASSES = []


def check(cond, label):
    PASSES.append(bool(cond))
    print(("PASS " if cond else "FAIL ") + label)
    return cond


def frame(obj):
    body = json.dumps(obj).encode()
    return b"Content-Length: %d\r\n\r\n%s" % (len(body), body)


def read_message(stream):
    headers = {}
    while True:
        line = stream.readline()
        if not line:
            return None
        line = line.strip()
        if not line:
            break
        k, _, v = line.partition(b":")
        headers[k.strip().lower()] = v.strip()
    n = int(headers.get(b"content-length", b"0"))
    return json.loads(stream.read(n)) if n else None


class Server:
    def __init__(self, binary, timeout=120):
        self.proc = subprocess.Popen(
            [binary, "--server", "--timeout", str(timeout)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL)

    def send(self, *msgs):
        for m in msgs:
            self.proc.stdin.write(frame(m))
        self.proc.stdin.flush()

    def collect(self, want_ids=(), want_methods=(), deadline_s=300):
        """Read until every wanted id has a response and at least one of
        each wanted method notification arrived (or deadline)."""
        responses, notifications = {}, []
        deadline = time.time() + deadline_s
        need_methods = set(want_methods)
        while time.time() < deadline:
            if all(i in responses for i in want_ids) and not need_methods:
                break
            msg = read_message(self.proc.stdout)
            if msg is None:
                break
            if msg.get("id") is not None and ("result" in msg or "error" in msg):
                responses[msg["id"]] = msg
            elif msg.get("method"):
                notifications.append(msg)
                need_methods.discard(msg["method"])
        return responses, notifications

    def shutdown(self, label):
        try:
            self.send({"jsonrpc": "2.0", "id": 99, "method": "shutdown"},
                      {"jsonrpc": "2.0", "method": "exit"})
        except BrokenPipeError:
            pass
        try:
            self.proc.wait(timeout=15)
            check(self.proc.returncode == 0, f"{label}: clean exit rc=0")
        except subprocess.TimeoutExpired:
            self.proc.kill()
            check(False, f"{label}: server exited after `exit`")


def init_msg():
    return {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}


def convert_msg(mid, path, text):
    return {"jsonrpc": "2.0", "id": mid, "method": "latexml/convert",
            "params": {"uri": f"file://{path}", "text": text}}


def scenario_basic(binary):
    print("== basic ==")
    srv = Server(binary)
    with tempfile.TemporaryDirectory() as tmp:
        doc = os.path.join(tmp, "doc.tex")
        tex = ("\\documentclass{article}\n\\begin{document}\n"
               "Hello $1+1=2$.\n\\end{document}\n")
        srv.send(init_msg(), convert_msg(2, doc, tex))
        resp, _ = srv.collect(want_ids=(1, 2))
        check("capabilities" in (resp.get(1, {}).get("result") or {}),
              "basic: initialize -> capabilities")
        r = (resp.get(2) or {}).get("result") or {}
        check(r.get("statusCode") == 0 and "Hello" in r.get("html", ""),
              "basic: convert -> HTML, statusCode 0")
        check(r.get("root", "").endswith("doc.tex"),
              "basic: response carries the converted root")
    srv.shutdown("basic")


def scenario_preempt(binary):
    print("== preempt ==")
    srv = Server(binary)
    with tempfile.TemporaryDirectory() as tmp:
        doc = os.path.join(tmp, "doc.tex")
        tex1 = ("\\documentclass{article}\n\\begin{document}\nFirst.\n"
                "\\end{document}\n")
        tex2 = tex1.replace("First.", "Second.")
        srv.send(init_msg(),
                 convert_msg(2, doc, tex1),
                 convert_msg(3, doc, tex2),
                 # Malformed (no text) + different project: must be ANSWERED.
                 {"jsonrpc": "2.0", "id": 4, "method": "latexml/convert",
                  "params": {"uri": "file:///nonexistent/other.tex"}})
        resp, _ = srv.collect(want_ids=(1, 2, 3, 4))
        r2 = (resp.get(2) or {}).get("result") or {}
        check(r2.get("status") == "cancelled" and r2.get("statusCode") == 0,
              "preempt: stale convert answered `cancelled` (int statusCode)")
        r3 = (resp.get(3) or {}).get("result") or {}
        check(r3.get("statusCode") == 0 and "Second" in r3.get("html", ""),
              "preempt: newest text wins")
        e4 = (resp.get(4) or {}).get("error") or {}
        check(e4.get("code") == -32602,
              "preempt: malformed convert answered -32602")
    srv.shutdown("preempt")


def scenario_multifile(binary):
    print("== multifile ==")
    srv = Server(binary)
    with tempfile.TemporaryDirectory() as tmp:
        main = os.path.join(tmp, "main.tex")
        sections = os.path.join(tmp, "sections")
        os.makedirs(sections)
        ch2 = os.path.join(sections, "ch2.tex")
        with open(main, "w") as f:
            f.write("\\documentclass{article}\n\\begin{document}\n"
                    "Intro from main.\n\\input{sections/ch2}\n"
                    "\\end{document}\n")
        with open(ch2, "w") as f:
            f.write("DISK chapter two text.\n")

        unsaved = ("UNSAVED-EDIT chapter two text with an error: "
                   "\\undefinedmacroxyz{arg}.\n")
        srv.send(init_msg(),
                 {"jsonrpc": "2.0", "method": "textDocument/didOpen",
                  "params": {"textDocument": {
                      "uri": f"file://{ch2}", "version": 1,
                      "languageId": "latex", "text": unsaved}}})
        # didOpen triggers a root conversion + (grouped, multiple)
        # publishDiagnostics notifications. Fence with a sentinel request:
        # the server answers strictly in order, so once id=5 (an unknown
        # method -> -32601) comes back, every prior publish has arrived.
        srv.send({"jsonrpc": "2.0", "id": 5, "method": "latexml/ping"})
        resp5, notes = srv.collect(want_ids=(5,))
        check("error" in (resp5.get(5) or {}),
              "multifile: sentinel fenced the notification stream")
        diag_uris = {n["params"]["uri"]: n["params"]["diagnostics"]
                     for n in notes
                     if n.get("method") == "textDocument/publishDiagnostics"}
        attributed = [u for u, d in diag_uris.items()
                      if u.endswith("ch2.tex") and d]
        check(bool(attributed),
              "multifile: diagnostics attributed to sections/ch2.tex")

        # Convert the CHAPTER buffer; expect the ROOT document converted,
        # with the UNSAVED text (overlay) instead of the disk content.
        srv.send(convert_msg(7, ch2, unsaved))
        resp, _ = srv.collect(want_ids=(7,))
        r = (resp.get(7) or {}).get("result") or {}
        html = r.get("html", "")
        check(r.get("root", "").endswith("main.tex"),
              "multifile: chapter convert resolves to the project root")
        check("Intro from main" in html,
              "multifile: preview contains the root's content")
        check("UNSAVED-EDIT" in html and "DISK chapter" not in html,
              "multifile: preview uses the UNSAVED buffer, not the disk file")
        diags = r.get("diagnostics") or []
        check(any((d.get("file") or "").endswith("ch2.tex") for d in diags),
              "multifile: response diagnostics carry file attribution")
    srv.shutdown("multifile")


SCENARIOS = {"basic": scenario_basic, "preempt": scenario_preempt,
             "multifile": scenario_multifile}


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    binary = sys.argv[1]
    wanted = sys.argv[2:] or list(SCENARIOS)
    for name in wanted:
        SCENARIOS[name](binary)
    ok = all(PASSES)
    print(f"{'OK' if ok else 'FAILED'}: {sum(PASSES)}/{len(PASSES)} checks")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
