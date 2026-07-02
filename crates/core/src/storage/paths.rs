pub const MOUNT_POINT: &str = "/sdcard";
pub const BOOKS_PATH: &str = "/books";
pub const BOOK_FILES_PATH: &str = "/books/books";
pub const ARTICLE_FILES_PATH: &str = "/books/articles";
pub const CONFIG_PATH: &str = "/config";
pub const TEXT_EXTENSION: &str = ".txt";
pub const RSVP_EXTENSION: &str = ".rsvp";
pub const EPUB_EXTENSION: &str = ".epub";
pub const INDEX_EXTENSION: &str = ".ridx";
pub const DATA_EXTENSION: &str = ".rdat";
pub const TEMP_EXTENSION: &str = ".tmp";
pub const FAILED_EXTENSION: &str = ".failed";
pub const CONVERTING_EXTENSION: &str = ".converting";

pub fn has_text_extension(path: &str) -> bool {
    path.to_lowercase().ends_with(TEXT_EXTENSION)
}

pub fn has_rsvp_extension(path: &str) -> bool {
    path.to_lowercase().ends_with(RSVP_EXTENSION)
}

pub fn has_epub_extension(path: &str) -> bool {
    path.to_lowercase().ends_with(EPUB_EXTENSION)
}

pub fn parent_directory_for_path(path: &str) -> String {
    match path.rfind('/') {
        Some(idx) if idx > 0 => path[..idx].to_string(),
        Some(_) => "/".to_string(),
        None => String::new(),
    }
}

pub fn sibling_path_with_extension(path: &str, extension: &str) -> String {
    let stem = match path.rfind('.') {
        Some(dot_idx) => &path[..dot_idx],
        None => path,
    };
    format!("{}{}", stem, extension)
}

pub fn display_name_for_path(path: &str) -> String {
    let filename = match path.rfind('/') {
        Some(idx) => &path[idx + 1..],
        None => path,
    };
    filename.to_string()
}

pub fn display_name_without_extension(path: &str) -> String {
    let name = display_name_for_path(path);
    match name.rfind('.') {
        Some(dot_idx) => name[..dot_idx].to_string(),
        None => name,
    }
}

pub fn rsvp_cache_path_for_epub(epub_path: &str) -> String {
    sibling_path_with_extension(epub_path, RSVP_EXTENSION)
}

pub fn indexed_index_path_for(path: &str) -> String {
    sibling_path_with_extension(path, INDEX_EXTENSION)
}

pub fn indexed_data_path_for(path: &str) -> String {
    sibling_path_with_extension(path, DATA_EXTENSION)
}

pub fn indexed_temp_path_for(path: &str) -> String {
    sibling_path_with_extension(path, TEMP_EXTENSION)
}

pub fn is_hidden_or_sidecar_path(path: &str) -> bool {
    let filename = match path.rfind('/') {
        Some(idx) => &path[idx + 1..],
        None => path,
    };
    if filename.starts_with('.') {
        return true;
    }
    let ext = match filename.rfind('.') {
        Some(dot_idx) => &filename[dot_idx..],
        None => return false,
    };
    matches!(
        ext.to_lowercase().as_str(),
        ".ridx" | ".rdat" | ".tmp" | ".failed" | ".converting"
    )
}
