use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;

#[tauri::command]
pub fn read_flight_data(path: String) -> Vec<String> {
    let mut entries = fs::read_dir(Path::new(&path))
        .expect("Failed to read directory")
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .expect("Failed to read directory");

    entries.sort_by_key(|path| {
        fs::metadata(path)
            .and_then(|metadata| metadata.created())
            .unwrap_or(SystemTime::now())
    });

    entries
        .iter()
        .filter(|path| path.is_file() && path.extension().unwrap_or_default() == "csv")
        .filter_map(|path| path.file_stem())
        .filter_map(|name| name.to_str().map(String::from))
        .rev()
        .collect()
}
