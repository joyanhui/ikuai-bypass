package logger

import (
	"fmt"
	"log"
)

// Logger 统一日志工具，支持并发标识和多语言格式
// Unified logging tool, supporting concurrency identification and multi-language format
type Logger struct {
	Module string // 模块名 (Module name)
}

// NewLogger 创建一个新的日志记录器
// Create a new logger
func NewLogger(module string) *Logger {
	return &Logger{Module: module}
}

// Log 打印统一格式的日志：[时间] [模块] [中文标识] English detail...
// Print unified format log: [Time] [Module] [Chinese Tag] English detail...
func (l *Logger) Log(tag, format string, v ...interface{}) {
	detail := fmt.Sprintf(format, v...)
	log.Printf("[%s] [%s] %s\n", l.Module, tag, detail)
}

// Info 别名，用于常规信息打印
// Alias for regular information printing
func (l *Logger) Info(tag, format string, v ...interface{}) {
	l.Log(tag, format, v...)
}

// Error 打印错误日志
// Print error log
func (l *Logger) Error(tag, format string, v ...interface{}) {
	detail := fmt.Sprintf(format, v...)
	log.Printf("[%s] [%s] [ERROR] %s\n", l.Module, tag, detail)
}

// Success 打印成功日志
// Print success log
func (l *Logger) Success(tag, format string, v ...interface{}) {
	detail := fmt.Sprintf(format, v...)
	log.Printf("[%s] [%s] [SUCCESS] %s\n", l.Module, tag, detail)
}
