use std::env;

fn main() {
    if env::var("CARGO_CFG_WINDOWS").is_ok() {
        embed_resource::compile("smplinfo.rc");
    }
}
