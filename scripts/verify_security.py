"""Local end-to-end checks with an isolated temporary DB and disabled external SMTP.

Run after cargo build --locked: python scripts/verify_security.py
"""
import json
import os
from pathlib import Path
import secrets
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("FASZIEN_TEST_BINARY", ROOT / "target" / "debug" / ("faszienbehandlung.exe" if os.name == "nt" else "faszienbehandlung")))


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def main():
    assert BINARY.is_file(), "Run cargo build --locked first."
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        port = listener.getsockname()[1]
    base = f"http://127.0.0.1:{port}"
    sentinel = "secret-must-never-appear-in-access-logs"
    with tempfile.TemporaryDirectory(prefix="faszien-security-") as directory:
        work = Path(directory).resolve()
        assert work.parent == Path(tempfile.gettempdir()).resolve()
        database = work / "faszien-marketing-preview.db"
        env = dict(os.environ, HOST="127.0.0.1", PORT=str(port), BASE_URL=base,
                   DATABASE_URL=str(database), JWT_SECRET=secrets.token_hex(32), JWT_EXPIRY_HOURS="24",
                   ADMIN_EMAIL="preview@example.test", ADMIN_PASSWORD="preview-local-only",
                   SMTP_HOST="127.0.0.1", SMTP_PORT="1", SMTP_USER="test", SMTP_PASS="test",
                   SMTP_FROM="preview@example.test", SMTP_FROM_NAME="Local test", RUST_LOG="info")
        log_path = work / "server.log"
        opener = urllib.request.build_opener(NoRedirect)

        def call(path, data=None, headers=None, method=None):
            request_headers = {"Origin": base, "Content-Type": "application/json"}
            request_headers.update(headers or {})
            body = None if data is None else json.dumps(data).encode()
            request = urllib.request.Request(base + path, data=body, headers=request_headers, method=method)
            try:
                response = opener.open(request, timeout=10)
            except urllib.error.HTTPError as error:
                response = error
            with response:
                return response.status, response.read().decode(), response.headers

        with log_path.open("w", encoding="utf-8") as log:
            server = subprocess.Popen([str(BINARY)], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT,
                                      creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0)
            try:
                for _ in range(100):
                    assert server.poll() is None, "Local server exited during startup; inspect configuration."
                    try:
                        if call("/")[0] == 200:
                            break
                    except (urllib.error.URLError, TimeoutError):
                        pass
                    time.sleep(0.1)
                else:
                    raise AssertionError("Local server did not start.")

                status, _, headers = call("/reset-password/" + sentinel, headers={"Referer": base + "/?token=" + sentinel})
                assert status == 200
                for header, expected in [("Cache-Control", "no-store"), ("Referrer-Policy", "no-referrer"),
                                         ("X-Content-Type-Options", "nosniff"), ("X-Frame-Options", "DENY")]:
                    assert headers[header] == expected, (header, headers)
                assert call("/api/auth/login", {}, headers={"Origin": "https://attacker.invalid"})[0] == 403
                assert call("/api/auth/login", {"password": "x" * 70000})[0] == 413
                assert call("/admin/nextcloud/disconnect")[0] in (404, 405)
                assert call("/api/customer/export-data")[0] == 401
                assert call("/api/admin/settings", {})[0] == 303

                status, _, headers = call("/api/auth/login", {"email": "preview@example.test", "password": "preview-local-only"})
                assert status == 200
                cookie = headers["Set-Cookie"]
                assert "HttpOnly" in cookie and "SameSite=Lax" in cookie and "Secure" not in cookie
                for path in ("/", "/portal", "/admin", "/admin/faq", "/admin/settings"):
                    assert call(path, headers={"Cookie": cookie.split(";", 1)[0]})[0] == 200, path
                print("PASS: real server startup, authenticated pages, cookie flags, CSRF, body limits and headers")

                check_script = "verify_compliance.py" if "--compliance" in sys.argv else "verify_marketing.py"
                subprocess.run([sys.executable, str(ROOT / "scripts" / check_script),
                                "--database", str(database), "--base-url", base], cwd=ROOT, check=True, timeout=60)

                statuses = [call("/api/auth/login", {"email": "nobody@example.test", "password": "wrong"})[0] for _ in range(21)]
                assert 429 in statuses and statuses[-1] == 429
                print("PASS: authentication rate limits are enforced by the running application")
            finally:
                server.terminate()
                try:
                    server.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    server.kill()
                    server.wait(timeout=5)
        assert sentinel not in log_path.read_text(encoding="utf-8"), "Sensitive URL leaked into access log"
        print("PASS: access logs omit bearer URLs, query strings and Referer")
    print("All local security smoke checks passed; temporary database and logs removed.")


if __name__ == "__main__":
    main()
