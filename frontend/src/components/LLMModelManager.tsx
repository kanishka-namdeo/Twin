'use client';

import { useState, useEffect, useRef } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { motion, AnimatePresence } from 'framer-motion';
import { toast } from 'sonner';
import { LLMModelInfo } from '@/types/llm';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Download, Trash2, FolderOpen, Check, Loader2, AlertCircle, Star, AlertTriangle } from 'lucide-react';

interface LLMModelManagerProps {
  selectedModel?: string;
  onModelSelect?: (modelName: string) => void;
  className?: string;
}

export function LLMModelManager({
  selectedModel,
  onModelSelect,
  className = '',
}: LLMModelManagerProps) {
  const [models, setModels] = useState<LLMModelInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [downloadingModels, setDownloadingModels] = useState<Set<string>>(new Set());
  const [modelsDirectory, setModelsDirectory] = useState<string>('');
  const [recommendedModel, setRecommendedModel] = useState<string | null>(null);

  // Refs for stable callbacks
  const onModelSelectRef = useRef(onModelSelect);

  // Update refs when props change
  useEffect(() => {
    onModelSelectRef.current = onModelSelect;
  }, [onModelSelect]);

  // Initialize models
  useEffect(() => {
    const initializeModels = async () => {
      try {
        setLoading(true);
        setError(null);

        // Fetch available models
        const modelList = await invoke<LLMModelInfo[]>('llm_get_available_models');
        setModels(modelList);

        // Get models directory
        const dir = await invoke<string>('llm_get_models_directory');
        setModelsDirectory(dir);

        // Get model recommendation
        try {
          const recommendation = await invoke<string | null>('llm_recommend_model');
          setRecommendedModel(recommendation);
          if (recommendation) {
            console.log(`[LLMModelManager] Recommended model: ${recommendation}`);
          }
        } catch (err) {
          console.warn('Failed to get model recommendation:', err);
          // Non-fatal error, continue without recommendation
        }
      } catch (err) {
        console.error('Failed to initialize LLM models:', err);
        setError(err instanceof Error ? err.message : 'Failed to load models');
        toast.error('Failed to load LLM models', {
          description: err instanceof Error ? err.message : 'Unknown error',
          duration: 5000,
        });
      } finally {
        setLoading(false);
      }
    };

    initializeModels();
  }, []);

  // Set up event listeners for download progress
  useEffect(() => {
    let unlistenProgress: (() => void) | null = null;
    let unlistenComplete: (() => void) | null = null;
    let unlistenError: (() => void) | null = null;

    const setupListeners = async () => {
      console.log('[LLMModelManager] Setting up event listeners...');

      // Download progress
      unlistenProgress = await listen<{ modelName: string; progress: number }>(
        'llm-model-download-progress',
        (event) => {
          const { modelName, progress } = event.payload;
          console.log(`[LLMModelManager] Progress update for ${modelName}: ${progress}%`);

          setModels((prevModels) =>
            prevModels.map((model) =>
              model.name === modelName
                ? { ...model, status: 'downloading', download_progress: progress }
                : model
            )
          );
        }
      );

      // Download complete
      unlistenComplete = await listen<{ modelName: string }>(
        'llm-model-download-complete',
        (event) => {
          const { modelName } = event.payload;
          const model = models.find((m) => m.name === modelName);
          const displayName = model?.display_name || modelName;

          setModels((prevModels) =>
            prevModels.map((model) =>
              model.name === modelName
                ? { ...model, status: 'available', download_progress: undefined }
                : model
            )
          );

          setDownloadingModels((prev) => {
            const newSet = new Set(prev);
            newSet.delete(modelName);
            return newSet;
          });

          toast.success(`${displayName} ready!`, {
            description: 'Model downloaded and ready to use',
            duration: 4000,
          });

          // Auto-select after download
          if (onModelSelectRef.current) {
            onModelSelectRef.current(modelName);
          }
        }
      );

      // Download error
      unlistenError = await listen<{ modelName: string; error: string }>(
        'llm-model-download-error',
        (event) => {
          const { modelName, error } = event.payload;
          const model = models.find((m) => m.name === modelName);
          const displayName = model?.display_name || modelName;

          setModels((prevModels) =>
            prevModels.map((model) =>
              model.name === modelName
                ? { ...model, status: 'error', download_progress: undefined }
                : model
            )
          );

          setDownloadingModels((prev) => {
            const newSet = new Set(prev);
            newSet.delete(modelName);
            return newSet;
          });

          toast.error(`Failed to download ${displayName}`, {
            description: error,
            duration: 6000,
            action: {
              label: 'Retry',
              onClick: () => downloadModel(modelName),
            },
          });
        }
      );
    };

    setupListeners();

    return () => {
      console.log('[LLMModelManager] Cleaning up event listeners...');
      if (unlistenProgress) unlistenProgress();
      if (unlistenComplete) unlistenComplete();
      if (unlistenError) unlistenError();
    };
  }, [models]);

  const downloadModel = async (modelName: string) => {
    if (downloadingModels.has(modelName)) return;

    const model = models.find((m) => m.name === modelName);
    const displayName = model?.display_name || modelName;

    try {
      setDownloadingModels((prev) => new Set([...prev, modelName]));

      setModels((prevModels) =>
        prevModels.map((model) =>
          model.name === modelName
            ? { ...model, status: 'downloading', download_progress: 0 }
            : model
        )
      );

      toast.info(`Downloading ${displayName}...`, {
        description: 'This may take a few minutes',
        duration: 5000,
      });

      await invoke('llm_download_model', { modelName });
    } catch (err) {
      console.error('Download failed:', err);
      setDownloadingModels((prev) => {
        const newSet = new Set(prev);
        newSet.delete(modelName);
        return newSet;
      });

      const errorMessage = err instanceof Error ? err.message : 'Download failed';
      setModels((prev) =>
        prev.map((model) =>
          model.name === modelName ? { ...model, status: 'error' } : model
        )
      );

      toast.error(`Failed to download ${displayName}`, {
        description: errorMessage,
        duration: 4000,
      });
    }
  };

  const deleteModel = async (modelName: string) => {
    const model = models.find((m) => m.name === modelName);
    const displayName = model?.display_name || modelName;

    try {
      await invoke('llm_delete_model', { modelName });

      // Refresh models list
      const modelList = await invoke<LLMModelInfo[]>('llm_get_available_models');
      setModels(modelList);

      toast.success(`${displayName} deleted`, {
        description: 'Model removed to free up space',
        duration: 3000,
      });

      // If deleted model was selected, clear selection
      if (selectedModel === modelName && onModelSelect) {
        onModelSelect('');
      }
    } catch (err) {
      console.error('Failed to delete model:', err);
      toast.error(`Failed to delete ${displayName}`, {
        description: err instanceof Error ? err.message : 'Delete failed',
        duration: 4000,
      });
    }
  };

  const selectModel = async (modelName: string) => {
    if (onModelSelect) {
      onModelSelect(modelName);
    }

    const model = models.find((m) => m.name === modelName);
    const displayName = model?.display_name || modelName;

    // Show warning if selected model is not the recommended one
    if (recommendedModel && modelName !== recommendedModel) {
      toast.warning(`${displayName} selected`, {
        description: 'This model may be too large for your system and could cause performance issues',
        duration: 5000,
      });
    } else {
      toast.success(`Switched to ${displayName}`, {
        duration: 3000,
      });
    }
  };

  const openModelsFolder = async () => {
    try {
      await invoke('open_llm_models_folder');
    } catch (err) {
      console.error('Failed to open models folder:', err);
      toast.error('Failed to open models folder');
    }
  };

  const formatFileSize = (sizeMb: number): string => {
    if (sizeMb >= 1024) {
      return `${(sizeMb / 1024).toFixed(1)} GB`;
    }
    return `${sizeMb} MB`;
  };

  if (loading) {
    return (
      <div className={`space-y-3 ${className}`}>
        <div className="animate-pulse space-y-3">
          <div className="h-24 bg-gray-100 rounded-lg"></div>
          <div className="h-24 bg-gray-100 rounded-lg"></div>
          <div className="h-24 bg-gray-100 rounded-lg"></div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={`bg-red-50 border border-red-200 rounded-lg p-4 ${className}`}>
        <p className="text-sm text-red-800">Failed to load models</p>
        <p className="text-xs text-red-600 mt-1">{error}</p>
      </div>
    );
  }

  return (
    <div className={`space-y-3 ${className}`}>
      {/* Header with Open Folder button */}
      <div className="flex items-center justify-between mb-4">
        <p className="text-sm text-gray-600">
          Download and manage local LLM models for summarization
        </p>
        {modelsDirectory && (
          <Button
            variant="outline"
            size="sm"
            onClick={openModelsFolder}
            className="flex items-center gap-2"
          >
            <FolderOpen className="h-4 w-4" />
            Open Models Folder
          </Button>
        )}
      </div>

      {/* Download Recommended Model button - shown when no models are downloaded */}
      {recommendedModel && !models.some(m => m.status === 'available') && (
        <motion.div
          initial={{ opacity: 0, y: -5 }}
          animate={{ opacity: 1, y: 0 }}
          className="mb-3"
        >
          <Card className="border-amber-200 bg-amber-50">
            <CardContent className="p-4">
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                  <div className="bg-amber-100 p-2 rounded-full">
                    <Star className="h-5 w-5 text-amber-600" />
                  </div>
                  <div>
                    <p className="text-sm font-semibold text-amber-900">
                      Recommended for your system
                    </p>
                    <p className="text-xs text-amber-700">
                      {models.find(m => m.name === recommendedModel)?.display_name || recommendedModel}
                      {' — '}
                      {formatFileSize(models.find(m => m.name === recommendedModel)?.size_mb || 0)}
                    </p>
                  </div>
                </div>
                <Button
                  onClick={() => downloadModel(recommendedModel)}
                  disabled={downloadingModels.has(recommendedModel)}
                  className="bg-amber-600 hover:bg-amber-700 text-white"
                >
                  {downloadingModels.has(recommendedModel) ? (
                    <>
                      <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                      Downloading...
                    </>
                  ) : (
                    <>
                      <Download className="h-4 w-4 mr-2" />
                      Download Recommended Model
                    </>
                  )}
                </Button>
              </div>
            </CardContent>
          </Card>
        </motion.div>
      )}

      {/* Models List */}
      <div className="space-y-3">
        {models.map((model) => {
          const isSelected = selectedModel === model.name;
          const isAvailable = model.status === 'available';
          const isDownloading = model.status === 'downloading';
          const isError = model.status === 'error';
          const downloadProgress = model.download_progress || 0;

          return (
            <motion.div
              key={model.name}
              initial={{ opacity: 0, y: 5 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.2 }}
            >
              <Card
                className={`
                  transition-all cursor-pointer
                  ${isSelected && isAvailable
                    ? 'border-blue-500 bg-blue-50'
                    : isAvailable
                    ? 'border-gray-200 hover:border-gray-300 bg-white'
                    : 'border-gray-200 bg-gray-50'
                  }
                  ${!isAvailable && !isDownloading ? 'cursor-default' : ''}
                `}
                onClick={() => {
                  if (isAvailable) selectModel(model.name);
                }}
              >
                <CardHeader className="pb-3">
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center gap-2 mb-1">
                        <CardTitle className="text-lg">{model.display_name}</CardTitle>
                        {isSelected && isAvailable && (
                          <motion.div
                            initial={{ scale: 0 }}
                            animate={{ scale: 1 }}
                            className="bg-blue-600 text-white px-2 py-0.5 rounded-full text-xs font-medium flex items-center gap-1"
                          >
                            <Check className="h-3 w-3" />
                            Active
                          </motion.div>
                        )}
                        {model.name === recommendedModel && (
                          <motion.div
                            initial={{ scale: 0 }}
                            animate={{ scale: 1 }}
                            className="bg-amber-500 text-white px-2 py-0.5 rounded-full text-xs font-medium flex items-center gap-1"
                          >
                            <Star className="h-3 w-3" />
                            Recommended
                          </motion.div>
                        )}
                      </div>
                      <p className="text-sm text-gray-600">{model.description}</p>
                      {model.name === recommendedModel && (
                        <p className="text-xs text-amber-600 font-medium mt-1">
                          Recommended for your system
                        </p>
                      )}
                      {recommendedModel && model.name !== recommendedModel && isSelected && isAvailable && (
                        <div className="flex items-center gap-1.5 mt-1.5 text-amber-600">
                          <AlertTriangle className="h-3.5 w-3.5" />
                          <span className="text-xs">This model may be too large for your system and could cause performance issues</span>
                        </div>
                      )}
                      <div className="flex items-center gap-3 mt-2 text-xs text-gray-500">
                        <span>📦 {formatFileSize(model.size_mb)}</span>
                        <span>📝 {model.context_length.toLocaleString()} context</span>
                      </div>
                    </div>

                    {/* Action Button */}
                    <div className="ml-4">
                      {isAvailable && (
                        <div className="flex items-center gap-2">
                          <div className="flex items-center gap-1.5 text-green-600">
                            <div className="w-2 h-2 bg-green-500 rounded-full"></div>
                            <span className="text-xs font-medium">Ready</span>
                          </div>
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={(e) => {
                              e.stopPropagation();
                              deleteModel(model.name);
                            }}
                            className="text-gray-400 hover:text-red-600"
                            title="Delete model to free up space"
                          >
                            <Trash2 className="h-4 w-4" />
                          </Button>
                        </div>
                      )}

                      {model.status === 'missing' && (
                        <Button
                          onClick={(e) => {
                            e.stopPropagation();
                            downloadModel(model.name);
                          }}
                          className="bg-blue-600 hover:bg-blue-700 text-white"
                        >
                          <Download className="h-4 w-4 mr-2" />
                          Download
                        </Button>
                      )}

                      {isError && (
                        <Button
                          onClick={(e) => {
                            e.stopPropagation();
                            downloadModel(model.name);
                          }}
                          className="bg-red-600 hover:bg-red-700 text-white"
                        >
                          <AlertCircle className="h-4 w-4 mr-2" />
                          Retry
                        </Button>
                      )}
                    </div>
                  </div>
                </CardHeader>

                {/* Download Progress */}
                <AnimatePresence>
                  {isDownloading && (
                    <motion.div
                      initial={{ opacity: 0, height: 0 }}
                      animate={{ opacity: 1, height: 'auto' }}
                      exit={{ opacity: 0, height: 0 }}
                      className="px-6 pb-4"
                    >
                      <CardContent className="p-0">
                        <div className="flex items-center justify-between mb-2">
                          <div className="flex items-center gap-2">
                            <Loader2 className="h-4 w-4 animate-spin text-blue-600" />
                            <span className="text-sm font-medium text-blue-600">
                              Downloading...
                            </span>
                          </div>
                          <span className="text-sm font-semibold text-blue-600">
                            {Math.round(downloadProgress)}%
                          </span>
                        </div>
                        <Progress value={downloadProgress} className="h-2" />
                        <p className="text-xs text-gray-500 mt-1">
                          {formatFileSize((model.size_mb * downloadProgress) / 100)} /{' '}
                          {formatFileSize(model.size_mb)}
                        </p>
                      </CardContent>
                    </motion.div>
                  )}
                </AnimatePresence>
              </Card>
            </motion.div>
          );
        })}
      </div>

      {/* Helper text */}
      {selectedModel && (
        <motion.div
          initial={{ opacity: 0, y: -5 }}
          animate={{ opacity: 1, y: 0 }}
          className="text-xs text-gray-500 text-center pt-2"
        >
          Using {models.find((m) => m.name === selectedModel)?.display_name || selectedModel} for
          summarization
        </motion.div>
      )}
    </div>
  );
}
