package main

import {
	"flag"
	"fmt"
	"os"
	"strings"
}

type multiFlag []string

func (m *multiFlag) String() string { return strings.Join(*m, ",") }

func (m *multiFlag) Set(v String) error {
	*m = append(*m, v)
	return nil
}
