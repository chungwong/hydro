use embedded_svc::httpd::{registry::Registry, Response};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use esp_idf_svc::httpd::{Server, ServerRegistry};
use esp_idf_sys::esp_restart;

use crate::storage::{Storage, StorageBase};

pub(crate) fn start(master_storage: &Storage) -> anyhow::Result<Server> {
    let storage = master_storage.0.clone();
    let storage2 = master_storage.0.clone();

    let server = ServerRegistry::new()
        .at("/")
        .get(move |_| {
            let ssid = storage.get("WIFI_SSID").unwrap_or_default();
            let pass = storage.get("WIFI_PASS").unwrap_or_default();

            resp(format!(
                r#"
                <div>
                  <p>Current Date Time: {:?}</p>
                  <form method = "post" action = "/save" enctype="application/x-www-form-urlencoded">
                    <label for="ssid">WiFi SSID:</label><br>
                    <input type="text" id="ssid" name="WIFI_SSID" value="{}"><br>

                    <label for="pass">WiFi PASS:</label><br>
                    <input type="text" id="pass" name="WIFI_PASS" value="{}"><br><br>

                    <input type="submit" value="Submit">
                  </form>

                  <form method = "post" action = "/reboot" enctype="application/x-www-form-urlencoded">
                    <input type="submit" value="Reboot">
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
            let body = req.as_bytes()?;
            let allowed_keys = ["WIFI_SSID", "WIFI_PASS"];

            url::form_urlencoded::parse(&body).map(|(k, v)| -> anyhow::Result<()>{
                if allowed_keys.contains(&k.as_ref()) {
                    storage2.put(&k, v.as_bytes())?;
                }
                Ok(())
            }).for_each(std::mem::drop);

            Ok(Response::redirect("/"))
        })?
        .at("/reboot")
        .post(|_| {
            unsafe {
                esp_restart();
            }
            Ok("".into())
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
