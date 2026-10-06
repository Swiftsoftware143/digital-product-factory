# DPF — Module Test Sheet

For walking every module in one sitting. Each row: **what to do** → **what you should see**.

**Before you start**
- Paste your Enterprise key (Dashboard → *Enter licence key*, or the sidebar **Licence** button).
  Enterprise unlocks all 23 modules, so nothing should be locked.
- Add ONE AI key (Settings) if you want to test the AI modules. Without a key those modules should
  tell you so plainly rather than fail silently — that itself is worth checking.
- If the window is ever black: `dpf-startup.log` sits beside `dpf.exe`. It names the stage.

Legend: **🔑 = needs an AI key** · **— = works offline**

---

## Main

| # | Module | Do this | Should see |
|---|---|---|---|
| 1 | **Dashboard** | Open it | Idea count · "No AI provider key yet" banner until you add one · Enter-licence prompt if unlicensed · Recent Sales |
| 2 | **Pipeline** | Create an idea, move it | Idea appears as a card; can move between stages; **delete** removes it |
| 3 | **Create** 🔑 | Generate a product | A generated product; with no key, a clear message (not a hang) |
| 4 | **Research** 🔑 | Enter a market, run analysis | Avg price · competition level · **opportunity score** · the **Market Insight** panel now renders |
| 5 | **Templates** | Search / filter | Results narrow; searching a **category** name now matches |
| 6 | **Mockup** | Open a mockup | Preview renders |

## Tools

| # | Module | Do this | Should see |
|---|---|---|---|
| 7 | **Bundles** | Create a bundle | Bundle saved and listed |
| 8 | **Scheduler** | **➕ Add Task** → fill → save | New task appears; **🗑 delete** and **⏸ toggle** actually change state (these were dead buttons) |
| 9 | **Presets** | Browse presets | Preset list loads |

## Content

| # | Module | Do this | Should see |
|---|---|---|---|
| 10 | **Contracts** | Pick a **Category** → **Select** a template → fill the form → **Generate contract** | Real generated contract + **Copy**. (Filter and Select were dead; **Custom Agreement** is now reachable) |
| 11 | **Adverts** | **Edit (Composer)** → write → **Preview** → **Export Summary** | Composer + preview open with **Back to Suite**; **Export Summary** writes a Markdown file |

## Business

| # | Module | Do this | Should see |
|---|---|---|---|
| 12 | **Analytics** | Open it | Numbers/charts from your real data |
| 13 | **Publish** | Attempt a publish | Platform list; publishing is off until configured — should SAY so |

## Quality

| # | Module | Do this | Should see |
|---|---|---|---|
| 14 | **QC Checklist** | Run a check | Pass/fail list |
| 15 | **Compliance** | Open it | Disclosure rules load |
| 16 | **Clients** | Add a client | Client saved (Agency+ feature; Enterprise includes it) |

## Product Data

| # | Module | Do this | Should see |
|---|---|---|---|
| 17 | **Variants** | Create a variant | Variant stored and listed |

## Library

| # | Module | Do this | Should see |
|---|---|---|---|
| 18 | **Logo Generator** | Generate → **Export** | PNG/favicon pack **and now raw SVG** (new) |
| 19 | **Vector Generator** | Generate → export | SVG + PNG downloads |
| 20 | **Asset Library** | Open after generating assets | Your generated files listed with sizes |

## System

| # | Module | Do this | Should see |
|---|---|---|---|
| 21 | **Webhooks** | Open it | ⚠ Says **"Not implemented in this build"** — deliberate and honest. Reports nothing false |
| 22 | **Admin** | Open it | Admin view loads |
| 23 | **Settings** | Enter your AI key(s) | Key saved; Dashboard banner disappears. Provider + model selectable, plus a **free-text model id** |

---

## What to report back

For anything wrong, these three make it a 5-minute fix instead of an investigation:
1. **Module name**
2. **What you clicked, what you expected, what happened**
3. **`dpf-startup.log`** if it froze or closed

**Known and deliberate — not bugs:**
- **Webhooks** is labelled not-implemented (a previous version falsely claimed it was listening).
- **Publish** needs platform configuration.
- **No prices appear anywhere** in the app — by design.

**Totals: 23 modules · 22 unlocked on Enterprise · 1 honestly marked not-implemented.**
