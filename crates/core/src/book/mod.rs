#[derive(Debug, Clone, Default)]
pub struct ChapterMarker {
    pub title: String,
    pub word_index: usize,
}

#[derive(Debug, Clone, Default)]
pub struct BookMetadata {
    pub title: String,
    pub author: String,
    pub word_count: usize,
    pub chapters: Vec<ChapterMarker>,
    pub paragraph_starts: Vec<usize>,
}

impl BookMetadata {
    pub fn clear(&mut self) {
        self.title.clear();
        self.author.clear();
        self.word_count = 0;
        self.chapters.clear();
        self.paragraph_starts.clear();
    }
}
