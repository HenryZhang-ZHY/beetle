// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import { spawn } from 'child_process';

// Interfaces for Beetle data structures
interface SearchResult {
	file_path: string;
	line_number: number;
	content: string;
	relevance_score: number;
}

interface BeetleIndex {
	name: string;
	path: string;
	created_at: string;
	file_count: number;
}

// Tree data providers
class SearchResultProvider implements vscode.TreeDataProvider<SearchResultItem> {
	private _onDidChangeTreeData: vscode.EventEmitter<SearchResultItem | undefined | void> = new vscode.EventEmitter<SearchResultItem | undefined | void>();
	readonly onDidChangeTreeData: vscode.Event<SearchResultItem | undefined | void> = this._onDidChangeTreeData.event;

	private results: SearchResult[] = [];
	private query: string = '';

	constructor() {}

	refresh(): void {
		this._onDidChangeTreeData.fire();
	}

	updateResults(query: string, results: SearchResult[]): void {
		this.query = query;
		this.results = results;
		this.refresh();
	}

	getTreeItem(element: SearchResultItem): vscode.TreeItem {
		return element;
	}

	getChildren(element?: SearchResultItem): Thenable<SearchResultItem[]> {
		if (!element) {
			return Promise.resolve(this.results.map(result => new SearchResultItem(
				result,
				vscode.TreeItemCollapsibleState.None
			)));
		}
		return Promise.resolve([]);
	}
}

class SearchResultItem extends vscode.TreeItem {
	constructor(
		public readonly result: SearchResult,
		public readonly collapsibleState: vscode.TreeItemCollapsibleState
	) {
		super(`${path.basename(result.file_path)}:${result.line_number}`, collapsibleState);
		
		this.tooltip = `${result.file_path}:${result.line_number}\nScore: ${result.relevance_score}`;
		this.description = result.content.trim().substring(0, 50) + '...';
		this.resourceUri = vscode.Uri.file(result.file_path);
		
		this.command = {
			command: 'vscode.open',
			title: 'Open File',
			arguments: [
				vscode.Uri.file(result.file_path),
				{
					selection: new vscode.Range(
						result.line_number - 1, 0,
						result.line_number - 1, result.content.length
					)
				}
			]
		};
		
		this.iconPath = new vscode.ThemeIcon('file-code');
	}
}

class IndexProvider implements vscode.TreeDataProvider<IndexItem> {
	private _onDidChangeTreeData: vscode.EventEmitter<IndexItem | undefined | void> = new vscode.EventEmitter<IndexItem | undefined | void>();
	readonly onDidChangeTreeData: vscode.Event<IndexItem | undefined | void> = this._onDidChangeTreeData.event;

	private indexes: BeetleIndex[] = [];

	constructor() {
		this.loadIndexes();
	}

	refresh(): void {
		this.loadIndexes();
		this._onDidChangeTreeData.fire();
	}

	getTreeItem(element: IndexItem): vscode.TreeItem {
		return element;
	}

	getChildren(element?: IndexItem): Thenable<IndexItem[]> {
		if (!element) {
			return Promise.resolve(this.indexes.map(index => new IndexItem(
				index,
				vscode.TreeItemCollapsibleState.None
			)));
		}
		return Promise.resolve([]);
	}

	private async loadIndexes(): Promise<void> {
		try {
			const result = await this.executeBeetleCommand(['list']);
			if (result.success) {
				// Parse the list output (assuming JSON format)
				try {
					this.indexes = JSON.parse(result.output);
				} catch {
					// If not JSON, parse as text
					this.indexes = this.parseTextIndexList(result.output);
				}
			}
		} catch (error) {
			console.error('Failed to load indexes:', error);
			this.indexes = [];
		}
	}

	private parseTextIndexList(output: string): BeetleIndex[] {
		const lines = output.split('\n').filter(line => line.trim());
		return lines.map(line => ({
			name: line.trim(),
			path: '',
			created_at: '',
			file_count: 0
		}));
	}

	private async executeBeetleCommand(args: string[]): Promise<{success: boolean, output: string}> {
		return new Promise((resolve) => {
			const beetlePath = vscode.workspace.getConfiguration('beetle').get<string>('executablePath', 'beetle');
			const process = spawn(beetlePath, args, { shell: true });
			
			let output = '';
			let error = '';

			process.stdout.on('data', (data: Buffer) => {
				output += data.toString();
			});

			process.stderr.on('data', (data: Buffer) => {
				error += data.toString();
			});

			process.on('close', (code: number | null) => {
				resolve({
					success: code === 0,
					output: code === 0 ? output : error
				});
			});
		});
	}
}

class IndexItem extends vscode.TreeItem {
	constructor(
		public readonly index: BeetleIndex,
		public readonly collapsibleState: vscode.TreeItemCollapsibleState
	) {
		super(index.name, collapsibleState);
		
		this.tooltip = `Path: ${index.path}\nCreated: ${index.created_at}\nFiles: ${index.file_count}`;
		this.description = index.file_count > 0 ? `${index.file_count} files` : '';
		this.contextValue = 'index';
		this.iconPath = new vscode.ThemeIcon('database');
	}
}

// Main Beetle service class
class BeetleService {
	private statusBarItem: vscode.StatusBarItem;

	constructor() {
		this.statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
		this.statusBarItem.text = "$(search) Beetle";
		this.statusBarItem.tooltip = "Beetle Code Search";
		this.statusBarItem.command = 'beetle.openSearchPanel';
		this.statusBarItem.show();
	}

	async executeBeetleCommand(args: string[]): Promise<{success: boolean, output: string}> {
		return new Promise((resolve) => {
			const beetlePath = vscode.workspace.getConfiguration('beetle').get<string>('executablePath', 'beetle');
			const process = spawn(beetlePath, args, { shell: true });
			
			let output = '';
			let error = '';

			process.stdout.on('data', (data: Buffer) => {
				output += data.toString();
			});

			process.stderr.on('data', (data: Buffer) => {
				error += data.toString();
			});

			process.on('close', (code: number | null) => {
				resolve({
					success: code === 0,
					output: code === 0 ? output : error
				});
			});
		});
	}

	async searchCode(indexName: string, query: string): Promise<SearchResult[]> {
		try {
			const result = await this.executeBeetleCommand(['search', indexName, '-q', query, '--formatter', 'json']);
			
			if (result.success) {
				try {
					const parsed = JSON.parse(result.output);
					return parsed.results || [];
				} catch {
					// If JSON parsing fails, try to parse as text
					return this.parseTextSearchResults(result.output);
				}
			} else {
				vscode.window.showErrorMessage(`Search failed: ${result.output}`);
				return [];
			}
		} catch (error) {
			vscode.window.showErrorMessage(`Search error: ${error}`);
			return [];
		}
	}

	private parseTextSearchResults(output: string): SearchResult[] {
		const lines = output.split('\n').filter(line => line.trim());
		const results: SearchResult[] = [];
		
		for (const line of lines) {
			// Basic parsing for text format - this would need to be adjusted based on actual output format
			const match = line.match(/^(.+):(\d+):(.+)$/);
			if (match) {
				results.push({
					file_path: match[1],
					line_number: parseInt(match[2]),
					content: match[3],
					relevance_score: 1.0
				});
			}
		}
		
		return results;
	}

	async createIndex(name: string, repoPath: string, outputPath?: string): Promise<boolean> {
		try {
			const defaultIndexPath = vscode.workspace.getConfiguration('beetle').get<string>('defaultIndexPath', '');
			const indexPath = outputPath || defaultIndexPath || path.join(os.homedir(), '.beetle', 'indexes');
			
			// Ensure the index directory exists
			if (!fs.existsSync(indexPath)) {
				fs.mkdirSync(indexPath, { recursive: true });
			}

			const result = await this.executeBeetleCommand([
				'create', name, 
				'-p', repoPath, 
				'-o', indexPath
			]);
			
			if (result.success) {
				vscode.window.showInformationMessage(`Index "${name}" created successfully!`);
				return true;
			} else {
				vscode.window.showErrorMessage(`Failed to create index: ${result.output}`);
				return false;
			}
		} catch (error) {
			vscode.window.showErrorMessage(`Index creation error: ${error}`);
			return false;
		}
	}

	async listIndexes(): Promise<BeetleIndex[]> {
		try {
			const result = await this.executeBeetleCommand(['list']);
			
			if (result.success) {
				try {
					return JSON.parse(result.output);
				} catch {
					// Parse as text if JSON fails
					const lines = result.output.split('\n').filter(line => line.trim());
					return lines.map(line => ({
						name: line.trim(),
						path: '',
						created_at: '',
						file_count: 0
					}));
				}
			} else {
				vscode.window.showErrorMessage(`Failed to list indexes: ${result.output}`);
				return [];
			}
		} catch (error) {
			vscode.window.showErrorMessage(`List indexes error: ${error}`);
			return [];
		}
	}

	dispose() {
		this.statusBarItem.dispose();
	}
}

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
	indexProvider = new IndexProvider();

	// Set context for when extension is enabled
	vscode.commands.executeCommand('setContext', 'beetle.enabled', true);

	// Register tree data providers
	vscode.window.createTreeView('beetleSearch', {
		treeDataProvider: searchResultProvider,
		showCollapseAll: true
	});

	vscode.window.createTreeView('beetleIndexes', {
		treeDataProvider: indexProvider,
		showCollapseAll: true
	});

	// Register commands
	const commands = [
		vscode.commands.registerCommand('beetle.search', async () => {
			const indexes = await beetleService.listIndexes();
			if (indexes.length === 0) {
				const create = await vscode.window.showInformationMessage(
					'No indexes found. Would you like to create one?',
					'Create Index'
				);
				if (create) {
					vscode.commands.executeCommand('beetle.createIndex');
				}
				return;
			}

			const selectedIndex = await vscode.window.showQuickPick(
				indexes.map(idx => idx.name),
				{ placeHolder: 'Select an index to search' }
			);

			if (!selectedIndex) {return;}

			const query = await vscode.window.showInputBox({
				placeHolder: 'Enter your search query',
				prompt: 'Search for code patterns, functions, or text'
			});

			if (!query) {return;}

			vscode.window.withProgress({
				location: vscode.ProgressLocation.Notification,
				title: 'Searching code...',
				cancellable: false
			}, async () => {
				const results = await beetleService.searchCode(selectedIndex, query);
				searchResultProvider.updateResults(query, results);
				
				if (results.length === 0) {
					vscode.window.showInformationMessage(`No results found for "${query}"`);
				} else {
					vscode.window.showInformationMessage(`Found ${results.length} results for "${query}"`);
				}
			});
		}),

		vscode.commands.registerCommand('beetle.createIndex', async () => {
			const name = await vscode.window.showInputBox({
				placeHolder: 'Enter index name',
				prompt: 'Choose a unique name for your index'
			});

			if (!name) {return;}

			// Default to current workspace if available
			const workspaceFolders = vscode.workspace.workspaceFolders;
			let defaultPath = '';
			if (workspaceFolders && workspaceFolders.length > 0) {
				defaultPath = workspaceFolders[0].uri.fsPath;
			}

			const repoPath = await vscode.window.showInputBox({
				placeHolder: 'Enter repository path',
				prompt: 'Path to the codebase you want to index',
				value: defaultPath
			});

			if (!repoPath) {return;}

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
				description: idx.file_count > 0 ? `${idx.file_count} files` : '',
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

				if (!selected) {return;}
				indexName = selected;
			}

			const confirm = await vscode.window.showWarningMessage(
				`Are you sure you want to delete the index "${indexName}"?`,
				{ modal: true },
				'Delete'
			);

			if (confirm === 'Delete') {
				// Note: The beetle CLI might not have a delete command yet
				vscode.window.showInformationMessage(`Index deletion not yet implemented in beetle CLI`);
			}
		}),

		vscode.commands.registerCommand('beetle.openSearchPanel', () => {
			vscode.commands.executeCommand('workbench.view.extension.beetle');
		}),

		vscode.commands.registerCommand('beetle.refreshIndexes', () => {
			indexProvider.refresh();
		})
	];

	// Add all commands to subscriptions
	commands.forEach(cmd => context.subscriptions.push(cmd));
	
	// Add service cleanup
	context.subscriptions.push(beetleService);

	// Auto-create index for current workspace if configured
	const autoCreate = vscode.workspace.getConfiguration('beetle').get<boolean>('autoCreateIndex', false);
	if (autoCreate && vscode.workspace.workspaceFolders) {
		const workspaceFolder = vscode.workspace.workspaceFolders[0];
		const workspaceName = path.basename(workspaceFolder.uri.fsPath);
		
		setTimeout(async () => {
			const indexes = await beetleService.listIndexes();
			const hasWorkspaceIndex = indexes.some(idx => idx.name === workspaceName);
			
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
