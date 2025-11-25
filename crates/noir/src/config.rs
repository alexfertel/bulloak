//! configuration specific to bulloak's noir backend
use std::path::PathBuf;
/// `bulloak-noir`'s configuration.
///
/// Note that configuration coming from the command line is aggregated to this
/// struct only if it makes sense. For example, the `--vm-skip` flag doesn't make
/// sense in the context of noir tests, as there is not a `vm.skip` equivalent
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// List of files being processed.
    pub files: Vec<PathBuf>,
    /// Skip generation of helper functions for conditions.
    pub skip_helpers: bool,
    /// Format action descriptions (capitalize, etc).
    pub format_descriptions: bool,
}
