use sf_core::FileAccess;

use crate::{app::App, args::FileIdentifier};

pub fn access(app: App, file: FileIdentifier, access: FileAccess) {
    let path_component = match file.clone() {
        FileIdentifier::Id(id) => id,
        FileIdentifier::Path(p) => p.to_string_lossy().to_string(),
    };

    let mut request = ureq::post(app.get_url(format!("/m/access/{path_component}")));
    request = app.add_auth_to_req(request);
    request = app.add_agent_to_req(request);
    request = request.query("access", (access.clone() as i64).to_string());
    if let FileIdentifier::Id(_) = file {
        request = request.query("id", "true")
    }
    let mut response = request.send_empty().unwrap();

    if response.status().as_u16() != 200 {
        return tracing::error!(
            "Failed to change access: {:?}",
            response.body_mut().read_to_string()
        );
    }

    tracing::info!("Successfully changed access to {access:?}");
}
