package ui

import (
	"image/color"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/theme"
)

type ChineseTheme struct {
	font fyne.Resource
}

func NewChineseTheme(font fyne.Resource) *ChineseTheme {
	return &ChineseTheme{font: font}
}

func (c *ChineseTheme) Color(name fyne.ThemeColorName, variant fyne.ThemeVariant) color.Color {
	return theme.DefaultTheme().Color(name, variant)
}

func (c *ChineseTheme) Font(style fyne.TextStyle) fyne.Resource {
	if c.font == nil {
		return theme.DefaultTheme().Font(style)
	}
	return c.font
}

func (c *ChineseTheme) Icon(name fyne.ThemeIconName) fyne.Resource {
	return theme.DefaultTheme().Icon(name)
}

func (c *ChineseTheme) Size(name fyne.ThemeSizeName) float32 {
	return theme.DefaultTheme().Size(name)
}
