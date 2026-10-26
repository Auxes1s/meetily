import React, { useEffect } from 'react';
import { useOnboarding } from '@/contexts/OnboardingContext';
import {
  WelcomeStep,
  PermissionsStep,
  DownloadProgressStep,
  SetupOverviewStep,
} from './steps';

interface OnboardingFlowProps {
  onComplete: () => void;
}

export function OnboardingFlow({ onComplete }: OnboardingFlowProps) {
  const { currentStep, goToStep } = useOnboarding();
  const [isMac, setIsMac] = React.useState<boolean | null>(null);

  useEffect(() => {
    // Check if running on macOS
    const checkPlatform = async () => {
      try {
        // Dynamic import to avoid SSR issues if any
        const { platform } = await import('@tauri-apps/plugin-os');
        setIsMac(platform() === 'macos');
      } catch (e) {
        console.error('Failed to detect platform:', e);
        // Fallback
        setIsMac(navigator.userAgent.includes('Mac'));
      }
    };
    checkPlatform();
  }, []);

  useEffect(() => {
    // Older/resumed profiles can contain the macOS-only final step on Windows.
    if (isMac === false && currentStep === 4) goToStep(3);
  }, [isMac, currentStep, goToStep]);

  // 4-Step Onboarding Flow (System-Recommended Models):
  // Step 1: Welcome - Introduce Meetily features
  // Step 2: Setup Overview - Database initialization + show recommended downloads
  // Step 3: Download Progress - Download Parakeet + Summary Model (auto-selected based on platform/RAM)
  // Step 4: Permissions - Request mic + system audio (macOS only)

  return (
    <div className="onboarding-flow">
      {currentStep === 1 && <WelcomeStep />}
      {currentStep === 2 && <SetupOverviewStep />}
      {currentStep === 3 && <DownloadProgressStep />}
      {currentStep === 4 && isMac === true && <PermissionsStep />}
      {currentStep === 4 && isMac !== true && (
        <div role="status" className="fixed inset-0 flex items-center justify-center bg-background">
          Restoring setup…
        </div>
      )}
    </div>
  );
}
