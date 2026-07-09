/**
 * Typstify Search — Client-side search with JSON index
 *
 * Loads search-index.json, performs fuzzy matching, and renders
 * results in a dropdown below the search input.
 */
(function () {
  'use strict';

  const DEBOUNCE_MS = 150;
  const MIN_QUERY_LENGTH = 2;
  const MAX_RESULTS = 10;

  let searchIndex = null;
  let loading = false;
  let error = null;
  let activeIndex = -1;

  const input = document.getElementById('searchInput');
  const resultsContainer = document.getElementById('searchResults');
  const searchWrapper = document.getElementById('searchWrapper');

  if (!input || !resultsContainer || !searchWrapper) return;

  // --- ARIA Setup (P0 fix) ---
  input.setAttribute('aria-label', 'Search posts, tags, and pages');
  input.setAttribute('aria-autocomplete', 'list');
  input.setAttribute('aria-controls', 'searchResults');
  input.setAttribute('aria-expanded', 'false');
  resultsContainer.setAttribute('role', 'listbox');
  resultsContainer.setAttribute('aria-label', 'Search results');

  // --- Index Loading ---

  async function loadIndex() {
    if (searchIndex || loading) return searchIndex;

    loading = true;
    error = null;

    try {
      const basePath = getBasePath();
      const response = await fetch(`${basePath}/search-index.json`);
      if (!response.ok) throw new Error(`HTTP ${response.status}`);
      searchIndex = await response.json();
    } catch (e) {
      error = e.message;
      console.error('Search index load failed:', e);
    } finally {
      loading = false;
    }

    return searchIndex;
  }

  function getBasePath() {
    const base = document.querySelector('base');
    if (base) return base.href.replace(/\/$/, '');
    const path = window.location.pathname;
    const depth = path.split('/').filter(Boolean).length;
    return depth > 1 ? '../'.repeat(depth - 1) : '.';
  }

  // --- Search Logic ---

  function search(query, index) {
    if (!index || !index.documents) return [];

    const terms = query.toLowerCase().split(/\s+/).filter(Boolean);
    if (terms.length === 0) return [];

    const scored = [];

    for (let i = 0; i < index.documents.length; i++) {
      const doc = index.documents[i];
      let score = 0;

      const titleLower = (doc.title || '').toLowerCase();
      const descLower = (doc.description || '').toLowerCase();
      const tagsLower = (doc.tags || []).join(' ').toLowerCase();
      const termsLower = (doc.terms || []).join(' ').toLowerCase();

      for (const term of terms) {
        if (titleLower.includes(term)) score += 10;
        if (descLower.includes(term)) score += 5;
        if (tagsLower.includes(term)) score += 3;
        if (termsLower.includes(term)) score += 1;

        if (titleLower.startsWith(term)) score += 5;
      }

      if (score > 0) {
        scored.push({ doc, score });
      }
    }

    scored.sort((a, b) => b.score - a.score);
    return scored.slice(0, MAX_RESULTS).map((s) => s.doc);
  }

  // --- Rendering ---

  function renderResults(results, query) {
    if (!resultsContainer) return;

    if (results.length === 0) {
      resultsContainer.innerHTML = `<div class="search-empty">No results for "${escapeHtml(query)}"</div>`;
      resultsContainer.classList.remove('hidden');
      activeIndex = -1;
      return;
    }

    const html = results
      .map((r, i) => {
        const tags = (r.tags || [])
          .map((t) => `<span class="search-result-tag">${escapeHtml(t)}</span>`)
          .join('');

        const resultId = `search-result-${i}`;
        return `
        <a href="${escapeHtml(r.url || '#')}" class="search-result-item" id="${resultId}" data-index="${i}" role="option" aria-selected="false">
          <div class="search-result-item-title">${escapeHtml(r.title || 'Untitled')}</div>
          ${r.description ? `<div class="search-result-item-summary">${escapeHtml(r.description)}</div>` : ''}
          ${tags ? `<div class="search-result-item-tags">${tags}</div>` : ''}
        </a>`;
      })
      .join('');

    resultsContainer.innerHTML = html;
    resultsContainer.classList.remove('hidden');
    input.setAttribute('aria-expanded', 'true');
    activeIndex = -1;
  }

  function renderLoading() {
    if (!resultsContainer) return;
    resultsContainer.innerHTML = '<div class="search-loading">Searching...</div>';
    resultsContainer.classList.remove('hidden');
  }

  function renderError(msg) {
    if (!resultsContainer) return;
    resultsContainer.innerHTML = `<div class="search-error">Search unavailable: ${escapeHtml(msg)}</div>`;
    resultsContainer.classList.remove('hidden');
  }

  function renderEmpty() {
    if (!resultsContainer) return;
    resultsContainer.innerHTML = '';
    resultsContainer.classList.add('hidden');
    input.setAttribute('aria-expanded', 'false');
    input.removeAttribute('aria-activedescendant');
    activeIndex = -1;
  }

  // --- Keyboard Navigation ---

  function getVisibleItems() {
    return resultsContainer.querySelectorAll('.search-result-item');
  }

  function setActive(index) {
    const items = getVisibleItems();
    items.forEach((el, i) => {
      el.setAttribute('data-active', i === index ? 'true' : 'false');
      el.setAttribute('aria-selected', i === index ? 'true' : 'false');
    });
    activeIndex = index;

    if (index >= 0 && items[index]) {
      input.setAttribute('aria-activedescendant', items[index].id);
      items[index].scrollIntoView({ block: 'nearest' });
    } else {
      input.removeAttribute('aria-activedescendant');
    }
  }

  function handleKeydown(e) {
    if (!searchWrapper || searchWrapper.classList.contains('hidden')) return;

    const items = getVisibleItems();
    if (items.length === 0) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setActive(activeIndex < items.length - 1 ? activeIndex + 1 : 0);
        break;

      case 'ArrowUp':
        e.preventDefault();
        setActive(activeIndex > 0 ? activeIndex - 1 : items.length - 1);
        break;

      case 'Enter':
        e.preventDefault();
        if (activeIndex >= 0 && items[activeIndex]) {
          items[activeIndex].click();
        }
        break;

      case 'Escape':
        e.preventDefault();
        renderEmpty();
        input.blur();
        break;
    }
  }

  // --- Event Handlers ---

  let debounceTimer = null;

  function handleInput() {
    const query = input.value.trim();

    clearTimeout(debounceTimer);

    if (query.length < MIN_QUERY_LENGTH) {
      renderEmpty();
      return;
    }

    debounceTimer = setTimeout(async () => {
      const index = await loadIndex();

      if (error) {
        renderError(error);
        return;
      }

      if (!index) {
        renderLoading();
        return;
      }

      const results = search(query, index);
      renderResults(results, query);
    }, DEBOUNCE_MS);
  }

  function handleClickOutside(e) {
    if (searchWrapper && !searchWrapper.contains(e.target)) {
      renderEmpty();
    }
  }

  // --- Focus handling for shortcut hint ---

  function handleFocus() {
    const query = input.value.trim();
    if (query.length >= MIN_QUERY_LENGTH && searchIndex) {
      const results = search(query, searchIndex);
      renderResults(results, query);
    }
  }

  // --- Init ---

  function init() {
    input.addEventListener('input', handleInput);
    input.addEventListener('keydown', handleKeydown);
    input.addEventListener('focus', handleFocus);
    document.addEventListener('click', handleClickOutside);

    // P1 fix: Keyboard shortcut (/) to focus search
    document.addEventListener('keydown', (e) => {
      if (e.key === '/' && !e.ctrlKey && !e.metaKey && !e.altKey) {
        const activeTag = document.activeElement?.tagName;
        if (activeTag !== 'INPUT' && activeTag !== 'TEXTAREA' && activeTag !== 'SELECT') {
          e.preventDefault();
          input.focus();
        }
      }
    });

    // Preload index on idle
    if ('requestIdleCallback' in window) {
      requestIdleCallback(() => loadIndex());
    } else {
      setTimeout(() => loadIndex(), 2000);
    }
  }

  // --- Helpers ---

  function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
  }

  // Run
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
