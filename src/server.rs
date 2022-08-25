use time::OffsetDateTime;

use embedded_svc::httpd::registry::*;
use esp_idf_svc::httpd::{Server, ServerRegistry};

pub fn start() -> anyhow::Result<Server> {
    httpd()
}

fn httpd() -> anyhow::Result<Server> {
    let server = ServerRegistry::new()
        .at("/")
        .get(|_| Ok(format!("{:?}", OffsetDateTime::now_utc()).into()))?;

    server.start(&Default::default())
}
