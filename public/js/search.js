// Client-side search functionality for KPGB static site
(function() {
    'use strict';
    
    let searchIndex = null;
    let searchInput = null;
    let searchResults = null;
    
    // Load search index
    async function loadSearchIndex() {
        try {
            const response = await fetch('/search-index.json');
            searchIndex = await response.json();
        } catch (error) {
            console.error('Failed to load search index:', error);
        }
    }
    
    // Perform search
    function performSearch(query) {
        if (!searchIndex || !query) {
            return [];
        }
        
        const lowerQuery = query.toLowerCase();
        const words = lowerQuery.split(/\s+/).filter(w => w.length > 0);
        
        const results = searchIndex.filter(post => {
            const searchableText = `${post.title} ${post.content} ${post.tags.join(' ')}`.toLowerCase();
            return words.every(word => searchableText.includes(word));
        });
        
        // Score results based on relevance
        results.forEach(post => {
            let score = 0;
            const titleLower = post.title.toLowerCase();
            
            // Title matches are weighted more
            words.forEach(word => {
                if (titleLower.includes(word)) score += 10;
                if (post.content.toLowerCase().includes(word)) score += 1;
                if (post.tags.some(tag => tag.toLowerCase().includes(word))) score += 5;
            });
            
            post._score = score;
        });
        
        // Sort by score
        results.sort((a, b) => b._score - a._score);
        
        return results.slice(0, 20); // Return top 20 results
    }
    
    // Display search results
    function displayResults(results) {
        if (!searchResults) return;
        
        if (results.length === 0) {
            searchResults.innerHTML = '<p class="no-results">No results found.</p>';
            return;
        }
        
        const html = results.map(post => {
            const excerpt = post.content.substring(0, 200) + '...';
            return `
                <article class="search-result">
                    <h3><a href="/${post.url}">${escapeHtml(post.title)}</a></h3>
                    <time>${post.date}</time>
                    <p>${escapeHtml(excerpt)}</p>
                    ${post.tags.length > 0 ? `<div class="tags">${post.tags.map(tag => `<span class="tag">${escapeHtml(tag)}</span>`).join('')}</div>` : ''}
                </article>
            `;
        }).join('');
        
        searchResults.innerHTML = html;
    }
    
    // Escape HTML to prevent XSS
    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
    
    // Debounce function
    function debounce(func, wait) {
        let timeout;
        return function(...args) {
            clearTimeout(timeout);
            timeout = setTimeout(() => func.apply(this, args), wait);
        };
    }
    
    // Initialize search
    function initSearch() {
        searchInput = document.getElementById('search-input');
        searchResults = document.getElementById('search-results');
        
        if (!searchInput || !searchResults) return;
        
        // Load search index
        loadSearchIndex();
        
        // Handle search input
        const handleSearch = debounce(() => {
            const query = searchInput.value.trim();
            if (query.length < 2) {
                searchResults.innerHTML = '';
                return;
            }
            
            const results = performSearch(query);
            displayResults(results);
        }, 300);
        
        searchInput.addEventListener('input', handleSearch);
        
        // Handle search form submission
        const searchForm = searchInput.closest('form');
        if (searchForm) {
            searchForm.addEventListener('submit', (e) => {
                e.preventDefault();
                handleSearch();
            });
        }
    }
    
    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initSearch);
    } else {
        initSearch();
    }
    
    // Export for widget use
    window.KPGBSearch = {
        load: loadSearchIndex,
        search: performSearch,
        init: initSearch
    };
})();