//! Repository pattern implementation for database operations
//! 
//! This module provides a higher-level abstraction over the database models,
//! implementing the repository pattern for better separation of concerns
//! and easier testing.

use sqlx::PgPool;
use std::sync::Arc;

pub mod user_repository;
pub mod item_repository;
pub mod raffle_repository;
pub mod credit_repository;
pub mod transaction_repository;

pub use user_repository::UserRepository;
pub use item_repository::ItemRepository;
pub use raffle_repository::RaffleRepository;
pub use credit_repository::CreditRepository;
pub use transaction_repository::TransactionRepository;

/// Repository container that holds all repositories
#[derive(Clone)]
pub struct Repositories {
    pub users: UserRepository,
    pub items: ItemRepository,
    pub raffles: RaffleRepository,
    pub credits: CreditRepository,
    pub transactions: TransactionRepository,
}

impl Repositories {
    /// Create a new repository container
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            users: UserRepository::new(pool.clone()),
            items: ItemRepository::new(pool.clone()),
            raffles: RaffleRepository::new(pool.clone()),
            credits: CreditRepository::new(pool.clone()),
            transactions: TransactionRepository::new(pool),
        }
    }
}

/// Base repository trait that all repositories should implement
pub trait Repository {
    type Entity;
    type Id;
    type CreateRequest;
    type UpdateRequest;

    /// Find entity by ID
    async fn find_by_id(&self, id: Self::Id) -> Result<Option<Self::Entity>, crate::error::AppError>;

    /// Create a new entity
    async fn create(&self, request: Self::CreateRequest) -> Result<Self::Entity, crate::error::AppError>;

    /// Update an existing entity
    async fn update(&self, id: Self::Id, request: Self::UpdateRequest) -> Result<Self::Entity, crate::error::AppError>;

    /// Delete an entity
    async fn delete(&self, id: Self::Id) -> Result<bool, crate::error::AppError>;
}

/// Pagination parameters
#[derive(Debug, Clone)]
pub struct PaginationParams {
    pub limit: i64,
    pub offset: i64,
}

impl PaginationParams {
    pub fn new(limit: Option<i64>, offset: Option<i64>) -> Self {
        Self {
            limit: limit.unwrap_or(20).min(100).max(1),
            offset: offset.unwrap_or(0).max(0),
        }
    }

    pub fn from_page(page: i64, per_page: i64) -> Self {
        let per_page = per_page.min(100).max(1);
        let page = page.max(1);
        Self {
            limit: per_page,
            offset: (page - 1) * per_page,
        }
    }
}

/// Paginated result wrapper
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

impl<T> PaginatedResult<T> {
    pub fn new(data: Vec<T>, total: i64, limit: i64, offset: i64) -> Self {
        Self {
            data,
            total,
            limit,
            offset,
        }
    }

    pub fn has_more(&self) -> bool {
        self.offset + self.limit < self.total
    }

    pub fn current_page(&self) -> i64 {
        (self.offset / self.limit) + 1
    }

    pub fn total_pages(&self) -> i64 {
        (self.total + self.limit - 1) / self.limit
    }
}