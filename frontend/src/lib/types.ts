export interface User {
  id: string;
  username: string;
  email: string;
  display_name: string | null;
  avatar_url: string | null;
  created_at: string;
}

export interface SnippetFile {
  id: string;
  snippet_id: string;
  version_id: string;
  filename: string;
  content: string;
  language: string | null;
  created_at: string;
}

export interface Version {
  id: string;
  snippet_id: string;
  version_number: number;
  commit_message: string | null;
  user_id: string;
  created_at: string;
}

export interface Comment {
  id: string;
  snippet_id: string;
  user: User;
  content: string;
  parent_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface Snippet {
  id: string;
  title: string;
  description: string | null;
  language: string;
  is_public: boolean;
  user: User;
  parent_id: string | null;
  likes_count: number;
  forks_count: number;
  views_count: number;
  files: SnippetFile[];
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface CreateSnippet {
  title: string;
  description?: string;
  language: string;
  is_public: boolean;
  files: CreateFile[];
  tags: string[];
}

export interface CreateFile {
  filename: string;
  content: string;
  language?: string;
}

export interface UpdateSnippet {
  title?: string;
  description?: string;
  language?: string;
  is_public?: boolean;
  files?: CreateFile[];
  tags?: string[];
  commit_message?: string;
}

export interface Tag {
  name: string;
  count: number;
}

export const LANGUAGES = [
  { value: 'javascript', label: 'JavaScript' },
  { value: 'typescript', label: 'TypeScript' },
  { value: 'python', label: 'Python' },
  { value: 'rust', label: 'Rust' },
  { value: 'go', label: 'Go' },
  { value: 'java', label: 'Java' },
  { value: 'csharp', label: 'C#' },
  { value: 'cpp', label: 'C++' },
  { value: 'php', label: 'PHP' },
  { value: 'ruby', label: 'Ruby' },
  { value: 'swift', label: 'Swift' },
  { value: 'kotlin', label: 'Kotlin' },
  { value: 'html', label: 'HTML' },
  { value: 'css', label: 'CSS' },
  { value: 'sql', label: 'SQL' },
  { value: 'json', label: 'JSON' },
  { value: 'yaml', label: 'YAML' },
  { value: 'markdown', label: 'Markdown' },
  { value: 'bash', label: 'Bash' },
  { value: 'text', label: 'Plain Text' }
];

export function getLanguageLabel(value: string): string {
  const lang = LANGUAGES.find(l => l.value === value);
  return lang ? lang.label : value;
}
