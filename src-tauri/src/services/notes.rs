//! 阅读笔记业务

use crate::storage::notes::{
    create_note, delete_note, get_note_by_id, list_all_notes_with_meta,
    list_notes_by_chapter, list_notes_grouped_by_book, update_note, CreateNotePayload,
    NoteListItem, NoteRecord, NotesByBook, UpdateNotePayload,
};
use crate::utils::AppResult;

pub fn add_note(payload: CreateNotePayload) -> AppResult<NoteRecord> {
    create_note(&payload)
}

pub fn edit_note(payload: UpdateNotePayload) -> AppResult<NoteRecord> {
    update_note(&payload)
}

pub fn remove_note(note_id: i64) -> AppResult<()> {
    delete_note(note_id)
}

pub fn get_note(note_id: i64) -> AppResult<NoteRecord> {
    get_note_by_id(note_id)
}

pub fn chapter_notes(book_id: i64, chapter_id: i64) -> AppResult<Vec<NoteRecord>> {
    list_notes_by_chapter(book_id, chapter_id)
}

pub fn all_notes() -> AppResult<Vec<NoteListItem>> {
    list_all_notes_with_meta()
}

pub fn notes_by_book() -> AppResult<Vec<NotesByBook>> {
    list_notes_grouped_by_book()
}
