# Digital Product Factory

A high-performance Rust desktop application for creating, managing, and selling digital products. Built with egui for native speed.

## Features

### Core Modules

- **Pipeline** — Kanban-style product tracking across 7 stages
- **Create** — 20+ digital product templates with AI generation
- **Market Research** — Search Etsy, Gumroad, Amazon for product validation
- **Contract Generator** — NDA, Service Agreements, Coaching Contracts and more
- **Export** — 7 formats (Markdown, HTML, PDF, DOCX, XLSX, JSON, ZIP)
- **Templates** — Browse and manage product templates
- **Adverts & Campaign Suite** — Multi-format ad generation with AI copy (PAS/AIDA/BAB), 3 aspect ratios (Square, Story, Landscape), conversion scoring, campaign management, JSON export
- **Product Variants** — Multiple variants per product with version history

### Business Modules (Team+)

- **Analytics** — Revenue tracking, sales records, CSV export
- **Publishing** — Marketplace publishing (Etsy, Gumroad) with credential management
- **Bundles** — Bundle products with discount pricing
- **Scheduler** — Automate tasks (generation, publishing, research)
- **Industry Presets** — 9 pre-configured workflows
- **QC Checklist** — Pre-publish quality checks and duplicate detection
- **Asset Library** — Local media manager with version tracking
- **Webhooks** — HTTP webhook server for external tools

### Advanced (Agency+)

- **Adverts & Campaign Suite** — Campaign planning, AI creative generation, conversion scoring
- **Compliance Scanner** — AI disclosure rules and denylist checks
- Client management
- Custom integrations

## Quick Start

\\\ash
cargo run
\\\

## Configuration Files

| File | Purpose |
|------|---------|
| `feature_tiers.json` | Feature-to-license-tier mapping |
| `platform_formats.json` | Marketplace format requirements |
| `revoked_keys.json` | Revoked licence keys, refused at activation |

There is deliberately no `pricing.json`: prices live on the sales page, never in the software.
## Inline Help

Press **❓ Help** in the status bar or click **?** next to any section header for contextual help. Full help index available from the status bar button. All license tiers can access all help content.

## License Tiers

| Tier | Users | Key Features |
|------|-------|--------------|
| Personal | 1 | Pipeline, Create, Research, Contracts, Export |
| Team | 5 | Personal + Analytics, Publishing, Bundles, Scheduler |
| Agency | 20 | Team + Client Management |
| Enterprise | Unlimited | All features |

Prices are not part of the software: they live on the sales page, because a licence may be a
one-time payment or a subscription.
## Project Structure

\\\
dpf/
  src/
    main.rs
    app.rs              # Application state and UI routing
    adverts.rs           # Advert & Campaign domain types
    advert_generator.rs  # AI ad creative generation
    advert_export.rs     # Advert JSON export
    config.rs           # App configuration
    database.rs         # SQLite database layer
    inline_help.rs      # Inline help system
    pipeline.rs         # Kanban pipeline
    product_generator.rs # AI product generation
    license_manager.rs   # License key management
    llm_router.rs       # AI model routing
    research.rs         # Market research
    scheduler.rs        # Task scheduling
    bundler.rs          # Product bundling
    exporter.rs         # Export to various formats
    contract_generator.rs # Legal contract generation
    presets.rs          # Industry presets
    analytics.rs        # Sales analytics
    publishing.rs       # Marketplace publishing
    db_ext.rs           # Database extensions
    ui/
      mod.rs            # UI module routing
      main_content.rs   # Tab routing
      sidebar.rs        # Navigation sidebar
      status_bar.rs     # Bottom status bar
      pipeline_view.rs  # Pipeline Kanban UI
      dashboard_view.rs # Dashboard UI
      create_view.rs    # Create product UI
      settings_dialog.rs # Settings UI
      license_dialog.rs # License entry UI
      analytics_view.rs # Analytics UI
      publish_view.rs   # Publishing UI
      bundle_view.rs    # Bundle management UI
      scheduler_view.rs # Scheduler UI
      contract_view.rs  # Contract generator UI
      research_view.rs  # Market research UI
      presets_view.rs   # Industry presets UI
      components.rs     # Shared UI components
  mcp/                  # Headless MCP door (dpf mcp / dpf serve)
    mod.rs              #   transports, key-file rules, credential check
    engine.rs           #   the tools — same engine calls as the screens
    settings.rs         #   BYOK key discovery (app settings, or env vars)
  scripts/
    mcp_acceptance.py   # 48-check proof: stdio, HTTP, BYOK, refusals, audit, SQLite
  GUIDE.md              # User guide
  ADMIN_GUIDE.md        # Admin guide
  README.md             # This file
\\\

## Driving DPF from an agent (headless MCP)

The app can be driven by an AI agent over the **Model Context Protocol (MCP)**. It runs headless:
no window, no display, and on Windows nothing visible at all. The agent works with the same
database, the same templates and the same generator the screens use — an agent-made product shows
up in your Pipeline exactly like one you made yourself.

Two forms, one door:

### 1. stdio (what desktop MCP clients use)

Point the client at the executable with the `mcp` subcommand. There is no port and no token to
manage: the client already runs as you, so the process boundary is the credential.

```json
{
  "mcpServers": {
    "digital-product-factory": {
      "command": "/path/to/dpf",
      "args": ["mcp"]
    }
  }
}
```

On Windows the command is the full path to `dpf.exe` (for example `C:\DigitalProductFactory\dpf.exe`).
The same subcommand works from a terminal: `dpf mcp`.

### 2. localhost HTTP

```bash
openssl rand -hex 24 > ~/.dpf-mcp-key      # the bearer token, kept in a file
chmod 600 ~/.dpf-mcp-key                   # owner-only
dpf serve --bind 127.0.0.1:8765 --key-file ~/.dpf-mcp-key
```

Then `POST http://127.0.0.1:8765/mcp` with `Authorization: Bearer <token>` and a JSON-RPC body.
`GET /healthz` answers without a token so a supervisor can tell it is up.

### Key-file rules (enforced, not just documented)

- `serve` **refuses to start without `--key-file`**, and refuses any bind that is not loopback
  (`127.0.0.1` or `::1`). A DPF door is never reachable from another machine.
- The token is the **first non-empty line that does not start with `#`**, trimmed.
- At least **24 characters**. A short "password" on localhost is still a password.
- No whitespace inside the token (one line, one value).
- On Linux/macOS the file should be `0600`; anything looser is reported as a warning on startup.
- The token is compared in constant time and is **never** written to a log, the audit file or the
  screen.

### Where the AI keys come from (your own key, BYOK)

Generation and the strategy brief use **your** provider key. Resolution order:

1. `--settings PATH` — the app's `app.ron`, or a RON/JSON settings file (see below);
2. otherwise the app's own saved settings, found automatically — so **a key you already entered in
   Settings just works**, with nothing to configure for the agent;
3. `DPF_OPENAI_KEY`, `DPF_ANTHROPIC_KEY`, `DPF_GOOGLE_KEY`, `DPF_DEEPSEEK_KEY`,
   `DPF_MOONSHOT_KEY` (**always win** over a file value; handy for a key you would rather not keep
   on disk).

There is no baked-in key and no shared key: if you have not configured a provider, generation fails
with a message telling you to add one, and nothing is spent.

### Options

| Option | Applies to | Meaning |
|--------|-----------|---------|
| `--db PATH` | `mcp`, `serve` | Database to use. Default: the app's own. Point it at a copy to let an agent work without touching your live data. |
| `--settings PATH` | `mcp`, `serve` | Settings file holding your AI keys. |
| `--key-file PATH` | `serve` (required), `mcp` (optional) | Bearer token file. |
| `--audit PATH` | `mcp`, `serve` | Audit log. Default: `mcp-audit.log` beside the database. |
| `--bind ADDR` | `serve` | Must be loopback. Default `127.0.0.1:8765`. |

`dpf --help` prints the same list.

### The tools

| Tool | What it does |
|------|--------------|
| `dpf_list_products` | List your products (pipeline projects), newest activity first; optional `stage` filter. |
| `dpf_get_product` | Read one product in full by id, including the notes where generated content lives. |
| `dpf_create_product` | Create a product from a title and optional details, then read the stored row back. |
| `dpf_list_templates` | The built-in product templates and their parameters. |
| `dpf_generate_content` | Generate product content with your own key — the same generator the Create screen uses. Optionally saves the result as a product. |
| `dpf_strategy_brief` | The deep-thinking niche brief (one model call): what to build, pricing, and one explicit do-not-build. |

Generation and the strategy brief are marked as **writes** on purpose: they spend your provider
credit, so a well-behaved client asks you before calling them.

### What is deliberately absent

There is no tool that **publishes or lists** a product, **delivers a paid product**, **charges**
anything, **records a sale**, or **touches the licence** (activation, deactivation, revocation).
Those stay off the agent's surface until the payment hold is lifted; a missing tool cannot be
misused, and an agent cannot be talked into publishing something you did not approve.

### Audit

Every tool call is written to the audit log (default `mcp-audit.log` beside the database) as one
JSON line: time, tool, arguments (truncated, secrets redacted), outcome, duration. Rejected
credentials are logged too, under the subject `-`. The bearer token never appears in it.

### Verify it yourself

```bash
cargo build                      # debug binary
python3 scripts/mcp_acceptance.py            # 48 checks: stdio, HTTP, BYOK, refusals, audit, SQLite
```

The script starts the real binary against a scratch database, completes `initialize` →
`tools/list` → a real `dpf_create_product` → `dpf_get_product` read-back, then confirms the row
directly in SQLite and repeats the round trip over loopback HTTP.

## Requirements

- Rust 2021 edition
- SQLite (bundled)
- API keys (user-provided): OpenAI, Anthropic, Google, DeepSeek, and/or Moonshot

## Build

\\\ash
cargo build --release
\\\

## Documentation

- **User Guide:** \GUIDE.md\
- **Admin Guide:** \ADMIN_GUIDE.md\
- **Inline Help:** Press ❓ in the app status bar

*Version 1.4.2 - Native Rust Edition*
