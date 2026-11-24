// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Opens a window from a custom protocol and fetches the app's `tauri` protocol from it (a cross-origin request).

use std::time::Duration;

fn page() -> String {
  let target = if cfg!(windows) || cfg!(target_os = "android") {
    "http://tauri.localhost/index.html"
  } else {
    "tauri://localhost/index.html"
  };
  format!(
    r#"<!doctype html><html><head><meta charset="utf-8"></head><body><script>
const report = (message) => fetch("/report?" + encodeURIComponent(message));
(async () => {{
  await report("href " + location.href + " origin " + location.origin);
  try {{
    const response = await fetch("{target}");
    await report("TAURI_FETCH OK " + response.status + " allow-origin=" + response.headers.get("access-control-allow-origin"));
  }} catch (error) {{
    await report("TAURI_FETCH ERROR " + error);
  }}
  await report("DONE");
}})();
</script></body></html>"#
  )
}

fn main() {
  tauri::Builder::default()
    .register_uri_scheme_protocol("myproto", |_context, request| {
      let uri = request.uri();
      if uri.path() == "/report" {
        let message = percent_encoding::percent_decode_str(uri.query().unwrap_or_default())
          .decode_utf8_lossy()
          .into_owned();
        println!("REPORT {message}");
        if message == "DONE" {
          std::process::exit(0);
        }
        return http::Response::builder().body(Vec::new()).unwrap();
      }
      http::Response::builder()
        .header("Content-Type", "text/html")
        .body(page().into_bytes())
        .unwrap()
    })
    .setup(|app| {
      std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(90));
        println!("REPORT TIMEOUT");
        std::process::exit(2);
      });
      tauri::WebviewWindowBuilder::new(
        app,
        "main",
        tauri::WebviewUrl::CustomProtocol("myproto://localhost/index.html".parse().unwrap()),
      )
      .build()?;
      Ok(())
    })
    .run(tauri::generate_context!(
      "../../examples/helloworld/tauri.conf.json"
    ))
    .expect("error while running tauri application");
}
