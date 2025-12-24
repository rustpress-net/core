//! # Social Features
//!
//! Author pages, following system, messaging, reputation, and badges.
//!
//! Features:
//! - Author archives and bio pages
//! - User following/subscription system
//! - User messaging/inbox
//! - Reputation/points system
//! - Badges/achievements

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// Author Archives & Bio
// ============================================================================

/// Author page data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorPage {
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    pub slug: String,
    pub avatar_url: Option<String>,
    pub bio: String,
    pub rich_bio: Option<String>,
    pub website: Option<String>,
    pub social_links: HashMap<String, String>,
    pub meta: AuthorMeta,
    pub stats: AuthorStats,
}

/// Author metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorMeta {
    pub title: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub joined_date: DateTime<Utc>,
    pub expertise: Vec<String>,
    pub custom_fields: HashMap<String, String>,
}

impl Default for AuthorMeta {
    fn default() -> Self {
        Self {
            title: None,
            company: None,
            location: None,
            joined_date: Utc::now(),
            expertise: Vec::new(),
            custom_fields: HashMap::new(),
        }
    }
}

/// Author statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthorStats {
    pub posts_count: u32,
    pub comments_count: u32,
    pub followers_count: u32,
    pub following_count: u32,
    pub total_views: u64,
    pub likes_received: u32,
    pub reputation: i32,
    pub badges_count: u32,
}

/// Author archive settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorArchiveSettings {
    pub enabled: bool,
    pub posts_per_page: u32,
    pub show_bio: bool,
    pub show_avatar: bool,
    pub show_social_links: bool,
    pub show_stats: bool,
    pub default_sort: AuthorPostSort,
    pub post_types: Vec<String>,
}

/// Author post sorting options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorPostSort {
    DateDesc,
    DateAsc,
    TitleAsc,
    TitleDesc,
    ViewsDesc,
    CommentsDesc,
}

impl Default for AuthorArchiveSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            posts_per_page: 10,
            show_bio: true,
            show_avatar: true,
            show_social_links: true,
            show_stats: true,
            default_sort: AuthorPostSort::DateDesc,
            post_types: vec!["post".to_string()],
        }
    }
}

// ============================================================================
// Following System
// ============================================================================

/// Follow relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Follow {
    pub id: Uuid,
    pub follower_id: i64,
    pub following_id: i64,
    pub followed_at: DateTime<Utc>,
    pub notify_on_post: bool,
    pub notify_on_comment: bool,
}

impl Follow {
    pub fn new(follower_id: i64, following_id: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            follower_id,
            following_id,
            followed_at: Utc::now(),
            notify_on_post: true,
            notify_on_comment: false,
        }
    }
}

/// Content subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: i64,
    pub subscription_type: SubscriptionType,
    pub target_id: i64,
    pub created_at: DateTime<Utc>,
    pub notify_on_update: bool,
    pub notify_on_comment: bool,
}

/// Subscription types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubscriptionType {
    Post,
    Page,
    Category,
    Tag,
    Author,
    Topic,
}

impl Subscription {
    pub fn new(user_id: i64, subscription_type: SubscriptionType, target_id: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            subscription_type,
            target_id,
            created_at: Utc::now(),
            notify_on_update: true,
            notify_on_comment: true,
        }
    }
}

/// Follow manager
pub struct FollowManager {
    follows: Vec<Follow>,
    subscriptions: Vec<Subscription>,
}

impl Default for FollowManager {
    fn default() -> Self {
        Self {
            follows: Vec::new(),
            subscriptions: Vec::new(),
        }
    }
}

impl FollowManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Follow a user
    pub fn follow(&mut self, follower_id: i64, following_id: i64) -> Result<&Follow, String> {
        if follower_id == following_id {
            return Err("Cannot follow yourself".to_string());
        }

        if self.is_following(follower_id, following_id) {
            return Err("Already following this user".to_string());
        }

        let follow = Follow::new(follower_id, following_id);
        self.follows.push(follow);
        Ok(self.follows.last().unwrap())
    }

    /// Unfollow a user
    pub fn unfollow(&mut self, follower_id: i64, following_id: i64) -> bool {
        let initial_len = self.follows.len();
        self.follows.retain(|f| !(f.follower_id == follower_id && f.following_id == following_id));
        self.follows.len() < initial_len
    }

    /// Check if following
    pub fn is_following(&self, follower_id: i64, following_id: i64) -> bool {
        self.follows.iter().any(|f| f.follower_id == follower_id && f.following_id == following_id)
    }

    /// Get followers of a user
    pub fn get_followers(&self, user_id: i64) -> Vec<&Follow> {
        self.follows.iter()
            .filter(|f| f.following_id == user_id)
            .collect()
    }

    /// Get users that a user follows
    pub fn get_following(&self, user_id: i64) -> Vec<&Follow> {
        self.follows.iter()
            .filter(|f| f.follower_id == user_id)
            .collect()
    }

    /// Get follower count
    pub fn follower_count(&self, user_id: i64) -> usize {
        self.follows.iter().filter(|f| f.following_id == user_id).count()
    }

    /// Get following count
    pub fn following_count(&self, user_id: i64) -> usize {
        self.follows.iter().filter(|f| f.follower_id == user_id).count()
    }

    /// Subscribe to content
    pub fn subscribe(&mut self, subscription: Subscription) {
        if !self.is_subscribed(subscription.user_id, &subscription.subscription_type, subscription.target_id) {
            self.subscriptions.push(subscription);
        }
    }

    /// Unsubscribe
    pub fn unsubscribe(&mut self, user_id: i64, subscription_type: &SubscriptionType, target_id: i64) {
        self.subscriptions.retain(|s| {
            !(s.user_id == user_id && s.subscription_type == *subscription_type && s.target_id == target_id)
        });
    }

    /// Check subscription
    pub fn is_subscribed(&self, user_id: i64, subscription_type: &SubscriptionType, target_id: i64) -> bool {
        self.subscriptions.iter().any(|s| {
            s.user_id == user_id && s.subscription_type == *subscription_type && s.target_id == target_id
        })
    }

    /// Get subscriptions for user
    pub fn get_subscriptions(&self, user_id: i64) -> Vec<&Subscription> {
        self.subscriptions.iter()
            .filter(|s| s.user_id == user_id)
            .collect()
    }

    /// Get subscribers for content
    pub fn get_subscribers(&self, subscription_type: &SubscriptionType, target_id: i64) -> Vec<i64> {
        self.subscriptions.iter()
            .filter(|s| s.subscription_type == *subscription_type && s.target_id == target_id)
            .map(|s| s.user_id)
            .collect()
    }
}

// ============================================================================
// Messaging System
// ============================================================================

/// Private message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub sender_id: i64,
    pub content: String,
    pub sent_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted: bool,
    pub attachments: Vec<MessageAttachment>,
}

/// Message attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub id: Uuid,
    pub filename: String,
    pub url: String,
    pub mime_type: String,
    pub size: u64,
}

/// Message thread (conversation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageThread {
    pub id: Uuid,
    pub participants: Vec<i64>,
    pub subject: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<Message>,
    pub is_group: bool,
}

impl MessageThread {
    pub fn new(participants: Vec<i64>, subject: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            participants,
            subject,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            messages: Vec::new(),
            is_group: false,
        }
    }

    pub fn group(participants: Vec<i64>, subject: &str) -> Self {
        Self {
            id: Uuid::new_v4(),
            participants,
            subject: Some(subject.to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            messages: Vec::new(),
            is_group: true,
        }
    }

    pub fn add_message(&mut self, sender_id: i64, content: &str) -> &Message {
        let message = Message {
            id: Uuid::new_v4(),
            thread_id: self.id,
            sender_id,
            content: content.to_string(),
            sent_at: Utc::now(),
            read_at: None,
            edited_at: None,
            deleted: false,
            attachments: Vec::new(),
        };
        self.messages.push(message);
        self.updated_at = Utc::now();
        self.messages.last().unwrap()
    }

    pub fn get_other_participant(&self, user_id: i64) -> Option<i64> {
        self.participants.iter()
            .find(|&&p| p != user_id)
            .copied()
    }

    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    pub fn unread_count(&self, user_id: i64) -> usize {
        self.messages.iter()
            .filter(|m| m.sender_id != user_id && m.read_at.is_none() && !m.deleted)
            .count()
    }
}

/// Inbox manager
pub struct InboxManager {
    threads: HashMap<Uuid, MessageThread>,
    user_threads: HashMap<i64, Vec<Uuid>>,
    blocked_users: HashMap<i64, Vec<i64>>,
}

impl Default for InboxManager {
    fn default() -> Self {
        Self {
            threads: HashMap::new(),
            user_threads: HashMap::new(),
            blocked_users: HashMap::new(),
        }
    }
}

impl InboxManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new conversation
    pub fn start_thread(&mut self, sender_id: i64, recipient_id: i64, message: &str) -> Result<&MessageThread, String> {
        // Check if blocked
        if self.is_blocked(recipient_id, sender_id) {
            return Err("You cannot message this user".to_string());
        }

        // Check for existing thread
        if let Some(thread_id) = self.find_existing_thread(sender_id, recipient_id) {
            let thread = self.threads.get_mut(&thread_id).unwrap();
            thread.add_message(sender_id, message);
            return Ok(self.threads.get(&thread_id).unwrap());
        }

        // Create new thread
        let mut thread = MessageThread::new(vec![sender_id, recipient_id], None);
        thread.add_message(sender_id, message);
        let thread_id = thread.id;

        self.threads.insert(thread_id, thread);

        // Update user thread lists
        self.user_threads.entry(sender_id)
            .or_insert_with(Vec::new)
            .push(thread_id);
        self.user_threads.entry(recipient_id)
            .or_insert_with(Vec::new)
            .push(thread_id);

        Ok(self.threads.get(&thread_id).unwrap())
    }

    /// Find existing thread between two users
    fn find_existing_thread(&self, user1: i64, user2: i64) -> Option<Uuid> {
        if let Some(thread_ids) = self.user_threads.get(&user1) {
            for thread_id in thread_ids {
                if let Some(thread) = self.threads.get(thread_id) {
                    if !thread.is_group && thread.participants.contains(&user2) {
                        return Some(*thread_id);
                    }
                }
            }
        }
        None
    }

    /// Send message to existing thread
    pub fn send_message(&mut self, thread_id: Uuid, sender_id: i64, content: &str) -> Result<(), String> {
        let thread = self.threads.get_mut(&thread_id)
            .ok_or_else(|| "Thread not found".to_string())?;

        if !thread.participants.contains(&sender_id) {
            return Err("Not a participant in this thread".to_string());
        }

        thread.add_message(sender_id, content);
        Ok(())
    }

    /// Get threads for user
    pub fn get_threads(&self, user_id: i64) -> Vec<&MessageThread> {
        self.user_threads.get(&user_id)
            .map(|thread_ids| {
                let mut threads: Vec<_> = thread_ids.iter()
                    .filter_map(|id| self.threads.get(id))
                    .collect();
                threads.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                threads
            })
            .unwrap_or_default()
    }

    /// Get thread by ID
    pub fn get_thread(&self, thread_id: Uuid) -> Option<&MessageThread> {
        self.threads.get(&thread_id)
    }

    /// Mark messages as read
    pub fn mark_as_read(&mut self, thread_id: Uuid, user_id: i64) {
        if let Some(thread) = self.threads.get_mut(&thread_id) {
            for message in &mut thread.messages {
                if message.sender_id != user_id && message.read_at.is_none() {
                    message.read_at = Some(Utc::now());
                }
            }
        }
    }

    /// Get unread count
    pub fn get_unread_count(&self, user_id: i64) -> usize {
        self.get_threads(user_id).iter()
            .map(|t| t.unread_count(user_id))
            .sum()
    }

    /// Block user
    pub fn block_user(&mut self, user_id: i64, blocked_id: i64) {
        self.blocked_users.entry(user_id)
            .or_insert_with(Vec::new)
            .push(blocked_id);
    }

    /// Unblock user
    pub fn unblock_user(&mut self, user_id: i64, blocked_id: i64) {
        if let Some(blocked) = self.blocked_users.get_mut(&user_id) {
            blocked.retain(|&id| id != blocked_id);
        }
    }

    /// Check if blocked
    pub fn is_blocked(&self, user_id: i64, by_user_id: i64) -> bool {
        self.blocked_users.get(&user_id)
            .map(|blocked| blocked.contains(&by_user_id))
            .unwrap_or(false)
    }

    /// Delete thread
    pub fn delete_thread(&mut self, thread_id: Uuid, user_id: i64) {
        // Just remove from user's list, don't delete actual thread
        if let Some(thread_ids) = self.user_threads.get_mut(&user_id) {
            thread_ids.retain(|&id| id != thread_id);
        }
    }
}

// ============================================================================
// Reputation System
// ============================================================================

/// Reputation action types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReputationAction {
    PostCreated,
    PostLiked,
    PostViewed,
    CommentCreated,
    CommentLiked,
    AnswerAccepted,
    BadgeEarned,
    FollowerGained,
    Custom(String),
}

impl ReputationAction {
    pub fn default_points(&self) -> i32 {
        match self {
            Self::PostCreated => 5,
            Self::PostLiked => 10,
            Self::PostViewed => 1,
            Self::CommentCreated => 2,
            Self::CommentLiked => 5,
            Self::AnswerAccepted => 15,
            Self::BadgeEarned => 10,
            Self::FollowerGained => 5,
            Self::Custom(_) => 0,
        }
    }
}

/// Reputation log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationLog {
    pub id: Uuid,
    pub user_id: i64,
    pub action: ReputationAction,
    pub points: i32,
    pub reference_type: Option<String>,
    pub reference_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub description: String,
}

/// User reputation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReputation {
    pub user_id: i64,
    pub total_points: i32,
    pub level: ReputationLevel,
    pub rank: Option<u32>,
    pub log: Vec<ReputationLog>,
}

/// Reputation levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReputationLevel {
    Newcomer,
    Contributor,
    Regular,
    Expert,
    Master,
    Legend,
}

impl ReputationLevel {
    pub fn from_points(points: i32) -> Self {
        match points {
            p if p < 100 => Self::Newcomer,
            p if p < 500 => Self::Contributor,
            p if p < 1000 => Self::Regular,
            p if p < 5000 => Self::Expert,
            p if p < 10000 => Self::Master,
            _ => Self::Legend,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Newcomer => "Newcomer",
            Self::Contributor => "Contributor",
            Self::Regular => "Regular",
            Self::Expert => "Expert",
            Self::Master => "Master",
            Self::Legend => "Legend",
        }
    }

    pub fn min_points(&self) -> i32 {
        match self {
            Self::Newcomer => 0,
            Self::Contributor => 100,
            Self::Regular => 500,
            Self::Expert => 1000,
            Self::Master => 5000,
            Self::Legend => 10000,
        }
    }
}

impl UserReputation {
    pub fn new(user_id: i64) -> Self {
        Self {
            user_id,
            total_points: 0,
            level: ReputationLevel::Newcomer,
            rank: None,
            log: Vec::new(),
        }
    }

    pub fn add_points(&mut self, action: ReputationAction, points: i32, description: &str) {
        self.total_points += points;
        self.level = ReputationLevel::from_points(self.total_points);

        self.log.push(ReputationLog {
            id: Uuid::new_v4(),
            user_id: self.user_id,
            action,
            points,
            reference_type: None,
            reference_id: None,
            created_at: Utc::now(),
            description: description.to_string(),
        });
    }

    pub fn points_to_next_level(&self) -> i32 {
        let next_level = match self.level {
            ReputationLevel::Newcomer => ReputationLevel::Contributor,
            ReputationLevel::Contributor => ReputationLevel::Regular,
            ReputationLevel::Regular => ReputationLevel::Expert,
            ReputationLevel::Expert => ReputationLevel::Master,
            ReputationLevel::Master => ReputationLevel::Legend,
            ReputationLevel::Legend => return 0,
        };
        next_level.min_points() - self.total_points
    }
}

// ============================================================================
// Badges/Achievements
// ============================================================================

/// Badge definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Badge {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub tier: BadgeTier,
    pub category: String,
    pub criteria: BadgeCriteria,
    pub points: i32,
    pub secret: bool,
}

/// Badge tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BadgeTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

impl BadgeTier {
    pub fn color(&self) -> &str {
        match self {
            Self::Bronze => "#cd7f32",
            Self::Silver => "#c0c0c0",
            Self::Gold => "#ffd700",
            Self::Platinum => "#e5e4e2",
        }
    }
}

/// Badge criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeCriteria {
    pub criteria_type: CriteriaType,
    pub threshold: i64,
}

/// Criteria types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CriteriaType {
    PostsCreated,
    CommentsCreated,
    LikesReceived,
    ViewsReceived,
    FollowersGained,
    DaysActive,
    ReputationReached,
    Custom(String),
}

/// Earned badge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarnedBadge {
    pub badge_id: String,
    pub user_id: i64,
    pub earned_at: DateTime<Utc>,
    pub progress_when_earned: i64,
}

/// Badge manager
pub struct BadgeManager {
    badges: HashMap<String, Badge>,
    earned: HashMap<i64, Vec<EarnedBadge>>,
    user_progress: HashMap<i64, HashMap<CriteriaType, i64>>,
}

impl Default for BadgeManager {
    fn default() -> Self {
        let mut manager = Self {
            badges: HashMap::new(),
            earned: HashMap::new(),
            user_progress: HashMap::new(),
        };
        manager.register_default_badges();
        manager
    }
}

impl BadgeManager {
    pub fn new() -> Self {
        Self::default()
    }

    fn register_default_badges(&mut self) {
        // Post badges
        self.register(Badge {
            id: "first_post".to_string(),
            name: "First Post".to_string(),
            description: "Published your first post".to_string(),
            icon: "edit".to_string(),
            tier: BadgeTier::Bronze,
            category: "content".to_string(),
            criteria: BadgeCriteria {
                criteria_type: CriteriaType::PostsCreated,
                threshold: 1,
            },
            points: 10,
            secret: false,
        });

        self.register(Badge {
            id: "prolific_writer".to_string(),
            name: "Prolific Writer".to_string(),
            description: "Published 50 posts".to_string(),
            icon: "feather".to_string(),
            tier: BadgeTier::Gold,
            category: "content".to_string(),
            criteria: BadgeCriteria {
                criteria_type: CriteriaType::PostsCreated,
                threshold: 50,
            },
            points: 100,
            secret: false,
        });

        // Social badges
        self.register(Badge {
            id: "first_follower".to_string(),
            name: "Getting Noticed".to_string(),
            description: "Gained your first follower".to_string(),
            icon: "users".to_string(),
            tier: BadgeTier::Bronze,
            category: "social".to_string(),
            criteria: BadgeCriteria {
                criteria_type: CriteriaType::FollowersGained,
                threshold: 1,
            },
            points: 10,
            secret: false,
        });

        self.register(Badge {
            id: "popular".to_string(),
            name: "Popular".to_string(),
            description: "Gained 100 followers".to_string(),
            icon: "star".to_string(),
            tier: BadgeTier::Gold,
            category: "social".to_string(),
            criteria: BadgeCriteria {
                criteria_type: CriteriaType::FollowersGained,
                threshold: 100,
            },
            points: 100,
            secret: false,
        });

        // Activity badges
        self.register(Badge {
            id: "week_streak".to_string(),
            name: "Week Streak".to_string(),
            description: "Active for 7 consecutive days".to_string(),
            icon: "zap".to_string(),
            tier: BadgeTier::Bronze,
            category: "activity".to_string(),
            criteria: BadgeCriteria {
                criteria_type: CriteriaType::DaysActive,
                threshold: 7,
            },
            points: 20,
            secret: false,
        });
    }

    pub fn register(&mut self, badge: Badge) {
        self.badges.insert(badge.id.clone(), badge);
    }

    pub fn get_badge(&self, id: &str) -> Option<&Badge> {
        self.badges.get(id)
    }

    pub fn get_all_badges(&self) -> Vec<&Badge> {
        self.badges.values().collect()
    }

    pub fn get_earned(&self, user_id: i64) -> Vec<&EarnedBadge> {
        self.earned.get(&user_id)
            .map(|badges| badges.iter().collect())
            .unwrap_or_default()
    }

    pub fn has_badge(&self, user_id: i64, badge_id: &str) -> bool {
        self.earned.get(&user_id)
            .map(|badges| badges.iter().any(|b| b.badge_id == badge_id))
            .unwrap_or(false)
    }

    pub fn update_progress(&mut self, user_id: i64, criteria_type: CriteriaType, value: i64) {
        let progress = self.user_progress.entry(user_id)
            .or_insert_with(HashMap::new);
        progress.insert(criteria_type, value);
    }

    pub fn check_and_award(&mut self, user_id: i64) -> Vec<&Badge> {
        let mut awarded = Vec::new();
        let progress = self.user_progress.get(&user_id).cloned().unwrap_or_default();

        for badge in self.badges.values() {
            // Skip if already earned
            if self.has_badge(user_id, &badge.id) {
                continue;
            }

            // Check criteria
            let current = progress.get(&badge.criteria.criteria_type).copied().unwrap_or(0);
            if current >= badge.criteria.threshold {
                // Award badge
                self.earned.entry(user_id)
                    .or_insert_with(Vec::new)
                    .push(EarnedBadge {
                        badge_id: badge.id.clone(),
                        user_id,
                        earned_at: Utc::now(),
                        progress_when_earned: current,
                    });

                awarded.push(&badge.id);
            }
        }

        // Return awarded badges
        awarded.iter()
            .filter_map(|id| self.badges.get(*id))
            .collect()
    }

    pub fn get_progress(&self, user_id: i64, badge_id: &str) -> Option<(i64, i64)> {
        let badge = self.badges.get(badge_id)?;
        let progress = self.user_progress.get(&user_id)?;
        let current = progress.get(&badge.criteria.criteria_type).copied().unwrap_or(0);
        Some((current, badge.criteria.threshold))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_follow() {
        let mut manager = FollowManager::new();

        manager.follow(1, 2).unwrap();
        assert!(manager.is_following(1, 2));
        assert!(!manager.is_following(2, 1));

        assert_eq!(manager.follower_count(2), 1);
        assert_eq!(manager.following_count(1), 1);
    }

    #[test]
    fn test_unfollow() {
        let mut manager = FollowManager::new();

        manager.follow(1, 2).unwrap();
        assert!(manager.unfollow(1, 2));
        assert!(!manager.is_following(1, 2));
    }

    #[test]
    fn test_messaging() {
        let mut inbox = InboxManager::new();

        inbox.start_thread(1, 2, "Hello!").unwrap();
        let threads = inbox.get_threads(1);
        assert_eq!(threads.len(), 1);

        assert_eq!(inbox.get_unread_count(2), 1);
    }

    #[test]
    fn test_reputation() {
        let mut rep = UserReputation::new(1);
        rep.add_points(ReputationAction::PostCreated, 5, "Created a post");
        rep.add_points(ReputationAction::PostLiked, 10, "Post was liked");

        assert_eq!(rep.total_points, 15);
        assert_eq!(rep.level, ReputationLevel::Newcomer);
    }

    #[test]
    fn test_reputation_levels() {
        assert_eq!(ReputationLevel::from_points(0), ReputationLevel::Newcomer);
        assert_eq!(ReputationLevel::from_points(100), ReputationLevel::Contributor);
        assert_eq!(ReputationLevel::from_points(10000), ReputationLevel::Legend);
    }

    #[test]
    fn test_badges() {
        let mut manager = BadgeManager::new();

        manager.update_progress(1, CriteriaType::PostsCreated, 1);
        let awarded = manager.check_and_award(1);

        assert!(!awarded.is_empty());
        assert!(manager.has_badge(1, "first_post"));
    }
}
