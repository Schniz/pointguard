use std::path::{Path, PathBuf};

use crate::admin::file_hash_helper;
use handlebars::{Handlebars, HelperDef};

pub(crate) fn load(handlebars: &mut Handlebars) {
    file_hash_helper::register(handlebars);
    handlebars.register_helper("stringify_json", Box::new(StringifyJsonHelper));
    handlebars.register_helper("inline_if", Box::new(InlineIf));
    load_specific(handlebars);
}

struct InlineIf;
impl HelperDef for InlineIf {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &handlebars::Helper<'reg, 'rc>,
        _r: &'reg Handlebars<'reg>,
        _ctx: &'rc handlebars::Context,
        _rc: &mut handlebars::RenderContext<'reg, 'rc>,
        out: &mut dyn handlebars::Output,
    ) -> handlebars::HelperResult {
        let condition = h
            .param(0)
            .ok_or_else(|| handlebars::RenderError::new("condition is missing"))?;
        let truthy = h
            .param(1)
            .ok_or_else(|| handlebars::RenderError::new("truthy is missing"))?;
        let falsy = h.param(2);

        let condition = match condition.value() {
            serde_json::Value::Null => false,
            serde_json::Value::Bool(b) => *b,
            serde_json::Value::Number(n) => n.as_i64().map(|n| n != 0).unwrap_or(false),
            serde_json::Value::String(s) => !s.is_empty(),
            serde_json::Value::Array(a) => !a.is_empty(),
            serde_json::Value::Object(o) => !o.is_empty(),
        };

        if condition {
            out.write(&truthy.render()).unwrap();
        } else if let Some(falsy) = falsy {
            out.write(&falsy.render()).unwrap();
        }

        Ok(())
    }
}

struct StringifyJsonHelper;
impl HelperDef for StringifyJsonHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &handlebars::Helper<'reg, 'rc>,
        _r: &'reg Handlebars<'reg>,
        _ctx: &'rc handlebars::Context,
        _rc: &mut handlebars::RenderContext<'reg, 'rc>,
        out: &mut dyn handlebars::Output,
    ) -> handlebars::HelperResult {
        let value = h.param(0).unwrap();
        let pretty = h.param(1);
        let value = match pretty.map(|x| x.value()) {
            Some(serde_json::Value::Bool(false)) => serde_json::to_string(value.value()).unwrap(),
            _ => serde_json::to_string_pretty(value.value()).unwrap(),
        };
        out.write(&value).unwrap();
        Ok(())
    }
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
