use handlebars::{Context, Handlebars, Helper, HelperDef, Output, RenderContext};
use rust_embed::RustEmbed;
use std::path::PathBuf;

trait FileHash {
    fn file_hash(&self, template: &str, path: &str) -> Option<String>;
}

#[derive(Debug)]
struct Hashable<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<R: RustEmbed> FileHash for Hashable<R> {
    fn file_hash(&self, _template: &str, path: &str) -> Option<String> {
        R::get(path).map(|x| hex::encode(x.metadata.sha256_hash()))
    }
}

impl FileHash for PathBuf {
    fn file_hash(&self, template: &str, path: &str) -> Option<String> {
        let path = self.join(template).parent().unwrap().join(path);
        std::fs::read(path).ok().map(|x| sha256(&x))
    }
}

#[derive(Debug)]
struct FileHashHelper<H> {
    inner: H,
}

pub fn register(handlebars: &mut Handlebars) {
    #[cfg(debug_assertions)]
    {
        use std::path::Path;
        let public_dir = Path::new(file!())
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .unwrap()
            .join("public");
        handlebars.register_helper("file_hash", Box::new(FileHashHelper { inner: public_dir }));
    }
    #[cfg(not(debug_assertions))]
    {
        handlebars.register_helper(
            "file_hash",
            Box::new(FileHashHelper {
                inner: Hashable {
                    _marker: std::marker::PhantomData::<super::public::release::Public>,
                },
            }),
        );
    }
}

impl<H: FileHash> HelperDef for FileHashHelper<H> {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _r: &'reg Handlebars<'reg>,
        _ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> handlebars::HelperResult {
        let value = h
            .param(0)
            .map(|x| x.value())
            .map(|x| match x {
                serde_json::Value::String(s) => s,
                _ => panic!("first value is not a string"),
            })
            .expect("first value");
        let template_name = rc
            .get_current_template_name()
            .map(|x| x.to_string())
            .expect("template name");
        let hash = self.inner.file_hash(&template_name, &value).expect("hash");
        out.write(&hash).expect("write");
        Ok(())
    }
}

fn sha256(s: impl AsRef<[u8]>) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(s);
    let bytes = hex::encode(hasher.finalize());
    bytes
}
