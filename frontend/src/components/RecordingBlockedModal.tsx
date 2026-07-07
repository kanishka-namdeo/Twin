'use client';

import React from 'react';
import { Download } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog';
import { useModelDownload } from '@/hooks/useModelDownload';

interface RecordingBlockedModalProps {
  isOpen: boolean;
  onClose: () => void;
}

/**
 * Modal shown when user tries to record without models downloaded.
 * Provides option to start downloading models or dismiss.
 */
export function RecordingBlockedModal({ isOpen, onClose }: RecordingBlockedModalProps) {
  const { isDownloading, downloadProgress, startDownload } = useModelDownload();

  const handleDownload = async () => {
    await startDownload();
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Download className="w-5 h-5" />
            Models Required
          </DialogTitle>
          <DialogDescription>
            Download AI models to enable meeting transcription. This takes a few minutes.
          </DialogDescription>
        </DialogHeader>

        <div className="py-4">
          {isDownloading && (
            <div className="space-y-2">
              <div className="flex items-center justify-between text-sm">
                <span className="text-gray-600">Downloading...</span>
                <span className="font-medium">{Math.round(downloadProgress)}%</span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div
                  className="bg-gray-900 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${downloadProgress}%` }}
                />
              </div>
            </div>
          )}
        </div>

        <DialogFooter className="flex gap-2 sm:gap-0">
          <Button
            variant="outline"
            onClick={onClose}
            disabled={isDownloading}
          >
            Later
          </Button>
          <Button
            onClick={handleDownload}
            disabled={isDownloading}
            className="bg-gray-900 hover:bg-gray-800 text-white"
          >
            {isDownloading ? (
              <>
                <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin mr-2" />
                Downloading...
              </>
            ) : (
              <>
                <Download className="w-4 h-4 mr-2" />
                Download Models
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
