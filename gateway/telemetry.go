package main

import (
	"bytes"
	"encoding/json"
	"log"
	"net/http"
	"os"
)

func LogTelemetryEvent(eventName string, payload map[string]interface{}) {
	supabaseUrl := os.Getenv("SUPABASE_URL")
	serviceKey := os.Getenv("SUPABASE_SERVICE_ROLE_KEY")

	if supabaseUrl == "" || serviceKey == "" {
		return
	}

	go func() {
		url := supabaseUrl + "/rest/v1/telemetry"

		data := map[string]interface{}{
			"event_type": eventName,
			"event_data": payload,
		}

		body, err := json.Marshal(data)
		if err != nil {
			return
		}

		req, err := http.NewRequest(http.MethodPost, url, bytes.NewBuffer(body))
		if err != nil {
			return
		}

		req.Header.Set("apikey", serviceKey)
		req.Header.Set("Authorization", "Bearer "+serviceKey)
		req.Header.Set("Content-Type", "application/json")

		client := &http.Client{}
		resp, err := client.Do(req)
		if err != nil {
			log.Printf("Failed to log telemetry: %v\n", err)
			return
		}
		defer resp.Body.Close()
	}()
}
