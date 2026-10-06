//! Client management — for running work for multiple clients.
//!
//! This implements the `client_management` feature that the **Pro** and **Enterprise** tiers
//! sell. Until now that slug was gated by the licence system while **no module existed behind
//! it** — the tier advertised something that did not exist. (The same was true of `whitelabel`,
//! which has been removed from the product entirely.)

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::database::Database;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientStatus {
    Prospect,
    Active,
    Paused,
    Closed,
}

impl ClientStatus {
    pub fn all() -> [ClientStatus; 4] {
        [
            ClientStatus::Prospect,
            ClientStatus::Active,
            ClientStatus::Paused,
            ClientStatus::Closed,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            ClientStatus::Prospect => "Prospect",
            ClientStatus::Active => "Active",
            ClientStatus::Paused => "Paused",
            ClientStatus::Closed => "Closed",
        }
    }

    pub fn from_label(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "active" => ClientStatus::Active,
            "paused" => ClientStatus::Paused,
            "closed" => ClientStatus::Closed,
            _ => ClientStatus::Prospect,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: usize,
    pub name: String,
    pub email: String,
    pub company: String,
    pub status: ClientStatus,
    pub notes: String,
    pub created_at: DateTime<Utc>,
}

impl Client {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            name: String::new(),
            email: String::new(),
            company: String::new(),
            status: ClientStatus::Prospect,
            notes: String::new(),
            created_at: Utc::now(),
        }
    }

    /// A client is only meaningful with a name; everything else is optional.
    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
    }
}

pub struct ClientManager {
    db: Arc<Database>,
    pub clients: Vec<Client>,
    pub next_id: usize,
}

impl ClientManager {
    pub fn new(db: &Arc<Database>) -> Self {
        let mut manager = Self {
            db: Arc::clone(db),
            clients: Vec::new(),
            next_id: 1,
        };

        match db.load_clients() {
            Ok(list) => {
                manager.next_id = list.iter().map(|c| c.id).max().unwrap_or(0) + 1;
                manager.clients = list;
            }
            // Never fail app start because of client data, but do not swallow it silently.
            Err(e) => eprintln!("[client_manager] could not load clients: {e}"),
        }

        manager
    }

    pub fn add(&mut self, mut client: Client) -> Result<usize, String> {
        if !client.is_valid() {
            return Err("A client needs a name.".to_string());
        }
        client.id = self.next_id;
        self.db
            .save_client(&client)
            .map_err(|e| format!("Could not save the client: {e}"))?;

        let id = client.id;
        self.next_id += 1;
        self.clients.push(client);
        Ok(id)
    }

    pub fn update(&mut self, client: Client) -> Result<(), String> {
        if !client.is_valid() {
            return Err("A client needs a name.".to_string());
        }
        self.db
            .save_client(&client)
            .map_err(|e| format!("Could not update the client: {e}"))?;

        if let Some(slot) = self.clients.iter_mut().find(|c| c.id == client.id) {
            *slot = client;
        }
        Ok(())
    }

    pub fn delete(&mut self, id: usize) -> Result<(), String> {
        self.db
            .delete_client(id)
            .map_err(|e| format!("Could not delete the client: {e}"))?;
        self.clients.retain(|c| c.id != id);
        Ok(())
    }

    pub fn count_by(&self, status: ClientStatus) -> usize {
        self.clients.iter().filter(|c| c.status == status).count()
    }

    /// Filter by name, email or company (case-insensitive). Empty query returns everything.
    pub fn search(&self, query: &str) -> Vec<&Client> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return self.clients.iter().collect();
        }
        self.clients
            .iter()
            .filter(|c| {
                c.name.to_lowercase().contains(&q)
                    || c.email.to_lowercase().contains(&q)
                    || c.company.to_lowercase().contains(&q)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_labels_round_trip() {
        for s in ClientStatus::all() {
            assert_eq!(ClientStatus::from_label(s.label()), s);
        }
        // Unknown text must not silently become a nonsense state.
        assert_eq!(ClientStatus::from_label("banana"), ClientStatus::Prospect);
    }

    #[test]
    fn a_client_needs_a_name() {
        let mut c = Client::new(1);
        assert!(!c.is_valid(), "blank name must be rejected");
        c.name = "   ".to_string();
        assert!(!c.is_valid(), "whitespace-only name must be rejected");
        c.name = "Acme".to_string();
        assert!(c.is_valid());
    }
}
