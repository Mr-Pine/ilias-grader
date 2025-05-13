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
use snafu::{report, whatever, OptionExt, ResultExt, Whatever};

mod cli;
mod download;
mod feedback;

#[report]
fn main() -> Result<(), Whatever> {
    let env = Env::default().filter_or("RUST_LOG", "info");
    env_logger::init_from_env(env);
    let cli_args = Cli::parse();

    let username = cli_args.username;
    let password = cli_args.password.unwrap_or_else(|| {
        let keyring_entry = Entry::new("ilias_grader", &username).unwrap();

        let stored_password = keyring_entry.get_password();

        stored_password.unwrap_or_else(|_| {
            let pw = Password::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Ilias password for user: {}", &username))
                .interact()
                .unwrap();

            keyring_entry.set_password(&pw).unwrap();

            pw
        })
    });

    let ilias_client = IliasClient::new(Url::parse(ILIAS_URL).whatever_context("Unable to parse ilias url")?).whatever_context("Could not create ilias client")?;
    ilias_client.authenticate(&username, &password)?;

    let mut exercise = Exercise::parse(
        ilias_client
            .get_querypath(
                &Exercise::querypath_from_id(&cli_args.id.to_string())
                    .whatever_context("Could not get querypath from id")?,
            )?
            .root_element(),
        &ilias_client,
    )?;

    let grades = exercise
        .get_grades(&ilias_client)
        .whatever_context("No grading options for this exercise")?;
    let assignment_grades = &grades.assignment_grades;
    let grade_page = &assignment_grades[cli_args.assignment];

    let grade_page = match grade_page {
        Reference::Unavailable => whatever!("Could not get grade page"),
        Reference::Resolved(page) => page,
        Reference::Unresolved(_) => &grade_page
            .resolve(&ilias_client)
            .whatever_context("Something went wrong resolving the grade page")?,
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
