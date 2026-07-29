//! Pipeline module - Fast kanban-style product workflow

use crate::database::Database;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum PipelineStage {
    Idea,       // Just an idea
    Research,   // Market research done
    Creating,   // Building the product
    Review,     // QA/review
    Listed,     // Published
    Selling,    // Live & selling
    Archived,   // No longer active
}

impl PipelineStage {
    pub fn name(&self) -> &'static str {
        match self {
            PipelineStage::Idea => "💡 Idea",
            PipelineStage::Research => "🔍 Research",
            PipelineStage::Creating => "🔨 Creating",
            PipelineStage::Review => "👀 Review",
            PipelineStage::Listed => "📋 Listed",
            PipelineStage::Selling => "💰 Selling",
            PipelineStage::Archived => "📦 Archived",
        }
    }
    
    pub fn color(&self) -> egui::Color32 {
        match self {
            PipelineStage::Idea => egui::Color32::from_rgb(100, 149, 237),      // Cornflower blue
            PipelineStage::Research => egui::Color32::from_rgb(255, 165, 0),    // Orange
            PipelineStage::Creating => egui::Color32::from_rgb(50, 205, 50),    // Lime green
            PipelineStage::Review => egui::Color32::from_rgb(255, 215, 0),      // Gold
            PipelineStage::Listed => egui::Color32::from_rgb(138, 43, 226),     // Blue violet
            PipelineStage::Selling => egui::Color32::from_rgb(0, 206, 209),     // Dark turquoise
            PipelineStage::Archived => egui::Color32::from_rgb(128, 128, 128),  // Gray
        }
    }
    
    pub fn all() -> Vec<PipelineStage> {
        vec![
            PipelineStage::Idea,
            PipelineStage::Research,
            PipelineStage::Creating,
            PipelineStage::Review,
            PipelineStage::Listed,
            PipelineStage::Selling,
            PipelineStage::Archived,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductIdea {
    pub id: usize,
    pub title: String,
    pub description: String,
    pub stage: PipelineStage,
    pub product_type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub priority: Priority,
    pub tags: Vec<String>,
    pub estimated_value: f64,
    pub actual_value: Option<f64>,
    pub notes: String,
    pub platform: Vec<String>, // Etsy, Gumroad, etc.
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Priority {
    #[default]
    Low,
    Medium,
    High,
    Urgent,
}

impl Priority {
    pub fn name(&self) -> &'static str {
        match self {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
            Priority::Urgent => "🔥 Urgent",
        }
    }
    
    pub fn color(&self) -> egui::Color32 {
        match self {
            Priority::Low => egui::Color32::GRAY,
            Priority::Medium => egui::Color32::YELLOW,
            Priority::High => egui::Color32::from_rgb(255, 140, 0), // Dark orange
            Priority::Urgent => egui::Color32::RED,
        }
    }
}

pub struct Pipeline {
    pub ideas: Vec<ProductIdea>,
    pub selected_idea: Option<usize>,
    pub filter_stage: Option<PipelineStage>,
    pub filter_priority: Option<Priority>,
    pub search_query: String,
    pub view_mode: ViewMode,
    pub drag_source: Option<usize>,
    pub new_idea_draft: NewIdeaDraft,
    pub show_new_idea_dialog: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Kanban,
    List,
    Calendar,
}

#[derive(Clone, Default)]
pub struct NewIdeaDraft {
    pub title: String,
    pub description: String,
    pub product_type: String,
    pub priority: Priority,
    pub estimated_value: String,
}

impl Pipeline {
    pub fn load(db: &Arc<Database>) -> Self {
        let ideas = db.load_ideas().unwrap_or_default();
        
        Self {
            ideas,
            selected_idea: None,
            filter_stage: None,
            filter_priority: None,
            search_query: String::new(),
            view_mode: ViewMode::Kanban,
            drag_source: None,
            new_idea_draft: NewIdeaDraft::default(),
            show_new_idea_dialog: false,
        }
    }
    
    pub fn add_idea(&mut self, db: &Database, idea: ProductIdea) {
        self.ideas.push(idea.clone());
        db.save_idea(&idea).ok();
    }
    
    pub fn update_idea(&mut self, db: &Database, id: usize, f: impl FnOnce(&mut ProductIdea)) {
        if let Some(idea) = self.ideas.iter_mut().find(|i| i.id == id) {
            f(idea);
            idea.updated_at = Utc::now();
            db.save_idea(idea).ok();
        }
    }
    
    pub fn move_to_stage(&mut self, db: &Database, id: usize, stage: PipelineStage) {
        self.update_idea(db, id, |idea| {
            idea.stage = stage;
        });
    }
    
    pub fn delete_idea(&mut self, db: &Database, id: usize) {
        self.ideas.retain(|i| i.id != id);
        db.delete_idea(id).ok();
    }
    
    pub fn filtered_ideas(&self) -> Vec<&ProductIdea> {
        self.ideas
            .iter()
            .filter(|idea| {
                // Stage filter
                if let Some(stage) = &self.filter_stage {
                    if idea.stage != *stage {
                        return false;
                    }
                }
                
                // Priority filter
                if let Some(priority) = &self.filter_priority {
                    if idea.priority != *priority {
                        return false;
                    }
                }
                
                // Search filter
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    if !idea.title.to_lowercase().contains(&query)
                        && !idea.description.to_lowercase().contains(&query)
                        && !idea.tags.iter().any(|t| t.to_lowercase().contains(&query))
                    {
                        return false;
                    }
                }
                
                true
            })
            .collect()
    }
    
    pub fn ideas_by_stage(&self, stage: PipelineStage) -> Vec<&ProductIdea> {
        self.ideas
            .iter()
            .filter(|i| i.stage == stage)
            .collect()
    }
    
    pub fn stats(&self) -> PipelineStats {
        PipelineStats {
            total: self.ideas.len(),
            by_stage: PipelineStage::all()
                .into_iter()
                .map(|s| (s, self.ideas_by_stage(s).len()))
                .collect(),
            total_value: self.ideas.iter().filter_map(|i| i.actual_value).sum(),
            potential_value: self.ideas.iter().map(|i| i.estimated_value).sum(),
        }
    }
}

#[allow(dead_code)]
pub struct PipelineStats {
    pub total: usize,
    pub by_stage: Vec<(PipelineStage, usize)>,
    pub total_value: f64,
    pub potential_value: f64,
}
