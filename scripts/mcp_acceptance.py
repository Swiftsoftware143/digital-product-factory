#!/usr/bin/env python3
"""Acceptance for the DPF headless MCP door (card t_74f0e3ce).

Proves, on Linux, with no Windows and no AI provider key:

  1. stdio:  initialize -> tools/list -> create a product -> read it back -> list it,
             with the record confirmed directly in the SQLite file;
  2. serve:  the same door over HTTP on loopback, refusing a missing or wrong bearer;
  3. refusals that must never regress: a non-loopback bind, a short key, a missing key file;
  4. the tool surface contains nothing that publishes, delivers, charges or touches the licence;
  5. the audit log records every call with the local subject and never a token.

Run it from the repo root:
    python3 scripts/mcp_acceptance.py                 # uses target/debug/dpf
    python3 scripts/mcp_acceptance.py --bin target/release/dpf
"""

import argparse
import json
import os
import select
import shutil
import socket
import sqlite3
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request

PASS = 0
FAIL = 0
FAILURES = []


def check(name, condition, detail=""):
    global PASS, FAIL
    if condition:
        PASS += 1
        print(f"  ok   {name}")
    else:
        FAIL += 1
        FAILURES.append(f"{name}: {detail}")
        print(f"  FAIL {name}  {detail}")


def newest_protocol_version():
    return "2025-06-18"


# ── stdio ────────────────────────────────────────────────────────────────────────────


class Stdio:
    def __init__(self, argv):
        self.proc = subprocess.Popen(
            argv,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
            # A developer machine may export DPF_*_KEY; the acceptance must not depend on it.
            env={k: v for k, v in os.environ.items() if not k.startswith("DPF_")},
        )
        stdin, stdout, stderr = self.proc.stdin, self.proc.stdout, self.proc.stderr
        assert stdin is not None and stdout is not None and stderr is not None
        self.stdin, self.stdout, self.stderr = stdin, stdout, stderr

    def send(self, message):
        self.stdin.write(json.dumps(message) + "\n")
        self.stdin.flush()

    def read(self, timeout=30.0):
        """One newline-delimited JSON response, or None if the process ended/closed."""
        deadline = time.time() + timeout
        while True:
            remaining = max(0.0, deadline - time.time())
            ready, _, _ = select.select([self.stdout], [], [], remaining)
            if not ready:
                raise TimeoutError("no response from the door")
            line = self.stdout.readline()
            if line == "":
                return None
            line = line.strip()
            if line:
                return json.loads(line)

    def call(self, rid, method, params=None):
        message = {"jsonrpc": "2.0", "id": rid, "method": method}
        if params is not None:
            message["params"] = params
        self.send(message)
        return self.read()

    def close(self):
        try:
            self.stdin.close()
        except Exception:
            pass
        try:
            return self.proc.wait(timeout=15)
        except subprocess.TimeoutExpired:
            self.proc.kill()
            return -1

    def stderr_text(self):
        try:
            return self.stderr.read()
        except Exception:
            return ""


def tool_result(response):
    """(payload_dict, is_error) from a tools/call response."""
    result = response.get("result") or {}
    structured = result.get("structuredContent")
    if structured is None:
        # fall back to the text block, which is what every client renders
        blocks = result.get("content") or []
        if blocks and blocks[0].get("type") == "text":
            structured = json.loads(blocks[0]["text"])
    return structured, bool(result.get("isError"))


def write_config(path, openai_key):
    """A complete AppConfig as JSON (what `--settings` accepts), with the five provider keys."""
    json.dump({
        "openai_key": openai_key,
        "anthropic_key": "", "google_key": "", "deepseek_key": "", "moonshot_key": "",
        "etsy_api_key": "",
        "auto_save": True, "dark_mode": True, "sidebar_expanded": True,
        "default_view": "dashboard",
        "max_concurrent_tasks": 4, "cache_size_mb": 100,
        "max_searches_per_hour": 20, "max_products_per_day": 10, "max_publish_per_hour": 5,
        "strategy_provider": "", "strategy_model": "",
    }, open(path, "w", encoding="utf-8"))
    return path


def run_stdio_suite(binary, workdir):
    print("stdio: initialize -> tools/list -> create -> read back")
    db = os.path.join(workdir, "stdio.db")
    audit = os.path.join(workdir, "stdio-audit.log")
    settings = write_config(os.path.join(workdir, "no-keys.json"), "")
    door = Stdio([binary, "mcp", "--db", db, "--audit", audit, "--settings", settings])
    try:
        init = door.call(1, "initialize", {
            "protocolVersion": newest_protocol_version(),
            "clientInfo": {"name": "dpf-acceptance", "version": "1"},
            "capabilities": {},
        })
        check("initialize answers serverInfo", bool(init and init.get("result", {}).get("serverInfo")),
              str(init)[:200])
        check("server names itself digital-product-factory",
              (init or {}).get("result", {}).get("serverInfo", {}).get("name") == "digital-product-factory",
              str((init or {}).get("result", {}).get("serverInfo")))

        # The initialized notification must produce NO reply; the next id we read must be ours.
        door.send({"jsonrpc": "2.0", "method": "notifications/initialized"})

        listed = door.call(2, "tools/list")
        tools = ((listed or {}).get("result") or {}).get("tools") or []
        names = [t["name"] for t in tools]
        expected = {
            "dpf_list_products", "dpf_get_product", "dpf_create_product",
            "dpf_list_templates", "dpf_generate_content", "dpf_strategy_brief",
        }
        check("tools/list returns the six documented tools", set(names) == expected,
              f"got {sorted(names)}")
        check("every tool name is dpf_-prefixed", all(n.startswith("dpf_") for n in names), str(names))

        # The deliberate absences: nothing that publishes, delivers, charges or touches the licence.
        forbidden = ("publish", "charge", "checkout", "payment", "licen", "deliver", "sale", "refund")
        offenders = [n for n in names if any(word in n for word in forbidden)]
        check("no publishing/payment/licence tool exists", not offenders, str(offenders))

        created = door.call(3, "tools/call", {
            "name": "dpf_create_product",
            "arguments": {
                "title": "Acceptance Product 1",
                "description": "Created by scripts/mcp_acceptance.py",
                "tags": ["acceptance"],
                "notes": "notes-body-123",
            },
        })
        record, is_error = tool_result(created)
        check("create_product succeeds", not is_error, str(created)[:300])
        check("create_product returns a stored record with an id",
              isinstance(record, dict) and isinstance(record.get("id"), int), str(record)[:200])
        new_id = record.get("id") if isinstance(record, dict) else None

        fetched = door.call(4, "tools/call", {"name": "dpf_get_product", "arguments": {"id": new_id}})
        read_back, is_error = tool_result(fetched)
        check("get_product reads the created record", not is_error, str(fetched)[:300])
        check("title round-trips", (read_back or {}).get("title") == "Acceptance Product 1",
              str(read_back)[:200])
        check("notes round-trip", (read_back or {}).get("notes") == "notes-body-123",
              str(read_back)[:200])

        listed_products = door.call(5, "tools/call", {"name": "dpf_list_products", "arguments": {}})
        inventory, is_error = tool_result(listed_products)
        check("list_products sees the new product",
              not is_error and any(i.get("id") == new_id for i in (inventory or {}).get("items", [])),
              str(inventory)[:300])

        missing = door.call(6, "tools/call", {"name": "dpf_get_product", "arguments": {"id": 999999}})
        missing_payload, missing_error = tool_result(missing)
        check("a missing id is a tool error, not a record",
              missing_error and (missing_payload or {}).get("error") == "not_found",
              str(missing)[:300])

        templates = door.call(7, "tools/call", {"name": "dpf_list_templates", "arguments": {"limit": 3}})
        template_payload, is_error = tool_result(templates)
        check("templates are listed from the app's own registry",
              not is_error and (template_payload or {}).get("count", 0) >= 1,
              str(templates)[:300])

        # Generation with no key must fail honestly (not hang, not invent content).
        gen = door.call(8, "tools/call", {
            "name": "dpf_generate_content",
            "arguments": {"template_id": "nonexistent-template", "params": {}},
        })
        gen_payload, gen_error = tool_result(gen)
        check("generation without a BYOK key fails honestly",
              gen_error and "key" in json.dumps(gen_payload).lower(),
              str(gen)[:300])
    finally:
        exit_code = door.close()
        stderr = door.stderr_text()

    check("stdio door exits 0 when the client closes the stream", exit_code == 0,
          f"exit={exit_code} stderr={stderr[-300:]}")
    check("stdio writes nothing to stderr on a clean run", stderr.strip() == "", stderr[-300:])

    # The database is the truth: the row must be there even with the door gone.
    con = sqlite3.connect(db)
    try:
        rows = con.execute(
            "select id, title, notes, stage from ideas where id = ?", (new_id,)
        ).fetchall()
    finally:
        con.close()
    check("the product exists as a real SQLite row", len(rows) == 1, str(rows))
    if rows:
        check("the stored row carries the title and notes",
              rows[0][1] == "Acceptance Product 1" and rows[0][2] == "notes-body-123", str(rows[0]))

    check_audit(audit, "stdio", min_tool_calls=6, rejected=0)
    return new_id


def check_audit(path, label, min_tool_calls, rejected):
    """`rejected` is how many requests this transport sent with a bad/absent bearer."""
    check(f"{label}: audit log exists", os.path.isfile(path), path)
    if not os.path.isfile(path):
        return
    lines = [l for l in open(path, encoding="utf-8").read().splitlines() if l.strip()]
    check(f"{label}: audit has a line per tool call and per rejected credential",
          len(lines) >= min_tool_calls + rejected,
          f"{len(lines)} lines, expected >= {min_tool_calls + rejected}")
    try:
        records = [json.loads(l) for l in lines]
    except json.JSONDecodeError as e:
        check(f"{label}: audit lines are JSON", False, str(e))
        return
    check(f"{label}: audit names the door",
          all(r.get("door") == "digital-product-factory" for r in records),
          str({r.get("door") for r in records}))
    authed = [r for r in records if r.get("outcome") != "unauthorized: missing"
              and not str(r.get("outcome", "")).startswith("unauthorized")]
    rejected_rows = [r for r in records if r not in authed]
    check(f"{label}: every authenticated call is audited as the local agent",
          authed and all(r.get("subject") == "local-agent" for r in authed),
          str({r.get("subject") for r in authed}))
    check(f"{label}: a rejected credential is audited under '-', not a token",
          len(rejected_rows) == rejected
          and all(r.get("subject") == "-" for r in rejected_rows),
          f"{len(rejected_rows)} rejection(s): {[r.get('subject') for r in rejected_rows]}")
    check(f"{label}: audit carries the tool name and outcome",
          all(r.get("tool") and r.get("outcome") for r in records), "missing fields")
    raw = open(path, encoding="utf-8").read()
    check(f"{label}: no bearer token in the audit", "Bearer" not in raw, "token-shaped text found")


# ── HTTP ────────────────────────────────────────────────────────────────────────────


def free_port():
    with socket.socket() as s:
        s.bind(("127.0.0.1", 0))
        return s.getsockname()[1]


def http_post(url, payload, token=None, timeout=30):
    body = json.dumps(payload).encode()
    request = urllib.request.Request(url, data=body, method="POST")
    request.add_header("Content-Type", "application/json")
    if token is not None:
        request.add_header("Authorization", f"Bearer {token}")
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            return response.status, json.loads(response.read().decode() or "{}")
    except urllib.error.HTTPError as e:
        return e.code, json.loads(e.read().decode() or "{}")


def wait_for_port(port, timeout=20.0):
    deadline = time.time() + timeout
    while time.time() < deadline:
        with socket.socket() as s:
            if s.connect_ex(("127.0.0.1", port)) == 0:
                return True
        time.sleep(0.1)
    return False


def run_http_suite(binary, workdir):
    print("serve: the same door over loopback HTTP")
    db = os.path.join(workdir, "serve.db")
    audit = os.path.join(workdir, "serve-audit.log")
    key_file = os.path.join(workdir, "key.txt")
    token = "acceptance-token-0123456789abcdef0123456789abcdef"
    with open(key_file, "w", encoding="utf-8") as f:
        f.write("# DPF acceptance key file\n\n" + token + "\n")
    os.chmod(key_file, 0o600)

    port = free_port()
    proc = subprocess.Popen(
        [binary, "serve", "--bind", f"127.0.0.1:{port}", "--key-file", key_file,
         "--db", db, "--audit", audit],
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
    )
    try:
        listening = wait_for_port(port)
        early = proc.poll()
        check("serve starts and listens on loopback", listening,
              f"port never opened; exited={early}")
        url = f"http://127.0.0.1:{port}/mcp"

        try:
            with urllib.request.urlopen(f"http://127.0.0.1:{port}/healthz", timeout=5) as probe:
                health_status = probe.status
        except urllib.error.HTTPError as e:
            health_status = e.code
        check("GET /healthz answers without a token", health_status == 200,
              f"status={health_status}")

        status, _ = http_post(url, {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}})
        check("no bearer -> 401", status == 401, f"status={status}")

        status, _ = http_post(url, {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}},
                              token="wrong-token-wrong-token-wrong")
        check("wrong bearer -> 401", status == 401, f"status={status}")

        status, init = http_post(url, {
            "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": {"protocolVersion": newest_protocol_version()},
        }, token=token)
        check("right bearer -> 200 with serverInfo",
              status == 200 and init.get("result", {}).get("serverInfo", {}).get("name")
              == "digital-product-factory", f"status={status} body={str(init)[:200]}")

        status, listed = http_post(url, {"jsonrpc": "2.0", "id": 2, "method": "tools/list"},
                                   token=token)
        tools = [t["name"] for t in listed.get("result", {}).get("tools", [])]
        check("HTTP tools/list lists the same six tools", status == 200 and len(tools) == 6,
              str(tools))

        status, created = http_post(url, {
            "jsonrpc": "2.0", "id": 3, "method": "tools/call",
            "params": {"name": "dpf_create_product",
                       "arguments": {"title": "HTTP Acceptance Product", "stage": "review"}},
        }, token=token)
        payload, is_error = tool_result(created)
        check("HTTP create_product writes a record", status == 200 and not is_error
              and isinstance(payload, dict) and isinstance(payload.get("id"), int),
              str(created)[:300])
        http_id = (payload or {}).get("id")

        status, fetched = http_post(url, {
            "jsonrpc": "2.0", "id": 4, "method": "tools/call",
            "params": {"name": "dpf_get_product", "arguments": {"id": http_id}},
        }, token=token)
        read_back, is_error = tool_result(fetched)
        check("HTTP get_product reads it back", not is_error
              and (read_back or {}).get("title") == "HTTP Acceptance Product", str(fetched)[:300])
        check("HTTP create honours the requested stage", (read_back or {}).get("stage") == "review",
              str(read_back)[:200])
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            proc.kill()

    con = sqlite3.connect(db)
    try:
        rows = con.execute("select id, title from ideas where title = 'HTTP Acceptance Product'").fetchall()
    finally:
        con.close()
    check("the HTTP-created product is a real SQLite row", len(rows) == 1, str(rows))
    check_audit(audit, "serve", min_tool_calls=2, rejected=2)


def run_byok_suite(binary, workdir):
    """A key in --settings must reach the generator.

    Proven without spending anything or needing the network: with a key configured the call gets
    as far as resolving the template; with none it stops at "no AI provider key is configured".
    Only the second message means the settings were not read.
    """
    print("BYOK: the key in --settings actually reaches the generator")
    db = os.path.join(workdir, "byok.db")
    audit = os.path.join(workdir, "byok-audit.log")
    settings = write_config(os.path.join(workdir, "with-key.json"), "sk-acceptance-not-a-real-key")
    door = Stdio([binary, "mcp", "--db", db, "--audit", audit, "--settings", settings])
    try:
        init = door.call(1, "initialize", {"protocolVersion": newest_protocol_version()})
        check("BYOK: door starts with an explicit settings file", bool(init and init.get("result")))
        response = door.call(2, "tools/call", {
            "name": "dpf_generate_content",
            "arguments": {"template_id": "no-such-template-xyz", "params": {}},
        })
        payload, is_error = tool_result(response)
        message = json.dumps(payload).lower()
        check("BYOK: the configured key is used (fails on the template, not on the key)",
              is_error and "no ai provider key" not in message
              and "template" in message,
              str(payload)[:300])
    finally:
        door.close()


def run_refusals(binary, workdir):
    print("refusals that must never regress")
    key_file = os.path.join(workdir, "short-key.txt")
    with open(key_file, "w", encoding="utf-8") as f:
        f.write("short\n")

    result = subprocess.run(
        [binary, "serve", "--bind", "127.0.0.1:0", "--key-file", key_file, "--db",
         os.path.join(workdir, "r1.db")],
        capture_output=True, text=True, timeout=60,
    )
    check("a short key is refused", result.returncode != 0 and "24" in result.stderr,
          f"exit={result.returncode} stderr={result.stderr[-200:]}")

    good_key = os.path.join(workdir, "good-key.txt")
    with open(good_key, "w", encoding="utf-8") as f:
        f.write("acceptance-token-0123456789abcdef0123456789abcdef\n")
    result = subprocess.run(
        [binary, "serve", "--bind", "0.0.0.0:8899", "--key-file", good_key, "--db",
         os.path.join(workdir, "r2.db")],
        capture_output=True, text=True, timeout=60,
    )
    check("a non-loopback bind is refused",
          result.returncode != 0 and "loopback" in result.stderr.lower(),
          f"exit={result.returncode} stderr={result.stderr[-200:]}")

    result = subprocess.run(
        [binary, "serve", "--bind", "127.0.0.1:8899", "--db", os.path.join(workdir, "r3.db")],
        capture_output=True, text=True, timeout=60,
    )
    check("serve without --key-file is refused",
          result.returncode != 0 and "key-file" in result.stderr,
          f"exit={result.returncode} stderr={result.stderr[-200:]}")

    result = subprocess.run(
        [binary, "serve", "--bind", "127.0.0.1:8899", "--key-file",
         os.path.join(workdir, "missing-key.txt"), "--db", os.path.join(workdir, "r4.db")],
        capture_output=True, text=True, timeout=60,
    )
    check("a missing key file is refused",
          result.returncode != 0 and "cannot read key file" in result.stderr.lower(),
          f"exit={result.returncode} stderr={result.stderr[-200:]}")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--bin", default="target/debug/dpf")
    parser.add_argument("--keep", action="store_true", help="keep the scratch directory")
    args = parser.parse_args()

    binary = os.path.abspath(args.bin)
    if not os.path.isfile(binary):
        print(f"error: no binary at {binary} (cargo build first)", file=sys.stderr)
        return 2

    workdir = tempfile.mkdtemp(prefix="dpf-mcp-acceptance-")
    print(f"binary: {binary}\nscratch: {workdir}\n")
    try:
        run_stdio_suite(binary, workdir)
        print()
        run_http_suite(binary, workdir)
        print()
        run_byok_suite(binary, workdir)
        print()
        run_refusals(binary, workdir)
    finally:
        if args.keep:
            print(f"\nkept: {workdir}")
        else:
            shutil.rmtree(workdir, ignore_errors=True)

    print(f"\n{PASS} passed, {FAIL} failed")
    if FAIL:
        for failure in FAILURES:
            print(f"  - {failure}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
