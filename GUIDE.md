# Digital Product Factory - User Guide

## Table of Contents
1. [Getting Started](#getting-started)
2. [Bring Your Own Key (BYOK)](#bring-your-own-key-byok)
3. [Industry Presets](#industry-presets)
4. [The Pipeline](#the-pipeline)
5. [Creating Products](#creating-products)
6. [Market Research](#market-research)
7. [Strategy (deep thinking)](#strategy-deep-thinking)
8. [Clients](#clients)
9. [Contract Generator](#contract-generator)
10. [Bundles](#bundles)
11. [Scheduler](#scheduler)
12. [Exporting](#exporting)
13. [Settings](#settings)
14. [Logo Generator](#logo-generator)
15. [Vector Generator](#vector-generator)
16. [Adverts & Campaign Suite](#adverts--campaign-suite)
17. [Licence & Activation](#licence--activation)
18. [Workflow Examples](#workflow-examples)

---

## Getting Started

### Installation

1. Download the latest release from GitHub
2. Extract the archive
3. Run the executable:
 - Windows: `dpf.exe`
 - macOS: `dpf`
 - Linux: `dpf`

There is **no installer** and nothing is added to your system. The app is a single program you can
move anywhere, including a USB stick.

### Where your work is stored

Your ideas, products, sales, clients, licence, and generated files are stored in your **own user
data folder** — not in the program folder:

| Platform | Location |
|----------|----------|
| Windows | `%APPDATA%\DigitalProductFactory\` |
| macOS | `~/Library/Application Support/DigitalProductFactory/` |
| Linux | `~/.local/share/DigitalProductFactory/` (`$XDG_DATA_HOME` if set) |

This matters in two practical ways:

- **Updating cannot touch your data.** Replacing the program file leaves your work alone.
- **Moving the program cannot lose your work**, and launching it from a different folder shows the
  same data. (Earlier versions stored the database next to the program, which meant both of those
  could lose work — that is fixed. If you have an old database in the program folder, the app
  imports it on first start and leaves the original in place.)

**Back up that folder** to keep your work safe — it is the whole of your data.

### Updating to a new version

There is **no automatic update**. To update:

1. Download the new release from the same address
2. Unzip it and replace `dpf.exe` with the new one

**Do not delete the folder to update** — there is no need, and your data is not in it. You do not
need to re-enter your licence key or your AI key. If a release ever needs your attention for a
change, it is called out in the release notes.

### First Launch

On first launch, you'll see:
- **Dashboard**: Overview of your products and pipeline
- **Sidebar**: Navigation between all features
- **Status Bar**: License status and system info

### Initial Setup

1. Click **⚙ Settings** — the button at the **bottom of the left sidebar** (there is no top-right settings icon).
2. Add your own API keys. The app ships with **no AI credits**; you bring your own keys (see [Bring Your Own Key (BYOK)](#bring-your-own-key-byok)):
 - OpenAI API key (for GPT-4o, GPT-3.5)
 - Anthropic API key (for Claude)
 - Google API key (for Gemini)
 - DeepSeek API key (for DeepSeek Chat)
 - Moonshot API key (for Moonshot v1)
 All five fields are **masked** on screen and stored locally in the app's own config on your machine.
3. Configure preferences (auto-save, dark mode)
4. Click **Save**

---

## Bring Your Own Key (BYOK)

Digital Product Factory does **not** include AI credits, and there is no shared key pool. You bring your own API keys — your generation runs on your own provider account, billed by that provider at their rates.

### Where to Enter Keys

All five providers are entered in **⚙ Settings** (the button at the bottom of the left sidebar). Settings is the **complete** surface for all five:

- **OpenAI**
- **Anthropic** (Claude)
- **Google** (Gemini)
- **DeepSeek**
- **Moonshot**

Every key field is **masked** on screen. Keys are stored locally on your own machine in the app's config.

### How It Works

- There are **no shared keys**, **no proxy**, and **no markup** on your usage.
- Nothing is sent anywhere except **directly to the provider you chose** for a given task.
- The app picks a sensible model per task from whichever providers you've configured (see [AI Models](#ai-models)).
- Check your provider's pricing before generating at volume — cost and rate limits are entirely between you and your provider.

### Minimum to Get Started

You need **at least one** API key configured to use Create / AI generation. Add keys in **⚙ Settings**, then go to the Create tab (see [Creating Products](#creating-products)).

---

## Industry Presets

Industry Presets are pre-configured workflows designed for specific business models. Each preset includes a complete pipeline with recommended actions, modules, and tips tailored to that industry.

### How Industries Benefit from Digital Product Factory

#### Affiliate Marketing
**Who:** Product reviewers, email marketers, comparison site owners, YouTube reviewers 
**Benefits:**
- **AI Content Generation**: Create review copy, email sequences, and social posts in minutes
- **Legal Compliance**: Auto-generate affiliate disclosures and FTC compliance text
- **Bundle Builder**: Stack bonus products to increase conversions 30-50%
- **Scheduler**: Time email sequences for optimal open rates
- **Contract Generator**: Protect yourself with proper terms

**How to Use:**
1. Load the Affiliate Marketing preset
2. Research products in the Market Research module
3. Use AI Generation to write authentic reviews
4. Bundle bonuses with the Bundle Builder
5. Schedule launch sequences
6. Export deliverables in multiple formats

---

#### Content Creator / Influencer
**Who:** YouTubers, TikTok creators, Instagram influencers, podcasters, newsletter writers 
**Benefits:**
- **Sponsor Deal Management**: Generate contracts, SOWs, and rate cards
- **Content Pipeline**: Track scripts → filming → editing → publishing
- **Media Kit Builder**: Create professional press kits and portfolios
- **Legal Protection**: Usage rights, revision limits, payment terms
- **Repurposing System**: Archive content for future remix and clip extraction

**How to Use:**
1. Load the Content Creator preset
2. Use AI Generation for content ideas and scripts
3. Track production in the Pipeline
4. Generate sponsor contracts
5. Build media kits with Bundle Builder
6. Schedule content calendar
7. Export campaign reports for brands

---

#### Creator + Affiliate Hybrid
**Who:** YouTubers with affiliate links, newsletter writers, review channel owners 
**Benefits:**
- **Dual Revenue Streams**: Manage brand deals AND affiliate revenue in one workflow
- **Authentic Integration**: Blend sponsored content with natural affiliate mentions
- **Legal Shield**: Combined sponsor terms + affiliate disclosure compliance
- **Value Stacking**: Build email list (freebie) → nurture → affiliate pitch
- **Performance Tracking**: Compare organic vs. affiliate content metrics

**How to Use:**
1. Load the Hybrid preset
2. Map organic content to affiliate opportunities
3. Create sponsor pitches AND apply to affiliate programs
4. Use AI to integrate recommendations naturally
5. Build lead magnets + bonus stacks
6. Schedule coordinated campaigns
7. Split test and optimize

---

#### E-Learning & Online Courses
**Who:** Course creators, coaches, mentors, training program builders, membership site owners 
**Benefits:**
- **Curriculum Design**: AI-assisted course structure and learning objectives
- **Content Creation**: Generate scripts, workbooks, and assessments
- **Legal Protection**: Terms of service, enrollment agreements, refund policies
- **Drip Scheduling**: Release content on schedule to reduce overwhelm
- **Student Materials**: Export workbooks, guides, and resources (open the HTML export and print to PDF for a PDF copy)

**How to Use:**
1. Load the E-Learning preset
2. Design curriculum with AI assistance
3. Create lesson content and workbooks
4. Generate legal documents
5. Build tiered course packages
6. Schedule drip releases
7. Export student materials

---

#### SaaS & Software Products
**Who:** SaaS founders, app developers, API providers, dev tool creators 
**Benefits:**
- **Product Roadmap**: Track features from idea to shipped
- **Legal Foundation**: Terms of Service, Privacy Policy, EULA generation
- **Beta Management**: Beta agreements and feedback collection
- **Documentation**: Auto-generate API docs and user guides
- **Launch Coordination**: Press kits, demo assets, launch sequences

**How to Use:**
1. Load the SaaS preset
2. Research competitor features
3. Track development in Pipeline
4. Generate legal documents
5. Manage beta program
6. Create launch assets
7. Export documentation

---

#### Digital Marketing Agency
**Who:** Marketing agencies, PPC managers, SEO consultants, social media managers 
**Benefits:**
- **Client Onboarding**: Service agreements, SOWs, KPI definitions
- **Campaign Management**: Full pipeline from pitch to performance report
- **AI Copywriting**: Generate ad copy, content calendars, campaign concepts
- **Automated Reporting**: Export performance reports and analytics summaries
- **Contract Protection**: 3-month minimum commitments, scope definitions

**How to Use:**
1. Load the Marketing Agency preset
2. Research prospects' markets
3. Generate proposals and contracts
4. Create campaign strategies with AI
5. Execute and track campaigns
6. Optimize based on data
7. Export client reports

---

#### Freelancer / Consultant
**Who:** Freelance developers, designers, writers, business consultants 
**Benefits:**
- **Lead Management**: Qualify prospects and track inquiries
- **Proposal Generation**: AI-written custom proposals with pricing
- **Contract Protection**: 50% upfront, revision limits, payment terms
- **Project Tracking**: Milestones, check-ins, progress documentation
- **Deliverable Packaging**: Professional export of final files

**How to Use:**
1. Load the Freelancer preset
2. Qualify leads and estimate scope
3. Generate proposals
4. Create service agreements
5. Track project milestones
6. Manage client feedback
7. Export final deliverables

---

#### Info Products & Templates
**Who:** Template creators, ebook authors, Notion template sellers, digital download shops 
**Benefits:**
- **Trend Research**: Identify hot topics and validate demand
- **Rapid Creation**: AI-generated content and designs
- **Multi-Format Export**: Markdown, HTML, JSON, ZIP (see [Exporting](#exporting) for exactly which formats are real today)
- **License Protection**: Terms of use and usage rights
- **Bundle Strategy**: Package related products for higher order value

**How to Use:**
1. Load the Info Products preset
2. Research trending topics
3. Outline and create products with AI
4. Design covers and layouts
5. Generate license terms
6. Create bonus bundles
7. Export in multiple formats
8. Launch and iterate

---

#### Business Setup (Complete Workflow)
**Who:** New Etsy sellers, POD entrepreneurs, digital product beginners, side hustlers 
**Benefits:**
- **Step-by-Step Guidance**: 12-stage workflow from zero to launch
- **Platform Setup**: Covers all major marketplaces (Etsy, eBay, Gumroad)
- **Brand Building**: Name generation, logo creation, storefront design
- **Automation**: Email systems, fulfillment, scheduling
- **Testing Framework**: User testing and feedback collection

**How to Use:**
1. Load the Business Setup preset
2. Complete each stage in order
3. Use Factory modules for each step
4. Set up accounts and storefronts
5. Create initial product line
6. Build marketing systems
7. Test and optimize
8. Launch and scale

---

### Industry-Specific Quick Reference

| Industry | Key Modules | Primary Output | Time to First Sale |
|----------|-------------|----------------|-------------------|
| Affiliate Marketing | AI Generation, Scheduler, Bundle Builder | Email sequences, bonus packs | 1-2 weeks |
| Content Creator | Pipeline, Contract Generator, Scheduler | Content calendar, media kit | 1 week |
| Creator Hybrid | All modules | Integrated campaigns | 2-3 weeks |
| E-Learning | AI Generation, Scheduler, Export | Course curriculum, workbooks | 1-2 months |
| SaaS | Pipeline, Contract Generator, Export | Product + documentation | 3-6 months |
| Marketing Agency | Market Research, AI Generation, Export | Client campaigns, reports | 1-2 weeks |
| Freelancer | Pipeline, Contract Generator, Export | Proposals, deliverables | 1 week |
| Info Products | AI Generation, Bundle Builder, Export | Templates, ebooks, bundles | 1-2 weeks |
| Business Setup | All modules | Complete business launch | 1-2 months |

---

### Choosing the Right Preset

**Just starting out?** → Business Setup 
**Have an audience?** → Content Creator or Creator Hybrid 
**Want passive income?** → Affiliate Marketing or Info Products 
**Have expertise to teach?** → E-Learning 
**Building software?** → SaaS 
**Serving clients?** → Freelancer or Marketing Agency

**Pro Tip:** You can load multiple presets and combine workflows. Many successful creators use a hybrid approach.

### Available Presets

| Preset | Best For | Stages |
|--------|----------|--------|
| **Affiliate Marketing** | Product reviewers, email marketers | Research → Ideation → Create → Legal → Bundle → Schedule → Publish → Analyze |
| **Content Creator** | YouTubers, influencers, podcasters | Ideation → Sponsor Deals → Production → Legal → Assets → Schedule → Export → Archive |
| **Creator + Affiliate Hybrid** | Multi-monetization creators | Combined workflow for brand deals + affiliate revenue |
| **E-Learning** | Course creators, coaches | Curriculum → Content → Legal → Bundle → Schedule → Export |
| **SaaS** | Software founders, app developers | Roadmap → Development → Legal → Beta → Launch → Export |
| **Marketing Agency** | Agencies, consultants | Prospect → Onboard → Strategy → Execute → Optimize → Report |
| **Freelancer** | Developers, designers, writers | Lead → Proposal → Contract → Work → Review → Deliver |
| **Info Products** | Template sellers, ebook authors | Research → Outline → Create → Design → Legal → Bundle → Export → Launch |
| **Business Setup** | New sellers, side hustlers | Complete 12-step workflow from research to launch |

### Using Presets

1. Go to **🎯 Presets** tab in the sidebar
2. Browse preset cards with descriptions
3. Click **View Details** to see full workflow
4. Click **Load Pipeline** to create sample ideas for each stage
5. Customize the generated ideas for your needs

### Preset Features

Each preset includes:
- **Stage breakdown**: What to do at each step
- **Recommended modules**: Which Factory features to use
- **Action checklists**: Specific tasks to complete
- **Output descriptions**: What you should have after each stage
- **Quick tips**: Industry-specific advice

---

## The Pipeline

The Pipeline is your command center for managing products from idea to sale. It's a kanban-style workflow with 7 stages.

### Pipeline Stages

| Stage | Icon | Purpose |
|-------|------|---------|
| **Idea** | 💡 | Capture product ideas |
| **Research** | 🔍 | Market research completed |
| **Creating** | 🔨 | Actively building the product |
| **Review** | 👀 | Quality check before publishing |
| **Listed** | 📋 | Published on platforms |
| **Selling** | 💰 | Active sales and revenue |
| **Archived** | 📦 | No longer active |

### View Modes

Switch between three views using the toolbar:

1. **Kanban** (default): Drag-and-drop cards between columns
2. **List**: Sortable table view with all details
3. **Calendar**: Timeline view for scheduled releases

### Adding Ideas

**Quick Add:**
1. Click **"➕ Quick Add Idea"** button
2. Fill in:
 - Title (required)
 - Description
 - Product type
 - Priority (Low/Medium/High/Urgent)
 - Estimated value
3. Click **Add Idea**

**From Dashboard:**
- Click any Quick Action card to add directly to that stage

### Managing the Pipeline

**Moving Cards:**
- **Drag and drop**: Click and drag the ⋮⋮ handle to move between stages
- **Keyboard**: Select card, use arrow keys + Enter

**Editing:**
- Click any card to open details
- Edit inline or open full editor
- Changes auto-save

**Filtering:**
- Use search box to filter by title, description, or tags
- Click stage headers to filter by stage
- Use priority filters for urgent items

### Pipeline Statistics

The Dashboard shows:
- **Total Ideas**: All items in pipeline
- **In Progress**: Items in Creating stage
- **Selling**: Active revenue-generating products
- **Potential Value**: Sum of all estimated values

---

## Creating Products

### Selecting a Template

1. Go to the **Create** tab.
2. Pick a category filter, or just scroll the template catalogue.
3. Click **Select** on the template you want.

### Built-in Templates

The catalogue ships **12 ready templates**. The template *categories* shown in the filter are broader than the number of populated templates, so some categories may be sparse or empty.

#### Core Templates

| Template | Best For | Output |
|----------|----------|--------|
| **Daily Planner** | Productivity enthusiasts | PDF |
| **Gratitude Journal** | Wellness market | PDF |
| **Budget Tracker** | Finance niche | XLSX |
| **Freelance Contract** | Service providers | DOCX |

#### Digital Product Templates

| Template | Category | Description |
|----------|----------|-------------|
| **Digital Stickers Pack** | Digital Stickers | Sticker packs for OneNote, GoodNotes, and note-taking apps |
| **Digital Art Printables** | Digital Art | AI-generated art for wall art and home decor |
| **Clip Art Bundle** | Clip Art | Pre-made graphics for presentations and documents |
| **Adult Coloring Pages** | Coloring Pages | Intricate mandalas, animals, landscapes for coloring books |
| **Logo Design Pack** | Logo Design | Professional logos for small businesses and startups |
| **Notion Template** | Notion Templates | Productivity templates for students and professionals |
| **Printables & Planners** | Printables | Printable planners, trackers, and organizers |
| **Print-on-Demand Designs** | POD Designs | AI designs for mugs, shirts, hoodies, and POD products |

> The **Output** column is the format a template's content is written for. See [Exporting](#exporting) for which formats the exporter can produce for real today.

### Template Categories

Templates are organized by category:
- **Planners**: Daily, weekly, monthly planners
- **Journals**: Gratitude, fitness, travel journals
- **Spreadsheets**: Budget trackers, calculators
- **Guides**: How-to guides, workbooks
- **Legal**: Contracts, agreements
- **Digital Stickers**: For note-taking apps
- **Digital Art**: Printable wall art
- **Clip Art**: Graphics for documents
- **Coloring Pages**: Adult coloring books
- **Logo Design**: Business branding
- **Notion Templates**: Productivity systems
- **Printables**: Planners and trackers
- **POD Designs**: Print-on-demand graphics

### The Templates Tab

The **Templates** tab in the sidebar is a real browsable catalogue. It lists every template with its name, description, category, tags, and trending score (🔥), sorted by trending. You can filter the list and choose an output format to export from there as well.

### Configuring Parameters

Each template has customizable parameters:

**Example: Daily Planner**
- Morning routine duration (15/30/60 min)
- Style (Minimal/Decorative/Professional)
- Color scheme

**Example: Freelance Contract**
- Client name
- Service description
- Payment amount
- Timeline
- Number of revisions

### Generating

Generation uses your own provider keys — you need **at least one API key** configured in **⚙ Settings** first (see [Bring Your Own Key (BYOK)](#bring-your-own-key-byok)).

1. Pick a category filter or scroll the catalogue, and click **Select** on a template.
2. Fill in the parameter form. Only the fields that template actually needs are shown.
3. Optionally click **Preview Prompt** to see the exact prompt that will be sent to the model.
4. Click **⚡ Generate Product**.
5. Wait for the result (typically 5-30 seconds).
6. The result shows the **model used** and the generated content. A status line reports any errors.
7. Export the result, or save it to your pipeline.

If a required parameter is blank, the app tells you before spending a request. If no API key is configured, the Create tab prompts you to add one in Settings.

### AI Models

The model list is whatever **your own provider account exposes** — the app doesn't ship or proxy models. For each task it picks a sensible model from the keys you've configured. A representative mapping:

| Task | Model | Provider |
|------|-------|----------|
| Creative writing | GPT-4o | OpenAI |
| Structured data | Claude 3.5 Sonnet | Anthropic |
| Technical content | Gemini 1.5 Pro | Google |
| Logic & reasoning | DeepSeek Chat | DeepSeek |
| Chinese content | Moonshot v1 | Moonshot |
| Quick tasks | GPT-3.5 | OpenAI |

**API keys required:** OpenAI, Anthropic, Google, DeepSeek, Moonshot — all five are configured in **⚙ Settings** (see [Bring Your Own Key (BYOK)](#bring-your-own-key-byok)). The exact model names available depend on your provider account.

### AI Prompt Templates

Each template includes optimized AI prompts with:
- **Aspect ratio presets**: `--ar 293:151` for mugs, `--ar 1:1` for stickers, etc.
- **Style modifiers**: Watercolor, minimalist, vintage, etc.
- **Output specifications**: Format, resolution, use case
- **Time estimates**: Most products take ~1 day/month to maintain

Click **Preview Prompt** in the Create tab to see the exact prompt a template will send, and copy it if you want to reuse it in another tool.

---

## Market Research

### Searching Platforms

1. Go to **Research** tab
2. Enter search query (e.g., "digital planner 2026")
3. Select platforms:
 - Etsy (handmade/creative)
 - Gumroad (digital products)
 - Amazon (broader market)
4. Click **🔍 Search**

### Understanding Results

For each product found:
- **Title**: Product name
- **Price**: Current selling price
- **Rating**: Customer rating (if available)
- **Reviews**: Number of reviews
- **Seller**: Shop/platform name

### Market Insights

After searching, the system analyzes:

**Price Analysis:**
- Average price
- Price range (min/max)
- Recommended pricing

**Competition Level:**
- **Low** (0-10 results): High opportunity
- **Medium** (11-50): Moderate competition
- **High** (51-200): Saturated market
- **Saturated** (200+): Very competitive

**Top Keywords:**
- Most common words in successful listings
- Use these for SEO and titles

### Trending Searches

Quick-access buttons for hot niches:
- planner 2026
- digital journal
- budget tracker
- social media templates
- notion template
- resume template

Click any trend to auto-fill search.

---

## Strategy (deep thinking)

**Market Research tells you what the market looks like. Strategy tells you what to build.**

This is the one part of the app that does not generate anything — it *decides*. It exists because
the most expensive mistake a creator makes is not bad copy, it is **building something nobody buys**.

### Using it

Find the **Strategy** panel on the **Research** tab, below the search results.

1. **Niche / market** — what you are thinking of selling into.
   *e.g. "ADHD planners for working parents"*
2. **You already sell** *(optional)* — your existing products. Filling this in sharpens the advice
   considerably, because it can suggest bundles rather than starting from nothing.
3. **Price positioning** *(optional)* — *e.g. "under $50 impulse buy"*.
4. Press **🧠 Build strategy brief**.

### What you get back

| Section | What it tells you |
|---|---|
| **The buyer** | Who they are and the moment they decide to spend |
| **Build these** | Three product ideas, each with why it should sell and who it competes with |
| **Pricing and bundling** | A concrete recommendation with reasoning |
| **DO NOT build this** | One tempting idea that would fail, and why |
| **Confidence** | Low/Medium/High, plus the one assumption that would change the answer |

**The "DO NOT build this" section is deliberate and is often the most valuable part.** Advice that
only ever says "yes, build it" is flattery, not help.

### Choosing the AI that runs it

Strategy uses **the AI key you already have** — the same key that runs the rest of the app. There
is no separate subscription and nothing extra to sign up for.

**Use:** leave on **Auto (best available)** and it picks the strongest key you hold
(Claude → DeepSeek → OpenAI → Google → Moonshot). Or pick a provider yourself.

**Model:** the dropdown lists suggested models for the chosen provider. **You can also type any
model id directly into the box** — so if a provider releases or retires a model, you are never
stuck waiting for an app update.

**If you choose a provider you have no key for, you get a clear error** rather than a silent switch
to a different provider. That is intentional: switching behind your back would spend a different
account's credit.

### Two things to know

- **It is ONE call, and it is not a chat.** Press the button, get the brief. There is no
  conversation box, deliberately — a strategy chat could run up your usage without you noticing.
- **The brief is judgement, not verified fact.** It reasons from what you tell it and is instructed
  not to invent statistics, but treat it as a well-argued second opinion, not a guarantee.

*Available on every tier — it uses your own key, so there is nothing for us to meter.*

---

## Clients

**For running the same workflow for several paying clients.**

Turn client management on and each piece of work can be tagged to a client, so you can see what you
are producing for whom instead of keeping it all in one pile.

Useful if you sell product-creation as a service rather than selling the products yourself.

*Agency tier and above.*

---

## Contract Generator

### Creating Legal Contracts

1. Go to **Contracts** section (in Create tab)
2. Select contract type:
 - Freelance Service Agreement
 - Mutual NDA
 - Rental/Lease Agreement
 - Employment Contract
 - Sales of Goods
 - Partnership Agreement
 - Consulting Agreement
 - Coaching Agreement

### Guided Prompts

The assistant will ask for:
- Party names (client, freelancer, etc.)
- Specific details (payment, timeline, scope)
- Jurisdiction (governing law)

**Example: Freelance Contract**
- Client name
- Your name/business
- Service description
- Payment amount
- Payment schedule
- Project timeline
- Revision count
- Jurisdiction

### Understanding the Output

**Full Contract:**
- Professional legal language
- All standard clauses
- Customized to your inputs
- Ready for signatures

**Plain English Summary:**
- Simple explanation of terms
- Who the parties are
- Key obligations
- Payment details

**Legal Disclaimer:**
Every contract includes:
```
IMPORTANT: This is a template only, not legal advice.
Always have a lawyer review before signing.
```

### When to Use vs. Hire Lawyer

| Use Generator | Hire Lawyer |
|---------------|-------------|
| Simple freelance work | High-value deals ($10K+) |
| Basic NDA | Complex partnerships |
| Standard rental | Employment with equity |
| Small sales | International contracts |
| Coaching services | Regulated industries |

---

## Bundles

### Auto-Bundle Strategies

The three auto-strategies below run for real, alongside the manual builder and bundle statistics. Bundles require a **Team** or higher licence.

**By Category:**
Groups products by type (all planners, all journals, etc.)
- Best for: Themed collections
- Discount: 20%

**By Value:**
Creates tiered bundles:
- **Premium Collection**: Top 5 products (25% off)
- **Starter Pack**: Entry-level selection (30% off)

**Seasonal:**
Creates time-themed bundles:
- Winter Collection (Jan-Mar)
- Spring Collection (Apr-Jun)
- Summer Collection (Jul-Sep)
- Fall Collection (Oct-Dec)

### Manual Bundle Builder

1. Go to **Bundles** tab
2. Select products from your pipeline
3. Set bundle name and description
4. Choose discount percentage
5. Review bundle statistics
6. Export as ZIP

### Bundle Statistics

For each bundle:
- **Product Count**: Number of items
- **Total Value**: Sum of individual prices
- **Customer Savings**: Dollar amount saved
- **Savings %**: Discount percentage
- **Est. Conversion**: Predicted sales rate

---

## Scheduler

### Schedule Types

**Once:**
- Single execution at specific date/time
- Use for: One-time product drops

**Daily:**
- Repeats every day at set time
- Use for: Daily social media posts

**Weekly:**
- Repeats on specific day of week
- Use for: Weekly product releases

**Interval:**
- Repeats every X minutes
- Use for: Frequent monitoring tasks

**Smart:**
- Automatically picks optimal time
- Business hours only
- Best for: Pinterest pins (8-11pm, 2-4pm optimal)

### Task Types

| Task | Description |
|------|-------------|
| **Generate Product** | Auto-create from template |
| **Publish Product** | Push to Etsy/Gumroad |
| **Research Market** | Run market analysis |
| **Create Bundle** | Auto-bundle products |
| **Pinterest Pin** | Schedule pin posts |
| **Backup Data** | Database backup |

### Managing Tasks

**Add Task:**
1. Click **➕ Add Task**
2. Select task type
3. Configure parameters
4. Set schedule
5. Enable/disable

**Monitor:**
- Green dot: Completed
- Yellow dot: Running
- Red dot: Failed
- Gray dot: Pending

**Actions:**
- ▶ Start scheduler
- ⏸ Stop scheduler
- Toggle individual tasks
- Delete old tasks

---

## Exporting

### Export Formats

The exporter offers **seven** formats and **all seven now produce genuine files**:

| Format | What you get |
|--------|--------------|
| **Markdown** | A `.md` file |
| **HTML** | A styled `.html` page |
| **PDF** | A real PDF document — opens in any reader, prints directly, paginates automatically |
| **DOCX** | A real Word document — headings, paragraphs and bullet lists preserved |
| **XLSX** | A real Excel workbook — markdown tables become spreadsheet rows and columns |
| **JSON** | A `.json` file with the content and metadata |
| **ZIP** | A `.zip` archive (multiple products) |

**Note on PDF:** it uses a standard built-in font (Helvetica), so accented characters outside that
character set are substituted. The layout is plain text — headings, paragraphs and lists — rather
than a designed brochure. For a fully typeset PDF, export to DOCX and print from Word.

### Exporting Single Product

1. Open product from pipeline
2. Click **Export** button
3. Select format
4. Choose output directory
5. Click **Save**

### Exporting Bundles

1. Go to **Bundles** tab
2. Select bundle
3. Click **Export Bundle**
4. Choose format (ZIP recommended)
5. All products exported together

### Batch Export

Select multiple products in pipeline:
1. Hold Ctrl/Cmd and click products
2. Right-click → **Export Selected**
3. Choose format
4. Products exported individually or as ZIP

---

## Settings

### API Configuration

**This is the complete place to enter all five provider keys.** Every field is **masked** on screen, and keys are stored locally in the app's own config on your machine — there are no shared keys and no proxy (see [Bring Your Own Key (BYOK)](#bring-your-own-key-byok)).

**OpenAI:**
- Get key: https://platform.openai.com/api-keys
- Models: GPT-4o, GPT-4, GPT-3.5

**Anthropic:**
- Get key: https://console.anthropic.com
- Models: Claude 3.5 Sonnet

**Google:**
- Get key: https://makersuite.google.com/app/apikey
- Models: Gemini 1.5 Pro

**DeepSeek:**
- Get key: https://platform.deepseek.com/api_keys
- Models: DeepSeek Chat (logic, reasoning, analysis)

**Moonshot:**
- Get key: https://platform.moonshot.cn/console/api-keys
- Models: Moonshot v1 (Chinese content, general)

### Safety Limits

Configure rate limiting:
- Max searches per hour: 20 (default)
- Max products per day: 10 (default)
- Max publishes per hour: 5 (default)

These prevent API overuse and platform bans.

### Performance

- **Auto-save**: Save work automatically
- **Dark mode**: Toggle light/dark theme
- **Max concurrent tasks**: Limit parallel operations
- **Cache size**: Database cache in MB


---

## Logo Generator

Create production-ready logos as SVG. Requires a **Team** or higher licence.

### What It Produces

For each logo it generates **3 SVGs** — an icon, a typography lockup, and a combined mark — across **7 styles**:

- Minimal
- Modern
- Vintage
- Playful
- Corporate
- Tech
- HandDrawn

### Generating a Logo

1. Go to the **🎨 Logo Generator** tab in the sidebar (under Library).
2. Enter the **brand name** and an optional **tagline**.
3. Set a **hex colour palette** and an **icon description**.
4. Preview updates live as you adjust the inputs.
5. Generate — the logo **saves to your library** and persists across restarts.

### Favicon Package

You can also produce a **Favicon Package** into a folder you choose. It contains:

- PNG icons at **16, 32, 48, 192, and 512** px
- `favicon.ico`
- `apple-touch-icon.png`
- `site.webmanifest`

### Saving a raw SVG

**Export SVG** saves the logo as a **scalable vector file** (`.svg`), rather than a picture of one.

Reach for this when you need the artwork itself rather than a fixed-size image:

- Opening or editing the logo in Illustrator, Figma, Inkscape or Affinity
- Placing it on a website or a print job at any size — vectors stay sharp at any resolution
- Handing the logo to a printer or a client who needs the source artwork

Favicons are raster images at set sizes; the raw SVG is the original vector. Choose per job, and
save both when you are unsure.

---

## Vector Generator

Generate reusable SVG vector assets. Requires a **Team** or higher licence.

### What It Produces

SVG assets across **7 categories**:

- Icon
- Illustration
- Badge
- Pattern
- Decorative
- Infographic
- UI Element

### Generating and Exporting

1. Go to the **📐 Vector Generator** tab in the sidebar (under Library).
2. Pick a category and describe the asset.
3. Generate — the asset **saves to your library** and persists across restarts.
4. Export as **SVG** or **PNG (512×512)**.

---

## Adverts & Campaign Suite

Create advertising creatives across multiple aspect ratios with AI-generated copy, background concepts, conversion scores, and per-ratio layout specs. Generate pro-quality ad concepts for social media, display networks, and print — all from within the app.

### Accessing the Feature

Click **📢 Adverts** in the sidebar under the Business section. Requires a Pipeline product to exist (ad concepts are generated from your existing products).

### Key Concepts

- **Campaign** — A grouping of adverts targeting a specific product and audience
- **Advert** — A single creative in a specific aspect ratio with AI copy and visual concept
- **Aspect Ratios** — 3 formats supported:
 - **Square (1:1)** — 1080×1080 px — Instagram feed, Facebook feed
 - **Story (9:16)** — 1080×1920 px — Instagram Stories, TikTok, Facebook Stories
 - **Landscape (16:9)** — 1200×628 px — Facebook link ads, Google Display, retargeting
- **Copy Frameworks** — 5 AI frameworks for ad copy:
 - **PAS** — Problem → Agitate → Solution
 - **AIDA** — Attention → Interest → Desire → Action
 - **BAB** — Before → After → Bridge
 - **Social Proof** — Testimonials, stats, trust signals
 - **Benefit-Driven** — Feature → Benefit → Outcome
- **Conversion Score** — AI-generated score (0-100) predicting creative effectiveness with reasoning
- **Brand Identity** — Extracted brand voice, recommended color palette, target platforms
- **Layout Spec** — Per-ratio positioning: headline position, subheadline position, CTA position, product scale percentage, product rotation

### Creating a Campaign

1. Open the **📢 Adverts** tab
2. Click **New Campaign**
3. Fill in:
 - **Campaign Name** — Internal label (e.g., "Q4 Product Launch")
 - **Description** — Campaign strategy notes
 - **Goal** — Campaign goal (Lead Generation, Brand Awareness, Sales Conversion, or Promo Sale)
 - **Product Name** — Select from existing Pipeline products
 - **Target Audience** — Describe who you're targeting (e.g., "Small business owners aged 25-45")
 - **Landing URL** — Optional click destination
 - **Platform** — Facebook, Instagram, Google, TikTok, LinkedIn, Print, or Other
4. Click **Save Campaign**
5. The campaign is stored in the local SQLite database

### Generating Ad Concepts (AI)

1. Select a campaign from the campaign list
2. Click **Generate Concepts**
3. Configure generation parameters:
 - **Target Audience** — Refine the audience description
 - **Brand Identity** — Set brand voice (e.g., "Professional yet friendly"), primary colors, industry
 - **Number of Concepts** — 1-5 concepts per format
4. AI generates for each aspect ratio:
 - **Copy Variations** — Headline, body text, CTA in the selected framework
 - **Visual Concepts** — 2-3 background concepts with color schemes and prompts
 - **Conversion Score** — 0-100 with reasoning for the score
 - **Brand Identity Extraction** — Tone analysis, palette recommendations
 - **Layout Specs** — Per-ratio positioning data:
 - Square (1:1): Headline at top-center, CTA at bottom-right, product at 60% scale
 - Story (9:16): Headline at top-third, CTA at bottom-center-swipe, product at 75% scale
 - Landscape (16:9): Headline at left-half, CTA at bottom-right, product at 50% scale

### Editing Adverts (Composer View)

In the **Composer** view, every aspect is editable after AI generation:

- **Copy**: Edit headline, subheadline, body text, CTA button text, and tagline individually
- **Copy Framework**: Switch between PAS, AIDA, BAB, Social Proof, or Benefit-Driven
- **Brand Identity**: Adjust brand voice, primary colors (hex codes), secondary palette, industry
- **Product Placement**: 
 - Scale: 30-100% slider
 - Position: Center, Left, Right
 - Rotation: 0-360 degrees
- **Layout Toggles**: Overlay on/off, shadow on/off, text position adjustments
- **Background**: Swap between generated concepts or enter a custom background prompt

Changes are reflected live in the Preview pane.

### Previewing Adverts

The **Preview** view renders a canvas representation of each ad:

- **Aspect Ratio Canvas** — Drawn with guide lines showing safe zone, margins, center crosshairs
- **Product Mockup** — Product image rendered at configured scale with rotation
- **Brand Colors** — Background color applied from brand identity palette
- **Copy Overlay** — Headline and body text displayed at correct positions per LayoutSpec
- **CTA Button** — Rendered as a badge at the CTA position with brand colors
- **Conversion Score Badge** — Displayed in a corner with color coding (green 80+, yellow 50-79, red below 50)
- **Multi-Ratio Tabs** — Switch between Square, Story, and Landscape previews for the same advert concept

### Exporting

- **JSON Export** — Export advert concepts as structured JSON
- Export options:
 - **Single Advert** — Export just one concept
 - **Campaign Bundle** — Export all adverts in a campaign
- Exported data includes: copy text, layout specs per ratio, brand identity, product placement params, conversion score, generation config
- Files are saved to `{app_data_dir}/assets/exports/`
- File naming: `{campaign_name}_{product_name}_{ratio}_{timestamp}.json`

**Export Summary (Markdown)**

**Export Summary** writes a readable campaign overview as a **Markdown** file — the version to read
yourself or send to a client, rather than feed to another program.

Use it when you want to review or share a campaign:

- A human-readable summary of every advert in the campaign, with its copy and conversion score
- Paste into Notion, Obsidian, GitHub, or any Markdown editor
- Send to a client for sign-off without them needing the app

**Which export to use:** JSON is for machines (feeding Canva, Figma, Photoshop); Markdown is for
people. The names in the copy may be added as social usernames or hashtags where relevant.

### Copy Frameworks Explained

| Framework | Structure | Best For |
|-----------|-----------|----------|
| **PAS** | Problem → Agitate → Solution | Pain-point marketing, problem solvers |
| **AIDA** | Attention → Interest → Desire → Action | Launch campaigns, new products |
| **BAB** | Before → After → Bridge | Transformation offers, coaching |
| **Social Proof** | Proof → Pain → Solution → Trust | Established products, testimonial-rich |
| **Benefit-Driven** | Feature → Benefit → Outcome | Feature-rich products, B2B |

### Aspect Ratio Quick Reference

| Format | Dimensions | Best For | Key Placement |
|--------|-----------|----------|--------------|
| Square (1:1) | 1080×1080 | Instagram feed, Facebook feed | Headline top-center, CTA bottom-right |
| Story (9:16) | 1080×1920 | Instagram Stories, TikTok | Headline top-third, CTA bottom-center |
| Landscape (16:9) | 1200×628 | Facebook link ads, Google Display | Headline left-half, CTA bottom-right |

### Tips

- **Target Audience is everything** — Be specific. "Small business owners 25-45" beats "everyone"
- **Generate 3-5 concepts** — Then pick the best, don't use the first result
- **Conversion Score 80+** — Strong creative alignment. 60-79 needs tweaking. Below 60 — regenerate
- **Story format converts best** — Mobile-first campaigns consistently outperform
- **Landscape for retargeting** — 16:9 is ideal for display networks and Google Ads
- **Always edit AI copy** — AI generates the raw material, you add the polish
- **Brand Identity consistency** — Use the same brand colors/voice across all adverts in a campaign
- **Export first, then use external tools** — JSON export feeds into Canva, Figma, or Photoshop for final pixel-perfect rendering

## Licence & Activation

The app is free to use on the **Personal** tier. Paid tiers unlock more seats and modules.

### Tiers

| Tier | Seats | Unlocks |
|------|-------|---------|
| **Personal (Free)** | 1 | Pipeline, Create / AI generation, Templates, Market Research, Contract Generator, Export, Presets, Variants |
| **Team** | 5 | Everything in Personal, plus Analytics, Publishing, Bundles, Scheduler, Adverts, QC Checklist, Asset Library, Webhooks, Mockup compositor, Logo Generator, Vector Generator |
| **Agency** | 20 | Everything in Team, plus Compliance Scanner, Client Management |
| **Enterprise** | Unlimited | Everything in Agency, plus Admin Panel |

Prices come from the sales page, not from the software: a licence may be a one-time payment or a
subscription (beta testers get a one-off), so the app tells you which plan you hold and never
what you paid for it.

Your current tier is shown in the sidebar underneath the app name.

### Upgrading

Anything your plan does not include shows a **🔒** in the sidebar and cannot be opened. Click it and
the licence dialog opens, telling you what is locked and offering an **Upgrade** button.

- The button takes you to the sales page in your browser, where you can buy.
- **It offers the next step up, not the far end of the ladder** — on the free plan you are offered
  Team, not Enterprise, because that is the smaller jump.
- **If you already hold everything, no button appears at all.** You will never be advertised to once
  you have bought the top plan.
- After buying, you get a licence key: paste it into the same dialog and press **Activate**.

*You are never shown a price inside the app — not in the dialog, not in the upgrade offer. What each
plan costs is on the sales page you are sent to.*

### Activating a Licence

1. Click **🔑 Licence** — the button at the **bottom of the left sidebar**. This opens the Licence dialog.
2. Paste your key and press **Activate**.
3. The key is validated **offline** (this product has no licence server) and saved, so activation survives restarting the app.

The key format is `DPF-TIER-XXXXXXXX-CCCC` — for example `DPF-TEAM-DEMO1234-2P7A`.

### Locked Modules

Modules your tier doesn't include appear in the sidebar with a **🔒** and cannot be opened. Clicking a locked module tells you which plan unlocks it and opens the Licence dialog. You can also **deactivate** a licence from the Licence dialog to drop back to the free Personal tier.

<!-- TODO (owner): add the real purchase / store URL here before publishing. The guide does not currently contain a live store link. -->

### Device Seats

Each paid tier includes a set number of seats (Personal 1, Team 5, Agency 20, Enterprise unlimited). Deactivate a licence to release its seat before activating it elsewhere.

---

## Workflow Examples

### Quick Product Launch (1 hour)

1. **Research** (10 min):
 - Search Etsy for "digital planner"
 - Note top keywords and prices
 - Identify gap in market

2. **Create** (30 min):
 - Select "Daily Planner" template
 - Configure: 30-min morning routine, Minimal style
 - Generate with GPT-4o
 - Review and refine

3. **Export** (5 min):
 - Export as HTML, then print to PDF if you need a PDF (see [Exporting](#exporting))
 - Create listing images (Canva/Figma)

4. **Publish** (15 min):
 - Upload to Etsy
 - Use researched keywords in title
 - Price at market average

5. **Pipeline**:
 - Add to "Listed" stage
 - Set reminder to check sales in 1 week

### Weekly Batch Production (4 hours)

1. **Monday - Research** (1 hour):
 - Identify 5 trending niches
 - Analyze top 10 products in each
 - Document keywords and pricing

2. **Tuesday-Wednesday - Create** (2 hours):
 - Generate 10 products
 - Use different templates
 - Vary styles and parameters

3. **Thursday - Bundle** (30 min):
 - Auto-bundle by category
 - Create 3 themed bundles
 - Set 20-30% discounts

4. **Friday - Schedule** (30 min):
 - Schedule 2 products/day for next week
 - Set Pinterest pins for optimal times
 - Enable auto-backup

### VA Team Operation

1. **Setup**:
 - Install on Team license (5 devices)
 - Create shared API key pool
 - Document brand guidelines

2. **Workflow**:
 - VA generates products from approved templates
 - Manager reviews in "Review" stage
 - Approved products auto-scheduled
 - Sales tracked in "Selling" stage

3. **Quality Control**:
 - Random sample review (10%)
 - Consistency checks
 - Monthly performance reports

---

## Troubleshooting

### Common Issues

**"API Key Invalid"**
- Check key is copied correctly
- Verify key has credits/balance
- Try regenerating key

**"Generation Failed"**
- Check internet connection
- Verify API service status
- Try different AI model

**"Export Failed"**
- Check disk space
- Verify write permissions
- Try different output directory

**"Slow Performance"**
- Close other applications
- Reduce concurrent tasks
- Check database size (auto-vacuum runs monthly)

**"The window is black / nothing appears"**

This is a graphics back end that cannot present on your machine, and the app now recovers from it
by itself. What to know:

- On start-up the app picks a back end automatically. If the first one fails to initialise, it
  relaunches itself once using the other one. **You do not need to do anything.**
- If the window stays black for about 20 seconds, the app relaunches itself on the other back end.
  A black window that resolves by itself after a few seconds is this watchdog working — not a fault.
- To choose a back end yourself, use the launcher next to the program:
  - **`dpf-glow.bat`** — OpenGL. Try this first on Windows Server, a VPS, or any machine with no
    dedicated graphics card.
  - **`dpf-wgpu.bat`** — DirectX 12. The default; best on a normal desktop or laptop with a GPU.
- You can also pass it on the command line: `dpf.exe --renderer glow` (or `--renderer wgpu`).
  Set `DPF_RENDERER` to the same value to make it stick.
- **If both back ends leave a black window**, the fault is earlier than graphics. The app writes
  **`dpf-startup.log`** next to itself, and it names the exact start-up stage it reached. Send that
  file to support — it turns "it is black" into a specific answer.
- The log also prints **`frame 1 rendered`** as soon as the app actually paints. If that line is
  absent, the app never reached its first frame and no graphics setting will help.

⚠️ Only one copy of the app should hold the data file at a time. Opening a second copy while the
first is running is supported, but if the app ever appears to hang on start-up, close every copy
and try once more.

### Getting Help

1. Check this guide first — and the **?** button on any screen for help on that screen
2. Review error messages carefully
3. Open an issue: https://github.com/Swiftsoftware143/digital-product-factory/issues
4. Contact support with:
 - Error message
 - Steps to reproduce
 - System info (OS, RAM)

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+N | New idea |
| Ctrl+S | Save |
| Ctrl+G | Generate product |
| Ctrl+E | Export |
| Ctrl+1 | Dashboard |
| Ctrl+2 | Pipeline |
| Ctrl+3 | Create |
| Ctrl+4 | Research |
| Ctrl+, | Settings |
| Esc | Close dialog |

---

## Tips for Success

1. **Start with research**: Never create without checking market
2. **Use the pipeline**: Track every idea from start to finish
3. **Bundle strategically**: 3+ products bundled sell 40% better
4. **Schedule consistently**: Regular releases build audience
5. **Track everything**: Use actual_value field for real sales data
6. **Iterate**: Move products back to "Creating" for improvements
7. **Stay legal**: Always use contract generator for client work
8. **Backup regularly**: Enable scheduled backups
9. **Use Industry Presets**: Start with a preset that matches your business model
10. **Focus first**: Perfect one product before expanding (per Business Setup preset)
11. **Create handcrafted-looking images**: Good shops have authentic, non-stock visuals
12. **Test multiple platforms**: Different platforms attract different buyer types


## Pre-Publish QC Checklist

Run products through a quality checklist before publishing to ensure consistency and catch issues early.

### QC Checks

| Check | What It Does |
|-------|-------------|
| **Duplicate Detection** | Scans product files for duplicates by name + size heuristic — no duplicates slip through |
| **Format Validation** | Verifies file format matches the product type (PDF, DOCX, XLSX, etc.) |
| **Title Check** | Ensures product title is set and reasonable length |
| **Description Check** | Flags missing or too-short descriptions |
| **Price Validation** | Warns if price is zero or unreasonably low |
| **Template Compliance** | Checks that the product matches its template requirements |

### Running a QC Check

1. Go to **QC Checklist** tab in the sidebar
2. Select a product from the dropdown
3. Click **Run QC** to execute all checks
4. Results show: ✅ Pass, ❌ Fail, ⚠️ Warning, or ⏭️ Skipped
5. Address flagged issues, then re-run

### Duplicate Registry

The app maintains a **file fingerprint registry** so you never upload or publish duplicate products. The registry auto-detects when you add a file that matches a previously-registered product.

### Team+ Feature

QC Checklist requires a **Team** or higher license.

---

## Compliance Scanner

Ensure your AI-generated content complies with platform policies and disclosure requirements.

### AI Disclosure Rules

Configurable rules for AI-generated content disclosure:
- **FTC Guidelines** — Auto-generated disclosure text for affiliate content
- **Platform Rules** — Etsy, Gumroad, Amazon disclosure requirements
- **Custom Rules** — Add your own disclosure requirements

### Denylist Scanner

Prevents your generation prompts from including restricted terms:
- **Trademark violations** — Catch brand names that shouldn't appear
- **Restricted categories** — Health claims, financial advice, etc.
- **Platform banned terms** — Terms that get products removed from marketplaces

### Tool License Checks

When generating with AI, the scanner checks that the output tool licenses are compatible with your intended use (commercial, resale, personal).

### Using Compliance

1. Go to **Compliance** tab in the sidebar
2. Configure disclosure rules (or use defaults)
3. Add denylist terms as needed
4. Run a scan before generating or publishing

### Team+ Feature

Compliance Scanner requires a **Team** or higher license.

---

## Asset Library

Manage your media assets — images, logos, icons, and fonts — in one central location.

### Features

- **Asset browser** — Browse all uploaded assets with thumbnails and tags
- **Tag system** — Tag assets by product, project, or category for easy filtering
- **Version tracking** — Each asset keeps version history (overwrites become new versions)
- **Rollback** — Restore any previous version of an asset
- **Cloud backup config** — Configure backup destinations (configurable JSON, no hardcoded settings)

### Adding Assets

1. Go to **Asset Library** tab in the sidebar
2. Click **Add Asset** and select a file (PNG, JPG, TTF, OTF, SVG)
3. Add tags and notes for organization
4. The asset is stored locally in the app's data directory

### Versioning

Every time you register a new version of an existing asset, the old version is preserved. You can:
- Click **Versions** on any asset to see its history
- Click **Rollback** to restore a previous version
- See timestamps and notes for each version

### Cloud Backup Configuration

Edit `backup_config.json` to set cloud backup destinations. The config file is editable without rebuilding the app:

```json
{
 "enabled": false,
 "scheduled_backup": false,
 "backup_interval_hours": 24,
 "retention_days": 30,
 "destinations": []
}
```

### Team+ Feature

Asset Library requires a **Team** or higher license.

---

## Webhooks — not yet available

> **This feature is not built yet.** The Webhooks tab exists and, when you press its start button,
> the interface reports a running server on a port — but **no web server is actually started** in
> this build. Nothing can connect to it. Do not build any automation against it, and do not treat
> the "running" indicator as real. This is a known gap in the app, not a setting you have got wrong.

### What is planned

When it is implemented, the intent is a small local HTTP endpoint so external tools can trigger a
generation run without opening the app, roughly:

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/generate` | POST | Trigger product generation with a JSON payload |
| `/status` | GET | Report whether the server is running |
| `/schema` | GET | Return the JSON schema for the generate endpoint |

The server would bind to **localhost only**, so it is never exposed to your network.

### What to do instead, today

Use the app's own automation that *is* implemented: the **Scheduler** tab can run generation,
publishing and research on a recurring basis, and **Publishing** pushes to your connected
marketplaces directly. If you need to drive the app from outside, do it through the user interface
until this section is replaced with real instructions.

Webhooks are intended to be a **Team** or higher feature. Until they ship, nothing is charged for
them.

---

## Product Variants

Create multiple variants of a product for different formats, prices, or configurations — with full version history.

### What Are Variants?

A product in your pipeline can have multiple variants. For example:
- **"Daily Planner"** → Variant: PDF ($9.99), Variant: DOCX ($12.99), Variant: Notion Template ($14.99)
- **"Logo Pack"** → Variant: Full Color ($29), Variant: B&W ($19), Variant: SVG Source ($49)

### Versioning

Each variant tracks its own version history:
- Every time you update a variant's content, a new version snapshot is saved
- Old versions are never deleted — you can view or restore them
- Active version is shown with a green ▶ indicator

### Using Variants

1. Go to **Variants** tab in the sidebar
2. Select a product from the dropdown
3. Click **Add Variant** to create a new variant
4. Set: name, format (PDF, DOCX, XLSX, ZIP, PNG, JPG, HTML, Markdown, JSON, TXT), and price
5. The variant is created as Draft — add content and set to Active
6. Click the clipboard icon on any variant to view version history
7. Click **Restore** to rollback to a previous version

### All Tiers

Product Variants are available on all license tiers — Personal, Team, Agency, and Enterprise.

---

---

*Version 1.4.2 - Native Rust Edition*
*Last Updated: July 2026*

## Changelog

### v1.4.2
- Added **Adverts & Campaign Suite** — Multi-format ad generation with AI copy frameworks (PAS, AIDA, BAB, Social Proof, Benefit-Driven), 3 aspect ratios (Square 1080×1080, Story 1080×1920, Landscape 1200×628), per-ratio layout specs, conversion scoring with reasoning, brand identity extraction, visual concept generation, campaign management, JSON export
- Added **DeepSeek** provider — Logic and Reasoning profiles using deepseek-chat
- Added **Moonshot** provider — Chinese language profile using moonshot-v1-8k
- Added per-ratio LayoutSpec struct — headline/subheadline/CTA positioning, product scale/rotation per aspect ratio
- Expanded API settings with DeepSeek and Moonshot key fields
- All 5 providers selectable per task via auto-select or manual profile

### v1.3.0
- Added **Product Variants & Versioning** — multiple variants per product with full version history and rollback
- Added **Pre-Publish QC Checklist** — duplicate detection, format validation, title/description/price checks
- Added **Compliance Scanner** — AI disclosure rules, denylist scanner, tool license checks
- Added **Asset Library** — local media manager with tag system, version tracking, and cloud backup config
- Added **Webhooks** — HTTP webhook server for external tool integration
- Updated feature_tiers.json with new module tier gating
- Updated ADMIN_GUIDE.md with new feature listings

### v1.2.0
- Added QC, Compliance, Asset Library, and Webhooks modules
- Updated pricing and feature tier configs

### v1.1.0
- Added **Industry Presets** with 9 pre-configured workflows
- Added **8 new digital product templates** (stickers, art, clip art, coloring pages, logos, Notion templates, printables, POD designs)
- Removed external tool references - Digital Product Factory is now the complete solution
- Updated Business Setup preset with 12-step workflow
- Removed hardcoded pricing from templates (users set their own prices)

### v1.0.0
- Initial release
- Pipeline kanban workflow
- AI product generation (OpenAI, Anthropic, Google)
- Market research
- Contract generator
- Scheduler
- Bundle builder
- Export (PDF, DOCX, XLSX, ZIP)

## Analytics & Sales Tracking

Track revenue, fees, and performance across all your products and marketplaces.

### Dashboard Summary Cards

The Analytics tab shows summary cards at the top:
- **Total Revenue** — Gross revenue from all recorded sales
- **Total Fees** — Sum of all platform fees
- **Net Revenue** — Revenue minus fees (your actual earnings)
- **Total Sales** — Total units sold across all products
- **Products** — Number of unique products with sales

### Revenue by Product

A scrollable list shows each product with total revenue and units sold. This lets you quickly see which products are performing best.

### Adding Sales Records

Click **Add Sale** to log a new sale. Enter:
- **Product name** — Free text (matches up with pipeline products)
- **Platform** — Etsy, Gumroad, Shopify, PayPal, etc.
- **Units sold** — Quantity sold
- **Revenue ($)** — Gross revenue before fees
- **Fee ($)** — Platform/processing fees

Net revenue is calculated automatically. Records are saved to the local SQLite database and persist between sessions.

### Sales Records List

All recorded sales are shown in a scrollable list with date, product name, units, and net revenue. Green highlighting makes profitable sales stand out.

### CSV Export

Click **Export CSV** to export the full sales ledger as a CSV file. Opens in Excel, Google Sheets, or any spreadsheet app. The file is saved as \sales_export.csv\ in the application directory.

### Team+ Feature

Analytics requires a **Team** or higher license. See [Licence & Activation](#licence--activation) for upgrade options.

---

## Marketplace Publishing

Publish digital products directly to marketplaces from within the app.

### Connected Platforms

| Platform | Status | Requirements |
|----------|--------|-------------|
| **Etsy** | ✅ Supported | API key, 3000x3000 thumbnails, 20MB max |
| **Gumroad** | ✅ Supported | Access token, 1280x720 thumbnails, 50MB max |
| **Shopify** | 🔧 Coming Soon | — |
| **Payhip** | 🔧 Coming Soon | — |

### Credential Management

API keys are stored in your **OS keychain** (not in plaintext files). Each platform button shows:
- ✅ **Green check** — Credentials stored and ready
- ❌ **Red X** — No credentials configured

To add credentials:
1. Select a platform from the left panel
2. Enter your API key or access token
3. Click **Save Key**
4. The key is encrypted via your OS keychain

To remove credentials, click **Remove** — it deletes the key from the keychain.

### Publish a Product

1. Move a product to the **Listed** or **Review** stage in the Pipeline
2. Go to the Publishing tab
3. Select a platform (must have credentials saved)
4. Choose the product from the dropdown
5. Set a price
6. Click **Publish**

The publish is queued and a log entry is created. Actual publishing to marketplace APIs requires marketplace-specific formatting handled by the platform format rules.

### Platform Format Configuration

Platform-specific requirements are stored in \platform_formats.json\:

- Thumbnail dimensions (width x height)
- Max file size in MB
- Max title length (characters)
- Max description length (characters)
- Max tags count

You can edit this JSON file directly to update rules without rebuilding the app.

### Publish Log

Every publish attempt is logged. Each entry shows:
- Date and time
- Product name
- Target platform
- Status: ✅ Published / ⏳ Pending / ❌ Failed / Removed
- Link to live listing (if available)

### Team+ Feature

Marketplace Publishing requires a **Team** or higher license.

---

## Inline Help System

Throughout the app, look for **❓ Help** in the status bar and **?** buttons next to feature headers.

- **Status bar help button** — Opens the full Help Index with all topics
- **Per-feature help buttons** — Click **?** next to any section header for contextual help
- **Help Index** — Browseable list of all help topics, grouped by feature area
- **Tier badges** — Help topics show which license tier a feature requires

The inline help system is available on all license tiers — no license required to access help content.

---
