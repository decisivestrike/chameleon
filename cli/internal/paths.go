package internal

import (
	"os"
	"path/filepath"
)

// Chameleon root folder path
//
//	$HOME/.local/chameleon
var ChameleonRoot = getChameleonRoot()

// Chameleon bin folder path
//
//	$HOME/.local/chameleon/bin
var ChameleonBin = filepath.Join(ChameleonRoot, "bin")

// Chameleon repo folder path
//
//	$HOME/.local/chameleon/repo
var ChameleonRepo = filepath.Join(ChameleonRoot, "repo")

func getChameleonRoot() string {
	home, err := os.UserHomeDir()

	if err != nil {
		panic(err)
	}

	return home + "/.local/chameleon"
}
