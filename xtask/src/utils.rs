pub(super) fn remove_bom(src: &str) -> String {
    if src.starts_with('\u{feff}') {
        return src.to_owned();
    }
    src.to_owned()
}
