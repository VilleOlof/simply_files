use crate::{App, config::Host};

pub fn add(mut app: App, name: String, url: String, token: Option<String>, autoset: bool) {
    let is_name_in_use = app.config.hosts.iter().any(|h| h.name == name);
    if is_name_in_use {
        tracing::error!("Host name is already in use");
        return;
    }
    app.config.hosts.push(Host {
        name: name.clone(),
        url: url.clone(),
        token: token.clone(),
    });

    tracing::info!("Added {name} ({url}) has a new host");

    if autoset {
        app.config.default_host = Some(name.clone());
        tracing::info!("Set {name} as the default host");
    }

    app.config.save();
}

pub fn ls(app: App) {
    if app.config.hosts.is_empty() {
        tracing::info!("No saved hosts");
        return;
    }

    tracing::info!("Saved hosts:");
    for host in app.config.hosts {
        tracing::info!(
            "{} [{}] {} ({})",
            if let Some(default_host) = &app.config.default_host {
                if default_host == &host.name { ">" } else { " " }
            } else {
                " "
            },
            host.name,
            host.url,
            if let Some(_) = host.token {
                "******"
            } else {
                "No token"
            }
        );
    }
}

pub fn rm(mut app: App, name: String) {
    let exists_in_hosts = app.config.hosts.iter().any(|h| h.name == name);
    if !exists_in_hosts {
        tracing::error!("Host doesn't exist in saved hosts");
        return;
    }

    let mut new_hosts = vec![];
    for host in app.config.hosts {
        if host.name == name {
            tracing::info!("Removed {name} from saved hosts");
            continue;
        }
        new_hosts.push(host);
    }
    app.config.hosts = new_hosts;

    match app.config.default_host {
        Some(default) => {
            if default == name {
                app.config.default_host = None;
                tracing::info!("Removed {name} as default host");
            }
        }
        _ => (),
    };
}

pub fn set(mut app: App, name: String) {
    let exists_in_hosts = app.config.hosts.iter().any(|h| h.name == name);
    if !exists_in_hosts {
        tracing::error!("Host doesn't exist in saved hosts");
        return;
    }

    let old_default = app.config.default_host.clone();

    app.config.default_host = Some(name.clone());

    if old_default != app.config.default_host {
        tracing::info!(
            "Changed default host from {:?} to {}",
            old_default.unwrap_or(String::from("None")),
            name
        );
    } else {
        tracing::error!("{} is already the default host", name);
    }

    app.config.save();
}
