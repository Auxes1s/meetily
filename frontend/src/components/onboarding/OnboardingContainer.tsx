import React from 'react';
import { ChevronLeft, ChevronRight } from 'lucide-react';
import { cn } from '@/lib/utils';
import { ProgressIndicator } from './shared/ProgressIndicator';
import { useOnboarding } from '@/contexts/OnboardingContext';
import type { OnboardingContainerProps } from '@/types/onboarding';

export function OnboardingContainer({
  title,
  description,
  children,
  step,
  totalSteps = 5,
  stepOffset = 0,
  hideProgress = false,
  className,
  showNavigation = false,
  onNext,
  onPrevious,
  canGoNext = true,
  canGoPrevious = true,
}: OnboardingContainerProps) {
  const { goToStep, goPrevious, goNext } = useOnboarding();

  const handlePrevious = () => {
    if (onPrevious) {
      onPrevious();
    } else {
      goPrevious();
    }
  };

  const handleNext = () => {
    if (onNext) {
      onNext();
    } else {
      goNext();
    }
  };

  const handleStepClick = (s: number) => {
    goToStep(s + stepOffset);
  };

  return (
    <div className="fixed inset-0 bg-background flex items-center justify-center z-50 overflow-hidden">
      <div className={cn('w-full max-w-2xl h-full max-h-screen min-h-0 flex flex-col px-4 py-6 sm:px-8 sm:py-10', className)}>
        {/* Progress Indicator with Navigation - Fixed */}
        {step && !hideProgress && (
          <div className="mb-2 relative flex-shrink-0">
            {/* Navigation Buttons */}
            {showNavigation && (
              <div className="absolute top-1/2 -translate-y-1/2 left-0 right-0 flex justify-between pointer-events-none">
                <button
                  aria-label="Previous setup step"
                  onClick={handlePrevious}
                  disabled={!canGoPrevious || step === 1}
                  className={cn(
                    'pointer-events-auto w-9 h-9 rounded-xl bg-card border border-border flex items-center justify-center transition-colors duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
                    canGoPrevious && step !== 1
                      ? 'hover:bg-muted text-foreground'
                      : 'opacity-0 cursor-not-allowed'
                  )}
                >
                  <ChevronLeft className="w-4 h-4" />
                </button>

                <button
                  aria-label="Next setup step"
                  onClick={handleNext}
                  disabled={!canGoNext || step === totalSteps}
                  className={cn(
                    'pointer-events-auto w-9 h-9 rounded-xl bg-card border border-border flex items-center justify-center transition-colors duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
                    canGoNext && step !== totalSteps
                      ? 'hover:bg-muted text-foreground'
                      : 'opacity-0 cursor-not-allowed'
                  )}
                >
                  <ChevronRight className="w-4 h-4" />
                </button>
              </div>
            )}

            {/* Progress Indicator */}
            <ProgressIndicator current={step} total={totalSteps} onStepClick={handleStepClick} />
          </div>
        )}

        {/* Header - Fixed */}
        <div className="mb-7 text-center space-y-3 flex-shrink-0">
          <h1 className="text-3xl sm:text-4xl font-semibold tracking-tight text-foreground animate-fade-in-up">{title}</h1>
          {description && (
            <p className="text-sm sm:text-base leading-relaxed text-muted-foreground max-w-md mx-auto animate-fade-in-up delay-75">
              {description}
            </p>
          )}
        </div>

        {/* Content - Scrollable */}
        <div className="flex-1 min-h-0 overflow-y-auto custom-scrollbar px-1 pb-4">
          <div className="space-y-6">{children}</div>
        </div>
      </div>
    </div>
  );
}
