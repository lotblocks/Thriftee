use serde::{Deserialize, Serialize};
use std::fmt;

// User-related enums
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    User,
    Seller,
    Admin,
    Operator,
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UserRole::User => write!(f, "user"),
            UserRole::Seller => write!(f, "seller"),
            UserRole::Admin => write!(f, "admin"),
            UserRole::Operator => write!(f, "operator"),
        }
    }
}

// Item-related enums
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "item_status", rename_all = "lowercase")]
pub enum ItemStatus {
    Available,
    Sold,
    Inactive,
}

impl fmt::Display for ItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemStatus::Available => write!(f, "available"),
            ItemStatus::Sold => write!(f, "sold"),
            ItemStatus::Inactive => write!(f, "inactive"),
        }
    }
}

// Raffle-related enums
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "raffle_status", rename_all = "lowercase")]
pub enum RaffleStatus {
    Open,
    Full,
    Drawing,
    Completed,
    Cancelled,
}

impl fmt::Display for RaffleStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RaffleStatus::Open => write!(f, "open"),
            RaffleStatus::Full => write!(f, "full"),
            RaffleStatus::Drawing => write!(f, "drawing"),
            RaffleStatus::Completed => write!(f, "completed"),
            RaffleStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

// Credit-related enums
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "credit_source", rename_all = "snake_case")]
pub enum CreditSource {
    RaffleLoss,
    Deposit,
    Refund,
    Bonus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "credit_type", rename_all = "snake_case")]
pub enum CreditType {
    General,
    ItemSpecific,
}

// Transaction-related enums
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_type", rename_all = "snake_case")]
pub enum TransactionType {
    CreditDeposit,
    CreditWithdrawal,
    BoxPurchaseCreditDeduction,
    ItemPurchaseCreditDeduction,
    RaffleWinCreditAddition,
    Payout,
    FreeItemRedemption,
    SellerSubscriptionFee,
    SellerListingFee,
    SellerTransactionFee,
}

// Audit-related enums
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditAction {
    Create,
    Update,
    Delete,
    Login,
    Logout,
    PasswordChange,
    EmailChange,
    RoleChange,
    Payment,
    Refund,
    Withdrawal,
    CreditIssue,
    RaffleCreate,
    RaffleComplete,
    BoxPurchase,
    AdminAction,
    SecurityEvent,
}

// Common status enum for various entities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Pending => write!(f, "pending"),
            Status::Processing => write!(f, "processing"),
            Status::Completed => write!(f, "completed"),
            Status::Failed => write!(f, "failed"),
            Status::Cancelled => write!(f, "cancelled"),
        }
    }
}