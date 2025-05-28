use std::{
    ffi::OsString,
    fs::read_dir,
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};

use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect};
use ilias::{
    client::IliasClient,
    exercise::grades::{submission::GradeSubmission, GradePage},
    local_file::NamedLocalFile,
};
use log::{debug, info};
use regex::Regex;
use snafu::{whatever, OptionExt, ResultExt, Whatever};

static FLATTENED_FILENAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("@student\\.kit\\.edu_\\d+-(?<filename>.*)").expect("Invalid regex pattern")
});

pub fn upload_points(grade_page: &GradePage, ilias_client: &IliasClient) -> Result<(), Whatever> {
    let mut changed_submissions = vec![];
    while let Some(selected_student) = FuzzySelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick a student")
        .default(0)
        .items(grade_page.submissions.as_slice())
        .interact_opt()
        .unwrap()
    {
        let selected_points: String = dialoguer::Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Points:")
            .interact_text()
            .whatever_context("Inputting points failed.")?;

        let mut submission = grade_page
            .submissions
            .get(selected_student)
            .whatever_context("Could not get selected_student")?
            .clone();
        submission.points = selected_points;
        changed_submissions.push(submission);
    }
    debug!("{changed_submissions:?}",);
    grade_page.update_points(ilias_client, &changed_submissions)
}

pub fn upload_feedback(
    grade_page: &GradePage,
    no_confim: bool,
    feedback_dir_path: &Path,
    filter_expr: Option<&Regex>,
    suffix: Option<impl AsRef<str>>,
    ilias_client: &IliasClient,
) -> Result<(), Whatever> {
    let confirmation = no_confim
        || Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Upload feedback{} from {} to {}?",
                filter_expr
                    .map(|filter_expr| format!(" matching {}", filter_expr.as_str()))
                    .unwrap_or_default(),
                feedback_dir_path
                    .to_str()
                    .expect("Could not display feedback_dir"),
                grade_page.name,
            ))
            .interact()
            .expect("Interaction with confirmation prompt failed");
    if !confirmation {
        info!("Aborted");
        return Ok(());
    }

    let feedback_dir =
        read_dir(feedback_dir_path).whatever_context("Unable to read feedback directory")?;
    let feedback_entries = feedback_dir
        .collect::<Result<Vec<_>, _>>()
        .whatever_context("Could not get directory entries")?;

    debug!("Available submissions: {:?}", grade_page.submissions);
    for submission in &grade_page.submissions {
        for feedback_entry in &feedback_entries {
            let path = feedback_entry.path();
            let relative_path = path
                .strip_prefix(feedback_dir_path)
                .whatever_context("Could not strip prefix from file")?;

            let relative_path_string = relative_path.as_os_str();

            if relative_path_string
                .to_str()
                .is_some_and(|str| str.starts_with(&submission.identifier))
                || relative_path_string
                    .to_str()
                    .is_some_and(|str| str.starts_with(&submission.identifier.replace(' ', "_")))
            {
                debug!("Feedback entry {feedback_entry:?}");
                if feedback_entry
                    .file_type()
                    .whatever_context(format!(
                        "Could not determine filetype for entry {feedback_entry:?}"
                    ))?
                    .is_dir()
                {
                    let user_dir = feedback_entry;

                    let user_files =
                        read_dir(user_dir.path()).whatever_context("Could not get user files")?;
                    for user_file in user_files {
                        let user_file = user_file.whatever_context("Bad user file")?;

                        if !user_file
                            .file_type()
                            .whatever_context(format!(
                                "Could not determine filetype for user entry {user_file:?}"
                            ))?
                            .is_file()
                        {
                            whatever!("Unsupported feedback file structure: Expected file");
                        }

                        let target_filename = user_file.file_name();
                        let target_filename = target_filename
                            .to_str()
                            .whatever_context("Could not convert filename to &str")?;

                        upload_filtered_file_with_suffix(
                            user_file.path(),
                            target_filename,
                            filter_expr,
                            suffix.as_ref(),
                            submission,
                            ilias_client,
                        )?;
                    }
                } else {
                    let target_filename = feedback_entry.file_name();
                    let target_filename = FLATTENED_FILENAME_REGEX
                        .captures(
                            target_filename
                                .to_str()
                                .whatever_context("Could not convert filename to &str")?,
                        )
                        .whatever_context("Could not extract raw filename")?
                        .name("filename")
                        .whatever_context("No filename captured")?
                        .as_str();

                    upload_filtered_file_with_suffix(
                        feedback_entry.path(),
                        target_filename,
                        filter_expr,
                        suffix.as_ref(),
                        submission,
                        ilias_client,
                    )?;
                }
            } else {
                debug!(
                    "Unhandled file {:?}: Did not start with {:?} or {:?}",
                    relative_path,
                    submission.identifier,
                    submission.identifier.replace(' ', "_")
                );
            }
        }
    }

    Ok(())
}

fn upload_filtered_file_with_suffix(
    path: PathBuf,
    target_filename: &str,
    filter_expr: Option<&Regex>,
    suffix: Option<impl AsRef<str>>,
    submission: &GradeSubmission,
    ilias_client: &IliasClient,
) -> Result<(), Whatever> {
    if !filter_expr.is_none_or(|filter_expr| filter_expr.is_match(target_filename)) {
        debug!("Skipped uploading {target_filename}");
        return Ok(());
    }

    let target_filename = match suffix {
        Some(ref suffix) => append_suffix(target_filename, suffix.as_ref())?,
        None => target_filename.to_owned(),
    };

    info!("Uploading {} to {}", target_filename, submission.identifier);
    let local_file = NamedLocalFile {
        name: target_filename.clone(),
        path,
    };

    submission
        .upload(local_file, ilias_client)
        .whatever_context(format!(
            "Could not upload {} to {}",
            target_filename, submission.identifier
        ))?;

    Ok(())
}

fn append_suffix(name: &str, suffix: &str) -> Result<String, Whatever> {
    let parsed_name =
        PathBuf::from_str(name).whatever_context("Could not parse name to PathBuf")?;
    let mut appended_name = PathBuf::new();
    appended_name.set_file_name(format!(
        "{}{}",
        parsed_name
            .file_stem()
            .whatever_context("Parsed name had no stem")?
            .to_str()
            .unwrap(),
        suffix
    ));
    appended_name.set_extension(parsed_name.extension().unwrap_or(&OsString::new()));

    Ok(appended_name.to_str().unwrap().to_string())
}
