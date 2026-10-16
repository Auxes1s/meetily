"use client"

import { useEffect, useState, useRef } from "react"
import { Switch } from "./ui/switch"
import { FolderOpen } from "lucide-react"
import { invoke } from "@tauri-apps/api/core"
import Analytics from "@/lib/analytics"
import { toast } from "sonner"
import AnalyticsConsentSwitch from "./AnalyticsConsentSwitch"
import { useConfig, NotificationSettings } from "@/contexts/ConfigContext"

export function PreferenceSettings() {
  const {
    notificationSettings,
    storageLocations,
    isLoadingPreferences,
    loadPreferences,
    updateNotificationSettings
  } = useConfig();

  const [notificationsEnabled, setNotificationsEnabled] = useState<boolean | null>(null);
  const [savingNotifications, setSavingNotifications] = useState(false);
  const notificationWritePending = useRef(false);
  const hasTrackedViewRef = useRef(false);

  // Lazy load preferences on mount (only loads if not already cached)
  useEffect(() => {
    loadPreferences();
    // Reset tracking ref on mount (every tab visit)
    hasTrackedViewRef.current = false;
  }, [loadPreferences]);

  // Track preferences viewed analytics on every tab visit (once per mount)
  useEffect(() => {
    if (hasTrackedViewRef.current) return;

    const trackPreferencesViewed = async () => {
      // Wait for notification settings to be available (either from cache or after loading)
      if (notificationSettings) {
        await Analytics.track('preferences_viewed', {
          notifications_enabled: notificationSettings.notification_preferences.show_recording_started ? 'true' : 'false'
        });
        hasTrackedViewRef.current = true;
      } else if (!isLoadingPreferences) {
        // If not loading and no settings available, track with default value
        await Analytics.track('preferences_viewed', {
          notifications_enabled: 'false'
        });
        hasTrackedViewRef.current = true;
      }
    };

    trackPreferencesViewed();
  }, [notificationSettings, isLoadingPreferences]);

  // External/cache refreshes cannot replace an in-flight user choice.
  useEffect(() => {
    if (notificationWritePending.current) return;
    if (notificationSettings) {
      setNotificationsEnabled(
        notificationSettings.notification_preferences.show_recording_started &&
        notificationSettings.notification_preferences.show_recording_stopped
      );
    } else if (!isLoadingPreferences) setNotificationsEnabled(true);
  }, [notificationSettings, isLoadingPreferences]);

  const handleNotificationToggle = async (enabled: boolean) => {
    if (notificationWritePending.current || !notificationSettings) return;
    notificationWritePending.current = true;
    const previous = notificationsEnabled;
    setSavingNotifications(true);
    setNotificationsEnabled(enabled);
    try {
      const updatedSettings: NotificationSettings = {
        ...notificationSettings,
        notification_preferences: {
          ...notificationSettings.notification_preferences,
          show_recording_started: enabled,
          show_recording_stopped: enabled,
        }
      };
      await updateNotificationSettings(updatedSettings);
      void Analytics.track('notification_settings_changed', {
        notifications_enabled: enabled.toString()
      }).catch(error => console.error('Failed to track notification preference:', error));
    } catch (error) {
      setNotificationsEnabled(previous);
      toast.error('Failed to save notification preference');
    } finally {
      notificationWritePending.current = false;
      setSavingNotifications(false);
    }
  };

  const handleOpenFolder = async (folderType: 'database' | 'models' | 'recordings') => {
    try {
      switch (folderType) {
        case 'database':
          await invoke('open_database_folder');
          break;
        case 'models':
          await invoke('open_models_folder');
          break;
        case 'recordings':
          await invoke('open_recordings_folder');
          break;
      }

      // Track storage folder access
      await Analytics.track('storage_folder_opened', {
        folder_type: folderType
      });
    } catch (error) {
      console.error(`Failed to open ${folderType} folder:`, error);
    }
  };

  // Show loading only if we're actually loading and don't have cached data
  if (isLoadingPreferences && !notificationSettings && !storageLocations) {
    return <div className="max-w-2xl mx-auto p-6">Loading Preferences...</div>
  }

  // Show loading if notificationsEnabled hasn't been determined yet
  if (notificationsEnabled === null && !isLoadingPreferences) {
    return <div className="max-w-2xl mx-auto p-6">Loading Preferences...</div>
  }

  // Ensure we have a boolean value for the Switch component
  const notificationsEnabledValue = notificationsEnabled ?? false;

  return (
    <div className="space-y-6">
      {/* Notifications Section */}
      <div className="paper-surface rounded-2xl border border-border p-5 sm:p-6">
        <div className="flex items-start justify-between gap-4">
          <div className="min-w-0">
            <h3 className="text-lg font-semibold text-foreground mb-2">Notifications</h3>
            <p className="text-sm text-muted-foreground">Enable or disable notifications of start and end of meeting</p>
          </div>
          <Switch aria-label="Meeting notifications" checked={notificationsEnabledValue} disabled={savingNotifications || !notificationSettings} onCheckedChange={handleNotificationToggle} />
          {savingNotifications && <span role="status" className="text-xs text-muted-foreground">Saving…</span>}
        </div>
      </div>

      {/* Data Storage Locations Section */}
      <div className="paper-surface rounded-2xl border border-border p-5 sm:p-6">
        <h3 className="text-lg font-semibold text-foreground mb-4">Data Storage Locations</h3>
        <p className="text-sm text-muted-foreground mb-6">
          View and access where Meetily stores your data
        </p>

        <div className="space-y-4">
          {/* Database Location */}
          {/* <div className="p-4 border rounded-lg bg-muted/50">
            <div className="font-medium mb-2">Database</div>
            <div className="text-sm text-muted-foreground mb-3 break-all font-mono text-xs">
              {storageLocations?.database || 'Loading...'}
            </div>
            <button
              onClick={() => handleOpenFolder('database')}
              className="flex items-center gap-2 px-3 py-2 text-sm border border-input rounded-md hover:bg-muted transition-colors"
            >
              <FolderOpen className="w-4 h-4" />
              Open Folder
            </button>
          </div> */}

          {/* Models Location */}
          {/* <div className="p-4 border rounded-lg bg-muted/50">
            <div className="font-medium mb-2">Whisper Models</div>
            <div className="text-sm text-muted-foreground mb-3 break-all font-mono text-xs">
              {storageLocations?.models || 'Loading...'}
            </div>
            <button
              onClick={() => handleOpenFolder('models')}
              className="flex items-center gap-2 px-3 py-2 text-sm border border-input rounded-md hover:bg-muted transition-colors"
            >
              <FolderOpen className="w-4 h-4" />
              Open Folder
            </button>
          </div> */}

          {/* Recordings Location */}
          <div className="p-4 border rounded-lg bg-muted/50">
            <div className="font-medium mb-2">Meeting Recordings</div>
            <div className="text-sm text-muted-foreground mb-3 break-all font-mono text-xs">
              {storageLocations?.recordings || 'Loading...'}
            </div>
            <button
              onClick={() => handleOpenFolder('recordings')}
              className="flex items-center gap-2 px-3 py-2 text-sm border border-input rounded-md hover:bg-muted transition-colors"
            >
              <FolderOpen className="w-4 h-4" />
              Open Folder
            </button>
          </div>
        </div>

        <div className="mt-4 p-3 bg-primary/5 rounded-md">
          <p className="text-xs text-primary">
            <strong>Note:</strong> Database and models are stored together in your application data directory for unified management.
          </p>
        </div>
      </div>

      {/* Analytics Section */}
      <div className="paper-surface rounded-2xl border border-border p-5 sm:p-6">
        <AnalyticsConsentSwitch />
      </div>
    </div>
  )
}
