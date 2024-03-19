use anyhow::{anyhow, Result};
use dialoguer::{theme::ColorfulTheme, Select};

use crate::{
    detail::DetailInfo,
    pipeline::{Pipeline, PipelineState},
};

#[derive(Debug, Clone, Copy)]
pub enum OperatorCommand {
    Fix,
    RemoveDetail,
    End,
    Prepare,
    Process,
    ShiftPipeline,
    EnableAutomatic,
    DisableAutomatic,
}

impl ToString for OperatorCommand {
    fn to_string(&self) -> String {
        match self {
            OperatorCommand::Fix => "Fix detail",
            OperatorCommand::RemoveDetail => "Remove detail from pipeline",
            OperatorCommand::Prepare => "Prepare details",
            OperatorCommand::Process => "Process detail",
            OperatorCommand::End => "End program",
            OperatorCommand::ShiftPipeline => "Shift pipeline (generate new details on pipeline)",
            OperatorCommand::EnableAutomatic => "Enable automatic pipeline process",
            OperatorCommand::DisableAutomatic => "Disable automatic pipeline process",
        }
        .to_string()
    }
}

#[derive(Debug, Default)]
pub struct Operator;

impl Operator {
    // General operator menu
    pub async fn show_ui(&self, pipeline: &mut Pipeline) -> Result<()> {
        let prompt = format!("Operator menu. You can: ");
        let selections = &[
            OperatorCommand::ShiftPipeline,
            OperatorCommand::Prepare,
            OperatorCommand::Process,
            OperatorCommand::EnableAutomatic,
            OperatorCommand::DisableAutomatic,
            OperatorCommand::End,
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(0)
            .items(&selections[..])
            .interact()?;

        match selections[selection] {
            OperatorCommand::End => return Err(anyhow!("End program")),
            OperatorCommand::ShiftPipeline => pipeline.shift_pipeline(12),
            OperatorCommand::EnableAutomatic => pipeline.set_state(PipelineState::Automatic).await,
            OperatorCommand::DisableAutomatic => pipeline.set_state(PipelineState::None).await,
            OperatorCommand::Prepare => pipeline.set_state(PipelineState::Prepare).await,
            OperatorCommand::Process => pipeline.set_state(PipelineState::Work).await,
            _ => {}
        }

        Ok(())
    }

    pub async fn show_ui_removed(&self, pipeline: &mut Pipeline, detail_id: usize) -> Result<()> {
        let prompt = format!("Detail `{detail_id}` removed. You can: ");
        let selections = &[
            OperatorCommand::ShiftPipeline,
            OperatorCommand::Prepare,
            OperatorCommand::Process,
            OperatorCommand::EnableAutomatic,
            OperatorCommand::DisableAutomatic,
            OperatorCommand::End,
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(0)
            .items(&selections[..])
            .interact()?;

        match selections[selection] {
            OperatorCommand::End => return Err(anyhow!("End program")),
            OperatorCommand::ShiftPipeline => pipeline.shift_pipeline(12),
            OperatorCommand::EnableAutomatic => pipeline.set_state(PipelineState::Automatic).await,
            OperatorCommand::DisableAutomatic => pipeline.set_state(PipelineState::None).await,
            OperatorCommand::Prepare => pipeline.set_state(PipelineState::Prepare).await,
            OperatorCommand::Process => pipeline.set_state(PipelineState::Work).await,
            _ => {}
        }

        Ok(())
    }

    pub async fn show_ui_accident(&self, pipeline: &mut Pipeline, d: DetailInfo) -> Result<()> {
        let detail_id = {
            let detail_guard = d.detail.read().await;
            detail_guard.id
        };
        let mech_id = d.mech_id;

        let prompt = format!(
            "Accident was happened! Automatic mode disabled. \n Mech `{mech_id}` Detail `{detail_id}` was broken. You can: "
        );
        let selections = &[
            OperatorCommand::Fix,
            OperatorCommand::RemoveDetail,
            OperatorCommand::End,
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(0)
            .items(&selections[..])
            .interact()?;

        match selections[selection] {
            OperatorCommand::Fix => pipeline.fix_detail(d.mech_id, d.detail).await?,
            OperatorCommand::RemoveDetail => pipeline.remove_detail(d.mech_id, detail_id)?,
            OperatorCommand::End => return Err(anyhow!("End program")),
            _ => {}
        }

        Ok(())
    }

    pub async fn show_ui_fixed(&self, pipeline: &mut Pipeline) -> Result<()> {
        let prompt = format!("Detail fixed! You can: ");
        let selections = &[
            OperatorCommand::Process,
            OperatorCommand::Prepare,
            OperatorCommand::EnableAutomatic,
            OperatorCommand::End,
        ];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(0)
            .items(&selections[..])
            .interact()?;

        match selections[selection] {
            OperatorCommand::End => return Err(anyhow!("End program")),
            OperatorCommand::EnableAutomatic => pipeline.set_state(PipelineState::Automatic).await,
            OperatorCommand::Process => pipeline.set_state(PipelineState::Work).await,
            OperatorCommand::Prepare => pipeline.set_state(PipelineState::Prepare).await,
            _ => {}
        }

        Ok(())
    }
}
