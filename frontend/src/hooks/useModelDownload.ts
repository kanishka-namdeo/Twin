import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const PARAKEET_MODEL = 'parakeet-tdt-0.6b-v3-int8';

interface UseModelDownloadReturn {
  isParakeetReady: boolean;
  isSummaryReady: boolean;
  isDownloading: boolean;
  downloadProgress: number;
  startDownload: () => Promise<void>;
  checkModelStatus: () => Promise<void>;
}

/**
 * Hook for managing model downloads from anywhere in the app.
 * Used by empty-state nudge, recording-blocked modal, and sidebar indicator.
 */
export function useModelDownload(): UseModelDownloadReturn {
  const [isParakeetReady, setIsParakeetReady] = useState(false);
  const [isSummaryReady, setIsSummaryReady] = useState(false);
  const [isDownloading, setIsDownloading] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState(0);

  const checkModelStatus = useCallback(async () => {
    try {
      // Check Parakeet
      await invoke('parakeet_init');
      const parakeetReady = await invoke<boolean>('parakeet_has_available_models');
      setIsParakeetReady(parakeetReady);

      // Check summary model
      const recommendedModel = await invoke<string>('builtin_ai_get_recommended_model');
      if (recommendedModel) {
        const summaryReady = await invoke<boolean>('builtin_ai_is_model_ready', {
          modelName: recommendedModel,
          refresh: true,
        });
        setIsSummaryReady(summaryReady);
      }
    } catch (error) {
      console.error('[useModelDownload] Failed to check model status:', error);
    }
  }, []);

  const startDownload = useCallback(async () => {
    try {
      console.log('[useModelDownload] Starting model download');
      setIsDownloading(true);
      setDownloadProgress(0);

      // Get recommended summary model
      const recommendedModel = await invoke<string>('builtin_ai_get_recommended_model');

      // Start Parakeet download
      if (!isParakeetReady) {
        console.log('[useModelDownload] Starting Parakeet download');
        invoke('parakeet_download_model', { modelName: PARAKEET_MODEL }).catch(err => {
          if (!String(err).includes('Download already in progress')) {
            console.error('[useModelDownload] Parakeet download failed:', err);
          }
        });
      }

      // Start summary model download
      if (!isSummaryReady && recommendedModel) {
        console.log('[useModelDownload] Starting summary model download');
        invoke('builtin_ai_download_model', { modelName: recommendedModel }).catch(err => {
          if (!String(err).includes('Download already in progress')) {
            console.error('[useModelDownload] Summary download failed:', err);
          }
        });
      }
    } catch (error) {
      console.error('[useModelDownload] Failed to start download:', error);
      setIsDownloading(false);
    }
  }, [isParakeetReady, isSummaryReady]);

  // Listen to download progress
  useEffect(() => {
    const unlistenParakeet = listen<{
      modelName: string;
      progress: number;
      status?: string;
    }>('parakeet-model-download-progress', (event) => {
      const { modelName, progress, status } = event.payload;
      if (modelName === PARAKEET_MODEL) {
        setDownloadProgress(progress);
        if (status === 'completed' || progress >= 100) {
          setIsParakeetReady(true);
        }
      }
    });

    const unlistenParakeetComplete = listen<{ modelName: string }>(
      'parakeet-model-download-complete',
      (event) => {
        if (event.payload.modelName === PARAKEET_MODEL) {
          setIsParakeetReady(true);
          setDownloadProgress(100);
        }
      }
    );

    const unlistenSummary = listen<{
      model: string;
      progress: number;
      status: string;
    }>('builtin-ai-download-progress', (event) => {
      const { model, progress, status } = event.payload;
      if (status === 'completed' || progress >= 100) {
        setIsSummaryReady(true);
      }
    });

    // Check if downloads are already in progress
    const checkActiveDownloads = async () => {
      try {
        const models = await invoke<any[]>('parakeet_get_available_models');
        const isDownloading = models.some(m =>
          m.status && (
            typeof m.status === 'object'
              ? 'Downloading' in m.status
              : m.status === 'Downloading'
          )
        );
        if (isDownloading) {
          setIsDownloading(true);
        }
      } catch (error) {
        console.warn('[useModelDownload] Failed to check active downloads:', error);
      }
    };

    checkModelStatus();
    checkActiveDownloads();

    return () => {
      unlistenParakeet.then(fn => fn());
      unlistenParakeetComplete.then(fn => fn());
      unlistenSummary.then(fn => fn());
    };
  }, [checkModelStatus]);

  // Update downloading state based on model readiness
  useEffect(() => {
    if (isParakeetReady && isSummaryReady) {
      setIsDownloading(false);
    }
  }, [isParakeetReady, isSummaryReady]);

  return {
    isParakeetReady,
    isSummaryReady,
    isDownloading,
    downloadProgress,
    startDownload,
    checkModelStatus,
  };
}
