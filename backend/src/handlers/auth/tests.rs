#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use serde_json::json;
    use sqlx::PgPool;
    use std::env;

    use crate::database::Database;
    use crate::utils::jwt::JwtService;

    async fn setup_test_app() -> (
        impl actix_web::dev::Service<
            actix_web::dev::ServiceRequest,
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
        >,
        PgPool,
        JwtService,
    ) {
        // Set up test database
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/raffle_platform_test".to_string());
        
        let database = Database::new(&database_url).await.expect("Failed to connect to test database");
        database.migrate().await.expect("Failed to run migrations");
        let pool = database.pool().clone();

        // Set up JWT service
        env::set_var("JWT_SECRET", "test-secret-key-for-testing-only");
        let jwt_service = JwtService::new().expect("Failed to create JWT service");

        // Create test app
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .app_data(web::Data::new(jwt_service.clone()))
                .service(
                    web::scope("/auth")
                        .service(register)
                        .service(login)
                        .service(refresh_token)
                        .service(forgot_password)
                        .service(reset_password)
                        .service(verify_email)
                        .service(logout)
                        .service(get_current_user)
                        .service(update_current_user)
                        .service(change_password)
                )
        ).await;

        (app, pool, jwt_service)
    }

    #[actix_web::test]
    async fn test_user_registration_success() {
        let (app, _pool, _jwt_service) = setup_test_app().await;

        let registration_data = json!({
            "username": "testuser123",
            "email": "test@example.com",
            "password": "SecurePass123!",
            "phone_number": "+1234567890"
        });

        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&registration_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["message"], SUCCESS_USER_CREATED);
        assert!(body["data"]["access_token"].is_string());
        assert!(body["data"]["refresh_token"].is_string());
        assert_eq!(body["data"]["user"]["username"], "testuser123");
        assert_eq!(body["data"]["user"]["email"], "test@example.com");
    }

    #[actix_web::test]
    async fn test_user_registration_duplicate_email() {
        let (app, pool, _jwt_service) = setup_test_app().await;

        // First registration
        let registration_data = json!({
            "username": "testuser1",
            "email": "duplicate@example.com",
            "password": "SecurePass123!"
        });

        let req1 = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&registration_data)
            .to_request();

        let resp1 = test::call_service(&app, req1).await;
        assert_eq!(resp1.status(), 201);

        // Second registration with same email
        let registration_data2 = json!({
            "username": "testuser2",
            "email": "duplicate@example.com",
            "password": "SecurePass123!"
        });

        let req2 = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&registration_data2)
            .to_request();

        let resp2 = test::call_service(&app, req2).await;
        assert_eq!(resp2.status(), 409);

        let body: serde_json::Value = test::read_body_json(resp2).await;
        assert_eq!(body["message"], ERROR_EMAIL_ALREADY_EXISTS);
    }

    #[actix_web::test]
    async fn test_user_registration_invalid_data() {
        let (app, _pool, _jwt_service) = setup_test_app().await;

        // Test invalid email
        let invalid_data = json!({
            "username": "testuser",
            "email": "invalid-email",
            "password": "SecurePass123!"
        });

        let req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&invalid_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        // Test weak password
        let weak_password_data = json!({
            "username": "testuser",
            "email": "test@example.com",
            "password": "weak"
        });

        let req2 = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&weak_password_data)
            .to_request();

        let resp2 = test::call_service(&app, req2).await;
        assert_eq!(resp2.status(), 400);
    }

    #[actix_web::test]
    async fn test_user_login_success() {
        let (app, _pool, _jwt_service) = setup_test_app().await;

        // First register a user
        let registration_data = json!({
            "username": "loginuser",
            "email": "login@example.com",
            "password": "SecurePass123!"
        });

        let register_req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&registration_data)
            .to_request();

        let register_resp = test::call_service(&app, register_req).await;
        assert_eq!(register_resp.status(), 201);

        // Now login
        let login_data = json!({
            "email": "login@example.com",
            "password": "SecurePass123!"
        });

        let login_req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&login_data)
            .to_request();

        let login_resp = test::call_service(&app, login_req).await;
        assert_eq!(login_resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(login_resp).await;
        assert_eq!(body["message"], SUCCESS_LOGIN);
        assert!(body["data"]["access_token"].is_string());
        assert!(body["data"]["refresh_token"].is_string());
        assert_eq!(body["data"]["user"]["email"], "login@example.com");
    }

    #[actix_web::test]
    async fn test_user_login_invalid_credentials() {
        let (app, _pool, _jwt_service) = setup_test_app().await;

        // Try to login with non-existent user
        let login_data = json!({
            "email": "nonexistent@example.com",
            "password": "SecurePass123!"
        });

        let req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&login_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["message"], ERROR_INVALID_CREDENTIALS);
    }

    #[actix_web::test]
    async fn test_token_refresh() {
        let (app, _pool, _jwt_service) = setup_test_app().await;

        // Register and login to get tokens
        let registration_data = json!({
            "username": "refreshuser",
            "email": "refresh@example.com",
            "password": "SecurePass123!"
        });

        let register_req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&registration_data)
            .to_request();

        let register_resp = test::call_service(&app, register_req).await;
        assert_eq!(register_resp.status(), 201);

        let register_body: serde_json::Value = test::read_body_json(register_resp).await;
        let refresh_token = register_body["data"]["refresh_token"].as_str().unwrap();

        // Use refresh token to get new tokens
        let refresh_data = json!({
            "refresh_token": refresh_token
        });

        let refresh_req = test::TestRequest::post()
            .uri("/auth/refresh")
            .set_json(&refresh_data)
            .to_request();

        let refresh_resp = test::call_service(&app, refresh_req).await;
        assert_eq!(refresh_resp.status(), 200);

        let refresh_body: serde_json::Value = test::read_body_json(refresh_resp).await;
        assert!(refresh_body["data"]["access_token"].is_string());
        assert!(refresh_body["data"]["refresh_token"].is_string());
        
        // New tokens should be different from original
        assert_ne!(refresh_body["data"]["refresh_token"], refresh_token);
    }

    #[actix_web::test]
    async fn test_forgot_password() {
        let (app, _pool, _jwt_service) = setup_test_app().await;

        // Register a user first
        let registration_data = json!({
            "username": "forgotuser",
            "email": "forgot@example.com",
            "password": "SecurePass123!"
        });

        let register_req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&registration_data)
            .to_request();

        let register_resp = test::call_service(&app, register_req).await;
        assert_eq!(register_resp.status(), 201);

        // Request password reset
        let forgot_data = json!({
            "email": "forgot@example.com"
        });

        let forgot_req = test::TestRequest::post()
            .uri("/auth/forgot-password")
            .set_json(&forgot_data)
            .to_request();

        let forgot_resp = test::call_service(&app, forgot_req).await;
        assert_eq!(forgot_resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(forgot_resp).await;
        assert_eq!(body["message"], SUCCESS_PASSWORD_RESET);
    }

    #[actix_web::test]
    async fn test_get_current_user() {
        let (app, _pool, jwt_service) = setup_test_app().await;

        // Register a user
        let registration_data = json!({
            "username": "currentuser",
            "email": "current@example.com",
            "password": "SecurePass123!"
        });

        let register_req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&registration_data)
            .to_request();

        let register_resp = test::call_service(&app, register_req).await;
        assert_eq!(register_resp.status(), 201);

        let register_body: serde_json::Value = test::read_body_json(register_resp).await;
        let access_token = register_body["data"]["access_token"].as_str().unwrap();

        // Get current user
        let me_req = test::TestRequest::get()
            .uri("/auth/me")
            .insert_header(("Authorization", format!("Bearer {}", access_token)))
            .to_request();

        let me_resp = test::call_service(&app, me_req).await;
        assert_eq!(me_resp.status(), 200);

        let me_body: serde_json::Value = test::read_body_json(me_resp).await;
        assert_eq!(me_body["data"]["username"], "currentuser");
        assert_eq!(me_body["data"]["email"], "current@example.com");
    }

    #[actix_web::test]
    async fn test_change_password() {
        let (app, _pool, _jwt_service) = setup_test_app().await;

        // Register a user
        let registration_data = json!({
            "username": "changepassuser",
            "email": "changepass@example.com",
            "password": "OldPass123!"
        });

        let register_req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&registration_data)
            .to_request();

        let register_resp = test::call_service(&app, register_req).await;
        assert_eq!(register_resp.status(), 201);

        let register_body: serde_json::Value = test::read_body_json(register_resp).await;
        let access_token = register_body["data"]["access_token"].as_str().unwrap();

        // Change password
        let change_data = json!({
            "current_password": "OldPass123!",
            "new_password": "NewPass123!"
        });

        let change_req = test::TestRequest::post()
            .uri("/auth/change-password")
            .insert_header(("Authorization", format!("Bearer {}", access_token)))
            .set_json(&change_data)
            .to_request();

        let change_resp = test::call_service(&app, change_req).await;
        assert_eq!(change_resp.status(), 200);

        // Try to login with new password
        let login_data = json!({
            "email": "changepass@example.com",
            "password": "NewPass123!"
        });

        let login_req = test::TestRequest::post()
            .uri("/auth/login")
            .set_json(&login_data)
            .to_request();

        let login_resp = test::call_service(&app, login_req).await;
        assert_eq!(login_resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_logout() {
        let (app, _pool, _jwt_service) = setup_test_app().await;

        // Register a user
        let registration_data = json!({
            "username": "logoutuser",
            "email": "logout@example.com",
            "password": "SecurePass123!"
        });

        let register_req = test::TestRequest::post()
            .uri("/auth/register")
            .set_json(&registration_data)
            .to_request();

        let register_resp = test::call_service(&app, register_req).await;
        assert_eq!(register_resp.status(), 201);

        let register_body: serde_json::Value = test::read_body_json(register_resp).await;
        let access_token = register_body["data"]["access_token"].as_str().unwrap();

        // Logout
        let logout_req = test::TestRequest::post()
            .uri("/auth/logout")
            .insert_header(("Authorization", format!("Bearer {}", access_token)))
            .to_request();

        let logout_resp = test::call_service(&app, logout_req).await;
        assert_eq!(logout_resp.status(), 200);

        let logout_body: serde_json::Value = test::read_body_json(logout_resp).await;
        assert_eq!(logout_body["message"], SUCCESS_LOGOUT);
    }

    #[actix_web::test]
    async fn test_unauthorized_access() {
        let (app, _pool, _jwt_service) = setup_test_app().await;

        // Try to access protected endpoint without token
        let req = test::TestRequest::get()
            .uri("/auth/me")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);

        // Try with invalid token
        let req2 = test::TestRequest::get()
            .uri("/auth/me")
            .insert_header(("Authorization", "Bearer invalid_token"))
            .to_request();

        let resp2 = test::call_service(&app, req2).await;
        assert_eq!(resp2.status(), 401);
    }
}
"