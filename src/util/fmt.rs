pub fn as_section(content: String) -> String {
    let empty_string = content.is_empty();
    let mut content = format!("[{content}");
    let len = content.len();
    let rep = 73 - len;
    if rep > 0 {
        let sep = "-".repeat(rep);
        if !empty_string {
            content.push_str(" ");
        } else {
            content.push_str("-");
        }
        content.push_str(&sep);
    }
    content.push_str("]");
    content
}
