use std::{env::current_dir, fs::exists, path::PathBuf, time::Instant};

use human_bytes::human_bytes;
use sf_core::{FileAccess, PreviewData};

use crate::{app::App, args::FileIdentifier};

pub fn get(app: App, file: FileIdentifier, local: Option<PathBuf>, metadata: bool, link: bool) {
    let id = file.id(&app);

    let data = get_metadata(&app, &id);
    // if local is provided, use it only as the outpath, but if not we combine current_dir + file_name
    let local = if let Some(local) = local {
        local
    } else {
        current_dir().unwrap().join(&data.file_name)
    };

    if exists(&local).unwrap() {
        return tracing::error!("{} already exists", data.file_name);
    }

    if link {
        tracing::info!("Download link:    {}", app.get_url(format!("/d/{id}")));
        tracing::info!("Raw preview link: {}", app.get_url(format!("/d/{id}?r=t")));
        if let Some(web_url) = app.get_host().web_url {
            tracing::info!("Web preview link: {}", format!("{web_url}/d/{id}"));
        }
        return;
    }

    if metadata {
        tracing::info!("Metadata   {id} / {}", data.file_name);
        tracing::info!("Access     {:?}", FileAccess::from(data.access));
        tracing::info!("Size       {}", human_bytes(data.size as f64));
        tracing::info!("Type       {}", data.mime_type);
        tracing::info!("Created at {:?}", data.created_at);
        if let Some(path) = data.path {
            tracing::info!("Path       {path}");
        }
    } else {
        let mut request = ureq::get(app.get_url(format!("/d/{id}")));
        request = request.query("r", "t");
        request = app.add_auth_to_req(request);
        request = app.add_agent_to_req(request);

        tracing::info!(
            "Downloading {} ({})",
            data.file_name,
            human_bytes(data.size as f64)
        );
        let instant = Instant::now();

        let mut response = request.call().unwrap();
        let mut reader = response.body_mut().as_reader();

        tracing::debug!("Streaming to {local:?}");

        let mut file = std::fs::File::create(&local).unwrap();
        std::io::copy(&mut reader, &mut file).unwrap();

        tracing::debug!("Took {:?} to download file to local", instant.elapsed());
    }
}

fn get_metadata(app: &App, id: &str) -> PreviewData {
    tracing::debug!("Requesting preview data for Get");
    let instant = Instant::now();
    let agent = ureq::agent();

    let mut request = agent.get(app.get_url(format!("/preview_data/{id}")));
    request = app.add_auth_to_req(request);
    request = app.add_agent_to_req(request);

    let mut response = request.call().unwrap();
    let data: PreviewData = response.body_mut().read_json().unwrap();

    tracing::debug!("Preview data took: {:?}", instant.elapsed());

    data
}
