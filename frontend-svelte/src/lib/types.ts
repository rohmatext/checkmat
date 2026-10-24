export type TaskStatus = 'todo' | 'inprogress' | 'done';

export type Task = {
	id: number;
	text: string;
	description?: string;
	done: boolean;
	status?: TaskStatus;
	deleted: boolean;
	pos: number;
	category_id: number;
	project_id: number;
	created_at?: string;
	status_updated_at?: string;
	subtask_count?: number;
	subtask_done_count?: number;
	tags?: string;
};

export function getTaskTags(task: Task | { tags?: string; text?: string } | null | undefined): string[] {
	if (!task) return [];
	const set = new Set<string>();
	if (task.tags) {
		for (const t of task.tags.split(',')) {
			const clean = t.trim().replace(/^#+/, '').toLowerCase();
			if (clean) set.add(clean);
		}
	}
	return Array.from(set);
}

export function formatTagsString(tags: string[]): string {
	const set = new Set<string>();
	for (const t of tags) {
		const clean = t.trim().replace(/^#+/, '').toLowerCase();
		if (clean) set.add(clean);
	}
	return Array.from(set).join(',');
}

export type Subtask = {
	id: number;
	task_id: number;
	title: string;
	done: boolean;
	pos: number;
	created_at?: string;
};

export type TaskEvent = {
	id: number;
	task_id: number;
	status: TaskStatus;
	created_at: string;
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

export type Tab = 'todo' | 'active' | 'inprogress' | 'done' | 'deleted';
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

export const CAT_KEY = 'checkmat.cat';
export const PROJECT_KEY = 'checkmat.project';
export const SORT_FIELD_KEY = 'checkmat.sortField';
export const SORT_DIR_KEY = 'checkmat.sortDir';
export const THEME_KEY = 'checkmat.theme';
export const EXPORT_DIR_KEY = 'checkmat.exportDirectory';
export const AUTOLOCK_KEY = 'checkmat.autolock_delay';

export const AUTOLOCK_OPTIONS = [
	{ value: 'never', label: 'Tidak Pernah', desc: 'Aplikasi tidak akan terkunci otomatis saat diminimize' },
	{ value: '0', label: 'Langsung saat Minimize', desc: 'Langsung terkunci otomatis saat jendela diminimize' },
	{ value: '15', label: '15 Detik', desc: 'Terkunci jika diminimize lebih dari 15 detik' },
	{ value: '30', label: '30 Detik', desc: 'Terkunci jika diminimize lebih dari 30 detik' },
	{ value: '60', label: '1 Menit', desc: 'Terkunci jika diminimize lebih dari 1 menit (Disarankan)' },
	{ value: '120', label: '2 Menit', desc: 'Terkunci jika diminimize lebih dari 2 menit' },
	{ value: '300', label: '5 Menit', desc: 'Terkunci jika diminimize lebih dari 5 menit' },
	{ value: '600', label: '10 Menit', desc: 'Terkunci jika diminimize lebih dari 10 menit' },
	{ value: '900', label: '15 Menit', desc: 'Terkunci jika diminimize lebih dari 15 menit' }
] as const;

export function getBadgeCls(color?: string): string {
	return TONE[color ?? '']?.badge ?? 'bg-muted text-muted-foreground border-border';
}

export function getSolidCls(color?: string): string {
	return TONE[color ?? '']?.solid ?? 'bg-gray-400';
}

export function formatTaskDate(raw?: string): string {
	if (!raw) return '';
	let d: Date;
	const num = Number(raw);
	if (!isNaN(num) && num > 0) {
		d = new Date(num > 1e11 ? num : num * 1000);
	} else {
		d = new Date(raw);
	}
	if (isNaN(d.getTime())) return '';

	const now = new Date();
	const isToday =
		d.getDate() === now.getDate() &&
		d.getMonth() === now.getMonth() &&
		d.getFullYear() === now.getFullYear();

	if (isToday) {
		const h = String(d.getHours()).padStart(2, '0');
		const m = String(d.getMinutes()).padStart(2, '0');
		return `${h}:${m}`;
	}

	const yesterday = new Date(now);
	yesterday.setDate(now.getDate() - 1);
	const isYesterday =
		d.getDate() === yesterday.getDate() &&
		d.getMonth() === yesterday.getMonth() &&
		d.getFullYear() === yesterday.getFullYear();

	if (isYesterday) {
		return 'Kemarin';
	}

	const months = ['Jan', 'Feb', 'Mar', 'Apr', 'Mei', 'Jun', 'Jul', 'Agu', 'Sep', 'Okt', 'Nov', 'Des'];
	const day = d.getDate();
	const month = months[d.getMonth()];

	if (d.getFullYear() === now.getFullYear()) {
		return `${day} ${month}`;
	}
	const yearShort = String(d.getFullYear()).slice(-2);
	return `${day} ${month} '${yearShort}`;
}

export function formatFullDate(raw?: string): string {
	if (!raw) return '';
	let d: Date;
	const num = Number(raw);
	if (!isNaN(num) && num > 0) {
		d = new Date(num > 1e11 ? num : num * 1000);
	} else {
		d = new Date(raw);
	}
	if (isNaN(d.getTime())) return '';
	return d.toLocaleString('id-ID', {
		day: 'numeric',
		month: 'short',
		year: 'numeric',
		hour: '2-digit',
		minute: '2-digit'
	});
}

export function getTaskEventTooltip(status?: TaskStatus, dateStr?: string, createdStr?: string): string {
	const eventTime = dateStr || createdStr;
	const fullDate = formatFullDate(eventTime);
	if (!fullDate) return '';
	if (status === 'inprogress') {
		return `Mulai In Progress: ${fullDate}`;
	} else if (status === 'done') {
		return `Selesai: ${fullDate}`;
	} else {
		const isSame = !dateStr || dateStr === createdStr;
		return isSame ? `Dibuat: ${fullDate}` : `Kembali ke Todo: ${fullDate}`;
	}
}
