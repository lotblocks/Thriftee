# Item and Raffle API Documentation

## Overview

The Item and Raffle API provides comprehensive functionality for managing items and raffles on the platform. Items are products that sellers list, and raffles are the gamified purchasing mechanism where users buy "boxes" to potentially win items.

## Authentication

Most endpoints require JWT authentication. Include the JWT token in the Authorization header:

```
Authorization: Bearer <jwt_token>
```

## Item Management API

### Public Item Endpoints

#### Search Items
Search and filter items with pagination.

**GET** `/api/v1/items?search=electronics&category=Electronics&min_price=10&max_price=100&sort_by=price_asc&limit=20&offset=0`

**Query Parameters:**
- `search` (optional): Search term for item name/description
- `category` (optional): Filter by category
- `min_price` (optional): Minimum price filter
- `max_price` (optional): Maximum price filter
- `status` (optional): Filter by status (available, sold, inactive)
- `seller_id` (optional): Filter by seller
- `sort_by` (optional): Sort order (price_asc, price_desc, created_asc, created_desc, name)
- `limit` (optional): Number of items to return (1-100, default: 20)
- `offset` (optional): Pagination offset

**Response:**
```json
{
  "data": [
    {
      "id": "uuid-123",
      "seller_id": "uuid-456",
      "name": "Wireless Headphones",
      "description": "High-quality wireless headphones",
      "images": ["https://example.com/image1.jpg"],
      "retail_price": 99.99,
      "cost_of_goods": 50.00,
      "status": "available",
      "stock_quantity": 10,
      "listing_fee_applied": 5.00,
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 150,
  "limit": 20,
  "offset": 0,
  "has_more": true
}
```

#### Get Item by ID
Get detailed information about a specific item.

**GET** `/api/v1/items/{item_id}`

**Response:**
```json
{
  "id": "uuid-123",
  "seller_id": "uuid-456",
  "name": "Wireless Headphones",
  "description": "High-quality wireless headphones with noise cancellation",
  "images": ["https://example.com/image1.jpg", "https://example.com/image2.jpg"],
  "retail_price": 99.99,
  "cost_of_goods": 50.00,
  "status": "available",
  "stock_quantity": 10,
  "listing_fee_applied": 5.00,
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

#### Get Popular Items
Get trending/popular items based on views, favorites, and raffle activity.

**GET** `/api/v1/items/popular?limit=10`

#### Get Item Categories
Get all available item categories.

**GET** `/api/v1/items/categories`

**Response:**
```json
{
  "categories": [
    {
      "id": 1,
      "name": "Electronics",
      "description": "Electronic devices and gadgets"
    },
    {
      "id": 2,
      "name": "Fashion",
      "description": "Clothing, accessories, and fashion items"
    }
  ]
}
```

### Seller Item Endpoints (Authentication Required)

#### Create Item
Create a new item listing (sellers only).

**POST** `/api/v1/items`

**Request Body:**
```json
{
  "name": "Wireless Headphones",
  "description": "High-quality wireless headphones with noise cancellation",
  "images": ["https://example.com/image1.jpg", "https://example.com/image2.jpg"],
  "retail_price": 99.99,
  "cost_of_goods": 50.00,
  "stock_quantity": 10,
  "category": "Electronics"
}
```

**Response:**
```json
{
  "item": {
    "id": "uuid-123",
    "seller_id": "uuid-456",
    "name": "Wireless Headphones",
    "description": "High-quality wireless headphones with noise cancellation",
    "images": ["https://example.com/image1.jpg"],
    "retail_price": 99.99,
    "cost_of_goods": 50.00,
    "status": "available",
    "stock_quantity": 10,
    "listing_fee_applied": 5.00,
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  },
  "listing_fee": 5.00,
  "message": "Item created successfully"
}
```

#### Get Seller's Items
Get all items belonging to the authenticated seller.

**GET** `/api/v1/items/my-items?limit=20&offset=0`

#### Update Item
Update an existing item (sellers only).

**PUT** `/api/v1/items/{item_id}`

**Request Body:**
```json
{
  "name": "Updated Wireless Headphones",
  "description": "Updated description",
  "retail_price": 89.99,
  "stock_quantity": 15
}
```

#### Delete Item
Soft delete an item (sellers only).

**DELETE** `/api/v1/items/{item_id}`

**Response:**
```json
{
  "success": true,
  "message": "Item deleted successfully",
  "item_id": "uuid-123"
}
```

#### Update Item Status
Update the status of an item.

**PUT** `/api/v1/items/{item_id}/status`

**Request Body:**
```json
{
  "status": "inactive"
}
```

#### Update Item Stock
Update the stock quantity of an item.

**PUT** `/api/v1/items/{item_id}/stock`

**Request Body:**
```json
{
  "quantity": 25
}
```

#### Get Item Analytics
Get analytics data for a specific item (sellers only).

**GET** `/api/v1/items/{item_id}/analytics`

**Response:**
```json
{
  "item_id": "uuid-123",
  "views_count": 1250,
  "unique_viewers": 890,
  "raffle_count": 5,
  "total_boxes_sold": 150,
  "total_revenue": 750.00,
  "conversion_rate": 12.5,
  "profit_margin": 49.95,
  "performance_score": 85.5
}
```

#### Bulk Operations
Perform bulk operations on multiple items.

**POST** `/api/v1/items/bulk-operation`

**Request Body:**
```json
{
  "item_ids": ["uuid-123", "uuid-456", "uuid-789"],
  "operation": "update_status",
  "value": "inactive"
}
```

**Operations:**
- `update_status`: Update status of multiple items
- `update_category`: Update category of multiple items
- `delete`: Delete multiple items
- `update_stock`: Update stock quantity of multiple items

#### Get Item Statistics
Get statistics for seller's items.

**GET** `/api/v1/items/statistics`

**Response:**
```json
{
  "total_items": 25,
  "available_items": 20,
  "sold_items": 3,
  "inactive_items": 2,
  "total_value": 2500.00,
  "average_price": 125.00,
  "items_by_category": {
    "Electronics": 15,
    "Fashion": 10
  },
  "items_by_seller": {
    "uuid-456": 25
  }
}
```

### Admin Item Endpoints

#### Get All Items (Admin Only)
Get all items on the platform with admin privileges.

**GET** `/api/v1/items/admin/all`

#### Get Platform Item Statistics (Admin Only)
Get platform-wide item statistics.

**GET** `/api/v1/items/admin/statistics`

## Raffle Management API

### Public Raffle Endpoints

#### Search Raffles
Search and filter raffles with pagination.

**GET** `/api/v1/raffles?status=open&min_price=5&max_price=50&sort_by=created_desc&limit=20&offset=0`

**Query Parameters:**
- `status` (optional): Filter by status (open, full, drawing, completed, cancelled)
- `item_id` (optional): Filter by item
- `min_price` (optional): Minimum box price filter
- `max_price` (optional): Maximum box price filter
- `completion_min` (optional): Minimum completion percentage (0-100)
- `completion_max` (optional): Maximum completion percentage (0-100)
- `sort_by` (optional): Sort order (created_asc, created_desc, price_asc, price_desc, completion)
- `limit` (optional): Number of raffles to return (1-100, default: 20)
- `offset` (optional): Pagination offset

**Response:**
```json
{
  "data": [
    {
      "id": "uuid-raffle-123",
      "item_id": "uuid-item-456",
      "item": {
        "id": "uuid-item-456",
        "name": "Wireless Headphones",
        "images": ["https://example.com/image1.jpg"],
        "retail_price": 99.99
      },
      "total_boxes": 100,
      "box_price": 2.50,
      "boxes_sold": 75,
      "total_winners": 1,
      "status": "open",
      "winner_user_ids": [],
      "grid_rows": 10,
      "grid_cols": 10,
      "started_at": null,
      "completed_at": null,
      "created_at": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 50,
  "limit": 20,
  "offset": 0,
  "has_more": true
}
```

#### Get Raffle by ID
Get detailed information about a specific raffle.

**GET** `/api/v1/raffles/{raffle_id}`

#### Get Active Raffles
Get all currently active (open) raffles.

**GET** `/api/v1/raffles/active?limit=20`

#### Get Featured Raffles
Get featured/promoted raffles.

**GET** `/api/v1/raffles/featured?limit=10`

#### Get Raffle Grid State
Get the current state of the raffle grid showing purchased and available boxes.

**GET** `/api/v1/raffles/{raffle_id}/grid`

**Response:**
```json
{
  "raffle_id": "uuid-raffle-123",
  "grid_rows": 10,
  "grid_cols": 10,
  "purchased_boxes": {
    "1": {
      "user_id": "uuid-user-789",
      "username": "user@example.com",
      "purchased_at": "2024-01-15T10:30:00Z",
      "is_current_user": false
    },
    "5": {
      "user_id": "uuid-user-456",
      "username": "buyer@example.com",
      "purchased_at": "2024-01-15T11:00:00Z",
      "is_current_user": true
    }
  },
  "available_boxes": [2, 3, 4, 6, 7, 8, 9, 10],
  "total_boxes": 100,
  "boxes_sold": 75,
  "completion_percentage": 75.0
}
```

#### Get Raffle Winners
Get the winners of a completed raffle.

**GET** `/api/v1/raffles/{raffle_id}/winners`

**Response:**
```json
{
  "raffle_id": "uuid-raffle-123",
  "status": "completed",
  "winner_user_ids": ["uuid-user-789"],
  "total_winners": 1,
  "completed_at": "2024-01-15T15:30:00Z"
}
```

### User Raffle Endpoints (Authentication Required)

#### Purchase Boxes
Purchase boxes in a raffle.

**POST** `/api/v1/raffles/{raffle_id}/buy-boxes`

**Request Body:**
```json
{
  "box_numbers": [1, 5, 10, 25],
  "use_credits": true
}
```

**Response:**
```json
{
  "purchases": [
    {
      "id": "uuid-purchase-123",
      "raffle_id": "uuid-raffle-123",
      "user_id": "uuid-user-456",
      "box_number": 1,
      "purchase_price_in_credits": 2.50,
      "blockchain_tx_hash": null,
      "created_at": "2024-01-15T10:30:00Z"
    }
  ],
  "total_cost": 10.00,
  "raffle_status": "open",
  "boxes_remaining": 21,
  "message": "Successfully purchased 4 boxes"
}
```

#### Get User's Purchases for Raffle
Get the authenticated user's purchases for a specific raffle.

**GET** `/api/v1/raffles/{raffle_id}/my-purchases`

**Response:**
```json
{
  "raffle_id": "uuid-raffle-123",
  "user_id": "uuid-user-456",
  "purchases": [
    {
      "id": "uuid-purchase-123",
      "raffle_id": "uuid-raffle-123",
      "user_id": "uuid-user-456",
      "box_number": 1,
      "purchase_price_in_credits": 2.50,
      "blockchain_tx_hash": null,
      "created_at": "2024-01-15T10:30:00Z"
    }
  ],
  "total_boxes": 4,
  "total_spent": 10.00
}
```

#### Get User's Purchase History
Get the authenticated user's complete purchase history across all raffles.

**GET** `/api/v1/raffles/my-history?limit=20&offset=0`

### Seller Raffle Endpoints (Authentication Required)

#### Create Raffle
Create a new raffle for an item (sellers only).

**POST** `/api/v1/raffles`

**Request Body:**
```json
{
  "item_id": "uuid-item-456",
  "total_boxes": 100,
  "box_price": 2.50,
  "total_winners": 1,
  "grid_rows": 10,
  "grid_cols": 10
}
```

**Response:**
```json
{
  "raffle": {
    "id": "uuid-raffle-123",
    "item_id": "uuid-item-456",
    "item": {
      "id": "uuid-item-456",
      "name": "Wireless Headphones",
      "images": ["https://example.com/image1.jpg"],
      "retail_price": 99.99
    },
    "total_boxes": 100,
    "box_price": 2.50,
    "boxes_sold": 0,
    "total_winners": 1,
    "status": "open",
    "winner_user_ids": [],
    "grid_rows": 10,
    "grid_cols": 10,
    "started_at": null,
    "completed_at": null,
    "created_at": "2024-01-15T10:30:00Z"
  },
  "transaction_fee": 12.50,
  "blockchain_tx_hash": null,
  "message": "Raffle created successfully"
}
```

#### Cancel Raffle
Cancel an active raffle (sellers/admin only).

**POST** `/api/v1/raffles/{raffle_id}/cancel`

**Request Body:**
```json
{
  "reason": "Item no longer available"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Raffle cancelled successfully",
  "raffle_id": "uuid-raffle-123",
  "reason": "Item no longer available"
}
```

#### Get Raffle Statistics
Get raffle statistics for sellers/admins.

**GET** `/api/v1/raffles/statistics`

**Response:**
```json
{
  "total_raffles": 150,
  "active_raffles": 25,
  "completed_raffles": 120,
  "cancelled_raffles": 5,
  "total_revenue": 15000.00,
  "total_boxes_sold": 6000,
  "average_completion_time": 1440,
  "completion_rate": 80.0
}
```

### Admin Raffle Endpoints

#### Get All Raffles (Admin Only)
Get all raffles on the platform with admin privileges.

**GET** `/api/v1/raffles/admin/all`

#### Force Complete Raffle (Admin Only)
Force complete a raffle for administrative purposes.

**POST** `/api/v1/raffles/admin/{raffle_id}/force-complete`

## Health Check Endpoints

#### Item Service Health
Check the health status of the item service.

**GET** `/api/v1/items/health`

#### Raffle Service Health
Check the health status of the raffle service.

**GET** `/api/v1/raffles/health`

## Error Responses

All endpoints return consistent error responses:

```json
{
  "error": "validation_error",
  "message": "Invalid box numbers provided",
  "details": {
    "field": "box_numbers",
    "code": "invalid_range"
  }
}
```

### Common Error Codes

- `400 Bad Request`: Invalid request data or validation errors
- `401 Unauthorized`: Missing or invalid authentication token
- `403 Forbidden`: Insufficient permissions (e.g., seller-only endpoint)
- `404 Not Found`: Resource not found (item, raffle, etc.)
- `409 Conflict`: Resource conflict (e.g., box already purchased)
- `500 Internal Server Error`: Server error or external service failure

## Business Rules

### Item Management Rules
- Only sellers can create, update, and delete items
- Items must have at least one image
- Retail price must be greater than cost of goods
- Stock quantity cannot be negative
- Items with active raffles cannot be deleted

### Raffle Management Rules
- Only sellers can create raffles for their own items
- Items can only have one active raffle at a time
- Grid size must accommodate all boxes (rows × cols ≥ total_boxes)
- Total winners cannot exceed total boxes
- Box prices must be positive
- Boxes cannot be purchased twice
- Raffles automatically move to "full" status when all boxes are sold
- Winner selection is triggered automatically when raffle is full

### Purchase Rules
- Users can only purchase available boxes
- Multiple boxes can be purchased in a single transaction
- Credits are the primary payment method
- Purchases are immediately reflected in the grid state
- Refunds are issued as credits if raffle is cancelled

## Integration Notes

### Blockchain Integration
- Raffles can optionally be created on the blockchain for transparency
- Winner selection uses Chainlink VRF for verifiable randomness
- All blockchain transactions are recorded for audit purposes

### Credit System Integration
- Box purchases automatically deduct credits from user accounts
- Refunds are issued as credits with appropriate expiration dates
- Credit redemption is validated before purchase completion

### Notification Integration
- Users receive notifications for successful purchases
- Sellers are notified when their raffles complete
- Winners receive notifications when selected