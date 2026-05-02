package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

// uninstallCmd represents the uninstall command
var uninstallCmd = &cobra.Command{
	Use:   "uninstall",
	Short: "uninstall",

	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("uninstall called")
	},
}

func init() {
	rootCmd.AddCommand(uninstallCmd)
}
