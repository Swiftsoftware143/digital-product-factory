//! Template system for digital products

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub tags: Vec<String>,
    pub trending_score: u32,
    pub seasonal_peak: Option<String>,
    pub prompt_template: String,
    pub output_format: OutputFormat,
    pub parameters: Vec<TemplateParameter>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TemplateCategory {
    Planner,
    Journal,
    Spreadsheet,
    Guide,
    Resume,
    Cookbook,
    Business,
    Legal,
    DigitalStickers,
    DigitalArt,
    ClipArt,
    ColoringPages,
    LogoDesign,
    NotionTemplates,
    Printables,
    PodDesigns,
    Other,
}

impl TemplateCategory {
    pub fn name(&self) -> &'static str {
        match self {
            TemplateCategory::Planner => "Planner",
            TemplateCategory::Journal => "Journal",
            TemplateCategory::Spreadsheet => "Spreadsheet",
            TemplateCategory::Guide => "Guide",
            TemplateCategory::Resume => "Resume",
            TemplateCategory::Cookbook => "Cookbook",
            TemplateCategory::Business => "Business",
            TemplateCategory::Legal => "Legal",
            TemplateCategory::DigitalStickers => "Digital Stickers",
            TemplateCategory::DigitalArt => "Digital Art",
            TemplateCategory::ClipArt => "Clip Art",
            TemplateCategory::ColoringPages => "Coloring Pages",
            TemplateCategory::LogoDesign => "Logo Design",
            TemplateCategory::NotionTemplates => "Notion Templates",
            TemplateCategory::Printables => "Printables",
            TemplateCategory::PodDesigns => "POD Designs",
            TemplateCategory::Other => "Other",
        }
    }
    
    pub fn all() -> Vec<TemplateCategory> {
        vec![
            TemplateCategory::Planner,
            TemplateCategory::Journal,
            TemplateCategory::Spreadsheet,
            TemplateCategory::Guide,
            TemplateCategory::Resume,
            TemplateCategory::Cookbook,
            TemplateCategory::Business,
            TemplateCategory::Legal,
            TemplateCategory::DigitalStickers,
            TemplateCategory::DigitalArt,
            TemplateCategory::ClipArt,
            TemplateCategory::ColoringPages,
            TemplateCategory::LogoDesign,
            TemplateCategory::NotionTemplates,
            TemplateCategory::Printables,
            TemplateCategory::PodDesigns,
        ]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutputFormat {
    Markdown,
    Html,
    Pdf,
    Docx,
    Xlsx,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    Text,
    Number,
    Select(Vec<String>),
    Boolean,
    Color,
    Date,
}

pub struct TemplateRegistry {
    templates: HashMap<String, Template>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            templates: HashMap::new(),
        };
        registry.load_builtin_templates();
        registry
    }
    
    fn load_builtin_templates(&mut self) {
        let mut templates = vec![
            Template {
                id: "planner_daily".to_string(),
                name: "Daily Planner".to_string(),
                description: "A comprehensive daily planner with schedule, tasks, and notes".to_string(),
                category: TemplateCategory::Planner,
                tags: vec!["planner".to_string(), "daily".to_string(), "productivity".to_string()],
                trending_score: 95,
                seasonal_peak: Some("january".to_string()),
                prompt_template: "Create a daily planner with the following sections:
1. Morning routine ({morning_routine})
2. Hourly schedule (6am-10pm)
3. Top 3 priorities
4. Task list
5. Notes section
6. Evening reflection

Style: {style}
Color scheme: {color_scheme}".to_string(),
                output_format: OutputFormat::Pdf,
                parameters: vec![
                    TemplateParameter {
                        name: "morning_routine".to_string(),
                        description: "Morning routine duration".to_string(),
                        param_type: ParameterType::Select(vec!["15 min".to_string(), "30 min".to_string(), "1 hour".to_string()]),
                        required: true,
                        default: Some("30 min".to_string()),
                    },
                    TemplateParameter {
                        name: "style".to_string(),
                        description: "Planner style".to_string(),
                        param_type: ParameterType::Select(vec!["Minimal".to_string(), "Decorative".to_string(), "Professional".to_string()]),
                        required: true,
                        default: Some("Minimal".to_string()),
                    },
                    TemplateParameter {
                        name: "color_scheme".to_string(),
                        description: "Color scheme".to_string(),
                        param_type: ParameterType::Color,
                        required: false,
                        default: Some("#4A90D9".to_string()),
                    },
                ],
            },
            Template {
                id: "gratitude_journal".to_string(),
                name: "Gratitude Journal".to_string(),
                description: "Daily gratitude practice journal with prompts".to_string(),
                category: TemplateCategory::Journal,
                tags: vec!["journal".to_string(), "gratitude".to_string(), "wellness".to_string()],
                trending_score: 88,
                seasonal_peak: Some("november".to_string()),
                prompt_template: "Create a gratitude journal with:
1. Daily gratitude prompts ({prompt_count} prompts)
2. Reflection section
3. Weekly summary
4. Monthly review

Theme: {theme}
Pages: {page_count}".to_string(),
                output_format: OutputFormat::Pdf,
                parameters: vec![
                    TemplateParameter {
                        name: "prompt_count".to_string(),
                        description: "Number of daily prompts".to_string(),
                        param_type: ParameterType::Select(vec!["3".to_string(), "5".to_string(), "10".to_string()]),
                        required: true,
                        default: Some("3".to_string()),
                    },
                    TemplateParameter {
                        name: "theme".to_string(),
                        description: "Journal theme".to_string(),
                        param_type: ParameterType::Select(vec!["Nature".to_string(), "Minimal".to_string(), "Colorful".to_string()]),
                        required: true,
                        default: Some("Nature".to_string()),
                    },
                    TemplateParameter {
                        name: "page_count".to_string(),
                        description: "Number of pages".to_string(),
                        param_type: ParameterType::Select(vec!["30".to_string(), "90".to_string(), "365".to_string()]),
                        required: true,
                        default: Some("90".to_string()),
                    },
                ],
            },
            Template {
                id: "budget_tracker".to_string(),
                name: "Budget Tracker".to_string(),
                description: "Monthly budget spreadsheet with categories and charts".to_string(),
                category: TemplateCategory::Spreadsheet,
                tags: vec!["budget".to_string(), "finance".to_string(), "spreadsheet".to_string()],
                trending_score: 92,
                seasonal_peak: Some("january".to_string()),
                prompt_template: "Create a budget tracker spreadsheet with:
1. Income tracking
2. Expense categories: {categories}
3. Monthly summary
4. Charts and graphs
5. Annual overview

Currency: {currency}
Complexity: {complexity}".to_string(),
                output_format: OutputFormat::Xlsx,
                parameters: vec![
                    TemplateParameter {
                        name: "categories".to_string(),
                        description: "Expense categories".to_string(),
                        param_type: ParameterType::Text,
                        required: true,
                        default: Some("Housing, Food, Transport, Entertainment, Savings".to_string()),
                    },
                    TemplateParameter {
                        name: "currency".to_string(),
                        description: "Currency symbol".to_string(),
                        param_type: ParameterType::Select(vec!["$".to_string(), "€".to_string(), "£".to_string(), "¥".to_string()]),
                        required: true,
                        default: Some("$".to_string()),
                    },
                    TemplateParameter {
                        name: "complexity".to_string(),
                        description: "Complexity level".to_string(),
                        param_type: ParameterType::Select(vec!["Simple".to_string(), "Detailed".to_string(), "Advanced".to_string()]),
                        required: true,
                        default: Some("Detailed".to_string()),
                    },
                ],
            },
            Template {
                id: "freelance_contract".to_string(),
                name: "Freelance Contract".to_string(),
                description: "Professional freelance service agreement".to_string(),
                category: TemplateCategory::Legal,
                tags: vec!["contract".to_string(), "freelance".to_string(), "legal".to_string()],
                trending_score: 85,
                seasonal_peak: None,
                prompt_template: "Create a freelance contract with:
1. Parties: {client_name} and {freelancer_name}
2. Services: {service_description}
3. Payment: {payment_terms}
4. Timeline: {timeline}
5. Revisions: {revision_count}
6. Jurisdiction: {jurisdiction}

Include standard clauses for intellectual property, termination, and dispute resolution.".to_string(),
                output_format: OutputFormat::Docx,
                parameters: vec![
                    TemplateParameter {
                        name: "client_name".to_string(),
                        description: "Client name".to_string(),
                        param_type: ParameterType::Text,
                        required: true,
                        default: None,
                    },
                    TemplateParameter {
                        name: "freelancer_name".to_string(),
                        description: "Freelancer name".to_string(),
                        param_type: ParameterType::Text,
                        required: true,
                        default: None,
                    },
                    TemplateParameter {
                        name: "service_description".to_string(),
                        description: "Description of services".to_string(),
                        param_type: ParameterType::Text,
                        required: true,
                        default: None,
                    },
                    TemplateParameter {
                        name: "payment_terms".to_string(),
                        description: "Payment terms".to_string(),
                        param_type: ParameterType::Select(vec!["50% upfront, 50% on completion".to_string(), "100% upfront".to_string(), "100% on completion".to_string(), "Monthly billing".to_string()]),
                        required: true,
                        default: Some("50% upfront, 50% on completion".to_string()),
                    },
                    TemplateParameter {
                        name: "timeline".to_string(),
                        description: "Project timeline".to_string(),
                        param_type: ParameterType::Text,
                        required: true,
                        default: Some("30 days".to_string()),
                    },
                    TemplateParameter {
                        name: "revision_count".to_string(),
                        description: "Number of revisions included".to_string(),
                        param_type: ParameterType::Select(vec!["1".to_string(), "2".to_string(), "3".to_string(), "Unlimited".to_string()]),
                        required: true,
                        default: Some("2".to_string()),
                    },
                    TemplateParameter {
                        name: "jurisdiction".to_string(),
                        description: "Governing law jurisdiction".to_string(),
                        param_type: ParameterType::Text,
                        required: true,
                        default: Some("California, USA".to_string()),
                    },
                ],
            },
        ];
        
        // Add new digital product templates from user's research
        templates.extend(vec![
            // Digital Stickers
            Template {
                id: "digital_stickers_pack".to_string(),
                name: "Digital Stickers Pack".to_string(),
                description: "Sticker packs for OneNote, GoodNotes, and other note-taking apps. High-margin digital product.".to_string(),
                category: TemplateCategory::DigitalStickers,
                tags: vec!["stickers".to_string(), "goodnotes".to_string(), "onenote".to_string(), "high-margin".to_string()],
                trending_score: 95,
                seasonal_peak: Some("september".to_string()),
                prompt_template: "Create digital stickers with theme: {theme}

Style: {style}
Format: PNG with transparent background
Size: Optimized for tablet apps (OneNote, GoodNotes)

AI Prompt for Midjourney 5:
'{theme} stickers, cute kawaii style, isolated on white background, clean vector art, vibrant colors --ar 1:1 --v 5'

Pack includes:
- 20-30 individual stickers
- Pre-cropped PNG files
- GoodNotes/OneNote optimized
- Commercial license included

Time investment: 1 day per month".to_string(),
                output_format: OutputFormat::Pdf,
                parameters: vec![
                    TemplateParameter {
                        name: "theme".to_string(),
                        description: "Sticker theme".to_string(),
                        param_type: ParameterType::Select(vec!["Cute Animals".to_string(), "Plants & Nature".to_string(), "Food & Drinks".to_string(), "Planner Icons".to_string(), "Motivational Quotes".to_string(), "Seasonal".to_string()]),
                        required: true,
                        default: Some("Cute Animals".to_string()),
                    },
                    TemplateParameter {
                        name: "style".to_string(),
                        description: "Art style".to_string(),
                        param_type: ParameterType::Select(vec!["Kawaii".to_string(), "Watercolor".to_string(), "Flat Vector".to_string(), "Hand-drawn".to_string()]),
                        required: true,
                        default: Some("Kawaii".to_string()),
                    },
                ],
            },
            // Digital Art
            Template {
                id: "digital_art_printable".to_string(),
                name: "Digital Art Printables".to_string(),
                description: "AI-generated art resized for printable downloads. Popular for wall art and home decor.".to_string(),
                category: TemplateCategory::DigitalArt,
                tags: vec!["art".to_string(), "printable".to_string(), "wall-art".to_string(), "ai-generated".to_string()],
                trending_score: 90,
                seasonal_peak: None,
                prompt_template: "Generate digital art for printable downloads.

Theme: {theme}
Style: {art_style}

AI Prompt:
'{theme} in {art_style} style, high quality, detailed, suitable for printing --ar {aspect_ratio}'

Aspect Ratios for Products:
- Mug: --ar 293:151
- Standard print: --ar 3:4
- Landscape: --ar 16:9
- Square: --ar 1:1

Deliverables:
- High-res JPG (300 DPI)
- Multiple sizes included
- Print-ready files
- Commercial license

Time investment: 1 day per month".to_string(),
                output_format: OutputFormat::Pdf,
                parameters: vec![
                    TemplateParameter {
                        name: "theme".to_string(),
                        description: "Art theme".to_string(),
                        param_type: ParameterType::Select(vec!["Abstract".to_string(), "Nature".to_string(), "Botanical".to_string(), "Geometric".to_string(), "Landscape".to_string(), "Minimalist".to_string()]),
                        required: true,
                        default: Some("Botanical".to_string()),
                    },
                    TemplateParameter {
                        name: "art_style".to_string(),
                        description: "Art style".to_string(),
                        param_type: ParameterType::Select(vec!["Watercolor".to_string(), "Oil Painting".to_string(), "Digital Art".to_string(), "Line Art".to_string(), "Photography".to_string()]),
                        required: true,
                        default: Some("Watercolor".to_string()),
                    },
                    TemplateParameter {
                        name: "aspect_ratio".to_string(),
                        description: "Target aspect ratio".to_string(),
                        param_type: ParameterType::Select(vec!["293:151 (Mug)".to_string(), "3:4 (Standard Print)".to_string(), "16:9 (Landscape)".to_string(), "1:1 (Square)".to_string()]),
                        required: true,
                        default: Some("3:4 (Standard Print)".to_string()),
                    },
                ],
            },
            // Clip Art
            Template {
                id: "clip_art_bundle".to_string(),
                name: "Clip Art Bundle".to_string(),
                description: "Pre-made graphics for documents, presentations, and products. High demand on Etsy.".to_string(),
                category: TemplateCategory::ClipArt,
                tags: vec!["clipart".to_string(), "graphics".to_string(), "presentation".to_string(), "commercial-use".to_string()],
                trending_score: 88,
                seasonal_peak: None,
                prompt_template: "Create clip art graphics bundle.

Category: {category}
Style: {style}

AI Prompt:
'{category} clip art set, {style}, isolated on transparent background, clean edges, professional quality --ar 1:1'

Bundle includes:
- 50+ individual graphics
- PNG with transparency
- SVG vector files
- Commercial license
- Organized by category

Use for:
- Presentations
- Documents
- Product designs
- Social media

Time investment: 1 day per month".to_string(),
                output_format: OutputFormat::Pdf,
                parameters: vec![
                    TemplateParameter {
                        name: "category".to_string(),
                        description: "Clip art category".to_string(),
                        param_type: ParameterType::Select(vec!["Business Icons".to_string(), "Floral Elements".to_string(), "Seasonal Decorations".to_string(), "Food & Kitchen".to_string(), "Arrows & Symbols".to_string(), "People & Characters".to_string()]),
                        required: true,
                        default: Some("Business Icons".to_string()),
                    },
                    TemplateParameter {
                        name: "style".to_string(),
                        description: "Visual style".to_string(),
                        param_type: ParameterType::Select(vec!["Flat Design".to_string(), "3D Render".to_string(), "Hand-drawn".to_string(), "Watercolor".to_string(), "Line Art".to_string()]),
                        required: true,
                        default: Some("Flat Design".to_string()),
                    },
                ],
            },
            // Coloring Pages
            Template {
                id: "coloring_pages_book".to_string(),
                name: "Adult Coloring Pages".to_string(),
                description: "Intricate coloring pages for adults. Mandalas, animals, landscapes. High seller on Etsy.".to_string(),
                category: TemplateCategory::ColoringPages,
                tags: vec!["coloring".to_string(), "adult-coloring".to_string(), "mandalas".to_string(), "stress-relief".to_string()],
                trending_score: 92,
                seasonal_peak: Some("december".to_string()),
                prompt_template: "Create intricate adult coloring pages.

Type: {page_type}
Complexity: {complexity}

AI Prompt for DALL-E/Artistly:
'Intricate {page_type} coloring page for adults, black and white line art, high detail, no shading, clean lines, printable quality'

Or use MarketingBlocks/Synthesys for vector output.

Book includes:
- 30-50 unique designs
- Single-sided pages
- 8.5x11 inch format
- PDF for easy printing
- Cover design included

Popular themes:
- Intricate mandalas
- Detailed animals
- Fantasy landscapes
- Geometric patterns

Time investment: 1 day per month".to_string(),
                output_format: OutputFormat::Pdf,
                parameters: vec![
                    TemplateParameter {
                        name: "page_type".to_string(),
                        description: "Coloring page type".to_string(),
                        param_type: ParameterType::Select(vec!["Mandalas".to_string(), "Animals".to_string(), "Landscapes".to_string(), "Floral".to_string(), "Fantasy".to_string(), "Geometric".to_string()]),
                        required: true,
                        default: Some("Mandalas".to_string()),
                    },
                    TemplateParameter {
                        name: "complexity".to_string(),
                        description: "Detail level".to_string(),
                        param_type: ParameterType::Select(vec!["Simple".to_string(), "Moderate".to_string(), "Intricate".to_string(), "Expert".to_string()]),
                        required: true,
                        default: Some("Intricate".to_string()),
                    },
                ],
            },
            // Logo Design
            Template {
                id: "logo_design_template".to_string(),
                name: "Logo Design Pack".to_string(),
                description: "Simple logos for small businesses, startups, and side hustles.".to_string(),
                category: TemplateCategory::LogoDesign,
                tags: vec!["logo".to_string(), "branding".to_string(), "business".to_string(), "minimal".to_string()],
                trending_score: 87,
                seasonal_peak: Some("january".to_string()),
                prompt_template: "Create professional logo designs.

Business Type: {business_type}
Style: {logo_style}

AI Tools: DesignBeast, Logoai, Looka, Artistly

Prompt for AI:
'{business_type} logo, {logo_style}, professional, minimalist, vector style, clean typography, memorable icon --ar 1:1'

Pack includes:
- 3 logo variations
- Color + B&W versions
- Horizontal & vertical layouts
- Source files (if applicable)
- Usage guidelines

Target: Small businesses, startups, side hustles

Time investment: 1 day per month".to_string(),
                output_format: OutputFormat::Pdf,
                parameters: vec![
                    TemplateParameter {
                        name: "business_type".to_string(),
                        description: "Target business type".to_string(),
                        param_type: ParameterType::Select(vec!["Coffee Shop".to_string(), "Tech Startup".to_string(), "Boutique".to_string(), "Consulting".to_string(), "Fitness".to_string(), "Creative Agency".to_string()]),
                        required: true,
                        default: Some("Coffee Shop".to_string()),
                    },
                    TemplateParameter {
                        name: "logo_style".to_string(),
                        description: "Logo style".to_string(),
                        param_type: ParameterType::Select(vec!["Minimalist".to_string(), "Vintage".to_string(), "Modern".to_string(), "Playful".to_string(), "Luxury".to_string()]),
                        required: true,
                        default: Some("Minimalist".to_string()),
                    },
                ],
            },
            // Notion Templates
            Template {
                id: "notion_template".to_string(),
                name: "Notion Template".to_string(),
                description: "Productivity templates for Notion. High demand from students and professionals.".to_string(),
                category: TemplateCategory::NotionTemplates,
                tags: vec!["notion".to_string(), "productivity".to_string(), "template".to_string(), "organization".to_string()],
                trending_score: 94,
                seasonal_peak: Some("september".to_string()),
                prompt_template: "Create a Notion template.

Template Type: {template_type}
Target User: {target_user}

Structure includes:
- Dashboard with overview
- Organized databases
- Pre-built views (table, board, calendar)
- Instruction page
- Template duplication link

Popular templates:
- Second brain / PKM
- Project management
- Habit tracker
- Content calendar
- Budget tracker
- Student planner

Use AI to generate:
- Icon designs (Midjourney)
- Cover images
- Content suggestions

Time investment: 1 day per month".to_string(),
                output_format: OutputFormat::Markdown,
                parameters: vec![
                    TemplateParameter {
                        name: "template_type".to_string(),
                        description: "Template purpose".to_string(),
                        param_type: ParameterType::Select(vec!["Second Brain".to_string(), "Project Management".to_string(), "Habit Tracker".to_string(), "Content Calendar".to_string(), "Budget Tracker".to_string(), "Student Planner".to_string()]),
                        required: true,
                        default: Some("Second Brain".to_string()),
                    },
                    TemplateParameter {
                        name: "target_user".to_string(),
                        description: "Target audience".to_string(),
                        param_type: ParameterType::Select(vec!["Students".to_string(), "Professionals".to_string(), "Creators".to_string(), "Small Business".to_string(), "Freelancers".to_string()]),
                        required: true,
                        default: Some("Professionals".to_string()),
                    },
                ],
            },
            // Printables
            Template {
                id: "printables_pack".to_string(),
                name: "Printables & Planners".to_string(),
                description: "Printable planners, trackers, and organizers for productivity and organization.".to_string(),
                category: TemplateCategory::Printables,
                tags: vec!["printables".to_string(), "planner".to_string(), "tracker".to_string(), "organization".to_string()],
                trending_score: 91,
                seasonal_peak: Some("january".to_string()),
                prompt_template: "Create printable planners and organizers.

Type: {printable_type}
Format: {format}

Design in Canva or use AI Printable Automation

Pack includes:
- Multiple page layouts
- Cover page
- Instructions for printing
- PDF optimized for home printing
- US Letter & A4 sizes

Popular printables:
- Daily/weekly planners
- Budget trackers
- Habit trackers
- Meal planners
- Goal setting worksheets
- Real estate marketing materials
- Resume templates

Time investment: 1 day per month".to_string(),
                output_format: OutputFormat::Pdf,
                parameters: vec![
                    TemplateParameter {
                        name: "printable_type".to_string(),
                        description: "Type of printable".to_string(),
                        param_type: ParameterType::Select(vec!["Daily Planner".to_string(), "Budget Tracker".to_string(), "Habit Tracker".to_string(), "Meal Planner".to_string(), "Goal Worksheet".to_string(), "Resume Template".to_string()]),
                        required: true,
                        default: Some("Daily Planner".to_string()),
                    },
                    TemplateParameter {
                        name: "format".to_string(),
                        description: "File format".to_string(),
                        param_type: ParameterType::Select(vec!["PDF (Printable)".to_string(), "Google Docs".to_string(), "Microsoft Word".to_string(), "Canva Template".to_string()]),
                        required: true,
                        default: Some("PDF (Printable)".to_string()),
                    },
                ],
            },
            // POD Designs
            Template {
                id: "pod_design_pack".to_string(),
                name: "Print-on-Demand Designs".to_string(),
                description: "AI designs for mugs, shirts, hoodies, and other print-on-demand products.".to_string(),
                category: TemplateCategory::PodDesigns,
                tags: vec!["pod".to_string(), "print-on-demand".to_string(), "mug-designs".to_string(), "tshirt-designs".to_string()],
                trending_score: 89,
                seasonal_peak: None,
                prompt_template: "Create print-on-demand designs.

Product: {product_type}
Theme: {theme}

AI Prompt for Midjourney:
'{theme} design, {style}, sharp focus, on pure white background, print-ready, high quality --ar {aspect_ratio}'

Aspect Ratios:
- Mug: --ar 293:151
- T-shirt: --ar 1:1
- All-over hoodie: --tile

Add funny puns with ChatGPT (trademark-free):
- Generate 10 puns about {theme}
- Check on USPTO.gov
- Pair with matching images

Pro tip: Find best-selling digital downloads on Etsy, mock up on POD products with Photoshop smart objects

Tools:
- Subliminator for all-over print hoodies
- Printful/Printify for standard products
- Photoshop for enhancement

Time investment: 1 day per month".to_string(),
                output_format: OutputFormat::Pdf,
                parameters: vec![
                    TemplateParameter {
                        name: "product_type".to_string(),
                        description: "POD product".to_string(),
                        param_type: ParameterType::Select(vec!["Mug".to_string(), "T-Shirt".to_string(), "Hoodie".to_string(), "Phone Case".to_string(), "Tote Bag".to_string()]),
                        required: true,
                        default: Some("Mug".to_string()),
                    },
                    TemplateParameter {
                        name: "theme".to_string(),
                        description: "Design theme".to_string(),
                        param_type: ParameterType::Select(vec!["Funny Quotes".to_string(), "Nature".to_string(), "Animals".to_string(), "Hobbies".to_string(), "Professions".to_string(), "Seasonal".to_string()]),
                        required: true,
                        default: Some("Funny Quotes".to_string()),
                    },
                    TemplateParameter {
                        name: "style".to_string(),
                        description: "Art style".to_string(),
                        param_type: ParameterType::Select(vec!["Watercolor".to_string(), "Retro/Vintage".to_string(), "Minimalist".to_string(), "Bold Typography".to_string(), "Illustration".to_string()]),
                        required: true,
                        default: Some("Watercolor".to_string()),
                    },
                    TemplateParameter {
                        name: "aspect_ratio".to_string(),
                        description: "Image aspect ratio".to_string(),
                        param_type: ParameterType::Select(vec!["293:151 (Mug)".to_string(), "1:1 (Square)".to_string(), "4:5 (Portrait)".to_string(), "Tile Pattern".to_string()]),
                        required: true,
                        default: Some("293:151 (Mug)".to_string()),
                    },
                ],
            },
        ]);
        
        for template in templates {
            self.templates.insert(template.id.clone(), template);
        }
    }
    
    pub fn get(&self, id: &str) -> Option<&Template> {
        self.templates.get(id)
    }
    
    pub fn list(&self) -> Vec<&Template> {
        self.templates.values().collect()
    }
    
    pub fn by_category(&self, category: TemplateCategory) -> Vec<&Template> {
        self.templates.values()
            .filter(|t| t.category == category)
            .collect()
    }
    
    pub fn search(&self, query: &str) -> Vec<&Template> {
        let query_lower = query.to_lowercase();
        self.templates.values()
            .filter(|t| {
                t.name.to_lowercase().contains(&query_lower) ||
                t.description.to_lowercase().contains(&query_lower) ||
                t.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
            })
            .collect()
    }
    
    pub fn trending(&self, limit: usize) -> Vec<&Template> {
        let mut templates: Vec<_> = self.templates.values().collect();
        templates.sort_by(|a, b| b.trending_score.cmp(&a.trending_score));
        templates.into_iter().take(limit).collect()
    }
}
