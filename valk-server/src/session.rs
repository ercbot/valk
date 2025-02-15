use std::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Session {
    pub id: String,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug)]
pub struct SessionManager {
    active_session: RwLock<Option<Session>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            active_session: RwLock::new(None),
        }
    }

    pub fn create_session(&self) -> Result<Session, &'static str> {
        let mut session_guard = self.active_session.write().unwrap();
        if session_guard.is_some() {
            return Err("A session is already active");
        }
        let session = Session::new();
        *session_guard = Some(session.clone());
        Ok(session)
    }

    pub fn validate_session_id(&self, session_id: &str) -> bool {
        self.active_session
            .read()
            .unwrap()
            .as_ref()
            .map_or(false, |session| session.id == session_id)
    }

    pub fn clear_session(&self) {
        *self.active_session.write().unwrap() = None;
    }
}
