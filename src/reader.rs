//! Offline Reader state, TXT pagination and Reader-owned persistence.
//!
//! v0.16.7 completes the FAT 8.3 runtime audit for every Reader-owned write
//! path and exposes layout-aware bookmark page labels. EPUB remains a recognized
//! future format. Per-book TXT resume, aligned Reader controls, paragraph alignment
//! and staged first-page-first opening remain active.

use std::{
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use crate::buttons::ButtonEvent;

/// SD-card library owned by the Reader subsystem.
pub const READER_BOOKS_DIRECTORY: &str = "/sdcard/RUSTMIX/BOOKS";
/// SD-card state directory owned by the Reader subsystem.
pub const READER_STATE_DIRECTORY: &str = "/sdcard/RUSTMIX/READER";
/// Persistent last-read state file.
pub const READER_STATE_FILE: &str = "STATE.TXT";
/// Persistent per-book last-position map.
pub const READER_POSITIONS_FILE: &str = "POSITS.TXT";
/// Legacy long-name per-book positions file accepted read-only for migration.
pub const LEGACY_READER_POSITIONS_FILE: &str = "POSITIONS.TXT";
/// Persistent recent-book list.
pub const READER_RECENT_FILE: &str = "RECENT.TXT";
/// Persistent bookmark list.
pub const READER_BOOKMARKS_FILE: &str = "MARKS.TXT";
/// Persistent Reader-specific preferences.
pub const READER_PREFS_FILE: &str = "PREFS.TXT";
/// SD-backed TXT anchor-cache directory.
pub const READER_CACHE_DIRECTORY: &str = "CACHE";
/// Number of text lines rendered on one portrait Reader page.
pub const READER_LINES_PER_PAGE: usize = 22;
/// Maximum wrapped characters per line for the current Reader body profile.
pub const READER_CHARS_PER_LINE: usize = 43;
/// Nearby page cache retained in RAM while one book is open.
pub const READER_NEARBY_PAGE_CACHE: usize = 8;
/// Maximum bytes read while generating a single page.
pub const READER_PAGE_READ_BYTES: usize = 16 * 1024;
/// Maximum library rows retained for the embedded product UI.
pub const READER_LIBRARY_LIMIT: usize = 128;
/// Maximum per-book last-position records retained on removable storage.
pub const READER_POSITION_LIMIT: usize = 64;
/// Maximum recent-book records retained on removable storage.
pub const READER_RECENT_LIMIT: usize = 16;
/// Maximum bookmark records retained on removable storage.
pub const READER_BOOKMARK_LIMIT: usize = 128;
/// Maximum page anchors accepted from one SD-backed cache file.
pub const READER_CACHE_OFFSET_LIMIT: usize = 4096;
/// Persist an anchor-cache checkpoint after this many newly indexed pages.
pub const READER_CACHE_CHECKPOINT_PAGES: usize = 4;

const READER_PERSISTENCE_VERSION: &str = "1";
const READER_CACHE_VERSION: &str = "3";
const READER_PREFS_VERSION: &str = "1";
const CACHE_FNV_OFFSET: u64 = 0xcbf29ce484222325;
const CACHE_FNV_PRIME: u64 = 0x100000001b3;

/// Reader-supported content types. EPUB remains recognized for the next
/// milestone, but opening it returns a clean placeholder status in v0.16.1.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BookFormat {
    Text,
    Epub,
}

impl BookFormat {
    #[must_use]
    pub const fn badge(self) -> &'static str {
        match self {
            Self::Text => "TXT",
            Self::Epub => "EPUB",
        }
    }

    #[must_use]
    const fn marker(self) -> &'static str {
        match self {
            Self::Text => "txt",
            Self::Epub => "epub",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "txt" => Some(Self::Text),
            "epub" => Some(Self::Epub),
            _ => None,
        }
    }
}

/// One Reader library row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReaderBook {
    pub path: String,
    pub title: String,
    pub format: BookFormat,
    pub size_bytes: u64,
    pub modified_seconds: u64,
}

/// Stable logical reading position used by STATE.TXT, RECENT.TXT and
/// MARKS.TXT. TXT byte offsets remain valid independently of generated UI page
/// labels. EPUB will reuse this boundary with a structured anchor in v0.17.0.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReaderLocation {
    pub path: String,
    pub title: String,
    pub format: BookFormat,
    pub size_bytes: u64,
    pub modified_seconds: u64,
    pub page_index: usize,
    pub byte_offset: u64,
}

impl ReaderLocation {
    #[must_use]
    pub fn as_book(&self) -> ReaderBook {
        ReaderBook {
            path: self.path.clone(),
            title: self.title.clone(),
            format: self.format,
            size_bytes: self.size_bytes,
            modified_seconds: self.modified_seconds,
        }
    }

    #[must_use]
    fn matches_book(&self, book: &ReaderBook) -> bool {
        self.path == book.path
            && self.size_bytes == book.size_bytes
            && self.modified_seconds == book.modified_seconds
            && self.format == book.format
    }

    #[must_use]
    fn same_position(&self, other: &Self) -> bool {
        self.path == other.path && self.byte_offset == other.byte_offset
    }
}

/// One list row rendered by Recent, Books, Files or Bookmarks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReaderLibraryEntry {
    pub book: ReaderBook,
    pub location: Option<ReaderLocation>,
}

/// Reader Library tab model.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ReaderLibraryTab {
    Recent,
    #[default]
    Books,
    Files,
    Bookmarks,
}

impl ReaderLibraryTab {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Recent => "Recent",
            Self::Books => "Books",
            Self::Files => "Files",
            Self::Bookmarks => "Bookmarks",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Recent => Self::Books,
            Self::Books => Self::Files,
            Self::Files => Self::Bookmarks,
            Self::Bookmarks => Self::Recent,
        }
    }
}

/// Text decoding mode detected when a TXT book is opened.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextEncoding {
    Utf8,
    Utf8Bom,
    Windows1252,
}

impl TextEncoding {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Utf8 => "UTF-8",
            Self::Utf8Bom => "UTF-8 BOM",
            Self::Windows1252 => "WIN-1252",
        }
    }
}

/// E-paper-friendly Reader page theme.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ReadingTheme {
    #[default]
    Classic,
    HighContrast,
}

impl ReadingTheme {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Classic => "Classic",
            Self::HighContrast => "High Contrast",
        }
    }

    #[must_use]
    pub const fn marker(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::HighContrast => "high-contrast",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Classic => Self::HighContrast,
            Self::HighContrast => Self::Classic,
        }
    }

    #[must_use]
    pub const fn previous(self) -> Self {
        self.next()
    }

    fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "classic" => Ok(Self::Classic),
            "high-contrast" | "high_contrast" => Ok(Self::HighContrast),
            other => Err(format!("unsupported theme value {other:?}")),
        }
    }
}

/// Reader-page orientation independent from the portrait system UI.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ReaderOrientation {
    #[default]
    Portrait,
    Landscape,
}

impl ReaderOrientation {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Portrait => "Portrait",
            Self::Landscape => "Landscape",
        }
    }

    #[must_use]
    pub const fn marker(self) -> &'static str {
        match self {
            Self::Portrait => "portrait",
            Self::Landscape => "landscape",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Portrait => Self::Landscape,
            Self::Landscape => Self::Portrait,
        }
    }

    #[must_use]
    pub const fn previous(self) -> Self {
        self.next()
    }

    fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "portrait" => Ok(Self::Portrait),
            "landscape" => Ok(Self::Landscape),
            other => Err(format!("unsupported orientation value {other:?}")),
        }
    }
}

/// Reader-specific book font size. This is intentionally independent from
/// `/sdcard/RUSTMIX/DISPLAY.TXT`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BookFontSize {
    Small,
    #[default]
    Medium,
    Large,
    XLarge,
}

impl BookFontSize {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Medium => "Medium",
            Self::Large => "Large",
            Self::XLarge => "XLarge",
        }
    }

    #[must_use]
    pub const fn marker(self) -> &'static str {
        match self {
            Self::Small => "small",
            Self::Medium => "medium",
            Self::Large => "large",
            Self::XLarge => "xlarge",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Small => Self::Medium,
            Self::Medium => Self::Large,
            Self::Large => Self::XLarge,
            Self::XLarge => Self::Small,
        }
    }

    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::Small => Self::XLarge,
            Self::Medium => Self::Small,
            Self::Large => Self::Medium,
            Self::XLarge => Self::Large,
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "small" => Ok(Self::Small),
            "medium" => Ok(Self::Medium),
            "large" => Ok(Self::Large),
            "xlarge" | "extra-large" | "extra_large" => Ok(Self::XLarge),
            other => Err(format!("unsupported book_font_size value {other:?}")),
        }
    }
}

/// Reader-specific body font family. Serif is a generated printable-ASCII
/// DejaVu Serif bitmap strike; raw font files are not distributed.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BookFont {
    Inter,
    AtkinsonHyperlegible,
    #[default]
    Serif,
}

impl BookFont {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Inter => "Inter",
            Self::AtkinsonHyperlegible => "Atkinson",
            Self::Serif => "Serif",
        }
    }

    #[must_use]
    pub const fn marker(self) -> &'static str {
        match self {
            Self::Inter => "inter",
            Self::AtkinsonHyperlegible => "atkinson-hyperlegible",
            Self::Serif => "serif",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Inter => Self::AtkinsonHyperlegible,
            Self::AtkinsonHyperlegible => Self::Serif,
            Self::Serif => Self::Inter,
        }
    }

    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::Inter => Self::Serif,
            Self::AtkinsonHyperlegible => Self::Inter,
            Self::Serif => Self::AtkinsonHyperlegible,
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "inter" => Ok(Self::Inter),
            "atkinson" | "atkinson-hyperlegible" | "atkinson_hyperlegible" => {
                Ok(Self::AtkinsonHyperlegible)
            }
            "serif" | "dejavu-serif" => Ok(Self::Serif),
            other => Err(format!("unsupported book_font value {other:?}")),
        }
    }
}

/// Reader paragraph alignment. Justified is the default e-book presentation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ParagraphAlignment {
    #[default]
    Justified,
    Left,
    Center,
    Right,
}

impl ParagraphAlignment {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Justified => "Justified",
            Self::Left => "Left",
            Self::Center => "Center",
            Self::Right => "Right",
        }
    }

    #[must_use]
    pub const fn marker(self) -> &'static str {
        match self {
            Self::Justified => "justified",
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Justified => Self::Left,
            Self::Left => Self::Center,
            Self::Center => Self::Right,
            Self::Right => Self::Justified,
        }
    }

    #[must_use]
    pub const fn previous(self) -> Self {
        match self {
            Self::Justified => Self::Right,
            Self::Left => Self::Justified,
            Self::Center => Self::Left,
            Self::Right => Self::Center,
        }
    }

    fn parse(value: &str) -> Result<Self, String> {
        match value.trim().to_ascii_lowercase().as_str() {
            "justified" | "justify" => Ok(Self::Justified),
            "left" => Ok(Self::Left),
            "center" | "centred" => Ok(Self::Center),
            "right" => Ok(Self::Right),
            other => Err(format!("unsupported paragraph_alignment value {other:?}")),
        }
    }
}

/// Layout dimensions affecting TXT pagination and cache fingerprints.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderLayout {
    pub chars_per_line: usize,
    pub lines_per_page: usize,
    pub orientation: ReaderOrientation,
    pub font_size: BookFontSize,
    pub book_font: BookFont,
    pub paragraph_alignment: ParagraphAlignment,
}

/// Reader-owned preference file persisted as `/RUSTMIX/READER/PREFS.TXT`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderPreferences {
    pub theme: ReadingTheme,
    pub orientation: ReaderOrientation,
    pub font_size: BookFontSize,
    pub book_font: BookFont,
    pub paragraph_alignment: ParagraphAlignment,
    pub show_progress: bool,
}

impl Default for ReaderPreferences {
    fn default() -> Self {
        Self {
            theme: ReadingTheme::Classic,
            orientation: ReaderOrientation::Portrait,
            font_size: BookFontSize::Medium,
            book_font: BookFont::Serif,
            paragraph_alignment: ParagraphAlignment::Justified,
            show_progress: true,
        }
    }
}

impl ReaderPreferences {
    #[must_use]
    pub const fn layout(self) -> ReaderLayout {
        // Reader pages share one bounded body viewport across Classic and
        // High Contrast. Serif uses proportional glyphs, so its conservative
        // character budget is slightly smaller than the UI-family strikes.
        // A final pixel clip in the renderer guards unusually wide lines.
        let (chars_per_line, lines_per_page) =
            match (self.orientation, self.font_size, self.book_font) {
                (ReaderOrientation::Portrait, BookFontSize::Small, BookFont::Serif) => (39, 25),
                (ReaderOrientation::Portrait, BookFontSize::Medium, BookFont::Serif) => (35, 22),
                (ReaderOrientation::Portrait, BookFontSize::Large, BookFont::Serif) => (30, 19),
                (ReaderOrientation::Portrait, BookFontSize::XLarge, BookFont::Serif) => (25, 16),
                (ReaderOrientation::Portrait, BookFontSize::Small, _) => (43, 25),
                (ReaderOrientation::Portrait, BookFontSize::Medium, _) => (38, 22),
                (ReaderOrientation::Portrait, BookFontSize::Large, _) => (33, 19),
                (ReaderOrientation::Portrait, BookFontSize::XLarge, _) => (27, 16),
                (ReaderOrientation::Landscape, BookFontSize::Small, BookFont::Serif) => (68, 13),
                (ReaderOrientation::Landscape, BookFontSize::Medium, BookFont::Serif) => (58, 11),
                (ReaderOrientation::Landscape, BookFontSize::Large, BookFont::Serif) => (49, 10),
                (ReaderOrientation::Landscape, BookFontSize::XLarge, BookFont::Serif) => (41, 8),
                (ReaderOrientation::Landscape, BookFontSize::Small, _) => (72, 13),
                (ReaderOrientation::Landscape, BookFontSize::Medium, _) => (64, 11),
                (ReaderOrientation::Landscape, BookFontSize::Large, _) => (55, 10),
                (ReaderOrientation::Landscape, BookFontSize::XLarge, _) => (45, 8),
            };
        ReaderLayout {
            chars_per_line,
            lines_per_page,
            orientation: self.orientation,
            font_size: self.font_size,
            book_font: self.book_font,
            paragraph_alignment: self.paragraph_alignment,
        }
    }

    #[must_use]
    pub fn serialized(self) -> String {
        let show_progress = if self.show_progress { "true" } else { "false" };
        format!(
            "version={}\ntheme={}\norientation={}\nfont_size={}\nbook_font={}\nparagraph_alignment={}\nshow_progress={}\n",
            READER_PREFS_VERSION,
            self.theme.marker(),
            self.orientation.marker(),
            self.font_size.marker(),
            self.book_font.marker(),
            self.paragraph_alignment.marker(),
            show_progress,
        )
    }

    fn parse(text: &str) -> Result<Self, String> {
        let mut prefs = Self::default();
        let mut version = None;
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| "Reader preference line must contain '='".to_string())?;
            match key.trim() {
                "version" => version = Some(value.trim().to_string()),
                "theme" => prefs.theme = ReadingTheme::parse(value)?,
                "orientation" => prefs.orientation = ReaderOrientation::parse(value)?,
                "font_size" => prefs.font_size = BookFontSize::parse(value)?,
                "book_font" => prefs.book_font = BookFont::parse(value)?,
                "paragraph_alignment" => {
                    prefs.paragraph_alignment = ParagraphAlignment::parse(value)?
                }
                "show_progress" => {
                    prefs.show_progress = match value.trim() {
                        "true" => true,
                        "false" => false,
                        _ => return Err("show_progress must be true or false".into()),
                    }
                }
                other => return Err(format!("unsupported Reader preference key {other:?}")),
            }
        }
        if version.as_deref() != Some(READER_PREFS_VERSION) {
            return Err("unsupported Reader preference version".into());
        }
        Ok(prefs)
    }
}

/// Coarse stages used by the e-paper loading screen. The runtime advances only
/// at meaningful boundaries so progress remains visible without excessive
/// refreshes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReaderLoadingStage {
    OpeningFile,
    DetectingEncoding,
    LoadingSavedPosition,
    UpdatingLayout,
    BuildingFirstPage,
    IndexingNearbyPages,
    Ready,
    UnsupportedEpub,
    Failed,
}

impl ReaderLoadingStage {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::OpeningFile => "Opening file",
            Self::DetectingEncoding => "Detecting text encoding",
            Self::LoadingSavedPosition => "Loading saved position",
            Self::UpdatingLayout => "Updating layout cache",
            Self::BuildingFirstPage => "Building first page",
            Self::IndexingNearbyPages => "Caching nearby pages",
            Self::Ready => "Ready",
            Self::UnsupportedEpub => "EPUB opens in v0.17.0",
            Self::Failed => "Unable to open book",
        }
    }

    #[must_use]
    pub const fn progress(self) -> u8 {
        match self {
            Self::OpeningFile => 10,
            Self::DetectingEncoding => 25,
            Self::LoadingSavedPosition => 40,
            Self::UpdatingLayout => 45,
            Self::BuildingFirstPage => 55,
            Self::IndexingNearbyPages => 80,
            Self::Ready => 100,
            Self::UnsupportedEpub | Self::Failed => 100,
        }
    }
}

/// Pending staged book open retained while the loading screen is visible.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingReaderOpen {
    pub book: ReaderBook,
    pub stage: ReaderLoadingStage,
    pub encoding: Option<TextEncoding>,
    pub resume: Option<ReaderLocation>,
    pub message: String,
}

/// One wrapped Reader line. `paragraph_end` prevents Justified rendering from
/// stretching the final line of a paragraph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReaderPageLine {
    pub text: String,
    pub paragraph_end: bool,
}

/// One cached portrait page and its byte anchor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReaderCachedPage {
    /// Absolute book-page index, independent of a cache-recovery base offset.
    pub page_index: usize,
    pub byte_offset: u64,
    pub next_byte_offset: u64,
    pub lines: Vec<ReaderPageLine>,
}

/// SD-backed page-anchor cache. The cache is intentionally text-based and
/// bounded so corrupt records can be rejected without blocking book opening.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ReaderAnchorCache {
    fingerprint: u64,
    base_page: usize,
    offsets: Vec<u64>,
    indexed_through: u64,
    complete: bool,
}

/// Active TXT session. Generated page anchors and nearby rendered pages remain
/// bounded in RAM and are rebuilt lazily when the reader advances.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReaderSession {
    pub book: ReaderBook,
    pub encoding: TextEncoding,
    pub layout: ReaderLayout,
    /// Local index within page_offsets.
    pub current_page: usize,
    /// Absolute page index represented by page_offsets[0]. Normally zero. A
    /// non-zero value is allowed when STATE.TXT survives but a cache is absent.
    pub page_number_base: usize,
    pub page_offsets: Vec<u64>,
    pub indexed_through: u64,
    pub index_complete: bool,
    pub cache: Vec<ReaderCachedPage>,
}

impl ReaderSession {
    #[must_use]
    pub fn current_absolute_page(&self) -> usize {
        self.page_number_base.saturating_add(self.current_page)
    }

    #[must_use]
    pub fn current_cached_page(&self) -> Option<&ReaderCachedPage> {
        let absolute = self.current_absolute_page();
        self.cache.iter().find(|page| page.page_index == absolute)
    }

    #[must_use]
    pub fn current_location(&self) -> ReaderLocation {
        let byte_offset = self
            .page_offsets
            .get(self.current_page)
            .copied()
            .or_else(|| self.current_cached_page().map(|page| page.byte_offset))
            .unwrap_or(0);
        ReaderLocation {
            path: self.book.path.clone(),
            title: self.book.title.clone(),
            format: self.book.format,
            size_bytes: self.book.size_bytes,
            modified_seconds: self.book.modified_seconds,
            page_index: self.current_absolute_page(),
            byte_offset,
        }
    }

    #[must_use]
    pub fn progress_percent(&self) -> u8 {
        if self.book.size_bytes == 0 {
            return 100;
        }
        ((self.indexed_through.saturating_mul(100) / self.book.size_bytes).min(100)) as u8
    }

    #[must_use]
    pub fn page_label(&self) -> String {
        if self.index_complete {
            format!(
                "{}/{}",
                self.current_absolute_page() + 1,
                self.page_number_base + self.page_offsets.len()
            )
        } else {
            format!("{}+", self.current_absolute_page() + 1)
        }
    }

    fn push_cached_page(&mut self, page: ReaderCachedPage) {
        if let Some(existing) = self
            .cache
            .iter_mut()
            .find(|cached| cached.page_index == page.page_index)
        {
            *existing = page;
            return;
        }
        self.cache.push(page);
        self.cache.sort_by_key(|page| page.page_index);
        while self.cache.len() > READER_NEARBY_PAGE_CACHE {
            let current = self.current_absolute_page();
            let remove = if current.saturating_sub(self.cache[0].page_index)
                > self
                    .cache
                    .last()
                    .map_or(0, |page| page.page_index.saturating_sub(current))
            {
                0
            } else {
                self.cache.len() - 1
            };
            self.cache.remove(remove);
        }
    }

    fn ensure_page_cached(&mut self, local_page_index: usize) -> Result<(), String> {
        let absolute = self.page_number_base.saturating_add(local_page_index);
        if self.cache.iter().any(|page| page.page_index == absolute) {
            return Ok(());
        }
        let offset = *self
            .page_offsets
            .get(local_page_index)
            .ok_or_else(|| "page anchor is not indexed yet".to_string())?;
        let page = read_txt_page(&self.book, self.encoding, self.layout, offset, absolute)?;
        self.push_cached_page(page);
        Ok(())
    }

    fn index_one_page(&mut self) -> Result<bool, String> {
        if self.index_complete {
            return Ok(false);
        }
        let absolute_page = self
            .page_number_base
            .saturating_add(self.page_offsets.len());
        let offset = self.indexed_through;
        if offset >= self.book.size_bytes {
            self.index_complete = true;
            return Ok(false);
        }
        let page = read_txt_page(
            &self.book,
            self.encoding,
            self.layout,
            offset,
            absolute_page,
        )?;
        if page.next_byte_offset <= offset {
            self.index_complete = true;
            return Ok(false);
        }
        self.page_offsets.push(offset);
        self.indexed_through = page.next_byte_offset;
        self.index_complete = self.indexed_through >= self.book.size_bytes;
        self.push_cached_page(page);
        Ok(true)
    }

    pub fn next_page(&mut self) -> Result<(), String> {
        let target = self.current_page.saturating_add(1);
        while target >= self.page_offsets.len() && !self.index_complete {
            self.index_one_page()?;
        }
        if target < self.page_offsets.len() {
            self.current_page = target;
            self.ensure_page_cached(target)?;
        }
        Ok(())
    }

    pub fn previous_page(&mut self) -> Result<(), String> {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.ensure_page_cached(self.current_page)?;
        }
        Ok(())
    }

    #[must_use]
    fn anchor_cache(&self) -> ReaderAnchorCache {
        ReaderAnchorCache {
            fingerprint: book_fingerprint(&self.book, self.layout),
            base_page: self.page_number_base,
            offsets: self.page_offsets.clone(),
            indexed_through: self.indexed_through,
            complete: self.index_complete,
        }
    }
}

/// Reader Options action rows. Editable values live on the separate
/// Reading Preferences editor so menu controls match the rest of the firmware.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReaderOption {
    Bookmark,
    Bookmarks,
    TableOfContents,
    ReadingPreferences,
    ClearGhosting,
    GoToLibrary,
    GoHome,
}

impl ReaderOption {
    pub const ALL: [Self; 7] = [
        Self::Bookmark,
        Self::Bookmarks,
        Self::TableOfContents,
        Self::ReadingPreferences,
        Self::ClearGhosting,
        Self::GoToLibrary,
        Self::GoHome,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Bookmark => "Add / Remove Bookmark",
            Self::Bookmarks => "Bookmarks",
            Self::TableOfContents => "Table of Contents",
            Self::ReadingPreferences => "Reading Preferences",
            Self::ClearGhosting => "Clear Ghosting",
            Self::GoToLibrary => "Go to Library",
            Self::GoHome => "Go Home",
        }
    }

    #[must_use]
    pub const fn badge(self) -> &'static str {
        match self {
            Self::Bookmark => "TOGGLE",
            Self::Bookmarks => "LIST",
            Self::TableOfContents => "NONE",
            Self::ReadingPreferences => ">>>",
            Self::ClearGhosting => "RUN",
            Self::GoToLibrary | Self::GoHome => ">>>",
        }
    }
}

/// Reading Preferences editor rows. UP/DOWN changes the active value and
/// SELECT advances to the next row, matching the firmware editor convention.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadingPreference {
    ReadingTheme,
    Orientation,
    BookFontSize,
    BookFont,
    ParagraphAlignment,
    ShowProgress,
}

impl ReadingPreference {
    pub const ALL: [Self; 6] = [
        Self::ReadingTheme,
        Self::Orientation,
        Self::BookFontSize,
        Self::BookFont,
        Self::ParagraphAlignment,
        Self::ShowProgress,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ReadingTheme => "Reading Theme",
            Self::Orientation => "Orientation",
            Self::BookFontSize => "Book Font Size",
            Self::BookFont => "Book Font",
            Self::ParagraphAlignment => "Paragraph Alignment",
            Self::ShowProgress => "Show Progress",
        }
    }
}

/// Coarse background tick result used by main.rs to refresh only meaningful
/// loading-screen transitions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReaderTickOutcome {
    None,
    LoadingStageChanged,
    FirstPageReady,
    BackgroundCacheAdvanced,
    Failed,
}

/// Non-fatal Reader persistence startup report.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReaderPersistenceReport {
    pub state_loaded: bool,
    pub preferences_loaded: bool,
    pub position_count: usize,
    pub recent_count: usize,
    pub bookmark_count: usize,
    pub warning: Option<String>,
}

/// Hardware-independent Reader UI state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReaderUiState {
    pub books_root: String,
    pub state_root: String,
    pub books: Vec<ReaderBook>,
    pub positions: Vec<ReaderLocation>,
    pub recent: Vec<ReaderLocation>,
    pub bookmarks: Vec<ReaderLocation>,
    pub resume: Option<ReaderLocation>,
    pub preferences: ReaderPreferences,
    pub library_error: Option<String>,
    pub persistence_warning: Option<String>,
    pub library_tab: ReaderLibraryTab,
    /// Row zero is the explicit tab-control row; book rows begin at one.
    pub library_selected: usize,
    pub bookmarks_selected: usize,
    pub loading: Option<PendingReaderOpen>,
    pub session: Option<ReaderSession>,
    pub options_selected: usize,
    pub preferences_selected: usize,
    preferences_layout_dirty: bool,
    pub last_message: Option<String>,
    persistence_event: Option<String>,
    last_persistence_event: Option<String>,
    clear_ghost_requested: bool,
}

impl Default for ReaderUiState {
    fn default() -> Self {
        Self {
            books_root: READER_BOOKS_DIRECTORY.into(),
            state_root: READER_STATE_DIRECTORY.into(),
            books: Vec::new(),
            positions: Vec::new(),
            recent: Vec::new(),
            bookmarks: Vec::new(),
            resume: None,
            preferences: ReaderPreferences::default(),
            library_error: None,
            persistence_warning: None,
            library_tab: ReaderLibraryTab::default(),
            library_selected: 0,
            bookmarks_selected: 0,
            loading: None,
            session: None,
            options_selected: 0,
            preferences_selected: 0,
            preferences_layout_dirty: false,
            last_message: None,
            persistence_event: None,
            last_persistence_event: None,
            clear_ghost_requested: false,
        }
    }
}

impl ReaderUiState {
    #[must_use]
    pub fn with_books_root(root: impl Into<String>) -> Self {
        Self {
            books_root: root.into(),
            ..Self::default()
        }
    }

    #[must_use]
    pub fn with_roots(books_root: impl Into<String>, state_root: impl Into<String>) -> Self {
        Self {
            books_root: books_root.into(),
            state_root: state_root.into(),
            ..Self::default()
        }
    }

    /// Load persisted state without making startup dependent on removable
    /// storage. Corrupt records are ignored and reported as a warning.
    pub fn load_persistent_state(&mut self) -> ReaderPersistenceReport {
        let mut warnings = Vec::new();
        let preferences_loaded = match load_preferences(&self.preferences_path()) {
            Ok(Some(preferences)) => {
                self.preferences = preferences;
                true
            }
            Ok(None) => false,
            Err(error) => {
                warnings.push(format!("PREFS.TXT: {error}"));
                false
            }
        };
        self.resume = match load_location_record(&self.state_path()) {
            Ok(value) => value,
            Err(error) => {
                warnings.push(format!("STATE.TXT: {error}"));
                None
            }
        };
        self.positions = match self.load_positions_with_legacy_migration() {
            Ok(value) => value,
            Err(error) => {
                warnings.push(format!("POSITS.TXT: {error}"));
                Vec::new()
            }
        };
        self.recent = match load_location_list(&self.recent_path(), READER_RECENT_LIMIT) {
            Ok(value) => value,
            Err(error) => {
                warnings.push(format!("RECENT.TXT: {error}"));
                Vec::new()
            }
        };
        self.bookmarks = match load_location_list(&self.bookmarks_path(), READER_BOOKMARK_LIMIT) {
            Ok(value) => value,
            Err(error) => {
                warnings.push(format!("MARKS.TXT: {error}"));
                Vec::new()
            }
        };
        self.bookmarks_selected = self
            .bookmarks_selected
            .min(self.bookmarks.len().saturating_sub(1));
        let warning = if warnings.is_empty() {
            None
        } else {
            Some(warnings.join("; "))
        };
        self.persistence_warning = warning.clone();
        ReaderPersistenceReport {
            state_loaded: self.resume.is_some(),
            preferences_loaded,
            position_count: self.positions.len(),
            recent_count: self.recent.len(),
            bookmark_count: self.bookmarks.len(),
            warning,
        }
    }

    pub fn refresh_library(&mut self) {
        match scan_txt_library(&self.books_root) {
            Ok(books) => {
                self.books = books;
                self.library_error = None;
            }
            Err(error) => {
                self.books.clear();
                self.library_error = Some(error);
            }
        }
        self.library_selected = 0;
    }

    #[must_use]
    pub fn can_continue(&self) -> bool {
        self.session.is_some() || self.resume.is_some() || !self.recent.is_empty()
    }

    pub fn request_continue(&mut self) -> bool {
        let Some(location) = self.resume.clone().or_else(|| self.recent.first().cloned()) else {
            return false;
        };
        self.request_open_book(location.as_book(), Some(location));
        true
    }

    #[must_use]
    pub fn visible_entries(&self) -> Vec<ReaderLibraryEntry> {
        match self.library_tab {
            ReaderLibraryTab::Recent => self
                .recent
                .iter()
                .cloned()
                .map(|location| ReaderLibraryEntry {
                    book: location.as_book(),
                    location: Some(location),
                })
                .collect(),
            ReaderLibraryTab::Books | ReaderLibraryTab::Files => self
                .books
                .iter()
                .cloned()
                .map(|book| ReaderLibraryEntry {
                    location: self.saved_position_for_book(&book),
                    book,
                })
                .collect(),
            ReaderLibraryTab::Bookmarks => self
                .bookmarks
                .iter()
                .cloned()
                .map(|location| ReaderLibraryEntry {
                    book: location.as_book(),
                    location: Some(location),
                })
                .collect(),
        }
    }

    #[must_use]
    pub fn library_row_count(&self) -> usize {
        self.visible_entries().len().saturating_add(1)
    }

    pub fn apply_library_button(&mut self, event: ButtonEvent) -> bool {
        let count = self.library_row_count().max(1);
        match event {
            ButtonEvent::Up => {
                self.library_selected = self.library_selected.checked_sub(1).unwrap_or(count - 1);
                false
            }
            ButtonEvent::Down => {
                self.library_selected = (self.library_selected + 1) % count;
                false
            }
            ButtonEvent::Select if self.library_selected == 0 => {
                self.library_tab = self.library_tab.next();
                self.library_selected = 0;
                false
            }
            ButtonEvent::Select => self.request_open_visible(self.library_selected - 1),
        }
    }

    pub fn apply_bookmarks_button(&mut self, event: ButtonEvent) -> bool {
        if self.bookmarks.is_empty() {
            return false;
        }
        match event {
            ButtonEvent::Up => {
                self.bookmarks_selected = self
                    .bookmarks_selected
                    .checked_sub(1)
                    .unwrap_or(self.bookmarks.len() - 1);
                false
            }
            ButtonEvent::Down => {
                self.bookmarks_selected = (self.bookmarks_selected + 1) % self.bookmarks.len();
                false
            }
            ButtonEvent::Select => self.request_open_bookmark(self.bookmarks_selected),
        }
    }

    pub fn request_open_visible(&mut self, visible_index: usize) -> bool {
        let Some(entry) = self.visible_entries().get(visible_index).cloned() else {
            return false;
        };
        let resume = entry
            .location
            .or_else(|| self.saved_position_for_book(&entry.book))
            .or_else(|| {
                self.resume
                    .clone()
                    .filter(|location| location.matches_book(&entry.book))
            });
        self.request_open_book(entry.book, resume);
        true
    }

    #[must_use]
    fn saved_position_for_book(&self, book: &ReaderBook) -> Option<ReaderLocation> {
        self.positions
            .iter()
            .find(|location| location.matches_book(book))
            .cloned()
    }

    pub fn request_open_bookmark(&mut self, bookmark_index: usize) -> bool {
        let Some(location) = self.bookmarks.get(bookmark_index).cloned() else {
            return false;
        };
        self.request_open_book(location.as_book(), Some(location));
        true
    }

    fn request_open_book(&mut self, book: ReaderBook, resume: Option<ReaderLocation>) {
        self.loading = Some(PendingReaderOpen {
            book,
            stage: ReaderLoadingStage::OpeningFile,
            encoding: None,
            resume,
            message: "Preparing reader...".into(),
        });
    }

    fn request_layout_rebuild(&mut self) -> bool {
        let Some(session) = self.session.as_ref() else {
            self.persist_preferences_best_effort();
            return false;
        };
        let book = session.book.clone();
        let encoding = session.encoding;
        let resume = session.current_location();
        self.loading = Some(PendingReaderOpen {
            book,
            stage: ReaderLoadingStage::UpdatingLayout,
            encoding: Some(encoding),
            resume: Some(resume),
            message: "Rebuilding the current page first...".into(),
        });
        self.persist_preferences_best_effort();
        true
    }

    pub fn cancel_loading(&mut self) {
        self.loading = None;
        self.last_message = Some("Book opening cancelled".into());
    }

    #[must_use]
    pub fn loading_stage(&self) -> Option<ReaderLoadingStage> {
        self.loading.as_ref().map(|loading| loading.stage)
    }

    pub fn tick(&mut self) -> ReaderTickOutcome {
        if let Some(mut loading) = self.loading.take() {
            let outcome = match loading.stage {
                ReaderLoadingStage::OpeningFile => {
                    loading.stage = match loading.book.format {
                        BookFormat::Text => ReaderLoadingStage::DetectingEncoding,
                        BookFormat::Epub => ReaderLoadingStage::UnsupportedEpub,
                    };
                    loading.message = loading.stage.label().into();
                    ReaderTickOutcome::LoadingStageChanged
                }
                ReaderLoadingStage::DetectingEncoding => {
                    match detect_txt_encoding(&loading.book.path) {
                        Ok(encoding) => {
                            loading.encoding = Some(encoding);
                            loading.stage = if loading.resume.is_some() {
                                ReaderLoadingStage::LoadingSavedPosition
                            } else {
                                ReaderLoadingStage::BuildingFirstPage
                            };
                            loading.message = format!("{} detected", encoding.label());
                            ReaderTickOutcome::LoadingStageChanged
                        }
                        Err(error) => {
                            loading.stage = ReaderLoadingStage::Failed;
                            loading.message = error;
                            ReaderTickOutcome::Failed
                        }
                    }
                }
                ReaderLoadingStage::LoadingSavedPosition => {
                    loading.stage = ReaderLoadingStage::BuildingFirstPage;
                    loading.message = "Resume anchor ready".into();
                    ReaderTickOutcome::LoadingStageChanged
                }
                ReaderLoadingStage::UpdatingLayout => {
                    loading.stage = ReaderLoadingStage::BuildingFirstPage;
                    loading.message = "Layout cache update ready".into();
                    ReaderTickOutcome::LoadingStageChanged
                }
                ReaderLoadingStage::BuildingFirstPage => {
                    let encoding = loading.encoding.unwrap_or(TextEncoding::Utf8);
                    match self.open_txt_session(&loading.book, encoding, loading.resume.as_ref()) {
                        Ok(session) => {
                            self.session = Some(session);
                            self.last_message =
                                Some("Saved position ready; caching continues lazily".into());
                            self.persist_current_session_best_effort();
                            ReaderTickOutcome::FirstPageReady
                        }
                        Err(error) => {
                            loading.stage = ReaderLoadingStage::Failed;
                            loading.message = error;
                            ReaderTickOutcome::Failed
                        }
                    }
                }
                ReaderLoadingStage::UnsupportedEpub | ReaderLoadingStage::Failed => {
                    self.loading = Some(loading);
                    return ReaderTickOutcome::None;
                }
                ReaderLoadingStage::IndexingNearbyPages | ReaderLoadingStage::Ready => {
                    ReaderTickOutcome::None
                }
            };
            if !matches!(outcome, ReaderTickOutcome::FirstPageReady) {
                self.loading = Some(loading);
            }
            return outcome;
        }

        let (outcome, checkpoint) = if let Some(session) = self.session.as_mut() {
            if session.cache.len() < READER_NEARBY_PAGE_CACHE && !session.index_complete {
                match session.index_one_page() {
                    Ok(true) => (
                        ReaderTickOutcome::BackgroundCacheAdvanced,
                        session.page_offsets.len() % READER_CACHE_CHECKPOINT_PAGES == 0
                            || session.index_complete,
                    ),
                    Ok(false) => (ReaderTickOutcome::None, session.index_complete),
                    Err(error) => {
                        self.last_message = Some(error);
                        return ReaderTickOutcome::Failed;
                    }
                }
            } else {
                (ReaderTickOutcome::None, false)
            }
        } else {
            (ReaderTickOutcome::None, false)
        };
        if checkpoint {
            self.persist_anchor_cache_best_effort();
        }
        outcome
    }

    pub fn previous_page(&mut self) {
        if let Some(session) = self.session.as_mut() {
            if let Err(error) = session.previous_page() {
                self.last_message = Some(error);
                return;
            }
            self.persist_current_session_best_effort();
        }
    }

    pub fn next_page(&mut self) {
        if let Some(session) = self.session.as_mut() {
            if let Err(error) = session.next_page() {
                self.last_message = Some(error);
                return;
            }
            self.persist_current_session_best_effort();
        }
    }

    pub fn cycle_option_previous(&mut self) {
        self.options_selected = self
            .options_selected
            .checked_sub(1)
            .unwrap_or(ReaderOption::ALL.len() - 1);
    }

    pub fn cycle_option_next(&mut self) {
        self.options_selected = (self.options_selected + 1) % ReaderOption::ALL.len();
    }

    #[must_use]
    pub fn selected_option(&self) -> ReaderOption {
        ReaderOption::ALL[self.options_selected]
    }

    /// Resolve a bookmark's user-facing page label against the active layout
    /// when nearby anchors are available. The persisted byte offset remains the
    /// canonical bookmark authority; the stored page index is a safe fallback.
    #[must_use]
    pub fn bookmark_display_page(&self, bookmark: &ReaderLocation) -> usize {
        self.session
            .as_ref()
            .filter(|session| bookmark.matches_book(&session.book))
            .and_then(|session| {
                session
                    .page_offsets
                    .iter()
                    .enumerate()
                    .rev()
                    .find(|(_, offset)| **offset <= bookmark.byte_offset)
                    .map(|(index, _)| {
                        session
                            .page_number_base
                            .saturating_add(index)
                            .saturating_add(1)
                    })
            })
            .unwrap_or_else(|| bookmark.page_index.saturating_add(1))
    }

    #[must_use]
    pub fn current_page_is_bookmarked(&self) -> bool {
        let Some(location) = self.session.as_ref().map(ReaderSession::current_location) else {
            return false;
        };
        self.bookmarks
            .iter()
            .any(|bookmark| bookmark.same_position(&location))
    }

    pub fn toggle_current_bookmark(&mut self) {
        let Some(location) = self.session.as_ref().map(ReaderSession::current_location) else {
            self.last_message = Some("Open a TXT page before adding a bookmark".into());
            return;
        };
        if let Some(index) = self
            .bookmarks
            .iter()
            .position(|bookmark| bookmark.same_position(&location))
        {
            self.bookmarks.remove(index);
            self.bookmarks_selected = self
                .bookmarks_selected
                .min(self.bookmarks.len().saturating_sub(1));
            self.last_message = Some("Bookmark removed".into());
        } else {
            self.bookmarks.insert(0, location);
            self.bookmarks.truncate(READER_BOOKMARK_LIMIT);
            self.bookmarks_selected = 0;
            self.last_message = Some("Bookmark saved".into());
        }
        self.persist_bookmarks_best_effort();
    }

    pub fn begin_preferences_edit(&mut self) {
        self.preferences_selected = 0;
        self.preferences_layout_dirty = false;
    }

    pub fn cycle_preference_previous(&mut self) {
        self.preferences_selected = self
            .preferences_selected
            .checked_sub(1)
            .unwrap_or(ReadingPreference::ALL.len() - 1);
    }

    pub fn cycle_preference_next(&mut self) {
        self.preferences_selected = (self.preferences_selected + 1) % ReadingPreference::ALL.len();
    }

    #[must_use]
    pub fn selected_preference(&self) -> ReadingPreference {
        ReadingPreference::ALL[self.preferences_selected]
    }

    /// Apply one Settings-style SELECT action to the highlighted preference.
    /// Redraw-only settings persist immediately in place. Layout-sensitive
    /// settings persist immediately and request a staged current-page rebuild.
    #[must_use]
    pub fn activate_selected_preference(&mut self) -> bool {
        let layout_sensitive = match self.selected_preference() {
            ReadingPreference::ReadingTheme => {
                self.preferences.theme = self.preferences.theme.next();
                self.last_message =
                    Some(format!("Reading theme: {}", self.preferences.theme.label()));
                self.persist_preferences_best_effort();
                self.request_clear_ghosting();
                false
            }
            ReadingPreference::Orientation => {
                self.preferences.orientation = self.preferences.orientation.next();
                self.last_message = Some(format!(
                    "Orientation: {}",
                    self.preferences.orientation.label()
                ));
                true
            }
            ReadingPreference::BookFontSize => {
                self.preferences.font_size = self.preferences.font_size.next();
                self.last_message = Some(format!(
                    "Book font size: {}",
                    self.preferences.font_size.label()
                ));
                true
            }
            ReadingPreference::BookFont => {
                self.preferences.book_font = self.preferences.book_font.next();
                self.last_message =
                    Some(format!("Book font: {}", self.preferences.book_font.label()));
                true
            }
            ReadingPreference::ParagraphAlignment => {
                self.preferences.paragraph_alignment = self.preferences.paragraph_alignment.next();
                self.last_message = Some(format!(
                    "Paragraph alignment: {}",
                    self.preferences.paragraph_alignment.label()
                ));
                true
            }
            ReadingPreference::ShowProgress => {
                self.preferences.show_progress = !self.preferences.show_progress;
                self.last_message = Some(format!(
                    "Show progress: {}",
                    if self.preferences.show_progress {
                        "On"
                    } else {
                        "Off"
                    }
                ));
                self.persist_preferences_best_effort();
                false
            }
        };
        if layout_sensitive {
            self.request_layout_rebuild()
        } else {
            false
        }
    }

    /// Finish the Settings-style editor. SELECT already persists changes and
    /// launches any required staged rebuild, so BOOT simply returns to options.
    pub fn finish_preferences_edit(&mut self) -> bool {
        self.preferences_layout_dirty = false;
        false
    }

    pub fn cycle_reading_theme(&mut self) {
        self.preferences.theme = self.preferences.theme.next();
        self.last_message = Some(format!("Reading theme: {}", self.preferences.theme.label()));
        self.persist_preferences_best_effort();
        self.request_clear_ghosting();
    }

    pub fn cycle_orientation(&mut self) -> bool {
        self.preferences.orientation = self.preferences.orientation.next();
        self.last_message = Some(format!(
            "Orientation: {}",
            self.preferences.orientation.label()
        ));
        self.request_layout_rebuild()
    }

    pub fn cycle_book_font_size(&mut self) -> bool {
        self.preferences.font_size = self.preferences.font_size.next();
        self.last_message = Some(format!(
            "Book font size: {}",
            self.preferences.font_size.label()
        ));
        self.request_layout_rebuild()
    }

    pub fn cycle_book_font(&mut self) -> bool {
        self.preferences.book_font = self.preferences.book_font.next();
        self.last_message = Some(format!("Book font: {}", self.preferences.book_font.label()));
        self.request_layout_rebuild()
    }

    pub fn toggle_show_progress(&mut self) {
        self.preferences.show_progress = !self.preferences.show_progress;
        self.last_message = Some(format!(
            "Show progress: {}",
            if self.preferences.show_progress {
                "On"
            } else {
                "Off"
            }
        ));
        self.persist_preferences_best_effort();
    }

    pub fn request_clear_ghosting(&mut self) {
        self.clear_ghost_requested = true;
        self.last_message = Some("Global ghost-clearing refresh requested".into());
    }

    #[must_use]
    pub fn take_clear_ghost_request(&mut self) -> bool {
        core::mem::take(&mut self.clear_ghost_requested)
    }

    #[must_use]
    pub fn take_persistence_event(&mut self) -> Option<String> {
        self.persistence_event.take()
    }

    #[must_use]
    fn state_path(&self) -> PathBuf {
        Path::new(&self.state_root).join(READER_STATE_FILE)
    }

    #[must_use]
    fn positions_path(&self) -> PathBuf {
        Path::new(&self.state_root).join(READER_POSITIONS_FILE)
    }

    #[must_use]
    fn legacy_positions_path(&self) -> PathBuf {
        Path::new(&self.state_root).join(LEGACY_READER_POSITIONS_FILE)
    }

    fn load_positions_with_legacy_migration(&mut self) -> Result<Vec<ReaderLocation>, String> {
        let positions = self.positions_path();
        let positions_backup = with_extension(&positions, "BAK");
        if positions.exists() || positions_backup.exists() {
            return load_location_list(&positions, READER_POSITION_LIMIT);
        }

        let legacy = self.legacy_positions_path();
        let legacy_backup = with_extension(&legacy, "BAK");
        if !legacy.exists() && !legacy_backup.exists() {
            return Ok(Vec::new());
        }

        let migrated = load_location_list(&legacy, READER_POSITION_LIMIT)?;
        if !migrated.is_empty() {
            if let Err(error) = atomic_replace_text(&positions, &serialize_location_list(&migrated))
            {
                self.persistence_warning = Some(format!(
                    "legacy POSITIONS.TXT loaded; POSITS.TXT migration deferred: {error}"
                ));
            }
        }
        Ok(migrated)
    }

    #[must_use]
    fn recent_path(&self) -> PathBuf {
        Path::new(&self.state_root).join(READER_RECENT_FILE)
    }

    #[must_use]
    fn bookmarks_path(&self) -> PathBuf {
        Path::new(&self.state_root).join(READER_BOOKMARKS_FILE)
    }

    #[must_use]
    fn preferences_path(&self) -> PathBuf {
        Path::new(&self.state_root).join(READER_PREFS_FILE)
    }

    #[must_use]
    fn cache_directory(&self) -> PathBuf {
        Path::new(&self.state_root).join(READER_CACHE_DIRECTORY)
    }

    #[must_use]
    fn cache_file_name_for(book: &ReaderBook, layout: ReaderLayout) -> String {
        format!("{:08X}.CCH", book_fingerprint(book, layout) as u32)
    }

    #[must_use]
    fn cache_path_for(&self, book: &ReaderBook, layout: ReaderLayout) -> PathBuf {
        self.cache_directory()
            .join(Self::cache_file_name_for(book, layout))
    }

    fn open_txt_session(
        &mut self,
        book: &ReaderBook,
        encoding: TextEncoding,
        requested: Option<&ReaderLocation>,
    ) -> Result<ReaderSession, String> {
        let cached = match load_anchor_cache(
            &self.cache_path_for(book, self.preferences.layout()),
            book,
            self.preferences.layout(),
        ) {
            Ok(value) => value,
            Err(error) => {
                self.persistence_warning = Some(format!("TXT cache ignored: {error}"));
                None
            }
        };
        let (page_number_base, page_offsets, current_page, indexed_through, index_complete) =
            if let Some(cache) = cached {
                let selected = requested
                    .filter(|location| location.matches_book(book))
                    .and_then(|location| {
                        location
                            .page_index
                            .checked_sub(cache.base_page)
                            .filter(|index| *index < cache.offsets.len())
                    })
                    .unwrap_or(0);
                (
                    cache.base_page,
                    cache.offsets,
                    selected,
                    cache.indexed_through,
                    cache.complete,
                )
            } else if let Some(location) = requested.filter(|location| location.matches_book(book))
            {
                (
                    location.page_index,
                    vec![location.byte_offset.min(book.size_bytes)],
                    0,
                    location.byte_offset.min(book.size_bytes),
                    false,
                )
            } else {
                (0, vec![0], 0, 0, false)
            };
        let offset = page_offsets.get(current_page).copied().unwrap_or(0);
        let absolute_page = page_number_base.saturating_add(current_page);
        let layout = self.preferences.layout();
        let page = read_txt_page(book, encoding, layout, offset, absolute_page)?;
        let indexed_through = indexed_through.max(page.next_byte_offset);
        let index_complete = index_complete || indexed_through >= book.size_bytes;
        Ok(ReaderSession {
            book: book.clone(),
            encoding,
            layout,
            current_page,
            page_number_base,
            page_offsets,
            indexed_through,
            index_complete,
            cache: vec![page],
        })
    }

    fn persist_current_session_best_effort(&mut self) {
        let Some(location) = self.session.as_ref().map(ReaderSession::current_location) else {
            return;
        };
        self.resume = Some(location.clone());
        self.positions.retain(|entry| entry.path != location.path);
        self.positions.insert(0, location.clone());
        self.positions.truncate(READER_POSITION_LIMIT);
        self.recent.retain(|entry| entry.path != location.path);
        self.recent.insert(0, location);
        self.recent.truncate(READER_RECENT_LIMIT);
        let mut errors = Vec::new();
        if let Some(location) = self.resume.as_ref() {
            if let Err(error) =
                atomic_replace_text(&self.state_path(), &serialize_location(location))
            {
                errors.push(format!("STATE.TXT: {error}"));
            }
        }
        if let Err(error) = atomic_replace_text(
            &self.positions_path(),
            &serialize_location_list(&self.positions),
        ) {
            errors.push(format!("POSITS.TXT: {error}"));
        }
        if let Err(error) =
            atomic_replace_text(&self.recent_path(), &serialize_location_list(&self.recent))
        {
            errors.push(format!("RECENT.TXT: {error}"));
        }
        if let Err(error) = self.persist_anchor_cache() {
            errors.push(format!("CACHE: {error}"));
        }
        self.finish_persistence("state-positions-recent-cache", errors);
    }

    fn persist_bookmarks_best_effort(&mut self) {
        let mut errors = Vec::new();
        if let Err(error) = atomic_replace_text(
            &self.bookmarks_path(),
            &serialize_location_list(&self.bookmarks),
        ) {
            errors.push(format!("MARKS.TXT: {error}"));
        }
        self.finish_persistence("bookmarks", errors);
    }

    fn persist_anchor_cache_best_effort(&mut self) {
        let mut errors = Vec::new();
        if let Err(error) = self.persist_anchor_cache() {
            errors.push(format!("CACHE: {error}"));
        }
        self.finish_persistence("anchor-cache", errors);
    }

    fn persist_anchor_cache(&self) -> Result<(), String> {
        let Some(session) = self.session.as_ref() else {
            return Ok(());
        };
        let cache = session.anchor_cache();
        atomic_replace_text(
            &self.cache_path_for(&session.book, session.layout),
            &serialize_anchor_cache(&cache),
        )
    }

    fn persist_preferences_best_effort(&mut self) {
        let mut errors = Vec::new();
        if let Err(error) =
            atomic_replace_text(&self.preferences_path(), &self.preferences.serialized())
        {
            errors.push(format!("PREFS.TXT: {error}"));
        }
        self.finish_persistence("preferences", errors);
    }

    fn finish_persistence(&mut self, scope: &str, errors: Vec<String>) {
        let event = if errors.is_empty() {
            format!("status=saved scope={scope}")
        } else {
            let warning = errors.join("; ");
            self.persistence_warning = Some(warning.clone());
            format!("status=degraded scope={scope} error={warning}")
        };
        if self.last_persistence_event.as_deref() != Some(event.as_str()) {
            self.last_persistence_event = Some(event.clone());
            self.persistence_event = Some(event);
        }
    }
}

/// Scan one bounded Reader library. TXT is enabled now; EPUB/EPU rows are
/// exposed as clean future placeholders so the library architecture does not
/// need to change in v0.17.0.
pub fn scan_txt_library(root: impl AsRef<Path>) -> Result<Vec<ReaderBook>, String> {
    let root = root.as_ref();
    let mut books = Vec::new();
    let entries =
        fs::read_dir(root).map_err(|error| format!("Books folder unavailable: {error}"))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(format) = book_format_from_path(&path) else {
            continue;
        };
        let metadata = entry.metadata().ok();
        let size_bytes = metadata.as_ref().map_or(0, |meta| meta.len());
        let modified_seconds = metadata
            .and_then(|meta| meta.modified().ok())
            .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |duration| duration.as_secs());
        let title = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("Untitled book")
            .to_string();
        books.push(ReaderBook {
            path: path.to_string_lossy().into_owned(),
            title,
            format,
            size_bytes,
            modified_seconds,
        });
        if books.len() >= READER_LIBRARY_LIMIT {
            break;
        }
    }
    books.sort_by(|left, right| left.title.to_lowercase().cmp(&right.title.to_lowercase()));
    Ok(books)
}

#[must_use]
pub fn book_format_from_path(path: &Path) -> Option<BookFormat> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "txt" => Some(BookFormat::Text),
        "epub" | "epu" => Some(BookFormat::Epub),
        _ => None,
    }
}

pub fn detect_txt_encoding(path: impl AsRef<Path>) -> Result<TextEncoding, String> {
    let mut file = File::open(path.as_ref()).map_err(|error| format!("Open failed: {error}"))?;
    let mut sample = vec![0_u8; 4096];
    let read = file
        .read(&mut sample)
        .map_err(|error| format!("Read failed: {error}"))?;
    sample.truncate(read);
    if sample.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return Ok(TextEncoding::Utf8Bom);
    }
    match std::str::from_utf8(&sample) {
        Ok(_) => Ok(TextEncoding::Utf8),
        Err(error) if error.error_len().is_none() => Ok(TextEncoding::Utf8),
        Err(_) => Ok(TextEncoding::Windows1252),
    }
}

fn read_txt_page(
    book: &ReaderBook,
    encoding: TextEncoding,
    layout: ReaderLayout,
    byte_offset: u64,
    page_index: usize,
) -> Result<ReaderCachedPage, String> {
    let mut file = File::open(&book.path).map_err(|error| format!("Open failed: {error}"))?;
    file.seek(SeekFrom::Start(byte_offset))
        .map_err(|error| format!("Seek failed: {error}"))?;
    let mut bytes = vec![0_u8; READER_PAGE_READ_BYTES];
    let read = file
        .read(&mut bytes)
        .map_err(|error| format!("Read failed: {error}"))?;
    bytes.truncate(read);
    let skip_bom = byte_offset == 0 && bytes.starts_with(&[0xEF, 0xBB, 0xBF]);
    let base = byte_offset + if skip_bom { 3 } else { 0 };
    let decoded = decode_with_offsets(&bytes[if skip_bom { 3 } else { 0 }..], encoding, base);
    let normalized = normalize_decoded(&decoded);
    let (lines, consumed) = paginate_decoded(&normalized, layout);
    let next_byte_offset = consumed.max(base).min(book.size_bytes);
    Ok(ReaderCachedPage {
        page_index,
        byte_offset,
        next_byte_offset,
        lines,
    })
}

fn decode_with_offsets(bytes: &[u8], encoding: TextEncoding, base: u64) -> Vec<(char, u64)> {
    match encoding {
        TextEncoding::Windows1252 => bytes
            .iter()
            .enumerate()
            .map(|(index, byte)| (decode_windows_1252(*byte), base + index as u64 + 1))
            .collect(),
        TextEncoding::Utf8 | TextEncoding::Utf8Bom => {
            let valid = match std::str::from_utf8(bytes) {
                Ok(text) => text,
                Err(error) => std::str::from_utf8(&bytes[..error.valid_up_to()]).unwrap_or(""),
            };
            valid
                .char_indices()
                .map(|(index, character)| {
                    (character, base + index as u64 + character.len_utf8() as u64)
                })
                .collect()
        }
    }
}

fn normalize_decoded(decoded: &[(char, u64)]) -> Vec<(char, u64)> {
    let mut normalized = Vec::new();
    for (index, (character, next_offset)) in decoded.iter().copied().enumerate() {
        if character == '_' {
            let previous = index
                .checked_sub(1)
                .and_then(|value| decoded.get(value))
                .map(|value| value.0);
            let next = decoded.get(index + 1).map(|value| value.0);
            let word_internal =
                previous.is_some_and(is_word_character) && next.is_some_and(is_word_character);
            let repeated_separator = previous == Some('_') || next == Some('_');

            // Project Gutenberg TXT files often wrap emphasis across multiple
            // source lines: `_first line ... last line_`. Remove each bounded
            // delimiter independently so closing markers after punctuation do
            // not leak into rendered pages. Keep filename-style word_internal
            // underscores and repeated separator rows intact.
            if !word_internal && !repeated_separator {
                continue;
            }
        }
        push_normalized_character(&mut normalized, character, next_offset);
    }
    normalized
}

fn push_normalized_character(output: &mut Vec<(char, u64)>, character: char, next_offset: u64) {
    let replacement: &str = match character {
        '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{00AB}' | '\u{00BB}' => "\"",
        '\u{2018}' | '\u{2019}' | '\u{201A}' => "'",
        '\u{2014}' => "--",
        '\u{2013}' => "-",
        '\u{2026}' => "...",
        '\u{00A0}' => " ",
        'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => "e",
        'à' | 'á' | 'â' | 'ä' | 'À' | 'Á' | 'Â' | 'Ä' => "a",
        'ç' | 'Ç' => "c",
        'ï' | 'î' | 'í' | 'ì' | 'Ï' | 'Î' | 'Í' | 'Ì' => "i",
        'ô' | 'ö' | 'ó' | 'ò' | 'Ô' | 'Ö' | 'Ó' | 'Ò' => "o",
        'ù' | 'û' | 'ü' | 'ú' | 'Ù' | 'Û' | 'Ü' | 'Ú' => "u",
        'ñ' | 'Ñ' => "n",
        value
            if value == '\n'
                || value == '\r'
                || value == '\t'
                || value.is_ascii_graphic()
                || value == ' ' =>
        {
            output.push((value, next_offset));
            return;
        }
        _ => "?",
    };
    for value in replacement.chars() {
        output.push((value, next_offset));
    }
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric()
}

fn paginate_decoded(decoded: &[(char, u64)], layout: ReaderLayout) -> (Vec<ReaderPageLine>, u64) {
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut consumed = decoded
        .first()
        .map_or(0, |(_, offset)| offset.saturating_sub(1));
    for (character, next_offset) in decoded.iter().copied() {
        let character = match character {
            '\r' => continue,
            '\n' => {
                lines.push(ReaderPageLine {
                    text: core::mem::take(&mut line),
                    paragraph_end: true,
                });
                consumed = next_offset;
                if lines.len() >= layout.lines_per_page {
                    break;
                }
                continue;
            }
            value if value.is_control() => ' ',
            value => value,
        };
        if line.chars().count() >= layout.chars_per_line {
            lines.push(ReaderPageLine {
                text: core::mem::take(&mut line),
                paragraph_end: false,
            });
            if lines.len() >= layout.lines_per_page {
                break;
            }
        }
        if character.is_whitespace() {
            if !line.is_empty() && !line.ends_with(' ') {
                line.push(' ');
            }
        } else {
            line.push(character);
        }
        consumed = next_offset;
    }
    if lines.len() < layout.lines_per_page && (!line.is_empty() || lines.is_empty()) {
        lines.push(ReaderPageLine {
            text: line,
            paragraph_end: true,
        });
    }
    (lines, consumed)
}

fn decode_windows_1252(byte: u8) -> char {
    match byte {
        0x80 => '€',
        0x82 => '‚',
        0x83 => 'ƒ',
        0x84 => '„',
        0x85 => '…',
        0x86 => '†',
        0x87 => '‡',
        0x88 => 'ˆ',
        0x89 => '‰',
        0x8A => 'Š',
        0x8B => '‹',
        0x8C => 'Œ',
        0x8E => 'Ž',
        0x91 => '‘',
        0x92 => '’',
        0x93 => '“',
        0x94 => '”',
        0x95 => '•',
        0x96 => '–',
        0x97 => '—',
        0x98 => '˜',
        0x99 => '™',
        0x9A => 'š',
        0x9B => '›',
        0x9C => 'œ',
        0x9E => 'ž',
        0x9F => 'Ÿ',
        value => char::from(value),
    }
}

fn book_fingerprint(book: &ReaderBook, layout: ReaderLayout) -> u64 {
    let mut hash = CACHE_FNV_OFFSET;
    fn feed(hash: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *hash ^= u64::from(*byte);
            *hash = hash.wrapping_mul(CACHE_FNV_PRIME);
        }
    }
    feed(&mut hash, book.path.as_bytes());
    feed(&mut hash, &book.size_bytes.to_le_bytes());
    feed(&mut hash, &book.modified_seconds.to_le_bytes());
    feed(&mut hash, book.format.marker().as_bytes());
    feed(&mut hash, &layout.lines_per_page.to_le_bytes());
    feed(&mut hash, &layout.chars_per_line.to_le_bytes());
    feed(&mut hash, layout.orientation.marker().as_bytes());
    feed(&mut hash, layout.font_size.marker().as_bytes());
    feed(&mut hash, layout.book_font.marker().as_bytes());
    feed(&mut hash, layout.paragraph_alignment.marker().as_bytes());
    feed(&mut hash, READER_CACHE_VERSION.as_bytes());
    hash
}

fn serialize_location(location: &ReaderLocation) -> String {
    format!(
        "version={}\npath={}\ntitle={}\nformat={}\nsize={}\nmodified={}\npage={}\noffset={}\n",
        READER_PERSISTENCE_VERSION,
        escape_field(&location.path),
        escape_field(&location.title),
        location.format.marker(),
        location.size_bytes,
        location.modified_seconds,
        location.page_index,
        location.byte_offset
    )
}

fn parse_location_record(text: &str) -> Result<ReaderLocation, String> {
    let mut version = None;
    let mut path = None;
    let mut title = None;
    let mut format = None;
    let mut size = None;
    let mut modified = None;
    let mut page = None;
    let mut offset = None;
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "version" => version = Some(value),
            "path" => path = Some(unescape_field(value)?),
            "title" => title = Some(unescape_field(value)?),
            "format" => format = BookFormat::parse(value),
            "size" => size = value.parse().ok(),
            "modified" => modified = value.parse().ok(),
            "page" => page = value.parse().ok(),
            "offset" => offset = value.parse().ok(),
            _ => {}
        }
    }
    if version != Some(READER_PERSISTENCE_VERSION) {
        return Err("unsupported persistence version".into());
    }
    Ok(ReaderLocation {
        path: path.ok_or_else(|| "missing path".to_string())?,
        title: title.ok_or_else(|| "missing title".to_string())?,
        format: format.ok_or_else(|| "missing format".to_string())?,
        size_bytes: size.ok_or_else(|| "missing size".to_string())?,
        modified_seconds: modified.unwrap_or(0),
        page_index: page.ok_or_else(|| "missing page".to_string())?,
        byte_offset: offset.ok_or_else(|| "missing offset".to_string())?,
    })
}

fn serialize_location_list(locations: &[ReaderLocation]) -> String {
    let mut output = format!("version={}\n", READER_PERSISTENCE_VERSION);
    for location in locations {
        output.push_str("entry=");
        output.push_str(&serialize_location_fields(location));
        output.push('\n');
    }
    output
}

fn serialize_location_fields(location: &ReaderLocation) -> String {
    [
        escape_field(&location.path),
        escape_field(&location.title),
        location.format.marker().into(),
        location.size_bytes.to_string(),
        location.modified_seconds.to_string(),
        location.page_index.to_string(),
        location.byte_offset.to_string(),
    ]
    .join("\t")
}

fn parse_location_fields(value: &str) -> Result<ReaderLocation, String> {
    let fields = split_escaped_tabs(value)?;
    if fields.len() != 7 {
        return Err("invalid location field count".into());
    }
    Ok(ReaderLocation {
        path: fields[0].clone(),
        title: fields[1].clone(),
        format: BookFormat::parse(&fields[2]).ok_or_else(|| "invalid format".to_string())?,
        size_bytes: fields[3].parse().map_err(|_| "invalid size".to_string())?,
        modified_seconds: fields[4]
            .parse()
            .map_err(|_| "invalid modified time".to_string())?,
        page_index: fields[5].parse().map_err(|_| "invalid page".to_string())?,
        byte_offset: fields[6]
            .parse()
            .map_err(|_| "invalid offset".to_string())?,
    })
}

fn parse_location_list(text: &str, limit: usize) -> Result<Vec<ReaderLocation>, String> {
    let mut version = None;
    let mut output = Vec::new();
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("version=") {
            version = Some(value);
        } else if let Some(value) = line.strip_prefix("entry=") {
            if output.len() < limit {
                output.push(parse_location_fields(value)?);
            }
        }
    }
    if version != Some(READER_PERSISTENCE_VERSION) {
        return Err("unsupported persistence version".into());
    }
    Ok(output)
}

fn serialize_anchor_cache(cache: &ReaderAnchorCache) -> String {
    let mut output = format!(
        "version={}\nfingerprint={:016X}\nbase_page={}\nindexed_through={}\ncomplete={}\n",
        READER_CACHE_VERSION,
        cache.fingerprint,
        cache.base_page,
        cache.indexed_through,
        cache.complete
    );
    for offset in cache.offsets.iter().take(READER_CACHE_OFFSET_LIMIT) {
        output.push_str(&format!("offset={offset}\n"));
    }
    output
}

fn parse_anchor_cache(
    text: &str,
    book: &ReaderBook,
    layout: ReaderLayout,
) -> Result<ReaderAnchorCache, String> {
    let mut version = None;
    let mut fingerprint = None;
    let mut base_page = None;
    let mut indexed_through: Option<u64> = None;
    let mut complete = None;
    let mut offsets = Vec::new();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "version" => version = Some(value),
            "fingerprint" => fingerprint = u64::from_str_radix(value, 16).ok(),
            "base_page" => base_page = value.parse().ok(),
            "indexed_through" => indexed_through = value.parse().ok(),
            "complete" => complete = value.parse().ok(),
            "offset" if offsets.len() < READER_CACHE_OFFSET_LIMIT => {
                offsets.push(
                    value
                        .parse()
                        .map_err(|_| "invalid cache offset".to_string())?,
                );
            }
            _ => {}
        }
    }
    if version != Some(READER_CACHE_VERSION) {
        return Err("unsupported cache version".into());
    }
    let fingerprint = fingerprint.ok_or_else(|| "missing cache fingerprint".to_string())?;
    if fingerprint != book_fingerprint(book, layout) {
        return Err("cache fingerprint mismatch".into());
    }
    if offsets.is_empty() {
        return Err("cache contains no offsets".into());
    }
    if offsets.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err("cache offsets are not strictly increasing".into());
    }
    if offsets.iter().any(|offset| *offset > book.size_bytes) {
        return Err("cache offset exceeds book size".into());
    }
    Ok(ReaderAnchorCache {
        fingerprint,
        base_page: base_page.ok_or_else(|| "missing base page".to_string())?,
        offsets,
        indexed_through: indexed_through
            .ok_or_else(|| "missing indexed offset".to_string())?
            .min(book.size_bytes),
        complete: complete.ok_or_else(|| "missing complete flag".to_string())?,
    })
}

fn load_preferences(path: &Path) -> Result<Option<ReaderPreferences>, String> {
    load_with_backup(path, ReaderPreferences::parse)
}

fn load_location_record(path: &Path) -> Result<Option<ReaderLocation>, String> {
    load_with_backup(path, parse_location_record)
}

fn load_location_list(path: &Path, limit: usize) -> Result<Vec<ReaderLocation>, String> {
    load_with_backup(path, |text| parse_location_list(text, limit))
        .map(|value| value.unwrap_or_default())
}

fn load_anchor_cache(
    path: &Path,
    book: &ReaderBook,
    layout: ReaderLayout,
) -> Result<Option<ReaderAnchorCache>, String> {
    load_with_backup(path, |text| parse_anchor_cache(text, book, layout))
}

fn load_with_backup<T>(
    path: &Path,
    parser: impl Fn(&str) -> Result<T, String>,
) -> Result<Option<T>, String> {
    let backup = with_extension(path, "BAK");
    let mut errors = Vec::new();
    for candidate in [path.to_path_buf(), backup] {
        if !candidate.exists() {
            continue;
        }
        match fs::read_to_string(&candidate) {
            Ok(text) => match parser(&text) {
                Ok(value) => return Ok(Some(value)),
                Err(error) => errors.push(format!("{}: {error}", candidate.display())),
            },
            Err(error) => errors.push(format!("{}: {error}", candidate.display())),
        }
    }
    if errors.is_empty() {
        Ok(None)
    } else {
        Err(errors.join("; "))
    }
}

fn is_fat83_safe_file_name(path: &Path) -> bool {
    let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    let Some((stem, extension)) = file_name.rsplit_once('.') else {
        return false;
    };
    !stem.is_empty()
        && stem.len() <= 8
        && !extension.is_empty()
        && extension.len() <= 3
        && stem
            .bytes()
            .chain(extension.bytes())
            .all(|value| value.is_ascii_alphanumeric() || value == b'_')
}

/// Power-safe bounded text replacement for Reader-owned state. The previous
/// primary is retained as .BAK until the new .TMP file has been renamed into
/// place. Readers accept the backup if startup observes an interrupted write.
fn atomic_replace_text(path: &Path, text: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "state path has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    let temp = with_extension(path, "TMP");
    let backup = with_extension(path, "BAK");
    for candidate in [path, temp.as_path(), backup.as_path()] {
        if !is_fat83_safe_file_name(candidate) {
            return Err(format!(
                "Reader state filename is not FAT 8.3 safe: {}",
                candidate.display()
            ));
        }
    }
    let _ = fs::remove_file(&temp);
    let _ = fs::remove_file(&backup);
    {
        let mut file =
            File::create(&temp).map_err(|error| format!("create {}: {error}", temp.display()))?;
        file.write_all(text.as_bytes())
            .map_err(|error| format!("write {}: {error}", temp.display()))?;
        file.sync_all()
            .map_err(|error| format!("sync {}: {error}", temp.display()))?;
    }
    if path.exists() {
        fs::rename(path, &backup).map_err(|error| format!("backup {}: {error}", path.display()))?;
    }
    if let Err(error) = fs::rename(&temp, path) {
        if backup.exists() {
            let _ = fs::rename(&backup, path);
        }
        return Err(format!("replace {}: {error}", path.display()));
    }
    let _ = fs::remove_file(&backup);
    Ok(())
}

fn with_extension(path: &Path, extension: &str) -> PathBuf {
    let mut output = path.to_path_buf();
    output.set_extension(extension);
    output
}

fn escape_field(value: &str) -> String {
    let mut output = String::new();
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '\t' => output.push_str("\\t"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            value => output.push(value),
        }
    }
    output
}

fn unescape_field(value: &str) -> Result<String, String> {
    let mut output = String::new();
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            match character {
                '\\' => output.push('\\'),
                't' => output.push('\t'),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                _ => return Err("invalid escape sequence".into()),
            }
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            output.push(character);
        }
    }
    if escaped {
        return Err("trailing escape sequence".into());
    }
    Ok(output)
}

fn split_escaped_tabs(value: &str) -> Result<Vec<String>, String> {
    let mut output = Vec::new();
    let mut current = String::new();
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            current.push('\\');
            current.push(character);
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '\t' {
            output.push(unescape_field(&current)?);
            current.clear();
        } else {
            current.push(character);
        }
    }
    if escaped {
        return Err("trailing escape sequence".into());
    }
    output.push(unescape_field(&current)?);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use super::{
        atomic_replace_text, book_format_from_path, detect_txt_encoding, is_fat83_safe_file_name,
        load_location_record, normalize_decoded, scan_txt_library, BookFont, BookFontSize,
        BookFormat, ParagraphAlignment, ReaderBook, ReaderLoadingStage, ReaderLocation,
        ReaderOrientation, ReaderPreferences, ReaderSession, ReaderTickOutcome, ReaderUiState,
        ReadingPreference, ReadingTheme, TextEncoding, LEGACY_READER_POSITIONS_FILE,
        READER_BOOKMARKS_FILE, READER_POSITIONS_FILE, READER_PREFS_FILE, READER_RECENT_FILE,
        READER_STATE_FILE,
    };
    use crate::buttons::ButtonEvent;

    fn temp_dir(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("rustmix-reader-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn detects_txt_epub_and_short_epu_aliases() {
        assert_eq!(
            book_format_from_path(PathBuf::from("a.TXT").as_path()),
            Some(BookFormat::Text)
        );
        assert_eq!(
            book_format_from_path(PathBuf::from("a.epub").as_path()),
            Some(BookFormat::Epub)
        );
        assert_eq!(
            book_format_from_path(PathBuf::from("a.EPU").as_path()),
            Some(BookFormat::Epub)
        );
    }

    #[test]
    fn detects_utf8_bom_and_windows_1252() {
        let root = temp_dir("encoding");
        let bom = root.join("bom.txt");
        let cp = root.join("cp.txt");
        fs::write(&bom, [0xEF, 0xBB, 0xBF, b'H', b'i']).unwrap();
        fs::write(&cp, [b'H', 0x92, b'i']).unwrap();
        assert_eq!(detect_txt_encoding(&bom).unwrap(), TextEncoding::Utf8Bom);
        assert_eq!(detect_txt_encoding(&cp).unwrap(), TextEncoding::Windows1252);
    }

    #[test]
    fn scans_txt_and_epub_rows_but_ignores_other_files() {
        let root = temp_dir("scan");
        fs::write(root.join("Dracula.txt"), "hello").unwrap();
        fs::write(root.join("Later.epu"), "zip").unwrap();
        fs::write(root.join("ignore.bin"), "no").unwrap();
        let books = scan_txt_library(&root).unwrap();
        assert_eq!(books.len(), 2);
        assert_eq!(books[0].title, "Dracula");
        assert_eq!(books[1].format, BookFormat::Epub);
    }

    #[test]
    fn opening_txt_is_staged_first_page_first_and_lazy() {
        let root = temp_dir("open");
        let state = temp_dir("open-state");
        fs::write(root.join("Book.txt"), "hello world ".repeat(600)).unwrap();
        let mut reader = ReaderUiState::with_roots(
            root.to_string_lossy().into_owned(),
            state.to_string_lossy().into_owned(),
        );
        reader.refresh_library();
        reader.library_selected = 1;
        assert!(reader.apply_library_button(ButtonEvent::Select));
        assert_eq!(reader.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(reader.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(reader.tick(), ReaderTickOutcome::FirstPageReady);
        let session = reader.session.as_ref().unwrap();
        assert_eq!(session.current_page, 0);
        assert!(!session.cache.is_empty());
        assert!(session.indexed_through > 0);
    }

    #[test]
    fn persists_continue_recent_bookmarks_and_anchor_cache() {
        let root = temp_dir("persist-books");
        let state = temp_dir("persist-state");
        fs::write(root.join("Dracula.txt"), "Dracula text ".repeat(1000)).unwrap();
        let mut reader = ReaderUiState::with_roots(
            root.to_string_lossy().into_owned(),
            state.to_string_lossy().into_owned(),
        );
        reader.refresh_library();
        reader.library_selected = 1;
        assert!(reader.apply_library_button(ButtonEvent::Select));
        assert_eq!(reader.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(reader.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(reader.tick(), ReaderTickOutcome::FirstPageReady);
        reader.next_page();
        reader.toggle_current_bookmark();
        assert!(state.join(READER_STATE_FILE).exists());
        assert!(state.join(READER_POSITIONS_FILE).exists());
        assert!(state.join(READER_RECENT_FILE).exists());
        assert!(state.join(READER_BOOKMARKS_FILE).exists());
        assert!(state.join("CACHE").read_dir().unwrap().next().is_some());

        let mut restored = ReaderUiState::with_roots(
            root.to_string_lossy().into_owned(),
            state.to_string_lossy().into_owned(),
        );
        let report = restored.load_persistent_state();
        assert!(report.state_loaded);
        assert_eq!(report.recent_count, 1);
        assert_eq!(report.bookmark_count, 1);
        assert!(restored.request_continue());
        assert_eq!(restored.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(restored.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(restored.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(restored.tick(), ReaderTickOutcome::FirstPageReady);
        assert_eq!(
            restored.session.as_ref().unwrap().current_absolute_page(),
            1
        );
    }

    #[test]
    fn invalid_anchor_cache_fingerprint_falls_back_to_saved_offset() {
        let root = temp_dir("fingerprint-books");
        let state = temp_dir("fingerprint-state");
        fs::write(root.join("Book.txt"), "text body ".repeat(1000)).unwrap();
        let mut reader = ReaderUiState::with_roots(
            root.to_string_lossy().into_owned(),
            state.to_string_lossy().into_owned(),
        );
        reader.refresh_library();
        reader.library_selected = 1;
        assert!(reader.apply_library_button(ButtonEvent::Select));
        assert_eq!(reader.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(reader.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(reader.tick(), ReaderTickOutcome::FirstPageReady);
        reader.next_page();
        let cache = state
            .join("CACHE")
            .read_dir()
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let text = fs::read_to_string(&cache).unwrap();
        fs::write(
            &cache,
            text.replace("fingerprint=", "fingerprint=0000000000000000#"),
        )
        .unwrap();

        let mut restored = ReaderUiState::with_roots(
            root.to_string_lossy().into_owned(),
            state.to_string_lossy().into_owned(),
        );
        restored.load_persistent_state();
        assert!(restored.request_continue());
        assert_eq!(restored.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(restored.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(restored.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(restored.tick(), ReaderTickOutcome::FirstPageReady);
        assert!(restored
            .persistence_warning
            .as_deref()
            .unwrap_or("")
            .contains("TXT cache ignored"));
        assert_eq!(
            restored.session.as_ref().unwrap().current_absolute_page(),
            1
        );
    }

    #[test]
    fn bookmark_toggle_removes_existing_mark() {
        let root = temp_dir("toggle-books");
        let state = temp_dir("toggle-state");
        fs::write(root.join("Book.txt"), "text ".repeat(100)).unwrap();
        let mut reader = ReaderUiState::with_roots(
            root.to_string_lossy().into_owned(),
            state.to_string_lossy().into_owned(),
        );
        reader.refresh_library();
        reader.library_selected = 1;
        assert!(reader.apply_library_button(ButtonEvent::Select));
        assert_eq!(reader.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(reader.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(reader.tick(), ReaderTickOutcome::FirstPageReady);
        reader.toggle_current_bookmark();
        assert_eq!(reader.bookmarks.len(), 1);
        reader.toggle_current_bookmark();
        assert!(reader.bookmarks.is_empty());
    }

    #[test]
    fn interrupted_atomic_replace_recovers_backup() {
        let root = temp_dir("backup");
        let state = root.join(READER_STATE_FILE);
        atomic_replace_text(
            &state,
            "version=1\npath=a.txt\ntitle=A\nformat=txt\nsize=1\nmodified=0\npage=0\noffset=0\n",
        )
        .unwrap();
        let backup = root.join("STATE.BAK");
        fs::rename(&state, &backup).unwrap();
        let restored = load_location_record(&state).unwrap().unwrap();
        assert_eq!(restored.title, "A");
    }

    #[test]
    fn corrupt_primary_falls_back_to_backup() {
        let root = temp_dir("corrupt");
        let state = root.join(READER_STATE_FILE);
        fs::write(&state, "not-valid").unwrap();
        fs::write(
            root.join("STATE.BAK"),
            "version=1\npath=b.txt\ntitle=B\nformat=txt\nsize=2\nmodified=0\npage=3\noffset=4\n",
        )
        .unwrap();
        let restored = load_location_record(&state).unwrap().unwrap();
        assert_eq!(restored.title, "B");
    }

    #[test]
    fn reader_options_request_manual_clear_ghosting() {
        let mut reader = ReaderUiState::default();
        reader.request_clear_ghosting();
        assert!(reader.take_clear_ghost_request());
        assert!(!reader.take_clear_ghost_request());
    }

    #[test]
    fn normalizes_utf8_punctuation_accents_and_simple_emphasis() {
        let decoded: Vec<(char, u64)> = "“En vérité!” _I_—once…"
            .chars()
            .enumerate()
            .map(|(index, value)| (value, index as u64 + 1))
            .collect();
        let normalized: String = normalize_decoded(&decoded)
            .into_iter()
            .map(|(value, _)| value)
            .collect();
        assert_eq!(normalized, "\"En verite!\" I--once...");
    }

    #[test]
    fn removes_multiline_gutenberg_emphasis_but_preserves_safe_underscores() {
        let decoded: Vec<(char, u64)> =
            "'_You have lost your\ngold pencil-case? Couragez!'_ file_name\n_____"
                .chars()
                .enumerate()
                .map(|(index, value)| (value, index as u64 + 1))
                .collect();
        let normalized: String = normalize_decoded(&decoded)
            .into_iter()
            .map(|(value, _)| value)
            .collect();
        assert_eq!(
            normalized,
            "'You have lost your\ngold pencil-case? Couragez!' file_name\n_____"
        );
    }

    #[test]
    fn theme_switch_keeps_layout_geometry_and_cache_fingerprint_inputs_stable() {
        let classic = ReaderPreferences::default();
        let mut contrast = classic;
        contrast.theme = ReadingTheme::HighContrast;
        assert_eq!(classic.layout(), contrast.layout());
    }

    #[test]
    fn parses_serializes_and_cycles_reader_preferences() {
        let parsed = ReaderPreferences::parse(
            "version=1\ntheme=high-contrast\norientation=landscape\nfont_size=xlarge\nbook_font=serif\nparagraph_alignment=right\nshow_progress=false\n",
        )
        .unwrap();
        assert_eq!(parsed.theme, ReadingTheme::HighContrast);
        assert_eq!(parsed.orientation, ReaderOrientation::Landscape);
        assert_eq!(parsed.font_size, BookFontSize::XLarge);
        assert_eq!(parsed.book_font, BookFont::Serif);
        assert_eq!(parsed.paragraph_alignment, ParagraphAlignment::Right);
        assert!(!parsed.show_progress);
        assert!(parsed.serialized().contains("font_size=xlarge"));
        assert!(parsed.serialized().contains("book_font=serif"));
        assert!(parsed.serialized().contains("paragraph_alignment=right"));
    }

    #[test]
    fn layout_changes_request_first_page_first_rebuild_and_persist_preferences() {
        let root = temp_dir("prefs-books");
        let state = temp_dir("prefs-state");
        fs::write(root.join("Book.txt"), "hello world ".repeat(800)).unwrap();
        let mut reader = ReaderUiState::with_roots(
            root.to_string_lossy().into_owned(),
            state.to_string_lossy().into_owned(),
        );
        reader.refresh_library();
        reader.library_selected = 1;
        assert!(reader.apply_library_button(ButtonEvent::Select));
        assert_eq!(reader.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(reader.tick(), ReaderTickOutcome::LoadingStageChanged);
        assert_eq!(reader.tick(), ReaderTickOutcome::FirstPageReady);
        assert!(reader.cycle_book_font_size());
        assert_eq!(
            reader.loading_stage(),
            Some(ReaderLoadingStage::UpdatingLayout)
        );
        assert!(state.join(READER_PREFS_FILE).exists());
    }

    #[test]
    fn books_and_files_reopen_from_per_book_positions_while_bookmarks_remain_explicit() {
        let root = temp_dir("positions-books");
        let state = temp_dir("positions-state");
        fs::write(root.join("A.txt"), "alpha body ".repeat(1200)).unwrap();
        fs::write(root.join("B.txt"), "beta body ".repeat(1200)).unwrap();
        let mut reader = ReaderUiState::with_roots(
            root.to_string_lossy().into_owned(),
            state.to_string_lossy().into_owned(),
        );
        reader.refresh_library();
        reader.library_selected = 1;
        assert!(reader.apply_library_button(ButtonEvent::Select));
        for _ in 0..3 {
            reader.tick();
        }
        reader.next_page();
        reader.next_page();
        let saved = reader.session.as_ref().unwrap().current_location();
        assert_eq!(saved.page_index, 2);

        reader.refresh_library();
        reader.library_selected = 1;
        assert!(reader.apply_library_button(ButtonEvent::Select));
        for _ in 0..4 {
            reader.tick();
        }
        assert_eq!(reader.session.as_ref().unwrap().current_absolute_page(), 2);

        let mut explicit = saved.clone();
        explicit.page_index = 1;
        explicit.byte_offset = reader.session.as_ref().unwrap().page_offsets[1];
        reader.bookmarks = vec![explicit];
        assert!(reader.request_open_bookmark(0));
        for _ in 0..4 {
            reader.tick();
        }
        assert_eq!(reader.session.as_ref().unwrap().current_absolute_page(), 1);
    }

    #[test]
    fn paragraph_alignment_defaults_to_justified_and_changes_cache_fingerprint_inputs() {
        let justified = ReaderPreferences::default();
        assert_eq!(justified.paragraph_alignment, ParagraphAlignment::Justified);
        let mut left = justified;
        left.paragraph_alignment = ParagraphAlignment::Left;
        assert_ne!(justified.layout(), left.layout());
    }

    #[test]
    fn preference_editor_uses_move_then_select_change_policy() {
        let mut reader = ReaderUiState::default();
        reader.begin_preferences_edit();
        assert_eq!(
            reader.selected_preference(),
            ReadingPreference::ReadingTheme
        );
        reader.cycle_preference_next();
        assert_eq!(reader.selected_preference(), ReadingPreference::Orientation);
        reader.cycle_preference_previous();
        assert_eq!(
            reader.selected_preference(),
            ReadingPreference::ReadingTheme
        );
        assert!(!reader.activate_selected_preference());
        assert_eq!(reader.preferences.theme, ReadingTheme::HighContrast);
    }

    #[test]
    fn reader_owned_runtime_filenames_are_fat83_safe() {
        for name in [
            READER_STATE_FILE,
            READER_POSITIONS_FILE,
            READER_RECENT_FILE,
            READER_BOOKMARKS_FILE,
            READER_PREFS_FILE,
            "ED9B69AF.CCH",
            "ED9B69AF.TMP",
            "ED9B69AF.BAK",
        ] {
            assert!(is_fat83_safe_file_name(Path::new(name)), "{name}");
        }
        assert!(!is_fat83_safe_file_name(Path::new(
            LEGACY_READER_POSITIONS_FILE
        )));
        assert!(!is_fat83_safe_file_name(Path::new("BED9B69AF.CCH")));
    }

    #[test]
    fn cache_filename_uses_exactly_eight_hexadecimal_characters() {
        let root = temp_dir("fat83-cache-books");
        let state = temp_dir("fat83-cache-state");
        fs::write(root.join("Book.txt"), "text body ".repeat(1000)).unwrap();
        let mut reader = ReaderUiState::with_roots(
            root.to_string_lossy().into_owned(),
            state.to_string_lossy().into_owned(),
        );
        reader.refresh_library();
        let book = reader.books.first().unwrap();
        let cache = reader.cache_path_for(book, reader.preferences.layout());
        let file = cache.file_name().unwrap().to_str().unwrap();
        assert_eq!(file.len(), 12);
        assert_eq!(&file[8..], ".CCH");
        assert!(file[..8].bytes().all(|value| value.is_ascii_hexdigit()));
        assert!(is_fat83_safe_file_name(&cache));
    }

    #[test]
    fn legacy_positions_file_migrates_to_short_name_safe_primary() {
        let root = temp_dir("legacy-positions-books");
        let state = temp_dir("legacy-positions-state");
        fs::write(root.join("Book.txt"), "text body ".repeat(1000)).unwrap();
        let legacy = state.join(LEGACY_READER_POSITIONS_FILE);
        fs::write(
            &legacy,
            "version=1\nentry=Book.txt\tBook\ttxt\t1000\t0\t3\t42\n",
        )
        .unwrap();
        let mut reader = ReaderUiState::with_roots(
            root.to_string_lossy().into_owned(),
            state.to_string_lossy().into_owned(),
        );
        let report = reader.load_persistent_state();
        assert_eq!(report.position_count, 1);
        assert!(state.join(READER_POSITIONS_FILE).exists());
        assert_eq!(reader.positions[0].byte_offset, 42);
    }

    #[test]
    fn fat83_runtime_primary_temp_and_backup_paths_are_safe_without_cache_prefix() {
        let root = temp_dir("fat83-runtime-books");
        let state = temp_dir("fat83-runtime-state");
        fs::write(root.join("Book.txt"), "text body ".repeat(1000)).unwrap();
        let mut reader = ReaderUiState::with_roots(
            root.to_string_lossy().into_owned(),
            state.to_string_lossy().into_owned(),
        );
        reader.refresh_library();
        let book = reader.books.first().unwrap();
        let positions = reader.positions_path();
        let cache = reader.cache_path_for(book, reader.preferences.layout());
        for path in [
            positions.clone(),
            super::with_extension(&positions, "TMP"),
            super::with_extension(&positions, "BAK"),
            cache.clone(),
            super::with_extension(&cache, "TMP"),
            super::with_extension(&cache, "BAK"),
        ] {
            assert!(is_fat83_safe_file_name(&path), "{}", path.display());
        }
        let cache_file = cache.file_name().unwrap().to_str().unwrap();
        assert_eq!(&cache_file[8..], ".CCH");
        assert!(
            !cache_file.starts_with('B')
                || cache_file[..8]
                    .bytes()
                    .all(|value| value.is_ascii_hexdigit())
        );
        assert_eq!(cache_file[..8].len(), 8);
    }

    #[test]
    fn bookmark_page_label_uses_active_layout_offsets_and_stored_fallback() {
        let book = ReaderBook {
            path: "Book.txt".into(),
            title: "Book".into(),
            format: BookFormat::Text,
            size_bytes: 1000,
            modified_seconds: 0,
        };
        let bookmark = ReaderLocation {
            path: book.path.clone(),
            title: book.title.clone(),
            format: book.format,
            size_bytes: book.size_bytes,
            modified_seconds: book.modified_seconds,
            page_index: 8,
            byte_offset: 220,
        };
        let mut reader = ReaderUiState::default();
        assert_eq!(reader.bookmark_display_page(&bookmark), 9);
        reader.session = Some(ReaderSession {
            book,
            encoding: TextEncoding::Utf8,
            layout: ReaderPreferences::default().layout(),
            current_page: 0,
            page_number_base: 0,
            page_offsets: vec![0, 100, 200, 300],
            indexed_through: 300,
            index_complete: false,
            cache: Vec::new(),
        });
        assert_eq!(reader.bookmark_display_page(&bookmark), 3);
    }

    #[test]
    fn repeated_degraded_persistence_events_are_suppressed_until_status_changes() {
        let mut reader = ReaderUiState::default();
        reader.finish_persistence("anchor-cache", vec!["CACHE: failed".into()]);
        assert!(reader.take_persistence_event().is_some());
        reader.finish_persistence("anchor-cache", vec!["CACHE: failed".into()]);
        assert!(reader.take_persistence_event().is_none());
        reader.finish_persistence("anchor-cache", Vec::new());
        assert_eq!(
            reader.take_persistence_event().as_deref(),
            Some("status=saved scope=anchor-cache")
        );
    }
}
