package main

import (
	"embed"
	"errors"

	"fyne.io/fyne/v2"
)

var _ embed.FS

//go:embed assets/fonts/NotoSansCJKsc-Regular.otf
var fontData []byte

func loadEmbeddedFont() (fyne.Resource, error) {
	if len(fontData) == 0 {
		return nil, errors.New("Embedded font is empty")
	}
	return fyne.NewStaticResource("NotoSansCJKsc-Regular.otf", fontData), nil
}
