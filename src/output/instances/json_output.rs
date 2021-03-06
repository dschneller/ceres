use serde_json;

use output::instances::*;
use provider::{InstanceDescriptor, StateChange};
use utils::command::CommandResult;

pub struct JsonOutputInstances;

impl OutputInstances for JsonOutputInstances {
    fn output<T: Write>(&self, writer: &mut T, instances: &[InstanceDescriptor]) -> Result<()> {
        serde_json::to_writer_pretty(writer, instances).chain_err(|| ErrorKind::OutputFailed)
    }
}

pub struct JsonOutputStateChanges;

impl OutputStateChanges for JsonOutputStateChanges {
    fn output<T: Write>(&self, writer: &mut T, state_changes: &[StateChange]) -> Result<()> {
        serde_json::to_writer_pretty(writer, state_changes).chain_err(|| ErrorKind::OutputFailed)
    }
}

pub struct JsonOutputCommandResults;

impl OutputCommandResults for JsonOutputCommandResults {
    fn output<T: Write>(&self, writer: &mut T, results: &[CommandResult]) -> Result<()> {
        serde_json::to_writer_pretty(writer, results).chain_err(|| ErrorKind::OutputFailed)
    }
}
