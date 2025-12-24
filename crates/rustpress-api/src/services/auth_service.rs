//! Authentication service.

use rustpress_core::error::Result;
use uuid::Uuid;

/// Authentication service for handling auth-related business logic
pub struct AuthService;

impl AuthService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}
