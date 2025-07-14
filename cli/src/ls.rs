use std::path::PathBuf;

use comfy_table::{Cell, Color, Table, presets::UTF8_FULL_CONDENSED};
use human_bytes::human_bytes;
use owo_colors::OwoColorize;
use sf_core::{ClientFile, FileAccess};

use crate::{app::App, args::ArgPath};

pub fn ls(app: App, directory: ArgPath) {
    let path = directory.path();
    let is_root = path.to_string_lossy().is_empty() || path == PathBuf::from("/");

    let url_path = format!(
        "/m/directory{}",
        if is_root {
            String::from("")
        } else {
            String::from("/") + path.to_string_lossy().to_string().as_str()
        }
    );
    let url = app.get_url(&url_path);
    tracing::debug!("Crafted '{url}' for ls");

    let mut request = ureq::get(url);
    request = app.add_auth_to_req(request);
    request = app.add_agent_to_req(request);
    let mut response = request.call().unwrap();

    let files: Vec<ClientFile> = response.body_mut().read_json().unwrap();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .set_header(vec!["Name", "Id", "Size", "Access"]);

    for (idx, file) in files.iter().enumerate() {
        let path = PathBuf::from(&file.path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let mut row = vec![Cell::new(if file.is_dir {
            String::from("/") + &path
        } else {
            path
        })];
        if file.is_dir {
            row.append(&mut vec!["".into(), "".into(), "".into()]);
        } else {
            let id = file.id.clone().unwrap().to_string();
            let size = human_bytes(file.size as f64);
            let acc = FileAccess::from(file.access.unwrap()).to_string();

            row.append(&mut vec![Cell::new(id), Cell::new(size), Cell::new(acc)]);
        }

        if idx % 2 == 0 {
            row = row.into_iter().map(|c| c.fg(Color::DarkGrey)).collect();
        }

        if file.is_dir {
            row = row.into_iter().map(|c| c.fg(Color::DarkCyan)).collect();
        }

        table.add_row(row);
    }

    println!(
        "{}",
        format!(
            "{} files in {:?}",
            files.len().to_string().bright_green(),
            path.bright_magenta()
        )
        .bold()
    );
    println!("{table}");
}
