# Authentication API Documentation

## Overview

The Authentication API provides comprehensive user registration, login, and profile management functionality for the Raffle Shopping Platform. All endpoints follow RESTful conventions and return JSON responses.

## Base URL

```
http://localhost:8080/api/auth
```

## Authentication

Most endpoints require authentication via Bearer token in the Authorization header:

```
Authorization: Bearer <access_token>
```

## Rate Limiting

Authentication endpoints are rate-limited to 10 requests per minute per IP address.

## Endpoints

### Public Endpoints

#### POST /register

Register a new user account.

**Request Body:**
```json
{
  "username": "string (3-30 chars, alphanumeric + _ -)",
  "email": "string (valid email format)",
  "phone_number": "string (optional, E.164 format)"
}
```

**Response (201 Created):**
```json
{
  "user": {
    "id": "uuid",
    "username": "string",
    "email": "string",
    "role": "user",
    "credit_balance": "0.00",
    "internal_wallet_address": "0x...",
    "phone_number": "string|null",
    "is_active": true,
    "email_verified": false,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  },
  "tokens": {
    "access_token": "string",
    "refresh_token": "string",
    "expires_in": 3600,
    "token_type": "Bearer"
  },
  "message": "Registration successful. Please check your email to verify your account."
}
```

**Error Responses:**
- `400 Bad Request` - Validation errors
- `409 Conflict` - Username or email already exists
- `429 Too Many Requests` - Rate limit exceeded

#### POST /login

Authenticate a user and create a new session.

**Request Body:**
```json
{
  "email": "string",
  "password": "string"
}
```

**Response (200 OK):**
```json
{
  "user": {
    "id": "uuid",
    "username": "string",
    "email": "string",
    "role": "user|seller|admin",
    "credit_balance": "decimal",
    "internal_wallet_address": "0x...",
    "phone_number": "string|null",
    "is_active": true,
    "email_verified": boolean,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  },
  "tokens": {
    "access_token": "string",
    "refresh_token": "string",
    "expires_in": 3600,
    "token_type": "Bearer"
  },
  "session_info": {
    "session_id": "uuid",
    "device_info": "string|null",
    "ip_address": "string|null",
    "expires_at": "2024-01-08T00:00:00Z"
  }
}
```

**Error Responses:**
- `400 Bad Request` - Validation errors
- `401 Unauthorized` - Invalid credentials
- `429 Too Many Requests` - Rate limit exceeded

#### POST /refresh

Refresh an access token using a refresh token.

**Request Body:**
```json
{
  "refresh_token": "string"
}
```

**Response (200 OK):**
```json
{
  "access_token": "string",
  "refresh_token": "string",
  "expires_in": 3600,
  "token_type": "Bearer"
}
```

**Error Responses:**
- `400 Bad Request` - Invalid refresh token format
- `401 Unauthorized` - Expired or invalid refresh token
- `429 Too Many Requests` - Rate limit exceeded

### Protected Endpoints

These endpoints require authentication via Bearer token.

#### GET /user/profile

Get the current user's profile information.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Response (200 OK):**
```json
{
  "id": "uuid",
  "username": "string",
  "email": "string",
  "role": "user|seller|admin",
  "credit_balance": "decimal",
  "internal_wallet_address": "0x...",
  "phone_number": "string|null",
  "is_active": true,
  "email_verified": boolean,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### PUT /user/profile

Update the current user's profile information.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "username": "string (optional)",
  "phone_number": "string (optional)"
}
```

**Response (200 OK):**
```json
{
  "id": "uuid",
  "username": "string",
  "email": "string",
  "role": "user|seller|admin",
  "credit_balance": "decimal",
  "internal_wallet_address": "0x...",
  "phone_number": "string|null",
  "is_active": true,
  "email_verified": boolean,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### POST /user/logout

Logout the current user and invalidate their session.

**Headers:**
```
Authorization: Bearer <access_token>
```

**Request Body:**
```json
{
  "logout_all_devices": "boolean (optional, default: false)"
}
```

**Response (200 OK):**
```json
{
  "message": "Logged out successfully"
}
```

## Error Handling

All endpoints return consistent error responses:

```json
{
  "error": "error_code",
  "message": "Human readable error message",
  "details": "Additional error details (optional)"
}
```

### Common Error Codes

- `validation_error` - Request validation failed
- `authentication_required` - Authentication token required
- `invalid_token` - Authentication token is invalid or expired
- `insufficient_permissions` - User lacks required permissions
- `rate_limit_exceeded` - Too many requests
- `internal_server_error` - Server error occurred

## Security Considerations

### Rate Limiting
- Authentication endpoints: 10 requests per minute per IP
- Profile endpoints: 60 requests per minute per user
- Password operations: 5 requests per minute per user

### Token Security
- Access tokens expire in 1 hour
- Refresh tokens expire in 7 days
- Tokens are automatically rotated on refresh
- All tokens are revoked on password change

### Input Validation
- All inputs are validated and sanitized
- Email format validation
- Username character restrictions
- Phone number format validation
- Password strength requirements

### Session Management
- Sessions are tracked and can be managed by users
- Concurrent session limits can be configured
- Sessions are invalidated on suspicious activity