use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use embedded_svc::httpd::registry::Registry;
use embedded_svc::httpd::Response;

use esp_idf_svc::httpd::{Server, ServerRegistry};

use crate::storage::{Storage, StorageBase};

pub(crate) fn start(master_storage: &Storage) -> anyhow::Result<Server> {
    let storage = master_storage.0.clone();
    let storage2 = master_storage.0.clone();

    let server = ServerRegistry::new()
        .at("/")
        .get(move |_| {
            let ssid = storage.get("WIFI_SSID").unwrap_or_default();
            dbg!(&ssid);
            let pass = storage.get("WIFI_PASS").unwrap_or_default();
            dbg!(&pass);

            resp(format!(
                r#"
                <div>
                  <p>Current Date Time: {:?}</p>
                  <form method = "post" action = "/save" enctype="application/x-www-form-urlencoded">
                    <label for="ssid">WiFi SSID:</label><br>
                    <input type="text" id="ssid" name="ssid" value="{:?}"><br>

                    <label for="pass">WiFi PASS:</label><br>
                    <input type="text" id="pass" name="pass" value="{:?}"><br><br>

                    <input type="submit" value="Submit">
                  </form>
                </div>
                "#,
                OffsetDateTime::now_utc().format(&Rfc3339),
                ssid,
                pass
            ))
        })?
        .at("/save")
        .post(move |mut req| {
            // let body = req.as_bytes()?;
            // dbg!(body);
            // let val = match req.query_string() {
            //     Some(s) => s,
            //     _ => String::from("default"),
            // };

            // resp(format!("{:?}", storage2.put("who", val.as_bytes())))
            resp("hello")
        })?;

    server.start(&Default::default())
}

fn resp(content: impl AsRef<str>) -> anyhow::Result<Response> {
    Ok(Response::default().body(template(content).into()))
}

fn template(content: impl AsRef<str>) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <title>Web Server</title>
    </head>
    <body>
        {}
    </body>
</html>
"#,
        content.as_ref()
    )
}
