"use client"

import { Switch } from "./ui/switch"
import { FlaskConical, AlertCircle } from "lucide-react"
import { useConfig } from "@/contexts/ConfigContext"
import { useContinuousTranscription } from "@/hooks/useContinuousTranscription"
import {
  BetaFeatureKey,
  BETA_FEATURE_NAMES,
  BETA_FEATURE_DESCRIPTIONS
} from "@/types/betaFeatures"

export function BetaSettings() {
  const { betaFeatures, toggleBetaFeature } = useConfig();
  const continuous = useContinuousTranscription();

  // Define feature order for display (allows custom ordering)
  const featureOrder: BetaFeatureKey[] = ['importAndRetranscribe'];

  return (
    <div className="space-y-6">
      {/* Yellow Warning Banner */}
      <div className="flex items-start gap-3 p-4 bg-yellow-50 border border-yellow-200 rounded-lg">
        <AlertCircle className="h-5 w-5 text-yellow-600 flex-shrink-0 mt-0.5" />
        <div className="text-sm text-yellow-800">
          <p className="font-medium">Beta Features</p>
          <p className="mt-1">
            These features are still being tested. You may encounter issues, and we appreciate your feedback.
          </p>
        </div>
      </div>

      <div className="paper-surface rounded-2xl border border-border p-5 sm:p-6">
        <div className="flex items-start justify-between gap-4">
          <div className="min-w-0 flex-1">
            <div className="flex flex-wrap items-center gap-2 mb-2">
              <FlaskConical className="h-5 w-5 text-muted-foreground" />
              <h3 id="continuous-transcription-label" className="text-lg font-semibold text-foreground">
                Continuous Whisper Transcription
              </h3>
              <span className="px-2 py-0.5 text-xs font-medium bg-yellow-100 text-yellow-800 rounded-full">
                BETA
              </span>
            </div>
            <p id="continuous-transcription-description" className="text-sm text-muted-foreground">
              Show and revise live text as you speak with local Whisper. Enabled by default.
              Turning this off uses the previous transcription mode. Changes apply to your next recording.
            </p>
          </div>
          <Switch
            aria-labelledby="continuous-transcription-label"
            aria-describedby="continuous-transcription-description"
            checked={continuous.enabled}
            disabled={continuous.loading || continuous.loadFailed}
            onCheckedChange={(checked) => { void continuous.update(checked); }}
          />
        </div>
        {continuous.error && (
          <div className="mt-3 text-sm text-red-700" role="alert">
            <p>{continuous.error}</p>
            <button type="button" className="mt-1 underline" disabled={continuous.loading} onClick={() => { void continuous.reload(); }}>
              Reload setting
            </button>
          </div>
        )}
      </div>

      {/* Dynamic Feature Toggles - Automatically renders all features */}
      {featureOrder.map((featureKey) => (
        <div
          key={featureKey}
          className="paper-surface rounded-2xl border border-border p-5 sm:p-6"
        >
          <div className="flex items-start justify-between gap-4">
            <div className="min-w-0 flex-1">
              <div className="flex flex-wrap items-center gap-2 mb-2">
                <FlaskConical className="h-5 w-5 text-muted-foreground" />
                <h3 className="text-lg font-semibold text-foreground">
                  {BETA_FEATURE_NAMES[featureKey]}
                </h3>
                <span className="px-2 py-0.5 text-xs font-medium bg-yellow-100 text-yellow-800 rounded-full">
                  BETA
                </span>
              </div>
              <p className="text-sm text-muted-foreground">
                {BETA_FEATURE_DESCRIPTIONS[featureKey]}
              </p>
            </div>

            <div className="shrink-0">
              <Switch
                aria-label={BETA_FEATURE_NAMES[featureKey]}
                checked={betaFeatures[featureKey]}
                onCheckedChange={(checked) => toggleBetaFeature(featureKey, checked)}
              />
            </div>
          </div>
        </div>
      ))}

      {/* Info Box */}
      <div className="p-4 bg-primary/5 border border-primary/20 rounded-lg">
        <p className="text-sm text-primary">
          <strong>Note:</strong> Your choices are saved for future use. Existing meetings remain unaffected.
        </p>
      </div>
    </div>
  );
}
