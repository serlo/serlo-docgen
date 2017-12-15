
/// MFNF Transformation settings.
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    discard_noprint: bool,
}


impl Default for Settings {
    fn default() -> Self {
        Settings {
            discard_noprint: true,
        }
    }
}

