//! 阅读笔记命令

use crate::services::{notes, to_cmd_err};
use crate::storage::notes::{
    CreateNotePayload, NoteListItem, NoteRecord, NotesByBook, UpdateNotePayload,
};

#[tauri::command]
pub fn create_reader_note(payload: CreateNotePayload) -> Result<NoteRecord, String> {
    notes::add_note(payload).map_err(to_cmd_err)
}

#[tauri::command]
pub fn update_reader_note(payload: UpdateNotePayload) -> Result<NoteRecord, String> {
    notes::edit_note(payload).map_err(to_cmd_err)
}

#[tauri::command]
pub fn delete_reader_note(note_id: i64) -> Result<(), String> {
    notes::remove_note(note_id).map_err(to_cmd_err)
}

#[tauri::command]
pub fn list_chapter_notes(book_id: i64, chapter_id: i64) -> Result<Vec<NoteRecord>, String> {
    notes::chapter_notes(book_id, chapter_id).map_err(to_cmd_err)
}

#[tauri::command]
pub fn list_all_reader_notes() -> Result<Vec<NoteListItem>, String> {
    notes::all_notes().map_err(to_cmd_err)
}

#[tauri::command]
pub fn list_reader_notes_by_book() -> Result<Vec<NotesByBook>, String> {
    notes::notes_by_book().map_err(to_cmd_err)
}
