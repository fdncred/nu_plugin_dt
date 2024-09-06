mod commands;

use commands::Dt;
use nu_plugin::{Plugin, PluginCommand};

pub use commands::DtAdd;
pub use commands::DtDiff;
pub use commands::DtFormat;
pub use commands::DtNow;
pub use commands::DtPart;
pub use commands::DtTo;
pub use commands::DtUtcNow;

pub struct DtPlugin;

impl Plugin for DtPlugin {
    fn version(&self) -> String {
        // This automatically uses the version of your package from Cargo.toml as the plugin version
        // sent to Nushell
        env!("CARGO_PKG_VERSION").into()
    }

    fn commands(&self) -> Vec<Box<dyn PluginCommand<Plugin = Self>>> {
        vec![
            // Commands should be added here
            Box::new(DtAdd),
            Box::new(DtNow),
            Box::new(DtUtcNow),
            Box::new(Dt),
            Box::new(DtPart),
            Box::new(DtDiff),
            Box::new(DtFormat),
            Box::new(DtTo),
        ]
    }
}
