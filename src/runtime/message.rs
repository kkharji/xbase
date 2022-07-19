use crate::server::{BuildRequest, RunRequest};
use crate::{Event, PathExt};
use std::{collections::HashSet, path::PathBuf};
use tokio::sync::mpsc;

/// Project Runime Message
#[derive(Debug)]
pub enum PRMessage {
    /// Process filesystem changes
    FSEvent(Event),
    /// Connect to client
    Connect(u32),
    /// Disconnect from client
    Disconnect(u32),
    /// Process Run Request
    Run(RunRequest),
    /// Process Build Request
    Build(BuildRequest),
}

#[derive(Debug)]
pub struct PRMessageSender {
    /// Project Root
    root: PathBuf,
    /// Message Broadcaster address
    broadcaster_adderss: PathBuf,
    /// PRMessage sender
    sender: mpsc::UnboundedSender<PRMessage>,
    /// Connect Cilents
    clients: HashSet<u32>,
}

impl PRMessageSender {
    pub fn new(
        root: &PathBuf,
        baddress: &PathBuf,
        sender: &mpsc::UnboundedSender<PRMessage>,
    ) -> Self {
        Self {
            root: root.clone(),
            broadcaster_adderss: baddress.clone(),
            sender: sender.clone(),
            clients: HashSet::default(),
        }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn name(&self) -> String {
        self.root().name().unwrap()
    }

    pub fn connect(&mut self, id: u32) {
        if !self.clients.contains(&id) {
            self.send(PRMessage::Connect(id));
            self.clients.insert(id);
        }
    }

    pub fn disconnect(&mut self, id: u32) {
        if self.clients.contains(&id) {
            self.clients.remove(&id);
            self.send(PRMessage::Disconnect(id));
        }
    }

    pub fn send(&self, message: PRMessage) {
        if let Err(e) = self.sender.send(message) {
            tracing::error!("Failed to send {e:#?}");
        };
    }

    pub fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }

    pub fn broadcaster_adderss(&self) -> &PathBuf {
        &self.broadcaster_adderss
    }

    pub fn contains(&self, value: &u32) -> bool {
        self.clients.contains(value)
    }

    pub fn insert(&mut self, value: u32) -> bool {
        self.clients.insert(value)
    }
}
