import { ModelConfig } from "@/components/ModelSettingsModal";
import { PreferenceSettings } from "@/components/PreferenceSettings";
import { DeviceSelection } from "@/components/DeviceSelection";
import { LanguageSelection } from "@/components/LanguageSelection";
import { TranscriptSettings } from "@/components/TranscriptSettings";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { toast } from "sonner";
import { useConfig } from "@/contexts/ConfigContext";
import { useRecordingState } from "@/contexts/RecordingStateContext";

type modalType = "modelSettings" | "deviceSettings" | "languageSettings" | "modelSelector" | "errorAlert" | "chunkDropWarning";

/**
 * SettingsModals Component
 *
 * All settings modals consolidated into a single component.
 * Uses ConfigContext and RecordingStateContext internally - no prop drilling needed!
 */

interface SettingsModalsProps {
  modals: {
    modelSettings: boolean;
    deviceSettings: boolean;
    languageSettings: boolean;
    modelSelector: boolean;
    errorAlert: boolean;
    chunkDropWarning: boolean;
  };
  messages: {
    errorAlert: string;
    chunkDropWarning: string;
    modelSelector: string;
  };
  onClose: (name: modalType) => void;
}

export function SettingsModals({
  modals,
  messages,
  onClose,
}: SettingsModalsProps) {
  // Contexts
  const {
    modelConfig,
    setModelConfig,
    models,
    modelOptions,
    error,
    selectedDevices,
    setSelectedDevices,
    selectedLanguage,
    setSelectedLanguage,
    transcriptModelConfig,
    setTranscriptModelConfig,
    showConfidenceIndicator,
    toggleConfidenceIndicator,
  } = useConfig();

  const { isRecording } = useRecordingState();

  return <>
    {/* Legacy Settings Modal */}
    {modals.modelSettings && (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
        <div className="bg-[var(--background)] rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-hidden flex flex-col">
          {/* Header */}
          <div className="flex justify-between items-center p-6 border-b border-[var(--border)]">
            <h3 className="text-xl font-semibold text-[var(--foreground)]">Preferences</h3>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => onClose("modelSettings")}
              className="h-6 w-6 text-[var(--muted-foreground)] hover:text-[var(--foreground)]"
            >
              <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </Button>
          </div>

          {/* Content - Scrollable */}
          <div className="flex-1 overflow-y-auto p-6 space-y-8">
            {/* General Preferences Section */}
            <PreferenceSettings />

            {/* Divider */}
            <div className="border-t border-[var(--border)] pt-8">
              <h4 className="text-lg font-semibold text-[var(--foreground)] mb-4">AI Model Configuration</h4>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium text-[var(--foreground)] mb-1">
                    Summarization Model
                  </label>
                  <div className="flex space-x-2">
                    <Select
                      value={modelConfig.provider}
                      onValueChange={(provider) => {
                        const p = provider as ModelConfig['provider'];
                        setModelConfig({
                          ...modelConfig,
                          provider: p,
                          model: modelOptions[p][0]
                        });
                      }}
                    >
                      <SelectTrigger className="w-[180px]">
                        <SelectValue placeholder="Select provider" />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="builtin-ai">Built-in AI</SelectItem>
                        <SelectItem value="claude">Claude</SelectItem>
                        <SelectItem value="ollama">Ollama</SelectItem>
                        <SelectItem value="openai">OpenAI</SelectItem>
                      </SelectContent>
                    </Select>

                    <Select
                      value={modelConfig.model}
                      onValueChange={(model) => setModelConfig((prev: ModelConfig) => ({ ...prev, model }))}
                    >
                      <SelectTrigger className="flex-1">
                        <SelectValue placeholder="Select model" />
                      </SelectTrigger>
                      <SelectContent>
                        {modelOptions[modelConfig.provider].map((model: string) => (
                          <SelectItem key={model} value={model}>
                            {model}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                </div>
                {modelConfig.provider === 'ollama' && (
                  <div>
                    <h4 className="text-lg font-bold mb-4 text-[var(--foreground)]">Available Ollama Models</h4>
                    {error && (
                      <div className="bg-[var(--destructive)]/10 border border-[var(--destructive)]/30 text-[var(--destructive)] px-4 py-3 rounded mb-4">
                        {error}
                      </div>
                    )}
                    <div className="grid gap-4 max-h-[400px] overflow-y-auto pr-2">
                      {models.map((model) => (
                        <div
                          key={model.id}
                          className={`bg-[var(--card)] p-4 rounded-lg shadow cursor-pointer transition-colors ${modelConfig.model === model.name ? 'ring-2 ring-[var(--accent)] bg-[var(--accent)]/10' : 'hover:bg-[var(--accent)]/5'
                            }`}
                          onClick={() => setModelConfig((prev: ModelConfig) => ({ ...prev, model: model.name }))}
                        >
                          <h3 className="font-bold text-[var(--foreground)]">{model.name}</h3>
                          <p className="text-[var(--muted-foreground)]">Size: {model.size}</p>
                          <p className="text-[var(--muted-foreground)]">Modified: {model.modified}</p>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* Footer */}
          <div className="border-t border-[var(--border)] p-6 flex justify-end">
            <Button
              variant="default"
              onClick={() => onClose('modelSettings')}
            >
              Done
            </Button>
          </div>
        </div>
      </div>
    )}

    {/* Device Settings Modal */}
    {modals.deviceSettings && (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <div className="bg-[var(--background)] rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-lg font-semibold text-[var(--foreground)]">Audio Device Settings</h3>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => onClose('deviceSettings')}
              className="h-6 w-6 text-[var(--muted-foreground)] hover:text-[var(--foreground)]"
            >
              <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </Button>
          </div>

          <DeviceSelection
            selectedDevices={selectedDevices}
            onDeviceChange={setSelectedDevices}
            disabled={isRecording}
          />

          <div className="mt-6 flex justify-end">
            <Button
              variant="default"
              onClick={() => {
                const micDevice = selectedDevices.micDevice || 'Default';
                const systemDevice = selectedDevices.systemDevice || 'Default';
                toast.success("Devices selected", {
                  description: `Microphone: ${micDevice}, System Audio: ${systemDevice}`
                });
                onClose('deviceSettings');
              }}
            >
              Done
            </Button>
          </div>
        </div>
      </div>
    )}

    {/* Language Settings Modal */}
    {modals.languageSettings && (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <div className="bg-[var(--background)] rounded-lg p-6 max-w-md w-full mx-4 shadow-xl">
          <div className="flex justify-between items-center mb-4">
            <h3 className="text-lg font-semibold text-[var(--foreground)]">Language Settings</h3>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => onClose('languageSettings')}
              className="h-6 w-6 text-[var(--muted-foreground)] hover:text-[var(--foreground)]"
            >
              <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </Button>
          </div>

          <LanguageSelection
            selectedLanguage={selectedLanguage}
            onLanguageChange={setSelectedLanguage}
            disabled={isRecording}
            provider={transcriptModelConfig.provider}
          />

          <div className="mt-6 flex justify-end">
            <Button
              variant="default"
              onClick={() => onClose('languageSettings')}
            >
              Done
            </Button>
          </div>
        </div>
      </div>
    )}

    {/* Model Selection Modal */}
    {modals.modelSelector && (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <div className="bg-[var(--background)] rounded-lg max-w-4xl w-full mx-4 shadow-xl max-h-[90vh] flex flex-col">
          {/* Fixed Header */}
          <div className="flex justify-between items-center p-6 pb-4 border-b border-[var(--border)]">
            <h3 className="text-lg font-semibold text-[var(--foreground)]">
              {messages.modelSelector ? 'Speech Recognition Setup Required' : 'Transcription Model Settings'}
            </h3>
            <Button
              variant="ghost"
              size="icon"
              onClick={() => onClose('modelSelector')}
              className="h-6 w-6 text-[var(--muted-foreground)] hover:text-[var(--foreground)]"
            >
              <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </Button>
          </div>

          {/* Scrollable Content */}
          <div className="flex-1 overflow-y-auto p-6 pt-4">
            <TranscriptSettings
              transcriptModelConfig={transcriptModelConfig}
              setTranscriptModelConfig={setTranscriptModelConfig}
              onModelSelect={() => onClose('modelSelector')}
            />
          </div>

          {/* Fixed Footer */}
          <div className="p-6 pt-4 border-t border-[var(--border)] flex items-center justify-between">
            {/* Confidence Indicator Toggle */}
            <div className="flex items-center gap-3">
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={showConfidenceIndicator}
                  onChange={(e) => toggleConfidenceIndicator(e.target.checked)}
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-[var(--muted)] peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-[var(--accent)]/30 rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-[var(--background)] after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-[var(--background)] after:border-[var(--border)] after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-[var(--accent)]"></div>
              </label>
              <div>
                <p className="text-sm font-medium text-[var(--foreground)]">Show Confidence Indicators</p>
                <p className="text-xs text-[var(--muted-foreground)]">Display colored dots showing transcription confidence quality</p>
              </div>
            </div>

            <Button
              variant="secondary"
              onClick={() => onClose('modelSelector')}
            >
              {messages.modelSelector ? 'Cancel' : 'Done'}
            </Button>
          </div>
        </div>
      </div>
    )}

    {/* Error Alert Modal */}
    {modals.errorAlert && (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <Alert className="max-w-md mx-4 border-[var(--destructive)]/30 bg-[var(--background)] shadow-xl">
          <AlertTitle className="text-[var(--destructive)]">Recording Stopped</AlertTitle>
          <AlertDescription className="text-[var(--destructive)]">
            {messages.errorAlert}
            <Button
              variant="link"
              size="sm"
              onClick={() => onClose('errorAlert')}
              className="ml-2 text-[var(--destructive)] underline p-0 h-auto"
            >
              Dismiss
            </Button>
          </AlertDescription>
        </Alert>
      </div>
    )}

    {/* Chunk Drop Warning Modal */}
    {modals.chunkDropWarning && (
      <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
        <Alert className="max-w-lg mx-4 border-yellow-500/30 bg-[var(--background)] shadow-xl">
          <AlertTitle className="text-yellow-700 dark:text-yellow-500">Transcription Performance Warning</AlertTitle>
          <AlertDescription className="text-yellow-600 dark:text-yellow-400">
            {messages.chunkDropWarning}
            <Button
              variant="link"
              size="sm"
              onClick={() => onClose('chunkDropWarning')}
              className="ml-2 text-yellow-600 dark:text-yellow-400 underline p-0 h-auto"
            >
              Dismiss
            </Button>
          </AlertDescription>
        </Alert>
      </div>
    )}
  </>
}
