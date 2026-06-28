package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"strings"
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
	if r.Method != http.MethodGet {
		http.Error(w, "Method not allowed", http.StatusMethodNotAllowed)
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
			"max_resolution":      "4K",
			"exclude_av1":         false,
		}
	}

	reqBody, _ := json.Marshal(map[string]interface{}{
		"stremio_id":    id,
		"install_token": token,
		"prefs":         prefs,
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
	if len(parts) != 3 {
		http.NotFound(w, r)
		return
	}
	provider := parts[1]
	hash := parts[2]

	url := fmt.Sprintf("%s/internal/resolve_hash/%s/%s", coreUrl, provider, hash)

	client := &http.Client{
		CheckRedirect: func(req *http.Request, via []*http.Request) error {
			return http.ErrUseLastResponse
		},
	}

	resp, err := client.Get(url)
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
