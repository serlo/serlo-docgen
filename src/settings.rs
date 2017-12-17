use util::TravFunc;


/// An export target.
pub struct Target<'a> {
    /// The target name.
    pub name: String,
    /// Target export settings.
    pub settings: Settings,
    /// The path to write output files to.
    pub output_path: String,
    /// A function to call for export.
    pub export_func: TravFunc<'a>,
}

/// General MFNF transformation settings for all targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub download_images: bool,
    pub latex_settings: LaTeXSettings,
}

/// General MFNF transformation settings for all targets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaTeXSettings {
    pub mode: LaTeXMode,
    /// Page trim in mm.
    pub page_trim: f32,
    /// Paper width in mm.
    pub page_width: f32,
    /// Paper height in mm.
    pub page_height: f32,
    /// Font size in pt.
    pub font_size: f32,
    /// Baseline height in pt.
    pub baseline_height: f32,
    /// Paper border in mm as [top, bottom, outer, inner]
    pub border: [f32; 4],
    /// Document class options.
    pub document_options: String,
}

/// The export configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LaTeXMode {
    /// All articles, no filters applied.
    Complete,
    /// Digital print version, reasonably sized.
    PrintDigital,
    /// Print version with special extras for print, (like page trim, etc).
    PrintSpecials,
    /// A minimal version, with only the most important content.
    Minimal,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            download_images: true,
            latex_settings: LaTeXSettings::default(),
        }
    }
}

impl Default for LaTeXSettings {
    fn default() -> Self {
        LaTeXSettings {
            mode: LaTeXMode::Complete,
            page_trim: 0.,
            page_width: 155.,
            page_height: 235.,
            font_size: 9.,
            baseline_height: 12.,
            border: [20.5, 32.6, 22., 18.5],
            document_options: String::from("tocflat, listof=chapterentry"),
        }
    }
}

