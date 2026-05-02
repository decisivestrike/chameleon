package cmd

import (
	. "chameleon/internal"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"slices"
	"strings"

	"github.com/spf13/cobra"
)

// Modules available for installation
var modules = [...]string{"launcher", "notifications", "panel", "widgets"}

var installCmd = &cobra.Command{
	Use:     "install",
	Short:   "Allows you to install shell modules",
	Aliases: []string{"i"},
	RunE:    runInstall,
}

func init() {
	rootCmd.AddCommand(installCmd)
}

func moduleNamesFromArgs(args []string) ([]string, error) {
	input := strings.Join(args, ",")
	parts := strings.Split(input, ",")
	var modules []string

	for _, part := range parts {
		part = strings.TrimSpace(part)

		if part != "" {
			modules = append(modules, part)
		}
	}

	return modules, nil
}

func runInstall(cmd *cobra.Command, args []string) error {
	if len(args) == 0 {
		fmt.Println("picker")
	}

	modules, err := moduleNamesFromArgs(args)

	if err != nil {
		return err
	}

	for _, module := range modules {
		fmt.Printf("Installing %s...\n", module)

		if err := installModule(module); err != nil {
			return fmt.Errorf("Ошибка при установке %s: %w", module, err)
		}
	}

	return nil
}

func installModule(name string) error {
	if !slices.Contains(modules[:], name) {
		return fmt.Errorf("\"%s\" in not a shell module!\n\nModules: %v", name, modules)
	}

	binaryName := "chameleon-" + name

	buildCmd := exec.Command("cargo", "build", "--bin", binaryName)
	buildCmd.Dir = ChameleonRoot
	buildCmd.Stdout = os.Stdout
	buildCmd.Stderr = os.Stderr

	if err := buildCmd.Run(); err != nil {
		return fmt.Errorf("Cargo error: %w", err)
	}

	srcBinary := filepath.Join(ChameleonRepo, "target", "release", binaryName)
	destBinary := filepath.Join(ChameleonRepo, name)

	if err := os.MkdirAll(ChameleonBin, 0755); err != nil {
		return fmt.Errorf("Cant create %s: %w", ChameleonBin, err)
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
