"use client";

import { useState, useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { ButtonGroup } from '@/components/ui/button-group';
import { Copy, FolderOpen, RefreshCw, Download, FileText, Package, ChevronDown } from 'lucide-react';
import { RetranscribeDialog } from './RetranscribeDialog';
import { useConfig } from '@/contexts/ConfigContext';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { invoke } from '@tauri-apps/api/core';
import { save } from '@tauri-apps/plugin-dialog';
import { toast } from 'sonner';

interface TranscriptButtonGroupProps {
  transcriptCount: number;
  onCopyTranscript: () => void;
  onOpenMeetingFolder: () => Promise<void>;
  meetingId?: string;
  meetingFolderPath?: string | null;
  onRefetchTranscripts?: () => Promise<void>;
}

export function TranscriptButtonGroup({
  transcriptCount,
  onCopyTranscript,
  onOpenMeetingFolder,
  meetingId,
  meetingFolderPath,
  onRefetchTranscripts,
}: TranscriptButtonGroupProps) {
  const { betaFeatures } = useConfig();
  const [showRetranscribeDialog, setShowRetranscribeDialog] = useState(false);
  const [isExporting, setIsExporting] = useState(false);

  const handleRetranscribeComplete = useCallback(async () => {
    // Refetch transcripts to show the updated data
    if (onRefetchTranscripts) {
      await onRefetchTranscripts();
    }
  }, [onRefetchTranscripts]);

  // Handle export transcript
  const handleExportTranscript = async (format: 'srt' | 'vtt' | 'txt') => {
    if (!meetingId) {
      toast.error('No meeting ID available');
      return;
    }

    try {
      setIsExporting(true);

      // Show save dialog
      const defaultName = `transcript_${meetingId.substring(0, 8)}.${format}`;
      const outputPath = await save({
        defaultPath: defaultName,
        filters: [
          {
            name: format.toUpperCase(),
            extensions: [format],
          },
        ],
      });

      if (!outputPath) {
        // User cancelled the dialog
        setIsExporting(false);
        return;
      }

      // Call export command
      const result = await invoke<{ status: string; message: string }>('api_export_meeting_transcript', {
        meeting_id: meetingId,
        format: format,
        output_path: outputPath,
      });

      toast.success(`Transcript exported as ${format.toUpperCase()}`, {
        description: `Saved to: ${outputPath}`,
      });
    } catch (error) {
      console.error('Export failed:', error);
      toast.error('Failed to export transcript', {
        description: error instanceof Error ? error.message : String(error),
      });
    } finally {
      setIsExporting(false);
    }
  };

  // Handle export bundle (ZIP)
  const handleExportBundle = async () => {
    if (!meetingId) {
      toast.error('No meeting ID available');
      return;
    }

    try {
      setIsExporting(true);

      // Show save dialog
      const defaultName = `meeting_${meetingId.substring(0, 8)}_export.zip`;
      const outputPath = await save({
        defaultPath: defaultName,
        filters: [
          {
            name: 'ZIP Archive',
            extensions: ['zip'],
          },
        ],
      });

      if (!outputPath) {
        // User cancelled the dialog
        setIsExporting(false);
        return;
      }

      // Call export bundle command
      const result = await invoke<{ status: string; message: string }>('api_export_meeting_bundle', {
        meeting_id: meetingId,
        output_path: outputPath,
      });

      toast.success('Meeting bundle exported', {
        description: `Saved to: ${outputPath}`,
      });
    } catch (error) {
      console.error('Bundle export failed:', error);
      toast.error('Failed to export meeting bundle', {
        description: error instanceof Error ? error.message : String(error),
      });
    } finally {
      setIsExporting(false);
    }
  };

  return (
    <div className="flex items-center justify-center w-full gap-2">
      <ButtonGroup>
        <Button
          variant="outline"
          size="sm"
          onClick={() => {
            onCopyTranscript();
          }}
          disabled={transcriptCount === 0}
          title={transcriptCount === 0 ? 'No transcript available' : 'Copy Transcript'}
        >
          <Copy />
          <span className="hidden lg:inline">Copy</span>
        </Button>

        <Button
          size="sm"
          variant="outline"
          className="xl:px-4"
          onClick={() => {
            onOpenMeetingFolder();
          }}
          title="Open Recording Folder"
        >
          <FolderOpen className="xl:mr-2" size={18} />
          <span className="hidden lg:inline">Recording</span>
        </Button>

        {betaFeatures.importAndRetranscribe && meetingId && meetingFolderPath && (
          <Button
            size="sm"
            variant="outline"
            className="bg-gradient-to-r from-blue-50 to-purple-50 hover:from-blue-100 hover:to-purple-100 border-blue-200 xl:px-4"
            onClick={() => {
              setShowRetranscribeDialog(true);
            }}
            title="Retranscribe to enhance your recorded audio"
          >
            <RefreshCw className="xl:mr-2" size={18} />
            <span className="hidden lg:inline">Enhance</span>
          </Button>
        )}

        {/* Export dropdown */}
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button
              size="sm"
              variant="outline"
              disabled={transcriptCount === 0 || isExporting || !meetingId}
              className="xl:px-4"
              title="Export Transcript"
            >
              <Download className="xl:mr-2" size={18} />
              <span className="hidden lg:inline">Export</span>
              <ChevronDown className="ml-1" size={14} />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-48">
            <DropdownMenuLabel>Export Format</DropdownMenuLabel>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              onClick={() => handleExportTranscript('txt')}
              disabled={isExporting}
            >
              <FileText className="mr-2 h-4 w-4" />
              Plain Text (.txt)
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={() => handleExportTranscript('srt')}
              disabled={isExporting}
            >
              <FileText className="mr-2 h-4 w-4" />
              Subtitles (.srt)
            </DropdownMenuItem>
            <DropdownMenuItem
              onClick={() => handleExportTranscript('vtt')}
              disabled={isExporting}
            >
              <FileText className="mr-2 h-4 w-4" />
              WebVTT (.vtt)
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem
              onClick={() => handleExportBundle()}
              disabled={isExporting}
            >
              <Package className="mr-2 h-4 w-4" />
              ZIP Bundle
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </ButtonGroup>

      {betaFeatures.importAndRetranscribe && meetingId && meetingFolderPath && (
        <RetranscribeDialog
          open={showRetranscribeDialog}
          onOpenChange={setShowRetranscribeDialog}
          meetingId={meetingId}
          meetingFolderPath={meetingFolderPath}
          onComplete={handleRetranscribeComplete}
        />
      )}
    </div>
  );
}