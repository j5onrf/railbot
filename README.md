<div align="center">

<h1>
  <img alt="RailBot Logo" src="assets/railbot.svg" width="38" height="38" align="absmiddle" />
  RailBot
</h1>

<p>
  <a href="https://github.com/j5onrf/railbot/releases"><img src="https://shieldcn.dev/badge/version-v0.5.3.svg?variant=secondary" alt="Version"></a>
  <a href="https://github.com/j5onrf/railbot"><img src="https://shieldcn.dev/badge/Rust.svg?variant=branded&brand=rust" alt="Rust"></a>
  <a href="https://github.com/j5onrf/railbot/blob/main/LICENSE"><img src="https://shieldcn.dev/badge/license-MIT.svg?variant=secondary" alt="License"></a>
  <a href="https://j5onrf.github.io/railbot/"><img src="https://shieldcn.dev/badge/demo-live-22c55e.svg?variant=secondary" alt="Live Demo"></a>
</p>

<p>
  <strong>Sub-millisecond native Rust AI desktop &amp; web harness for local GGUF models and frontier cloud endpoints.</strong><br />
  Parallel micro-bots &nbsp;•&nbsp; Local-first private &nbsp;•&nbsp; Zero-trust sandbox &nbsp;•&nbsp; Single binary with embedded UI
</p>

<p>
  <a href="https://j5onrf.github.io/railbot/">
    <img src="assets/banner.png" alt="RailBot — Parallel Micro-Bots on High-Speed Rails" width="100%" />
  </a>
</p>

### <a href="https://j5onrf.github.io/railbot/">Launch Interactive Web Demo &rarr;</a>

</div>

---

## Highlights

- **Single Native Binary:** Zero runtime dependencies; web/desktop interface compiled right in (`ui.html`).
- **Frameless Desktop App:** Launch instantly with `railbot.desktop` via Chrome/Brave/Chromium `--app=` mode or Omarchy.
- **Parallel Micro-Bots (`/call`):** Consult any peer bot read-only. Assign per-bot cloud providers in Settings → Projects and auto-collaborators in the bot badge menu.
- **Zero-Trust Sandbox:** Security boundary traps directory traversals, blocks `.env` reads/writes, and gates mutating commands.
- **Themes & Desktop Sync:** themes (`eldritch`, `claude`), custom JSON palettes, live Omarchy/Noctalia desktop sync, and a built-in palette catalog browser.
- **Application:** RailBot is running on `http://127.0.0.1:24244` and built directly on <a href="https://j5onrf.github.io/rs-agent/">rs-agent</a>.

---

## Quickstart

```bash
git clone https://github.com/j5onrf/railbot.git ~/.config/railbot
cd ~/.config/railbot
cargo build --release
./launch.sh
```

Launch options:
- `./launch.sh` — opens standalone frameless app window
- `./launch.sh web` — opens in your default browser
- `./launch.sh [path]` — opens workspace folder directly

---

<div align="center">
  <a href="LICENSE">MIT License</a>
</div>
