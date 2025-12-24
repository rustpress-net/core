//! Repository pattern for database abstraction.
//!
//! Provides a clean abstraction over database operations.

use crate::error::Result;
use crate::service::{ListParams, ListResult};
use async_trait::async_trait;
use std::fmt::Debug;

/// Base repository trait for CRUD operations
#[async_trait]
pub trait Repository<T, Id>: Send + Sync
where
    T: Send + Sync,
    Id: Send + Sync,
{
    /// Create a new entity
    async fn create(&self, entity: T) -> Result<T>;

    /// Find an entity by ID
    async fn find_by_id(&self, id: &Id) -> Result<Option<T>>;

    /// Update an existing entity
    async fn update(&self, entity: T) -> Result<T>;

    /// Delete an entity by ID
    async fn delete(&self, id: &Id) -> Result<()>;

    /// Check if an entity exists
    async fn exists(&self, id: &Id) -> Result<bool>;

    /// Count total entities
    async fn count(&self) -> Result<u64>;
}

/// Extended repository with listing and querying capabilities
#[async_trait]
pub trait QueryableRepository<T, Id>: Repository<T, Id>
where
    T: Send + Sync,
    Id: Send + Sync,
{
    /// List entities with pagination
    async fn list(&self, params: &ListParams) -> Result<ListResult<T>>;

    /// Find entities by a specific field
    async fn find_by_field(&self, field: &str, value: &str) -> Result<Vec<T>>;

    /// Find a single entity by a unique field
    async fn find_one_by_field(&self, field: &str, value: &str) -> Result<Option<T>>;

    /// Search entities
    async fn search(&self, query: &str, params: &ListParams) -> Result<ListResult<T>>;
}

/// Repository with tenant support for multi-tenancy
#[async_trait]
pub trait TenantRepository<T, Id>: Repository<T, Id>
where
    T: Send + Sync,
    Id: Send + Sync,
{
    /// Get tenant ID for scoping queries
    fn tenant_id(&self) -> Option<&crate::id::TenantId>;

    /// Create a new repository instance scoped to a tenant
    fn with_tenant(&self, tenant_id: crate::id::TenantId) -> Box<dyn TenantRepository<T, Id>>;
}

/// Soft-delete repository trait
#[async_trait]
pub trait SoftDeleteRepository<T, Id>: Repository<T, Id>
where
    T: Send + Sync,
    Id: Send + Sync,
{
    /// Soft delete an entity
    async fn soft_delete(&self, id: &Id) -> Result<()>;

    /// Restore a soft-deleted entity
    async fn restore(&self, id: &Id) -> Result<()>;

    /// Find including soft-deleted entities
    async fn find_with_deleted(&self, id: &Id) -> Result<Option<T>>;

    /// Find only soft-deleted entities
    async fn find_deleted(&self, params: &ListParams) -> Result<ListResult<T>>;

    /// Permanently delete (hard delete)
    async fn hard_delete(&self, id: &Id) -> Result<()>;
}

/// Transaction support trait
#[async_trait]
pub trait Transactional: Send + Sync {
    /// Execute operations within a transaction
    async fn transaction<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<R>> + Send,
        R: Send;
}

/// Unit of Work pattern for coordinating multiple repositories
#[async_trait]
pub trait UnitOfWork: Send + Sync {
    /// Begin a unit of work
    async fn begin(&self) -> Result<()>;

    /// Commit all changes
    async fn commit(&self) -> Result<()>;

    /// Rollback all changes
    async fn rollback(&self) -> Result<()>;
}

/// Specification pattern for complex queries
pub trait Specification<T>: Send + Sync {
    /// Check if an entity satisfies this specification
    fn is_satisfied_by(&self, entity: &T) -> bool;

    /// Combine with another specification using AND
    fn and<S: Specification<T>>(self, other: S) -> AndSpecification<T, Self, S>
    where
        Self: Sized,
    {
        AndSpecification {
            left: self,
            right: other,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Combine with another specification using OR
    fn or<S: Specification<T>>(self, other: S) -> OrSpecification<T, Self, S>
    where
        Self: Sized,
    {
        OrSpecification {
            left: self,
            right: other,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Negate this specification
    fn not(self) -> NotSpecification<T, Self>
    where
        Self: Sized,
    {
        NotSpecification {
            spec: self,
            _phantom: std::marker::PhantomData,
        }
    }
}

/// AND combination of specifications
pub struct AndSpecification<T, L, R>
where
    L: Specification<T>,
    R: Specification<T>,
{
    left: L,
    right: R,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, L, R> Specification<T> for AndSpecification<T, L, R>
where
    T: Send + Sync,
    L: Specification<T>,
    R: Specification<T>,
{
    fn is_satisfied_by(&self, entity: &T) -> bool {
        self.left.is_satisfied_by(entity) && self.right.is_satisfied_by(entity)
    }
}

/// OR combination of specifications
pub struct OrSpecification<T, L, R>
where
    L: Specification<T>,
    R: Specification<T>,
{
    left: L,
    right: R,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, L, R> Specification<T> for OrSpecification<T, L, R>
where
    T: Send + Sync,
    L: Specification<T>,
    R: Specification<T>,
{
    fn is_satisfied_by(&self, entity: &T) -> bool {
        self.left.is_satisfied_by(entity) || self.right.is_satisfied_by(entity)
    }
}

/// NOT specification
pub struct NotSpecification<T, S>
where
    S: Specification<T>,
{
    spec: S,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, S> Specification<T> for NotSpecification<T, S>
where
    T: Send + Sync,
    S: Specification<T>,
{
    fn is_satisfied_by(&self, entity: &T) -> bool {
        !self.spec.is_satisfied_by(entity)
    }
}

/// Repository that supports specifications
#[async_trait]
pub trait SpecificationRepository<T, Id>: Repository<T, Id>
where
    T: Send + Sync + 'static,
    Id: Send + Sync,
{
    /// Find entities matching a specification
    async fn find_by_spec(&self, spec: &dyn Specification<T>) -> Result<Vec<T>>;

    /// Count entities matching a specification
    async fn count_by_spec(&self, spec: &dyn Specification<T>) -> Result<u64>;
}

/// Builder for repository queries
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    table: String,
    select: Vec<String>,
    conditions: Vec<String>,
    params: Vec<String>,
    order_by: Option<(String, String)>,
    limit: Option<u32>,
    offset: Option<u32>,
}

impl QueryBuilder {
    pub fn new(table: impl Into<String>) -> Self {
        Self {
            table: table.into(),
            select: Vec::new(),
            conditions: Vec::new(),
            params: Vec::new(),
            order_by: None,
            limit: None,
            offset: None,
        }
    }

    pub fn select(mut self, columns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.select = columns.into_iter().map(|c| c.into()).collect();
        self
    }

    pub fn where_eq(mut self, field: &str, param: &str) -> Self {
        self.conditions.push(format!("{} = ${}", field, self.params.len() + 1));
        self.params.push(param.to_string());
        self
    }

    pub fn where_in(mut self, field: &str, count: usize) -> Self {
        let placeholders: Vec<String> = (0..count)
            .map(|i| format!("${}", self.params.len() + i + 1))
            .collect();
        self.conditions.push(format!("{} IN ({})", field, placeholders.join(", ")));
        self
    }

    pub fn order_by(mut self, field: impl Into<String>, direction: &str) -> Self {
        self.order_by = Some((field.into(), direction.to_string()));
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn build_select(&self) -> String {
        let columns = if self.select.is_empty() {
            "*".to_string()
        } else {
            self.select.join(", ")
        };

        let mut query = format!("SELECT {} FROM {}", columns, self.table);

        if !self.conditions.is_empty() {
            query.push_str(&format!(" WHERE {}", self.conditions.join(" AND ")));
        }

        if let Some((field, dir)) = &self.order_by {
            query.push_str(&format!(" ORDER BY {} {}", field, dir));
        }

        if let Some(limit) = self.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            query.push_str(&format!(" OFFSET {}", offset));
        }

        query
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEntity {
        id: i32,
        active: bool,
    }

    struct IsActive;
    impl Specification<TestEntity> for IsActive {
        fn is_satisfied_by(&self, entity: &TestEntity) -> bool {
            entity.active
        }
    }

    struct HasIdGreaterThan(i32);
    impl Specification<TestEntity> for HasIdGreaterThan {
        fn is_satisfied_by(&self, entity: &TestEntity) -> bool {
            entity.id > self.0
        }
    }

    #[test]
    fn test_specification_and() {
        let spec = IsActive.and(HasIdGreaterThan(5));

        let entity1 = TestEntity { id: 10, active: true };
        let entity2 = TestEntity { id: 3, active: true };
        let entity3 = TestEntity { id: 10, active: false };

        assert!(spec.is_satisfied_by(&entity1));
        assert!(!spec.is_satisfied_by(&entity2));
        assert!(!spec.is_satisfied_by(&entity3));
    }

    #[test]
    fn test_specification_or() {
        let spec = IsActive.or(HasIdGreaterThan(5));

        let entity1 = TestEntity { id: 10, active: false };
        let entity2 = TestEntity { id: 3, active: true };
        let entity3 = TestEntity { id: 3, active: false };

        assert!(spec.is_satisfied_by(&entity1));
        assert!(spec.is_satisfied_by(&entity2));
        assert!(!spec.is_satisfied_by(&entity3));
    }

    #[test]
    fn test_specification_not() {
        let spec = IsActive.not();

        let entity1 = TestEntity { id: 1, active: true };
        let entity2 = TestEntity { id: 1, active: false };

        assert!(!spec.is_satisfied_by(&entity1));
        assert!(spec.is_satisfied_by(&entity2));
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new("posts")
            .select(["id", "title", "content"])
            .where_eq("status", "published")
            .order_by("created_at", "DESC")
            .limit(10)
            .offset(0)
            .build_select();

        assert!(query.contains("SELECT id, title, content FROM posts"));
        assert!(query.contains("WHERE status = $1"));
        assert!(query.contains("ORDER BY created_at DESC"));
        assert!(query.contains("LIMIT 10"));
    }
}
