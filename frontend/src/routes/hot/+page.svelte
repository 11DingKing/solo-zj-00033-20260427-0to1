<script lang="ts">
  import { onMount } from 'svelte';
  import SnippetCard from '$lib/components/SnippetCard.svelte';
  import { api } from '$lib/api';
  import type { Snippet, PaginatedResponse } from '$lib/types';

  let snippets: Snippet[] = [];
  let loading: boolean = true;
  let totalPages: number = 0;
  let currentPage: number = 1;

  async function loadSnippets(pageNum: number = 1) {
    loading = true;
    try {
      const response = await api.get(`/api/snippets/hot?page=${pageNum}&per_page=12`);
      if (response.ok) {
        const data: PaginatedResponse<Snippet> = await response.json();
        snippets = data.data;
        totalPages = data.total_pages;
        currentPage = data.page;
      }
    } catch (e) {
      console.error('Failed to load hot snippets:', e);
    } finally {
      loading = false;
    }
  }

  function goToPage(pageNum: number) {
    loadSnippets(pageNum);
  }

  onMount(() => {
    loadSnippets();
  });
</script>

<div class="list-page">
  <div class="page-header">
    <h1>🔥 Hot Snippets</h1>
    <p class="subtitle">Most popular snippets sorted by likes</p>
  </div>

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
      <h3>No hot snippets yet</h3>
      <p>Be the first to create an awesome snippet!</p>
      <a href="/new" class="btn btn-primary">Create Snippet</a>
    </div>
  {/if}
</div>

<style>
  .list-page {
    max-width: 1400px;
    margin: 0 auto;
  }

  .page-header {
    margin-bottom: 32px;
  }

  .page-header h1 {
    margin: 0 0 8px 0;
    font-size: 32px;
    font-weight: 600;
  }

  .subtitle {
    margin: 0;
    color: #858585;
    font-size: 16px;
  }

  .loading {
    text-align: center;
    padding: 80px;
    color: #858585;
    font-size: 16px;
  }

  .snippets-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
    gap: 24px;
  }

  .pagination {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 16px;
    margin-top: 40px;
    padding-top: 24px;
    border-top: 1px solid #333;
  }

  .page-btn {
    padding: 12px 20px;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 6px;
    color: #ccc;
    font-size: 14px;
    font-weight: 500;
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
    padding: 80px 20px;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 8px;
  }

  .empty-state h3 {
    margin: 0 0 8px 0;
    font-size: 24px;
  }

  .empty-state p {
    margin: 0 0 20px 0;
    color: #858585;
    font-size: 16px;
  }

  .btn {
    display: inline-block;
    padding: 12px 24px;
    border-radius: 6px;
    font-size: 15px;
    font-weight: 600;
    text-decoration: none;
    cursor: pointer;
    border: none;
    transition: all 0.2s;
  }

  .btn-primary {
    background: #007acc;
    color: #fff;
  }

  .btn-primary:hover {
    background: #005a9e;
  }

  @media (max-width: 768px) {
    .snippets-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
