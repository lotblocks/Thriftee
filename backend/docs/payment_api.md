# Payment API Documentation

## Overview

The Payment API provides comprehensive payment processing capabilities using Stripe as the payment gateway. It supports credit purchases, subscription management, and webhook processing for real-time payment updates.

## Authentication

All payment endpoints (except webhooks) require JWT authentication. Include the JWT token in the Authorization header:

```
Authorization: Bearer <jwt_token>
```

## Endpoints

### User Payment Endpoints

#### Create Payment Intent
Create a payment intent for credit purchase.

**POST** `/api/v1/payments/create-intent`

**Request Body:**
```json
{
  "amount": 50.00,
  "currency": "USD",
  "description": "Credit purchase - 50 credits",
  "metadata": {
    "purpose": "credit_purchase",
    "credits": "50"
  }
}
```

**Response:**
```json
{
  "payment_intent_id": "pi_1234567890",
  "client_secret": "pi_1234567890_secret_abc123",
  "amount": 5000,
  "currency": "USD",
  "status": "requires_payment_method"
}
```

#### Confirm Payment Intent
Confirm a payment intent with a payment method.

**POST** `/api/v1/payments/confirm-intent/{payment_intent_id}`

**Request Body:**
```json
{
  "payment_method_id": "pm_1234567890"
}
```

**Response:**
```json
{
  "payment_intent_id": "pi_1234567890",
  "client_secret": "pi_1234567890_secret_abc123",
  "amount": 5000,
  "currency": "USD",
  "status": "succeeded"
}
```

#### Get Payment Intent Status
Get the current status of a payment intent.

**GET** `/api/v1/payments/intent-status/{payment_intent_id}`

**Response:**
```json
{
  "payment_intent_id": "pi_1234567890",
  "status": "succeeded",
  "amount": 50.00,
  "currency": "USD",
  "description": "Credit purchase - 50 credits",
  "created_at": "2024-01-15T10:30:00Z",
  "completed_at": "2024-01-15T10:31:00Z",
  "failure_reason": null
}
```

#### Get Payment History
Get user's payment history with optional filtering.

**GET** `/api/v1/payments/history?limit=20&status=succeeded`

**Query Parameters:**
- `limit` (optional): Number of payments to return (1-100, default: 50)
- `status` (optional): Filter by payment status (pending, processing, succeeded, failed, cancelled, refunded)

**Response:**
```json
{
  "payments": [
    {
      "id": "uuid-1234",
      "stripe_payment_intent_id": "pi_1234567890",
      "amount": 50.00,
      "currency": "USD",
      "status": "succeeded",
      "description": "Credit purchase - 50 credits",
      "created_at": "2024-01-15T10:30:00Z",
      "completed_at": "2024-01-15T10:31:00Z",
      "failure_reason": null
    }
  ],
  "total_amount": 150.00,
  "successful_payments": 3,
  "failed_payments": 1
}
```

#### Get Payment Statistics
Get user's payment statistics and summary.

**GET** `/api/v1/payments/statistics`

**Response:**
```json
{
  "total_spent": 150.00,
  "successful_payments": 3,
  "failed_payments": 1,
  "average_payment": 50.00,
  "active_subscriptions": 1,
  "monthly_subscription_cost": 29.99,
  "first_payment_date": "2024-01-01T10:00:00Z",
  "last_payment_date": "2024-01-15T10:30:00Z"
}
```

### Subscription Endpoints

#### Create Subscription
Create a subscription for seller plans.

**POST** `/api/v1/payments/subscriptions`

**Request Body:**
```json
{
  "price_id": "price_1234567890",
  "trial_period_days": 14,
  "metadata": {
    "plan": "professional",
    "features": "unlimited_listings"
  }
}
```

**Response:**
```json
{
  "subscription_id": "sub_1234567890",
  "status": "trialing",
  "current_period_start": "2024-01-15T10:30:00Z",
  "current_period_end": "2024-02-15T10:30:00Z",
  "trial_end": "2024-01-29T10:30:00Z"
}
```

#### Get User Subscriptions
Get user's active subscriptions.

**GET** `/api/v1/payments/subscriptions`

**Response:**
```json
[
  {
    "id": "uuid-5678",
    "stripe_subscription_id": "sub_1234567890",
    "status": "active",
    "current_period_start": "2024-01-15T10:30:00Z",
    "current_period_end": "2024-02-15T10:30:00Z",
    "trial_end": null,
    "cancelled_at": null,
    "created_at": "2024-01-15T10:30:00Z"
  }
]
```

#### Cancel Subscription
Cancel a subscription (will remain active until period end).

**POST** `/api/v1/payments/subscriptions/{subscription_id}/cancel`

**Response:**
```json
{
  "success": true,
  "message": "Subscription cancelled successfully",
  "subscription_id": "sub_1234567890"
}
```

### Admin Endpoints

#### Get Payment Analytics
Get platform-wide payment analytics (admin only).

**GET** `/api/v1/payments/admin/analytics`

**Response:**
```json
{
  "daily_revenue": [
    {
      "date": "2024-01-15",
      "revenue": 1250.00,
      "payments": 25,
      "unique_customers": 20
    }
  ],
  "total_revenue": 15000.00,
  "successful_payments": 300,
  "failed_payments": 15,
  "unique_customers": 150,
  "average_payment_amount": 50.00
}
```

#### Get User Payment Details
Get detailed payment information for a specific user (admin only).

**GET** `/api/v1/payments/admin/user/{user_id}`

**Response:**
```json
{
  "user_id": "uuid-user-123",
  "payment_history": {
    "payments": [...],
    "total_amount": 150.00,
    "successful_payments": 3,
    "failed_payments": 1
  },
  "subscriptions": [...]
}
```

#### Process Refund
Process a refund for a payment (admin only).

**POST** `/api/v1/payments/admin/refund/{payment_id}`

**Response:**
```json
{
  "success": true,
  "message": "Refund processed successfully",
  "payment_id": "uuid-payment-123"
}
```

### Webhook Endpoints

#### Stripe Webhook
Handles Stripe webhook events for payment processing.

**POST** `/api/payments/webhook`

**Headers:**
- `stripe-signature`: Stripe webhook signature for verification

**Supported Events:**
- `payment_intent.succeeded`: Payment completed successfully
- `payment_intent.payment_failed`: Payment failed
- `invoice.payment_succeeded`: Subscription payment succeeded
- `invoice.payment_failed`: Subscription payment failed
- `customer.subscription.deleted`: Subscription cancelled

### Health Check

#### Payment Service Health
Check the health status of the payment service.

**GET** `/api/v1/payments/health`

**Response:**
```json
{
  "status": "healthy",
  "service": "payment",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Error Responses

All endpoints return consistent error responses:

```json
{
  "error": "validation_error",
  "message": "Invalid amount: must be between 1.00 and 10000.00",
  "details": {
    "field": "amount",
    "code": "out_of_range"
  }
}
```

### Common Error Codes

- `400 Bad Request`: Invalid request data or validation errors
- `401 Unauthorized`: Missing or invalid authentication token
- `403 Forbidden`: Insufficient permissions (e.g., admin-only endpoint)
- `404 Not Found`: Resource not found (payment, subscription, etc.)
- `409 Conflict`: Resource conflict (e.g., duplicate payment)
- `500 Internal Server Error`: Server error or external service failure

## Integration Flow

### Credit Purchase Flow

1. **Create Payment Intent**: Client calls `/create-intent` with amount and description
2. **Collect Payment**: Client uses Stripe.js to collect payment method and confirm payment
3. **Webhook Processing**: Stripe sends webhook to `/webhook` when payment succeeds/fails
4. **Credit Issuance**: On success, credits are automatically issued to user's account
5. **Notification**: User receives confirmation of credit purchase

### Subscription Flow

1. **Create Subscription**: Client calls `/subscriptions` with price ID
2. **Payment Collection**: Stripe handles recurring payment collection
3. **Webhook Processing**: Stripe sends webhooks for subscription events
4. **Access Control**: Backend checks subscription status for seller features
5. **Renewal/Cancellation**: Automatic renewal or user-initiated cancellation

## Security Considerations

- All webhook payloads are verified using Stripe signature verification
- Payment intents are validated to ensure they belong to the requesting user
- Sensitive payment data is never stored locally (PCI compliance)
- All payment operations are logged for audit purposes
- Rate limiting is applied to prevent abuse

## Testing

Use Stripe's test mode for development and testing:

- Test card numbers: `4242424242424242` (Visa), `4000000000000002` (declined)
- Test webhook events can be triggered from Stripe Dashboard
- Use `sk_test_` keys for all test operations

## Monitoring

The payment service includes comprehensive monitoring:

- Payment success/failure rates
- Average payment processing time
- Webhook delivery status
- Subscription churn rates
- Revenue analytics and trends