-- Migration for item management and analytics tables

-- Add missing columns to items table if they don't exist
DO $$ 
BEGIN
    -- Add category column if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'items' AND column_name = 'category') THEN
        ALTER TABLE items ADD COLUMN category VARCHAR(100);
    END IF;
    
    -- Add listing_fee_type column if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns 
                   WHERE table_name = 'items' AND column_name = 'listing_fee_type') THEN
        ALTER TABLE items ADD COLUMN listing_fee_type VARCHAR(50);
    END IF;
END $$;

-- Item activity log table
CREATE TABLE IF NOT EXISTS item_activity_log (
    id BIGSERIAL PRIMARY KEY,
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    activity_type VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Item views tracking table
CREATE TABLE IF NOT EXISTS item_views (
    id BIGSERIAL PRIMARY KEY,
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    session_id VARCHAR(255),
    ip_address INET,
    user_agent TEXT,
    viewed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    view_count INTEGER DEFAULT 1,
    
    -- Unique constraint to prevent duplicate views per day per item
    UNIQUE(item_id, user_id, DATE(viewed_at))
);

-- Item categories table
CREATE TABLE IF NOT EXISTS item_categories (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    parent_category_id BIGINT REFERENCES item_categories(id) ON DELETE SET NULL,
    display_order INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Item images table (for better image management)
CREATE TABLE IF NOT EXISTS item_images (
    id BIGSERIAL PRIMARY KEY,
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    image_url TEXT NOT NULL,
    alt_text VARCHAR(255),
    display_order INTEGER DEFAULT 0,
    is_primary BOOLEAN DEFAULT FALSE,
    file_size BIGINT,
    width INTEGER,
    height INTEGER,
    format VARCHAR(10), -- jpg, png, webp, etc.
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Item favorites/wishlist table
CREATE TABLE IF NOT EXISTS item_favorites (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Unique constraint to prevent duplicate favorites
    UNIQUE(user_id, item_id)
);

-- Item reviews/ratings table
CREATE TABLE IF NOT EXISTS item_reviews (
    id BIGSERIAL PRIMARY KEY,
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    rating INTEGER NOT NULL CHECK (rating BETWEEN 1 AND 5),
    review_text TEXT,
    is_verified_purchase BOOLEAN DEFAULT FALSE,
    is_approved BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Unique constraint to prevent multiple reviews per user per item
    UNIQUE(user_id, item_id)
);

-- Item inventory tracking table
CREATE TABLE IF NOT EXISTS item_inventory_log (
    id BIGSERIAL PRIMARY KEY,
    item_id UUID NOT NULL REFERENCES items(id) ON DELETE CASCADE,
    change_type VARCHAR(50) NOT NULL, -- 'stock_added', 'stock_reduced', 'stock_adjustment'
    quantity_change INTEGER NOT NULL,
    previous_quantity INTEGER NOT NULL,
    new_quantity INTEGER NOT NULL,
    reason VARCHAR(255),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_item_activity_log_item_id ON item_activity_log(item_id);
CREATE INDEX IF NOT EXISTS idx_item_activity_log_user_id ON item_activity_log(user_id);
CREATE INDEX IF NOT EXISTS idx_item_activity_log_activity_type ON item_activity_log(activity_type);
CREATE INDEX IF NOT EXISTS idx_item_activity_log_created_at ON item_activity_log(created_at);

CREATE INDEX IF NOT EXISTS idx_item_views_item_id ON item_views(item_id);
CREATE INDEX IF NOT EXISTS idx_item_views_user_id ON item_views(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_item_views_viewed_at ON item_views(viewed_at);
CREATE INDEX IF NOT EXISTS idx_item_views_session_id ON item_views(session_id) WHERE session_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_item_categories_parent_id ON item_categories(parent_category_id) WHERE parent_category_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_item_categories_active ON item_categories(is_active) WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS idx_item_images_item_id ON item_images(item_id);
CREATE INDEX IF NOT EXISTS idx_item_images_primary ON item_images(item_id, is_primary) WHERE is_primary = TRUE;
CREATE INDEX IF NOT EXISTS idx_item_images_display_order ON item_images(item_id, display_order);

CREATE INDEX IF NOT EXISTS idx_item_favorites_user_id ON item_favorites(user_id);
CREATE INDEX IF NOT EXISTS idx_item_favorites_item_id ON item_favorites(item_id);
CREATE INDEX IF NOT EXISTS idx_item_favorites_created_at ON item_favorites(created_at);

CREATE INDEX IF NOT EXISTS idx_item_reviews_item_id ON item_reviews(item_id);
CREATE INDEX IF NOT EXISTS idx_item_reviews_user_id ON item_reviews(user_id);
CREATE INDEX IF NOT EXISTS idx_item_reviews_rating ON item_reviews(rating);
CREATE INDEX IF NOT EXISTS idx_item_reviews_approved ON item_reviews(is_approved) WHERE is_approved = TRUE;

CREATE INDEX IF NOT EXISTS idx_item_inventory_log_item_id ON item_inventory_log(item_id);
CREATE INDEX IF NOT EXISTS idx_item_inventory_log_change_type ON item_inventory_log(change_type);
CREATE INDEX IF NOT EXISTS idx_item_inventory_log_created_at ON item_inventory_log(created_at);

-- Add indexes to existing items table
CREATE INDEX IF NOT EXISTS idx_items_category ON items(category) WHERE category IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_items_retail_price ON items(retail_price);
CREATE INDEX IF NOT EXISTS idx_items_status_stock ON items(status, stock_quantity);
CREATE INDEX IF NOT EXISTS idx_items_seller_status ON items(seller_id, status) WHERE seller_id IS NOT NULL;

-- Add comments for documentation
COMMENT ON TABLE item_activity_log IS 'Logs all activities performed on items (creation, updates, status changes, etc.)';
COMMENT ON TABLE item_views IS 'Tracks item views for analytics and recommendations';
COMMENT ON TABLE item_categories IS 'Hierarchical category system for items';
COMMENT ON TABLE item_images IS 'Manages multiple images per item with ordering and metadata';
COMMENT ON TABLE item_favorites IS 'User favorites/wishlist functionality';
COMMENT ON TABLE item_reviews IS 'User reviews and ratings for items';
COMMENT ON TABLE item_inventory_log IS 'Tracks all inventory changes for audit purposes';

COMMENT ON COLUMN item_views.view_count IS 'Number of views for this item on this date (aggregated)';
COMMENT ON COLUMN item_images.is_primary IS 'Whether this is the primary/featured image for the item';
COMMENT ON COLUMN item_reviews.is_verified_purchase IS 'Whether the reviewer actually purchased/won this item';
COMMENT ON COLUMN item_inventory_log.quantity_change IS 'Positive for additions, negative for reductions';

-- Create triggers for updated_at timestamps
CREATE TRIGGER update_item_categories_updated_at BEFORE UPDATE ON item_categories FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_item_reviews_updated_at BEFORE UPDATE ON item_reviews FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert default categories
INSERT INTO item_categories (name, description, display_order) VALUES
('Electronics', 'Electronic devices and gadgets', 1),
('Fashion', 'Clothing, accessories, and fashion items', 2),
('Home & Garden', 'Home decor, furniture, and garden items', 3),
('Sports & Outdoors', 'Sports equipment and outdoor gear', 4),
('Books & Media', 'Books, movies, music, and digital media', 5),
('Toys & Games', 'Toys, games, and entertainment items', 6),
('Health & Beauty', 'Health, beauty, and personal care products', 7),
('Automotive', 'Car accessories and automotive items', 8),
('Art & Collectibles', 'Artwork, collectibles, and unique items', 9),
('Other', 'Items that don\'t fit other categories', 10)
ON CONFLICT (name) DO NOTHING;

-- Create a view for item statistics
CREATE OR REPLACE VIEW item_statistics AS
SELECT 
    i.id,
    i.name,
    i.seller_id,
    i.category,
    i.status,
    i.retail_price,
    i.stock_quantity,
    COALESCE(v.total_views, 0) as total_views,
    COALESCE(v.unique_viewers, 0) as unique_viewers,
    COALESCE(f.favorite_count, 0) as favorite_count,
    COALESCE(r.review_count, 0) as review_count,
    COALESCE(r.average_rating, 0) as average_rating,
    COALESCE(raffle_stats.raffle_count, 0) as raffle_count,
    COALESCE(raffle_stats.total_boxes_sold, 0) as total_boxes_sold,
    COALESCE(raffle_stats.total_revenue, 0) as total_revenue,
    i.created_at,
    i.updated_at
FROM items i
LEFT JOIN (
    SELECT 
        item_id,
        SUM(view_count) as total_views,
        COUNT(DISTINCT user_id) as unique_viewers
    FROM item_views 
    GROUP BY item_id
) v ON i.id = v.item_id
LEFT JOIN (
    SELECT 
        item_id,
        COUNT(*) as favorite_count
    FROM item_favorites 
    GROUP BY item_id
) f ON i.id = f.item_id
LEFT JOIN (
    SELECT 
        item_id,
        COUNT(*) as review_count,
        AVG(rating::DECIMAL) as average_rating
    FROM item_reviews 
    WHERE is_approved = TRUE
    GROUP BY item_id
) r ON i.id = r.item_id
LEFT JOIN (
    SELECT 
        item_id,
        COUNT(*) as raffle_count,
        COALESCE(SUM(boxes_sold), 0) as total_boxes_sold,
        COALESCE(SUM(boxes_sold * box_price), 0) as total_revenue
    FROM raffles 
    GROUP BY item_id
) raffle_stats ON i.id = raffle_stats.item_id;

COMMENT ON VIEW item_statistics IS 'Comprehensive statistics view for items including views, favorites, reviews, and raffle performance';

-- Create a function to get item analytics
CREATE OR REPLACE FUNCTION get_item_analytics(p_item_id UUID, p_days INTEGER DEFAULT 30)
RETURNS TABLE (
    views_last_period BIGINT,
    unique_viewers_last_period BIGINT,
    favorites_last_period BIGINT,
    conversion_rate DECIMAL,
    revenue_last_period DECIMAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        COALESCE(SUM(iv.view_count), 0) as views_last_period,
        COUNT(DISTINCT iv.user_id) as unique_viewers_last_period,
        COUNT(DISTINCT if_recent.user_id) as favorites_last_period,
        CASE 
            WHEN COALESCE(SUM(iv.view_count), 0) > 0 
            THEN (COALESCE(SUM(r.boxes_sold), 0)::DECIMAL / COALESCE(SUM(iv.view_count), 1)::DECIMAL) * 100
            ELSE 0
        END as conversion_rate,
        COALESCE(SUM(r.boxes_sold * r.box_price), 0) as revenue_last_period
    FROM items i
    LEFT JOIN item_views iv ON i.id = iv.item_id 
        AND iv.viewed_at >= CURRENT_DATE - INTERVAL '%d days'
    LEFT JOIN item_favorites if_recent ON i.id = if_recent.item_id 
        AND if_recent.created_at >= CURRENT_DATE - INTERVAL '%d days'
    LEFT JOIN raffles r ON i.id = r.item_id 
        AND r.created_at >= CURRENT_DATE - INTERVAL '%d days'
    WHERE i.id = p_item_id
    GROUP BY i.id;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION get_item_analytics IS 'Function to get item analytics for a specific time period';

-- Create a function to update item stock with logging
CREATE OR REPLACE FUNCTION update_item_stock_with_log(
    p_item_id UUID,
    p_quantity_change INTEGER,
    p_reason VARCHAR(255),
    p_user_id UUID DEFAULT NULL
) RETURNS BOOLEAN AS $$
DECLARE
    current_stock INTEGER;
    new_stock INTEGER;
BEGIN
    -- Get current stock
    SELECT stock_quantity INTO current_stock FROM items WHERE id = p_item_id;
    
    IF current_stock IS NULL THEN
        RETURN FALSE; -- Item not found
    END IF;
    
    new_stock := current_stock + p_quantity_change;
    
    -- Prevent negative stock
    IF new_stock < 0 THEN
        RETURN FALSE;
    END IF;
    
    -- Update stock
    UPDATE items SET stock_quantity = new_stock, updated_at = NOW() WHERE id = p_item_id;
    
    -- Log the change
    INSERT INTO item_inventory_log (item_id, change_type, quantity_change, previous_quantity, new_quantity, reason, user_id)
    VALUES (
        p_item_id,
        CASE 
            WHEN p_quantity_change > 0 THEN 'stock_added'
            WHEN p_quantity_change < 0 THEN 'stock_reduced'
            ELSE 'stock_adjustment'
        END,
        p_quantity_change,
        current_stock,
        new_stock,
        p_reason,
        p_user_id
    );
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION update_item_stock_with_log IS 'Function to update item stock with automatic logging';

-- Create a materialized view for popular items
CREATE MATERIALIZED VIEW IF NOT EXISTS popular_items AS
SELECT 
    i.id,
    i.name,
    i.seller_id,
    i.category,
    i.retail_price,
    i.status,
    COALESCE(v.total_views, 0) as total_views,
    COALESCE(f.favorite_count, 0) as favorite_count,
    COALESCE(r.average_rating, 0) as average_rating,
    COALESCE(raffle_stats.total_revenue, 0) as total_revenue,
    -- Popularity score calculation
    (
        COALESCE(v.total_views, 0) * 0.3 +
        COALESCE(f.favorite_count, 0) * 2.0 +
        COALESCE(r.average_rating, 0) * 10.0 +
        COALESCE(raffle_stats.raffle_count, 0) * 5.0
    ) as popularity_score,
    i.created_at
FROM items i
LEFT JOIN (
    SELECT item_id, SUM(view_count) as total_views
    FROM item_views 
    WHERE viewed_at >= CURRENT_DATE - INTERVAL '30 days'
    GROUP BY item_id
) v ON i.id = v.item_id
LEFT JOIN (
    SELECT item_id, COUNT(*) as favorite_count
    FROM item_favorites 
    WHERE created_at >= CURRENT_DATE - INTERVAL '30 days'
    GROUP BY item_id
) f ON i.id = f.item_id
LEFT JOIN (
    SELECT item_id, AVG(rating::DECIMAL) as average_rating
    FROM item_reviews 
    WHERE is_approved = TRUE AND created_at >= CURRENT_DATE - INTERVAL '30 days'
    GROUP BY item_id
) r ON i.id = r.item_id
LEFT JOIN (
    SELECT 
        item_id,
        COUNT(*) as raffle_count,
        COALESCE(SUM(boxes_sold * box_price), 0) as total_revenue
    FROM raffles 
    WHERE created_at >= CURRENT_DATE - INTERVAL '30 days'
    GROUP BY item_id
) raffle_stats ON i.id = raffle_stats.item_id
WHERE i.status = 'available'
ORDER BY popularity_score DESC;

-- Create unique index for materialized view
CREATE UNIQUE INDEX IF NOT EXISTS idx_popular_items_id ON popular_items(id);

COMMENT ON MATERIALIZED VIEW popular_items IS 'Materialized view of popular items based on views, favorites, ratings, and raffle activity';

-- Create a function to refresh popular items
CREATE OR REPLACE FUNCTION refresh_popular_items()
RETURNS VOID AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY popular_items;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION refresh_popular_items IS 'Function to refresh the popular items materialized view';