package main

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"

	"golang.org/x/net/webdav"
)

type AtlasFS struct {
	Token    string
	Prefs    map[string]interface{}
	Trakt    *TraktClient
	Cinemeta *CinemetaClient
}

func (fs *AtlasFS) Mkdir(ctx context.Context, name string, perm os.FileMode) error {
	return os.ErrPermission
}

func (fs *AtlasFS) RemoveAll(ctx context.Context, name string) error {
	return os.ErrPermission
}

func (fs *AtlasFS) Rename(ctx context.Context, oldName, newName string) error {
	return os.ErrPermission
}

func (fs *AtlasFS) Stat(ctx context.Context, name string) (os.FileInfo, error) {
	name = strings.TrimSuffix(name, "/")
	if name == "" {
		return &VirtualDir{name: "/"}, nil
	}

	parts := strings.Split(strings.Trim(name, "/"), "/")

	if len(parts) == 1 {
		if parts[0] == "Movies" || parts[0] == "Series" {
			return &VirtualDir{name: parts[0]}, nil
		}
		return nil, os.ErrNotExist
	}

	if len(parts) == 2 {
		if parts[1] == "Trending" || parts[1] == "Watchlist" {
			return &VirtualDir{name: parts[1]}, nil
		}
		return nil, os.ErrNotExist
	}

	// /Movies/Trending/Dune Part Two [tt123].mp4
	if parts[0] == "Movies" && len(parts) == 3 {
		return &VirtualFile{name: parts[2], size: 10 * 1024 * 1024 * 1024}, nil // fake 10GB size
	}

	// /Series/Trending/Fallout [tt123]
	if parts[0] == "Series" && len(parts) == 3 {
		return &VirtualDir{name: parts[2]}, nil
	}

	// /Series/Trending/Fallout [tt123]/Season 1
	if parts[0] == "Series" && len(parts) == 4 {
		return &VirtualDir{name: parts[3]}, nil
	}

	// /Series/Trending/Fallout [tt123]/Season 1/S01E01.mp4
	if parts[0] == "Series" && len(parts) == 5 {
		return &VirtualFile{name: parts[4], size: 2 * 1024 * 1024 * 1024}, nil // fake 2GB size
	}

	return nil, os.ErrNotExist
}

// Helpers to extract IDs
var imdbRegex = regexp.MustCompile(`\[(tt\d+)\]`)
var seasonRegex = regexp.MustCompile(`Season (\d+)`)
var episodeRegex = regexp.MustCompile(`S(\d+)E(\d+)`)

// Virtual File implementation
type VirtualFile struct {
	name string
	size int64
}

var fixedTime = time.Date(2024, 1, 1, 0, 0, 0, 0, time.UTC)

func (f *VirtualFile) Name() string       { return f.name }
func (f *VirtualFile) Size() int64        { return f.size }
func (f *VirtualFile) Mode() os.FileMode  { return 0644 }
func (f *VirtualFile) ModTime() time.Time { return fixedTime }
func (f *VirtualFile) IsDir() bool        { return false }
func (f *VirtualFile) Sys() interface{}   { return nil }
func (f *VirtualFile) ContentType(ctx context.Context) (string, error) {
	if strings.HasSuffix(f.name, ".mp4") {
		return "video/mp4", nil
	}
	return "application/octet-stream", nil
}
func (f *VirtualFile) ETag(ctx context.Context) (string, error) {
	return fmt.Sprintf(`"%s"`, f.name), nil
}

type VirtualDir struct {
	name string
}

func (d *VirtualDir) Name() string       { return d.name }
func (d *VirtualDir) Size() int64        { return 0 }
func (d *VirtualDir) Mode() os.FileMode  { return os.ModeDir | 0755 }
func (d *VirtualDir) ModTime() time.Time { return fixedTime }
func (d *VirtualDir) IsDir() bool        { return true }
func (d *VirtualDir) Sys() interface{}   { return nil }

type VirtualNode struct {
	isDir bool
	name  string
	size  int64
	fs    *AtlasFS
	path  string
	// For actual playback reading
	reader io.ReadCloser
	ctx    context.Context // store context for lazy loading
}

func (fs *AtlasFS) OpenFile(ctx context.Context, name string, flag int, perm os.FileMode) (webdav.File, error) {
	stat, err := fs.Stat(ctx, name)
	if err != nil {
		return nil, err
	}

	node := &VirtualNode{
		isDir: stat.IsDir(),
		name:  stat.Name(),
		size:  stat.Size(),
		fs:    fs,
		path:  name,
		ctx:   ctx,
	}

	return node, nil
}

func (n *VirtualNode) lazyInit() error {
	if n.reader != nil {
		return nil // already initialized
	}
	if n.isDir {
		return os.ErrInvalid
	}

	imdbMatch := imdbRegex.FindStringSubmatch(n.path)
	if len(imdbMatch) < 2 {
		// This is a fake error file or something without an IMDB ID.
		// Return a dummy reader so it doesn't crash.
		n.reader = io.NopCloser(strings.NewReader("Dummy Content"))
		return nil
	}
	imdbId := imdbMatch[1]

	stremioId := imdbId

	season, episode := 0, 0
	epMatch := episodeRegex.FindStringSubmatch(n.name)
	if len(epMatch) == 3 {
		season, _ = strconv.Atoi(epMatch[1])
		episode, _ = strconv.Atoi(epMatch[2])
		stremioId = fmt.Sprintf("%s:%d:%d", imdbId, season, episode)
	}

	log.Printf("WebDAV resolving stream for playback: %s", stremioId)

	reqBody, _ := json.Marshal(map[string]interface{}{
		"stremio_id":    stremioId,
		"install_token": n.fs.Token,
		"prefs":         n.fs.Prefs,
		"user_agent":    "Atlas Infuse WebDAV",
	})

	resp, err := http.Post(coreUrl+"/internal/resolve", "application/json", strings.NewReader(string(reqBody)))
	if err != nil {
		return err
	}

	if resp.StatusCode != http.StatusOK && resp.StatusCode != http.StatusFound {
		resp.Body.Close()
		return os.ErrNotExist
	}

	n.reader = resp.Body
	return nil
}

func (n *VirtualNode) Close() error {
	if n.reader != nil {
		return n.reader.Close()
	}
	return nil
}

func (n *VirtualNode) Read(p []byte) (int, error) {
	if err := n.lazyInit(); err != nil {
		return 0, err
	}
	return n.reader.Read(p)
}

func (n *VirtualNode) Seek(offset int64, whence int) (int64, error) {
	// Seeking is hard for proxy streams without ranged requests
	return 0, nil
}

func (n *VirtualNode) Write(p []byte) (int, error) {
	return 0, os.ErrPermission
}

func (n *VirtualNode) Stat() (os.FileInfo, error) {
	if n.isDir {
		return &VirtualDir{name: n.name}, nil
	}
	return &VirtualFile{name: n.name, size: n.size}, nil
}

func (n *VirtualNode) Readdir(count int) ([]os.FileInfo, error) {
	if !n.isDir {
		return nil, os.ErrInvalid
	}

	parts := strings.Split(strings.Trim(n.path, "/"), "/")
	var infos []os.FileInfo

	if n.path == "/" || n.path == "" {
		infos = append(infos, &VirtualDir{name: "Movies"}, &VirtualDir{name: "Series"})
		return infos, nil
	}

	if len(parts) == 1 {
		infos = append(infos, &VirtualDir{name: "Trending"}, &VirtualDir{name: "Watchlist"})
		return infos, nil
	}

	// e.g. /Movies/Trending or /Series/Watchlist
	if len(parts) == 2 {
		kind := parts[0]
		listType := parts[1]

		var traktUsername string
		if u, ok := n.fs.Prefs["trakt_username"].(string); ok {
			traktUsername = u
		}

		if kind == "Movies" {
			var movies []TraktMovie
			var err error
			if listType == "Trending" {
				movies, err = n.fs.Trakt.GetTrendingMovies()
			} else if listType == "Watchlist" {
				if traktUsername != "" {
					movies, err = n.fs.Trakt.GetWatchlistMovies(traktUsername)
				} else {
					err = fmt.Errorf("Trakt username not configured")
				}
			}

			if err != nil {
				log.Printf("Error fetching Trakt movies for %s: %v", listType, err)
				infos = append(infos, &VirtualFile{name: fmt.Sprintf("Error - %v.mp4", err), size: 1024})
			} else if len(movies) == 0 {
				infos = append(infos, &VirtualFile{name: "No movies found.mp4", size: 1024})
			}

			for _, m := range movies {
				if m.IDs.Imdb != "" {
					name := fmt.Sprintf("%s [%s].mp4", sanitize(m.Title), m.IDs.Imdb)
					infos = append(infos, &VirtualFile{name: name, size: 10 * 1024 * 1024 * 1024})
				}
			}
		} else {
			var shows []TraktShow
			var err error
			if listType == "Trending" {
				shows, err = n.fs.Trakt.GetTrendingShows()
			} else if listType == "Watchlist" {
				if traktUsername != "" {
					shows, err = n.fs.Trakt.GetWatchlistShows(traktUsername)
				} else {
					err = fmt.Errorf("Trakt username not configured")
				}
			}

			if err != nil {
				log.Printf("Error fetching Trakt shows for %s: %v", listType, err)
				infos = append(infos, &VirtualFile{name: fmt.Sprintf("Error - %v.mp4", err), size: 1024})
			} else if len(shows) == 0 {
				infos = append(infos, &VirtualFile{name: "No shows found.mp4", size: 1024})
			}

			for _, s := range shows {
				if s.IDs.Imdb != "" {
					name := fmt.Sprintf("%s [%s]", sanitize(s.Title), s.IDs.Imdb)
					infos = append(infos, &VirtualDir{name: name})
				}
			}
		}
		return infos, nil
	}

	// /Series/Trending/Fallout [tt123] -> List Seasons
	if parts[0] == "Series" && len(parts) == 3 {
		match := imdbRegex.FindStringSubmatch(parts[2])
		if len(match) == 2 {
			imdbId := match[1]
			meta, err := n.fs.Cinemeta.GetSeries(imdbId)
			if err == nil && meta != nil {
				seasons := make(map[int]bool)
				for _, v := range meta.Videos {
					seasons[v.Season] = true
				}
				var seasonList []int
				for s := range seasons {
					if s > 0 { // Ignore season 0 usually (specials)
						seasonList = append(seasonList, s)
					}
				}
				sort.Ints(seasonList)
				for _, s := range seasonList {
					infos = append(infos, &VirtualDir{name: fmt.Sprintf("Season %d", s)})
				}
			}
		}
		return infos, nil
	}

	// /Series/Trending/Fallout [tt123]/Season 1 -> List Episodes
	if parts[0] == "Series" && len(parts) == 4 {
		imdbMatch := imdbRegex.FindStringSubmatch(parts[2])
		seasonMatch := seasonRegex.FindStringSubmatch(parts[3])

		if len(imdbMatch) == 2 && len(seasonMatch) == 2 {
			imdbId := imdbMatch[1]
			season, _ := strconv.Atoi(seasonMatch[1])

			meta, err := n.fs.Cinemeta.GetSeries(imdbId)
			if err == nil && meta != nil {
				for _, v := range meta.Videos {
					if v.Season == season {
						title := sanitize(v.Title)
						name := fmt.Sprintf("S%02dE%02d - %s.mp4", v.Season, v.Episode, title)
						infos = append(infos, &VirtualFile{name: name, size: 2 * 1024 * 1024 * 1024})
					}
				}
			}
		}
		return infos, nil
	}

	return infos, nil
}

func sanitize(s string) string {
	s = strings.ReplaceAll(s, "/", "-")
	s = strings.ReplaceAll(s, ":", " -")
	s = strings.ReplaceAll(s, "?", "")
	return s
}
