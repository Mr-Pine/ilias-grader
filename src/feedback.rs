use std::{
    ffi::{OsStr, OsString},
    fs::read_dir,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm};
use ilias::{client::IliasClient, exercise::grades::GradePage, local_file::NamedLocalFile};
use log::{debug, info};
use regex::Regex;

pub fn upload_feedback(
    grade_page: &GradePage,
    no_confim: bool,
    feedback_dir_path: &Path,
    suffix: &str,
    ilias_client: &IliasClient,
) -> Result<()> {
    let confirmation = no_confim
        || Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Upload feedback from {} to {}?",
                feedback_dir_path
                    .to_str()
                    .expect("Could not display feedback_dir"),
                grade_page.name
            ))
            .interact()
            .expect("Interaction with confirmation prompt failed");
    if !confirmation {
        println!("Aborted");
        return Ok(());
    }

    let feedback_dir = read_dir(feedback_dir_path)?;
    let feedback_entries = feedback_dir.collect::<std::result::Result<Vec<_>, _>>()?;

    let flattened_filename_regex = Regex::new("@student\\.kit\\.edu_\\d+-(?<filename>.*)")?;

    for submission in &grade_page.submissions {
        for feedback_entry in &feedback_entries {
            let path = feedback_entry.path();
            let relative_path = path.strip_prefix(feedback_dir_path)?;

            let relative_path_string = relative_path.as_os_str();

            if relative_path_string
                .to_str()
                .map_or(false, |str| str.starts_with(&submission.identifier))
                || relative_path_string.to_str().map_or(false, |str| {
                    str.starts_with(&submission.identifier.replace(" ", "_"))
                })
            {
                if feedback_entry.file_type()?.is_dir() {
                    let user_dirs = read_dir(&path)?;

                    for user_dir in user_dirs {
                        let user_dir = user_dir?;
                        if !user_dir.file_type()?.is_dir() {
                            return Err(anyhow!("Unsupported feedback file structure"));
                        }

                        let user_files = read_dir(user_dir.path())?;
                        for user_file in user_files {
                            let user_file = user_file?;

                            if !user_file.file_type()?.is_file() {
                                return Err(anyhow!("Unsupported feedback file structure"));
                            }

                            let target_filename = append_suffix(
                                user_file
                                    .file_name()
                                    .to_str()
                                    .context("Could not convert filename to &str")?,
                                suffix,
                            )?;

                            info!("Uploading {} to {}", target_filename, submission.identifier);
                            let local_file = NamedLocalFile {
                                name: target_filename,
                                path: user_file.path(),
                            };

                            submission.upload(local_file, ilias_client)?;
                        }
                    }
                } else {
                    let target_filename = feedback_entry.file_name();
                    let target_filename = flattened_filename_regex
                        .captures(
                            target_filename
                                .to_str()
                                .context("Could not convert filename to &str")?,
                        )
                        .context("Could not extract raw filename")?
                        .name("filename")
                        .context("No filename captured")?
                        .as_str();
                    let target_filename = append_suffix(target_filename, suffix)?;

                    info!("Uploading {} to {}", target_filename, submission.identifier);
                    let local_file = NamedLocalFile {
                        name: target_filename,
                        path: feedback_entry.path(),
                    };

                    submission.upload(local_file, ilias_client)?;
                }
            } else {
                debug!(
                    "Unhandled file {:?}: Did not start with {:?} or {:?}",
                    relative_path,
                    submission.identifier,
                    submission.identifier.replace(" ", "_")
                );
            }
        }
    }

    Ok(())
}

fn append_suffix(name: &str, suffix: &str) -> Result<String> {
    if suffix.is_empty() {
        Ok(name.to_string())
    } else {
        let parsed_name = PathBuf::from_str(name)?;
        let mut appended_name = PathBuf::new();
        appended_name.set_file_name(format!(
            "{}{}",
            parsed_name
                .file_stem()
                .context("Parsed name had no stem")?
                .to_str()
                .unwrap(),
            suffix
        ));
        appended_name.set_extension(parsed_name.extension().unwrap_or(&OsString::new()));

        Ok(appended_name.to_str().unwrap().to_string())
    }
}
