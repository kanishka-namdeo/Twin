'use client';

import { useState, useRef, useEffect } from 'react';
import { Pencil, Check, X } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'sonner';

interface EditableTranscriptProps {
  transcriptId: string;
  meetingId: string;
  text: string;
  timestamp: number;
  confidence?: number;
  isPartial?: boolean;
  onSave?: (transcriptId: string, newText: string) => void;
  className?: string;
}

export function EditableTranscript({
  transcriptId,
  meetingId,
  text,
  timestamp,
  confidence,
  isPartial = false,
  onSave,
  className = '',
}: EditableTranscriptProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editedText, setEditedText] = useState(text);
  const [isSaving, setIsSaving] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);

  // Focus input when editing starts
  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  // Format time as MM:SS
  const formatTime = (seconds: number): string => {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  // Handle save
  const handleSave = async () => {
    if (editedText.trim() === text.trim()) {
      setIsEditing(false);
      return;
    }

    setIsSaving(true);
    try {
      await invoke('api_save_transcript_edit', {
        transcriptId,
        meetingId,
        editedText: editedText.trim(),
      });

      toast.success('Transcript updated');
      onSave?.(transcriptId, editedText.trim());
      setIsEditing(false);
    } catch (error) {
      console.error('Failed to save transcript edit:', error);
      toast.error('Failed to save edit');
    } finally {
      setIsSaving(false);
    }
  };

  // Handle cancel
  const handleCancel = () => {
    setEditedText(text);
    setIsEditing(false);
  };

  // Handle key press
  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSave();
    } else if (e.key === 'Escape') {
      handleCancel();
    }
  };

  // Get confidence-based styling
  const getConfidenceStyle = () => {
    if (confidence === undefined) return '';
    if (confidence < 0.5) return 'bg-red-50 border-red-200';
    if (confidence < 0.7) return 'bg-yellow-50 border-yellow-200';
    return '';
  };

  // Get text color based on partial/final
  const getTextColorClass = () => {
    if (isPartial) return 'text-gray-500';
    return 'text-gray-800';
  };

  return (
    <div className={`group relative flex items-start gap-2 mb-3 ${className}`}>
      {/* Timestamp */}
      <span className="text-xs text-gray-400 mt-1 flex-shrink-0 min-w-[50px]">
        {formatTime(timestamp)}
      </span>

      {/* Transcript content */}
      <div className="flex-1 relative">
        {isEditing ? (
          <div className="flex items-center gap-2">
            <Input
              ref={inputRef}
              value={editedText}
              onChange={(e) => setEditedText(e.target.value)}
              onKeyDown={handleKeyPress}
              disabled={isSaving}
              className="flex-1 h-8 text-sm"
            />
            <Button
              variant="ghost"
              size="sm"
              onClick={handleSave}
              disabled={isSaving}
              className="h-8 w-8 p-0"
            >
              <Check className="h-4 w-4 text-green-600" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleCancel}
              disabled={isSaving}
              className="h-8 w-8 p-0"
            >
              <X className="h-4 w-4 text-red-600" />
            </Button>
          </div>
        ) : (
          <div
            className={`relative p-2 rounded border border-transparent hover:border-gray-200 hover:bg-gray-50 transition-colors ${getConfidenceStyle()}`}
          >
            <p className={`text-base leading-relaxed ${getTextColorClass()}`}>
              {text}
            </p>

            {/* Edit button - shows on hover */}
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsEditing(true)}
              className="absolute top-1 right-1 h-6 w-6 p-0 opacity-0 group-hover:opacity-100 transition-opacity"
            >
              <Pencil className="h-3 w-3 text-gray-500" />
            </Button>

            {/* Confidence tooltip */}
            {confidence !== undefined && (
              <div className="absolute bottom-1 right-1 text-xs text-gray-400 opacity-0 group-hover:opacity-100 transition-opacity">
                {(confidence * 100).toFixed(0)}%
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
