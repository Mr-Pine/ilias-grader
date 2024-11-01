use anyhow::{anyhow, Context, Result};
use clap::Parser;
use cli::Cli;
use dialoguer::{theme::ColorfulTheme, Password};
use download::download_submissions;
use env_logger::Env;
use feedback::upload_feedback;
use ilias::{
    client::IliasClient, exercise::Exercise, reference::Reference, IliasElement, ILIAS_URL,
};
use keyring::Entry;
use reqwest::Url;

mod cli;
mod download;
mod feedback;

fn main() -> Result<()> {
    let env = Env::default().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);
    let cli_args = Cli::parse();

    let username = cli_args.username;
    let password = match cli_args.password {
        Some(pw) => pw,
        None => {
            let keyring_entry = Entry::new("ilias_grader", &username).unwrap();

            let stored_password = keyring_entry.get_password();

            match stored_password {
                Ok(pw) => pw,
                Err(_) => {
                    let pw = Password::with_theme(&ColorfulTheme::default())
                        .with_prompt(format!("Ilias password for user: {}", &username))
                        .interact()
                        .unwrap();

                    keyring_entry.set_password(&pw).unwrap();

                    pw
                }
            }
        }
    };

    let ilias_client = IliasClient::new(Url::parse(ILIAS_URL)?)?;
    ilias_client.authenticate(&username, &password)?;

    let mut exercise = Exercise::parse(
        ilias_client
            .get_querypath(
                &Exercise::querypath_from_id(&cli_args.id.to_string())
                    .context("Could not get querypath from id")?,
            )?
            .root_element(),
        &ilias_client,
    )?;

    let grades = exercise
        .get_grades(&ilias_client)
        .context("No grading options for this exercise")?;
    let assignment_grades = &grades.assignment_grades;
    let grade_page = &assignment_grades[cli_args.assignment];

    let grade_page = match grade_page {
        Reference::Unavailable => return Err(anyhow!("Could not get grade page")),
        Reference::Resolved(page) => page,
        Reference::Unresolved(_) => &grade_page
            .resolve(&ilias_client)
            .context("Something went wrong resolving the grade page")?,
    };

    match cli_args.command {
        cli::Commands::Download {
            to,
            extract,
            flatten,
        } => download_submissions(grade_page, &to, extract, flatten, &ilias_client),
        cli::Commands::Feedback {
            no_confim,
            feedback_dir,
            filter_expr,
            suffix,
        } => upload_feedback(
            grade_page,
            no_confim,
            &feedback_dir,
            filter_expr.as_ref(),
            suffix.as_ref(),
            &ilias_client,
        ),
    }?;

    Ok(())
}
