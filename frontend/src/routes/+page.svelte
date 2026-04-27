<script lang="ts">
  import { onMount } from 'svelte';
  import SnippetCard from '$lib/components/SnippetCard.svelte';
  import { api } from '$lib/api';
  import type { Snippet, PaginatedResponse, Tag } from '$lib/types';

  let hotSnippets: Snippet[] = [];
  let latestSnippets: Snippet[] = [];
  let popularTags: Tag[] = [];
  let languages: string[] = [];
  let loading = true;

  async function loadData() {
    try {
      const [hotRes, latestRes, tagsRes, langRes] = await Promise.all([
        api.get('/api/snippets/hot?per_page=6'),
        api.get('/api/snippets/latest?per_page=6'),
        api.get('/api/search/tags'),
        api.get('/api/search/languages')
      ]);

      if (hotRes.ok) {
        const data: PaginatedResponse<Snippet> = await hotRes.json();
        hotSnippets = data.data;
      }

      if (latestRes.ok) {
        const data: PaginatedResponse<Snippet> = await latestRes.json();
        latestSnippets = data.data;
      }

      if (tagsRes.ok) {
        popularTags = await tagsRes.json();
      }

      if (langRes.ok) {
        languages = await langRes.json();
      }
    } catch (e) {
      console.error('Failed to load data:', e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadData();
  });
</script>

<div class="home-page">
  <section class="hero">
    <div class="hero-content">
      <h1>Share Your Code Instantly</h1>
      <p>Create, collaborate, and discover code snippets with syntax highlighting, version control, and more.</p>
      <div class="hero-actions">
        <a href="/new" class="btn btn-primary">Create New Snippet</a>
        <a href="/latest" class="btn btn-secondary">Browse Snippets</a>
      </div>
    </div>
    <div class="hero-features">
      <div class="feature">
        <span class="feature-icon">📝</span>
        <h3>Multi-File Snippets</h3>
        <p>Create snippets with multiple files, just like a real project.</p>
      </div>
      <div class="feature">
        <span class="feature-icon">🔄</span>
        <h3>Version Control</h3>
        <p>Track changes, compare versions, and roll back with ease.</p>
      </div>
      <div class="feature">
        <span class="feature-icon">🔍</span>
        <h3>Full-Text Search</h3>
        <p>Find snippets by keywords, language, tags, or code content.</p>
      </div>
    </div>
  </section>

  {#if loading}
    <div class="loading">Loading...</div>
  {:else}
    <section class="snippets-section">
      <div class="section-header">
        <h2>🔥 Hot Snippets</h2>
        <a href="/hot" class="view-all">View All →</a>
      </div>
      {#if hotSnippets.length > 0}
        <div class="snippets-grid">
          {#each hotSnippets as snippet}
            <SnippetCard {snippet} />
          {/each}
        </div>
      {:else}
        <div class="empty-state">No hot snippets yet. Be the first!</div>
      {/if}
    </section>

    <section class="snippets-section">
      <div class="section-header">
        <h2>🆕 Latest Snippets</h2>
        <a href="/latest" class="view-all">View All →</a>
      </div>
      {#if latestSnippets.length > 0}
        <div class="snippets-grid">
          {#each latestSnippets as snippet}
            <SnippetCard {snippet} />
          {/each}
        </div>
      {:else}
        <div class="empty-state">No latest snippets yet. Be the first!</div>
      {/if}
    </section>

    <section class="sidebar-section">
      <div class="sidebar-card">
        <h3>🏷️ Popular Tags</h3>
        {#if popularTags.length > 0}
          <div class="tags-cloud">
            {#each popularTags.slice(0, 15) as tag}
              <a href="/search?tags={tag.name}" class="tag">
                #{tag.name}
                <span class="tag-count">({tag.count})</span>
              </a>
            {/each}
          </div>
        {:else}
          <div class="empty-state">No tags yet</div>
        {/if}
      </div>

      <div class="sidebar-card">
        <h3>💻 Languages</h3>
        {#if languages.length > 0}
          <div class="languages-list">
            {#each languages.slice(0, 10) as lang}
              <a href="/search?language={lang}" class="language-item">
                {lang}
              </a>
            {/each}
          </div>
        {:else}
          <div class="empty-state">No languages yet</div>
        {/if}
      </div>
    </section>
  {/if}
</div>

<style>
  .home-page {
    display: flex;
    flex-direction: column;
    gap: 32px;
  }

  .hero {
    background: linear-gradient(135deg, #1e3c72 0%, #2a5298 100%);
    border-radius: 12px;
    padding: 48px;
    text-align: center;
  }

  .hero h1 {
    margin: 0 0 16px 0;
    font-size: 42px;
    font-weight: 700;
    color: #fff;
  }

  .hero p {
    margin: 0 0 24px 0;
    font-size: 18px;
    color: rgba(255, 255, 255, 0.85);
  }

  .hero-actions {
    display: flex;
    gap: 16px;
    justify-content: center;
    margin-bottom: 40px;
  }

  .btn {
    padding: 14px 28px;
    border-radius: 8px;
    font-size: 16px;
    font-weight: 600;
    text-decoration: none;
    cursor: pointer;
    border: none;
    transition: all 0.2s;
  }

  .btn-primary {
    background: #fff;
    color: #1e3c72;
  }

  .btn-primary:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
  }

  .btn-secondary {
    background: rgba(255, 255, 255, 0.15);
    color: #fff;
    border: 1px solid rgba(255, 255, 255, 0.3);
  }

  .btn-secondary:hover {
    background: rgba(255, 255, 255, 0.25);
  }

  .hero-features {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 24px;
    margin-top: 40px;
  }

  .feature {
    background: rgba(255, 255, 255, 0.1);
    padding: 24px;
    border-radius: 8px;
  }

  .feature-icon {
    font-size: 32px;
    display: block;
    margin-bottom: 12px;
  }

  .feature h3 {
    margin: 0 0 8px 0;
    font-size: 18px;
    color: #fff;
  }

  .feature p {
    margin: 0;
    font-size: 14px;
    color: rgba(255, 255, 255, 0.8);
    line-height: 1.5;
  }

  .snippets-section {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .section-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .section-header h2 {
    margin: 0;
    font-size: 24px;
    font-weight: 600;
  }

  .view-all {
    color: #007acc;
    text-decoration: none;
    font-size: 14px;
  }

  .view-all:hover {
    text-decoration: underline;
  }

  .snippets-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
    gap: 20px;
  }

  .loading {
    text-align: center;
    padding: 40px;
    color: #858585;
    font-size: 16px;
  }

  .empty-state {
    text-align: center;
    padding: 40px;
    color: #858585;
    background: #1e1e1e;
    border: 1px dashed #333;
    border-radius: 8px;
  }

  .sidebar-section {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 24px;
  }

  .sidebar-card {
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 20px;
  }

  .sidebar-card h3 {
    margin: 0 0 16px 0;
    font-size: 18px;
    font-weight: 600;
  }

  .tags-cloud {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .tag {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    background: rgba(0, 122, 204, 0.15);
    color: #007acc;
    padding: 6px 12px;
    border-radius: 4px;
    font-size: 13px;
    text-decoration: none;
  }

  .tag:hover {
    background: rgba(0, 122, 204, 0.25);
  }

  .tag-count {
    color: #858585;
    font-size: 12px;
  }

  .languages-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }

  .language-item {
    background: #2d2d2d;
    color: #4ec9b0;
    padding: 6px 14px;
    border-radius: 4px;
    font-size: 13px;
    text-decoration: none;
    font-weight: 500;
  }

  .language-item:hover {
    background: #333;
  }

  @media (max-width: 768px) {
    .hero {
      padding: 24px;
    }

    .hero h1 {
      font-size: 28px;
    }

    .hero-actions {
      flex-direction: column;
      align-items: center;
    }

    .hero-features {
      grid-template-columns: 1fr;
    }

    .snippets-grid {
      grid-template-columns: 1fr;
    }

    .sidebar-section {
      grid-template-columns: 1fr;
    }
  }
</style>
