-- Migration 028: Plan change requests for manual admin review
CREATE TABLE IF NOT EXISTS auth.plan_change_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES auth.users(id) ON DELETE CASCADE,
    current_plan VARCHAR(32) NOT NULL,
    requested_plan VARCHAR(32) NOT NULL,
    status VARCHAR(32) NOT NULL DEFAULT 'pending', -- pending, approved, rejected
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_at TIMESTAMPTZ,
    reviewed_by VARCHAR(64)
);

CREATE INDEX IF NOT EXISTS idx_plan_req_user_status ON auth.plan_change_requests(user_id, status);
CREATE INDEX IF NOT EXISTS idx_plan_req_created ON auth.plan_change_requests(created_at DESC);

-- Grant permissions to atlsd user
GRANT ALL PRIVILEGES ON TABLE auth.plan_change_requests TO atlsd;

-- Also provide a view in public schema if appropriate
CREATE OR REPLACE VIEW public.plan_change_requests AS SELECT * FROM auth.plan_change_requests;
GRANT ALL PRIVILEGES ON TABLE public.plan_change_requests TO atlsd;
