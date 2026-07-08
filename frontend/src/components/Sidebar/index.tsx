'use client';

import React, { useState, useMemo, useEffect, useCallback } from 'react';
import { ChevronDown, ChevronRight, File, Settings, ChevronLeftCircle, ChevronRightCircle, Calendar, StickyNote, Home, Trash2, Mic, Square, Plus, Search, Pencil, NotebookPen, SearchIcon, X, Upload, Download, Filter, CheckCircle, ClipboardList } from 'lucide-react';
import { useRouter, usePathname } from 'next/navigation';
import { useSidebar } from './SidebarProvider';
import type { CurrentMeeting } from '@/components/Sidebar/SidebarProvider';
import { ConfirmationModal } from '../ConfirmationModel/confirmation-modal';
import { ModelConfig } from '@/components/ModelSettingsModal';
import { SettingTabs } from '../SettingTabs';
import { TranscriptModelProps } from '@/components/TranscriptSettings';
import { invoke } from '@tauri-apps/api/core';
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip';
import { toast } from 'sonner';
import { useRecordingState } from '@/contexts/RecordingStateContext';
import { useImportDialog } from '@/contexts/ImportDialogContext';
import { useConfig } from '@/contexts/ConfigContext';
import { useModelDownload } from '@/hooks/useModelDownload';

import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogTitle,
} from "@/components/ui/dialog"
import { VisuallyHidden } from "@/components/ui/visually-hidden"
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Label } from '@/components/ui/label';

import { MessageToast } from '../MessageToast';
import Logo from '../Logo';
import Info from '../Info';
import { Input } from '../ui/input';
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput } from '../ui/input-group';

interface SidebarItem {
  id: string;
  title: string;
  type: 'folder' | 'file';
  children?: SidebarItem[];
}

const Sidebar: React.FC = () => {
  const router = useRouter();
  const pathname = usePathname();
  const {
    currentMeeting,
    setCurrentMeeting,
    sidebarItems,
    isCollapsed,
    toggleCollapse,
    handleRecordingToggle,
    searchTranscripts,
    searchResults,
    isSearching,
    meetings,
    setMeetings,
    serverAddress,
    // FTS5 search
    searchMeetingsFts,
    ftsSearchResults,
    searchFilters,
    setSearchFilters,
    clearSearchFilters,
  } = useSidebar();

  // Get recording state from RecordingStateContext (single source of truth)
  const { isRecording } = useRecordingState();
  const { openImportDialog } = useImportDialog();
  const { betaFeatures } = useConfig();
  const { isParakeetReady, isSummaryReady, startDownload, checkModelStatus } = useModelDownload();
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set(['meetings']));
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [showModelSettings, setShowModelSettings] = useState(false);
  const [showSearchFilters, setShowSearchFilters] = useState(false);
  const [modelConfig, setModelConfig] = useState<ModelConfig>({
    provider: 'ollama',
    model: '',
    whisperModel: '',
    apiKey: null,
    ollamaEndpoint: null
  });
  const [transcriptModelConfig, setTranscriptModelConfig] = useState<TranscriptModelProps>({
    provider: 'parakeet',
    model: 'parakeet-tdt-0.6b-v3-int8',
  });
  const [settingsSaveSuccess, setSettingsSaveSuccess] = useState<boolean | null>(null);

  // State for edit modal
  const [editModalState, setEditModalState] = useState<{ isOpen: boolean; meetingId: string | null; currentTitle: string }>({
    isOpen: false,
    meetingId: null,
    currentTitle: ''
  });
  const [editingTitle, setEditingTitle] = useState<string>('');

  // Ensure 'meetings' folder is always expanded
  useEffect(() => {
    if (!expandedFolders.has('meetings')) {
      const newExpanded = new Set(expandedFolders);
      newExpanded.add('meetings');
      setExpandedFolders(newExpanded);
    }
  }, [expandedFolders]);

  // useEffect(() => {
  //   if (settingsSaveSuccess !== null) {
  //     const timer = setTimeout(() => {
  //       setSettingsSaveSuccess(null);
  //     }, 3000);
  //   }
  // }, [settingsSaveSuccess]);


  const [deleteModalState, setDeleteModalState] = useState<{ isOpen: boolean; itemId: string | null }>({ isOpen: false, itemId: null });

  useEffect(() => {
    // Note: Don't set hardcoded defaults - let DB be the source of truth
    const fetchModelConfig = async () => {
      // Only make API call if serverAddress is loaded
      if (!serverAddress) {
        console.log('Waiting for server address to load before fetching model config');
        return;
      }

      try {
        const data = await invoke('api_get_model_config') as any;
        if (data && data.provider !== null) {
          // Fetch API key if not included and provider requires it
          if (data.provider !== 'ollama' && !data.apiKey) {
            try {
              const apiKeyData = await invoke('api_get_api_key', {
                provider: data.provider
              }) as string;
              data.apiKey = apiKeyData;
            } catch (err) {
              console.error('Failed to fetch API key:', err);
            }
          }
          setModelConfig(data);
        }
      } catch (error) {
        console.error('Failed to fetch model config:', error);
      }
    };

    fetchModelConfig();
  }, [serverAddress]);


  useEffect(() => {
    // Note: Don't set hardcoded defaults - let DB be the source of truth
    const fetchTranscriptSettings = async () => {
      // Only make API call if serverAddress is loaded
      if (!serverAddress) {
        console.log('Waiting for server address to load before fetching transcript settings');
        return;
      }

      try {
        const data = await invoke('api_get_transcript_config') as any;
        if (data && data.provider !== null) {
          setTranscriptModelConfig(data);
        }
      } catch (error) {
        console.error('Failed to fetch transcript settings:', error);
      }
    };
    fetchTranscriptSettings();
  }, [serverAddress]);

  // Listen for model config updates from other components
  useEffect(() => {
    const setupListener = async () => {
      const { listen } = await import('@tauri-apps/api/event');
      const unlisten = await listen<ModelConfig>('model-config-updated', (event) => {
        console.log('Sidebar received model-config-updated event:', event.payload);
        setModelConfig(event.payload);
      });

      return unlisten;
    };

    let cleanup: (() => void) | undefined;
    setupListener().then(fn => cleanup = fn);

    return () => {
      cleanup?.();
    };
  }, []);



  // Handle model config save
  const handleSaveModelConfig = async (config: ModelConfig) => {
    try {
      await invoke('api_save_model_config', {
        provider: config.provider,
        model: config.model,
        whisperModel: config.whisperModel,
        apiKey: config.apiKey,
        ollamaEndpoint: config.ollamaEndpoint,
      });

      setModelConfig(config);
      console.log('Model config saved successfully');
      setSettingsSaveSuccess(true);

      // Emit event to sync other components
      const { emit } = await import('@tauri-apps/api/event');
      await emit('model-config-updated', config);

    } catch (error) {
      console.error('Error saving model config:', error);
      setSettingsSaveSuccess(false);
    }
  };

  const handleSaveTranscriptConfig = async (updatedConfig?: TranscriptModelProps) => {
    try {
      const configToSave = updatedConfig || transcriptModelConfig;
      const payload = {
        provider: configToSave.provider,
        model: configToSave.model,
        apiKey: configToSave.apiKey ?? null
      };
      console.log('Saving transcript config with payload:', payload);

      await invoke('api_save_transcript_config', {
        provider: payload.provider,
        model: payload.model,
        apiKey: payload.apiKey,
      });


      setSettingsSaveSuccess(true);

    } catch (error) {
      console.error('Failed to save transcript config:', error);
      setSettingsSaveSuccess(false);
    }
  };

  // Handle search input changes - use FTS5 search
  const handleSearchChange = useCallback(async (value: string) => {
    setSearchQuery(value);

    // If search query is empty, just return to normal view
    if (!value.trim()) {
      return;
    }

    // Use FTS5 search with current filters
    await searchMeetingsFts(value, searchFilters);

    // Make sure the meetings folder is expanded when searching
    if (!expandedFolders.has('meetings')) {
      const newExpanded = new Set(expandedFolders);
      newExpanded.add('meetings');
      setExpandedFolders(newExpanded);
    }
  }, [expandedFolders, searchMeetingsFts, searchFilters]);

  // Count active filters
  const activeFilterCount = useMemo(() => {
    let count = 0;
    if (searchFilters.date_from) count++;
    if (searchFilters.date_to) count++;
    if (searchFilters.min_duration) count++;
    if (searchFilters.has_summary !== null) count++;
    return count;
  }, [searchFilters]);

  // Combine FTS5 search results with sidebar items
  const filteredSidebarItems = useMemo(() => {
    if (!searchQuery.trim()) return sidebarItems;

    // If we have FTS5 search results, highlight matching meetings with snippets
    if (ftsSearchResults.length > 0) {
      // Get the IDs of meetings that matched
      const matchedMeetingIds = new Set(ftsSearchResults.map(result => result.meeting_id));
      // Create a map of meeting ID to snippet for display
      const snippetMap = new Map(ftsSearchResults.map(result => [result.meeting_id, result.snippet]));

      return sidebarItems
        .map(folder => {
          // Always include folders in the results
          if (folder.type === 'folder') {
            if (!folder.children) return folder;

            // Filter children based on search results or title match
            const filteredChildren = folder.children.filter(item => {
              // Include if the meeting ID is in our search results
              if (matchedMeetingIds.has(item.id)) return true;

              // Or if the title matches the search query
              return item.title.toLowerCase().includes(searchQuery.toLowerCase());
            });

            return {
              ...folder,
              children: filteredChildren
            };
          }

          // For non-folder items, check if they match the search
          return (matchedMeetingIds.has(folder.id) ||
            folder.title.toLowerCase().includes(searchQuery.toLowerCase()))
            ? folder : undefined;
        })
        .filter((item): item is SidebarItem => item !== undefined); // Type-safe filter
    } else {
      // Fall back to title-only filtering if no FTS5 results
      return sidebarItems
        .map(folder => {
          // Always include folders in the results
          if (folder.type === 'folder') {
            if (!folder.children) return folder;

            // Filter children based on search query
            const filteredChildren = folder.children.filter(item =>
              item.title.toLowerCase().includes(searchQuery.toLowerCase())
            );

            return {
              ...folder,
              children: filteredChildren
            };
          }

          // For non-folder items, check if they match the search
          return folder.title.toLowerCase().includes(searchQuery.toLowerCase()) ? folder : undefined;
        })
        .filter((item): item is SidebarItem => item !== undefined); // Type-safe filter
    }
  }, [sidebarItems, searchQuery, ftsSearchResults, expandedFolders]);


  const handleDelete = async (itemId: string) => {
    console.log('Deleting item:', itemId);
    const payload = {
      meetingId: itemId
    };

    try {
      const { invoke } = await import('@tauri-apps/api/core');
      await invoke('api_delete_meeting', {
        meetingId: itemId,
      });
      console.log('Meeting deleted successfully');
      const updatedMeetings = meetings.filter((m: CurrentMeeting) => m.id !== itemId);
      setMeetings(updatedMeetings);

      // Show success toast
      toast.success("Meeting deleted successfully", {
        description: "All associated data has been removed"
      });

      // If deleting the active meeting, navigate to home
      if (currentMeeting?.id === itemId) {
        setCurrentMeeting({ id: 'intro-call', title: '+ New Call' });
        router.push('/');
      }
    } catch (error) {
      console.error('Failed to delete meeting:', error);
      toast.error("Failed to delete meeting", {
        description: error instanceof Error ? error.message : String(error)
      });
    }
  };

  const handleDeleteConfirm = () => {
    if (deleteModalState.itemId) {
      handleDelete(deleteModalState.itemId);
    }
    setDeleteModalState({ isOpen: false, itemId: null });
  };

  // Handle modal editing of meeting names
  const handleEditStart = (meetingId: string, currentTitle: string) => {
    setEditModalState({
      isOpen: true,
      meetingId: meetingId,
      currentTitle: currentTitle
    });
    setEditingTitle(currentTitle);
  };

  const handleEditConfirm = async () => {
    const newTitle = editingTitle.trim();
    const meetingId = editModalState.meetingId;

    if (!meetingId) return;

    // Prevent empty titles
    if (!newTitle) {
      toast.error("Meeting title cannot be empty");
      return;
    }

    try {
      await invoke('api_save_meeting_title', {
        meetingId: meetingId,
        title: newTitle,
      });

      // Update local state
      const updatedMeetings = meetings.map((m: CurrentMeeting) =>
        m.id === meetingId ? { ...m, title: newTitle } : m
      );
      setMeetings(updatedMeetings);

      // Update current meeting if it's the one being edited
      if (currentMeeting?.id === meetingId) {
        setCurrentMeeting({ id: meetingId, title: newTitle });
      }

      toast.success("Meeting title updated successfully");

      // Close modal and reset state
      setEditModalState({ isOpen: false, meetingId: null, currentTitle: '' });
      setEditingTitle('');
    } catch (error) {
      console.error('Failed to update meeting title:', error);
      toast.error("Failed to update meeting title", {
        description: error instanceof Error ? error.message : String(error)
      });
    }
  };

  const handleEditCancel = () => {
    setEditModalState({ isOpen: false, meetingId: null, currentTitle: '' });
    setEditingTitle('');
  };

  const toggleFolder = (folderId: string) => {
    // Normal toggle behavior for all folders
    const newExpanded = new Set(expandedFolders);
    if (newExpanded.has(folderId)) {
      newExpanded.delete(folderId);
    } else {
      newExpanded.add(folderId);
    }
    setExpandedFolders(newExpanded);
  };

  // Expose setShowModelSettings to window for Rust tray to call
  useEffect(() => {
    (window as any).openSettings = () => {
      setShowModelSettings(true);
    };

    // Cleanup on unmount
    return () => {
      delete (window as any).openSettings;
    };
  }, []);

  const renderCollapsedIcons = () => {
    if (!isCollapsed) return null;

    const isHomePage = pathname === '/';
    const isMeetingPage = pathname?.includes('/meeting-details');
    const isSettingsPage = pathname === '/settings';

    return (
      <TooltipProvider>
        <div className="flex flex-col items-center space-y-4 mt-4">
          <Logo isCollapsed={isCollapsed} />

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => router.push('/')}
                className={`${isHomePage ? 'bg-[var(--muted)]' : ''}`}
              >
                <Home className="w-5 h-5 text-[var(--muted-foreground)]" />
              </Button>
            </TooltipTrigger>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="destructive"
                size="icon"
                onClick={handleRecordingToggle}
                disabled={isRecording}
                className="rounded-full"
              >
                {isRecording ? (
                  <Square className="w-5 h-5 text-[var(--destructive-foreground)]" />
                ) : (
                  <Mic className="w-5 h-5 text-[var(--destructive-foreground)]" />
                )}
              </Button>
            </TooltipTrigger>
            <TooltipContent side="right">
              <p>{isRecording ? "Recording in progress..." : "Start Recording"}</p>
            </TooltipContent>
          </Tooltip>

          {betaFeatures.importAndRetranscribe && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  variant="secondary"
                  size="icon"
                  onClick={() => openImportDialog()}
                >
                  <Upload className="w-5 h-5 text-[var(--accent)]" />
                </Button>
              </TooltipTrigger>
              <TooltipContent side="right">
                <p>Import Audio</p>
              </TooltipContent>
            </Tooltip>
          )}

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => {
                  if (isCollapsed) toggleCollapse();
                  toggleFolder('meetings');
                }}
                className={`${isMeetingPage ? 'bg-[var(--muted)]' : ''}`}
              >
                <NotebookPen className="w-5 h-5 text-[var(--muted-foreground)]" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="right">
              <p>Meeting Notes</p>
            </TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                variant="ghost"
                size="icon"
                onClick={() => router.push('/settings')}
                className={`${isSettingsPage ? 'bg-[var(--muted)]' : ''}`}
              >
                <Settings className="w-5 h-5 text-[var(--muted-foreground)]" />
              </Button>
            </TooltipTrigger>
            <TooltipContent side="right">
              <p>Settings</p>
            </TooltipContent>
          </Tooltip>

          <Info isCollapsed={isCollapsed} />
        </div>
      </TooltipProvider>
    );
  };

  // Find matching transcript snippet for a meeting item (uses FTS5)
  const findMatchingSnippet = (itemId: string) => {
    if (!searchQuery.trim() || !ftsSearchResults.length) return null;
    return ftsSearchResults.find(result => result.meeting_id === itemId);
  };

  const renderItem = (item: SidebarItem, depth = 0) => {
    const isExpanded = expandedFolders.has(item.id);
    const paddingLeft = `${depth * 12 + 12}px`;
    const isActive = item.type === 'file' && currentMeeting?.id === item.id;
    const isMeetingItem = item.id.includes('-') && !item.id.startsWith('intro-call');

    // Check if this item has a matching transcript snippet
    const matchingResult = isMeetingItem ? findMatchingSnippet(item.id) : null;
    const hasTranscriptMatch = !!matchingResult;

    if (isCollapsed) return null;

    return (
      <div key={item.id}>
        <div
          className={`flex items-center transition-all duration-150 group min-w-0 ${item.type === 'folder' && depth === 0
            ? 'p-3 text-lg font-semibold h-10 mx-3 mt-3 rounded-lg'
            : `px-3 py-2 my-0.5 rounded-md text-sm ${isActive ? 'bg-blue-100 text-blue-700 font-medium' :
              hasTranscriptMatch ? 'bg-yellow-50' : 'hover:bg-gray-50'
            } cursor-pointer`
            }`}
          style={item.type === 'folder' && depth === 0 ? {} : { paddingLeft }}
          onClick={() => {
            if (item.type === 'folder') {
              toggleFolder(item.id);
            } else {
              setCurrentMeeting({ id: item.id, title: item.title });
              const basePath = item.id.startsWith('intro-call') ? '/' :
                item.id.includes('-') ? `/meeting-details?id=${item.id}` : `/notes/${item.id}`;
              router.push(basePath);
            }
          }}
        >
          {item.type === 'folder' ? (
            <>
              {item.id === 'meetings' ? (
                <Calendar className="w-4 h-4 mr-2" />
              ) : item.id === 'notes' ? (
                <Calendar className="w-4 h-4 mr-2" />
              ) : null}
              <span className={depth === 0 ? "" : "font-medium"}>{item.title}</span>
              <div className="ml-auto">
                {isExpanded ? (
                  <ChevronDown className="w-4 h-4 text-[var(--muted-foreground)]" />
                ) : (
                  <ChevronRight className="w-4 h-4 text-[var(--muted-foreground)]" />
                )}
              </div>
              {searchQuery && item.id === 'meetings' && isSearching && (
                <span className="ml-2 text-xs text-[var(--accent)] animate-pulse">Searching...</span>
              )}
            </>
          ) : (
            <div className="flex flex-col w-full">
              <div className="flex items-center w-full">
                {isMeetingItem ? (
                  <div className="flex-shrink-0 flex items-center justify-center w-6 h-6 rounded-full mr-2 bg-[var(--muted)]">
                    <File className="w-3.5 h-3.5 text-[var(--muted-foreground)]" />
                  </div>
                ) : (
                  <div className="flex-shrink-0 flex items-center justify-center w-6 h-6 rounded-full mr-2 bg-[var(--accent)]">
                    <Plus className="w-3.5 h-3.5 text-[var(--accent-foreground)]" />
                  </div>
                )}
                <span className="flex-1 truncate">{item.title}</span>
                {isMeetingItem && (
                  <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-150">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleEditStart(item.id, item.title);
                      }}
                      className="text-[var(--accent)] hover:text-[var(--accent)] hover:bg-[var(--muted)] p-1 flex-shrink-0"
                      aria-label="Edit meeting title"
                    >
                      <Pencil className="w-4 h-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={(e) => {
                        e.stopPropagation();
                        setDeleteModalState({ isOpen: true, itemId: item.id });
                      }}
                      className="text-[var(--destructive)] hover:text-[var(--destructive)] hover:bg-[var(--muted)] p-1 flex-shrink-0"
                      aria-label="Delete meeting"
                    >
                      <Trash2 className="w-4 h-4" />
                    </Button>
                  </div>
                )}
              </div>
              {/* Show transcript match snippet if available */}
              {hasTranscriptMatch && (
                <div className="mt-1 ml-8 text-xs text-gray-500 bg-yellow-50 p-1.5 rounded border border-yellow-100 line-clamp-2">
                  <span className="font-medium text-yellow-600">Match:</span> {matchingResult?.snippet}
                </div>
              )}
            </div>
          )}
        </div>
        {item.type === 'folder' && isExpanded && item.children && (
          <div className="ml-1">
            {item.children.map(child => renderItem(child, depth + 1))}
          </div>
        )}
      </div>
    );
  };

  return (
    <div className="fixed top-0 left-0 h-screen z-40">
      {/* Floating collapse button */}
      <Button
        variant="outline"
        size="icon"
        onClick={toggleCollapse}
        className="absolute -right-6 top-20 z-50 rounded-full shadow-lg"
        style={{ transform: 'translateX(50%)' }}
      >
        {isCollapsed ? (
          <ChevronRightCircle className="w-6 h-6" />
        ) : (
          <ChevronLeftCircle className="w-6 h-6" />
        )}
      </Button>

      <div
        className={`h-full min-h-0 bg-white border-r shadow-sm flex flex-col transition-all duration-300 overflow-hidden ${isCollapsed ? 'w-16' : 'w-64'
          }`}
      >
        {/*  Header with traffic light spacing */}
        <div className="flex-shrink-0 h-[88px] flex items-center">

          {/* Title container */}



          <div className="flex-1">
            {!isCollapsed && (
              <div className="p-3">
                {/* <span className="text-lg text-center border rounded-full bg-blue-50 border-white font-semibold text-gray-700 mb-2 block items-center">
                  <span>Twin</span>
                </span> */}
                <Logo isCollapsed={isCollapsed} />

                <div className="relative mb-1">
                  <InputGroup >
                    <InputGroupInput placeholder='Search meeting content...' value={searchQuery}
                      onChange={(e) => handleSearchChange(e.target.value)}
                    />
                    <InputGroupAddon>
                      <Popover open={showSearchFilters} onOpenChange={setShowSearchFilters}>
                        <PopoverTrigger asChild>
                          <Button
                            variant="ghost"
                            size="sm"
                            className={`p-1 rounded transition-colors ${activeFilterCount > 0 ? 'text-[var(--accent)] bg-[var(--muted)]' : 'text-[var(--muted-foreground)] hover:text-[var(--foreground)]'}`}
                            title="Search filters"
                          >
                            <Filter className="h-4 w-4" />
                            {activeFilterCount > 0 && (
                              <span className="absolute -top-1 -right-1 bg-[var(--accent)] text-[var(--accent-foreground)] text-xs rounded-full w-4 h-4 flex items-center justify-center">
                                {activeFilterCount}
                              </span>
                            )}
                          </Button>
                        </PopoverTrigger>
                        <PopoverContent className="w-64" align="start">
                          <div className="space-y-4">
                            <h4 className="font-medium text-sm">Search Filters</h4>

                            {/* Date range */}
                            <div className="space-y-2">
                              <Label className="text-xs text-[var(--muted-foreground)]">Date Range</Label>
                              <div className="grid grid-cols-2 gap-2">
                                <Input
                                  type="date"
                                  placeholder="From"
                                  value={searchFilters.date_from || ''}
                                  onChange={(e) => setSearchFilters({ ...searchFilters, date_from: e.target.value || null })}
                                  className="text-xs"
                                />
                                <Input
                                  type="date"
                                  placeholder="To"
                                  value={searchFilters.date_to || ''}
                                  onChange={(e) => setSearchFilters({ ...searchFilters, date_to: e.target.value || null })}
                                  className="text-xs"
                                />
                              </div>
                            </div>

                              <Label className="text-xs text-[var(--muted-foreground)]">Min Duration (minutes)</Label>
                            <div className="space-y-2">
                              <Input
                                type="number"
                                placeholder="e.g. 5"
                                value={searchFilters.min_duration || ''}
                                onChange={(e) => setSearchFilters({ ...searchFilters, min_duration: e.target.value ? parseFloat(e.target.value) : null })}
                                className="text-xs rounded border-[var(--border)]"
                              />
                            </div>

                            {/* Has summary */}
                            <div className="flex items-center gap-2">
                              <input
                                type="checkbox"
                                checked={searchFilters.has_summary === true}
                                onChange={(e) => setSearchFilters({ ...searchFilters, has_summary: e.target.checked ? true : null })}
                                className="rounded border-[var(--border)]"
                              />
                              <Label className="text-xs">Has summary</Label>
                            </div>

                            {/* Clear filters */}
                            {activeFilterCount > 0 && (
                              <Button
                                variant="link"
                                size="sm"
                                onClick={() => {
                                  clearSearchFilters();
                                  setShowSearchFilters(false);
                                  // Re-search with cleared filters if there's a query
                                  if (searchQuery.trim()) {
                                    searchMeetingsFts(searchQuery, {
                                      date_from: null,
                                      date_to: null,
                                      min_duration: null,
                                      has_summary: null,
                                    });
                                  }
                                }}
                                className="text-xs text-[var(--muted-foreground)] hover:text-[var(--foreground)] p-0 h-auto"
                              >
                                Clear all filters
                              </Button>
                            )}
                          </div>
                        </PopoverContent>
                      </Popover>
                    </InputGroupAddon>
                    {searchQuery &&
                      <InputGroupAddon align={'inline-end'}>
                        <InputGroupButton
                          onClick={() => {
                            handleSearchChange('');
                            clearSearchFilters();
                          }}
                        >
                          <X />
                        </InputGroupButton>
                      </InputGroupAddon>
                    }
                  </InputGroup>

                  {/* Active filter chips */}
                  {activeFilterCount > 0 && (
                    <div className="flex gap-1 mt-2 flex-wrap">
                      {searchFilters.date_from && (
                        <Badge variant="secondary" className="text-xs gap-1">
                          From: {searchFilters.date_from}
                          <X
                            className="h-3 w-3 cursor-pointer"
                            onClick={() => {
                              setSearchFilters({ ...searchFilters, date_from: null });
                              if (searchQuery.trim()) {
                                searchMeetingsFts(searchQuery, { ...searchFilters, date_from: null });
                              }
                            }}
                          />
                        </Badge>
                      )}
                      {searchFilters.date_to && (
                        <Badge variant="secondary" className="text-xs gap-1">
                          To: {searchFilters.date_to}
                          <X
                            className="h-3 w-3 cursor-pointer"
                            onClick={() => {
                              setSearchFilters({ ...searchFilters, date_to: null });
                              if (searchQuery.trim()) {
                                searchMeetingsFts(searchQuery, { ...searchFilters, date_to: null });
                              }
                            }}
                          />
                        </Badge>
                      )}
                      {searchFilters.min_duration && (
                        <Badge variant="secondary" className="text-xs gap-1">
                          {searchFilters.min_duration}+ min
                          <X
                            className="h-3 w-3 cursor-pointer"
                            onClick={() => {
                              setSearchFilters({ ...searchFilters, min_duration: null });
                              if (searchQuery.trim()) {
                                searchMeetingsFts(searchQuery, { ...searchFilters, min_duration: null });
                              }
                            }}
                          />
                        </Badge>
                      )}
                      {searchFilters.has_summary === true && (
                        <Badge variant="secondary" className="text-xs gap-1">
                          Has summary
                          <X
                            className="h-3 w-3 cursor-pointer"
                            onClick={() => {
                              setSearchFilters({ ...searchFilters, has_summary: null });
                              if (searchQuery.trim()) {
                                searchMeetingsFts(searchQuery, { ...searchFilters, has_summary: null });
                              }
                            }}
                          />
                        </Badge>
                      )}
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Main content - scrollable area */}
        <div className="flex-1 flex flex-col min-h-0">
          {/* Fixed navigation items */}
          <div className="flex-shrink-0">
            {!isCollapsed && (
              <div
                onClick={() => router.push('/')}
                className="p-3 text-lg font-semibold items-center hover:bg-[var(--muted)] h-10 flex mx-3 mt-3 rounded-lg cursor-pointer"
              >
                <Home className="w-4 h-4 mr-2" />
                <span>Home</span>
              </div>
            )}
          </div>

            {!isCollapsed && (
              <div className="flex-shrink-0">
                {filteredSidebarItems.filter(item => item.type === 'folder').map(item => (
                  <div key={item.id}>
                    <div
                      className="flex items-center transition-all duration-150 p-3 text-lg font-semibold h-10 mx-3 mt-3 rounded-lg"
                    >
                      <NotebookPen className="w-4 h-4 mr-2 text-gray-600" />
                      <span className="text-gray-700 truncate">{item.title}</span>
                      {searchQuery && item.id === 'meetings' && isSearching && (
                        <span className="ml-2 text-xs text-blue-500 animate-pulse">Searching...</span>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}

            {/* Scrollable meeting items */}
            {!isCollapsed && (
              <div className="flex-1 overflow-y-auto custom-scrollbar min-h-0">
                {filteredSidebarItems
                  .filter(item => item.type === 'folder' && expandedFolders.has(item.id) && item.children)
                  .map(item => renderItem(item, 1))}
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        {!isCollapsed && (

          <div className="flex-shrink-0 p-2 border-t border-[var(--border)]">
            <Button
              variant="destructive"
              onClick={handleRecordingToggle}
              disabled={isRecording}
              className="w-full"
            >
              {isRecording ? (
                <>
                  <Square className="w-4 h-4 mr-2" />
                  <span>Recording in progress...</span>
                </>
              ) : (
                <>
                  <Mic className="w-4 h-4 mr-2" />
                  <span>Start Recording</span>
                </>
              )}
            </Button>

            {betaFeatures.importAndRetranscribe && (
              <Button
                variant="secondary"
                onClick={() => openImportDialog()}
                className="w-full mt-1"
              >
                <Upload className="w-4 h-4 mr-2" />
                <span>Import Audio</span>
              </Button>
            )}

            {/* Action Items button */}
            <Button
              variant="secondary"
              onClick={() => router.push('/action-items')}
              className="w-full mt-1"
            >
              <ClipboardList className="w-4 h-4 mr-2" />
              <span>Action Items</span>
            </Button>

            {/* Model status badge - show when models are not ready */}
            {(!isParakeetReady || !isSummaryReady) && (
              <Button
                variant="outline"
                onClick={() => startDownload()}
                className="w-full mt-1 mb-1"
              >
                <Download className="w-3 h-3 mr-1.5" />
                <span>Setup models</span>
              </Button>
            )}

            <Button
              variant="secondary"
              onClick={() => router.push('/settings')}
              className="w-full mt-1 mb-1"
            >
              <Settings className="w-4 h-4 mr-2" />
              <span>Settings</span>
            </Button>
            <Info isCollapsed={isCollapsed} />
            <div className="w-full flex items-center justify-center px-3 py-1 text-xs text-[var(--muted-foreground)]">
              v0.4.0
            </div>
          </div>
        )}
      {/* Confirmation Modal for Delete */}
      <ConfirmationModal
        isOpen={deleteModalState.isOpen}
        text="Are you sure you want to delete this meeting? This action cannot be undone."
        onConfirm={handleDeleteConfirm}
        onCancel={() => setDeleteModalState({ isOpen: false, itemId: null })}
      />

      {/* Edit Meeting Title Modal */}
      <Dialog open={editModalState.isOpen} onOpenChange={(open) => {
        if (!open) handleEditCancel();
      }}>
        <DialogContent className="sm:max-w-[425px]">
          <VisuallyHidden>
            <DialogTitle>Edit Meeting Title</DialogTitle>
          </VisuallyHidden>
          <div className="py-4">
            <div className="space-y-4">
              <div>
                <label htmlFor="meeting-title" className="block text-sm font-medium text-[var(--foreground)] mb-2">
                  Meeting Title
                </label>
                <input
                  id="meeting-title"
                  type="text"
                  value={editingTitle}
                  onChange={(e) => setEditingTitle(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') {
                      handleEditConfirm();
                    } else if (e.key === 'Escape') {
                      handleEditCancel();
                    }
                  }}
                  className="w-full px-3 py-2 border border-[var(--border)] rounded-md focus:outline-none focus:ring-2 focus:ring-[var(--ring)] focus:border-transparent"
                  placeholder="Enter meeting title"
                  autoFocus
                />
              </div>
            </div>
          </div>
          <DialogFooter>
            <Button
              variant="secondary"
              onClick={handleEditCancel}
            >
              Cancel
            </Button>
            <Button
              onClick={handleEditConfirm}
            >
              Save
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default Sidebar;

