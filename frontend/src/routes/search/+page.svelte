<script lang="ts">
  import { goto, page } from '$app/stores';
  import { onMount } from 'svelte';
  import SnippetCard from '$lib/components/SnippetCard.svelte';
  import { api } from '$lib/api';
  import type { Snippet, PaginatedResponse, Tag } from '$lib/types';

  let snippets: Snippet[] = [];
  let popularTags: Tag[] = [];
  let languages: string[] = [];
  let loading: boolean = false;
  let totalPages: number = 0;
  let currentPage: number = 1;

  let searchQuery: string = '';
  let selectedLanguage: string = '';
  let selectedTags: string = '';

  $: {
    const urlParams = new URLSearchParams($page.url.search);
    searchQuery = urlParams.get('q') || '';
    selectedLanguage = urlParams.get('language') || '';
    selectedTags = urlParams.get('tags') || '';
  }

  async function loadSnippets(pageNum: number = 1) {
    loading = true;
    try {
      let endpoint = `/api/search?page=${pageNum}&per_page=12`;
      if (searchQuery) endpoint += `&q=${encodeURIComponent(searchQuery)}`;
      if (selectedLanguage) endpoint += `&language=${encodeURIComponent(selectedLanguage)}`;
      if (selectedTags) endpoint += `&tags=${encodeURIComponent(selectedTags)}`;

      const response = await api.get(endpoint);
      if (response.ok) {
        const data: PaginatedResponse<Snippet> = await response.json();
        snippets = data.data;
        totalPages = data.total_pages;
        currentPage = data.page;
      }
    } catch (e) {
      console.error('Failed to load snippets:', e);
    } finally {
      loading = false;
    }
  }

  async function loadFilters() {
    try {
      const [tagsRes, langRes] = await Promise.all([
        api.get('/api/search/tags'),
        api.get('/api/search/languages')
      ]);

      if (tagsRes.ok) {
        popularTags = await tagsRes.json();
      }

      if (langRes.ok) {
        languages = await langRes.json();
      }
    } catch (e) {
      console.error('Failed to load filters:', e);
    }
  }

  function handleSearch(e: SubmitEvent) {
    e.preventDefault();
    updateUrl();
  }

  function handleLanguageChange(lang: string) {
    selectedLanguage = lang;
    updateUrl();
  }

  function handleTagClick(tag: string) {
    if (selectedTags) {
      const tags = selectedTags.split(',').map(t => t.trim());
      if (tags.includes(tag)) {
        selectedTags = tags.filter(t => t !== tag).join(', ');
      } else {
        selectedTags = [...tags, tag].join(', ');
      }
    } else {
      selectedTags = tag;
    }
    updateUrl();
  }

  function updateUrl() {
    const params = new URLSearchParams();
    if (searchQuery) params.set('q', searchQuery);
    if (selectedLanguage) params.set('language', selectedLanguage);
    if (selectedTags) params.set('tags', selectedTags);
    
    const query = params.toString();
    goto(query ? `/search?${query}` : '/search');
  }

  function goToPage(pageNum: number) {
    loadSnippets(pageNum);
  }

  onMount(() => {
    loadSnippets();
    loadFilters();
  });

  $: {
    loadSnippets();
  }
</script>

<div class="search-page">
  <div class="search-header">
    <h1>Search Snippets</h1>
    <form class="search-form" on:submit={handleSearch}>
      <div class="search-input-wrapper">
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search by keywords, title, or description..."
        />
        <button type="submit" class="search-btn">🔍</button>
      </div>
    </form>
  </div>

  <div class="search-content">
    <aside class="filters-sidebar">
      <div class="filter-section">
        <h3>💻 Language</h3>
        <div class="filter-options">
          <button 
            class="filter-btn {!selectedLanguage ? 'active' : ''}"
            on:click={() => handleLanguageChange('')}
          >
            All
          </button>
          {#each languages.slice(0, 10) as lang}
            <button 
              class="filter-btn {selectedLanguage === lang ? 'active' : ''}"
              on:click={() => handleLanguageChange(lang)}
            >
              {lang}
            </button>
          {/each}
        </div>
      </div>

      <div class="filter-section">
        <h3>🏷️ Popular Tags</h3>
        <div class="filter-options tags">
          {#each popularTags.slice(0, 15) as tag}
            <button 
              class="tag-btn {selectedTags.split(',').map(t => t.trim()).includes(tag.name) ? 'active' : ''}"
              on:click={() => handleTagClick(tag.name)}
            >
              #{tag.name}
              <span class="tag-count">({tag.count})</span>
            </button>
          {/each}
        </div>
      </div>

      {#if selectedLanguage || selectedTags || searchQuery}
        <button class="clear-filters-btn" on:click={() => {
          searchQuery = '';
          selectedLanguage = '';
          selectedTags = '';
          updateUrl();
        }}>
          ✕ Clear Filters
        </button>
      {/if}
    </aside>

    <main class="results-main">
      {#if loading}
        <div class="loading">Loading...</div>
      {:else if snippets.length > 0}
        <div class="snippets-grid">
          {#each snippets as snippet}
            <SnippetCard {snippet} />
          {/each}
        </div>

        {#if totalPages > 1}
          <div class="pagination">
            {#if currentPage > 1}
              <button class="page-btn" on:click={() => goToPage(currentPage - 1)}>← Prev</button>
            {/if}
            
            <span class="page-info">Page {currentPage} of {totalPages}</span>
            
            {#if currentPage < totalPages}
              <button class="page-btn" on:click={() => goToPage(currentPage + 1)}>Next →</button>
            {/if}
          </div>
        {/if}
      {:else}
        <div class="empty-state">
          <h3>No snippets found</h3>
          <p>Try adjusting your search or filters</p>
          <a href="/search" class="btn btn-secondary">Clear all filters</a>
        </div>
      {/if}
    </main>
  </div>
</div>

<style>
  .search-page {
    max-width: 1400px;
    margin: 0 auto;
  }

  .search-header {
    margin-bottom: 32px;
  }

  .search-header h1 {
    margin: 0 0 16px 0;
    font-size: 32px;
    font-weight: 600;
  }

  .search-form {
    max-width: 600px;
  }

  .search-input-wrapper {
    position: relative;
  }

  .search-input-wrapper input {
    width: 100%;
    padding: 14px 50px 14px 16px;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 8px;
    color: #fff;
    font-size: 15px;
  }

  .search-input-wrapper input:focus {
    outline: none;
    border-color: #007acc;
  }

  .search-btn {
    position: absolute;
    right: 12px;
    top: 50%;
    transform: translateY(-50%);
    background: none;
    border: none;
    cursor: pointer;
    font-size: 18px;
  }

  .search-content {
    display: grid;
    grid-template-columns: 280px 1fr;
    gap: 32px;
  }

  .filters-sidebar {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .filter-section {
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 20px;
  }

  .filter-section h3 {
    margin: 0 0 16px 0;
    font-size: 16px;
    font-weight: 600;
  }

  .filter-options {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .filter-options.tags {
    flex-direction: row;
    flex-wrap: wrap;
  }

  .filter-btn {
    padding: 8px 12px;
    background: #252526;
    border: 1px solid #333;
    border-radius: 4px;
    color: #ccc;
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: all 0.2s;
  }

  .filter-btn:hover {
    background: #333;
    border-color: #444;
  }

  .filter-btn.active {
    background: #007acc;
    border-color: #007acc;
    color: #fff;
  }

  .tag-btn {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 6px 10px;
    background: rgba(0, 122, 204, 0.1);
    border: 1px solid rgba(0, 122, 204, 0.3);
    border-radius: 4px;
    color: #007acc;
    font-size: 12px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .tag-btn:hover {
    background: rgba(0, 122, 204, 0.2);
  }

  .tag-btn.active {
    background: #007acc;
    color: #fff;
  }

  .tag-count {
    color: #858585;
    font-size: 11px;
  }

  .tag-btn.active .tag-count {
    color: rgba(255, 255, 255, 0.8);
  }

  .clear-filters-btn {
    padding: 10px 16px;
    background: #252526;
    border: 1px solid #333;
    border-radius: 4px;
    color: #f44336;
    font-size: 13px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .clear-filters-btn:hover {
    background: rgba(244, 67, 54, 0.1);
    border-color: rgba(244, 67, 54, 0.3);
  }

  .results-main {
    min-height: 400px;
  }

  .loading {
    text-align: center;
    padding: 60px;
    color: #858585;
    font-size: 16px;
  }

  .snippets-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
    gap: 20px;
  }

  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 16px;
    margin-top: 32px;
    padding-top: 24px;
    border-top: 1px solid #333;
  }

  .page-btn {
    padding: 10px 16px;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 4px;
    color: #ccc;
    font-size: 14px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .page-btn:hover {
    background: #333;
    border-color: #444;
  }

  .page-info {
    color: #858585;
    font-size: 14px;
  }

  .empty-state {
    text-align: center;
    padding: 60px 20px;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 8px;
  }

  .empty-state h3 {
    margin: 0 0 8px 0;
    font-size: 20px;
  }

  .empty-state p {
    margin: 0 0 20px 0;
    color: #858585;
  }

  .btn {
    display: inline-block;
    padding: 10px 20px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    text-decoration: none;
    cursor: pointer;
    border: none;
    transition: all 0.2s;
  }

  .btn-secondary {
    background: #333;
    color: #ccc;
  }

  .btn-secondary:hover {
    background: #444;
    color: #fff;
  }

  @media (max-width: 900px) {
    .search-content {
      grid-template-columns: 1fr;
    }

    .filters-sidebar {
      flex-direction: row;
      flex-wrap: wrap;
    }

    .filter-section {
      flex: 1;
      min-width: 200px;
    }

    .snippets-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
