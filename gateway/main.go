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

// resolvedProfile is everything an install token buys: the preferences Atlas
// will act on, and the profile's display name.
type resolvedProfile struct {
	Prefs       map[string]interface{}
	ProfileName string
}

var prefsCache = expirable.NewLRU[string, resolvedProfile](10000, nil, time.Minute*10)

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

	// ATLAS_PUBLIC_BASE_URL means the public GATEWAY url in core, but meant the
	// DASHBOARD url here — a collision that already left this pointing at
	// localhost in production, silently breaking the global-config fetch. The
	// old name is still read so a stale Fly secret keeps working.
	dashboardUrl = os.Getenv("ATLAS_DASHBOARD_URL")
	if dashboardUrl == "" {
		dashboardUrl = os.Getenv("ATLAS_PUBLIC_BASE_URL")
	}
	if dashboardUrl == "" {
		log.Println("Neither ATLAS_DASHBOARD_URL nor ATLAS_PUBLIC_BASE_URL is set — global config will fall back to defaults")
		dashboardUrl = "http://127.0.0.1:3000"
	}

	http.HandleFunc("/", handleRoot)
	http.HandleFunc("/health", handleRoot)
	http.HandleFunc("/logo.svg", handleLogo)
	http.HandleFunc("/stremio/", handleStremio)
	// Jellyfin clients are configured with the prefix as part of the server URL,
	// so the token never appears in a path. Official Jellyfin answers on both.
	http.HandleFunc("/jellyfin/", handleJellyfin)
	http.HandleFunc("/emby/", handleJellyfin)

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
	// Only a path is passed today, but Jellyfin clients put credentials in
	// api_key query parameters, so a caller that ever hands over a full request
	// URI must not leak one into telemetry.
	if cut := strings.IndexByte(p, '?'); cut >= 0 {
		p = p[:cut]
	}

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
		} else if strings.Contains(strings.ToLower(ua), "infuse") {
			// Infuse announces its connection mode here (Infuse-Direct,
			// Infuse-Library, Infuse-Download), so keep it whole: comparing the
			// two surfaces is how a Stremio regression becomes visible.
			uaType = strings.ToLower(ua)
			if index := strings.Index(uaType, "/"); index > 0 {
				uaType = uaType[:index]
			}
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

// jellyfinPublicRoutes are answered before a client holds any credential.
// /System/Info/Public in particular is how a client decides the URL it was given
// is a media server at all — gate it and the user never reaches a login form.
var jellyfinPublicRoutes = map[string]bool{
	"/system/info/public":     true,
	"/users/public":           true,
	"/quickconnect/enabled":   true,
	"/system/endpoint":        true,
	"/branding/configuration": true,
	"/branding/css":           true,
}

// Headers the gateway alone may set. Any client-supplied copy is discarded
// before forwarding, so a caller cannot present itself as a resolved profile.
var atlasInjectedHeaders = []string{
	"X-Atlas-Token",
	"X-Atlas-Prefs",
	"X-Atlas-Profile-Name",
	"X-Atlas-Monetization",
}

// normalizeJellyfinPath drops a trailing slash.
//
// Infuse asks for a container's children with "/Items/?ParentId=…", and the
// core router treats "/Items/" and "/Items" as different routes, so the request
// fell through to the not-implemented fallback. That one difference emptied the
// Movies library and broke opening a series, which are the same request.
func normalizeJellyfinPath(path string) string {
	for len(path) > 1 && strings.HasSuffix(path, "/") {
		path = path[:len(path)-1]
	}
	return path
}

func jellyfinRoute(path string) string {
	path = normalizeJellyfinPath(path)
	for _, prefix := range []string{"/jellyfin", "/emby"} {
		if strings.HasPrefix(path, prefix) {
			return strings.TrimPrefix(path, prefix)
		}
	}
	return path
}

// jellyfinToken finds the install token a client is presenting. Infuse sends it
// as a header once authenticated, and inside the body on the login call itself.
func jellyfinToken(r *http.Request, body []byte) string {
	for _, header := range []string{"X-Emby-Token", "X-MediaBrowser-Token"} {
		if value := strings.TrimSpace(r.Header.Get(header)); value != "" {
			return value
		}
	}

	for _, header := range []string{"X-Emby-Authorization", "Authorization"} {
		if token := tokenFromAuthorization(r.Header.Get(header)); token != "" {
			return token
		}
	}

	// Jellyfin clients append api_key to image and stream URLs.
	if value := strings.TrimSpace(r.URL.Query().Get("api_key")); value != "" {
		return value
	}

	if len(body) > 0 {
		var login struct {
			Pw       string `json:"Pw"`
			Password string `json:"Password"`
		}
		if err := json.Unmarshal(body, &login); err == nil {
			if token := strings.TrimSpace(login.Pw); token != "" {
				return token
			}
			if token := strings.TrimSpace(login.Password); token != "" {
				return token
			}
		}
	}

	return ""
}

// tokenFromAuthorization reads Token="…" out of
// `MediaBrowser Client="Infuse-Direct", Device="Apple TV", Token="…"`.
func tokenFromAuthorization(value string) string {
	if value == "" {
		return ""
	}
	for _, pair := range strings.Split(value, ",") {
		key, raw, found := strings.Cut(pair, "=")
		if !found {
			continue
		}
		if !strings.EqualFold(strings.TrimSpace(key), "Token") {
			continue
		}
		if token := strings.TrimSpace(strings.Trim(strings.TrimSpace(raw), `"`)); token != "" {
			return token
		}
	}
	return ""
}

func writeJellyfinCORS(w http.ResponseWriter) {
	w.Header().Set("Access-Control-Allow-Origin", "*")
	w.Header().Set("Access-Control-Allow-Methods", "GET, POST, DELETE, HEAD, OPTIONS")
	w.Header().Set("Access-Control-Allow-Headers",
		"Content-Type, Authorization, X-Emby-Token, X-Emby-Authorization, X-MediaBrowser-Token, Range")
}

func respondJellyfinUnauthorized(w http.ResponseWriter) {
	writeJellyfinCORS(w)
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(http.StatusUnauthorized)
	w.Write([]byte(`{"Error":"Unauthorized","Message":"Enter your Atlas install token as the password."}`))
}

// handleJellyfin proxies the Jellyfin surface to core, resolving the install
// token here so core never needs to know about Supabase. It mirrors
// handleResolve in refusing to follow redirects: a 302 is the answer, and the
// player has to receive it in order to fetch bytes straight from the CDN.
func handleJellyfin(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodOptions {
		writeJellyfinCORS(w)
		w.WriteHeader(http.StatusOK)
		return
	}

	var body []byte
	if r.Body != nil {
		read, err := io.ReadAll(io.LimitReader(r.Body, 1<<20))
		if err != nil {
			http.Error(w, "Unable to read request body", http.StatusBadRequest)
			return
		}
		r.Body.Close()
		body = read
	}

	route := strings.ToLower(jellyfinRoute(r.URL.Path))
	token := jellyfinToken(r, body)

	profile := resolvedProfile{Prefs: defaultPreferences(), ProfileName: "Atlas"}
	if token != "" {
		resolved, ok := resolveProfile(token)
		if !ok {
			respondJellyfinUnauthorized(w)
			return
		}
		profile = resolved
	} else if !jellyfinPublicRoutes[route] {
		respondJellyfinUnauthorized(w)
		return
	}

	target := coreUrl + normalizeJellyfinPath(r.URL.Path)
	if r.URL.RawQuery != "" {
		target += "?" + r.URL.RawQuery
	}

	req, err := http.NewRequest(r.Method, target, bytes.NewReader(body))
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	for name, values := range r.Header {
		for _, value := range values {
			req.Header.Add(name, value)
		}
	}
	for _, header := range atlasInjectedHeaders {
		req.Header.Del(header)
	}

	if token != "" {
		prefsJson, err := json.Marshal(profile.Prefs)
		if err != nil {
			http.Error(w, "Unable to encode preferences", http.StatusInternalServerError)
			return
		}
		req.Header.Set("X-Atlas-Token", token)
		req.Header.Set("X-Atlas-Prefs", string(prefsJson))
		// Ranking takes this as an input, so both surfaces have to agree on it.
		req.Header.Set("X-Atlas-Monetization", strconv.FormatBool(getMonetizationEnabled()))
		if profile.ProfileName != "" {
			req.Header.Set("X-Atlas-Profile-Name", profile.ProfileName)
		}
	}

	client := &http.Client{
		CheckRedirect: func(req *http.Request, via []*http.Request) error {
			return http.ErrUseLastResponse
		},
	}

	resp, err := client.Do(req)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadGateway)
		return
	}
	defer resp.Body.Close()

	for name, values := range resp.Header {
		for _, value := range values {
			w.Header().Add(name, value)
		}
	}
	writeJellyfinCORS(w)
	w.WriteHeader(resp.StatusCode)
	io.Copy(w, resp.Body)
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

const atlasLogoSVG = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">
  <defs>
    <linearGradient id="grad" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#12141b" />
      <stop offset="100%" stop-color="#04BF8A" />
    </linearGradient>
  </defs>
  <rect width="256" height="256" rx="50" fill="url(#grad)" />
  <path d="M128 50 L208 190 L48 190 Z" fill="none" stroke="#FFFFFF" stroke-width="20" stroke-linejoin="round" />
  <circle cx="128" cy="140" r="24" fill="#FFFFFF" />
</svg>`

func handleLogo(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "image/svg+xml")
	w.Header().Set("Cache-Control", "public, max-age=86400")
	w.Write([]byte(atlasLogoSVG))
}

func handleManifest(w http.ResponseWriter, r *http.Request, token string) {
	w.Header().Set("Content-Type", "application/json")
	w.Header().Set("Access-Control-Allow-Origin", "*")

	addonName := "Atlas"
	supabase := NewSupabaseClient()
	if doc, err := supabase.GetUserPreferences(token); err == nil && doc.ProfileName != "" {
		addonName = "Atlas - " + doc.ProfileName
	}

	scheme := "http://"
	if r.TLS != nil || r.Header.Get("X-Forwarded-Proto") == "https" {
		scheme = "https://"
	}
	logoUrl := scheme + r.Host + "/logo.svg"

	manifest := map[string]interface{}{
		"id":          "com.cindrallabs.atlas." + token,
		"name":        addonName,
		"version":     "1.0.0",
		"description": "Premium multi-source streaming via TorBox. Experience personalized, unthrottled, and ultra-fast playback.",
		"logo":        logoUrl,
		"resources":   []string{"stream"},
		"types":       []string{"movie", "series"},
		"idPrefixes":  []string{"tt"},
	}

	json.NewEncoder(w).Encode(manifest)
}

// defaultPreferences is what Atlas acts on when a token's preferences cannot be
// loaded. Everything is inert so the Rust core has well-formed input, and the
// tier is free so a Supabase outage can never hand out premium.
func defaultPreferences() map[string]interface{} {
	return map[string]interface{}{
		"torbox_api_key":  "",
		"trakt_client_id": "",
		"trakt_username":  "",
		"max_resolution":  "4K",
		"exclude_av1":     false,
		"sort_preference": "balanced",
		"stream_limit":    5,
		"is_premium":      false,
	}
}

// loadPreferences resolves the preferences Atlas will act on for an install
// token, with entitlement decided server-side.
//
// prefs_json is user-writable — Supabase RLS lets a user update their own
// preferences row, and nothing validates the blob's contents — so an
// is_premium found in there is untrusted input and is always overwritten with
// the tier from app_users, which users can read but not write.
//
// Both the stream and resolve paths go through here so neither can forget the
// override. The result is cached per token, entitlement included.
func loadPreferences(token string) map[string]interface{} {
	resolved, ok := resolveProfile(token)
	if !ok {
		return defaultPreferences()
	}
	return resolved.Prefs
}

// resolveProfile is the one place an install token turns into preferences, so
// the entitlement override below cannot be forgotten by a caller. It reports
// whether the token resolved at all, which the Jellyfin surface needs in order
// to reject a bad password rather than quietly serving a free-tier profile.
func resolveProfile(token string) (resolvedProfile, bool) {
	if cached, ok := prefsCache.Get(token); ok {
		return cached, true
	}

	supabase := NewSupabaseClient()
	doc, err := supabase.GetUserPreferences(token)
	if err != nil {
		log.Printf("Failed to fetch user preferences from Supabase for token %s: %v", token, err)
		return resolvedProfile{}, false
	}

	prefs := doc.PrefsJson
	if prefs == nil {
		prefs = map[string]interface{}{}
	}
	prefs["is_premium"] = supabase.IsPremium(doc.UserId)

	resolved := resolvedProfile{Prefs: prefs, ProfileName: doc.ProfileName}
	prefsCache.Add(token, resolved)
	return resolved, true
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

	prefs := loadPreferences(token)

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

	prefs := loadPreferences(token)

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
