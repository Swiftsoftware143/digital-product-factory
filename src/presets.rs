//! Industry Presets - Pre-configured pipelines for different digital business models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A pipeline stage definition with recommended modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetStage {
    pub name: String,
    pub emoji: String,
    pub description: String,
    pub recommended_modules: Vec<ModuleType>,
    pub actions: Vec<String>,
    pub output_description: String,
}

/// Types of modules available in the app
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModuleType {
    Pipeline,
    AiGeneration,
    MarketResearch,
    ContractGenerator,
    Scheduler,
    BundleBuilder,
    ExportPdf,
    ExportDocx,
    ExportXlsx,
    ExportZip,
    Notes,
}

impl ModuleType {
    pub fn name(&self) -> &'static str {
        match self {
            ModuleType::Pipeline => "Pipeline Kanban",
            ModuleType::AiGeneration => "AI Generation",
            ModuleType::MarketResearch => "Market Research",
            ModuleType::ContractGenerator => "Contract Generator",
            ModuleType::Scheduler => "Scheduler",
            ModuleType::BundleBuilder => "Bundle Builder",
            ModuleType::ExportPdf => "Export PDF",
            ModuleType::ExportDocx => "Export DOCX",
            ModuleType::ExportXlsx => "Export XLSX",
            ModuleType::ExportZip => "Export ZIP",
            ModuleType::Notes => "Notes",
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            ModuleType::Pipeline => "📋",
            ModuleType::AiGeneration => "🤖",
            ModuleType::MarketResearch => "📊",
            ModuleType::ContractGenerator => "📄",
            ModuleType::Scheduler => "📅",
            ModuleType::BundleBuilder => "📦",
            ModuleType::ExportPdf => "📕",
            ModuleType::ExportDocx => "📘",
            ModuleType::ExportXlsx => "📗",
            ModuleType::ExportZip => "🗜️",
            ModuleType::Notes => "📝",
        }
    }
}

/// A complete industry preset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndustryPreset {
    pub id: String,
    pub name: String,
    pub emoji: String,
    pub description: String,
    pub best_for: Vec<String>,
    pub stages: Vec<PresetStage>,
    pub quick_tips: Vec<String>,
}

/// Registry of all available presets
pub struct PresetRegistry {
    presets: HashMap<String, IndustryPreset>,
}

impl PresetRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            presets: HashMap::new(),
        };
        registry.load_all_presets();
        registry
    }
    
    fn load_all_presets(&mut self) {
        let presets = vec![
            Self::affiliate_marketing_preset(),
            Self::content_creator_preset(),
            Self::creator_affiliate_hybrid_preset(),
            Self::elearning_preset(),
            Self::saas_preset(),
            Self::marketing_agency_preset(),
            Self::freelancer_preset(),
            Self::info_products_preset(),
            Self::business_setup_preset(),
        ];
        
        for preset in presets {
            self.presets.insert(preset.id.clone(), preset);
        }
    }
    
    /// Affiliate Marketing Preset
    fn affiliate_marketing_preset() -> IndustryPreset {
        IndustryPreset {
            id: "affiliate_marketing".to_string(),
            name: "Affiliate Marketing".to_string(),
            emoji: "💰".to_string(),
            description: "Promote products for commissions with bonus stacking and email sequences".to_string(),
            best_for: vec![
                "Product reviewers".to_string(),
                "Email marketers".to_string(),
                "Comparison site owners".to_string(),
                "YouTube reviewers".to_string(),
            ],
            stages: vec![
                PresetStage {
                    name: "Research".to_string(),
                    emoji: "🔍".to_string(),
                    description: "Find products & validate demand".to_string(),
                    recommended_modules: vec![ModuleType::MarketResearch, ModuleType::Notes],
                    actions: vec![
                        "Analyze niche competition".to_string(),
                        "Compare commission rates".to_string(),
                        "Validate audience demand".to_string(),
                    ],
                    output_description: "Niche analysis, top products list".to_string(),
                },
                PresetStage {
                    name: "Ideation".to_string(),
                    emoji: "💡".to_string(),
                    description: "Plan review/promo angles".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration, ModuleType::Notes],
                    actions: vec![
                        "Generate content hooks".to_string(),
                        "Create angle variations".to_string(),
                        "Draft headlines".to_string(),
                    ],
                    output_description: "Content angles, hook library".to_string(),
                },
                PresetStage {
                    name: "Create".to_string(),
                    emoji: "✍️".to_string(),
                    description: "Build the actual content".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration, ModuleType::Pipeline],
                    actions: vec![
                        "Write review copy".to_string(),
                        "Create email sequence".to_string(),
                        "Draft social posts".to_string(),
                        "Script video content".to_string(),
                    ],
                    output_description: "Review content, email series, scripts".to_string(),
                },
                PresetStage {
                    name: "Legal".to_string(),
                    emoji: "⚖️".to_string(),
                    description: "Compliance & disclaimers".to_string(),
                    recommended_modules: vec![ModuleType::ContractGenerator],
                    actions: vec![
                        "Add affiliate disclosure".to_string(),
                        "Generate FTC compliance text".to_string(),
                        "Create terms page".to_string(),
                    ],
                    output_description: "Disclosure statements, compliance docs".to_string(),
                },
                PresetStage {
                    name: "Bundle".to_string(),
                    emoji: "🎁".to_string(),
                    description: "Package bonuses".to_string(),
                    recommended_modules: vec![ModuleType::BundleBuilder],
                    actions: vec![
                        "Create bonus stack".to_string(),
                        "Build lead magnets".to_string(),
                        "Package deliverables".to_string(),
                    ],
                    output_description: "Bonus package, lead magnet bundle".to_string(),
                },
                PresetStage {
                    name: "Schedule".to_string(),
                    emoji: "📅".to_string(),
                    description: "Time the launch".to_string(),
                    recommended_modules: vec![ModuleType::Scheduler],
                    actions: vec![
                        "Queue email sequences".to_string(),
                        "Schedule social posts".to_string(),
                        "Set launch timing".to_string(),
                    ],
                    output_description: "Scheduled campaign, timed releases".to_string(),
                },
                PresetStage {
                    name: "Publish".to_string(),
                    emoji: "🚀".to_string(),
                    description: "Go live".to_string(),
                    recommended_modules: vec![ModuleType::ExportPdf, ModuleType::ExportZip],
                    actions: vec![
                        "Export lead magnets".to_string(),
                        "Package bonus downloads".to_string(),
                        "Publish review content".to_string(),
                    ],
                    output_description: "Live campaign, downloadable bonuses".to_string(),
                },
                PresetStage {
                    name: "Analyze".to_string(),
                    emoji: "📈".to_string(),
                    description: "Track performance".to_string(),
                    recommended_modules: vec![ModuleType::Notes, ModuleType::ExportXlsx],
                    actions: vec![
                        "Log conversion rates".to_string(),
                        "Track EPC data".to_string(),
                        "Optimize underperformers".to_string(),
                    ],
                    output_description: "Performance report, optimization notes".to_string(),
                },
            ],
            quick_tips: vec![
                "Always disclose affiliate relationships - it's the law".to_string(),
                "Bonus stacks increase conversions by 30-50%".to_string(),
                "Email sequences outperform single blasts 3:1".to_string(),
                "Review products you've actually used for authenticity".to_string(),
            ],
        }
    }
    
    /// Content Creator / Influencer Preset
    fn content_creator_preset() -> IndustryPreset {
        IndustryPreset {
            id: "content_creator".to_string(),
            name: "Content Creator / Influencer".to_string(),
            emoji: "🎬".to_string(),
            description: "Grow audience and land brand deals with consistent content".to_string(),
            best_for: vec![
                "YouTubers".to_string(),
                "TikTok creators".to_string(),
                "Instagram influencers".to_string(),
                "Podcasters".to_string(),
                "Newsletter writers".to_string(),
            ],
            stages: vec![
                PresetStage {
                    name: "Ideation".to_string(),
                    emoji: "💭".to_string(),
                    description: "Brainstorm content".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration, ModuleType::Notes],
                    actions: vec![
                        "Generate content themes".to_string(),
                        "Create series ideas".to_string(),
                        "Draft viral hooks".to_string(),
                    ],
                    output_description: "Content calendar, series plan".to_string(),
                },
                PresetStage {
                    name: "Sponsor Deals".to_string(),
                    emoji: "🤝".to_string(),
                    description: "Manage brand partnerships".to_string(),
                    recommended_modules: vec![ModuleType::ContractGenerator, ModuleType::Notes],
                    actions: vec![
                        "Draft sponsorship agreements".to_string(),
                        "Create SOWs".to_string(),
                        "Set rate cards".to_string(),
                    ],
                    output_description: "Signed contracts, deliverables list".to_string(),
                },
                PresetStage {
                    name: "Production".to_string(),
                    emoji: "🎥".to_string(),
                    description: "Create the content".to_string(),
                    recommended_modules: vec![ModuleType::Pipeline, ModuleType::Scheduler],
                    actions: vec![
                        "Script the content".to_string(),
                        "Film/record".to_string(),
                        "Edit and polish".to_string(),
                        "Review and approve".to_string(),
                    ],
                    output_description: "Finished content, ready to publish".to_string(),
                },
                PresetStage {
                    name: "Legal".to_string(),
                    emoji: "⚖️".to_string(),
                    description: "Protect yourself".to_string(),
                    recommended_modules: vec![ModuleType::ContractGenerator],
                    actions: vec![
                        "Set usage rights".to_string(),
                        "Define revision limits".to_string(),
                        "Lock payment terms".to_string(),
                    ],
                    output_description: "Rights agreements, payment protection".to_string(),
                },
                PresetStage {
                    name: "Assets".to_string(),
                    emoji: "📁".to_string(),
                    description: "Build deliverables".to_string(),
                    recommended_modules: vec![ModuleType::BundleBuilder],
                    actions: vec![
                        "Create media kits".to_string(),
                        "Build press packs".to_string(),
                        "Package portfolio samples".to_string(),
                    ],
                    output_description: "Media kit, portfolio bundle".to_string(),
                },
                PresetStage {
                    name: "Schedule".to_string(),
                    emoji: "📅".to_string(),
                    description: "Content calendar".to_string(),
                    recommended_modules: vec![ModuleType::Scheduler],
                    actions: vec![
                        "Queue posts".to_string(),
                        "Set premiere times".to_string(),
                        "Plan story sequences".to_string(),
                    ],
                    output_description: "Scheduled content calendar".to_string(),
                },
                PresetStage {
                    name: "Export".to_string(),
                    emoji: "📤".to_string(),
                    description: "Deliver to brands".to_string(),
                    recommended_modules: vec![ModuleType::ExportXlsx, ModuleType::ExportPdf],
                    actions: vec![
                        "Generate campaign reports".to_string(),
                        "Export analytics summaries".to_string(),
                        "Deliver final assets".to_string(),
                    ],
                    output_description: "Campaign reports, analytics docs".to_string(),
                },
                PresetStage {
                    name: "Archive".to_string(),
                    emoji: "🗃️".to_string(),
                    description: "Repurpose later".to_string(),
                    recommended_modules: vec![ModuleType::Notes, ModuleType::Pipeline],
                    actions: vec![
                        "Tag for future remix".to_string(),
                        "Log clip extraction opportunities".to_string(),
                        "Build content library".to_string(),
                    ],
                    output_description: "Organized archive, repurposing notes".to_string(),
                },
            ],
            quick_tips: vec![
                "Consistency beats virality - post on a schedule".to_string(),
                "Media kits 10x your sponsorship close rate".to_string(),
                "Always get contracts in writing, even for 'small' deals".to_string(),
                "Repurpose every piece of content across 3+ platforms".to_string(),
            ],
        }
    }
    
    /// Creator + Affiliate Hybrid Preset
    fn creator_affiliate_hybrid_preset() -> IndustryPreset {
        IndustryPreset {
            id: "creator_affiliate_hybrid".to_string(),
            name: "Creator + Affiliate Hybrid".to_string(),
            emoji: "🌟".to_string(),
            description: "Combine brand deals with affiliate revenue for multiple income streams".to_string(),
            best_for: vec![
                "YouTubers with affiliate links".to_string(),
                "Newsletter writers".to_string(),
                "Review channel owners".to_string(),
                "Multi-monetization creators".to_string(),
            ],
            stages: vec![
                PresetStage {
                    name: "Ideate".to_string(),
                    emoji: "💡".to_string(),
                    description: "Content + affiliate angle".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration, ModuleType::Notes],
                    actions: vec![
                        "Blend content ideas with affiliate opportunities".to_string(),
                        "Map organic content to products".to_string(),
                        "Plan soft-sell integration".to_string(),
                    ],
                    output_description: "Content plan with embedded affiliate strategy".to_string(),
                },
                PresetStage {
                    name: "Pitch".to_string(),
                    emoji: "📨".to_string(),
                    description: "Brand deals OR affiliate approvals".to_string(),
                    recommended_modules: vec![ModuleType::ContractGenerator, ModuleType::Notes],
                    actions: vec![
                        "Pitch sponsors".to_string(),
                        "Apply to affiliate programs".to_string(),
                        "Negotiate hybrid deals".to_string(),
                    ],
                    output_description: "Sponsor contracts + affiliate program access".to_string(),
                },
                PresetStage {
                    name: "Create".to_string(),
                    emoji: "🎨".to_string(),
                    description: "Authentic content with soft-sell".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration, ModuleType::Pipeline],
                    actions: vec![
                        "Create value-first content".to_string(),
                        "Integrate affiliate naturally".to_string(),
                        "Balance sponsored + organic".to_string(),
                    ],
                    output_description: "Content with embedded recommendations".to_string(),
                },
                PresetStage {
                    name: "Legal Shield".to_string(),
                    emoji: "🛡️".to_string(),
                    description: "Sponsor terms + affiliate disclosure".to_string(),
                    recommended_modules: vec![ModuleType::ContractGenerator],
                    actions: vec![
                        "Combine sponsor terms with affiliate disclosure".to_string(),
                        "Set clear #ad labeling".to_string(),
                        "Protect with proper contracts".to_string(),
                    ],
                    output_description: "Dual-compliant legal documentation".to_string(),
                },
                PresetStage {
                    name: "Value Stack".to_string(),
                    emoji: "🎁".to_string(),
                    description: "Freebie + affiliate bonus".to_string(),
                    recommended_modules: vec![ModuleType::BundleBuilder],
                    actions: vec![
                        "Create list-building freebie".to_string(),
                        "Stack affiliate bonus".to_string(),
                        "Package sponsor deliverables".to_string(),
                    ],
                    output_description: "Lead magnet + conversion bonus bundle".to_string(),
                },
                PresetStage {
                    name: "Queue".to_string(),
                    emoji: "⏰".to_string(),
                    description: "Organic + affiliate timing".to_string(),
                    recommended_modules: vec![ModuleType::Scheduler],
                    actions: vec![
                        "Schedule organic posts".to_string(),
                        "Time affiliate drops".to_string(),
                        "Coordinate sponsor deadlines".to_string(),
                    ],
                    output_description: "Integrated content calendar".to_string(),
                },
                PresetStage {
                    name: "Package".to_string(),
                    emoji: "📦".to_string(),
                    description: "Sponsor deliverables + lead magnets".to_string(),
                    recommended_modules: vec![ModuleType::ExportPdf, ModuleType::ExportDocx, ModuleType::ExportZip],
                    actions: vec![
                        "Deliver sponsor reports".to_string(),
                        "Export lead magnets".to_string(),
                        "Package bonus downloads".to_string(),
                    ],
                    output_description: "Complete deliverable package".to_string(),
                },
                PresetStage {
                    name: "Split Test".to_string(),
                    emoji: "🧪".to_string(),
                    description: "Organic vs affiliate performance".to_string(),
                    recommended_modules: vec![ModuleType::Notes, ModuleType::ExportXlsx],
                    actions: vec![
                        "Track organic content metrics".to_string(),
                        "Log affiliate conversion rates".to_string(),
                        "Compare monetization strategies".to_string(),
                    ],
                    output_description: "Performance comparison, optimization insights".to_string(),
                },
            ],
            quick_tips: vec![
                "The best affiliate content doesn't feel like affiliate content".to_string(),
                "Hybrid creators earn 2-3x more than single-stream creators".to_string(),
                "Always over-deliver on sponsor contracts".to_string(),
                "Build your email list - it's the only platform you own".to_string(),
            ],
        }
    }
    
    /// E-Learning & Online Courses Preset
    fn elearning_preset() -> IndustryPreset {
        IndustryPreset {
            id: "elearning".to_string(),
            name: "E-Learning & Online Courses".to_string(),
            emoji: "🎓".to_string(),
            description: "Create, market, and sell educational content and courses".to_string(),
            best_for: vec![
                "Course creators".to_string(),
                "Coaches and mentors".to_string(),
                "Training program builders".to_string(),
                "Membership site owners".to_string(),
            ],
            stages: vec![
                PresetStage {
                    name: "Curriculum".to_string(),
                    emoji: "📚".to_string(),
                    description: "Design course structure".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration, ModuleType::Notes],
                    actions: vec![
                        "Outline learning objectives".to_string(),
                        "Design module structure".to_string(),
                        "Plan assessments".to_string(),
                    ],
                    output_description: "Complete course curriculum".to_string(),
                },
                PresetStage {
                    name: "Content".to_string(),
                    emoji: "🎬".to_string(),
                    description: "Create lessons".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration, ModuleType::Pipeline],
                    actions: vec![
                        "Script video lessons".to_string(),
                        "Create workbooks".to_string(),
                        "Design quizzes".to_string(),
                    ],
                    output_description: "Lesson scripts, student materials".to_string(),
                },
                PresetStage {
                    name: "Legal".to_string(),
                    emoji: "⚖️".to_string(),
                    description: "Terms & enrollment".to_string(),
                    recommended_modules: vec![ModuleType::ContractGenerator],
                    actions: vec![
                        "Create terms of service".to_string(),
                        "Draft enrollment agreements".to_string(),
                        "Set refund policies".to_string(),
                    ],
                    output_description: "Legal docs, enrollment contracts".to_string(),
                },
                PresetStage {
                    name: "Bundle".to_string(),
                    emoji: "📦".to_string(),
                    description: "Course packages".to_string(),
                    recommended_modules: vec![ModuleType::BundleBuilder],
                    actions: vec![
                        "Create tiered packages".to_string(),
                        "Build bonus libraries".to_string(),
                        "Package workbooks".to_string(),
                    ],
                    output_description: "Course tiers, bonus bundles".to_string(),
                },
                PresetStage {
                    name: "Schedule".to_string(),
                    emoji: "📅".to_string(),
                    description: "Lesson releases".to_string(),
                    recommended_modules: vec![ModuleType::Scheduler],
                    actions: vec![
                        "Set drip schedule".to_string(),
                        "Plan live Q&A sessions".to_string(),
                        "Coordinate cohort starts".to_string(),
                    ],
                    output_description: "Release schedule, cohort calendar".to_string(),
                },
                PresetStage {
                    name: "Export".to_string(),
                    emoji: "📤".to_string(),
                    description: "Student materials".to_string(),
                    recommended_modules: vec![ModuleType::ExportPdf, ModuleType::ExportDocx],
                    actions: vec![
                        "Export workbooks".to_string(),
                        "Create PDF guides".to_string(),
                        "Package slide decks".to_string(),
                    ],
                    output_description: "Student downloads, course materials".to_string(),
                },
            ],
            quick_tips: vec![
                "Workbooks increase completion rates by 40%".to_string(),
                "Drip content prevents overwhelm and reduces refunds".to_string(),
                "Always include action steps, not just theory".to_string(),
                "Cohort-based courses command 3-5x higher prices".to_string(),
            ],
        }
    }
    
    /// SaaS & Software Products Preset
    fn saas_preset() -> IndustryPreset {
        IndustryPreset {
            id: "saas".to_string(),
            name: "SaaS & Software Products".to_string(),
            emoji: "💻".to_string(),
            description: "Build and launch software products with proper legal protection".to_string(),
            best_for: vec![
                "SaaS founders".to_string(),
                "App developers".to_string(),
                "API providers".to_string(),
                "Dev tool creators".to_string(),
            ],
            stages: vec![
                PresetStage {
                    name: "Roadmap".to_string(),
                    emoji: "🗺️".to_string(),
                    description: "Feature planning".to_string(),
                    recommended_modules: vec![ModuleType::MarketResearch, ModuleType::Notes],
                    actions: vec![
                        "Research competitor features".to_string(),
                        "Prioritize MVP scope".to_string(),
                        "Plan release phases".to_string(),
                    ],
                    output_description: "Product roadmap, feature backlog".to_string(),
                },
                PresetStage {
                    name: "Development".to_string(),
                    emoji: "⚙️".to_string(),
                    description: "Build the product".to_string(),
                    recommended_modules: vec![ModuleType::Pipeline, ModuleType::Notes],
                    actions: vec![
                        "Track sprint progress".to_string(),
                        "Document features".to_string(),
                        "Manage bug fixes".to_string(),
                    ],
                    output_description: "Shipped features, release notes".to_string(),
                },
                PresetStage {
                    name: "Legal".to_string(),
                    emoji: "⚖️".to_string(),
                    description: "Terms & privacy".to_string(),
                    recommended_modules: vec![ModuleType::ContractGenerator],
                    actions: vec![
                        "Draft Terms of Service".to_string(),
                        "Create Privacy Policy".to_string(),
                        "Set EULA".to_string(),
                    ],
                    output_description: "Legal docs, user agreements".to_string(),
                },
                PresetStage {
                    name: "Beta".to_string(),
                    emoji: "🧪".to_string(),
                    description: "Beta program".to_string(),
                    recommended_modules: vec![ModuleType::Scheduler, ModuleType::ContractGenerator],
                    actions: vec![
                        "Schedule beta invites".to_string(),
                        "Create beta agreements".to_string(),
                        "Plan feedback collection".to_string(),
                    ],
                    output_description: "Beta cohort, feedback reports".to_string(),
                },
                PresetStage {
                    name: "Launch".to_string(),
                    emoji: "🚀".to_string(),
                    description: "Go to market".to_string(),
                    recommended_modules: vec![ModuleType::Scheduler, ModuleType::BundleBuilder],
                    actions: vec![
                        "Schedule launch sequence".to_string(),
                        "Create press kit".to_string(),
                        "Prepare demo assets".to_string(),
                    ],
                    output_description: "Launch campaign, media coverage".to_string(),
                },
                PresetStage {
                    name: "Export".to_string(),
                    emoji: "📤".to_string(),
                    description: "Documentation".to_string(),
                    recommended_modules: vec![ModuleType::ExportPdf, ModuleType::ExportDocx],
                    actions: vec![
                        "Export API docs".to_string(),
                        "Create user guides".to_string(),
                        "Package onboarding materials".to_string(),
                    ],
                    output_description: "Documentation, help center".to_string(),
                },
            ],
            quick_tips: vec![
                "Start with annual billing to improve cash flow".to_string(),
                "Your first 10 customers teach you more than any research".to_string(),
                "Documentation is a feature - invest in it early".to_string(),
                "Beta agreements protect you from liability during testing".to_string(),
            ],
        }
    }
    
    /// Digital Marketing Agency Preset
    fn marketing_agency_preset() -> IndustryPreset {
        IndustryPreset {
            id: "marketing_agency".to_string(),
            name: "Digital Marketing Agency".to_string(),
            emoji: "📢".to_string(),
            description: "Manage client campaigns from pitch to performance reporting".to_string(),
            best_for: vec![
                "Marketing agencies".to_string(),
                "PPC managers".to_string(),
                "SEO consultants".to_string(),
                "Social media managers".to_string(),
            ],
            stages: vec![
                PresetStage {
                    name: "Prospect".to_string(),
                    emoji: "🎯".to_string(),
                    description: "New business".to_string(),
                    recommended_modules: vec![ModuleType::MarketResearch, ModuleType::Notes],
                    actions: vec![
                        "Research prospect's market".to_string(),
                        "Analyze competitors".to_string(),
                        "Prepare pitch deck".to_string(),
                    ],
                    output_description: "Pitch proposal, competitive analysis".to_string(),
                },
                PresetStage {
                    name: "Onboard".to_string(),
                    emoji: "📋".to_string(),
                    description: "Client setup".to_string(),
                    recommended_modules: vec![ModuleType::ContractGenerator],
                    actions: vec![
                        "Sign service agreement".to_string(),
                        "Set SOW terms".to_string(),
                        "Define KPIs".to_string(),
                    ],
                    output_description: "Signed contract, project scope".to_string(),
                },
                PresetStage {
                    name: "Strategy".to_string(),
                    emoji: "🧠".to_string(),
                    description: "Campaign planning".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration, ModuleType::Notes],
                    actions: vec![
                        "Create campaign concepts".to_string(),
                        "Draft ad copy variations".to_string(),
                        "Plan content calendar".to_string(),
                    ],
                    output_description: "Campaign strategy, creative concepts".to_string(),
                },
                PresetStage {
                    name: "Execute".to_string(),
                    emoji: "⚡".to_string(),
                    description: "Campaign launch".to_string(),
                    recommended_modules: vec![ModuleType::Pipeline, ModuleType::Scheduler],
                    actions: vec![
                        "Build campaign assets".to_string(),
                        "Schedule campaign launch".to_string(),
                        "Set up tracking".to_string(),
                    ],
                    output_description: "Live campaign, tracking dashboard".to_string(),
                },
                PresetStage {
                    name: "Optimize".to_string(),
                    emoji: "🔧".to_string(),
                    description: "Performance tuning".to_string(),
                    recommended_modules: vec![ModuleType::Notes, ModuleType::AiGeneration],
                    actions: vec![
                        "Analyze performance data".to_string(),
                        "Generate optimization ideas".to_string(),
                        "A/B test variations".to_string(),
                    ],
                    output_description: "Optimized campaigns, test results".to_string(),
                },
                PresetStage {
                    name: "Report".to_string(),
                    emoji: "📊".to_string(),
                    description: "Client reporting".to_string(),
                    recommended_modules: vec![ModuleType::ExportXlsx, ModuleType::ExportPdf],
                    actions: vec![
                        "Generate performance reports".to_string(),
                        "Export analytics summaries".to_string(),
                        "Create presentation decks".to_string(),
                    ],
                    output_description: "Client reports, performance decks".to_string(),
                },
            ],
            quick_tips: vec![
                "Monthly retainers beat project work for cash flow".to_string(),
                "Under-promise and over-deliver on KPIs".to_string(),
                "Automated reporting saves 5+ hours per client monthly".to_string(),
                "Always get 3-month minimum commitments".to_string(),
            ],
        }
    }
    
    /// Freelancer / Consultant Preset
    fn freelancer_preset() -> IndustryPreset {
        IndustryPreset {
            id: "freelancer".to_string(),
            name: "Freelancer / Consultant".to_string(),
            emoji: "👔".to_string(),
            description: "Manage client projects from proposal to final delivery".to_string(),
            best_for: vec![
                "Freelance developers".to_string(),
                "Designers".to_string(),
                "Writers".to_string(),
                "Business consultants".to_string(),
            ],
            stages: vec![
                PresetStage {
                    name: "Lead".to_string(),
                    emoji: "👋".to_string(),
                    description: "New inquiries".to_string(),
                    recommended_modules: vec![ModuleType::Notes, ModuleType::AiGeneration],
                    actions: vec![
                        "Qualify prospects".to_string(),
                        "Draft proposals".to_string(),
                        "Estimate scope".to_string(),
                    ],
                    output_description: "Qualified leads, draft proposals".to_string(),
                },
                PresetStage {
                    name: "Proposal".to_string(),
                    emoji: "📄".to_string(),
                    description: "Send quotes".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration, ModuleType::ExportPdf],
                    actions: vec![
                        "Write custom proposals".to_string(),
                        "Set pricing".to_string(),
                        "Define deliverables".to_string(),
                    ],
                    output_description: "Client proposals, SOW documents".to_string(),
                },
                PresetStage {
                    name: "Contract".to_string(),
                    emoji: "✍️".to_string(),
                    description: "Get agreement".to_string(),
                    recommended_modules: vec![ModuleType::ContractGenerator],
                    actions: vec![
                        "Generate service agreement".to_string(),
                        "Set payment terms".to_string(),
                        "Define revision limits".to_string(),
                    ],
                    output_description: "Signed contract, payment schedule".to_string(),
                },
                PresetStage {
                    name: "Work".to_string(),
                    emoji: "🔨".to_string(),
                    description: "Do the work".to_string(),
                    recommended_modules: vec![ModuleType::Pipeline, ModuleType::Scheduler],
                    actions: vec![
                        "Track project milestones".to_string(),
                        "Schedule check-ins".to_string(),
                        "Document progress".to_string(),
                    ],
                    output_description: "Completed deliverables, progress reports".to_string(),
                },
                PresetStage {
                    name: "Review".to_string(),
                    emoji: "👀".to_string(),
                    description: "Client feedback".to_string(),
                    recommended_modules: vec![ModuleType::Notes],
                    actions: vec![
                        "Submit for review".to_string(),
                        "Collect feedback".to_string(),
                        "Track revisions".to_string(),
                    ],
                    output_description: "Approved deliverables, feedback log".to_string(),
                },
                PresetStage {
                    name: "Deliver".to_string(),
                    emoji: "📤".to_string(),
                    description: "Final handoff".to_string(),
                    recommended_modules: vec![ModuleType::ExportPdf, ModuleType::ExportDocx, ModuleType::ExportZip],
                    actions: vec![
                        "Package final files".to_string(),
                        "Export deliverables".to_string(),
                        "Create handoff documentation".to_string(),
                    ],
                    output_description: "Final deliverables, project archive".to_string(),
                },
            ],
            quick_tips: vec![
                "50% upfront payment eliminates most bad clients".to_string(),
                "Scope creep is real - document everything".to_string(),
                "Revision limits protect your time and sanity".to_string(),
                "Always get testimonials from happy clients".to_string(),
            ],
        }
    }
    
    /// Info Products & Templates Preset
    fn info_products_preset() -> IndustryPreset {
        IndustryPreset {
            id: "info_products".to_string(),
            name: "Info Products & Templates".to_string(),
            emoji: "📦".to_string(),
            description: "Create and sell downloadable digital products like ebooks, templates, and guides".to_string(),
            best_for: vec![
                "Template creators".to_string(),
                "Ebook authors".to_string(),
                "Notion template sellers".to_string(),
                "Digital download shops".to_string(),
            ],
            stages: vec![
                PresetStage {
                    name: "Research".to_string(),
                    emoji: "🔍".to_string(),
                    description: "Find demand".to_string(),
                    recommended_modules: vec![ModuleType::MarketResearch, ModuleType::Notes],
                    actions: vec![
                        "Identify trending topics".to_string(),
                        "Analyze competitor products".to_string(),
                        "Validate pricing".to_string(),
                    ],
                    output_description: "Market validation, competitive analysis".to_string(),
                },
                PresetStage {
                    name: "Outline".to_string(),
                    emoji: "📝".to_string(),
                    description: "Structure content".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration, ModuleType::Notes],
                    actions: vec![
                        "Create table of contents".to_string(),
                        "Design template structure".to_string(),
                        "Plan sections/modules".to_string(),
                    ],
                    output_description: "Product outline, structure plan".to_string(),
                },
                PresetStage {
                    name: "Create".to_string(),
                    emoji: "✍️".to_string(),
                    description: "Build the product".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration, ModuleType::Pipeline],
                    actions: vec![
                        "Write content".to_string(),
                        "Design templates".to_string(),
                        "Create examples".to_string(),
                    ],
                    output_description: "Completed product, template files".to_string(),
                },
                PresetStage {
                    name: "Design".to_string(),
                    emoji: "🎨".to_string(),
                    description: "Make it beautiful".to_string(),
                    recommended_modules: vec![ModuleType::Notes],
                    actions: vec![
                        "Create cover design".to_string(),
                        "Format layouts".to_string(),
                        "Add branding".to_string(),
                    ],
                    output_description: "Polished design, branded assets".to_string(),
                },
                PresetStage {
                    name: "Legal".to_string(),
                    emoji: "⚖️".to_string(),
                    description: "Protect your work".to_string(),
                    recommended_modules: vec![ModuleType::ContractGenerator],
                    actions: vec![
                        "Create license terms".to_string(),
                        "Set usage rights".to_string(),
                        "Add disclaimers".to_string(),
                    ],
                    output_description: "License agreement, terms of use".to_string(),
                },
                PresetStage {
                    name: "Bundle".to_string(),
                    emoji: "🎁".to_string(),
                    description: "Package extras".to_string(),
                    recommended_modules: vec![ModuleType::BundleBuilder],
                    actions: vec![
                        "Create bonus materials".to_string(),
                        "Package quick-start guides".to_string(),
                        "Bundle related templates".to_string(),
                    ],
                    output_description: "Value bundle, bonus package".to_string(),
                },
                PresetStage {
                    name: "Export".to_string(),
                    emoji: "📤".to_string(),
                    description: "Prepare files".to_string(),
                    recommended_modules: vec![ModuleType::ExportPdf, ModuleType::ExportDocx, ModuleType::ExportXlsx, ModuleType::ExportZip],
                    actions: vec![
                        "Export in multiple formats".to_string(),
                        "Create preview samples".to_string(),
                        "Package for delivery".to_string(),
                    ],
                    output_description: "Ready-to-sell files, preview assets".to_string(),
                },
                PresetStage {
                    name: "Launch".to_string(),
                    emoji: "🚀".to_string(),
                    description: "Go live".to_string(),
                    recommended_modules: vec![ModuleType::Scheduler],
                    actions: vec![
                        "Schedule product launch".to_string(),
                        "Plan promotional sequence".to_string(),
                        "Set up sales page".to_string(),
                    ],
                    output_description: "Live product, sales campaign".to_string(),
                },
            ],
            quick_tips: vec![
                "Multiple formats increase perceived value".to_string(),
                "Preview samples drive 3x more conversions".to_string(),
                "Bundle related products for higher average order value".to_string(),
                "Update products annually to stay relevant".to_string(),
            ],
        }
    }
    
    /// Complete Business Setup Preset (12-Step Workflow)
    fn business_setup_preset() -> IndustryPreset {
        IndustryPreset {
            id: "business_setup".to_string(),
            name: "Complete Business Setup".to_string(),
            emoji: "🏪".to_string(),
            description: "12-step workflow from research to launch. Based on proven Etsy/POD success patterns.".to_string(),
            best_for: vec![
                "New Etsy sellers".to_string(),
                "POD entrepreneurs".to_string(),
                "Digital product beginners".to_string(),
                "Side hustle starters".to_string(),
            ],
            stages: vec![
                PresetStage {
                    name: "Research".to_string(),
                    emoji: "🔍".to_string(),
                    description: "Research products and competitors".to_string(),
                    recommended_modules: vec![ModuleType::MarketResearch, ModuleType::Notes],
                    actions: vec![
                        "Analyze market trends".to_string(),
                        "Study competitor offerings".to_string(),
                        "Validate demand for product ideas".to_string(),
                        "Document findings".to_string(),
                    ],
                    output_description: "Market validation, competitor analysis, product ideas".to_string(),
                },
                PresetStage {
                    name: "Branding".to_string(),
                    emoji: "🎨".to_string(),
                    description: "Shop name and logo".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration],
                    actions: vec![
                        "Brainstorm shop names with AI".to_string(),
                        "Generate logo concepts".to_string(),
                        "Create brand identity assets".to_string(),
                        "Check name availability".to_string(),
                    ],
                    output_description: "Shop name, logo files, brand identity".to_string(),
                },
                PresetStage {
                    name: "Accounts".to_string(),
                    emoji: "📧".to_string(),
                    description: "Create platform accounts".to_string(),
                    recommended_modules: vec![ModuleType::Notes],
                    actions: vec![
                        "Create dedicated business email".to_string(),
                        "Register marketplace accounts".to_string(),
                        "Set up payment processors".to_string(),
                        "Document login credentials".to_string(),
                    ],
                    output_description: "Active seller accounts on all platforms".to_string(),
                },
                PresetStage {
                    name: "Storefront".to_string(),
                    emoji: "🏪".to_string(),
                    description: "Build shop presence".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration, ModuleType::BundleBuilder],
                    actions: vec![
                        "Create banner images".to_string(),
                        "Write shop description".to_string(),
                        "Set up shop policies".to_string(),
                        "Optimize for search".to_string(),
                    ],
                    output_description: "Complete storefront with SEO optimization".to_string(),
                },
                PresetStage {
                    name: "Product Focus".to_string(),
                    emoji: "🎯".to_string(),
                    description: "Focus on one product/design".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration, ModuleType::Pipeline],
                    actions: vec![
                        "Choose your first product type".to_string(),
                        "Create initial designs with AI".to_string(),
                        "Perfect one before expanding".to_string(),
                    ],
                    output_description: "First product line ready for listing".to_string(),
                },
                PresetStage {
                    name: "Marketing Assets".to_string(),
                    emoji: "📢".to_string(),
                    description: "Banners and social posts".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration, ModuleType::ExportPdf],
                    actions: vec![
                        "Create incentive banners".to_string(),
                        "Design social media templates".to_string(),
                        "Make promotional graphics".to_string(),
                    ],
                    output_description: "Marketing asset library".to_string(),
                },
                PresetStage {
                    name: "Automation".to_string(),
                    emoji: "⚙️".to_string(),
                    description: "Set up fulfillment automation".to_string(),
                    recommended_modules: vec![ModuleType::Scheduler, ModuleType::Notes],
                    actions: vec![
                        "Configure auto-delivery for digital products".to_string(),
                        "Set up POD fulfillment connections".to_string(),
                        "Automate order processing".to_string(),
                        "Test automation workflows".to_string(),
                    ],
                    output_description: "Automated fulfillment pipeline".to_string(),
                },
                PresetStage {
                    name: "Social Media".to_string(),
                    emoji: "📱".to_string(),
                    description: "Create social presence".to_string(),
                    recommended_modules: vec![ModuleType::Scheduler, ModuleType::AiGeneration],
                    actions: vec![
                        "Create social media accounts".to_string(),
                        "Generate content with AI".to_string(),
                        "Schedule posts".to_string(),
                        "Plan promotional campaigns".to_string(),
                    ],
                    output_description: "Active social media presence".to_string(),
                },
                PresetStage {
                    name: "Email System".to_string(),
                    emoji: "📧".to_string(),
                    description: "Build email list".to_string(),
                    recommended_modules: vec![ModuleType::ContractGenerator, ModuleType::Notes],
                    actions: vec![
                        "Set up email marketing platform".to_string(),
                        "Create lead magnets".to_string(),
                        "Build signup forms".to_string(),
                        "Draft welcome sequences".to_string(),
                    ],
                    output_description: "Email marketing system ready".to_string(),
                },
                PresetStage {
                    name: "Partnerships".to_string(),
                    emoji: "🤝".to_string(),
                    description: "Marketing and distribution".to_string(),
                    recommended_modules: vec![ModuleType::ContractGenerator],
                    actions: vec![
                        "Draft marketing contracts".to_string(),
                        "Create distribution agreements".to_string(),
                        "Set up affiliate programs".to_string(),
                    ],
                    output_description: "Signed partnership contracts".to_string(),
                },
                PresetStage {
                    name: "Website".to_string(),
                    emoji: "🌐".to_string(),
                    description: "Create own website".to_string(),
                    recommended_modules: vec![ModuleType::AiGeneration, ModuleType::ExportPdf],
                    actions: vec![
                        "Build website with AI assistance".to_string(),
                        "Add product listings".to_string(),
                        "Set up payment processing".to_string(),
                        "Connect custom domain".to_string(),
                    ],
                    output_description: "Live website with store".to_string(),
                },
                PresetStage {
                    name: "Testing".to_string(),
                    emoji: "🧪".to_string(),
                    description: "User testing and feedback".to_string(),
                    recommended_modules: vec![ModuleType::Notes],
                    actions: vec![
                        "Recruit beta testers".to_string(),
                        "Collect feedback".to_string(),
                        "Document issues".to_string(),
                        "Iterate based on results".to_string(),
                    ],
                    output_description: "Tested and optimized store".to_string(),
                },
            ],
            quick_tips: vec![
                "Thomas Frank made $1M in 2 years selling Notion templates".to_string(),
                "Good marketplace shops have handcrafted-looking images".to_string(),
                "Some shoppers care less about presentation - test multiple platforms".to_string(),
                "Focus on ONE product before expanding".to_string(),
                "SEO optimization is critical for discovery".to_string(),
            ],
        }
    }
    
    // Public API
    
    pub fn get(&self, id: &str) -> Option<&IndustryPreset> {
        self.presets.get(id)
    }
    
    pub fn list(&self) -> Vec<&IndustryPreset> {
        self.presets.values().collect()
    }
    
    pub fn list_ids(&self) -> Vec<&String> {
        self.presets.keys().collect()
    }
    
    pub fn by_category(&self, category: &str) -> Vec<&IndustryPreset> {
        // For now, simple filtering by checking if category appears in best_for
        self.presets.values()
            .filter(|p| p.best_for.iter().any(|b| b.to_lowercase().contains(&category.to_lowercase())))
            .collect()
    }
}