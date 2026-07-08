"use client";

import { Transcript, TranscriptSegmentData } from '@/types';
import { TranscriptView } from '@/components/TranscriptView';
import { VirtualizedTranscriptView } from '@/components/VirtualizedTranscriptView';
import { TranscriptButtonGroup } from './TranscriptButtonGroup';
import { AudioPlayer } from '@/components/Recording/AudioPlayer';
import { useMemo, useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ChevronDown, ChevronUp, Save, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { toast } from 'sonner';

interface TranscriptPanelProps {
  transcripts: Transcript[];
  customPrompt: string;
  onPromptChange: (value: string) => void;
  onCopyTranscript: () => void;
  onOpenMeetingFolder: () => Promise<void>;
  isRecording: boolean;
  disableAutoScroll?: boolean;

  // Optional pagination props (when using virtualization)
  usePagination?: boolean;
  segments?: TranscriptSegmentData[];
  hasMore?: boolean;
  isLoadingMore?: boolean;
  totalCount?: number;
  loadedCount?: number;
  onLoadMore?: () => void;

  // Retranscription props
  meetingId?: string;
  meetingFolderPath?: string | null;
  onRefetchTranscripts?: () => Promise<void>;

  // Audio playback props
  showAudioPlayer?: boolean;
  onAudioTimeUpdate?: (currentTime: number) => void;
  onAudioSeek?: (time: number) => void;
  highlightSegmentId?: string | null;
}

export function TranscriptPanel({
  transcripts,
  customPrompt,
  onPromptChange,
  onCopyTranscript,
  onOpenMeetingFolder,
  isRecording,
  disableAutoScroll = false,
  usePagination = false,
  segments,
  hasMore,
  isLoadingMore,
  totalCount,
  loadedCount,
  onLoadMore,
  meetingId,
  meetingFolderPath,
  onRefetchTranscripts,
  showAudioPlayer = false,
  onAudioTimeUpdate,
  onAudioSeek,
  highlightSegmentId,
}: TranscriptPanelProps) {
  // Convert transcripts to segments if pagination is not used but we want virtualization
  const convertedSegments = useMemo(() => {
    if (usePagination && segments) {
      return segments;
    }
    // Convert transcripts to segments for virtualization
    return transcripts.map(t => ({
      id: t.id,
      timestamp: t.audio_start_time ?? 0,
      endTime: t.audio_end_time,
      text: t.text,
      confidence: t.confidence,
    }));
  }, [transcripts, usePagination, segments]);

  // Meeting context state
  const [meetingContext, setMeetingContext] = useState<string>('');
  const [savedContext, setSavedContext] = useState<string | null>(null);
  const [isContextExpanded, setIsContextExpanded] = useState<boolean>(false);
  const [isSavingContext, setIsSavingContext] = useState<boolean>(false);
  const [isLoadingContext, setIsLoadingContext] = useState<boolean>(false);

  // Load meeting context when meetingId changes
  useEffect(() => {
    if (meetingId) {
      setIsLoadingContext(true);
      invoke<string | null>('api_get_meeting_context', { meetingId })
        .then((context) => {
          setSavedContext(context);
          if (context) {
            // Show saved context as collapsed section initially
            setIsContextExpanded(false);
          }
        })
        .catch((err) => {
          console.error('Failed to load meeting context:', err);
        })
        .finally(() => {
          setIsLoadingContext(false);
        });
    }
  }, [meetingId]);

  // Save meeting context
  const handleSaveContext = useCallback(async () => {
    if (!meetingId || !meetingContext.trim()) return;

    setIsSavingContext(true);
    try {
      await invoke('api_save_meeting_context', {
        meeting_id: meetingId,
        context: meetingContext.trim(),
      });

      setSavedContext(meetingContext.trim());
      setMeetingContext('');
      setIsContextExpanded(false);

      toast.success('Meeting context saved');
    } catch (err) {
      console.error('Failed to save meeting context:', err);
      toast.error('Failed to save meeting context');
    } finally {
      setIsSavingContext(false);
    }
  }, [meetingId, meetingContext]);

  // Check if there's unsaved context
  const hasUnsavedContext = meetingContext.trim().length > 0;

  return (
    <div className="hidden md:flex md:w-1/4 lg:w-1/3 min-w-0 border-r border-gray-200 bg-white flex-col relative shrink-0">
      {/* Title area */}
      <div className="p-4 border-b border-gray-200">
        <TranscriptButtonGroup
          transcriptCount={usePagination ? (totalCount ?? convertedSegments.length) : (transcripts?.length || 0)}
          onCopyTranscript={onCopyTranscript}
          onOpenMeetingFolder={onOpenMeetingFolder}
          meetingId={meetingId}
          meetingFolderPath={meetingFolderPath}
          onRefetchTranscripts={onRefetchTranscripts}
        />
      </div>

      {/* Audio player - shown when enabled */}
      {showAudioPlayer && meetingId && (
        <div className="border-b border-gray-200">
          <AudioPlayer
            meetingId={meetingId}
            folderPath={meetingFolderPath}
            onTimeUpdate={onAudioTimeUpdate}
            onSeek={onAudioSeek}
          />
        </div>
      )}

      {/* Saved Meeting Context (collapsible) */}
      {!isRecording && savedContext && !isLoadingContext && (
        <div className="border-b border-gray-200">
          <div
            className="flex items-center justify-between px-3 py-2 cursor-pointer hover:bg-gray-50 transition-colors"
            onClick={() => setIsContextExpanded(!isContextExpanded)}
          >
            <span className="text-sm font-medium text-gray-700">Meeting Context</span>
            {isContextExpanded ? (
              <ChevronUp className="h-4 w-4 text-gray-500" />
            ) : (
              <ChevronDown className="h-4 w-4 text-gray-500" />
            )}
          </div>
          {isContextExpanded && (
            <div className="px-3 py-2 pt-0 text-sm text-gray-600 bg-gray-50 border-t">
              {savedContext}
            </div>
          )}
        </div>
      )}

      {/* Transcript content - use virtualized view for better performance */}
      <div className="flex-1 overflow-hidden pb-4">
        <VirtualizedTranscriptView
          segments={convertedSegments}
          isRecording={isRecording}
          isPaused={false}
          isProcessing={false}
          isStopping={false}
          enableStreaming={false}
          showConfidence={true}
          disableAutoScroll={disableAutoScroll}
          hasMore={hasMore}
          isLoadingMore={isLoadingMore}
          totalCount={totalCount}
          loadedCount={loadedCount}
          onLoadMore={onLoadMore}
          highlightSegmentId={highlightSegmentId}
        />
      </div>

      {/* Meeting Context Input for AI Summary */}
      {!isRecording && convertedSegments.length > 0 && (
        <div className="p-2 border-t border-gray-200">
          {/* Context input */}
          <div className="space-y-2">
            <textarea
              placeholder="Add agenda, attendees, project names for better summaries..."
              className="w-full px-3 py-2 border border-gray-200 rounded-md text-sm focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 bg-white shadow-sm min-h-[60px] resize-y"
              value={meetingContext}
              onChange={(e) => setMeetingContext(e.target.value)}
            />

            {/* Save context button */}
            {hasUnsavedContext && (
              <Button
                size="sm"
                variant="outline"
                onClick={handleSaveContext}
                disabled={isSavingContext}
                className="w-full gap-2"
              >
                {isSavingContext ? (
                  <Loader2 className="h-3 w-3 animate-spin" />
                ) : (
                  <Save className="h-3 w-3" />
                )}
                Save Context
              </Button>
            )}
          </div>

          {/* Legacy custom prompt (kept for compatibility) */}
          <textarea
            placeholder="Additional prompt instructions for AI..."
            className="w-full px-3 py-2 border border-gray-200 rounded-md text-sm focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 bg-white shadow-sm min-h-[40px] resize-y mt-2"
            value={customPrompt}
            onChange={(e) => onPromptChange(e.target.value)}
          />
        </div>
      )}
    </div>
  );
}