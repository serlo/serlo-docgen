/// Structure definition of meta data for various objects.
use serde_derive::{Deserialize, Serialize};

/// Meta data for a media file.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MediaMeta {
    pub license: MediaLicense,
}

/// License data for a media file.
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MediaLicense {
    /// Uploader's username.
    pub user: String,
    /// License common name.
    pub name: String,
    /// License short name.
    pub shortname: String,
    /// License text url.
    pub licenseurl: String,
    /// Media file url.
    pub url: String,
    /// Contributing authors.
    pub authors: Vec<String>,
    /// Original image source. Often contains additional markup.
    pub source: String,
    /// Link to a page with more detailed information (mostly wikimedia commons)
    pub detailsurl: String,
    /// Original filename of the image
    pub filename: String,
}
