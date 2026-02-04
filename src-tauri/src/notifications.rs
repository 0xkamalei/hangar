use crate::types::Notification;
use anyhow::Result;
use std::sync::Mutex;
use lazy_static::lazy_static;
use chrono::Utc;
use uuid::Uuid;

lazy_static! {
    static ref NOTIFICATIONS: Mutex<Vec<Notification>> = Mutex::new(Vec::new());
}

pub fn add_notification(title: &str, message: &str, severity: &str) {
    let mut notifications = NOTIFICATIONS.lock().unwrap();
    notifications.push(Notification {
        id: Uuid::new_v4().to_string(),
        title: title.to_string(),
        message: message.to_string(),
        timestamp: Utc::now().timestamp(),
        is_read: false,
        severity: severity.to_string(),
    });

    // Keep only last 50 notifications
    if notifications.len() > 50 {
        notifications.remove(0);
    }
}

pub fn get_notifications() -> Vec<Notification> {
    NOTIFICATIONS.lock().unwrap().clone()
}

pub fn mark_as_read(id: &str) {
    let mut notifications = NOTIFICATIONS.lock().unwrap();
    if let Some(n) = notifications.iter_mut().find(|n| n.id == id) {
        n.is_read = true;
    }
}

pub fn clear_notifications() {
    NOTIFICATIONS.lock().unwrap().clear();
}
