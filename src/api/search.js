import { invoke } from "@tauri-apps/api/core";

/** 全网搜索、榜单、从搜索加书架 */

export async function searchBooks(keyword, origin) {
  return invoke("search_books", { keyword, origin });
}

export async function addBookFromSearch({ title, author, catalogUrl, sourceId }) {
  return invoke("add_book_from_search", {
    payload: {
      title,
      author: author || null,
      catalog_url: catalogUrl,
      source_id: sourceId,
    },
  });
}

export async function getRankTypes(sourceId) {
  return invoke("get_rank_types", { sourceId });
}

export async function fetchRankings(sourceId, rankType, origin) {
  return invoke("fetch_rankings_cmd", { sourceId, rankType, origin });
}
