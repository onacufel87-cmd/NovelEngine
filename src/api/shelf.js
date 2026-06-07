import { invoke } from "@tauri-apps/api/core";

/** 书架 CRUD 与本地导入 */

export async function getShelfBooks() {
  return invoke("get_shelf_books");
}

export async function addBookToShelf({ title, catalogUrl, rule, chapters }) {
  return invoke("add_book_to_shelf", {
    payload: {
      title,
      catalog_url: catalogUrl,
      rule_json: JSON.stringify(rule),
      chapters,
    },
  });
}

export async function removeBookFromShelf(bookId) {
  return invoke("remove_book_from_shelf", { bookId });
}

export async function importLocalBook(filePath) {
  return invoke("import_local_book", { filePath });
}
