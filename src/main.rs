use nu_plugin::{serve_plugin, MsgPackSerializer};
use nu_plugin_dt::DtPlugin;

fn main() {
    serve_plugin(&DtPlugin, MsgPackSerializer);
}
