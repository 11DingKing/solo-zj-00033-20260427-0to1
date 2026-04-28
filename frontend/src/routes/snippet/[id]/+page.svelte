<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import CodeEditor from '$lib/components/CodeEditor.svelte';
  import { api, auth } from '$lib/api';
  import type { Snippet, Version, Comment, SnippetFile, User } from '$lib/types';
  import { getLanguageLabel } from '$lib/types';
  import { marked } from 'marked';
  import Prism from 'prismjs';
  import 'prismjs/components/prism-javascript';
  import 'prismjs/components/prism-typescript';
  import 'prismjs/components/prism-python';
  import 'prismjs/components/prism-rust';
  import 'prismjs/components/prism-go';
  import 'prismjs/components/prism-java';
  import 'prismjs/components/prism-csharp';
  import 'prismjs/components/prism-cpp';
  import 'prismjs/components/prism-php';
  import 'prismjs/components/prism-ruby';
  import 'prismjs/components/prism-swift';
  import 'prismjs/components/prism-sql';
  import 'prismjs/components/prism-json';
  import 'prismjs/components/prism-css';
  import 'prismjs/components/prism-markup';
  import 'prismjs/components/prism-bash';

  let snippet: Snippet | null = null;
  let versions: Version[] = [];
  let comments: Comment[] = [];
  let activeFileIndex: number = 0;
  let newComment: string = '';
  let loading: boolean = true;
  let error: string = '';
  let liked: boolean = false;
  let submittingComment: boolean = false;
  let rollingBack: boolean = false;
  let selectedVersionForCompare: number | null = null;

  $: snippetId = $page.params.id;

  marked.setOptions({
    highlight: function(code, lang) {
      if (Prism.languages[lang]) {
        return Prism.highlight(code, Prism.languages[lang], lang);
      }
      return code;
    },
    breaks: true,
    gfm: true
  });

  function renderMarkdown(content: string): string {
    try {
      return marked.parse(content) as string;
    } catch (e) {
      return content;
    }
  }

  async function loadSnippet() {
    try {
      const [snippetRes, versionsRes, commentsRes] = await Promise.all([
        api.get(`/api/snippets/${snippetId}`, $auth.token),
        api.get(`/api/snippets/${snippetId}/versions`),
        api.get(`/api/snippets/${snippetId}/comments`)
      ]);

      if (snippetRes.ok) {
        snippet = await snippetRes.json();
      } else if (snippetRes.status === 403) {
        error = 'This snippet is private';
      } else if (snippetRes.status === 404) {
        error = 'Snippet not found';
      }

      if (versionsRes.ok) {
        versions = await versionsRes.json();
      }

      if (commentsRes.ok) {
        comments = await commentsRes.json();
      }

      if (snippet && $auth.token) {
        try {
          const likeRes = await api.get(`/api/snippets/${snippetId}/like`, $auth.token);
          if (likeRes.ok) {
            const data = await likeRes.json();
            liked = data.liked;
          }
        } catch (e) {
          // Ignore like check errors
        }
      }
    } catch (e) {
      error = 'Failed to load snippet';
      console.error(e);
    } finally {
      loading = false;
    }
  }

  async function handleLike() {
    if (!$auth.token) {
      goto('/login');
      return;
    }

    if (!snippet) return;

    try {
      const response = await api.post(`/api/snippets/${snippetId}/like`, {}, $auth.token);
      if (response.ok) {
        const data = await response.json();
        liked = data.liked;
        snippet.likes_count = data.likes_count;
      }
    } catch (e) {
      console.error('Failed to like:', e);
    }
  }

  async function handleFork() {
    if (!$auth.token) {
      goto('/login');
      return;
    }

    if (!snippet) return;

    try {
      const response = await api.post(`/api/snippets/${snippetId}/fork`, {}, $auth.token);
      if (response.ok) {
        const data = await response.json();
        goto(`/snippet/${data.id}`);
      }
    } catch (e) {
      console.error('Failed to fork:', e);
    }
  }

  async function handleComment(e: SubmitEvent) {
    e.preventDefault();
    if (!$auth.token) {
      goto('/login');
      return;
    }

    if (!newComment.trim()) return;

    submittingComment = true;
    try {
      const response = await api.post(
        `/api/snippets/${snippetId}/comments`,
        { content: newComment.trim() },
        $auth.token
      );

      if (response.ok) {
        const comment = await response.json();
        comments = [...comments, comment];
        newComment = '';
      }
    } catch (e) {
      console.error('Failed to comment:', e);
    } finally {
      submittingComment = false;
    }
  }

  function handleCompare(versionNumber: number) {
    if (selectedVersionForCompare === null) {
      selectedVersionForCompare = versionNumber;
    } else if (selectedVersionForCompare === versionNumber) {
      selectedVersionForCompare = null;
    } else {
      const v1 = Math.min(selectedVersionForCompare, versionNumber);
      const v2 = Math.max(selectedVersionForCompare, versionNumber);
      goto(`/snippet/${snippetId}/diff?v_a=${v1}&v_b=${v2}`);
      selectedVersionForCompare = null;
    }
  }

  async function handleRollback(version: Version) {
    if (!$auth.token) {
      goto('/login');
      return;
    }

    if (!snippet || $auth.user?.id !== snippet.user.id) {
      return;
    }

    if (!confirm(`Are you sure you want to rollback to version ${version.version_number}?`)) {
      return;
    }

    rollingBack = true;
    try {
      const versionRes = await api.get(`/api/snippets/${snippetId}/versions/${version.version_number}`);
      if (!versionRes.ok) {
        throw new Error('Failed to get version data');
      }
      const versionData = await versionRes.json();

      const updatePayload = {
        files: versionData.files.map((f: SnippetFile) => ({
          filename: f.filename,
          content: f.content,
          language: f.language || undefined
        })),
        commit_message: `Rollback to version ${version.version_number}: ${version.commit_message || 'No message'}`
      };

      const response = await api.put(
        `/api/snippets/${snippetId}`,
        updatePayload,
        $auth.token
      );

      if (response.ok) {
        loadSnippet();
      }
    } catch (e) {
      console.error('Failed to rollback:', e);
      error = 'Failed to rollback. Please try again.';
    } finally {
      rollingBack = false;
    }
  }

  function formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }

  function getEmbedUrl(): string {
    return `${import.meta.env.VITE_API_URL || 'http://localhost:8080'}/embed/${snippetId}`;
  }

  onMount(() => {
    loadSnippet();
  });
</script>

<div class="snippet-detail-page">
  {#if loading}
    <div class="loading">Loading snippet...</div>
  {:else if error}
    <div class="error-state">
      <h2>{error}</h2>
      <a href="/" class="btn btn-secondary">Go Home</a>
    </div>
  {:else if snippet}
    <div class="snippet-header">
      <div class="snippet-title-bar">
        <div class="snippet-info">
          <h1 class="snippet-title">{snippet.title}</h1>
          <div class="snippet-meta">
            <a href={`/user/${snippet.user.id}`} class="author">
              {snippet.user.display_name || snippet.user.username}
            </a>
            <span class="separator">·</span>
            <span class="language">{getLanguageLabel(snippet.language)}</span>
            <span class="separator">·</span>
            <span class="timestamp">{formatDate(snippet.created_at)}</span>
            {#if snippet.parent_id}
              <span class="separator">·</span>
              <a href={`/snippet/${snippet.parent_id}`} class="forked-from">Forked</a>
            {/if}
          </div>
        </div>
        <div class="snippet-actions">
          {#if $auth.user?.id === snippet.user.id}
            <a href={`/snippet/${snippet.id}/edit`} class="btn btn-secondary">Edit</a>
          {/if}
          <button 
            class="btn {liked ? 'btn-liked' : 'btn-secondary'}"
            on:click={handleLike}
          >
            ❤️ {snippet.likes_count}
          </button>
          <button class="btn btn-secondary" on:click={handleFork}>
            🍴 Fork
          </button>
        </div>
      </div>

      {#if snippet.description}
        <div class="snippet-description">
          {@html renderMarkdown(snippet.description)}
        </div>
      {/if}

      {#if snippet.tags.length > 0}
        <div class="snippet-tags">
          {#each snippet.tags as tag}
            <a href={`/search?tags=${tag}`} class="tag">#{tag}</a>
          {/each}
        </div>
      {/if}

      <div class="snippet-stats">
        <span class="stat">👁️ {snippet.views_count} views</span>
        <span class="stat">🍴 {snippet.forks_count} forks</span>
        <span class="stat">📄 {snippet.files.length} files</span>
        <span class="stat">
          🔗 Embed: 
          <code class="embed-url">{getEmbedUrl()}</code>
        </span>
      </div>
    </div>

    {#if snippet.files.length > 0}
      <div class="code-section">
        <div class="file-tabs">
          {#each snippet.files as file, index}
            <button 
              class="file-tab {activeFileIndex === index ? 'active' : ''}"
              on:click={() => activeFileIndex = index}
            >
              {file.filename}
            </button>
          {/each}
        </div>

        <div class="code-editor">
          <CodeEditor
            value={snippet.files[activeFileIndex].content}
            language={snippet.files[activeFileIndex].language || snippet.language}
            readOnly={true}
          />
        </div>
      </div>
    {/if}

    {#if versions.length > 0}
      <div class="section">
        <h2 class="section-title">
          📜 Version History
          {#if selectedVersionForCompare !== null}
            <span class="compare-hint">
              (Select another version to compare with v{selectedVersionForCompare})
              <button class="clear-compare-btn" on:click={() => selectedVersionForCompare = null}>✕ Clear</button>
            </span>
          {/if}
        </h2>
        <div class="versions-list">
          {#each versions as version}
            <div class="version-item {selectedVersionForCompare === version.version_number ? 'selected-for-compare' : ''}">
              <div class="version-info">
                <span class="version-number">v{version.version_number}</span>
                <span class="version-message">
                  {version.commit_message || 'No message'}
                </span>
              </div>
              <div class="version-actions">
                <span class="version-date">{formatDate(version.created_at)}</span>
                <button 
                  class="btn btn-small {selectedVersionForCompare === version.version_number ? 'btn-primary' : 'btn-secondary'}"
                  on:click={() => handleCompare(version.version_number)}
                  disabled={rollingBack}
                >
                  {selectedVersionForCompare === version.version_number ? 'Selected' : 'Compare'}
                </button>
                {#if $auth.user?.id === snippet.user.id}
                  <button 
                    class="btn btn-small btn-warning"
                    on:click={() => handleRollback(version)}
                    disabled={rollingBack}
                  >
                    {rollingBack ? 'Rolling back...' : 'Rollback'}
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <div class="section">
      <h2 class="section-title">💬 Comments ({comments.length})</h2>
      
      {#if $auth.token}
        <form class="comment-form" on:submit={handleComment}>
          <textarea
            bind:value={newComment}
            placeholder="Add a comment (Markdown supported)..."
            rows={3}
            disabled={submittingComment}
          />
          <button type="submit" class="btn btn-primary" disabled={submittingComment || !newComment.trim()}>
            {submittingComment ? 'Posting...' : 'Post Comment'}
          </button>
        </form>
      {:else}
        <div class="login-prompt">
          <a href="/login">Login</a> to comment
        </div>
      {/if}

      <div class="comments-list">
        {#each comments as comment}
          <div class="comment">
            <div class="comment-header">
              <a href={`/user/${comment.user.id}`} class="comment-author">
                {comment.user.display_name || comment.user.username}
              </a>
              <span class="comment-date">{formatDate(comment.created_at)}</span>
            </div>
            <div class="comment-content markdown-content">
              {@html renderMarkdown(comment.content)}
            </div>
          </div>
        {/each}

        {#if comments.length === 0}
          <div class="empty-state">No comments yet. Be the first!</div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .snippet-detail-page {
    max-width: 1200px;
    margin: 0 auto;
  }

  .loading {
    text-align: center;
    padding: 80px;
    color: #858585;
    font-size: 16px;
  }

  .error-state {
    text-align: center;
    padding: 80px;
  }

  .error-state h2 {
    margin: 0 0 20px 0;
    color: #f44336;
  }

  .snippet-header {
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 24px;
    margin-bottom: 24px;
  }

  .snippet-title-bar {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 24px;
    margin-bottom: 16px;
  }

  .snippet-info {
    flex: 1;
  }

  .snippet-title {
    margin: 0 0 8px 0;
    font-size: 28px;
    font-weight: 600;
    color: #569cd6;
  }

  .snippet-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    font-size: 14px;
    color: #858585;
  }

  .author {
    color: #4ec9b0;
    text-decoration: none;
    font-weight: 500;
  }

  .author:hover {
    text-decoration: underline;
  }

  .separator {
    color: #555;
  }

  .language {
    background: #333;
    padding: 2px 8px;
    border-radius: 4px;
    color: #ce9178;
    font-weight: 500;
  }

  .forked-from {
    color: #007acc;
    text-decoration: none;
  }

  .forked-from:hover {
    text-decoration: underline;
  }

  .snippet-actions {
    display: flex;
    gap: 8px;
  }

  .snippet-description {
    padding: 16px;
    background: #252526;
    border-radius: 6px;
    margin-bottom: 16px;
    line-height: 1.6;
    color: #ccc;
  }

  .snippet-description :global(p) {
    margin: 0 0 10px 0;
  }

  .snippet-description :global(p:last-child) {
    margin-bottom: 0;
  }

  .snippet-description :global(code) {
    background: #1e1e1e;
    padding: 2px 6px;
    border-radius: 4px;
    font-family: monospace;
    font-size: 13px;
    color: #ce9178;
  }

  .snippet-description :global(pre) {
    background: #1e1e1e;
    padding: 12px;
    border-radius: 6px;
    overflow-x: auto;
  }

  .snippet-description :global(pre code) {
    background: none;
    padding: 0;
  }

  .snippet-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-bottom: 16px;
  }

  .tag {
    background: rgba(0, 122, 204, 0.15);
    color: #007acc;
    padding: 4px 10px;
    border-radius: 4px;
    text-decoration: none;
    font-size: 13px;
  }

  .tag:hover {
    background: rgba(0, 122, 204, 0.25);
  }

  .snippet-stats {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    padding-top: 16px;
    border-top: 1px solid #333;
    font-size: 13px;
    color: #858585;
  }

  .stat {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .embed-url {
    background: #252526;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 12px;
    color: #ccc;
  }

  .code-section {
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 8px;
    overflow: hidden;
    margin-bottom: 24px;
  }

  .file-tabs {
    display: flex;
    background: #252526;
    border-bottom: 1px solid #333;
    overflow-x: auto;
  }

  .file-tab {
    background: none;
    border: none;
    padding: 12px 20px;
    color: #858585;
    cursor: pointer;
    font-size: 14px;
    white-space: nowrap;
  }

  .file-tab:hover {
    background: #2d2d2d;
    color: #ccc;
  }

  .file-tab.active {
    background: #1e1e1e;
    color: #fff;
    border-bottom: 2px solid #007acc;
  }

  .code-editor {
    min-height: 400px;
    max-height: 600px;
    overflow: auto;
  }

  .section {
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 24px;
    margin-bottom: 24px;
  }

  .section-title {
    margin: 0 0 20px 0;
    font-size: 18px;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 12px;
    flex-wrap: wrap;
  }

  .compare-hint {
    font-size: 13px;
    font-weight: 400;
    color: #858585;
  }

  .clear-compare-btn {
    background: rgba(244, 67, 54, 0.1);
    border: 1px solid rgba(244, 67, 54, 0.3);
    color: #f44336;
    padding: 4px 10px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 12px;
  }

  .clear-compare-btn:hover {
    background: rgba(244, 67, 54, 0.2);
  }

  .versions-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .version-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: #252526;
    border-radius: 6px;
    border: 2px solid transparent;
  }

  .version-item.selected-for-compare {
    border-color: #007acc;
    background: rgba(0, 122, 204, 0.1);
  }

  .version-info {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .version-number {
    background: #007acc;
    color: #fff;
    padding: 4px 10px;
    border-radius: 4px;
    font-weight: 600;
    font-size: 13px;
  }

  .version-message {
    color: #ccc;
  }

  .version-date {
    color: #858585;
    font-size: 13px;
  }

  .version-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .comment-form {
    display: flex;
    flex-direction: column;
    gap: 12px;
    margin-bottom: 24px;
  }

  .comment-form textarea {
    padding: 12px 16px;
    background: #252526;
    border: 1px solid #333;
    border-radius: 6px;
    color: #fff;
    font-size: 14px;
    font-family: inherit;
    resize: vertical;
    min-height: 80px;
  }

  .comment-form textarea:focus {
    outline: none;
    border-color: #007acc;
  }

  .login-prompt {
    padding: 16px;
    background: #252526;
    border-radius: 6px;
    text-align: center;
    margin-bottom: 24px;
    color: #858585;
  }

  .login-prompt a {
    color: #007acc;
    text-decoration: none;
  }

  .comments-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .comment {
    padding: 16px;
    background: #252526;
    border-radius: 6px;
  }

  .comment-header {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-bottom: 8px;
  }

  .comment-author {
    color: #4ec9b0;
    text-decoration: none;
    font-weight: 500;
    font-size: 14px;
  }

  .comment-author:hover {
    text-decoration: underline;
  }

  .comment-date {
    color: #858585;
    font-size: 12px;
  }

  .comment-content {
    color: #ccc;
    line-height: 1.6;
  }

  .comment-content :global(p) {
    margin: 0 0 10px 0;
  }

  .comment-content :global(p:last-child) {
    margin-bottom: 0;
  }

  .comment-content :global(code) {
    background: #1e1e1e;
    padding: 2px 6px;
    border-radius: 4px;
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
    font-size: 13px;
    color: #ce9178;
  }

  .comment-content :global(pre) {
    background: #1e1e1e;
    padding: 12px;
    border-radius: 6px;
    overflow-x: auto;
    margin: 10px 0;
  }

  .comment-content :global(pre code) {
    background: none;
    padding: 0;
    display: block;
    white-space: pre;
  }

  .comment-content :global(.token.comment),
  .comment-content :global(.token.prolog),
  .comment-content :global(.token.doctype),
  .comment-content :global(.token.cdata) {
    color: #6a9955;
  }

  .comment-content :global(.token.punctuation) {
    color: #d4d4d4;
  }

  .comment-content :global(.token.property),
  .comment-content :global(.token.tag),
  .comment-content :global(.token.boolean),
  .comment-content :global(.token.number),
  .comment-content :global(.token.constant),
  .comment-content :global(.token.symbol),
  .comment-content :global(.token.deleted) {
    color: #b5cea8;
  }

  .comment-content :global(.token.selector),
  .comment-content :global(.token.attr-name),
  .comment-content :global(.token.string),
  .comment-content :global(.token.char),
  .comment-content :global(.token.builtin),
  .comment-content :global(.token.inserted) {
    color: #ce9178;
  }

  .comment-content :global(.token.operator),
  .comment-content :global(.token.entity),
  .comment-content :global(.token.url) {
    color: #d4d4d4;
  }

  .comment-content :global(.token.atrule),
  .comment-content :global(.token.attr-value),
  .comment-content :global(.token.keyword) {
    color: #569cd6;
  }

  .comment-content :global(.token.function),
  .comment-content :global(.token.class-name) {
    color: #dcdcaa;
  }

  .comment-content :global(.token.regex),
  .comment-content :global(.token.important),
  .comment-content :global(.token.variable) {
    color: #d16969;
  }

  .comment-content :global(blockquote) {
    border-left: 4px solid #007acc;
    padding-left: 12px;
    margin: 10px 0;
    color: #858585;
    font-style: italic;
  }

  .comment-content :global(ul),
  .comment-content :global(ol) {
    padding-left: 20px;
    margin: 10px 0;
  }

  .comment-content :global(li) {
    margin: 4px 0;
  }

  .comment-content :global(a) {
    color: #007acc;
    text-decoration: none;
  }

  .comment-content :global(a:hover) {
    text-decoration: underline;
  }

  .comment-content :global(h1),
  .comment-content :global(h2),
  .comment-content :global(h3),
  .comment-content :global(h4),
  .comment-content :global(h5),
  .comment-content :global(h6) {
    margin: 10px 0;
    color: #fff;
  }

  .comment-content :global(hr) {
    border: none;
    border-top: 1px solid #333;
    margin: 10px 0;
  }

  .comment-content :global(table) {
    border-collapse: collapse;
    width: 100%;
    margin: 10px 0;
  }

  .comment-content :global(th),
  .comment-content :global(td) {
    border: 1px solid #333;
    padding: 8px 12px;
    text-align: left;
  }

  .comment-content :global(th) {
    background: #252526;
  }

  .empty-state {
    text-align: center;
    padding: 32px;
    color: #858585;
    background: #252526;
    border: 1px dashed #333;
    border-radius: 6px;
  }

  .btn {
    padding: 10px 18px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    cursor: pointer;
    border: none;
    transition: all 0.2s;
  }

  .btn-small {
    padding: 6px 12px;
    font-size: 13px;
  }

  .btn-primary {
    background: #007acc;
    color: #fff;
  }

  .btn-primary:hover:not(:disabled) {
    background: #005a9e;
  }

  .btn-secondary {
    background: #333;
    color: #ccc;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #444;
    color: #fff;
  }

  .btn-warning {
    background: rgba(255, 152, 0, 0.2);
    color: #ff9800;
    border: 1px solid rgba(255, 152, 0, 0.3);
  }

  .btn-warning:hover:not(:disabled) {
    background: rgba(255, 152, 0, 0.3);
  }

  .btn-liked {
    background: rgba(244, 67, 54, 0.2);
    color: #f44336;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  @media (max-width: 768px) {
    .snippet-title-bar {
      flex-direction: column;
    }

    .snippet-actions {
      width: 100%;
    }

    .snippet-actions .btn {
      flex: 1;
    }

    .version-item {
      flex-direction: column;
      align-items: flex-start;
      gap: 12px;
    }

    .version-actions {
      width: 100%;
      justify-content: space-between;
    }
  }
</style>
