//! Tauri 命令层：薄接口，按业务域拆分

mod books;
mod notes;
mod search;
mod settings;
mod sources;

/// 注册全部 invoke 命令
pub fn handler() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        // 书架 / 阅读
        books::get_shelf_books,
        books::add_book_to_shelf,
        books::remove_book_from_shelf,
        books::get_book_detail_cmd,
        books::read_chapter_content,
        books::save_read_progress,
        books::export_book,
        books::import_local_book,
        // 书源 / 解析 / 自动接入
        sources::validate_source_rule,
        sources::fetch_chapters,
        sources::get_book_content,
        sources::list_book_sources,
        sources::toggle_book_source,
        sources::subscribe_remote_source,
        sources::import_book_source_json,
        sources::import_book_sources_batch,
        sources::list_source_subscriptions,
        sources::sync_source_subscription,
        sources::ping_book_source,
        sources::ping_book_sources,
        sources::delete_book_sources,
        sources::set_book_sources_enabled,
        sources::auto_detect_selectors_cmd,
        sources::auto_detect_source_rule,
        sources::auto_detect_from_url,
        sources::auto_detect_from_url_rendered,
        sources::receive_captured_html,
        // 搜索 / 榜单
        search::search_books,
        search::add_book_from_search,
        search::get_rank_types,
        search::fetch_rankings_cmd,
        // 设置
        settings::get_reader_settings,
        settings::save_reader_settings,
        settings::get_library_path,
        settings::set_library_path,
        settings::reset_library_path,
        // 阅读笔记
        notes::create_reader_note,
        notes::update_reader_note,
        notes::delete_reader_note,
        notes::list_chapter_notes,
        notes::list_all_reader_notes,
        notes::list_reader_notes_by_book,
    ]
}
