package main

import (
	"bytes"
	"encoding/json"
	"log"
	"net/http"
	"os"
	"sync"
	"time"
)

type telemetryEvent struct {
	EventType string                 `json:"event_type"`
	EventData map[string]interface{} `json:"event_data"`
}

// Telemetry runs on a fixed set of workers behind a bounded queue. Every
// gateway request emits an event, so an unbounded goroutine per event would let
// a traffic burst pile up goroutines and TLS handshakes without limit. Dropping
// events is the correct failure mode here: telemetry must never slow down or
// destabilise playback.
const (
	telemetryQueueSize = 512
	telemetryWorkers   = 4
)

var (
	telemetryQueue   chan telemetryEvent
	telemetryOnce    sync.Once
	telemetryDropped uint64
	telemetryMu      sync.Mutex

	// One pooled client for all telemetry, rather than a fresh client (and so a
	// fresh TCP + TLS handshake) per event.
	telemetryClient = &http.Client{Timeout: 5 * time.Second}
)

func startTelemetryWorkers() {
	telemetryQueue = make(chan telemetryEvent, telemetryQueueSize)

	for i := 0; i < telemetryWorkers; i++ {
		go func() {
			for event := range telemetryQueue {
				postTelemetryEvent(event)
			}
		}()
	}
}

func LogTelemetryEvent(eventName string, payload map[string]interface{}) {
	if os.Getenv("SUPABASE_URL") == "" || os.Getenv("SUPABASE_SERVICE_ROLE_KEY") == "" {
		return
	}

	telemetryOnce.Do(startTelemetryWorkers)

	select {
	case telemetryQueue <- telemetryEvent{EventType: eventName, EventData: payload}:
	default:
		recordTelemetryDrop()
	}
}

func recordTelemetryDrop() {
	telemetryMu.Lock()
	telemetryDropped++
	dropped := telemetryDropped
	telemetryMu.Unlock()

	// Only mention it occasionally — a log line per drop would be its own flood.
	if dropped%100 == 1 {
		log.Printf("Telemetry queue full, dropped %d events so far", dropped)
	}
}

func postTelemetryEvent(event telemetryEvent) {
	supabaseUrl := os.Getenv("SUPABASE_URL")
	serviceKey := os.Getenv("SUPABASE_SERVICE_ROLE_KEY")
	if supabaseUrl == "" || serviceKey == "" {
		return
	}

	body, err := json.Marshal(event)
	if err != nil {
		return
	}

	req, err := http.NewRequest(http.MethodPost, supabaseUrl+"/rest/v1/telemetry", bytes.NewBuffer(body))
	if err != nil {
		return
	}

	req.Header.Set("apikey", serviceKey)
	req.Header.Set("Authorization", "Bearer "+serviceKey)
	req.Header.Set("Content-Type", "application/json")

	resp, err := telemetryClient.Do(req)
	if err != nil {
		log.Printf("Failed to log telemetry: %v", err)
		return
	}
	defer resp.Body.Close()
}
