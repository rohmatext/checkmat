export type Task = {
	id: number;
	text: string;
	description?: string;
	done: boolean;
	deleted: boolean;
	pos: number;
	category_id: number;
	project_id: number;
};

export type Category = {
	id: number;
	name: string;
	color: string;
	pos?: number;
};

export type ProjectStatus = 'active' | 'completed' | 'archived';

export type Project = {
	id: number;
	name: string;
	description: string;
	status?: ProjectStatus;
	pos?: number;
};

export type Tab = 'active' | 'done' | 'deleted';
export type Page = 'home' | 'tasks' | 'categories' | 'about';
export type SortField = 'cat_pos' | 'pos' | 'category' | 'id' | 'text';
export type SortDir = 'asc' | 'desc';

export const PALETTE = ['red', 'orange', 'amber', 'green', 'cyan', 'blue', 'violet', 'pink'] as const;

export const TONE: Record<string, { badge: string; solid: string }> = {
	red: { badge: 'bg-red-50 text-red-700 border-red-200 dark:bg-red-950 dark:text-red-400 dark:border-red-900', solid: 'bg-red-500' },
	orange: { badge: 'bg-orange-50 text-orange-700 border-orange-200 dark:bg-orange-950 dark:text-orange-400 dark:border-orange-900', solid: 'bg-orange-500' },
	amber: { badge: 'bg-amber-50 text-amber-700 border-amber-200 dark:bg-amber-950 dark:text-amber-400 dark:border-amber-900', solid: 'bg-amber-500' },
	green: { badge: 'bg-green-50 text-green-700 border-green-200 dark:bg-green-950 dark:text-green-400 dark:border-green-900', solid: 'bg-green-500' },
	cyan: { badge: 'bg-cyan-50 text-cyan-700 border-cyan-200 dark:bg-cyan-950 dark:text-cyan-400 dark:border-cyan-900', solid: 'bg-cyan-500' },
	blue: { badge: 'bg-blue-50 text-blue-700 border-blue-200 dark:bg-blue-950 dark:text-blue-400 dark:border-blue-900', solid: 'bg-blue-500' },
	violet: { badge: 'bg-violet-50 text-violet-700 border-violet-200 dark:bg-violet-950 dark:text-violet-400 dark:border-violet-900', solid: 'bg-violet-500' },
	pink: { badge: 'bg-pink-50 text-pink-700 border-pink-200 dark:bg-pink-950 dark:text-pink-400 dark:border-pink-900', solid: 'bg-pink-500' }
};

export const CAT_KEY = 'taskmini.cat';
export const PROJECT_KEY = 'taskmini.project';
export const SORT_FIELD_KEY = 'taskmini.sortField';
export const SORT_DIR_KEY = 'taskmini.sortDir';
export const THEME_KEY = 'taskmini.theme';
export const EXPORT_DIR_KEY = 'taskmini.exportDirectory';

export function getBadgeCls(color?: string): string {
	return TONE[color ?? '']?.badge ?? 'bg-muted text-muted-foreground border-border';
}

export function getSolidCls(color?: string): string {
	return TONE[color ?? '']?.solid ?? 'bg-gray-400';
}
