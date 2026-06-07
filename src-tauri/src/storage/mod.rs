pub mod books;
pub mod chapters;
pub mod content_cache;
pub mod db;
pub mod library;
pub mod local_import;
pub mod notes;
pub mod settings;
pub mod shelf;
pub mod sources;

pub use books::{
    add_book_with_chapters, delete_book, get_book_by_id, get_source_rule_json, list_books,
    update_book_progress, Book,
};
pub use chapters::{cache_content, get_chapter, list_chapters, load_chapter_content, save_chapters, ChapterRecord};
pub use library::{
    clear_custom_root, imports_dir, library_root, path_info, set_custom_root, texts_dir,
    LibraryPathInfo,
};
pub use db::{init_db, init_db_at, set_db_path};
pub use notes::{
    create_note, delete_note, get_note_by_id, list_all_notes_with_meta, list_notes_by_chapter,
    list_notes_grouped_by_book, update_note, CreateNotePayload, NoteListItem, NoteRecord,
    NotesByBook, UpdateNotePayload,
};
pub use settings::{get_all_settings, get_setting, set_setting};
pub use local_import::import_local_file;
pub use shelf::{
    add_from_search, add_to_shelf, export_book_to_file, get_book_detail, read_chapter,
    BookDetail, ExportBookResult,
};
pub use sources::{
    add_remote_source, delete_book_sources, get_source_by_id, import_source_json,
    import_sources_batch, init_builtin_sources, list_sources, list_subscriptions,
    ping_book_source, ping_book_sources, set_source_enabled, set_sources_enabled,
    sync_subscription, SourceHealth, SourceRecord, SourceSubscriptionRecord, SubscribeBatchResult,
};
