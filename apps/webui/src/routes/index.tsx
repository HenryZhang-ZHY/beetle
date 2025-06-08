import { createFileRoute, Link } from '@tanstack/react-router'
import { useState, useEffect } from 'react'
import { Button, Select, Input, Spin, } from 'antd'
import { SearchOutlined, FileTextOutlined, FolderOutlined, SettingOutlined } from '@ant-design/icons'

const { Search } = Input
const { Option } = Select

interface IndexResponse {
  name: string
  path: string
}

interface SearchResult {
  path: string
  score: number
  extension: string
  snippet: string
}

interface SearchResponse {
  query: string
  index_name: string
  results: SearchResult[]
  total_results: number
}

export const Route = createFileRoute('/')({
  component: IndexPage,
})

function IndexPage() {
  const [indexes, setIndexes] = useState<IndexResponse[]>([])
  const [selectedIndex, setSelectedIndex] = useState<string>('')
  const [searchQuery, setSearchQuery] = useState<string>('')
  const [searchResults, setSearchResults] = useState<SearchResponse | null>(null)
  const [loading, setLoading] = useState(false)
  const [loadingIndexes, setLoadingIndexes] = useState(true)

  const fetchIndexes = async () => {
    try {
      setLoadingIndexes(true)
      const response = await fetch('/api/indexes')
      if (!response.ok) throw new Error('Failed to fetch indexes')
      const data = await response.json()
      setIndexes(data)
      if (data.length > 0) {
        setSelectedIndex(data[0].name)
      }
    } catch (error) {
      console.error('Error fetching indexes:', error)
    } finally {
      setLoadingIndexes(false)
    }
  }

  const handleSearch = async (query: string) => {
    if (!query.trim() || !selectedIndex) return

    setLoading(true)
    try {
      const response = await fetch(`/api/indexes/${selectedIndex}/search?q=${encodeURIComponent(query)}`)
      if (!response.ok) throw new Error('Search failed')
      const data = await response.json()
      setSearchResults(data)
    } catch (error) {
      console.error('Error searching:', error)
      setSearchResults(null)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchIndexes()
  }, [])

  return (
    <div className="min-h-screen bg-slate-50">
      {/* Header with Search */}
      <header className="bg-white border-b border-slate-200 px-6 py-6">
        <div className="mx-auto">
          <div className="flex items-center justify-center gap-8">
            {/* Left: Icon and Title */}
            <div className="flex items-center space-x-3 min-w-0">
              <div>
                <h1 className="text-2xl font-bold text-slate-900">Beetle</h1>
              </div>
            </div>

            {/* Middle: Search Bar */}
            <div className="flex gap-4 items-end flex-1">
              <div className="w-56 flex-shrink-0">
                <Select
                  value={selectedIndex}
                  onChange={setSelectedIndex}
                  className="w-full"
                  placeholder="Select an index"
                  loading={loadingIndexes}
                  size="large"
                >
                  {indexes.map(index => (
                    <Option key={index.name} value={index.name}>
                      <div className="flex items-center space-x-2">
                        <FolderOutlined className="text-slate-400" />
                        <span>{index.name}</span>
                      </div>
                    </Option>
                  ))}
                </Select>
              </div>

              <div className="flex-1 min-w-0">
                <Search
                  placeholder="Search functions, variables, files..."
                  allowClear
                  enterButton={
                    <Button type="primary" className="bg-slate-900 hover:bg-slate-800 border-slate-900 h-10">
                      <SearchOutlined />
                    </Button>
                  }
                  size="large"
                  className="h-10"
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  onSearch={handleSearch}
                  disabled={!selectedIndex || indexes.length === 0}
                />
              </div>
            </div>

            {/* Right: Manage Indexes Button */}
            <Link to="/indexes">
              <Button
                type="text"
                icon={<SettingOutlined />}
                className="text-slate-600 hover:text-slate-900 hover:bg-slate-100 border-0 h-10 px-4"
                size="large"
              >
              </Button>
            </Link>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-6 py-8">

        {/* Only show this when user has no indexes */}
        {indexes.length === 0 && !loadingIndexes && (
          <div className="text-center py-16">
            <div className="bg-white rounded-lg border border-slate-200 p-8 shadow-sm">
              <FolderOutlined className="text-4xl text-slate-400 mb-4" />
              <h3 className="text-lg font-medium text-slate-900 mb-2">No indexes found</h3>
              <p className="text-slate-600 mb-6">
                Create your first index to start searching your code
              </p>
              <Link to="/indexes">
                <Button type="primary" size="large" className="bg-slate-900 hover:bg-slate-800 border-slate-900">
                  Create Your First Index
                </Button>
              </Link>
            </div>
          </div>
        )}

        {/* Loading State */}
        {loading && (
          <div className="flex justify-center py-12">
            <Spin size="large" />
          </div>
        )}

        {/* Search Results */}
        {searchResults && !loading && (
          <div className="space-y-6">
            <div className="flex items-center justify-between">
              <div>
                <h2 className="text-lg font-semibold text-slate-900">
                  Search Results
                </h2>
                <p className="text-sm text-slate-600">
                  {searchResults.total_results} results for "{searchResults.query}" in {searchResults.index_name}
                </p>
              </div>
            </div>

            {searchResults.results.length === 0 ? (
              <div className="text-center py-12">
                <div className="bg-white rounded-lg border border-slate-200 p-8 shadow-sm">
                  <SearchOutlined className="text-4xl text-slate-400 mb-4" />
                  <h3 className="text-lg font-medium text-slate-900 mb-2">No results found</h3>
                  <p className="text-slate-600">
                    Try adjusting your search query or check a different index
                  </p>
                </div>
              </div>
            ) : (
              <div className="space-y-4">
                {searchResults.results.map((result, index) => (
                  <div key={index} className="bg-white rounded-lg border border-slate-200 p-4 shadow-sm hover:shadow-md transition-shadow">
                    <div className="flex items-start space-x-3">
                      <FileTextOutlined className="text-slate-400 mt-1 text-lg" />
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center space-x-3 mb-3">
                          <code className="text-sm font-medium text-slate-900 bg-slate-100 px-2 py-1 rounded">
                            {result.path}
                          </code>
                          <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                            {Math.round(result.score * 100)}% match
                          </span>
                        </div>
                        <div className="bg-slate-50 border border-slate-200 rounded-md p-3 overflow-x-auto">
                          <pre
                            className="text-xs text-slate-700 font-mono whitespace-pre-wrap"
                            dangerouslySetInnerHTML={{ __html: result.snippet }}
                          />
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {/* Welcome state for when user has indexes but hasn't searched yet */}
        {indexes.length > 0 && !searchResults && !loading && (
          <div className="text-center py-16">
            <div className="max-w-md mx-auto">
              <SearchOutlined className="text-4xl text-slate-400 mb-4" />
              <h3 className="text-lg font-medium text-slate-900 mb-2">Ready to search</h3>
              <p className="text-slate-600">
                Enter a search query above to find code across your indexed repositories
              </p>
            </div>
          </div>
        )}
      </main>
    </div>
  )
}
