#[derive(Debug)]
pub struct Image {
    pub pk: String,
    pub mimetype: String,
    pub etag: String,
    pub last_modified: String,
    pub data: Vec<u8>,
}

impl PartialEq for Image {
    fn eq(&self, other: &Image) -> bool {
        self.pk == other.pk && self.etag == other.etag
    }
}

impl Eq for Image {}
