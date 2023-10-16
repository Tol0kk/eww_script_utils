#![allow(dead_code)]

use std::error::Error;

use clap::Subcommand;
mod utils;

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    Hyprland(HyprlandCommand),
    #[command(subcommand)]
    Network(NetworkCommand),
    // TODO
    Volume,
    // TODO
    Bluetooth,
    // TODO
    MPRIS,
}

#[derive(Subcommand)]
enum HyprlandCommand {
    Workspace,
    ActiveWindow,
    KeyboardLanguage,
}

#[derive(Subcommand)]
enum NetworkCommand {
    Info,
    List,
    Connect,
    Test,
}

mod hyprland;
mod network;

#[tokio::main]
async fn main() ->  Result<(), Box<dyn Error>> {
    let cli = clap::Command::new("script");
    let cli = Commands::augment_subcommands(cli);

    match cli.get_matches().subcommand() {
        Some(("hyprland", subcommand)) => match subcommand.subcommand() {
            Some(("workspace",_)) => hyprland::workspaces_listener()?,
            Some(("active-window",_)) => hyprland::active_window_listener()?,
            Some(("keyboard-language",_)) => hyprland::keyboard_language_listener()?,
            _ => ()
        },
        Some(("network", subcommand)) => match subcommand.subcommand() {
            Some(("info",_)) => network::info().await?,
            Some(("test",_)) => network::test().await?,
            _ => ()
        },
        _ => {}
    }

    Ok(())
}
