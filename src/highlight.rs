use crate::utils::html_decode;
use syntect::{
    highlighting::ThemeSet,
    html::{css_for_theme_with_class_style, ClassStyle, ClassedHTMLGenerator},
    util::LinesWithEndings,
};

pub const CODE_COPY_SCRIPT: &str = "<script>\
document.querySelectorAll('.code-block__copy').forEach(function(btn){\
  btn.addEventListener('click',function(){\
    var code=btn.closest('.code-block').querySelector('code');\
    navigator.clipboard.writeText(code.innerText).then(function(){\
      btn.textContent='Copied!';\
      setTimeout(function(){btn.textContent='Copy';},2000);\
    });\
  });\
});\
</script>";

fn extract_language(opening_tag: &str) -> Option<&str> {
    let start = opening_tag.find("language-")? + "language-".len();
    let rest = &opening_tag[start..];
    let end = rest.find(['"', '\''])?;
    Some(&rest[..end])
}

pub fn convert_callouts(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut remaining = html;

    while let Some(bq_start) = remaining.find("<blockquote>") {
        result.push_str(&remaining[..bq_start]);
        let after_open = &remaining[bq_start + "<blockquote>".len()..];

        let Some(bq_end) = after_open.find("</blockquote>") else {
            result.push_str("<blockquote>");
            remaining = after_open;
            break;
        };

        let inner = after_open[..bq_end].trim();
        remaining = &after_open[bq_end + "</blockquote>".len()..];

        if !inner.starts_with("<p>[!") {
            result.push_str("<blockquote>");
            result.push_str(&after_open[..bq_end]);
            result.push_str("</blockquote>");
            continue;
        }

        // inner = "<p>[!type] title\nbody</p>...rest..."
        let after_type_marker = &inner[5..]; // skip "<p>[!"
        let Some(bracket_close) = after_type_marker.find(']') else {
            result.push_str("<blockquote>");
            result.push_str(&after_open[..bq_end]);
            result.push_str("</blockquote>");
            continue;
        };

        let callout_type = after_type_marker[..bracket_close].trim();
        let type_lower = callout_type.to_lowercase();
        let after_bracket = &after_type_marker[bracket_close + 1..];

        let Some(p_close) = after_bracket.find("</p>") else {
            result.push_str("<blockquote>");
            result.push_str(&after_open[..bq_end]);
            result.push_str("</blockquote>");
            continue;
        };

        let first_p_content = after_bracket[..p_close].trim();
        let after_first_p = after_bracket[p_close + 4..].trim();

        let (title, inline_body) = match first_p_content.find('\n') {
            Some(nl) => {
                let t = first_p_content[..nl].trim();
                let b = first_p_content[nl + 1..].trim();
                (t, if b.is_empty() { None } else { Some(b) })
            }
            None => (first_p_content, None),
        };

        let title_html = if !title.is_empty() {
            format!("<span class=\"callout__title\">{title}</span>")
        } else {
            String::new()
        };

        let mut body_parts = String::new();
        if let Some(ib) = inline_body {
            body_parts.push_str("<p>");
            body_parts.push_str(ib);
            body_parts.push_str("</p>");
        }
        if !after_first_p.is_empty() {
            body_parts.push_str(after_first_p);
        }

        let body_html = if !body_parts.is_empty() {
            format!("<div class=\"callout__body\">{body_parts}</div>")
        } else {
            String::new()
        };

        result.push_str(&format!(
            "<div class=\"callout callout--{type_lower}\">\
            <div class=\"callout__header\">\
            <span class=\"callout__type\">{callout_type}</span>\
            {title_html}\
            </div>\
            {body_html}\
            </div>"
        ));
    }

    result.push_str(remaining);
    result
}

pub fn highlight_code_blocks(html: &str, ss: &syntect::parsing::SyntaxSet) -> String {
    let mut result = String::with_capacity(html.len() + 1024);
    let mut remaining = html;

    while let Some(start) = remaining.find("<pre><code") {
        result.push_str(&remaining[..start]);
        remaining = &remaining[start..];

        let tag_end = match remaining[5..].find('>') {
            Some(i) => 5 + i + 1,
            None => break,
        };

        let lang = extract_language(&remaining[..tag_end]);
        let close = "</code></pre>";

        match remaining.find(close) {
            Some(end) => {
                let code_html = &remaining[tag_end..end];
                let code = html_decode(code_html);

                let highlighted = lang
                    .and_then(|l| ss.find_syntax_by_token(l))
                    .map(|syntax| {
                        let mut gen = ClassedHTMLGenerator::new_with_class_style(
                            syntax,
                            ss,
                            ClassStyle::Spaced,
                        );
                        for line in LinesWithEndings::from(&code) {
                            let _ = gen.parse_html_for_line_which_includes_newline(line);
                        }
                        gen.finalize()
                    })
                    .unwrap_or_else(|| code_html.to_string());

                let lang_label = lang.unwrap_or("");
                result.push_str(&format!(
                    "<div class=\"code-block\">\
                    <div class=\"code-block__header\">\
                    <span class=\"code-block__lang\">{lang_label}</span>\
                    <button class=\"code-block__copy\">Copy</button>\
                    </div>\
                    <pre><code>{highlighted}</code></pre>\
                    </div>"
                ));
                remaining = &remaining[end + close.len()..];
            }
            None => break,
        }
    }

    result.push_str(remaining);
    result
}

pub fn syntax_highlight_css(ts: &ThemeSet) -> String {
    let light = ts
        .themes
        .get("InspiredGitHub")
        .and_then(|t| css_for_theme_with_class_style(t, ClassStyle::Spaced).ok())
        .unwrap_or_default();
    let dark = ts
        .themes
        .get("base16-ocean.dark")
        .and_then(|t| css_for_theme_with_class_style(t, ClassStyle::Spaced).ok())
        .unwrap_or_default();
    format!(
        "<style>\
        pre code{{background:none;padding:0;}}\
        .code-block{{margin:1.5rem 0;}}\
        .code-block pre{{margin:0;border-radius:0 0 4px 4px;}}\
        .code-block__header{{display:flex;justify-content:space-between;align-items:center;\
          background:#e8e8e8;padding:0.3rem 0.75rem;border-radius:4px 4px 0 0;font-size:0.8rem;}}\
        .code-block__lang{{color:#666;text-transform:uppercase;font-size:0.75rem;letter-spacing:0.05em;}}\
        .code-block__copy{{all:unset;cursor:pointer;color:#666;padding:0.15rem 0.5rem;\
          border:1px solid #aaa;border-radius:3px;font-size:0.75rem;}}\
        .code-block__copy:hover{{color:var(--button-bg,#b5a642);border-color:var(--button-bg,#b5a642);}}\
        {light}\
        @media(prefers-color-scheme:dark){{\
          {dark}\
          .code-block__header{{background:#2d3035;}}\
          .code-block__lang,.code-block__copy{{color:#ccc;}}\
        }}\
        #theme:checked ~ * .code-block__header{{background:#2d3035;}}\
        #theme:checked ~ * .code-block__lang,\
        #theme:checked ~ * .code-block__copy{{color:#ccc;}}\
        @media(prefers-color-scheme:dark){{\
          #theme:checked ~ * .code-block__header{{background:#e8e8e8;}}\
          #theme:checked ~ * .code-block__lang,\
          #theme:checked ~ * .code-block__copy{{color:#666;}}\
        }}\
        </style>"
    )
}
