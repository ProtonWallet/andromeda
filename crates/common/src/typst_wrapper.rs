/// this wrapper is modified from: https://github.com/tfachmann/typst-as-library/tree/main
///
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

use chrono::{Datelike, Timelike, Utc};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::Library;
use typst_timing::TimingScope;

const PT_SANS_REGULAR: &[u8] = include_bytes!("fonts/PTSans-Regular.ttf");

/// Main interface that determines the environment for Typst.
pub struct TypstWrapper {
    /// The content of a source.
    source: Source,

    /// The standard library.
    library: LazyHash<Library>,

    /// Metadata about all known fonts.
    book: LazyHash<FontBook>,

    /// Metadata about all known fonts.
    fonts: Vec<FontSlot>,

    /// Map of all known files.
    files: Arc<Mutex<HashMap<FileId, FileEntry>>>,
}

impl TypstWrapper {
    pub fn new(source: String) -> Self {
        let mut book = FontBook::new();
        let mut fonts = vec![];
        for data in [PT_SANS_REGULAR] {
            let buffer = Bytes::new(data);

            for (i, font) in Font::iter(buffer).enumerate() {
                book.push(font.info().clone());
                fonts.push(FontSlot {
                    path: None,
                    index: i as u32,
                    font: OnceLock::from(Some(font)),
                });
            }
        }

        Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(book),
            fonts: fonts,
            source: Source::detached(source),
            files: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// Holds details about the location of a font and lazily the font itself.
#[derive(Clone, Debug)]
struct FontSlot {
    /// The path at which the font can be found on the system.
    path: Option<PathBuf>,
    /// The index of the font in its collection. Zero if the path does not point
    /// to a collection.
    index: u32,
    /// The lazily loaded font.
    font: OnceLock<Option<Font>>,
}

impl FontSlot {
    /// Get the font for this slot. This loads the font into memory on first
    /// access.
    pub fn get(&self) -> Option<Font> {
        self.font
            .get_or_init(|| {
                let _scope = TimingScope::new("load font");
                let data = std::fs::read(
                    self.path
                        .as_ref()
                        .expect("`path` is not `None` if `font` is uninitialized"),
                )
                .ok()?;
                Font::new(Bytes::new(data), self.index)
            })
            .clone()
    }
}

/// A File that will be stored in the HashMap.
#[derive(Clone, Debug)]
struct FileEntry {
    bytes: Bytes,
    source: Option<Source>,
}

impl FileEntry {
    fn source(&mut self, id: FileId) -> FileResult<Source> {
        let source = if let Some(source) = &self.source {
            source
        } else {
            let contents = std::str::from_utf8(&self.bytes).map_err(|_| FileError::InvalidUtf8)?;
            let contents = contents.trim_start_matches('\u{feff}');
            let source = Source::new(id, contents.into());
            self.source.insert(source)
        };
        Ok(source.clone())
    }
}

impl TypstWrapper {
    /// Helper to handle file requests.
    ///
    /// Requests will be either in packages or a local file.
    fn file(&self, id: FileId) -> FileResult<FileEntry> {
        let files = self.files.lock().map_err(|_| FileError::AccessDenied)?;
        if let Some(entry) = files.get(&id) {
            Ok(entry.clone())
        } else {
            // we don't allow to download unpacked package, raise
            Err(FileError::NotFound("Not allow to download unpacked packages.".into()))
        }
    }
}

/// This is the interface we have to implement such that `typst` can compile it.
impl typst::World for TypstWrapper {
    /// Standard library.
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    /// Metadata about all known Books.
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    /// Accessing the main source file.
    fn main(&self) -> FileId {
        self.source.id()
    }

    /// Accessing a specified source file (based on `FileId`).
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            self.file(id)?.source(id)
        }
    }

    /// Accessing a specified file (non-file).
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.file(id).map(|file| file.bytes.clone())
    }

    /// Accessing a specified font per index of font book.
    fn font(&self, id: usize) -> Option<Font> {
        self.fonts[id].get()
    }

    /// Get the current date.
    ///
    /// Optionally, an offset in hours is given.
    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let offset = offset.unwrap_or(0);
        let now = Utc::now();
        let adjusted_time = now + chrono::Duration::hours(offset);
        let date = Datetime::from_ymd_hms(
            adjusted_time.year(),
            adjusted_time.month() as u8,
            (adjusted_time.day0() as u8) + 1,
            adjusted_time.hour() as u8,
            adjusted_time.minute() as u8,
            adjusted_time.second() as u8,
        );

        date
    }
}
