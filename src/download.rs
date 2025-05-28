use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
    str::FromStr,
};

use ilias::{client::IliasClient, exercise::grades::GradePage};
use log::info;
use snafu::{ResultExt, Whatever};
use tempfile::NamedTempFile;
use zip::ZipArchive;

pub fn download_submissions(
    grade_page: &GradePage,
    to: &Path,
    extract: bool,
    flatten: bool,
    ilias_client: &IliasClient,
) -> Result<(), Whatever> {
    info!("Downloading all submissions for {}", grade_page.name);
    if extract {
        let tempfile = NamedTempFile::new().whatever_context("Could not create temp file")?;
        let temppath = tempfile.into_temp_path();
        grade_page
            .download_all_submissions_zip(ilias_client, &temppath)
            .whatever_context("Failed to download submissions zip")?;

        let zipfile = File::open(temppath).whatever_context("Unable to open tempfile")?;
        let mut zip_archive =
            ZipArchive::new(zipfile).whatever_context("Unable to read submissions zip")?;

        for i in 0..zip_archive.len() {
            let mut file = zip_archive
                .by_index(i)
                .whatever_context(format!("Could not get entry {i} in submissions zip"))?;
            if !file.is_file() {
                continue;
            }

            let mut zip_path = file.enclosed_name().expect("Malformed path in zip file");
            let original_zip_path = zip_path.clone();
            zip_path = drop_components(&mut zip_path, 2)
                .whatever_context("Could not drop first two path components")?
                .to_path_buf();

            if flatten {
                zip_path = flatten_path(&zip_path).whatever_context("Could not flatten path")?;
            }

            let file_path = to.join(zip_path);

            if file_path.is_dir() {
                continue;
            }
            info!(
                "Extracting '{}' to '{}'",
                original_zip_path
                    .to_str()
                    .expect("Could not display zip path"),
                file_path.to_str().expect("Could not display target path")
            );

            let dir = file_path
                .parent()
                .expect("Could not get containing directory");
            fs::create_dir_all(dir).whatever_context(format!("Could not create {dir:?}"))?;

            let mut target_file = File::create(&file_path)
                .whatever_context(format!("Could not create {file_path:?}"))?;
            io::copy(&mut file, &mut target_file)
                .whatever_context(format!("Could not write to {target_file:?}"))?;
        }
    } else {
        grade_page.download_all_submissions_zip(ilias_client, to)?;
        info!(
            "Downloaded zip file to {}",
            to.to_str().unwrap_or("<unknown>")
        );
    }

    Ok(())
}

fn drop_components(path: &mut Path, count: usize) -> Result<&Path, Whatever> {
    let prefix = path.components().take(count).collect::<PathBuf>();
    path.strip_prefix(prefix.clone())
        .whatever_context(format!("Could not strip prefix {prefix:?} from {path:?}"))
}

fn flatten_path(path: &Path) -> Result<PathBuf, Whatever> {
    let components = path
        .components()
        .map(|component| {
            component
                .as_os_str()
                .to_str()
                .expect("Weird filename")
                .replace(' ', "_")
        })
        .collect::<Vec<_>>();
    let path = components.join("-");

    PathBuf::from_str(&path).whatever_context(format!("Could not convert {path} to PathBuf"))
}
