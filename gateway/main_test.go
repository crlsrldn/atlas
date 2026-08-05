package main

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"strings"
	"testing"
)

// A stand-in for Supabase's REST API, covering the two lookups the gateway
// makes: the preferences row for an install token, and the authoritative tier
// for the user that row belongs to.
func fakeSupabase(w http.ResponseWriter, r *http.Request) {
	id := strings.TrimPrefix(r.URL.Query().Get("id"), "eq.")
	w.Header().Set("Content-Type", "application/json")

	switch {
	case strings.HasPrefix(r.URL.Path, "/rest/v1/preferences"):
		prefs, ok := map[string]map[string]interface{}{
			// This user has written is_premium into their own prefs_json, which
			// Supabase RLS permits. It must not be believed.
			"token-liar": {
				"user_id":      "user-free",
				"profile_name": "Liar",
				"prefs_json": map[string]interface{}{
					"torbox_api_key": "tb_key",
					"is_premium":     true,
				},
			},
			"token-premium": {
				"user_id":      "user-premium",
				"profile_name": "Paid",
				"prefs_json":   map[string]interface{}{"torbox_api_key": "tb_key"},
			},
			"token-no-tier-row": {
				"user_id":      "user-unknown",
				"profile_name": "New",
				"prefs_json":   map[string]interface{}{"torbox_api_key": "tb_key"},
			},
		}[id]
		if !ok {
			w.WriteHeader(http.StatusNotAcceptable)
			return
		}
		_ = json.NewEncoder(w).Encode(prefs)

	case strings.HasPrefix(r.URL.Path, "/rest/v1/app_users"):
		tier, ok := map[string]string{
			"user-free":    "free",
			"user-premium": "premium",
		}[id]
		if !ok {
			// PostgREST answers a single-object request for a missing row this way.
			w.WriteHeader(http.StatusNotAcceptable)
			return
		}
		_ = json.NewEncoder(w).Encode(map[string]string{"tier": tier})

	default:
		w.WriteHeader(http.StatusNotFound)
	}
}

func TestMain(m *testing.M) {
	server := httptest.NewServer(http.HandlerFunc(fakeSupabase))
	// Must be set before the first NewSupabaseClient call, which caches config.
	os.Setenv("SUPABASE_URL", server.URL)
	os.Setenv("SUPABASE_SERVICE_ROLE_KEY", "test-service-role-key")

	code := m.Run()

	server.Close()
	os.Exit(code)
}

func TestSelfDeclaredPremiumIsIgnored(t *testing.T) {
	prefs := loadPreferences("token-liar")

	if prefs["is_premium"] != false {
		t.Fatalf("is_premium from user-writable prefs_json was trusted: got %v, want false", prefs["is_premium"])
	}
	if prefs["torbox_api_key"] != "tb_key" {
		t.Fatalf("the rest of prefs_json should survive the override, got %v", prefs["torbox_api_key"])
	}
}

func TestAuthoritativeTierGrantsPremium(t *testing.T) {
	prefs := loadPreferences("token-premium")

	if prefs["is_premium"] != true {
		t.Fatalf("a premium tier in app_users should grant premium, got %v", prefs["is_premium"])
	}
}

func TestMissingTierRowIsNotPremium(t *testing.T) {
	prefs := loadPreferences("token-no-tier-row")

	if prefs["is_premium"] != false {
		t.Fatalf("a user with no app_users row must not be premium, got %v", prefs["is_premium"])
	}
}

func TestUnknownTokenFallsBackToFreeDefaults(t *testing.T) {
	prefs := loadPreferences("token-does-not-exist")

	if prefs["is_premium"] != false {
		t.Fatalf("failed lookups must fail closed, got %v", prefs["is_premium"])
	}
	if prefs["torbox_api_key"] != "" {
		t.Fatalf("fallback prefs should carry no credentials, got %v", prefs["torbox_api_key"])
	}
}

func TestIsPremiumFailsClosedWithoutUser(t *testing.T) {
	if NewSupabaseClient().IsPremium("") {
		t.Fatal("an empty user id must never be premium")
	}
}

func TestDefaultPreferencesAreNotPremium(t *testing.T) {
	if defaultPreferences()["is_premium"] != false {
		t.Fatal("default preferences must not grant premium")
	}
}

func TestAnonymizePathRedactsInstallToken(t *testing.T) {
	got := anonymizePath("/stremio/atl_secrettoken/stream/movie/tt0133093.json")

	if strings.Contains(got, "atl_secrettoken") {
		t.Fatalf("install token leaked into telemetry path: %s", got)
	}
	if got != "/stremio/[redacted]/stream/movie/tt0133093.json" {
		t.Fatalf("unexpected redaction: %s", got)
	}
}

func TestTelemetryDoesNotBlockWhenUnconfigured(t *testing.T) {
	url := os.Getenv("SUPABASE_URL")
	os.Setenv("SUPABASE_URL", "")
	defer os.Setenv("SUPABASE_URL", url)

	// Should return immediately rather than queueing or dialling anything.
	for i := 0; i < telemetryQueueSize*2; i++ {
		LogTelemetryEvent("test_event", map[string]interface{}{"i": i})
	}
}
