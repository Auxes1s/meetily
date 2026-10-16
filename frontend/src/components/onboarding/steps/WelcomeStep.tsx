import React, { useRef, useState } from 'react';
import { Lock, Sparkles, Cpu, NotebookPen } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { OnboardingContainer } from '../OnboardingContainer';
import { useOnboarding } from '@/contexts/OnboardingContext';
import { toast } from 'sonner';

export function WelcomeStep() {
  const { goNext, completeNotesOnlySetup } = useOnboarding();
  const [isCompleting, setIsCompleting] = useState(false);
  const completing = useRef(false);

  const useNotesOnly = async () => {
    if (completing.current) return;
    completing.current = true;
    setIsCompleting(true);
    try {
      await completeNotesOnlySetup();
      window.location.reload();
    } catch (error) {
      toast.error('Could not finish setup', { description: String(error) });
      completing.current = false;
      setIsCompleting(false);
    }
  };

  const features = [
    {
      icon: Lock,
      title: 'Local processing by default',
    },
    {
      icon: Sparkles,
      title: 'Intelligent summaries & insights',
    },
    {
      icon: Cpu,
      title: 'Works offline after model downloads',
    },
  ];

  return (
    <OnboardingContainer
      title="Welcome to Meetily"
      description="Record. Transcribe. Summarize. All on your device."
      step={1}
      hideProgress={true}
    >
      <div className="flex flex-col items-center space-y-7">
        <div aria-hidden="true" className="glass-surface flex h-16 w-16 items-center justify-center rounded-3xl border border-primary/15 text-primary">
          <NotebookPen className="h-7 w-7" />
        </div>

        {/* Features Card */}
        <div className="paper-surface w-full max-w-md rounded-3xl border border-border p-6 space-y-5">
          {features.map((feature, index) => {
            const Icon = feature.icon;
            return (
              <div key={index} className="flex items-start gap-3">
                <div className="flex-shrink-0 mt-0.5">
                  <div className="w-8 h-8 rounded-xl bg-primary/10 flex items-center justify-center">
                    <Icon className="w-4 h-4 text-primary" />
                  </div>
                </div>
                <p className="pt-1 text-sm text-foreground leading-relaxed">{feature.title}</p>
              </div>
            );
          })}
        </div>

        {/* CTA Section */}
        <div className="w-full max-w-xs space-y-3">
          <Button
            onClick={goNext}
            disabled={isCompleting}
            className="w-full h-11"
          >
            Get Started
          </Button>
          <Button variant="outline" className="w-full" disabled={isCompleting} onClick={useNotesOnly}>
            {isCompleting ? 'Finishing setup…' : 'Use notes only'}
          </Button>
          <p className="text-xs text-center text-muted-foreground">Notes need no downloads. AI setup time depends on your connection.</p>
        </div>
      </div>
    </OnboardingContainer>
  );
}
