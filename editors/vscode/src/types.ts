// Types and interfaces for Beetle VS Code extension

export interface SearchResult {
	path: string;
	score: number;
	snippet: string;
	file_path?: string;
	line_number?: number;
	content?: string;
	relevance_score?: number;
}

export interface BeetleIndex {
	name: string;
	path?: string;
	created_at?: string;
	file_count?: number;
}

export interface BeetleCommandResult {
	success: boolean;
	output: string;
}
