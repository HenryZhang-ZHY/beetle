// Acquire VS Code API
const vscode = acquireVsCodeApi();

// DOM Elements
const indexSelect = document.getElementById('indexSelect');
const searchInput = document.getElementById('searchInput');
const searchButton = document.getElementById('searchButton');
const resultsCount = document.getElementById('resultsCount');
const resultsContent = document.getElementById('resultsContent');
const errorContainer = document.getElementById('errorContainer');
const resizeStatus = document.getElementById('resizeStatus');

// State
let currentResults = [];

// Column width persistence
const COLUMN_WIDTHS_KEY = 'beetle-search-column-widths';

// Initialize
init();

function init() {
    // Request indexes on load
    vscode.postMessage({ type: 'getIndexes' });
    
    // Event listeners
    indexSelect.addEventListener('change', updateSearchButtonState);
    searchInput.addEventListener('input', updateSearchButtonState);
    searchInput.addEventListener('keydown', handleKeyDown);
    searchButton.addEventListener('click', performSearch);
    
    // Listen for messages from the extension
    window.addEventListener('message', handleMessage);
    
    // Apply saved column widths
    applyColumnWidths();
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
        option.textContent = index.name + (index.file_count ? ` (${index.file_count} files)` : '');
        indexSelect.appendChild(option);
    });
    updateSearchButtonState();
}

function performSearch() {
    if (searchButton.disabled) {
        return;
    }
    
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
    const helpText = document.querySelector('.help-text');
    
    if (results.length === 0) {
        resultsContent.innerHTML = `<div class="empty-state">No results found for "${query}"</div>`;
        resultsCount.textContent = 'No Results';
        if (helpText) {
            helpText.style.display = 'none';
        }
        return;
    }
    
    // Show help text when there are results
    if (helpText) {
        helpText.style.display = 'inline';
    }
    
    resultsCount.textContent = `${results.length} result${results.length === 1 ? '' : 's'} for "${query}"`;
    
    let html = '<table class="results-grid">';
    
    // Header row with resizable columns
    html += `
        <thead>
            <tr class="results-header-row">
                <th class="results-cell column-header column-file-path">
                    File Path
                    <div class="column-resizer" data-column="0"></div>
                </th>
                <th class="results-cell column-header column-file-name">
                    File Name
                    <div class="column-resizer" data-column="1"></div>
                </th>
                <th class="results-cell column-header column-code-line">
                    Code Line
                    <div class="column-resizer" data-column="2"></div>
                </th>
            </tr>
        </thead>
        <tbody>
    `;
      // Data rows
    results.forEach((result, index) => {
        const fileName = result.path.split(/[\\\/]/).pop() || '';
        const lineNumber = result.line_number || 1;
        const snippet = result.snippet || result.content || '';
        
        html += `
            <tr class="results-row" data-file-path="${result.path}" data-line-number="${lineNumber}">
                <td class="results-cell column-file-path">
                    <div class="file-path">${result.path}</div>
                </td>
                <td class="results-cell column-file-name">
                    <div class="file-name">${fileName}</div>
                    <div class="line-number">Line ${lineNumber}</div>
                </td>
                <td class="results-cell column-code-line">
                    <div class="code-line">${snippet}</div>
                </td>
            </tr>
        `;
    });
      html += '</tbody></table>';
    resultsContent.innerHTML = html;
    
    // Add double-click event listeners to result rows
    const resultRows = document.querySelectorAll('.results-row');
    resultRows.forEach(row => {
        row.addEventListener('dblclick', function() {
            const filePath = this.dataset.filePath;
            const lineNumber = parseInt(this.dataset.lineNumber) || 1;
            openFile(filePath, lineNumber);
        });
        
        // Optional: Add visual feedback for clickable rows
        row.style.cursor = 'pointer';
    });
      // Initialize column resizing
    initializeColumnResizing();
    
    // Apply saved column widths
    applyColumnWidths();
}

function openFile(filePath, lineNumber) {
    vscode.postMessage({
        type: 'openFile',
        filePath: filePath,
        lineNumber: lineNumber
    });
}

function showError(errorMessage) {
    errorContainer.innerHTML = `<div class="error">Error: ${errorMessage}</div>`;
}

function clearError() {
    errorContainer.innerHTML = '';
}

function initializeColumnResizing() {
    const resizers = document.querySelectorAll('.column-resizer');
    let isResizing = false;
    let currentResizer = null;
    let startX = 0;
    let startWidth = 0;
    let currentTable = null;
    
    resizers.forEach(resizer => {
        resizer.addEventListener('mousedown', initResize);
    });
    
    function initResize(e) {
        currentResizer = e.target;
        isResizing = true;
        startX = e.clientX;
        
        const columnIndex = parseInt(currentResizer.dataset.column);
        currentTable = document.querySelector('.results-grid');
        const headerCells = currentTable.querySelectorAll('thead th');
        const targetCell = headerCells[columnIndex];
        startWidth = targetCell.offsetWidth;
        
        currentResizer.classList.add('resizing');
        document.body.classList.add('resizing-active');
        document.addEventListener('mousemove', doResize);
        document.addEventListener('mouseup', stopResize);
        
        e.preventDefault();
        e.stopPropagation();
    }
    
    function doResize(e) {
        if (!isResizing || !currentResizer || !currentTable) {
            return;
        }
        
        const columnIndex = parseInt(currentResizer.dataset.column);
        const headerCells = currentTable.querySelectorAll('thead th');
        const targetCell = headerCells[columnIndex];
        
        const diff = e.clientX - startX;
        const minWidth = 80;
        const newWidth = Math.max(minWidth, startWidth + diff);
        const tableWidth = currentTable.offsetWidth;
        const percentage = Math.min(80, Math.max(10, (newWidth / tableWidth) * 100));
        
        // Show resize status
        const columnNames = ['File Path', 'File Name', 'Code Line'];
        resizeStatus.textContent = `Resizing ${columnNames[columnIndex]}: ${Math.round(newWidth)}px (${Math.round(percentage)}%)`;
        
        // Apply the new width directly to the column
        const className = targetCell.className.split(' ').find(c => c.startsWith('column-'));
        if (className) {
            // Remove any existing dynamic style for this column
            const existingStyle = document.querySelector(`style[data-column="${className}"]`);
            if (existingStyle) {
                existingStyle.remove();
            }
            
            // Create new style
            const style = document.createElement('style');
            style.textContent = `
                .${className} { 
                    width: ${percentage}% !important; 
                    min-width: ${minWidth}px !important;
                }
            `;
            style.setAttribute('data-column', className);
            document.head.appendChild(style);
        }
        
        e.preventDefault();
    }
    
    function stopResize(e) {
        if (!isResizing) {
            return;
        }
        
        isResizing = false;
        document.body.classList.remove('resizing-active');
        
        // Clear resize status after a delay
        setTimeout(() => {
            resizeStatus.textContent = '';
        }, 1000);
        
        if (currentResizer) {
            currentResizer.classList.remove('resizing');
            currentResizer = null;
        }
        
        currentTable = null;
        
        document.removeEventListener('mousemove', doResize);
        document.removeEventListener('mouseup', stopResize);
        
        // Save column widths after resizing
        saveColumnWidths();
    }
}

// Add helper functions for persisting column widths
function saveColumnWidths() {
    const table = document.querySelector('.results-grid');
    if (!table) {
        return;
    }
    
    const headerCells = table.querySelectorAll('thead th');
    const widths = {};
    
    headerCells.forEach((cell, index) => {
        const className = cell.className.split(' ').find(c => c.startsWith('column-'));
        if (className) {
            const width = (cell.offsetWidth / table.offsetWidth) * 100;
            widths[className] = width;
        }
    });
    
    try {
        vscode.setState({ columnWidths: widths });
    } catch (e) {
        // Fallback to localStorage if vscode.setState is not available
        localStorage.setItem(COLUMN_WIDTHS_KEY, JSON.stringify(widths));
    }
}

function loadColumnWidths() {
    try {
        const state = vscode.getState();
        return state?.columnWidths || JSON.parse(localStorage.getItem(COLUMN_WIDTHS_KEY) || '{}');
    } catch (e) {
        return {};
    }
}

function applyColumnWidths() {
    const savedWidths = loadColumnWidths();
    
    Object.entries(savedWidths).forEach(([className, width]) => {
        const existingStyle = document.querySelector(`style[data-column="${className}"]`);
        if (existingStyle) {
            existingStyle.remove();
        }
        
        const style = document.createElement('style');
        style.textContent = `
            .${className} { 
                width: ${width}% !important; 
                min-width: 80px !important;
            }
        `;
        style.setAttribute('data-column', className);
        document.head.appendChild(style);
    });
}
