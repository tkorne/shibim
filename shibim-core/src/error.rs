pub enum LoadError{
    IOError(std::io::Error),
    ParseError
}