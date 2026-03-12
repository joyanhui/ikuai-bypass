package service

import (
	"bytes"
	"context"
	"io"
	"regexp"
	"sync"
	"time"
)

// ansiEscapeRE 用于剥离终端 ANSI 颜色码（避免 WebUI 显示乱码）。
// ansiEscapeRE strips terminal ANSI color sequences for WebUI readability.
var ansiEscapeRE = regexp.MustCompile(`\x1b\[[0-9;]*[a-zA-Z]`)

type LogEntry struct {
	Time time.Time `json:"time"`
	Line string    `json:"line"`
}

// LogBroker 负责收集日志并支持订阅推送（SSE/GUI 实时查看）。
// LogBroker collects logs and supports live subscriptions (SSE/GUI).
type LogBroker struct {
	mu sync.Mutex

	maxLines int
	lines    []LogEntry

	partial bytes.Buffer

	nextSubID int64
	subs      map[int64]chan LogEntry
}

func NewLogBroker(maxLines int) *LogBroker {
	if maxLines <= 0 {
		maxLines = 2000
	}
	return &LogBroker{
		maxLines: maxLines,
		lines:    make([]LogEntry, 0, maxLines),
		subs:     make(map[int64]chan LogEntry),
	}
}

func (b *LogBroker) Writer() io.Writer {
	return b
}

func (b *LogBroker) Write(p []byte) (int, error) {
	b.mu.Lock()
	defer b.mu.Unlock()

	_, _ = b.partial.Write(p)
	for {
		data := b.partial.Bytes()
		idx := bytes.IndexByte(data, '\n')
		if idx < 0 {
			break
		}
		line := string(bytes.TrimRight(data[:idx], "\r"))
		b.partial.Next(idx + 1)
		b.appendLocked(line)
	}
	return len(p), nil
}

func (b *LogBroker) Tail(n int) []LogEntry {
	b.mu.Lock()
	defer b.mu.Unlock()
	if n <= 0 {
		n = 200
	}
	if n > len(b.lines) {
		n = len(b.lines)
	}
	start := len(b.lines) - n
	out := make([]LogEntry, 0, n)
	out = append(out, b.lines[start:]...)
	return out
}

func (b *LogBroker) Subscribe(ctx context.Context, buffer int) <-chan LogEntry {
	if buffer <= 0 {
		buffer = 200
	}
	ch := make(chan LogEntry, buffer)

	b.mu.Lock()
	b.nextSubID++
	id := b.nextSubID
	b.subs[id] = ch
	b.mu.Unlock()

	go func() {
		<-ctx.Done()
		b.mu.Lock()
		if c, ok := b.subs[id]; ok {
			delete(b.subs, id)
			close(c)
		}
		b.mu.Unlock()
	}()

	return ch
}

func stripANSI(s string) string {
	return ansiEscapeRE.ReplaceAllString(s, "")
}

func (b *LogBroker) appendLocked(line string) {
	line = stripANSI(line)
	entry := LogEntry{Time: time.Now(), Line: line}
	b.lines = append(b.lines, entry)
	if len(b.lines) > b.maxLines {
		over := len(b.lines) - b.maxLines
		b.lines = b.lines[over:]
	}

	for _, ch := range b.subs {
		select {
		case ch <- entry:
		default:
		}
	}
}
