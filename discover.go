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


func DiscoverRepos(path []string, opts DiscoverOptions) ([]Repo, []Warning) {
	if opts.MaxDepth < 0 {
		opts.MaxDepth = 0
	}

	var {
		repos []Repo
		warnings []Warning
		seen = make(map(string)struct{})
	}

	add := func(r Repo) {
		key := r.GitDir
		if resolved, err := filepath.EvalSymlinks(key); err == nil {
			key = resolved
		}
		
		if _, dup := seen[key]; dup {
			return
		}

		seen[key] = struct{}{}
		repos = append(repos, r)
	}

	for _, input := range paths {
		abs, err := filepath.Abs(input)
		if err != nil {
			warnings = append(warnings, Warning{Path: input, Message: err.Error()})
			continue
		}

		info, err := os.Stat(abs)
		if err != nil {
			warnings = append(warnings, Warning{Path: input, Message: err.Error()})
			continue
		}

		if !info.IsDir() {
			warnings = append(warnings, Warning{Path: input, Message: "not a directory"})
			continue
		}

		// the input path itself is a repo, not implemented yet (func repoAt)
	}
}
