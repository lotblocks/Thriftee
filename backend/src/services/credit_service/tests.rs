#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::user::User;
    use crate::models::item::Item;
    use chrono::Duration;
    use raffle_platform_shared::{ItemStatus, UserRole};
    use sqlx::PgPool;
    use tokio_test;

    async fn setup_test_data(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
        // Create test user
        let user = User::create(
            pool,
            "test@example.com",
            "password123",
            UserRole::User,
        ).await.unwrap();

        // Create test seller
        let seller = User::create(
            pool,
            "seller@example.com",
            "password123",
            UserRole::Seller,
        ).await.unwrap();

        // Create test item
        let item = Item::create(
            pool,
            seller.id,
            "Test Item",
            "A test item for credit redemption",
            Decimal::from(50),
            "electronics",
            vec!["https://example.com/image.jpg".to_string()],
        ).await.unwrap();

        (user.id, seller.id, item.id)
    }

    #[tokio::test]
    async fn test_issue_credits() {
        let pool = setup_test_pool().await;
        let credit_service = CreditService::new(pool.clone());
        let (user_id, _, _) = setup_test_data(&pool).await;

        let request = CreditIssuanceRequest {
            user_id,
            amount: Decimal::from(100),
            source: CreditSource::Bonus,
            credit_type: CreditType::General,
            redeemable_on_item_id: None,
            expires_at: Some(Utc::now() + Duration::days(30)),
            description: "Test bonus credits".to_string(),
        };

        let credit = credit_service.issue_credits(request).await.unwrap();

        assert_eq!(credit.user_id, user_id);
        assert_eq!(credit.amount, Decimal::from(100));
        assert_eq!(credit.source, CreditSource::Bonus);
        assert_eq!(credit.credit_type, CreditType::General);
        assert!(!credit.is_used);
    }

    #[tokio::test]
    async fn test_get_user_balance() {
        let pool = setup_test_pool().await;
        let credit_service = CreditService::new(pool.clone());
        let (user_id, _, _) = setup_test_data(&pool).await;

        // Issue some credits
        let request1 = CreditIssuanceRequest {
            user_id,
            amount: Decimal::from(50),
            source: CreditSource::Bonus,
            credit_type: CreditType::General,
            redeemable_on_item_id: None,
            expires_at: None,
            description: "General credits".to_string(),
        };

        let request2 = CreditIssuanceRequest {
            user_id,
            amount: Decimal::from(30),
            source: CreditSource::RaffleLoss,
            credit_type: CreditType::ItemSpecific,
            redeemable_on_item_id: None,
            expires_at: Some(Utc::now() + Duration::days(7)), // Expiring soon
            description: "Item-specific credits".to_string(),
        };

        credit_service.issue_credits(request1).await.unwrap();
        credit_service.issue_credits(request2).await.unwrap();

        let balance = credit_service.get_user_balance(user_id).await.unwrap();

        assert_eq!(balance.total_general, Decimal::from(50));
        assert_eq!(balance.total_item_specific, Decimal::from(30));
        assert_eq!(balance.total_available, Decimal::from(80));
        assert_eq!(balance.expiring_soon, Decimal::from(30));
    }

    #[tokio::test]
    async fn test_redeem_credits() {
        let pool = setup_test_pool().await;
        let credit_service = CreditService::new(pool.clone());
        let (user_id, _, item_id) = setup_test_data(&pool).await;

        // Issue credits
        let request = CreditIssuanceRequest {
            user_id,
            amount: Decimal::from(100),
            source: CreditSource::Bonus,
            credit_type: CreditType::General,
            redeemable_on_item_id: None,
            expires_at: None,
            description: "Test credits".to_string(),
        };

        credit_service.issue_credits(request).await.unwrap();

        // Redeem some credits
        let redemption_request = CreditRedemptionRequest {
            user_id,
            amount: Decimal::from(30),
            item_id: Some(item_id),
            credit_type: None,
            description: "Test redemption".to_string(),
        };

        let result = credit_service.redeem_credits(redemption_request).await.unwrap();

        assert_eq!(result.total_amount_used, Decimal::from(30));
        assert_eq!(result.remaining_balance, Decimal::from(70));
        assert_eq!(result.used_credits.len(), 1);
    }

    #[tokio::test]
    async fn test_insufficient_credits() {
        let pool = setup_test_pool().await;
        let credit_service = CreditService::new(pool.clone());
        let (user_id, _, item_id) = setup_test_data(&pool).await;

        // Issue small amount of credits
        let request = CreditIssuanceRequest {
            user_id,
            amount: Decimal::from(10),
            source: CreditSource::Bonus,
            credit_type: CreditType::General,
            redeemable_on_item_id: None,
            expires_at: None,
            description: "Small credits".to_string(),
        };

        credit_service.issue_credits(request).await.unwrap();

        // Try to redeem more than available
        let redemption_request = CreditRedemptionRequest {
            user_id,
            amount: Decimal::from(50),
            item_id: Some(item_id),
            credit_type: None,
            description: "Test redemption".to_string(),
        };

        let result = credit_service.redeem_credits(redemption_request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_expiring_credits() {
        let pool = setup_test_pool().await;
        let credit_service = CreditService::new(pool.clone());
        let (user_id, _, _) = setup_test_data(&pool).await;

        // Issue credits expiring in 5 days
        let request = CreditIssuanceRequest {
            user_id,
            amount: Decimal::from(25),
            source: CreditSource::RaffleLoss,
            credit_type: CreditType::General,
            redeemable_on_item_id: None,
            expires_at: Some(Utc::now() + Duration::days(5)),
            description: "Expiring credits".to_string(),
        };

        credit_service.issue_credits(request).await.unwrap();

        // Get expiring credits within 7 days
        let expiring = credit_service.get_expiring_credits(user_id, 7).await.unwrap();
        assert_eq!(expiring.len(), 1);
        assert_eq!(expiring[0].amount, Decimal::from(25));

        // Get expiring credits within 3 days (should be empty)
        let expiring_3_days = credit_service.get_expiring_credits(user_id, 3).await.unwrap();
        assert_eq!(expiring_3_days.len(), 0);
    }

    #[tokio::test]
    async fn test_check_sufficient_credits() {
        let pool = setup_test_pool().await;
        let credit_service = CreditService::new(pool.clone());
        let (user_id, _, item_id) = setup_test_data(&pool).await;

        // Issue credits
        let request = CreditIssuanceRequest {
            user_id,
            amount: Decimal::from(75),
            source: CreditSource::Bonus,
            credit_type: CreditType::General,
            redeemable_on_item_id: None,
            expires_at: None,
            description: "Test credits".to_string(),
        };

        credit_service.issue_credits(request).await.unwrap();

        // Check sufficient credits
        let has_sufficient = credit_service
            .check_sufficient_credits(user_id, Decimal::from(50), Some(item_id), None)
            .await
            .unwrap();
        assert!(has_sufficient);

        // Check insufficient credits
        let has_insufficient = credit_service
            .check_sufficient_credits(user_id, Decimal::from(100), Some(item_id), None)
            .await
            .unwrap();
        assert!(!has_insufficient);
    }

    #[tokio::test]
    async fn test_credit_statistics() {
        let pool = setup_test_pool().await;
        let credit_service = CreditService::new(pool.clone());
        let (user_id, _, _) = setup_test_data(&pool).await;

        // Issue various types of credits
        let requests = vec![
            CreditIssuanceRequest {
                user_id,
                amount: Decimal::from(50),
                source: CreditSource::Bonus,
                credit_type: CreditType::General,
                redeemable_on_item_id: None,
                expires_at: None,
                description: "Bonus credits".to_string(),
            },
            CreditIssuanceRequest {
                user_id,
                amount: Decimal::from(30),
                source: CreditSource::RaffleLoss,
                credit_type: CreditType::ItemSpecific,
                redeemable_on_item_id: None,
                expires_at: None,
                description: "Raffle loss credits".to_string(),
            },
        ];

        for request in requests {
            credit_service.issue_credits(request).await.unwrap();
        }

        let stats = credit_service.get_credit_statistics().await.unwrap();

        assert_eq!(stats.total_credits_issued, Decimal::from(80));
        assert_eq!(stats.active_users_with_credits, 1);
        assert!(stats.credits_by_source.contains_key(&CreditSource::Bonus));
        assert!(stats.credits_by_source.contains_key(&CreditSource::RaffleLoss));
        assert!(stats.credits_by_type.contains_key(&CreditType::General));
        assert!(stats.credits_by_type.contains_key(&CreditType::ItemSpecific));
    }

    #[tokio::test]
    async fn test_partial_credit_usage() {
        let pool = setup_test_pool().await;
        let credit_service = CreditService::new(pool.clone());
        let (user_id, _, item_id) = setup_test_data(&pool).await;

        // Issue a single large credit
        let request = CreditIssuanceRequest {
            user_id,
            amount: Decimal::from(100),
            source: CreditSource::Bonus,
            credit_type: CreditType::General,
            redeemable_on_item_id: None,
            expires_at: None,
            description: "Large credit".to_string(),
        };

        credit_service.issue_credits(request).await.unwrap();

        // Redeem part of the credit
        let redemption_request = CreditRedemptionRequest {
            user_id,
            amount: Decimal::from(30),
            item_id: Some(item_id),
            credit_type: None,
            description: "Partial redemption".to_string(),
        };

        let result = credit_service.redeem_credits(redemption_request).await.unwrap();

        assert_eq!(result.total_amount_used, Decimal::from(30));
        assert_eq!(result.remaining_balance, Decimal::from(70));

        // Check that a new credit was created for the remaining amount
        let balance = credit_service.get_user_balance(user_id).await.unwrap();
        assert_eq!(balance.total_available, Decimal::from(70));
    }

    // Helper function to set up test database pool
    async fn setup_test_pool() -> PgPool {
        // This would typically connect to a test database
        // For now, we'll assume the pool is properly configured
        todo!("Set up test database connection")
    }
}