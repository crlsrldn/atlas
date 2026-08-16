-- Playback state for the Jellyfin surface.
--
-- Jellyfin clients expect the server to remember where a viewer stopped and
-- what they marked as a favourite. Atlas had nowhere to put either:
-- engines/history.rs tracks whether a *source* played, not where a *person*
-- got to, and it writes to the process working directory, which on Fly is
-- per-machine and disappears on redeploy.
--
-- Keyed on preferences.id — the install token — rather than on the user, so
-- state follows a profile the way the Stremio addon URL already does. Someone
-- with two profiles gets two sets of progress, which is the point of profiles.
--
-- Create-only: nothing existing is altered, so the Stremio path is untouched.

CREATE TABLE IF NOT EXISTS playstate (
  profile_id     UUID NOT NULL REFERENCES preferences(id) ON DELETE CASCADE,
  item_id        TEXT NOT NULL,
  -- The Atlas media key ("tt0944947:1:2"). Redundant with item_id, which
  -- encodes the same thing in Jellyfin's id format, but readable in a console.
  atlas_key      TEXT NOT NULL,
  position_ticks BIGINT NOT NULL DEFAULT 0,
  runtime_ticks  BIGINT,
  played         BOOLEAN NOT NULL DEFAULT FALSE,
  play_count     INTEGER NOT NULL DEFAULT 0,
  updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (profile_id, item_id)
);

-- Resume and Up Next read the most recently touched rows for a profile.
CREATE INDEX IF NOT EXISTS playstate_profile_recent_idx
  ON playstate (profile_id, updated_at DESC);

CREATE TABLE IF NOT EXISTS favorites (
  profile_id UUID NOT NULL REFERENCES preferences(id) ON DELETE CASCADE,
  item_id    TEXT NOT NULL,
  atlas_key  TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (profile_id, item_id)
);

CREATE INDEX IF NOT EXISTS favorites_profile_recent_idx
  ON favorites (profile_id, created_at DESC);

-- Written only by the service role, which bypasses RLS. Enabling it without
-- policies is what stops anyone reaching these with an anon key — the same
-- posture the telemetry table takes.
--
-- Read policies are deliberately omitted rather than forgotten: nothing in the
-- dashboard shows playback history today. Adding one later means joining back
-- to preferences.user_id, since profile_id is not itself a user.
ALTER TABLE playstate ENABLE ROW LEVEL SECURITY;
ALTER TABLE favorites ENABLE ROW LEVEL SECURITY;
