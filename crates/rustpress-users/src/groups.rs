//! # User Groups & Invitations
//!
//! User groups, invitations, approval workflow, directory, and privacy.
//!
//! Features:
//! - User invitation system
//! - User approval workflow
//! - User groups beyond roles
//! - User directory/member listing
//! - Profile privacy settings

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Invitation System
// ============================================================================

/// User invitation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invitation {
    pub id: Uuid,
    pub email: String,
    pub invited_by: i64,
    pub role: String,
    pub group_ids: Vec<Uuid>,
    pub token: String,
    pub message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub user_id: Option<i64>,
    pub status: InvitationStatus,
}

/// Invitation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvitationStatus {
    Pending,
    Accepted,
    Expired,
    Revoked,
}

impl Invitation {
    pub fn new(email: &str, invited_by: i64, role: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            email: email.to_string(),
            invited_by,
            role: role.to_string(),
            group_ids: Vec::new(),
            token: Uuid::new_v4().to_string(),
            message: None,
            created_at: now,
            expires_at: now + Duration::days(7),
            accepted_at: None,
            user_id: None,
            status: InvitationStatus::Pending,
        }
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = Some(message.to_string());
        self
    }

    pub fn with_groups(mut self, group_ids: Vec<Uuid>) -> Self {
        self.group_ids = group_ids;
        self
    }

    pub fn with_expiry(mut self, days: i64) -> Self {
        self.expires_at = Utc::now() + Duration::days(days);
        self
    }

    pub fn is_valid(&self) -> bool {
        self.status == InvitationStatus::Pending && Utc::now() < self.expires_at
    }

    pub fn accept(&mut self, user_id: i64) {
        self.status = InvitationStatus::Accepted;
        self.accepted_at = Some(Utc::now());
        self.user_id = Some(user_id);
    }

    pub fn revoke(&mut self) {
        self.status = InvitationStatus::Revoked;
    }

    pub fn invitation_url(&self, base_url: &str) -> String {
        format!("{}/register?invite={}", base_url, self.token)
    }
}

/// Invitation manager
pub struct InvitationManager {
    invitations: HashMap<Uuid, Invitation>,
    by_token: HashMap<String, Uuid>,
    by_email: HashMap<String, Vec<Uuid>>,
    max_pending_per_email: usize,
}

impl Default for InvitationManager {
    fn default() -> Self {
        Self {
            invitations: HashMap::new(),
            by_token: HashMap::new(),
            by_email: HashMap::new(),
            max_pending_per_email: 3,
        }
    }
}

impl InvitationManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Send invitation
    pub fn invite(&mut self, invitation: Invitation) -> Result<&Invitation, String> {
        // Check for existing pending invitations
        let pending_count = self.by_email.get(&invitation.email)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.invitations.get(id))
                    .filter(|i| i.is_valid())
                    .count()
            })
            .unwrap_or(0);

        if pending_count >= self.max_pending_per_email {
            return Err("Too many pending invitations for this email".to_string());
        }

        let id = invitation.id;
        let token = invitation.token.clone();
        let email = invitation.email.clone();

        self.invitations.insert(id, invitation);
        self.by_token.insert(token, id);
        self.by_email.entry(email).or_insert_with(Vec::new).push(id);

        Ok(self.invitations.get(&id).unwrap())
    }

    /// Get invitation by token
    pub fn get_by_token(&self, token: &str) -> Option<&Invitation> {
        self.by_token.get(token)
            .and_then(|id| self.invitations.get(id))
    }

    /// Validate and accept invitation
    pub fn accept(&mut self, token: &str, user_id: i64) -> Result<&Invitation, String> {
        let invitation_id = *self.by_token.get(token)
            .ok_or_else(|| "Invalid invitation token".to_string())?;

        let invitation = self.invitations.get_mut(&invitation_id)
            .ok_or_else(|| "Invitation not found".to_string())?;

        if !invitation.is_valid() {
            return Err("Invitation has expired or been revoked".to_string());
        }

        invitation.accept(user_id);

        Ok(self.invitations.get(&invitation_id).unwrap())
    }

    /// Revoke invitation
    pub fn revoke(&mut self, invitation_id: Uuid) -> Result<(), String> {
        let invitation = self.invitations.get_mut(&invitation_id)
            .ok_or_else(|| "Invitation not found".to_string())?;

        invitation.revoke();
        Ok(())
    }

    /// Get invitations sent by user
    pub fn get_sent_by(&self, user_id: i64) -> Vec<&Invitation> {
        self.invitations.values()
            .filter(|i| i.invited_by == user_id)
            .collect()
    }

    /// Get pending invitations for email
    pub fn get_pending_for_email(&self, email: &str) -> Vec<&Invitation> {
        self.by_email.get(email)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.invitations.get(id))
                    .filter(|i| i.is_valid())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Cleanup expired invitations
    pub fn cleanup_expired(&mut self) {
        let expired: Vec<Uuid> = self.invitations.iter()
            .filter(|(_, i)| i.status == InvitationStatus::Pending && Utc::now() > i.expires_at)
            .map(|(id, _)| *id)
            .collect();

        for id in expired {
            if let Some(invitation) = self.invitations.get_mut(&id) {
                invitation.status = InvitationStatus::Expired;
            }
        }
    }
}

// ============================================================================
// Approval Workflow
// ============================================================================

/// User registration that requires approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub password_hash: String,
    pub requested_role: String,
    pub reason: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub created_at: DateTime<Utc>,
    pub status: ApprovalStatus,
    pub reviewed_by: Option<i64>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_notes: Option<String>,
    pub user_id: Option<i64>,
}

/// Approval status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Spam,
}

impl PendingUser {
    pub fn new(username: &str, email: &str, password_hash: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            username: username.to_string(),
            email: email.to_string(),
            display_name: username.to_string(),
            password_hash: password_hash.to_string(),
            requested_role: "subscriber".to_string(),
            reason: None,
            ip_address: String::new(),
            user_agent: String::new(),
            created_at: Utc::now(),
            status: ApprovalStatus::Pending,
            reviewed_by: None,
            reviewed_at: None,
            review_notes: None,
            user_id: None,
        }
    }

    pub fn approve(&mut self, by_user: i64, user_id: i64, notes: Option<&str>) {
        self.status = ApprovalStatus::Approved;
        self.reviewed_by = Some(by_user);
        self.reviewed_at = Some(Utc::now());
        self.review_notes = notes.map(String::from);
        self.user_id = Some(user_id);
    }

    pub fn reject(&mut self, by_user: i64, notes: Option<&str>) {
        self.status = ApprovalStatus::Rejected;
        self.reviewed_by = Some(by_user);
        self.reviewed_at = Some(Utc::now());
        self.review_notes = notes.map(String::from);
    }

    pub fn mark_spam(&mut self, by_user: i64) {
        self.status = ApprovalStatus::Spam;
        self.reviewed_by = Some(by_user);
        self.reviewed_at = Some(Utc::now());
    }
}

/// Approval workflow manager
pub struct ApprovalManager {
    pending: HashMap<Uuid, PendingUser>,
    auto_approve_rules: Vec<AutoApproveRule>,
}

/// Auto-approve rule
#[derive(Debug, Clone)]
pub struct AutoApproveRule {
    pub id: String,
    pub rule_type: AutoApproveRuleType,
    pub value: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoApproveRuleType {
    EmailDomain,
    InvitedBy,
    HasInvitation,
}

impl Default for ApprovalManager {
    fn default() -> Self {
        Self {
            pending: HashMap::new(),
            auto_approve_rules: Vec::new(),
        }
    }
}

impl ApprovalManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add pending user
    pub fn add_pending(&mut self, user: PendingUser) -> Uuid {
        let id = user.id;
        self.pending.insert(id, user);
        id
    }

    /// Check if should auto-approve
    pub fn should_auto_approve(&self, user: &PendingUser, has_invitation: bool) -> bool {
        for rule in &self.auto_approve_rules {
            if !rule.enabled {
                continue;
            }

            match rule.rule_type {
                AutoApproveRuleType::EmailDomain => {
                    if user.email.ends_with(&format!("@{}", rule.value)) {
                        return true;
                    }
                }
                AutoApproveRuleType::HasInvitation => {
                    if has_invitation {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Get pending users
    pub fn get_pending(&self) -> Vec<&PendingUser> {
        self.pending.values()
            .filter(|u| u.status == ApprovalStatus::Pending)
            .collect()
    }

    /// Approve user
    pub fn approve(&mut self, pending_id: Uuid, by_user: i64, user_id: i64, notes: Option<&str>) -> Result<(), String> {
        let pending = self.pending.get_mut(&pending_id)
            .ok_or_else(|| "Pending user not found".to_string())?;

        pending.approve(by_user, user_id, notes);
        Ok(())
    }

    /// Reject user
    pub fn reject(&mut self, pending_id: Uuid, by_user: i64, notes: Option<&str>) -> Result<(), String> {
        let pending = self.pending.get_mut(&pending_id)
            .ok_or_else(|| "Pending user not found".to_string())?;

        pending.reject(by_user, notes);
        Ok(())
    }

    /// Add auto-approve rule
    pub fn add_rule(&mut self, rule: AutoApproveRule) {
        self.auto_approve_rules.push(rule);
    }
}

// ============================================================================
// User Groups
// ============================================================================

/// User group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroup {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub group_type: GroupType,
    pub visibility: GroupVisibility,
    pub created_by: i64,
    pub created_at: DateTime<Utc>,
    pub member_count: u32,
    pub settings: GroupSettings,
}

/// Group types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupType {
    Open,
    Approval,
    Invite,
    Hidden,
}

/// Group visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupVisibility {
    Public,
    Members,
    Private,
}

/// Group settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettings {
    pub allow_member_list: bool,
    pub allow_member_invites: bool,
    pub enable_forum: bool,
    pub enable_activity_feed: bool,
    pub max_members: Option<u32>,
}

impl Default for GroupSettings {
    fn default() -> Self {
        Self {
            allow_member_list: true,
            allow_member_invites: false,
            enable_forum: false,
            enable_activity_feed: true,
            max_members: None,
        }
    }
}

impl UserGroup {
    pub fn new(slug: &str, name: &str, created_by: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            slug: slug.to_string(),
            name: name.to_string(),
            description: String::new(),
            group_type: GroupType::Open,
            visibility: GroupVisibility::Public,
            created_by,
            created_at: Utc::now(),
            member_count: 0,
            settings: GroupSettings::default(),
        }
    }
}

/// Group membership
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupMember {
    pub group_id: Uuid,
    pub user_id: i64,
    pub role: GroupRole,
    pub joined_at: DateTime<Utc>,
    pub invited_by: Option<i64>,
}

/// Group roles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupRole {
    Member,
    Moderator,
    Admin,
    Owner,
}

impl GroupRole {
    pub fn can_moderate(&self) -> bool {
        matches!(self, Self::Moderator | Self::Admin | Self::Owner)
    }

    pub fn can_admin(&self) -> bool {
        matches!(self, Self::Admin | Self::Owner)
    }
}

/// Group manager
pub struct GroupManager {
    groups: HashMap<Uuid, UserGroup>,
    members: HashMap<Uuid, Vec<GroupMember>>,
    user_groups: HashMap<i64, Vec<Uuid>>,
}

impl Default for GroupManager {
    fn default() -> Self {
        Self {
            groups: HashMap::new(),
            members: HashMap::new(),
            user_groups: HashMap::new(),
        }
    }
}

impl GroupManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create group
    pub fn create(&mut self, group: UserGroup, owner_id: i64) -> Uuid {
        let id = group.id;
        self.groups.insert(id, group);

        // Add owner as member
        self.add_member(id, owner_id, GroupRole::Owner, None);

        id
    }

    /// Get group
    pub fn get(&self, id: Uuid) -> Option<&UserGroup> {
        self.groups.get(&id)
    }

    /// Add member
    pub fn add_member(&mut self, group_id: Uuid, user_id: i64, role: GroupRole, invited_by: Option<i64>) {
        let member = GroupMember {
            group_id,
            user_id,
            role,
            joined_at: Utc::now(),
            invited_by,
        };

        self.members.entry(group_id)
            .or_insert_with(Vec::new)
            .push(member);

        self.user_groups.entry(user_id)
            .or_insert_with(Vec::new)
            .push(group_id);

        if let Some(group) = self.groups.get_mut(&group_id) {
            group.member_count += 1;
        }
    }

    /// Remove member
    pub fn remove_member(&mut self, group_id: Uuid, user_id: i64) -> Result<(), String> {
        let members = self.members.get_mut(&group_id)
            .ok_or_else(|| "Group not found".to_string())?;

        // Cannot remove owner
        if members.iter().any(|m| m.user_id == user_id && m.role == GroupRole::Owner) {
            return Err("Cannot remove group owner".to_string());
        }

        members.retain(|m| m.user_id != user_id);

        if let Some(groups) = self.user_groups.get_mut(&user_id) {
            groups.retain(|&id| id != group_id);
        }

        if let Some(group) = self.groups.get_mut(&group_id) {
            group.member_count = group.member_count.saturating_sub(1);
        }

        Ok(())
    }

    /// Get group members
    pub fn get_members(&self, group_id: Uuid) -> Vec<&GroupMember> {
        self.members.get(&group_id)
            .map(|m| m.iter().collect())
            .unwrap_or_default()
    }

    /// Get user's groups
    pub fn get_user_groups(&self, user_id: i64) -> Vec<&UserGroup> {
        self.user_groups.get(&user_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.groups.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if user is member
    pub fn is_member(&self, group_id: Uuid, user_id: i64) -> bool {
        self.members.get(&group_id)
            .map(|m| m.iter().any(|member| member.user_id == user_id))
            .unwrap_or(false)
    }

    /// Get public groups
    pub fn get_public_groups(&self) -> Vec<&UserGroup> {
        self.groups.values()
            .filter(|g| g.visibility == GroupVisibility::Public)
            .collect()
    }
}

// ============================================================================
// User Directory
// ============================================================================

/// Directory listing item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub joined_date: DateTime<Utc>,
    pub last_active: Option<DateTime<Utc>>,
    pub posts_count: u32,
    pub followers_count: u32,
    pub groups: Vec<String>,
    pub badges: Vec<String>,
    pub is_online: bool,
}

/// Directory query
#[derive(Debug, Clone, Default)]
pub struct DirectoryQuery {
    pub search: Option<String>,
    pub role: Option<String>,
    pub group_id: Option<Uuid>,
    pub order_by: DirectoryOrderBy,
    pub page: u32,
    pub per_page: u32,
    pub only_online: bool,
}

/// Directory sort options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DirectoryOrderBy {
    #[default]
    Name,
    JoinedDate,
    LastActive,
    PostsCount,
    FollowersCount,
    Alphabetical,
}

impl DirectoryQuery {
    pub fn new() -> Self {
        Self {
            page: 1,
            per_page: 20,
            ..Default::default()
        }
    }
}

// ============================================================================
// Privacy Settings
// ============================================================================

/// User privacy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub user_id: i64,
    pub profile_visibility: ProfilePrivacy,
    pub email_visibility: FieldPrivacy,
    pub activity_visibility: FieldPrivacy,
    pub location_visibility: FieldPrivacy,
    pub social_links_visibility: FieldPrivacy,
    pub online_status_visibility: FieldPrivacy,
    pub allow_search_engines: bool,
    pub allow_mentions: bool,
    pub allow_messages: MessagePrivacy,
    pub allow_friend_requests: bool,
    pub show_in_directory: bool,
    pub hide_from_suggestions: bool,
    pub blocked_users: Vec<i64>,
}

/// Profile privacy levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfilePrivacy {
    Public,
    RegisteredOnly,
    FollowersOnly,
    Private,
}

/// Field privacy levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FieldPrivacy {
    Public,
    Registered,
    Followers,
    Nobody,
}

/// Message privacy levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagePrivacy {
    Anyone,
    Followers,
    Following,
    Mutual,
    Nobody,
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            user_id: 0,
            profile_visibility: ProfilePrivacy::Public,
            email_visibility: FieldPrivacy::Nobody,
            activity_visibility: FieldPrivacy::Public,
            location_visibility: FieldPrivacy::Public,
            social_links_visibility: FieldPrivacy::Public,
            online_status_visibility: FieldPrivacy::Registered,
            allow_search_engines: true,
            allow_mentions: true,
            allow_messages: MessagePrivacy::Anyone,
            allow_friend_requests: true,
            show_in_directory: true,
            hide_from_suggestions: false,
            blocked_users: Vec::new(),
        }
    }
}

impl PrivacySettings {
    pub fn new(user_id: i64) -> Self {
        Self {
            user_id,
            ..Default::default()
        }
    }

    pub fn can_view_profile(&self, viewer_id: Option<i64>, is_following: bool) -> bool {
        match self.profile_visibility {
            ProfilePrivacy::Public => true,
            ProfilePrivacy::RegisteredOnly => viewer_id.is_some(),
            ProfilePrivacy::FollowersOnly => is_following,
            ProfilePrivacy::Private => viewer_id == Some(self.user_id),
        }
    }

    pub fn can_view_field(&self, privacy: FieldPrivacy, viewer_id: Option<i64>, is_following: bool) -> bool {
        match privacy {
            FieldPrivacy::Public => true,
            FieldPrivacy::Registered => viewer_id.is_some(),
            FieldPrivacy::Followers => is_following,
            FieldPrivacy::Nobody => false,
        }
    }

    pub fn can_message(&self, from_id: i64, is_follower: bool, is_following: bool) -> bool {
        if self.blocked_users.contains(&from_id) {
            return false;
        }

        match self.allow_messages {
            MessagePrivacy::Anyone => true,
            MessagePrivacy::Followers => is_follower,
            MessagePrivacy::Following => is_following,
            MessagePrivacy::Mutual => is_follower && is_following,
            MessagePrivacy::Nobody => false,
        }
    }

    pub fn is_blocked(&self, user_id: i64) -> bool {
        self.blocked_users.contains(&user_id)
    }

    pub fn block_user(&mut self, user_id: i64) {
        if !self.blocked_users.contains(&user_id) {
            self.blocked_users.push(user_id);
        }
    }

    pub fn unblock_user(&mut self, user_id: i64) {
        self.blocked_users.retain(|&id| id != user_id);
    }
}

/// Privacy settings manager
pub struct PrivacyManager {
    settings: HashMap<i64, PrivacySettings>,
}

impl Default for PrivacyManager {
    fn default() -> Self {
        Self {
            settings: HashMap::new(),
        }
    }
}

impl PrivacyManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_create(&mut self, user_id: i64) -> &mut PrivacySettings {
        self.settings.entry(user_id)
            .or_insert_with(|| PrivacySettings::new(user_id))
    }

    pub fn get(&self, user_id: i64) -> Option<&PrivacySettings> {
        self.settings.get(&user_id)
    }

    pub fn save(&mut self, settings: PrivacySettings) {
        self.settings.insert(settings.user_id, settings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invitation() {
        let mut manager = InvitationManager::new();

        let invite = Invitation::new("test@example.com", 1, "subscriber");
        let token = invite.token.clone();

        manager.invite(invite).unwrap();

        assert!(manager.get_by_token(&token).is_some());
        manager.accept(&token, 2).unwrap();

        let accepted = manager.get_by_token(&token).unwrap();
        assert_eq!(accepted.status, InvitationStatus::Accepted);
    }

    #[test]
    fn test_approval_workflow() {
        let mut manager = ApprovalManager::new();

        let pending = PendingUser::new("testuser", "test@example.com", "hash");
        let id = manager.add_pending(pending);

        manager.approve(id, 1, 100, Some("Looks good")).unwrap();

        let pending_list = manager.get_pending();
        assert!(pending_list.is_empty());
    }

    #[test]
    fn test_user_groups() {
        let mut manager = GroupManager::new();

        let group = UserGroup::new("developers", "Developers", 1);
        let id = manager.create(group, 1);

        assert!(manager.is_member(id, 1));

        manager.add_member(id, 2, GroupRole::Member, Some(1));
        assert!(manager.is_member(id, 2));
    }

    #[test]
    fn test_privacy_settings() {
        let mut settings = PrivacySettings::new(1);

        // Public profile, anyone can view
        assert!(settings.can_view_profile(None, false));

        settings.profile_visibility = ProfilePrivacy::FollowersOnly;
        assert!(!settings.can_view_profile(Some(2), false));
        assert!(settings.can_view_profile(Some(2), true));
    }

    #[test]
    fn test_blocking() {
        let mut settings = PrivacySettings::new(1);

        settings.block_user(2);
        assert!(settings.is_blocked(2));
        assert!(!settings.can_message(2, true, true));

        settings.unblock_user(2);
        assert!(!settings.is_blocked(2));
    }
}
