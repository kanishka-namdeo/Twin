"use client";

import { useState, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useRouter } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Checkbox } from '@/components/ui/checkbox';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Skeleton } from '@/components/ui/skeleton';
import {
  CheckCircle2,
  Circle,
  Calendar,
  Filter,
  Copy,
  ExternalLink,
  ChevronLeft,
  ClipboardList,
  ArrowRight
} from 'lucide-react';
import { toast } from 'sonner';
import { format } from 'date-fns';

// Action item type matching backend ActionItemResponse
interface ActionItem {
  id: string;
  meeting_id: string;
  meeting_title: string;
  text: string;
  completed: boolean;
  created_at: string;
}

// Filter state
interface FilterState {
  meeting_id: string | null;
  completed: 'all' | 'completed' | 'pending';
  date_from: string | null;
  date_to: string | null;
}

export default function ActionItemsPage() {
  const router = useRouter();
  const [actionItems, setActionItems] = useState<ActionItem[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [meetings, setMeetings] = useState<{ id: string; title: string }[]>([]);
  const [filters, setFilters] = useState<FilterState>({
    meeting_id: null,
    completed: 'all',
    date_from: null,
    date_to: null,
  });
  const [showFilters, setShowFilters] = useState(false);

  // Load action items and meetings on mount
  useEffect(() => {
    const loadData = async () => {
      setIsLoading(true);
      try {
        // Load action items
        const items = await invoke<ActionItem[]>('api_get_all_action_items');
        setActionItems(items);

        // Load meetings for filter dropdown
        const meetingsData = await invoke<{ id: string; title: string }[]>('api_get_meetings');
        setMeetings(meetingsData || []);
      } catch (error) {
        console.error('Failed to load action items:', error);
        toast.error('Failed to load action items');
      } finally {
        setIsLoading(false);
      }
    };
    loadData();
  }, []);

  // Filtered action items
  const filteredItems = useMemo(() => {
    return actionItems.filter(item => {
      // Filter by meeting
      if (filters.meeting_id && item.meeting_id !== filters.meeting_id) {
        return false;
      }

      // Filter by completion status
      if (filters.completed === 'completed' && !item.completed) {
        return false;
      }
      if (filters.completed === 'pending' && item.completed) {
        return false;
      }

      // Filter by date range
      const itemDate = new Date(item.created_at);
      if (filters.date_from) {
        const fromDate = new Date(filters.date_from);
        if (itemDate < fromDate) return false;
      }
      if (filters.date_to) {
        const toDate = new Date(filters.date_to);
        if (itemDate > toDate) return false;
      }

      return true;
    });
  }, [actionItems, filters]);

  // Statistics
  const stats = useMemo(() => {
    const total = filteredItems.length;
    const completed = filteredItems.filter(i => i.completed).length;
    const pending = total - completed;
    return { total, completed, pending };
  }, [filteredItems]);

  // Toggle action item completion
  const handleToggleComplete = async (itemId: string, currentCompleted: boolean) => {
    try {
      await invoke('api_update_action_item', {
        action_item_id: itemId,
        completed: !currentCompleted,
      });

      // Update local state
      setActionItems(prev =>
        prev.map(item =>
          item.id === itemId ? { ...item, completed: !currentCompleted } : item
        )
      );

      toast.success(currentCompleted ? 'Marked as pending' : 'Marked as complete');
    } catch (error) {
      console.error('Failed to update action item:', error);
      toast.error('Failed to update action item');
    }
  };

  // Export as markdown checklist
  const handleExportMarkdown = async () => {
    const lines = filteredItems.map(item =>
      `- [${item.completed ? 'x' : ' '}] ${item.text} (${item.meeting_title})`
    );

    const header = `# Action Items\n\nGenerated: ${format(new Date(), 'PPP')}\n\n## Summary\n- Total: ${stats.total}\n- Completed: ${stats.completed}\n- Pending: ${stats.pending}\n\n## Items\n\n`;
    const markdown = header + lines.join('\n');

    try {
      await navigator.clipboard.writeText(markdown);
      toast.success('Action items copied to clipboard as markdown');
    } catch (error) {
      console.error('Failed to copy:', error);
      toast.error('Failed to copy to clipboard');
    }
  };

  // Navigate to meeting details
  const handleGoToMeeting = (meetingId: string) => {
    router.push(`/meeting-details?id=${meetingId}`);
  };

  // Format date for display
  const formatDate = (dateString: string) => {
    try {
      return format(new Date(dateString), 'MMM d, yyyy');
    } catch {
      return dateString;
    }
  };

  // Clear filters
  const handleClearFilters = () => {
    setFilters({
      meeting_id: null,
      completed: 'all',
      date_from: null,
      date_to: null,
    });
  };

  return (
    <div className="h-screen bg-[var(--background)] flex flex-col overflow-hidden">
      {/* Header */}
      <div className="bg-[var(--card)] border-b sticky top-0 z-10">
        <div className="max-w-4xl mx-auto px-4 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-3">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => router.push('/')}
                className="gap-2"
              >
                <ChevronLeft className="h-4 w-4" />
                Back
              </Button>
              <div className="flex items-center gap-2">
                <ClipboardList className="h-5 w-5 text-blue-600" />
                <h1 className="text-xl font-semibold">Action Items</h1>
              </div>
            </div>

            <div className="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowFilters(!showFilters)}
                className="gap-2"
              >
                <Filter className="h-4 w-4" />
                Filters
                {(filters.meeting_id || filters.completed !== 'all' || filters.date_from || filters.date_to) && (
                  <Badge variant="secondary" className="ml-1">Active</Badge>
                )}
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={handleExportMarkdown}
                disabled={filteredItems.length === 0}
                className="gap-2"
              >
                <Copy className="h-4 w-4" />
                Export
              </Button>
            </div>
          </div>

          {/* Filter panel */}
          {showFilters && (
            <div className="mt-4 p-4 bg-[var(--muted)] rounded-lg border-[var(--border)]">
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                {/* Meeting filter */}
                <div>
                  <Label className="text-sm text-gray-600">Meeting</Label>
                  <Select
                    value={filters.meeting_id || 'all'}
                    onValueChange={(value) =>
                      setFilters(prev => ({
                        ...prev,
                        meeting_id: value === 'all' ? null : value,
                      }))
                    }
                  >
                    <SelectTrigger className="mt-1">
                      <SelectValue placeholder="All meetings" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">All meetings</SelectItem>
                      {meetings.map(m => (
                        <SelectItem key={m.id} value={m.id}>
                          {m.title}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                {/* Status filter */}
                <div>
                  <Label className="text-sm text-gray-600">Status</Label>
                  <Select
                    value={filters.completed}
                    onValueChange={(value: 'all' | 'completed' | 'pending') =>
                      setFilters(prev => ({
                        ...prev,
                        completed: value,
                      }))
                    }
                  >
                    <SelectTrigger className="mt-1">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="all">All</SelectItem>
                      <SelectItem value="completed">Completed</SelectItem>
                      <SelectItem value="pending">Pending</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                {/* Date from filter */}
                <div>
                  <Label className="text-sm text-gray-600">From Date</Label>
                  <Input
                    type="date"
                    value={filters.date_from || ''}
                    onChange={(e) =>
                      setFilters(prev => ({
                        ...prev,
                        date_from: e.target.value || null,
                      }))
                    }
                    className="mt-1"
                  />
                </div>

                {/* Date to filter */}
                <div>
                  <Label className="text-sm text-gray-600">To Date</Label>
                  <Input
                    type="date"
                    value={filters.date_to || ''}
                    onChange={(e) =>
                      setFilters(prev => ({
                        ...prev,
                        date_to: e.target.value || null,
                      }))
                    }
                    className="mt-1"
                  />
                </div>
              </div>

              {/* Clear filters button */}
              {(filters.meeting_id || filters.completed !== 'all' || filters.date_from || filters.date_to) && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleClearFilters}
                  className="mt-3 text-gray-500"
                >
                  Clear all filters
                </Button>
              )}
            </div>
          )}
        </div>
      </div>

      {/* Main content */}
      <div className="flex-1 overflow-y-auto">
        <div className="max-w-4xl mx-auto px-4 py-6">
        {/* Stats cards */}
        <div className="grid grid-cols-3 gap-4 mb-6">
          <Card>
            <CardContent className="p-4">
              <div className="text-2xl font-bold">{stats.total}</div>
              <div className="text-sm text-gray-500">Total</div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="p-4">
              <div className="text-2xl font-bold text-green-600">{stats.completed}</div>
              <div className="text-sm text-gray-500">Completed</div>
            </CardContent>
          </Card>
          <Card>
            <CardContent className="p-4">
              <div className="text-2xl font-bold text-orange-600">{stats.pending}</div>
              <div className="text-sm text-gray-500">Pending</div>
            </CardContent>
          </Card>
        </div>

        {/* Action items list */}
        {isLoading ? (
          <div className="space-y-3">
            {[1, 2, 3].map(i => (
              <Skeleton key={i} className="h-16 w-full rounded-lg" />
            ))}
          </div>
        ) : filteredItems.length === 0 ? (
          <Card>
            <CardContent className="p-8 text-center">
              <ClipboardList className="h-12 w-12 text-gray-300 mx-auto mb-4" />
              <h3 className="text-lg font-medium text-gray-900 mb-2">
                {actionItems.length === 0 ? 'No action items found' : 'No items match your filters'}
              </h3>
              <p className="text-gray-500">
                {actionItems.length === 0
                  ? 'Action items are generated when AI summaries include them.'
                  : 'Try adjusting your filters to see more items.'}
              </p>
              {actionItems.length > 0 && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleClearFilters}
                  className="mt-4"
                >
                  Clear filters
                </Button>
              )}
            </CardContent>
          </Card>
        ) : (
          <div className="space-y-3">
            {filteredItems.map(item => (
              <Card
                key={item.id}
                className={`transition-colors ${
                  item.completed ? 'bg-[var(--muted)] border-[var(--border)]' : 'bg-[var(--card)] border-[var(--border)]'
                }`}
              >
                <CardContent className="p-4">
                  <div className="flex items-start gap-3">
                    {/* Checkbox */}
                    <Checkbox
                      checked={item.completed}
                      onCheckedChange={() => handleToggleComplete(item.id, item.completed)}
                      className="mt-0.5"
                    />

                    {/* Content */}
                    <div className="flex-1 min-w-0">
                      <p
                        className={`text-sm ${
                          item.completed ? 'text-gray-500 line-through' : 'text-gray-900'
                        }`}
                      >
                        {item.text}
                      </p>

                      {/* Metadata */}
                      <div className="flex items-center gap-4 mt-2 text-xs text-gray-500">
                        <div className="flex items-center gap-1">
                          <Calendar className="h-3 w-3" />
                          {formatDate(item.created_at)}
                        </div>
                        <div
                          className="flex items-center gap-1 cursor-pointer hover:text-blue-600 transition-colors"
                          onClick={() => handleGoToMeeting(item.meeting_id)}
                        >
                          <ExternalLink className="h-3 w-3" />
                          <span className="truncate max-w-[150px]">{item.meeting_title}</span>
                        </div>
                      </div>
                    </div>

                    {/* Go to meeting button */}
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleGoToMeeting(item.meeting_id)}
                      className="gap-1 text-gray-500 hover:text-blue-600"
                    >
                      <ArrowRight className="h-4 w-4" />
                    </Button>
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        )}
      </div>
      </div>
    </div>
  );
}