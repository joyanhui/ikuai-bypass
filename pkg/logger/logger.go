package logger

import (
	"fmt"
	"io"
	"regexp"
	"strings"
	"time"

	"github.com/fatih/color"
	"github.com/mattn/go-colorable"
)

// Logger 统一日志工具，支持并发标识、颜色显示和多平台适配
type Logger struct {
	Module string
	out    io.Writer
}

// 预定义颜色方案
var (
	colorTime      = color.New(color.FgHiBlack)                 // 时间戳颜色
	colorModule    = color.New(color.FgCyan).Add(color.Bold)    // 模块名颜色
	colorTagInfo   = color.New(color.FgBlue)                    // 信息标签颜色
	colorTagSucc   = color.New(color.FgGreen)                   // 成功标签颜色
	colorTagErr    = color.New(color.FgRed).Add(color.Bold)     // 错误标签颜色
	colorTagWarn   = color.New(color.FgYellow)                  // 警告标签颜色
	colorHighlight = color.New(color.FgHiYellow).Add(color.Bold) // 关键内容高亮
	colorNumber    = color.New(color.FgHiMagenta)               // 数字高亮
)

// NewLogger 创建一个新的日志记录器
func NewLogger(module string) *Logger {
	return &Logger{
		Module: module,
		out:    colorable.NewColorableStdout(),
	}
}

// highlight 自动识别并高亮字符串中的关键信息，同时避免破坏 ANSI 代码
func highlight(s string) string {
	// 1. 高亮引号内容 (Yellow Bold)
	reQuoted := regexp.MustCompile(`'([^']+)'`)
	s = reQuoted.ReplaceAllStringFunc(s, func(m string) string {
		return colorHighlight.Sprint(m)
	})

	// 2. 高亮特定关键字后的值
	reKV := regexp.MustCompile(`(?i)(Prefix|Tag|IDs?|found|error|status|interface):\s*([^\s,)]+)`)
	s = reKV.ReplaceAllStringFunc(s, func(m string) string {
		sub := reKV.FindStringSubmatch(m)
		if len(sub) == 3 {
			// 如果该部分已经包含颜色代码，跳过以防嵌套损坏
			if strings.Contains(sub[2], "\x1b[") {
				return sub[1] + ": " + sub[2]
			}
			return sub[1] + ": " + colorHighlight.Sprint(sub[2])
		}
		return m
	})

	// 3. 高亮纯数字 (注意避免匹配 ANSI 颜色代码内部的数字)
	// 正则说明：匹配 ANSI 转义序列 (\x1b\[...) OR 独立数字 (\b\d+\b)
	reSafeNum := regexp.MustCompile(`\x1b\[[0-9;]*[a-zA-Z]|\b\d+\b`)
	s = reSafeNum.ReplaceAllStringFunc(s, func(m string) string {
		if strings.HasPrefix(m, "\x1b") {
			return m // 如果是颜色代码，直接透传
		}
		return colorNumber.Sprint(m) // 如果是数字，应用品红色
	})

	return s
}

// formatLog 内部统一格式化逻辑
func (l *Logger) formatLog(tagColor *color.Color, tag, format string, v ...interface{}) {
	timestamp := time.Now().Format("2006/01/02 15:04:05")
	timeStr := colorTime.Sprint(timestamp)

	moduleStr := colorModule.Sprint("[" + l.Module + "]")
	tagStr := tagColor.Sprint("[" + tag + "]")

	detailStr := fmt.Sprintf(format, v...)
	detailStr = highlight(detailStr)

	fmt.Fprintf(l.out, "%s %s %s %s\n", timeStr, moduleStr, tagStr, detailStr)
}

func (l *Logger) Log(tag, format string, v ...interface{})     { l.formatLog(colorTagInfo, tag, format, v...) }
func (l *Logger) Info(tag, format string, v ...interface{})    { l.formatLog(colorTagInfo, tag, format, v...) }
func (l *Logger) Error(tag, format string, v ...interface{})   { l.formatLog(colorTagErr, tag, format, v...) }
func (l *Logger) Success(tag, format string, v ...interface{}) { l.formatLog(colorTagSucc, tag, format, v...) }
func (l *Logger) Warn(tag, format string, v ...interface{})    { l.formatLog(colorTagWarn, tag, format, v...) }
