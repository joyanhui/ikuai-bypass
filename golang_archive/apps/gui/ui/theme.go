package ui

import (
	"image/color"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/theme"
)

// ChineseTheme 自定义中文主题
type ChineseTheme struct {
	font fyne.Resource
}

// NewChineseTheme 创建中文主题
func NewChineseTheme(font fyne.Resource) *ChineseTheme {
	return &ChineseTheme{font: font}
}

// Color 返回颜色
func (c *ChineseTheme) Color(name fyne.ThemeColorName, variant fyne.ThemeVariant) color.Color {
	// 强制使用明亮主题，避免深色模式下的默认组件（如下拉菜单、对话框、滚动条点击态）变成黑色
	// Force light variant globally so it matches our hardcoded light/blue layout.
	variant = theme.VariantLight

	// 使用现代化的蓝色系作为主色调
	switch name {
	case theme.ColorNamePrimary:
		// 主色调：现代蓝色
		return color.RGBA{R: 74, G: 144, B: 217, A: 255}
	case theme.ColorNameForeground:
		// 前景文字在浅色卡片上必须保持足够对比，否则会出现“有字但看不清”的问题。
		// Keep foreground text dark enough on light cards, otherwise labels become unreadable.
		if variant == theme.VariantLight {
			return color.RGBA{R: 50, G: 63, B: 86, A: 255}
		}
		return color.RGBA{R: 235, G: 240, B: 248, A: 255}
	case theme.ColorNameButton:
		// 按钮颜色
		if variant == theme.VariantLight {
			return color.RGBA{R: 74, G: 144, B: 217, A: 255}
		}
		return color.RGBA{R: 58, G: 123, B: 189, A: 255}
	case theme.ColorNameHover:
		// 悬停颜色
		return color.RGBA{R: 88, G: 162, B: 235, A: 255}
	case theme.ColorNamePlaceHolder:
		// 占位符颜色
		if variant == theme.VariantLight {
			return color.RGBA{R: 150, G: 150, B: 150, A: 255}
		}
		return color.RGBA{R: 100, G: 100, B: 100, A: 255}
	case theme.ColorNameDisabled:
		// 禁用状态颜色
		if variant == theme.VariantLight {
			return color.RGBA{R: 200, G: 200, B: 200, A: 255}
		}
		return color.RGBA{R: 80, G: 80, B: 80, A: 255}
	case theme.ColorNameBackground:
		// 背景色
		if variant == theme.VariantLight {
			return color.RGBA{R: 245, G: 247, B: 250, A: 255}
		}
		return color.RGBA{R: 30, G: 30, B: 35, A: 255}
	case theme.ColorNameInputBackground:
		// 输入框背景
		if variant == theme.VariantLight {
			return color.RGBA{R: 255, G: 255, B: 255, A: 255}
		}
		return color.RGBA{R: 45, G: 45, B: 50, A: 255}
	case theme.ColorNameShadow:
		// 阴影颜色
		if variant == theme.VariantLight {
			return color.RGBA{R: 0, G: 0, B: 0, A: 20}
		}
		return color.RGBA{R: 0, G: 0, B: 0, A: 50}
	case theme.ColorNameScrollBar:
		// 滚动条颜色
		return color.RGBA{R: 180, G: 180, B: 180, A: 120}
	case theme.ColorNamePressed:
		// 点击/按下时的统一高亮反馈色，不要让它变成深黑色
		return color.RGBA{R: 74, G: 144, B: 217, A: 50}
	case theme.ColorNameFocus:
		// 获得焦点时不要有突兀的黑色边框或背景
		return color.Transparent
	case theme.ColorNameSelection:
		// 选中文本时的颜色
		return color.RGBA{R: 74, G: 144, B: 217, A: 80}
	case theme.ColorNameSeparator:
		// 分隔线颜色
		if variant == theme.VariantLight {
			return color.RGBA{R: 220, G: 220, B: 220, A: 255}
		}
		return color.RGBA{R: 60, G: 60, B: 65, A: 255}
	}
	return theme.DefaultTheme().Color(name, variant)
}

// Font 返回字体
func (c *ChineseTheme) Font(style fyne.TextStyle) fyne.Resource {
	if c.font != nil {
		return c.font
	}
	return theme.DefaultTheme().Font(style)
}

// Icon 返回图标
func (c *ChineseTheme) Icon(name fyne.ThemeIconName) fyne.Resource {
	return theme.DefaultTheme().Icon(name)
}

// Size 返回尺寸
func (c *ChineseTheme) Size(name fyne.ThemeSizeName) float32 {
	switch name {
	case theme.SizeNameText:
		return 13
	case theme.SizeNameHeadingText:
		return 18
	case theme.SizeNameSubHeadingText:
		return 15
	case theme.SizeNameCaptionText:
		return 11
	case theme.SizeNameInputBorder:
		return 1
	case theme.SizeNameInputRadius:
		return 6
	case theme.SizeNamePadding:
		return 8
	case theme.SizeNameInlineIcon:
		return 16
	case theme.SizeNameScrollBar:
		return 6
	case theme.SizeNameScrollBarSmall:
		return 4
	}
	return theme.DefaultTheme().Size(name)
}
