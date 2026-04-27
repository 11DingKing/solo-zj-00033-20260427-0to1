<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import MultiFileEditor from '$lib/components/MultiFileEditor.svelte';
  import { api, auth } from '$lib/api';
  import type { CreateSnippet, CreateFile, Snippet } from '$lib/types';
  import { LANGUAGES, getLanguageLabel } from '$lib/types';

  export let data: { snippet?: Snippet };

  let title: string = '';
  let description: string = '';
  let language: string = 'javascript';
  let is_public: boolean = true;
  let tagsInput: string = '';
  let files: CreateFile[] = [
    { filename: 'main.js', content: '', language: 'javascript' }
  ];
  let commitMessage: string = '';
  let error: string = '';
  let loading: boolean = false;
  let isEdit: boolean = false;

  $: {
    if (data.snippet) {
      title = data.snippet.title;
      description = data.snippet.description || '';
      language = data.snippet.language;
      is_public = data.snippet.is_public;
      tagsInput = data.snippet.tags.join(', ');
      files = data.snippet.files.map(f => ({
        filename: f.filename,
        content: f.content,
        language: f.language || 'text'
      }));
      isEdit = true;
    }
  }

  function getTags(): string[] {
    return tagsInput
      .split(',')
      .map(t => t.trim())
      .filter(t => t.length > 0);
  }

  async function handleSubmit(e: SubmitEvent) {
    e.preventDefault();
    error = '';

    if (!title.trim()) {
      error = 'Title is required';
      return;
    }

    if (files.length === 0) {
      error = 'At least one file is required';
      return;
    }

    for (const file of files) {
      if (!file.filename.trim()) {
        error = 'All files must have a filename';
        return;
      }
    }

    loading = true;

    try {
      const payload: CreateSnippet = {
        title: title.trim(),
        description: description.trim() || undefined,
        language,
        is_public,
        files: files.map(f => ({
          filename: f.filename,
          content: f.content,
          language: f.language
        })),
        tags: getTags()
      };

      let response;
      if (isEdit && data.snippet) {
        const updatePayload = {
          ...payload,
          commit_message: commitMessage.trim() || `Updated ${title}`
        };
        response = await api.put(
          `/api/snippets/${data.snippet.id}`,
          updatePayload,
          $auth.token
        );
      } else {
        response = await api.post(
          '/api/snippets',
          payload,
          $auth.token
        );
      }

      if (response.ok) {
        const result = await response.json();
        goto(`/snippet/${result.id}`);
      } else {
        const err = await response.json().catch(() => ({}));
        error = err.message || 'Failed to save snippet. Please try again.';
      }
    } catch (e) {
      error = 'Network error. Please try again.';
    } finally {
      loading = false;
    }
  }

  function onFilesChange(newFiles: CreateFile[]) {
    files = newFiles;
  }

  onMount(() => {
    if (!$auth.token) {
      goto('/login');
    }
  });
</script>

<div class="create-page">
  <div class="page-header">
    <h1>{isEdit ? 'Edit Snippet' : 'New Snippet'}</h1>
  </div>

  {#if error}
    <div class="alert alert-error">
      {error}
    </div>
  {/if}

  <form class="create-form" on:submit={handleSubmit}>
    <div class="form-section">
      <div class="form-grid">
        <div class="form-group">
          <label for="title">Title *</label>
          <input
            id="title"
            type="text"
            bind:value={title}
            placeholder="Enter a title for your snippet"
            required
            disabled={loading}
          />
        </div>

        <div class="form-group">
          <label for="language">Language</label>
          <select id="language" bind:value={language} disabled={loading}>
            {#each LANGUAGES as lang}
              <option value={lang.value}>{lang.label}</option>
            {/each}
          </select>
        </div>
      </div>

      <div class="form-group">
        <label for="description">Description</label>
        <textarea
          id="description"
          bind:value={description}
          placeholder="Describe your snippet (optional)"
          rows={3}
          disabled={loading}
        />
      </div>

      <div class="form-group">
        <label for="tags">Tags (comma-separated)</label>
        <input
          id="tags"
          type="text"
          bind:value={tagsInput}
          placeholder="e.g., javascript, react, tutorial"
          disabled={loading}
        />
      </div>

      <div class="form-group">
        <label class="checkbox-label">
          <input type="checkbox" bind:checked={is_public} disabled={loading} />
          Make this snippet public
        </label>
        <span class="hint">
          {is_public ? 'Anyone can view and fork this snippet' : 'Only you can view this snippet'}
        </span>
      </div>

      {#if isEdit}
        <div class="form-group">
          <label for="commitMessage">Commit Message</label>
          <input
            id="commitMessage"
            type="text"
            bind:value={commitMessage}
            placeholder="Describe what changed in this version"
            disabled={loading}
          />
        </div>
      {/if}
    </div>

    <div class="form-section">
      <div class="section-header">
        <h2>Files</h2>
      </div>
      <div class="editor-container">
        <MultiFileEditor
          bind:files
          onFilesChange={onFilesChange}
        />
      </div>
    </div>

    <div class="form-actions">
      <button type="button" class="btn btn-secondary" on:click={() => history.back()} disabled={loading}>
        Cancel
      </button>
      <button type="submit" class="btn btn-primary" disabled={loading}>
        {loading ? 'Saving...' : isEdit ? 'Update Snippet' : 'Create Snippet'}
      </button>
    </div>
  </form>
</div>

<style>
  .create-page {
    max-width: 1000px;
    margin: 0 auto;
  }

  .page-header {
    margin-bottom: 24px;
  }

  .page-header h1 {
    margin: 0;
    font-size: 28px;
    font-weight: 600;
  }

  .alert {
    padding: 12px 16px;
    border-radius: 6px;
    margin-bottom: 20px;
    font-size: 14px;
  }

  .alert-error {
    background: rgba(244, 67, 54, 0.15);
    border: 1px solid rgba(244, 67, 54, 0.3);
    color: #f44336;
  }

  .create-form {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .form-section {
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 8px;
    padding: 24px;
  }

  .section-header {
    margin-bottom: 16px;
    padding-bottom: 12px;
    border-bottom: 1px solid #333;
  }

  .section-header h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
  }

  .form-grid {
    display: grid;
    grid-template-columns: 1fr 200px;
    gap: 16px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .form-group label {
    font-size: 14px;
    font-weight: 500;
    color: #ccc;
  }

  .form-group input,
  .form-group textarea,
  .form-group select {
    padding: 12px 16px;
    background: #252526;
    border: 1px solid #333;
    border-radius: 6px;
    color: #fff;
    font-size: 14px;
    font-family: inherit;
    transition: border-color 0.2s;
  }

  .form-group input:focus,
  .form-group textarea:focus,
  .form-group select:focus {
    outline: none;
    border-color: #007acc;
  }

  .form-group input:disabled,
  .form-group textarea:disabled,
  .form-group select:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .form-group textarea {
    resize: vertical;
    min-height: 80px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
  }

  .checkbox-label input {
    width: 18px;
    height: 18px;
    cursor: pointer;
  }

  .hint {
    font-size: 12px;
    color: #858585;
  }

  .editor-container {
    min-height: 500px;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    padding-top: 12px;
  }

  .btn {
    padding: 12px 24px;
    border-radius: 6px;
    font-size: 15px;
    font-weight: 600;
    cursor: pointer;
    border: none;
    transition: all 0.2s;
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

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  @media (max-width: 768px) {
    .form-grid {
      grid-template-columns: 1fr;
    }

    .form-actions {
      flex-direction: column-reverse;
    }

    .editor-container {
      min-height: 400px;
    }
  }
</style>
