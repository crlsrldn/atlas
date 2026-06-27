package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"
)

type AppwriteClient struct {
	Endpoint  string
	ProjectID string
	APIKey    string
	Client    *http.Client
}

func NewAppwriteClient() *AppwriteClient {
	endpoint := os.Getenv("APPWRITE_ENDPOINT")
	project := os.Getenv("APPWRITE_PROJECT_ID")
	apiKey := os.Getenv("APPWRITE_API_KEY")

	if endpoint == "" {
		endpoint = "https://cloud.appwrite.io/v1"
	}

	return &AppwriteClient{
		Endpoint:  endpoint,
		ProjectID: project,
		APIKey:    apiKey,
		Client:    &http.Client{},
	}
}

type AppwriteDocument struct {
	PrefsJson string `json:"prefs_json"`
}

func (a *AppwriteClient) GetUserPreferences(token string) (map[string]interface{}, error) {
	// Fallback/dummy if no credentials
	if a.ProjectID == "" {
		return map[string]interface{}{
			"torbox_api_key":      "",
			"real_debrid_api_key": "",
			"max_resolution":      "4K",
			"exclude_av1":         false,
		}, nil
	}

	url := fmt.Sprintf("%s/databases/atlas/collections/preferences/documents/%s", a.Endpoint, token)
	
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return nil, err
	}

	req.Header.Set("X-Appwrite-Project", a.ProjectID)
	if a.APIKey != "" {
		req.Header.Set("X-Appwrite-Key", a.APIKey)
	}

	resp, err := a.Client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("appwrite returned status: %d", resp.StatusCode)
	}

	var doc AppwriteDocument
	if err := json.NewDecoder(resp.Body).Decode(&doc); err != nil {
		return nil, err
	}

	var prefs map[string]interface{}
	if err := json.Unmarshal([]byte(doc.PrefsJson), &prefs); err != nil {
		return nil, err
	}

	return prefs, nil
}
