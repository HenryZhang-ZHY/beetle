<template>
  <div class="container mx-auto py-6 space-y-6">
    <!-- Search Input Bar -->
    <div class="w-full max-w-10xl mx-auto">
      <div class="relative">
        <Search class="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground h-4 w-4" />
        <Input v-model="queryString" placeholder="Search..." class="pl-10 pr-4 py-2 w-full" @keyup.enter="search" />
      </div>
    </div>

    <!-- Results Data Table -->
    <div class="w-full">
      <div class="max-h-[70vh] overflow-auto">
        <Table>
          <TableHeader class="sticky top-0 bg-background z-10">
            <TableRow v-for="headerGroup in table.getHeaderGroups()" :key="headerGroup.id">
              <TableHead v-for="header in headerGroup.headers" :key="header.id">
                <FlexRender v-if="!header.isPlaceholder" :render="header.column.columnDef.header"
                  :props="header.getContext()" />
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <template v-if="table.getRowModel().rows?.length">
              <template v-for="row in table.getRowModel().rows" :key="row.id">
                <TableRow :data-state="row.getIsSelected() && 'selected'">
                  <TableCell v-for="cell in row.getVisibleCells()" :key="cell.id">
                    <FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" />
                  </TableCell>
                </TableRow>
                <TableRow v-if="row.getIsExpanded()">
                  <TableCell :colspan="row.getAllCells().length">
                    {{ JSON.stringify(row.original) }}
                  </TableCell>
                </TableRow>
              </template>
            </template>

            <TableRow v-else>
              <TableCell :colspan="columns.length" class="h-24 text-center">
                No results.
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>
      <div class="flex items-center justify-end space-x-2 py-4">
        <div class="flex-1 text-sm text-muted-foreground">
          {{ table.getFilteredRowModel().rows.length }} row(s).
        </div>
        <div class="space-x-2">
          <Button variant="outline" size="sm" :disabled="!table.getCanPreviousPage()" @click="table.previousPage()">
            Previous
          </Button>
          <Button variant="outline" size="sm" :disabled="!table.getCanNextPage()" @click="table.nextPage()">
            Next
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, h } from 'vue'
import { Search } from 'lucide-vue-next'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import type {
  ColumnDef,
  ColumnFiltersState,
  ExpandedState,
  SortingState,
  VisibilityState,
} from '@tanstack/vue-table'
import {
  FlexRender,
  getCoreRowModel,
  getExpandedRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useVueTable,
} from '@tanstack/vue-table'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'

const queryString = ref('')
const data = ref([])

const columns = reactive([
  {
    accessorKey: 'path',
    header: 'Path',
  },
  {
    accessorKey: 'snippet',
    header: 'Snippet',
    cell: ({ row }) => {
      return h('div', { innerHTML: row.getValue('snippet') })
    },
  },
])

const table = useVueTable({
  data,
  columns,
  getCoreRowModel: getCoreRowModel(),
  getPaginationRowModel: getPaginationRowModel(),
  getSortedRowModel: getSortedRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  getExpandedRowModel: getExpandedRowModel(),
  initialState: {
    pagination: {
      pageSize: 1000,
    },
  },
})

const search = async () => {
  const response = await fetch(`/api/indexes/beetle/search?q=${encodeURIComponent(queryString.value)}`)
  if (!response.ok) {
    console.error('Search request failed:', response.statusText)
    return
  }

  const results = await response.json()
  data.value = results.results
}

</script>

<style scoped>
:deep(b) {
  background-color: #dbebfe;
  padding: 2px 4px;
  border-radius: 3px;
}
</style>
