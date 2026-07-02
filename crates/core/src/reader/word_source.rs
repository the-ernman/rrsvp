pub trait WordSource {
    fn word_count(&self) -> usize;
    fn word_at(&self, index: usize) -> String;
    fn prefetch_around(&self, _index: usize) {}
}
