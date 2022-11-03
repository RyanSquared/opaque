pub(crate) struct Author {
    pub(crate) name: String,
    pub(crate) email: String,
}

// TODO: This should be serde-able, and deserialized from a config file, similar to how
// Jekyll is configured. Information that goes here is typically information that will be
// displayed on the header and footer of every page. However, it will *not* include content such
// as a list of links to display in the nav bar.
pub(crate) struct Config {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) author: Author,
}

pub(crate) struct State {
    pub(crate) config: Config,
    pub(crate) url: String,
    pub(crate) page_map: Vec<(String, String)>,
}

impl State {
    pub(crate) fn new() -> State {
        State {
            config: Config {
                name: "Enigma".to_string(),
                description: "".to_string(),
                author: Author {
                    name: "RyanSquared".to_string(),
                    email: "me@ryansquared.pub".to_string(),
                }
            },
            url: "https://ryansquared.pub".to_string(),
            page_map: vec![],
        }
    }
}
