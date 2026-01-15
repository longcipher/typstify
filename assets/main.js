// Prevent duplicate initialization
if (window.__typstifyInit) { throw new Error('already initialized'); }
window.__typstifyInit = true;

// Global cleanup controller
const cleanupController = new AbortController();
const { signal } = cleanupController;

// Cleanup on page unload to prevent memory leaks
window.addEventListener('pagehide', () => cleanupController.abort());
window.addEventListener('beforeunload', () => cleanupController.abort());

// Theme toggle functionality
(function() {
    const toggle = document.querySelector('.theme-toggle');
    if (!toggle) return;
    const html = document.documentElement;

    function getTheme() {
        const saved = localStorage.getItem('theme');
        if (saved) return saved;
        return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }

    function setTheme(theme) {
        html.setAttribute('data-theme', theme);
        localStorage.setItem('theme', theme);
    }

    setTheme(getTheme());

    toggle.addEventListener('click', () => {
        const current = html.getAttribute('data-theme') || getTheme();
        setTheme(current === 'dark' ? 'light' : 'dark');
    }, { signal });

    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
        if (!localStorage.getItem('theme')) {
            setTheme(e.matches ? 'dark' : 'light');
        }
    }, { signal });
})();

// Search functionality
(function() {
    const wrapper = document.getElementById('searchWrapper');
    const btn = document.getElementById('searchBtn');
    const input = document.getElementById('searchInput');
    const results = document.getElementById('searchResults');
    if (!wrapper || !btn || !input || !results) return;

    let searchIndex = null;
    let isLoading = false;
    let debounceTimer = null;

    // Clear debounce on cleanup
    signal.addEventListener('abort', () => clearTimeout(debounceTimer));

    btn.addEventListener('click', (e) => {
        e.stopPropagation();
        if (wrapper.classList.contains('active')) {
            if (input.value.trim()) {
                performSearch(input.value.trim());
            }
        } else {
            wrapper.classList.add('active');
            input.focus();
            loadSearchIndex();
        }
    }, { signal });

    document.addEventListener('click', (e) => {
        if (!wrapper.contains(e.target)) {
            wrapper.classList.remove('active');
            results.classList.remove('show');
        }
    }, { signal });

    input.addEventListener('input', () => {
        clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => {
            const query = input.value.trim();
            if (query.length >= 1) {
                performSearch(query);
            } else {
                results.classList.remove('show');
            }
        }, 150);
    }, { signal });

    input.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            e.preventDefault();
            performSearch(input.value.trim());
        } else if (e.key === 'Escape') {
            wrapper.classList.remove('active');
            results.classList.remove('show');
        }
    }, { signal });

    async function loadSearchIndex() {
        if (searchIndex || isLoading) return;
        isLoading = true;
        try {
            const pathParts = window.location.pathname.split('/').filter(Boolean);
            const langPrefix = pathParts.length > 0 && pathParts[0].length === 2 ? pathParts[0] : '';
            const indexPath = langPrefix ? `/${langPrefix}/search-index.json` : '/search-index.json';
            
            const response = await fetch(indexPath, { signal });
            if (response.ok) {
                searchIndex = await response.json();
            }
        } catch (err) {
            if (err.name !== 'AbortError') {
                console.log('Search index not available');
            }
        }
        isLoading = false;
    }

    function performSearch(query) {
        if (!searchIndex || !searchIndex.documents) {
            results.innerHTML = '<div class="search-no-results">Search is loading...</div>';
            results.classList.add('show');
            return;
        }

        const q = query.toLowerCase();
        const matches = searchIndex.documents.filter(doc => {
            const title = doc.title.toLowerCase();
            const desc = (doc.description || '').toLowerCase();
            const terms = doc.terms || [];
            
            if (title.includes(q) || desc.includes(q)) return true;
            if (terms.some(t => t.includes(q) || q.includes(t))) return true;
            return false;
        }).slice(0, 10);

        if (matches.length === 0) {
            results.innerHTML = '<div class="search-no-results">No results found</div>';
        } else {
            results.innerHTML = matches.map(doc => 
                `<a href="${doc.url}" class="search-result-item">
                    <div class="search-result-title">${escapeHtml(doc.title)}</div>
                    ${doc.description ? `<div class="search-result-snippet">${escapeHtml(doc.description)}</div>` : ''}
                </a>`
            ).join('');
        }
        results.classList.add('show');
    }
    
    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
})();
