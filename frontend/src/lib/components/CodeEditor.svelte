<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { EditorState } from '@codemirror/state';
  import { EditorView, lineNumbers, highlightActiveLineGutter, drawSelection, dropCursor, rectangularSelection, crosshairCursor, highlightActiveLine, keymap, placeholder as cmPlaceholder } from '@codemirror/view';
  import { defaultKeymap, history, historyKeymap, indentWithTab } from '@codemirror/commands';
  import { syntaxHighlighting, defaultHighlightStyle, bracketMatching, foldGutter, foldKeymap, indentOnInput } from '@codemirror/language';
  import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
  import { completionKeymap } from '@codemirror/autocomplete';
  import { lintKeymap } from '@codemirror/lint';
  import { javascript } from '@codemirror/lang-javascript';
  import { python } from '@codemirror/lang-python';
  import { rust } from '@codemirror/lang-rust';
  import { json } from '@codemirror/lang-json';
  import { html } from '@codemirror/lang-html';
  import { css } from '@codemirror/lang-css';
  import { sql } from '@codemirror/lang-sql';
  import { markdown } from '@codemirror/lang-markdown';
  import { go } from '@codemirror/lang-go';
  import { java } from '@codemirror/lang-java';
  import { php } from '@codemirror/lang-php';
  import { cpp } from '@codemirror/lang-cpp';
  import type { Extension } from '@codemirror/state';

  export let value: string = '';
  export let language: string = 'javascript';
  export let readOnly: boolean = false;
  export let placeholder: string = 'Enter code here...';
  export let onChange: (value: string) => void = () => {};

  let container: HTMLDivElement;
  let editorView: EditorView | null = null;

  const languageExtensions: Record<string, Extension> = {
    javascript: javascript(),
    typescript: javascript({ typescript: true }),
    python: python(),
    rust: rust(),
    go: go(),
    java: java(),
    csharp: cpp(),
    cpp: cpp(),
    php: php(),
    ruby: cpp(),
    swift: cpp(),
    kotlin: java(),
    html: html(),
    css: css(),
    sql: sql(),
    json: json(),
    yaml: json(),
    markdown: markdown(),
    bash: javascript()
  };

  $: currentExtension = languageExtensions[language] || [];

  const updateListener = EditorView.updateListener.of((update) => {
    if (update.docChanged) {
      const newText = update.state.doc.toString();
      value = newText;
      onChange(newText);
    }
  });

  function createEditor() {
    if (!container) return;

    const extensions: Extension[] = [
      history(),
      lineNumbers(),
      highlightActiveLineGutter(),
      foldGutter(),
      drawSelection(),
      dropCursor(),
      EditorState.allowMultipleSelections.of(true),
      indentOnInput(),
      syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
      bracketMatching(),
      rectangularSelection(),
      crosshairCursor(),
      highlightActiveLine(),
      highlightSelectionMatches(),
      cmPlaceholder(placeholder),
      keymap.of([
        ...defaultKeymap,
        ...historyKeymap,
        ...searchKeymap,
        ...completionKeymap,
        ...lintKeymap,
        indentWithTab
      ]),
      updateListener,
      currentExtension
    ];

    if (readOnly) {
      extensions.push(EditorView.editable.of(false));
    }

    const state = EditorState.create({
      doc: value,
      extensions
    });

    editorView = new EditorView({
      state,
      parent: container
    });
  }

  function destroyEditor() {
    if (editorView) {
      editorView.destroy();
      editorView = null;
    }
  }

  $: {
    if (editorView && value !== editorView.state.doc.toString()) {
      const transaction = editorView.state.update({
        changes: {
          from: 0,
          to: editorView.state.doc.length,
          insert: value
        }
      });
      editorView.dispatch(transaction);
    }
  }

  $: {
    if (editorView) {
      destroyEditor();
      createEditor();
    }
  }

  onMount(() => {
    createEditor();
  });

  onDestroy(() => {
    destroyEditor();
  });
</script>

<div class="codemirror-container" bind:this={container}></div>

<style>
  .codemirror-container {
    width: 100%;
    height: 100%;
  }

  .codemirror-container :global(.cm-editor) {
    height: 100%;
    font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
    font-size: 14px;
    background: #1e1e1e;
  }

  .codemirror-container :global(.cm-scroller) {
    overflow: auto;
  }

  .codemirror-container :global(.cm-gutters) {
    background: #1e1e1e;
    border-right: 1px solid #333;
  }

  .codemirror-container :global(.cm-activeLineGutter) {
    background: #2d2d2d;
  }

  .codemirror-container :global(.cm-activeLine) {
    background: #2d2d2d;
  }

  .codemirror-container :global(.cm-selectionBackground) {
    background: #264f78;
  }

  .codemirror-container :global(.cm-cursor) {
    border-left-color: #fff;
  }

  .codemirror-container :global(.cm-placeholder) {
    color: #858585;
    font-style: italic;
  }
</style>
