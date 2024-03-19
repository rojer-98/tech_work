use std::sync::Arc;

use anyhow::{anyhow, Result};
use futures::future::join_all;
use log::info;
use tokio::{
    sync::{Mutex, RwLock},
    time::{sleep, Duration},
};

use crate::{
    detail::Detail,
    mech::{ExecMech, ExecPlace},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PipelineState {
    None,
    Automatic,
    Prepare,
    Work,
}

#[derive(Debug)]
pub struct Pipeline {
    exec_mechs: Vec<ExecMech>,
    exec_places: Vec<ExecPlace>,
    mechs_count: usize,
    details_count: usize,
    state: Arc<Mutex<PipelineState>>,
}

impl Pipeline {
    pub fn new(exec_mechs: Vec<ExecMech>, details: Vec<Detail>) -> Self {
        let mechs_count = exec_mechs.len();
        let details_count = details.len();

        Self {
            details_count,
            mechs_count,
            exec_mechs,
            exec_places: details
                .chunks(mechs_count)
                .map(|chunk| ExecPlace::new(chunk.to_vec()))
                .collect(),
            state: Arc::new(Mutex::new(PipelineState::Automatic)),
        }
    }

    async fn prepare(&mut self) -> Result<()> {
        let _ = join_all(
            self.exec_mechs
                .iter_mut()
                .zip(self.exec_places.iter_mut())
                .filter(|(_, d)| !d.is_prepared)
                .map(|(m, d)| async { m.prepare(d).await }),
        )
        .await;

        info!("-------Prepare cycle is done------");
        Ok(())
    }

    async fn work(&mut self) -> Result<()> {
        let _ = join_all(
            self.exec_mechs
                .iter_mut()
                .zip(self.exec_places.iter_mut())
                .filter(|(_, d)| d.is_prepared && !d.is_processed)
                .map(|(m, d)| async { m.work(d).await }),
        )
        .await;

        info!("-------Work cycle is done------");
        Ok(())
    }

    pub async fn process(&mut self) -> Result<()> {
        match self.get_state().await {
            PipelineState::Automatic => {
                self.prepare().await.and(self.work().await)?;
                self.shift_pipeline(self.details_count);
            }
            PipelineState::Prepare => {
                self.prepare().await?;
                self.set_state(PipelineState::None).await;
            }
            PipelineState::Work => {
                self.work().await?;
                self.set_state(PipelineState::None).await;
            }
            _ => sleep(Duration::from_millis(500)).await,
        }

        Ok(())
    }

    pub fn shift_pipeline(&mut self, detail_count: usize) {
        self.exec_places = (0..detail_count)
            .map(|_| Detail::new())
            .collect::<Vec<_>>()
            .chunks(detail_count / self.mechs_count)
            .map(|chunk| ExecPlace::new(chunk.to_vec()))
            .collect();

        info!("-------Shift pipeline------");
    }

    pub async fn fix_detail(&mut self, mech_id: usize, detail: Arc<RwLock<Detail>>) -> Result<()> {
        self.exec_mechs
            .iter_mut()
            .find(|mech| mech.id == mech_id)
            .ok_or(anyhow!("Mech wasn't found"))?
            .fix(detail)
            .await
    }

    pub fn remove_detail(&mut self, mech_id: usize, detail_id: usize) -> Result<()> {
        let exec_place = self
            .exec_places
            .iter_mut()
            .find(|place| place.contain_detail(detail_id))
            .ok_or(anyhow!("Exec place wasn't found"))?;

        self.exec_mechs
            .iter_mut()
            .find(|mech| mech.id == mech_id)
            .ok_or(anyhow!("Mech wasn't found"))?
            .remove_detail(exec_place, detail_id)
    }

    pub async fn set_state(&mut self, state: PipelineState) {
        *self.state.lock().await = state;
    }

    pub async fn get_state(&self) -> PipelineState {
        self.state.lock().await.clone()
    }

    pub async fn is_automatic(&self) -> bool {
        self.get_state().await == PipelineState::Automatic
    }
}
