use std::{collections::BTreeMap, fs, sync::Arc};

use yiyin_application::ResourceRepository;
use yiyin_domain::ResourceId;

const NO_SNIFF: &str = "nosniff";

pub struct ResourceProtocol {
    resources: Arc<dyn ResourceRepository>,
}

impl ResourceProtocol {
    #[must_use]
    pub fn new(resources: Arc<dyn ResourceRepository>) -> Self {
        Self { resources }
    }

    #[must_use]
    pub fn respond(&self, raw_uri: &str) -> ProtocolResponse {
        self.try_respond(raw_uri).unwrap_or_else(|()| forbidden())
    }

    fn try_respond(&self, raw_uri: &str) -> Result<ProtocolResponse, ()> {
        let uri = raw_uri.parse::<tauri::http::Uri>().map_err(|_| ())?;
        if uri.scheme_str() != Some("yiyin")
            || uri.authority().map(tauri::http::uri::Authority::as_str) != Some("resource")
            || uri.query().is_some()
        {
            return Err(());
        }
        let segment = uri.path().strip_prefix('/').ok_or(())?;
        if segment.is_empty()
            || segment.contains('/')
            || segment.contains('\\')
            || segment.contains('%')
            || matches!(segment, "." | "..")
        {
            return Err(());
        }
        let id = ResourceId::try_from(segment).map_err(|_| ())?;
        let original = self
            .resources
            .snapshot()
            .into_iter()
            .find(|record| record.id() == &id)
            .ok_or(())?;
        if fs::symlink_metadata(original.source())
            .map_err(|_| ())?
            .file_type()
            .is_symlink()
        {
            return Err(());
        }
        let record = self.resources.resolve(&id).map_err(|_| ())?;
        let content_type = match record.mime_type() {
            "image/jpeg" | "image/png" | "image/webp" | "font/ttf" | "font/otf" => {
                record.mime_type()
            }
            _ => return Err(()),
        };
        let body = fs::read(record.source()).map_err(|_| ())?;
        let mut headers = BTreeMap::new();
        headers.insert("content-type".to_owned(), content_type.to_owned());
        headers.insert("x-content-type-options".to_owned(), NO_SNIFF.to_owned());
        headers.insert("cache-control".to_owned(), "no-store".to_owned());
        Ok(ProtocolResponse {
            status: 200,
            body,
            headers,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtocolResponse {
    status: u16,
    body: Vec<u8>,
    headers: BTreeMap<String, String>,
}

impl ProtocolResponse {
    #[must_use]
    pub const fn status(&self) -> u16 {
        self.status
    }

    #[must_use]
    pub fn content_type(&self) -> Option<&str> {
        self.header("content-type")
    }

    #[must_use]
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .get(&name.to_ascii_lowercase())
            .map(String::as_str)
    }

    #[must_use]
    pub fn into_http(self) -> tauri::http::Response<Vec<u8>> {
        let mut builder = tauri::http::Response::builder().status(self.status);
        for (name, value) in &self.headers {
            builder = builder.header(name, value);
        }
        builder
            .body(self.body)
            .unwrap_or_else(|_| tauri::http::Response::new(Vec::new()))
    }
}

fn forbidden() -> ProtocolResponse {
    let mut headers = BTreeMap::new();
    headers.insert("x-content-type-options".to_owned(), NO_SNIFF.to_owned());
    headers.insert("cache-control".to_owned(), "no-store".to_owned());
    ProtocolResponse {
        status: 403,
        body: Vec::new(),
        headers,
    }
}
