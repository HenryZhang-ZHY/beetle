import * as vscode from 'vscode';
import * as fs from 'fs';
import { BeetleService } from './beetleService';
import { SearchResult } from './types';

export class SearchEditorProvider implements vscode.WebviewViewProvider {
    public static readonly viewType = 'beetleSearchEditor';
    private readonly beetleService: BeetleService;
    private readonly extensionUri: vscode.Uri;
    private _view?: vscode.WebviewView;

    constructor(beetleService: BeetleService, extensionUri: vscode.Uri) {
        this.beetleService = beetleService;
        this.extensionUri = extensionUri;
    }

    public resolveWebviewView(
        webviewView: vscode.WebviewView,
        context: vscode.WebviewViewResolveContext,
        _token: vscode.CancellationToken,
    ) {
        this._view = webviewView;        webviewView.webview.options = {
            enableScripts: true,
            localResourceRoots: [
                vscode.Uri.joinPath(this.extensionUri, 'media')
            ]
        };

        webviewView.webview.html = this.getWebviewContent(webviewView.webview, this.extensionUri);

        // Handle messages from the webview
        webviewView.webview.onDidReceiveMessage(
            async (message) => {
                switch (message.type) {
                    case 'getIndexes':
                        const indexes = await this.beetleService.listIndexes();
                        webviewView.webview.postMessage({
                            type: 'indexesLoaded',
                            indexes: indexes
                        });
                        break;
                    
                    case 'search':
                        if (message.indexName && message.query) {
                            try {
                                const results = await this.beetleService.searchCode(message.indexName, message.query);
                                webviewView.webview.postMessage({
                                    type: 'searchResults',
                                    results: results,
                                    query: message.query
                                });
                            } catch (error) {
                                webviewView.webview.postMessage({
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
            }
        );
    }private getWebviewContent(webview: vscode.Webview, extensionUri: vscode.Uri): string {
        // Get the URIs for the HTML, CSS, and JavaScript files
        const htmlUri = vscode.Uri.joinPath(extensionUri, 'media', 'searchEditor.html');
        const cssUri = webview.asWebviewUri(vscode.Uri.joinPath(extensionUri, 'media', 'searchEditor.css'));
        const scriptUri = webview.asWebviewUri(vscode.Uri.joinPath(extensionUri, 'media', 'searchEditor.js'));
        
        // Read the HTML template and replace placeholders
        let htmlContent = fs.readFileSync(htmlUri.fsPath, 'utf8');
        
        // Replace placeholders with actual URIs
        htmlContent = htmlContent
            .replace(/#{cspSource}/g, webview.cspSource)
            .replace(/#{cssUri}/g, cssUri.toString())
            .replace(/#{scriptUri}/g, scriptUri.toString());
            
        return htmlContent;
    }
}
