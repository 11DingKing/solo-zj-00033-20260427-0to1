<script lang="ts">
  import { goto } from '$app/navigation';
  import type { Snippet } from '$lib/types';
  import { getLanguageLabel } from '$lib/types';

  export let snippet: Snippet;

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    
    if (days === 0) {
      const hours = Math.floor(diff / (1000 * 60 * 60));
      if (hours === 0) {
        const minutes = Math.floor(diff / (1000 * 60));
        return minutes <= 1 ? 'just now' : `${minutes} minutes ago`;
      }
      return hours === 1 ? '1 hour ago' : `${hours} hours ago`;
    } else if (days < 7) {
      return days === 1 ? '1 day ago' : `${days} days ago`;
    } else if (days < 30) {
      const weeks = Math.floor(days / 7);
      return weeks === 1 ? '1 week ago' : `${weeks} weeks ago`;
    } else {
      return date.toLocaleDateString();
    }
  }

  function getPreviewContent(): string {
    const firstFile = snippet.files[0];
    if (!firstFile) return '';
    const lines = firstFile.content.split('\n');
    return lines.slice(0, 10).join('\n');
  }

  function truncate(text: string, length: number): string {
    if (text.length <= length) return text;
    return text.slice(0, length) + '...';
  }
</script>

<div class="snippet-card" on:click={() => goto(`/snippet/${snippet.id}`)}>
  <div class="card-header">
    <div class="user-info">
      <div class="avatar">
        {snippet.user.display_name?.charAt(0)?.toUpperCase() || snippet.user.username.charAt(0).toUpperCase()}
      </div>
      <div class="user-details">
        <span class="username">{snippet.user.display_name || snippet.user.username}</span>
        <span class="timestamp">{formatDate(snippet.created_at)}</span>
      </div>
    </div>
    <div class="language-badge">{getLanguageLabel(snippet.language)}</div>
  </div>

  <div class="card-body">
    <h3 class="title">{snippet.title}</h3>
    {#if snippet.description}
      <p class="description">{truncate(snippet.description, 150)}</p>
    {/if}
    
    <div class="code-preview">
      <pre><code>{getPreviewContent()}</code></pre>
    </div>

    {#if snippet.tags.length > 0}
      <div class="tags">
        {#each snippet.tags.slice(0, 3) as tag}
          <span class="tag">#{tag}</span>
        {/each}
        {#if snippet.tags.length > 3}
          <span class="tag-count">+{snippet.tags.length - 3} more</span>
        {/if}
      </div>
    {/if}
  </div>

  <div class="card-footer">
    <div class="stats">
      <span class="stat">
        <span class="stat-icon">❤️</span>
        <span class="stat-value">{snippet.likes_count}</span>
      </span>
      <span class="stat">
        <span class="stat-icon">🍴</span>
        <span class="stat-value">{snippet.forks_count}</span>
      </span>
      <span class="stat">
        <span class="stat-icon">👁️</span>
        <span class="stat-value">{snippet.views_count}</span>
      </span>
      {#if snippet.files.length > 1}
        <span class="stat">
          <span class="stat-icon">📄</span>
          <span class="stat-value">{snippet.files.length} files</span>
        </span>
      {/if}
    </div>
  </div>
</div>

<style>
  .snippet-card {
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 8px;
    overflow: hidden;
    cursor: pointer;
    transition: all 0.2s;
  }

  .snippet-card:hover {
    border-color: #007acc;
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: #252526;
    border-bottom: 1px solid #333;
  }

  .user-info {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .avatar {
    width: 32px;
    height: 32px;
    background: #007acc;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    font-weight: 600;
    font-size: 14px;
  }

  .user-details {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .username {
    font-weight: 500;
    color: #fff;
    font-size: 14px;
  }

  .timestamp {
    font-size: 12px;
    color: #858585;
  }

  .language-badge {
    background: #333;
    padding: 4px 10px;
    border-radius: 12px;
    font-size: 12px;
    color: #4ec9b0;
    font-weight: 500;
  }

  .card-body {
    padding: 16px;
  }

  .title {
    margin: 0 0 8px 0;
    font-size: 16px;
    font-weight: 600;
    color: #569cd6;
  }

  .description {
    margin: 0 0 12px 0;
    font-size: 14px;
    color: #ccc;
    line-height: 1.5;
  }

  .code-preview {
    background: #1a1a1a;
    border-radius: 4px;
    padding: 12px;
    margin: 0 0 12px 0;
    overflow: hidden;
  }

  .code-preview pre {
    margin: 0;
    padding: 0;
  }

  .code-preview code {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 13px;
    color: #d4d4d4;
    line-height: 1.4;
    white-space: pre;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tags {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .tag {
    background: rgba(0, 122, 204, 0.15);
    color: #007acc;
    padding: 3px 8px;
    border-radius: 4px;
    font-size: 12px;
  }

  .tag-count {
    color: #858585;
    font-size: 12px;
    padding: 3px 0;
  }

  .card-footer {
    padding: 10px 16px;
    background: #252526;
    border-top: 1px solid #333;
  }

  .stats {
    display: flex;
    gap: 16px;
  }

  .stat {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 13px;
    color: #858585;
  }

  .stat-icon {
    font-size: 14px;
  }

  .stat-value {
    font-weight: 500;
  }
</style>
