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
  <strong>Native Rust desktop &amp; web UI built on <a href="https://github.com/j5onrf/rs-agent">rs-agent</a> for local GGUF and cloud models.</strong><br />
  Parallel micro-bots &nbsp;&middot;&nbsp; One-turn <code>/sub</code> agents &nbsp;&middot;&nbsp; Zero-trust sandbox &nbsp;&middot;&nbsp; Single binary
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

- **Single Native Binary:** Web/desktop UI compiled right in (`ui.html`).
- **Frameless Desktop App:** Opens via Chrome/Brave/Chromium `--app=` mode, Omarchy, or `railbot.desktop`.
- **Parallel Micro-Bots (`/call`):** Consult peer micro-bots read-only via `/call`.
- **One-Off Sub-Agents (`/sub`):** Injects `delegate_task` for one turn (`0t` overhead otherwise), clamped to depth `1`.
- **Zero-Trust Sandbox:** Composer shield toggles **Zero Trust** vs **Allow All** (`.env` locked).
- **Themes & Desktop Sync:** Live Omarchy/Noctalia sync, `+ Noctalia` catalog browser, and custom JSON palettes (`eldritch`, `darkbot`, `claude`).
- **Built on [rs-agent](https://github.com/j5onrf/rs-agent):** Listens on `http://127.0.0.1:24244` (`RAILBOT_PORT` in `.env`); see [rs-agent](https://github.com/j5onrf/rs-agent) for core engine.

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
