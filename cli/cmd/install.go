package cmd

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"slices"
	"strings"

	"github.com/spf13/cobra"
)

var chameleonDir string

var installCmd = &cobra.Command{
	Use:     "install",
	Short:   "Installing shell modules",
	Aliases: []string{"i"},
	RunE:    runInstall,
}

func init() {
	installCmd.Flags().StringVarP(&chameleonDir, "chameleon-dir", "c", "",
		"But where is the chameleon?")
	rootCmd.AddCommand(installCmd)
}

func runInstall(cmd *cobra.Command, args []string) error {
	if len(args) == 0 {
		return fmt.Errorf("укажите хотя бы имя одного крейта")
	}

	input := strings.Join(args, ",")
	parts := strings.Split(input, ",")

	var crates []string
	for _, part := range parts {
		part = strings.TrimSpace(part)

		if part != "" {
			crates = append(crates, part)
		}
	}

	if len(crates) == 0 {
		return fmt.Errorf("не указано ни одного крейта")
	}

	crateNames := [...]string{"launcher", "panel", "notifications"}

	for _, crate := range crates {
		if !slices.Contains(crateNames[:], crate) {
			return fmt.Errorf("Crate not found")
		}

		fmt.Printf("Installing %s...\n", crate)

		if err := installCrate(crate, chameleonDir); err != nil {
			return fmt.Errorf("ошибка при установке %s: %w", crate, err)
		}
	}
	return nil
}

func installCrate(name string, chameleonDir string) error {
	crateDir := filepath.Join(chameleonDir, "crates", name)
	if _, err := os.Stat(crateDir); os.IsNotExist(err) {
		return fmt.Errorf("директория крейта не найдена: %s", crateDir)
	}

	mainRSPath := filepath.Join(crateDir, "src", "main.rs")

	if _, err := os.Stat(mainRSPath); os.IsNotExist(err) {
		return fmt.Errorf("крейт %s не является бинарным (отсутствует src/main.rs)", name)
	}

	buildCmd := exec.Command("cargo", "build", "--release")
	buildCmd.Dir = crateDir
	buildCmd.Stdout = os.Stdout
	buildCmd.Stderr = os.Stderr

	if err := buildCmd.Run(); err != nil {
		return fmt.Errorf("ошибка сборки cargo: %w", err)
	}

	binaryName := "chameleon-" + name

	srcBinary := filepath.Join(crateDir, "target", "release", binaryName)
	destBinary := filepath.Join("/opt/chameleon", name)

	if err := os.MkdirAll("/opt/chameleon", 0755); err != nil {
		return fmt.Errorf("Cant create /opt/chameleon: %w", err)
	}

	input, err := os.ReadFile(srcBinary)

	if err != nil {
		return fmt.Errorf("не удалось прочитать собранный файл %s: %w", srcBinary, err)
	}

	if err := os.WriteFile(destBinary, input, 0755); err != nil {
		return fmt.Errorf("не удалось записать в %s: %w", destBinary, err)
	}

	fmt.Printf("Успешно установлен %s в %s\n", binaryName, destBinary)
	return nil
}
