<template>
  <div class="message-editor">
    <div class="editor-toolbar d-flex align-center ga-1 px-2 py-1">
      <v-btn
        icon="mdi-format-bold"
        size="x-small"
        variant="text"
        :color="editor?.isActive('bold') ? 'primary' : undefined"
        @click="editor?.chain().focus().toggleBold().run()"
      />
      <v-btn
        icon="mdi-format-italic"
        size="x-small"
        variant="text"
        :color="editor?.isActive('italic') ? 'primary' : undefined"
        @click="editor?.chain().focus().toggleItalic().run()"
      />
      <v-btn
        icon="mdi-format-underline"
        size="x-small"
        variant="text"
        :color="editor?.isActive('underline') ? 'primary' : undefined"
        @click="editor?.chain().focus().toggleUnderline().run()"
      />
      <v-btn
        icon="mdi-format-strikethrough"
        size="x-small"
        variant="text"
        :color="editor?.isActive('strike') ? 'primary' : undefined"
        @click="editor?.chain().focus().toggleStrike().run()"
      />

      <v-divider vertical class="mx-1" />

      <v-btn
        icon="mdi-code-tags"
        size="x-small"
        variant="text"
        :color="editor?.isActive('code') ? 'primary' : undefined"
        @click="editor?.chain().focus().toggleCode().run()"
      />
      <v-btn
        icon="mdi-code-braces"
        size="x-small"
        variant="text"
        :color="editor?.isActive('codeBlock') ? 'primary' : undefined"
        @click="editor?.chain().focus().toggleCodeBlock().run()"
      />

      <v-divider vertical class="mx-1" />

      <v-btn
        icon="mdi-format-list-bulleted"
        size="x-small"
        variant="text"
        :color="editor?.isActive('bulletList') ? 'primary' : undefined"
        @click="editor?.chain().focus().toggleBulletList().run()"
      />
      <v-btn
        icon="mdi-format-list-numbered"
        size="x-small"
        variant="text"
        :color="editor?.isActive('orderedList') ? 'primary' : undefined"
        @click="editor?.chain().focus().toggleOrderedList().run()"
      />
      <v-btn
        icon="mdi-format-quote-close"
        size="x-small"
        variant="text"
        :color="editor?.isActive('blockquote') ? 'primary' : undefined"
        @click="editor?.chain().focus().toggleBlockquote().run()"
      />

      <v-divider vertical class="mx-1" />

      <v-btn
        icon="mdi-link"
        size="x-small"
        variant="text"
        :color="editor?.isActive('link') ? 'primary' : undefined"
        @click="toggleLink"
      />

      <v-spacer />

      <v-btn
        icon="mdi-emoticon-outline"
        size="x-small"
        variant="text"
        @click="$emit('open-emoji-picker')"
      />
      <v-btn
        icon="mdi-gif"
        size="x-small"
        variant="text"
        @click="$emit('open-giphy-picker')"
      />
    </div>

    <editor-content :editor="editor" class="editor-content" />
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, watch } from 'vue'
import { useEditor, EditorContent } from '@tiptap/vue-3'
import StarterKit from '@tiptap/starter-kit'
import Placeholder from '@tiptap/extension-placeholder'
import Link from '@tiptap/extension-link'
import Underline from '@tiptap/extension-underline'
import { Markdown } from 'tiptap-markdown'

const props = withDefaults(
  defineProps<{
    placeholder?: string
    initialContent?: string
  }>(),
  {
    placeholder: 'Write a message...',
    initialContent: '',
  },
)

const emit = defineEmits<{
  send: [content: string]
  'open-emoji-picker': []
  'open-giphy-picker': []
}>()

const editor = useEditor({
  content: props.initialContent,
  extensions: [
    StarterKit,
    Placeholder.configure({ placeholder: props.placeholder }),
    Link.configure({ openOnClick: false, autolink: true }),
    Underline,
    Markdown.configure({ html: false, transformPastedText: true }),
  ],
  editorProps: {
    handleKeyDown(_view, event) {
      if (event.key === 'Enter' && !event.shiftKey) {
        event.preventDefault()
        handleSend()
        return true
      }
      return false
    },
  },
})

watch(
  () => props.initialContent,
  (val) => {
    if (editor.value && val !== getMarkdown()) {
      editor.value.commands.setContent(val)
    }
  },
)

function getMarkdown(): string {
  if (!editor.value) return ''
  return editor.value.storage.markdown.getMarkdown()
}

function handleSend() {
  const content = getMarkdown().trim()
  if (!content) return
  emit('send', content)
  editor.value?.commands.clearContent(true)
}

function toggleLink() {
  if (!editor.value) return
  if (editor.value.isActive('link')) {
    editor.value.chain().focus().unsetLink().run()
  } else {
    const url = window.prompt('URL')
    if (url) {
      editor.value.chain().focus().setLink({ href: url }).run()
    }
  }
}

function insertContent(text: string) {
  editor.value?.chain().focus().insertContent(text).run()
}

function setContent(markdown: string) {
  editor.value?.commands.setContent(markdown)
}

function focus() {
  editor.value?.commands.focus()
}

defineExpose({ insertContent, setContent, focus })

onBeforeUnmount(() => {
  editor.value?.destroy()
})
</script>

<style scoped>
.message-editor {
  border: 1px solid rgba(var(--v-theme-on-surface), 0.2);
  border-radius: 8px;
  overflow: hidden;
}
.editor-toolbar {
  border-bottom: 1px solid rgba(var(--v-theme-on-surface), 0.08);
  background: rgba(var(--v-theme-on-surface), 0.02);
}
.editor-content {
  min-height: 40px;
  max-height: 200px;
  overflow-y: auto;
  padding: 8px 12px;
}
.editor-content :deep(.tiptap) {
  outline: none;
  min-height: 24px;
}
.editor-content :deep(.tiptap p.is-editor-empty:first-child::before) {
  content: attr(data-placeholder);
  float: left;
  color: rgba(var(--v-theme-on-surface), 0.35);
  pointer-events: none;
  height: 0;
}
.editor-content :deep(.tiptap p) {
  margin: 0;
}
.editor-content :deep(.tiptap code) {
  background: rgba(var(--v-theme-on-surface), 0.08);
  border-radius: 3px;
  padding: 1px 4px;
  font-size: 0.85em;
}
.editor-content :deep(.tiptap pre) {
  background: rgba(var(--v-theme-on-surface), 0.08);
  border-radius: 4px;
  padding: 8px 12px;
  margin: 4px 0;
}
.editor-content :deep(.tiptap blockquote) {
  border-left: 3px solid rgba(var(--v-theme-primary), 0.5);
  padding-left: 12px;
  margin: 4px 0;
}
</style>
