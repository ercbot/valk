use std::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Session {
    pub token: String,
}

impl Session {
    pub fn new() -> Self {
        Self {
            token: Uuid::new_v4().to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub struct SessionManager {
    active_session: RwLock<Option<Session>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            active_session: RwLock::new(None),
        }
    }

    pub fn create_session(&self) -> Session {
        let session = Session::new();
        *self.active_session.write().unwrap() = Some(session.clone());
        session
    }

    pub fn validate_token(&self, token: &str) -> bool {
        self.active_session
            .read()
            .unwrap()
            .as_ref()
            .map_or(false, |session| session.token == token)
    }

    pub fn clear_session(&self) {
        *self.active_session.write().unwrap() = None;
    }
}
