use clap::Parser;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

use crate::{
    app::App,
    args::{Args, AuthCommands, Command},
    config::Config,
};

mod access;
mod app;
mod args;
mod auth;
mod config;
mod get;
mod ls;
mod rm;
mod upload;

// TODO: Beautify all the output, switch from tracing to like a custom logging that
// uses colors and fancy formatting to look nice for the user while it does stuff
// like loaders for http requests and stuff

fn main() {
    let app = App {
        args: Args::parse(),
        config: Config::get(),
    };

    setup_tracing(&app);

    match app.args.command.clone() {
        Command::Upload {
            local,
            remote,
            access,
            id,
        } => upload::upload(app, local, remote, access, id),
        Command::Get {
            file,
            local,
            metadata,
            link,
        } => get::get(app, file, local, metadata, link),
        Command::Rm { file } => rm::rm(&app, file),
        Command::Ls { directory } => ls::ls(app, directory),
        Command::Access { file, access } => access::access(app, file, access),
        Command::Auth(auth_command) => match auth_command {
            AuthCommands::Add {
                name,
                url,
                token,
                web_url,
                autoset,
            } => auth::add(app, name, url, token, web_url, autoset),
            AuthCommands::Ls => auth::ls(app),
            AuthCommands::Rm { name } => auth::rm(app, name),
            AuthCommands::Set { name } => auth::set(app, name),
        },
        _ => todo!("This sub-command is not implemented yet"),
    }
}

fn setup_tracing(app: &App) {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(if app.args.debug {
            Level::DEBUG
        } else {
            Level::INFO
        })
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    tracing::debug!("Initialized tracing logging");
}
