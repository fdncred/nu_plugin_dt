mod commands;

use commands::Dt;
use nu_plugin::{Plugin, PluginCommand};

pub use commands::Add;
pub use commands::Now;
pub use commands::UtcNow;

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
            Box::new(Add),
            Box::new(Now),
            Box::new(UtcNow),
            Box::new(Dt),
        ]
    }
}
