package config

import (
	"errors"
	"io"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"time"
)

func DownloadToFile(source string, dest string) error {
	if _, err := url.ParseRequestURI(source); err != nil {
		return errors.New("Invalid download URL")
	}
	client := &http.Client{Timeout: 15 * time.Second}
	resp, err := client.Get(source)
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return errors.New("Download failed with status: " + resp.Status)
	}
	if err := os.MkdirAll(filepath.Dir(dest), 0700); err != nil {
		return err
	}
	out, err := os.OpenFile(dest, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0600)
	if err != nil {
		return err
	}
	defer out.Close()
	_, err = io.Copy(out, resp.Body)
	return err
}

func EnsureConfigFromURL(path string, source string) error {
	if path == "" {
		return errors.New("Config path is empty")
	}
	if _, err := os.Stat(path); err == nil {
		return nil
	} else if !os.IsNotExist(err) {
		return err
	}
	return DownloadToFile(source, path)
}
