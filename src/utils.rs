pub fn rfind_utf8(s: &str, chr: char) -> Option<usize> {
    s.chars()
        .rev()
        .position(|c| c == chr)
        .map(|rev_pos| s.chars().count() - rev_pos - 1)
}

pub fn extract_extension(s: &str) -> String {
    s.chars().rev().take_while(|c| *c != '.').collect::<String>()
}

pub fn cannonicalise<'a>(comps: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let mut stack = Vec::new();
    for t in comps.map(&str::trim) {
        if t.is_empty() {
            continue;
        } else if t == ".." {
            stack.pop();
        } else if t == "." {
            continue;
        } else {
            stack.push(t);
        }
    }
    stack
}
