ALTER TABLE business_profile ADD COLUMN IF NOT EXISTS always_request_overcapture BOOLEAN NOT NULL DEFAULT FALSE;