use crate::{app::App, args::FileIdentifier};

pub fn rm(app: &App, file: FileIdentifier) {
    let path = file.path(&app);

    let mut request = ureq::delete(app.get_url(format!(
        "/m/delete_file/{}",
        path.to_string_lossy().to_string()
    )));
    request = app.add_auth_to_req(request);
    request = app.add_agent_to_req(request);
    let mut response = request.call().unwrap();

    if response.status().as_u16() != 200 {
        return tracing::error!(
            "Failed to delete file: {}: {:?}",
            response.status(),
            response.body_mut().read_to_string()
        );
    }

    tracing::info!(
        "Deleted {} successfully",
        path.file_name().unwrap().to_string_lossy()
    );
}
