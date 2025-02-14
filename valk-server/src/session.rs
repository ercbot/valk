use std::sync::RwLock;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
    expires_at: Instant,
}

impl Session {
    pub fn new(duration: Duration) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            expires_at: Instant::now() + duration,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }

    pub fn extend_expiration(&mut self, duration: Duration) {
        self.expires_at = Instant::now() + duration;
    }
}

#[derive(Debug)]
pub struct SessionManager {
    active_session: RwLock<Option<Session>>,
    session_duration: Duration,
}

impl SessionManager {
    pub fn new(session_duration: Duration) -> Self {
        Self {
            active_session: RwLock::new(None),
            session_duration,
        }
    }

    pub fn create_session(&self) -> Session {
        let session = Session::new(self.session_duration);
        *self.active_session.write().unwrap() = Some(session.clone());
        session
    }

    pub fn validate_session_id(&self, session_id: &str) -> bool {
        let mut session_guard = self.active_session.write().unwrap();

        if let Some(session) = session_guard.as_mut() {
            if session.id == session_id {
                if session.is_expired() {
                    *session_guard = None;
                    false
                } else {
                    // Extend session duration on successful validation
                    session.extend_expiration(self.session_duration);
                    true
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn clear_session(&self) {
        *self.active_session.write().unwrap() = None;
    }
}
