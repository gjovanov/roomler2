import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '@/api/client'

interface FileEntry {
  id: string
  tenant_id: string
  filename: string
  content_type: string
  size: number
  uploaded_by: string
  created_at: string
}

export const useFileStore = defineStore('files', () => {
  const files = ref<FileEntry[]>([])
  const loading = ref(false)

  async function fetchFiles(tenantId: string, channelId: string) {
    loading.value = true
    try {
      const data = await api.get<{ items: FileEntry[] }>(
        `/tenant/${tenantId}/channel/${channelId}/file`,
      )
      files.value = data.items
    } finally {
      loading.value = false
    }
  }

  async function uploadFile(tenantId: string, channelId: string, file: File) {
    const form = new FormData()
    form.append('file', file)
    form.append('channel_id', channelId)
    const entry = await api.upload<FileEntry>(
      `/tenant/${tenantId}/channel/file/upload`,
      form,
    )
    files.value.push(entry)
    return entry
  }

  async function deleteFile(tenantId: string, fileId: string) {
    await api.delete(`/tenant/${tenantId}/channel/file/${fileId}`)
    files.value = files.value.filter((f) => f.id !== fileId)
  }

  function downloadUrl(tenantId: string, fileId: string): string {
    return `/api/tenant/${tenantId}/channel/file/${fileId}/download`
  }

  return { files, loading, fetchFiles, uploadFile, deleteFile, downloadUrl }
})
