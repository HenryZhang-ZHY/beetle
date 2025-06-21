import * as vscode from 'vscode';
import * as path from 'path';
import { SearchResult, BeetleIndex } from './types';
import { BeetleService } from './beetleService';

export class SearchResultProvider implements vscode.TreeDataProvider<SearchResultItem> {
	private _onDidChangeTreeData: vscode.EventEmitter<SearchResultItem | undefined | void> = new vscode.EventEmitter<SearchResultItem | undefined | void>();
	readonly onDidChangeTreeData: vscode.Event<SearchResultItem | undefined | void> = this._onDidChangeTreeData.event;

	private results: SearchResult[] = [];
	private query: string = '';

	constructor() { }

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

export class SearchResultItem extends vscode.TreeItem {
	constructor(
		public readonly result: SearchResult,
		public readonly collapsibleState: vscode.TreeItemCollapsibleState
	) {
		super(path.basename(result.path), collapsibleState);

		this.tooltip = `${result.path}\nScore: ${result.score}`;
		this.description = result.snippet.trim().substring(0, 50) + '...';
		this.resourceUri = vscode.Uri.file(result.path);

		this.command = {
			command: 'vscode.open',
			title: 'Open File',
			arguments: [
				vscode.Uri.file(result.path),
				{
					selection: new vscode.Range(
						1, 0,
						1, 100
					)
				}
			]
		};

		this.iconPath = new vscode.ThemeIcon('file-code');
	}
}

export class IndexProvider implements vscode.TreeDataProvider<IndexItem> {
	private _onDidChangeTreeData: vscode.EventEmitter<IndexItem | undefined | void> = new vscode.EventEmitter<IndexItem | undefined | void>();
	readonly onDidChangeTreeData: vscode.Event<IndexItem | undefined | void> = this._onDidChangeTreeData.event;

	private indexes: BeetleIndex[] = [];
	private beetleService: BeetleService;

	constructor(beetleService: BeetleService) {
		this.beetleService = beetleService;
		this.refresh();
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
			const result = await this.beetleService.listIndexes();
			if (result.length > 0) {
				this.indexes = result.map(index => ({
					name: index.name,
					path: index.path || '',
					created_at: index.created_at || new Date().toISOString(),
					file_count: index.file_count || 0
				}));
			} else {
				this.indexes = [];
				vscode.window.showInformationMessage('No indexes found. Create one to start using Beetle.');
			}
		} catch (error) {
			console.error('Failed to load indexes:', error);
			this.indexes = [];
		}
	}
}

export class IndexItem extends vscode.TreeItem {
	constructor(
		public readonly index: BeetleIndex,
		public readonly collapsibleState: vscode.TreeItemCollapsibleState
	) {
		super(index.name, collapsibleState);

		this.tooltip = `Path: ${index.path}\nCreated: ${index.created_at}\nFiles: ${index.file_count}`;
		this.description = (index.file_count ?? 0) > 0 ? `${index.file_count} files` : '';
		this.contextValue = 'index';
		this.iconPath = new vscode.ThemeIcon('database');
	}
}
