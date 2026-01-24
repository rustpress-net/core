//! Real-time Collaboration
//!
//! Google Docs-style collaborative editing support.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Collaboration session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSession {
    /// Session ID
    pub id: Uuid,

    /// Post ID being edited
    pub post_id: i64,

    /// Active users
    pub users: HashMap<i64, CollaboratorInfo>,

    /// Current operations queue
    #[serde(skip)]
    pub operations: Vec<Operation>,

    /// Session started at
    pub started_at: DateTime<Utc>,

    /// Session settings
    pub settings: CollaborationSettings,
}

impl CollaborationSession {
    pub fn new(post_id: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            post_id,
            users: HashMap::new(),
            operations: Vec::new(),
            started_at: Utc::now(),
            settings: CollaborationSettings::default(),
        }
    }

    /// Add a user to the session
    pub fn join(&mut self, user: CollaboratorInfo) {
        self.users.insert(user.user_id, user);
    }

    /// Remove a user from the session
    pub fn leave(&mut self, user_id: i64) {
        self.users.remove(&user_id);
    }

    /// Update user cursor position
    pub fn update_cursor(&mut self, user_id: i64, cursor: CursorInfo) {
        if let Some(user) = self.users.get_mut(&user_id) {
            user.cursor = Some(cursor);
            user.last_active = Utc::now();
        }
    }

    /// Get active user count
    pub fn active_user_count(&self) -> usize {
        self.users.len()
    }

    /// Check if session is empty
    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }
}

/// Collaborator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaboratorInfo {
    /// User ID
    pub user_id: i64,

    /// Display name
    pub name: String,

    /// Avatar URL
    pub avatar_url: Option<String>,

    /// Assigned color for cursor/selection
    pub color: String,

    /// Current cursor position
    pub cursor: Option<CursorInfo>,

    /// Last activity timestamp
    pub last_active: DateTime<Utc>,

    /// User role in this document
    pub role: CollaboratorRole,

    /// Currently editing block
    pub editing_block: Option<Uuid>,
}

impl CollaboratorInfo {
    pub fn new(user_id: i64, name: String, color: String) -> Self {
        Self {
            user_id,
            name,
            avatar_url: None,
            color,
            cursor: None,
            last_active: Utc::now(),
            role: CollaboratorRole::Editor,
            editing_block: None,
        }
    }
}

/// Collaborator role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CollaboratorRole {
    /// Can view only
    Viewer,
    /// Can suggest changes
    Commenter,
    /// Can edit
    Editor,
    /// Full control
    Owner,
}

/// Cursor position information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorInfo {
    /// Block ID
    pub block_id: Uuid,

    /// Character offset within block
    pub offset: usize,

    /// Selection start (if selecting)
    pub selection_start: Option<SelectionPoint>,

    /// Selection end (if selecting)
    pub selection_end: Option<SelectionPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionPoint {
    pub block_id: Uuid,
    pub offset: usize,
}

/// Collaboration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationSettings {
    /// Allow anonymous viewers
    pub allow_anonymous: bool,

    /// Show user cursors
    pub show_cursors: bool,

    /// Show user names on cursors
    pub show_cursor_names: bool,

    /// Sync interval in milliseconds
    pub sync_interval_ms: u32,

    /// Auto-save interval
    pub autosave_interval_ms: u32,

    /// Maximum collaborators
    pub max_collaborators: u32,
}

impl Default for CollaborationSettings {
    fn default() -> Self {
        Self {
            allow_anonymous: false,
            show_cursors: true,
            show_cursor_names: true,
            sync_interval_ms: 100,
            autosave_interval_ms: 30000,
            max_collaborators: 10,
        }
    }
}

/// Operational Transform operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation ID
    pub id: Uuid,

    /// Operation type
    pub op_type: OperationType,

    /// User who performed the operation
    pub user_id: i64,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Operation version (for OT)
    pub version: u64,
}

/// Operation types for OT
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OperationType {
    /// Insert text
    InsertText {
        block_id: Uuid,
        offset: usize,
        text: String,
    },

    /// Delete text
    DeleteText {
        block_id: Uuid,
        offset: usize,
        length: usize,
    },

    /// Replace text
    ReplaceText {
        block_id: Uuid,
        offset: usize,
        length: usize,
        text: String,
    },

    /// Insert block
    InsertBlock {
        parent_id: Option<Uuid>,
        index: usize,
        block: serde_json::Value,
    },

    /// Delete block
    DeleteBlock { block_id: Uuid },

    /// Move block
    MoveBlock {
        block_id: Uuid,
        new_parent_id: Option<Uuid>,
        new_index: usize,
    },

    /// Update block attributes
    UpdateBlockAttributes {
        block_id: Uuid,
        attributes: serde_json::Value,
    },

    /// Update block styles
    UpdateBlockStyles {
        block_id: Uuid,
        styles: serde_json::Value,
    },

    /// Set selection
    SetSelection {
        block_id: Uuid,
        start: usize,
        end: usize,
    },

    /// Format text
    FormatText {
        block_id: Uuid,
        offset: usize,
        length: usize,
        format: TextFormat,
    },
}

/// Text formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFormat {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strikethrough: Option<bool>,
    pub code: Option<bool>,
    pub link: Option<String>,
    pub color: Option<String>,
    pub background_color: Option<String>,
}

/// Comment/suggestion on content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Comment ID
    pub id: Uuid,

    /// Author
    pub author_id: i64,
    pub author_name: String,
    pub author_avatar: Option<String>,

    /// Comment content
    pub content: String,

    /// Target block
    pub block_id: Option<Uuid>,

    /// Target text range (if commenting on specific text)
    pub text_range: Option<TextRange>,

    /// Comment type
    pub comment_type: CommentType,

    /// Status
    pub status: CommentStatus,

    /// Replies
    #[serde(default)]
    pub replies: Vec<CommentReply>,

    /// Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextRange {
    pub start_offset: usize,
    pub end_offset: usize,
    pub quoted_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommentType {
    Comment,
    Suggestion,
    Question,
    Task,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommentStatus {
    Open,
    Resolved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentReply {
    pub id: Uuid,
    pub author_id: i64,
    pub author_name: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CollaborationMessage {
    /// User joined
    UserJoined { user: CollaboratorInfo },

    /// User left
    UserLeft { user_id: i64 },

    /// Cursor update
    CursorUpdate { user_id: i64, cursor: CursorInfo },

    /// Operation
    Operation { operation: Operation },

    /// Operations batch
    OperationsBatch { operations: Vec<Operation> },

    /// Comment added
    CommentAdded { comment: Comment },

    /// Comment updated
    CommentUpdated { comment: Comment },

    /// Comment deleted
    CommentDeleted { comment_id: Uuid },

    /// Document saved
    DocumentSaved { version: u32, saved_by: i64 },

    /// Sync request
    SyncRequest { from_version: u64 },

    /// Sync response
    SyncResponse {
        operations: Vec<Operation>,
        current_version: u64,
    },

    /// Lock block (prevent editing)
    LockBlock { block_id: Uuid, user_id: i64 },

    /// Unlock block
    UnlockBlock { block_id: Uuid },

    /// Error
    Error { code: String, message: String },

    /// Ping/Pong for keepalive
    Ping,
    Pong,
}

/// Predefined cursor colors for collaborators
pub const CURSOR_COLORS: &[&str] = &[
    "#ef4444", // red
    "#f97316", // orange
    "#eab308", // yellow
    "#22c55e", // green
    "#14b8a6", // teal
    "#3b82f6", // blue
    "#8b5cf6", // violet
    "#ec4899", // pink
    "#f43f5e", // rose
    "#06b6d4", // cyan
];

/// Get a color for a collaborator based on their position
pub fn get_collaborator_color(index: usize) -> &'static str {
    CURSOR_COLORS[index % CURSOR_COLORS.len()]
}
