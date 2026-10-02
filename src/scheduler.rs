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
    
    /// Turn scheduled execution on.
    ///
    /// The real work is driven by `DpfApp::tick_scheduler` from the UI loop, NOT by a
    /// background thread — execution needs the generator, publisher and pipeline, which live on
    /// the app, not here.
    ///
    /// This method used to spawn a thread that slept 60 seconds in a loop and did nothing: its
    /// body literally read "In production, this would check and execute due tasks. For now, this
    /// is a placeholder for the scheduling loop." Combined with `run_due_tasks()` having zero
    /// call sites and `execute_task()` printing a line and returning Ok, every scheduled task
    /// reported success while nothing whatsoever happened.
    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Tasks that are enabled and whose time has arrived.
    ///
    /// Read-only: it hands the caller a copy and lets the caller decide what to do, because this
    /// module has no access to the modules that actually do the work.
    pub fn due_tasks(&self) -> Vec<ScheduledTask> {
        let now = Utc::now();
        self.tasks
            .iter()
            .filter(|t| {
                t.enabled
                    && !matches!(t.status, TaskStatus::Paused | TaskStatus::Running)
                    && t.next_run.map(|nr| nr <= now).unwrap_or(false)
            })
            .cloned()
            .collect()
    }

    /// Record the outcome of a run: status, last_run, next occurrence, then persist.
    pub fn record_run(&mut self, id: usize, result: Result<(), String>) {
        let now = Utc::now();
        if let Some(index) = self.tasks.iter().position(|t| t.id == id) {
            let mut task = self.tasks[index].clone();
            task.last_run = Some(now);
            task.status = match result {
                Ok(()) => TaskStatus::Completed,
                Err(e) => TaskStatus::Failed(e),
            };
            task = self.calculate_next_run(task);
            self.tasks[index] = task;
            self.save_tasks();
        }
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
}
