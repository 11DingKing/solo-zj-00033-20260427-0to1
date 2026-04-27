<script lang="ts">
  import { page, goto } from '$app/stores';
  import { onMount } from 'svelte';
  import SnippetCard from '$lib/components/SnippetCard.svelte';
  import { api } from '$lib/api';
  import type { Snippet, PaginatedResponse, User } from '$lib/types';

  let user: User | null = null;
  let snippets: Snippet[] = [];
  let activeTab: 'snippets' | 'forks' = 'snippets';
  let loading: boolean = true;

  $: userId = $page.params.id;

  async function loadUser() {
    try {
      const response = await api.get(`/api/users/${userId}`);
      if (response.ok) {
        user = await response.json();
      }
    } catch (e) {
      console.error('Failed to load user:', e);
    }
  }

  async function loadSnippets() {
    try {
      const endpoint = activeTab === 'forks' 
        ? `/api/users/${userId}/forks?per_page=12`
        : `/api/users/${userId}/snippets?per_page=12`;
      
      const response = await api.get(endpoint);
      if (response.ok) {
        const data: PaginatedResponse<Snippet> = await response.json();
        snippets = data.data;
      }
    } catch (e) {
      console.error('Failed to load snippets:', e);
    } finally {
      loading = false;
    }
  }

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  }

  onMount(() => {
    loadUser();
    loadSnippets();
  });

  $: {
    if (activeTab) {
      loadSnippets();
    }
  }
</script>

<div class="user-profile-page">
  {#if loading}
    <div class="loading">Loading...</div>
  {:else if user}
    <div class="profile-header">
      <div class="profile-info">
        <div class="avatar">
          {user.display_name?.charAt(0)?.toUpperCase() || user.username.charAt(0).toUpperCase()}
        </div>
        <div class="profile-details">
          <h1 class="display-name">
            {user.display_name || user.username}
          </h1>
          <p class="username">@{user.username}</p>
          <p class="joined">
            Member since {formatDate(user.created_at)}
          </p>
        </div>
      </div>
    </div>

    <div class="tabs">
      <button 
        class="tab-btn {activeTab === 'snippets' ? 'active' : ''}"
        on:click={() => activeTab = 'snippets'}
      >
        📝 Snippets
      </button>
      <button 
        class="tab-btn {activeTab === 'forks' ? 'active' : ''}"
        on:click={() => activeTab = 'forks'}
      >
        🍴 Forks
      </button>
    </div>

    <div class="content">
      {#if snippets.length > 0}
        <div class="snippets-grid">
          {#each snippets as snippet}
            <SnippetCard {snippet} />
          {/each}
        </div>
      {:else}
        <div class="empty-state">
          <h3>{activeTab === 'forks' ? 'No forks yet' : 'No snippets yet'}</h3>
          <p>
            {activeTab === 'forks' 
              ? 'This user hasn\'t forked any snippets yet'
              : 'This user hasn\'t created any snippets yet'
            }
          </p>
        </div>
      {/if}
    </div>
  {:else}
    <div class="error-state">
      <h2>User not found</h2>
      <p>The user you're looking for doesn't exist.</p>
      <a href="/" class="btn btn-secondary">Go Home</a>
    </div>
  {/if}
</div>

<style>
  .user-profile-page {
    max-width: 1200px;
    margin: 0 auto;
  }

  .loading {
    text-align: center;
    padding: 80px;
    color: #858585;
    font-size: 16px;
  }

  .profile-header {
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 32px;
    margin-bottom: 24px;
  }

  .profile-info {
    display: flex;
    gap: 24px;
    align-items: center;
  }

  .avatar {
    width: 120px;
    height: 120px;
    background: linear-gradient(135deg, #007acc, #005a9e);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 48px;
    font-weight: 600;
    color: #fff;
  }

  .profile-details {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .display-name {
    margin: 0;
    font-size: 32px;
    font-weight: 700;
    color: #fff;
  }

  .username {
    margin: 0;
    font-size: 18px;
    color: #858585;
  }

  .joined {
    margin: 0;
    font-size: 14px;
    color: #858585;
    margin-top: 8px;
  }

  .tabs {
    display: flex;
    gap: 16px;
    margin-bottom: 24px;
    border-bottom: 1px solid #333;
  }

  .tab-btn {
    padding: 12px 24px;
    background: none;
    border: none;
    color: #858585;
    font-size: 15px;
    font-weight: 500;
    cursor: pointer;
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
    transition: all 0.2s;
  }

  .tab-btn:hover {
    color: #ccc;
  }

  .tab-btn.active {
    color: #007acc;
    border-bottom-color: #007acc;
  }

  .snippets-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(400px, 1fr));
    gap: 24px;
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
    margin: 0;
    color: #858585;
  }

  .error-state {
    text-align: center;
    padding: 80px 20px;
  }

  .error-state h2 {
    margin: 0 0 12px 0;
    font-size: 28px;
    color: #f44336;
  }

  .error-state p {
    margin: 0 0 24px 0;
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

  .btn-secondary {
    background: #333;
    color: #ccc;
  }

  .btn-secondary:hover {
    background: #444;
    color: #fff;
  }

  @media (max-width: 768px) {
    .profile-info {
      flex-direction: column;
      text-align: center;
    }

    .snippets-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
