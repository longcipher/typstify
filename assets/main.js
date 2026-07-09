
(function() {
    'use strict';

    function initThemeToggle() {
        var toggle = document.querySelector('.theme-toggle');
        if (!toggle) return;
        var html = document.documentElement;

        toggle.addEventListener('click', function() {
            var current = html.getAttribute('data-theme');
            if (current !== 'dark' && current !== 'light') {
                current = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
            }
            var next = current === 'dark' ? 'light' : 'dark';
            html.setAttribute('data-theme', next);
            try { localStorage.setItem('theme', next); } catch (e) {}
        });

        window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', function(e) {
            try {
                if (!localStorage.getItem('theme')) {
                    html.setAttribute('data-theme', e.matches ? 'dark' : 'light');
                }
            } catch (e) {}
        });
    }

    function initLangSwitcher() {
        var switcher = document.querySelector('.lang-switcher');
        if (!switcher) return;

        switcher.addEventListener('click', function(e) {
            e.stopPropagation();
            var isOpen = switcher.classList.contains('open');
            document.querySelectorAll('.lang-switcher.open').forEach(function(el) {
                el.classList.remove('open');
            });
            if (!isOpen) {
                switcher.classList.add('open');
            }
        });

        switcher.addEventListener('keydown', function(e) {
            if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                switcher.click();
            } else if (e.key === 'Escape') {
                switcher.classList.remove('open');
            }
        });

        document.addEventListener('click', function() {
            switcher.classList.remove('open');
        });
    }

    function initSearch() {
        var input = document.getElementById('searchInput');
        var results = document.getElementById('searchResults');
        var wrapper = document.getElementById('searchWrapper');
        var searchBtn = wrapper ? wrapper.querySelector('.search-btn') : null;
        if (!input || !results || !wrapper) return;

        if (searchBtn) {
            searchBtn.addEventListener('click', function() {
                input.focus();
            });
        }

        var searchIndex = null;
        var debounceTimer = null;
        var activeIndex = -1;
        var currentMatches = [];

        function getBasePath() {
            var path = window.location.pathname;
            var segments = path.split('/').filter(Boolean);
            if (segments.length > 0 && segments[0].length === 2) {
                return '/' + segments[0];
            }
            return '';
        }

        function escapeHtml(text) {
            var div = document.createElement('div');
            div.appendChild(document.createTextNode(text));
            return div.innerHTML;
        }

        function loadSearchIndex() {
            if (searchIndex) return Promise.resolve(searchIndex);
            var basePath = getBasePath();
            var indexPath = basePath + '/search-index.json';
            return fetch(indexPath)
                .then(function(res) {
                    if (!res.ok) throw new Error('not found');
                    return res.json();
                })
                .then(function(data) {
                    searchIndex = data;
                    return data;
                })
                .catch(function() {
                    return null;
                });
        }

        function showResults(items) {
            currentMatches = items;
            activeIndex = -1;
            if (!items || items.length === 0) {
                results.innerHTML = '<div class="search-empty">No results found</div>';
            } else {
                results.innerHTML = items.map(function(doc, idx) {
                    var tagsHtml = '';
                    if (doc.tags && doc.tags.length > 0) {
                        tagsHtml = '<div class="search-result-item-tags">' +
                            doc.tags.map(function(t) {
                                return '<span class="search-result-tag">' + escapeHtml(t) + '</span>';
                            }).join('') + '</div>';
                    }
                    return '<a href="' + escapeHtml(doc.url) + '" class="search-result-item" data-index="' + idx + '">' +
                        '<div class="search-result-item-title">' + escapeHtml(doc.title) + '</div>' +
                        (doc.description ? '<div class="search-result-item-summary">' + escapeHtml(doc.description) + '</div>' : '') +
                        tagsHtml + '</a>';
                }).join('');
            }
            results.classList.add('visible');
        }

        function hideResults() {
            results.classList.remove('visible');
            currentMatches = [];
            activeIndex = -1;
        }

        function updateActive() {
            var items = results.querySelectorAll('.search-result-item');
            items.forEach(function(el, idx) {
                el.setAttribute('data-active', idx === activeIndex ? 'true' : 'false');
            });
        }

        function performSearch(query) {
            loadSearchIndex().then(function(index) {
                if (!index || !index.documents) {
                    results.innerHTML = '<div class="search-error">Search index not available</div>';
                    results.classList.add('visible');
                    return;
                }
                var q = query.toLowerCase();
                var matches = index.documents.filter(function(doc) {
                    var title = (doc.title || '').toLowerCase();
                    var desc = (doc.description || '').toLowerCase();
                    var tags = (doc.tags || []).join(' ').toLowerCase();
                    if (title.indexOf(q) !== -1) return true;
                    if (desc.indexOf(q) !== -1) return true;
                    if (tags.indexOf(q) !== -1) return true;
                    return false;
                }).slice(0, 8);
                showResults(matches);
            });
        }

        input.addEventListener('focus', function() {
            loadSearchIndex();
            if (input.value.trim()) {
                performSearch(input.value.trim());
            }
        });

        input.addEventListener('input', function() {
            clearTimeout(debounceTimer);
            var query = input.value.trim();
            if (query.length === 0) {
                hideResults();
                return;
            }
            debounceTimer = setTimeout(function() {
                performSearch(query);
            }, 200);
        });

        input.addEventListener('keydown', function(e) {
            var items = results.querySelectorAll('.search-result-item');
            if (e.key === 'ArrowDown') {
                e.preventDefault();
                if (items.length === 0) return;
                activeIndex = activeIndex < items.length - 1 ? activeIndex + 1 : 0;
                updateActive();
            } else if (e.key === 'ArrowUp') {
                e.preventDefault();
                if (items.length === 0) return;
                activeIndex = activeIndex > 0 ? activeIndex - 1 : items.length - 1;
                updateActive();
            } else if (e.key === 'Enter') {
                e.preventDefault();
                if (activeIndex >= 0 && currentMatches[activeIndex]) {
                    window.location.href = currentMatches[activeIndex].url;
                }
            } else if (e.key === 'Escape') {
                hideResults();
                input.blur();
            }
        });

        document.addEventListener('click', function(e) {
            if (!wrapper.contains(e.target)) {
                hideResults();
            }
        });
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', function() {
            initThemeToggle();
            initLangSwitcher();
            initSearch();
        });
    } else {
        initThemeToggle();
        initLangSwitcher();
        initSearch();
    }
})();
