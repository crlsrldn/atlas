package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"time"
)

type SupabaseClient struct {
	Endpoint       string
	ServiceRoleKey string
	Client         *http.Client
}

func NewSupabaseClient() *SupabaseClient {
	endpoint := os.Getenv("SUPABASE_URL")
	key := os.Getenv("SUPABASE_SERVICE_ROLE_KEY")

	return &SupabaseClient{
		Endpoint:       endpoint,
		ServiceRoleKey: key,
		Client: &http.Client{
			Timeout: 5 * time.Second,
		},
	}
}

type SupabasePreferenceDoc struct {
	PrefsJson map[string]interface{} `json:"prefs_json"`
}

func (s *SupabaseClient) GetUserPreferences(token string) (map[string]interface{}, error) {
	if s.Endpoint == "" || s.ServiceRoleKey == "" {
		return nil, fmt.Errorf("supabase not configured")
	}

	url := fmt.Sprintf("%s/rest/v1/preferences?id=eq.%s&select=prefs_json", s.Endpoint, token)
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

	return doc.PrefsJson, nil
}
