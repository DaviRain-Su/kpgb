/**
 * KPGB Blog Widget
 * Embeddable widget for displaying blog posts from KPGB on external websites
 * 
 * Usage:
 * <div id="kpgb-widget" data-base-url="https://YOUR_USERNAME.github.io/kpgb" data-count="5"></div>
 * <script src="https://YOUR_USERNAME.github.io/kpgb/js/widget.js"></script>
 */

(function() {
    'use strict';
    
    // Widget configuration
    const WIDGET_ID = 'kpgb-widget';
    const DEFAULT_COUNT = 5;
    const DEFAULT_THEME = 'light';
    
    // Find all widget containers
    const widgets = document.querySelectorAll(`#${WIDGET_ID}, .${WIDGET_ID}`);
    
    widgets.forEach(widget => {
        initializeWidget(widget);
    });
    
    async function initializeWidget(container) {
        // Get configuration from data attributes
        const baseUrl = container.dataset.baseUrl || window.location.origin;
        const count = parseInt(container.dataset.count) || DEFAULT_COUNT;
        const theme = container.dataset.theme || DEFAULT_THEME;
        const showSearch = container.dataset.search === 'true';
        const showTags = container.dataset.tags !== 'false';
        
        // Apply widget styles
        applyWidgetStyles(container, theme);
        
        try {
            // Fetch search index
            const response = await fetch(`${baseUrl}/search-index.json`);
            if (!response.ok) {
                throw new Error('Failed to load blog data');
            }
            
            const posts = await response.json();
            
            // Create widget content
            const widgetContent = createWidgetContent(posts.slice(0, count), baseUrl, {
                showSearch,
                showTags
            });
            
            container.innerHTML = widgetContent;
            
            // Initialize search if enabled
            if (showSearch) {
                initializeWidgetSearch(container, posts, baseUrl);
            }
            
        } catch (error) {
            console.error('KPGB Widget Error:', error);
            container.innerHTML = createErrorContent(baseUrl);
        }
    }
    
    function createWidgetContent(posts, baseUrl, options) {
        const searchHtml = options.showSearch ? `
            <div class="kpgb-search">
                <input type="search" class="kpgb-search-input" placeholder="Search posts...">
                <div class="kpgb-search-results"></div>
            </div>
        ` : '';
        
        const postsHtml = posts.map(post => createPostHtml(post, baseUrl, options)).join('');
        
        return `
            <div class="kpgb-widget-container">
                <div class="kpgb-widget-header">
                    <h3 class="kpgb-widget-title">Recent Posts</h3>
                    <a href="${baseUrl}" class="kpgb-widget-link" target="_blank">View all →</a>
                </div>
                ${searchHtml}
                <div class="kpgb-posts">
                    ${postsHtml}
                </div>
                <div class="kpgb-widget-footer">
                    <a href="${baseUrl}/feed.xml" class="kpgb-rss-link" target="_blank">
                        <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M3.429 5.1v2.4c7.061 0 12.771 5.71 12.771 12.771h2.4C18.6 11.91 11.79 5.1 3.429 5.1zm0 4.8v2.4a5.578 5.578 0 015.571 5.571h2.4c0-4.406-3.566-7.971-7.971-7.971zM6.171 15.486a1.714 1.714 0 11-2.428 2.428 1.714 1.714 0 012.428-2.428z"/>
                        </svg>
                        RSS Feed
                    </a>
                </div>
            </div>
        `;
    }
    
    function createPostHtml(post, baseUrl, options) {
        const tagsHtml = options.showTags && post.tags.length > 0 ? `
            <div class="kpgb-post-tags">
                ${post.tags.map(tag => `<span class="kpgb-tag">${escapeHtml(tag)}</span>`).join('')}
            </div>
        ` : '';
        
        return `
            <article class="kpgb-post">
                <h4 class="kpgb-post-title">
                    <a href="${baseUrl}/${post.url}" target="_blank">${escapeHtml(post.title)}</a>
                </h4>
                <time class="kpgb-post-date">${post.date}</time>
                ${tagsHtml}
            </article>
        `;
    }
    
    function createErrorContent(baseUrl) {
        return `
            <div class="kpgb-widget-container kpgb-error">
                <p>Unable to load blog posts.</p>
                <a href="${baseUrl}" target="_blank">Visit blog →</a>
            </div>
        `;
    }
    
    function initializeWidgetSearch(container, posts, baseUrl) {
        const searchInput = container.querySelector('.kpgb-search-input');
        const searchResults = container.querySelector('.kpgb-search-results');
        const postsContainer = container.querySelector('.kpgb-posts');
        
        if (!searchInput || !searchResults) return;
        
        let searchTimeout;
        
        searchInput.addEventListener('input', (e) => {
            clearTimeout(searchTimeout);
            const query = e.target.value.trim();
            
            if (query.length < 2) {
                searchResults.style.display = 'none';
                postsContainer.style.display = 'block';
                return;
            }
            
            searchTimeout = setTimeout(() => {
                const results = searchPosts(posts, query);
                displaySearchResults(results, searchResults, baseUrl);
                postsContainer.style.display = 'none';
                searchResults.style.display = 'block';
            }, 300);
        });
    }
    
    function searchPosts(posts, query) {
        const lowerQuery = query.toLowerCase();
        const words = lowerQuery.split(/\s+/);
        
        return posts.filter(post => {
            const searchText = `${post.title} ${post.content} ${post.tags.join(' ')}`.toLowerCase();
            return words.every(word => searchText.includes(word));
        }).slice(0, 10);
    }
    
    function displaySearchResults(results, container, baseUrl) {
        if (results.length === 0) {
            container.innerHTML = '<p class="kpgb-no-results">No results found.</p>';
            return;
        }
        
        const html = results.map(post => createPostHtml(post, baseUrl, { showTags: true })).join('');
        container.innerHTML = html;
    }
    
    function escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
    
    function applyWidgetStyles(container, theme) {
        // Check if styles are already injected
        if (document.getElementById('kpgb-widget-styles')) return;
        
        const styles = `
            .kpgb-widget-container {
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
                background: ${theme === 'dark' ? '#1a1a1a' : '#ffffff'};
                color: ${theme === 'dark' ? '#e0e0e0' : '#333333'};
                border: 1px solid ${theme === 'dark' ? '#333' : '#e0e0e0'};
                border-radius: 8px;
                padding: 1.5rem;
                max-width: 100%;
                box-sizing: border-box;
            }
            
            .kpgb-widget-header {
                display: flex;
                justify-content: space-between;
                align-items: center;
                margin-bottom: 1rem;
            }
            
            .kpgb-widget-title {
                margin: 0;
                font-size: 1.25rem;
                color: ${theme === 'dark' ? '#ffffff' : '#2c3e50'};
            }
            
            .kpgb-widget-link {
                color: #3498db;
                text-decoration: none;
                font-size: 0.9rem;
            }
            
            .kpgb-widget-link:hover {
                text-decoration: underline;
            }
            
            .kpgb-search {
                margin-bottom: 1rem;
            }
            
            .kpgb-search-input {
                width: 100%;
                padding: 0.5rem;
                border: 1px solid ${theme === 'dark' ? '#444' : '#ddd'};
                border-radius: 4px;
                background: ${theme === 'dark' ? '#2a2a2a' : '#f9f9f9'};
                color: ${theme === 'dark' ? '#e0e0e0' : '#333'};
                font-size: 0.9rem;
            }
            
            .kpgb-search-results {
                display: none;
                margin-top: 1rem;
            }
            
            .kpgb-posts {
                display: block;
            }
            
            .kpgb-post {
                padding: 0.75rem 0;
                border-bottom: 1px solid ${theme === 'dark' ? '#333' : '#f0f0f0'};
            }
            
            .kpgb-post:last-child {
                border-bottom: none;
                padding-bottom: 0;
            }
            
            .kpgb-post-title {
                margin: 0 0 0.25rem 0;
                font-size: 1rem;
            }
            
            .kpgb-post-title a {
                color: ${theme === 'dark' ? '#4db8ff' : '#2c3e50'};
                text-decoration: none;
            }
            
            .kpgb-post-title a:hover {
                color: #3498db;
            }
            
            .kpgb-post-date {
                font-size: 0.85rem;
                color: ${theme === 'dark' ? '#999' : '#7f8c8d'};
            }
            
            .kpgb-post-tags {
                margin-top: 0.25rem;
            }
            
            .kpgb-tag {
                display: inline-block;
                background: ${theme === 'dark' ? '#333' : '#ecf0f1'};
                color: ${theme === 'dark' ? '#ccc' : '#34495e'};
                padding: 0.15rem 0.4rem;
                border-radius: 3px;
                font-size: 0.75rem;
                margin-right: 0.25rem;
            }
            
            .kpgb-widget-footer {
                margin-top: 1rem;
                padding-top: 1rem;
                border-top: 1px solid ${theme === 'dark' ? '#333' : '#f0f0f0'};
                text-align: center;
            }
            
            .kpgb-rss-link {
                display: inline-flex;
                align-items: center;
                gap: 0.25rem;
                color: ${theme === 'dark' ? '#999' : '#7f8c8d'};
                text-decoration: none;
                font-size: 0.85rem;
            }
            
            .kpgb-rss-link:hover {
                color: #3498db;
            }
            
            .kpgb-no-results {
                text-align: center;
                color: ${theme === 'dark' ? '#999' : '#7f8c8d'};
                padding: 1rem;
                margin: 0;
            }
            
            .kpgb-error {
                text-align: center;
                padding: 2rem;
            }
            
            .kpgb-error p {
                color: ${theme === 'dark' ? '#ff6b6b' : '#e74c3c'};
                margin-bottom: 1rem;
            }
        `;
        
        const styleSheet = document.createElement('style');
        styleSheet.id = 'kpgb-widget-styles';
        styleSheet.textContent = styles;
        document.head.appendChild(styleSheet);
    }
    
    // Auto-initialize on script load
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            // Re-scan for widgets that might have been added dynamically
            const dynamicWidgets = document.querySelectorAll(`#${WIDGET_ID}, .${WIDGET_ID}`);
            dynamicWidgets.forEach(widget => {
                if (!widget.querySelector('.kpgb-widget-container')) {
                    initializeWidget(widget);
                }
            });
        });
    }
    
    // Export for manual initialization
    window.KPGBWidget = {
        init: initializeWidget,
        refresh: () => {
            const widgets = document.querySelectorAll(`#${WIDGET_ID}, .${WIDGET_ID}`);
            widgets.forEach(widget => {
                widget.innerHTML = '';
                initializeWidget(widget);
            });
        }
    };
})();