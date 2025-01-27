use std::{fs, io, path::Path};

pub fn read_dir_to_vec(dir: &str) -> io::Result<Vec<String>> {
    let path = Path::new(dir);
    let mut file_names = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry.file_name();
        if let Some(file_str) = file_name.to_str() {
            file_names.push(file_str.to_string());
        }
    }

    Ok(file_names)
}
