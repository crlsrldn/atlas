package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"os"
	"sync"
	"time"
)

type SupabaseClient struct {
	Endpoint       string
	ServiceRoleKey string
	Client         *http.Client
}

var globalSupabaseClient *SupabaseClient
var supabaseClientOnce sync.Once

func NewSupabaseClient() *SupabaseClient {
	supabaseClientOnce.Do(func() {
		endpoint := os.Getenv("SUPABASE_URL")
		key := os.Getenv("SUPABASE_SERVICE_ROLE_KEY")

		t := http.DefaultTransport.(*http.Transport).Clone()
		t.MaxIdleConns = 100
		t.MaxConnsPerHost = 100
		t.MaxIdleConnsPerHost = 100

		globalSupabaseClient = &SupabaseClient{
			Endpoint:       endpoint,
			ServiceRoleKey: key,
			Client: &http.Client{
				Timeout:   5 * time.Second,
				Transport: t,
			},
		}
	})
	return globalSupabaseClient
}

type SupabasePreferenceDoc struct {
	PrefsJson   map[string]interface{} `json:"prefs_json"`
	ProfileName string                 `json:"profile_name"`
	UserId      string                 `json:"user_id"`
}

func (s *SupabaseClient) GetUserPreferences(token string) (*SupabasePreferenceDoc, error) {
	if s.Endpoint == "" || s.ServiceRoleKey == "" {
		return nil, fmt.Errorf("supabase not configured")
	}

	url := fmt.Sprintf("%s/rest/v1/preferences?id=eq.%s&select=*", s.Endpoint, token)
	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		return nil, err
	}

	req.Header.Set("apikey", s.ServiceRoleKey)
	req.Header.Set("Authorization", "Bearer "+s.ServiceRoleKey)
	req.Header.Set("Content-Type", "application/json")
	// Request single object instead of array
	req.Header.Set("Accept", "application/vnd.pgrst.object+json")

	resp, err := s.Client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("failed to fetch preferences, status code: %d", resp.StatusCode)
	}

	var doc SupabasePreferenceDoc
	if err := json.NewDecoder(resp.Body).Decode(&doc); err != nil {
		return nil, err
	}

	return &doc, nil
}

type supabaseUserTierDoc struct {
	Tier string `json:"tier"`
}

// IsPremium reports whether a user is entitled to premium streams.
//
// The tier lives in app_users, which RLS lets a user read but not write. It is
// deliberately NOT read from preferences.prefs_json: users can update their own
// preferences row, so anything in that blob is user-controlled input.
//
// Fails closed — any lookup error yields free tier.
func (s *SupabaseClient) IsPremium(userId string) bool {
	if userId == "" || s.Endpoint == "" || s.ServiceRoleKey == "" {
		return false
	}

	url := fmt.Sprintf("%s/rest/v1/app_users?id=eq.%s&select=tier", s.Endpoint, userId)
	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		return false
	}

	req.Header.Set("apikey", s.ServiceRoleKey)
	req.Header.Set("Authorization", "Bearer "+s.ServiceRoleKey)
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Accept", "application/vnd.pgrst.object+json")

	resp, err := s.Client.Do(req)
	if err != nil {
		log.Printf("Failed to fetch tier for user %s: %v", userId, err)
		return false
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		// No app_users row yet is normal for a new signup, and means free.
		if resp.StatusCode != http.StatusNotAcceptable && resp.StatusCode != http.StatusNotFound {
			log.Printf("Unexpected status fetching tier for user %s: %d", userId, resp.StatusCode)
		}
		return false
	}

	var doc supabaseUserTierDoc
	if err := json.NewDecoder(resp.Body).Decode(&doc); err != nil {
		return false
	}

	return doc.Tier != "" && doc.Tier != "free"
}
