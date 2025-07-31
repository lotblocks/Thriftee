-- Add performance optimizations and additional constraints

-- Add composite indexes for common query patterns
CREATE INDEX idx_raffles_status_created_at ON raffles(status, created_at);
CREATE INDEX idx_box_purchases_user_raffle ON box_purchases(user_id, raffle_id);
CREATE INDEX idx_user_credits_user_expires ON user_credits(user_id, expires_at) WHERE is_used = FALSE;
CREATE INDEX idx_transactions_user_type_created ON transactions(user_id, type, created_at);
CREATE INDEX idx_items_seller_status_created ON items(seller_id, status, created_at);

-- Add partial indexes for active records
CREATE INDEX idx_users_active_email ON users(email) WHERE is_active = TRUE;
CREATE INDEX idx_sellers_verified ON sellers(user_id) WHERE is_verified = TRUE;
CREATE INDEX idx_raffles_open ON raffles(created_at) WHERE status = 'open';

-- Add check constraints for data integrity
ALTER TABLE raffles ADD CONSTRAINT check_raffles_boxes_sold 
    CHECK (boxes_sold >= 0 AND boxes_sold <= total_boxes);

ALTER TABLE raffles ADD CONSTRAINT check_raffles_total_winners 
    CHECK (total_winners > 0 AND total_winners <= total_boxes);

ALTER TABLE raffles ADD CONSTRAINT check_raffles_grid_dimensions 
    CHECK (grid_rows > 0 AND grid_cols > 0 AND grid_rows * grid_cols >= total_boxes);

ALTER TABLE box_purchases ADD CONSTRAINT check_box_purchases_price 
    CHECK (purchase_price_in_credits > 0);

ALTER TABLE user_credits ADD CONSTRAINT check_user_credits_amount 
    CHECK (amount > 0);

ALTER TABLE items ADD CONSTRAINT check_items_prices 
    CHECK (retail_price > 0 AND cost_of_goods >= 0);

ALTER TABLE users ADD CONSTRAINT check_users_credit_balance 
    CHECK (credit_balance >= 0);

-- Add function to automatically update credit balance
CREATE OR REPLACE FUNCTION update_user_credit_balance()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        -- Add credits
        UPDATE users 
        SET credit_balance = credit_balance + NEW.amount 
        WHERE id = NEW.user_id;
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        -- Handle credit usage
        IF OLD.is_used = FALSE AND NEW.is_used = TRUE THEN
            UPDATE users 
            SET credit_balance = credit_balance - OLD.amount 
            WHERE id = OLD.user_id;
        ELSIF OLD.is_used = TRUE AND NEW.is_used = FALSE THEN
            UPDATE users 
            SET credit_balance = credit_balance + OLD.amount 
            WHERE id = OLD.user_id;
        END IF;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        -- Remove unused credits
        IF OLD.is_used = FALSE THEN
            UPDATE users 
            SET credit_balance = credit_balance - OLD.amount 
            WHERE id = OLD.user_id;
        END IF;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for automatic credit balance updates
CREATE TRIGGER trigger_update_user_credit_balance
    AFTER INSERT OR UPDATE OR DELETE ON user_credits
    FOR EACH ROW EXECUTE FUNCTION update_user_credit_balance();

-- Add function to validate raffle completion
CREATE OR REPLACE FUNCTION validate_raffle_completion()
RETURNS TRIGGER AS $$
BEGIN
    -- Check if raffle is full when a box is purchased
    IF NEW.boxes_sold = NEW.total_boxes AND OLD.boxes_sold < NEW.total_boxes THEN
        NEW.status = 'full';
        NEW.started_at = NOW();
    END IF;
    
    -- Prevent modifications to completed raffles
    IF OLD.status IN ('completed', 'cancelled') AND NEW.status != OLD.status THEN
        RAISE EXCEPTION 'Cannot modify completed or cancelled raffle';
    END IF;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for raffle validation
CREATE TRIGGER trigger_validate_raffle_completion
    BEFORE UPDATE ON raffles
    FOR EACH ROW EXECUTE FUNCTION validate_raffle_completion();