package logger

import (
	"fmt"
	"io"
	"time"

	"github.com/fatih/color"
	"github.com/mattn/go-colorable"
)

// Logger 统一日志工具，支持并发标识、颜色显示和多平台适配
// Unified logging tool, supporting concurrency identification, color display and multi-platform adaptation
type Logger struct {
	Module string
	out    io.Writer
}

// 预定义颜色方案 (Predefined color schemes)
var (
	colorTime    = color.New(color.FgHiBlack) // 时间戳颜色 (Timestamp color)
	colorModule  = color.New(color.FgCyan).Add(color.Bold)
	colorTagInfo = color.New(color.FgBlue)
	colorTagSucc = color.New(color.FgGreen)
	colorTagErr  = color.New(color.FgRed).Add(color.Bold)
	colorTagWarn = color.New(color.FgYellow)
)

// NewLogger 创建一个新的日志记录器，自动适配跨平台颜色输出
// Create a new logger, automatically adapting to cross-platform color output
func NewLogger(module string) *Logger {
	return &Logger{
		Module: module,
		out:    colorable.NewColorableStdout(), // 自动处理 Windows 颜色支持
	}
}

// formatLog 内部统一格式化逻辑
// formatLog internal unified formatting logic
func (l *Logger) formatLog(tagColor *color.Color, tag, format string, v ...interface{}) {
	// 获取并格式化时间 (Get and format time)
	timestamp := time.Now().Format("2006/01/02 15:04:05")
	timeStr := colorTime.Sprint(timestamp)

	// 构造模块和标签 (Construct module and tag)
	moduleStr := colorModule.Sprint("[" + l.Module + "]")
	tagStr := tagColor.Sprint("[" + tag + "]")
	detailStr := fmt.Sprintf(format, v...)

	// 使用 Fprintf 确保颜色字符正确写入支持的输出流
	// 格式: 时间 [模块] [标签] 详情
	fmt.Fprintf(l.out, "%s %s %s %s\n", timeStr, moduleStr, tagStr, detailStr)
}

// Log 打印常规日志
func (l *Logger) Log(tag, format string, v ...interface{}) {
	l.formatLog(colorTagInfo, tag, format, v...)
}

// Info 别名，用于常规信息打印
func (l *Logger) Info(tag, format string, v ...interface{}) {
	l.formatLog(colorTagInfo, tag, format, v...)
}

// Error 打印错误日志
func (l *Logger) Error(tag, format string, v ...interface{}) {
	l.formatLog(colorTagErr, tag, format, v...)
}

// Success 打印成功日志
func (l *Logger) Success(tag, format string, v ...interface{}) {
	l.formatLog(colorTagSucc, tag, format, v...)
}

// Warn 打印警告日志
func (l *Logger) Warn(tag, format string, v ...interface{}) {
	l.formatLog(colorTagWarn, tag, format, v...)
}
