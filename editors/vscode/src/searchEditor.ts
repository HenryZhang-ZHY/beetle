import * as vscode from 'vscode';
import { BeetleService } from './beetleService';
import { SearchResult } from './types';

export class SearchEditorProvider {
    private static readonly viewType = 'beetle.searchEditor';
    private readonly beetleService: BeetleService;

    constructor(beetleService: BeetleService) {
        this.beetleService = beetleService;
    }

    public async openSearchEditor(context: vscode.ExtensionContext) {
        const panel = vscode.window.createWebviewPanel(
            SearchEditorProvider.viewType,
            'Beetle Search Editor',
            vscode.ViewColumn.One,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
                localResourceRoots: [vscode.Uri.joinPath(context.extensionUri, 'media')]
            }
        );

        panel.webview.html = this.getWebviewContent(panel.webview, context.extensionUri);
        
        // Handle messages from the webview
        panel.webview.onDidReceiveMessage(
            async (message) => {
                switch (message.type) {
                    case 'getIndexes':
                        const indexes = await this.beetleService.listIndexes();
                        panel.webview.postMessage({
                            type: 'indexesLoaded',
                            indexes: indexes
                        });
                        break;
                    
                    case 'search':
                        if (message.indexName && message.query) {
                            try {
                                const results = await this.beetleService.searchCode(message.indexName, message.query);
                                panel.webview.postMessage({
                                    type: 'searchResults',
                                    results: results,
                                    query: message.query
                                });
                            } catch (error) {
                                panel.webview.postMessage({
                                    type: 'searchError',
                                    error: error instanceof Error ? error.message : 'Search failed'
                                });
                            }
                        }
                        break;
                    
                    case 'openFile':
                        if (message.filePath && message.lineNumber) {
                            const uri = vscode.Uri.file(message.filePath);
                            const doc = await vscode.workspace.openTextDocument(uri);
                            const editor = await vscode.window.showTextDocument(doc);
                            
                            // Jump to the specific line
                            const line = Math.max(0, (message.lineNumber || 1) - 1);
                            const position = new vscode.Position(line, 0);
                            editor.selection = new vscode.Selection(position, position);
                            editor.revealRange(new vscode.Range(position, position));
                        }
                        break;
                }
            },
            undefined,
            context.subscriptions
        );
    }

    private getWebviewContent(webview: vscode.Webview, extensionUri: vscode.Uri): string {
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Beetle Search Editor</title>
    <style>
        body {
            font-family: var(--vscode-font-family);
            font-size: var(--vscode-font-size);
            color: var(--vscode-foreground);
            background-color: var(--vscode-editor-background);
            margin: 0;
            padding: 20px;
        }
        
        .search-container {
            margin-bottom: 20px;
            display: flex;
            gap: 10px;
            align-items: center;
            flex-wrap: wrap;
        }
        
        .search-input, .index-select {
            padding: 8px;
            border: 1px solid var(--vscode-input-border);
            background-color: var(--vscode-input-background);
            color: var(--vscode-input-foreground);
            border-radius: 4px;
        }
        
        .search-input {
            flex: 1;
            min-width: 300px;
        }
        
        .index-select {
            min-width: 150px;
        }
        
        .search-button {
            padding: 8px 16px;
            background-color: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            border: none;
            border-radius: 4px;
            cursor: pointer;
        }
        
        .search-button:hover {
            background-color: var(--vscode-button-hoverBackground);
        }
        
        .search-button:disabled {
            opacity: 0.5;
            cursor: not-allowed;
        }
        
        .results-container {
            border: 1px solid var(--vscode-panel-border);
            border-radius: 4px;
            overflow: hidden;
        }
        
        .results-header {
            background-color: var(--vscode-panel-background);
            padding: 10px;
            border-bottom: 1px solid var(--vscode-panel-border);
            font-weight: bold;
        }
        
        .results-grid {
            display: table;
            width: 100%;
            border-collapse: collapse;
        }
        
        .results-header-row {
            display: table-row;
            background-color: var(--vscode-list-activeSelectionBackground);
            font-weight: bold;
        }
        
        .results-row {
            display: table-row;
            cursor: pointer;
            transition: background-color 0.1s ease;
        }
        
        .results-row:hover {
            background-color: var(--vscode-list-hoverBackground);
        }
        
        .results-row:nth-child(even) {
            background-color: var(--vscode-list-inactiveSelectionBackground);
        }
        
        .results-cell {
            display: table-cell;
            padding: 8px 12px;
            border-bottom: 1px solid var(--vscode-panel-border);
            vertical-align: top;
        }
        
        .file-path {
            font-family: var(--vscode-editor-font-family);
            font-size: 0.9em;
            color: var(--vscode-descriptionForeground);
        }
        
        .file-name {
            font-weight: bold;
            color: var(--vscode-foreground);
        }
        
        .code-line {
            font-family: var(--vscode-editor-font-family);
            font-size: 0.9em;
            background-color: var(--vscode-textCodeBlock-background);
            padding: 4px 8px;
            border-radius: 3px;
            max-width: 400px;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }
        
        .line-number {
            color: var(--vscode-editorLineNumber-foreground);
            font-family: var(--vscode-editor-font-family);
            font-size: 0.8em;
        }
        
        .empty-state {
            text-align: center;
            padding: 40px;
            color: var(--vscode-descriptionForeground);
        }
        
        .loading {
            text-align: center;
            padding: 20px;
            color: var(--vscode-descriptionForeground);
        }
        
        .error {
            color: var(--vscode-errorForeground);
            background-color: var(--vscode-inputValidation-errorBackground);
            padding: 8px 12px;
            border-radius: 4px;
            margin-bottom: 10px;
        }
    </style>
</head>
<body>
    <div class="search-container">
        <select id="indexSelect" class="index-select">
            <option value="">Select an index...</option>
        </select>
        <input type="text" id="searchInput" class="search-input" placeholder="Enter search query...">
        <button id="searchButton" class="search-button" disabled>Search</button>
    </div>
    
    <div id="errorContainer"></div>
    
    <div class="results-container">
        <div class="results-header">
            <span id="resultsCount">Search Results</span>
        </div>
        <div id="resultsContent">
            <div class="empty-state">
                Select an index and enter a search query to begin
            </div>
        </div>
    </div>

    <script>
        const vscode = acquireVsCodeApi();
        
        const indexSelect = document.getElementById('indexSelect');
        const searchInput = document.getElementById('searchInput');
        const searchButton = document.getElementById('searchButton');
        const resultsCount = document.getElementById('resultsCount');
        const resultsContent = document.getElementById('resultsContent');
        const errorContainer = document.getElementById('errorContainer');
        
        let currentResults = [];
        
        // Request indexes on load
        vscode.postMessage({ type: 'getIndexes' });
        
        // Event listeners
        indexSelect.addEventListener('change', updateSearchButtonState);
        searchInput.addEventListener('input', updateSearchButtonState);
        searchInput.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && !searchButton.disabled) {
                performSearch();
            }
        });
        searchButton.addEventListener('click', performSearch);
        
        // Listen for messages from the extension
        window.addEventListener('message', event => {
            const message = event.data;
            
            switch (message.type) {
                case 'indexesLoaded':
                    populateIndexes(message.indexes);
                    break;
                case 'searchResults':
                    displayResults(message.results, message.query);
                    break;
                case 'searchError':
                    showError(message.error);
                    break;
            }
        });
        
        function updateSearchButtonState() {
            const hasIndex = indexSelect.value !== '';
            const hasQuery = searchInput.value.trim() !== '';
            searchButton.disabled = !hasIndex || !hasQuery;
        }
        
        function populateIndexes(indexes) {
            indexSelect.innerHTML = '<option value="">Select an index...</option>';
            indexes.forEach(index => {
                const option = document.createElement('option');
                option.value = index.name;
                option.textContent = index.name + (index.file_count ? \` (\${index.file_count} files)\` : '');
                indexSelect.appendChild(option);
            });
            updateSearchButtonState();
        }
        
        function performSearch() {
            if (searchButton.disabled) return;
            
            const indexName = indexSelect.value;
            const query = searchInput.value.trim();
            
            showLoading();
            clearError();
            
            vscode.postMessage({
                type: 'search',
                indexName: indexName,
                query: query
            });
        }
        
        function showLoading() {
            resultsContent.innerHTML = '<div class="loading">Searching...</div>';
            resultsCount.textContent = 'Searching...';
        }
        
        function displayResults(results, query) {
            currentResults = results;
            
            if (results.length === 0) {
                resultsContent.innerHTML = \`<div class="empty-state">No results found for "\${query}"</div>\`;
                resultsCount.textContent = 'No Results';
                return;
            }
            
            resultsCount.textContent = \`\${results.length} result\${results.length === 1 ? '' : 's'} for "\${query}"\`;
            
            let html = '<div class="results-grid">';
            
            // Header row
            html += \`
                <div class="results-header-row">
                    <div class="results-cell">File Path</div>
                    <div class="results-cell">File Name</div>
                    <div class="results-cell">Code Line</div>
                </div>
            \`;
            
            // Data rows
            results.forEach((result, index) => {
                const fileName = result.path.split(/[\\\\/]/).pop() || '';
                const lineNumber = result.line_number || 1;
                const snippet = result.snippet || result.content || '';
                
                html += \`
                    <div class="results-row" onclick="openFile('\${result.path}', \${lineNumber})">
                        <div class="results-cell">
                            <div class="file-path">\${result.path}</div>
                        </div>
                        <div class="results-cell">
                            <div class="file-name">\${fileName}</div>
                            <div class="line-number">Line \${lineNumber}</div>
                        </div>
                        <div class="results-cell">
                            <div class="code-line">\${snippet}</div>
                        </div>
                    </div>
                \`;
            });
            
            html += '</div>';
            resultsContent.innerHTML = html;
        }
        
        function openFile(filePath, lineNumber) {
            vscode.postMessage({
                type: 'openFile',
                filePath: filePath,
                lineNumber: lineNumber
            });
        }
        
        function showError(errorMessage) {
            errorContainer.innerHTML = \`<div class="error">Error: \${errorMessage}</div>\`;
        }
        
        function clearError() {
            errorContainer.innerHTML = '';
        }
    </script>
</body>
</html>`;
    }
}
