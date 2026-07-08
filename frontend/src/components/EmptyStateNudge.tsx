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
    <div className="flex flex-col items-center justify-center min-h-[280px] md:min-h-[400px] p-8 animate-in fade-in duration-300">
      <div className="max-w-md w-full bg-[var(--card)] rounded-lg border border-[var(--border)] shadow-sm p-8 space-y-6">
        {/* Icon */}
        <div className="flex justify-center">
          <div className="w-16 h-16 rounded-full bg-[var(--muted)] flex items-center justify-center">
            <Download className="w-8 h-8 text-[var(--muted-foreground)]" />
          </div>
        </div>

        {/* Title */}
        <div className="text-center space-y-2">
          <h2 className="text-xl font-semibold text-[var(--foreground)]">
            Download AI Models
          </h2>
          <p className="text-sm text-[var(--muted-foreground)]">
            Download AI models to start transcribing meetings. This takes a few minutes.
          </p>
        </div>

        {/* Download Button */}
        <div className="space-y-3">
          <Button
            onClick={handleDownload}
            disabled={isDownloading}
            className="w-full h-11 bg-[var(--primary)] hover:bg-[var(--primary)]/90 text-[var(--primary-foreground)] disabled:opacity-50"
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
              className="w-full text-sm text-[var(--muted-foreground)] hover:text-[var(--foreground)] transition-colors"
            >
              Skip for now
            </button>
          )}
        </div>

        {/* Info */}
        <div className="pt-4 border-t border-[var(--border)]">
          <div className="flex items-start gap-3 text-xs text-[var(--muted-foreground)]">
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
