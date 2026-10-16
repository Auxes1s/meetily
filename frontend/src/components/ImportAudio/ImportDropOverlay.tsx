import React from 'react';
import { Upload } from 'lucide-react';
import { getAudioFormatsDisplayList } from '@/constants/audioFormats';

interface ImportDropOverlayProps {
  visible: boolean;
}

export function ImportDropOverlay({ visible }: ImportDropOverlayProps) {
  if (!visible) return null;

  return (
    <div
      role="status"
      aria-live="polite"
      className="fixed inset-0 z-50 bg-background/80 backdrop-blur-sm
                 flex items-center justify-center p-5 pointer-events-none
                 transition-opacity duration-200"
    >
      <div className="glass-surface w-full max-w-lg border-2 border-dashed border-primary/30 rounded-3xl
                      p-7 sm:p-12 text-center shadow-xl">
        <Upload className="h-12 w-12 text-primary mx-auto mb-5" />
        <p className="text-xl font-medium text-foreground">Drop audio file to import</p>
        <p className="text-sm leading-relaxed text-muted-foreground mt-2">{getAudioFormatsDisplayList()}</p>
      </div>
    </div>
  );
}
