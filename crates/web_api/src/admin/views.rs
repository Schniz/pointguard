use std::path::{Path, PathBuf};

use crate::admin::file_hash_helper;
use handlebars::Handlebars;

pub(crate) fn load(handlebars: &mut Handlebars) {
    file_hash_helper::register(handlebars);
    load_specific(handlebars);
}

#[cfg(debug_assertions)]
fn load_specific(handlebars: &mut Handlebars) {
    let views = PathBuf::from(file!())
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .unwrap()
        .join("views");
    handlebars.set_dev_mode(true);
    handlebars
        .register_templates_directory(".html.hbs", views)
        .expect("load views");
}

#[cfg(not(debug_assertions))]
fn load_specific(handlebars: &mut Handlebars) {
    #[derive(rust_embed::RustEmbed)]
    #[folder = "views"]
    struct Views;

    for key in Views::iter() {
        let value = Views::get(&key).unwrap();
        let key = key.strip_suffix(".html.hbs").unwrap_or(&key);
        handlebars
            .register_template_string(key, &String::from_utf8_lossy(&value.data)[..])
            .expect("register template string");
    }
}
