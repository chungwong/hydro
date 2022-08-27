use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use embedded_svc::httpd::registry::*;
use esp_idf_svc::httpd::{Server, ServerRegistry};

pub fn start() -> anyhow::Result<Server> {
    httpd()
}

fn httpd() -> anyhow::Result<Server> {
    let server = ServerRegistry::new()
        .at("/")
        .get(|_| Ok(format!("{:?}", OffsetDateTime::now_utc().format(&Rfc3339)).into()))?;

    server.start(&Default::default())
}
