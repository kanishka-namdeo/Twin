'use client';

import React from 'react';
import { Download, Sparkles } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useModelDownload } from '@/hooks/useModelDownload';

interface EmptyStateNudgeProps {
  onClose?: () => void;
}

/**
 * Empty-state nudge component shown when no transcription models are present.
 * Provides a prominent "Download Models" button to trigger model download.
 */
export function EmptyStateNudge({ onClose }: EmptyStateNudgeProps) {
  const { isParakeetReady, isSummaryReady, isDownloading, downloadProgress, startDownload } = useModelDownload();

  // Don't show if models are ready
  if (isParakeetReady && isSummaryReady) {
    return null;
  }

  const handleDownload = async () => {
    await startDownload();
  };

  return (
    <div className="flex flex-col items-center justify-center min-h-[400px] p-8 animate-in fade-in duration-300">
      <div className="max-w-md w-full bg-white rounded-lg border border-gray-200 shadow-sm p-8 space-y-6">
        {/* Icon */}
        <div className="flex justify-center">
          <div className="w-16 h-16 rounded-full bg-gray-100 flex items-center justify-center">
            <Download className="w-8 h-8 text-gray-600" />
          </div>
        </div>

        {/* Title */}
        <div className="text-center space-y-2">
          <h2 className="text-xl font-semibold text-gray-900">
            Download AI Models
          </h2>
          <p className="text-sm text-gray-600">
            Download AI models to start transcribing meetings. This takes a few minutes.
          </p>
        </div>

        {/* Download Button */}
        <div className="space-y-3">
          <Button
            onClick={handleDownload}
            disabled={isDownloading}
            className="w-full h-11 bg-gray-900 hover:bg-gray-800 text-white disabled:opacity-50"
          >
            {isDownloading ? (
              <>
                <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin mr-2" />
                Downloading... {Math.round(downloadProgress)}%
              </>
            ) : (
              <>
                <Download className="w-4 h-4 mr-2" />
                Download Models
              </>
            )}
          </Button>

          {onClose && (
            <button
              onClick={onClose}
              className="w-full text-sm text-gray-500 hover:text-gray-700 transition-colors"
            >
              Skip for now
            </button>
          )}
        </div>

        {/* Info */}
        <div className="pt-4 border-t border-gray-100">
          <div className="flex items-start gap-3 text-xs text-gray-500">
            <Sparkles className="w-4 h-4 flex-shrink-0 mt-0.5" />
            <p>
              All AI processing happens locally on your device. Your data never leaves your computer.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
