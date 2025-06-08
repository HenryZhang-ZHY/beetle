import * as vscode from 'vscode';
import { BeetleService } from './beetleService';
import { SearchResult } from './types';
import { WebviewTemplates } from './webviewTemplates';

/**
 * Alternative SearchEditorProvider using template-based approach
 * This version demonstrates better organization without separate files
 */
export class SearchEditorProviderTemplate {
    private static readonly viewType = 'beetle.searchEditorTemplate';
    private readonly beetleService: BeetleService;

    constructor(beetleService: BeetleService) {
        this.beetleService = beetleService;
    }

    public async openSearchEditor(context: vscode.ExtensionContext) {
        const panel = vscode.window.createWebviewPanel(
            SearchEditorProviderTemplate.viewType,
            'Beetle Search Editor (Template)',
            vscode.ViewColumn.One,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
                localResourceRoots: [vscode.Uri.joinPath(context.extensionUri, 'media')]
            }
        );

        panel.webview.html = WebviewTemplates.createSearchEditor(panel.webview, context.extensionUri);
        
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
}
