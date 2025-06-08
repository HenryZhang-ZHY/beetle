import { createFileRoute, Link } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { Button, Spin, message, Modal, Form, Input } from 'antd'
import { FolderOpenOutlined, PlusOutlined, ReloadOutlined, SearchOutlined, DeleteOutlined, ArrowLeftOutlined, SyncOutlined } from '@ant-design/icons'

interface IndexResponse {
  name: string
  path: string
}

interface CreateIndexRequest {
  name: string
  path: string
}

export const Route = createFileRoute('/indexes')({
  component: IndexesPage,
})

function IndexesPage() {
  const [indexes, setIndexes] = useState<IndexResponse[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [updatingIndexes, setUpdatingIndexes] = useState<Set<string>>(new Set())
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false)
  const [createForm] = Form.useForm()
  const [creating, setCreating] = useState(false)
  const [deletingIndexes, setDeletingIndexes] = useState<Set<string>>(new Set())
  const [deleteModalOpen, setDeleteModalOpen] = useState(false)
  const [indexToDelete, setIndexToDelete] = useState<string>('')

  const fetchIndexes = async () => {
    try {
      setLoading(true)
      setError(null)
      const response = await fetch('/api/indexes')

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      const data = await response.json()
      setIndexes(data)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch indexes')
      message.error('Failed to load indexes')
    } finally {
      setLoading(false)
    }
  }

  const updateIndex = async (indexName: string) => {
    try {
      setUpdatingIndexes(prev => new Set(prev).add(indexName))

      const response = await fetch(`/api/indexes/${indexName}/update`, {
        method: 'POST'
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.error || `HTTP error! status: ${response.status}`)
      }

      message.success(`Index "${indexName}" updated successfully`)
      await fetchIndexes() // Refresh the list
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to update index'
      message.error(`Failed to update index: ${errorMessage}`)
    } finally {
      setUpdatingIndexes(prev => {
        const newSet = new Set(prev)
        newSet.delete(indexName)
        return newSet
      })
    }
  }

  const deleteIndex = async (indexName: string) => {
    try {
      setDeletingIndexes(prev => new Set(prev).add(indexName))

      const response = await fetch(`/api/indexes/${indexName}`, {
        method: 'DELETE'
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.error || `HTTP error! status: ${response.status}`)
      }

      message.success(`Index "${indexName}" deleted successfully`)
      await fetchIndexes() // Refresh the list
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to delete index'
      message.error(`Failed to delete index: ${errorMessage}`)
    } finally {
      setDeletingIndexes(prev => {
        const newSet = new Set(prev)
        newSet.delete(indexName)
        return newSet
      })
    }
  }

  const handleDeleteClick = (indexName: string) => {
    setIndexToDelete(indexName)
    setDeleteModalOpen(true)
  }

  const handleDeleteConfirm = () => {
    if (indexToDelete) {
      deleteIndex(indexToDelete)
      setDeleteModalOpen(false)
      setIndexToDelete('')
    }
  }

  const handleDeleteCancel = () => {
    setDeleteModalOpen(false)
    setIndexToDelete('')
  }

  const createIndex = async (values: CreateIndexRequest) => {
    try {
      setCreating(true)

      const response = await fetch('/api/indexes', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(values),
      })

      if (!response.ok) {
        const errorData = await response.json()
        throw new Error(errorData.error || `HTTP error! status: ${response.status}`)
      }

      message.success(`Index "${values.name}" created successfully`)
      setIsCreateModalOpen(false)
      createForm.resetFields()
      await fetchIndexes() // Refresh the list
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to create index'
      message.error(`Failed to create index: ${errorMessage}`)
    } finally {
      setCreating(false)
    }
  }

  const handleCreateModalCancel = () => {
    setIsCreateModalOpen(false)
    createForm.resetFields()
  }

  useEffect(() => {
    fetchIndexes()
  }, [])

  return (
    <div className="min-h-screen bg-slate-50">
      {/* Modern Navigation */}
      <nav className="bg-white border-b border-slate-200 px-6 py-4">
        <div className="max-w-7xl mx-auto flex items-center justify-between">
          <div className="flex items-center space-x-4">
            <Link to="/">
              <Button
                type="text"
                icon={<ArrowLeftOutlined />}
                className="text-slate-600 hover:text-slate-900 hover:bg-slate-100 border-0"
              >
                Back to Search
              </Button>
            </Link>
            <div className="border-l border-slate-200 pl-4">
              <h1 className="text-lg font-semibold text-slate-900">Index Management</h1>
              <p className="text-xs text-slate-500">Manage your code indexes</p>
            </div>
          </div>

          <div className="flex items-center space-x-3">
            <Button
              type="text"
              icon={<ReloadOutlined />}
              onClick={fetchIndexes}
              loading={loading}
              className="text-slate-600 hover:text-slate-900 hover:bg-slate-100 border-0"
            >
              Refresh
            </Button>
            <Button
              type="primary"
              icon={<PlusOutlined />}
              onClick={() => setIsCreateModalOpen(true)}
              className="bg-slate-900 hover:bg-slate-800 border-slate-900"
            >
              New Index
            </Button>
          </div>
        </div>
      </nav>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-6 py-12">
        {loading ? (
          <div className="flex justify-center py-16">
            <Spin size="large" />
          </div>
        ) : error ? (
          <div className="text-center py-16">
            <div className="bg-white rounded-lg border border-slate-200 p-8 shadow-sm max-w-md mx-auto">
              <div className="w-12 h-12 bg-red-100 rounded-lg flex items-center justify-center mx-auto mb-4">
                <span className="text-red-600 text-xl">⚠</span>
              </div>
              <h3 className="text-lg font-medium text-slate-900 mb-2">Error loading indexes</h3>
              <p className="text-slate-600 mb-6">{error}</p>
              <Button
                type="primary"
                onClick={fetchIndexes}
                className="bg-slate-900 hover:bg-slate-800 border-slate-900"
              >
                Try Again
              </Button>
            </div>
          </div>
        ) : indexes.length === 0 ? (
          <div className="text-center py-16">
            <div className="bg-white rounded-lg border border-slate-200 p-8 shadow-sm max-w-md mx-auto">
              <FolderOpenOutlined className="text-4xl text-slate-400 mb-4" />
              <h3 className="text-lg font-medium text-slate-900 mb-2">No indexes found</h3>
              <p className="text-slate-600 mb-6">
                Create your first index to start searching your code
              </p>
              <Button
                type="primary"
                icon={<PlusOutlined />}
                size="large"
                onClick={() => setIsCreateModalOpen(true)}
                className="bg-slate-900 hover:bg-slate-800 border-slate-900"
              >
                Create Your First Index
              </Button>
            </div>
          </div>
        ) : (
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-2xl font-bold text-slate-900 mb-2">
                  Your Indexes
                </h2>
                <p className="text-slate-600">
                  {indexes.length} {indexes.length === 1 ? 'index' : 'indexes'} available
                </p>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {indexes.map((index) => (
                <div
                  key={index.name}
                  className="bg-white rounded-xl border border-slate-200 hover:border-slate-300 shadow-sm hover:shadow-lg transition-all duration-200 overflow-hidden"
                >
                  {/* Header with icon and title */}
                  <div className="p-6 pb-4">
                    <div className="flex items-start space-x-4">
                      <div className="w-14 h-14 bg-gradient-to-br from-slate-100 to-slate-200 rounded-xl flex items-center justify-center flex-shrink-0">
                        <FolderOpenOutlined className="text-slate-600 text-xl" />
                      </div>
                      <div className="flex-1 min-w-0">
                        <h3 className="text-xl font-bold text-slate-900 mb-1 truncate">
                          {index.name}
                        </h3>
                        <p className="text-sm text-slate-500">Code Index</p>
                      </div>
                    </div>
                  </div>

                  {/* Path section */}
                  <div className="px-6 pb-4">
                    <div className="bg-slate-50 rounded-lg p-4 border border-slate-100">
                      <div className="flex items-center space-x-2 mb-2">
                        <FolderOpenOutlined className="text-slate-400 text-sm" />
                        <span className="text-sm font-medium text-slate-700">Target Path</span>
                      </div>
                      <code className="text-sm text-slate-800 bg-white border border-slate-200 px-3 py-2 rounded-md block truncate font-mono">
                        {index.path}
                      </code>
                    </div>
                  </div>

                  {/* Actions */}
                  <div className="px-6 pb-6">
                    <div className="flex items-center space-x-2">
                      <Link to={`/`} className="flex-1">
                        <Button
                          type="primary"
                          size="large"
                          icon={<SearchOutlined />}
                          className="w-full bg-slate-900 hover:bg-slate-800 border-slate-900 font-medium"
                        >
                          Search
                        </Button>
                      </Link>
                      <Button
                        type="text"
                        size="large"
                        icon={<SyncOutlined />}
                        onClick={() => updateIndex(index.name)}
                        loading={updatingIndexes.has(index.name)}
                        className="text-blue-600 hover:text-blue-700 hover:bg-blue-50 border-0"
                        title="Update index"
                      />
                      <Button
                        type="text"
                        size="large"
                        danger
                        icon={<DeleteOutlined />}
                        onClick={() => handleDeleteClick(index.name)}
                        loading={deletingIndexes.has(index.name)}
                        className="text-red-600 hover:text-red-700 hover:bg-red-50 border-0"
                        title="Delete index"
                      />
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </main>

      {/* Create Index Modal */}
      <Modal
        title="Create New Index"
        open={isCreateModalOpen}
        onCancel={handleCreateModalCancel}
        footer={null}
        width={600}
      >
        <Form
          form={createForm}
          layout="vertical"
          onFinish={createIndex}
          className="mt-4"
        >
          <Form.Item
            label="Index Name"
            name="name"
            rules={[
              { required: true, message: 'Please enter an index name' },
              { pattern: /^[a-zA-Z0-9_-]+$/, message: 'Index name can only contain letters, numbers, underscores, and hyphens' }
            ]}
          >
            <Input placeholder="e.g., my-project" />
          </Form.Item>

          <Form.Item
            label="Target Path"
            name="path"
            rules={[
              { required: true, message: 'Please enter a absolute target path' },
            ]}
          >
            <Input placeholder="e.g., /home/hzh/projects/my-project" />
          </Form.Item>

          <div className="flex justify-end space-x-3 mt-6">
            <Button onClick={handleCreateModalCancel}>
              Cancel
            </Button>
            <Button
              type="primary"
              htmlType="submit"
              loading={creating}
              className="bg-slate-900 hover:bg-slate-800 border-slate-900"
            >
              Create Index
            </Button>
          </div>
        </Form>
      </Modal>

      {/* Delete Confirmation Modal */}
      <Modal
        title="Delete Index"
        open={deleteModalOpen}
        onCancel={handleDeleteCancel}
        footer={[
          <Button key="cancel" onClick={handleDeleteCancel}>
            Cancel
          </Button>,
          <Button
            key="delete"
            type="primary"
            danger
            loading={deletingIndexes.has(indexToDelete)}
            onClick={handleDeleteConfirm}
          >
            Delete
          </Button>,
        ]}
        width={500}
      >
        <p>Are you sure you want to delete the index <strong>"{indexToDelete}"</strong>?</p>
        <p className="text-red-600 text-sm mt-2">This action cannot be undone.</p>
      </Modal>
    </div>
  )
}
