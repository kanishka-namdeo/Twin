"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import { invoke } from "@tauri-apps/api/core";
import { FileText, Calendar, Clock, ArrowRight, StickyNote } from "lucide-react";
import { format } from "date-fns";

interface MeetingWithNotes {
  meeting_id: string;
  meeting_title: string;
  notes_markdown: string | null;
  notes_json: string | null;
  created_at: { 0: string };
  updated_at: { 0: string };
}

export default function NotesPage() {
  const router = useRouter();
  const [meetingsWithNotes, setMeetingsWithNotes] = useState<MeetingWithNotes[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchMeetingsWithNotes();
  }, []);

  const fetchMeetingsWithNotes = async () => {
    try {
      setLoading(true);
      const result = await invoke("api_get_meetings_with_notes");
      setMeetingsWithNotes(result as MeetingWithNotes[]);
    } catch (error) {
      console.error("Failed to fetch meetings with notes:", error);
    } finally {
      setLoading(false);
    }
  };

  const formatDate = (dateStr: { 0: string }) => {
    try {
      const date = new Date(dateStr[0]);
      return format(date, "MMM d, yyyy");
    } catch {
      return "Unknown date";
    }
  };

  const formatTime = (dateStr: { 0: string }) => {
    try {
      const date = new Date(dateStr[0]);
      return format(date, "h:mm a");
    } catch {
      return "";
    }
  };

  const getPreview = (markdown: string | null): string => {
    if (!markdown) return "No content";
    // Strip markdown headers and get first meaningful line
    const lines = markdown.split("\n").filter(l => l.trim() && !l.startsWith("#"));
    if (lines.length === 0) return "Empty note";
    return lines[0].replace(/[*_`]/g, "").slice(0, 120) + (lines[0].length > 120 ? "..." : "");
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" />
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col overflow-hidden">
      <div className="p-6 max-w-4xl mx-auto flex-1 overflow-y-auto">
      <div className="mb-6">
        <h1 className="text-2xl font-bold flex items-center gap-2">
          <StickyNote className="w-6 h-6 text-blue-600" />
          Meeting Notes
        </h1>
        <p className="text-sm text-gray-500 mt-1">
          Notes you've taken during meetings
        </p>
      </div>

      {meetingsWithNotes.length === 0 ? (
        <div className="text-center py-16">
          <FileText className="w-12 h-12 text-gray-300 mx-auto mb-4" />
          <h3 className="text-lg font-medium text-gray-600 mb-2">No notes yet</h3>
          <p className="text-sm text-gray-400 max-w-sm mx-auto">
            Start taking notes during a meeting by clicking the notes icon in the recording panel.
            Your notes will appear here.
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {meetingsWithNotes.map((meeting) => (
            <div
              key={meeting.meeting_id}
              onClick={() => router.push(`/notes/${meeting.meeting_id}`)}
              className="group p-4 bg-[var(--card)] border border-[var(--border)] rounded-lg hover:border-[var(--accent)]/30 hover:shadow-sm transition-all cursor-pointer"
            >
              <div className="flex items-start justify-between gap-4">
                <div className="flex-1 min-w-0">
                  <h3 className="font-medium text-gray-900 group-hover:text-blue-600 transition-colors truncate">
                    {meeting.meeting_title}
                  </h3>
                  <p className="text-sm text-gray-500 mt-1 line-clamp-2">
                    {getPreview(meeting.notes_markdown)}
                  </p>
                  <div className="flex items-center gap-3 mt-2 text-xs text-gray-400">
                    <span className="flex items-center gap-1">
                      <Calendar className="w-3 h-3" />
                      {formatDate(meeting.updated_at)}
                    </span>
                    <span className="flex items-center gap-1">
                      <Clock className="w-3 h-3" />
                      {formatTime(meeting.updated_at)}
                    </span>
                  </div>
                </div>
                <ArrowRight className="w-4 h-4 text-gray-300 group-hover:text-blue-500 transition-colors flex-shrink-0 mt-1" />
              </div>
            </div>
          ))}
        </div>
      )}
      </div>
    </div>
  );
}
