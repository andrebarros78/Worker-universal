import { createServer } from "node:http";
import type { Server } from "node:http";

export class SyntheticFixtureServer {
  private server?: Server;
  private base = "";

  async start(): Promise<string> {
    this.server = createServer((req, res) => {
      const url = new URL(req.url ?? "/", "http://127.0.0.1");

      if (url.pathname === "/download") {
        res.writeHead(200, {
          "Content-Type": "text/plain; charset=utf-8",
          "Content-Disposition": 'attachment; filename="receipt.txt"',
        });
        res.end("receipt:synthetic\n");
        return;
      }

      if (url.pathname === "/login") {
        res.writeHead(200, {
          "Content-Type": "text/html; charset=utf-8",
          "Set-Cookie": "tma_session=valid; Path=/; SameSite=Lax",
        });
        res.end(page("Login", '<div data-testid="login-status">Logged in</div>'));
        return;
      }

      if (url.pathname === "/session") {
        const authenticated = (req.headers.cookie ?? "").includes("tma_session=valid");
        res.writeHead(200, { "Content-Type": "text/html; charset=utf-8" });
        res.end(
          page(
            "Session",
            '<div data-testid="session-status">' +
              (authenticated ? "Authenticated" : "Guest") +
              "</div>",
          ),
        );
        return;
      }

      if (url.pathname === "/drift") {
        const version = url.searchParams.get("v") === "2" ? "v2" : "v1";
        const id = version === "v2" ? "continue-renamed" : "continue-original";
        res.writeHead(200, { "Content-Type": "text/html; charset=utf-8" });
        res.end(
          page(
            "Drift",
            '<button id="' +
              id +
              '" onclick="document.getElementById(\'drift-status\').textContent=\'continued\'">Continue</button>' +
              '<div id="drift-status"></div>',
          ),
        );
        return;
      }

      if (url.pathname === "/form") {
        res.writeHead(200, { "Content-Type": "text/html; charset=utf-8" });
        res.end(
          page(
            "Form",
            [
              '<label for="full-name-new">Full name</label>',
              '<input id="full-name-new" placeholder="Your full name">',
              '<label for="attachment-new">Attachment</label>',
              '<input id="attachment-new" type="file">',
              '<div data-testid="upload-status"></div>',
              '<button id="submit-new">Submit mission</button>',
              '<div data-testid="submit-status"></div>',
              '<a href="/download">Download receipt</a>',
              '<script>',
              "document.getElementById('attachment-new').addEventListener('change', function(e) {",
              "  document.querySelector('[data-testid=upload-status]').textContent = e.target.files[0]?.name || '';",
              "});",
              "document.getElementById('submit-new').addEventListener('click', function() {",
              "  const value = document.getElementById('full-name-new').value;",
              "  document.querySelector('[data-testid=submit-status]').textContent = 'Submitted: ' + value;",
              "});",
              "</script>",
            ].join(""),
          ),
        );
        return;
      }

      res.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
      res.end("not found");
    });

    await new Promise<void>((resolve, reject) => {
      this.server?.once("error", reject);
      this.server?.listen(0, "127.0.0.1", () => resolve());
    });

    const address = this.server.address();
    if (!address || typeof address === "string") {
      throw new Error("fixture_server_address_unavailable");
    }
    this.base = "http://127.0.0.1:" + address.port;
    return this.base;
  }

  url(path: string): string {
    if (!this.base) throw new Error("fixture_server_not_started");
    return this.base + path;
  }

  async stop(): Promise<void> {
    const server = this.server;
    this.server = undefined;
    if (!server) return;
    await new Promise<void>((resolve, reject) => {
      server.close((error) => (error ? reject(error) : resolve()));
    });
  }
}

function page(title: string, body: string): string {
  return (
    "<!doctype html><html><head><meta charset=\"utf-8\"><title>" +
    title +
    "</title></head><body>" +
    body +
    "</body></html>"
  );
}
