use std::{fs::{self, File}, io, path::{Path, PathBuf}, str::FromStr};

use anyhow::Result;
use ilias::{client::IliasClient, exercise::grades::GradePage};
use tempfile::NamedTempFile;
use zip::ZipArchive;

pub fn download_submissions(
    grade_page: &GradePage,
    to: &Path,
    extract: bool,
    flatten: bool,
    ilias_client: &IliasClient,
) -> Result<()> {
    if extract {
        let tempfile = NamedTempFile::new()?;
        let temppath = tempfile.into_temp_path();
        grade_page.download_all_submissions_zip(ilias_client, &temppath)?;

        let zipfile = File::open(temppath)?;
        let mut zip_archive = ZipArchive::new(zipfile)?;

        for i in 0..zip_archive.len() {
            let mut file = zip_archive.by_index(i)?;
            if !file.is_file() {
                continue;
            }

            let mut zip_path = file.enclosed_name().expect("Malformed path in zip file");
            zip_path = drop_components(&mut zip_path, 2)?.to_path_buf();

            if flatten {
                zip_path = flatten_path(&zip_path)?;
            }

            let file_path = to.join(zip_path);

            let dir = file_path.parent().expect("Could not get containing directory");
            fs::create_dir_all(dir)?;

            let mut target_file = File::create(file_path)?;
            io::copy(&mut file, &mut target_file)?;
        }
    } else {
        grade_page.download_all_submissions_zip(ilias_client, to)?;
        println!("Downloaded zip file to {}", to.to_str().unwrap_or("<unknown>"));
    }

    Ok(())
}

fn drop_components(path: &mut PathBuf, count: usize) -> Result<&Path> {
    let prefix = path.components().take(count).collect::<PathBuf>();
    Ok(path.strip_prefix(prefix)?)
}

fn flatten_path(path: &Path) -> Result<PathBuf> {
    let components = path.components().map(|component| component.as_os_str().to_str().expect("Weird filename").replace(" ", "_")).collect::<Vec<_>>();
    let path = components.join("-");

    Ok(PathBuf::from_str(&path)?)
}
