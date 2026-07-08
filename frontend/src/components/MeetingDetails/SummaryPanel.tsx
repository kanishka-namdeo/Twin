"use client";

import { Summary, SummaryResponse, Transcript } from '@/types';
import { EditableTitle } from '@/components/EditableTitle';
import { BlockNoteSummaryView, BlockNoteSummaryViewRef } from '@/components/AISummary/BlockNoteSummaryView';
import { EmptyStateSummary } from '@/components/EmptyStateSummary';
import { ModelConfig } from '@/components/ModelSettingsModal';
import { SummaryGeneratorButtonGroup } from './SummaryGeneratorButtonGroup';
import { SummaryUpdaterButtonGroup } from './SummaryUpdaterButtonGroup';
import { useEffect, useRef, useState, RefObject } from 'react';
import { toast } from 'sonner';
import { Languages, ChevronDown, Settings2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Popover, PopoverTrigger, PopoverContent } from '@/components/ui/popover';
import { LanguagePickerPopover } from '@/components/LanguagePickerPopover';
import { useRecentLanguages } from '@/hooks/useRecentLanguages';
import { labelForCode } from '@/lib/summary-languages';
import {
  readMeetingSummaryLanguage,
  saveMeetingSummaryLanguage,
  SummaryLanguageStorage,
} from '@/lib/summary-language-preferences';
import { Slider } from '@/components/ui/slider';
import { listen } from '@tauri-apps/api/event';

export interface GenerationParams {
  temperature: number;
  topP: number;
  topK: number;
  maxTokens: number;
}

const DEFAULT_GENERATION_PARAMS: GenerationParams = {
  temperature: 0.7,
  topP: 0.9,
  topK: 40,
  maxTokens: 2048,
};

const STORAGE_KEY = 'meetily-generation-params';

function loadGenerationParams(): GenerationParams {
  if (typeof window === 'undefined') return DEFAULT_GENERATION_PARAMS;
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return { ...DEFAULT_GENERATION_PARAMS, ...parsed };
    }
  } catch (e) {
    console.warn('Failed to load generation params from localStorage:', e);
  }
  return DEFAULT_GENERATION_PARAMS;
}

function saveGenerationParams(params: GenerationParams): void {
  if (typeof window === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(params));
  } catch (e) {
    console.warn('Failed to save generation params to localStorage:', e);
  }
}

interface SummaryPanelProps {
  meeting: {
    id: string;
    title: string;
    created_at: string;
  };
  meetingTitle: string;
  onTitleChange: (title: string) => void;
  isEditingTitle: boolean;
  onStartEditTitle: () => void;
  onFinishEditTitle: () => void;
  isTitleDirty: boolean;
  summaryRef: RefObject<BlockNoteSummaryViewRef | null>;
  isSaving: boolean;
  onSaveAll: () => Promise<void>;
  onCopySummary: () => Promise<void>;
  aiSummary: Summary | null;
  summaryStatus: 'idle' | 'processing' | 'summarizing' | 'regenerating' | 'completed' | 'error';
  transcripts: Transcript[];
  modelConfig: ModelConfig;
  setModelConfig: (config: ModelConfig | ((prev: ModelConfig) => ModelConfig)) => void;
  onSaveModelConfig: (config?: ModelConfig) => Promise<void>;
  onGenerateSummary: (customPrompt: string, generationParams?: GenerationParams) => Promise<void>;
  onStopGeneration: () => void;
  customPrompt: string;
  summaryResponse: SummaryResponse | null;
  onSaveSummary: (summary: Summary | { markdown?: string; summary_json?: any[] }) => Promise<void>;
  onSummaryChange: (summary: Summary) => void;
  onDirtyChange: (isDirty: boolean) => void;
  summaryError: string | null;
  onRegenerateSummary: (generationParams?: GenerationParams) => Promise<void>;
  getSummaryStatusMessage: (status: 'idle' | 'processing' | 'summarizing' | 'regenerating' | 'completed' | 'error') => string;
  availableTemplates: Array<{ id: string, name: string, description: string }>;
  selectedTemplate: string;
  onTemplateSelect: (templateId: string, templateName: string) => void;
  isModelConfigLoading?: boolean;
  onOpenModelSettings?: (openFn: () => void) => void;
}

export function SummaryPanel({
  meeting,
  meetingTitle,
  onTitleChange,
  isEditingTitle,
  onStartEditTitle,
  onFinishEditTitle,
  isTitleDirty,
  summaryRef,
  isSaving,
  onSaveAll,
  onCopySummary,
  aiSummary,
  summaryStatus,
  transcripts,
  modelConfig,
  setModelConfig,
  onSaveModelConfig,
  onGenerateSummary,
  onStopGeneration,
  customPrompt,
  summaryResponse,
  onSaveSummary,
  onSummaryChange,
  onDirtyChange,
  summaryError,
  onRegenerateSummary,
  getSummaryStatusMessage,
  availableTemplates,
  selectedTemplate,
  onTemplateSelect,
  isModelConfigLoading = false,
  onOpenModelSettings
}: SummaryPanelProps) {
  const [summaryLang, setSummaryLang] = useState<string | null>(null);
  const [summaryLangStorage, setSummaryLangStorage] = useState<SummaryLanguageStorage>('metadata');
  const [langPickerOpen, setLangPickerOpen] = useState(false);
  const languageLoadVersionRef = useRef(0);
  const activeMeetingIdRef = useRef(meeting.id);
  const languageSaveVersionRef = useRef(0);
  const languageSaveLoopRunningRef = useRef(false);
  const latestLanguageSaveRequestRef = useRef<{
    version: number;
    meetingId: string;
    language: string | null;
    rollback: {
      language: string | null;
      storage: SummaryLanguageStorage;
    };
  } | null>(null);
  activeMeetingIdRef.current = meeting.id;
  const { addRecent } = useRecentLanguages();

  const [generationParams, setGenerationParams] = useState<GenerationParams>(loadGenerationParams);
  const [showAdvancedOptions, setShowAdvancedOptions] = useState(false);

  const isLocalLLM = modelConfig.provider === 'local-llm';

  // Streaming state
  const [streamingTokens, setStreamingTokens] = useState<string>('');
  const [isStreaming, setIsStreaming] = useState(false);
  const streamingBufferRef = useRef<string>('');

  // Inference progress metrics
  const [inferenceProgress, setInferenceProgress] = useState<{
    tokens_generated: number;
    tokens_per_second: number;
    estimated_remaining_seconds: number | null;
  } | null>(null);
  const [inferenceComplete, setInferenceComplete] = useState<{
    tokens_generated: number;
    tokens_per_second: number;
    total_duration_seconds: number;
  } | null>(null);

  // Quality score from summary result
  const [qualityScore, setQualityScore] = useState<number | null>(null);

  // Model capability warning
  const [modelCapabilityWarning, setModelCapabilityWarning] = useState<{
    transcript_tokens: number;
    model_context_window: number;
    model_name: string;
    warning: string;
  } | null>(null);

  // Listen for streaming tokens from backend
  useEffect(() => {
    if (!isLocalLLM) return;

    const unlisten = listen<{ token: string; meeting_id: string }>(
      'summary-token-stream',
      (event) => {
        // Only process tokens for the current meeting
        if (event.payload.meeting_id === meeting.id) {
          streamingBufferRef.current += event.payload.token;
          setStreamingTokens(streamingBufferRef.current);
          setIsStreaming(true);
        }
      }
    );

    return () => {
      unlisten.then((unlistenFn) => unlistenFn());
    };
  }, [isLocalLLM, meeting.id]);

  // Listen for inference progress metrics
  useEffect(() => {
    if (!isLocalLLM) return;

    const unlistenProgress = listen<{
      tokens_generated: number;
      tokens_per_second: number;
      estimated_remaining_seconds: number | null;
      meeting_id: string;
    }>('llm-inference-progress', (event) => {
      if (event.payload.meeting_id === meeting.id) {
        setInferenceProgress({
          tokens_generated: event.payload.tokens_generated,
          tokens_per_second: event.payload.tokens_per_second,
          estimated_remaining_seconds: event.payload.estimated_remaining_seconds,
        });
      }
    });

    const unlistenComplete = listen<{
      tokens_generated: number;
      tokens_per_second: number;
      total_duration_seconds: number;
      meeting_id: string;
    }>('llm-inference-complete', (event) => {
      if (event.payload.meeting_id === meeting.id) {
        setInferenceComplete({
          tokens_generated: event.payload.tokens_generated,
          tokens_per_second: event.payload.tokens_per_second,
          total_duration_seconds: event.payload.total_duration_seconds,
        });
        setInferenceProgress(null);
      }
    });

    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
    };
  }, [isLocalLLM, meeting.id]);

  // Listen for model capability warnings (transcript too long for model)
  useEffect(() => {
    const unlistenWarning = listen<{
      meeting_id: string;
      transcript_tokens: number;
      model_context_window: number;
      model_name: string;
      warning: string;
    }>('model-capability-warning', (event) => {
      if (event.payload.meeting_id === meeting.id) {
        setModelCapabilityWarning(event.payload);
        toast.warning('Model capacity warning', {
          description: event.payload.warning,
          duration: 8000,
        });
      }
    });

    return () => {
      unlistenWarning.then((fn) => fn());
    };
  }, [meeting.id]);

  // Extract quality score from summary response when it changes
  useEffect(() => {
    if (summaryResponse?.data && typeof summaryResponse.data === 'object') {
      const score = summaryResponse.data.quality_score;
      if (typeof score === 'number') {
        setQualityScore(score);
      } else {
        setQualityScore(null);
      }
    } else {
      setQualityScore(null);
    }
  }, [summaryResponse]);

  // Reset streaming state when summary generation completes
  useEffect(() => {
    if (summaryStatus === 'completed' || summaryStatus === 'error' || summaryStatus === 'idle') {
      setStreamingTokens('');
      setIsStreaming(false);
      streamingBufferRef.current = '';
      setInferenceProgress(null);
      // Keep inferenceComplete visible until next generation starts
    }
    if (summaryStatus === 'processing' || summaryStatus === 'summarizing' || summaryStatus === 'regenerating') {
      setInferenceComplete(null);
      setInferenceProgress(null);
    }
  }, [summaryStatus]);

  const handleGenerationParamChange = (key: keyof GenerationParams, value: number) => {
    const updated = { ...generationParams, [key]: value };
    setGenerationParams(updated);
    saveGenerationParams(updated);
  };

  const handleResetToDefaults = () => {
    setGenerationParams(DEFAULT_GENERATION_PARAMS);
    saveGenerationParams(DEFAULT_GENERATION_PARAMS);
  };

  // Wrap onGenerateSummary to inject generation params when LocalLLM is selected
  const handleGenerateSummaryWithParams = (customPrompt: string) => {
    if (isLocalLLM) {
      return onGenerateSummary(customPrompt, generationParams);
    }
    return onGenerateSummary(customPrompt);
  };

  // Wrap onRegenerateSummary to inject generation params when LocalLLM is selected
  const handleRegenerateSummaryWithParams = () => {
    if (isLocalLLM) {
      return onRegenerateSummary(generationParams);
    }
    return onRegenerateSummary();
  };

  const effectiveLangLabel = summaryLang ? labelForCode(summaryLang) : 'Auto';
  const isLocalFallbackLanguage = summaryLangStorage === 'local_fallback';
  const autoSubtitle = isLocalFallbackLanguage
    ? 'Saved on this device for folderless meetings'
    : 'Uses dominant transcript language';

  useEffect(() => {
    let cancelled = false;
    const loadVersion = languageLoadVersionRef.current + 1;
    languageLoadVersionRef.current = loadVersion;

    const loadSummaryLanguage = async () => {
      try {
        const stored = await readMeetingSummaryLanguage(meeting.id);
        if (!cancelled && languageLoadVersionRef.current === loadVersion) {
          setSummaryLang(stored.language);
          setSummaryLangStorage(stored.storage);
        }
      } catch (err) {
        console.error('Failed to load summary language:', err);
        toast.warning('Could not load saved summary language', {
          description: 'Using Auto until meeting metadata can be read.',
        });
        if (!cancelled && languageLoadVersionRef.current === loadVersion) setSummaryLang(null);
      }
    };

    loadSummaryLanguage();

    return () => {
      cancelled = true;
    };
  }, [meeting.id]);

  const persistLatestLanguageSelection = async () => {
    if (languageSaveLoopRunningRef.current) return;
    languageSaveLoopRunningRef.current = true;

    try {
      while (true) {
        const request = latestLanguageSaveRequestRef.current;
        if (!request) return;

        try {
          const saved = await saveMeetingSummaryLanguage(request.meetingId, request.language);
          const latest = latestLanguageSaveRequestRef.current;
          if (
            latest?.version === request.version &&
            activeMeetingIdRef.current === request.meetingId
          ) {
            setSummaryLang(saved.language);
            setSummaryLangStorage(saved.storage);
            if (saved.storage === 'local_fallback') {
              toast.info('Summary language saved on this device', {
                description: 'This meeting has no recording folder, so the preference cannot be written to meeting metadata.',
              });
            }
            if (request.language) {
              addRecent(request.language);
            }
            return;
          }

          if (latest?.version === request.version) return;
        } catch (err) {
          const latest = latestLanguageSaveRequestRef.current;
          if (
            latest?.version === request.version &&
            activeMeetingIdRef.current === request.meetingId
          ) {
            console.error('Failed to persist summary language:', err);
            toast.error('Failed to save summary language');
            setSummaryLang(request.rollback.language);
            setSummaryLangStorage(request.rollback.storage);
            return;
          }

          console.warn('Ignoring failed stale summary language save:', err);
          if (latest?.version === request.version) return;
        }
      }
    } finally {
      languageSaveLoopRunningRef.current = false;
    }
  };

  const handleLangChange = (code: string | null) => {
    const previous = summaryLang;
    const previousStorage = summaryLangStorage;
    const nextStored = code;
    languageLoadVersionRef.current += 1;
    latestLanguageSaveRequestRef.current = {
      version: languageSaveVersionRef.current + 1,
      meetingId: meeting.id,
      language: nextStored,
      rollback: {
        language: previous,
        storage: previousStorage,
      },
    };
    languageSaveVersionRef.current += 1;
    setSummaryLang(nextStored);
    setLangPickerOpen(false);
    void persistLatestLanguageSelection();
  };

  const isSummaryLoading = summaryStatus === 'processing' || summaryStatus === 'summarizing' || summaryStatus === 'regenerating';

  const languageSlot = (
    <Popover open={langPickerOpen} onOpenChange={setLangPickerOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          size="sm"
          title={`Summary language: ${effectiveLangLabel}${isLocalFallbackLanguage ? ' (saved on this device)' : ''}`}
          aria-label="Set summary language"
        >
          <Languages size={18} />
          <span className="hidden lg:inline">{effectiveLangLabel}</span>
          <ChevronDown size={14} className="text-gray-400" />
        </Button>
      </PopoverTrigger>
      <PopoverContent
        align="end"
        className="w-auto p-0 border-0 shadow-none bg-transparent"
      >
        <LanguagePickerPopover
          value={summaryLang}
          onChange={handleLangChange}
          onClose={() => setLangPickerOpen(false)}
          autoSubtitle={autoSubtitle}
        />
      </PopoverContent>
    </Popover>
  );

  const advancedOptionsSlot = isLocalLLM && (
    <Popover open={showAdvancedOptions} onOpenChange={setShowAdvancedOptions}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          size="sm"
          title="Advanced generation options"
          aria-label="Advanced options"
        >
          <Settings2 size={18} />
          <span className="hidden lg:inline">Options</span>
        </Button>
      </PopoverTrigger>
      <PopoverContent
        align="end"
        className="w-80 p-4"
      >
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-semibold">Generation Parameters</h3>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleResetToDefaults}
              className="text-xs h-7"
            >
              Reset
            </Button>
          </div>

          <div className="space-y-3">
            <div>
              <label className="text-xs font-medium text-gray-700 flex justify-between mb-1">
                <span>Temperature</span>
                <span className="text-gray-500">{generationParams.temperature.toFixed(1)}</span>
              </label>
              <Slider
                value={[generationParams.temperature]}
                onValueChange={(val) => handleGenerationParamChange('temperature', val[0])}
                min={0.1}
                max={1.5}
                step={0.1}
                className="w-full"
              />
              <p className="text-xs text-gray-500 mt-1">Controls randomness (0.1 = focused, 1.5 = creative)</p>
            </div>

            <div>
              <label className="text-xs font-medium text-gray-700 flex justify-between mb-1">
                <span>Top P</span>
                <span className="text-gray-500">{generationParams.topP.toFixed(2)}</span>
              </label>
              <Slider
                value={[generationParams.topP]}
                onValueChange={(val) => handleGenerationParamChange('topP', val[0])}
                min={0.1}
                max={1.0}
                step={0.05}
                className="w-full"
              />
              <p className="text-xs text-gray-500 mt-1">Nucleus sampling threshold</p>
            </div>

            <div>
              <label className="text-xs font-medium text-gray-700 flex justify-between mb-1">
                <span>Top K</span>
                <span className="text-gray-500">{generationParams.topK}</span>
              </label>
              <Slider
                value={[generationParams.topK]}
                onValueChange={(val) => handleGenerationParamChange('topK', val[0])}
                min={1}
                max={100}
                step={1}
                className="w-full"
              />
              <p className="text-xs text-gray-500 mt-1">Limits token selection pool</p>
            </div>

            <div>
              <label className="text-xs font-medium text-gray-700 flex justify-between mb-1">
                <span>Max Tokens</span>
                <span className="text-gray-500">{generationParams.maxTokens}</span>
              </label>
              <input
                type="number"
                value={generationParams.maxTokens}
                onChange={(e) => {
                  const val = parseInt(e.target.value);
                  if (!isNaN(val) && val >= 256 && val <= 8192) {
                    handleGenerationParamChange('maxTokens', val);
                  }
                }}
                min={256}
                max={8192}
                step={256}
                className="w-full px-2 py-1 text-sm border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
              <p className="text-xs text-gray-500 mt-1">Maximum output length (256-8192)</p>
            </div>
          </div>
        </div>
      </PopoverContent>
    </Popover>
  );

  return (
    <div className="flex-1 min-w-0 flex flex-col bg-white">
      {/* Title area */}
      <div className="p-4 border-b border-gray-200">
        {/* <EditableTitle
          title={meetingTitle}
          isEditing={isEditingTitle}
          onStartEditing={onStartEditTitle}
          onFinishEditing={onFinishEditTitle}
          onChange={onTitleChange}
        /> */}

        {/* Button groups - only show when summary exists */}
        {aiSummary && !isSummaryLoading && (
          <div className="flex items-center justify-center w-full pt-0 gap-2">
            {/* Left-aligned: Summary Generator Button Group */}
            <div className="flex-shrink-0">
              <SummaryGeneratorButtonGroup
                modelConfig={modelConfig}
                setModelConfig={setModelConfig}
                onSaveModelConfig={onSaveModelConfig}
                onGenerateSummary={handleGenerateSummaryWithParams}
                onRegenerateSummary={handleRegenerateSummaryWithParams}
                onStopGeneration={onStopGeneration}
                customPrompt={customPrompt}
                summaryStatus={summaryStatus}
                availableTemplates={availableTemplates}
                selectedTemplate={selectedTemplate}
                onTemplateSelect={onTemplateSelect}
                hasTranscripts={transcripts.length > 0}
                hasSummary={!!aiSummary}
                isModelConfigLoading={isModelConfigLoading}
                onOpenModelSettings={onOpenModelSettings}
                languageSlot={languageSlot}
                advancedOptionsSlot={advancedOptionsSlot}
              />
            </div>

            {/* Right-aligned: Summary Updater Button Group */}
            <div className="flex-shrink-0">
              <SummaryUpdaterButtonGroup
                isSaving={isSaving}
                isDirty={isTitleDirty || (summaryRef.current?.isDirty || false)}
                onSave={onSaveAll}
                onCopy={onCopySummary}
                hasSummary={!!aiSummary}
              />
            </div>
          </div>
        )}
      </div>

      {isSummaryLoading ? (
        <div className="flex flex-col h-full">
          {/* Show button group during generation */}
          <div className="flex items-center justify-center pt-8 pb-4">
            <SummaryGeneratorButtonGroup
              modelConfig={modelConfig}
              setModelConfig={setModelConfig}
              onSaveModelConfig={onSaveModelConfig}
              onGenerateSummary={handleGenerateSummaryWithParams}
              onStopGeneration={onStopGeneration}
              customPrompt={customPrompt}
              summaryStatus={summaryStatus}
              availableTemplates={availableTemplates}
              selectedTemplate={selectedTemplate}
              onTemplateSelect={onTemplateSelect}
              hasTranscripts={transcripts.length > 0}
              isModelConfigLoading={isModelConfigLoading}
              onOpenModelSettings={onOpenModelSettings}
              advancedOptionsSlot={advancedOptionsSlot}
            />
          </div>
          {/* Streaming output or loading spinner */}
          {isStreaming && streamingTokens ? (
            <div className="flex-1 overflow-y-auto px-6 pb-6">
              <div className="max-w-4xl mx-auto">
                {isLocalLLM && inferenceProgress && (
                  <div className="mb-3 flex items-center gap-3 text-xs text-gray-500 bg-gray-50 rounded-lg px-3 py-2">
                    <span className="font-medium text-blue-600">{inferenceProgress.tokens_per_second} tok/s</span>
                    <span>•</span>
                    <span>{inferenceProgress.tokens_generated} tokens</span>
                    {inferenceProgress.estimated_remaining_seconds !== null && (
                      <>
                        <span>•</span>
                        <span>~{Math.round(inferenceProgress.estimated_remaining_seconds)}s remaining</span>
                      </>
                    )}
                  </div>
                )}
                <div className="prose prose-sm max-w-none">
                  <div className="whitespace-pre-wrap text-sm leading-relaxed">
                    {streamingTokens}
                    <span className="inline-block w-2 h-4 ml-1 bg-blue-500 animate-pulse" />
                  </div>
                </div>
              </div>
            </div>
          ) : (
            <div className="flex items-center justify-center flex-1">
              <div className="text-center">
                <div className="inline-block animate-spin rounded-full h-12 w-12 border-t-2 border-b-2 border-blue-500 mb-4"></div>
                <p className="text-gray-600">Generating AI Summary...</p>
                {isLocalLLM && inferenceProgress && (
                  <div className="mt-3 text-xs text-gray-500">
                    <span className="font-medium text-blue-600">{inferenceProgress.tokens_per_second} tok/s</span>
                    <span className="mx-2">•</span>
                    <span>{inferenceProgress.tokens_generated} tokens</span>
                    {inferenceProgress.estimated_remaining_seconds !== null && (
                      <>
                        <span className="mx-2">•</span>
                        <span>~{Math.round(inferenceProgress.estimated_remaining_seconds)}s remaining</span>
                      </>
                    )}
                  </div>
                )}
              </div>
            </div>
          )}
        </div>
      ) : !aiSummary ? (
        <div className="flex flex-col h-full">
          {/* Centered Summary Generator Button Group when no summary */}
          <div className="flex items-center justify-center gap-2 pt-8 pb-4">
            <SummaryGeneratorButtonGroup
              modelConfig={modelConfig}
              setModelConfig={setModelConfig}
              onSaveModelConfig={onSaveModelConfig}
              onGenerateSummary={handleGenerateSummaryWithParams}
              onStopGeneration={onStopGeneration}
              customPrompt={customPrompt}
              summaryStatus={summaryStatus}
              availableTemplates={availableTemplates}
              selectedTemplate={selectedTemplate}
              onTemplateSelect={onTemplateSelect}
              hasTranscripts={transcripts.length > 0}
              hasSummary={false}
              isModelConfigLoading={isModelConfigLoading}
              onOpenModelSettings={onOpenModelSettings}
              languageSlot={transcripts.length > 0 ? languageSlot : undefined}
              advancedOptionsSlot={advancedOptionsSlot}
            />
          </div>
          {/* Empty state message */}
          <EmptyStateSummary
            onGenerate={() => handleGenerateSummaryWithParams(customPrompt)}
            hasModel={modelConfig.provider !== null && modelConfig.model !== null}
            isGenerating={isSummaryLoading}
          />
        </div>
      ) : transcripts?.length > 0 && (
        <div className="flex-1 overflow-y-auto min-h-0">
          {summaryResponse && (
            <div className="sticky bottom-0 left-0 right-0 bg-white shadow-lg p-4 max-h-[33vh] overflow-y-auto border-t border-gray-200">
              <h3 className="text-lg font-semibold mb-2">Meeting Summary</h3>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="bg-white p-4 rounded-lg shadow-sm">
                  <h4 className="font-medium mb-1">Key Points</h4>
                  <ul className="list-disc pl-4">
                    {summaryResponse.summary.key_points.blocks.map((block, i) => (
                      <li key={i} className="text-sm">{block.content}</li>
                    ))}
                  </ul>
                </div>
                <div className="bg-white p-4 rounded-lg shadow-sm">
                  <h4 className="font-medium mb-1">Action Items</h4>
                  <ul className="list-disc pl-4">
                    {summaryResponse.summary.action_items.blocks.map((block, i) => (
                      <li key={i} className="text-sm">{block.content}</li>
                    ))}
                  </ul>
                </div>
                <div className="bg-white p-4 rounded-lg shadow-sm">
                  <h4 className="font-medium mb-1">Decisions</h4>
                  <ul className="list-disc pl-4">
                    {summaryResponse.summary.decisions.blocks.map((block, i) => (
                      <li key={i} className="text-sm">{block.content}</li>
                    ))}
                  </ul>
                </div>
                <div className="bg-white p-4 rounded-lg shadow-sm">
                  <h4 className="font-medium mb-1">Main Topics</h4>
                  <ul className="list-disc pl-4">
                    {summaryResponse.summary.main_topics.blocks.map((block, i) => (
                      <li key={i} className="text-sm">{block.content}</li>
                    ))}
                  </ul>
                </div>
              </div>
              {summaryResponse.raw_summary ? (
                <div className="mt-4">
                  <h4 className="font-medium mb-1">Full Summary</h4>
                  <p className="text-sm whitespace-pre-wrap">{summaryResponse.raw_summary}</p>
                </div>
              ) : null}
            </div>
          )}
          <div className="p-6 w-full">
            <BlockNoteSummaryView
              ref={summaryRef}
              summaryData={aiSummary}
              onSave={onSaveSummary}
              onSummaryChange={onSummaryChange}
              onDirtyChange={onDirtyChange}
              status={summaryStatus}
              error={summaryError}
              onRegenerateSummary={handleRegenerateSummaryWithParams}
              meeting={{
                id: meeting.id,
                title: meetingTitle,
                created_at: meeting.created_at
              }}
            />
          </div>
          {summaryStatus !== 'idle' && (
            <div className={`mt-4 p-4 rounded-lg ${summaryStatus === 'error' ? 'bg-red-100 text-red-700' :
              summaryStatus === 'completed' ? 'bg-green-100 text-green-700' :
                'bg-blue-100 text-blue-700'
              }`}>
              <p className="text-sm font-medium">{getSummaryStatusMessage(summaryStatus)}</p>
              {summaryStatus === 'completed' && inferenceComplete && isLocalLLM && (
                <div className="mt-2 text-xs text-green-700 space-y-1">
                  <div className="flex items-center gap-3">
                    <span className="font-medium">{inferenceComplete.tokens_generated} tokens</span>
                    <span>•</span>
                    <span>{inferenceComplete.tokens_per_second} tok/s</span>
                    <span>•</span>
                    <span>{inferenceComplete.total_duration_seconds}s total</span>
                  </div>
                </div>
              )}
              {summaryStatus === 'completed' && qualityScore !== null && (
                <div className="mt-3 pt-3 border-t border-green-200">
                  <div className="flex items-center gap-2">
                    <span className="text-xs font-medium text-green-800">Summary Quality:</span>
                    <div className="flex items-center gap-1">
                      {[1, 2, 3, 4, 5].map((star) => (
                        <span
                          key={star}
                          className={`text-lg ${star <= qualityScore ? 'text-yellow-500' : 'text-gray-300'}`}
                        >
                          ★
                        </span>
                      ))}
                      <span className="text-xs font-semibold text-green-800 ml-1">
                        {qualityScore}/5
                      </span>
                    </div>
                  </div>
                  {qualityScore < 3 && (
                    <p className="mt-2 text-xs text-orange-700 bg-orange-50 p-2 rounded">
                      ⚠️ This summary may be incomplete or low quality. Consider regenerating with a larger model or cloud provider for better results.
                    </p>
                  )}
                </div>
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
