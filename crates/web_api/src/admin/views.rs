use std::path::{Path, PathBuf};

use handlebars::Handlebars;

#[cfg(debug_assertions)]
pub(crate) fn load(handlebars: &mut Handlebars) {
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
pub(crate) fn load(handlebars: &mut Handlebars) {
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
