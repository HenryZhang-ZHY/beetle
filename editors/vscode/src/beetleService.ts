import * as vscode from 'vscode';
import { spawn } from 'child_process';
import { SearchResult, BeetleIndex, BeetleCommandResult } from './types';

export class BeetleService {
	private statusBarItem: vscode.StatusBarItem;

	constructor() {
		this.statusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
		this.statusBarItem.text = "$(search) Beetle";
		this.statusBarItem.tooltip = "Beetle Code Search";
		this.statusBarItem.command = 'beetle.openSearchPanel';
		this.statusBarItem.show();
	}

	async executeBeetleCommand(args: string[]): Promise<BeetleCommandResult> {
		return new Promise((resolve) => {
			const process = spawn('beetle', args, { shell: true, });

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
			const result = await this.executeBeetleCommand(['search', '-i', indexName, '-q', query, '--format', 'json']);

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
					path: match[1],
					score: 1.0,
					snippet: match[3],
					file_path: match[1],
					line_number: parseInt(match[2]),
					content: match[3],
					relevance_score: 1.0
				});
			}
		}

		return results;
	}

	async createIndex(name: string, repoPath: string): Promise<boolean> {
		try {
			const result = await this.executeBeetleCommand([
				'new',
				'--index', name,
				'--path', repoPath
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
			const result = await this.executeBeetleCommand(['list']);			if (result.success) {
				return result.output.split('\n')
					.filter((line: string) => line.trim())
					.map((line: string) => ({ name: line.trim() }));

			} else {
				vscode.window.showErrorMessage(`Failed to list indexes: ${result.output}`);
				return [];
			}
		} catch (error) {
			vscode.window.showErrorMessage(`List indexes error: ${error}`);
			return [];
		}
	}

	async removeIndex(name: string): Promise<boolean> {
		try {
			const result = await this.executeBeetleCommand(['remove', '--index', name]);

			if (result.success) {
				vscode.window.showInformationMessage(`Index "${name}" removed successfully!`);
				return true;
			} else {
				vscode.window.showErrorMessage(`Failed to remove index: ${result.output}`);
				return false;
			}
		} catch (error) {
			vscode.window.showErrorMessage(`Delete index error: ${error}`);
			return false;
		}
	}

	dispose() {
		this.statusBarItem.dispose();
	}
}
