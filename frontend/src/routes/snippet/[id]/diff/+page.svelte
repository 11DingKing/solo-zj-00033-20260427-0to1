<script lang="ts">
  import { page, goto } from '$app/stores';
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import type { Version, SnippetFile } from '$lib/types';
  import * as Diff from 'diff';

  $: snippetId = $page.params.id;
  $: {
    const params = new URLSearchParams($page.url.search);
    versionA = parseInt(params.get('v_a') || '1');
    versionB = parseInt(params.get('v_b') || '2');
  }

  let versionA: number = 1;
  let versionB: number = 2;
  let versions: Version[] = [];
  let filesA: SnippetFile[] = [];
  let filesB: SnippetFile[] = [];
  let activeFileIndex: number = 0;
  let loading: boolean = true;
  let error: string = '';
  let diffLines: { type: 'added' | 'removed' | 'unchanged' | 'modified'; left: string | null; right: string | null; lineNumberLeft: number; lineNumberRight: number }[] = [];

  async function loadVersions() {
    try {
      const response = await api.get(`/api/snippets/${snippetId}/versions`);
      if (response.ok) {
        versions = await response.json();
        if (versions.length < 2) {
          error = 'This snippet has fewer than 2 versions to compare';
        }
      }
    } catch (e) {
      console.error('Failed to load versions:', e);
    }
  }

  async function loadDiff() {
    if (versionA === versionB) {
      error = 'Please select two different versions to compare';
      return;
    }

    loading = true;
    error = '';

    try {
      const response = await api.get(`/api/snippets/${snippetId}/diff/${versionA}/${versionB}`);
      if (response.ok) {
        const data = await response.json();
        filesA = data.files_a || [];
        filesB = data.files_b || [];
        
        if (filesA.length > 0 && filesB.length > 0) {
          computeDiff();
        }
      } else {
        error = 'Failed to load diff data';
      }
    } catch (e) {
      error = 'Network error. Please try again.';
    } finally {
      loading = false;
    }
  }

  function computeDiff() {
    const fileA = filesA[activeFileIndex];
    const fileB = filesB[activeFileIndex] || filesA[activeFileIndex];

    if (!fileA && !fileB) {
      diffLines = [];
      return;
    }

    const contentA = fileA?.content || '';
    const contentB = fileB?.content || '';

    const diffResult = Diff.diffLines(contentA, contentB);
    diffLines = [];

    let lineNumberLeft = 1;
    let lineNumberRight = 1;

    for (const part of diffResult) {
      const lines = part.value.split('\n');
      const isLastLineEmpty = lines[lines.length - 1] === '';
      if (isLastLineEmpty) lines.pop();

      for (const line of lines) {
        if (part.added) {
          diffLines.push({
            type: 'added',
            left: null,
            right: line,
            lineNumberLeft: 0,
            lineNumberRight: lineNumberRight++
          });
        } else if (part.removed) {
          diffLines.push({
            type: 'removed',
            left: line,
            right: null,
            lineNumberLeft: lineNumberLeft++,
            lineNumberRight: 0
          });
        } else {
          diffLines.push({
            type: 'unchanged',
            left: line,
            right: line,
            lineNumberLeft: lineNumberLeft++,
            lineNumberRight: lineNumberRight++
          });
        }
      }
    }
  }

  function selectFile(index: number) {
    activeFileIndex = index;
    computeDiff();
  }

  function onVersionChange() {
    const params = new URLSearchParams($page.url.search);
    params.set('v_a', versionA.toString());
    params.set('v_b', versionB.toString());
    goto(`/snippet/${snippetId}/diff?${params.toString()}`);
  }

  function getLineClass(type: string): string {
    switch (type) {
      case 'added': return 'diff-added';
      case 'removed': return 'diff-removed';
      case 'modified': return 'diff-modified';
      default: return 'diff-unchanged';
    }
  }

  onMount(() => {
    loadVersions();
  });

  $: {
    if (versionA && versionB && versionA !== versionB) {
      loadDiff();
    }
  }
</script>

<div class="diff-page">
  <div class="page-header">
    <div class="header-left">
      <a href={`/snippet/${snippetId}`} class="back-link">← Back to snippet</a>
      <h1>Version Comparison</h1>
    </div>
    <div class="version-selectors">
      <div class="version-selector">
        <label>Version A:</label>
        <select 
          bind:value={versionA}
          on:change={onVersionChange}
        >
          {#each versions as v}
            <option value={v.version_number}>
              v{v.version_number} - {v.commit_message || 'No message'}
            </option>
          {/each}
        </select>
      </div>
      <span class="separator">⇨</span>
      <div class="version-selector">
        <label>Version B:</label>
        <select 
          bind:value={versionB}
          on:change={onVersionChange}
        >
          {#each versions as v}
            <option value={v.version_number}>
              v{v.version_number} - {v.commit_message || 'No message'}
            </option>
          {/each}
        </select>
      </div>
    </div>
  </div>

  {#if error}
    <div class="error-box">
      {error}
    </div>
  {:else if loading}
    <div class="loading">Loading diff...</div>
  {:else}
    {#if filesA.length > 1 || filesB.length > 1}
      <div class="file-tabs">
        {#each filesA as file, index}
          <button 
            class="file-tab {activeFileIndex === index ? 'active' : ''}"
            on:click={() => selectFile(index)}
          >
            {file.filename}
          </button>
        {/each}
      </div>
    {/if}

    <div class="diff-container">
      <div class="diff-legend">
        <div class="legend-item">
          <span class="legend-color diff-removed"></span>
          <span>Removed</span>
        </div>
        <div class="legend-item">
          <span class="legend-color diff-added"></span>
          <span>Added</span>
        </div>
      </div>

      <div class="diff-table-wrapper">
        <table class="diff-table">
          <thead>
            <tr>
              <th class="line-num-col">Line</th>
              <th class="code-col left-col">
                v{versionA}
                {#if filesA[activeFileIndex]}
                  <span class="filename">{filesA[activeFileIndex].filename}</span>
                {/if}
              </th>
              <th class="line-num-col">Line</th>
              <th class="code-col right-col">
                v{versionB}
                {#if filesB[activeFileIndex] || filesA[activeFileIndex]}
                  <span class="filename">{(filesB[activeFileIndex] || filesA[activeFileIndex])?.filename}</span>
                {/if}
              </th>
            </tr>
          </thead>
          <tbody>
            {#each diffLines as line}
              <tr class="{getLineClass(line.type)}">
                <td class="line-number {line.type === 'added' ? 'empty' : ''}">
                  {line.lineNumberLeft > 0 ? line.lineNumberLeft : ''}
                </td>
                <td class="code-cell {line.type === 'added' ? 'empty' : ''}">
                  <span class="diff-marker">{line.type === 'removed' ? '-' : ' '}</span>
                  <span class="code-text">{line.left !== null ? line.left : ' '}</span>
                </td>
                <td class="line-number {line.type === 'removed' ? 'empty' : ''}">
                  {line.lineNumberRight > 0 ? line.lineNumberRight : ''}
                </td>
                <td class="code-cell {line.type === 'removed' ? 'empty' : ''}">
                  <span class="diff-marker">{line.type === 'added' ? '+' : ' '}</span>
                  <span class="code-text">{line.right !== null ? line.right : ' '}</span>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
</div>

<style>
  .diff-page {
    max-width: 1400px;
    margin: 0 auto;
  }

  .page-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 24px;
    margin-bottom: 24px;
    flex-wrap: wrap;
  }

  .header-left {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .back-link {
    color: #007acc;
    text-decoration: none;
    font-size: 14px;
  }

  .back-link:hover {
    text-decoration: underline;
  }

  .page-header h1 {
    margin: 0;
    font-size: 28px;
    font-weight: 600;
  }

  .version-selectors {
    display: flex;
    align-items: center;
    gap: 16px;
    flex-wrap: wrap;
  }

  .version-selector {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .version-selector label {
    font-size: 12px;
    color: #858585;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .version-selector select {
    padding: 10px 16px;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 6px;
    color: #fff;
    font-size: 14px;
    min-width: 200px;
  }

  .version-selector select:focus {
    outline: none;
    border-color: #007acc;
  }

  .separator {
    font-size: 24px;
    color: #007acc;
    margin-top: 20px;
  }

  .error-box {
    background: rgba(244, 67, 54, 0.15);
    border: 1px solid rgba(244, 67, 54, 0.3);
    border-radius: 6px;
    padding: 16px;
    color: #f44336;
    text-align: center;
  }

  .loading {
    text-align: center;
    padding: 80px;
    color: #858585;
    font-size: 16px;
  }

  .file-tabs {
    display: flex;
    background: #252526;
    border: 1px solid #333;
    border-radius: 8px 8px 0 0;
    border-bottom: none;
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
    border-bottom: 2px solid transparent;
  }

  .file-tab:hover {
    background: #2d2d2d;
    color: #ccc;
  }

  .file-tab.active {
    background: #1e1e1e;
    color: #fff;
    border-bottom-color: #007acc;
  }

  .diff-container {
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 8px;
    overflow: hidden;
  }

  .diff-legend {
    display: flex;
    gap: 24px;
    padding: 12px 20px;
    background: #252526;
    border-bottom: 1px solid #333;
  }

  .legend-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    color: #ccc;
  }

  .legend-color {
    width: 20px;
    height: 20px;
    border-radius: 4px;
  }

  .diff-table-wrapper {
    overflow-x: auto;
    max-height: 70vh;
    overflow-y: auto;
  }

  .diff-table {
    width: 100%;
    border-collapse: collapse;
    font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
    font-size: 13px;
    line-height: 1.5;
  }

  .diff-table thead {
    position: sticky;
    top: 0;
    z-index: 10;
  }

  .diff-table th {
    background: #252526;
    padding: 10px 12px;
    text-align: left;
    font-weight: 600;
    color: #ccc;
    border-bottom: 1px solid #333;
  }

  .line-num-col {
    width: 50px;
  }

  .code-col {
    width: calc(50% - 50px);
  }

  .code-col .filename {
    font-size: 11px;
    color: #858585;
    margin-left: 8px;
    font-weight: 400;
  }

  .diff-table td {
    padding: 2px 8px;
    vertical-align: top;
    white-space: pre;
  }

  .line-number {
    text-align: right;
    color: #858585;
    background: #252526;
    border-right: 1px solid #333;
    user-select: none;
  }

  .line-number.empty {
    background: #1a1a1a;
    color: #444;
  }

  .code-cell {
    padding: 2px 12px;
  }

  .code-cell.empty {
    background: #1a1a1a;
  }

  .diff-marker {
    display: inline-block;
    width: 16px;
    user-select: none;
    color: #858585;
  }

  .code-text {
    color: #d4d4d4;
  }

  .diff-added {
    background: rgba(61, 183, 81, 0.15);
  }

  .diff-added .diff-marker {
    color: #3db751;
  }

  .diff-removed {
    background: rgba(244, 67, 54, 0.15);
  }

  .diff-removed .diff-marker {
    color: #f44336;
  }

  .diff-unchanged {
    background: transparent;
  }

  .diff-modified {
    background: rgba(255, 193, 7, 0.15);
  }

  .legend-color.diff-removed {
    background: rgba(244, 67, 54, 0.3);
  }

  .legend-color.diff-added {
    background: rgba(61, 183, 81, 0.3);
  }

  @media (max-width: 768px) {
    .page-header {
      flex-direction: column;
    }

    .version-selectors {
      width: 100%;
    }

    .version-selector {
      flex: 1;
      min-width: 150px;
    }

    .version-selector select {
      min-width: auto;
      width: 100%;
    }

    .separator {
      display: none;
    }

    .diff-table {
      font-size: 11px;
    }

    .line-num-col {
      width: 35px;
    }

    .diff-table td {
      padding: 2px 4px;
    }
  }
</style>
