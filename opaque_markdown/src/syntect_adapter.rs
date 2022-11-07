use comrak::adapters::SyntaxHighlighterAdapter;
use syntect::{
    html::{ClassStyle, ClassedHTMLGenerator},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

#[derive(Debug, Clone)]
pub struct SyntectAdapter {
    syntax_set: SyntaxSet,
}

impl SyntectAdapter {
    pub fn new() -> SyntectAdapter {
        SyntectAdapter {
            syntax_set: SyntaxSet::load_defaults_newlines(),
        }
    }
}

impl SyntaxHighlighterAdapter for SyntectAdapter {
    fn highlight(&self, lang: Option<&str>, code: &str) -> String {
        let syntax_reference = if let Some(lang_name) = lang {
            self.syntax_set.find_syntax_by_token(lang_name)
        } else {
            let Some(line) = code.lines().next() else {
                return code.to_string()
            };
            self.syntax_set.find_syntax_by_first_line(format!("{line}\n").as_str())
        };

        let Some(syntax_reference) = syntax_reference else {
            return code.to_string()
        };

        let mut html_generator = ClassedHTMLGenerator::new_with_class_style(
            syntax_reference,
            &self.syntax_set,
            ClassStyle::SpacedPrefixed { prefix: "syntect-" },
        );
        for line in LinesWithEndings::from(code) {
            match html_generator.parse_html_for_line_which_includes_newline(line) {
                Ok(_) => (),
                Err(_) => return code.to_string(),
            }
        }

        html_generator.finalize()
    }

    fn build_pre_tag(&self, attributes: &std::collections::HashMap<String, String>) -> String {
        if let Some(lang) = attributes.get("lang") {
            format!("<pre lang=\"{lang}\">")
        } else {
            "<pre>".to_string()
        }
    }

    fn build_code_tag(&self, _attributes: &std::collections::HashMap<String, String>) -> String {
        "<code>".to_string()
    }
}
