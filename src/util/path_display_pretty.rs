use std::{
    fmt::{Debug, Display, Formatter},
    path::{Path, PathBuf},
};

pub struct DisplayPretty<'a>(&'a Path);

impl Debug for DisplayPretty<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("\"")?;
        Display::fmt(&self, f)?;
        f.write_str("\"")?;
        Ok(())
    }
}

impl Display for DisplayPretty<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let path = self.0.components().collect::<PathBuf>();

        if let Ok(path_stripped) = self.0.strip_prefix(dirs::home_dir().unwrap_or_default()) {
            f.write_str("~/")?;
            f.write_str(&path_stripped.to_string_lossy())?;
        } else {
            f.write_str(&path.to_string_lossy())?;
        }

        if path.is_dir() {
            f.write_str("/")?;
        }

        Ok(())
    }
}

pub trait PathDisplayPretty {
    fn display_pretty(&self) -> DisplayPretty;
}

impl PathDisplayPretty for Path {
    fn display_pretty(&self) -> DisplayPretty {
        DisplayPretty(self)
    }
}
