//! Task scheduler for automation - runs tasks at scheduled times

use crate::database::Database;
use chrono::{DateTime, Duration, Utc, Datelike, Timelike};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::runtime::Runtime;
use std::thread;
use std::time::Duration as StdDuration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: usize,
    pub name: String,
    pub task_type: TaskType,
    pub schedule: Schedule,
    pub next_run: Option<DateTime<Utc>>,
    pub last_run: Option<DateTime<Utc>>,
    pub status: TaskStatus,
    pub data: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    GenerateProduct { template_id: String, params: serde_json::Value },
    PublishProduct { product_id: usize, platforms: Vec<String> },
    ResearchMarket { query: String },
    CreateBundle { product_ids: Vec<usize>, name: String },
    PinterestPin { product_id: usize, board: String },
    BackupData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Schedule {
    Once(DateTime<Utc>),
    Daily { hour: u32, minute: u32 },
    Weekly { day: u32, hour: u32, minute: u32 },
    Interval { minutes: u64 },
    Smart, // Business hours only, optimal timing
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Paused,
}

pub struct Scheduler {
    db: Arc<Database>,
    runtime: Arc<Runtime>,
    tasks: Vec<ScheduledTask>,
    running: bool,
}

impl Scheduler {
    pub fn new(db: &Arc<Database>, runtime: Arc<Runtime>) -> Self {
        let tasks = db.load_scheduled_tasks().unwrap_or_default();
        
        Self {
            db: db.clone(),
            runtime,
            tasks,
            running: false,
        }
    }
    
    pub fn add_task(&mut self, task: ScheduledTask) {
        // Calculate next run time
        let task = self.calculate_next_run(task);
        self.tasks.push(task);
        self.save_tasks();
    }
    
    pub fn remove_task(&mut self, id: usize) {
        self.tasks.retain(|t| t.id != id);
        self.save_tasks();
    }
    
    pub fn toggle_task(&mut self, id: usize) {
        let idx = self.tasks.iter().position(|t| t.id == id);
        if let Some(idx) = idx {
            self.tasks[idx].enabled = !self.tasks[idx].enabled;
            if self.tasks[idx].enabled {
                let task = self.tasks[idx].clone();
                self.tasks[idx] = self.calculate_next_run(task);
            }
            self.save_tasks();
        }
    }
    
    pub fn tasks(&self) -> &[ScheduledTask] {
        &self.tasks
    }
    
    pub fn start(&mut self) {
        self.running = true;
        
        // Spawn background thread for scheduling
        let db = self.db.clone();
        let runtime = self.runtime.clone();
        
        thread::spawn(move || {
            loop {
                // Check for due tasks every minute
                thread::sleep(StdDuration::from_secs(60));
                
                // In production, this would check and execute due tasks
                // For now, this is a placeholder for the scheduling loop
            }
        });
    }
    
    pub fn stop(&mut self) {
        self.running = false;
    }
    
    fn calculate_next_run(&self, mut task: ScheduledTask) -> ScheduledTask {
        let now = Utc::now();
        
        task.next_run = Some(match &task.schedule {
            Schedule::Once(datetime) => *datetime,
            Schedule::Daily { hour, minute } => {
                let next = now.date_naive()
                    .and_hms_opt(*hour, *minute, 0)
                    .unwrap()
                    .and_local_timezone(Utc)
                    .unwrap();
                
                if next <= now {
                    next + Duration::days(1)
                } else {
                    next
                }
            },
            Schedule::Weekly { day, hour, minute } => {
                // Calculate next occurrence of this day
                let current_day = now.weekday().num_days_from_sunday();
                let target_day = *day % 7;
                let days_ahead = (target_day + 7 - current_day) % 7;
                
                let next = (now + Duration::days(days_ahead as i64))
                    .date_naive()
                    .and_hms_opt(*hour, *minute, 0)
                    .unwrap()
                    .and_local_timezone(Utc)
                    .unwrap();
                
                if next <= now {
                    next + Duration::weeks(1)
                } else {
                    next
                }
            },
            Schedule::Interval { minutes } => {
                now + Duration::minutes(*minutes as i64)
            },
            Schedule::Smart => {
                // Next business hours slot (8-11pm or 2-4pm optimal for Pinterest)
                let hour = now.hour();
                
                if hour < 14 {
                    // Schedule for 2 PM today
                    now.date_naive().and_hms_opt(14, 0, 0).unwrap().and_local_timezone(Utc).unwrap()
                } else if hour < 20 {
                    // Schedule for 8 PM today
                    now.date_naive().and_hms_opt(20, 0, 0).unwrap().and_local_timezone(Utc).unwrap()
                } else {
                    // Schedule for 2 PM tomorrow
                    (now + Duration::days(1)).date_naive().and_hms_opt(14, 0, 0).unwrap().and_local_timezone(Utc).unwrap()
                }
            },
        });
        
        task
    }
    
    fn save_tasks(&self) {
        // Save to database
        for task in &self.tasks {
            self.db.save_scheduled_task(task).ok();
        }
    }
    
    pub fn run_due_tasks(&mut self) {
        let now = Utc::now();
        let due_tasks: Vec<_> = self.tasks.iter_mut()
            .filter(|t| {
                t.enabled &&
                matches!(t.status, TaskStatus::Pending | TaskStatus::Completed | TaskStatus::Failed(_)) &&
                t.next_run.map(|nr| nr <= now).unwrap_or(false)
            })
            .map(|t| t.clone())
            .collect();
        
        for mut task in due_tasks {
            task.status = TaskStatus::Running;
            task.last_run = Some(now);
            
            // Execute task
            match self.execute_task(&task) {
                Ok(_) => {
                    task.status = TaskStatus::Completed;
                },
                Err(e) => {
                    task.status = TaskStatus::Failed(e);
                },
            }
            
            // Reschedule if recurring
            task = self.calculate_next_run(task);
            
            // Update in list
            if let Some(idx) = self.tasks.iter().position(|t| t.id == task.id) {
                self.tasks[idx] = task;
            }
        }
        
        self.save_tasks();
    }
    
    fn execute_task(&self, task: &ScheduledTask) -> Result<(), String> {
        match &task.task_type {
            TaskType::GenerateProduct { template_id, params } => {
                println!("Generating product from template: {}", template_id);
                // Call product generator
                Ok(())
            },
            TaskType::PublishProduct { product_id, platforms } => {
                println!("Publishing product {} to {:?}", product_id, platforms);
                Ok(())
            },
            TaskType::ResearchMarket { query } => {
                println!("Researching market for: {}", query);
                Ok(())
            },
            TaskType::CreateBundle { product_ids, name } => {
                println!("Creating bundle '{}' with {} products", name, product_ids.len());
                Ok(())
            },
            TaskType::PinterestPin { product_id, board } => {
                println!("Pinning product {} to board {}", product_id, board);
                Ok(())
            },
            TaskType::BackupData => {
                println!("Backing up data");
                Ok(())
            },
        }
    }
}
