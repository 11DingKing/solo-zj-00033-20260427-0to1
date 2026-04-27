<script lang="ts">
  import { writable, type Writable } from 'svelte/store';
  import CodeEditor from './CodeEditor.svelte';
  import type { CreateFile } from '$lib/types';

  export let files: CreateFile[] = [];
  export let activeIndex: number = 0;
  export let onFilesChange: (files: CreateFile[]) => void = () => {};

  let fileContents: Writable<string[]> = writable(files.map(f => f.content));
  let filenames: Writable<string[]> = writable(files.map(f => f.filename));
  let languages: Writable<string[]> = writable(files.map(f => f.language || ''));

  $: {
    fileContents.set(files.map(f => f.content));
    filenames.set(files.map(f => f.filename));
    languages.set(files.map(f => f.language || ''));
  }

  function addFile() {
    const newIndex = files.length;
    const newFile: CreateFile = {
      filename: `file${newIndex + 1}.txt`,
      content: '',
      language: 'text'
    };
    files = [...files, newFile];
    activeIndex = newIndex;
    onFilesChange(files);
  }

  function removeFile(index: number) {
    if (files.length <= 1) return;
    
    files = files.filter((_, i) => i !== index);
    if (activeIndex >= files.length) {
      activeIndex = Math.max(0, files.length - 1);
    }
    onFilesChange(files);
  }

  function selectTab(index: number) {
    activeIndex = index;
  }

  function updateFilename(index: number, newName: string) {
    files = files.map((f, i) => i === index ? { ...f, filename: newName } : f);
    onFilesChange(files);
  }

  function updateFileLanguage(index: number, newLang: string) {
    files = files.map((f, i) => i === index ? { ...f, language: newLang || undefined } : f);
    onFilesChange(files);
  }

  function updateFileContent(index: number, content: string) {
    files = files.map((f, i) => i === index ? { ...f, content } : f);
    onFilesChange(files);
  }
</script>

<div class="file-editor">
  <div class="tabs-bar">
    <div class="tabs">
      {#each files as file, index}
        <div 
          class="tab {activeIndex === index ? 'active' : ''}"
          on:click={() => selectTab(index)}
        >
          <span class="tab-filename">{file.filename}</span>
          {#if files.length > 1}
            <button 
              class="tab-close"
              on:click|stopPropagation={() => removeFile(index)}
            >×</button>
          {/if}
        </div>
      {/each}
    </div>
    <button class="add-tab-btn" on:click={addFile} title="Add new file">+</button>
  </div>

  {#if files.length > 0}
    <div class="file-settings">
      <div class="setting-group">
        <label>Filename:</label>
        <input 
          type="text" 
          value={files[activeIndex].filename}
          on:input={(e) => updateFilename(activeIndex, (e.target as HTMLInputElement).value)}
          placeholder="filename.ext"
        />
      </div>
      <div class="setting-group">
        <label>Language:</label>
        <select 
          value={files[activeIndex].language || 'text'}
          on:change={(e) => updateFileLanguage(activeIndex, (e.target as HTMLSelectElement).value)}
        >
          <option value="text">Plain Text</option>
          <option value="javascript">JavaScript</option>
          <option value="typescript">TypeScript</option>
          <option value="python">Python</option>
          <option value="rust">Rust</option>
          <option value="go">Go</option>
          <option value="java">Java</option>
          <option value="csharp">C#</option>
          <option value="cpp">C++</option>
          <option value="php">PHP</option>
          <option value="ruby">Ruby</option>
          <option value="swift">Swift</option>
          <option value="html">HTML</option>
          <option value="css">CSS</option>
          <option value="sql">SQL</option>
          <option value="json">JSON</option>
          <option value="yaml">YAML</option>
          <option value="markdown">Markdown</option>
          <option value="bash">Bash</option>
        </select>
      </div>
    </div>

    <div class="editor-wrapper">
      {#each files as file, index}
        {#if activeIndex === index}
          <CodeEditor
            value={file.content}
            language={file.language || 'text'}
            on:change={(content) => updateFileContent(index, content)}
          />
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .file-editor {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 6px;
    overflow: hidden;
  }

  .tabs-bar {
    display: flex;
    align-items: center;
    background: #252526;
    border-bottom: 1px solid #333;
    min-height: 35px;
  }

  .tabs {
    display: flex;
    flex: 1;
    overflow-x: auto;
  }

  .tab {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    background: #2d2d2d;
    border-right: 1px solid #333;
    cursor: pointer;
    min-width: 100px;
    max-width: 200px;
  }

  .tab:hover {
    background: #333;
  }

  .tab.active {
    background: #1e1e1e;
    border-bottom: 2px solid #007acc;
  }

  .tab-filename {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 13px;
    color: #ccc;
  }

  .tab.active .tab-filename {
    color: #fff;
  }

  .tab-close {
    background: none;
    border: none;
    color: #858585;
    cursor: pointer;
    font-size: 16px;
    padding: 0;
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 2px;
  }

  .tab-close:hover {
    background: #f44336;
    color: #fff;
  }

  .add-tab-btn {
    background: none;
    border: 1px solid #333;
    color: #858585;
    cursor: pointer;
    font-size: 20px;
    padding: 4px 10px;
    margin-right: 8px;
    border-radius: 4px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .add-tab-btn:hover {
    background: #333;
    color: #fff;
  }

  .file-settings {
    display: flex;
    gap: 16px;
    padding: 12px 16px;
    background: #252526;
    border-bottom: 1px solid #333;
  }

  .setting-group {
    display: flex;
    flex-direction: column;
    gap: 4px;
    flex: 1;
  }

  .setting-group:last-child {
    max-width: 200px;
  }

  .setting-group label {
    font-size: 12px;
    color: #858585;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .setting-group input,
  .setting-group select {
    padding: 8px 12px;
    background: #1e1e1e;
    border: 1px solid #333;
    border-radius: 4px;
    color: #fff;
    font-size: 13px;
  }

  .setting-group input:focus,
  .setting-group select:focus {
    outline: none;
    border-color: #007acc;
  }

  .editor-wrapper {
    flex: 1;
    min-height: 400px;
  }
</style>
