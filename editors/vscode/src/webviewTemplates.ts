import * as vscode from 'vscode';

/**
 * Template-based webview content generator
 * This approach uses template literals with better organization
 */
export class WebviewTemplates {
    
    static createSearchEditor(webview: vscode.Webview, extensionUri: vscode.Uri): string {
        return this.htmlTemplate({
            title: 'Beetle Search Editor',
            css: this.searchEditorStyles(),
            body: this.searchEditorBody(),
            scripts: this.searchEditorScripts(),
            cspSource: webview.cspSource
        });
    }

    private static htmlTemplate(params: {
        title: string;
        css: string;
        body: string;
        scripts: string;
        cspSource: string;
    }): string {
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${params.cspSource} 'unsafe-inline'; script-src ${params.cspSource} 'unsafe-inline';">
    <title>${params.title}</title>
    <style>${params.css}</style>
</head>
<body>
    ${params.body}
    <script>${params.scripts}</script>
</body>
</html>`;
    }

    private static searchEditorStyles(): string {
        return `
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
        `;
    }

    private static searchEditorBody(): string {
        return `
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
        `;
    }

    private static searchEditorScripts(): string {
        return `
        const vscode = acquireVsCodeApi();
        
        // DOM Elements
        const indexSelect = document.getElementById('indexSelect');
        const searchInput = document.getElementById('searchInput');
        const searchButton = document.getElementById('searchButton');
        const resultsCount = document.getElementById('resultsCount');
        const resultsContent = document.getElementById('resultsContent');
        const errorContainer = document.getElementById('errorContainer');
        
        // Initialize
        init();

        function init() {
            vscode.postMessage({ type: 'getIndexes' });
            
            indexSelect.addEventListener('change', updateSearchButtonState);
            searchInput.addEventListener('input', updateSearchButtonState);
            searchInput.addEventListener('keydown', handleKeyDown);
            searchButton.addEventListener('click', performSearch);
            
            window.addEventListener('message', handleMessage);
        }

        function handleKeyDown(e) {
            if (e.key === 'Enter' && !searchButton.disabled) {
                performSearch();
            }
        }

        function handleMessage(event) {
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
        }

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
            
            showLoading();
            clearError();
            
            vscode.postMessage({
                type: 'search',
                indexName: indexSelect.value,
                query: searchInput.value.trim()
            });
        }

        function showLoading() {
            resultsContent.innerHTML = '<div class="loading">Searching...</div>';
            resultsCount.textContent = 'Searching...';
        }

        function displayResults(results, query) {
            if (results.length === 0) {
                resultsContent.innerHTML = \`<div class="empty-state">No results found for "\${query}"</div>\`;
                resultsCount.textContent = 'No Results';
                return;
            }
            
            resultsCount.textContent = \`\${results.length} result\${results.length === 1 ? '' : 's'} for "\${query}"\`;
            
            let html = '<div class="results-list">';
            results.forEach(result => {
                const fileName = result.path.split(/[\\\/]/).pop() || '';
                const lineNumber = result.line_number || 1;
                const snippet = result.snippet || result.content || '';
                
                html += \`
                    <div class="result-item" onclick="openFile('\${result.path}', \${lineNumber})">
                        <div class="result-header">
                            <span class="file-name">\${fileName}</span>
                            <span class="line-number">Line \${lineNumber}</span>
                        </div>
                        <div class="file-path">\${result.path}</div>
                        <div class="code-snippet">\${snippet}</div>
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
        `;
    }
}
