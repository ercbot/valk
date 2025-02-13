use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the core set of actions that can be performed
/// Each variant defines a specific operation that can be requested
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Action {
    LeftClick,
    RightClick,
    MiddleClick,
    DoubleClick,
    #[serde(rename_all = "snake_case")]
    MouseMove {
        #[serde(flatten)]
        input: MouseMoveInput,
    },
    #[serde(rename_all = "snake_case")]
    LeftClickDrag {
        #[serde(flatten)]
        input: MouseMoveInput,
    },
    #[serde(rename_all = "snake_case")]
    TypeText {
        #[serde(flatten)]
        input: TypeTextInput,
    },
    #[serde(rename_all = "snake_case")]
    KeyPress {
        #[serde(flatten)]
        input: KeyPressInput,
    },
    Screenshot,
    CursorPosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseMoveInput {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeTextInput {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPressInput {
    pub key: String,
}

/// Output data produced by actions that return information
/// Only certain actions (Screenshot, CursorPosition) produce output
#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum ActionOutput {
    Screenshot { image: String },
    CursorPosition { x: u32, y: u32 },
    NoData, // Used for actions that don't produce output
}

/// Represents possible errors that can occur during action execution
#[derive(Debug, Clone)]
pub enum ActionError {
    /// Action took too long to complete
    Timeout,
    /// Action failed during execution
    ExecutionFailed(String),
    /// Invalid input parameters
    InvalidInput(String),
    /// Internal queue communication error
    ChannelError(String),
}

// Custom serialization implementation for ActionError

// Custom serialization implementation for ActionError
impl serde::Serialize for ActionError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ActionError", 2)?;

        // Convert the enum variant to a string for the type field
        let (error_type, message) = match self {
            ActionError::Timeout => ("timeout", "Action timed out".to_string()),
            ActionError::ExecutionFailed(msg) => ("execution_failed", msg.clone()),
            ActionError::InvalidInput(msg) => ("invalid_input", msg.clone()),
            ActionError::ChannelError(msg) => ("channel_error", msg.clone()),
        };

        state.serialize_field("type", error_type)?;
        state.serialize_field("message", &message)?;

        state.end()
    }
}

/// Incoming message requesting an action to be performed
/// Contains a unique ID and the requested action
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ActionRequest {
    pub id: String,
    pub action: Action,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ActionResponseStatus {
    Success,
    Error,
}

/// Outgoing message containing the result of an action
/// Includes request tracking, timing, status, and any output or error information
#[derive(Debug, Serialize, Clone)]
pub struct ActionResponse {
    pub id: Uuid,
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
    pub status: ActionResponseStatus,
    pub action: Action,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<ActionOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ActionError>,
}

impl ActionResponse {
    /// Creates a successful response, optionally including output data
    pub fn success(request_id: String, action: Action, output: ActionOutput) -> Self {
        let data = if let ActionOutput::NoData = output {
            None
        } else {
            Some(output)
        };

        Self {
            id: Uuid::new_v4(),
            request_id,
            timestamp: Utc::now(),
            status: ActionResponseStatus::Success,
            action,
            data: data,
            error: None,
        }
    }

    /// Creates an error response with the specified error code and message
    pub fn error(request_id: String, action: Action, error: ActionError) -> Self {
        Self {
            id: Uuid::new_v4(),
            request_id,
            timestamp: Utc::now(),
            status: ActionResponseStatus::Error,
            action,
            data: None,
            error: Some(error),
        }
    }
}
