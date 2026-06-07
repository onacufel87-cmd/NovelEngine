/**
 * 划词锚点：在正文容器内计算字符偏移与上下文
 */

const CONTEXT_LEN = 30;

/** 简单内容指纹，用于检测正文是否变化 */
export function hashContent(text) {
  let h = 5381;
  for (let i = 0; i < text.length; i += 1) {
    h = (h * 33) ^ text.charCodeAt(i);
  }
  return (h >>> 0).toString(16);
}

function buildAnchor(plainText, start, end, quote) {
  return {
    start_offset: start,
    end_offset: end,
    quote,
    context_before: plainText.slice(Math.max(0, start - CONTEXT_LEN), start),
    context_after: plainText.slice(end, end + CONTEXT_LEN),
    content_hash: hashContent(plainText),
  };
}

/**
 * 以用户实际选中文本为准，在 plainText 中定位准确偏移（纠正 DOM/索引偏差）
 */
export function locateSelectionInPlainText(plainText, selected, startHint, endHint) {
  if (!selected?.trim() || !plainText) return null;

  // 1. DOM 推算区间切片正确
  if (startHint != null && endHint != null && endHint > startHint) {
    const slice = plainText.slice(startHint, endHint);
    if (slice === selected) {
      return buildAnchor(plainText, startHint, endHint, selected);
    }
  }

  // 2. 在 hint 附近搜索选中文本
  const searchFrom = Math.max(0, (startHint ?? 0) - 120);
  let idx = plainText.indexOf(selected, searchFrom);
  if (idx >= 0) {
    return buildAnchor(plainText, idx, idx + selected.length, selected);
  }

  // 3. 用 hint 上下文构造锚点串再匹配
  if (startHint != null && endHint != null) {
    const ctxBefore = plainText.slice(
      Math.max(0, startHint - CONTEXT_LEN),
      startHint
    );
    const ctxAfter = plainText.slice(endHint, endHint + CONTEXT_LEN);
    const needle = ctxBefore + selected + ctxAfter;
    const ctxIdx = plainText.indexOf(needle);
    if (ctxIdx >= 0) {
      const start = ctxIdx + ctxBefore.length;
      return buildAnchor(plainText, start, start + selected.length, selected);
    }
  }

  // 4. 全文首次出现（兜底）
  idx = plainText.indexOf(selected);
  if (idx >= 0) {
    return buildAnchor(plainText, idx, idx + selected.length, selected);
  }

  return null;
}

/**
 * 将 Range 端点映射到 plainText 字符偏移
 * 依赖每行 .para-text 上的 data-line-start
 */
function rangePointToPlainOffset(container, node, offset) {
  let el = node.nodeType === Node.TEXT_NODE ? node.parentElement : node;
  while (el && el !== container) {
    if (el.classList?.contains("para-text")) {
      const lineStart = Number(el.dataset.lineStart || 0);
      const pre = document.createRange();
      pre.selectNodeContents(el);
      try {
        pre.setEnd(node, offset);
      } catch {
        return lineStart;
      }
      return lineStart + pre.toString().length;
    }
    el = el.parentElement;
  }
  return null;
}

/**
 * 从当前选区读取在 plainText 中的起止位置
 * @param {HTMLElement} container 正文容器（仅含分段正文，不含顶部统计条）
 * @param {string} plainText 原始正文字符串
 */
export function getSelectionAnchor(container, plainText) {
  const selection = window.getSelection();
  if (!selection || selection.rangeCount === 0 || selection.isCollapsed) {
    return null;
  }

  const range = selection.getRangeAt(0);
  if (!container.contains(range.commonAncestorContainer)) {
    return null;
  }

  const selected = range.toString();
  if (!selected.trim()) {
    return null;
  }

  const start = rangePointToPlainOffset(
    container,
    range.startContainer,
    range.startOffset
  );
  const end = rangePointToPlainOffset(container, range.endContainer, range.endOffset);

  if (start == null || end == null) {
    return locateSelectionInPlainText(plainText, selected, null, null);
  }

  return locateSelectionInPlainText(plainText, selected, start, end);
}

/**
 * 将正文按标注切分为渲染片段
 * @param {string} plainText
 * @param {Array} notes 含 start_offset/end_offset/id/body/color
 */
export function buildHighlightSegments(plainText, notes) {
  if (!plainText) return [{ type: "text", text: "" }];

  const sorted = [...notes]
    .filter(
      (n) =>
        n.start_offset != null &&
        n.end_offset != null &&
        n.end_offset > n.start_offset
    )
    .sort((a, b) => a.start_offset - b.start_offset);

  const segments = [];
  let cursor = 0;

  for (const note of sorted) {
    const start = Math.max(0, Math.min(note.start_offset, plainText.length));
    const end = Math.max(start, Math.min(note.end_offset, plainText.length));
    if (start < cursor) continue;

    if (start > cursor) {
      segments.push({ type: "text", text: plainText.slice(cursor, start) });
    }

    segments.push({
      type: "highlight",
      text: plainText.slice(start, end),
      noteId: note.id,
      color: note.color,
      hasBody: Boolean(note.body?.trim()),
    });
    cursor = end;
  }

  if (cursor < plainText.length) {
    segments.push({ type: "text", text: plainText.slice(cursor) });
  }

  return segments.length ? segments : [{ type: "text", text: plainText }];
}

/** 获取一行内的评论（起始位置在本行） */
export function notesStartingInLine(notes, lineStart, lineEnd) {
  return notes.filter(
    (n) =>
      n.start_offset != null &&
      n.end_offset != null &&
      n.start_offset >= lineStart &&
      n.start_offset < lineEnd
  );
}

/** 清除当前选区 */
export function clearSelection() {
  const sel = window.getSelection();
  sel?.removeAllRanges();
}
