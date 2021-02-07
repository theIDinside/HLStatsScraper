pub type FileResult = Result<String, std::io::Error>;

pub trait FileString {
    fn string_read(&mut self) -> FileResult;
}