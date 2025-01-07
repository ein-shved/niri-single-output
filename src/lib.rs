//!
//! Basic utility types. The [Args] is core type which both handles command line
//! arguments and executes process. The main argument is
//! [command](Args::command). The command specify which operation to perform
//! with niri outputs.
//!
//! Each [command's](Command) emum type implements [Parser] and [Runner] traits
//! to parse arguments from one side and to perform action from another.
//!
#![warn(missing_docs)]

use clap::Subcommand;
pub use clap::{Parser, ValueEnum};
use niri_ipc::{Output, Request, Response};
use std::{
    collections::HashMap,
    env, fs, io,
    path::{Path, PathBuf},
};

/// Top-level arguments structure
#[derive(Parser, Debug)]
#[command(
    author = "Yury Shvedov (github:ein-shved)",
    version = "0.1",
    about = "Niri single-output setup",
    long_about = "Simple utility to control niri outputs withing single-output scheme."
)]
pub struct Args {
    /// The procedure to run
    #[command(subcommand)]
    command: Command,

    /// Optional path to niri socket
    #[arg(short, long, help = "Path to niri socket")]
    path: Option<PathBuf>,

    /// Optional path to state file
    #[arg(short, long, help = "Path to niri socket")]
    state: Option<PathBuf>,
}

/// The list of supported commands
#[derive(Subcommand, Debug, Clone)]
#[command(about, long_about)]
pub enum Command {
    /// Check niri availability.
    ///
    /// Exits with success if niri is available and panics if niri is
    /// unavailable.
    #[command(about, long_about)]
    Test(TestSocket),

    /// Init outputs at startup.
    ///
    /// This tries to read the special state file, which holds the name of last
    /// niri output and attempt to switch it on and other outputs - to switch
    /// off. If file not found - this will switch on first available output and
    /// switch off all others.
    #[command(about, long_about)]
    Init(InitOutputs),

    /// Switch to next output.
    ///
    /// This reads all outputs of niri and switch on output which goes after
    /// first active output and switches off all other outputs.
    #[command(about, long_about)]
    Next(NextOutput),
}

/// The trait for subcommand
pub trait Runner {
    /// The [Args] will create socket for niri and pass it here
    fn run(self, socket: Socket, statefile: PathBuf);
}

impl Args {
    /// Run chosen subcommand
    pub fn run(self) {
        let socket = Socket::connect(self.path);
        let statefile = self.state.unwrap_or(default_state_file());
        match self.command {
            Command::Test(cmd) => cmd.run(socket, statefile),
            Command::Init(cmd) => cmd.run(socket, statefile),
            Command::Next(cmd) => cmd.run(socket, statefile),
        }
    }
}

/// Wrapper on [niri_ipc::socket::Socket] which allows to reuse single object
/// for many `send` calls.
pub struct Socket {
    path: Option<PathBuf>,
}

impl Socket {
    /// See [niri_ipc::socket::Socket::connect()] and
    /// [niri_ipc::socket::Socket::connect_on()]
    pub fn connect(path: Option<PathBuf>) -> Self {
        Self { path }
    }

    /// See [niri_ipc::socket::Socket::send()]
    pub fn send(
        &self,
        request: Request,
    ) -> io::Result<(
        niri_ipc::Reply,
        impl FnMut() -> io::Result<niri_ipc::Event>,
    )> {
        self.get_socket().send(request)
    }

    /// Returns [niri_ipc::socket::Socket] or panics
    pub fn get_socket(&self) -> niri_ipc::socket::Socket {
        if let Some(path) = &self.path {
            niri_ipc::socket::Socket::connect_to(path).unwrap()
        } else {
            niri_ipc::socket::Socket::connect().unwrap()
        }
    }
}

/// Check niri availability.
#[derive(Parser, Debug, Clone)]
pub struct TestSocket {}

impl Runner for TestSocket {
    fn run(self, socket: Socket, _statefile: PathBuf) {
        // Will panic if niri socket is unavailable
        socket.get_socket();
    }
}

fn get_outputs(socket: &Socket) -> HashMap<String, Output> {
    let result = socket.send(Request::Outputs).unwrap().0.unwrap();
    if let Response::Outputs(outputs) = result {
        return outputs;
    } else {
        panic!("Unexpected response type form niri")
    }
}

fn default_state_file() -> PathBuf {
    let state_dir = if let Result::Ok(value) = env::var("XDG_STATE_HOME") {
        value
    } else {
        let home_dir = env::var("HOME").unwrap();
        home_dir + "/.local/state"
    };

    (state_dir + "/niri/last-output").into()
}

fn get_last_output(statefile: &Path) -> Option<String> {
    prepare_statedirs(statefile);
    fs::read_to_string(statefile).ok()
}

fn set_last_output(statefile: &Path, output: &str) {
    prepare_statedirs(statefile);
    fs::write(statefile, output).unwrap();
}

fn prepare_statedirs(statefile: &Path) {
    fs::create_dir_all(statefile.parent().unwrap()).unwrap();
}

fn set_output(
    socket: &Socket,
    output: &str,
    statefile: &Path,
    outputs: &HashMap<String, Output>,
) {
    for (out, &_) in outputs.iter() {
        let action = if out == output {
            niri_ipc::OutputAction::On
        } else {
            niri_ipc::OutputAction::Off
        };
        println!("For output {} call {:?}", out, action);
        socket
            .send(Request::Output {
                output: out.into(),
                action,
            })
            .unwrap()
            .0
            .unwrap();
    }
    set_last_output(statefile, output);
}

/// Init outputs at startup.
#[derive(Parser, Debug, Clone)]
pub struct InitOutputs {}

impl Runner for InitOutputs {
    fn run(self, socket: Socket, statefile: PathBuf) {
        let last = get_last_output(&statefile);
        let outputs = get_outputs(&socket);

        let last = last.unwrap_or({
            let mut iter = outputs.iter();
            loop {
                if let Some((out, state)) = iter.next() {
                    if state.current_mode.is_some() {
                        break out.into();
                    };
                } else {
                    break outputs.iter().next().unwrap().0.into();
                }
            }
        });

        set_output(&socket, &last, &statefile, &outputs)
    }
}

/// Switch to next output.
#[derive(Parser, Debug, Clone)]
pub struct NextOutput {}
impl Runner for NextOutput {
    fn run(self, socket: Socket, statefile: PathBuf) {
        let outputs = get_outputs(&socket);

        let mut iter = outputs.iter();
        loop {
            if let Some((&_, state)) = iter.next() {
                if state.current_mode.is_some() {
                    break;
                };
            } else {
                break;
            }
        }

        let next = iter.next().unwrap_or(outputs.iter().next().unwrap()).0;

        set_output(&socket, &next, &statefile, &outputs)
    }
}
