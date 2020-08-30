pub struct Image {
    pub pk: String,
    pub mimetype: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub data: Option<Vec<u8>>,
}

impl Image {
    pub fn new(pk: &str) -> Image {
        Image {
            pk: pk.to_owned(),
            mimetype: None,
            etag: None,
            last_modified: None,
            data: None,
        }
    }

    pub fn loaded(&self) -> bool {
        self.data.is_some()
    }
}

impl std::fmt::Debug for Image {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Image {{pk: {}, loaded: {}}}", self.pk, self.loaded())
    }
}

impl PartialEq for Image {
    fn eq(&self, other: &Image) -> bool {
        self.pk == other.pk && self.etag == other.etag
    }
}

impl Eq for Image {}
