use regex::Regex;
use scraper::{Html, Selector};

pub fn html_to_md(description: &str) -> String {
    let mut md = description.trim().to_string();

    md = md.replace("&lt;", "<");
    md = md.replace("&gt;", ">");
    md = md.replace("&nbsp;", " ");

    let bold_regex = Regex::new(r"</?(b|strong)>").unwrap();
    md = bold_regex.replace_all(&md, "**").to_string();
    
    let italics_regex = Regex::new(r"</?(i|em)>").unwrap();
    md = italics_regex.replace_all(&md, "*").to_string();
    
    let underline_regex = Regex::new(r"</?u>").unwrap();
    md = underline_regex.replace_all(&md, "__").to_string();
    
    let strike_regex = Regex::new(r"</?strike>").unwrap();
    md = strike_regex.replace_all(&md, "~~").to_string();
    
    let underline_regex = Regex::new(r"__+").unwrap();
    md = underline_regex.replace_all(&md, "").to_string();
    
    let span_regex = Regex::new(r"</?span>").unwrap();
    md = span_regex.replace_all(&md, "").to_string();

    let br_regex = Regex::new(r"</?br\s?/?>").unwrap();
    md = br_regex.replace_all(&md, "\n").to_string();

    let a_regex = Regex::new(r#"<a.*?href=["']([^"']*)["'][^>]*>([^<]*)</a>"#).unwrap();
    md = a_regex.replace_all(&md, "[$2]($1)").to_string();

    // let parsed_fragment = Html::parse_fragment(&md);
    //
    // let selector_a = Selector::parse("a").unwrap();
    //
    // for element in parsed_fragment.select(&selector_a) {
    //     let href = element.value().attr("href").unwrap_or_default();
    //     let text = element.text().collect::<String>();
    //     let replacement = format!("[{}]({})", text, href);
    //     md = md.replace(&text, &replacement);
    // }
    //
    // let selector_ul = Selector::parse("ul").unwrap();
    // let selector_ol = Selector::parse("ol").unwrap();
    // let selector_li = Selector::parse("li").unwrap();
    //
    // let ul = parsed_fragment.select(&selector_ul);
    //
    // for element in ul {
    //     let mut replacement = String::new();
    //     for li in element.select(&selector_li) {
    //         replacement.push_str(&format!("* {}\n", li.text().collect::<String>()));
    //     }
    //     md = md.replace(&element.text().collect::<String>(), &replacement);
    // }
    //
    // let ol = parsed_fragment.select(&selector_ol);
    //
    // for element in ol {
    //     let mut replacement = String::new();
    //     for (i, li) in element.select(&selector_li).enumerate() {
    //         replacement.push_str(&format!("{}. {}\n", i + 1, li.text().collect::<String>()));
    //     }
    //     md = md.replace(&element.text().collect::<String>(), &replacement);
    // }
    //
    // let selector_span = Selector::parse("span").unwrap();
    //
    // for element in parsed_fragment.select(&selector_span) {
    //
    //     let text = element.text().collect::<String>();
    //     md = md.replace(&text, "");
    // }
    //
    // let selector_html_blob = Selector::parse("html-blob").unwrap();
    //
    // for element in parsed_fragment.select(&selector_html_blob) {
    //     let text = element.text().collect::<String>();
    //     md = md.replace(&text, "");
    // }

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
