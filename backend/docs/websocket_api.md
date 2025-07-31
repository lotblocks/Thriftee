# WebSocket Real-time API Documentation

## Overview

The WebSocket API provides real-time communication for the raffle platform, enabling live updates for raffle grids, item changes, user activities, and system events. This allows for an interactive and engaging user experience with instant feedback.

## Connection

### WebSocket Endpoint
```
ws://localhost:8080/ws
wss://your-domain.com/ws (production)
```

### Connection Flow
1. **Connect**: Establish WebSocket connection
2. **Authenticate**: Send authentication message with JWT token
3. **Subscribe**: Subscribe to specific events or rooms
4. **Receive Events**: Listen for real-time events
5. **Send Messages**: Send ping/pong and other client messages

## Message Format

All messages are JSON objects with the following structure:

```json
{
  "message_type": "string",
  "data": {},
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Client Messages

### Authentication
Authenticate the WebSocket connection with a JWT token.

```json
{
  "message_type": "auth",
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  }
}
```

**Response**: Connection will be authenticated and user events will be enabled.

### Subscribe to Events
Subscribe to specific event types and filters.

```json
{
  "message_type": "subscribe",
  "data": {
    "subscriptions": [
      {
        "event_type": "box_purchased",
        "raffle_id": "uuid-raffle-123",
        "item_id": null,
        "user_id": null,
        "room": null
      },
      {
        "event_type": "all",
        "raffle_id": null,
        "item_id": "uuid-item-456",
        "user_id": null,
        "room": "raffle_uuid-raffle-123"
      }
    ]
  }
}
```

**Event Types:**
- `all` - Subscribe to all events (filtered by other parameters)
- `raffle_created` - New raffle created
- `box_purchased` - Box purchased in raffle
- `raffle_full` - Raffle completed (all boxes sold)
- `winner_selected` - Winners selected for raffle
- `raffle_cancelled` - Raffle cancelled
- `item_created` - New item created
- `item_updated` - Item details updated
- `item_stock_changed` - Item stock quantity changed
- `user_joined` - User connected to WebSocket
- `user_left` - User disconnected from WebSocket
- `credits_issued` - Credits issued to user
- `credits_redeemed` - Credits redeemed by user
- `system_maintenance` - System maintenance notifications
- `system_alert` - System alerts and announcements

**Filters:**
- `raffle_id` - Filter events for specific raffle
- `item_id` - Filter events for specific item
- `user_id` - Filter events for specific user
- `room` - Subscribe to room-based events

### Ping
Send periodic ping to maintain connection.

```json
{
  "message_type": "ping",
  "data": {}
}
```

**Response**: Server will respond with pong message.

## Server Events

### Raffle Events

#### Raffle Created
Sent when a new raffle is created.

```json
{
  "message_type": "raffle_created",
  "data": {
    "raffle_id": "uuid-raffle-123",
    "item_id": "uuid-item-456",
    "seller_id": "uuid-seller-789",
    "total_boxes": 100,
    "box_price": 2.50,
    "created_at": "2024-01-15T10:30:00Z"
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### Box Purchased
Sent when a box is purchased in a raffle.

```json
{
  "message_type": "box_purchased",
  "data": {
    "raffle_id": "uuid-raffle-123",
    "user_id": "uuid-user-456",
    "box_number": 25,
    "boxes_remaining": 75,
    "completion_percentage": 25.0,
    "purchased_at": "2024-01-15T10:35:00Z"
  },
  "timestamp": "2024-01-15T10:35:00Z"
}
```

#### Raffle Full
Sent when all boxes in a raffle are sold.

```json
{
  "message_type": "raffle_full",
  "data": {
    "raffle_id": "uuid-raffle-123",
    "item_id": "uuid-item-456",
    "total_boxes": 100,
    "completed_at": "2024-01-15T12:00:00Z"
  },
  "timestamp": "2024-01-15T12:00:00Z"
}
```

#### Winner Selected
Sent when winners are selected for a completed raffle.

```json
{
  "message_type": "winner_selected",
  "data": {
    "raffle_id": "uuid-raffle-123",
    "item_id": "uuid-item-456",
    "winner_user_ids": ["uuid-user-789"],
    "completed_at": "2024-01-15T12:05:00Z"
  },
  "timestamp": "2024-01-15T12:05:00Z"
}
```

#### Raffle Cancelled
Sent when a raffle is cancelled.

```json
{
  "message_type": "raffle_cancelled",
  "data": {
    "raffle_id": "uuid-raffle-123",
    "item_id": "uuid-item-456",
    "reason": "Item no longer available",
    "cancelled_at": "2024-01-15T11:00:00Z"
  },
  "timestamp": "2024-01-15T11:00:00Z"
}
```

### Item Events

#### Item Created
Sent when a new item is created.

```json
{
  "message_type": "item_created",
  "data": {
    "item_id": "uuid-item-456",
    "seller_id": "uuid-seller-789",
    "name": "Wireless Headphones",
    "category": "Electronics",
    "created_at": "2024-01-15T09:00:00Z"
  },
  "timestamp": "2024-01-15T09:00:00Z"
}
```

#### Item Stock Changed
Sent when item stock quantity is updated.

```json
{
  "message_type": "item_stock_changed",
  "data": {
    "item_id": "uuid-item-456",
    "old_quantity": 10,
    "new_quantity": 5,
    "updated_at": "2024-01-15T14:00:00Z"
  },
  "timestamp": "2024-01-15T14:00:00Z"
}
```

### User Events

#### User Joined
Sent when a user connects and authenticates.

```json
{
  "message_type": "user_joined",
  "data": {
    "user_id": "uuid-user-456",
    "username": "user@example.com",
    "joined_at": "2024-01-15T10:00:00Z"
  },
  "timestamp": "2024-01-15T10:00:00Z"
}
```

#### User Left
Sent when a user disconnects.

```json
{
  "message_type": "user_left",
  "data": {
    "user_id": "uuid-user-456",
    "left_at": "2024-01-15T11:00:00Z"
  },
  "timestamp": "2024-01-15T11:00:00Z"
}
```

### Credit Events

#### Credits Issued
Sent when credits are issued to a user.

```json
{
  "message_type": "credits_issued",
  "data": {
    "user_id": "uuid-user-456",
    "amount": 50.00,
    "source": "payment",
    "issued_at": "2024-01-15T10:30:00Z"
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

#### Credits Redeemed
Sent when credits are redeemed by a user.

```json
{
  "message_type": "credits_redeemed",
  "data": {
    "user_id": "uuid-user-456",
    "amount": 10.00,
    "item_id": "uuid-item-456",
    "redeemed_at": "2024-01-15T10:35:00Z"
  },
  "timestamp": "2024-01-15T10:35:00Z"
}
```

### System Events

#### System Maintenance
Sent for scheduled maintenance notifications.

```json
{
  "message_type": "system_maintenance",
  "data": {
    "message": "Scheduled maintenance will begin in 30 minutes",
    "scheduled_at": "2024-01-15T15:00:00Z"
  },
  "timestamp": "2024-01-15T14:30:00Z"
}
```

#### System Alert
Sent for system-wide alerts and announcements.

```json
{
  "message_type": "system_alert",
  "data": {
    "level": "info",
    "message": "New feature: Mobile app now available!",
    "created_at": "2024-01-15T12:00:00Z"
  },
  "timestamp": "2024-01-15T12:00:00Z"
}
```

### Server Responses

#### Pong
Response to client ping messages.

```json
{
  "message_type": "pong",
  "data": {
    "timestamp": 1705320600
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Connection Management

### Heartbeat
- Client should send ping messages every 30 seconds
- Server will close connections that don't ping within 5 minutes
- Server sends pong responses to maintain connection health

### Authentication
- Connections start unauthenticated
- Send auth message with valid JWT token to authenticate
- Authenticated connections receive user-specific events
- Invalid tokens will result in connection termination

### Subscription Management
- Connections start with no subscriptions
- Send subscribe message to receive events
- Subscriptions can be updated by sending new subscribe message
- Use filters to limit events to relevant data

## Error Handling

### Connection Errors
- Invalid JSON messages are ignored with warning log
- Unknown message types are ignored with warning log
- Authentication failures result in connection termination
- Subscription errors are logged but don't terminate connection

### Event Delivery
- Failed event deliveries result in connection cleanup
- Stale connections are automatically removed
- Events are not queued for offline users

## Usage Examples

### Frontend JavaScript Example

```javascript
// Connect to WebSocket
const ws = new WebSocket('ws://localhost:8080/ws');

// Handle connection open
ws.onopen = function() {
    console.log('WebSocket connected');
    
    // Authenticate
    ws.send(JSON.stringify({
        message_type: 'auth',
        data: {
            token: localStorage.getItem('jwt_token')
        }
    }));
    
    // Subscribe to raffle events
    ws.send(JSON.stringify({
        message_type: 'subscribe',
        data: {
            subscriptions: [
                {
                    event_type: 'box_purchased',
                    raffle_id: 'current-raffle-id',
                    item_id: null,
                    user_id: null,
                    room: null
                },
                {
                    event_type: 'raffle_full',
                    raffle_id: 'current-raffle-id',
                    item_id: null,
                    user_id: null,
                    room: null
                }
            ]
        }
    }));
};

// Handle incoming messages
ws.onmessage = function(event) {
    const message = JSON.parse(event.data);
    
    switch(message.message_type) {
        case 'box_purchased':
            updateRaffleGrid(message.data);
            break;
        case 'raffle_full':
            showRaffleCompleted(message.data);
            break;
        case 'winner_selected':
            showWinners(message.data);
            break;
        case 'pong':
            console.log('Pong received');
            break;
        default:
            console.log('Unknown message type:', message.message_type);
    }
};

// Send periodic pings
setInterval(() => {
    if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({
            message_type: 'ping',
            data: {}
        }));
    }
}, 30000);

// Handle connection close
ws.onclose = function() {
    console.log('WebSocket disconnected');
    // Implement reconnection logic
};

// Handle errors
ws.onerror = function(error) {
    console.error('WebSocket error:', error);
};
```

### React Hook Example

```javascript
import { useEffect, useState, useRef } from 'react';

export function useWebSocket(url, token) {
    const [isConnected, setIsConnected] = useState(false);
    const [lastMessage, setLastMessage] = useState(null);
    const ws = useRef(null);

    useEffect(() => {
        ws.current = new WebSocket(url);

        ws.current.onopen = () => {
            setIsConnected(true);
            
            // Authenticate
            ws.current.send(JSON.stringify({
                message_type: 'auth',
                data: { token }
            }));
        };

        ws.current.onmessage = (event) => {
            const message = JSON.parse(event.data);
            setLastMessage(message);
        };

        ws.current.onclose = () => {
            setIsConnected(false);
        };

        return () => {
            ws.current.close();
        };
    }, [url, token]);

    const sendMessage = (message) => {
        if (ws.current.readyState === WebSocket.OPEN) {
            ws.current.send(JSON.stringify(message));
        }
    };

    const subscribe = (subscriptions) => {
        sendMessage({
            message_type: 'subscribe',
            data: { subscriptions }
        });
    };

    return { isConnected, lastMessage, sendMessage, subscribe };
}
```

## Monitoring and Statistics

### Connection Statistics Endpoint
Get current WebSocket connection statistics.

**GET** `/ws/stats`

**Response:**
```json
{
  "total_connections": 150,
  "authenticated_connections": 120,
  "active_rooms": {
    "raffle_uuid-123": 25,
    "raffle_uuid-456": 30
  },
  "events_sent_last_hour": 1250
}
```

## Security Considerations

- All connections require JWT authentication for user-specific events
- Subscription filters prevent unauthorized access to private data
- Connection limits prevent DoS attacks
- Message rate limiting prevents spam
- Automatic cleanup of stale connections
- Secure WebSocket (WSS) required in production

## Performance Notes

- Events are broadcast only to relevant subscribers
- Failed connections are automatically cleaned up
- Memory usage is optimized through connection pooling
- Event delivery is non-blocking and asynchronous
- Supports thousands of concurrent connections

## Troubleshooting

### Common Issues

1. **Connection Drops**: Check network stability and implement reconnection logic
2. **Missing Events**: Verify subscription filters and authentication
3. **High Latency**: Check server load and network conditions
4. **Authentication Failures**: Verify JWT token validity and format

### Debug Tips

- Enable WebSocket debugging in browser developer tools
- Check server logs for connection and event information
- Monitor connection statistics endpoint
- Implement client-side logging for message tracking