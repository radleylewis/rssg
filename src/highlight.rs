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
