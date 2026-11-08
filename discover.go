package main

import (
	"io/fs"
	"os"
	"path/filepath"
	"sort"
)

type Repo struct {
	Path   string
	GitDir string
}

type Warning struct {
	Path    string
	Message string
}

type DiscoverOptions struct {
	MaxDepth int
	Exclude []string
}
