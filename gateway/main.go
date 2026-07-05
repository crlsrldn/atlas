package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/hashicorp/golang-lru/v2/expirable"
)

var coreUrl string
var dashboardUrl string

var globalConfigCache struct {
	sync.RWMutex
	MonetizationEnabled bool
	LastFetched         time.Time
}

var prefsCache = expirable.NewLRU[string, map[string]interface{}](10000, nil, time.Minute*10)

func getMonetizationEnabled() bool {
	globalConfigCache.RLock()
	cacheTime := globalConfigCache.LastFetched
	val := globalConfigCache.MonetizationEnabled
	globalConfigCache.RUnlock()

	if time.Since(cacheTime) < 1*time.Minute {
		return val
	}

	// Fetch update
	go func() {
		resp, err := http.Get(dashboardUrl + "/api/global-config")
		if err == nil {
			defer resp.Body.Close()
			var data struct {
				MonetizationEnabled bool `json:"monetization_enabled"`
			}
			if err := json.NewDecoder(resp.Body).Decode(&data); err == nil {
				globalConfigCache.Lock()
				globalConfigCache.MonetizationEnabled = data.MonetizationEnabled
				globalConfigCache.LastFetched = time.Now()
				globalConfigCache.Unlock()
			}
		}
	}()

	return val
}

func main() {
	coreUrl = os.Getenv("ATLAS_CORE_URL")
	if coreUrl == "" {
		coreUrl = "http://127.0.0.1:3000"
	}

	dashboardUrl = os.Getenv("ATLAS_PUBLIC_BASE_URL")
	if dashboardUrl == "" {
		dashboardUrl = "http://127.0.0.1:3000"
	}

	http.HandleFunc("/", handleRoot)
	http.HandleFunc("/health", handleRoot)
	http.HandleFunc("/stremio/", handleStremio)

	log.Println("Starting API gateway on :8080")
	if err := http.ListenAndServe(":8080", telemetryMiddleware(http.DefaultServeMux)); err != nil {
		log.Fatalf("Server failed: %v", err)
	}
}

type customResponseWriter struct {
	http.ResponseWriter
	statusCode int
}

func (rw *customResponseWriter) WriteHeader(code int) {
	rw.statusCode = code
	rw.ResponseWriter.WriteHeader(code)
}

func anonymizePath(p string) string {
	if strings.HasPrefix(p, "/stremio/") {
		parts := strings.Split(p, "/")
		if len(parts) >= 4 {
			// parts: ["", "stremio", "TOKEN", "stream", ...]
			parts[2] = "[redacted]"
			return strings.Join(parts, "/")
		}
	}
	return p
}

func telemetryMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		start := time.Now()

		crw := &customResponseWriter{ResponseWriter: w, statusCode: http.StatusOK}

		next.ServeHTTP(crw, r)

		latency := time.Since(start).Milliseconds()

		ua := r.Header.Get("User-Agent")
		uaType := "unknown"
		if strings.Contains(strings.ToLower(ua), "stremio") {
			uaType = "stremio"
		} else if strings.Contains(strings.ToLower(ua), "mozilla") || strings.Contains(strings.ToLower(ua), "chrome") || strings.Contains(strings.ToLower(ua), "safari") {
			uaType = "browser"
		} else if ua != "" {
			uaType = "other"
		}

		LogTelemetryEvent("gateway_request", map[string]interface{}{
			"path":            anonymizePath(r.URL.Path),
			"status_code":     crw.statusCode,
			"latency_ms":      latency,
			"user_agent_type": uaType,
		})
	})
}

func handleRoot(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodOptions {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Methods", "OPTIONS, LOCK, DELETE, PROPPATCH, COPY, MOVE, UNLOCK, PROPFIND, GET, HEAD")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization, Depth")
		w.Header().Set("Dav", "1, 2")
		w.Header().Set("Ms-Author-Via", "DAV")
		w.Header().Set("Allow", "OPTIONS, LOCK, DELETE, PROPPATCH, COPY, MOVE, UNLOCK, PROPFIND, GET, HEAD")
		w.WriteHeader(http.StatusOK)
		return
	}

	if r.URL.Path != "/" && r.URL.Path != "/health" {
		http.NotFound(w, r)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	w.Write([]byte(`{"status": "ok", "service": "cindral-atlas-gateway"}`))
}

func handleStremio(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodGet && r.Method != http.MethodHead && r.Method != http.MethodOptions {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
		return
	}

	if r.Method == http.MethodOptions {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Methods", "GET, HEAD, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization")
		w.WriteHeader(http.StatusOK)
		return
	}

	path := strings.TrimPrefix(r.URL.Path, "/stremio/")
	parts := strings.SplitN(path, "/", 2)
	if len(parts) < 2 {
		http.NotFound(w, r)
		return
	}

	token := parts[0]
	rest := parts[1]

	if rest == "manifest.json" {
		handleManifest(w, r, token)
		return
	}

	if strings.HasPrefix(rest, "stream/") {
		handleStream(w, r, token, rest)
		return
	}

	if strings.HasPrefix(rest, "resolve/") {
		handleResolve(w, r, token, rest)
		return
	}

	http.NotFound(w, r)
}

func handleManifest(w http.ResponseWriter, r *http.Request, token string) {
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Access-Control-Allow-Origin", "*")

	addonName := "Atlas"
	supabase := NewSupabaseClient()
	if doc, err := supabase.GetUserPreferences(token); err == nil && doc.ProfileName != "" {
		addonName = "Atlas - " + doc.ProfileName
	}

	manifest := map[string]interface{}{
		"id":          "com.cindrallabs.atlas." + token,
		"name":        addonName,
		"version":     "1.0.0",
		"description": "Premium AI-powered multi-source streaming",
		"resources":   []string{"stream"},
		"types":       []string{"movie", "series"},
		"idPrefixes":  []string{"tt"},
	}

	json.NewEncoder(w).Encode(manifest)
}

func handleStream(w http.ResponseWriter, r *http.Request, token, rest string) {
	parts := strings.Split(rest, "/")
	if len(parts) != 3 {
		http.NotFound(w, r)
		return
	}

	idParam := parts[2]
	if !strings.HasSuffix(idParam, ".json") {
		http.NotFound(w, r)
		return
	}
	id := strings.TrimSuffix(idParam, ".json")

	supabase := NewSupabaseClient()
	var prefs map[string]interface{}

	if cachedPrefs, ok := prefsCache.Get(token); ok {
		prefs = cachedPrefs
	} else {
		doc, err := supabase.GetUserPreferences(token)
		if err != nil {
			log.Printf("Failed to fetch user preferences from Supabase for token %s: %v", token, err)
			// Fallback to empty prefs or handle error appropriately.
			// For MVP, we will send empty strings if fetch fails so the Rust core won't crash
			prefs = map[string]interface{}{
				"torbox_api_key":  "",
				"trakt_client_id": "",
				"trakt_username":  "",
				"max_resolution":  "4K",
				"exclude_av1":     false,
				"sort_preference": "balanced",
				"stream_limit":    5,
				"is_premium":      false,
			}
		} else {
			prefs = doc.PrefsJson
			prefsCache.Add(token, prefs)
		}
	}

	userAgent := r.Header.Get("User-Agent")

	reqBody, _ := json.Marshal(map[string]interface{}{
		"stremio_id":           id,
		"install_token":        token,
		"prefs":                prefs,
		"user_agent":           userAgent,
		"monetization_enabled": getMonetizationEnabled(),
	})

	resp, err := http.Post(coreUrl+"/internal/resolve", "application/json", bytes.NewBuffer(reqBody))
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	defer resp.Body.Close()

	w.Header().Set("Content-Type", resp.Header.Get("Content-Type"))
	w.Header().Set("Access-Control-Allow-Origin", "*")
	w.WriteHeader(resp.StatusCode)
	io.Copy(w, resp.Body)
}

func handleResolve(w http.ResponseWriter, r *http.Request, token, rest string) {
	parts := strings.Split(rest, "/")
	if len(parts) < 3 {
		http.NotFound(w, r)
		return
	}
	provider := parts[1]
	hash := parts[2]

	url := fmt.Sprintf("%s/internal/resolve_hash/%s/%s", coreUrl, provider, hash)

	supabase := NewSupabaseClient()
	var prefs map[string]interface{}
	doc, err := supabase.GetUserPreferences(token)
	if err != nil {
		log.Printf("Failed to fetch user preferences from Supabase for token %s: %v", token, err)
		// Fallback to empty prefs or handle error appropriately.
		prefs = map[string]interface{}{
			"torbox_api_key":  "",
			"trakt_client_id": "",
			"trakt_username":  "",
			"max_resolution":  "4K",
			"exclude_av1":     false,
		}
	} else {
		prefs = doc.PrefsJson
	}

	userAgent := r.Header.Get("User-Agent")

	payload := map[string]interface{}{
		"prefs":                prefs,
		"user_agent":           userAgent,
		"monetization_enabled": getMonetizationEnabled(),
		"install_token":        token,
	}

	// Forward season and episode if present
	if season := r.URL.Query().Get("season"); season != "" {
		if s, err := strconv.Atoi(season); err == nil {
			payload["season"] = s
		}
	}
	if episode := r.URL.Query().Get("episode"); episode != "" {
		if e, err := strconv.Atoi(episode); err == nil {
			payload["episode"] = e
		}
	}
	if cached := r.URL.Query().Get("cached"); cached != "" {
		if c, err := strconv.ParseBool(cached); err == nil {
			payload["cached"] = c
		}
	}

	reqBody, _ := json.Marshal(payload)

	req, err := http.NewRequest(http.MethodPost, url, bytes.NewBuffer(reqBody))
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{
		CheckRedirect: func(req *http.Request, via []*http.Request) error {
			return http.ErrUseLastResponse
		},
	}

	resp, err := client.Do(req)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	defer resp.Body.Close()

	for k, v := range resp.Header {
		for _, val := range v {
			w.Header().Add(k, val)
		}
	}
	w.Header().Set("Access-Control-Allow-Origin", "*")
	w.WriteHeader(resp.StatusCode)
	io.Copy(w, resp.Body)
}
