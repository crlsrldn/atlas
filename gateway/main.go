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

	"golang.org/x/net/webdav"
)

var coreUrl string

func main() {
	coreUrl = os.Getenv("ATLAS_CORE_URL")
	if coreUrl == "" {
		coreUrl = "http://127.0.0.1:3000"
	}

	http.HandleFunc("/", handleRoot)
	http.HandleFunc("/health", handleRoot)
	http.HandleFunc("/stremio/", handleStremio)
	http.HandleFunc("/webdav/", handleWebDAV)

	log.Println("Starting API gateway on :8080")
	if err := http.ListenAndServe(":8080", nil); err != nil {
		log.Fatalf("Server failed: %v", err)
	}
}

func handleRoot(w http.ResponseWriter, r *http.Request) {
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
		handleManifest(w, r)
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

func handleManifest(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Access-Control-Allow-Origin", "*")

	manifest := map[string]interface{}{
		"id":          "com.cindrallabs.atlas",
		"name":        "Atlas",
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
	prefs, err := supabase.GetUserPreferences(token)
	if err != nil {
		log.Printf("Failed to fetch user preferences from Supabase for token %s: %v", token, err)
		// Fallback to empty prefs or handle error appropriately.
		// For MVP, we will send empty strings if fetch fails so the Rust core won't crash
		prefs = map[string]interface{}{
			"torbox_api_key":      "",
			"real_debrid_api_key": "",
			"trakt_client_id":     "",
			"trakt_username":      "",
			"max_resolution":      "4K",
			"exclude_av1":         false,
		}
	}

	userAgent := r.Header.Get("User-Agent")

	reqBody, _ := json.Marshal(map[string]interface{}{
		"stremio_id":    id,
		"install_token": token,
		"prefs":         prefs,
		"user_agent":    userAgent,
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
	prefs, err := supabase.GetUserPreferences(token)
	if err != nil {
		log.Printf("Failed to fetch user preferences from Supabase for token %s: %v", token, err)
		// Fallback to empty prefs or handle error appropriately.
		prefs = map[string]interface{}{
			"torbox_api_key":      "",
			"real_debrid_api_key": "",
			"trakt_client_id":     "",
			"trakt_username":      "",
			"max_resolution":      "4K",
			"exclude_av1":         false,
		}
	}

	userAgent := r.Header.Get("User-Agent")

	payload := map[string]interface{}{
		"prefs":      prefs,
		"user_agent": userAgent,
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

func handleWebDAV(w http.ResponseWriter, r *http.Request) {
	path := strings.TrimPrefix(r.URL.Path, "/webdav/")
	parts := strings.SplitN(path, "/", 2)
	if len(parts) < 1 {
		http.NotFound(w, r)
		return
	}

	token := parts[0]
	// The prefix the WebDAV handler needs to strip
	prefix := "/webdav/" + token

	supabase := NewSupabaseClient()
	prefs, err := supabase.GetUserPreferences(token)
	if err != nil {
		log.Printf("Failed to fetch user preferences for WebDAV token %s: %v", token, err)
		prefs = map[string]interface{}{}
	}

	traktClientID := ""
	if id, ok := prefs["trakt_client_id"].(string); ok {
		traktClientID = id
	}

	fs := &AtlasFS{
		Token:    token,
		Prefs:    prefs,
		Trakt:    NewTraktClient(traktClientID),
		Cinemeta: NewCinemetaClient(),
	}

	handler := &webdav.Handler{
		Prefix:     prefix,
		FileSystem: fs,
		LockSystem: webdav.NewMemLS(),
		Logger: func(r *http.Request, err error) {
			if err != nil {
				log.Printf("WebDAV [%s]: %s, ERROR: %s\n", r.Method, r.URL, err)
			}
		},
	}

	handler.ServeHTTP(w, r)
}
