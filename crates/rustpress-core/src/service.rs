//! Service layer traits and abstractions.
//!
//! Defines the contract for business logic services, separate from handlers.

use crate::context::RequestContext;
use crate::error::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Base trait for all services
#[async_trait]
pub trait Service: Send + Sync {
    /// Service name for logging and identification
    fn name(&self) -> &str;

    /// Initialize the service
    async fn init(&self) -> Result<()> {
        Ok(())
    }

    /// Shutdown the service gracefully
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    /// Health check for this service
    async fn health_check(&self) -> Result<ServiceHealth> {
        Ok(ServiceHealth::healthy(self.name()))
    }
}

/// Health status of a service
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
}

impl ServiceHealth {
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
            message: None,
            latency_ms: None,
        }
    }

    pub fn unhealthy(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            latency_ms: None,
        }
    }

    pub fn degraded(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            latency_ms: None,
        }
    }

    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }
}

/// Health status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// CRUD service trait for entities
#[async_trait]
pub trait CrudService<T, CreateDto, UpdateDto>: Service
where
    T: Send + Sync,
    CreateDto: Send + Sync,
    UpdateDto: Send + Sync,
{
    /// Create a new entity
    async fn create(&self, ctx: &RequestContext, dto: CreateDto) -> Result<T>;

    /// Get an entity by ID
    async fn get_by_id(&self, ctx: &RequestContext, id: &str) -> Result<Option<T>>;

    /// Update an existing entity
    async fn update(&self, ctx: &RequestContext, id: &str, dto: UpdateDto) -> Result<T>;

    /// Delete an entity
    async fn delete(&self, ctx: &RequestContext, id: &str) -> Result<()>;

    /// List entities with pagination
    async fn list(&self, ctx: &RequestContext, params: ListParams) -> Result<ListResult<T>>;
}

/// Parameters for listing entities
#[derive(Debug, Clone, Default)]
pub struct ListParams {
    pub page: u32,
    pub per_page: u32,
    pub sort_by: Option<String>,
    pub sort_order: SortOrder,
    pub filters: Vec<Filter>,
    pub search: Option<String>,
}

impl ListParams {
    pub fn new() -> Self {
        Self {
            page: 1,
            per_page: 20,
            sort_by: None,
            sort_order: SortOrder::Desc,
            filters: Vec::new(),
            search: None,
        }
    }

    pub fn page(mut self, page: u32) -> Self {
        self.page = page;
        self
    }

    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = per_page;
        self
    }

    pub fn sort_by(mut self, field: impl Into<String>) -> Self {
        self.sort_by = Some(field.into());
        self
    }

    pub fn sort_order(mut self, order: SortOrder) -> Self {
        self.sort_order = order;
        self
    }

    pub fn filter(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    pub fn search(mut self, query: impl Into<String>) -> Self {
        self.search = Some(query.into());
        self
    }

    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.per_page
    }
}

/// Sort order
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

/// Filter for queries
#[derive(Debug, Clone)]
pub struct Filter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: FilterValue,
}

impl Filter {
    pub fn eq(field: impl Into<String>, value: impl Into<FilterValue>) -> Self {
        Self {
            field: field.into(),
            operator: FilterOperator::Equals,
            value: value.into(),
        }
    }

    pub fn ne(field: impl Into<String>, value: impl Into<FilterValue>) -> Self {
        Self {
            field: field.into(),
            operator: FilterOperator::NotEquals,
            value: value.into(),
        }
    }

    pub fn gt(field: impl Into<String>, value: impl Into<FilterValue>) -> Self {
        Self {
            field: field.into(),
            operator: FilterOperator::GreaterThan,
            value: value.into(),
        }
    }

    pub fn lt(field: impl Into<String>, value: impl Into<FilterValue>) -> Self {
        Self {
            field: field.into(),
            operator: FilterOperator::LessThan,
            value: value.into(),
        }
    }

    pub fn contains(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            operator: FilterOperator::Contains,
            value: FilterValue::String(value.into()),
        }
    }

    pub fn is_in(field: impl Into<String>, values: Vec<FilterValue>) -> Self {
        Self {
            field: field.into(),
            operator: FilterOperator::In,
            value: FilterValue::Array(values),
        }
    }
}

/// Filter operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Contains,
    StartsWith,
    EndsWith,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

/// Filter values
#[derive(Debug, Clone)]
pub enum FilterValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<FilterValue>),
    Null,
}

impl From<String> for FilterValue {
    fn from(s: String) -> Self {
        FilterValue::String(s)
    }
}

impl From<&str> for FilterValue {
    fn from(s: &str) -> Self {
        FilterValue::String(s.to_string())
    }
}

impl From<i64> for FilterValue {
    fn from(n: i64) -> Self {
        FilterValue::Integer(n)
    }
}

impl From<i32> for FilterValue {
    fn from(n: i32) -> Self {
        FilterValue::Integer(n as i64)
    }
}

impl From<f64> for FilterValue {
    fn from(n: f64) -> Self {
        FilterValue::Float(n)
    }
}

impl From<bool> for FilterValue {
    fn from(b: bool) -> Self {
        FilterValue::Boolean(b)
    }
}

/// Result of a list operation
#[derive(Debug, Clone)]
pub struct ListResult<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

impl<T> ListResult<T> {
    pub fn new(items: Vec<T>, total: u64, params: &ListParams) -> Self {
        let total_pages = (total as f64 / params.per_page as f64).ceil() as u32;
        Self {
            items,
            total,
            page: params.page,
            per_page: params.per_page,
            total_pages,
        }
    }

    pub fn empty(params: &ListParams) -> Self {
        Self {
            items: Vec::new(),
            total: 0,
            page: params.page,
            per_page: params.per_page,
            total_pages: 0,
        }
    }

    pub fn has_next(&self) -> bool {
        self.page < self.total_pages
    }

    pub fn has_prev(&self) -> bool {
        self.page > 1
    }

    pub fn map<U, F: FnMut(T) -> U>(self, f: F) -> ListResult<U> {
        ListResult {
            items: self.items.into_iter().map(f).collect(),
            total: self.total,
            page: self.page,
            per_page: self.per_page,
            total_pages: self.total_pages,
        }
    }
}

/// Service container for dependency injection
pub struct ServiceContainer {
    services: std::collections::HashMap<std::any::TypeId, Arc<dyn std::any::Any + Send + Sync>>,
}

impl ServiceContainer {
    pub fn new() -> Self {
        Self {
            services: std::collections::HashMap::new(),
        }
    }

    /// Register a service
    pub fn register<T: Send + Sync + 'static>(&mut self, service: Arc<T>) {
        self.services
            .insert(std::any::TypeId::of::<T>(), service);
    }

    /// Get a service
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.services
            .get(&std::any::TypeId::of::<T>())
            .and_then(|s| s.clone().downcast::<T>().ok())
    }

    /// Get a service, panicking if not found
    pub fn require<T: Send + Sync + 'static>(&self) -> Arc<T> {
        self.get::<T>()
            .expect(&format!("Service {} not registered", std::any::type_name::<T>()))
    }
}

impl Default for ServiceContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_params() {
        let params = ListParams::new()
            .page(2)
            .per_page(10)
            .sort_by("created_at")
            .sort_order(SortOrder::Asc);

        assert_eq!(params.page, 2);
        assert_eq!(params.per_page, 10);
        assert_eq!(params.offset(), 10);
    }

    #[test]
    fn test_list_result() {
        let params = ListParams::new().page(1).per_page(10);
        let result: ListResult<i32> = ListResult::new(vec![1, 2, 3], 25, &params);

        assert_eq!(result.total_pages, 3);
        assert!(result.has_next());
        assert!(!result.has_prev());
    }

    #[test]
    fn test_service_container() {
        struct TestService {
            value: i32,
        }

        let mut container = ServiceContainer::new();
        container.register(Arc::new(TestService { value: 42 }));

        let service = container.get::<TestService>().unwrap();
        assert_eq!(service.value, 42);
    }

    #[test]
    fn test_filter() {
        let filter = Filter::eq("status", "published");
        assert_eq!(filter.field, "status");
        assert_eq!(filter.operator, FilterOperator::Equals);
    }
}
