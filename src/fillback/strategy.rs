use crate::Result;
use async_trait::async_trait;
use std::path::Path;

/// Common interface for all import strategies
///
/// Each strategy (OpenSpec, Speckit, Code) implements this trait to provide
/// a consistent way to execute imports and detect if they can handle a given source.
#[async_trait]
pub trait ImportStrategy: Send + Sync {
    /// Execute the import strategy
    ///
    /// # Arguments
    /// * `source` - Path to the source directory or file to import from
    /// * `change_id` - The change ID to create/populate in agentd/changes/
    ///
    /// # Errors
    /// Returns an error if the import fails for any reason (parsing, file I/O, etc.)
    async fn execute(&self, source: &Path, change_id: &str) -> Result<()>;

    /// Check if this strategy can handle the given source
    ///
    /// Used for auto-detection when strategy is set to "auto".
    /// Each strategy implements its own detection logic.
    ///
    /// # Arguments
    /// * `source` - Path to check
    ///
    /// # Returns
    /// `true` if this strategy can handle the source, `false` otherwise
    fn can_handle(&self, source: &Path) -> bool;

    /// Get the name of this strategy for display purposes
    fn name(&self) -> &'static str;
}
