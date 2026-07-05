-- 1. Modify preferences to support multiple profiles
ALTER TABLE preferences 
  DROP CONSTRAINT IF EXISTS preferences_id_fkey;

ALTER TABLE preferences 
  ADD COLUMN user_id UUID REFERENCES auth.users(id) ON DELETE CASCADE;

-- Migrate existing data
UPDATE preferences SET user_id = id;

ALTER TABLE preferences 
  ALTER COLUMN user_id SET NOT NULL,
  ALTER COLUMN id SET DEFAULT gen_random_uuid(),
  ADD COLUMN profile_name TEXT NOT NULL DEFAULT 'Default Profile';

-- Drop old RLS policies
DROP POLICY IF EXISTS "Users can view their own preferences" ON preferences;
DROP POLICY IF EXISTS "Users can insert their own preferences" ON preferences;
DROP POLICY IF EXISTS "Users can update their own preferences" ON preferences;

-- Create new RLS policies
CREATE POLICY "Users can view their own preferences" 
  ON preferences FOR SELECT 
  USING (auth.uid() = user_id);

CREATE POLICY "Users can insert their own preferences" 
  ON preferences FOR INSERT 
  WITH CHECK (auth.uid() = user_id);

CREATE POLICY "Users can update their own preferences" 
  ON preferences FOR UPDATE 
  USING (auth.uid() = user_id);

CREATE POLICY "Users can delete their own preferences"
  ON preferences FOR DELETE
  USING (auth.uid() = user_id);

-- 2. Add RLS policy for admins to subscribe to telemetry
CREATE POLICY "Admins can read telemetry" 
  ON telemetry FOR SELECT 
  USING (
    EXISTS (
      SELECT 1 FROM public.profiles 
      WHERE id = auth.uid() AND role = 'admin'
    )
  );
