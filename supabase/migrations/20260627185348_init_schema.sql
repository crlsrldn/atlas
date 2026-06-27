CREATE TABLE preferences (
  id UUID PRIMARY KEY REFERENCES auth.users(id) ON DELETE CASCADE,
  prefs_json JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

ALTER TABLE preferences ENABLE ROW LEVEL SECURITY;

CREATE POLICY "Users can view their own preferences" 
  ON preferences FOR SELECT 
  USING (auth.uid() = id);

CREATE POLICY "Users can insert their own preferences" 
  ON preferences FOR INSERT 
  WITH CHECK (auth.uid() = id);

CREATE POLICY "Users can update their own preferences" 
  ON preferences FOR UPDATE 
  USING (auth.uid() = id);

CREATE TABLE telemetry (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_type TEXT NOT NULL,
  event_data JSONB NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Telemetry is strictly internal, so no RLS policies are needed for public users.
-- The Service Role Key bypasses RLS anyway.
ALTER TABLE telemetry ENABLE ROW LEVEL SECURITY;
