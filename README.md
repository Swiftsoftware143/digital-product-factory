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
- Whitelabel branding
- Client management
- Custom integrations

## Quick Start

\\\ash
cargo run
\\\

## Configuration Files

| File | Purpose |
|------|---------|
| \eature_tiers.json\ | Feature-to-license-tier mapping |
| \pricing.json\ | Pricing and plan details |
| \platform_formats.json\ | Marketplace format requirements |
| \
evoked_keys.json\ | Revoked license keys list |

## Inline Help

Press **❓ Help** in the status bar or click **?** next to any section header for contextual help. Full help index available from the status bar button. All license tiers can access all help content.

## License Tiers

| Tier | Price | Users | Key Features |
|------|-------|-------|--------------|
| Personal | Free | 1 | Pipeline, Create, Research, Contracts, Export |
| Team | \/mo | 5 | Personal + Analytics, Publishing, Bundles, Scheduler |
| Agency | \/mo | 20 | Team + Whitelabel, Client Management |
| Enterprise | \/mo | Unlimited | All features + API access |

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
  GUIDE.md              # User guide
  ADMIN_GUIDE.md        # Admin guide
  README.md             # This file
\\\

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
