package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

type CinemetaClient struct {
	HTTPClient *http.Client
}

func NewCinemetaClient() *CinemetaClient {
	return &CinemetaClient{
		HTTPClient: &http.Client{
			Timeout: 10 * time.Second,
		},
	}
}

type CinemetaVideo struct {
	ID       string `json:"id"`
	Title    string `json:"title"`
	Season   int    `json:"season"`
	Episode  int    `json:"episode"`
	Released string `json:"released"`
}

type CinemetaMeta struct {
	ID     string          `json:"id"`
	Type   string          `json:"type"`
	Name   string          `json:"name"`
	Videos []CinemetaVideo `json:"videos"`
}

type CinemetaResponse struct {
	Meta CinemetaMeta `json:"meta"`
}

func (c *CinemetaClient) GetSeries(imdbID string) (*CinemetaMeta, error) {
	url := fmt.Sprintf("https://v3-cinemeta.strem.io/meta/series/%s.json", imdbID)
	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		return nil, err
	}

	resp, err := c.HTTPClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("cinemeta error: %d", resp.StatusCode)
	}

	var data CinemetaResponse
	if err := json.NewDecoder(resp.Body).Decode(&data); err != nil {
		return nil, err
	}

	return &data.Meta, nil
}
