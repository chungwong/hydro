use core::str::FromStr;
use embedded_svc::httpd::{registry::Registry, Response};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use esp_idf_svc::httpd::{Server, ServerRegistry};
use esp_idf_sys::esp_restart;

use crate::{
    light::LightHours,
    storage::{Storage, StorageBase},
};

pub(crate) fn start(master_storage: &Storage) -> anyhow::Result<Server> {
    let storage = master_storage.0.clone();
    let storage2 = master_storage.0.clone();

    let server = ServerRegistry::new()
        .at("/")
        .get(move |_| {
            let ssid = storage.get("WIFI_SSID").unwrap_or_default();
            let pass = storage.get("WIFI_PASS").unwrap_or_default();
            let light_hours = storage.get("LIGHT_HOURS").unwrap_or_default();

            let light_options = LightHours::from_str(&light_hours).unwrap_or_default().to_html_options();

            resp(format!(
                r#"
                <div>
                  <p>Current Date Time: {:?}</p>
                  <form method = "post" action = "/save" enctype="application/x-www-form-urlencoded">
                    <label for="ssid">WiFi SSID:</label><br/>
                    <input type="text" id="ssid" name="WIFI_SSID" value="{ssid}"/><br/>

                    <label for="pass">WiFi Pass:</label><br/>
                    <input type="text" id="pass" name="WIFI_PASS" value="{pass}"/><br/>

                    <label for="cars">Light Hours:</label><br/>
                    <select name="LIGHT_HOURS" id="light-hours" multiple style="width: 100px">
                    {light_options}
                    </select></br/>

                    <input type="submit" value="Submit"/>
                  </form>

                  <form method = "post" action = "/reboot" enctype="application/x-www-form-urlencoded" style="margin-top: 20px">
                    <input type="submit" value="Reboot"/>
                  </form>
                </div>
                "#,
                OffsetDateTime::now_utc().format(&Rfc3339),
            ))
        })?
        .at("/save")
        .post(move |mut req| {
            let body = req.as_bytes()?;
            let allowed_keys = ["WIFI_SSID", "WIFI_PASS", "LIGHT_HOURS"];

            url::form_urlencoded::parse(&body)
            .filter(|(k, _)| allowed_keys.contains(&k.as_ref()))
            .collect::<Vec<_>>()
            .group_by(|(k1, _), (k2, _)| k1 == k2)
            .map(|queries| -> anyhow::Result<bool> {
                let vals = queries.iter().map(|(_, v)| v.to_string()).collect::<Vec<String>>().join(",");
                let key = queries[0].0.clone();

                storage2.put(&key, vals.as_bytes())
            })
            .for_each(drop);

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
