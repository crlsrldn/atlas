package main

import (
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

// fakeCore captures what the gateway forwarded, so the tests can assert on the
// headers core would have received.
type fakeCore struct {
	server  *httptest.Server
	lastReq *http.Request
	lastBod string
}

func newFakeCore(t *testing.T) *fakeCore {
	t.Helper()

	core := &fakeCore{}
	core.server = httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		body, _ := io.ReadAll(r.Body)
		core.lastReq = r.Clone(r.Context())
		core.lastBod = string(body)

		w.Header().Set("Content-Type", "application/json")
		_ = json.NewEncoder(w).Encode(map[string]string{"ok": r.URL.Path})
	}))

	previous := coreUrl
	coreUrl = core.server.URL
	t.Cleanup(func() {
		coreUrl = previous
		core.server.Close()
	})

	return core
}

func jellyfinRequest(t *testing.T, method, target string, body string) *http.Request {
	t.Helper()

	var reader io.Reader
	if body != "" {
		reader = strings.NewReader(body)
	}
	return httptest.NewRequest(method, target, reader)
}

func TestJellyfinPublicRouteNeedsNoToken(t *testing.T) {
	core := newFakeCore(t)

	recorder := httptest.NewRecorder()
	handleJellyfin(recorder, jellyfinRequest(t, http.MethodGet, "/jellyfin/System/Info/Public", ""))

	if recorder.Code != http.StatusOK {
		t.Fatalf("the server-discovery probe must answer without credentials, got %d", recorder.Code)
	}
	if core.lastReq.Header.Get("X-Atlas-Token") != "" {
		t.Fatal("no token should be injected when none was presented")
	}
}

func TestJellyfinProtectedRouteRequiresAToken(t *testing.T) {
	newFakeCore(t)

	recorder := httptest.NewRecorder()
	handleJellyfin(recorder, jellyfinRequest(t, http.MethodGet, "/jellyfin/Users/abc/Views", ""))

	if recorder.Code != http.StatusUnauthorized {
		t.Fatalf("browsing without a token must be rejected, got %d", recorder.Code)
	}
}

func TestJellyfinUnknownTokenIsRejectedRatherThanDowngraded(t *testing.T) {
	newFakeCore(t)

	request := jellyfinRequest(t, http.MethodGet, "/jellyfin/Users/abc/Views", "")
	request.Header.Set("X-Emby-Token", "token-does-not-exist")

	recorder := httptest.NewRecorder()
	handleJellyfin(recorder, request)

	// The Stremio path falls back to free defaults on a failed lookup. Doing
	// that here would silently sign a client in with someone else's blank
	// profile instead of telling them the password is wrong.
	if recorder.Code != http.StatusUnauthorized {
		t.Fatalf("an unresolvable token must 401 rather than serve defaults, got %d", recorder.Code)
	}
}

func TestJellyfinInjectsResolvedPreferences(t *testing.T) {
	core := newFakeCore(t)

	request := jellyfinRequest(t, http.MethodGet, "/jellyfin/Users/abc/Views", "")
	request.Header.Set("X-Emby-Token", "token-liar")

	recorder := httptest.NewRecorder()
	handleJellyfin(recorder, request)

	if recorder.Code != http.StatusOK {
		t.Fatalf("a valid token should be forwarded, got %d", recorder.Code)
	}
	if got := core.lastReq.Header.Get("X-Atlas-Token"); got != "token-liar" {
		t.Fatalf("core should receive the resolved token, got %q", got)
	}
	if got := core.lastReq.Header.Get("X-Atlas-Profile-Name"); got != "Liar" {
		t.Fatalf("core should receive the profile name, got %q", got)
	}

	var prefs map[string]interface{}
	if err := json.Unmarshal([]byte(core.lastReq.Header.Get("X-Atlas-Prefs")), &prefs); err != nil {
		t.Fatalf("injected preferences must be valid JSON: %v", err)
	}
	// The same server-side entitlement override the Stremio path relies on.
	if prefs["is_premium"] != false {
		t.Fatalf("self-declared premium must not survive into the Jellyfin surface, got %v", prefs["is_premium"])
	}
}

func TestJellyfinDiscardsClientSuppliedAtlasHeaders(t *testing.T) {
	core := newFakeCore(t)

	request := jellyfinRequest(t, http.MethodGet, "/jellyfin/Users/abc/Views", "")
	request.Header.Set("X-Emby-Token", "token-premium")
	// A client trying to present itself as an already-resolved premium profile.
	request.Header.Set("X-Atlas-Prefs", `{"is_premium":true}`)
	request.Header.Set("X-Atlas-Profile-Name", "Injected")

	recorder := httptest.NewRecorder()
	handleJellyfin(recorder, request)

	if got := core.lastReq.Header.Get("X-Atlas-Profile-Name"); got != "Paid" {
		t.Fatalf("client-supplied Atlas headers must be replaced, got %q", got)
	}

	var prefs map[string]interface{}
	_ = json.Unmarshal([]byte(core.lastReq.Header.Get("X-Atlas-Prefs")), &prefs)
	if _, spoofed := prefs["torbox_api_key"]; !spoofed {
		t.Fatal("preferences should have been rebuilt from Supabase, not taken from the client")
	}
}

func TestJellyfinForwardsTheMonetizationFlag(t *testing.T) {
	// It is an input to rank_sources, so a client that could set it would be
	// choosing its own ranking.
	core := newFakeCore(t)

	request := jellyfinRequest(t, http.MethodGet, "/jellyfin/Users/abc/Views", "")
	request.Header.Set("X-Emby-Token", "token-premium")
	request.Header.Set("X-Atlas-Monetization", "true")

	recorder := httptest.NewRecorder()
	handleJellyfin(recorder, request)

	if got := core.lastReq.Header.Get("X-Atlas-Monetization"); got != "false" {
		t.Fatalf("the gateway decides monetization, not the client: got %q", got)
	}
}

func TestJellyfinReadsTheTokenFromTheLoginBody(t *testing.T) {
	core := newFakeCore(t)

	body := `{"Username":"anything","Pw":"token-premium"}`
	request := jellyfinRequest(t, http.MethodPost, "/jellyfin/Users/AuthenticateByName", body)
	request.Header.Set("Content-Type", "application/json")

	recorder := httptest.NewRecorder()
	handleJellyfin(recorder, request)

	if recorder.Code != http.StatusOK {
		t.Fatalf("login should be accepted, got %d", recorder.Code)
	}
	if got := core.lastReq.Header.Get("X-Atlas-Token"); got != "token-premium" {
		t.Fatalf("the password is the install token, got %q", got)
	}
	// The body still has to reach core, which builds the response from it.
	if core.lastBod != body {
		t.Fatalf("login body was not forwarded intact, got %q", core.lastBod)
	}
}

func TestJellyfinTokenSources(t *testing.T) {
	cases := []struct {
		name    string
		prepare func(*http.Request)
		want    string
	}{
		{
			name:    "emby token header",
			prepare: func(r *http.Request) { r.Header.Set("X-Emby-Token", "from-header") },
			want:    "from-header",
		},
		{
			name:    "mediabrowser token header",
			prepare: func(r *http.Request) { r.Header.Set("X-MediaBrowser-Token", "from-mb") },
			want:    "from-mb",
		},
		{
			name: "authorization header",
			prepare: func(r *http.Request) {
				r.Header.Set("X-Emby-Authorization",
					`MediaBrowser Client="Infuse-Direct", Device="Apple TV", Token="from-auth"`)
			},
			want: "from-auth",
		},
		{
			name:    "no credential at all",
			prepare: func(r *http.Request) {},
			want:    "",
		},
	}

	for _, testCase := range cases {
		t.Run(testCase.name, func(t *testing.T) {
			request := jellyfinRequest(t, http.MethodGet, "/jellyfin/Users/abc/Views", "")
			testCase.prepare(request)

			if got := jellyfinToken(request, nil); got != testCase.want {
				t.Fatalf("got %q, want %q", got, testCase.want)
			}
		})
	}
}

func TestJellyfinTokenFromApiKeyQuery(t *testing.T) {
	// Jellyfin clients append api_key to image and stream URLs rather than
	// setting a header.
	request := jellyfinRequest(t, http.MethodGet, "/jellyfin/Items/abc/Images/Primary?api_key=from-query", "")

	if got := jellyfinToken(request, nil); got != "from-query" {
		t.Fatalf("got %q, want %q", got, "from-query")
	}
}

func TestAnonymizePathDropsQueryStrings(t *testing.T) {
	// Jellyfin clients append api_key to image and stream URLs. Only a path
	// reaches telemetry today, but a full request URI must not leak one.
	got := anonymizePath("/jellyfin/Items/abc/Images/Primary?api_key=secret")

	if strings.Contains(got, "secret") {
		t.Fatalf("credential leaked into telemetry path: %s", got)
	}
	if got != "/jellyfin/Items/abc/Images/Primary" {
		t.Fatalf("unexpected path: %s", got)
	}
}

func TestAnonymizePathStillRedactsStremioTokensWithAQuery(t *testing.T) {
	got := anonymizePath("/stremio/atl_secret/resolve/torbox/abc/play.mp4?cached=true")

	if strings.Contains(got, "atl_secret") {
		t.Fatalf("install token leaked: %s", got)
	}
	if strings.Contains(got, "cached=true") {
		t.Fatalf("query survived: %s", got)
	}
}

func TestJellyfinRouteStripsBothPrefixes(t *testing.T) {
	for _, path := range []string{"/jellyfin/System/Info/Public", "/emby/System/Info/Public"} {
		if got := jellyfinRoute(path); got != "/System/Info/Public" {
			t.Fatalf("%s: got %q", path, got)
		}
	}
}

func TestJellyfinTrailingSlashesAreNormalized(t *testing.T) {
	// Infuse lists a container's children with "/Items/?ParentId=…". The core
	// router treats that as a different route from "/Items", so the request
	// fell through — which emptied the Movies library and broke opening a
	// series, both being the same request.
	cases := map[string]string{
		"/jellyfin/Items/":  "/jellyfin/Items",
		"/jellyfin/Items//": "/jellyfin/Items",
		"/jellyfin/Items":   "/jellyfin/Items",
		"/":                 "/",
	}

	for input, want := range cases {
		if got := normalizeJellyfinPath(input); got != want {
			t.Fatalf("%s: got %q, want %q", input, got, want)
		}
	}

	if got := jellyfinRoute("/jellyfin/Items/"); got != "/Items" {
		t.Fatalf("route with a trailing slash: got %q", got)
	}
}

func TestJellyfinForwardsTheNormalizedPath(t *testing.T) {
	core := newFakeCore(t)

	request := jellyfinRequest(t, http.MethodGet, "/jellyfin/Items/?ParentId=abc", "")
	request.Header.Set("X-Emby-Token", "token-premium")

	recorder := httptest.NewRecorder()
	handleJellyfin(recorder, request)

	if got := core.lastReq.URL.Path; got != "/jellyfin/Items" {
		t.Fatalf("core should receive the normalized path, got %q", got)
	}
	if got := core.lastReq.URL.RawQuery; got != "ParentId=abc" {
		t.Fatalf("the query must survive normalization, got %q", got)
	}
}

func TestJellyfinPreflightNeedsNoCredential(t *testing.T) {
	newFakeCore(t)

	recorder := httptest.NewRecorder()
	handleJellyfin(recorder, jellyfinRequest(t, http.MethodOptions, "/jellyfin/Users/abc/Views", ""))

	if recorder.Code != http.StatusOK {
		t.Fatalf("preflight must not be gated on auth, got %d", recorder.Code)
	}
	if recorder.Header().Get("Access-Control-Allow-Headers") == "" {
		t.Fatal("preflight should advertise the headers Jellyfin clients send")
	}
}

func TestJellyfinDoesNotFollowRedirects(t *testing.T) {
	// Playback depends on the 302 reaching the player, which then fetches bytes
	// straight from the CDN. Following it here would pull video through Atlas.
	core := newFakeCore(t)
	core.server.Config.Handler = http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		http.Redirect(w, r, "https://cdn.example.invalid/movie.mkv", http.StatusFound)
	})

	request := jellyfinRequest(t, http.MethodGet, "/jellyfin/Videos/abc/stream", "")
	request.Header.Set("X-Emby-Token", "token-premium")

	recorder := httptest.NewRecorder()
	handleJellyfin(recorder, request)

	if recorder.Code != http.StatusFound {
		t.Fatalf("the redirect must be passed through, got %d", recorder.Code)
	}
	if location := recorder.Header().Get("Location"); location != "https://cdn.example.invalid/movie.mkv" {
		t.Fatalf("unexpected Location: %q", location)
	}
}
