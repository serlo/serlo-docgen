/// Structure definition of meta data for various objects.

/// Meta data for a media file.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MediaMeta {
    license: MediaLicense,
}

/// License data for a media file.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MediaLicense {
    /// Uploader's username.
    user: String,
    /// License common name.
    name: String,
    /// License short name.
    shortname: String,
    /// License text url.
    licenseurl: String,
    /// Media file url.
    url: String,
    /// Contributing authors.
    authors: Vec<String>,
    /// Original image source. Often contains additional markup.
    source: String,
}
