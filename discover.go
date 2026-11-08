package main

import (
	"io/fs"
	"os"
	"path/filepath"
	"sort"
)

type Repo struct {
	Path string
	GitDir string
}

type Warning struct {
	Path string
	Message string
}

type DiscoverOptions struct {
	MaxDepth int
	Exclude []string
}

func DiscoverRepos(paths []string, opts DiscoverOptions) ([]Repo, []Warning) {
	if opts.MaxDepth < 0 {
		opts.MaxDepth = 0
	}

	var (
		repos []Repo
		warnings []Warning
		seen = make(map[string]struct{})
	)

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

		if repo, ok := repoAt(abs); ok {
			add(repo)
			continue
		}

		walkErr := walkForRepos(abs, opts, add, &warnings)
		if walkErr != nil {
			warnings = append(warnings, Warning{Path: input, Message: walkErr.Error()})
		}
	}

	sort.Slice(repos, func(i, j int) bool { return repos[i].Path < repos[j].Path })

	return repos, warnings
}

func walkForRepos(root string, opts DiscoverOptions, add func(Repo), warnings *[]Warning) error {
	rootDepth := depthOf(root)

	return filepath.WalkDir(root, func(path string, d fs.DirEntry, err error) error {
		if err != nil {
			*warnings = append(*warnings, Warning{Path: path, Message: err.Error()})

			if d != nil && d.IsDir() {
				return fs.SkipDir
			}
			return nil
		}

		if !d.IsDir() {
			return nil
		}

		if path != root && matchesAny(filepath.Base(path), opts.Exclude) {
			return fs.SkipDir
		}

		if depthOf(path)-rootDepth > opts.MaxDepth {
			return fs.SkipDir
		}

		if repo, ok := repoAt(path); ok {
			add(repo)
			return fs.SkipDir
		}

		return nil
	})
}

func repoAt(dir string) (Repo, bool) {
	gitPath := filepath.Join(dir, ".git")
	info, err := os.Lstat(gitPath)

	if err == nil {
		if info.IsDir() || info.Mode().IsRegular() {
			return Repo{Path: dir, GitDir: gitPath}, true
		}
	}

	if isBareRepo(dir) {
		return Repo{Path: dir, GitDir: dir}, true
	}

	return Repo{}, false
}

func isBareRepo(dir string) bool {
	if _, err := os.Stat(filepath.Join(dir, "HEAD")); err != nil {
		return false
	}

	info, err := os.Stat(filepath.Join(dir, "objects"))
	if err != nil || !info.IsDir() {
		return false
	}

	return true
}

func matchesAny(name string, patterns []string) bool {
	for _, pat := range patterns {
		if ok, _ := filepath.Match(pat, name); ok {
			return true
		}
	}
	return false
}

func depthOf(p string) int {
	p = filepath.Clean(p)
	vol := filepath.VolumeName(p)
	p = p[len(vol):]
	p = filepath.ToSlash(p)
	n := 0

	for _, r := range p {
		if r == '/' {
			n++
		}
	}
	return n
}
