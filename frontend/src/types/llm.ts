export type LLMModelStatus =
  | 'available'
  | 'missing'
  | 'corrupted'
  | 'downloading'
  | 'error';

export interface LLMModelInfo {
  name: string;
  display_name: string;
  size_mb: number;
  context_length: number;
  description: string;
  status: LLMModelStatus;
  download_progress?: number;
}
