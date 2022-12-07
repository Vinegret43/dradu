use egui_extras::RetainedImage;

use std::fs::{self, File, ReadDir};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::utils;
use crate::DraduError;

pub struct AssetDirHandler {
    asset_dir: Option<PathBuf>,
}

// XXX WARNING XXX: Editing this code may result in new vulnerabilities since it
// accesses and modifies the filesystem. You can use `.validate_path()?` method to
// check for absolute paths or directory traversals, read the comment above it for
// further information

impl AssetDirHandler {
    pub fn new() -> Self {
        let asset_dir = if let Some(mut path) = utils::local_dir() {
            path.push("assets");
            if !path.exists() {
                #[allow(unused)]
                {
                    fs::create_dir_all(&path);
                }
            }
            Some(path)
        } else {
            None
        };
        Self { asset_dir }
    }

    // The former path should be relative to the directory with assets. This method
    // actually joins it with the path to that directory and returns the new path. If the
    // former path is absolute or has directory traversals (..), it returns an error.
    // Since all paths are user input, you MUST pass them through this method and use `?`
    // after it, otherwise users will be able to access ANY file on the computer
    pub fn validate_path<P: AsRef<Path>>(&self, path: P) -> Result<PathBuf, DraduError> {
        if let Some(ref asset_dir) = self.asset_dir {
            let path = asset_dir.join(path);
            if path.starts_with(asset_dir) && !utils::directory_traversal(&path) {
                Ok(path)
            } else {
                Err(DraduError::InvalidPath)
            }
        } else {
            Err(DraduError::ProjectDirNotFound)
        }
    }

    pub fn create_dir<P: AsRef<Path>>(&self, path: P) -> Result<(), DraduError> {
        let path = self.validate_path(path)?;
        fs::create_dir_all(path)?;
        Ok(())
    }

    pub fn list_entries<P: AsRef<Path>>(&self, path: P) -> Result<ReadDir, DraduError> {
        let path = self.validate_path(path)?;
        Ok(fs::read_dir(path)?)
    }

    pub fn get_filesize<P: AsRef<Path>>(&self, path: P) -> Result<u64, DraduError> {
        let path = self.validate_path(path)?;
        Ok(fs::metadata(path)?.len())
    }

    // TODO: Add filesize check
    pub fn copy_into<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        from: P,
        to: Q,
    ) -> Result<u64, DraduError> {
        let mut to = self.validate_path(to)?;
        if to.is_dir() {
            to = to.join(from.as_ref().file_name().ok_or(DraduError::InvalidPath)?);
        }
        Ok(fs::copy(from, to)?)
    }

    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>, DraduError> {
        let path = self.validate_path(path)?;
        let mut file = File::open(&path)?;
        let mut b = Vec::new();
        file.read_to_end(&mut b)?;
        Ok(b)
    }

    pub fn get_retained_image<P: AsRef<Path>>(&self, path: P) -> Result<RetainedImage, DraduError> {
        let path = self.validate_path(path)?;
        let mut file = File::open(&path)?;
        let mut b = Vec::new();
        file.read_to_end(&mut b)?;
        match RetainedImage::from_image_bytes(path.to_str().unwrap(), &b[..]) {
            Ok(img) => Ok(img),
            Err(s) => Err(DraduError::ImageLoadError(s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::fs::AssetDirHandler;
    #[test]
    fn test_validate_path() {
        let fs = AssetDirHandler::new();
        fs.validate_path("path/to/smth").unwrap();
        fs.validate_path("./path/to/smth").unwrap();
        fs.validate_path("/abspath/to/smth").unwrap_err();
        fs.validate_path("traversal/../../smth").unwrap_err();
    }
}
