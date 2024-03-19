mod config;
mod detail;
mod mech;
mod operator;
mod pipeline;
mod utils;

use std::{path::PathBuf, str::FromStr};

use anyhow::Result;
use serde::Deserialize;
use tokio::{select, sync::mpsc::unbounded_channel};

use config::Config;
use detail::{Detail, DetailInfo};
use mech::ExecMech;
use operator::Operator;
use pipeline::Pipeline;

#[derive(Debug, Clone)]
pub enum EventType {
    Accident(DetailInfo),
    Removed(usize),
    Fixed,
    Ui,
}

#[derive(Debug, Deserialize)]
pub struct InitConfig {
    pub mech_count: usize,
    pub details_count: usize,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 16)]
async fn main() -> Result<()> {
    env_logger::init();

    let config =
        InitConfig::load(PathBuf::from_str(env!("CARGO_MANIFEST_DIR"))?.join("config.yml"))?;

    let (tx_event, mut rx_event) = unbounded_channel();
    let operator = Operator::default();
    let mut pipeline = Pipeline::new(
        (0..config.mech_count)
            .into_iter()
            .map(|_| ExecMech::new(tx_event.clone()))
            .collect::<Vec<_>>(),
        (0..config.details_count)
            .map(|_| Detail::new())
            .collect::<Vec<_>>(),
    );

    // Event loop
    loop {
        select! {
            Some(event) = rx_event.recv() => {
                if let Err(_) = match event {
                    EventType::Accident(d) => operator.show_ui_accident(&mut pipeline, d).await,
                    EventType::Removed(id) =>  operator.show_ui_removed(&mut pipeline, id).await,
                    EventType::Fixed =>  operator.show_ui_fixed(&mut pipeline).await,
                    EventType::Ui => operator.show_ui(&mut pipeline).await,
                } {
                    return Ok(());
                }
            }
            _ =  pipeline.process() => {
                if !pipeline.is_automatic().await {
                    tx_event.send(EventType::Ui)?
                }
            }
        }
    }
}
