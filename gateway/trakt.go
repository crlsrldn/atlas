package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"time"
)

type TraktClient struct {
	ClientID   string
	HTTPClient *http.Client
}

func NewTraktClient(clientID string) *TraktClient {
	return &TraktClient{
		ClientID: clientID,
		HTTPClient: &http.Client{
			Timeout: 10 * time.Second,
		},
	}
}

type TraktIDs struct {
	Trakt int    `json:"trakt"`
	Slug  string `json:"slug"`
	Imdb  string `json:"imdb"`
	Tmdb  int    `json:"tmdb"`
}

type TraktMovie struct {
	Title string   `json:"title"`
	Year  int      `json:"year"`
	IDs   TraktIDs `json:"ids"`
}

type TraktShow struct {
	Title string   `json:"title"`
	Year  int      `json:"year"`
	IDs   TraktIDs `json:"ids"`
}

type TraktTrendingMovieResponse struct {
	Watchers int        `json:"watchers"`
	Movie    TraktMovie `json:"movie"`
}

type TraktTrendingShowResponse struct {
	Watchers int       `json:"watchers"`
	Show     TraktShow `json:"show"`
}

type TraktWatchlistMovieResponse struct {
	Movie TraktMovie `json:"movie"`
}

type TraktWatchlistShowResponse struct {
	Show TraktShow `json:"show"`
}

func (c *TraktClient) doRequest(url string, target interface{}) error {
	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("trakt-api-version", "2")
	req.Header.Set("trakt-api-key", c.ClientID)

	resp, err := c.HTTPClient.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("trakt api error: status code %d", resp.StatusCode)
	}

	return json.NewDecoder(resp.Body).Decode(target)
}

func (c *TraktClient) GetTrendingMovies() ([]TraktMovie, error) {
	var resp []TraktTrendingMovieResponse
	err := c.doRequest("https://api.trakt.tv/movies/trending?limit=20", &resp)
	if err != nil {
		return nil, err
	}

	var movies []TraktMovie
	for _, r := range resp {
		movies = append(movies, r.Movie)
	}
	return movies, nil
}

func (c *TraktClient) GetTrendingShows() ([]TraktShow, error) {
	var resp []TraktTrendingShowResponse
	err := c.doRequest("https://api.trakt.tv/shows/trending?limit=20", &resp)
	if err != nil {
		return nil, err
	}

	var shows []TraktShow
	for _, r := range resp {
		shows = append(shows, r.Show)
	}
	return shows, nil
}

func (c *TraktClient) GetWatchlistMovies(username string) ([]TraktMovie, error) {
	var resp []TraktWatchlistMovieResponse
	url := fmt.Sprintf("https://api.trakt.tv/users/%s/watchlist/movies", username)
	err := c.doRequest(url, &resp)
	if err != nil {
		return nil, err
	}

	var movies []TraktMovie
	for _, r := range resp {
		movies = append(movies, r.Movie)
	}
	return movies, nil
}

func (c *TraktClient) GetWatchlistShows(username string) ([]TraktShow, error) {
	var resp []TraktWatchlistShowResponse
	url := fmt.Sprintf("https://api.trakt.tv/users/%s/watchlist/shows", username)
	err := c.doRequest(url, &resp)
	if err != nil {
		return nil, err
	}

	var shows []TraktShow
	for _, r := range resp {
		shows = append(shows, r.Show)
	}
	return shows, nil
}
