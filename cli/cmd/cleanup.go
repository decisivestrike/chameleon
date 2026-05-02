package cmd

import (
	. "chameleon/internal"
	"os"
	"os/exec"

	"github.com/spf13/cobra"
)

var clearCacheCmd = &cobra.Command{
	Use:   "cleanup",
	Short: "Calls cargo clean",
	Long: `
Remove artifacts from the target directory that Cargo has generated in the past.

Will delete the entire target directory.`,

	RunE: func(cmd *cobra.Command, args []string) error {
		clearCmd := exec.Command("cargo", "clean")
		clearCmd.Dir = ChameleonRoot
		clearCmd.Stdout = os.Stdout
		clearCmd.Stderr = os.Stderr

		return clearCmd.Run()
	},
}

func init() {
	rootCmd.AddCommand(clearCacheCmd)
}
