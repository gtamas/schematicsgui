use std::{
    fs::{self, read_dir},
    io,
    path::PathBuf,
};

pub struct FileUtils {}

impl FileUtils {
    pub fn read_fs_entries(path: &PathBuf, dirs: bool) -> Vec<PathBuf> {
        read_dir(path)
            .unwrap()
            .filter_map(|f| {
                let entry = f.unwrap();
                if dirs {
                    if !entry.file_type().unwrap().is_dir() {
                        return None;
                    }
                } else if !entry.file_type().unwrap().is_file() {
                    return None;
                }

                Some(entry.path())
            })
            .collect()
    }

    pub fn read_fs_entries_recursive(
        dir: &PathBuf,
        allowed: &Option<Vec<&str>>,
    ) -> io::Result<Vec<PathBuf>> {
        let mut result: Vec<PathBuf> = vec![];
        let allowed_dirs = allowed.clone().unwrap_or_default();
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    result.extend(Self::read_fs_entries_recursive(&path, allowed)?);
                } else if allowed_dirs.is_empty()
                    || path.ancestors().any(|f| {
                        allowed_dirs.contains(&f.file_name().unwrap_or_default().to_str().unwrap())
                    })
                {
                    result.push(path);
                }
            }
        }
        Ok(result)
    }
}
