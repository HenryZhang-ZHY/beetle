// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import * as path from 'path';
import { BeetleService } from './beetleService';
import { SearchResultProvider, IndexProvider } from './treeProviders';
import { SearchEditorProvider } from './searchEditor';
import { registerCommands } from './commands';

// Global variables
let beetleService: BeetleService;
let searchResultProvider: SearchResultProvider;
let indexProvider: IndexProvider;

// This method is called when your extension is activated
export function activate(context: vscode.ExtensionContext) {
	console.log('Beetle extension is now active!');
	// Initialize services
	beetleService = new BeetleService();
	searchResultProvider = new SearchResultProvider();
	indexProvider = new IndexProvider(beetleService);

	// Create and register SearchEditorProvider
	const searchEditorProvider = new SearchEditorProvider(beetleService, context.extensionUri);
	
	// Set context for when extension is enabled
	vscode.commands.executeCommand('setContext', 'beetle.enabled', true);

	// Register WebviewViewProvider for Search Editor
	context.subscriptions.push(
		vscode.window.registerWebviewViewProvider(SearchEditorProvider.viewType, searchEditorProvider)
	);

	vscode.window.createTreeView('beetleIndexes', {
		treeDataProvider: indexProvider,
		showCollapseAll: true
	});
	// Register commands
	registerCommands(context, beetleService, searchResultProvider, indexProvider, searchEditorProvider);

	// Add service cleanup
	context.subscriptions.push(beetleService);

	// Auto-create index for current workspace if configured
	const autoCreate = vscode.workspace.getConfiguration('beetle').get<boolean>('autoCreateIndex', false);
	if (autoCreate && vscode.workspace.workspaceFolders) {
		const workspaceFolder = vscode.workspace.workspaceFolders[0];
		const workspaceName = path.basename(workspaceFolder.uri.fsPath);
		setTimeout(async () => {
			const indexes = await beetleService.listIndexes();
			const hasWorkspaceIndex = indexes.some((idx: any) => idx.name === workspaceName);

			if (!hasWorkspaceIndex) {
				const create = await vscode.window.showInformationMessage(
					`Would you like to create a Beetle index for the "${workspaceName}" workspace?`,
					'Create Index'
				);

				if (create) {
					await beetleService.createIndex(workspaceName, workspaceFolder.uri.fsPath);
					indexProvider.refresh();
				}
			}
		}, 2000); // Wait 2 seconds after activation
	}
}

// This method is called when your extension is deactivated
export function deactivate() {
	if (beetleService) {
		beetleService.dispose();
	}
}
