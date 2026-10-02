# Digital Product Factory — Administrator Guide

## Overview

This guide is for administrators managing licence keys, feature gating, and platform
configuration for the Digital Product Factory desktop app. The app uses an **offline
licence key** system — there is **no licence server** and no internet call during
activation. Validation is local: key shape, tier token, a check code, and a revocation
list file.

**Read the security model before relying on this as copy protection.** Offline
validation gates the app honestly for honest users, but it is **not tamper-proof**.
Anyone who can patch the binary can bypass the check. Real enforcement needs either
online activation (which also makes revocation at scale possible) or code signing +
notarisation so a patched binary fails to launch. Do not describe this system as secure
DRM.

---

## License Architecture

### How Activation Works

Activation is implemented in `src/license_manager.rs` and runs entirely offline. On
**Activate**, `LicenseManager::activate()` performs these steps in order:

1. **Shape** — the key must split on `-` into exactly four segments with the first
   segment `DPF`. Otherwise it is refused.
2. **Tier token** — segment 2 must be `PERSONAL`, `TEAM`, `AGENCY`, or `ENTERPRISE`.
   Any other value is refused.
3. **Check code** — the app computes an **FNV-1a (64-bit)** hash over the string
   `"<TIER>-<BLOCK><SALT>"`, where `SALT = "dpf-1.4.2-license"` (`SALT` is a
   compile-time constant in `license_manager.rs`). The hash is rendered as **four
   characters** drawn from the alphabet `ABCDEFGHJKLMNPQRSTUVWXYZ23456789`. That
   4-character value must equal segment 4. A single wrong character is rejected.
4. **Revocation** — the full key is compared (case-insensitively) against the
   `revoked_keys` array in `revoked_keys.json`. A listed key is refused.

On success the licence is persisted to the SQLite `licenses` table so activation
survives a restart; `LicenseManager::new()` reloads an active licence at startup.
Deactivating removes the row and returns the app to the free tier.

### License Tiers

Tier is encoded in the key and enforced by the feature list for that tier.

| Tier | Key token | Seats | Price | Features unlocked (count) |
|------|-----------|-------|-------|---------------------------|
| **Personal** (default/free) | `PERSONAL` | 1 | Free | 8 |
| **Team** | `TEAM` | 5 | $29/month | 19 |
| **Agency** | `AGENCY` | 20 | $99/month | 23 |
| **Enterprise** | `ENTERPRISE` | Unlimited | $299/month | 25 |

An install with no active licence sits on the free **Personal** tier. That default is
what makes gating work with no key present.

---

## Feature Gating

All **22 sidebar tabs** are gated on the active tier's feature list. A tab the current
tier does not include is rendered **dimmed with a 🔒**, cannot be opened, and — when
clicked — opens the licence dialog naming the plan that unlocks it
(`LicenseManager::required_tier_for`). The mapping from tab to feature slug lives in
`src/ui/sidebar.rs`.

**`feature_tiers.json` is the source of truth.** The gating reads it at runtime. The
built-in table in `license_manager.rs` is only a fallback when that file is missing or
malformed. See the warning in *Configuration Files* below.

### Feature lists by tier

Each tier is a strict **superset** of the tier below it (enforced by the
`tiers_are_supersets` test). The lists below are copied from `feature_tiers.json`.

**Personal** (8): `pipeline`, `ai_generation`, `templates`, `market_research`,
`contract_generator`, `export`, `presets`, `variants`

**Team** (19) — all Personal features plus: `analytics`, `publishing`, `bundles`,
`scheduler`, `adverts`, `qc`, `assets`, `webhooks`, `mockup_compositor`,
`logo_generator`, `vector_generator`

**Agency** (23) — all Team features plus: `whitelabel`, `client_management`,
`compliance`, `custom_integrations`

**Enterprise** (25) — all Agency features plus: `api_access`, `admin_panel`

### Tab → feature slug

| Feature slug | Sidebar tab(s) |
|--------------|----------------|
| `pipeline` | Dashboard, Pipeline, Settings |
| `ai_generation` | Create |
| `templates` | Templates |
| `market_research` | Research |
| `contract_generator` | Contracts |
| `presets` | Presets |
| `variants` | Variants |
| `mockup_compositor` | Mockups |
| `bundles` | Bundles |
| `scheduler` | Scheduler |
| `analytics` | Analytics |
| `publishing` | Publish |
| `qc` | QC Checklist |
| `assets` | Asset Library |
| `webhooks` | Webhooks |
| `adverts` | Adverts |
| `logo_generator` | Logo Generator |
| `vector_generator` | Vector Generator |
| `compliance` | Compliance |
| `admin_panel` | Admin |

---

## Configuration Files

All licence and feature configuration files are read from the **app's working
directory** (the directory the binary is launched from). They are external JSON files —
no rebuild needed to change tiers, pricing, or platform formats.

### feature_tiers.json

Tier → feature list. **This is the gating source of truth.**

Structure:

```json
{
  "tiers": {
    "personal":   { "name": "Personal",   "price": 0,   "period": "free",  "devices": 1,  "features": ["pipeline", "..."] },
    "team":       { "name": "Team",       "price": 29,  "period": "month", "devices": 5,  "features": ["pipeline", "..."] },
    "agency":     { "name": "Agency",     "price": 99,  "period": "month", "devices": 20, "features": ["pipeline", "..."] },
    "enterprise": { "name": "Enterprise", "price": 299, "period": "month", "devices": -1, "features": ["pipeline", "..."] }
  }
}
```

**Warning:** the reader does `v.get("tiers")` first. If the top-level key is renamed or
misspelled, or the file is corrupt, the lookup fails and the app **silently falls back**
to the built-in table in `license_manager.rs`. Editing this file therefore has no effect
unless the first key is exactly `"tiers"` and the JSON parses. Verify any edit by
re-launching and re-checking which tabs are locked.

### pricing.json

Tier prices and billing periods, read for display. The real file is minimal:

```json
{
  "personal":   { "price": 0,   "period": "free" },
  "team":       { "price": 29,  "period": "month" },
  "agency":     { "price": 99,  "period": "month" },
  "enterprise": { "price": 299, "period": "month" }
}
```

### platform_formats.json

Per-marketplace publishing limits used by QC validation and the publishing flow. Each
entry carries `name`, `thumbnail_width`/`thumbnail_height`, `max_file_size_mb`,
`max_tags`, `max_title_length`, `max_description_length`, `allowed_formats`, and
`digital_download`. Current contents:

| Key | Name | Thumbnail | Max file (MB) | Max tags | Title len | Desc len | Allowed formats |
|-----|------|-----------|---------------|----------|-----------|----------|-----------------|
| `etsy` | Etsy | 3000×3000 | 20 | 13 | 140 | 5000 | pdf, zip, png, jpg |
| `gumroad` | Gumroad | 1280×720 | 50 | 0 | 255 | 10000 | pdf, zip, epub, mp4 |
| `shopify` | Shopify | 2048×2048 | 20 | 0 | 255 | 5000 | pdf, zip, jpg, png |
| `payhip` | Payhip | 1200×1200 | 25 | 0 | 150 | 4000 | pdf, zip, jpg, png |

(A `max_tags` of `0` means no tag field is used for that marketplace.)

### revoked_keys.json

```json
{ "revoked_keys": [] }
```

Keys listed in this array are refused at activation. Match is case-insensitive and
trimmed.

**Revocation procedure and its limitation.** Add the key string to the `revoked_keys`
array (via the Admin panel Revocations section, or by editing the file directly and
saving). Revocation is checked **only at activation time**. Consequences:

- A key that has never been activated on a machine is refused once listed.
- A key that is **already activated continues to work** until it is deactivated and
  re-activated. Revocation does not retroactively kick out a running install.

State this limitation honestly to customers. Distributing a new `revoked_keys.json`
only affects installs that re-activate.

---

## Adverts & Campaign Suite — Admin

The Adverts module is gated at the **Team** tier or higher via the `adverts` feature
slug. Personal-tier users see the tab dimmed with a 🔒 and get the upgrade prompt.
There is no separate `advert_export` feature slug — JSON export of adverts is covered by
the generic `export` feature.

### Database Tables

The SQLite database has two tables for the Adverts module:

**`campaigns`:** `id` (INTEGER PK), `name` (TEXT NOT NULL), `description` (TEXT),
`goal` (TEXT — lead_generation, brand_awareness, sales_conversion, promo_sale),
`product_id` (INTEGER), `product_name` (TEXT), `target_audience` (TEXT),
`landing_url` (TEXT), `platform` (TEXT), `created_at` (TEXT ISO 8601).

**`adverts`:** `id` (INTEGER PK), `campaign_id` (INTEGER), `product_id` (INTEGER),
`product_name` (TEXT), `format` (TEXT — square_1_1, story_9_16, landscape_16_9),
`copy_framework` (TEXT — pas, aida, bab, social_proof, benefit_driven), `headline`,
`subheadline`, `body_text`, `cta_text`, `tagline`, `brand_voice`, `brand_colors`
(JSON hex array), `product_scale` (REAL 0.3–1.0), `product_position` (TEXT — center,
left, right), `product_rotation` (REAL degrees), `background_prompt`, `concept_name`,
`conversion_score` (INTEGER 0–100), `score_reasoning`, `status` (draft, ready),
`created_at` (ISO 8601), `layout_specs_json` (TEXT).

### Generation

Advert copy generation uses the LLM provider configured in Settings (up to all five
providers — see *Admin Panel*). The prompt includes product context, copy-framework
instructions, aspect-ratio constraints, and brand-identity extraction.

### Troubleshooting

- **Generation fails:** confirm an LLM API key is configured in Settings and has
  credits; check connectivity. The error is logged in the database.
- **Export produces empty files:** the advert must have `status` of draft/ready with copy
  populated; check disk space and write permission to the exports folder.
- **Preview looks wrong:** verify the pipeline product exists with a thumbnail, and
  re-generate the concept if the `layout_specs_json` is corrupt.

---

## Key Generation

### Key format

```
DPF-<TIER>-<BLOCK>-<CCCC>
```

| Segment | Meaning |
|---------|---------|
| `DPF` | Fixed product prefix. |
| `TIER` | `PERSONAL` \| `TEAM` \| `AGENCY` \| `ENTERPRISE`. |
| `BLOCK` | 8 characters, `A–Z0–9`. Use it to carry the customer or order id. |
| `CCCC` | The 4-character FNV-1a check code (see below). |

The check-code alphabet deliberately **excludes `I`, `O`, `0` and `1`** so keys cannot
be mis-transcribed between look-alike characters.

### Minting keys — the real tool

Keys are minted with the Python tool:

```
/opt/swift/scripts/dpf-mint-license.py
```

Usage:

```bash
python3 dpf-mint-license.py TEAM AB12CD34     # mint one key for a given block
python3 dpf-mint-license.py --batch TEAM 5     # mint 5 keys with random blocks
python3 dpf-mint-license.py --tiers            # print the tier table
```

The Python tool and the Rust app implement the **identical** FNV-1a algorithm over
`"<TIER>-<BLOCK>" + SALT` with `SALT = "dpf-1.4.2-license"`. This equivalence is
**pinned by a unit test** (`mint_matches_python`): if the two implementations ever
drift, that test fails and **every newly minted key is dead on arrival**. Never edit the
salt or the alphabet in one implementation without changing the other.

Verified worked examples (all reproduced live from the tool):

| Command | Resulting key |
|---------|---------------|
| `TEAM AB12CD34` | `DPF-TEAM-AB12CD34-P9SU` |
| `AGENCY ZZ99YY88` | `DPF-AGENCY-ZZ99YY88-R5QY` |
| `TEAM DEMO1234` | `DPF-TEAM-DEMO1234-2P7A` |
| `AGENCY DEMO5678` | `DPF-AGENCY-DEMO5678-8RVQ` |
| `ENTERPRISE DEMO9012` | `DPF-ENTERPRISE-DEMO9012-64XS` |

Rust-side minting uses `license_manager::mint_key(&tier, block)`, which shares the same
`check_code` definition.

### Validating keys

When a user enters a key, `activate()`:

1. Checks the `DPF-…` four-segment shape and a recognised tier token.
2. Recomputes the FNV-1a check code and compares it to segment 4.
3. Checks the key against `revoked_keys.json`.
4. On success, persists the licence and unlocks the tier's feature list.

### Revoking keys

Add the key string to `revoked_keys.json` (or use the Admin panel). Remember the
activation-time-only limitation described under *Configuration Files*.

---

## Admin Panel

The Admin panel is a sidebar tab gated on the `admin_panel` feature, which is
**Enterprise only**. It exposes five sections (`src/ui/admin_view.rs`):

| Section | What it does |
|---------|--------------|
| **Feature Tiers** | JSON editor for the tier definitions. Edits the in-memory `feature_tiers.json` content. |
| **Pricing** | JSON editor for tier prices/periods; **Save** writes the config. |
| **Platform Formats** | JSON editor for the per-marketplace limits. |
| **License Keys** | Tier dropdown + device slider + **Generate Key**. See caveat below. |
| **Revocations** | Enter a key and **Revoke** it; lists current revoked keys. Writes `revoked_keys.json`. |

There is **no database viewer** and no separate “view all generated keys” list — the
admin panel edits the four JSON config files and the revocation list only.

### Caveat: the Admin panel key generator is not the real minting path

The Admin panel's **License Keys** section calls `AdminState::generate_key()`, which
emits a **legacy 5-segment key** of the form
`DPF-{P|T|A|E}-{RAND4}-{RAND4}-{SUM}` (a hex byte-sum check, not FNV-1a). The current
activation path requires a **4-segment** `DPF-<TIER>-<BLOCK>-<CCCC>` key and an FNV-1a
check code, so keys produced by this panel **will not activate**. Mint production keys
with `/opt/swift/scripts/dpf-mint-license.py` (or `mint_key`) instead.

### Provider API keys — the complete surface

API keys are **not** entered in the Admin panel. The app exposes them in Settings, in two
places only:

- The in-tab **API Keys** group (`src/ui/main_content.rs`) — **OpenAI** and
  **Anthropic** only.
- The **Settings dialog** (`src/ui/settings_dialog.rs`) — **all five** providers:
  OpenAI, Anthropic, Google, DeepSeek, Moonshot.

That is the complete provider-key surface. The Admin panel does not edit API keys.

### Webhooks are not implemented

The `webhooks` feature is in the Team/Agency/Enterprise feature lists and the Webhooks
tab has **Start**/**Stop** buttons, but the module (`src/webhook.rs`) contains **no
listener** — no TCP socket is bound. The **Running/Stopped** indicator is driven by an
in-memory `AtomicBool` the UI sets itself, not by a live server. Treat the webhook
module as **not implemented**: do not document it as a working automation endpoint.

### Testing licence keys in the UI

1. Open Settings → License.
2. Enter a key minted with the Python tool.
3. Features unlock immediately; no restart is needed.

---

## Deployment Checklist

1. **Ship the four config files in the app's working directory:**
   - `feature_tiers.json` (first key must be `"tiers"`)
   - `pricing.json`
   - `platform_formats.json`
   - `revoked_keys.json` (`{"revoked_keys": []}`)
2. **Database:** SQLite file `dpf_data.db`, opened **relative to the working directory**
   (not `$HOME`).
3. **LLM provider keys (user-provided, in Settings):** OpenAI, Anthropic, Google,
   DeepSeek, Moonshot.
4. **Marketplace API keys:** Etsy API key, Gumroad access token.
5. **Font:** `/assets/Inter-Regular.ttf`
6. **Minting tool available at** `/opt/swift/scripts/dpf-mint-license.py` for
   issuing keys.

---

## Testing

### Run the regression suite

```bash
cargo test
```

The licence system is covered by **five unit tests** in `src/license_manager.rs`:

| Test | What it protects |
|------|------------------|
| `mint_matches_python` | The Rust and Python minting implementations pin the same keys (`DPF-TEAM-AB12CD34-P9SU`, `DPF-AGENCY-ZZ99YY88-R5QY`). If this fails, every new key is DOA. |
| `check_code_validates_its_own_keys` | Every tier's minted key round-trips through its own check code (4 segments, `DPF` prefix). |
| `tampered_key_is_rejected` | A single wrong final character fails the check code. |
| `tiers_are_supersets` | Each tier contains everything in the tier below it, or gating is incoherent. |
| `free_tier_cannot_reach_paid_modules` | **The business rule:** the free Personal tier must not include any paid module (analytics, publishing, bundles, scheduler, adverts, qc, assets, webhooks, whitelabel, client_management, compliance, api_access, admin_panel). If it fails, the product is giving away what it sells. |

### Manual gating test

1. Launch with no licence — confirm only Personal features are reachable and every paid
   tab is dimmed with a 🔒.
2. Activate a `TEAM` key — confirm Team tabs unlock.
3. Activate an `AGENCY` key — confirm whitelabel, client management, compliance and
   custom integrations unlock.
4. Activate an `ENTERPRISE` key — confirm API access and the Admin panel unlock.
5. Try a tampered key (change one character) — confirm rejection.
6. Add a key to `revoked_keys.json`, then try to activate it — confirm rejection.
7. Try a key from the Admin panel generator — confirm it is **rejected** (legacy format).

### Marketplace publishing test

1. Add an Etsy API key via the Publishing tab; confirm it shows as stored.
2. Remove it; confirm it shows as not stored. Repeat with Gumroad.

---

## Config File Locations

| File | Purpose | Location |
|------|---------|----------|
| `feature_tiers.json` | Tier → feature list (gating source of truth) | App working directory |
| `pricing.json` | Tier prices and billing periods | App working directory |
| `platform_formats.json` | Per-marketplace format/tag/length limits | App working directory |
| `revoked_keys.json` | Revoked licence keys (`{"revoked_keys": []}`) | App working directory |
| `dpf_data.db` | SQLite database (licences, products, campaigns, adverts, …) | App working directory |
| `sales_export.csv` | CSV export output | App working directory |

---

*Version 1.4.2 — Digital Product Factory Admin Guide*
