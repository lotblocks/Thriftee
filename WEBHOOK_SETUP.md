# Webhook Setup Guide

This document explains how to set up webhooks for the Unit Shopping Platform to handle real-time events from external services.

## Overview

The platform uses webhooks to handle:
1. **Stripe Payment Events** - Credit purchases, subscription payments
2. **Blockchain Events** - Smart contract interactions, transaction confirmations
3. **Notification Events** - Email/SMS delivery status

## Webhook Endpoints

All webhooks are available at:
- **Stripe**: `https://your-domain.com/webhooks/stripe`
- **Blockchain**: `https://your-domain.com/webhooks/blockchain`
- **Notifications**: `https://your-domain.com/webhooks/notifications`

## 1. Stripe Webhook Setup

### Step 1: Create Webhook Endpoint in Stripe Dashboard

1. Go to [Stripe Dashboard](https://dashboard.stripe.com/webhooks)
2. Click "Add endpoint"
3. Enter your webhook URL: `https://your-domain.com/webhooks/stripe`
4. Select these events:
   - `payment_intent.succeeded`
   - `payment_intent.payment_failed`
   - `invoice.payment_succeeded`
   - `invoice.payment_failed`
   - `customer.subscription.deleted`

### Step 2: Configure Webhook Secret

1. After creating the endpoint, copy the "Signing secret"
2. Add it to your environment variables:
   ```env
   STRIPE_WEBHOOK_SECRET=whsec_your_actual_webhook_secret_here
   ```

### Step 3: Test Webhook

Use Stripe CLI to test:
```bash
stripe listen --forward-to localhost:8080/webhooks/stripe
stripe trigger payment_intent.succeeded
```

## 2. Blockchain Webhook Setup

### Option A: Alchemy Notify (Recommended)

1. Go to [Alchemy Dashboard](https://dashboard.alchemy.com/)
2. Navigate to "Notify" section
3. Create a new webhook:
   - **Webhook URL**: `https://your-domain.com/webhooks/blockchain`
   - **Network**: Polygon Mumbai (testnet) or Polygon Mainnet
   - **Addresses to watch**: Your deployed contract address
   - **Events**: All events

4. Copy the webhook signing key and add to environment:
   ```env
   BLOCKCHAIN_WEBHOOK_SECRET=your_alchemy_webhook_secret
   ```

### Option B: Manual Event Listening

If not using Alchemy Notify, the backend will listen to events directly via WebSocket connection to the blockchain node.

### Step 3: Configure Contract Address

Add your deployed contract address:
```env
CONTRACT_ADDRESS=0xYourActualContractAddress
```

## 3. Notification Webhook Setup

### For Email (SendGrid/Mailgun)

#### SendGrid:
1. Go to SendGrid Settings > Mail Settings > Event Webhook
2. Set HTTP POST URL: `https://your-domain.com/webhooks/notifications`
3. Select events: Delivered, Bounced, Dropped
4. Add webhook signature verification

#### Mailgun:
1. Go to Mailgun Dashboard > Webhooks
2. Add webhook URL: `https://your-domain.com/webhooks/notifications`
3. Select events: delivered, failed, bounced

### For SMS (Twilio)

1. Go to Twilio Console > Phone Numbers
2. Configure webhook URL for status callbacks
3. Set URL: `https://your-domain.com/webhooks/notifications`

## 4. Development Setup

### Local Development with ngrok

For local testing, use ngrok to expose your local server:

```bash
# Install ngrok
npm install -g ngrok

# Expose local server
ngrok http 8080

# Use the ngrok URL for webhook endpoints
# Example: https://abc123.ngrok.io/webhooks/stripe
```

### Environment Variables

Update your `.env` file:
```env
# Webhook secrets
STRIPE_WEBHOOK_SECRET=whsec_your_stripe_secret
BLOCKCHAIN_WEBHOOK_SECRET=your_blockchain_secret
NOTIFICATION_WEBHOOK_SECRET=your_notification_secret

# For development with ngrok
WEBHOOK_BASE_URL=https://abc123.ngrok.io
```

## 5. Security Considerations

### Webhook Signature Verification

All webhooks verify signatures to ensure authenticity:

1. **Stripe**: Uses HMAC-SHA256 with timestamp validation
2. **Blockchain**: Uses service-specific signature verification
3. **Notifications**: Uses HMAC-SHA256 verification

### Rate Limiting

Implement rate limiting for webhook endpoints:
- Max 100 requests per minute per IP
- Exponential backoff for failed webhook processing

### Retry Logic

The platform implements retry logic for failed webhook processing:
- 3 retry attempts with exponential backoff
- Dead letter queue for permanently failed webhooks
- Monitoring and alerting for webhook failures

## 6. Monitoring and Debugging

### Webhook Logs

All webhook events are logged with structured logging:
```rust
info!("Received webhook", 
    webhook_type = "stripe",
    event_type = "payment_intent.succeeded",
    event_id = "evt_123"
);
```

### Health Checks

Monitor webhook health:
- Response time metrics
- Success/failure rates
- Event processing latency

### Testing Webhooks

Use these tools for testing:

#### Stripe:
```bash
stripe listen --forward-to localhost:8080/webhooks/stripe
stripe trigger payment_intent.succeeded
```

#### Blockchain:
```bash
# Use Hardhat to trigger contract events
npx hardhat run scripts/trigger-events.js --network mumbai
```

#### Manual Testing:
```bash
curl -X POST http://localhost:8080/webhooks/stripe \
  -H "Content-Type: application/json" \
  -H "Stripe-Signature: t=timestamp,v1=signature" \
  -d '{"type": "payment_intent.succeeded", "data": {...}}'
```

## 7. Production Deployment

### SSL/TLS Requirements

All webhook endpoints must use HTTPS in production:
- Use Let's Encrypt or commercial SSL certificates
- Configure proper TLS settings
- Ensure webhook URLs use `https://`

### Load Balancing

For high availability:
- Use load balancer with health checks
- Ensure webhook endpoints are available on all instances
- Implement proper session affinity if needed

### Monitoring

Set up monitoring for:
- Webhook endpoint availability
- Processing latency
- Error rates
- Failed webhook retry queues

## 8. Troubleshooting

### Common Issues

#### Webhook Not Receiving Events
1. Check webhook URL is accessible from internet
2. Verify SSL certificate is valid
3. Check firewall settings
4. Validate webhook endpoint configuration

#### Signature Verification Failures
1. Ensure webhook secret is correct
2. Check timestamp tolerance (5 minutes for Stripe)
3. Verify payload is not modified in transit
4. Check for encoding issues

#### High Latency
1. Optimize webhook processing logic
2. Use async processing for heavy operations
3. Implement proper database connection pooling
4. Monitor resource usage

### Debug Commands

```bash
# Check webhook endpoint health
curl -I https://your-domain.com/webhooks/stripe

# Test webhook processing
curl -X POST https://your-domain.com/webhooks/stripe \
  -H "Content-Type: application/json" \
  -d '{"test": true}'

# View webhook logs
docker logs -f backend-container | grep webhook
```

## 9. Webhook Event Examples

### Stripe Payment Success
```json
{
  "type": "payment_intent.succeeded",
  "data": {
    "object": {
      "id": "pi_1234567890",
      "amount": 2000,
      "currency": "usd",
      "status": "succeeded"
    }
  }
}
```

### Blockchain Box Purchase
```json
{
  "type": "ADDRESS_ACTIVITY",
  "activity": [{
    "hash": "0xabc123...",
    "blockNum": "0x123456",
    "log": {
      "topics": ["0xBoxPurchasedEventSignature", "0xraffleId", "0xuserAddress"],
      "data": "0xboxNumber"
    }
  }]
}
```

### Email Delivery Status
```json
{
  "event": "delivered",
  "message_id": "msg_123456",
  "email": "user@example.com",
  "timestamp": 1234567890
}
```

This webhook infrastructure ensures reliable, real-time processing of all external events critical to the platform's operation.