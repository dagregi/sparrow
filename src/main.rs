use std::{cell::RefCell, rc::Rc};

use clap::Parser;
use cli::Cli;
use color_eyre::Result;
use transmission_rpc::{types::BasicAuth, TransClient};

use crate::app::App;

mod action;
mod app;
mod cli;
mod colors;
mod components;
mod config;
mod data;
mod errors;
mod logging;
mod tui;
mod utils;

#[tokio::main]
async fn main() -> Result<()> {
    crate::errors::init()?;
    crate::logging::init()?;

    let args = Cli::parse();
    let url = args.url;
    let client;
    if let (Some(user), Some(password)) = (args.username, args.password) {
        client = Rc::new(RefCell::new(TransClient::with_auth(
            url.parse()?,
            BasicAuth { user, password },
        )));
    } else {
        client = Rc::new(RefCell::new(TransClient::new(url.parse()?)));
    }
    let mut app = App::new(args.tick_rate, args.frame_rate, &client)?;
    app.run().await?;
    Ok(())
}
