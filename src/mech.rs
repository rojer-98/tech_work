use std::{collections::HashMap, sync::Arc};

use anyhow::{anyhow, Result};
use futures::{stream::iter, StreamExt};
use log::{error, info};
use tokio::sync::{mpsc::UnboundedSender, RwLock};

use crate::{
    detail::{Detail, DetailInfo, DetailState},
    utils::{generate_chance_of_accident, generate_unique_id, imitation_of_work},
    EventType,
};

#[derive(Debug)]
pub struct ExecPlace {
    pub id: usize,
    pub is_prepared: bool,
    pub is_processed: bool,

    details: HashMap<usize, Arc<RwLock<Detail>>>,
}

impl ExecPlace {
    pub fn new<T: IntoIterator<Item = Detail>>(details: T) -> Self {
        Self {
            id: generate_unique_id(),
            details: details
                .into_iter()
                .map(|d| (d.id, Arc::new(RwLock::new(d))))
                .collect(),
            is_processed: false,
            is_prepared: false,
        }
    }

    pub fn contain_detail(&self, detail_id: usize) -> bool {
        self.details.contains_key(&detail_id)
    }
}

#[derive(Debug)]
pub struct ExecMech {
    pub id: usize,
    tx_event: UnboundedSender<EventType>,
}

impl ExecMech {
    pub fn new(tx_event: UnboundedSender<EventType>) -> Self {
        Self {
            id: generate_unique_id(),
            tx_event,
        }
    }

    // Remove the broken detail
    pub fn remove_detail(&self, exec_place: &mut ExecPlace, detail_id: usize) -> Result<()> {
        info!("Remove detail: `{detail_id}`");

        if exec_place.details.remove(&detail_id).is_none() {
            return Err(anyhow!("Detail is not exist"));
        } else {
            self.tx_event.send(EventType::Removed(detail_id))?;
        }

        Ok(())
    }

    // Fix the broken detail
    pub async fn fix(&mut self, detail: Arc<RwLock<Detail>>) -> Result<()> {
        match process_fix(detail.clone()).await {
            Ok(_) => {
                self.tx_event.send(EventType::Fixed)?;
            }
            Err(e) => {
                error!("{e}");
                self.tx_event.send(EventType::Accident(DetailInfo {
                    mech_id: self.id,
                    detail: detail.clone(),
                }))?;
            }
        }

        Ok(())
    }

    // Preparing the details for future work
    pub async fn prepare(&mut self, exec_place: &mut ExecPlace) {
        exec_place.is_prepared = iter(&exec_place.details)
            .fold(Ok(()), |acc, (_, detail)| async {
                let res = process_prepare(detail.clone()).await;
                if let Err(e) = res.as_ref() {
                    error!("{e}");

                    // Critical error if the basic `event loop` does not work
                    // then you can crash
                    self.tx_event
                        .send(EventType::Accident(DetailInfo {
                            mech_id: self.id,
                            detail: detail.clone(),
                        }))
                        .unwrap();
                }

                acc.and(res)
            })
            .await
            .is_ok();
    }

    // Final work on the details
    pub async fn work(&mut self, exec_place: &mut ExecPlace) {
        exec_place.is_processed = iter(&exec_place.details)
            .fold(Ok(()), |acc, (_, detail)| async {
                let res = process_work(detail.clone()).await;
                if let Err(e) = res.as_ref() {
                    error!("{e}");

                    // Critical error if the basic `event loop` does not work
                    // then you can crash
                    self.tx_event
                        .send(EventType::Accident(DetailInfo {
                            mech_id: self.id,
                            detail: detail.clone(),
                        }))
                        .unwrap();
                }

                acc.and(res)
            })
            .await
            .is_ok()
    }
}

async fn process_fix(detail: Arc<RwLock<Detail>>) -> Result<()> {
    let mut detail_guard = detail.write().await;
    let detail_id = detail_guard.id;
    info!("Fix detail: `{detail_id}`");

    if let DetailState::Broken = detail_guard.state {
        if process().await {
            info!("Fixed detail: `{detail_id}`");
            detail_guard.state = DetailState::Prepared;

            return Ok(());
        }

        Err(anyhow!("Detail wasn't fixed"))
    } else {
        Err(anyhow!("Detail is not broken"))
    }
}

async fn process_work(detail: Arc<RwLock<Detail>>) -> Result<()> {
    let mut detail_guard = detail.write().await;
    let detail_id = detail_guard.id;

    if let DetailState::Prepared = detail_guard.state {
        if process().await {
            info!("Process detail: `{detail_id}`");
            detail_guard.state = DetailState::Procesed;

            return Ok(());
        }

        detail_guard.state = DetailState::Broken;
        return Err(anyhow!("Detail was broken: `{detail_id}`"));
    }

    Ok(())
}

async fn process_prepare(detail: Arc<RwLock<Detail>>) -> Result<()> {
    let mut detail_guard = detail.write().await;
    let detail_id = detail_guard.id;

    if let DetailState::Start = detail_guard.state {
        if process().await {
            info!("Prepare detail: `{detail_id}`");
            detail_guard.state = DetailState::Prepared;

            return Ok(());
        }

        detail_guard.state = DetailState::Broken;
        return Err(anyhow!("Detail was broken: `{detail_id}`"));
    }

    Ok(())
}

#[inline(always)]
// Work imitiation
async fn process() -> bool {
    imitation_of_work().await;

    generate_chance_of_accident().is_ok_and(|x| x)
}
