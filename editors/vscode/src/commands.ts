import * as vscode from 'vscode';
import * as fs from 'fs';
import { BeetleService } from './beetleService';
import { SearchResultProvider, IndexProvider, IndexItem } from './treeProviders';
import { SearchEditorProvider } from './searchEditor';

export function registerCommands(
	context: vscode.ExtensionContext,
	beetleService: BeetleService,
	searchResultProvider: SearchResultProvider,
	indexProvider: IndexProvider,
	searchEditorProvider: SearchEditorProvider
): void {
	
	const commands = [
		vscode.commands.registerCommand('beetle.createIndex', async () => {
			const name = await vscode.window.showInputBox({
				placeHolder: 'Enter index name',
				prompt: 'Choose a unique name for your index'
			});

			if (!name) { return; }

			// Default to current workspace if available
			const workspaceFolders = vscode.workspace.workspaceFolders;
			let defaultPath = '';
			if (workspaceFolders && workspaceFolders.length > 0) {
				defaultPath = workspaceFolders[0].uri.fsPath;
			}

			const folderUri = await vscode.window.showOpenDialog({
				canSelectFolders: true,
				canSelectFiles: false,
				canSelectMany: false,
				openLabel: 'Select Repository Folder',
				defaultUri: defaultPath ? vscode.Uri.file(defaultPath) : undefined
			});

			if (!folderUri || folderUri.length === 0) { return; }
			const repoPath = folderUri[0].fsPath;

			if (!repoPath) { return; }

			// Validate path exists
			if (!fs.existsSync(repoPath)) {
				vscode.window.showErrorMessage('The specified path does not exist');
				return;
			}

			vscode.window.withProgress({
				location: vscode.ProgressLocation.Notification,
				title: `Creating index "${name}"...`,
				cancellable: false
			}, async () => {
				const success = await beetleService.createIndex(name, repoPath);
				if (success) {
					indexProvider.refresh();
				}
			});
		}),

		vscode.commands.registerCommand('beetle.listIndexes', async () => {
			const indexes = await beetleService.listIndexes();

			if (indexes.length === 0) {
				vscode.window.showInformationMessage('No indexes found');
				return;
			}

			const items = indexes.map(idx => ({
				label: idx.name,
				description: (idx.file_count ?? 0) > 0 ? `${idx.file_count} files` : '',
				detail: idx.path
			}));

			vscode.window.showQuickPick(items, {
				placeHolder: 'Available indexes'
			});
		}),
		vscode.commands.registerCommand('beetle.deleteIndex', async (item?: IndexItem) => {
			let indexName: string;

			if (item) {
				indexName = item.index.name;
			} else {
				const indexes = await beetleService.listIndexes();
				if (indexes.length === 0) {
					vscode.window.showInformationMessage('No indexes found');
					return;
				}

				const selected = await vscode.window.showQuickPick(
					indexes.map(idx => idx.name),
					{ placeHolder: 'Select an index to delete' }
				);

				if (!selected) { return; }
				indexName = selected;
			}

			const confirm = await vscode.window.showWarningMessage(
				`Are you sure you want to delete the index "${indexName}"?`,
				{ modal: true },
				'Delete'
			);

			if (confirm === 'Delete') {
				vscode.window.withProgress({
					location: vscode.ProgressLocation.Notification,
					title: `Deleting index "${indexName}"...`,
					cancellable: false
				}, async () => {
					const success = await beetleService.deleteIndex(indexName);
					if (success) {
						indexProvider.refresh();
					}
				});
			}
		}),

		vscode.commands.registerCommand('beetle.openSearchPanel', () => {
			vscode.commands.executeCommand('workbench.view.extension.beetle');
		}),
		
		vscode.commands.registerCommand('beetle.refreshIndexes', () => {
			indexProvider.refresh();
		}),
		vscode.commands.registerCommand('beetle.openSearchEditor', async () => {
			await vscode.commands.executeCommand('beetleSearchEditor.focus');
		}),
	];

	// Add all commands to subscriptions
	commands.forEach(cmd => context.subscriptions.push(cmd));
}
