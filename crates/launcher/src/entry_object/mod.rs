mod imp;

use freedesktop_desktop_entry::DesktopEntry;
use glib::Object;
use gtk::glib;
use thiserror::Error;

const TRUE_LITERAL: &str = "true";

glib::wrapper! {
    pub struct ApplicationEntry(ObjectSubclass<imp::EntryObject>);
}

#[derive(Debug, Error)]
pub enum EntryConversionError {
    #[error("Type should be Application")]
    NotApplication,

    #[error("Can't get `Desktop Entry` group")]
    NoGroup,

    #[error("This application exists, but don't display it in the menus")]
    NoDisplay,

    #[error("Entry is hidden")]
    Hidden,

    #[error("Can't find this app in $PATH")]
    NotInPath,

    #[error("Missing required field: {0}")]
    MissingField(&'static str),
}

impl ApplicationEntry {
    pub fn new(
        name: String,
        exec: String,
        comment: String,
        icon: String,
        terminal: bool,
    ) -> Self {
        Object::builder()
            .property("name", name)
            .property("exec", exec)
            .property("comment", comment)
            .property("icon", icon)
            .property("terminal", terminal)
            .build()
    }

    /// Removes field codes from a desktop entry `Exec` line.
    ///
    /// For more details, see
    /// [here](https://specifications.freedesktop.org/desktop-entry/latest/exec-variables.html)
    fn remove_field_codes(exec: &str) -> String {
        const FIELD_CODES: &[&str] = &[
            "%f", "%F", "%u", "%U", "%d", "%D", "%n", "%N", "%v", "%m", "%i",
            "%c", "%k",
        ];

        let mut exec = exec.to_string();

        for code in FIELD_CODES {
            exec = exec.replace(code, "");
        }

        let parts: Vec<_> = exec.split_whitespace().collect();

        parts.join(" ")
    }
}

impl TryFrom<DesktopEntry> for ApplicationEntry {
    type Error = EntryConversionError;

    fn try_from(entry: DesktopEntry) -> Result<Self, Self::Error> {
        let group = entry
            .groups
            .desktop_entry()
            .ok_or(EntryConversionError::NoGroup)?;

        if let Some(TRUE_LITERAL) = group.entry("NoDisplay") {
            return Err(EntryConversionError::NoDisplay);
        }

        if let Some(TRUE_LITERAL) = group.entry("Hidden") {
            return Err(EntryConversionError::Hidden);
        }

        let name = group
            .entry("Name")
            .ok_or(EntryConversionError::MissingField("Name"))?
            .to_string();

        let exec = group
            .entry("Exec")
            .map(Self::remove_field_codes)
            .ok_or(EntryConversionError::MissingField("Exec"))?;

        let comment = group
            .entry("Comment")
            .map(String::from)
            .unwrap_or(String::new());

        let icon = group
            .entry("Icon")
            .map(String::from)
            .unwrap_or(String::new());

        let terminal = group
            .entry("Terminal")
            .map(|e| e == "true")
            .unwrap_or(false);

        Ok(Self::new(name, exec, comment, icon, terminal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_field_codes() {
        assert_eq!(
            ApplicationEntry::remove_field_codes("firefox %u"),
            "firefox"
        );
        assert_eq!(
            ApplicationEntry::remove_field_codes("zeditor %U"),
            "zeditor"
        );
        assert_eq!(
            ApplicationEntry::remove_field_codes("myapp %f %F %u %U"),
            "myapp"
        );
        assert_eq!(
            ApplicationEntry::remove_field_codes("app %f arg1 %U arg2"),
            "app arg1 arg2"
        );
    }
}
