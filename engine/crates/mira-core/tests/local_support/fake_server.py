#!/usr/bin/env python3
import json, os, sys, time, threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

opts = {}
argv = sys.argv[1:]
i = 0
while i < len(argv):
    if argv[i].startswith("--") and i + 1 < len(argv) and not argv[i + 1].startswith("--"):
        opts[argv[i]] = argv[i + 1]
        i += 2
    else:
        opts[argv[i]] = True
        i += 1

log = opts.get("--fake-log")
key = open(opts["--api-key-file"]).read().strip() if "--api-key-file" in opts else ""
if log:
    with open(log, "a") as f:
        f.write(json.dumps({"event": "start", "pid": os.getpid(), "alias": opts.get("--alias"), "argv": argv, "key": key}) + "\n")
if "--fake-exit-code" in opts:
    sys.exit(int(opts["--fake-exit-code"]))
ready_at = time.time() + int(opts.get("--fake-ready-after-ms", "0")) / 1000.0
hang = "--fake-hang" in opts


class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def reply(self, code, body=b"", ctype="application/json"):
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        if self.path == "/health":
            if hang or time.time() < ready_at:
                return self.reply(503, b'{"error":{"code":503}}')
            return self.reply(200, b'{"status":"ok"}')
        if self.path == "/die":
            os._exit(1)
        self.reply(404)

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(length)
        if self.headers.get("Authorization") != "Bearer " + key:
            return self.reply(401, b'{"error":"bad key"}')
        if log:
            with open(log, "a") as f:
                f.write(json.dumps({"event": "chat", "body": body.decode()}) + "\n")
        chunk = {"choices": [{"index": 0, "delta": {"content": "ok"}, "finish_reason": None}]}
        done = {"choices": [{"index": 0, "delta": {}, "finish_reason": "stop"}]}
        data = "data: %s\n\ndata: %s\n\ndata: [DONE]\n\n" % (json.dumps(chunk), json.dumps(done))
        self.reply(200, data.encode(), "text/event-stream")


server = ThreadingHTTPServer((opts.get("--host", "127.0.0.1"), int(opts["--port"])), Handler)
server.serve_forever()
