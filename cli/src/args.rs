use std::{path::PathBuf, str::FromStr};

use clap::{Parser, Subcommand};
use sf_core::FileAccess;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
    #[arg(
        long,
        help = "Specify the host to operate the command on\nIf not specified, will use the default host (if set)\nFormat in token@url"
    )]
    pub host: Option<String>,
    #[arg(
        long,
        global = true,
        default_value_t = false,
        help = "Enables debug logging"
    )]
    pub debug: bool,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
    #[clap(
        about = "Uploads a file to the server",
        long_about = "Uploads a file to the server, returning it's metadata if successful.\nDirectory is optional if not uploaded via a link id (would default to root)\nIf no --access flag is provided, it defaults to Private"
    )]
    Upload {
        #[arg(help = "The local path to the file you want to upload")]
        local: PathBuf,
        #[clap(
            default_value_t,
            help = "The remote directory, prefixed with the root on the server.\nCan be left empty to upload in root directory"
        )]
        remote: ArgPath,
        #[arg(
            short,
            long,
            help = "Provides the initial access level the file with be set at"
        )]
        access: Option<FileAccess>,
        #[arg(
            long,
            help = "If uploading a file via a one-time link,\nuse this flag to provide the id of said link to use it."
        )]
        id: Option<String>,
    },
    // Dont know if 'get' is a good name for this.
    // otherwise "Download", but its too long imo
    Get {
        file: FileIdentifier,
        local: Option<PathBuf>,
        #[arg(short, long)]
        metadata: bool,
        #[arg(short, long)]
        link: bool,
    },
    Rm {
        file: FileIdentifier,
    },
    Mv {
        file: FileIdentifier,
        new_path: PathBuf,
    },
    Ls {
        #[clap(default_value_t)]
        directory: ArgPath,
    },
    Mkdir {
        directory: ArgPath,
    },
    Access {
        file: FileIdentifier,
        access: FileAccess,
    },

    #[clap(subcommand)]
    Link(LinkCommands),

    Speedtest,

    #[clap(
        subcommand,
        about = "A set of authentication commands to configure which host server you're connected to"
    )]
    Auth(AuthCommands),

    Sync,
    Log,
    Config,
    Stats,
    Cleanup,
}

#[derive(Debug, Subcommand, Clone)]
pub enum LinkCommands {
    Create,
    Rm { link_id: String },
    Ls,
}

#[derive(Debug, Subcommand, Clone)]
pub enum AuthCommands {
    #[clap(about = "Adds a new server host to upload via.")]
    Add {
        #[arg(
            help = "A name for this host, only used for your convenience and to identify hosts in configuration."
        )]
        name: String,
        #[arg(help = "The url pointing to the host.")]
        url: String,
        #[arg(
            short,
            long,
            help = "The token used for the main host login.\nIf not specified, most commands won't work\nand only token-free commands like one-time link uploads and public file gets will work"
        )]
        token: Option<String>,
        #[arg(
            short,
            long,
            help = "The URL for the web client interface for this specific host.\nMust be specified to use one-time link and fancy preview related features."
        )]
        web_url: Option<String>,
        #[arg(
            short,
            long,
            default_value_t = true,
            help = "If it should automatically set this new host as the default one\nCan be changed later via 'auth set'\nDefaults to true"
        )]
        autoset: bool,
    },
    #[clap(about = "Lists all saved hosts.")]
    Ls,
    #[clap(about = "Removes the specified host from the configuration.")]
    Rm { name: String },
    #[clap(
        about = "Specifies the default host to use in commands if no other host is specified via --host"
    )]
    Set { name: String },
}

#[derive(Debug, Clone)]
#[allow(unused)]
pub enum FileIdentifier {
    Id(String),
    Path(PathBuf),
}

impl FileIdentifier {
    const ID_LEN: usize = 10;
}

impl FromStr for FileIdentifier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // An id is only ID_LEN long, contains no . nor / \
        if s.len() == FileIdentifier::ID_LEN
            && !(s.contains(".") || s.contains("/") || s.contains("\\"))
        {
            Ok(FileIdentifier::Id(s.to_string()))
        } else {
            Ok(FileIdentifier::Path(PathBuf::from(s)))
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArgPath(PathBuf);

#[allow(unused)]
impl ArgPath {
    pub fn path(self) -> PathBuf {
        self.0
    }
}

impl FromStr for ArgPath {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ArgPath(PathBuf::from(s)))
    }
}

impl Default for ArgPath {
    fn default() -> Self {
        ArgPath(PathBuf::from(""))
    }
}

impl ToString for ArgPath {
    fn to_string(&self) -> String {
        self.0.to_string_lossy().to_string()
    }
}
