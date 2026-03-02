package ikuai_api4

import (
	"ikuai-bypass/pkg/ikuai_common"
	"testing"
)

func TestBuildCustomIspChunkComment(t *testing.T) {
	tests := []struct {
		name  string
		index int
		want  string
	}{
		{name: "first chunk", index: 0, want: ikuai_common.NEW_COMMENT},
		{name: "second chunk", index: 1, want: ikuai_common.NEW_COMMENT + "-2"},
		{name: "third chunk", index: 2, want: ikuai_common.NEW_COMMENT + "-3"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := buildCustomIspChunkComment(tt.index)
			if got != tt.want {
				t.Fatalf("buildCustomIspChunkComment(%d) = %q, want %q", tt.index, got, tt.want)
			}
		})
	}
}

func TestParseCustomIspChunkIndexFromComment(t *testing.T) {
	tests := []struct {
		name    string
		comment string
		want    int
		ok      bool
	}{
		{name: "new comment first", comment: ikuai_common.NEW_COMMENT, want: 1, ok: true},
		{name: "new comment with dash index", comment: ikuai_common.NEW_COMMENT + "-2", want: 2, ok: true},
		{name: "new comment with underscore index", comment: ikuai_common.NEW_COMMENT + "_3", want: 3, ok: true},
		{name: "legacy comment first", comment: ikuai_common.COMMENT_IKUAI_BYPASS, want: 1, ok: true},
		{name: "legacy comment with index", comment: ikuai_common.COMMENT_IKUAI_BYPASS + "_4", want: 4, ok: true},
		{name: "invalid comment", comment: "random", want: 0, ok: false},
		{name: "invalid suffix", comment: ikuai_common.NEW_COMMENT + "-x", want: 0, ok: false},
		{name: "empty comment", comment: "", want: 0, ok: false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, ok := parseCustomIspChunkIndexFromComment(tt.comment)
			if got != tt.want || ok != tt.ok {
				t.Fatalf("parseCustomIspChunkIndexFromComment(%q) = (%d, %v), want (%d, %v)", tt.comment, got, ok, tt.want, tt.ok)
			}
		})
	}
}

func TestParseCustomIspChunkIndexFromName(t *testing.T) {
	tag := "国内"
	tests := []struct {
		name string
		raw  string
		want int
		ok   bool
	}{
		{name: "name with suffix", raw: buildTagName(tag) + "2", want: 2, ok: true},
		{name: "name without suffix", raw: buildTagName(tag), want: 0, ok: false},
		{name: "invalid suffix", raw: buildTagName(tag) + "x", want: 0, ok: false},
		{name: "other prefix", raw: "OTHER2", want: 0, ok: false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, ok := parseCustomIspChunkIndexFromName(tt.raw, tag)
			if got != tt.want || ok != tt.ok {
				t.Fatalf("parseCustomIspChunkIndexFromName(%q, %q) = (%d, %v), want (%d, %v)", tt.raw, tag, got, ok, tt.want, tt.ok)
			}
		})
	}
}
