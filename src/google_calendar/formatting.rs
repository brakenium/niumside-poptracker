use regex::Regex;

pub fn html_to_md(description: &str) -> String {
    let mut md = description.trim().to_string();

    md = md.replace("&lt;", "<");
    md = md.replace("&gt;", ">");
    md = md.replace("&nbsp;", " ");

    if let Ok(bold_regex) = Regex::new(r"</?(b|strong)>") {
        md = bold_regex.replace_all(&md, "**").to_string();
    };

    if let Ok(italics_regex) = Regex::new(r"</?(i|em)>") {
        md = italics_regex.replace_all(&md, "*").to_string();
    };

    if let Ok(underline_regex) = Regex::new(r"</?u>") {
        md = underline_regex.replace_all(&md, "__").to_string();
    };

    if let Ok(strike_regex) = Regex::new(r"</?strike>") {
        md = strike_regex.replace_all(&md, "~~").to_string();
    };

    if let Ok(underline_regex) = Regex::new(r"__+") {
        md = underline_regex.replace_all(&md, "").to_string();
    };

    if let Ok(span_regex) = Regex::new(r"</?span>") {
        md = span_regex.replace_all(&md, "").to_string();
    };

    if let Ok(br_regex) = Regex::new(r"</?br\s?/?>") {
        md = br_regex.replace_all(&md, "\n").to_string();
    };

    if let Ok(a_regex) = Regex::new(r#"<a.*?href=["']([^"']*)["'][^>]*>([^<]*)</a>"#) {
        md = a_regex.replace_all(&md, "[$2]($1)").to_string();
    };

    md
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_to_md() {
        let html = r"
We're alive?<br><br>OPs by &lt;@<span>88392346210680832&gt;</span><br>See here for more info <a href='https://www.google.com'>Google</a>
        ";

        let md = html_to_md(html);

        let result = "We're alive?\n\nOPs by <@88392346210680832>\nSee here for more info [Google](https://www.google.com)";

        assert_eq!(md, result);
    }
}
