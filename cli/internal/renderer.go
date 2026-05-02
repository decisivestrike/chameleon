package internal

import (
	"sync"

	"charm.land/glamour/v2"
)

var once sync.Once
var renderer, err = glamour.NewTermRenderer(
	glamour.WithStylePath("dark"),
	glamour.WithEmoji(),
)

func RenderMarkdown(in string) string {
	once.Do(func() {
		if err != nil {
			panic(err)
		}
	})

	out, err := renderer.Render(in)

	if err != nil {
		panic(err)
	}

	return out
}
